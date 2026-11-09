//! Oredictionificator (`TileEntityOredictionificator`, `OredictionificatorItemFilter`).
//!
//! A machine that unifies items between ore dictionary / tag equivalents (e.g., converting any copper ingot to a chosen mod's copper ingot).
//! Operates passively without consuming energy.
//!
//! Java references:
//! - `TileEntityOredictionificator.java:57`: config `RelativeSide.RIGHT` default output
//! - `TileEntityOredictionificator.java:68`: `inputSlot` at (56, 115)
//! - `TileEntityOredictionificator.java:69`: `outputSlot` at (164, 115)
//! - `TileEntityOredictionificator.java:82-96`: converts 1 item per tick for matching enabled filter
//! - `TileEntityOredictionificator.java:128-135`: valid filter prefixes (`ores`, `ingots`, `dusts`, `gems`, `storage_blocks`, `raw_materials`, `nuggets`)
//! - `TileEntityOredictionificator.java:182`: `supportsMode(RedstoneControl mode) -> true`
//! - `OredictionificatorItemFilter.java:75`: `FilterType.OREDICTIONIFICATOR_ITEM_FILTER`

use super::*;

pub const BLOCK: &str = "mekanism:oredictionificator";

pub const INPUT_SLOT: usize = 0;
pub const OUTPUT_SLOT: usize = 1;
pub const TOTAL_SLOTS: usize = 2;

/// Valid tag prefix paths (Java: `MekanismConfig.general.validOredictionificatorFilters`).
pub const VALID_PREFIXES: [&str; 7] = [
    "ores",
    "ingots",
    "dusts",
    "gems",
    "storage_blocks",
    "raw_materials",
    "nuggets",
];

/// Item filter for the Oredictionificator.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemFilter {
    pub enabled: bool,
    pub filter_tag: String,
    pub selected_output: String,
}

impl ItemFilter {
    pub fn new(filter_tag: String, selected_output: String) -> Self {
        ItemFilter {
            enabled: true,
            filter_tag,
            selected_output,
        }
    }

    /// Checks if a tag is allowed under Mekanism's valid prefixes (Java: `TileEntityOredictionificator.java:128-135`).
    pub fn is_valid_tag(tag: &str) -> bool {
        // Must be in "c:" namespace or similar common tag convention
        let path = if let Some(stripped) = tag.strip_prefix("c:") {
            stripped
        } else if let Some(stripped) = tag.strip_prefix("forge:") {
            stripped
        } else {
            tag
        };

        VALID_PREFIXES.iter().any(|prefix| path.starts_with(prefix))
    }
}

/// State of the Oredictionificator.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct State {
    pub filters: Vec<ItemFilter>,
    pub redstone_control: redstone::Control,
    pub did_process: bool,
    pub auto_eject: bool,
}

pub fn encode(state: &State) -> Vec<u8> {
    let mut out = vec![
        state.redstone_control as u8,
        u8::from(state.did_process),
        u8::from(state.auto_eject),
    ];
    out.extend((state.filters.len() as u16).to_le_bytes());
    for f in &state.filters {
        out.push(u8::from(f.enabled));
        let tb = f.filter_tag.as_bytes();
        out.extend((tb.len() as u16).to_le_bytes());
        out.extend(tb);
        let ob = f.selected_output.as_bytes();
        out.extend((ob.len() as u16).to_le_bytes());
        out.extend(ob);
    }
    out
}

pub fn decode(bytes: &[u8]) -> State {
    if bytes.len() < 5 {
        return State::default();
    }
    let redstone_control = redstone::Control::from_byte(bytes[0]);
    let did_process = bytes[1] == 1;
    let auto_eject = bytes[2] == 1;
    let filter_count = u16::from_le_bytes([bytes[3], bytes[4]]) as usize;

    let mut cursor = 5;
    let mut filters = Vec::with_capacity(filter_count);

    for _ in 0..filter_count {
        if cursor >= bytes.len() { break; }
        let enabled = bytes[cursor] == 1;
        cursor += 1;

        if cursor + 2 > bytes.len() { break; }
        let tlen = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
        cursor += 2;
        if cursor + tlen > bytes.len() { break; }
        let filter_tag = String::from_utf8_lossy(&bytes[cursor..cursor + tlen]).to_string();
        cursor += tlen;

        if cursor + 2 > bytes.len() { break; }
        let olen = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
        cursor += 2;
        if cursor + olen > bytes.len() { break; }
        let selected_output = String::from_utf8_lossy(&bytes[cursor..cursor + olen]).to_string();
        cursor += olen;

        filters.push(ItemFilter {
            enabled,
            filter_tag,
            selected_output,
        });
    }

    State {
        filters,
        redstone_control,
        did_process,
        auto_eject,
    }
}

/// Output item side configuration: default RelativeSide.RIGHT (Java: `TileEntityOredictionificator.java:57`).
pub fn default_item_faces(facing: &str) -> Vec<Vec<u32>> {
    let front = laser::face_of(facing);
    // RelativeSide.RIGHT:
    // North (2) -> East (5)
    // South (3) -> West (4)
    // West (4) -> North (2)
    // East (5) -> South (3)
    let right = match front {
        laser::NORTH => laser::EAST,
        laser::SOUTH => laser::WEST,
        laser::WEST => laser::NORTH,
        laser::EAST => laser::SOUTH,
        _ => laser::EAST,
    };

    let mut faces = vec![Vec::new(); 6];
    // Input accepted from any side or top/back
    faces[laser::UP] = vec![INPUT_SLOT as u32];
    // Output ejected to right
    faces[right] = vec![OUTPUT_SLOT as u32];
    faces
}

/// Pure conversion step for one tick (Java: `TileEntityOredictionificator.java:82-96`).
pub fn step_conversion(
    filters: &[ItemFilter],
    input_item: &mut Option<ItemStack>,
    output_item: &mut Option<ItemStack>,
    get_item_tags: impl Fn(&str) -> Vec<String>,
) -> bool {
    let Some(ref mut input) = input_item else {
        return false;
    };
    if input.count == 0 {
        return false;
    }

    let tags = get_item_tags(&input.item);

    // Find first enabled filter matching any of the item's tags (Java: `TileEntityOredictionificator.java:152-167`)
    for filter in filters.iter().filter(|f| f.enabled) {
        if tags.iter().any(|t| t == &filter.filter_tag) {
            let target = &filter.selected_output;
            if target.is_empty() {
                continue;
            }

            // Check if output slot can accept the result
            match output_item {
                None => {
                    *output_item = Some(ItemStack {
                        item: target.clone(),
                        count: 1,
                        components: String::new(),
                    });
                    input.count -= 1;
                    if input.count == 0 {
                        *input_item = None;
                    }
                    return true;
                }
                Some(ref mut out) => {
                    if out.item == *target && out.count < 64 {
                        out.count += 1;
                        input.count -= 1;
                        if input.count == 0 {
                            *input_item = None;
                        }
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// Host API bridge for retrieving item tags in WASM runtime.
// FORK-23: Item tags inspection API in WASM
pub fn host_get_item_tags(item_id: &str) -> Vec<String> {
    let _ = item_id;
    Vec::new()
}

impl Mekanism {
    /// Server tick handler for Oredictionificators.
    pub(crate) fn run_oredictionificators(
        &mut self,
        machines: &[&BlockEntity],
    ) -> Vec<BlockEntityUpdate> {
        let mut out = Vec::new();
        for e in machines {
            let block = e.state.split('[').next().unwrap_or("");
            if block != BLOCK {
                continue;
            }

            let mut state = decode(&e.data);
            let powered = world::get_redstone(&e.dimension, e.pos.x, e.pos.y, e.pos.z) > 0;
            if !state.redstone_control.can_function(powered, false) {
                continue;
            }

            let mut items: Vec<Option<ItemStack>> = (0..TOTAL_SLOTS)
                .map(|i| e.items.get(i).cloned().flatten())
                .collect();

            let mut input = items[INPUT_SLOT].clone();
            let mut output = items[OUTPUT_SLOT].clone();

            let converted = step_conversion(
                &state.filters,
                &mut input,
                &mut output,
                host_get_item_tags,
            );

            state.did_process = converted;

            if converted {
                items[INPUT_SLOT] = input;
                items[OUTPUT_SLOT] = output;

                out.push(BlockEntityUpdate {
                    dimension: e.dimension.clone(),
                    pos: e.pos,
                    data: encode(&state),
                    items: Some(items),
                    energy: None,
                    energy_faces: None,
                    tanks: None,
                    tank_faces: None,
                    menu_data: Some(vec![i32::from(state.did_process)]),
                    transmitter_faces: None,
                });
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slots_and_prefixes() {
        // Java: TileEntityOredictionificator.java:68-69
        assert_eq!(INPUT_SLOT, 0);
        assert_eq!(OUTPUT_SLOT, 1);
        assert_eq!(TOTAL_SLOTS, 2);

        // Java: TileEntityOredictionificator.java:128-135
        assert!(ItemFilter::is_valid_tag("c:ingots/copper"));
        assert!(ItemFilter::is_valid_tag("c:ores/tin"));
        assert!(ItemFilter::is_valid_tag("c:dusts/iron"));
        assert!(ItemFilter::is_valid_tag("c:gems/diamond"));
        assert!(ItemFilter::is_valid_tag("c:storage_blocks/steel"));
        assert!(ItemFilter::is_valid_tag("c:raw_materials/osmium"));
        assert!(ItemFilter::is_valid_tag("c:nuggets/gold"));

        // Invalid prefix
        assert!(!ItemFilter::is_valid_tag("c:tools/pickaxes"));
        assert!(!ItemFilter::is_valid_tag("minecraft:planks"));
    }

    #[test]
    fn test_step_conversion_into_empty_output() {
        let filters = vec![ItemFilter {
            enabled: true,
            filter_tag: "c:ingots/copper".into(),
            selected_output: "mekanism:ingot_copper".into(),
        }];

        let mut input = Some(ItemStack {
            item: "minecraft:copper_ingot".into(),
            count: 5,
            components: String::new(),
        });
        let mut output = None;

        let processed = step_conversion(&filters, &mut input, &mut output, |_item| {
            vec!["c:ingots/copper".into()]
        });

        assert!(processed);
        assert_eq!(input.unwrap().count, 4);
        assert_eq!(output.unwrap().item, "mekanism:ingot_copper");
        assert_eq!(output.unwrap().count, 1);
    }

    #[test]
    fn test_step_conversion_stacks_in_output() {
        let filters = vec![ItemFilter {
            enabled: true,
            filter_tag: "c:ingots/copper".into(),
            selected_output: "mekanism:ingot_copper".into(),
        }];

        let mut input = Some(ItemStack {
            item: "minecraft:copper_ingot".into(),
            count: 1,
            components: String::new(),
        });
        let mut output = Some(ItemStack {
            item: "mekanism:ingot_copper".into(),
            count: 10,
            components: String::new(),
        });

        let processed = step_conversion(&filters, &mut input, &mut output, |_item| {
            vec!["c:ingots/copper".into()]
        });

        assert!(processed);
        assert!(input.is_none()); // Consumed the last item
        assert_eq!(output.unwrap().count, 11);
    }

    #[test]
    fn test_disabled_filter_is_ignored() {
        let filters = vec![
            ItemFilter {
                enabled: false, // Disabled!
                filter_tag: "c:ingots/copper".into(),
                selected_output: "other_mod:copper_ingot".into(),
            },
            ItemFilter {
                enabled: true, // Fallback enabled filter
                filter_tag: "c:ingots/copper".into(),
                selected_output: "mekanism:ingot_copper".into(),
            },
        ];

        let mut input = Some(ItemStack {
            item: "minecraft:copper_ingot".into(),
            count: 1,
            components: String::new(),
        });
        let mut output = None;

        let processed = step_conversion(&filters, &mut input, &mut output, |_item| {
            vec!["c:ingots/copper".into()]
        });

        assert!(processed);
        // Produced mekanism:ingot_copper, not other_mod:copper_ingot
        assert_eq!(output.unwrap().item, "mekanism:ingot_copper");
    }

    #[test]
    fn test_state_serialization_roundtrip() {
        let state = State {
            filters: vec![
                ItemFilter::new("c:ingots/copper".into(), "mekanism:ingot_copper".into()),
                ItemFilter::new("c:dusts/iron".into(), "mekanism:dust_iron".into()),
            ],
            redstone_control: redstone::Control::Low,
            did_process: true,
            auto_eject: true,
        };

        let encoded = encode(&state);
        let decoded = decode(&encoded);

        assert_eq!(decoded.redstone_control, redstone::Control::Low);
        assert!(decoded.did_process);
        assert!(decoded.auto_eject);
        assert_eq!(decoded.filters.len(), 2);
        assert_eq!(decoded.filters[0], state.filters[0]);
        assert_eq!(decoded.filters[1], state.filters[1]);
    }
}

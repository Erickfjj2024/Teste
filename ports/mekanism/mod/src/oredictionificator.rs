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

/// The tags a filter may name (`MekanismConfig.general.validOredictionificatorFilters`, default `{c: [ingots/, ores/, dusts/, nuggets/, storage_blocks/,
/// raw_materials/]}`): in the `c` namespace, with a path that starts with one of these.
pub const VALID_NAMESPACE: &str = "c";
pub const VALID_PREFIXES: [&str; 6] = ["ingots/", "ores/", "dusts/", "nuggets/", "storage_blocks/", "raw_materials/"];

/// `isPossibleFilter`: whether a tag (`c:ores/iron`) may be filtered on.
pub fn is_possible_filter(tag: &str) -> bool {
    let Some((namespace, path)) = tag.split_once(':') else { return false };
    namespace == VALID_NAMESPACE && VALID_PREFIXES.iter().any(|prefix| path.starts_with(prefix))
}

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

    /// Whether a tag may be the filter of an oredictionificator (`isValidTarget`, without the check that the tag exists, which the game answers).
    pub fn is_valid_tag(tag: &str) -> bool {
        is_possible_filter(tag)
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

/// The result of the first filter that matches (`getResult`): the item's filterable tags in order, and for each the enabled filters in order; a filter
/// with no output is skipped and the search goes on.
pub fn result_of(filters: &[ItemFilter], tags: &[String]) -> Option<String> {
    for tag in tags.iter().filter(|t| is_possible_filter(t)) {
        for filter in filters.iter().filter(|f| f.enabled) {
            if &filter.filter_tag == tag && !filter.selected_output.is_empty() {
                return Some(filter.selected_output.clone());
            }
        }
    }
    None
}

/// Pure conversion step for one tick (Java: `TileEntityOredictionificator.onUpdateServer`, java:82-96): the first result found is the only one tried; it goes
/// into an empty output slot, or onto a stack of the same item below the item's own limit (`max_stack`). Says whether it converted.
pub fn step_conversion(
    filters: &[ItemFilter],
    input_item: &mut Option<ItemStack>,
    output_item: &mut Option<ItemStack>,
    get_item_tags: impl Fn(&str) -> Vec<String>,
    max_stack: impl Fn(&str) -> u32,
) -> bool {
    let Some(input) = input_item else { return false };
    if input.count == 0 {
        return false;
    }
    let tags = get_item_tags(&input.item);
    let Some(target) = result_of(filters, &tags) else { return false };
    match output_item {
        None => {
            *output_item = Some(ItemStack { item: target, count: 1, components: String::new() });
        }
        Some(out) if out.item == target && out.components.trim().is_empty() && out.count < max_stack(&target) => {
            out.count += 1;
        }
        Some(_) => return false,
    }
    input.count -= 1;
    if input.count == 0 {
        *input_item = None;
    }
    true
}

/// The tags an item is in, from the game (`recipe-types.item-tags`, API 30).
pub fn host_get_item_tags(item_id: &str) -> Vec<String> {
    horizon_mod_sdk::server::recipe_types::item_tags(item_id)
}

/// The numbers the screen reads: 0 it converted this tick, 1 the redstone control, 2 how many filters it has, 3 to 5 whether each of the first three is on.
pub fn menu_numbers(state: &State) -> Vec<i32> {
    let mut v = vec![i32::from(state.did_process), state.redstone_control as i32, state.filters.len() as i32];
    for i in 0..3 {
        v.push(state.filters.get(i).map_or(0, |f| i32::from(f.enabled)));
    }
    v
}

/// A button of its screen: a filter switched on or off (`TOGGLE_FILTER_STATE`, `BUTTON_TOGGLE_FILTER_BASE + index`), or the redstone tab.
pub const BUTTON_TOGGLE_FILTER_BASE: i32 = 7700;

impl Mekanism {
    /// Server tick handler for Oredictionificators (`onUpdateServer`).
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
            let key = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
            let powered = world::get_redstone(&e.dimension, e.pos.x, e.pos.y, e.pos.z) > 0;
            let last = self.last_power.insert(key, powered).unwrap_or(false);
            let mut items: Vec<Option<ItemStack>> = (0..TOTAL_SLOTS).map(|i| e.items.get(i).cloned().flatten()).collect();
            let mut converted = false;
            if state.redstone_control.can_function(powered, last) {
                let mut input = items[INPUT_SLOT].clone();
                let mut output = items[OUTPUT_SLOT].clone();
                converted = step_conversion(&state.filters, &mut input, &mut output, host_get_item_tags, |i| crate::bin::stack_size(i) as u32);
                if converted {
                    items[INPUT_SLOT] = input;
                    items[OUTPUT_SLOT] = output;
                }
            }
            let was = state.did_process;
            state.did_process = converted;
            let numbers = menu_numbers(&state);
            if converted || was != converted || e.menu_data != numbers {
                out.push(BlockEntityUpdate {
                    dimension: e.dimension.clone(),
                    pos: e.pos,
                    data: encode(&state),
                    items: if converted { Some(items) } else { None },
                    energy: None,
                    energy_faces: None,
                    tanks: None,
                    tank_faces: None,
                    menu_data: Some(numbers),
                    transmitter_faces: None,
                });
            }
        }
        out
    }

    /// A button of the screen: the redstone tab or a filter's switch.
    pub(crate) fn oredictionificator_button(&mut self, e: &BlockEntity, id: i32) -> Option<BlockEntityUpdate> {
        let mut state = decode(&e.data);
        if id == crate::redstone::NEXT {
            state.redstone_control = state.redstone_control.next();
        } else if id == crate::redstone::PREVIOUS {
            state.redstone_control = state.redstone_control.previous();
        } else if id >= BUTTON_TOGGLE_FILTER_BASE && ((id - BUTTON_TOGGLE_FILTER_BASE) as usize) < state.filters.len() {
            let f = &mut state.filters[(id - BUTTON_TOGGLE_FILTER_BASE) as usize];
            f.enabled = !f.enabled;
        } else {
            return None;
        }
        Some(BlockEntityUpdate {
            dimension: e.dimension.clone(),
            pos: e.pos,
            data: encode(&state),
            items: None,
            energy: None,
            energy_faces: None,
            tanks: None,
            tank_faces: None,
            menu_data: Some(menu_numbers(&state)),
            transmitter_faces: None,
        })
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
        // `gems/` is not in the default list of valid filters (`GeneralConfig:243-244`).
        assert!(!ItemFilter::is_valid_tag("c:gems/diamond"));
        assert!(!ItemFilter::is_valid_tag("forge:ores/tin"));
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
        }, |_| 64);

        assert!(processed);
        assert_eq!(input.unwrap().count, 4);
        let out = output.unwrap();
        assert_eq!(out.item, "mekanism:ingot_copper");
        assert_eq!(out.count, 1);
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
        }, |_| 64);

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
        }, |_| 64);

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

    #[test]
    fn test_the_items_tag_order_decides_and_the_first_result_is_the_only_one_tried() {
        let filters = vec![
            ItemFilter::new("c:ores/iron".into(), "mod_a:iron_ore".into()),
            ItemFilter::new("c:raw_materials/iron".into(), "mod_b:raw_iron".into()),
        ];
        // Tag-major: the item's first tag that a filter wants decides, whatever the filters' order.
        let tags = vec!["c:raw_materials/iron".to_owned(), "c:ores/iron".to_owned()];
        assert_eq!(result_of(&filters, &tags).as_deref(), Some("mod_b:raw_iron"));
        // A tag outside the valid list is not looked at.
        let odd = vec!["minecraft:logs".to_owned()];
        assert_eq!(result_of(&[ItemFilter::new("minecraft:logs".into(), "x:y".into())], &odd), None);
        // The first result is the only one tried: a full output of another item blocks it.
        let mut input = Some(ItemStack { item: "a:b".into(), count: 3, components: String::new() });
        let mut output = Some(ItemStack { item: "other:thing".into(), count: 1, components: String::new() });
        let tags2 = tags.clone();
        assert!(!step_conversion(&filters, &mut input, &mut output, move |_| tags2.clone(), |_| 64));
        assert_eq!(input.unwrap().count, 3);
    }

    #[test]
    fn test_output_stops_at_the_items_own_limit() {
        let filters = vec![ItemFilter::new("c:ingots/tin".into(), "mod:tin_ingot".into())];
        let mut input = Some(ItemStack { item: "a:b".into(), count: 1, components: String::new() });
        let mut output = Some(ItemStack { item: "mod:tin_ingot".into(), count: 16, components: String::new() });
        assert!(!step_conversion(&filters, &mut input, &mut output, |_| vec!["c:ingots/tin".into()], |_| 16));
        assert!(step_conversion(&filters, &mut input, &mut output, |_| vec!["c:ingots/tin".into()], |_| 64));
        assert_eq!(output.unwrap().count, 17);
    }

    #[test]
    fn test_screen_numbers_and_filter_switch() {
        let mut state = State { filters: vec![ItemFilter::new("c:ores/tin".into(), "m:t".into())], ..State::default() };
        assert_eq!(menu_numbers(&state), vec![0, 0, 1, 1, 0, 0]);
        state.filters[0].enabled = false;
        assert_eq!(menu_numbers(&state)[3], 0);
    }
}

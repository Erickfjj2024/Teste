//! Formulaic Assemblicator (`TileEntityFormulaicAssemblicator`) and Crafting Formula (`ItemCraftingFormula`).
//!
//! A 3x3 autocrafting machine and workbench with 35 slots total:
//! - 1 formula slot (`formulaSlot`, slot 0) at (6, 26), accepts only `mekanism:crafting_formula`
//! - 18 input slots (`inputSlots`, slots 1..=18, 2 rows of 9) at (8 + x*18, 98 + y*18)
//! - 9 crafting grid slots (`craftingGridSlots`, slots 19..=27, 3x3) at (26 + x*18, 17 + y*18)
//! - 6 output slots (`outputSlots`, slots 28..=33, 3 rows of 2) at (116 + x*18, 17 + y*18)
//! - 1 energy slot (`energySlot`, slot 34) at (152, 76)
//!
//! Features:
//! - Manual crafting mode and automatic crafting mode (`autoMode`)
//! - Formula encoding onto `ItemCraftingFormula` with `FormulaAttachment`
//! - Stock control mode (`stockControl`) restricting 18 input slots to recipe ingredients
//! - Upgrades: Speed (reduces ticks from 40 down to 4) and Energy (scales capacity and reduces usage)
//! - Redstone control modes: Disabled, High, Low, Pulse
//!
//! Java source of truth:
//! - `mekanism.common.tile.machine.TileEntityFormulaicAssemblicator`
//! - `mekanism.common.content.assemblicator.RecipeFormula`
//! - `mekanism.common.inventory.container.tile.FormulaicAssemblicatorContainer`
//! - `mekanism.client.gui.machine.GuiFormulaicAssemblicator`
//! - `mekanism.common.item.ItemCraftingFormula`
//! - `mekanism.common.attachments.FormulaAttachment`

use super::*;
use std::collections::HashMap;

pub const BLOCK: &str = "mekanism:formulaic_assemblicator";
pub const CRAFTING_FORMULA: &str = "mekanism:crafting_formula";
pub const FORMULA_COMPONENT: &str = "mekanism:formula_holder";

/// Total slots in the container: 1 formula + 18 inputs + 9 grid + 6 outputs + 1 energy = 35.
/// Java: `TileEntityFormulaicAssemblicator.java:154-190`.
pub const TOTAL_SLOTS: usize = 35;

/// Slot 0: Formula slot (accepts only `mekanism:crafting_formula`).
/// Java: `TileEntityFormulaicAssemblicator.java:154`.
pub const SLOT_FORMULA: usize = 0;

/// Slots 1..=18: 18 Input / stock buffer slots (2 rows of 9).
/// Java: `TileEntityFormulaicAssemblicator.java:156-175`.
pub const SLOT_INPUT_START: usize = 1;
pub const INPUT_SLOTS: usize = 18;

/// Slots 19..=27: 9 Crafting grid slots (3x3).
/// Java: `TileEntityFormulaicAssemblicator.java:176-182`.
pub const SLOT_CRAFT_START: usize = 19;
pub const CRAFT_SLOTS: usize = 9;

/// Slots 28..=33: 6 Output buffer slots (3 rows of 2).
/// Java: `TileEntityFormulaicAssemblicator.java:183-188`.
pub const SLOT_OUTPUT_START: usize = 28;
pub const OUTPUT_SLOTS: usize = 6;

/// Slot 34: Energy slot.
/// Java: `TileEntityFormulaicAssemblicator.java:190`.
pub const SLOT_ENERGY: usize = 34;

/// Base ticks required per craft operation: 40 ticks (2.0s).
/// Java: `TileEntityFormulaicAssemblicator.java:80`: BASE_TICKS_REQUIRED = 2 * TICKS_PER_SECOND = 40.
pub const BASE_TICKS_REQUIRED: u32 = 40;

/// Base energy usage: 100 Joules/t = 40 FE/t (`fe(100)`).
/// Java: `UsageConfig.formulaicAssemblicator = 100L`.
pub const BASE_ENERGY_USAGE: u64 = 40;

/// Base energy buffer capacity: 40,000 Joules = 16,000 FE (`fe(40_000)`).
/// Java: `StorageConfig.formulaicAssemblicator = 40_000L`.
pub const BASE_STORAGE: u64 = 16_000;

/// GUI Buttons
pub const BUTTON_ENCODE_FORMULA: i32 = 7900;
pub const BUTTON_STOCK_CONTROL: i32 = 7901;
pub const BUTTON_FILL_EMPTY: i32 = 7902;
pub const BUTTON_CRAFT_SINGLE: i32 = 7903;
pub const BUTTON_CRAFT_ALL: i32 = 7904;
pub const BUTTON_AUTO_MODE: i32 = 7905;

// ============================================================================
// 1. CRAFTING FORMULA & FORMULA ATTACHMENT
// ============================================================================

/// What a Crafting Formula item stores (`FormulaAttachment.java:21-31`).
/// Contains a 9-element array representing the 3x3 grid, plus an `invalid` flag.
#[derive(Clone, Debug, PartialEq)]
pub struct FormulaAttachment {
    pub items: [Option<ItemStack>; 9],
    pub invalid: bool,
}

impl FormulaAttachment {
    pub const EMPTY: Self = Self {
        items: [None, None, None, None, None, None, None, None, None],
        invalid: false,
    };

    pub fn new(items: [Option<ItemStack>; 9], invalid: bool) -> Self {
        Self { items, invalid }
    }

    /// Java: `FormulaAttachment.java:57-61`: whether all 9 stacks are empty.
    pub fn is_empty(&self) -> bool {
        self.items.iter().all(|s| s.as_ref().map_or(true, |it| it.count == 0 || it.item.is_empty()))
    }

    /// Java: `FormulaAttachment.java:71-75`: whether any stack is non-empty.
    pub fn has_items(&self) -> bool {
        !self.is_empty()
    }

    /// Java: `FormulaAttachment.java:48-54`: returns copy marked invalid.
    pub fn as_invalid(&self) -> Self {
        Self {
            items: self.items.clone(),
            invalid: true,
        }
    }

    /// Serializes this attachment into an SNBT string for `mekanism:formula_holder`.
    pub fn encode_snbt(&self) -> String {
        let mut out = String::from("{\"items\":[");
        for (i, opt) in self.items.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            match opt {
                Some(st) if st.count > 0 && !st.item.is_empty() => {
                    out.push_str(&format!("{{\"id\":\"{}\",\"count\":{}}}", st.item, st.count));
                }
                _ => {
                    out.push_str("{\"id\":\"\",\"count\":0}");
                }
            }
        }
        out.push_str(&format!("],\"invalid\":{}}}", if self.invalid { "1b" } else { "0b" }));
        out
    }

    /// Decodes an attachment from an SNBT/JSON component string.
    pub fn decode_snbt(snbt: &str) -> Option<Self> {
        let text = snbt.trim();
        if text.is_empty() || text == "{}" {
            return Some(Self::EMPTY);
        }
        // Check for invalid flag
        let invalid = text.contains("\"invalid\":true") || text.contains("\"invalid\":1") || text.contains("invalid:1b");

        // Parse items array: look for items within `items:[...]`
        let mut items: [Option<ItemStack>; 9] = Default::default();
        let items_start = text.find("\"items\":[")?.checked_add(9)?;
        let items_slice = &text[items_start..];
        let items_end = items_slice.find(']')?;
        let array_str = &items_slice[..items_end];

        let mut idx = 0;
        let mut cur = array_str;
        while let Some(open) = cur.find('{') {
            if idx >= 9 {
                break;
            }
            let close = cur[open..].find('}')? + open;
            let elem = &cur[open + 1..close];

            let mut item_id = String::new();
            let mut count = 0u32;

            for part in elem.split(',') {
                let part = part.trim();
                if let Some(id_part) = part.strip_prefix("\"id\":") {
                    item_id = id_part.trim().trim_matches('"').to_string();
                } else if let Some(id_part) = part.strip_prefix("id:") {
                    item_id = id_part.trim().trim_matches('"').to_string();
                } else if let Some(cnt_part) = part.strip_prefix("\"count\":") {
                    count = cnt_part.trim().trim_end_matches('b').parse().unwrap_or(0);
                } else if let Some(cnt_part) = part.strip_prefix("count:") {
                    count = cnt_part.trim().trim_end_matches('b').parse().unwrap_or(0);
                }
            }

            if !item_id.is_empty() && count > 0 {
                items[idx] = Some(ItemStack {
                    item: item_id,
                    count,
                    components: String::new(),
                });
            }
            idx += 1;
            cur = &cur[close + 1..];
        }

        Some(Self { items, invalid })
    }
}

/// Helper methods for `ItemCraftingFormula` (`ItemCraftingFormula.java`).
pub struct CraftingFormulaItem;

impl CraftingFormulaItem {
    /// Gets the formula attachment stored in the item's components.
    pub fn get_attachment(stack: &ItemStack) -> FormulaAttachment {
        if let Some(entries) = config::split_components(&stack.components) {
            for (key, range) in entries {
                if key == FORMULA_COMPONENT {
                    if let Some(att) = FormulaAttachment::decode_snbt(&stack.components[range]) {
                        return att;
                    }
                }
            }
        }
        FormulaAttachment::EMPTY
    }

    /// Sets the formula attachment on the item's components.
    pub fn set_attachment(stack: &mut ItemStack, att: &FormulaAttachment) {
        let pair = format!("\"{FORMULA_COMPONENT}\":{}", att.encode_snbt());
        if stack.components.trim().is_empty() || stack.components.trim() == "{}" {
            stack.components = format!("{{{pair}}}");
            return;
        }
        if let Some(entries) = config::split_components(&stack.components) {
            for (key, range) in &entries {
                if key == FORMULA_COMPONENT {
                    stack.components.replace_range(range.clone(), &att.encode_snbt());
                    return;
                }
            }
            let at = stack.components.rfind('}').unwrap_or(stack.components.len());
            stack.components.insert_str(at, &format!(",{pair}"));
        } else {
            stack.components = format!("{{{pair}}}");
        }
    }

    /// Java: `ItemCraftingFormula.java:57-65`: Name calculation.
    /// - Empty: "Crafting Formula"
    /// - Encoded valid: "Crafting Formula (Encoded)" (dark green)
    /// - Encoded invalid: "Crafting Formula (Invalid)" (dark red)
    pub fn display_name(stack: &ItemStack) -> String {
        let att = Self::get_attachment(stack);
        if att.has_items() {
            if att.invalid {
                "Crafting Formula (Invalid)".to_string()
            } else {
                "Crafting Formula (Encoded)".to_string()
            }
        } else {
            "Crafting Formula".to_string()
        }
    }

    /// Java: `ItemCraftingFormula.java:31-39`: Tooltip ingredients list.
    /// Aggregates non-empty items by item type and count.
    pub fn tooltip_ingredients(stack: &ItemStack) -> Vec<(String, u32)> {
        let att = Self::get_attachment(stack);
        let mut map: HashMap<String, u32> = HashMap::new();
        for it in att.items.iter().flatten() {
            if !it.item.is_empty() && it.count > 0 {
                *map.entry(it.item.clone()).or_insert(0) += it.count;
            }
        }
        let mut list: Vec<(String, u32)> = map.into_iter().collect();
        list.sort_by(|a, b| a.0.cmp(&b.0));
        list
    }

    /// Java: `ItemCraftingFormula.java:44-53`: Sneak + right-click in air clears formula.
    pub fn clear(stack: &mut ItemStack) {
        Self::set_attachment(stack, &FormulaAttachment::EMPTY);
    }
}

// ============================================================================
// 2. RECIPE FORMULA & CRAFTING RESOLUTION
// ============================================================================

/// Recipe representation for 3x3 crafting matrix.
#[derive(Clone, Debug, PartialEq)]
pub enum CraftingPattern {
    Shaped {
        width: usize,
        height: usize,
        grid: Vec<Option<String>>,
    },
    Shapeless {
        ingredients: Vec<String>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct CraftingRecipeDef {
    pub id: String,
    pub pattern: CraftingPattern,
    pub output: ItemStack,
    pub remaining: Vec<ItemStack>,
}

impl CraftingRecipeDef {
    /// Matches this recipe against a 3x3 crafting grid.
    pub fn matches(&self, grid: &[Option<ItemStack>; 9]) -> bool {
        match &self.pattern {
            CraftingPattern::Shapeless { ingredients } => {
                let mut grid_items: Vec<String> = grid.iter()
                    .filter_map(|s| s.as_ref())
                    .filter(|s| !s.item.is_empty() && s.count > 0)
                    .map(|s| s.item.clone())
                    .collect();
                if grid_items.len() != ingredients.len() {
                    return false;
                }
                grid_items.sort();
                let mut req = ingredients.clone();
                req.sort();
                grid_items == req
            }
            CraftingPattern::Shaped { width, height, grid: pat } => {
                if *width > 3 || *height > 3 {
                    return false;
                }
                // Try each valid top-left offset (x in 0..=3-w, y in 0..=3-h)
                for off_y in 0..=(3 - height) {
                    for off_x in 0..=(3 - width) {
                        let mut ok = true;
                        for r in 0..3 {
                            for c in 0..3 {
                                let grid_idx = r * 3 + c;
                                let grid_item = grid[grid_idx].as_ref().filter(|s| !s.item.is_empty() && s.count > 0);
                                if r >= off_y && r < off_y + height && c >= off_x && c < off_x + width {
                                    let pat_idx = (r - off_y) * width + (c - off_x);
                                    let expected = pat.get(pat_idx).and_then(|p| p.as_deref());
                                    match (grid_item, expected) {
                                        (Some(st), Some(exp)) if st.item == exp => {}
                                        (None, None) => {}
                                        _ => { ok = false; break; }
                                    }
                                } else if grid_item.is_some() {
                                    ok = false;
                                    break;
                                }
                            }
                            if !ok {
                                break;
                            }
                        }
                        if ok {
                            return true;
                        }
                    }
                }
                false
            }
        }
    }
}

/// Dynamic Crafting Recipe Resolver:
/// Wraps pure Rust logic and the host engine call (`// FORK-32`).
pub struct RecipeResolver;

impl RecipeResolver {
    /// Default built-in recipes (Mekanism recipes from this family).
    pub fn built_in_recipes() -> Vec<CraftingRecipeDef> {
        vec![
            // Formulaic Assemblicator:
            // S P S
            // C X C
            // S # S
            // S = mekanism:ingot_steel, P = minecraft:crafter, C = mekanism:basic_control_circuit
            // X = mekanism:steel_casing, # = minecraft:chest
            CraftingRecipeDef {
                id: "mekanism:formulaic_assemblicator".to_string(),
                pattern: CraftingPattern::Shaped {
                    width: 3,
                    height: 3,
                    grid: vec![
                        Some("mekanism:ingot_steel".into()), Some("minecraft:crafter".into()), Some("mekanism:ingot_steel".into()),
                        Some("mekanism:basic_control_circuit".into()), Some("mekanism:steel_casing".into()), Some("mekanism:basic_control_circuit".into()),
                        Some("mekanism:ingot_steel".into()), Some("minecraft:chest".into()), Some("mekanism:ingot_steel".into()),
                    ],
                },
                output: ItemStack {
                    item: "mekanism:formulaic_assemblicator".into(),
                    count: 1,
                    components: String::new(),
                },
                remaining: Vec::new(),
            },
            // Crafting Formula (shapeless: 1 paper + 1 basic circuit)
            CraftingRecipeDef {
                id: "mekanism:crafting_formula".to_string(),
                pattern: CraftingPattern::Shapeless {
                    ingredients: vec!["minecraft:paper".into(), "mekanism:basic_control_circuit".into()],
                },
                output: ItemStack {
                    item: "mekanism:crafting_formula".into(),
                    count: 1,
                    components: String::new(),
                },
                remaining: Vec::new(),
            },
            // Configuration Card:
            //   A
            //  A C A
            //   A
            // A = mekanism:alloy_infused, C = mekanism:basic_control_circuit
            CraftingRecipeDef {
                id: "mekanism:configuration_card".to_string(),
                pattern: CraftingPattern::Shaped {
                    width: 3,
                    height: 3,
                    grid: vec![
                        None, Some("mekanism:alloy_infused".into()), None,
                        Some("mekanism:alloy_infused".into()), Some("mekanism:basic_control_circuit".into()), Some("mekanism:alloy_infused".into()),
                        None, Some("mekanism:alloy_infused".into()), None,
                    ],
                },
                output: ItemStack {
                    item: "mekanism:configuration_card".into(),
                    count: 1,
                    components: String::new(),
                },
                remaining: Vec::new(),
            },
        ]
    }

    /// Resolves recipe for the given 3x3 crafting grid.
    /// First queries the host via `// FORK-32`, falling back to built-in recipes.
    pub fn find_recipe(grid: &[Option<ItemStack>; 9]) -> Option<CraftingRecipeDef> {
        // FORK-32: Dynamic host crafting recipe lookup
        // In the fork, this calls `horizon_recipes::match_crafting_recipe(grid)`.
        // If the host provides the recipe, return it; otherwise check built-ins:
        for r in Self::built_in_recipes() {
            if r.matches(grid) {
                return Some(r);
            }
        }
        None
    }
}

/// Java: `RecipeFormula` (`RecipeFormula.java:17-36`).
#[derive(Clone, Debug, PartialEq)]
pub struct RecipeFormula {
    pub input_stacks: [Option<ItemStack>; 9],
    pub recipe: Option<CraftingRecipeDef>,
}

impl RecipeFormula {
    pub const EMPTY: Self = Self {
        input_stacks: [None, None, None, None, None, None, None, None, None],
        recipe: None,
    };

    pub fn from_attachment(att: &FormulaAttachment) -> Self {
        if att.is_empty() || att.invalid {
            return Self::EMPTY;
        }
        let rec = RecipeResolver::find_recipe(&att.items);
        Self {
            input_stacks: att.items.clone(),
            recipe: rec,
        }
    }

    pub fn from_grid(grid: &[Option<ItemStack>; 9]) -> Self {
        let rec = RecipeResolver::find_recipe(grid);
        Self {
            input_stacks: grid.clone(),
            recipe: rec,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.input_stacks.iter().all(|s| s.as_ref().map_or(true, |it| it.count == 0 || it.item.is_empty()))
    }

    pub fn is_valid(&self) -> bool {
        self.recipe.is_some() || !self.is_empty()
    }

    /// Java: `RecipeFormula.java:82-103`: Checks if stack is valid ingredient at slot `pos`.
    pub fn is_ingredient_in_pos(&self, stack: &Option<ItemStack>, pos: usize) -> bool {
        if pos >= 9 {
            return false;
        }
        let expected = &self.input_stacks[pos];
        match (stack.as_ref(), expected.as_ref()) {
            (None, None) => true,
            (Some(st), Some(exp)) => st.item == exp.item,
            _ => false,
        }
    }

    /// Java: `RecipeFormula.java:105-127`: Checks if stack is an ingredient anywhere in the formula.
    pub fn is_valid_ingredient(&self, stack: &ItemStack) -> bool {
        if stack.item.is_empty() || stack.count == 0 {
            return false;
        }
        self.input_stacks.iter().flatten().any(|exp| exp.item == stack.item)
    }

    /// Java: `RecipeFormula.java:74-80`: Checks if the crafting grid matches this formula's recipe.
    pub fn matches(&self, grid: &[Option<ItemStack>; 9]) -> bool {
        if let Some(r) = &self.recipe {
            r.matches(grid)
        } else {
            false
        }
    }
}

// ============================================================================
// 3. ASSEMBLICATOR STATE & DATA SERIALIZATION
// ============================================================================

/// What a Formulaic Assemblicator keeps in `BlockEntity.data`.
#[derive(Clone, Debug, PartialEq)]
pub struct AssemblicatorState {
    pub auto_mode: bool,
    pub stock_control: bool,
    pub operating_ticks: u32,
    pub ticks_required: u32,
    pub pulse_operations: u32,
    pub control: Control,
    pub speed_upgrades: u8,
    pub energy_upgrades: u8,
    pub config: Config,
}

impl Default for AssemblicatorState {
    fn default() -> Self {
        Self {
            auto_mode: false,
            stock_control: false,
            operating_ticks: 0,
            ticks_required: BASE_TICKS_REQUIRED,
            pulse_operations: 0,
            control: Control::Disabled,
            speed_upgrades: 0,
            energy_upgrades: 0,
            config: config::default_config(Layout::Electric),
        }
    }
}

impl AssemblicatorState {
    /// Encodes into byte representation for persistence.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(64);
        out.push(1); // Format version = 1
        out.push(u8::from(self.auto_mode));
        out.push(u8::from(self.stock_control));
        out.push(self.control as u8);
        out.push(self.speed_upgrades);
        out.push(self.energy_upgrades);
        out.extend(self.operating_ticks.to_le_bytes());
        out.extend(self.ticks_required.to_le_bytes());
        out.extend(self.pulse_operations.to_le_bytes());
        out.extend(self.config.encode());
        out
    }

    /// Decodes from byte representation.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() || bytes[0] != 1 || bytes.len() < 18 {
            return None;
        }
        let auto_mode = bytes[1] == 1;
        let stock_control = bytes[2] == 1;
        let control = Control::from_byte(bytes[3]);
        let speed_upgrades = bytes[4];
        let energy_upgrades = bytes[5];

        let mut b = [0u8; 4];
        b.copy_from_slice(&bytes[6..10]);
        let operating_ticks = u32::from_le_bytes(b);

        b.copy_from_slice(&bytes[10..14]);
        let ticks_required = u32::from_le_bytes(b);

        b.copy_from_slice(&bytes[14..18]);
        let pulse_operations = u32::from_le_bytes(b);

        let config = if bytes.len() >= 18 + 35 {
            Config::decode(&bytes[18..], Layout::Electric).unwrap_or_else(|| config::default_config(Layout::Electric))
        } else {
            config::default_config(Layout::Electric)
        };

        Some(Self {
            auto_mode,
            stock_control,
            operating_ticks,
            ticks_required: if ticks_required == 0 { BASE_TICKS_REQUIRED } else { ticks_required },
            pulse_operations,
            control,
            speed_upgrades,
            energy_upgrades,
            config,
        })
    }
}

// ============================================================================
// 4. INVENTORY & SIDE CONFIGURATION
// ============================================================================

/// Computes slot faces for hoppers and pipes based on facing and items configuration.
/// Java: `TileEntityFormulaicAssemblicator.java:115-120`:
/// - Input: input slots 1..=18
/// - Output: output slots 28..=33
/// - Input/Output: input slots 1..=18 + output slots 28..=33
/// - Energy: energy slot 34
/// - Extra: formula slot 0
pub fn assemblicator_item_faces(facing: &str, items: &[Data; 6]) -> Vec<Vec<u32>> {
    let mut faces = vec![Vec::new(); 6];
    for (side, data) in items.iter().enumerate() {
        let face = config::face_of(facing, side);
        let slots: Vec<u32> = match data {
            Data::Input => (SLOT_INPUT_START..SLOT_INPUT_START + INPUT_SLOTS).map(|i| i as u32).collect(),
            Data::Output => (SLOT_OUTPUT_START..SLOT_OUTPUT_START + OUTPUT_SLOTS).map(|i| i as u32).collect(),
            Data::InputOutput => (SLOT_INPUT_START..SLOT_INPUT_START + INPUT_SLOTS)
                .chain(SLOT_OUTPUT_START..SLOT_OUTPUT_START + OUTPUT_SLOTS)
                .map(|i| i as u32)
                .collect(),
            Data::Energy => vec![SLOT_ENERGY as u32],
            Data::Extra => vec![SLOT_FORMULA as u32],
            _ => Vec::new(),
        };
        faces[face] = slots;
    }
    faces
}

// ============================================================================
// 5. CORE ASSEMBLICATOR OPERATING LOGIC
// ============================================================================

/// Builds the stock control map mapping the 9 recipe ingredients to 18 stock slots.
/// Java: `TileEntityFormulaicAssemblicator.java:642-658`:
/// `j = i * 2`, `stockControlMap[j] = stack`, `stockControlMap[j + 1] = stack`.
pub fn build_stock_control_map(formula: &FormulaAttachment) -> [Option<String>; INPUT_SLOTS] {
    let mut map: [Option<String>; INPUT_SLOTS] = Default::default();
    if formula.is_empty() {
        return map;
    }
    for i in 0..9 {
        let j = i * 2;
        if let Some(stack) = &formula.items[i] {
            if !stack.item.is_empty() && stack.count > 0 {
                map[j] = Some(stack.item.clone());
                map[j + 1] = Some(stack.item.clone());
            }
        }
    }
    map
}

/// Checks whether an item is valid for an input slot under stock control.
/// Java: `TileEntityFormulaicAssemblicator.java:159-172`.
pub fn is_item_valid_for_input(
    formula: &RecipeFormula,
    stock_control: bool,
    stock_map: &[Option<String>; INPUT_SLOTS],
    slot_idx: usize,
    stack: &ItemStack,
) -> bool {
    if formula.is_empty() {
        true
    } else if !formula.is_valid() {
        false
    } else if stock_control {
        if let Some(expected_item) = &stock_map[slot_idx] {
            *expected_item == stack.item
        } else {
            false
        }
    } else {
        formula.is_valid_ingredient(stack)
    }
}

/// Moves items from the 18 input slots to the 9 crafting grid slots to satisfy the formula.
/// Java: `TileEntityFormulaicAssemblicator.java:406-459`.
pub fn move_items_to_grid(slots: &mut [Option<ItemStack>], formula: &RecipeFormula) -> bool {
    if formula.is_empty() || !formula.is_valid() {
        return false;
    }
    let mut ret = true;
    for i in 0..CRAFT_SLOTS {
        let craft_idx = SLOT_CRAFT_START + i;
        let expected_opt = formula.input_stacks[i].clone();

        // If the slot already has the exact expected ingredient, continue
        if formula.is_ingredient_in_pos(&slots[craft_idx], i) {
            continue;
        }

        let is_empty = slots[craft_idx].as_ref().map_or(true, |s| s.count == 0);
        if is_empty {
            // Need an item for this slot if expected is Some
            if let Some(exp) = expected_opt {
                let mut found = false;
                // Search input slots backwards from 17 down to 0
                for j in (0..INPUT_SLOTS).rev() {
                    let in_idx = SLOT_INPUT_START + j;
                    let matches_exp = slots[in_idx].as_ref().map_or(false, |in_st| in_st.item == exp.item && in_st.count > 0);
                    if matches_exp {
                        // Take 1 item from input slot and put into crafting slot
                        let (taken_item, components, remaining_count) = {
                            let in_st = slots[in_idx].as_mut().unwrap();
                            let count = in_st.count - 1;
                            (in_st.item.clone(), in_st.components.clone(), count)
                        };
                        slots[craft_idx] = Some(ItemStack {
                            item: taken_item,
                            count: 1,
                            components,
                        });
                        if remaining_count == 0 {
                            slots[in_idx] = None;
                        } else if let Some(st_mut) = slots[in_idx].as_mut() {
                            st_mut.count = remaining_count;
                        }
                        found = true;
                        break;
                    }
                }
                if !found {
                    ret = false;
                }
            }
        } else {
            // Crafting slot contains a wrong item, try moving it to input slots
            let wrong_st = slots[craft_idx].take().unwrap();
            let remainder = try_insert_input(slots, wrong_st);
            if let Some(rem) = remainder {
                // Could not push completely back, put remainder back in craft slot
                slots[craft_idx] = Some(rem);
                ret = false;
            }
        }
    }
    ret
}

/// Tries to insert a stack into the 18 input slots, returning leftover if any.
pub fn try_insert_input(slots: &mut [Option<ItemStack>], mut stack: ItemStack) -> Option<ItemStack> {
    // Pass 1: Add to existing matching stacks
    for i in 0..INPUT_SLOTS {
        let idx = SLOT_INPUT_START + i;
        if let Some(cur) = &mut slots[idx] {
            if cur.item == stack.item && config::same_components(&cur.components, &stack.components) {
                let room = 64u32.saturating_sub(cur.count);
                let add = stack.count.min(room);
                cur.count += add;
                stack.count -= add;
                if stack.count == 0 {
                    return None;
                }
            }
        }
    }
    // Pass 2: Place in empty slots
    for i in 0..INPUT_SLOTS {
        let idx = SLOT_INPUT_START + i;
        if slots[idx].is_none() {
            slots[idx] = Some(stack);
            return None;
        }
    }
    Some(stack)
}

/// Tries to insert a stack into the 6 output slots, returning leftover if any.
pub fn try_insert_output(slots: &mut [Option<ItemStack>], mut stack: ItemStack) -> Option<ItemStack> {
    // Pass 1: Add to existing matching stacks
    for i in 0..OUTPUT_SLOTS {
        let idx = SLOT_OUTPUT_START + i;
        if let Some(cur) = &mut slots[idx] {
            if cur.item == stack.item && config::same_components(&cur.components, &stack.components) {
                let room = 64u32.saturating_sub(cur.count);
                let add = stack.count.min(room);
                cur.count += add;
                stack.count -= add;
                if stack.count == 0 {
                    return None;
                }
            }
        }
    }
    // Pass 2: Place in empty slots
    for i in 0..OUTPUT_SLOTS {
        let idx = SLOT_OUTPUT_START + i;
        if slots[idx].is_none() {
            slots[idx] = Some(stack);
            return None;
        }
    }
    Some(stack)
}

/// Simulates whether an item fits into the output slots.
pub fn can_fit_output(slots: &[Option<ItemStack>], stack: &ItemStack) -> bool {
    let mut needed = stack.count;
    for i in 0..OUTPUT_SLOTS {
        let idx = SLOT_OUTPUT_START + i;
        match &slots[idx] {
            None => return true,
            Some(cur) if cur.item == stack.item && config::same_components(&cur.components, &stack.components) => {
                let room = 64u32.saturating_sub(cur.count);
                if room >= needed {
                    return true;
                }
                needed -= room;
            }
            _ => {}
        }
    }
    false
}

/// Executes a single crafting operation (`doSingleCraft`).
/// Java: `TileEntityFormulaicAssemblicator.java:368-396`.
pub fn do_single_craft(
    slots: &mut [Option<ItemStack>],
    recipe: &CraftingRecipeDef,
    formula: &RecipeFormula,
) -> bool {
    // 1. Simulate output insertion
    if !can_fit_output(slots, &recipe.output) {
        return false;
    }
    // 2. Simulate remaining items insertion
    for rem in &recipe.remaining {
        if !can_fit_output(slots, rem) {
            return false;
        }
    }

    // 3. Execute outputs
    try_insert_output(slots, recipe.output.clone());
    for rem in &recipe.remaining {
        try_insert_output(slots, rem.clone());
    }

    // 4. Shrink 1 from each crafting grid slot
    for i in 0..CRAFT_SLOTS {
        let idx = SLOT_CRAFT_START + i;
        let should_clear = match &mut slots[idx] {
            Some(st) if st.count <= 1 => true,
            Some(st) => {
                st.count -= 1;
                false
            }
            None => false,
        };
        if should_clear {
            slots[idx] = None;
        }
    }

    // 5. If formula present, immediately refill grid from input buffer
    if !formula.is_empty() {
        move_items_to_grid(slots, formula);
    }

    true
}

/// Performs a manual craft single (`craftSingle`).
/// Java: `TileEntityFormulaicAssemblicator.java:398-404`.
pub fn craft_single(slots: &mut [Option<ItemStack>], formula: &RecipeFormula) -> bool {
    let mut grid: [Option<ItemStack>; 9] = Default::default();
    for i in 0..9 {
        grid[i] = slots[SLOT_CRAFT_START + i].clone();
    }
    let recipe = if !formula.is_empty() && formula.is_valid() {
        if !formula.matches(&grid) {
            move_items_to_grid(slots, formula);
            for i in 0..9 {
                grid[i] = slots[SLOT_CRAFT_START + i].clone();
            }
        }
        formula.recipe.clone()
    } else {
        RecipeResolver::find_recipe(&grid)
    };

    if let Some(r) = recipe {
        do_single_craft(slots, &r, formula)
    } else {
        false
    }
}

/// Performs craft all (`craftAll`).
/// Java: `TileEntityFormulaicAssemblicator.java:461-465`.
pub fn craft_all(slots: &mut [Option<ItemStack>], formula: &RecipeFormula) -> u32 {
    let mut count = 0;
    while craft_single(slots, formula) {
        count += 1;
        if count >= 64 * 9 {
            break; // Sanity limit
        }
    }
    count
}

/// Empties all items from crafting grid back into input buffer.
/// Java: `TileEntityFormulaicAssemblicator.java:473-477`.
pub fn empty_grid(slots: &mut [Option<ItemStack>]) {
    for i in 0..CRAFT_SLOTS {
        let idx = SLOT_CRAFT_START + i;
        if let Some(st) = slots[idx].take() {
            let rem = try_insert_input(slots, st);
            if let Some(r) = rem {
                slots[idx] = Some(r);
            }
        }
    }
}

/// Organizes the input buffer according to the stock control map.
/// Java: `TileEntityFormulaicAssemblicator.java:543-616`.
pub fn organize_stock(slots: &mut [Option<ItemStack>], stock_map: &[Option<String>; INPUT_SLOTS]) {
    // 1. Gather all items in input slots into a count map
    let mut counts: HashMap<String, u32> = HashMap::new();
    for i in 0..INPUT_SLOTS {
        let idx = SLOT_INPUT_START + i;
        if let Some(st) = slots[idx].take() {
            if !st.item.is_empty() && st.count > 0 {
                *counts.entry(st.item).or_insert(0) += st.count;
            }
        }
    }

    // 2. Place items into their respective controlled slots
    let mut unused_slots: Vec<usize> = Vec::new();
    for i in 0..INPUT_SLOTS {
        if let Some(expected) = &stock_map[i] {
            if let Some(stored) = counts.get_mut(expected) {
                if *stored > 0 {
                    let take = (*stored).min(64);
                    *stored -= take;
                    slots[SLOT_INPUT_START + i] = Some(ItemStack {
                        item: expected.clone(),
                        count: take,
                        components: String::new(),
                    });
                }
            }
        } else {
            unused_slots.push(i);
        }
    }

    // 3. Clean zero counts
    counts.retain(|_, count| *count > 0);

    // 4. Fill remaining items into unused slots
    for i in unused_slots {
        if counts.is_empty() {
            break;
        }
        let first_key = counts.keys().next().cloned().unwrap();
        let stored = counts.get_mut(&first_key).unwrap();
        let take = (*stored).min(64);
        *stored -= take;
        slots[SLOT_INPUT_START + i] = Some(ItemStack {
            item: first_key.clone(),
            count: take,
            components: String::new(),
        });
        if *stored == 0 {
            counts.remove(&first_key);
        }
    }
}

/// Encodes the current crafting grid recipe onto the formula item in `SLOT_FORMULA`.
/// Java: `TileEntityFormulaicAssemblicator.java:673-686`.
pub fn encode_formula_slot(slots: &mut [Option<ItemStack>]) -> bool {
    let is_valid_formula = slots[SLOT_FORMULA].as_ref().map_or(false, |s| s.item == CRAFTING_FORMULA);
    if !is_valid_formula {
        return false;
    }
    let att = CraftingFormulaItem::get_attachment(slots[SLOT_FORMULA].as_ref().unwrap());
    if !att.is_empty() {
        return false; // Already encoded
    }

    let mut grid: [Option<ItemStack>; 9] = Default::default();
    for i in 0..9 {
        grid[i] = slots[SLOT_CRAFT_START + i].clone();
    }
    let Some(_) = RecipeResolver::find_recipe(&grid) else { return false };

    let new_att = FormulaAttachment::new(grid, false);
    if let Some(formula_stack) = &mut slots[SLOT_FORMULA] {
        CraftingFormulaItem::set_attachment(formula_stack, &new_att);
        true
    } else {
        false
    }
}

// ============================================================================
// 6. TICK EXECUTION & INTEGRATION
// ============================================================================

/// Ticks a single Formulaic Assemblicator block entity.
/// Java: `TileEntityFormulaicAssemblicator.java:213-267`.
pub fn tick_assemblicator(
    _e: &BlockEntity,
    state: &mut AssemblicatorState,
    items: &mut [Option<ItemStack>],
    energy: &mut u64,
    powered: bool,
    last_powered: bool,
) -> bool {
    let mut changed = false;

    // Upgrades recalculation
    state.ticks_required = upgrades::ticks(BASE_TICKS_REQUIRED, state.speed_upgrades);
    let _max_storage = upgrades::max_energy(BASE_STORAGE, state.energy_upgrades);
    let energy_per_tick = upgrades::energy_per_tick(BASE_ENERGY_USAGE, state.speed_upgrades, state.energy_upgrades);

    // Pulse redstone operation tracking
    if state.control != Control::Pulse {
        state.pulse_operations = 0;
    } else if state.control.can_function(powered, last_powered) {
        state.pulse_operations += 1;
    }

    // Read formula from slot 0
    let formula_att = items[SLOT_FORMULA].as_ref().map(CraftingFormulaItem::get_attachment).unwrap_or(FormulaAttachment::EMPTY);
    let formula = RecipeFormula::from_attachment(&formula_att);

    // AutoMode validation: if formula is empty, autoMode shuts off
    if state.auto_mode && formula.is_empty() {
        state.auto_mode = false;
        state.operating_ticks = 0;
        changed = true;
    }

    // Auto-crafting execution loop
    let can_run = if state.control == Control::Pulse {
        state.pulse_operations > 0
    } else {
        state.control.can_function(powered, last_powered)
    };

    if state.auto_mode && !formula.is_empty() && can_run {
        let mut grid: [Option<ItemStack>; 9] = Default::default();
        for i in 0..9 {
            grid[i] = items[SLOT_CRAFT_START + i].clone();
        }

        let mut can_operate = true;
        if !formula.matches(&grid) {
            can_operate = move_items_to_grid(items, &formula);
            changed = true;
        }

        if can_operate {
            if state.operating_ticks >= state.ticks_required {
                if let Some(recipe) = &formula.recipe {
                    if do_single_craft(items, recipe, &formula) {
                        state.operating_ticks = 0;
                        if state.pulse_operations > 0 {
                            state.pulse_operations -= 1;
                        }
                        changed = true;
                    }
                }
            } else if *energy >= energy_per_tick {
                *energy -= energy_per_tick;
                state.operating_ticks += 1;
                changed = true;
            }
        } else {
            state.operating_ticks = 0;
        }
    } else if !state.auto_mode {
        state.operating_ticks = 0;
    }

    changed
}

impl Mekanism {
    /// Runs all Formulaic Assemblicators in the world during the mod's server tick.
    pub(crate) fn run_assemblicators(&mut self, assemblicators: &[&BlockEntity], _tick: u64) -> Vec<BlockEntityUpdate> {
        let mut updates = Vec::new();

        for e in assemblicators {
            let at = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
            let mut state = AssemblicatorState::decode(&e.data).unwrap_or_default();
            let facing = sides::property(&e.state, "facing").unwrap_or("north").to_owned();

            // First-time setup: configure faces and eject
            let first = !self.applied.contains(&at);
            if first {
                self.applied.insert(at.clone());
                let faces = assemblicator_item_faces(&facing, state.config.items());
                let _ = world::set_item_faces(&e.dimension, e.pos.x, e.pos.y, e.pos.z, &faces);
                let _ = world::set_item_eject(&e.dimension, e.pos.x, e.pos.y, e.pos.z, state.config.ejects(Transmission::Items));
            }

            // Ensure 35 items
            let mut items: Vec<Option<ItemStack>> = (0..TOTAL_SLOTS).map(|i| e.items.get(i).cloned().flatten()).collect();
            let mut energy = e.energy;

            let powered = crate::fork_utilidades::has_neighbor_signal(&e.dimension, e.pos.x, e.pos.y, e.pos.z);
            let last_powered = self.last_power.insert(at.clone(), powered).unwrap_or(false);

            let changed = tick_assemblicator(e, &mut state, &mut items, &mut energy, powered, last_powered);

            if changed || first {
                let encoded_data = state.encode();
                updates.push(BlockEntityUpdate {
                    dimension: e.dimension.clone(),
                    pos: e.pos.clone(),
                    data: encoded_data,
                    items: Some(items),
                    energy: Some(energy),
                    menu_data: Some(vec![
                        state.auto_mode as i32,
                        state.stock_control as i32,
                        state.operating_ticks as i32,
                        state.ticks_required as i32,
                        state.control as i32,
                    ]),
                    tanks: None,
                    tank_faces: None,
                    energy_faces: None,
                    transmitter_faces: None,
                });
            }
        }

        updates
    }

    /// Handles button presses from the Formulaic Assemblicator GUI.
    pub(crate) fn assemblicator_button(&mut self, e: &BlockEntity, id: i32) -> Option<BlockEntityUpdate> {
        let mut state = AssemblicatorState::decode(&e.data).unwrap_or_default();
        let mut items: Vec<Option<ItemStack>> = (0..TOTAL_SLOTS).map(|i| e.items.get(i).cloned().flatten()).collect();
        let mut changed = false;

        let formula_att = items[SLOT_FORMULA].as_ref().map(CraftingFormulaItem::get_attachment).unwrap_or(FormulaAttachment::EMPTY);
        let formula = RecipeFormula::from_attachment(&formula_att);

        match id {
            BUTTON_ENCODE_FORMULA => {
                changed = encode_formula_slot(&mut items);
            }
            BUTTON_STOCK_CONTROL => {
                if !formula.is_empty() {
                    state.stock_control = !state.stock_control;
                    if state.stock_control {
                        let stock_map = build_stock_control_map(&formula_att);
                        organize_stock(&mut items, &stock_map);
                    }
                    changed = true;
                }
            }
            BUTTON_FILL_EMPTY => {
                if formula.is_empty() {
                    empty_grid(&mut items);
                } else {
                    move_items_to_grid(&mut items, &formula);
                }
                changed = true;
            }
            BUTTON_CRAFT_SINGLE => {
                if !state.auto_mode {
                    changed = craft_single(&mut items, &formula);
                }
            }
            BUTTON_CRAFT_ALL => {
                if !state.auto_mode {
                    changed = craft_all(&mut items, &formula) > 0;
                }
            }
            BUTTON_AUTO_MODE => {
                if !formula.is_empty() && formula.is_valid() {
                    state.auto_mode = !state.auto_mode;
                    state.operating_ticks = 0;
                    changed = true;
                }
            }
            _ => {}
        }

        if changed {
            Some(BlockEntityUpdate {
                dimension: e.dimension.clone(),
                pos: e.pos.clone(),
                data: state.encode(),
                items: Some(items),
                energy: Some(e.energy),
                menu_data: Some(vec![
                    state.auto_mode as i32,
                    state.stock_control as i32,
                    state.operating_ticks as i32,
                    state.ticks_required as i32,
                    state.control as i32,
                ]),
                tanks: None,
                tank_faces: None,
                energy_faces: None,
                transmitter_faces: None,
            })
        } else {
            None
        }
    }
}

// ============================================================================
// 7. UNIT TESTS (AUDITED AGAINST JAVA 1.21.1)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test: 35 Slots inventory structure matches Java exactly.
    /// Java source: `TileEntityFormulaicAssemblicator.java:154-190`.
    #[test]
    fn test_inventory_slot_layout() {
        assert_eq!(TOTAL_SLOTS, 35);
        assert_eq!(SLOT_FORMULA, 0);
        assert_eq!(SLOT_INPUT_START, 1);
        assert_eq!(INPUT_SLOTS, 18);
        assert_eq!(SLOT_CRAFT_START, 19);
        assert_eq!(CRAFT_SLOTS, 9);
        assert_eq!(SLOT_OUTPUT_START, 28);
        assert_eq!(OUTPUT_SLOTS, 6);
        assert_eq!(SLOT_ENERGY, 34);
    }

    /// Test: Base configuration constants match Java.
    /// Java source: `TileEntityFormulaicAssemblicator.java:80`, `MekanismConfig.java`.
    #[test]
    fn test_base_constants() {
        assert_eq!(BASE_TICKS_REQUIRED, 40); // 2 * SharedConstants.TICKS_PER_SECOND
        assert_eq!(BASE_ENERGY_USAGE, 40);   // 100 J/t * 2 / 5 = 40 FE/t
        assert_eq!(BASE_STORAGE, 16_000);    // 40_000 J * 2 / 5 = 16_000 FE
    }

    /// Test: FormulaAttachment SNBT serialization and deserialization.
    /// Java source: `FormulaAttachment.java:21-31`.
    #[test]
    fn test_formula_attachment_snbt_roundtrip() {
        let mut items: [Option<ItemStack>; 9] = Default::default();
        items[0] = Some(ItemStack { item: "minecraft:paper".into(), count: 1, components: String::new() });
        items[4] = Some(ItemStack { item: "mekanism:basic_control_circuit".into(), count: 1, components: String::new() });

        let att = FormulaAttachment::new(items, false);
        assert!(att.has_items());
        assert!(!att.is_empty());
        assert!(!att.invalid);

        let encoded = att.encode_snbt();
        let decoded = FormulaAttachment::decode_snbt(&encoded).expect("failed to decode SNBT");

        assert_eq!(decoded.invalid, false);
        assert_eq!(decoded.items[0].as_ref().map(|s| &s.item), Some(&"minecraft:paper".to_string()));
        assert_eq!(decoded.items[4].as_ref().map(|s| &s.item), Some(&"mekanism:basic_control_circuit".to_string()));
        assert!(decoded.items[1].is_none());
    }

    /// Test: Formula display name and tooltips match Java rules.
    /// Java source: `ItemCraftingFormula.java:31-65`.
    #[test]
    fn test_crafting_formula_display() {
        let mut empty_stack = ItemStack {
            item: CRAFTING_FORMULA.into(),
            count: 1,
            components: String::new(),
        };
        assert_eq!(CraftingFormulaItem::display_name(&empty_stack), "Crafting Formula");
        assert!(CraftingFormulaItem::tooltip_ingredients(&empty_stack).is_empty());

        let mut items: [Option<ItemStack>; 9] = Default::default();
        items[0] = Some(ItemStack { item: "minecraft:iron_ingot".into(), count: 3, components: String::new() });
        items[1] = Some(ItemStack { item: "minecraft:stick".into(), count: 2, components: String::new() });

        let att = FormulaAttachment::new(items, false);
        CraftingFormulaItem::set_attachment(&mut empty_stack, &att);

        assert_eq!(CraftingFormulaItem::display_name(&empty_stack), "Crafting Formula (Encoded)");
        let tooltips = CraftingFormulaItem::tooltip_ingredients(&empty_stack);
        assert_eq!(tooltips.len(), 2);
        assert_eq!(tooltips[0], ("minecraft:iron_ingot".to_string(), 3));
        assert_eq!(tooltips[1], ("minecraft:stick".to_string(), 2));

        // Test invalid
        let invalid_att = att.as_invalid();
        CraftingFormulaItem::set_attachment(&mut empty_stack, &invalid_att);
        assert_eq!(CraftingFormulaItem::display_name(&empty_stack), "Crafting Formula (Invalid)");

        // Test clear
        CraftingFormulaItem::clear(&mut empty_stack);
        assert_eq!(CraftingFormulaItem::display_name(&empty_stack), "Crafting Formula");
    }

    /// Test: Stock control map generates 2 input slots per recipe position.
    /// Java source: `TileEntityFormulaicAssemblicator.java:642-658`.
    #[test]
    fn test_stock_control_map() {
        let mut items: [Option<ItemStack>; 9] = Default::default();
        items[0] = Some(ItemStack { item: "minecraft:diamond".into(), count: 1, components: String::new() });
        items[4] = Some(ItemStack { item: "minecraft:stick".into(), count: 1, components: String::new() });

        let att = FormulaAttachment::new(items, false);
        let map = build_stock_control_map(&att);

        assert_eq!(map[0], Some("minecraft:diamond".to_string()));
        assert_eq!(map[1], Some("minecraft:diamond".to_string()));
        assert_eq!(map[2], None);
        assert_eq!(map[3], None);
        assert_eq!(map[8], Some("minecraft:stick".to_string()));
        assert_eq!(map[9], Some("minecraft:stick".to_string()));
    }

    /// Test: Assemblicator state serialization and deserialization.
    #[test]
    fn test_assemblicator_state_encode_decode() {
        let mut state = AssemblicatorState::default();
        state.auto_mode = true;
        state.stock_control = true;
        state.operating_ticks = 25;
        state.ticks_required = 40;
        state.pulse_operations = 2;
        state.control = Control::Pulse;
        state.speed_upgrades = 4;
        state.energy_upgrades = 8;

        let encoded = state.encode();
        let decoded = AssemblicatorState::decode(&encoded).expect("failed to decode state");

        assert_eq!(decoded.auto_mode, true);
        assert_eq!(decoded.stock_control, true);
        assert_eq!(decoded.operating_ticks, 25);
        assert_eq!(decoded.ticks_required, 40);
        assert_eq!(decoded.pulse_operations, 2);
        assert_eq!(decoded.control, Control::Pulse);
        assert_eq!(decoded.speed_upgrades, 4);
        assert_eq!(decoded.energy_upgrades, 8);
    }

    /// Test: Pure Rust recipe resolution and execution of craft.
    /// Java source: `TileEntityFormulaicAssemblicator.java:368-396`.
    #[test]
    fn test_crafting_execution() {
        let mut slots: Vec<Option<ItemStack>> = vec![None; TOTAL_SLOTS];

        // Prepare grid for Crafting Formula: 1 paper + 1 basic circuit
        slots[SLOT_CRAFT_START] = Some(ItemStack { item: "minecraft:paper".into(), count: 1, components: String::new() });
        slots[SLOT_CRAFT_START + 1] = Some(ItemStack { item: "mekanism:basic_control_circuit".into(), count: 1, components: String::new() });

        let mut grid: [Option<ItemStack>; 9] = Default::default();
        grid[0] = slots[SLOT_CRAFT_START].clone();
        grid[1] = slots[SLOT_CRAFT_START + 1].clone();

        let recipe = RecipeResolver::find_recipe(&grid).expect("recipe not found");
        assert_eq!(recipe.output.item, "mekanism:crafting_formula");

        let formula = RecipeFormula::from_grid(&grid);
        let success = do_single_craft(&mut slots, &recipe, &formula);
        assert!(success);

        // Crafting grid slots should be empty (count consumed)
        assert!(slots[SLOT_CRAFT_START].is_none());
        assert!(slots[SLOT_CRAFT_START + 1].is_none());

        // Output buffer slot 28 should contain 1 crafting formula
        let out = slots[SLOT_OUTPUT_START].as_ref().expect("no output produced");
        assert_eq!(out.item, "mekanism:crafting_formula");
        assert_eq!(out.count, 1);
    }

    /// Test: Speed upgrades reduce ticks from 40 down to 4 at 8 upgrades.
    /// Java source: `TileEntityFormulaicAssemblicator.java:714-717`, `MekanismUtils.java`.
    #[test]
    fn test_speed_upgrades_scaling() {
        assert_eq!(upgrades::ticks(BASE_TICKS_REQUIRED, 0), 40);
        assert_eq!(upgrades::ticks(BASE_TICKS_REQUIRED, 8), 4);
    }

    /// Test: Stock control filtering allows only mapped ingredients.
    /// Java source: `TileEntityFormulaicAssemblicator.java:159-172`.
    #[test]
    fn test_stock_control_input_validation() {
        let mut items: [Option<ItemStack>; 9] = Default::default();
        items[0] = Some(ItemStack { item: "minecraft:iron_ingot".into(), count: 1, components: String::new() });
        let att = FormulaAttachment::new(items, false);
        let formula = RecipeFormula::from_attachment(&att);
        let stock_map = build_stock_control_map(&att);

        let iron = ItemStack { item: "minecraft:iron_ingot".into(), count: 1, components: String::new() };
        let gold = ItemStack { item: "minecraft:gold_ingot".into(), count: 1, components: String::new() };

        // Slot 0 and 1 are mapped to iron_ingot
        assert!(is_item_valid_for_input(&formula, true, &stock_map, 0, &iron));
        assert!(is_item_valid_for_input(&formula, true, &stock_map, 1, &iron));
        assert!(!is_item_valid_for_input(&formula, true, &stock_map, 0, &gold));
        // Slot 2 is unmapped -> rejects when stock control is true
        assert!(!is_item_valid_for_input(&formula, true, &stock_map, 2, &iron));

        // When stock control is false, iron is valid for any slot as long as formula needs it
        assert!(is_item_valid_for_input(&formula, false, &stock_map, 2, &iron));
        assert!(!is_item_valid_for_input(&formula, false, &stock_map, 2, &gold));
    }

    /// Test: empty_grid moves items back to input slots.
    /// Java source: `TileEntityFormulaicAssemblicator.java:473-477`.
    #[test]
    fn test_empty_grid_moves_to_inputs() {
        let mut slots: Vec<Option<ItemStack>> = vec![None; TOTAL_SLOTS];
        slots[SLOT_CRAFT_START] = Some(ItemStack { item: "minecraft:paper".into(), count: 5, components: String::new() });

        empty_grid(&mut slots);

        assert!(slots[SLOT_CRAFT_START].is_none());
        assert_eq!(slots[SLOT_INPUT_START].as_ref().map(|s| &s.item), Some(&"minecraft:paper".to_string()));
        assert_eq!(slots[SLOT_INPUT_START].as_ref().map(|s| s.count), Some(5));
    }
}

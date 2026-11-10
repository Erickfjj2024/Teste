//! QIO Dashboard (`TileEntityQIODashboard`, `QIOCraftingWindow`).
//!
//! Reference: Mekanism Java 1.21.1-10.7.19.85
//! - Tile Entity: `java/main/java/mekanism/common/tile/qio/TileEntityQIODashboard.java`
//! - Crafting Window: `java/main/java/mekanism/common/content/qio/QIOCraftingWindow.java`
//! - Holder Interface: `java/main/java/mekanism/common/content/qio/IQIOCraftingWindowHolder.java`
//! - Packets: `java/main/java/mekanism/common/network/to_server/qio/`
//!
//! Features:
//! - 6D Facing: Can be attached to floors, ceilings, or walls (`FacePlacementType.SELECTED_FACE`).
//! - 3 Independent Crafting Windows (`MAX_CRAFTING_WINDOWS = 3`, indices 0, 1, 2).
//! - 10 slots per window: 9 input slots (3x3 grid) + 1 output slot = 30 total internal slots.
//! - Shift-Click Toggle (`insertIntoFrequency`, default `true`, `MekanismDataComponents.INSERT_INTO_FREQUENCY`):
//!   - `true`: Shift-click sends item from player inventory directly into QIO Frequency.
//!   - `false`: Shift-click moves item between player main inventory and hotbar.
//! - Virtual catalog actions: Take (1, half stack, full stack), ShiftTake, Place cursor into frequency, Clear crafting window.
use super::*;
use super::qio_transfer::QioStorageView;

pub const BLOCK: &str = "mekanism:qio_dashboard";

/// Maximum independent crafting windows (`IQIOCraftingWindowHolder.MAX_CRAFTING_WINDOWS = 3`).
pub const MAX_CRAFTING_WINDOWS: usize = 3;

/// Grid size per crafting window (3x3).
pub const CRAFTING_GRID_SIZE: usize = 9;

/// Total slots per window: 9 input + 1 output.
pub const SLOTS_PER_WINDOW: usize = 10;

/// Total internal slots across all windows (`TileEntityQIODashboard.java:66-73`): 30 slots.
pub const TOTAL_INTERNAL_SLOTS: usize = MAX_CRAFTING_WINDOWS * SLOTS_PER_WINDOW;

/// Screen button IDs.
pub const TOGGLE_SHIFT_CLICK_DIRECTION: i32 = 7610;
pub const CLEAR_CRAFTING_WINDOW_BASE: i32 = 7611; // 7611, 7612, 7613 for windows 0, 1, 2

// ============================================================================
// Crafting Window Data Structures (QIOCraftingWindow.java)
// ============================================================================

/// Represents one 3x3 crafting grid with output slot.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct CraftingWindow {
    pub inputs: [Option<ItemStack>; CRAFTING_GRID_SIZE],
    pub output: Option<ItemStack>,
}

impl CraftingWindow {
    pub fn new() -> Self {
        Self {
            inputs: Default::default(),
            output: None,
        }
    }

    /// Clears all 9 input slots, returning items that were present to be deposited into frequency.
    pub fn clear_inputs(&mut self) -> Vec<ItemStack> {
        let mut out = Vec::new();
        for slot in &mut self.inputs {
            if let Some(s) = slot.take() {
                out.push(s);
            }
        }
        self.output = None;
        out
    }

    /// Evaluates recipe and sets output slot.
    pub fn update_recipe<R: Fn(&[Option<ItemStack>; 9]) -> Option<ItemStack>>(&mut self, recipe_resolver: R) {
        self.output = recipe_resolver(&self.inputs);
    }

    /// Performs craft: consumes 1 from each input slot, leaves remainder if any.
    pub fn craft_once<Rem: Fn(&str) -> Option<String>>(&mut self, remainder_resolver: Rem) -> Option<ItemStack> {
        let crafted = self.output.take()?;

        for slot in &mut self.inputs {
            if let Some(stack) = slot {
                let remainder_item = remainder_resolver(&stack.item);
                if stack.count <= 1 {
                    *slot = remainder_item.map(|item| ItemStack {
                        item,
                        count: 1,
                        components: String::new(),
                    });
                } else {
                    stack.count -= 1;
                    // If recipe item leaves a remainder (e.g. bucket), place remainder back if space or drop
                    // In vanilla crafting grids, containers stay in place if empty
                }
            }
        }

        Some(crafted)
    }
}

// ============================================================================
// Dashboard Settings & State
// ============================================================================

#[derive(Clone, Debug, PartialEq)]
pub struct DashboardState {
    /// Shift-click routing flag (`insertIntoFrequency`, default `true`, `TileEntityQIODashboard.java:42`).
    pub insert_into_frequency: bool,
    pub frequency_name: String,
    pub facing: String,
    pub windows: [CraftingWindow; MAX_CRAFTING_WINDOWS],
}

impl Default for DashboardState {
    fn default() -> Self {
        Self {
            insert_into_frequency: true,
            frequency_name: String::new(),
            facing: "north".into(),
            windows: [CraftingWindow::new(), CraftingWindow::new(), CraftingWindow::new()],
        }
    }
}

pub fn encode_dashboard(d: &DashboardState) -> Vec<u8> {
    let mut out = vec![
        1, // version
        u8::from(d.insert_into_frequency),
    ];
    let freq_bytes = d.frequency_name.as_bytes();
    out.push(freq_bytes.len().min(255) as u8);
    out.extend_from_slice(&freq_bytes[..freq_bytes.len().min(255)]);

    let facing_bytes = d.facing.as_bytes();
    out.push(facing_bytes.len().min(255) as u8);
    out.extend_from_slice(&facing_bytes[..facing_bytes.len().min(255)]);

    // Encode windows slots
    for w in &d.windows {
        for slot in &w.inputs {
            encode_slot(slot, &mut out);
        }
        encode_slot(&w.output, &mut out);
    }
    out
}

fn encode_slot(slot: &Option<ItemStack>, out: &mut Vec<u8>) {
    match slot {
        None => out.push(0),
        Some(s) => {
            out.push(1);
            let ib = s.item.as_bytes();
            out.push(ib.len().min(255) as u8);
            out.extend_from_slice(&ib[..ib.len().min(255)]);
            out.extend_from_slice(&s.count.to_le_bytes());
            let cb = s.components.as_bytes();
            out.push(cb.len().min(255) as u8);
            out.extend_from_slice(&cb[..cb.len().min(255)]);
        }
    }
}

pub fn decode_dashboard(data: &[u8]) -> Option<DashboardState> {
    if data.len() < 4 || data[0] != 1 {
        return None;
    }
    let insert_into_frequency = data[1] == 1;
    let mut idx = 2;

    let flen = data[idx] as usize;
    idx += 1;
    if idx + flen > data.len() {
        return None;
    }
    let frequency_name = String::from_utf8_lossy(&data[idx..idx + flen]).to_string();
    idx += flen;

    if idx >= data.len() {
        return None;
    }
    let facelen = data[idx] as usize;
    idx += 1;
    if idx + facelen > data.len() {
        return None;
    }
    let facing = String::from_utf8_lossy(&data[idx..idx + facelen]).to_string();
    idx += facelen;

    let mut windows = [CraftingWindow::new(), CraftingWindow::new(), CraftingWindow::new()];
    for w in &mut windows {
        for slot in &mut w.inputs {
            *slot = decode_slot(data, &mut idx);
        }
        w.output = decode_slot(data, &mut idx);
    }

    Some(DashboardState {
        insert_into_frequency,
        frequency_name,
        facing,
        windows,
    })
}

fn decode_slot(data: &[u8], idx: &mut usize) -> Option<ItemStack> {
    if *idx >= data.len() {
        return None;
    }
    let has_item = data[*idx];
    *idx += 1;
    if has_item == 0 {
        return None;
    }
    if *idx >= data.len() {
        return None;
    }
    let ilen = data[*idx] as usize;
    *idx += 1;
    if *idx + ilen + 4 > data.len() {
        return None;
    }
    let item = String::from_utf8_lossy(&data[*idx..*idx + ilen]).to_string();
    *idx += ilen;

    let count = u32::from_le_bytes([data[*idx], data[*idx + 1], data[*idx + 2], data[*idx + 3]]);
    *idx += 4;

    if *idx >= data.len() {
        return None;
    }
    let clen = data[*idx] as usize;
    *idx += 1;
    if *idx + clen > data.len() {
        return None;
    }
    let components = String::from_utf8_lossy(&data[*idx..*idx + clen]).to_string();
    *idx += clen;

    Some(ItemStack { item, count, components })
}

// ============================================================================
// Shift-Click Routing Logic (TileEntityQIODashboard.java:172-178)
// ============================================================================

/// Handles player shift-click inside the QIO Dashboard GUI.
///
/// Java logic:
/// - If `insertIntoFrequency` is `true`: inserts stack into QIO Frequency; returns remainder.
/// - If `insertIntoFrequency` is `false`: moves stack between hotbar and main inventory.
pub fn route_shift_click<F: QioStorageView>(
    state: &DashboardState,
    stack: ItemStack,
    frequency: &mut F,
    player_inventory: &mut [Option<ItemStack>], // slots 0..36 (0..9 hotbar, 9..36 main)
    from_hotbar: bool,
    max_stack: u32,
) -> Option<ItemStack> {
    if state.insert_into_frequency {
        let inserted = frequency.add_item(&stack.item, stack.count, &stack.components);
        if inserted >= stack.count {
            None
        } else {
            Some(ItemStack {
                item: stack.item,
                count: stack.count - inserted,
                components: stack.components,
            })
        }
    } else {
        // Move between player inventory sections:
        // Hotbar (0..9) -> Main (9..36), or Main (9..36) -> Hotbar (0..9)
        let target_range = if from_hotbar { 9..36 } else { 0..9 };
        let mut remaining = stack.count;

        // Try merge
        for idx in target_range.clone() {
            if remaining == 0 {
                break;
            }
            if let Some(s) = &mut player_inventory[idx] {
                if s.item == stack.item && s.components == stack.components && s.count < max_stack {
                    let room = max_stack - s.count;
                    let add = room.min(remaining);
                    s.count += add;
                    remaining -= add;
                }
            }
        }

        // Try empty slots
        for idx in target_range {
            if remaining == 0 {
                break;
            }
            if player_inventory[idx].is_none() {
                let add = max_stack.min(remaining);
                player_inventory[idx] = Some(ItemStack {
                    item: stack.item.clone(),
                    count: add,
                    components: stack.components.clone(),
                });
                remaining -= add;
            }
        }

        if remaining == 0 {
            None
        } else {
            Some(ItemStack {
                item: stack.item,
                count: remaining,
                components: stack.components,
            })
        }
    }
}

// ============================================================================
// Catalog Actions (Virtual Item Viewer: PacketQIOItemViewerSlot*)
// ============================================================================

#[derive(Clone, Debug, PartialEq)]
pub enum CatalogAction {
    /// Takes 1 item (`PacketQIOItemViewerSlotTake.java`).
    TakeSingle { item: String, components: String },
    /// Takes half a stack (`min(stored, max_stack / 2)`).
    TakeHalf { item: String, components: String, max_stack: u32 },
    /// Takes up to 1 full stack (`min(stored, max_stack)`).
    TakeMax { item: String, components: String, max_stack: u32 },
    /// Shift-takes full stack directly into player inventory (`PacketQIOItemViewerSlotShiftTake.java`).
    ShiftTake { item: String, components: String, max_stack: u32 },
    /// Places cursor stack into frequency (`PacketQIOItemViewerSlotPlace.java`).
    Place { stack: ItemStack },
    /// Clears window inputs back to frequency (`PacketQIOClearCraftingWindow.java`).
    ClearCraftingWindow { window_index: usize },
}

/// Executes a catalog viewer action.
pub fn execute_catalog_action<F: QioStorageView>(
    action: CatalogAction,
    state: &mut DashboardState,
    frequency: &mut F,
    cursor_item: &mut Option<ItemStack>,
    player_inventory: &mut [Option<ItemStack>],
) {
    match action {
        CatalogAction::TakeSingle { item, components } => {
            if cursor_item.is_none() {
                let taken = frequency.extract_item(&item, 1, &components);
                if taken > 0 {
                    *cursor_item = Some(ItemStack { item, count: taken, components });
                }
            } else if let Some(c) = cursor_item {
                if c.item == item && c.components == components {
                    let taken = frequency.extract_item(&item, 1, &components);
                    c.count += taken;
                }
            }
        }
        CatalogAction::TakeHalf { item, components, max_stack } => {
            let half = (max_stack / 2).max(1);
            if cursor_item.is_none() {
                let taken = frequency.extract_item(&item, half, &components);
                if taken > 0 {
                    *cursor_item = Some(ItemStack { item, count: taken, components });
                }
            }
        }
        CatalogAction::TakeMax { item, components, max_stack } => {
            if cursor_item.is_none() {
                let taken = frequency.extract_item(&item, max_stack, &components);
                if taken > 0 {
                    *cursor_item = Some(ItemStack { item, count: taken, components });
                }
            }
        }
        CatalogAction::ShiftTake { item, components, max_stack } => {
            let stored = frequency.get_stored_count(&item, &components);
            if stored > 0 {
                let to_take = (stored.min(u64::from(max_stack))) as u32;
                let taken = frequency.extract_item(&item, to_take, &components);
                if taken > 0 {
                    let stack = ItemStack { item, count: taken, components };
                    let mut remaining = stack.count;

                    // Insert into player inventory
                    for slot in player_inventory.iter_mut() {
                        if remaining == 0 {
                            break;
                        }
                        if let Some(s) = slot {
                            if s.item == stack.item && s.components == stack.components && s.count < max_stack {
                                let add = (max_stack - s.count).min(remaining);
                                s.count += add;
                                remaining -= add;
                            }
                        }
                    }
                    for slot in player_inventory.iter_mut() {
                        if remaining == 0 {
                            break;
                        }
                        if slot.is_none() {
                            let add = max_stack.min(remaining);
                            *slot = Some(ItemStack {
                                item: stack.item.clone(),
                                count: add,
                                components: stack.components.clone(),
                            });
                            remaining -= add;
                        }
                    }

                    // Return any uncollected items to frequency
                    if remaining > 0 {
                        frequency.add_item(&stack.item, remaining, &stack.components);
                    }
                }
            }
        }
        CatalogAction::Place { stack } => {
            let inserted = frequency.add_item(&stack.item, stack.count, &stack.components);
            if inserted >= stack.count {
                *cursor_item = None;
            } else if let Some(c) = cursor_item {
                c.count -= inserted;
            }
        }
        CatalogAction::ClearCraftingWindow { window_index } => {
            if window_index < MAX_CRAFTING_WINDOWS {
                let items = state.windows[window_index].clear_inputs();
                for item in items {
                    let remaining = item.count - frequency.add_item(&item.item, item.count, &item.components);
                    if remaining > 0 {
                        // Place uninserted back in window
                        state.windows[window_index].inputs[0] = Some(ItemStack {
                            item: item.item,
                            count: remaining,
                            components: item.components,
                        });
                    }
                }
            }
        }
    }
}

// ============================================================================
// Unit Tests (Comparing with Java reference sources)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    struct MockQioFrequency {
        items: std::collections::HashMap<(String, String), u64>,
    }

    impl MockQioFrequency {
        fn new() -> Self {
            Self { items: std::collections::HashMap::new() }
        }
    }

    impl QioStorageView for MockQioFrequency {
        fn add_item(&mut self, item: &str, count: u32, components: &str) -> u32 {
            let key = (item.to_string(), components.to_string());
            let current = self.items.entry(key).or_insert(0);
            *current += u64::from(count);
            count
        }

        fn extract_item(&mut self, item: &str, count: u32, components: &str) -> u32 {
            let key = (item.to_string(), components.to_string());
            if let Some(entry) = self.items.get_mut(&key) {
                let to_take = (*entry).min(u64::from(count)) as u32;
                *entry -= u64::from(to_take);
                to_take
            } else {
                0
            }
        }

        fn get_stored_count(&self, item: &str, components: &str) -> u64 {
            self.items.get(&(item.to_string(), components.to_string())).copied().unwrap_or(0)
        }

        fn get_all_stored(&self) -> Vec<(String, String, u64)> {
            self.items.iter().map(|((i, c), n)| (i.clone(), c.clone(), *n)).collect()
        }
    }

    #[test]
    fn test_total_dashboard_slots_is_30() {
        // Java reference: `TileEntityQIODashboard.java:66-73`
        // 3 windows x (9 crafting input slots + 1 crafting output slot) = 30 slots
        assert_eq!(MAX_CRAFTING_WINDOWS, 3);
        assert_eq!(CRAFTING_GRID_SIZE, 9);
        assert_eq!(SLOTS_PER_WINDOW, 10);
        assert_eq!(TOTAL_INTERNAL_SLOTS, 30);
    }

    #[test]
    fn test_shift_click_routing_frequency_vs_inventory() {
        // Java reference: `TileEntityQIODashboard.java:172-178`
        let mut freq = MockQioFrequency::new();
        let mut player_inv = vec![None; 36];

        // 1. insert_into_frequency = true
        let state_freq = DashboardState {
            insert_into_frequency: true,
            ..Default::default()
        };
        let stack = ItemStack { item: "minecraft:iron_ingot".into(), count: 64, components: String::new() };
        let remainder = route_shift_click(&state_freq, stack, &mut freq, &mut player_inv, false, 64);
        assert!(remainder.is_none());
        assert_eq!(freq.get_stored_count("minecraft:iron_ingot", ""), 64);

        // 2. insert_into_frequency = false (moves to player inventory hotbar / main)
        let state_inv = DashboardState {
            insert_into_frequency: false,
            ..Default::default()
        };
        let stack2 = ItemStack { item: "minecraft:gold_ingot".into(), count: 32, components: String::new() };
        // Shift click from main inventory (from_hotbar = false) -> should land in hotbar (0..9)
        let remainder2 = route_shift_click(&state_inv, stack2, &mut freq, &mut player_inv, false, 64);
        assert!(remainder2.is_none());
        assert_eq!(freq.get_stored_count("minecraft:gold_ingot", ""), 0); // Not added to frequency!
        assert!(player_inv[0].is_some());
        assert_eq!(player_inv[0].as_ref().unwrap().item, "minecraft:gold_ingot");
        assert_eq!(player_inv[0].as_ref().unwrap().count, 32);
    }

    #[test]
    fn test_clear_crafting_window_returns_items_to_frequency() {
        // Java reference: `PacketQIOClearCraftingWindow.java`
        let mut freq = MockQioFrequency::new();
        let mut state = DashboardState::default();

        state.windows[0].inputs[0] = Some(ItemStack { item: "minecraft:oak_planks".into(), count: 4, components: String::new() });
        state.windows[0].inputs[1] = Some(ItemStack { item: "minecraft:stick".into(), count: 2, components: String::new() });

        let mut cursor = None;
        let mut player_inv = vec![None; 36];
        execute_catalog_action(
            CatalogAction::ClearCraftingWindow { window_index: 0 },
            &mut state,
            &mut freq,
            &mut cursor,
            &mut player_inv,
        );

        assert!(state.windows[0].inputs[0].is_none());
        assert!(state.windows[0].inputs[1].is_none());
        assert_eq!(freq.get_stored_count("minecraft:oak_planks", ""), 4);
        assert_eq!(freq.get_stored_count("minecraft:stick", ""), 2);
    }

    #[test]
    fn test_catalog_take_actions() {
        let mut freq = MockQioFrequency::new();
        freq.add_item("minecraft:diamond", 100, "");

        let mut state = DashboardState::default();
        let mut cursor = None;
        let mut player_inv = vec![None; 36];

        // Take single: extracts 1
        execute_catalog_action(
            CatalogAction::TakeSingle { item: "minecraft:diamond".into(), components: String::new() },
            &mut state,
            &mut freq,
            &mut cursor,
            &mut player_inv,
        );
        assert_eq!(cursor.as_ref().unwrap().count, 1);
        assert_eq!(freq.get_stored_count("minecraft:diamond", ""), 99);

        // Clear cursor
        cursor = None;

        // Take half of max stack (64 / 2 = 32)
        execute_catalog_action(
            CatalogAction::TakeHalf { item: "minecraft:diamond".into(), components: String::new(), max_stack: 64 },
            &mut state,
            &mut freq,
            &mut cursor,
            &mut player_inv,
        );
        assert_eq!(cursor.as_ref().unwrap().count, 32);
        assert_eq!(freq.get_stored_count("minecraft:diamond", ""), 67);

        // Clear cursor
        cursor = None;

        // Take full stack (64)
        execute_catalog_action(
            CatalogAction::TakeMax { item: "minecraft:diamond".into(), components: String::new(), max_stack: 64 },
            &mut state,
            &mut freq,
            &mut cursor,
            &mut player_inv,
        );
        assert_eq!(cursor.as_ref().unwrap().count, 64);
        assert_eq!(freq.get_stored_count("minecraft:diamond", ""), 3);
    }

    #[test]
    fn test_dashboard_serialization_roundtrip() {
        let mut state = DashboardState::default();
        state.insert_into_frequency = false;
        state.frequency_name = "BaseNet".into();
        state.facing = "west".into();
        state.windows[1].inputs[4] = Some(ItemStack {
            item: "mekanism:alloy_infused".into(),
            count: 8,
            components: "{\"foo\":\"bar\"}".into(),
        });
        state.windows[1].output = Some(ItemStack {
            item: "mekanism:advanced_control_circuit".into(),
            count: 1,
            components: String::new(),
        });

        let encoded = encode_dashboard(&state);
        let decoded = decode_dashboard(&encoded).expect("decode dashboard");
        assert_eq!(state, decoded);
    }
}

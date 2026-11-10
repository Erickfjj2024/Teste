//! Modification Station (`TileEntityModificationStation`).
//!
//! Reference: Mekanism Java 1.21.1-10.7.19.85
//! - Tile Entity: `java/main/java/mekanism/common/tile/TileEntityModificationStation.java`
//! - GUI: `java/main/java/mekanism/client/gui/GuiModificationStation.java`
//! - Configs:
//!   - Storage: `StorageConfig.java:104` (40,000 Joules = 16,000 FE)
//!   - Usage: `UsageConfig.java:74` (400 Joules/tick = 160 FE/tick)
//!   - Ticks: `TileEntityModificationStation.java:42` (10 ticks, `MekanismUtils.TICKS_PER_HALF_SECOND`)
//!   - Total Energy per module: 1,600 FE (4,000 Joules)
//! - Module Registry & Data: `java/main/java/mekanism/common/registries/MekanismModules.java`, `ModuleContainer.java`
//!
//! Features:
//! - 2x2x1 Multiblock Footprint: Master at `pos`, 3 bounding blocks at `pos + UP`, `pos + RIGHT(facing)`, `pos + RIGHT(facing) + UP`.
//! - Slots:
//!   - Slot 0: `moduleSlot` ($X=35, Y=118$)
//!   - Slot 1: `containerSlot` ($X=125, Y=118$)
//!   - Slot 2: `energySlot` ($X=151, Y=21$)
//! - Upgrades: Upgrades NOT supported (neither Speed nor Energy).
//! - Installs modules into gear item data components over 10 ticks consuming 160 FE/t.
//! - Uninstalls modules (`PacketRemoveModule.java`) returning module items to player.
use super::*;
use horizon_mod_sdk::server::{player_items, world, BlockEntity, BlockEntityUpdate, ItemStack, Player};
use std::collections::HashMap;

pub const BLOCK: &str = "mekanism:modification_station";

/// Slot indices (`TileEntityModificationStation.java:75-80`).
pub const MODULE_SLOT: usize = 0;
pub const CONTAINER_SLOT: usize = 1;
pub const ENERGY_SLOT: usize = 2;
pub const TOTAL_SLOTS: usize = 3;

/// Energy constants (Java in Joules, port in Forge Energy: FE = Joules * 2 / 5).
/// Storage: `StorageConfig.java:104` -> 40,000 J = 16,000 FE
pub const ENERGY_CAPACITY: u64 = crate::fe(40_000);

/// Energy consumption: `UsageConfig.java:74` -> 400 J/t = 160 FE/t
pub const ENERGY_USAGE: u64 = crate::fe(400);

/// Operation duration: `TileEntityModificationStation.java:42` (`MekanismUtils.TICKS_PER_HALF_SECOND = 10`)
pub const TICKS_REQUIRED: u32 = 10;

/// Total energy per module installation: 10 * 160 = 1,600 FE (4,000 Joules)
pub const TOTAL_ENERGY_PER_OPERATION: u64 = ENERGY_USAGE * (TICKS_REQUIRED as u64);

// ============================================================================
// Multiblock 2x2x1 Geometry (TileEntityModificationStation.java, IBoundingBlock)
// ============================================================================

/// Returns relative bounding block offsets based on horizontal facing.
///
/// Footprint (4 blocks total):
/// 1. Base: `[0, 0, 0]` (master block)
/// 2. Superior: `[0, 1, 0]`
/// 3. Lateral Direita: `[rx, 0, rz]`
/// 4. Superior Direita: `[rx, 1, rz]`
pub fn bounding_offsets(facing: &str) -> [[i32; 3]; 3] {
    let right = match facing {
        "north" => [1, 0, 0],  // East is right of North
        "south" => [-1, 0, 0], // West is right of South
        "east" => [0, 0, 1],   // South is right of East
        "west" => [0, 0, -1],  // North is right of West
        _ => [1, 0, 0],
    };
    [
        [0, 1, 0],
        [right[0], 0, right[2]],
        [right[0], 1, right[2]],
    ]
}

// ============================================================================
// Module Definitions & Compatibility Registry (MekanismModules.java)
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GearType {
    Helmet,
    BodyArmor,
    Pants,
    Boots,
    MekaTool,
    AnyArmor,
    AnyGear,
}

/// Identifies gear category from item ID.
pub fn gear_type_of(item: &str) -> Option<GearType> {
    match item {
        "mekanism:mekasuit_helmet" => Some(GearType::Helmet),
        "mekanism:mekasuit_bodyarmor" => Some(GearType::BodyArmor),
        "mekanism:mekasuit_pants" => Some(GearType::Pants),
        "mekanism:mekasuit_boots" => Some(GearType::Boots),
        "mekanism:meka_tool" => Some(GearType::MekaTool),
        _ => None,
    }
}

/// Module definition with max stack size and target equipment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleDefinition {
    pub max_stack_size: u32,
    pub target: GearType,
}

/// Module registry metadata from `MekanismModules.java`.
pub fn module_info(module_id: &str) -> Option<ModuleDefinition> {
    let def = match module_id {
        // Shared / All gear
        "mekanism:module_energy_unit" => ModuleDefinition { max_stack_size: 8, target: GearType::AnyGear },
        // Shared Armor
        "mekanism:module_color_modulation_unit" => ModuleDefinition { max_stack_size: 1, target: GearType::AnyArmor },
        "mekanism:module_laser_dissipation_unit" => ModuleDefinition { max_stack_size: 1, target: GearType::AnyArmor },
        "mekanism:module_radiation_shielding_unit" => ModuleDefinition { max_stack_size: 1, target: GearType::AnyArmor },
        // Meka-Tool
        "mekanism:module_attack_amplification_unit" => ModuleDefinition { max_stack_size: 4, target: GearType::MekaTool },
        "mekanism:module_blasting_unit" => ModuleDefinition { max_stack_size: 4, target: GearType::MekaTool },
        "mekanism:module_excavation_escalation_unit" => ModuleDefinition { max_stack_size: 4, target: GearType::MekaTool },
        "mekanism:module_farming_unit" => ModuleDefinition { max_stack_size: 4, target: GearType::MekaTool },
        "mekanism:module_shearing_unit" => ModuleDefinition { max_stack_size: 1, target: GearType::MekaTool },
        "mekanism:module_silk_touch_unit" => ModuleDefinition { max_stack_size: 1, target: GearType::MekaTool },
        "mekanism:module_fortune_unit" => ModuleDefinition { max_stack_size: 3, target: GearType::MekaTool },
        "mekanism:module_teleportation_unit" => ModuleDefinition { max_stack_size: 1, target: GearType::MekaTool },
        "mekanism:module_vein_mining_unit" => ModuleDefinition { max_stack_size: 4, target: GearType::MekaTool },
        // Helmet
        "mekanism:module_electrolytic_breathing_unit" => ModuleDefinition { max_stack_size: 4, target: GearType::Helmet },
        "mekanism:module_inhalation_purification_unit" => ModuleDefinition { max_stack_size: 1, target: GearType::Helmet },
        "mekanism:module_vision_enhancement_unit" => ModuleDefinition { max_stack_size: 4, target: GearType::Helmet },
        "mekanism:module_nutritional_injection_unit" => ModuleDefinition { max_stack_size: 1, target: GearType::Helmet },
        // BodyArmor
        "mekanism:module_dosimeter_unit" => ModuleDefinition { max_stack_size: 1, target: GearType::BodyArmor },
        "mekanism:module_geiger_unit" => ModuleDefinition { max_stack_size: 1, target: GearType::BodyArmor },
        "mekanism:module_jetpack_unit" => ModuleDefinition { max_stack_size: 4, target: GearType::BodyArmor },
        "mekanism:module_charge_distribution_unit" => ModuleDefinition { max_stack_size: 1, target: GearType::BodyArmor },
        "mekanism:module_gravitational_modulating_unit" => ModuleDefinition { max_stack_size: 1, target: GearType::BodyArmor },
        "mekanism:module_elytra_unit" => ModuleDefinition { max_stack_size: 1, target: GearType::BodyArmor },
        // Pants
        "mekanism:module_locomotive_boosting_unit" => ModuleDefinition { max_stack_size: 4, target: GearType::Pants },
        "mekanism:module_gyroscopic_stabilization_unit" => ModuleDefinition { max_stack_size: 1, target: GearType::Pants },
        "mekanism:module_hydrostatic_repulsor_unit" => ModuleDefinition { max_stack_size: 4, target: GearType::Pants },
        "mekanism:module_motorized_servo_unit" => ModuleDefinition { max_stack_size: 5, target: GearType::Pants },
        // Boots
        "mekanism:module_hydraulic_propulsion_unit" => ModuleDefinition { max_stack_size: 4, target: GearType::Boots },
        "mekanism:module_magnetic_attraction_unit" => ModuleDefinition { max_stack_size: 4, target: GearType::Boots },
        "mekanism:module_frost_walker_unit" => ModuleDefinition { max_stack_size: 2, target: GearType::Boots },
        "mekanism:module_soul_surfer_unit" => ModuleDefinition { max_stack_size: 3, target: GearType::Boots },
        _ => return None,
    };
    Some(def)
}

/// Checks if a module can be installed on a piece of gear.
pub fn is_module_compatible(module_id: &str, gear_item: &str) -> bool {
    let gear = match gear_type_of(gear_item) {
        Some(g) => g,
        None => return false,
    };
    let mod_def = match module_info(module_id) {
        Some(m) => m,
        None => return false,
    };

    match mod_def.target {
        GearType::AnyGear => true,
        GearType::AnyArmor => matches!(gear, GearType::Helmet | GearType::BodyArmor | GearType::Pants | GearType::Boots),
        target => gear == target,
    }
}

// ============================================================================
// Gear Module Container Data Component (ModuleContainer.java)
// ============================================================================

/// Parsed installed modules on an item stack.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct InstalledModuleContainer {
    pub modules: HashMap<String, u32>,
}

impl InstalledModuleContainer {
    pub fn parse(components: &str) -> Self {
        let mut modules = HashMap::new();
        // Simple JSON / KV parsing for `modules: {"module_id": count, ...}`
        if let Some(pos) = components.find("\"modules\":") {
            let after_modules = &components[pos + 10..];
            if let Some(brace_start) = after_modules.find('{') {
                let rest = &after_modules[brace_start + 1..];
                if let Some(end) = rest.find('}') {
                    let inner = &rest[..end];
                    for part in inner.split(',') {
                        if let Some((k, v)) = part.rsplit_once(':') {
                            let key = k.trim().trim_matches('"');
                            if let Ok(count) = v.trim().parse::<u32>() {
                                if !key.is_empty() && count > 0 {
                                    modules.insert(key.to_string(), count);
                                }
                            }
                        }
                    }
                }
            }
        }
        Self { modules }
    }

    pub fn serialize(&self) -> String {
        let mut parts = Vec::new();
        let mut sorted: Vec<_> = self.modules.iter().collect();
        sorted.sort_by_key(|(k, _)| (*k).clone());
        for (k, v) in sorted {
            parts.push(format!("\"{k}\":{v}"));
        }
        format!("{{\"modules\":{{{}}}}}", parts.join(","))
    }

    pub fn installed_count(&self, module_id: &str) -> u32 {
        self.modules.get(module_id).copied().unwrap_or(0)
    }

    pub fn can_install(&self, module_id: &str, gear_item: &str) -> bool {
        if !is_module_compatible(module_id, gear_item) {
            return false;
        }
        let info = match module_info(module_id) {
            Some(i) => i,
            None => return false,
        };
        self.installed_count(module_id) < info.max_stack_size
    }

    pub fn add_module(&mut self, module_id: &str, gear_item: &str, count: u32) -> u32 {
        if !is_module_compatible(module_id, gear_item) {
            return 0;
        }
        let info = match module_info(module_id) {
            Some(i) => i,
            None => return 0,
        };
        let current = self.installed_count(module_id);
        let room = info.max_stack_size.saturating_sub(current);
        let added = room.min(count);
        if added > 0 {
            self.modules.insert(module_id.to_string(), current + added);
        }
        added
    }

    pub fn remove_module(&mut self, module_id: &str, count: u32) -> u32 {
        let current = self.installed_count(module_id);
        if current == 0 {
            return 0;
        }
        let removed = current.min(count);
        let remaining = current - removed;
        if remaining > 0 {
            self.modules.insert(module_id.to_string(), remaining);
        } else {
            self.modules.remove(module_id);
        }
        removed
    }
}

// ============================================================================
// Modification Station State & Server Tick (TileEntityModificationStation.java)
// ============================================================================

#[derive(Clone, Debug, PartialEq)]
pub struct ModificationStationState {
    pub energy: u64,
    pub operating_ticks: u32,
    pub used_energy: bool,
    pub slots: [Option<ItemStack>; TOTAL_SLOTS],
    pub facing: String,
}

impl Default for ModificationStationState {
    fn default() -> Self {
        Self {
            energy: 0,
            operating_ticks: 0,
            used_energy: false,
            slots: [None, None, None],
            facing: "north".into(),
        }
    }
}

/// Executes one server tick of Modification Station logic.
pub fn step_modification_station(state: &mut ModificationStationState, can_function: bool) -> bool {
    // 1. Charge energy buffer from slot 2 (if battery present)
    if let Some(energy_stack) = &mut state.slots[ENERGY_SLOT] {
        if energy_stack.item == "mekanism:energy_tablet" && state.energy < ENERGY_CAPACITY {
            let room = ENERGY_CAPACITY - state.energy;
            let charge = room.min(10_000);
            state.energy += charge;
        }
    }

    state.used_energy = false;
    if !can_function {
        state.operating_ticks = 0;
        return false;
    }

    // 2. Check installation requirements
    let has_energy = state.energy >= ENERGY_USAGE;
    let has_module = state.slots[MODULE_SLOT].is_some();
    let has_gear = state.slots[CONTAINER_SLOT].is_some();

    if !has_energy || !has_module || !has_gear {
        state.operating_ticks = 0;
        return false;
    }

    let module_item = state.slots[MODULE_SLOT].as_ref().unwrap().item.clone();
    let gear_stack = state.slots[CONTAINER_SLOT].as_ref().unwrap().clone();
    let mut container = InstalledModuleContainer::parse(&gear_stack.components);

    if !container.can_install(&module_item, &gear_stack.item) {
        state.operating_ticks = 0;
        return false;
    }

    // 3. Process operation tick
    state.energy -= ENERGY_USAGE;
    state.operating_ticks += 1;
    state.used_energy = true;

    // 4. Complete operation at 10 ticks
    if state.operating_ticks >= TICKS_REQUIRED {
        state.operating_ticks = 0;
        let added = container.add_module(&module_item, &gear_stack.item, 1);
        if added > 0 {
            // Update gear component
            if let Some(gear) = &mut state.slots[CONTAINER_SLOT] {
                gear.components = container.serialize();
            }
            // Shrink module input stack
            if let Some(module) = &mut state.slots[MODULE_SLOT] {
                if module.count <= 1 {
                    state.slots[MODULE_SLOT] = None;
                } else {
                    module.count -= 1;
                }
            }
        }
    }

    true
}

/// Handles player module uninstallation (`PacketRemoveModule.java`).
pub fn remove_module_from_container(
    state: &mut ModificationStationState,
    module_id: &str,
    remove_all: bool,
) -> Option<ItemStack> {
    let gear_stack = state.slots[CONTAINER_SLOT].as_ref()?.clone();
    let mut container = InstalledModuleContainer::parse(&gear_stack.components);

    let installed = container.installed_count(module_id);
    if installed == 0 {
        return None;
    }

    let to_remove = if remove_all { installed } else { 1 };
    let removed = container.remove_module(module_id, to_remove);
    if removed > 0 {
        if let Some(gear) = &mut state.slots[CONTAINER_SLOT] {
            gear.components = container.serialize();
        }
        Some(ItemStack {
            item: module_id.to_string(),
            count: removed,
            components: String::new(),
        })
    } else {
        None
    }
}

// ============================================================================
// Serialization
// ============================================================================

pub fn encode_modification_station(s: &ModificationStationState) -> Vec<u8> {
    let mut out = vec![
        1, // version
        u8::from(s.used_energy),
    ];
    out.extend_from_slice(&s.energy.to_le_bytes());
    out.extend_from_slice(&s.operating_ticks.to_le_bytes());

    let fb = s.facing.as_bytes();
    out.push(fb.len().min(255) as u8);
    out.extend_from_slice(&fb[..fb.len().min(255)]);

    for slot in &s.slots {
        match slot {
            None => out.push(0),
            Some(stack) => {
                out.push(1);
                let ib = stack.item.as_bytes();
                out.push(ib.len().min(255) as u8);
                out.extend_from_slice(&ib[..ib.len().min(255)]);
                out.extend_from_slice(&stack.count.to_le_bytes());
                let cb = stack.components.as_bytes();
                out.push(cb.len().min(255) as u8);
                out.extend_from_slice(&cb[..cb.len().min(255)]);
            }
        }
    }
    out
}

pub fn decode_modification_station(data: &[u8]) -> Option<ModificationStationState> {
    if data.len() < 15 || data[0] != 1 {
        return None;
    }
    let used_energy = data[1] == 1;
    let mut e = [0u8; 8];
    e.copy_from_slice(&data[2..10]);
    let energy = u64::from_le_bytes(e);

    let mut t = [0u8; 4];
    t.copy_from_slice(&data[10..14]);
    let operating_ticks = u32::from_le_bytes(t);

    let mut idx = 14;
    let flen = data[idx] as usize;
    idx += 1;
    if idx + flen > data.len() {
        return None;
    }
    let facing = String::from_utf8_lossy(&data[idx..idx + flen]).to_string();
    idx += flen;

    let mut slots = [None, None, None];
    for slot in &mut slots {
        if idx >= data.len() {
            break;
        }
        let has_item = data[idx] == 1;
        idx += 1;
        if has_item {
            if idx >= data.len() {
                break;
            }
            let ilen = data[idx] as usize;
            idx += 1;
            if idx + ilen + 4 > data.len() {
                break;
            }
            let item = String::from_utf8_lossy(&data[idx..idx + ilen]).to_string();
            idx += ilen;

            let count = u32::from_le_bytes([data[idx], data[idx + 1], data[idx + 2], data[idx + 3]]);
            idx += 4;

            if idx >= data.len() {
                break;
            }
            let clen = data[idx] as usize;
            idx += 1;
            if idx + clen > data.len() {
                break;
            }
            let components = String::from_utf8_lossy(&data[idx..idx + clen]).to_string();
            idx += clen;

            *slot = Some(ItemStack { item, count, components });
        }
    }

    Some(ModificationStationState {
        energy,
        operating_ticks,
        used_energy,
        slots,
        facing,
    })
}

// ============================================================================
// Unit Tests (Comparing with Java reference sources)
// ============================================================================


// ============================================================================
// In the world: the block entity (`onUpdateServer`, `removeModule`), on the module system of `modules.rs`
// (the one the MekaSuit and the Meka-Tool read), not on the container of this file's own pure model above.
// ============================================================================

use crate::modules::{ModuleContainer, ModuleType};

/// The buttons of its screen: the redstone tab, a row of the module list (`BUTTON_REMOVE_ONE_BASE + row`: take one out) and the same with Shift
/// (`BUTTON_REMOVE_ALL_BASE + row`: take them all out). The original selects a module in a scrolling list and presses "Remove" (Shift removes all).
pub const BUTTON_REMOVE_ONE_BASE: i32 = 7800;
pub const BUTTON_REMOVE_ALL_BASE: i32 = 7820;
/// How many rows of the list the screen shows.
pub const LIST_ROWS: usize = 6;

/// The name a module shows (`module_energy_unit` -> `Energy Unit`, the module's own name in the original's language file).
pub fn module_title(t: ModuleType) -> String {
    let id = t.item_id();
    let path = id.split(':').nth(1).unwrap_or(id).trim_start_matches("module_");
    path.split('_')
        .map(|w| {
            let mut c = w.chars();
            c.next().map_or_else(String::new, |f| f.to_uppercase().collect::<String>() + c.as_str())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The modules installed on a gear item, in the order the list shows them (by item id).
pub fn installed_list(container: &ModuleContainer) -> Vec<(ModuleType, u8)> {
    let mut v: Vec<(ModuleType, u8)> = container.modules.iter().map(|(t, m)| (*t, m.installed)).collect();
    v.sort_by_key(|(t, _)| t.item_id());
    v
}

/// The strings of the screen, one per row of the list (`GuiModuleScrollList`): the module and how many are installed.
pub fn list_texts(container_stack: Option<&ItemStack>) -> Vec<String> {
    let list = container_stack.map(|s| installed_list(&ModuleContainer::decode_from_components(&s.components))).unwrap_or_default();
    (0..LIST_ROWS).map(|i| list.get(i).map_or_else(String::new, |(t, n)| format!("{} x{}", module_title(*t), n))).collect()
}

/// The numbers of the screen: 0 ticks worked, 1 ticks an installation takes, 2-3 energy, 4-5 capacity, 6 the redstone control, 7-8 the energy still
/// needed, 9 the energy a tick while it works, 10 how many different modules the gear holds.
pub fn menu_numbers(ticks: u32, control: u8, energy: u64, working: bool, installed: usize) -> Vec<i32> {
    let half = |v: u64, s: u32| ((v >> s) & 0xFFFF) as i32;
    let needed = ENERGY_CAPACITY.saturating_sub(energy);
    vec![
        ticks as i32,
        TICKS_REQUIRED as i32,
        half(energy, 0),
        half(energy, 16),
        half(ENERGY_CAPACITY, 0),
        half(ENERGY_CAPACITY, 16),
        i32::from(control),
        half(needed, 0),
        half(needed, 16),
        if working { ENERGY_USAGE as i32 } else { 0 },
        installed as i32,
    ]
}

/// The station's saved bytes: ticks worked (u32) and the redstone control.
fn station_data(ticks: u32, control: Control) -> Vec<u8> {
    let mut v = vec![1u8];
    v.extend(ticks.to_le_bytes());
    v.push(control as u8);
    v
}

fn read_station(data: &[u8]) -> (u32, Control) {
    if data.len() >= 6 && data[0] == 1 {
        (u32::from_le_bytes([data[1], data[2], data[3], data[4]]), Control::from_byte(data[5]))
    } else {
        (0, Control::Disabled)
    }
}

/// One tick of the installation (`onUpdateServer`, java:92-118), without the block entity: returns what happened to the slots.
/// `slots` are the module, the gear and the charge slot. Says whether it worked this tick, and gives the new ticks.
pub fn step_station(slots: &mut [Option<ItemStack>], energy: &mut u64, ticks: &mut u32, can_function: bool) -> bool {
    let mut operated = false;
    if can_function && *energy >= ENERGY_USAGE {
        if let (Some(module), Some(gear)) = (slots[MODULE_SLOT].clone(), slots[CONTAINER_SLOT].clone()) {
            if let Some(kind) = ModuleType::from_item(&module.item) {
                let mut container = ModuleContainer::decode_from_components(&gear.components);
                // The gear takes this module and has room for one more of it (`canInstall`).
                if container.can_install(&gear.item, kind) {
                    operated = true;
                    *ticks += 1;
                    *energy -= ENERGY_USAGE;
                    if *ticks == TICKS_REQUIRED {
                        *ticks = 0;
                        let added = container.add_module(&gear.item, kind, module.count.min(255) as u8);
                        if added > 0 {
                            let mut new_gear = gear.clone();
                            new_gear.components = container.encode_to_components(&gear.components);
                            slots[CONTAINER_SLOT] = Some(new_gear);
                            let left = module.count - u32::from(added);
                            slots[MODULE_SLOT] = if left == 0 { None } else { Some(ItemStack { count: left, ..module }) };
                        }
                    }
                }
            }
        }
    }
    if !operated {
        *ticks = 0;
    }
    operated
}

impl Mekanism {
    /// A tick of the modification stations.
    pub(crate) fn run_modification_stations(&mut self, stations: &[&BlockEntity]) -> Vec<BlockEntityUpdate> {
        let mut out = Vec::new();
        for e in stations {
            let (mut ticks, control) = read_station(&e.data);
            let mut slots: Vec<Option<ItemStack>> = (0..TOTAL_SLOTS).map(|i| e.items.get(i).cloned().flatten()).collect();
            let mut energy = e.energy;
            let mut slot_changed = false;
            // The charge slot (`fillContainerOrConvert`).
            if let Some((gain, left)) = crate::cube::fill_from_slot(&slots[ENERGY_SLOT], energy, ENERGY_CAPACITY) {
                energy = gain;
                slots[ENERGY_SLOT] = left;
                slot_changed = true;
            }
            let key = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
            let powered = world::get_redstone(&e.dimension, e.pos.x, e.pos.y, e.pos.z) > 0;
            let last = self.last_power.insert(key.clone(), powered).unwrap_or(false);
            let before = slots.clone();
            let operated = step_station(&mut slots, &mut energy, &mut ticks, control.can_function(powered, last));
            slot_changed |= slots != before;

            let texts = list_texts(slots[CONTAINER_SLOT].as_ref());
            if self.menu_texts.get(&key) != Some(&texts) {
                let _ = world::set_menu_text(&e.dimension, e.pos.x, e.pos.y, e.pos.z, &texts);
                self.menu_texts.insert(key, texts.clone());
            }
            let installed = texts.iter().filter(|t| !t.is_empty()).count();
            let numbers = menu_numbers(ticks, control as u8, energy, operated, installed);
            if slot_changed || energy != e.energy || e.menu_data != numbers || e.data != station_data(ticks, control) {
                out.push(BlockEntityUpdate {
                    dimension: e.dimension.clone(),
                    pos: e.pos,
                    data: station_data(ticks, control),
                    items: if slot_changed { Some(slots) } else { None },
                    energy: Some(energy),
                    energy_faces: Some(crate::seismic_vibrator::energy_input_faces(crate::sides::property(&e.state, "facing").unwrap_or("north"))),
                    tanks: None,
                    tank_faces: None,
                    menu_data: Some(numbers),
                    transmitter_faces: None,
                });
            }
        }
        out
    }

    /// A button of the screen: the redstone tab, or a row of the list taken off the gear (`PacketRemoveModule`, `removeModule`). The module items go to
    /// the player's inventory (`Inventory.add`); with no room for all of them nothing is removed.
    pub(crate) fn modification_station_button(&mut self, player: &Player, e: &BlockEntity, id: i32) -> Option<BlockEntityUpdate> {
        let (ticks, mut control) = read_station(&e.data);
        let mut slots: Vec<Option<ItemStack>> = (0..TOTAL_SLOTS).map(|i| e.items.get(i).cloned().flatten()).collect();
        let mut slot_changed = false;
        if id == crate::redstone::NEXT {
            control = control.next();
        } else if id == crate::redstone::PREVIOUS {
            control = control.previous();
        } else {
            let (row, all) = if (BUTTON_REMOVE_ALL_BASE..BUTTON_REMOVE_ALL_BASE + LIST_ROWS as i32).contains(&id) {
                ((id - BUTTON_REMOVE_ALL_BASE) as usize, true)
            } else if (BUTTON_REMOVE_ONE_BASE..BUTTON_REMOVE_ONE_BASE + LIST_ROWS as i32).contains(&id) {
                ((id - BUTTON_REMOVE_ONE_BASE) as usize, false)
            } else {
                return None;
            };
            let gear = slots[CONTAINER_SLOT].clone()?;
            let mut container = ModuleContainer::decode_from_components(&gear.components);
            let (kind, installed) = *installed_list(&container).get(row)?;
            let count = if all { installed } else { 1 };
            if !give_to_player(&player.id, kind.item_id(), u32::from(count), u32::from(kind.max_stack())) {
                return None;
            }
            container.remove_module(kind, count);
            let mut new_gear = gear.clone();
            new_gear.components = container.encode_to_components(&gear.components);
            slots[CONTAINER_SLOT] = Some(new_gear);
            slot_changed = true;
        }
        let numbers = menu_numbers(ticks, control as u8, e.energy, false, list_texts(slots[CONTAINER_SLOT].as_ref()).iter().filter(|t| !t.is_empty()).count());
        Some(BlockEntityUpdate {
            dimension: e.dimension.clone(),
            pos: e.pos,
            data: station_data(ticks, control),
            items: if slot_changed { Some(slots) } else { None },
            energy: None,
            energy_faces: None,
            tanks: None,
            tank_faces: None,
            menu_data: Some(numbers),
            transmitter_faces: None,
        })
    }
}

/// Puts `count` of `item` in the player's inventory like `Inventory.add`: onto stacks of it with room first, then empty slots; all or nothing. The slots are written
/// with `compare-and-set-slot`, so a slot that changed since it was read is left alone (and the whole thing is then not done).
fn give_to_player(player: &str, item: &str, count: u32, max_stack: u32) -> bool {
    let Some(inv) = player_items::get_inventory(player) else { return false };
    let mut left = count;
    let mut writes: Vec<(u32, Option<ItemStack>, ItemStack)> = Vec::new();
    for (i, s) in inv.iter().enumerate().take(36) {
        if left == 0 {
            break;
        }
        if let Some(stack) = s {
            if stack.item == item && stack.components.trim().is_empty() && stack.count < max_stack {
                let add = left.min(max_stack - stack.count);
                writes.push((i as u32, Some(stack.clone()), ItemStack { count: stack.count + add, ..stack.clone() }));
                left -= add;
            }
        }
    }
    for (i, s) in inv.iter().enumerate().take(36) {
        if left == 0 {
            break;
        }
        if s.is_none() {
            let add = left.min(max_stack);
            writes.push((i as u32, None, ItemStack { item: item.to_owned(), count: add, components: String::new() }));
            left -= add;
        }
    }
    if left > 0 {
        return false;
    }
    for (slot, expected, new) in writes {
        let _ = player_items::compare_and_set_slot(player, slot, expected.as_ref(), Some(&new));
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_energy_constants_match_java() {
        // Java references:
        // Storage: `StorageConfig.java:104` (40,000 Joules = 16,000 FE)
        // Usage: `UsageConfig.java:74` (400 Joules/tick = 160 FE/tick)
        // Duration: `TileEntityModificationStation.java:42` (10 ticks)
        assert_eq!(ENERGY_CAPACITY, 16_000);
        assert_eq!(ENERGY_USAGE, 160);
        assert_eq!(TICKS_REQUIRED, 10);
        assert_eq!(TOTAL_ENERGY_PER_OPERATION, 1_600);
    }

    #[test]
    fn test_multiblock_offsets_all_facings() {
        // Footprint 2x2x1: [0, 1, 0] above, [right_x, 0, right_z] right, [right_x, 1, right_z] right above
        let north_offsets = bounding_offsets("north");
        assert_eq!(north_offsets, [[0, 1, 0], [1, 0, 0], [1, 1, 0]]);

        let south_offsets = bounding_offsets("south");
        assert_eq!(south_offsets, [[0, 1, 0], [-1, 0, 0], [-1, 1, 0]]);

        let east_offsets = bounding_offsets("east");
        assert_eq!(east_offsets, [[0, 1, 0], [0, 0, 1], [0, 1, 1]]);

        let west_offsets = bounding_offsets("west");
        assert_eq!(west_offsets, [[0, 1, 0], [0, 0, -1], [0, 1, -1]]);
    }

    #[test]
    fn test_module_compatibility_and_max_stacks() {
        // Java reference: `MekanismModules.java:61-232`
        // Energy Unit fits on any gear up to 8 stacks
        assert!(is_module_compatible("mekanism:module_energy_unit", "mekanism:meka_tool"));
        assert!(is_module_compatible("mekanism:module_energy_unit", "mekanism:mekasuit_helmet"));
        assert_eq!(module_info("mekanism:module_energy_unit").unwrap().max_stack_size, 8);

        // Jetpack Unit only fits on bodyarmor up to 4 stacks
        assert!(is_module_compatible("mekanism:module_jetpack_unit", "mekanism:mekasuit_bodyarmor"));
        assert!(!is_module_compatible("mekanism:module_jetpack_unit", "mekanism:mekasuit_helmet"));
        assert!(!is_module_compatible("mekanism:module_jetpack_unit", "mekanism:meka_tool"));
        assert_eq!(module_info("mekanism:module_jetpack_unit").unwrap().max_stack_size, 4);

        // Vein Mining Unit fits only on meka-tool up to 4 stacks
        assert!(is_module_compatible("mekanism:module_vein_mining_unit", "mekanism:meka_tool"));
        assert!(!is_module_compatible("mekanism:module_vein_mining_unit", "mekanism:mekasuit_pants"));
        assert_eq!(module_info("mekanism:module_vein_mining_unit").unwrap().max_stack_size, 4);

        // Electrolytic breathing fits only on helmet up to 4 stacks
        assert!(is_module_compatible("mekanism:module_electrolytic_breathing_unit", "mekanism:mekasuit_helmet"));
        assert!(!is_module_compatible("mekanism:module_electrolytic_breathing_unit", "mekanism:mekasuit_boots"));
        assert_eq!(module_info("mekanism:module_electrolytic_breathing_unit").unwrap().max_stack_size, 4);
    }

    #[test]
    fn test_installation_operation_cycle() {
        let mut state = ModificationStationState {
            energy: 16_000,
            operating_ticks: 0,
            used_energy: false,
            slots: [
                Some(ItemStack { item: "mekanism:module_energy_unit".into(), count: 2, components: String::new() }),
                Some(ItemStack { item: "mekanism:meka_tool".into(), count: 1, components: String::new() }),
                None,
            ],
            facing: "north".into(),
        };

        // Run 9 ticks: energy consumed, but module not installed yet
        for i in 1..=9 {
            let active = step_modification_station(&mut state, true);
            assert!(active);
            assert_eq!(state.operating_ticks, i);
            assert_eq!(state.energy, 16_000 - u64::from(i) * 160);
        }

        // 10th tick: completes installation!
        let active = step_modification_station(&mut state, true);
        assert!(active);
        assert_eq!(state.operating_ticks, 0); // resets
        assert_eq!(state.energy, 16_000 - 1_600); // 1,600 FE total consumed

        // Module slot shrunk by 1
        assert_eq!(state.slots[MODULE_SLOT].as_ref().unwrap().count, 1);

        // Gear has module installed
        let gear_components = &state.slots[CONTAINER_SLOT].as_ref().unwrap().components;
        assert!(gear_components.contains("\"mekanism:module_energy_unit\":1"));
    }

    #[test]
    fn test_remove_module_single_and_all() {
        let mut state = ModificationStationState {
            energy: 16_000,
            operating_ticks: 0,
            used_energy: false,
            slots: [
                None,
                Some(ItemStack {
                    item: "mekanism:meka_tool".into(),
                    count: 1,
                    components: "{\"modules\":{\"mekanism:module_energy_unit\":3}}".into(),
                }),
                None,
            ],
            facing: "north".into(),
        };

        // 1. Remove single: removes 1, leaves 2
        let removed_one = remove_module_from_container(&mut state, "mekanism:module_energy_unit", false);
        assert!(removed_one.is_some());
        let r1 = removed_one.unwrap();
        assert_eq!(r1.item, "mekanism:module_energy_unit");
        assert_eq!(r1.count, 1);
        assert!(state.slots[CONTAINER_SLOT].as_ref().unwrap().components.contains("\"mekanism:module_energy_unit\":2"));

        // 2. Remove all: removes remaining 2
        let removed_all = remove_module_from_container(&mut state, "mekanism:module_energy_unit", true);
        assert!(removed_all.is_some());
        let r2 = removed_all.unwrap();
        assert_eq!(r2.item, "mekanism:module_energy_unit");
        assert_eq!(r2.count, 2);
        assert!(!state.slots[CONTAINER_SLOT].as_ref().unwrap().components.contains("\"mekanism:module_energy_unit\""));
    }

    #[test]
    fn test_modification_station_serialization() {
        let state = ModificationStationState {
            energy: 12_800,
            operating_ticks: 7,
            used_energy: true,
            slots: [
                Some(ItemStack { item: "mekanism:module_jetpack_unit".into(), count: 1, components: String::new() }),
                Some(ItemStack { item: "mekanism:mekasuit_bodyarmor".into(), count: 1, components: "{\"modules\":{\"mekanism:module_jetpack_unit\":1}}".into() }),
                Some(ItemStack { item: "mekanism:energy_tablet".into(), count: 1, components: String::new() }),
            ],
            facing: "east".into(),
        };

        let encoded = encode_modification_station(&state);
        let decoded = decode_modification_station(&encoded).expect("decode mod station");

        assert_eq!(state.energy, decoded.energy);
        assert_eq!(state.operating_ticks, decoded.operating_ticks);
        assert_eq!(state.used_energy, decoded.used_energy);
        assert_eq!(state.facing, decoded.facing);
        assert_eq!(state.slots, decoded.slots);
    }

    fn gear_with(components: &str) -> ItemStack {
        ItemStack { item: "mekanism:meka_tool".into(), count: 1, components: components.into() }
    }

    fn module(item: &str, count: u32) -> ItemStack {
        ItemStack { item: item.into(), count, components: String::new() }
    }

    #[test]
    fn a_module_goes_into_the_gear_after_ten_ticks_of_energy() {
        let mut slots = vec![Some(module("mekanism:module_attack_amplification_unit", 2)), Some(gear_with("{}")), None];
        let (mut energy, mut ticks) = (ENERGY_CAPACITY, 0u32);
        for _ in 0..(TICKS_REQUIRED - 1) {
            assert!(step_station(&mut slots, &mut energy, &mut ticks, true));
        }
        assert_eq!(ticks, TICKS_REQUIRED - 1);
        assert_eq!(slots[MODULE_SLOT].as_ref().map(|s| s.count), Some(2), "nothing yet");
        assert!(step_station(&mut slots, &mut energy, &mut ticks, true));
        assert_eq!(ticks, 0);
        // Both modules went in (the unit takes four), the slot is empty, the gear carries them in the format the tool reads.
        assert!(slots[MODULE_SLOT].is_none());
        let installed = ModuleContainer::decode_from_components(&slots[CONTAINER_SLOT].as_ref().unwrap().components);
        assert_eq!(installed.installed_count(ModuleType::AttackAmplificationUnit), 2);
        assert_eq!(ENERGY_CAPACITY - energy, ENERGY_USAGE * u64::from(TICKS_REQUIRED));
    }

    #[test]
    fn the_station_stops_when_it_cannot_install_or_cannot_function() {
        // A module that does not go on this gear, a gear that is full of it, no energy, redstone off: the count starts over.
        let mut slots = vec![Some(module("mekanism:module_jetpack_unit", 1)), Some(gear_with("{}")), None];
        let (mut energy, mut ticks) = (ENERGY_CAPACITY, 3u32);
        assert!(!step_station(&mut slots, &mut energy, &mut ticks, true), "a jetpack unit does not go on a tool");
        assert_eq!(ticks, 0);
        let mut slots = vec![Some(module("mekanism:module_teleportation_unit", 1)), Some(gear_with("{}")), None];
        let (mut energy, mut ticks) = (ENERGY_USAGE - 1, 2u32);
        assert!(!step_station(&mut slots, &mut energy, &mut ticks, true));
        assert_eq!(ticks, 0);
        let (mut energy, mut ticks) = (ENERGY_CAPACITY, 2u32);
        assert!(!step_station(&mut slots, &mut energy, &mut ticks, false));
        assert_eq!(ticks, 0);
    }

    #[test]
    fn the_list_shows_what_is_installed_by_name() {
        let mut c = ModuleContainer::new();
        c.add_module("mekanism:meka_tool", ModuleType::FortuneUnit, 2);
        c.add_module("mekanism:meka_tool", ModuleType::TeleportationUnit, 1);
        let stack = ItemStack { item: "mekanism:meka_tool".into(), count: 1, components: c.encode_to_components("{}") };
        let texts = list_texts(Some(&stack));
        assert_eq!(texts.len(), LIST_ROWS);
        assert_eq!(texts[0], "Fortune Unit x2");
        assert_eq!(texts[1], "Teleportation Unit x1");
        assert!(texts[2].is_empty());
        assert_eq!(module_title(ModuleType::ElectrolyticBreathingUnit), "Electrolytic Breathing Unit");
    }
}

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
        if let Some(pos) = components.find("\"modules\":{") {
            let rest = &components[pos + 10..];
            if let Some(end) = rest.find('}') {
                let inner = &rest[..end];
                for part in inner.split(',') {
                    let mut kv = part.split(':');
                    if let (Some(k), Some(v)) = (kv.next(), kv.next()) {
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

#[derive(Clone, Debug, PartialEq, Eq)]
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
}

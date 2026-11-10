//! Meka-Tool implementation for Horizon (Rust / WASM mod).
//!
//! Faithful port of Mekanism 1.21.1-10.7.19.85:
//! - Java item: `mekanism.common.item.gear.ItemMekaTool` (`ItemMekaTool.java`)
//! - Java config: `mekanism.common.config.GearConfig` (`GearConfig.java`)
//! - Java modules:
//!   - `ModuleEnergyUnit.java` (max 8)
//!   - `ModuleAttackAmplificationUnit.java` (max 4)
//!   - `ModuleExcavationEscalationUnit.java` (max 4)
//!   - `ModuleBlastingUnit.java` (max 4)
//!   - `ModuleVeinMiningUnit.java` (max 4)
//!   - `ModuleFarmingUnit.java` (max 4)
//!   - `ModuleShearingUnit.java` (max 1)
//!   - `ModuleTeleportationUnit.java` (max 1)
//!   - `SimpleEnchantmentAwareModule` Silk Touch (max 1)
//!   - `SimpleEnchantmentAwareModule` Fortune (max 3)
//! - Area mining helper: `IBlastingItem.java`
//! - Vein mining traversal: `ModuleVeinMiningUnit.java`
//! - Energy conversion: 1 FE = 2.5 Joules (`fe = J * 2 / 5`).

pub const MEKA_TOOL: &str = "mekanism:meka_tool";

/// Energy conversion helper: 1 FE = 2.5 Joules.
pub const fn fe(joules: u64) -> u64 {
    joules * 2 / 5
}

// -----------------------------------------------------------------------------
// Constants from Java (Mekanism 1.21.1, GearConfig.java)
// -----------------------------------------------------------------------------

/// Base energy capacity: 16_000_000 J = 6_400_000 FE (`GearConfig.java:223`).
pub const BASE_ENERGY_CAPACITY: u64 = fe(16_000_000);

/// Base charge rate: 100_000 J/t = 40_000 FE/t (`GearConfig.java:224`).
pub const BASE_CHARGE_RATE: u64 = fe(100_000);

/// Base attack damage: 4 (`GearConfig.java:226`).
pub const BASE_ATTACK_DAMAGE: f64 = 4.0;

/// Base attack speed modifier: -2.4 (`GearConfig.java:228`).
/// Combined with player base (4.0), yields 1.6 attacks/sec.
pub const BASE_ATTACK_SPEED: f64 = -2.4;

/// Base mining efficiency: 4.0 (`GearConfig.java:230`).
pub const BASE_EFFICIENCY: f32 = 4.0;

/// Extended vein mining enabled by default: true (`GearConfig.java:232`).
pub const DEFAULT_EXTENDED_MINING: bool = true;

/// Maximum reach for teleportation: 100 blocks (`GearConfig.java:234`).
pub const MAX_TELEPORT_REACH: u32 = 100;

/// Base block break energy usage: 10 J = 4 FE (`GearConfig.java:237`).
pub const ENERGY_USAGE_BASE: u64 = fe(10);

/// Block break energy usage with Silk Touch: 100 J = 40 FE (`GearConfig.java:238`).
pub const ENERGY_USAGE_SILK: u64 = fe(100);

/// Base weapon attack energy usage: 2_000 J = 800 FE (`GearConfig.java:239`).
pub const ENERGY_USAGE_WEAPON: u64 = fe(2_000);

/// Hoe use energy usage: 10 J = 4 FE (`GearConfig.java:240`).
pub const ENERGY_USAGE_HOE: u64 = fe(10);

/// Shovel use energy usage (flatten/douse): 10 J = 4 FE (`GearConfig.java:241`).
pub const ENERGY_USAGE_SHOVEL: u64 = fe(10);

/// Axe use energy usage (strip/scrape/wax-off): 10 J = 4 FE (`GearConfig.java:242`).
pub const ENERGY_USAGE_AXE: u64 = fe(10);

/// Shear entity energy usage: 10 J = 4 FE (`GearConfig.java:243`).
pub const ENERGY_USAGE_SHEAR_ENTITY: u64 = fe(10);

/// Shear block (trim) energy usage: 10 J = 4 FE (`GearConfig.java:244`).
pub const ENERGY_USAGE_SHEAR_TRIM: u64 = fe(10);

/// Base teleport energy usage per 10 blocks: 1_000 J = 400 FE (`GearConfig.java:245`).
pub const ENERGY_USAGE_TELEPORT: u64 = fe(1_000);

/// Maximum vein mining block count: 128 (`GearConfig.java:143`).
pub const MAX_VEIN_MINING_COUNT: usize = 128;

// -----------------------------------------------------------------------------
// Module Types and Modes
// -----------------------------------------------------------------------------

/// The 10 modules supported by Meka-Tool (`MekanismModules.java:60-125`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ModuleType {
    EnergyUnit,
    AttackAmplificationUnit,
    ExcavationEscalationUnit,
    BlastingUnit,
    VeinMiningUnit,
    FarmingUnit,
    ShearingUnit,
    TeleportationUnit,
    SilkTouchUnit,
    FortuneUnit,
}

impl ModuleType {
    /// Maximum installed count for this module type (`MekanismModules.java`).
    pub const fn max_stack(&self) -> u8 {
        match self {
            ModuleType::EnergyUnit => 8,
            ModuleType::AttackAmplificationUnit => 4,
            ModuleType::ExcavationEscalationUnit => 4,
            ModuleType::BlastingUnit => 4,
            ModuleType::VeinMiningUnit => 4,
            ModuleType::FarmingUnit => 4,
            ModuleType::ShearingUnit => 1,
            ModuleType::TeleportationUnit => 1,
            ModuleType::SilkTouchUnit => 1,
            ModuleType::FortuneUnit => 3,
        }
    }

    /// Resource location registry name (`MekanismModules.java`).
    pub const fn registry_name(&self) -> &'static str {
        match self {
            ModuleType::EnergyUnit => "mekanism:energy_unit",
            ModuleType::AttackAmplificationUnit => "mekanism:attack_amplification_unit",
            ModuleType::ExcavationEscalationUnit => "mekanism:excavation_escalation_unit",
            ModuleType::BlastingUnit => "mekanism:blasting_unit",
            ModuleType::VeinMiningUnit => "mekanism:vein_mining_unit",
            ModuleType::FarmingUnit => "mekanism:farming_unit",
            ModuleType::ShearingUnit => "mekanism:shearing_unit",
            ModuleType::TeleportationUnit => "mekanism:teleportation_unit",
            ModuleType::SilkTouchUnit => "mekanism:silk_touch_unit",
            ModuleType::FortuneUnit => "mekanism:fortune_unit",
        }
    }

    pub fn from_registry_name(name: &str) -> Option<Self> {
        match name {
            "mekanism:energy_unit" | "energy_unit" => Some(ModuleType::EnergyUnit),
            "mekanism:attack_amplification_unit" | "attack_amplification_unit" => Some(ModuleType::AttackAmplificationUnit),
            "mekanism:excavation_escalation_unit" | "excavation_escalation_unit" => Some(ModuleType::ExcavationEscalationUnit),
            "mekanism:blasting_unit" | "blasting_unit" => Some(ModuleType::BlastingUnit),
            "mekanism:vein_mining_unit" | "vein_mining_unit" => Some(ModuleType::VeinMiningUnit),
            "mekanism:farming_unit" | "farming_unit" => Some(ModuleType::FarmingUnit),
            "mekanism:shearing_unit" | "shearing_unit" => Some(ModuleType::ShearingUnit),
            "mekanism:teleportation_unit" | "teleportation_unit" => Some(ModuleType::TeleportationUnit),
            "mekanism:silk_touch_unit" | "silk_touch_unit" => Some(ModuleType::SilkTouchUnit),
            "mekanism:fortune_unit" | "fortune_unit" => Some(ModuleType::FortuneUnit),
            _ => None,
        }
    }
}

/// Attack damage modes (`ModuleAttackAmplificationUnit.java:49-54`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttackDamage {
    Off = 0,
    Low = 4,
    Med = 8,
    High = 16,
    Extreme = 24,
    Max = 32,
}

impl AttackDamage {
    pub const fn damage(&self) -> u32 {
        *self as u32
    }

    pub const fn from_index(index: usize) -> Self {
        match index {
            0 => AttackDamage::Off,
            1 => AttackDamage::Low,
            2 => AttackDamage::Med,
            3 => AttackDamage::High,
            4 => AttackDamage::Extreme,
            _ => AttackDamage::Max,
        }
    }

    pub const fn max_index_for_installed(installed: u8) -> usize {
        // installed + 2 values accessible (ModuleAttackAmplificationUnit.java:85)
        let val = installed as usize + 2;
        if val < 5 { val } else { 5 }
    }
}

/// Excavation speed modes (`ModuleExcavationEscalationUnit.java:124-129`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExcavationMode {
    Off = 0,
    Slow = 4,
    Normal = 16,
    Fast = 32,
    SuperFast = 64,
    Extreme = 128,
}

impl ExcavationMode {
    pub const fn efficiency(&self) -> u32 {
        *self as u32
    }

    pub const fn from_index(index: usize) -> Self {
        match index {
            0 => ExcavationMode::Off,
            1 => ExcavationMode::Slow,
            2 => ExcavationMode::Normal,
            3 => ExcavationMode::Fast,
            4 => ExcavationMode::SuperFast,
            _ => ExcavationMode::Extreme,
        }
    }

    pub const fn max_index_for_installed(installed: u8) -> usize {
        // installed + 2 values accessible (ModuleExcavationEscalationUnit.java:51)
        let val = installed as usize + 2;
        if val < 5 { val } else { 5 }
    }
}

/// Blasting radius modes (`ModuleBlastingUnit.java:107-111`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlastRadius {
    Off = 0,
    Low = 1,     // 3x3
    Med = 2,     // 5x5
    High = 3,    // 7x7
    Extreme = 4, // 9x9
}

impl BlastRadius {
    pub const fn radius(&self) -> u8 {
        *self as u8
    }

    pub const fn diameter(&self) -> u8 {
        2 * self.radius() + 1
    }

    pub const fn from_index(index: usize) -> Self {
        match index {
            0 => BlastRadius::Off,
            1 => BlastRadius::Low,
            2 => BlastRadius::Med,
            3 => BlastRadius::High,
            _ => BlastRadius::Extreme,
        }
    }

    pub const fn max_index_for_installed(installed: u8) -> usize {
        // installed + 1 values accessible (ModuleBlastingUnit.java:49)
        let val = installed as usize + 1;
        if val < 4 { val } else { 4 }
    }
}

/// Vein mining excavation range modes (`ModuleVeinMiningUnit.java:191-195`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExcavationRange {
    Off = 0,
    Low = 2,
    Med = 4,
    High = 6,
    Extreme = 8,
}

impl ExcavationRange {
    pub const fn range(&self) -> u32 {
        *self as u32
    }

    pub const fn from_index(index: usize) -> Self {
        match index {
            0 => ExcavationRange::Off,
            1 => ExcavationRange::Low,
            2 => ExcavationRange::Med,
            3 => ExcavationRange::High,
            _ => ExcavationRange::Extreme,
        }
    }

    pub const fn max_index_for_installed(installed: u8) -> usize {
        let val = installed as usize + 1;
        if val < 4 { val } else { 4 }
    }
}

/// Farming radius modes (`ModuleFarmingUnit.java:119-123`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FarmingRadius {
    Off = 0,
    Low = 1,   // 1x1
    Med = 3,   // 3x3
    High = 5,  // 5x5
    Ultra = 7, // 7x7
}

impl FarmingRadius {
    pub const fn diameter(&self) -> u8 {
        *self as u8
    }

    pub const fn radius(&self) -> u8 {
        if self.diameter() == 0 {
            0
        } else {
            (self.diameter() - 1) / 2
        }
    }

    pub const fn from_index(index: usize) -> Self {
        match index {
            0 => FarmingRadius::Off,
            1 => FarmingRadius::Low,
            2 => FarmingRadius::Med,
            3 => FarmingRadius::High,
            _ => FarmingRadius::Ultra,
        }
    }

    pub const fn max_index_for_installed(installed: u8) -> usize {
        let val = installed as usize + 1;
        if val < 4 { val } else { 4 }
    }
}

/// Facing direction for raycast hits (`Direction` in Java).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitFace {
    Down = 0,
    Up = 1,
    North = 2,
    South = 3,
    West = 4,
    East = 5,
}

// -----------------------------------------------------------------------------
// Module Configuration and Container
// -----------------------------------------------------------------------------

/// State of an installed module instance on a Meka-Tool.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstalledModule {
    pub module_type: ModuleType,
    pub installed_count: u8,
    pub enabled: bool,
    // Module-specific configs:
    pub attack_damage: AttackDamage,
    pub excavation_mode: ExcavationMode,
    pub blast_radius: BlastRadius,
    pub vein_extended: bool,
    pub vein_range: ExcavationRange,
    pub farming_radius: FarmingRadius,
    pub teleport_requires_target: bool,
}

impl InstalledModule {
    pub fn new(module_type: ModuleType, installed_count: u8) -> Self {
        let count = installed_count.min(module_type.max_stack());
        Self {
            module_type,
            installed_count: count,
            enabled: true,
            attack_damage: AttackDamage::Med,
            excavation_mode: ExcavationMode::Normal,
            blast_radius: BlastRadius::Low,
            vein_extended: false,
            vein_range: ExcavationRange::Low,
            farming_radius: FarmingRadius::Low,
            teleport_requires_target: true,
        }
    }
}

/// In-memory representation of a Meka-Tool stack with its energy and installed modules.
#[derive(Clone, Debug, PartialEq)]
pub struct MekaToolData {
    pub energy: u64,
    pub modules: Vec<InstalledModule>,
}

impl Default for MekaToolData {
    fn default() -> Self {
        Self {
            energy: 0,
            modules: Vec::new(),
        }
    }
}

impl MekaToolData {
    pub fn new(energy: u64) -> Self {
        Self {
            energy,
            modules: Vec::new(),
        }
    }

    /// Number of installed Energy Units (max 8).
    pub fn energy_units_count(&self) -> u8 {
        self.modules
            .iter()
            .find(|m| m.module_type == ModuleType::EnergyUnit)
            .map(|m| m.installed_count)
            .unwrap_or(0)
    }

    /// Maximum energy capacity in FE: base * 2^(energy_units) (`ModuleEnergyUnit.java:33`).
    pub fn max_energy(&self) -> u64 {
        let count = self.energy_units_count();
        BASE_ENERGY_CAPACITY.saturating_mul(1u64 << count)
    }

    /// Maximum charge rate in FE/t: base * 2^(energy_units) (`ModuleEnergyUnit.java:25, 33`).
    pub fn charge_rate(&self) -> u64 {
        let count = self.energy_units_count();
        BASE_CHARGE_RATE.saturating_mul(1u64 << count)
    }

    /// Checks if a module type is present and enabled.
    pub fn is_module_enabled(&self, module_type: ModuleType) -> bool {
        self.modules.iter().any(|m| m.module_type == module_type && m.enabled)
    }

    /// Gets a reference to an installed module.
    pub fn get_module(&self, module_type: ModuleType) -> Option<&InstalledModule> {
        self.modules.iter().find(|m| m.module_type == module_type)
    }

    /// Gets a mutable reference to an installed module.
    pub fn get_module_mut(&mut self, module_type: ModuleType) -> Option<&mut InstalledModule> {
        self.modules.iter_mut().find(|m| m.module_type == module_type)
    }

    /// Installs a module onto the tool. Returns the updated installed count.
    pub fn install_module(&mut self, module_type: ModuleType) -> Result<u8, &'static str> {
        let max = module_type.max_stack();
        if let Some(m) = self.get_module_mut(module_type) {
            if m.installed_count >= max {
                return Err("Module reached max stack limit");
            }
            m.installed_count += 1;
            Ok(m.installed_count)
        } else {
            self.modules.push(InstalledModule::new(module_type, 1));
            Ok(1)
        }
    }

    /// Removes one unit of a module from the tool. Returns the remaining count.
    pub fn remove_module(&mut self, module_type: ModuleType) -> Result<u8, &'static str> {
        if let Some(pos) = self.modules.iter().position(|m| m.module_type == module_type) {
            if self.modules[pos].installed_count > 1 {
                self.modules[pos].installed_count -= 1;
                Ok(self.modules[pos].installed_count)
            } else {
                self.modules.remove(pos);
                // When removing energy unit, clamp energy to new capacity (ModuleEnergyUnit.java:44)
                if module_type == ModuleType::EnergyUnit {
                    self.energy = self.energy.min(self.max_energy());
                }
                Ok(0)
            }
        } else {
            Err("Module not installed")
        }
    }

    /// Whether Silk Touch unit is installed and active.
    pub fn has_silk_touch(&self) -> bool {
        self.is_module_enabled(ModuleType::SilkTouchUnit)
    }

    /// Fortune enchantment level (0 to 3).
    pub fn fortune_level(&self) -> u8 {
        self.get_module(ModuleType::FortuneUnit)
            .filter(|m| m.enabled)
            .map(|m| m.installed_count.min(3))
            .unwrap_or(0)
    }

    // -------------------------------------------------------------------------
    // Weapon & Attack Mechanics (ItemMekaTool.java:283-300, 346-369)
    // -------------------------------------------------------------------------

    /// Calculates attack damage and weapon energy cost.
    /// Returns `(effective_damage, energy_cost_fe)`.
    pub fn calculate_attack(&self) -> (f64, u64) {
        let mut damage = BASE_ATTACK_DAMAGE;
        let mut energy_cost = 0u64;

        if let Some(m) = self.get_module(ModuleType::AttackAmplificationUnit) {
            if m.enabled {
                let unit_damage = m.attack_damage.damage();
                if unit_damage > 0 {
                    // Java: clampToLong(mekaToolEnergyUsageWeapon * (unitDamage / 4D))
                    // mekaToolEnergyUsageWeapon = 2_000 J = 800 FE
                    // 800 * unitDamage / 4 = 200 * unitDamage FE
                    energy_cost = (ENERGY_USAGE_WEAPON as f64 * (unit_damage as f64 / 4.0)).round() as u64;

                    if self.energy < energy_cost && energy_cost > 0 {
                        // Partial power damage scaling (ItemMekaTool.java:359)
                        let ratio = self.energy as f64 / energy_cost as f64;
                        damage += unit_damage as f64 * ratio;
                    } else {
                        damage += unit_damage as f64;
                    }
                }
            }
        }

        (damage, energy_cost)
    }

    /// Attack speed modifier: -2.4 (`ItemMekaTool.java:349`).
    pub fn attack_speed(&self) -> f64 {
        BASE_ATTACK_SPEED
    }

    // -------------------------------------------------------------------------
    // Digging & Mining Speed / Energy Mechanics (ItemMekaTool.java:147-156, 234-248, 326-343)
    // -------------------------------------------------------------------------

    /// Efficiency factor of the Meka-Tool.
    pub fn efficiency(&self) -> f32 {
        if let Some(m) = self.get_module(ModuleType::ExcavationEscalationUnit) {
            if m.enabled {
                return m.excavation_mode.efficiency() as f32;
            }
        }
        BASE_EFFICIENCY
    }

    /// Base destroy energy in FE before block hardness scaling.
    /// Java: `getDestroyEnergy(boolean silk)` (ItemMekaTool.java:326).
    pub fn base_destroy_energy(&self) -> u64 {
        if self.has_silk_touch() {
            ENERGY_USAGE_SILK
        } else {
            ENERGY_USAGE_BASE
        }
    }

    /// Modified destroy energy including efficiency multiplier.
    /// Java: `clampToLong(destroyEnergy * efficiency)` (ItemMekaTool.java:155, 342).
    pub fn mod_destroy_energy(&self) -> u64 {
        let base = self.base_destroy_energy();
        let eff = self.efficiency();
        (base as f32 * eff).round().max(1.0) as u64
    }

    /// Total destroy energy required to break a block with given hardness.
    /// Java: `hardness == 0 ? Math.max(baseDestroyEnergy / 2, 1) : baseDestroyEnergy` (ItemMekaTool.java:335).
    pub fn get_destroy_energy(&self, hardness: f32) -> u64 {
        let mod_energy = self.mod_destroy_energy();
        if hardness == 0.0 {
            (mod_energy / 2).max(1)
        } else {
            mod_energy
        }
    }

    /// Mining break speed against block state hardness (`ItemMekaTool.java:234-248`).
    pub fn destroy_speed(&self, hardness: f32) -> f32 {
        let energy_required = self.get_destroy_energy(hardness);
        let eff = self.efficiency();

        if self.energy < energy_required && energy_required > 0 {
            // Speed scaled down by energy availability (ItemMekaTool.java:244)
            BASE_EFFICIENCY * (self.energy as f32 / energy_required as f32)
        } else {
            eff
        }
    }
}

// -----------------------------------------------------------------------------
// Mining Algorithms: Blasting and Vein Mining
// -----------------------------------------------------------------------------

/// Finds positions affected by Blasting Unit (`IBlastingItem.java:26-60`).
///
/// Orientations:
/// - UP / DOWN: `(-radius..=radius, 0..=0, -radius..=radius)`
/// - EAST / WEST: `(0..=0, -1..=(2*radius - 1), -radius..=radius)`
/// - NORTH / SOUTH: `(-radius..=radius, -1..=(2*radius - 1), 0..=0)`
pub fn find_blasting_positions(
    target_pos: [i32; 3],
    hit_face: HitFace,
    radius: u8,
    can_blast: impl Fn([i32; 3]) -> bool,
) -> Vec<[i32; 3]> {
    if radius == 0 {
        return Vec::new();
    }
    let r = radius as i32;
    let mut positions = Vec::new();

    match hit_face {
        HitFace::Up | HitFace::Down => {
            for dx in -r..=r {
                for dz in -r..=r {
                    let pos = [target_pos[0] + dx, target_pos[1], target_pos[2] + dz];
                    if can_blast(pos) {
                        positions.push(pos);
                    }
                }
            }
        }
        HitFace::East | HitFace::West => {
            for dy in -1..=(2 * r - 1) {
                for dz in -r..=r {
                    let pos = [target_pos[0], target_pos[1] + dy, target_pos[2] + dz];
                    if can_blast(pos) {
                        positions.push(pos);
                    }
                }
            }
        }
        HitFace::North | HitFace::South => {
            for dx in -r..=r {
                for dy in -1..=(2 * r - 1) {
                    let pos = [target_pos[0] + dx, target_pos[1] + dy, target_pos[2]];
                    if can_blast(pos) {
                        positions.push(pos);
                    }
                }
            }
        }
    }

    positions
}

/// Breadth-first search for Vein Mining traversal (`ModuleVeinMiningUnit.java:136-177`).
///
/// Returns pairs of `(block_pos, traversal_distance)`.
/// - `distance == 0` for initial target / blasted blocks.
/// - `distance >= 1` for connected blocks.
pub fn find_vein_positions(
    initial_blocks: &[[i32; 3]],
    extended_range: u32,
    is_ore: bool,
    max_count: usize,
    matches_target_block: impl Fn([i32; 3]) -> bool,
) -> Vec<([i32; 3], u32)> {
    use std::collections::{HashSet, VecDeque};

    let mut found = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    for &pos in initial_blocks {
        visited.insert(pos);
        found.push((pos, 0u32));
        queue.push_back((pos, 0u32));
    }

    let limit = max_count.min(MAX_VEIN_MINING_COUNT);

    while let Some((curr, dist)) = queue.pop_front() {
        if found.len() >= limit {
            break;
        }

        // If block is ore or distance is within extendedRange (ModuleVeinMiningUnit.java:158)
        if is_ore || extended_range > dist {
            for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        if dx == 0 && dy == 0 && dz == 0 {
                            continue;
                        }
                        let next = [curr[0] + dx, curr[1] + dy, curr[2] + dz];
                        if !visited.contains(&next) && matches_target_block(next) {
                            visited.insert(next);
                            let next_dist = dist + 1;
                            found.push((next, next_dist));
                            queue.push_back((next, next_dist));
                            if found.len() >= limit {
                                return found;
                            }
                        }
                    }
                }
            }
        }
    }

    found
}

/// Calculates vein mining energy for a connected block at a given BFS distance.
/// Java: `ItemMekaTool.java:272-275`:
/// `multiplier = 0.5 * Math.pow(distance, isOre ? 1.5 : 2)`
/// `energy = ceil(destroyEnergy * multiplier)`
pub fn calculate_vein_energy(base_energy: u64, hardness: f32, distance: u32, is_ore: bool) -> u64 {
    if distance == 0 {
        if hardness == 0.0 {
            (base_energy / 2).max(1)
        } else {
            base_energy
        }
    } else {
        let exponent = if is_ore { 1.5 } else { 2.0 };
        let multiplier = 0.5 * (distance as f64).powf(exponent);
        let block_base = if hardness == 0.0 {
            (base_energy / 2).max(1)
        } else {
            base_energy
        };
        ((block_base as f64) * multiplier).ceil() as u64
    }
}

// -----------------------------------------------------------------------------
// Farming Unit AOE Positions and Action Costs
// -----------------------------------------------------------------------------

/// Finds positions for Farming Unit AOE tools (`ModuleFarmingUnit.java:220`).
pub fn find_farming_positions(target_pos: [i32; 3], hit_face: HitFace, radius: u8) -> Vec<[i32; 3]> {
    if radius == 0 {
        return vec![target_pos];
    }
    let r = radius as i32;
    let mut positions = Vec::new();

    match hit_face {
        HitFace::Up | HitFace::Down => {
            for dx in -r..=r {
                for dz in -r..=r {
                    positions.push([target_pos[0] + dx, target_pos[1], target_pos[2] + dz]);
                }
            }
        }
        HitFace::North | HitFace::South => {
            for dx in -r..=r {
                for dy in -r..=r {
                    positions.push([target_pos[0] + dx, target_pos[1] + dy, target_pos[2]]);
                }
            }
        }
        HitFace::East | HitFace::West => {
            for dz in -r..=r {
                for dy in -r..=r {
                    positions.push([target_pos[0], target_pos[1] + dy, target_pos[2] + dz]);
                }
            }
        }
    }

    positions
}

// -----------------------------------------------------------------------------
// Teleportation Mechanics (ItemMekaTool.java:373-417)
// -----------------------------------------------------------------------------

/// Validates teleportation requirements and calculates distance and energy.
/// Returns `Ok((target_coords, energy_needed))` or `Err(reason)`.
pub fn check_teleport(
    player_pos: [f64; 3],
    target_block: [i32; 3],
    head_free: bool,
    above_head_free: bool,
    requires_target: bool,
    hit_block: bool,
    energy_available: u64,
) -> Result<([f64; 3], u64), &'static str> {
    if requires_target && !hit_block {
        return Err("Requires block target");
    }

    // Target coordinates: pos.getX() + 0.5, pos.getY() + 1.5, pos.getZ() + 0.5 (ItemMekaTool.java:393-395)
    let target = [
        target_block[0] as f64 + 0.5,
        target_block[1] as f64 + 1.5,
        target_block[2] as f64 + 0.5,
    ];

    // Destination block clearance (ItemMekaTool.java:383)
    if !head_free || !above_head_free {
        return Err("Destination blocked");
    }

    let dx = player_pos[0] - target_block[0] as f64;
    let dy = player_pos[1] - target_block[1] as f64;
    let dz = player_pos[2] - target_block[2] as f64;
    let dist_sqr = dx * dx + dy * dy + dz * dz;

    if dist_sqr < 5.0 {
        return Err("Too close to teleport (min distance sqr 5.0)");
    }

    let distance = dist_sqr.sqrt();
    if distance > MAX_TELEPORT_REACH as f64 {
        return Err("Exceeds max reach (100 blocks)");
    }

    // Java: MathUtils.ceilToLong(mekaToolEnergyUsageTeleport * (distance / 10D))
    // mekaToolEnergyUsageTeleport = 1_000 J = 400 FE
    let energy_needed = (ENERGY_USAGE_TELEPORT as f64 * (distance / 10.0)).ceil() as u64;

    if energy_available < energy_needed {
        return Err("Insufficient energy for teleport");
    }

    Ok((target, energy_needed))
}

// -----------------------------------------------------------------------------
// Data Components Serialization (SNBT / JSON) (FORK-23)
// -----------------------------------------------------------------------------

/// Serializes MekaToolData into Horizon data components JSON string.
pub fn serialize_to_components(data: &MekaToolData) -> String {
    let mut modules_json = Vec::new();
    for m in &data.modules {
        let entry = format!(
            r#"{{"type":"{}","installed":{},"enabled":{},"attack_damage":{},"excavation_mode":{},"blast_radius":{},"vein_extended":{},"vein_range":{},"farming_radius":{},"teleport_requires_target":{}}}"#,
            m.module_type.registry_name(),
            m.installed_count,
            m.enabled,
            m.attack_damage.damage(),
            m.excavation_mode.efficiency(),
            m.blast_radius.radius(),
            m.vein_extended,
            m.vein_range.range(),
            m.farming_radius.diameter(),
            m.teleport_requires_target
        );
        modules_json.push(entry);
    }

    format!(
        r#"{{"mekanism:energy":{{"energy_containers":[{}]}},"mekanism:modules":[{}]}}"#,
        data.energy,
        modules_json.join(",")
    )
}

/// Parses MekaToolData from Horizon data components JSON string.
pub fn parse_from_components(components: &str) -> Option<MekaToolData> {
    if components.trim().is_empty() {
        return Some(MekaToolData::default());
    }

    let mut energy = 0u64;
    // Extract energy from "energy_containers":[<energy>]
    if let Some(pos) = components.find("\"energy_containers\":[") {
        let rest = &components[pos + 21..];
        if let Some(end) = rest.find(']') {
            let num_str = rest[..end].trim();
            if let Ok(val) = num_str.parse::<u64>() {
                energy = val;
            }
        }
    }

    let mut data = MekaToolData::new(energy);

    // Extract modules list
    if let Some(mod_pos) = components.find("\"mekanism:modules\":[") {
        let rest = &components[mod_pos + 20..];
        if let Some(end) = rest.find(']') {
            let list_str = &rest[..end];
            for block in list_str.split("},{") {
                for m_type in [
                    ModuleType::EnergyUnit,
                    ModuleType::AttackAmplificationUnit,
                    ModuleType::ExcavationEscalationUnit,
                    ModuleType::BlastingUnit,
                    ModuleType::VeinMiningUnit,
                    ModuleType::FarmingUnit,
                    ModuleType::ShearingUnit,
                    ModuleType::TeleportationUnit,
                    ModuleType::SilkTouchUnit,
                    ModuleType::FortuneUnit,
                ] {
                    if block.contains(m_type.registry_name()) {
                        let mut installed = 1u8;
                        if let Some(cnt_pos) = block.find("\"installed\":") {
                            let cnt_rest = &block[cnt_pos + 12..];
                            let num = cnt_rest.split(|c: char| !c.is_ascii_digit()).next().unwrap_or("");
                            if let Ok(c) = num.parse::<u8>() {
                                installed = c;
                            }
                        }
                        let _ = data.install_module(m_type);
                        if let Some(mod_ref) = data.get_module_mut(m_type) {
                            mod_ref.installed_count = installed;
                        }
                    }
                }
            }
        }
    }

    Some(data)
}

// -----------------------------------------------------------------------------
// Host API Hooks / Stubs (Marked with FORK-n)
// -----------------------------------------------------------------------------

/// Reads Meka-Tool data from an ItemStack via data components.
// FORK-23
pub fn host_read_meka_tool(components_str: &str) -> MekaToolData {
    parse_from_components(components_str).unwrap_or_default()
}

/// Writes Meka-Tool data back to an ItemStack.
// FORK-23
pub fn host_write_meka_tool(data: &MekaToolData) -> String {
    serialize_to_components(data)
}

/// Triggers radial menu on the client.
// FORK-27
pub fn host_open_radial_menu(_player_id: &str, _item_id: &str) -> bool {
    // Requires FORK-27 radial menu API in horizon-mod-sdk
    false
}

/// Performs multi-block area mining in world simulation.
// FORK-31
pub fn host_execute_area_mining(
    _dimension: &str,
    _player_id: &str,
    _positions: &[[i32; 3]],
    _total_energy: u64,
) -> usize {
    // Requires FORK-31 multi-block break API
    0
}

/// Teleports player and sends portal FX packet.
// FORK-32
pub fn host_execute_teleport(
    _dimension: &str,
    _player_id: &str,
    _target: [f64; 3],
    _energy_cost: u64,
) -> bool {
    // Requires FORK-32 teleport & particle FX API
    false
}

// -----------------------------------------------------------------------------
// Unit Tests (Comparing with Java 1.21.1 source of truth)
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_match_java_gear_config() {
        // GearConfig.java:223-245
        assert_eq!(BASE_ENERGY_CAPACITY, 6_400_000, "Base capacity 16MJ = 6.4M FE (GearConfig.java:223)");
        assert_eq!(BASE_CHARGE_RATE, 40_000, "Base charge rate 100kJ/t = 40k FE/t (GearConfig.java:224)");
        assert_eq!(BASE_ATTACK_DAMAGE, 4.0, "Base damage 4 (GearConfig.java:226)");
        assert_eq!(BASE_ATTACK_SPEED, -2.4, "Attack speed -2.4 (GearConfig.java:228)");
        assert_eq!(BASE_EFFICIENCY, 4.0, "Base efficiency 4.0 (GearConfig.java:230)");
        assert_eq!(DEFAULT_EXTENDED_MINING, true, "Extended mining true (GearConfig.java:232)");
        assert_eq!(MAX_TELEPORT_REACH, 100, "Max teleport reach 100 (GearConfig.java:234)");
        assert_eq!(ENERGY_USAGE_BASE, 4, "Base break 10 J = 4 FE (GearConfig.java:237)");
        assert_eq!(ENERGY_USAGE_SILK, 40, "Silk break 100 J = 40 FE (GearConfig.java:238)");
        assert_eq!(ENERGY_USAGE_WEAPON, 800, "Weapon 2_000 J = 800 FE (GearConfig.java:239)");
        assert_eq!(ENERGY_USAGE_HOE, 4, "Hoe 10 J = 4 FE (GearConfig.java:240)");
        assert_eq!(ENERGY_USAGE_SHOVEL, 4, "Shovel 10 J = 4 FE (GearConfig.java:241)");
        assert_eq!(ENERGY_USAGE_AXE, 4, "Axe 10 J = 4 FE (GearConfig.java:242)");
        assert_eq!(ENERGY_USAGE_SHEAR_ENTITY, 4, "Shear entity 10 J = 4 FE (GearConfig.java:243)");
        assert_eq!(ENERGY_USAGE_SHEAR_TRIM, 4, "Shear trim 10 J = 4 FE (GearConfig.java:244)");
        assert_eq!(ENERGY_USAGE_TELEPORT, 400, "Teleport base 1_000 J = 400 FE (GearConfig.java:245)");
        assert_eq!(MAX_VEIN_MINING_COUNT, 128, "Max vein count 128 (GearConfig.java:143)");
    }

    #[test]
    fn energy_capacity_scales_exponentially_with_energy_units() {
        // ModuleEnergyUnit.java:33: base * 2^installedCount
        let mut tool = MekaToolData::default();
        assert_eq!(tool.max_energy(), 6_400_000);
        assert_eq!(tool.charge_rate(), 40_000);

        for count in 1..=8 {
            assert!(tool.install_module(ModuleType::EnergyUnit).is_ok());
            let expected_cap = 6_400_000u64 * (1u64 << count);
            let expected_rate = 40_000u64 * (1u64 << count);
            assert_eq!(tool.max_energy(), expected_cap);
            assert_eq!(tool.charge_rate(), expected_rate);
        }

        // 8 Energy Units = 6.4M * 256 = 1_638_400_000 FE (4_096_000_000 J)
        assert_eq!(tool.max_energy(), 1_638_400_000);
        assert_eq!(tool.charge_rate(), 10_240_000);

        // Attempting to install 9th fails (max stack is 8)
        assert!(tool.install_module(ModuleType::EnergyUnit).is_err());
    }

    #[test]
    fn attack_amplification_damage_and_energy_costs() {
        // ModuleAttackAmplificationUnit.java:49-54 & ItemMekaTool.java:346-369
        let mut tool = MekaToolData::new(100_000);
        assert!(tool.install_module(ModuleType::AttackAmplificationUnit).is_ok());

        // Default mode is Med (damage 8)
        let (dmg, cost) = tool.calculate_attack();
        assert_eq!(dmg, 4.0 + 8.0, "Base 4 + 8 = 12.0");
        assert_eq!(cost, 1_600, "800 * 8 / 4 = 1600 FE");

        // Test all modes
        tool.get_module_mut(ModuleType::AttackAmplificationUnit).unwrap().attack_damage = AttackDamage::Off;
        assert_eq!(tool.calculate_attack(), (4.0, 0));

        tool.get_module_mut(ModuleType::AttackAmplificationUnit).unwrap().attack_damage = AttackDamage::Low;
        assert_eq!(tool.calculate_attack(), (8.0, 800));

        tool.get_module_mut(ModuleType::AttackAmplificationUnit).unwrap().attack_damage = AttackDamage::Med;
        assert_eq!(tool.calculate_attack(), (12.0, 1_600));

        tool.get_module_mut(ModuleType::AttackAmplificationUnit).unwrap().attack_damage = AttackDamage::High;
        assert_eq!(tool.calculate_attack(), (20.0, 3_200));

        tool.get_module_mut(ModuleType::AttackAmplificationUnit).unwrap().attack_damage = AttackDamage::Extreme;
        assert_eq!(tool.calculate_attack(), (28.0, 4_800));

        tool.get_module_mut(ModuleType::AttackAmplificationUnit).unwrap().attack_damage = AttackDamage::Max;
        assert_eq!(tool.calculate_attack(), (36.0, 6_400));

        // Partial power scaling: 800 FE available for 1600 FE cost (ratio 0.5)
        tool.energy = 800;
        tool.get_module_mut(ModuleType::AttackAmplificationUnit).unwrap().attack_damage = AttackDamage::Med;
        let (dmg_partial, cost_partial) = tool.calculate_attack();
        assert_eq!(cost_partial, 1_600);
        assert_eq!(dmg_partial, 4.0 + 8.0 * 0.5, "4 + 4 = 8.0 with half power");
    }

    #[test]
    fn excavation_escalation_modes_and_energy_calculations() {
        // ModuleExcavationEscalationUnit.java:124-129 & ItemMekaTool.java:234-248
        let mut tool = MekaToolData::new(100_000);
        assert_eq!(tool.efficiency(), 4.0, "Base efficiency without module");
        assert_eq!(tool.get_destroy_energy(1.5), 16, "Base break: 4 FE * 4 eff = 16 FE");
        assert_eq!(tool.get_destroy_energy(0.0), 8, "Zero hardness halved: 16 / 2 = 8 FE");

        assert!(tool.install_module(ModuleType::ExcavationEscalationUnit).is_ok());
        // Default mode is Normal (efficiency 16)
        assert_eq!(tool.efficiency(), 16.0);
        assert_eq!(tool.get_destroy_energy(1.5), 64, "4 FE * 16 eff = 64 FE");

        // With Silk Touch: base 40 FE * 16 eff = 640 FE
        assert!(tool.install_module(ModuleType::SilkTouchUnit).is_ok());
        assert_eq!(tool.get_destroy_energy(1.5), 640);
        assert_eq!(tool.get_destroy_energy(0.0), 320);

        // Extreme speed mode: efficiency 128
        let m = tool.get_module_mut(ModuleType::ExcavationEscalationUnit).unwrap();
        m.excavation_mode = ExcavationMode::Extreme;
        assert_eq!(tool.efficiency(), 128.0);
        assert_eq!(tool.get_destroy_energy(1.5), 5_120, "40 FE * 128 = 5120 FE");

        // Destroy speed reduction when energy depleted
        tool.energy = 2_560; // half of required 5120
        let speed = tool.destroy_speed(1.5);
        assert_eq!(speed, 4.0 * 0.5, "Speed scaled down to baseEff * 0.5 = 2.0");
    }

    #[test]
    fn blasting_positions_calculation() {
        // IBlastingItem.java:26-50
        let center = [10, 60, 20];

        // Radius 1 = 3x3 on Up face
        let up_blocks = find_blasting_positions(center, HitFace::Up, 1, |_| true);
        assert_eq!(up_blocks.len(), 9, "3x3 = 9 blocks on horizontal plane");
        assert!(up_blocks.contains(&[10, 60, 20]));
        assert!(up_blocks.contains(&[9, 60, 19]));
        assert!(up_blocks.contains(&[11, 60, 21]));

        // Radius 2 = 5x5 on North face (dx in -2..=2, dy in -1..=3, dz = 0)
        let north_blocks = find_blasting_positions(center, HitFace::North, 2, |_| true);
        assert_eq!(north_blocks.len(), 25, "5x5 = 25 blocks on vertical plane");
        assert!(north_blocks.contains(&[8, 59, 20]));  // -2, -1
        assert!(north_blocks.contains(&[12, 63, 20])); // +2, +3

        // Radius 4 = 9x9 on East face (dx = 0, dy in -1..=7, dz in -4..=4)
        let east_blocks = find_blasting_positions(center, HitFace::East, 4, |_| true);
        assert_eq!(east_blocks.len(), 81, "9x9 = 81 blocks");
    }

    #[test]
    fn vein_mining_traversal_and_energy_scaling() {
        // ModuleVeinMiningUnit.java:136-177 & ItemMekaTool.java:272-275
        let initial = [[0, 0, 0]];

        // Ore traversal: follows connected line of 10 blocks
        let found_ores = find_vein_positions(&initial, 0, true, 50, |p| p[0] >= 0 && p[0] <= 9 && p[1] == 0 && p[2] == 0);
        assert_eq!(found_ores.len(), 10);
        assert_eq!(found_ores[0], ([0, 0, 0], 0));
        assert_eq!(found_ores[9], ([9, 0, 0], 9));

        // Energy for ore at distance 4: 0.5 * 4^1.5 = 0.5 * 8 = 4.0 multiplier
        let base_cost = 4u64;
        let ore_energy = calculate_vein_energy(base_cost, 1.0, 4, true);
        assert_eq!(ore_energy, 16, "4 * (0.5 * 4^1.5) = 4 * 4 = 16 FE");

        // Energy for non-ore at distance 4: 0.5 * 4^2 = 0.5 * 16 = 8.0 multiplier
        let non_ore_energy = calculate_vein_energy(base_cost, 1.0, 4, false);
        assert_eq!(non_ore_energy, 32, "4 * (0.5 * 4^2) = 4 * 8 = 32 FE");

        // Non-ore traversal respecting extended range limit
        let found_non_ore = find_vein_positions(&initial, 3, false, 50, |p| p[0] >= 0 && p[0] <= 10 && p[1] == 0 && p[2] == 0);
        // extended_range = 3 allows expansion from dist 0, 1, 2 up to dist 3
        assert_eq!(found_non_ore.len(), 4, "Covers dist 0, 1, 2, 3 = 4 blocks");
    }

    #[test]
    fn farming_positions_and_tool_energy_costs() {
        // ModuleFarmingUnit.java:119-123 & GearConfig.java:240-242
        let pos = [5, 10, 15];

        // Farming radii: Low = 1x1, Med = 3x3, High = 5x5, Ultra = 7x7
        assert_eq!(FarmingRadius::Low.radius(), 0);
        assert_eq!(FarmingRadius::Med.radius(), 1);
        assert_eq!(FarmingRadius::High.radius(), 2);
        assert_eq!(FarmingRadius::Ultra.radius(), 3);

        let positions_med = find_farming_positions(pos, HitFace::Up, 1);
        assert_eq!(positions_med.len(), 9, "3x3 = 9 blocks");

        let positions_ultra = find_farming_positions(pos, HitFace::Up, 3);
        assert_eq!(positions_ultra.len(), 49, "7x7 = 49 blocks");

        // Tool action costs
        assert_eq!(ENERGY_USAGE_HOE, 4);
        assert_eq!(ENERGY_USAGE_SHOVEL, 4);
        assert_eq!(ENERGY_USAGE_AXE, 4);
        assert_eq!(ENERGY_USAGE_SHEAR_ENTITY, 4);
        assert_eq!(ENERGY_USAGE_SHEAR_TRIM, 4);
    }

    #[test]
    fn teleportation_checks_and_energy_cost() {
        // ItemMekaTool.java:373-417 & GearConfig.java:245
        let player = [0.0, 60.0, 0.0];

        // Too close: dx=1, dy=1, dz=1 -> dist_sqr = 3.0 < 5.0
        let close = check_teleport(player, [1, 61, 1], true, true, true, true, 10_000);
        assert!(close.is_err(), "Fails if dist_sqr < 5.0");

        // Obstruction at target
        let blocked = check_teleport(player, [10, 60, 0], false, true, true, true, 10_000);
        assert!(blocked.is_err(), "Fails if head space not clear");

        // Valid teleport at distance 20 blocks
        // energy = ceil(400 * 20 / 10) = 800 FE
        let valid = check_teleport(player, [20, 60, 0], true, true, true, true, 10_000);
        assert!(valid.is_ok());
        let (target, energy) = valid.unwrap();
        assert_eq!(target, [20.5, 61.5, 0.5], "Target centered +0.5, +1.5, +0.5");
        assert_eq!(energy, 800, "20 blocks * 40 FE/block = 800 FE");

        // Insufficient energy
        let low_energy = check_teleport(player, [20, 60, 0], true, true, true, true, 500);
        assert!(low_energy.is_err(), "Fails when energy < 800");
    }

    #[test]
    fn module_serialization_and_deserialization() {
        let mut tool = MekaToolData::new(5_000_000);
        assert!(tool.install_module(ModuleType::EnergyUnit).is_ok());
        assert!(tool.install_module(ModuleType::AttackAmplificationUnit).is_ok());
        assert!(tool.install_module(ModuleType::SilkTouchUnit).is_ok());

        let json = serialize_to_components(&tool);
        assert!(json.contains("\"energy_containers\":[5000000]"));
        assert!(json.contains("mekanism:energy_unit"));
        assert!(json.contains("mekanism:attack_amplification_unit"));
        assert!(json.contains("mekanism:silk_touch_unit"));

        let restored = parse_from_components(&json).expect("Should parse back");
        assert_eq!(restored.energy, 5_000_000);
        assert!(restored.is_module_enabled(ModuleType::EnergyUnit));
        assert!(restored.is_module_enabled(ModuleType::AttackAmplificationUnit));
        assert!(restored.is_module_enabled(ModuleType::SilkTouchUnit));
    }
}

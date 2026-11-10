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
pub const ENERGY_USAGE_BASE_JOULES: u64 = 10;
pub const ENERGY_USAGE_BASE: u64 = fe(ENERGY_USAGE_BASE_JOULES);

/// Block break energy usage with Silk Touch: 100 J = 40 FE (`GearConfig.java:238`).
pub const ENERGY_USAGE_SILK_JOULES: u64 = 100;
pub const ENERGY_USAGE_SILK: u64 = fe(ENERGY_USAGE_SILK_JOULES);

/// Base weapon attack energy usage: 2_000 J = 800 FE (`GearConfig.java:239`).
pub const ENERGY_USAGE_WEAPON: u64 = fe(2_000);

/// Hoe use energy usage: 10 J = 4 FE (`GearConfig.java:240`).
pub const ENERGY_USAGE_HOE: u64 = fe(10);

/// Shovel use energy usage (flatten/douse): 10 J = 4 FE (`GearConfig.java:241`).
pub const ENERGY_USAGE_SHOVEL: u64 = fe(10);

/// Axe use energy usage (strip/scrape/wax-off): 10 J = 4 FE (`GearConfig.java:242`).
pub const ENERGY_USAGE_AXE: u64 = fe(10);

/// Shear entity energy usage: 10 J = 4 FE (`GearConfig.java:243`).
pub const ENERGY_USAGE_SHEAR_ENTITY_JOULES: u64 = 10;
pub const ENERGY_USAGE_SHEAR_ENTITY: u64 = fe(ENERGY_USAGE_SHEAR_ENTITY_JOULES);

/// Shear block (trim) energy usage: 10 J = 4 FE (`GearConfig.java:244`).
pub const ENERGY_USAGE_SHEAR_TRIM_JOULES: u64 = 10;
pub const ENERGY_USAGE_SHEAR_TRIM: u64 = fe(ENERGY_USAGE_SHEAR_TRIM_JOULES);

/// Base teleport energy usage per 10 blocks: 1_000 J = 400 FE (`GearConfig.java:245`).
pub const ENERGY_USAGE_TELEPORT_JOULES: u64 = 1_000;
pub const ENERGY_USAGE_TELEPORT: u64 = fe(ENERGY_USAGE_TELEPORT_JOULES);

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
    /// What the container holds, in Joules (the original counts Joules, the port Forge Energy).
    fn stored_joules(&self) -> u128 {
        u128::from(self.energy) * 5 / 2
    }

    /// `ItemMekaTool.hasEnergyForDigAction` (java:135-146): the energy for a block of hardness 0 (the least a dig can cost), or at least a ratio above 1e-5 of it
    /// (the dig then goes slower).
    pub fn has_energy_for_dig_action(&self) -> bool {
        let required = u128::from(self.get_destroy_joules(0.0));
        let available = self.stored_joules();
        required <= available || (available as f64 / required as f64) > 1.0e-5
    }

    /// `ModuleShearingUnit.canPerformAction(SHEARS_DISARM)` (java:~22-30): the energy for a block of hardness 0 with the silk touch setting.
    pub fn can_shear_disarm(&self) -> bool {
        let cost = u128::from(self.get_destroy_joules(0.0));
        cost == 0 || self.stored_joules() >= cost
    }

    /// Shearing an entity or trimming a block (`ModuleShearingUnit.onInteract`/`onItemUse`): 10 J each, free when the cost is 0; refused without the energy.
    /// Returns whether it can go on, taking the energy when it does.
    pub fn use_shear_energy(&mut self, joules: u64) -> bool {
        if joules == 0 {
            return true;
        }
        if self.stored_joules() < u128::from(joules) {
            return false;
        }
        // The port counts Forge Energy: the conversion is rounded up so that a use is never cheaper than the original.
        let fe_cost = (joules * 2).div_ceil(5);
        self.energy = self.energy.saturating_sub(fe_cost);
        true
    }

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

    /// Base destroy energy in Joules, before efficiency and hardness (`getDestroyEnergy(boolean silk)`, ItemMekaTool.java:326).
    pub fn base_destroy_joules(&self) -> u64 {
        if self.has_silk_touch() {
            ENERGY_USAGE_SILK_JOULES
        } else {
            ENERGY_USAGE_BASE_JOULES
        }
    }

    /// Base destroy energy in FE before block hardness scaling.
    pub fn base_destroy_energy(&self) -> u64 {
        fe(self.base_destroy_joules())
    }

    /// Destroy energy in Joules with the efficiency: `MathUtils.clampToLong(destroyEnergy * efficiency)` (ItemMekaTool.java:151-156): a `long`
    /// times a `float`, so the product is a float. With the excavation unit off the efficiency is 0 and so is the cost.
    pub fn mod_destroy_joules(&self) -> u64 {
        let product = self.base_destroy_joules() as f32 * self.efficiency();
        if product <= 0.0 { 0 } else { product as u64 }
    }

    /// Modified destroy energy in FE (the original works in Joules, the port counts Forge Energy).
    pub fn mod_destroy_energy(&self) -> u64 {
        fe(self.mod_destroy_joules())
    }

    /// Destroy energy in Joules for a block of this hardness: `hardness == 0 ? Math.max(base / 2, 1) : base` (ItemMekaTool.java:335).
    pub fn get_destroy_joules(&self, hardness: f32) -> u64 {
        let joules = self.mod_destroy_joules();
        if hardness == 0.0 { (joules / 2).max(1) } else { joules }
    }

    /// Total destroy energy in FE required to break a block with given hardness.
    pub fn get_destroy_energy(&self, hardness: f32) -> u64 {
        fe(self.get_destroy_joules(hardness))
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

/// `IBlastingItem.canBlastBlock` (java:82-84): not air, not a liquid, not a bounding block (`ModuleVeinMiningUnit.canVeinBlock`) and with a destroy speed above 0
/// (so bedrock, hardness -1, and a block of hardness 0 are left alone).
pub fn can_blast_block(is_air: bool, is_liquid: bool, is_bounding_block: bool, hardness: f32) -> bool {
    !is_air && !is_liquid && !is_bounding_block && hardness > 0.0
}

/// Finds positions affected by Blasting Unit (`IBlastingItem.findPositions`, java:26-60). The positions come in the order of `BlockPos.betweenClosed`
/// (x fastest, then y, then z).
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
    let (lower, upper) = match hit_face {
        HitFace::Up | HitFace::Down => ([-r, 0, -r], [r, 0, r]),
        HitFace::East | HitFace::West => ([0, -1, -r], [0, 2 * r - 1, r]),
        HitFace::North | HitFace::South => ([-r, -1, 0], [r, 2 * r - 1, 0]),
    };
    let mut positions = Vec::new();
    for dz in lower[2]..=upper[2] {
        for dy in lower[1]..=upper[1] {
            for dx in lower[0]..=upper[0] {
                let pos = [target_pos[0] + dx, target_pos[1] + dy, target_pos[2] + dz];
                if can_blast(pos) {
                    positions.push(pos);
                }
            }
        }
    }
    positions
}

/// One block that the blasting or the vein mining wants to break (`found` of `MekanismUtils.veinMineArea`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AreaTarget {
    pub pos: [i32; 3],
    /// 0 for the blasted blocks (and the first block of a vein), the traversal distance for the vein.
    pub distance: u32,
    pub hardness: f32,
    /// The block is an ore of the vein tracker (`ATOMIC_DISASSEMBLER_ORE`): the exponent of the energy is 1.5, otherwise 2.
    pub is_ore: bool,
    pub is_air: bool,
}

/// `MekanismUtils.veinMineArea`: which blocks get broken and how many Joules they use. `energy_required` is what the target block takes
/// (`getDestroyEnergy` with the efficiency); the rest of the stored energy pays the others in order. A block is **skipped when `used + cost >= available`**
/// (equal does not pass), but the next ones are still tried. Air and indestructible blocks (hardness -1) and the target itself are passed over.
/// The base for the blasted blocks (`base_blast_joules`) includes the efficiency, the one for the vein (`base_vein_joules`) does not.
pub fn plan_area_mining(
    stored_joules: u64,
    energy_required: u64,
    target: [i32; 3],
    base_blast_joules: u64,
    base_vein_joules: u64,
    found: &[AreaTarget],
) -> (Vec<[i32; 3]>, u64) {
    let available = stored_joules as i128 - energy_required as i128;
    let mut used: i128 = 0;
    let mut broken = Vec::new();
    for t in found {
        if t.pos == target || t.is_air || t.hardness < 0.0 {
            continue;
        }
        let cost = if t.distance == 0 {
            // `blastEnergy.calc(baseBlastEnergy, hardness)` = `getDestroyEnergy(base, hardness)`
            if t.hardness == 0.0 { (base_blast_joules / 2).max(1) } else { base_blast_joules }
        } else {
            // `ceilToLong(getDestroyEnergy(base, hardness) * 0.5 * pow(distance, ore ? 1.5 : 2))`
            let base = if t.hardness == 0.0 { (base_vein_joules / 2).max(1) } else { base_vein_joules };
            let multiplier = 0.5 * f64::from(t.distance).powf(if t.is_ore { 1.5 } else { 2.0 });
            (base as f64 * multiplier).ceil() as u64
        };
        if used + cost as i128 >= available {
            continue;
        }
        used += cost as i128;
        broken.push(t.pos);
    }
    (broken, used as u64)
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
    tracked_types: usize,
    matches_target_block: impl Fn([i32; 3]) -> bool,
) -> Vec<([i32; 3], u32)> {
    use std::collections::{HashSet, VecDeque};

    // `maxCount = initial.size() + maxVein * oreTracker.size()` (ModuleVeinMiningUnit.java:139-140): the limit grows with each tracked block type.
    let max_count = initial_blocks.len() + MAX_VEIN_MINING_COUNT * tracked_types;
    let mut found: Vec<([i32; 3], u32)> = Vec::new();
    let mut found_set: HashSet<[i32; 3]> = HashSet::new();
    let mut frontier: VecDeque<[i32; 3]> = VecDeque::new();
    let mut in_frontier: HashSet<[i32; 3]> = HashSet::new();
    for &p in initial_blocks {
        if in_frontier.insert(p) {
            frontier.push_back(p);
        }
    }
    // `TraversalDistance`: the distance goes up when every block of the current layer has been taken off the frontier.
    let mut distance = 0u32;
    let mut next = frontier.len();
    while let Some(pos) = frontier.pop_front() {
        in_frontier.remove(&pos);
        // A block counts as found when it comes off the frontier, and the limit is checked right then.
        found.push((pos, distance));
        found_set.insert(pos);
        if found.len() >= max_count {
            break;
        }
        // If the block is an ore, or the distance is within the extended range (ModuleVeinMiningUnit.java:158).
        if is_ore || extended_range > distance {
            // `BlockPos.betweenClosed` goes x fastest, then y, then z.
            for dz in -1..=1 {
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let n = [pos[0] + dx, pos[1] + dy, pos[2] + dz];
                        if !found_set.contains(&n) && !in_frontier.contains(&n) && matches_target_block(n) {
                            in_frontier.insert(n);
                            frontier.push_back(n);
                        }
                    }
                }
            }
        }
        // `dist.updateDistance(found.size(), frontier.size())`
        if found.len() == next {
            distance += 1;
            next += frontier.len();
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

/// The tools of `ModuleFarmingUnit.onItemUse`: the axe (strip, scrape, wax off) works on a plane facing the clicked side, the shovel (flatten, douse) and the hoe
/// (till) on the horizontal plane around the block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FarmingTool {
    Axe,
    Shovel,
    Hoe,
}

/// The blocks a farming use covers (`IToolAOEData.getTargetPositions`, `ModuleFarmingUnit.java:222-310`), in the order of `BlockPos.betweenClosed` (x fastest,
/// then y, then z). `radius` is `(diameter - 1) / 2`; with the `Off` setting (diameter 0) it does nothing and returns no position.
pub fn find_farming_positions(tool: FarmingTool, target_pos: [i32; 3], hit_face: HitFace, farming: FarmingRadius) -> Vec<[i32; 3]> {
    if farming == FarmingRadius::Off {
        return Vec::new();
    }
    let r = i32::from(farming.radius());
    // Axe: `EAST/WEST (0, r, r)`, `UP/DOWN (r, 0, r)`, `SOUTH/NORTH (r, r, 0)`; shovel and hoe: `(r, 0, r)` whatever the face.
    let (rx, ry, rz) = match (tool, hit_face) {
        (FarmingTool::Axe, HitFace::East | HitFace::West) => (0, r, r),
        (FarmingTool::Axe, HitFace::North | HitFace::South) => (r, r, 0),
        _ => (r, 0, r),
    };
    let mut positions = Vec::new();
    for dz in -rz..=rz {
        for dy in -ry..=ry {
            for dx in -rx..=rx {
                positions.push([target_pos[0] + dx, target_pos[1] + dy, target_pos[2] + dz]);
            }
        }
    }
    positions
}

/// One block the farming use may change: it is valid when `IToolAOEData.isValid` says so and its tool-modified state is the one of the clicked block.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FarmingCandidate {
    pub pos: [i32; 3],
    pub valid: bool,
}

/// The energy of a farming use (`ModuleFarmingUnit.useAOE`, java:108-190): it fails (`None`) with less than one use of energy. The clicked block is always
/// changed and paid; then each other candidate, in order, is paid `usage` more, and the loop **stops** when `used + usage` would be above the energy (equal passes).
/// Returns the blocks changed (the first one included) and the energy used.
pub fn plan_farming_use(energy: u64, usage: u64, target: [i32; 3], candidates: &[FarmingCandidate]) -> Option<(Vec<[i32; 3]>, u64)> {
    if energy < usage {
        return None;
    }
    let mut used = usage;
    let mut changed = vec![target];
    for c in candidates {
        if c.pos == target {
            continue;
        }
        let next = used + usage;
        if next > energy {
            break;
        }
        if c.valid {
            used = next;
            changed.push(c.pos);
        }
    }
    Some((changed, used))
}

/// `ShovelToolAOEData.isValid`: a block can be flattened when the one above is air, or a replaceable plant (the farming-override tag, or
/// replaceable and in `REPLACEABLE_BY_TREES`) that has no fluid and does not render as solid.
pub fn shovel_position_valid(above_is_air: bool, above_is_replaceable_plant: bool, above_has_fluid: bool, above_is_solid_render: bool) -> bool {
    if above_is_air {
        return true;
    }
    above_is_replaceable_plant && !above_has_fluid && !above_is_solid_render
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

    // `ItemMekaTool.use` (java:380-392): the ray is cut at `mekaToolMaxTeleportReach` by the host; the energy is
    // `ceilToLong(mekaToolEnergyUsageTeleport * (distance / 10D))` where `distance` is **`distanceToSqr`**, the squared distance, in Joules.
    // The port counts Forge Energy and rounds the conversion up, so a teleport is never cheaper than the original.
    let joules = (ENERGY_USAGE_TELEPORT_JOULES as f64 * (dist_sqr / 10.0)).ceil() as u64;
    let energy_needed = (joules * 2).div_ceil(5);

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
        let found_ores = find_vein_positions(&initial, 0, true, 1, |p| p[0] >= 0 && p[0] <= 9 && p[1] == 0 && p[2] == 0);
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
        let found_non_ore = find_vein_positions(&initial, 3, false, 1, |p| p[0] >= 0 && p[0] <= 10 && p[1] == 0 && p[2] == 0);
        // extended_range = 3 allows expansion from dist 0, 1, 2 up to dist 3
        assert_eq!(found_non_ore.len(), 4, "Covers dist 0, 1, 2, 3 = 4 blocks");
    }

    #[test]
    fn java_vein_limit_grows_with_the_tracked_block_types() {
        // ModuleVeinMiningUnit.java:139-140: maxCount = initial.size() + miningCount (128) * oreTracker.size(); a block counts as found when it
        // comes off the frontier and the limit is checked then.
        let line = |p: [i32; 3]| p[1] == 0 && p[2] == 0 && p[0] >= 0 && p[0] < 1_000;
        assert_eq!(find_vein_positions(&[[0, 0, 0]], 0, true, 1, line).len(), 1 + 128);
        assert_eq!(find_vein_positions(&[[0, 0, 0]], 0, true, 2, line).len(), 1 + 256);
        assert_eq!(find_vein_positions(&[[0, 0, 0], [500, 0, 0]], 0, true, 1, line).len(), 2 + 128);
    }

    #[test]
    fn java_vein_neighbours_go_x_then_y_then_z() {
        // `BlockPos.betweenClosed` (used by ModuleVeinMiningUnit.findPositions) goes x fastest, then y, then z.
        let cube = |p: [i32; 3]| p.iter().all(|c| (-1..=1).contains(c));
        let found = find_vein_positions(&[[0, 0, 0]], 5, false, 1, cube);
        assert_eq!(found.len(), 27);
        assert_eq!(found[0], ([0, 0, 0], 0));
        assert_eq!(found[1], ([-1, -1, -1], 1));
        assert_eq!(found[2], ([0, -1, -1], 1));
        assert_eq!(found[3], ([1, -1, -1], 1));
        assert_eq!(found[4], ([-1, 0, -1], 1));
    }

    #[test]
    fn java_excavation_unit_off_costs_nothing() {
        // ItemMekaTool.java:151-156: clampToLong(destroyEnergy * efficiency); the Off mode has efficiency 0, so a block with hardness costs 0.
        let mut tool = MekaToolData::new(100_000);
        assert!(tool.install_module(ModuleType::ExcavationEscalationUnit).is_ok());
        tool.get_module_mut(ModuleType::ExcavationEscalationUnit).unwrap().excavation_mode = ExcavationMode::Off;
        assert_eq!(tool.efficiency(), 0.0);
        assert_eq!(tool.get_destroy_energy(1.5), 0);
        // A block of hardness 0 still takes `Math.max(base / 2, 1)` Joules, which is below one Forge Energy.
        assert_eq!(tool.get_destroy_joules(0.0), 1);
    }

    #[test]
    fn java_blast_energy_rule_skips_when_used_plus_cost_reaches_the_available() {
        // `MekanismUtils.veinMineArea`: available = stored - energyRequired; a block is skipped when `used + cost >= available`.
        // Default efficiency: 10 J * 4 = 40 J a block. Target + 8 blasted neighbours at hardness 1.5, stored exactly 360 J: available 320.
        let target = [0, 0, 0];
        let mut found = vec![AreaTarget { pos: target, distance: 0, hardness: 1.5, is_ore: false, is_air: false }];
        for x in 1..=8 {
            found.push(AreaTarget { pos: [x, 0, 0], distance: 0, hardness: 1.5, is_ore: false, is_air: false });
        }
        let (broken, used) = plan_area_mining(360, 40, target, 40, 10, &found);
        assert_eq!(broken.len(), 7, "the 8th would make used + cost = 320, not below 320");
        assert_eq!(used, 280);
        // One joule more and the 8th passes.
        let (broken, used) = plan_area_mining(361, 40, target, 40, 10, &found);
        assert_eq!((broken.len(), used), (8, 320));
        // Not enough for the target itself: nothing else is mined.
        assert_eq!(plan_area_mining(30, 40, target, 40, 10, &found).0.len(), 0);
        // Air and indestructible blocks are passed over without cost; hardness 0 costs half.
        let mixed = [
            AreaTarget { pos: [1, 0, 0], distance: 0, hardness: 1.5, is_ore: false, is_air: true },
            AreaTarget { pos: [2, 0, 0], distance: 0, hardness: -1.0, is_ore: false, is_air: false },
            AreaTarget { pos: [3, 0, 0], distance: 0, hardness: 0.0, is_ore: false, is_air: false },
        ];
        assert_eq!(plan_area_mining(1_000, 40, target, 40, 10, &mixed), (vec![[3, 0, 0]], 20));
        // A vein block at distance 4 of an ore costs ceil(10 J * 0.5 * 4^1.5) = 40 J, a non-ore one ceil(10 * 0.5 * 4^2) = 80 J.
        let vein = [
            AreaTarget { pos: [1, 0, 0], distance: 4, hardness: 1.5, is_ore: true, is_air: false },
            AreaTarget { pos: [2, 0, 0], distance: 4, hardness: 1.5, is_ore: false, is_air: false },
        ];
        assert_eq!(plan_area_mining(10_000, 40, target, 40, 10, &vein).1, 120);
    }

    #[test]
    fn java_blast_block_filter_and_order() {
        // `IBlastingItem.canBlastBlock`
        assert!(can_blast_block(false, false, false, 1.5));
        assert!(!can_blast_block(true, false, false, 1.5), "air");
        assert!(!can_blast_block(false, true, false, 1.5), "liquid");
        assert!(!can_blast_block(false, false, true, 1.5), "bounding block");
        assert!(!can_blast_block(false, false, false, 0.0), "a destroy speed of 0 is left alone");
        assert!(!can_blast_block(false, false, false, -1.0), "bedrock");
        // The horizontal plane of an UP/DOWN face goes x fastest, then z.
        let plane = find_blasting_positions([10, 60, 20], HitFace::Up, 1, |_| true);
        assert_eq!(plane[0], [9, 60, 19]);
        assert_eq!(plane[1], [10, 60, 19]);
        assert_eq!(plane[3], [9, 60, 20]);
        assert_eq!(plane.len(), 9);
        // A wall: radius 2 on NORTH is x -2..=2, y -1..=3.
        let wall = find_blasting_positions([0, 10, 0], HitFace::North, 2, |_| true);
        assert_eq!(wall.len(), 25);
        assert_eq!(wall[0], [-2, 9, 0]);
        assert_eq!(wall[24], [2, 13, 0]);
    }

    #[test]
    fn java_farming_planes_per_tool_and_the_off_setting() {
        // ModuleFarmingUnit: the axe uses a plane facing the clicked side, the shovel and the hoe the horizontal plane; OFF (diameter 0) does nothing.
        let pos = [5, 10, 15];
        assert!(find_farming_positions(FarmingTool::Hoe, pos, HitFace::Up, FarmingRadius::Off).is_empty());
        assert_eq!(find_farming_positions(FarmingTool::Hoe, pos, HitFace::Up, FarmingRadius::Low), vec![pos], "diameter 1: the block alone");
        // A hoe or shovel clicking a wall still works on the horizontal plane around the block.
        let hoe_wall = find_farming_positions(FarmingTool::Hoe, pos, HitFace::North, FarmingRadius::Med);
        assert_eq!(hoe_wall.len(), 9);
        assert!(hoe_wall.iter().all(|p| p[1] == 10));
        let shovel_wall = find_farming_positions(FarmingTool::Shovel, pos, HitFace::East, FarmingRadius::High);
        assert_eq!(shovel_wall.len(), 25);
        assert!(shovel_wall.iter().all(|p| p[1] == 10));
        // An axe on a north face covers x and y with z fixed; on an east face y and z with x fixed.
        let axe_north = find_farming_positions(FarmingTool::Axe, pos, HitFace::North, FarmingRadius::Med);
        assert!(axe_north.iter().all(|p| p[2] == 15) && axe_north.len() == 9);
        let axe_east = find_farming_positions(FarmingTool::Axe, pos, HitFace::East, FarmingRadius::Med);
        assert!(axe_east.iter().all(|p| p[0] == 5) && axe_east.len() == 9);
        // `betweenClosed` order: x fastest, then y, then z.
        assert_eq!(axe_north[0], [4, 9, 15]);
        assert_eq!(axe_north[1], [5, 9, 15]);
        assert_eq!(axe_north[3], [4, 10, 15]);
        assert_eq!(axe_east[0], [5, 9, 14]);
        assert_eq!(axe_east[1], [5, 10, 14]);
    }

    #[test]
    fn java_farming_energy_stops_when_the_next_use_would_exceed_it() {
        // `useAOE`: it fails without one use of energy; each other block is paid `usage` more; the loop stops when `used + usage > energy` (equal passes).
        let target = [0, 0, 0];
        let line: Vec<FarmingCandidate> = (0..6).map(|x| FarmingCandidate { pos: [x, 0, 0], valid: true }).collect();
        assert_eq!(plan_farming_use(3, 4, target, &line), None);
        assert_eq!(plan_farming_use(4, 4, target, &line), Some((vec![target], 4)));
        // Energy 10, usage 4: the target (4), one more (8), the next would be 12 > 10: stop.
        assert_eq!(plan_farming_use(10, 4, target, &line), Some((vec![target, [1, 0, 0]], 8)));
        // Energy 8: the second use brings it to exactly 8, which is allowed.
        assert_eq!(plan_farming_use(8, 4, target, &line).unwrap().1, 8);
        // A block that cannot be changed costs nothing and does not stop the loop.
        let mixed = [FarmingCandidate { pos: [1, 0, 0], valid: false }, FarmingCandidate { pos: [2, 0, 0], valid: true }];
        assert_eq!(plan_farming_use(100, 4, target, &mixed), Some((vec![target, [2, 0, 0]], 8)));
        // A shovel needs air above, or a plant that is not solid and has no fluid.
        assert!(shovel_position_valid(true, false, false, false));
        assert!(shovel_position_valid(false, true, false, false));
        assert!(!shovel_position_valid(false, true, true, false));
        assert!(!shovel_position_valid(false, true, false, true));
        assert!(!shovel_position_valid(false, false, false, false));
    }

    #[test]
    fn java_shearing_and_dig_energy_rules() {
        // ModuleShearingUnit: 10 J (4 FE) to shear an entity or trim a block, free if the cost is 0; the disarm needs the destroy energy of hardness 0
        // (with the efficiency: 10 J * 4 / 2 = 20 J = 8 FE, with silk touch 100 * 4 / 2 = 200 J = 80 FE).
        let mut tool = MekaToolData::new(100_000);
        tool.energy = 3;
        assert!(!tool.use_shear_energy(ENERGY_USAGE_SHEAR_ENTITY_JOULES));
        assert_eq!(tool.energy, 3);
        tool.energy = 4;
        assert!(tool.use_shear_energy(ENERGY_USAGE_SHEAR_TRIM_JOULES));
        assert_eq!(tool.energy, 0);
        assert!(tool.use_shear_energy(0), "a free use needs no energy");
        tool.energy = 7;
        assert!(!tool.can_shear_disarm());
        tool.energy = 8;
        assert!(tool.can_shear_disarm());
        assert!(tool.install_module(ModuleType::SilkTouchUnit).is_ok());
        tool.energy = 79;
        assert!(!tool.can_shear_disarm());
        tool.energy = 80;
        assert!(tool.can_shear_disarm());
        // `hasEnergyForDigAction`: enough for hardness 0, or any energy above a ratio of 1e-5 of it (the dig goes slower).
        let mut empty = MekaToolData::new(100_000);
        empty.energy = 0;
        assert!(!empty.has_energy_for_dig_action());
        empty.energy = 1;
        assert!(empty.has_energy_for_dig_action());
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

        let positions_med = find_farming_positions(FarmingTool::Hoe, pos, HitFace::Up, FarmingRadius::Med);
        assert_eq!(positions_med.len(), 9, "3x3 = 9 blocks");

        let positions_ultra = find_farming_positions(FarmingTool::Hoe, pos, HitFace::Up, FarmingRadius::Ultra);
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

        // Valid teleport 20 blocks away: `distance` in the original is `distanceToSqr` = 400, so
        // energy = ceil(1000 J * (400 / 10)) = 40 000 J = 16 000 FE (`ItemMekaTool.java:389`).
        let valid = check_teleport(player, [20, 60, 0], true, true, true, true, 100_000);
        assert!(valid.is_ok());
        let (target, energy) = valid.unwrap();
        assert_eq!(target, [20.5, 61.5, 0.5], "Target centered +0.5, +1.5, +0.5");
        assert_eq!(energy, 16_000, "1000 J * 400 / 10 = 40 000 J");
        // 50 blocks: distanceToSqr 2500 -> 250 000 J = 100 000 FE: the cost grows with the square of the distance.
        assert_eq!(check_teleport(player, [50, 60, 0], true, true, true, true, 1_000_000).unwrap().1, 100_000);
        // The edge of the "too close" check: distanceToSqr 5 is allowed, below 5 passes the use to the game.
        assert!(check_teleport(player, [2, 61, 0], true, true, true, true, 100_000).is_ok(), "distanceToSqr 5.0 is not below 5");
        assert!(check_teleport(player, [2, 60, 0], true, true, true, true, 100_000).is_err(), "distanceToSqr 4.0 is below 5");

        // Insufficient energy
        let low_energy = check_teleport(player, [20, 60, 0], true, true, true, true, 15_999);
        assert!(low_energy.is_err(), "Fails when energy < 16 000");
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

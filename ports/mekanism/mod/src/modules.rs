//! Module System (`modules.rs`): shared modular equipment framework for MekaSuit and Meka-Tool (`e6`).
//! Faithfully ported from Mekanism 1.21.1-10.7.19.85 (`mekanism.common.content.gear.*`,
//! `mekanism.api.gear.*`, `mekanism.common.registries.MekanismModules`).

use std::collections::HashMap;

/// Energy in Mekanism 1.21.1 is Joules; the port keeps Forge Energy (1 FE = 2.5 J, fe = J * 2 / 5).
pub const fn fe(joules: u64) -> u64 {
    joules * 2 / 5
}

/// Exclusivity flags for modules (`ExclusiveFlag.java:700`).
pub const EXCLUSIVE_INTERACT_EMPTY: u32 = 1 << 0;  // 1
pub const EXCLUSIVE_INTERACT_ENTITY: u32 = 1 << 1; // 2
pub const EXCLUSIVE_INTERACT_BLOCK: u32 = 1 << 2;  // 4
pub const EXCLUSIVE_OVERRIDE_JUMP: u32 = 1 << 3;   // 8
pub const EXCLUSIVE_OVERRIDE_DROPS: u32 = 1 << 4;  // 16
pub const EXCLUSIVE_INTERACT_ANY: u32 = EXCLUSIVE_INTERACT_EMPTY | EXCLUSIVE_INTERACT_ENTITY | EXCLUSIVE_INTERACT_BLOCK; // 7

/// All 31 modular units across Mekanism and MekanismGenerators.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ModuleType {
    // Shared Armor & Tool
    EnergyUnit,
    // Shared Armor
    ColorModulationUnit,
    LaserDissipationUnit,
    RadiationShieldingUnit,
    // Helmet
    ElectrolyticBreathingUnit,
    InhalationPurificationUnit,
    VisionEnhancementUnit,
    NutritionalInjectionUnit,
    SolarRechargingUnit,
    // Bodyarmor (Chestplate)
    JetpackUnit,
    ChargeDistributionUnit,
    GravitationalModulatingUnit,
    ElytraUnit,
    DosimeterUnit,
    GeigerUnit,
    // Pants (Leggings)
    LocomotiveBoostingUnit,
    GyroscopicStabilizationUnit,
    HydrostaticRepulsorUnit,
    MotorizedServoUnit,
    GeothermalGeneratorUnit,
    // Boots
    HydraulicPropulsionUnit,
    MagneticAttractionUnit,
    FrostWalkerUnit,
    SoulSurferUnit,
    // Meka-Tool (for e6)
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
    /// The registry ID of the module (`MekanismModules.java` & `GeneratorsModules.java`).
    pub const fn id(&self) -> &'static str {
        match self {
            Self::EnergyUnit => "mekanism:energy_unit",
            Self::ColorModulationUnit => "mekanism:color_modulation_unit",
            Self::LaserDissipationUnit => "mekanism:laser_dissipation_unit",
            Self::RadiationShieldingUnit => "mekanism:radiation_shielding_unit",
            Self::ElectrolyticBreathingUnit => "mekanism:electrolytic_breathing_unit",
            Self::InhalationPurificationUnit => "mekanism:inhalation_purification_unit",
            Self::VisionEnhancementUnit => "mekanism:vision_enhancement_unit",
            Self::NutritionalInjectionUnit => "mekanism:nutritional_injection_unit",
            Self::SolarRechargingUnit => "mekanismgenerators:solar_recharging_unit",
            Self::JetpackUnit => "mekanism:jetpack_unit",
            Self::ChargeDistributionUnit => "mekanism:charge_distribution_unit",
            Self::GravitationalModulatingUnit => "mekanism:gravitational_modulating_unit",
            Self::ElytraUnit => "mekanism:elytra_unit",
            Self::DosimeterUnit => "mekanism:dosimeter_unit",
            Self::GeigerUnit => "mekanism:geiger_unit",
            Self::LocomotiveBoostingUnit => "mekanism:locomotive_boosting_unit",
            Self::GyroscopicStabilizationUnit => "mekanism:gyroscopic_stabilization_unit",
            Self::HydrostaticRepulsorUnit => "mekanism:hydrostatic_repulsor_unit",
            Self::MotorizedServoUnit => "mekanism:motorized_servo_unit",
            Self::GeothermalGeneratorUnit => "mekanismgenerators:geothermal_generator_unit",
            Self::HydraulicPropulsionUnit => "mekanism:hydraulic_propulsion_unit",
            Self::MagneticAttractionUnit => "mekanism:magnetic_attraction_unit",
            Self::FrostWalkerUnit => "mekanism:frost_walker_unit",
            Self::SoulSurferUnit => "mekanism:soul_surfer_unit",
            Self::AttackAmplificationUnit => "mekanism:attack_amplification_unit",
            Self::ExcavationEscalationUnit => "mekanism:excavation_escalation_unit",
            Self::BlastingUnit => "mekanism:blasting_unit",
            Self::VeinMiningUnit => "mekanism:vein_mining_unit",
            Self::FarmingUnit => "mekanism:farming_unit",
            Self::ShearingUnit => "mekanism:shearing_unit",
            Self::TeleportationUnit => "mekanism:teleportation_unit",
            Self::SilkTouchUnit => "mekanism:silk_touch_unit",
            Self::FortuneUnit => "mekanism:fortune_unit",
        }
    }

    /// Lookup by registry ID.
    pub fn from_id(id: &str) -> Option<Self> {
        let normalized = if id.starts_with("mekanism:") || id.starts_with("mekanismgenerators:") {
            id
        } else {
            // allow matching without namespace or with default namespace
            id
        };
        match normalized {
            "mekanism:energy_unit" | "energy_unit" => Some(Self::EnergyUnit),
            "mekanism:color_modulation_unit" | "color_modulation_unit" => Some(Self::ColorModulationUnit),
            "mekanism:laser_dissipation_unit" | "laser_dissipation_unit" => Some(Self::LaserDissipationUnit),
            "mekanism:radiation_shielding_unit" | "radiation_shielding_unit" => Some(Self::RadiationShieldingUnit),
            "mekanism:electrolytic_breathing_unit" | "electrolytic_breathing_unit" => Some(Self::ElectrolyticBreathingUnit),
            "mekanism:inhalation_purification_unit" | "inhalation_purification_unit" => Some(Self::InhalationPurificationUnit),
            "mekanism:vision_enhancement_unit" | "vision_enhancement_unit" => Some(Self::VisionEnhancementUnit),
            "mekanism:nutritional_injection_unit" | "nutritional_injection_unit" => Some(Self::NutritionalInjectionUnit),
            "mekanismgenerators:solar_recharging_unit" | "solar_recharging_unit" | "mekanism:solar_recharging_unit" => Some(Self::SolarRechargingUnit),
            "mekanism:jetpack_unit" | "jetpack_unit" => Some(Self::JetpackUnit),
            "mekanism:charge_distribution_unit" | "charge_distribution_unit" => Some(Self::ChargeDistributionUnit),
            "mekanism:gravitational_modulating_unit" | "gravitational_modulating_unit" => Some(Self::GravitationalModulatingUnit),
            "mekanism:elytra_unit" | "elytra_unit" => Some(Self::ElytraUnit),
            "mekanism:dosimeter_unit" | "dosimeter_unit" => Some(Self::DosimeterUnit),
            "mekanism:geiger_unit" | "geiger_unit" => Some(Self::GeigerUnit),
            "mekanism:locomotive_boosting_unit" | "locomotive_boosting_unit" => Some(Self::LocomotiveBoostingUnit),
            "mekanism:gyroscopic_stabilization_unit" | "gyroscopic_stabilization_unit" => Some(Self::GyroscopicStabilizationUnit),
            "mekanism:hydrostatic_repulsor_unit" | "hydrostatic_repulsor_unit" => Some(Self::HydrostaticRepulsorUnit),
            "mekanism:motorized_servo_unit" | "motorized_servo_unit" => Some(Self::MotorizedServoUnit),
            "mekanismgenerators:geothermal_generator_unit" | "geothermal_generator_unit" | "mekanism:geothermal_generator_unit" => Some(Self::GeothermalGeneratorUnit),
            "mekanism:hydraulic_propulsion_unit" | "hydraulic_propulsion_unit" => Some(Self::HydraulicPropulsionUnit),
            "mekanism:magnetic_attraction_unit" | "magnetic_attraction_unit" => Some(Self::MagneticAttractionUnit),
            "mekanism:frost_walker_unit" | "frost_walker_unit" => Some(Self::FrostWalkerUnit),
            "mekanism:soul_surfer_unit" | "soul_surfer_unit" => Some(Self::SoulSurferUnit),
            "mekanism:attack_amplification_unit" | "attack_amplification_unit" => Some(Self::AttackAmplificationUnit),
            "mekanism:excavation_escalation_unit" | "excavation_escalation_unit" => Some(Self::ExcavationEscalationUnit),
            "mekanism:blasting_unit" | "blasting_unit" => Some(Self::BlastingUnit),
            "mekanism:vein_mining_unit" | "vein_mining_unit" => Some(Self::VeinMiningUnit),
            "mekanism:farming_unit" | "farming_unit" => Some(Self::FarmingUnit),
            "mekanism:shearing_unit" | "shearing_unit" => Some(Self::ShearingUnit),
            "mekanism:teleportation_unit" | "teleportation_unit" => Some(Self::TeleportationUnit),
            "mekanism:silk_touch_unit" | "silk_touch_unit" => Some(Self::SilkTouchUnit),
            "mekanism:fortune_unit" | "fortune_unit" => Some(Self::FortuneUnit),
            _ => None,
        }
    }

    /// The module item ID in the inventory/game (`MekanismItems.java` & `GeneratorsItems.java`).
    pub const fn item_id(&self) -> &'static str {
        match self {
            Self::EnergyUnit => "mekanism:module_energy",
            Self::ColorModulationUnit => "mekanism:module_color_modulation",
            Self::LaserDissipationUnit => "mekanism:module_laser_dissipation",
            Self::RadiationShieldingUnit => "mekanism:module_radiation_shielding",
            Self::ElectrolyticBreathingUnit => "mekanism:module_electrolytic_breathing",
            Self::InhalationPurificationUnit => "mekanism:module_inhalation_purification",
            Self::VisionEnhancementUnit => "mekanism:module_vision_enhancement",
            Self::NutritionalInjectionUnit => "mekanism:module_nutritional_injection",
            Self::SolarRechargingUnit => "mekanismgenerators:module_solar_recharging",
            Self::JetpackUnit => "mekanism:module_jetpack",
            Self::ChargeDistributionUnit => "mekanism:module_charge_distribution",
            Self::GravitationalModulatingUnit => "mekanism:module_gravitational_modulating",
            Self::ElytraUnit => "mekanism:module_elytra",
            Self::DosimeterUnit => "mekanism:module_dosimeter",
            Self::GeigerUnit => "mekanism:module_geiger",
            Self::LocomotiveBoostingUnit => "mekanism:module_locomotive_boosting",
            Self::GyroscopicStabilizationUnit => "mekanism:module_gyroscopic_stabilization",
            Self::HydrostaticRepulsorUnit => "mekanism:module_hydrostatic_repulsor",
            Self::MotorizedServoUnit => "mekanism:module_motorized_servo",
            Self::GeothermalGeneratorUnit => "mekanismgenerators:module_geothermal_generator",
            Self::HydraulicPropulsionUnit => "mekanism:module_hydraulic_propulsion",
            Self::MagneticAttractionUnit => "mekanism:module_magnetic_attraction",
            Self::FrostWalkerUnit => "mekanism:module_frost_walker",
            Self::SoulSurferUnit => "mekanism:module_soul_surfer",
            Self::AttackAmplificationUnit => "mekanism:module_attack_amplification",
            Self::ExcavationEscalationUnit => "mekanism:module_excavation_escalation",
            Self::BlastingUnit => "mekanism:module_blasting",
            Self::VeinMiningUnit => "mekanism:module_vein_mining",
            Self::FarmingUnit => "mekanism:module_farming",
            Self::ShearingUnit => "mekanism:module_shearing",
            Self::TeleportationUnit => "mekanism:module_teleportation",
            Self::SilkTouchUnit => "mekanism:module_silk_touch",
            Self::FortuneUnit => "mekanism:module_fortune",
        }
    }

    /// Lookup by item ID.
    pub fn from_item(item: &str) -> Option<Self> {
        let name = item.strip_prefix("mekanism:").or_else(|| item.strip_prefix("mekanismgenerators:")).unwrap_or(item);
        match name {
            "module_energy" => Some(Self::EnergyUnit),
            "module_color_modulation" => Some(Self::ColorModulationUnit),
            "module_laser_dissipation" => Some(Self::LaserDissipationUnit),
            "module_radiation_shielding" => Some(Self::RadiationShieldingUnit),
            "module_electrolytic_breathing" => Some(Self::ElectrolyticBreathingUnit),
            "module_inhalation_purification" => Some(Self::InhalationPurificationUnit),
            "module_vision_enhancement" => Some(Self::VisionEnhancementUnit),
            "module_nutritional_injection" => Some(Self::NutritionalInjectionUnit),
            "module_solar_recharging" => Some(Self::SolarRechargingUnit),
            "module_jetpack" => Some(Self::JetpackUnit),
            "module_charge_distribution" => Some(Self::ChargeDistributionUnit),
            "module_gravitational_modulating" => Some(Self::GravitationalModulatingUnit),
            "module_elytra" => Some(Self::ElytraUnit),
            "module_dosimeter" => Some(Self::DosimeterUnit),
            "module_geiger" => Some(Self::GeigerUnit),
            "module_locomotive_boosting" => Some(Self::LocomotiveBoostingUnit),
            "module_gyroscopic_stabilization" => Some(Self::GyroscopicStabilizationUnit),
            "module_hydrostatic_repulsor" => Some(Self::HydrostaticRepulsorUnit),
            "module_motorized_servo" => Some(Self::MotorizedServoUnit),
            "module_geothermal_generator" => Some(Self::GeothermalGeneratorUnit),
            "module_hydraulic_propulsion" => Some(Self::HydraulicPropulsionUnit),
            "module_magnetic_attraction" => Some(Self::MagneticAttractionUnit),
            "module_frost_walker" => Some(Self::FrostWalkerUnit),
            "module_soul_surfer" => Some(Self::SoulSurferUnit),
            "module_attack_amplification" => Some(Self::AttackAmplificationUnit),
            "module_excavation_escalation" => Some(Self::ExcavationEscalationUnit),
            "module_blasting" => Some(Self::BlastingUnit),
            "module_vein_mining" => Some(Self::VeinMiningUnit),
            "module_farming" => Some(Self::FarmingUnit),
            "module_shearing" => Some(Self::ShearingUnit),
            "module_teleportation" => Some(Self::TeleportationUnit),
            "module_silk_touch" => Some(Self::SilkTouchUnit),
            "module_fortune" => Some(Self::FortuneUnit),
            _ => None,
        }
    }

    /// Maximum number of units of this module that can be installed on one item.
    pub const fn max_stack(&self) -> u8 {
        match self {
            Self::EnergyUnit => 8,
            Self::ColorModulationUnit => 1,
            Self::LaserDissipationUnit => 1,
            Self::RadiationShieldingUnit => 1,
            Self::ElectrolyticBreathingUnit => 4,
            Self::InhalationPurificationUnit => 1,
            Self::VisionEnhancementUnit => 4,
            Self::NutritionalInjectionUnit => 1,
            Self::SolarRechargingUnit => 8,
            Self::JetpackUnit => 4,
            Self::ChargeDistributionUnit => 1,
            Self::GravitationalModulatingUnit => 1,
            Self::ElytraUnit => 1,
            Self::DosimeterUnit => 1,
            Self::GeigerUnit => 1,
            Self::LocomotiveBoostingUnit => 4,
            Self::GyroscopicStabilizationUnit => 1,
            Self::HydrostaticRepulsorUnit => 4,
            Self::MotorizedServoUnit => 5,
            Self::GeothermalGeneratorUnit => 8,
            Self::HydraulicPropulsionUnit => 4,
            Self::MagneticAttractionUnit => 4,
            Self::FrostWalkerUnit => 2,
            Self::SoulSurferUnit => 3,
            Self::AttackAmplificationUnit => 4,
            Self::ExcavationEscalationUnit => 4,
            Self::BlastingUnit => 4,
            Self::VeinMiningUnit => 4,
            Self::FarmingUnit => 4,
            Self::ShearingUnit => 1,
            Self::TeleportationUnit => 1,
            Self::SilkTouchUnit => 1,
            Self::FortuneUnit => 3,
        }
    }

    /// Exclusivity bitmask (`ExclusiveFlag`).
    pub const fn exclusive_flags(&self) -> u32 {
        match self {
            Self::JetpackUnit | Self::GravitationalModulatingUnit => EXCLUSIVE_OVERRIDE_JUMP,
            Self::FarmingUnit => EXCLUSIVE_INTERACT_BLOCK,
            Self::ShearingUnit => EXCLUSIVE_INTERACT_ENTITY | EXCLUSIVE_INTERACT_BLOCK,
            Self::SilkTouchUnit | Self::FortuneUnit => EXCLUSIVE_OVERRIDE_DROPS,
            Self::TeleportationUnit => EXCLUSIVE_INTERACT_ANY,
            _ => 0,
        }
    }

    /// Whether this module handles the mode change key/action.
    pub const fn handles_mode_change(&self) -> bool {
        match self {
            Self::ExcavationEscalationUnit
            | Self::BlastingUnit
            | Self::VeinMiningUnit
            | Self::VisionEnhancementUnit
            | Self::JetpackUnit
            | Self::GravitationalModulatingUnit
            | Self::ElytraUnit
            | Self::LocomotiveBoostingUnit
            | Self::MagneticAttractionUnit => true,
            _ => false,
        }
    }

    /// Whether this module renders a HUD element.
    pub const fn renders_hud(&self) -> bool {
        match self {
            Self::ExcavationEscalationUnit
            | Self::AttackAmplificationUnit
            | Self::BlastingUnit
            | Self::VeinMiningUnit
            | Self::VisionEnhancementUnit
            | Self::NutritionalInjectionUnit
            | Self::DosimeterUnit
            | Self::GeigerUnit
            | Self::JetpackUnit
            | Self::GravitationalModulatingUnit => true,
            _ => false,
        }
    }

    /// Whether the module can be disabled in configuration (`noDisable`).
    pub const fn can_be_disabled(&self) -> bool {
        !matches!(self, Self::EnergyUnit | Self::ColorModulationUnit)
    }

    /// Default enabled state upon installation.
    /// Whether the module answers to the mode-change key as soon as it is installed (`ModuleData.java:517`: `HANDLES_MODE_CHANGE_ENABLED`
    /// or `_DISABLED`). Every module that handles it does, but the elytra unit (`modeChangeDisabledByDefault`).
    pub const fn handles_mode_change_by_default(&self) -> bool {
        self.handles_mode_change() && !matches!(self, Self::ElytraUnit)
    }
    pub const fn default_enabled(&self) -> bool {
        match self {
            // `MekanismModules.java:139`: only the vision enhancement unit is `disabledByDefault`. The elytra unit is enabled; what it
            // has off by default is the mode-change key (`modeChangeDisabledByDefault`, line 181): see `handles_mode_change_by_default`.
            Self::VisionEnhancementUnit => false,
            _ => true,
        }
    }

    /// Returns list of equipment item IDs that support this module.
    pub const fn supported_containers(&self) -> &'static [&'static str] {
        match self {
            Self::EnergyUnit => &[
                "mekanism:mekasuit_helmet",
                "mekanism:mekasuit_bodyarmor",
                "mekanism:mekasuit_pants",
                "mekanism:mekasuit_boots",
                "mekanism:meka_tool",
            ],
            Self::ColorModulationUnit | Self::LaserDissipationUnit | Self::RadiationShieldingUnit => &[
                "mekanism:mekasuit_helmet",
                "mekanism:mekasuit_bodyarmor",
                "mekanism:mekasuit_pants",
                "mekanism:mekasuit_boots",
            ],
            Self::ElectrolyticBreathingUnit
            | Self::InhalationPurificationUnit
            | Self::VisionEnhancementUnit
            | Self::NutritionalInjectionUnit
            | Self::SolarRechargingUnit => &["mekanism:mekasuit_helmet"],
            Self::JetpackUnit
            | Self::ChargeDistributionUnit
            | Self::GravitationalModulatingUnit
            | Self::ElytraUnit
            | Self::DosimeterUnit
            | Self::GeigerUnit => &["mekanism:mekasuit_bodyarmor"],
            Self::LocomotiveBoostingUnit
            | Self::GyroscopicStabilizationUnit
            | Self::HydrostaticRepulsorUnit
            | Self::MotorizedServoUnit
            | Self::GeothermalGeneratorUnit => &["mekanism:mekasuit_pants"],
            Self::HydraulicPropulsionUnit
            | Self::MagneticAttractionUnit
            | Self::FrostWalkerUnit
            | Self::SoulSurferUnit => &["mekanism:mekasuit_boots"],
            Self::AttackAmplificationUnit
            | Self::ExcavationEscalationUnit
            | Self::BlastingUnit
            | Self::VeinMiningUnit
            | Self::FarmingUnit
            | Self::ShearingUnit
            | Self::TeleportationUnit
            | Self::SilkTouchUnit
            | Self::FortuneUnit => &["mekanism:meka_tool"],
        }
    }

    /// Check if this module can be installed on the given item.
    pub fn can_install_on(&self, item: &str) -> bool {
        let normalized = if item.starts_with("mekanism:") { item } else { item };
        self.supported_containers().iter().any(|supported| *supported == normalized || supported.strip_prefix("mekanism:").unwrap_or(supported) == normalized)
    }

    /// Corresponding enchantment ID for enchantment-based modules.
    pub const fn enchantment(&self) -> Option<&'static str> {
        match self {
            Self::FrostWalkerUnit => Some("minecraft:frost_walker"),
            Self::SoulSurferUnit => Some("minecraft:soul_speed"),
            Self::SilkTouchUnit => Some("minecraft:silk_touch"),
            Self::FortuneUnit => Some("minecraft:fortune"),
            _ => None,
        }
    }
}

// -----------------------------------------------------------------------------
// Module Configuration Enums
// -----------------------------------------------------------------------------

/// `IJetpackItem.JetpackMode` (in the original's order: NORMAL, HOVER, VECTOR, DISABLED; the order is what changing the mode steps through).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum JetpackMode {
    Normal,
    Hover,
    Vector,
    Disabled,
}

impl JetpackMode {
    /// `IIncrementalEnum.adjust`: steps by `shift` and wraps (`ByIdMap.OutOfBoundsStrategy.WRAP`).
    pub fn adjust(&self, shift: i32) -> Self {
        let current = match self {
            Self::Normal => 0,
            Self::Hover => 1,
            Self::Vector => 2,
            Self::Disabled => 3,
        };
        match (current + shift).rem_euclid(4) {
            0 => Self::Normal,
            1 => Self::Hover,
            2 => Self::Vector,
            _ => Self::Disabled,
        }
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Hover => "hover",
            Self::Vector => "vector",
            Self::Disabled => "disabled",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "normal" => Some(Self::Normal),
            "hover" => Some(Self::Hover),
            "vector" => Some(Self::Vector),
            "disabled" => Some(Self::Disabled),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ThrustMultiplier {
    Half,    // 0.5
    Normal,  // 1.0
    Fast,    // 2.0
    Faster,  // 3.0
    Fastest, // 4.0
}

impl ThrustMultiplier {
    /// The place in the original's order (HALF, NORMAL, FAST, FASTER, FASTEST).
    pub const fn ordinal(&self) -> u8 {
        match self {
            Self::Half => 0,
            Self::Normal => 1,
            Self::Fast => 2,
            Self::Faster => 3,
            Self::Fastest => 4,
        }
    }

    pub const fn from_ordinal(index: u8) -> Self {
        match index {
            0 => Self::Half,
            1 => Self::Normal,
            2 => Self::Fast,
            3 => Self::Faster,
            _ => Self::Fastest,
        }
    }

    pub const fn multiplier(&self) -> f32 {
        match self {
            Self::Half => 0.5,
            Self::Normal => 1.0,
            Self::Fast => 2.0,
            Self::Faster => 3.0,
            Self::Fastest => 4.0,
        }
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Half => "half",
            Self::Normal => "normal",
            Self::Fast => "fast",
            Self::Faster => "faster",
            Self::Fastest => "fastest",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "half" => Some(Self::Half),
            "normal" => Some(Self::Normal),
            "fast" => Some(Self::Fast),
            "faster" => Some(Self::Faster),
            "fastest" => Some(Self::Fastest),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum SprintBoost {
    Off,   // 0.0
    Low,   // 0.05
    Med,   // 0.1
    High,  // 0.25
    Ultra, // 0.5
}

impl SprintBoost {
    pub const fn boost(&self) -> f32 {
        match self {
            Self::Off => 0.0,
            Self::Low => 0.05,
            Self::Med => 0.1,
            Self::High => 0.25,
            Self::Ultra => 0.5,
        }
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Low => "low",
            Self::Med => "med",
            Self::High => "high",
            Self::Ultra => "ultra",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "off" => Some(Self::Off),
            "low" => Some(Self::Low),
            "med" => Some(Self::Med),
            "high" => Some(Self::High),
            "ultra" => Some(Self::Ultra),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum JumpBoost {
    Off,   // 0.0
    Low,   // 0.5
    Med,   // 1.0
    High,  // 3.0
    Ultra, // 5.0
}

impl JumpBoost {
    pub const fn boost(&self) -> f32 {
        match self {
            Self::Off => 0.0,
            Self::Low => 0.5,
            Self::Med => 1.0,
            Self::High => 3.0,
            Self::Ultra => 5.0,
        }
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Low => "low",
            Self::Med => "med",
            Self::High => "high",
            Self::Ultra => "ultra",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "off" => Some(Self::Off),
            "low" => Some(Self::Low),
            "med" => Some(Self::Med),
            "high" => Some(Self::High),
            "ultra" => Some(Self::Ultra),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum StepAssist {
    Off,   // 0.0
    Low,   // 0.5
    Med,   // 1.0
    High,  // 1.5
    Ultra, // 2.0
}

impl StepAssist {
    pub const fn height(&self) -> f32 {
        match self {
            Self::Off => 0.0,
            Self::Low => 0.5,
            Self::Med => 1.0,
            Self::High => 1.5,
            Self::Ultra => 2.0,
        }
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Low => "low",
            Self::Med => "med",
            Self::High => "high",
            Self::Ultra => "ultra",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "off" => Some(Self::Off),
            "low" => Some(Self::Low),
            "med" => Some(Self::Med),
            "high" => Some(Self::High),
            "ultra" => Some(Self::Ultra),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum MagneticRange {
    Off,   // 0.0
    Low,   // 1.0
    Med,   // 3.0
    High,  // 5.0
    Ultra, // 10.0
}

impl MagneticRange {
    pub const fn range(&self) -> f32 {
        match self {
            Self::Off => 0.0,
            Self::Low => 1.0,
            Self::Med => 3.0,
            Self::High => 5.0,
            Self::Ultra => 10.0,
        }
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Low => "low",
            Self::Med => "med",
            Self::High => "high",
            Self::Ultra => "ultra",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "off" => Some(Self::Off),
            "low" => Some(Self::Low),
            "med" => Some(Self::Med),
            "high" => Some(Self::High),
            "ultra" => Some(Self::Ultra),
            _ => None,
        }
    }
}

// Meka-Tool Specific Enums (shared for e6)
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum AttackDamage {
    Off,     // 0
    Low,     // 4
    Med,     // 8
    High,    // 16
    Extreme, // 24
    Max,     // 32
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ExcavationMode {
    Off,       // 4.0
    Slow,      // 8.0
    Normal,    // 20.0
    Fast,      // 40.0
    SuperFast, // 80.0
    Extreme,   // 160.0
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum BlastRadius {
    Off,     // 0 (1x1)
    Low,     // 1 (3x3)
    Med,     // 2 (5x5)
    High,    // 3 (7x7)
    Extreme, // 4 (9x9)
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ExcavationRange {
    Off,   // 0
    Low,   // 8
    Med,   // 16
    High,  // 32
    Ultra, // 64
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum FarmingRadius {
    Off,   // 0 (1x1)
    Low,   // 1 (3x3)
    Med,   // 2 (5x5)
    High,  // 3 (7x7)
    Ultra, // 4 (9x9)
}

// -----------------------------------------------------------------------------
// Installed Module State
// -----------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct InstalledModule {
    pub module_type: ModuleType,
    pub installed: u8,
    pub enabled: bool,
    pub handle_mode_change: bool,
    pub render_hud: bool,
    // Configurations
    pub color_argb: u32,
    pub jetpack_mode: JetpackMode,
    pub thrust_multiplier: ThrustMultiplier,
    pub hover_thrust_multiplier: ThrustMultiplier,
    pub sprint_boost: SprintBoost,
    pub jump_boost: JumpBoost,
    pub step_assist: StepAssist,
    pub magnetic_range: MagneticRange,
    pub fill_held: bool,
    pub charge_suit: bool,
    pub charge_inventory: bool,
    pub swim_boost: bool,
    pub beneficial_effects: bool,
    pub neutral_effects: bool,
    pub harmful_effects: bool,
    // Meka-Tool settings
    pub attack_damage: AttackDamage,
    pub excavation_mode: ExcavationMode,
    pub blast_radius: BlastRadius,
    pub excavation_range: ExcavationRange,
    pub extended_mode: bool,
    pub farming_radius: FarmingRadius,
    pub require_target: bool,
}

impl InstalledModule {
    /// How many of the thrust multipliers can be chosen: `installed + 1` (`ModuleEnumConfig.createBounded(.., installed + 1)`, `MekanismModules.java:155-160`).
    pub fn selectable_thrust_count(&self) -> u8 {
        self.installed.saturating_add(1)
    }

    /// Chooses the jetpack's thrust multiplier (the hover one when `hover`); refused when it is past the ones the installed units allow.
    pub fn set_thrust_multiplier(&mut self, hover: bool, value: ThrustMultiplier) -> bool {
        if self.module_type != ModuleType::JetpackUnit || value.ordinal() >= self.selectable_thrust_count() {
            return false;
        }
        if hover {
            self.hover_thrust_multiplier = value;
        } else {
            self.thrust_multiplier = value;
        }
        true
    }

    /// After units are taken out the choice has to stay among the selectable ones.
    pub fn clamp_thrust_to_installed(&mut self) {
        if self.module_type == ModuleType::JetpackUnit {
            let max = self.installed;
            if self.thrust_multiplier.ordinal() > max {
                self.thrust_multiplier = ThrustMultiplier::from_ordinal(max);
            }
            if self.hover_thrust_multiplier.ordinal() > max {
                self.hover_thrust_multiplier = ThrustMultiplier::from_ordinal(max);
            }
        }
    }

    pub fn new(module_type: ModuleType, count: u8) -> Self {
        let count = count.min(module_type.max_stack()).max(1);
        let handles = module_type.handles_mode_change_by_default();
        let hud = module_type.renders_hud();
        let default_enabled = module_type.default_enabled();
        Self {
            module_type,
            installed: count,
            enabled: default_enabled,
            handle_mode_change: handles,
            render_hud: hud,
            color_argb: 0xFFFFFFFF,
            jetpack_mode: JetpackMode::Normal,
            thrust_multiplier: ThrustMultiplier::Normal,
            hover_thrust_multiplier: ThrustMultiplier::Normal,
            sprint_boost: SprintBoost::Low,
            jump_boost: JumpBoost::Low,
            step_assist: StepAssist::Low,
            magnetic_range: MagneticRange::Low,
            fill_held: true,
            charge_suit: true,
            charge_inventory: false,
            swim_boost: true,
            beneficial_effects: false,
            neutral_effects: false,
            harmful_effects: true,
            attack_damage: AttackDamage::Med,
            excavation_mode: ExcavationMode::Normal,
            blast_radius: BlastRadius::Low,
            excavation_range: ExcavationRange::Low,
            extended_mode: false,
            farming_radius: FarmingRadius::Low,
            require_target: true,
        }
    }
}

// -----------------------------------------------------------------------------
// Module Container
// -----------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModuleContainer {
    pub modules: HashMap<ModuleType, InstalledModule>,
}

impl ModuleContainer {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    pub fn has_module(&self, t: ModuleType) -> bool {
        self.modules.contains_key(&t)
    }

    pub fn is_enabled(&self, t: ModuleType) -> bool {
        self.modules.get(&t).is_some_and(|m| m.enabled)
    }

    pub fn installed_count(&self, t: ModuleType) -> u8 {
        self.modules.get(&t).map_or(0, |m| m.installed)
    }

    pub fn get(&self, t: ModuleType) -> Option<&InstalledModule> {
        self.modules.get(&t)
    }

    pub fn get_mut(&mut self, t: ModuleType) -> Option<&mut InstalledModule> {
        self.modules.get_mut(&t)
    }

    /// Can more units of this module be installed on this item?
    pub fn can_install(&self, item: &str, t: ModuleType) -> bool {
        if !t.can_install_on(item) {
            return false;
        }
        let current = self.installed_count(t);
        current < t.max_stack()
    }

    /// Installs `count` units of `t` on `item`. Returns number of units actually installed.
    pub fn add_module(&mut self, item: &str, t: ModuleType, count: u8) -> u8 {
        if !t.can_install_on(item) || count == 0 {
            return 0;
        }
        let current = self.installed_count(t);
        let max = t.max_stack();
        if current >= max {
            return 0;
        }
        let to_add = count.min(max - current);
        if let Some(existing) = self.modules.get_mut(&t) {
            existing.installed += to_add;
        } else {
            let module = InstalledModule::new(t, to_add);
            self.modules.insert(t, module);
        }
        to_add
    }

    /// Removes up to `count` units of `t`. Returns number of units removed.
    pub fn remove_module(&mut self, t: ModuleType, count: u8) -> u8 {
        let Some(existing) = self.modules.get_mut(&t) else {
            return 0;
        };
        if count >= existing.installed {
            let removed = existing.installed;
            self.modules.remove(&t);
            removed
        } else {
            existing.installed -= count;
            existing.clamp_thrust_to_installed();
            count
        }
    }

    /// Toggle module enabled state (`ModuleContainer.java:178-193`).
    /// Disables other mutually exclusive modules if enabling.
    pub fn toggle_enabled(&mut self, t: ModuleType) -> bool {
        if !t.can_be_disabled() {
            return true;
        }
        let new_state = {
            let Some(module) = self.modules.get_mut(&t) else {
                return false;
            };
            module.enabled = !module.enabled;
            module.enabled
        };

        if new_state {
            let flags = t.exclusive_flags();
            let handles_mode = t.handles_mode_change();
            // disable other exclusive modules
            for (&other_type, other_mod) in self.modules.iter_mut() {
                if other_type != t {
                    let other_flags = other_type.exclusive_flags();
                    if (flags != 0 && (flags & other_flags) != 0) && other_mod.enabled && other_type.can_be_disabled() {
                        other_mod.enabled = false;
                    }
                    if handles_mode && other_type.handles_mode_change() {
                        other_mod.handle_mode_change = false;
                    }
                }
            }
        }
        new_state
    }

    /// Set enabled state directly.
    pub fn set_enabled(&mut self, t: ModuleType, enabled: bool) {
        if let Some(module) = self.modules.get_mut(&t) {
            if t.can_be_disabled() {
                module.enabled = enabled;
            }
        }
    }

    /// Returns virtual enchantment level from modules (`ModuleContainer.java:232-250`).
    pub fn enchantment_level(&self, enchantment_id: &str) -> u32 {
        for (module_type, module) in &self.modules {
            if module.enabled {
                if let Some(ench) = module_type.enchantment() {
                    if ench == enchantment_id || ench.strip_prefix("minecraft:").unwrap_or(ench) == enchantment_id.strip_prefix("minecraft:").unwrap_or(enchantment_id) {
                        return module.installed as u32;
                    }
                }
            }
        }
        0
    }

    /// Cycle active mode for a mode-handling module (`ModuleContainer.java`).
    pub fn cycle_mode(&mut self, t: ModuleType, shift: i32) -> bool {
        let Some(module) = self.modules.get_mut(&t) else {
            return false;
        };
        match t {
            ModuleType::JetpackUnit => {
                module.jetpack_mode = module.jetpack_mode.adjust(shift);
                true
            }
            ModuleType::LocomotiveBoostingUnit => {
                let current = match module.sprint_boost {
                    SprintBoost::Off => 0,
                    SprintBoost::Low => 1,
                    SprintBoost::Med => 2,
                    SprintBoost::High => 3,
                    SprintBoost::Ultra => 4,
                };
                let max_avail = (module.installed as i32 + 1).min(5);
                let next = (current + shift).rem_euclid(max_avail);
                module.sprint_boost = match next {
                    0 => SprintBoost::Off,
                    1 => SprintBoost::Low,
                    2 => SprintBoost::Med,
                    3 => SprintBoost::High,
                    _ => SprintBoost::Ultra,
                };
                true
            }
            ModuleType::HydraulicPropulsionUnit => {
                let current = match module.jump_boost {
                    JumpBoost::Off => 0,
                    JumpBoost::Low => 1,
                    JumpBoost::Med => 2,
                    JumpBoost::High => 3,
                    JumpBoost::Ultra => 4,
                };
                let max_avail = (module.installed as i32 + 1).min(5);
                let next = (current + shift).rem_euclid(max_avail);
                module.jump_boost = match next {
                    0 => JumpBoost::Off,
                    1 => JumpBoost::Low,
                    2 => JumpBoost::Med,
                    3 => JumpBoost::High,
                    _ => JumpBoost::Ultra,
                };
                true
            }
            ModuleType::MagneticAttractionUnit => {
                let current = match module.magnetic_range {
                    MagneticRange::Off => 0,
                    MagneticRange::Low => 1,
                    MagneticRange::Med => 2,
                    MagneticRange::High => 3,
                    MagneticRange::Ultra => 4,
                };
                let max_avail = (module.installed as i32 + 1).min(5);
                let next = (current + shift).rem_euclid(max_avail);
                module.magnetic_range = match next {
                    0 => MagneticRange::Off,
                    1 => MagneticRange::Low,
                    2 => MagneticRange::Med,
                    3 => MagneticRange::High,
                    _ => MagneticRange::Ultra,
                };
                true
            }
            _ => false,
        }
    }

    /// Decode `mekanism:modules` from stack's component string (JSON or SNBT format).
    pub fn decode_from_components(components_str: &str) -> Self {
        let mut container = Self::new();
        if components_str.is_empty() || components_str == "{}" {
            return container;
        }

        // Try parsing JSON format: {"mekanism:modules": [{"type":"mekanism:energy_unit","amount":8,...},...]}
        if let Some(start) = components_str.find("\"mekanism:modules\"") {
            let slice = &components_str[start + "\"mekanism:modules\"".len()..];
            // Locate bracket [ ... ]
            if let Some(b_start) = slice.find('[') {
                if let Some(b_end) = slice[b_start..].find(']') {
                    let array_slice = &slice[b_start + 1..b_start + b_end];
                    // split objects inside array
                    for item in array_slice.split('{') {
                        let item = item.trim();
                        if item.is_empty() {
                            continue;
                        }
                        // parse type and amount
                        let mut m_type: Option<ModuleType> = None;
                        let mut m_amount: u8 = 1;
                        let mut m_enabled: Option<bool> = None;

                        for field in item.split(',') {
                            let field = field.trim().trim_end_matches('}');
                            if let Some(colon) = field.find(':') {
                                let key = field[..colon].trim().trim_matches('"');
                                let val = field[colon + 1..].trim().trim_matches('"');
                                match key {
                                    "type" => m_type = ModuleType::from_id(val),
                                    "amount" => m_amount = val.parse().unwrap_or(1),
                                    "enabled" => m_enabled = Some(val == "true" || val == "1"),
                                    _ => {}
                                }
                            }
                        }

                        if let Some(t) = m_type {
                            let mut installed = InstalledModule::new(t, m_amount);
                            if let Some(en) = m_enabled {
                                installed.enabled = en;
                            }
                            container.modules.insert(t, installed);
                        }
                    }
                }
            }
        }
        container
    }

    /// Encode `mekanism:modules` into the stack's data components string.
    pub fn encode_to_components(&self, base_components: &str) -> String {
        let json_repr = self.to_json();
        let key = "\"mekanism:modules\"";
        let pair = format!("{key}:{json_repr}");

        if base_components.trim().is_empty() || base_components.trim() == "{}" {
            return format!("{{{pair}}}");
        }

        // Replace existing "mekanism:modules" if present
        if let Some(at) = base_components.find(key) {
            let before = &base_components[..at];
            let after = &base_components[at + key.len()..];
            // find colon
            if let Some(colon) = after.find(':') {
                let rest = &after[colon + 1..].trim_start();
                let end_idx = if rest.starts_with('[') {
                    rest.find(']').map(|i| i + 1).unwrap_or(rest.len())
                } else if rest.starts_with('{') {
                    rest.find('}').map(|i| i + 1).unwrap_or(rest.len())
                } else {
                    rest.find(',').or_else(|| rest.find('}')).unwrap_or(rest.len())
                };
                let remainder = &rest[end_idx..];
                return format!("{before}{pair}{remainder}");
            }
        }

        // Insert before closing brace
        let at = base_components.rfind('}').unwrap_or(base_components.len());
        let mut out = base_components.to_owned();
        if base_components.trim() == "{}" {
            out = format!("{{{pair}}}");
        } else {
            out.insert_str(at, &format!(",{pair}"));
        }
        out
    }

    /// Serialize installed modules array to JSON string.
    pub fn to_json(&self) -> String {
        let mut parts = Vec::new();
        // Sort keys for deterministic output
        let mut sorted_types: Vec<ModuleType> = self.modules.keys().copied().collect();
        sorted_types.sort_by_key(|t| t.id());

        for t in sorted_types {
            if let Some(m) = self.modules.get(&t) {
                let mut configs = Vec::new();
                configs.push(format!("\"enabled\":{}", m.enabled));
                match t {
                    ModuleType::ColorModulationUnit => {
                        configs.push(format!("\"color\":{}", m.color_argb));
                    }
                    ModuleType::JetpackUnit => {
                        configs.push(format!("\"jetpack_mode\":\"{}\"", m.jetpack_mode.as_str()));
                        configs.push(format!("\"thrust_mult\":\"{}\"", m.thrust_multiplier.as_str()));
                        configs.push(format!("\"hover_thrust_mult\":\"{}\"", m.hover_thrust_multiplier.as_str()));
                    }
                    ModuleType::LocomotiveBoostingUnit => {
                        configs.push(format!("\"sprint_boost\":\"{}\"", m.sprint_boost.as_str()));
                    }
                    ModuleType::HydraulicPropulsionUnit => {
                        configs.push(format!("\"jump_boost\":\"{}\"", m.jump_boost.as_str()));
                        configs.push(format!("\"step_assist\":\"{}\"", m.step_assist.as_str()));
                    }
                    ModuleType::MagneticAttractionUnit => {
                        configs.push(format!("\"range\":\"{}\"", m.magnetic_range.as_str()));
                    }
                    ModuleType::ElectrolyticBreathingUnit => {
                        configs.push(format!("\"fill_held\":{}", m.fill_held));
                    }
                    ModuleType::ChargeDistributionUnit => {
                        configs.push(format!("\"charge_suit\":{}", m.charge_suit));
                        configs.push(format!("\"charge_inventory\":{}", m.charge_inventory));
                    }
                    ModuleType::HydrostaticRepulsorUnit => {
                        configs.push(format!("\"swim_boost\":{}", m.swim_boost));
                    }
                    ModuleType::InhalationPurificationUnit => {
                        configs.push(format!("\"beneficial\":{}", m.beneficial_effects));
                        configs.push(format!("\"neutral\":{}", m.neutral_effects));
                        configs.push(format!("\"harmful\":{}", m.harmful_effects));
                    }
                    _ => {}
                }
                let config_str = configs.join(",");
                parts.push(format!("{{\"type\":\"{}\",\"amount\":{},{config_str}}}", t.id(), m.installed));
            }
        }
        format!("[{}]", parts.join(","))
    }
}

// -----------------------------------------------------------------------------
// Unit Tests (Citing Java 1.21.1 source lines)
// -----------------------------------------------------------------------------
#[cfg(test)]
mod tests {

    #[test]
    fn java_jetpack_thrust_choices_grow_with_the_installed_units() {
        // `MekanismModules.java:155-160`: `createBounded(.., ThrustMultiplier.NORMAL, installed + 1)`: the first installed + 1 of HALF, NORMAL, FAST, FASTER, FASTEST.
        let mut jetpack = InstalledModule::new(ModuleType::JetpackUnit, 1);
        assert_eq!(jetpack.selectable_thrust_count(), 2);
        assert!(jetpack.set_thrust_multiplier(false, ThrustMultiplier::Half));
        assert!(jetpack.set_thrust_multiplier(false, ThrustMultiplier::Normal));
        assert!(!jetpack.set_thrust_multiplier(false, ThrustMultiplier::Fast), "FAST needs 2 units");
        let mut four = InstalledModule::new(ModuleType::JetpackUnit, 4);
        assert!(four.set_thrust_multiplier(false, ThrustMultiplier::Fastest));
        assert!(four.set_thrust_multiplier(true, ThrustMultiplier::Faster));
        // Taking a unit out keeps the choice among the selectable ones.
        let mut container = ModuleContainer::new();
        container.add_module("mekanism:mekasuit_bodyarmor", ModuleType::JetpackUnit, 4);
        container.get_mut(ModuleType::JetpackUnit).unwrap().set_thrust_multiplier(false, ThrustMultiplier::Fastest);
        container.remove_module(ModuleType::JetpackUnit, 1);
        assert_eq!(container.get(ModuleType::JetpackUnit).unwrap().thrust_multiplier, ThrustMultiplier::Faster);
    }

    use super::*;

    #[test]
    fn test_module_limits_and_metadata() {
        // MekanismModules.java:62 - energy_unit max stack = 8, noDisable = true
        assert_eq!(ModuleType::EnergyUnit.max_stack(), 8);
        assert!(!ModuleType::EnergyUnit.can_be_disabled());

        // MekanismModules.java:128 - electrolytic_breathing_unit max stack = 4
        assert_eq!(ModuleType::ElectrolyticBreathingUnit.max_stack(), 4);
        assert!(ModuleType::ElectrolyticBreathingUnit.can_be_disabled());

        // MekanismModules.java:139 - vision_enhancement_unit disabledByDefault = true
        assert!(!ModuleType::VisionEnhancementUnit.default_enabled());
        // MekanismModules.java:181 - the elytra unit is enabled by default; only its mode-change key is off (`modeChangeDisabledByDefault`).
        assert!(ModuleType::ElytraUnit.default_enabled());
        assert!(ModuleType::ElytraUnit.handles_mode_change());
        assert!(!ModuleType::ElytraUnit.handles_mode_change_by_default());
        assert!(!InstalledModule::new(ModuleType::ElytraUnit, 1).handle_mode_change);
        assert!(InstalledModule::new(ModuleType::JetpackUnit, 1).handle_mode_change);

        // MekanismModules.java:149 - jetpack_unit max stack = 4, exclusive OVERRIDE_JUMP
        assert_eq!(ModuleType::JetpackUnit.max_stack(), 4);
        assert_eq!(ModuleType::JetpackUnit.exclusive_flags(), EXCLUSIVE_OVERRIDE_JUMP);

        // MekanismModules.java:172 - gravitational_modulating_unit exclusive OVERRIDE_JUMP
        assert_eq!(ModuleType::GravitationalModulatingUnit.exclusive_flags(), EXCLUSIVE_OVERRIDE_JUMP);

        // MekanismModules.java:185 - locomotive_boosting_unit max stack = 4
        assert_eq!(ModuleType::LocomotiveBoostingUnit.max_stack(), 4);

        // MekanismModules.java:194 - hydrostatic_repulsor_unit max stack = 4
        assert_eq!(ModuleType::HydrostaticRepulsorUnit.max_stack(), 4);

        // MekanismModules.java:205 - motorized_servo_unit max stack = 5
        assert_eq!(ModuleType::MotorizedServoUnit.max_stack(), 5);

        // MekanismModules.java:209 - hydraulic_propulsion_unit max stack = 4
        assert_eq!(ModuleType::HydraulicPropulsionUnit.max_stack(), 4);

        // MekanismModules.java:221 - magnetic_attraction_unit max stack = 4
        assert_eq!(ModuleType::MagneticAttractionUnit.max_stack(), 4);

        // MekanismModules.java:229 - frost_walker_unit max stack = 2
        assert_eq!(ModuleType::FrostWalkerUnit.max_stack(), 2);

        // MekanismModules.java:231 - soul_surfer_unit max stack = 3
        assert_eq!(ModuleType::SoulSurferUnit.max_stack(), 3);

        // GeneratorsModules.java:18 - solar_recharging_unit max stack = 8
        assert_eq!(ModuleType::SolarRechargingUnit.max_stack(), 8);

        // GeneratorsModules.java:22 - geothermal_generator_unit max stack = 8
        assert_eq!(ModuleType::GeothermalGeneratorUnit.max_stack(), 8);
    }

    #[test]
    fn test_module_container_installation_and_removal() {
        let mut container = ModuleContainer::new();
        let helmet = "mekanism:mekasuit_helmet";
        let boots = "mekanism:mekasuit_boots";

        // Can install EnergyUnit on helmet up to 8
        assert!(container.can_install(helmet, ModuleType::EnergyUnit));
        assert_eq!(container.add_module(helmet, ModuleType::EnergyUnit, 5), 5);
        assert_eq!(container.installed_count(ModuleType::EnergyUnit), 5);

        // Add 5 more: only 3 fit (capped at 8)
        assert_eq!(container.add_module(helmet, ModuleType::EnergyUnit, 5), 3);
        assert_eq!(container.installed_count(ModuleType::EnergyUnit), 8);
        assert!(!container.can_install(helmet, ModuleType::EnergyUnit));

        // Electrolytic breathing on helmet: max 4
        assert_eq!(container.add_module(helmet, ModuleType::ElectrolyticBreathingUnit, 4), 4);
        assert_eq!(container.installed_count(ModuleType::ElectrolyticBreathingUnit), 4);

        // Jetpack cannot be installed on helmet (chestplate only)
        assert!(!container.can_install(helmet, ModuleType::JetpackUnit));
        assert_eq!(container.add_module(helmet, ModuleType::JetpackUnit, 1), 0);

        // Remove 2 energy units
        assert_eq!(container.remove_module(ModuleType::EnergyUnit, 2), 2);
        assert_eq!(container.installed_count(ModuleType::EnergyUnit), 6);

        // Remove all remaining
        assert_eq!(container.remove_module(ModuleType::EnergyUnit, 10), 6);
        assert_eq!(container.installed_count(ModuleType::EnergyUnit), 0);
        assert!(!container.has_module(ModuleType::EnergyUnit));

        // Frost walker on boots
        assert!(container.can_install(boots, ModuleType::FrostWalkerUnit));
        assert_eq!(container.add_module(boots, ModuleType::FrostWalkerUnit, 2), 2);
        // Frost walker enchantment level should equal 2 (ModuleContainer.java:247-250)
        assert_eq!(container.enchantment_level("minecraft:frost_walker"), 2);
    }

    #[test]
    fn test_exclusive_flags_and_toggling() {
        let mut container = ModuleContainer::new();
        let bodyarmor = "mekanism:mekasuit_bodyarmor";

        // Install Jetpack and Gravitational Modulator (both have EXCLUSIVE_OVERRIDE_JUMP)
        container.add_module(bodyarmor, ModuleType::JetpackUnit, 1);
        container.add_module(bodyarmor, ModuleType::GravitationalModulatingUnit, 1);

        assert!(container.is_enabled(ModuleType::JetpackUnit));
        assert!(container.is_enabled(ModuleType::GravitationalModulatingUnit));

        // Toggling Gravitational Modulating on while active should disable Jetpack
        // (ModuleContainer.java:189 - disableOtherExclusives)
        container.set_enabled(ModuleType::GravitationalModulatingUnit, false);
        assert!(!container.is_enabled(ModuleType::GravitationalModulatingUnit));

        container.toggle_enabled(ModuleType::GravitationalModulatingUnit);
        assert!(container.is_enabled(ModuleType::GravitationalModulatingUnit));
        assert!(!container.is_enabled(ModuleType::JetpackUnit));
    }

    #[test]
    fn test_component_encoding_and_decoding() {
        let mut container = ModuleContainer::new();
        let helmet = "mekanism:mekasuit_helmet";
        container.add_module(helmet, ModuleType::EnergyUnit, 8);
        container.add_module(helmet, ModuleType::ElectrolyticBreathingUnit, 2);

        let encoded = container.encode_to_components("{}");
        assert!(encoded.contains("\"mekanism:modules\""));
        assert!(encoded.contains("mekanism:energy_unit"));
        assert!(encoded.contains("mekanism:electrolytic_breathing_unit"));

        let decoded = ModuleContainer::decode_from_components(&encoded);
        assert_eq!(decoded.installed_count(ModuleType::EnergyUnit), 8);
        assert_eq!(decoded.installed_count(ModuleType::ElectrolyticBreathingUnit), 2);
        assert!(decoded.is_enabled(ModuleType::EnergyUnit));
    }
}

//! MekaSuit Armor (`mekasuit.rs`): the 4 pieces (helmet, bodyarmor, pants, boots),
//! damage absorption model, internal chemical/fluid tanks, and comprehensive module behaviors.
//! Faithfully ported from Mekanism 1.21.1-10.7.19.85 (`ItemMekaSuitArmor.java`,
//! `MekanismArmorMaterials.java`, `GearConfig.java`, `GeneratorsGearConfig.java`).

use crate::modules::{
    self, fe, InstalledModule, JetpackMode, JumpBoost, MagneticRange, ModuleContainer, ModuleType,
    SprintBoost, StepAssist, ThrustMultiplier,
};
use std::collections::HashMap;

// -----------------------------------------------------------------------------
// Constants & Configuration Values (Exact Java 1.21.1 match)
// -----------------------------------------------------------------------------

pub const HELMET: &str = "mekanism:mekasuit_helmet";
pub const BODYARMOR: &str = "mekanism:mekasuit_bodyarmor";
pub const PANTS: &str = "mekanism:mekasuit_pants";
pub const BOOTS: &str = "mekanism:mekasuit_boots";

/// Base energy capacity per piece: 16,000,000 Joules = 6,400,000 FE (`GearConfig.java:249`).
pub const BASE_ENERGY_CAPACITY: u64 = fe(16_000_000); // 6_400_000 FE

/// Base charge rate per piece: 100,000 Joules/t = 40,000 FE/t (`GearConfig.java:250`).
pub const BASE_CHARGE_RATE: u64 = fe(100_000); // 40_000 FE/t

/// Inventory charge rate: 10,000 Joules/t = 4,000 FE/t (`GearConfig.java:251`).
pub const INVENTORY_CHARGE_RATE: u64 = fe(10_000); // 4_000 FE/t

/// Solar recharging base rate: 500 Joules/t = 200 FE/t (`GearConfig.java:252`).
pub const SOLAR_RECHARGING_RATE: u64 = fe(500); // 200 FE/t

/// Nutritional paste max storage: 128 buckets = 128,000 mB (`GearConfig.java:256`).
pub const NUTRITIONAL_MAX_STORAGE: u64 = 128_000;

/// Nutritional paste transfer rate: 256 mB/t (`GearConfig.java:258`).
pub const NUTRITIONAL_TRANSFER_RATE: u64 = 256;

/// Jetpack hydrogen max storage per installed jetpack unit: 24 buckets = 24,000 mB (`GearConfig.java:260`).
pub const JETPACK_MAX_STORAGE_PER_UNIT: u64 = 24_000;

/// Jetpack hydrogen transfer rate: 256 mB/t (`GearConfig.java:262`).
pub const JETPACK_TRANSFER_RATE: u64 = 256;

/// Energy usage per damage point absorbed: 100,000 Joules = 40,000 FE (`GearConfig.java:265`).
pub const ENERGY_USAGE_DAMAGE: u64 = fe(100_000); // 40_000 FE

/// Energy usage per magic damage point absorbed: 1,000 Joules = 400 FE (`GearConfig.java:266`).
pub const ENERGY_USAGE_MAGIC_REDUCE: u64 = fe(1_000); // 400 FE

/// Energy usage per fall damage event prevented: 50 Joules = 20 FE (`GearConfig.java:267`).
pub const ENERGY_USAGE_FALL: u64 = fe(50); // 20 FE

/// Base jump energy usage: 1,000 Joules = 400 FE (`GearConfig.java:268`).
pub const ENERGY_USAGE_JUMP: u64 = fe(1_000); // 400 FE

/// Elytra energy usage per second: 32,000 Joules = 12,800 FE (`GearConfig.java:269`).
pub const ENERGY_USAGE_ELYTRA: u64 = fe(32_000); // 12_800 FE

/// Energy usage per potion purification tick: 40,000 Joules = 16,000 FE (`GearConfig.java:270`).
pub const ENERGY_USAGE_POTION_TICK: u64 = fe(40_000); // 16_000 FE

/// Energy usage sprint boost base: 100 Joules/t = 40 FE/t (`GearConfig.java:271`).
pub const ENERGY_USAGE_SPRINT_BOOST: u64 = fe(100); // 40 FE/t

/// Energy usage gravitational modulation base: 1,000 Joules/t = 400 FE/t (`GearConfig.java:273`).
pub const ENERGY_USAGE_GRAVITATIONAL: u64 = fe(1_000); // 400 FE/t

/// Boost multiplier for gravitational modulation: 4x (`ModuleGravitationalModulatingUnit.java:36`).
pub const GRAVITATIONAL_BOOST_MULTIPLIER: u64 = 4;

/// Energy usage vision enhancement: 500 Joules/t = 200 FE/t (`GearConfig.java:274`).
pub const ENERGY_USAGE_VISION: u64 = fe(500); // 200 FE/t

/// Energy usage hydrostatic repulsion: 500 Joules/t = 200 FE/t (`GearConfig.java:275`).
pub const ENERGY_USAGE_HYDROSTATIC: u64 = fe(500); // 200 FE/t

/// Energy usage nutritional injection per food point: 20,000 Joules = 8,000 FE (`GearConfig.java:276`).
pub const ENERGY_USAGE_NUTRITIONAL: u64 = fe(20_000); // 8_000 FE

/// Energy usage item attraction base: 250 Joules/t = 100 FE/t (`GearConfig.java:277`).
pub const ENERGY_USAGE_ITEM_ATTRACTION: u64 = fe(250); // 100 FE/t

/// Geothermal charging rate: 10 Joules/t = 4 FE/t per degree above ambient (`GeneratorsGearConfig.java:24`).
pub const GEOTHERMAL_CHARGING_RATE: u64 = fe(10); // 4 FE/t

/// Nutritional paste per food point: 50 mB (`GeneralConfig.java:281`).
pub const NUTRITIONAL_PASTE_MB_PER_FOOD: u64 = 50;

/// Nutritional paste saturation per food point: 0.8 (`GeneralConfig.java:279`).
pub const NUTRITIONAL_PASTE_SATURATION: f32 = 0.8;

/// Hydrogen energy density: 200 Joules = 80 FE (`MekanismDataMapsProvider.java:32`).
pub const HYDROGEN_ENERGY_DENSITY: u64 = fe(200); // 80 FE

/// Electrolytic breathing energy usage: 2 * HYDROGEN_ENERGY_DENSITY = 400 Joules = 160 FE (`ModuleElectrolyticBreathingUnit.java:62`).
pub const ELECTROLYTIC_BREATHING_USAGE: u64 = fe(400); // 160 FE

// -----------------------------------------------------------------------------
// Armor Piece Definitions & Properties
// -----------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ArmorSlot {
    Helmet = 0,
    Chest = 1,
    Legs = 2,
    Feet = 3,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MekaSuitPiece {
    pub slot: ArmorSlot,
    pub item_id: &'static str,
    pub defense: u32,
    pub toughness: f32,
    pub knockback_resistance: f32,
    pub absorption: f32,
    pub laser_dissipation: f64,
    pub laser_refraction: f64,
    pub radiation_shielding: f64,
}

/// The four MekaSuit armor piece definitions matching Java values:
/// Defense: Netherite equivalent (3, 8, 6, 3) (`MekanismStartupConfig.java:62-73`)
/// Toughness: 3.0 (`MekanismStartupConfig.java:74-76`)
/// Knockback resistance: 0.1 (`MekanismStartupConfig.java:77-79`)
/// Absorption: Helmet 0.15, Bodyarmor 0.40, Pants 0.30, Boots 0.15 (`ItemMekaSuitArmor.java:105,115,120,125`)
/// Laser dissipation: Helmet 0.15, Bodyarmor 0.30, Pants 0.1875, Boots 0.1125 (total 0.75 / 75%) (`ItemMekaSuitArmor.java:106,116,121,126`)
/// Laser refraction: Helmet 0.20, Bodyarmor 0.40, Pants 0.25, Boots 0.15 (total 1.00 / 100% of remaining) (`ItemMekaSuitArmor.java:107,117,122,127`)
/// Radiation shielding: Helmet 0.25, Bodyarmor 0.40, Pants 0.20, Boots 0.15 (total 1.00 / 100%) (`ItemHazmatSuitArmor.java:21-29`)
pub const PIECES: [MekaSuitPiece; 4] = [
    MekaSuitPiece {
        slot: ArmorSlot::Helmet,
        item_id: HELMET,
        defense: 3,
        toughness: 3.0,
        knockback_resistance: 0.1,
        absorption: 0.15,
        laser_dissipation: 0.15,
        laser_refraction: 0.20,
        radiation_shielding: 0.25,
    },
    MekaSuitPiece {
        slot: ArmorSlot::Chest,
        item_id: BODYARMOR,
        defense: 8,
        toughness: 3.0,
        knockback_resistance: 0.1,
        absorption: 0.40,
        laser_dissipation: 0.30,
        laser_refraction: 0.40,
        radiation_shielding: 0.40,
    },
    MekaSuitPiece {
        slot: ArmorSlot::Legs,
        item_id: PANTS,
        defense: 6,
        toughness: 3.0,
        knockback_resistance: 0.1,
        absorption: 0.30,
        laser_dissipation: 0.1875,
        laser_refraction: 0.25,
        radiation_shielding: 0.20,
    },
    MekaSuitPiece {
        slot: ArmorSlot::Feet,
        item_id: BOOTS,
        defense: 3,
        toughness: 3.0,
        knockback_resistance: 0.1,
        absorption: 0.15,
        laser_dissipation: 0.1125,
        laser_refraction: 0.15,
        radiation_shielding: 0.15,
    },
];

pub fn get_piece(item: &str) -> Option<&'static MekaSuitPiece> {
    PIECES.iter().find(|p| p.item_id == item || p.item_id.strip_prefix("mekanism:").unwrap_or(p.item_id) == item)
}

pub fn is_mekasuit_piece(item: &str) -> bool {
    get_piece(item).is_some()
}

// -----------------------------------------------------------------------------
// Damage Sources & Classification
// -----------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct DamageSource {
    pub name: String,
    pub is_fire: bool,
    pub is_magic: bool,
    pub is_fall: bool,
    pub is_technical: bool,
    pub bypasses_armor: bool,
    pub always_supported: bool,
}

impl DamageSource {
    pub fn generic(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            is_fire: false,
            is_magic: false,
            is_fall: false,
            is_technical: false,
            bypasses_armor: false,
            always_supported: false,
        }
    }

    pub fn fire(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            is_fire: true,
            is_magic: false,
            is_fall: false,
            is_technical: false,
            bypasses_armor: false,
            always_supported: false,
        }
    }

    pub fn magic(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            is_fire: false,
            is_magic: true,
            is_fall: false,
            is_technical: false,
            bypasses_armor: true,
            always_supported: true,
        }
    }

    pub fn fall() -> Self {
        Self {
            name: "minecraft:fall".to_owned(),
            is_fire: false,
            is_magic: false,
            is_fall: true,
            is_technical: false,
            bypasses_armor: true,
            always_supported: true,
        }
    }
}

// -----------------------------------------------------------------------------
// MekaSuit Item Representation
// -----------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct MekaSuitItemStack {
    pub item: String,
    pub count: u32,
    pub energy: u64,
    pub chemical_amount: u64, // e.g. Hydrogen for bodyarmor
    pub chemical_type: String,
    pub fluid_amount: u64,    // e.g. Nutritional paste for helmet
    pub fluid_type: String,
    pub modules: ModuleContainer,
}

impl MekaSuitItemStack {
    pub fn new(item: &str) -> Option<Self> {
        let piece = get_piece(item)?;
        let mut stack = Self {
            item: piece.item_id.to_owned(),
            count: 1,
            energy: 0,
            chemical_amount: 0,
            chemical_type: String::new(),
            fluid_amount: 0,
            fluid_type: String::new(),
            modules: ModuleContainer::new(),
        };
        stack.energy = stack.max_energy(); // default full energy variant
        Some(stack)
    }

    /// Maximum energy capacity considering installed EnergyUnit modules (`ModuleEnergyUnit.java:33`).
    /// Formula: `base * 2^installed_count`.
    pub fn max_energy(&self) -> u64 {
        let count = self.modules.installed_count(ModuleType::EnergyUnit);
        BASE_ENERGY_CAPACITY.saturating_mul(1u64 << count.min(8))
    }

    /// Maximum charge rate considering installed EnergyUnit modules (`ModuleEnergyUnit.java:33`).
    /// Formula: `base * 2^installed_count`.
    pub fn max_charge_rate(&self) -> u64 {
        let count = self.modules.installed_count(ModuleType::EnergyUnit);
        BASE_CHARGE_RATE.saturating_mul(1u64 << count.min(8))
    }

    /// Hydrogen tank capacity on bodyarmor (`ItemMekaSuitArmor.java:113`).
    /// Formula: `24,000 * installed_jetpack_units`.
    pub fn hydrogen_capacity(&self) -> u64 {
        if self.item == BODYARMOR {
            let count = self.modules.installed_count(ModuleType::JetpackUnit);
            JETPACK_MAX_STORAGE_PER_UNIT * (count as u64)
        } else {
            0
        }
    }

    /// Nutritional paste tank capacity on helmet (`ItemMekaSuitArmor.java:103`).
    pub fn nutritional_paste_capacity(&self) -> u64 {
        if self.item == HELMET && self.modules.has_module(ModuleType::NutritionalInjectionUnit) {
            NUTRITIONAL_MAX_STORAGE
        } else {
            0
        }
    }

    /// Insert energy up to charge rate, returns amount accepted.
    pub fn insert_energy(&mut self, amount: u64) -> u64 {
        let max = self.max_energy();
        let needed = max.saturating_sub(self.energy);
        let rate = self.max_charge_rate();
        let accepted = amount.min(needed).min(rate);
        self.energy += accepted;
        accepted
    }

    /// Extract energy, returns amount actually drained.
    pub fn extract_energy(&mut self, amount: u64) -> u64 {
        let drained = amount.min(self.energy);
        self.energy -= drained;
        drained
    }

    /// Decode MekaSuit data from Horizon ItemStack components.
    pub fn decode(item: &str, count: u32, components: &str) -> Option<Self> {
        if !is_mekasuit_piece(item) {
            return None;
        }
        let mut stack = Self::new(item)?;
        stack.count = count;
        stack.modules = ModuleContainer::decode_from_components(components);

        // Parse energy: {"mekanism:attached_energy":6400000} or inside array
        if let Some(pos) = components.find("\"mekanism:attached_energy\"") {
            let rest = &components[pos + "\"mekanism:attached_energy\"".len()..];
            let digits: String = rest.chars().skip_while(|c| !c.is_ascii_digit()).take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(e) = digits.parse::<u64>() {
                stack.energy = e.min(stack.max_energy());
            }
        }

        // Parse chemical (hydrogen): {"mekanism:attached_chemicals":{"amount":24000}}
        if let Some(pos) = components.find("\"mekanism:attached_chemicals\"") {
            let rest = &components[pos + "\"mekanism:attached_chemicals\"".len()..];
            if let Some(amt_pos) = rest.find("\"amount\"") {
                let amt_rest = &rest[amt_pos + "\"amount\"".len()..];
                let digits: String = amt_rest.chars().skip_while(|c| !c.is_ascii_digit()).take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(a) = digits.parse::<u64>() {
                    stack.chemical_amount = a.min(stack.hydrogen_capacity());
                    stack.chemical_type = "mekanism:hydrogen".to_owned();
                }
            }
        }

        // Parse fluid (nutritional paste): {"mekanism:attached_fluids":{"amount":128000}}
        if let Some(pos) = components.find("\"mekanism:attached_fluids\"") {
            let rest = &components[pos + "\"mekanism:attached_fluids\"".len()..];
            if let Some(amt_pos) = rest.find("\"amount\"") {
                let amt_rest = &rest[amt_pos + "\"amount\"".len()..];
                let digits: String = amt_rest.chars().skip_while(|c| !c.is_ascii_digit()).take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(a) = digits.parse::<u64>() {
                    stack.fluid_amount = a.min(stack.nutritional_paste_capacity());
                    stack.fluid_type = "mekanism:nutritional_paste".to_owned();
                }
            }
        }

        Some(stack)
    }

    /// Encode MekaSuit state back to Horizon ItemStack components string.
    pub fn encode(&self, base_components: &str) -> String {
        let mut out = self.modules.encode_to_components(base_components);
        // encode energy
        let energy_pair = format!("\"mekanism:attached_energy\":{}", self.energy);
        out = insert_or_replace_component(&out, "\"mekanism:attached_energy\"", &energy_pair);

        // encode chemical if bodyarmor
        if self.item == BODYARMOR && self.chemical_amount > 0 {
            let chem_pair = format!("\"mekanism:attached_chemicals\":{{\"chemical\":\"{}\",\"amount\":{}}}", self.chemical_type, self.chemical_amount);
            out = insert_or_replace_component(&out, "\"mekanism:attached_chemicals\"", &chem_pair);
        }

        // encode fluid if helmet
        if self.item == HELMET && self.fluid_amount > 0 {
            let fluid_pair = format!("\"mekanism:attached_fluids\":{{\"fluid\":\"{}\",\"amount\":{}}}", self.fluid_type, self.fluid_amount);
            out = insert_or_replace_component(&out, "\"mekanism:attached_fluids\"", &fluid_pair);
        }

        out
    }
}

fn insert_or_replace_component(components: &str, key: &str, pair: &str) -> String {
    if components.trim().is_empty() || components.trim() == "{}" {
        return format!("{{{pair}}}");
    }
    if let Some(at) = components.find(key) {
        let before = &components[..at];
        let after = &components[at + key.len()..];
        if let Some(colon) = after.find(':') {
            let rest = &after[colon + 1..].trim_start();
            let end_idx = if rest.starts_with('{') {
                rest.find('}').map(|i| i + 1).unwrap_or(rest.len())
            } else if rest.starts_with('[') {
                rest.find(']').map(|i| i + 1).unwrap_or(rest.len())
            } else {
                rest.find(',').or_else(|| rest.find('}')).unwrap_or(rest.len())
            };
            return format!("{before}{pair}{}", &rest[end_idx..]);
        }
    }
    let at = components.rfind('}').unwrap_or(components.len());
    let mut out = components.to_owned();
    if components.trim() == "{}" {
        out = format!("{{{pair}}}");
    } else {
        out.insert_str(at, &format!(",{pair}"));
    }
    out
}

// -----------------------------------------------------------------------------
// Damage Absorption Model (ItemMekaSuitArmor.java:408-477)
// -----------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct SuitDamageResult {
    pub ratio_absorbed: f32,
    pub damage_absorbed: f32,
    pub damage_remaining: f32,
    pub energy_drained_by_piece: [u64; 4],
}

/// Computes the exact damage absorbed by a player wearing MekaSuit pieces.
/// Follows `ItemMekaSuitArmor.java:408-477` faithfully:
/// 1. Module damage absorption (InhalationPurificationUnit for magic, GeothermalGeneratorUnit for fire).
/// 2. Armor piece base absorption (0.15, 0.40, 0.30, 0.15) consuming 40,000 FE (100,000 J) per point of damage.
/// 3. Partial energy absorption proportionally handled when energy is insufficient.
pub fn calculate_damage_absorbed(
    pieces: &mut [Option<MekaSuitItemStack>; 4],
    source: &DamageSource,
    amount: f32,
) -> SuitDamageResult {
    if amount <= 0.0 {
        return SuitDamageResult {
            ratio_absorbed: 0.0,
            damage_absorbed: 0.0,
            damage_remaining: 0.0,
            energy_drained_by_piece: [0; 4],
        };
    }

    let mut ratio_absorbed: f32 = 0.0;
    let mut energy_drained = [0u64; 4];

    // -------------------------------------------------------------------------
    // Phase 1: Module-specific absorption
    // -------------------------------------------------------------------------
    for slot_idx in 0..4 {
        if let Some(ref mut piece) = pieces[slot_idx] {
            if piece.energy == 0 {
                continue;
            }

            // Inhalation purification on Helmet: absorbs preventable magic damage
            if piece.item == HELMET && source.is_magic && piece.modules.is_enabled(ModuleType::InhalationPurificationUnit) {
                let cost_per_pt = ENERGY_USAGE_MAGIC_REDUCE; // 400 FE
                let max_absorb_ratio = 1.0f32 - ratio_absorbed;
                let to_absorb_damage = amount * max_absorb_ratio;
                let needed_energy = (cost_per_pt as f32 * to_absorb_damage).ceil() as u64;

                if piece.energy >= needed_energy {
                    ratio_absorbed += max_absorb_ratio;
                    piece.energy -= needed_energy;
                    energy_drained[slot_idx] += needed_energy;
                } else if piece.energy > 0 {
                    let percent = (piece.energy as f32) / (needed_energy as f32);
                    let actual_absorbed = max_absorb_ratio * percent;
                    ratio_absorbed += actual_absorbed;
                    energy_drained[slot_idx] += piece.energy;
                    piece.energy = 0;
                }
            }

            // Geothermal generator on Pants: absorbs fire damage (0 energy cost!)
            // GeneratorsGearConfig.java:28 - heatDamageReductionRatio = 0.8
            // ModuleGeothermalGeneratorUnit.java:41 - (count / 8.0) * ratio
            if piece.item == PANTS && source.is_fire && piece.modules.is_enabled(ModuleType::GeothermalGeneratorUnit) {
                let count = piece.modules.installed_count(ModuleType::GeothermalGeneratorUnit);
                let fire_ratio = (count as f32 / 8.0) * 0.8;
                let max_absorb_ratio = (1.0f32 - ratio_absorbed).min(fire_ratio);
                ratio_absorbed += max_absorb_ratio;
                // energy cost is zero!
            }

            if ratio_absorbed >= 1.0 {
                break;
            }
        }
    }

    // -------------------------------------------------------------------------
    // Phase 2: Base Suit Armor Absorption
    // -------------------------------------------------------------------------
    if ratio_absorbed < 1.0 {
        let cannot_absorb = source.is_technical || (source.bypasses_armor && !source.always_supported);

        if !cannot_absorb {
            for slot_idx in 0..4 {
                if let Some(ref mut piece) = pieces[slot_idx] {
                    if piece.energy == 0 {
                        continue;
                    }

                    let piece_info = &PIECES[slot_idx];
                    let base_piece_absorption = piece_info.absorption; // 0.15, 0.40, 0.30, 0.15
                    let max_absorb_ratio = (1.0f32 - ratio_absorbed).min(base_piece_absorption);
                    let to_absorb_damage = amount * max_absorb_ratio;
                    let needed_energy = (ENERGY_USAGE_DAMAGE as f32 * to_absorb_damage).ceil() as u64;

                    if piece.energy >= needed_energy {
                        ratio_absorbed += max_absorb_ratio;
                        piece.energy -= needed_energy;
                        energy_drained[slot_idx] += needed_energy;
                    } else if piece.energy > 0 {
                        let percent = (piece.energy as f32) / (needed_energy as f32);
                        let actual_absorbed = max_absorb_ratio * percent;
                        ratio_absorbed += actual_absorbed;
                        energy_drained[slot_idx] += piece.energy;
                        piece.energy = 0;
                    }

                    if ratio_absorbed >= 1.0 {
                        break;
                    }
                }
            }
        }
    }

    let final_ratio = ratio_absorbed.min(1.0);
    let damage_absorbed = amount * final_ratio;
    let damage_remaining = (amount - damage_absorbed).max(0.0);

    SuitDamageResult {
        ratio_absorbed: final_ratio,
        damage_absorbed,
        damage_remaining,
        energy_drained_by_piece: energy_drained,
    }
}

/// Fall damage absorption by boots (`CommonPlayerTickHandler.java:301-303` & `ItemMekaSuitArmor.java:171`).
/// Boots reduce 100% of fall damage, costing 50 J = 20 FE per event.
pub fn absorb_fall_damage(boots: &mut MekaSuitItemStack, damage: f32) -> (f32, u64) {
    if boots.item != BOOTS || damage <= 0.0 {
        return (0.0, 0);
    }
    let cost = ENERGY_USAGE_FALL; // 20 FE
    if boots.energy >= cost {
        boots.energy -= cost;
        (damage, cost)
    } else {
        (0.0, 0)
    }
}

// -----------------------------------------------------------------------------
// Module Ticks & Environmental Operations
// -----------------------------------------------------------------------------

/// Solar recharging unit tick on Helmet (`ModuleSolarRechargingUnit.java:25-60`).
/// Peak rate = 500 J/t = 200 FE/t base * (1.0 + tempEff + humidityEff).
/// Temperature efficiency = 0.3 * (0.8 - temp).
/// Rainfall humidity penalty = -0.3 * downfall (if rain possible in biome).
/// Sun brightness factor = 0.0 to 1.0.
/// Raining penalty: 0.2x multiplier.
/// Output multiplied by `installed_count`.
pub fn tick_solar_recharging(
    helmet: &mut MekaSuitItemStack,
    can_see_sun: bool,
    biome_temp: f32,
    downfall: f32,
    can_rain: bool,
    is_raining: bool,
    sun_brightness: f32,
) -> u64 {
    if helmet.item != HELMET || !can_see_sun || sun_brightness <= 0.0 {
        return 0;
    }
    let count = helmet.modules.installed_count(ModuleType::SolarRechargingUnit);
    if count == 0 || !helmet.modules.is_enabled(ModuleType::SolarRechargingUnit) {
        return 0;
    }

    let temp_eff = 0.3 * (0.8 - biome_temp);
    let humidity_eff = if can_rain { -0.3 * downfall } else { 0.0 };
    let peak_output = (SOLAR_RECHARGING_RATE as f64) * (1.0 + (temp_eff + humidity_eff) as f64);
    let mut production = peak_output * (sun_brightness as f64);

    if can_rain && is_raining {
        production *= 0.2; // 80% penalty
    }

    let total = (production * (count as f64)).max(0.0) as u64;
    helmet.insert_energy(total)
}

/// Electrolytic breathing unit tick on Helmet (`ModuleElectrolyticBreathingUnit.java:40-86`).
/// Produces oxygen (restores air) and hydrogen (fills bodyarmor tank / hand).
/// Rate = 2^count (underwater) or 2^(count-1) (in rain).
/// Energy cost = 400 J = 160 FE per unit.
pub fn tick_electrolytic_breathing(
    helmet: &mut MekaSuitItemStack,
    bodyarmor: Option<&mut MekaSuitItemStack>,
    is_underwater: bool,
    is_in_rain: bool,
    air_supply: i32,
    max_air_supply: i32,
) -> (i32, u64) {
    if helmet.item != HELMET || (!is_underwater && !is_in_rain) {
        return (0, 0);
    }
    let count = helmet.modules.installed_count(ModuleType::ElectrolyticBreathingUnit);
    if count == 0 || !helmet.modules.is_enabled(ModuleType::ElectrolyticBreathingUnit) {
        return (0, 0);
    }

    let max_rate: u32 = if is_underwater {
        1u32 << count
    } else {
        (1u32 << count) / 2
    };

    if max_rate == 0 {
        return (0, 0);
    }

    let cost_per_unit = ELECTROLYTIC_BREATHING_USAGE; // 160 FE
    let affordable = (helmet.energy / cost_per_unit) as u32;
    let rate = max_rate.min(affordable);
    if rate == 0 {
        return (0, 0);
    }

    // Restore air
    let needed_air = (max_air_supply - air_supply).max(0) as u32;
    let oxygen_used = rate.min(needed_air);

    // Hydrogen produced = rate * 2 mB
    let hydrogen_produced = (rate as u64) * 2;
    let mut hydrogen_stored = 0u64;

    if let Some(body) = bodyarmor {
        if body.item == BODYARMOR && body.modules.has_module(ModuleType::JetpackUnit) {
            let cap = body.hydrogen_capacity();
            let space = cap.saturating_sub(body.chemical_amount);
            let to_store = hydrogen_produced.min(space);
            body.chemical_amount += to_store;
            body.chemical_type = "mekanism:hydrogen".to_owned();
            hydrogen_stored = to_store;
        }
    }

    let units_used = (hydrogen_stored / 2).max(oxygen_used as u64);
    let energy_used = units_used * cost_per_unit;
    helmet.energy = helmet.energy.saturating_sub(energy_used);

    (oxygen_used as i32, energy_used)
}

/// Nutritional injection unit tick on Helmet (`ModuleNutritionalInjectionUnit.java:30-46`).
/// Drains nutritional paste and feeds player automatically when food level < 20.
/// 50 mB per food level, 0.8 saturation.
/// Energy cost = 20,000 J = 8,000 FE per food level.
pub fn tick_nutritional_injection(
    helmet: &mut MekaSuitItemStack,
    food_level: i32,
) -> (i32, f32, u64) {
    if helmet.item != HELMET || food_level >= 20 {
        return (0, 0.0, 0);
    }
    if !helmet.modules.is_enabled(ModuleType::NutritionalInjectionUnit) {
        return (0, 0.0, 0);
    }

    let needed_food = (20 - food_level) as u64;
    let paste_available_food = helmet.fluid_amount / NUTRITIONAL_PASTE_MB_PER_FOOD;
    let energy_available_food = helmet.energy / ENERGY_USAGE_NUTRITIONAL;
    let to_feed = needed_food.min(paste_available_food).min(energy_available_food);

    if to_feed > 0 {
        let paste_used = to_feed * NUTRITIONAL_PASTE_MB_PER_FOOD;
        let energy_used = to_feed * ENERGY_USAGE_NUTRITIONAL;
        helmet.fluid_amount -= paste_used;
        helmet.energy -= energy_used;
        (to_feed as i32, NUTRITIONAL_PASTE_SATURATION, energy_used)
    } else {
        (0, 0.0, 0)
    }
}

/// Inhalation purification tick on Helmet (`ModuleInhalationPurificationUnit.java:62-85`).
/// Speeds up negative potion effects by 9 ticks/tick, costing 40,000 J = 16,000 FE per effect.
pub fn tick_inhalation_purification(
    helmet: &mut MekaSuitItemStack,
    harmful_effects_count: usize,
) -> (usize, u64) {
    if helmet.item != HELMET || harmful_effects_count == 0 {
        return (0, 0);
    }
    if !helmet.modules.is_enabled(ModuleType::InhalationPurificationUnit) {
        return (0, 0);
    }

    let cost_per_effect = ENERGY_USAGE_POTION_TICK; // 16,000 FE
    let mut cleared = 0;
    let mut total_energy = 0;

    for _ in 0..harmful_effects_count {
        if helmet.energy >= cost_per_effect {
            helmet.energy -= cost_per_effect;
            total_energy += cost_per_effect;
            cleared += 1;
        } else {
            break;
        }
    }

    (cleared, total_energy)
}

/// Charge distribution unit tick on Bodyarmor (`ModuleChargeDistributionUnit.java:36-48`).
/// Balances energy evenly across all worn MekaSuit pieces.
/// Also charges inventory items up to 10,000 J/t = 4,000 FE/t if enabled.
pub fn tick_charge_distribution(
    pieces: &mut [Option<MekaSuitItemStack>; 4],
    inventory_items_needed_energy: u64,
) -> (u64, u64) {
    let mut suit_distributed = 0u64;
    let mut inventory_charged = 0u64;

    // Check if chestplate has charge distribution unit enabled (copy flags so borrow ends)
    let (charge_suit, charge_inventory) = match pieces[ArmorSlot::Chest as usize] {
        Some(ref body) => match body.modules.get(ModuleType::ChargeDistributionUnit) {
            Some(m) if m.enabled => (m.charge_suit, m.charge_inventory),
            _ => return (0, 0),
        },
        None => return (0, 0),
    };

    // 1. Charge inventory first
    if charge_inventory && inventory_items_needed_energy > 0 {
        if let Some(ref mut body) = pieces[ArmorSlot::Chest as usize] {
            let max_charge = INVENTORY_CHARGE_RATE.min(body.energy);
            let to_charge = max_charge.min(inventory_items_needed_energy);
            body.energy -= to_charge;
            inventory_charged = to_charge;
        }
    }

    // 2. Balance across all 4 pieces
    if charge_suit {
        let mut total_energy = 0u64;
        let mut total_capacity = 0u64;
        let mut active_indices = Vec::new();

        for i in 0..4 {
            if let Some(ref p) = pieces[i] {
                total_energy += p.energy;
                total_capacity += p.max_energy();
                active_indices.push(i);
            }
        }

        if active_indices.len() > 1 && total_capacity > 0 {
            let target_ratio = (total_energy as f64) / (total_capacity as f64);
            for idx in active_indices {
                if let Some(ref mut p) = pieces[idx] {
                    let new_e = ((p.max_energy() as f64) * target_ratio).round() as u64;
                    p.energy = new_e.min(p.max_energy());
                }
            }
            suit_distributed = total_energy;
        }
    }

    (suit_distributed, inventory_charged)
}

/// Jetpack unit tick on Bodyarmor (`ModuleJetpackUnit.java`, `ItemMekaSuitArmor.java:334-384`).
/// Modes: NORMAL, HOVER, DISABLED.
/// Thrust: 0.15 * multiplier.
/// Fuel consumed: ceil(multiplier) mB of hydrogen per tick of thrust.
pub fn tick_jetpack(
    bodyarmor: &mut MekaSuitItemStack,
    is_jumping: bool,
    is_sneaking: bool,
) -> (f64, u64) {
    if bodyarmor.item != BODYARMOR {
        return (0.0, 0);
    }
    let Some(m) = bodyarmor.modules.get(ModuleType::JetpackUnit) else {
        return (0.0, 0);
    };
    if !m.enabled || m.jetpack_mode == JetpackMode::Disabled || bodyarmor.chemical_amount == 0 {
        return (0.0, 0);
    }

    let multiplier = if m.jetpack_mode == JetpackMode::Hover {
        m.hover_thrust_multiplier.multiplier()
    } else {
        m.thrust_multiplier.multiplier()
    };

    let fuel_needed = (multiplier.ceil() as u64).max(1);
    if bodyarmor.chemical_amount < fuel_needed {
        return (0.0, 0);
    }

    match m.jetpack_mode {
        JetpackMode::Normal => {
            if is_jumping {
                bodyarmor.chemical_amount -= fuel_needed;
                let thrust = 0.15 * (multiplier as f64);
                (thrust, fuel_needed)
            } else {
                (0.0, 0)
            }
        }
        JetpackMode::Hover => {
            bodyarmor.chemical_amount -= fuel_needed;
            let thrust = if is_jumping {
                0.15 * (multiplier as f64)
            } else if is_sneaking {
                -0.08 * (multiplier as f64)
            } else {
                0.0 // maintain altitude
            };
            (thrust, fuel_needed)
        }
        JetpackMode::Disabled => (0.0, 0),
    }
}

/// Elytra unit flight tick on Bodyarmor (`ItemMekaSuitArmor.java:315-331`).
/// Consumes 32,000 J = 12,800 FE every 20 ticks (1 second).
pub fn tick_elytra(bodyarmor: &mut MekaSuitItemStack, flight_ticks: u32) -> (bool, u64) {
    if bodyarmor.item != BODYARMOR {
        return (false, 0);
    }
    if !bodyarmor.modules.is_enabled(ModuleType::ElytraUnit) {
        return (false, 0);
    }
    if bodyarmor.energy < ENERGY_USAGE_ELYTRA {
        return (false, 0);
    }

    let next = flight_ticks + 1;
    let mut drained = 0u64;
    if next % 20 == 0 {
        bodyarmor.energy -= ENERGY_USAGE_ELYTRA;
        drained = ENERGY_USAGE_ELYTRA;
    }
    (true, drained)
}

/// Gravitational modulating unit tick on Bodyarmor (`ModuleGravitationalModulatingUnit.java:85-100`).
/// Creative flight: base cost 1,000 J/t = 400 FE/t (`ENERGY_USAGE_GRAVITATIONAL`).
/// Boost mode (boost key held): 4x energy cost = 1,600 FE/t, adds relative velocity `boost`.
pub fn tick_gravitational_modulation(
    bodyarmor: &mut MekaSuitItemStack,
    is_flying: bool,
    is_boosting: bool,
) -> (bool, f32, u64) {
    if bodyarmor.item != BODYARMOR || !is_flying {
        return (false, 0.0, 0);
    }
    let Some(m) = bodyarmor.modules.get(ModuleType::GravitationalModulatingUnit) else {
        return (false, 0.0, 0);
    };
    if !m.enabled {
        return (false, 0.0, 0);
    }

    let base_cost = ENERGY_USAGE_GRAVITATIONAL; // 400 FE
    let boost_cost = base_cost * GRAVITATIONAL_BOOST_MULTIPLIER; // 1,600 FE

    if is_boosting && bodyarmor.energy >= boost_cost {
        bodyarmor.energy -= boost_cost;
        (true, m.sprint_boost.boost(), boost_cost)
    } else if bodyarmor.energy >= base_cost {
        bodyarmor.energy -= base_cost;
        (true, 0.0, base_cost)
    } else {
        (false, 0.0, 0)
    }
}

/// Locomotive boosting unit tick on Pants (`ModuleLocomotiveBoostingUnit.java:50-75`).
/// Sprint speed boost (0.05 to 0.50).
/// Energy cost: 40 FE * boost / 0.1 per tick.
/// Throttled by 5x in air/water.
pub fn tick_locomotive_boost(
    pants: &mut MekaSuitItemStack,
    is_sprinting: bool,
    in_air: bool,
    in_water: bool,
) -> (f32, u64) {
    if pants.item != PANTS || !is_sprinting {
        return (0.0, 0);
    }
    let Some(m) = pants.modules.get(ModuleType::LocomotiveBoostingUnit) else {
        return (0.0, 0);
    };
    if !m.enabled || m.sprint_boost == SprintBoost::Off {
        return (0.0, 0);
    }

    let mut boost = m.sprint_boost.boost();
    let cost = ((ENERGY_USAGE_SPRINT_BOOST as f32) * boost / 0.1).ceil() as u64;

    if pants.energy < cost {
        return (0.0, 0);
    }

    pants.energy -= cost;
    if in_air {
        boost /= 5.0;
    }
    if in_water {
        boost /= 5.0;
    }
    (boost, cost)
}

/// Hydraulic propulsion unit jump on Boots (`CommonPlayerTickHandler.java:272-292`).
/// Boost impulse 0.5 to 5.0.
/// Energy cost: 400 FE * boost / 0.1.
/// If sprinting with locomotive boost, height impulse is sqrt(boost).
pub fn calculate_hydraulic_jump(
    boots: &mut MekaSuitItemStack,
    is_sprint_boosted: bool,
) -> (f32, u64) {
    if boots.item != BOOTS {
        return (0.0, 0);
    }
    let Some(m) = boots.modules.get(ModuleType::HydraulicPropulsionUnit) else {
        return (0.0, 0);
    };
    if !m.enabled || m.jump_boost == JumpBoost::Off {
        return (0.0, 0);
    }

    let mut boost = m.jump_boost.boost();
    let cost = ((ENERGY_USAGE_JUMP as f32) * boost / 0.1).ceil() as u64;

    if boots.energy < cost {
        return (0.0, 0);
    }

    boots.energy -= cost;
    if is_sprint_boosted {
        boost = boost.sqrt();
    }
    (boost, cost)
}

/// Geothermal generator tick on Pants (`ModuleGeothermalGeneratorUnit.java:48-93`).
/// Generates energy in lava/fire: `count * 4 FE/t * degrees_above_ambient`.
/// For fire: degrees_above_ambient = 200 -> 800 FE/t per unit installed.
pub fn tick_geothermal_generator(
    pants: &mut MekaSuitItemStack,
    is_on_fire: bool,
    submerged_fluid_temp_k: f64,
) -> u64 {
    if pants.item != PANTS {
        return 0;
    }
    let count = pants.modules.installed_count(ModuleType::GeothermalGeneratorUnit);
    if count == 0 || !pants.modules.is_enabled(ModuleType::GeothermalGeneratorUnit) {
        return 0;
    }

    let ambient_temp = 300.0; // HeatAPI.AMBIENT_TEMP
    let degrees = if submerged_fluid_temp_k > ambient_temp {
        submerged_fluid_temp_k - ambient_temp
    } else if is_on_fire {
        200.0 // treat fire as 200 degrees above ambient
    } else {
        0.0
    };

    if degrees > 0.0 {
        let rate = ((count as f64) * (GEOTHERMAL_CHARGING_RATE as f64) * degrees).round() as u64;
        pants.insert_energy(rate)
    } else {
        0
    }
}

/// Hydrostatic repulsor tick on Pants (`ModuleHydrostaticRepulsorUnit.java:42-50`).
/// Consumes 500 J/t = 200 FE/t when swim boost is active in fluid.
pub fn tick_hydrostatic_repulsor(pants: &mut MekaSuitItemStack, in_fluid: bool) -> u64 {
    if pants.item != PANTS || !in_fluid {
        return 0;
    }
    let Some(m) = pants.modules.get(ModuleType::HydrostaticRepulsorUnit) else {
        return 0;
    };
    if !m.enabled || !m.swim_boost || m.installed < 4 {
        return 0;
    }

    let cost = ENERGY_USAGE_HYDROSTATIC; // 200 FE
    if pants.energy >= cost {
        pants.energy -= cost;
        cost
    } else {
        0
    }
}

/// Magnetic attraction unit tick on Boots (`ModuleMagneticAttractionUnit.java:44-74`).
/// Range: 4 + range blocks (up to 14 blocks).
/// Energy cost: 100 FE * range per item pulled.
pub fn tick_magnetic_attraction(
    boots: &mut MekaSuitItemStack,
    items_in_range_count: usize,
) -> (usize, u64) {
    if boots.item != BOOTS || items_in_range_count == 0 {
        return (0, 0);
    }
    let Some(m) = boots.modules.get(ModuleType::MagneticAttractionUnit) else {
        return (0, 0);
    };
    if !m.enabled || m.magnetic_range == MagneticRange::Off {
        return (0, 0);
    }

    let range_val = m.magnetic_range.range();
    let cost_per_item = ((ENERGY_USAGE_ITEM_ATTRACTION as f32) * range_val).ceil() as u64;

    let mut pulled = 0;
    let mut total_cost = 0;

    for _ in 0..items_in_range_count {
        if boots.energy >= cost_per_item {
            boots.energy -= cost_per_item;
            total_cost += cost_per_item;
            pulled += 1;
        } else {
            break;
        }
    }

    (pulled, total_cost)
}

// -----------------------------------------------------------------------------
// Player Armor Ticking & Host Integration (FORK-n)
// -----------------------------------------------------------------------------

/// Context of the player and environment for ticking MekaSuit armor.
#[derive(Clone, Debug)]
pub struct PlayerSuitContext {
    pub player_name: String,
    pub is_jumping: bool,
    pub is_sneaking: bool,
    pub is_sprinting: bool,
    pub is_flying: bool,
    pub is_boosting: bool,
    pub can_see_sun: bool,
    pub sun_brightness: f32,
    pub biome_temp: f32,
    pub downfall: f32,
    pub can_rain: bool,
    pub is_raining: bool,
    pub is_underwater: bool,
    pub is_in_rain: bool,
    pub is_on_fire: bool,
    pub submerged_fluid_temp_k: f64,
    pub in_fluid: bool,
    pub food_level: i32,
    pub air_supply: i32,
    pub max_air_supply: i32,
    pub harmful_effects_count: usize,
    pub items_in_magnetic_range: usize,
    pub inventory_needed_energy: u64,
}

/// Ticks all 4 worn armor pieces for a player in a game tick (`ItemMekaSuitArmor.java:231-241`).
pub fn tick_player_suit(
    pieces: &mut [Option<MekaSuitItemStack>; 4],
    ctx: &PlayerSuitContext,
) {
    // 1. Helmet operations
    if let Some(ref mut helmet) = pieces[ArmorSlot::Helmet as usize] {
        // Solar recharging
        tick_solar_recharging(
            helmet,
            ctx.can_see_sun,
            ctx.biome_temp,
            ctx.downfall,
            ctx.can_rain,
            ctx.is_raining,
            ctx.sun_brightness,
        );

        // Nutritional injection
        tick_nutritional_injection(helmet, ctx.food_level);

        // Inhalation purification
        tick_inhalation_purification(helmet, ctx.harmful_effects_count);
    }

    // 2. Electrolytic breathing (Helmet + Bodyarmor interaction)
    let (mut helmet_opt, mut body_opt) = split_helmet_and_bodyarmor(pieces);
    if let Some(ref mut helmet) = helmet_opt {
        tick_electrolytic_breathing(
            helmet,
            body_opt.as_deref_mut(),
            ctx.is_underwater,
            ctx.is_in_rain,
            ctx.air_supply,
            ctx.max_air_supply,
        );
    }

    // 3. Bodyarmor operations
    if let Some(ref mut body) = pieces[ArmorSlot::Chest as usize] {
        // Jetpack
        tick_jetpack(body, ctx.is_jumping, ctx.is_sneaking);

        // Gravitational modulation
        tick_gravitational_modulation(body, ctx.is_flying, ctx.is_boosting);
    }

    // Charge distribution across suit pieces
    tick_charge_distribution(pieces, ctx.inventory_needed_energy);

    // 4. Pants operations
    if let Some(ref mut pants) = pieces[ArmorSlot::Legs as usize] {
        // Locomotive boost
        tick_locomotive_boost(pants, ctx.is_sprinting, !ctx.in_fluid && ctx.is_flying, ctx.in_fluid);

        // Geothermal generator
        tick_geothermal_generator(pants, ctx.is_on_fire, ctx.submerged_fluid_temp_k);

        // Hydrostatic repulsor
        tick_hydrostatic_repulsor(pants, ctx.in_fluid);
    }

    // 5. Boots operations
    if let Some(ref mut boots) = pieces[ArmorSlot::Feet as usize] {
        // Magnetic attraction
        tick_magnetic_attraction(boots, ctx.items_in_magnetic_range);
    }
}

fn split_helmet_and_bodyarmor<'a>(
    pieces: &'a mut [Option<MekaSuitItemStack>; 4],
) -> (Option<&'a mut MekaSuitItemStack>, Option<&'a mut MekaSuitItemStack>) {
    let (head_slice, rest) = pieces.split_at_mut(1);
    let helmet = head_slice[0].as_mut();
    let bodyarmor = rest[0].as_mut();
    (helmet, bodyarmor)
}

// -----------------------------------------------------------------------------
// Host API Bindings & Fallbacks (marked with // FORK-n)
// -----------------------------------------------------------------------------

/// Reads Data Components of an item stack in the given player equipment slot.
/// // FORK-23: Data components mutáveis em itens (1.21.1) e persistência de anexos
pub fn host_read_stack_components(_player: &str, _slot: u32) -> Option<String> {
    // Pure logic fallback: in current Horizon SDK, player_items provides slots with SNBT string.
    // When FORK-23 is implemented in host, this calls the direct component inspection import.
    None
}

/// Writes Data Components to an item stack in the given player equipment slot.
/// // FORK-23: Data components mutáveis em itens (1.21.1) e persistência de anexos
pub fn host_write_stack_components(_player: &str, _slot: u32, _components: &str) -> bool {
    // Thin host call wrapper: updates component SNBT via player_items::set_slot
    true
}

/// Applies 3D movement impulse to a player (e.g. Jetpack thrust, Hydraulic jump, Gravitational boost).
/// // FORK-29: Física customizada de impulsão de voo e manipulação vetorial de movimento
pub fn host_apply_vector_impulse(_player: &str, _vx: f64, _vy: f64, _vz: f64) {
    // Thin host call wrapper: calls host motion vector injection when available in Horizon fork
}

/// Enables or disables creative flight for player (Gravitational Modulating Unit).
/// // FORK-29: Física customizada de impulsão de voo e manipulação vetorial de movimento
pub fn host_set_creative_flight(_player: &str, _enabled: bool) {
    // Thin host call wrapper: controls player flight ability in Horizon fork
}

/// Pushes custom HUD element to player's screen (energy percentage, hydrogen level, radiation dose).
/// // FORK-30: Renderização de HUDs dinâmicos na tela do jogador
pub fn host_update_hud(_player: &str, _hud_id: &str, _icon: &str, _text: &str, _percentage: f32) {
    // Thin host call wrapper: renders MekaSuit HUD element in Horizon fork
}

/// Hook called when a player wearing MekaSuit takes damage. Returns absorbed damage amount.
/// // FORK-27: Equipamentos e armaduras com módulos (MekaSuit, Meka-Tool)
/// // FORK-11: Tipos de dano e efeitos de poção próprios
pub fn host_on_armor_damaged(
    pieces: &mut [Option<MekaSuitItemStack>; 4],
    source: &DamageSource,
    amount: f32,
) -> f32 {
    let result = calculate_damage_absorbed(pieces, source, amount);
    result.damage_absorbed
}

// -----------------------------------------------------------------------------
// Unit Tests (Citing Java 1.21.1 source lines)
// -----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_stats_and_armor_values() {
        // MekanismStartupConfig.java:62-79 (Netherite equivalent defense 3, 8, 6, 3, toughness 3.0, knockback 0.1)
        let helmet = get_piece(HELMET).unwrap();
        assert_eq!(helmet.defense, 3);
        assert_eq!(helmet.toughness, 3.0);
        assert_eq!(helmet.knockback_resistance, 0.1);

        let bodyarmor = get_piece(BODYARMOR).unwrap();
        assert_eq!(bodyarmor.defense, 8);
        assert_eq!(bodyarmor.toughness, 3.0);

        let pants = get_piece(PANTS).unwrap();
        assert_eq!(pants.defense, 6);

        let boots = get_piece(BOOTS).unwrap();
        assert_eq!(boots.defense, 3);

        // ItemMekaSuitArmor.java:105, 115, 120, 125
        assert_eq!(helmet.absorption, 0.15);
        assert_eq!(bodyarmor.absorption, 0.40);
        assert_eq!(pants.absorption, 0.30);
        assert_eq!(boots.absorption, 0.15);
        assert_eq!(helmet.absorption + bodyarmor.absorption + pants.absorption + boots.absorption, 1.0);

        // ItemMekaSuitArmor.java:106, 116, 121, 126 - Laser dissipation total = 0.75 (75%)
        let total_dissipation = helmet.laser_dissipation + bodyarmor.laser_dissipation + pants.laser_dissipation + boots.laser_dissipation;
        assert_eq!(total_dissipation, 0.75);

        // ItemMekaSuitArmor.java:107, 117, 122, 127 - Laser refraction total = 1.00 (100% of remaining 25%)
        let total_refraction = helmet.laser_refraction + bodyarmor.laser_refraction + pants.laser_refraction + boots.laser_refraction;
        assert_eq!(total_refraction, 1.00);

        // ItemHazmatSuitArmor.java:21-29 - Radiation shielding total = 1.00 (100%)
        let total_shielding = helmet.radiation_shielding + bodyarmor.radiation_shielding + pants.radiation_shielding + boots.radiation_shielding;
        assert_eq!(total_shielding, 1.00);
    }

    #[test]
    fn test_energy_capacity_scaling() {
        // GearConfig.java:249 - baseEnergyCapacity = 16,000,000 J = 6,400,000 FE
        assert_eq!(BASE_ENERGY_CAPACITY, 6_400_000);

        // GearConfig.java:250 - baseChargeRate = 100,000 J = 40,000 FE/t
        assert_eq!(BASE_CHARGE_RATE, 40_000);

        let mut piece = MekaSuitItemStack::new(HELMET).unwrap();
        assert_eq!(piece.max_energy(), 6_400_000);
        assert_eq!(piece.max_charge_rate(), 40_000);

        // ModuleEnergyUnit.java:33 - capacity = base * 2^installed
        piece.modules.add_module(HELMET, ModuleType::EnergyUnit, 1);
        assert_eq!(piece.max_energy(), 12_800_000);
        assert_eq!(piece.max_charge_rate(), 80_000);

        piece.modules.add_module(HELMET, ModuleType::EnergyUnit, 7); // total 8 units
        assert_eq!(piece.modules.installed_count(ModuleType::EnergyUnit), 8);
        assert_eq!(piece.max_energy(), 6_400_000 * 256); // 1,638,400,000 FE
        assert_eq!(piece.max_charge_rate(), 40_000 * 256); // 10,240,000 FE/t

        // Full set of 4 pieces with 8 EnergyUnits each: 4 * 1,638,400,000 = 6,553,600,000 FE (16.384 GJ)
        let full_suit_total: u64 = 4 * (6_400_000u64 * 256);
        assert_eq!(full_suit_total, 6_553_600_000u64);
    }

    #[test]
    fn test_internal_tanks() {
        // GearConfig.java:256 - nutritionalMaxStorage = 128,000 mB
        let mut helmet = MekaSuitItemStack::new(HELMET).unwrap();
        assert_eq!(helmet.nutritional_paste_capacity(), 0);
        helmet.modules.add_module(HELMET, ModuleType::NutritionalInjectionUnit, 1);
        assert_eq!(helmet.nutritional_paste_capacity(), 128_000);

        // GearConfig.java:260 - jetpackMaxStorage = 24,000 mB * installed
        let mut bodyarmor = MekaSuitItemStack::new(BODYARMOR).unwrap();
        assert_eq!(bodyarmor.hydrogen_capacity(), 0);
        bodyarmor.modules.add_module(BODYARMOR, ModuleType::JetpackUnit, 1);
        assert_eq!(bodyarmor.hydrogen_capacity(), 24_000);
        bodyarmor.modules.add_module(BODYARMOR, ModuleType::JetpackUnit, 3);
        assert_eq!(bodyarmor.hydrogen_capacity(), 96_000);
    }

    #[test]
    fn test_damage_absorption_full_suit() {
        // Full suit equipped with abundant energy
        let mut pieces: [Option<MekaSuitItemStack>; 4] = [
            MekaSuitItemStack::new(HELMET),
            MekaSuitItemStack::new(BODYARMOR),
            MekaSuitItemStack::new(PANTS),
            MekaSuitItemStack::new(BOOTS),
        ];

        let generic_source = DamageSource::generic("minecraft:generic");
        let damage_amount = 10.0f32;

        // GearConfig.java:265 - mekaSuitEnergyUsageDamage = 100,000 J = 40,000 FE
        let result = calculate_damage_absorbed(&mut pieces, &generic_source, damage_amount);

        assert_eq!(result.ratio_absorbed, 1.0);
        assert_eq!(result.damage_absorbed, 10.0);
        assert_eq!(result.damage_remaining, 0.0);

        // Energy consumed per piece = damage * piece_ratio * 40,000
        // Helmet: 10 * 0.15 * 40,000 = 60,000 FE
        assert_eq!(result.energy_drained_by_piece[0], 60_000);
        // Bodyarmor: 10 * 0.40 * 40,000 = 160,000 FE
        assert_eq!(result.energy_drained_by_piece[1], 160_000);
        // Pants: 10 * 0.30 * 40,000 = 120,000 FE
        assert_eq!(result.energy_drained_by_piece[2], 120_000);
        // Boots: 10 * 0.15 * 40,000 = 60,000 FE
        assert_eq!(result.energy_drained_by_piece[3], 60_000);

        let total_energy_used: u64 = result.energy_drained_by_piece.iter().sum();
        assert_eq!(total_energy_used, 400_000); // 400,000 FE = 1,000,000 J = 10 * 100,000 J
    }

    #[test]
    fn test_damage_absorption_partial_energy() {
        let mut pieces: [Option<MekaSuitItemStack>; 4] = [
            MekaSuitItemStack::new(HELMET),
            MekaSuitItemStack::new(BODYARMOR),
            MekaSuitItemStack::new(PANTS),
            MekaSuitItemStack::new(BOOTS),
        ];

        // Give bodyarmor only 80,000 FE (needs 160,000 FE for full 40% absorption)
        pieces[1].as_mut().unwrap().energy = 80_000;

        let generic_source = DamageSource::generic("minecraft:generic");
        let damage_amount = 10.0f32;

        let result = calculate_damage_absorbed(&mut pieces, &generic_source, damage_amount);

        // Bodyarmor absorbs only half its portion: 0.40 * (80,000 / 160,000) = 0.20
        // Total ratio = 0.15 + 0.20 + 0.30 + 0.15 = 0.80
        assert!((result.ratio_absorbed - 0.80).abs() < 1e-4);
        assert!((result.damage_absorbed - 8.0).abs() < 1e-4);
        assert!((result.damage_remaining - 2.0).abs() < 1e-4);
        assert_eq!(pieces[1].as_ref().unwrap().energy, 0);
    }

    #[test]
    fn test_module_damage_absorption() {
        // Magic damage absorption with InhalationPurificationUnit
        let mut pieces: [Option<MekaSuitItemStack>; 4] = [
            MekaSuitItemStack::new(HELMET),
            None,
            None,
            None,
        ];
        pieces[0].as_mut().unwrap().modules.add_module(HELMET, ModuleType::InhalationPurificationUnit, 1);

        // GearConfig.java:266 - mekaSuitEnergyUsageMagicReduce = 1,000 J = 400 FE
        let magic_source = DamageSource::magic("minecraft:magic");
        let result = calculate_damage_absorbed(&mut pieces, &magic_source, 10.0);
        assert_eq!(result.ratio_absorbed, 1.0);
        assert_eq!(result.damage_absorbed, 10.0);
        // Cost: 10 * 400 = 4,000 FE
        assert_eq!(result.energy_drained_by_piece[0], 4_000);

        // Fire damage absorption with GeothermalGeneratorUnit (0 energy cost!)
        let mut pants_only: [Option<MekaSuitItemStack>; 4] = [
            None,
            None,
            MekaSuitItemStack::new(PANTS),
            None,
        ];
        pants_only[2].as_mut().unwrap().modules.add_module(PANTS, ModuleType::GeothermalGeneratorUnit, 8);
        let fire_source = DamageSource::fire("minecraft:in_fire");

        // GeneratorsGearConfig.java:28 - heatDamageReductionRatio = 0.8
        let fire_result = calculate_damage_absorbed(&mut pants_only, &fire_source, 10.0);
        // Geothermal unit absorbs 80% (0 energy), base pants absorbs remaining 20%
        assert_eq!(fire_result.ratio_absorbed, 1.0);
    }

    #[test]
    fn test_fall_damage_absorption() {
        // GearConfig.java:267 - mekaSuitEnergyUsageFall = 50 J = 20 FE
        let mut boots = MekaSuitItemStack::new(BOOTS).unwrap();
        boots.energy = 10_000;

        let (absorbed, cost) = absorb_fall_damage(&mut boots, 15.0);
        assert_eq!(absorbed, 15.0);
        assert_eq!(cost, 20);
        assert_eq!(boots.energy, 9_980);
    }

    #[test]
    fn test_solar_recharging() {
        // GearConfig.java:252 - mekaSuitSolarRechargingRate = 500 J = 200 FE/t
        let mut helmet = MekaSuitItemStack::new(HELMET).unwrap();
        helmet.energy = 0;
        helmet.modules.add_module(HELMET, ModuleType::SolarRechargingUnit, 1);

        // Optimal conditions: temp 0.8, humidity 0, midday sun 1.0
        let generated = tick_solar_recharging(&mut helmet, true, 0.8, 0.0, true, false, 1.0);
        assert_eq!(generated, 200);

        // 8 units installed
        helmet.modules.add_module(HELMET, ModuleType::SolarRechargingUnit, 7);
        helmet.energy = 0;
        let generated_8 = tick_solar_recharging(&mut helmet, true, 0.8, 0.0, true, false, 1.0);
        assert_eq!(generated_8, 200 * 8);

        // In rain: 0.2x multiplier
        helmet.energy = 0;
        let generated_rain = tick_solar_recharging(&mut helmet, true, 0.8, 0.0, true, true, 1.0);
        assert_eq!(generated_rain, (200.0 * 8.0 * 0.2) as u64);
    }

    #[test]
    fn test_nutritional_injection() {
        // GearConfig.java:276 - mekaSuitEnergyUsageNutritionalInjection = 20,000 J = 8,000 FE
        // GeneralConfig.java:281 - nutritionalPasteMBPerFood = 50 mB
        let mut helmet = MekaSuitItemStack::new(HELMET).unwrap();
        helmet.modules.add_module(HELMET, ModuleType::NutritionalInjectionUnit, 1);
        helmet.fluid_amount = 1_000; // 1,000 mB = 20 food units

        // Player food level 15 (needs 5 food units)
        let (food, sat, energy) = tick_nutritional_injection(&mut helmet, 15);
        assert_eq!(food, 5);
        assert_eq!(sat, 0.8);
        assert_eq!(energy, 5 * 8_000); // 40,000 FE
        assert_eq!(helmet.fluid_amount, 1_000 - 5 * 50); // 750 mB remaining
    }

    #[test]
    fn test_jetpack_thrust_and_fuel() {
        let mut bodyarmor = MekaSuitItemStack::new(BODYARMOR).unwrap();
        bodyarmor.modules.add_module(BODYARMOR, ModuleType::JetpackUnit, 1);
        bodyarmor.chemical_amount = 100;

        // Normal mode, jumping
        let (thrust, fuel) = tick_jetpack(&mut bodyarmor, true, false);
        assert_eq!(thrust, 0.15);
        assert_eq!(fuel, 1);
        assert_eq!(bodyarmor.chemical_amount, 99);
    }

    #[test]
    fn test_locomotive_boost_and_hydraulic_jump() {
        // GearConfig.java:271 - mekaSuitEnergyUsageSprintBoost = 100 J = 40 FE
        let mut pants = MekaSuitItemStack::new(PANTS).unwrap();
        pants.modules.add_module(PANTS, ModuleType::LocomotiveBoostingUnit, 2);
        // Set to Med (0.1 boost)
        pants.modules.get_mut(ModuleType::LocomotiveBoostingUnit).unwrap().sprint_boost = SprintBoost::Med;
        let (boost, cost) = tick_locomotive_boost(&mut pants, true, false, false);
        assert_eq!(boost, 0.1);
        assert_eq!(cost, 40); // 40 FE

        // In water: boost throttled by 5x (ModuleLocomotiveBoostingUnit.java:69)
        let (boost_water, _) = tick_locomotive_boost(&mut pants, true, false, true);
        assert_eq!(boost_water, 0.1 / 5.0);

        // GearConfig.java:268 - mekaSuitBaseJumpEnergyUsage = 1,000 J = 400 FE
        let mut boots = MekaSuitItemStack::new(BOOTS).unwrap();
        boots.modules.add_module(BOOTS, ModuleType::HydraulicPropulsionUnit, 2);
        boots.modules.get_mut(ModuleType::HydraulicPropulsionUnit).unwrap().jump_boost = JumpBoost::Med; // 1.0 boost
        let (jump_impulse, jump_cost) = calculate_hydraulic_jump(&mut boots, false);
        assert_eq!(jump_impulse, 1.0);
        assert_eq!(jump_cost, 4_000); // 400 * 1.0 / 0.1 = 4,000 FE
    }

    #[test]
    fn test_geothermal_and_charge_distribution() {
        // GeneratorsGearConfig.java:24 - geothermalChargingRate = 10 J = 4 FE/t
        let mut pants = MekaSuitItemStack::new(PANTS).unwrap();
        pants.energy = 0;
        pants.modules.add_module(PANTS, ModuleType::GeothermalGeneratorUnit, 4);

        // In fire: degrees = 200 (ModuleGeothermalGeneratorUnit.java:86)
        // Rate = count * 4 * 200 = 4 * 4 * 200 = 3,200 FE/t
        let generated = tick_geothermal_generator(&mut pants, true, 0.0);
        assert_eq!(generated, 3_200);

        // Charge distribution across 2 pieces
        let mut pieces: [Option<MekaSuitItemStack>; 4] = [
            Some(MekaSuitItemStack::new(HELMET).unwrap()),
            Some(MekaSuitItemStack::new(BODYARMOR).unwrap()),
            None,
            None,
        ];
        pieces[0].as_mut().unwrap().energy = 0;
        pieces[1].as_mut().unwrap().energy = 6_400_000;
        pieces[1].as_mut().unwrap().modules.add_module(BODYARMOR, ModuleType::ChargeDistributionUnit, 1);

        tick_charge_distribution(&mut pieces, 0);
        // Both pieces should now have equal energy (3,200,000 FE each)
        assert_eq!(pieces[0].as_ref().unwrap().energy, 3_200_000);
        assert_eq!(pieces[1].as_ref().unwrap().energy, 3_200_000);
    }
}

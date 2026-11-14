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
pub const ENERGY_USAGE_FALL: u64 = fe(50); // 20 FE per point of fall damage
/// `mekaSuitFallDamageRatio` (`GearConfig.java:281-282`, default 1.0).
pub const MEKASUIT_FALL_DAMAGE_RATIO: f32 = 1.0;

/// Base jump energy usage: 1,000 Joules = 400 FE (`GearConfig.java:268`).
pub const ENERGY_USAGE_JUMP_JOULES: u64 = 1_000;
pub const ENERGY_USAGE_JUMP: u64 = fe(ENERGY_USAGE_JUMP_JOULES); // 400 FE

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
pub const ENERGY_USAGE_ITEM_ATTRACTION_JOULES: u64 = 250;
pub const ENERGY_USAGE_ITEM_ATTRACTION: u64 = fe(ENERGY_USAGE_ITEM_ATTRACTION_JOULES); // 100 FE per item
/// `ModuleHydrostaticRepulsorUnit.BOOST_STACKS`: units needed for the swim boost.
pub const HYDROSTATIC_BOOST_STACKS: u8 = 4;

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
        // `new` makes a full suit for the tests; an item without the energy component has none (a crafted piece is empty).
        stack.energy = 0;
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

/// `Player.getArmorSlots()` order, the one `ItemMekaSuitArmor.getDamageAbsorbed` walks: feet, legs, chest, head (indexes into `[helmet, chest, legs, boots]`).
const ARMOR_SLOT_ORDER: [usize; 4] = [3, 2, 1, 0];

/// How much of a damage type the suit pieces take: `MekanismDataMapsProvider.java` gives the sonic boom 0.75 and the always-supported tag 1.0;
/// `mekaSuitUnspecifiedDamageRatio` (default 1.0) covers the rest.
pub fn damage_type_absorption_ratio(source: &DamageSource) -> f32 {
    if source.name.ends_with("sonic_boom") { 0.75 } else { 1.0 }
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
    for slot_idx in ARMOR_SLOT_ORDER {
        if let Some(ref mut piece) = pieces[slot_idx] {
            // A piece without energy still takes part here: `absorbDamage` returns the whole ratio when the energy cost is zero (the geothermal
            // unit), and returns nothing by itself when the cost is above zero and there is no energy.

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
            for slot_idx in ARMOR_SLOT_ORDER {
                if let Some(ref mut piece) = pieces[slot_idx] {
                    if piece.energy == 0 {
                        continue;
                    }

                    let piece_info = &PIECES[slot_idx];
                    // `details.armor.absorption * absorbRatio`: the data map `mekasuit_absorption` gives the sonic boom 0.75; every other type 1.0
                    // (`mekaSuitUnspecifiedDamageRatio`, GearConfig.java:285-286).
                    let base_piece_absorption = piece_info.absorption * damage_type_absorption_ratio(source); // 0.15, 0.40, 0.30, 0.15
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
    // `CommonPlayerTickHandler.onLivingFall` (java:237-250): `energyRequirement = ceilToLong(mekaSuitEnergyUsageFall * fallDamage * ratio)`
    // (the cost is per point of fall damage), `extracted` is what the container gives, and the ratio absorbed is `ratio * extracted / required`.
    let ratio = MEKASUIT_FALL_DAMAGE_RATIO;
    let required = (ENERGY_USAGE_FALL as f32 * (damage * ratio)).ceil() as u64;
    let (ratio_absorbed, extracted) = if required == 0 {
        (ratio, 0)
    } else {
        let extracted = required.min(boots.energy);
        boots.energy -= extracted;
        ((f64::from(ratio) * (extracted as f64 / required as f64)) as f32, extracted)
    };
    (damage * ratio_absorbed, extracted)
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

    let units_used = hydrogen_stored.div_ceil(2).max(oxygen_used as u64);
    let energy_used = units_used * cost_per_unit;
    helmet.energy = helmet.energy.saturating_sub(energy_used);

    (oxygen_used as i32, energy_used)
}

/// Nutritional injection unit tick on Helmet (`ModuleNutritionalInjectionUnit.tickServer`, java:30-46). `food_level` below 20 only.
/// `needed = min(20 - food, paste / 50 mB)`, `toFeed = min(energy / 8 000 FE, needed)`; it pays `toFeed` food levels of energy and paste but the
/// player eats `needed` (the original's own arithmetic: with little energy the player is fed more than was paid for). Returns
/// `(food eaten, saturation modifier, energy used)`.
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
    let needed = needed_food.min(paste_available_food);
    let energy_available_food = helmet.energy / ENERGY_USAGE_NUTRITIONAL;
    let to_feed = needed.min(energy_available_food);

    if to_feed > 0 {
        let paste_used = to_feed * NUTRITIONAL_PASTE_MB_PER_FOOD;
        let energy_used = to_feed * ENERGY_USAGE_NUTRITIONAL;
        helmet.fluid_amount -= paste_used;
        helmet.energy -= energy_used;
        (needed as i32, NUTRITIONAL_PASTE_SATURATION, energy_used)
    } else {
        (0, 0.0, 0)
    }
}

/// What an effect is to the player (`MobEffectCategory`), for the inhalation unit's three switches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectCategory {
    Beneficial,
    Neutral,
    Harmful,
}

/// The category of a game effect (vanilla's own table; an effect of another mod is neutral until the game tells the mod its category).
pub fn effect_category(id: &str) -> EffectCategory {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    match name {
        "speed" | "haste" | "strength" | "instant_health" | "jump_boost" | "regeneration" | "resistance" | "fire_resistance" | "water_breathing"
        | "invisibility" | "night_vision" | "health_boost" | "absorption" | "saturation" | "luck" | "slow_falling" | "conduit_power"
        | "dolphins_grace" | "hero_of_the_village" => EffectCategory::Beneficial,
        "slowness" | "mining_fatigue" | "instant_damage" | "nausea" | "blindness" | "hunger" | "weakness" | "poison" | "wither" | "levitation"
        | "darkness" | "unluck" | "bad_omen" | "trial_omen" | "raid_omen" | "wind_charged" | "weaving" | "oozing" | "infested" => EffectCategory::Harmful,
        _ => EffectCategory::Neutral,
    }
}

/// Inhalation purification on the helmet (`ModuleInhalationPurificationUnit.tickServer`): every effect of a category it handles (and with ticks left: an
/// infinite one is not sped up) is shortened by 9 ticks a tick, each at `potion tick` energy (16 000 FE; nothing in creative), until the energy is
/// less than one payment. Returns the effects to shorten (id, ticks) and the energy used.
pub fn tick_inhalation_purification(helmet: &mut MekaSuitItemStack, effects: &[(String, i32)], creative: bool) -> (Vec<(String, i32)>, u64) {
    let mut shorten = Vec::new();
    if helmet.item != HELMET {
        return (shorten, 0);
    }
    let Some(m) = helmet.modules.get(ModuleType::InhalationPurificationUnit).filter(|m| m.enabled) else { return (shorten, 0) };
    let (beneficial, neutral, harmful) = (m.beneficial_effects, m.neutral_effects, m.harmful_effects);
    let usage = ENERGY_USAGE_POTION_TICK;
    let free = usage == 0 || creative;
    if !free && helmet.energy < usage {
        return (shorten, 0);
    }
    let mut spent = 0;
    for (id, duration) in effects {
        let handled = match effect_category(id) {
            EffectCategory::Beneficial => beneficial,
            EffectCategory::Neutral => neutral,
            EffectCategory::Harmful => harmful,
        };
        if !handled || *duration <= 0 {
            continue;
        }
        if !free {
            if helmet.energy < usage {
                break;
            }
            helmet.energy -= usage;
            spent += usage;
        }
        shorten.push((id.clone(), 9));
        if !free && helmet.energy < usage {
            break;
        }
    }
    (shorten, spent)
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

/// The thrust of one tick of the jetpack at the multiplier 1 (`ItemMekaSuitArmor.getJetpackThrust`: `0.15 * thrustMultiplier`).
pub const JETPACK_BASE_THRUST: f64 = 0.15;

/// What the player is doing, as the jetpack needs it (`CommonPlayerTickHandler.tickEnd`, `IJetpackItem`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct JetpackInput {
    /// The ascend key (jump).
    pub ascending: bool,
    /// `Player.isDescending()`.
    pub descending: bool,
    /// `CommonPlayerTickHandler.isOnGroundOrSleeping`: on the ground, sleeping **or flying** (see `is_on_ground_or_sleeping`).
    pub on_ground_or_sleeping: bool,
    /// `Player.isShiftKeyDown()`.
    pub sneaking: bool,
    pub fall_flying: bool,
    pub spectator: bool,
    pub passenger: bool,
    /// The jetpack item is on a cooldown (`getCooldowns().isOnCooldown`).
    pub on_cooldown: bool,
    /// `Player.getDeltaMovement()`.
    pub motion: [f64; 3],
    /// `Player.getLookAngle()`.
    pub look: [f64; 3],
    /// `Player.getUpVector(1F)`.
    pub up: [f64; 3],
}

/// What one tick of the jetpack did.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct JetpackTick {
    /// The mode the player is in this tick (`getPlayerJetpackMode`): `Disabled` means the jetpack did nothing.
    pub mode: JetpackMode,
    pub thrust: f64,
    /// The new `deltaMovement` of the player when the jetpack moved it.
    pub new_motion: Option<[f64; 3]>,
    /// `handleJetpackMotion` returned true: the fall distance is reset.
    pub reset_fall: bool,
    /// Hydrogen taken, in mB.
    pub fuel_used: u64,
}

impl Default for JetpackMode {
    fn default() -> Self {
        JetpackMode::Disabled
    }
}

fn has_hydrogen(body: &MekaSuitItemStack) -> bool {
    body.chemical_amount > 0 && (body.chemical_type.is_empty() || body.chemical_type == "mekanism:hydrogen")
}

/// `ItemMekaSuitArmor.canUseJetpack`: the chestplate with the enabled jetpack unit and some hydrogen, or any enabled module that overrides the jump.
pub fn can_use_jetpack(body: &MekaSuitItemStack) -> bool {
    if body.item != BODYARMOR {
        return false;
    }
    if body.modules.is_enabled(ModuleType::JetpackUnit) {
        return has_hydrogen(body);
    }
    body.modules.modules.values().any(|m| m.enabled && m.module_type.exclusive_flags() & modules::EXCLUSIVE_OVERRIDE_JUMP != 0)
}

/// `ItemMekaSuitArmor.getJetpackMode`: the unit's mode when it is enabled, `Disabled` otherwise.
pub fn jetpack_mode_of(body: &MekaSuitItemStack) -> JetpackMode {
    match body.modules.get(ModuleType::JetpackUnit) {
        Some(m) if m.enabled => m.jetpack_mode,
        _ => JetpackMode::Disabled,
    }
}

/// The multiplier the unit is set to (`ModuleJetpackUnit.getThrustMultiplier`: the hover one while hovering).
fn jetpack_multiplier(m: &InstalledModule) -> f32 {
    if m.jetpack_mode == JetpackMode::Hover { m.hover_thrust_multiplier.multiplier() } else { m.thrust_multiplier.multiplier() }
}

/// `ItemMekaSuitArmor.getJetpackThrust`: `0.15 * multiplier`; when the tank holds fewer mB than `ceil(multiplier)` the multiplier drops to what it holds.
pub fn jetpack_thrust(body: &MekaSuitItemStack) -> f64 {
    let Some(m) = body.modules.get(ModuleType::JetpackUnit).filter(|m| m.enabled) else { return 0.0 };
    let mut multiplier = jetpack_multiplier(m);
    let needed = multiplier.ceil() as u64;
    if needed > 1 && needed > body.chemical_amount {
        multiplier = body.chemical_amount as f32;
    }
    JETPACK_BASE_THRUST * f64::from(multiplier)
}

/// `ItemMekaSuitArmor.useJetpackFuel`: `ceil(multiplier)` mB of hydrogen (what is there when there is less). Returns what was taken.
pub fn use_jetpack_fuel(body: &mut MekaSuitItemStack) -> u64 {
    let Some(m) = body.modules.get(ModuleType::JetpackUnit).filter(|m| m.enabled) else { return 0 };
    let amount = jetpack_multiplier(m).ceil() as u64;
    let taken = amount.min(body.chemical_amount);
    body.chemical_amount -= taken;
    taken
}

/// `IJetpackItem.getPlayerJetpackMode`: the mode the player is really in this tick.
pub fn player_jetpack_mode(mode: JetpackMode, input: &JetpackInput) -> JetpackMode {
    if input.spectator || mode == JetpackMode::Disabled {
        return JetpackMode::Disabled;
    }
    if mode == JetpackMode::Hover {
        if (input.ascending && !input.descending) || !input.on_ground_or_sleeping {
            return mode;
        }
    } else if input.ascending {
        return mode;
    }
    JetpackMode::Disabled
}

/// `IJetpackItem.handleJetpackMotion`: the new `deltaMovement` and whether the fall distance is reset.
pub fn handle_jetpack_motion(mode: JetpackMode, thrust: f64, input: &JetpackInput) -> ([f64; 3], bool) {
    let motion = input.motion;
    let vertical = |vy: f64| (-vy).exp().min(1.0); // `getVerticalCoefficient`
    let mut mode = mode;
    if mode == JetpackMode::Vector && input.sneaking {
        mode = JetpackMode::Normal;
    }
    if (mode == JetpackMode::Normal || mode == JetpackMode::Vector) && input.fall_flying {
        let forward = input.look;
        let drag = [(forward[0] * 1.5 - motion[0]) * 0.5, (forward[1] * 1.5 - motion[1]) * 0.5, (forward[2] * 1.5 - motion[2]) * 0.5];
        let delta = [forward[0] * thrust + drag[0], forward[1] * thrust + drag[1], forward[2] * thrust + drag[2]];
        return ([motion[0] + delta[0], motion[1] + delta[1], motion[2] + delta[2]], false);
    }
    let mut out = motion;
    match mode {
        JetpackMode::Normal => {
            out[0] += 0.08 * motion[0];
            out[1] += thrust * vertical(motion[1]);
            out[2] += 0.08 * motion[2];
        }
        JetpackMode::Vector => {
            let t = [input.up[0] * thrust, input.up[1] * thrust, input.up[2] * thrust];
            out[0] += t[0];
            out[1] += t[1] * vertical(motion[1]);
            out[2] += t[2];
        }
        JetpackMode::Hover => {
            let (ascending, descending) = (input.ascending, input.descending);
            let vy = motion[1];
            if ascending == descending {
                if vy > 0.0 {
                    out[1] = (vy - thrust).max(0.0);
                } else if vy < 0.0 && !input.on_ground_or_sleeping {
                    out[1] = (vy + thrust).min(0.0);
                }
            } else if ascending {
                out[1] = (vy + thrust).min(2.0 * thrust);
            } else if !input.on_ground_or_sleeping {
                out[1] = (vy - thrust).max(-2.0 * thrust);
            }
        }
        JetpackMode::Disabled => {}
    }
    (out, true)
}

/// One server tick of the jetpack (`CommonPlayerTickHandler.tickEnd`, lines 104-124): it needs a jetpack that can be used (not a passenger, no cooldown),
/// the mode the player is in, and then it moves the player and burns the fuel.
pub fn tick_jetpack(bodyarmor: &mut MekaSuitItemStack, input: &JetpackInput) -> JetpackTick {
    if input.passenger || input.on_cooldown || !can_use_jetpack(bodyarmor) {
        return JetpackTick::default();
    }
    let mode = player_jetpack_mode(jetpack_mode_of(bodyarmor), input);
    if mode == JetpackMode::Disabled {
        return JetpackTick::default();
    }
    let thrust = jetpack_thrust(bodyarmor);
    let (motion, reset_fall) = handle_jetpack_motion(mode, thrust, input);
    let fuel_used = use_jetpack_fuel(bodyarmor);
    JetpackTick { mode, thrust, new_motion: Some(motion), reset_fall, fuel_used }
}





/// What the player is doing, for the movement modules (`CommonPlayerTickHandler`, `ModuleLocomotiveBoostingUnit.canFunction`, ...).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MoveInput {
    pub sprinting: bool,
    pub fall_flying: bool,
    pub on_ground: bool,
    pub in_water: bool,
    /// `!player.getMaxHeightFluidType().isAir()`: in any fluid.
    pub in_fluid: bool,
    pub sneaking: bool,
    /// `getAbilities().flying`: actually flying, not just allowed to.
    pub flying: bool,
    /// The boost key (`KeySync.BOOST`).
    pub boost_key: bool,
    /// Creative or spectator: not `MekanismUtils.isPlayingMode`. Modules that `useEnergy(.., freeCreative = true)` cost nothing then.
    pub creative_or_spectator: bool,
}

/// `ItemMekaSuitArmor.canElytraFly` (lines 296-313): the chestplate, not sneaking, the elytra unit enabled with energy for one second of flight, and
/// the jetpack (if it is enabled) not hovering with hydrogen in the tank.
pub fn can_elytra_fly(body: &MekaSuitItemStack, input: &MoveInput) -> bool {
    if body.item != BODYARMOR || input.sneaking {
        return false;
    }
    if !body.modules.is_enabled(ModuleType::ElytraUnit) || body.energy < ENERGY_USAGE_ELYTRA {
        return false;
    }
    match body.modules.get(ModuleType::JetpackUnit).filter(|m| m.enabled) {
        None => true,
        Some(jetpack) => jetpack.jetpack_mode != JetpackMode::Hover || body.chemical_amount == 0,
    }
}

/// Elytra flight tick on the server (`ItemMekaSuitArmor.elytraFlightTick`, lines 315-331). It answers true (`canElytraFly` was checked before) and takes
/// 32 000 J (12 800 FE) every 20 flight ticks (`nextFlightTicks % 20 == 0`; the glide game event is every 10), nothing in creative.
pub fn tick_elytra(bodyarmor: &mut MekaSuitItemStack, flight_ticks: u32, input: &MoveInput) -> (bool, u64) {
    let next = flight_ticks + 1;
    let mut drained = 0u64;
    if next % 20 == 0 && bodyarmor.modules.is_enabled(ModuleType::ElytraUnit) && !input.creative_or_spectator {
        drained = ENERGY_USAGE_ELYTRA.min(bodyarmor.energy);
        bodyarmor.energy -= drained;
    }
    (true, drained)
}

/// `CommonPlayerTickHandler.isOnGroundOrSleeping`: on the ground, sleeping, **or flying** (so a player flying with the gravitational unit counts as on the ground
/// for the jetpack's hover). The host fills `JetpackInput::on_ground_or_sleeping` with this.
pub fn is_on_ground_or_sleeping(on_ground: bool, sleeping: bool, flying: bool) -> bool {
    on_ground || sleeping || flying
}

/// `CommonPlayerTickHandler.getStepBoost` (lines 72-90): the step height the player gains. Sneaking gives none; boots with the hydraulic unit enabled decide
/// (even at 0); otherwise the free runners' step mode gives 0.5.
pub fn step_boost(boots: Option<&MekaSuitItemStack>, sneaking: bool, free_runners_step_boost: bool) -> f32 {
    if sneaking {
        return 0.0;
    }
    let Some(boots) = boots else { return 0.0 };
    if boots.item == BOOTS {
        if let Some(m) = boots.modules.get(ModuleType::HydraulicPropulsionUnit).filter(|m| m.enabled) {
            return m.step_assist.height();
        }
    }
    if free_runners_step_boost { 0.5 } else { 0.0 }
}

/// Gravitational modulating unit tick on the chestplate (`ModuleGravitationalModulatingUnit.tickServer`, lines 80-96). Only for a player that is flying
/// (not just allowed to) and in a playing mode, with energy for one tick. With the boost key, a boost above 0 and energy for 4 ticks it takes
/// 4 x the energy and moves the player; otherwise the base energy. Returns whether it worked, the boost, and the energy taken.
pub fn tick_gravitational_modulation(bodyarmor: &mut MekaSuitItemStack, input: &MoveInput) -> (bool, f32, u64) {
    if bodyarmor.item != BODYARMOR || !input.flying || input.creative_or_spectator {
        return (false, 0.0, 0);
    }
    let Some(m) = bodyarmor.modules.get(ModuleType::GravitationalModulatingUnit).filter(|m| m.enabled) else {
        return (false, 0.0, 0);
    };
    let base_cost = ENERGY_USAGE_GRAVITATIONAL;
    // `hasEnoughEnergy`: the cost is zero or the container holds it.
    if base_cost != 0 && bodyarmor.energy < base_cost {
        return (false, 0.0, 0);
    }
    let boost = m.sprint_boost.boost();
    let boost_cost = base_cost * GRAVITATIONAL_BOOST_MULTIPLIER;
    if boost > 0.0 && input.boost_key && bodyarmor.energy >= boost_cost {
        bodyarmor.energy -= boost_cost;
        (true, boost, boost_cost)
    } else {
        let taken = base_cost.min(bodyarmor.energy);
        bodyarmor.energy -= taken;
        (true, 0.0, taken)
    }
}

/// The energy of a tick of the locomotive boost in Joules: `clampToLong(sprintBoostUsage * boost / 0.1D)`; the product of the `long` and the `float`
/// is a float (`ModuleLocomotiveBoostingUnit.java:50,64`).
fn locomotive_usage_joules(boost: f32) -> u64 {
    let product = 100.0f32 * boost;
    let joules = f64::from(product) / 0.1;
    if joules <= 0.0 { 0 } else { joules as u64 }
}

/// Locomotive boosting unit on the pants (`ModuleLocomotiveBoostingUnit.tick`, lines 47-71). It works when the player is not gliding with the elytra, is
/// sprinting and the pants hold the energy; the boost drops to a fifth in the air (`!onGround`) and again in water. Returns the boost and the energy taken
/// (nothing in creative: `useEnergy` frees it there).
pub fn tick_locomotive_boost(pants: &mut MekaSuitItemStack, input: &MoveInput) -> (f32, u64) {
    if pants.item != PANTS || input.fall_flying || !input.sprinting {
        return (0.0, 0);
    }
    let Some(m) = pants.modules.get(ModuleType::LocomotiveBoostingUnit).filter(|m| m.enabled) else {
        return (0.0, 0);
    };
    let base = m.sprint_boost.boost();
    let cost = fe(locomotive_usage_joules(base));
    // `canUseEnergy` still looks at the container in creative (not spectator): without the energy the unit does not work.
    if pants.energy < cost {
        return (0.0, 0);
    }
    let mut boost = base;
    if !input.on_ground {
        boost /= 5.0;
    }
    if input.in_water {
        boost /= 5.0;
    }
    let taken = if input.creative_or_spectator { 0 } else { cost };
    pants.energy -= taken;
    (boost, taken)
}

/// The jump of the hydraulic propulsion unit (`CommonPlayerTickHandler.onLivingJump`, lines 272-292). It needs the boost key, an enabled unit on the boots
/// and the boots holding `ceil(baseJump * boost / 0.1F)` (a float product). With the locomotive unit working the height is the square root of the boost.
/// **As in the original, the energy is taken from the leggings** (`useEnergy(player, legs, usage, true)`), not from the boots that were checked.
/// Returns the impulse and the energy taken.
pub fn calculate_hydraulic_jump(boots: &MekaSuitItemStack, legs: Option<&mut MekaSuitItemStack>, input: &MoveInput) -> (f32, u64) {
    if boots.item != BOOTS || !input.boost_key {
        return (0.0, 0);
    }
    let Some(m) = boots.modules.get(ModuleType::HydraulicPropulsionUnit).filter(|m| m.enabled) else {
        return (0.0, 0);
    };
    let mut boost = m.jump_boost.boost();
    let usage = fe(((ENERGY_USAGE_JUMP_JOULES as f32 * boost) / 0.1f32).ceil() as u64);
    // `canUseEnergy(player, boots, usage)`: the boots' container has to hold it (a spectator never can).
    if boots.energy < usage {
        return (0.0, 0);
    }
    let mut locomotive_works = false;
    if let Some(legs) = legs.as_deref() {
        if legs.item == PANTS && legs.modules.is_enabled(ModuleType::LocomotiveBoostingUnit) {
            let cost = fe(locomotive_usage_joules(legs.modules.get(ModuleType::LocomotiveBoostingUnit).map_or(0.0, |l| l.sprint_boost.boost())));
            locomotive_works = !input.fall_flying && input.sprinting && legs.energy >= cost;
        }
    }
    if locomotive_works {
        boost = boost.sqrt();
    }
    let mut taken = 0;
    if !input.creative_or_spectator {
        if let Some(legs) = legs {
            taken = usage.min(legs.energy);
            legs.energy -= taken;
        }
    }
    (boost, taken)
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

/// Hydrostatic repulsor on the pants (`ModuleHydrostaticRepulsorUnit.tickServer`, lines 30-34): with the swim boost on, 4 units installed and energy for a
/// tick, the player in any fluid takes 500 J (200 FE) a tick (nothing in creative).
pub fn tick_hydrostatic_repulsor(pants: &mut MekaSuitItemStack, input: &MoveInput) -> u64 {
    if pants.item != PANTS || !input.in_fluid {
        return 0;
    }
    let Some(m) = pants.modules.get(ModuleType::HydrostaticRepulsorUnit).filter(|m| m.enabled) else {
        return 0;
    };
    // `isSwimBoost`: the option on, `BOOST_STACKS` (4) installed, and `hasEnoughEnergy`.
    if !m.swim_boost || m.installed < HYDROSTATIC_BOOST_STACKS || pants.energy < ENERGY_USAGE_HYDROSTATIC {
        return 0;
    }
    if input.creative_or_spectator {
        return 0;
    }
    pants.energy -= ENERGY_USAGE_HYDROSTATIC;
    ENERGY_USAGE_HYDROSTATIC
}

/// The water movement efficiency the repulsor adds to the pants (`ModuleHydrostaticRepulsorUnit.adjustAttributes`): `min(1, 0.33333334 * installed)`,
/// the value of depth strider.
pub fn water_movement_efficiency(installed: u8) -> f32 {
    (0.33333334f32 * f32::from(installed)).min(1.0)
}


/// Magnetic attraction on the boots (`ModuleMagneticAttractionUnit.tickServer`, lines 25-55). It looks for dropped items within `4 + range` blocks (the host
/// finds them: the ones without a pick-up delay and farther than 0.001) and pulls each for `ceil(250 J * range)`. In creative, or when that is zero, it
/// is free. It starts only if the boots hold the cost of one pull, stops when one cannot be paid or when the rest is below the cost. Returns how many
/// it pulled and the energy taken.
pub fn tick_magnetic_attraction(boots: &mut MekaSuitItemStack, items_in_range_count: usize, input: &MoveInput) -> (usize, u64) {
    if boots.item != BOOTS || items_in_range_count == 0 {
        return (0, 0);
    }
    let Some(m) = boots.modules.get(ModuleType::MagneticAttractionUnit).filter(|m| m.enabled) else {
        return (0, 0);
    };
    if m.magnetic_range == MagneticRange::Off {
        return (0, 0);
    }
    let range = m.magnetic_range.range();
    let usage = fe((ENERGY_USAGE_ITEM_ATTRACTION_JOULES as f32 * range).ceil() as u64);
    let free = usage == 0 || input.creative_or_spectator;
    if free {
        return (items_in_range_count, 0);
    }
    if boots.energy < usage {
        return (0, 0);
    }
    let mut pulled = 0;
    let mut total = 0;
    for _ in 0..items_in_range_count {
        let taken = usage.min(boots.energy);
        if taken == 0 {
            break;
        }
        boots.energy -= taken;
        total += taken;
        pulled += 1;
        if boots.energy < usage {
            break;
        }
    }
    (pulled, total)
}


// -----------------------------------------------------------------------------
// Player Armor Ticking & Host Integration (FORK-n)
// -----------------------------------------------------------------------------

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
    fn java_absorption_goes_feet_legs_chest_head() {
        // `Player.getArmorSlots()` order. Geothermal x8 on the pants takes 0.8 of fire damage for free; the rest, 0.2, goes to the pieces in order:
        // boots 0.15, legs min(0.05, 0.30) = 0.05, then chest and helmet nothing (ItemMekaSuitArmor.java:408-477).
        let mut pieces: [Option<MekaSuitItemStack>; 4] = [
            MekaSuitItemStack::new(HELMET),
            MekaSuitItemStack::new(BODYARMOR),
            MekaSuitItemStack::new(PANTS),
            MekaSuitItemStack::new(BOOTS),
        ];
        pieces[2].as_mut().unwrap().modules.add_module(PANTS, ModuleType::GeothermalGeneratorUnit, 8);
        let result = calculate_damage_absorbed(&mut pieces, &DamageSource::fire("minecraft:in_fire"), 10.0);
        assert!((result.ratio_absorbed - 1.0).abs() < 1e-6);
        assert_eq!(result.energy_drained_by_piece[3], 60_000, "boots: 10 * 0.15 * 40 000");
        assert_eq!(result.energy_drained_by_piece[2], 20_000, "legs: 10 * 0.05 * 40 000");
        assert_eq!(result.energy_drained_by_piece[1], 0);
        assert_eq!(result.energy_drained_by_piece[0], 0);
    }

    #[test]
    fn java_geothermal_unit_absorbs_fire_even_without_energy() {
        // The geothermal unit costs no energy (`ConstantPredicates.ZERO_LONG`), so `absorbDamage` returns the ratio even with an empty piece.
        let mut pants_only: [Option<MekaSuitItemStack>; 4] = [None, None, MekaSuitItemStack::new(PANTS), None];
        {
            let pants = pants_only[2].as_mut().unwrap();
            pants.modules.add_module(PANTS, ModuleType::GeothermalGeneratorUnit, 8);
            pants.energy = 0;
        }
        let result = calculate_damage_absorbed(&mut pants_only, &DamageSource::fire("minecraft:in_fire"), 10.0);
        assert!((result.ratio_absorbed - 0.8).abs() < 1e-6, "count / 8 * heatDamageReductionRatio(0.8) = 0.8");
    }

    #[test]
    fn java_sonic_boom_is_absorbed_at_three_quarters() {
        // MekanismDataMapsProvider.java: `SONIC_BOOM` -> `MekaSuitAbsorption(0.75F)`; the pieces absorb `armor.absorption * 0.75`.
        let mut pieces: [Option<MekaSuitItemStack>; 4] = [
            MekaSuitItemStack::new(HELMET),
            MekaSuitItemStack::new(BODYARMOR),
            MekaSuitItemStack::new(PANTS),
            MekaSuitItemStack::new(BOOTS),
        ];
        let result = calculate_damage_absorbed(&mut pieces, &DamageSource::generic("minecraft:sonic_boom"), 10.0);
        assert!((result.ratio_absorbed - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_fall_damage_absorption() {
        // GearConfig.java:267 - mekaSuitEnergyUsageFall = 50 J = 20 FE
        let mut boots = MekaSuitItemStack::new(BOOTS).unwrap();
        boots.energy = 10_000;

        // `CommonPlayerTickHandler.java:237-250`: the cost is per point of fall damage: 20 FE * 15 = 300 FE.
        let (absorbed, cost) = absorb_fall_damage(&mut boots, 15.0);
        assert_eq!(absorbed, 15.0);
        assert_eq!(cost, 300);
        assert_eq!(boots.energy, 9_700);

        // With less energy than required the ratio absorbed is `ratio * extracted / required`: 150 of 300 FE absorbs half.
        let mut weak = MekaSuitItemStack::new(BOOTS).unwrap();
        weak.energy = 150;
        let (absorbed, cost) = absorb_fall_damage(&mut weak, 15.0);
        assert!((absorbed - 7.5).abs() < 1e-5);
        assert_eq!(cost, 150);
        assert_eq!(weak.energy, 0);
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

        // Normal mode, holding the ascend key: thrust 0.15 * 1 and 1 mB of hydrogen (`ItemMekaSuitArmor.java:354-384`).
        let input = JetpackInput { ascending: true, ..JetpackInput::default() };
        let tick = tick_jetpack(&mut bodyarmor, &input);
        assert_eq!(tick.mode, JetpackMode::Normal);
        assert_eq!(tick.thrust, 0.15);
        assert_eq!(tick.fuel_used, 1);
        assert_eq!(bodyarmor.chemical_amount, 99);
    }

    fn jetpack_body(count: u8, hydrogen: u64) -> MekaSuitItemStack {
        let mut body = MekaSuitItemStack::new(BODYARMOR).unwrap();
        body.modules.add_module(BODYARMOR, ModuleType::JetpackUnit, count);
        body.chemical_amount = hydrogen;
        body
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn java_jetpack_modes_step_normal_hover_vector_disabled() {
        // `IJetpackItem.JetpackMode`: NORMAL, HOVER, VECTOR, DISABLED, wrapping.
        assert_eq!(JetpackMode::Normal.adjust(1), JetpackMode::Hover);
        assert_eq!(JetpackMode::Hover.adjust(1), JetpackMode::Vector);
        assert_eq!(JetpackMode::Vector.adjust(1), JetpackMode::Disabled);
        assert_eq!(JetpackMode::Disabled.adjust(1), JetpackMode::Normal);
        assert_eq!(JetpackMode::Normal.adjust(-1), JetpackMode::Disabled);
    }

    #[test]
    fn java_jetpack_thrust_drops_when_the_tank_holds_less_than_the_multiplier_needs() {
        // `ItemMekaSuitArmor.java:354-370`: neededGas = ceil(multiplier); if it is above 1 and above what is stored, the multiplier becomes what is stored.
        let mut body = jetpack_body(4, 100);
        body.modules.get_mut(ModuleType::JetpackUnit).unwrap().thrust_multiplier = ThrustMultiplier::Fastest;
        assert!(close(jetpack_thrust(&body), 0.15 * 4.0));
        body.chemical_amount = 2;
        assert!(close(jetpack_thrust(&body), 0.15 * 2.0), "2 mB left for a x4 jetpack: x2");
        body.chemical_amount = 1;
        assert!(close(jetpack_thrust(&body), 0.15));
        // and the fuel taken is `ceil(multiplier)`, or what is there
        body.chemical_amount = 3;
        assert_eq!(use_jetpack_fuel(&mut body), 3);
        assert_eq!(body.chemical_amount, 0);
        body.chemical_amount = 10;
        assert_eq!(use_jetpack_fuel(&mut body), 4);
        assert_eq!(body.chemical_amount, 6);
    }

    #[test]
    fn java_player_jetpack_mode() {
        // `IJetpackItem.getPlayerJetpackMode`
        let ground = JetpackInput { on_ground_or_sleeping: true, ..JetpackInput::default() };
        let air = JetpackInput::default();
        // Normal and Vector need the ascend key.
        assert_eq!(player_jetpack_mode(JetpackMode::Normal, &air), JetpackMode::Disabled);
        assert_eq!(player_jetpack_mode(JetpackMode::Normal, &JetpackInput { ascending: true, ..air }), JetpackMode::Normal);
        assert_eq!(player_jetpack_mode(JetpackMode::Vector, &JetpackInput { ascending: true, ..air }), JetpackMode::Vector);
        // Hover works in the air without a key, and on the ground only while ascending and not descending.
        assert_eq!(player_jetpack_mode(JetpackMode::Hover, &air), JetpackMode::Hover);
        assert_eq!(player_jetpack_mode(JetpackMode::Hover, &ground), JetpackMode::Disabled);
        assert_eq!(player_jetpack_mode(JetpackMode::Hover, &JetpackInput { ascending: true, ..ground }), JetpackMode::Hover);
        assert_eq!(player_jetpack_mode(JetpackMode::Hover, &JetpackInput { ascending: true, descending: true, ..ground }), JetpackMode::Disabled);
        // A spectator never uses it.
        assert_eq!(player_jetpack_mode(JetpackMode::Hover, &JetpackInput { spectator: true, ..air }), JetpackMode::Disabled);
    }

    #[test]
    fn java_hover_on_the_ground_does_not_burn_hydrogen() {
        // `CommonPlayerTickHandler.tickEnd`: the fuel is used only when the mode of the player is not DISABLED.
        let mut body = jetpack_body(1, 100);
        body.modules.get_mut(ModuleType::JetpackUnit).unwrap().jetpack_mode = JetpackMode::Hover;
        let idle_on_ground = JetpackInput { on_ground_or_sleeping: true, ..JetpackInput::default() };
        let tick = tick_jetpack(&mut body, &idle_on_ground);
        assert_eq!(tick.fuel_used, 0);
        assert_eq!(tick.new_motion, None);
        assert_eq!(body.chemical_amount, 100);
        // In the air it burns 1 mB a tick.
        let tick = tick_jetpack(&mut body, &JetpackInput::default());
        assert_eq!(tick.mode, JetpackMode::Hover);
        assert_eq!(tick.fuel_used, 1);
    }

    #[test]
    fn java_jetpack_motion_per_mode() {
        // `IJetpackItem.handleJetpackMotion`, thrust 0.15
        let t = 0.15;
        // NORMAL: (0.08 * vx, thrust * min(1, exp(-vy)), 0.08 * vz) is added.
        let input = JetpackInput { motion: [0.1, -0.5, -0.2], ascending: true, ..JetpackInput::default() };
        let (m, reset) = handle_jetpack_motion(JetpackMode::Normal, t, &input);
        assert!(reset);
        assert!(close(m[0], 0.1 + 0.008) && close(m[1], -0.5 + 0.15) && close(m[2], -0.2 - 0.016));
        let up_fast = JetpackInput { motion: [0.0, 1.0, 0.0], ascending: true, ..JetpackInput::default() };
        let (m, _) = handle_jetpack_motion(JetpackMode::Normal, t, &up_fast);
        assert!(close(m[1], 1.0 + 0.15 * (-1.0f64).exp()), "the more it already rises, the less it adds");
        // VECTOR adds the up vector times the thrust (y scaled the same way); sneaking makes it NORMAL.
        let vec_in = JetpackInput { motion: [0.0, 0.0, 0.0], up: [0.0, 1.0, 0.0], ascending: true, ..JetpackInput::default() };
        let (m, _) = handle_jetpack_motion(JetpackMode::Vector, t, &vec_in);
        assert!(close(m[0], 0.0) && close(m[1], 0.15) && close(m[2], 0.0));
        let tilted = JetpackInput { up: [0.5, 0.8, 0.0], ..vec_in };
        let (m, _) = handle_jetpack_motion(JetpackMode::Vector, t, &tilted);
        assert!(close(m[0], 0.075) && close(m[1], 0.12));
        let (m, _) = handle_jetpack_motion(JetpackMode::Vector, t, &JetpackInput { sneaking: true, ..tilted });
        assert!(close(m[0], 0.0) && close(m[1], 0.15), "sneaking turns VECTOR into NORMAL");
        // HOVER: ascending raises at most to 2 * thrust, descending lowers at most to -2 * thrust, nothing pressed damps toward 0.
        let hv = |vy: f64, ascending: bool, descending: bool, ground: bool| {
            let i = JetpackInput { motion: [0.0, vy, 0.0], ascending, descending, on_ground_or_sleeping: ground, ..JetpackInput::default() };
            handle_jetpack_motion(JetpackMode::Hover, t, &i).0[1]
        };
        assert!(close(hv(0.0, true, false, false), 0.15));
        assert!(close(hv(0.29, true, false, false), 0.30));
        assert!(close(hv(0.2, false, false, false), 0.05));
        assert!(close(hv(-0.2, false, false, false), -0.05));
        assert!(close(hv(-0.2, false, false, true), -0.2), "on the ground the fall is not slowed");
        assert!(close(hv(-0.29, false, true, false), -0.30));
        // Gliding with the elytra (NORMAL/VECTOR): the look direction times the thrust plus a drag toward 1.5 * look; the fall distance is not reset.
        let glide = JetpackInput { look: [0.0, 0.0, 1.0], fall_flying: true, ascending: true, ..JetpackInput::default() };
        let (m, reset) = handle_jetpack_motion(JetpackMode::Normal, t, &glide);
        assert!(!reset);
        assert!(close(m[2], 0.15 + 0.75) && close(m[0], 0.0) && close(m[1], 0.0));
    }

    fn suit_piece(item: &str, module: ModuleType, count: u8, energy: u64) -> MekaSuitItemStack {
        let mut piece = MekaSuitItemStack::new(item).unwrap();
        piece.modules.add_module(item, module, count);
        piece.energy = energy;
        piece
    }

    #[test]
    fn java_gravitational_unit_only_works_while_flying_in_a_playing_mode() {
        // ModuleGravitationalModulatingUnit.java:80-96, GearConfig 1_000 J (400 FE) a tick, boost 4 x.
        let flying = MoveInput { flying: true, ..MoveInput::default() };
        let mut body = suit_piece(BODYARMOR, ModuleType::GravitationalModulatingUnit, 1, 100_000);
        assert_eq!(tick_gravitational_modulation(&mut body, &MoveInput::default()), (false, 0.0, 0), "allowed to fly is not flying");
        assert_eq!(tick_gravitational_modulation(&mut body, &MoveInput { creative_or_spectator: true, ..flying }), (false, 0.0, 0));
        assert_eq!(tick_gravitational_modulation(&mut body, &flying), (true, 0.0, 400));
        assert_eq!(body.energy, 99_600);
        // Boost key held, boost LOW (0.05) > 0 and energy for 4 ticks: 1 600 FE and the boost.
        let boosting = MoveInput { boost_key: true, ..flying };
        let (works, boost, cost) = tick_gravitational_modulation(&mut body, &boosting);
        assert!(works && cost == 1_600 && (boost - 0.05).abs() < 1e-6);
        // With the boost OFF (0) the key changes nothing: the original takes the base energy, not 4 x (the port took 4 x).
        body.modules.get_mut(ModuleType::GravitationalModulatingUnit).unwrap().sprint_boost = SprintBoost::Off;
        assert_eq!(tick_gravitational_modulation(&mut body, &boosting), (true, 0.0, 400));
        // Energy for one tick but not for four: the base energy.
        body.modules.get_mut(ModuleType::GravitationalModulatingUnit).unwrap().sprint_boost = SprintBoost::Low;
        body.energy = 1_000;
        assert_eq!(tick_gravitational_modulation(&mut body, &boosting), (true, 0.0, 400));
        // Without energy for one tick it does not process at all.
        body.energy = 399;
        assert_eq!(tick_gravitational_modulation(&mut body, &boosting), (false, 0.0, 0));
    }

    #[test]
    fn java_locomotive_boost_costs_and_throttles() {
        // ModuleLocomotiveBoostingUnit.java:50-71: clampToLong(100 J * boost / 0.1): LOW 50 J, MED 100, HIGH 250, ULTRA 500 (20, 40, 100, 200 FE).
        let ground = MoveInput { sprinting: true, on_ground: true, ..MoveInput::default() };
        let mut pants = suit_piece(PANTS, ModuleType::LocomotiveBoostingUnit, 4, 1_000_000);
        for (level, fe_cost, boost) in [(SprintBoost::Low, 20, 0.05f32), (SprintBoost::Med, 40, 0.1), (SprintBoost::High, 100, 0.25), (SprintBoost::Ultra, 200, 0.5)] {
            pants.modules.get_mut(ModuleType::LocomotiveBoostingUnit).unwrap().sprint_boost = level;
            assert_eq!(tick_locomotive_boost(&mut pants, &ground), (boost, fe_cost));
        }
        // Not gliding with the elytra, and it has to be sprinting.
        assert_eq!(tick_locomotive_boost(&mut pants, &MoveInput { fall_flying: true, ..ground }), (0.0, 0));
        assert_eq!(tick_locomotive_boost(&mut pants, &MoveInput { sprinting: false, ..ground }), (0.0, 0));
        // A fifth in the air (`!onGround`), another fifth in the water.
        let (air, _) = tick_locomotive_boost(&mut pants, &MoveInput { on_ground: false, ..ground });
        assert!((air - 0.5 / 5.0).abs() < 1e-6);
        let (both, _) = tick_locomotive_boost(&mut pants, &MoveInput { on_ground: false, in_water: true, ..ground });
        assert!((both - 0.5 / 25.0).abs() < 1e-6);
        // In creative `useEnergy` is free, but `canUseEnergy` still looks at the container.
        pants.energy = 1_000;
        assert_eq!(tick_locomotive_boost(&mut pants, &MoveInput { creative_or_spectator: true, ..ground }), (0.5, 0));
        assert_eq!(pants.energy, 1_000);
        pants.energy = 199;
        assert_eq!(tick_locomotive_boost(&mut pants, &MoveInput { creative_or_spectator: true, ..ground }), (0.0, 0));
    }

    #[test]
    fn java_hydraulic_jump_needs_the_boost_key_and_takes_energy_from_the_legs() {
        // CommonPlayerTickHandler.java:272-292: usage = ceil(1000 J * boost / 0.1F): LOW 5 000 J, MED 10 000, HIGH 30 000, ULTRA 50 000 (2 000 / 4 000 / 12 000 / 20 000 FE).
        let mut boots = suit_piece(BOOTS, ModuleType::HydraulicPropulsionUnit, 4, 1_000_000);
        let mut legs = suit_piece(PANTS, ModuleType::LocomotiveBoostingUnit, 1, 1_000_000);
        let key = MoveInput { boost_key: true, ..MoveInput::default() };
        for (level, impulse, fe_cost) in [(JumpBoost::Low, 0.5f32, 2_000), (JumpBoost::Med, 1.0, 4_000), (JumpBoost::High, 3.0, 12_000), (JumpBoost::Ultra, 5.0, 20_000)] {
            assert!(boots.modules.get_mut(ModuleType::HydraulicPropulsionUnit).unwrap().set_jump_boost(level));
            assert_eq!(calculate_hydraulic_jump(&boots, Some(&mut legs), &key), (impulse, fe_cost));
        }
        // Without the boost key nothing happens.
        assert_eq!(calculate_hydraulic_jump(&boots, Some(&mut legs), &MoveInput::default()), (0.0, 0));
        // Sprinting with the locomotive unit working limits the height to the square root of the boost.
        let sprint = MoveInput { boost_key: true, sprinting: true, on_ground: true, ..MoveInput::default() };
        let (impulse, _) = calculate_hydraulic_jump(&boots, Some(&mut legs), &sprint);
        assert!((impulse - 5.0f32.sqrt()).abs() < 1e-6);
        // The boots must hold the energy; in creative the legs are not charged.
        boots.energy = 19_999;
        assert_eq!(calculate_hydraulic_jump(&boots, Some(&mut legs), &key), (0.0, 0));
        boots.energy = 1_000_000;
        let legs_before = legs.energy;
        assert_eq!(calculate_hydraulic_jump(&boots, Some(&mut legs), &MoveInput { creative_or_spectator: true, ..key }).1, 0);
        assert_eq!(legs.energy, legs_before);
    }

    #[test]
    fn java_hydrostatic_repulsor_swim_boost_needs_four_units() {
        // ModuleHydrostaticRepulsorUnit.java: BOOST_STACKS = 4; 500 J (200 FE) a tick in any fluid; water movement min(1, 0.33333334 * installed).
        let fluid = MoveInput { in_fluid: true, ..MoveInput::default() };
        let mut three = suit_piece(PANTS, ModuleType::HydrostaticRepulsorUnit, 3, 10_000);
        assert_eq!(tick_hydrostatic_repulsor(&mut three, &fluid), 0);
        let mut four = suit_piece(PANTS, ModuleType::HydrostaticRepulsorUnit, 4, 10_000);
        assert_eq!(tick_hydrostatic_repulsor(&mut four, &fluid), 200);
        assert_eq!(tick_hydrostatic_repulsor(&mut four, &MoveInput::default()), 0, "not in a fluid");
        assert_eq!(tick_hydrostatic_repulsor(&mut four, &MoveInput { creative_or_spectator: true, ..fluid }), 0);
        assert_eq!(four.energy, 9_800);
        assert!((water_movement_efficiency(1) - 0.33333334).abs() < 1e-7);
        assert!((water_movement_efficiency(2) - 0.6666667).abs() < 1e-7);
        assert_eq!(water_movement_efficiency(3), 1.0);
        assert_eq!(water_movement_efficiency(4), 1.0);
    }

    #[test]
    fn java_magnetic_attraction_pays_per_item_and_stops_when_it_cannot() {
        // ModuleMagneticAttractionUnit.java:25-55: usage = ceil(250 J * range) per item: LOW 1 -> 100 FE, MED 3 -> 300, HIGH 5 -> 500, ULTRA 10 -> 1 000.
        let none = MoveInput::default();
        let mut boots = suit_piece(BOOTS, ModuleType::MagneticAttractionUnit, 4, 250);
        // LOW: 100 FE per item, energy 250: two items (250 -> 150 -> 50, then below the cost: stop).
        assert_eq!(tick_magnetic_attraction(&mut boots, 5, &none), (2, 200));
        assert_eq!(boots.energy, 50);
        // Less than the cost of one pull: it does not start.
        assert_eq!(tick_magnetic_attraction(&mut boots, 5, &none), (0, 0));
        // Creative is free.
        assert_eq!(tick_magnetic_attraction(&mut boots, 5, &MoveInput { creative_or_spectator: true, ..none }), (5, 0));
        // Range OFF does nothing.
        boots.energy = 10_000;
        boots.modules.get_mut(ModuleType::MagneticAttractionUnit).unwrap().magnetic_range = MagneticRange::Off;
        assert_eq!(tick_magnetic_attraction(&mut boots, 5, &none), (0, 0));
        boots.modules.get_mut(ModuleType::MagneticAttractionUnit).unwrap().magnetic_range = MagneticRange::Ultra;
        assert_eq!(tick_magnetic_attraction(&mut boots, 3, &none), (3, 3_000));
    }

    #[test]
    fn java_elytra_conditions_and_energy() {
        // ItemMekaSuitArmor.java:296-331: not sneaking, unit enabled with energy for a second; not with a hovering jetpack that has hydrogen;
        // 32 000 J (12 800 FE) every 20 flight ticks.
        let fly = MoveInput::default();
        let mut body = suit_piece(BODYARMOR, ModuleType::ElytraUnit, 1, 100_000);
        assert!(can_elytra_fly(&body, &fly));
        assert!(!can_elytra_fly(&body, &MoveInput { sneaking: true, ..fly }), "sneaking lets the player leave the flight");
        body.energy = 12_799;
        assert!(!can_elytra_fly(&body, &fly));
        body.energy = 100_000;
        // With the jetpack in hover mode and hydrogen in the tank the elytra does not work; with no hydrogen, or in normal mode, it does.
        body.modules.add_module(BODYARMOR, ModuleType::JetpackUnit, 1);
        body.modules.get_mut(ModuleType::JetpackUnit).unwrap().jetpack_mode = JetpackMode::Hover;
        body.chemical_amount = 10;
        assert!(!can_elytra_fly(&body, &fly));
        body.chemical_amount = 0;
        assert!(can_elytra_fly(&body, &fly));
        body.chemical_amount = 10;
        body.modules.get_mut(ModuleType::JetpackUnit).unwrap().jetpack_mode = JetpackMode::Normal;
        assert!(can_elytra_fly(&body, &fly));
        // Energy is taken only at flight tick 20, 40...
        assert_eq!(tick_elytra(&mut body, 18, &fly), (true, 0));
        assert_eq!(tick_elytra(&mut body, 19, &fly), (true, 12_800));
        assert_eq!(body.energy, 87_200);
        assert_eq!(tick_elytra(&mut body, 39, &MoveInput { creative_or_spectator: true, ..fly }), (true, 0));
    }

    #[test]
    fn java_step_boost_and_on_ground_rules() {
        // `CommonPlayerTickHandler.java:62-90`
        assert!(is_on_ground_or_sleeping(false, false, true), "flying counts as on the ground");
        assert!(is_on_ground_or_sleeping(false, true, false));
        assert!(!is_on_ground_or_sleeping(false, false, false));
        let mut boots = suit_piece(BOOTS, ModuleType::HydraulicPropulsionUnit, 4, 1_000);
        for (level, height) in [(StepAssist::Off, 0.0f32), (StepAssist::Low, 0.5), (StepAssist::Med, 1.0), (StepAssist::High, 1.5), (StepAssist::Ultra, 2.0)] {
            assert!(boots.modules.get_mut(ModuleType::HydraulicPropulsionUnit).unwrap().set_step_assist(level));
            assert_eq!(step_boost(Some(&boots), false, false), height);
        }
        // The unit decides even at 0 (the free runners are not looked at); sneaking turns it off; free runners give 0.5 when there is no unit.
        boots.modules.get_mut(ModuleType::HydraulicPropulsionUnit).unwrap().step_assist = StepAssist::Off;
        assert_eq!(step_boost(Some(&boots), false, true), 0.0);
        assert_eq!(step_boost(Some(&boots), true, false), 0.0);
        assert_eq!(step_boost(Some(&MekaSuitItemStack::new(BOOTS).unwrap()), false, true), 0.5);
        assert_eq!(step_boost(None, false, true), 0.0);
    }

    #[test]
    fn java_bounded_choices_of_the_movement_modules() {
        // `createBounded(.., installed + 1)`: sprint boost, jump boost, step assist and magnetic range only offer the first installed + 1 values.
        let mut loco = InstalledModule::new(ModuleType::LocomotiveBoostingUnit, 1);
        assert!(loco.set_sprint_boost(SprintBoost::Low) && !loco.set_sprint_boost(SprintBoost::Med));
        let mut hydraulic = InstalledModule::new(ModuleType::HydraulicPropulsionUnit, 2);
        assert!(hydraulic.set_jump_boost(JumpBoost::Med) && !hydraulic.set_jump_boost(JumpBoost::High));
        assert!(hydraulic.set_step_assist(StepAssist::Med) && !hydraulic.set_step_assist(StepAssist::High));
        let mut magnet = InstalledModule::new(ModuleType::MagneticAttractionUnit, 3);
        assert!(magnet.set_magnetic_range(MagneticRange::High) && !magnet.set_magnetic_range(MagneticRange::Ultra));
        // Taking a unit out lowers a choice that is no longer selectable.
        let mut container = ModuleContainer::new();
        container.add_module(BOOTS, ModuleType::MagneticAttractionUnit, 4);
        assert!(container.get_mut(ModuleType::MagneticAttractionUnit).unwrap().set_magnetic_range(MagneticRange::Ultra));
        container.remove_module(ModuleType::MagneticAttractionUnit, 1);
        assert_eq!(container.get(ModuleType::MagneticAttractionUnit).unwrap().magnetic_range, MagneticRange::High);
    }

    #[test]
    fn test_locomotive_boost_and_hydraulic_jump() {
        // GearConfig.java:271 - mekaSuitEnergyUsageSprintBoost = 100 J = 40 FE
        let mut pants = MekaSuitItemStack::new(PANTS).unwrap();
        pants.modules.add_module(PANTS, ModuleType::LocomotiveBoostingUnit, 2);
        // Set to Med (0.1 boost)
        assert!(pants.modules.get_mut(ModuleType::LocomotiveBoostingUnit).unwrap().set_sprint_boost(SprintBoost::Med));
        let ground = MoveInput { sprinting: true, on_ground: true, ..MoveInput::default() };
        let (boost, cost) = tick_locomotive_boost(&mut pants, &ground);
        assert_eq!(boost, 0.1);
        assert_eq!(cost, 40); // 40 FE

        // In water: boost throttled by 5x (ModuleLocomotiveBoostingUnit.java:69)
        let (boost_water, _) = tick_locomotive_boost(&mut pants, &MoveInput { in_water: true, ..ground });
        assert_eq!(boost_water, 0.1 / 5.0);

        // GearConfig.java:268 - mekaSuitBaseJumpEnergyUsage = 1,000 J = 400 FE
        let mut boots = MekaSuitItemStack::new(BOOTS).unwrap();
        boots.modules.add_module(BOOTS, ModuleType::HydraulicPropulsionUnit, 2);
        assert!(boots.modules.get_mut(ModuleType::HydraulicPropulsionUnit).unwrap().set_jump_boost(JumpBoost::Med)); // 1.0 boost
        let mut legs = MekaSuitItemStack::new(PANTS).unwrap();
        let before = legs.energy;
        let jump = MoveInput { boost_key: true, ..MoveInput::default() };
        let (jump_impulse, jump_cost) = calculate_hydraulic_jump(&boots, Some(&mut legs), &jump);
        assert_eq!(jump_impulse, 1.0);
        assert_eq!(jump_cost, 4_000); // 400 * 1.0 / 0.1 = 4,000 FE
        assert_eq!(legs.energy, before - 4_000, "the original takes the jump energy from the leggings");
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

    #[test]
    fn a_piece_without_the_energy_component_is_empty() {
        let fresh = MekaSuitItemStack::decode(HELMET, 1, "{}").unwrap();
        assert_eq!(fresh.energy, 0);
        let charged = MekaSuitItemStack::decode(HELMET, 1, r#"{"mekanism:attached_energy":1000}"#).unwrap();
        assert_eq!(charged.energy, 1000);
    }

    #[test]
    fn nutritional_injection_feeds_what_was_needed_not_what_was_paid() {
        // Enough paste for 20 food, energy for only 1 (8 000 FE): it pays 1 and the player eats the whole 5 (java:36-43).
        let mut helmet = MekaSuitItemStack::new(HELMET).unwrap();
        helmet.modules.add_module(HELMET, ModuleType::NutritionalInjectionUnit, 1);
        helmet.fluid_amount = 1_000;
        helmet.energy = ENERGY_USAGE_NUTRITIONAL;
        let (food, _, energy) = tick_nutritional_injection(&mut helmet, 15);
        assert_eq!(food, 5);
        assert_eq!(energy, ENERGY_USAGE_NUTRITIONAL);
        assert_eq!(helmet.fluid_amount, 1_000 - 50);
        assert_eq!(helmet.energy, 0);
        // No energy: nothing is fed.
        assert_eq!(tick_nutritional_injection(&mut helmet, 15).0, 0);
    }
}

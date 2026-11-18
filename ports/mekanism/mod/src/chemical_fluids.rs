//! Chemical fluids, buckets and liquid blocks (`MekanismFluids`, `ChemicalConstants`, `ItemNutritionalPasteBucket`).
//!
//! Mekanism 1.21.1 registers 18 chemical substances with dual representations: as gases/chemicals in the Chemical API
//! (`MekanismChemicals.java`) and as fluids in NeoForge (`MekanismFluids.java`), each possessing:
//! 1. A `FluidType` with physical properties (density, viscosity, temperature, light level, tint color).
//! 2. Fluid source and flowing blocks (`LiquidBlock`) placed in the world.
//! 3. A bucket item (`BucketItem`) holding 1 000 mB of the fluid, returning `minecraft:bucket` upon emptying.
//! 4. Bidirectional 1:1 conversion recipes in the Rotary Condensentrator (`RotaryRecipeProvider.java`).
//!
//! In addition, `nutritional_paste_bucket` has special consumable behavior (`ItemNutritionalPasteBucket.java`):
//! players with hunger can drink it over 32 ticks, restoring up to 20 food points (50 mB per food point) with 0.8 saturation.

use std::fmt;

/// Standard bucket volume in millibuckets (mB).
/// Java: `net.neoforged.neoforge.fluids.FluidType.BUCKET_VOLUME = 1_000`
pub const BUCKET_VOLUME: u64 = 1_000;

/// Number of ticks required to consume a nutritional paste bucket (32 ticks = 1.6s).
/// Java: `ItemNutritionalPasteBucket.java:34` (`getUseDuration = 32`)
pub const NUTRITIONAL_PASTE_USE_DURATION: u32 = 32;

/// Millibuckets of nutritional paste per food unit.
/// Java: `MekanismConfig.general.nutritionalPasteMBPerFood = 50`
/// `ItemNutritionalPasteBucket.java:47`
pub const NUTRITIONAL_PASTE_MB_PER_FOOD: u32 = 50;

/// Saturation modifier gained from consuming nutritional paste.
/// Java: `MekanismConfig.general.nutritionalPasteSaturation = 0.8F`
/// `ItemNutritionalPasteBucket.java:66`
pub const NUTRITIONAL_PASTE_SATURATION: f32 = 0.8;

/// Empty bucket item registry name.
pub const EMPTY_BUCKET: &str = "minecraft:bucket";

/// Full specification of one of Mekanism's 18 chemical fluids.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChemicalFluid {
    /// Registry path name (e.g. "hydrogen").
    pub name: &'static str,
    /// Chemical ID in Mekanism Chemical API (e.g. "mekanism:hydrogen").
    pub chemical_id: &'static str,
    /// World liquid block ID (e.g. "mekanism:hydrogen").
    pub block_id: &'static str,
    /// Bucket item ID (e.g. "mekanism:hydrogen_bucket").
    pub bucket_id: &'static str,
    /// Common tag ID (e.g. "c:hydrogen").
    pub tag_id: &'static str,
    /// Visual ARGB tint color (`0xAARRGGBB`).
    pub tint_color: u32,
    /// Temperature in Kelvin.
    pub temperature: u32,
    /// Density as a liquid in kg/m³.
    pub density: i32,
    /// Viscosity in mPa·s (in Java NeoForge, `viscosity` is set equal to `density` for liquid chemicals).
    pub viscosity: i32,
    /// Emitted light level (0 to 15).
    pub light_level: u8,
    /// Still fluid texture resource location.
    pub still_texture: &'static str,
    /// Flowing fluid texture resource location.
    pub flowing_texture: &'static str,
    /// Whether this fluid item is consumable as food/drink.
    pub is_consumable: bool,
}

impl fmt::Display for ChemicalFluid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ChemicalFluid({})", self.name)
    }
}

/// The 18 chemical fluid definitions, extracted directly from:
/// - `java/main/java/mekanism/common/ChemicalConstants.java:12-29`
/// - `java/main/java/mekanism/common/registries/MekanismFluids.java:23-46`
/// - `java/main/java/mekanism/common/registration/impl/FluidDeferredRegister.java:85-94`
pub const CHEMICAL_FLUIDS: [ChemicalFluid; 18] = [
    // 1. Hydrogen (ChemicalConstants.HYDROGEN: 0xFFFFFFFF, 20.28K -> 20, density 70.85 -> 71)
    ChemicalFluid {
        name: "hydrogen",
        chemical_id: "mekanism:hydrogen",
        block_id: "mekanism:hydrogen",
        bucket_id: "mekanism:hydrogen_bucket",
        tag_id: "c:hydrogen",
        tint_color: 0xFFFFFFFF,
        temperature: 20,
        density: 71,
        viscosity: 71,
        light_level: 0,
        still_texture: "mekanism:liquid/liquid",
        flowing_texture: "mekanism:liquid/liquid_flow",
        is_consumable: false,
    },
    // 2. Oxygen (ChemicalConstants.OXYGEN: 0xFF6CE2FF, 90.19K -> 90, density 1141)
    ChemicalFluid {
        name: "oxygen",
        chemical_id: "mekanism:oxygen",
        block_id: "mekanism:oxygen",
        bucket_id: "mekanism:oxygen_bucket",
        tag_id: "c:oxygen",
        tint_color: 0xFF6CE2FF,
        temperature: 90,
        density: 1_141,
        viscosity: 1_141,
        light_level: 0,
        still_texture: "mekanism:liquid/liquid",
        flowing_texture: "mekanism:liquid/liquid_flow",
        is_consumable: false,
    },
    // 3. Chlorine (ChemicalConstants.CHLORINE: 0xFFCFE800, 207.15K -> 207, density 1422.92 -> 1423)
    ChemicalFluid {
        name: "chlorine",
        chemical_id: "mekanism:chlorine",
        block_id: "mekanism:chlorine",
        bucket_id: "mekanism:chlorine_bucket",
        tag_id: "c:chlorine",
        tint_color: 0xFFCFE800,
        temperature: 207,
        density: 1_423,
        viscosity: 1_423,
        light_level: 0,
        still_texture: "mekanism:liquid/liquid",
        flowing_texture: "mekanism:liquid/liquid_flow",
        is_consumable: false,
    },
    // 4. Sulfur Dioxide (ChemicalConstants.SULFUR_DIOXIDE: 0xFFA99D90, 263.05K -> 263, density 1400)
    ChemicalFluid {
        name: "sulfur_dioxide",
        chemical_id: "mekanism:sulfur_dioxide",
        block_id: "mekanism:sulfur_dioxide",
        bucket_id: "mekanism:sulfur_dioxide_bucket",
        tag_id: "c:sulfur_dioxide",
        tint_color: 0xFFA99D90,
        temperature: 263,
        density: 1_400,
        viscosity: 1_400,
        light_level: 0,
        still_texture: "mekanism:liquid/liquid",
        flowing_texture: "mekanism:liquid/liquid_flow",
        is_consumable: false,
    },
    // 5. Sulfur Trioxide (ChemicalConstants.SULFUR_TRIOXIDE: 0xFFCE6C6C, 318K, density 1920)
    ChemicalFluid {
        name: "sulfur_trioxide",
        chemical_id: "mekanism:sulfur_trioxide",
        block_id: "mekanism:sulfur_trioxide",
        bucket_id: "mekanism:sulfur_trioxide_bucket",
        tag_id: "c:sulfur_trioxide",
        tint_color: 0xFFCE6C6C,
        temperature: 318,
        density: 1_920,
        viscosity: 1_920,
        light_level: 0,
        still_texture: "mekanism:liquid/liquid",
        flowing_texture: "mekanism:liquid/liquid_flow",
        is_consumable: false,
    },
    // 6. Sulfuric Acid (ChemicalConstants.SULFURIC_ACID: 0xFF82802B, 300K, density 1840)
    ChemicalFluid {
        name: "sulfuric_acid",
        chemical_id: "mekanism:sulfuric_acid",
        block_id: "mekanism:sulfuric_acid",
        bucket_id: "mekanism:sulfuric_acid_bucket",
        tag_id: "c:sulfuric_acid",
        tint_color: 0xFF82802B,
        temperature: 300,
        density: 1_840,
        viscosity: 1_840,
        light_level: 0,
        still_texture: "mekanism:liquid/liquid",
        flowing_texture: "mekanism:liquid/liquid_flow",
        is_consumable: false,
    },
    // 7. Hydrogen Chloride (ChemicalConstants.HYDROGEN_CHLORIDE: 0xFFA8F1E9, 188.1K -> 188, density 821.43 -> 821)
    ChemicalFluid {
        name: "hydrogen_chloride",
        chemical_id: "mekanism:hydrogen_chloride",
        block_id: "mekanism:hydrogen_chloride",
        bucket_id: "mekanism:hydrogen_chloride_bucket",
        tag_id: "c:hydrogen_chloride",
        tint_color: 0xFFA8F1E9,
        temperature: 188,
        density: 821,
        viscosity: 821,
        light_level: 0,
        still_texture: "mekanism:liquid/liquid",
        flowing_texture: "mekanism:liquid/liquid_flow",
        is_consumable: false,
    },
    // 8. Hydrofluoric Acid (ChemicalConstants.HYDROFLUORIC_ACID: 0xFFC6C7BD, 189.6K -> 190, density 1150)
    ChemicalFluid {
        name: "hydrofluoric_acid",
        chemical_id: "mekanism:hydrofluoric_acid",
        block_id: "mekanism:hydrofluoric_acid",
        bucket_id: "mekanism:hydrofluoric_acid_bucket",
        tag_id: "c:hydrofluoric_acid",
        tint_color: 0xFFC6C7BD,
        temperature: 190,
        density: 1_150,
        viscosity: 1_150,
        light_level: 0,
        still_texture: "mekanism:liquid/liquid",
        flowing_texture: "mekanism:liquid/liquid_flow",
        is_consumable: false,
    },
    // 9. Uranium Oxide (ChemicalConstants.URANIUM_OXIDE: 0xFFE1F573, 3138.15K -> 3138, density 10970)
    ChemicalFluid {
        name: "uranium_oxide",
        chemical_id: "mekanism:uranium_oxide",
        block_id: "mekanism:uranium_oxide",
        bucket_id: "mekanism:uranium_oxide_bucket",
        tag_id: "c:uranium_oxide",
        tint_color: 0xFFE1F573,
        temperature: 3_138,
        density: 10_970,
        viscosity: 10_970,
        light_level: 0,
        still_texture: "mekanism:liquid/liquid",
        flowing_texture: "mekanism:liquid/liquid_flow",
        is_consumable: false,
    },
    // 10. Uranium Hexafluoride (ChemicalConstants.URANIUM_HEXAFLUORIDE: 0xFF809960, 337.2K -> 337, density 5090)
    ChemicalFluid {
        name: "uranium_hexafluoride",
        chemical_id: "mekanism:uranium_hexafluoride",
        block_id: "mekanism:uranium_hexafluoride",
        bucket_id: "mekanism:uranium_hexafluoride_bucket",
        tag_id: "c:uranium_hexafluoride",
        tint_color: 0xFF809960,
        temperature: 337,
        density: 5_090,
        viscosity: 5_090,
        light_level: 0,
        still_texture: "mekanism:liquid/liquid",
        flowing_texture: "mekanism:liquid/liquid_flow",
        is_consumable: false,
    },
    // 11. Ethene (ChemicalConstants.ETHENE: 0xFFEACCF9, 169.45K -> 169, density 577)
    ChemicalFluid {
        name: "ethene",
        chemical_id: "mekanism:ethene",
        block_id: "mekanism:ethene",
        bucket_id: "mekanism:ethene_bucket",
        tag_id: "c:ethene",
        tint_color: 0xFFEACCF9,
        temperature: 169,
        density: 577,
        viscosity: 577,
        light_level: 0,
        still_texture: "mekanism:liquid/liquid",
        flowing_texture: "mekanism:liquid/liquid_flow",
        is_consumable: false,
    },
    // 12. Sodium (ChemicalConstants.SODIUM: 0xFFE9FEF4, 370.944K -> 371, density 927)
    ChemicalFluid {
        name: "sodium",
        chemical_id: "mekanism:sodium",
        block_id: "mekanism:sodium",
        bucket_id: "mekanism:sodium_bucket",
        tag_id: "c:sodium",
        tint_color: 0xFFE9FEF4,
        temperature: 371,
        density: 927,
        viscosity: 927,
        light_level: 0,
        still_texture: "mekanism:liquid/liquid",
        flowing_texture: "mekanism:liquid/liquid_flow",
        is_consumable: false,
    },
    // 13. Superheated Sodium (ChemicalConstants.SUPERHEATED_SODIUM: 0xFFD19469, 2000.0K -> 2000, density 927)
    ChemicalFluid {
        name: "superheated_sodium",
        chemical_id: "mekanism:superheated_sodium",
        block_id: "mekanism:superheated_sodium",
        bucket_id: "mekanism:superheated_sodium_bucket",
        tag_id: "c:superheated_sodium",
        tint_color: 0xFFD19469,
        temperature: 2_000,
        density: 927,
        viscosity: 927,
        light_level: 0,
        still_texture: "mekanism:liquid/liquid",
        flowing_texture: "mekanism:liquid/liquid_flow",
        is_consumable: false,
    },
    // 14. Brine (MekanismFluids.java:37: tint 0xFFFEEF9C, defaults: temp 300, density 1000, viscosity 1000)
    ChemicalFluid {
        name: "brine",
        chemical_id: "mekanism:brine",
        block_id: "mekanism:brine",
        bucket_id: "mekanism:brine_bucket",
        tag_id: "c:brine",
        tint_color: 0xFFFEEF9C,
        temperature: 300,
        density: 1_000,
        viscosity: 1_000,
        light_level: 0,
        still_texture: "mekanism:liquid/liquid",
        flowing_texture: "mekanism:liquid/liquid_flow",
        is_consumable: false,
    },
    // 15. Lithium (ChemicalConstants.LITHIUM: 0xFFEBA400, 453.65K -> 454, density 512)
    ChemicalFluid {
        name: "lithium",
        chemical_id: "mekanism:lithium",
        block_id: "mekanism:lithium",
        bucket_id: "mekanism:lithium_bucket",
        tag_id: "c:lithium",
        tint_color: 0xFFEBA400,
        temperature: 454,
        density: 512,
        viscosity: 512,
        light_level: 0,
        still_texture: "mekanism:liquid/liquid",
        flowing_texture: "mekanism:liquid/liquid_flow",
        is_consumable: false,
    },
    // 16. Steam (MekanismFluids.java:40-41: temp 373, density 0, textures liquid/steam & liquid/steam_flow)
    ChemicalFluid {
        name: "steam",
        chemical_id: "mekanism:steam",
        block_id: "mekanism:steam",
        bucket_id: "mekanism:steam_bucket",
        tag_id: "c:steam",
        tint_color: 0xFFFFFFFF,
        temperature: 373,
        density: 0,
        viscosity: 0,
        light_level: 0,
        still_texture: "mekanism:liquid/steam",
        flowing_texture: "mekanism:liquid/steam_flow",
        is_consumable: false,
    },
    // 17. Heavy Water (MekanismFluids.java:42-44: tint 0xFF0D1455, vanilla water textures)
    ChemicalFluid {
        name: "heavy_water",
        chemical_id: "mekanism:heavy_water",
        block_id: "mekanism:heavy_water",
        bucket_id: "mekanism:heavy_water_bucket",
        tag_id: "c:heavy_water",
        tint_color: 0xFF0D1455,
        temperature: 300,
        density: 1_000,
        viscosity: 1_000,
        light_level: 0,
        still_texture: "minecraft:block/water_still",
        flowing_texture: "minecraft:block/water_flow",
        is_consumable: false,
    },
    // 18. Nutritional Paste (MekanismFluids.java:45-46: tint 0xFFEB6CA3, ItemNutritionalPasteBucket)
    ChemicalFluid {
        name: "nutritional_paste",
        chemical_id: "mekanism:nutritional_paste",
        block_id: "mekanism:nutritional_paste",
        bucket_id: "mekanism:nutritional_paste_bucket",
        tag_id: "c:nutritional_paste",
        tint_color: 0xFFEB6CA3,
        temperature: 300,
        density: 1_000,
        viscosity: 1_000,
        light_level: 0,
        still_texture: "mekanism:liquid/liquid",
        flowing_texture: "mekanism:liquid/liquid_flow",
        is_consumable: true,
    },
];

/// Finds a chemical fluid by its short name (e.g. "hydrogen").
pub fn by_name(name: &str) -> Option<&'static ChemicalFluid> {
    CHEMICAL_FLUIDS.iter().find(|f| f.name == name)
}

/// Finds a chemical fluid by its bucket item ID (e.g. "mekanism:hydrogen_bucket").
pub fn by_bucket(bucket: &str) -> Option<&'static ChemicalFluid> {
    CHEMICAL_FLUIDS.iter().find(|f| f.bucket_id == bucket)
}

/// Finds a chemical fluid by its in-world block ID (e.g. "mekanism:hydrogen").
pub fn by_block(block: &str) -> Option<&'static ChemicalFluid> {
    CHEMICAL_FLUIDS.iter().find(|f| f.block_id == block)
}

/// Returns true if the given item ID is one of the 18 chemical buckets.
pub fn is_chemical_bucket(item: &str) -> bool {
    by_bucket(item).is_some()
}

/// State of an in-world liquid block (`LiquidBlock` level 0..15).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LiquidLevel {
    /// Source block (`level = 0`). Can be scooped with an empty bucket.
    Source,
    /// Flowing fluid (`level = 1..7`).
    Flowing(u8),
    /// Falling fluid column (`level = 8..15`).
    Falling(u8),
}

impl LiquidLevel {
    /// Decodes a blockstate `level` integer property (0 to 15).
    pub fn from_level(level: u8) -> Self {
        match level {
            0 => LiquidLevel::Source,
            1..=7 => LiquidLevel::Flowing(level),
            _ => LiquidLevel::Falling(level),
        }
    }

    /// Converts to the blockstate integer level.
    pub fn to_level(self) -> u8 {
        match self {
            LiquidLevel::Source => 0,
            LiquidLevel::Flowing(l) => l.clamp(1, 7),
            LiquidLevel::Falling(l) => l.clamp(8, 15),
        }
    }

    /// True if this block is a harvestable source block.
    pub fn is_source(self) -> bool {
        matches!(self, LiquidLevel::Source)
    }
}

/// Result of using an empty bucket on a block in the world.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BucketFillResult {
    /// Collected a fluid source block; yields the filled bucket.
    Success { filled_bucket: &'static str },
    /// Not a collectible fluid source.
    Pass,
}

/// Attempt to fill an empty bucket from a block in the world.
/// Java: `BucketItem.use` -> scoops source block (`level = 0`), converting block to air and returning filled bucket.
pub fn try_fill_bucket(block_id: &str, level: u8) -> BucketFillResult {
    if level != 0 {
        return BucketFillResult::Pass;
    }
    match by_block(block_id) {
        Some(fluid) => BucketFillResult::Success { filled_bucket: fluid.bucket_id },
        None => BucketFillResult::Pass,
    }
}

/// Result of emptying a bucket into the world.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BucketEmptyResult {
    /// Successfully placed a fluid source block into the world.
    Placed { block_id: &'static str, returned_item: &'static str },
    /// Cannot place (blocked target).
    Pass,
}

/// Attempt to empty a chemical bucket at a target position.
/// Java: `BucketItem.emptyContents` -> places `LiquidBlock` (source level 0), leaves empty `minecraft:bucket`.
pub fn try_empty_bucket(bucket_id: &str, can_place: bool) -> BucketEmptyResult {
    if !can_place {
        return BucketEmptyResult::Pass;
    }
    match by_bucket(bucket_id) {
        Some(fluid) => BucketEmptyResult::Placed {
            block_id: fluid.block_id,
            returned_item: EMPTY_BUCKET,
        },
        None => BucketEmptyResult::Pass,
    }
}

/// Nutritional Paste consumption calculation.
/// Java: `ItemNutritionalPasteBucket.java:47,59-69`
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NutritionalConsumption {
    /// How many food points are restored.
    pub food_restored: u32,
    /// Saturation gained.
    pub saturation_gained: f32,
    /// Item remaining in hand after drinking.
    pub remaining_item: &'static str,
}

/// Evaluates drinking nutritional paste from a bucket.
///
/// Rules from `ItemNutritionalPasteBucket.java`:
/// 1. `MekanismUtils.isPlayingMode(player)`: player is survival/adventure (not creative/spectator).
/// 2. `needed = Math.min(20 - food_level, 1000 / 50)` = `Math.min(20 - food_level, 20)`.
/// 3. If `needed > 0`: starts drinking for 32 ticks.
/// 4. When finished: `eat(needed, 0.8F)` and returns `minecraft:bucket`.
/// 5. If `needed == 0` or player is sneaking: acts as a normal fluid bucket placing the fluid into the world.
pub fn evaluate_nutritional_drink(current_food_level: u32, is_playing_mode: bool) -> Option<NutritionalConsumption> {
    if !is_playing_mode {
        return None;
    }
    let max_food = 20u32;
    if current_food_level >= max_food {
        return None;
    }
    let capacity_food = (BUCKET_VOLUME as u32) / NUTRITIONAL_PASTE_MB_PER_FOOD; // 1000 / 50 = 20
    let needed = (max_food - current_food_level).min(capacity_food);
    if needed > 0 {
        Some(NutritionalConsumption {
            food_restored: needed,
            saturation_gained: NUTRITIONAL_PASTE_SATURATION,
            remaining_item: EMPTY_BUCKET,
        })
    } else {
        None
    }
}

/// Rotary Condensentrator 1:1 recipe definition for chemical fluids.
/// Java: `RotaryRecipeProvider.java:26-41`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RotaryRecipe {
    pub chemical: &'static str,
    pub fluid_block: &'static str,
    pub fluid_tag: &'static str,
    pub chemical_amount: u64,
    pub fluid_amount: u64,
}

/// Returns the 1:1 Rotary Condensentrator conversion recipe for the given chemical.
pub fn rotary_recipe(chemical_or_fluid_name: &str) -> Option<RotaryRecipe> {
    let fluid = by_name(chemical_or_fluid_name)
        .or_else(|| CHEMICAL_FLUIDS.iter().find(|f| f.chemical_id == chemical_or_fluid_name || f.block_id == chemical_or_fluid_name))?;
    Some(RotaryRecipe {
        chemical: fluid.chemical_id,
        fluid_block: fluid.block_id,
        fluid_tag: fluid.tag_id,
        chemical_amount: 1,
        fluid_amount: 1,
    })
}

// -------------------------------------------------------------------------------------------------
// FORK Requirements (isolated host call stubs and declarations)
// -------------------------------------------------------------------------------------------------

/// Host interface for registering custom fluids and fluid types in the Horizon engine.
///
/// WIT proposed in `FORK_robit_quimicos.md`:
/// ```wit
/// record fluid-spec {
///     name: string,
///     tint-color: u32,
///     temperature: u32,
///     density: s32,
///     viscosity: s32,
///     light-level: u8,
///     still-texture: string,
///     flowing-texture: string,
///     bucket-item: string,
/// }
/// register-fluid: func(spec: fluid-spec) -> result<_, string>;
/// ```
// FORK-26
pub fn host_register_chemical_fluid(fluid: &ChemicalFluid) -> Result<(), &'static str> {
    // Thin host call stub marked for fork implementation
    let _ = fluid;
    Ok(())
}

/// Host interface for consumable items that derive from bucket items with custom duration, hunger and animation.
///
/// WIT proposed in `FORK_robit_quimicos.md`:
/// ```wit
/// record consumable-bucket-spec {
///     item: string,
///     use-duration-ticks: u32,
///     use-animation: string, // "drink"
///     mb-per-food: u32,
///     saturation: f32,
///     returned-empty-item: string,
/// }
/// register-consumable-bucket: func(spec: consumable-bucket-spec) -> result<_, string>;
/// ```
// FORK-29
pub fn host_register_consumable_bucket(
    item_id: &str,
    use_duration: u32,
    mb_per_food: u32,
    saturation: f32,
    returned_empty_item: &str,
) -> Result<(), &'static str> {
    let _ = (item_id, use_duration, mb_per_food, saturation, returned_empty_item);
    Ok(())
}

// -------------------------------------------------------------------------------------------------
// Unit Tests (verifying exact 1:1 match with Java constants and lines)
// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_eighteen_chemical_fluids_registered() {
        assert_eq!(CHEMICAL_FLUIDS.len(), 18);
    }

    #[test]
    fn chemical_constants_match_java_definitions() {
        // ChemicalConstants.java:12: HYDROGEN("hydrogen", 0xFFFFFFFF, 0, 20.28F, 70.85F)
        let hydrogen = by_name("hydrogen").expect("hydrogen must exist");
        assert_eq!(hydrogen.chemical_id, "mekanism:hydrogen");
        assert_eq!(hydrogen.bucket_id, "mekanism:hydrogen_bucket");
        assert_eq!(hydrogen.tint_color, 0xFFFFFFFF);
        assert_eq!(hydrogen.temperature, 20); // Math.round(20.28F) = 20
        assert_eq!(hydrogen.density, 71); // Math.round(70.85F) = 71
        assert_eq!(hydrogen.viscosity, 71);

        // ChemicalConstants.java:13: OXYGEN("oxygen", 0xFF6CE2FF, 0, 90.19F, 1_141)
        let oxygen = by_name("oxygen").expect("oxygen must exist");
        assert_eq!(oxygen.tint_color, 0xFF6CE2FF);
        assert_eq!(oxygen.temperature, 90);
        assert_eq!(oxygen.density, 1141);

        // ChemicalConstants.java:14: CHLORINE("chlorine", 0xFFCFE800, 0, 207.15F, 1_422.92F)
        let chlorine = by_name("chlorine").expect("chlorine must exist");
        assert_eq!(chlorine.tint_color, 0xFFCFE800);
        assert_eq!(chlorine.temperature, 207);
        assert_eq!(chlorine.density, 1423);

        // ChemicalConstants.java:15: SULFUR_DIOXIDE("sulfur_dioxide", 0xFFA99D90, 0, 263.05F, 1_400)
        let so2 = by_name("sulfur_dioxide").expect("sulfur_dioxide must exist");
        assert_eq!(so2.tint_color, 0xFFA99D90);
        assert_eq!(so2.temperature, 263);
        assert_eq!(so2.density, 1400);

        // ChemicalConstants.java:16: SULFUR_TRIOXIDE("sulfur_trioxide", 0xFFCE6C6C, 0, 318, 1_920)
        let so3 = by_name("sulfur_trioxide").expect("sulfur_trioxide must exist");
        assert_eq!(so3.tint_color, 0xFFCE6C6C);
        assert_eq!(so3.temperature, 318);
        assert_eq!(so3.density, 1920);

        // ChemicalConstants.java:19: SULFURIC_ACID("sulfuric_acid", 0xFF82802B, 0, 300, 1_840)
        let h2so4 = by_name("sulfuric_acid").expect("sulfuric_acid must exist");
        assert_eq!(h2so4.tint_color, 0xFF82802B);
        assert_eq!(h2so4.temperature, 300);
        assert_eq!(h2so4.density, 1840);

        // ChemicalConstants.java:20: HYDROGEN_CHLORIDE("hydrogen_chloride", 0xFFA8F1E9, 0, 188.1F, 821.43F)
        let hcl = by_name("hydrogen_chloride").expect("hydrogen_chloride must exist");
        assert_eq!(hcl.tint_color, 0xFFA8F1E9);
        assert_eq!(hcl.temperature, 188);
        assert_eq!(hcl.density, 821);

        // ChemicalConstants.java:27: HYDROFLUORIC_ACID("hydrofluoric_acid", 0xFFC6C7BD, 0, 189.6F, 1_150)
        let hf = by_name("hydrofluoric_acid").expect("hydrofluoric_acid must exist");
        assert_eq!(hf.tint_color, 0xFFC6C7BD);
        assert_eq!(hf.temperature, 190);
        assert_eq!(hf.density, 1150);

        // ChemicalConstants.java:28: URANIUM_OXIDE("uranium_oxide", 0xFFE1F573, 0, 3138.15F, 10_970)
        let uo = by_name("uranium_oxide").expect("uranium_oxide must exist");
        assert_eq!(uo.tint_color, 0xFFE1F573);
        assert_eq!(uo.temperature, 3138);
        assert_eq!(uo.density, 10970);

        // ChemicalConstants.java:29: URANIUM_HEXAFLUORIDE("uranium_hexafluoride", 0xFF809960, 0, 337.2F, 5_090)
        let uf6 = by_name("uranium_hexafluoride").expect("uranium_hexafluoride must exist");
        assert_eq!(uf6.tint_color, 0xFF809960);
        assert_eq!(uf6.temperature, 337);
        assert_eq!(uf6.density, 5090);

        // ChemicalConstants.java:21: ETHENE("ethene", 0xFFEACCF9, 0, 169.45F, 577)
        let ethene = by_name("ethene").expect("ethene must exist");
        assert_eq!(ethene.tint_color, 0xFFEACCF9);
        assert_eq!(ethene.temperature, 169);
        assert_eq!(ethene.density, 577);

        // ChemicalConstants.java:23: SODIUM("sodium", 0xFFE9FEF4, 0, 370.944F, 927)
        let sodium = by_name("sodium").expect("sodium must exist");
        assert_eq!(sodium.tint_color, 0xFFE9FEF4);
        assert_eq!(sodium.temperature, 371);
        assert_eq!(sodium.density, 927);

        // ChemicalConstants.java:24: SUPERHEATED_SODIUM("superheated_sodium", 0xFFD19469, 0, 2_000.0F, 927)
        let super_na = by_name("superheated_sodium").expect("superheated_sodium must exist");
        assert_eq!(super_na.tint_color, 0xFFD19469);
        assert_eq!(super_na.temperature, 2000);
        assert_eq!(super_na.density, 927);

        // MekanismFluids.java:37: BRINE -> tint 0xFFFEEF9C
        let brine = by_name("brine").expect("brine must exist");
        assert_eq!(brine.tint_color, 0xFFFEEF9C);
        assert_eq!(brine.temperature, 300);
        assert_eq!(brine.density, 1000);

        // ChemicalConstants.java:26: LITHIUM("lithium", 0xFFEBA400, 0, 453.65F, 512)
        let lithium = by_name("lithium").expect("lithium must exist");
        assert_eq!(lithium.tint_color, 0xFFEBA400);
        assert_eq!(lithium.temperature, 454);
        assert_eq!(lithium.density, 512);

        // MekanismFluids.java:40-41: STEAM -> temp 373, density 0
        let steam = by_name("steam").expect("steam must exist");
        assert_eq!(steam.temperature, 373);
        assert_eq!(steam.density, 0);
        assert_eq!(steam.still_texture, "mekanism:liquid/steam");
        assert_eq!(steam.flowing_texture, "mekanism:liquid/steam_flow");

        // MekanismFluids.java:42-44: HEAVY_WATER -> tint 0xFF0D1455, water textures
        let hw = by_name("heavy_water").expect("heavy_water must exist");
        assert_eq!(hw.tint_color, 0xFF0D1455);
        assert_eq!(hw.still_texture, "minecraft:block/water_still");

        // MekanismFluids.java:45-46: NUTRITIONAL_PASTE -> tint 0xFFEB6CA3, consumable
        let paste = by_name("nutritional_paste").expect("nutritional_paste must exist");
        assert_eq!(paste.tint_color, 0xFFEB6CA3);
        assert!(paste.is_consumable);
    }

    #[test]
    fn bucket_fill_and_empty_behavior() {
        // Empty bucket on a hydrogen source block (level 0) fills hydrogen bucket
        let fill = try_fill_bucket("mekanism:hydrogen", 0);
        assert_eq!(fill, BucketFillResult::Success { filled_bucket: "mekanism:hydrogen_bucket" });

        // Empty bucket on a flowing hydrogen block (level 3) passes (no harvest)
        let flowing_fill = try_fill_bucket("mekanism:hydrogen", 3);
        assert_eq!(flowing_fill, BucketFillResult::Pass);

        // Emptying hydrogen bucket in world places source block and returns minecraft:bucket
        let empty = try_empty_bucket("mekanism:hydrogen_bucket", true);
        assert_eq!(empty, BucketEmptyResult::Placed {
            block_id: "mekanism:hydrogen",
            returned_item: "minecraft:bucket",
        });

        // Unknown bucket passes
        let unknown = try_empty_bucket("minecraft:water_bucket", true);
        assert_eq!(unknown, BucketEmptyResult::Pass);
    }

    #[test]
    fn nutritional_paste_bucket_drinking_rules() {
        // Java: ItemNutritionalPasteBucket.java:47,59-69
        // When player is hungry (food_level = 5) in survival mode (is_playing_mode = true):
        // needed = min(20 - 5, 20) = 15 points
        let res = evaluate_nutritional_drink(5, true).expect("should drink");
        assert_eq!(res.food_restored, 15);
        assert_eq!(res.saturation_gained, 0.8f32);
        assert_eq!(res.remaining_item, "minecraft:bucket");

        // When player is completely starving (food_level = 0):
        // restores full 20 food points
        let starved = evaluate_nutritional_drink(0, true).expect("should drink full");
        assert_eq!(starved.food_restored, 20);

        // When player is nearly full (food_level = 19):
        // restores only 1 food point
        let nearly_full = evaluate_nutritional_drink(19, true).expect("should drink 1");
        assert_eq!(nearly_full.food_restored, 1);

        // When player is full (food_level = 20):
        // cannot drink (acts as normal bucket placement)
        assert_eq!(evaluate_nutritional_drink(20, true), None);

        // In creative mode (is_playing_mode = false):
        // cannot drink
        assert_eq!(evaluate_nutritional_drink(10, false), None);
    }

    #[test]
    fn rotary_condensentrator_one_to_one_conversion_recipes() {
        // Java: RotaryRecipeProvider.java:26-41
        for fluid in &CHEMICAL_FLUIDS {
            let recipe = rotary_recipe(fluid.name).expect("rotary recipe must exist for each chemical");
            assert_eq!(recipe.chemical, fluid.chemical_id);
            assert_eq!(recipe.fluid_block, fluid.block_id);
            assert_eq!(recipe.fluid_tag, fluid.tag_id);
            assert_eq!(recipe.chemical_amount, 1);
            assert_eq!(recipe.fluid_amount, 1);
        }
    }

    #[test]
    fn liquid_level_parsing_and_source_check() {
        assert_eq!(LiquidLevel::from_level(0), LiquidLevel::Source);
        assert!(LiquidLevel::from_level(0).is_source());
        assert_eq!(LiquidLevel::from_level(4), LiquidLevel::Flowing(4));
        assert!(!LiquidLevel::from_level(4).is_source());
        assert_eq!(LiquidLevel::from_level(10), LiquidLevel::Falling(10));
        assert!(!LiquidLevel::from_level(10).is_source());
    }
}

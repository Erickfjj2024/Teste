//! Fuelwood Heater (`TileEntityFuelwoodHeater`, `FuelInventorySlot`):
//! Burns solid fuels (coal, wood, blaze rods, lava buckets) to generate thermal energy.
//!
//! Mekanism 1.21.1 source of truth:
//! - `mekanism.common.tile.machine.TileEntityFuelwoodHeater` (`TileEntityFuelwoodHeater.java`)
//! - `mekanism.client.gui.machine.GuiFuelwoodHeater` (`GuiFuelwoodHeater.java`)
//! - `mekanism.common.inventory.slot.FuelInventorySlot` (`FuelInventorySlot.java`)
//! - `mekanism.common.config.GeneralConfig` (`GeneralConfig.java`)

use super::*;
use crate::heat::{HeatCapacitor, AIR_INVERSE_COEFFICIENT, AMBIENT_TEMP};

// ============================================================================
// Constants (TileEntityFuelwoodHeater.java:31-33 & GeneralConfig.java:145-147)
// ============================================================================

pub const BLOCK: &str = "mekanism:fuelwood_heater";

/// Heat generated per fuel tick in Joules.
/// Java: `GeneralConfig.heatPerFuelTick = 400.0` (GeneralConfig.java:145).
pub const HEAT_PER_FUEL_TICK: f64 = 400.0;

/// Number of fuel burn ticks consumed per game tick.
/// Java: `GeneralConfig.fuelwoodTickMultiplier = 1` (GeneralConfig.java:147).
pub const FUELWOOD_TICK_MULTIPLIER: u32 = 1;

/// Heat capacity in Joules / Kelvin.
/// Java: `TileEntityFuelwoodHeater.HEAT_CAPACITY = 100` (TileEntityFuelwoodHeater.java:31).
pub const HEAT_CAPACITY: f64 = 100.0;

/// Inverse conduction coefficient.
/// Java: `TileEntityFuelwoodHeater.INVERSE_CONDUCTION_COEFFICIENT = 5` (TileEntityFuelwoodHeater.java:32).
pub const INVERSE_CONDUCTION: f64 = 5.0;

/// Inverse insulation coefficient.
/// Java: `TileEntityFuelwoodHeater.INVERSE_INSULATION_COEFFICIENT = 10` (TileEntityFuelwoodHeater.java:33).
pub const INVERSE_INSULATION: f64 = 10.0;

/// Light level emitted while burning.
pub const ACTIVE_LIGHT_LEVEL: u8 = 8;

// ============================================================================
// Standard Fuel Burn Durations (Furnace / Forge standard burn times)
// ============================================================================

/// Returns standard burn duration in ticks for furnace fuels.
/// Java: `Vanilla ItemStack.getBurnTime(null)`.
pub fn fuel_burn_duration(item: &str) -> Option<u32> {
    // The original asks the game's fuel registry (`FuelInventorySlot` -> `ItemStack.getBurnTime`); the port cannot, so this is the vanilla 1.21.1 table
    // for the common fuels (FORK-32: `item-types.burn-time(item)` makes it exact, with data-pack and other-mod fuels).
    let name = item.strip_prefix("minecraft:").unwrap_or(item);
    // Only the Overworld woods burn: crimson and warped (Nether) planks, stems and slabs do not.
    const WOODS: [&str; 8] = ["oak", "spruce", "birch", "jungle", "acacia", "dark_oak", "mangrove", "cherry"];
    let wood = |s: &str| {
        let s = s.strip_prefix("stripped_").unwrap_or(s);
        WOODS.iter().chain(["bamboo"].iter()).any(|w| s.strip_prefix(w).is_some_and(|rest| rest.starts_with('_')))
    };
    match name {
        "lava_bucket" => Some(20_000),
        "coal_block" => Some(16_000),
        "dried_kelp_block" => Some(4_000),
        "blaze_rod" => Some(2_400),
        "coal" | "charcoal" => Some(1_600),
        "stick" | "bowl" | "wooden_button" => Some(100),
        // Mekanism's own fuels (`MekanismDataMapsProvider`: `FURNACE_FUELS`; bio fuel is 5 seconds, its block ten times that).
        "mekanism:charcoal_block" => Some(16_000),
        "mekanism:bio_fuel" => Some(100),
        "mekanism:bio_fuel_block" => Some(1_000),
        "scaffolding" | "bamboo" => Some(50),
        s if s.ends_with("_sapling") => Some(100),
        s if wood(s) && (s.ends_with("_planks") || s.ends_with("_log") || s.ends_with("_wood") || s.ends_with("_block") || s.ends_with("_mosaic")) => Some(300),
        s if wood(s) && (s.ends_with("_stairs") || s.ends_with("_fence") || s.ends_with("_fence_gate") || s.ends_with("_trapdoor") || s.ends_with("_pressure_plate")) => Some(300),
        s if wood(s) && s.ends_with("_slab") => Some(150),
        s if wood(s) && s.ends_with("_door") => Some(200),
        s if wood(s) && (s.ends_with("_boat") || s.ends_with("_chest_boat")) => Some(1_200),
        _ => None,
    }
}

// ============================================================================
// Fuelwood Heater Structure
// ============================================================================

#[derive(Clone, Debug, PartialEq)]
pub struct FuelwoodHeater {
    /// Ticks remaining for the currently burning fuel item.
    pub burn_time: u32,
    /// Total burn duration of the consumed item (for GUI flame progress).
    pub max_burn_time: u32,
    /// Fuel item in input slot 0 (item name and count).
    pub fuel_slot_item: Option<String>,
    pub fuel_slot_count: u32,
    /// Internal heat capacitor.
    pub capacitor: HeatCapacitor,
    /// Facing orientation (horizontal: 2: North, 3: South, 4: West, 5: East).
    pub facing: usize,
    /// Active combustion status.
    pub active: bool,
    /// Heat transferred to adjacent acceptors in previous tick.
    pub last_transfer_loss: f64,
    /// Heat lost to air in previous tick.
    pub last_environment_loss: f64,
}

impl Default for FuelwoodHeater {
    fn default() -> Self {
        Self::new(AMBIENT_TEMP)
    }
}

impl FuelwoodHeater {
    pub fn new(ambient: f64) -> Self {
        let capacitor = HeatCapacitor::new(
            HEAT_CAPACITY,
            INVERSE_CONDUCTION,
            INVERSE_INSULATION,
            ambient,
        );
        FuelwoodHeater {
            burn_time: 0,
            max_burn_time: 0,
            fuel_slot_item: None,
            fuel_slot_count: 0,
            capacitor,
            facing: 2, // North
            active: false,
            last_transfer_loss: 0.0,
            last_environment_loss: 0.0,
        }
    }

    /// Flame progress ratio between 0.0 and 1.0.
    /// Java: `GuiFuelwoodHeater.java:38`: `burnTime / (double) maxBurnTime`.
    pub fn flame_progress(&self) -> f64 {
        if self.max_burn_time > 0 {
            (self.burn_time as f64) / (self.max_burn_time as f64)
        } else {
            0.0
        }
    }

    /// Sets fuel in the inventory slot (slot 0).
    pub fn set_fuel(&mut self, item: &str, count: u32) {
        if count == 0 {
            self.fuel_slot_item = None;
            self.fuel_slot_count = 0;
        } else {
            self.fuel_slot_item = Some(item.to_owned());
            self.fuel_slot_count = count;
        }
    }

    /// Simulates one server tick:
    /// 1. Consumes new fuel if burn_time is 0.
    /// 2. Advances burning, generates 400.0 J per fuel tick.
    /// 3. Dissipates to environment (air on 6 faces).
    /// 4. Conduction to adjacent heat acceptors.
    ///
    /// Java reference: `TileEntityFuelwoodHeater.java:67-84`.
    pub fn tick(
        &mut self,
        ambient: f64,
        adjacent_sinks: &mut [HeatCapacitor],
    ) -> bool {
        // Step 1: Ignite new fuel if currently idle
        // Java: TileEntityFuelwoodHeater.java:69-71:
        // if (burnTime == 0) { maxBurnTime = burnTime = fuelSlot.burn(); }
        if self.burn_time == 0 {
            if let Some(ref item) = self.fuel_slot_item {
                if let Some(duration) = fuel_burn_duration(item) {
                    // `FuelInventorySlot.burn`: the heater burns half of the fuel's burn time (`getBurnTime(null) / 2`); an item that
                    // gives zero is not consumed, and a container item (a lava bucket) in a stack of more than one does not burn at all.
                    let burn = duration / 2;
                    let container_stack = item == "minecraft:lava_bucket" && self.fuel_slot_count > 1;
                    if burn != 0 && self.fuel_slot_count > 0 && !container_stack {
                        self.burn_time = burn;
                        self.max_burn_time = burn;

                        // Decrement fuel or handle bucket
                        if item == "minecraft:lava_bucket" {
                            self.fuel_slot_item = Some("minecraft:bucket".to_owned());
                        } else {
                            self.fuel_slot_count -= 1;
                            if self.fuel_slot_count == 0 {
                                self.fuel_slot_item = None;
                            }
                        }
                    }
                }
            }
        }

        // Step 2: Burn progress and heat generation
        // Java: TileEntityFuelwoodHeater.java:72-79
        if self.burn_time > 0 {
            let ticks = self.burn_time.min(FUELWOOD_TICK_MULTIPLIER);
            self.burn_time -= ticks;
            let heat_generated = HEAT_PER_FUEL_TICK * (ticks as f64);
            self.capacitor.handle_heat(heat_generated);
            self.active = true;
        } else {
            self.active = false;
        }

        // Step 3: Environmental dissipation across all 6 faces (ITileHeatHandler.java:47-62)
        let inv_air = AIR_INVERSE_COEFFICIENT + INVERSE_INSULATION + INVERSE_CONDUCTION;
        let temp = self.capacitor.temperature();
        let mut env_loss = 0.0;

        for _ in 0..6 {
            let delta_t = (temp - ambient) / inv_air;
            let q = delta_t * HEAT_CAPACITY;
            self.capacitor.handle_heat(-q);
            if delta_t > 0.0 {
                env_loss += delta_t;
            }
        }
        self.last_environment_loss = env_loss;

        // Step 4: Adjacent conduction to connected sinks (ITileHeatHandler.java:64-100)
        let mut adj_loss = 0.0;
        for sink in adjacent_sinks.iter_mut() {
            let total_inv = sink.inverse_conduction + INVERSE_CONDUCTION;
            let delta_t = (temp - ambient) / total_inv;
            let q = delta_t * HEAT_CAPACITY;

            self.capacitor.handle_heat(-q);
            sink.handle_heat(q);

            // `ITileHeatHandler.incrementAdjacentTransfer` adds the amount whatever its sign.
            adj_loss += delta_t;
        }
        self.last_transfer_loss = adj_loss;

        // Commit queued heat
        self.capacitor.update();

        self.active
    }

    /// Serializes state to bytes:
    /// byte 0: version (1)
    /// byte 1: facing (0..=5)
    /// byte 2: active (0/1)
    /// bytes 3..7: burn_time (u32 LE)
    /// bytes 7..11: max_burn_time (u32 LE)
    /// bytes 11..19: stored_heat (f64 LE)
    /// bytes 19..23: fuel_slot_count (u32 LE)
    /// byte 23: item name len (N)
    /// bytes 24..24+N: item name string
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(32);
        out.push(1); // version
        out.push(self.facing as u8);
        out.push(u8::from(self.active));
        out.extend(self.burn_time.to_le_bytes());
        out.extend(self.max_burn_time.to_le_bytes());
        out.extend(self.capacitor.stored_heat.to_le_bytes());
        out.extend(self.fuel_slot_count.to_le_bytes());

        if let Some(ref item) = self.fuel_slot_item {
            let bytes = item.as_bytes();
            let len = bytes.len().min(255) as u8;
            out.push(len);
            out.extend_from_slice(&bytes[..len as usize]);
        } else {
            out.push(0);
        }
        out
    }

    /// Deserializes state from bytes.
    pub fn decode(data: &[u8], ambient: f64) -> Option<Self> {
        if data.len() < 24 || data[0] != 1 {
            return None;
        }
        let facing = usize::from(data[1]).min(5);
        let active = data[2] == 1;

        let mut u32_buf = [0u8; 4];
        u32_buf.copy_from_slice(&data[3..7]);
        let burn_time = u32::from_le_bytes(u32_buf);

        u32_buf.copy_from_slice(&data[7..11]);
        let max_burn_time = u32::from_le_bytes(u32_buf);

        let mut f64_buf = [0u8; 8];
        f64_buf.copy_from_slice(&data[11..19]);
        let stored_heat = f64::from_le_bytes(f64_buf);

        u32_buf.copy_from_slice(&data[19..23]);
        let fuel_slot_count = u32::from_le_bytes(u32_buf);

        let item_len = data[23] as usize;
        let fuel_slot_item = if item_len > 0 && data.len() >= 24 + item_len {
            std::str::from_utf8(&data[24..24 + item_len]).ok().map(|s| s.to_owned())
        } else {
            None
        };

        let mut capacitor = HeatCapacitor::new(
            HEAT_CAPACITY,
            INVERSE_CONDUCTION,
            INVERSE_INSULATION,
            ambient,
        );
        capacitor.set_heat(stored_heat);

        Some(FuelwoodHeater {
            burn_time,
            max_burn_time,
            fuel_slot_item,
            fuel_slot_count,
            capacitor,
            facing,
            active,
            last_transfer_loss: 0.0,
            last_environment_loss: 0.0,
        })
    }

    /// Prepares data slots for GUI container tracking.
    /// Java: `TileEntityFuelwoodHeater.java:111-117`.
    pub fn menu_data(&self) -> Vec<i32> {
        let temp_fixed = (self.capacitor.temperature() * 100.0) as i64;
        vec![
            self.burn_time as i32,
            self.max_burn_time as i32,
            (temp_fixed & 0xFFFF) as i32,
            ((temp_fixed >> 16) & 0xFFFF) as i32,
            self.active as i32,
        ]
    }
}

// ============================================================================
// Unit Tests (comparing with Java source files and line numbers)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuelwood_heater_constants_match_java() {
        // TileEntityFuelwoodHeater.java:31: HEAT_CAPACITY = 100
        assert_eq!(HEAT_CAPACITY, 100.0);
        // TileEntityFuelwoodHeater.java:32: INVERSE_CONDUCTION_COEFFICIENT = 5
        assert_eq!(INVERSE_CONDUCTION, 5.0);
        // TileEntityFuelwoodHeater.java:33: INVERSE_INSULATION_COEFFICIENT = 10
        assert_eq!(INVERSE_INSULATION, 10.0);

        // GeneralConfig.java:145: heatPerFuelTick = 400
        assert_eq!(HEAT_PER_FUEL_TICK, 400.0);
        // GeneralConfig.java:147: fuelwoodTickMultiplier = 1
        assert_eq!(FUELWOOD_TICK_MULTIPLIER, 1);
    }

    #[test]
    fn test_standard_fuel_durations() {
        // Coal: 1600 ticks
        assert_eq!(fuel_burn_duration("minecraft:coal"), Some(1_600));
        assert_eq!(fuel_burn_duration("minecraft:charcoal"), Some(1_600));

        // Blaze Rod: 2400 ticks
        assert_eq!(fuel_burn_duration("minecraft:blaze_rod"), Some(2_400));

        // Logs and Planks: 300 ticks
        assert_eq!(fuel_burn_duration("minecraft:oak_planks"), Some(300));
        assert_eq!(fuel_burn_duration("minecraft:birch_log"), Some(300));

        // Coal Block: 16000 ticks
        assert_eq!(fuel_burn_duration("minecraft:coal_block"), Some(16_000));
    }

    #[test]
    fn java_burn_time_is_half_and_mekanism_fuels_are_known() {
        // `FuelInventorySlot.java:burn`: getBurnTime(null) / 2. Bio fuel: 5 * 20 = 100 ticks (block: 1000), charcoal block 16000
        // (`MekanismDataMapsProvider.java:52-57`); a stick burns 100, so the heater gets 50.
        assert_eq!(fuel_burn_duration("mekanism:bio_fuel"), Some(100));
        assert_eq!(fuel_burn_duration("mekanism:bio_fuel_block"), Some(1_000));
        assert_eq!(fuel_burn_duration("mekanism:charcoal_block"), Some(16_000));
        assert_eq!(fuel_burn_duration("minecraft:stick").map(|d| d / 2), Some(50));
        // Nether woods do not burn.
        assert_eq!(fuel_burn_duration("minecraft:crimson_planks"), None);
        assert_eq!(fuel_burn_duration("minecraft:stone_slab"), None);
    }

    #[test]
    fn test_burning_coal_yields_expected_heat() {
        // 1 coal burns for 1600 ticks, producing 1600 * 400.0 = 640,000 J of heat
        let mut heater = FuelwoodHeater::new(300.0);
        heater.set_fuel("minecraft:coal", 1);

        let initial_heat = heater.capacitor.stored_heat;
        assert_eq!(initial_heat, 30_000.0); // 100.0 * 300.0 K

        let mut sinks = [];
        // First tick ignites fuel and starts burning
        heater.tick(300.0, &mut sinks);

        assert!(heater.active);
        // `FuelInventorySlot.burn`: coal burns 1600 ticks in a furnace, the heater takes half: 800; one tick consumed.
        assert_eq!(heater.max_burn_time, 800);
        assert_eq!(heater.burn_time, 799);
        assert_eq!(heater.fuel_slot_count, 0); // Consumed 1 item
        assert!(heater.capacitor.temperature() > 300.0);
    }

    #[test]
    fn test_flame_progress_calculation() {
        let mut heater = FuelwoodHeater::new(300.0);
        heater.burn_time = 800;
        heater.max_burn_time = 1600;

        // GuiFuelwoodHeater.java:38: burnTime / (double) maxBurnTime
        assert!((heater.flame_progress() - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_fuelwood_heater_serialization_roundtrip() {
        let mut heater = FuelwoodHeater::new(300.0);
        heater.facing = 3; // South
        heater.burn_time = 1_234;
        heater.max_burn_time = 1_600;
        heater.set_fuel("minecraft:oak_planks", 15);
        heater.capacitor.set_heat(42_000.0);
        heater.active = true;

        let bytes = heater.encode();
        let decoded = FuelwoodHeater::decode(&bytes, 300.0).expect("decode failed");

        assert_eq!(decoded.facing, 3);
        assert!(decoded.active);
        assert_eq!(decoded.burn_time, 1_234);
        assert_eq!(decoded.max_burn_time, 1_600);
        assert_eq!(decoded.fuel_slot_item.as_deref(), Some("minecraft:oak_planks"));
        assert_eq!(decoded.fuel_slot_count, 15);
        assert!((decoded.capacitor.stored_heat - 42_000.0).abs() < 1e-6);
    }
}

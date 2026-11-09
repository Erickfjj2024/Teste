//! Resistive Heater (`TileEntityResistiveHeater`, `ResistiveHeaterEnergyContainer`):
//! Consumes electric energy (FE / Joules) to generate heat with configurable usage and 60% thermal efficiency.
//!
//! Mekanism 1.21.1 source of truth:
//! - `mekanism.common.tile.machine.TileEntityResistiveHeater` (`TileEntityResistiveHeater.java`)
//! - `mekanism.common.capabilities.energy.ResistiveHeaterEnergyContainer` (`ResistiveHeaterEnergyContainer.java`)
//! - `mekanism.client.gui.machine.GuiResistiveHeater` (`GuiResistiveHeater.java`)
//! - `mekanism.common.config.GeneralConfig` (`GeneralConfig.java`)

use super::*;
use crate::heat::{HeatCapacitor, AIR_INVERSE_COEFFICIENT, AMBIENT_TEMP, EPSILON};

// ============================================================================
// Constants (TileEntityResistiveHeater.java:46-50 & GeneralConfig.java:149)
// ============================================================================

pub const BLOCK: &str = "mekanism:resistive_heater";

/// Base usage in Joules per tick.
/// Java: `TileEntityResistiveHeater.BASE_USAGE = 100L` (TileEntityResistiveHeater.java:50).
pub const BASE_USAGE_JOULES: u64 = 100;

/// Base usage converted to Forge Energy: `fe(100) = 40 FE/t`.
pub const BASE_USAGE_FE: u64 = fe(BASE_USAGE_JOULES);

/// Buffer capacity multiplier in ticks (20 seconds of buffer).
/// Java: `ResistiveHeaterEnergyContainer.USAGE_MULTIPLIER = 20 * 20 = 400` (ResistiveHeaterEnergyContainer.java:22).
pub const USAGE_MULTIPLIER: u64 = 400;

/// Efficiency of electrical to thermal conversion: 60%.
/// Java: `GeneralConfig.resistiveHeaterEfficiency = 0.6` (GeneralConfig.java:149).
pub const EFFICIENCY: f64 = 0.6;

/// Heat capacity in Joules / Kelvin.
/// Java: `TileEntityResistiveHeater.HEAT_CAPACITY = 100` (TileEntityResistiveHeater.java:46).
pub const HEAT_CAPACITY: f64 = 100.0;

/// Inverse conduction coefficient.
/// Java: `TileEntityResistiveHeater.INVERSE_CONDUCTION_COEFFICIENT = 5` (TileEntityResistiveHeater.java:47).
pub const INVERSE_CONDUCTION: f64 = 5.0;

/// Inverse insulation coefficient.
/// Java: `TileEntityResistiveHeater.INVERSE_INSULATION_COEFFICIENT = 10` (TileEntityResistiveHeater.java:48).
pub const INVERSE_INSULATION: f64 = 10.0;

/// Emitted light level when active.
pub const ACTIVE_LIGHT_LEVEL: u8 = 8;

// ============================================================================
// Redstone Control
// ============================================================================

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RedstoneMode {
    #[default]
    Ignored = 0,
    Normal = 1,
    Inverted = 2,
}

impl RedstoneMode {
    pub fn can_function(self, powered: bool) -> bool {
        match self {
            RedstoneMode::Ignored => true,
            RedstoneMode::Normal => powered,
            RedstoneMode::Inverted => !powered,
        }
    }

    pub fn next(self) -> Self {
        match self {
            RedstoneMode::Ignored => RedstoneMode::Normal,
            RedstoneMode::Normal => RedstoneMode::Inverted,
            RedstoneMode::Inverted => RedstoneMode::Ignored,
        }
    }

    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => RedstoneMode::Normal,
            2 => RedstoneMode::Inverted,
            _ => RedstoneMode::Ignored,
        }
    }
}

// ============================================================================
// Resistive Heater Structure
// ============================================================================

#[derive(Clone, Debug, PartialEq)]
pub struct ResistiveHeater {
    /// Configured energy consumption rate in Joules per tick.
    pub energy_usage_joules: u64,
    /// Energy currently stored in Forge Energy.
    pub energy_stored_fe: u64,
    /// Internal heat capacitor.
    pub capacitor: HeatCapacitor,
    /// Facing orientation (0..=5; horizontal faces: 2: North, 3: South, 4: West, 5: East).
    pub facing: usize,
    /// Active heating status.
    pub active: bool,
    /// Redstone control setting.
    pub redstone: RedstoneMode,
    /// Energy consumed in the last tick in FE.
    pub client_energy_used_fe: u64,
    /// Sound scale factor for volume modulation: `toUseJoules / 100_000.0`.
    /// Java: `TileEntityResistiveHeater.java:108`.
    pub sound_scale: f32,
    /// Heat transferred to neighbors in previous tick.
    pub last_transfer_loss: f64,
    /// Heat lost to environment in previous tick.
    pub last_environment_loss: f64,
}

impl Default for ResistiveHeater {
    fn default() -> Self {
        Self::new(AMBIENT_TEMP)
    }
}

impl ResistiveHeater {
    pub fn new(ambient: f64) -> Self {
        let capacitor = HeatCapacitor::new(
            HEAT_CAPACITY,
            INVERSE_CONDUCTION,
            INVERSE_INSULATION,
            ambient,
        );
        ResistiveHeater {
            energy_usage_joules: BASE_USAGE_JOULES,
            energy_stored_fe: 0,
            capacitor,
            facing: 2, // North
            active: false,
            redstone: RedstoneMode::Ignored,
            client_energy_used_fe: 0,
            sound_scale: 0.0,
            last_transfer_loss: 0.0,
            last_environment_loss: 0.0,
        }
    }

    /// Current usage rate converted to Forge Energy.
    pub fn usage_fe(&self) -> u64 {
        fe(self.energy_usage_joules)
    }

    /// Maximum energy buffer capacity in Forge Energy: `fe(energy_usage * 400)`.
    /// Java: `ResistiveHeaterEnergyContainer.java:41`.
    pub fn max_energy_fe(&self) -> u64 {
        fe(self.energy_usage_joules.saturating_mul(USAGE_MULTIPLIER))
    }

    /// Updates configured energy usage (up to 7 digits in GUI text field).
    /// Java: `TileEntityResistiveHeater.java:131-134` & `ResistiveHeaterEnergyContainer.java:39-42`.
    pub fn set_energy_usage(&mut self, usage_joules: u64) {
        self.energy_usage_joules = usage_joules;
        let max_fe = self.max_energy_fe();
        if self.energy_stored_fe > max_fe {
            self.energy_stored_fe = max_fe;
        }
    }

    /// Acoustic volume: `sqrt(soundScale)`.
    /// Java: `TileEntityResistiveHeater.java:138`.
    pub fn volume(&self) -> f32 {
        self.sound_scale.max(0.0).sqrt()
    }

    /// Receives energy into buffer up to `max_energy_fe()`. Returns energy accepted.
    pub fn receive_energy(&mut self, amount_fe: u64) -> u64 {
        let max_fe = self.max_energy_fe();
        let space = max_fe.saturating_sub(self.energy_stored_fe);
        let accepted = amount_fe.min(space);
        self.energy_stored_fe += accepted;
        accepted
    }

    /// Simulates one server tick:
    /// 1. Energy consumption and heat generation.
    /// 2. Environmental heat dissipation to air.
    /// 3. Adjacent conduction to connected heat sinks.
    /// 4. Sound scale and active state updates.
    ///
    /// Java reference: `TileEntityResistiveHeater.java:92-114`.
    pub fn tick(
        &mut self,
        has_redstone: bool,
        ambient: f64,
        adjacent_sinks: &mut [HeatCapacitor],
    ) -> bool {
        let mut to_use_fe = 0u64;

        if self.redstone.can_function(has_redstone) {
            let wanted_fe = self.usage_fe();
            to_use_fe = wanted_fe.min(self.energy_stored_fe);

            if to_use_fe > 0 {
                self.energy_stored_fe -= to_use_fe;
                // Convert consumed FE back to Joules for heat generation: J = FE * 5 / 2
                let to_use_joules = (to_use_fe as f64) * 2.5;
                // Java: TileEntityResistiveHeater.java:99:
                // heatCapacitor.handleHeat(toUse * MekanismConfig.general.resistiveHeaterEfficiency.get());
                let heat_made = to_use_joules * EFFICIENCY;
                self.capacitor.handle_heat(heat_made);
            }
        }

        self.active = to_use_fe > 0;
        self.client_energy_used_fe = to_use_fe;

        // Sound scale update (Java: TileEntityResistiveHeater.java:108):
        // float newSoundScale = (float) (toUse / 100_000D);
        let to_use_joules = (to_use_fe as f64) * 2.5;
        self.sound_scale = (to_use_joules / 100_000.0) as f32;

        // Environmental dissipation across all 6 faces (Java: ITileHeatHandler.java:47-62)
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

        // Adjacent conduction to connected sinks (Java: ITileHeatHandler.java:64-100)
        let mut adj_loss = 0.0;
        for sink in adjacent_sinks.iter_mut() {
            let total_inv = sink.inverse_conduction + INVERSE_CONDUCTION;
            let delta_t = (temp - ambient) / total_inv;
            let q = delta_t * HEAT_CAPACITY;

            self.capacitor.handle_heat(-q);
            sink.handle_heat(q);

            if delta_t > 0.0 {
                adj_loss += delta_t;
            }
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
    /// byte 3: redstone mode (0, 1, 2)
    /// bytes 4..12: energy_usage_joules (u64 LE)
    /// bytes 12..20: energy_stored_fe (u64 LE)
    /// bytes 20..28: stored_heat (f64 LE)
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(28);
        out.push(1); // version
        out.push(self.facing as u8);
        out.push(u8::from(self.active));
        out.push(self.redstone as u8);
        out.extend(self.energy_usage_joules.to_le_bytes());
        out.extend(self.energy_stored_fe.to_le_bytes());
        out.extend(self.capacitor.stored_heat.to_le_bytes());
        out
    }

    /// Deserializes state from bytes.
    pub fn decode(data: &[u8], ambient: f64) -> Option<Self> {
        if data.len() < 28 || data[0] != 1 {
            return None;
        }
        let facing = usize::from(data[1]).min(5);
        let active = data[2] == 1;
        let redstone = RedstoneMode::from_u8(data[3]);

        let mut u64_buf = [0u8; 8];
        u64_buf.copy_from_slice(&data[4..12]);
        let energy_usage_joules = u64::from_le_bytes(u64_buf);

        u64_buf.copy_from_slice(&data[12..20]);
        let energy_stored_fe = u64::from_le_bytes(u64_buf);

        u64_buf.copy_from_slice(&data[20..28]);
        let stored_heat = f64::from_le_bytes(u64_buf);

        let mut capacitor = HeatCapacitor::new(
            HEAT_CAPACITY,
            INVERSE_CONDUCTION,
            INVERSE_INSULATION,
            ambient,
        );
        capacitor.set_heat(stored_heat);

        Some(ResistiveHeater {
            energy_usage_joules,
            energy_stored_fe,
            capacitor,
            facing,
            active,
            redstone,
            client_energy_used_fe: 0,
            sound_scale: 0.0,
            last_transfer_loss: 0.0,
            last_environment_loss: 0.0,
        })
    }

    /// Prepares data for GUI screen and network tracking slots.
    /// Java: `TileEntityResistiveHeater.java:159-164`.
    pub fn menu_data(&self) -> Vec<i32> {
        let half = |v: u64, shift: u32| ((v >> shift) & 0xFFFF) as i32;
        let temp_fixed = (self.capacitor.temperature() * 100.0) as i64;
        vec![
            half(self.energy_stored_fe, 0),
            half(self.energy_stored_fe, 16),
            half(self.max_energy_fe(), 0),
            half(self.max_energy_fe(), 16),
            half(self.energy_usage_joules, 0),
            half(self.energy_usage_joules, 16),
            half(self.client_energy_used_fe, 0),
            half(self.client_energy_used_fe, 16),
            (temp_fixed & 0xFFFF) as i32,
            ((temp_fixed >> 16) & 0xFFFF) as i32,
            self.active as i32,
            self.redstone as i32,
        ]
    }
}

// ============================================================================
// Host Bridges (FORK-4 & FORK-20)
// ============================================================================

/// FORK-4: Heat medium bridge for resistive heater thermal emission.
pub fn host_emit_heat(_joules: f64) {
    // FORK-4: First-class heat output to adjacent blocks
}

/// FORK-20: GUI text field input packet handler (`PacketGuiSetEnergy`).
/// Java: `PacketGuiSetEnergy.java` & `GuiResistiveHeater.java:79-80`.
pub fn handle_packet_gui_set_energy(heater: &mut ResistiveHeater, joules: u64) {
    // FORK-20: Interactive text field input
    heater.set_energy_usage(joules);
}

// ============================================================================
// Unit Tests (comparing with Java source files and line numbers)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resistive_heater_constants_match_java() {
        // TileEntityResistiveHeater.java:46: HEAT_CAPACITY = 100
        assert_eq!(HEAT_CAPACITY, 100.0);
        // TileEntityResistiveHeater.java:47: INVERSE_CONDUCTION_COEFFICIENT = 5
        assert_eq!(INVERSE_CONDUCTION, 5.0);
        // TileEntityResistiveHeater.java:48: INVERSE_INSULATION_COEFFICIENT = 10
        assert_eq!(INVERSE_INSULATION, 10.0);
        // TileEntityResistiveHeater.java:50: BASE_USAGE = 100L
        assert_eq!(BASE_USAGE_JOULES, 100);
        assert_eq!(BASE_USAGE_FE, 40); // 100 * 2 / 5 = 40 FE

        // ResistiveHeaterEnergyContainer.java:22: USAGE_MULTIPLIER = 400
        assert_eq!(USAGE_MULTIPLIER, 400);

        // GeneralConfig.java:149: resistiveHeaterEfficiency = 0.6
        assert_eq!(EFFICIENCY, 0.6);
    }

    #[test]
    fn test_energy_buffer_scales_with_configured_usage() {
        // ResistiveHeaterEnergyContainer.java:41:
        // setMaxEnergy(MathUtils.multiplyClamped(energyUsage, USAGE_MULTIPLIER));
        let mut heater = ResistiveHeater::new(300.0);
        // At default 100 J/t -> capacity = 100 * 400 = 40_000 J = 16_000 FE
        assert_eq!(heater.max_energy_fe(), fe(40_000));
        assert_eq!(heater.max_energy_fe(), 16_000);

        // Configure usage to 10_000 J/t (4_000 FE/t)
        heater.set_energy_usage(10_000);
        // Capacity = 10_000 * 400 = 4_000_000 J = 1_600_000 FE
        assert_eq!(heater.max_energy_fe(), fe(4_000_000));
        assert_eq!(heater.max_energy_fe(), 1_600_000);
    }

    #[test]
    fn test_heat_generation_at_60_percent_efficiency() {
        // TileEntityResistiveHeater.java:99:
        // heatCapacitor.handleHeat(toUse * MekanismConfig.general.resistiveHeaterEfficiency.get());
        let mut heater = ResistiveHeater::new(300.0);
        heater.set_energy_usage(100); // 100 J/t = 40 FE/t
        heater.receive_energy(10_000); // Fill buffer

        let initial_heat = heater.capacitor.stored_heat;
        assert_eq!(initial_heat, 30_000.0); // 100.0 * 300.0 K

        let mut sinks = [];
        heater.tick(false, 300.0, &mut sinks); // Active with redstone ignored

        // Energy consumed: 40 FE = 100 Joules
        // Heat generated: 100 * 0.6 = 60 Joules
        // Heat before dissipation: 30_000.0 + 60.0 = 30_060.0
        // Dissipation on 6 faces: (T - 300) / 10015.0 per face
        assert!(heater.active);
        assert_eq!(heater.client_energy_used_fe, 40);
        assert!(heater.capacitor.temperature() > 300.0);
        assert!(heater.capacitor.stored_heat > 30_000.0);
    }

    #[test]
    fn test_redstone_control_modes() {
        let mut heater = ResistiveHeater::new(300.0);
        heater.receive_energy(1_000);

        // Ignored mode: operates regardless of redstone power
        heater.redstone = RedstoneMode::Ignored;
        let mut sinks = [];
        heater.tick(false, 300.0, &mut sinks);
        assert!(heater.active);

        // Normal mode: only operates when redstone powered
        heater.redstone = RedstoneMode::Normal;
        heater.tick(false, 300.0, &mut sinks);
        assert!(!heater.active);
        heater.tick(true, 300.0, &mut sinks);
        assert!(heater.active);

        // Inverted mode: only operates when NOT redstone powered
        heater.redstone = RedstoneMode::Inverted;
        heater.tick(true, 300.0, &mut sinks);
        assert!(!heater.active);
        heater.tick(false, 300.0, &mut sinks);
        assert!(heater.active);
    }

    #[test]
    fn test_sound_scale_calculation() {
        // TileEntityResistiveHeater.java:108, 138:
        // soundScale = toUseJoules / 100_000.0
        // volume = sqrt(soundScale)
        let mut heater = ResistiveHeater::new(300.0);
        heater.set_energy_usage(100_000); // 100,000 J/t = 40,000 FE/t
        heater.receive_energy(1_000_000);

        let mut sinks = [];
        heater.tick(false, 300.0, &mut sinks);

        assert!((heater.sound_scale - 1.0).abs() < 1e-4);
        assert!((heater.volume() - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_resistive_heater_serialization_roundtrip() {
        let mut heater = ResistiveHeater::new(300.0);
        heater.facing = 4; // West
        heater.redstone = RedstoneMode::Normal;
        heater.set_energy_usage(5_000);
        heater.receive_energy(25_000);
        heater.capacitor.set_heat(35_400.0);
        heater.active = true;

        let bytes = heater.encode();
        let decoded = ResistiveHeater::decode(&bytes, 300.0).expect("decode failed");

        assert_eq!(decoded.facing, 4);
        assert!(decoded.active);
        assert_eq!(decoded.redstone, RedstoneMode::Normal);
        assert_eq!(decoded.energy_usage_joules, 5_000);
        assert_eq!(decoded.energy_stored_fe, 25_000);
        assert!((decoded.capacitor.stored_heat - 35_400.0).abs() < 1e-6);
        assert_eq!(decoded.max_energy_fe(), fe(5_000 * 400));
    }
}

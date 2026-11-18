//! Seismic Vibrator (`TileEntitySeismicVibrator`).
//!
//! A 1×2×1 tall machine that vibrates the ground beneath it to map the strata and ores of the chunk.
//!
//! Java references:
//! - `TileEntitySeismicVibrator.java:40`: `MekanismBlocks.SEISMIC_VIBRATOR`
//! - `TileEntitySeismicVibrator.java:48`: `RelativeSide.BACK` energy input only
//! - `TileEntitySeismicVibrator.java:56`: `EnergyInventorySlot` at (143, 35)
//! - `TileEntitySeismicVibrator.java:78`: `ticker % (2 * TICKS_PER_SECOND) == 0` -> 40 ticks interval
//! - `TileEntitySeismicVibrator.java:79`: `level.gameEvent(null, MekanismGameEvents.SEISMIC_VIBRATION, worldPosition)`
//! - `TileEntitySeismicVibrator.java:93`: `Mekanism.activeVibrators.add(getTileGlobalPos())`
//! - `TileEntitySeismicVibrator.java:121-130`: `chunkRelativeX` (0..=15), `chunkRelativeZ` (0..=15)
//! - `TileEntitySeismicVibrator.java:142-151`: `getColumnAt(chunkRelativeX, chunkRelativeZ)`
//! - `StorageConfig.java:40`: `seismicVibrator = 20_000L` J = 8_000 FE (`fe(20_000)`)
//! - `UsageConfig.java:43`: `seismicVibrator = 50L` J/t = 20 FE/t (`fe(50)`)
//! - `bounding.rs:22`: `([0, 1, 0], "none")`

use super::*;
use std::collections::HashSet;

pub const BLOCK: &str = "mekanism:seismic_vibrator";

/// Buffer capacity: 20,000 J = 8,000 FE (Java: `StorageConfig.seismicVibrator`, `fe(20_000)`).
pub const CAPACITY: u64 = fe(20_000);

/// Energy consumption: 50 J/t = 20 FE/t (Java: `UsageConfig.seismicVibrator`, `fe(50)`).
pub const USAGE: u64 = fe(50);

/// Vibration interval: 40 ticks (2 seconds) (Java: `TileEntitySeismicVibrator.java:78`).
pub const VIBRATION_INTERVAL: u64 = 40;

/// Energy slot index in the block entity inventory.
pub const ENERGY_SLOT: usize = 0;

/// Returns the input energy sides relative to facing (Java: `TileEntitySeismicVibrator.java:48`).
/// Only RelativeSide.BACK accepts energy.
pub fn energy_input_faces(facing: &str) -> Vec<FlowSide> {
    let front = laser::face_of(facing);
    let back = laser::opposite(front);
    let mut faces = vec![FlowSide::None; 6];
    faces[back] = FlowSide::Input;
    faces
}

/// Global tracking of active vibrators in dimensions.
pub type ActiveVibratorKey = (String, i32, i32, i32);

/// State of the Seismic Vibrator.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub energy: u64,
    pub active: bool,
    pub client_piston: u32,
    pub redstone_control: redstone::Control,
}

impl Default for State {
    fn default() -> Self {
        State {
            energy: 0,
            active: false,
            client_piston: 0,
            redstone_control: redstone::Control::Disabled,
        }
    }
}

pub fn decode(data: &[u8]) -> State {
    if data.len() < 13 {
        return State::default();
    }
    let energy = u64::from_le_bytes([
        data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
    ]);
    let active = data[8] == 1;
    let client_piston = u32::from_le_bytes([data[9], data[10], data[11], data[12]]);
    let redstone_control = redstone::Control::from_byte(data.get(13).copied().unwrap_or(0));
    State {
        energy: energy.min(CAPACITY),
        active,
        client_piston,
        redstone_control,
    }
}

pub fn encode(state: &State) -> Vec<u8> {
    let mut out = state.energy.to_le_bytes().to_vec();
    out.push(u8::from(state.active));
    out.extend(state.client_piston.to_le_bytes());
    out.push(state.redstone_control as u8);
    out
}

/// The numbers the screen reads: 0 it vibrates, 1-2 energy, 3-4 capacity (low and high 16 bits), 5 the energy a tick (Forge Energy), 6 the redstone
/// control, 7-8 the energy still needed.
pub fn menu_numbers(state: &State, energy: u64) -> Vec<i32> {
    let half = |v: u64, s: u32| ((v >> s) & 0xFFFF) as i32;
    let needed = CAPACITY.saturating_sub(energy);
    vec![
        i32::from(state.active),
        half(energy, 0),
        half(energy, 16),
        half(CAPACITY, 0),
        half(CAPACITY, 16),
        USAGE as i32,
        state.redstone_control as i32,
        half(needed, 0),
        half(needed, 16),
    ]
}

/// Computes the absolute block coordinate for a chunk-relative query (Java: `TileEntitySeismicVibrator.java:121-130`).
pub fn get_vertical_pos(
    base_pos: [i32; 3],
    chunk_rel_x: i32,
    y: i32,
    chunk_rel_z: i32,
) -> Result<[i32; 3], &'static str> {
    if !(0..=15).contains(&chunk_rel_x) {
        return Err("Chunk Relative X is out of range, must be between 0 and 15");
    }
    if !(0..=15).contains(&chunk_rel_z) {
        return Err("Chunk Relative Z is out of range, must be between 0 and 15");
    }
    let chunk_x = base_pos[0] >> 4;
    let chunk_z = base_pos[2] >> 4;
    let abs_x = (chunk_x << 4) + chunk_rel_x;
    let abs_z = (chunk_z << 4) + chunk_rel_z;
    Ok([abs_x, y, abs_z])
}

/// Host API bridge for dispatching seismic vibration game events.
// FORK-31: Seismic vibration game event and active vibrator registration
pub fn emit_seismic_vibration(dimension: &str, pos: BlockPos) {
    let _ = (dimension, pos);
}

/// Global active vibrator cache.
pub static mut ACTIVE_VIBRATORS: Option<HashSet<ActiveVibratorKey>> = None;

impl Mekanism {
    /// Server tick handler for Seismic Vibrators.
    pub(crate) fn run_seismic_vibrators(
        &mut self,
        entities: &[&BlockEntity],
        tick: u64,
    ) -> Vec<BlockEntityUpdate> {
        let mut out = Vec::new();
        for e in entities {
            let block = e.state.split('[').next().unwrap_or("");
            if block != BLOCK {
                continue;
            }

            let facing = sides::property(&e.state, "facing").unwrap_or("north");
            let mut state = decode(&e.data);
            let mut energy = e.energy;

            // The charge slot turns redstone into energy when it fits (`fillContainerOrConvert`).
            let mut slot = e.items.first().cloned().flatten();
            let mut slot_changed = false;
            if let Some((gain, left)) = crate::cube::fill_from_slot(&slot, energy, CAPACITY) {
                energy = gain;
                slot = left;
                slot_changed = true;
            }

            // `canFunction`: the redstone control with the signal now and the one of the tick before.
            let key = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
            let powered = world::get_redstone(&e.dimension, e.pos.x, e.pos.y, e.pos.z) > 0;
            let last = self.last_power.insert(key.clone(), powered).unwrap_or(false);
            let can_function = state.redstone_control.can_function(powered, last);

            let was_active = state.active;

            if can_function && energy >= USAGE {
                energy -= USAGE;
                state.active = true;
                self.vibrations.insert((e.dimension.clone(), e.pos.x >> 4, e.pos.z >> 4), tick);
                state.client_piston = state.client_piston.wrapping_add(1);

                // Every two seconds a new vibration is sent (java:78); the game event itself is FORK-31.
                if tick % VIBRATION_INTERVAL == 0 {
                    emit_seismic_vibration(&e.dimension, e.pos);
                }
            } else {
                state.active = false;
            }

            state.energy = energy;

            // The chunk the screen says (`CHUNK`), given once: the strings are the screen's text 0.
            if self.menu_texts.get(&key).is_none() {
                let text = format!("Chunk: {}, {}", e.pos.x >> 4, e.pos.z >> 4);
                let _ = world::set_menu_text(&e.dimension, e.pos.x, e.pos.y, e.pos.z, std::slice::from_ref(&text));
                self.menu_texts.insert(key, vec![text]);
            }

            let numbers = menu_numbers(&state, energy);
            let changed = was_active != state.active || energy != e.energy || slot_changed || e.menu_data != numbers;
            if changed {
                if was_active != state.active {
                    let _ = world::set_block_property(
                        &e.dimension,
                        e.pos.x,
                        e.pos.y,
                        e.pos.z,
                        "active",
                        if state.active { "true" } else { "false" },
                    );
                }

                out.push(BlockEntityUpdate {
                    dimension: e.dimension.clone(),
                    pos: e.pos,
                    data: encode(&state),
                    items: if slot_changed { Some(vec![slot]) } else { None },
                    energy: Some(energy),
                    energy_faces: Some(energy_input_faces(facing)),
                    tanks: None,
                    tank_faces: None,
                    menu_data: Some(numbers),
                    transmitter_faces: None,
                });
            }
        }
        out
    }

    /// A button of the screen: the redstone tab.
    pub(crate) fn seismic_vibrator_button(&mut self, e: &BlockEntity, id: i32) -> Option<BlockEntityUpdate> {
        let mut state = decode(&e.data);
        if id == crate::redstone::NEXT {
            state.redstone_control = state.redstone_control.next();
        } else if id == crate::redstone::PREVIOUS {
            state.redstone_control = state.redstone_control.previous();
        } else {
            return None;
        }
        Some(BlockEntityUpdate {
            dimension: e.dimension.clone(),
            pos: e.pos,
            data: encode(&state),
            items: None,
            energy: None,
            energy_faces: None,
            tanks: None,
            tank_faces: None,
            menu_data: Some(menu_numbers(&state, e.energy)),
            transmitter_faces: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seismic_vibrator_energy_and_timing() {
        // Java: StorageConfig.java:40 seismicVibrator = 20_000 J = 8_000 FE
        assert_eq!(CAPACITY, 8_000);

        // Java: UsageConfig.java:43 seismicVibrator = 50 J/t = 20 FE/t
        assert_eq!(USAGE, 20);

        // Java: TileEntitySeismicVibrator.java:78
        // ticker % (2 * SharedConstants.TICKS_PER_SECOND) == 0 (2 * 20 = 40)
        assert_eq!(VIBRATION_INTERVAL, 40);
    }

    #[test]
    fn test_energy_input_face_back_only() {
        // Java: TileEntitySeismicVibrator.java:48
        // RelativeSide.BACK only
        let north_faces = energy_input_faces("north");
        assert_eq!(north_faces[laser::SOUTH], FlowSide::Input); // Opposite of North
        assert_eq!(north_faces[laser::NORTH], FlowSide::None);
        assert_eq!(north_faces[laser::DOWN], FlowSide::None);
        assert_eq!(north_faces[laser::UP], FlowSide::None);
        assert_eq!(north_faces[laser::WEST], FlowSide::None);
        assert_eq!(north_faces[laser::EAST], FlowSide::None);

        let west_faces = energy_input_faces("west");
        assert_eq!(west_faces[laser::EAST], FlowSide::Input); // Opposite of West
    }

    #[test]
    fn test_chunk_relative_coordinates_validation() {
        // Java: TileEntitySeismicVibrator.java:121-130
        let base = [32, 64, 48]; // chunk (2, 3)

        // Valid inside chunk
        let pos = get_vertical_pos(base, 0, 10, 0);
        assert!(pos.is_ok());
        assert_eq!(pos.unwrap(), [32, 10, 48]);

        let pos = get_vertical_pos(base, 15, 60, 15);
        assert!(pos.is_ok());
        assert_eq!(pos.unwrap(), [47, 60, 63]);

        // Invalid: < 0 or > 15
        assert!(get_vertical_pos(base, -1, 10, 5).is_err());
        assert!(get_vertical_pos(base, 16, 10, 5).is_err());
        assert!(get_vertical_pos(base, 5, 10, -1).is_err());
        assert!(get_vertical_pos(base, 5, 10, 16).is_err());
    }

    #[test]
    fn test_seismic_vibrator_state_serialization() {
        let state = State {
            energy: 6_400,
            active: true,
            client_piston: 120,
            redstone_control: redstone::Control::High,
        };
        let encoded = encode(&state);
        let decoded = decode(&encoded);

        assert_eq!(decoded.energy, 6_400);
        assert!(decoded.active);
        assert_eq!(decoded.client_piston, 120);
        assert_eq!(decoded.redstone_control, redstone::Control::High);
    }

    #[test]
    fn the_screen_numbers_follow_the_state() {
        let state = State { active: true, ..State::default() };
        let n = menu_numbers(&state, 5_000);
        assert_eq!(n.len(), 9);
        assert_eq!((n[0], n[1], n[5]), (1, 5_000, USAGE as i32));
        assert_eq!(n[7] + (n[8] << 16), (CAPACITY - 5_000) as i32);
    }
}

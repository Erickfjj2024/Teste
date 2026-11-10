//! Dimensional Stabilizer (`TileEntityDimensionalStabilizer`).
//!
//! Keeps chunks loaded around it within a configurable 5×5 chunk grid (diameter 5, radius 2).
//!
//! Java references:
//! - `TileEntityDimensionalStabilizer.java:46`: `MAX_LOAD_RADIUS = 2`
//! - `TileEntityDimensionalStabilizer.java:47`: `MAX_LOAD_DIAMETER = 5` (5×5 grid)
//! - `TileEntityDimensionalStabilizer.java:55`: `chunksLoaded` starts at 1
//! - `TileEntityDimensionalStabilizer.java:68`: center chunk `[2][2]` is always loaded
//! - `TileEntityDimensionalStabilizer.java:92`: `allowChunkloading` config
//! - `TileEntityDimensionalStabilizer.java:186`: `Redstone.SIGNAL_MAX` (15) when active, 0 when inactive
//! - `TileEntityDimensionalStabilizer.java:219-225`: 25-byte array serialization
//! - `UsageConfig.java`: `dimensionalStabilizer = 5_000L` J/t = 2_000 FE/t per loaded chunk (`fe(5_000)`)
//! - `StorageConfig.java`: `dimensionalStabilizer = 40_000L` J = 16_000 FE (`fe(40_000)`)

use super::*;

pub const BLOCK: &str = "mekanism:dimensional_stabilizer";

pub const MAX_LOAD_RADIUS: i32 = 2;
pub const MAX_LOAD_DIAMETER: usize = 5;
pub const TOTAL_CHUNKS: usize = 25;

/// Base energy usage per loaded chunk: 5,000 J/t = 2,000 FE/t (Java: `UsageConfig.dimensionalStabilizer`, `fe(5_000)`).
pub const BASE_ENERGY_PER_CHUNK: u64 = fe(5_000);

/// Base buffer capacity: 40,000 J = 16,000 FE (Java: `StorageConfig.dimensionalStabilizer`, `fe(40_000)`).
pub const BASE_CAPACITY: u64 = fe(40_000);

/// Energy slot index in the block entity inventory (Java: `TileEntityDimensionalStabilizer.java:83`).
pub const ENERGY_SLOT: usize = 0;

/// GUI action buttons.
pub const BUTTON_TOGGLE_CHUNK_BASE: i32 = 7500; // 7500 + x * 5 + z
pub const BUTTON_RADIUS_1: i32 = 7530; // 3x3 (9 chunks)
pub const BUTTON_RADIUS_2: i32 = 7531; // 5x5 (25 chunks)
pub const BUTTON_TOGGLE_ALL: i32 = 7532;

/// A chunk coordinate pair (chunk_x, chunk_z).
pub type ChunkPos = (i32, i32);

/// State of the Dimensional Stabilizer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    /// 5×5 grid of chunks to load. Center [2][2] is ALWAYS true.
    pub loading_chunks: [[bool; MAX_LOAD_DIAMETER]; MAX_LOAD_DIAMETER],
    pub chunks_loaded: u32,
    pub energy: u64,
    pub active: bool,
    pub energy_upgrades: u8,
}

impl Default for State {
    fn default() -> Self {
        let mut chunks = [[false; MAX_LOAD_DIAMETER]; MAX_LOAD_DIAMETER];
        chunks[2][2] = true; // Center chunk is always loaded
        State {
            loading_chunks: chunks,
            chunks_loaded: 1,
            energy: 0,
            active: false,
            energy_upgrades: 0,
        }
    }
}

impl State {
    /// Toggles chunk loading at relative grid coordinate (x, z) in [0..5).
    /// Center chunk [2][2] cannot be disabled (Java: `TileEntityDimensionalStabilizer.java:148`).
    pub fn toggle_chunk(&mut self, x: usize, z: usize) -> bool {
        if x >= MAX_LOAD_DIAMETER || z >= MAX_LOAD_DIAMETER {
            return false;
        }
        if x == MAX_LOAD_RADIUS as usize && z == MAX_LOAD_RADIUS as usize {
            // Center chunk is always loaded
            return false;
        }
        self.loading_chunks[x][z] = !self.loading_chunks[x][z];
        if self.loading_chunks[x][z] {
            self.chunks_loaded += 1;
        } else {
            self.chunks_loaded = self.chunks_loaded.saturating_sub(1);
        }
        true
    }

    /// Sets all chunks within a given radius (1 = 3x3, 2 = 5x5) to the given loaded state.
    pub fn set_radius(&mut self, radius: i32, load: bool) {
        let r = radius.clamp(1, MAX_LOAD_RADIUS);
        for dx in -r..=r {
            for dz in -r..=r {
                let x = (dx + MAX_LOAD_RADIUS) as usize;
                let z = (dz + MAX_LOAD_RADIUS) as usize;
                if x == 2 && z == 2 {
                    continue; // Skip center chunk
                }
                if self.loading_chunks[x][z] != load {
                    self.loading_chunks[x][z] = load;
                    if load {
                        self.chunks_loaded += 1;
                    } else {
                        self.chunks_loaded = self.chunks_loaded.saturating_sub(1);
                    }
                }
            }
        }
    }

    /// Calculates energy capacity scaled by energy upgrades (up to 8 upgrades).
    pub fn max_energy(&self) -> u64 {
        upgrades::max_energy(BASE_CAPACITY, self.energy_upgrades)
    }

    /// Calculates total energy usage per tick for the loaded chunks.
    pub fn energy_per_tick(&self) -> u64 {
        let base_total = BASE_ENERGY_PER_CHUNK.saturating_mul(self.chunks_loaded as u64);
        upgrades::energy_per_tick(base_total, 0, self.energy_upgrades)
    }

    /// Returns the redstone signal level emitted (15 when active, 0 when inactive).
    pub fn redstone_signal(&self) -> u8 {
        if self.active { 15 } else { 0 }
    }

    /// Computes the set of absolute chunk coordinates that this stabilizer keeps loaded.
    pub fn get_loaded_chunk_set(&self, block_x: i32, block_z: i32) -> Vec<ChunkPos> {
        let center_chunk_x = block_x >> 4;
        let center_chunk_z = block_z >> 4;
        let mut list = Vec::new();
        for dx in -MAX_LOAD_RADIUS..=MAX_LOAD_RADIUS {
            for dz in -MAX_LOAD_RADIUS..=MAX_LOAD_RADIUS {
                let gx = (dx + MAX_LOAD_RADIUS) as usize;
                let gz = (dz + MAX_LOAD_RADIUS) as usize;
                if self.loading_chunks[gx][gz] {
                    list.push((center_chunk_x + dx, center_chunk_z + dz));
                }
            }
        }
        list
    }
}

pub fn decode(data: &[u8]) -> State {
    if data.len() < 34 {
        return State::default();
    }
    let energy = u64::from_le_bytes([
        data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
    ]);
    let active = data[8] == 1;
    let energy_upgrades = data[9];
    let mut loading_chunks = [[false; MAX_LOAD_DIAMETER]; MAX_LOAD_DIAMETER];
    let mut count = 0u32;
    for x in 0..MAX_LOAD_DIAMETER {
        for z in 0..MAX_LOAD_DIAMETER {
            let idx = 10 + x * MAX_LOAD_DIAMETER + z;
            let val = data.get(idx).copied().unwrap_or(0) == 1;
            loading_chunks[x][z] = val;
            if val {
                count += 1;
            }
        }
    }
    // Always enforce center chunk
    if !loading_chunks[2][2] {
        loading_chunks[2][2] = true;
        count += 1;
    }
    State {
        loading_chunks,
        chunks_loaded: count,
        energy,
        active,
        energy_upgrades,
    }
}

pub fn encode(state: &State) -> Vec<u8> {
    let mut out = state.energy.to_le_bytes().to_vec();
    out.push(u8::from(state.active));
    out.push(state.energy_upgrades);
    for x in 0..MAX_LOAD_DIAMETER {
        for z in 0..MAX_LOAD_DIAMETER {
            out.push(u8::from(state.loading_chunks[x][z]));
        }
    }
    out
}

/// Host API bridge for requesting chunk loading tickets in the game world.
// FORK-14: Chunk ticket loading API in Horizon host (`IChunkLoader`)
pub fn sync_chunk_tickets(
    dimension: &str,
    chunks: &[ChunkPos],
    active: bool,
) {
    let _ = (dimension, chunks, active);
    // In WASM without host API support, chunk tickets are simulated purely in state
}

impl Mekanism {
    /// Server tick handler for Dimensional Stabilizers.
    pub(crate) fn run_dimensional_stabilizers(
        &mut self,
        entities: &[&BlockEntity],
        tick: u64,
    ) -> Vec<BlockEntityUpdate> {
        let _ = tick;
        let mut out = Vec::new();
        for e in entities {
            let block = e.state.split('[').next().unwrap_or("");
            if block != BLOCK {
                continue;
            }

            let mut state = decode(&e.data);
            let mut energy = e.energy;

            // Handle GUI button clicks
            for (_, p, id) in self.sorter_buttons.iter().filter(|(d, pos, _)| {
                *d == e.dimension && (pos.x, pos.y, pos.z) == (e.pos.x, e.pos.y, e.pos.z)
            }) {
                let _ = p;
                if *id >= BUTTON_TOGGLE_CHUNK_BASE && *id < BUTTON_TOGGLE_CHUNK_BASE + 25 {
                    let offset = (*id - BUTTON_TOGGLE_CHUNK_BASE) as usize;
                    let x = offset / 5;
                    let z = offset % 5;
                    state.toggle_chunk(x, z);
                } else if *id == BUTTON_RADIUS_1 {
                    state.set_radius(1, true);
                } else if *id == BUTTON_RADIUS_2 {
                    state.set_radius(2, true);
                } else if *id == BUTTON_TOGGLE_ALL {
                    let any_off = state.chunks_loaded < 25;
                    state.set_radius(2, any_off);
                }
            }

            // Energy drain: 2_000 FE/t per loaded chunk (Java: `TileEntityDimensionalStabilizer.java:93-99`)
            let needed = state.energy_per_tick();
            let was_active = state.active;
            if energy >= needed && needed > 0 {
                energy -= needed;
                state.active = true;
            } else {
                state.active = false;
            }

            state.energy = energy;

            // Sync chunk tickets with host
            let loaded_chunks = state.get_loaded_chunk_set(e.pos.x, e.pos.z);
            sync_chunk_tickets(&e.dimension, &loaded_chunks, state.active);

            let changed = was_active != state.active || energy != e.energy;
            if changed {
                // Sits in menu_data: [active, chunks_loaded, energy_low, energy_high]
                let menu_data = vec![
                    i32::from(state.active),
                    state.chunks_loaded as i32,
                    (energy & 0xFFFF_FFFF) as i32,
                    ((energy >> 32) & 0xFFFF_FFFF) as i32,
                ];

                out.push(BlockEntityUpdate {
                    dimension: e.dimension.clone(),
                    pos: e.pos,
                    data: encode(&state),
                    items: None,
                    energy: Some(energy),
                    energy_faces: Some(vec![FlowSide::Input; 6]),
                    tanks: None,
                    tank_faces: None,
                    menu_data: Some(menu_data),
                    transmitter_faces: None,
                });
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimensional_stabilizer_energy_values() {
        // Java: UsageConfig.java: dimensionalStabilizer = 5_000 J/t = 2_000 FE/t
        assert_eq!(BASE_ENERGY_PER_CHUNK, 2_000);

        // Java: StorageConfig.java: dimensionalStabilizer = 40_000 J = 16_000 FE
        assert_eq!(BASE_CAPACITY, 16_000);
    }

    #[test]
    fn test_center_chunk_always_loaded() {
        // Java: TileEntityDimensionalStabilizer.java:68
        // Center chunk where the stabilizer is, is always loaded
        let mut state = State::default();
        assert!(state.loading_chunks[2][2]);
        assert_eq!(state.chunks_loaded, 1);

        // Attempting to toggle center chunk must fail and keep it loaded (line 148)
        let toggled = state.toggle_chunk(2, 2);
        assert!(!toggled);
        assert!(state.loading_chunks[2][2]);
        assert_eq!(state.chunks_loaded, 1);
    }

    #[test]
    fn test_chunk_toggling_and_counts() {
        let mut state = State::default();
        assert_eq!(state.chunks_loaded, 1);

        // Enable [0][0]
        assert!(state.toggle_chunk(0, 0));
        assert!(state.loading_chunks[0][0]);
        assert_eq!(state.chunks_loaded, 2);
        assert_eq!(state.energy_per_tick(), 4_000); // 2 * 2_000 FE/t

        // Disable [0][0]
        assert!(state.toggle_chunk(0, 0));
        assert!(!state.loading_chunks[0][0]);
        assert_eq!(state.chunks_loaded, 1);
        assert_eq!(state.energy_per_tick(), 2_000);
    }

    #[test]
    fn test_radius_shortcuts() {
        let mut state = State::default();
        // Radius 1: 3x3 = 9 chunks
        state.set_radius(1, true);
        assert_eq!(state.chunks_loaded, 9);
        assert_eq!(state.energy_per_tick(), 18_000); // 9 * 2_000 FE/t

        // Radius 2: 5x5 = 25 chunks
        state.set_radius(2, true);
        assert_eq!(state.chunks_loaded, 25);
        assert_eq!(state.energy_per_tick(), 50_000); // 25 * 2_000 FE/t

        // Unload radius 2 (keeps center loaded)
        state.set_radius(2, false);
        assert_eq!(state.chunks_loaded, 1);
        assert!(state.loading_chunks[2][2]);
    }

    #[test]
    fn test_redstone_emission() {
        // Java: TileEntityDimensionalStabilizer.java:186
        // return getActive() ? Redstone.SIGNAL_MAX : Redstone.SIGNAL_NONE;
        let mut state = State::default();
        state.active = false;
        assert_eq!(state.redstone_signal(), 0);

        state.active = true;
        assert_eq!(state.redstone_signal(), 15);
    }

    #[test]
    fn test_chunkpos_calculation() {
        // Stabilizer at block position (x: 100, z: -200)
        // Center chunk is (100 >> 4 = 6, -200 >> 4 = -13)
        let state = State::default();
        let chunks = state.get_loaded_chunk_set(100, -200);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], (6, -13));
    }

    #[test]
    fn test_state_serialization_roundtrip() {
        let mut state = State::default();
        state.energy = 12_500;
        state.active = true;
        state.energy_upgrades = 3;
        state.set_radius(1, true);

        let encoded = encode(&state);
        let decoded = decode(&encoded);

        assert_eq!(decoded.energy, 12_500);
        assert_eq!(decoded.active, true);
        assert_eq!(decoded.energy_upgrades, 3);
        assert_eq!(decoded.chunks_loaded, 9);
        assert_eq!(decoded.loading_chunks, state.loading_chunks);
    }
}

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
use horizon_mod_sdk::server::chunks;

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
pub const BUTTON_TOGGLE_CHUNK_BASE: i32 = 7500; // 7500 + x * 5 + z (`toggleChunkLoadingAt(extra / 5, extra % 5)`)
/// `adjustChunkLoadingRadius(radius, true)` is 7530 + radius, `adjustChunkLoadingRadius(radius, false)` 7540 + radius (the three packets of `PacketGuiInteract`).
pub const BUTTON_RING_ENABLE_BASE: i32 = 7530;
pub const BUTTON_RING_DISABLE_BASE: i32 = 7540;

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
    /// The redstone control (`canFunction`).
    pub control: Control,
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
            control: Control::Disabled,
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

    /// `adjustChunkLoadingRadius(radius, load)` (java:122-146): only the ring of chunks at that distance from the middle (the border of the square of that
    /// radius, not what is inside it) is set to `load`; the middle one is never touched. Says whether anything changed.
    pub fn adjust_radius(&mut self, radius: i32, load: bool) -> bool {
        if radius <= 0 || radius > MAX_LOAD_RADIUS {
            return false;
        }
        let mut changed = false;
        for dx in -radius..=radius {
            let skip_inner = dx > -radius && dx < radius;
            let mut dz = -radius;
            while dz <= radius {
                let x = (dx + MAX_LOAD_RADIUS) as usize;
                let z = (dz + MAX_LOAD_RADIUS) as usize;
                if !(x == MAX_LOAD_RADIUS as usize && z == MAX_LOAD_RADIUS as usize) && self.loading_chunks[x][z] != load {
                    self.loading_chunks[x][z] = load;
                    if load {
                        self.chunks_loaded += 1;
                    } else {
                        self.chunks_loaded = self.chunks_loaded.saturating_sub(1);
                    }
                    changed = true;
                }
                dz += if skip_inner { 2 * radius } else { 1 };
            }
        }
        changed
    }

    /// `GuiDimensionalStabilizer.hasAtRadius`: whether some chunk of the ring at `radius` is `state`.
    pub fn has_at_radius(&self, radius: i32, state: bool) -> bool {
        for dx in -radius..=radius {
            let skip_inner = dx > -radius && dx < radius;
            let mut dz = -radius;
            while dz <= radius {
                if self.loading_chunks[(dx + MAX_LOAD_RADIUS) as usize][(dz + MAX_LOAD_RADIUS) as usize] == state {
                    return true;
                }
                dz += if skip_inner { 2 * radius } else { 1 };
            }
        }
        false
    }

    /// The numbers the screen reads: 0-1 energy, 2-3 capacity (low and high 16 bits), 4 active, 5 chunks loaded, 6-7 energy a tick, 8 the radius the
    /// middle button's left click turns on (the first ring with a chunk off, 0 for none), 9 the radius its right click turns off (the outermost ring with
    /// a chunk on, 0 for none), 10 the redstone control, 11-12 energy still needed, then the 25 chunks in grid order (`x * 5 + z`), then at 38 both radii in one number.
    pub fn menu_numbers(&self, energy: u64) -> Vec<i32> {
        let half = |v: u64, s: u32| ((v >> s) & 0xFFFF) as i32;
        let capacity = self.max_energy();
        let per_tick = self.energy_per_tick();
        let enable = (1..=MAX_LOAD_RADIUS).find(|r| self.has_at_radius(*r, false)).unwrap_or(0);
        let disable = (1..=MAX_LOAD_RADIUS).rev().find(|r| self.has_at_radius(*r, true)).unwrap_or(0);
        let mut v = vec![
            half(energy, 0),
            half(energy, 16),
            half(capacity, 0),
            half(capacity, 16),
            i32::from(self.active),
            self.chunks_loaded as i32,
            half(per_tick, 0),
            half(per_tick, 16),
            enable,
            disable,
            self.control as i32,
            half(capacity.saturating_sub(energy), 0),
            half(capacity.saturating_sub(energy), 16),
        ];
        for x in 0..MAX_LOAD_DIAMETER {
            for z in 0..MAX_LOAD_DIAMETER {
                v.push(i32::from(self.loading_chunks[x][z]));
            }
        }
        // 38: both radii in one number (`turn on * 3 + turn off`), for a screen that can test only one number at a time.
        v.push(enable * 3 + disable);
        v
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
        control: Control::from_byte(data.get(10 + MAX_LOAD_DIAMETER * MAX_LOAD_DIAMETER).copied().unwrap_or(0)),
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

impl Mekanism {
    /// Keeps exactly the chunks the stabilizer loads forced (`TileComponentChunkLoader.refreshChunkTickets`, which only holds tickets while it is active):
    /// the new ones are asked for, the ones it no longer loads (or all of them when it stops) are let go. The game keeps at most 256 a mod.
    fn sync_stabilizer_tickets(&mut self, key: &(String, i32, i32, i32), wanted: &[ChunkPos], active: bool) {
        let want: std::collections::HashSet<ChunkPos> = if active { wanted.iter().copied().collect() } else { std::collections::HashSet::new() };
        let held = self.stabilizer_tickets.entry(key.clone()).or_default();
        for c in want.difference(held).copied().collect::<Vec<_>>() {
            let _ = chunks::set_forced(&key.0, c.0, c.1, true);
        }
        for c in held.difference(&want).copied().collect::<Vec<_>>() {
            let _ = chunks::set_forced(&key.0, c.0, c.1, false);
        }
        *held = want;
    }

    /// A stabilizer that is gone lets go of its chunks (`TileComponentChunkLoader.removed`).
    pub(crate) fn release_stabilizer_tickets(&mut self, dimension: &str, pos: BlockPos) {
        if let Some(held) = self.stabilizer_tickets.remove(&(dimension.to_owned(), pos.x, pos.y, pos.z)) {
            for c in held {
                let _ = chunks::set_forced(dimension, c.0, c.1, false);
            }
        }
    }

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

            // The buttons of its screen since the last tick (`PacketGuiInteract`): a chunk, or a ring of chunks on or off.
            let key = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
            let mut pressed: Vec<i32> = Vec::new();
            self.stabilizer_buttons.retain(|(d, p, id)| {
                if *d == e.dimension && (p.x, p.y, p.z) == (e.pos.x, e.pos.y, e.pos.z) {
                    pressed.push(*id);
                    false
                } else {
                    true
                }
            });
            for id in pressed {
                if (BUTTON_TOGGLE_CHUNK_BASE..BUTTON_TOGGLE_CHUNK_BASE + 25).contains(&id) {
                    let offset = (id - BUTTON_TOGGLE_CHUNK_BASE) as usize;
                    state.toggle_chunk(offset / MAX_LOAD_DIAMETER, offset % MAX_LOAD_DIAMETER);
                } else if id == crate::redstone::NEXT {
                    state.control = state.control.next();
                } else if id == crate::redstone::PREVIOUS {
                    state.control = state.control.previous();
                } else if (BUTTON_RING_ENABLE_BASE + 1..=BUTTON_RING_ENABLE_BASE + MAX_LOAD_RADIUS).contains(&id) {
                    state.adjust_radius(id - BUTTON_RING_ENABLE_BASE, true);
                } else if (BUTTON_RING_DISABLE_BASE + 1..=BUTTON_RING_DISABLE_BASE + MAX_LOAD_RADIUS).contains(&id) {
                    state.adjust_radius(id - BUTTON_RING_DISABLE_BASE, false);
                }
            }

            // It works when the config allows chunk loading and the redstone says so (`canFunction`), and takes the energy of every chunk each tick.
            let powered = world::get_redstone(&e.dimension, e.pos.x, e.pos.y, e.pos.z) > 0;
            let last = self.last_power.insert(key.clone(), powered).unwrap_or(false);
            let can_function = state.control.can_function(powered, last);
            let needed = state.energy_per_tick();
            let was_active = state.active;
            if can_function && energy >= needed {
                energy -= needed;
                state.active = true;
            } else {
                state.active = false;
            }
            if was_active != state.active {
                let _ = world::set_block_property(&e.dimension, e.pos.x, e.pos.y, e.pos.z, "active", if state.active { "true" } else { "false" });
            }

            state.energy = energy;

            // Sync chunk tickets with host
            let loaded_chunks = state.get_loaded_chunk_set(e.pos.x, e.pos.z);
            self.sync_stabilizer_tickets(&key, &loaded_chunks, state.active);

            let numbers = state.menu_numbers(energy);
            let changed = was_active != state.active || energy != e.energy || e.menu_data != numbers || e.data != encode(&state);
            if changed {
                let menu_data = numbers;

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
    fn test_rings_change_only_their_own_chunks() {
        let mut state = State::default();
        // Ring 1: the 8 chunks around the middle, so 9 loaded.
        assert!(state.adjust_radius(1, true));
        assert_eq!(state.chunks_loaded, 9);
        assert_eq!(state.energy_per_tick(), 18_000); // 9 * 2_000 FE/t
        // Ring 2: the 16 chunks of the outer border, so 25.
        assert!(state.adjust_radius(2, true));
        assert_eq!(state.chunks_loaded, 25);
        assert_eq!(state.energy_per_tick(), 50_000);
        // Turning ring 2 off leaves ring 1 (java:122-146 only touches the border), and the middle one is never turned off.
        assert!(state.adjust_radius(2, false));
        assert_eq!(state.chunks_loaded, 9);
        assert!(state.loading_chunks[1][1] && state.loading_chunks[2][2] && !state.loading_chunks[0][0]);
        assert!(state.adjust_radius(1, false));
        assert_eq!(state.chunks_loaded, 1);
        assert!(state.loading_chunks[2][2]);
        // A radius out of range, or a ring already as asked, changes nothing.
        assert!(!state.adjust_radius(3, true) && !state.adjust_radius(0, true) && !state.adjust_radius(1, false));
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
        state.adjust_radius(1, true);
        state.control = Control::Low;

        let encoded = encode(&state);
        let decoded = decode(&encoded);

        assert_eq!(decoded.energy, 12_500);
        assert_eq!(decoded.active, true);
        assert_eq!(decoded.energy_upgrades, 3);
        assert_eq!(decoded.chunks_loaded, 9);
        assert_eq!(decoded.loading_chunks, state.loading_chunks);
    }

    #[test]
    fn test_screen_numbers_and_the_middle_button_radii() {
        let mut state = State::default();
        // Only the middle chunk: left click turns on ring 1, nothing is on at the rings to turn off.
        let n = state.menu_numbers(1_000);
        assert_eq!(n.len(), 13 + 25 + 1);
        assert_eq!(n[38], 1 * 3 + 0);
        assert_eq!((n[8], n[9]), (1, 0));
        assert_eq!(n[13 + 2 * 5 + 2], 1, "the middle chunk");
        assert_eq!(n[13], 0);
        // Ring 1 on: left click now turns on ring 2, right click turns ring 1 off.
        state.adjust_radius(1, true);
        let n = state.menu_numbers(1_000);
        assert_eq!((n[8], n[9]), (2, 1));
        // Everything on: nothing to turn on, the outermost ring turns off.
        state.adjust_radius(2, true);
        let n = state.menu_numbers(1_000);
        assert_eq!((n[8], n[9]), (0, 2));
        assert_eq!(n[5], 25);
        // The energy halves.
        assert_eq!(n[0], 1_000);
    }
}

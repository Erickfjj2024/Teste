//! Laser Tractor Beam (`TileEntityLaserTractorBeam`, `TileEntityLaserReceptor`, `TileEntityBasicLaser`):
//! Absorbs laser energy, breaks target blocks by energy proportional to hardness, collects block drops
//! directly into its 27 internal inventory slots, attracts floating item entities, and expels overflow.
//!
//! Mekanism 1.21.1 source of truth:
//! - `mekanism.common.tile.laser.TileEntityLaserTractorBeam` (`TileEntityLaserTractorBeam.java`)
//! - `mekanism.common.tile.laser.TileEntityLaserReceptor` (`TileEntityLaserReceptor.java`)
//! - `mekanism.common.tile.laser.TileEntityBasicLaser` (`TileEntityBasicLaser.java`)
//! - `mekanism.client.gui.GuiLaserTractorBeam` (`GuiLaserTractorBeam.java`)
//! - `mekanism.common.capabilities.energy.LaserEnergyContainer` (`LaserEnergyContainer.java`)
//! - `mekanism.common.config.StorageConfig` (`StorageConfig.java`)
//! - `mekanism.common.config.GeneralConfig` (`GeneralConfig.java`)

use super::*;
use crate::laser::{face_of, facing_name, direction, opposite};

// ============================================================================
// Constants (StorageConfig.java:98 & GeneralConfig.java:237-238)
// ============================================================================

pub const BLOCK: &str = "mekanism:laser_tractor_beam";

/// Total buffer capacity in Joules (5 GJ).
/// Java: `StorageConfig.laserTractorBeam = 5_000_000_000L` (StorageConfig.java:98).
pub const CAPACITY_JOULES: u64 = 5_000_000_000;

/// Total buffer capacity in Forge Energy: `fe(5_000_000_000) = 2_000_000_000 FE`.
pub const CAPACITY_FE: u64 = fe(CAPACITY_JOULES);

/// Laser energy required per 1.0 block hardness point in Joules.
/// Java: `GeneralConfig.laserEnergyPerHardness = 100_000L` (GeneralConfig.java:238).
pub const LASER_ENERGY_PER_HARDNESS_JOULES: u64 = 100_000;

/// Laser energy per hardness in Forge Energy: `fe(100_000) = 40_000 FE`.
pub const LASER_ENERGY_PER_HARDNESS_FE: u64 = fe(LASER_ENERGY_PER_HARDNESS_JOULES);

/// Energy per damage point for entity hurt: 2,500 Joules = 1,000 FE.
/// Java: `GeneralConfig.laserEnergyPerDamage = 2_500L` (GeneralConfig.java:239).
pub const LASER_ENERGY_PER_DAMAGE_FE: u64 = fe(2_500);

/// Maximum laser reach in blocks.
/// Java: `GeneralConfig.laserRange = 64` (GeneralConfig.java:237).
pub const RANGE: i32 = 64;

/// Inventory container slots (3 rows of 9 slots = 27 slots).
/// Java: `TileEntityLaserTractorBeam.java:46-52`.
pub const SLOTS: usize = 27;

// ============================================================================
// Item Slot Representation
// ============================================================================

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InventoryItem {
    pub id: String,
    pub count: u32,
    pub max_stack: u32,
}

impl InventoryItem {
    pub fn new(id: &str, count: u32, max_stack: u32) -> Self {
        InventoryItem {
            id: id.to_owned(),
            count,
            max_stack: max_stack.max(1),
        }
    }
}

// ============================================================================
// Laser Tractor Beam Structure
// ============================================================================

#[derive(Clone, Debug, PartialEq)]
pub struct LaserTractorBeam {
    /// Energy buffer stored in Forge Energy (up to 2,000,000,000 FE).
    pub energy_fe: u64,
    /// Facing orientation (0: Down, 1: Up, 2: North, 3: South, 4: West, 5: East).
    pub facing: usize,
    /// 27 inventory output slots.
    pub slots: [Option<InventoryItem>; SLOTS],
    /// Block position currently being dug.
    pub digging_pos: Option<[i32; 3]>,
    /// Accumulated digging progress on target block in Forge Energy.
    pub digging_progress_fe: u64,
    /// Active firing status.
    pub active: bool,
}

impl Default for LaserTractorBeam {
    fn default() -> Self {
        Self::new(2) // Default North
    }
}

impl LaserTractorBeam {
    pub fn new(facing: usize) -> Self {
        const NONE_ITEM: Option<InventoryItem> = None;
        LaserTractorBeam {
            energy_fe: 0,
            facing: facing.min(5),
            slots: [NONE_ITEM; SLOTS],
            digging_pos: None,
            digging_progress_fe: 0,
            active: false,
        }
    }

    /// `ILaserReceptor.canLasersDig()`: always returns false.
    /// Java: `TileEntityLaserReceptor.java:24`.
    pub fn can_lasers_dig(&self) -> bool {
        false
    }

    /// `ILaserReceptor.receiveLaserEnergy(long energy)`: receives laser beam energy into buffer.
    /// Java: `TileEntityLaserReceptor.java:18-20`.
    pub fn receive_laser_energy(&mut self, energy_fe: u64) -> u64 {
        let space = CAPACITY_FE.saturating_sub(self.energy_fe);
        let accepted = energy_fe.min(space);
        self.energy_fe += accepted;
        accepted
    }

    /// Inserts an item into the 27 internal inventory slots.
    /// Strategy matches Mekanism Java `InventoryUtils.insertItem`:
    /// 1. Tries to merge into existing non-full matching stacks.
    /// 2. Places remaining items into first empty slots.
    /// Returns any excess items that could not fit.
    ///
    /// Java reference: `TileEntityLaserTractorBeam.java:70-73` & `InventoryUtils.java`.
    pub fn insert_item(&mut self, item_id: &str, mut count: u32, max_stack: u32) -> u32 {
        if count == 0 {
            return 0;
        }

        // Pass 1: Existing matching stacks with room
        for slot in self.slots.iter_mut().flatten() {
            if slot.id == item_id && slot.count < slot.max_stack {
                let space = slot.max_stack - slot.count;
                let to_add = count.min(space);
                slot.count += to_add;
                count -= to_add;
                if count == 0 {
                    return 0;
                }
            }
        }

        // Pass 2: Empty slots
        for slot in self.slots.iter_mut() {
            if slot.is_none() {
                let to_add = count.min(max_stack);
                *slot = Some(InventoryItem::new(item_id, to_add, max_stack));
                count -= to_add;
                if count == 0 {
                    return 0;
                }
            }
        }

        count
    }

    /// Handles floating ItemEntity interception in the path of the laser beam.
    /// Java: `TileEntityLaserTractorBeam.java:87-97`:
    /// `handleHitItem(ItemEntity entity)`: inserts into inventory, discards entity if fully grabbed.
    /// Returns remainder count that could not be absorbed.
    pub fn handle_hit_item(&mut self, item_id: &str, count: u32, max_stack: u32) -> u32 {
        self.insert_item(item_id, count, max_stack)
    }

    /// Calculates analog comparator signal (0..=15) based on inventory fullness.
    /// Java: `Container.getRedstoneSignalFromContainer` & `TileEntityLaserTractorBeam.java`.
    pub fn comparator_level(&self) -> u8 {
        let mut total_fraction = 0.0;
        let mut item_count = 0;

        for slot in self.slots.iter().flatten() {
            total_fraction += (slot.count as f64) / (slot.max_stack as f64);
            item_count += slot.count;
        }

        if item_count == 0 {
            0
        } else {
            let level = ((total_fraction / (SLOTS as f64)) * 14.0).floor() as u8 + 1;
            level.min(15)
        }
    }

    /// Progresses digging on target block.
    /// - `target_pos`: block coordinate hit by laser beam.
    /// - `hardness`: block hardness ($H \ge 0$).
    /// - `energy_available_fe`: energy delivered by laser this tick.
    /// - `drops`: item drops produced if block breaks.
    ///
    /// When `digging_progress_fe >= hardness * 40_000 FE`:
    /// - Consumes necessary energy.
    /// - Collects drops into 27 internal slots.
    /// - Any overflow drops that could not fit are returned (to be spawned in world via `Block.popResourceFromFace`).
    ///
    /// Java reference: `TileEntityLaserTractorBeam.java:57-85` & `TileEntityBasicLaser.java:286-302`.
    pub fn tick_digging(
        &mut self,
        target_pos: [i32; 3],
        hardness: f32,
        energy_available_fe: u64,
        drops: &[(&str, u32, u32)], // (id, count, max_stack)
    ) -> Option<Vec<(String, u32)>> {
        if hardness < 0.0 {
            // Unbreakable block (bedrock)
            self.digging_pos = None;
            self.digging_progress_fe = 0;
            return None;
        }

        // Reset progress if target block moved
        if self.digging_pos != Some(target_pos) {
            self.digging_pos = Some(target_pos);
            self.digging_progress_fe = 0;
        }

        let needed_fe = ((hardness as f64) * (LASER_ENERGY_PER_HARDNESS_FE as f64)).round() as u64;
        self.digging_progress_fe += energy_available_fe;

        if self.digging_progress_fe >= needed_fe {
            // Block successfully mined!
            self.digging_pos = None;
            self.digging_progress_fe = 0;

            let mut overflow = Vec::new();
            for &(item_id, count, max_stack) in drops {
                let remainder = self.insert_item(item_id, count, max_stack);
                if remainder > 0 {
                    // Item overflow: expels into world
                    // Java: TileEntityLaserTractorBeam.java:74-82:
                    // Block.popResourceFromFace(level, dropPos, opposite, drop);
                    overflow.push((item_id.to_owned(), remainder));
                }
            }
            Some(overflow)
        } else {
            None
        }
    }

    /// Serializes Laser Tractor Beam to bytes:
    /// byte 0: version (1)
    /// byte 1: facing (0..=5)
    /// byte 2: active (0/1)
    /// bytes 3..11: energy_fe (u64 LE)
    /// bytes 11..19: digging_progress_fe (u64 LE)
    /// byte 19: slot count (27)
    /// for each slot:
    ///   byte 0: has_item (0/1)
    ///   if has_item:
    ///     bytes 1..5: count (u32 LE)
    ///     bytes 5..9: max_stack (u32 LE)
    ///     byte 9: id length (N)
    ///     bytes 10..10+N: id string
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(128);
        out.push(1); // version
        out.push(self.facing as u8);
        out.push(u8::from(self.active));
        out.extend(self.energy_fe.to_le_bytes());
        out.extend(self.digging_progress_fe.to_le_bytes());
        out.push(SLOTS as u8);

        for slot in &self.slots {
            if let Some(ref item) = slot {
                out.push(1);
                out.extend(item.count.to_le_bytes());
                out.extend(item.max_stack.to_le_bytes());
                let bytes = item.id.as_bytes();
                let len = bytes.len().min(255) as u8;
                out.push(len);
                out.extend_from_slice(&bytes[..len as usize]);
            } else {
                out.push(0);
            }
        }
        out
    }

    /// Deserializes Laser Tractor Beam from bytes.
    pub fn decode(data: &[u8]) -> Option<Self> {
        if data.len() < 20 || data[0] != 1 {
            return None;
        }
        let facing = usize::from(data[1]).min(5);
        let active = data[2] == 1;

        let mut u64_buf = [0u8; 8];
        u64_buf.copy_from_slice(&data[3..11]);
        let energy_fe = u64::from_le_bytes(u64_buf);

        u64_buf.copy_from_slice(&data[11..19]);
        let digging_progress_fe = u64::from_le_bytes(u64_buf);

        const NONE: Option<InventoryItem> = None;
        let mut slots = [NONE; SLOTS];
        let mut offset = 20;

        for slot in slots.iter_mut() {
            if offset >= data.len() {
                break;
            }
            let has_item = data[offset] == 1;
            offset += 1;

            if has_item {
                if offset + 9 > data.len() {
                    return None;
                }
                let mut u32_buf = [0u8; 4];
                u32_buf.copy_from_slice(&data[offset..offset + 4]);
                let count = u32::from_le_bytes(u32_buf);
                offset += 4;

                u32_buf.copy_from_slice(&data[offset..offset + 4]);
                let max_stack = u32::from_le_bytes(u32_buf);
                offset += 4;

                let id_len = data[offset] as usize;
                offset += 1;

                if offset + id_len > data.len() {
                    return None;
                }
                let id = std::str::from_utf8(&data[offset..offset + id_len]).ok()?.to_owned();
                offset += id_len;

                *slot = Some(InventoryItem::new(&id, count, max_stack));
            }
        }

        Some(LaserTractorBeam {
            energy_fe,
            facing,
            slots,
            digging_pos: None,
            digging_progress_fe,
            active,
        })
    }
}

// ============================================================================
// Host Bridges (FORK-7 & FORK-8: Laser Physics and Entity Interaction)
// ============================================================================

/// FORK-7: Mod-driven entity damage and item absorption.
pub fn host_laser_entity_damage(_dim: &str, _from: [f64; 3], _to: [f64; 3], _damage_fe: u64) {
    // FORK-7: Hurt entities intersecting beam
}

/// FORK-8: Energy-driven block breaking by hardness and item drop intercept.
pub fn host_pop_resource_from_face(_dim: &str, _pos: [i32; 3], _face: usize, _item: &str, _count: u32) {
    // FORK-8: Eject overflow item into world with directional velocity
}

// ============================================================================
// Unit Tests (comparing with Java source files and line numbers)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_laser_tractor_beam_constants_match_java() {
        // StorageConfig.java:98: laserTractorBeam = 5_000_000_000L
        assert_eq!(CAPACITY_JOULES, 5_000_000_000);
        assert_eq!(CAPACITY_FE, 2_000_000_000);

        // GeneralConfig.java:238: laserEnergyPerHardness = 100_000L
        assert_eq!(LASER_ENERGY_PER_HARDNESS_JOULES, 100_000);
        assert_eq!(LASER_ENERGY_PER_HARDNESS_FE, 40_000);

        // TileEntityLaserTractorBeam.java:46-52: 27 inventory slots (9x3)
        assert_eq!(SLOTS, 27);

        // GeneralConfig.java:237: laserRange = 64
        assert_eq!(RANGE, 64);
    }

    #[test]
    fn test_laser_receptor_properties() {
        // TileEntityLaserReceptor.java:24: canLasersDig = false
        let beam = LaserTractorBeam::new(2);
        assert!(!beam.can_lasers_dig());

        // TileEntityLaserReceptor.java:18-20: receiveLaserEnergy
        let mut beam = beam;
        let accepted = beam.receive_laser_energy(500_000);
        assert_eq!(accepted, 500_000);
        assert_eq!(beam.energy_fe, 500_000);

        // Clamps at capacity (2_000_000_000 FE)
        let accepted = beam.receive_laser_energy(3_000_000_000);
        assert_eq!(accepted, CAPACITY_FE - 500_000);
        assert_eq!(beam.energy_fe, CAPACITY_FE);
    }

    #[test]
    fn test_mining_and_drop_collection_stone() {
        // Stone hardness: 1.5
        // Energy required: 1.5 * 40_000 = 60_000 FE (150,000 Joules)
        let mut beam = LaserTractorBeam::new(2);
        let target = [10, 64, 10];
        let drops = [("minecraft:cobblestone", 1, 64)];

        // Tick 1: delivers 30,000 FE (not enough)
        let result = beam.tick_digging(target, 1.5, 30_000, &drops);
        assert!(result.is_none());
        assert_eq!(beam.digging_progress_fe, 30_000);

        // Tick 2: delivers another 30,000 FE (reaches 60,000 FE -> mined!)
        let result = beam.tick_digging(target, 1.5, 30_000, &drops);
        assert!(result.is_some());
        let overflow = result.unwrap();
        assert!(overflow.is_empty()); // Stored in internal slots, no overflow

        // Drop placed into slot 0
        assert_eq!(beam.slots[0], Some(InventoryItem::new("minecraft:cobblestone", 1, 64)));
    }

    #[test]
    fn test_mining_obsidian_hardness_and_energy() {
        // Obsidian hardness: 50.0
        // Energy required: 50.0 * 40_000 = 2_000_000 FE (5,000,000 Joules)
        let mut beam = LaserTractorBeam::new(2);
        let target = [0, 10, 0];
        let drops = [("minecraft:obsidian", 1, 64)];

        let result = beam.tick_digging(target, 50.0, 1_999_999, &drops);
        assert!(result.is_none());
        assert_eq!(beam.digging_progress_fe, 1_999_999);

        let result = beam.tick_digging(target, 50.0, 1, &drops);
        assert!(result.is_some());
        assert_eq!(beam.slots[0], Some(InventoryItem::new("minecraft:obsidian", 1, 64)));
    }

    #[test]
    fn test_inventory_overflow_ejection() {
        // Fill all 27 slots to capacity with 64 dirt each
        let mut beam = LaserTractorBeam::new(2);
        for slot in beam.slots.iter_mut() {
            *slot = Some(InventoryItem::new("minecraft:dirt", 64, 64));
        }

        // Now break a block yielding diamond
        let target = [5, 64, 5];
        let drops = [("minecraft:diamond", 3, 64)];
        let result = beam.tick_digging(target, 3.0, 120_000, &drops);

        assert!(result.is_some());
        let overflow = result.unwrap();
        // Since inventory is full, all 3 diamonds overflow for world ejection
        assert_eq!(overflow, vec![("minecraft:diamond".to_owned(), 3)]);
    }

    #[test]
    fn test_item_entity_absorption() {
        // TileEntityLaserTractorBeam.java:87-97
        let mut beam = LaserTractorBeam::new(2);
        beam.slots[0] = Some(InventoryItem::new("minecraft:iron_ingot", 10, 64));

        // Floating entity with 5 iron ingots absorbs into existing stack
        let remainder = beam.handle_hit_item("minecraft:iron_ingot", 5, 64);
        assert_eq!(remainder, 0);
        assert_eq!(beam.slots[0].as_ref().unwrap().count, 15);

        // Floating entity with 60 gold ingots absorbs into new slot 1
        let remainder = beam.handle_hit_item("minecraft:gold_ingot", 60, 64);
        assert_eq!(remainder, 0);
        assert_eq!(beam.slots[1], Some(InventoryItem::new("minecraft:gold_ingot", 60, 64)));
    }

    #[test]
    fn test_comparator_output_levels() {
        let mut beam = LaserTractorBeam::new(2);
        assert_eq!(beam.comparator_level(), 0);

        // 1 item in 1 slot -> level 1
        beam.slots[0] = Some(InventoryItem::new("minecraft:stone", 1, 64));
        assert_eq!(beam.comparator_level(), 1);

        // Fill all 27 slots with full stacks -> level 15
        for slot in beam.slots.iter_mut() {
            *slot = Some(InventoryItem::new("minecraft:stone", 64, 64));
        }
        assert_eq!(beam.comparator_level(), 15);
    }

    #[test]
    fn test_laser_tractor_beam_serialization_roundtrip() {
        let mut beam = LaserTractorBeam::new(3); // South
        beam.energy_fe = 1_500_000;
        beam.digging_progress_fe = 25_000;
        beam.active = true;
        beam.slots[0] = Some(InventoryItem::new("minecraft:redstone", 32, 64));
        beam.slots[26] = Some(InventoryItem::new("mekanism:ingot_osmium", 12, 64));

        let bytes = beam.encode();
        let decoded = LaserTractorBeam::decode(&bytes).expect("decode failed");

        assert_eq!(decoded.facing, 3);
        assert!(decoded.active);
        assert_eq!(decoded.energy_fe, 1_500_000);
        assert_eq!(decoded.digging_progress_fe, 25_000);
        assert_eq!(decoded.slots[0], Some(InventoryItem::new("minecraft:redstone", 32, 64)));
        assert_eq!(decoded.slots[26], Some(InventoryItem::new("mekanism:ingot_osmium", 12, 64)));
        assert_eq!(decoded.slots[1], None);
    }
}

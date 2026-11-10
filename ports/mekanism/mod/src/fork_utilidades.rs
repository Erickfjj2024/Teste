//! Host fork functions for the utilities and transport modules (grupo: utilidades).
//!
//! Functions that are required by Mekanism logic but are not currently available
//! in the upstream Horizon Mod SDK are isolated here behind feature flags.
//!
//! When compiling without the `fork` feature (standard Horizon SDK), stub fallbacks
//! are provided. When `feature = "fork"` is enabled, calls are delegated to the
//! host environment API extensions.

use horizon_mod_sdk::server::world;

/// FORK-23: Sets a block in the world directly.
///
/// Java: `BlockCardboardBox.java:39-58` unpacks and places the previously captured block
/// state and tile entity into the world.
#[cfg(feature = "fork")]
pub fn set_block(dimension: &str, x: i32, y: i32, z: i32, block: &str) -> Result<(), String> {
    world::set_block(dimension, x, y, z, block)
}

#[cfg(not(feature = "fork"))]
pub fn set_block(_dimension: &str, _x: i32, _y: i32, _z: i32, _block: &str) -> Result<(), String> {
    Err("FORK-23 pendente".into())
}

/// FORK-35: Queries whether a position receives a redstone signal from any neighbor block.
///
/// Java: `Level.hasNeighborSignal(BlockPos)` in `TileEntityFormulaicAssemblicator.java:340`.
/// Used to detect redstone pulses for auto-crafting triggers.
#[cfg(feature = "fork")]
pub fn has_neighbor_signal(dimension: &str, x: i32, y: i32, z: i32) -> bool {
    world::has_neighbor_signal(dimension, x, y, z)
}

#[cfg(not(feature = "fork"))]
pub fn has_neighbor_signal(dimension: &str, x: i32, y: i32, z: i32) -> bool {
    world::get_redstone(dimension, x, y, z) > 0
}

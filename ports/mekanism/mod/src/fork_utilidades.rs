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

/// A crafting recipe the game found for a 3x3 grid.
#[derive(Clone, Debug, PartialEq)]
pub struct CraftMatch {
    pub id: String,
    pub output: horizon_mod_sdk::server::ItemStack,
    /// What the recipe leaves in the grid (`getRemainingItems`: the bucket of a cake), empty stacks left out.
    pub remaining: Vec<horizon_mod_sdk::server::ItemStack>,
}

/// `crafting-find` (API 28): the first crafting recipe of the game (vanilla and mods) that the 3x3 grid makes,
/// `RecipeManager.getRecipeFor(RecipeType.CRAFTING, input, level)`.
pub fn crafting_find(grid: &[Option<horizon_mod_sdk::server::ItemStack>; 9]) -> Option<CraftMatch> {
    let m = horizon_mod_sdk::server::recipe_types::crafting_find(&grid.to_vec())?;
    Some(CraftMatch { id: m.id, output: m.output, remaining: m.remaining.into_iter().flatten().collect() })
}

/// `crafting-matches` (API 28): whether the recipe `id` accepts the grid (`recipe.matches(input, level)`).
pub fn crafting_matches(id: &str, grid: &[Option<horizon_mod_sdk::server::ItemStack>; 9]) -> bool {
    horizon_mod_sdk::server::recipe_types::crafting_matches(id, &grid.to_vec())
}

/// FORK-20 (pedido em PROPOSTA-API.wit, `item-tag-members`): the items of an item tag, in the order the game lists them. Until the host has it the
/// list is empty, and the Oredictionificator cannot choose a result for a new filter.
pub fn item_tag_members(_tag: &str) -> Vec<String> {
    Vec::new()
}

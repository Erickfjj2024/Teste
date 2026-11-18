//! FORK-34 (`world.set-capabilities-enabled`): what a multiblock's valves and ports let through. In Java the multiblock's tanks, energy
//! container and inventory (`MultiblockData.getFluidTanks`, `getEnergyContainers`...) are empty lists while the structure is not formed, so
//! nothing reaches the valve from outside; here the valve's capabilities are closed to the outside until the structure is valid again
//! (its contents are kept where they were).
use super::*;

/// Closes (`formed` false) or opens (`formed` true) the items, fluids and energy of the valve or port at `pos`.
pub fn set_formed(dimension: &str, pos: BlockPos, formed: bool) {
    let _ = world::set_capabilities_enabled(dimension, pos.x, pos.y, pos.z, formed, formed, formed);
}

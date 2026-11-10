//! The Meka-Tool in the world: what the right click does with the pieces of `meka_tool`.
//!
//! Done here: the teleportation unit (`ItemMekaTool.use`, java:373-417): the ray, the destination check, the energy,
//! the teleport. Not done, because the game gives the mod no way yet: the extra attack damage and its energy
//! (`item-hit` only hears; FORK damage hook), the radial menu (FORK-27), the area and vein mining (FORK-31) and the
//! farming units (a use that changes blocks with the tool's own rules). Nothing here charges energy for an effect the
//! game does not apply.

use horizon_mod_sdk::server::{player_control, player_items, world, ItemStack, Player};

use crate::{meka_tool, Mekanism};

/// Whether a ray passes through the block: air and fluids (`ClipContext.Fluid.NONE` skips fluids; a plant or a torch has an outline and is hit).
pub fn passes_ray(block: Option<&str>) -> bool {
    let Some(state) = block else { return true };
    let name = state.split('[').next().unwrap_or("");
    matches!(
        name,
        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air" | "minecraft:water" | "minecraft:lava" | "minecraft:bubble_column"
    )
}

/// `isValidDestinationBlock`: air or a fluid.
pub fn valid_destination(block: Option<&str>) -> bool {
    passes_ray(block)
}

/// The first block a ray from `eye` along `dir` meets within `reach` blocks (a voxel walk, Amanatides-Woo), or none for a miss.
pub fn ray_block(eye: [f64; 3], dir: [f64; 3], reach: f64, passable: impl Fn(i32, i32, i32) -> bool) -> Option<[i32; 3]> {
    let mut cell = [eye[0].floor() as i32, eye[1].floor() as i32, eye[2].floor() as i32];
    let mut step = [0i32; 3];
    let mut t_max = [f64::INFINITY; 3];
    let mut t_delta = [f64::INFINITY; 3];
    for i in 0..3 {
        if dir[i] > 0.0 {
            step[i] = 1;
            t_delta[i] = 1.0 / dir[i];
            t_max[i] = (f64::from(cell[i]) + 1.0 - eye[i]) / dir[i];
        } else if dir[i] < 0.0 {
            step[i] = -1;
            t_delta[i] = -1.0 / dir[i];
            t_max[i] = (eye[i] - f64::from(cell[i])) / -dir[i];
        }
    }
    loop {
        if !passable(cell[0], cell[1], cell[2]) {
            return Some(cell);
        }
        let axis = if t_max[0] <= t_max[1] && t_max[0] <= t_max[2] { 0 } else if t_max[1] <= t_max[2] { 1 } else { 2 };
        if t_max[axis] > reach {
            return None;
        }
        cell[axis] += step[axis];
        t_max[axis] += t_delta[axis];
    }
}

/// The components with the tool's stored energy replaced (`"energy_containers":[N]`), the rest byte-for-byte.
pub fn with_energy(components: &str, energy: u64) -> Option<String> {
    const KEY: &str = "\"energy_containers\":[";
    let at = components.find(KEY)? + KEY.len();
    let end = at + components[at..].find(']')?;
    let mut out = components.to_owned();
    out.replace_range(at..end, &energy.to_string());
    Some(out)
}

impl Mekanism {
    /// A right click with a Meka-Tool in the air or on a block. Answers whether the mod took the use.
    pub(crate) fn meka_tool_used(&mut self, player: &Player, stack: &ItemStack) -> bool {
        let data = meka_tool::host_read_meka_tool(&stack.components);
        if !data.is_module_enabled(meka_tool::ModuleType::TeleportationUnit) {
            return false;
        }
        let requires_target = data.get_module(meka_tool::ModuleType::TeleportationUnit).is_some_and(|m| m.teleport_requires_target);
        let sneaking = player_control::get_motion(&player.id).is_some_and(|m| m.sneaking);
        // `MekanismUtils.getHeadVec`: the eye, lowered 0.08 while crouching.
        let eye = [player.x, player.y + if sneaking { 1.62 - 0.08 } else { 1.62 }, player.z];
        let Some((yaw, pitch)) = player_control::get_look(&player.id) else { return false };
        let dir = crate::sides::look(yaw, pitch);
        let dim = player.dimension.clone();
        let hit = ray_block(eye, dir, f64::from(meka_tool::MAX_TELEPORT_REACH), |x, y, z| passes_ray(world::get_block(&dim, x, y, z).as_deref()));
        // A miss ends the ray at its far end (`BlockHitResult.getBlockPos` of a miss).
        let target = hit.unwrap_or_else(|| {
            let r = f64::from(meka_tool::MAX_TELEPORT_REACH);
            [(eye[0] + dir[0] * r).floor() as i32, (eye[1] + dir[1] * r).floor() as i32, (eye[2] + dir[2] * r).floor() as i32]
        });
        let head_free = valid_destination(world::get_block(&dim, target[0], target[1] + 1, target[2]).as_deref());
        let above_free = valid_destination(world::get_block(&dim, target[0], target[1] + 2, target[2]).as_deref());
        let Ok((dest, cost)) = meka_tool::check_teleport([player.x, player.y, player.z], target, head_free, above_free, requires_target, hit.is_some(), data.energy) else {
            return false;
        };
        let Some((slot, components)) = Self::find_hotbar_slot(&player.id, stack) else { return false };
        let Some(written) = with_energy(&components, data.energy - cost) else { return false };
        if player_control::teleport(&player.id, &dim, dest[0], dest[1], dest[2], None, None).is_err() {
            return false;
        }
        let _ = player_items::set_slot(&player.id, slot, Some(&ItemStack { item: stack.item.clone(), count: stack.count, components: written }));
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ray_stops_at_the_first_solid_block() {
        // A ray that starts inside a solid cell hits that cell at once.
        assert_eq!(ray_block([0.5, 0.5, 0.5], [1.0, 0.0, 0.0], 10.0, |_, _, _| false), Some([0, 0, 0]));
        let one_block = |x: i32, y: i32, z: i32| !(x == 5 && y == 0 && z == 0);
        assert_eq!(ray_block([0.5, 0.5, 0.5], [1.0, 0.0, 0.0], 10.0, one_block), Some([5, 0, 0]));
        let air_until_x5 = |x: i32, _y: i32, _z: i32| x < 5;
        assert_eq!(ray_block([0.5, 0.5, 0.5], [1.0, 0.0, 0.0], 10.0, air_until_x5), Some([5, 0, 0]));
    }

    #[test]
    fn the_ray_misses_beyond_its_reach() {
        assert_eq!(ray_block([0.5, 0.5, 0.5], [1.0, 0.0, 0.0], 3.0, |_, _, _| true), None);
    }

    #[test]
    fn the_ray_walks_a_diagonal_and_downwards() {
        let floor = |_x: i32, y: i32, _z: i32| y > -3;
        let hit = ray_block([0.5, 0.5, 0.5], [0.0, -1.0, 0.0], 20.0, floor);
        assert_eq!(hit, Some([0, -3, 0]));
    }

    #[test]
    fn air_and_fluids_pass_the_ray_and_a_plant_does_not() {
        assert!(passes_ray(Some("minecraft:air")));
        assert!(passes_ray(Some("minecraft:water[level=0]")));
        assert!(passes_ray(None));
        assert!(!passes_ray(Some("minecraft:short_grass")));
        assert!(!passes_ray(Some("minecraft:stone")));
    }

    #[test]
    fn the_energy_is_replaced_in_place() {
        let c = r#"{"mekanism:energy":{"energy_containers":[1000]},"minecraft:damage":3}"#;
        assert_eq!(with_energy(c, 250).as_deref(), Some(r#"{"mekanism:energy":{"energy_containers":[250]},"minecraft:damage":3}"#));
        assert_eq!(with_energy("{}", 5), None);
    }
}

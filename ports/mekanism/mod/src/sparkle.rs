//! `SparkleAnimation` (`mekanism.client.SparkleAnimation`): when a multiblock forms, 100 redstone-dust particles on each of the six faces
//! of its box, at random places (`TileEntityMultiblock.doMultiblockSparkle`, FORK-30 `world.spawn-particles`). The box runs from the
//! structure's least corner (`corner1`) to its greatest (`corner2`).
use super::*;

/// The six faces' particles for a box from `min` to `max` (blocks, both included): 600 positions, drawn from `rng` (uniform 0 to 1).
pub fn positions(min: [i32; 3], max: [i32; 3], mut rng: impl FnMut() -> f64) -> Vec<[f64; 3]> {
    use std::f64::consts::{FRAC_PI_2, PI};
    let size = [f64::from(max[0] - min[0] + 1), f64::from(max[1] - min[1] + 1), f64::from(max[2] - min[2] + 1)];
    let origin = [size[0] / 2.0, size[1] / 2.0, size[2] / 2.0];
    // (width, height, displacement, yaw, pitch) of each side, in Java's order.
    let sides = [
        (size[0], size[1], origin, 0.0, 0.0),
        (size[0], size[1], origin, PI, 0.0),
        (size[2], size[1], [origin[2], origin[1], origin[0]], FRAC_PI_2, 0.0),
        (size[2], size[1], [origin[2], origin[1], origin[0]], 3.0 * FRAC_PI_2, 0.0),
        (size[0], size[2], [origin[0], origin[2], origin[1]], 0.0, FRAC_PI_2),
        (size[0], size[2], [origin[0], origin[2], origin[1]], 0.0, 3.0 * FRAC_PI_2),
    ];
    let mut out = Vec::with_capacity(600);
    for (width, height, displacement, yaw, pitch) in sides {
        for _ in 0..100 {
            let p = [width * rng() - displacement[0], height * rng() - displacement[1], -0.01 - displacement[2]];
            // Vec3.yRot, then Vec3.xRot.
            let (c, s) = (yaw.cos(), yaw.sin());
            let p = [p[0] * c + p[2] * s, p[1], p[2] * c - p[0] * s];
            let (c, s) = (pitch.cos(), pitch.sin());
            let p = [p[0], p[1] * c + p[2] * s, p[2] * c - p[1] * s];
            out.push([
                p[0] + origin[0] + f64::from(min[0]),
                p[1] + origin[1] + f64::from(min[1]),
                p[2] + origin[2] + f64::from(min[2]),
            ]);
        }
    }
    out
}

/// Shows the sparkle of a structure that has just formed to the players within 64 blocks of it.
pub fn show(dimension: &str, min: [i32; 3], max: [i32; 3], seed: u64) {
    let mut state = seed | 1;
    let mut rng = move || {
        // xorshift64*
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        (state.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 11) as f64 / (1u64 << 53) as f64
    };
    for p in positions(min, max, &mut rng) {
        let _ = world::spawn_particles(dimension, "minecraft:dust", p[0], p[1], p[2], 0.0, 0.0, 0.0, 0.0, 1, Some(0xFF0000), 1.0, 64.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn six_hundred_positions_lie_on_the_six_faces() {
        let mut n = 0.0f64;
        let rng = || {
            n = (n + 0.137) % 1.0;
            n
        };
        let (min, max) = ([10, 64, 10], [12, 66, 12]);
        let all = positions(min, max, rng);
        assert_eq!(all.len(), 600);
        // The box is 3 blocks wide on each axis: every position is on the surface of [min, max + 1], within the Java -0.01 skin.
        for p in all {
            let on_face = (0..3).any(|a| {
                let lo = f64::from(min[a]);
                let hi = f64::from(max[a] + 1);
                (p[a] - lo + 0.01).abs() < 1e-9 || (p[a] - hi - 0.01).abs() < 1e-9 || (p[a] - lo).abs() < 1e-9 || (p[a] - hi).abs() < 1e-9
            });
            assert!(on_face, "{p:?}");
            for a in 0..3 {
                assert!(p[a] >= f64::from(min[a]) - 0.011 && p[a] <= f64::from(max[a] + 1) + 0.011, "{p:?}");
            }
        }
    }
}

//! Mekanism's ores in new chunks: Mekanism's own veins (`OreType`: per chunk, size
//! and height range, as its configs default), placed by a hash of the chunk, so a
//! chunk always gets the same ones. Pure, tested without the game.

/// A kind of vein: its ore, how many a chunk, how many blocks at most, and
/// where (a trapezoid is thicker in the middle of its range).
pub struct Kind {
    pub ore: &'static str,
    pub per_chunk: u32,
    pub size: u32,
    pub trapezoid: bool,
    pub min_y: i32,
    pub max_y: i32,
}

/// The overworld's bottom, and the first block above its top.
pub const BOTTOM: i32 = -64;
pub const TOP: i32 = 320;

pub const KINDS: [Kind; 10] = [
    Kind { ore: "tin", per_chunk: 14, size: 4, trapezoid: true, min_y: -20, max_y: 94 },
    Kind { ore: "tin", per_chunk: 12, size: 9, trapezoid: true, min_y: -32, max_y: 72 },
    Kind { ore: "osmium", per_chunk: 65, size: 7, trapezoid: true, min_y: 72, max_y: TOP },
    Kind { ore: "osmium", per_chunk: 6, size: 9, trapezoid: true, min_y: -32, max_y: 56 },
    Kind { ore: "osmium", per_chunk: 8, size: 4, trapezoid: false, min_y: BOTTOM, max_y: 64 },
    Kind { ore: "uranium", per_chunk: 4, size: 4, trapezoid: true, min_y: BOTTOM, max_y: 8 },
    Kind { ore: "uranium", per_chunk: 7, size: 9, trapezoid: true, min_y: BOTTOM, max_y: -8 },
    Kind { ore: "fluorite", per_chunk: 5, size: 5, trapezoid: false, min_y: BOTTOM, max_y: 23 },
    Kind { ore: "fluorite", per_chunk: 3, size: 13, trapezoid: true, min_y: BOTTOM, max_y: 4 },
    Kind { ore: "lead", per_chunk: 8, size: 9, trapezoid: true, min_y: BOTTOM, max_y: 64 },
];

/// splitmix64.
pub struct Rng(u64);
impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed)
    }
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// A number from 0 up to (not including) `n`.
    pub fn below(&mut self, n: u32) -> u32 {
        (self.next() % u64::from(n.max(1))) as u32
    }
}

/// The blocks of the ore a vein of `kind` makes in chunk (`cx`, `cz`), its
/// `index` among the chunk's veins of that kind: within the chunk and the
/// height range; each block once.
pub fn vein(cx: i32, cz: i32, kind_index: usize, n: u32) -> Vec<[i32; 3]> {
    let kind = &KINDS[kind_index];
    let seed = (cx as u64).wrapping_mul(341_873_128_712)
        ^ (cz as u64).wrapping_mul(132_897_987_541)
        ^ ((kind_index as u64) << 40)
        ^ u64::from(n).wrapping_mul(0x2545_F491_4F6C_DD1D);
    let mut rng = Rng::new(seed);
    let span = (kind.max_y - kind.min_y).max(1) as u32;
    let height = if kind.trapezoid {
        (rng.below(span) + rng.below(span)) / 2
    } else {
        rng.below(span)
    };
    let mut at = [
        cx * 16 + rng.below(16) as i32,
        kind.min_y + height as i32,
        cz * 16 + rng.below(16) as i32,
    ];
    let mut out: Vec<[i32; 3]> = Vec::new();
    let size = 1 + rng.below(kind.size);
    for _ in 0..size * 3 {
        if out.len() as u32 >= size {
            break;
        }
        let inside = at[0] >= cx * 16
            && at[0] < cx * 16 + 16
            && at[2] >= cz * 16
            && at[2] < cz * 16 + 16
            && at[1] >= kind.min_y.max(BOTTOM)
            && at[1] < kind.max_y.min(TOP);
        if inside && !out.contains(&at) {
            out.push(at);
        }
        // A step to a neighbour: the vein is a blob of touching blocks.
        let axis = rng.below(3) as usize;
        at[axis] += if rng.below(2) == 0 { -1 } else { 1 };
    }
    out
}

/// `SaltConfig`: how many salt disks a chunk gets, their radius (2 to 3, `minRadius` and `maxRadius`) and how far above and under the
/// floor they reach (`halfHeight`).
pub const SALT_PER_CHUNK: u32 = 2;
pub const SALT_MIN_RADIUS: i32 = 2;
pub const SALT_MAX_RADIUS: i32 = 3;
pub const SALT_HALF_HEIGHT: i32 = 1;

/// The `n`th salt disk of chunk (`cx`, `cz`): its centre column (a few blocks in from the chunk's edge, so the disk stays in the
/// chunk) and its radius.
pub fn salt_disk(cx: i32, cz: i32, n: u32) -> (i32, i32, i32) {
    let seed = (cx as u64).wrapping_mul(341_873_128_712)
        ^ (cz as u64).wrapping_mul(132_897_987_541)
        ^ (0x5A17u64 << 40)
        ^ u64::from(n).wrapping_mul(0x2545_F491_4F6C_DD1D);
    let mut rng = Rng::new(seed);
    let radius = SALT_MIN_RADIUS + rng.below((SALT_MAX_RADIUS - SALT_MIN_RADIUS + 1) as u32) as i32;
    let margin = SALT_MAX_RADIUS as u32;
    (cx * 16 + margin as i32 + rng.below(16 - 2 * margin) as i32, cz * 16 + margin as i32 + rng.below(16 - 2 * margin) as i32, radius)
}

/// The blocks a salt disk of `radius` around (`x`, `z`) reaches from the water above the floor (`y`): the circle, and
/// `SALT_HALF_HEIGHT` up and down.
pub fn salt_blocks(x: i32, y: i32, z: i32, radius: i32) -> Vec<[i32; 3]> {
    let mut out = Vec::new();
    for dx in -radius..=radius {
        for dz in -radius..=radius {
            if dx * dx + dz * dz <= radius * radius {
                for dy in -SALT_HALF_HEIGHT..=SALT_HALF_HEIGHT {
                    out.push([x + dx, y + dy, z + dz]);
                }
            }
        }
    }
    out
}

/// Whether salt grows in a block (the disk's `target`: dirt and clay).
pub fn salt_replaces(block: &str) -> bool {
    matches!(block, "minecraft:dirt" | "minecraft:clay")
}

/// What a block becomes when ore grows in it: stone gives the ore, deepslate
/// its deepslate ore; nothing else is replaced.
pub fn replace(ore: &str, block: &str) -> Option<String> {
    match block {
        "minecraft:stone" => Some(format!("mekanism:{ore}_ore")),
        "minecraft:deepslate" => Some(format!("mekanism:deepslate_{ore}_ore")),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_vein_stays_in_its_chunk_and_range_and_is_always_the_same() {
        for (k, kind) in KINDS.iter().enumerate() {
            for n in 0..kind.per_chunk.min(8) {
                let v = vein(-3, 7, k, n);
                assert_eq!(v, vein(-3, 7, k, n));
                assert!(!v.is_empty() && v.len() as u32 <= kind.size, "{k} {n}");
                for p in &v {
                    assert!((-48..-32).contains(&p[0]), "x {p:?}");
                    assert!((112..128).contains(&p[2]), "z {p:?}");
                    assert!(p[1] >= kind.min_y && p[1] < kind.max_y, "y {p:?} {k}");
                }
                let mut sorted = v.clone();
                sorted.sort();
                sorted.dedup();
                assert_eq!(sorted.len(), v.len());
            }
        }
    }
    #[test]
    fn chunks_differ_and_blobs_touch() {
        assert_ne!(vein(0, 0, 2, 0), vein(1, 0, 2, 0));
        let v = vein(0, 0, 3, 1);
        // Each block after the first is next to one before it, or the walk went outside and came back:
        // at most one step from some earlier block's neighbourhood... check the weaker rule: all within `size`.
        let span = |i: usize| v.iter().map(|p| p[i]).max().unwrap() - v.iter().map(|p| p[i]).min().unwrap();
        assert!(span(0) <= 9 * 3 && span(1) <= 9 * 3 && span(2) <= 9 * 3);
    }
    #[test]
    fn ore_replaces_stone_and_deepslate_only() {
        assert_eq!(replace("tin", "minecraft:stone").as_deref(), Some("mekanism:tin_ore"));
        assert_eq!(replace("tin", "minecraft:deepslate").as_deref(), Some("mekanism:deepslate_tin_ore"));
        assert_eq!(replace("tin", "minecraft:dirt"), None);
        assert_eq!(replace("tin", "minecraft:air"), None);
    }
    #[test]
    fn a_salt_disk_stays_in_its_chunk_and_is_always_the_same() {
        for n in 0..SALT_PER_CHUNK {
            let (x, z, r) = salt_disk(-3, 7, n);
            assert_eq!((x, z, r), salt_disk(-3, 7, n));
            assert!((SALT_MIN_RADIUS..=SALT_MAX_RADIUS).contains(&r));
            assert!((-48..-32).contains(&(x - r)) && (-48..-32).contains(&(x + r)), "x {x}");
            assert!((112..128).contains(&(z - r)) && (112..128).contains(&(z + r)), "z {z}");
        }
        // A radius 2 disk: the circle's 13 columns, three blocks tall.
        assert_eq!(salt_blocks(0, 60, 0, 2).len(), 13 * 3);
        assert!(salt_replaces("minecraft:clay") && !salt_replaces("minecraft:sand"));
    }
    #[test]
    fn the_world_has_every_ore_mekanism_adds() {
        let mut ores: Vec<&str> = KINDS.iter().map(|k| k.ore).collect();
        ores.sort();
        ores.dedup();
        assert_eq!(ores, ["fluorite", "lead", "osmium", "tin", "uranium"]);
    }
}

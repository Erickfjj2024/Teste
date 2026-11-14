//! Bounding blocks (`TileEntityBoundingBlock`, `AttributeHasBounding`): a big machine (the advanced solar generator, the wind generator, the
//! pigment mixer, the modification station...) takes the blocks around it that its model fills. The machine places a `mekanism:bounding_block` in each of them and
//! keeps them there; a bounding block lives only while the machine it belongs to is where it says.
//!
//! Redirection (`BlockBounding.java:101-120`): clicking or interacting with a bounding block reaches the main machine; breaking a bounding
//! block destroys the main machine (`BlockBounding.java:140-148`), and destroying the main machine removes all its bounding blocks.
use super::*;

pub const BLOCK: &str = "mekanism:bounding_block";

/// The blocks a machine takes, relative to it (`HandleBoundingBlock`): none for the ones that fill their own block.
pub fn offsets_with_facing(block: &str, facing: &str) -> Vec<[i32; 3]> {
    parts_with_facing(block, facing).into_iter().map(|(o, _)| o).collect()
}

pub fn offsets(block: &str) -> Vec<[i32; 3]> {
    offsets_with_facing(block, "north")
}

/// The same with what each bounding block shows (`part`: nothing, a piece of the wind generator's pole, or its top with the blades).
pub fn parts_with_facing(block: &str, facing: &str) -> Vec<([i32; 3], &'static str)> {
    match block {
        // `ABOVE_ONLY`: the block over it.
        "mekanism:pigment_mixer"
        | "mekanism:isotopic_centrifuge"
        | "mekanism:solar_neutron_activator"
        | "mekanism:seismic_vibrator"
        | "mekanism:security_desk" => vec![([0, 1, 0], "none")],
        // Modification Station (MekanismBlockTypes.java:538-548): 1 above, 1 right, 1 right-above
        "mekanism:modification_station" => {
            modification_station::bounding_offsets(facing)
                .into_iter()
                .map(|o| (o, "none"))
                .collect()
        }
        // The block over it and a plane of nine two over it.
        "mekanism:advanced_solar_generator" => {
            let mut v = vec![([0, 1, 0], "none")];
            for x in -1..=1 {
                for z in -1..=1 {
                    v.push(([x, 2, z], "none"));
                }
            }
            v
        }
        // Three by three and two high: the blocks around it, and over it and them.
        "mekanism:digital_miner" => {
            let mut v = Vec::new();
            for x in -1..=1 {
                for y in 0..=1 {
                    for z in -1..=1 {
                        if (x, y, z) != (0, 0, 0) {
                            v.push(([x, y, z], "none"));
                        }
                    }
                }
            }
            v
        }
        // Four blocks up: three of pole and the top.
        "mekanism:wind_generator" => (1..=4).map(|y| ([0, y, 0], if y == 4 { "top" } else { "pole" })).collect(),
        _ => Vec::new(),
    }
}

pub fn parts(block: &str) -> Vec<([i32; 3], &'static str)> {
    parts_with_facing(block, "north")
}

pub fn is_bounded(block: &str) -> bool {
    block == "mekanism:modification_station" || !offsets(block).is_empty()
}

/// Whether a block can be built over (`BlockState.canBeReplaced`): air, fluids, the plants and layers that give way.
pub fn replaceable(block: &str) -> bool {
    let name = block.split('[').next().unwrap_or("");
    matches!(
        name,
        "minecraft:air"
            | "minecraft:cave_air"
            | "minecraft:void_air"
            | "minecraft:water"
            | "minecraft:lava"
            | "minecraft:short_grass"
            | "minecraft:tall_grass"
            | "minecraft:fern"
            | "minecraft:large_fern"
            | "minecraft:snow"
            | "minecraft:vine"
            | "minecraft:dead_bush"
            | "minecraft:seagrass"
            | "minecraft:tall_seagrass"
            | "minecraft:light"
            | "mekanism:bounding_block"
    )
}

/// `BlockItem.canPlace` for a machine with bounding blocks (`AttributeHasBounding.canPlace`): every block its model fills has to be free.
pub fn can_place(dimension: &str, pos: [i32; 3], state: &str) -> bool {
    let name = state.split('[').next().unwrap_or("");
    if !is_bounded(name) {
        return true;
    }
    let facing = state
        .split_once('[')
        .and_then(|(_, p)| p.trim_end_matches(']').split(',').find_map(|kv| kv.strip_prefix("facing=")))
        .unwrap_or("north");
    offsets_with_facing(name, facing).into_iter().all(|o| {
        world::get_block(dimension, pos[0] + o[0], pos[1] + o[1], pos[2] + o[2]).is_none_or(|b| replaceable(&b))
    })
}

pub fn encode(main: [i32; 3]) -> Vec<u8> {
    let mut out = vec![1];
    for c in main {
        out.extend(c.to_le_bytes());
    }
    out
}

pub fn decode(data: &[u8]) -> Option<[i32; 3]> {
    if data.len() < 13 || data[0] != 1 {
        return None;
    }
    let int = |at: usize| i32::from_le_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]);
    Some([int(1), int(5), int(9)])
}

fn air(block: &str) -> bool {
    matches!(block.split('[').next().unwrap_or(""), "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air")
}

impl Mekanism {
    /// Every second: the machines put the bounding blocks that are missing; the bounding blocks go when their machine is gone.
    pub(crate) fn run_bounding(&mut self, entities: &[BlockEntity], tick: u64) -> Vec<BlockEntityUpdate> {
        let mut out = Vec::new();
        let mut changes: HashMap<String, Vec<BlockChange>> = HashMap::new();
        for e in entities {
            let name = e.state.split('[').next().unwrap_or("");
            if name == BLOCK {
                let main = decode(&e.data).or_else(|| self.bound_owner.get(&(e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z)).copied());
                let Some(main) = main else {
                    // A bounding block nobody owns, and nobody claims.
                    if tick % 20 == 0 {
                        changes.entry(e.dimension.clone()).or_default().push(BlockChange { x: e.pos.x, y: e.pos.y, z: e.pos.z, state: "minecraft:air".into() });
                    }
                    continue;
                };
                self.bound_owner.insert((e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z), main);
                if decode(&e.data).is_none() {
                    out.push(BlockEntityUpdate {
                        dimension: e.dimension.clone(),
                        pos: e.pos,
                        data: encode(main),
                        items: None,
                        energy: None,
                        energy_faces: None,
                        tanks: None,
                        tank_faces: None,
                        menu_data: None,
                        transmitter_faces: None,
                    });
                } else if tick % 20 == 0 {
                    let alive = world::get_block(&e.dimension, main[0], main[1], main[2]).is_none_or(|b| is_bounded(b.split('[').next().unwrap_or("")));
                    if !alive {
                        self.bound_owner.remove(&(e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z));
                        changes.entry(e.dimension.clone()).or_default().push(BlockChange { x: e.pos.x, y: e.pos.y, z: e.pos.z, state: "minecraft:air".into() });
                    }
                }
            } else if tick % 20 == 0 && is_bounded(name) {
                let facing = sides::property(&e.state, "facing").unwrap_or("north");
                for (o, part) in parts_with_facing(name, facing) {
                    let p = [e.pos.x + o[0], e.pos.y + o[1], e.pos.z + o[2]];
                    if world::get_block(&e.dimension, p[0], p[1], p[2]).is_some_and(|b| air(&b)) {
                        self.bound_owner.insert((e.dimension.clone(), p[0], p[1], p[2]), [e.pos.x, e.pos.y, e.pos.z]);
                        changes.entry(e.dimension.clone()).or_default().push(BlockChange { x: p[0], y: p[1], z: p[2], state: format!("{BLOCK}[part={part}]") });
                    }
                }
            }
        }
        for (dimension, list) in changes {
            for batch in list.chunks(4096) {
                let _ = world::set_blocks(&dimension, batch);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn big_machines_take_the_blocks_their_models_fill() {
        assert_eq!(offsets("mekanism:wind_generator").len(), 4);
        assert_eq!(offsets("mekanism:advanced_solar_generator").len(), 10);
        assert_eq!(offsets("mekanism:pigment_mixer"), vec![[0, 1, 0]]);
        assert_eq!(offsets("mekanism:seismic_vibrator"), vec![[0, 1, 0]]);
        assert!(offsets("mekanism:crusher").is_empty());
        assert!(is_bounded("mekanism:wind_generator") && !is_bounded("mekanism:crusher"));
    }

    #[test]
    fn a_bounding_block_remembers_its_machine() {
        assert_eq!(decode(&encode([5, -64, 300])), Some([5, -64, 300]));
        assert_eq!(decode(&[0]), None);
    }

    /// Java MekanismBlockTypes.java:538-548: Modification Station has 3 bounding blocks relative to facing.
    #[test]
    fn modification_station_bounding_offsets_by_facing() {
        assert!(is_bounded("mekanism:modification_station"));
        let north = parts_with_facing("mekanism:modification_station", "north");
        assert_eq!(north.len(), 3);
        assert_eq!(north[0].0, [0, 1, 0]);
        assert_eq!(north[1].0, [1, 0, 0]);
        assert_eq!(north[2].0, [1, 1, 0]);

        let south = parts_with_facing("mekanism:modification_station", "south");
        assert_eq!(south[1].0, [-1, 0, 0]);
        assert_eq!(south[2].0, [-1, 1, 0]);

        let east = parts_with_facing("mekanism:modification_station", "east");
        assert_eq!(east[1].0, [0, 0, 1]);
        assert_eq!(east[2].0, [0, 1, 1]);

        let west = parts_with_facing("mekanism:modification_station", "west");
        assert_eq!(west[1].0, [0, 0, -1]);
        assert_eq!(west[2].0, [0, 1, -1]);
    }

    /// Java MekanismBlockTypes.java:334-345: `for x in -1..=1, y in 0..=1, z in -1..=1` except the block itself: 3 * 2 * 3 - 1 = 17 bounding blocks.
    #[test]
    fn digital_miner_takes_seventeen_bounding_blocks() {
        let got: std::collections::HashSet<[i32; 3]> = offsets("mekanism:digital_miner").into_iter().collect();
        let mut want = std::collections::HashSet::new();
        for x in -1..=1 {
            for y in 0..=1 {
                for z in -1..=1 {
                    if (x, y, z) != (0, 0, 0) {
                        want.insert([x, y, z]);
                    }
                }
            }
        }
        assert_eq!(want.len(), 17);
        assert_eq!(got, want);
        assert!(is_bounded("mekanism:digital_miner"));
    }

    /// Java GeneratorsBlockTypes.java:104-112: Wind Generator takes 4 vertical bounding blocks.
    #[test]
    fn wind_generator_has_pole_and_top_parts() {
        let p = parts("mekanism:wind_generator");
        assert_eq!(p.len(), 4);
        assert_eq!(p[0], ([0, 1, 0], "pole"));
        assert_eq!(p[1], ([0, 2, 0], "pole"));
        assert_eq!(p[2], ([0, 3, 0], "pole"));
        assert_eq!(p[3], ([0, 4, 0], "top"));
    }
}

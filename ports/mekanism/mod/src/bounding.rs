//! Bounding blocks (`TileEntityBoundingBlock`, `AttributeHasBounding`): a big machine (the advanced solar generator, the wind generator, the
//! pigment mixer...) takes the blocks around it that its model fills. The machine places a `mekanism:bounding_block` in each of them and
//! keeps them there (one that is broken is put back); a bounding block lives only while the machine it belongs to is where it says.
//!
//! Not here: clicking a bounding block does not reach the machine, and a machine is not refused where the blocks it needs are taken.
use super::*;

pub const BLOCK: &str = "mekanism:bounding_block";

/// The blocks a machine takes, relative to it (`HandleBoundingBlock`): none for the ones that fill their own block.
pub fn offsets(block: &str) -> Vec<[i32; 3]> {
    parts(block).into_iter().map(|(o, _)| o).collect()
}

/// The same with what each bounding block shows (`part`: nothing, a piece of the wind generator's pole, or its top with the blades).
pub fn parts(block: &str) -> Vec<([i32; 3], &'static str)> {
    match block {
        // `ABOVE_ONLY`: the block over it.
        "mekanism:pigment_mixer" | "mekanism:isotopic_centrifuge" | "mekanism:solar_neutron_activator" => vec![([0, 1, 0], "none")],
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

pub fn is_bounded(block: &str) -> bool {
    !offsets(block).is_empty()
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
                let main = decode(&e.data).or_else(|| self.bound_owner.remove(&(e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z)));
                let Some(main) = main else {
                    // A bounding block nobody owns, and nobody claims.
                    if tick % 20 == 0 {
                        changes.entry(e.dimension.clone()).or_default().push(BlockChange { x: e.pos.x, y: e.pos.y, z: e.pos.z, state: "minecraft:air".into() });
                    }
                    continue;
                };
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
                        changes.entry(e.dimension.clone()).or_default().push(BlockChange { x: e.pos.x, y: e.pos.y, z: e.pos.z, state: "minecraft:air".into() });
                    }
                }
            } else if tick % 20 == 0 && is_bounded(name) {
                for (o, part) in parts(name) {
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
        assert!(offsets("mekanism:crusher").is_empty());
        assert!(is_bounded("mekanism:wind_generator") && !is_bounded("mekanism:crusher"));
    }

    #[test]
    fn a_bounding_block_remembers_its_machine() {
        assert_eq!(decode(&encode([5, -64, 300])), Some([5, -64, 300]));
        assert_eq!(decode(&[0]), None);
    }
}

//! Mekanism for Horizon (a Rust port of Mekanism, MIT): the machines' logic.
//! The blocks, items, textures, recipes and screens are the mod's content
//! (`data/` and `assets/`, made by `tools/convert.py` from Mekanism's own).
//!
//! What runs so far: the crusher, the enrichment chamber and the energized
//! smelter (`machine.rs`), the energy cubes and the universal cable.
mod machine;
mod worldgen;

use horizon_mod_sdk::server::{
    BlockEntity, BlockEntityKind, BlockEntityUpdate, ContainerSpec, EnergySpec, FlowSide, ItemStack,
    BlockChange, Registration, ServerMod, TransmitterMedium, TransmitterSpec, recipe_types, serde_json, world,
};
use machine::{Book, Io, Recipe, Stack, step};
use std::collections::HashMap;

/// Where a machine's recipes come from.
#[derive(Clone, Copy)]
enum Source {
    /// Recipe files of a type of Mekanism's own (`mekanism:crushing`).
    Files(&'static str),
    /// The game's cooking recipes (`smelting`), vanilla's and the mods'.
    Cooking(&'static str),
}

/// An electric machine: its block, where its recipes come from and its energy
/// use a tick (`MekanismConfig.usage`).
struct Def {
    block: &'static str,
    source: Source,
    usage: u64,
}

const MACHINES: [Def; 3] = [
    Def { block: "mekanism:crusher", source: Source::Files("mekanism:crushing"), usage: 8 },
    Def { block: "mekanism:enrichment_chamber", source: Source::Files("mekanism:enriching"), usage: 5 },
    Def { block: "mekanism:energized_smelter", source: Source::Cooking("smelting"), usage: 5 },
];

/// Energy a machine holds: ticks of its use (an assumption: 26.3's own table is
/// still to be written upstream).
const STORED_TICKS: u64 = 2500;
const BASIC_CUBE: u64 = 4_000_000;
const BASIC_CUBE_RATE: u64 = 4_000;
const CREATIVE_CUBE: u64 = 1_000_000_000_000;
const CABLE_RATE: u64 = 1_024;

struct Mekanism {
    books: HashMap<&'static str, Book>,
}

fn kind(block: &str, ticks: bool) -> BlockEntityKind {
    BlockEntityKind {
        block: block.into(),
        ticks,
        container: None,
        energy: None,
        tanks: Vec::new(),
        keep_data: true,
        transmitter: None,
        multiblock: None,
    }
}

fn energy(capacity: u64, rate: u64, side: FlowSide) -> EnergySpec {
    EnergySpec {
        capacity,
        max_receive: rate,
        max_extract: rate,
        faces: vec![side; 6],
    }
}

fn load_book(source: Source) -> Book {
    let mut book = Book::default();
    match source {
        Source::Files(kind) => {
            for r in recipe_types::recipes_of_type(kind) {
                let Ok(json) = serde_json::from_str::<serde_json::Value>(&r.json) else {
                    continue;
                };
                let Some(output) = json["output"]["id"].as_str() else { continue };
                let inputs = recipe_types::ingredient_items(&json["input"]["ingredient"].to_string());
                book.add(
                    inputs,
                    Recipe {
                        count: json["input"]["count"].as_u64().unwrap_or(1) as u32,
                        output: output.to_owned(),
                        out_count: json["output"]["count"].as_u64().unwrap_or(1) as u32,
                    },
                );
            }
        }
        Source::Cooking(name) => {
            for c in recipe_types::cooking_recipes(name) {
                book.add(
                    c.inputs,
                    Recipe {
                        count: 1,
                        output: c.output,
                        out_count: c.count,
                    },
                );
            }
        }
    }
    book
}

fn stack(s: &Option<ItemStack>) -> Option<Stack> {
    s.as_ref().map(|s| Stack {
        item: s.item.clone(),
        count: s.count,
        components: s.components.clone(),
    })
}
fn item(s: &Option<Stack>) -> Option<ItemStack> {
    s.as_ref().map(|s| ItemStack {
        item: s.item.clone(),
        count: s.count,
        components: s.components.clone(),
    })
}

/// The screen's data slots: progress, then the energy and its capacity as low
/// and high 16 bits.
fn menu_data(progress: u32, energy: u64, capacity: u64) -> Vec<i32> {
    let half = |v: u64, shift: u32| ((v >> shift) & 0xFFFF) as i32;
    vec![progress as i32, half(energy, 0), half(energy, 16), half(capacity, 0), half(capacity, 16)]
}

impl Mekanism {
    /// A machine's tick: what it holds after, if that differs from before.
    fn run(&mut self, def: &Def, e: &BlockEntity) -> Option<BlockEntityUpdate> {
        let book = self
            .books
            .entry(def.block)
            .or_insert_with(|| load_book(def.source));
        let progress = e
            .data
            .get(..2)
            .map_or(0, |b| u32::from(u16::from_le_bytes([b[0], b[1]])));
        let io = Io {
            input: stack(e.items.first().unwrap_or(&None)),
            output: stack(e.items.get(1).unwrap_or(&None)),
            energy: e.energy,
            progress,
        };
        let (next, active) = step(def.usage, book, &io);
        // The block shows when it runs, with its entity kept.
        let lit = e.state.contains("lit=true");
        if active != lit {
            let _ = world::set_block_property(&e.dimension, e.pos.x, e.pos.y, e.pos.z, "lit", if active { "true" } else { "false" });
        }
        let capacity = def.usage * STORED_TICKS;
        let data = menu_data(next.progress, next.energy, capacity);
        if next == io && data == e.menu_data {
            return None;
        }
        Some(BlockEntityUpdate {
            dimension: e.dimension.clone(),
            pos: e.pos,
            data: (next.progress as u16).to_le_bytes().to_vec(),
            items: Some(vec![item(&next.input), item(&next.output)]),
            energy: Some(next.energy),
            energy_faces: None,
            tanks: None,
            tank_faces: None,
            menu_data: Some(data),
            transmitter_faces: None,
        })
    }
}

impl ServerMod for Mekanism {
    fn init() -> Result<(Self, Registration), String> {
        let mut r = Registration::new();
        for def in &MACHINES {
            let mut k = kind(def.block, true);
            k.container = Some(ContainerSpec {
                slots: 2,
                // Hoppers feed the input from above and the sides, and empty the output below.
                faces: vec![vec![1], vec![0], vec![0], vec![0], vec![0], vec![0]],
                inputs: vec![0],
                outputs: vec![1],
                menu: true,
            });
            k.energy = Some(energy(def.usage * STORED_TICKS, def.usage * STORED_TICKS, FlowSide::Input));
            r = r.block_entity_kind(k);
        }
        let mut basic = kind("mekanism:basic_energy_cube", false);
        basic.energy = Some(energy(BASIC_CUBE, BASIC_CUBE_RATE, FlowSide::Both));
        let mut creative = kind("mekanism:creative_energy_cube", true);
        creative.energy = Some(energy(CREATIVE_CUBE, 1_000_000_000, FlowSide::Output));
        let mut cable = kind("mekanism:basic_universal_cable", false);
        cable.keep_data = false;
        cable.transmitter = Some(TransmitterSpec {
            medium: TransmitterMedium::Energy,
            throughput: CABLE_RATE,
        });
        r = r
            .block_entity_kind(basic)
            .block_entity_kind(creative)
            .block_entity_kind(cable);
        Ok((Mekanism { books: HashMap::new() }, r.on_chunks()))
    }

    fn chunk_loaded(&mut self, dimension: &str, chunk_x: i32, chunk_z: i32, new: bool) {
        // Ores in the overworld's new chunks only.
        if !new || dimension != "minecraft:overworld" {
            return;
        }
        let mut changes: Vec<BlockChange> = Vec::new();
        for (k, kind) in worldgen::KINDS.iter().enumerate() {
            for n in 0..kind.per_chunk {
                for p in worldgen::vein(chunk_x, chunk_z, k, n) {
                    let Some(block) = world::get_block(dimension, p[0], p[1], p[2]) else { continue };
                    let name = block.split('[').next().unwrap_or("");
                    if let Some(state) = worldgen::replace(kind.ore, name) {
                        changes.push(BlockChange { x: p[0], y: p[1], z: p[2], state });
                    }
                }
            }
        }
        for batch in changes.chunks(4096) {
            let _ = world::set_blocks(dimension, batch);
        }
    }

    fn block_entities_tick(&mut self, _tick: u64, entities: Vec<BlockEntity>) -> Vec<BlockEntityUpdate> {
        let mut out = Vec::new();
        for e in &entities {
            let block = e.state.split('[').next().unwrap_or("");
            if let Some(def) = MACHINES.iter().find(|d| d.block == block) {
                out.extend(self.run(def, e));
            } else if block == "mekanism:creative_energy_cube" && e.energy < CREATIVE_CUBE {
                // Creative: never empty.
                out.push(BlockEntityUpdate {
                    dimension: e.dimension.clone(),
                    pos: e.pos,
                    data: e.data.clone(),
                    items: None,
                    energy: Some(CREATIVE_CUBE),
                    energy_faces: None,
                    tanks: None,
                    tank_faces: None,
                    menu_data: None,
                    transmitter_faces: None,
                });
            }
        }
        out
    }
}

horizon_mod_sdk::export_server_mod!(Mekanism);

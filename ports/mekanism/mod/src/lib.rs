//! Mekanism for Horizon (a Rust port of Mekanism, MIT): the machines' logic.
//! The blocks, items, textures, recipes and screens are the mod's content
//! (`data/` and `assets/`, made by `tools/convert.py` from Mekanism's own).
//!
//! What runs so far: the crusher, the enrichment chamber and the energized
//! smelter (`machine.rs`), the energy cubes and the universal cable.
mod machine;
mod sides;
mod worldgen;

use horizon_mod_sdk::server::{
    BlockEntity, BlockEntityKind, BlockEntityUpdate, ContainerSpec, EnergySpec, FlowSide, ItemStack,
    BlockChange, BlockPos, Player, Registration, ServerMod, TransmitterMedium, TransmitterSpec, player_control, players,
    recipe_types, serde_json, world,
};
use sides::{Mode, SIDES};
use machine::{Book, Io, Recipe, Stack, Status, step};
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
    /// The tick each book was last asked for: the game lends its recipes a little
    /// after the mod starts, so a book that came out empty is asked again.
    tried: HashMap<&'static str, u64>,
    /// Clicks of the configurator waiting for their cube's next tick (a use can't change a block entity).
    pending: Vec<Click>,
}

/// A configurator click on a block: the cube there is changed in its next tick.
struct Click {
    player: String,
    dimension: String,
    pos: BlockPos,
    /// The relative-to-world face clicked: down, up, north, south, west, east.
    face: usize,
    back: bool,
    tick: u64,
}

const CONFIGURATOR: &str = "mekanism:configurator";
/// The eye above the feet, standing and sneaking.
const EYE: f64 = 1.62;
const EYE_SNEAKING: f64 = 1.27;

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
fn menu_data(progress: u32, energy: u64, capacity: u64, status: Status) -> Vec<i32> {
    let half = |v: u64, shift: u32| ((v >> shift) & 0xFFFF) as i32;
    vec![
        progress as i32,
        half(energy, 0),
        half(energy, 16),
        half(capacity, 0),
        half(capacity, 16),
        status as i32,
    ]
}

impl Mekanism {
    /// A machine's tick: what it holds after, if that differs from before.
    fn run(&mut self, def: &Def, e: &BlockEntity, tick: u64) -> Option<BlockEntityUpdate> {
        let empty = self.books.get(def.block).is_none_or(Book::is_empty);
        let due = self.tried.get(def.block).is_none_or(|t| tick.saturating_sub(*t) >= 20);
        if empty && due {
            self.tried.insert(def.block, tick);
            self.books.insert(def.block, load_book(def.source));
        }
        let book = self.books.entry(def.block).or_default();
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
        let (next, status) = step(def.usage, book, &io);
        let active = status == Status::Running;
        // The block shows when it runs, with its entity kept.
        let lit = e.state.contains("lit=true");
        if active != lit {
            let _ = world::set_block_property(&e.dimension, e.pos.x, e.pos.y, e.pos.z, "lit", if active { "true" } else { "false" });
        }
        let capacity = def.usage * STORED_TICKS;
        let data = menu_data(next.progress, next.energy, capacity, status);
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

impl Mekanism {
    /// An energy cube's tick: a configurator click changes the mode of the side it met, the block shows the
    /// modes (the bytes of its data, which the clients draw its ports from), and the game's energy flow follows them.
    fn cube(&mut self, e: &BlockEntity, creative: bool) -> Option<BlockEntityUpdate> {
        let stored = sides::decode(&e.data);
        let mut changed = stored.is_none();
        // A new cube's front looks at the nearest player, as one placed in front of them does.
        let mut modes = stored.unwrap_or_else(|| sides::default_modes(front_for(e)));
        if let Some(i) = self
            .pending
            .iter()
            .position(|c| c.dimension == e.dimension && (c.pos.x, c.pos.y, c.pos.z) == (e.pos.x, e.pos.y, e.pos.z))
        {
            let click = self.pending.remove(i);
            modes[click.face] = modes[click.face].step(click.back);
            changed = true;
            let _ = players::tell(
                &click.player,
                &format!("Energy: {} side is now {}", SIDES[click.face], modes[click.face].name()),
            );
        }
        let flow = sides::flow_faces(&modes);
        let faces: Vec<FlowSide> = flow.iter().map(|f| flow_side(*f)).collect();
        let refill = creative && e.energy < CREATIVE_CUBE;
        if !changed && !refill && e.energy_faces == faces {
            return None;
        }
        Some(BlockEntityUpdate {
            dimension: e.dimension.clone(),
            pos: e.pos,
            data: sides::encode(&modes),
            items: None,
            energy: refill.then_some(CREATIVE_CUBE),
            energy_faces: Some(faces),
            tanks: None,
            tank_faces: None,
            menu_data: None,
            transmitter_faces: None,
        })
    }
}

/// The face a new cube at `e` takes as its front: it looks at the nearest player (within 12 blocks, in its
/// dimension) as it does when placed in front of one; north when none is near.
fn front_for(e: &BlockEntity) -> usize {
    let at = (f64::from(e.pos.x) + 0.5, f64::from(e.pos.z) + 0.5);
    players::online()
        .into_iter()
        .filter(|p| p.dimension == e.dimension)
        .map(|p| ((p.x - at.0).powi(2) + (p.z - at.1).powi(2), p))
        .filter(|(d, _)| *d < 144.0)
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .and_then(|(_, p)| player_control::get_look(&p.id))
        .map_or(sides::NORTH, |(yaw, _)| sides::front_toward(yaw))
}

fn flow_side(f: u8) -> FlowSide {
    match Mode::from_byte(f) {
        Mode::None => FlowSide::None,
        Mode::Input => FlowSide::Input,
        Mode::Output => FlowSide::Output,
        Mode::Both => FlowSide::Both,
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
        let mut basic = kind("mekanism:basic_energy_cube", true);
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
        Ok((Mekanism { books: HashMap::new(), tried: HashMap::new(), pending: Vec::new() }, r.on_chunks().on_items()))
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

    fn item_used(&mut self, player: &Player, stack: ItemStack, pos: Option<BlockPos>) -> bool {
        if stack.item != CONFIGURATOR {
            return false;
        }
        let say = |text: &str| {
            let _ = players::tell(&player.id, &format!("Configurator: {text}"));
        };
        let Some(pos) = pos else {
            say("heard, but not aimed at a block");
            return false;
        };
        let Some(block) = world::get_block(&player.dimension, pos.x, pos.y, pos.z) else {
            say(&format!("heard at {} {} {}, but the block there is unknown", pos.x, pos.y, pos.z));
            return false;
        };
        if !block.contains("_energy_cube") {
            say(&format!("heard on {block}, which it does not configure yet"));
            return false;
        }
        let sneaking = player_control::get_motion(&player.id).is_some_and(|m| m.sneaking);
        let eye = [player.x, player.y + if sneaking { EYE_SNEAKING } else { EYE }, player.z];
        let block = [pos.x, pos.y, pos.z];
        // The side the look meets; when the look is not known, or misses, the one toward the player.
        let look = player_control::get_look(&player.id);
        let face = look
            .and_then(|(yaw, pitch)| sides::face_hit(eye, sides::look(yaw, pitch), block))
            .unwrap_or_else(|| sides::face_toward(eye, block));
        say(&format!(
            "{} side{}",
            SIDES[face],
            if look.is_some() { "" } else { " (look unknown)" }
        ));
        self.pending.push(Click {
            player: player.id.clone(),
            dimension: player.dimension.clone(),
            pos,
            face,
            back: sneaking,
            tick: 0,
        });
        true
    }

    fn block_entities_tick(&mut self, tick: u64, entities: Vec<BlockEntity>) -> Vec<BlockEntityUpdate> {
        let mut out = Vec::new();
        for e in &entities {
            let block = e.state.split('[').next().unwrap_or("");
            if let Some(def) = MACHINES.iter().find(|d| d.block == block) {
                out.extend(self.run(def, e, tick));
            } else if block.ends_with("_energy_cube") {
                out.extend(self.cube(e, block == "mekanism:creative_energy_cube"));
            }
        }
        // A click whose cube did not tick (it was not there) is dropped after a few ticks.
        for c in &mut self.pending {
            c.tick += 1;
        }
        for c in self.pending.iter().filter(|c| c.tick >= 5) {
            let _ = players::tell(&c.player, "Configurator: no energy cube answered there");
        }
        self.pending.retain(|c| c.tick < 5);
        out
    }
}

horizon_mod_sdk::export_server_mod!(Mekanism);

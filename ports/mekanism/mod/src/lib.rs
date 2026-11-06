//! Mekanism for Horizon (a Rust port of Mekanism, MIT): the machines' logic.
//! The blocks, items, textures, recipes and screens are the mod's content
//! (`data/` and `assets/`, made by `tools/convert.py` from Mekanism's own).
//!
//! What runs so far: the crusher, the enrichment chamber and the energized
//! smelter (`machine.rs`), the energy cubes and the universal cable.
mod combiner;
mod chem;
mod config;
mod cube;
mod infuser;
mod machine;
mod sawmill;
mod sides;
mod worldgen;

use horizon_mod_sdk::server::{
    BlockEntity, BlockEntityKind, BlockEntityUpdate, ContainerSpec, EnergySpec, FlowSide, ItemStack, Tank, TankSpec,
    BlockChange, BlockPos, Player, Registration, ServerMod, TransmitterMedium, TransmitterSpec, player_control, players,
    recipe_types, serde_json, world,
};
use config::{Config, Data, Layout, Transmission};
use sides::Mode;
use machine::{Book, Io, Recipe, Stack, Status, step};
use std::collections::{HashMap, HashSet};

/// Where a machine's recipes come from.
#[derive(Clone, Copy)]
enum Source {
    /// Recipe files of a type of Mekanism's own (`mekanism:crushing`).
    Files(&'static str),
    /// The game's cooking recipes (`smelting`), vanilla's and the mods'.
    Cooking(&'static str),
}

/// What a machine does: one input and one output (the electric machines), a main and a secondary output
/// (the precision sawmill) or two inputs (the combiner).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Electric,
    Sawmill,
    Combiner,
    /// An item and an infusion type from its tank make another item (the metallurgic infuser).
    Infuser,
}

/// An electric machine: its block, where its recipes come from and its energy
/// use a tick (`MekanismConfig.usage`).
struct Def {
    block: &'static str,
    source: Source,
    usage: u64,
    kind: Kind,
}

impl Def {
    /// How its slots are laid out, which says what the data types of its sides reach.
    fn layout(&self) -> Layout {
        match self.kind {
            Kind::Electric => Layout::Electric,
            Kind::Sawmill => Layout::Sawmill,
            Kind::Combiner => Layout::Combiner,
            Kind::Infuser => Layout::Infuser,
        }
    }
}

const MACHINES: [Def; 6] = [
    Def { block: "mekanism:crusher", source: Source::Files("mekanism:crushing"), usage: 8, kind: Kind::Electric },
    Def { block: "mekanism:enrichment_chamber", source: Source::Files("mekanism:enriching"), usage: 5, kind: Kind::Electric },
    Def { block: "mekanism:energized_smelter", source: Source::Cooking("smelting"), usage: 5, kind: Kind::Electric },
    Def { block: "mekanism:precision_sawmill", source: Source::Files("mekanism:sawing"), usage: 5, kind: Kind::Sawmill },
    Def { block: "mekanism:combiner", source: Source::Files("mekanism:combining"), usage: 5, kind: Kind::Combiner },
    Def { block: "mekanism:metallurgic_infuser", source: Source::Files("mekanism:metallurgic_infusing"), usage: 5, kind: Kind::Infuser },
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
    saws: sawmill::Book,
    combs: combiner::Book,
    infs: infuser::Book,
    /// Clicks of the configurator waiting for their machine's next tick (a use can't change a block entity).
    pending: Vec<Click>,
    /// What each player's configurator is set to configure (Mekanism keeps it in the item).
    tool: HashMap<String, Transmission>,
    /// The machines whose item faces and eject were given to the game since the mod started: a machine loaded from the
    /// world or from an item gives them again once.
    applied: HashSet<(String, i32, i32, i32)>,
    /// The dice's state (see `Mekanism::roll`).
    rng: u64,
    /// The server tick the mod last saw, and when each player's configurator last took a click on a block: the client
    /// sends a use in the air after the use on a block for the same click, which must not change the tool's mode.
    now: u64,
    last_block_use: HashMap<String, u64>,
    /// Each cube's energy last tick and the rate it takes in, for its screen.
    flow: HashMap<(String, i32, i32, i32), (u64, u64)>,
    /// The window's buttons pressed on a cube, until its next tick.
    buttons: HashMap<(String, i32, i32, i32), Vec<i32>>,
}

/// A configurator click on a block: the cube there is changed in its next tick.
struct Click {
    player: String,
    dimension: String,
    pos: BlockPos,
    /// The relative-to-world face clicked: down, up, north, south, west, east.
    face: usize,
    /// Sneaking: the click changes the side; otherwise it only tells what it is.
    sneak: bool,
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

/// The count of an item entry (`{"count": 2, "id": ...}`), 1 when it has none.
fn count_of(v: &serde_json::Value) -> u32 {
    v["count"].as_u64().unwrap_or(1) as u32
}

fn load_saw_book() -> sawmill::Book {
    let mut book = sawmill::Book::default();
    for r in recipe_types::recipes_of_type("mekanism:sawing") {
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&r.json) else { continue };
        let Some(main) = json["main_output"]["id"].as_str() else { continue };
        let inputs = recipe_types::ingredient_items(&json["input"]["ingredient"].to_string());
        let secondary = json["secondary_output"]["id"].as_str().map(|id| {
            (id.to_owned(), count_of(&json["secondary_output"]), json["secondary_chance"].as_f64().unwrap_or(1.0))
        });
        book.add(
            inputs,
            sawmill::Recipe { count: count_of(&json["input"]), main: (main.to_owned(), count_of(&json["main_output"])), secondary },
        );
    }
    book
}

fn load_comb_book() -> combiner::Book {
    let mut book = combiner::Book::default();
    for r in recipe_types::recipes_of_type("mekanism:combining") {
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&r.json) else { continue };
        let Some(output) = json["output"]["id"].as_str() else { continue };
        let mains = recipe_types::ingredient_items(&json["main_input"]["ingredient"].to_string());
        let extras = recipe_types::ingredient_items(&json["extra_input"]["ingredient"].to_string());
        book.add(
            &mains,
            &extras,
            combiner::Recipe {
                main_count: count_of(&json["main_input"]),
                extra_count: count_of(&json["extra_input"]),
                output: output.to_owned(),
                out_count: count_of(&json["output"]),
            },
        );
    }
    book
}

/// The infusion types an ingredient names (`"#mekanism:redstone"`, a tag, or `"mekanism:redstone"`, a chemical; a list of
/// them). Every infusion type tag of the mod holds the one infusion type of its name, so a tag stands for it.
fn chemicals_of(ingredient: &serde_json::Value) -> Vec<String> {
    match ingredient {
        serde_json::Value::String(s) => vec![s.trim_start_matches('#').to_owned()],
        serde_json::Value::Array(list) => list.iter().flat_map(chemicals_of).collect(),
        _ => Vec::new(),
    }
}

fn load_inf_book() -> infuser::Book {
    let mut book = infuser::Book::default();
    for r in recipe_types::recipes_of_type("mekanism:metallurgic_infusing") {
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&r.json) else { continue };
        let Some(output) = json["output"]["id"].as_str() else { continue };
        if json["per_tick_usage"].as_bool().unwrap_or(false) {
            // Used a little each tick while it runs; no recipe of the mod does, so it is not made.
            continue;
        }
        let items = recipe_types::ingredient_items(&json["item_input"]["ingredient"].to_string());
        let chemicals = chemicals_of(&json["chemical_input"]["ingredient"]);
        book.add(
            &items,
            &chemicals,
            infuser::Recipe {
                item_count: count_of(&json["item_input"]),
                amount: json["chemical_input"]["amount"].as_u64().unwrap_or(1),
                output: output.to_owned(),
                out_count: count_of(&json["output"]),
            },
        );
    }
    // The infusion slot's conversions: an item to an infusion type (the other chemicals they make are not the tank's).
    for r in recipe_types::recipes_of_type("mekanism:chemical_conversion") {
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&r.json) else { continue };
        let (Some(chemical), Some(amount)) = (json["output"]["id"].as_str(), json["output"]["amount"].as_u64()) else { continue };
        let items = recipe_types::ingredient_items(&json["input"]["ingredient"].to_string());
        book.add_conversion(&items, chemical, amount);
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

/// The screen's data slots: progress, the energy and its capacity as low and high 16 bits, the status, then the four words
/// of the side configuration (`Config::screen_words`).
fn menu_data(progress: u32, energy: u64, capacity: u64, status: Status, cfg: &Config) -> Vec<i32> {
    let half = |v: u64, shift: u32| ((v >> shift) & 0xFFFF) as i32;
    let mut data = vec![
        progress as i32,
        half(energy, 0),
        half(energy, 16),
        half(capacity, 0),
        half(capacity, 16),
        status as i32,
    ];
    data.extend(cfg.screen_words());
    data
}

impl Mekanism {
    /// A roll of dice, 0 up to 1, for the sawmill's secondary output (Java rolls `level.random.nextDouble()`): a generator
    /// that keeps its state between rolls, started from where the players stand and the tick, and mixed again with the
    /// tick and the place each time, so two machines and two ticks never repeat each other.
    fn roll(&mut self, tick: u64, pos: &BlockPos) -> f64 {
        if self.rng == 0 {
            let mut seed = tick.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0x2545_F491_4F6C_DD1D;
            for p in players::online() {
                seed = seed.rotate_left(13) ^ p.x.to_bits() ^ p.y.to_bits().rotate_left(21) ^ p.z.to_bits().rotate_left(42);
            }
            self.rng = seed | 1;
        }
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng = x;
        let mix = (pos.x as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9)
            ^ (pos.y as u64).wrapping_mul(0x94D0_49BB_1331_11EB)
            ^ (pos.z as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93)
            ^ tick.rotate_left(29);
        let mut h = x.wrapping_mul(0x2545_F491_4F6C_DD1D) ^ mix;
        h ^= h >> 31;
        h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
        h ^= h >> 29;
        (h >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Loads the recipes of a machine when it has none (the game lends them a little after the mod starts, so a
    /// book that came out empty is asked again every second).
    fn refresh(&mut self, def: &Def, tick: u64) {
        let empty = match def.kind {
            Kind::Electric => self.books.get(def.block).is_none_or(Book::is_empty),
            Kind::Sawmill => self.saws.is_empty(),
            Kind::Combiner => self.combs.is_empty(),
            Kind::Infuser => self.infs.is_empty(),
        };
        let due = self.tried.get(def.block).is_none_or(|t| tick.saturating_sub(*t) >= 20);
        if empty && due {
            self.tried.insert(def.block, tick);
            match def.kind {
                Kind::Electric => {
                    self.books.insert(def.block, load_book(def.source));
                }
                Kind::Sawmill => self.saws = load_saw_book(),
                Kind::Combiner => self.combs = load_comb_book(),
                Kind::Infuser => self.infs = load_inf_book(),
            }
        }
    }

    /// What a tool click on a machine does: it tells the side's behaviour, and while sneaking bumps it to the next
    /// (`ItemConfigurator.useOn`). Whether the configuration changed.
    fn configure(&mut self, cfg: &mut Config, facing: &str, layout: Layout, click: &Click) -> bool {
        let t = self.tool.get(&click.player).copied().unwrap_or(Transmission::Items);
        // A configurator set to a transmission the machine does not have has nothing to do there.
        if !layout.transmissions().contains(&t) {
            return false;
        }
        let side = config::side_of(facing, click.face);
        let now = cfg.side(t)[side];
        if !click.sneak {
            let _ = players::tell(
                &click.player,
                &format!("Current {} behavior: {} ({})", t.label(), now.label(), now.color_name()),
            );
            return false;
        }
        let next = config::cycle(layout, t, now, false);
        cfg.side_mut(t)[side] = next;
        let _ = players::tell(
            &click.player,
            &format!("{} behavior bumped to: {} ({})", t.label(), next.label(), next.color_name()),
        );
        next != now
    }

    /// Gives the game what a machine's configuration says: the slots each face reaches, whether it ejects, and (as the
    /// return) where energy comes in.
    fn apply_config(&mut self, e: &BlockEntity, layout: Layout, facing: &str, cfg: &Config) -> Vec<FlowSide> {
        let faces = config::item_faces(layout, facing, cfg.items());
        let _ = world::set_item_faces(&e.dimension, e.pos.x, e.pos.y, e.pos.z, &faces);
        let _ = world::set_item_eject(&e.dimension, e.pos.x, e.pos.y, e.pos.z, cfg.ejects(Transmission::Items));
        self.applied.insert((e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z));
        config::energy_in(facing, cfg.energy())
            .iter()
            .map(|on| if *on { FlowSide::Input } else { FlowSide::None })
            .collect()
    }

    /// What each face does with an infuser's tank, as its chemical sides say.
    fn tank_sides(&self, facing: &str, cfg: &Config) -> Vec<Vec<FlowSide>> {
        let flow = config::tank_flow(facing, cfg.side(Transmission::Chemicals));
        vec![flow.iter().map(|f| flow_side(*f)).collect()]
    }

    /// The tool click waiting for the entity at this place, if any.
    fn take_click(&mut self, e: &BlockEntity) -> Option<Click> {
        let i = self
            .pending
            .iter()
            .position(|c| c.dimension == e.dimension && (c.pos.x, c.pos.y, c.pos.z) == (e.pos.x, e.pos.y, e.pos.z))?;
        Some(self.pending.remove(i))
    }

    /// A machine's tick: what it holds after, if that differs from before.
    fn run(&mut self, def: &Def, e: &BlockEntity, tick: u64) -> Option<BlockEntityUpdate> {
        self.refresh(def, tick);
        let layout = def.layout();
        let facing = sides::property(&e.state, "facing").unwrap_or("north").to_owned();
        let progress = e
            .data
            .get(..2)
            .map_or(0, |b| u32::from(u16::from_le_bytes([b[0], b[1]])));
        // Its side configuration: what its data holds after the progress, or a new machine's.
        let stored = Config::decode(e.data.get(2..).unwrap_or(&[]), layout);
        let mut cfg = stored.unwrap_or_else(|| config::default_config(layout));
        let key = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
        let mut dirty = stored.is_none() || !self.applied.contains(&key);
        if let Some(click) = self.take_click(e) {
            dirty |= self.configure(&mut cfg, &facing, layout, &click);
        }
        let dice = if def.kind == Kind::Sawmill { self.roll(tick, &e.pos) } else { 0.0 };
        let slot = |i: usize| stack(e.items.get(i).unwrap_or(&None));
        // The infuser's tank before and after.
        let mut tank_now = infuser::Tank::default();
        let mut tank_next = infuser::Tank::default();
        // The slots after, the energy and progress after, what it did, and whether anything changed.
        let (items, energy, next_progress, status, changed): (Vec<Option<ItemStack>>, u64, u32, Status, bool) = match def.kind {
            Kind::Electric => {
                let book = self.books.entry(def.block).or_default();
                let io = Io { input: slot(0), output: slot(1), energy: e.energy, progress };
                let (next, status) = step(def.usage, book, &io);
                (vec![item(&next.input), item(&next.output)], next.energy, next.progress, status, next != io)
            }
            Kind::Sawmill => {
                let io = sawmill::Io { input: slot(0), main: slot(1), secondary: slot(2), energy: e.energy, progress };
                let (next, status) = sawmill::step(def.usage, &self.saws, &io, dice);
                (
                    vec![item(&next.input), item(&next.main), item(&next.secondary)],
                    next.energy,
                    next.progress,
                    status,
                    next != io,
                )
            }
            Kind::Infuser => {
                tank_now = e.tanks.first().map_or_else(infuser::Tank::default, |t| infuser::Tank {
                    chemical: t.substance.clone(),
                    amount: t.amount,
                });
                let io = infuser::Io {
                    input: slot(0),
                    output: slot(1),
                    infusion: slot(2),
                    tank: tank_now.clone(),
                    energy: e.energy,
                    progress,
                };
                let (next, status) = infuser::step(def.usage, &self.infs, &io);
                tank_next = next.tank.clone();
                (
                    vec![item(&next.input), item(&next.output), item(&next.infusion)],
                    next.energy,
                    next.progress,
                    status,
                    next != io,
                )
            }
            Kind::Combiner => {
                let io = combiner::Io { main: slot(0), extra: slot(1), output: slot(2), energy: e.energy, progress };
                let (next, status) = combiner::step(def.usage, &self.combs, &io);
                (
                    vec![item(&next.main), item(&next.extra), item(&next.output)],
                    next.energy,
                    next.progress,
                    status,
                    next != io,
                )
            }
        };
        let active = status == Status::Running;
        // The block shows when it runs, with its entity kept.
        let lit = e.state.contains("lit=true");
        if active != lit {
            let _ = world::set_block_property(&e.dimension, e.pos.x, e.pos.y, e.pos.z, "lit", if active { "true" } else { "false" });
        }
        let energy_faces = dirty.then(|| self.apply_config(e, layout, &facing, &cfg));
        let capacity = def.usage * STORED_TICKS;
        let mut data = menu_data(next_progress, energy, capacity, status, &cfg);
        let has_tank = def.kind == Kind::Infuser;
        if has_tank {
            data.extend(chem::screen(&tank_next));
        }
        if !changed && !dirty && data == e.menu_data {
            return None;
        }
        let tanks = (has_tank && (tank_next != tank_now || e.tanks.is_empty())).then(|| {
            vec![Tank { substance: tank_next.chemical.clone(), amount: tank_next.amount }]
        });
        let tank_faces = (has_tank && dirty).then(|| self.tank_sides(&facing, &cfg));
        Some(BlockEntityUpdate {
            dimension: e.dimension.clone(),
            pos: e.pos,
            data: machine_data(next_progress, &cfg),
            items: Some(items),
            energy: Some(energy),
            energy_faces,
            tanks,
            tank_faces,
            menu_data: Some(data),
            transmitter_faces: None,
        })
    }
}

/// What a machine keeps in its data: its progress (two bytes), then its side configuration.
fn machine_data(progress: u32, cfg: &Config) -> Vec<u8> {
    let mut data = (progress as u16).to_le_bytes().to_vec();
    data.extend(cfg.encode());
    data
}

impl Mekanism {
    /// An energy cube's tick: a configurator click changes the mode of the side it met, the block shows the
    /// modes (the bytes of its data, which the clients draw its ports from), and the game's energy flow follows them.
    fn cube(&mut self, e: &BlockEntity, creative: bool) -> Option<BlockEntityUpdate> {
        let stored = sides::decode(&e.data);
        let mut changed = stored.is_none() || e.data.len() < 7;
        // A new cube's front looks at the nearest player, as one placed in front of them does.
        let front_new = front_for(e);
        let mut modes = stored.unwrap_or_else(|| sides::default_modes(front_new));
        let front = if stored.is_some() { cube::front_of(&e.data, &modes) } else { front_new };
        if let Some(click) = self.take_click(e) {
            // The cube configures energy only: a configurator set to items has nothing to do here.
            if self.tool.get(&click.player).copied().unwrap_or(Transmission::Items) == Transmission::Energy {
                let now = modes[click.face];
                let d = cube::data_of;
                if click.sneak {
                    modes[click.face] = now.step(false);
                    changed = true;
                    let next = d(modes[click.face]);
                    let _ = players::tell(
                        &click.player,
                        &format!("Energy behavior bumped to: {} ({})", next.label(), next.color_name()),
                    );
                } else {
                    let _ = players::tell(
                        &click.player,
                        &format!("Current Energy behavior: {} ({})", d(now).label(), d(now).color_name()),
                    );
                }
            }
        }
        changed |= self.take_button(e, &mut modes, front);
        let capacity = if creative { CREATIVE_CUBE } else { BASIC_CUBE };
        let mut energy = e.energy;
        // The discharge slot turns what it holds into energy when it fits (`fillContainerOrConvert`).
        let mut items: Option<Vec<Option<ItemStack>>> = None;
        if let Some(Some(slot)) = e.items.get(cube::DISCHARGE)
            && let Some(gain) = cube::conversion(&slot.item)
            && energy + gain <= capacity
            && slot.count > 0
        {
            energy += gain;
            let mut next = e.items.clone();
            next[cube::DISCHARGE] = (slot.count > 1).then(|| ItemStack { count: slot.count - 1, ..slot.clone() });
            items = Some(next);
        }
        let flow = sides::flow_faces(&modes);
        let faces: Vec<FlowSide> = flow.iter().map(|f| flow_side(*f)).collect();
        let refill = creative && energy < CREATIVE_CUBE;
        if refill {
            energy = CREATIVE_CUBE;
        }
        // The rate it takes energy in, smoothed over a few ticks, for the energy tab.
        let key = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
        let (prev, rate) = self.flow.get(&key).copied().unwrap_or((energy, 0));
        let gain = e.energy.saturating_sub(prev);
        let rate = (rate * 3 + gain) / 4;
        self.flow.insert(key, (e.energy, rate));
        let data = cube::screen(rate, energy, capacity, &modes, front);
        if !changed && items.is_none() && !refill && e.energy_faces == faces && data == e.menu_data {
            return None;
        }
        Some(BlockEntityUpdate {
            dimension: e.dimension.clone(),
            pos: e.pos,
            data: cube::encode(&modes, front),
            items,
            energy: (energy != e.energy).then_some(energy),
            energy_faces: Some(faces),
            tanks: None,
            tank_faces: None,
            menu_data: Some(data),
            transmitter_faces: None,
        })
    }

    /// The window's buttons pressed on cube `e` since its last tick, applied to its `modes`; true when they changed.
    fn take_button(&mut self, e: &BlockEntity, modes: &mut [Mode; 6], front: usize) -> bool {
        let key = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
        let Some(ids) = self.buttons.remove(&key) else { return false };
        let mut changed = false;
        for id in ids {
            if let Some(click) = config::click_of(id) {
                changed |= cube::press(modes, front, click);
            }
        }
        changed
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
            k.container = Some(match def.kind {
                // The faces are the default configuration's, the machine's own once it has one.
                Kind::Electric => ContainerSpec {
                    slots: 2,
                    faces: config::item_faces(Layout::Electric, "north", config::default_config(Layout::Electric).items()),
                    inputs: vec![0],
                    outputs: vec![1],
                    menu: true,
                },
                // The input, the main and the secondary output; the faces are the default configuration's, the
                // machine's own once it has one.
                Kind::Sawmill => ContainerSpec {
                    slots: 3,
                    faces: config::item_faces(Layout::Sawmill, "north", config::default_config(Layout::Sawmill).items()),
                    inputs: vec![0],
                    outputs: vec![1, 2],
                    menu: true,
                },
                // The input, the output and the infusion slot; hoppers put into the input and the infusion slot.
                Kind::Infuser => ContainerSpec {
                    slots: 3,
                    faces: config::item_faces(Layout::Infuser, "north", config::default_config(Layout::Infuser).items()),
                    inputs: vec![0, 2],
                    outputs: vec![1],
                    menu: true,
                },
                // The main input, the extra one and the output, on the faces of the default configuration.
                Kind::Combiner => ContainerSpec {
                    slots: 3,
                    faces: config::item_faces(Layout::Combiner, "north", config::default_config(Layout::Combiner).items()),
                    inputs: vec![0, 1],
                    outputs: vec![2],
                    menu: true,
                },
            });
            k.energy = Some(energy(def.usage * STORED_TICKS, def.usage * STORED_TICKS, FlowSide::Input));
            if def.kind == Kind::Infuser {
                // The infusion tank takes in on every side to start with (`LightConfigInfo.INPUT_ONLY`).
                k.tanks = vec![TankSpec {
                    kind: "infusion".into(),
                    capacity: infuser::MAX_INFUSE,
                    max_rate: infuser::MAX_INFUSE,
                    faces: vec![FlowSide::Input; 6],
                }];
            }
            r = r.block_entity_kind(k);
        }
        // The discharge and charge slots of its screen; no face reaches them (the cube's item sides are still to do).
        let slots = || ContainerSpec { slots: 2, faces: vec![Vec::new(); 6], inputs: Vec::new(), outputs: Vec::new(), menu: true };
        let mut basic = kind("mekanism:basic_energy_cube", true);
        basic.container = Some(slots());
        basic.energy = Some(energy(BASIC_CUBE, BASIC_CUBE_RATE, FlowSide::Both));
        let mut creative = kind("mekanism:creative_energy_cube", true);
        creative.container = Some(slots());
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
        Ok((Mekanism {
            books: HashMap::new(),
            tried: HashMap::new(),
            saws: sawmill::Book::default(),
            combs: combiner::Book::default(),
            infs: infuser::Book::default(),
            pending: Vec::new(),
            tool: HashMap::new(),
            applied: HashSet::new(),
            rng: 0,
            now: 0,
            last_block_use: HashMap::new(),
            flow: HashMap::new(),
            buttons: HashMap::new(),
        }, r.on_chunks().on_items()))
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
        let sneaking = player_control::get_motion(&player.id).is_some_and(|m| m.sneaking);
        let Some(pos) = pos else {
            // Sneaking in the air changes what the tool configures (`ItemConfigurator.use`), unless this is the use in
            // the air that follows a click the tool just took on a machine: that click is one, and does nothing more.
            if sneaking
                && self
                    .last_block_use
                    .get(&player.id)
                    .is_some_and(|t| self.now.saturating_sub(*t) <= 2)
            {
                return true;
            }
            if sneaking {
                let t = self.tool.entry(player.id.clone()).or_insert(Transmission::Items);
                *t = t.next_tool();
                let _ = players::tell(&player.id, &format!("Configure State: Configurate ({})", t.label()));
                return true;
            }
            return false;
        };
        let Some(block) = world::get_block(&player.dimension, pos.x, pos.y, pos.z) else { return false };
        let name = block.split('[').next().unwrap_or("");
        if !name.ends_with("_energy_cube") && !MACHINES.iter().any(|d| d.block == name) {
            return false;
        }
        let eye = [player.x, player.y + if sneaking { EYE_SNEAKING } else { EYE }, player.z];
        let block = [pos.x, pos.y, pos.z];
        // The side the look meets; when the look is not known, or misses, the one toward the player.
        let face = player_control::get_look(&player.id)
            .and_then(|(yaw, pitch)| sides::face_hit(eye, sides::look(yaw, pitch), block))
            .unwrap_or_else(|| sides::face_toward(eye, block));
        self.pending.push(Click {
            player: player.id.clone(),
            dimension: player.dimension.clone(),
            pos,
            face,
            sneak: sneaking,
            tick: 0,
        });
        self.last_block_use.insert(player.id.clone(), self.now);
        true
    }

    /// A button of a machine's screen: the side configuration window's (a side, the eject toggle, the clear button).
    fn menu_button(&mut self, _player: &Player, _pos: BlockPos, id: i32, e: BlockEntity) -> Option<BlockEntityUpdate> {
        let block = e.state.split('[').next()?;
        if block.ends_with("_energy_cube") {
            // The cube applies it on its next tick.
            self.buttons.entry((e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z)).or_default().push(id);
            return None;
        }
        let def = MACHINES.iter().find(|d| d.block == block)?;
        let layout = def.layout();
        let facing = sides::property(&e.state, "facing").unwrap_or("north").to_owned();
        let mut cfg = Config::decode(e.data.get(2..).unwrap_or(&[]), layout).unwrap_or_else(|| config::default_config(layout));
        if id == chem::DUMP && def.kind == Kind::Infuser {
            // GuiDumpButton: the tank is emptied (`dump`).
            let mut data = e.menu_data.clone();
            data.truncate(16);
            data.extend(chem::screen(&infuser::Tank::default()));
            return Some(BlockEntityUpdate {
                dimension: e.dimension.clone(),
                pos: e.pos,
                data: e.data.clone(),
                items: None,
                energy: None,
                energy_faces: None,
                tanks: Some(vec![Tank { substance: String::new(), amount: 0 }]),
                tank_faces: None,
                menu_data: Some(data),
                transmitter_faces: None,
            });
        }
        match config::click_of(id)? {
            config::Click::Side { t, side, back } => {
                let now = cfg.side(t)[side];
                cfg.side_mut(t)[side] = config::cycle(layout, t, now, back);
            }
            config::Click::Eject(t) => cfg.eject[t as usize] = !cfg.eject[t as usize],
            config::Click::Clear { t, back } => {
                let target = config::clear_target(layout, t, cfg.side(t), back);
                *cfg.side_mut(t) = [target; 6];
            }
        }
        let progress = e.data.get(..2).map_or(0, |b| u32::from(u16::from_le_bytes([b[0], b[1]])));
        let faces = self.apply_config(&e, layout, &facing, &cfg);
        // The screen's first six numbers stay; the configuration's four follow them.
        let tail: Vec<i32> = e.menu_data.get(16..).map(<[i32]>::to_vec).unwrap_or_default();
        let mut data = e.menu_data.clone();
        data.resize(6, 0);
        data.extend(cfg.screen_words());
        data.extend(tail);
        let tank_faces = (def.kind == Kind::Infuser).then(|| self.tank_sides(&facing, &cfg));
        Some(BlockEntityUpdate {
            dimension: e.dimension.clone(),
            pos: e.pos,
            data: machine_data(progress, &cfg),
            items: None,
            energy: None,
            energy_faces: Some(faces),
            tanks: None,
            tank_faces,
            menu_data: Some(data),
            transmitter_faces: None,
        })
    }

    fn block_entities_tick(&mut self, tick: u64, entities: Vec<BlockEntity>) -> Vec<BlockEntityUpdate> {
        self.now = tick;
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
        self.pending.retain(|c| c.tick < 5);
        out
    }
}

horizon_mod_sdk::export_server_mod!(Mekanism);

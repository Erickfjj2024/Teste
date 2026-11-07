//! Mekanism for Horizon (a Rust port of Mekanism, MIT): the machines' logic.
//! The blocks, items, textures, recipes and screens are the mod's content
//! (`data/` and `assets/`, made by `tools/convert.py` from Mekanism's own).
//!
//! What runs so far: the crusher, the enrichment chamber and the energized
//! smelter (`machine.rs`), the energy cubes and the universal cable.
mod combiner;
mod chem;
mod continuous;
mod config;
mod cube;
mod chem_table;
mod plus;
mod process;
mod reaction;
mod redstone;
mod generators;
mod machine;
mod sawmill;
mod sides;
mod tank;
mod upgrades;
mod worldgen;

use horizon_mod_sdk::server::{
    BlockEntity, BlockEntityKind, BlockEntityUpdate, ContainerSpec, EnergySpec, FlowSide, ItemStack, Tank, TankSpec,
    BlockChange, BlockPos, Player, Registration, ServerMod, TransmitterMedium, TransmitterSpec, player_control, players,
    recipe_types, serde_json, world,
};
use config::{Config, Data, Layout, Transmission};
use generators::GenKind;
use redstone::Control;
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
    /// A machine of `process`: an item and a chemical, in and out (the infuser, the chambers, the oxidizer...).
    Process,
    /// A machine of `continuous`: chemicals and fluids in its tanks, a little every tick.
    Continuous,
    /// A storage tank (`tank`), of fluid or of chemical.
    Tank,
    /// A generator of MekanismGenerators (`generators`).
    Generator,
    /// The pressurized reaction chamber (`reaction`).
    Reaction,
}

/// A storage tank's tier: what it holds and moves a tick.
#[derive(Clone, Copy)]
struct TankDef {
    fluid: bool,
    capacity: u64,
    rate: u64,
    creative: bool,
}

/// An electric machine: its block, where its recipes come from and its energy
/// use a tick (`MekanismConfig.usage`).
struct Def {
    block: &'static str,
    source: Source,
    usage: u64,
    kind: Kind,
    /// How its slots are laid out, which says what the data types of its sides reach.
    layout: Layout,
    /// What a `Kind::Process` machine does.
    proc: Option<process::Spec>,
    /// What a `Kind::Continuous` machine does.
    cont: Option<continuous::Spec>,
    /// A `Kind::Tank`'s tier.
    tank: Option<TankDef>,
    /// A `Kind::Generator`'s kind.
    generator: Option<GenKind>,
}

const fn electric(block: &'static str, source: Source, usage: u64) -> Def {
    Def { block, source, usage, kind: Kind::Electric, layout: Layout::Electric, proc: None, cont: None, tank: None, generator: None }
}

/// A machine that works an item and a chemical (`process`): its recipes are of type `source`.
const fn chemical(block: &'static str, source: &'static str, layout: Layout, spec: process::Spec) -> Def {
    Def { block, source: Source::Files(source), usage: spec.usage, kind: Kind::Process, layout, proc: Some(spec), cont: None, tank: None, generator: None }
}

/// A machine of `continuous`.
const fn continuous_machine(block: &'static str, source: &'static str, layout: Layout, spec: continuous::Spec) -> Def {
    Def { block, source: Source::Files(source), usage: spec.usage, kind: Kind::Continuous, layout, proc: None, cont: Some(spec), tank: None, generator: None }
}

/// A generator of MekanismGenerators.
const fn generator(block: &'static str, kind: GenKind) -> Def {
    Def { block, source: Source::Files(""), usage: 0, kind: Kind::Generator, layout: Layout::Electric, proc: None, cont: None, tank: None, generator: Some(kind) }
}

/// The pressurized reaction chamber.
const fn reaction_machine(block: &'static str) -> Def {
    Def { block, source: Source::Files(""), usage: reaction::USAGE, kind: Kind::Reaction, layout: Layout::Reaction, proc: None, cont: None, tank: None, generator: None }
}

/// A storage tank block of a tier.
const fn tank_block(block: &'static str, fluid: bool, capacity: u64, rate: u64, creative: bool) -> Def {
    Def {
        block,
        source: Source::Files(""),
        usage: 0,
        kind: Kind::Tank,
        layout: if fluid { Layout::Electric } else { Layout::ChemTank },
        proc: None,
        cont: None,
        tank: Some(TankDef { fluid, capacity, rate, creative }),
        generator: None,
    }
}

use process::Shape;

const fn spec(shape: Shape, usage: u64, ticks: u32, cap_in: u64, cap_out: u64, random_use: bool) -> process::Spec {
    process::Spec { shape, usage, ticks, cap_in, cap_out, random_use, mean: 1.0 }
}

const MACHINES: [Def; 34] = [
    electric("mekanism:crusher", Source::Files("mekanism:crushing"), 8),
    electric("mekanism:enrichment_chamber", Source::Files("mekanism:enriching"), 5),
    electric("mekanism:energized_smelter", Source::Cooking("smelting"), 5),
    Def { block: "mekanism:precision_sawmill", source: Source::Files("mekanism:sawing"), usage: 5, kind: Kind::Sawmill, layout: Layout::Sawmill, proc: None, cont: None, tank: None, generator: None },
    Def { block: "mekanism:combiner", source: Source::Files("mekanism:combining"), usage: 5, kind: Kind::Combiner, layout: Layout::Combiner, proc: None, cont: None, tank: None, generator: None },
    chemical("mekanism:metallurgic_infuser", "mekanism:metallurgic_infusing", Layout::Infuser, spec(Shape::ItemChemToItem, 5, 200, 1000, 0, false)),
    // The purification and injection chambers use their chemical by chance (the config's randomized consumption).
    chemical("mekanism:purification_chamber", "mekanism:purifying", Layout::Adv, spec(Shape::ItemChemToItem, 10, 200, 210, 0, true)),
    chemical("mekanism:chemical_injection_chamber", "mekanism:injecting", Layout::Adv, spec(Shape::ItemChemToItem, 15, 200, 210, 0, true)),
    chemical("mekanism:osmium_compressor", "mekanism:compressing", Layout::Compressor, spec(Shape::ItemChemToItem, 15, 200, 210, 0, false)),
    chemical("mekanism:chemical_oxidizer", "mekanism:oxidizing", Layout::Oxidizer, spec(Shape::ItemToChem, 15, 100, 0, 10_000, false)),
    chemical("mekanism:chemical_dissolution_chamber", "mekanism:dissolution", Layout::Dissolution, spec(Shape::ItemChemToChem, 16, 100, 10_000, 10_000, true)),
    chemical("mekanism:chemical_crystallizer", "mekanism:crystallizing", Layout::Crystallizer, spec(Shape::ChemToItem, 15, 200, 10_000, 0, false)),
    // The chemical infuser, the washer and the electrolytic separator work their tanks every tick.
    continuous_machine("mekanism:chemical_infuser", "mekanism:chemical_infusing", Layout::ChemInfuser,
        continuous::Spec { shape: continuous::Shape::Infuser, usage: 15, caps: [10_000; 3] }),
    continuous_machine("mekanism:chemical_washer", "mekanism:washing", Layout::Washer,
        continuous::Spec { shape: continuous::Shape::Washer, usage: 5, caps: [10_000; 3] }),
    // Its energy is twice the hydrogen's energy density (2), so 4 a tick.
    continuous_machine("mekanism:electrolytic_separator", "mekanism:separating", Layout::Separator,
        continuous::Spec { shape: continuous::Shape::Separator, usage: 4, caps: [24_000, 2_400, 2_400] }),
    // The rotary condensentrator turns a gas into a fluid or a fluid into a gas, by its mode (10 buckets in each tank).
    continuous_machine("mekanism:rotary_condensentrator", "mekanism:rotary", Layout::Rotary,
        continuous::Spec { shape: continuous::Shape::Rotary, usage: 2, caps: [10_000, 10_000, 0] }),
    // Pigments: an item gives a pigment, a pigment paints an item, two pigments mix into a third.
    chemical("mekanism:pigment_extractor", "mekanism:pigment_extracting", Layout::Oxidizer, spec(Shape::ItemToChem, 5, 100, 0, 20_000, false)),
    chemical("mekanism:painting_machine", "mekanism:painting", Layout::Adv, spec(Shape::ItemChemToItem, 5, 200, 15_000, 0, false)),
    continuous_machine("mekanism:pigment_mixer", "mekanism:pigment_mixing", Layout::Mixer,
        continuous::Spec { shape: continuous::Shape::Infuser, usage: 5, caps: [1_000, 1_000, 2_000] }),
    // An item, a fluid and a gas make an item and a gas (14 recipes: gasification and substrate).
    reaction_machine("mekanism:pressurized_reaction_chamber"),
    // The generators of MekanismGenerators: they make energy from bioethanol, lava and heat, the sun and gases.
    generator("mekanism:bio_generator", GenKind::Bio),
    generator("mekanism:heat_generator", GenKind::Heat),
    generator("mekanism:solar_generator", GenKind::Solar),
    generator("mekanism:gas_burning_generator", GenKind::Gas),
    // The storage tanks: `FluidTankTier` and `ChemicalTankTier` (a bucket is 1000).
    tank_block("mekanism:basic_fluid_tank", true, 32_000, 1_000, false),
    tank_block("mekanism:advanced_fluid_tank", true, 64_000, 4_000, false),
    tank_block("mekanism:elite_fluid_tank", true, 128_000, 16_000, false),
    tank_block("mekanism:ultimate_fluid_tank", true, 256_000, 64_000, false),
    tank_block("mekanism:creative_fluid_tank", true, 1_000_000_000_000, 1_000_000_000, true),
    tank_block("mekanism:basic_chemical_tank", false, 64_000, 1_000, false),
    tank_block("mekanism:advanced_chemical_tank", false, 256_000, 16_000, false),
    tank_block("mekanism:elite_chemical_tank", false, 1_024_000, 128_000, false),
    tank_block("mekanism:ultimate_chemical_tank", false, 8_192_000, 512_000, false),
    tank_block("mekanism:creative_chemical_tank", false, 1_000_000_000_000, 1_000_000_000, true),
];

/// Energy a machine holds: ticks of its use (an assumption: 26.3's own table is
/// still to be written upstream).
const STORED_TICKS: u64 = 2500;
const CREATIVE_CUBE: u64 = 1_000_000_000_000;
/// `EnergyCubeTier`: what each cube holds and moves a tick (the creative one: everything, refilled).
const CUBE_TIERS: [(&str, u64, u64); 4] = [
    ("mekanism:basic_energy_cube", 4_000_000, 4_000),
    ("mekanism:advanced_energy_cube", 16_000_000, 16_000),
    ("mekanism:elite_energy_cube", 64_000_000, 64_000),
    ("mekanism:ultimate_energy_cube", 256_000_000, 256_000),
];

/// What a cube of this block holds, whether it is the creative one.
fn cube_capacity(block: &str) -> (u64, bool) {
    CUBE_TIERS.iter().find(|(b, _, _)| *b == block).map_or((CREATIVE_CUBE, true), |(_, c, _)| (*c, false))
}
/// What a cable, a tube and a pipe of each tier carry a tick.
const CABLE_RATES: [u64; 4] = [8_000, 128_000, 1_024_000, 8_192_000];
const TUBE_RATES: [u64; 4] = [750, 2_000, 64_000, 256_000];
const PIPE_RATES: [u64; 4] = [250, 1_000, 8_000, 32_000];

struct Mekanism {
    books: HashMap<&'static str, Book>,
    /// The tick each book was last asked for: the game lends its recipes a little
    /// after the mod starts, so a book that came out empty is asked again.
    tried: HashMap<&'static str, u64>,
    saws: sawmill::Book,
    combs: combiner::Book,
    /// The recipes of the machines of `process`, by block.
    procs: HashMap<&'static str, process::Book>,
    /// The recipes of the machines of `continuous`, by block.
    conts: HashMap<&'static str, continuous::Book>,
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
    /// The machines whose input tank was told what it takes in (once their recipes are known).
    filtered: HashSet<(String, i32, i32, i32)>,
    /// Each cube's energy last tick and the rate it takes in, for its screen.
    flow: HashMap<(String, i32, i32, i32), (u64, u64)>,
    /// The window's buttons pressed on a cube, until its next tick.
    buttons: HashMap<(String, i32, i32, i32), Vec<i32>>,
    /// Whether a solar generator saw the sky when last asked, and when.
    suns: HashMap<(String, i32, i32, i32), (u64, bool)>,
    /// The redstone signal each machine had the tick before (`redstoneLastTick`), and whether the one being run may work.
    last_power: HashMap<(String, i32, i32, i32), bool>,
    can: bool,
    /// How long the upgrade in each machine's slot has waited, the window's remove buttons since the last tick, the energy
    /// capacity each machine was given, and whether upgrades changed the machine being run (so it answers even if idle).
    install: HashMap<(String, i32, i32, i32), u32>,
    removals: HashMap<(String, i32, i32, i32), Vec<i32>>,
    capacity: HashMap<(String, i32, i32, i32), u64>,
    force: bool,
    /// The reaction chamber's recipes.
    reactions: reaction::Book,
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

/// How many tanks a machine of `process` has: the one its chemical goes into, and the one it comes out in.
fn tank_count(spec: Option<process::Spec>) -> usize {
    spec.map_or(0, |s| usize::from(s.cap_in > 0) + usize::from(s.cap_out > 0))
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

/// The recipes of a machine of `process` and the conversions its filling slot does (the chemical items of
/// `mekanism:chemical_conversion`, whose chemicals the tank takes only where some recipe uses them).
fn load_proc_book(source: Source, shape: process::Shape) -> process::Book {
    use process::{Out, Recipe, Shape};
    let mut book = process::Book::default();
    let Source::Files(kind) = source else { return book };
    for r in recipe_types::recipes_of_type(kind) {
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&r.json) else { continue };
        // What it takes: an item, a chemical, or both (oxidizing and crystallizing name theirs `input`).
        let (item_in, chem_in) = match shape {
            Shape::ItemToChem => (&json["input"], &serde_json::Value::Null),
            Shape::ChemToItem => (&serde_json::Value::Null, &json["input"]),
            _ => (&json["item_input"], &json["chemical_input"]),
        };
        let items = if item_in.is_null() {
            Vec::new()
        } else {
            recipe_types::ingredient_items(&item_in["ingredient"].to_string())
        };
        let chemicals = if chem_in.is_null() { Vec::new() } else { chemicals_of(&chem_in["ingredient"]) };
        if (!item_in.is_null() && items.is_empty()) || (!chem_in.is_null() && chemicals.is_empty()) {
            continue;
        }
        let Some(id) = json["output"]["id"].as_str() else { continue };
        let out = match shape {
            Shape::ItemChemToItem | Shape::ChemToItem => Out::Item { id: id.to_owned(), count: count_of(&json["output"]) },
            Shape::ItemToChem | Shape::ItemChemToChem => {
                Out::Chem { id: id.to_owned(), amount: json["output"]["amount"].as_u64().unwrap_or(1) }
            }
        };
        book.add(
            &items,
            &chemicals,
            Recipe {
                item_count: if item_in.is_null() { 0 } else { count_of(item_in) },
                amount: chem_in["amount"].as_u64().unwrap_or(0),
                per_tick: json["per_tick_usage"].as_bool().unwrap_or(false),
                out,
            },
        );
    }
    // The filling slot's conversions: an item to a chemical.
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
            Kind::Process => self.procs.get(def.block).is_none_or(process::Book::is_empty),
            // Their ticks are `run_continuous` and `run_tank`.
            Kind::Continuous | Kind::Tank | Kind::Generator | Kind::Reaction => false,
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
                Kind::Continuous | Kind::Tank | Kind::Generator | Kind::Reaction => {}
                Kind::Process => {
                    if let Some(spec) = &def.proc {
                        self.procs.insert(def.block, load_proc_book(def.source, spec.shape));
                    }
                }
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
        if config::disabled(layout, side) {
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

    /// What each face does with a machine's tanks, as its chemical sides say (one list of faces for each tank).
    fn tank_sides(&self, layout: Layout, facing: &str, cfg: &Config, mode: bool) -> Vec<Vec<FlowSide>> {
        config::tank_flow(layout, facing, cfg, mode)
            .iter()
            .map(|flow| flow.iter().map(|f| flow_side(*f)).collect())
            .collect()
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
        let layout = def.layout;
        let mut facing = sides::property(&e.state, "facing").unwrap_or("north").to_owned();
        let progress = e
            .data
            .get(..2)
            .map_or(0, |b| u32::from(u16::from_le_bytes([b[0], b[1]])));
        // Its side configuration: what its data holds after the progress, or a new machine's.
        let stored = Config::decode(e.data.get(2..).unwrap_or(&[]), layout);
        let mut cfg = stored.unwrap_or_else(|| config::default_config(layout));
        if stored.is_none()
            && matches!(def.layout, Layout::Infuser | Layout::Oxidizer | Layout::Dissolution | Layout::Crystallizer)
        {
            // Its block is not a furnace, which turns to face the player: a new one looks at the nearest player.
            let front = cube::facing_name(front_for(e));
            if front != facing {
                let _ = world::set_block_property(&e.dimension, e.pos.x, e.pos.y, e.pos.z, "facing", front);
                facing = front.to_owned();
            }
        }
        let key = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
        let mut dirty = stored.is_none() || !self.applied.contains(&key);
        if let Some(click) = self.take_click(e) {
            dirty |= self.configure(&mut cfg, &facing, layout, &click);
        }
        let dice = if def.kind == Kind::Sawmill || def.proc.is_some_and(|s| s.random_use) { self.roll(tick, &e.pos) } else { 0.0 };
        let slot = |i: usize| stack(e.items.get(i).unwrap_or(&None));
        // The infuser's tank before and after.
        let mut tank_now = [process::Tank::default(), process::Tank::default()];
        let mut tank_next = [process::Tank::default(), process::Tank::default()];
        // The slots after, the energy and progress after, what it did, and whether anything changed.
        let can = self.can;
        let counts = upgrades::counts_of(def, e);
        let (speed, energy_up, chem_up) = (counts[upgrades::SPEED], counts[upgrades::ENERGY], counts[upgrades::CHEMICAL]);
        let usage_now = upgrades::energy_per_tick(def.usage, speed, energy_up);
        let ticks_now = upgrades::ticks(machine::TICKS, speed);
        let (items, energy, next_progress, status, changed): (Vec<Option<ItemStack>>, u64, u32, Status, bool) = if !can {
            // `canFunction` is false: the machine rests where it is, its tanks as they were.
            if let Some(spec) = def.proc {
                let held = |i: usize| {
                    e.tanks.get(i).map_or_else(process::Tank::default, |t| process::Tank { chemical: t.substance.clone(), amount: t.amount })
                };
                if spec.cap_in > 0 {
                    tank_now[0] = held(0);
                }
                if spec.cap_out > 0 {
                    tank_now[1] = held(usize::from(spec.cap_in > 0));
                }
                tank_next = tank_now.clone();
            }
            (e.items.clone(), e.energy, progress, Status::Idle, false)
        } else { match def.kind {
            Kind::Electric => {
                let book = self.books.entry(def.block).or_default();
                let io = Io { input: slot(0), output: slot(1), energy: e.energy, progress, ticks: ticks_now };
                let (next, status) = step(usage_now, book, &io);
                (vec![item(&next.input), item(&next.output)], next.energy, next.progress, status, next != io)
            }
            Kind::Sawmill => {
                let io = sawmill::Io { input: slot(0), main: slot(1), secondary: slot(2), energy: e.energy, progress, ticks: ticks_now };
                let (next, status) = sawmill::step(usage_now, &self.saws, &io, dice);
                (
                    vec![item(&next.input), item(&next.main), item(&next.secondary)],
                    next.energy,
                    next.progress,
                    status,
                    next != io,
                )
            }
            Kind::Process => {
                let spec = def.proc.expect("a process machine has its spec");
                // Its tanks, in the order its kind lists them: the one the chemical goes into, then the one it comes out in.
                let held = |i: usize| {
                    e.tanks.get(i).map_or_else(process::Tank::default, |t| process::Tank {
                        chemical: t.substance.clone(),
                        amount: t.amount,
                    })
                };
                if spec.cap_in > 0 {
                    tank_now[0] = held(0);
                }
                if spec.cap_out > 0 {
                    tank_now[1] = held(usize::from(spec.cap_in > 0));
                }
                // Which slot is which: the input (0), the output item (1) or a chemical item slot that is not the
                // machine's to fill or empty, and the slot a chemical item is turned into the tank from (2; a
                // crystallizer's first; the oxidizer has none).
                let takes_item = spec.shape != Shape::ChemToItem;
                let makes_item = matches!(spec.shape, Shape::ItemChemToItem | Shape::ChemToItem);
                let conv_at = match def.layout {
                    Layout::Oxidizer => None,
                    Layout::Crystallizer => Some(0),
                    _ => Some(2),
                };
                let io = process::Io {
                    input: if takes_item { slot(0) } else { None },
                    output: if makes_item { slot(1) } else { None },
                    conv: conv_at.and_then(slot),
                    tank_in: tank_now[0].clone(),
                    tank_out: tank_now[1].clone(),
                    energy: e.energy,
                    progress,
                };
                let book = self.procs.entry(def.block).or_default();
                let mut spec = spec;
                spec.usage = usage_now;
                spec.ticks = upgrades::ticks(spec.ticks, speed);
                spec.mean = upgrades::gas_mean(speed, chem_up);
                let (next, status) = process::step(&spec, book, &io, dice);
                tank_next = [next.tank_in.clone(), next.tank_out.clone()];
                let mut slots: Vec<Option<ItemStack>> = (0..e.items.len()).map(|i| item(&slot(i))).collect();
                if takes_item {
                    slots[0] = item(&next.input);
                }
                if makes_item {
                    slots[1] = item(&next.output);
                }
                if let Some(at) = conv_at {
                    slots[at] = item(&next.conv);
                }
                (slots, next.energy, next.progress, status, next != io)
            }
            Kind::Continuous | Kind::Tank | Kind::Generator | Kind::Reaction => unreachable!("they have their own ticks"),
            Kind::Combiner => {
                let io = combiner::Io { main: slot(0), extra: slot(1), output: slot(2), energy: e.energy, progress, ticks: ticks_now };
                let (next, status) = combiner::step(usage_now, &self.combs, &io);
                (
                    vec![item(&next.main), item(&next.extra), item(&next.output)],
                    next.energy,
                    next.progress,
                    status,
                    next != io,
                )
            }
        } };
        // The upgrade slots after the machine's own stay as they are.
        let mut items = items;
        let have = items.len();
        items.extend(e.items.iter().skip(have).cloned());
        let active = status == Status::Running;
        // The block shows when it runs, with its entity kept.
        let lit = e.state.contains("lit=true");
        if active != lit {
            let _ = world::set_block_property(&e.dimension, e.pos.x, e.pos.y, e.pos.z, "lit", if active { "true" } else { "false" });
        }
        // The tank the chemical goes into takes in only what a recipe of the machine uses.
        if let Some(spec) = def.proc
            && spec.cap_in > 0
            && !self.filtered.contains(&key)
            && let Some(book) = self.procs.get(def.block).filter(|b| !b.is_empty())
        {
            let _ = world::set_tank_filter(&e.dimension, e.pos.x, e.pos.y, e.pos.z, 0, &book.accepted());
            self.filtered.insert(key.clone());
        }
        let energy_faces = dirty.then(|| self.apply_config(e, layout, &facing, &cfg));
        let capacity = upgrades::max_energy(def.usage * STORED_TICKS, counts[upgrades::ENERGY]);
        let mut data = menu_data(next_progress, energy, capacity, status, &cfg);
        let spec = def.proc;
        if let Some(spec) = spec {
            // The screen's numbers about the tanks: for each, how much and which chemical (an absent tank reads 0, 0).
            data.extend(chem::screen(&tank_next[0]));
            data.extend(chem::screen(&tank_next[1]));
            let _ = spec;
        }
        let control = Control::from_byte(e.data.get(redstone::offset(def.kind)).copied().unwrap_or(0));
        let mut saved = machine_data(next_progress, &cfg);
        let base_ticks = def.proc.map_or(machine::TICKS, |s| s.ticks);
        let shown = (upgrades::ticks(base_ticks, speed), self.install.get(&(e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z)).copied().unwrap_or(0), counts, usage_now);
        redstone::stamp(def.kind, control, capacity.saturating_sub(energy), shown, &mut saved, &mut data);
        if !changed && !dirty && data == e.menu_data && saved == e.data && !self.force {
            return None;
        }
        let tanks = spec
            .filter(|_| tank_next != tank_now || e.tanks.len() != tank_count(spec))
            .map(|s| {
                let mut v = Vec::new();
                if s.cap_in > 0 {
                    v.push(Tank { substance: tank_next[0].chemical.clone(), amount: tank_next[0].amount });
                }
                if s.cap_out > 0 {
                    v.push(Tank { substance: tank_next[1].chemical.clone(), amount: tank_next[1].amount });
                }
                v
            });
        let tank_faces = (spec.is_some() && dirty).then(|| self.tank_sides(layout, &facing, &cfg, false));
        Some(BlockEntityUpdate {
            dimension: e.dimension.clone(),
            pos: e.pos,
            data: saved,
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
    fn cube(&mut self, e: &BlockEntity, block: &str) -> Option<BlockEntityUpdate> {
        let (capacity, creative) = cube_capacity(block);
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
                // The slots of a machine of `process`; hoppers put into the input and the filling slot, and take from the output.
                Kind::Process => {
                    let (slots, inputs, outputs) = match def.layout {
                        Layout::Oxidizer => (2, vec![0], vec![]),
                        Layout::Crystallizer => (2, vec![0], vec![1]),
                        _ => (3, vec![0, 2], vec![1]),
                    };
                    ContainerSpec {
                        slots,
                        faces: config::item_faces(def.layout, "north", config::default_config(def.layout).items()),
                        inputs,
                        outputs,
                        menu: true,
                    }
                }
                // The chemical infuser, the washer and the separator: three slots, on the default configuration's faces.
                Kind::Continuous => {
                    let (inputs, outputs) = match def.layout {
                        Layout::ChemInfuser | Layout::Mixer => (vec![0, 1], vec![2]),
                        Layout::Rotary => (vec![0, 2], vec![1, 3]),
                        _ => (vec![0], vec![1, 2]),
                    };
                    ContainerSpec {
                        slots: if def.layout == Layout::Rotary { 5 } else { 3 },
                        faces: config::item_faces(def.layout, "north", config::default_config(def.layout).items()),
                        inputs,
                        outputs,
                        menu: true,
                    }
                }
                // The item in, the item out and the charge slot, on the faces of the default configuration.
                Kind::Reaction => ContainerSpec {
                    slots: 3,
                    faces: config::item_faces(def.layout, "north", config::default_config(def.layout).items()),
                    inputs: vec![0],
                    outputs: vec![1],
                    menu: true,
                },
                // A generator's fuel slot and its charge slot (the solar one has the charge slot alone), on the faces of its facing.
                Kind::Generator => {
                    let kind = def.generator.unwrap_or(GenKind::Solar);
                    ContainerSpec {
                        slots: if kind == GenKind::Solar { 1 } else { 2 },
                        faces: generators::item_faces(kind, "north"),
                        inputs: if kind == GenKind::Solar { Vec::new() } else { vec![0] },
                        outputs: Vec::new(),
                        menu: true,
                    }
                }
                // A tank's two slots: a fluid tank's bucket in (at the top) and out (at the bottom), a chemical tank's
                // drain and fill slots on the faces of its configuration.
                Kind::Tank => {
                    let fluid = def.tank.is_some_and(|t| t.fluid);
                    let faces = if fluid {
                        vec![vec![1], vec![0], Vec::new(), Vec::new(), Vec::new(), Vec::new()]
                    } else {
                        config::item_faces(def.layout, "north", config::default_config(def.layout).items())
                    };
                    ContainerSpec { slots: 2, faces, inputs: vec![0], outputs: vec![1], menu: true }
                }
                // The main input, the extra one and the output, on the faces of the default configuration.
                Kind::Combiner => ContainerSpec {
                    slots: 3,
                    faces: config::item_faces(Layout::Combiner, "north", config::default_config(Layout::Combiner).items()),
                    inputs: vec![0, 1],
                    outputs: vec![2],
                    menu: true,
                },
            });
            // The slot upgrades are put in and the one they come out in.
            if def.kind != Kind::Tank
                && let Some(c) = &mut k.container
            {
                c.slots += 2;
            }
            if def.kind == Kind::Generator {
                // It gives its energy by the faces the generator tick sets (the front, the bottom for the solar one).
                let kind = def.generator.unwrap_or(GenKind::Solar);
                let (cap, tank) = match kind {
                    GenKind::Bio => (generators::BIO_ENERGY, Some(("fluid", generators::BIO_TANK))),
                    GenKind::Heat => (generators::HEAT_ENERGY, Some(("fluid", generators::HEAT_TANK))),
                    GenKind::Gas => (generators::GAS_ENERGY, Some(("chemical", generators::GAS_TANK))),
                    GenKind::Solar => (generators::SOLAR_ENERGY, None),
                };
                k.energy = Some(EnergySpec { capacity: cap, max_receive: 0, max_extract: cap, faces: generators::energy_faces(kind, "north") });
                if let Some((fluid, capacity)) = tank {
                    k.tanks.push(TankSpec { kind: fluid.into(), capacity, max_rate: capacity.min(1000), faces: generators::tank_faces("north") });
                }
            } else if def.kind != Kind::Tank {
                // The separator stores more: its use is tiny and a bucket of water is many ticks of it.
                let stored = def.usage * STORED_TICKS * if def.layout == Layout::Separator { 25 } else { 1 };
                k.energy = Some(energy(stored, stored, FlowSide::Input));
            }
            if let Some(t) = def.tank {
                let flow = if t.fluid { vec![FlowSide::Both; 6] } else { vec![FlowSide::Both; 6] };
                k.tanks.push(TankSpec {
                    kind: if t.fluid { "fluid" } else { "chemical" }.into(),
                    capacity: t.capacity,
                    max_rate: t.rate,
                    faces: flow,
                });
            }
            if def.kind == Kind::Reaction {
                // Its fluid tank, then the gas it takes in and the gas it makes, on the default configuration's faces.
                let flow = config::tank_flow(def.layout, "north", &config::default_config(def.layout), false);
                for (i, fluid) in [true, false, false].into_iter().enumerate() {
                    k.tanks.push(TankSpec {
                        kind: if fluid { "fluid" } else { "chemical" }.into(),
                        capacity: reaction::TANK,
                        max_rate: 1000,
                        faces: flow.get(i).map_or_else(|| vec![FlowSide::None; 6], |f| f.iter().map(|x| flow_side(*x)).collect()),
                    });
                }
            }
            if let Some(spec) = def.cont {
                // Chemicals go through chemical tanks and fluids through fluid ones; the faces are the configuration's.
                let flow = config::tank_flow(def.layout, "north", &config::default_config(def.layout), false);
                for (i, cap) in spec.caps.iter().enumerate().filter(|(_, c)| **c > 0) {
                    let fluid = i == 0 && spec.shape != continuous::Shape::Infuser;
                    k.tanks.push(TankSpec {
                        kind: if fluid { "fluid" } else { "chemical" }.into(),
                        capacity: *cap,
                        max_rate: (*cap).min(1000),
                        faces: flow.get(i).map_or_else(|| vec![FlowSide::None; 6], |f| f.iter().map(|x| flow_side(*x)).collect()),
                    });
                }
            }
            if let Some(spec) = def.proc {
                // The tanks: the one the chemical goes into, then the one it comes out in. Every chemical is the same
                // family, so a tank takes from a neighbour that holds one; the faces are the default configuration's.
                let flow = config::tank_flow(def.layout, "north", &config::default_config(def.layout), false);
                let mut tank = |capacity: u64, i: usize| TankSpec {
                    kind: "chemical".into(),
                    capacity,
                    max_rate: capacity.min(1000),
                    faces: flow.get(i).map_or_else(|| vec![FlowSide::None; 6], |f| f.iter().map(|x| flow_side(*x)).collect()),
                };
                if spec.cap_in > 0 {
                    k.tanks.push(tank(spec.cap_in, 0));
                }
                if spec.cap_out > 0 {
                    k.tanks.push(tank(spec.cap_out, usize::from(spec.cap_in > 0)));
                }
            }
            r = r.block_entity_kind(k);
        }
        // The discharge and charge slots of its screen; no face reaches them (the cube's item sides are still to do).
        let slots = || ContainerSpec { slots: 2, faces: vec![Vec::new(); 6], inputs: Vec::new(), outputs: Vec::new(), menu: true };
        for (block, capacity, rate) in CUBE_TIERS {
            let mut tier = kind(block, true);
            tier.container = Some(slots());
            tier.energy = Some(energy(capacity, rate, FlowSide::Both));
            r = r.block_entity_kind(tier);
        }
        let mut creative = kind("mekanism:creative_energy_cube", true);
        creative.container = Some(slots());
        creative.energy = Some(energy(CREATIVE_CUBE, 1_000_000_000, FlowSide::Output));
        r = r.block_entity_kind(creative);
        // Cables, tubes and pipes of the four tiers (`CableTier`, `TubeTier`, `PipeTier`): what each carries a tick.
        for (i, tier) in ["basic", "advanced", "elite", "ultimate"].iter().enumerate() {
            for (name, medium, rates) in [
                ("universal_cable", TransmitterMedium::Energy, CABLE_RATES),
                ("pressurized_tube", TransmitterMedium::Tank("chemical".into()), TUBE_RATES),
                ("mechanical_pipe", TransmitterMedium::Tank("fluid".into()), PIPE_RATES),
            ] {
                let mut k = kind(&format!("mekanism:{tier}_{name}"), false);
                k.keep_data = false;
                k.transmitter = Some(TransmitterSpec { medium, throughput: rates[i] });
                r = r.block_entity_kind(k);
            }
        }
        Ok((Mekanism {
            books: HashMap::new(),
            tried: HashMap::new(),
            saws: sawmill::Book::default(),
            combs: combiner::Book::default(),
            procs: HashMap::new(),
            conts: HashMap::new(),
            filtered: HashSet::new(),
            pending: Vec::new(),
            tool: HashMap::new(),
            applied: HashSet::new(),
            rng: 0,
            now: 0,
            last_block_use: HashMap::new(),
            flow: HashMap::new(),
            buttons: HashMap::new(),
            suns: HashMap::new(),
            last_power: HashMap::new(),
            can: true,
            install: HashMap::new(),
            removals: HashMap::new(),
            capacity: HashMap::new(),
            force: false,
            reactions: reaction::Book::default(),
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
        // Salt on the floors of lakes and oceans: disks of dirt and clay turned to salt (`SaltConfig`, placed on the ocean floor
        // with water over it).
        for n in 0..worldgen::SALT_PER_CHUNK {
            let (x, z, radius) = worldgen::salt_disk(chunk_x, chunk_z, n);
            // The floor: the first block that stops motion going down from the sky (air, water, and the plants of the water
            // that have no collision are passed; a chunk section with nothing in it has no blocks to read).
            let mut floor = None;
            for y in (worldgen::BOTTOM..320).rev() {
                let Some(block) = world::get_block(dimension, x, y, z) else { continue };
                let name = block.split('[').next().unwrap_or("");
                if matches!(name, "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air" | "minecraft:water") {
                    continue;
                }
                if world::get_collision(dimension, x, y, z).is_some_and(|c| !c.is_empty()) {
                    floor = Some(y);
                    break;
                }
            }
            let Some(floor) = floor else { continue };
            let wet = |b: &str| {
                let name = b.split('[').next().unwrap_or("");
                name == "minecraft:water"
                    || b.contains("waterlogged=true")
                    || matches!(name, "minecraft:seagrass" | "minecraft:tall_seagrass" | "minecraft:kelp" | "minecraft:kelp_plant" | "minecraft:bubble_column")
            };
            if !world::get_block(dimension, x, floor + 1, z).is_some_and(|b| wet(&b)) {
                continue;
            }
            for p in worldgen::salt_blocks(x, floor + 1, z, radius) {
                let Some(block) = world::get_block(dimension, p[0], p[1], p[2]) else { continue };
                if worldgen::salt_replaces(block.split('[').next().unwrap_or("")) {
                    changes.push(BlockChange { x: p[0], y: p[1], z: p[2], state: "mekanism:block_salt".into() });
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
        if id == redstone::NEXT || id == redstone::PREVIOUS {
            return self.redstone_button(def, id, &e);
        }
        if upgrades::is_removal(id) {
            self.removals.entry((e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z)).or_default().push(id);
            return None;
        }
        if def.kind == Kind::Generator {
            return None;
        }
        if matches!(def.kind, Kind::Continuous | Kind::Tank | Kind::Reaction) {
            return self.plus_button(def, id, &e);
        }
        let layout = def.layout;
        let facing = sides::property(&e.state, "facing").unwrap_or("north").to_owned();
        let mut cfg = Config::decode(e.data.get(2..).unwrap_or(&[]), layout).unwrap_or_else(|| config::default_config(layout));
        if id == chem::DUMP && def.block == "mekanism:metallurgic_infuser" {
            // GuiDumpButton: the tank is emptied (`dump`).
            let tail: Vec<i32> = e.menu_data.get(20..).map(<[i32]>::to_vec).unwrap_or_default();
            let mut data = e.menu_data.clone();
            data.truncate(16);
            data.extend(chem::screen(&process::Tank::default()));
            data.extend(chem::screen(&process::Tank::default()));
            data.extend(tail);
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
        config::enforce(layout, &mut cfg);
        let progress = e.data.get(..2).map_or(0, |b| u32::from(u16::from_le_bytes([b[0], b[1]])));
        let faces = self.apply_config(&e, layout, &facing, &cfg);
        // The screen's first six numbers stay; the configuration's four follow them.
        let tail: Vec<i32> = e.menu_data.get(16..).map(<[i32]>::to_vec).unwrap_or_default();
        let mut data = e.menu_data.clone();
        data.resize(6, 0);
        data.extend(cfg.screen_words());
        data.extend(tail);
        let tank_faces = def.proc.map(|_| self.tank_sides(layout, &facing, &cfg, false));
        Some(BlockEntityUpdate {
            dimension: e.dimension.clone(),
            pos: e.pos,
            data: {
                let mut saved = machine_data(progress, &cfg);
                saved.extend(e.data.get(saved.len()..).unwrap_or(&[]));
                saved
            },
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
                self.can = true;
                self.force = false;
                // Upgrades install and come out before the machine works, in a copy it then runs from.
                let mut e2 = e.clone();
                if def.kind != Kind::Tank {
                    self.read_control(def, &e2);
                    self.force = self.tick_upgrades(def, &mut e2);
                }
                let e = &e2;
                out.extend(match def.kind {
                    Kind::Continuous => self.run_continuous(def, e, tick),
                    Kind::Tank => self.run_tank(def, e),
                    Kind::Generator => self.run_generator(def, e, tick),
                    Kind::Reaction => self.run_reaction(def, e, tick),
                    _ => self.run(def, e, tick),
                });
            } else if block.ends_with("_energy_cube") {
                out.extend(self.cube(e, block));
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

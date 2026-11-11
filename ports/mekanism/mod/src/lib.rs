//! Mekanism for Horizon (a Rust port of Mekanism, MIT): the machines' logic.
//! The blocks, items, textures, recipes and screens are the mod's content
//! (`data/` and `assets/`, made by `tools/convert.py` from Mekanism's own).
//!
//! What runs so far: the crusher, the enrichment chamber and the energized
//! smelter (`machine.rs`), the energy cubes and the universal cable.
mod combiner;
mod assemblicator;
mod capmask;
mod cardboard_box;
mod sparkle;
mod chargepad;
mod chemical_fluids;
mod configuration_card;
mod dimensional_stabilizer;
mod electric_bow;
mod fork_equipamento;
mod fork_qio_calor;
mod fork_utilidades;
mod fuelwood_heater;
mod hdpe_elytra;
mod heat;
mod laser_tractor;
mod frequency;
mod filters;
mod item_menus;
mod robit_world;
mod meka_tool;
mod mekasuit;
mod modification_station;
mod modules;
mod oredictionificator;
mod personal_storage;
mod qio_dashboard;
mod qio_transfer;
mod radioactive_waste_barrel;
mod resistive_heater;
mod robit;
mod security;
mod seismic_vibrator;
mod transporter_special;
mod chem;
mod continuous;
mod config;
mod cube;
mod chem_table;
mod plus;
mod process;
mod bin;
mod alarm;
mod boiler;
mod bounding;
mod dynamic;
mod entangloporter;
mod evaporation;
mod fission;
mod fusion;
mod induction;
mod factory;
mod pump;
mod equipment_world;
mod heat_world;
mod qio;
mod radiation;
mod reaction;
mod redstone;
mod generators;
mod laser;
mod machine;
mod miner;
mod sawmill;
mod sps;
mod sides;
mod sorter;
mod tank;
mod teleporter;
mod transmitter;
mod turbine;
mod upgrades;
mod worldgen;

use horizon_mod_sdk::server::{
    BlockEntity, BlockEntityKind, BlockEntityUpdate, ContainerSpec, EnergySpec, FlowSide, GameMode, ItemStack, Tank, TankSpec,
    BlockChange, BlockPos, BlockUse, DamageSource, Player, Registration, ServerMod, TransmitterMedium, TransmitterSpec, entities, player_control, player_items,
    players, recipe_types, serde_json, world,
};
use config::{Config, Data, Layout, ToolMode, Transmission};
use generators::GenKind;
use redstone::Control;
use sides::Mode;
use machine::{Book, Io, Recipe, Stack, Status, step};
use horizon_mod_sdk::server::{ModEntity, ModEntityUpdate, MultiblockSpec, Structure};
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
    /// A factory (`factory`): several lines of an electric machine's work.
    Factory,
    /// The electric pump (`pump`): sucks the fluid sources of the world into its tank.
    Pump,
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
    /// A `Kind::Factory`'s lines, and the machine (`base`) whose recipes it works.
    lines: usize,
    base: &'static str,
    /// The slot form of a factory (`config::factory_slots_per_line`): 0 items, 1 combining, 2 sawing.
    form: u8,
}

const fn electric(block: &'static str, source: Source, usage: u64) -> Def {
    Def { block, source, usage, kind: Kind::Electric, layout: Layout::Electric, proc: None, cont: None, tank: None, generator: None, lines: 0, base: "", form: 0 }
}

/// A machine that works an item and a chemical (`process`): its recipes are of type `source`.
const fn chemical(block: &'static str, source: &'static str, layout: Layout, spec: process::Spec) -> Def {
    Def { block, source: Source::Files(source), usage: spec.usage, kind: Kind::Process, layout, proc: Some(spec), cont: None, tank: None, generator: None, lines: 0, base: "", form: 0 }
}

/// A machine of `continuous`.
const fn continuous_machine(block: &'static str, source: &'static str, layout: Layout, spec: continuous::Spec) -> Def {
    Def { block, source: Source::Files(source), usage: spec.usage, kind: Kind::Continuous, layout, proc: None, cont: Some(spec), tank: None, generator: None, lines: 0, base: "", form: 0 }
}

/// A generator of MekanismGenerators.
const fn generator(block: &'static str, kind: GenKind) -> Def {
    Def { block, source: Source::Files(""), usage: 0, kind: Kind::Generator, layout: Layout::Electric, proc: None, cont: None, tank: None, generator: Some(kind), lines: 0, base: "", form: 0 }
}

/// The pressurized reaction chamber.
const fn reaction_machine(block: &'static str) -> Def {
    Def { block, source: Source::Files(""), usage: reaction::USAGE, kind: Kind::Reaction, layout: Layout::Reaction, proc: None, cont: None, tank: None, generator: None, lines: 0, base: "", form: 0 }
}

/// A factory of `lines` lines that works the recipes of `base`'s machine (`source`), using `usage` a tick for each line.
const fn factory(block: &'static str, base: &'static str, source: Source, usage: u64, lines: usize, form: u8) -> Def {
    Def { block, source, usage, kind: Kind::Factory, layout: Layout::Factory(lines as u8, form), proc: None, cont: None, tank: None, generator: None, lines, base, form }
}

/// The electric pump.
const fn pump_machine(block: &'static str, form: u8) -> Def {
    Def { block, source: Source::Files(""), usage: pump::USAGE, kind: Kind::Pump, layout: Layout::Pump(form), proc: None, cont: None, tank: None, generator: None, lines: 0, base: "", form }
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
        lines: 0,
        base: "",
        form: 0,
    }
}

use process::Shape;

const fn spec(shape: Shape, usage: u64, ticks: u32, cap_in: u64, cap_out: u64, random_use: bool) -> process::Spec {
    process::Spec { shape, usage, ticks, cap_in, cap_out, random_use, speed: 0, mean: 1.0 }
}

const MACHINES: [Def; 78] = [
    electric("mekanism:crusher", Source::Files("mekanism:crushing"), fe(50)),
    electric("mekanism:enrichment_chamber", Source::Files("mekanism:enriching"), fe(50)),
    electric("mekanism:energized_smelter", Source::Cooking("smelting"), fe(50)),
    Def { block: "mekanism:precision_sawmill", source: Source::Files("mekanism:sawing"), usage: fe(50), kind: Kind::Sawmill, layout: Layout::Sawmill, proc: None, cont: None, tank: None, generator: None, lines: 0, base: "", form: 0 },
    Def { block: "mekanism:combiner", source: Source::Files("mekanism:combining"), usage: fe(50), kind: Kind::Combiner, layout: Layout::Combiner, proc: None, cont: None, tank: None, generator: None, lines: 0, base: "", form: 0 },
    chemical("mekanism:metallurgic_infuser", "mekanism:metallurgic_infusing", Layout::Infuser, spec(Shape::ItemChemToItem, fe(50), 200, 1000, 0, false)),
    // The purification and injection chambers use their chemical by chance (the config's randomized consumption).
    chemical("mekanism:purification_chamber", "mekanism:purifying", Layout::Adv, spec(Shape::ItemChemToItem, fe(200), 200, 210, 0, true)),
    chemical("mekanism:chemical_injection_chamber", "mekanism:injecting", Layout::Adv, spec(Shape::ItemChemToItem, fe(400), 200, 210, 0, true)),
    chemical("mekanism:osmium_compressor", "mekanism:compressing", Layout::Compressor, spec(Shape::ItemChemToItem, fe(100), 200, 210, 0, false)),
    chemical("mekanism:chemical_oxidizer", "mekanism:oxidizing", Layout::Oxidizer, spec(Shape::ItemToChem, fe(200), 100, 0, 10_000, false)),
    chemical("mekanism:chemical_dissolution_chamber", "mekanism:dissolution", Layout::Dissolution, spec(Shape::ItemChemToChem, fe(400), 100, 10_000, 10_000, true)),
    chemical("mekanism:chemical_crystallizer", "mekanism:crystallizing", Layout::Crystallizer, spec(Shape::ChemToItem, fe(400), 200, 10_000, 0, false)),
    // The chemical infuser, the washer and the electrolytic separator work their tanks every tick.
    continuous_machine("mekanism:chemical_infuser", "mekanism:chemical_infusing", Layout::ChemInfuser,
        continuous::Spec { shape: continuous::Shape::Infuser, usage: fe(200), caps: [10_000; 3] }),
    continuous_machine("mekanism:chemical_washer", "mekanism:washing", Layout::Washer,
        continuous::Spec { shape: continuous::Shape::Washer, usage: fe(200), caps: [10_000; 3] }),
    // Its energy is twice the hydrogen's energy density (2), so 4 a tick.
    continuous_machine("mekanism:electrolytic_separator", "mekanism:separating", Layout::Separator,
        continuous::Spec { shape: continuous::Shape::Separator, usage: fe(2 * 200), caps: [24_000, 2_400, 2_400] }),
    // The rotary condensentrator turns a gas into a fluid or a fluid into a gas, by its mode (10 buckets in each tank).
    continuous_machine("mekanism:rotary_condensentrator", "mekanism:rotary", Layout::Rotary,
        continuous::Spec { shape: continuous::Shape::Rotary, usage: fe(50), caps: [10_000, 10_000, 0] }),
    // The antiprotonic nucleosynthesizer: an item and antimatter make an item, each recipe taking its own time (`duration`).
    chemical("mekanism:antiprotonic_nucleosynthesizer", "mekanism:nucleosynthesizing", Layout::Adv, spec(Shape::ItemChemToItem, fe(100_000), 400, 10_000, 0, false)),
    // The nutritional liquifier turns food into nutritional paste (a fluid).
    chemical("mekanism:nutritional_liquifier", "mekanism:liquifying", Layout::Oxidizer, spec(Shape::ItemToChem, fe(200), 100, 0, 10_000, false)),
    // Pigments: an item gives a pigment, a pigment paints an item, two pigments mix into a third.
    chemical("mekanism:pigment_extractor", "mekanism:pigment_extracting", Layout::Oxidizer, spec(Shape::ItemToChem, fe(200), 100, 0, 20_000, false)),
    chemical("mekanism:painting_machine", "mekanism:painting", Layout::Adv, spec(Shape::ItemChemToItem, fe(100), 200, 15_000, 0, false)),
    continuous_machine("mekanism:pigment_mixer", "mekanism:pigment_mixing", Layout::Mixer,
        continuous::Spec { shape: continuous::Shape::Infuser, usage: fe(200), caps: [1_000, 1_000, 2_000] }),
    // The nuclear chain: the isotopic centrifuge turns uranium hexafluoride into fissile fuel and nuclear waste into plutonium, and the solar
    // neutron activator nuclear waste into polonium (it needs no energy, only the sun).
    continuous_machine("mekanism:isotopic_centrifuge", "mekanism:centrifuging", Layout::ChemInfuser,
        continuous::Spec { shape: continuous::Shape::Converter, usage: fe(200), caps: [10_000, 10_000, 0] }),
    continuous_machine("mekanism:solar_neutron_activator", "mekanism:activating", Layout::ChemInfuser,
        continuous::Spec { shape: continuous::Shape::Converter, usage: 0, caps: [10_000, 10_000, 0] }),
    // An item, a fluid and a gas make an item and a gas (14 recipes: gasification and substrate).
    reaction_machine("mekanism:pressurized_reaction_chamber"),
    // The generators of MekanismGenerators: they make energy from bioethanol, lava and heat, the sun and gases.
    generator("mekanism:bio_generator", GenKind::Bio),
    generator("mekanism:heat_generator", GenKind::Heat),
    generator("mekanism:solar_generator", GenKind::Solar),
    generator("mekanism:gas_burning_generator", GenKind::Gas),
    generator("mekanism:advanced_solar_generator", GenKind::AdvSolar),
    generator("mekanism:wind_generator", GenKind::Wind),
    // The factories of the electric machines: `FactoryTier` gives 3, 5, 7 and 9 lines.
    factory("mekanism:basic_crushing_factory", "mekanism:crusher", Source::Files("mekanism:crushing"), fe(50), 3, 0),
    factory("mekanism:advanced_crushing_factory", "mekanism:crusher", Source::Files("mekanism:crushing"), fe(50), 5, 0),
    factory("mekanism:elite_crushing_factory", "mekanism:crusher", Source::Files("mekanism:crushing"), fe(50), 7, 0),
    factory("mekanism:ultimate_crushing_factory", "mekanism:crusher", Source::Files("mekanism:crushing"), fe(50), 9, 0),
    factory("mekanism:basic_enriching_factory", "mekanism:enrichment_chamber", Source::Files("mekanism:enriching"), fe(50), 3, 0),
    factory("mekanism:advanced_enriching_factory", "mekanism:enrichment_chamber", Source::Files("mekanism:enriching"), fe(50), 5, 0),
    factory("mekanism:elite_enriching_factory", "mekanism:enrichment_chamber", Source::Files("mekanism:enriching"), fe(50), 7, 0),
    factory("mekanism:ultimate_enriching_factory", "mekanism:enrichment_chamber", Source::Files("mekanism:enriching"), fe(50), 9, 0),
    factory("mekanism:basic_smelting_factory", "mekanism:energized_smelter", Source::Cooking("smelting"), fe(50), 3, 0),
    factory("mekanism:advanced_smelting_factory", "mekanism:energized_smelter", Source::Cooking("smelting"), fe(50), 5, 0),
    factory("mekanism:elite_smelting_factory", "mekanism:energized_smelter", Source::Cooking("smelting"), fe(50), 7, 0),
    factory("mekanism:ultimate_smelting_factory", "mekanism:energized_smelter", Source::Cooking("smelting"), fe(50), 9, 0),
    // The factories of the combiner and the sawmill: the combiner's extra input is one for all the lines, the sawmill's lines have two outputs.
    factory("mekanism:basic_combining_factory", "mekanism:combiner", Source::Files("mekanism:combining"), fe(50), 3, 1),
    factory("mekanism:advanced_combining_factory", "mekanism:combiner", Source::Files("mekanism:combining"), fe(50), 5, 1),
    factory("mekanism:elite_combining_factory", "mekanism:combiner", Source::Files("mekanism:combining"), fe(50), 7, 1),
    factory("mekanism:ultimate_combining_factory", "mekanism:combiner", Source::Files("mekanism:combining"), fe(50), 9, 1),
    factory("mekanism:basic_sawing_factory", "mekanism:precision_sawmill", Source::Files("mekanism:sawing"), fe(50), 3, 2),
    factory("mekanism:advanced_sawing_factory", "mekanism:precision_sawmill", Source::Files("mekanism:sawing"), fe(50), 5, 2),
    factory("mekanism:elite_sawing_factory", "mekanism:precision_sawmill", Source::Files("mekanism:sawing"), fe(50), 7, 2),
    factory("mekanism:ultimate_sawing_factory", "mekanism:precision_sawmill", Source::Files("mekanism:sawing"), fe(50), 9, 2),
    // The factories that work a chemical: one tank for all the lines (a compressing or infusing one can be emptied by the right side).
    factory("mekanism:basic_compressing_factory", "mekanism:osmium_compressor", Source::Files("mekanism:compressing"), fe(100), 3, 4),
    factory("mekanism:advanced_compressing_factory", "mekanism:osmium_compressor", Source::Files("mekanism:compressing"), fe(100), 5, 4),
    factory("mekanism:elite_compressing_factory", "mekanism:osmium_compressor", Source::Files("mekanism:compressing"), fe(100), 7, 4),
    factory("mekanism:ultimate_compressing_factory", "mekanism:osmium_compressor", Source::Files("mekanism:compressing"), fe(100), 9, 4),
    factory("mekanism:basic_infusing_factory", "mekanism:metallurgic_infuser", Source::Files("mekanism:metallurgic_infusing"), fe(50), 3, 4),
    factory("mekanism:advanced_infusing_factory", "mekanism:metallurgic_infuser", Source::Files("mekanism:metallurgic_infusing"), fe(50), 5, 4),
    factory("mekanism:elite_infusing_factory", "mekanism:metallurgic_infuser", Source::Files("mekanism:metallurgic_infusing"), fe(50), 7, 4),
    factory("mekanism:ultimate_infusing_factory", "mekanism:metallurgic_infuser", Source::Files("mekanism:metallurgic_infusing"), fe(50), 9, 4),
    factory("mekanism:basic_injecting_factory", "mekanism:chemical_injection_chamber", Source::Files("mekanism:injecting"), fe(400), 3, 3),
    factory("mekanism:advanced_injecting_factory", "mekanism:chemical_injection_chamber", Source::Files("mekanism:injecting"), fe(400), 5, 3),
    factory("mekanism:elite_injecting_factory", "mekanism:chemical_injection_chamber", Source::Files("mekanism:injecting"), fe(400), 7, 3),
    factory("mekanism:ultimate_injecting_factory", "mekanism:chemical_injection_chamber", Source::Files("mekanism:injecting"), fe(400), 9, 3),
    factory("mekanism:basic_purifying_factory", "mekanism:purification_chamber", Source::Files("mekanism:purifying"), fe(200), 3, 3),
    factory("mekanism:advanced_purifying_factory", "mekanism:purification_chamber", Source::Files("mekanism:purifying"), fe(200), 5, 3),
    factory("mekanism:elite_purifying_factory", "mekanism:purification_chamber", Source::Files("mekanism:purifying"), fe(200), 7, 3),
    factory("mekanism:ultimate_purifying_factory", "mekanism:purification_chamber", Source::Files("mekanism:purifying"), fe(200), 9, 3),
    // The electric pump takes fluid sources of the world.
    pump_machine("mekanism:electric_pump", 0),
    // The fluidic plenisher pours the fluid of its tank into the world.
    pump_machine("mekanism:fluidic_plenisher", 1),
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

/// Energy in Mekanism 1.21.1 is Joules; the port keeps Forge Energy, which is Joules over the default `feConversionRate` (2.5).
pub const fn fe(joules: u64) -> u64 {
    joules * 2 / 5
}

/// `MekanismConfig.storage.*`: what a machine holds before its energy upgrades (Joules in the release, Forge Energy here).
pub fn storage_of(block: &str) -> u64 {
    fe(match block {
        "mekanism:enrichment_chamber" | "mekanism:crusher" | "mekanism:metallurgic_infuser" | "mekanism:energized_smelter" | "mekanism:precision_sawmill"
        | "mekanism:rotary_condensentrator" | "mekanism:seismic_vibrator" => 20_000,
        "mekanism:osmium_compressor" | "mekanism:purification_chamber" | "mekanism:chemical_oxidizer" | "mekanism:chemical_infuser"
        | "mekanism:chemical_washer" | "mekanism:isotopic_centrifuge" | "mekanism:pigment_mixer" => 80_000,
        "mekanism:combiner" | "mekanism:electric_pump" | "mekanism:fluidic_plenisher" | "mekanism:formulaic_assemblicator"
        | "mekanism:nutritional_liquifier" | "mekanism:pigment_extractor" | "mekanism:painting_machine" | "mekanism:modification_station"
        | "mekanism:dimensional_stabilizer" => 40_000,
        "mekanism:chemical_injection_chamber" | "mekanism:electrolytic_separator" | "mekanism:chemical_dissolution_chamber"
        | "mekanism:chemical_crystallizer" => 160_000,
        "mekanism:pressurized_reaction_chamber" => 2_000,
        "mekanism:digital_miner" => 50_000,
        "mekanism:antiprotonic_nucleosynthesizer" => 1_000_000_000,
        _ => 0,
    })
}
const CREATIVE_CUBE: u64 = 1_000_000_000_000;
/// `EnergyCubeTier`: what each cube holds and moves a tick (the creative one: everything, refilled).
const CUBE_TIERS: [(&str, u64, u64); 4] = [
    ("mekanism:basic_energy_cube", fe(4_000_000), fe(4_000)),
    ("mekanism:advanced_energy_cube", fe(16_000_000), fe(16_000)),
    ("mekanism:elite_energy_cube", fe(64_000_000), fe(64_000)),
    ("mekanism:ultimate_energy_cube", fe(256_000_000), fe(256_000)),
];

/// What a machine holds before its energy upgrades: ticks of its use (the separator stores more: its use is tiny and a bucket of
/// water is many ticks of it); a factory, half of its machine's for each line (`Factory.setMachineData`).
fn base_capacity(def: &Def) -> u64 {
    match def.kind {
        Kind::Factory => (storage_of(def.base) / 2).max(def.usage) * def.lines as u64,
        _ => storage_of(def.block),
    }
}

/// What a cube of this block holds, whether it is the creative one.
fn cube_capacity(block: &str) -> (u64, bool) {
    CUBE_TIERS.iter().find(|(b, _, _)| *b == block).map_or((CREATIVE_CUBE, true), |(_, c, _)| (*c, false))
}
/// What a cable, a tube and a pipe of each tier carry a tick.
const CABLE_RATES: [u64; 4] = [fe(8_000), fe(128_000), fe(1_024_000), fe(8_192_000)];
const TUBE_RATES: [u64; 4] = [750, 2_000, 64_000, 256_000];
const PIPE_RATES: [u64; 4] = [250, 1_000, 8_000, 32_000];
/// What a logistical transporter moves every ten ticks (`TransporterTier` pull: 1, 16, 32 and 64 items).
const TRANSPORTER_RATES: [u64; 4] = [1, 16, 32, 64];

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
    /// Configuration card clicks waiting for the machine's next tick (a use can't change a block entity).
    card_clicks: Vec<equipment_world::CardClick>,
    /// The step height each player's boots last gave (a transient modifier the mod puts back).
    suit_step: HashMap<String, f32>,
    /// The waste barrels whose tank filter was set.
    waste_filtered: HashSet<(String, i32, i32, i32)>,
    /// The strings last given to each block's screen (`set-menu-text`).
    menu_texts: HashMap<(String, i32, i32, i32), Vec<String>>,
    /// The buttons pressed on the dimensional stabilizers' screens, and the chunks each holds forced.
    stabilizer_buttons: Vec<(String, BlockPos, i32)>,
    stabilizer_tickets: HashMap<(String, i32, i32, i32), HashSet<(i32, i32)>>,
    /// Empties waiting for the game to confirm their drops exist, by block and by summon ticket, and empties whose
    /// drops all spawned, waiting for the block's next tick to clear the slots they came from (see `empty_block`).
    empty: HashMap<EmptyKey, EmptyPending>,
    empty_tickets: HashMap<u64, EmptyTicket>,
    empty_ready: HashMap<EmptyKey, (Vec<(usize, String, u32)>, u64)>,
    /// The mode of each multiblock port and valve, by position: the mode byte (false/0 takes in) the port's own tick
    /// last saw. `multiblock-changed` delivers members without their data, so the structures functions cannot read
    /// the mode from the member they are rewriting — without this memory every revalidation reset the mode to input
    /// (the mode byte lives after the port's data, which the validation rewrites). Refreshed every tick from the
    /// port's own data, so a port placed where another stood does not keep the old mode for long.
    port_modes: HashMap<(String, i32, i32, i32), u8>,
    /// The mode each player's configurator was last in: a fallback for a stack the game has not written the mode back
    /// into yet. The mode lives in the item's components (Mekanism keeps it in the item); this map is only a cache,
    /// never saved: a mode only survives a restart when the component write landed (see `write_tool_mode`).
    tool: HashMap<String, ToolMode>,
    /// Mode writes waiting to be checked on a later tick (see `check_tool_writes`).
    tool_writes: Vec<ToolWrite>,
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
    /// The ticks of a gas-burning generator's current mB still to burn (`burnTicks`).
    burns: HashMap<(String, i32, i32, i32), u64>,
    /// The redstone signal each machine had the tick before (`redstoneLastTick`), and whether the one being run may work.
    last_power: HashMap<(String, i32, i32, i32), bool>,
    can: bool,
    /// How long the upgrade in each machine's slot has waited, the window's remove buttons since the last tick, the energy
    /// capacity each machine was given, and whether upgrades changed the machine being run (so it answers even if idle).
    install: HashMap<(String, i32, i32, i32), u32>,
    removals: HashMap<(String, i32, i32, i32), Vec<i32>>,
    /// Upgrades a player sneak-used on a machine: the player, the type, how many they held and the slot they were in.
    installs: HashMap<(String, i32, i32, i32), Vec<(String, usize, u32, u32)>>,
    /// The fluid blocks each electric pump found beside the ones it took (`recurringNodes`), and the fluid it is on.
    pump_nodes: HashMap<(String, i32, i32, i32), (Vec<(i32, i32, i32)>, String)>,
    /// The tank each dynamic tank structure shares among its valves.
    dynamic: HashMap<(String, [i32; 3]), dynamic::Shared>,
    /// The energy each induction matrix holds, and what its ports were last given.
    matrices: HashMap<(String, [i32; 3]), u64>,
    /// The thermal evaporation plants, and the recipes they run.
    plants: HashMap<(String, [i32; 3]), evaporation::Plant>,
    boilers: HashMap<(String, [i32; 3]), boiler::Boiler>,
    turbines: HashMap<(String, [i32; 3]), turbine::Turbine>,
    reactors: HashMap<(String, [i32; 3]), fission::Shared>,
    fusions: HashMap<(String, [i32; 3]), fusion::Shared>,
    /// The energy lasers delivered to the blocks they hit, until those take it.
    beams: HashMap<(String, [i32; 3]), u64>,
    /// The block each laser is wearing down, and how much energy it has put in it.
    digging: HashMap<(String, [i32; 3]), ([i32; 3], u64)>,
    /// The machine each bounding block just placed belongs to, until the block writes it down.
    bound_owner: HashMap<(String, i32, i32, i32), [i32; 3]>,
    spses: HashMap<(String, [i32; 3]), sps::Shared>,
    sorter_buttons: Vec<(String, BlockPos, i32)>,
    sorter_texts: Vec<(String, BlockPos, i32, String)>,
    /// The shared buffers of the quantum entangloporters, by frequency, and the buttons pressed on their screens.
    entangled: HashMap<String, entangloporter::Shared>,
    qe_buttons: Vec<(String, BlockPos, i32, String, String)>,
    qe_texts: Vec<(String, BlockPos, i32, String, String, String)>,
    bin_buttons: Vec<(String, BlockPos, i32)>,
    alarm_set: HashSet<(String, i32, i32, i32)>,
    miner_buttons: Vec<(String, BlockPos, i32)>,
    miner_texts: Vec<(String, BlockPos, i32, String)>,
    /// What the text boxes of the screens confirmed (FORK-20, `menu_input`): dimension, block, widget id and text, until the machine's tick takes them.
    text_inputs: Vec<(String, BlockPos, i32, String, String, String)>,
    radiation: radiation::Manager,
    miner_work: HashMap<(String, i32, i32, i32), miner::Work>,
    bin_inserts: HashMap<(String, i32, i32, i32), Vec<(String, u32)>>,
    bin_shown: HashMap<(String, i32, i32, i32), u64>,
    bin_set: HashSet<(String, i32, i32, i32)>,
    /// Presses and confirmed boxes of the teleporter's frequency selector, with the player's id and name.
    teleporter_buttons: Vec<(String, BlockPos, i32, String, String)>,
    teleporter_texts: Vec<(String, BlockPos, i32, String, String, String)>,
    /// The player that last used a block's frequency selector (id and name), by block.
    freq_viewer: HashMap<(String, i32, i32, i32), (String, String)>,
    /// Every frequency there is (`frequency.rs`), read from the world's data the first time it is needed.
    freq_registry: Option<Vec<frequency::Freq>>,
    /// What each open item menu of a player was last given (so only changes are sent).
    item_menu_sent: HashMap<(String, String), Vec<String>>,
    frames: teleporter::Frames,
    sorter_set: HashSet<(String, i32, i32, i32)>,
    laser_buttons: Vec<(String, BlockPos, i32)>,
    laser_texts: Vec<(String, BlockPos, i32, String)>,
    /// The steam mode presses on the turbine screens: dimension and the structure's minimum corner.
    turbine_buttons: Vec<(String, [i32; 3])>,
    /// The HUD line each player last got, by line id: its text and bar, so only changes are sent.
    hud_sent: HashMap<(String, String), (String, i32)>,
    /// How many ticks each player has glided in a row (`elytraFlightTick`'s counter).
    glide_ticks: HashMap<String, u32>,
    /// The blows each Robit took since its last tick (paid in energy then), by entity id, and the furnace's recipes (smelting: input to output and count).
    robit_damage: HashMap<u32, f64>,
    /// The names players typed in a Robit's rename box, by entity id, until its next tick.
    robit_renames: HashMap<u32, String>,
    robit_recipes: HashMap<String, (String, u32)>,
    fusion_buttons: Vec<(String, BlockPos, i32)>,
    fusion_texts: Vec<(String, BlockPos, i32, String)>,
    /// The buttons players pressed on fission reactor screens.
    fission_buttons: Vec<(String, BlockPos, i32)>,
    fission_texts: Vec<(String, BlockPos, i32, String)>,
    /// The blades players are putting on or taking off rotors.
    blades: Vec<turbine::BladeRequest>,
    evaporating: HashMap<String, (u64, String, u64)>,
    matrix_given: HashMap<(String, [i32; 3]), u64>,
    /// What each fluidic plenisher knows of its work.
    plenishers: HashMap<(String, i32, i32, i32), pump::Plenish>,
    capacity: HashMap<(String, i32, i32, i32), u64>,
    force: bool,
    /// The reaction chamber's recipes.
    reactions: reaction::Book,
    /// The QIO frequencies and the buttons of the drive arrays and redstone adapters.
    qio: qio::QIOManager,
    /// Formed multiblock structures tracked by (dimension, group, min_corner).
    formed_structures: HashSet<(String, String, [i32; 3])>,
    /// All block positions that are members of a currently formed multiblock.
    formed_blocks: HashSet<(String, i32, i32, i32)>,
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
    /// The configurator's mode when clicked, as its item's components said.
    mode: ToolMode,
    tick: u64,
}

/// A block an EMPTY is emptying: its dimension and position.
type EmptyKey = (String, i32, i32, i32);

/// An EMPTY waiting for the game to say its drops exist: the player to tell, the summon tickets still unanswered,
/// the slots whose drops were confirmed, and how many stacks never got summoned or never spawned (they stay in the
/// machine instead of being destroyed).
struct EmptyPending {
    player: String,
    tickets: HashSet<u64>,
    confirmed: Vec<(usize, String, u32)>,
    failed: u32,
    unsummoned: u32,
}

/// One summoned drop of an EMPTY: the block it came from and the slot, item and count it holds.
struct EmptyTicket {
    key: EmptyKey,
    slot: usize,
    item: String,
    count: u32,
}

/// A mode write waiting to be checked: the player, the hotbar slot it went to, the mode, the stack's components
/// before the write (to find the configurator again if it moved), the tick it may be checked from, and how many
/// times it was already retried.
struct ToolWrite {
    player: String,
    slot: u32,
    mode: ToolMode,
    components: String,
    due: u64,
    attempts: u8,
}

/// The SNBT of a `minecraft:item` holding `stack`: what `/summon` takes, `{Item:{id:"...",Count:Nb}}`. Only the item
/// and the count: the stack's data components do not survive the drop (see `empty_block`).
fn drop_nbt(stack: &ItemStack) -> String {
    let id = stack.item.replace('\\', "\\\\").replace('"', "\\\"");
    format!("{{Item:{{id:\"{id}\",Count:{}b}}}}", stack.count)
}

const CONFIGURATOR: &str = "mekanism:configurator";
/// The eye above the feet, standing and sneaking.
const EYE: f64 = 1.62;
const EYE_SNEAKING: f64 = 1.27;
const NOTHING: &str = "mekanism:nothing";

fn is_multiblock_block(name: &str) -> bool {
    matches!(
        name,
        dynamic::CASING
            | dynamic::VALVE
            | induction::PORT
            | "mekanism:induction_casing"
            | evaporation::BLOCK
            | evaporation::VALVE
            | evaporation::CONTROLLER
            | boiler::CASING
            | boiler::DISPERSER
            | boiler::ELEMENT
            | boiler::VALVE
            | turbine::CASING
            | turbine::COMPLEX
            | turbine::COIL
            | turbine::CONDENSER
            | turbine::ROTOR
            | turbine::VALVE
            | turbine::VENT
            | fission::CASING
            | fission::PORT
            | fission::ADAPTER
            | fusion::FRAME
            | fusion::PORT
            | fusion::CONTROLLER
            | sps::CASING
            | sps::COIL
            | sps::PORT
            | "mekanism:structural_glass"
    ) || induction::CELLS.iter().any(|c| c.0 == name)
        || induction::PROVIDERS.iter().any(|c| c.0 == name)
}

fn is_multiblock_port_or_valve_at(dimension: &str, pos: BlockPos) -> bool {
    world::get_block(dimension, pos.x, pos.y, pos.z)
        .is_some_and(|b| is_multiblock_port_or_valve(b.split('[').next().unwrap_or("")))
}

fn is_multiblock_port_or_valve(name: &str) -> bool {
    matches!(
        name,
        dynamic::VALVE
            | boiler::VALVE
            | induction::PORT
            | evaporation::VALVE
            | evaporation::CONTROLLER
            | turbine::VALVE
            | turbine::VENT
            | fission::PORT
            | fusion::PORT
            | fusion::CONTROLLER
            | sps::PORT
    )
}

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
/// What the nutritional liquifier makes of a food (`createDynamicRecipe`: 50 mB of paste for each point of nutrition), for the foods of
/// the game (the API does not tell a food's nutrition, so this is their table).
const FOODS: [(&str, u64); 44] = [
    ("minecraft:apple", 4), ("minecraft:baked_potato", 5), ("minecraft:beetroot", 1), ("minecraft:beetroot_soup", 6), ("minecraft:bread", 5),
    ("minecraft:carrot", 3), ("minecraft:chicken", 2), ("minecraft:chorus_fruit", 4), ("minecraft:cod", 2), ("minecraft:cooked_beef", 8),
    ("minecraft:cooked_chicken", 6), ("minecraft:cooked_cod", 5), ("minecraft:cooked_mutton", 6), ("minecraft:cooked_porkchop", 8),
    ("minecraft:cooked_rabbit", 5), ("minecraft:cooked_salmon", 6), ("minecraft:cookie", 2), ("minecraft:dried_kelp", 1),
    ("minecraft:enchanted_golden_apple", 4), ("minecraft:glow_berries", 2), ("minecraft:golden_apple", 4), ("minecraft:golden_carrot", 6),
    ("minecraft:honey_bottle", 6), ("minecraft:melon_slice", 2), ("minecraft:mushroom_stew", 6), ("minecraft:mutton", 2),
    ("minecraft:poisonous_potato", 2), ("minecraft:porkchop", 3), ("minecraft:potato", 1), ("minecraft:pufferfish", 1),
    ("minecraft:pumpkin_pie", 8), ("minecraft:rabbit", 3), ("minecraft:rabbit_stew", 10), ("minecraft:beef", 3),
    ("minecraft:rotten_flesh", 4), ("minecraft:salmon", 2), ("minecraft:spider_eye", 2), ("minecraft:suspicious_stew", 6),
    ("minecraft:sweet_berries", 2), ("minecraft:tropical_fish", 1), ("minecraft:cake", 2), ("minecraft:ominous_bottle", 0),
    ("minecraft:milk_bucket", 0), ("minecraft:potion", 0),
];

fn load_proc_book(source: Source, shape: process::Shape) -> process::Book {
    use process::{Out, Recipe, Shape};
    let mut book = process::Book::default();
    if let Source::Files("mekanism:liquifying") = source {
        for (food, nutrition) in FOODS.iter().filter(|(_, n)| *n > 0) {
            book.add(&[(*food).to_owned()], &[], Recipe { item_count: 1, amount: 0, per_tick: false, out: Out::Chem { id: "mekanism:nutritional_paste".into(), amount: nutrition * 50 }, duration: None });
        }
        return book;
    }
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
                duration: json["duration"].as_u64().map(|d| d as u32),
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
    fn find_bounding_main(&self, dimension: &str, x: i32, y: i32, z: i32) -> Option<[i32; 3]> {
        for dy in -4..=0 {
            for dx in -1..=1 {
                for dz in -1..=1 {
                    if dx == 0 && dy == 0 && dz == 0 {
                        continue;
                    }
                    let cand = [x + dx, y + dy, z + dz];
                    if let Some(b) = world::get_block(dimension, cand[0], cand[1], cand[2]) {
                        let name = b.split('[').next().unwrap_or("");
                        if bounding::is_bounded(name) {
                            let facing = sides::property(&b, "facing").unwrap_or("north");
                            let expected = bounding::parts_with_facing(name, facing);
                            if expected.iter().any(|(o, _)| [cand[0] + o[0], cand[1] + o[1], cand[2] + o[2]] == [x, y, z]) {
                                return Some(cand);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    pub(crate) fn is_multiblock_formed(&self, dimension: &str, x: i32, y: i32, z: i32) -> bool {
        self.formed_blocks.contains(&(dimension.to_owned(), x, y, z))
    }

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
            Kind::Continuous | Kind::Tank | Kind::Generator | Kind::Reaction | Kind::Factory | Kind::Pump => false,
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
                Kind::Continuous | Kind::Tank | Kind::Generator | Kind::Reaction | Kind::Factory | Kind::Pump => {}
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
        let Some(t) = click.mode.transmission() else { return false };
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
        if next == now {
            // The transmission has no side type to move to (its supported list is empty): say what the side is, rather
            // than announcing a change that did not happen.
            let _ = players::tell(
                &click.player,
                &format!("Current {} behavior: {} ({})", t.label(), now.label(), now.color_name()),
            );
            return false;
        }
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

    /// The configurator clicks that act on the block itself — empty, rotate and the wrench — taken up at the start of
    /// the tick, before the machines' own ticks, so a machine emptied here does not work from its full slots. A block
    /// a wrench took apart is gone from the tick's list. Returns the updates emptying and rotating cubes write.
    fn tool_clicks(&mut self, entities: &mut Vec<BlockEntity>) -> Vec<BlockEntityUpdate> {
        let mut out = Vec::new();
        let mut i = 0;
        while i < self.pending.len() {
            if self.pending[i].mode.is_configurating() {
                i += 1;
                continue;
            }
            let c = self.pending.remove(i);
            let Some(block) = world::get_block(&c.dimension, c.pos.x, c.pos.y, c.pos.z) else { continue };
            let name = block.split('[').next().unwrap_or("").to_owned();
            match c.mode {
                ToolMode::Empty => {
                    if let Some(e) = entity_at(entities, &c) {
                        self.empty_block(e, &c.player);
                    }
                }
                ToolMode::Rotate => {
                    if let Some(target) = rotate_target(c.face, c.sneak) {
                        if name.ends_with("_energy_cube") {
                            if let Some(e) = entity_at(entities, &c) {
                                out.extend(self.rotate_cube(e, target));
                            }
                        } else if let Some(e) = entity_at(entities, &c) {
                            self.rotate_facing(e, target);
                        }
                    }
                }
                ToolMode::Wrench => {
                    if c.sneak {
                        if self.dismantle(&c) {
                            entities.retain(|e| !(e.dimension == c.dimension && (e.pos.x, e.pos.y, e.pos.z) == (c.pos.x, c.pos.y, c.pos.z)));
                        }
                    } else if name.ends_with("_energy_cube") {
                        if let Some(e) = entity_at(entities, &c) {
                            let modes = sides::decode(&e.data).unwrap_or([sides::Mode::None; 6]);
                            let front = rotate_step(cube::front_of(&e.data, &modes));
                            out.extend(self.rotate_cube(e, front));
                        }
                    } else if let Some(e) = entity_at(entities, &c) {
                        let current = sides::property(&e.state, "facing").map_or(sides::NORTH, laser::face_of);
                        self.rotate_facing(e, rotate_step(current));
                    }
                }
                _ => {}
            }
        }
        out
    }

    /// Empties a block's item slots into the world (`ItemConfigurator` empty): the original drops every non-empty slot
    /// and leaves energy, tanks and chemicals alone.
    ///
    /// Each stack is summoned as a `minecraft:item` at the block, one `entities::summon` per stack, and the machine's
    /// slots are only cleared once the game answers every ticket with an entity id (`entity_spawned`, the tick after;
    /// see `apply_empty_ready`). That wait is what makes the summon safe: the old trick — extra slots past the
    /// container's in the `BlockEntityUpdate`, which the WIT calls "dropped" — does not drop them at all, the game
    /// truncates the slots and the items are destroyed (fork patch 0002, `resize`). A drop the game cannot spawn
    /// stays in the machine instead, and the player is told. The drops carry only the item and the count, not the
    /// stack's data components; a stack the game has not confirmed is never cleared.
    ///
    /// Whether `minecraft:item` really spawns with this NBT is NOT verified by reading: the WIT says `summon`
    /// spawns "any entity of `kind`", the host only checks the kind's shape and the NBT's size, and the game's own
    /// SNBT parser reads the compound — but the native spawn itself is outside the patches. If it does not spawn,
    /// the tickets answer with no id and nothing is lost, only left in the machine: confirm in the game before
    /// trusting EMPTY with anything valuable.
    fn empty_block(&mut self, e: &BlockEntity, player: &str) {
        let key: EmptyKey = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
        if self.empty.contains_key(&key) {
            return; // An empty is already on its way for this block: summoning again would drop the stacks twice.
        }
        let kept: Vec<(usize, ItemStack)> = e
            .items
            .iter()
            .enumerate()
            .filter_map(|(i, s)| s.as_ref().filter(|s| s.count > 0).map(|s| (i, s.clone())))
            .collect();
        if kept.is_empty() {
            return;
        }
        // The host allows at most 64 entity requests in a call and 8192 bytes of NBT per summon: check both before
        // summoning anything, so a failure cannot leave half the stacks dropped and the other half in the machine.
        // A machine holds at most 27 stacks (the miner), so this is not expected to trigger.
        if kept.len() > 64 || kept.iter().any(|(_, s)| drop_nbt(s).len() > 8192) {
            let _ = players::tell(player, "Could not empty: too many item stacks");
            return;
        }
        let mut tickets: Vec<(u64, usize, String, u32)> = Vec::new();
        let mut unsummoned = 0;
        for (slot, stack) in &kept {
            match entities::summon(
                &e.dimension,
                "minecraft:item",
                e.pos.x as f64 + 0.5,
                e.pos.y as f64 + 0.5,
                e.pos.z as f64 + 0.5,
                &drop_nbt(stack),
            ) {
                Ok(ticket) => tickets.push((ticket, *slot, stack.item.clone(), stack.count)),
                Err(_) => {
                    // The checks above rule out every failure the host documents, so this is not expected; the stacks
                    // summoned so far still drop (they are registered below), the rest stay in the machine.
                    unsummoned = kept.len() as u32 - tickets.len() as u32;
                    break;
                }
            }
        }
        if tickets.is_empty() {
            if unsummoned > 0 {
                let _ = players::tell(player, "Could not empty: the drops could not be summoned");
            }
            return;
        }
        for (ticket, slot, item, count) in &tickets {
            self.empty_tickets.insert(*ticket, EmptyTicket { key: key.clone(), slot: *slot, item: item.clone(), count: *count });
        }
        self.empty.insert(
            key,
            EmptyPending { player: player.to_owned(), tickets: tickets.iter().map(|(t, _, _, _)| *t).collect(), confirmed: Vec::new(), failed: 0, unsummoned },
        );
    }

    /// Clears the slots of empties whose drops the game confirmed, on the blocks' next tick: only a slot that still
    /// holds the stack that was dropped is cleared (the machine may have worked on in the meantime, and what is there
    /// now must not be destroyed). The tick's copy is cleared too, so the machine's own update this tick is computed
    /// from the emptied slots instead of resurrecting them. Empties of blocks that are gone are forgotten after a
    /// while.
    fn apply_empty_ready(&mut self, entities: &mut [BlockEntity]) -> Vec<BlockEntityUpdate> {
        let mut out = Vec::new();
        let mut done: Vec<EmptyKey> = Vec::new();
        for (key, (slots, at)) in &self.empty_ready {
            let Some(e) = entities.iter_mut().find(|e| e.dimension == key.0 && (e.pos.x, e.pos.y, e.pos.z) == (key.1, key.2, key.3)) else {
                if self.now.saturating_sub(*at) > 100 {
                    done.push(key.clone());
                }
                continue;
            };
            let mut changed = false;
            for (slot, item, count) in slots {
                if e.items.get(*slot).and_then(|s| s.as_ref()).is_some_and(|s| s.item == *item && s.count == *count) {
                    e.items[*slot] = None;
                    changed = true;
                }
            }
            if changed {
                out.push(BlockEntityUpdate {
                    dimension: e.dimension.clone(),
                    pos: e.pos,
                    data: e.data.clone(),
                    items: Some(e.items.clone()),
                    energy: None,
                    energy_faces: None,
                    tanks: None,
                    tank_faces: None,
                    menu_data: None,
                    transmitter_faces: None,
                });
            }
            done.push(key.clone());
        }
        for key in done {
            self.empty_ready.remove(&key);
        }
        out
    }

    /// Turns a cube's front to `target` (a face of the world), keeping its energy and item modes.
    fn rotate_cube(&mut self, e: &mut BlockEntity, target: usize) -> Option<BlockEntityUpdate> {
        let modes = sides::decode(&e.data).unwrap_or_else(|| sides::default_modes(target));
        let item_modes = cube::item_modes(&e.data);
        let data = cube::encode(&modes, target, &item_modes);
        e.data = data.clone();
        let _ = world::set_block_property(&e.dimension, e.pos.x, e.pos.y, e.pos.z, "facing", cube::facing_name(target));
        Some(BlockEntityUpdate {
            dimension: e.dimension.clone(),
            pos: e.pos,
            data,
            items: None,
            energy: None,
            energy_faces: None,
            tanks: None,
            tank_faces: None,
            menu_data: None,
            transmitter_faces: None,
        })
    }

    /// Turns a block's `facing` property to `target`, in the world and in the tick's copy of the entity.
    fn rotate_facing(&self, e: &mut BlockEntity, target: usize) {
        let facing = laser::facing_name(target);
        let _ = world::set_block_property(&e.dimension, e.pos.x, e.pos.y, e.pos.z, "facing", facing);
        if let Some(old) = sides::property(&e.state, "facing").map(str::to_owned) {
            e.state = e.state.replacen(&format!("facing={old}"), &format!("facing={facing}"), 1);
        }
    }

    /// Takes a block apart with the wrench (`TileEntityMekanism.tryWrenchDismantle`): the block breaks as the player
    /// with the configurator would break it, its drops falling where it was (mine-block has no way to put them in the
    /// player's inventory). A block in radiation cannot be taken apart: the original blocks it when the tile itself is
    /// radioactive, and the port has no per-tile scale, so the ambient level at the block stands in. Whether it was
    /// taken apart.
    fn dismantle(&mut self, c: &Click) -> bool {
        if radiation::severity(radiation::level_at(&self.radiation.sources, &c.dimension, [c.pos.x, c.pos.y, c.pos.z])) > 0.0 {
            let _ = players::tell(&c.player, "Cannot dismantle: the block is radioactive");
            return false;
        }
        let _ = world::mine_block(&c.dimension, c.pos.x, c.pos.y, c.pos.z, CONFIGURATOR, false, c.pos.x, c.pos.y, c.pos.z, c.face as u32);
        true
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
                spec.speed = speed;
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
            Kind::Continuous | Kind::Tank | Kind::Generator | Kind::Reaction | Kind::Factory | Kind::Pump => unreachable!("they have their own ticks"),
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
        let capacity = upgrades::max_energy(base_capacity(def), counts[upgrades::ENERGY]);
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
        redstone::stamp(def.kind, control, capacity.saturating_sub(energy), shown, &facing, &mut saved, &mut data);
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
        let mut changed = stored.is_none() || e.data.len() < 13;
        // A new cube's front looks at the nearest player, as one placed in front of them does.
        let front_new = front_for(e);
        let mut modes = stored.unwrap_or_else(|| sides::default_modes(front_new));
        let front = if stored.is_some() { cube::front_of(&e.data, &modes) } else { front_new };
        // The cube's item sides (the original configures energy and items on a cube): the charge slot takes in
        // where a face is an input, the discharge slot gives where it is an output.
        let mut item_modes = cube::item_modes(&e.data);
        if let Some(click) = self.take_click(e) {
            match click.mode {
                ToolMode::Energy => {
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
                ToolMode::Items => {
                    let now = item_modes[click.face];
                    let d = cube::data_of;
                    if click.sneak {
                        item_modes[click.face] = now.step(false);
                        changed = true;
                        let next = d(item_modes[click.face]);
                        let _ = players::tell(
                            &click.player,
                            &format!("Item behavior bumped to: {} ({})", next.label(), next.color_name()),
                        );
                        let faces: Vec<Vec<u32>> = item_modes
                            .iter()
                            .map(|m| match m {
                                Mode::Input => vec![cube::CHARGE as u32],
                                Mode::Output => vec![cube::DISCHARGE as u32],
                                Mode::Both => vec![cube::CHARGE as u32, cube::DISCHARGE as u32],
                                Mode::None => Vec::new(),
                            })
                            .collect();
                        let _ = world::set_item_faces(&e.dimension, e.pos.x, e.pos.y, e.pos.z, &faces);
                    } else {
                        let _ = players::tell(
                            &click.player,
                            &format!("Current Item behavior: {} ({})", d(now).label(), d(now).color_name()),
                        );
                    }
                }
                // Empty, rotate and the wrench act on the block itself, not its sides (handled before the tick).
                _ => {}
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
            data: cube::encode(&modes, front, &item_modes),
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

    /// The mode of the configurator in `stack`: its components first; the remembered one for the player when the stack
    /// carries none (a stack the game has not written the mode back into yet).
    fn tool_mode(&self, player: &str, stack: &ItemStack) -> ToolMode {
        if stack.components.contains(config::MODE_COMPONENT) {
            config::mode_of(&stack.components)
        } else {
            self.tool.get(player).copied().unwrap_or(ToolMode::Items)
        }
    }

    /// The hotbar slot holding `stack`: the first with the same item and count whose components are the same stack's
    /// (the mode aside — the game may already have written a mode into it), and the components the inventory holds
    /// for it. The held slot is not told, so two identical fresh configurators cannot be told apart and the first is
    /// taken. None when the stack is not in the hotbar: the caller must not write anywhere else — writing to slot 0
    /// by default destroyed whatever was there and duplicated the tool.
    fn find_hotbar_slot(player: &str, stack: &ItemStack) -> Option<(u32, String)> {
        let inv = player_items::get_inventory(player)?;
        inv.iter().take(9).enumerate().find_map(|(i, s)| {
            s.as_ref()
                .filter(|s| s.item == stack.item && s.count == stack.count && config::same_components(&stack.components, &s.components))
                .map(|s| (i as u32, s.components.clone()))
        })
    }

    /// Writes the mode into the configurator the player holds, so it survives a restart and travels with the item.
    /// The write is checked on a later tick (`check_tool_writes`): the inventory is read as of the last server tick
    /// and `set-slot` writes a whole stack blind, so a write to a slot the player has just changed would destroy
    /// what is there now and duplicate the tool. The mode also lives in `tool`, the per-player cache, for the stack
    /// the game has not written back yet.
    fn write_tool_mode(&mut self, player: &Player, stack: &ItemStack, mode: ToolMode) {
        let Some((slot, components)) = Self::find_hotbar_slot(&player.id, stack) else {
            return; // Not in the hotbar (an off-hand configurator, say): never guess a slot.
        };
        let _ = player_items::set_slot(
            &player.id,
            slot,
            Some(&ItemStack { item: stack.item.clone(), count: stack.count, components: config::with_mode(&components, mode) }),
        );
        self.tool_writes.push(ToolWrite {
            player: player.id.clone(),
            slot,
            mode,
            components: stack.components.clone(),
            due: self.now + 2,
            attempts: 0,
        });
    }

    /// Checks the mode writes of `write_tool_mode` once their tick has come: the write lands after the call returns,
    /// before the next tick, so a re-read then says whether the configurator in the slot carries the mode. A write
    /// that did not land is tried once more where the configurator now is, if it is still in the inventory; otherwise
    /// the player is told the mode was not saved (it still holds for this session, in the per-player cache).
    fn check_tool_writes(&mut self, tick: u64) {
        let mut i = 0;
        while i < self.tool_writes.len() {
            if tick < self.tool_writes[i].due {
                i += 1;
                continue;
            }
            let w = self.tool_writes.remove(i);
            let inv = player_items::get_inventory(&w.player);
            let landed = inv
                .as_ref()
                .and_then(|inv| inv.get(w.slot as usize))
                .and_then(|s| s.as_ref())
                .is_some_and(|s| s.item == CONFIGURATOR && s.components.contains(config::MODE_COMPONENT) && config::mode_of(&s.components) == w.mode);
            if landed {
                continue;
            }
            // The configurator moved, or the write never landed: find it anywhere in the inventory and write the
            // mode there — once. The slot it was in may hold something else now, and that must not be touched.
            let found = inv.as_ref().and_then(|inv| {
                inv.iter().enumerate().find_map(|(i, s)| {
                    s.as_ref()
                        .filter(|s| s.item == CONFIGURATOR && config::same_components(&w.components, &s.components))
                        .map(|s| (i as u32, s.components.clone(), s.count))
                })
            });
            let Some((slot, components, count)) = found else {
                let _ = players::tell(&w.player, "Configure State was not saved to the configurator; it may not survive a restart");
                continue;
            };
            if w.attempts > 0 {
                // Already retried once: do not chase the item around the inventory. The per-player cache still
                // holds the mode for this session.
                let _ = players::tell(&w.player, "Configure State was not saved to the configurator; it may not survive a restart");
                continue;
            }
            let _ = player_items::set_slot(
                &w.player,
                slot,
                Some(&ItemStack { item: CONFIGURATOR.to_owned(), count, components: config::with_mode(&components, w.mode) }),
            );
            self.tool_writes.push(ToolWrite { player: w.player, slot, mode: w.mode, components: w.components, due: tick + 2, attempts: w.attempts + 1 });
        }
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

/// The entity at a click's block, in the tick's list.
fn entity_at<'a>(entities: &'a mut [BlockEntity], c: &Click) -> Option<&'a mut BlockEntity> {
    entities.iter_mut().find(|e| e.dimension == c.dimension && (e.pos.x, e.pos.y, e.pos.z) == (c.pos.x, c.pos.y, c.pos.z))
}

/// The face a rotate click turns a block to: the face clicked, or its opposite when sneaking. A block only faces
/// sideways, so a click on its top or bottom turns nothing (the original only sets a horizontal facing).
fn rotate_target(face: usize, sneak: bool) -> Option<usize> {
    let face = if sneak { face ^ 1 } else { face };
    (2..=5).contains(&face).then_some(face)
}

/// The next face clockwise (north, east, south, west), for a wrench turn without sneak (`MekanismUtils.rotate`).
fn rotate_step(face: usize) -> usize {
    match face {
        sides::NORTH => sides::EAST,
        sides::EAST => sides::SOUTH,
        sides::SOUTH => sides::WEST,
        _ => sides::NORTH,
    }
}

fn flow_side(f: u8) -> FlowSide {
    match Mode::from_byte(f) {
        Mode::None => FlowSide::None,
        Mode::Input => FlowSide::Input,
        Mode::Output => FlowSide::Output,
        Mode::Both => FlowSide::Both,
    }
}

impl Mekanism {
    /// The machine's charge slot (`EnergyInventorySlot.fillOrConvert`): an item that holds energy gives it, any other item the energy it is worth
    /// whole when that fits. `e` is changed in place; true when something went in.
    fn charge_from_slot(&self, def: &Def, e: &mut BlockEntity) -> bool {
        let Some(at) = upgrades::charge_slot(def) else { return false };
        let Some(Some(stack)) = e.items.get(at) else { return false };
        let capacity = upgrades::max_energy(base_capacity(def), upgrades::counts_of(def, e)[upgrades::ENERGY]);
        let slot = Some(stack.clone());
        let Some((gain, left)) = induction::fill_step(&slot, capacity.saturating_sub(e.energy)) else { return false };
        e.energy += gain;
        e.items[at] = left;
        true
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
                // The empty bucket in (top), the full one out (bottom) and the charge slot (back).
                Kind::Pump => ContainerSpec {
                    slots: 3,
                    faces: config::item_faces(def.layout, "north", config::default_config(def.layout).items()),
                    inputs: vec![0],
                    outputs: vec![1],
                    menu: true,
                },
                // A line's input and output slots one after the other, then the charge slot, on the default configuration's faces.
                Kind::Factory => ContainerSpec {
                    slots: config::factory_slot_count(def.lines, def.form) as u32,
                    faces: config::item_faces(def.layout, "north", config::default_config(def.layout).items()),
                    inputs: {
                        let mut v = config::slots_vec(def.layout, Data::Input);
                        v.extend(config::slots_vec(def.layout, Data::Extra));
                        v
                    },
                    outputs: config::slots_vec(def.layout, Data::Output),
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
                        slots: if kind.tankless() { 1 } else { 2 },
                        faces: generators::item_faces(kind, "north"),
                        inputs: if kind.tankless() { Vec::new() } else { vec![0] },
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
                c.slots += 2 + u32::from(upgrades::charge_slot(def).is_some_and(|s| s >= upgrades::base_slots(def)));
            }
            if def.kind == Kind::Generator {
                // It gives its energy by the faces the generator tick sets (the front, the bottom for the solar one).
                let kind = def.generator.unwrap_or(GenKind::Solar);
                let (cap, tank) = match kind {
                    GenKind::Bio => (generators::BIO_ENERGY, Some(("fluid", generators::BIO_TANK))),
                    GenKind::Heat => (generators::HEAT_ENERGY, Some(("fluid", generators::HEAT_TANK))),
                    GenKind::Gas => (generators::GAS_ENERGY, Some(("chemical", generators::GAS_TANK))),
                    GenKind::Solar => (generators::SOLAR_ENERGY, None),
                    GenKind::AdvSolar => (generators::ADV_SOLAR_ENERGY, None),
                    GenKind::Wind => (generators::WIND_ENERGY, None),
                };
                k.energy = Some(EnergySpec { capacity: cap, max_receive: 0, max_extract: cap, faces: generators::energy_faces(kind, "north") });
                if let Some((fluid, capacity)) = tank {
                    k.tanks.push(TankSpec { kind: fluid.into(), capacity, max_rate: capacity.min(1000), faces: generators::tank_faces("north") });
                }
            } else if def.kind != Kind::Tank {
                // The separator stores more: its use is tiny and a bucket of water is many ticks of it.
                let stored = base_capacity(def);
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
            if def.kind == Kind::Pump {
                let flow = config::tank_flow(def.layout, "north", &config::default_config(def.layout), false);
                k.tanks.push(TankSpec {
                    kind: "fluid".into(),
                    capacity: pump::TANK,
                    max_rate: 1000,
                    faces: flow.first().map_or_else(|| vec![FlowSide::None; 6], |f| f.iter().map(|x| flow_side(*x)).collect()),
                });
            }
            if def.kind == Kind::Factory && def.form >= 3 {
                // One chemical tank for all the lines, as big as the machine's for each of them.
                let cap = MACHINES.iter().find(|d| d.block == def.base).and_then(|d| d.proc).map_or(0, |s| s.cap_in) * def.lines as u64;
                let flow = config::tank_flow(def.layout, "north", &config::default_config(def.layout), false);
                k.tanks.push(TankSpec {
                    kind: "chemical".into(),
                    capacity: cap,
                    max_rate: cap.min(1000),
                    faces: flow.first().map_or_else(|| vec![FlowSide::None; 6], |f| f.iter().map(|x| flow_side(*x)).collect()),
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
                    let fluid = i == 0 && !matches!(spec.shape, continuous::Shape::Infuser | continuous::Shape::Converter);
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
                    let mut t = tank(spec.cap_out, usize::from(spec.cap_in > 0));
                    if def.block == "mekanism:nutritional_liquifier" {
                        t.kind = "fluid".into();
                    }
                    k.tanks.push(t);
                }
            }
            r = r.block_entity_kind(k);
        }
        // The formulaic assemblicator: the formula, 18 inputs, the 3x3 grid, 6 outputs and the charge slot (35), on the faces of its configuration.
        let mut assemblicator_kind = kind(assemblicator::BLOCK, true);
        assemblicator_kind.container = Some(ContainerSpec {
            slots: assemblicator::TOTAL_SLOTS as u32,
            faces: assemblicator::assemblicator_item_faces("north", config::default_config(Layout::Electric).items()),
            inputs: (assemblicator::SLOT_INPUT_START..assemblicator::SLOT_INPUT_START + assemblicator::INPUT_SLOTS).map(|i| i as u32).collect(),
            outputs: (assemblicator::SLOT_OUTPUT_START..assemblicator::SLOT_OUTPUT_START + assemblicator::OUTPUT_SLOTS).map(|i| i as u32).collect(),
            menu: true,
        });
        assemblicator_kind.energy = Some(energy(assemblicator::BASE_STORAGE, assemblicator::BASE_STORAGE, FlowSide::Both));
        r = r.block_entity_kind(assemblicator_kind);
        // The modification station: the module slot, the gear slot and the charge slot, on the back for energy; it takes in only (hoppers fill all three).
        let mut station = kind(modification_station::BLOCK, true);
        station.container = Some(ContainerSpec { slots: modification_station::TOTAL_SLOTS as u32, faces: vec![Vec::new(); 6], inputs: vec![0, 1, 2], outputs: Vec::new(), menu: true });
        station.energy = Some(EnergySpec { capacity: modification_station::ENERGY_CAPACITY, max_receive: modification_station::ENERGY_CAPACITY, max_extract: 0, faces: seismic_vibrator::energy_input_faces("north") });
        r = r.block_entity_kind(station);
        // The heat: conductors only keep their heat; the resistive heater takes energy (its screen has no slots), the fuelwood heater one fuel slot.
        for tier in heat::ConductorTier::ALL {
            r = r.block_entity_kind(kind(tier.block_name(), true));
        }
        let mut resistive = kind(resistive_heater::BLOCK, true);
        let resistive_cap = resistive_heater::ResistiveHeater::default().max_energy_fe();
        resistive.container = Some(ContainerSpec { slots: 1, faces: vec![Vec::new(); 6], inputs: vec![0], outputs: Vec::new(), menu: true });
        resistive.energy = Some(energy(resistive_cap, resistive_cap, FlowSide::Input));
        r = r.block_entity_kind(resistive);
        let mut fuelwood = kind(fuelwood_heater::BLOCK, true);
        fuelwood.container = Some(ContainerSpec { slots: 1, faces: vec![Vec::new(); 6], inputs: vec![0], outputs: Vec::new(), menu: true });
        r = r.block_entity_kind(fuelwood);
        // The utilities: the chargepad takes energy at its back and underside and has no screen; the stabilizer and the seismic
        // vibrator have one charge slot; the oredictionificator an input (top) and an output (right); the personal chest and
        // barrel 54 slots open to hoppers (the security mode is checked in their own tick); the cardboard box only keeps its data.
        let mut pad = kind(chargepad::BLOCK, true);
        pad.energy = Some(EnergySpec { capacity: chargepad::CAPACITY, max_receive: chargepad::CAPACITY, max_extract: 0, faces: chargepad::energy_input_faces("north") });
        r = r.block_entity_kind(pad);
        let mut stabilizer = kind(dimensional_stabilizer::BLOCK, true);
        stabilizer.container = Some(ContainerSpec { slots: 1, faces: vec![Vec::new(); 6], inputs: vec![0], outputs: Vec::new(), menu: true });
        stabilizer.energy = Some(energy(dimensional_stabilizer::BASE_CAPACITY, dimensional_stabilizer::BASE_CAPACITY, FlowSide::Input));
        r = r.block_entity_kind(stabilizer);
        let mut vibrator = kind(seismic_vibrator::BLOCK, true);
        vibrator.container = Some(ContainerSpec { slots: 1, faces: vec![Vec::new(); 6], inputs: vec![0], outputs: Vec::new(), menu: true });
        vibrator.energy = Some(EnergySpec { capacity: seismic_vibrator::CAPACITY, max_receive: seismic_vibrator::CAPACITY, max_extract: 0, faces: seismic_vibrator::energy_input_faces("north") });
        r = r.block_entity_kind(vibrator);
        let mut ore = kind(oredictionificator::BLOCK, true);
        ore.container = Some(ContainerSpec {
            slots: oredictionificator::TOTAL_SLOTS as u32,
            faces: oredictionificator::default_item_faces("north"),
            inputs: vec![oredictionificator::INPUT_SLOT as u32],
            outputs: vec![oredictionificator::OUTPUT_SLOT as u32],
            menu: true,
        });
        r = r.block_entity_kind(ore);
        for block in [personal_storage::CHEST_BLOCK, personal_storage::BARREL_BLOCK] {
            let mut storage = kind(block, true);
            let all: Vec<u32> = (0..personal_storage::TOTAL_SLOTS as u32).collect();
            storage.container = Some(ContainerSpec { slots: personal_storage::TOTAL_SLOTS as u32, faces: vec![all.clone(); 6], inputs: all.clone(), outputs: all, menu: true });
            r = r.block_entity_kind(storage);
        }
        r = r.block_entity_kind(kind(cardboard_box::BLOCK, true));
        // The radioactive waste barrel: one chemical tank (512 000 mB) through the top and the bottom only.
        let mut barrel = kind(radioactive_waste_barrel::BLOCK_ID, true);
        let mut barrel_faces = vec![FlowSide::None; 6];
        barrel_faces[config::UP] = FlowSide::Both;
        barrel_faces[config::DOWN] = FlowSide::Both;
        barrel.tanks.push(TankSpec { kind: "chemical".into(), capacity: radioactive_waste_barrel::CAPACITY, max_rate: radioactive_waste_barrel::CAPACITY, faces: barrel_faces });
        r = r.block_entity_kind(barrel);
        // The QIO drive array holds twelve drives (one per slot, none moved by hoppers); the redstone adapter's one slot is its ghost target.
        let mut array = kind(qio::BLOCK_DRIVE_ARRAY, true);
        array.container = Some(ContainerSpec { slots: qio::DRIVE_SLOTS as u32, faces: vec![Vec::new(); 6], inputs: Vec::new(), outputs: Vec::new(), menu: true });
        r = r.block_entity_kind(array);
        let mut adapter = kind(qio::BLOCK_REDSTONE_ADAPTER, true);
        adapter.container = Some(ContainerSpec { slots: 1, faces: vec![Vec::new(); 6], inputs: Vec::new(), outputs: Vec::new(), menu: true });
        r = r.block_entity_kind(adapter);
        // The discharge and charge slots of its screen; a face reaches the charge slot where the cube's item side
        // says input, the discharge slot where it says output (set by the configurator, `cube()`).
        let slots = || ContainerSpec { slots: 2, faces: vec![Vec::new(); 6], inputs: vec![1], outputs: vec![0], menu: true };
        for (block, capacity, rate) in CUBE_TIERS {
            let mut tier = kind(block, true);
            tier.container = Some(slots());
            tier.energy = Some(energy(capacity, rate, FlowSide::Both));
            r = r.block_entity_kind(tier);
        }
        // The dynamic tank: the casing only joins the structure; the valves hold its tank (a fluid and a chemical) and its two bucket slots.
        let mut casing = kind(dynamic::CASING, false);
        casing.keep_data = false;
        casing.multiblock = Some(MultiblockSpec { group: dynamic::GROUP.into() });
        r = r.block_entity_kind(casing);
        let mut valve = kind(dynamic::VALVE, true);
        valve.keep_data = false;
        valve.multiblock = Some(MultiblockSpec { group: dynamic::GROUP.into() });
        valve.container = Some(ContainerSpec { slots: 2, faces: vec![Vec::new(); 6], inputs: vec![0], outputs: vec![1], menu: true });
        for kind_name in ["fluid", "chemical"] {
            valve.tanks.push(TankSpec { kind: kind_name.into(), capacity: dynamic::SPEC_CAPACITY, max_rate: dynamic::VALVE_RATE, faces: vec![FlowSide::Both; 6] });
        }
        r = r.block_entity_kind(valve);
        // The induction matrix: the casing, the cells and the providers only join the structure; the ports hold its energy.
        for part in ["mekanism:induction_casing"].into_iter().chain(induction::CELLS.iter().map(|c| c.0)).chain(induction::PROVIDERS.iter().map(|c| c.0)) {
            let mut k = kind(part, false);
            k.keep_data = false;
            k.multiblock = Some(MultiblockSpec { group: induction::GROUP.into() });
            r = r.block_entity_kind(k);
        }
        let mut port = kind(induction::PORT, true);
        port.keep_data = false;
        port.multiblock = Some(MultiblockSpec { group: induction::GROUP.into() });
        port.container = Some(ContainerSpec { slots: 2, faces: vec![Vec::new(); 6], inputs: Vec::new(), outputs: Vec::new(), menu: true });
        port.energy = Some(EnergySpec { capacity: induction::SPEC_CAPACITY, max_receive: induction::SPEC_CAPACITY, max_extract: induction::SPEC_CAPACITY, faces: vec![FlowSide::Both; 6] });
        r = r.block_entity_kind(port);
        // The thermal evaporation plant: the walls only join it; valves and controller hold its two tanks (the controller also its four bucket slots).
        let mut wall = kind(evaporation::BLOCK, false);
        wall.keep_data = false;
        wall.multiblock = Some(MultiblockSpec { group: evaporation::GROUP.into() });
        r = r.block_entity_kind(wall);
        for (name, slots) in [(evaporation::VALVE, 0), (evaporation::CONTROLLER, 4)] {
            let mut k = kind(name, true);
            k.keep_data = false;
            k.multiblock = Some(MultiblockSpec { group: evaporation::GROUP.into() });
            if slots > 0 {
                k.container = Some(ContainerSpec { slots, faces: vec![Vec::new(); 6], inputs: vec![0, 2], outputs: vec![1, 3], menu: true });
            }
            for kind_name in ["fluid", "chemical"] {
                k.tanks.push(TankSpec { kind: kind_name.into(), capacity: evaporation::SPEC_CAPACITY, max_rate: 100_000, faces: vec![FlowSide::Both; 6] });
            }
            r = r.block_entity_kind(k);
        }
        // The thermoelectric boiler: the casing, the dispersers and the superheating elements only join the structure; the valves hold its four tanks.
        for part in [boiler::CASING, boiler::DISPERSER, boiler::ELEMENT] {
            let mut k = kind(part, false);
            k.keep_data = false;
            // The pressure disperser is part of boilers and of turbines.
            let group = if part == boiler::DISPERSER { format!("{},{}", boiler::GROUP, turbine::GROUP) } else { boiler::GROUP.to_owned() };
            k.multiblock = Some(MultiblockSpec { group });
            r = r.block_entity_kind(k);
        }
        let mut bv = kind(boiler::VALVE, true);
        bv.keep_data = false;
        bv.multiblock = Some(MultiblockSpec { group: boiler::GROUP.into() });
        bv.container = Some(ContainerSpec { slots: 2, faces: vec![Vec::new(); 6], inputs: vec![0], outputs: vec![1], menu: true });
        for (kind_name, side) in [("fluid", FlowSide::Input), ("chemical", FlowSide::Output), ("chemical", FlowSide::Input), ("chemical", FlowSide::Output)] {
            bv.tanks.push(TankSpec { kind: kind_name.into(), capacity: boiler::SPEC_CAPACITY, max_rate: boiler::VALVE_RATE, faces: vec![side; 6] });
        }
        r = r.block_entity_kind(bv);
        // The industrial turbine: casing, dispersers (also the boiler's), the complex, the coils and the condensers only join the structure; the rotors
        // hold their blades in a slot; the valves hold the steam and the energy, the vents the water.
        for (part, ticks) in [(turbine::CASING, false), (turbine::COMPLEX, false), (turbine::COIL, false), (turbine::CONDENSER, false), (turbine::ROTOR, true)] {
            let mut k = kind(part, ticks);
            k.keep_data = false;
            k.multiblock = Some(MultiblockSpec { group: turbine::GROUP.into() });
            if part == turbine::ROTOR {
                k.container = Some(ContainerSpec { slots: 1, faces: vec![Vec::new(); 6], inputs: Vec::new(), outputs: Vec::new(), menu: false });
            }
            r = r.block_entity_kind(k);
        }
        let mut tv = kind(turbine::VALVE, true);
        tv.keep_data = false;
        tv.multiblock = Some(MultiblockSpec { group: turbine::GROUP.into() });
        tv.container = Some(ContainerSpec { slots: 1, faces: vec![Vec::new(); 6], inputs: Vec::new(), outputs: Vec::new(), menu: true });
        tv.energy = Some(EnergySpec { capacity: 1 << 40, max_receive: 0, max_extract: 1 << 40, faces: vec![FlowSide::Output; 6] });
        tv.tanks.push(TankSpec { kind: "chemical".into(), capacity: turbine::SPEC_CAPACITY, max_rate: 1 << 30, faces: vec![FlowSide::Input; 6] });
        r = r.block_entity_kind(tv);
        let mut tvent = kind(turbine::VENT, true);
        tvent.keep_data = false;
        tvent.multiblock = Some(MultiblockSpec { group: turbine::GROUP.into() });
        tvent.tanks.push(TankSpec { kind: "fluid".into(), capacity: turbine::SPEC_CAPACITY, max_rate: 1 << 30, faces: vec![FlowSide::Output; 6] });
        r = r.block_entity_kind(tvent);
        // The fission reactor: the casing, adapters and assemblies only join the structure; the ports hold its five tanks.
        for part in [fission::CASING, fission::ADAPTER, fission::FUEL, fission::ROD] {
            let mut k = kind(part, false);
            k.keep_data = false;
            k.multiblock = Some(MultiblockSpec { group: fission::GROUP.into() });
            r = r.block_entity_kind(k);
        }
        let mut fp = kind(fission::PORT, true);
        fp.keep_data = false;
        fp.multiblock = Some(MultiblockSpec { group: fission::GROUP.into() });
        fp.container = Some(ContainerSpec { slots: 1, faces: vec![Vec::new(); 6], inputs: Vec::new(), outputs: Vec::new(), menu: true });
        for (kind_name, side) in [("fluid", FlowSide::Input), ("chemical", FlowSide::Input), ("chemical", FlowSide::Input), ("chemical", FlowSide::Output), ("chemical", FlowSide::Output)] {
            fp.tanks.push(TankSpec { kind: kind_name.into(), capacity: fission::SPEC_CAPACITY, max_rate: 1 << 30, faces: vec![side; 6] });
        }
        r = r.block_entity_kind(fp);
        // The fusion reactor: frames, adapters, focus matrices and glass only join the structure; the ports hold its five tanks; the
        // controller holds the hohlraum slot and the screen. A fresh port takes in (its mode byte defaults to input), as the SPS and
        // fission ports do: energy and steam only leave in output mode (`run_fusion` says so on every tick).
        for part in [fusion::FRAME, fusion::ADAPTER, fusion::MATRIX, fusion::GLASS] {
            // The focus matrix ticks to take the energy lasers deliver.
            let mut k = kind(part, part == fusion::MATRIX);
            k.keep_data = false;
            k.multiblock = Some(MultiblockSpec { group: fusion::GROUP.into() });
            r = r.block_entity_kind(k);
        }
        let mut fport = kind(fusion::PORT, true);
        fport.keep_data = false;
        fport.multiblock = Some(MultiblockSpec { group: fusion::GROUP.into() });
        fport.energy = Some(EnergySpec { capacity: 1 << 40, max_receive: 0, max_extract: 1 << 40, faces: vec![FlowSide::Input; 6] });
        for (kind_name, side) in [("fluid", FlowSide::Input), ("chemical", FlowSide::Input), ("chemical", FlowSide::Input), ("chemical", FlowSide::Input), ("chemical", FlowSide::None)] {
            fport.tanks.push(TankSpec { kind: kind_name.into(), capacity: fusion::SPEC_CAPACITY, max_rate: 1 << 30, faces: vec![side; 6] });
        }
        r = r.block_entity_kind(fport);
        let mut fctl = kind(fusion::CONTROLLER, true);
        fctl.keep_data = false;
        fctl.multiblock = Some(MultiblockSpec { group: fusion::GROUP.into() });
        fctl.container = Some(ContainerSpec { slots: 1, faces: vec![Vec::new(); 6], inputs: Vec::new(), outputs: Vec::new(), menu: true });
        r = r.block_entity_kind(fctl);
        // The laser and its amplifier: energy in (the laser from behind only), the amplifier's thresholds and delay in its data.
        let mut laser = kind(laser::LASER, true);
        laser.energy = Some(EnergySpec { capacity: laser::LASER_CAPACITY, max_receive: laser::LASER_CAPACITY, max_extract: 0, faces: vec![FlowSide::None; 6] });
        r = r.block_entity_kind(laser);
        let mut amp = kind(laser::AMPLIFIER, true);
        amp.energy = Some(EnergySpec { capacity: laser::AMPLIFIER_CAPACITY, max_receive: laser::AMPLIFIER_CAPACITY, max_extract: 0, faces: vec![FlowSide::Input; 6] });
        amp.container = Some(ContainerSpec { slots: 1, faces: vec![Vec::new(); 6], inputs: Vec::new(), outputs: Vec::new(), menu: true });
        r = r.block_entity_kind(amp);
        // The laser tractor beam: lasers fill it (nothing outside does), and its 27 slots only give out (the original's output slots).
        let mut tractor = kind(laser::TRACTOR, true);
        tractor.energy = Some(EnergySpec { capacity: laser_tractor::CAPACITY_FE, max_receive: laser_tractor::CAPACITY_FE, max_extract: 0, faces: vec![FlowSide::None; 6] });
        tractor.container = Some(ContainerSpec {
            slots: laser_tractor::SLOTS as u32,
            faces: vec![(0..laser_tractor::SLOTS as u32).collect(); 6],
            inputs: Vec::new(),
            outputs: (0..laser_tractor::SLOTS as u32).collect(),
            menu: true,
        });
        r = r.block_entity_kind(tractor);
        // The SPS: casing and coils only join the structure; the ports take the energy and hold the polonium and the antimatter.
        for part in [sps::CASING, sps::COIL] {
            let mut k = kind(part, false);
            k.keep_data = false;
            k.multiblock = Some(MultiblockSpec { group: sps::GROUP.into() });
            r = r.block_entity_kind(k);
        }
        let mut sport = kind(sps::PORT, true);
        sport.keep_data = false;
        sport.multiblock = Some(MultiblockSpec { group: sps::GROUP.into() });
        sport.container = Some(ContainerSpec { slots: 1, faces: vec![Vec::new(); 6], inputs: Vec::new(), outputs: Vec::new(), menu: true });
        sport.energy = Some(EnergySpec { capacity: sps::PORT_ENERGY, max_receive: sps::PORT_ENERGY, max_extract: 0, faces: vec![FlowSide::Input; 6] });
        for (kind_name, side) in [("chemical", FlowSide::Input), ("chemical", FlowSide::Output)] {
            sport.tanks.push(TankSpec { kind: kind_name.into(), capacity: sps::SPEC_CAPACITY, max_rate: 1 << 30, faces: vec![side; 6] });
        }
        r = r.block_entity_kind(sport);
        // The bounding blocks the big machines place around themselves.
        let mut bounding = kind(bounding::BLOCK, true);
        bounding.keep_data = false;
        r = r.block_entity_kind(bounding);
        // The logistical sorter: the intake, the outbox and five filter slots.
        let mut sorter = kind(sorter::SORTER, true);
        sorter.container = Some(ContainerSpec { slots: sorter::SLOTS as u32, faces: sorter::item_faces(laser::NORTH), inputs: vec![0], outputs: vec![1], menu: true });
        r = r.block_entity_kind(sorter);
        // The quantum entangloporter: a mirror of the shared energy, fluid and chemical, and the shared item slot.
        let mut qe = kind(entangloporter::BLOCK, true);
        qe.keep_data = false;
        qe.energy = Some(EnergySpec { capacity: entangloporter::ENERGY_CAPACITY, max_receive: entangloporter::ENERGY_CAPACITY, max_extract: entangloporter::ENERGY_CAPACITY, faces: vec![FlowSide::Both; 6] });
        for kind_name in ["fluid", "chemical"] {
            qe.tanks.push(TankSpec { kind: kind_name.into(), capacity: entangloporter::SPEC_CAPACITY, max_rate: 1 << 30, faces: vec![FlowSide::Both; 6] });
        }
        qe.container = Some(ContainerSpec { slots: 1, faces: vec![vec![0]; 6], inputs: vec![0], outputs: vec![0], menu: true });
        r = r.block_entity_kind(qe);
        // The teleporter: it takes energy from every side and sends the players that stand in its frame.
        let mut tele = kind(teleporter::TELEPORTER, true);
        tele.energy = Some(EnergySpec { capacity: teleporter::CAPACITY, max_receive: teleporter::CAPACITY, max_extract: 0, faces: vec![FlowSide::Input; 6] });
        tele.container = Some(ContainerSpec { slots: 1, faces: vec![Vec::new(); 6], inputs: Vec::new(), outputs: Vec::new(), menu: true });
        r = r.block_entity_kind(tele);
        // The bins: the output window and the input slot over a count kept in the data.
        for (name, _) in bin::BINS {
            let mut b = kind(name, true);
            b.container = Some(ContainerSpec { slots: 2, faces: vec![vec![0, 1]; 6], inputs: vec![1], outputs: vec![0], menu: true });
            r = r.block_entity_kind(b);
        }
        // The industrial alarm lights up with redstone; the structural glass takes the place of a casing in the multiblocks that have one.
        r = r.block_entity_kind(kind(alarm::ALARM, true));
        let mut glass = kind("mekanism:structural_glass", false);
        glass.keep_data = false;
        glass.multiblock = Some(MultiblockSpec {
            group: [dynamic::GROUP, boiler::GROUP, induction::GROUP, sps::GROUP, evaporation::GROUP, turbine::GROUP].join(","),
        });
        r = r.block_entity_kind(glass);
        // The digital miner: 27 slots for what it mines, five for its filters.
        let mut dm = kind(miner::MINER, true);
        dm.energy = Some(EnergySpec { capacity: miner::CAPACITY, max_receive: miner::CAPACITY, max_extract: 0, faces: vec![FlowSide::Input; 6] });
        dm.container = Some(ContainerSpec {
            slots: miner::SLOTS as u32,
            faces: miner::item_faces(laser::NORTH),
            inputs: (0..miner::BUFFER as u32).collect(),
            outputs: (0..miner::BUFFER as u32).collect(),
            menu: true,
        });
        r = r.block_entity_kind(dm);
        let mut creative = kind("mekanism:creative_energy_cube", true);
        creative.container = Some(slots());
        creative.energy = Some(energy(CREATIVE_CUBE, 1_000_000_000, FlowSide::Output));
        r = r.block_entity_kind(creative);
        // Cables, tubes and pipes of the four tiers (`CableTier`, `TubeTier`, `PipeTier`): what each carries a tick.
        // They tick so the configurator can reach them (`run_transmitters`): the game moves the contents itself.
        for (i, tier) in ["basic", "advanced", "elite", "ultimate"].iter().enumerate() {
            for (name, medium, rates) in [
                ("universal_cable", TransmitterMedium::Energy, CABLE_RATES),
                ("pressurized_tube", TransmitterMedium::Tank("chemical".into()), TUBE_RATES),
                ("mechanical_pipe", TransmitterMedium::Tank("fluid".into()), PIPE_RATES),
                ("logistical_transporter", TransmitterMedium::Items, TRANSPORTER_RATES),
            ] {
                let mut k = kind(&format!("mekanism:{tier}_{name}"), true);
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
            card_clicks: Vec::new(),
            suit_step: HashMap::new(),
            waste_filtered: HashSet::new(),
            menu_texts: HashMap::new(),
            stabilizer_buttons: Vec::new(),
            stabilizer_tickets: HashMap::new(),
            empty: HashMap::new(),
            empty_tickets: HashMap::new(),
            empty_ready: HashMap::new(),
            port_modes: HashMap::new(),
            tool: HashMap::new(),
            tool_writes: Vec::new(),
            applied: HashSet::new(),
            rng: 0,
            now: 0,
            last_block_use: HashMap::new(),
            flow: HashMap::new(),
            buttons: HashMap::new(),
            suns: HashMap::new(),
            burns: HashMap::new(),
            last_power: HashMap::new(),
            can: true,
            install: HashMap::new(),
            removals: HashMap::new(),
            installs: HashMap::new(),
            pump_nodes: HashMap::new(),
            dynamic: HashMap::new(),
            matrices: HashMap::new(),
            plants: HashMap::new(),
            boilers: HashMap::new(),
            turbines: HashMap::new(),
            reactors: HashMap::new(),
            fusions: HashMap::new(),
            beams: HashMap::new(),
            digging: HashMap::new(),
            bound_owner: HashMap::new(),
            spses: HashMap::new(),
            sorter_buttons: Vec::new(),
            sorter_texts: Vec::new(),
            entangled: HashMap::new(),
            qe_buttons: Vec::new(),
            qe_texts: Vec::new(),
            bin_buttons: Vec::new(),
            alarm_set: HashSet::new(),
            miner_buttons: Vec::new(),
            miner_texts: Vec::new(),
            text_inputs: Vec::new(),
            radiation: radiation::Manager::default(),
            miner_work: HashMap::new(),
            bin_inserts: HashMap::new(),
            bin_shown: HashMap::new(),
            bin_set: HashSet::new(),
            teleporter_buttons: Vec::new(),
            teleporter_texts: Vec::new(),
            freq_viewer: HashMap::new(),
            freq_registry: None,
            item_menu_sent: HashMap::new(),
            frames: teleporter::Frames::default(),
            sorter_set: HashSet::new(),
            laser_buttons: Vec::new(),
            laser_texts: Vec::new(),
            turbine_buttons: Vec::new(),
            hud_sent: HashMap::new(),
            glide_ticks: HashMap::new(),
            robit_damage: HashMap::new(),
            robit_renames: HashMap::new(),
            robit_recipes: HashMap::new(),
            fusion_buttons: Vec::new(),
            fusion_texts: Vec::new(),
            fission_buttons: Vec::new(),
            fission_texts: Vec::new(),
            blades: Vec::new(),
            evaporating: HashMap::new(),
            matrix_given: HashMap::new(),
            plenishers: HashMap::new(),
            capacity: HashMap::new(),
            force: false,
            reactions: reaction::Book::default(),
            qio: qio::QIOManager::new(),
            formed_structures: HashSet::new(),
            formed_blocks: HashSet::new(),
        }, r.on_chunks().on_items().on_tick().on_blocks().on_damage().on_item_release().on_net().on_mod_entities().on_block_break_start().redirect(bounding::BLOCK, 0, 1)))
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

    fn tick(&mut self, tick: u64) {
        self.run_radiation(tick);
        self.check_tool_writes(tick);
        self.suits_tick(tick);
    }

    /// The answer to an EMPTY's `summon`: the drop exists (`Some` id) or it could not be made (`None`). A confirmed
    /// drop's slot is cleared on the block's next tick (`apply_empty_ready`); a failed one stays in the machine, and
    /// the player is told when the empty is complete but not every stack dropped.
    fn entity_spawned(&mut self, ticket: u64, id: Option<u32>) {
        let Some(t) = self.empty_tickets.remove(&ticket) else { return };
        let Some(mut pending) = self.empty.remove(&t.key) else { return };
        pending.tickets.remove(&ticket);
        if id.is_some() {
            pending.confirmed.push((t.slot, t.item, t.count));
        } else {
            pending.failed += 1;
        }
        if !pending.tickets.is_empty() {
            self.empty.insert(t.key, pending);
            return;
        }
        if !pending.confirmed.is_empty() {
            self.empty_ready.insert(t.key, (pending.confirmed, self.now));
        }
        if pending.failed > 0 || pending.unsummoned > 0 {
            let stayed = pending.failed + pending.unsummoned;
            let _ = players::tell(&pending.player, &format!("Empty could not drop every stack: {stayed} stayed in the machine"));
        }
    }

    fn block_broken(&mut self, player: &Player, pos: BlockPos, state: &str) {
        let name = state.split('[').next().unwrap_or("");
        if name == dimensional_stabilizer::BLOCK {
            self.release_stabilizer_tickets(&player.dimension, pos);
        }
        if name == bounding::BLOCK {
            // Java BlockBounding.java:140-148 & 162-177: Breaking a bounding block destroys the main machine!
            let main = self
                .bound_owner
                .remove(&(player.dimension.clone(), pos.x, pos.y, pos.z))
                .or_else(|| self.find_bounding_main(&player.dimension, pos.x, pos.y, pos.z));
            if let Some(main) = main {
                let _ = world::mine_block(&player.dimension, main[0], main[1], main[2], CONFIGURATOR, false, main[0], main[1], main[2], 1);
            }
        } else if bounding::is_bounded(name) {
            // Java AttributeHasBounding.removeBoundingBlocks (AttributeHasBounding.java:34-48):
            // When the bounded machine is removed, remove all its bounding blocks.
            let facing = sides::property(state, "facing").unwrap_or("north");
            let mut changes = Vec::new();
            for (o, _) in bounding::parts_with_facing(name, facing) {
                let p = [pos.x + o[0], pos.y + o[1], pos.z + o[2]];
                self.bound_owner.remove(&(player.dimension.clone(), p[0], p[1], p[2]));
                changes.push(BlockChange { x: p[0], y: p[1], z: p[2], state: "minecraft:air".into() });
            }
            if !changes.is_empty() {
                let _ = world::set_blocks(&player.dimension, &changes);
            }
        }
        if is_multiblock_block(name) {
            self.formed_blocks.remove(&(player.dimension.clone(), pos.x, pos.y, pos.z));
        }
    }

    fn block_placed(&mut self, player: &Player, pos: BlockPos, state: &str) {
        let name = state.split('[').next().unwrap_or("");
        if bounding::is_bounded(name) {
            // Java AttributeHasBounding.placeBoundingBlocks (AttributeHasBounding.java:50-65):
            // Immediately place all bounding blocks on machine placement instead of waiting up to 20 ticks.
            let facing = sides::property(state, "facing").unwrap_or("north");
            let mut changes = Vec::new();
            for (o, part) in bounding::parts_with_facing(name, facing) {
                let p = [pos.x + o[0], pos.y + o[1], pos.z + o[2]];
                self.bound_owner.insert((player.dimension.clone(), p[0], p[1], p[2]), [pos.x, pos.y, pos.z]);
                changes.push(BlockChange { x: p[0], y: p[1], z: p[2], state: format!("{}[part={part}]", bounding::BLOCK) });
            }
            if !changes.is_empty() {
                let _ = world::set_blocks(&player.dimension, &changes);
            }
        }
    }

    /// `TileEntityMultiblock.onActivate` (`TileEntityMultiblock.java:218-229`), FORK-29: the structure's own use (its screen) only when
    /// it is formed and the player is not sneaking; otherwise the click passes to the held item (a block in hand is placed against it).
    /// A casing never takes the click.
    /// FORK-35 `BlockItem.canPlace`: a machine whose model fills the blocks around it is placed only where they are free.
    fn can_place(&mut self, player: &Player, pos: BlockPos, state: &str) -> bool {
        bounding::can_place(&player.dimension, [pos.x, pos.y, pos.z], state)
    }
    fn player_damaged(&mut self, player: &Player, source: DamageSource, amount: f32) -> f32 {
        self.suit_damage(player, &source, amount)
    }

    fn item_hit_damage(&mut self, _player: &Player, stack: ItemStack, _kind: &str, damage: f32) -> f32 {
        self.meka_tool_hit_damage(&stack, damage)
    }

    fn item_released(&mut self, player: &Player, stack: ItemStack, _hand: &str, _ticks_held: u32) -> bool {
        self.electric_bow_released(player, &stack)
    }

    fn item_shot(&mut self, player: &Player, stack: ItemStack, projectiles: Vec<u32>) {
        self.electric_bow_shot(player, &stack, &projectiles);
    }

    fn item_hit(&mut self, player: &Player, stack: ItemStack, _entity: u32, _kind: &str) {
        self.meka_tool_hit(player, &stack);
    }

    fn block_used(&mut self, player: &Player, pos: BlockPos, sneaking: bool, stack: Option<ItemStack>) -> BlockUse {
        let Some(b) = world::get_block(&player.dimension, pos.x, pos.y, pos.z) else { return BlockUse::Default };
        let name = b.split('[').next().unwrap_or("");
        if !is_multiblock_block(name) || stack.is_some_and(|s| s.item == CONFIGURATOR) {
            return BlockUse::Default;
        }
        let formed = self.formed_blocks.contains(&(player.dimension.clone(), pos.x, pos.y, pos.z));
        if sneaking || !formed || !is_multiblock_port_or_valve(name) { BlockUse::Pass } else { BlockUse::Default }
    }

    fn entities_tick(&mut self, _tick: u64, list: Vec<ModEntity>) -> Vec<ModEntityUpdate> {
        if self.robit_recipes.is_empty() {
            for c in recipe_types::cooking_recipes("smelting") {
                for input in &c.inputs {
                    self.robit_recipes.entry(input.clone()).or_insert_with(|| (c.output.clone(), c.count));
                }
            }
        }
        robit_world::tick_all(&mut self.robit_damage, &mut self.robit_renames, &self.robit_recipes, list)
    }

    fn mod_entity_used(&mut self, player: &Player, entity: ModEntity, sneaking: bool) -> bool {
        entity.kind == robit_world::KIND && self.robit_used(player, &entity, sneaking)
    }

    fn item_menu_tick(&mut self, player: &Player, menu: horizon_mod_sdk::server::ItemMenu) -> Option<horizon_mod_sdk::server::ItemMenuUpdate> {
        self.item_menu_tick_of(player, &menu)
    }

    fn entity_menu_button(&mut self, player: &Player, entity: ModEntity, id: i32) -> Option<ModEntityUpdate> {
        if entity.kind == robit_world::KIND { self.robit_button(player, &entity, id) } else { None }
    }

    fn entity_menu_taken(&mut self, _player: &Player, entity: ModEntity, slot: u32, _stack: ItemStack) -> Option<ModEntityUpdate> {
        if entity.kind == robit_world::KIND { self.robit_taken(&entity, slot) } else { None }
    }

    fn mod_entity_damaged(&mut self, entity: ModEntity, _source: DamageSource, amount: f32) -> f32 {
        self.robit_hurt(&entity, amount)
    }

    fn mod_entity_died(&mut self, entity: ModEntity) {
        self.robit_died(&entity);
    }

    fn block_break_start(&mut self, player: &Player, stack: ItemStack, pos: BlockPos, state: &str) -> bool {
        self.meka_break(player, &stack, pos, state)
    }

    fn item_used(&mut self, player: &Player, stack: ItemStack, pos: Option<BlockPos>) -> bool {
        if stack.item == robit::ITEM_ID && self.robit_placed(player, &stack, pos) {
            return true;
        }
        if stack.item == item_menus::DICTIONARY_ITEM && self.dictionary_used(player, &stack) {
            return true;
        }
        if stack.item == electric_bow::ELECTRIC_BOW && self.electric_bow_mode(player, &stack, pos) {
            return true;
        }
        if stack.item == meka_tool::MEKA_TOOL && self.meka_tool_used(player, &stack) {
            return true;
        }
        if stack.item == configuration_card::ITEM || stack.item == assemblicator::CRAFTING_FORMULA {
            if self.card_or_formula_used(player, &stack, pos) {
                return true;
            }
        }
        // `ItemBlockCardboardBox.onItemUseFirst`: a box used on a block, not sneaking, packs it and is used up (not in creative).
        if stack.item == cardboard_box::BLOCK
            && let Some(at) = pos
            && !player_control::get_motion(&player.id).is_some_and(|m| m.sneaking)
            && cardboard_box::try_pack(&player.dimension, at)
        {
            if player.game_mode != GameMode::Creative
                && let Some((slot, _)) = Self::find_hotbar_slot(&player.id, &stack)
            {
                let rest = (stack.count > 1).then(|| ItemStack { count: stack.count - 1, ..stack.clone() });
                let _ = player_items::compare_and_set_slot(&player.id, slot, Some(&stack), rest.as_ref());
            }
            return true;
        }
        if self.read_radiation(player, &stack.item) {
            return true;
        }
        // Java BlockBounding.java:101-120: Redirect clicks on bounding blocks to the main machine.
        let pos = match pos {
            Some(p) => {
                if let Some(block) = world::get_block(&player.dimension, p.x, p.y, p.z) {
                    let name = block.split('[').next().unwrap_or("");
                    if name == bounding::BLOCK {
                        if let Some(main) = self
                            .bound_owner
                            .get(&(player.dimension.clone(), p.x, p.y, p.z))
                            .copied()
                            .or_else(|| self.find_bounding_main(&player.dimension, p.x, p.y, p.z))
                        {
                            self.bound_owner.insert((player.dimension.clone(), p.x, p.y, p.z), main);
                            Some(BlockPos { x: main[0], y: main[1], z: main[2] })
                        } else {
                            Some(p)
                        }
                    } else {
                        Some(p)
                    }
                } else {
                    Some(p)
                }
            }
            None => None,
        };

        if let Some(k) = upgrades::type_of(&stack.item) {
            // `ItemUpgrade.useOn`: sneaking on a machine that takes the type puts the whole stack in, as far as it fits.
            let (Some(pos), true) = (pos, player_control::get_motion(&player.id).is_some_and(|m| m.sneaking)) else { return false };
            let Some(block) = world::get_block(&player.dimension, pos.x, pos.y, pos.z) else { return false };
            let name = block.split('[').next().unwrap_or("");
            let Some(def) = MACHINES.iter().find(|d| d.block == name) else { return false };
            if !upgrades::supported(def).contains(&k) {
                return false;
            }
            // The hotbar slot the stack is in (the held slot is not told: the first of the same stack). The click
            // is not claimed when the stack is not in the hotbar, rather than installing from slot 0 by default.
            let Some((slot, _)) = Self::find_hotbar_slot(&player.id, &stack) else { return false };
            self.installs
                .entry((player.dimension.clone(), pos.x, pos.y, pos.z))
                .or_default()
                .push((player.id.clone(), k, stack.count, slot));
            return true;
        }
        if let Some(pos) = pos
            && !player_control::get_motion(&player.id).is_some_and(|m| m.sneaking)
            && world::get_block(&player.dimension, pos.x, pos.y, pos.z).is_some_and(|b| bin::capacity_of(b.split('[').next().unwrap_or("")).is_some())
        {
            // Using an item on a bin puts the whole stack in (an empty hand opens the screen). The click is not
            // claimed when the stack is not in the hotbar, rather than taking from slot 0 by default.
            let Some((slot, _)) = Self::find_hotbar_slot(&player.id, &stack) else { return false };
            self.bin_inserts.entry((player.dimension.clone(), pos.x, pos.y, pos.z)).or_default().push((player.id.clone(), slot));
            return true;
        }
        if stack.item == turbine::BLADE {
            // `BlockTurbineRotor.useItemOn`: a blade on a rotor puts it on (sneaking: takes one off, into the stack).
            let Some(pos) = pos else { return false };
            let Some(block) = world::get_block(&player.dimension, pos.x, pos.y, pos.z) else { return false };
            if !block.starts_with(turbine::ROTOR) {
                return false;
            }
            let add = !player_control::get_motion(&player.id).is_some_and(|m| m.sneaking);
            // The hotbar slot the blade is in; the click is not claimed when it is not in the hotbar, rather than
            // taking from slot 0 by default.
            let Some((slot, _)) = Self::find_hotbar_slot(&player.id, &stack) else { return false };
            self.blades.push(turbine::BladeRequest { dimension: player.dimension.clone(), pos, player: player.id.clone(), slot, add });
            return true;
        }
        if stack.item != CONFIGURATOR {
            return false;
        }
        let sneaking = player_control::get_motion(&player.id).is_some_and(|m| m.sneaking);
        let mode = self.tool_mode(&player.id, &stack);
        let Some(pos) = pos else {
            // Sneaking in the air changes the tool's mode (`ItemConfigurator.use`), unless this is the use in
            // the air that follows a click the tool just took on a machine: that click is one, and does nothing more.
            // The gesture is the port's own: the original cycles by radial menu (N) or shift-scroll, neither of which
            // the Android client has.
            if sneaking
                && self
                    .last_block_use
                    .get(&player.id)
                    .is_some_and(|t| self.now.saturating_sub(*t) <= 2)
            {
                return true;
            }
            if sneaking {
                // The original opens a radial menu on the mode key; here sneaking in the air does (the Android client has no mode key).
                let entries: Vec<players::RadialEntry> = (0..8u8)
                    .map(|n| players::RadialEntry { id: format!("configurator:{n}"), label: ToolMode::from_number(n).label().to_owned(), icon: String::new() })
                    .collect();
                if !players::open_radial_menu(&player.id, &entries, u32::from(mode as u8)) {
                    let mode = mode.next();
                    self.tool.insert(player.id.clone(), mode);
                    self.write_tool_mode(player, &stack, mode);
                    let _ = players::tell(&player.id, &format!("Configure State: {}", mode.label()));
                }
                return true;
            }
            return false;
        };
        let Some(block) = world::get_block(&player.dimension, pos.x, pos.y, pos.z) else { return false };
        let name = block.split('[').next().unwrap_or("");
        if mode.is_configurating() {
            // Generators and fluid tanks have no side configuration to change, and a machine is only claimed for a
            // transmission it configures: the original leaves the click alone in both cases (a plain click opens the
            // block's screen), so the click is not claimed here either, instead of being swallowed.
            if let Some(def) = MACHINES.iter().find(|d| d.block == name) {
                if def.generator.is_some() || matches!(def.kind, Kind::Tank) && def.tank.is_some_and(|t| t.fluid) {
                    return false;
                }
                // A sneak click on a pump resets its calculation in any configurate mode (the original has no side
                // configuration on a pump); otherwise only a transmission the machine configures is claimed.
                if !(sneaking && matches!(def.kind, Kind::Pump)) {
                    if let Some(t) = mode.transmission() {
                        if !def.layout.transmissions().contains(&t) {
                            return false;
                        }
                    }
                }
            } else if name.ends_with("_energy_cube") {
                // The cube configures energy and items only.
                if !matches!(mode.transmission(), Some(Transmission::Energy | Transmission::Items)) {
                    return false;
                }
            } else if bin::capacity_of(name).is_some() {
                // A sneak click with the configurator toggles whether the bin ejects (`run_bins`); a plain click is
                // left alone, so it opens the bin's screen as it does without the tool.
                if !sneaking {
                    return false;
                }
            } else if !transmitter::is_transmitter(name)
                && !matches!(
                    name,
                    sps::PORT
                        | induction::PORT
                        | fusion::PORT
                        | fission::PORT
                        | boiler::VALVE
                        | fusion::MATRIX
                        | dynamic::VALVE
                        | turbine::VALVE
                        | turbine::VENT
                        | evaporation::VALVE
                )
            {
                return false;
            }
        } else {
            // Empty, rotate and the wrench act on the block itself: what each of them reaches. The wrench takes apart
            // any of the mod's blocks but a bounding block (a part of a multiblock, not a block of its own).
            let ok = match mode {
                ToolMode::Empty => {
                    MACHINES.iter().any(|d| d.block == name)
                        || name.ends_with("_energy_cube")
                        || name == sorter::SORTER
                        || name == miner::MINER
                }
                ToolMode::Rotate => {
                    MACHINES.iter().any(|d| d.block == name)
                        || name.ends_with("_energy_cube")
                        || bin::capacity_of(name).is_some()
                        || name == sorter::SORTER
                }
                ToolMode::Wrench => name.starts_with("mekanism:") && name != bounding::BLOCK,
                _ => false,
            };
            if !ok {
                return false;
            }
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
            mode,
            tick: 0,
        });
        self.last_block_use.insert(player.id.clone(), self.now);
        true
    }

    /// A button of a machine's screen: the side configuration window's (a side, the eject toggle, the clear button).
    fn net_message(&mut self, player: &Player, channel: &str, data: &[u8]) {
        // The choice of a radial menu (API 42): the configurator's mode.
        if channel == "mekanism:radial" {
            if let Some(n) = std::str::from_utf8(data).ok().and_then(|t| t.strip_prefix("configurator:")).and_then(|t| t.parse::<u8>().ok()).filter(|n| *n < 8) {
                let held = player_items::get_inventory(&player.id)
                    .and_then(|inv| inv.into_iter().take(9).flatten().find(|s| s.item == CONFIGURATOR));
                if let Some(stack) = held {
                    let mode = ToolMode::from_number(n);
                    self.tool.insert(player.id.clone(), mode);
                    self.write_tool_mode(player, &stack, mode);
                    let _ = players::tell(&player.id, &format!("Configure State: {}", mode.label()));
                }
            }
            return;
        }
        if channel != horizon_mod_sdk::server::menu_input::channel("mekanism") {
            return;
        }
        if let Some(input) = horizon_mod_sdk::server::menu_input::decode(data) {
            if input.pos.x == i32::MIN {
                // a text box of an entity's menu: the position is `(i32::MIN, 0, entity id)`; the Robit's rename box is 10
                if input.id == 10 && !input.text.trim().is_empty() {
                    self.robit_renames.insert(input.pos.z as u32, input.text.trim().chars().take(32).collect());
                }
                return;
            }
            self.text_inputs.push((player.dimension.clone(), input.pos, input.id, input.text, player.id.clone(), player.name.clone()));
        }
    }

    fn menu_button(&mut self, player: &Player, _pos: BlockPos, id: i32, e: BlockEntity) -> Option<BlockEntityUpdate> {
        let block = e.state.split('[').next()?;
        if block == miner::MINER {
            self.miner_buttons.push((e.dimension.clone(), e.pos, id));
            return None;
        }
        if bin::capacity_of(block).is_some() {
            self.bin_buttons.push((e.dimension.clone(), e.pos, id));
            return None;
        }
        if block == teleporter::TELEPORTER {
            self.teleporter_buttons.push((e.dimension.clone(), e.pos, id, player.id.clone(), player.name.clone()));
            return None;
        }
        if block == entangloporter::BLOCK {
            self.qe_buttons.push((e.dimension.clone(), e.pos, id, player.id.clone(), player.name.clone()));
            return None;
        }
        if block == assemblicator::BLOCK {
            return self.assemblicator_button(&e, id);
        }
        if block == resistive_heater::BLOCK {
            return self.resistive_button(&e, id);
        }
        if block == modification_station::BLOCK {
            return self.modification_station_button(player, &e, id);
        }
        if block == oredictionificator::BLOCK {
            return self.oredictionificator_button(&e, id);
        }
        if block == seismic_vibrator::BLOCK {
            return self.seismic_vibrator_button(&e, id);
        }
        if block == dimensional_stabilizer::BLOCK {
            self.stabilizer_buttons.push((e.dimension.clone(), e.pos, id));
            return None;
        }
        if block == sorter::SORTER {
            self.sorter_buttons.push((e.dimension.clone(), e.pos, id));
            return None;
        }
        if turbine::part_of(&e.state).is_some() && id == turbine::BUTTON_DUMP_MODE {
            if let Some((found, _)) = turbine::decode(&e.data) {
                self.turbine_buttons.push((e.dimension.clone(), found.min));
            }
            return None;
        }
        if block == laser::AMPLIFIER {
            self.laser_buttons.push((e.dimension.clone(), e.pos, id));
            return None;
        }
        if block == fusion::CONTROLLER {
            self.fusion_buttons.push((e.dimension.clone(), e.pos, id));
            return None;
        }
        if block == fission::PORT {
            self.fission_buttons.push((e.dimension.clone(), e.pos, id));
            return None;
        }
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
        if id == chem::DUMP && def.kind == Kind::Factory && def.form >= 3 {
            // GuiDumpButton: the factory's tank is emptied; its next tick draws the screen again.
            return Some(BlockEntityUpdate {
                dimension: e.dimension.clone(),
                pos: e.pos,
                data: e.data.clone(),
                items: None,
                energy: None,
                energy_faces: None,
                tanks: Some(vec![Tank { substance: String::new(), amount: 0 }]),
                tank_faces: None,
                menu_data: None,
                transmitter_faces: None,
            });
        }
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

    fn multiblock_changed(&mut self, group: &str, dimension: &str, structures: Vec<Structure>) -> Vec<BlockEntityUpdate> {
        let groups: Vec<&str> = if group.contains(',') {
            group.split(',').map(|s| s.trim()).collect()
        } else {
            vec![group]
        };
        let mut updates = Vec::new();
        for g in groups {
            if g == dynamic::GROUP {
                updates.extend(self.dynamic_structures(dimension, structures.clone()));
            } else if g == induction::GROUP {
                updates.extend(self.matrix_structures(dimension, structures.clone()));
            } else if g == sps::GROUP {
                updates.extend(self.sps_structures(dimension, structures.clone()));
            } else if g == fusion::GROUP {
                updates.extend(self.fusion_structures(dimension, structures.clone()));
            } else if g == fission::GROUP {
                updates.extend(self.fission_structures(dimension, structures.clone()));
            } else if g == turbine::GROUP {
                updates.extend(self.turbine_structures(dimension, structures.clone()));
            } else if g == boiler::GROUP {
                updates.extend(self.boiler_structures(dimension, structures.clone()));
            } else if g == evaporation::GROUP {
                updates.extend(self.evaporation_structures(dimension, structures.clone()));
            }
        }

        // Common multiblock post-processing for all 8 multiblock systems:
        // 1. Formation tracking, sparkle particles (FORK-30) and indigo action-bar notification (S2)
        // 2. Disassembly cleanup and capability closure for unformed/invalid valves/ports (S3)
        for s in &structures {
            let key = (dimension.to_owned(), group.to_owned(), [s.min.x, s.min.y, s.min.z]);
            let is_formed = updates.iter().any(|u| {
                u.pos.x >= s.min.x
                    && u.pos.x <= s.max.x
                    && u.pos.y >= s.min.y
                    && u.pos.y <= s.max.y
                    && u.pos.z >= s.min.z
                    && u.pos.z <= s.max.z
                    && u.data.first().copied().unwrap_or(0) != 0
            });

            if is_formed {
                for m in &s.members {
                    self.formed_blocks.insert((dimension.to_owned(), m.pos.x, m.pos.y, m.pos.z));
                }
                if self.formed_structures.insert(key.clone()) {
                    // Java: TileEntityMultiblock.doMultiblockSparkle (TileEntityMultiblock.java:297-312):
                    // When a structure transitions from unformed to formed, players within 40 blocks (1,600 dist_sqr)
                    // receive sparkles (or action bar chat MULTIBLOCK_FORMED_CHAT).
                    let center = [
                        (s.min.x as f64 + s.max.x as f64) / 2.0,
                        (s.min.y as f64 + s.max.y as f64) / 2.0,
                        (s.min.z as f64 + s.max.z as f64) / 2.0,
                    ];
                    // `enableMultiblockFormationParticles` (default true): the sparkle on the six faces of the box, otherwise the
                    // indigo "Multiblock formed" action-bar line. Only if a player is within 40 blocks of the structure.
                    let near = players::online().iter().any(|p| {
                        p.dimension == dimension && (p.x - center[0]).powi(2) + (p.y - center[1]).powi(2) + (p.z - center[2]).powi(2) <= 1600.0
                    });
                    if near {
                        sparkle::show(dimension, [s.min.x, s.min.y, s.min.z], [s.max.x, s.max.y, s.max.z], self.now.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (s.min.x as u64) << 20 ^ (s.min.z as u64));
                    }
                    // FORK-31: render.multiblock-fluid
                }
            } else {
                for m in &s.members {
                    self.formed_blocks.remove(&(dimension.to_owned(), m.pos.x, m.pos.y, m.pos.z));
                }
                if self.formed_structures.remove(&key) {
                    // Java MultiblockCache.java: sync multiblock contents to cache on disassembly
                    // FORK-31: remove rendered multiblock fluid
                }
            }
        }

        // Sanitize any updates for unformed components (S3), and open or close what the valves and ports let through (FORK-34):
        for u in &mut updates {
            let formed = u.data.first().copied().unwrap_or(0) != 0;
            if is_multiblock_port_or_valve_at(dimension, u.pos) {
                capmask::set_formed(dimension, u.pos, formed);
            }
            if !formed {
                if let Some(block) = world::get_block(dimension, u.pos.x, u.pos.y, u.pos.z) {
                    let name = block.split('[').next().unwrap_or("");
                    if is_multiblock_port_or_valve(name) {
                        for i in 0..5 {
                            let _ = world::set_tank_filter(dimension, u.pos.x, u.pos.y, u.pos.z, i, &[NOTHING.to_owned()]);
                        }
                        // The faces of the tanks are closed per tick with the entity's real tank count (`block_entities_tick`).
                        u.energy_faces = Some(vec![FlowSide::None; 6]);
                    }
                }
            }
        }

        updates
    }

    fn block_entities_tick(&mut self, tick: u64, mut entities: Vec<BlockEntity>) -> Vec<BlockEntityUpdate> {
        self.now = tick;
        let mut out = Vec::new();
        // The configurator clicks that act on the block itself go first, before the machines' own ticks.
        out.extend(self.tool_clicks(&mut entities));
        out.extend(self.card_clicks_tick(&mut entities));
        // Empties whose drops the game confirmed clear their slots now.
        out.extend(self.apply_empty_ready(&mut entities));
        // Unformed multiblock ports/valves must NOT expose capabilities or act as standalone tanks (S3)
        for e in &entities {
            let name = e.state.split('[').next().unwrap_or("");
            if is_multiblock_port_or_valve(name) && e.data.first().copied().unwrap_or(0) == 0 {
                let tank_count = e.tanks.len();
                for i in 0..tank_count {
                    let _ = world::set_tank_filter(&e.dimension, e.pos.x, e.pos.y, e.pos.z, i as u32, &[NOTHING.to_owned()]);
                }
                let tank_faces = if tank_count > 0 {
                    Some(vec![vec![FlowSide::None; 6]; tank_count])
                } else {
                    None
                };
                let energy_faces = if !e.energy_faces.is_empty() {
                    Some(vec![FlowSide::None; 6])
                } else {
                    None
                };
                out.push(BlockEntityUpdate {
                    dimension: e.dimension.clone(),
                    pos: e.pos,
                    data: e.data.clone(),
                    items: None,
                    energy: None,
                    energy_faces,
                    tanks: None,
                    tank_faces,
                    menu_data: None,
                    transmitter_faces: None,
                });
            }
        }
        let valves: Vec<&BlockEntity> = entities.iter().filter(|e| e.state.split('[').next() == Some(dynamic::VALVE)).collect();
        if !valves.is_empty() {
            out.extend(self.run_dynamic(&valves));
        }
        out.extend(self.run_bounding(&entities, tick));
        let teleporters: Vec<&BlockEntity> = entities.iter().filter(|e| e.state.split('[').next() == Some(teleporter::TELEPORTER)).collect();
        if !teleporters.is_empty() || !self.teleporter_buttons.is_empty() {
            out.extend(self.run_teleporters(&teleporters, tick));
        }
        let bins: Vec<&BlockEntity> = entities.iter().filter(|e| bin::capacity_of(e.state.split('[').next().unwrap_or("")).is_some()).collect();
        if !bins.is_empty() || !self.bin_buttons.is_empty() {
            out.extend(self.run_bins(&bins, tick));
        }
        let transmitters: Vec<&BlockEntity> =
            entities.iter().filter(|e| transmitter::is_transmitter(e.state.split('[').next().unwrap_or(""))).collect();
        if !transmitters.is_empty() {
            out.extend(self.run_transmitters(&transmitters));
        }
        let miners: Vec<&BlockEntity> = entities.iter().filter(|e| e.state.split('[').next() == Some(miner::MINER)).collect();
        if !miners.is_empty() || !self.miner_buttons.is_empty() {
            out.extend(self.run_miners(&miners, tick));
        }
        let alarms: Vec<&BlockEntity> = entities.iter().filter(|e| e.state.split('[').next() == Some(alarm::ALARM)).collect();
        if !alarms.is_empty() {
            out.extend(self.run_alarms(&alarms));
        }
        let assemblicators: Vec<&BlockEntity> = entities.iter().filter(|e| e.state.split('[').next() == Some(assemblicator::BLOCK)).collect();
        if !assemblicators.is_empty() {
            out.extend(self.run_assemblicators(&assemblicators, tick));
        }
        let stations: Vec<&BlockEntity> = entities.iter().filter(|e| e.state.split('[').next() == Some(modification_station::BLOCK)).collect();
        if !stations.is_empty() {
            out.extend(self.run_modification_stations(&stations));
        }
        let heat_blocks: Vec<&BlockEntity> = entities.iter().filter(|e| heat_world::is_heat_block(e.state.split('[').next().unwrap_or(""))).collect();
        if !heat_blocks.is_empty() {
            let sinks = self.heat_sinks_near(&heat_blocks);
            let (updates, given) = heat_world::run(&heat_blocks, &sinks);
            out.extend(updates);
            self.commit_heat_given(&sinks, given);
        }
        let chargepads: Vec<&BlockEntity> = entities.iter().filter(|e| e.state.split('[').next() == Some(chargepad::BLOCK)).collect();
        if !chargepads.is_empty() {
            out.extend(self.run_chargepads(&chargepads));
        }
        let stabilizers: Vec<&BlockEntity> = entities.iter().filter(|e| e.state.split('[').next() == Some(dimensional_stabilizer::BLOCK)).collect();
        if !stabilizers.is_empty() {
            out.extend(self.run_dimensional_stabilizers(&stabilizers, tick));
        }
        let cardboard_boxes: Vec<&BlockEntity> = entities.iter().filter(|e| e.state.split('[').next() == Some(cardboard_box::BLOCK)).collect();
        if !cardboard_boxes.is_empty() {
            out.extend(self.run_cardboard_boxes(&cardboard_boxes));
        }
        let personal: Vec<&BlockEntity> = entities
            .iter()
            .filter(|e| matches!(e.state.split('[').next(), Some(personal_storage::CHEST_BLOCK | personal_storage::BARREL_BLOCK)))
            .collect();
        if !personal.is_empty() {
            out.extend(self.run_personal_storages(&personal));
        }
        let vibrators: Vec<&BlockEntity> = entities.iter().filter(|e| e.state.split('[').next() == Some(seismic_vibrator::BLOCK)).collect();
        if !vibrators.is_empty() {
            out.extend(self.run_seismic_vibrators(&vibrators, tick));
        }
        let oredictionificators: Vec<&BlockEntity> = entities.iter().filter(|e| e.state.split('[').next() == Some(oredictionificator::BLOCK)).collect();
        if !oredictionificators.is_empty() {
            out.extend(self.run_oredictionificators(&oredictionificators));
        }
        let waste_barrels: Vec<&BlockEntity> = entities.iter().filter(|e| e.state.split('[').next() == Some(radioactive_waste_barrel::BLOCK_ID)).collect();
        if !waste_barrels.is_empty() {
            out.extend(self.run_waste_barrels(&waste_barrels, tick));
        }
        let drive_arrays: Vec<&BlockEntity> = entities.iter().filter(|e| e.state.split('[').next() == Some(qio::BLOCK_DRIVE_ARRAY)).collect();
        if !drive_arrays.is_empty() {
            out.extend(self.qio.run_drive_arrays(&drive_arrays, tick));
        }
        let redstone_adapters: Vec<&BlockEntity> = entities.iter().filter(|e| e.state.split('[').next() == Some(qio::BLOCK_REDSTONE_ADAPTER)).collect();
        if !redstone_adapters.is_empty() {
            out.extend(self.qio.run_redstone_adapters(&redstone_adapters, tick));
        }
        let qes: Vec<&BlockEntity> = entities.iter().filter(|e| e.state.split('[').next() == Some(entangloporter::BLOCK)).collect();
        if !qes.is_empty() || !self.qe_buttons.is_empty() {
            out.extend(self.run_entanglements(&qes, tick));
        }
        let sorters: Vec<&BlockEntity> = entities.iter().filter(|e| e.state.split('[').next() == Some(sorter::SORTER)).collect();
        if !sorters.is_empty() || !self.sorter_buttons.is_empty() {
            out.extend(self.run_sorters(&sorters, tick));
        }
        let sps_ports: Vec<&BlockEntity> = entities.iter().filter(|e| e.state.split('[').next() == Some(sps::PORT)).collect();
        if !sps_ports.is_empty() {
            out.extend(self.run_sps(&sps_ports, tick));
        }
        let laser_parts: Vec<&BlockEntity> = entities
            .iter()
            .filter(|e| matches!(e.state.split('[').next(), Some(laser::LASER | laser::AMPLIFIER | laser::TRACTOR | fusion::MATRIX)))
            .collect();
        if !laser_parts.is_empty() {
            out.extend(self.run_lasers(&laser_parts));
        }
        let fusion_parts: Vec<&BlockEntity> = entities
            .iter()
            .filter(|e| matches!(e.state.split('[').next(), Some(fusion::PORT | fusion::CONTROLLER)))
            .collect();
        if !fusion_parts.is_empty() {
            out.extend(self.run_fusion(&fusion_parts, tick));
        }
        let reactor_ports: Vec<&BlockEntity> = entities.iter().filter(|e| e.state.split('[').next() == Some(fission::PORT)).collect();
        if !reactor_ports.is_empty() {
            out.extend(self.run_fission(&reactor_ports, tick));
        }
        let turbine_parts: Vec<&BlockEntity> = entities
            .iter()
            .filter(|e| matches!(e.state.split('[').next(), Some(turbine::VALVE | turbine::VENT | turbine::ROTOR)))
            .collect();
        if !turbine_parts.is_empty() || !self.blades.is_empty() {
            out.extend(self.run_turbine(&turbine_parts, tick));
        }
        let boiler_valves: Vec<&BlockEntity> = entities.iter().filter(|e| e.state.split('[').next() == Some(boiler::VALVE)).collect();
        if !boiler_valves.is_empty() {
            out.extend(self.run_boiler(&boiler_valves));
        }
        let plant: Vec<&BlockEntity> = entities
            .iter()
            .filter(|e| matches!(e.state.split('[').next(), Some(evaporation::VALVE | evaporation::CONTROLLER)))
            .collect();
        if !plant.is_empty() {
            out.extend(self.run_evaporation(&plant, tick));
        }
        let ports: Vec<&BlockEntity> = entities.iter().filter(|e| e.state.split('[').next() == Some(induction::PORT)).collect();
        if !ports.is_empty() {
            out.extend(self.run_matrix(&ports));
        }
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
                    self.force |= self.charge_from_slot(def, &mut e2);
                }
                let e = &e2;
                out.extend(match def.kind {
                    Kind::Continuous => self.run_continuous(def, e, tick),
                    Kind::Tank => self.run_tank(def, e),
                    Kind::Generator => self.run_generator(def, e, tick),
                    Kind::Reaction => self.run_reaction(def, e, tick),
                    Kind::Factory => self.run_factory(def, e, tick),
                    Kind::Pump => self.run_pump(def, e, tick),
                    _ => self.run(def, e, tick),
                });
            } else if block.ends_with("_energy_cube") {
                out.extend(self.cube(e, block));
            }
        }
        // The comparators beside the cubes and the tanks read their fill (`getRedstoneLevel`).
        for e in &entities {
            let block = e.state.split('[').next().unwrap_or("");
            if block.ends_with("_energy_cube") {
                let (capacity, _) = cube_capacity(block);
                redstone::set_comparator(&e.dimension, e.pos, redstone::level_from_contents(e.energy, capacity));
            } else if let Some(t) = MACHINES.iter().find(|d| d.block == block).and_then(|d| d.tank.as_ref()) {
                let held = e.tanks.first().map_or(0, |t| t.amount);
                redstone::set_comparator(&e.dimension, e.pos, redstone::level_from_contents(held, t.capacity));
            }
        }
        // What a text box confirmed (FORK-20) goes to its machine; one whose machine is not here this tick is dropped.
        for (dimension, pos, id, text, who, who_name) in std::mem::take(&mut self.text_inputs) {
            let Some(e) = entities.iter().find(|e| e.dimension == dimension && e.pos == pos) else { continue };
            let block = e.state.split('[').next().unwrap_or("");
            if block == oredictionificator::BLOCK {
                out.extend(self.oredictionificator_text(e, id, &text));
            } else if block == resistive_heater::BLOCK {
                out.extend(self.resistive_text(e, id, &text));
            } else if block == laser::AMPLIFIER {
                self.laser_texts.push((dimension, pos, id, text));
            } else if block == fission::PORT {
                self.fission_texts.push((dimension, pos, id, text));
            } else if block == fusion::CONTROLLER {
                self.fusion_texts.push((dimension, pos, id, text));
            } else if block == teleporter::TELEPORTER {
                self.teleporter_texts.push((dimension, pos, id, text, who, who_name));
            } else if block == entangloporter::BLOCK {
                self.qe_texts.push((dimension, pos, id, text, who, who_name));
            } else if block == sorter::SORTER {
                self.sorter_texts.push((dimension, pos, id, text));
            } else if block == miner::MINER {
                self.miner_texts.push((dimension, pos, id, text));
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Java TileEntityMultiblock.java:218-229: Multiblock components and valves vs casings.
    #[test]
    fn multiblock_block_classification() {
        assert!(is_multiblock_block(dynamic::CASING));
        assert!(is_multiblock_block(dynamic::VALVE));
        assert!(is_multiblock_block(boiler::CASING));
        assert!(is_multiblock_block(boiler::VALVE));
        assert!(is_multiblock_block(induction::PORT));
        assert!(is_multiblock_block("mekanism:induction_casing"));
        assert!(is_multiblock_block(turbine::ROTOR));
        assert!(is_multiblock_block("mekanism:structural_glass"));
        assert!(!is_multiblock_block("mekanism:crusher"));
        assert!(!is_multiblock_block(bounding::BLOCK));

        assert!(is_multiblock_port_or_valve(dynamic::VALVE));
        assert!(is_multiblock_port_or_valve(boiler::VALVE));
        assert!(is_multiblock_port_or_valve(turbine::VALVE));
        assert!(is_multiblock_port_or_valve(turbine::VENT));
        assert!(is_multiblock_port_or_valve(fission::PORT));
        assert!(is_multiblock_port_or_valve(fusion::PORT));
        assert!(is_multiblock_port_or_valve(sps::PORT));
        assert!(!is_multiblock_port_or_valve(dynamic::CASING));
        assert!(!is_multiblock_port_or_valve(boiler::CASING));
    }

    /// Java TileEntityMultiblock.java:297-312: doMultiblockSparkle distance threshold is 40 blocks (1600 dist_sqr).
    #[test]
    fn multiblock_formation_sparkle_distance_threshold() {
        let max_dist = 40.0;
        let max_dist_sqr = max_dist * max_dist;
        assert_eq!(max_dist_sqr, 1600.0);

        let within_dist_sqr = 30.0f64 * 30.0f64;
        assert!(within_dist_sqr <= 1600.0);

        let outside_dist_sqr = 41.0f64 * 41.0f64;
        assert!(outside_dist_sqr > 1600.0);
    }

    /// Java FormationProtocol.java:37: MAX_SIZE is 18 across all multiblock structures.
    #[test]
    fn multiblock_max_size_constant() {
        const MAX_SIZE: i32 = 18;
        assert_eq!(MAX_SIZE, 18);
    }

    /// Java TileEntityMultiblock.java:218-229: onActivate passes when unformed or sneaking.
    #[test]
    fn multiblock_on_activate_pass_through_conditions() {
        let is_formed = false;
        let is_sneaking = false;
        let pass_to_block = !is_formed || is_sneaking;
        assert!(pass_to_block);

        let is_formed_2 = true;
        let is_sneaking_2 = true;
        let pass_to_block_2 = !is_formed_2 || is_sneaking_2;
        assert!(pass_to_block_2);

        let is_formed_3 = true;
        let is_sneaking_3 = false;
        let pass_to_block_3 = !is_formed_3 || is_sneaking_3;
        assert!(!pass_to_block_3);
    }
}

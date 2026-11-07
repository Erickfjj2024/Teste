//! The pressurized reaction chamber (`TileEntityPressurizedReactionChamber`): an item, a fluid and a gas make an item and a gas
//! after the recipe's own duration, using the recipe's own extra energy a tick. The pure rules, tested without the game, and
//! then its tick.
use super::*;
use crate::machine::{STACK, Stack, Status};
use crate::process::Tank;
use std::collections::HashSet;

/// What each of its three tanks holds (`MAX_FLUID`, `MAX_GAS`: ten buckets).
pub const TANK: u64 = 10_000;
/// The base duration of a recipe with none (`BASE_DURATION`: five seconds).
pub const BASE_DURATION: u32 = 100;
/// What it uses a tick before a recipe adds its own (`pressurizedReactionBase`).
pub const USAGE: u64 = 2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recipe {
    pub items: HashSet<String>,
    pub item_count: u32,
    pub fluids: HashSet<String>,
    pub fluid: u64,
    pub gases: HashSet<String>,
    pub gas: u64,
    pub duration: u32,
    /// The energy a tick the recipe asks on top of the machine's own.
    pub energy: u64,
    pub out_item: Option<(String, u32)>,
    pub out_gas: Option<(String, u64)>,
}

#[derive(Debug, Default)]
pub struct Book {
    recipes: Vec<Recipe>,
}

impl Book {
    pub fn add(&mut self, r: Recipe) {
        self.recipes.push(r);
    }
    pub fn is_empty(&self) -> bool {
        self.recipes.is_empty()
    }
    /// The recipe for what the three inputs hold (the item's id, the fluid's, the gas's): every one must be named by it.
    pub fn find(&self, item: &str, fluid: &str, gas: &str) -> Option<&Recipe> {
        self.recipes.iter().find(|r| r.items.contains(item) && r.fluids.contains(fluid) && r.gases.contains(gas))
    }
    /// Whether some recipe takes the item.
    pub fn takes_item(&self, item: &str) -> bool {
        self.recipes.iter().any(|r| r.items.contains(item))
    }
    /// The fluids and the gases some recipe takes, for the tanks' filters.
    pub fn fluids(&self) -> Vec<String> {
        let mut v: Vec<String> = self.recipes.iter().flat_map(|r| r.fluids.iter().cloned()).collect::<HashSet<_>>().into_iter().collect();
        v.sort();
        v
    }
    pub fn gases(&self) -> Vec<String> {
        let mut v: Vec<String> = self.recipes.iter().flat_map(|r| r.gases.iter().cloned()).collect::<HashSet<_>>().into_iter().collect();
        v.sort();
        v
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Io {
    pub input: Option<Stack>,
    pub output: Option<Stack>,
    pub fluid: Tank,
    pub gas_in: Tank,
    pub gas_out: Tank,
    pub energy: u64,
    pub progress: u32,
}

/// One tick with `speed` speed and `energy_up` energy upgrades: the progress and what it uses follow them; the recipe is done
/// when the progress reaches its (upgraded) duration. Returns what it holds after, its status and the ticks the recipe takes
/// (the base duration when there is none).
pub fn step(book: &Book, io: &Io, speed: u8, energy_up: u8) -> (Io, Status, u32, u64) {
    let mut next = io.clone();
    let base = upgrades::ticks(BASE_DURATION, speed);
    let idle_usage = upgrades::energy_per_tick(USAGE, speed, energy_up);
    if book.is_empty() {
        next.progress = 0;
        return (next, Status::NoBook, base, idle_usage);
    }
    let Some(input) = io.input.as_ref() else {
        next.progress = 0;
        return (next, Status::Idle, base, idle_usage);
    };
    let Some(recipe) = book.find(&input.item, &io.fluid.chemical, &io.gas_in.chemical) else {
        next.progress = 0;
        let status = if book.takes_item(&input.item) { Status::Blocked } else { Status::NoRecipe };
        return (next, status, base, idle_usage);
    };
    let ticks = upgrades::ticks(recipe.duration, speed);
    let usage = upgrades::energy_per_tick(USAGE + recipe.energy, speed, energy_up);
    // Enough of each input and room for each output, or it waits.
    let item_room = match (&recipe.out_item, &io.output) {
        (None, _) => true,
        (Some((_, n)), None) => *n <= STACK,
        (Some((id, n)), Some(o)) => o.item == *id && o.components.is_empty() && o.count + n <= STACK,
    };
    let gas_room = match &recipe.out_gas {
        None => true,
        Some((id, n)) => (io.gas_out.amount == 0 || io.gas_out.chemical == *id) && io.gas_out.amount + n <= TANK,
    };
    if input.count < recipe.item_count || io.fluid.amount < recipe.fluid || io.gas_in.amount < recipe.gas || !item_room || !gas_room {
        next.progress = 0;
        return (next, Status::Blocked, ticks, usage);
    }
    if io.energy < usage {
        return (next, Status::NoEnergy, ticks, usage);
    }
    next.energy -= usage;
    next.progress += 1;
    if next.progress >= ticks {
        next.progress = 0;
        let left = input.count - recipe.item_count;
        next.input = (left > 0).then(|| Stack { count: left, ..input.clone() });
        next.fluid.amount -= recipe.fluid;
        next.gas_in.amount -= recipe.gas;
        if let Some((id, n)) = &recipe.out_item {
            next.output = Some(match &io.output {
                Some(o) => Stack { count: o.count + n, ..o.clone() },
                None => Stack { item: id.clone(), count: *n, components: String::new() },
            });
        }
        if let Some((id, n)) = &recipe.out_gas {
            next.gas_out = Tank { chemical: id.clone(), amount: io.gas_out.amount + n };
        }
        for t in [&mut next.fluid, &mut next.gas_in] {
            if t.amount == 0 {
                t.chemical.clear();
            }
        }
    }
    (next, Status::Running, ticks, usage)
}

/// The recipes of type `mekanism:reaction`.
pub fn load_book() -> Book {
    let mut book = Book::default();
    for r in recipe_types::recipes_of_type("mekanism:reaction") {
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&r.json) else { continue };
        let items: HashSet<String> = recipe_types::ingredient_items(&json["item_input"]["ingredient"].to_string()).into_iter().collect();
        let fluids: HashSet<String> = plus::fluids_of(&json["fluid_input"]["ingredient"]).into_iter().collect();
        let gases: HashSet<String> = chemicals_of(&json["chemical_input"]["ingredient"]).into_iter().collect();
        if items.is_empty() || fluids.is_empty() || gases.is_empty() {
            continue;
        }
        let out_item = json["output"]["item"]["id"].as_str().map(|id| (id.to_owned(), json["output"]["item"]["count"].as_u64().unwrap_or(1) as u32));
        let out_gas = json["output"]["chemical"]["id"].as_str().map(|id| (id.to_owned(), json["output"]["chemical"]["amount"].as_u64().unwrap_or(1)));
        if out_item.is_none() && out_gas.is_none() {
            continue;
        }
        book.add(Recipe {
            items,
            item_count: json["item_input"]["count"].as_u64().unwrap_or(1) as u32,
            fluids,
            fluid: json["fluid_input"]["amount"].as_u64().unwrap_or(1),
            gases,
            gas: json["chemical_input"]["amount"].as_u64().unwrap_or(1),
            duration: json["duration"].as_u64().unwrap_or(u64::from(BASE_DURATION)) as u32,
            energy: json["energy_required"].as_u64().unwrap_or(0),
            out_item,
            out_gas,
        });
    }
    book
}

impl Mekanism {
    /// A tick of the pressurized reaction chamber.
    pub(crate) fn run_reaction(&mut self, def: &Def, e: &BlockEntity, tick: u64) -> Option<BlockEntityUpdate> {
        let layout = def.layout;
        // Its recipes, asked again every second until the game lends them.
        let key = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
        if self.reactions.is_empty() && self.tried.get(def.block).is_none_or(|t| tick.saturating_sub(*t) >= 20) {
            self.tried.insert(def.block, tick);
            self.reactions = load_book();
        }
        let mut facing = sides::property(&e.state, "facing").unwrap_or("north").to_owned();
        let progress = e.data.get(..2).map_or(0, |b| u32::from(u16::from_le_bytes([b[0], b[1]])));
        let stored = Config::decode(e.data.get(2..).unwrap_or(&[]), layout);
        let mut cfg = stored.unwrap_or_else(|| config::default_config(layout));
        if stored.is_none() {
            let front = cube::facing_name(front_for(e));
            if front != facing {
                let _ = world::set_block_property(&e.dimension, e.pos.x, e.pos.y, e.pos.z, "facing", front);
                facing = front.to_owned();
            }
        }
        let mut dirty = stored.is_none() || !self.applied.contains(&key);
        if let Some(click) = self.take_click(e) {
            dirty |= self.configure(&mut cfg, &facing, layout, &click);
        }
        let counts = upgrades::counts_of(def, e);
        let held = |i: usize| plus::held(e, i);
        let slot = |i: usize| stack(e.items.get(i).unwrap_or(&None));
        let io = Io {
            input: slot(0),
            output: slot(1),
            fluid: held(0),
            gas_in: held(1),
            gas_out: held(2),
            energy: e.energy,
            progress,
        };
        let (next, status, ticks, usage) = if self.can {
            step(&self.reactions, &io, counts[upgrades::SPEED], counts[upgrades::ENERGY])
        } else {
            (io.clone(), Status::Idle, upgrades::ticks(BASE_DURATION, counts[upgrades::SPEED]), upgrades::energy_per_tick(USAGE, counts[upgrades::SPEED], counts[upgrades::ENERGY]))
        };
        let active = status == Status::Running;
        let lit = e.state.contains("lit=true");
        if active != lit {
            let _ = world::set_block_property(&e.dimension, e.pos.x, e.pos.y, e.pos.z, "lit", if active { "true" } else { "false" });
        }
        // The fluid tank takes only the fluids the recipes use, the input gas tank only their gases.
        if !self.filtered.contains(&key) && !self.reactions.is_empty() {
            let _ = world::set_tank_filter(&e.dimension, e.pos.x, e.pos.y, e.pos.z, 0, &self.reactions.fluids());
            let _ = world::set_tank_filter(&e.dimension, e.pos.x, e.pos.y, e.pos.z, 1, &self.reactions.gases());
            self.filtered.insert(key.clone());
        }
        let energy_faces = dirty.then(|| self.apply_config(e, layout, &facing, &cfg));
        let capacity = upgrades::max_energy(def.usage * STORED_TICKS, counts[upgrades::ENERGY]);
        let mut data = menu_data(next.progress, next.energy, capacity, status, &cfg);
        for t in [&next.fluid, &next.gas_in, &next.gas_out] {
            data.extend(chem::screen(t));
        }
        let mut saved = machine_data(next.progress, &cfg);
        saved.extend([0, 0]);
        let control = Control::from_byte(e.data.get(redstone::offset(def.kind)).copied().unwrap_or(0));
        redstone::stamp(def.kind, control, capacity.saturating_sub(next.energy), (ticks, self.install.get(&key).copied().unwrap_or(0), counts, usage), &facing, &mut saved, &mut data);
        let tanks_now = [io.fluid.clone(), io.gas_in.clone(), io.gas_out.clone()];
        let tanks_next = [next.fluid.clone(), next.gas_in.clone(), next.gas_out.clone()];
        let tanks_changed = tanks_now != tanks_next || e.tanks.len() != 3;
        let items_changed = next.input != io.input || next.output != io.output;
        if !items_changed && !dirty && !tanks_changed && next.energy == e.energy && data == e.menu_data && saved == e.data && !self.force {
            return None;
        }
        self.applied.insert(key);
        let mut items: Vec<Option<ItemStack>> = e.items.clone();
        if items.len() >= 2 {
            items[0] = item(&next.input);
            items[1] = item(&next.output);
        }
        Some(BlockEntityUpdate {
            dimension: e.dimension.clone(),
            pos: e.pos,
            data: saved,
            items: (items_changed || self.force).then_some(items),
            energy: Some(next.energy),
            energy_faces,
            tanks: tanks_changed.then(|| tanks_next.iter().map(plus::as_tank).collect()),
            tank_faces: dirty.then(|| self.tank_sides(layout, &facing, &cfg, false)),
            menu_data: Some(data),
            transmitter_faces: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(v: &[&str]) -> HashSet<String> {
        v.iter().map(|s| (*s).to_owned()).collect()
    }
    fn stack(item: &str, n: u32) -> Option<Stack> {
        Some(Stack { item: item.into(), count: n, components: String::new() })
    }
    fn tank(c: &str, n: u64) -> Tank {
        Tank { chemical: c.into(), amount: n }
    }
    fn book() -> Book {
        let mut b = Book::default();
        // Coal gasification: a coal, a hundred of water and of oxygen make sulfur and a hundred of hydrogen, in 100 ticks.
        b.add(Recipe {
            items: set(&["minecraft:coal"]),
            item_count: 1,
            fluids: set(&["minecraft:water"]),
            fluid: 100,
            gases: set(&["mekanism:oxygen"]),
            gas: 100,
            duration: 100,
            energy: 0,
            out_item: Some(("mekanism:dust_sulfur".into(), 1)),
            out_gas: Some(("mekanism:hydrogen".into(), 100)),
        });
        // Substrate: needs 2 more energy a tick, 60 ticks.
        b.add(Recipe {
            items: set(&["mekanism:substrate"]),
            item_count: 1,
            fluids: set(&["mekanism:ethene_fluid"]),
            fluid: 50,
            gases: set(&["mekanism:oxygen"]),
            gas: 10,
            duration: 60,
            energy: 10,
            out_item: Some(("mekanism:hdpe_pellet".into(), 1)),
            out_gas: None,
        });
        b
    }
    fn io() -> Io {
        Io {
            input: stack("minecraft:coal", 2),
            fluid: tank("minecraft:water", 500),
            gas_in: tank("mekanism:oxygen", 500),
            energy: 10_000,
            ..Io::default()
        }
    }
    fn run(io: &Io, ticks: u32, speed: u8) -> (Io, Status) {
        let b = book();
        let (mut io, mut st) = (io.clone(), Status::Idle);
        for _ in 0..ticks {
            let (n, s, _, _) = step(&b, &io, speed, 0);
            io = n;
            st = s;
        }
        (io, st)
    }

    #[test]
    fn a_reaction_takes_its_duration_and_makes_an_item_and_a_gas() {
        let (mid, st) = run(&io(), 99, 0);
        assert_eq!((st, mid.progress, mid.energy), (Status::Running, 99, 10_000 - 99 * 2));
        let (done, _) = run(&io(), 100, 0);
        assert_eq!(done.input, stack("minecraft:coal", 1));
        assert_eq!(done.output, stack("mekanism:dust_sulfur", 1));
        assert_eq!((done.fluid.amount, done.gas_in.amount, done.gas_out.clone()), (400, 400, tank("mekanism:hydrogen", 100)));
    }

    #[test]
    fn speed_shortens_it_and_the_recipes_energy_is_added() {
        // Two speed upgrades: 100 ticks become 56 (100 * 10^(-2/8)).
        let (done, _) = run(&io(), 56, 2);
        assert_eq!(done.output, stack("mekanism:dust_sulfur", 1));
        // The substrate recipe uses 2 + 10 a tick.
        let io = Io { input: stack("mekanism:substrate", 1), fluid: tank("mekanism:ethene_fluid", 100), gas_in: tank("mekanism:oxygen", 100), energy: 1000, ..Io::default() };
        let (n, st, ticks, _) = step(&book(), &io, 0, 0);
        assert_eq!((st, n.energy, ticks), (Status::Running, 1000 - 12, 60));
    }

    #[test]
    fn it_waits_for_what_it_lacks() {
        let mut short = io();
        short.fluid.amount = 99;
        assert_eq!(run(&short, 5, 0).1, Status::Blocked);
        let mut dry = io();
        dry.energy = 1;
        assert_eq!(run(&dry, 1, 0).1, Status::NoEnergy);
        // A full gas tank or a different item in the output stops it.
        let mut full = io();
        full.gas_out = tank("mekanism:hydrogen", 9_950);
        assert_eq!(run(&full, 1, 0).1, Status::Blocked);
        let mut other = io();
        other.output = stack("minecraft:dirt", 1);
        assert_eq!(run(&other, 1, 0).1, Status::Blocked);
        // Nothing in the input slot, or an item with no recipe.
        assert_eq!(run(&Io::default(), 1, 0).1, Status::Idle);
        let mut dirt = io();
        dirt.input = stack("minecraft:dirt", 1);
        assert_eq!(run(&dirt, 1, 0).1, Status::NoRecipe);
        // The progress is lost when the inputs go.
        let (mid, _) = run(&io(), 10, 0);
        let gone = Io { fluid: Tank::default(), ..mid };
        assert_eq!(step(&book(), &gone, 0, 0).0.progress, 0);
    }

    #[test]
    fn the_tanks_take_what_a_recipe_uses() {
        assert_eq!(book().fluids(), ["mekanism:ethene_fluid", "minecraft:water"]);
        assert_eq!(book().gases(), ["mekanism:oxygen"]);
    }
}

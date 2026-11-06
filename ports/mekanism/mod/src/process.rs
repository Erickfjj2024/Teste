//! The machines that work an item and a chemical (Mekanism's infusion types, gases and slurries are all chemicals here):
//! the metallurgic infuser, the purification and injection chambers and the osmium compressor (an item and a chemical make
//! an item), the chemical oxidizer (an item makes a chemical), the chemical dissolution chamber (an item and a chemical
//! make a chemical) and the chemical crystallizer (a chemical makes an item). The pure rules, tested without the game.
use crate::machine::{STACK, Stack, Status};
use std::collections::{HashMap, HashSet};

/// What goes in and what comes out of a machine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// The infuser, the purification and injection chambers, the osmium compressor.
    ItemChemToItem,
    /// The chemical oxidizer.
    ItemToChem,
    /// The chemical dissolution chamber.
    ItemChemToChem,
    /// The chemical crystallizer.
    ChemToItem,
}

/// How a machine is built: its shape, ticks a recipe takes, what its tanks hold and how its chemical is used.
#[derive(Clone, Copy, Debug)]
pub struct Spec {
    pub shape: Shape,
    pub usage: u64,
    pub ticks: u32,
    /// What the tank the chemical goes in holds, and the one it comes out in.
    pub cap_in: u64,
    pub cap_out: u64,
    /// A per-tick recipe uses the chemical by chance (the "randomized consumption" of the config), a mean of one a tick;
    /// otherwise one a tick.
    pub random_use: bool,
    /// The mean of that chance (1, or what the speed and chemical upgrades make it).
    pub mean: f64,
}

/// What an output is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Out {
    Item { id: String, count: u32 },
    Chem { id: String, amount: u64 },
}

/// What a recipe takes and makes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recipe {
    pub item_count: u32,
    /// The chemical it needs in the tank: used up when it finishes, or (`per_tick`) at least this much to start and a
    /// little each tick while it runs.
    pub amount: u64,
    pub per_tick: bool,
    pub out: Out,
}

/// The recipes by their item and chemical (`""` where a shape has none); the first of two alike wins.
#[derive(Debug, Default)]
pub struct Book {
    by_pair: HashMap<(String, String), Recipe>,
    /// The items some recipe takes, and the chemicals some recipe uses (what the input tank accepts).
    items: HashSet<String>,
    chemicals: HashSet<String>,
    /// Item -> the chemical it turns into in the filling slot, and how much.
    conversions: HashMap<String, (String, u64)>,
}

impl Book {
    /// `items` or `chemicals` empty stand for "none" (the shape has no such input).
    pub fn add(&mut self, items: &[String], chemicals: &[String], recipe: Recipe) {
        let none = [String::new()];
        let (items, chemicals) = (
            if items.is_empty() { &none[..] } else { items },
            if chemicals.is_empty() { &none[..] } else { chemicals },
        );
        for i in items {
            self.items.insert(i.clone());
            for c in chemicals {
                self.chemicals.insert(c.clone());
                self.by_pair.entry((i.clone(), c.clone())).or_insert_with(|| recipe.clone());
            }
        }
    }
    pub fn add_conversion(&mut self, items: &[String], chemical: &str, amount: u64) {
        for i in items {
            self.conversions.entry(i.clone()).or_insert_with(|| (chemical.to_owned(), amount));
        }
    }
    pub fn is_empty(&self) -> bool {
        self.by_pair.is_empty()
    }
    /// The chemicals some recipe uses, in order (what the input tank takes in from outside).
    pub fn accepted(&self) -> Vec<String> {
        let mut list: Vec<String> = self.chemicals.iter().filter(|c| !c.is_empty()).cloned().collect();
        list.sort();
        list
    }
    pub fn recipe(&self, item: &str, chemical: &str) -> Option<&Recipe> {
        self.by_pair.get(&(item.to_owned(), chemical.to_owned()))
    }
    /// Whether some recipe of the item uses its chemical a little each tick (what waits when the tank runs dry).
    pub fn has_per_tick(&self, item: &str) -> bool {
        self.by_pair.iter().any(|((i, _), r)| i == item && r.per_tick)
    }
    /// Whether a chemical is one some recipe uses (the tank takes no other).
    pub fn accepts(&self, chemical: &str) -> bool {
        self.chemicals.contains(chemical)
    }
}

/// A tank: a chemical and how much of it; none is empty.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tank {
    pub chemical: String,
    pub amount: u64,
}

impl Tank {
    fn settle(&mut self) {
        if self.amount == 0 {
            self.chemical.clear();
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Io {
    /// The item input, the item output and the filling slot (a chemical item turned into the input tank's chemical).
    pub input: Option<Stack>,
    pub output: Option<Stack>,
    pub conv: Option<Stack>,
    pub tank_in: Tank,
    pub tank_out: Tank,
    pub energy: u64,
    pub progress: u32,
}

/// How much of the chemical a tick uses (`StatUtils.inversePoisson`): 0 up to 3 times the mean's ceiling, the chance of a
/// number following a Poisson curve; `r` is a roll from 0 up to 1.
pub fn inverse_poisson(mean: f64, r: f64) -> u32 {
    let target = r * mean.exp();
    let stirling = mean * std::f64::consts::E;
    let bound = 3.0 * mean.ceil();
    let coeff = 1.0 / (2.0 * std::f64::consts::PI).sqrt();
    let (mut m, mut p) = (0u32, 1.0f64);
    while p < target && f64::from(m) < bound {
        m += 1;
        p += coeff / f64::from(m).sqrt() * (stirling / f64::from(m)).powi(m as i32);
    }
    m
}

/// One tick. The filling slot's item is turned into the input tank's chemical when it fits (one item a tick); the machine
/// then runs when its inputs have a recipe, there is enough of them, room for what it makes and energy for the tick. A
/// per-tick recipe uses some of the chemical each tick and waits (without losing its progress) when the tank is short.
pub fn step(spec: &Spec, book: &Book, io: &Io, dice: f64) -> (Io, Status) {
    let mut next = io.clone();
    if let Some(slot) = &io.conv
        && let Some((chemical, amount)) = book.conversions.get(&slot.item)
        && book.accepts(chemical)
        && (io.tank_in.amount == 0 || io.tank_in.chemical == *chemical)
        && io.tank_in.amount + amount <= spec.cap_in
    {
        next.tank_in = Tank { chemical: chemical.clone(), amount: io.tank_in.amount + amount };
        next.conv = (slot.count > 1).then(|| Stack { count: slot.count - 1, ..slot.clone() });
    }
    if book.is_empty() {
        next.progress = 0;
        return (next, Status::NoBook);
    }
    let needs_item = spec.shape != Shape::ChemToItem;
    let needs_chem = spec.shape != Shape::ItemToChem;
    let input = io.input.as_ref().filter(|_| needs_item);
    if needs_item {
        let Some(input) = input else {
            next.progress = 0;
            return (next, Status::Idle);
        };
        if !book.items.contains(&input.item) {
            next.progress = 0;
            return (next, Status::NoRecipe);
        }
    } else if next.tank_in.amount == 0 {
        next.progress = 0;
        return (next, Status::Idle);
    }
    let item = input.map_or("", |s| s.item.as_str());
    let chem = if needs_chem { next.tank_in.chemical.as_str() } else { "" };
    let found = book.recipe(item, chem).filter(|_| !needs_chem || next.tank_in.amount > 0);
    let Some(recipe) = found else {
        // A per-tick recipe whose tank ran dry waits where it was (`ItemStackConstantChemicalToObjectCachedRecipe` does not
        // force a reset on the secondary input); otherwise the item has recipes but not with this chemical, or none at
        // all: short of the second input.
        let dry = needs_chem && next.tank_in.amount == 0 && io.progress > 0 && input.is_some_and(|s| book.has_per_tick(&s.item));
        if !dry {
            next.progress = 0;
        }
        return (next, if needs_item { Status::Blocked } else { Status::NoRecipe });
    };
    let room = match &recipe.out {
        Out::Item { id, count } => match &io.output {
            None => *count <= STACK,
            Some(o) => o.item == *id && o.components.is_empty() && o.count + count <= STACK,
        },
        Out::Chem { id, amount } => {
            (io.tank_out.amount == 0 || io.tank_out.chemical == *id) && io.tank_out.amount + amount <= spec.cap_out
        }
    };
    let item_ok = input.is_none_or(|s| s.count >= recipe.item_count);
    if !item_ok || (needs_chem && next.tank_in.amount < recipe.amount) || !room {
        next.progress = 0;
        return (next, Status::Blocked);
    }
    // What a per-tick recipe uses this tick.
    let uses = if needs_chem && recipe.per_tick {
        if spec.random_use { inverse_poisson(spec.mean, dice) } else { 1 }
    } else {
        0
    };
    if next.tank_in.amount < u64::from(uses) {
        // Short of the chemical for this tick: it waits where it is.
        return (next, Status::Blocked);
    }
    if io.energy < spec.usage {
        return (next, Status::NoEnergy);
    }
    next.energy -= spec.usage;
    next.progress += 1;
    next.tank_in.amount -= u64::from(uses);
    if next.progress >= spec.ticks {
        next.progress = 0;
        if let Some(input) = input {
            let left = input.count - recipe.item_count;
            next.input = (left > 0).then(|| Stack { count: left, ..input.clone() });
        }
        if needs_chem && !recipe.per_tick {
            next.tank_in.amount -= recipe.amount;
        }
        match &recipe.out {
            Out::Item { id, count } => {
                next.output = Some(match &io.output {
                    Some(o) => Stack { count: o.count + count, ..o.clone() },
                    None => Stack { item: id.clone(), count: *count, components: String::new() },
                });
            }
            Out::Chem { id, amount } => {
                next.tank_out = Tank { chemical: id.clone(), amount: io.tank_out.amount + amount };
            }
        }
    }
    next.tank_in.settle();
    (next, Status::Running)
}

#[cfg(test)]
mod tests {
    use super::*;

    const INFUSER: Spec = Spec { shape: Shape::ItemChemToItem, usage: 5, ticks: 200, cap_in: 1000, cap_out: 0, random_use: false, mean: 1.0 };
    const PURIFIER: Spec = Spec { shape: Shape::ItemChemToItem, usage: 10, ticks: 200, cap_in: 210, cap_out: 0, random_use: false, mean: 1.0 };
    const OXIDIZER: Spec = Spec { shape: Shape::ItemToChem, usage: 15, ticks: 100, cap_in: 0, cap_out: 10_000, random_use: false, mean: 1.0 };
    const DISSOLVER: Spec = Spec { shape: Shape::ItemChemToChem, usage: 16, ticks: 100, cap_in: 10_000, cap_out: 10_000, random_use: false, mean: 1.0 };
    const CRYSTALLIZER: Spec = Spec { shape: Shape::ChemToItem, usage: 15, ticks: 200, cap_in: 10_000, cap_out: 0, random_use: false, mean: 1.0 };

    fn stack(item: &str, count: u32) -> Option<Stack> {
        Some(Stack { item: item.into(), count, components: String::new() })
    }
    fn s(v: &str) -> Vec<String> {
        vec![v.to_owned()]
    }
    fn tank(c: &str, n: u64) -> Tank {
        Tank { chemical: c.into(), amount: n }
    }
    fn item(id: &str) -> Out {
        Out::Item { id: id.into(), count: 1 }
    }
    fn book() -> Book {
        let mut b = Book::default();
        b.add(&s("minecraft:iron_ingot"), &s("mekanism:redstone"), Recipe { item_count: 1, amount: 10, per_tick: false, out: item("mekanism:alloy_infused") });
        b.add(&s("minecraft:gold_ore"), &s("mekanism:oxygen"), Recipe { item_count: 1, amount: 1, per_tick: true, out: Out::Item { id: "mekanism:clump_gold".into(), count: 3 } });
        b.add(&s("mekanism:dust_salt"), &[], Recipe { item_count: 1, amount: 0, per_tick: false, out: Out::Chem { id: "mekanism:brine".into(), amount: 15 } });
        b.add(&s("minecraft:gold_ore"), &s("mekanism:sulfuric_acid"), Recipe { item_count: 1, amount: 1, per_tick: true, out: Out::Chem { id: "mekanism:dirty_gold".into(), amount: 1000 } });
        b.add(&[], &s("mekanism:clean_gold"), Recipe { item_count: 0, amount: 200, per_tick: false, out: item("mekanism:crystal_gold") });
        b.add_conversion(&s("minecraft:redstone"), "mekanism:redstone", 10);
        b.add_conversion(&s("minecraft:coal"), "mekanism:carbon", 10);
        b
    }
    fn run(spec: &Spec, io: &Io, ticks: u32) -> (Io, Status) {
        let b = book();
        let mut io = io.clone();
        let mut status = Status::Idle;
        for _ in 0..ticks {
            let (n, st) = step(spec, &b, &io, 0.5);
            io = n;
            status = st;
        }
        (io, status)
    }

    #[test]
    fn an_item_in_the_filling_slot_fills_the_tank() {
        let io = Io { conv: stack("minecraft:redstone", 3), energy: 100, ..Io::default() };
        let (n, _) = run(&INFUSER, &io, 2);
        assert_eq!(n.tank_in, tank("mekanism:redstone", 20));
        assert_eq!(n.conv, stack("minecraft:redstone", 1));
        let mixed = Io { conv: stack("minecraft:coal", 1), tank_in: tank("mekanism:redstone", 5), ..Io::default() };
        assert_eq!(run(&INFUSER, &mixed, 1).0.conv, stack("minecraft:coal", 1));
        let none = Io { conv: stack("minecraft:coal", 1), ..Io::default() };
        assert_eq!(run(&INFUSER, &none, 1).0.conv, stack("minecraft:coal", 1), "carbon is no recipe's");
    }

    #[test]
    fn the_tank_holds_what_its_machine_says() {
        let io = Io { conv: stack("minecraft:redstone", 2), tank_in: tank("mekanism:redstone", 995), ..Io::default() };
        assert_eq!(run(&INFUSER, &io, 3).0.tank_in.amount, 995);
        let io = Io { conv: stack("minecraft:redstone", 2), tank_in: tank("mekanism:redstone", 205), ..Io::default() };
        assert_eq!(run(&PURIFIER, &io, 3).0.tank_in.amount, 205, "210 is the most");
    }

    #[test]
    fn an_item_and_a_chemical_make_an_item_using_the_chemical_at_the_end() {
        let io = Io { input: stack("minecraft:iron_ingot", 2), tank_in: tank("mekanism:redstone", 25), energy: 10_000, ..Io::default() };
        let (done, status) = run(&INFUSER, &io, 200);
        assert_eq!(status, Status::Running);
        assert_eq!(done.output, stack("mekanism:alloy_infused", 1));
        assert_eq!(done.input, stack("minecraft:iron_ingot", 1));
        assert_eq!(done.tank_in.amount, 15);
        assert_eq!(done.energy, 10_000 - 5 * 200);
        let empty = Io { input: stack("minecraft:iron_ingot", 1), tank_in: tank("mekanism:redstone", 10), energy: 10_000, ..Io::default() };
        assert_eq!(run(&INFUSER, &empty, 200).0.tank_in, Tank::default());
    }

    #[test]
    fn a_per_tick_recipe_uses_a_little_each_tick_and_waits_when_short() {
        let io = Io { input: stack("minecraft:gold_ore", 1), tank_in: tank("mekanism:oxygen", 250), energy: 10_000, ..Io::default() };
        let (half, _) = run(&PURIFIER, &io, 100);
        assert_eq!(half.tank_in.amount, 150, "one a tick");
        let (done, _) = run(&PURIFIER, &io, 200);
        assert_eq!(done.output, Some(Stack { item: "mekanism:clump_gold".into(), count: 3, components: String::new() }));
        assert_eq!(done.tank_in.amount, 50);
        // Short of the chemical: it waits where it was, with its progress.
        let dry = Io { progress: 40, tank_in: tank("mekanism:oxygen", 0), ..io.clone() };
        let (n, st) = run(&PURIFIER, &dry, 3);
        assert_eq!((n.progress, st), (40, Status::Blocked), "an empty tank keeps the progress");
        let idle = Io { progress: 0, tank_in: Tank::default(), ..io.clone() };
        assert_eq!(run(&PURIFIER, &idle, 3).0.progress, 0);
        let one = Io { progress: 40, tank_in: tank("mekanism:oxygen", 1), ..io.clone() };
        let (n, _) = run(&PURIFIER, &one, 5);
        assert_eq!(n.progress, 41, "it runs on the one it had, then waits with its progress");
        // The tank running dry keeps the progress of a per-tick recipe; another chemical in it starts over.
        let ran_dry = Io { progress: 40, tank_in: Tank::default(), ..io.clone() };
        let (n, st) = run(&PURIFIER, &ran_dry, 3);
        assert_eq!((n.progress, st), (40, Status::Blocked));
        let other = Io { progress: 40, tank_in: tank("mekanism:hydrogen", 50), ..io };
        assert_eq!(run(&PURIFIER, &other, 1).0.progress, 0);
    }

    #[test]
    fn a_random_use_follows_the_roll() {
        let spec = Spec { random_use: true, ..PURIFIER };
        let b = book();
        let io = Io { input: stack("minecraft:gold_ore", 1), tank_in: tank("mekanism:oxygen", 100), energy: 1000, ..Io::default() };
        // The same roll gives the same use; a low one nothing, a high one a few.
        let low = step(&spec, &b, &io, 0.0).0.tank_in.amount;
        let high = step(&spec, &b, &io, 0.999).0.tank_in.amount;
        assert_eq!(low, 100);
        assert!(high < 100 && high >= 97);
        assert_eq!(inverse_poisson(1.0, 0.0), 0);
        assert!((1..=3).contains(&inverse_poisson(1.0, 0.99)));
    }

    #[test]
    fn an_item_makes_a_chemical_into_the_output_tank() {
        let io = Io { input: stack("mekanism:dust_salt", 2), energy: 10_000, ..Io::default() };
        let (done, _) = run(&OXIDIZER, &io, 100);
        assert_eq!(done.tank_out, tank("mekanism:brine", 15));
        assert_eq!(done.input, stack("mekanism:dust_salt", 1));
        // Another chemical in the output tank, or no room, stops it.
        let other = Io { tank_out: tank("mekanism:oxygen", 5), ..io.clone() };
        assert_eq!(run(&OXIDIZER, &other, 1).1, Status::Blocked);
        let full = Io { tank_out: tank("mekanism:brine", 9_990), ..io };
        assert_eq!(run(&OXIDIZER, &full, 1).1, Status::Blocked);
    }

    #[test]
    fn an_item_and_a_chemical_make_a_chemical() {
        let io = Io { input: stack("minecraft:gold_ore", 1), tank_in: tank("mekanism:sulfuric_acid", 150), energy: 10_000, ..Io::default() };
        let (done, _) = run(&DISSOLVER, &io, 100);
        assert_eq!(done.tank_out, tank("mekanism:dirty_gold", 1000));
        assert_eq!(done.tank_in.amount, 50);
        assert_eq!(done.input, None);
    }

    #[test]
    fn a_chemical_makes_an_item() {
        let io = Io { tank_in: tank("mekanism:clean_gold", 450), energy: 10_000, ..Io::default() };
        let (done, _) = run(&CRYSTALLIZER, &io, 200);
        assert_eq!(done.output, stack("mekanism:crystal_gold", 1));
        assert_eq!(done.tank_in.amount, 250);
        let short = Io { tank_in: tank("mekanism:clean_gold", 199), energy: 10_000, ..Io::default() };
        assert_eq!(run(&CRYSTALLIZER, &short, 1).1, Status::Blocked);
        assert_eq!(run(&CRYSTALLIZER, &Io::default(), 1).1, Status::Idle);
        let wrong = Io { tank_in: tank("mekanism:oxygen", 300), energy: 10_000, ..Io::default() };
        assert_eq!(run(&CRYSTALLIZER, &wrong, 1).1, Status::NoRecipe);
    }

    #[test]
    fn it_waits_for_enough_chemical_a_recipe_energy_and_room() {
        let b = |amount| Io { input: stack("minecraft:iron_ingot", 1), tank_in: tank("mekanism:redstone", amount), energy: 100, ..Io::default() };
        assert_eq!(run(&INFUSER, &b(9), 1).1, Status::Blocked, "10 are needed");
        let none = Io { input: stack("minecraft:iron_ingot", 1), energy: 100, ..Io::default() };
        assert_eq!(run(&INFUSER, &none, 1).1, Status::Blocked, "an empty tank");
        let stick = Io { input: stack("minecraft:stick", 1), energy: 100, ..Io::default() };
        assert_eq!(run(&INFUSER, &stick, 1).1, Status::NoRecipe);
        assert_eq!(run(&INFUSER, &Io::default(), 1).1, Status::Idle);
        assert_eq!(run(&INFUSER, &Io { energy: 4, ..b(10) }, 1).1, Status::NoEnergy);
        let full = Io { output: stack("mekanism:alloy_infused", 64), ..b(10) };
        assert_eq!(run(&INFUSER, &full, 1).1, Status::Blocked);
        assert_eq!(step(&INFUSER, &Book::default(), &b(10), 0.5).1, Status::NoBook);
    }
}

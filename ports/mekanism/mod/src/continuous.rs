//! The machines that work chemicals and fluids in their tanks a little every tick, with no item and no progress: the
//! chemical infuser (two chemicals make a third), the chemical washer (a fluid and a slurry make a clean slurry) and the
//! electrolytic separator (a fluid makes two gases). The pure rules, tested without the game.
use crate::machine::Status;
use crate::process::Tank;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// Tanks: left, right, centre (out). Either side takes either input.
    Infuser,
    /// Tanks: the fluid, the slurry, the clean slurry (out).
    Washer,
    /// Tanks: the fluid, the left gas (out), the right gas (out).
    Separator,
    /// The rotary condensentrator: tanks fluid, gas. One of them takes in and the other gives, by its mode.
    Rotary,
}

#[derive(Clone, Copy, Debug)]
pub struct Spec {
    pub shape: Shape,
    pub usage: u64,
    /// What each of the three tanks holds.
    pub caps: [u64; 3],
}

/// What one operation (one tick) takes from the two inputs and gives to the outputs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recipe {
    pub a: u64,
    pub b: u64,
    pub outs: Vec<(String, u64)>,
}

#[derive(Debug, Default)]
pub struct Book {
    by_pair: HashMap<(String, String), Recipe>,
    firsts: HashSet<String>,
    seconds: HashSet<String>,
}

impl Book {
    /// A recipe of inputs `a` and `b` (`b` empty for one input). With `either`, the inputs may sit on either side.
    pub fn add(&mut self, a: &[String], b: &[String], recipe: Recipe, either: bool) {
        let none = [String::new()];
        let b = if b.is_empty() { &none[..] } else { b };
        for x in a {
            for y in b {
                self.firsts.insert(x.clone());
                self.seconds.insert(y.clone());
                self.by_pair.entry((x.clone(), y.clone())).or_insert_with(|| recipe.clone());
                if either && x != y {
                    self.firsts.insert(y.clone());
                    self.seconds.insert(x.clone());
                    let swapped = Recipe { a: recipe.b, b: recipe.a, outs: recipe.outs.clone() };
                    self.by_pair.entry((y.clone(), x.clone())).or_insert(swapped);
                }
            }
        }
    }
    pub fn is_empty(&self) -> bool {
        self.by_pair.is_empty()
    }
    /// What the first (0) or second (1) input tank takes in from outside.
    pub fn accepted(&self, input: usize) -> Vec<String> {
        let set = if input == 0 { &self.firsts } else { &self.seconds };
        let mut v: Vec<String> = set.iter().filter(|s| !s.is_empty()).cloned().collect();
        v.sort();
        v
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Io {
    pub tanks: [Tank; 3],
    pub energy: u64,
    /// The rotary condensentrator's mode: false condensentrates (gas to fluid), true decondensentrates (fluid to gas).
    pub mode: bool,
    /// The most operations a tick (the speed upgrades: 2 to their number); 0 is one.
    pub ops: u32,
}

fn room(tank: &Tank, cap: u64, id: &str, amount: u64) -> bool {
    (tank.amount == 0 || tank.chemical == id) && tank.amount + amount <= cap
}

fn put(tank: &mut Tank, id: &str, amount: u64) {
    tank.chemical = id.to_owned();
    tank.amount += amount;
}

/// One tick: the inputs have a recipe and enough of each, the outputs have room and there is energy for the tick; the
/// recipe's amounts are taken and its outputs added.
pub fn step(spec: &Spec, book: &Book, io: &Io) -> (Io, Status) {
    let mut next = io.clone();
    if book.is_empty() {
        return (next, Status::NoBook);
    }
    let [t0, t1, _] = &io.tanks;
    // The tank the recipe reads (and, for the rotary, the one it writes).
    let in_tank = if spec.shape == Shape::Rotary && !io.mode { 1 } else { 0 };
    let (first, second) = match spec.shape {
        Shape::Separator => (t0, None),
        Shape::Rotary => (&io.tanks[in_tank], None),
        _ => (t0, Some(t1)),
    };
    if first.amount == 0 || second.is_some_and(|t| t.amount == 0) {
        return (next, Status::Idle);
    }
    let key = (first.chemical.clone(), second.map_or_else(String::new, |t| t.chemical.clone()));
    let Some(recipe) = book.by_pair.get(&key) else {
        return (next, Status::NoRecipe);
    };
    // Where the outputs go: the centre tank, or the two gases.
    let out_tanks: Vec<usize> = match spec.shape {
        Shape::Separator => vec![1, 2],
        Shape::Rotary => vec![1 - in_tank],
        _ => vec![2],
    };
    let fits = recipe.outs.iter().zip(&out_tanks).all(|((id, n), t)| room(&io.tanks[*t], spec.caps[*t], id, *n));
    if first.amount < recipe.a || second.is_some_and(|t| t.amount < recipe.b) || !fits {
        return (next, Status::Blocked);
    }
    if io.energy < spec.usage {
        return (next, Status::NoEnergy);
    }
    // How many operations this tick: the most it may, what the inputs, the room and the energy allow.
    let mut times = u64::from(io.ops.max(1)).min(first.amount / recipe.a.max(1));
    if let Some(t) = second {
        times = times.min(t.amount / recipe.b.max(1));
    }
    for ((id, n), t) in recipe.outs.iter().zip(&out_tanks) {
        let tank = &io.tanks[*t];
        let space = if tank.amount == 0 || tank.chemical == *id { spec.caps[*t].saturating_sub(tank.amount) } else { 0 };
        times = times.min(space / (*n).max(1));
    }
    if spec.usage > 0 {
        times = times.min(io.energy / spec.usage);
    }
    let times = times.max(1);
    next.energy -= spec.usage * times;
    next.tanks[in_tank].amount -= recipe.a * times;
    if second.is_some() {
        next.tanks[1].amount -= recipe.b * times;
    }
    for ((id, n), t) in recipe.outs.iter().zip(&out_tanks) {
        put(&mut next.tanks[*t], id, *n * times);
    }
    for t in &mut next.tanks {
        if t.amount == 0 {
            t.chemical.clear();
        }
    }
    (next, Status::Running)
}

#[cfg(test)]
mod tests {
    use super::*;

    const INFUSER: Spec = Spec { shape: Shape::Infuser, usage: 15, caps: [10_000; 3] };
    const WASHER: Spec = Spec { shape: Shape::Washer, usage: 5, caps: [10_000; 3] };
    const SEPARATOR: Spec = Spec { shape: Shape::Separator, usage: 4, caps: [24_000, 2_400, 2_400] };
    const ROTARY: Spec = Spec { shape: Shape::Rotary, usage: 2, caps: [10_000, 10_000, 0] };

    fn t(c: &str, n: u64) -> Tank {
        Tank { chemical: c.into(), amount: n }
    }
    fn s(v: &str) -> Vec<String> {
        vec![v.to_owned()]
    }
    fn book() -> Book {
        let mut b = Book::default();
        b.add(&s("mekanism:hydrogen"), &s("mekanism:chlorine"), Recipe { a: 1, b: 1, outs: vec![("mekanism:hydrogen_chloride".into(), 1)] }, true);
        b.add(&s("minecraft:water"), &s("mekanism:dirty_gold"), Recipe { a: 5, b: 1, outs: vec![("mekanism:clean_gold".into(), 1)] }, false);
        b.add(&s("minecraft:water"), &[], Recipe { a: 2, b: 0, outs: vec![("mekanism:hydrogen".into(), 2), ("mekanism:oxygen".into(), 1)] }, false);
        b
    }

    #[test]
    fn two_chemicals_make_a_third_on_either_side() {
        let io = Io { tanks: [t("mekanism:hydrogen", 5), t("mekanism:chlorine", 3), Tank::default()], energy: 100, ..Io::default() };
        let (n, st) = step(&INFUSER, &book(), &io);
        assert_eq!(st, Status::Running);
        assert_eq!(n.tanks, [t("mekanism:hydrogen", 4), t("mekanism:chlorine", 2), t("mekanism:hydrogen_chloride", 1)]);
        assert_eq!(n.energy, 85);
        // The same with the inputs the other way round.
        let swapped = Io { tanks: [t("mekanism:chlorine", 3), t("mekanism:hydrogen", 5), Tank::default()], energy: 100, ..Io::default() };
        let (n, st) = step(&INFUSER, &book(), &swapped);
        assert_eq!((st, n.tanks[2].amount), (Status::Running, 1));
        // An empty tank clears its chemical.
        let one = Io { tanks: [t("mekanism:hydrogen", 1), t("mekanism:chlorine", 1), Tank::default()], energy: 100, ..Io::default() };
        assert_eq!(step(&INFUSER, &book(), &one).0.tanks[0], Tank::default());
    }

    #[test]
    fn a_washer_uses_five_of_fluid_for_one_of_slurry() {
        let io = Io { tanks: [t("minecraft:water", 12), t("mekanism:dirty_gold", 2), Tank::default()], energy: 100, ..Io::default() };
        let (n, st) = step(&WASHER, &book(), &io);
        assert_eq!(st, Status::Running);
        assert_eq!(n.tanks, [t("minecraft:water", 7), t("mekanism:dirty_gold", 1), t("mekanism:clean_gold", 1)]);
        let dry = Io { tanks: [t("minecraft:water", 4), t("mekanism:dirty_gold", 2), Tank::default()], energy: 100, ..Io::default() };
        assert_eq!(step(&WASHER, &book(), &dry).1, Status::Blocked);
    }

    #[test]
    fn a_separator_splits_water_into_two_gases() {
        let io = Io { tanks: [t("minecraft:water", 10), Tank::default(), Tank::default()], energy: 100, ..Io::default() };
        let (n, st) = step(&SEPARATOR, &book(), &io);
        assert_eq!(st, Status::Running);
        assert_eq!(n.tanks, [t("minecraft:water", 8), t("mekanism:hydrogen", 2), t("mekanism:oxygen", 1)]);
        // A full gas tank stops it.
        let full = Io { tanks: [t("minecraft:water", 10), t("mekanism:hydrogen", 2_399), Tank::default()], energy: 100, ..Io::default() };
        assert_eq!(step(&SEPARATOR, &book(), &full).1, Status::Blocked);
    }

    #[test]
    fn a_rotary_condensentrator_turns_gas_to_fluid_and_back_by_its_mode() {
        let mut gas_book = Book::default();
        gas_book.add(&s("mekanism:water_vapor"), &[], Recipe { a: 1, b: 0, outs: vec![("minecraft:water".into(), 1)] }, false);
        let mut fluid_book = Book::default();
        fluid_book.add(&s("minecraft:water"), &[], Recipe { a: 1, b: 0, outs: vec![("mekanism:water_vapor".into(), 1)] }, false);
        // Mode false: the gas tank (1) gives to the fluid tank (0).
        let io = Io { tanks: [Tank::default(), t("mekanism:water_vapor", 5), Tank::default()], energy: 10, mode: false, ops: 0 };
        let (n, st) = step(&ROTARY, &gas_book, &io);
        assert_eq!(st, Status::Running);
        assert_eq!((n.tanks[0].clone(), n.tanks[1].clone(), n.energy), (t("minecraft:water", 1), t("mekanism:water_vapor", 4), 8));
        // Mode true: the fluid tank gives to the gas tank.
        let io = Io { tanks: [t("minecraft:water", 5), Tank::default(), Tank::default()], energy: 10, mode: true, ops: 0 };
        let (n, st) = step(&ROTARY, &fluid_book, &io);
        assert_eq!((st, n.tanks[0].amount, n.tanks[1].clone()), (Status::Running, 4, t("mekanism:water_vapor", 1)));
        // Nothing to take in the tank this mode reads: idle.
        let io = Io { tanks: [t("minecraft:water", 5), Tank::default(), Tank::default()], energy: 10, mode: false, ops: 0 };
        assert_eq!(step(&ROTARY, &gas_book, &io).1, Status::Idle);
        // The output tank full stops it.
        let io = Io { tanks: [t("minecraft:water", 10_000), t("mekanism:water_vapor", 5), Tank::default()], energy: 10, mode: false, ops: 0 };
        assert_eq!(step(&ROTARY, &gas_book, &io).1, Status::Blocked);
    }

    #[test]
    fn it_needs_a_recipe_energy_and_room() {
        let io = Io { tanks: [t("mekanism:hydrogen", 5), t("mekanism:oxygen", 3), Tank::default()], energy: 100, ..Io::default() };
        assert_eq!(step(&INFUSER, &book(), &io).1, Status::NoRecipe);
        assert_eq!(step(&INFUSER, &book(), &Io::default()).1, Status::Idle);
        let dry = Io { tanks: [t("mekanism:hydrogen", 5), t("mekanism:chlorine", 3), Tank::default()], energy: 14, ..Io::default() };
        assert_eq!(step(&INFUSER, &book(), &dry).1, Status::NoEnergy);
        let other = Io { tanks: [t("mekanism:hydrogen", 5), t("mekanism:chlorine", 3), t("mekanism:oxygen", 1)], energy: 100, ..Io::default() };
        assert_eq!(step(&INFUSER, &book(), &other).1, Status::Blocked);
        assert_eq!(step(&INFUSER, &Book::default(), &io).1, Status::NoBook);
        assert_eq!(book().accepted(0), ["mekanism:chlorine", "mekanism:hydrogen", "minecraft:water"]);
    }
}

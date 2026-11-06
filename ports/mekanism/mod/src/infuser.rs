//! The metallurgic infuser (`TileEntityMetallurgicInfuser`): an item and an infusion type (the chemical in its tank) make
//! another item, and an item in its infusion slot turns into infusion type (`mekanism:chemical_conversion`). The pure rules,
//! tested without the game.
use crate::machine::{STACK, Stack, Status, TICKS};
use std::collections::{HashMap, HashSet};

/// What its tank holds at most (`TileEntityMetallurgicInfuser.MAX_INFUSE`, a bucket).
pub const MAX_INFUSE: u64 = 1000;

/// What a recipe takes and makes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recipe {
    pub item_count: u32,
    /// Infusion type used up when it finishes (`per_tick_usage` is false in every recipe of the mod).
    pub amount: u64,
    pub output: String,
    pub out_count: u32,
}

/// The recipes by their item and infusion type; the first of two alike wins.
#[derive(Debug, Default)]
pub struct Book {
    by_pair: HashMap<(String, String), Recipe>,
    /// The items some recipe takes, and the infusion types some recipe uses (what the tank accepts).
    items: HashSet<String>,
    chemicals: HashSet<String>,
    /// Item -> the infusion type it turns into, and how much.
    conversions: HashMap<String, (String, u64)>,
}

impl Book {
    pub fn add(&mut self, items: &[String], chemicals: &[String], recipe: Recipe) {
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
    pub fn recipe(&self, item: &str, chemical: &str) -> Option<&Recipe> {
        self.by_pair.get(&(item.to_owned(), chemical.to_owned()))
    }
    /// Whether an infusion type is one some recipe uses (the tank takes no other).
    pub fn accepts(&self, chemical: &str) -> bool {
        self.chemicals.contains(chemical)
    }
}

/// The tank: an infusion type and how much of it; none is empty.
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
    pub input: Option<Stack>,
    pub output: Option<Stack>,
    /// The infusion slot.
    pub infusion: Option<Stack>,
    pub tank: Tank,
    pub energy: u64,
    pub progress: u32,
}

/// One tick: the infusion slot's item is turned into infusion type when it fits the tank (`fillTankOrConvert`, one item a
/// tick), then the machine runs when the item and the infusion type have a recipe, there is enough of both, room for what
/// it makes and energy for the tick; a finished recipe uses its item and its infusion type up at once.
pub fn step(usage: u64, book: &Book, io: &Io) -> (Io, Status) {
    let mut next = io.clone();
    // The infusion slot.
    if let Some(slot) = &io.infusion
        && let Some((chemical, amount)) = book.conversions.get(&slot.item)
        && book.accepts(chemical)
        && (io.tank.amount == 0 || io.tank.chemical == *chemical)
        && io.tank.amount + amount <= MAX_INFUSE
    {
        next.tank = Tank { chemical: chemical.clone(), amount: io.tank.amount + amount };
        next.infusion = (slot.count > 1).then(|| Stack { count: slot.count - 1, ..slot.clone() });
    }
    if book.is_empty() {
        next.progress = 0;
        return (next, Status::NoBook);
    }
    let Some(input) = io.input.as_ref() else {
        next.progress = 0;
        return (next, Status::Idle);
    };
    if !book.items.contains(&input.item) {
        next.progress = 0;
        return (next, Status::NoRecipe);
    }
    // The tank as it is after the conversion above: a recipe needs what is in it now.
    let tank = &next.tank;
    let Some(recipe) = book.recipe(&input.item, &tank.chemical).filter(|_| tank.amount > 0) else {
        // The item has recipes but not with this infusion type (or the tank is empty): short of the second input.
        next.progress = 0;
        return (next, Status::Blocked);
    };
    let room = match &io.output {
        None => recipe.out_count <= STACK,
        Some(o) => o.item == recipe.output && o.components.is_empty() && o.count + recipe.out_count <= STACK,
    };
    if input.count < recipe.item_count || tank.amount < recipe.amount || !room {
        next.progress = 0;
        return (next, Status::Blocked);
    }
    if io.energy < usage {
        return (next, Status::NoEnergy);
    }
    next.energy -= usage;
    next.progress += 1;
    if next.progress >= TICKS {
        next.progress = 0;
        let left = input.count - recipe.item_count;
        next.input = (left > 0).then(|| Stack { count: left, ..input.clone() });
        next.tank.amount -= recipe.amount;
        next.tank.settle();
        next.output = Some(match &io.output {
            Some(o) => Stack { count: o.count + recipe.out_count, ..o.clone() },
            None => Stack { item: recipe.output.clone(), count: recipe.out_count, components: String::new() },
        });
    }
    (next, Status::Running)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(item: &str, count: u32) -> Option<Stack> {
        Some(Stack { item: item.into(), count, components: String::new() })
    }
    fn book() -> Book {
        let mut b = Book::default();
        b.add(
            &["minecraft:iron_ingot".to_owned()],
            &["mekanism:redstone".to_owned()],
            Recipe { item_count: 1, amount: 10, output: "mekanism:alloy_infused".into(), out_count: 1 },
        );
        b.add_conversion(&["minecraft:redstone".to_owned()], "mekanism:redstone", 10);
        b.add_conversion(&["minecraft:coal".to_owned()], "mekanism:carbon", 10);
        b
    }
    fn run(io: &Io, ticks: u32) -> (Io, Status) {
        let b = book();
        let mut io = io.clone();
        let mut status = Status::Idle;
        for _ in 0..ticks {
            let (n, s) = step(5, &b, &io);
            io = n;
            status = s;
        }
        (io, status)
    }

    #[test]
    fn an_item_in_the_infusion_slot_fills_the_tank() {
        let io = Io { infusion: stack("minecraft:redstone", 3), energy: 100, ..Io::default() };
        let (n, _) = run(&io, 2);
        assert_eq!(n.tank, Tank { chemical: "mekanism:redstone".into(), amount: 20 });
        assert_eq!(n.infusion, stack("minecraft:redstone", 1));
        // Another infusion type does not mix; one no recipe uses is not taken.
        let mixed = Io { infusion: stack("minecraft:coal", 1), tank: Tank { chemical: "mekanism:redstone".into(), amount: 5 }, ..Io::default() };
        assert_eq!(run(&mixed, 1).0.infusion, stack("minecraft:coal", 1));
        let none = Io { infusion: stack("minecraft:coal", 1), ..Io::default() };
        assert_eq!(run(&none, 1).0.infusion, stack("minecraft:coal", 1), "carbon is no recipe's");
    }

    #[test]
    fn the_tank_holds_a_bucket_at_most() {
        let io = Io { infusion: stack("minecraft:redstone", 2), tank: Tank { chemical: "mekanism:redstone".into(), amount: 995 }, ..Io::default() };
        assert_eq!(run(&io, 3).0.tank.amount, 995);
        let io = Io { infusion: stack("minecraft:redstone", 2), tank: Tank { chemical: "mekanism:redstone".into(), amount: 990 }, ..Io::default() };
        assert_eq!(run(&io, 3).0.tank.amount, 1000);
    }

    #[test]
    fn an_item_and_the_infusion_type_make_another_item() {
        let io = Io {
            input: stack("minecraft:iron_ingot", 2),
            tank: Tank { chemical: "mekanism:redstone".into(), amount: 25 },
            energy: 10_000,
            ..Io::default()
        };
        let (done, status) = run(&io, TICKS);
        assert_eq!(status, Status::Running);
        assert_eq!(done.output, stack("mekanism:alloy_infused", 1));
        assert_eq!(done.input, stack("minecraft:iron_ingot", 1));
        assert_eq!(done.tank.amount, 15);
        assert_eq!(done.energy, 10_000 - 5 * u64::from(TICKS));
    }

    #[test]
    fn an_empty_tank_clears_its_infusion_type() {
        let io = Io {
            input: stack("minecraft:iron_ingot", 1),
            tank: Tank { chemical: "mekanism:redstone".into(), amount: 10 },
            energy: 10_000,
            ..Io::default()
        };
        assert_eq!(run(&io, TICKS).0.tank, Tank::default());
    }

    #[test]
    fn it_waits_for_enough_infusion_type_and_a_recipe() {
        let b = |amount| Io {
            input: stack("minecraft:iron_ingot", 1),
            tank: Tank { chemical: "mekanism:redstone".into(), amount },
            energy: 100,
            ..Io::default()
        };
        assert_eq!(run(&b(9), 1).1, Status::Blocked, "10 are needed");
        let none = Io { input: stack("minecraft:iron_ingot", 1), energy: 100, ..Io::default() };
        assert_eq!(run(&none, 1).1, Status::Blocked, "an empty tank");
        let stick = Io { input: stack("minecraft:stick", 1), energy: 100, ..Io::default() };
        assert_eq!(run(&stick, 1).1, Status::NoRecipe);
        assert_eq!(run(&Io::default(), 1).1, Status::Idle);
        let dry = Io { energy: 4, ..b(10) };
        assert_eq!(run(&dry, 1).1, Status::NoEnergy);
        let full = Io { output: stack("mekanism:alloy_infused", 64), ..b(10) };
        assert_eq!(run(&full, 1).1, Status::Blocked);
        assert_eq!(step(5, &Book::default(), &b(10)).1, Status::NoBook);
    }
}

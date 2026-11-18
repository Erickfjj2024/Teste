//! The precision sawmill (`TileEntityPrecisionSawmill`): one input makes a main output and, by chance, a secondary one
//! (logs make planks and sometimes sawdust). The pure rules, tested without the game.
use crate::machine::{STACK, Stack, Status, TICKS};
use std::collections::HashMap;

/// What one input makes.
#[derive(Clone, Debug, PartialEq)]
pub struct Recipe {
    /// Items of the input stack it uses.
    pub count: u32,
    pub main: (String, u32),
    /// The secondary output, its count and its chance (0 to 1).
    pub secondary: Option<(String, u32, f64)>,
}

#[derive(Debug, Default)]
pub struct Book {
    by_input: HashMap<String, Recipe>,
}

impl Book {
    pub fn add(&mut self, inputs: impl IntoIterator<Item = String>, recipe: Recipe) {
        for input in inputs {
            self.by_input.entry(input).or_insert_with(|| recipe.clone());
        }
    }
    pub fn is_empty(&self) -> bool {
        self.by_input.is_empty()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Io {
    pub input: Option<Stack>,
    pub main: Option<Stack>,
    pub secondary: Option<Stack>,
    pub energy: u64,
    pub progress: u32,
    /// The ticks a recipe takes with this machine's speed upgrades; 0 is the base (`TICKS`).
    pub ticks: u32,
}

fn fits(slot: &Option<Stack>, item: &str, count: u32) -> bool {
    match slot {
        None => count <= STACK,
        Some(s) => s.item == item && s.components.is_empty() && s.count + count <= STACK,
    }
}

fn add(slot: &Option<Stack>, item: &str, count: u32) -> Option<Stack> {
    Some(match slot {
        Some(s) => Stack { count: s.count + count, ..s.clone() },
        None => Stack { item: item.to_owned(), count, components: String::new() },
    })
}

/// One tick. `roll` (0 up to 1) is the dice of a finished recipe's secondary output: it is made when `roll` is
/// at most its chance, as Mekanism rolls it. The machine needs room for both outputs to run.
pub fn step(usage: u64, book: &Book, io: &Io, roll: f64) -> (Io, Status) {
    let mut next = io.clone();
    if book.is_empty() {
        next.progress = 0;
        return (next, Status::NoBook);
    }
    let Some(input) = io.input.as_ref() else {
        next.progress = 0;
        return (next, Status::Idle);
    };
    let Some(recipe) = book.by_input.get(&input.item) else {
        next.progress = 0;
        return (next, Status::NoRecipe);
    };
    let room = fits(&io.main, &recipe.main.0, recipe.main.1)
        && recipe.secondary.as_ref().is_none_or(|s| fits(&io.secondary, &s.0, s.1));
    if input.count < recipe.count || !room {
        next.progress = 0;
        return (next, Status::Blocked);
    }
    if io.energy < usage {
        return (next, Status::NoEnergy);
    }
    next.energy -= usage;
    next.progress += 1;
    if next.progress >= if io.ticks == 0 { TICKS } else { io.ticks } {
        next.progress = 0;
        let left = input.count - recipe.count;
        next.input = (left > 0).then(|| Stack { count: left, ..input.clone() });
        next.main = add(&io.main, &recipe.main.0, recipe.main.1);
        if let Some((item, count, chance)) = &recipe.secondary
            && roll <= *chance
        {
            next.secondary = add(&io.secondary, item, *count);
        }
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
            ["minecraft:oak_log".to_owned()],
            Recipe {
                count: 1,
                main: ("minecraft:oak_planks".into(), 6),
                secondary: Some(("mekanism:sawdust".into(), 1, 0.25)),
            },
        );
        b.add(
            ["minecraft:bamboo_block".to_owned()],
            Recipe { count: 1, main: ("minecraft:bamboo".into(), 9), secondary: None },
        );
        b
    }
    fn run(io: &Io, ticks: u32, roll: f64) -> (Io, Status) {
        let b = book();
        let mut io = io.clone();
        let mut status = Status::Idle;
        for _ in 0..ticks {
            let (n, s) = step(5, &b, &io, roll);
            io = n;
            status = s;
        }
        (io, status)
    }

    #[test]
    fn a_log_makes_planks_and_sometimes_sawdust() {
        let io = Io { input: stack("minecraft:oak_log", 2), energy: 10_000, ..Io::default() };
        let (done, _) = run(&io, TICKS, 0.1);
        assert_eq!(done.main, stack("minecraft:oak_planks", 6));
        assert_eq!(done.secondary, stack("mekanism:sawdust", 1));
        assert_eq!(done.input, stack("minecraft:oak_log", 1));
        assert_eq!(done.energy, 10_000 - 5 * u64::from(TICKS));
        // A roll above the chance makes no sawdust.
        let (done, _) = run(&io, TICKS, 0.9);
        assert_eq!(done.main, stack("minecraft:oak_planks", 6));
        assert_eq!(done.secondary, None);
    }

    #[test]
    fn a_recipe_without_a_secondary_output_makes_only_the_main_one() {
        let io = Io { input: stack("minecraft:bamboo_block", 1), energy: 10_000, ..Io::default() };
        let (done, _) = run(&io, TICKS, 0.0);
        assert_eq!(done.main, stack("minecraft:bamboo", 9));
        assert_eq!(done.secondary, None);
        assert_eq!(done.input, None);
    }

    #[test]
    fn it_waits_for_room_for_both_outputs() {
        // The planks fit, the secondary slot holds something else: it does not start.
        let io = Io {
            input: stack("minecraft:oak_log", 1),
            secondary: stack("minecraft:stick", 3),
            energy: 10_000,
            ..Io::default()
        };
        let (n, s) = run(&io, 5, 0.0);
        assert_eq!(s, Status::Blocked);
        assert_eq!(n.progress, 0);
        // Planks that are nearly a stack do not take another six.
        let io = Io { input: stack("minecraft:oak_log", 1), main: stack("minecraft:oak_planks", 60), energy: 10_000, ..Io::default() };
        assert_eq!(run(&io, 1, 0.0).1, Status::Blocked);
    }

    #[test]
    fn it_needs_energy_and_a_recipe() {
        let io = Io { input: stack("minecraft:oak_log", 1), energy: 3, ..Io::default() };
        assert_eq!(run(&io, 1, 0.0).1, Status::NoEnergy);
        let io = Io { input: stack("minecraft:dirt", 1), energy: 100, ..Io::default() };
        assert_eq!(run(&io, 1, 0.0).1, Status::NoRecipe);
        assert_eq!(run(&Io::default(), 1, 0.0).1, Status::Idle);
    }
}

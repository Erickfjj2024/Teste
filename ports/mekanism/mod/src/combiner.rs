//! The combiner (`TileEntityCombiner`): two inputs make one output (flint and cobblestone make gravel).
//! The pure rules, tested without the game.
use crate::machine::{STACK, Stack, Status, TICKS};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Recipe {
    pub main_count: u32,
    pub extra_count: u32,
    pub output: String,
    pub out_count: u32,
}

/// The recipes by their two inputs (the main one, then the extra one); the first of two alike wins.
#[derive(Debug, Default)]
pub struct Book {
    by_pair: HashMap<(String, String), Recipe>,
}

impl Book {
    pub fn add(&mut self, mains: &[String], extras: &[String], recipe: Recipe) {
        for m in mains {
            for e in extras {
                self.by_pair.entry((m.clone(), e.clone())).or_insert_with(|| recipe.clone());
            }
        }
    }
    pub fn is_empty(&self) -> bool {
        self.by_pair.is_empty()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Io {
    pub main: Option<Stack>,
    pub extra: Option<Stack>,
    pub output: Option<Stack>,
    pub energy: u64,
    pub progress: u32,
}

/// One tick: it runs when the two inputs have a recipe and have enough, there is room for the output and energy.
pub fn step(usage: u64, book: &Book, io: &Io) -> (Io, Status) {
    let mut next = io.clone();
    if book.is_empty() {
        next.progress = 0;
        return (next, Status::NoBook);
    }
    let (Some(main), Some(extra)) = (io.main.as_ref(), io.extra.as_ref()) else {
        next.progress = 0;
        return (next, Status::Idle);
    };
    let Some(recipe) = book.by_pair.get(&(main.item.clone(), extra.item.clone())) else {
        next.progress = 0;
        return (next, Status::NoRecipe);
    };
    let room = match &io.output {
        None => recipe.out_count <= STACK,
        Some(o) => o.item == recipe.output && o.components.is_empty() && o.count + recipe.out_count <= STACK,
    };
    if main.count < recipe.main_count || extra.count < recipe.extra_count || !room {
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
        let take = |s: &Stack, n: u32| (s.count > n).then(|| Stack { count: s.count - n, ..s.clone() });
        next.main = take(main, recipe.main_count);
        next.extra = take(extra, recipe.extra_count);
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
            &["minecraft:flint".to_owned()],
            &["minecraft:cobblestone".to_owned(), "minecraft:cobbled_deepslate".to_owned()],
            Recipe { main_count: 1, extra_count: 1, output: "minecraft:gravel".into(), out_count: 1 },
        );
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
    fn two_inputs_make_one_output() {
        let io = Io {
            main: stack("minecraft:flint", 2),
            extra: stack("minecraft:cobbled_deepslate", 1),
            energy: 10_000,
            ..Io::default()
        };
        let (done, _) = run(&io, TICKS);
        assert_eq!(done.output, stack("minecraft:gravel", 1));
        assert_eq!(done.main, stack("minecraft:flint", 1));
        assert_eq!(done.extra, None);
    }

    #[test]
    fn it_needs_both_inputs_a_recipe_for_the_pair_and_room() {
        let one = Io { main: stack("minecraft:flint", 1), energy: 100, ..Io::default() };
        assert_eq!(run(&one, 1).1, Status::Idle);
        let wrong = Io { main: stack("minecraft:flint", 1), extra: stack("minecraft:dirt", 1), energy: 100, ..Io::default() };
        assert_eq!(run(&wrong, 1).1, Status::NoRecipe);
        // The order matters: the main input is the one the recipe says.
        let swapped = Io { main: stack("minecraft:cobblestone", 1), extra: stack("minecraft:flint", 1), energy: 100, ..Io::default() };
        assert_eq!(run(&swapped, 1).1, Status::NoRecipe);
        let full = Io {
            main: stack("minecraft:flint", 1),
            extra: stack("minecraft:cobblestone", 1),
            output: stack("minecraft:gravel", 64),
            energy: 100,
            ..Io::default()
        };
        assert_eq!(run(&full, 1).1, Status::Blocked);
        let dry = Io { main: stack("minecraft:flint", 1), extra: stack("minecraft:cobblestone", 1), energy: 2, ..Io::default() };
        assert_eq!(run(&dry, 1).1, Status::NoEnergy);
    }
}

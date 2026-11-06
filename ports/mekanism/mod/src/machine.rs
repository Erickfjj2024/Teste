//! An electric machine that turns one stack into another over time
//! (Mekanism's `TileEntityElectricMachine`: the crusher, the enrichment
//! chamber, the energized smelter): the pure rules, tested without the game.
use std::collections::HashMap;

/// Ticks a recipe takes (`TileEntityProgressMachine.BASE_TICKS_REQUIRED`).
pub const TICKS: u32 = 200;
/// Items in a stack, for an output slot that is not empty.
const STACK: u32 = 64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stack {
    pub item: String,
    pub count: u32,
    pub components: String,
}

/// What one input item makes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recipe {
    /// Items of the input stack it uses.
    pub count: u32,
    pub output: String,
    pub out_count: u32,
}

/// The recipes of one machine by the input item that starts each (the first one wins).
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
    pub fn get(&self, item: &str) -> Option<&Recipe> {
        self.by_input.get(item)
    }
    pub fn len(&self) -> usize {
        self.by_input.len()
    }
    pub fn is_empty(&self) -> bool {
        self.by_input.is_empty()
    }
}

/// What a machine holds.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Io {
    pub input: Option<Stack>,
    pub output: Option<Stack>,
    pub energy: u64,
    pub progress: u32,
}

/// One tick: the machine runs when its input has a recipe, there is room for
/// what it makes and energy for the tick (`usage`); a finished recipe is made
/// at once. A machine with no recipe or no room starts over; one short of
/// energy waits where it was. Returns the new contents and whether it ran.
pub fn step(usage: u64, book: &Book, io: &Io) -> (Io, bool) {
    let mut next = io.clone();
    let recipe = io.input.as_ref().and_then(|s| book.get(&s.item).map(|r| (s, r)));
    let Some((input, recipe)) = recipe else {
        next.progress = 0;
        return (next, false);
    };
    let room = match &io.output {
        None => recipe.out_count <= STACK,
        Some(o) => {
            o.item == recipe.output && o.components.is_empty() && o.count + recipe.out_count <= STACK
        }
    };
    if input.count < recipe.count || !room {
        next.progress = 0;
        return (next, false);
    }
    if io.energy < usage {
        return (next, false);
    }
    next.energy -= usage;
    next.progress += 1;
    if next.progress >= TICKS {
        next.progress = 0;
        let left = input.count - recipe.count;
        next.input = (left > 0).then(|| Stack {
            count: left,
            ..input.clone()
        });
        next.output = Some(match &io.output {
            Some(o) => Stack {
                count: o.count + recipe.out_count,
                ..o.clone()
            },
            None => Stack {
                item: recipe.output.clone(),
                count: recipe.out_count,
                components: String::new(),
            },
        });
    }
    (next, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(item: &str, count: u32) -> Option<Stack> {
        Some(Stack {
            item: item.into(),
            count,
            components: String::new(),
        })
    }
    fn book() -> Book {
        let mut b = Book::default();
        b.add(
            ["mekanism:ore_a".to_owned(), "mekanism:ore_b".to_owned()],
            Recipe {
                count: 1,
                output: "mekanism:dust".into(),
                out_count: 2,
            },
        );
        b
    }
    fn run(io: Io, ticks: u32) -> (Io, u32) {
        let book = book();
        let (mut io, mut ran) = (io, 0);
        for _ in 0..ticks {
            let (next, active) = step(8, &book, &io);
            io = next;
            ran += u32::from(active);
        }
        (io, ran)
    }

    #[test]
    fn a_recipe_takes_200_ticks_and_8_a_tick() {
        let io = Io {
            input: stack("mekanism:ore_a", 3),
            energy: 10_000,
            ..Io::default()
        };
        let (io, ran) = run(io, 199);
        assert_eq!((io.progress, ran, io.energy), (199, 199, 10_000 - 199 * 8));
        let (io, _) = run(io, 1);
        assert_eq!(io.progress, 0);
        assert_eq!(io.input, stack("mekanism:ore_a", 2));
        assert_eq!(io.output, stack("mekanism:dust", 2));
    }
    #[test]
    fn it_empties_the_input_and_stacks_the_output() {
        let io = Io {
            input: stack("mekanism:ore_b", 1),
            output: stack("mekanism:dust", 10),
            energy: 100_000,
            ..Io::default()
        };
        let (io, _) = run(io, 200);
        assert_eq!(io.input, None);
        assert_eq!(io.output, stack("mekanism:dust", 12));
    }
    #[test]
    fn it_waits_without_energy_and_starts_over_without_room_or_recipe() {
        let io = Io {
            input: stack("mekanism:ore_a", 1),
            energy: 8 * 50,
            ..Io::default()
        };
        let (io, ran) = run(io, 80);
        assert_eq!((io.progress, ran, io.energy), (50, 50, 0));
        // Another item in the output slot: no room.
        let blocked = Io {
            output: stack("mekanism:other", 1),
            energy: 1000,
            progress: 40,
            ..io.clone()
        };
        assert_eq!(run(blocked, 1), (Io { progress: 0, energy: 1000, ..io.clone() }.with_output("mekanism:other", 1), 0));
        // A full stack: no room.
        let full = Io {
            output: stack("mekanism:dust", 63),
            energy: 1000,
            progress: 40,
            ..io.clone()
        };
        assert_eq!(run(full, 1).0.progress, 0);
        // No recipe for the input.
        let none = Io {
            input: stack("mekanism:nothing", 1),
            energy: 1000,
            progress: 40,
            ..Io::default()
        };
        assert_eq!(run(none, 1), (Io { progress: 0, ..Io { input: stack("mekanism:nothing", 1), energy: 1000, ..Io::default() } }, 0));
    }
    impl Io {
        fn with_output(mut self, item: &str, count: u32) -> Io {
            self.output = stack(item, count);
            self
        }
    }
    #[test]
    fn the_first_recipe_for_an_input_wins() {
        let mut b = book();
        b.add(
            ["mekanism:ore_a".to_owned()],
            Recipe { count: 1, output: "mekanism:other".into(), out_count: 1 },
        );
        assert_eq!(b.get("mekanism:ore_a").unwrap().output, "mekanism:dust");
        assert_eq!(b.len(), 2);
    }
}

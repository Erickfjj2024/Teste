//! Factories (`TileEntityFactory`): the machine of the electric kind (the crusher, the enrichment chamber, the energized smelter)
//! many times over, 3, 5, 7 or 9 lines by tier (`FactoryTier.processes`). Each line has its own input and output slot and its own
//! progress; they all work from one energy store, each using what the machine uses a tick. The pure rules, tested without the
//! game, and then the tick.
use super::*;
use crate::machine::{Book, Io, Stack, Status, step};

/// Where a factory keeps the progress of its lines after the first (the first is in the first two bytes, as every machine's):
/// after its side configuration, its control and its upgrades.
pub const TAIL: usize = 46;

/// One line of a factory.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Line {
    pub input: Option<Stack>,
    pub output: Option<Stack>,
    pub progress: u32,
}

/// One tick of every line, in order, each from the energy the ones before left. Returns the lines after, the energy left, what each
/// did and the energy used.
pub fn step_lines(usage: u64, ticks: u32, book: &Book, lines: &[Line], energy: u64) -> (Vec<Line>, u64, Vec<Status>, u64) {
    let mut left = energy;
    let mut out = Vec::with_capacity(lines.len());
    let mut statuses = Vec::with_capacity(lines.len());
    for line in lines {
        let io = Io { input: line.input.clone(), output: line.output.clone(), energy: left, progress: line.progress, ticks };
        let (next, status) = step(usage, book, &io);
        left = next.energy;
        out.push(Line { input: next.input, output: next.output, progress: next.progress });
        statuses.push(status);
    }
    (out, left, statuses, energy - left)
}

/// What the screen's status says of all the lines: working if one is, else the first problem, else idle.
pub fn overall(statuses: &[Status]) -> Status {
    for wanted in [Status::Running, Status::NoEnergy, Status::Blocked, Status::NoRecipe, Status::NoBook] {
        if statuses.contains(&wanted) {
            return wanted;
        }
    }
    Status::Idle
}

/// The charge slot (`EnergyInventorySlot.fillOrConvert`): an item the conversion recipes know turns into energy while the store has
/// room for all of it.
pub fn charge(energy: u64, capacity: u64, slot: Option<Stack>) -> (u64, Option<Stack>) {
    let Some(s) = slot else { return (energy, None) };
    match cube::conversion(&s.item) {
        Some(gain) if energy + gain <= capacity => {
            let rest = (s.count > 1).then(|| Stack { count: s.count - 1, ..s.clone() });
            (energy + gain, rest)
        }
        _ => (energy, Some(s)),
    }
}

impl Mekanism {
    /// A tick of a factory.
    pub(crate) fn run_factory(&mut self, def: &Def, e: &BlockEntity, tick: u64) -> Option<BlockEntityUpdate> {
        let n = def.lines;
        let layout = def.layout;
        // Its machine's recipes, asked again every second until the game lends them.
        if self.books.get(def.base).is_none_or(Book::is_empty) && self.tried.get(def.block).is_none_or(|t| tick.saturating_sub(*t) >= 20) {
            self.tried.insert(def.block, tick);
            self.books.insert(def.base, load_book(def.source));
        }
        let mut facing = sides::property(&e.state, "facing").unwrap_or("north").to_owned();
        let progress_of = |i: usize| {
            let at = if i == 0 { 0 } else { TAIL + 2 * (i - 1) };
            e.data.get(at..at + 2).map_or(0, |b| u32::from(u16::from_le_bytes([b[0], b[1]])))
        };
        let stored = Config::decode(e.data.get(2..).unwrap_or(&[]), layout);
        let mut cfg = stored.unwrap_or_else(|| config::default_config(layout));
        if stored.is_none() {
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
        let counts = upgrades::counts_of(def, e);
        let (speed, energy_up) = (counts[upgrades::SPEED], counts[upgrades::ENERGY]);
        let usage = upgrades::energy_per_tick(def.usage, speed, energy_up);
        let ticks = upgrades::ticks(machine::TICKS, speed);
        let capacity = upgrades::max_energy(base_capacity(def), energy_up);
        let slot = |i: usize| stack(e.items.get(i).unwrap_or(&None));
        let lines: Vec<Line> = (0..n).map(|i| Line { input: slot(2 * i), output: slot(2 * i + 1), progress: progress_of(i) }).collect();
        // The charge slot first, then the lines.
        let (energy, charged) = if self.can { charge(e.energy, capacity, slot(2 * n)) } else { (e.energy, slot(2 * n)) };
        let (next, left, statuses, used) = if self.can {
            let book = self.books.entry(def.base).or_default();
            step_lines(usage, ticks, book, &lines, energy)
        } else {
            (lines.clone(), energy, vec![Status::Idle; n], 0)
        };
        let status = overall(&statuses);
        let active = statuses.contains(&Status::Running);
        let lit = e.state.contains("lit=true");
        if active != lit {
            let _ = world::set_block_property(&e.dimension, e.pos.x, e.pos.y, e.pos.z, "lit", if active { "true" } else { "false" });
        }
        let energy_faces = dirty.then(|| self.apply_config(e, layout, &facing, &cfg));
        let mut items: Vec<Option<ItemStack>> = Vec::with_capacity(e.items.len());
        for (i, l) in next.iter().enumerate() {
            items.push(item(&l.input));
            items.push(item(&l.output));
            let _ = i;
        }
        items.push(item(&charged));
        items.extend(e.items.iter().skip(items.len()).cloned());
        let mut data = menu_data(next[0].progress, left, capacity, status, &cfg);
        let control = Control::from_byte(e.data.get(redstone::offset(def.kind)).copied().unwrap_or(0));
        let mut saved = machine_data(next[0].progress, &cfg);
        let shown = (ticks, self.install.get(&key).copied().unwrap_or(0), counts, used);
        redstone::stamp(def.kind, control, capacity.saturating_sub(left), shown, &mut saved, &mut data);
        // The lines' progress: all in the screen's numbers, the ones after the first in the saved bytes' tail.
        data.extend(next.iter().map(|l| l.progress as i32));
        saved.resize(TAIL, 0);
        for l in next.iter().skip(1) {
            saved.extend((l.progress as u16).to_le_bytes());
        }
        let changed = next != lines || charged != slot(2 * n) || left != e.energy;
        if !changed && !dirty && data == e.menu_data && saved == e.data && !self.force {
            return None;
        }
        Some(BlockEntityUpdate {
            dimension: e.dimension.clone(),
            pos: e.pos,
            data: saved,
            items: Some(items),
            energy: Some(left),
            energy_faces,
            tanks: None,
            tank_faces: None,
            menu_data: Some(data),
            transmitter_faces: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::Recipe;

    fn stack(item: &str, count: u32) -> Option<Stack> {
        Some(Stack { item: item.into(), count, components: String::new() })
    }
    fn book() -> Book {
        let mut b = Book::default();
        b.add(["minecraft:cobblestone".to_owned()], Recipe { count: 1, output: "minecraft:gravel".into(), out_count: 1 });
        b
    }

    #[test]
    fn each_line_works_and_uses_energy() {
        let lines = vec![
            Line { input: stack("minecraft:cobblestone", 4), ..Line::default() },
            Line { input: stack("minecraft:cobblestone", 1), ..Line::default() },
            Line::default(),
        ];
        let (next, left, statuses, used) = step_lines(8, 0, &book(), &lines, 100);
        assert_eq!(statuses, [Status::Running, Status::Running, Status::Idle]);
        assert_eq!((used, left), (16, 84));
        assert_eq!((next[0].progress, next[1].progress, next[2].progress), (1, 1, 0));
    }

    #[test]
    fn the_lines_share_the_energy() {
        let lines = vec![
            Line { input: stack("minecraft:cobblestone", 4), ..Line::default() },
            Line { input: stack("minecraft:cobblestone", 4), ..Line::default() },
        ];
        // Enough for one line only: the first works, the second waits for energy.
        let (next, left, statuses, _) = step_lines(8, 0, &book(), &lines, 10);
        assert_eq!(statuses, [Status::Running, Status::NoEnergy]);
        assert_eq!((left, next[1].progress), (2, 0));
        assert_eq!(overall(&statuses), Status::Running);
    }

    #[test]
    fn a_line_finishes_its_recipe_alone() {
        let lines = vec![
            Line { input: stack("minecraft:cobblestone", 2), progress: 199, ..Line::default() },
            Line { input: stack("minecraft:cobblestone", 2), progress: 5, ..Line::default() },
        ];
        let (next, _, _, _) = step_lines(8, 0, &book(), &lines, 100);
        assert_eq!(next[0].output, stack("minecraft:gravel", 1));
        assert_eq!(next[0].input, stack("minecraft:cobblestone", 1));
        assert_eq!((next[0].progress, next[1].progress), (0, 6));
        assert_eq!(next[1].output, None);
    }

    #[test]
    fn the_charge_slot_turns_redstone_into_energy_while_there_is_room() {
        assert_eq!(charge(0, 100_000, stack("minecraft:redstone", 2)), (10_000, stack("minecraft:redstone", 1)));
        assert_eq!(charge(0, 100_000, stack("minecraft:redstone", 1)), (10_000, None));
        // No room for all of it: it stays.
        assert_eq!(charge(95_000, 100_000, stack("minecraft:redstone", 1)), (95_000, stack("minecraft:redstone", 1)));
        assert_eq!(charge(0, 100_000, stack("minecraft:stick", 1)), (0, stack("minecraft:stick", 1)));
    }

    #[test]
    fn overall_picks_working_then_problems() {
        assert_eq!(overall(&[Status::Idle, Status::Blocked]), Status::Blocked);
        assert_eq!(overall(&[Status::NoEnergy, Status::Blocked]), Status::NoEnergy);
        assert_eq!(overall(&[Status::Idle, Status::Idle]), Status::Idle);
    }
}

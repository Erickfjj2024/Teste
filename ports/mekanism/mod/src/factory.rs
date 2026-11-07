//! Factories (`TileEntityFactory`): the machine of the electric kind (the crusher, the enrichment chamber, the energized smelter)
//! many times over, 3, 5, 7 or 9 lines by tier (`FactoryTier.processes`). Each line has its own input and output slot and its own
//! progress; they all work from one energy store, each using what the machine uses a tick. The pure rules, tested without the
//! game, and then the tick.
use super::*;
use crate::machine::{Book, Io, Stack, Status, step};

/// Where a factory keeps the progress of its lines after the first (the first is in the first two bytes, as every machine's):
/// after its side configuration, its control and its upgrades.
pub const TAIL: usize = 46;

/// One line of a factory: its input, what it makes (one slot, two for sawing) and its progress.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Line {
    pub input: Option<Stack>,
    pub output: Option<Stack>,
    pub secondary: Option<Stack>,
    pub progress: u32,
}

/// The recipes a factory works from.
pub struct Books<'a> {
    pub items: &'a Book,
    pub combine: &'a combiner::Book,
    pub saw: &'a sawmill::Book,
}

/// One tick of every line, in order, each from the energy the ones before left (`form` 0 items, 1 combining with the `extra` input all
/// the lines share, 2 sawing, whose chance is rolled by `dice`). Returns the lines after, the extra input, the energy left, what each
/// did and the energy used.
pub fn step_lines(
    form: u8,
    usage: u64,
    ticks: u32,
    books: &Books,
    lines: &[Line],
    extra: Option<Stack>,
    energy: u64,
    dice: &mut dyn FnMut() -> f64,
) -> (Vec<Line>, Option<Stack>, u64, Vec<Status>, u64) {
    let mut left = energy;
    let mut extra = extra;
    let mut out = Vec::with_capacity(lines.len());
    let mut statuses = Vec::with_capacity(lines.len());
    for line in lines {
        let (next, status) = match form {
            1 => {
                let io = combiner::Io { main: line.input.clone(), extra: extra.clone(), output: line.output.clone(), energy: left, progress: line.progress, ticks };
                let (n, status) = combiner::step(usage, books.combine, &io);
                extra = n.extra.clone();
                left = n.energy;
                (Line { input: n.main, output: n.output, secondary: None, progress: n.progress }, status)
            }
            2 => {
                let io = sawmill::Io { input: line.input.clone(), main: line.output.clone(), secondary: line.secondary.clone(), energy: left, progress: line.progress, ticks };
                let (n, status) = sawmill::step(usage, books.saw, &io, dice());
                left = n.energy;
                (Line { input: n.input, output: n.main, secondary: n.secondary, progress: n.progress }, status)
            }
            _ => {
                let io = Io { input: line.input.clone(), output: line.output.clone(), energy: left, progress: line.progress, ticks };
                let (n, status) = step(usage, books.items, &io);
                left = n.energy;
                (Line { input: n.input, output: n.output, secondary: None, progress: n.progress }, status)
            }
        };
        out.push(next);
        statuses.push(status);
    }
    (out, extra, left, statuses, energy - left)
}

/// One tick of every line of a factory that works a chemical (compressing, injecting, purifying, infusing), from the one tank all the
/// lines share and the energy the lines before left: `spec` is its machine's (its tank as big as all the lines' together). The chemical
/// item slot fills the tank once a tick. Returns the lines after, the chemical item slot, the tank, the energy left, what each line did
/// and the energy used.
pub fn step_chem_lines(
    spec: &process::Spec,
    book: &process::Book,
    lines: &[Line],
    conv: Option<Stack>,
    tank: process::Tank,
    energy: u64,
    dice: &mut dyn FnMut() -> f64,
) -> (Vec<Line>, Option<Stack>, process::Tank, u64, Vec<Status>, u64) {
    let (mut left, mut conv, mut tank) = (energy, conv, tank);
    let mut out = Vec::with_capacity(lines.len());
    let mut statuses = Vec::with_capacity(lines.len());
    for (i, line) in lines.iter().enumerate() {
        let io = process::Io {
            input: line.input.clone(),
            output: line.output.clone(),
            // Only the first line takes the chemical item (once a tick, as the slot fills the tank).
            conv: if i == 0 { conv.clone() } else { None },
            tank_in: tank.clone(),
            tank_out: process::Tank::default(),
            energy: left,
            progress: line.progress,
        };
        let (n, status) = process::step(spec, book, &io, dice());
        if i == 0 {
            conv = n.conv.clone();
        }
        left = n.energy;
        tank = n.tank_in.clone();
        out.push(Line { input: n.input, output: n.output, secondary: None, progress: n.progress });
        statuses.push(status);
    }
    (out, conv, tank, left, statuses, energy - left)
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
        let form = def.form;
        let per = config::factory_slots_per_line(form);
        let layout = def.layout;
        // Its machine's recipes, asked again every second until the game lends them.
        let base_def = MACHINES.iter().find(|d| d.block == def.base);
        let chem = form >= 3;
        let loaded = match form {
            1 => !self.combs.is_empty(),
            2 => !self.saws.is_empty(),
            3 | 4 => self.procs.get(def.base).is_some_and(|b| !b.is_empty()),
            _ => self.books.get(def.base).is_some_and(|b| !b.is_empty()),
        };
        if !loaded && self.tried.get(def.block).is_none_or(|t| tick.saturating_sub(*t) >= 20) {
            self.tried.insert(def.block, tick);
            match form {
                1 => self.combs = load_comb_book(),
                2 => self.saws = load_saw_book(),
                3 | 4 => {
                    if let Some(spec) = base_def.and_then(|d| d.proc) {
                        self.procs.insert(def.base, load_proc_book(def.source, spec.shape));
                    }
                }
                _ => {
                    self.books.insert(def.base, load_book(def.source));
                }
            }
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
        let ticks = upgrades::ticks(base_def.and_then(|d| d.proc).map_or(machine::TICKS, |p| p.ticks), speed);
        let capacity = upgrades::max_energy(base_capacity(def), energy_up);
        let slot = |i: usize| stack(e.items.get(i).unwrap_or(&None));
        let lines: Vec<Line> = (0..n)
            .map(|i| Line {
                input: slot(per * i),
                output: slot(per * i + 1),
                secondary: if form == 2 { slot(per * i + 2) } else { None },
                progress: progress_of(i),
            })
            .collect();
        let extra_at = n * per;
        let charge_at = config::factory_slot_count(n, form) - 1;
        let extra = if config::factory_has_extra(form) { slot(extra_at) } else { None };
        let tank_now = plus::held(e, 0);
        // The charge slot first, then the lines.
        let (energy, charged) = if self.can { charge(e.energy, capacity, slot(charge_at)) } else { (e.energy, slot(charge_at)) };
        let (next, extra_next, tank_next, left, statuses, used) = if self.can {
            let mut seed = self.roll(tick, &e.pos);
            let mut dice = || {
                seed = (seed * 7919.0 + 0.618_033_988_75).fract();
                seed
            };
            if chem {
                // The machine's own numbers, its tank as big as every line's and its upgrades'.
                let mut spec = base_def.and_then(|d| d.proc).expect("a chemical factory's machine has its spec");
                spec.cap_in *= n as u64;
                spec.usage = usage;
                spec.ticks = upgrades::ticks(spec.ticks, speed);
                spec.mean = upgrades::gas_mean(speed, counts[upgrades::CHEMICAL]);
                let book = self.procs.entry(def.base).or_default();
                let (l, conv, tank, left, st, used) = step_chem_lines(&spec, book, &lines, extra.clone(), tank_now.clone(), energy, &mut dice);
                (l, conv, tank, left, st, used)
            } else {
                let books = Books { items: self.books.get(def.base).unwrap_or(&EMPTY_BOOK), combine: &self.combs, saw: &self.saws };
                let (l, ex, left, st, used) = step_lines(form, usage, ticks, &books, &lines, extra.clone(), energy, &mut dice);
                (l, ex, tank_now.clone(), left, st, used)
            }
        } else {
            (lines.clone(), extra.clone(), tank_now.clone(), energy, vec![Status::Idle; n], 0)
        };
        let status = overall(&statuses);
        let active = statuses.contains(&Status::Running);
        let lit = e.state.contains("lit=true");
        if active != lit {
            let _ = world::set_block_property(&e.dimension, e.pos.x, e.pos.y, e.pos.z, "lit", if active { "true" } else { "false" });
        }
        // The tank takes in only what a recipe of the machine uses.
        if chem
            && !self.filtered.contains(&key)
            && let Some(book) = self.procs.get(def.base).filter(|b| !b.is_empty())
        {
            let _ = world::set_tank_filter(&e.dimension, e.pos.x, e.pos.y, e.pos.z, 0, &book.accepted());
            self.filtered.insert(key.clone());
        }
        let energy_faces = dirty.then(|| self.apply_config(e, layout, &facing, &cfg));
        let mut items: Vec<Option<ItemStack>> = Vec::with_capacity(e.items.len());
        for l in &next {
            items.push(item(&l.input));
            items.push(item(&l.output));
            if form == 2 {
                items.push(item(&l.secondary));
            }
        }
        if config::factory_has_extra(form) {
            items.push(item(&extra_next));
        }
        items.push(item(&charged));
        items.extend(e.items.iter().skip(items.len()).cloned());
        let mut data = menu_data(next[0].progress, left, capacity, status, &cfg);
        let control = Control::from_byte(e.data.get(redstone::offset(def.kind)).copied().unwrap_or(0));
        let mut saved = machine_data(next[0].progress, &cfg);
        let shown = (ticks, self.install.get(&key).copied().unwrap_or(0), counts, used);
        redstone::stamp(def.kind, control, capacity.saturating_sub(left), shown, &facing, &mut saved, &mut data);
        // The lines' progress: all in the screen's numbers, the ones after the first in the saved bytes' tail.
        data.extend(next.iter().map(|l| l.progress as i32));
        if chem {
            // The tank's amount and chemical, after the lines' progress.
            data.extend(chem::screen(&tank_next));
        }
        saved.resize(TAIL, 0);
        for l in next.iter().skip(1) {
            saved.extend((l.progress as u16).to_le_bytes());
        }
        let changed = next != lines || charged != slot(charge_at) || extra_next != extra || left != e.energy || tank_next != tank_now;
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
            tanks: chem.then(|| vec![plus::as_tank(&tank_next)]),
            tank_faces: (chem && dirty).then(|| self.tank_sides(layout, &facing, &cfg, false)),
            menu_data: Some(data),
            transmitter_faces: None,
        })
    }
}

static EMPTY_BOOK: std::sync::LazyLock<Book> = std::sync::LazyLock::new(Book::default);

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

    fn go(lines: &[Line], energy: u64) -> (Vec<Line>, Option<Stack>, u64, Vec<Status>, u64) {
        let (comb, saw) = (combiner::Book::default(), sawmill::Book::default());
        let items = book();
        let books = Books { items: &items, combine: &comb, saw: &saw };
        step_lines(0, 8, 0, &books, lines, None, energy, &mut || 0.0)
    }

    #[test]
    fn each_line_works_and_uses_energy() {
        let lines = vec![
            Line { input: stack("minecraft:cobblestone", 4), ..Line::default() },
            Line { input: stack("minecraft:cobblestone", 1), ..Line::default() },
            Line::default(),
        ];
        let (next, _, left, statuses, used) = go(&lines, 100);
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
        let (next, _, left, statuses, _) = go(&lines, 10);
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
        let (next, _, _, _, _) = go(&lines, 100);
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

    #[test]
    fn combining_lines_share_the_extra_input() {
        let mut comb = combiner::Book::default();
        comb.add(&["minecraft:flint".to_owned()], &["minecraft:cobblestone".to_owned()], combiner::Recipe { main_count: 1, extra_count: 1, output: "minecraft:gravel".into(), out_count: 1 });
        let (saw, items) = (sawmill::Book::default(), Book::default());
        let books = Books { items: &items, combine: &comb, saw: &saw };
        let lines = vec![
            Line { input: stack("minecraft:flint", 2), ..Line::default() },
            Line { input: stack("minecraft:flint", 2), ..Line::default() },
        ];
        let (next, extra, _, statuses, _) = step_lines(1, 8, 1, &books, &lines, stack("minecraft:cobblestone", 5), 100, &mut || 0.0);
        // One tick takes each line's recipe at once here (a one tick recipe): each used one of the shared stack.
        assert_eq!(statuses, [Status::Running, Status::Running]);
        assert_eq!(extra, stack("minecraft:cobblestone", 3));
        assert_eq!(next[0].output, stack("minecraft:gravel", 1));
        assert_eq!(next[1].output, stack("minecraft:gravel", 1));
    }

    #[test]
    fn the_slots_of_each_form_follow_the_screen() {
        use crate::config::{factory_slot_count, slots_vec, Data, Layout};
        assert_eq!(factory_slot_count(3, 0), 7);
        assert_eq!(factory_slot_count(3, 1), 8);
        assert_eq!(factory_slot_count(3, 2), 10);
        assert_eq!(slots_vec(Layout::Factory(3, 0), Data::Input), [0, 2, 4]);
        assert_eq!(slots_vec(Layout::Factory(3, 0), Data::Energy), [6]);
        assert_eq!(slots_vec(Layout::Factory(3, 1), Data::Extra), [6]);
        assert_eq!(slots_vec(Layout::Factory(3, 1), Data::Energy), [7]);
        assert_eq!(slots_vec(Layout::Factory(3, 2), Data::Output), [1, 2, 4, 5, 7, 8]);
        assert_eq!(slots_vec(Layout::Factory(3, 2), Data::Input), [0, 3, 6]);
    }
}

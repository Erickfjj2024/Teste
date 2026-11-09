//! The electric pump (`TileEntityElectricPump`): every 19 ticks (fewer with speed upgrades) it spends a tick's energy to suck a fluid
//! source into its tank, first the block below it and then the blocks around the ones it already took; the full tank gives its
//! fluid to the block above it. The pure rules are tested without the game.
use super::*;
use crate::tank::EditMode;

/// `MekanismConfig.usage.electricPump`.
pub const USAGE: u64 = crate::fe(100);
/// `BASE_TICKS_REQUIRED`.
pub const BASE_TICKS: u32 = 19;
/// `MAX_FLUID`: ten buckets.
pub const TANK: u64 = 10_000;
/// `MekanismConfig.general.maxPumpRange`.
pub const RANGE: i32 = 80;
/// A bucket.
pub const BUCKET: u64 = 1000;

/// The fluid a source block is: water and lava at their lowest level (a source), `None` for anything else.
pub fn source_fluid(state: &str) -> Option<&'static str> {
    let (name, props) = state.split_once('[').map_or((state, ""), |(n, p)| (n, p));
    let source = props.trim_end_matches(']').split(',').all(|p| p.is_empty() || !p.starts_with("level=") || p == "level=0");
    match name {
        "minecraft:water" if source => Some("minecraft:water"),
        "minecraft:lava" if source => Some("minecraft:lava"),
        _ => None,
    }
}

/// Whether sucking a source leaves it (`isInfiniteSource`: water, while the game rule makes two sources a new one).
pub fn infinite(fluid: &str) -> bool {
    fluid == "minecraft:water"
}

/// What one operation does: the tank after (one bucket of `fluid` more), if it fits and the tank holds nothing else.
pub fn take(tank: &process::Tank, fluid: &str) -> Option<process::Tank> {
    if tank.amount + BUCKET > TANK || (tank.amount > 0 && tank.chemical != fluid) {
        return None;
    }
    Some(process::Tank { chemical: fluid.to_owned(), amount: tank.amount + BUCKET })
}

/// The six neighbours of a block.
pub fn around(p: (i32, i32, i32)) -> [(i32, i32, i32); 6] {
    [(p.0, p.1 - 1, p.2), (p.0, p.1 + 1, p.2), (p.0 - 1, p.1, p.2), (p.0 + 1, p.1, p.2), (p.0, p.1, p.2 - 1), (p.0, p.1, p.2 + 1)]
}

/// What a plenisher knows of its work: the nodes still to fill (a queue), the ones used, and whether the search is done.
#[derive(Default)]
pub struct Plenish {
    pub active: std::collections::VecDeque<(i32, i32, i32)>,
    pub used: std::collections::HashSet<(i32, i32, i32)>,
    pub done: bool,
}

/// `MekanismConfig.general.maxPlenisherNodes`.
pub const MAX_NODES: usize = 4096;

/// The block a fluid makes when poured: its source.
pub fn source_of(fluid: &str) -> Option<&'static str> {
    match fluid {
        "minecraft:water" => Some("minecraft:water[level=0]"),
        "minecraft:lava" => Some("minecraft:lava[level=0]"),
        _ => None,
    }
}

/// Whether the fluid may be poured into a block (`canReplace`): air, a fluid that is not a source, or a plant it would wash away.
pub fn replaceable(state: &str) -> bool {
    let name = state.split('[').next().unwrap_or("");
    if name == "minecraft:air" || name == "minecraft:cave_air" || name == "minecraft:void_air" {
        return true;
    }
    if name == "minecraft:water" || name == "minecraft:lava" {
        return source_fluid(state).is_none();
    }
    matches!(name, "minecraft:short_grass" | "minecraft:tall_grass" | "minecraft:fern" | "minecraft:snow" | "minecraft:fire" | "minecraft:vine" | "minecraft:seagrass" | "minecraft:kelp")
}

/// The blocks a plenisher spreads to from a node: the four sides and the one below.
pub fn spread(p: (i32, i32, i32)) -> [(i32, i32, i32); 5] {
    [(p.0, p.1 - 1, p.2), (p.0 - 1, p.1, p.2), (p.0 + 1, p.1, p.2), (p.0, p.1, p.2 - 1), (p.0, p.1, p.2 + 1)]
}

impl Mekanism {
    /// One operation of a plenisher: fills the next node with a bucket of the tank's fluid (the search runs first, then only the
    /// block below is refilled). Whether a bucket was spent.
    fn plenish(&mut self, e: &BlockEntity, key: &(String, i32, i32, i32), tank: &mut process::Tank) -> bool {
        let Some(source) = source_of(&tank.chemical) else { return false };
        let mut st = self.plenishers.remove(key).unwrap_or_default();
        let below = (e.pos.x, e.pos.y - 1, e.pos.z);
        let state_at = |at: (i32, i32, i32)| world::get_block(&e.dimension, at.0, at.1, at.2).unwrap_or_default();
        let pour = |at: (i32, i32, i32)| {
            let _ = world::set_blocks(&e.dimension, &[BlockChange { x: at.0, y: at.1, z: at.2, state: source.into() }]);
        };
        let mut spent = false;
        if st.done {
            if replaceable(&state_at(below)) {
                pour(below);
                spent = true;
            }
        } else if st.used.len() >= MAX_NODES {
            st.done = true;
        } else {
            if st.active.is_empty() {
                if st.used.is_empty() && replaceable(&state_at(below)) {
                    st.active.push_back(below);
                } else {
                    st.done = true;
                }
            }
            if let Some(node) = st.active.pop_front() {
                if replaceable(&state_at(node)) {
                    pour(node);
                    spent = true;
                }
                for next in spread(node) {
                    if !st.used.contains(&next) && !st.active.contains(&next) && replaceable(&state_at(next)) {
                        st.active.push_back(next);
                    }
                }
                st.used.insert(node);
            }
        }
        if spent {
            tank.amount -= BUCKET;
            if tank.amount == 0 {
                tank.chemical.clear();
            }
        }
        self.plenishers.insert(key.clone(), st);
        spent
    }

    /// Tries to suck the source at `at`: whether it did (the tank gets the bucket, a finite source turns to air).
    fn suck_at(&mut self, e: &BlockEntity, at: (i32, i32, i32), tank: &mut process::Tank, active: &str) -> bool {
        let Some(state) = world::get_block(&e.dimension, at.0, at.1, at.2) else { return false };
        let Some(fluid) = source_fluid(&state) else { return false };
        if !active.is_empty() && active != fluid {
            return false;
        }
        let Some(next) = take(tank, fluid) else { return false };
        if !infinite(fluid) {
            let _ = world::set_blocks(&e.dimension, &[BlockChange { x: at.0, y: at.1, z: at.2, state: "minecraft:air".into() }]);
        }
        *tank = next;
        true
    }

    /// One operation: the block below first, then the ones around the nodes found so far (shuffled by the tick), adding to them.
    fn suck(&mut self, e: &BlockEntity, key: &(String, i32, i32, i32), tank: &mut process::Tank, seed: u64) -> bool {
        let (mut nodes, mut active) = self.pump_nodes.remove(key).unwrap_or_default();
        let below = (e.pos.x, e.pos.y - 1, e.pos.z);
        let mut done = false;
        if self.suck_at(e, below, tank, &active) {
            active = tank.chemical.clone();
            if !nodes.contains(&below) {
                nodes.push(below);
            }
            done = true;
        }
        if !done {
            let n = nodes.len();
            for k in 0..n {
                let at = nodes[(k + (seed as usize % n.max(1))) % n];
                if self.suck_at(e, at, tank, &active) {
                    done = true;
                    break;
                }
                for next in around(at) {
                    let far = (next.0 - e.pos.x).abs().max((next.1 - e.pos.y).abs()).max((next.2 - e.pos.z).abs());
                    if far <= RANGE && self.suck_at(e, next, tank, &active) {
                        active = tank.chemical.clone();
                        if !nodes.contains(&next) {
                            nodes.push(next);
                        }
                        done = true;
                        break;
                    }
                }
                if done {
                    break;
                }
                nodes.retain(|p| *p != at);
                break;
            }
        }
        if done {
            self.pump_nodes.insert(key.clone(), (nodes, active));
        }
        done
    }

    /// A tick of an electric pump.
    pub(crate) fn run_pump(&mut self, def: &Def, e: &BlockEntity, tick: u64) -> Option<BlockEntityUpdate> {
        let layout = def.layout;
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
        let key = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
        let mut dirty = stored.is_none() || !self.applied.contains(&key);
        if let Some(click) = self.take_click(e) {
            // A sneak click with the configurator resets the pump's calculation (`TileEntityElectricPump` /
            // `TileEntityFluidicPlenisher` on sneak right click), instead of changing a side; the original has no
            // side configuration on a pump.
            if click.sneak && click.mode.is_configurating() {
                if def.form == 1 {
                    self.plenishers.remove(&key);
                    let _ = players::tell(&click.player, "Reset Fluidic Plenisher calculation");
                } else {
                    self.pump_nodes.remove(&key);
                    let _ = players::tell(&click.player, "Reset Electric Pump calculation");
                }
            } else {
                dirty |= self.configure(&mut cfg, &facing, layout, &click);
            }
        }
        let counts = upgrades::counts_of(def, e);
        let usage = upgrades::energy_per_tick(def.usage, counts[upgrades::SPEED], counts[upgrades::ENERGY]);
        let ticks = upgrades::ticks(if def.form == 1 { 20 } else { BASE_TICKS }, counts[upgrades::SPEED]);
        let capacity = upgrades::max_energy(base_capacity(def), counts[upgrades::ENERGY]);
        let slot = |i: usize| stack(e.items.get(i).unwrap_or(&None));
        let mut items: Vec<Option<Stack>> = (0..e.items.len()).map(slot).collect();
        let mut tank = plus::held(e, 0);
        // The charge slot, and the empty bucket in the top slot filling from the tank into the bottom one.
        let mut energy = e.energy;
        if e.items.len() >= 3 {
            let (e2, rest) = factory::charge(energy, capacity, items[2].clone());
            energy = e2;
            items[2] = rest;
            let (mut i0, mut o1) = (items[0].clone(), items[1].clone());
            if def.form == 0 && tank::buckets(&mut i0, &mut o1, &mut tank, TANK, EditMode::Empty) {
                items[0] = i0;
                items[1] = o1;
            }
        }
        let mut operating = progress;
        let mut status = Status::Idle;
        if def.form == 1 {
            // The plenisher: a full bucket in the top slot fills the tank; it pours while it has a bucket.
            if e.items.len() >= 3 {
                let (mut i0, mut o1) = (items[0].clone(), items[1].clone());
                if tank::buckets(&mut i0, &mut o1, &mut tank, TANK, EditMode::Fill) {
                    items[0] = i0;
                    items[1] = o1;
                }
            }
            if self.can && tank.amount >= BUCKET {
                if energy >= usage {
                    energy -= usage;
                    operating += 1;
                    status = Status::Running;
                    if operating >= ticks {
                        operating = 0;
                        if !self.plenish(e, &key, &mut tank) {
                            status = Status::Blocked;
                        }
                    }
                } else {
                    status = Status::NoEnergy;
                }
            }
        }
        let roomy = tank.amount == 0 || TANK - tank.amount >= BUCKET;
        if def.form == 1 {
        } else if self.can && roomy {
            if energy >= usage {
                energy -= usage;
                operating += 1;
                status = Status::Running;
                if operating >= ticks {
                    operating = 0;
                    let seed = tick.wrapping_mul(0x9E37_79B9).wrapping_add(e.pos.x as u64 ^ e.pos.z as u64);
                    if !self.suck(e, &key, &mut tank, seed) {
                        self.pump_nodes.remove(&key);
                        status = Status::Idle;
                    }
                }
            } else {
                status = Status::NoEnergy;
            }
        } else if !roomy {
            status = Status::Blocked;
        }
        let active = status == Status::Running;
        let lit = e.state.contains("lit=true");
        if active != lit {
            let _ = world::set_block_property(&e.dimension, e.pos.x, e.pos.y, e.pos.z, "lit", if active { "true" } else { "false" });
        }
        if def.form == 1 && !self.filtered.contains(&key) {
            let _ = world::set_tank_filter(&e.dimension, e.pos.x, e.pos.y, e.pos.z, 0, &["minecraft:lava".to_owned(), "minecraft:water".to_owned()]);
            self.filtered.insert(key.clone());
        }
        let energy_faces = dirty.then(|| self.apply_config(e, layout, &facing, &cfg));
        let mut data = menu_data(operating, energy, capacity, status, &cfg);
        data.extend(chem::screen(&tank));
        let control = Control::from_byte(e.data.get(redstone::offset(def.kind)).copied().unwrap_or(0));
        let mut saved = machine_data(operating, &cfg);
        let shown = (ticks, self.install.get(&key).copied().unwrap_or(0), counts, if active { usage } else { 0 });
        redstone::stamp(def.kind, control, capacity.saturating_sub(energy), shown, &facing, &mut saved, &mut data);
        let before = plus::held(e, 0);
        let changed = tank != before || energy != e.energy || items.iter().map(item).collect::<Vec<_>>().len() != e.items.len();
        let new_items: Vec<Option<ItemStack>> = items.iter().map(item).collect();
        let items_changed = new_items.iter().zip(e.items.iter()).any(|(a, b)| a.as_ref().map(|s| (&s.item, s.count)) != b.as_ref().map(|s| (&s.item, s.count)));
        if !changed && !items_changed && !dirty && data == e.menu_data && saved == e.data && !self.force {
            return None;
        }
        Some(BlockEntityUpdate {
            dimension: e.dimension.clone(),
            pos: e.pos,
            data: saved,
            items: Some(new_items),
            energy: Some(energy),
            energy_faces,
            tanks: (tank != before || e.tanks.len() != 1).then(|| vec![plus::as_tank(&tank)]),
            tank_faces: dirty.then(|| self.tank_sides(layout, &facing, &cfg, false)),
            menu_data: Some(data),
            transmitter_faces: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_sources_are_taken() {
        assert_eq!(source_fluid("minecraft:water[level=0]"), Some("minecraft:water"));
        assert_eq!(source_fluid("minecraft:lava"), Some("minecraft:lava"));
        assert_eq!(source_fluid("minecraft:water[level=3]"), None);
        assert_eq!(source_fluid("minecraft:stone"), None);
    }

    #[test]
    fn a_bucket_fits_one_kind_of_fluid() {
        let empty = process::Tank::default();
        assert_eq!(take(&empty, "minecraft:water").map(|t| t.amount), Some(1000));
        let lava = process::Tank { chemical: "minecraft:lava".into(), amount: 2000 };
        assert!(take(&lava, "minecraft:water").is_none());
        let full = process::Tank { chemical: "minecraft:lava".into(), amount: 9500 };
        assert!(take(&full, "minecraft:lava").is_none());
        assert!(infinite("minecraft:water") && !infinite("minecraft:lava"));
    }
}

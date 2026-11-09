//! The dynamic tank (`TankMultiblockData`, `TankValidator`): `mekanism:dynamic_tank` blocks that make the shell of a hollow cuboid
//! (3 to 18 blocks on a side, empty inside), `mekanism:dynamic_valve` blocks on its faces (not on an edge or a corner) that give access to
//! the one tank the structure holds: sixteen thousand buckets for each block of its inside, of a fluid or of a chemical, not both.
//!
//! The structure is found by the game (`multiblock-changed`); this validates it and writes what each member knows into its bytes
//! (`[valid, volume (4 bytes), min x, y, z (4 bytes each)]`). The valves hold the tank: all of them keep the same amount (a mirror of the
//! shared total); what one gains or loses in a tick is added to or taken from the total, which is then written to all of them again.
use super::*;
use std::collections::{HashMap, HashSet};

pub const CASING: &str = "mekanism:dynamic_tank";
pub const VALVE: &str = "mekanism:dynamic_valve";
pub const GROUP: &str = "mekanism:dynamic_tank";
/// `fluidPerTank`: 350 buckets (350,000 mB) for each block of the inside.
pub const FLUID_PER_BLOCK: u64 = 350 * 1000;
/// `chemicalPerTank`: 16,000 buckets (16,000,000 mB) for each block of the inside.
pub const CHEMICAL_PER_BLOCK: u64 = 16_000 * 1000;
/// Backwards-compatible alias for chemical capacity per block.
pub const PER_BLOCK: u64 = CHEMICAL_PER_BLOCK;
/// What a valve's tanks are registered with (the real capacity is the structure's): far more than any structure holds.
pub const SPEC_CAPACITY: u64 = 1 << 50;
/// What a valve moves a tick at most.
pub const VALVE_RATE: u64 = 1_000_000;
/// A substance nothing is made of: a full tank's filter, so nothing more is taken in.
pub const NOTHING: &str = "mekanism:nothing";

/// A found structure: whether it is a tank, and the box and volume of its inside.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Found {
    pub valid: bool,
    pub volume: u64,
    pub min: [i32; 3],
}

/// Whether the members of one candidate make a dynamic tank, given what each is (`is_valve`) and what the inside holds (`inside_empty`).
pub fn validate(members: &[([i32; 3], bool)], min: [i32; 3], max: [i32; 3], inside_empty: impl Fn([i32; 3]) -> bool) -> Found {
    let size = [max[0] - min[0] + 1, max[1] - min[1] + 1, max[2] - min[2] + 1];
    let bad = Found { valid: false, volume: 0, min };
    if size.iter().any(|s| !(3..=18).contains(s)) {
        return bad;
    }
    let have: HashMap<[i32; 3], bool> = members.iter().copied().collect();
    for x in min[0]..=max[0] {
        for y in min[1]..=max[1] {
            for z in min[2]..=max[2] {
                let p = [x, y, z];
                let edges = [x == min[0] || x == max[0], y == min[1] || y == max[1], z == min[2] || z == max[2]].iter().filter(|b| **b).count();
                match (edges, have.get(&p)) {
                    // The inside holds no member and nothing solid.
                    (0, Some(_)) => return bad,
                    (0, None) if !inside_empty(p) => return bad,
                    (0, None) => {}
                    // The shell is made of members; a valve is on a face alone.
                    (_, None) => return bad,
                    (e, Some(true)) if e > 1 => return bad,
                    _ => {}
                }
            }
        }
    }
    // Nothing else of the group is part of it.
    if members.len() != have.len() || members.iter().any(|(p, _)| (0..3).any(|i| p[i] < min[i] || p[i] > max[i])) {
        return bad;
    }
    let inside = ((size[0] - 2) * (size[1] - 2) * (size[2] - 2)) as u64;
    Found { valid: true, volume: inside, min }
}

/// What a member keeps of its structure.
pub fn encode(f: &Found) -> Vec<u8> {
    let mut out = vec![u8::from(f.valid)];
    out.extend((f.volume as u32).to_le_bytes());
    for c in f.min {
        out.extend(c.to_le_bytes());
    }
    out
}

pub fn decode(data: &[u8]) -> Option<Found> {
    if data.len() < 17 {
        return None;
    }
    let int = |at: usize| i32::from_le_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]);
    Some(Found { valid: data[0] == 1, volume: u64::from(u32::from_le_bytes([data[1], data[2], data[3], data[4]])), min: [int(5), int(9), int(13)] })
}

/// The total after a tick: what the valves held before, changed by what each gained or lost against the mirror it was given.
pub fn merge(total: u64, given: &[u64], now: &[u64], cap: u64) -> u64 {
    let mut t = i128::from(total);
    for (g, n) in given.iter().zip(now) {
        t += i128::from(*n) - i128::from(*g);
    }
    t.clamp(0, i128::from(cap)) as u64
}

/// The shared state of one structure between ticks.
#[derive(Default)]
pub struct Shared {
    pub fluid: process::Tank,
    pub chemical: process::Tank,
    /// What each valve was last given.
    pub given: HashMap<[i32; 3], (u64, u64)>,
}

impl Mekanism {
    /// The members of a candidate structure the game found: validates it and tells every member.
    pub(crate) fn dynamic_structures(&mut self, dimension: &str, structures: Vec<Structure>) -> Vec<BlockEntityUpdate> {
        let mut out = Vec::new();
        for s in structures {
            let members: Vec<([i32; 3], bool)> = s
                .members
                .iter()
                .map(|m| ([m.pos.x, m.pos.y, m.pos.z], m.state.starts_with(VALVE)))
                .collect();
            let min = [s.min.x, s.min.y, s.min.z];
            let max = [s.max.x, s.max.y, s.max.z];
            let found = validate(&members, min, max, |p| {
                world::get_block(dimension, p[0], p[1], p[2]).is_none_or(|b| {
                    let name = b.split('[').next().unwrap_or("");
                    matches!(name, "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air" | "minecraft:water" | "minecraft:lava")
                })
            });
            for m in &s.members {
                out.push(BlockEntityUpdate {
                    dimension: dimension.to_owned(),
                    pos: m.pos,
                    data: encode(&found),
                    items: None,
                    energy: None,
                    energy_faces: None,
                    tanks: None,
                    tank_faces: None,
                    menu_data: None,
                    transmitter_faces: None,
                });
            }
        }
        out
    }

    /// A tick of the valves of the dynamic tanks: groups them by structure and shares each structure's tank among them.
    pub(crate) fn run_dynamic(&mut self, valves: &[&BlockEntity]) -> Vec<BlockEntityUpdate> {
        let mut by: HashMap<(String, [i32; 3]), Vec<&BlockEntity>> = HashMap::new();
        for v in valves {
            if let Some(f) = decode(&v.data).filter(|f| f.valid) {
                by.entry((v.dimension.clone(), f.min)).or_default().push(*v);
            }
        }
        let mut out = Vec::new();
        for (key, group) in by {
            let volume = decode(&group[0].data).map_or(0, |f| f.volume);
            let fluid_cap = volume * FLUID_PER_BLOCK;
            let chem_cap = volume * CHEMICAL_PER_BLOCK;
            let shared = self.dynamic.entry(key.clone()).or_default();
            let fresh = shared.given.is_empty();
            let held = |v: &BlockEntity, i: usize| v.tanks.get(i).map_or(0, |t| t.amount);
            // After a load nothing was given yet: the structure holds what its valves do.
            if fresh {
                for i in 0..2 {
                    let cap = if i == 0 { fluid_cap } else { chem_cap };
                    if let Some(best) = group.iter().max_by_key(|v| held(v, i)) {
                        let t = best.tanks.get(i).cloned();
                        if let Some(t) = t.filter(|t| t.amount > 0) {
                            let tank = process::Tank { chemical: t.substance, amount: t.amount.min(cap) };
                            if i == 0 { shared.fluid = tank } else { shared.chemical = tank }
                        }
                    }
                }
                for v in &group {
                    let p = [v.pos.x, v.pos.y, v.pos.z];
                    shared.given.insert(p, (shared.fluid.amount, shared.chemical.amount));
                }
            }
            // What the valves took in or gave since they were last written.
            for i in 0..2 {
                let cap = if i == 0 { fluid_cap } else { chem_cap };
                let (given, now): (Vec<u64>, Vec<u64>) =
                    group.iter().map(|v| (shared.given.get(&[v.pos.x, v.pos.y, v.pos.z]).map_or(0, |g| if i == 0 { g.0 } else { g.1 }), held(v, i))).unzip();
                let total = if i == 0 { shared.fluid.amount } else { shared.chemical.amount };
                let next = merge(total, &given, &now, cap);
                // Which substance it is: the one a valve now holds that the structure did not.
                let name = group
                    .iter()
                    .filter_map(|v| v.tanks.get(i))
                    .find(|t| t.amount > 0)
                    .map(|t| t.substance.clone())
                    .unwrap_or_default();
                let tank = if i == 0 { &mut shared.fluid } else { &mut shared.chemical };
                if tank.amount == 0 && next > 0 {
                    tank.chemical = name;
                }
                tank.amount = next;
                if next == 0 {
                    tank.chemical.clear();
                }
            }
            // A tank holds a fluid or a chemical, not both.
            if shared.fluid.amount > 0 && shared.chemical.amount > 0 {
                shared.chemical = process::Tank::default();
            }
            let (fluid, chemical) = (shared.fluid.clone(), shared.chemical.clone());
            for v in &group {
                let p = [v.pos.x, v.pos.y, v.pos.z];
                shared.given.insert(p, (fluid.amount, chemical.amount));
            }
            let mut tanks = vec![plus::as_tank(&fluid), plus::as_tank(&chemical)];
            // The valve's slots: a bucket in the first fills or empties the shared tank.
            for v in &group {
                let slot = |i: usize| stack(v.items.get(i).unwrap_or(&None));
                let (mut i0, mut o1) = (slot(0), slot(1));
                let mut f = fluid.clone();
                let used = tank::buckets(&mut i0, &mut o1, &mut f, fluid_cap, tank::EditMode::Both);
                if used {
                    // A bucket moved fluid: it counts for this valve alone, the others follow next tick.
                    tanks[0] = plus::as_tank(&f);
                }
                let shown_cap = if chemical.amount > 0 { chem_cap } else if fluid.amount > 0 { fluid_cap } else { chem_cap };
                let data = dynamic_menu(&v.data, &f_or(&tanks, 0), &tanks[1], shown_cap);
                let _ = world::set_tank_filter(&v.dimension, v.pos.x, v.pos.y, v.pos.z, 0, &if fluid.amount >= fluid_cap || chemical.amount > 0 { vec![NOTHING.to_owned()] } else { Vec::new() });
                let _ = world::set_tank_filter(&v.dimension, v.pos.x, v.pos.y, v.pos.z, 1, &if chemical.amount >= chem_cap || fluid.amount > 0 { vec![NOTHING.to_owned()] } else { Vec::new() });
                let items = used.then(|| vec![item(&i0), item(&o1)]);
                if used {
                    shared.fluid = f;
                    shared.given.insert([v.pos.x, v.pos.y, v.pos.z], (shared.fluid.amount, chemical.amount));
                }
                out.push(BlockEntityUpdate {
                    dimension: v.dimension.clone(),
                    pos: v.pos,
                    data: v.data.clone(),
                    items,
                    energy: None,
                    energy_faces: None,
                    tanks: Some(tanks.clone()),
                    tank_faces: None,
                    menu_data: Some(data),
                    transmitter_faces: None,
                });
            }
        }
        out
    }
}

fn f_or(tanks: &[Tank], i: usize) -> Tank {
    tanks[i].clone()
}

/// The screen's numbers: the amount and the capacity in buckets (low and high halves each), the chemical's number, the volume (halves).
pub fn dynamic_menu(structure: &[u8], tank: &Tank, chemical_tank: &Tank, cap: u64) -> Vec<i32> {
    let half = |v: u64, s: u32| ((v >> s) & 0xFFFF) as i32;
    let (shown, name) = if chemical_tank.amount > 0 { (chemical_tank, &chemical_tank.substance) } else { (tank, &tank.substance) };
    let buckets = shown.amount / 1000;
    let cap_b = cap / 1000;
    let volume = decode(structure).map_or(0, |f| f.volume);
    vec![
        half(buckets, 0),
        half(buckets, 16),
        if shown.amount == 0 { 0 } else { chem::index(name) },
        half(cap_b, 0),
        half(cap_b, 16),
        half(volume, 0),
        half(volume, 16),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A hollow 3x3x3 box of casing with a valve in the middle of its top face.
    fn shell(n: i32) -> Vec<([i32; 3], bool)> {
        let mut out = Vec::new();
        for x in 0..n {
            for y in 0..n {
                for z in 0..n {
                    let edge = |c: i32| c == 0 || c == n - 1;
                    let e = [edge(x), edge(y), edge(z)].iter().filter(|b| **b).count();
                    if e > 0 {
                        out.push(([x, y, z], (x, y, z) == (n / 2, n - 1, n / 2)));
                    }
                }
            }
        }
        out
    }

    #[test]
    fn a_hollow_box_is_a_tank() {
        let f = validate(&shell(3), [0; 3], [2; 3], |_| true);
        assert_eq!((f.valid, f.volume), (true, 1));
        let big = validate(&shell(5), [0; 3], [4; 3], |_| true);
        assert_eq!((big.valid, big.volume), (true, 27));
    }

    #[test]
    fn it_has_to_be_hollow_closed_and_sized_right() {
        assert!(!validate(&shell(3), [0; 3], [2; 3], |_| false).valid);
        let mut hole = shell(4);
        hole.retain(|(p, _)| *p != [0, 1, 1]);
        assert!(!validate(&hole, [0; 3], [3; 3], |_| true).valid);
        assert!(!validate(&shell(2), [0; 3], [1; 3], |_| true).valid);
        assert!(!validate(&shell(19), [0; 3], [18; 3], |_| true).valid);
        // A valve on an edge is not allowed.
        let mut edge = shell(4);
        for (p, v) in &mut edge {
            *v = *p == [0, 0, 1];
        }
        assert!(!validate(&edge, [0; 3], [3; 3], |_| true).valid);
        // A member inside is not allowed.
        let mut inner = shell(5);
        inner.push(([2, 2, 2], false));
        assert!(!validate(&inner, [0; 3], [4; 3], |_| true).valid);
    }

    #[test]
    fn what_valves_gain_and_lose_is_merged() {
        // Two valves given 1000 each: one took 300 in, the other gave 100 out.
        assert_eq!(merge(1000, &[1000, 1000], &[1300, 900], 10_000), 1200);
        assert_eq!(merge(1000, &[1000], &[0], 10_000), 0);
        assert_eq!(merge(9_900, &[9_900], &[10_500], 10_000), 10_000);
    }

    #[test]
    fn a_member_remembers_its_structure() {
        let f = Found { valid: true, volume: 27, min: [-5, 64, 300] };
        assert_eq!(decode(&encode(&f)), Some(f));
        assert_eq!(decode(&[1, 2]), None);
    }

    #[test]
    fn tank_capacity_fluid_vs_chemical() {
        let f = Found { valid: true, volume: 10, min: [0; 3] };
        assert_eq!(f.volume * FLUID_PER_BLOCK, 3_500_000);
        assert_eq!(f.volume * CHEMICAL_PER_BLOCK, 160_000_000);
    }
}

//! The induction matrix (`MatrixMultiblockData`, `MatrixValidator`): a hollow cuboid (3 to 18 on a side) of `mekanism:induction_casing`
//! with `mekanism:induction_port` blocks on its faces, whose inside holds induction cells (what it stores) and providers (how fast it
//! takes in and gives out) and is empty otherwise. The ports are where energy goes in and out.
//!
//! Like the dynamic tank, the game finds the structure and this validates it; every port holds a mirror of the shared energy (at most
//! what the providers move in a tick, which is also all the room it has), what a port gains or loses is added to or taken from the total,
//! and the total is kept in the ports' bytes so it survives a reload.
use super::*;
use std::collections::HashMap;

pub const CASING: &str = "mekanism:induction_casing";
pub const PORT: &str = "mekanism:induction_port";
pub const GROUP: &str = "mekanism:induction";
/// `InductionCellTier` and `InductionProviderTier`: what a cell stores and what a provider moves each tick.
pub const CELLS: [(&str, u64); 4] = [
    ("mekanism:basic_induction_cell", 8_000_000_000),
    ("mekanism:advanced_induction_cell", 64_000_000_000),
    ("mekanism:elite_induction_cell", 512_000_000_000),
    ("mekanism:ultimate_induction_cell", 4_000_000_000_000),
];
pub const PROVIDERS: [(&str, u64); 4] = [
    ("mekanism:basic_induction_provider", 256_000),
    ("mekanism:advanced_induction_provider", 2_048_000),
    ("mekanism:elite_induction_provider", 16_384_000),
    ("mekanism:ultimate_induction_provider", 131_072_000),
];
/// What a port is registered to hold: far more than a matrix can (the real room is set each tick).
pub const SPEC_CAPACITY: u64 = 1 << 60;

/// A found structure.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Found {
    pub valid: bool,
    pub min: [i32; 3],
    pub capacity: u64,
    pub rate: u64,
    pub cells: u16,
    pub providers: u16,
}

/// What a member is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Casing,
    Port,
    Cell(u64),
    Provider(u64),
}

pub fn part_of(state: &str) -> Option<Part> {
    let name = state.split('[').next().unwrap_or("");
    if name == CASING {
        return Some(Part::Casing);
    }
    if name == PORT {
        return Some(Part::Port);
    }
    CELLS.iter().find(|(n, _)| *n == name).map(|(_, c)| Part::Cell(*c)).or_else(|| PROVIDERS.iter().find(|(n, _)| *n == name).map(|(_, r)| Part::Provider(*r)))
}

/// Whether the members make a matrix: the shell of casing and ports (ports on a face, not an edge), the inside cells and providers and
/// air, and a size of 3 to 18.
pub fn validate(members: &[([i32; 3], Part)], min: [i32; 3], max: [i32; 3], inside_empty: impl Fn([i32; 3]) -> bool) -> Found {
    let size = [max[0] - min[0] + 1, max[1] - min[1] + 1, max[2] - min[2] + 1];
    let bad = Found { min, ..Found::default() };
    if size.iter().any(|s| !(3..=18).contains(s)) {
        return bad;
    }
    let have: HashMap<[i32; 3], Part> = members.iter().copied().collect();
    let (mut capacity, mut rate, mut cells, mut providers) = (0u64, 0u64, 0u16, 0u16);
    for x in min[0]..=max[0] {
        for y in min[1]..=max[1] {
            for z in min[2]..=max[2] {
                let p = [x, y, z];
                let edges = [x == min[0] || x == max[0], y == min[1] || y == max[1], z == min[2] || z == max[2]].iter().filter(|b| **b).count();
                match (edges, have.get(&p)) {
                    (0, Some(Part::Cell(c))) => {
                        capacity += c;
                        cells += 1;
                    }
                    (0, Some(Part::Provider(r))) => {
                        rate += r;
                        providers += 1;
                    }
                    (0, Some(_)) => return bad,
                    (0, None) if !inside_empty(p) => return bad,
                    (0, None) => {}
                    (_, Some(Part::Casing)) => {}
                    (e, Some(Part::Port)) if e == 1 => {}
                    _ => return bad,
                }
            }
        }
    }
    if members.iter().any(|(p, _)| (0..3).any(|i| p[i] < min[i] || p[i] > max[i])) || cells == 0 {
        return bad;
    }
    Found { valid: true, min, capacity, rate, cells, providers }
}

/// A member's bytes: validity, the box's minimum corner, the capacity, the rate and the counts, then the energy the structure holds.
pub fn encode(f: &Found, total: u64) -> Vec<u8> {
    let mut out = vec![u8::from(f.valid)];
    for c in f.min {
        out.extend(c.to_le_bytes());
    }
    out.extend(f.capacity.to_le_bytes());
    out.extend(f.rate.to_le_bytes());
    out.extend(f.cells.to_le_bytes());
    out.extend(f.providers.to_le_bytes());
    out.extend(total.to_le_bytes());
    out
}

pub fn decode(data: &[u8]) -> Option<(Found, u64)> {
    if data.len() < 33 {
        return None;
    }
    let int = |at: usize| i32::from_le_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]);
    let long = |at: usize| u64::from_le_bytes(data[at..at + 8].try_into().unwrap_or([0; 8]));
    let short = |at: usize| u16::from_le_bytes([data[at], data[at + 1]]);
    let found = Found { valid: data[0] == 1, min: [int(1), int(5), int(9)], capacity: long(13), rate: long(21), cells: short(29), providers: short(31) };
    Some((found, data.get(33..41).map_or(0, |_| long(33))))
}

/// The energy after a tick: what the ports held before, changed by what each gained or lost against the mirror it was given.
pub fn merge(total: u64, given: u64, now: &[u64], capacity: u64) -> u64 {
    let mut t = i128::from(total);
    for n in now {
        t += i128::from(*n) - i128::from(given);
    }
    t.clamp(0, i128::from(capacity)) as u64
}

impl Mekanism {
    /// The candidate structures the game found: validates each and tells every member.
    pub(crate) fn matrix_structures(&mut self, dimension: &str, structures: Vec<Structure>) -> Vec<BlockEntityUpdate> {
        let mut out = Vec::new();
        for s in structures {
            let members: Vec<([i32; 3], Part)> = s.members.iter().filter_map(|m| part_of(&m.state).map(|p| ([m.pos.x, m.pos.y, m.pos.z], p))).collect();
            let min = [s.min.x, s.min.y, s.min.z];
            let max = [s.max.x, s.max.y, s.max.z];
            let found = validate(&members, min, max, |p| {
                world::get_block(dimension, p[0], p[1], p[2]).is_none_or(|b| matches!(b.split('[').next().unwrap_or(""), "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"))
            });
            for m in &s.members {
                if part_of(&m.state) == Some(Part::Port) {
                    // A port holds at most what the providers move in a tick (nothing of its own in a broken structure).
                    let _ = world::set_energy_capacity(dimension, m.pos.x, m.pos.y, m.pos.z, if found.valid { found.rate.max(1) } else { 1 });
                }
                let total = self.matrices.get(&(dimension.to_owned(), found.min)).copied().unwrap_or(0).min(found.capacity);
                out.push(BlockEntityUpdate {
                    dimension: dimension.to_owned(),
                    pos: m.pos,
                    data: encode(&found, total),
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

    /// A tick of the ports of the matrices.
    pub(crate) fn run_matrix(&mut self, ports: &[&BlockEntity]) -> Vec<BlockEntityUpdate> {
        let mut by: HashMap<(String, [i32; 3]), Vec<&BlockEntity>> = HashMap::new();
        for p in ports {
            if let Some((f, _)) = decode(&p.data).filter(|(f, _)| f.valid) {
                by.entry((p.dimension.clone(), f.min)).or_default().push(*p);
            }
        }
        let mut out = Vec::new();
        for (key, group) in by {
            let Some((found, stored)) = decode(&group[0].data) else { continue };
            let known = self.matrices.get(&key).copied();
            // After a load the structure holds what its ports remember.
            let total = known.unwrap_or(stored.min(found.capacity));
            let given = self.matrix_given.get(&key).copied().unwrap_or_else(|| total.min(found.rate));
            let now: Vec<u64> = group.iter().map(|p| p.energy).collect();
            let next = merge(total, given, &now, found.capacity);
            self.matrices.insert(key.clone(), next);
            let mirror = next.min(found.rate);
            self.matrix_given.insert(key.clone(), mirror);
            let mega = |v: u64| v / 1_000_000;
            let half = |v: u64, s: u32| ((v >> s) & 0xFFFF) as i32;
            let data = vec![
                half(mega(next), 0),
                half(mega(next), 16),
                half(mega(found.capacity), 0),
                half(mega(found.capacity), 16),
                half(found.rate / 1000, 0),
                half(found.rate / 1000, 16),
                i32::from(found.cells),
                i32::from(found.providers),
            ];
            for p in &group {
                // The room a port has is the most the providers move in a tick, less what it holds.
                let _ = world::set_energy_capacity(&p.dimension, p.pos.x, p.pos.y, p.pos.z, found.rate.max(1));
                let changed = next != stored || p.energy != mirror;
                out.push(BlockEntityUpdate {
                    dimension: p.dimension.clone(),
                    pos: p.pos,
                    data: if changed { encode(&found, next) } else { p.data.clone() },
                    items: None,
                    energy: Some(mirror),
                    energy_faces: None,
                    tanks: None,
                    tank_faces: None,
                    menu_data: Some(data.clone()),
                    transmitter_faces: None,
                });
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A hollow box of `n` with a port at the middle of its top face and a basic cell and provider inside (when `n` > 3, two cells).
    fn matrix(n: i32) -> Vec<([i32; 3], Part)> {
        let mut out = Vec::new();
        for x in 0..n {
            for y in 0..n {
                for z in 0..n {
                    let edge = |c: i32| c == 0 || c == n - 1;
                    let e = [edge(x), edge(y), edge(z)].iter().filter(|b| **b).count();
                    if e > 0 {
                        out.push(([x, y, z], if (x, y, z) == (n / 2, n - 1, n / 2) { Part::Port } else { Part::Casing }));
                    }
                }
            }
        }
        out.push(([1, 1, 1], Part::Cell(8_000_000_000)));
        out.push(([1, 1, 2], Part::Provider(256_000)));
        out
    }

    #[test]
    fn a_matrix_adds_up_its_cells_and_providers() {
        let f = validate(&matrix(4), [0; 3], [3; 3], |_| true);
        assert_eq!((f.valid, f.capacity, f.rate, f.cells, f.providers), (true, 8_000_000_000, 256_000, 1, 1));
    }

    #[test]
    fn it_needs_a_cell_and_a_closed_shell() {
        let mut none = matrix(4);
        none.retain(|(_, p)| !matches!(p, Part::Cell(_)));
        assert!(!validate(&none, [0; 3], [3; 3], |_| true).valid);
        let mut hole = matrix(4);
        hole.retain(|(p, _)| *p != [0, 1, 1]);
        assert!(!validate(&hole, [0; 3], [3; 3], |_| true).valid);
        // A port on an edge, a cell in the shell.
        let mut edge = matrix(4);
        edge.push(([0, 0, 1], Part::Port));
        edge.retain(|(p, q)| !(*p == [0, 0, 1] && *q == Part::Casing));
        assert!(!validate(&edge, [0; 3], [3; 3], |_| true).valid);
        assert!(!validate(&matrix(4), [0; 3], [3; 3], |_| false).valid);
    }

    #[test]
    fn the_energy_follows_the_ports_and_is_kept_in_their_bytes() {
        let f = Found { valid: true, min: [1, 2, 3], capacity: 9_000, rate: 100, cells: 2, providers: 1 };
        assert_eq!(decode(&encode(&f, 777)), Some((f, 777)));
        // The ports were given 100 each: one took 40 in, another gave 30 out.
        assert_eq!(merge(500, 100, &[140, 70], 9_000), 510);
        assert_eq!(merge(10, 100, &[0], 9_000), 0);
        assert_eq!(merge(8_990, 100, &[200], 9_000), 9_000);
    }
}

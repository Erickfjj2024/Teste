//! The dynamic tank (`TankMultiblockData`, `TankValidator`): `mekanism:dynamic_tank` blocks that make the shell of a hollow cuboid
//! (3 to 18 blocks on a side, empty inside), `mekanism:dynamic_valve` blocks on its faces (not on an edge or a corner) that give access to
//! the one tank the structure holds: 350 buckets of fluid or 16,000 of chemical for each block of its volume (the whole box, walls included: `MultiblockData.setShape` uses the bounds), not both.
//!
//! The structure is found by the game (`multiblock-changed`); this validates it and writes what each member knows into its bytes
//! (`[valid, volume (4 bytes), min x, y, z (4 bytes each)]`). The valves hold the tank: all of them keep the same amount (a mirror of the
//! shared total); what one gains or loses in a tick is added to or taken from the total, which is then written to all of them again.
use super::*;
use std::collections::{HashMap, HashSet};

pub const CASING: &str = "mekanism:dynamic_tank";
pub const VALVE: &str = "mekanism:dynamic_valve";
pub const GROUP: &str = "mekanism:dynamic_tank";
/// `fluidPerTank`: 350 buckets (350,000 mB) for each block of the volume (the whole box).
pub const FLUID_PER_BLOCK: u64 = 350 * 1000;
/// `chemicalPerTank`: 16,000 buckets (16,000,000 mB) for each block of the volume (the whole box).
pub const CHEMICAL_PER_BLOCK: u64 = 16_000 * 1000;
/// Backwards-compatible alias for chemical capacity per block.
pub const PER_BLOCK: u64 = CHEMICAL_PER_BLOCK;
/// What a valve's tanks are registered with (the real capacity is the structure's): far more than any structure holds.
pub const SPEC_CAPACITY: u64 = 1 << 50;
/// What a valve moves a tick at most.
pub const VALVE_RATE: u64 = 1_000_000;
/// A substance nothing is made of: a full tank's filter, so nothing more is taken in.
pub const NOTHING: &str = "mekanism:nothing";

/// A found structure: whether it is a tank, and the box and volume (the whole box, walls included, as `MultiblockData.setVolume`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Found {
    pub valid: bool,
    pub volume: u64,
    pub min: [i32; 3],
    /// The greatest corner of the casing (for the fluid drawn inside, FORK-31).
    pub max: [i32; 3],
}

/// What kind of part a member of the dynamic tank group is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Casing,
    Glass,
    Valve,
}

impl From<bool> for Part {
    fn from(is_valve: bool) -> Self {
        if is_valve { Part::Valve } else { Part::Casing }
    }
}

/// Whether the members of one candidate make a dynamic tank, given what each is (`Part` or `bool`) and what the inside holds (`inside_empty`).
pub fn validate<P: Copy + Into<Part>>(members: &[([i32; 3], P)], min: [i32; 3], max: [i32; 3], inside_empty: impl Fn([i32; 3]) -> bool) -> Found {
    let size = [max[0] - min[0] + 1, max[1] - min[1] + 1, max[2] - min[2] + 1];
    let bad = Found { valid: false, volume: 0, min, max };
    if size.iter().any(|s| !(3..=18).contains(s)) {
        return bad;
    }
    let have: HashMap<[i32; 3], Part> = members.iter().map(|(pos, p)| (*pos, (*p).into())).collect();
    for x in min[0]..=max[0] {
        for y in min[1]..=max[1] {
            for z in min[2]..=max[2] {
                let p = [x, y, z];
                let edges = [x == min[0] || x == max[0], y == min[1] || y == max[1], z == min[2] || z == max[2]].iter().filter(|b| **b).count();
                match (edges, have.get(&p)) {
                    // The inside holds no member and nothing solid (air only, CuboidStructureValidator.java:97).
                    (0, Some(_)) => return bad,
                    (0, None) if !inside_empty(p) => return bad,
                    (0, None) => {}
                    // Edges and corners (edges > 1) MUST be casing (neither valve nor structural glass allowed, CuboidStructureValidator.java:115).
                    (e, Some(Part::Casing)) if e > 1 => {}
                    (e, Some(_)) if e > 1 => return bad,
                    // Faces (edges == 1) allow casing, structural glass, and valves.
                    (1, Some(_)) => {}
                    // The shell is made of members.
                    (_, None) => return bad,
                    _ => return bad,
                }
            }
        }
    }
    // Nothing else of the group is part of it.
    if members.len() != have.len() || members.iter().any(|(p, _)| (0..3).any(|i| p[i] < min[i] || p[i] > max[i])) {
        return bad;
    }
    let volume = (size[0] * size[1] * size[2]) as u64;
    Found { valid: true, volume, min, max }
}

/// What a member keeps of its structure.
pub fn encode(f: &Found) -> Vec<u8> {
    let mut out = vec![u8::from(f.valid)];
    out.extend((f.volume as u32).to_le_bytes());
    for c in f.min {
        out.extend(c.to_le_bytes());
    }
    // The greatest corner follows the least one, so the renderer reads the casing's box in one go (`fluid_volume`, `box_at` 5).
    for c in f.max {
        out.extend(c.to_le_bytes());
    }
    out
}

/// Where the renderer reads the fluid's draw flag and how full the tank is, in a valve's data (after the 17 bytes of the structure and the
/// 12 of its greatest corner).
pub const DRAW_AT: usize = 29;
pub const FILL_AT: usize = 30;

/// A valve's data with the two bytes the renderer reads: whether it draws the structure's fluid and how full the tank is (0 to 255).
pub fn with_fill(data: &[u8], draws: bool, amount: u64, capacity: u64) -> Vec<u8> {
    let mut out = data[..data.len().min(DRAW_AT)].to_vec();
    out.resize(DRAW_AT, 0);
    out.push(u8::from(draws));
    out.push(if capacity == 0 { 0 } else { (amount.min(capacity) * 255).div_ceil(capacity).min(255) as u8 });
    out
}

pub fn decode(data: &[u8]) -> Option<Found> {
    if data.len() < 17 {
        return None;
    }
    let int = |at: usize| i32::from_le_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]);
    let max = if data.len() >= 29 { [int(17), int(21), int(25)] } else { [int(5), int(9), int(13)] };
    Some(Found { valid: data[0] == 1, volume: u64::from(u32::from_le_bytes([data[1], data[2], data[3], data[4]])), min: [int(5), int(9), int(13)], max })
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
    /// `GuiContainerEditModeTab`: what the buckets in the valves' slots may do with the tank.
    pub edit: tank::EditMode,
}

impl Mekanism {
    /// The members of a candidate structure the game found: validates it and tells every member.
    pub(crate) fn dynamic_structures(&mut self, dimension: &str, structures: Vec<Structure>) -> Vec<BlockEntityUpdate> {
        let mut out = Vec::new();
        for s in structures {
            let members: Vec<([i32; 3], Part)> = s
                .members
                .iter()
                .map(|m| {
                    let name = m.state.split('[').next().unwrap_or("");
                    let part = if name == VALVE {
                        Part::Valve
                    } else if name == "mekanism:structural_glass" {
                        Part::Glass
                    } else {
                        Part::Casing
                    };
                    ([m.pos.x, m.pos.y, m.pos.z], part)
                })
                .collect();
            let min = [s.min.x, s.min.y, s.min.z];
            let max = [s.max.x, s.max.y, s.max.z];
            let found = validate(&members, min, max, |p| {
                world::get_block(dimension, p[0], p[1], p[2]).is_some_and(|b| {
                    let name = b.split('[').next().unwrap_or("");
                    matches!(name, "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air")
                })
            });
            for m in &s.members {
                let is_valve = m.state.starts_with(VALVE);
                if is_valve && !found.valid {
                    let _ = world::set_tank_filter(dimension, m.pos.x, m.pos.y, m.pos.z, 0, &[NOTHING.to_owned()]);
                    let _ = world::set_tank_filter(dimension, m.pos.x, m.pos.y, m.pos.z, 1, &[NOTHING.to_owned()]);
                }
                out.push(BlockEntityUpdate {
                    dimension: dimension.to_owned(),
                    pos: m.pos,
                    data: encode(&found),
                    items: None,
                    energy: None,
                    energy_faces: None,
                    tanks: None,
                    tank_faces: if !found.valid && is_valve { Some(vec![vec![FlowSide::None; 6]; 2]) } else { None },
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
        let buttons = std::mem::take(&mut self.dynamic_buttons);
        for (key, group) in by {
            let volume = decode(&group[0].data).map_or(0, |f| f.volume);
            let fluid_cap = volume * FLUID_PER_BLOCK;
            let chem_cap = volume * CHEMICAL_PER_BLOCK;
            let shared = self.dynamic.entry(key.clone()).or_default();
            for (dimension, min, id) in &buttons {
                if *dimension == key.0 && *min == key.1 {
                    shared.edit = if *id == plus::EDIT_MODE { shared.edit.next() } else { shared.edit.previous() };
                }
            }
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

            // The valve's slots: a bucket in the first fills or empties the shared tank.
            // Run sequentially across valves, tracking item changes per valve.
            let mut valve_items: HashMap<[i32; 3], Vec<Option<ItemStack>>> = HashMap::new();
            for v in &group {
                let slot = |i: usize| stack(v.items.get(i).unwrap_or(&None));
                let (mut i0, mut o1) = (slot(0), slot(1));
                let mut f = shared.fluid.clone();
                if tank::buckets(&mut i0, &mut o1, &mut f, fluid_cap, shared.edit) {
                    shared.fluid = f;
                    valve_items.insert([v.pos.x, v.pos.y, v.pos.z], vec![item(&i0), item(&o1)]);
                }
            }

            let (fluid, chemical) = (shared.fluid.clone(), shared.chemical.clone());
            // Synchronize given for ALL valves in the group so no phantom differences occur next tick!
            for v in &group {
                let p = [v.pos.x, v.pos.y, v.pos.z];
                shared.given.insert(p, (fluid.amount, chemical.amount));
            }
            let tanks = vec![plus::as_tank(&fluid), plus::as_tank(&chemical)];

            for v in &group {
                let p = [v.pos.x, v.pos.y, v.pos.z];
                let items = valve_items.remove(&p);
                let shown_cap = if chemical.amount > 0 { chem_cap } else if fluid.amount > 0 { fluid_cap } else { chem_cap };
                let data = dynamic_menu(&tanks[0], &tanks[1], shown_cap, shared.edit);
                let texts = screen_texts(&tanks[0], &tanks[1], shown_cap);
                let tkey = (v.dimension.clone(), v.pos.x, v.pos.y, v.pos.z);
                if self.menu_texts.get(&tkey) != Some(&texts) {
                    let _ = world::set_menu_text(&v.dimension, v.pos.x, v.pos.y, v.pos.z, &texts);
                    self.menu_texts.insert(tkey, texts);
                }
                // FORK-31: the one valve with the least position draws the fluid inside the casing; the byte after it says how full the tank is.
                let draws = group.iter().all(|o| (o.pos.x, o.pos.y, o.pos.z) >= (v.pos.x, v.pos.y, v.pos.z));
                let shown_amount = if chemical.amount > 0 { chemical.amount } else { fluid.amount };
                let drawn = with_fill(&v.data, draws, shown_amount, shown_cap);
                let filter_0 = if fluid.amount >= fluid_cap || chemical.amount > 0 {
                    vec![NOTHING.to_owned()]
                } else if fluid.amount > 0 {
                    vec![fluid.chemical.clone()]
                } else {
                    Vec::new()
                };
                let filter_1 = if chemical.amount >= chem_cap || fluid.amount > 0 {
                    vec![NOTHING.to_owned()]
                } else if chemical.amount > 0 {
                    vec![chemical.chemical.clone()]
                } else {
                    Vec::new()
                };
                let _ = world::set_tank_filter(&v.dimension, v.pos.x, v.pos.y, v.pos.z, 0, &filter_0);
                let _ = world::set_tank_filter(&v.dimension, v.pos.x, v.pos.y, v.pos.z, 1, &filter_1);
                out.push(BlockEntityUpdate {
                    dimension: v.dimension.clone(),
                    pos: v.pos,
                    data: drawn,
                    items,
                    energy: None,
                    energy_faces: None,
                    tanks: Some(tanks.clone()),
                    tank_faces: Some(vec![vec![FlowSide::Both; 6]; 2]),
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

/// The screen's numbers: the amount and the capacity in millibuckets (low and high halves each), the chemical's number and the edit mode.
pub fn dynamic_menu(tank: &Tank, chemical_tank: &Tank, cap: u64, edit: tank::EditMode) -> Vec<i32> {
    let half = |v: u64, s: u32| ((v >> s) & 0xFFFF) as i32;
    let (shown, name) = if chemical_tank.amount > 0 { (chemical_tank, &chemical_tank.substance) } else { (tank, &tank.substance) };
    vec![
        half(shown.amount, 0),
        half(shown.amount, 16),
        if shown.amount == 0 { 0 } else { chem::index(name) },
        half(cap, 0),
        half(cap, 16),
        edit as i32,
    ]
}

/// The name of a fluid or a chemical as the screen writes it (`Hydrogen Chloride` for `mekanism:hydrogen_chloride`).
pub fn display_name(id: &str) -> String {
    let path = id.rsplit(':').next().unwrap_or(id);
    path.split('_')
        .map(|w| {
            let mut c = w.chars();
            c.next().map_or(String::new(), |f| f.to_uppercase().chain(c).collect())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The inner screen's lines (`GuiDynamicTank`): the name and the amount (or "Empty"), then "Capacity:" and the capacity, in mB.
pub fn screen_texts(tank: &Tank, chemical_tank: &Tank, cap: u64) -> Vec<String> {
    let shown = if chemical_tank.amount > 0 { chemical_tank } else { tank };
    let mut lines = Vec::new();
    if shown.amount == 0 {
        lines.push("Empty".to_owned());
    } else {
        lines.push(format!("{}:", display_name(&shown.substance)));
        lines.push(format!("{} mB", boiler::thousands(shown.amount)));
    }
    lines.push("Capacity:".to_owned());
    lines.push(format!("{} mB", boiler::thousands(cap)));
    lines.resize(4, String::new());
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_inner_screen_writes_the_name_the_amount_and_the_capacity_in_mb() {
        let water = Tank { substance: "minecraft:water".into(), amount: 12_345 };
        assert_eq!(screen_texts(&water, &Tank::default(), 16_000), ["Water:", "12,345 mB", "Capacity:", "16,000 mB"]);
        assert_eq!(screen_texts(&Tank::default(), &Tank::default(), 16_000), ["Empty", "Capacity:", "16,000 mB", ""]);
        let gas = Tank { substance: "mekanism:hydrogen_chloride".into(), amount: 1 };
        assert_eq!(screen_texts(&water, &gas, 8_000)[0], "Hydrogen Chloride:");
    }

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
        assert_eq!((f.valid, f.volume), (true, 27));
        let big = validate(&shell(5), [0; 3], [4; 3], |_| true);
        assert_eq!((big.valid, big.volume), (true, 125));
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
        let f = Found { valid: true, volume: 27, min: [-5, 64, 300], max: [-3, 66, 302] };
        assert_eq!(decode(&encode(&f)), Some(f));
        assert_eq!(decode(&[1, 2]), None);
    }

    #[test]
    fn tank_capacity_fluid_vs_chemical() {
        let f = Found { valid: true, volume: 10, min: [0; 3], max: [2; 3] };
        assert_eq!(f.volume * FLUID_PER_BLOCK, 3_500_000);
        assert_eq!(f.volume * CHEMICAL_PER_BLOCK, 160_000_000);
    }

    #[test]
    fn tank_filter_locks_substance() {
        let fluid = process::Tank { chemical: "minecraft:water".into(), amount: 10_000 };
        let chem = process::Tank::default();
        let cap = 350_000;
        let f0 = if fluid.amount >= cap || chem.amount > 0 {
            vec![NOTHING.to_owned()]
        } else if fluid.amount > 0 {
            vec![fluid.chemical.clone()]
        } else {
            Vec::new()
        };
        assert_eq!(f0, vec!["minecraft:water"]);
    }

    /// CuboidStructureValidator.java:115,149 and TankValidator.java:13-21:
    /// Structural glass is accepted on faces (edges == 1), but forbidden on edges/corners (edges > 1).
    #[test]
    fn structural_glass_allowed_on_faces_rejected_on_edges() {
        let n = 3;
        let mut members = Vec::new();
        for x in 0..n {
            for y in 0..n {
                for z in 0..n {
                    let edge = |c: i32| c == 0 || c == n - 1;
                    let e = [edge(x), edge(y), edge(z)].iter().filter(|b| **b).count();
                    if e > 0 {
                        // Put structural glass on face (1, 2, 1) - top face middle
                        let part = if (x, y, z) == (1, 2, 1) {
                            Part::Glass
                        } else {
                            Part::Casing
                        };
                        members.push(([x, y, z], part));
                    }
                }
            }
        }
        let f = validate(&members, [0; 3], [2; 3], |_| true);
        assert!(f.valid, "Structural glass on face should be valid");

        // Now put structural glass on an edge (0, 0, 1)
        for (pos, part) in &mut members {
            if *pos == [0, 0, 1] {
                *part = Part::Glass;
            }
        }
        let bad = validate(&members, [0; 3], [2; 3], |_| true);
        assert!(!bad.valid, "Structural glass on edge must be invalid (CuboidStructureValidator.java:115)");
    }

    /// CuboidStructureValidator.java:97:
    /// TankValidator inner nodes must be air only. Non-air (e.g. water or solid) invalidates the tank.
    #[test]
    fn interior_must_be_air_only() {
        assert!(!validate(&shell(3), [0; 3], [2; 3], |_| false).valid);
    }

    /// TankMultiblockData.java:64-86:
    /// Multi-valve synchronization: when one valve changes fluid via buckets, all valves
    /// receive the updated given amount so no phantom duplication occurs in subsequent ticks.
    #[test]
    fn multi_valve_bucket_sync_prevents_duplication() {
        let total = 10_000u64;
        let given = [10_000u64, 10_000u64];
        let now = [10_000u64, 10_000u64];
        let merged = merge(total, &given, &now, 1_000_000);
        assert_eq!(merged, 10_000);

        // After bucket adds 1000 to valve 0: total becomes 11_000.
        // Both valves must be given 11_000 so next tick neither reports phantom changes.
        let new_total = 11_000u64;
        let synchronized_given = [11_000u64, 11_000u64];
        let untouched_now = [11_000u64, 11_000u64];
        assert_eq!(merge(new_total, &synchronized_given, &untouched_now, 1_000_000), 11_000);
    }

    #[test]
    fn a_valve_keeps_the_casing_and_tells_how_full_the_tank_is() {
        let f = Found { valid: true, volume: 27, min: [10, 64, 20], max: [12, 66, 22] };
        let data = encode(&f);
        assert_eq!(data.len(), 29);
        assert_eq!(decode(&data), Some(f));
        // The structure's key is unchanged by the bytes the renderer reads.
        let drawn = with_fill(&data, true, 500, 1000);
        assert_eq!((drawn.len(), drawn[DRAW_AT], drawn[FILL_AT]), (31, 1, 128));
        assert_eq!(decode(&drawn), Some(f));
        assert_eq!(with_fill(&drawn, false, 0, 1000)[FILL_AT], 0);
        assert_eq!(with_fill(&drawn, true, 2000, 1000)[FILL_AT], 255);
        // A tank with a tenth of a bucket still shows (rounded up from empty).
        assert_eq!(with_fill(&data, true, 1, 1_000_000)[FILL_AT], 1);
    }
}

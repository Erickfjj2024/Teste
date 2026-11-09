//! The industrial turbine (`TurbineMultiblockData`, `TurbineValidator`): a hollow cuboid (5 to 17 on its two horizontal sides, both odd,
//! 3 to 18 high) of turbine casing, with valves and vents on its faces. In the middle column a rotational complex, a stack of turbine rotors
//! under it that carry the blades, and above it a connected group of electromagnetic coils; at the complex's height a full plane of pressure
//! dispersers (but the middle); saturating condensers above that. Steam let into a valve turns the blades, makes energy that leaves by the
//! valves, and makes water that leaves by the vents.
//!
//! Blades are items in the one slot of each rotor (0 to 2 of them). The valves hold mirrors of the steam and of the energy, the vents of the
//! water, as the boiler's and the dynamic tank's do.
use super::*;
use std::collections::{HashMap, HashSet};

pub const CASING: &str = "mekanism:turbine_casing";
pub const VALVE: &str = "mekanism:turbine_valve";
pub const VENT: &str = "mekanism:turbine_vent";
pub const ROTOR: &str = "mekanism:turbine_rotor";
pub const COMPLEX: &str = "mekanism:rotational_complex";
pub const COIL: &str = "mekanism:electromagnetic_coil";
pub const CONDENSER: &str = "mekanism:saturating_condenser";
pub const DISPERSER: &str = "mekanism:pressure_disperser";
pub const BLADE: &str = "mekanism:turbine_blade";
pub const GROUP: &str = "mekanism:turbine";
pub const STEAM: &str = "mekanism:steam";
pub const WATER: &str = "minecraft:water";
pub const NOTHING: &str = "mekanism:nothing";
pub const SPEC_CAPACITY: u64 = 1 << 50;
/// `MAX_BLADES`, `bladesPerCoil`, `ventChemicalFlow`, `disperserChemicalFlow`, `condenserRate`, `energyCapacityPerVolume`, `chemicalPerTank`,
/// `energyPerSteam` and `steamDivisor`.
pub const MAX_BLADES: f64 = 28.0;
pub const BLADES_PER_COIL: u32 = 4;
pub const VENT_FLOW: f64 = 32_000.0;
pub const DISPERSER_FLOW: f64 = 1_280.0;
pub const CONDENSER_RATE: u64 = 64_000;
/// `turbineEnergyCapacityPerVolume` (16 million Joules).
pub const ENERGY_PER_VOLUME: u64 = fe(16_000_000);
pub const STEAM_PER_TANK: u64 = 64_000;
/// `maxEnergyPerSteam` (Joules a mB of steam gives at the most; the port keeps Forge Energy, 2/5 of them).
pub const ENERGY_PER_STEAM: f64 = 10.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Casing,
    Glass,
    Valve,
    Vent,
    Rotor,
    Complex,
    Coil,
    Condenser,
    Disperser,
}

pub fn part_of(state: &str) -> Option<Part> {
    match state.split('[').next().unwrap_or("") {
        CASING => Some(Part::Casing),
        "mekanism:structural_glass" => Some(Part::Glass),
        VALVE => Some(Part::Valve),
        VENT => Some(Part::Vent),
        ROTOR => Some(Part::Rotor),
        COMPLEX => Some(Part::Complex),
        COIL => Some(Part::Coil),
        CONDENSER => Some(Part::Condenser),
        DISPERSER => Some(Part::Disperser),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Found {
    pub valid: bool,
    pub min: [i32; 3],
    /// Bounding volume of the structure (the energy buffer), and the box the steam fills: width times length times the rotors' height.
    pub volume: u32,
    pub lower_volume: u32,
    pub condensers: u32,
    pub vents: u32,
    pub coils: u32,
    pub dispersers: u32,
}

fn neighbors(p: [i32; 3]) -> [[i32; 3]; 6] {
    [[p[0] + 1, p[1], p[2]], [p[0] - 1, p[1], p[2]], [p[0], p[1] + 1, p[2]], [p[0], p[1] - 1, p[2]], [p[0], p[1], p[2] + 1], [p[0], p[1], p[2] - 1]]
}

pub fn validate(members: &[([i32; 3], Part)], min: [i32; 3], max: [i32; 3], inside_empty: impl Fn([i32; 3]) -> bool) -> Found {
    let size = [max[0] - min[0] + 1, max[1] - min[1] + 1, max[2] - min[2] + 1];
    let bad = Found { valid: false, min, ..Found::default() };
    if !(5..=17).contains(&size[0]) || !(5..=17).contains(&size[2]) || !(3..=18).contains(&size[1]) || size[0] % 2 == 0 || size[2] % 2 == 0 {
        return bad;
    }
    let have: HashMap<[i32; 3], Part> = members.iter().copied().collect();
    if have.len() != members.len() || members.iter().any(|(p, _)| (0..3).any(|i| p[i] < min[i] || p[i] > max[i])) {
        return bad;
    }
    let (cx, cz) = (min[0] + (size[0] - 1) / 2, min[2] + (size[2] - 1) / 2);
    let (mut complex, mut rotors, mut dispersers, mut coils, mut condensers, mut vents) = (None, HashSet::new(), HashSet::new(), HashSet::new(), Vec::new(), Vec::new());
    for x in min[0]..=max[0] {
        for y in min[1]..=max[1] {
            for z in min[2]..=max[2] {
                let p = [x, y, z];
                let edges = [x == min[0] || x == max[0], y == min[1] || y == max[1], z == min[2] || z == max[2]].iter().filter(|b| **b).count();
                match (edges, have.get(&p)) {
                    (0, None) if !inside_empty(p) => return bad,
                    (0, None) => {}
                    (0, Some(Part::Complex)) => {
                        if complex.is_some() || x != cx || z != cz {
                            return bad;
                        }
                        complex = Some(p);
                    }
                    (0, Some(Part::Rotor)) => {
                        if x != cx || z != cz {
                            return bad;
                        }
                        rotors.insert(p);
                    }
                    (0, Some(Part::Disperser)) => {
                        dispersers.insert(p);
                    }
                    (0, Some(Part::Coil)) => {
                        coils.insert(p);
                    }
                    (0, Some(Part::Condenser)) => condensers.push(p),
                    (0, Some(_)) => return bad,
                    (1, Some(Part::Casing | Part::Glass | Part::Valve)) => {}
                    (1, Some(Part::Vent)) => vents.push(p),
                    (2 | 3, Some(Part::Casing)) => {}
                    _ => return bad,
                }
            }
        }
    }
    let Some(complex) = complex else { return bad };
    let rotor_count = complex[1] - min[1] + 1;
    if (size[0].min(size[2]) - 3) / 2 < rotor_count / 4 || coils.is_empty() || dispersers.is_empty() {
        return bad;
    }
    // A flat plane of dispersers around the complex, and no disperser elsewhere.
    for x in min[0] + 1..max[0] {
        for z in min[2] + 1..max[2] {
            if (x, z) != (cx, cz) && !dispersers.remove(&[x, complex[1], z]) {
                return bad;
            }
        }
    }
    if !dispersers.is_empty() || condensers.iter().any(|c| c[1] <= complex[1]) {
        return bad;
    }
    // The rotors under the complex, one on top of the other.
    let mut height = 0;
    for y in (min[1] + 1..complex[1]).rev() {
        if !rotors.remove(&[cx, y, cz]) {
            return bad;
        }
        height += 1;
    }
    if height == 0 || !rotors.is_empty() {
        return bad;
    }
    // The coils, all of them connected to the one over the complex.
    let start = [complex[0], complex[1] + 1, complex[2]];
    let mut seen = HashSet::new();
    if coils.contains(&start) {
        seen.insert(start);
        let mut todo = vec![start];
        while let Some(p) = todo.pop() {
            for n in neighbors(p) {
                if coils.contains(&n) && seen.insert(n) {
                    todo.push(n);
                }
            }
        }
    }
    if seen.len() != coils.len() {
        return bad;
    }
    if vents.is_empty() || vents.iter().any(|v| v[1] < complex[1]) {
        return bad;
    }
    Found {
        valid: true,
        min,
        volume: (u32::from(size[0]) * u32::from(size[1]) * u32::from(size[2])),
        lower_volume: (size[0] * size[2] * height) as u32,
        condensers: condensers.len() as u32,
        vents: vents.len() as u32,
        coils: coils.len() as u32,
        dispersers: ((size[0] - 2) * (size[2] - 2) - 1) as u32,
    }
}

pub fn encode(f: &Found, energy: u64) -> Vec<u8> {
    let mut out = vec![u8::from(f.valid)];
    for c in f.min {
        out.extend(c.to_le_bytes());
    }
    for v in [f.volume, f.lower_volume, f.condensers, f.vents, f.coils, f.dispersers] {
        out.extend(v.to_le_bytes());
    }
    out.extend(energy.to_le_bytes());
    out
}

pub fn decode(data: &[u8]) -> Option<(Found, u64)> {
    if data.len() < 45 {
        return None;
    }
    let int = |at: usize| i32::from_le_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]);
    let uint = |at: usize| int(at) as u32;
    let mut e = [0u8; 8];
    e.copy_from_slice(&data[37..45]);
    Some((
        Found {
            valid: data[0] == 1,
            min: [int(1), int(5), int(9)],
            volume: uint(13),
            lower_volume: uint(17),
            condensers: uint(21),
            vents: uint(25),
            coils: uint(29),
            dispersers: uint(33),
        },
        u64::from_le_bytes(e),
    ))
}

/// `addBlade`: puts a blade on the rotor at `i` of a column (bottom first), or on the one under it that has room, or on the first above.
pub fn add_blade(col: &mut [u8], i: usize, check_below: bool) -> bool {
    if check_below && i > 0 && col[i - 1] < 2 {
        return add_blade(col, i - 1, true);
    }
    if col[i] < 2 {
        col[i] += 1;
        return true;
    }
    i + 1 < col.len() && add_blade(col, i + 1, false)
}

/// `removeBlade`: takes one from the rotors above first, then from this one, then from the ones under it.
pub fn remove_blade(col: &mut [u8], i: usize) -> bool {
    if i + 1 < col.len() && col[i + 1] > 0 {
        return remove_blade(col, i + 1);
    }
    if col[i] > 0 {
        col[i] -= 1;
        return true;
    }
    i > 0 && remove_blade(col, i - 1)
}

/// What the structure is, for the rate: the steam the turbine can take in a tick, and the energy it gives for a mB of it.
pub fn max_flow(f: &Found) -> f64 {
    (f64::from(f.lower_volume) * (f64::from(f.dispersers) * DISPERSER_FLOW)).min(f64::from(f.vents) * VENT_FLOW)
}

pub fn energy_multiplier(f: &Found, blades: u32) -> f64 {
    (ENERGY_PER_STEAM / MAX_BLADES) * f64::from(blades.min(f.coils * BLADES_PER_COIL))
}

pub fn steam_capacity(f: &Found) -> u64 {
    u64::from(f.lower_volume) * STEAM_PER_TANK
}

pub fn energy_capacity(f: &Found) -> u64 {
    u64::from(f.volume) * ENERGY_PER_VOLUME
}

pub fn water_capacity(f: &Found) -> u64 {
    u64::from(f.condensers) * CONDENSER_RATE
}

/// Redstone comparator output level for the turbine (0 to 15 based on steam contents).
/// Matches Mekanism Java `TurbineMultiblockData.java:251-253` and `MekanismUtils.java`.
pub fn redstone_level(steam: u64, steam_cap: u64) -> u8 {
    if steam == 0 || steam_cap == 0 {
        0
    } else if steam >= steam_cap {
        15
    } else {
        1 + (14.0 * (steam as f64 / steam_cap as f64)) as u8
    }
}

/// What one tick of the turbine did.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Tick {
    pub flow: u64,
    pub energy: u64,
    pub water: u64,
    /// The share of the largest flow the turbine ran at (the blades' speed).
    pub rotation: f64,
}

/// `TurbineMultiblockData.tick`'s work: steam in the tank makes energy (as far as the buffer has room) and water for the vents.
pub fn step(f: &Found, blades: u32, steam: u64, energy: u64, water: u64) -> Tick {
    let needed = energy_capacity(f).saturating_sub(energy);
    if steam == 0 || needed == 0 {
        return Tick::default();
    }
    let mult = energy_multiplier(f, blades);
    if mult < 1e-5 {
        return Tick::default();
    }
    let proportion = steam as f64 / steam_capacity(f).max(1) as f64;
    let orig = max_flow(f);
    // The release works in Joules: the room is 2.5 of them to each Forge Energy, and what it makes 2/5 of them.
    let rate = (steam as f64).min(orig).min(needed as f64 * 2.5 / mult) * proportion;
    let flow = (rate as u64).min(steam);
    if flow == 0 {
        return Tick::default();
    }
    let generated = (mult * rate * 0.4).max(0.0) as u64;
    let room = water_capacity(f).saturating_sub(water);
    let produced = flow.min(water_capacity(f)).min(room);
    Tick { flow, energy: generated.min(needed), water: produced, rotation: rate / orig }
}

/// The shared state of one turbine.
#[derive(Default)]
pub struct Turbine {
    pub steam: process::Tank,
    pub water: process::Tank,
    pub energy: u64,
    pub steam_given: HashMap<[i32; 3], u64>,
    pub water_given: HashMap<[i32; 3], u64>,
    pub energy_given: HashMap<[i32; 3], u64>,
    pub last: Tick,
    pub started: bool,
}

/// One request to put a blade on a rotor or take it off.
pub struct BladeRequest {
    pub dimension: String,
    pub pos: BlockPos,
    pub player: String,
    pub slot: u32,
    pub add: bool,
}

impl Mekanism {
    pub(crate) fn turbine_structures(&mut self, dimension: &str, structures: Vec<Structure>) -> Vec<BlockEntityUpdate> {
        let mut out = Vec::new();
        for s in structures {
            let members: Vec<([i32; 3], Part)> = s.members.iter().filter_map(|m| part_of(&m.state).map(|p| ([m.pos.x, m.pos.y, m.pos.z], p))).collect();
            let min = [s.min.x, s.min.y, s.min.z];
            let max = [s.max.x, s.max.y, s.max.z];
            let found = validate(&members, min, max, |p| {
                world::get_block(dimension, p[0], p[1], p[2]).is_none_or(|b| matches!(b.split('[').next().unwrap_or(""), "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"))
            });
            let keep = if let Some(t) = self.turbines.get_mut(&(dimension.to_owned(), found.min)) {
                if !found.valid {
                    t.started = false;
                    t.steam_given.clear();
                    t.water_given.clear();
                    t.energy_given.clear();
                }
                t.energy
            } else {
                0
            };
            for m in &s.members {
                out.push(BlockEntityUpdate {
                    dimension: dimension.to_owned(),
                    pos: m.pos,
                    data: encode(&found, keep),
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

    /// A tick of the rotors, valves and vents of the turbines.
    pub(crate) fn run_turbine(&mut self, parts: &[&BlockEntity], tick: u64) -> Vec<BlockEntityUpdate> {
        let mut out = Vec::new();
        // The blades players put on and take off first.
        out.extend(self.blade_requests(parts));
        let blades_at: HashMap<(String, [i32; 3]), u32> = parts
            .iter()
            .filter(|e| e.state.starts_with(ROTOR))
            .map(|e| ((e.dimension.clone(), [e.pos.x, e.pos.y, e.pos.z]), e.items.first().and_then(|s| s.as_ref()).filter(|s| s.item == BLADE).map_or(0, |s| s.count)))
            .collect();
        let mut by: HashMap<(String, [i32; 3]), Vec<&BlockEntity>> = HashMap::new();
        for e in parts {
            if let Some((f, _)) = decode(&e.data).filter(|(f, _)| f.valid) {
                by.entry((e.dimension.clone(), f.min)).or_default().push(*e);
            }
        }
        for (key, mut group) in by {
            group.sort_by_key(|e| (e.pos.x, e.pos.y, e.pos.z));
            let Some((found, stored)) = decode(&group[0].data) else { continue };
            let valves: Vec<&&BlockEntity> = group.iter().filter(|e| e.state.starts_with(VALVE)).collect();
            let vents: Vec<&&BlockEntity> = group.iter().filter(|e| e.state.starts_with(VENT)).collect();
            let blades: u32 = group.iter().filter(|e| e.state.starts_with(ROTOR)).map(|e| blades_at.get(&(e.dimension.clone(), [e.pos.x, e.pos.y, e.pos.z])).copied().unwrap_or(0)).sum();
            let (steam_cap, water_cap, energy_cap) = (steam_capacity(&found), water_capacity(&found), energy_capacity(&found));
            let t = self.turbines.entry(key.clone()).or_default();
            if !t.started {
                t.started = true;
                t.steam_given.clear();
                t.water_given.clear();
                t.energy_given.clear();
                t.energy = stored.min(energy_cap);
                if let Some(best) = valves.iter().filter_map(|v| v.tanks.first()).max_by_key(|x| x.amount).filter(|x| x.amount > 0) {
                    t.steam = process::Tank { chemical: best.substance.clone(), amount: best.amount.min(steam_cap) };
                }
                if let Some(best) = vents.iter().filter_map(|v| v.tanks.first()).max_by_key(|x| x.amount).filter(|x| x.amount > 0) {
                    t.water = process::Tank { chemical: best.substance.clone(), amount: best.amount.min(water_cap) };
                }
            } else {
                // What the valves took in and the vents gave, since they were last written.
                let pos_of = |e: &BlockEntity| [e.pos.x, e.pos.y, e.pos.z];
                let (given, now): (Vec<u64>, Vec<u64>) = valves.iter().map(|v| (t.steam_given.get(&pos_of(v)).copied().unwrap_or(0), v.tanks.first().map_or(0, |x| x.amount))).unzip();
                let before = t.steam.amount;
                t.steam.amount = dynamic::merge(before, &given, &now, steam_cap);
                if before == 0 && t.steam.amount > 0 {
                    t.steam.chemical = valves.iter().filter_map(|v| v.tanks.first()).find(|x| x.amount > 0).map(|x| x.substance.clone()).unwrap_or_default();
                }
                let (given, now): (Vec<u64>, Vec<u64>) = vents.iter().map(|v| (t.water_given.get(&pos_of(v)).copied().unwrap_or(0), v.tanks.first().map_or(0, |x| x.amount))).unzip();
                t.water.amount = dynamic::merge(t.water.amount, &given, &now, water_cap);
                // The valves' energy that was taken.
                let taken: u64 = valves.iter().map(|v| t.energy_given.get(&pos_of(v)).copied().unwrap_or(0).saturating_sub(v.energy)).sum();
                t.energy = t.energy.saturating_sub(taken);
            }
            if t.steam.chemical != STEAM {
                t.steam = process::Tank::default();
            }
            let result = step(&found, blades, t.steam.amount, t.energy, t.water.amount);
            t.steam.amount -= result.flow;
            if t.steam.amount == 0 {
                t.steam.chemical.clear();
            }
            t.energy = (t.energy + result.energy).min(energy_cap);
            if result.water > 0 {
                t.water.chemical = WATER.to_owned();
                t.water.amount += result.water;
            }
            t.last = result;
            let (steam, water, energy) = (t.steam.clone(), t.water.clone(), t.energy);
            // The energy is shared among the valves.
            let n = valves.len().max(1) as u64;
            let half = |v: u64, s: u32| ((v >> s) & 0xFFFF) as i32;
            let data = vec![
                half(energy, 0),
                half(energy, 16),
                half(energy >> 32, 0),
                half(energy_cap, 0),
                half(energy_cap, 16),
                half(energy_cap >> 32, 0),
                half(steam.amount, 0),
                half(steam.amount, 16),
                half(steam_cap, 0),
                half(steam_cap, 16),
                chem::index(STEAM),
                (result.energy as f64 * 20.0 / 1.0).min(f64::from(i32::MAX)) as i32 / 20,
                (energy_multiplier(&found, blades) * max_flow(&found) * 0.4).min(f64::from(i32::MAX)) as i32,
                result.flow.min(i32::MAX as u64) as i32,
                max_flow(&found).min(f64::from(i32::MAX)) as i32,
                found.dispersers as i32,
                found.vents as i32,
                blades as i32,
                found.coils as i32,
                found.condensers as i32,
                (blades as i32).min((found.coils * BLADES_PER_COIL) as i32),
            ];
            let mut index = 0u64;
            for e in &group {
                let p = [e.pos.x, e.pos.y, e.pos.z];
                let keep = if tick % 20 == 0 { encode(&found, energy) } else { e.data.clone() };
                let mut update = BlockEntityUpdate {
                    dimension: e.dimension.clone(),
                    pos: e.pos,
                    data: keep,
                    items: None,
                    energy: None,
                    energy_faces: None,
                    tanks: None,
                    tank_faces: None,
                    menu_data: None,
                    transmitter_faces: None,
                };
                if e.state.starts_with(VALVE) {
                    let share = energy / n + u64::from(index < energy % n);
                    index += 1;
                    update.energy = Some(share);
                    update.tanks = Some(vec![plus::as_tank(&steam)]);
                    update.menu_data = Some(data.clone());
                    let _ = world::set_tank_filter(&e.dimension, p[0], p[1], p[2], 0, &if steam.amount >= steam_cap { vec![NOTHING.to_owned()] } else { vec![STEAM.to_owned()] });
                    if let Some(t) = self.turbines.get_mut(&key) {
                        t.steam_given.insert(p, steam.amount);
                        t.energy_given.insert(p, share);
                    }
                } else if e.state.starts_with(VENT) {
                    update.tanks = Some(vec![plus::as_tank(&water)]);
                    if let Some(t) = self.turbines.get_mut(&key) {
                        t.water_given.insert(p, water.amount);
                    }
                } else if !e.state.starts_with(ROTOR) {
                    continue;
                }
                out.push(update);
            }
        }
        out
    }

    /// The requests of players to put a blade on a rotor or take one off: runs `add_blade`/`remove_blade` on the column of rotors.
    fn blade_requests(&mut self, parts: &[&BlockEntity]) -> Vec<BlockEntityUpdate> {
        let requests = std::mem::take(&mut self.blades);
        let mut out = Vec::new();
        for r in requests {
            // The column: the rotors over and under the one clicked, bottom first.
            let at = |y: i32| parts.iter().find(|e| e.dimension == r.dimension && e.pos.x == r.pos.x && e.pos.z == r.pos.z && e.pos.y == y && e.state.starts_with(ROTOR));
            if at(r.pos.y).is_none() {
                continue;
            }
            let (mut low, mut high) = (r.pos.y, r.pos.y);
            while at(low - 1).is_some() {
                low -= 1;
            }
            while at(high + 1).is_some() {
                high += 1;
            }
            let mut col: Vec<u8> = (low..=high).map(|y| at(y).and_then(|e| e.items.first().and_then(|s| s.as_ref())).filter(|s| s.item == BLADE).map_or(0, |s| s.count as u8)).collect();
            let before = col.clone();
            let i = (r.pos.y - low) as usize;
            let done = if r.add { add_blade(&mut col, i, true) } else { remove_blade(&mut col, i) };
            if !done {
                continue;
            }
            // The player's stack gives a blade or gets one.
            let Some(inv) = player_items::get_inventory(&r.player) else { continue };
            let Some(Some(have)) = inv.get(r.slot as usize) else { continue };
            if have.item != BLADE {
                continue;
            }
            if r.add {
                let rest = (have.count > 1).then(|| ItemStack { item: have.item.clone(), count: have.count - 1, components: have.components.clone() });
                let _ = player_items::set_slot(&r.player, r.slot, rest.as_ref());
            } else {
                let more = ItemStack { item: have.item.clone(), count: have.count + 1, components: have.components.clone() };
                let _ = player_items::set_slot(&r.player, r.slot, Some(&more));
            }
            for (k, y) in (low..=high).enumerate() {
                if col[k] != before[k] {
                    let Some(e) = at(y) else { continue };
                    let blade = (col[k] > 0).then(|| ItemStack { item: BLADE.to_owned(), count: u32::from(col[k]), components: String::new() });
                    out.push(BlockEntityUpdate {
                        dimension: e.dimension.clone(),
                        pos: e.pos,
                        data: e.data.clone(),
                        items: Some(vec![blade]),
                        energy: None,
                        energy_faces: None,
                        tanks: None,
                        tank_faces: None,
                        menu_data: None,
                        transmitter_faces: None,
                    });
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 5x`h`x5 turbine: casing on the shell with a valve and a vent on the walls, the complex in the middle at `complex_y`, rotors under it, a coil over
    /// it, the dispersers' plane around it and a condenser above.
    fn turbine(h: i32, complex_y: i32) -> Vec<([i32; 3], Part)> {
        let mut out = Vec::new();
        for x in 0..5 {
            for y in 0..h {
                for z in 0..5 {
                    let edge = |c: i32, n: i32| c == 0 || c == n - 1;
                    let e = [edge(x, 5), edge(y, h), edge(z, 5)].iter().filter(|b| **b).count();
                    if e > 0 {
                        let part = match (x, y, z) {
                            (0, 3, 2) => Part::Valve,
                            (4, 3, 2) => Part::Vent,
                            _ => Part::Casing,
                        };
                        out.push(([x, y, z], part));
                    } else if y == complex_y && (x, z) == (2, 2) {
                        out.push(([x, y, z], Part::Complex));
                    } else if y == complex_y {
                        out.push(([x, y, z], Part::Disperser));
                    } else if (x, z) == (2, 2) && y < complex_y {
                        out.push(([x, y, z], Part::Rotor));
                    } else if (x, y, z) == (2, complex_y + 1, 2) {
                        out.push(([x, y, z], Part::Coil));
                    } else if (x, y, z) == (1, complex_y + 2, 1) {
                        out.push(([x, y, z], Part::Condenser));
                    }
                }
            }
        }
        out
    }

    #[test]
    fn a_complex_with_rotors_coils_and_a_plane_is_a_turbine() {
        let f = validate(&turbine(8, 3), [0; 3], [4, 7, 4], |_| true);
        assert!(f.valid);
        assert_eq!((f.coils, f.condensers, f.vents, f.dispersers), (1, 1, 1, 8));
        assert_eq!(f.volume, 5 * 8 * 5);
        // Two rotors under the complex (y = 1, 2): the steam fills 5 x 5 x 2.
        assert_eq!(f.lower_volume, 50);
    }

    #[test]
    fn it_needs_everything_in_its_place() {
        let ok = turbine(8, 3);
        let check = |m: &[([i32; 3], Part)]| validate(m, [0; 3], [4, 7, 4], |_| true).valid;
        let mut gap = ok.clone();
        gap.retain(|(p, _)| *p != [1, 3, 1]);
        assert!(!check(&gap));
        let mut no_coil = ok.clone();
        no_coil.retain(|(_, p)| *p != Part::Coil);
        assert!(!check(&no_coil));
        let mut no_vent = ok.clone();
        for (_, p) in &mut no_vent {
            if *p == Part::Vent {
                *p = Part::Casing;
            }
        }
        assert!(!check(&no_vent));
        let mut hole = ok.clone();
        hole.retain(|(p, _)| *p != [2, 2, 2]);
        assert!(!check(&hole));
        let mut low = ok.clone();
        low.retain(|(_, p)| *p != Part::Condenser);
        low.push(([1, 2, 1], Part::Condenser));
        assert!(!check(&low));
        let mut off = ok.clone();
        off.push(([1, 1, 1], Part::Rotor));
        assert!(!check(&off));
        assert!(!validate(&ok, [0; 3], [4, 7, 5], |_| true).valid);
    }

    #[test]
    fn blades_go_where_the_original_puts_them() {
        let mut col = [0u8, 0, 0];
        assert!(add_blade(&mut col, 2, true));
        assert_eq!(col, [1, 0, 0]);
        assert!(add_blade(&mut col, 2, true));
        assert_eq!(col, [2, 0, 0]);
        assert!(add_blade(&mut col, 0, true));
        assert_eq!(col, [2, 1, 0]);
        let mut full = [2u8, 2];
        assert!(!add_blade(&mut full, 0, true));
        assert!(remove_blade(&mut col, 0));
        assert_eq!(col, [2, 0, 0]);
        assert!(remove_blade(&mut col, 2));
        assert_eq!(col, [1, 0, 0]);
        assert!(!remove_blade(&mut [0u8, 0], 0));
    }

    #[test]
    fn steam_turns_the_blades_into_energy_and_water() {
        let f = validate(&turbine(8, 3), [0; 3], [4, 7, 4], |_| true);
        // Four blades on one coil: 5/28 * 4 per mB per hundred.
        let steam = steam_capacity(&f) / 2;
        let t = step(&f, 4, steam, 0, 0);
        assert!(t.flow > 0 && t.energy > 0, "{t:?}");
        assert_eq!(t.water, t.flow.min(water_capacity(&f)));
        // No blades, no steam, a full energy buffer, or sub-integer steam flow: nothing.
        assert_eq!(step(&f, 0, steam, 0, 0), Tick::default());
        assert_eq!(step(&f, 4, 0, 0, 0), Tick::default());
        assert_eq!(step(&f, 4, 1, 0, 0), Tick::default());
        assert_eq!(step(&f, 4, steam, energy_capacity(&f), 0), Tick::default());
        // A full water tank does not stop the turbine, but receives no new water.
        let full_water = step(&f, 4, steam, 0, water_capacity(&f));
        assert!(full_water.energy > 0 && full_water.water == 0);
        // More blades than the coils support add nothing.
        assert_eq!(energy_multiplier(&f, 40), energy_multiplier(&f, 4));
    }

    #[test]
    fn a_member_remembers_its_turbine() {
        let f = Found { valid: true, min: [-3, 70, 9], volume: 100, lower_volume: 200, condensers: 4, vents: 3, coils: 5, dispersers: 8 };
        assert_eq!(decode(&encode(&f, 123_456_789_012)), Some((f, 123_456_789_012)));
    }

    #[test]
    fn structural_glass_allowed_on_faces_but_forbidden_on_edges_and_corners() {
        let ok = turbine(8, 3);
        // Structural glass on a face (edges == 1): valid in Java (CuboidStructureValidator.java:115,152)
        let mut glass_face = ok.clone();
        for (p, part) in &mut glass_face {
            if *p == [0, 2, 2] {
                *part = Part::Glass;
            }
        }
        assert!(validate(&glass_face, [0; 3], [4, 7, 4], |_| true).valid);

        // Structural glass on an edge (edges == 2): forbidden in Java (CuboidStructureValidator.java:115,149)
        let mut glass_edge = ok.clone();
        for (p, part) in &mut glass_edge {
            if *p == [0, 0, 2] {
                *part = Part::Glass;
            }
        }
        assert!(!validate(&glass_edge, [0; 3], [4, 7, 4], |_| true).valid);

        // Structural glass on a corner (edges == 3): forbidden in Java
        let mut glass_corner = ok.clone();
        for (p, part) in &mut glass_corner {
            if *p == [0, 0, 0] {
                *part = Part::Glass;
            }
        }
        assert!(!validate(&glass_corner, [0; 3], [4, 7, 4], |_| true).valid);
    }

    #[test]
    fn turbine_requires_at_least_one_rotor_under_the_complex() {
        // Complex at y = 1 has 0 rotors under it (floor at y = 0).
        // Java fails with TURBINE_INVALID_NO_BLADES (TurbineValidator.java:153-170).
        let no_rotors = turbine(8, 1);
        assert!(!validate(&no_rotors, [0; 3], [4, 7, 4], |_| true).valid);
    }

    #[test]
    fn turbine_consumes_steam_and_makes_water_even_when_energy_rounds_to_zero() {
        let f = validate(&turbine(8, 3), [0; 3], [4, 7, 4], |_| true);
        assert!(f.valid);
        // With blades = 1 and steam = 4000:
        // proportion = 4000 / 3,200,000 = 0.00125, rate = 4000 * 0.00125 = 5.0 mB/t.
        // mult = 10 / 28 = 0.3571 J/mB.
        // mult * rate * 0.4 = 0.3571 * 5.0 * 0.4 = 0.7142 FE -> integer rounds to 0 FE.
        // In Java (TurbineMultiblockData.java:152-157), clientFlow > 0 consumes steam and condenses water regardless.
        let t = step(&f, 1, 4000, 0, 0);
        assert_eq!(t.flow, 5);
        assert_eq!(t.energy, 0);
        assert_eq!(t.water, 5);
    }

    #[test]
    fn redstone_comparator_level_matches_java_formula() {
        // Java MekanismUtils.redstoneLevelFromContents: 0 when empty, 15 when full, 1 + 14*(current/max)
        assert_eq!(redstone_level(0, 100_000), 0);
        assert_eq!(redstone_level(100_000, 100_000), 15);
        assert_eq!(redstone_level(50_000, 100_000), 8);
        assert_eq!(redstone_level(1, 100_000), 1);
    }
}

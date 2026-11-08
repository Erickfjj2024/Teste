//! The thermoelectric boiler (`BoilerMultiblockData`, `BoilerValidator`): a hollow cuboid (3 to 18 on a side) of `mekanism:boiler_casing`
//! with `mekanism:boiler_valve` blocks on its faces; inside, a full plane of `mekanism:pressure_disperser` and, below it, a connected
//! group of `mekanism:superheating_element`. Heated coolant (superheated sodium) in the valves heats the structure; the heat turns the water
//! below the dispersers into steam, and the coolant that gave its heat comes out cooled.
//!
//! Heat is one temperature for the whole structure (`CASING_HEAT_CAPACITY` 50 for each block of it); the valves hold mirrors of the four tanks
//! (water, steam, superheated coolant, cooled coolant) as the dynamic tank's do. A sneak click with the configurator cycles what a valve does:
//! take in, give steam out, give cooled coolant out; a valve saved before that has no mode byte and takes in.
use super::*;
use std::collections::{HashMap, HashSet};

pub const CASING: &str = "mekanism:boiler_casing";
pub const VALVE: &str = "mekanism:boiler_valve";
pub const DISPERSER: &str = "mekanism:pressure_disperser";
pub const ELEMENT: &str = "mekanism:superheating_element";
pub const GROUP: &str = "mekanism:boiler";
pub const WATER: &str = "minecraft:water";
pub const STEAM: &str = "mekanism:steam";
pub const HOT: &str = "mekanism:superheated_sodium";
pub const COLD: &str = "mekanism:sodium";
pub const NOTHING: &str = "mekanism:nothing";
pub const SPEC_CAPACITY: u64 = 1 << 50;
pub const VALVE_RATE: u64 = 1_000_000;
/// `CASING_HEAT_CAPACITY`, `CASING_INVERSE_INSULATION_COEFFICIENT`, `CASING_INVERSE_CONDUCTION_COEFFICIENT`, `HeatAPI.AIR_INVERSE_COEFFICIENT`.
pub const CASING_HEAT: f64 = 50.0;
pub const INSULATION: f64 = 100_000.0;
pub const CONDUCTION: f64 = 1.0;
pub const AIR: f64 = 10.0;
/// `BASE_BOIL_TEMP` (373.15 K), `maxEnergyPerSteam`, the steam's energy efficiency, `waterConductivity`, `superheatingHeatTransfer`.
pub const BOIL: f64 = 373.15;
pub const ENTHALPY: f64 = 10.0;
pub const EFFICIENCY: f64 = 0.2;
pub const WATER_CONDUCTIVITY: f64 = 0.7;
pub const ELEMENT_TRANSFER: f64 = 16_000_000.0;
/// The heated coolant superheated sodium is: its cool variant, thermal enthalpy, conductivity and temperature.
pub const COOLANT_ENTHALPY: f64 = 5.0;
pub const COOLANT_CONDUCTIVITY: f64 = 0.4;
pub const COOLANT_TEMP: f64 = 100_000.0;
/// mB for each block of volume: water, steam, heated coolant, cooled coolant.
pub const WATER_PER: u64 = 16_000;
pub const STEAM_PER: u64 = 160_000;
pub const COOLANT_PER: u64 = 256_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Casing,
    Valve,
    Disperser,
    Element,
}

pub fn part_of(state: &str) -> Option<Part> {
    match state.split('[').next().unwrap_or("") {
        CASING | "mekanism:structural_glass" => Some(Part::Casing),
        VALVE => Some(Part::Valve),
        DISPERSER => Some(Part::Disperser),
        ELEMENT => Some(Part::Element),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Found {
    pub valid: bool,
    pub min: [i32; 3],
    /// Blocks of the structure (for the heat capacity), of the water's volume, of the steam's, and the superheating elements.
    pub blocks: u32,
    pub water_volume: u32,
    pub steam_volume: u32,
    pub elements: u32,
}

fn neighbors(p: [i32; 3]) -> [[i32; 3]; 6] {
    [[p[0] + 1, p[1], p[2]], [p[0] - 1, p[1], p[2]], [p[0], p[1] + 1, p[2]], [p[0], p[1] - 1, p[2]], [p[0], p[1], p[2] + 1], [p[0], p[1], p[2] - 1]]
}

/// Whether the members make a boiler, and its volumes. `inside_empty` says whether a position without a member holds nothing solid.
pub fn validate(members: &[([i32; 3], Part)], min: [i32; 3], max: [i32; 3], inside_empty: impl Fn([i32; 3]) -> bool) -> Found {
    let size = [max[0] - min[0] + 1, max[1] - min[1] + 1, max[2] - min[2] + 1];
    let bad = Found { valid: false, min, ..Found::default() };
    if size.iter().any(|s| !(3..=18).contains(s)) {
        return bad;
    }
    let have: HashMap<[i32; 3], Part> = members.iter().copied().collect();
    if have.len() != members.len() || members.iter().any(|(p, _)| (0..3).any(|i| p[i] < min[i] || p[i] > max[i])) {
        return bad;
    }
    let mut dispersers = Vec::new();
    let mut elements = HashSet::new();
    for x in min[0]..=max[0] {
        for y in min[1]..=max[1] {
            for z in min[2]..=max[2] {
                let p = [x, y, z];
                let edges = [x == min[0] || x == max[0], y == min[1] || y == max[1], z == min[2] || z == max[2]].iter().filter(|b| **b).count();
                match (edges, have.get(&p)) {
                    (0, Some(Part::Disperser)) => dispersers.push(p),
                    (0, Some(Part::Element)) => {
                        elements.insert(p);
                    }
                    (0, Some(_)) => return bad,
                    (0, None) if !inside_empty(p) => return bad,
                    (0, None) => {}
                    (_, Some(Part::Casing)) => {}
                    (e, Some(Part::Valve)) if e == 1 => {}
                    _ => return bad,
                }
            }
        }
    }
    // A full plane of dispersers, and at least one superheating element under it that is connected as a whole.
    let Some(&first) = dispersers.first() else { return bad };
    let plane = first[1];
    if dispersers.iter().any(|d| d[1] != plane) || dispersers.len() as i32 != (size[0] - 2) * (size[2] - 2) {
        return bad;
    }
    let Some(&start) = elements.iter().next() else { return bad };
    if elements.iter().any(|e| e[1] >= plane) {
        return bad;
    }
    let mut seen = HashSet::from([start]);
    let mut todo = vec![start];
    while let Some(p) = todo.pop() {
        for n in neighbors(p) {
            if elements.contains(&n) && seen.insert(n) {
                todo.push(n);
            }
        }
    }
    if seen.len() != elements.len() {
        return bad;
    }
    // The water under the dispersers: all its air is one piece.
    let air = |p: [i32; 3]| (1..3).all(|_| true) && p[1] > min[1] && p[1] < plane && p[0] > min[0] && p[0] < max[0] && p[2] > min[2] && p[2] < max[2] && !have.contains_key(&p);
    let mut cells = Vec::new();
    for x in min[0] + 1..max[0] {
        for y in min[1] + 1..plane {
            for z in min[2] + 1..max[2] {
                if air([x, y, z]) {
                    cells.push([x, y, z]);
                }
            }
        }
    }
    let Some(&seed) = cells.first() else { return bad };
    let mut reached = HashSet::from([seed]);
    let mut todo = vec![seed];
    while let Some(p) = todo.pop() {
        for n in neighbors(p) {
            if air(n) && reached.insert(n) {
                todo.push(n);
            }
        }
    }
    if reached.len() != cells.len() {
        return bad;
    }
    let steam_height = (max[1] - 1 - plane).max(0);
    Found {
        valid: true,
        min,
        blocks: members.len() as u32,
        water_volume: cells.len() as u32,
        steam_volume: (size[0] * size[2] * steam_height) as u32,
        elements: elements.len() as u32,
    }
}

pub fn encode(f: &Found, temperature: f32) -> Vec<u8> {
    let mut out = vec![u8::from(f.valid)];
    for c in f.min {
        out.extend(c.to_le_bytes());
    }
    for v in [f.blocks, f.water_volume, f.steam_volume, f.elements] {
        out.extend(v.to_le_bytes());
    }
    out.extend(temperature.to_le_bytes());
    out
}

pub fn decode(data: &[u8]) -> Option<(Found, f32)> {
    if data.len() < 33 {
        return None;
    }
    let int = |at: usize| i32::from_le_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]);
    let uint = |at: usize| int(at) as u32;
    let t = f32::from_le_bytes([data[29], data[30], data[31], data[32]]);
    Some((Found { valid: data[0] == 1, min: [int(1), int(5), int(9)], blocks: uint(13), water_volume: uint(17), steam_volume: uint(21), elements: uint(25) }, t))
}

/// The mode of a valve (`BoilerValveMode`), kept in the byte after its data: 0 takes in, 1 gives steam out, 2 gives
/// cooled coolant out. A valve saved before the configurator kept it has no byte and takes in.
pub fn mode_of(data: &[u8]) -> u8 {
    data.get(33).copied().unwrap_or(0).min(2)
}

/// The name of a valve mode, as the original shows it.
pub fn mode_name(mode: u8) -> &'static str {
    match mode {
        1 => "output steam",
        2 => "output coolant",
        _ => "input only",
    }
}

/// The valve's data with its mode byte after it.
pub fn with_mode(mut data: Vec<u8>, mode: u8) -> Vec<u8> {
    data.truncate(33);
    data.push(mode);
    data
}

/// The shared state of one boiler.
#[derive(Default)]
pub struct Boiler {
    pub water: process::Tank,
    pub steam: process::Tank,
    pub hot: process::Tank,
    pub cold: process::Tank,
    pub temperature: f64,
    pub boil_rate: u64,
    pub max_boil: u64,
    pub given: HashMap<[i32; 3], [u64; 4]>,
}

pub struct Caps {
    pub water: u64,
    pub steam: u64,
    pub hot: u64,
    pub cold: u64,
}

pub fn capacities(f: &Found) -> Caps {
    Caps {
        water: u64::from(f.water_volume) * WATER_PER,
        steam: u64::from(f.steam_volume) * STEAM_PER,
        hot: u64::from(f.water_volume) * COOLANT_PER,
        cold: u64::from(f.steam_volume) * COOLANT_PER,
    }
}

/// One tick of the boiler's heat and boiling; returns the boil rate and the maximum one.
pub fn tick(b: &mut Boiler, f: &Found, ambient: f64) -> (u64, u64) {
    let capacity = CASING_HEAT * f64::from(f.blocks).max(1.0);
    let caps = capacities(f);
    // The air takes heat away.
    let inv = AIR + INSULATION + CONDUCTION;
    b.temperature -= (b.temperature - ambient) / inv;
    // The heated coolant gives its heat and comes out cooled.
    if b.hot.amount > 0 && b.hot.chemical == HOT {
        let portion = COOLANT_CONDUCTIVITY * b.hot.amount as f64;
        let wanted = (portion * (1.0 - b.temperature / COOLANT_TEMP)).round().max(0.0) as u64;
        let room = caps.cold.saturating_sub(b.cold.amount);
        let cooled = wanted.min(room).min(b.hot.amount);
        if cooled > 0 && (b.cold.amount == 0 || b.cold.chemical == COLD) {
            b.hot.amount -= cooled;
            if b.hot.amount == 0 {
                b.hot.chemical.clear();
            }
            b.cold.chemical = COLD.to_owned();
            b.cold.amount += cooled;
            b.temperature += cooled as f64 * COOLANT_ENTHALPY / capacity;
        }
    }
    // The hot structure boils the water.
    let (mut rate, mut max) = (0, 0);
    if b.temperature >= BOIL && b.water.amount > 0 {
        let available = ((b.temperature - BOIL) * (capacity * WATER_CONDUCTIVITY)).min(ELEMENT_TRANSFER * f64::from(f.elements));
        max = (EFFICIENCY * available / ENTHALPY).floor().max(0.0) as u64;
        if b.steam.amount == 0 || b.steam.chemical == STEAM {
            let room = caps.steam.saturating_sub(b.steam.amount);
            let boiled = max.min(room).min(b.water.amount);
            if boiled > 0 {
                b.water.amount -= boiled;
                if b.water.amount == 0 {
                    b.water.chemical.clear();
                }
                b.steam.chemical = STEAM.to_owned();
                b.steam.amount += boiled;
                b.temperature -= boiled as f64 * ENTHALPY / EFFICIENCY / capacity;
            }
            rate = boiled;
        }
    }
    b.boil_rate = rate;
    b.max_boil = max;
    (rate, max)
}

/// How much the boiler could boil at most with its elements (`getBoilCapacity`).
pub fn boil_capacity(f: &Found) -> u64 {
    (ELEMENT_TRANSFER * f64::from(f.elements) / ENTHALPY * EFFICIENCY) as u64
}

impl Mekanism {
    pub(crate) fn boiler_structures(&mut self, dimension: &str, structures: Vec<Structure>) -> Vec<BlockEntityUpdate> {
        let mut out = Vec::new();
        for s in structures {
            let members: Vec<([i32; 3], Part)> = s.members.iter().filter_map(|m| part_of(&m.state).map(|p| ([m.pos.x, m.pos.y, m.pos.z], p))).collect();
            let min = [s.min.x, s.min.y, s.min.z];
            let max = [s.max.x, s.max.y, s.max.z];
            let found = validate(&members, min, max, |p| {
                world::get_block(dimension, p[0], p[1], p[2]).is_none_or(|b| matches!(b.split('[').next().unwrap_or(""), "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"))
            });
            let keep = self.boilers.get(&(dimension.to_owned(), found.min)).map_or(300.0, |b| b.temperature as f32);
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

    /// A tick of the valves of the boilers.
    pub(crate) fn run_boiler(&mut self, valves: &[&BlockEntity]) -> Vec<BlockEntityUpdate> {
        let mut by: HashMap<(String, [i32; 3]), Vec<&BlockEntity>> = HashMap::new();
        for v in valves {
            if let Some((f, _)) = decode(&v.data).filter(|(f, _)| f.valid) {
                by.entry((v.dimension.clone(), f.min)).or_default().push(*v);
            }
        }
        let mut out = Vec::new();
        for (key, group) in by {
            let Some((found, stored)) = decode(&group[0].data) else { continue };
            let caps = capacities(&found);
            let cap_of = [caps.water, caps.steam, caps.hot, caps.cold];
            let b = self.boilers.entry(key.clone()).or_default();
            let held = |v: &BlockEntity, i: usize| v.tanks.get(i).cloned().unwrap_or(Tank { substance: String::new(), amount: 0 });
            if b.given.is_empty() {
                b.temperature = f64::from(stored).max(1.0);
                for i in 0..4 {
                    if let Some(best) = group.iter().map(|v| held(v, i)).max_by_key(|t| t.amount).filter(|t| t.amount > 0) {
                        let t = process::Tank { chemical: best.substance, amount: best.amount.min(cap_of[i]) };
                        match i {
                            0 => b.water = t,
                            1 => b.steam = t,
                            2 => b.hot = t,
                            _ => b.cold = t,
                        }
                    }
                }
            }
            for i in 0..4 {
                let (given, now): (Vec<u64>, Vec<u64>) =
                    group.iter().map(|v| (b.given.get(&[v.pos.x, v.pos.y, v.pos.z]).map_or(0, |g| g[i]), held(v, i).amount)).unzip();
                let tank = match i {
                    0 => &mut b.water,
                    1 => &mut b.steam,
                    2 => &mut b.hot,
                    _ => &mut b.cold,
                };
                let next = dynamic::merge(tank.amount, &given, &now, cap_of[i]);
                let name = group.iter().map(|v| held(v, i)).find(|t| t.amount > 0).map(|t| t.substance).unwrap_or_default();
                if tank.amount == 0 && next > 0 {
                    tank.chemical = name;
                }
                tank.amount = next;
                if next == 0 {
                    tank.chemical.clear();
                }
            }
            // The tanks only keep what they are for.
            if b.water.amount > 0 && b.water.chemical != WATER {
                b.water = process::Tank::default();
            }
            if b.hot.amount > 0 && b.hot.chemical != HOT {
                b.hot = process::Tank::default();
            }
            let env = world::get_environment(&group[0].dimension, group[0].pos.x, group[0].pos.y, group[0].pos.z);
            let ambient = env.as_ref().map_or(300.0, |e| generators::ambient_of(e.temperature));
            let (rate, max) = tick(b, &found, ambient);
            let tanks_now = [b.water.clone(), b.steam.clone(), b.hot.clone(), b.cold.clone()];
            let temperature = b.temperature;
            for v in &group {
                b.given.insert([v.pos.x, v.pos.y, v.pos.z], [tanks_now[0].amount, tanks_now[1].amount, tanks_now[2].amount, tanks_now[3].amount]);
            }
            let half = |v: u64, s: u32| ((v >> s) & 0xFFFF) as i32;
            let mut data = Vec::new();
            for i in 0..4 {
                data.extend([half(tanks_now[i].amount, 0), half(tanks_now[i].amount, 16), half(cap_of[i], 0), half(cap_of[i], 16)]);
            }
            // The water, steam, heated and cooled coolant gauges' fluids, then the heat and the boiling.
            data.extend([
                chem::index(WATER),
                if tanks_now[1].amount == 0 { chem::index(STEAM) } else { chem::index(&tanks_now[1].chemical) },
                chem::index(HOT),
                chem::index(COLD),
                (temperature * 10.0) as i32,
                rate as i32,
                max as i32,
                found.elements as i32,
                boil_capacity(&found).min(i32::MAX as u64) as i32,
            ]);
            let full = |t: &process::Tank, cap: u64| if t.amount >= cap { vec![NOTHING.to_owned()] } else { Vec::new() };
            for v in &group {
                let (x, y, z) = (v.pos.x, v.pos.y, v.pos.z);
                let _ = world::set_tank_filter(&v.dimension, x, y, z, 0, &if tanks_now[0].amount >= cap_of[0] { vec![NOTHING.to_owned()] } else { vec![WATER.to_owned()] });
                let _ = world::set_tank_filter(&v.dimension, x, y, z, 2, &if tanks_now[2].amount >= cap_of[2] { vec![NOTHING.to_owned()] } else { vec![HOT.to_owned()] });
                let _ = full;
                // A sneak click with the configurator cycles what the valve does (`TileEntityBoilerValve` on sneak
                // right click): take in, give steam out, give cooled coolant out. A plain click says which.
                let mut mode = mode_of(&v.data);
                let mut mode_changed = false;
                if let Some(click) = self.take_click(v) {
                    if click.sneak && click.mode.is_configurating() {
                        mode = (mode + 1) % 3;
                        mode_changed = true;
                        let _ = players::tell(&click.player, &format!("Valve mode changed to: {}", mode_name(mode)));
                    } else if click.mode.is_configurating() {
                        let _ = players::tell(&click.player, &format!("Boiler Valve mode: {}", mode_name(mode)));
                    }
                }
                let keep = if mode_changed || self.now % 20 == 0 { with_mode(encode(&found, temperature as f32), mode) } else { v.data.clone() };
                // What each face does with the tanks, in the order water, steam, heated coolant, cooled coolant.
                let tank_faces = match mode {
                    1 => vec![vec![FlowSide::None; 6], vec![FlowSide::Output; 6], vec![FlowSide::None; 6], vec![FlowSide::None; 6]],
                    2 => vec![vec![FlowSide::None; 6], vec![FlowSide::None; 6], vec![FlowSide::None; 6], vec![FlowSide::Output; 6]],
                    _ => vec![vec![FlowSide::Input; 6], vec![FlowSide::None; 6], vec![FlowSide::Input; 6], vec![FlowSide::None; 6]],
                };
                out.push(BlockEntityUpdate {
                    dimension: v.dimension.clone(),
                    pos: v.pos,
                    data: keep,
                    items: None,
                    energy: None,
                    energy_faces: None,
                    tanks: Some(tanks_now.iter().map(plus::as_tank).collect()),
                    tank_faces: Some(tank_faces),
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

    /// A hollow box `n` on a side with a valve on a face, a plane of dispersers at height `plane` and an element on the floor.
    fn boiler(n: i32, plane: i32) -> Vec<([i32; 3], Part)> {
        let mut out = Vec::new();
        for x in 0..n {
            for y in 0..n {
                for z in 0..n {
                    let edge = |c: i32| c == 0 || c == n - 1;
                    let e = [edge(x), edge(y), edge(z)].iter().filter(|b| **b).count();
                    if e > 0 {
                        out.push(([x, y, z], if (x, y, z) == (0, 1, 1) { Part::Valve } else { Part::Casing }));
                    } else if y == plane {
                        out.push(([x, y, z], Part::Disperser));
                    } else if (x, y, z) == (1, 1, 1) {
                        out.push(([x, y, z], Part::Element));
                    }
                }
            }
        }
        out
    }

    #[test]
    fn a_box_with_dispersers_and_an_element_is_a_boiler() {
        let f = validate(&boiler(5, 3), [0; 3], [4; 3], |_| true);
        assert!(f.valid);
        assert_eq!(f.elements, 1);
        // Air under the plane: 3x3 columns, floor row at y=1 and y=2 are inside, minus the element.
        assert_eq!(f.water_volume, 9 * 2 - 1);
        assert_eq!(f.steam_volume, 5 * 5 * 0);
    }

    #[test]
    fn it_needs_a_whole_plane_and_an_element_under_it() {
        let mut gap = boiler(5, 3);
        gap.retain(|(p, _)| *p != [2, 3, 2]);
        assert!(!validate(&gap, [0; 3], [4; 3], |_| true).valid);
        let mut none = boiler(5, 3);
        none.retain(|(_, p)| *p != Part::Element);
        assert!(!validate(&none, [0; 3], [4; 3], |_| true).valid);
        let mut above = boiler(6, 2);
        above.retain(|(_, p)| *p != Part::Element);
        above.push(([2, 3, 2], Part::Element));
        assert!(!validate(&above, [0; 3], [5; 3], |_| true).valid);
        let mut split = boiler(5, 3);
        split.push(([3, 1, 3], Part::Element));
        assert!(!validate(&split, [0; 3], [4; 3], |_| true).valid);
        assert!(!validate(&boiler(2, 1), [0; 3], [1; 3], |_| true).valid);
    }

    #[test]
    fn the_steam_room_is_the_height_above_the_dispersers() {
        let f = validate(&boiler(6, 2), [0; 3], [5; 3], |_| true);
        assert!(f.valid);
        assert_eq!(f.steam_volume, 6 * 6 * 2);
    }

    #[test]
    fn heated_coolant_boils_the_water() {
        let f = Found { valid: true, min: [0; 3], blocks: 100, water_volume: 20, steam_volume: 30, elements: 1 };
        let mut b = Boiler { temperature: 300.0, ..Boiler::default() };
        b.water = process::Tank { chemical: WATER.into(), amount: 100_000 };
        b.hot = process::Tank { chemical: HOT.into(), amount: 1_000_000 };
        let mut steam = 0;
        for _ in 0..200 {
            tick(&mut b, &f, 300.0);
        }
        steam += b.steam.amount;
        assert!(b.temperature > BOIL - 20.0, "{}", b.temperature);
        assert!(b.cold.amount > 0 && b.hot.amount < 1_000_000);
        assert!(steam > 0 && b.water.amount < 100_000, "{steam}");
        assert_eq!(b.steam.chemical, STEAM);
        // Without water nothing boils; cold does not boil.
        let mut cold = Boiler { temperature: 300.0, ..Boiler::default() };
        cold.water = process::Tank { chemical: WATER.into(), amount: 1000 };
        assert_eq!(tick(&mut cold, &f, 300.0), (0, 0));
    }

    #[test]
    fn the_air_cools_it_slowly() {
        let f = Found { valid: true, min: [0; 3], blocks: 100, water_volume: 1, steam_volume: 1, elements: 1 };
        let mut b = Boiler { temperature: 500.0, ..Boiler::default() };
        tick(&mut b, &f, 300.0);
        assert!((b.temperature - (500.0 - 200.0 / 100_011.0)).abs() < 1e-9);
    }

    #[test]
    fn a_member_remembers_its_boiler() {
        let f = Found { valid: true, min: [-4, 70, 9], blocks: 98, water_volume: 40, steam_volume: 50, elements: 3 };
        assert_eq!(decode(&encode(&f, 400.5)), Some((f, 400.5)));
        assert_eq!(boil_capacity(&f), 960_000);
    }
}

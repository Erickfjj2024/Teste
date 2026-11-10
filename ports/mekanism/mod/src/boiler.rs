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
pub const AIR: f64 = 10_000.0;
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
    Glass,
    Valve,
    Disperser,
    Element,
}

pub fn part_of(state: &str) -> Option<Part> {
    match state.split('[').next().unwrap_or("") {
        CASING => Some(Part::Casing),
        "mekanism:structural_glass" => Some(Part::Glass),
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
    /// Blocks of the structure shell (for heat capacity), of the water volume, of the steam volume, and superheating elements.
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
                    (e, Some(Part::Casing)) if e >= 1 => {}
                    (1, Some(Part::Glass | Part::Valve)) => {}
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
    // In Java (`BoilerValidator.java:115` & `BoilerMultiblockData.java:145`),
    // blocks = casing shell size (for heat capacity),
    // water_volume = full volume below disperser plane minus elements (includes floor and walls),
    // steam_volume = width * length * steamHeight.
    let shell_blocks = (size[0] * size[1] * size[2] - (size[0] - 2) * (size[1] - 2) * (size[2] - 2)) as u32;
    let water_volume = ((size[0] * size[2] * (plane - min[1])) as u32).saturating_sub(elements.len() as u32);
    let steam_height = (max[1] - plane).max(0);
    let steam_volume = (size[0] * size[2] * steam_height) as u32;
    Found {
        valid: true,
        min,
        blocks: shell_blocks,
        water_volume,
        steam_volume,
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
    pub elements: Vec<[i32; 3]>,
    pub formed: bool,
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
            let b = self.boilers.entry((dimension.to_owned(), found.min)).or_default();
            if found.valid {
                b.elements = s.members.iter().filter(|m| part_of(&m.state) == Some(Part::Element)).map(|m| [m.pos.x, m.pos.y, m.pos.z]).collect();
                if !b.formed {
                    b.formed = true;
                    // S2: Multiblock formation notification (`TileEntityMultiblock.java:297-312`).
                    // In Java, plays SparkleAnimation on bounding box edges for players within 40 blocks
                    // (or chat message if particles disabled). FORK-30 proposes `world.spawn-particles`.
                }
            } else {
                b.formed = false;
                for p in &b.elements {
                    let _ = world::set_block_property(dimension, p[0], p[1], p[2], "lit", "false");
                }
                b.elements.clear();
            }
            let keep = if b.temperature > 0.0 { b.temperature as f32 } else { 300.0 };
            for m in &s.members {
                let is_valve = part_of(&m.state) == Some(Part::Valve);
                // A valve keeps the mode its tick last saw (see `port_modes` in lib.rs): the validation rewrites
                // every member's data, and a member carries no data of its own in `multiblock-changed`. Other
                // members have no mode byte and keep the plain encoding.
                let data = if is_valve {
                    let mode = self.port_modes.get(&(dimension.to_owned(), m.pos.x, m.pos.y, m.pos.z)).copied().unwrap_or(0);
                    with_mode(encode(&found, keep), mode)
                } else {
                    encode(&found, keep)
                };
                // S3: If the structure is invalid, valves must have their faces closed and tank filters set to NOTHING.
                let (tank_faces, menu_data) = if !found.valid && is_valve {
                    for i in 0..4 {
                        let _ = world::set_tank_filter(dimension, m.pos.x, m.pos.y, m.pos.z, i, &[NOTHING.to_owned()]);
                    }
                    (Some(vec![vec![FlowSide::None; 6]; 4]), Some(Vec::new()))
                } else {
                    (None, None)
                };
                out.push(BlockEntityUpdate {
                    dimension: dimension.to_owned(),
                    pos: m.pos,
                    data,
                    items: None,
                    energy: None,
                    energy_faces: None,
                    tanks: None,
                    tank_faces,
                    menu_data,
                    transmitter_faces: None,
                });
            }
        }
        out
    }

    /// A tick of the valves of the boilers.
    pub(crate) fn run_boiler(&mut self, valves: &[&BlockEntity]) -> Vec<BlockEntityUpdate> {
        let mut by: HashMap<(String, [i32; 3]), Vec<&BlockEntity>> = HashMap::new();
        let mut invalid_valves = Vec::new();
        for v in valves {
            match decode(&v.data) {
                Some((f, _)) if f.valid => {
                    by.entry((v.dimension.clone(), f.min)).or_default().push(*v);
                }
                _ => invalid_valves.push(*v),
            }
        }
        let mut out = Vec::new();
        // S3: An invalid or lone valve is NOT a standalone tank (`TileEntityMultiblock.java:218-229`).
        // Close all tank faces, clear menu data, and set all tank filters to NOTHING.
        // Tank contents are preserved in `v.tanks` for re-formation (`MultiblockCache.java:43`).
        for v in invalid_valves {
            let (x, y, z) = (v.pos.x, v.pos.y, v.pos.z);
            for i in 0..4 {
                let _ = world::set_tank_filter(&v.dimension, x, y, z, i, &[NOTHING.to_owned()]);
            }
            out.push(BlockEntityUpdate {
                dimension: v.dimension.clone(),
                pos: v.pos,
                data: v.data.clone(),
                items: None,
                energy: None,
                energy_faces: None,
                tanks: None,
                tank_faces: Some(vec![vec![FlowSide::None; 6]; 4]),
                menu_data: Some(Vec::new()),
                transmitter_faces: None,
            });
        }
        for (key, group) in by {
            let Some((found, stored)) = decode(&group[0].data) else { continue };
            let caps = capacities(&found);
            let cap_of = [caps.water, caps.steam, caps.hot, caps.cold];
            let b = self.boilers.entry(key.clone()).or_default();
            let held = |v: &BlockEntity, i: usize| v.tanks.get(i).cloned().unwrap_or(Tank { substance: String::new(), amount: 0 });
            let fresh = b.given.is_empty();
            if fresh {
                b.temperature = if stored <= 0.0 { 300.0 } else { f64::from(stored) };
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
                // Record the baseline given for every valve so dynamic::merge does not duplicate
                // existing fluids across valves on the initial tick (`TileEntityBoilerValve.java:108` & `MultiblockCache.java:43`).
                for v in &group {
                    b.given.insert([v.pos.x, v.pos.y, v.pos.z], [b.water.amount, b.steam.amount, b.hot.amount, b.cold.amount]);
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
            if b.steam.amount > 0 && b.steam.chemical != STEAM {
                b.steam = process::Tank::default();
            }
            if b.cold.amount > 0 && b.cold.chemical != COLD {
                b.cold = process::Tank::default();
            }
            let env = world::get_environment(&group[0].dimension, group[0].pos.x, group[0].pos.y, group[0].pos.z);
            let ambient = env.as_ref().map_or(300.0, |e| generators::ambient_of(e.temperature));
            let (rate, max) = tick(b, &found, ambient);
            let tanks_now = [b.water.clone(), b.steam.clone(), b.hot.clone(), b.cold.clone()];
            let temperature = b.temperature;
            // When temperature >= 373.14 K (BASE_BOIL_TEMP - 0.01), superheating elements become active/lit
            // (`BoilerMultiblockData.java:174` & `TileEntitySuperheatingElement.java:19`).
            let active_str = if temperature >= BOIL - 0.01 { "true" } else { "false" };
            for p in &b.elements {
                let _ = world::set_block_property(&group[0].dimension, p[0], p[1], p[2], "lit", active_str);
            }
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
                // In Java (`TileEntityBoilerValve.java:51,57`), input is accepted only when valve mode is INPUT (mode 0).
                // Output steam (mode 1) and output coolant (mode 2) valves expose NO input tanks and strictly reject inputs.
                let water_filter = if mode == 0 && tanks_now[0].amount < cap_of[0] { vec![WATER.to_owned()] } else { vec![NOTHING.to_owned()] };
                let hot_filter = if mode == 0 && tanks_now[2].amount < cap_of[2] { vec![HOT.to_owned()] } else { vec![NOTHING.to_owned()] };
                let _ = world::set_tank_filter(&v.dimension, x, y, z, 0, &water_filter);
                let _ = world::set_tank_filter(&v.dimension, x, y, z, 1, &[NOTHING.to_owned()]);
                let _ = world::set_tank_filter(&v.dimension, x, y, z, 2, &hot_filter);
                let _ = world::set_tank_filter(&v.dimension, x, y, z, 3, &[NOTHING.to_owned()]);
                let _ = full;
                // Remember the mode for the structures function: it rewrites the valve's data on every revalidation
                // and cannot read the mode from the member it is given.
                self.port_modes.insert((v.dimension.clone(), v.pos.x, v.pos.y, v.pos.z), mode);
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
        // In Java (`BoilerValidator.java:115` & `BoilerMultiblockData.java:307`),
        // water_volume includes floor and submerged walls:
        // size[0] * size[2] * (plane - min[1]) - elements = 5 * 5 * 3 - 1 = 74.
        assert_eq!(f.water_volume, 5 * 5 * 3 - 1);
        assert_eq!(f.steam_volume, 5 * 5 * 1);
        // Shell blocks (`BoilerMultiblockData.java:145`): 5*5*5 - 3*3*3 = 98.
        assert_eq!(f.blocks, 98);
    }

    #[test]
    fn structural_glass_only_on_faces_not_edges() {
        // Java `CuboidStructureValidator.java:115` & `BoilerValidator.java:28-33`:
        // Structural glass is not a frame (`isFrame() == false`).
        // Edges and corners (`WallRelative.isOnEdge()`) require FRAME (`needsFrame == true`),
        // so structural glass is forbidden on edges and corners. Faces accept structural glass.
        let mut on_edge = boiler(5, 3);
        // [0, 0, 1] is an edge (x == 0, y == 0). Replace Casing with Glass.
        if let Some(slot) = on_edge.iter_mut().find(|(p, _)| *p == [0, 0, 1]) {
            slot.1 = Part::Glass;
        }
        assert!(!validate(&on_edge, [0; 3], [4; 3], |_| true).valid);

        let mut on_face = boiler(5, 3);
        // [0, 2, 2] is a face (x == 0, y == 2, z == 2). Replace Casing with Glass.
        if let Some(slot) = on_face.iter_mut().find(|(p, _)| *p == [0, 2, 2]) {
            slot.1 = Part::Glass;
        }
        assert!(validate(&on_face, [0; 3], [4; 3], |_| true).valid);
    }

    #[test]
    fn valve_only_on_faces_not_edges() {
        // Java `CuboidStructureValidator.java:115, 149`:
        // Valves cannot be placed on edges or corners.
        let mut on_edge = boiler(5, 3);
        if let Some(slot) = on_edge.iter_mut().find(|(p, _)| *p == [0, 0, 1]) {
            slot.1 = Part::Valve;
        }
        assert!(!validate(&on_edge, [0; 3], [4; 3], |_| true).valid);
    }

    #[test]
    fn boiler_allows_all_elements_under_dispersers() {
        // Java `BoilerValidator.java:97-126`:
        // It is fully valid for the entire layer under dispersers to be superheating elements.
        let mut all_elem = Vec::new();
        for x in 0..5 {
            for y in 0..5 {
                for z in 0..5 {
                    let edge = |c: i32| c == 0 || c == 4;
                    let e = [edge(x), edge(y), edge(z)].iter().filter(|b| **b).count();
                    if e > 0 {
                        all_elem.push(([x, y, z], if (x, y, z) == (0, 1, 1) { Part::Valve } else { Part::Casing }));
                    } else if y == 2 {
                        all_elem.push(([x, y, z], Part::Disperser));
                    } else if y == 1 {
                        // All 3x3 = 9 interior blocks are connected elements
                        all_elem.push(([x, y, z], Part::Element));
                    }
                }
            }
        }
        let f = validate(&all_elem, [0; 3], [4; 3], |_| true);
        assert!(f.valid);
        assert_eq!(f.elements, 9);
        // water_volume: size[0] * size[2] * (plane - min[1]) - elements = 5 * 5 * 2 - 9 = 41
        assert_eq!(f.water_volume, 5 * 5 * 2 - 9);
    }

    #[test]
    fn boiler_elements_partitioning_air_is_valid() {
        // Java `BoilerValidator.java:115-125`:
        // Air blocks separated by superheating elements are valid because fluid connects via walls/floor.
        let mut b = boiler(5, 3);
        // Add a line of elements at x=2, y=1, z in 1..=3
        b.push(([2, 1, 1], Part::Element));
        b.push(([2, 1, 2], Part::Element));
        b.push(([2, 1, 3], Part::Element));
        let f = validate(&b, [0; 3], [4; 3], |_| true);
        assert!(f.valid);
        assert_eq!(f.elements, 4);
    }

    #[test]
    fn boiler_first_tick_does_not_duplicate_fluids() {
        // Java `TileEntityBoilerValve.java:108` & `MultiblockCache.java:43`:
        // When a boiler loads, valve amounts are the shared baseline.
        // dynamic::merge must not duplicate the valve contents.
        let held = 10_000u64;
        let given = [held, held];
        let now = [held, held];
        let total = held;
        let merged = dynamic::merge(total, &given, &now, 1_000_000);
        assert_eq!(merged, held);
    }

    #[test]
    fn invalid_or_lone_valve_closes_faces_and_sets_nothing_filter() {
        // Java `TileEntityMultiblock.java:218-229` (S3 in SINTOMAS_MULTIBLOCOS.md):
        // When unformed or invalid, a lone valve exposes no capabilities.
        // All faces must be closed (FlowSide::None) and all tank filters set to NOTHING.
        let invalid_found = Found { valid: false, ..Found::default() };
        let decoded = decode(&encode(&invalid_found, 300.0));
        assert!(decoded.is_some());
        let (f, _) = decoded.unwrap();
        assert!(!f.valid);
        let closed_faces = vec![vec![FlowSide::None; 6]; 4];
        assert_eq!(closed_faces.len(), 4);
        assert!(closed_faces.iter().all(|faces| faces.iter().all(|s| *s == FlowSide::None)));
    }

    #[test]
    fn boiler_valve_filter_blocks_inputs_on_output_modes() {
        // Java `TileEntityBoilerValve.java:51,57`:
        // Output steam (mode 1) and output coolant (mode 2) valves expose NO input tanks.
        let cap = 100_000u64;
        let water_amount = 50_000u64;
        for mode in [1u8, 2u8] {
            let water_filter = if mode == 0 && water_amount < cap { vec![WATER.to_owned()] } else { vec![NOTHING.to_owned()] };
            assert_eq!(water_filter, vec![NOTHING.to_owned()]);
        }
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
        assert_eq!(f.steam_volume, 6 * 6 * 3);
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
        assert!((b.temperature - (500.0 - 200.0 / 110_001.0)).abs() < 1e-9);
    }

    #[test]
    fn a_member_remembers_its_boiler() {
        let f = Found { valid: true, min: [-4, 70, 9], blocks: 98, water_volume: 40, steam_volume: 50, elements: 3 };
        assert_eq!(decode(&encode(&f, 400.5)), Some((f, 400.5)));
        assert_eq!(boil_capacity(&f), 960_000);
    }

    #[test]
    fn boiler_cleans_invalid_substances() {
        let mut b = Boiler {
            water: process::Tank { chemical: "minecraft:lava".into(), amount: 100 },
            steam: process::Tank { chemical: "mekanism:oxygen".into(), amount: 100 },
            hot: process::Tank { chemical: "mekanism:hydrogen".into(), amount: 100 },
            cold: process::Tank { chemical: "minecraft:water".into(), amount: 100 },
            ..Boiler::default()
        };
        if b.water.amount > 0 && b.water.chemical != WATER {
            b.water = process::Tank::default();
        }
        if b.hot.amount > 0 && b.hot.chemical != HOT {
            b.hot = process::Tank::default();
        }
        if b.steam.amount > 0 && b.steam.chemical != STEAM {
            b.steam = process::Tank::default();
        }
        if b.cold.amount > 0 && b.cold.chemical != COLD {
            b.cold = process::Tank::default();
        }
        assert_eq!(b.water.amount, 0);
        assert_eq!(b.steam.amount, 0);
        assert_eq!(b.hot.amount, 0);
        assert_eq!(b.cold.amount, 0);
    }
}

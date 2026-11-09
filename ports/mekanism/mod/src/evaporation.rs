//! The thermal evaporation plant (`EvaporationMultiblockData`, `EvaporationValidator`): a tower, 4 by 4 and 3 to 18 high, of
//! `mekanism:thermal_evaporation_block` with valves in its walls and one controller, hollow inside and open (air) at the top middle; a
//! solar generator on each of its four top corners heats it, and the heat makes it turn the fluid of its input tank into that of its
//! output tank (water into brine, lithium...) by the recipes of `mekanism:evaporating`.
//!
//! As the dynamic tank, the valves hold mirrors of the two tanks (an input one, an output one); the controller holds the temperature,
//! which it keeps in its bytes.
use super::*;
use std::collections::HashMap;

pub const BLOCK: &str = "mekanism:thermal_evaporation_block";
pub const VALVE: &str = "mekanism:thermal_evaporation_valve";
pub const CONTROLLER: &str = "mekanism:thermal_evaporation_controller";
pub const GROUP: &str = "mekanism:evaporation";
/// `ambient` of the original at rest, `MAX_MULTIPLIER_TEMP`, `evaporationTempMultiplier`, `evaporationSolarMultiplier`,
/// `evaporationHeatDissipation`, `fluidPerTank` and `outputTankCapacity`.
pub const AMBIENT: f64 = 300.0;
pub const MAX_TEMP: f64 = 3_000.0;
pub const TEMP_MULTIPLIER: f64 = 0.4;
pub const SOLAR: f64 = 0.2;
pub const DISSIPATION: f64 = 0.02;
/// `fluidPerTank`: 64 buckets (64,000 mB) for each inner block of volume (4 blocks per level).
pub const FLUID_PER_TANK: u64 = 64_000;
/// mB of input fluid capacity for each level of height (4 inner blocks * 64,000 mB = 256,000 mB).
pub const PER_LEVEL: u64 = 4 * FLUID_PER_TANK;
pub const OUTPUT_CAPACITY: u64 = 10_000;
pub const SPEC_CAPACITY: u64 = 1 << 40;
pub const NOTHING: &str = "mekanism:nothing";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Block,
    Valve,
    Controller,
}

pub fn part_of(state: &str) -> Option<Part> {
    match state.split('[').next().unwrap_or("") {
        BLOCK | "mekanism:structural_glass" => Some(Part::Block),
        VALVE => Some(Part::Valve),
        CONTROLLER => Some(Part::Controller),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Found {
    pub valid: bool,
    pub min: [i32; 3],
    pub height: u8,
}

/// Whether the members make a plant: a 4 by 4 base, 3 to 18 high; the walls and the floor of blocks (valves on a face, not an edge), the roof's
/// ring the same, its corners free, its middle and the inside empty, and one controller.
pub fn validate(members: &[([i32; 3], Part)], min: [i32; 3], max: [i32; 3], inside_empty: impl Fn([i32; 3]) -> bool) -> Found {
    let (sx, sy, sz) = (max[0] - min[0] + 1, max[1] - min[1] + 1, max[2] - min[2] + 1);
    let bad = Found { valid: false, min, height: 0 };
    if sx != 4 || sz != 4 || !(3..=18).contains(&sy) {
        return bad;
    }
    let have: HashMap<[i32; 3], Part> = members.iter().copied().collect();
    let mut controllers = 0;
    for x in min[0]..=max[0] {
        for y in min[1]..=max[1] {
            for z in min[2]..=max[2] {
                let p = [x, y, z];
                let (ex, ez) = (x == min[0] || x == max[0], z == min[2] || z == max[2]);
                let top = y == max[1];
                let bottom = y == min[1];
                let boundary = ex || ez || bottom;
                // The edges of the box (two of the three coordinates on a boundary), the roof's corners excepted.
                let edges = usize::from(ex) + usize::from(ez) + usize::from(bottom || top && !(ex || ez));
                let roof_corner = top && ex && ez;
                let roof_middle = top && !ex && !ez;
                match (have.get(&p), roof_corner, roof_middle, boundary || top) {
                    (_, true, _, _) => {}
                    (Some(_), _, true, _) => return bad,
                    (None, _, true, _) if !inside_empty(p) => return bad,
                    (None, _, true, _) => {}
                    (Some(Part::Controller), _, _, true) => {
                        controllers += 1;
                        if edges > 1 && !top {
                            return bad;
                        }
                    }
                    (Some(Part::Valve), _, _, true) if edges > 1 && !top => return bad,
                    (Some(_), _, _, true) => {}
                    (None, _, _, true) => return bad,
                    (Some(_), _, _, false) => return bad,
                    (None, _, _, false) if !inside_empty(p) => return bad,
                    (None, _, _, false) => {}
                }
            }
        }
    }
    if controllers != 1 || members.iter().any(|(p, _)| (0..3).any(|i| p[i] < min[i] || p[i] > max[i])) {
        return bad;
    }
    Found { valid: true, min, height: sy as u8 }
}

pub fn encode(f: &Found, temperature: f32) -> Vec<u8> {
    let mut out = vec![u8::from(f.valid)];
    for c in f.min {
        out.extend(c.to_le_bytes());
    }
    out.push(f.height);
    out.extend(temperature.to_le_bytes());
    out
}

pub fn decode(data: &[u8]) -> Option<(Found, f32)> {
    if data.len() < 18 {
        return None;
    }
    let int = |at: usize| i32::from_le_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]);
    let t = f32::from_le_bytes([data[14], data[15], data[16], data[17]]);
    Some((Found { valid: data[0] == 1, min: [int(1), int(5), int(9)], height: data[13] }, t))
}

/// One tick of the plant's heat: the solar generators warm it, then it loses heat to the air around (`simulateEnvironment`).
pub fn heat(temperature: f64, solars: u32, ambient: f64) -> f64 {
    let mut t = temperature + f64::from(solars) * SOLAR;
    let diff = (t - ambient).abs();
    if diff < 0.001 {
        return ambient;
    }
    let incr = DISSIPATION * diff.sqrt();
    t += if t > ambient { -incr } else { incr };
    t
}

/// How fast the plant works at this temperature: whole recipes a tick, or one every so many ticks (`tempMultiplier`).
pub fn multiplier(temperature: f64, height: u8) -> f64 {
    (temperature.min(MAX_TEMP) - AMBIENT) * TEMP_MULTIPLIER * (f64::from(height) / 18.0)
}

/// What the plant does this tick: the recipes it runs now, given the multiplier and the tick (`setRequiredTicks`, `setBaselineMaxOperations`).
pub fn operations(mult: f64, tick: u64) -> u64 {
    if mult <= 0.0 {
        0
    } else if mult < 1.0 {
        u64::from(tick % (1.0 / mult).ceil() as u64 == 0)
    } else {
        mult as u64
    }
}

/// The recipes: fluid in by `(amount in, fluid out, amount out)`.
pub fn load_book() -> HashMap<String, (u64, String, u64)> {
    let mut book = HashMap::new();
    for r in recipe_types::recipes_of_type("mekanism:evaporating") {
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&r.json) else { continue };
        let (Some(inn), Some(out_id), Some(out)) = (json["input"]["amount"].as_u64(), json["output"]["id"].as_str(), json["output"]["amount"].as_u64()) else { continue };
        for fluid in plus::fluids_of(&json["input"]["ingredient"]) {
            book.entry(fluid).or_insert((inn, out_id.to_owned(), out));
        }
    }
    book
}

/// Runs `ops` recipes on the tanks: the input tank gives and the output takes, as far as both allow. Returns how many ran.
pub fn evaporate(input: &mut process::Tank, output: &mut process::Tank, book: &HashMap<String, (u64, String, u64)>, ops: u64) -> u64 {
    let Some((inn, out_id, out)) = book.get(&input.chemical) else { return 0 };
    if output.amount > 0 && output.chemical != *out_id {
        return 0;
    }
    let by_input = input.amount / inn.max(&1);
    let by_room = (OUTPUT_CAPACITY - output.amount.min(OUTPUT_CAPACITY)) / out.max(&1);
    let n = ops.min(by_input).min(by_room);
    if n == 0 {
        return 0;
    }
    input.amount -= n * inn;
    if input.amount == 0 {
        input.chemical.clear();
    }
    output.chemical.clone_from(out_id);
    output.amount += n * out;
    n
}

/// The shared state of one plant.
#[derive(Default)]
pub struct Plant {
    pub input: process::Tank,
    pub output: process::Tank,
    pub temperature: f64,
    pub given: HashMap<[i32; 3], (u64, u64)>,
    pub rate: f64,
}

impl Mekanism {
    pub(crate) fn evaporation_structures(&mut self, dimension: &str, structures: Vec<Structure>) -> Vec<BlockEntityUpdate> {
        let mut out = Vec::new();
        for s in structures {
            let members: Vec<([i32; 3], Part)> = s.members.iter().filter_map(|m| part_of(&m.state).map(|p| ([m.pos.x, m.pos.y, m.pos.z], p))).collect();
            let min = [s.min.x, s.min.y, s.min.z];
            let max = [s.max.x, s.max.y, s.max.z];
            let found = validate(&members, min, max, |p| {
                world::get_block(dimension, p[0], p[1], p[2]).is_none_or(|b| matches!(b.split('[').next().unwrap_or(""), "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"))
            });
            for m in &s.members {
                let keep = self.plants.get(&(dimension.to_owned(), found.min)).map_or(AMBIENT as f32, |p| p.temperature as f32);
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

    /// A tick of the valves and controllers of the plants.
    pub(crate) fn run_evaporation(&mut self, members: &[&BlockEntity], tick: u64) -> Vec<BlockEntityUpdate> {
        let mut by: HashMap<(String, [i32; 3]), Vec<&BlockEntity>> = HashMap::new();
        for m in members {
            if let Some((f, _)) = decode(&m.data).filter(|(f, _)| f.valid) {
                by.entry((m.dimension.clone(), f.min)).or_default().push(*m);
            }
        }
        if self.evaporating.is_empty() {
            self.evaporating = load_book();
        }
        let mut out = Vec::new();
        for (key, group) in by {
            let Some((found, stored_t)) = decode(&group[0].data) else { continue };
            let in_cap = u64::from(found.height) * PER_LEVEL;
            let plant = self.plants.entry(key.clone()).or_default();
            let held = |m: &BlockEntity, i: usize| m.tanks.get(i).cloned().unwrap_or(Tank { substance: String::new(), amount: 0 });
            if plant.given.is_empty() {
                // After a load: what the valves hold, and the temperature the plant kept.
                plant.temperature = f64::from(stored_t);
                for i in 0..2 {
                    if let Some(best) = group.iter().map(|m| held(m, i)).max_by_key(|t| t.amount).filter(|t| t.amount > 0) {
                        let t = process::Tank { chemical: best.substance, amount: best.amount };
                        if i == 0 { plant.input = t } else { plant.output = t }
                    }
                }
            }
            let caps = [in_cap, OUTPUT_CAPACITY];
            for i in 0..2 {
                let (given, now): (Vec<u64>, Vec<u64>) =
                    group.iter().map(|m| (plant.given.get(&[m.pos.x, m.pos.y, m.pos.z]).map_or(0, |g| if i == 0 { g.0 } else { g.1 }), held(m, i).amount)).unzip();
                let total = if i == 0 { plant.input.amount } else { plant.output.amount };
                let next = dynamic::merge(total, &given, &now, caps[i]);
                let name = group.iter().map(|m| held(m, i)).find(|t| t.amount > 0).map(|t| t.substance).unwrap_or_default();
                let tank = if i == 0 { &mut plant.input } else { &mut plant.output };
                if tank.amount == 0 && next > 0 {
                    tank.chemical = name;
                }
                tank.amount = next;
                if next == 0 {
                    tank.chemical.clear();
                }
            }
            // The heat: the solar generators on the top corners that see the sun, then the air.
            let corners = {
                let (min, top) = (found.min, found.min[1] + i32::from(found.height) - 1);
                [[min[0], top, min[2]], [min[0] + 3, top, min[2]], [min[0], top, min[2] + 3], [min[0] + 3, top, min[2] + 3]]
            };
            let ctl = group.iter().find(|m| part_of(&m.state) == Some(Part::Controller)).copied().unwrap_or(group[0]);
            let env = world::get_environment(&ctl.dimension, ctl.pos.x, ctl.pos.y, ctl.pos.z);
            let ambient = env.as_ref().map_or(AMBIENT, |e| generators::ambient_of(e.temperature));
            let sunny = env.as_ref().is_some_and(|e| e.has_skylight && generators::sky(e.day_time, e.rain, e.thunder).1 < 4);
            let solars = if sunny {
                corners
                    .iter()
                    .filter(|c| world::get_block(&ctl.dimension, c[0], c[1], c[2]).is_some_and(|b| b.starts_with("mekanism:solar_generator") || b.starts_with("mekanism:advanced_solar_generator")))
                    .count() as u32
            } else {
                0
            };
            plant.temperature = heat(plant.temperature.max(1.0), solars, ambient);
            let mult = multiplier(plant.temperature, found.height);
            let ops = operations(mult, tick);
            let mut done = 0;
            if ops > 0 {
                let (mut i, mut o) = (plant.input.clone(), plant.output.clone());
                done = evaporate(&mut i, &mut o, &self.evaporating, ops);
                plant.input = i;
                plant.output = o;
            }
            plant.rate = if done > 0 { mult.max(0.0) } else { 0.0 };
            let (input, output) = (plant.input.clone(), plant.output.clone());
            for m in &group {
                plant.given.insert([m.pos.x, m.pos.y, m.pos.z], (input.amount, output.amount));
            }
            let (temperature, rate) = (plant.temperature, plant.rate);
            let half = |v: u64, s: u32| ((v >> s) & 0xFFFF) as i32;
            let data = vec![
                half(input.amount, 0),
                half(input.amount, 16),
                half(in_cap, 0),
                half(in_cap, 16),
                half(output.amount, 0),
                half(output.amount, 16),
                chem::index(&input.chemical),
                chem::index(&output.chemical),
                (temperature * 10.0) as i32,
                solars as i32,
                (rate * 10.0) as i32,
                i32::from(found.height),
            ];
            for m in &group {
                let mut items = None;
                let mut tanks = vec![plus::as_tank(&input), plus::as_tank(&output)];
                if part_of(&m.state) == Some(Part::Controller) && m.items.len() >= 4 {
                    // The bucket slots: a full bucket fills the input tank, an empty one is filled from the output tank.
                    let slot = |i: usize| stack(m.items.get(i).unwrap_or(&None));
                    let (mut a, mut b, mut c, mut d) = (slot(0), slot(1), slot(2), slot(3));
                    let (mut ti, mut to) = (input.clone(), output.clone());
                    let used_in = tank::buckets(&mut a, &mut b, &mut ti, in_cap, tank::EditMode::Fill);
                    let used_out = tank::buckets(&mut c, &mut d, &mut to, OUTPUT_CAPACITY, tank::EditMode::Empty);
                    if used_in || used_out {
                        items = Some(vec![item(&a), item(&b), item(&c), item(&d)]);
                        tanks = vec![plus::as_tank(&ti), plus::as_tank(&to)];
                        let plant = self.plants.get_mut(&key).expect("the plant is there");
                        plant.input = ti;
                        plant.output = to;
                    }
                }
                let keep = if part_of(&m.state) == Some(Part::Controller) && tick % 20 == 0 { encode(&found, temperature as f32) } else { m.data.clone() };
                // The input tank only takes the fluids a recipe uses, and neither takes more when full.
                let filter_in: Vec<String> = if input.amount >= in_cap { vec![NOTHING.to_owned()] } else { self.evaporating.keys().cloned().collect() };
                let _ = world::set_tank_filter(&m.dimension, m.pos.x, m.pos.y, m.pos.z, 0, &filter_in);
                out.push(BlockEntityUpdate {
                    dimension: m.dimension.clone(),
                    pos: m.pos,
                    data: keep,
                    items,
                    energy: None,
                    energy_faces: None,
                    tanks: Some(tanks),
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

    /// A 4x4x`h` tower: blocks everywhere on its shell but the roof's corners and middle, a valve in a wall and the controller in another.
    fn tower(h: i32) -> Vec<([i32; 3], Part)> {
        let mut out = Vec::new();
        for x in 0..4 {
            for y in 0..h {
                for z in 0..4 {
                    let (ex, ez) = (x == 0 || x == 3, z == 0 || z == 3);
                    let top = y == h - 1;
                    let shell = ex || ez || y == 0 || top && !(!ex && !ez);
                    if !shell || (top && ex && ez) || (top && !ex && !ez) {
                        continue;
                    }
                    let part = match (x, y, z) {
                        (0, 1, 1) => Part::Valve,
                        (3, 1, 1) => Part::Controller,
                        _ => Part::Block,
                    };
                    out.push(([x, y, z], part));
                }
            }
        }
        out
    }

    #[test]
    fn a_tower_with_a_controller_is_a_plant() {
        assert_eq!(validate(&tower(5), [0; 3], [3, 4, 3], |_| true), Found { valid: true, min: [0; 3], height: 5 });
    }

    #[test]
    fn it_needs_the_size_one_controller_and_a_closed_wall() {
        assert!(!validate(&tower(2), [0; 3], [3, 1, 3], |_| true).valid);
        assert!(!validate(&tower(19), [0; 3], [3, 18, 3], |_| true).valid);
        let mut none = tower(5);
        none.retain(|(_, p)| *p != Part::Controller);
        assert!(!validate(&none, [0; 3], [3, 4, 3], |_| true).valid);
        let mut two = tower(5);
        two.push(([3, 2, 1], Part::Controller));
        two.retain(|(p, q)| !(*p == [3, 2, 1] && *q == Part::Block));
        assert!(!validate(&two, [0; 3], [3, 4, 3], |_| true).valid);
        let mut hole = tower(5);
        hole.retain(|(p, _)| *p != [0, 2, 2]);
        assert!(!validate(&hole, [0; 3], [3, 4, 3], |_| true).valid);
        assert!(!validate(&tower(5), [0; 3], [3, 4, 3], |_| false).valid);
    }

    #[test]
    fn the_sun_heats_it_until_the_air_takes_as_much() {
        // Four solar generators settle at 100 * 4^2 degrees above the air's (heat 0.8 = 0.02 * sqrt(diff)).
        let mut t = AMBIENT;
        for _ in 0..60_000 {
            t = heat(t, 4, AMBIENT);
        }
        assert!((t - (AMBIENT + 1600.0)).abs() < 2.0, "{t}");
        assert_eq!(heat(AMBIENT, 0, AMBIENT), AMBIENT);
        assert!(heat(1000.0, 0, AMBIENT) < 1000.0);
    }

    #[test]
    fn the_temperature_sets_how_many_recipes_run() {
        assert_eq!(multiplier(AMBIENT, 18), 0.0);
        assert!((multiplier(1900.0, 18) - 640.0).abs() < 1e-9);
        assert_eq!(operations(640.0, 3), 640);
        // A slow plant runs one recipe every so many ticks.
        assert_eq!((0..10).map(|t| operations(0.25, t)).sum::<u64>(), 3);
        assert_eq!(operations(0.0, 0), 0);
    }

    #[test]
    fn evaporating_turns_the_input_into_the_output() {
        let mut book = HashMap::new();
        book.insert("minecraft:water".to_owned(), (10, "mekanism:brine".to_owned(), 1));
        let mut input = process::Tank { chemical: "minecraft:water".into(), amount: 95 };
        let mut output = process::Tank::default();
        assert_eq!(evaporate(&mut input, &mut output, &book, 100), 9);
        assert_eq!((input.amount, output.amount, output.chemical.as_str()), (5, 9, "mekanism:brine"));
        // A different fluid in the output tank blocks it; a full one too.
        let mut other = process::Tank { chemical: "mekanism:lithium".into(), amount: 1 };
        assert_eq!(evaporate(&mut input, &mut other, &book, 1), 0);
        let mut full = process::Tank { chemical: "mekanism:brine".into(), amount: OUTPUT_CAPACITY };
        let mut more = process::Tank { chemical: "minecraft:water".into(), amount: 50 };
        assert_eq!(evaporate(&mut more, &mut full, &book, 1), 0);
    }

    #[test]
    fn a_member_remembers_its_plant() {
        let f = Found { valid: true, min: [-3, 70, 12], height: 9 };
        assert_eq!(decode(&encode(&f, 812.5)), Some((f, 812.5)));
    }

    #[test]
    fn evaporation_input_capacity() {
        assert_eq!(3 * PER_LEVEL, 768_000);
        assert_eq!(18 * PER_LEVEL, 4_608_000);
    }
}

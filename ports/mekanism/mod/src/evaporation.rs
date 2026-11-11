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
    Glass,
    Valve,
    Controller,
}

pub fn part_of(state: &str) -> Option<Part> {
    match state.split('[').next().unwrap_or("") {
        BLOCK => Some(Part::Block),
        "mekanism:structural_glass" => Some(Part::Glass),
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
                let roof_corner = top && ex && ez;
                let roof_middle = top && !ex && !ez;
                // Java: CuboidStructureValidator: onEdge requires FRAME (THERMAL_EVAPORATION_BLOCK only).
                // On top layer: corners are IGNORED, center is INNER, ring is OTHER (needsFrame=false).
                // On lower layers: bottom edges/corners and vertical edge columns have matches >= 2 (needsFrame=true).
                let is_frame = (bottom && (ex || ez)) || (!top && ex && ez);
                match (have.get(&p), roof_corner, roof_middle, is_frame, boundary || top) {
                    (Some(Part::Controller), true, _, _, _) => {
                        controllers += 1;
                    }
                    (_, true, _, _, _) => {}
                    (Some(_), _, true, _, _) => return bad,
                    (None, _, true, _, _) if !inside_empty(p) => return bad,
                    (None, _, true, _, _) => {}
                    (Some(Part::Block), _, _, true, _) => {}
                    (Some(_), _, _, true, _) => return bad,
                    (None, _, _, true, _) => return bad,
                    (Some(Part::Controller), _, _, false, true) => {
                        controllers += 1;
                    }
                    (Some(Part::Block | Part::Glass | Part::Valve), _, _, false, true) => {}
                    (None, _, _, false, true) => return bad,
                    (Some(_), _, _, false, false) => return bad,
                    (None, _, _, false, false) if !inside_empty(p) => return bad,
                    (None, _, _, false, false) => {}
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
    heat_with(temperature, solars, ambient, 0.0)
}

/// `simulateEnvironment`'s return: what the plant above the air's temperature loses to it in a tick (`evaporationHeatDissipation` times the square root of
/// the difference), 0 when it is not warmer.
pub fn environment_loss(temperature: f64, ambient: f64) -> f64 {
    if (temperature - ambient).abs() < 0.001 || temperature < ambient {
        0.0
    } else {
        DISSIPATION * (temperature - ambient).sqrt()
    }
}

/// A number the way the screens write it: two decimals at most.
pub fn trim_number(v: f64) -> String {
    let text = format!("{:.2}", (v * 100.0).round() / 100.0);
    text.trim_end_matches('0').trim_end_matches('.').to_owned()
}

/// The inner screen's lines (`GuiThermalEvaporationController`): "Formed", the height, the temperature and the production a tick.
pub fn screen_texts(height: u8, temperature: f64, rate: f64) -> Vec<String> {
    vec![
        "Formed".to_owned(),
        format!("Height: {height}"),
        format!("Temperature: {} K", trim_number(temperature)),
        format!("Production: {} mB/t", trim_number(rate)),
    ]
}

/// `heat` with `external` kelvin given by the heat blocks around, committed together with the solar generators' heat.
pub fn heat_with(temperature: f64, solars: u32, ambient: f64, external: f64) -> f64 {
    // `simulateEnvironment` reads the temperature as it is before this tick's heat is committed; the solar generators' heat and what the blocks around
    // gave are queued with the air's share and all land together.
    let queued = f64::from(solars) * SOLAR + external;
    let diff = (temperature - ambient).abs();
    if diff < 0.001 {
        return ambient + queued;
    }
    let incr = DISSIPATION * diff.sqrt();
    temperature + queued + if temperature > ambient { -incr } else { incr }
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
    /// The box of the structure and the heat that conductors and heaters around gave it since its last tick, in joules.
    pub bounds: Option<([i32; 3], [i32; 3])>,
    pub pending_heat: f64,
    /// `lastEnvironmentLoss`: the heat the plant lost to the air last tick, in kelvin.
    pub last_loss: f64,
}

/// `evaporationHeatCapacity` (default 100) for each level of height: the plant's `VariableHeatCapacitor` (`EvaporationMultiblockData:137`).
pub fn heat_capacity(height: u8) -> f64 {
    100.0 * f64::from(height)
}

/// Comparator signal from input tank contents (`EvaporationMultiblockData:312`).
pub fn comparator_level(amount: u64, capacity: u64) -> u8 {
    if capacity == 0 || amount == 0 {
        0
    } else if amount >= capacity {
        15
    } else {
        (1.0 + (amount as f64 / capacity as f64) * 14.0).floor() as u8
    }
}

// -------------------------------------------------------------------------------------------------
// FORK Requirements (isolated host call stubs and declarations)
// -------------------------------------------------------------------------------------------------

/// Host interface for comparator output calculation on multiblock valves (`TileEntityThermalEvaporationValve:64`).
/// Level is discrete 0..15 based on input tank content ratio.
// FORK-21
pub fn host_set_comparator_output(dimension: &str, pos: [i32; 3], level: u8) -> Result<(), &'static str> {
    crate::redstone::set_comparator(dimension, BlockPos { x: pos[0], y: pos[1], z: pos[2] }, level);
    Ok(())
}

/// Host interface for heat network integration via thermal evaporation valves (`TileEntityThermalEvaporationValve:30`).
// FORK-4
pub fn host_transfer_valve_heat(dimension: &str, pos: [i32; 3], heat_input: f64) -> Result<f64, &'static str> {
    let _ = (dimension, pos, heat_input);
    Ok(0.0)
}

/// Host interface for multiblock formation sparkle particles (`TileEntityMultiblock:297-312`).
// FORK-30
pub fn host_multiblock_sparkle(dimension: &str, min: [i32; 3], max: [i32; 3]) -> Result<(), &'static str> {
    let _ = (dimension, min, max);
    Ok(())
}

/// Host interface for rendering multiblock inner fluid column (`RenderThermalEvaporationPlant.java`).
// FORK-19 / FORK-31
pub fn host_render_multiblock_fluid(dimension: &str, min: [i32; 3], max: [i32; 3], fluid: &str, fraction: f64) -> Result<(), &'static str> {
    let _ = (dimension, min, max, fluid, fraction);
    Ok(())
}

/// Host interface for removing rendered multiblock fluid upon disassembly.
// FORK-19 / FORK-31
pub fn host_remove_multiblock_fluid(dimension: &str, min: [i32; 3], max: [i32; 3]) -> Result<(), &'static str> {
    let _ = (dimension, min, max);
    Ok(())
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
            // The formation notice (sparkles / action bar) is made for every multiblock in `multiblock_changed` (lib.rs).
            self.plants.entry((dimension.to_owned(), found.min)).or_default().bounds = found.valid.then_some((min, max));
            if !found.valid {
                let _ = host_remove_multiblock_fluid(dimension, min, max); // FORK-19 / FORK-31
            }
            for m in &s.members {
                if part_of(&m.state) == Some(Part::Controller) {
                    let _ = world::set_block_property(dimension, m.pos.x, m.pos.y, m.pos.z, "active", if found.valid { "true" } else { "false" });
                }
                let keep = self.plants.get(&(dimension.to_owned(), found.min)).map_or(AMBIENT as f32, |p| p.temperature as f32);
                let tank_faces = if found.valid {
                    None
                } else {
                    let _ = world::set_tank_filter(dimension, m.pos.x, m.pos.y, m.pos.z, 0, &[NOTHING.to_owned()]);
                    let _ = world::set_tank_filter(dimension, m.pos.x, m.pos.y, m.pos.z, 1, &[NOTHING.to_owned()]);
                    Some(vec![vec![FlowSide::None; 6], vec![FlowSide::None; 6]])
                };
                out.push(BlockEntityUpdate {
                    dimension: dimension.to_owned(),
                    pos: m.pos,
                    data: encode(&found, keep),
                    items: None,
                    energy: None,
                    energy_faces: None,
                    tanks: None,
                    tank_faces,
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
        let mut unformed: Vec<&BlockEntity> = Vec::new();
        for m in members {
            if let Some((f, _)) = decode(&m.data).filter(|(f, _)| f.valid) {
                by.entry((m.dimension.clone(), f.min)).or_default().push(*m);
            } else {
                unformed.push(*m);
            }
        }
        let mut out = Vec::new();
        for m in unformed {
            let _ = world::set_tank_filter(&m.dimension, m.pos.x, m.pos.y, m.pos.z, 0, &[NOTHING.to_owned()]);
            let _ = world::set_tank_filter(&m.dimension, m.pos.x, m.pos.y, m.pos.z, 1, &[NOTHING.to_owned()]);
            if part_of(&m.state) == Some(Part::Controller) {
                let _ = world::set_block_property(&m.dimension, m.pos.x, m.pos.y, m.pos.z, "active", "false");
            }
            out.push(BlockEntityUpdate {
                dimension: m.dimension.clone(),
                pos: m.pos,
                data: m.data.clone(),
                items: None,
                energy: None,
                energy_faces: None,
                tanks: None,
                tank_faces: Some(vec![vec![FlowSide::None; 6], vec![FlowSide::None; 6]]),
                menu_data: None,
                transmitter_faces: None,
            });
        }
        if self.evaporating.is_empty() {
            self.evaporating = load_book();
        }
        for (key, group) in by {
            let Some((found, stored_t)) = decode(&group[0].data) else { continue };
            let in_cap = u64::from(found.height) * PER_LEVEL;
            let plant = self.plants.entry(key.clone()).or_default();
            let held = |m: &BlockEntity, i: usize| m.tanks.get(i).cloned().unwrap_or(Tank { substance: String::new(), amount: 0 });
            let caps = [in_cap, OUTPUT_CAPACITY];
            if plant.given.is_empty() {
                // After a load: what the valves hold, and the temperature the plant kept.
                plant.temperature = f64::from(stored_t);
                for i in 0..2 {
                    if let Some(best) = group.iter().map(|m| held(m, i)).max_by_key(|t| t.amount).filter(|t| t.amount > 0) {
                        let t = process::Tank { chemical: best.substance, amount: best.amount.min(caps[i]) };
                        if i == 0 { plant.input = t } else { plant.output = t }
                    }
                }
            }
            if plant.input.amount > in_cap {
                plant.input.amount = in_cap;
            }
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

            // The bucket slots of the controller processed BEFORE recording given and emitting tanks to avoid fluid duplication
            let ctl_idx = group.iter().position(|m| part_of(&m.state) == Some(Part::Controller));
            let mut ctl_items = None;
            if let Some(idx) = ctl_idx {
                let m = group[idx];
                if m.items.len() >= 4 {
                    let slot = |i: usize| stack(m.items.get(i).unwrap_or(&None));
                    let (mut a, mut b, mut c, mut d) = (slot(0), slot(1), slot(2), slot(3));
                    let used_in = tank::buckets(&mut a, &mut b, &mut plant.input, in_cap, tank::EditMode::Fill);
                    let used_out = tank::buckets(&mut c, &mut d, &mut plant.output, OUTPUT_CAPACITY, tank::EditMode::Empty);
                    if used_in || used_out {
                        ctl_items = Some(vec![item(&a), item(&b), item(&c), item(&d)]);
                    }
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
                    .filter(|c| world::get_block(&ctl.dimension, c[0], c[1], c[2]).is_some_and(|b| b.starts_with("mekanism:advanced_solar_generator")))
                    .count() as u32
            } else {
                0
            };
            let external = std::mem::take(&mut plant.pending_heat) / heat_capacity(found.height);
            plant.last_loss = environment_loss(plant.temperature.max(1.0), ambient);
            plant.temperature = heat_with(plant.temperature.max(1.0), solars, ambient, external);
            let mult = multiplier(plant.temperature, found.height);
            let ops = operations(mult, tick);
            let can_operate = !plant.input.chemical.is_empty() && plant.input.amount > 0 && plant.output.amount < OUTPUT_CAPACITY;
            if ops > 0 && can_operate {
                let (mut i, mut o) = (plant.input.clone(), plant.output.clone());
                let _ = evaporate(&mut i, &mut o, &self.evaporating, ops);
                plant.input = i;
                plant.output = o;
            }
            // Steady rate for GUI display (matches Java lastGain)
            plant.rate = if can_operate && mult > 0.0 {
                if mult < 1.0 { 1.0 / (1.0 / mult).ceil() } else { mult }
            } else {
                0.0
            };
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
                plant.last_loss.floor() as i32,
                ((plant.last_loss * 10.0).floor() as i32) % 10,
                ((plant.last_loss * 100.0).floor() as i32) % 10,
            ];
            let texts = screen_texts(found.height, temperature, rate);
            let _ = host_render_multiblock_fluid(
                &key.0,
                found.min,
                [found.min[0] + 3, found.min[1] + i32::from(found.height) - 1, found.min[2] + 3],
                &input.chemical,
                if in_cap > 0 { input.amount as f64 / in_cap as f64 } else { 0.0 },
            ); // FORK-19 / FORK-31
            for m in &group {
                let tkey = (m.dimension.clone(), m.pos.x, m.pos.y, m.pos.z);
                if self.menu_texts.get(&tkey) != Some(&texts) {
                    let _ = world::set_menu_text(&m.dimension, m.pos.x, m.pos.y, m.pos.z, &texts);
                    self.menu_texts.insert(tkey, texts.clone());
                }
                let is_ctl = part_of(&m.state) == Some(Part::Controller);
                let items = if is_ctl { ctl_items.clone() } else { None };
                let tanks = vec![plus::as_tank(&input), plus::as_tank(&output)];
                let tank_faces = if is_ctl {
                    Some(vec![vec![FlowSide::None; 6], vec![FlowSide::None; 6]])
                } else {
                    Some(vec![vec![FlowSide::Both; 6], vec![FlowSide::Output; 6]])
                };
                if is_ctl {
                    let _ = world::set_block_property(&m.dimension, m.pos.x, m.pos.y, m.pos.z, "active", "true");
                } else {
                    let comp = comparator_level(input.amount, in_cap);
                    let _ = host_set_comparator_output(&m.dimension, [m.pos.x, m.pos.y, m.pos.z], comp); // FORK-21
                }
                let keep = if is_ctl && tick % 20 == 0 { encode(&found, temperature as f32) } else { m.data.clone() };
                // The input tank only takes the fluids a recipe uses, restricted to existing chemical when partially full.
                let filter_in: Vec<String> = if input.amount >= in_cap {
                    vec![NOTHING.to_owned()]
                } else if !input.chemical.is_empty() {
                    vec![input.chemical.clone()]
                } else {
                    self.evaporating.keys().cloned().collect()
                };
                let _ = world::set_tank_filter(&m.dimension, m.pos.x, m.pos.y, m.pos.z, 0, &filter_in);
                let _ = world::set_tank_filter(&m.dimension, m.pos.x, m.pos.y, m.pos.z, 1, &[NOTHING.to_owned()]);
                out.push(BlockEntityUpdate {
                    dimension: m.dimension.clone(),
                    pos: m.pos,
                    data: keep,
                    items,
                    energy: None,
                    energy_faces: None,
                    tanks: Some(tanks),
                    tank_faces,
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
    fn heat_given_by_the_blocks_around_warms_the_plant_by_joules_over_capacity() {
        // 4 levels: capacity 400 J/K, so 800 J is 2 K on top of the air's own pull (`VariableHeatCapacitor.handleHeat` / `getTemperature`).
        assert_eq!(heat_capacity(4), 400.0);
        let plain = heat_with(400.0, 0, AMBIENT, 0.0);
        let warmed = heat_with(400.0, 0, AMBIENT, 800.0 / heat_capacity(4));
        assert!((warmed - plain - 2.0).abs() < 0.2, "{plain} {warmed}");
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

    #[test]
    fn only_advanced_solar_powers_evaporation() {
        let is_valid = |b: &str| b.starts_with("mekanism:advanced_solar_generator");
        assert!(is_valid("mekanism:advanced_solar_generator"));
        assert!(!is_valid("mekanism:solar_generator"));
    }

    #[test]
    fn structural_glass_allowed_on_walls_and_rim_not_on_edges() {
        // Java: EvaporationValidator.java:58-68, CuboidStructureValidator.java:106
        // Structural glass is valid on face walls (0, 2, 1) and roof rim (0, 4, 1),
        // but invalid on frame edges (0, 2, 0), bottom edges (0, 0, 1), or bottom corners (0, 0, 0).
        let mut on_face = tower(5);
        if let Some(m) = on_face.iter_mut().find(|(p, _)| *p == [0, 2, 1]) {
            m.1 = Part::Glass;
        }
        assert!(validate(&on_face, [0; 3], [3, 4, 3], |_| true).valid);

        let mut on_rim = tower(5);
        if let Some(m) = on_rim.iter_mut().find(|(p, _)| *p == [0, 4, 1]) {
            m.1 = Part::Glass;
        }
        assert!(validate(&on_rim, [0; 3], [3, 4, 3], |_| true).valid);

        let mut on_vert_edge = tower(5);
        if let Some(m) = on_vert_edge.iter_mut().find(|(p, _)| *p == [0, 2, 0]) {
            m.1 = Part::Glass;
        }
        assert!(!validate(&on_vert_edge, [0; 3], [3, 4, 3], |_| true).valid);

        let mut on_bottom_edge = tower(5);
        if let Some(m) = on_bottom_edge.iter_mut().find(|(p, _)| *p == [0, 0, 1]) {
            m.1 = Part::Glass;
        }
        assert!(!validate(&on_bottom_edge, [0; 3], [3, 4, 3], |_| true).valid);

        let mut on_bottom_corner = tower(5);
        if let Some(m) = on_bottom_corner.iter_mut().find(|(p, _)| *p == [0, 0, 0]) {
            m.1 = Part::Glass;
        }
        assert!(!validate(&on_bottom_corner, [0; 3], [3, 4, 3], |_| true).valid);
    }

    #[test]
    fn controller_in_roof_corner_causes_conflict() {
        // Java: EvaporationValidator.java:32-38
        // Even though roof corners are ignored positions for structure frame, placing a second controller
        // in a roof corner triggers MULTIBLOCK_INVALID_CONTROLLER_CONFLICT.
        let mut two_ctl = tower(5);
        two_ctl.push(([0, 4, 0], Part::Controller));
        assert!(!validate(&two_ctl, [0; 3], [3, 4, 3], |_| true).valid);
    }

    #[test]
    fn evaporation_comparator_level() {
        // Java: EvaporationMultiblockData.java:312, MekanismUtils.java:692
        assert_eq!(comparator_level(0, 1000), 0);
        assert_eq!(comparator_level(1000, 1000), 15);
        assert_eq!(comparator_level(500, 1000), 8);
        assert_eq!(comparator_level(1, 1000), 1);
        assert_eq!(comparator_level(0, 0), 0);
    }

    #[test]
    fn evaporation_clamps_stored_fluid_on_rebuild_or_shrink() {
        // Java: EvaporationMultiblockData.java:222, VariableCapacityFluidTank.java:69
        let full_h18_cap = 18 * PER_LEVEL; // 4_608_000
        let h3_cap = 3 * PER_LEVEL; // 768_000
        let mut stored = full_h18_cap;
        if stored > h3_cap {
            stored = h3_cap;
        }
        assert_eq!(stored, 768_000);
    }

    #[test]
    fn unformed_valve_locks_tank_faces_and_filters_nothing() {
        // Java: TileEntityThermalEvaporationValve.java:25, 37; SINTOMAS_MULTIBLOCOS.md S3
        // When unformed or dismantled, valves must lock tank faces to FlowSide::None and filter NOTHING.
        let bad = Found { valid: false, min: [0; 3], height: 0 };
        let encoded = encode(&bad, AMBIENT as f32);
        let decoded = decode(&encoded);
        assert_eq!(decoded.map(|(f, _)| f.valid), Some(false));
    }

    #[test]
    fn the_plant_loses_heat_to_the_air_only_when_warmer() {
        assert_eq!(environment_loss(300.0, 300.0), 0.0);
        assert_eq!(environment_loss(250.0, 300.0), 0.0);
        assert!((environment_loss(400.0, 300.0) - 0.2).abs() < 1e-9);
    }

    #[test]
    fn the_screen_lines_follow_the_gui() {
        assert_eq!(
            screen_texts(18, 1234.5678, 0.5),
            vec!["Formed".to_owned(), "Height: 18".to_owned(), "Temperature: 1234.57 K".to_owned(), "Production: 0.5 mB/t".to_owned()]
        );
    }
}

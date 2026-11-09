//! The fission reactor (`FissionReactorMultiblockData`, `FissionReactorValidator`): a hollow cuboid (3 to 18 on a side) of fission reactor
//! casing with ports and logic adapters on its faces. Inside, columns of fuel assemblies topped by one control rod assembly each. Fissile fuel
//! burns into heat and nuclear waste; the heat boils the coolant (water into steam, sodium into superheated sodium) that leaves by the ports.
//! Too much heat damages the reactor, and a damaged one that stays hot melts down.
//!
//! As the boiler, the ports hold mirrors of the shared tanks (water, coolant, fuel, heated coolant, waste). Unlike the original, a port has no
//! mode: every port takes fuel and coolant in and gives heated coolant and waste out. The logic adapter is a plain wall block.
use super::*;
use std::collections::{HashMap, HashSet};

pub const CASING: &str = "mekanism:fission_reactor_casing";
pub const PORT: &str = "mekanism:fission_reactor_port";
pub const ADAPTER: &str = "mekanism:fission_reactor_logic_adapter";
pub const FUEL: &str = "mekanism:fission_fuel_assembly";
pub const ROD: &str = "mekanism:control_rod_assembly";
pub const GROUP: &str = "mekanism:fission";
pub const WATER: &str = "minecraft:water";
pub const STEAM: &str = "mekanism:steam";
pub const SODIUM: &str = "mekanism:sodium";
pub const HOT_SODIUM: &str = "mekanism:superheated_sodium";
pub const FISSILE: &str = "mekanism:fissile_fuel";
pub const WASTE: &str = "mekanism:nuclear_waste";
pub const NOTHING: &str = "mekanism:nothing";
pub const SPEC_CAPACITY: u64 = 1 << 50;
/// The constants of the reactor's heat and the config's defaults.
pub const INSULATION: f64 = 10_000.0;
pub const CONDUCTION: f64 = 10.0;
pub const AIR: f64 = 10_000.0;
pub const WATER_CONDUCTIVITY: f64 = 0.5;
pub const BASE_BOIL: f64 = 373.15;
pub const WATER_ENTHALPY: f64 = 10.0 / 0.2;
pub const SODIUM_ENTHALPY: f64 = 5.0;
pub const SODIUM_CONDUCTIVITY: f64 = 1.0;
pub const MIN_DAMAGE_TEMP: f64 = 1_200.0;
pub const MAX_DAMAGE_TEMP: f64 = 1_800.0;
pub const MAX_DAMAGE: f64 = 100.0;
pub const HEAT_PER_FUEL: f64 = 1_000_000.0;
pub const CASING_HEAT: f64 = 1_000.0;
pub const SURFACE_TARGET: f64 = 4.0;
pub const DEFAULT_RATE: f64 = 0.1;
pub const BURN_PER_ASSEMBLY: f64 = 1.0;
pub const FUEL_PER_ASSEMBLY: u64 = 8_000;
pub const COOLANT_PER_VOLUME: u64 = 100_000;
pub const HEATED_PER_VOLUME: u64 = 1_000_000;
pub const MELTDOWN_CHANCE: f64 = 0.001;
pub const MELTDOWN_RADIUS: i32 = 8;
pub const POST_MELTDOWN_DAMAGE: f64 = 75.0;
/// The buttons of the screen.
pub const TOGGLE: i32 = 7100;
pub const RATE_DOWN_1: i32 = 7101;
pub const RATE_DOWN_TENTH: i32 = 7102;
pub const RATE_UP_TENTH: i32 = 7103;
pub const RATE_UP_1: i32 = 7104;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Casing,
    Port,
    Adapter,
    Fuel,
    Rod,
}

pub fn part_of(state: &str) -> Option<Part> {
    match state.split('[').next().unwrap_or("") {
        CASING => Some(Part::Casing),
        PORT => Some(Part::Port),
        ADAPTER => Some(Part::Adapter),
        FUEL => Some(Part::Fuel),
        ROD => Some(Part::Rod),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Found {
    pub valid: bool,
    pub min: [i32; 3],
    pub size: [u8; 3],
    pub volume: u32,
    pub blocks: u32,
    pub assemblies: u32,
    pub surface: u32,
}

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
    let mut columns: HashMap<(i32, i32), (Vec<i32>, Vec<i32>)> = HashMap::new();
    let mut fuels = HashSet::new();
    for x in min[0]..=max[0] {
        for y in min[1]..=max[1] {
            for z in min[2]..=max[2] {
                let p = [x, y, z];
                let edges = [x == min[0] || x == max[0], y == min[1] || y == max[1], z == min[2] || z == max[2]].iter().filter(|b| **b).count();
                match (edges, have.get(&p)) {
                    (0, None) if !inside_empty(p) => return bad,
                    (0, None) => {}
                    (0, Some(Part::Fuel)) => {
                        columns.entry((x, z)).or_default().0.push(y);
                        fuels.insert(p);
                    }
                    (0, Some(Part::Rod)) => columns.entry((x, z)).or_default().1.push(y),
                    (0, Some(_)) => return bad,
                    (_, Some(Part::Casing)) => {}
                    (1, Some(Part::Port | Part::Adapter)) => {}
                    _ => return bad,
                }
            }
        }
    }
    if columns.is_empty() {
        return bad;
    }
    // Each column: fuel assemblies one over the other, a single control rod right over the last.
    for (fuel, rods) in columns.values() {
        if fuel.is_empty() || rods.len() != 1 {
            return bad;
        }
        let mut ys = fuel.clone();
        ys.sort_unstable();
        if ys.windows(2).any(|w| w[1] != w[0] + 1) || rods[0] != ys[ys.len() - 1] + 1 {
            return bad;
        }
    }
    // The surface the fuel shows: six faces each, less two for every pair of touching assemblies.
    let mut surface = 6 * fuels.len() as u32;
    for p in &fuels {
        for q in [[p[0] + 1, p[1], p[2]], [p[0], p[1] + 1, p[2]], [p[0], p[1], p[2] + 1]] {
            if fuels.contains(&q) {
                surface -= 2;
            }
        }
    }
    Found {
        valid: true,
        min,
        size: [size[0] as u8, size[1] as u8, size[2] as u8],
        volume: ((size[0] - 2) * (size[1] - 2) * (size[2] - 2)) as u32,
        blocks: members.len() as u32,
        assemblies: fuels.len() as u32,
        surface,
    }
}

/// What the reactor keeps between ticks and across loads.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Reactor {
    pub temperature: f64,
    pub damage: f64,
    pub burn_remaining: f64,
    pub partial_waste: f64,
    pub rate_limit: f64,
    pub active: bool,
    pub force_disable: bool,
    pub fuel: process::Tank,
    pub water: process::Tank,
    pub coolant: process::Tank,
    pub heated: process::Tank,
    pub waste: process::Tank,
    pub last_burn: f64,
    pub last_boil: u64,
    /// Radiation (Sv/h) the reactor leaked this tick: waste that did not fit in its tank.
    pub leak: f64,
}

pub fn encode(f: &Found, r: &Reactor) -> Vec<u8> {
    let mut out = vec![u8::from(f.valid)];
    for c in f.min {
        out.extend(c.to_le_bytes());
    }
    out.extend(f.size);
    for v in [f.volume, f.blocks, f.assemblies, f.surface] {
        out.extend(v.to_le_bytes());
    }
    for v in [r.temperature, r.damage, r.burn_remaining, r.partial_waste, r.rate_limit] {
        out.extend((v as f32).to_le_bytes());
    }
    out.push(u8::from(r.active) | (u8::from(r.force_disable) << 1));
    out
}

pub fn decode(data: &[u8]) -> Option<(Found, Reactor)> {
    if data.len() < 53 {
        return None;
    }
    let int = |at: usize| i32::from_le_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]);
    let float = |at: usize| f64::from(f32::from_le_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]));
    let found = Found {
        valid: data[0] == 1,
        min: [int(1), int(5), int(9)],
        size: [data[13], data[14], data[15]],
        volume: int(16) as u32,
        blocks: int(20) as u32,
        assemblies: int(24) as u32,
        surface: int(28) as u32,
    };
    let reactor = Reactor {
        temperature: float(32),
        damage: float(36),
        burn_remaining: float(40),
        partial_waste: float(44),
        rate_limit: float(48),
        active: data[52] & 1 == 1,
        force_disable: data[52] & 2 == 2,
        ..Reactor::default()
    };
    Some((found, reactor))
}

/// The mode of a port (`FissionPortMode`), kept in the byte after its data: 0 takes everything in, 1 gives waste out,
/// 2 gives heated coolant out. A port saved before the configurator kept it has no byte and takes in.
pub fn mode_of(data: &[u8]) -> u8 {
    data.get(53).copied().unwrap_or(0).min(2)
}

/// The name of a port mode, as the original shows it.
pub fn mode_name(mode: u8) -> &'static str {
    match mode {
        1 => "output waste",
        2 => "output coolant",
        _ => "input only",
    }
}

/// The port's data with its mode byte after it.
pub fn with_mode(mut data: Vec<u8>, mode: u8) -> Vec<u8> {
    data.truncate(53);
    data.push(mode);
    data
}

pub fn max_burn(f: &Found) -> f64 {
    f64::from(f.assemblies) * BURN_PER_ASSEMBLY
}

pub fn boil_efficiency(f: &Found) -> f64 {
    if f.assemblies == 0 {
        return 0.0;
    }
    (f64::from(f.surface) / f64::from(f.assemblies) / SURFACE_TARGET).min(1.0)
}

pub fn fuel_capacity(f: &Found) -> u64 {
    u64::from(f.assemblies) * FUEL_PER_ASSEMBLY
}

pub fn coolant_capacity(f: &Found) -> u64 {
    u64::from(f.volume) * COOLANT_PER_VOLUME
}

pub fn heated_capacity(f: &Found) -> u64 {
    u64::from(f.volume) * HEATED_PER_VOLUME
}

/// One tick: the fuel burns, the coolant boils, the air takes heat and the damage follows. Returns whether the reactor melts down (`roll` is a
/// random number in 0..1).
pub fn tick(r: &mut Reactor, f: &Found, ambient: f64, roll: f64) -> bool {
    let capacity = CASING_HEAT * f64::from(f.blocks).max(1.0);
    // The fuel burns.
    r.last_burn = 0.0;
    r.leak = 0.0;
    if r.active && r.fuel.amount > 0 && r.fuel.chemical == FISSILE {
        let stored = r.fuel.amount as f64 + r.burn_remaining;
        let burn = r.rate_limit.min(stored).min(max_burn(f));
        let left = stored - burn;
        r.fuel.amount = (left.floor() as u64).min(fuel_capacity(f));
        if r.fuel.amount == 0 {
            r.fuel.chemical.clear();
        }
        r.burn_remaining = left % 1.0;
        r.temperature += burn * HEAT_PER_FUEL / capacity;
        r.partial_waste += burn;
        let new_waste = r.partial_waste.floor() as u64;
        if new_waste > 0 {
            r.partial_waste %= 1.0;
            let mut kept = 0;
            if r.waste.amount == 0 || r.waste.chemical == WASTE {
                let room = fuel_capacity(f).saturating_sub(r.waste.amount);
                r.waste.chemical = WASTE.to_owned();
                kept = new_waste.min(room);
                r.waste.amount += kept;
            }
            // What does not fit leaks as radiation.
            r.leak += (new_waste - kept) as f64 * radiation::radioactivity(WASTE);
        }
        r.last_burn = burn;
    }
    // The coolant boils.
    r.last_boil = 0;
    let (tank, conductivity, enthalpy, heated) = if r.water.amount > 0 {
        (&mut r.water, WATER_CONDUCTIVITY, WATER_ENTHALPY, STEAM)
    } else if r.coolant.amount > 0 && r.coolant.chemical == SODIUM {
        (&mut r.coolant, SODIUM_CONDUCTIVITY, SODIUM_ENTHALPY, HOT_SODIUM)
    } else {
        (&mut r.water, 0.0, 1.0, STEAM)
    };
    if conductivity > 0.0 {
        let heat = boil_efficiency(f) * capacity * (r.temperature - BASE_BOIL);
        let boil = ((heat * conductivity / enthalpy).max(0.0).min(tank.amount as f64)) as u64;
        if boil > 0 {
            tank.amount -= boil;
            if tank.amount == 0 {
                tank.chemical.clear();
            }
            // The extra steam is dumped: only what fits is kept.
            if r.heated.amount == 0 || r.heated.chemical == heated {
                let room = heated_capacity(f).saturating_sub(r.heated.amount);
                r.heated.chemical = heated.to_owned();
                r.heated.amount += boil.min(room);
            }
            r.temperature -= boil as f64 * enthalpy / capacity;
            r.last_boil = boil;
        }
    }
    // The air takes heat away.
    r.temperature -= (r.temperature - ambient) / (AIR + INSULATION + CONDUCTION);
    // The damage.
    let t = r.temperature;
    if t > MIN_DAMAGE_TEMP {
        r.damage += t.min(MAX_DAMAGE_TEMP) / (MIN_DAMAGE_TEMP * 10.0);
    } else {
        r.damage = (r.damage - (MIN_DAMAGE_TEMP - t) / (MIN_DAMAGE_TEMP * 100.0)).max(0.0);
    }
    if r.damage >= MAX_DAMAGE && t >= MIN_DAMAGE_TEMP {
        if roll < (r.damage / MAX_DAMAGE) * MELTDOWN_CHANCE {
            return true;
        }
    } else if r.damage < MAX_DAMAGE && t < MIN_DAMAGE_TEMP {
        r.force_disable = false;
    }
    false
}

/// What a meltdown leaves of the reactor itself.
pub fn after_meltdown(r: &mut Reactor, ambient: f64) {
    r.active = false;
    r.damage = POST_MELTDOWN_DAMAGE;
    r.burn_remaining = 0.0;
    r.partial_waste = 0.0;
    r.temperature = ambient;
    r.heated = process::Tank::default();
    r.fuel = process::Tank::default();
    r.waste = process::Tank::default();
}

/// A press of a button of the screen.
pub fn press(r: &mut Reactor, f: &Found, id: i32) {
    let max = max_burn(f);
    let set = |r: &mut Reactor, v: f64| r.rate_limit = ((v.clamp(0.0, max)) * 100.0).round() / 100.0;
    match id {
        TOGGLE => {
            if r.active {
                r.active = false;
            } else if !r.force_disable {
                r.active = true;
            }
        }
        RATE_DOWN_1 => set(r, r.rate_limit - 1.0),
        RATE_DOWN_TENTH => set(r, r.rate_limit - 0.1),
        RATE_UP_TENTH => set(r, r.rate_limit + 0.1),
        RATE_UP_1 => set(r, r.rate_limit + 1.0),
        _ => {}
    }
}

/// A random number in 0..1 from the tick and the place.
pub fn roll(tick: u64, salt: i32) -> f64 {
    let mut x = tick.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (salt as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 31;
    x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 29;
    (x >> 11) as f64 / (1u64 << 53) as f64
}

#[derive(Default)]
pub struct Shared {
    pub reactor: Reactor,
    pub started: bool,
    pub given: HashMap<[i32; 3], [u64; 5]>,
}

impl Mekanism {
    pub(crate) fn fission_structures(&mut self, dimension: &str, structures: Vec<Structure>) -> Vec<BlockEntityUpdate> {
        let mut out = Vec::new();
        for s in structures {
            let members: Vec<([i32; 3], Part)> = s.members.iter().filter_map(|m| part_of(&m.state).map(|p| ([m.pos.x, m.pos.y, m.pos.z], p))).collect();
            let min = [s.min.x, s.min.y, s.min.z];
            let max = [s.max.x, s.max.y, s.max.z];
            let found = validate(&members, min, max, |p| {
                world::get_block(dimension, p[0], p[1], p[2]).is_none_or(|b| matches!(b.split('[').next().unwrap_or(""), "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"))
            });
            let keep = self.reactors.get(&(dimension.to_owned(), found.min)).map(|s| s.reactor.clone()).unwrap_or_default();
            for m in &s.members {
                // A port keeps the mode its tick last saw (see `port_modes` in lib.rs): the validation rewrites every
                // member's data, and a member carries no data of its own in `multiblock-changed`. Other members have
                // no mode byte and keep the plain encoding.
                let data = if part_of(&m.state) == Some(Part::Port) {
                    let mode = self.port_modes.get(&(dimension.to_owned(), m.pos.x, m.pos.y, m.pos.z)).copied().unwrap_or(0);
                    with_mode(encode(&found, &keep), mode)
                } else {
                    encode(&found, &keep)
                };
                out.push(BlockEntityUpdate {
                    dimension: dimension.to_owned(),
                    pos: m.pos,
                    data,
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

    /// A tick of the ports of the reactors.
    pub(crate) fn run_fission(&mut self, ports: &[&BlockEntity], tick: u64) -> Vec<BlockEntityUpdate> {
        let presses = std::mem::take(&mut self.fission_buttons);
        let mut by: HashMap<(String, [i32; 3]), Vec<&BlockEntity>> = HashMap::new();
        for p in ports {
            if let Some((f, _)) = decode(&p.data).filter(|(f, _)| f.valid) {
                by.entry((p.dimension.clone(), f.min)).or_default().push(*p);
            }
        }
        let mut out = Vec::new();
        for (key, group) in by {
            let Some((found, saved)) = decode(&group[0].data) else { continue };
            let caps = [coolant_capacity(&found), coolant_capacity(&found), fuel_capacity(&found), heated_capacity(&found), fuel_capacity(&found)];
            let s = self.reactors.entry(key.clone()).or_default();
            let held = |p: &BlockEntity, i: usize| p.tanks.get(i).cloned().unwrap_or(Tank { substance: String::new(), amount: 0 });
            if !s.started {
                s.started = true;
                s.reactor = Reactor { rate_limit: if saved.rate_limit > 0.0 { saved.rate_limit } else { DEFAULT_RATE }, ..saved };
                for i in 0..5 {
                    if let Some(best) = group.iter().map(|p| held(p, i)).max_by_key(|t| t.amount).filter(|t| t.amount > 0) {
                        let t = process::Tank { chemical: best.substance, amount: best.amount.min(caps[i]) };
                        match i {
                            0 => s.reactor.water = t,
                            1 => s.reactor.coolant = t,
                            2 => s.reactor.fuel = t,
                            3 => s.reactor.heated = t,
                            _ => s.reactor.waste = t,
                        }
                    }
                }
            } else {
                for i in 0..5 {
                    let (given, now): (Vec<u64>, Vec<u64>) =
                        group.iter().map(|p| (s.given.get(&[p.pos.x, p.pos.y, p.pos.z]).map_or(0, |g| g[i]), held(p, i).amount)).unzip();
                    let tank = match i {
                        0 => &mut s.reactor.water,
                        1 => &mut s.reactor.coolant,
                        2 => &mut s.reactor.fuel,
                        3 => &mut s.reactor.heated,
                        _ => &mut s.reactor.waste,
                    };
                    let next = dynamic::merge(tank.amount, &given, &now, caps[i]);
                    if tank.amount == 0 && next > 0 {
                        tank.chemical = group.iter().map(|p| held(p, i)).find(|t| t.amount > 0).map(|t| t.substance).unwrap_or_default();
                    }
                    tank.amount = next;
                    if next == 0 {
                        tank.chemical.clear();
                    }
                }
            }
            for (dim, pos, id) in presses.iter().filter(|(d, p, _)| *d == key.0 && group.iter().any(|g| (g.pos.x, g.pos.y, g.pos.z) == (p.x, p.y, p.z))) {
                let _ = (dim, pos);
                press(&mut s.reactor, &found, *id);
            }
            s.reactor.rate_limit = s.reactor.rate_limit.min(max_burn(&found));
            // The tanks only keep what they are for.
            if s.reactor.water.amount > 0 && s.reactor.water.chemical != WATER {
                s.reactor.water = process::Tank::default();
            }
            if s.reactor.coolant.amount > 0 && s.reactor.coolant.chemical != SODIUM {
                s.reactor.coolant = process::Tank::default();
            }
            if s.reactor.fuel.amount > 0 && s.reactor.fuel.chemical != FISSILE {
                s.reactor.fuel = process::Tank::default();
            }
            let env = world::get_environment(&group[0].dimension, group[0].pos.x, group[0].pos.y, group[0].pos.z);
            let ambient = env.as_ref().map_or(300.0, |e| generators::ambient_of(e.temperature));
            if s.reactor.temperature < 1.0 {
                s.reactor.temperature = ambient;
            }
            let melted = tick_reactor(&mut s.reactor, &found, ambient, tick, key.1[0] ^ key.1[2]);
            if melted {
                let (dim, min) = (key.0.clone(), key.1);
                let size = found.size.map(i32::from);
                let centre = [min[0] + size[0] / 2, min[1] + size[1] / 2, min[2] + size[2] / 2];
                // `meltdownHappened`: the radioactive substances in its tanks are released, fifty times over.
                let waste_chem = if s.reactor.waste.amount == 0 { WASTE } else { &s.reactor.waste.chemical };
                let released = (s.reactor.waste.amount as f64 + s.reactor.partial_waste) * radiation::radioactivity(waste_chem)
                    + s.reactor.fuel.amount as f64 * radiation::radioactivity(&s.reactor.fuel.chemical)
                    + s.reactor.heated.amount as f64 * radiation::radioactivity(&s.reactor.heated.chemical);
                after_meltdown(&mut s.reactor, ambient);
                self.radiation.radiate(&dim, centre, released * 50.0);
                self.meltdown(&dim, centre);
            }
            let s = self.reactors.get_mut(&key).expect("the reactor is there");
            let r = s.reactor.clone();
            if r.leak > 0.0 {
                let size = found.size.map(i32::from);
                let centre = [key.1[0] + size[0] / 2, key.1[1] + size[1] / 2, key.1[2] + size[2] / 2];
                self.radiation.radiate(&key.0, centre, r.leak);
            }
            let s = self.reactors.get_mut(&key).expect("the reactor is there");
            let tanks_now = [r.water.clone(), r.coolant.clone(), r.fuel.clone(), r.heated.clone(), r.waste.clone()];
            for p in &group {
                s.given.insert([p.pos.x, p.pos.y, p.pos.z], [tanks_now[0].amount, tanks_now[1].amount, tanks_now[2].amount, tanks_now[3].amount, tanks_now[4].amount]);
            }
            let half = |v: u64, s: u32| ((v >> s) & 0xFFFF) as i32;
            let coolant_shown = if r.water.amount > 0 { &r.water } else { &r.coolant };
            let kind = |t: &process::Tank, fallback: &str| chem::index(if t.amount == 0 { fallback } else { &t.chemical });
            let data = vec![
                half(r.fuel.amount, 0),
                half(r.fuel.amount, 16),
                half(caps[2], 0),
                half(caps[2], 16),
                half(coolant_shown.amount, 0),
                half(coolant_shown.amount, 16),
                half(caps[0], 0),
                half(caps[0], 16),
                half(r.heated.amount, 0),
                half(r.heated.amount, 16),
                half(caps[3], 0),
                half(caps[3], 16),
                half(r.waste.amount, 0),
                half(r.waste.amount, 16),
                half(caps[4], 0),
                half(caps[4], 16),
                chem::index(FISSILE),
                kind(coolant_shown, WATER),
                kind(&r.heated, STEAM),
                chem::index(WASTE),
                (r.temperature * 10.0).min(f64::from(i32::MAX)) as i32,
                r.damage.round().min(100.0) as i32,
                (r.rate_limit * 100.0).round() as i32,
                (r.last_burn * 100.0).round() as i32,
                max_burn(&found) as i32,
                r.last_boil.min(i32::MAX as u64) as i32,
                if r.active { 1 } else if r.force_disable { 2 } else { 0 },
                found.assemblies as i32,
                (boil_efficiency(&found) * 100.0).round() as i32,
                found.surface as i32,
            ];
            for p in &group {
                let (x, y, z) = (p.pos.x, p.pos.y, p.pos.z);
                let full = |amount: u64, cap: u64, ok: &str, other: bool| if amount >= cap || other { vec![NOTHING.to_owned()] } else { vec![ok.to_owned()] };
                let _ = world::set_tank_filter(&p.dimension, x, y, z, 0, &full(r.water.amount, caps[0], WATER, r.coolant.amount > 0));
                let _ = world::set_tank_filter(&p.dimension, x, y, z, 1, &full(r.coolant.amount, caps[1], SODIUM, r.water.amount > 0));
                let _ = world::set_tank_filter(&p.dimension, x, y, z, 2, &full(r.fuel.amount, caps[2], FISSILE, false));
                // A sneak click with the configurator cycles what the port does (`TileEntityFissionReactorPort`
                // on sneak right click): take in, give waste out, give heated coolant out. A plain click says which.
                let mut mode = mode_of(&p.data);
                let mut mode_changed = false;
                if let Some(click) = self.take_click(p) {
                    if click.sneak && click.mode.is_configurating() {
                        mode = (mode + 1) % 3;
                        mode_changed = true;
                        let _ = players::tell(&click.player, &format!("Valve mode changed to: {}", mode_name(mode)));
                    } else if click.mode.is_configurating() {
                        let _ = players::tell(&click.player, &format!("Reactor Port mode: {}", mode_name(mode)));
                    }
                }
                // Remember the mode for the structures function: it rewrites the port's data on every revalidation
                // and cannot read the mode from the member it is given.
                self.port_modes.insert((p.dimension.clone(), p.pos.x, p.pos.y, p.pos.z), mode);
                let keep = if mode_changed || tick % 20 == 0 { with_mode(encode(&found, &r), mode) } else { p.data.clone() };
                // What each face does with the tanks, in the order water, coolant, fuel, heated coolant, waste.
                let tank_faces = match mode {
                    1 => vec![vec![FlowSide::None; 6], vec![FlowSide::None; 6], vec![FlowSide::None; 6], vec![FlowSide::None; 6], vec![FlowSide::Output; 6]],
                    2 => vec![vec![FlowSide::None; 6], vec![FlowSide::None; 6], vec![FlowSide::None; 6], vec![FlowSide::Output; 6], vec![FlowSide::None; 6]],
                    _ => vec![vec![FlowSide::Input; 6], vec![FlowSide::Input; 6], vec![FlowSide::Input; 6], vec![FlowSide::None; 6], vec![FlowSide::None; 6]],
                };
                out.push(BlockEntityUpdate {
                    dimension: p.dimension.clone(),
                    pos: p.pos,
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

    /// `createMeltdown`: the reactor blows up; everything within the radius but what an explosion could not break is gone.
    fn meltdown(&self, dimension: &str, centre: [i32; 3]) {
        let r = MELTDOWN_RADIUS;
        let mut changes = Vec::new();
        for x in -r..=r {
            for y in -r..=r {
                for z in -r..=r {
                    if x * x + y * y + z * z > r * r {
                        continue;
                    }
                    let (px, py, pz) = (centre[0] + x, centre[1] + y, centre[2] + z);
                    let Some(block) = world::get_block(dimension, px, py, pz) else { continue };
                    let name = block.split('[').next().unwrap_or("");
                    if name.ends_with(":air") || name.contains("bedrock") || name.contains("obsidian") || name.contains("barrier") || name.contains("portal") {
                        continue;
                    }
                    changes.push(BlockChange { x: px, y: py, z: pz, state: "minecraft:air".into() });
                }
            }
        }
        for batch in changes.chunks(4096) {
            let _ = world::set_blocks(dimension, batch);
        }
    }
}

fn tick_reactor(r: &mut Reactor, f: &Found, ambient: f64, tick_no: u64, salt: i32) -> bool {
    tick(r, f, ambient, roll(tick_no, salt))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A hollow box `n` on a side with a port, and in the middle column `fuel` fuel assemblies under a control rod.
    fn reactor(n: i32, fuel: i32) -> Vec<([i32; 3], Part)> {
        let mut out = Vec::new();
        let c = n / 2;
        for x in 0..n {
            for y in 0..n {
                for z in 0..n {
                    let edge = |v: i32| v == 0 || v == n - 1;
                    let e = [edge(x), edge(y), edge(z)].iter().filter(|b| **b).count();
                    if e > 0 {
                        out.push(([x, y, z], if (x, y, z) == (0, 1, 1) { Part::Port } else { Part::Casing }));
                    } else if (x, z) == (c, c) && y >= 1 && y <= fuel {
                        out.push(([x, y, z], Part::Fuel));
                    } else if (x, z) == (c, c) && y == fuel + 1 {
                        out.push(([x, y, z], Part::Rod));
                    }
                }
            }
        }
        out
    }

    #[test]
    fn a_column_of_fuel_under_a_rod_is_a_reactor() {
        let f = validate(&reactor(5, 2), [0; 3], [4; 3], |_| true);
        assert!(f.valid);
        assert_eq!((f.assemblies, f.surface, f.volume, f.blocks), (2, 10, 27, reactor(5, 2).len() as u32));
        assert!((boil_efficiency(&f) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn the_columns_have_to_be_whole() {
        let check = |m: &[([i32; 3], Part)]| validate(m, [0; 3], [4; 3], |_| true).valid;
        let mut no_rod = reactor(5, 2);
        no_rod.retain(|(_, p)| *p != Part::Rod);
        assert!(!check(&no_rod));
        let mut gap = reactor(5, 3);
        gap.retain(|(p, _)| *p != [2, 2, 2]);
        assert!(!check(&gap));
        let mut none = reactor(5, 2);
        none.retain(|(_, p)| !matches!(p, Part::Fuel | Part::Rod));
        assert!(!check(&none));
        let mut two = reactor(5, 1);
        two.push(([2, 3, 2], Part::Rod));
        assert!(!check(&two));
        assert!(!validate(&reactor(5, 2), [0; 3], [4; 3], |_| false).valid);
    }

    fn running(f: &Found) -> Reactor {
        Reactor {
            temperature: 300.0,
            rate_limit: 0.5,
            active: true,
            fuel: process::Tank { chemical: FISSILE.into(), amount: 1000 },
            water: process::Tank { chemical: WATER.into(), amount: 100_000 },
            ..Reactor::default()
        }
        .tap(|r| r.rate_limit = r.rate_limit.min(max_burn(f)))
    }

    trait Tap: Sized {
        fn tap(mut self, f: impl FnOnce(&mut Self)) -> Self {
            f(&mut self);
            self
        }
    }
    impl Tap for Reactor {}

    #[test]
    fn burning_fuel_makes_heat_waste_and_steam() {
        let f = validate(&reactor(5, 2), [0; 3], [4; 3], |_| true);
        let mut r = running(&f);
        for _ in 0..400 {
            tick(&mut r, &f, 300.0, 0.99);
        }
        assert!(r.fuel.amount < 1000 && r.waste.amount > 0, "{r:?}");
        assert_eq!(r.waste.chemical, WASTE);
        assert!(r.heated.amount > 0 && r.heated.chemical == STEAM && r.water.amount < 100_000);
        assert!(r.temperature > 300.0);
        // Off, it burns nothing.
        let mut off = running(&f);
        off.active = false;
        tick(&mut off, &f, 300.0, 0.99);
        assert_eq!(off.last_burn, 0.0);
        assert_eq!(off.fuel.amount, 1000);
    }

    #[test]
    fn sodium_becomes_superheated_sodium() {
        let f = validate(&reactor(5, 2), [0; 3], [4; 3], |_| true);
        let mut r = running(&f);
        r.water = process::Tank::default();
        r.coolant = process::Tank { chemical: SODIUM.into(), amount: 100_000 };
        for _ in 0..400 {
            tick(&mut r, &f, 300.0, 0.99);
        }
        assert_eq!(r.heated.chemical, HOT_SODIUM);
        assert!(r.heated.amount > 0);
    }

    #[test]
    fn heat_damages_and_cold_repairs() {
        let f = validate(&reactor(5, 2), [0; 3], [4; 3], |_| true);
        let mut hot = Reactor { temperature: 1500.0, ..Reactor::default() };
        tick(&mut hot, &f, 300.0, 0.99);
        assert!(hot.damage > 0.0);
        let mut cool = Reactor { temperature: 300.0, damage: 10.0, ..Reactor::default() };
        tick(&mut cool, &f, 300.0, 0.99);
        assert!(cool.damage < 10.0);
        // A destroyed and hot reactor melts down when the roll is low.
        let mut doomed = Reactor { temperature: 1500.0, damage: 200.0, ..Reactor::default() };
        assert!(tick(&mut doomed, &f, 300.0, 0.0));
        assert!(!tick(&mut Reactor { temperature: 1500.0, damage: 200.0, ..Reactor::default() }, &f, 300.0, 0.999));
        after_meltdown(&mut doomed, 300.0);
        assert_eq!((doomed.active, doomed.damage), (false, POST_MELTDOWN_DAMAGE));
    }

    #[test]
    fn the_buttons_work_within_the_limits() {
        let f = validate(&reactor(5, 2), [0; 3], [4; 3], |_| true);
        let mut r = Reactor { rate_limit: 0.1, ..Reactor::default() };
        press(&mut r, &f, TOGGLE);
        assert!(r.active);
        press(&mut r, &f, RATE_UP_1);
        assert!((r.rate_limit - 1.1).abs() < 1e-9);
        press(&mut r, &f, RATE_UP_1);
        press(&mut r, &f, RATE_UP_1);
        assert!((r.rate_limit - 2.0).abs() < 1e-9);
        press(&mut r, &f, RATE_DOWN_1);
        press(&mut r, &f, RATE_DOWN_1);
        press(&mut r, &f, RATE_DOWN_TENTH);
        assert_eq!(r.rate_limit, 0.0);
        r.force_disable = true;
        press(&mut r, &f, TOGGLE);
        press(&mut r, &f, TOGGLE);
        assert!(!r.active);
    }

    #[test]
    fn a_member_remembers_its_reactor() {
        let f = Found { valid: true, min: [-3, 64, 7], size: [5, 6, 7], volume: 90, blocks: 120, assemblies: 4, surface: 20 };
        let r = Reactor { temperature: 900.0, damage: 12.5, burn_remaining: 0.25, partial_waste: 0.5, rate_limit: 1.5, active: true, force_disable: false, ..Reactor::default() };
        assert_eq!(decode(&encode(&f, &r)), Some((f, r)));
    }

    #[test]
    fn the_roll_is_in_range() {
        for t in 0..1000 {
            let v = roll(t, 7);
            assert!((0.0..1.0).contains(&v));
        }
    }
}

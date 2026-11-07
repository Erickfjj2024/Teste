//! The fusion reactor (`FusionReactorMultiblockData`, `FusionReactorValidator`): a 5 by 5 by 5 cube whose faces are a rounded shape (twelve
//! missing blocks on each), of reactor frames, with ports, a logic adapter and laser focus matrices where the shape allows, and one controller
//! in the middle of its top face; the 3 by 3 by 3 inside is empty. Deuterium and tritium are injected as fuel; a laser heats the plasma, and
//! above 100 million kelvin a hohlraum starts the burn, which heats the case, which boils water into steam and gives passive energy.
//!
//! The ports hold mirrors of the five tanks (water, deuterium, tritium, fuel in; steam out) and give the energy; the controller holds the
//! hohlraum slot and the screen. Unlike the original, a hohlraum counts as full (the port has no chemical items).
use super::*;
use std::collections::HashMap;

pub const FRAME: &str = "mekanism:fusion_reactor_frame";
pub const PORT: &str = "mekanism:fusion_reactor_port";
pub const ADAPTER: &str = "mekanism:fusion_reactor_logic_adapter";
pub const CONTROLLER: &str = "mekanism:fusion_reactor_controller";
pub const MATRIX: &str = "mekanism:laser_focus_matrix";
pub const GLASS: &str = "mekanism:reactor_glass";
pub const HOHLRAUM: &str = "mekanism:hohlraum";
pub const GROUP: &str = "mekanism:fusion";
pub const WATER: &str = "minecraft:water";
pub const STEAM: &str = "mekanism:steam";
pub const DEUTERIUM: &str = "mekanism:deuterium";
pub const TRITIUM: &str = "mekanism:tritium";
pub const FUEL: &str = "mekanism:fusion_fuel";
pub const NOTHING: &str = "mekanism:nothing";
pub const SPEC_CAPACITY: u64 = 1 << 50;
/// The constants of the original and of its default config.
pub const BURN_TEMPERATURE: f64 = 100_000_000.0;
pub const PLASMA_HEAT_CAPACITY: f64 = 100.0;
pub const CASE_HEAT_CAPACITY: f64 = 1.0;
pub const PLASMA_CASE_CONDUCTIVITY: f64 = 0.2;
pub const ENERGY_PER_FUEL: f64 = 10_000_000.0;
pub const THERMOCOUPLE: f64 = 0.05;
pub const CASING_CONDUCTIVITY: f64 = 0.1;
pub const WATER_HEATING: f64 = 0.3;
pub const FUEL_CAPACITY: u64 = 1_000;
pub const ENERGY_CAPACITY: u64 = 1_000_000_000;
pub const WATER_PER_INJECTION: u64 = 1_000_000;
pub const STEAM_PER_INJECTION: u64 = 100_000_000;
pub const HOHLRAUM_FUEL: u64 = 10;
pub const DEFAULT_INJECTION: u64 = 2;
pub const MAX_INJECTION: u64 = 98;
/// The buttons of the screen.
pub const INJECTION_DOWN: i32 = 7200;
pub const INJECTION_UP: i32 = 7201;

/// `ALLOWED_GRID`: 0 nothing needed, 1 a frame, 2 any block of the reactor.
const GRID: [[u8; 5]; 5] = [[0, 0, 1, 0, 0], [0, 1, 2, 1, 0], [1, 2, 2, 2, 1], [0, 1, 2, 1, 0], [0, 0, 1, 0, 0]];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Frame,
    Port,
    Adapter,
    Controller,
    Matrix,
    Glass,
}

pub fn part_of(state: &str) -> Option<Part> {
    match state.split('[').next().unwrap_or("") {
        FRAME => Some(Part::Frame),
        PORT => Some(Part::Port),
        ADAPTER => Some(Part::Adapter),
        CONTROLLER => Some(Part::Controller),
        MATRIX => Some(Part::Matrix),
        GLASS => Some(Part::Glass),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Found {
    pub valid: bool,
    pub min: [i32; 3],
}

/// What a position of the shell needs, from its coordinates inside the 5x5x5 box (0 to 4 each), or `None` inside.
pub fn requirement(rel: [usize; 3]) -> Option<u8> {
    let boundary: Vec<usize> = (0..3).filter(|&i| rel[i] == 0 || rel[i] == 4).collect();
    let Some(&axis) = boundary.first() else { return None };
    let others: Vec<usize> = (0..3).filter(|&i| i != axis).map(|i| rel[i]).collect();
    Some(GRID[others[0]][others[1]])
}

pub fn validate(members: &[([i32; 3], Part)], min: [i32; 3], max: [i32; 3], inside_empty: impl Fn([i32; 3]) -> bool) -> Found {
    let bad = Found { valid: false, min };
    if (0..3).any(|i| max[i] - min[i] != 4) {
        return bad;
    }
    let have: HashMap<[i32; 3], Part> = members.iter().copied().collect();
    if have.len() != members.len() {
        return bad;
    }
    let mut controllers = 0;
    for x in 0..5 {
        for y in 0..5 {
            for z in 0..5 {
                let p = [min[0] + x as i32, min[1] + y as i32, min[2] + z as i32];
                let part = have.get(&p).copied();
                match requirement([x, y, z]) {
                    None => {
                        if part.is_some() || !inside_empty(p) {
                            return bad;
                        }
                    }
                    Some(0) => {
                        // Nothing is needed here; what is here is not checked, but the controller may not hide here.
                        if part == Some(Part::Controller) {
                            return bad;
                        }
                    }
                    Some(need) => {
                        let is_controller_spot = y == 4 && x == 2 && z == 2;
                        match part {
                            None => return bad,
                            Some(Part::Controller) => {
                                controllers += 1;
                                if !is_controller_spot {
                                    return bad;
                                }
                            }
                            Some(Part::Frame | Part::Glass) => {
                                if is_controller_spot {
                                    return bad;
                                }
                            }
                            Some(Part::Port | Part::Adapter | Part::Matrix) => {
                                // Not on the frame's places, and not in the controller's spot.
                                if need != 2 || is_controller_spot {
                                    return bad;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if controllers != 1 || members.iter().any(|(p, _)| (0..3).any(|i| p[i] < min[i] || p[i] > max[i])) {
        return bad;
    }
    Found { valid: true, min }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Reactor {
    pub plasma: f64,
    pub case: f64,
    pub burning: bool,
    pub injection: u64,
    pub deuterium: process::Tank,
    pub tritium: process::Tank,
    pub fuel: process::Tank,
    pub water: process::Tank,
    pub steam: process::Tank,
    pub energy: u64,
    pub last_burned: u64,
    pub last_power: u64,
}

pub fn encode(f: &Found, r: &Reactor) -> Vec<u8> {
    let mut out = vec![u8::from(f.valid)];
    for c in f.min {
        out.extend(c.to_le_bytes());
    }
    out.extend((r.plasma as f32).to_le_bytes());
    out.extend((r.case as f32).to_le_bytes());
    out.push(r.injection as u8);
    out.push(u8::from(r.burning));
    out.extend(r.energy.to_le_bytes());
    out
}

pub fn decode(data: &[u8]) -> Option<(Found, Reactor)> {
    if data.len() < 31 {
        return None;
    }
    let int = |at: usize| i32::from_le_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]);
    let float = |at: usize| f64::from(f32::from_le_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]));
    let mut e = [0u8; 8];
    e.copy_from_slice(&data[23..31]);
    Some((
        Found { valid: data[0] == 1, min: [int(1), int(5), int(9)] },
        Reactor { plasma: float(13), case: float(17), injection: u64::from(data[21]), burning: data[22] == 1, energy: u64::from_le_bytes(e), ..Reactor::default() },
    ))
}

pub fn water_capacity(injection: u64) -> u64 {
    injection * WATER_PER_INJECTION
}

pub fn steam_capacity(injection: u64) -> u64 {
    injection * STEAM_PER_INJECTION
}

/// `addTemperatureFromEnergyInput`: a laser's energy heats the plasma (ten times as much while it does not burn).
pub fn add_laser_energy(r: &mut Reactor, energy: u64) {
    if energy > 0 {
        let heat = energy as f64 / PLASMA_HEAT_CAPACITY;
        r.plasma += if r.burning { heat } else { heat * 10.0 };
    }
}

/// One tick: ignition, injection, the burn and the heat. `hohlraum` says a hohlraum waits in the slot; returns whether it was used.
pub fn tick(r: &mut Reactor, ambient: f64, hohlraum: bool) -> bool {
    let mut used = false;
    let mut burned = 0u64;
    if r.plasma >= BURN_TEMPERATURE {
        if !r.burning && hohlraum {
            // The hohlraum gives its fuel to the tank, as much as fits.
            let room = FUEL_CAPACITY.saturating_sub(r.fuel.amount);
            if room == 0 || r.fuel.amount == 0 || r.fuel.chemical == FUEL {
                let give = HOHLRAUM_FUEL.min(room);
                if give > 0 {
                    r.fuel.chemical = FUEL.to_owned();
                    r.fuel.amount += give;
                }
                r.burning = true;
                used = true;
            }
        }
        if r.burning {
            // Inject: deuterium and tritium in halves.
            let needed = FUEL_CAPACITY.saturating_sub(r.fuel.amount);
            let available = 2 * r.deuterium.amount.min(r.tritium.amount);
            let mut inject = needed.min(available).min(r.injection);
            inject -= inject % 2;
            let half = inject / 2;
            if half > 0 && (r.fuel.amount == 0 || r.fuel.chemical == FUEL) {
                r.deuterium.amount -= half;
                r.tritium.amount -= half;
                if r.deuterium.amount == 0 {
                    r.deuterium.chemical.clear();
                }
                if r.tritium.amount == 0 {
                    r.tritium.chemical.clear();
                }
                r.fuel.chemical = FUEL.to_owned();
                r.fuel.amount += inject;
            }
            // Burn.
            if r.fuel.amount > 0 {
                let burn = (((r.plasma - BURN_TEMPERATURE) * 1.0).max(0.0) as u64).min(r.fuel.amount);
                r.fuel.amount -= burn;
                if r.fuel.amount == 0 {
                    r.fuel.chemical.clear();
                }
                r.plasma += ENERGY_PER_FUEL * burn as f64 / PLASMA_HEAT_CAPACITY;
                burned = burn;
            }
        }
        if burned == 0 {
            r.burning = false;
        }
    } else {
        r.burning = false;
    }
    r.last_burned = burned;
    // Plasma to case.
    let plasma_case = PLASMA_CASE_CONDUCTIVITY * (r.plasma - r.case);
    if plasma_case.abs() > 1e-9 {
        r.plasma -= plasma_case / PLASMA_HEAT_CAPACITY;
        r.case += plasma_case / CASE_HEAT_CAPACITY;
    }
    // Case to water.
    let case_water = WATER_HEATING * (r.case - ambient);
    if r.water.amount > 0 && case_water.abs() > 1e-9 {
        let to_vaporize = (0.2 * case_water / 10.0) as i64;
        let room = steam_capacity(r.injection).saturating_sub(r.steam.amount);
        let vaporized = (to_vaporize.max(0) as u64).min(room).min(r.water.amount);
        if vaporized > 0 {
            r.water.amount -= vaporized;
            if r.water.amount == 0 {
                r.water.chemical.clear();
            }
            r.steam.chemical = STEAM.to_owned();
            r.steam.amount += vaporized;
            r.case -= vaporized as f64 * 10.0 / 0.2 / CASE_HEAT_CAPACITY;
        }
    }
    // The air, and the passive energy.
    let case_air = CASING_CONDUCTIVITY * (r.case - ambient);
    r.last_power = 0;
    if case_air.abs() > 1e-9 {
        r.case -= case_air / CASE_HEAT_CAPACITY;
        let power = (case_air * THERMOCOUPLE).clamp(0.0, f64::from(i32::MAX)) as u64;
        if power > 0 {
            let added = power.min(ENERGY_CAPACITY.saturating_sub(r.energy));
            r.energy += added;
            r.last_power = added;
        }
    }
    used
}

/// The most the injection rate lets the plasma and the case reach (`getMaxPlasmaTemperature`, `getMaxCasingTemperature`), water cooled or not.
pub fn max_plasma(injection: u64, water: bool) -> f64 {
    let k = if water { WATER_HEATING } else { 0.0 };
    injection as f64 * ENERGY_PER_FUEL / PLASMA_CASE_CONDUCTIVITY * (PLASMA_CASE_CONDUCTIVITY + k + CASING_CONDUCTIVITY) / (k + CASING_CONDUCTIVITY)
}

pub fn press(r: &mut Reactor, id: i32) {
    let step = match id {
        INJECTION_UP => 2,
        INJECTION_DOWN => -2,
        _ => return,
    };
    r.injection = (r.injection as i64 + step).clamp(0, MAX_INJECTION as i64) as u64;
    r.water.amount = r.water.amount.min(water_capacity(r.injection));
    r.steam.amount = r.steam.amount.min(steam_capacity(r.injection));
}

#[derive(Default)]
pub struct Shared {
    pub reactor: Reactor,
    pub started: bool,
    pub given: HashMap<[i32; 3], [u64; 6]>,
}

impl Mekanism {
    pub(crate) fn fusion_structures(&mut self, dimension: &str, structures: Vec<Structure>) -> Vec<BlockEntityUpdate> {
        let mut out = Vec::new();
        for s in structures {
            let members: Vec<([i32; 3], Part)> = s.members.iter().filter_map(|m| part_of(&m.state).map(|p| ([m.pos.x, m.pos.y, m.pos.z], p))).collect();
            let min = [s.min.x, s.min.y, s.min.z];
            let max = [s.max.x, s.max.y, s.max.z];
            let found = validate(&members, min, max, |p| {
                world::get_block(dimension, p[0], p[1], p[2]).is_none_or(|b| matches!(b.split('[').next().unwrap_or(""), "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"))
            });
            let keep = self.fusions.get(&(dimension.to_owned(), found.min)).map(|s| s.reactor.clone()).unwrap_or(Reactor { injection: DEFAULT_INJECTION, ..Reactor::default() });
            for m in &s.members {
                out.push(BlockEntityUpdate {
                    dimension: dimension.to_owned(),
                    pos: m.pos,
                    data: encode(&found, &keep),
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

    /// A tick of the ports and controllers of the reactors.
    pub(crate) fn run_fusion(&mut self, parts: &[&BlockEntity], tick_no: u64) -> Vec<BlockEntityUpdate> {
        let presses = std::mem::take(&mut self.fusion_buttons);
        let mut by: HashMap<(String, [i32; 3]), Vec<&BlockEntity>> = HashMap::new();
        for p in parts {
            if let Some((f, _)) = decode(&p.data).filter(|(f, _)| f.valid) {
                by.entry((p.dimension.clone(), f.min)).or_default().push(*p);
            }
        }
        let mut out = Vec::new();
        for (key, mut group) in by {
            group.sort_by_key(|e| (e.pos.x, e.pos.y, e.pos.z));
            let Some((_, saved)) = decode(&group[0].data) else { continue };
            let ports: Vec<&&BlockEntity> = group.iter().filter(|e| e.state.starts_with(PORT)).collect();
            let controller = group.iter().find(|e| e.state.starts_with(CONTROLLER)).copied();
            let s = self.fusions.entry(key.clone()).or_default();
            let held = |p: &BlockEntity, i: usize| p.tanks.get(i).cloned().unwrap_or(Tank { substance: String::new(), amount: 0 });
            if !s.started {
                s.started = true;
                s.reactor = Reactor { injection: if saved.injection > 0 { saved.injection } else { DEFAULT_INJECTION }, ..saved };
                for i in 0..5 {
                    if let Some(best) = ports.iter().map(|p| held(p, i)).max_by_key(|t| t.amount).filter(|t| t.amount > 0) {
                        let t = process::Tank { chemical: best.substance, amount: best.amount };
                        match i {
                            0 => s.reactor.water = t,
                            1 => s.reactor.deuterium = t,
                            2 => s.reactor.tritium = t,
                            3 => s.reactor.fuel = t,
                            _ => s.reactor.steam = t,
                        }
                    }
                }
            } else {
                let caps = [water_capacity(s.reactor.injection), FUEL_CAPACITY, FUEL_CAPACITY, FUEL_CAPACITY, steam_capacity(s.reactor.injection)];
                for i in 0..5 {
                    let (given, now): (Vec<u64>, Vec<u64>) =
                        ports.iter().map(|p| (s.given.get(&[p.pos.x, p.pos.y, p.pos.z]).map_or(0, |g| g[i]), held(p, i).amount)).unzip();
                    let tank = match i {
                        0 => &mut s.reactor.water,
                        1 => &mut s.reactor.deuterium,
                        2 => &mut s.reactor.tritium,
                        3 => &mut s.reactor.fuel,
                        _ => &mut s.reactor.steam,
                    };
                    let next = dynamic::merge(tank.amount, &given, &now, caps[i]);
                    if tank.amount == 0 && next > 0 {
                        tank.chemical = ports.iter().map(|p| held(p, i)).find(|t| t.amount > 0).map(|t| t.substance).unwrap_or_default();
                    }
                    tank.amount = next;
                    if next == 0 {
                        tank.chemical.clear();
                    }
                }
                // The energy the ports gave away.
                let taken: u64 = ports.iter().map(|p| s.given.get(&[p.pos.x, p.pos.y, p.pos.z]).map_or(0, |g| g[5]).saturating_sub(p.energy)).sum();
                s.reactor.energy = s.reactor.energy.saturating_sub(taken);
            }
            for (_, pos, id) in presses.iter().filter(|(d, _, _)| *d == key.0) {
                if group.iter().any(|g| (g.pos.x, g.pos.y, g.pos.z) == (pos.x, pos.y, pos.z)) {
                    press(&mut s.reactor, *id);
                }
            }
            let ambient = world::get_environment(&group[0].dimension, group[0].pos.x, group[0].pos.y, group[0].pos.z).map_or(300.0, |e| generators::ambient_of(e.temperature));
            if s.reactor.plasma < 1.0 {
                s.reactor.plasma = ambient;
                s.reactor.case = ambient;
            }
            let slot = controller.and_then(|c| c.items.first().and_then(|x| x.as_ref())).filter(|x| x.item == HOHLRAUM).cloned();
            let used = tick(&mut s.reactor, ambient, slot.is_some());
            let r = s.reactor.clone();
            let caps = [water_capacity(r.injection), FUEL_CAPACITY, FUEL_CAPACITY, FUEL_CAPACITY, steam_capacity(r.injection)];
            let tanks_now = [r.water.clone(), r.deuterium.clone(), r.tritium.clone(), r.fuel.clone(), r.steam.clone()];
            let n = ports.len().max(1) as u64;
            let mut index = 0u64;
            let half = |v: u64, sh: u32| ((v >> sh) & 0xFFFF) as i32;
            let kind = |t: &process::Tank, fallback: &str| chem::index(if t.amount == 0 { fallback } else { &t.chemical });
            let data = vec![
                half(r.deuterium.amount, 0),
                half(r.deuterium.amount, 16),
                half(r.tritium.amount, 0),
                half(r.tritium.amount, 16),
                half(r.fuel.amount, 0),
                half(r.fuel.amount, 16),
                half(r.water.amount, 0),
                half(r.water.amount, 16),
                half(r.steam.amount, 0),
                half(r.steam.amount, 16),
                FUEL_CAPACITY as i32,
                0,
                half(caps[0], 0),
                half(caps[0], 16),
                half(caps[4], 0),
                half(caps[4], 16),
                chem::index(DEUTERIUM),
                chem::index(TRITIUM),
                chem::index(FUEL),
                chem::index(WATER),
                chem::index(STEAM),
                (r.plasma / 1000.0).min(f64::from(i32::MAX)) as i32,
                (r.case * 10.0).min(f64::from(i32::MAX)) as i32,
                r.injection as i32,
                i32::from(r.burning),
                half(r.energy, 0),
                half(r.energy, 16),
                r.last_burned.min(i32::MAX as u64) as i32,
                r.last_power.min(i32::MAX as u64) as i32,
            ];
            let _ = kind;
            for e in &group {
                let p = [e.pos.x, e.pos.y, e.pos.z];
                let keep = if tick_no % 20 == 0 { encode(&Found { valid: true, min: key.1 }, &r) } else { e.data.clone() };
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
                if e.state.starts_with(PORT) {
                    let share = r.energy / n + u64::from(index < r.energy % n);
                    index += 1;
                    update.energy = Some(share);
                    update.tanks = Some(tanks_now.iter().map(plus::as_tank).collect());
                    let ok = |tank: &process::Tank, cap: u64, name: &str| if tank.amount >= cap { vec![NOTHING.to_owned()] } else { vec![name.to_owned()] };
                    let _ = world::set_tank_filter(&e.dimension, p[0], p[1], p[2], 0, &ok(&r.water, caps[0], WATER));
                    let _ = world::set_tank_filter(&e.dimension, p[0], p[1], p[2], 1, &ok(&r.deuterium, caps[1], DEUTERIUM));
                    let _ = world::set_tank_filter(&e.dimension, p[0], p[1], p[2], 2, &ok(&r.tritium, caps[2], TRITIUM));
                    let _ = world::set_tank_filter(&e.dimension, p[0], p[1], p[2], 3, &ok(&r.fuel, caps[3], FUEL));
                    if let Some(sh) = self.fusions.get_mut(&key) {
                        sh.given.insert(p, [tanks_now[0].amount, tanks_now[1].amount, tanks_now[2].amount, tanks_now[3].amount, tanks_now[4].amount, share]);
                    }
                } else if e.state.starts_with(CONTROLLER) {
                    update.menu_data = Some(data.clone());
                    if used {
                        update.items = Some(vec![None]);
                    }
                }
                out.push(update);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A reactor: frames everywhere the grid asks for one, other blocks where it allows any (a port in a corner of the plus), the controller on top.
    fn reactor() -> Vec<([i32; 3], Part)> {
        let mut out = Vec::new();
        for x in 0..5usize {
            for y in 0..5usize {
                for z in 0..5usize {
                    let part = match requirement([x, y, z]) {
                        None | Some(0) => continue,
                        Some(_) if (x, y, z) == (2, 4, 2) => Part::Controller,
                        Some(1) => Part::Frame,
                        Some(_) if (x, y, z) == (0, 2, 1) => Part::Port,
                        Some(_) => Part::Glass,
                    };
                    out.push(([x as i32, y as i32, z as i32], part));
                }
            }
        }
        out
    }

    #[test]
    fn the_rounded_cube_has_twelve_missing_blocks_a_face() {
        let missing = (0..5usize).flat_map(|a| (0..5usize).map(move |b| (a, b))).filter(|(a, b)| GRID[*a][*b] == 0).count();
        assert_eq!(missing, 12);
        assert_eq!(requirement([2, 2, 2]), None);
    }

    #[test]
    fn a_whole_shell_with_a_controller_is_a_reactor() {
        assert!(validate(&reactor(), [0; 3], [4; 3], |_| true).valid);
    }

    #[test]
    fn it_needs_the_shape_the_controller_and_an_empty_inside() {
        let check = |m: &[([i32; 3], Part)]| validate(m, [0; 3], [4; 3], |_| true).valid;
        let mut none = reactor();
        none.retain(|(_, p)| *p != Part::Controller);
        assert!(!check(&none));
        let mut moved = reactor();
        for (p, part) in &mut moved {
            if *part == Part::Controller {
                *part = Part::Glass;
            }
            if *p == [2, 2, 4] {
                *part = Part::Controller;
            }
        }
        assert!(!check(&moved));
        let mut hole = reactor();
        hole.retain(|(p, _)| *p != [2, 0, 2]);
        assert!(!check(&hole));
        let mut inside = reactor();
        inside.push(([2, 2, 2], Part::Frame));
        assert!(!check(&inside));
        assert!(!validate(&reactor(), [0; 3], [4; 3], |_| false).valid);
        assert!(!validate(&reactor(), [0; 3], [5, 4, 4], |_| true).valid);
        // A port where the grid wants a frame.
        let mut port = reactor();
        for (p, part) in &mut port {
            if *p == [2, 0, 0] {
                *part = Part::Port;
            }
        }
        assert!(!check(&port));
    }

    fn hot(burning: bool) -> Reactor {
        Reactor {
            plasma: 1.2e8,
            case: 300.0,
            injection: 2,
            burning,
            fuel: process::Tank { chemical: FUEL.into(), amount: 100 },
            deuterium: process::Tank { chemical: DEUTERIUM.into(), amount: 500 },
            tritium: process::Tank { chemical: TRITIUM.into(), amount: 500 },
            water: process::Tank { chemical: WATER.into(), amount: 100_000 },
            ..Reactor::default()
        }
    }

    #[test]
    fn a_hohlraum_ignites_a_hot_plasma_and_it_keeps_burning() {
        let mut cold = hot(false);
        cold.plasma = 5e7;
        assert!(!tick(&mut cold, 300.0, true));
        assert!(!cold.burning);
        let mut r = hot(false);
        assert!(tick(&mut r, 300.0, true));
        assert!(r.burning && r.last_burned > 0);
        // Burning raises the plasma and heats the case, which boils water and makes energy.
        for _ in 0..200 {
            tick(&mut r, 300.0, false);
        }
        assert!(r.case > 300.0 && r.steam.amount > 0 && r.energy > 0, "{r:?}");
        // Without fuel the fire dies.
        let mut dry = hot(true);
        dry.fuel = process::Tank::default();
        dry.deuterium = process::Tank::default();
        tick(&mut dry, 300.0, false);
        assert!(!dry.burning);
    }

    #[test]
    fn the_laser_heats_ten_times_more_before_the_burn() {
        let mut r = Reactor::default();
        add_laser_energy(&mut r, 1000);
        assert_eq!(r.plasma, 100.0);
        r.burning = true;
        add_laser_energy(&mut r, 1000);
        assert_eq!(r.plasma, 110.0);
    }

    #[test]
    fn the_injection_rate_moves_by_two() {
        let mut r = Reactor { injection: 2, ..Reactor::default() };
        press(&mut r, INJECTION_UP);
        assert_eq!(r.injection, 4);
        press(&mut r, INJECTION_DOWN);
        press(&mut r, INJECTION_DOWN);
        press(&mut r, INJECTION_DOWN);
        assert_eq!(r.injection, 0);
        r.injection = 98;
        press(&mut r, INJECTION_UP);
        assert_eq!(r.injection, 98);
        assert!(max_plasma(2, true) > 0.0);
    }

    #[test]
    fn a_member_remembers_its_reactor() {
        let f = Found { valid: true, min: [-3, 64, 9] };
        let r = Reactor { plasma: 1.5e8, case: 900.0, injection: 8, burning: true, energy: 123_456_789, ..Reactor::default() };
        let (g, back) = decode(&encode(&f, &r)).unwrap();
        assert_eq!((g, back.injection, back.burning, back.energy), (f, 8, true, 123_456_789));
        assert!((back.plasma - 1.5e8).abs() < 100.0);
    }
}

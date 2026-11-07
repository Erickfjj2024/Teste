//! The Supercritical Phase Shifter (`SPSMultiblockData`, `SPSValidator`): a 7 by 7 by 7 cube of SPS casing, rounded like the fusion reactor's
//! (twelve missing blocks on each face), with SPS ports where the shape allows. Inside, only supercharged coils, each one right behind a port.
//! Ports give the energy they get to the coils behind them; the energy turns polonium into antimatter: a million FE for each mB of polonium,
//! a thousand mB of polonium for each mB of antimatter.
//!
//! The ports hold mirrors of the polonium and antimatter tanks, as the boiler's valves do. Unlike the original, a port has no input/output mode
//! (the original toggles it by sneak-clicking): every port takes polonium and energy in and gives antimatter out.
use super::*;
use std::collections::{HashMap, HashSet};

pub const CASING: &str = "mekanism:sps_casing";
pub const PORT: &str = "mekanism:sps_port";
pub const COIL: &str = "mekanism:supercharged_coil";
pub const GROUP: &str = "mekanism:sps";
pub const POLONIUM: &str = "mekanism:polonium";
pub const ANTIMATTER: &str = "mekanism:antimatter";
pub const NOTHING: &str = "mekanism:nothing";
pub const SPEC_CAPACITY: u64 = 1 << 50;
/// `inputPerAntimatter`, `outputTankCapacity`, `energyPerInput` and the port's energy storage.
pub const INPUT_PER_ANTIMATTER: u64 = 1_000;
pub const OUTPUT_CAPACITY: u64 = 1_000;
pub const ENERGY_PER_INPUT: u64 = 1_000_000;
pub const PORT_ENERGY: u64 = 1_000_000_000;

const GRID: [[u8; 7]; 7] = [
    [0, 0, 1, 1, 1, 0, 0],
    [0, 1, 2, 2, 2, 1, 0],
    [1, 2, 2, 2, 2, 2, 1],
    [1, 2, 2, 2, 2, 2, 1],
    [1, 2, 2, 2, 2, 2, 1],
    [0, 1, 2, 2, 2, 1, 0],
    [0, 0, 1, 1, 1, 0, 0],
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Casing,
    Port,
    Coil,
}

pub fn part_of(state: &str) -> Option<Part> {
    match state.split('[').next().unwrap_or("") {
        CASING => Some(Part::Casing),
        PORT => Some(Part::Port),
        COIL => Some(Part::Coil),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Found {
    pub valid: bool,
    pub min: [i32; 3],
    pub coils: u32,
}

/// What a shell position needs (0 nothing, 1 casing, 2 casing or port); none inside.
pub fn requirement(rel: [usize; 3]) -> Option<u8> {
    let axis = (0..3).find(|&i| rel[i] == 0 || rel[i] == 6)?;
    let others: Vec<usize> = (0..3).filter(|&i| i != axis).map(|i| rel[i]).collect();
    Some(GRID[others[0]][others[1]])
}

/// The inward neighbour of a position on a face of the box (the coil's place behind a port); none on an edge.
pub fn behind(rel: [usize; 3]) -> Option<[usize; 3]> {
    let on: Vec<usize> = (0..3).filter(|&i| rel[i] == 0 || rel[i] == 6).collect();
    if on.len() != 1 {
        return None;
    }
    let mut p = rel;
    p[on[0]] = if rel[on[0]] == 0 { 1 } else { 5 };
    Some(p)
}

pub fn validate(members: &[([i32; 3], Part)], min: [i32; 3], max: [i32; 3], inside_empty: impl Fn([i32; 3]) -> bool) -> Found {
    let bad = Found { valid: false, min, coils: 0 };
    if (0..3).any(|i| max[i] - min[i] != 6) {
        return bad;
    }
    let have: HashMap<[i32; 3], Part> = members.iter().copied().collect();
    if have.len() != members.len() || members.iter().any(|(p, _)| (0..3).any(|i| p[i] < min[i] || p[i] > max[i])) {
        return bad;
    }
    let at = |rel: [usize; 3]| [min[0] + rel[0] as i32, min[1] + rel[1] as i32, min[2] + rel[2] as i32];
    let mut behind_ports = HashSet::new();
    let mut coils = 0;
    for x in 0..7usize {
        for y in 0..7usize {
            for z in 0..7usize {
                let rel = [x, y, z];
                let p = at(rel);
                let part = have.get(&p).copied();
                match requirement(rel) {
                    None => match part {
                        Some(Part::Coil) => coils += 1,
                        Some(_) => return bad,
                        None if !inside_empty(p) => return bad,
                        None => {}
                    },
                    Some(0) => {}
                    Some(1) => {
                        if part != Some(Part::Casing) {
                            return bad;
                        }
                    }
                    Some(_) => match part {
                        Some(Part::Casing) => {}
                        Some(Part::Port) => {
                            if let Some(b) = behind(rel) {
                                behind_ports.insert(at(b));
                            }
                        }
                        _ => return bad,
                    },
                }
            }
        }
    }
    // Every coil sits right behind a port.
    if members.iter().filter(|(_, p)| *p == Part::Coil).any(|(p, _)| !behind_ports.contains(p)) {
        return bad;
    }
    Found { valid: true, min, coils }
}

/// The state of an SPS: the tanks, the progress toward the next mB of antimatter and the energy it got.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Sps {
    pub input: process::Tank,
    pub output: process::Tank,
    pub progress: f64,
    pub input_processed: u64,
    pub last_received: u64,
    pub last_processed: f64,
    pub could_operate: bool,
}

pub fn input_capacity() -> u64 {
    INPUT_PER_ANTIMATTER * 2
}

pub fn encode(f: &Found, s: &Sps) -> Vec<u8> {
    let mut out = vec![u8::from(f.valid)];
    for c in f.min {
        out.extend(c.to_le_bytes());
    }
    out.extend(f.coils.to_le_bytes());
    out.extend((s.progress as f32).to_le_bytes());
    out.extend((s.input_processed as u32).to_le_bytes());
    out
}

pub fn decode(data: &[u8]) -> Option<(Found, Sps)> {
    if data.len() < 25 {
        return None;
    }
    let int = |at: usize| i32::from_le_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]);
    Some((
        Found { valid: data[0] == 1, min: [int(1), int(5), int(9)], coils: int(13) as u32 },
        Sps {
            progress: f64::from(f32::from_le_bytes([data[17], data[18], data[19], data[20]])),
            input_processed: u64::from(int(21) as u32),
            ..Sps::default()
        },
    ))
}

/// `process`: spends `operations` mB of polonium; every thousand makes an mB of antimatter (all of it has to fit, or nothing is spent).
fn process(s: &mut Sps, operations: u64) -> u64 {
    if operations == 0 || s.input.amount == 0 {
        return 0;
    }
    let processed = operations.min(s.input.amount);
    let total = s.input_processed + processed;
    let mut left = total;
    if total >= INPUT_PER_ANTIMATTER {
        let add = total / INPUT_PER_ANTIMATTER;
        let room = OUTPUT_CAPACITY.saturating_sub(s.output.amount);
        if add > room || (s.output.amount > 0 && s.output.chemical != ANTIMATTER) {
            return 0;
        }
        s.output.chemical = ANTIMATTER.to_owned();
        s.output.amount += add;
        left = total % INPUT_PER_ANTIMATTER;
    }
    s.input_processed = left;
    s.input.amount -= processed;
    if s.input.amount == 0 {
        s.input.chemical.clear();
    }
    processed
}

/// One tick with `received` FE from the ports.
pub fn tick(s: &mut Sps, received: u64) {
    let mut processed = 0.0;
    s.could_operate = s.input.amount > 0 && s.output.amount < OUTPUT_CAPACITY;
    if s.could_operate && received > 0 {
        let needed_out = OUTPUT_CAPACITY - s.output.amount;
        let input_needed = (INPUT_PER_ANTIMATTER - s.input_processed) as f64 + (INPUT_PER_ANTIMATTER * (needed_out - 1)) as f64;
        let processable = received as f64 / ENERGY_PER_INPUT as f64;
        if processable + s.progress >= input_needed {
            processed = process(s, input_needed as u64) as f64;
            s.progress = 0.0;
        } else {
            processed = processable;
            s.progress += processable;
            let to_process = s.progress as u64;
            let actual = process(s, to_process);
            if actual < to_process {
                let diff = (to_process - actual) as f64;
                s.progress -= diff;
                processed -= diff;
            }
            s.progress %= 1.0;
        }
    }
    s.last_received = received;
    s.last_processed = processed;
}

#[derive(Default)]
pub struct Shared {
    pub sps: Sps,
    pub started: bool,
    pub given: HashMap<[i32; 3], [u64; 2]>,
}

impl Mekanism {
    pub(crate) fn sps_structures(&mut self, dimension: &str, structures: Vec<Structure>) -> Vec<BlockEntityUpdate> {
        let mut out = Vec::new();
        for s in structures {
            let members: Vec<([i32; 3], Part)> = s.members.iter().filter_map(|m| part_of(&m.state).map(|p| ([m.pos.x, m.pos.y, m.pos.z], p))).collect();
            let min = [s.min.x, s.min.y, s.min.z];
            let max = [s.max.x, s.max.y, s.max.z];
            let found = validate(&members, min, max, |p| {
                world::get_block(dimension, p[0], p[1], p[2]).is_none_or(|b| matches!(b.split('[').next().unwrap_or(""), "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"))
            });
            // Each coil looks at the port it is behind.
            if found.valid {
                let ports: HashSet<[i32; 3]> = members.iter().filter(|(_, p)| *p == Part::Port).map(|(p, _)| *p).collect();
                for (c, _) in members.iter().filter(|(_, p)| *p == Part::Coil) {
                    for face in 0..6 {
                        let d = laser::direction(face);
                        if ports.contains(&[c[0] + d[0], c[1] + d[1], c[2] + d[2]]) {
                            let _ = world::set_block_property(dimension, c[0], c[1], c[2], "facing", laser::facing_name(face));
                            break;
                        }
                    }
                }
            }
            let keep = self.spses.get(&(dimension.to_owned(), found.min)).map(|s| s.sps.clone()).unwrap_or_default();
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

    /// A tick of the ports of the SPSs.
    pub(crate) fn run_sps(&mut self, ports: &[&BlockEntity], tick_no: u64) -> Vec<BlockEntityUpdate> {
        let mut by: HashMap<(String, [i32; 3]), Vec<&BlockEntity>> = HashMap::new();
        for p in ports {
            if let Some((f, _)) = decode(&p.data).filter(|(f, _)| f.valid) {
                by.entry((p.dimension.clone(), f.min)).or_default().push(*p);
            }
        }
        let mut out = Vec::new();
        for (key, group) in by {
            let Some((found, saved)) = decode(&group[0].data) else { continue };
            let s = self.spses.entry(key.clone()).or_default();
            let held = |p: &BlockEntity, i: usize| p.tanks.get(i).cloned().unwrap_or(Tank { substance: String::new(), amount: 0 });
            let caps = [input_capacity(), OUTPUT_CAPACITY];
            if !s.started {
                s.started = true;
                s.sps = Sps { progress: saved.progress, input_processed: saved.input_processed, ..Sps::default() };
                for i in 0..2 {
                    if let Some(best) = group.iter().map(|p| held(p, i)).max_by_key(|t| t.amount).filter(|t| t.amount > 0) {
                        let t = process::Tank { chemical: best.substance, amount: best.amount.min(caps[i]) };
                        if i == 0 { s.sps.input = t } else { s.sps.output = t }
                    }
                }
            } else {
                for i in 0..2 {
                    let (given, now): (Vec<u64>, Vec<u64>) =
                        group.iter().map(|p| (s.given.get(&[p.pos.x, p.pos.y, p.pos.z]).map_or(0, |g| g[i]), held(p, i).amount)).unzip();
                    let tank = if i == 0 { &mut s.sps.input } else { &mut s.sps.output };
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
            if s.sps.input.amount > 0 && s.sps.input.chemical != POLONIUM {
                s.sps.input = process::Tank::default();
            }
            // The energy the ports got goes to the coils (when there are coils to take it).
            let received: u64 = if found.coils > 0 { group.iter().map(|p| p.energy).sum() } else { 0 };
            tick(&mut s.sps, received);
            let st = s.sps.clone();
            let tanks_now = [st.input.clone(), st.output.clone()];
            for p in &group {
                s.given.insert([p.pos.x, p.pos.y, p.pos.z], [tanks_now[0].amount, tanks_now[1].amount]);
            }
            let half = |v: u64, sh: u32| ((v >> sh) & 0xFFFF) as i32;
            let data = vec![
                half(st.input.amount, 0),
                half(st.input.amount, 16),
                half(caps[0], 0),
                half(caps[0], 16),
                half(st.output.amount, 0),
                half(st.output.amount, 16),
                half(caps[1], 0),
                half(caps[1], 16),
                chem::index(POLONIUM),
                chem::index(ANTIMATTER),
                (st.progress * 100.0) as i32,
                half(st.last_received, 0),
                half(st.last_received, 16),
                found.coils as i32,
                (st.last_processed * 100.0).min(f64::from(i32::MAX)) as i32,
            ];
            for p in &group {
                let (x, y, z) = (p.pos.x, p.pos.y, p.pos.z);
                let _ = world::set_tank_filter(&p.dimension, x, y, z, 0, &if st.input.amount >= caps[0] { vec![NOTHING.to_owned()] } else { vec![POLONIUM.to_owned()] });
                let keep = if tick_no % 20 == 0 { encode(&found, &st) } else { p.data.clone() };
                out.push(BlockEntityUpdate {
                    dimension: p.dimension.clone(),
                    pos: p.pos,
                    data: keep,
                    items: None,
                    // The energy was spent on the coils.
                    energy: Some(0),
                    energy_faces: None,
                    tanks: Some(tanks_now.iter().map(plus::as_tank).collect()),
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

    /// A full SPS: casing where the grid asks for one, one port in the middle of the top face and the coil behind it.
    fn sps() -> Vec<([i32; 3], Part)> {
        let mut out = Vec::new();
        for x in 0..7usize {
            for y in 0..7usize {
                for z in 0..7usize {
                    let part = match requirement([x, y, z]) {
                        None | Some(0) => continue,
                        Some(2) if (x, y, z) == (3, 6, 3) => Part::Port,
                        Some(_) => Part::Casing,
                    };
                    out.push(([x as i32, y as i32, z as i32], part));
                }
            }
        }
        out.push(([3, 5, 3], Part::Coil));
        out
    }

    #[test]
    fn the_shape_misses_twelve_blocks_a_face() {
        let missing = GRID.iter().flatten().filter(|v| **v == 0).count();
        assert_eq!(missing, 12);
        assert_eq!(behind([3, 6, 3]), Some([3, 5, 3]));
        assert_eq!(behind([0, 0, 3]), None);
    }

    #[test]
    fn a_shell_with_a_coil_behind_a_port_is_an_sps() {
        let f = validate(&sps(), [0; 3], [6; 3], |_| true);
        assert_eq!((f.valid, f.coils), (true, 1));
    }

    #[test]
    fn a_coil_has_to_be_behind_a_port() {
        let check = |m: &[([i32; 3], Part)]| validate(m, [0; 3], [6; 3], |_| true).valid;
        let mut stray = sps();
        stray.push(([2, 2, 2], Part::Coil));
        assert!(!check(&stray));
        let mut hole = sps();
        hole.retain(|(p, _)| *p != [3, 0, 3]);
        assert!(!check(&hole));
        let mut inside = sps();
        inside.push(([3, 3, 3], Part::Casing));
        assert!(!check(&inside));
        // A port where only casing goes (an edge of the shape).
        let mut port = sps();
        for (p, part) in &mut port {
            if *p == [3, 0, 0] {
                *part = Part::Port;
            }
        }
        assert!(!check(&port));
        assert!(!validate(&sps(), [0; 3], [6; 3], |_| false).valid);
        assert!(!validate(&sps(), [0; 3], [6, 6, 5], |_| true).valid);
    }

    #[test]
    fn energy_turns_polonium_into_antimatter() {
        let mut s = Sps { input: process::Tank { chemical: POLONIUM.into(), amount: 2000 }, ..Sps::default() };
        // A million FE a tick is a mB of polonium a tick: a thousand ticks for one of antimatter.
        for _ in 0..1000 {
            tick(&mut s, 1_000_000);
        }
        assert_eq!((s.output.amount, s.input.amount), (1, 1000));
        assert_eq!(s.output.chemical, ANTIMATTER);
        // A full output tank stops it; so does no polonium.
        let mut full = Sps { input: process::Tank { chemical: POLONIUM.into(), amount: 2000 }, output: process::Tank { chemical: ANTIMATTER.into(), amount: 1000 }, ..Sps::default() };
        tick(&mut full, 5_000_000);
        assert_eq!((full.input.amount, full.could_operate), (2000, false));
        let mut dry = Sps::default();
        tick(&mut dry, 5_000_000);
        assert_eq!(dry.last_processed, 0.0);
    }

    #[test]
    fn a_lot_of_energy_makes_whole_antimatter_at_once() {
        let mut s = Sps { input: process::Tank { chemical: POLONIUM.into(), amount: 2000 }, ..Sps::default() };
        tick(&mut s, 2_000_000_000);
        // The output tank holds one: the most that is made.
        assert_eq!(s.output.amount, 1000.min(2));
        let mut t = Sps { input: process::Tank { chemical: POLONIUM.into(), amount: 2000 }, ..Sps::default() };
        tick(&mut t, 500_000_000);
        assert!(t.output.amount <= 1 && t.last_processed > 0.0);
    }

    #[test]
    fn a_member_remembers_its_sps() {
        let f = Found { valid: true, min: [4, 70, -9], coils: 6 };
        let s = Sps { progress: 0.5, input_processed: 321, ..Sps::default() };
        let (g, back) = decode(&encode(&f, &s)).unwrap();
        assert_eq!((g, back.input_processed), (f, 321));
        assert!((back.progress - 0.5).abs() < 1e-6);
    }
}

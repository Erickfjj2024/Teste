//! The laser and the laser amplifier (`TileEntityLaser`, `TileEntityLaserAmplifier`, `TileEntityBasicLaser`): a block that fires energy in a
//! line from its front, up to 64 blocks, until something solid. What it hits that receives lasers takes the energy: a laser amplifier fills its
//! buffer with it, a laser focus matrix heats the fusion reactor's plasma. The laser fires 20 FE a tick while it has them; the amplifier fires
//! as much as its threshold says (between its minimum and maximum; by default 20) every `delay` ticks.
//!
//! Not here: digging blocks, hurting or burning what the beam crosses, the visible beam, the tractor beam, and redstone modes.
use super::*;

pub const LASER: &str = "mekanism:laser";
pub const AMPLIFIER: &str = "mekanism:laser_amplifier";
pub const RANGE: i32 = 64;
/// `usage.laser`, `storage.laserAmplifier` and the laser's buffer (usage * 2500 as for the machines).
pub const FIRE: u64 = 20;
pub const AMPLIFIER_CAPACITY: u64 = 5_000_000;
pub const LASER_CAPACITY: u64 = 50_000;
/// The buttons of the amplifier's screen.
pub const MIN_DOWN_BIG: i32 = 7300;
pub const MIN_DOWN: i32 = 7301;
pub const MIN_UP: i32 = 7302;
pub const MIN_UP_BIG: i32 = 7303;
pub const MAX_DOWN_BIG: i32 = 7304;
pub const MAX_DOWN: i32 = 7305;
pub const MAX_UP: i32 = 7306;
pub const MAX_UP_BIG: i32 = 7307;
pub const DELAY_DOWN: i32 = 7308;
pub const DELAY_UP: i32 = 7309;

/// The six faces in the order the rest of the mod uses: down, up, north, south, west, east.
pub const DOWN: usize = 0;
pub const UP: usize = 1;
pub const NORTH: usize = 2;
pub const SOUTH: usize = 3;
pub const WEST: usize = 4;
pub const EAST: usize = 5;

pub fn direction(face: usize) -> [i32; 3] {
    match face {
        DOWN => [0, -1, 0],
        UP => [0, 1, 0],
        NORTH => [0, 0, -1],
        SOUTH => [0, 0, 1],
        WEST => [-1, 0, 0],
        _ => [1, 0, 0],
    }
}

pub fn facing_name(face: usize) -> &'static str {
    ["down", "up", "north", "south", "west", "east"][face.min(5)]
}

pub fn face_of(name: &str) -> usize {
    match name {
        "down" => DOWN,
        "up" => UP,
        "south" => SOUTH,
        "west" => WEST,
        "east" => EAST,
        _ => NORTH,
    }
}

pub fn opposite(face: usize) -> usize {
    [UP, DOWN, SOUTH, NORTH, EAST, WEST][face.min(5)]
}

/// The face a new laser looks out of: toward the player that looks at it (`getNearestLookingDirection().getOpposite()`).
pub fn facing_for(yaw: f32, pitch: f32) -> usize {
    if pitch > 45.0 {
        UP
    } else if pitch < -45.0 {
        DOWN
    } else {
        sides::front_toward(yaw)
    }
}

/// The first solid block along a line from `from` (not included), or none within the range.
pub fn trace(from: [i32; 3], face: usize, range: i32, solid: impl Fn([i32; 3]) -> bool) -> Option<[i32; 3]> {
    let d = direction(face);
    (1..=range).map(|i| [from[0] + d[0] * i, from[1] + d[1] * i, from[2] + d[2] * i]).find(|p| solid(*p))
}

/// What an amplifier remembers: its thresholds and delay, and the ticks counted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Amp {
    pub min: u32,
    pub max: u32,
    pub delay: u32,
    pub ticks: u32,
}

impl Default for Amp {
    fn default() -> Amp {
        Amp { min: 0, max: AMPLIFIER_CAPACITY as u32, delay: 0, ticks: 0 }
    }
}

pub fn encode_amp(a: &Amp) -> Vec<u8> {
    let mut out = vec![1];
    for v in [a.min, a.max, a.delay, a.ticks] {
        out.extend(v.to_le_bytes());
    }
    out
}

pub fn decode_amp(data: &[u8]) -> Option<Amp> {
    if data.len() < 17 || data[0] != 1 {
        return None;
    }
    let u = |at: usize| u32::from_le_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]);
    Some(Amp { min: u(1), max: u(5), delay: u(9), ticks: u(13) })
}

impl Amp {
    /// How much the amplifier fires this tick (`toFire`): the laser's 20 held between the thresholds, once the delay has run.
    pub fn to_fire(&self) -> u64 {
        if self.ticks >= self.delay {
            u64::from(FIRE.clamp(u64::from(self.min), u64::from(self.max.max(self.min))) as u32)
        } else {
            0
        }
    }

    /// The tick count after a tick (`ticks < delay ? ticks + 1 : 0`).
    pub fn advance(&mut self) {
        self.ticks = if self.ticks < self.delay { self.ticks + 1 } else { 0 };
    }

    pub fn press(&mut self, id: i32) {
        let cap = AMPLIFIER_CAPACITY as u32;
        let add = |v: u32, d: i64| (i64::from(v) + d).clamp(0, i64::from(cap)) as u32;
        match id {
            MIN_DOWN_BIG => self.min = add(self.min, -100_000),
            MIN_DOWN => self.min = add(self.min, -1_000),
            MIN_UP => self.min = add(self.min, 1_000),
            MIN_UP_BIG => self.min = add(self.min, 100_000),
            MAX_DOWN_BIG => self.max = add(self.max, -100_000),
            MAX_DOWN => self.max = add(self.max, -1_000),
            MAX_UP => self.max = add(self.max, 1_000),
            MAX_UP_BIG => self.max = add(self.max, 100_000),
            DELAY_DOWN => self.delay = self.delay.saturating_sub(1),
            DELAY_UP => self.delay = (self.delay + 1).min(1_000_000),
            _ => return,
        }
        // The minimum never goes over the maximum, nor the maximum under the minimum.
        if self.min > self.max {
            if matches!(id, MIN_DOWN_BIG | MIN_DOWN | MIN_UP | MIN_UP_BIG) {
                self.max = self.min;
            } else {
                self.min = self.max;
            }
        }
    }
}

impl Mekanism {
    /// A tick of the lasers, amplifiers and focus matrices.
    pub(crate) fn run_lasers(&mut self, parts: &[&BlockEntity]) -> Vec<BlockEntityUpdate> {
        let presses = std::mem::take(&mut self.laser_buttons);
        let mut out = Vec::new();
        for e in parts {
            let block = e.state.split('[').next().unwrap_or("");
            let at = [e.pos.x, e.pos.y, e.pos.z];
            if block == fusion::MATRIX {
                // Energy a laser delivered heats the plasma of the reactor this matrix is part of.
                if let Some(energy) = self.beams.remove(&(e.dimension.clone(), at)) {
                    if let Some((found, _)) = fusion::decode(&e.data).filter(|(f, _)| f.valid) {
                        if let Some(s) = self.fusions.get_mut(&(e.dimension.clone(), found.min)) {
                            fusion::add_laser_energy(&mut s.reactor, energy);
                        }
                    }
                }
                // A plain click with the configurator sets the plasma to a billion degrees, but only in creative and
                // only while the reactor is formed (`TileEntityLaserFocusMatrix.onRightClick`). The port has no
                // empty-hand use, so with the configurator is the only way it works here.
                if let Some(click) = self.take_click(e) {
                    if !click.sneak && click.mode.is_configurating() {
                        let creative = players::online()
                            .into_iter()
                            .find(|p| p.id == click.player)
                            .is_some_and(|p| p.game_mode == GameMode::Creative);
                        if creative {
                            if let Some((found, _)) = fusion::decode(&e.data).filter(|(f, _)| f.valid) {
                                if let Some(s) = self.fusions.get_mut(&(e.dimension.clone(), found.min)) {
                                    s.reactor.plasma = 1_000_000_000.0;
                                }
                            }
                        }
                    }
                }
                continue;
            }
            let is_amp = block == AMPLIFIER;
            let mut amp = if is_amp { decode_amp(&e.data) } else { Some(Amp::default()) };
            // A new block looks toward the player that placed it.
            let mut facing = sides::property(&e.state, "facing").unwrap_or("north").to_owned();
            let fresh = if is_amp { amp.is_none() } else { e.data.first() != Some(&1) };
            if fresh {
                if let Some((yaw, pitch)) = nearest_look(e) {
                    let name = facing_name(facing_for(yaw, pitch));
                    if name != facing {
                        let _ = world::set_block_property(&e.dimension, e.pos.x, e.pos.y, e.pos.z, "facing", name);
                        facing = name.to_owned();
                    }
                }
            }
            let face = face_of(&facing);
            let cap = if is_amp { AMPLIFIER_CAPACITY } else { LASER_CAPACITY };
            let mut energy = e.energy;
            if is_amp {
                let a = amp.get_or_insert_with(Amp::default);
                for (_, _, id) in presses.iter().filter(|(d, p, _)| *d == e.dimension && (p.x, p.y, p.z) == (at[0], at[1], at[2])) {
                    a.press(*id);
                }
                // What lasers delivered fills the buffer (what does not fit is lost).
                if let Some(inflow) = self.beams.remove(&(e.dimension.clone(), at)) {
                    energy = (energy + inflow).min(cap);
                }
            }
            let to_fire = amp.as_ref().map_or(FIRE, Amp::to_fire);
            let mut fired = 0;
            if to_fire > 0 && energy >= to_fire {
                energy -= to_fire;
                fired = to_fire;
                let dim = e.dimension.clone();
                let target = trace(at, face, RANGE, |p| world::get_collision(&dim, p[0], p[1], p[2]).is_some_and(|c| !c.is_empty()));
                if let Some(t) = target {
                    *self.beams.entry((dim, t)).or_default() += fired;
                }
            }
            if let Some(a) = amp.as_mut() {
                a.advance();
            }
            // The laser takes energy from behind only (the amplifier from every side).
            let mut faces = vec![FlowSide::Input; 6];
            if !is_amp {
                faces = vec![FlowSide::None; 6];
                faces[opposite(face)] = FlowSide::Input;
            }
            let half = |v: u64, s: u32| ((v >> s) & 0xFFFF) as i32;
            let a = amp.unwrap_or_default();
            let menu = is_amp.then(|| {
                vec![
                    half(energy, 0),
                    half(energy, 16),
                    half(cap, 0),
                    half(cap, 16),
                    half(u64::from(a.min), 0),
                    half(u64::from(a.min), 16),
                    half(u64::from(a.max), 0),
                    half(u64::from(a.max), 16),
                    a.delay as i32,
                    fired.min(i32::MAX as u64) as i32,
                ]
            });
            out.push(BlockEntityUpdate {
                dimension: e.dimension.clone(),
                pos: e.pos,
                data: if is_amp { encode_amp(&a) } else { vec![1] },
                items: None,
                energy: (energy != e.energy).then_some(energy),
                energy_faces: Some(faces),
                tanks: None,
                tank_faces: None,
                menu_data: menu,
                transmitter_faces: None,
            });
        }
        out
    }
}

/// The look of the nearest player within twelve blocks.
fn nearest_look(e: &BlockEntity) -> Option<(f32, f32)> {
    let at = (f64::from(e.pos.x) + 0.5, f64::from(e.pos.z) + 0.5);
    players::online()
        .into_iter()
        .filter(|p| p.dimension == e.dimension)
        .map(|p| ((p.x - at.0).powi(2) + (p.z - at.1).powi(2), p))
        .filter(|(d, _)| *d < 144.0)
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .and_then(|(_, p)| player_control::get_look(&p.id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_beam_stops_at_the_first_solid_block() {
        let wall = [5, 64, 0];
        assert_eq!(trace([0, 64, 0], EAST, RANGE, |p| p == wall), Some(wall));
        assert_eq!(trace([0, 64, 0], WEST, RANGE, |p| p == wall), None);
        // Past the range nothing is hit.
        assert_eq!(trace([0, 64, 0], EAST, 4, |p| p == wall), None);
        assert_eq!(trace([0, 64, 0], UP, 10, |p| p[1] == 70), Some([0, 70, 0]));
    }

    #[test]
    fn the_amplifier_fires_between_its_thresholds() {
        let mut a = Amp::default();
        assert_eq!(a.to_fire(), 20);
        a.min = 1_000_000;
        a.max = 1_000_000;
        assert_eq!(a.to_fire(), 1_000_000);
        a.delay = 2;
        a.ticks = 0;
        assert_eq!(a.to_fire(), 0);
        a.advance();
        a.advance();
        assert_eq!(a.to_fire(), 1_000_000);
        a.advance();
        assert_eq!(a.ticks, 0);
    }

    #[test]
    fn the_buttons_keep_the_minimum_under_the_maximum() {
        let mut a = Amp { min: 0, max: 5000, delay: 0, ticks: 0 };
        a.press(MIN_UP_BIG);
        assert_eq!((a.min, a.max), (100_000, 100_000));
        a.press(MAX_DOWN_BIG);
        assert_eq!((a.min, a.max), (0, 0));
        a.press(MAX_UP);
        a.press(DELAY_UP);
        a.press(DELAY_UP);
        a.press(DELAY_DOWN);
        assert_eq!((a.max, a.delay), (1000, 1));
        let big = Amp { min: 0, max: AMPLIFIER_CAPACITY as u32, ..Amp::default() };
        let mut b = big;
        b.press(MAX_UP_BIG);
        assert_eq!(b.max, AMPLIFIER_CAPACITY as u32);
    }

    #[test]
    fn a_new_laser_looks_back_at_the_player() {
        assert_eq!(facing_for(0.0, 80.0), UP);
        assert_eq!(facing_for(0.0, -80.0), DOWN);
        assert_eq!(opposite(EAST), WEST);
        assert_eq!(face_of(facing_name(SOUTH)), SOUTH);
    }

    #[test]
    fn an_amplifier_remembers_its_settings() {
        let a = Amp { min: 5, max: 10, delay: 3, ticks: 2 };
        assert_eq!(decode_amp(&encode_amp(&a)), Some(a));
        assert_eq!(decode_amp(&[0, 1, 2]), None);
    }
}

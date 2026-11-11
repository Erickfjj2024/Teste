//! The laser and the laser amplifier (`TileEntityLaser`, `TileEntityLaserAmplifier`, `TileEntityBasicLaser`): a block that fires energy in a
//! line from its front, up to 64 blocks, until something solid. What it hits that receives lasers takes the energy: a laser amplifier fills its
//! buffer with it, a laser focus matrix heats the fusion reactor's plasma. The laser fires 4,000 FE a tick while it has them; the amplifier fires
//! as much as its threshold says (between its minimum and maximum; by default all it holds) every `delay` ticks.
//!
//! Digging: what the beam hits that does not receive lasers is worn down by the energy (hardness times 40,000 FE) and then broken as a player with a tool would
//! (`world.break-blocks`). The laser tractor beam is a laser that takes everything it holds, and that picks up the items inside its beam (`items-in-box`).
//!
//! Not here: hurting or burning what the beam crosses (the host has no laser damage yet), the visible beam, and redstone modes.
use super::*;

pub const LASER: &str = "mekanism:laser";
pub const AMPLIFIER: &str = "mekanism:laser_amplifier";
pub const TRACTOR: &str = "mekanism:laser_tractor_beam";
pub const RANGE: i32 = 64;
/// `usage.laser` (what the laser fires a tick), `storage.laserAmplifier` and `storage.laser`, in Forge Energy.
pub const FIRE: u64 = fe(10_000);
pub const AMPLIFIER_CAPACITY: u64 = fe(5_000_000_000);
pub const LASER_CAPACITY: u64 = fe(2_000_000);
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

/// What the tool that breaks a block for a laser carries: the original breaks with a fully charged Atomic Disassembler, which counts as the right tool for
/// everything; the netherite pickaxe is the nearest the host has.
pub const BREAK_TOOL: &str = "minecraft:netherite_pickaxe";

/// `getEnergyScale`: how thick the beam is for the energy it fires, 0.1 to 0.6 (a tenth of a block for each `FIRE`).
pub fn energy_scale(fired: u64) -> f64 {
    ((fired as f64 / FIRE as f64) / 10.0).min(0.6)
}

/// One tick of digging (`onUpdateServer`): the progress after the energy this tick fired is added; at the energy the block's hardness asks for
/// (`laserEnergyPerHardness`) it is broken and the progress starts again. A block with a negative hardness (unbreakable) is not worn at all.
pub fn dig_step(progress: u64, fired: u64, hardness: f32) -> (u64, bool) {
    if hardness < 0.0 {
        return (progress, false);
    }
    let progress = progress.saturating_add(fired);
    if progress as f64 >= f64::from(hardness) * crate::laser_tractor::LASER_ENERGY_PER_HARDNESS_FE as f64 {
        (0, true)
    } else {
        (progress, false)
    }
}

/// The box of the tractor beam (`getLaserBox`): from the laser's front to the block the beam hit (the whole of that block, where what it breaks drops),
/// as wide as the beam is thick.
pub fn beam_box(at: [i32; 3], face: usize, target: [i32; 3], scale: f64) -> ([f64; 3], [f64; 3]) {
    let d = direction(face);
    let half = scale / 2.0;
    let mut min = [0.0; 3];
    let mut max = [0.0; 3];
    for axis in 0..3 {
        let a = f64::from(at[axis]) + 0.5 + f64::from(d[axis]) * 0.501;
        let b = f64::from(target[axis]) + if d[axis] == 0 { 0.5 } else { 0.5 + f64::from(d[axis]) * 0.5 };
        min[axis] = a.min(b) - if d[axis] == 0 { half } else { 0.0 };
        max[axis] = a.max(b) + if d[axis] == 0 { half } else { 0.0 };
    }
    (min, max)
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
    /// How much the amplifier fires this tick (`toFire`, extracted from what it holds): all of it up to the maximum threshold, once
    /// the delay has run and it holds the minimum.
    pub fn to_fire(&self, energy: u64) -> u64 {
        if self.ticks >= self.delay && energy >= u64::from(self.min) {
            energy.min(u64::from(self.max))
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
            let is_tractor = block == TRACTOR;
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
            let cap = if is_amp { AMPLIFIER_CAPACITY } else if is_tractor { crate::laser_tractor::CAPACITY_FE } else { LASER_CAPACITY };
            let mut energy = e.energy;
            if is_tractor {
                // What lasers delivered fills the buffer, as the amplifier's does.
                if let Some(inflow) = self.beams.remove(&(e.dimension.clone(), at)) {
                    energy = (energy + inflow).min(cap);
                }
            }
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
            // `energyContainer.extract(toFire(), SIMULATE)`: the laser fires what it has up to its usage, the amplifier what it holds.
            let to_fire = if is_tractor { energy } else { amp.as_ref().map_or(FIRE.min(energy), |a| a.to_fire(energy)) };
            let mut fired = 0;
            let mut items_update = None;
            let key = (e.dimension.clone(), at);
            if to_fire > 0 {
                energy -= to_fire;
                fired = to_fire;
                let dim = e.dimension.clone();
                let target = trace(at, face, RANGE, |p| world::get_collision(&dim, p[0], p[1], p[2]).is_some_and(|c| !c.is_empty()));
                if let Some(t) = target {
                    self.laser_hits(&dim, key.clone(), t, fired);
                    if is_tractor {
                        items_update = self.tractor_pick_up(e, at, face, t, fired);
                    }
                } else {
                    self.digging.remove(&key);
                }
            } else {
                self.digging.remove(&key);
            }
            if let Some(a) = amp.as_mut() {
                a.advance();
            }
            // The laser takes energy from behind only (the amplifier from every side).
            let mut faces = vec![FlowSide::Input; 6];
            if !is_amp {
                faces = vec![FlowSide::None; 6];
                if !is_tractor {
                    faces[opposite(face)] = FlowSide::Input;
                }
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
                items: items_update,
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

impl Mekanism {
    /// What a beam does to the block it hit (`TileEntityBasicLaser.onUpdateServer`): a block that receives lasers takes the energy; any other is worn
    /// down by it and broken when the progress reaches what its hardness asks for. The progress starts again when the beam moves to another block.
    fn laser_hits(&mut self, dim: &str, key: (String, [i32; 3]), target: [i32; 3], fired: u64) {
        let state = world::get_block(dim, target[0], target[1], target[2]).unwrap_or_default();
        let block = state.split('[').next().unwrap_or("");
        if matches!(block, AMPLIFIER | TRACTOR | fusion::MATRIX) {
            self.digging.remove(&key);
            *self.beams.entry((dim.to_owned(), target)).or_default() += fired;
            return;
        }
        let Some(hardness) = world::block_hardness(dim, target[0], target[1], target[2]) else { return };
        let progress = match self.digging.get(&key) {
            Some((at, progress)) if *at == target => *progress,
            _ => 0,
        };
        let (progress, broke) = dig_step(progress, fired, hardness);
        if broke {
            let _ = world::break_blocks(dim, &[(target[0], target[1], target[2])], BREAK_TOOL, false, 0);
            self.digging.remove(&key);
        } else {
            self.digging.insert(key, (target, progress));
        }
    }

    /// `handleHitItem` for the items inside the tractor beam: what fits in the 27 slots is taken from the item entity (what does not, stays).
    /// Returns the slots when something went in.
    fn tractor_pick_up(&mut self, e: &BlockEntity, at: [i32; 3], face: usize, target: [i32; 3], fired: u64) -> Option<Vec<Option<ItemStack>>> {
        let (min, max) = beam_box(at, face, target, energy_scale(fired));
        let mut slots: Vec<Option<ItemStack>> = (0..crate::laser_tractor::SLOTS).map(|i| e.items.get(i).cloned().flatten()).collect();
        let mut changed = false;
        for item in entities::items_in_box(&e.dimension, (min[0], min[1], min[2]), (max[0], max[1], max[2])) {
            let max_stack = crate::bin::stack_size(&item.stack.item) as u32;
            let mut trial = slots.clone();
            let left = crate::qio_transfer::insert_into_container(&mut trial, &item.stack.item, item.stack.count, &item.stack.components, max_stack);
            let taken = item.stack.count - left;
            if taken == 0 {
                continue;
            }
            if entities::take_dropped_item(&e.dimension, item.id, taken).is_ok() {
                slots = trial;
                changed = true;
            }
        }
        changed.then_some(slots)
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
    fn digging_wears_a_block_down_by_its_hardness() {
        // Stone, 1.5: 60,000 FE; a laser fires 4,000 FE a tick.
        let mut progress = 0;
        let mut ticks = 0;
        loop {
            ticks += 1;
            let (p, broke) = dig_step(progress, FIRE, 1.5);
            progress = p;
            if broke {
                break;
            }
            assert!(ticks < 100);
        }
        assert_eq!(ticks, 15);
        assert_eq!(progress, 0);
        // Bedrock is never worn, and a block of no hardness breaks at once.
        assert_eq!(dig_step(5, FIRE, -1.0), (5, false));
        assert_eq!(dig_step(0, 1, 0.0), (0, true));
    }

    #[test]
    fn the_beam_gets_thicker_with_the_energy() {
        assert!((energy_scale(FIRE) - 0.1).abs() < 1e-9);
        assert!((energy_scale(FIRE * 3) - 0.3).abs() < 1e-9);
        assert_eq!(energy_scale(FIRE * 100), 0.6);
    }

    #[test]
    fn the_tractor_box_runs_from_its_front_to_the_end_of_the_target() {
        let (min, max) = beam_box([0, 64, 0], EAST, [5, 64, 0], 0.2);
        assert!((min[0] - 1.001).abs() < 1e-9 && (max[0] - 6.0).abs() < 1e-9);
        assert!((min[1] - 64.4).abs() < 1e-9 && (max[1] - 64.6).abs() < 1e-9);
        let (min, max) = beam_box([0, 64, 0], WEST, [-5, 64, 0], 0.2);
        assert!((min[0] + 5.0).abs() < 1e-9 && (max[0] + 0.001).abs() < 1e-9, "{min:?} {max:?}");
    }

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
    fn the_amplifier_fires_what_it_holds_between_its_thresholds() {
        let mut a = Amp::default();
        // By default it fires all it holds, up to the capacity.
        assert_eq!(a.to_fire(1_234), 1_234);
        a.min = 1_000;
        a.max = 5_000;
        assert_eq!(a.to_fire(999), 0);
        assert_eq!(a.to_fire(3_000), 3_000);
        assert_eq!(a.to_fire(9_000), 5_000);
        a.delay = 2;
        a.ticks = 0;
        assert_eq!(a.to_fire(3_000), 0);
        a.advance();
        a.advance();
        assert_eq!(a.to_fire(3_000), 3_000);
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

    #[test]
    fn laser_constants_match_java_config() {
        assert_eq!(RANGE, 64);
        assert_eq!(FIRE, 4_000);
        assert_eq!(LASER_CAPACITY, 800_000);
        assert_eq!(AMPLIFIER_CAPACITY, 2_000_000_000);
    }
}

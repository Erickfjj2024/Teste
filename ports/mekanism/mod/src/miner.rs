//! The digital miner (`TileEntityDigitalMiner`): it mines the blocks around it, in a square of `radius` blocks to each side between two heights,
//! one every `delay` ticks while it has energy, and keeps what they drop in its 27 slots; with auto-eject it pushes them out of its back.
//!
//! Which blocks: the ones its filters name (five slots holding an item of the block each), or with "inverse" every block but those. The scan of
//! the area runs a little each tick and queues what it finds, from the top down.
//!
//! Not here: tag and mod filters, replacing what was mined, pulling items from the back, the visual of the area, the upgrades, and mining
//! only inside loaded chunks is not told apart from mining out of reach: a chunk not loaded is skipped.
use super::*;

pub const MINER: &str = "mekanism:digital_miner";
/// `DEFAULT_RADIUS`, the min and max radius, `minerTicksPerMine`, `usage.digitalMiner`, `minerSilkMultiplier` and the buffer (`storage.digitalMiner`).
pub const DEFAULT_RADIUS: u8 = 10;
pub const MIN_RADIUS: u8 = 0;
pub const MAX_RADIUS: u8 = 32;
pub const TICKS_PER_MINE: u32 = 80;
pub const USAGE: u64 = crate::fe(1_000);
pub const SILK_MULTIPLIER: u64 = 12;
pub const CAPACITY: u64 = crate::fe(50_000);
pub const DEFAULT_MIN_Y: i32 = 0;
pub const DEFAULT_MAX_Y: i32 = 60;
/// 27 slots of what was mined, then five filters.
pub const BUFFER: usize = 27;
pub const FILTERS: usize = 5;
pub const SLOTS: usize = BUFFER + FILTERS;
/// Positions scanned a tick, and the most queued.
pub const SCAN_PER_TICK: usize = 512;
pub const QUEUE_MAX: usize = 4096;
pub const TOGGLE_RUN: i32 = 7800;
pub const RADIUS_DOWN: i32 = 7801;
pub const RADIUS_UP: i32 = 7802;
pub const MIN_DOWN: i32 = 7803;
pub const MIN_UP: i32 = 7804;
pub const MAX_DOWN: i32 = 7805;
pub const MAX_UP: i32 = 7806;
pub const TOGGLE_EJECT: i32 = 7807;
pub const TOGGLE_SILK: i32 = 7808;
pub const TOGGLE_INVERSE: i32 = 7809;
pub const RESET: i32 = 7810;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    pub radius: u8,
    pub min_y: i32,
    pub max_y: i32,
    pub running: bool,
    pub eject: bool,
    pub silk: bool,
    pub inverse: bool,
    pub delay: u32,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings { radius: DEFAULT_RADIUS, min_y: DEFAULT_MIN_Y, max_y: DEFAULT_MAX_Y, running: false, eject: false, silk: false, inverse: false, delay: 0 }
    }
}

pub fn encode(s: &Settings) -> Vec<u8> {
    let mut out = vec![1, s.radius, u8::from(s.running) | (u8::from(s.eject) << 1) | (u8::from(s.silk) << 2) | (u8::from(s.inverse) << 3)];
    out.extend(s.min_y.to_le_bytes());
    out.extend(s.max_y.to_le_bytes());
    out.extend(s.delay.to_le_bytes());
    out
}

pub fn decode(data: &[u8]) -> Option<Settings> {
    if data.len() < 15 || data[0] != 1 {
        return None;
    }
    let int = |at: usize| i32::from_le_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]);
    Some(Settings {
        radius: data[1].clamp(MIN_RADIUS, MAX_RADIUS),
        running: data[2] & 1 != 0,
        eject: data[2] & 2 != 0,
        silk: data[2] & 4 != 0,
        inverse: data[2] & 8 != 0,
        min_y: int(3),
        max_y: int(7),
        delay: int(11) as u32,
    })
}

impl Settings {
    pub fn press(&mut self, id: i32) -> bool {
        match id {
            TOGGLE_RUN => self.running = !self.running,
            RADIUS_DOWN => self.radius = self.radius.saturating_sub(1).max(MIN_RADIUS),
            RADIUS_UP => self.radius = (self.radius + 1).min(MAX_RADIUS),
            MIN_DOWN => self.min_y = (self.min_y - 1).max(-64),
            MIN_UP => self.min_y = (self.min_y + 1).min(self.max_y),
            MAX_DOWN => self.max_y = (self.max_y - 1).max(self.min_y),
            MAX_UP => self.max_y = (self.max_y + 1).min(319),
            TOGGLE_EJECT => self.eject = !self.eject,
            TOGGLE_SILK => self.silk = !self.silk,
            TOGGLE_INVERSE => self.inverse = !self.inverse,
            RESET => {}
            _ => return false,
        }
        // The area changed: what was queued is stale.
        matches!(id, RADIUS_DOWN | RADIUS_UP | MIN_DOWN | MIN_UP | MAX_DOWN | MAX_UP | TOGGLE_INVERSE | RESET)
    }
}

/// Whether a block is one the miner mines: not air, liquid or unbreakable, not its own, and named by a filter (or, inverse, not named).
pub fn wants(block: &str, filters: &[String], inverse: bool) -> bool {
    let name = block.split('[').next().unwrap_or("");
    if name.ends_with(":air")
        || matches!(
            name,
            "minecraft:water" | "minecraft:lava" | "minecraft:bedrock" | "minecraft:barrier" | "minecraft:end_portal_frame" | "minecraft:end_portal" | "minecraft:nether_portal"
                | "minecraft:command_block" | "minecraft:structure_block" | "minecraft:jigsaw" | "mekanism:bounding_block" | MINER
        )
    {
        return false;
    }
    filters.iter().any(|f| f == name) != inverse
}

/// The `index`th position of the area, from the top down, then across: `(dx, y, dz)` in the square of side `2r + 1`.
pub fn position(index: usize, radius: u8, min_y: i32, max_y: i32) -> Option<(i32, i32, i32)> {
    let side = usize::from(radius) * 2 + 1;
    let layers = (max_y - min_y + 1).max(0) as usize;
    if index >= side * side * layers {
        return None;
    }
    let (layer, rest) = (index / (side * side), index % (side * side));
    let y = max_y - layer as i32;
    Some(((rest % side) as i32 - i32::from(radius), y, (rest / side) as i32 - i32::from(radius)))
}

/// The energy the miner uses a tick (`MinerEnergyContainer.updateMinerEnergyPerTick`):
/// 1 000 J/t (400 FE/t) base, 12× with silk touch (`minerSilkMultiplier`), scaled up with radius above 10 and height above 60.
pub fn energy_usage(s: &Settings) -> u64 {
    let base = if s.silk { 1_000.0 * (SILK_MULTIPLIER as f64) } else { 1_000.0 };
    let radius_cost = if s.radius > DEFAULT_RADIUS {
        (f64::from(s.radius) - f64::from(DEFAULT_RADIUS)) / (f64::from(MAX_RADIUS) - f64::from(DEFAULT_RADIUS))
    } else {
        0.0
    };
    let height = (s.max_y - s.min_y).max(0) as f64;
    let height_cost = if height > f64::from(DEFAULT_MAX_Y - DEFAULT_MIN_Y) {
        (height - f64::from(DEFAULT_MAX_Y - DEFAULT_MIN_Y)) / 323.0
    } else {
        0.0
    };
    let total_joules = (base * (1.0 + radius_cost) * (1.0 + height_cost)).ceil() as u64;
    crate::fe(total_joules)
}

/// What the miner is doing, between ticks.
#[derive(Default)]
pub struct Work {
    pub cursor: usize,
    pub queue: Vec<[i32; 3]>,
    pub finished: bool,
    pub mined: u64,
    pub seen_set: bool,
}

pub fn item_faces(back: usize) -> Vec<Vec<u32>> {
    let mut faces = vec![Vec::new(); 6];
    faces[back] = (0..BUFFER as u32).collect();
    faces
}

impl Mekanism {
    pub(crate) fn run_miners(&mut self, miners: &[&BlockEntity], tick: u64) -> Vec<BlockEntityUpdate> {
        let presses = std::mem::take(&mut self.miner_buttons);
        let mut out = Vec::new();
        for e in miners {
            let key = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
            let known = decode(&e.data);
            let mut s = known.unwrap_or_default();
            let mut facing = sides::property(&e.state, "facing").unwrap_or("north").to_owned();
            if known.is_none() {
                let front = cube::facing_name(front_for(e));
                if front != facing {
                    let _ = world::set_block_property(&e.dimension, e.pos.x, e.pos.y, e.pos.z, "facing", front);
                    facing = front.to_owned();
                }
            }
            let work = self.miner_work.entry(key.clone()).or_default();
            for (_, p, id) in presses.iter().filter(|(d, p, _)| *d == e.dimension && (p.x, p.y, p.z) == (e.pos.x, e.pos.y, e.pos.z)) {
                let _ = p;
                if s.press(*id) {
                    work.cursor = 0;
                    work.queue.clear();
                    work.finished = false;
                }
            }
            let back = laser::opposite(laser::face_of(&facing));
            if !work.seen_set || known.is_none() {
                work.seen_set = true;
                let _ = world::set_item_faces(&e.dimension, e.pos.x, e.pos.y, e.pos.z, &item_faces(back));
            }
            let _ = world::set_item_eject(&e.dimension, e.pos.x, e.pos.y, e.pos.z, s.eject);
            let mut items: Vec<Option<ItemStack>> = (0..SLOTS).map(|i| e.items.get(i).cloned().flatten()).collect();
            let filters: Vec<String> = items[BUFFER..].iter().flatten().map(|i| i.item.clone()).collect();
            let mut energy = e.energy;
            let mut status = 0; // 0 stopped, 1 searching, 2 mining, 3 no energy, 4 finished
            if s.running && !work.finished {
                // The scan: a little each tick.
                if work.queue.len() < QUEUE_MAX {
                    for _ in 0..SCAN_PER_TICK {
                        let Some((dx, y, dz)) = position(work.cursor, s.radius, s.min_y, s.max_y) else {
                            work.finished = work.queue.is_empty();
                            work.cursor = usize::MAX / 2;
                            break;
                        };
                        work.cursor += 1;
                        let p = [e.pos.x + dx, y, e.pos.z + dz];
                        if world::get_block(&e.dimension, p[0], p[1], p[2]).is_some_and(|b| wants(&b, &filters, s.inverse)) {
                            work.queue.push(p);
                            if work.queue.len() >= QUEUE_MAX {
                                break;
                            }
                        }
                    }
                }
                status = 1;
                if !work.queue.is_empty() {
                    status = 2;
                    let usage = energy_usage(&s);
                    if energy >= usage {
                        energy -= usage;
                        if s.delay > 0 {
                            s.delay -= 1;
                        }
                        if s.delay == 0 {
                            s.delay = TICKS_PER_MINE;
                            // The next block that is still there to mine.
                            while let Some(p) = work.queue.pop() {
                                let still = world::get_block(&e.dimension, p[0], p[1], p[2]).is_some_and(|b| wants(&b, &filters, s.inverse));
                                if still {
                                    let _ = world::mine_block(
                                        &e.dimension,
                                        p[0],
                                        p[1],
                                        p[2],
                                        "minecraft:netherite_pickaxe",
                                        s.silk,
                                        e.pos.x,
                                        e.pos.y,
                                        e.pos.z,
                                        back as u32,
                                    );
                                    work.mined += 1;
                                    break;
                                }
                            }
                        }
                    } else {
                        status = 3;
                    }
                } else if work.finished {
                    status = 4;
                    s.running = false;
                }
            }
            let _ = tick;
            let half = |v: u64, sh: u32| ((v >> sh) & 0xFFFF) as i32;
            let menu = vec![
                i32::from(s.running),
                half(energy, 0),
                half(energy, 16),
                half(CAPACITY, 0),
                half(CAPACITY, 16),
                i32::from(s.radius),
                s.min_y,
                s.max_y,
                i32::from(s.eject),
                i32::from(s.silk),
                i32::from(s.inverse),
                work.queue.len().min(i32::MAX as usize) as i32,
                work.mined.min(i32::MAX as u64) as i32,
                status,
            ];
            let _ = &mut items;
            let data = encode(&s);
            if energy == e.energy && data == e.data && menu == e.menu_data {
                continue;
            }
            out.push(BlockEntityUpdate {
                dimension: e.dimension.clone(),
                pos: e.pos,
                data,
                items: None,
                energy: (energy != e.energy).then_some(energy),
                energy_faces: None,
                tanks: None,
                tank_faces: None,
                menu_data: Some(menu),
                transmitter_faces: None,
            });
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_area_is_walked_from_the_top_down() {
        assert_eq!(position(0, 1, 0, 2), Some((-1, 2, -1)));
        assert_eq!(position(8, 1, 0, 2), Some((1, 2, 1)));
        assert_eq!(position(9, 1, 0, 2), Some((-1, 1, -1)));
        assert_eq!(position(27, 1, 0, 2), None);
        assert_eq!(position(0, 1, 5, 4), None);
    }

    #[test]
    fn it_mines_what_the_filters_name_or_everything_else() {
        let f = vec!["minecraft:diamond_ore".to_owned()];
        assert!(wants("minecraft:diamond_ore", &f, false));
        assert!(!wants("minecraft:stone", &f, false));
        assert!(wants("minecraft:stone", &f, true));
        assert!(!wants("minecraft:diamond_ore", &f, true));
        // Never air, liquids, bedrock or itself, even when inverse.
        for b in ["minecraft:air", "minecraft:water[level=0]", "minecraft:bedrock", MINER, "mekanism:bounding_block[part=none]"] {
            assert!(!wants(b, &f, true), "{b}");
        }
        assert!(!wants("minecraft:stone", &[], false));
    }

    #[test]
    fn the_buttons_keep_the_area_sane() {
        let mut s = Settings::default();
        assert!(s.press(RADIUS_UP));
        assert_eq!(s.radius, 11);
        for _ in 0..40 {
            s.press(RADIUS_UP);
        }
        assert_eq!(s.radius, MAX_RADIUS);
        for _ in 0..40 {
            s.press(RADIUS_DOWN);
        }
        assert_eq!(s.radius, MIN_RADIUS);
        s.press(MIN_UP);
        assert_eq!(s.min_y, 1);
        s.max_y = 5;
        s.min_y = 5;
        s.press(MIN_UP);
        s.press(MAX_DOWN);
        assert_eq!((s.min_y, s.max_y), (5, 5));
        assert!(!s.press(TOGGLE_RUN));
        assert!(s.running);
        assert!(s.press(TOGGLE_INVERSE) && s.inverse);
    }

    #[test]
    fn energy_usage_scales_with_silk_radius_and_height() {
        let s = Settings::default();
        assert_eq!(energy_usage(&s), USAGE);
        let silk = Settings { silk: true, ..Settings::default() };
        assert_eq!(energy_usage(&silk), USAGE * SILK_MULTIPLIER);
        assert_eq!(position(0, 0, 0, 1), Some((0, 1, 0)));
    }

    #[test]
    fn a_miner_remembers_its_settings() {
        let s = Settings { radius: 7, min_y: -20, max_y: 99, running: true, eject: true, silk: false, inverse: true, delay: 33 };
        assert_eq!(decode(&encode(&s)), Some(s));
        assert_eq!(decode(&[]), None);
    }

    #[test]
    fn its_back_gives_the_buffer_and_nothing_else_does() {
        let f = item_faces(laser::NORTH);
        assert_eq!(f[laser::NORTH].len(), BUFFER);
        assert!(f[laser::SOUTH].is_empty() && f[laser::UP].is_empty());
    }
}

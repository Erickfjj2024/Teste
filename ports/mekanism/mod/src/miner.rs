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
/// The energy slot (`EnergyInventorySlot.fillOrConvert`, 152, 20) after the 27 of the buffer.
pub const ENERGY_SLOT: usize = BUFFER;
pub const SLOTS: usize = BUFFER + 1;
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
pub const START: i32 = 7811;
pub const STOP: i32 = 7812;
/// The filter list's rows (`ROW_BASE + row`), the delete button and the boxes of the config page: a new item filter, a new tag filter, the radius and the two heights.
pub const ROW_BASE: i32 = 7830;
pub const DELETE: i32 = 7860;
pub const TEXT_ITEM: i32 = 7861;
pub const TEXT_TAG: i32 = 7862;
pub const TEXT_RADIUS: i32 = 7863;
pub const TEXT_MIN: i32 = 7864;
pub const TEXT_MAX: i32 = 7865;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub radius: u8,
    pub min_y: i32,
    pub max_y: i32,
    pub running: bool,
    pub eject: bool,
    pub silk: bool,
    pub inverse: bool,
    pub delay: u32,
    /// The filters (`filters.rs`) and the row selected in their list.
    pub filters: Vec<filters::Filter>,
    pub selected: Option<usize>,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings { radius: DEFAULT_RADIUS, min_y: DEFAULT_MIN_Y, max_y: DEFAULT_MAX_Y, running: false, eject: false, silk: false, inverse: false, delay: 0, filters: Vec::new(), selected: None }
    }
}

pub fn encode(s: &Settings) -> Vec<u8> {
    let mut out = vec![1, s.radius, u8::from(s.running) | (u8::from(s.eject) << 1) | (u8::from(s.silk) << 2) | (u8::from(s.inverse) << 3)];
    out.extend(s.min_y.to_le_bytes());
    out.extend(s.max_y.to_le_bytes());
    out.extend(s.delay.to_le_bytes());
    out.push(s.selected.map_or(0, |i| i as u8 + 1));
    filters::encode(&s.filters, &mut out);
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
        filters: filters::decode(data, 16).0,
        selected: data.get(15).copied().filter(|b| *b > 0).map(|b| usize::from(b - 1)),
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
            RESET => self.running = false,
            START => self.running = true,
            STOP => self.running = false,
            DELETE => {
                if let Some(i) = self.selected.take().filter(|i| *i < self.filters.len()) {
                    self.filters.remove(i);
                    return true;
                }
                return false;
            }
            r if (ROW_BASE..ROW_BASE + filters::ROWS as i32).contains(&r) => {
                let i = (r - ROW_BASE) as usize;
                if i < self.filters.len() {
                    self.selected = (self.selected != Some(i)).then_some(i);
                }
                return false;
            }
            _ => return false,
        }
        // The area changed: what was queued is stale.
        matches!(id, RADIUS_DOWN | RADIUS_UP | MIN_DOWN | MIN_UP | MAX_DOWN | MAX_UP | TOGGLE_INVERSE | RESET)
    }

    /// What a confirmed box of the config page asks (`GuiDigitalMinerConfig.setText`): a new filter from an id or a tag, the radius (0 to the most), the lowest and the highest height
    /// (each kept between the world's floor and ceiling and the other one). Says whether the area changed.
    pub fn text(&mut self, id: i32, text: &str) -> bool {
        match id {
            TEXT_ITEM | TEXT_TAG => filters::add(&mut self.filters, id == TEXT_TAG, text),
            TEXT_RADIUS => match text.trim().parse::<u32>() {
                Ok(v) if v != u32::from(self.radius) => {
                    self.radius = v.clamp(u32::from(MIN_RADIUS), u32::from(MAX_RADIUS)) as u8;
                    true
                }
                _ => false,
            },
            TEXT_MIN => match text.trim().parse::<i32>() {
                Ok(v) => {
                    let v = v.clamp(-64, self.max_y);
                    let changed = v != self.min_y;
                    self.min_y = v;
                    changed
                }
                Err(_) => false,
            },
            TEXT_MAX => match text.trim().parse::<i32>() {
                Ok(v) => {
                    let v = v.clamp(self.min_y, 319);
                    let changed = v != self.max_y;
                    self.max_y = v;
                    changed
                }
                Err(_) => false,
            },
            _ => false,
        }
    }
}

/// Whether a block is one the miner mines: not air, liquid or unbreakable, not its own, and named by a filter (or, inverse, not named).
pub fn wants(block: &str, list: &[filters::Filter], inverse: bool, tags: &dyn Fn(&str) -> Vec<String>) -> bool {
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
    list.iter().any(|f| filters::matches(f, name, tags)) != inverse
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
        let texts_in = std::mem::take(&mut self.miner_texts);
        let mut out = Vec::new();
        for e in miners {
            let key = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
            let known = decode(&e.data);
            let mut s = known.clone().unwrap_or_default();
            let mut facing = sides::property(&e.state, "facing").unwrap_or("north").to_owned();
            if known.is_none() {
                let front = cube::facing_name(front_for(e));
                if front != facing {
                    let _ = world::set_block_property(&e.dimension, e.pos.x, e.pos.y, e.pos.z, "facing", front);
                    facing = front.to_owned();
                }
            }
            let work = self.miner_work.entry(key.clone()).or_default();
            let here = |p: &BlockPos| (p.x, p.y, p.z) == (e.pos.x, e.pos.y, e.pos.z);
            for (_, _, id) in presses.iter().filter(|(d, p, _)| *d == e.dimension && here(p)) {
                if s.press(*id) {
                    work.cursor = 0;
                    work.queue.clear();
                    work.finished = false;
                }
            }
            for (_, _, id, text) in texts_in.iter().filter(|(d, p, _, _)| *d == e.dimension && here(p)) {
                if s.text(*id, text) {
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
            let list = s.filters.clone();
            let tags = |id: &str| recipe_types::item_tags(id);
            let mut energy = e.energy;
            // `EnergyInventorySlot.fillOrConvert` (152, 20): an item worth energy fills the buffer when it all fits.
            let mut energy_item = false;
            if let Some((now, left)) = cube::fill_from_slot(&items[ENERGY_SLOT], energy, CAPACITY) {
                energy = now;
                items[ENERGY_SLOT] = left;
                energy_item = true;
            }
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
                        if world::get_block(&e.dimension, p[0], p[1], p[2]).is_some_and(|b| wants(&b, &list, s.inverse, &tags)) {
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
                            // The next block that is still there to mine (top down).
                            while !work.queue.is_empty() {
                                let p = work.queue.remove(0);
                                let still = world::get_block(&e.dimension, p[0], p[1], p[2]).is_some_and(|b| wants(&b, &list, s.inverse, &tags));
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
                s.filters.len() as i32,
                s.selected.map_or(0, |i| i as i32 + 1),
            ];
            let texts = filters::texts(&s.filters, s.selected);
            if self.menu_texts.get(&key) != Some(&texts) {
                let _ = world::set_menu_text(&e.dimension, e.pos.x, e.pos.y, e.pos.z, &texts);
                self.menu_texts.insert(key.clone(), texts);
            }
            let data = encode(&s);
            if energy == e.energy && data == e.data && menu == e.menu_data && !energy_item {
                continue;
            }
            out.push(BlockEntityUpdate {
                dimension: e.dimension.clone(),
                pos: e.pos,
                data,
                items: energy_item.then_some(items),
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
        let f = vec![filters::Filter { tag: false, target: "minecraft:diamond_ore".into() }, filters::Filter { tag: true, target: "c:ores/tin".into() }];
        let tags = |id: &str| if id == "mekanism:tin_ore" { vec!["c:ores/tin".to_owned()] } else { Vec::new() };
        assert!(wants("minecraft:diamond_ore", &f, false, &tags));
        assert!(wants("mekanism:tin_ore[x=1]", &f, false, &tags));
        assert!(!wants("minecraft:stone", &f, false, &tags));
        assert!(wants("minecraft:stone", &f, true, &tags));
        assert!(!wants("minecraft:diamond_ore", &f, true, &tags));
        // Never air, liquids, bedrock or itself, even when inverse.
        for b in ["minecraft:air", "minecraft:water[level=0]", "minecraft:bedrock", MINER, "mekanism:bounding_block[part=none]"] {
            assert!(!wants(b, &f, true, &tags), "{b}");
        }
        assert!(!wants("minecraft:stone", &[], false, &tags));
    }

    #[test]
    fn the_config_boxes_set_the_area_and_make_filters() {
        let mut s = Settings::default();
        assert!(s.text(TEXT_RADIUS, "20"));
        assert_eq!(s.radius, 20);
        assert!(s.text(TEXT_RADIUS, "500"));
        assert_eq!(s.radius, MAX_RADIUS);
        assert!(!s.text(TEXT_RADIUS, "x"));
        assert!(s.text(TEXT_MIN, "-30"));
        assert_eq!(s.min_y, -30);
        assert!(s.text(TEXT_MIN, "9999"));
        assert_eq!(s.min_y, s.max_y);
        assert!(s.text(TEXT_MAX, "100"));
        assert_eq!(s.max_y, 100);
        assert!(!s.text(TEXT_MAX, "100"));
        assert!(s.text(TEXT_ITEM, "minecraft:coal_ore"));
        assert!(!s.text(TEXT_ITEM, "minecraft:coal_ore"));
        assert!(s.text(TEXT_TAG, "c:ores/coal"));
        assert_eq!(s.filters.len(), 2);
        assert!(!s.press(ROW_BASE + 1));
        assert_eq!(s.selected, Some(1));
        assert!(s.press(DELETE));
        assert_eq!(s.filters.len(), 1);
        assert!(s.press(START) || s.running);
        assert!(s.running);
        s.press(STOP);
        assert!(!s.running);
        assert_eq!(decode(&encode(&s)), Some(s));
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
        assert!(s.press(RESET));
        assert!(!s.running);
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
        let s = Settings { radius: 7, min_y: -20, max_y: 99, running: true, eject: true, silk: false, inverse: true, delay: 33, filters: vec![filters::Filter { tag: true, target: "c:ores/iron".into() }], selected: Some(0) };
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

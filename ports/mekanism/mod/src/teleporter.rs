//! The teleporter (`TileEntityTeleporter`): a block with a frame of `mekanism:teleporter_frame` blocks (a 1 by 2 opening beside it, framed by
//! nine frames). Teleporters on the same frequency send what stands in their opening to the nearest other one, for energy: 100 plus 2 for each
//! block of distance (and 1000 more to change dimension).
//!
//! Frequencies are named ones (`frequency.rs`), chosen on the screen's list or made with its boxes, as the quantum entangloporter's. Only players are teleported
//! (mobs, items and vehicles are not), nothing is shown in the opening and there is no sound.
use super::*;
use std::collections::{HashMap, HashSet};

pub const TELEPORTER: &str = "mekanism:teleporter";
pub const FRAME: &str = "mekanism:teleporter_frame";
/// `usage.teleporter` base, distance and dimension penalty; the teleporter's energy buffer.
/// `usage.teleporterBase`, `teleporterDistance` and `dimensionPenalty` (Joules) and `storage.teleporter` (Forge Energy).
pub const BASE_COST: u64 = 1_000;
pub const DISTANCE_COST: f64 = 10.0;
pub const DIMENSION_PENALTY: u64 = 10_000;
pub const CAPACITY: u64 = fe(5_000_000);
/// Ticks the destination waits after receiving someone.
pub const DELAY: u32 = 5;
/// The first id of the frequency selector's buttons and boxes (`frequency::id_*`).
pub const FREQ_BASE: i32 = 8100;

pub fn direction(face: usize) -> [i32; 3] {
    laser::direction(face)
}

/// The frame blocks around a teleporter, relative to it, for the opening in `face` and a frame turned 90 degrees or not (`hasFrame`).
pub fn frame_offsets(face: usize, rotated: bool) -> Vec<[i32; 3]> {
    let d = direction(face);
    // The direction the sides of the frame go: across the opening.
    let across = if rotated {
        if d[2] != 0 { [1, 0, 0] } else { [0, 0, 1] }
    } else if d[1] != 0 {
        [1, 0, 0]
    } else {
        [0, 1, 0]
    };
    let mut out = Vec::new();
    for k in 0..=3 {
        for sign in [-1, 1] {
            out.push([d[0] * k + across[0] * sign, d[1] * k + across[1] * sign, d[2] * k + across[2] * sign]);
        }
    }
    out.push([d[0] * 3, d[1] * 3, d[2] * 3]);
    out
}

/// The way the teleporter's frame is, if any: the face of its opening and whether the frame is turned.
pub fn find_frame(is_frame: impl Fn([i32; 3]) -> bool) -> Option<(usize, bool)> {
    for face in 0..6 {
        for rotated in [false, true] {
            if frame_offsets(face, rotated).into_iter().all(&is_frame) {
                return Some((face, rotated));
            }
        }
    }
    None
}

/// The energy a trip costs: the distance and the dimension change (`calculateEnergyCost`, without passengers).
pub fn cost(distance: f64, other_dimension: bool) -> u64 {
    let travel = (DISTANCE_COST * distance).round().max(0.0) as u64;
    fe(BASE_COST + travel + if other_dimension { DIMENSION_PENALTY } else { 0 })
}

/// How far a dimension scales the coordinates (`coordinateScale`).
pub fn scale_of(dimension: &str) -> f64 {
    if dimension == "minecraft:the_nether" { 8.0 } else { 1.0 }
}

/// The cost of a trip from a player at `from` in one dimension to `to` in another (or the same).
pub fn trip_cost(from_dim: &str, from: [f64; 3], to_dim: &str, to: [i32; 3]) -> u64 {
    if from_dim == to_dim {
        let d = ((from[0] - f64::from(to[0])).powi(2) + (from[1] - f64::from(to[1])).powi(2) + (from[2] - f64::from(to[2])).powi(2)).sqrt();
        return cost(d, false);
    }
    let (cur, target) = (scale_of(from_dim), scale_of(to_dim));
    let (dx, dz) = if cur <= target {
        let s = cur / target;
        (from[0] * s - f64::from(to[0]), from[2] * s - f64::from(to[2]))
    } else {
        let s = target / cur;
        (from[0] - f64::from(to[0]) * s, from[2] - f64::from(to[2]) * s)
    };
    let d = (dx * dx + (from[1] - f64::from(to[1])).powi(2) + dz * dz).sqrt();
    cost(d, true)
}

/// The nearest other teleporter of a frequency (same dimension first, by distance; else the first in another).
pub fn closest<'a>(me: (&str, [i32; 3]), others: &'a [(String, [i32; 3], usize)]) -> Option<&'a (String, [i32; 3], usize)> {
    let dist = |p: [i32; 3]| {
        let d = [p[0] - me.1[0], p[1] - me.1[1], p[2] - me.1[2]];
        i64::from(d[0]).pow(2) + i64::from(d[1]).pow(2) + i64::from(d[2]).pow(2)
    };
    others
        .iter()
        .filter(|o| !(o.0 == me.0 && o.1 == me.1))
        .min_by_key(|o| (u8::from(o.0 != me.0), dist(o.1)))
}

/// Where a teleporter puts what arrives (`getTeleporterTargetPos`): the first cell of its opening, two below for a downward one.
pub fn target(pos: [i32; 3], face: usize) -> [i32; 3] {
    let d = direction(face);
    if face == laser::DOWN {
        [pos[0], pos[1] - 2, pos[2]]
    } else {
        [pos[0] + d[0], pos[1] + d[1], pos[2] + d[2]]
    }
}

/// The two cells of the opening: the first to the second (`getTeleporterBoundingBox`).
pub fn opening(pos: [i32; 3], face: usize) -> ([i32; 3], [i32; 3]) {
    let d = direction(face);
    let a = [pos[0] + d[0], pos[1] + d[1], pos[2] + d[2]];
    let b = [pos[0] + 2 * d[0], pos[1] + 2 * d[1], pos[2] + 2 * d[2]];
    ([a[0].min(b[0]), a[1].min(b[1]), a[2].min(b[2])], [a[0].max(b[0]), a[1].max(b[1]), a[2].max(b[2])])
}

pub fn inside(point: [f64; 3], bounds: ([i32; 3], [i32; 3])) -> bool {
    (0..3).all(|i| point[i] >= f64::from(bounds.0[i]) && point[i] < f64::from(bounds.1[i]) + 1.0)
}

pub fn encode(frequency: &Option<frequency::Freq>) -> Vec<u8> {
    let mut out = vec![1];
    frequency::encode(frequency, &mut out);
    out
}

/// The frequency a teleporter's bytes hold (none when it has none or no bytes yet).
pub fn decode(data: &[u8]) -> Option<frequency::Freq> {
    if data.first() != Some(&1) {
        return None;
    }
    frequency::decode(data, 1).0
}

#[derive(Default)]
pub struct Frames {
    /// The frame found for each teleporter and when it was looked for.
    pub found: HashMap<(String, i32, i32, i32), (Option<(usize, bool)>, u64)>,
    /// Who arrived at each teleporter and has to leave its opening before being sent on.
    pub arrived: HashMap<(String, i32, i32, i32), HashSet<String>>,
    pub delay: HashMap<(String, i32, i32, i32), u32>,
}

impl Mekanism {
    pub(crate) fn run_teleporters(&mut self, parts: &[&BlockEntity], tick: u64) -> Vec<BlockEntityUpdate> {
        let presses = std::mem::take(&mut self.teleporter_buttons);
        let texts_in = std::mem::take(&mut self.teleporter_texts);
        let mut out = Vec::new();
        let mut table: Vec<(String, String, [i32; 3], usize)> = Vec::new();
        let mut info: HashMap<(String, i32, i32, i32), (Option<frequency::Freq>, Option<usize>)> = HashMap::new();
        let known: Vec<frequency::Freq> = parts.iter().filter_map(|e| decode(&e.data)).collect();
        for e in parts {
            let key = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
            let mut freq = decode(&e.data);
            // What the screen asked for, by the player that did (`GuiFrequencySelector`): the list a private frequency is chosen from is that player's.
            let asked: Vec<(frequency::Action, String, String)> = presses
                .iter()
                .filter(|(d, p, _, _, _)| *d == e.dimension && (p.x, p.y, p.z) == (e.pos.x, e.pos.y, e.pos.z))
                .filter_map(|(_, _, id, who, name)| frequency::button_action(*id, FREQ_BASE).map(|a| (a, who.clone(), name.clone())))
                .chain(
                    texts_in
                        .iter()
                        .filter(|(d, p, _, _, _, _)| *d == e.dimension && (p.x, p.y, p.z) == (e.pos.x, e.pos.y, e.pos.z))
                        .filter_map(|(_, _, id, text, who, name)| frequency::text_action(*id, FREQ_BASE, text).map(|a| (a, who.clone(), name.clone()))),
                )
                .collect();
            for (action, who, name) in asked {
                self.freq_viewer.insert(key.clone(), (who.clone(), name.clone()));
                if let Some(next) = frequency::apply(&action, &freq, &frequency::listing(&known, &who), &who, &name) {
                    freq = next;
                }
            }
            // The frame is looked for once a second.
            let cached = self.frames.found.get(&key).copied();
            let frame = match cached {
                Some((f, at)) if tick.saturating_sub(at) < 20 => f,
                _ => {
                    let f = find_frame(|o| world::get_block(&e.dimension, e.pos.x + o[0], e.pos.y + o[1], e.pos.z + o[2]).is_some_and(|b| b.starts_with(FRAME)));
                    self.frames.found.insert(key.clone(), (f, tick));
                    f
                }
            };
            if let Some(f) = &freq {
                if let Some((face, _)) = frame {
                    table.push((f.key(), e.dimension.clone(), [e.pos.x, e.pos.y, e.pos.z], face));
                }
            }
            info.insert(key, (freq, frame.map(|f| f.0)));
        }
        let players = players::online();
        for e in parts {
            let pos = [e.pos.x, e.pos.y, e.pos.z];
            let key = (e.dimension.clone(), pos[0], pos[1], pos[2]);
            let (freq, frame) = info[&key].clone();
            let mut energy = e.energy;
            // `EnergyInventorySlot.fillOrConvert` (153, 7): an item that is worth energy fills the buffer when it all fits.
            let mut items_update = None;
            if let Some((now, left)) = cube::fill_from_slot(&e.items.first().cloned().flatten(), energy, CAPACITY) {
                energy = now;
                items_update = Some(vec![left]);
            }
            let others: Vec<(String, [i32; 3], usize)> = match &freq {
                Some(f) => table.iter().filter(|t| t.0 == f.key()).map(|t| (t.1.clone(), t.2, t.3)).collect(),
                None => Vec::new(),
            };
            let dest = closest((&e.dimension, pos), &others).cloned();
            // 0 no frequency, 1 no frame, 2 no destination, 3 not enough energy, 4 ready.
            let mut status = if freq.is_none() {
                0
            } else if frame.is_none() {
                1
            } else if dest.is_none() {
                2
            } else {
                4
            };
            if let (Some(face), Some((dest_dim, dest_pos, dest_face))) = (frame, dest.as_ref()) {
                let bounds = opening(pos, face);
                let here: Vec<&Player> = players.iter().filter(|p| p.dimension == e.dimension && inside([p.x, p.y, p.z], bounds)).collect();
                // Who left the opening may be sent again.
                if let Some(set) = self.frames.arrived.get_mut(&key) {
                    set.retain(|id| here.iter().any(|p| &p.id == id));
                }
                let delay = self.frames.delay.entry(key.clone()).or_insert(0);
                let waiting = *delay > 0;
                *delay = delay.saturating_sub(1);
                let arrived = self.frames.arrived.get(&key).cloned().unwrap_or_default();
                let candidates: Vec<&&Player> = here.iter().filter(|p| !arrived.contains(&p.id)).collect();
                if !waiting {
                    for p in candidates {
                        let need = trip_cost(&e.dimension, [p.x, p.y, p.z], dest_dim, *dest_pos);
                        if energy < need {
                            status = 3;
                            continue;
                        }
                        let t = target(*dest_pos, *dest_face);
                        if player_control::teleport(&p.id, dest_dim, f64::from(t[0]) + 0.5, f64::from(t[1]), f64::from(t[2]) + 0.5, None, None).is_ok() {
                            energy -= need;
                            let dkey = (dest_dim.clone(), dest_pos[0], dest_pos[1], dest_pos[2]);
                            self.frames.arrived.entry(dkey.clone()).or_default().insert(p.id.clone());
                            self.frames.delay.insert(dkey, DELAY);
                        }
                    }
                }
            }
            let half = |v: u64, s: u32| ((v >> s) & 0xFFFF) as i32;
            // The list the screen shows is the viewer's: the player that used it last, or else the owner of its private frequency.
            let viewer = self.freq_viewer.get(&key).map(|(id, _)| id.clone()).or_else(|| freq.as_ref().filter(|f| f.private).map(|f| f.owner.clone())).unwrap_or_default();
            let listing = frequency::listing(&known, &viewer);
            let menu = vec![
                i32::from(freq.is_some()),
                status,
                half(energy, 0),
                half(energy, 16),
                half(CAPACITY, 0),
                half(CAPACITY, 16),
                others.len() as i32,
                listing.public.len() as i32,
                listing.private.len() as i32,
            ];
            let texts = frequency::texts(&freq, &listing);
            if self.menu_texts.get(&key) != Some(&texts) {
                let _ = world::set_menu_text(&e.dimension, e.pos.x, e.pos.y, e.pos.z, &texts);
                self.menu_texts.insert(key.clone(), texts);
            }
            if energy == e.energy && menu == e.menu_data && decode(&e.data) == freq && e.data.first() == Some(&1) && items_update.is_none() {
                continue;
            }
            out.push(BlockEntityUpdate {
                dimension: e.dimension.clone(),
                pos: e.pos,
                data: encode(&freq),
                items: items_update,
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
    fn the_frame_is_nine_blocks_around_a_two_high_opening() {
        let f = frame_offsets(laser::UP, false);
        assert_eq!(f.len(), 9);
        assert!(f.contains(&[1, 0, 0]) && f.contains(&[-1, 3, 0]) && f.contains(&[0, 3, 0]));
        // The opening itself is empty.
        assert!(!f.contains(&[0, 1, 0]) && !f.contains(&[0, 2, 0]));
        let side = frame_offsets(laser::EAST, false);
        assert!(side.contains(&[0, 1, 0]) && side.contains(&[3, 0, 0]) && !side.contains(&[1, 0, 0]));
    }

    #[test]
    fn a_frame_is_found_in_any_direction_and_rotation() {
        let around: HashSet<[i32; 3]> = frame_offsets(laser::NORTH, true).into_iter().collect();
        assert_eq!(find_frame(|p| around.contains(&p)), Some((laser::NORTH, true)));
        assert_eq!(find_frame(|_| false), None);
    }

    #[test]
    fn a_trip_costs_a_base_distance_and_a_penalty() {
        // 1000 J a trip, 10 J a block, 10000 J for a dimension (Forge Energy: 2/5 of them).
        assert_eq!(cost(0.0, false), 400);
        assert_eq!(cost(50.0, false), 600);
        assert_eq!(cost(10.0, true), 4_440);
        assert_eq!(trip_cost("minecraft:overworld", [0.0, 64.0, 0.0], "minecraft:overworld", [30, 64, 40]), 600);
        // Overworld to nether: the distance is taken after the 1:8 scale.
        assert_eq!(trip_cost("minecraft:overworld", [80.0, 64.0, 0.0], "minecraft:the_nether", [10, 64, 0]), fe(1_000 + 10_000));
    }

    #[test]
    fn the_nearest_other_teleporter_is_the_destination() {
        let others = vec![
            ("minecraft:overworld".to_owned(), [0, 64, 0], laser::UP),
            ("minecraft:overworld".to_owned(), [100, 64, 0], laser::UP),
            ("minecraft:the_nether".to_owned(), [1, 64, 0], laser::UP),
            ("minecraft:overworld".to_owned(), [10, 64, 0], laser::UP),
        ];
        let near = closest(("minecraft:overworld", [0, 64, 0]), &others).unwrap();
        assert_eq!(near.1, [10, 64, 0]);
        // Alone in its dimension, the other dimension's one is the destination.
        assert_eq!(closest(("minecraft:the_end", [5, 5, 5]), &others).unwrap().0, "minecraft:the_nether");
        assert!(closest(("minecraft:overworld", [0, 64, 0]), &others[..1]).is_none());
    }

    #[test]
    fn what_arrives_stands_in_the_opening() {
        assert_eq!(target([5, 64, 5], laser::UP), [5, 65, 5]);
        assert_eq!(target([5, 64, 5], laser::DOWN), [5, 62, 5]);
        let b = opening([5, 64, 5], laser::UP);
        assert!(inside([5.5, 65.5, 5.5], b) && inside([5.5, 66.9, 5.5], b) && !inside([5.5, 67.1, 5.5], b));
    }

    #[test]
    fn a_teleporter_remembers_its_frequency() {
        let f = frequency::Freq { name: "Home".into(), private: true, owner: "u1".into(), owner_name: "Alex".into() };
        assert_eq!(decode(&encode(&Some(f.clone()))), Some(f));
        assert_eq!(decode(&encode(&None)), None);
        assert_eq!(decode(&[]), None);
    }
}

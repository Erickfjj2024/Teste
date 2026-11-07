//! The logistical sorter (`TileEntityLogisticalSorter`): it takes items from the container behind it and puts them in front of it: into a
//! container, or onto a logistical transporter. With auto-eject it moves everything; otherwise only the items of its filters. In the port the
//! filters are items placed in five slots (the items are not used up: they stay there to be taken back).
//!
//! The sorter has an intake slot (0, fed from behind by the game's `pull-items`) and an outbox (1, pushed out of its front by the game's
//! ejector, or taken by a transporter beside it); this moves the item from one to the other.
//!
//! Not here: item, tag, material and mod filters, a color for each filter (routing in the transporters), minimum and maximum counts,
//! round robin, redstone control.
use super::*;

pub const SORTER: &str = "mekanism:logistical_sorter";
/// The buttons of the screen.
pub const TOGGLE_AUTO: i32 = 7400;
pub const TOGGLE_SINGLE: i32 = 7401;
/// The slots: the intake, the outbox and the filters.
pub const INTAKE: usize = 0;
pub const OUTBOX: usize = 1;
pub const FILTERS: usize = 5;
pub const SLOTS: usize = 2 + FILTERS;
/// How often it pulls (ticks).
pub const DELAY: u64 = 10;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Settings {
    pub auto: bool,
    pub single: bool,
}

pub fn encode(s: &Settings) -> Vec<u8> {
    vec![1, u8::from(s.auto), u8::from(s.single)]
}

pub fn decode(data: &[u8]) -> Option<Settings> {
    (data.len() >= 3 && data[0] == 1).then(|| Settings { auto: data[1] == 1, single: data[2] == 1 })
}

/// The items the sorter pulls: any with auto-eject, else its filters (none: nothing is pulled, which `None` says).
pub fn wanted(s: &Settings, filters: &[String]) -> Option<Vec<String>> {
    if s.auto {
        Some(Vec::new())
    } else if filters.is_empty() {
        None
    } else {
        Some(filters.to_vec())
    }
}

/// The faces of the container: the intake from behind, the outbox out of the front.
pub fn item_faces(face: usize) -> Vec<Vec<u32>> {
    let mut faces = vec![Vec::new(); 6];
    faces[laser::opposite(face)] = vec![INTAKE as u32];
    faces[face] = vec![OUTBOX as u32];
    faces
}

impl Mekanism {
    pub(crate) fn run_sorters(&mut self, sorters: &[&BlockEntity], tick: u64) -> Vec<BlockEntityUpdate> {
        let presses = std::mem::take(&mut self.sorter_buttons);
        let mut out = Vec::new();
        for e in sorters {
            let at = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
            let mut settings = decode(&e.data);
            let mut facing = sides::property(&e.state, "facing").unwrap_or("north").to_owned();
            if settings.is_none() {
                // A new sorter looks toward the player that placed it.
                let look = players::online()
                    .into_iter()
                    .filter(|p| p.dimension == e.dimension)
                    .map(|p| ((p.x - f64::from(e.pos.x) - 0.5).powi(2) + (p.z - f64::from(e.pos.z) - 0.5).powi(2), p))
                    .filter(|(d, _)| *d < 144.0)
                    .min_by(|a, b| a.0.total_cmp(&b.0))
                    .and_then(|(_, p)| player_control::get_look(&p.id));
                if let Some((yaw, pitch)) = look {
                    let name = laser::facing_name(laser::facing_for(yaw, pitch));
                    if name != facing {
                        let _ = world::set_block_property(&e.dimension, e.pos.x, e.pos.y, e.pos.z, "facing", name);
                        facing = name.to_owned();
                    }
                }
            }
            let mut s = settings.take().unwrap_or_default();
            for (_, p, id) in presses.iter().filter(|(d, p, _)| *d == e.dimension && (p.x, p.y, p.z) == (e.pos.x, e.pos.y, e.pos.z)) {
                let _ = p;
                match *id {
                    TOGGLE_AUTO => s.auto = !s.auto,
                    TOGGLE_SINGLE => s.single = !s.single,
                    _ => {}
                }
            }
            let face = laser::face_of(&facing);
            let first = !self.sorter_set.contains(&at);
            if first {
                self.sorter_set.insert(at.clone());
                let _ = world::set_item_faces(&e.dimension, e.pos.x, e.pos.y, e.pos.z, &item_faces(face));
                let _ = world::set_item_eject(&e.dimension, e.pos.x, e.pos.y, e.pos.z, true);
            }
            let mut items: Vec<Option<ItemStack>> = (0..SLOTS).map(|i| e.items.get(i).cloned().flatten()).collect();
            let mut changed = false;
            // The intake moves to the outbox when that is empty.
            if items[INTAKE].is_some() && items[OUTBOX].is_none() {
                items[OUTBOX] = items[INTAKE].take();
                changed = true;
            }
            // Every few ticks it pulls from behind, when it holds nothing.
            if tick % DELAY == 0 && items[INTAKE].is_none() && items[OUTBOX].is_none() {
                let filters: Vec<String> = items[2..].iter().flatten().map(|s| s.item.clone()).collect();
                if let Some(list) = wanted(&s, &filters) {
                    let _ = world::pull_items(&e.dimension, e.pos.x, e.pos.y, e.pos.z, laser::opposite(face) as u32, &list, if s.single { 1 } else { 64 });
                }
            }
            let data = encode(&s);
            let menu = vec![i32::from(s.auto), i32::from(s.single)];
            if !changed && !first && data == e.data && menu == e.menu_data {
                continue;
            }
            out.push(BlockEntityUpdate {
                dimension: e.dimension.clone(),
                pos: e.pos,
                data,
                items: changed.then_some(items),
                energy: None,
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
    fn it_pulls_what_it_is_told_to() {
        let none = Settings::default();
        assert_eq!(wanted(&none, &[]), None);
        assert_eq!(wanted(&none, &["minecraft:iron_ingot".into()]), Some(vec!["minecraft:iron_ingot".to_owned()]));
        assert_eq!(wanted(&Settings { auto: true, single: false }, &["x".into()]), Some(Vec::new()));
    }

    #[test]
    fn its_faces_take_in_from_behind_and_give_out_in_front() {
        let faces = item_faces(laser::EAST);
        assert_eq!(faces[laser::WEST], vec![0]);
        assert_eq!(faces[laser::EAST], vec![1]);
        assert!(faces[laser::UP].is_empty());
    }

    #[test]
    fn it_remembers_its_settings() {
        let s = Settings { auto: true, single: false };
        assert_eq!(decode(&encode(&s)), Some(s));
        assert_eq!(decode(&[]), None);
    }
}

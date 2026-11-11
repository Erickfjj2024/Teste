//! The logistical sorter (`TileEntityLogisticalSorter`): it takes items from the container behind it and puts them in front of it: into a
//! container, or onto a logistical transporter. With auto-eject it moves everything; otherwise only the items of its filters: item filters and tag filters made on its screen
//! (`filters.rs`) and kept in its bytes.
//!
//! The sorter has an intake slot (0, fed from behind by the game's `pull-items`) and an outbox (1, pushed out of its front by the game's
//! ejector, or taken by a transporter beside it); this moves the item from one to the other.
//!
//! Not here: mod-id and material filters, a colour for each filter (routing in the transporters), minimum and maximum counts, round robin, redstone control.
use super::*;

pub const SORTER: &str = "mekanism:logistical_sorter";
/// The buttons of the screen.
pub const TOGGLE_AUTO: i32 = 7400;
pub const TOGGLE_SINGLE: i32 = 7401;
/// A row of the list of filters selected (`ROW_BASE + row`), the delete button, and the boxes that make an item filter and a tag filter.
pub const ROW_BASE: i32 = 7410;
pub const DELETE: i32 = 7450;
pub const TEXT_ITEM: i32 = 7451;
pub const TEXT_TAG: i32 = 7452;
/// The slots: the intake and the outbox.
pub const INTAKE: usize = 0;
pub const OUTBOX: usize = 1;
pub const SLOTS: usize = 2;
/// How often it pulls (ticks).
pub const DELAY: u64 = 10;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Settings {
    pub auto: bool,
    pub single: bool,
    pub filters: Vec<filters::Filter>,
    /// The row of the list that is selected (the delete button takes it away).
    pub selected: Option<usize>,
}

pub fn encode(s: &Settings) -> Vec<u8> {
    let mut out = vec![1, u8::from(s.auto), u8::from(s.single), s.selected.map_or(0, |i| i as u8 + 1)];
    filters::encode(&s.filters, &mut out);
    out
}

pub fn decode(data: &[u8]) -> Option<Settings> {
    if data.len() < 5 || data[0] != 1 {
        return None;
    }
    let filters = filters::decode(data, 4).0;
    let selected = (data[3] > 0).then(|| usize::from(data[3] - 1)).filter(|i| *i < filters.len());
    Some(Settings { auto: data[1] == 1, single: data[2] == 1, filters, selected })
}

/// What a press or a confirmed box of the screen does to the settings.
pub fn apply(s: &mut Settings, id: i32, text: Option<&str>) {
    match id {
        TOGGLE_AUTO => s.auto = !s.auto,
        TOGGLE_SINGLE => s.single = !s.single,
        DELETE => {
            if let Some(i) = s.selected.take().filter(|i| *i < s.filters.len()) {
                s.filters.remove(i);
            }
        }
        TEXT_ITEM | TEXT_TAG => {
            if let Some(t) = text {
                filters::add(&mut s.filters, id == TEXT_TAG, t);
            }
        }
        r if (ROW_BASE..ROW_BASE + filters::ROWS as i32).contains(&r) => {
            let i = (r - ROW_BASE) as usize;
            if i < s.filters.len() {
                s.selected = (s.selected != Some(i)).then_some(i);
            }
        }
        _ => {}
    }
}

/// The items the sorter pulls: any with auto-eject, else what its filters name (none: nothing is pulled, which `None` says).
pub fn wanted(s: &Settings, members: &dyn Fn(&str) -> Vec<String>) -> Option<Vec<String>> {
    if s.auto {
        Some(Vec::new())
    } else {
        let ids = filters::items(&s.filters, members);
        (!ids.is_empty()).then_some(ids)
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
        let texts_in = std::mem::take(&mut self.sorter_texts);
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
            let here = |p: &BlockPos| (p.x, p.y, p.z) == (e.pos.x, e.pos.y, e.pos.z);
            for (_, _, id) in presses.iter().filter(|(d, p, _)| *d == e.dimension && here(p)) {
                apply(&mut s, *id, None);
            }
            for (_, _, id, text) in texts_in.iter().filter(|(d, p, _, _)| *d == e.dimension && here(p)) {
                apply(&mut s, *id, Some(text));
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
                if let Some(list) = wanted(&s, &|t| recipe_types::item_tag_members(t)) {
                    let _ = world::pull_items(&e.dimension, e.pos.x, e.pos.y, e.pos.z, laser::opposite(face) as u32, &list, if s.single { 1 } else { 64 });
                }
            }
            let data = encode(&s);
            let menu = vec![i32::from(s.auto), i32::from(s.single), s.filters.len() as i32, s.selected.map_or(0, |i| i as i32 + 1)];
            let texts = filters::texts(&s.filters, s.selected);
            if self.menu_texts.get(&at) != Some(&texts) {
                let _ = world::set_menu_text(&e.dimension, e.pos.x, e.pos.y, e.pos.z, &texts);
                self.menu_texts.insert(at.clone(), texts);
            }
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
        let members = |t: &str| if t == "c:ingots/iron" { vec!["minecraft:iron_ingot".to_owned()] } else { Vec::new() };
        let mut s = Settings::default();
        assert_eq!(wanted(&s, &members), None);
        apply(&mut s, TEXT_ITEM, Some("minecraft:stone"));
        apply(&mut s, TEXT_TAG, Some("c:ingots/iron"));
        assert_eq!(wanted(&s, &members), Some(vec!["minecraft:stone".to_owned(), "minecraft:iron_ingot".to_owned()]));
        apply(&mut s, TOGGLE_AUTO, None);
        assert_eq!(wanted(&s, &members), Some(Vec::new()));
    }

    #[test]
    fn a_row_is_selected_and_deleted() {
        let mut s = Settings::default();
        apply(&mut s, TEXT_ITEM, Some("minecraft:stone"));
        apply(&mut s, TEXT_ITEM, Some("minecraft:dirt"));
        apply(&mut s, ROW_BASE + 1, None);
        assert_eq!(s.selected, Some(1));
        apply(&mut s, ROW_BASE + 1, None);
        assert_eq!(s.selected, None);
        apply(&mut s, ROW_BASE + 5, None);
        assert_eq!(s.selected, None);
        apply(&mut s, ROW_BASE, None);
        apply(&mut s, DELETE, None);
        assert_eq!(s.filters.len(), 1);
        assert_eq!(s.filters[0].target, "minecraft:dirt");
        assert_eq!(s.selected, None);
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
        let mut s = Settings { auto: true, single: false, ..Settings::default() };
        apply(&mut s, TEXT_TAG, Some("c:ores/tin"));
        apply(&mut s, ROW_BASE, None);
        assert_eq!(decode(&encode(&s)), Some(s));
        assert_eq!(decode(&[]), None);
    }
}

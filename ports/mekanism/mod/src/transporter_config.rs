//! The Transporter Config tab (`GuiTransporterConfig`, `TileComponentEjector`): the window of a machine that moves items where the player picks the colour of what it ejects (the colour button) and
//! whether it takes items from a logistical transporter only in that colour (strict input). The colour is one of the 18 transporter colours (`EnumColor`, `TransporterUtils.colors`), none
//! included; a click goes to the next, a right click to the previous (`PacketEjectColor`: `TransporterUtils.increment` and `decrement`).
//!
//! What the host keeps is `world.set-item-eject-color` and `world.set-item-strict-input` (API 54). The mod keeps the same two values in the world's data, by block position, to show them on the screen:
//! the screen's two numbers after its own (`transporter_table.rs`, written by the converter) are the colour (0 none, 1-18) and the strict flag.
//!
//! Not here: the six input colour buttons of the window (the host has one strict-input colour per block, its eject colour, not one for each side), shift-click to clear the colour, and the
//! configuration kept in the dismantled block's item.
use super::*;
use std::collections::HashMap;

pub use crate::transporter_table::TRANSPORTED;
const KEY: &str = "mekanism:transporter_config";
/// The window's buttons: the colour (next, previous) and strict input.
pub const COLOR: i32 = 9300;
pub const PREVIOUS: i32 = 9301;
pub const STRICT: i32 = 9302;
/// How many colours there are (`EnumColor` has 18).
pub const COLORS: u8 = 18;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Config {
    pub color: Option<u8>,
    pub strict: bool,
}

pub type Registry = HashMap<(String, i32, i32, i32), Config>;

/// The index of the window's first number for a block, when its screens have the tab.
pub fn data_index(block: &str) -> Option<usize> {
    TRANSPORTED.iter().find(|(b, _)| *b == block).map(|(_, at)| *at)
}

/// `TransporterUtils.increment`: none, then each colour in turn, then none again.
pub fn next_color(c: Option<u8>) -> Option<u8> {
    match c {
        None => Some(0),
        Some(n) if n + 1 >= COLORS => None,
        Some(n) => Some(n + 1),
    }
}

/// `TransporterUtils.decrement`.
pub fn previous_color(c: Option<u8>) -> Option<u8> {
    match c {
        None => Some(COLORS - 1),
        Some(0) => None,
        Some(n) => Some(n - 1),
    }
}

pub fn encode(list: &Registry) -> Vec<u8> {
    let mut lines: Vec<String> = list
        .iter()
        .filter(|(_, c)| **c != Config::default())
        .map(|((d, x, y, z), c)| format!("{d}|{x}|{y}|{z}|{}|{}", c.color.map_or(0, |n| n + 1), u8::from(c.strict)))
        .collect();
    lines.sort();
    lines.join("\n").into_bytes()
}

pub fn decode(data: &[u8]) -> Registry {
    let mut out = HashMap::new();
    for line in String::from_utf8_lossy(data).lines() {
        let mut f = line.splitn(6, '|');
        let (Some(d), Some(x), Some(y), Some(z), Some(c), Some(s)) = (f.next(), f.next(), f.next(), f.next(), f.next(), f.next()) else { continue };
        let (Ok(x), Ok(y), Ok(z), Ok(c)) = (x.parse(), y.parse(), z.parse(), c.parse::<u8>()) else { continue };
        out.insert((d.to_owned(), x, y, z), Config { color: c.checked_sub(1).filter(|n| *n < COLORS), strict: s == "1" });
    }
    out
}

/// A screen's data with the two numbers at `at` (anything the data had past it is dropped).
pub fn with_config(mut data: Vec<i32>, at: usize, c: Config) -> Vec<i32> {
    data.resize(at, 0);
    data.push(c.color.map_or(0, |n| i32::from(n) + 1));
    data.push(i32::from(c.strict));
    data
}

impl Mekanism {
    fn transporter_registry(&mut self) -> &mut Registry {
        self.transporter_cfg.get_or_insert_with(|| horizon_mod_sdk::server::world_data::get(KEY).map(|d| decode(&d)).unwrap_or_default())
    }

    fn transporter_save(&mut self) {
        let bytes = encode(self.transporter_registry());
        let _ = horizon_mod_sdk::server::world_data::set(KEY, &bytes);
    }

    /// A press on the window: the colour button or strict input. True when the press was one of the window's.
    pub(crate) fn transporter_button(&mut self, e: &BlockEntity, id: i32) -> bool {
        if !(COLOR..=STRICT).contains(&id) {
            return false;
        }
        let key = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
        let mut c = self.transporter_registry().get(&key).copied().unwrap_or_default();
        match id {
            COLOR => c.color = next_color(c.color),
            PREVIOUS => c.color = previous_color(c.color),
            _ => c.strict = !c.strict,
        }
        let _ = world::set_item_eject_color(&e.dimension, e.pos.x, e.pos.y, e.pos.z, c.color);
        let _ = world::set_item_strict_input(&e.dimension, e.pos.x, e.pos.y, e.pos.z, c.strict);
        self.transporter_registry().insert(key.clone(), c);
        self.transporter_dirty.insert(key);
        self.transporter_save();
        true
    }

    /// A block is broken: what it kept goes with it.
    pub(crate) fn transporter_broken(&mut self, dimension: &str, pos: BlockPos) {
        if self.transporter_registry().remove(&(dimension.to_owned(), pos.x, pos.y, pos.z)).is_some() {
            self.transporter_save();
        }
    }

    /// The window's numbers at the end of each screen's data that was sent, and the screens whose colour or strictness just changed.
    pub(crate) fn transporter_stamp(&mut self, entities: &[BlockEntity], out: &mut Vec<BlockEntityUpdate>) {
        let dirty = std::mem::take(&mut self.transporter_dirty);
        for e in entities {
            let Some(at) = data_index(e.state.split('[').next().unwrap_or("")) else { continue };
            let key = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
            let c = self.transporter_registry().get(&key).copied().unwrap_or_default();
            if let Some(u) = out.iter_mut().find(|u| u.dimension == e.dimension && u.pos == e.pos) {
                if let Some(data) = u.menu_data.take() {
                    u.menu_data = Some(with_config(data, at, c));
                } else if dirty.contains(&key) {
                    u.menu_data = Some(with_config(e.menu_data.clone(), at, c));
                }
            } else if dirty.contains(&key) {
                out.push(BlockEntityUpdate {
                    dimension: e.dimension.clone(),
                    pos: e.pos,
                    data: e.data.clone(),
                    items: None,
                    energy: None,
                    energy_faces: None,
                    tanks: None,
                    tank_faces: None,
                    menu_data: Some(with_config(e.menu_data.clone(), at, c)),
                    transmitter_faces: None,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_colour_steps_through_none_and_the_eighteen() {
        let mut c = None;
        for n in 0..COLORS {
            c = next_color(c);
            assert_eq!(c, Some(n));
        }
        assert_eq!(next_color(c), None, "after the last one comes none");
        assert_eq!(previous_color(None), Some(17));
        assert_eq!(previous_color(Some(0)), None);
        assert_eq!(previous_color(Some(5)), Some(4));
    }

    #[test]
    fn the_registry_survives_its_text_and_keeps_only_what_is_set() {
        let mut m = HashMap::new();
        m.insert(("minecraft:overworld".to_owned(), 1, -2, 3), Config { color: Some(17), strict: true });
        m.insert(("minecraft:the_nether".to_owned(), 0, 0, 0), Config::default());
        let back = decode(&encode(&m));
        assert_eq!(back.len(), 1);
        assert_eq!(back[&("minecraft:overworld".to_owned(), 1, -2, 3)], Config { color: Some(17), strict: true });
    }

    #[test]
    fn the_numbers_go_after_the_screens_own() {
        assert_eq!(with_config(vec![1, 2], 4, Config { color: Some(0), strict: true }), vec![1, 2, 0, 0, 1, 1]);
        assert_eq!(with_config(vec![1, 2, 3, 4, 5], 3, Config::default()), vec![1, 2, 3, 0, 0]);
    }
}

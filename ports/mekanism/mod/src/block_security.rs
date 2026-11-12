//! The security of blocks (`TileComponentSecurity`, `GuiSecurityTab`): a block of the mod that has a screen remembers who placed it (its owner) and a mode (public, private or trusted). A private
//! block opens only for its owner; the owner changes the mode with the lock tab at the screen's right (click for the next mode, right click for the previous one).
//!
//! The owners and modes are kept in the world's data (`world_data`), by block position. A screen's last data number is the mode (`security_table.rs`, written by the converter, says which number).
//! Not here: the trusted players (the Security Desk's lists, so a trusted block is the owner's alone for now), the Security Desk's global override, the items' and entities' security, and the
//! protection against breaking a block that is not yours.
use super::*;
use security::SecurityMode;
use std::collections::HashMap;

pub use crate::security_table::SECURED;
const KEY: &str = "mekanism:block_security";
/// The tab's buttons: the next mode (left click) and the previous one (right click).
pub const NEXT: i32 = 9100;
pub const PREVIOUS: i32 = 9101;

#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub owner: String,
    pub owner_name: String,
    pub mode: SecurityMode,
}

/// The index of the screen's security number for a block, when its screens have the tab.
pub fn data_index(block: &str) -> Option<usize> {
    SECURED.iter().find(|(b, _)| *b == block).map(|(_, at)| *at)
}

/// The registry as text, a block on each line.
pub fn encode(list: &HashMap<(String, i32, i32, i32), Entry>) -> Vec<u8> {
    let mut lines: Vec<String> = list
        .iter()
        .map(|((d, x, y, z), e)| format!("{d}|{x}|{y}|{z}|{}|{}|{}", e.mode as u8, e.owner, e.owner_name))
        .collect();
    lines.sort();
    lines.join("\n").into_bytes()
}

pub fn decode(data: &[u8]) -> HashMap<(String, i32, i32, i32), Entry> {
    let mut out = HashMap::new();
    for line in String::from_utf8_lossy(data).lines() {
        let mut f = line.splitn(7, '|');
        let (Some(d), Some(x), Some(y), Some(z), Some(m), Some(o), Some(n)) = (f.next(), f.next(), f.next(), f.next(), f.next(), f.next(), f.next()) else { continue };
        let (Ok(x), Ok(y), Ok(z), Ok(m)) = (x.parse(), y.parse(), z.parse(), m.parse::<u8>()) else { continue };
        out.insert((d.to_owned(), x, y, z), Entry { owner: o.to_owned(), owner_name: n.to_owned(), mode: SecurityMode::from_u8(m) });
    }
    out
}

/// What a player keeps at the Security Desk (`SecurityFrequency`): the mode and the trusted names the desk gives to what the player owns, and whether it overrides the blocks' own modes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OwnerData {
    pub name: String,
    pub mode: SecurityMode,
    pub overridden: bool,
    pub trusted: Vec<String>,
}

pub const OWNERS_KEY: &str = "mekanism:security_owners";

pub fn encode_owners(list: &HashMap<String, OwnerData>) -> Vec<u8> {
    let mut lines: Vec<String> =
        list.iter().map(|(id, o)| format!("{id}|{}|{}|{}|{}", o.name, o.mode as u8, u8::from(o.overridden), o.trusted.join(","))).collect();
    lines.sort();
    lines.join("\n").into_bytes()
}

pub fn decode_owners(data: &[u8]) -> HashMap<String, OwnerData> {
    let mut out = HashMap::new();
    for line in String::from_utf8_lossy(data).lines() {
        let mut f = line.splitn(5, '|');
        let (Some(id), Some(name), Some(m), Some(o), Some(t)) = (f.next(), f.next(), f.next(), f.next(), f.next()) else { continue };
        let Ok(m) = m.parse::<u8>() else { continue };
        let trusted = t.split(',').filter(|n| !n.is_empty()).map(str::to_owned).collect();
        out.insert(id.to_owned(), OwnerData { name: name.to_owned(), mode: SecurityMode::from_u8(m), overridden: o == "1", trusted });
    }
    out
}

/// `SecurityUtils.canAccess`: whether `player` may use a block with this entry. The owner always; anyone for a public one; for a trusted one the names the owner's desk lists; with the desk's
/// override on, the desk's mode stands in for the block's.
pub fn can_access(entry: &Entry, player: &str, player_name: &str, owner: Option<&OwnerData>) -> bool {
    if entry.owner == player {
        return true;
    }
    let mode = owner.filter(|o| o.overridden).map_or(entry.mode, |o| o.mode);
    match mode {
        SecurityMode::Public => true,
        SecurityMode::Private => false,
        SecurityMode::Trusted => owner.is_some_and(|o| o.trusted.iter().any(|t| t.eq_ignore_ascii_case(player_name))),
    }
}

/// The data of a screen with the mode in it: the original numbers cut or padded to the security number's place, then the mode.
pub fn with_mode(mut data: Vec<i32>, at: usize, mode: SecurityMode) -> Vec<i32> {
    data.resize(at, 0);
    data.push(mode as i32);
    data
}

impl Mekanism {
    fn security_registry(&mut self) -> &mut HashMap<(String, i32, i32, i32), Entry> {
        self.block_owners.get_or_insert_with(|| horizon_mod_sdk::server::world_data::get(KEY).map(|d| decode(&d)).unwrap_or_default())
    }

    pub(crate) fn security_owners(&mut self) -> &mut HashMap<String, OwnerData> {
        self.owner_data.get_or_insert_with(|| horizon_mod_sdk::server::world_data::get(OWNERS_KEY).map(|d| decode_owners(&d)).unwrap_or_default())
    }

    pub(crate) fn security_owners_save(&mut self) {
        let bytes = encode_owners(self.security_owners());
        let _ = horizon_mod_sdk::server::world_data::set(OWNERS_KEY, &bytes);
    }

    pub(crate) fn block_owners_get(&mut self, key: &(String, i32, i32, i32)) -> Option<Entry> {
        self.security_registry().get(key).cloned()
    }

    fn security_save(&mut self) {
        let bytes = encode(self.security_registry());
        let _ = horizon_mod_sdk::server::world_data::set(KEY, &bytes);
    }

    /// A block of the mod is placed: its placer is its owner, and it starts public.
    pub(crate) fn security_placed(&mut self, player: &Player, pos: BlockPos, block: &str) {
        if data_index(block).is_none() {
            return;
        }
        let key = (player.dimension.clone(), pos.x, pos.y, pos.z);
        self.security_registry().insert(key, Entry { owner: player.id.clone(), owner_name: player.name.clone(), mode: SecurityMode::Public });
        self.security_save();
    }

    /// A secured block is gone: its entry goes with it.
    pub(crate) fn security_broken(&mut self, player: &Player, pos: BlockPos, block: &str) {
        if data_index(block).is_some() && self.security_registry().remove(&(player.dimension.clone(), pos.x, pos.y, pos.z)).is_some() {
            self.security_save();
        }
    }

    /// A click on a block: false (and a message) when it is private and the player is not its owner.
    pub(crate) fn security_allows(&mut self, player: &Player, pos: BlockPos, block: &str) -> bool {
        if data_index(block).is_none() {
            return true;
        }
        let Some(entry) = self.security_registry().get(&(player.dimension.clone(), pos.x, pos.y, pos.z)).cloned() else { return true };
        let owner = self.security_owners().get(&entry.owner).cloned();
        if can_access(&entry, &player.id, &player.name, owner.as_ref()) {
            return true;
        }
        let _ = players::tell(&player.id, &format!("Access denied: this block belongs to {}", entry.owner_name));
        false
    }

    /// The tab's click on a screen: the owner changes the mode, which the screen shows from the next update.
    pub(crate) fn security_button(&mut self, player: &Player, e: &BlockEntity, id: i32) -> bool {
        if id != NEXT && id != PREVIOUS {
            return false;
        }
        let key = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
        if let Some(entry) = self.security_registry().get_mut(&key).filter(|en| en.owner == player.id) {
            entry.mode = if id == NEXT { entry.mode.next() } else { entry.mode.previous() };
            self.security_dirty.insert(key);
            self.security_save();
        }
        true
    }

    /// Before the machines' ticks: the screens' data as the machines made it, without the mode at its end.
    pub(crate) fn security_strip(&mut self, entities: &mut [BlockEntity]) {
        for e in entities.iter_mut() {
            if let Some(at) = data_index(e.state.split('[').next().unwrap_or("")) {
                e.menu_data.truncate(at);
            }
        }
    }

    /// After them: the mode at the end of each screen's data that was sent, and the screens whose mode just changed.
    pub(crate) fn security_stamp(&mut self, entities: &[BlockEntity], out: &mut Vec<BlockEntityUpdate>) {
        let dirty = std::mem::take(&mut self.security_dirty);
        for e in entities {
            let Some(at) = data_index(e.state.split('[').next().unwrap_or("")) else { continue };
            let key = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
            let mode = self.security_registry().get(&key).map_or(SecurityMode::Public, |en| en.mode);
            if let Some(u) = out.iter_mut().find(|u| u.dimension == e.dimension && u.pos == e.pos) {
                if let Some(data) = u.menu_data.take() {
                    u.menu_data = Some(with_mode(data, at, mode));
                } else if dirty.contains(&key) {
                    u.menu_data = Some(with_mode(e.menu_data.clone(), at, mode));
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
                    menu_data: Some(with_mode(e.menu_data.clone(), at, mode)),
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
    fn the_registry_survives_its_text() {
        let mut m = HashMap::new();
        m.insert(("minecraft:overworld".to_owned(), 1, -2, 3), Entry { owner: "id-1".into(), owner_name: "Steve".into(), mode: SecurityMode::Private });
        assert_eq!(decode(&encode(&m)), m);
    }

    #[test]
    fn a_private_block_is_the_owners_alone() {
        let mut e = Entry { owner: "a".into(), owner_name: "A".into(), mode: SecurityMode::Public };
        assert!(can_access(&e, "b", "B", None));
        e.mode = SecurityMode::Private;
        assert!(can_access(&e, "a", "A", None) && !can_access(&e, "b", "B", None));
        e.mode = SecurityMode::Trusted;
        assert!(!can_access(&e, "b", "B", None));
        let desk = OwnerData { name: "A".into(), mode: SecurityMode::Private, overridden: false, trusted: vec!["bob".into()] };
        assert!(can_access(&e, "b", "Bob", Some(&desk)), "a trusted name goes in");
        assert!(!can_access(&e, "c", "Carl", Some(&desk)));
        e.mode = SecurityMode::Public;
        let forced = OwnerData { overridden: true, ..desk };
        assert!(!can_access(&e, "c", "Carl", Some(&forced)), "the override makes the desk's mode the block's");
    }

    #[test]
    fn the_mode_goes_at_the_place_after_the_screens_numbers() {
        assert_eq!(with_mode(vec![1, 2], 4, SecurityMode::Private), vec![1, 2, 0, 0, 1]);
        assert_eq!(with_mode(vec![1, 2, 3, 4, 5], 4, SecurityMode::Trusted), vec![1, 2, 3, 4, 2]);
    }
}

#[cfg(test)]
mod owner_tests {
    use super::*;

    #[test]
    fn the_desk_data_survives_its_text() {
        let mut m = HashMap::new();
        m.insert("id-1".to_owned(), OwnerData { name: "Steve".into(), mode: SecurityMode::Trusted, overridden: true, trusted: vec!["Alex".into(), "Bob".into()] });
        m.insert("id-2".to_owned(), OwnerData { name: "Kim".into(), ..Default::default() });
        assert_eq!(decode_owners(&encode_owners(&m)), m);
    }
}

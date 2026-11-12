//! The Security Desk (`TileEntitySecurityDesk`, `GuiSecurityDesk`): the screen where a player keeps, for everything they own, the list of trusted players, the mode and the override. Its owner
//! (who placed it) is the one who changes things; others only see. The trusted names are what a block in trusted mode lets in (`block_security::can_access`), and the override makes the mode set here
//! the mode of all the owner's blocks.
//!
//! The screen's numbers: 0 the mode (0 public, 1 private, 2 trusted), 1 the override, 2 whether the viewer is the desk's owner, 3 how many trusted names, 4 the selected row (-1 none). The strings are
//! the trusted names, one each. Not here: the lock and unlock slots (the ownership of items) and the names of players that never joined (a name is only checked when a player uses a block).
use super::*;
use crate::block_security::OwnerData;
use crate::security::SecurityMode;

pub const BLOCK: &str = "mekanism:security_desk";
/// The ids: the list's rows from `ROW_BASE`, then the buttons and the name box.
pub const ROW_BASE: i32 = 9200;
pub const MAX_ROWS: usize = 60;
pub const REMOVE: i32 = 9270;
pub const OVERRIDE: i32 = 9271;
pub const MODE_PUBLIC: i32 = 9272;
pub const MODE_PRIVATE: i32 = 9273;
pub const MODE_TRUSTED: i32 = 9274;
pub const NAME_BOX: i32 = 9275;

/// A name a player may be added with (`InputValidator.USERNAME`: letters, digits and underscore, up to 16).
pub fn valid_name(name: &str) -> bool {
    !name.is_empty() && name.len() <= 16 && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// What a press does to the owner's data; true when something changed. `selected` is the chosen row of the list.
pub fn press(data: &mut OwnerData, id: i32, selected: Option<usize>) -> bool {
    match id {
        REMOVE => selected.filter(|i| *i < data.trusted.len()).map(|i| data.trusted.remove(i)).is_some(),
        OVERRIDE => {
            data.overridden = !data.overridden;
            true
        }
        MODE_PUBLIC | MODE_PRIVATE | MODE_TRUSTED => {
            let mode = SecurityMode::from_u8((id - MODE_PUBLIC) as u8);
            let changed = data.mode != mode;
            data.mode = mode;
            changed
        }
        _ => false,
    }
}

/// Adds the name to the trusted list: false for an invalid or a repeated one (`SecurityFrequency.addTrusted`).
pub fn add_trusted(data: &mut OwnerData, name: &str) -> bool {
    if !valid_name(name) || data.trusted.iter().any(|t| t.eq_ignore_ascii_case(name)) {
        return false;
    }
    data.trusted.push(name.to_owned());
    true
}

impl Mekanism {
    /// A tick of the desks: what was pressed or typed since the last one, then each desk's numbers and strings. The desk's owner is the block's owner in the registry.
    pub(crate) fn run_security_desks(&mut self, desks: &[&BlockEntity]) -> Vec<BlockEntityUpdate> {
        let presses = std::mem::take(&mut self.desk_presses);
        let typed = std::mem::take(&mut self.desk_texts);
        let mut out = Vec::new();
        for e in desks {
            let key = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
            let owner_id = self.block_owners_get(&key).map(|en| en.owner);
            let Some(owner_id) = owner_id else { continue };
            let mut changed = false;
            for (d, p, id, who) in presses.iter().filter(|(d, p, _, _)| *d == e.dimension && (p.x, p.y, p.z) == (e.pos.x, e.pos.y, e.pos.z)).map(|(d, p, id, w)| (d, p, id, w)) {
                let _ = (d, p);
                if *who != owner_id {
                    continue;
                }
                if (ROW_BASE..ROW_BASE + MAX_ROWS as i32).contains(id) {
                    self.desk_selected.insert(key.clone(), (*id - ROW_BASE) as usize);
                    continue;
                }
                let selected = self.desk_selected.get(&key).copied();
                let data = self.security_owners().entry(owner_id.clone()).or_default();
                if press(data, *id, selected) {
                    changed = true;
                    if *id == REMOVE {
                        self.desk_selected.remove(&key);
                    }
                }
            }
            for (d, p, id, text, who) in typed.iter().filter(|(d, p, _, _, _)| *d == e.dimension && (p.x, p.y, p.z) == (e.pos.x, e.pos.y, e.pos.z)) {
                let _ = (d, p);
                if *id == NAME_BOX && *who == owner_id {
                    let data = self.security_owners().entry(owner_id.clone()).or_default();
                    changed |= add_trusted(data, text.trim());
                }
            }
            if changed {
                self.security_owners_save();
            }
            let data = self.security_owners().get(&owner_id).cloned().unwrap_or_default();
            let viewer_is_owner = i32::from(self.desk_viewer.get(&key).is_some_and(|v| *v == owner_id));
            let selected = self.desk_selected.get(&key).map_or(-1, |i| *i as i32);
            let numbers = vec![data.mode as i32, i32::from(data.overridden), viewer_is_owner, data.trusted.len() as i32, selected];
            let texts: Vec<String> = data.trusted.iter().take(MAX_ROWS).cloned().collect();
            if self.menu_texts.get(&key) != Some(&texts) {
                let _ = world::set_menu_text(&e.dimension, e.pos.x, e.pos.y, e.pos.z, &texts);
                self.menu_texts.insert(key.clone(), texts);
            }
            if e.menu_data != numbers {
                out.push(BlockEntityUpdate {
                    dimension: e.dimension.clone(),
                    pos: e.pos,
                    data: e.data.clone(),
                    items: None,
                    energy: None,
                    energy_faces: None,
                    tanks: None,
                    tank_faces: None,
                    menu_data: Some(numbers),
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

    #[test]
    fn names_are_checked_and_not_repeated() {
        let mut d = OwnerData::default();
        assert!(add_trusted(&mut d, "Alex_1"));
        assert!(!add_trusted(&mut d, "alex_1"), "the same name in other letters");
        assert!(!add_trusted(&mut d, "no spaces"));
        assert!(!add_trusted(&mut d, ""));
        assert!(!add_trusted(&mut d, "waytoolongaplayername"));
        assert_eq!(d.trusted, ["Alex_1"]);
    }

    #[test]
    fn the_buttons_change_the_mode_the_override_and_the_list() {
        let mut d = OwnerData { trusted: vec!["a".into(), "b".into()], ..Default::default() };
        assert!(press(&mut d, MODE_TRUSTED, None) && d.mode == SecurityMode::Trusted);
        assert!(!press(&mut d, MODE_TRUSTED, None), "no change");
        assert!(press(&mut d, OVERRIDE, None) && d.overridden);
        assert!(!press(&mut d, REMOVE, None), "nothing selected");
        assert!(press(&mut d, REMOVE, Some(0)) && d.trusted == ["b"]);
    }
}

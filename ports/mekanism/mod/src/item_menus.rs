//! The screens of items (FORK item menus, `players.open-item-menu`): the Dictionary (`GuiDictionary`) shows the tags of the item put in its slot. The other item screens of the original
//! (Module Tweaker, Portable Teleporter, Seismic Reader, Portable QIO Dashboard) are not here yet.
//!
//! Not in the Dictionary: the tag types other than item (block, block entity, fluid, entity, attribute, potion, effect, enchantment and chemical: the host lists only an item's tags), and a
//! tag's other members.
use super::*;
use horizon_mod_sdk::server::{ItemMenu, ItemMenuUpdate};

pub const DICTIONARY_MENU: &str = "mekanism:dictionary";
pub const DICTIONARY_ITEM: &str = "mekanism:dictionary";
/// The strings of the Dictionary: 0 what it shows, then the tags (at most this many).
pub const DICTIONARY_ROWS: usize = 60;

/// The strings and numbers of the Dictionary for the item in its slot: the first line says what is shown, then each tag of the item, sorted; the number is how many tags.
pub fn dictionary_view(item: Option<&str>, tags: &[String]) -> (Vec<String>, Vec<i32>) {
    let Some(item) = item else { return (vec!["Put an item in the slot".to_owned()], vec![0]) };
    let mut list: Vec<String> = tags.iter().take(DICTIONARY_ROWS).map(|t| format!("#{t}")).collect();
    list.sort();
    let head = if list.is_empty() { format!("{item}: no tags") } else { format!("{item}: {} tags", tags.len()) };
    let n = list.len() as i32;
    list.insert(0, head);
    (list, vec![n])
}

impl Mekanism {
    /// One tick of an open item menu of the mod.
    pub(crate) fn item_menu_tick_of(&mut self, player: &Player, menu: &ItemMenu) -> Option<ItemMenuUpdate> {
        if menu.menu != DICTIONARY_MENU {
            return None;
        }
        let item = menu.items.first().cloned().flatten().map(|s| s.item);
        let tags = item.as_deref().map(recipe_types::item_tags).unwrap_or_default();
        let (texts, data) = dictionary_view(item.as_deref(), &tags);
        let key = (player.id.clone(), menu.menu.clone());
        if self.item_menu_sent.get(&key) == Some(&texts) && menu.menu_data == data {
            return None;
        }
        self.item_menu_sent.insert(key, texts.clone());
        Some(ItemMenuUpdate { items: None, menu_data: Some(data), menu_text: Some(texts), close: false })
    }

    /// A use of the Dictionary opens its screen over the slot it is in.
    pub(crate) fn dictionary_used(&mut self, player: &Player, stack: &ItemStack) -> bool {
        if stack.item != DICTIONARY_ITEM {
            return false;
        }
        let Some((slot, _)) = Self::find_hotbar_slot(&player.id, stack) else { return false };
        self.item_menu_sent.remove(&(player.id.clone(), DICTIONARY_MENU.to_owned()));
        players::open_item_menu(&player.id, slot, DICTIONARY_MENU)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_dictionary_lists_the_tags_of_the_item() {
        let (t, d) = dictionary_view(None, &[]);
        assert_eq!((t.len(), d), (1, vec![0]));
        let (t, d) = dictionary_view(Some("minecraft:iron_ingot"), &["c:ingots/iron".into(), "c:ingots".into()]);
        assert_eq!(t[0], "minecraft:iron_ingot: 2 tags");
        assert_eq!(&t[1..], &["#c:ingots".to_owned(), "#c:ingots/iron".to_owned()]);
        assert_eq!(d, vec![2]);
        let (t, d) = dictionary_view(Some("minecraft:stick"), &[]);
        assert_eq!(t, vec!["minecraft:stick: no tags".to_owned()]);
        assert_eq!(d, vec![0]);
    }
}

//! The Portable QIO Dashboard (`ItemPortableQIODashboard`, `GuiPortableQIODashboard`): the QIO Dashboard's catalogue as an item. A use opens the screen over the slot the item is in (an item menu); the
//! frequency is kept in the item's components (`mekanism:qio_frequency`, the frequency's key in hexadecimal), and the grid, the search, the sort and the frequency selector are the block's
//! (`qio_world.rs`), with the sort and the search kept for each player.
//!
//! Not here: the crafting windows, the Shift-click from the inventory into the frequency, and the item's security and owner.
use super::*;
use crate::frequency::Freq;
use crate::qio_world::{self, DashView, SEARCH_BOX, SORT_DIRECTION, SORT_TYPE};
use horizon_mod_sdk::server::{GridEntry, ItemMenu, ItemMenuUpdate};

pub const ITEM: &str = "mekanism:portable_qio_dashboard";
pub const MENU: &str = "mekanism:portable_qio_dashboard";
const KEY: &str = "mekanism:qio_frequency";

/// The frequency key an item's components hold (empty for none).
pub fn frequency_of(components: &str) -> String {
    let Some(entries) = config::split_components(components) else { return String::new() };
    let Some((_, range)) = entries.into_iter().find(|(k, _)| k == KEY) else { return String::new() };
    qio_world::from_hex(components[range].trim().trim_matches('"')).and_then(|b| String::from_utf8(b).ok()).unwrap_or_default()
}

/// The components with the frequency key put in (or taken out), the rest byte-for-byte.
pub fn with_frequency(components: &str, key: &str) -> String {
    let value = format!("\"{}\"", qio_world::to_hex(key.as_bytes()));
    let empty = components.trim().is_empty() || components.trim() == "{}";
    let entries = if empty { Some(Vec::new()) } else { config::split_components(components) };
    let Some(entries) = entries else { return components.to_owned() };
    if key.is_empty() && !entries.iter().any(|(k, _)| k == KEY) {
        return components.to_owned();
    }
    if empty {
        return format!("{{\"{KEY}\":{value}}}");
    }
    let mut out = components.to_owned();
    if let Some((_, range)) = entries.iter().find(|(k, _)| k == KEY) {
        out.replace_range(range.clone(), &value);
    } else {
        let at = out.rfind('}').unwrap_or(out.len());
        out.insert_str(at, &if entries.is_empty() { format!("\"{KEY}\":{value}") } else { format!(",\"{KEY}\":{value}") });
    }
    out
}

impl Mekanism {
    /// A use of the Portable QIO Dashboard opens its screen over the slot it is in.
    pub(crate) fn portable_qio_used(&mut self, player: &Player, stack: &ItemStack) -> bool {
        if stack.item != ITEM {
            return false;
        }
        let Some((slot, _)) = Self::find_hotbar_slot(&player.id, stack) else { return false };
        self.item_menu_sent.remove(&(player.id.clone(), MENU.to_owned()));
        self.qio_portable_views.remove(&player.id);
        players::open_item_menu(&player.id, slot, MENU)
    }

    /// The frequency of the item after a selector action, written back to the item.
    fn portable_qio_set(&mut self, player: &Player, menu: &ItemMenu, press: Option<i32>, text: Option<(i32, &str)>) {
        let Some(stack) = menu.stack.clone().filter(|s| s.item == ITEM) else { return };
        let now = frequency_of(&stack.components);
        let next = self.qio_apply((&player.id, &player.name), &now, press, text);
        if next != now {
            let written = ItemStack { components: with_frequency(&stack.components, &next), ..stack.clone() };
            let _ = player_items::compare_and_set_slot(&player.id, menu.slot, Some(&stack), Some(&written));
        }
    }

    pub(crate) fn portable_qio_button(&mut self, player: &Player, menu: &ItemMenu, id: i32) -> Option<ItemMenuUpdate> {
        let view = self.qio_portable_views.entry(player.id.clone()).or_insert_with(|| DashView { ascending: true, ..DashView::default() });
        match id {
            SORT_TYPE => view.sort = (view.sort + 1) % 4,
            SORT_DIRECTION => view.ascending = !view.ascending,
            _ => self.portable_qio_set(player, menu, Some(id), None),
        }
        self.item_menu_sent.remove(&(player.id.clone(), menu.menu.clone()));
        None
    }

    pub(crate) fn portable_qio_tick(&mut self, player: &Player, menu: &ItemMenu) -> Option<ItemMenuUpdate> {
        let stack = menu.stack.clone().filter(|s| s.item == ITEM)?;
        let typed: Vec<(i32, String)> = {
            let (mine, rest): (Vec<_>, Vec<_>) = std::mem::take(&mut self.portable_texts).into_iter().partition(|t| t.0 == player.id);
            self.portable_texts = rest;
            mine.into_iter().map(|t| (t.1, t.2)).collect()
        };
        for (id, text) in &typed {
            if *id == SEARCH_BOX {
                let view = self.qio_portable_views.entry(player.id.clone()).or_insert_with(|| DashView { ascending: true, ..DashView::default() });
                view.search = text.chars().take(50).collect();
            } else {
                self.portable_qio_set(player, menu, None, Some((*id, text)));
            }
        }
        // The item as the selector just left it (the menu's copy is one tick behind).
        let current = player_items::get_inventory(&player.id).and_then(|inv| inv.get(menu.slot as usize).cloned().flatten()).filter(|s| s.item == ITEM).unwrap_or(stack);
        let key = frequency_of(&current.components);
        let freq: Option<Freq> = self.qio_find(&key);
        let key = freq.as_ref().map(Freq::key).unwrap_or_default();
        match &freq {
            Some(f) => {
                self.qio_portable_freq.insert(player.id.clone(), f.key());
            }
            None => {
                self.qio_portable_freq.remove(&player.id);
            }
        }
        let mut view = self.qio_portable_views.remove(&player.id).unwrap_or(DashView { ascending: true, ..DashView::default() });
        let rows = self.qio_rows(&key, &view.search, view.sort, view.ascending);
        let grid = (rows != view.sent).then(|| rows.iter().map(|(i, c, n)| GridEntry { item: i.clone(), components: c.clone(), count: *n }).collect::<Vec<_>>());
        if grid.is_some() {
            view.cells = rows.iter().map(|(i, c, _)| (i.clone(), c.clone())).collect();
            view.sent = rows;
        }
        let header = match (&freq, self.qio.frequencies.get(&key)) {
            (Some(f), Some(n)) => format!("Frequency: {} ({}/{})", f.name, n.total_count, n.total_count_capacity),
            _ => "No frequency".to_owned(),
        };
        let (selector, public_n, private_n) = self.qio_selector(&player.id, &key);
        let mut strings = vec![header];
        strings.extend(selector);
        let numbers = vec![i32::from(freq.is_some()), 0, 0, i32::from(view.sort), i32::from(view.ascending), 1, public_n as i32, private_n as i32];
        let sent_key = (player.id.clone(), menu.menu.clone());
        let same = self.item_menu_sent.get(&sent_key) == Some(&strings) && menu.menu_data == numbers;
        self.qio_portable_views.insert(player.id.clone(), view);
        if same && grid.is_none() {
            return None;
        }
        self.item_menu_sent.insert(sent_key, strings.clone());
        Some(ItemMenuUpdate { items: None, menu_data: Some(numbers), menu_text: Some(strings), close: false, menu_grid: grid })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_frequency_key_is_kept_in_the_components_and_the_rest_stays() {
        assert_eq!(frequency_of(""), "");
        let written = with_frequency("{\"horizon:x\":1}", "p:Base \"1\"");
        assert!(written.contains("horizon:x"), "{written}");
        assert_eq!(frequency_of(&written), "p:Base \"1\"");
        assert_eq!(frequency_of(&with_frequency("", "o:id:Mine")), "o:id:Mine");
        assert_eq!(frequency_of(&with_frequency(&written, "")), "");
        assert_eq!(with_frequency("", ""), "");
    }
}

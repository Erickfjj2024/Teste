//! The Portable Teleporter (`ItemPortableTeleporter`, `GuiPortableTeleporter`): an item that holds energy and a teleporter frequency (the same named frequencies as the blocks', `frequency.rs`); its screen
//! (an item menu, API 48) is the frequency selector with a Teleport button, and the button sends the player to the nearest active teleporter of the frequency for the energy of the trip, taken from the item.
//!
//! The frequency is kept in the item's components (`mekanism:frequency`, as hexadecimal of `frequency::encode`), its energy in the host's item energy (API 51, `horizon:energy`).
//! Not here: the item's security and owner, the hover text with the frequency, the colour of the frequency, and the delay before the trip (the original's config is 0).
use super::*;
use horizon_mod_sdk::server::{ItemMenu, ItemMenuUpdate};

pub const ITEM: &str = "mekanism:portable_teleporter";
pub const MENU: &str = "mekanism:portable_teleporter";
/// The ids of the frequency selector's buttons and boxes (`frequency::button_action`), and of the Teleport button after them.
pub const FREQ_BASE: i32 = 8300;
pub const BUTTON_TELEPORT: i32 = 8390;
/// `gear.portableTeleporterMaxEnergy` (1,000,000 J) and `portableTeleporterChargeRate` (5,000 J), as Forge Energy (2/5 of a Joule).
pub const CAPACITY: u64 = 400_000;
pub const CHARGE_RATE: u64 = 2_000;
const KEY: &str = "mekanism:frequency";

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn from_hex(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 {
        return None;
    }
    (0..text.len()).step_by(2).map(|i| u8::from_str_radix(text.get(i..i + 2)?, 16).ok()).collect()
}

/// The frequency a stack's components hold.
pub fn frequency_of(components: &str) -> Option<frequency::Freq> {
    let entries = config::split_components(components)?;
    let (_, range) = entries.into_iter().find(|(k, _)| k == KEY)?;
    let bytes = from_hex(components[range].trim().trim_matches('"'))?;
    frequency::decode(&bytes, 0).0
}

/// The components with the frequency replaced (or put in, or taken out), the rest byte-for-byte.
pub fn with_frequency(components: &str, freq: &Option<frequency::Freq>) -> String {
    let mut bytes = Vec::new();
    frequency::encode(freq, &mut bytes);
    let value = format!("\"{}\"", to_hex(&bytes));
    if components.trim().is_empty() || components.trim() == "{}" {
        return format!("{{\"{KEY}\":{value}}}");
    }
    let Some(entries) = config::split_components(components) else { return components.to_owned() };
    let mut out = components.to_owned();
    if let Some((_, range)) = entries.iter().find(|(k, _)| k == KEY) {
        out.replace_range(range.clone(), &value);
    } else {
        let at = out.rfind('}').unwrap_or(out.len());
        out.insert_str(at, &if entries.is_empty() { format!("\"{KEY}\":{value}") } else { format!(",\"{KEY}\":{value}") });
    }
    out
}

/// 0 no frequency, 2 no destination, 3 not enough energy, 4 ready (the teleporter block's numbers, without 1 "no frame").
pub fn status_of(has_frequency: bool, destinations: usize, energy: u64, cost: u64) -> i32 {
    if !has_frequency {
        0
    } else if destinations == 0 {
        2
    } else if energy < cost {
        3
    } else {
        4
    }
}

impl Mekanism {
    /// A use of the Portable Teleporter opens its screen over the slot it is in.
    pub(crate) fn portable_teleporter_used(&mut self, player: &Player, stack: &ItemStack) -> bool {
        if stack.item != ITEM {
            return false;
        }
        let Some((slot, _)) = Self::find_hotbar_slot(&player.id, stack) else { return false };
        self.item_menu_sent.remove(&(player.id.clone(), MENU.to_owned()));
        players::open_item_menu(&player.id, slot, MENU)
    }

    /// What the screen shows for `stack` held by `player`: the strings, the numbers, and the nearest destination with the cost of the trip.
    fn portable_view(&mut self, player: &Player, stack: &ItemStack, freq: &Option<frequency::Freq>) -> (Vec<String>, Vec<i32>, Option<(String, [i32; 3], Option<usize>, u64)>) {
        let known = self.frequencies_known(&[]);
        let listing = frequency::listing(&known, &player.id);
        let others: Vec<(String, [i32; 3], Option<usize>)> = match freq {
            Some(f) => self.teleporter_table.iter().filter(|t| t.0 == f.key()).map(|t| (t.1.clone(), t.2, t.3)).collect(),
            None => Vec::new(),
        };
        let here = [player.x.floor() as i32, player.y.floor() as i32, player.z.floor() as i32];
        let dest = teleporter::closest((&player.dimension, here), &others).cloned();
        let cost = dest.as_ref().map_or(0, |(d, p, _)| teleporter::trip_cost(&player.dimension, [player.x, player.y, player.z], d, *p));
        let (energy, capacity) = recipe_types::item_energy(stack).map_or((0, CAPACITY), |i| (i.stored, i.capacity));
        let status = status_of(freq.is_some(), others.len(), energy, cost);
        let half = |v: u64, s: u32| ((v >> s) & 0xFFFF) as i32;
        let data = vec![
            i32::from(freq.is_some()),
            status,
            half(energy, 0),
            half(energy, 16),
            half(capacity, 0),
            half(capacity, 16),
            others.len() as i32,
            listing.public.len() as i32,
            listing.private.len() as i32,
        ];
        (frequency::texts(freq, &listing), data, dest.map(|(d, p, f)| (d, p, f, cost)))
    }

    /// One tick of the open screen: its frequency lines, numbers and texts, and what a text box confirmed.
    pub(crate) fn portable_teleporter_tick(&mut self, player: &Player, menu: &ItemMenu) -> Option<ItemMenuUpdate> {
        let stack = menu.stack.clone().filter(|s| s.item == ITEM)?;
        let mut stack = stack;
        let texts_in: Vec<(i32, String)> = {
            let (mine, rest): (Vec<_>, Vec<_>) = std::mem::take(&mut self.portable_texts).into_iter().partition(|t| t.0 == player.id);
            self.portable_texts = rest;
            mine.into_iter().map(|t| (t.1, t.2)).collect()
        };
        let mut freq = frequency_of(&stack.components);
        for (id, text) in texts_in {
            if let Some(action) = frequency::text_action(id, FREQ_BASE, &text) {
                freq = self.portable_apply(player, &mut stack, menu.slot, freq, &action);
            }
        }
        let (texts, data, _) = self.portable_view(player, &stack, &freq);
        let key = (player.id.clone(), menu.menu.clone());
        if self.item_menu_sent.get(&key) == Some(&texts) && menu.menu_data == data {
            return None;
        }
        self.item_menu_sent.insert(key, texts.clone());
        Some(ItemMenuUpdate { items: None, menu_data: Some(data), menu_text: Some(texts), close: false })
    }

    /// A frequency action of the screen: the item's frequency after it (the item is written back).
    fn portable_apply(&mut self, player: &Player, stack: &mut ItemStack, slot: u32, freq: Option<frequency::Freq>, action: &frequency::Action) -> Option<frequency::Freq> {
        let known = self.frequencies_known(&[]);
        let Some(next) = frequency::apply(action, &freq, &frequency::listing(&known, &player.id), &player.id, &player.name) else { return freq };
        match &next {
            None => {
                // `FrequencyManager.remove`: the frequency is gone for everything that held it.
                if let Some(f) = &freq {
                    self.frequency_deleted(&f.key());
                }
            }
            Some(made) => self.frequency_made(made),
        }
        let written = ItemStack { components: with_frequency(&stack.components, &next), ..stack.clone() };
        if player_items::compare_and_set_slot(&player.id, slot, Some(&*stack), Some(&written)).is_ok() {
            *stack = written;
        }
        next
    }

    /// A button of the screen: a frequency button or the Teleport button (which, when the trip is ready, takes its energy from the item, sends the player and closes the screen).
    pub(crate) fn portable_teleporter_button(&mut self, player: &Player, menu: &ItemMenu, id: i32) -> Option<ItemMenuUpdate> {
        let mut stack = menu.stack.clone().filter(|s| s.item == ITEM)?;
        let freq = frequency_of(&stack.components);
        if id == BUTTON_TELEPORT {
            let (_, data, dest) = self.portable_view(player, &stack, &freq);
            let (dest_dim, dest_pos, dest_face, cost) = dest?;
            if data.get(1) != Some(&4) {
                return None;
            }
            let (after, took) = recipe_types::item_energy_extract(&stack, cost, false);
            if took < cost || player_items::compare_and_set_slot(&player.id, menu.slot, Some(&stack), Some(&after)).is_err() {
                return None;
            }
            let t = teleporter::target(dest_pos, dest_face);
            let _ = player_control::teleport(&player.id, &dest_dim, f64::from(t[0]) + 0.5, f64::from(t[1]), f64::from(t[2]) + 0.5, None, None);
            return Some(ItemMenuUpdate { items: None, menu_data: None, menu_text: None, close: true });
        }
        let action = frequency::button_action(id, FREQ_BASE)?;
        self.portable_apply(player, &mut stack, menu.slot, freq, &action);
        self.item_menu_sent.remove(&(player.id.clone(), menu.menu.clone()));
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn freq() -> Option<frequency::Freq> {
        Some(frequency::Freq { name: "Base".into(), private: true, owner: "id-1".into(), owner_name: "Steve".into() })
    }

    #[test]
    fn the_frequency_is_kept_in_the_components_and_the_rest_stays() {
        assert_eq!(frequency_of(""), None);
        let fresh = with_frequency("", &freq());
        assert_eq!(frequency_of(&fresh), freq());
        let other = "{\"horizon:energy\":500L}";
        let both = with_frequency(other, &freq());
        assert!(both.contains("\"horizon:energy\":500L"), "{both}");
        assert_eq!(frequency_of(&both), freq());
        let cleared = with_frequency(&both, &None);
        assert_eq!(frequency_of(&cleared), None);
        assert!(cleared.contains("\"horizon:energy\":500L"));
    }

    #[test]
    fn the_status_follows_the_frequency_the_destinations_and_the_energy() {
        assert_eq!(status_of(false, 3, 100, 1), 0);
        assert_eq!(status_of(true, 0, 100, 1), 2);
        assert_eq!(status_of(true, 1, 10, 11), 3);
        assert_eq!(status_of(true, 1, 11, 11), 4);
    }
}

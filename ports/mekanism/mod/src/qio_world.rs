//! The QIO network in the world: the frequency the QIO blocks are set to (`GuiQIOFrequencyTab`), the drive arrays that put their drives into it, and the QIO Dashboard's item catalogue
//! (`GuiQIODashboard`, `QIOItemViewerContainer`), shown with the host's item grid (API 55).
//!
//! A QIO frequency is a named one like the teleporter's (`frequency.rs`: public by name, private by owner and name) with its own registry, kept in the world's data. A block keeps the frequency's
//! key; the network itself (`qio.rs::QIOFrequency`) is made when a block is set to it and lives in memory: every drive array puts its drives in each tick, and the items of a drive are kept in the
//! drive item's components (`mekanism:qio_drive`, the `DriveContents` bytes in hexadecimal), so they travel with the item and survive a restart.
//!
//! The Dashboard: the grid lists the frequency's items (sorted by name, count, mod or registry name, ascending or not, and filtered by the search box: a name, `@mod`, `#tag`). A left click takes a stack,
//! a right click half of one, a middle click one, shift puts it in the inventory, and a click with a stack on the cursor stores it (all of it with the left button, one with the others).
//! Not here: the three crafting windows, the shift-click that sends inventory items into the frequency (the host has no hook for it), the sort and search kept for each player (they are the
//! block's), the `$tooltip` search, the other QIO blocks' tab (importer, exporter, redstone adapter), and the Portable QIO Dashboard.
//! Port addition: the Dashboard has a slot where items are put in until the host sends a click on an empty cell of the grid.
use super::*;
use crate::frequency::{self, Freq};
use crate::qio::*;
use crate::qio_dashboard::{decode_dashboard, encode_dashboard};
use horizon_mod_sdk::server::{GridAnswer, GridEntry, GridTarget};

const KEY: &str = "mekanism:qio_frequencies";
const DRIVE_KEY: &str = "mekanism:qio_drive";
/// The ids of the frequency selector (`frequency.rs`), then the Dashboard's own buttons and its search box.
pub const FREQ_BASE: i32 = 8500;
pub const SORT_TYPE: i32 = 8560;
pub const SORT_DIRECTION: i32 = 8561;
pub const SEARCH_BOX: i32 = 8562;
/// How many entries the grid is given at most (the host's limit).
const GRID_MAX: usize = 4096;

/// What the Dashboard's grid last showed for a block.
#[derive(Clone, Debug, Default)]
pub struct DashView {
    pub sort: u8,
    pub ascending: bool,
    pub search: String,
    /// The items of the grid's cells in order (item, components), to know what a click is on.
    pub cells: Vec<(String, String)>,
    pub sent: Vec<(String, String, u64)>,
}

/// What a drive array last had in each slot, and the frequency its drives are in.
#[derive(Clone, Debug, Default)]
pub struct ArrayCache {
    pub freq: Option<String>,
    pub slots: Vec<Option<(String, String)>>,
}

pub type BlockKey = (String, i32, i32, i32);

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn from_hex(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 {
        return None;
    }
    (0..text.len()).step_by(2).map(|i| u8::from_str_radix(text.get(i..i + 2)?, 16).ok()).collect()
}

/// A drive item's stored contents (`mekanism:qio_drive` in its components).
pub fn drive_contents(components: &str) -> DriveContents {
    let Some(entries) = config::split_components(components) else { return DriveContents::new() };
    let Some((_, range)) = entries.into_iter().find(|(k, _)| k == DRIVE_KEY) else { return DriveContents::new() };
    from_hex(components[range].trim().trim_matches('"')).and_then(|b| DriveContents::decode(&b)).unwrap_or_default()
}

/// The components with the drive's contents put in (or taken out when empty), the rest byte-for-byte.
pub fn with_drive_contents(components: &str, contents: &DriveContents) -> String {
    let value = format!("\"{}\"", to_hex(&contents.encode()));
    let empty = components.trim().is_empty() || components.trim() == "{}";
    let entries = if empty { Some(Vec::new()) } else { config::split_components(components) };
    let Some(entries) = entries else { return components.to_owned() };
    if contents.is_empty() && !entries.iter().any(|(k, _)| k == DRIVE_KEY) {
        return components.to_owned();
    }
    if empty {
        return format!("{{\"{DRIVE_KEY}\":{value}}}");
    }
    let mut out = components.to_owned();
    if let Some((_, range)) = entries.iter().find(|(k, _)| k == DRIVE_KEY) {
        out.replace_range(range.clone(), &value);
    } else {
        let at = out.rfind('}').unwrap_or(out.len());
        out.insert_str(at, &if entries.is_empty() { format!("\"{DRIVE_KEY}\":{value}") } else { format!(",\"{DRIVE_KEY}\":{value}") });
    }
    out
}

fn hashed(item: &str, components: &str) -> HashedItem {
    HashedItem::with_components(item, components.as_bytes())
}

/// The registry name of the item the grid shows for a stored one: the item and the components as strings.
fn cell_of(h: &HashedItem) -> (String, String) {
    (h.id.clone(), String::from_utf8_lossy(&h.components).into_owned())
}

/// The sort type's name on the screen.
pub const SORT_NAMES: [&str; 4] = ["Name", "Size", "Mod", "Registry"];

/// `ListSortType.sort`: the rows ordered by the sort type and direction.
pub fn sort_rows(rows: &mut [(String, String, u64)], sort: u8, ascending: bool) {
    let name = |r: &(String, String, u64)| dynamic::display_name(&r.0);
    let namespace = |r: &(String, String, u64)| r.0.split(':').next().unwrap_or("").to_owned();
    rows.sort_by(|a, b| {
        match sort {
            1 => {
                let by = a.2.cmp(&b.2);
                (if ascending { by } else { by.reverse() }).then_with(|| name(a).cmp(&name(b)))
            }
            2 => {
                let by = namespace(a).cmp(&namespace(b));
                (if ascending { by } else { by.reverse() }).then_with(|| name(a).cmp(&name(b)))
            }
            3 => {
                let by = a.0.cmp(&b.0);
                (if ascending { by } else { by.reverse() }).then_with(|| a.2.cmp(&b.2))
            }
            _ => {
                let by = name(a).cmp(&name(b));
                if ascending { by } else { by.reverse() }
            }
        }
    });
}

/// The search (`SearchQueryParser`): a plain word matches the item's name, `@word` its mod, `#word` one of its tags. Several queries are separated by `|`, and any of them may match.
pub fn matches_search(query: &str, item: &str, tags: &dyn Fn(&str) -> Vec<String>) -> bool {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return true;
    }
    query.split('|').map(str::trim).filter(|q| !q.is_empty()).any(|q| {
        if let Some(m) = q.strip_prefix('@') {
            item.split(':').next().unwrap_or("").to_lowercase().contains(m)
        } else if let Some(t) = q.strip_prefix('#') {
            tags(item).iter().any(|tag| tag.to_lowercase().contains(t))
        } else {
            dynamic::display_name(item).to_lowercase().contains(q)
        }
    })
}

impl Mekanism {
    fn qio_registry(&mut self) -> &mut Vec<Freq> {
        self.qio_freqs.get_or_insert_with(|| horizon_mod_sdk::server::world_data::get(KEY).map(|d| frequency::decode_registry(&d)).unwrap_or_default())
    }

    fn qio_save(&mut self) {
        let bytes = frequency::encode_registry(self.qio_registry());
        let _ = horizon_mod_sdk::server::world_data::set(KEY, &bytes);
    }

    /// The frequency a block holds by its key, when the registry still has it.
    fn qio_find(&mut self, key: &str) -> Option<Freq> {
        if key.is_empty() {
            return None;
        }
        self.qio_registry().iter().find(|f| f.key() == key).cloned()
    }

    /// The network of a frequency, made when it is first asked for.
    fn qio_network(&mut self, f: &Freq) -> &mut QIOFrequency {
        self.qio.frequencies.entry(f.key()).or_insert_with(|| QIOFrequency::new(&f.name, None, &f.owner_name, SecurityMode::Public))
    }

    /// A press or a confirmed box of a frequency selector: the key of the frequency the block is set to after it.
    fn qio_apply(&mut self, player: (&str, &str), current: &str, press: Option<i32>, text: Option<(i32, &str)>) -> String {
        let known = self.qio_registry().clone();
        let listing = frequency::listing(&known, player.0);
        let cur = self.qio_find(current);
        let action = match (press, text) {
            (Some(id), _) => frequency::button_action(id, FREQ_BASE),
            (_, Some((id, t))) => frequency::text_action(id, FREQ_BASE, t),
            _ => None,
        };
        let Some(action) = action else { return current.to_owned() };
        let Some(next) = frequency::apply(&action, &cur, &listing, player.0, player.1) else { return current.to_owned() };
        match next {
            None => {
                if let Some(f) = cur {
                    self.qio_registry().retain(|r| r.key() != f.key());
                    self.qio.frequencies.remove(&f.key());
                    self.qio_save();
                }
                String::new()
            }
            Some(f) => {
                if !self.qio_registry().iter().any(|r| r.key() == f.key()) {
                    self.qio_registry().push(f.clone());
                    self.qio_save();
                }
                self.qio_network(&f);
                f.key()
            }
        }
    }

    /// The strings of a frequency selector and how many public and private frequencies it lists.
    fn qio_selector(&mut self, viewer: &str, current: &str) -> (Vec<String>, usize, usize) {
        let known = self.qio_registry().clone();
        let listing = frequency::listing(&known, viewer);
        let cur = self.qio_find(current);
        (frequency::texts(&cur, &listing), listing.public.len(), listing.private.len())
    }

    /// The viewers of a block: the player who pressed or typed last (a selector shows what that player may choose from).
    fn qio_viewer(&self, key: &BlockKey) -> String {
        self.qio_viewers.get(key).cloned().unwrap_or_default()
    }

    /// One tick of the QIO blocks: the buttons and boxes pressed since the last, then the drive arrays and the Dashboards.
    pub(crate) fn run_qio(&mut self, arrays: &[&BlockEntity], boards: &[&BlockEntity], tick: u64) -> Vec<BlockEntityUpdate> {
        let presses = std::mem::take(&mut self.qio_buttons);
        let texts = std::mem::take(&mut self.qio_texts);
        for (d, p, _, who, _) in &presses {
            self.qio_viewers.insert((d.clone(), p.x, p.y, p.z), who.clone());
        }
        for (d, p, _, _, who, _) in &texts {
            self.qio_viewers.insert((d.clone(), p.x, p.y, p.z), who.clone());
        }
        let mut out = Vec::new();
        let mut seen: std::collections::HashSet<BlockKey> = std::collections::HashSet::new();
        for e in arrays {
            seen.insert((e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z));
            out.extend(self.qio_array(e, &presses, &texts, tick));
        }
        // A drive array that is not in the world any more (broken, or its chunk unloaded) takes its drives out of the networks.
        for f in self.qio.frequencies.values_mut() {
            let gone: Vec<DriveKey> = f.drive_map.keys().filter(|k| !seen.contains(&(k.dimension.clone(), k.x, k.y, k.z))).cloned().collect();
            for k in gone {
                f.remove_drive(&k, true);
            }
        }
        self.qio_arrays.retain(|k, _| seen.contains(k));
        for e in boards {
            out.extend(self.qio_board(e, &presses, &texts));
        }
        out
    }

    fn qio_array(&mut self, e: &BlockEntity, presses: &[(String, BlockPos, i32, String, String)], texts: &[(String, BlockPos, i32, String, String, String)], tick: u64) -> Option<BlockEntityUpdate> {
        let key: BlockKey = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
        let mut state = decode_drive_array(&e.data).unwrap_or_default();
        let mut current = state.frequency_name.clone().unwrap_or_default();
        for (d, p, id, who, name) in presses {
            if *d == e.dimension && *p == e.pos {
                current = self.qio_apply((who, name), &current, Some(*id), None);
            }
        }
        for (d, p, id, text, who, name) in texts {
            if *d == e.dimension && *p == e.pos {
                current = self.qio_apply((who, name), &current, None, Some((*id, text)));
            }
        }
        let freq = self.qio_find(&current);
        let current = freq.as_ref().map(Freq::key).unwrap_or_default();
        state.frequency_name = (!current.is_empty()).then(|| current.clone());
        let cache = self.qio_arrays.entry(key.clone()).or_default();
        cache.slots.resize(DRIVE_SLOTS, None);
        // The frequency changed: this array's drives leave the old one.
        if cache.freq.as_deref().unwrap_or("") != current {
            if let Some(old) = cache.freq.take() {
                if let Some(f) = self.qio.frequencies.get_mut(&old) {
                    for slot in 0..DRIVE_SLOTS {
                        f.remove_drive(&DriveKey { dimension: key.0.clone(), x: key.1, y: key.2, z: key.3, slot }, true);
                    }
                }
            }
            let cache = self.qio_arrays.get_mut(&key)?;
            cache.freq = freq.as_ref().map(Freq::key);
            cache.slots = vec![None; DRIVE_SLOTS];
        }
        let mut items: Vec<Option<ItemStack>> = (0..DRIVE_SLOTS).map(|i| e.items.get(i).cloned().flatten()).collect();
        let mut items_changed = false;
        if let Some(f) = &freq {
            self.qio_network(f);
            let net = self.qio.frequencies.get_mut(&f.key())?;
            let cache = self.qio_arrays.get_mut(&key)?;
            for slot in 0..DRIVE_SLOTS {
                let dk = DriveKey { dimension: key.0.clone(), x: key.1, y: key.2, z: key.3, slot };
                let drive = items[slot].as_ref().and_then(|s| QIODriveTier::from_item(&s.item).map(|t| (s.clone(), t)));
                match drive {
                    None => {
                        net.remove_drive(&dk, true);
                        cache.slots[slot] = None;
                        state.drives[slot] = None;
                    }
                    Some((stack, tier)) => {
                        let identity = (stack.item.clone(), stack.components.clone());
                        if cache.slots[slot].as_ref() != Some(&identity) || !net.drive_map.contains_key(&dk) {
                            // A new drive (or one that was changed from outside): its stored items join the network.
                            net.add_drive(dk.clone(), QIODriveData::with_contents(slot, tier, drive_contents(&stack.components)));
                        } else if let Some(data) = net.drive_map.get(&dk) {
                            // The network changed it (a stack taken or stored): the item keeps the new contents.
                            let now = data.to_contents();
                            if sorted(&now) != sorted(&drive_contents(&stack.components)) {
                                let written = ItemStack { components: with_drive_contents(&stack.components, &now), ..stack.clone() };
                                items[slot] = Some(written);
                                items_changed = true;
                            }
                        }
                        let stack = items[slot].clone().unwrap_or(stack);
                        cache.slots[slot] = Some((stack.item.clone(), stack.components.clone()));
                        state.drives[slot] = net.drive_map.get(&dk).map(|d| (tier, d.to_contents()));
                    }
                }
            }
        } else {
            for slot in 0..DRIVE_SLOTS {
                state.drives[slot] = items[slot].as_ref().and_then(|s| QIODriveTier::from_item(&s.item).map(|t| (t, drive_contents(&s.components))));
            }
        }
        let net = freq.as_ref().and_then(|f| self.qio.frequencies.get(&f.key()));
        let changed = update_drive_array_status(&mut state, net);
        let (mut numbers, mut strings) = drive_array_screen(&state, net);
        let viewer = self.qio_viewer(&key);
        let (selector, public_n, private_n) = self.qio_selector(&viewer, &current);
        numbers.extend([public_n as i32, private_n as i32]);
        strings.extend(selector);
        if self.qio.menu_texts.get(&key) != Some(&strings) {
            let _ = world::set_menu_text(&e.dimension, e.pos.x, e.pos.y, e.pos.z, &strings);
            self.qio.menu_texts.insert(key, strings);
        }
        if changed || items_changed || tick % TICKS_PER_HALF_SECOND == 0 || e.menu_data != numbers {
            let active = if state.active { "true" } else { "false" };
            let _ = world::set_block_property(&e.dimension, e.pos.x, e.pos.y, e.pos.z, "active", active);
            return Some(BlockEntityUpdate {
                dimension: e.dimension.clone(),
                pos: e.pos,
                data: encode_drive_array(&state),
                items: items_changed.then_some(items),
                energy: None,
                energy_faces: None,
                tanks: None,
                tank_faces: None,
                menu_data: Some(numbers),
                transmitter_faces: None,
            });
        }
        None
    }

    fn qio_board(&mut self, e: &BlockEntity, presses: &[(String, BlockPos, i32, String, String)], texts: &[(String, BlockPos, i32, String, String, String)]) -> Option<BlockEntityUpdate> {
        let key: BlockKey = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
        let mut state = decode_dashboard(&e.data).unwrap_or_default();
        let before = state.clone();
        let mut view = self.qio_views.remove(&key).unwrap_or(DashView { ascending: true, ..DashView::default() });
        let mut current = state.frequency_name.clone();
        for (d, p, id, who, name) in presses {
            if *d != e.dimension || *p != e.pos {
                continue;
            }
            match *id {
                SORT_TYPE => view.sort = (view.sort + 1) % 4,
                SORT_DIRECTION => view.ascending = !view.ascending,
                crate::qio_dashboard::TOGGLE_SHIFT_CLICK_DIRECTION => state.insert_into_frequency = !state.insert_into_frequency,
                _ => current = self.qio_apply((who, name), &current, Some(*id), None),
            }
        }
        for (d, p, id, text, who, name) in texts {
            if *d != e.dimension || *p != e.pos {
                continue;
            }
            if *id == SEARCH_BOX {
                view.search = text.chars().take(50).collect();
            } else {
                current = self.qio_apply((who, name), &current, None, Some((*id, text)));
            }
        }
        let freq = self.qio_find(&current);
        state.frequency_name = freq.as_ref().map(Freq::key).unwrap_or_default();
        match &freq {
            Some(f) => self.qio_board_freq.insert(key.clone(), f.key()),
            None => self.qio_board_freq.remove(&key),
        };
        let mut items: Vec<Option<ItemStack>> = vec![e.items.first().cloned().flatten()];
        let mut items_changed = false;
        let mut rows: Vec<(String, String, u64)> = Vec::new();
        if let Some(f) = &freq {
            self.qio_network(f);
            let net = self.qio.frequencies.get_mut(&f.key())?;
            // The slot: what is put in goes to the network, and what does not fit stays.
            if let Some(stack) = items[0].clone() {
                let rejected = net.add_item(&hashed(&stack.item, &stack.components), u64::from(stack.count), true);
                if rejected != u64::from(stack.count) {
                    items[0] = (rejected > 0).then(|| ItemStack { count: rejected as u32, ..stack });
                    items_changed = true;
                }
            }
            let tags = |item: &str| recipe_types::item_tags(item);
            rows = net.item_data_map.values().map(|d| { let (i, c) = cell_of(&d.item); (i, c, d.count) }).filter(|r| matches_search(&view.search, &r.0, &tags)).collect();
            sort_rows(&mut rows, view.sort, view.ascending);
            rows.truncate(GRID_MAX);
        }
        if rows != view.sent {
            let entries: Vec<GridEntry> = rows.iter().map(|(i, c, n)| GridEntry { item: i.clone(), components: c.clone(), count: *n }).collect();
            let _ = world::set_menu_grid(&e.dimension, e.pos.x, e.pos.y, e.pos.z, &entries);
            view.cells = rows.iter().map(|(i, c, _)| (i.clone(), c.clone())).collect();
            view.sent = rows;
        }
        let net = freq.as_ref().and_then(|f| self.qio.frequencies.get(&f.key()));
        let header = match (&freq, net) {
            (Some(f), Some(n)) => format!("Frequency: {} ({}/{})", f.name, n.total_count, n.total_count_capacity),
            _ => "No frequency".to_owned(),
        };
        let viewer = self.qio_viewer(&key);
        let (selector, public_n, private_n) = self.qio_selector(&viewer, &state.frequency_name);
        let mut strings = vec![header];
        strings.extend(selector);
        let numbers = vec![i32::from(freq.is_some()), 0, 0, i32::from(view.sort), i32::from(view.ascending), i32::from(state.insert_into_frequency), public_n as i32, private_n as i32];
        self.qio_views.insert(key.clone(), view);
        if self.qio.menu_texts.get(&key) != Some(&strings) {
            let _ = world::set_menu_text(&e.dimension, e.pos.x, e.pos.y, e.pos.z, &strings);
            self.qio.menu_texts.insert(key, strings);
        }
        if state != before || items_changed || e.menu_data.get(..numbers.len()) != Some(&numbers[..]) {
            return Some(BlockEntityUpdate {
                dimension: e.dimension.clone(),
                pos: e.pos,
                data: encode_dashboard(&state),
                items: items_changed.then_some(items),
                energy: None,
                energy_faces: None,
                tanks: None,
                tank_faces: None,
                menu_data: Some(numbers),
                transmitter_faces: None,
            });
        }
        None
    }

    /// A press on a cell of the Dashboard's grid (`QIOItemViewerContainer.onClick`): the cursor's stack after, and stacks for the inventory.
    pub(crate) fn qio_grid_click(&mut self, target: &GridTarget, index: u32, button: u8, shift: bool, carried: Option<ItemStack>) -> Option<GridAnswer> {
        let pos = target.pos.as_ref()?;
        let key: BlockKey = (target.dimension.clone(), pos.x, pos.y, pos.z);
        let board_key = self.qio_board_freq.get(&key).cloned()?;
        let (item, components) = self.qio_views.get(&key)?.cells.get(index as usize).cloned()?;
        let net = self.qio.frequencies.get_mut(&board_key)?;
        let h = hashed(&item, &components);
        let max = bin::stack_size(&item).max(1);
        if shift {
            let have = net.get_stored(&h);
            let taken = net.remove_item(&h, have.min(max), true);
            let stack = ItemStack { item, count: taken as u32, components };
            return (taken > 0).then(|| GridAnswer { carried, to_inventory: vec![stack] });
        }
        match carried {
            None => {
                let have = net.get_stored(&h);
                let top = have.min(max);
                let want = match button {
                    0 => top,
                    2 => 1,
                    _ => (top / 2).max(1),
                };
                let taken = net.remove_item(&h, want, true);
                (taken > 0).then(|| GridAnswer { carried: Some(ItemStack { item, count: taken as u32, components }), to_inventory: Vec::new() })
            }
            Some(c) => {
                if button == 2 && c.item == item && c.components == components {
                    // `button == MIDDLE` with a matching stack in hand: one more from the network.
                    let taken = net.remove_item(&h, 1, true);
                    let room = max.saturating_sub(u64::from(c.count));
                    if taken > 0 && room > 0 {
                        return Some(GridAnswer { carried: Some(ItemStack { count: c.count + 1, ..c }), to_inventory: Vec::new() });
                    }
                    if taken > 0 {
                        net.add_item(&h, taken, true);
                    }
                    return None;
                }
                let to_add = if button == 0 { u64::from(c.count) } else { 1 };
                let rejected = net.add_item(&hashed(&c.item, &c.components), to_add, true);
                let placed = to_add - rejected;
                if placed == 0 {
                    return None;
                }
                let left = u64::from(c.count) - placed;
                Some(GridAnswer { carried: (left > 0).then(|| ItemStack { count: left as u32, ..c }), to_inventory: Vec::new() })
            }
        }
    }
}

fn sorted(c: &DriveContents) -> Vec<(HashedItem, u64)> {
    let mut v = c.items.clone();
    v.sort();
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(i: &str, n: u64) -> (String, String, u64) {
        (i.to_owned(), String::new(), n)
    }

    #[test]
    fn a_drive_keeps_its_contents_in_the_components_and_the_rest_stays() {
        let mut c = DriveContents::new();
        c.items.push((HashedItem::raw("minecraft:cobblestone"), 4000));
        let written = with_drive_contents("{\"horizon:energy\":5L}", &c);
        assert!(written.contains("horizon:energy"), "{written}");
        assert_eq!(drive_contents(&written), c);
        assert_eq!(drive_contents(&with_drive_contents("", &c)), c);
        let emptied = with_drive_contents(&written, &DriveContents::new());
        assert!(drive_contents(&emptied).is_empty());
        assert_eq!(with_drive_contents("", &DriveContents::new()), "", "nothing to keep, nothing written");
    }

    #[test]
    fn the_four_sorts_go_both_ways() {
        let mut rows = vec![row("minecraft:stone", 5), row("mekanism:steel_ingot", 9), row("minecraft:apple", 5)];
        sort_rows(&mut rows, 0, true);
        assert_eq!(rows.iter().map(|r| r.0.as_str()).collect::<Vec<_>>(), ["minecraft:apple", "mekanism:steel_ingot", "minecraft:stone"]);
        sort_rows(&mut rows, 0, false);
        assert_eq!(rows[0].0, "minecraft:stone");
        sort_rows(&mut rows, 1, true);
        assert_eq!(rows.iter().map(|r| r.2).collect::<Vec<_>>(), [5, 5, 9]);
        assert_eq!(rows[0].0, "minecraft:apple", "equal counts go by name");
        sort_rows(&mut rows, 1, false);
        assert_eq!(rows[0].2, 9);
        sort_rows(&mut rows, 2, true);
        assert_eq!(rows[0].0, "mekanism:steel_ingot");
        sort_rows(&mut rows, 3, false);
        assert_eq!(rows[0].0, "minecraft:stone");
    }

    #[test]
    fn the_search_matches_names_mods_and_tags() {
        let tags = |i: &str| if i == "minecraft:iron_ingot" { vec!["c:ingots/iron".to_owned()] } else { Vec::new() };
        assert!(matches_search("", "minecraft:stone", &tags));
        assert!(matches_search("iron", "minecraft:iron_ingot", &tags));
        assert!(!matches_search("gold", "minecraft:iron_ingot", &tags));
        assert!(matches_search("@mekanism", "mekanism:steel_ingot", &tags));
        assert!(!matches_search("@mekanism", "minecraft:stone", &tags));
        assert!(matches_search("#ingots", "minecraft:iron_ingot", &tags));
        assert!(matches_search("stone | iron", "minecraft:iron_ingot", &tags));
    }
}

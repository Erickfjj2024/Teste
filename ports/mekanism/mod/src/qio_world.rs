//! The QIO network in the world: the frequency the QIO blocks are set to (`GuiQIOFrequencyTab`), the drive arrays that put their drives into it, and the QIO Dashboard's item catalogue
//! (`GuiQIODashboard`, `QIOItemViewerContainer`), shown with the host's item grid (API 55).
//!
//! A QIO frequency is a named one like the teleporter's (`frequency.rs`: public by name, private by owner and name) with its own registry, kept in the world's data. A block keeps the frequency's
//! key; the network itself (`qio.rs::QIOFrequency`) is made when a block is set to it and lives in memory: every drive array puts its drives in each tick, and the items of a drive are kept in the
//! drive item's components (`mekanism:qio_drive`, the `DriveContents` bytes in hexadecimal), so they travel with the item and survive a restart.
//!
//! The Dashboard: the grid lists the frequency's items (sorted by name, count, mod or registry name, ascending or not, and filtered by the search box: a name, `@mod`, `#tag`). A left click takes a stack,
//! a right click half of one, a middle click one, shift puts it in the inventory, and a click with a stack on the cursor stores it (all of it with the left button, one with the others), on an entry or on an empty cell (`index` 4294967295).
//! The three crafting windows (`QIOCraftingWindow`): each a 3 by 3 grid and a result slot that shows what the grid makes; taking the result uses one of each ingredient (or leaves what the recipe
//! leaves, a bucket), and a cell that runs out is filled again from the frequency; the clear button sends the grid's stacks to the frequency.
//! Not here: the shift-click that crafts a whole stack from the frequency, the recipe viewer's transfer, the shift-click that sends inventory items into the frequency (the host has no hook for it), the sort and search kept for each player (they are the
//! block's), the `$tooltip` search, the other QIO blocks' tab (importer, exporter, redstone adapter), and the Portable QIO Dashboard.
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

pub(crate) fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub(crate) fn from_hex(text: &str) -> Option<Vec<u8>> {
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

/// The Dashboard's crafting windows: three, each nine ingredient slots then the result (`IQIOCraftingWindowHolder.MAX_CRAFTING_WINDOWS`), 30 slots in all.
pub const WINDOWS: usize = 3;
pub const PER_WINDOW: usize = 10;
pub const WINDOW_SLOTS: usize = WINDOWS * PER_WINDOW;

fn same_stack(a: &ItemStack, b: &ItemStack) -> bool {
    a.item == b.item && a.components == b.components
}

/// A craft's effect on the grid: each cell loses one (it keeps what the recipe leaves there, a bucket, when it ran out). Returns the cells that ran out with the stack they held (to be filled
/// again) and the leftovers that had no cell to go to.
pub fn consume(grid: &mut [Option<ItemStack>; 9], remaining: &[Option<ItemStack>]) -> (Vec<(usize, ItemStack)>, Vec<ItemStack>) {
    let mut ran_out = Vec::new();
    let mut leftovers = Vec::new();
    for i in 0..9 {
        let Some(cell) = grid[i].clone() else { continue };
        let kept = remaining.get(i).cloned().flatten();
        if cell.count > 1 {
            grid[i] = Some(ItemStack { count: cell.count - 1, ..cell });
            if let Some(k) = kept {
                leftovers.push(k);
            }
        } else {
            grid[i] = None;
            match kept {
                Some(k) => grid[i] = Some(k),
                None => ran_out.push((i, cell)),
            }
        }
    }
    (ran_out, leftovers)
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
    pub(crate) fn qio_find(&mut self, key: &str) -> Option<Freq> {
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
    pub(crate) fn qio_apply(&mut self, player: (&str, &str), current: &str, press: Option<i32>, text: Option<(i32, &str)>) -> String {
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
    pub(crate) fn qio_selector(&mut self, viewer: &str, current: &str) -> (Vec<String>, usize, usize) {
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

    /// The crafting windows of a Dashboard for a tick: the results that were taken use up their ingredients (and the cells that ran out are filled from the frequency), the clear buttons
    /// send a grid to the frequency, and each result slot shows what its grid makes. True when a slot changed.
    fn qio_craft(&mut self, key: &BlockKey, items: &mut [Option<ItemStack>], freq: Option<String>, clears: &[usize]) -> bool {
        let mut changed = false;
        for w in 0..WINDOWS {
            let base = w * PER_WINDOW;
            let grid_of = |items: &[Option<ItemStack>]| -> [Option<ItemStack>; 9] { std::array::from_fn(|i| items[base + i].clone()) };
            let mut grid = grid_of(items);
            let slot_key = (key.clone(), w);
            let had = self.qio_outputs.get(&slot_key).cloned().flatten();
            // The player took the result: its ingredients are used up.
            if had.is_some() && items[base + 9].is_none() {
                if let Some(m) = recipe_types::crafting_find(&grid.to_vec()) {
                    let (ran_out, leftovers) = consume(&mut grid, &m.remaining);
                    if let Some(net) = freq.as_ref().and_then(|f| self.qio.frequencies.get_mut(f)) {
                        for (i, template) in ran_out {
                            let h = hashed(&template.item, &template.components);
                            if net.remove_item(&h, 1, true) == 1 {
                                grid[i] = Some(ItemStack { count: 1, ..template });
                            }
                        }
                        for l in leftovers {
                            net.add_item(&hashed(&l.item, &l.components), u64::from(l.count), true);
                        }
                    }
                    for i in 0..9 {
                        items[base + i] = grid[i].clone();
                    }
                    changed = true;
                }
            }
            // The clear button: the grid goes to the frequency, what does not fit stays.
            if clears.contains(&w) {
                if let Some(net) = freq.as_ref().and_then(|f| self.qio.frequencies.get_mut(f)) {
                    for i in 0..9 {
                        if let Some(cell) = items[base + i].clone() {
                            let rejected = net.add_item(&hashed(&cell.item, &cell.components), u64::from(cell.count), true);
                            if rejected != u64::from(cell.count) {
                                items[base + i] = (rejected > 0).then(|| ItemStack { count: rejected as u32, ..cell });
                                changed = true;
                            }
                        }
                    }
                }
            }
            let grid = grid_of(items);
            let shown = recipe_types::crafting_find(&grid.to_vec()).map(|m| m.output);
            if items[base + 9] != shown && !(items[base + 9].is_some() && shown.as_ref().zip(items[base + 9].as_ref()).is_some_and(|(a, b)| same_stack(a, b) && a.count == b.count)) {
                items[base + 9] = shown;
                changed = true;
            }
            self.qio_outputs.insert(slot_key, items[base + 9].clone());
        }
        changed
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
        let mut items: Vec<Option<ItemStack>> = (0..WINDOW_SLOTS).map(|i| e.items.get(i).cloned().flatten()).collect();
        let clears: Vec<usize> = presses
            .iter()
            .filter(|(d, p, id, _, _)| *d == e.dimension && *p == e.pos && (crate::qio_dashboard::CLEAR_CRAFTING_WINDOW_BASE..crate::qio_dashboard::CLEAR_CRAFTING_WINDOW_BASE + WINDOWS as i32).contains(id))
            .map(|(_, _, id, _, _)| (*id - crate::qio_dashboard::CLEAR_CRAFTING_WINDOW_BASE) as usize)
            .collect();
        if let Some(f) = &freq {
            self.qio_network(f);
        }
        let items_changed = self.qio_craft(&key, &mut items, freq.as_ref().map(Freq::key), &clears);
        let mut rows: Vec<(String, String, u64)> = Vec::new();
        if let Some(f) = &freq {
            self.qio_network(f);
            let net = self.qio.frequencies.get_mut(&f.key())?;
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
        let cell = if index == u32::MAX { None } else { Some(self.qio_views.get(&key)?.cells.get(index as usize).cloned()?) };
        grid_action(self.qio.frequencies.get_mut(&board_key)?, cell, button, shift, carried)
    }

    /// The same press on the Portable QIO Dashboard's grid (an item menu: the player's own frequency and cells).
    pub(crate) fn qio_portable_click(&mut self, player: &str, index: u32, button: u8, shift: bool, carried: Option<ItemStack>) -> Option<GridAnswer> {
        let freq = self.qio_portable_freq.get(player).cloned()?;
        let cell = if index == u32::MAX { None } else { Some(self.qio_portable_views.get(player)?.cells.get(index as usize).cloned()?) };
        grid_action(self.qio.frequencies.get_mut(&freq)?, cell, button, shift, carried)
    }

    /// The grid's rows for a frequency: filtered by the search, sorted, at most what the host takes.
    pub(crate) fn qio_rows(&mut self, freq: &str, search: &str, sort: u8, ascending: bool) -> Vec<(String, String, u64)> {
        let Some(net) = self.qio.frequencies.get(freq) else { return Vec::new() };
        let tags = |item: &str| recipe_types::item_tags(item);
        let mut rows: Vec<(String, String, u64)> = net.item_data_map.values().map(|d| { let (i, c) = cell_of(&d.item); (i, c, d.count) }).filter(|r| matches_search(search, &r.0, &tags)).collect();
        sort_rows(&mut rows, sort, ascending);
        rows.truncate(GRID_MAX);
        rows
    }
}

/// What a press does to the network: `cell` is the entry pressed (none for a cell with no entry).
fn grid_action(net: &mut QIOFrequency, cell: Option<(String, String)>, button: u8, shift: bool, carried: Option<ItemStack>) -> Option<GridAnswer> {
    let Some((item, components)) = cell else {
        // A cell with no entry: a stack on the cursor is stored, all of it with the left button, one with the others.
        let c = carried?;
        return store(net, c, if button == 0 { None } else { Some(1) });
    };
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
            let top = net.get_stored(&h).min(max);
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
                // A middle click with a matching stack in hand: one more from the network.
                let taken = net.remove_item(&h, 1, true);
                if taken > 0 && u64::from(c.count) < max {
                    return Some(GridAnswer { carried: Some(ItemStack { count: c.count + 1, ..c }), to_inventory: Vec::new() });
                }
                if taken > 0 {
                    net.add_item(&h, taken, true);
                }
                return None;
            }
            store(net, c, if button == 0 { None } else { Some(1) })
        }
    }
}

/// Stores `limit` (or all) of the cursor's stack: the cursor after.
fn store(net: &mut QIOFrequency, c: ItemStack, limit: Option<u64>) -> Option<GridAnswer> {
    let to_add = limit.unwrap_or(u64::from(c.count)).min(u64::from(c.count));
    let rejected = net.add_item(&hashed(&c.item, &c.components), to_add, true);
    let placed = to_add - rejected;
    if placed == 0 {
        return None;
    }
    let left = u64::from(c.count) - placed;
    Some(GridAnswer { carried: (left > 0).then(|| ItemStack { count: left as u32, ..c }), to_inventory: Vec::new() })
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

    fn st(item: &str, count: u32) -> ItemStack {
        ItemStack { item: item.to_owned(), count, components: String::new() }
    }

    #[test]
    fn a_craft_uses_one_of_each_cell_and_reports_the_cells_that_ran_out() {
        let mut grid: [Option<ItemStack>; 9] = Default::default();
        grid[0] = Some(st("minecraft:cobblestone", 5));
        grid[1] = Some(st("minecraft:cobblestone", 1));
        let (ran_out, left) = consume(&mut grid, &[]);
        assert_eq!(grid[0].as_ref().map(|s| s.count), Some(4));
        assert!(grid[1].is_none());
        assert_eq!(ran_out.len(), 1);
        assert_eq!(ran_out[0].0, 1);
        assert!(left.is_empty());
    }

    #[test]
    fn what_a_recipe_leaves_takes_the_cell_of_an_ingredient_that_ran_out() {
        let mut grid: [Option<ItemStack>; 9] = Default::default();
        grid[4] = Some(st("minecraft:milk_bucket", 1));
        grid[5] = Some(st("minecraft:milk_bucket", 3));
        let mut rem: Vec<Option<ItemStack>> = vec![None; 9];
        rem[4] = Some(st("minecraft:bucket", 1));
        rem[5] = Some(st("minecraft:bucket", 1));
        let (ran_out, left) = consume(&mut grid, &rem);
        assert_eq!(grid[4].as_ref().map(|s| s.item.as_str()), Some("minecraft:bucket"));
        assert!(ran_out.is_empty(), "the bucket stays in the cell, nothing to refill");
        assert_eq!(grid[5].as_ref().map(|s| s.count), Some(2));
        assert_eq!(left.len(), 1, "the cell that kept its milk has nowhere for the bucket");
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

//! QIO Transfer: QIO Importer (`TileEntityQIOImporter`) and QIO Exporter (`TileEntityQIOExporter`).
//!
//! Reference: Mekanism Java 1.21.1-10.7.19.85
//! - Importer: `java/main/java/mekanism/common/tile/qio/TileEntityQIOImporter.java`
//! - Exporter: `java/main/java/mekanism/common/tile/qio/TileEntityQIOExporter.java`
//! - Filter Handler & Transit Math: `java/main/java/mekanism/common/tile/qio/TileEntityQIOFilterHandler.java`
//! - Filters: `java/main/java/mekanism/common/content/qio/filter/QIOFilter.java`
//!
//! Capabilities:
//! - Upgrades: Strictly `Upgrade.SPEED` up to 8 (`AttributeUpgradeSupport.SPEED_ONLY`).
//! - Transit Count: `64 + 32 * speedUpgrades` (64 to 320 items per pulse).
//! - Transit Types: `round(1.0 + speedUpgrades / 2.0)` (1 to 5 types per pulse).
//! - Delay: 10 ticks (0.5s, `MekanismUtils.TICKS_PER_HALF_SECOND = 10`).
//! - Importer: Scans back container slots in reverse (`slots - 1 down to 0`), checks filters or `importWithoutFilter` (default `true`).
//! - Exporter: Pulls from QIO frequency, checks filters or `exportWithoutFilter` (default `false`), supports round-robin destination alternation.
use super::*;
use std::collections::HashSet;

pub const IMPORTER_BLOCK: &str = "mekanism:qio_importer";
pub const EXPORTER_BLOCK: &str = "mekanism:qio_exporter";

/// Pulse delay in ticks (`MAX_DELAY = MekanismUtils.TICKS_PER_HALF_SECOND = 10` in `TileEntityQIOImporter.java:38`).
pub const MAX_DELAY: u64 = 10;

/// Maximum speed upgrades supported (`AttributeUpgradeSupport.SPEED_ONLY`, max 8).
pub const MAX_SPEED_UPGRADES: u8 = 8;

/// Screen button IDs for importer and exporter menus.
pub const TOGGLE_IMPORT_WITHOUT_FILTER: i32 = 7600;
pub const TOGGLE_EXPORT_WITHOUT_FILTER: i32 = 7601;
pub const TOGGLE_ROUND_ROBIN: i32 = 7602;

// ============================================================================
// Transit Math (TileEntityQIOFilterHandler.java:41-43)
// ============================================================================

/// Calculates maximum items moved per pulse (64 to 320 items).
/// Source Java: `maxTransitCount = 64 + 32 * speedUpgrades;` (`TileEntityQIOFilterHandler.java:41`)
pub fn max_transit_count(speed_upgrades: u8) -> u32 {
    let clamped = speed_upgrades.min(MAX_SPEED_UPGRADES) as u32;
    64 + 32 * clamped
}

/// Calculates maximum distinct item types moved per pulse (1 to 5 types).
/// Source Java: `maxTransitTypes = Math.round(1F + speedUpgrades / 2F);` (`TileEntityQIOFilterHandler.java:43`)
pub fn max_transit_types(speed_upgrades: u8) -> u32 {
    let clamped = speed_upgrades.min(MAX_SPEED_UPGRADES) as f32;
    (1.0f32 + clamped / 2.0f32).round() as u32
}

// ============================================================================
// Interface qio_armazenamento (Expected from e1)
// ============================================================================

/// Interface contract for QIO Frequency storage expected from e1 (`qio_armazenamento`).
pub trait QioStorageView {
    /// Inserts up to `count` of an item into the frequency. Returns amount inserted.
    fn add_item(&mut self, item: &str, count: u32, components: &str) -> u32;

    /// Extracts up to `count` of an item from the frequency. Returns amount extracted.
    fn extract_item(&mut self, item: &str, count: u32, components: &str) -> u32;

    /// Queries the quantity stored for a specific item and components.
    fn get_stored_count(&self, item: &str, components: &str) -> u64;

    /// Lists all items stored in the frequency as `(item_id, components, count)`.
    fn get_all_stored(&self) -> Vec<(String, String, u64)>;
}

// ============================================================================
// QIO Filters (QIOFilter.java, QIOItemStackFilter.java, QIOTagFilter.java, QIOModIDFilter.java)
// ============================================================================

/// Supported QIO filter types.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QioFilter {
    /// Filters by exact or fuzzy item stack (`QIOItemStackFilter.java`).
    ItemStack {
        item: String,
        components: String,
        fuzzy: bool,
        enabled: bool,
    },
    /// Filters by item tag wildcard (`QIOTagFilter.java`).
    Tag {
        tag: String,
        enabled: bool,
    },
    /// Filters by mod namespace (`QIOModIDFilter.java`).
    ModId {
        mod_id: String,
        enabled: bool,
    },
}

impl QioFilter {
    pub fn is_enabled(&self) -> bool {
        match self {
            QioFilter::ItemStack { enabled, .. } => *enabled,
            QioFilter::Tag { enabled, .. } => *enabled,
            QioFilter::ModId { enabled, .. } => *enabled,
        }
    }

    /// Evaluates if an item stack matches this filter.
    pub fn matches(&self, item: &str, components: &str, tags: &[String]) -> bool {
        if !self.is_enabled() {
            return false;
        }
        match self {
            QioFilter::ItemStack { item: target_item, components: target_comp, fuzzy, .. } => {
                if *fuzzy {
                    item == target_item
                } else {
                    item == target_item && components == target_comp
                }
            }
            QioFilter::Tag { tag, .. } => {
                let pattern = tag.trim();
                if pattern.ends_with('*') {
                    let prefix = &pattern[..pattern.len() - 1];
                    tags.iter().any(|t| t.starts_with(prefix))
                } else {
                    tags.iter().any(|t| t == pattern)
                }
            }
            QioFilter::ModId { mod_id, .. } => {
                let namespace = item.split(':').next().unwrap_or("");
                let pattern = mod_id.trim();
                if pattern.ends_with('*') {
                    let prefix = &pattern[..pattern.len() - 1];
                    namespace.starts_with(prefix)
                } else {
                    namespace == pattern
                }
            }
        }
    }
}

/// Filter manager with sorting and validation (`SortableFilterManager<QIOFilter<?>>`).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct QioFilterManager {
    pub filters: Vec<QioFilter>,
}

impl QioFilterManager {
    pub fn new() -> Self {
        Self { filters: Vec::new() }
    }

    pub fn has_enabled_filters(&self) -> bool {
        self.filters.iter().any(|f| f.is_enabled())
    }

    pub fn any_enabled_match(&self, item: &str, components: &str, tags: &[String]) -> bool {
        self.filters.iter().any(|f| f.matches(item, components, tags))
    }

    pub fn add_filter(&mut self, filter: QioFilter) -> bool {
        if !self.filters.contains(&filter) {
            self.filters.push(filter);
            true
        } else {
            false
        }
    }

    pub fn remove_filter(&mut self, index: usize) -> Option<QioFilter> {
        if index < self.filters.len() {
            Some(self.filters.remove(index))
        } else {
            None
        }
    }
}

// ============================================================================
// QIO Importer (TileEntityQIOImporter.java)
// ============================================================================

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImporterSettings {
    /// Whether to pull items when no filters are enabled (default `true`, `SerializationConstants.AUTO`).
    pub import_without_filter: bool,
    pub speed_upgrades: u8,
    pub redstone: redstone::Control,
    pub filters: QioFilterManager,
    pub delay: u8,
    pub frequency_name: String,
}

impl Default for ImporterSettings {
    fn default() -> Self {
        Self {
            import_without_filter: true,
            speed_upgrades: 0,
            redstone: redstone::Control::Disabled,
            filters: QioFilterManager::new(),
            delay: 0,
            frequency_name: String::new(),
        }
    }
}

pub fn encode_importer(s: &ImporterSettings) -> Vec<u8> {
    let mut out = vec![
        1, // version
        u8::from(s.import_without_filter),
        s.speed_upgrades,
        s.redstone as u8,
        s.delay,
    ];
    let freq_bytes = s.frequency_name.as_bytes();
    out.push(freq_bytes.len().min(255) as u8);
    out.extend_from_slice(&freq_bytes[..freq_bytes.len().min(255)]);

    out.push(s.filters.filters.len().min(255) as u8);
    for f in &s.filters.filters {
        match f {
            QioFilter::ItemStack { item, components, fuzzy, enabled } => {
                out.push(0);
                out.push(u8::from(*enabled));
                out.push(u8::from(*fuzzy));
                let ib = item.as_bytes();
                out.push(ib.len().min(255) as u8);
                out.extend_from_slice(&ib[..ib.len().min(255)]);
                let cb = components.as_bytes();
                out.push(cb.len().min(255) as u8);
                out.extend_from_slice(&cb[..cb.len().min(255)]);
            }
            QioFilter::Tag { tag, enabled } => {
                out.push(1);
                out.push(u8::from(*enabled));
                let tb = tag.as_bytes();
                out.push(tb.len().min(255) as u8);
                out.extend_from_slice(&tb[..tb.len().min(255)]);
            }
            QioFilter::ModId { mod_id, enabled } => {
                out.push(2);
                out.push(u8::from(*enabled));
                let mb = mod_id.as_bytes();
                out.push(mb.len().min(255) as u8);
                out.extend_from_slice(&mb[..mb.len().min(255)]);
            }
        }
    }
    out
}

pub fn decode_importer(data: &[u8]) -> Option<ImporterSettings> {
    if data.len() < 5 || data[0] != 1 {
        return None;
    }
    let import_without_filter = data[1] == 1;
    let speed_upgrades = data[2];
    let redstone = redstone::Control::from_byte(data[3]);
    let delay = data[4];
    let mut idx = 5;

    let mut frequency_name = String::new();
    if idx < data.len() {
        let flen = data[idx] as usize;
        idx += 1;
        if idx + flen <= data.len() {
            frequency_name = String::from_utf8_lossy(&data[idx..idx + flen]).to_string();
            idx += flen;
        }
    }

    let mut filters = Vec::new();
    if idx < data.len() {
        let count = data[idx] as usize;
        idx += 1;
        for _ in 0..count {
            if idx >= data.len() {
                break;
            }
            let kind = data[idx];
            idx += 1;
            match kind {
                0 => {
                    if idx + 4 > data.len() {
                        break;
                    }
                    let enabled = data[idx] == 1;
                    let fuzzy = data[idx + 1] == 1;
                    let ilen = data[idx + 2] as usize;
                    idx += 3;
                    if idx + ilen > data.len() {
                        break;
                    }
                    let item = String::from_utf8_lossy(&data[idx..idx + ilen]).to_string();
                    idx += ilen;
                    if idx >= data.len() {
                        break;
                    }
                    let clen = data[idx] as usize;
                    idx += 1;
                    if idx + clen > data.len() {
                        break;
                    }
                    let components = String::from_utf8_lossy(&data[idx..idx + clen]).to_string();
                    idx += clen;
                    filters.push(QioFilter::ItemStack { item, components, fuzzy, enabled });
                }
                1 => {
                    if idx + 2 > data.len() {
                        break;
                    }
                    let enabled = data[idx] == 1;
                    let tlen = data[idx + 1] as usize;
                    idx += 2;
                    if idx + tlen > data.len() {
                        break;
                    }
                    let tag = String::from_utf8_lossy(&data[idx..idx + tlen]).to_string();
                    idx += tlen;
                    filters.push(QioFilter::Tag { tag, enabled });
                }
                2 => {
                    if idx + 2 > data.len() {
                        break;
                    }
                    let enabled = data[idx] == 1;
                    let mlen = data[idx + 1] as usize;
                    idx += 2;
                    if idx + mlen > data.len() {
                        break;
                    }
                    let mod_id = String::from_utf8_lossy(&data[idx..idx + mlen]).to_string();
                    idx += mlen;
                    filters.push(QioFilter::ModId { mod_id, enabled });
                }
                _ => break,
            }
        }
    }

    Some(ImporterSettings {
        import_without_filter,
        speed_upgrades,
        redstone,
        filters: QioFilterManager { filters },
        delay,
        frequency_name,
    })
}

/// Executes one tick of QIO Importer logic.
/// Scans back inventory slots in reverse (`slots - 1 down to 0` in Java line 99).
pub fn step_importer<F: QioStorageView, T: Fn(&str) -> Vec<String>>(
    settings: &mut ImporterSettings,
    can_function: bool,
    back_inventory: &mut [Option<ItemStack>],
    frequency: &mut F,
    tags_lookup: T,
) -> bool {
    if !can_function {
        return false;
    }
    if settings.delay > 0 {
        settings.delay -= 1;
        return false;
    }
    settings.delay = MAX_DELAY as u8;

    let has_filters = settings.filters.has_enabled_filters();
    if !has_filters && !settings.import_without_filter {
        return false;
    }

    let max_count = max_transit_count(settings.speed_upgrades);
    let max_types = max_transit_types(settings.speed_upgrades) as usize;

    let mut types_added: HashSet<String> = HashSet::new();
    let mut count_added: u32 = 0;
    let mut modified = false;

    // Scan backwards through slots: Java `for (int i = slots - 1; i >= 0; i--)`
    for i in (0..back_inventory.len()).rev() {
        if count_added >= max_count {
            break;
        }
        let stack = match &back_inventory[i] {
            Some(s) if s.count > 0 => s.clone(),
            _ => continue,
        };

        let type_key = format!("{}#{}", stack.item, stack.components);
        if !types_added.contains(&type_key) && types_added.len() >= max_types {
            continue;
        }

        let tags = tags_lookup(&stack.item);
        let matches = if has_filters {
            settings.filters.any_enabled_match(&stack.item, &stack.components, &tags)
        } else {
            settings.import_without_filter
        };

        if !matches {
            continue;
        }

        let to_move = (max_count - count_added).min(stack.count);
        let inserted = frequency.add_item(&stack.item, to_move, &stack.components);
        if inserted > 0 {
            types_added.insert(type_key);
            count_added += inserted;
            modified = true;

            if stack.count <= inserted {
                back_inventory[i] = None;
            } else {
                back_inventory[i] = Some(ItemStack {
                    item: stack.item,
                    count: stack.count - inserted,
                    components: stack.components,
                });
            }
        }
    }

    modified
}

// ============================================================================
// QIO Exporter (TileEntityQIOExporter.java)
// ============================================================================

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExporterSettings {
    /// Whether to export all items when no filters are set (default `false`, `SerializationConstants.AUTO`).
    pub export_without_filter: bool,
    /// Round robin mode for multiple destinations (`SerializationConstants.ROUND_ROBIN`).
    pub round_robin: bool,
    pub speed_upgrades: u8,
    pub redstone: redstone::Control,
    pub filters: QioFilterManager,
    pub delay: u8,
    pub round_robin_index: usize,
    pub frequency_name: String,
}

impl Default for ExporterSettings {
    fn default() -> Self {
        Self {
            export_without_filter: false,
            round_robin: false,
            speed_upgrades: 0,
            redstone: redstone::Control::Disabled,
            filters: QioFilterManager::new(),
            delay: 0,
            round_robin_index: 0,
            frequency_name: String::new(),
        }
    }
}

pub fn encode_exporter(s: &ExporterSettings) -> Vec<u8> {
    let mut out = vec![
        1, // version
        u8::from(s.export_without_filter),
        u8::from(s.round_robin),
        s.speed_upgrades,
        s.redstone as u8,
        s.delay,
        (s.round_robin_index & 0xFF) as u8,
    ];
    let freq_bytes = s.frequency_name.as_bytes();
    out.push(freq_bytes.len().min(255) as u8);
    out.extend_from_slice(&freq_bytes[..freq_bytes.len().min(255)]);

    out.push(s.filters.filters.len().min(255) as u8);
    for f in &s.filters.filters {
        match f {
            QioFilter::ItemStack { item, components, fuzzy, enabled } => {
                out.push(0);
                out.push(u8::from(*enabled));
                out.push(u8::from(*fuzzy));
                let ib = item.as_bytes();
                out.push(ib.len().min(255) as u8);
                out.extend_from_slice(&ib[..ib.len().min(255)]);
                let cb = components.as_bytes();
                out.push(cb.len().min(255) as u8);
                out.extend_from_slice(&cb[..cb.len().min(255)]);
            }
            QioFilter::Tag { tag, enabled } => {
                out.push(1);
                out.push(u8::from(*enabled));
                let tb = tag.as_bytes();
                out.push(tb.len().min(255) as u8);
                out.extend_from_slice(&tb[..tb.len().min(255)]);
            }
            QioFilter::ModId { mod_id, enabled } => {
                out.push(2);
                out.push(u8::from(*enabled));
                let mb = mod_id.as_bytes();
                out.push(mb.len().min(255) as u8);
                out.extend_from_slice(&mb[..mb.len().min(255)]);
            }
        }
    }
    out
}

pub fn decode_exporter(data: &[u8]) -> Option<ExporterSettings> {
    if data.len() < 7 || data[0] != 1 {
        return None;
    }
    let export_without_filter = data[1] == 1;
    let round_robin = data[2] == 1;
    let speed_upgrades = data[3];
    let redstone = redstone::Control::from_byte(data[4]);
    let delay = data[5];
    let round_robin_index = data[6] as usize;
    let mut idx = 7;

    let mut frequency_name = String::new();
    if idx < data.len() {
        let flen = data[idx] as usize;
        idx += 1;
        if idx + flen <= data.len() {
            frequency_name = String::from_utf8_lossy(&data[idx..idx + flen]).to_string();
            idx += flen;
        }
    }

    let mut filters = Vec::new();
    if idx < data.len() {
        let count = data[idx] as usize;
        idx += 1;
        for _ in 0..count {
            if idx >= data.len() {
                break;
            }
            let kind = data[idx];
            idx += 1;
            match kind {
                0 => {
                    if idx + 4 > data.len() {
                        break;
                    }
                    let enabled = data[idx] == 1;
                    let fuzzy = data[idx + 1] == 1;
                    let ilen = data[idx + 2] as usize;
                    idx += 3;
                    if idx + ilen > data.len() {
                        break;
                    }
                    let item = String::from_utf8_lossy(&data[idx..idx + ilen]).to_string();
                    idx += ilen;
                    if idx >= data.len() {
                        break;
                    }
                    let clen = data[idx] as usize;
                    idx += 1;
                    if idx + clen > data.len() {
                        break;
                    }
                    let components = String::from_utf8_lossy(&data[idx..idx + clen]).to_string();
                    idx += clen;
                    filters.push(QioFilter::ItemStack { item, components, fuzzy, enabled });
                }
                1 => {
                    if idx + 2 > data.len() {
                        break;
                    }
                    let enabled = data[idx] == 1;
                    let tlen = data[idx + 1] as usize;
                    idx += 2;
                    if idx + tlen > data.len() {
                        break;
                    }
                    let tag = String::from_utf8_lossy(&data[idx..idx + tlen]).to_string();
                    idx += tlen;
                    filters.push(QioFilter::Tag { tag, enabled });
                }
                2 => {
                    if idx + 2 > data.len() {
                        break;
                    }
                    let enabled = data[idx] == 1;
                    let mlen = data[idx + 1] as usize;
                    idx += 2;
                    if idx + mlen > data.len() {
                        break;
                    }
                    let mod_id = String::from_utf8_lossy(&data[idx..idx + mlen]).to_string();
                    idx += mlen;
                    filters.push(QioFilter::ModId { mod_id, enabled });
                }
                _ => break,
            }
        }
    }

    Some(ExporterSettings {
        export_without_filter,
        round_robin,
        speed_upgrades,
        redstone,
        filters: QioFilterManager { filters },
        delay,
        round_robin_index,
        frequency_name,
    })
}

/// Inserts an item into an inventory slot array. Returns amount actually inserted.
pub fn insert_into_container(slots: &mut [Option<ItemStack>], item: &str, count: u32, components: &str, max_stack: u32) -> u32 {
    let mut remaining = count;

    // First pass: stack into existing matching stacks
    for slot in slots.iter_mut() {
        if remaining == 0 {
            break;
        }
        if let Some(s) = slot {
            if s.item == item && s.components == components && s.count < max_stack {
                let room = max_stack - s.count;
                let add = room.min(remaining);
                s.count += add;
                remaining -= add;
            }
        }
    }

    // Second pass: fill empty slots
    for slot in slots.iter_mut() {
        if remaining == 0 {
            break;
        }
        if slot.is_none() {
            let add = max_stack.min(remaining);
            *slot = Some(ItemStack {
                item: item.to_string(),
                count: add,
                components: components.to_string(),
            });
            remaining -= add;
        }
    }

    count - remaining
}

/// Executes one tick of QIO Exporter logic.
pub fn step_exporter<F: QioStorageView, T: Fn(&str) -> Vec<String>>(
    settings: &mut ExporterSettings,
    can_function: bool,
    destinations: &mut [&mut [Option<ItemStack>]],
    frequency: &mut F,
    tags_lookup: T,
    max_stack: u32,
) -> bool {
    if !can_function || destinations.is_empty() {
        return false;
    }
    if settings.delay > 0 {
        settings.delay -= 1;
        return false;
    }
    settings.delay = MAX_DELAY as u8;

    let has_filters = settings.filters.has_enabled_filters();
    if !has_filters && !settings.export_without_filter {
        return false;
    }

    let max_count = max_transit_count(settings.speed_upgrades);
    let max_types = max_transit_types(settings.speed_upgrades) as usize;

    let all_stored = frequency.get_all_stored();
    if all_stored.is_empty() {
        return false;
    }

    let mut types_exported: HashSet<String> = HashSet::new();
    let mut count_exported: u32 = 0;
    let mut modified = false;

    for (item, components, stored) in all_stored {
        if count_exported >= max_count {
            break;
        }
        if stored == 0 {
            continue;
        }

        let type_key = format!("{item}#{components}");
        if !types_exported.contains(&type_key) && types_exported.len() >= max_types {
            continue;
        }

        let tags = tags_lookup(&item);
        let matches = if has_filters {
            settings.filters.any_enabled_match(&item, &components, &tags)
        } else {
            settings.export_without_filter
        };

        if !matches {
            continue;
        }

        let dest_idx = if settings.round_robin {
            settings.round_robin_index % destinations.len()
        } else {
            0
        };

        let to_eject = (max_count - count_exported).min(stored.min(u64::from(max_stack)) as u32);
        let inserted = insert_into_container(destinations[dest_idx], &item, to_eject, &components, max_stack);

        if inserted > 0 {
            frequency.extract_item(&item, inserted, &components);
            types_exported.insert(type_key);
            count_exported += inserted;
            modified = true;

            if settings.round_robin && destinations.len() > 1 {
                settings.round_robin_index = (settings.round_robin_index + 1) % destinations.len();
            }
        }
    }

    modified
}

// ============================================================================
// Unit Tests (Comparing with Java reference sources)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Fake in-memory QIO frequency for unit testing.
    struct MockQioFrequency {
        items: std::collections::HashMap<(String, String), u64>,
    }

    impl MockQioFrequency {
        fn new() -> Self {
            Self { items: std::collections::HashMap::new() }
        }
    }

    impl QioStorageView for MockQioFrequency {
        fn add_item(&mut self, item: &str, count: u32, components: &str) -> u32 {
            let key = (item.to_string(), components.to_string());
            let current = self.items.entry(key).or_insert(0);
            *current += u64::from(count);
            count
        }

        fn extract_item(&mut self, item: &str, count: u32, components: &str) -> u32 {
            let key = (item.to_string(), components.to_string());
            if let Some(entry) = self.items.get_mut(&key) {
                let to_take = (*entry).min(u64::from(count)) as u32;
                *entry -= u64::from(to_take);
                to_take
            } else {
                0
            }
        }

        fn get_stored_count(&self, item: &str, components: &str) -> u64 {
            self.items.get(&(item.to_string(), components.to_string())).copied().unwrap_or(0)
        }

        fn get_all_stored(&self) -> Vec<(String, String, u64)> {
            self.items.iter().map(|((i, c), n)| (i.clone(), c.clone(), *n)).collect()
        }
    }

    #[test]
    fn test_transit_capacity_numbers_match_java() {
        // Java reference: `TileEntityQIOFilterHandler.java:41-43`
        // maxTransitCount = 64 + 32 * speedUpgrades (range: 64 to 320)
        // maxTransitTypes = Math.round(1F + speedUpgrades / 2F) (range: 1 to 5)

        // 0 upgrades
        assert_eq!(max_transit_count(0), 64);
        assert_eq!(max_transit_types(0), 1);

        // 1 upgrade
        assert_eq!(max_transit_count(1), 96);
        assert_eq!(max_transit_types(1), 2); // round(1.0 + 0.5) = 2

        // 2 upgrades
        assert_eq!(max_transit_count(2), 128);
        assert_eq!(max_transit_types(2), 2); // round(1.0 + 1.0) = 2

        // 3 upgrades
        assert_eq!(max_transit_count(3), 160);
        assert_eq!(max_transit_types(3), 3); // round(1.0 + 1.5) = 3

        // 4 upgrades
        assert_eq!(max_transit_count(4), 192);
        assert_eq!(max_transit_types(4), 3); // round(1.0 + 2.0) = 3

        // 5 upgrades
        assert_eq!(max_transit_count(5), 224);
        assert_eq!(max_transit_types(5), 4); // round(1.0 + 2.5) = 4

        // 6 upgrades
        assert_eq!(max_transit_count(6), 256);
        assert_eq!(max_transit_types(6), 4); // round(1.0 + 3.0) = 4

        // 7 upgrades
        assert_eq!(max_transit_count(7), 288);
        assert_eq!(max_transit_types(7), 5); // round(1.0 + 3.5) = 5

        // 8 upgrades (maximum cap)
        assert_eq!(max_transit_count(8), 320);
        assert_eq!(max_transit_types(8), 5); // round(1.0 + 4.0) = 5

        // Beyond cap clamped to 8
        assert_eq!(max_transit_count(10), 320);
        assert_eq!(max_transit_types(10), 5);
    }

    #[test]
    fn test_filter_matching_item_tag_modid() {
        // Exact ItemStack filter
        let exact = QioFilter::ItemStack {
            item: "minecraft:iron_ingot".into(),
            components: "{\"damage\":0}".into(),
            fuzzy: false,
            enabled: true,
        };
        assert!(exact.matches("minecraft:iron_ingot", "{\"damage\":0}", &[]));
        assert!(!exact.matches("minecraft:iron_ingot", "{\"damage\":1}", &[]));
        assert!(!exact.matches("minecraft:gold_ingot", "{\"damage\":0}", &[]));

        // Fuzzy ItemStack filter (ignores components)
        let fuzzy = QioFilter::ItemStack {
            item: "minecraft:diamond_sword".into(),
            components: String::new(),
            fuzzy: true,
            enabled: true,
        };
        assert!(fuzzy.matches("minecraft:diamond_sword", "{\"custom_name\":\"Excalibur\"}", &[]));
        assert!(!fuzzy.matches("minecraft:iron_sword", "{\"custom_name\":\"Excalibur\"}", &[]));

        // Tag filter exact and wildcard
        let tag_filter = QioFilter::Tag {
            tag: "c:ingots/iron".into(),
            enabled: true,
        };
        assert!(tag_filter.matches("mekanism:ingot_iron", "", &["c:ingots/iron".into(), "c:ingots".into()]));
        assert!(!tag_filter.matches("minecraft:apple", "", &["c:foods".into()]));

        let tag_wildcard = QioFilter::Tag {
            tag: "c:ingots/*".into(),
            enabled: true,
        };
        assert!(tag_wildcard.matches("mekanism:ingot_osmium", "", &["c:ingots/osmium".into()]));

        // ModID filter exact and wildcard
        let mod_filter = QioFilter::ModId {
            mod_id: "mekanism".into(),
            enabled: true,
        };
        assert!(mod_filter.matches("mekanism:enriched_iron", "", &[]));
        assert!(!mod_filter.matches("minecraft:iron_ingot", "", &[]));
    }

    #[test]
    fn test_importer_reverse_slot_order_and_types_limit() {
        // Java reference: `TileEntityQIOImporter.java:99`
        // Scans slots in reverse: `for (int i = slots - 1; i >= 0; i--)`
        let mut settings = ImporterSettings {
            import_without_filter: true,
            speed_upgrades: 0, // maxTransitTypes = 1, maxTransitCount = 64
            redstone: redstone::Control::Disabled,
            filters: QioFilterManager::new(),
            delay: 0,
            frequency_name: "test_freq".into(),
        };

        let mut back_inv = vec![
            Some(ItemStack { item: "minecraft:stone".into(), count: 64, components: String::new() }), // slot 0
            Some(ItemStack { item: "minecraft:dirt".into(), count: 64, components: String::new() }),  // slot 1
        ];
        let mut freq = MockQioFrequency::new();

        // With maxTransitTypes = 1, reverse traversal must pick slot 1 ("minecraft:dirt") first!
        let stepped = step_importer(&mut settings, true, &mut back_inv, &mut freq, |_| Vec::new());
        assert!(stepped);
        assert_eq!(freq.get_stored_count("minecraft:dirt", ""), 64);
        assert_eq!(freq.get_stored_count("minecraft:stone", ""), 0); // Not touched because types limit reached!
        assert!(back_inv[1].is_none());
        assert!(back_inv[0].is_some());
    }

    #[test]
    fn test_importer_filter_restriction() {
        let mut filter_manager = QioFilterManager::new();
        filter_manager.add_filter(QioFilter::ItemStack {
            item: "minecraft:gold_ingot".into(),
            components: String::new(),
            fuzzy: true,
            enabled: true,
        });

        let mut settings = ImporterSettings {
            import_without_filter: true, // Should be overridden by enabled filters
            speed_upgrades: 2,           // maxTransitCount = 128
            redstone: redstone::Control::Disabled,
            filters: filter_manager,
            delay: 0,
            frequency_name: "gold_only".into(),
        };

        let mut back_inv = vec![
            Some(ItemStack { item: "minecraft:dirt".into(), count: 64, components: String::new() }),
            Some(ItemStack { item: "minecraft:gold_ingot".into(), count: 32, components: String::new() }),
        ];
        let mut freq = MockQioFrequency::new();

        let stepped = step_importer(&mut settings, true, &mut back_inv, &mut freq, |_| Vec::new());
        assert!(stepped);
        assert_eq!(freq.get_stored_count("minecraft:gold_ingot", ""), 32);
        assert_eq!(freq.get_stored_count("minecraft:dirt", ""), 0); // Filtered out!
        assert!(back_inv[1].is_none());
        assert!(back_inv[0].is_some());
    }

    #[test]
    fn test_exporter_round_robin_distribution() {
        let mut settings = ExporterSettings {
            export_without_filter: true,
            round_robin: true,
            speed_upgrades: 0, // maxTransitCount = 64
            redstone: redstone::Control::Disabled,
            filters: QioFilterManager::new(),
            delay: 0,
            round_robin_index: 0,
            frequency_name: "export_freq".into(),
        };

        let mut freq = MockQioFrequency::new();
        freq.add_item("minecraft:iron_ingot", 200, "");

        let mut dest0 = vec![None; 2];
        let mut dest1 = vec![None; 2];

        // Step 1: Ejects to dest0
        {
            let mut destinations = [&mut dest0[..], &mut dest1[..]];
            let stepped = step_exporter(&mut settings, true, &mut destinations, &mut freq, |_| Vec::new(), 64);
            assert!(stepped);
        }
        assert_eq!(dest0[0].as_ref().unwrap().count, 64);
        assert!(dest1[0].is_none());
        assert_eq!(settings.round_robin_index, 1);

        // Reset delay for step 2
        settings.delay = 0;

        // Step 2: Ejects to dest1
        {
            let mut destinations = [&mut dest0[..], &mut dest1[..]];
            let stepped = step_exporter(&mut settings, true, &mut destinations, &mut freq, |_| Vec::new(), 64);
            assert!(stepped);
        }
        assert_eq!(dest1[0].as_ref().unwrap().count, 64);
        assert_eq!(settings.round_robin_index, 0);
    }

    #[test]
    fn test_importer_exporter_serialization() {
        let mut imp = ImporterSettings::default();
        imp.speed_upgrades = 5;
        imp.import_without_filter = false;
        imp.frequency_name = "GlobalQIO".into();
        imp.filters.add_filter(QioFilter::Tag { tag: "c:ingots/*".into(), enabled: true });

        let enc_imp = encode_importer(&imp);
        let dec_imp = decode_importer(&enc_imp).expect("decode importer");
        assert_eq!(imp, dec_imp);

        let mut exp = ExporterSettings::default();
        exp.speed_upgrades = 8;
        exp.export_without_filter = true;
        exp.round_robin = true;
        exp.frequency_name = "MainNetwork".into();
        exp.filters.add_filter(QioFilter::ModId { mod_id: "mekanism".into(), enabled: true });

        let enc_exp = encode_exporter(&exp);
        let dec_exp = decode_exporter(&enc_exp).expect("decode exporter");
        assert_eq!(exp, dec_exp);
    }
}

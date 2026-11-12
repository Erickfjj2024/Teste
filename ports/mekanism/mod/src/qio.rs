//! Quantum Item Orchestration (QIO) Storage Subsystem.
//!
//! Faithful Rust port of Mekanism Java `1.21.1-10.7.19.85`:
//! - QIO Drives (4 tiers: Base, Hyper-Dense, Time-Dilating, Supermassive) (`QIODriveTier.java`, `ItemQIODrive.java`)
//! - Drive Data and Metadata Components (`QIODriveData.java`, `DriveMetadata.java`, `DriveContents.java`)
//! - QIO Drive Array (`TileEntityQIODriveArray.java`, `DriveArrayBakedModel.java`, `GuiQIODriveArray.java`)
//! - QIO Redstone Adapter (`TileEntityQIORedstoneAdapter.java`, `GuiQIORedstoneAdapter.java`)
//! - QIO Frequencies and Global Item Network Index (`QIOFrequency.java`, `HashedItem.java`)
//! - Security and Color management (`SecurityMode.java`, `EnumColor.java`)

use super::*;
use std::collections::{BTreeMap, HashMap, HashSet};

// ============================================================================
// 1. REGISTRY NAMES AND CONSTANTS
// ============================================================================

pub const BLOCK_DRIVE_ARRAY: &str = "mekanism:qio_drive_array";
pub const BLOCK_REDSTONE_ADAPTER: &str = "mekanism:qio_redstone_adapter";

pub const ITEM_DRIVE_BASE: &str = "mekanism:qio_drive_base";
pub const ITEM_DRIVE_HYPER_DENSE: &str = "mekanism:qio_drive_hyper_dense";
pub const ITEM_DRIVE_TIME_DILATING: &str = "mekanism:qio_drive_time_dilating";
pub const ITEM_DRIVE_SUPERMASSIVE: &str = "mekanism:qio_drive_supermassive";

/// QIODriveTier capacities from Java:
/// Base: 16_000 items, 128 types (`QIODriveTier.java:7`)
pub const BASE_COUNT_CAPACITY: u64 = 16_000;
pub const BASE_TYPE_CAPACITY: usize = 128;

/// Hyper-Dense: 128_000 items, 256 types (`QIODriveTier.java:8`)
pub const HYPER_DENSE_COUNT_CAPACITY: u64 = 128_000;
pub const HYPER_DENSE_TYPE_CAPACITY: usize = 256;

/// Time-Dilating: 1_048_000 items, 1_024 types (`QIODriveTier.java:9`)
pub const TIME_DILATING_COUNT_CAPACITY: u64 = 1_048_000;
pub const TIME_DILATING_TYPE_CAPACITY: usize = 1_024;

/// Supermassive: 16_000_000_000 items, 8_192 types (`QIODriveTier.java:10`)
pub const SUPERMASSIVE_COUNT_CAPACITY: u64 = 16_000_000_000;
pub const SUPERMASSIVE_TYPE_CAPACITY: usize = 8_192;

/// Number of drive slots in a QIO Drive Array (`TileEntityQIODriveArray.java:38`)
pub const DRIVE_SLOTS: usize = 12;

/// Update period: every 10 ticks = 0.5s (`MekanismUtils.TICKS_PER_HALF_SECOND = 10`)
pub const TICKS_PER_HALF_SECOND: u64 = 10;

/// GUI and Network button identifiers
pub const BUTTON_TOGGLE_FUZZY: i32 = 7800;
pub const BUTTON_TOGGLE_INVERTED: i32 = 7801;
pub const BUTTON_CLEAR_TARGET: i32 = 7802;
pub const BUTTON_SET_TRIGGER_COUNT: i32 = 7803;
pub const BUTTON_CYCLE_SECURITY: i32 = 7804;
pub const BUTTON_CYCLE_COLOR: i32 = 7805;

// ============================================================================
// 2. HASHED ITEM (Canonical item type representation)
// ============================================================================

/// Canonical representation of an item type and its data components / NBT,
/// ignoring count. Corresponds to `mekanism.common.lib.inventory.HashedItem`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HashedItem {
    pub id: String,
    pub components: Vec<u8>,
}

impl HashedItem {
    pub fn raw(id: &str) -> Self {
        Self {
            id: id.to_owned(),
            components: Vec::new(),
        }
    }

    pub fn with_components(id: &str, components: &[u8]) -> Self {
        Self {
            id: id.to_owned(),
            components: components.to_vec(),
        }
    }

    /// Fuzzy equality: matches base item ID regardless of components/NBT (`TileEntityQIORedstoneAdapter.java:59`).
    pub fn matches_fuzzy(&self, other_item_id: &str) -> bool {
        self.id == other_item_id
    }

    pub fn mod_id(&self) -> &str {
        self.id.split(':').next().unwrap_or("minecraft")
    }

    pub fn item_name(&self) -> &str {
        self.id.split(':').nth(1).unwrap_or(&self.id)
    }

    pub fn is_empty(&self) -> bool {
        self.id.is_empty() || self.id == "minecraft:air" || self.id == "mekanism:nothing"
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        let id_bytes = self.id.as_bytes();
        out.extend((id_bytes.len() as u16).to_le_bytes());
        out.extend(id_bytes);
        out.extend((self.components.len() as u16).to_le_bytes());
        out.extend(&self.components);
        out
    }

    pub fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        if bytes.len() < 2 {
            return None;
        }
        let id_len = u16::from_le_bytes(bytes[0..2].try_into().ok()?) as usize;
        let mut offset = 2;
        if offset + id_len > bytes.len() {
            return None;
        }
        let id = std::str::from_utf8(&bytes[offset..offset + id_len]).ok()?.to_owned();
        offset += id_len;

        if offset + 2 > bytes.len() {
            return None;
        }
        let comp_len = u16::from_le_bytes(bytes[offset..offset + 2].try_into().ok()?) as usize;
        offset += 2;
        if offset + comp_len > bytes.len() {
            return None;
        }
        let components = bytes[offset..offset + comp_len].to_vec();
        offset += comp_len;

        Some((Self { id, components }, offset))
    }
}

// ============================================================================
// 3. QIO DRIVE TIER
// ============================================================================

/// QIO Drive Tiers (`mekanism.common.tier.QIODriveTier`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum QIODriveTier {
    Base,
    HyperDense,
    TimeDilating,
    Supermassive,
}

impl QIODriveTier {
    pub fn max_count(self) -> u64 {
        match self {
            QIODriveTier::Base => BASE_COUNT_CAPACITY,
            QIODriveTier::HyperDense => HYPER_DENSE_COUNT_CAPACITY,
            QIODriveTier::TimeDilating => TIME_DILATING_COUNT_CAPACITY,
            QIODriveTier::Supermassive => SUPERMASSIVE_COUNT_CAPACITY,
        }
    }

    pub fn max_types(self) -> usize {
        match self {
            QIODriveTier::Base => BASE_TYPE_CAPACITY,
            QIODriveTier::HyperDense => HYPER_DENSE_TYPE_CAPACITY,
            QIODriveTier::TimeDilating => TIME_DILATING_TYPE_CAPACITY,
            QIODriveTier::Supermassive => SUPERMASSIVE_TYPE_CAPACITY,
        }
    }

    pub fn registry_name(self) -> &'static str {
        match self {
            QIODriveTier::Base => ITEM_DRIVE_BASE,
            QIODriveTier::HyperDense => ITEM_DRIVE_HYPER_DENSE,
            QIODriveTier::TimeDilating => ITEM_DRIVE_TIME_DILATING,
            QIODriveTier::Supermassive => ITEM_DRIVE_SUPERMASSIVE,
        }
    }

    pub fn from_item(item_id: &str) -> Option<QIODriveTier> {
        match item_id {
            ITEM_DRIVE_BASE => Some(QIODriveTier::Base),
            ITEM_DRIVE_HYPER_DENSE => Some(QIODriveTier::HyperDense),
            ITEM_DRIVE_TIME_DILATING => Some(QIODriveTier::TimeDilating),
            ITEM_DRIVE_SUPERMASSIVE => Some(QIODriveTier::Supermassive),
            _ => None,
        }
    }
}

// ============================================================================
// 4. DRIVE METADATA & CONTENTS (Data components)
// ============================================================================

/// Drive metadata component (`mekanism.common.attachments.qio.DriveMetadata`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DriveMetadata {
    pub count: u64,
    pub types: u32,
}

impl DriveMetadata {
    pub fn new(count: u64, types: u32) -> Self {
        Self { count, types }
    }

    pub fn is_empty(self) -> bool {
        self.count == 0 && self.types == 0
    }

    pub fn encode(self) -> Vec<u8> {
        let mut out = Vec::with_capacity(12);
        out.extend(self.count.to_le_bytes());
        out.extend(self.types.to_le_bytes());
        out
    }

    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 12 {
            return None;
        }
        let count = u64::from_le_bytes(bytes[0..8].try_into().ok()?);
        let types = u32::from_le_bytes(bytes[8..12].try_into().ok()?);
        Some(Self { count, types })
    }
}

/// Drive contents component (`mekanism.common.attachments.qio.DriveContents`).
/// Preserves stored items and counts in the item stack when removed from array.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DriveContents {
    pub items: Vec<(HashedItem, u64)>,
}

impl DriveContents {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn total_count(&self) -> u64 {
        self.items.iter().map(|(_, c)| *c).sum()
    }

    pub fn total_types(&self) -> usize {
        self.items.len()
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(1u8); // Version 1
        out.extend((self.items.len() as u32).to_le_bytes());
        for (item, count) in &self.items {
            let item_bytes = item.encode();
            out.extend((item_bytes.len() as u32).to_le_bytes());
            out.extend(item_bytes);
            out.extend(count.to_le_bytes());
        }
        out
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        if data.is_empty() || data[0] != 1 {
            return None;
        }
        if data.len() < 5 {
            return None;
        }
        let count = u32::from_le_bytes(data[1..5].try_into().ok()?) as usize;
        let mut offset = 5;
        let mut items = Vec::with_capacity(count);
        for _ in 0..count {
            if offset + 4 > data.len() {
                return None;
            }
            let item_len = u32::from_le_bytes(data[offset..offset + 4].try_into().ok()?) as usize;
            offset += 4;
            if offset + item_len > data.len() {
                return None;
            }
            let (item, _) = HashedItem::decode(&data[offset..offset + item_len])?;
            offset += item_len;

            if offset + 8 > data.len() {
                return None;
            }
            let item_count = u64::from_le_bytes(data[offset..offset + 8].try_into().ok()?);
            offset += 8;

            items.push((item, item_count));
        }
        Some(Self { items })
    }
}

// ============================================================================
// 5. DRIVE STATUS (LED States in Drive Array)
// ============================================================================

/// Drive status in array slot (`TileEntityQIODriveArray.DriveStatus`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum DriveStatus {
    #[default]
    None = 0,
    Offline = 1,
    Ready = 2,
    NearFull = 3,
    Full = 4,
}

impl DriveStatus {
    pub fn from_u8(val: u8) -> Self {
        match val {
            1 => DriveStatus::Offline,
            2 => DriveStatus::Ready,
            3 => DriveStatus::NearFull,
            4 => DriveStatus::Full,
            _ => DriveStatus::None,
        }
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }

    pub fn led_index(self) -> Option<usize> {
        match self {
            DriveStatus::Ready => Some(0),
            DriveStatus::NearFull => Some(1),
            DriveStatus::Full => Some(2),
            _ => None,
        }
    }
}

// ============================================================================
// 6. SECURITY MODE & ENUM COLOR
// ============================================================================

/// Security mode access level (`mekanism.api.security.SecurityMode`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum SecurityMode {
    #[default]
    Public = 0,
    Private = 1,
    Trusted = 2,
}

impl SecurityMode {
    pub fn from_u8(val: u8) -> Self {
        match val {
            1 => SecurityMode::Private,
            2 => SecurityMode::Trusted,
            _ => SecurityMode::Public,
        }
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// Mekanism 18 EnumColors (`mekanism.api.text.EnumColor`), default Indigo (9).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum QIOColor {
    Black = 0,
    DarkBlue = 1,
    DarkGreen = 2,
    DarkAqua = 3,
    DarkRed = 4,
    Purple = 5,
    Orange = 6,
    Gray = 7,
    DarkGray = 8,
    Indigo = 9,
    BrightGreen = 10,
    Aqua = 11,
    Red = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
    Brown = 16,
    BrightPink = 17,
}

impl Default for QIOColor {
    fn default() -> Self {
        QIOColor::Indigo // QIOFrequency default color is Indigo (QIOFrequency.java:119)
    }
}

impl QIOColor {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => QIOColor::Black,
            1 => QIOColor::DarkBlue,
            2 => QIOColor::DarkGreen,
            3 => QIOColor::DarkAqua,
            4 => QIOColor::DarkRed,
            5 => QIOColor::Purple,
            6 => QIOColor::Orange,
            7 => QIOColor::Gray,
            8 => QIOColor::DarkGray,
            9 => QIOColor::Indigo,
            10 => QIOColor::BrightGreen,
            11 => QIOColor::Aqua,
            12 => QIOColor::Red,
            13 => QIOColor::Pink,
            14 => QIOColor::Yellow,
            15 => QIOColor::White,
            16 => QIOColor::Brown,
            17 => QIOColor::BrightPink,
            _ => QIOColor::Indigo,
        }
    }
}

// ============================================================================
// 7. QIO DRIVE DATA
// ============================================================================

/// Represents data of a single physical drive inside an array (`QIODriveData.java`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QIODriveData {
    pub slot: usize,
    pub tier: QIODriveTier,
    pub count_capacity: u64,
    pub type_capacity: usize,
    pub item_map: HashMap<HashedItem, u64>,
    pub item_count: u64,
}

impl QIODriveData {
    pub fn new(slot: usize, tier: QIODriveTier) -> Self {
        Self {
            slot,
            tier,
            count_capacity: tier.max_count(),
            type_capacity: tier.max_types(),
            item_map: HashMap::new(),
            item_count: 0,
        }
    }

    pub fn with_contents(slot: usize, tier: QIODriveTier, contents: DriveContents) -> Self {
        let mut item_map = HashMap::new();
        let mut item_count = 0u64;
        for (item, count) in contents.items {
            if count > 0 {
                item_count += count;
                *item_map.entry(item).or_insert(0) += count;
            }
        }
        Self {
            slot,
            tier,
            count_capacity: tier.max_count(),
            type_capacity: tier.max_types(),
            item_map,
            item_count,
        }
    }

    /// Add items to this drive (`QIODriveData.java:36-50`).
    /// Returns the rejected amount (0 if completely inserted).
    pub fn add(&mut self, item: &HashedItem, amount: u64, execute: bool) -> u64 {
        let stored = self.get_stored(item);
        // fail if we've reached item count capacity or adding this item would make us exceed type capacity
        if self.item_count == self.count_capacity || (stored == 0 && self.item_map.len() >= self.type_capacity) {
            return amount;
        }
        let space = self.count_capacity.saturating_sub(self.item_count);
        let to_add = amount.min(space);
        if execute && to_add > 0 {
            *self.item_map.entry(item.clone()).or_insert(0) += to_add;
            self.item_count += to_add;
        }
        amount - to_add
    }

    /// Remove items from this drive (`QIODriveData.java:52-67`).
    /// Returns the amount actually removed.
    pub fn remove(&mut self, item: &HashedItem, amount: u64, execute: bool) -> u64 {
        let stored = self.get_stored(item);
        let removed = amount.min(stored);
        if execute && removed > 0 {
            let remaining = stored - removed;
            if remaining > 0 {
                self.item_map.insert(item.clone(), remaining);
            } else {
                self.item_map.remove(item);
            }
            self.item_count -= removed;
        }
        removed
    }

    pub fn get_stored(&self, item: &HashedItem) -> u64 {
        self.item_map.get(item).copied().unwrap_or(0)
    }

    pub fn total_count(&self) -> u64 {
        self.item_count
    }

    pub fn total_types(&self) -> usize {
        self.item_map.len()
    }

    /// Status calculation (`TileEntityQIODriveArray.java:75-84`).
    pub fn status(&self) -> DriveStatus {
        if self.item_count == self.count_capacity {
            DriveStatus::Full
        } else if self.item_map.len() >= self.type_capacity || self.item_count >= (self.count_capacity * 3) / 4 {
            // Near full: type capacity reached OR >= 75% count capacity
            DriveStatus::NearFull
        } else {
            DriveStatus::Ready
        }
    }

    pub fn to_contents(&self) -> DriveContents {
        let mut items = Vec::with_capacity(self.item_map.len());
        for (item, count) in &self.item_map {
            items.push((item.clone(), *count));
        }
        DriveContents { items }
    }

    pub fn to_metadata(&self) -> DriveMetadata {
        DriveMetadata::new(self.item_count, self.item_map.len() as u32)
    }
}

// ============================================================================
// 8. QIO FREQUENCY & GLOBAL ITEM NETWORK
// ============================================================================

/// Location key of a drive in an array (`QIODriveKey.java:97`).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DriveKey {
    pub dimension: String,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub slot: usize,
}

/// Item type aggregate tracking inside a frequency (`QIOItemTypeData.java:680-779`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QIOItemTypeData {
    pub item: HashedItem,
    pub count: u64,
    pub containing_drives: HashSet<DriveKey>,
}

/// Global QIO Frequency network (`QIOFrequency.java:63-780`).
#[derive(Clone, Debug)]
pub struct QIOFrequency {
    pub name: String,
    pub owner: Option<u128>,
    pub owner_name: String,
    pub security_mode: SecurityMode,
    pub color: QIOColor,

    pub drive_map: BTreeMap<DriveKey, QIODriveData>,
    pub item_data_map: HashMap<HashedItem, QIOItemTypeData>,

    pub total_count: u64,
    pub total_count_capacity: u64,
    pub total_type_capacity: usize,

    pub is_dirty: bool,
}

impl QIOFrequency {
    pub fn new(name: &str, owner: Option<u128>, owner_name: &str, security_mode: SecurityMode) -> Self {
        Self {
            name: name.to_owned(),
            owner,
            owner_name: owner_name.to_owned(),
            security_mode,
            color: QIOColor::Indigo,
            drive_map: BTreeMap::new(),
            item_data_map: HashMap::new(),
            total_count: 0,
            total_count_capacity: 0,
            total_type_capacity: 0,
            is_dirty: false,
        }
    }

    /// Add a physical drive to this frequency (`QIOFrequency.java:591-610`).
    pub fn add_drive(&mut self, key: DriveKey, drive: QIODriveData) {
        if self.drive_map.contains_key(&key) {
            self.remove_drive(&key, true);
        }
        self.total_count_capacity += drive.count_capacity;
        self.total_type_capacity += drive.type_capacity;

        for (item, count) in &drive.item_map {
            if *count > 0 {
                self.total_count += *count;
                let entry = self.item_data_map.entry(item.clone()).or_insert_with(|| QIOItemTypeData {
                    item: item.clone(),
                    count: 0,
                    containing_drives: HashSet::new(),
                });
                entry.count += *count;
                entry.containing_drives.insert(key.clone());
            }
        }
        self.drive_map.insert(key, drive);
        self.is_dirty = true;
    }

    /// Remove a drive from this frequency (`QIOFrequency.java:612-643`).
    pub fn remove_drive(&mut self, key: &DriveKey, update_items: bool) -> Option<QIODriveData> {
        let drive = self.drive_map.remove(key)?;
        self.total_count_capacity = self.total_count_capacity.saturating_sub(drive.count_capacity);
        self.total_type_capacity = self.total_type_capacity.saturating_sub(drive.type_capacity);

        if update_items {
            for (item, count) in &drive.item_map {
                self.total_count = self.total_count.saturating_sub(*count);
                if let Some(entry) = self.item_data_map.get_mut(item) {
                    entry.containing_drives.remove(key);
                    entry.count = entry.count.saturating_sub(*count);
                    if entry.count == 0 || entry.containing_drives.is_empty() {
                        self.item_data_map.remove(item);
                    }
                }
            }
        }
        self.is_dirty = true;
        Some(drive)
    }

    /// Insert items into the frequency (`QIOFrequency.java:156-179`).
    /// Returns the quantity actually inserted.
    pub fn mass_insert(&mut self, item: &HashedItem, amount: u64, execute: bool) -> u64 {
        if item.is_empty() || amount == 0 {
            return 0;
        }
        // Capacity guards
        if self.total_count >= self.total_count_capacity
            || (!self.item_data_map.contains_key(item) && self.item_data_map.len() >= self.total_type_capacity)
        {
            return 0;
        }
        let rejects = self.add_item(item, amount, execute);
        amount.saturating_sub(rejects)
    }

    /// Core item addition across drives (`QIOFrequency.java:700-727`).
    /// Returns rejected amount.
    pub fn add_item(&mut self, item: &HashedItem, amount: u64, execute: bool) -> u64 {
        if item.is_empty() || amount == 0 {
            return amount;
        }
        if self.total_count >= self.total_count_capacity {
            return amount;
        }
        if !self.item_data_map.contains_key(item) && self.item_data_map.len() >= self.total_type_capacity {
            return amount;
        }

        let mut to_add = amount;

        // 1. First priority: try adding to drives that already contain this item (QIOFrequency.java:703-708)
        let existing_drives: Vec<DriveKey> = self
            .item_data_map
            .get(item)
            .map(|data| data.containing_drives.iter().cloned().collect())
            .unwrap_or_default();

        for key in &existing_drives {
            if let Some(drive) = self.drive_map.get_mut(key) {
                to_add = drive.add(item, to_add, execute);
                if to_add == 0 {
                    break;
                }
            }
        }

        // 2. Second priority: add to any drive that has room and type capacity (QIOFrequency.java:710-719)
        if to_add > 0 {
            let other_keys: Vec<DriveKey> = self
                .drive_map
                .keys()
                .filter(|k| !existing_drives.contains(k))
                .cloned()
                .collect();
            let mut new_containing = Vec::new();
            for key in other_keys {
                if let Some(drive) = self.drive_map.get_mut(&key) {
                    let before = to_add;
                    to_add = drive.add(item, to_add, execute);
                    if execute && to_add < before {
                        new_containing.push(key);
                    }
                    if to_add == 0 {
                        break;
                    }
                }
            }
            if execute && !new_containing.is_empty() {
                let entry = self.item_data_map.entry(item.clone()).or_insert_with(|| QIOItemTypeData {
                    item: item.clone(),
                    count: 0,
                    containing_drives: HashSet::new(),
                });
                for k in new_containing {
                    entry.containing_drives.insert(k);
                }
            }
        }

        let added = amount - to_add;
        if execute && added > 0 {
            self.total_count += added;
            let entry = self.item_data_map.entry(item.clone()).or_insert_with(|| QIOItemTypeData {
                item: item.clone(),
                count: 0,
                containing_drives: HashSet::new(),
            });
            entry.count += added;
            self.is_dirty = true;
        }

        to_add
    }

    /// Extract items from the frequency (`QIOFrequency.java:238-253`).
    /// Returns the quantity actually extracted.
    pub fn mass_extract(&mut self, item: &HashedItem, amount: u64, execute: bool) -> u64 {
        if amount == 0 || item.is_empty() || self.item_data_map.is_empty() {
            return 0;
        }
        self.remove_item(item, amount, execute)
    }

    /// Core item extraction across drives (`QIOFrequency.java:737-757`).
    /// Returns amount actually removed.
    pub fn remove_item(&mut self, item: &HashedItem, amount: u64, execute: bool) -> u64 {
        if amount == 0 || item.is_empty() {
            return 0;
        }
        let (target_remove, drive_keys) = match self.item_data_map.get(item) {
            Some(type_data) => (
                amount.min(type_data.count),
                type_data.containing_drives.iter().cloned().collect::<Vec<_>>(),
            ),
            None => return 0,
        };
        let mut removed = 0u64;
        let mut empty_drives = Vec::new();

        for key in drive_keys {
            if let Some(drive) = self.drive_map.get_mut(&key) {
                let want = target_remove - removed;
                let got = drive.remove(item, want, execute);
                removed += got;
                if execute && drive.get_stored(item) == 0 {
                    empty_drives.push(key);
                }
                if removed >= target_remove {
                    break;
                }
            }
        }

        if execute && removed > 0 {
            self.total_count = self.total_count.saturating_sub(removed);
            if let Some(data) = self.item_data_map.get_mut(item) {
                data.count = data.count.saturating_sub(removed);
                for key in empty_drives {
                    data.containing_drives.remove(&key);
                }
                if data.count == 0 || data.containing_drives.is_empty() {
                    self.item_data_map.remove(item);
                }
            }
            self.is_dirty = true;
        }

        removed
    }

    /// Removes an arbitrary item from the index (`QIOFrequency.java:255-257`).
    pub fn remove_any(&mut self, amount: u64) -> Option<(HashedItem, u64)> {
        let first_item = self.item_data_map.keys().next()?.clone();
        let removed = self.remove_item(&first_item, amount, true);
        if removed > 0 {
            Some((first_item, removed))
        } else {
            None
        }
    }

    /// Exact item count query (`QIOFrequency.java:456-459`).
    pub fn get_stored(&self, item: &HashedItem) -> u64 {
        self.item_data_map.get(item).map(|d| d.count).unwrap_or(0)
    }

    /// Fuzzy item count query: sums all variants matching base ID (`TileEntityQIORedstoneAdapter.java:59`).
    pub fn get_stored_fuzzy(&self, item_id: &str) -> u64 {
        self.item_data_map
            .iter()
            .filter(|(item, _)| item.matches_fuzzy(item_id))
            .map(|(_, data)| data.count)
            .sum()
    }

    /// Sync hash formula matching Java (`QIOFrequency.java:581-589`).
    pub fn get_sync_hash(&self) -> u64 {
        let mut code = 1u64;
        code = code.wrapping_mul(31).wrapping_add(self.total_count);
        code = code.wrapping_mul(31).wrapping_add(self.total_count_capacity);
        code = code.wrapping_mul(31).wrapping_add(self.item_data_map.len() as u64);
        code = code.wrapping_mul(31).wrapping_add(self.total_type_capacity as u64);
        code = code.wrapping_mul(31).wrapping_add(self.color as u64);
        code
    }

    /// Save all drives data for serialization (`QIOFrequency.java:645-652`).
    pub fn save_all(&self) -> Vec<(DriveKey, DriveMetadata, DriveContents)> {
        let mut out = Vec::with_capacity(self.drive_map.len());
        for (key, drive) in &self.drive_map {
            out.push((key.clone(), drive.to_metadata(), drive.to_contents()));
        }
        out
    }

    /// Check if a player can access this frequency (`SecurityFrequency.java`, `QIOFrequency.java:485`).
    pub fn can_access(&self, player: u128, trusted_list: &[u128]) -> bool {
        match self.security_mode {
            SecurityMode::Public => true,
            SecurityMode::Private => self.owner == Some(player),
            SecurityMode::Trusted => self.owner == Some(player) || trusted_list.contains(&player),
        }
    }
}

// ============================================================================
// 9. QIO DRIVE ARRAY (Tile entity and slot logic)
// ============================================================================

/// State of a QIO Drive Array block entity (`TileEntityQIODriveArray.java:35-230`).
#[derive(Clone, Debug)]
pub struct DriveArrayState {
    pub facing: String,
    pub active: bool,
    pub frequency_name: Option<String>,
    pub frequency_owner: Option<u128>,
    pub drive_statuses: [DriveStatus; DRIVE_SLOTS],
    pub drives: [Option<(QIODriveTier, DriveContents)>; DRIVE_SLOTS],
    pub prev_drive_hash: i32,
}

impl Default for DriveArrayState {
    fn default() -> Self {
        Self {
            facing: "north".to_owned(),
            active: false,
            frequency_name: None,
            frequency_owner: None,
            drive_statuses: [DriveStatus::None; DRIVE_SLOTS],
            drives: [
                None, None, None, None, None, None,
                None, None, None, None, None, None,
            ],
            prev_drive_hash: -1,
        }
    }
}

pub fn encode_drive_array(state: &DriveArrayState) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(1u8); // Version 1
    out.push(u8::from(state.active));
    let facing_bytes = state.facing.as_bytes();
    out.push(facing_bytes.len().min(16) as u8);
    out.extend(&facing_bytes[..facing_bytes.len().min(16)]);

    // Frequency name
    if let Some(name) = &state.frequency_name {
        let name_bytes = name.as_bytes();
        out.push(1u8);
        out.extend((name_bytes.len() as u16).to_le_bytes());
        out.extend(name_bytes);
    } else {
        out.push(0u8);
    }

    // Frequency owner UUID (u128)
    if let Some(owner) = state.frequency_owner {
        out.push(1u8);
        out.extend(owner.to_le_bytes());
    } else {
        out.push(0u8);
    }

    // Drive statuses: 12 bytes
    for st in &state.drive_statuses {
        out.push(st.to_u8());
    }

    // Number of drives
    for slot in 0..DRIVE_SLOTS {
        if let Some((tier, contents)) = &state.drives[slot] {
            out.push(1u8);
            out.push(match tier {
                QIODriveTier::Base => 0,
                QIODriveTier::HyperDense => 1,
                QIODriveTier::TimeDilating => 2,
                QIODriveTier::Supermassive => 3,
            });
            let encoded_contents = contents.encode();
            out.extend((encoded_contents.len() as u32).to_le_bytes());
            out.extend(encoded_contents);
        } else {
            out.push(0u8);
        }
    }

    out
}

pub fn decode_drive_array(data: &[u8]) -> Option<DriveArrayState> {
    if data.is_empty() || data[0] != 1 {
        return None;
    }
    let mut offset = 1;
    if offset >= data.len() {
        return None;
    }
    let active = data[offset] == 1;
    offset += 1;

    let facing_len = usize::from(data.get(offset).copied()?);
    offset += 1;
    if offset + facing_len > data.len() {
        return None;
    }
    let facing = std::str::from_utf8(&data[offset..offset + facing_len]).ok()?.to_owned();
    offset += facing_len;

    // Frequency name
    let has_name = *data.get(offset)? == 1;
    offset += 1;
    let frequency_name = if has_name {
        if offset + 2 > data.len() {
            return None;
        }
        let name_len = u16::from_le_bytes(data[offset..offset + 2].try_into().ok()?) as usize;
        offset += 2;
        if offset + name_len > data.len() {
            return None;
        }
        let name = std::str::from_utf8(&data[offset..offset + name_len]).ok()?.to_owned();
        offset += name_len;
        Some(name)
    } else {
        None
    };

    // Frequency owner
    let has_owner = *data.get(offset)? == 1;
    offset += 1;
    let frequency_owner = if has_owner {
        if offset + 16 > data.len() {
            return None;
        }
        let owner = u128::from_le_bytes(data[offset..offset + 16].try_into().ok()?);
        offset += 16;
        Some(owner)
    } else {
        None
    };

    // Drive statuses (12 slots)
    let mut drive_statuses = [DriveStatus::None; DRIVE_SLOTS];
    for s in 0..DRIVE_SLOTS {
        drive_statuses[s] = DriveStatus::from_u8(*data.get(offset)?);
        offset += 1;
    }

    // Drives
    let mut drives = [
        None, None, None, None, None, None,
        None, None, None, None, None, None,
    ];
    for slot in 0..DRIVE_SLOTS {
        let has_drive = *data.get(offset)? == 1;
        offset += 1;
        if has_drive {
            let tier_byte = *data.get(offset)?;
            offset += 1;
            let tier = match tier_byte {
                0 => QIODriveTier::Base,
                1 => QIODriveTier::HyperDense,
                2 => QIODriveTier::TimeDilating,
                _ => QIODriveTier::Supermassive,
            };
            if offset + 4 > data.len() {
                return None;
            }
            let c_len = u32::from_le_bytes(data[offset..offset + 4].try_into().ok()?) as usize;
            offset += 4;
            if offset + c_len > data.len() {
                return None;
            }
            let contents = DriveContents::decode(&data[offset..offset + c_len])?;
            offset += c_len;
            drives[slot] = Some((tier, contents));
        }
    }

    Some(DriveArrayState {
        facing,
        active,
        frequency_name,
        frequency_owner,
        drive_statuses,
        drives,
        prev_drive_hash: -1,
    })
}

/// Computes the 12 drive LED statuses from slot contents and connected frequency (`TileEntityQIODriveArray.java:68-85`).
pub fn update_drive_array_status(state: &mut DriveArrayState, freq: Option<&QIOFrequency>) -> bool {
    state.active = freq.is_some();
    for i in 0..DRIVE_SLOTS {
        let Some((tier, contents)) = &state.drives[i] else {
            state.drive_statuses[i] = DriveStatus::None;
            continue;
        };
        let Some(_frequency) = freq else {
            state.drive_statuses[i] = DriveStatus::Offline;
            continue;
        };
        let count_cap = tier.max_count();
        let type_cap = tier.max_types();
        let total_count = contents.total_count();
        let total_types = contents.total_types();

        if total_count >= count_cap {
            state.drive_statuses[i] = DriveStatus::Full;
        } else if total_types >= type_cap || total_count >= (count_cap * 3) / 4 {
            state.drive_statuses[i] = DriveStatus::NearFull;
        } else {
            state.drive_statuses[i] = DriveStatus::Ready;
        }
    }

    // Compute status hash
    let mut new_hash = 1i32;
    for st in &state.drive_statuses {
        new_hash = new_hash.wrapping_mul(31).wrapping_add(st.to_u8() as i32);
    }
    let changed = new_hash != state.prev_drive_hash;
    state.prev_drive_hash = new_hash;
    changed
}

// ============================================================================
// 10. QIO REDSTONE ADAPTER (Tile entity & 5-face signal logic)
// ============================================================================

/// State of a QIO Redstone Adapter (`TileEntityQIORedstoneAdapter.java:33-268`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RedstoneAdapterState {
    pub facing: String,
    pub active: bool,
    pub is_emitting: bool,
    pub item_type: Option<HashedItem>,
    pub count_trigger: u64,
    pub fuzzy: bool,
    pub inverted: bool,
    pub frequency_name: Option<String>,
    pub frequency_owner: Option<u128>,
}

impl Default for RedstoneAdapterState {
    fn default() -> Self {
        Self {
            facing: "north".to_owned(),
            active: false,
            is_emitting: false,
            item_type: None,
            count_trigger: 0,
            fuzzy: false,
            inverted: false,
            frequency_name: None,
            frequency_owner: None,
        }
    }
}

/// Evaluates if the redstone condition is met (`TileEntityQIORedstoneAdapter.java:101-103`).
/// hasStored = stored > 0 && stored >= count
/// shouldEmit = hasStored != inverted
pub fn evaluate_redstone_adapter(stored_count: u64, trigger: u64, inverted: bool) -> bool {
    let has_stored = stored_count > 0 && stored_count >= trigger;
    has_stored != inverted
}

/// Calculates redstone emission for a given side (`TileEntityQIORedstoneAdapter.java:47-49`).
/// Emits strength 15 on all sides EXCEPT the rear (opposite of facing direction)!
pub fn redstone_adapter_level(side: usize, facing_face: usize, active: bool, is_emitting: bool) -> u8 {
    let back = laser::opposite(facing_face);
    if side != back && active && is_emitting {
        15
    } else {
        0
    }
}

pub fn encode_redstone_adapter(state: &RedstoneAdapterState) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(1u8); // Version 1
    out.push(u8::from(state.active));
    out.push(u8::from(state.is_emitting));
    out.push(u8::from(state.fuzzy));
    out.push(u8::from(state.inverted));
    out.extend(state.count_trigger.to_le_bytes());

    let facing_bytes = state.facing.as_bytes();
    out.push(facing_bytes.len().min(16) as u8);
    out.extend(&facing_bytes[..facing_bytes.len().min(16)]);

    // Item type
    if let Some(item) = &state.item_type {
        out.push(1u8);
        let encoded_item = item.encode();
        out.extend((encoded_item.len() as u16).to_le_bytes());
        out.extend(encoded_item);
    } else {
        out.push(0u8);
    }

    // Frequency name
    if let Some(name) = &state.frequency_name {
        let name_bytes = name.as_bytes();
        out.push(1u8);
        out.extend((name_bytes.len() as u16).to_le_bytes());
        out.extend(name_bytes);
    } else {
        out.push(0u8);
    }

    // Frequency owner UUID
    if let Some(owner) = state.frequency_owner {
        out.push(1u8);
        out.extend(owner.to_le_bytes());
    } else {
        out.push(0u8);
    }

    out
}

pub fn decode_redstone_adapter(data: &[u8]) -> Option<RedstoneAdapterState> {
    if data.is_empty() || data[0] != 1 {
        return None;
    }
    if data.len() < 14 {
        return None;
    }
    let active = data[1] == 1;
    let is_emitting = data[2] == 1;
    let fuzzy = data[3] == 1;
    let inverted = data[4] == 1;
    let count_trigger = u64::from_le_bytes(data[5..13].try_into().ok()?);

    let mut offset = 13;
    let facing_len = usize::from(*data.get(offset)?);
    offset += 1;
    if offset + facing_len > data.len() {
        return None;
    }
    let facing = std::str::from_utf8(&data[offset..offset + facing_len]).ok()?.to_owned();
    offset += facing_len;

    // Item type
    let has_item = *data.get(offset)? == 1;
    offset += 1;
    let item_type = if has_item {
        if offset + 2 > data.len() {
            return None;
        }
        let item_len = u16::from_le_bytes(data[offset..offset + 2].try_into().ok()?) as usize;
        offset += 2;
        if offset + item_len > data.len() {
            return None;
        }
        let (item, _) = HashedItem::decode(&data[offset..offset + item_len])?;
        offset += item_len;
        Some(item)
    } else {
        None
    };

    // Frequency name
    let has_name = *data.get(offset)? == 1;
    offset += 1;
    let frequency_name = if has_name {
        if offset + 2 > data.len() {
            return None;
        }
        let name_len = u16::from_le_bytes(data[offset..offset + 2].try_into().ok()?) as usize;
        offset += 2;
        if offset + name_len > data.len() {
            return None;
        }
        let name = std::str::from_utf8(&data[offset..offset + name_len]).ok()?.to_owned();
        offset += name_len;
        Some(name)
    } else {
        None
    };

    // Frequency owner UUID
    let has_owner = *data.get(offset)? == 1;
    offset += 1;
    let frequency_owner = if has_owner {
        if offset + 16 > data.len() {
            return None;
        }
        let owner = u128::from_le_bytes(data[offset..offset + 16].try_into().ok()?);
        Some(owner)
    } else {
        None
    };

    Some(RedstoneAdapterState {
        facing,
        active,
        is_emitting,
        item_type,
        count_trigger,
        fuzzy,
        inverted,
        frequency_name,
        frequency_owner,
    })
}

// ============================================================================
// 11. THIN FORK HOOKS (Calls into host platform extensions)
// ============================================================================

// FORK-18: Host-side persistence and lookup for named frequencies by UUID and security mode
pub fn host_lookup_frequency(name: &str, owner: Option<u128>) -> Option<QIOFrequency> {
    let _ = crate::fork_qio_calor::lookup_frequency(name, owner);
    None
}

// FORK-20: Custom GUI layout and widget rendering (text field, virtual scroll, LEDs)
pub fn host_open_qio_screen(player_id: u64, screen_type: &str, data: &[u8]) {
    let _ = crate::fork_qio_calor::open_qio_screen(player_id, screen_type, data);
}

// FORK-21: Emitting redstone signal strength per face to adjacent blocks
pub fn host_set_redstone_output(dimension: &str, pos: [i32; 3], _side: usize, level: u8) {
    let _ = horizon_mod_sdk::server::world::set_redstone_output(dimension, pos[0], pos[1], pos[2], level, false);
}

// FORK-23: Reading and writing item data components (DriveMetadata, DriveContents)
pub fn host_set_item_components(_stack: &mut ItemStack, _metadata: &DriveMetadata, _contents: &DriveContents) {
    // Isolated thin call
}

// FORK-24: Sincronização em lote cliente-servidor (BulkQIOData)
pub fn host_broadcast_bulk_qio_data(dimension: &str, pos: [i32; 3], data: &[u8]) {
    let _ = crate::fork_qio_calor::broadcast_bulk_qio_data(dimension, pos, data);
}

/// What the drive array's screen shows (`GuiQIOFrequencyDataScreen`): the numbers (0 active, 1 how full the items are and 2 how full the types, in tenths of a
/// percent, 3 has a frequency) and the strings (0 the frequency's name, 1 the items stored of the capacity, 2 the types).
pub fn drive_array_screen(state: &DriveArrayState, freq: Option<&QIOFrequency>) -> (Vec<i32>, Vec<String>) {
    let thousands = |n: u64, d: u64| if d == 0 { 0 } else { (u128::from(n) * 1000 / u128::from(d)).min(1000) as i32 };
    match freq {
        Some(f) => {
            let types = f.item_data_map.len() as u64;
            (
                vec![i32::from(state.active), thousands(f.total_count, f.total_count_capacity), thousands(types, f.total_type_capacity as u64), 1],
                vec![
                    format!("Frequency: {}", f.name),
                    format!("Items: {} / {}", f.total_count, f.total_count_capacity),
                    format!("Types: {} / {}", types, f.total_type_capacity),
                ],
            )
        }
        None => (vec![i32::from(state.active), 0, 0, 0], vec![String::new(), String::new(), String::new()]),
    }
}

// ============================================================================
// 12. QIO STORAGE RUNNER & MANAGER
// ============================================================================

/// In-memory manager for QIO frequencies and interaction buttons.
#[derive(Default)]
pub struct QIOManager {
    pub frequencies: HashMap<String, QIOFrequency>,
    /// The strings last given to each drive array's screen (`world.set-menu-text`).
    pub menu_texts: HashMap<(String, i32, i32, i32), Vec<String>>,
    pub drive_array_buttons: Vec<(String, BlockPos, i32)>,
    pub redstone_adapter_buttons: Vec<(String, BlockPos, i32)>,
    pub prev_adapter_emissions: HashMap<(String, i32, i32, i32), bool>,
}

impl QIOManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ticking runner for QIO Drive Arrays.
    pub fn run_drive_arrays(&mut self, arrays: &[&BlockEntity], tick: u64) -> Vec<BlockEntityUpdate> {
        let mut updates = Vec::new();
        for entity in arrays {
            let mut state = decode_drive_array(&entity.data).unwrap_or_default();
            let freq = state.frequency_name.as_ref().and_then(|name| self.frequencies.get(name));

            let changed = update_drive_array_status(&mut state, freq);
            let (numbers, texts) = drive_array_screen(&state, freq);
            let key = (entity.dimension.clone(), entity.pos.x, entity.pos.y, entity.pos.z);
            if self.menu_texts.get(&key) != Some(&texts) {
                let _ = world::set_menu_text(&entity.dimension, entity.pos.x, entity.pos.y, entity.pos.z, &texts);
                self.menu_texts.insert(key, texts);
            }
            if changed || tick % TICKS_PER_HALF_SECOND == 0 || entity.menu_data != numbers {
                // Update active block property
                let active_str = if state.active { "true" } else { "false" };
                let _ = world::set_block_property(&entity.dimension, entity.pos.x, entity.pos.y, entity.pos.z, "active", active_str);

                updates.push(BlockEntityUpdate {
                    dimension: entity.dimension.clone(),
                    pos: entity.pos,
                    data: encode_drive_array(&state),
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
        updates
    }

    /// Ticking runner for QIO Redstone Adapters.
    pub fn run_redstone_adapters(&mut self, adapters: &[&BlockEntity], _tick: u64) -> Vec<BlockEntityUpdate> {
        let mut updates = Vec::new();
        for entity in adapters {
            let at = (entity.dimension.clone(), entity.pos.x, entity.pos.y, entity.pos.z);
            let mut state = decode_redstone_adapter(&entity.data).unwrap_or_default();
            let freq = state.frequency_name.as_ref().and_then(|name| self.frequencies.get(name));
            state.active = freq.is_some();

            let stored = match (freq, &state.item_type) {
                (Some(f), Some(item)) => {
                    if state.fuzzy {
                        f.get_stored_fuzzy(&item.id)
                    } else {
                        f.get_stored(item)
                    }
                }
                _ => 0,
            };

            let should_emit = evaluate_redstone_adapter(stored, state.count_trigger, state.inverted);
            let prev_emit = self.prev_adapter_emissions.get(&at).copied().unwrap_or(false);

            if state.is_emitting != should_emit || state.is_emitting != prev_emit {
                state.is_emitting = should_emit;
                self.prev_adapter_emissions.insert(at, should_emit);

                let facing_face = laser::face_of(&state.facing);
                // One level for every side (the host's output is the block's, not a face's): the rear side, which the original leaves out, gets it too.
                let level = (0..6).map(|side| redstone_adapter_level(side, facing_face, state.active, state.is_emitting)).max().unwrap_or(0);
                host_set_redstone_output(&entity.dimension, [entity.pos.x, entity.pos.y, entity.pos.z], 0, level);

                updates.push(BlockEntityUpdate {
                    dimension: entity.dimension.clone(),
                    pos: entity.pos,
                    data: encode_redstone_adapter(&state),
                    items: None,
                    energy: None,
                    energy_faces: None,
                    tanks: None,
                    tank_faces: None,
                    menu_data: None,
                    transmitter_faces: None,
                });
            }
        }
        updates
    }
}

// ============================================================================
// 13. UNIT TESTS (Faithfully comparing with Java source values)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies QIO Drive tier capacities match `QIODriveTier.java:7-10`.
    #[test]
    fn test_drive_tiers_capacity() {
        // Base tier: 16,000 items, 128 types (QIODriveTier.java:7)
        assert_eq!(QIODriveTier::Base.max_count(), 16_000);
        assert_eq!(QIODriveTier::Base.max_types(), 128);
        assert_eq!(QIODriveTier::Base.registry_name(), "mekanism:qio_drive_base");

        // Hyper-Dense tier: 128,000 items, 256 types (QIODriveTier.java:8)
        assert_eq!(QIODriveTier::HyperDense.max_count(), 128_000);
        assert_eq!(QIODriveTier::HyperDense.max_types(), 256);
        assert_eq!(QIODriveTier::HyperDense.registry_name(), "mekanism:qio_drive_hyper_dense");

        // Time-Dilating tier: 1,048,000 items, 1,024 types (QIODriveTier.java:9)
        assert_eq!(QIODriveTier::TimeDilating.max_count(), 1_048_000);
        assert_eq!(QIODriveTier::TimeDilating.max_types(), 1_024);
        assert_eq!(QIODriveTier::TimeDilating.registry_name(), "mekanism:qio_drive_time_dilating");

        // Supermassive tier: 16,000,000,000 items, 8,192 types (QIODriveTier.java:10)
        assert_eq!(QIODriveTier::Supermassive.max_count(), 16_000_000_000);
        assert_eq!(QIODriveTier::Supermassive.max_types(), 8_192);
        assert_eq!(QIODriveTier::Supermassive.registry_name(), "mekanism:qio_drive_supermassive");
    }

    /// Verifies serialization and roundtrip of DriveMetadata (`DriveMetadata.java:13-25`).
    #[test]
    fn test_drive_metadata_encode_decode() {
        let meta = DriveMetadata::new(125_400, 42);
        let bytes = meta.encode();
        assert_eq!(bytes.len(), 12);
        let decoded = DriveMetadata::decode(&bytes).expect("Failed to decode DriveMetadata");
        assert_eq!(decoded.count, 125_400);
        assert_eq!(decoded.types, 42);
        assert!(!decoded.is_empty());

        let empty = DriveMetadata::default();
        assert!(empty.is_empty());
    }

    /// Verifies serialization and roundtrip of DriveContents (`DriveContents.java:23,73-105`).
    #[test]
    fn test_drive_contents_encode_decode() {
        let mut contents = DriveContents::new();
        contents.items.push((HashedItem::raw("minecraft:iron_ingot"), 500));
        contents.items.push((HashedItem::with_components("mekanism:enriched_iron", &[1, 2, 3]), 250));

        let bytes = contents.encode();
        let decoded = DriveContents::decode(&bytes).expect("Failed to decode DriveContents");
        assert_eq!(decoded.items.len(), 2);
        assert_eq!(decoded.total_count(), 750);
        assert_eq!(decoded.items[0].0.id, "minecraft:iron_ingot");
        assert_eq!(decoded.items[0].1, 500);
        assert_eq!(decoded.items[1].0.id, "mekanism:enriched_iron");
        assert_eq!(decoded.items[1].0.components, vec![1, 2, 3]);
        assert_eq!(decoded.items[1].1, 250);
    }

    /// Verifies QIODriveData addition, capacity limits and rejection (`QIODriveData.java:36-50`).
    #[test]
    fn test_qio_drive_data_add_remove() {
        let mut drive = QIODriveData::new(0, QIODriveTier::Base);
        assert_eq!(drive.count_capacity, 16_000);
        assert_eq!(drive.type_capacity, 128);

        let iron = HashedItem::raw("minecraft:iron_ingot");
        let gold = HashedItem::raw("minecraft:gold_ingot");

        // Simulation does not alter state
        let rejects_sim = drive.add(&iron, 100, false);
        assert_eq!(rejects_sim, 0);
        assert_eq!(drive.total_count(), 0);

        // Real addition
        let rejects = drive.add(&iron, 10_000, true);
        assert_eq!(rejects, 0);
        assert_eq!(drive.total_count(), 10_000);
        assert_eq!(drive.get_stored(&iron), 10_000);

        // Add exceeding item capacity: space is 6_000
        let rejects2 = drive.add(&gold, 8_000, true);
        assert_eq!(rejects2, 2_000); // 8000 - 6000 = 2000 rejected
        assert_eq!(drive.total_count(), 16_000);
        assert_eq!(drive.get_stored(&gold), 6_000);

        // Full drive rejects everything
        let copper = HashedItem::raw("minecraft:copper_ingot");
        let rejects3 = drive.add(&copper, 50, true);
        assert_eq!(rejects3, 50);

        // Remove items
        let removed = drive.remove(&iron, 4_000, true);
        assert_eq!(removed, 4_000);
        assert_eq!(drive.get_stored(&iron), 6_000);
        assert_eq!(drive.total_count(), 12_000);
    }

    /// Verifies drive status LED transitions (`TileEntityQIODriveArray.java:75-84`).
    #[test]
    fn test_drive_status_transitions() {
        let mut drive = QIODriveData::new(0, QIODriveTier::Base);
        assert_eq!(drive.status(), DriveStatus::Ready);

        // 75% threshold: 16_000 * 0.75 = 12_000 items (TileEntityQIODriveArray.java:78)
        let item = HashedItem::raw("minecraft:redstone");
        drive.add(&item, 11_999, true);
        assert_eq!(drive.status(), DriveStatus::Ready);

        drive.add(&item, 1, true); // Now 12_000 (75%)
        assert_eq!(drive.status(), DriveStatus::NearFull);

        // 100% threshold: 16_000 items
        drive.add(&item, 4_000, true);
        assert_eq!(drive.status(), DriveStatus::Full);
    }

    /// Verifies QIOFrequency multi-drive aggregation and insert priority (`QIOFrequency.java:598-600, 701-719`).
    #[test]
    fn test_qio_frequency_aggregate_network() {
        let mut freq = QIOFrequency::new("TestNet", Some(12345), "Player1", SecurityMode::Public);
        assert_eq!(freq.color, QIOColor::Indigo);

        let key1 = DriveKey { dimension: "minecraft:overworld".into(), x: 0, y: 64, z: 0, slot: 0 };
        let key2 = DriveKey { dimension: "minecraft:overworld".into(), x: 0, y: 64, z: 0, slot: 1 };

        let drive1 = QIODriveData::new(0, QIODriveTier::Base); // 16,000 count, 128 types
        let drive2 = QIODriveData::new(1, QIODriveTier::HyperDense); // 128,000 count, 256 types

        freq.add_drive(key1.clone(), drive1);
        freq.add_drive(key2.clone(), drive2);

        assert_eq!(freq.total_count_capacity, 16_000 + 128_000);
        assert_eq!(freq.total_type_capacity, 128 + 256);

        let diamond = HashedItem::raw("minecraft:diamond");
        let inserted = freq.mass_insert(&diamond, 20_000, true);
        assert_eq!(inserted, 20_000);
        assert_eq!(freq.total_count, 20_000);
        assert_eq!(freq.get_stored(&diamond), 20_000);

        // Verification that drive 1 took 16,000 and drive 2 took remaining 4,000
        assert_eq!(freq.drive_map.get(&key1).unwrap().get_stored(&diamond), 16_000);
        assert_eq!(freq.drive_map.get(&key2).unwrap().get_stored(&diamond), 4_000);

        // Extraction
        let extracted = freq.mass_extract(&diamond, 5_000, true);
        assert_eq!(extracted, 5_000);
        assert_eq!(freq.get_stored(&diamond), 15_000);
        assert_eq!(freq.total_count, 15_000);
    }

    /// Verifies fuzzy search across items with different components (`TileEntityQIORedstoneAdapter.java:59`).
    #[test]
    fn test_qio_frequency_fuzzy_lookup() {
        let mut freq = QIOFrequency::new("FuzzyNet", None, "Server", SecurityMode::Public);
        let key = DriveKey { dimension: "minecraft:overworld".into(), x: 10, y: 70, z: 10, slot: 0 };
        let drive = QIODriveData::new(0, QIODriveTier::HyperDense);
        freq.add_drive(key, drive);

        let sword_plain = HashedItem::raw("minecraft:diamond_sword");
        let sword_enchanted = HashedItem::with_components("minecraft:diamond_sword", &[9, 9, 9]);

        freq.mass_insert(&sword_plain, 5, true);
        freq.mass_insert(&sword_enchanted, 3, true);

        // Exact lookups
        assert_eq!(freq.get_stored(&sword_plain), 5);
        assert_eq!(freq.get_stored(&sword_enchanted), 3);

        // Fuzzy lookup sums all items with base ID "minecraft:diamond_sword"
        assert_eq!(freq.get_stored_fuzzy("minecraft:diamond_sword"), 8);
    }

    /// Verifies QIO Redstone Adapter conditions and 5-side emission (`TileEntityQIORedstoneAdapter.java:48, 102-103`).
    #[test]
    fn test_qio_redstone_adapter_conditions_and_emission() {
        // hasStored = stored > 0 && stored >= count
        // shouldEmit = hasStored != inverted
        assert!(!evaluate_redstone_adapter(0, 10, false));
        assert!(!evaluate_redstone_adapter(9, 10, false));
        assert!(evaluate_redstone_adapter(10, 10, false));
        assert!(evaluate_redstone_adapter(15, 10, false));

        // Inverted mode: emits when condition is NOT met
        assert!(evaluate_redstone_adapter(0, 10, true));
        assert!(evaluate_redstone_adapter(9, 10, true));
        assert!(!evaluate_redstone_adapter(10, 10, true));

        // Facing north (laser::NORTH = 2, laser::opposite(NORTH) = SOUTH = 3)
        let facing_north = laser::NORTH;
        let active = true;
        let is_emitting = true;

        // Front, Top, Bottom, West, East emit strength 15
        assert_eq!(redstone_adapter_level(laser::NORTH, facing_north, active, is_emitting), 15);
        assert_eq!(redstone_adapter_level(laser::UP, facing_north, active, is_emitting), 15);
        assert_eq!(redstone_adapter_level(laser::DOWN, facing_north, active, is_emitting), 15);
        assert_eq!(redstone_adapter_level(laser::WEST, facing_north, active, is_emitting), 15);
        assert_eq!(redstone_adapter_level(laser::EAST, facing_north, active, is_emitting), 15);

        // Opposite side (SOUTH = back) emits 0!
        assert_eq!(redstone_adapter_level(laser::SOUTH, facing_north, active, is_emitting), 0);

        // Inactive adapter emits 0 on all sides
        assert_eq!(redstone_adapter_level(laser::NORTH, facing_north, false, is_emitting), 0);
    }

    /// Verifies Drive Array encode/decode roundtrip.
    #[test]
    fn test_drive_array_serialization() {
        let mut state = DriveArrayState::default();
        state.facing = "east".to_owned();
        state.active = true;
        state.frequency_name = Some("MainVault".to_owned());
        state.frequency_owner = Some(0xCAFEBABE_1234_5678);
        state.drive_statuses[0] = DriveStatus::Ready;
        state.drive_statuses[1] = DriveStatus::Full;

        let mut contents0 = DriveContents::new();
        contents0.items.push((HashedItem::raw("minecraft:gold_ingot"), 100));
        state.drives[0] = Some((QIODriveTier::Base, contents0));

        let bytes = encode_drive_array(&state);
        let decoded = decode_drive_array(&bytes).expect("Failed to decode DriveArrayState");

        assert_eq!(decoded.facing, "east");
        assert!(decoded.active);
        assert_eq!(decoded.frequency_name.as_deref(), Some("MainVault"));
        assert_eq!(decoded.frequency_owner, Some(0xCAFEBABE_1234_5678));
        assert_eq!(decoded.drive_statuses[0], DriveStatus::Ready);
        assert_eq!(decoded.drive_statuses[1], DriveStatus::Full);
        assert_eq!(decoded.drives[0].as_ref().unwrap().0, QIODriveTier::Base);
        assert_eq!(decoded.drives[0].as_ref().unwrap().1.total_count(), 100);
    }

    /// Verifies Redstone Adapter encode/decode roundtrip.
    #[test]
    fn test_redstone_adapter_serialization() {
        let state = RedstoneAdapterState {
            facing: "up".to_owned(),
            active: true,
            is_emitting: true,
            item_type: Some(HashedItem::raw("mekanism:refined_obsidian_ingot")),
            count_trigger: 64,
            fuzzy: true,
            inverted: false,
            frequency_name: Some("AutomationNet".to_owned()),
            frequency_owner: None,
        };

        let bytes = encode_redstone_adapter(&state);
        let decoded = decode_redstone_adapter(&bytes).expect("Failed to decode RedstoneAdapterState");

        assert_eq!(decoded.facing, "up");
        assert!(decoded.active);
        assert!(decoded.is_emitting);
        assert_eq!(decoded.item_type.unwrap().id, "mekanism:refined_obsidian_ingot");
        assert_eq!(decoded.count_trigger, 64);
        assert!(decoded.fuzzy);
        assert!(!decoded.inverted);
        assert_eq!(decoded.frequency_name.as_deref(), Some("AutomationNet"));
    }

    /// Verifies Security Mode permission logic (`SecurityFrequency.java`, `QIOFrequency.java:485`).
    #[test]
    fn test_security_mode_permissions() {
        let owner: u128 = 0x1111_2222_3333_4444;
        let friend: u128 = 0x5555_6666_7777_8888;
        let stranger: u128 = 0x9999_AAAA_BBBB_CCCC;
        let trusted_list = vec![friend];

        let mut freq = QIOFrequency::new("SecTest", Some(owner), "OwnerPlayer", SecurityMode::Public);
        assert!(freq.can_access(stranger, &trusted_list));

        freq.security_mode = SecurityMode::Private;
        assert!(freq.can_access(owner, &trusted_list));
        assert!(!freq.can_access(friend, &trusted_list));
        assert!(!freq.can_access(stranger, &trusted_list));

        freq.security_mode = SecurityMode::Trusted;
        assert!(freq.can_access(owner, &trusted_list));
        assert!(freq.can_access(friend, &trusted_list));
        assert!(!freq.can_access(stranger, &trusted_list));
    }

    #[test]
    fn the_drive_array_screen_says_how_full_the_frequency_is() {
        let state = DriveArrayState::default();
        let (n, texts) = drive_array_screen(&state, None);
        assert_eq!(n, vec![0, 0, 0, 0]);
        assert!(texts.iter().all(String::is_empty));
        let mut f = QIOFrequency::new("main", None, "", SecurityMode::Public);
        f.total_count = 500;
        f.total_count_capacity = 2_000;
        f.total_type_capacity = 10;
        let (n, texts) = drive_array_screen(&state, Some(&f));
        assert_eq!((n[1], n[3]), (250, 1));
        assert_eq!(texts[0], "Frequency: main");
        assert_eq!(texts[1], "Items: 500 / 2000");
    }
}

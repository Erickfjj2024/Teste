//! HDPE Reinforced Elytra implementation for Horizon (Rust / WASM mod).
//!
//! Faithful port of Mekanism 1.21.1-10.7.19.85:
//! - Java item: `mekanism.common.item.gear.ItemHDPEElytra` (`ItemHDPEElytra.java`)
//! - Java registration: `MekanismItems.java:184`
//! - Durability: 648 (1.5x vanilla elytra durability 432: `432 * 3 / 2 = 648`)
//! - Rarity: RARE
//! - Equipment slot: CHEST (`EquipmentSlot.CHEST`, `ItemHDPEElytra.java:19`)
//! - Repair material in anvil: `mekanism:hdpe_sheet` (`ItemHDPEElytra.java:24`)

pub const HDPE_ELYTRA: &str = "mekanism:hdpe_elytra";

// -----------------------------------------------------------------------------
// Constants from Java (Mekanism 1.21.1, MekanismItems.java & ItemHDPEElytra.java)
// -----------------------------------------------------------------------------

/// Durability of HDPE Reinforced Elytra: 648 (`MekanismItems.java:184`).
/// Exactly 1.5x vanilla elytra durability (432).
pub const MAX_DURABILITY: u32 = 648;

/// Vanilla Elytra base durability for comparison.
pub const VANILLA_ELYTRA_DURABILITY: u32 = 432;

/// Item used to repair the elytra in an anvil (`ItemHDPEElytra.java:24`).
pub const REPAIR_ITEM: &str = "mekanism:hdpe_sheet";

/// Equipment slot where elytra is equipped (`ItemHDPEElytra.java:19`).
pub const EQUIPMENT_SLOT: &str = "chest";

/// Standard flight damage interval: 1 durability loss per 20 ticks (1 second of glide).
pub const FLIGHT_DAMAGE_INTERVAL_TICKS: u32 = 20;

/// Repair amount per HDPE Sheet in an anvil (25% of max durability: 648 / 4 = 162).
pub const REPAIR_PER_SHEET: u32 = MAX_DURABILITY / 4;

// -----------------------------------------------------------------------------
// HDPE Elytra State and Logic
// -----------------------------------------------------------------------------

/// State of an HDPE Elytra item stack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HdpeElytraData {
    /// Accumulated damage points (0 = pristine, 647 = broken/unusable, 648 = depleted).
    pub damage: u32,
}

impl Default for HdpeElytraData {
    fn default() -> Self {
        Self { damage: 0 }
    }
}

impl HdpeElytraData {
    pub fn new(damage: u32) -> Self {
        Self {
            damage: damage.min(MAX_DURABILITY),
        }
    }

    /// Remaining durability points.
    pub const fn remaining_durability(&self) -> u32 {
        MAX_DURABILITY.saturating_sub(self.damage)
    }

    /// Whether the elytra is broken and cannot glide (durability <= 1, damage >= 647).
    /// Follows vanilla ElytraItem rules where an elytra with 1 durability stops functioning
    /// instead of breaking entirely, allowing repair in an anvil.
    pub const fn is_broken(&self) -> bool {
        self.remaining_durability() <= 1
    }

    /// Whether the elytra is in a valid state for gliding.
    pub const fn can_glide(&self) -> bool {
        !self.is_broken()
    }

    /// Applies flight damage over a given number of flight ticks.
    /// Returns the number of durability points deducted.
    pub fn apply_flight_damage(&mut self, flight_ticks: u32) -> u32 {
        if self.is_broken() {
            return 0;
        }

        let damage_to_apply = flight_ticks / FLIGHT_DAMAGE_INTERVAL_TICKS;
        if damage_to_apply == 0 {
            return 0;
        }

        // Cap damage at MAX_DURABILITY - 1 so the item is preserved in broken state
        let max_damage = MAX_DURABILITY - 1;
        let actual_damage = damage_to_apply.min(max_damage.saturating_sub(self.damage));
        self.damage += actual_damage;
        actual_damage
    }

    /// Checks if a given item ID is valid for anvil repair (`ItemHDPEElytra.java:24`).
    pub fn is_valid_repair_item(item_id: &str) -> bool {
        item_id == REPAIR_ITEM || item_id == "hdpe_sheet"
    }

    /// Repairs the elytra using HDPE Sheets in an anvil.
    /// Returns `(sheets_consumed, durability_restored)`.
    pub fn repair_with_sheets(&mut self, available_sheets: u32) -> (u32, u32) {
        if self.damage == 0 || available_sheets == 0 {
            return (0, 0);
        }

        let mut sheets_used = 0u32;
        let mut restored_total = 0u32;

        while self.damage > 0 && sheets_used < available_sheets {
            let restore = REPAIR_PER_SHEET.min(self.damage);
            self.damage -= restore;
            restored_total += restore;
            sheets_used += 1;
        }

        (sheets_used, restored_total)
    }
}

// -----------------------------------------------------------------------------
// Data Components Serialization (SNBT / JSON) (FORK-23)
// -----------------------------------------------------------------------------

/// Serializes HdpeElytraData into Horizon data components JSON.
pub fn serialize_to_components(data: &HdpeElytraData) -> String {
    format!(
        r#"{{"minecraft:damage":{},"minecraft:max_damage":{}}}"#,
        data.damage, MAX_DURABILITY
    )
}

/// Parses HdpeElytraData from Horizon data components JSON.
pub fn parse_from_components(components: &str) -> Option<HdpeElytraData> {
    if components.trim().is_empty() {
        return Some(HdpeElytraData::default());
    }

    let mut damage = 0u32;
    if let Some(pos) = components.find("\"minecraft:damage\":") {
        let rest = &components[pos + 19..];
        let num = rest.split(|c: char| !c.is_ascii_digit()).next().unwrap_or("");
        if let Ok(val) = num.parse::<u32>() {
            damage = val;
        }
    }

    Some(HdpeElytraData::new(damage))
}

// -----------------------------------------------------------------------------
// Host API Hooks / Stubs (Marked with FORK-n)
// -----------------------------------------------------------------------------

/// Reads HDPE Elytra state from ItemStack data components.
// FORK-23
pub fn host_read_hdpe_elytra(components_str: &str) -> HdpeElytraData {
    parse_from_components(components_str).unwrap_or_default()
}

/// Writes HDPE Elytra state to ItemStack data components.
// FORK-23
pub fn host_write_hdpe_elytra(data: &HdpeElytraData) -> String {
    serialize_to_components(data)
}

/// Flight tick check for player gliding with HDPE Elytra.
// FORK-34
pub fn host_tick_elytra_flight(
    _player_id: &str,
    _data: &mut HdpeElytraData,
    _flight_ticks: u32,
) -> bool {
    // Requires FORK-34 elytra flight physics & equipment wear hook in horizon-mod-sdk
    false
}

// -----------------------------------------------------------------------------
// Unit Tests (Comparing with Java 1.21.1 source of truth)
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durability_is_exactly_one_and_a_half_times_vanilla() {
        // MekanismItems.java:184: props.durability(648).rarity(Rarity.RARE)
        assert_eq!(MAX_DURABILITY, 648);
        assert_eq!(VANILLA_ELYTRA_DURABILITY, 432);
        assert_eq!(
            MAX_DURABILITY,
            VANILLA_ELYTRA_DURABILITY * 3 / 2,
            "HDPE Elytra durability 648 is exactly 1.5x vanilla elytra (432)"
        );
    }

    #[test]
    fn repair_item_matches_java() {
        // ItemHDPEElytra.java:24: return repair.is(MekanismItems.HDPE_SHEET);
        assert_eq!(REPAIR_ITEM, "mekanism:hdpe_sheet");
        assert!(HdpeElytraData::is_valid_repair_item("mekanism:hdpe_sheet"));
        assert!(HdpeElytraData::is_valid_repair_item("hdpe_sheet"));
        assert!(!HdpeElytraData::is_valid_repair_item("minecraft:phantom_membrane"));
        assert!(!HdpeElytraData::is_valid_repair_item("mekanism:ingot_steel"));
    }

    #[test]
    fn equipment_slot_is_chest() {
        // ItemHDPEElytra.java:19: return EquipmentSlot.CHEST;
        assert_eq!(EQUIPMENT_SLOT, "chest");
    }

    #[test]
    fn flight_wear_and_broken_state() {
        let mut elytra = HdpeElytraData::default();
        assert!(elytra.can_glide());
        assert_eq!(elytra.remaining_durability(), 648);

        // Fly for 100 ticks (5 seconds) -> 5 durability lost (1 per 20 ticks)
        let lost = elytra.apply_flight_damage(100);
        assert_eq!(lost, 5);
        assert_eq!(elytra.damage, 5);
        assert_eq!(elytra.remaining_durability(), 643);

        // Near breaking: damage 646 -> 2 durability left -> can still glide
        elytra.damage = 646;
        assert!(elytra.can_glide());

        // Fly for 20 more ticks -> damage 647 -> 1 durability left -> broken!
        elytra.apply_flight_damage(20);
        assert_eq!(elytra.damage, 647);
        assert_eq!(elytra.remaining_durability(), 1);
        assert!(elytra.is_broken());
        assert!(!elytra.can_glide(), "Cannot glide when durability <= 1");

        // Further flight ticks cause no further damage (preserved at 647)
        let no_more = elytra.apply_flight_damage(100);
        assert_eq!(no_more, 0);
        assert_eq!(elytra.damage, 647);
    }

    #[test]
    fn anvil_repair_with_hdpe_sheets() {
        // 25% of 648 = 162 per sheet
        assert_eq!(REPAIR_PER_SHEET, 162);

        let mut broken_elytra = HdpeElytraData::new(647);

        // 1 sheet repairs 162 points: 647 - 162 = 485 damage remaining
        let (used, restored) = broken_elytra.repair_with_sheets(1);
        assert_eq!(used, 1);
        assert_eq!(restored, 162);
        assert_eq!(broken_elytra.damage, 485);
        assert!(broken_elytra.can_glide(), "Restored to functional state");

        // 4 sheets available: needs only 3 more sheets to fully repair (485 - 3*162 = 485 - 486 -> 0)
        let (used2, restored2) = broken_elytra.repair_with_sheets(4);
        assert_eq!(used2, 3, "Only consumes 3 sheets to reach full repair");
        assert_eq!(restored2, 485);
        assert_eq!(broken_elytra.damage, 0);
        assert_eq!(broken_elytra.remaining_durability(), 648);
    }

    #[test]
    fn serialization_roundtrip() {
        let elytra = HdpeElytraData::new(123);
        let components = serialize_to_components(&elytra);
        assert!(components.contains("\"minecraft:damage\":123"));

        let restored = parse_from_components(&components).expect("Should parse");
        assert_eq!(restored.damage, 123);
        assert_eq!(restored.remaining_durability(), 648 - 123);
    }
}

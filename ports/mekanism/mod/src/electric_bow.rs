//! Electric Bow implementation for Horizon (Rust / WASM mod).
//!
//! Faithful port of Mekanism 1.21.1-10.7.19.85:
//! - Java item: `mekanism.common.item.gear.ItemElectricBow` (`ItemElectricBow.java`)
//! - Java config: `mekanism.common.config.GearConfig` (`GearConfig.java:146-151`)
//! - Data component: `mekanism:electric_bow_mode` (Boolean, default `false`)
//! - Energy conversion: 1 FE = 2.5 Joules (`fe = J * 2 / 5`).

pub const ELECTRIC_BOW: &str = "mekanism:electric_bow";

/// Energy conversion helper: 1 FE = 2.5 Joules.
pub const fn fe(joules: u64) -> u64 {
    joules * 2 / 5
}

// -----------------------------------------------------------------------------
// Constants from Java (Mekanism 1.21.1, GearConfig.java)
// -----------------------------------------------------------------------------

/// Maximum energy capacity: 120_000 J = 48_000 FE (`GearConfig.java:147`).
pub const MAX_ENERGY: u64 = fe(120_000);

/// Recharging rate: 600 J/t = 240 FE/t (`GearConfig.java:148`).
pub const CHARGE_RATE: u64 = fe(600);

/// Energy usage for firing a standard arrow: 120 J = 48 FE (`GearConfig.java:149`).
pub const ENERGY_USAGE: u64 = fe(120);

/// Energy usage for firing a flame arrow (Fire Mode active): 1_200 J = 480 FE (`GearConfig.java:150`).
pub const ENERGY_USAGE_FIRE: u64 = fe(1_200);

// -----------------------------------------------------------------------------
// Electric Bow State and Logic
// -----------------------------------------------------------------------------

/// State of an Electric Bow item stack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ElectricBowData {
    pub energy: u64,
    pub fire_mode: bool,
}

impl Default for ElectricBowData {
    fn default() -> Self {
        Self {
            energy: 0,
            fire_mode: false, // Default is false (ItemElectricBow.java:43, 120)
        }
    }
}

impl ElectricBowData {
    pub fn new(energy: u64, fire_mode: bool) -> Self {
        Self {
            energy: energy.min(MAX_ENERGY),
            fire_mode,
        }
    }

    /// Toggles between standard arrow mode and flame arrow mode (`ItemElectricBow.java:149-156`).
    /// Returns the new state.
    pub fn toggle_fire_mode(&mut self) -> bool {
        self.fire_mode = !self.fire_mode;
        self.fire_mode
    }

    /// Energy required for firing in current mode.
    /// Java: `getMode(bow) ? electricBowEnergyUsageFire : electricBowEnergyUsage` (`ItemElectricBow.java:56`).
    pub const fn energy_for_shot(&self) -> u64 {
        if self.fire_mode {
            ENERGY_USAGE_FIRE
        } else {
            ENERGY_USAGE_BASE
        }
    }

    /// Checks if the bow has enough energy to be released (`ItemElectricBow.java:53-62`).
    /// In creative mode, energy is not required.
    pub fn can_release(&self, is_creative: bool) -> bool {
        if is_creative {
            true
        } else {
            self.energy >= self.energy_for_shot()
        }
    }

    /// Fires an arrow, deducting energy in survival mode (`ItemElectricBow.java:64-76`).
    /// Returns `true` if the shot succeeded, or `false` if blocked by lack of energy.
    pub fn shoot(&mut self, is_creative: bool) -> bool {
        if is_creative {
            return true;
        }
        let needed = self.energy_for_shot();
        if self.energy >= needed {
            self.energy -= needed;
            true
        } else {
            false
        }
    }

    /// Enchantment level of FLAME provided dynamically (`ItemElectricBow.java:86-93`).
    /// When `fire_mode` is active, returns at least 1.
    pub const fn flame_enchantment_level(&self) -> u32 {
        if self.fire_mode {
            1
        } else {
            0
        }
    }

    /// Recharges energy, respecting max charge rate and capacity.
    /// Returns actual energy accepted.
    pub fn charge(&mut self, amount: u64) -> u64 {
        let max_transfer = amount.min(CHARGE_RATE);
        let space = MAX_ENERGY.saturating_sub(self.energy);
        let accepted = max_transfer.min(space);
        self.energy += accepted;
        accepted
    }
}

pub const ENERGY_USAGE_BASE: u64 = ENERGY_USAGE;

// -----------------------------------------------------------------------------
// Data Components Serialization (SNBT / JSON) (FORK-23)
// -----------------------------------------------------------------------------

/// Serializes ElectricBowData into Horizon data components JSON.
pub fn serialize_to_components(data: &ElectricBowData) -> String {
    format!(
        r#"{{"mekanism:energy":{{"energy_containers":[{}]}},"mekanism:electric_bow_mode":{}}}"#,
        data.energy, data.fire_mode
    )
}

/// Parses ElectricBowData from Horizon data components JSON.
pub fn parse_from_components(components: &str) -> Option<ElectricBowData> {
    if components.trim().is_empty() {
        return Some(ElectricBowData::default());
    }

    let mut energy = 0u64;
    if let Some(pos) = components.find("\"energy_containers\":[") {
        let rest = &components[pos + 21..];
        if let Some(end) = rest.find(']') {
            let num_str = rest[..end].trim();
            if let Ok(val) = num_str.parse::<u64>() {
                energy = val;
            }
        }
    }

    let mut fire_mode = false;
    if let Some(pos) = components.find("\"mekanism:electric_bow_mode\":") {
        let rest = &components[pos + 29..];
        if rest.starts_with("true") {
            fire_mode = true;
        }
    }

    Some(ElectricBowData::new(energy, fire_mode))
}

// -----------------------------------------------------------------------------
// Host API Hooks / Stubs (Marked with FORK-n)
// -----------------------------------------------------------------------------

/// Reads Electric Bow state from ItemStack data components.
// FORK-23
pub fn host_read_electric_bow(components_str: &str) -> ElectricBowData {
    parse_from_components(components_str).unwrap_or_default()
}

/// Writes Electric Bow state to ItemStack data components.
// FORK-23
pub fn host_write_electric_bow(data: &ElectricBowData) -> String {
    serialize_to_components(data)
}

/// Hook called when arrow is released by player.
// FORK-33
pub fn host_on_arrow_shoot(
    _player_id: &str,
    _data: &mut ElectricBowData,
    _is_creative: bool,
) -> bool {
    // Requires FORK-33 bow shoot event and arrow entity customization
    false
}

// -----------------------------------------------------------------------------
// Unit Tests (Comparing with Java 1.21.1 source of truth)
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_match_java_gear_config() {
        // GearConfig.java:146-151
        assert_eq!(MAX_ENERGY, 48_000, "Max energy 120_000 J = 48_000 FE (GearConfig.java:147)");
        assert_eq!(CHARGE_RATE, 240, "Charge rate 600 J/t = 240 FE/t (GearConfig.java:148)");
        assert_eq!(ENERGY_USAGE, 48, "Usage normal 120 J = 48 FE (GearConfig.java:149)");
        assert_eq!(ENERGY_USAGE_FIRE, 480, "Usage fire 1_200 J = 480 FE (GearConfig.java:150)");
    }

    #[test]
    fn mode_toggling_and_flame_enchantment() {
        // ItemElectricBow.java:86-93, 149-156
        let mut bow = ElectricBowData::default();
        assert!(!bow.fire_mode, "Default mode is OFF");
        assert_eq!(bow.flame_enchantment_level(), 0);
        assert_eq!(bow.energy_for_shot(), 48);

        // Toggle to fire mode
        assert!(bow.toggle_fire_mode(), "Toggles to true");
        assert!(bow.fire_mode);
        assert_eq!(bow.flame_enchantment_level(), 1, "Invasive Flame I enchantment active");
        assert_eq!(bow.energy_for_shot(), 480);

        // Toggle back to standard mode
        assert!(!bow.toggle_fire_mode(), "Toggles to false");
        assert!(!bow.fire_mode);
        assert_eq!(bow.flame_enchantment_level(), 0);
        assert_eq!(bow.energy_for_shot(), 48);
    }

    #[test]
    fn shooting_energy_deduction_and_creative_bypass() {
        // ItemElectricBow.java:53-76
        let mut bow = ElectricBowData::new(500, false);

        // Survival shoot in standard mode (cost 48 FE)
        assert!(bow.can_release(false));
        assert!(bow.shoot(false));
        assert_eq!(bow.energy, 500 - 48); // 452 FE left

        // Toggle to fire mode (cost 480 FE)
        bow.toggle_fire_mode();
        assert!(!bow.can_release(false), "Cannot release: 452 < 480 FE needed");
        assert!(!bow.shoot(false));
        assert_eq!(bow.energy, 452, "Energy not deducted on failed shot");

        // Creative mode bypass: shooting allowed with zero energy, energy not drained
        bow.energy = 0;
        assert!(bow.can_release(true), "Creative can release with 0 energy");
        assert!(bow.shoot(true), "Creative can shoot with 0 energy");
        assert_eq!(bow.energy, 0);
    }

    #[test]
    fn charging_rate_and_capacity_clamp() {
        let mut bow = ElectricBowData::default();
        // Request charge of 1000 FE -> clamped to CHARGE_RATE (240 FE/t)
        let accepted = bow.charge(1000);
        assert_eq!(accepted, 240);
        assert_eq!(bow.energy, 240);

        // Fully charge
        bow.energy = 47_900;
        let accepted2 = bow.charge(240);
        assert_eq!(accepted2, 100, "Clamped to remaining capacity 48_000 - 47_900 = 100 FE");
        assert_eq!(bow.energy, 48_000);
    }

    #[test]
    fn serialization_roundtrip() {
        let bow = ElectricBowData::new(32_000, true);
        let components = serialize_to_components(&bow);
        assert!(components.contains("\"energy_containers\":[32000]"));
        assert!(components.contains("\"mekanism:electric_bow_mode\":true"));

        let restored = parse_from_components(&components).expect("Should parse");
        assert_eq!(restored.energy, 32_000);
        assert!(restored.fire_mode);
    }
}

//! Radioactive Waste Barrel (`TileEntityRadioactiveWasteBarrel`, `StackedWasteBarrel`, `BlockRadioactiveWasteBarrel`).
//!
//! A specialized barrel for safely storing and passively decaying radioactive chemicals.
//! Java references:
//! - Block: `mekanism.common.block.BlockRadioactiveWasteBarrel` (`BlockRadioactiveWasteBarrel.java`)
//! - Tile Entity: `mekanism.common.tile.TileEntityRadioactiveWasteBarrel` (`TileEntityRadioactiveWasteBarrel.java`)
//! - Chemical Tank: `mekanism.common.capabilities.chemical.StackedWasteBarrel` (`StackedWasteBarrel.java`)
//! - Item: `mekanism.common.item.block.ItemBlockRadioactiveWasteBarrel` (`ItemBlockRadioactiveWasteBarrel.java`)
//! - Type Definition: `MekanismBlockTypes.RADIOACTIVE_WASTE_BARREL` (`MekanismBlockTypes.java:758-763`)
//!
//! Key mechanics:
//! 1. Holds up to 512 000 mB (512 buckets) of radioactive chemicals (`wasteBarrelCapacity`).
//! 2. Only accepts radioactive chemicals (`chemical.isRadioactive()`); non-radioactive chemicals are rejected.
//! 3. Decays 1 mB of stored chemical every 20 ticks (1 second = 3 600 mB/hour), unless blacklisted.
//! 4. Vertical connections only: accepts input from the TOP face and outputs to the BOTTOM face.
//! 5. Vertical Chaining:
//!    - Overflow push: when inserting, any overflow beyond capacity is pushed to the barrel directly above (`pos.y + 1`).
//!    - Auto-drain: when `active == true`, drains contents to the barrel or chemical handler below (`pos.y - 1`),
//!      limiting flow to prevent back-flow loops.
//! 6. Comparator support: outputs 0 to 15 redstone signal based on fill percentage.
//! 7. Catastrophic radiation dump: when destroyed, all contained radioactive chemical is released into the world
//!    environment as a radiation source, while the dropped item does NOT retain any radioactive chemicals.

use std::collections::HashMap;

use horizon_mod_sdk::server::{player_control, world, BlockEntity, BlockEntityUpdate, BlockPos};

use crate::process::Tank;
use crate::radiation::radioactivity;
use crate::Mekanism;

/// Block and Item registry ID.
pub const BLOCK_ID: &str = "mekanism:radioactive_waste_barrel";

/// Maximum chemical capacity in millibuckets (512 buckets = 512 000 mB).
/// Java: `GeneralConfig.java:216` (`wasteBarrelCapacity = 512L * FluidType.BUCKET_VOLUME`)
pub const CAPACITY: u64 = 512_000;

/// Number of game ticks per decay cycle (20 ticks = 1 second).
/// Java: `GeneralConfig.java:217` (`wasteBarrelProcessTicks = 20`)
pub const PROCESS_TICKS: u32 = 20;

/// Amount of chemical decayed per cycle (1 mB).
/// Java: `GeneralConfig.java:218` (`wasteBarrelDecayAmount = 1L`)
pub const DECAY_AMOUNT: u64 = 1;

/// Light level emitted when active (auto-draining).
/// Java: `Attributes.ACTIVE_LIGHT` (emits level 8 when active)
pub const ACTIVE_LIGHT_LEVEL: u8 = 8;

/// Sound played when toggling active state with a configurator.
pub const SOUND_TOGGLE_ACTIVE: &str = "minecraft:ui.button.click";

/// Relative sides accepted for chemical connections: TOP and BOTTOM only.
/// Java: `TileEntityRadioactiveWasteBarrel.java:66` (`RelativeSide.TOP, RelativeSide.BOTTOM`)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelativeSide {
    Top,
    Bottom,
    Side, // North, South, East, West (all rejected)
}

impl RelativeSide {
    /// Returns true if this side accepts chemical connections.
    pub fn accepts_chemicals(self) -> bool {
        matches!(self, RelativeSide::Top | RelativeSide::Bottom)
    }
}

/// Blacklisted chemicals that do not decay in the waste barrel.
/// Java: `MekanismAPITags.Chemicals.WASTE_BARREL_DECAY_BLACKLIST`
pub fn is_decay_blacklisted(chemical: &str) -> bool {
    // By default in Mekanism 1.21.1, none of the base waste chemicals are blacklisted,
    // but the tag check is present in Java: `TileEntityRadioactiveWasteBarrel.java:77`.
    chemical == "mekanism:decay_blacklist_example"
}

/// Validates whether a chemical is radioactive and allowed inside the barrel.
/// Java: `StackedWasteBarrel.java:23-33` (`ATTRIBUTE_VALIDATOR` -> `chemical.isRadioactive()`)
pub fn is_radioactive_chemical(chemical: &str) -> bool {
    radioactivity(chemical) > 0.0
}

/// State of a Radioactive Waste Barrel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WasteBarrel {
    /// The stored chemical tank.
    pub tank: Tank,
    /// Accumulator for ticks until next 1 mB decay (0 to 20).
    pub process_ticks: u32,
    /// The game time of the last processed tick (anti-tick accelerator protection).
    pub last_process_tick: u64,
    /// Whether auto-draining downward is active.
    pub active: bool,
}

impl Default for WasteBarrel {
    fn default() -> Self {
        Self {
            tank: Tank {
                chemical: String::new(),
                amount: 0,
            },
            process_ticks: 0,
            last_process_tick: 0,
            active: false,
        }
    }
}

impl WasteBarrel {
    /// Creates a new empty barrel.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a barrel with initial chemical and amount.
    pub fn with_contents(chemical: &str, amount: u64) -> Self {
        let mut b = Self::default();
        if is_radioactive_chemical(chemical) && amount > 0 {
            b.tank.chemical = chemical.to_owned();
            b.tank.amount = amount.min(CAPACITY);
        }
        b
    }

    /// Amount of space remaining in the tank.
    pub fn needed(&self) -> u64 {
        CAPACITY.saturating_sub(self.tank.amount)
    }

    /// Whether the tank is currently full.
    pub fn is_full(&self) -> bool {
        self.tank.amount >= CAPACITY
    }

    /// Percentage filled as a float from 0.0 to 1.0.
    pub fn filled_percentage(&self) -> f64 {
        if CAPACITY == 0 {
            0.0
        } else {
            (self.tank.amount as f64 / CAPACITY as f64).clamp(0.0, 1.0)
        }
    }

    /// Redstone comparator output level (0 to 15).
    /// Java: `TileEntityRadioactiveWasteBarrel.java:164` -> `MekanismUtils.redstoneLevelFromContents`
    pub fn redstone_level(&self) -> u8 {
        if self.tank.amount == 0 {
            0
        } else {
            let scaled = (self.tank.amount as f64 / CAPACITY as f64) * 15.0;
            scaled.floor() as u8
        }
    }

    /// Inserts radioactive chemical into this barrel.
    /// Returns the leftover amount that could not be accepted.
    /// Non-radioactive chemicals are rejected completely.
    /// Java: `StackedWasteBarrel.java:49-61`
    pub fn insert(&mut self, chemical: &str, amount: u64) -> u64 {
        if amount == 0 {
            return 0;
        }
        if !is_radioactive_chemical(chemical) {
            return amount;
        }
        if self.tank.amount == 0 {
            self.tank.chemical = chemical.to_owned();
        } else if self.tank.chemical != chemical {
            // Chemical mismatch
            return amount;
        }
        let room = self.needed();
        let to_add = amount.min(room);
        self.tank.amount += to_add;
        amount - to_add
    }

    /// Simulates chemical insertion, returning how much would remain.
    pub fn simulate_insert(&self, chemical: &str, amount: u64) -> u64 {
        if amount == 0 {
            return 0;
        }
        if !is_radioactive_chemical(chemical) {
            return amount;
        }
        if self.tank.amount != 0 && self.tank.chemical != chemical {
            return amount;
        }
        let room = self.needed();
        let to_add = amount.min(room);
        amount - to_add
    }

    /// Extracts chemical from the barrel. Returns the amount actually extracted.
    pub fn extract(&mut self, chemical: &str, amount: u64) -> u64 {
        if amount == 0 || self.tank.amount == 0 || self.tank.chemical != chemical {
            return 0;
        }
        let to_remove = amount.min(self.tank.amount);
        self.tank.amount -= to_remove;
        if self.tank.amount == 0 {
            self.tank.chemical.clear();
        }
        to_remove
    }

    /// Single tick update on the server.
    /// Java: `TileEntityRadioactiveWasteBarrel.java:71-105`
    ///
    /// Performs:
    /// 1. Anti-tick acceleration guard.
    /// 2. Natural decay: 1 mB every 20 ticks if not empty and not blacklisted.
    pub fn tick(&mut self, current_game_time: u64) {
        if current_game_time <= self.last_process_tick {
            return; // Already processed this tick
        }
        self.last_process_tick = current_game_time;

        // Passive decay
        if DECAY_AMOUNT > 0 && self.tank.amount > 0 && !is_decay_blacklisted(&self.tank.chemical) {
            self.process_ticks += 1;
            if self.process_ticks >= PROCESS_TICKS {
                self.process_ticks = 0;
                self.tank.amount = self.tank.amount.saturating_sub(DECAY_AMOUNT);
                if self.tank.amount == 0 {
                    self.tank.chemical.clear();
                }
            }
        }
    }
}

/// Binary serialization for block entity persistence.
/// Encodes: format version, active flag, process_ticks, chemical length, chemical string, stored amount.
pub fn encode(b: &WasteBarrel) -> Vec<u8> {
    let mut out = vec![1, u8::from(b.active)];
    out.extend(b.process_ticks.to_le_bytes());
    out.extend(b.last_process_tick.to_le_bytes());
    let chem_bytes = b.tank.chemical.as_bytes();
    out.push(chem_bytes.len().min(255) as u8);
    out.extend_from_slice(&chem_bytes[..chem_bytes.len().min(255)]);
    out.extend(b.tank.amount.to_le_bytes());
    out
}

/// Decodes persisted block entity data.
pub fn decode(data: &[u8]) -> Option<WasteBarrel> {
    if data.len() < 15 || data[0] != 1 {
        return None;
    }
    let active = data[1] == 1;
    let mut pt = [0u8; 4];
    pt.copy_from_slice(data.get(2..6)?);
    let process_ticks = u32::from_le_bytes(pt);

    let mut lpt = [0u8; 8];
    lpt.copy_from_slice(data.get(6..14)?);
    let last_process_tick = u64::from_le_bytes(lpt);

    let chem_len = usize::from(*data.get(14)?);
    if data.len() < 15 + chem_len + 8 {
        return None;
    }
    let chemical = std::str::from_utf8(data.get(15..15 + chem_len)?).ok()?.to_owned();

    let mut amt = [0u8; 8];
    amt.copy_from_slice(data.get(15 + chem_len..15 + chem_len + 8)?);
    let amount = u64::from_le_bytes(amt);

    Some(WasteBarrel {
        tank: Tank { chemical, amount },
        process_ticks,
        last_process_tick,
        active,
    })
}

// -------------------------------------------------------------------------------------------------
// Vertical Chaining Operations (Overflow Push and Auto-Drain)
// -------------------------------------------------------------------------------------------------

/// Result of vertical overflow insertion into a column of barrels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColumnInsertResult {
    /// How much was accepted by the target barrel.
    pub accepted_local: u64,
    /// How much was pushed to the barrel above.
    pub pushed_above: u64,
    /// How much remained unaccepted.
    pub remainder: u64,
}

/// Performs insertion with overflow push to the barrel above.
/// Java: `StackedWasteBarrel.java:49-61`
///
/// If `barrel_above` is present and matches the chemical type, excess is inserted into it.
pub fn insert_with_overflow(
    barrel: &mut WasteBarrel,
    barrel_above: Option<&mut WasteBarrel>,
    chemical: &str,
    amount: u64,
) -> ColumnInsertResult {
    let unaccepted_local = barrel.insert(chemical, amount);
    let accepted_local = amount - unaccepted_local;

    if unaccepted_local > 0 {
        if let Some(above) = barrel_above {
            let remainder = above.insert(chemical, unaccepted_local);
            let pushed_above = unaccepted_local - remainder;
            ColumnInsertResult {
                accepted_local,
                pushed_above,
                remainder,
            }
        } else {
            ColumnInsertResult {
                accepted_local,
                pushed_above: 0,
                remainder: unaccepted_local,
            }
        }
    } else {
        ColumnInsertResult {
            accepted_local,
            pushed_above: 0,
            remainder: 0,
        }
    }
}

/// Auto-drains contents downward to the barrel below when `active == true`.
/// Java: `TileEntityRadioactiveWasteBarrel.java:82-98`
///
/// Returns the amount of chemical successfully drained down.
/// Limits the amount to `min(below.needed(), barrel.tank.amount)` to avoid cyclic back-flow.
pub fn step_auto_drain(barrel: &mut WasteBarrel, barrel_below: Option<&mut WasteBarrel>) -> u64 {
    if !barrel.active || barrel.tank.amount == 0 {
        return 0;
    }
    if let Some(below) = barrel_below {
        let max_transfer = barrel.tank.amount.min(below.needed());
        if max_transfer == 0 {
            return 0;
        }
        let remainder = below.insert(&barrel.tank.chemical, max_transfer);
        let transferred = max_transfer - remainder;
        barrel.tank.amount -= transferred;
        if barrel.tank.amount == 0 {
            barrel.tank.chemical.clear();
        }
        transferred
    } else {
        // No barrel below or external handler
        0
    }
}

// -------------------------------------------------------------------------------------------------
// In the world: the block entities
// -------------------------------------------------------------------------------------------------

/// The chemicals a barrel takes (`StackedWasteBarrel`'s attribute validator): the radioactive ones, and while it holds one only that one.
fn accepted(held: &Tank) -> Vec<String> {
    if held.amount > 0 && !held.chemical.is_empty() {
        return vec![held.chemical.clone()];
    }
    ["mekanism:nuclear_waste", "mekanism:spent_nuclear_waste", "mekanism:plutonium", "mekanism:polonium"]
        .iter()
        .filter(|c| is_radioactive_chemical(c))
        .map(|c| (*c).to_owned())
        .collect()
}

impl Mekanism {
    /// A tick of the radioactive waste barrels (`TileEntityRadioactiveWasteBarrel.onUpdateServer`): the decay, the sneak click of the configurator
    /// that turns the draining on or off, and, when it drains, the barrel below taking what it can (`ChemicalUtil.emit`, limited to the room
    /// there). Draining into something that is not a barrel waits for the game to push a tank's content out of a face (FORK); so do the comparator
    /// level and the radiation dump of a broken barrel.
    pub(crate) fn run_waste_barrels(&mut self, barrels: &[&BlockEntity], tick: u64) -> Vec<BlockEntityUpdate> {
        let mut states: HashMap<(String, i32, i32, i32), WasteBarrel> = HashMap::new();
        let mut order: Vec<(String, i32, i32, i32)> = Vec::new();
        for b in barrels {
            let key = (b.dimension.clone(), b.pos.x, b.pos.y, b.pos.z);
            let mut state = decode(&b.data).unwrap_or_default();
            state.tank = crate::plus::held(b, 0);
            states.insert(key.clone(), state);
            order.push(key);
        }
        let mut out = Vec::new();
        for (key, b) in order.iter().zip(barrels) {
            let mut state = states.remove(key).unwrap_or_default();
            let mut toggled = false;
            if let Some(click) = self.take_click(b) {
                if click.sneak && click.mode.is_configurating() {
                    state.active = !state.active;
                    toggled = true;
                    let _ = player_control::play_sound(&b.dimension, SOUND_TOGGLE_ACTIVE, f64::from(b.pos.x) + 0.5, f64::from(b.pos.y) + 0.5, f64::from(b.pos.z) + 0.5, 0.3, 1.0);
                }
            }
            let before = state.tank.clone();
            state.tick(tick);
            if state.active && state.tank.amount > 0 {
                let below_key = (key.0.clone(), key.1, key.2 - 1, key.3);
                if let Some(below) = states.get_mut(&below_key) {
                    // Only as much as the barrel below can take: what goes up the chain again is not sent.
                    if below.tank.chemical.is_empty() || below.tank.chemical == state.tank.chemical {
                        step_auto_drain(&mut state, Some(below));
                    }
                }
            }
            let first = self.waste_filtered.insert((key.0.clone(), key.1, key.2, key.3));
            if first || state.tank != before {
                let _ = world::set_tank_filter(&b.dimension, b.pos.x, b.pos.y, b.pos.z, 0, &accepted(&state.tank));
            }
            let changed = toggled || state.tank != before || state.process_ticks != decode(&b.data).map_or(0, |d| d.process_ticks);
            let new_data = encode(&state);
            states.insert(key.clone(), state);
            if changed {
                out.push((key.clone(), new_data));
            }
        }
        // The barrels below may have changed too (they took the drain): every changed barrel gets its tank written.
        let mut updates = Vec::new();
        for ((dim, x, y, z), data) in out {
            let st = states.get(&(dim.clone(), x, y, z)).cloned().unwrap_or_default();
            updates.push(BlockEntityUpdate {
                dimension: dim,
                pos: BlockPos { x, y, z },
                data,
                items: None,
                energy: None,
                energy_faces: None,
                tanks: Some(vec![crate::plus::as_tank(&st.tank)]),
                tank_faces: None,
                menu_data: None,
                transmitter_faces: None,
            });
        }
        // A barrel that only received the drain (no change of its own) is written as well.
        for b in barrels {
            let key = (b.dimension.clone(), b.pos.x, b.pos.y, b.pos.z);
            if updates.iter().any(|u| (u.dimension.as_str(), u.pos.x, u.pos.y, u.pos.z) == (key.0.as_str(), key.1, key.2, key.3)) {
                continue;
            }
            if let Some(st) = states.get(&key) {
                if st.tank != crate::plus::held(b, 0) {
                    updates.push(BlockEntityUpdate {
                        dimension: key.0.clone(),
                        pos: b.pos,
                        data: encode(st),
                        items: None,
                        energy: None,
                        energy_faces: None,
                        tanks: Some(vec![crate::plus::as_tank(&st.tank)]),
                        tank_faces: None,
                        menu_data: None,
                        transmitter_faces: None,
                    });
                }
            }
        }
        updates
    }
}

// -------------------------------------------------------------------------------------------------
// Interaction Handlers (Sneak Right-Click)
// -------------------------------------------------------------------------------------------------

/// Result of player interaction with the barrel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InteractionResponse {
    /// Toggled active state with a configurator/wrench.
    ToggledActive { new_active: bool, sound: &'static str },
    /// Displayed stored contents to player chat on sneak empty-hand click.
    DisplayStatus { message: String },
    /// Normal pass (no interaction).
    Pass,
}

/// Evaluates player right-click interaction on the Radioactive Waste Barrel.
/// Java: `BlockRadioactiveWasteBarrel.java:30-48` and `TileEntityRadioactiveWasteBarrel.java:130-144`
pub fn handle_interaction(
    barrel: &mut WasteBarrel,
    is_sneaking: bool,
    held_item: Option<&str>,
) -> InteractionResponse {
    if !is_sneaking {
        return InteractionResponse::Pass;
    }
    match held_item {
        Some(item) if item == "mekanism:configurator" || item.ends_with("_wrench") => {
            // Shift + click with configurator: toggle active mode and play click sound
            barrel.active = !barrel.active;
            InteractionResponse::ToggledActive {
                new_active: barrel.active,
                sound: SOUND_TOGGLE_ACTIVE,
            }
        }
        None | Some("minecraft:air") | Some("") => {
            // Shift + click with empty hand: display chat message with stored content and percentage
            if barrel.tank.amount == 0 {
                InteractionResponse::DisplayStatus {
                    message: "No chemical".to_string(),
                }
            } else {
                let pct = (barrel.filled_percentage() * 100.0).round() as u64;
                InteractionResponse::DisplayStatus {
                    message: format!(
                        "Stored: {} ({} mB, {}%)",
                        barrel.tank.chemical, barrel.tank.amount, pct
                    ),
                }
            }
        }
        _ => InteractionResponse::Pass,
    }
}

// -------------------------------------------------------------------------------------------------
// Destruction / Radiation Dump Behavior
// -------------------------------------------------------------------------------------------------

/// Result of breaking a Radioactive Waste Barrel.
/// Java: `TileEntityMekanism.java:712-716` and `BlockRadioactiveWasteBarrel.java`
#[derive(Clone, Debug, PartialEq)]
pub struct BarrelBreakResult {
    /// Magnitude of radiation dumped into the environment (in Sv/h).
    pub radiation_dumped_sv_per_hour: f64,
    /// Item dropped in world (does NOT contain radioactive chemical!).
    pub dropped_item: &'static str,
}

/// Evaluates breaking the barrel in the world.
///
/// If radiation is enabled and the barrel contains radioactive waste, dumps radiation:
/// `dump_magnitude = stored_amount * radioactivity(chemical)` (Sv/h).
/// Dropped item is `"mekanism:radioactive_waste_barrel"` with 0 chemicals.
pub fn handle_block_broken(barrel: &WasteBarrel, radiation_enabled: bool) -> BarrelBreakResult {
    let dump_magnitude = if radiation_enabled && barrel.tank.amount > 0 {
        (barrel.tank.amount as f64) * radioactivity(&barrel.tank.chemical)
    } else {
        0.0
    };
    BarrelBreakResult {
        radiation_dumped_sv_per_hour: dump_magnitude,
        dropped_item: BLOCK_ID,
    }
}

// -------------------------------------------------------------------------------------------------
// FORK Requirements (isolated host call stubs and declarations)
// -------------------------------------------------------------------------------------------------

/// Host interface for dispersing atmospheric radiation when radioactive containers break.
///
/// WIT proposed in `FORK_robit_quimicos.md`:
/// ```wit
/// dump-radiation: func(dimension: string, x: s32, y: s32, z: s32, magnitude-sv-h: f64);
/// ```
// FORK-13
pub fn host_dump_radiation(
    dimension: &str,
    pos: [i32; 3],
    magnitude_sv_per_hour: f64,
) -> Result<(), &'static str> {
    let _ = (dimension, pos, magnitude_sv_per_hour);
    Ok(())
}

/// Host interface for querying vertical block entity capabilities and chaining.
///
/// WIT proposed in `FORK_robit_quimicos.md`:
/// ```wit
/// query-tile-above: func(dimension: string, x: s32, y: s32, z: s32) -> option<u32>;
/// query-tile-below: func(dimension: string, x: s32, y: s32, z: s32) -> option<u32>;
/// ```
// FORK-30
pub fn host_query_vertical_chain(dimension: &str, pos: [i32; 3]) -> Result<(), &'static str> {
    let _ = (dimension, pos);
    Ok(())
}

// -------------------------------------------------------------------------------------------------
// Unit Tests (verifying exact 1:1 match with Java constants and lines)
// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capacity_and_decay_constants_match_java() {
        // GeneralConfig.java:216: wasteBarrelCapacity = 512 * 1_000 = 512_000
        assert_eq!(CAPACITY, 512_000);
        // GeneralConfig.java:217: wasteBarrelProcessTicks = 20
        assert_eq!(PROCESS_TICKS, 20);
        // GeneralConfig.java:218: wasteBarrelDecayAmount = 1L
        assert_eq!(DECAY_AMOUNT, 1);
        // Active light level is 8
        assert_eq!(ACTIVE_LIGHT_LEVEL, 8);
    }

    #[test]
    fn accepts_only_radioactive_chemicals() {
        // StackedWasteBarrel.java:23-33: only radioactive chemicals pass attribute validator
        assert!(is_radioactive_chemical("mekanism:nuclear_waste"));
        assert!(is_radioactive_chemical("mekanism:spent_nuclear_waste"));
        assert!(is_radioactive_chemical("mekanism:plutonium"));
        assert!(is_radioactive_chemical("mekanism:polonium"));

        // Non-radioactive chemicals are refused
        assert!(!is_radioactive_chemical("mekanism:oxygen"));
        assert!(!is_radioactive_chemical("mekanism:hydrogen"));
        assert!(!is_radioactive_chemical("mekanism:steam"));
        assert!(!is_radioactive_chemical("mekanism:brine"));

        let mut barrel = WasteBarrel::new();
        let refused = barrel.insert("mekanism:oxygen", 100);
        assert_eq!(refused, 100);
        assert_eq!(barrel.tank.amount, 0);

        let accepted = barrel.insert("mekanism:nuclear_waste", 500);
        assert_eq!(accepted, 0);
        assert_eq!(barrel.tank.amount, 500);
        assert_eq!(barrel.tank.chemical, "mekanism:nuclear_waste");
    }

    #[test]
    fn side_connections_accept_only_top_and_bottom() {
        // TileEntityRadioactiveWasteBarrel.java:66
        assert!(RelativeSide::Top.accepts_chemicals());
        assert!(RelativeSide::Bottom.accepts_chemicals());
        assert!(!RelativeSide::Side.accepts_chemicals());
    }

    #[test]
    fn passive_decay_every_twenty_ticks() {
        // TileEntityRadioactiveWasteBarrel.java:78-81
        let mut barrel = WasteBarrel::with_contents("mekanism:nuclear_waste", 100);

        // Run 19 ticks: no decay yet
        for t in 1..=19 {
            barrel.tick(t);
            assert_eq!(barrel.tank.amount, 100);
        }

        // 20th tick: decays by 1 mB
        barrel.tick(20);
        assert_eq!(barrel.tank.amount, 99);
        assert_eq!(barrel.process_ticks, 0);

        // Anti-tick acceleration: ticking same game time again does nothing
        barrel.tick(20);
        assert_eq!(barrel.tank.amount, 99);
    }

    #[test]
    fn redstone_comparator_level_scaling() {
        // TileEntityRadioactiveWasteBarrel.java:164: MekanismUtils.redstoneLevelFromContents
        let mut barrel = WasteBarrel::new();
        assert_eq!(barrel.redstone_level(), 0);

        // Half full = level 7
        barrel.tank.amount = CAPACITY / 2; // 256_000
        assert_eq!(barrel.redstone_level(), 7);

        // Completely full = level 15
        barrel.tank.amount = CAPACITY; // 512_000
        assert_eq!(barrel.redstone_level(), 15);
    }

    #[test]
    fn vertical_chaining_overflow_push_upward() {
        // StackedWasteBarrel.java:49-61: excess pushes to barrel above
        let mut lower = WasteBarrel::with_contents("mekanism:nuclear_waste", 511_990);
        let mut upper = WasteBarrel::new();

        // Inserting 30 mB: 10 fits in lower, 20 overflows to upper
        let res = insert_with_overflow(&mut lower, Some(&mut upper), "mekanism:nuclear_waste", 30);
        assert_eq!(res.accepted_local, 10);
        assert_eq!(res.pushed_above, 20);
        assert_eq!(res.remainder, 0);
        assert_eq!(lower.tank.amount, CAPACITY);
        assert_eq!(upper.tank.amount, 20);
    }

    #[test]
    fn auto_drain_downward_when_active() {
        // TileEntityRadioactiveWasteBarrel.java:82-98
        let mut upper = WasteBarrel::with_contents("mekanism:nuclear_waste", 10_000);
        let mut lower = WasteBarrel::new();

        // Inactive: does not drain
        upper.active = false;
        assert_eq!(step_auto_drain(&mut upper, Some(&mut lower)), 0);
        assert_eq!(upper.tank.amount, 10_000);

        // Active: drains into lower barrel
        upper.active = true;
        let drained = step_auto_drain(&mut upper, Some(&mut lower));
        assert_eq!(drained, 10_000);
        assert_eq!(upper.tank.amount, 0);
        assert_eq!(lower.tank.amount, 10_000);
    }

    #[test]
    fn sneak_interactions_configurator_and_empty_hand() {
        let mut barrel = WasteBarrel::with_contents("mekanism:nuclear_waste", 256_000);

        // Sneak click empty hand displays status and percentage (50%)
        let status = handle_interaction(&mut barrel, true, None);
        match status {
            InteractionResponse::DisplayStatus { message } => {
                assert_eq!(message, "Stored: mekanism:nuclear_waste (256000 mB, 50%)");
            }
            _ => panic!("Expected DisplayStatus"),
        }

        // Sneak click with configurator toggles active state
        assert!(!barrel.active);
        let toggle = handle_interaction(&mut barrel, true, Some("mekanism:configurator"));
        match toggle {
            InteractionResponse::ToggledActive { new_active, sound } => {
                assert!(new_active);
                assert_eq!(sound, "minecraft:ui.button.click");
            }
            _ => panic!("Expected ToggledActive"),
        }
        assert!(barrel.active);
    }

    #[test]
    fn breaking_barrel_dumps_radiation_catastrophically() {
        // TileEntityMekanism.java:715: dumpRadiation
        // 512_000 mB of nuclear_waste (0.01 Sv/h/mB) -> 5,120 Sv/h!
        let full_barrel = WasteBarrel::with_contents("mekanism:nuclear_waste", 512_000);
        let broken = handle_block_broken(&full_barrel, true);

        assert_eq!(broken.radiation_dumped_sv_per_hour, 5_120.0);
        assert_eq!(broken.dropped_item, "mekanism:radioactive_waste_barrel");

        // When global radiation is disabled, 0 radiation dumped
        let safe_break = handle_block_broken(&full_barrel, false);
        assert_eq!(safe_break.radiation_dumped_sv_per_hour, 0.0);
    }

    #[test]
    fn serialization_roundtrip() {
        let original = WasteBarrel {
            tank: Tank {
                chemical: "mekanism:polonium".to_string(),
                amount: 45_678,
            },
            process_ticks: 12,
            last_process_tick: 98_765,
            active: true,
        };

        let encoded = encode(&original);
        let decoded = decode(&encoded).expect("decode should succeed");
        assert_eq!(decoded, original);
    }

    #[test]
    fn a_barrel_takes_the_radioactive_chemicals_and_then_only_the_one_it_holds() {
        let empty = accepted(&Tank::default());
        assert!(empty.contains(&"mekanism:plutonium".to_owned()) && empty.contains(&"mekanism:nuclear_waste".to_owned()));
        assert!(!empty.contains(&"mekanism:hydrogen".to_owned()));
        let holding = accepted(&Tank { chemical: "mekanism:polonium".into(), amount: 5 });
        assert_eq!(holding, vec!["mekanism:polonium".to_owned()]);
    }
}

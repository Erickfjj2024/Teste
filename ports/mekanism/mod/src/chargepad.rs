//! Chargepad (`TileEntityChargepad`, `BlockChargepad`).
//!
//! A pressure-plate style block that rapidly charges energy-storing items held in a standing player's inventory
//! or Robits standing on it.
//!
//! Java references:
//! - `BlockChargepad.java:19`: `BASE = box(0, 0, 0, 16, 1, 16)`
//! - `TileEntityChargepad.java:36`: `CHARGE_PREDICATE = !isSpectator && (Player || Robit)`
//! - `TileEntityChargepad.java:48`: `RelativeSide.BACK, RelativeSide.BOTTOM` energy inputs
//! - `TileEntityChargepad.java:58-59`: `AABB(x, y, z, x + 1, y + 0.4, z + 1)`
//! - `TileEntityChargepad.java:92-95`: charges one item per player per tick
//! - `TileEntityChargepad.java:105-117`: `energyToGive = energyContainer.getEnergyPerTick()`
//! - `TileEntityChargepad.java:128-133`: `SoundEvents.STONE_PRESSURE_PLATE_CLICK_ON` (pitch 0.8), `STONE_PRESSURE_PLATE_CLICK_OFF` (pitch 0.7)
//! - `StorageConfig.java:39`: `chargepad = 2_048_000L` Joules = 819_200 FE
//! - `UsageConfig.java:42`: `chargePad = 1_024_000L` Joules/t = 409_600 FE/t

use super::*;

pub const BLOCK: &str = "mekanism:chargepad";

/// Buffer capacity: 2,048,000 J = 819,200 FE (Java: `StorageConfig.chargepad`, `fe(2_048_000)`).
pub const CAPACITY: u64 = fe(2_048_000);

/// Maximum charge rate per tick: 1,024,000 J/t = 409_600 FE/t (Java: `UsageConfig.chargePad`, `fe(1_024_000)`).
pub const CHARGE_RATE: u64 = fe(1_024_000);

/// Sounds played on state transitions (Java: `TileEntityChargepad.java:128-133`).
pub const SOUND_CLICK_ON: &str = "minecraft:block.stone_pressure_plate.click_on";
pub const SOUND_CLICK_OFF: &str = "minecraft:block.stone_pressure_plate.click_off";
pub const PITCH_ON: f32 = 0.8;
pub const PITCH_OFF: f32 = 0.7;
pub const VOLUME: f32 = 0.3;

/// Returns the input energy sides relative to the facing direction (Java: `TileEntityChargepad.java:48`).
/// Only RelativeSide.BACK and RelativeSide.BOTTOM accept energy.
pub fn energy_input_faces(facing: &str) -> Vec<FlowSide> {
    let front = laser::face_of(facing);
    let back = laser::opposite(front);
    let mut faces = vec![FlowSide::None; 6];
    // Bottom is always DOWN (0)
    faces[laser::DOWN] = FlowSide::Input;
    // Back face
    faces[back] = FlowSide::Input;
    faces
}

/// Bounding box offset for detecting entities standing on the chargepad (Java: `TileEntityChargepad.java:58-59`).
/// `[x_min, y_min, z_min, x_max, y_max, z_max]` relative to block position.
pub const AABB_BOUNDS: [f64; 6] = [0.0, 0.0, 0.0, 1.0, 0.4, 1.0];

/// Represents an entity standing on the chargepad.
#[derive(Clone, Debug)]
pub enum ChargeTarget {
    Player {
        id: String,
        is_spectator: bool,
        rechargeable_slots: Vec<RechargeableSlot>,
    },
    Robit {
        id: String,
        energy_stored: u64,
        max_energy: u64,
    },
}

#[derive(Clone, Debug)]
pub struct RechargeableSlot {
    pub slot: usize,
    pub energy_stored: u64,
    pub max_energy: u64,
    /// The most the item takes in one insert (the rate limit of its energy container, e.g. `mekaSuitBaseChargeRate`); `u64::MAX` when it has none.
    pub max_insert: u64,
}

/// Simulated charging result for one tick.
#[derive(Clone, Debug, Default)]
pub struct ChargeResult {
    pub energy_extracted: u64,
    pub active: bool,
    pub changed_state: bool,
    pub sound_to_play: Option<(&'static str, f32)>,
}

/// Pure state of a Chargepad.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub energy: u64,
    pub active: bool,
    pub facing: String,
}

impl Default for State {
    fn default() -> Self {
        State {
            energy: 0,
            active: false,
            facing: "north".to_string(),
        }
    }
}

pub fn decode(data: &[u8]) -> State {
    if data.len() < 9 {
        return State::default();
    }
    let energy = u64::from_le_bytes([
        data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
    ]);
    let active = data[8] == 1;
    State {
        energy: energy.min(CAPACITY),
        active,
        facing: "north".to_string(),
    }
}

pub fn encode(state: &State) -> Vec<u8> {
    let mut out = state.energy.to_le_bytes().to_vec();
    out.push(u8::from(state.active));
    out
}

/// Pure charging tick step (Java: `TileEntityChargepad.java:53-84`).
pub fn step_charging(
    current_energy: &mut u64,
    was_active: bool,
    targets: &mut [ChargeTarget],
) -> ChargeResult {
    let mut active = false;
    let mut total_extracted = 0u64;

    if *current_energy > 0 {
        for target in targets.iter_mut() {
            if *current_energy == 0 {
                break;
            }
            match target {
                ChargeTarget::Player { is_spectator, rechargeable_slots, .. } => {
                    if *is_spectator {
                        continue;
                    }
                    // Charges only one item per player each tick (Java: TileEntityChargepad.java:92-95)
                    for slot in rechargeable_slots.iter_mut() {
                        let needed = slot.max_energy.saturating_sub(slot.energy_stored);
                        if needed > 0 {
                            // `provideEnergy`: the pad offers its rate, the item accepts what it has room for within its own insert limit, the pad pays that.
                            let to_give = CHARGE_RATE.min(*current_energy).min(needed).min(slot.max_insert);
                            if to_give > 0 {
                                slot.energy_stored += to_give;
                                *current_energy -= to_give;
                                total_extracted += to_give;
                                active = true;
                                break;
                            }
                        }
                    }
                }
                ChargeTarget::Robit { energy_stored, max_energy, .. } => {
                    let needed = max_energy.saturating_sub(*energy_stored);
                    if needed > 0 {
                        let to_give = CHARGE_RATE.min(*current_energy).min(needed);
                        if to_give > 0 {
                            *energy_stored += to_give;
                            *current_energy -= to_give;
                            total_extracted += to_give;
                            active = true;
                        }
                    }
                }
            }
        }
    }

    let sound = if was_active != active {
        if active {
            Some((SOUND_CLICK_ON, PITCH_ON))
        } else {
            Some((SOUND_CLICK_OFF, PITCH_OFF))
        }
    } else {
        None
    };

    ChargeResult {
        energy_extracted: total_extracted,
        active,
        changed_state: was_active != active,
        sound_to_play: sound,
    }
}

/// Host API bridge for detecting entities in AABB and charging player/robit energy.
// FORK-30: Host AABB entity query and player item inventory charging API
pub fn scan_and_charge_entities(
    dimension: &str,
    pos: BlockPos,
    current_energy: &mut u64,
    was_active: bool,
) -> ChargeResult {
    let _ = (dimension, pos);
    // In WASM without host capability, this acts as a stub that delegates to pure logic
    let mut dummy_targets = Vec::new();
    step_charging(current_energy, was_active, &mut dummy_targets)
}

impl Mekanism {
    /// Server tick handler for all Chargepad block entities.
    pub(crate) fn run_chargepads(&mut self, chargepads: &[&BlockEntity]) -> Vec<BlockEntityUpdate> {
        let mut out = Vec::new();
        for e in chargepads {
            let block = e.state.split('[').next().unwrap_or("");
            if block != BLOCK {
                continue;
            }

            let facing = sides::property(&e.state, "facing").unwrap_or("north");
            let mut state = decode(&e.data);
            state.facing = facing.to_string();

            // Energy stored in the block entity
            let mut energy = e.energy;

            // Execute charging step
            let result = scan_and_charge_entities(&e.dimension, e.pos, &mut energy, state.active);
            state.energy = energy;

            let mut state_changed = false;
            if result.active != state.active {
                state.active = result.active;
                state_changed = true;
                let _ = world::set_block_property(&e.dimension, e.pos.x, e.pos.y, e.pos.z, "active", if state.active { "true" } else { "false" });
            }

            if state_changed || energy != e.energy {
                out.push(BlockEntityUpdate {
                    dimension: e.dimension.clone(),
                    pos: e.pos,
                    data: encode(&state),
                    items: None,
                    energy: Some(energy),
                    energy_faces: Some(energy_input_faces(facing)),
                    tanks: None,
                    tank_faces: None,
                    menu_data: None,
                    transmitter_faces: None,
                });
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chargepad_capacities_and_rates() {
        // Java: StorageConfig.java:39 chargepad = 2_048_000 J
        // 2_048_000 * 2 / 5 = 819_200 FE
        assert_eq!(CAPACITY, 819_200);

        // Java: UsageConfig.java:42 chargePad = 1_024_000 J/t
        // 1_024_000 * 2 / 5 = 409_600 FE/t
        assert_eq!(CHARGE_RATE, 409_600);
    }

    #[test]
    fn test_energy_input_faces() {
        // Java: TileEntityChargepad.java:48
        // RelativeSide.BACK and RelativeSide.BOTTOM
        let faces_north = energy_input_faces("north");
        assert_eq!(faces_north[laser::DOWN], FlowSide::Input); // Bottom
        assert_eq!(faces_north[laser::SOUTH], FlowSide::Input); // Back of North
        assert_eq!(faces_north[laser::NORTH], FlowSide::None);
        assert_eq!(faces_north[laser::UP], FlowSide::None);
        assert_eq!(faces_north[laser::WEST], FlowSide::None);
        assert_eq!(faces_north[laser::EAST], FlowSide::None);

        let faces_east = energy_input_faces("east");
        assert_eq!(faces_east[laser::DOWN], FlowSide::Input);
        assert_eq!(faces_east[laser::WEST], FlowSide::Input); // Back of East
    }

    #[test]
    fn test_charge_single_item_per_player_tick() {
        // Java: TileEntityChargepad.java:92-95
        let mut buffer = CAPACITY;
        let mut targets = vec![ChargeTarget::Player {
            id: "Steve".into(),
            is_spectator: false,
            rechargeable_slots: vec![
                RechargeableSlot { slot: 0, energy_stored: 0, max_energy: 1_000_000, max_insert: u64::MAX },
                RechargeableSlot { slot: 1, energy_stored: 0, max_energy: 1_000_000, max_insert: u64::MAX },
            ],
        }];

        let result = step_charging(&mut buffer, false, &mut targets);
        assert!(result.active);
        assert_eq!(result.energy_extracted, CHARGE_RATE); // 409_600 FE
        assert_eq!(buffer, CAPACITY - CHARGE_RATE);

        // Verify ONLY the first item received charge
        if let ChargeTarget::Player { rechargeable_slots, .. } = &targets[0] {
            assert_eq!(rechargeable_slots[0].energy_stored, CHARGE_RATE);
            assert_eq!(rechargeable_slots[1].energy_stored, 0); // Second slot untouched this tick!
        }
    }

    #[test]
    fn test_charge_robit() {
        // Java: TileEntityChargepad.java:74
        let mut buffer = 500_000;
        let mut targets = vec![ChargeTarget::Robit {
            id: "Robit_1".into(),
            energy_stored: 10_000,
            max_energy: 100_000,
        }];

        let result = step_charging(&mut buffer, false, &mut targets);
        assert!(result.active);
        // Needed: 90_000, buffer has 500_000, rate is 409_600 -> min(409600, 500000, 90000) = 90_000
        assert_eq!(result.energy_extracted, 90_000);
        assert_eq!(buffer, 410_000);
    }

    #[test]
    fn java_chargepad_gives_one_item_per_player_within_the_item_insert_limit() {
        // `TileEntityChargepad.provideEnergy`/`chargeHandler`: the pad offers its rate; the item takes what fits within its own insert limit; only the first
        // item that takes energy is charged per player each tick; a spectator is skipped; the pad stops when it is empty.
        let slot = |i, stored, max, insert| RechargeableSlot { slot: i, energy_stored: stored, max_energy: max, max_insert: insert };
        let mut buffer = 1_000_000;
        let mut targets = vec![
            ChargeTarget::Player {
                id: "steve".into(),
                is_spectator: false,
                rechargeable_slots: vec![slot(0, 1_000, 1_000, u64::MAX) /* full: skipped */, slot(1, 0, 500_000, 40_000), slot(2, 0, 500_000, u64::MAX)],
            },
            ChargeTarget::Player { id: "ghost".into(), is_spectator: true, rechargeable_slots: vec![slot(0, 0, 500_000, u64::MAX)] },
        ];
        let result = step_charging(&mut buffer, false, &mut targets);
        assert!(result.active);
        assert_eq!(result.energy_extracted, 40_000, "the second item takes 40 000 FE (its limit); the third item waits for the next tick");
        assert_eq!(buffer, 960_000);
        match &targets[0] {
            ChargeTarget::Player { rechargeable_slots, .. } => {
                assert_eq!(rechargeable_slots[1].energy_stored, 40_000);
                assert_eq!(rechargeable_slots[2].energy_stored, 0);
            }
            _ => unreachable!(),
        }
        match &targets[1] {
            ChargeTarget::Player { rechargeable_slots, .. } => assert_eq!(rechargeable_slots[0].energy_stored, 0, "spectators are not charged"),
            _ => unreachable!(),
        }
        // A pad with less than the rate gives what it has.
        let mut low = 1_000;
        let mut robit = vec![ChargeTarget::Robit { id: "r".into(), energy_stored: 0, max_energy: 40_000 }];
        assert_eq!(step_charging(&mut low, false, &mut robit).energy_extracted, 1_000);
        assert_eq!(low, 0);
        // An empty pad does nothing.
        let mut empty = 0;
        assert!(!step_charging(&mut empty, true, &mut robit).active);
    }

    #[test]
    fn test_pressure_plate_sound_triggers() {
        // Java: TileEntityChargepad.java:128-133
        let mut buffer = CAPACITY;
        let mut targets = vec![ChargeTarget::Robit {
            id: "Robit_1".into(),
            energy_stored: 0,
            max_energy: 1_000_000,
        }];

        // false -> true: click_on (pitch 0.8)
        let res_on = step_charging(&mut buffer, false, &mut targets);
        assert_eq!(res_on.sound_to_play, Some((SOUND_CLICK_ON, PITCH_ON)));

        // true -> true: no sound
        let res_stay = step_charging(&mut buffer, true, &mut targets);
        assert_eq!(res_stay.sound_to_play, None);

        // true -> false: click_off (pitch 0.7)
        let mut empty_targets = vec![];
        let res_off = step_charging(&mut buffer, true, &mut empty_targets);
        assert_eq!(res_off.sound_to_play, Some((SOUND_CLICK_OFF, PITCH_OFF)));
    }

    #[test]
    fn test_chargepad_serialization_roundtrip() {
        let state = State {
            energy: 750_000,
            active: true,
            facing: "east".into(),
        };
        let encoded = encode(&state);
        let decoded = decode(&encoded);
        assert_eq!(decoded.energy, 750_000);
        assert!(decoded.active);
    }
}

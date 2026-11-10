//! Host integration and fork requirements for the equipamento group.
//!
//! Covers:
//! - Meka-Tool (`meka_tool.rs`): FORK-23, FORK-27, FORK-31, FORK-32
//! - Electric Bow (`electric_bow.rs`): FORK-23, FORK-33
//! - HDPE Elytra (`hdpe_elytra.rs`): FORK-23, FORK-34
//! - Modules & MekaSuit (`modules.rs`, `mekasuit.rs`): FORK-11, FORK-23, FORK-27, FORK-29, FORK-30
//! - Robit (`robit.rs`): FORK-10, FORK-24
//! - Chemical Fluids (`chemical_fluids.rs`): FORK-26, FORK-29
//! - Radioactive Waste Barrel (`radioactive_waste_barrel.rs`): FORK-13, FORK-30

#[cfg(feature = "fork")]
use horizon_mod_sdk::server::*;

// -----------------------------------------------------------------------------
// FORK-27: Radial menu for modular equipment (Meka-Tool, MekaSuit)
// -----------------------------------------------------------------------------
#[cfg(feature = "fork")]
pub fn open_radial_menu(_player_id: &str, _item_id: &str) -> Result<bool, String> {
    Ok(false)
}
#[cfg(not(feature = "fork"))]
pub fn open_radial_menu(_player_id: &str, _item_id: &str) -> Result<bool, String> {
    Err("FORK-27 pendente".into())
}

// -----------------------------------------------------------------------------
// FORK-31: Multi-block area mining (Blasting and Vein Mining units)
// -----------------------------------------------------------------------------
#[cfg(feature = "fork")]
pub fn execute_area_mining(
    _dimension: &str,
    _player_id: &str,
    _positions: &[[i32; 3]],
    _total_energy: u64,
) -> Result<usize, String> {
    Ok(0)
}
#[cfg(not(feature = "fork"))]
pub fn execute_area_mining(
    _dimension: &str,
    _player_id: &str,
    _positions: &[[i32; 3]],
    _total_energy: u64,
) -> Result<usize, String> {
    Err("FORK-31 pendente".into())
}

// -----------------------------------------------------------------------------
// FORK-32: Raycast teleportation with portal FX
// -----------------------------------------------------------------------------
#[cfg(feature = "fork")]
pub fn execute_teleport(
    _dimension: &str,
    _player_id: &str,
    _target: [f64; 3],
    _energy_cost: u64,
) -> Result<bool, String> {
    Ok(false)
}
#[cfg(not(feature = "fork"))]
pub fn execute_teleport(
    _dimension: &str,
    _player_id: &str,
    _target: [f64; 3],
    _energy_cost: u64,
) -> Result<bool, String> {
    Err("FORK-32 pendente".into())
}

// -----------------------------------------------------------------------------
// FORK-33: Bow shooting hooks and projectile customization
// -----------------------------------------------------------------------------
#[cfg(feature = "fork")]
pub fn on_arrow_shoot(_player_id: &str, _is_creative: bool) -> Result<bool, String> {
    Ok(false)
}
#[cfg(not(feature = "fork"))]
pub fn on_arrow_shoot(_player_id: &str, _is_creative: bool) -> Result<bool, String> {
    Err("FORK-33 pendente".into())
}

// -----------------------------------------------------------------------------
// FORK-34: Elytra flight tick physics and wear hook
// -----------------------------------------------------------------------------
#[cfg(feature = "fork")]
pub fn tick_elytra_flight(_player_id: &str, _flight_ticks: u32) -> Result<bool, String> {
    Ok(false)
}
#[cfg(not(feature = "fork"))]
pub fn tick_elytra_flight(_player_id: &str, _flight_ticks: u32) -> Result<bool, String> {
    Err("FORK-34 pendente".into())
}

// -----------------------------------------------------------------------------
// FORK-23: Data components read/write on player equipment
// -----------------------------------------------------------------------------
#[cfg(feature = "fork")]
pub fn read_stack_components(_player: &str, _slot: u32) -> Result<Option<String>, String> {
    Ok(None)
}
#[cfg(not(feature = "fork"))]
pub fn read_stack_components(_player: &str, _slot: u32) -> Result<Option<String>, String> {
    Err("FORK-23 pendente".into())
}

#[cfg(feature = "fork")]
pub fn write_stack_components(_player: &str, _slot: u32, _components: &str) -> Result<bool, String> {
    Ok(true)
}
#[cfg(not(feature = "fork"))]
pub fn write_stack_components(_player: &str, _slot: u32, _components: &str) -> Result<bool, String> {
    Err("FORK-23 pendente".into())
}

// -----------------------------------------------------------------------------
// FORK-29: 3D vector movement impulse and flight ability
// -----------------------------------------------------------------------------
#[cfg(feature = "fork")]
pub fn apply_vector_impulse(_player: &str, _vx: f64, _vy: f64, _vz: f64) -> Result<(), String> {
    Ok(())
}
#[cfg(not(feature = "fork"))]
pub fn apply_vector_impulse(_player: &str, _vx: f64, _vy: f64, _vz: f64) -> Result<(), String> {
    Err("FORK-29 pendente".into())
}

#[cfg(feature = "fork")]
pub fn set_creative_flight(_player: &str, _enabled: bool) -> Result<(), String> {
    Ok(())
}
#[cfg(not(feature = "fork"))]
pub fn set_creative_flight(_player: &str, _enabled: bool) -> Result<(), String> {
    Err("FORK-29 pendente".into())
}

// -----------------------------------------------------------------------------
// FORK-30: Dynamic HUD element rendering & tile capability query
// -----------------------------------------------------------------------------
#[cfg(feature = "fork")]
pub fn update_hud(
    _player: &str,
    _hud_id: &str,
    _icon: &str,
    _text: &str,
    _percentage: f32,
) -> Result<(), String> {
    Ok(())
}
#[cfg(not(feature = "fork"))]
pub fn update_hud(
    _player: &str,
    _hud_id: &str,
    _icon: &str,
    _text: &str,
    _percentage: f32,
) -> Result<(), String> {
    Err("FORK-30 pendente".into())
}

#[cfg(feature = "fork")]
pub fn query_vertical_chain(_dimension: &str, _pos: [i32; 3]) -> Result<(), String> {
    Ok(())
}
#[cfg(not(feature = "fork"))]
pub fn query_vertical_chain(_dimension: &str, _pos: [i32; 3]) -> Result<(), String> {
    Err("FORK-30 pendente".into())
}

// -----------------------------------------------------------------------------
// FORK-10: Spawning Robit entity in world
// -----------------------------------------------------------------------------
#[cfg(feature = "fork")]
pub fn spawn_robit_entity(
    _dimension: &str,
    _pos: [f64; 3],
    _owner_uuid: &str,
    _skin: &str,
) -> Result<u32, String> {
    Ok(1)
}
#[cfg(not(feature = "fork"))]
pub fn spawn_robit_entity(
    _dimension: &str,
    _pos: [f64; 3],
    _owner_uuid: &str,
    _skin: &str,
) -> Result<u32, String> {
    Err("FORK-10 pendente".into())
}

// -----------------------------------------------------------------------------
// FORK-24: Entity C2S/S2C button packets
// -----------------------------------------------------------------------------
#[cfg(feature = "fork")]
pub fn send_entity_button(
    _entity_id: u32,
    _button_id: u32,
    _extra_data: &[u8],
) -> Result<(), String> {
    Ok(())
}
#[cfg(not(feature = "fork"))]
pub fn send_entity_button(
    _entity_id: u32,
    _button_id: u32,
    _extra_data: &[u8],
) -> Result<(), String> {
    Err("FORK-24 pendente".into())
}

// -----------------------------------------------------------------------------
// FORK-26: Registering custom chemical fluids
// -----------------------------------------------------------------------------
#[cfg(feature = "fork")]
pub fn register_chemical_fluid(_fluid_name: &str) -> Result<(), String> {
    Ok(())
}
#[cfg(not(feature = "fork"))]
pub fn register_chemical_fluid(_fluid_name: &str) -> Result<(), String> {
    Err("FORK-26 pendente".into())
}

// -----------------------------------------------------------------------------
// FORK-29: Registering consumable bucket items
// -----------------------------------------------------------------------------
#[cfg(feature = "fork")]
pub fn register_consumable_bucket(
    _item_id: &str,
    _use_duration: u32,
    _mb_per_food: u32,
    _saturation: f32,
    _returned_empty_item: &str,
) -> Result<(), String> {
    Ok(())
}
#[cfg(not(feature = "fork"))]
pub fn register_consumable_bucket(
    _item_id: &str,
    _use_duration: u32,
    _mb_per_food: u32,
    _saturation: f32,
    _returned_empty_item: &str,
) -> Result<(), String> {
    Err("FORK-29 pendente".into())
}

// -----------------------------------------------------------------------------
// FORK-13: Dumping environmental radiation
// -----------------------------------------------------------------------------
#[cfg(feature = "fork")]
pub fn dump_radiation(
    _dimension: &str,
    _pos: [i32; 3],
    _magnitude_sv_per_hour: f64,
) -> Result<(), String> {
    Ok(())
}
#[cfg(not(feature = "fork"))]
pub fn dump_radiation(
    _dimension: &str,
    _pos: [i32; 3],
    _magnitude_sv_per_hour: f64,
) -> Result<(), String> {
    Err("FORK-13 pendente".into())
}

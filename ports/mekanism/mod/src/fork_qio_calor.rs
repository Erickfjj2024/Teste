//! Pontes e requisitos de host (FORK) para o grupo QIO e Calor (`qio_calor`).
//!
//! Isola chamadas de host que dependem de extensões do Horizon SDK.
//! Em ambiente padrão (sem feature `fork`), fornece stubs seguros que retornam erro informativo.

/// FORK-4: Emissão térmica de primeira classe para blocos adjacentes.
#[cfg(feature = "fork")]
pub fn emit_heat(_dimension: &str, _x: i32, _y: i32, _z: i32, _joules: f64) -> Result<(), String> {
    // Quando disponível no host: world::emit_heat(dimension, x, y, z, joules)
    Ok(())
}

#[cfg(not(feature = "fork"))]
pub fn emit_heat(_dimension: &str, _x: i32, _y: i32, _z: i32, _joules: f64) -> Result<(), String> {
    Err("FORK-4 pendente: meio calor (transmitter-medium::heat)".into())
}

/// FORK-7: Dano do feixe laser e interação térmica com entidades.
#[cfg(feature = "fork")]
pub fn laser_entity_damage(_dimension: &str, _from: [f64; 3], _to: [f64; 3], _damage_fe: u64) -> Result<(), String> {
    // Quando disponível no host: entities::laser_damage(...)
    Ok(())
}

#[cfg(not(feature = "fork"))]
pub fn laser_entity_damage(_dimension: &str, _from: [f64; 3], _to: [f64; 3], _damage_fe: u64) -> Result<(), String> {
    Err("FORK-7 pendente: dano a entidades por feixe laser".into())
}

/// FORK-8: Ejeção direcional de itens excedentes no mundo pelo Laser Tractor Beam.
#[cfg(feature = "fork")]
pub fn pop_resource_from_face(_dimension: &str, _pos: [i32; 3], _face: usize, _item: &str, _count: u32) -> Result<(), String> {
    // Quando disponível no host: world::pop_resource_from_face(...)
    Ok(())
}

#[cfg(not(feature = "fork"))]
pub fn pop_resource_from_face(_dimension: &str, _pos: [i32; 3], _face: usize, _item: &str, _count: u32) -> Result<(), String> {
    Err("FORK-8 pendente: pop_resource_from_face no mundo".into())
}

/// FORK-18: Persistência e busca de frequências nomeadas por UUID e modo de segurança.
#[cfg(feature = "fork")]
pub fn lookup_frequency(_name: &str, _owner: Option<u128>) -> Result<Option<String>, String> {
    // Quando disponível no host: frequency::get_frequency(...)
    Ok(None)
}

#[cfg(not(feature = "fork"))]
pub fn lookup_frequency(_name: &str, _owner: Option<u128>) -> Result<Option<String>, String> {
    Err("FORK-18 pendente: gerenciador de frequências universais".into())
}

/// FORK-20: Abertura de GUI enriquecida com widgets customizados (QIO Dashboard, Security Desk).
#[cfg(feature = "fork")]
pub fn open_qio_screen(_player_id: u64, _screen_type: &str, _data: &[u8]) -> Result<(), String> {
    // Quando disponível no host: menu::open_custom_screen(...)
    Ok(())
}

#[cfg(not(feature = "fork"))]
pub fn open_qio_screen(_player_id: u64, _screen_type: &str, _data: &[u8]) -> Result<(), String> {
    Err("FORK-20 pendente: telas customizadas de interface (widgets)".into())
}

/// FORK-21: Emissão de sinal redstone por face para blocos adjacentes.
#[cfg(feature = "fork")]
pub fn set_redstone_output(_dimension: &str, _pos: [i32; 3], _side: usize, _level: u8) -> Result<(), String> {
    // Quando disponível no host: world::set_redstone_output(...)
    Ok(())
}

#[cfg(not(feature = "fork"))]
pub fn set_redstone_output(_dimension: &str, _pos: [i32; 3], _side: usize, _level: u8) -> Result<(), String> {
    Err("FORK-21 pendente: emissão de sinal redstone direcional".into())
}

/// FORK-24: Sincronização em lote cliente-servidor para dados do catálogo virtual QIO (`BulkQIOData`).
#[cfg(feature = "fork")]
pub fn broadcast_bulk_qio_data(_dimension: &str, _pos: [i32; 3], _data: &[u8]) -> Result<(), String> {
    // Quando disponível no host: net::broadcast_bulk(...)
    Ok(())
}

#[cfg(not(feature = "fork"))]
pub fn broadcast_bulk_qio_data(_dimension: &str, _pos: [i32; 3], _data: &[u8]) -> Result<(), String> {
    Err("FORK-24 pendente: transmissão de pacotes bulk em lote".into())
}

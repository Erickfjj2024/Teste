//! Thermodynamic Conductor and Heat Network (`ThermodynamicConductor`, `HeatNetwork`, `HeatAPI`):
//! Transmits heat energy between heat-capable blocks (Resistive Heater, Fuelwood Heater, Thermal Evaporation Plant,
//! Boiler, Fission Reactor).
//!
//! Mekanism 1.21.1 source of truth:
//! - `mekanism.common.tier.ConductorTier` (`ConductorTier.java`)
//! - `mekanism.api.heat.HeatAPI` (`HeatAPI.java`)
//! - `mekanism.common.content.network.HeatNetwork` (`HeatNetwork.java`)
//! - `mekanism.common.content.network.transmitter.ThermodynamicConductor` (`ThermodynamicConductor.java`)
//! - `mekanism.common.capabilities.heat.ITileHeatHandler` (`ITileHeatHandler.java`)
//! - `mekanism.common.capabilities.heat.BasicHeatCapacitor` (`BasicHeatCapacitor.java`)
//! - `mekanism.common.item.ItemAlloy` (`ItemAlloy.java`)
//! - `mekanism.common.tile.transmitter.TileEntityTransmitter` (`TileEntityTransmitter.java`)

use super::*;
use std::collections::{HashMap, VecDeque};

// ============================================================================
// HeatAPI Constants (HeatAPI.java:20-42)
// ============================================================================

/// Default atmospheric temperature in Kelvin (26.85 °C).
/// Java: `HeatAPI.AMBIENT_TEMP = 300` (HeatAPI.java:20).
pub const AMBIENT_TEMP: f64 = 300.0;

/// Air heat transfer coefficient.
/// Java: `HeatAPI.AIR_INVERSE_COEFFICIENT = 10_000` (HeatAPI.java:24).
pub const AIR_INVERSE_COEFFICIENT: f64 = 10_000.0;

/// Default heat capacity in Joules / Kelvin.
/// Java: `HeatAPI.DEFAULT_HEAT_CAPACITY = 1` (HeatAPI.java:28).
pub const DEFAULT_HEAT_CAPACITY: f64 = 1.0;

/// Default inverse conduction coefficient.
/// Java: `HeatAPI.DEFAULT_INVERSE_CONDUCTION = 1` (HeatAPI.java:32).
pub const DEFAULT_INVERSE_CONDUCTION: f64 = 1.0;

/// Default inverse insulation coefficient.
/// Java: `HeatAPI.DEFAULT_INVERSE_INSULATION = 0` (HeatAPI.java:36).
pub const DEFAULT_INVERSE_INSULATION: f64 = 0.0;

/// Minimum delta heat change to be processed.
/// Java: `HeatAPI.EPSILON = Constants.EPSILON` (HeatAPI.java:42).
pub const EPSILON: f64 = 1e-5;

/// Calculates atmospheric temperature based on biome temperature modifier.
/// Baseline is 0.8 (plains), clamped to [-5.0, 5.0].
/// Java: `HeatAPI.getAmbientTemp(double biomeTemp)` (HeatAPI.java:80-81).
pub fn ambient_temp(biome_temp: f64) -> f64 {
    let clamped = biome_temp.clamp(-5.0, 5.0);
    AMBIENT_TEMP + 25.0 * (clamped - 0.8)
}

// ============================================================================
// Conductor Tiers (ConductorTier.java:11-14)
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ConductorTier {
    Basic,
    Advanced,
    Elite,
    Ultimate,
}

impl ConductorTier {
    pub const ALL: [ConductorTier; 4] = [
        ConductorTier::Basic,
        ConductorTier::Advanced,
        ConductorTier::Elite,
        ConductorTier::Ultimate,
    ];

    /// Registry block name.
    pub fn block_name(self) -> &'static str {
        match self {
            ConductorTier::Basic => "mekanism:basic_thermodynamic_conductor",
            ConductorTier::Advanced => "mekanism:advanced_thermodynamic_conductor",
            ConductorTier::Elite => "mekanism:elite_thermodynamic_conductor",
            ConductorTier::Ultimate => "mekanism:ultimate_thermodynamic_conductor",
        }
    }

    /// Inverse conduction coefficient.
    /// Java: `ConductorTier.java:11-14` (5.0 for all tiers).
    pub fn inverse_conduction(self) -> f64 {
        match self {
            ConductorTier::Basic => 5.0,
            ConductorTier::Advanced => 5.0,
            ConductorTier::Elite => 5.0,
            ConductorTier::Ultimate => 5.0,
        }
    }

    /// Heat capacity in Joules / Kelvin.
    /// Java: `ConductorTier.java:11-14` (1.0 J/K for all tiers).
    pub fn heat_capacity(self) -> f64 {
        match self {
            ConductorTier::Basic => DEFAULT_HEAT_CAPACITY,
            ConductorTier::Advanced => DEFAULT_HEAT_CAPACITY,
            ConductorTier::Elite => DEFAULT_HEAT_CAPACITY,
            ConductorTier::Ultimate => DEFAULT_HEAT_CAPACITY,
        }
    }

    /// Inverse insulation coefficient.
    /// Java: `ConductorTier.java:11-14`
    /// - Basic: 10.0
    /// - Advanced: 400.0
    /// - Elite: 8,000.0
    /// - Ultimate: 100,000.0
    pub fn inverse_insulation(self) -> f64 {
        match self {
            ConductorTier::Basic => 10.0,
            ConductorTier::Advanced => 400.0,
            ConductorTier::Elite => 8_000.0,
            ConductorTier::Ultimate => 100_000.0,
        }
    }

    /// Base RGBA color (0.2, 0.2, 0.2, 1.0) for all tiers in Java.
    /// Java: `ConductorTier.java:11-14`.
    pub fn base_color(self) -> [f32; 4] {
        [0.2, 0.2, 0.2, 1.0]
    }

    /// Resolves tier from block name.
    pub fn from_block(block: &str) -> Option<ConductorTier> {
        let name = block.strip_prefix("mekanism:").unwrap_or(block);
        match name {
            "basic_thermodynamic_conductor" => Some(ConductorTier::Basic),
            "advanced_thermodynamic_conductor" => Some(ConductorTier::Advanced),
            "elite_thermodynamic_conductor" => Some(ConductorTier::Elite),
            "ultimate_thermodynamic_conductor" => Some(ConductorTier::Ultimate),
            _ => None,
        }
    }

    /// Next tier in sequence.
    pub fn next_tier(self) -> Option<ConductorTier> {
        match self {
            ConductorTier::Basic => Some(ConductorTier::Advanced),
            ConductorTier::Advanced => Some(ConductorTier::Elite),
            ConductorTier::Elite => Some(ConductorTier::Ultimate),
            ConductorTier::Ultimate => None,
        }
    }

    /// Resolves tier upgrade from an alloy item.
    /// Java: `ItemAlloy.java:29-39` & `TileEntityTransmitter.java:336-340`.
    pub fn upgrade_with_alloy(self, alloy_item: &str) -> Option<ConductorTier> {
        let name = alloy_item.strip_prefix("mekanism:").unwrap_or(alloy_item);
        match (self, name) {
            (ConductorTier::Basic, "alloy_infused") => Some(ConductorTier::Advanced),
            (ConductorTier::Advanced, "alloy_reinforced") => Some(ConductorTier::Elite),
            (ConductorTier::Elite, "alloy_atomic") => Some(ConductorTier::Ultimate),
            _ => None,
        }
    }
}

// ============================================================================
// Connection Types (matching transmitter.rs)
// ============================================================================

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Connection {
    #[default]
    Normal = 0,
    Push = 1,
    Pull = 2,
    None = 3,
}

impl Connection {
    pub fn next(self) -> Connection {
        match self {
            Connection::Normal => Connection::Push,
            Connection::Push => Connection::Pull,
            Connection::Pull => Connection::None,
            Connection::None => Connection::Normal,
        }
    }

    pub fn from_byte(b: u8) -> Connection {
        match b {
            1 => Connection::Push,
            2 => Connection::Pull,
            3 => Connection::None,
            _ => Connection::Normal,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Connection::Normal => "Normal",
            Connection::Push => "Push",
            Connection::Pull => "Pull",
            Connection::None => "None",
        }
    }

    pub fn connects(self) -> bool {
        self != Connection::None
    }
}

// Direction face indices (down, up, north, south, west, east)
pub const DOWN: usize = 0;
pub const UP: usize = 1;
pub const NORTH: usize = 2;
pub const SOUTH: usize = 3;
pub const WEST: usize = 4;
pub const EAST: usize = 5;

pub fn direction_offset(face: usize) -> [i32; 3] {
    match face {
        DOWN => [0, -1, 0],
        UP => [0, 1, 0],
        NORTH => [0, 0, -1],
        SOUTH => [0, 0, 1],
        WEST => [-1, 0, 0],
        _ => [1, 0, 0],
    }
}

pub fn opposite_face(face: usize) -> usize {
    match face {
        DOWN => UP,
        UP => DOWN,
        NORTH => SOUTH,
        SOUTH => NORTH,
        WEST => EAST,
        _ => WEST,
    }
}

// ============================================================================
// Heat Capacitor (BasicHeatCapacitor.java:16-152)
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeatCapacitor {
    /// Heat energy stored in Joules (`heat = temperature * heat_capacity`).
    pub stored_heat: f64,
    /// Thermal capacity in Joules / Kelvin. Minimum 1.0.
    pub heat_capacity: f64,
    /// Inverse conduction coefficient. Minimum 1.0.
    pub inverse_conduction: f64,
    /// Inverse insulation coefficient.
    pub inverse_insulation: f64,
    /// Ambient reference temperature in Kelvin.
    pub ambient_temp: f64,
    /// Heat buffer accumulated during simulation tick before commit.
    pub heat_to_handle: f64,
}

impl Default for HeatCapacitor {
    fn default() -> Self {
        Self::new(
            DEFAULT_HEAT_CAPACITY,
            DEFAULT_INVERSE_CONDUCTION,
            DEFAULT_INVERSE_INSULATION,
            AMBIENT_TEMP,
        )
    }
}

impl HeatCapacitor {
    /// Creates a new heat capacitor initialized at ambient equilibrium.
    /// Java: `BasicHeatCapacitor.java:36-60`.
    pub fn new(capacity: f64, inv_conduction: f64, inv_insulation: f64, ambient: f64) -> Self {
        let cap = capacity.max(1.0);
        let cond = inv_conduction.max(1.0);
        let heat = cap * ambient;
        HeatCapacitor {
            stored_heat: heat,
            heat_capacity: cap,
            inverse_conduction: cond,
            inverse_insulation: inv_insulation.max(0.0),
            ambient_temp: ambient,
            heat_to_handle: 0.0,
        }
    }

    /// Current temperature in Kelvin: `T = heat / heat_capacity`.
    /// Java: `BasicHeatCapacitor.java:69`.
    pub fn temperature(&self) -> f64 {
        self.stored_heat / self.heat_capacity
    }

    /// Queues heat transfer.
    /// Java: `BasicHeatCapacitor.java:95-97`.
    pub fn handle_heat(&mut self, transfer: f64) {
        self.heat_to_handle += transfer;
    }

    /// Commits queued heat if exceeding EPSILON.
    /// Java: `BasicHeatCapacitor.java:104-113`.
    pub fn update(&mut self) -> bool {
        if self.heat_to_handle.abs() > EPSILON {
            self.stored_heat = (self.stored_heat + self.heat_to_handle).max(0.0);
            self.heat_to_handle = 0.0;
            true
        } else {
            self.heat_to_handle = 0.0;
            false
        }
    }

    /// Directly sets heat, clamping to non-negative.
    pub fn set_heat(&mut self, heat: f64) {
        self.stored_heat = heat.max(0.0);
    }
}

// ============================================================================
// Thermodynamic Conductor (ThermodynamicConductor.java:35-187)
// ============================================================================

#[derive(Clone, Debug, PartialEq)]
pub struct ThermodynamicConductor {
    pub tier: ConductorTier,
    pub connections: [Connection; 6],
    pub redstone_reactive: bool,
    pub capacitor: HeatCapacitor,
    pub client_temperature: f64,
}

impl ThermodynamicConductor {
    pub fn new(tier: ConductorTier, ambient: f64) -> Self {
        let capacitor = HeatCapacitor::new(
            tier.heat_capacity(),
            tier.inverse_conduction(),
            tier.inverse_insulation(),
            ambient,
        );
        ThermodynamicConductor {
            tier,
            connections: [Connection::Normal; 6],
            redstone_reactive: false,
            capacitor,
            client_temperature: -1.0,
        }
    }

    /// Serializes conductor state to bytes:
    /// byte 0: version (1)
    /// byte 1: tier (0: Basic, 1: Advanced, 2: Elite, 3: Ultimate)
    /// byte 2: redstone_reactive (0/1)
    /// bytes 3..9: 6 connections
    /// bytes 9..17: stored_heat (f64 LE)
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(17);
        out.push(1); // version
        let tier_b = match self.tier {
            ConductorTier::Basic => 0u8,
            ConductorTier::Advanced => 1u8,
            ConductorTier::Elite => 2u8,
            ConductorTier::Ultimate => 3u8,
        };
        out.push(tier_b);
        out.push(u8::from(self.redstone_reactive));
        for c in self.connections {
            out.push(c as u8);
        }
        out.extend(self.capacitor.stored_heat.to_le_bytes());
        out
    }

    /// Deserializes conductor from bytes.
    pub fn decode(data: &[u8], ambient: f64) -> Option<Self> {
        if data.len() < 17 || data[0] != 1 {
            return None;
        }
        let tier = match data[1] {
            1 => ConductorTier::Advanced,
            2 => ConductorTier::Elite,
            3 => ConductorTier::Ultimate,
            _ => ConductorTier::Basic,
        };
        let redstone_reactive = data[2] == 1;
        let mut connections = [Connection::Normal; 6];
        for i in 0..6 {
            connections[i] = Connection::from_byte(data[3 + i]);
        }
        let mut heat_bytes = [0u8; 8];
        heat_bytes.copy_from_slice(&data[9..17]);
        let heat = f64::from_le_bytes(heat_bytes);

        let mut capacitor = HeatCapacitor::new(
            tier.heat_capacity(),
            tier.inverse_conduction(),
            tier.inverse_insulation(),
            ambient,
        );
        capacitor.set_heat(heat);

        Some(ThermodynamicConductor {
            tier,
            connections,
            redstone_reactive,
            capacitor,
            client_temperature: -1.0,
        })
    }

    /// Checks if client update packet should be sent (5% temperature change threshold).
    /// Java: `ThermodynamicConductor.java:151-154`:
    /// `Math.abs(buffer.getTemperature() - clientTemperature) > (buffer.getTemperature() / 20)`
    pub fn should_sync_client(&mut self) -> bool {
        let temp = self.capacitor.temperature();
        if self.client_temperature < 0.0 {
            self.client_temperature = temp;
            return true;
        }
        let threshold = temp / 20.0;
        if (temp - self.client_temperature).abs() > threshold {
            self.client_temperature = temp;
            true
        } else {
            false
        }
    }
}

// ============================================================================
// Heat Network Simulation (HeatNetwork.java:48-66 & ITileHeatHandler.java:47-100)
// ============================================================================

#[derive(Clone, Debug, Default, PartialEq)]
pub struct HeatNetwork {
    pub mean_temp: f64,
    pub heat_lost: f64,
    pub heat_transferred: f64,
    pub efficiency: f64,
}

impl HeatNetwork {
    pub fn new() -> Self {
        HeatNetwork {
            mean_temp: AMBIENT_TEMP,
            heat_lost: 0.0,
            heat_transferred: 0.0,
            efficiency: 0.0,
        }
    }

    /// Simulates one server tick of the heat network:
    /// 1. Environmental loss across all 6 faces for each conductor.
    /// 2. Heat transfer to adjacent acceptors (external machines / pipes).
    /// 3. Internal conductor-to-conductor heat balancing.
    /// 4. Capacitor commit and metric recalculation.
    ///
    /// Java references:
    /// - `HeatNetwork.java:50-66`
    /// - `ITileHeatHandler.java:47-100`
    /// - `ThermodynamicConductor.java:176-184`
    pub fn tick(
        &mut self,
        conductors: &mut HashMap<[i32; 3], ThermodynamicConductor>,
        ambient: f64,
        external_sinks: &mut HashMap<[i32; 3], HeatCapacitor>,
    ) {
        if conductors.is_empty() {
            self.mean_temp = ambient;
            self.heat_lost = 0.0;
            self.heat_transferred = 0.0;
            self.efficiency = 0.0;
            return;
        }

        let mut new_heat_lost = 0.0;
        let mut new_heat_transferred = 0.0;

        // Collect conductor positions to iterate safely
        let positions: Vec<[i32; 3]> = conductors.keys().copied().collect();

        // --------------------------------------------------------------------
        // Step 1: Environmental Loss (ITileHeatHandler.java:47-62)
        // --------------------------------------------------------------------
        let mut env_losses: Vec<([i32; 3], f64)> = Vec::with_capacity(positions.len());
        for &pos in &positions {
            let (temp, cap, inv_air) = {
                let cond = conductors.get(&pos).unwrap();
                (
                    cond.capacitor.temperature(),
                    cond.capacitor.heat_capacity,
                    AIR_INVERSE_COEFFICIENT + cond.tier.inverse_insulation() + cond.tier.inverse_conduction(),
                )
            };

            let mut total_loss = 0.0;
            for face in 0..6 {
                let neighbor_pos = [
                    pos[0] + direction_offset(face)[0],
                    pos[1] + direction_offset(face)[1],
                    pos[2] + direction_offset(face)[2],
                ];

                // If face touches another conductor in the network, air dissipation does not occur on this face
                if conductors.contains_key(&neighbor_pos) {
                    continue;
                }

                // If face touches an external sink, air dissipation does not occur on this face
                if external_sinks.contains_key(&neighbor_pos) {
                    continue;
                }

                let delta_t = (temp - ambient) / inv_air;
                let heat_loss = delta_t * cap;
                total_loss += heat_loss;

                if delta_t > 0.0 {
                    new_heat_lost += delta_t;
                }
            }
            if total_loss.abs() > 0.0 {
                env_losses.push((pos, total_loss));
            }
        }

        for (pos, loss) in env_losses {
            if let Some(c) = conductors.get_mut(&pos) {
                c.capacitor.handle_heat(-loss);
            }
        }

        // --------------------------------------------------------------------
        // Step 2: Conductor-to-Conductor Heat Transfer (Internal Network Exchange)
        // --------------------------------------------------------------------
        // Conductors in the network exchange heat along connected faces.
        // To avoid duplicate processing, only process when pos < neighbor_pos.
        let mut internal_exchanges: Vec<([i32; 3], [i32; 3], f64)> = Vec::new();
        for &pos in &positions {
            let (temp1, cap1, inv1, connections) = {
                let cond = conductors.get(&pos).unwrap();
                (
                    cond.capacitor.temperature(),
                    cond.capacitor.heat_capacity,
                    cond.tier.inverse_conduction(),
                    cond.connections,
                )
            };

            for face in 0..6 {
                if !connections[face].connects() {
                    continue;
                }

                let neighbor_pos = [
                    pos[0] + direction_offset(face)[0],
                    pos[1] + direction_offset(face)[1],
                    pos[2] + direction_offset(face)[2],
                ];

                if pos < neighbor_pos {
                    if let Some(neighbor) = conductors.get(&neighbor_pos) {
                        let opp = opposite_face(face);
                        if neighbor.connections[opp].connects() {
                            let temp2 = neighbor.capacitor.temperature();
                            let cap2 = neighbor.capacitor.heat_capacity;
                            let inv2 = neighbor.tier.inverse_conduction();

                            // Combined inverse conduction
                            let total_inv = inv1 + inv2;
                            // Temperature difference between the two conductors
                            let delta_t = (temp1 - temp2) / total_inv;
                            let heat_exchange = delta_t * cap1.min(cap2);
                            internal_exchanges.push((pos, neighbor_pos, heat_exchange));
                        }
                    }
                }
            }
        }

        for (p1, p2, exchange) in internal_exchanges {
            if let Some(c1) = conductors.get_mut(&p1) {
                c1.capacitor.handle_heat(-exchange);
            }
            if let Some(c2) = conductors.get_mut(&p2) {
                c2.capacitor.handle_heat(exchange);
            }
        }

        // --------------------------------------------------------------------
        // Step 3: Adjacent Transfer to External Sinks (ITileHeatHandler.java:64-100)
        // --------------------------------------------------------------------
        let mut external_transfers: Vec<([i32; 3], [i32; 3], f64, f64)> = Vec::new();
        for &pos in &positions {
            let (temp, cap, inv_cond, connections) = {
                let cond = conductors.get(&pos).unwrap();
                (
                    cond.capacitor.temperature(),
                    cond.capacitor.heat_capacity,
                    cond.tier.inverse_conduction(),
                    cond.connections,
                )
            };

            for face in 0..6 {
                if !connections[face].connects() {
                    continue;
                }

                let sink_pos = [
                    pos[0] + direction_offset(face)[0],
                    pos[1] + direction_offset(face)[1],
                    pos[2] + direction_offset(face)[2],
                ];

                if let Some(sink) = external_sinks.get(&sink_pos) {
                    let total_inv = sink.inverse_conduction + inv_cond;
                    // Java ITileHeatHandler.java:71:
                    // double tempToTransfer = (getTotalTemperature(side) - getAmbientTemperature(side)) / invConduction;
                    let delta_t = (temp - ambient) / total_inv;
                    let heat_to_transfer = delta_t * cap;
                    external_transfers.push((pos, sink_pos, heat_to_transfer, delta_t));
                }
            }
        }

        for (pos, sink_pos, heat_to_transfer, delta_t) in external_transfers {
            if let Some(c) = conductors.get_mut(&pos) {
                c.capacitor.handle_heat(-heat_to_transfer);
            }
            if let Some(sink) = external_sinks.get_mut(&sink_pos) {
                sink.handle_heat(heat_to_transfer);
            }
            if delta_t > 0.0 {
                new_heat_transferred += delta_t;
            }
        }

        // --------------------------------------------------------------------
        // Step 4: Update Capacitors & Calculate Mean Metrics (HeatNetwork.java:59-66)
        // --------------------------------------------------------------------
        let mut new_sum_temp = 0.0;
        for cond in conductors.values_mut() {
            cond.capacitor.update();
            new_sum_temp += cond.capacitor.temperature();
        }

        self.heat_lost = new_heat_lost;
        self.heat_transferred = new_heat_transferred;
        self.mean_temp = new_sum_temp / (conductors.len() as f64);

        // Efficiency calculation: Math.round(heatTransferred / (heatTransferred + heatLost) * 10_000) / 100F
        // Java: HeatNetwork.java:44
        let denom = self.heat_transferred + self.heat_lost;
        self.efficiency = if denom > 0.0 {
            (self.heat_transferred / denom) * 100.0
        } else {
            0.0
        };
    }

    /// Formatted status string for Configurator display.
    /// Java: `HeatNetwork.java:33, 41-45`.
    pub fn status_text(&self) -> String {
        format!(
            "Stored: {:.2} K | Transferred: {:.2} K/t | Lost: {:.2} K/t | Efficiency: {:.2}%",
            self.mean_temp, self.heat_transferred, self.heat_lost, self.efficiency
        )
    }
}

// ============================================================================
// Configurator and Alloy In-World Upgrade
// ============================================================================

/// Cycles the connection of a conductor face (sneaking click with configurator).
/// Java: `ItemConfigurator.java` & `TileEntityTransmitter.java`.
pub fn cycle_connection(conductor: &mut ThermodynamicConductor, face: usize) -> Connection {
    let f = face.min(5);
    conductor.connections[f] = conductor.connections[f].next();
    conductor.connections[f]
}

/// Toggles whether conductor is sensitive to redstone signals.
pub fn toggle_redstone(conductor: &mut ThermodynamicConductor) -> bool {
    conductor.redstone_reactive = !conductor.redstone_reactive;
    conductor.redstone_reactive
}

/// Upgrades connected conductors in the network when clicked with an alloy item.
/// Java: `TileEntityTransmitter.java:323-370` & `ItemAlloy.java:29-39`.
/// Returns count of conductors upgraded.
pub fn upgrade_conductors_with_alloy(
    conductors: &mut HashMap<[i32; 3], ThermodynamicConductor>,
    clicked_pos: [i32; 3],
    alloy_item: &str,
) -> usize {
    let initial_tier = match conductors.get(&clicked_pos) {
        Some(c) => c.tier,
        None => return 0,
    };

    let target_tier = match initial_tier.upgrade_with_alloy(alloy_item) {
        Some(t) => t,
        None => return 0,
    };

    // Breadth-first traversal of all connected conductors of the same initial tier
    let mut visited = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back(clicked_pos);
    visited.push(clicked_pos);

    while let Some(pos) = queue.pop_front() {
        for face in 0..6 {
            let neighbor_pos = [
                pos[0] + direction_offset(face)[0],
                pos[1] + direction_offset(face)[1],
                pos[2] + direction_offset(face)[2],
            ];

            if !visited.contains(&neighbor_pos) {
                if let Some(neighbor) = conductors.get(&neighbor_pos) {
                    if neighbor.tier == initial_tier {
                        visited.push(neighbor_pos);
                        queue.push_back(neighbor_pos);
                    }
                }
            }
        }
    }

    // Apply tier upgrade while preserving existing heat and connections
    let mut count = 0;
    for pos in visited {
        if let Some(c) = conductors.get_mut(&pos) {
            c.tier = target_tier;
            c.capacitor.inverse_insulation = target_tier.inverse_insulation();
            count += 1;
        }
    }

    count
}

// ============================================================================
// Host Bridge (FORK-4: First-Class Heat Medium in Horizon Engine)
// ============================================================================

/// When the Horizon engine implements `Medium::Heat` natively, the host engine
/// moves heat along transmitter networks automatically.
/// Isolated host bridge:
pub fn host_register_heat_medium() -> bool {
    // FORK-4: Registration of `Medium::Heat` in TransmitterSpec
    false
}

// ============================================================================
// Unit Tests (verifying against Java Mekanism 1.21.1 source lines)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conductor_tier_constants_match_java() {
        // ConductorTier.java:11: BASIC(BaseTier.BASIC, 5, HeatAPI.DEFAULT_HEAT_CAPACITY, 10, Color.rgbad(0.2, 0.2, 0.2, 1))
        assert_eq!(ConductorTier::Basic.inverse_conduction(), 5.0);
        assert_eq!(ConductorTier::Basic.heat_capacity(), 1.0);
        assert_eq!(ConductorTier::Basic.inverse_insulation(), 10.0);

        // ConductorTier.java:12: ADVANCED(BaseTier.ADVANCED, 5, HeatAPI.DEFAULT_HEAT_CAPACITY, 400, Color.rgbad(0.2, 0.2, 0.2, 1))
        assert_eq!(ConductorTier::Advanced.inverse_conduction(), 5.0);
        assert_eq!(ConductorTier::Advanced.heat_capacity(), 1.0);
        assert_eq!(ConductorTier::Advanced.inverse_insulation(), 400.0);

        // ConductorTier.java:13: ELITE(BaseTier.ELITE, 5, HeatAPI.DEFAULT_HEAT_CAPACITY, 8_000, Color.rgbad(0.2, 0.2, 0.2, 1))
        assert_eq!(ConductorTier::Elite.inverse_conduction(), 5.0);
        assert_eq!(ConductorTier::Elite.heat_capacity(), 1.0);
        assert_eq!(ConductorTier::Elite.inverse_insulation(), 8_000.0);

        // ConductorTier.java:14: ULTIMATE(BaseTier.ULTIMATE, 5, HeatAPI.DEFAULT_HEAT_CAPACITY, 100_000, Color.rgbad(0.2, 0.2, 0.2, 1))
        assert_eq!(ConductorTier::Ultimate.inverse_conduction(), 5.0);
        assert_eq!(ConductorTier::Ultimate.heat_capacity(), 1.0);
        assert_eq!(ConductorTier::Ultimate.inverse_insulation(), 100_000.0);
    }

    #[test]
    fn test_heat_api_ambient_temperature_formula() {
        // HeatAPI.java:20: AMBIENT_TEMP = 300
        assert_eq!(AMBIENT_TEMP, 300.0);

        // HeatAPI.java:80-81: AMBIENT_TEMP + 25 * (biomeTemp - 0.8) clamped [-5, 5]
        // Plains biome (0.8) -> 300.0 K
        assert_eq!(ambient_temp(0.8), 300.0);
        // Desert biome (2.0) -> 300.0 + 25 * 1.2 = 330.0 K
        assert_eq!(ambient_temp(2.0), 330.0);
        // Snowy tundra (-0.5) -> 300.0 + 25 * (-1.3) = 267.5 K
        assert_eq!(ambient_temp(-0.5), 267.5);
        // Extreme upper clamp (> 5.0 -> 5.0) -> 300.0 + 25 * 4.2 = 405.0 K
        assert_eq!(ambient_temp(10.0), 405.0);
        // Extreme lower clamp (< -5.0 -> -5.0) -> 300.0 + 25 * (-5.8) = 155.0 K
        assert_eq!(ambient_temp(-8.0), 155.0);
    }

    #[test]
    fn test_capacitor_initialization_and_update() {
        // BasicHeatCapacitor.java:59: storedHeat = heatCapacity * getAmbientTemperature()
        let cap = HeatCapacitor::new(100.0, 5.0, 10.0, 300.0);
        assert_eq!(cap.stored_heat, 30_000.0);
        assert_eq!(cap.temperature(), 300.0);

        // BasicHeatCapacitor.java:105-113: update commits queued heat
        let mut cap = cap;
        cap.handle_heat(500.0);
        assert_eq!(cap.stored_heat, 30_000.0); // Not committed yet
        assert!(cap.update());
        assert_eq!(cap.stored_heat, 30_500.0);
        assert_eq!(cap.temperature(), 305.0);
    }

    #[test]
    fn test_environment_dissipation_basic_conductor() {
        // ITileHeatHandler.java:52-55:
        // invConduction = AIR_INVERSE_COEFFICIENT (10000) + invInsulation (10) + invConduction (5) = 10015.0
        // deltaT = (T - T_amb) / 10015.0
        let mut cond = ThermodynamicConductor::new(ConductorTier::Basic, 300.0);
        cond.capacitor.set_heat(500.0); // T = 500.0 K, heatCapacity = 1.0 J/K
        assert_eq!(cond.capacitor.temperature(), 500.0);

        let mut conductors = HashMap::new();
        conductors.insert([0, 0, 0], cond);
        let mut sinks = HashMap::new();

        let mut net = HeatNetwork::new();
        net.tick(&mut conductors, 300.0, &mut sinks);

        // Delta on each of 6 faces: (500.0 - 300.0) / 10015.0 = 200.0 / 10015.0
        let expected_loss_per_face = 200.0 / 10015.0;
        let expected_total_loss = expected_loss_per_face * 6.0;
        let c = conductors.get(&[0, 0, 0]).unwrap();
        assert!((c.capacitor.temperature() - (500.0 - expected_total_loss)).abs() < 1e-4);
        assert!((net.heat_lost - (expected_loss_per_face * 6.0)).abs() < 1e-4);
    }

    #[test]
    fn test_insulation_prevents_loss_on_ultimate_conductor() {
        // Ultimate Conductor has invInsulation = 100_000.0
        // Total inv = 10000 + 100000 + 5 = 110005.0
        let mut basic = ThermodynamicConductor::new(ConductorTier::Basic, 300.0);
        basic.capacitor.set_heat(1000.0);

        let mut ultimate = ThermodynamicConductor::new(ConductorTier::Ultimate, 300.0);
        ultimate.capacitor.set_heat(1000.0);

        let mut map_basic = HashMap::new();
        map_basic.insert([0, 0, 0], basic);
        let mut map_ultimate = HashMap::new();
        map_ultimate.insert([0, 0, 0], ultimate);

        let mut sinks = HashMap::new();
        let mut net = HeatNetwork::new();

        net.tick(&mut map_basic, 300.0, &mut sinks);
        let loss_basic = net.heat_lost;

        net.tick(&mut map_ultimate, 300.0, &mut sinks);
        let loss_ultimate = net.heat_lost;

        // Ultimate conductor dissipates ~11x less heat than basic conductor
        assert!(loss_ultimate < loss_basic / 10.0);
    }

    #[test]
    fn test_adjacent_transfer_and_network_efficiency() {
        // Conductor transferring heat to an external sink
        let mut cond = ThermodynamicConductor::new(ConductorTier::Basic, 300.0);
        cond.capacitor.set_heat(500.0); // 500 K

        let mut conductors = HashMap::new();
        conductors.insert([0, 0, 0], cond);

        let mut sinks = HashMap::new();
        // External sink at [1, 0, 0] with inv_conduction = 5.0
        let sink = HeatCapacitor::new(100.0, 5.0, 10.0, 300.0);
        sinks.insert([1, 0, 0], sink);

        let mut net = HeatNetwork::new();
        net.tick(&mut conductors, 300.0, &mut sinks);

        assert!(net.heat_transferred > 0.0);
        assert!(net.heat_lost > 0.0);
        assert!(net.efficiency > 0.0 && net.efficiency < 100.0);
        // Sink should have received heat
        assert!(sinks.get(&[1, 0, 0]).unwrap().stored_heat > 30_000.0);
    }

    #[test]
    fn test_client_temperature_sync_threshold() {
        // ThermodynamicConductor.java:151: abs(temp - clientTemp) > (temp / 20)
        let mut cond = ThermodynamicConductor::new(ConductorTier::Basic, 300.0);
        assert!(cond.should_sync_client()); // First sync initializes
        assert_eq!(cond.client_temperature, 300.0);

        // 3% change (< 5%) -> does not sync
        cond.capacitor.set_heat(309.0);
        assert!(!cond.should_sync_client());

        // 6% change (> 5%) -> triggers sync
        cond.capacitor.set_heat(320.0);
        assert!(cond.should_sync_client());
        assert_eq!(cond.client_temperature, 320.0);
    }

    #[test]
    fn test_alloy_upgrade_sequence() {
        // ItemAlloy.java:29-39 & TileEntityTransmitter.java:336-340
        let mut conductors = HashMap::new();
        conductors.insert([0, 0, 0], ThermodynamicConductor::new(ConductorTier::Basic, 300.0));
        conductors.insert([0, 1, 0], ThermodynamicConductor::new(ConductorTier::Basic, 300.0));

        // Upgrade with Infused Alloy -> Advanced
        let count = upgrade_conductors_with_alloy(&mut conductors, [0, 0, 0], "mekanism:alloy_infused");
        assert_eq!(count, 2);
        assert_eq!(conductors.get(&[0, 0, 0]).unwrap().tier, ConductorTier::Advanced);
        assert_eq!(conductors.get(&[0, 1, 0]).unwrap().tier, ConductorTier::Advanced);

        // Upgrade with Reinforced Alloy -> Elite
        let count = upgrade_conductors_with_alloy(&mut conductors, [0, 0, 0], "mekanism:alloy_reinforced");
        assert_eq!(count, 2);
        assert_eq!(conductors.get(&[0, 0, 0]).unwrap().tier, ConductorTier::Elite);

        // Upgrade with Atomic Alloy -> Ultimate
        let count = upgrade_conductors_with_alloy(&mut conductors, [0, 0, 0], "mekanism:alloy_atomic");
        assert_eq!(count, 2);
        assert_eq!(conductors.get(&[0, 0, 0]).unwrap().tier, ConductorTier::Ultimate);
    }

    #[test]
    fn test_conductor_serialization_roundtrip() {
        let mut cond = ThermodynamicConductor::new(ConductorTier::Elite, 300.0);
        cond.connections[NORTH] = Connection::Push;
        cond.connections[SOUTH] = Connection::Pull;
        cond.redstone_reactive = true;
        cond.capacitor.set_heat(456.78);

        let bytes = cond.encode();
        let decoded = ThermodynamicConductor::decode(&bytes, 300.0).expect("decode failed");

        assert_eq!(decoded.tier, ConductorTier::Elite);
        assert_eq!(decoded.connections[NORTH], Connection::Push);
        assert_eq!(decoded.connections[SOUTH], Connection::Pull);
        assert_eq!(decoded.connections[DOWN], Connection::Normal);
        assert!(decoded.redstone_reactive);
        assert!((decoded.capacitor.stored_heat - 456.78).abs() < 1e-6);
    }
}

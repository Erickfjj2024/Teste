//! The heat of the world: the conductors, the resistive heaters and the fuelwood heaters of every loaded chunk, simulated
//! together once per tick. The pieces (`heat`, `resistive_heater`, `fuelwood_heater`) know one block each; this joins them
//! by position the way `ITileHeatHandler` joins neighbouring tiles.
//!
//! Heat only crosses between touching blocks, so one pass over every block of a dimension equals one pass per network
//! (`HeatNetwork` only adds the statistics of its own network, which the screens of the port do not show).
//! Difference from Java: the heat a conductor hands to a heater is committed in the same tick (Java commits it when the
//! heater next ticks). It is one tick earlier and conserves the same total.

use std::collections::HashMap;

use horizon_mod_sdk::server::{world, BlockEntity, BlockEntityUpdate, ItemStack};

use crate::{fuelwood_heater, heat, resistive_heater};

type Pos = [i32; 3];

fn block_of(e: &BlockEntity) -> &str {
    e.state.split('[').next().unwrap_or("")
}

/// Whether `block` is a block of this module.
pub fn is_heat_block(block: &str) -> bool {
    heat::ConductorTier::from_block(block).is_some() || block == resistive_heater::BLOCK || block == fuelwood_heater::BLOCK
}

fn ambient_at(e: &BlockEntity) -> f64 {
    world::get_environment(&e.dimension, e.pos.x, e.pos.y, e.pos.z).map_or(heat::AMBIENT_TEMP, |env| heat::ambient_temp(f64::from(env.temperature)))
}

fn update(e: &BlockEntity, data: Vec<u8>) -> BlockEntityUpdate {
    BlockEntityUpdate {
        dimension: e.dimension.clone(),
        pos: e.pos,
        data,
        items: None,
        energy: None,
        energy_faces: None,
        tanks: None,
        tank_faces: None,
        menu_data: None,
        transmitter_faces: None,
    }
}

fn neighbour(pos: Pos, face: usize) -> Pos {
    let o = heat::direction_offset(face);
    [pos[0] + o[0], pos[1] + o[1], pos[2] + o[2]]
}

/// The state of the heat blocks of one dimension, and the outcome of a tick.
pub struct Heat {
    conductors: HashMap<Pos, heat::ThermodynamicConductor>,
    resistive: HashMap<Pos, resistive_heater::ResistiveHeater>,
    fuelwood: HashMap<Pos, fuelwood_heater::FuelwoodHeater>,
}

impl Heat {
    pub fn new() -> Self {
        Heat { conductors: HashMap::new(), resistive: HashMap::new(), fuelwood: HashMap::new() }
    }

    /// One tick of the three kinds together at `ambient` (a heater at a position uses its own ambient: `heater_ambient`).
    pub fn tick(&mut self, ambient: f64, powered: &HashMap<Pos, bool>, heater_ambient: &HashMap<Pos, f64>) {
        // Heaters first: they make heat, lose it to the air and hand it to the conductors that touch them.
        let mut to_conductors: Vec<(Pos, f64)> = Vec::new();
        let positions: Vec<Pos> = self.resistive.keys().copied().collect();
        for pos in positions {
            let mut sinks: Vec<heat::HeatCapacitor> = Vec::new();
            let mut at: Vec<Pos> = Vec::new();
            for face in 0..6 {
                let n = neighbour(pos, face);
                if let Some(c) = self.conductors.get(&n) {
                    if c.connections[heat::opposite_face(face)].connects() {
                        let mut copy = c.capacitor;
                        copy.heat_to_handle = 0.0;
                        sinks.push(copy);
                        at.push(n);
                    }
                }
            }
            let amb = heater_ambient.get(&pos).copied().unwrap_or(ambient);
            let h = self.resistive.get_mut(&pos).unwrap();
            h.tick(powered.get(&pos).copied().unwrap_or(false), amb, &mut sinks);
            for (n, s) in at.into_iter().zip(sinks) {
                to_conductors.push((n, s.heat_to_handle));
            }
        }
        let positions: Vec<Pos> = self.fuelwood.keys().copied().collect();
        for pos in positions {
            let mut sinks: Vec<heat::HeatCapacitor> = Vec::new();
            let mut at: Vec<Pos> = Vec::new();
            for face in 0..6 {
                let n = neighbour(pos, face);
                if let Some(c) = self.conductors.get(&n) {
                    if c.connections[heat::opposite_face(face)].connects() {
                        let mut copy = c.capacitor;
                        copy.heat_to_handle = 0.0;
                        sinks.push(copy);
                        at.push(n);
                    }
                }
            }
            let amb = heater_ambient.get(&pos).copied().unwrap_or(ambient);
            let h = self.fuelwood.get_mut(&pos).unwrap();
            h.tick(amb, &mut sinks);
            for (n, s) in at.into_iter().zip(sinks) {
                to_conductors.push((n, s.heat_to_handle));
            }
        }
        for (n, q) in to_conductors {
            if let Some(c) = self.conductors.get_mut(&n) {
                c.capacitor.handle_heat(q);
            }
        }

        // The conductors lose heat to the air and pass it to their neighbours, the heaters among them.
        let mut sinks: HashMap<Pos, heat::HeatCapacitor> = HashMap::new();
        for (pos, h) in &self.resistive {
            sinks.insert(*pos, h.capacitor);
        }
        for (pos, h) in &self.fuelwood {
            sinks.insert(*pos, h.capacitor);
        }
        for s in sinks.values_mut() {
            s.heat_to_handle = 0.0;
        }
        let mut network = heat::HeatNetwork::new();
        network.tick(&mut self.conductors, ambient, &mut sinks);
        for (pos, s) in sinks {
            if let Some(h) = self.resistive.get_mut(&pos) {
                h.capacitor.handle_heat(s.heat_to_handle);
                h.capacitor.update();
            } else if let Some(h) = self.fuelwood.get_mut(&pos) {
                h.capacitor.handle_heat(s.heat_to_handle);
                h.capacitor.update();
            }
        }
    }
}

/// Runs the heat blocks among `entities` for one tick and gives their new state, each dimension on its own.
pub fn run(entities: &[&BlockEntity]) -> Vec<BlockEntityUpdate> {
    let mut dimensions: Vec<&str> = entities.iter().map(|e| e.dimension.as_str()).collect();
    dimensions.sort_unstable();
    dimensions.dedup();
    let mut out = Vec::new();
    for d in dimensions {
        let group: Vec<&BlockEntity> = entities.iter().copied().filter(|e| e.dimension == d).collect();
        out.extend(run_dimension(&group));
    }
    out
}

fn run_dimension(entities: &[&BlockEntity]) -> Vec<BlockEntityUpdate> {
    let mut world_heat = Heat::new();
    let mut powered: HashMap<Pos, bool> = HashMap::new();
    let mut ambients: HashMap<Pos, f64> = HashMap::new();
    let mut by_pos: HashMap<Pos, &BlockEntity> = HashMap::new();
    let mut ambient = heat::AMBIENT_TEMP;
    for e in entities {
        let pos = [e.pos.x, e.pos.y, e.pos.z];
        let a = ambient_at(e);
        ambient = a;
        ambients.insert(pos, a);
        by_pos.insert(pos, e);
        let block = block_of(e);
        if let Some(tier) = heat::ConductorTier::from_block(block) {
            let c = heat::ThermodynamicConductor::decode(&e.data, a).unwrap_or_else(|| heat::ThermodynamicConductor::new(tier, a));
            world_heat.conductors.insert(pos, c);
        } else if block == resistive_heater::BLOCK {
            let mut h = resistive_heater::ResistiveHeater::decode(&e.data, a).unwrap_or_else(|| resistive_heater::ResistiveHeater::new(a));
            h.energy_stored_fe = e.energy;
            powered.insert(pos, world::get_redstone(&e.dimension, e.pos.x, e.pos.y, e.pos.z) > 0);
            world_heat.resistive.insert(pos, h);
        } else if block == fuelwood_heater::BLOCK {
            let mut h = fuelwood_heater::FuelwoodHeater::decode(&e.data, a).unwrap_or_else(|| fuelwood_heater::FuelwoodHeater::new(a));
            match e.items.first().cloned().flatten() {
                Some(s) => {
                    h.fuel_slot_item = Some(s.item);
                    h.fuel_slot_count = s.count;
                }
                None => {
                    h.fuel_slot_item = None;
                    h.fuel_slot_count = 0;
                }
            }
            world_heat.fuelwood.insert(pos, h);
        }
    }
    world_heat.tick(ambient, &powered, &ambients);

    let mut out = Vec::new();
    for (pos, e) in by_pos {
        if let Some(c) = world_heat.conductors.get(&pos) {
            out.push(update(e, c.encode()));
        } else if let Some(h) = world_heat.resistive.get(&pos) {
            let mut u = update(e, h.encode());
            u.energy = Some(h.energy_stored_fe);
            u.menu_data = Some(h.menu_data());
            out.push(u);
        } else if let Some(h) = world_heat.fuelwood.get(&pos) {
            let mut u = update(e, h.encode());
            u.items = Some(vec![h.fuel_slot_item.clone().filter(|_| h.fuel_slot_count > 0).map(|item| ItemStack { item, count: h.fuel_slot_count, components: String::new() })]);
            u.menu_data = Some(h.menu_data());
            out.push(u);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_resistive_heater_warms_the_conductor_beside_it_and_the_conductor_the_next_one() {
        let mut w = Heat::new();
        let mut heater = resistive_heater::ResistiveHeater::new(300.0);
        heater.energy_stored_fe = heater.max_energy_fe();
        w.resistive.insert([0, 0, 0], heater);
        w.conductors.insert([1, 0, 0], heat::ThermodynamicConductor::new(heat::ConductorTier::Basic, 300.0));
        w.conductors.insert([2, 0, 0], heat::ThermodynamicConductor::new(heat::ConductorTier::Basic, 300.0));
        let powered = HashMap::from([([0, 0, 0], true)]);
        for _ in 0..200 {
            w.tick(300.0, &powered, &HashMap::new());
        }
        let near = w.conductors[&[1, 0, 0]].capacitor.temperature();
        let far = w.conductors[&[2, 0, 0]].capacitor.temperature();
        assert!(near > 300.0, "the conductor beside the heater warms up: {near}");
        assert!(far > 300.0 && far <= near + 1e-6, "the next one warms up behind it: {far} {near}");
        // The original moves heat by (own temperature - AMBIENT) / resistance (`ITileHeatHandler.simulateAdjacent`), not by the
        // difference to the sink, so a conductor of capacity 1 ends far hotter than the heater (checked against the Java).
        assert!(w.resistive[&[0, 0, 0]].capacitor.temperature() > 300.0);
    }

    #[test]
    fn an_idle_world_stays_at_ambient() {
        let mut w = Heat::new();
        w.conductors.insert([0, 0, 0], heat::ThermodynamicConductor::new(heat::ConductorTier::Elite, 300.0));
        for _ in 0..50 {
            w.tick(300.0, &HashMap::new(), &HashMap::new());
        }
        assert!((w.conductors[&[0, 0, 0]].capacitor.temperature() - 300.0).abs() < 1e-6);
    }

    #[test]
    fn a_conductor_does_not_take_heat_through_a_disconnected_face() {
        let mut w = Heat::new();
        let mut heater = resistive_heater::ResistiveHeater::new(300.0);
        heater.energy_stored_fe = heater.max_energy_fe();
        w.resistive.insert([0, 0, 0], heater);
        let mut c = heat::ThermodynamicConductor::new(heat::ConductorTier::Basic, 300.0);
        c.connections[heat::opposite_face(heat::EAST)] = heat::Connection::None;
        w.conductors.insert([1, 0, 0], c);
        let powered = HashMap::from([([0, 0, 0], true)]);
        for _ in 0..50 {
            w.tick(300.0, &powered, &HashMap::new());
        }
        assert!((w.conductors[&[1, 0, 0]].capacitor.temperature() - 300.0).abs() < 1e-6);
    }
}

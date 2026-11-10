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

use crate::{boiler, evaporation, fuelwood_heater, heat, resistive_heater, Mekanism};

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

/// What kind of multiblock a heat sink belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SinkKind {
    Evaporation,
    Boiler,
}

/// A block that is part of a multiblock and shows its heat capacitor to the heat blocks around it. The multiblock publishes it each
/// tick (`TileEntityThermalEvaporationValve` and `TileEntityBoilerCasing.getInitialHeatCapacitors` give the structure's capacitors to every side).
/// Heat given to it is kept in `pending` and the multiblock commits it when it next ticks, as Java does (`handleHeat` queues,
/// `updateHeatCapacitors` commits).
#[derive(Clone, Copy, Debug)]
pub struct HeatSink {
    pub kind: SinkKind,
    /// The `min` corner of the structure, which is what the multiblock state is keyed by.
    pub key: Pos,
    pub capacitor: heat::HeatCapacitor,
}

/// The state of the heat blocks of one dimension, and the outcome of a tick.
pub struct Heat {
    /// Heat sinks of multiblocks around, by position.
    external: HashMap<Pos, heat::HeatCapacitor>,
    conductors: HashMap<Pos, heat::ThermodynamicConductor>,
    resistive: HashMap<Pos, resistive_heater::ResistiveHeater>,
    fuelwood: HashMap<Pos, fuelwood_heater::FuelwoodHeater>,
}

impl Heat {
    pub fn new() -> Self {
        Heat { external: HashMap::new(), conductors: HashMap::new(), resistive: HashMap::new(), fuelwood: HashMap::new() }
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
                else if let Some(c) = self.external.get(&n) {
                    let mut copy = *c;
                    copy.heat_to_handle = 0.0;
                    sinks.push(copy);
                    at.push(n);
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
                else if let Some(c) = self.external.get(&n) {
                    let mut copy = *c;
                    copy.heat_to_handle = 0.0;
                    sinks.push(copy);
                    at.push(n);
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
            } else if let Some(c) = self.external.get_mut(&n) {
                c.handle_heat(q);
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
        let outside: Vec<Pos> = self.external.keys().copied().collect();
        for pos in &outside {
            let mut copy = self.external[pos];
            copy.heat_to_handle = 0.0;
            sinks.insert(*pos, copy);
        }
        let mut network = heat::HeatNetwork::new();
        network.tick(&mut self.conductors, ambient, &mut sinks);
        for (pos, s) in sinks {
            if let Some(c) = self.external.get_mut(&pos) {
                c.handle_heat(s.heat_to_handle);
            } else if let Some(h) = self.resistive.get_mut(&pos) {
                h.capacitor.handle_heat(s.heat_to_handle);
                h.capacitor.update();
            } else if let Some(h) = self.fuelwood.get_mut(&pos) {
                h.capacitor.handle_heat(s.heat_to_handle);
                h.capacitor.update();
            }
        }
    }
}

/// Whether `p` is on the surface of the box `min..=max`.
fn on_shell(p: Pos, min: Pos, max: Pos) -> bool {
    (0..3).all(|i| p[i] >= min[i] && p[i] <= max[i]) && (0..3).any(|i| p[i] == min[i] || p[i] == max[i])
}

impl Mekanism {
    /// The multiblock blocks that touch a heat block, with the capacitor each shows (`getAdjacent` finds the neighbour's heat capability).
    /// Evaporation shows its heat only through the valves (`TileEntityThermalEvaporationValve`), the boiler through every casing part
    /// and valve (`TileEntityBoilerCasing`); both give the structure's one capacitor.
    pub(crate) fn heat_sinks_near(&self, heat_blocks: &[&BlockEntity]) -> HashMap<(String, Pos), HeatSink> {
        let mut out = HashMap::new();
        if self.plants.is_empty() && self.boilers.is_empty() {
            return out;
        }
        for e in heat_blocks {
            let at = [e.pos.x, e.pos.y, e.pos.z];
            for face in 0..6 {
                let n = neighbour(at, face);
                if out.contains_key(&(e.dimension.clone(), n)) {
                    continue;
                }
                for ((dim, key), plant) in &self.plants {
                    let Some((min, max)) = plant.bounds else { continue };
                    if *dim != e.dimension || !on_shell(n, min, max) {
                        continue;
                    }
                    if world::get_block(dim, n[0], n[1], n[2]).is_some_and(|b| b.split('[').next() == Some(evaporation::VALVE)) {
                        let height = (max[1] - min[1] + 1) as u8;
                        let mut c = heat::HeatCapacitor::new(evaporation::heat_capacity(height), 1.0, 0.0, evaporation::AMBIENT);
                        c.stored_heat = plant.temperature * c.heat_capacity;
                        out.insert((dim.clone(), n), HeatSink { kind: SinkKind::Evaporation, key: *key, capacitor: c });
                    }
                }
                for ((dim, key), b) in &self.boilers {
                    let Some((min, max)) = b.bounds else { continue };
                    if *dim != e.dimension || !on_shell(n, min, max) {
                        continue;
                    }
                    if world::get_block(dim, n[0], n[1], n[2]).is_some_and(|x| matches!(x.split('[').next(), Some(boiler::CASING | boiler::VALVE))) {
                        let size = [max[0] - min[0] + 1, max[1] - min[1] + 1, max[2] - min[2] + 1];
                        let shell = size[0] * size[1] * size[2] - (size[0] - 2) * (size[1] - 2) * (size[2] - 2);
                        let mut c = heat::HeatCapacitor::new(boiler::CASING_HEAT * f64::from(shell), boiler::CONDUCTION, boiler::INSULATION, boiler::AIR);
                        c.stored_heat = b.temperature * c.heat_capacity;
                        out.insert((dim.clone(), n), HeatSink { kind: SinkKind::Boiler, key: *key, capacitor: c });
                    }
                }
            }
        }
        out
    }

    /// Queues on each multiblock the heat the heat blocks passed to its blocks.
    pub(crate) fn commit_heat_given(&mut self, sinks: &HashMap<(String, Pos), HeatSink>, given: Vec<(String, Pos, f64)>) {
        for (dim, pos, q) in given {
            let Some(s) = sinks.get(&(dim.clone(), pos)) else { continue };
            match s.kind {
                SinkKind::Evaporation => {
                    if let Some(p) = self.plants.get_mut(&(dim, s.key)) {
                        p.pending_heat += q;
                    }
                }
                SinkKind::Boiler => {
                    if let Some(b) = self.boilers.get_mut(&(dim, s.key)) {
                        b.pending_heat += q;
                    }
                }
            }
        }
    }
}

/// Runs the heat blocks among `entities` for one tick and gives their new state, each dimension on its own, with the heat they
/// passed to the multiblocks in `sinks` (dimension, position, joules) for the multiblocks to commit.
pub fn run(entities: &[&BlockEntity], sinks: &HashMap<(String, Pos), HeatSink>) -> (Vec<BlockEntityUpdate>, Vec<(String, Pos, f64)>) {
    let mut dimensions: Vec<&str> = entities.iter().map(|e| e.dimension.as_str()).collect();
    dimensions.sort_unstable();
    dimensions.dedup();
    let mut out = Vec::new();
    let mut given = Vec::new();
    for d in dimensions {
        let group: Vec<&BlockEntity> = entities.iter().copied().filter(|e| e.dimension == d).collect();
        let outside: HashMap<Pos, heat::HeatCapacitor> =
            sinks.iter().filter(|((dim, _), _)| dim == d).map(|((_, pos), s)| (*pos, s.capacitor)).collect();
        let (updates, deliveries) = run_dimension(&group, outside);
        out.extend(updates);
        given.extend(deliveries.into_iter().map(|(pos, q)| (d.to_owned(), pos, q)));
    }
    (out, given)
}

fn run_dimension(entities: &[&BlockEntity], outside: HashMap<Pos, heat::HeatCapacitor>) -> (Vec<BlockEntityUpdate>, Vec<(Pos, f64)>) {
    let mut world_heat = Heat::new();
    world_heat.external = outside;
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
    for c in world_heat.external.values_mut() {
        c.heat_to_handle = 0.0;
    }
    world_heat.tick(ambient, &powered, &ambients);
    let deliveries: Vec<(Pos, f64)> = world_heat.external.iter().filter(|(_, c)| c.heat_to_handle != 0.0).map(|(p, c)| (*p, c.heat_to_handle)).collect();

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
    (out, deliveries)
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
    fn a_conductor_gives_heat_to_a_multiblock_block_and_a_heater_next_to_one_does_too() {
        // A basic conductor beside a boiler casing (capacity 50 a block): the casing capacitor is queued heat as the conductor's
        // `simulateAdjacent` finds it as its sink. A heater touching the casing directly gives it heat too.
        let casing = heat::HeatCapacitor::new(5000.0, boiler::CONDUCTION, boiler::INSULATION, boiler::AIR);
        let mut w = Heat::new();
        w.external.insert([2, 0, 0], casing);
        w.external.insert([0, 1, 0], casing);
        let mut heater = resistive_heater::ResistiveHeater::new(300.0);
        heater.energy_stored_fe = heater.max_energy_fe();
        w.resistive.insert([0, 0, 0], heater);
        w.conductors.insert([1, 0, 0], heat::ThermodynamicConductor::new(heat::ConductorTier::Basic, 300.0));
        let powered = HashMap::from([([0, 0, 0], true)]);
        let (mut by_conductor, mut by_heater) = (0.0, 0.0);
        for _ in 0..100 {
            for c in w.external.values_mut() {
                c.heat_to_handle = 0.0;
            }
            w.tick(300.0, &powered, &HashMap::new());
            by_conductor += w.external[&[2, 0, 0]].heat_to_handle;
            by_heater += w.external[&[0, 1, 0]].heat_to_handle;
        }
        assert!(by_conductor > 0.0, "the conductor passes heat to the block at its end: {by_conductor}");
        assert!(by_heater > 0.0, "the heater passes heat to the block it touches: {by_heater}");
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

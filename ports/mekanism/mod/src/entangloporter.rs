//! The quantum entangloporter (`TileEntityQuantumEntangloporter`, `InventoryFrequency`): every entangloporter on the same frequency shares one
//! buffer of energy, one fluid, one chemical and one item stack, and gives every side access to it, so that cables, pipes, tubes and
//! transporters beside any of them reach the same contents, wherever they are (in any dimension).
//!
//! Frequencies are numbers (1 and up; 0 is none) chosen with buttons: there is no name entry, no private frequencies and no security. The blocks
//! hold mirrors of the shared buffer (an equal share of the energy and tanks each); the item slot is shown whole in all.
//!
//! Not here: heat, the side configuration screen, ejecting on its own (a cable or pipe has to pull), chunk loading.
use super::*;
use std::collections::HashMap;

pub const BLOCK: &str = "mekanism:quantum_entangloporter";
/// `entangloporterEnergyBuffer`, `entangloporterFluidBuffer` and `entangloporterChemicalBuffer` (the ultimate tiers' capacities).
pub const ENERGY_CAPACITY: u64 = crate::fe(256_000_000);
pub const FLUID_CAPACITY: u64 = 256_000;
pub const CHEMICAL_CAPACITY: u64 = 8_192_000;
pub const SPEC_CAPACITY: u64 = 1 << 50;
pub const MAX_FREQUENCY: u8 = 99;
pub const NOTHING: &str = "mekanism:nothing";
/// The buttons of the screen.
pub const FREQUENCY_DOWN: i32 = 7500;
pub const FREQUENCY_UP: i32 = 7501;

pub fn encode(frequency: u8, energy: u64) -> Vec<u8> {
    let mut out = vec![1, frequency];
    out.extend(energy.to_le_bytes());
    out
}

pub fn decode(data: &[u8]) -> Option<(u8, u64)> {
    if data.len() < 10 || data[0] != 1 {
        return None;
    }
    let mut e = [0u8; 8];
    e.copy_from_slice(&data[2..10]);
    Some((data[1], u64::from_le_bytes(e)))
}

/// The share of `total` the block at `index` of `count` holds (they add up to the total).
pub fn share(total: u64, index: u64, count: u64) -> u64 {
    total / count + u64::from(index < total % count)
}

/// A press of a button on the frequency.
pub fn press(frequency: u8, id: i32) -> u8 {
    match id {
        FREQUENCY_UP => (frequency + 1).min(MAX_FREQUENCY),
        FREQUENCY_DOWN => frequency.saturating_sub(1),
        _ => frequency,
    }
}

/// What every entangloporter of one frequency shares.
#[derive(Default)]
pub struct Shared {
    pub energy: u64,
    pub fluid: process::Tank,
    pub chemical: process::Tank,
    pub item: Option<Stack>,
    pub started: bool,
    /// What each block was last given: energy, fluid, chemical and the item.
    pub given: HashMap<(String, i32, i32, i32), (u64, u64, u64, Option<Stack>)>,
}

impl Mekanism {
    pub(crate) fn run_entanglements(&mut self, qes: &[&BlockEntity], tick: u64) -> Vec<BlockEntityUpdate> {
        let presses = std::mem::take(&mut self.qe_buttons);
        let mut out = Vec::new();
        let mut by: HashMap<u8, Vec<&BlockEntity>> = HashMap::new();
        let mut frequency_of: HashMap<(String, i32, i32, i32), u8> = HashMap::new();
        for e in qes {
            let key = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
            let (mut f, _) = decode(&e.data).unwrap_or((0, 0));
            for (_, p, id) in presses.iter().filter(|(d, p, _)| *d == e.dimension && (p.x, p.y, p.z) == (e.pos.x, e.pos.y, e.pos.z)) {
                let _ = p;
                f = press(f, *id);
            }
            frequency_of.insert(key, f);
            if f > 0 {
                by.entry(f).or_default().push(*e);
            }
        }
        // The blocks with no frequency hold nothing.
        for e in qes {
            let key = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
            if frequency_of[&key] == 0 {
                let _ = world::set_energy_capacity(&e.dimension, e.pos.x, e.pos.y, e.pos.z, 1);
                for i in 0..2 {
                    let _ = world::set_tank_filter(&e.dimension, e.pos.x, e.pos.y, e.pos.z, i, &[NOTHING.to_owned()]);
                }
                out.push(BlockEntityUpdate {
                    dimension: e.dimension.clone(),
                    pos: e.pos,
                    data: encode(0, 0),
                    items: Some(vec![None]),
                    energy: Some(0),
                    energy_faces: None,
                    tanks: Some(vec![Tank { substance: String::new(), amount: 0 }, Tank { substance: String::new(), amount: 0 }]),
                    tank_faces: None,
                    menu_data: Some(vec![0; 11]),
                    transmitter_faces: None,
                });
            }
        }
        for (frequency, mut group) in by {
            group.sort_by_key(|e| (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z));
            let s = self.entangled.entry(frequency).or_default();
            let held_tank = |e: &BlockEntity, i: usize| e.tanks.get(i).cloned().unwrap_or(Tank { substance: String::new(), amount: 0 });
            let slot = |e: &BlockEntity| stack(e.items.first().unwrap_or(&None));
            if !s.started {
                s.started = true;
                s.energy = group.iter().filter_map(|e| decode(&e.data)).map(|(_, en)| en).max().unwrap_or(0).min(ENERGY_CAPACITY);
                if let Some(t) = group.iter().map(|e| held_tank(e, 0)).max_by_key(|t| t.amount).filter(|t| t.amount > 0) {
                    s.fluid = process::Tank { chemical: t.substance, amount: t.amount.min(FLUID_CAPACITY) };
                }
                if let Some(t) = group.iter().map(|e| held_tank(e, 1)).max_by_key(|t| t.amount).filter(|t| t.amount > 0) {
                    s.chemical = process::Tank { chemical: t.substance, amount: t.amount.min(CHEMICAL_CAPACITY) };
                }
                s.item = group.iter().find_map(|e| slot(e));
            } else {
                for e in &group {
                    let key = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
                    let Some((ge, gf, gc, gi)) = s.given.get(&key).cloned() else { continue };
                    // What the block gained or lost, since it was last written.
                    s.energy = (i128::from(s.energy) + i128::from(e.energy) - i128::from(ge)).clamp(0, i128::from(ENERGY_CAPACITY)) as u64;
                    let f = held_tank(e, 0);
                    s.fluid.amount = (i128::from(s.fluid.amount) + i128::from(f.amount) - i128::from(gf)).clamp(0, i128::from(FLUID_CAPACITY)) as u64;
                    if s.fluid.chemical.is_empty() && f.amount > 0 {
                        s.fluid.chemical = f.substance;
                    }
                    let c = held_tank(e, 1);
                    s.chemical.amount = (i128::from(s.chemical.amount) + i128::from(c.amount) - i128::from(gc)).clamp(0, i128::from(CHEMICAL_CAPACITY)) as u64;
                    if s.chemical.chemical.is_empty() && c.amount > 0 {
                        s.chemical.chemical = c.substance;
                    }
                    // The item: the first block whose slot changed decides.
                    let now = slot(e);
                    if now != gi {
                        s.item = now;
                    }
                }
                if s.fluid.amount == 0 {
                    s.fluid.chemical.clear();
                }
                if s.chemical.amount == 0 {
                    s.chemical.chemical.clear();
                }
            }
            let count = group.len() as u64;
            let room = ENERGY_CAPACITY - s.energy;
            let half = |v: u64, sh: u32| ((v >> sh) & 0xFFFF) as i32;
            let data = vec![
                i32::from(frequency),
                half(s.energy / 1000, 0),
                half(s.energy / 1000, 16),
                half(s.fluid.amount, 0),
                half(s.fluid.amount, 16),
                half(s.chemical.amount, 0),
                half(s.chemical.amount, 16),
                if s.fluid.amount == 0 { 0 } else { chem::index(&s.fluid.chemical) },
                if s.chemical.amount == 0 { 0 } else { chem::index(&s.chemical.chemical) },
                count as i32,
                s.item.as_ref().map_or(0, |i| i.count as i32),
            ];
            let (energy_total, fluid, chemical, item) = (s.energy, s.fluid.clone(), s.chemical.clone(), s.item.clone());
            for (index, e) in group.iter().enumerate() {
                let key = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
                let (se, sf, sc) = (share(energy_total, index as u64, count), share(fluid.amount, index as u64, count), share(chemical.amount, index as u64, count));
                s.given.insert(key, (se, sf, sc, item.clone()));
                let _ = world::set_energy_capacity(&e.dimension, e.pos.x, e.pos.y, e.pos.z, (se + room / count + 1).min(ENERGY_CAPACITY));
                let ok = |amount: u64, cap: u64| if amount >= cap { vec![NOTHING.to_owned()] } else { Vec::new() };
                let _ = world::set_tank_filter(&e.dimension, e.pos.x, e.pos.y, e.pos.z, 0, &ok(fluid.amount, FLUID_CAPACITY));
                let _ = world::set_tank_filter(&e.dimension, e.pos.x, e.pos.y, e.pos.z, 1, &ok(chemical.amount, CHEMICAL_CAPACITY));
                let keep = if tick % 20 == 0 || decode(&e.data).map(|d| d.0) != Some(frequency) { encode(frequency, energy_total) } else { e.data.clone() };
                out.push(BlockEntityUpdate {
                    dimension: e.dimension.clone(),
                    pos: e.pos,
                    data: keep,
                    items: Some(vec![item.as_ref().map(|i| ItemStack { item: i.item.clone(), count: i.count, components: i.components.clone() })]),
                    energy: Some(se),
                    energy_faces: None,
                    tanks: Some(vec![
                        Tank { substance: fluid.chemical.clone(), amount: sf },
                        Tank { substance: chemical.chemical.clone(), amount: sc },
                    ]),
                    tank_faces: None,
                    menu_data: Some(data.clone()),
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
    fn the_shares_add_up_to_the_total() {
        for total in [0u64, 1, 7, 100, 12345] {
            for count in [1u64, 2, 3, 5] {
                assert_eq!((0..count).map(|i| share(total, i, count)).sum::<u64>(), total);
            }
        }
    }

    #[test]
    fn the_frequency_moves_within_its_range() {
        assert_eq!(press(0, FREQUENCY_DOWN), 0);
        assert_eq!(press(0, FREQUENCY_UP), 1);
        assert_eq!(press(MAX_FREQUENCY, FREQUENCY_UP), MAX_FREQUENCY);
        assert_eq!(press(5, 99), 5);
    }

    #[test]
    fn a_block_remembers_its_frequency() {
        assert_eq!(decode(&encode(7, 123_456_789)), Some((7, 123_456_789)));
        assert_eq!(decode(&[]), None);
    }
}

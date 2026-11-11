//! The quantum entangloporter (`TileEntityQuantumEntangloporter`, `InventoryFrequency`): every entangloporter on the same frequency shares one
//! buffer of energy, one fluid, one chemical and one item stack, and gives every side access to it, so that cables, pipes, tubes and
//! transporters beside any of them reach the same contents, wherever they are (in any dimension).
//!
//! Frequencies are named ones (`frequency.rs`), public or private, chosen on the screen's list or made with its boxes. The blocks
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
pub const NOTHING: &str = "mekanism:nothing";
/// The first id of the frequency selector's buttons and boxes (`frequency::id_*`).
pub const FREQ_BASE: i32 = 8200;

pub fn encode(frequency: &Option<frequency::Freq>, energy: u64) -> Vec<u8> {
    let mut out = vec![1];
    frequency::encode(frequency, &mut out);
    out.extend(energy.to_le_bytes());
    out
}

pub fn decode(data: &[u8]) -> Option<(Option<frequency::Freq>, u64)> {
    if data.first() != Some(&1) {
        return None;
    }
    let (f, at) = frequency::decode(data, 1);
    let tail = data.get(at..at + 8)?;
    let mut e = [0u8; 8];
    e.copy_from_slice(tail);
    Some((f, u64::from_le_bytes(e)))
}

/// The share of `total` the block at `index` of `count` holds (they add up to the total).
pub fn share(total: u64, index: u64, count: u64) -> u64 {
    total / count + u64::from(index < total % count)
}

/// The frequency's item after one block's slot went from `before` to `now`: the same item in a bigger stack adds the difference to what the frequency holds (so two blocks that took
/// items in one tick both count), a different item or a smaller stack is what the block now holds.
pub fn merge_item(current: Option<Stack>, before: Option<&Stack>, now: Option<Stack>) -> Option<Stack> {
    match (current, before, now) {
        (Some(cur), Some(b), Some(n)) if cur.item == n.item && cur.components == n.components && b.item == n.item && n.count > b.count => {
            Some(Stack { count: cur.count.saturating_add(n.count - b.count), ..cur })
        }
        (Some(cur), None, Some(n)) if cur.item == n.item && cur.components == n.components => Some(Stack { count: cur.count.saturating_add(n.count), ..cur }),
        (_, _, n) => n,
    }
}

/// What a tank of the frequency takes (`BasicFluidTank` and `BasicChemicalTank`): nothing when full, only its own substance once it holds some, and for the chemical tank only substances that
/// are not radioactive (`ChemicalAttributeValidator.DEFAULT`) while it is empty. An empty list means no restriction.
pub fn tank_filter(tank: &process::Tank, capacity: u64, allowed: Option<&dyn Fn(&str) -> bool>) -> Vec<String> {
    if tank.amount >= capacity {
        vec![NOTHING.to_owned()]
    } else if tank.amount > 0 && !tank.chemical.is_empty() {
        vec![tank.chemical.clone()]
    } else if let Some(ok) = allowed {
        crate::chem_table::CHEMICALS.iter().filter(|c| ok(c)).map(|c| (*c).to_owned()).collect()
    } else {
        Vec::new()
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
        let texts_in = std::mem::take(&mut self.qe_texts);
        let mut out = Vec::new();
        let mut by: HashMap<String, (frequency::Freq, Vec<&BlockEntity>)> = HashMap::new();
        let mut frequency_of: HashMap<(String, i32, i32, i32), Option<frequency::Freq>> = HashMap::new();
        let known: Vec<frequency::Freq> = qes.iter().filter_map(|e| decode(&e.data).and_then(|d| d.0)).collect();
        let mut deleted: Vec<String> = Vec::new();
        for e in qes {
            let key = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
            let mut f = decode(&e.data).and_then(|d| d.0);
            let here = |p: &BlockPos| (p.x, p.y, p.z) == (e.pos.x, e.pos.y, e.pos.z);
            let asked: Vec<(frequency::Action, String, String)> = presses
                .iter()
                .filter(|(d, p, _, _, _)| *d == e.dimension && here(p))
                .filter_map(|(_, _, id, who, name)| frequency::button_action(*id, FREQ_BASE).map(|a| (a, who.clone(), name.clone())))
                .chain(
                    texts_in
                        .iter()
                        .filter(|(d, p, _, _, _, _)| *d == e.dimension && here(p))
                        .filter_map(|(_, _, id, text, who, name)| frequency::text_action(*id, FREQ_BASE, text).map(|a| (a, who.clone(), name.clone()))),
                )
                .collect();
            for (action, who, name) in asked {
                self.freq_viewer.insert(key.clone(), (who.clone(), name.clone()));
                if let Some(next) = frequency::apply(&action, &f, &frequency::listing(&known, &who), &who, &name) {
                    if next.is_none() {
                        // `FrequencyManager.remove`: the frequency is gone for every block that held it.
                        deleted.extend(f.as_ref().map(frequency::Freq::key));
                    }
                    f = next;
                }
            }
            if f.as_ref().is_some_and(|f| deleted.contains(&f.key())) {
                f = None;
            }
            if let Some(freq) = &f {
                by.entry(freq.key()).or_insert_with(|| (freq.clone(), Vec::new())).1.push(*e);
            }
            frequency_of.insert(key, f);
        }
        // The list a screen shows is the viewer's: the player that used it last, or else the owner of its private frequency.
        let viewer_of = |this: &Mekanism, key: &(String, i32, i32, i32), f: &Option<frequency::Freq>| -> frequency::Listing {
            let viewer = this.freq_viewer.get(key).map(|(id, _)| id.clone()).or_else(|| f.as_ref().filter(|f| f.private).map(|f| f.owner.clone())).unwrap_or_default();
            frequency::listing(&known, &viewer)
        };
        // The blocks with no frequency hold nothing.
        for e in qes {
            let key = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
            if frequency_of[&key].is_none() {
                let _ = world::set_energy_capacity(&e.dimension, e.pos.x, e.pos.y, e.pos.z, 1);
                for i in 0..2 {
                    let _ = world::set_tank_filter(&e.dimension, e.pos.x, e.pos.y, e.pos.z, i, &[NOTHING.to_owned()]);
                }
                let listing = viewer_of(self, &key, &None);
                let mut menu = vec![0; 13];
                menu[11] = listing.public.len() as i32;
                menu[12] = listing.private.len() as i32;
                let texts = frequency::texts(&None, &listing);
                if self.menu_texts.get(&key) != Some(&texts) {
                    let _ = world::set_menu_text(&e.dimension, e.pos.x, e.pos.y, e.pos.z, &texts);
                    self.menu_texts.insert(key.clone(), texts);
                }
                out.push(BlockEntityUpdate {
                    dimension: e.dimension.clone(),
                    pos: e.pos,
                    data: encode(&None, 0),
                    items: Some(vec![None]),
                    energy: Some(0),
                    energy_faces: None,
                    tanks: Some(vec![Tank { substance: String::new(), amount: 0 }, Tank { substance: String::new(), amount: 0 }]),
                    tank_faces: None,
                    menu_data: Some(menu),
                    transmitter_faces: None,
                });
            }
        }
        for (fkey, (frequency, mut group)) in by {
            group.sort_by_key(|e| (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z));
            let s = self.entangled.entry(fkey).or_default();
            let held_tank = |e: &BlockEntity, i: usize| e.tanks.get(i).cloned().unwrap_or(Tank { substance: String::new(), amount: 0 });
            let slot = |e: &BlockEntity| stack(e.items.first().unwrap_or(&None));
            if !s.started {
                s.started = true;
                s.energy = group.iter().filter_map(|e| decode(&e.data)).map(|(_, en)| en).max().unwrap_or(0).min(ENERGY_CAPACITY);
                // Each block holds its share of the frequency's fluid and chemical, so what the frequency holds is the sum (the energy is the total, kept in every block).
                let sum = |i: usize| group.iter().map(|e| held_tank(e, i).amount).fold(0u64, u64::saturating_add);
                let kind = |i: usize| group.iter().map(|e| held_tank(e, i)).find(|t| t.amount > 0).map(|t| t.substance).unwrap_or_default();
                if sum(0) > 0 {
                    s.fluid = process::Tank { chemical: kind(0), amount: sum(0).min(FLUID_CAPACITY) };
                }
                if sum(1) > 0 {
                    s.chemical = process::Tank { chemical: kind(1), amount: sum(1).min(CHEMICAL_CAPACITY) };
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
                    // The item: every block whose slot changed is heard. A bigger stack of the item the frequency holds adds what it gained; anything else replaces it.
                    let now = slot(e);
                    if now != gi {
                        s.item = merge_item(s.item.take(), gi.as_ref(), now);
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
                1,
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
                0,
                0,
            ];
            let (energy_total, fluid, chemical, item) = (s.energy, s.fluid.clone(), s.chemical.clone(), s.item.clone());
            for (index, e) in group.iter().enumerate() {
                let key = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
                let (se, sf, sc) = (share(energy_total, index as u64, count), share(fluid.amount, index as u64, count), share(chemical.amount, index as u64, count));
                s.given.insert(key, (se, sf, sc, item.clone()));
                let _ = world::set_energy_capacity(&e.dimension, e.pos.x, e.pos.y, e.pos.z, (se + room / count + 1).min(ENERGY_CAPACITY));
                let _ = world::set_tank_filter(&e.dimension, e.pos.x, e.pos.y, e.pos.z, 0, &tank_filter(&fluid, FLUID_CAPACITY, None));
                let _ = world::set_tank_filter(&e.dimension, e.pos.x, e.pos.y, e.pos.z, 1, &tank_filter(&chemical, CHEMICAL_CAPACITY, Some(&|c: &str| crate::radiation::radioactivity(c) == 0.0)));
                let key2 = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
                let listing = viewer_of_static(&self.freq_viewer, &known, &key2, &Some(frequency.clone()));
                let mut menu = data.clone();
                menu[11] = listing.public.len() as i32;
                menu[12] = listing.private.len() as i32;
                let texts = frequency::texts(&Some(frequency.clone()), &listing);
                if self.menu_texts.get(&key2) != Some(&texts) {
                    let _ = world::set_menu_text(&e.dimension, e.pos.x, e.pos.y, e.pos.z, &texts);
                    self.menu_texts.insert(key2, texts);
                }
                let keep = if tick % 20 == 0 || decode(&e.data).map(|d| d.0) != Some(Some(frequency.clone())) { encode(&Some(frequency.clone()), energy_total) } else { e.data.clone() };
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
                    menu_data: Some(menu),
                    transmitter_faces: None,
                });
            }
        }
        out
    }
}

/// The list a screen shows for a block (the same as the closure inside the tick, for where `self` is borrowed).
fn viewer_of_static(viewers: &HashMap<(String, i32, i32, i32), (String, String)>, known: &[frequency::Freq], key: &(String, i32, i32, i32), f: &Option<frequency::Freq>) -> frequency::Listing {
    let viewer = viewers.get(key).map(|(id, _)| id.clone()).or_else(|| f.as_ref().filter(|f| f.private).map(|f| f.owner.clone())).unwrap_or_default();
    frequency::listing(known, &viewer)
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
    fn a_block_remembers_its_frequency() {
        let f = frequency::Freq { name: "Hub".into(), private: false, owner: "u1".into(), owner_name: "Alex".into() };
        assert_eq!(decode(&encode(&Some(f.clone()), 123_456_789)), Some((Some(f), 123_456_789)));
        assert_eq!(decode(&encode(&None, 5)), Some((None, 5)));
        assert_eq!(decode(&[]), None);
    }

    #[test]
    fn the_items_of_two_blocks_both_count() {
        let st = |n| Some(Stack { item: "minecraft:stone".into(), count: n, components: String::new() });
        // the frequency holds 10; a block that went from 0 to 5 and another from 2 to 6 add 5 and 4
        let a = merge_item(st(10), None, st(5));
        let b = merge_item(a, st(2).as_ref(), st(6));
        assert_eq!(b.map(|s| s.count), Some(19));
        // a different item replaces it
        let other = Some(Stack { item: "minecraft:dirt".into(), count: 1, components: String::new() });
        assert_eq!(merge_item(st(10), None, other.clone()), other);
        // a smaller stack is what the block holds now
        assert_eq!(merge_item(st(10), st(8).as_ref(), st(3)).map(|s| s.count), Some(3));
    }

    #[test]
    fn the_tanks_take_what_the_original_allows() {
        let empty = process::Tank::default();
        let full = process::Tank { chemical: "mekanism:hydrogen".into(), amount: 10 };
        assert_eq!(tank_filter(&full, 10, None), vec![NOTHING.to_owned()]);
        assert_eq!(tank_filter(&process::Tank { amount: 3, ..full.clone() }, 10, None), vec!["mekanism:hydrogen".to_owned()]);
        assert!(tank_filter(&empty, 10, None).is_empty());
        let plain = tank_filter(&empty, 10, Some(&|c: &str| c != "mekanism:plutonium"));
        assert!(plain.contains(&"mekanism:hydrogen".to_owned()));
        assert!(!plain.contains(&"mekanism:plutonium".to_owned()));
    }
}

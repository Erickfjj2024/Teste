//! Upgrades (`TileComponentUpgrade`, `UpgradeUtils`): the upgrade items put in a machine's upgrade slot are installed after a
//! second, up to each type's maximum, and change how fast the machine works, what it uses and what it holds. The pure rules,
//! tested without the game, and then the tick that installs and removes them.
use super::*;

/// The upgrade types, in the order their counts are kept (`mekanism:upgrade` data: `speed`, `energy`, `chemical`, ...).
pub const TYPES: [&str; 8] = ["speed", "energy", "chemical", "muffling", "xp_containment", "anchor", "filter", "stone_generator"];
/// How many of each type fit (`max` of its definition).
pub const MAX: [u8; 8] = [8, 8, 8, 1, 1, 1, 1, 1];
pub const SPEED: usize = 0;
pub const ENERGY: usize = 1;
pub const CHEMICAL: usize = 2;
pub const MUFFLING: usize = 3;
/// `maxUpgradeMultiplier` (`MekanismConfig.general`).
pub const MULTIPLIER: f64 = 10.0;
/// Ticks an upgrade item takes to install (`UPGRADE_TICKS_REQUIRED`).
pub const INSTALL_TICKS: u32 = 20;
/// The window's buttons: remove one of type `k` (6100 + k), remove them all (6110 + k).
pub const REMOVE_ONE: i32 = 6100;
pub const REMOVE_ALL: i32 = 6110;

/// The item of upgrade type `k`.
pub fn item_of(k: usize) -> String {
    format!("mekanism:upgrade_{}", TYPES[k])
}

/// The type an item is, if it is an upgrade.
pub fn type_of(item: &str) -> Option<usize> {
    let name = item.strip_prefix("mekanism:upgrade_")?;
    TYPES.iter().position(|t| *t == name)
}

/// The types a machine takes (`AttributeUpgradeSupport`): speed, energy, muffling and experience for the ones that process
/// things; the three chambers that use chemical by chance take the chemical upgrade too; a generator only muffling.
pub fn supported(def: &Def) -> Vec<usize> {
    match def.kind {
        Kind::Generator => vec![MUFFLING],
        Kind::Tank => Vec::new(),
        _ => {
            let advanced = matches!(
                def.block,
                "mekanism:purification_chamber" | "mekanism:chemical_injection_chamber" | "mekanism:chemical_dissolution_chamber"
            );
            if advanced { vec![SPEED, ENERGY, CHEMICAL, MUFFLING, 4] } else { vec![SPEED, ENERGY, MUFFLING, 4] }
        }
    }
}

/// How many slots a machine has of its own (the upgrade slot comes next, then the one upgrades come out in).
pub fn base_slots(def: &Def) -> usize {
    match (def.kind, def.layout) {
        (Kind::Electric, _) => 2,
        (Kind::Sawmill | Kind::Combiner, _) => 3,
        (Kind::Process, Layout::Oxidizer | Layout::Crystallizer) => 2,
        (Kind::Process, _) => 3,
        (Kind::Continuous, Layout::Rotary) => 5,
        (Kind::Continuous, _) => 3,
        (Kind::Generator, _) => match def.generator {
            Some(GenKind::Solar) => 1,
            _ => 2,
        },
        (Kind::Tank, _) => 0,
    }
}

fn fraction(installed: u8, k: usize) -> f64 {
    f64::from(installed) / f64::from(MAX[k])
}

/// `getTicks`: the ticks a recipe takes with `speed` speed upgrades.
pub fn ticks(def: u32, speed: u8) -> u32 {
    ((f64::from(def) * MULTIPLIER.powf(-fraction(speed, SPEED))) as u32).max(1)
}

/// `getEnergyPerTick`: what a machine uses a tick with its speed and energy upgrades.
pub fn energy_per_tick(def: u64, speed: u8, energy: u8) -> u64 {
    (def as f64 * MULTIPLIER.powf(2.0 * fraction(speed, SPEED) - fraction(energy, ENERGY))).ceil() as u64
}

/// `getMaxEnergy`: what a machine holds with its energy upgrades.
pub fn max_energy(def: u64, energy: u8) -> u64 {
    (def as f64 * MULTIPLIER.powf(fraction(energy, ENERGY))) as u64
}

/// `getGasPerTickMeanMultiplier`: the mean of the chemical a chance-using machine takes a tick.
pub fn gas_mean(speed: u8, chemical: u8) -> f64 {
    MULTIPLIER.powf(2.0 * fraction(speed, SPEED) - fraction(chemical, CHEMICAL))
}

/// The operations a machine of `continuous` does a tick with `speed` speed upgrades (`baselineMaxOperations`).
pub fn operations(speed: u8) -> u32 {
    2u32.pow(u32::from(speed))
}

/// The counts a machine keeps, after its control byte.
pub fn counts_of(def: &Def, e: &BlockEntity) -> [u8; 8] {
    let at = redstone::offset(def.kind) + 1;
    let mut out = [0u8; 8];
    for (k, c) in out.iter_mut().enumerate() {
        *c = e.data.get(at + k).copied().unwrap_or(0).min(MAX[k]);
    }
    out
}

pub fn put_counts(kind: Kind, counts: &[u8; 8], data: &mut Vec<u8>) {
    let at = redstone::offset(kind) + 1;
    if data.len() < at + 8 {
        data.resize(at + 8, 0);
    }
    data[at..at + 8].copy_from_slice(counts);
}

/// The text of an upgrade's effect in the window (`Effect: 1.33x`): the effect is `multiplier ^ (installed / max)` rounded to
/// hundredths, shown as a float is (`1.0`, `1.33`).
pub fn effect_text(installed: u8, k: usize) -> String {
    let effect = (MULTIPLIER.powf(fraction(installed, k)) * 100.0).round() / 100.0;
    let mut s = format!("{effect}");
    if !s.contains('.') {
        s.push_str(".0");
    }
    s
}

/// One tick of installing: `slot` holds what the player put in; `progress` is how many ticks it has been there.
/// Returns the slot after, the counts after and the progress after.
pub fn install_tick(
    slot: Option<&Stack>,
    counts: [u8; 8],
    supported: &[usize],
    progress: u32,
) -> (Option<Stack>, [u8; 8], u32) {
    let Some(stack) = slot else { return (None, counts, 0) };
    let Some(k) = type_of(&stack.item).filter(|k| supported.contains(k)) else { return (Some(stack.clone()), counts, 0) };
    if counts[k] >= MAX[k] {
        return (Some(stack.clone()), counts, 0);
    }
    if progress < INSTALL_TICKS {
        return (Some(stack.clone()), counts, progress + 1);
    }
    let add = (MAX[k] - counts[k]).min(stack.count.min(255) as u8);
    let mut counts = counts;
    counts[k] += add;
    let left = stack.count - u32::from(add);
    ((left > 0).then(|| Stack { count: left, ..stack.clone() }), counts, 0)
}

/// Taking upgrades out: `one` or all of type `k` go into the output slot, as many as fit there (`removeUpgrade`).
pub fn remove(output: Option<&Stack>, counts: [u8; 8], k: usize, all: bool) -> (Option<Stack>, [u8; 8]) {
    let have = counts[k];
    if have == 0 {
        return (output.cloned(), counts);
    }
    let item = item_of(k);
    let (present, same) = match output {
        None => (0, true),
        Some(o) => (o.count, o.item == item && o.components.is_empty()),
    };
    if !same {
        return (output.cloned(), counts);
    }
    let room = 64u32.saturating_sub(present);
    let moved = u32::from(if all { have } else { 1 }).min(room);
    if moved == 0 {
        return (output.cloned(), counts);
    }
    let mut counts = counts;
    counts[k] -= moved as u8;
    (Some(Stack { item, count: present + moved, components: String::new() }), counts)
}

impl Mekanism {
    /// The upgrades of a machine this tick: what is in its upgrade slot is installed, and the window's remove buttons pressed
    /// since the last tick are carried out. `e` is changed in place (its data counts and its two slots); true when something did.
    pub(crate) fn tick_upgrades(&mut self, def: &Def, e: &mut BlockEntity) -> bool {
        let slots = base_slots(def);
        if def.kind == Kind::Tank || e.items.len() < slots + 2 {
            return false;
        }
        let key = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
        let mut counts = counts_of(def, e);
        let before = counts;
        let mut changed = false;
        // The window's buttons.
        if let Some(ids) = self.removals.remove(&key) {
            for id in ids {
                let (k, all) = if id >= REMOVE_ALL { ((id - REMOVE_ALL) as usize, true) } else { ((id - REMOVE_ONE) as usize, false) };
                if k < TYPES.len() {
                    let out = stack(&e.items[slots + 1]);
                    let (out, now) = remove(out.as_ref(), counts, k, all);
                    if now != counts {
                        counts = now;
                        e.items[slots + 1] = item(&out);
                        changed = true;
                    }
                }
            }
        }
        // What is waiting in the upgrade slot.
        let input = stack(&e.items[slots]);
        let progress = self.install.get(&key).copied().unwrap_or(0);
        let (left, now, next) = install_tick(input.as_ref(), counts, &supported(def), progress);
        if now != counts || left != input {
            e.items[slots] = item(&left);
            changed = true;
        }
        counts = now;
        if next != progress {
            self.install.insert(key.clone(), next);
            changed = true;
        }
        if next == 0 {
            self.install.remove(&key);
        }
        if counts != before || changed {
            put_counts(def.kind, &counts, &mut e.data);
        }
        // The energy upgrades change what the machine holds.
        let capacity = match def.kind {
            Kind::Generator | Kind::Tank => 0,
            _ => max_energy(def.usage * STORED_TICKS, counts[ENERGY]),
        };
        if capacity != 0 && self.capacity.get(&key) != Some(&capacity) {
            let _ = world::set_energy_capacity(&e.dimension, e.pos.x, e.pos.y, e.pos.z, capacity);
            self.capacity.insert(key, capacity);
            changed = true;
        }
        changed
    }
}

/// A button of the upgrade window (`removeUpgrade`: one, or all with the second button).
pub fn is_removal(id: i32) -> bool {
    (REMOVE_ONE..REMOVE_ONE + 8).contains(&id) || (REMOVE_ALL..REMOVE_ALL + 8).contains(&id)
}

/// The numbers the window reads: the ticks a recipe takes, the install progress, the eight counts and which of the five types
/// shown in the list (speed, energy, chemical, muffling, experience) are installed, one bit each.
pub fn window_numbers(ticks_required: u32, progress: u32, counts: &[u8; 8]) -> Vec<i32> {
    let mask: i32 = (0..5).filter(|k| counts[*k] > 0).map(|k| 1 << k).sum();
    let mut v = vec![ticks_required as i32, progress as i32];
    v.extend(counts.iter().map(|c| i32::from(*c)));
    v.push(mask);
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    fn upgrade(kind: &str, n: u32) -> Option<Stack> {
        Some(Stack { item: format!("mekanism:upgrade_{kind}"), count: n, components: String::new() })
    }

    #[test]
    fn the_effects_follow_the_formulas() {
        // Eight speed upgrades: a tenth of the ticks, a hundred times the energy.
        assert_eq!(ticks(200, 0), 200);
        assert_eq!(ticks(200, 8), 20);
        assert_eq!(ticks(200, 4), 63);
        assert_eq!(ticks(1, 8), 1);
        assert_eq!(energy_per_tick(8, 0, 0), 8);
        assert_eq!(energy_per_tick(8, 8, 0), 800);
        // Energy upgrades make it cheaper and hold more.
        assert_eq!(energy_per_tick(8, 0, 8), 1);
        assert_eq!(energy_per_tick(8, 8, 8), 80);
        assert_eq!(max_energy(20_000, 0), 20_000);
        assert_eq!(max_energy(20_000, 8), 200_000);
        assert!((gas_mean(8, 8) - 10.0).abs() < 1e-9);
        assert_eq!(operations(3), 8);
    }

    #[test]
    fn the_effect_is_written_as_the_original_does() {
        assert_eq!(effect_text(0, SPEED), "1.0");
        assert_eq!(effect_text(1, SPEED), "1.33");
        assert_eq!(effect_text(8, SPEED), "10.0");
        assert_eq!(effect_text(1, MUFFLING), "10.0");
    }

    #[test]
    fn an_upgrade_installs_after_a_second_up_to_its_maximum() {
        let none = [0u8; 8];
        let sup = [SPEED, ENERGY, MUFFLING, 4];
        let (mut slot, mut counts, mut progress) = (upgrade("speed", 3), none, 0);
        for _ in 0..INSTALL_TICKS {
            (slot, counts, progress) = install_tick(slot.as_ref(), counts, &sup, progress);
            assert_eq!(counts, none);
        }
        (slot, counts, progress) = install_tick(slot.as_ref(), counts, &sup, progress);
        assert_eq!((slot, counts[SPEED], progress), (None, 3, 0));
        // Only what fits: 6 more of the 8 leave the rest.
        let (slot, counts, _) = install_tick(upgrade("speed", 9).as_ref(), counts, &sup, INSTALL_TICKS);
        assert_eq!((slot.map(|s| s.count), counts[SPEED]), (Some(4), 8));
        // A type the machine does not take stays where it is.
        let (slot, counts, progress) = install_tick(upgrade("chemical", 1).as_ref(), none, &sup, 7);
        assert_eq!((slot.map(|s| s.count), counts, progress), (Some(1), none, 0));
        // A thing that is not an upgrade too.
        let junk = Some(Stack { item: "minecraft:dirt".into(), count: 1, components: String::new() });
        assert_eq!(install_tick(junk.as_ref(), none, &sup, 0).0, junk);
    }

    #[test]
    fn upgrades_come_out_one_or_all_into_the_output_slot() {
        let mut counts = [0u8; 8];
        counts[SPEED] = 5;
        let (out, now) = remove(None, counts, SPEED, false);
        assert_eq!((out.as_ref().map(|s| s.count), now[SPEED]), (Some(1), 4));
        let (out, now) = remove(out.as_ref(), now, SPEED, true);
        assert_eq!((out.map(|s| s.count), now[SPEED]), (Some(5), 0));
        // A different item in the output slot blocks it, and a full one takes only what fits.
        let other = Some(Stack { item: item_of(ENERGY), count: 1, components: String::new() });
        assert_eq!(remove(other.as_ref(), counts, SPEED, false).1, counts);
        let nearly = Some(Stack { item: item_of(SPEED), count: 63, components: String::new() });
        let (out, now) = remove(nearly.as_ref(), counts, SPEED, true);
        assert_eq!((out.map(|s| s.count), now[SPEED]), (Some(64), 4));
    }

    #[test]
    fn the_list_shows_what_is_installed() {
        let mut counts = [0u8; 8];
        counts[SPEED] = 2;
        counts[MUFFLING] = 1;
        let v = window_numbers(200, 5, &counts);
        assert_eq!(v.len(), 11);
        assert_eq!((v[0], v[1], v[2], v[5], v[10]), (200, 5, 2, 1, 0b01001));
        assert!(is_removal(6103) && is_removal(6117) && !is_removal(6108));
        assert_eq!(type_of("mekanism:upgrade_speed"), Some(SPEED));
        assert_eq!(type_of("mekanism:crusher"), None);
    }
}

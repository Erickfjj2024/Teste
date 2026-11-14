//! Radiation (`RadiationManager`, `PlayerExposure`, `RadiationEntity`): sources with a strength (Sv/h) at a block, which decay; the dose a
//! player carries, which grows with the radiation where they stand (less the shielding of the hazmat suit they wear) and decays, and the harm
//! a high dose does. The fission reactor leaks waste and a meltdown radiates; the geiger counter and the dosimeter read it.
//!
//! Not here: radioactive substances dumped when a tank is broken, the radiation of items, mobs (only players have a dose), the radiation tab
//! and the sounds; the harm is dealt as magic damage.
use super::*;
use std::collections::HashMap;

/// `BASELINE` and `MIN_MAGNITUDE` (Sv/h), the sources' and the doses' decay a second, the lowest dose that harms.
pub const BASELINE: f64 = 0.000_000_100;
pub const MIN_MAGNITUDE: f64 = 0.000_010;
pub const DECAY: f64 = 0.9995;
pub const MIN_SEVERITY: f64 = 0.1;
/// What a mB of each radioactive substance adds (`chemical_attribute_radioactivity`).
pub fn radioactivity(chemical: &str) -> f64 {
    match chemical {
        "mekanism:nuclear_waste" | "mekanism:spent_nuclear_waste" => 0.01,
        "mekanism:plutonium" => 0.02,
        "mekanism:polonium" => 0.05,
        _ => 0.0,
    }
}

/// The hazmat piece worn in an armor slot of the inventory (36 feet, 37 legs, 38 chest, 39 head): how much it shields.
pub fn shielding(slot: usize, item: &str) -> f64 {
    match (slot, item) {
        (39, "mekanism:hazmat_mask") => 0.25,
        (38, "mekanism:hazmat_gown") => 0.4,
        (37, "mekanism:hazmat_pants") => 0.2,
        (36, "mekanism:hazmat_boots") => 0.15,
        _ => 0.0,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Source {
    pub dimension: String,
    pub pos: [i32; 3],
    pub magnitude: f64,
}

/// `RadiationScale.getScaledDoseSeverity`: where a dose or a level falls between the minimum and 100, on a log scale (0 to 1).
pub fn severity(magnitude: f64) -> f64 {
    if magnitude < MIN_MAGNITUDE {
        return 0.0;
    }
    let (low, high) = (MIN_MAGNITUDE.log10(), 100f64.log10());
    ((magnitude.log10() - low) / (high - low)).clamp(0.0, 1.0)
}

/// The level at a block: the baseline and every source over the distance squared (at least one).
pub fn level_at(sources: &[Source], dimension: &str, pos: [i32; 3]) -> f64 {
    BASELINE
        + sources
            .iter()
            .filter(|s| s.dimension == dimension)
            .map(|s| {
                let d = [i64::from(pos[0] - s.pos[0]), i64::from(pos[1] - s.pos[1]), i64::from(pos[2] - s.pos[2])];
                s.magnitude / (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).max(1) as f64
            })
            .sum::<f64>()
}

/// What a source does to a dose or the effects of one: not used outside.
#[derive(Default)]
pub struct Manager {
    pub sources: Vec<Source>,
    pub doses: HashMap<String, f64>,
}

impl Manager {
    /// `radiate(level, pos, magnitude)`: a source at the block, or the one there made stronger.
    pub fn radiate(&mut self, dimension: &str, pos: [i32; 3], magnitude: f64) {
        if magnitude <= 0.0 {
            return;
        }
        match self.sources.iter_mut().find(|s| s.dimension == dimension && s.pos == pos) {
            Some(s) => s.magnitude += magnitude,
            None => self.sources.push(Source { dimension: dimension.to_owned(), pos, magnitude }),
        }
    }

    /// Once a second: the sources decay, and the weakest vanish.
    pub fn decay_sources(&mut self) {
        for s in &mut self.sources {
            s.magnitude *= DECAY;
        }
        self.sources.retain(|s| s.magnitude >= MIN_MAGNITUDE);
    }

    /// A player's second: the dose takes what the place radiates (in Sv/s, less the shielding), then decays.
    pub fn expose(&mut self, player: &str, dimension: &str, pos: [i32; 3], resistance: f64) {
        let level = level_at(&self.sources, dimension, pos);
        let dose = self.doses.entry(player.to_owned()).or_insert(BASELINE);
        if level > BASELINE {
            *dose += level / 3600.0 * (1.0 - resistance.min(1.0));
        }
        *dose = (*dose * DECAY).max(BASELINE);
    }
}

/// Whether a dose hurts this tick, and how much: `roll_a` and `roll_b` are random numbers in 0..1 (the chance and the coin).
pub fn harm(dose: f64, roll_a: f64, roll_b: f64) -> Option<f32> {
    let severity = severity(dose);
    if severity <= BASELINE {
        return None;
    }
    let chance = MIN_SEVERITY + roll_a * (1.0 - MIN_SEVERITY);
    if severity > chance && roll_b < 0.5 {
        Some((dose.ln_1p() as f32).max(1.0))
    } else {
        None
    }
}

impl Mekanism {
    /// Radiation of the whole server, a tick.
    pub(crate) fn run_radiation(&mut self, tick: u64) {
        if tick % 20 == 0 {
            self.radiation.decay_sources();
        }
        for p in players::online() {
            let Some(inv) = player_items::get_inventory(&p.id) else { continue };
            // Its second, spread over the ticks by the player's id.
            let phase = p.id.bytes().fold(0u64, |a, b| a.wrapping_mul(31).wrapping_add(u64::from(b))) % 20;
            if tick % 20 == phase {
                let resistance: f64 = (36..40).map(|i| inv.get(i).and_then(|s| s.as_ref()).map_or(0.0, |s| shielding(i, &s.item))).sum();
                self.radiation.expose(&p.id, &p.dimension, [p.x.floor() as i32, p.y.floor() as i32, p.z.floor() as i32], resistance);
            }
            let dose = self.radiation.doses.get(&p.id).copied().unwrap_or(BASELINE);
            if let Some(amount) = harm(dose, roll(tick, phase as i32 + 1), roll(tick, phase as i32 + 77)) {
                let _ = player_items::hurt(&p.id, "minecraft:magic", amount);
            }
        }
    }

    /// The geiger counter and the dosimeter, used: the radiation where the player stands, and the dose they carry.
    pub(crate) fn read_radiation(&mut self, player: &Player, item: &str) -> bool {
        let text = match item {
            "mekanism:geiger_counter" => {
                let level = level_at(&self.radiation.sources, &player.dimension, [player.x.floor() as i32, player.y.floor() as i32, player.z.floor() as i32]);
                format!("Radiation: {} Sv/h", format_sv(level))
            }
            "mekanism:dosimeter" => format!("Dose: {} Sv", format_sv(self.radiation.doses.get(&player.id).copied().unwrap_or(BASELINE))),
            _ => return false,
        };
        let _ = players::tell(&player.id, &text);
        true
    }
}

fn roll(tick: u64, salt: i32) -> f64 {
    fission::roll(tick, salt)
}

/// A dose or a level with the unit prefix of the original (`UnitDisplayUtils.RadiationUnit`).
pub fn format_sv(value: f64) -> String {
    let (scaled, prefix) = if value < 1e-6 {
        (value * 1e9, "n")
    } else if value < 1e-3 {
        (value * 1e6, "µ")
    } else if value < 1.0 {
        (value * 1e3, "m")
    } else {
        (value, "")
    };
    format!("{scaled:.2} {prefix}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_source_is_weaker_far_away_and_decays() {
        let mut m = Manager::default();
        m.radiate("minecraft:overworld", [0, 64, 0], 100.0);
        m.radiate("minecraft:overworld", [0, 64, 0], 50.0);
        assert_eq!(m.sources.len(), 1);
        let near = level_at(&m.sources, "minecraft:overworld", [0, 64, 0]);
        let far = level_at(&m.sources, "minecraft:overworld", [10, 64, 0]);
        assert!((near - (150.0 + BASELINE)).abs() < 1e-9 && (far - (1.5 + BASELINE)).abs() < 1e-9);
        assert_eq!(level_at(&m.sources, "minecraft:the_nether", [0, 64, 0]), BASELINE);
        for _ in 0..40_000 {
            m.decay_sources();
        }
        assert!(m.sources.is_empty());
    }

    #[test]
    fn the_dose_grows_where_it_is_hot_and_a_suit_shields() {
        let mut bare = Manager::default();
        bare.radiate("d", [0, 0, 0], 3600.0);
        let mut suited = Manager::default();
        suited.radiate("d", [0, 0, 0], 3600.0);
        bare.expose("p", "d", [0, 0, 0], 0.0);
        suited.expose("p", "d", [0, 0, 0], 1.0);
        assert!(bare.doses["p"] > 0.99 && suited.doses["p"] < 1e-6, "{:?} {:?}", bare.doses, suited.doses);
        // Away from every source it fades.
        let mut m = Manager::default();
        m.doses.insert("p".into(), 1.0);
        m.expose("p", "d", [0, 0, 0], 0.0);
        assert!(m.doses["p"] < 1.0);
    }

    #[test]
    fn only_a_big_enough_dose_hurts() {
        assert_eq!(harm(BASELINE, 0.0, 0.0), None);
        assert_eq!(severity(MIN_MAGNITUDE / 10.0), 0.0);
        assert!((severity(100.0) - 1.0).abs() < 1e-9);
        // A high dose with a low chance roll and a heads coin hurts, at least one.
        assert!(harm(50.0, 0.0, 0.1).is_some_and(|d| d >= 1.0));
        assert_eq!(harm(50.0, 0.0, 0.9), None);
        assert_eq!(harm(1e-5, 0.99, 0.1), None);
    }

    #[test]
    fn the_hazmat_suit_adds_up() {
        let total: f64 = [(39, "mekanism:hazmat_mask"), (38, "mekanism:hazmat_gown"), (37, "mekanism:hazmat_pants"), (36, "mekanism:hazmat_boots")].iter().map(|(s, i)| shielding(*s, i)).sum();
        assert!((total - 1.0).abs() < 1e-9);
        assert_eq!(shielding(38, "mekanism:hazmat_mask"), 0.0);
    }

    #[test]
    fn doses_read_with_the_original_prefixes() {
        assert_eq!(format_sv(2.5e-8), "25.00 n");
        assert_eq!(format_sv(5e-5), "50.00 µ");
        assert_eq!(format_sv(0.25), "250.00 m");
        assert_eq!(format_sv(3.0), "3.00 ");
        assert_eq!(radioactivity("mekanism:polonium"), 0.05);
    }

    #[test]
    fn radiation_constants_match_java_config() {
        assert_eq!(BASELINE, 0.000_000_100);
        assert_eq!(MIN_MAGNITUDE, 0.000_010);
        assert_eq!(DECAY, 0.9995);
        assert_eq!(MIN_SEVERITY, 0.1);
        assert_eq!(radioactivity("mekanism:nuclear_waste"), 0.01);
        assert_eq!(radioactivity("mekanism:spent_nuclear_waste"), 0.01);
        assert_eq!(radioactivity("mekanism:plutonium"), 0.02);
        assert_eq!(radioactivity("mekanism:polonium"), 0.05);
    }
}

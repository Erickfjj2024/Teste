//! The storage tanks (`TileEntityFluidTank`, `TileEntityChemicalTank`): their buckets, the creative tank that never empties
//! and the dumping modes of a chemical tank. The pure rules, tested without the game.
use crate::machine::Stack;
use crate::process::Tank;

/// A bucket's worth, in millibuckets.
pub const BUCKET: u64 = 1000;

/// `ContainerEditMode`: what a fluid tank lets be done with it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EditMode {
    #[default]
    Both = 0,
    Fill = 1,
    Empty = 2,
}

impl EditMode {
    pub fn from_byte(b: u8) -> EditMode {
        match b {
            1 => EditMode::Fill,
            2 => EditMode::Empty,
            _ => EditMode::Both,
        }
    }
    pub fn next(self) -> EditMode {
        EditMode::from_byte((self as u8 + 1) % 3)
    }
    pub fn previous(self) -> EditMode {
        EditMode::from_byte((self as u8 + 2) % 3)
    }
    pub fn label(self) -> &'static str {
        match self {
            EditMode::Both => "Fill and empty",
            EditMode::Fill => "Fill only",
            EditMode::Empty => "Empty only",
        }
    }
}

/// Chemical tank tier capacities (mB) and transfer rates (mB/t): Basic, Advanced, Elite, Ultimate, Creative.
pub const CHEMICAL_TANK_CAPACITIES: [u64; 5] = [64_000, 256_000, 1_024_000, 8_192_000, 1_000_000_000_000];
pub const CHEMICAL_TANK_RATES: [u64; 5] = [1_000, 16_000, 128_000, 512_000, 1_000_000_000];

/// Fluid tank tier capacities (mB) and transfer rates (mB/t): Basic, Advanced, Elite, Ultimate, Creative.
pub const FLUID_TANK_CAPACITIES: [u64; 5] = [32_000, 64_000, 128_000, 256_000, 1_000_000_000_000];
pub const FLUID_TANK_RATES: [u64; 5] = [1_000, 4_000, 16_000, 64_000, 1_000_000_000];

/// `GasMode`: what a tank does with its chemical besides holding it.
/// In Java (`TileEntityChemicalTank.GasMode`), the enum ordinals and cycling order are:
/// IDLE (0) -> DUMPING_EXCESS (1) -> DUMPING (2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DumpMode {
    Idle = 0,
    Excess = 1,
    Dumping = 2,
}

impl DumpMode {
    pub fn from_byte(b: u8) -> DumpMode {
        match b {
            1 => DumpMode::Excess,
            2 => DumpMode::Dumping,
            _ => DumpMode::Idle,
        }
    }
    pub fn next(self) -> DumpMode {
        DumpMode::from_byte((self as u8 + 1) % 3)
    }
    pub fn previous(self) -> DumpMode {
        DumpMode::from_byte((self as u8 + 2) % 3)
    }
    pub fn label(self) -> &'static str {
        match self {
            DumpMode::Idle => "Idle",
            DumpMode::Excess => "Dumping excess",
            DumpMode::Dumping => "Dumping",
        }
    }
}

/// `ChemicalUtils.dump`: a tank in dumping mode loses `dumping` a tick; one dumping its excess loses what is over 85% of its
/// capacity, at most `excess_rate` a tick.
pub fn dump(tank: &mut Tank, cap: u64, mode: DumpMode, dumping: u64, excess_rate: u64) {
    if mode == DumpMode::Idle || tank.amount == 0 {
        return;
    }
    let to_dump = match mode {
        DumpMode::Dumping => dumping,
        DumpMode::Excess => {
            let keep = (cap as f64 * 0.85) as u64;
            if keep < tank.amount { (tank.amount - keep).min(excess_rate) } else { 0 }
        }
        DumpMode::Idle => return,
    };
    tank.amount = tank.amount.saturating_sub(to_dump);
    if tank.amount == 0 {
        tank.chemical.clear();
    }
}

/// A creative tank never empties: while it holds something it is full of it again each tick.
pub fn creative(tank: &mut Tank, cap: u64) {
    if tank.amount > 0 {
        tank.amount = cap;
    }
}

/// The full buckets and the fluids they hold: vanilla's two and Mekanism's eighteen (its liquid gases, brine, lithium...).
const BUCKETS: [(&str, &str); 20] = [
    ("minecraft:water_bucket", "minecraft:water"),
    ("minecraft:lava_bucket", "minecraft:lava"),
    ("mekanism:brine_bucket", "mekanism:brine"),
    ("mekanism:chlorine_bucket", "mekanism:chlorine"),
    ("mekanism:ethene_bucket", "mekanism:ethene"),
    ("mekanism:heavy_water_bucket", "mekanism:heavy_water"),
    ("mekanism:hydrofluoric_acid_bucket", "mekanism:hydrofluoric_acid"),
    ("mekanism:hydrogen_bucket", "mekanism:hydrogen"),
    ("mekanism:hydrogen_chloride_bucket", "mekanism:hydrogen_chloride"),
    ("mekanism:lithium_bucket", "mekanism:lithium"),
    ("mekanism:nutritional_paste_bucket", "mekanism:nutritional_paste"),
    ("mekanism:oxygen_bucket", "mekanism:oxygen"),
    ("mekanism:sodium_bucket", "mekanism:sodium"),
    ("mekanism:steam_bucket", "mekanism:steam"),
    ("mekanism:sulfur_dioxide_bucket", "mekanism:sulfur_dioxide"),
    ("mekanism:sulfur_trioxide_bucket", "mekanism:sulfur_trioxide"),
    ("mekanism:sulfuric_acid_bucket", "mekanism:sulfuric_acid"),
    ("mekanism:superheated_sodium_bucket", "mekanism:superheated_sodium"),
    ("mekanism:uranium_hexafluoride_bucket", "mekanism:uranium_hexafluoride"),
    ("mekanism:uranium_oxide_bucket", "mekanism:uranium_oxide"),
];

/// The fluid a full bucket holds.
pub fn fluid_of(bucket: &str) -> Option<&'static str> {
    BUCKETS.iter().find(|(b, _)| *b == bucket).map(|(_, f)| *f)
}

/// The full bucket of a fluid.
pub fn bucket_of(fluid: &str) -> Option<&'static str> {
    BUCKETS.iter().find(|(_, f)| *f == fluid).map(|(b, _)| *b)
}

fn stack(item: &str, count: u32) -> Stack {
    Stack { item: item.into(), count, components: String::new() }
}

/// `FluidInventorySlot.handleTank`: a full bucket in the input slot empties into the tank when it fits (the empty bucket goes
/// to the output slot); an empty bucket there is filled from the tank when it holds a bucketful (the full one goes to the
/// output slot, which must be empty). The edit mode says which of the two is allowed.
pub fn buckets(input: &mut Option<Stack>, output: &mut Option<Stack>, tank: &mut Tank, cap: u64, mode: EditMode) -> bool {
    let Some(item) = input.clone() else { return false };
    if let Some(fluid) = fluid_of(&item.item) {
        let fits = (tank.amount == 0 || tank.chemical == fluid) && tank.amount + BUCKET <= cap;
        let out_ok = match output {
            None => true,
            Some(o) => o.item == "minecraft:bucket" && o.components.is_empty() && o.count < 16,
        };
        if mode != EditMode::Empty && fits && out_ok {
            tank.chemical = fluid.to_owned();
            tank.amount += BUCKET;
            *input = (item.count > 1).then(|| Stack { count: item.count - 1, ..item });
            *output = Some(match output.take() {
                Some(o) => Stack { count: o.count + 1, ..o },
                None => stack("minecraft:bucket", 1),
            });
            return true;
        }
        return false;
    }
    if item.item == "minecraft:bucket"
        && mode != EditMode::Fill
        && tank.amount >= BUCKET
        && output.is_none()
        && let Some(full) = bucket_of(&tank.chemical)
    {
        tank.amount -= BUCKET;
        if tank.amount == 0 {
            tank.chemical.clear();
        }
        *input = (item.count > 1).then(|| Stack { count: item.count - 1, ..item });
        *output = Some(stack(full, 1));
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(c: &str, n: u64) -> Tank {
        Tank { chemical: c.into(), amount: n }
    }
    fn st(i: &str, n: u32) -> Option<Stack> {
        Some(stack(i, n))
    }

    #[test]
    fn a_full_bucket_empties_into_the_tank_and_leaves_the_empty_one() {
        let (mut i, mut o, mut tank) = (st("minecraft:water_bucket", 1), None, Tank::default());
        assert!(buckets(&mut i, &mut o, &mut tank, 32_000, EditMode::Both));
        assert_eq!((i, o, tank), (None, st("minecraft:bucket", 1), t("minecraft:water", 1000)));
        // Lava does not mix with water, and the empty-only mode takes nothing in.
        let (mut i, mut o, mut tank) = (st("minecraft:lava_bucket", 1), None, t("minecraft:water", 1000));
        assert!(!buckets(&mut i, &mut o, &mut tank, 32_000, EditMode::Both));
        let (mut i, mut o, mut tank) = (st("minecraft:water_bucket", 1), None, Tank::default());
        assert!(!buckets(&mut i, &mut o, &mut tank, 32_000, EditMode::Empty));
        // A full tank has no room.
        let (mut i, mut o, mut tank) = (st("minecraft:water_bucket", 1), None, t("minecraft:water", 31_500));
        assert!(!buckets(&mut i, &mut o, &mut tank, 32_000, EditMode::Both));
    }

    #[test]
    fn an_empty_bucket_is_filled_from_the_tank() {
        let (mut i, mut o, mut tank) = (st("minecraft:bucket", 2), None, t("minecraft:lava", 1500));
        assert!(buckets(&mut i, &mut o, &mut tank, 32_000, EditMode::Both));
        assert_eq!((i, o, tank), (st("minecraft:bucket", 1), st("minecraft:lava_bucket", 1), t("minecraft:lava", 500)));
        // Fill-only mode gives nothing, and a tank with less than a bucketful too.
        let (mut i, mut o, mut tank) = (st("minecraft:bucket", 1), None, t("minecraft:water", 1500));
        assert!(!buckets(&mut i, &mut o, &mut tank, 32_000, EditMode::Fill));
        let (mut i, mut o, mut tank) = (st("minecraft:bucket", 1), None, t("minecraft:water", 999));
        assert!(!buckets(&mut i, &mut o, &mut tank, 32_000, EditMode::Both));
    }

    #[test]
    fn mekanism_buckets_hold_its_fluids() {
        assert_eq!(fluid_of("mekanism:brine_bucket"), Some("mekanism:brine"));
        assert_eq!(bucket_of("mekanism:sulfuric_acid"), Some("mekanism:sulfuric_acid_bucket"));
        let (mut i, mut o, mut tank) = (st("mekanism:hydrogen_bucket", 1), None, Tank::default());
        assert!(buckets(&mut i, &mut o, &mut tank, 32_000, EditMode::Both));
        assert_eq!((i, o, tank), (None, st("minecraft:bucket", 1), t("mekanism:hydrogen", 1000)));
    }

    #[test]
    fn modes_go_round() {
        assert_eq!(EditMode::Both.next(), EditMode::Fill);
        assert_eq!(EditMode::Both.previous(), EditMode::Empty);
        assert_eq!(DumpMode::Idle.next(), DumpMode::Excess);
        assert_eq!(DumpMode::Excess.next(), DumpMode::Dumping);
        assert_eq!(DumpMode::Dumping.next(), DumpMode::Idle);
        assert_eq!(DumpMode::Idle.previous(), DumpMode::Dumping);
    }

    #[test]
    fn dumping_loses_a_little_and_the_excess_only_what_is_over_85_percent() {
        let mut tank = t("mekanism:oxygen", 1000);
        dump(&mut tank, 64_000, DumpMode::Dumping, 160, 1000);
        assert_eq!(tank.amount, 840);
        dump(&mut tank, 64_000, DumpMode::Excess, 160, 1000);
        assert_eq!(tank.amount, 840, "under 85%");
        let mut over = t("mekanism:oxygen", 60_000);
        dump(&mut over, 64_000, DumpMode::Excess, 160, 1000);
        assert_eq!(over.amount, 59_000, "at most the rate");
        let mut idle = t("mekanism:oxygen", 5);
        dump(&mut idle, 64_000, DumpMode::Idle, 160, 1000);
        assert_eq!(idle.amount, 5);
        dump(&mut idle, 64_000, DumpMode::Dumping, 160, 1000);
        assert_eq!(idle, Tank::default());
    }

    #[test]
    fn a_creative_tank_refills_what_it_holds() {
        let mut tank = t("mekanism:oxygen", 3);
        creative(&mut tank, 10_000);
        assert_eq!(tank.amount, 10_000);
        let mut empty = Tank::default();
        creative(&mut empty, 10_000);
        assert_eq!(empty, Tank::default());
    }
}

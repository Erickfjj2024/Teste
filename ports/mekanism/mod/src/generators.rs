//! The generators of MekanismGenerators that the port has (`TileEntityBioGenerator`, `TileEntityHeatGenerator`,
//! `TileEntitySolarGenerator`, `TileEntityGasGenerator`): the pure rules, tested without the game, and then each one's
//! tick. The numbers are the 26.3 configuration's defaults.
use super::*;
use crate::plus::held;
use crate::tank::EditMode;

/// What a generator is.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GenKind {
    Bio,
    Heat,
    Solar,
    Gas,
}

/// The bioethanol a bio-generator burns, and where its tank comes from.
pub const BIOETHANOL: &str = "mekanismgenerators:bioethanol";
/// `bioGeneration`, `bioTankCapacity` and `bioFuelPerItem`.
pub const BIO_GENERATION: u64 = 25;
pub const BIO_TANK: u64 = 200;
pub const BIO_PER_ITEM: u64 = 64;
/// `heatGeneration` (heat added a tick from the lava), `heatGenerationFluidRate`, `heatGenerationLava` (a side of lava),
/// `heatGenerationNether` and `heatTankCapacity`.
pub const HEAT_GENERATION: f64 = 100.0;
pub const HEAT_FLUID_RATE: u64 = 100;
pub const HEAT_LAVA_SIDE: f64 = 7.0;
pub const HEAT_NETHER: f64 = 10.0;
pub const HEAT_TANK: u64 = 1_000;
/// `solarGeneration`.
pub const SOLAR_GENERATION: f64 = 2.0;
/// `gbgTankCapacity`.
pub const GAS_TANK: u64 = 1_000;
/// What the energy containers hold (`GeneratorsStorageConfig`: 12, 30 and 2 seconds of ticks, and 1000 times the hydrogen's
/// energy density for the gas-burning one).
pub const HEAT_ENERGY: u64 = 240;
pub const BIO_ENERGY: u64 = 600;
pub const SOLAR_ENERGY: u64 = 40;
pub const GAS_ENERGY: u64 = 2_000;

pub const HEAT_CAPACITY: f64 = 10.0;
pub const AMBIENT: f64 = 300.0;

/// `HeatAPI.getAmbientTemp`: 300 K at a biome temperature of 0.8, 25 K for each degree above or below (the biome's clamped to -5..5).
pub fn ambient_of(biome_temperature: f32) -> f64 {
    AMBIENT + 25.0 * (f64::from(biome_temperature).clamp(-5.0, 5.0) - 0.8)
}
const INVERSE_CONDUCTION: f64 = 5.0;
const INVERSE_INSULATION: f64 = 100.0;
const AIR_INVERSE: f64 = 10_000.0;
const THERMAL_EFFICIENCY: f64 = 0.5;
/// The most a heat generator makes from the heat in a tick: its three sources at their best.
pub const HEAT_MAX_PRODUCTION: u64 = 159;

/// What the heat generator does in a tick (`TileEntityHeatGenerator.onUpdateServer` and `simulate`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeatOut {
    pub heat: f64,
    /// The energy the heat made.
    pub energy: u64,
    /// The heat the generator lost to the air (a temperature a tick).
    pub environment: f64,
}

/// `boost` is the heat the lava beside it and the nether add; `burning` that it took lava from its tank this tick.
/// `ambient` is the air's temperature there (`ambient_of`).
pub fn heat_tick(heat: f64, boost: f64, burning: bool, ambient: f64) -> HeatOut {
    let mut heat = (heat + boost).max(0.0);
    if burning {
        heat += HEAT_GENERATION;
    }
    let temp = heat / HEAT_CAPACITY;
    let carnot = 1.0 - temp.min(ambient) / temp.max(ambient);
    let lost = THERMAL_EFFICIENCY * (temp - ambient);
    heat = (heat - lost).max(0.0);
    let energy = ((lost.abs() * carnot) as u64).min(HEAT_MAX_PRODUCTION);
    // The air takes heat on each of the six sides; the bottom is not insulated.
    let mut environment = 0.0;
    for insulation in [INVERSE_INSULATION, INVERSE_INSULATION, INVERSE_INSULATION, INVERSE_INSULATION, INVERSE_INSULATION, 0.0] {
        let t = heat / HEAT_CAPACITY;
        let per_side = (t - ambient) / (AIR_INVERSE + insulation + INVERSE_CONDUCTION);
        heat = (heat - per_side * HEAT_CAPACITY).max(0.0);
        if per_side > 0.0 {
            environment += per_side;
        }
    }
    HeatOut { heat, energy, environment }
}

/// The lava a heat generator takes from a fuel item: the bucket's volume by the item's share of a lava bucket's burn time.
pub fn lava_of(item: &str) -> Option<u64> {
    // Ticks in a furnace (`FuelValues`); a lava bucket burns 20000.
    let burn: u64 = match item {
        "minecraft:coal" | "minecraft:charcoal" => 1_600,
        "minecraft:coal_block" => 16_000,
        "minecraft:blaze_rod" => 2_400,
        "minecraft:dried_kelp_block" => 4_000,
        "minecraft:stick" | "minecraft:bamboo" => 100,
        "minecraft:wooden_sword" | "minecraft:wooden_pickaxe" | "minecraft:wooden_axe" | "minecraft:wooden_shovel" | "minecraft:wooden_hoe" => 200,
        "minecraft:bowl" | "minecraft:crafting_table" | "minecraft:chest" | "minecraft:bookshelf" => 300,
        i if i.ends_with("_planks") || i.ends_with("_log") || i.ends_with("_wood") || i.ends_with("_stem") || i.ends_with("_hyphae") => 300,
        _ => return None,
    };
    Some(1_000 * burn / 20_000)
}

/// The bioethanol a bio fuel item makes: `bioFuelPerItem` an item, nine times that for a block.
pub fn bioethanol_of(item: &str) -> Option<u64> {
    match item {
        "mekanism:bio_fuel" => Some(BIO_PER_ITEM),
        "mekanism:block_bio_fuel" => Some(9 * BIO_PER_ITEM),
        _ => None,
    }
}

/// `ChemicalFuel`: the energy a mB of a gas gives and the most a tick burns (`chemical_attribute_fuel` data map).
pub fn fuel_of(chemical: &str) -> Option<(u64, u64)> {
    match chemical {
        "mekanism:hydrogen" => Some((2, 10)),
        "mekanism:ethene" => Some((8, 10)),
        _ => None,
    }
}

/// What the gas-burning generator burns of a full-or-not tank this tick, and what it makes: (mB used, energy). `room` is
/// what its energy container can still take.
pub fn gas_tick(chemical: &str, amount: u64, capacity: u64, room: u64) -> (u64, u64) {
    let Some((density, max_burn)) = fuel_of(chemical) else { return (0, 0) };
    if amount == 0 {
        return (0, 0);
    }
    let fullness = amount as f64 / capacity as f64;
    let burn = ((max_burn as f64 * fullness).ceil() as u64).min(amount);
    let max_energy = density * burn;
    if max_energy == 0 {
        return (0, 0);
    }
    let inserted = max_energy.min(room);
    if inserted == 0 {
        return (0, 0);
    }
    let used = inserted.div_ceil(density);
    (used, inserted)
}

/// The sky: the share of the sun's light the day and the weather leave (`getSunBrightness`, as vanilla works its sky out) and
/// how dark the sky is (`getSkyDarken`, 0 to 11).
pub fn sky(day_time: u64, rain: f32, thunder: f32) -> (f32, u32) {
    let d = (day_time as f64 / 24_000.0 - 0.25).rem_euclid(1.0);
    let e = 0.5 - (d * std::f64::consts::PI).cos() / 2.0;
    let angle = (d * 2.0 + e) / 3.0;
    let mut f = 1.0 - ((angle * std::f64::consts::TAU).cos() * 2.0 + 0.2);
    f = 1.0 - f.clamp(0.0, 1.0);
    f *= 1.0 - f64::from(rain) * 5.0 / 16.0;
    f *= 1.0 - f64::from(thunder) * 5.0 / 16.0;
    (f as f32, ((1.0 - f) * 11.0) as u32)
}

/// A solar generator's share of the sun in a biome (`SolarCheck.peakMultiplier`).
pub fn peak(temperature: f32, downfall: f32, precipitation: bool) -> f32 {
    let temp_eff = 0.3 * (0.8 - temperature);
    let humidity_eff = if precipitation { -0.3 * downfall } else { 0.0 };
    1.0 + temp_eff + humidity_eff
}

/// What a solar generator makes of its sun (`getProduction`): its generation by the sky's brightness and the biome's share,
/// cut to a whole number.
pub fn solar_production(brightness: f32, peak: f32) -> u64 {
    (SOLAR_GENERATION * f64::from(brightness * peak)).max(0.0) as u64
}

/// The faces of a generator each item slot is reached from (`forSide` of its tile): the fuel slot (0) by every side but the
/// right, the energy slot (1) by the right.
pub fn item_faces(kind: GenKind, facing: &str) -> Vec<Vec<u32>> {
    let mut faces = vec![Vec::new(); 6];
    if kind == GenKind::Solar {
        return vec![vec![0]; 6];
    }
    for side in 0..6 {
        faces[config::face_of(facing, side)] = if side == config::RIGHT { vec![1] } else { vec![0] };
    }
    faces
}

/// What a face does with a generator's tank: it takes in on every side but the front (`forSide` of its tile).
pub fn tank_faces(facing: &str) -> Vec<FlowSide> {
    let mut faces = vec![FlowSide::Input; 6];
    faces[config::face_of(facing, config::FRONT)] = FlowSide::None;
    faces
}

/// Where a generator gives its energy: the front, the bottom for the solar one.
pub fn energy_faces(kind: GenKind, facing: &str) -> Vec<FlowSide> {
    let mut faces = vec![FlowSide::None; 6];
    match kind {
        GenKind::Solar => faces[config::DOWN] = FlowSide::Output,
        _ => faces[config::face_of(facing, config::FRONT)] = FlowSide::Output,
    }
    faces
}

/// How the generators tick: the world-dependent half (the sun, the lava beside, the items in the slots).
impl Mekanism {
    /// Whether nothing but air is above `pos` (`level.canSeeSky`: the world-surface heightmap is under the position).
    fn sky_clear(&mut self, e: &BlockEntity, tick: u64) -> bool {
        let key = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
        if let Some((at, clear)) = self.suns.get(&key)
            && tick.saturating_sub(*at) < 10
        {
            return *clear;
        }
        let mut clear = true;
        for y in e.pos.y + 1..320 {
            match world::get_block(&e.dimension, e.pos.x, y, e.pos.z) {
                Some(b) if !matches!(b.split('[').next().unwrap_or(""), "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air") => {
                    clear = false;
                    break;
                }
                Some(_) => {}
                // An unloaded chunk above: nothing known, taken as open.
                None => break,
            }
        }
        self.suns.insert(key, (tick, clear));
        clear
    }

    /// A tick of a generator.
    pub(crate) fn run_generator(&mut self, def: &Def, e: &BlockEntity, tick: u64) -> Option<BlockEntityUpdate> {
        let kind = def.generator?;
        let can = self.can;
        let key = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
        let mut facing = sides::property(&e.state, "facing").unwrap_or("north").to_owned();
        let placed = e.data.get(8) == Some(&1);
        let mut heat = e.data.get(..8).and_then(|b| b.try_into().ok()).map_or(AMBIENT * HEAT_CAPACITY, f64::from_le_bytes);
        if !placed && kind != GenKind::Solar {
            // A new generator looks at the nearest player, as one placed in front of them does.
            let front = cube::facing_name(front_for(e));
            if front != facing {
                let _ = world::set_block_property(&e.dimension, e.pos.x, e.pos.y, e.pos.z, "facing", front);
                facing = front.to_owned();
            }
        }
        let capacity = match kind {
            GenKind::Bio => BIO_ENERGY,
            GenKind::Heat => HEAT_ENERGY,
            GenKind::Solar => SOLAR_ENERGY,
            GenKind::Gas => GAS_ENERGY,
        };
        let tank_cap = match kind {
            GenKind::Bio => BIO_TANK,
            GenKind::Heat => HEAT_TANK,
            GenKind::Gas => GAS_TANK,
            GenKind::Solar => 0,
        };
        let mut energy = e.energy.min(capacity);
        let mut tank = held(e, 0);
        let mut items: Vec<Option<Stack>> = (0..e.items.len()).map(|i| stack(e.items.get(i).unwrap_or(&None))).collect();
        let mut items_changed = false;
        let (mut active, mut producing, mut extra) = (false, 0u64, 0i32);
        let mut losses = (0.0, 0.0);
        let room = |energy: u64| capacity.saturating_sub(energy);
        // Takes one of the fuel slot's items.
        let consume = |items: &mut Vec<Option<Stack>>| {
            if let Some(Some(s)) = items.get_mut(0) {
                if s.count > 1 {
                    s.count -= 1;
                } else {
                    items[0] = None;
                }
            }
        };
        match kind {
            GenKind::Bio => {
                // `fillOrBurn`: a bio fuel item turns into bioethanol when the tank has room for all it makes.
                let fuel = items.first().and_then(Option::as_ref).and_then(|s| bioethanol_of(&s.item));
                if let Some(mb) = fuel
                    && (tank.amount == 0 || tank.chemical == BIOETHANOL)
                    && mb <= BIO_TANK - tank.amount
                {
                    consume(&mut items);
                    tank.chemical = BIOETHANOL.into();
                    tank.amount += mb;
                    items_changed = true;
                }
                if can && tank.amount > 0 && room(energy) >= BIO_GENERATION {
                    energy += BIO_GENERATION;
                    tank.amount -= 1;
                    active = true;
                    producing = BIO_GENERATION;
                }
            }
            GenKind::Heat => {
                let mut i0 = items.first().cloned().flatten();
                let mut sink = None;
                if tank::buckets(&mut i0, &mut sink, &mut tank, HEAT_TANK, EditMode::Fill) {
                    items[0] = sink;
                    items_changed = true;
                } else if let Some(lava) = i0.as_ref().and_then(|s| lava_of(&s.item))
                    && (tank.amount == 0 || tank.chemical == "minecraft:lava")
                    && lava <= HEAT_TANK - tank.amount
                {
                    consume(&mut items);
                    tank.chemical = "minecraft:lava".into();
                    tank.amount += lava;
                    items_changed = true;
                }
                // The lava beside it (and the nether) heat it even with no fuel.
                let mut boost = 0.0;
                for (dx, dy, dz) in [(1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)] {
                    let side = world::get_block(&e.dimension, e.pos.x + dx, e.pos.y + dy, e.pos.z + dz);
                    if side.is_some_and(|b| b.starts_with("minecraft:lava")) {
                        boost += HEAT_LAVA_SIDE;
                    }
                }
                if e.dimension == "minecraft:the_nether" {
                    boost += HEAT_NETHER;
                }
                let burning = can && tank.chemical == "minecraft:lava" && tank.amount >= HEAT_FLUID_RATE && room(energy) > 0;
                if burning {
                    tank.amount -= HEAT_FLUID_RATE;
                    active = true;
                }
                let ambient = world::get_environment(&e.dimension, e.pos.x, e.pos.y, e.pos.z).map_or(AMBIENT, |env| ambient_of(env.temperature));
                let out = heat_tick(heat, boost, burning, ambient);
                heat = out.heat;
                let made = out.energy.min(room(energy));
                energy += made;
                producing = made;
                losses = (0.0, out.environment);
                extra = (heat / HEAT_CAPACITY).round() as i32;
            }
            GenKind::Gas => {
                let (used, made) = if can { gas_tick(&tank.chemical, tank.amount, GAS_TANK, room(energy)) } else { (0, 0) };
                if used > 0 {
                    tank.amount -= used;
                    energy += made;
                    active = true;
                    producing = made;
                }
                extra = used as i32;
            }
            GenKind::Solar => {
                if let Some(env) = world::get_environment(&e.dimension, e.pos.x, e.pos.y, e.pos.z) {
                    let (brightness, darken) = sky(env.day_time, env.rain, env.thunder);
                    let sees = env.has_skylight && darken < 4 && self.sky_clear(e, tick);
                    extra = i32::from(sees);
                    if sees && can {
                        let made = solar_production(brightness, peak(env.temperature, env.downfall, env.precipitation)).min(room(energy));
                        energy += made;
                        producing = made;
                        active = made > 0;
                    }
                }
            }
        }
        if tank.amount == 0 {
            tank.chemical.clear();
        }
        let lit = e.state.contains("lit=true");
        if active != lit {
            let _ = world::set_block_property(&e.dimension, e.pos.x, e.pos.y, e.pos.z, "lit", if active { "true" } else { "false" });
        }
        if !self.filtered.contains(&key) && kind != GenKind::Solar {
            let accepts: Vec<String> = match kind {
                GenKind::Bio => vec![BIOETHANOL.into()],
                GenKind::Heat => vec!["minecraft:lava".into()],
                _ => vec!["mekanism:hydrogen".into(), "mekanism:ethene".into()],
            };
            let _ = world::set_tank_filter(&e.dimension, e.pos.x, e.pos.y, e.pos.z, 0, &accepts);
            self.filtered.insert(key.clone());
        }
        let dirty = !placed || !self.applied.contains(&key);
        if dirty {
            let _ = world::set_item_faces(&e.dimension, e.pos.x, e.pos.y, e.pos.z, &item_faces(kind, &facing));
            self.applied.insert(key.clone());
        }
        let half = |v: u64, shift: u32| ((v >> shift) & 0xFFFF) as i32;
        let mut data = vec![
            i32::from(active),
            half(energy, 0),
            half(energy, 16),
            half(capacity, 0),
            half(capacity, 16),
            producing as i32,
            extra,
            losses.0 as i32,
            losses.1.round() as i32,
            0,
        ];
        data.extend(if kind == GenKind::Solar { [0; 5] } else { plus::big_tank(&tank, tank_cap) });
        let mut saved = heat.to_le_bytes().to_vec();
        saved.push(1);
        let control = Control::from_byte(e.data.get(redstone::offset(Kind::Generator)).copied().unwrap_or(0));
        redstone::stamp(Kind::Generator, control, capacity.saturating_sub(energy), (0, self.install.get(&key).copied().unwrap_or(0), upgrades::counts_of(def, e), 0), &facing, &mut saved, &mut data);
        let tanks_changed = kind != GenKind::Solar && (tank != held(e, 0) || e.tanks.len() != 1);
        if !items_changed && !dirty && !tanks_changed && energy == e.energy && data == e.menu_data && saved == e.data && !self.force {
            return None;
        }
        Some(BlockEntityUpdate {
            dimension: e.dimension.clone(),
            pos: e.pos,
            data: saved,
            items: (items_changed || self.force).then(|| items.iter().map(item).collect()),
            energy: Some(energy),
            energy_faces: dirty.then(|| energy_faces(kind, &facing)),
            tanks: tanks_changed.then(|| vec![plus::as_tank(&tank)]),
            tank_faces: dirty.then(|| vec![tank_faces(&facing)]).filter(|_| kind != GenKind::Solar),
            menu_data: Some(data),
            transmitter_faces: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heat_settles_where_the_loss_equals_the_burn() {
        // 100 heat a tick from the lava: it settles near 500 K and the energy near 40 a tick.
        let mut heat = AMBIENT * HEAT_CAPACITY;
        let mut last = HeatOut { heat, energy: 0, environment: 0.0 };
        for _ in 0..2000 {
            last = heat_tick(heat, 0.0, true, AMBIENT);
            heat = last.heat;
        }
        let temp = heat / HEAT_CAPACITY;
        assert!((480.0..=495.0).contains(&temp), "settled at {temp}");
        assert!((35..=42).contains(&last.energy), "makes {}", last.energy);
        // Without fuel it cools back to the air's temperature.
        for _ in 0..20_000 {
            heat = heat_tick(heat, 0.0, false, AMBIENT).heat;
        }
        assert!((heat / HEAT_CAPACITY - AMBIENT).abs() < 1.0);
        // The lava beside it and the nether add heat even with no fuel.
        let warm = (0..2000).fold(AMBIENT * HEAT_CAPACITY, |h, _| heat_tick(h, 7.0 * 3.0 + 10.0, false, AMBIENT).heat);
        assert!(warm / HEAT_CAPACITY > AMBIENT + 20.0);
        for (biome, kelvin) in [(0.8, 300.0), (2.0, 330.0), (-0.5, 267.5), (50.0, 405.0), (-9.0, 155.0)] {
            assert!((ambient_of(biome) - kelvin).abs() < 1e-3, "{biome}");
        }
    }

    #[test]
    fn fuel_items_become_lava_and_bioethanol() {
        assert_eq!(lava_of("minecraft:coal"), Some(80));
        assert_eq!(lava_of("minecraft:coal_block"), Some(800));
        assert_eq!(lava_of("minecraft:oak_planks"), Some(15));
        assert_eq!(lava_of("minecraft:dirt"), None);
        assert_eq!(bioethanol_of("mekanism:bio_fuel"), Some(64));
        assert_eq!(bioethanol_of("mekanism:block_bio_fuel"), Some(576));
        assert_eq!(bioethanol_of("minecraft:coal"), None);
    }

    #[test]
    fn a_gas_burns_faster_the_fuller_the_tank() {
        // Hydrogen: 2 an mB, up to 10 mB a tick, by how full the tank is.
        assert_eq!(gas_tick("mekanism:hydrogen", 1_000, 1_000, 2_000), (10, 20));
        assert_eq!(gas_tick("mekanism:hydrogen", 500, 1_000, 2_000), (5, 10));
        assert_eq!(gas_tick("mekanism:hydrogen", 3, 1_000, 2_000), (1, 2));
        assert_eq!(gas_tick("mekanism:ethene", 1_000, 1_000, 2_000), (10, 80));
        // Only what the container has room for, in whole mB.
        assert_eq!(gas_tick("mekanism:ethene", 1_000, 1_000, 20), (3, 20));
        assert_eq!(gas_tick("mekanism:oxygen", 1_000, 1_000, 2_000), (0, 0));
        assert_eq!(gas_tick("mekanism:hydrogen", 1_000, 1_000, 0), (0, 0));
    }

    #[test]
    fn the_sun_follows_the_day_and_the_weather() {
        let (noon, dark_noon) = sky(6_000, 0.0, 0.0);
        assert!((noon - 1.0).abs() < 1e-6 && dark_noon == 0);
        let (night, dark_night) = sky(18_000, 0.0, 0.0);
        assert!(night < 0.01 && dark_night >= 4);
        let (rainy, _) = sky(6_000, 1.0, 0.0);
        assert!((rainy - 11.0 / 16.0).abs() < 1e-6);
        // Plains (0.8, 0.4, rain): 0.88 of the sun, so 1 a tick of the base 2; a cold dry biome makes more.
        let plains = peak(0.8, 0.4, true);
        assert!((plains - 0.88).abs() < 1e-6);
        assert_eq!(solar_production(1.0, plains), 1);
        assert_eq!(solar_production(1.0, peak(2.0, 0.0, false)), 1);
        assert_eq!(solar_production(1.0, peak(-0.5, 0.0, false)), 2);
        assert_eq!(solar_production(0.0, plains), 0);
    }

    #[test]
    fn faces_turn_with_the_facing() {
        // Facing north: the front is north, the right (the energy slot's) is west.
        let f = item_faces(GenKind::Bio, "north");
        assert_eq!((f[config::WEST].clone(), f[config::NORTH].clone(), f[config::UP].clone()), (vec![1], vec![0], vec![0]));
        assert_eq!(tank_faces("north")[config::NORTH], FlowSide::None);
        assert_eq!(tank_faces("north")[config::SOUTH], FlowSide::Input);
        assert_eq!(energy_faces(GenKind::Gas, "east")[config::EAST], FlowSide::Output);
        assert_eq!(energy_faces(GenKind::Solar, "east")[config::DOWN], FlowSide::Output);
    }
}

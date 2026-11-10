//! The Meka-Tool in the world: what the right click does with the pieces of `meka_tool`.
//!
//! Done here: the teleportation unit (`ItemMekaTool.use`, java:373-417): the ray, the destination check, the energy,
//! the teleport. Not done, because the game gives the mod no way yet: the extra attack damage and its energy
//! (`item-hit` only hears; FORK damage hook), the radial menu (FORK-27), the area and vein mining (FORK-31) and the
//! farming units (a use that changes blocks with the tool's own rules). Nothing here charges energy for an effect the
//! game does not apply.

use horizon_mod_sdk::server::{player_control, player_items, players, world, BlockEntity, BlockEntityUpdate, BlockPos, DamageSource, ItemStack, Player};

use crate::{assemblicator, configuration_card, generators, mekasuit, meka_tool, Mekanism};

/// Whether a ray passes through the block: air and fluids (`ClipContext.Fluid.NONE` skips fluids; a plant or a torch has an outline and is hit).
pub fn passes_ray(block: Option<&str>) -> bool {
    let Some(state) = block else { return true };
    let name = state.split('[').next().unwrap_or("");
    matches!(
        name,
        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air" | "minecraft:water" | "minecraft:lava" | "minecraft:bubble_column"
    )
}

/// `isValidDestinationBlock`: air or a fluid.
pub fn valid_destination(block: Option<&str>) -> bool {
    passes_ray(block)
}

/// The first block a ray from `eye` along `dir` meets within `reach` blocks (a voxel walk, Amanatides-Woo), or none for a miss.
pub fn ray_block(eye: [f64; 3], dir: [f64; 3], reach: f64, passable: impl Fn(i32, i32, i32) -> bool) -> Option<[i32; 3]> {
    let mut cell = [eye[0].floor() as i32, eye[1].floor() as i32, eye[2].floor() as i32];
    let mut step = [0i32; 3];
    let mut t_max = [f64::INFINITY; 3];
    let mut t_delta = [f64::INFINITY; 3];
    for i in 0..3 {
        if dir[i] > 0.0 {
            step[i] = 1;
            t_delta[i] = 1.0 / dir[i];
            t_max[i] = (f64::from(cell[i]) + 1.0 - eye[i]) / dir[i];
        } else if dir[i] < 0.0 {
            step[i] = -1;
            t_delta[i] = -1.0 / dir[i];
            t_max[i] = (eye[i] - f64::from(cell[i])) / -dir[i];
        }
    }
    loop {
        if !passable(cell[0], cell[1], cell[2]) {
            return Some(cell);
        }
        let axis = if t_max[0] <= t_max[1] && t_max[0] <= t_max[2] { 0 } else if t_max[1] <= t_max[2] { 1 } else { 2 };
        if t_max[axis] > reach {
            return None;
        }
        cell[axis] += step[axis];
        t_max[axis] += t_delta[axis];
    }
}

/// The components with the tool's stored energy replaced (`"energy_containers":[N]`), the rest byte-for-byte.
pub fn with_energy(components: &str, energy: u64) -> Option<String> {
    const KEY: &str = "\"energy_containers\":[";
    let at = components.find(KEY)? + KEY.len();
    let end = at + components[at..].find(']')?;
    let mut out = components.to_owned();
    out.replace_range(at..end, &energy.to_string());
    Some(out)
}

impl Mekanism {
    /// A right click with a Meka-Tool in the air or on a block. Answers whether the mod took the use.
    pub(crate) fn meka_tool_used(&mut self, player: &Player, stack: &ItemStack) -> bool {
        let data = meka_tool::host_read_meka_tool(&stack.components);
        if !data.is_module_enabled(meka_tool::ModuleType::TeleportationUnit) {
            return false;
        }
        let requires_target = data.get_module(meka_tool::ModuleType::TeleportationUnit).is_some_and(|m| m.teleport_requires_target);
        let sneaking = player_control::get_motion(&player.id).is_some_and(|m| m.sneaking);
        // `MekanismUtils.getHeadVec`: the eye, lowered 0.08 while crouching.
        let eye = [player.x, player.y + if sneaking { 1.62 - 0.08 } else { 1.62 }, player.z];
        let Some((yaw, pitch)) = player_control::get_look(&player.id) else { return false };
        let dir = crate::sides::look(yaw, pitch);
        let dim = player.dimension.clone();
        let hit = ray_block(eye, dir, f64::from(meka_tool::MAX_TELEPORT_REACH), |x, y, z| passes_ray(world::get_block(&dim, x, y, z).as_deref()));
        // A miss ends the ray at its far end (`BlockHitResult.getBlockPos` of a miss).
        let target = hit.unwrap_or_else(|| {
            let r = f64::from(meka_tool::MAX_TELEPORT_REACH);
            [(eye[0] + dir[0] * r).floor() as i32, (eye[1] + dir[1] * r).floor() as i32, (eye[2] + dir[2] * r).floor() as i32]
        });
        let head_free = valid_destination(world::get_block(&dim, target[0], target[1] + 1, target[2]).as_deref());
        let above_free = valid_destination(world::get_block(&dim, target[0], target[1] + 2, target[2]).as_deref());
        let Ok((dest, cost)) = meka_tool::check_teleport([player.x, player.y, player.z], target, head_free, above_free, requires_target, hit.is_some(), data.energy) else {
            return false;
        };
        let Some((slot, components)) = Self::find_hotbar_slot(&player.id, stack) else { return false };
        let Some(written) = with_energy(&components, data.energy - cost) else { return false };
        if player_control::teleport(&player.id, &dim, dest[0], dest[1], dest[2], None, None).is_err() {
            return false;
        }
        let _ = player_items::set_slot(&player.id, slot, Some(&ItemStack { item: stack.item.clone(), count: stack.count, components: written }));
        true
    }
}

/// A configuration card used on a block, until the block's next tick (a use cannot change a block entity).
#[derive(Clone, Debug)]
pub struct CardClick {
    pub player: String,
    pub dimension: String,
    pub pos: BlockPos,
    pub sneaking: bool,
    pub stack: ItemStack,
}

impl Mekanism {
    /// The configuration card and the crafting formula. The card is wired for the formulaic assemblicator only: `apply_configuration`
    /// for the other machines assumes a layout and offsets that are not checked against each machine's data yet, and a wrong
    /// write would corrupt the machine. Answers whether the mod took the use.
    pub(crate) fn card_or_formula_used(&mut self, player: &Player, stack: &ItemStack, pos: Option<BlockPos>) -> bool {
        let sneaking = player_control::get_motion(&player.id).is_some_and(|m| m.sneaking);
        let original = stack.clone();
        let mut stack = stack.clone();
        if stack.item == assemblicator::CRAFTING_FORMULA {
            // Shift + use in the air clears the formula (`ItemCraftingFormula.use`).
            if pos.is_none() && sneaking {
                assemblicator::CraftingFormulaItem::clear(&mut stack);
                self.write_held(player, &original, &stack);
                let _ = players::tell(&player.id, "Crafting Formula cleared");
                return true;
            }
            return false;
        }
        match pos {
            None => {
                if configuration_card::ConfigurationCardItem::use_air(&player.id, sneaking, &mut stack) {
                    self.write_held(player, &original, &stack);
                    return true;
                }
                false
            }
            Some(p) => {
                let on_assemblicator = world::get_block(&player.dimension, p.x, p.y, p.z)
                    .is_some_and(|b| b.split('[').next() == Some(assemblicator::BLOCK));
                if !on_assemblicator {
                    return false;
                }
                self.card_clicks.push(CardClick { player: player.id.clone(), dimension: player.dimension.clone(), pos: p, sneaking, stack });
                true
            }
        }
    }

    /// Writes `stack` over the held item it came from (found again in the hotbar; never guessed).
    fn write_held(&mut self, player: &Player, original: &ItemStack, written: &ItemStack) {
        // The stack to replace is the one the player held before the change, found by its item, count and components.
        let Some(inv) = player_items::get_inventory(&player.id) else { return };
        let slot = inv.iter().take(9).position(|s| {
            s.as_ref().is_some_and(|s| s.item == original.item && s.count == original.count && crate::config::same_components(&original.components, &s.components))
        });
        if let Some(slot) = slot {
            let _ = player_items::set_slot(&player.id, slot as u32, Some(written));
        }
    }

    /// The card clicks whose machine ticks now: copy reads the machine, paste rewrites its data.
    pub(crate) fn card_clicks_tick(&mut self, entities: &mut [BlockEntity]) -> Vec<BlockEntityUpdate> {
        let mut out = Vec::new();
        for click in std::mem::take(&mut self.card_clicks) {
            let Some(e) = entities.iter_mut().find(|e| e.dimension == click.dimension && e.pos == click.pos) else { continue };
            let block = e.state.split('[').next().unwrap_or("").to_owned();
            let before = e.data.clone();
            let mut card = click.stack.clone();
            let res = configuration_card::ConfigurationCardItem::use_on(&click.player, click.sneaking, &block, e, &mut card);
            if card != click.stack {
                if let Some((slot, _)) = Self::find_hotbar_slot(&click.player, &click.stack) {
                    let _ = player_items::set_slot(&click.player, slot, Some(&card));
                }
            }
            if e.data != before && matches!(res, configuration_card::CardInteractionResult::Pasted { .. }) {
                out.push(BlockEntityUpdate {
                    dimension: e.dimension.clone(),
                    pos: e.pos,
                    data: e.data.clone(),
                    items: None,
                    energy: None,
                    energy_faces: None,
                    tanks: None,
                    tank_faces: None,
                    menu_data: None,
                    transmitter_faces: None,
                });
            }
        }
        out
    }
}

impl Mekanism {
    /// The MekaSuit worn by every player online, one tick (`ItemMekaSuitArmor.tickServer` via `CommonPlayerTickHandler`): what the
    /// game tells the mod today is enough for the solar recharging unit (the head's sun and weather) and for the charge distribution
    /// among the worn pieces. The rest of the suit (the jetpack and the movement modules need the velocity and the keys, the
    /// nutritional injection a way to feed, the breathing the air supply, the geothermal unit the fire, the damage absorption a
    /// damage hook, the charge of inventory items the other items' energy) waits for the API listed in `PROPOSTA-API.wit`; nothing of
    /// it runs on guessed inputs, so no energy is spent for an effect the game does not apply.
    pub(crate) fn suits_tick(&mut self, tick: u64) {
        for p in players::online() {
            let Some(inv) = player_items::get_inventory(&p.id) else { continue };
            // Slots 36 feet, 37 legs, 38 chest, 39 head; the suit's pieces are head, chest, legs, feet.
            let slots = [39usize, 38, 37, 36];
            let mut pieces: [Option<mekasuit::MekaSuitItemStack>; 4] = [None, None, None, None];
            let mut originals: [Option<ItemStack>; 4] = [None, None, None, None];
            for (i, &s) in slots.iter().enumerate() {
                if let Some(Some(stack)) = inv.get(s) {
                    pieces[i] = mekasuit::MekaSuitItemStack::decode(&stack.item, stack.count, &stack.components);
                    if pieces[i].is_some() {
                        originals[i] = Some(stack.clone());
                    }
                }
            }
            if pieces.iter().all(Option::is_none) {
                continue;
            }
            let before = pieces.clone();

            // The solar recharging unit: the position roughly where the panel is, the eye and 0.2 above (`ModuleSolarRechargingUnit`).
            if let Some(helmet) = pieces[mekasuit::ArmorSlot::Helmet as usize].as_mut() {
                if let Some(env) = world::get_environment(&p.dimension, p.x.floor() as i32, (p.y + 1.62 + 0.2).floor() as i32, p.z.floor() as i32) {
                    let (brightness, darken) = generators::sky(env.day_time, env.rain, env.thunder);
                    let pos = [p.x.floor() as i32, (p.y + 1.62 + 0.2).floor() as i32, p.z.floor() as i32];
                    let sees = env.has_skylight && darken < 4 && self.sky_clear_at(&p.dimension, pos, tick);
                    // `isRaining` is a rain level above 0.2, `isThundering` a thunder level above 0.9.
                    let raining = env.rain > 0.2 || env.thunder > 0.9;
                    mekasuit::tick_solar_recharging(helmet, sees, env.temperature, env.downfall, env.precipitation, raining, brightness);
                }
            }
            // Charge distribution among the pieces (inventory items' charge needs their energy, not readable yet: 0).
            mekasuit::tick_charge_distribution(&mut pieces, 0);

            for i in 0..4 {
                if pieces[i] == before[i] {
                    continue;
                }
                let (Some(piece), Some(original)) = (pieces[i].as_ref(), originals[i].as_ref()) else { continue };
                let replacement = ItemStack { item: original.item.clone(), count: original.count, components: piece.encode(&original.components) };
                let _ = player_items::compare_and_set_slot(&p.id, slots[i] as u32, Some(original), Some(&replacement));
            }
        }
    }
}

/// The damage types of `mekanism:mekasuit_always_supported` (the suit absorbs them even when they bypass armour): the jar's list, in case the
/// game does not hand over the mod's own tags.
const ALWAYS_SUPPORTED: [&str; 21] = [
    "minecraft:falling_anvil", "minecraft:cactus", "minecraft:cramming", "minecraft:dragon_breath", "minecraft:dry_out", "minecraft:fall",
    "minecraft:falling_block", "minecraft:fly_into_wall", "minecraft:generic", "minecraft:hot_floor", "minecraft:in_fire", "minecraft:in_wall",
    "minecraft:lava", "minecraft:lightning_bolt", "minecraft:on_fire", "minecraft:sweet_berry_bush", "minecraft:wither", "minecraft:freeze",
    "minecraft:falling_stalactite", "minecraft:stalagmite", "minecraft:sonic_boom",
];

/// The game's description of a hurt as the suit's own (`mekasuit::DamageSource`).
pub fn suit_source(source: &DamageSource) -> mekasuit::DamageSource {
    let has = |tag: &str| source.tags.iter().any(|t| t == tag);
    mekasuit::DamageSource {
        name: source.damage_type.clone(),
        is_fire: source.is_fire,
        // The inhalation unit takes `mekanism:is_preventable_magic` (magic and indirect magic).
        is_magic: has("mekanism:is_preventable_magic") || source.damage_type == "minecraft:magic" || source.damage_type == "minecraft:indirect_magic",
        is_fall: source.is_fall,
        is_technical: has("c:is_technical") || has("neoforge:is_technical"),
        bypasses_armor: has("minecraft:bypasses_armor"),
        always_supported: has("mekanism:mekasuit_always_supported") || ALWAYS_SUPPORTED.contains(&source.damage_type.as_str()),
    }
}

/// The attack damage with a Meka-Tool in hand: `damage` (the player's own) plus the tool's attribute; any other item leaves it.
pub fn hit_damage(stack: &ItemStack, damage: f32) -> f32 {
    if stack.item != meka_tool::MEKA_TOOL {
        return damage;
    }
    let data = meka_tool::host_read_meka_tool(&stack.components);
    damage + data.calculate_attack().0 as f32
}

impl Mekanism {
    /// The suit a player wears (head, chest, legs, feet), as read from the inventory of the last tick, with the stacks they came from.
    fn worn_suit(player: &str) -> Option<([Option<mekasuit::MekaSuitItemStack>; 4], [Option<ItemStack>; 4])> {
        let inv = player_items::get_inventory(player)?;
        let mut pieces: [Option<mekasuit::MekaSuitItemStack>; 4] = [None, None, None, None];
        let mut originals: [Option<ItemStack>; 4] = [None, None, None, None];
        for (i, &s) in [39usize, 38, 37, 36].iter().enumerate() {
            if let Some(Some(stack)) = inv.get(s) {
                pieces[i] = mekasuit::MekaSuitItemStack::decode(&stack.item, stack.count, &stack.components);
                if pieces[i].is_some() {
                    originals[i] = Some(stack.clone());
                }
            }
        }
        Some((pieces, originals))
    }

    /// Writes the pieces that changed back to the armour slots, each only if it is still the stack that was read.
    fn write_suit(player: &str, before: &[Option<mekasuit::MekaSuitItemStack>; 4], pieces: &[Option<mekasuit::MekaSuitItemStack>; 4], originals: &[Option<ItemStack>; 4]) {
        for (i, &slot) in [39u32, 38, 37, 36].iter().enumerate() {
            if pieces[i] == before[i] {
                continue;
            }
            let (Some(piece), Some(original)) = (pieces[i].as_ref(), originals[i].as_ref()) else { continue };
            let replacement = ItemStack { item: original.item.clone(), count: original.count, components: piece.encode(&original.components) };
            let _ = player_items::compare_and_set_slot(player, slot, Some(original), Some(&replacement));
        }
    }

    /// `CommonPlayerTickHandler.livingFall` then `onEntityAttacked`: the boots take their share of a fall (`mekaSuitFallDamageRatio`, 50 J a point),
    /// then the suit's modules and pieces take theirs of the damage left (`ItemMekaSuitArmor.getDamageAbsorbed`). Returns the damage to go on
    /// with. The gas mask's cancel of preventable magic (a scuba tank with gas) is not done yet: it needs the chemical tank of the mask.
    pub(crate) fn suit_damage(&mut self, player: &Player, source: &DamageSource, amount: f32) -> f32 {
        if amount <= 0.0 {
            return amount;
        }
        let Some((mut pieces, originals)) = Self::worn_suit(&player.id) else { return amount };
        if pieces.iter().all(Option::is_none) {
            return amount;
        }
        let before = pieces.clone();
        let src = suit_source(source);
        let mut left = amount;
        if src.is_fall {
            if let Some(boots) = pieces[mekasuit::ArmorSlot::Feet as usize].as_mut() {
                let (absorbed, _) = mekasuit::absorb_fall_damage(boots, left);
                left = (left - absorbed).max(0.0);
            }
        }
        if left > f32::EPSILON {
            left = mekasuit::calculate_damage_absorbed(&mut pieces, &src, left).damage_remaining;
        }
        Self::write_suit(&player.id, &before, &pieces, &originals);
        if left <= f32::EPSILON { 0.0 } else { left }
    }

    /// The attack damage of a Meka-Tool blow (`ItemMekaTool.adjustAttributes`: the attribute is the base damage plus the attack amplification
    /// unit's, at a reduced share when the energy is short), on top of the player's own 1.
    pub(crate) fn meka_tool_hit_damage(&mut self, stack: &ItemStack, damage: f32) -> f32 {
        hit_damage(stack, damage)
    }

    /// `ItemMekaTool.hurtEnemy`: a blow with the attack amplification unit costs its energy (all of what is left if that is less).
    pub(crate) fn meka_tool_hit(&mut self, player: &Player, stack: &ItemStack) {
        if stack.item != meka_tool::MEKA_TOOL {
            return;
        }
        let data = meka_tool::host_read_meka_tool(&stack.components);
        let (_, cost) = data.calculate_attack();
        if cost == 0 || data.energy == 0 {
            return;
        }
        let Some((slot, components)) = Self::find_hotbar_slot(&player.id, stack) else { return };
        let Some(written) = with_energy(&components, data.energy.saturating_sub(cost)) else { return };
        let current = ItemStack { item: stack.item.clone(), count: stack.count, components };
        let replacement = ItemStack { item: stack.item.clone(), count: stack.count, components: written };
        let _ = player_items::compare_and_set_slot(&player.id, slot, Some(&current), Some(&replacement));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ray_stops_at_the_first_solid_block() {
        // A ray that starts inside a solid cell hits that cell at once.
        assert_eq!(ray_block([0.5, 0.5, 0.5], [1.0, 0.0, 0.0], 10.0, |_, _, _| false), Some([0, 0, 0]));
        let one_block = |x: i32, y: i32, z: i32| !(x == 5 && y == 0 && z == 0);
        assert_eq!(ray_block([0.5, 0.5, 0.5], [1.0, 0.0, 0.0], 10.0, one_block), Some([5, 0, 0]));
        let air_until_x5 = |x: i32, _y: i32, _z: i32| x < 5;
        assert_eq!(ray_block([0.5, 0.5, 0.5], [1.0, 0.0, 0.0], 10.0, air_until_x5), Some([5, 0, 0]));
    }

    #[test]
    fn the_ray_misses_beyond_its_reach() {
        assert_eq!(ray_block([0.5, 0.5, 0.5], [1.0, 0.0, 0.0], 3.0, |_, _, _| true), None);
    }

    #[test]
    fn the_ray_walks_a_diagonal_and_downwards() {
        let floor = |_x: i32, y: i32, _z: i32| y > -3;
        let hit = ray_block([0.5, 0.5, 0.5], [0.0, -1.0, 0.0], 20.0, floor);
        assert_eq!(hit, Some([0, -3, 0]));
    }

    #[test]
    fn air_and_fluids_pass_the_ray_and_a_plant_does_not() {
        assert!(passes_ray(Some("minecraft:air")));
        assert!(passes_ray(Some("minecraft:water[level=0]")));
        assert!(passes_ray(None));
        assert!(!passes_ray(Some("minecraft:short_grass")));
        assert!(!passes_ray(Some("minecraft:stone")));
    }

    #[test]
    fn the_energy_is_replaced_in_place() {
        let c = r#"{"mekanism:energy":{"energy_containers":[1000]},"minecraft:damage":3}"#;
        assert_eq!(with_energy(c, 250).as_deref(), Some(r#"{"mekanism:energy":{"energy_containers":[250]},"minecraft:damage":3}"#));
        assert_eq!(with_energy("{}", 5), None);
    }

    fn source(damage_type: &str, tags: &[&str]) -> DamageSource {
        DamageSource {
            damage_type: damage_type.into(),
            causing: None,
            is_magic: false,
            is_fire: false,
            is_fall: damage_type == "minecraft:fall",
            is_explosion: false,
            tags: tags.iter().map(|t| (*t).to_owned()).collect(),
        }
    }

    #[test]
    fn the_suit_sees_the_damage_types_as_the_original_tags_do() {
        let fall = suit_source(&source("minecraft:fall", &["minecraft:bypasses_armor"]));
        assert!(fall.is_fall && fall.bypasses_armor && fall.always_supported && !fall.is_technical);
        let void = suit_source(&source("minecraft:out_of_world", &["minecraft:bypasses_armor"]));
        assert!(!void.always_supported && void.bypasses_armor);
        let magic = suit_source(&source("minecraft:magic", &[]));
        assert!(magic.is_magic);
        let tech = suit_source(&source("mod:odd", &["c:is_technical"]));
        assert!(tech.is_technical);
        // Without the tags the jar's list still marks the always-supported types.
        assert!(suit_source(&source("minecraft:lava", &[])).always_supported);
    }

    #[test]
    fn a_meka_tool_blow_adds_the_tools_damage_to_the_players() {
        let plain = ItemStack { item: meka_tool::MEKA_TOOL.into(), count: 1, components: "{}".into() };
        // No modules: the attribute is the tool's base damage on top of the player's 1.
        assert_eq!(hit_damage(&plain, 1.0), 1.0 + meka_tool::BASE_ATTACK_DAMAGE as f32);
        let other = ItemStack { item: "minecraft:stick".into(), count: 1, components: String::new() };
        assert_eq!(hit_damage(&other, 1.0), 1.0);
    }
}

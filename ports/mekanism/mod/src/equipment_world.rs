//! The Meka-Tool in the world: what the right click does with the pieces of `meka_tool`.
//!
//! Done here: the teleportation unit (`ItemMekaTool.use`, java:373-417): the ray, the destination check, the energy,
//! the teleport. Not done, because the game gives the mod no way yet: the extra attack damage and its energy
//! (`item-hit` only hears; FORK damage hook), the radial menu (FORK-27), the area and vein mining (FORK-31) and the
//! farming units (a use that changes blocks with the tool's own rules). Nothing here charges energy for an effect the
//! game does not apply.

use horizon_mod_sdk::server::{entities, GameMode, player_control, player_items, players, world, BlockEntity, BlockEntityUpdate, BlockPos, DamageSource, ItemStack, Player};

use crate::{assemblicator, configuration_card, electric_bow, generators, mekasuit, meka_tool, Mekanism};

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
    /// The configuration card and the crafting formula. The card works on the formulaic assemblicator, the resistive heater and the
    /// configurable machines (`configuration_card::supports`). Answers whether the mod took the use.
    pub(crate) fn card_or_formula_used(&mut self, player: &Player, stack: &ItemStack, pos: Option<BlockPos>) -> bool {
        let sneaking = player_control::get_motion(&player.id).is_some_and(|m| m.sneaking);
        let original = stack.clone();
        let mut stack = stack.clone();
        if stack.item == assemblicator::CRAFTING_FORMULA {
            // Shift + use clears the formula, silently (`ItemCraftingFormula.use`: `FORMULA_HOLDER` set to empty, no message).
            if sneaking {
                assemblicator::CraftingFormulaItem::clear(&mut stack);
                self.write_held(player, &original, &stack);
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
                let supported = world::get_block(&player.dimension, p.x, p.y, p.z)
                    .is_some_and(|b| configuration_card::supports(b.split('[').next().unwrap_or("")));
                if !supported {
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
    /// The MekaSuit worn by every player online, one tick (`ItemMekaSuitArmor.tickServer` via `CommonPlayerTickHandler`): the solar
    /// recharging unit, the nutritional injection, the geothermal unit, the jetpack and the charge distribution among the worn pieces.
    /// Still missing, because the game gives the mod no way: the breathing (set the air), the inhalation purification (shorten an effect),
    /// the locomotive boost and the step assist (attributes), the magnetic attraction (dropped items), the fall distance the jetpack
    /// resets, the elytra glide and the charge of inventory items. Nothing runs on guessed inputs: no energy is spent for an effect
    /// the game does not apply.
    pub(crate) fn suits_tick(&mut self, tick: u64) {
        let online = players::online();
        // A player who left loses the transient modifier: the next time they are here it is set again.
        self.suit_step.retain(|id, _| online.iter().any(|p| &p.id == id));
        for p in online {
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
            self.suit_hud(&p.id, &pieces);
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
            // The sky at the head for the rain: it rains (`isRainingAt`: a rain level above 0.2, rain and not snow, and nothing above).
            let head = [p.x.floor() as i32, (p.y + 1.62).floor() as i32, p.z.floor() as i32];
            let env_rain = world::get_environment(&p.dimension, head[0], head[1], head[2]).map(|env| {
                let sky = env.has_skylight && env.precipitation && self.sky_clear_at(&p.dimension, head, tick);
                (env.rain > 0.2, env.temperature, sky)
            });
            // The player's body (API 29): what the nutrition, the geothermal unit and the jetpack read.
            let state = player_control::get_state(&p.id);
            let mut glider: Option<bool> = None;
            let playing = p.game_mode != GameMode::Creative && p.game_mode != GameMode::Spectator;
            if let Some(st) = &state {
                // Nutritional injection: only in playing mode and when the player can eat (`canEat(false)`: food below 20).
                if playing {
                    if let Some(helmet) = pieces[mekasuit::ArmorSlot::Helmet as usize].as_mut() {
                        let (eaten, modifier, _) = mekasuit::tick_nutritional_injection(helmet, st.food as i32);
                        if eaten > 0 {
                            let (food, saturation) = eat(st.food, st.saturation, eaten as u32, modifier);
                            let _ = player_control::set_food(&p.id, food, saturation);
                        }
                    }
                }
                // Geothermal unit: lava (1300 K, taken as fully up the legs) or fire (the original's 200 degrees).
                if let Some(pants) = pieces[mekasuit::ArmorSlot::Legs as usize].as_mut() {
                    mekasuit::tick_geothermal_generator(pants, st.fire_ticks > 0, if st.in_lava { 1300.0 } else { 0.0 });
                }
                let motion = player_control::get_motion(&p.id);
                let sneaking = motion.as_ref().is_some_and(|m| m.sneaking);
                let on_ground = motion.as_ref().is_some_and(|m| m.on_ground);
                let creative_or_spectator = !playing;
                let look = player_control::get_look(&p.id);
                // The body's velocity as the tick began; the locomotive boost and then the jetpack change it (java: the armour tick, then `tickEnd`).
                let mut velocity = [st.velocity_x, st.velocity_y, st.velocity_z];
                let mut velocity_changed = false;
                let move_input = mekasuit::MoveInput {
                    sprinting: st.sprinting,
                    fall_flying: st.fall_flying,
                    on_ground,
                    in_water: st.in_water,
                    in_fluid: st.in_water || st.in_lava,
                    sneaking,
                    flying: st.flying,
                    boost_key: false,
                    creative_or_spectator,
                };
                // Locomotive boost: `moveRelative(boost, (0, 0, 1))`, a push along where the player faces.
                if let (Some(pants), Some((yaw, _))) = (pieces[mekasuit::ArmorSlot::Legs as usize].as_mut(), look) {
                    let (boost, _) = mekasuit::tick_locomotive_boost(pants, &move_input);
                    if boost != 0.0 {
                        let rad = f64::from(yaw).to_radians();
                        velocity[0] += -rad.sin() * f64::from(boost);
                        velocity[2] += rad.cos() * f64::from(boost);
                        velocity_changed = true;
                    }
                }
                // Step assist: the boots' extra step height as a transient modifier, put back when it changes.
                let step = mekasuit::step_boost(pieces[mekasuit::ArmorSlot::Feet as usize].as_ref(), sneaking, false);
                if self.suit_step.get(&p.id).copied() != Some(step) {
                    if step > 0.0 {
                        let _ = player_control::set_attribute_modifier(&p.id, "minecraft:step_height", STEP_MODIFIER, f64::from(step), "add_value");
                    } else {
                        let _ = player_control::remove_attribute_modifier(&p.id, "minecraft:step_height", STEP_MODIFIER);
                    }
                    self.suit_step.insert(p.id.clone(), step);
                }
                // Electrolytic breathing: the eyes in water (full rate) or in the rain (half).
                {
                    let in_rain = env_rain.is_some_and(|(raining, temperature, sky)| raining && sky && temperature >= 0.15);
                    let (head, rest) = pieces.split_at_mut(1);
                    if let Some(helmet) = head[0].as_mut() {
                        let (oxygen, _) = mekasuit::tick_electrolytic_breathing(helmet, rest[0].as_mut(), st.eyes_in_water, in_rain, st.air, st.max_air);
                        if oxygen > 0 {
                            let _ = player_control::set_air(&p.id, st.air + oxygen);
                        }
                        // Inhalation purification: each effect it handles is shortened by 9 ticks.
                        let effects: Vec<(String, i32)> = st.effects.iter().map(|e| (e.id.clone(), e.duration)).collect();
                        let (shorten, _) = mekasuit::tick_inhalation_purification(helmet, &effects, p.game_mode == GameMode::Creative);
                        for (id, ticks) in shorten {
                            let _ = player_control::reduce_effect(&p.id, &id, ticks);
                        }
                    }
                }
                // The jetpack: the keys, the velocity and the look; the new velocity is set whole.
                if let (Some(body), Some((yaw, pitch))) = (pieces[mekasuit::ArmorSlot::Chest as usize].as_mut(), look) {
                    let input = mekasuit::JetpackInput {
                        ascending: st.jumping,
                        descending: sneaking,
                        on_ground_or_sleeping: mekasuit::is_on_ground_or_sleeping(on_ground, st.sleeping, st.flying),
                        sneaking,
                        fall_flying: st.fall_flying,
                        spectator: p.game_mode == GameMode::Spectator,
                        passenger: st.passenger,
                        on_cooldown: false,
                        motion: velocity,
                        look: crate::sides::look(yaw, pitch),
                        // `Player.getUpVector(1F)`: the view vector of the same yaw with the pitch turned up by 90 degrees.
                        up: crate::sides::look(yaw, pitch - 90.0),
                    };
                    let jet = mekasuit::tick_jetpack(body, &input);
                    if let Some(m) = jet.new_motion {
                        velocity = m;
                        velocity_changed = true;
                    }
                    if jet.reset_fall {
                        let _ = player_control::reset_fall_distance(&p.id);
                    }
                }
                if velocity_changed {
                    let _ = player_control::set_motion(&p.id, velocity[0], velocity[1], velocity[2]);
                }
                // The Elytra Unit (`canElytraFly`, `elytraFlightTick`): the chestplate carries `minecraft:glider` while it can fly, and while the player glides
                // it takes its energy every 20 flight ticks.
                if let Some(body) = pieces[mekasuit::ArmorSlot::Chest as usize].as_mut() {
                    let can = mekasuit::can_elytra_fly(body, &move_input);
                    glider = Some(can);
                    if st.fall_flying && can {
                        let ticks = self.glide_ticks.entry(p.id.clone()).or_insert(0);
                        let (_, _) = mekasuit::tick_elytra(body, *ticks, &move_input);
                        *ticks += 1;
                    } else {
                        self.glide_ticks.remove(&p.id);
                    }
                }
            }
            // Charge distribution among the pieces (inventory items' charge needs their energy, not readable yet: 0).
            mekasuit::tick_charge_distribution(&mut pieces, 0);

            for i in 0..4 {
                let (Some(piece), Some(original)) = (pieces[i].as_ref(), originals[i].as_ref()) else { continue };
                let mut components = if pieces[i] == before[i] { original.components.clone() } else { piece.encode(&original.components) };
                if i == mekasuit::ArmorSlot::Chest as usize
                    && let Some(want) = glider
                {
                    components = set_glider(&components, want);
                }
                if components == original.components {
                    continue;
                }
                let replacement = ItemStack { item: original.item.clone(), count: original.count, components };
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

/// The id of the transient step height modifier of the boots.
const STEP_MODIFIER: &str = "mekanism:mekasuit_step_assist";

/// `FoodData.eat(food, saturationModifier)`: the level rises (at most 20), the saturation by `food * modifier * 2` (at most the level).
pub fn eat(food_level: u32, saturation: f32, food: u32, modifier: f32) -> (u32, f32) {
    let level = (food_level + food).min(20);
    let sat = (saturation + food as f32 * modifier * 2.0).min(level as f32);
    (level, sat)
}

/// The attack damage with a Meka-Tool in hand: `damage` (the player's own) plus the tool's attribute; any other item leaves it.
pub fn hit_damage(stack: &ItemStack, damage: f32) -> f32 {
    if stack.item != meka_tool::MEKA_TOOL {
        return damage;
    }
    let data = meka_tool::host_read_meka_tool(&stack.components);
    damage + data.calculate_attack().0 as f32
}

/// The lines of the MekaSuit's HUD (`MekaSuitHUD`, API 41): for each worn piece its energy as a percent with a bar, and the jetpack's mode when the chest has
/// the unit. Each is `(id, icon, text, fraction)`; a piece that is not worn gives an empty text, which takes its line away.
pub fn suit_hud_lines(pieces: &[Option<mekasuit::MekaSuitItemStack>; 4]) -> Vec<(String, String, String, f32)> {
    let names = ["head", "chest", "legs", "feet"];
    let mut out = Vec::new();
    for (i, piece) in pieces.iter().enumerate() {
        let id = format!("mekanism:suit_{}", names[i]);
        match piece {
            Some(p) => {
                let max = p.max_energy().max(1);
                let fraction = (p.energy as f64 / max as f64).clamp(0.0, 1.0);
                out.push((id, p.item.clone(), format!("Energy: {}%", (fraction * 100.0).round() as u32), (fraction * 100.0).round() as f32 / 100.0));
            }
            None => out.push((id, String::new(), String::new(), -1.0)),
        }
    }
    let jetpack = pieces[mekasuit::ArmorSlot::Chest as usize].as_ref().filter(|b| b.modules.has_module(crate::modules::ModuleType::JetpackUnit));
    out.push(match jetpack {
        Some(b) => {
            let mode = match mekasuit::jetpack_mode_of(b) {
                crate::modules::JetpackMode::Normal => "Normal",
                crate::modules::JetpackMode::Hover => "Hover",
                crate::modules::JetpackMode::Vector => "Vector",
                crate::modules::JetpackMode::Disabled => "Disabled",
            };
            ("mekanism:suit_jetpack".to_owned(), "mekanism:module_jetpack_unit".to_owned(), format!("Jetpack: {mode}"), -1.0)
        }
        None => ("mekanism:suit_jetpack".to_owned(), String::new(), String::new(), -1.0),
    });
    out
}

impl Mekanism {
    /// Sends a player the HUD lines that changed since the last time (`players.set-hud-line`).
    fn suit_hud(&mut self, player: &str, pieces: &[Option<mekasuit::MekaSuitItemStack>; 4]) {
        for (id, icon, text, fraction) in suit_hud_lines(pieces) {
            let key = (player.to_owned(), id.clone());
            let now = (text.clone(), (fraction * 100.0) as i32);
            let was = self.hud_sent.get(&key);
            if was == Some(&now) || (was.is_none() && text.is_empty()) {
                continue;
            }
            if players::set_hud_line(player, &id, &icon, &text, fraction) {
                self.hud_sent.insert(key, now);
            }
        }
    }

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

/// The components with `minecraft:glider` there (the stack glides as an elytra does, `LivingEntity.canGlideUsing`) or taken away.
pub fn set_glider(components: &str, on: bool) -> String {
    let has = crate::config::split_components(components).is_some_and(|e| e.iter().any(|(k, _)| k == "minecraft:glider"));
    if on {
        return if has { components.to_owned() } else { with_component(components, "minecraft:glider", "{}") };
    }
    if !has {
        return components.to_owned();
    }
    let entries = crate::config::split_components(components).unwrap_or_default();
    let kept: Vec<String> = entries.iter().filter(|(k, _)| k != "minecraft:glider").map(|(k, r)| format!("\"{k}\":{}", &components[r.clone()])).collect();
    format!("{{{}}}", kept.join(","))
}

/// The components with `key` set to `value` (SNBT text of the value), the rest byte-for-byte; the key is added when it is not there.
pub fn with_component(components: &str, key: &str, value: &str) -> String {
    let pair = format!("\"{key}\":{value}");
    let trimmed = components.trim();
    if trimmed.is_empty() || trimmed == "{}" {
        return format!("{{{pair}}}");
    }
    if let Some(entries) = crate::config::split_components(components) {
        for (k, range) in &entries {
            if k == key {
                let mut out = components.to_owned();
                out.replace_range(range.clone(), value);
                return out;
            }
        }
        let at = components.rfind('}').unwrap_or(components.len());
        let mut out = components.to_owned();
        out.insert_str(at, &if entries.is_empty() { pair } else { format!(",{pair}") });
        return out;
    }
    components.to_owned()
}

impl Mekanism {
    /// `ItemElectricBow.releaseUsing`: outside creative the string only goes if the bow holds the energy of a shot (1 200 J in fire mode, 120 J otherwise).
    pub(crate) fn electric_bow_released(&mut self, player: &Player, stack: &ItemStack) -> bool {
        if stack.item != electric_bow::ELECTRIC_BOW || player.game_mode == GameMode::Creative {
            return true;
        }
        electric_bow::host_read_electric_bow(&stack.components).can_release(false)
    }

    /// `ItemElectricBow.shoot`: after the arrows exist, outside creative the shot costs its energy, and in fire mode the arrows burn (`Flame` 1: 100 ticks).
    pub(crate) fn electric_bow_shot(&mut self, player: &Player, stack: &ItemStack, projectiles: &[u32]) {
        if stack.item != electric_bow::ELECTRIC_BOW {
            return;
        }
        let data = electric_bow::host_read_electric_bow(&stack.components);
        if data.fire_mode {
            for id in projectiles {
                let _ = entities::set_entity_fire(&player.dimension, *id, 100);
            }
        }
        if player.game_mode == GameMode::Creative || projectiles.is_empty() {
            return;
        }
        let Some((slot, components)) = Self::find_hotbar_slot(&player.id, stack) else { return };
        let Some(written) = with_energy(&components, data.energy.saturating_sub(data.energy_for_shot())) else { return };
        let current = ItemStack { item: stack.item.clone(), count: stack.count, components };
        let replacement = ItemStack { item: stack.item.clone(), count: stack.count, components: written };
        let _ = player_items::compare_and_set_slot(&player.id, slot, Some(&current), Some(&replacement));
    }

    /// The fire mode of the bow (`IAttachmentBasedModeItem.changeMode`, an odd shift toggles it). The original turns it with the mode key and the scroll
    /// of a sneaking player; Android has neither, so sneak + use in the air does it, as it does for the configurator. Says whether it took the use.
    pub(crate) fn electric_bow_mode(&mut self, player: &Player, stack: &ItemStack, pos: Option<BlockPos>) -> bool {
        if stack.item != electric_bow::ELECTRIC_BOW || pos.is_some() || !player_control::get_motion(&player.id).is_some_and(|m| m.sneaking) {
            return false;
        }
        let data = electric_bow::host_read_electric_bow(&stack.components);
        let new_mode = !data.fire_mode;
        let Some((slot, components)) = Self::find_hotbar_slot(&player.id, stack) else { return true };
        let written = with_component(&components, "mekanism:electric_bow_mode", if new_mode { "true" } else { "false" });
        let current = ItemStack { item: stack.item.clone(), count: stack.count, components };
        let replacement = ItemStack { item: stack.item.clone(), count: stack.count, components: written };
        let _ = player_items::compare_and_set_slot(&player.id, slot, Some(&current), Some(&replacement));
        let _ = players::tell_overlay(&player.id, &format!("Fire Mode: {}", if new_mode { "on" } else { "off" }));
        true
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

    #[test]
    fn eating_follows_the_food_data() {
        // 15 food, 2 saturation, 5 eaten at 0.8: 20 food, saturation up by 5 * 0.8 * 2 = 8, cut to the level.
        assert_eq!(eat(15, 2.0, 5, 0.8), (20, 10.0));
        // The level never passes 20 and the saturation never the level.
        assert_eq!(eat(18, 19.0, 5, 0.8), (20, 20.0));
    }

    #[test]
    fn a_component_is_set_in_place_or_added() {
        assert_eq!(with_component("{}", "mekanism:electric_bow_mode", "true"), r#"{"mekanism:electric_bow_mode":true}"#);
        let c = r#"{"mekanism:electric_bow_mode":false,"minecraft:damage":3}"#;
        assert_eq!(with_component(c, "mekanism:electric_bow_mode", "true"), r#"{"mekanism:electric_bow_mode":true,"minecraft:damage":3}"#);
        let d = r#"{"minecraft:damage":3}"#;
        assert_eq!(with_component(d, "mekanism:electric_bow_mode", "true"), r#"{"minecraft:damage":3,"mekanism:electric_bow_mode":true}"#);
    }

    #[test]
    fn the_suit_hud_has_a_line_for_each_piece_worn() {
        let mut helmet = mekasuit::MekaSuitItemStack::new(mekasuit::HELMET).unwrap();
        helmet.energy = helmet.max_energy() / 2;
        let pieces = [Some(helmet), None, None, None];
        let lines = suit_hud_lines(&pieces);
        assert_eq!(lines[0], ("mekanism:suit_head".to_owned(), mekasuit::HELMET.to_owned(), "Energy: 50%".to_owned(), 0.5));
        // a piece not worn and a chest with no jetpack unit give empty lines (which take theirs away)
        assert_eq!(lines[1].2, "");
        assert_eq!(lines[4].0, "mekanism:suit_jetpack");
        assert_eq!(lines[4].2, "");
    }

    #[test]
    fn the_glider_component_comes_and_goes() {
        assert_eq!(set_glider("", true), r#"{"minecraft:glider":{}}"#);
        assert_eq!(set_glider(r#"{"minecraft:damage":3}"#, true), r#"{"minecraft:damage":3,"minecraft:glider":{}}"#);
        assert_eq!(set_glider(r#"{"minecraft:glider":{}}"#, true), r#"{"minecraft:glider":{}}"#);
        assert_eq!(set_glider(r#"{"minecraft:damage":3,"minecraft:glider":{}}"#, false), r#"{"minecraft:damage":3}"#);
        assert_eq!(set_glider(r#"{"minecraft:glider":{}}"#, false), "{}");
        assert_eq!(set_glider(r#"{"minecraft:damage":3}"#, false), r#"{"minecraft:damage":3}"#);
    }
}

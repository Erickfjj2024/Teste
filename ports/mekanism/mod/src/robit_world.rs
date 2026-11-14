//! The Robit in the world (FORK-10): `mekanism:robit`, a mod entity kind (declared in `data/mekanism/horizon/entity_kind/robit.json`) ticked by `entities-tick`. The rules
//! are `robit.rs`'s (`RobitEntity`: the follow goal, the pick-up, the furnace, the energy cost of walking and of being hurt, the skins); this file keeps the entity's state in the
//! mod entity's saved bytes, its 30 slots, its energy, its skin and its name, and turns what the rules decide into the update the host takes.
//!
//! Not here: the original's tabbed screen (the menu is the 27 storage slots as a chest), the path finder (the host walks it in a straight line), and the damage of
//! sources other than a player's blow (`mod-entity-damaged` hears only that).
use super::*;
use crate::robit::{self, RobitEntity, RobitItemData, SecurityMode};
use horizon_mod_sdk::server::{ModEntity, ModEntityUpdate};

pub const KIND: &str = "mekanism:robit";

// ---- the saved bytes ----------------------------------------------------------------------------------------------

struct Writer(Vec<u8>);
impl Writer {
    fn u8(&mut self, v: u8) {
        self.0.push(v);
    }
    fn u32(&mut self, v: u32) {
        self.0.extend(v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.0.extend(v.to_le_bytes());
    }
    fn f64(&mut self, v: f64) {
        self.0.extend(v.to_le_bytes());
    }
    fn str(&mut self, s: &str) {
        self.u32(s.len() as u32);
        self.0.extend(s.as_bytes());
    }
}

struct Reader<'a>(&'a [u8], usize);
impl Reader<'_> {
    fn take(&mut self, n: usize) -> Option<&[u8]> {
        let out = self.0.get(self.1..self.1 + n)?;
        self.1 += n;
        Some(out)
    }
    fn u8(&mut self) -> Option<u8> {
        self.take(1).map(|b| b[0])
    }
    fn u32(&mut self) -> Option<u32> {
        self.take(4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn u64(&mut self) -> Option<u64> {
        self.take(8).map(|b| u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]))
    }
    fn f64(&mut self) -> Option<f64> {
        self.u64().map(f64::from_bits)
    }
    fn str(&mut self) -> Option<String> {
        let n = self.u32()? as usize;
        String::from_utf8(self.take(n)?.to_vec()).ok()
    }
}

/// What the entity keeps that the host's fields do not: the owner, the modes, the home, the furnace's progress, the follow goal and where it was a tick ago. A freshly
/// spawned one (`init`) also carries the energy and the 30 slots of the item it came from, which its first tick moves into the host's fields.
#[derive(Clone, Debug, PartialEq)]
pub struct Saved {
    pub robit: RobitEntity,
    pub init: Option<(u64, Vec<Option<Stack>>)>,
}

fn security_byte(m: SecurityMode) -> u8 {
    match m {
        SecurityMode::Public => 0,
        SecurityMode::Private => 1,
        SecurityMode::Trusted => 2,
    }
}

pub fn encode(s: &Saved) -> Vec<u8> {
    let r = &s.robit;
    let mut w = Writer(vec![1]);
    w.str(&r.owner_uuid);
    w.str(&r.owner_name);
    w.str(r.custom_name.as_deref().unwrap_or(""));
    w.u8(u8::from(r.custom_name.is_some()));
    w.u8(security_byte(r.security_mode));
    w.u8(u8::from(r.following));
    w.u8(u8::from(r.drop_pickup));
    w.u8(u8::from(r.default_skin_manually_selected));
    match &r.home_location {
        Some(h) => {
            w.u8(1);
            w.str(&h.dimension);
            for c in h.pos {
                w.u32(c as u32);
            }
        }
        None => w.u8(0),
    }
    w.u32(r.smelting_progress);
    w.u8(u8::from(r.follow_active));
    w.u32(r.follow_recalc);
    w.u64(r.rng);
    w.u64(r.tick_count);
    w.f64(r.xo);
    w.f64(r.yo);
    w.f64(r.zo);
    match &s.init {
        Some((energy, slots)) => {
            w.u8(1);
            w.u64(*energy);
            w.u8(slots.len() as u8);
            for slot in slots {
                match slot {
                    Some(st) => {
                        w.u8(1);
                        w.str(&st.item);
                        w.u32(st.count);
                        w.str(&st.components);
                    }
                    None => w.u8(0),
                }
            }
        }
        None => w.u8(0),
    }
    w.0
}

/// The state in `data`, over `base` (the entity as the host has it: its place, health, energy, skin, name and slots).
pub fn decode(data: &[u8], base: RobitEntity) -> Option<Saved> {
    let mut r = Reader(data, 0);
    if r.u8()? != 1 {
        return None;
    }
    let mut robit = base;
    robit.owner_uuid = r.str()?;
    robit.owner_name = r.str()?;
    let name = r.str()?;
    robit.custom_name = (r.u8()? == 1).then_some(name);
    robit.security_mode = match r.u8()? {
        1 => SecurityMode::Private,
        2 => SecurityMode::Trusted,
        _ => SecurityMode::Public,
    };
    robit.following = r.u8()? == 1;
    robit.drop_pickup = r.u8()? == 1;
    robit.default_skin_manually_selected = r.u8()? == 1;
    robit.home_location = if r.u8()? == 1 {
        let dimension = r.str()?;
        let pos = [r.u32()? as i32, r.u32()? as i32, r.u32()? as i32];
        Some(robit::GlobalPos { dimension, pos })
    } else {
        None
    };
    robit.smelting_progress = r.u32()?;
    robit.follow_active = r.u8()? == 1;
    robit.follow_recalc = r.u32()?;
    robit.rng = r.u64()?;
    robit.tick_count = r.u64()?;
    robit.xo = r.f64()?;
    robit.yo = r.f64()?;
    robit.zo = r.f64()?;
    let init = if r.u8()? == 1 {
        let energy = r.u64()?;
        let n = r.u8()? as usize;
        let mut slots = Vec::with_capacity(n);
        for _ in 0..n {
            slots.push(if r.u8()? == 1 { Some(Stack { item: r.str()?, count: r.u32()?, components: r.str()? }) } else { None });
        }
        Some((energy, slots))
    } else {
        None
    };
    Some(Saved { robit, init })
}

// ---- the item ---------------------------------------------------------------------------------------------------

/// The skin id the kind knows (`mekanism:pride`) for a skin of `robit.rs` (the same id).
fn skin_id(skin: &str) -> String {
    skin.to_owned()
}

/// The components of the Robit item: what `ItemRobit` keeps (energy, name, skin, owner, security and the 30 slots) as `minecraft:custom_data` (any item may carry it).
pub fn item_components(d: &RobitItemData) -> String {
    let esc = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"");
    let items: Vec<String> = d
        .items
        .iter()
        .enumerate()
        .filter_map(|(i, s)| s.as_ref().map(|s| format!("{{slot:{i},id:\"{}\",count:{}}}", esc(&s.item), s.count)))
        .collect();
    format!(
        "{{\"minecraft:custom_data\":{{robit:{{energy:{}L,skin:\"{}\",owner:\"{}\",security:{}b,manual:{}b,name:\"{}\",items:[{}]}}}}}}",
        d.energy,
        esc(&d.skin),
        esc(&d.owner_uuid),
        security_byte(d.security_mode),
        u8::from(d.default_manually_selected),
        esc(d.custom_name.as_deref().unwrap_or("")),
        items.join(",")
    )
}

fn between<'a>(text: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let from = text.find(start)? + start.len();
    let to = text[from..].find(end)? + from;
    Some(&text[from..to])
}

/// The item's data back (`item_components` read again); a Robit item with none is a fresh one.
pub fn read_item_components(components: &str) -> RobitItemData {
    let mut d = RobitItemData {
        energy: robit::MAX_ENERGY_FE,
        custom_name: None,
        skin: robit::SKIN_BASE.to_owned(),
        owner_uuid: String::new(),
        security_mode: SecurityMode::Public,
        default_manually_selected: false,
        items: vec![None; robit::TOTAL_SLOTS],
    };
    let Some(body) = between(components, "robit:{", "items:[") else { return d };
    if let Some(v) = between(body, "energy:", "L,").and_then(|v| v.parse().ok()) {
        d.energy = v;
    }
    if let Some(v) = between(body, "skin:\"", "\"") {
        d.skin = v.to_owned();
    }
    d.owner_uuid = between(body, "owner:\"", "\"").unwrap_or("").to_owned();
    d.security_mode = match between(body, "security:", "b") {
        Some("1") => SecurityMode::Private,
        Some("2") => SecurityMode::Trusted,
        _ => SecurityMode::Public,
    };
    d.default_manually_selected = between(body, "manual:", "b") == Some("1");
    d.custom_name = between(body, "name:\"", "\"").filter(|n| !n.is_empty()).map(str::to_owned);
    if let Some(list) = components.split("items:[").nth(1) {
        for entry in list.split("{slot:").skip(1) {
            let slot = entry.split(',').next().and_then(|v| v.parse::<usize>().ok());
            let id = between(entry, "id:\"", "\"");
            let count = between(entry, "count:", "}").and_then(|v| v.parse::<u32>().ok());
            if let (Some(slot), Some(id), Some(count)) = (slot, id, count) {
                if slot < d.items.len() {
                    d.items[slot] = Some(Stack { item: id.to_owned(), count, components: String::new() });
                }
            }
        }
    }
    d
}

// ---- the menu -----------------------------------------------------------------------------------------------------

/// The numbers and strings of the Robit's screens (`GuiRobitMain`): the numbers 0 follows, 1 picks up, 2 its energy, 3 the furnace's progress in percent; the strings 0 the greeting,
/// 1 the energy, 2 following, 3 pickup, 4 the owner, 5 its own name (the box of the rename window starts with it).
pub fn menu_of(r: &RobitEntity) -> (Vec<i32>, Vec<String>) {
    let yes = |b: bool| if b { "Yes" } else { "No" };
    let progress = (u64::from(r.smelting_progress) * 100 / u64::from(robit::SMELTING_TICKS_REQUIRED)).min(100) as i32;
    (
        vec![i32::from(r.following), i32::from(r.drop_pickup), r.energy.min(40_000) as i32, progress],
        vec![
            format!("Hi, {}!", r.display_name()),
            format!("Energy: {}", crate::laser::format_energy(r.energy)),
            format!("Following: {}", yes(r.following)),
            format!("Drop Pickup: {}", yes(r.drop_pickup)),
            format!("Owner: {}", r.owner_name),
            r.custom_name.clone().unwrap_or_default(),
        ],
    )
}

/// The crafting grid's cells (slots 30 to 38 of the menu, past the container's) of the entity as the menu shows it, when it is open.
fn grid_of(items: &[Option<ItemStack>]) -> Option<Vec<Option<ItemStack>>> {
    (items.len() > robit::TOTAL_SLOTS + 9).then(|| items[robit::TOTAL_SLOTS..robit::TOTAL_SLOTS + 9].to_vec())
}

/// The slot of the crafting result in the menu.
pub const CRAFT_RESULT: usize = robit::TOTAL_SLOTS + 9;
/// The repair tab's slots (`GuiRobitRepair`): the item, the material or book, and the result (the host's anvil, API 52).
pub const REPAIR_LEFT: usize = CRAFT_RESULT + 1;
pub const REPAIR_RIGHT: usize = CRAFT_RESULT + 2;
pub const REPAIR_OUT: usize = CRAFT_RESULT + 3;
/// The id of the repair tab's name box (the rename window's is 10).
pub const REPAIR_NAME_BOX: i32 = 11;

thread_local! {
    /// What each Robit's repair name box holds (`AnvilMenu.itemName`).
    static REPAIR_NAMES: std::cell::RefCell<HashMap<u32, String>> = std::cell::RefCell::new(HashMap::new());
}

pub fn set_repair_name(entity: u32, name: String) {
    REPAIR_NAMES.with(|m| m.borrow_mut().insert(entity, name));
}

/// Total experience points of a level (`Player.getXpNeededForNextLevel` summed).
pub fn xp_total(level: i32) -> i64 {
    let l = i64::from(level.max(0));
    if l <= 16 {
        l * l + 6 * l
    } else if l <= 31 {
        (5 * l * l - 81 * l + 720) / 2
    } else {
        (9 * l * l - 325 * l + 4440) / 2
    }
}

/// The anvil's answer for the repair slots and what it costs: the result stack, the level cost and the right stack's remaining material. `None` when nothing can be made, the item is
/// too expensive (40 or more levels) or the owner has too few levels.
pub fn repair_answer(items: &[Option<ItemStack>], name: Option<&str>, owner_level: i32) -> Option<(ItemStack, i32, u32)> {
    let left = items.get(REPAIR_LEFT)?.as_ref()?;
    let right = items.get(REPAIR_RIGHT).and_then(|s| s.as_ref());
    let answer = recipe_types::anvil_result(left, right, name)?;
    (answer.cost < 40 && answer.cost <= owner_level).then_some((answer.output, answer.cost, answer.material_cost))
}

// ---- the world ----------------------------------------------------------------------------------------------------

fn base_of(e: &ModEntity) -> RobitEntity {
    let mut r = RobitEntity::new(e.id, e.dimension.clone(), e.x, e.y, e.z, String::new(), String::new());
    r.yaw = e.yaw;
    r.health = e.health;
    r.energy = e.energy;
    r.skin = if e.skin.is_empty() { robit::SKIN_BASE.to_owned() } else { e.skin.clone() };
    r.slots = (0..robit::TOTAL_SLOTS)
        .map(|i| e.items.get(i).cloned().flatten().map(|s| Stack { item: s.item, count: s.count, components: s.components }))
        .collect();
    r.xo = e.x;
    r.yo = e.y;
    r.zo = e.z;
    r
}

/// The Robit a mod entity is.
pub fn robit_of(e: &ModEntity) -> Option<Saved> {
    let mut saved = decode(&e.data, base_of(e))?;
    if let Some((energy, slots)) = saved.init.clone() {
        saved.robit.energy = energy;
        saved.robit.slots = slots;
    }
    Some(saved)
}

fn item_slots(slots: &[Option<Stack>]) -> Vec<Option<ItemStack>> {
    slots.iter().map(|s| s.as_ref().map(|s| ItemStack { item: s.item.clone(), count: s.count, components: s.components.clone() })).collect()
}

/// Whether a Robit may stand at a block (`canTeleportTo`): the block and the one above it have no collision and the one below has.
fn can_stand(dimension: &str, pos: [i32; 3]) -> bool {
    let free = |y: i32| world::get_collision(dimension, pos[0], y, pos[2]).is_some_and(|c| c.is_empty());
    let solid = world::get_collision(dimension, pos[0], pos[1] - 1, pos[2]).is_some_and(|c| !c.is_empty());
    free(pos[1]) && free(pos[1] + 1) && solid
}

fn is_chargepad(dimension: &str, pos: [i32; 3]) -> bool {
    world::get_block(dimension, pos[0], pos[1], pos[2]).is_some_and(|b| b.split('[').next() == Some(crate::chargepad::BLOCK))
}

fn drop_stack(dimension: &str, x: f64, y: f64, z: f64, s: &Stack) {
    let nbt = format!("{{Item:{{id:\"{}\",Count:{}b}}}}", s.item, s.count);
    let _ = entities::summon(dimension, "minecraft:item", x, y, z, &nbt);
}

/// The tick of every Robit (`EntityRobit.tick`, `RobitAIFollow`, `RobitAIPickup`, `baseTick`).
pub fn tick_all(damage: &mut HashMap<u32, f64>, renames: &mut HashMap<u32, String>, recipes: &HashMap<String, (String, u32)>, list: Vec<ModEntity>) -> Vec<ModEntityUpdate> {
    let online = players::online();
    let mut out = Vec::new();
    for e in list.into_iter().filter(|e| e.kind == KIND) {
        let Some(mut saved) = robit_of(&e) else {
            // no saved state: not one of ours to keep
            out.push(update(&e, |u| u.remove = true));
            continue;
        };
        let was_fresh = saved.init.is_some();
        let r = &mut saved.robit;
        if let Some(d) = damage.remove(&e.id) {
            r.take_damage(d);
        }
        if let Some(name) = renames.remove(&e.id) {
            r.custom_name = Some(name).filter(|n| !n.trim().is_empty());
        }
        // the owner: in the same dimension and not a spectator
        let owner = online
            .iter()
            .find(|p| p.id == r.owner_uuid && p.game_mode != GameMode::Spectator)
            .map(|p| (p.x, p.y, p.z, p.dimension.clone()));
        let owner_pos = owner.as_ref().map(|(x, y, z, d)| (*x, *y, *z, d.as_str()));
        // the items lying within 10 blocks (`RobitAIPickup`'s search), for the pick-up
        let mut world_items = Vec::new();
        let mut ids = Vec::new();
        if r.drop_pickup {
            let rad = robit::PICKUP_SEARCH_RADIUS;
            for it in entities::items_in_box(&e.dimension, (e.x - rad, e.y - rad, e.z - rad), (e.x + rad, e.y + rad, e.z + rad)) {
                world_items.push(robit::WorldItemEntity {
                    id: it.id,
                    x: it.x,
                    y: it.y,
                    z: it.z,
                    item: Stack { item: it.stack.item.clone(), count: it.stack.count, components: it.stack.components.clone() },
                    pickup_delay: 0,
                    is_alive: true,
                });
                ids.push(it.stack.count);
            }
        }
        let before_items = world_items.clone();
        let (px, py, pz) = (r.x, r.y, r.z);
        let on_pad = is_chargepad(&e.dimension, [e.x.floor() as i32, (e.y - 0.1).floor() as i32, e.z.floor() as i32]);
        let dim = e.dimension.clone();
        r.tick(owner_pos, &mut world_items, recipes, on_pad, &|p| can_stand(&dim, p));
        // what it took from the ground
        for (after, before) in world_items.iter().zip(&before_items) {
            if after.item.count != before.item.count || !after.is_alive {
                let taken = before.item.count - if after.is_alive { after.item.count } else { 0 };
                if taken > 0 {
                    let _ = entities::take_dropped_item(&e.dimension, after.id, taken);
                }
            }
        }
        let moved = ((r.x - px).powi(2) + (r.y - py).powi(2) + (r.z - pz).powi(2)).sqrt();
        let (nx, ny, nz) = (r.x, r.y, r.z);
        r.x = px;
        r.y = py;
        r.z = pz;
        // the home: none and it goes; the pad gone and it drops as an item (`EntityRobit.tick`)
        let (home_loaded, pad_at_home) = match &r.home_location {
            Some(h) => (world::get_block(&h.dimension, h.pos[0], h.pos[1], h.pos[2]).is_some(), is_chargepad(&h.dimension, h.pos)),
            None => (false, false),
        };
        let verdict = r.home_check(home_loaded, pad_at_home);
        let name_now = r.custom_name.clone();
        let slots_now = r.slots.clone();
        let (energy, skin) = (r.energy, skin_id(&r.skin));
        r.xo = e.x;
        r.yo = e.y;
        r.zo = e.z;
        let rr = r.clone();
        let following = rr.follow_active && owner.is_some();
        let goal = owner.map(|(x, y, z, _)| (x, y, z));
        let (menu_data, menu_text) = menu_of(&rr);
        let rr_owner = rr.owner_uuid.clone();
        let mut items = item_slots(&slots_now);
        // While the menu is open the virtual slots are there too: the crafting grid makes its result.
        if e.items.len() > robit::TOTAL_SLOTS {
            items.extend(e.items[robit::TOTAL_SLOTS..].iter().cloned());
            if let Some(grid) = grid_of(&items) {
                items[CRAFT_RESULT] = recipe_types::crafting_find(&grid).map(|m| m.output);
            }
            // The repair tab: the anvil's result for the owner's levels.
            if items.len() > REPAIR_OUT {
                let level = player_control::get_motion(&rr_owner).map_or(0, |m| m.xp_level);
                let name = REPAIR_NAMES.with(|m| m.borrow().get(&e.id).cloned());
                items[REPAIR_OUT] = repair_answer(&items, name.as_deref(), level).map(|a| a.0);
            }
        }
        let mut u = update(&e, |u| {
            u.menu_data = Some(menu_data);
            u.menu_text = Some(menu_text);
            u.energy = Some(energy);
            u.items = Some(items);
            u.skin = Some(skin);
            u.name = Some(name_now);
        });
        match verdict {
            robit::HomeCheck::Stay => {}
            robit::HomeCheck::Discard => u.remove = true,
            robit::HomeCheck::DropAndDiscard => {
                let item = Stack { item: robit::ITEM_ID.to_owned(), count: 1, components: item_components(&robit::pack_robit_to_item(&rr)) };
                drop_stack(&e.dimension, e.x, e.y + 0.5, e.z, &item);
                u.remove = true;
            }
        }
        saved.robit = rr;
        saved.init = None;
        u.data = Some(encode(&saved));
        if !u.remove {
            if moved > 1.0 {
                u.teleport = Some((nx, ny, nz));
            } else if following && let Some((gx, gy, gz)) = goal {
                u.path_to = Some((gx, gy, gz, 1.0));
            } else if moved == 0.0 && was_fresh {
                u.stop = true;
            }
        }
        out.push(u);
    }
    out
}

fn update(e: &ModEntity, f: impl FnOnce(&mut ModEntityUpdate)) -> ModEntityUpdate {
    let mut u = ModEntityUpdate {
        dimension: e.dimension.clone(),
        id: e.id,
        data: None,
        items: None,
        energy: None,
        skin: None,
        name: None,
        teleport: None,
        path_to: None,
        stop: false,
        health: Some(robit::MAX_HEALTH),
        remove: false,
        menu_data: None,
        menu_text: None,
    };
    f(&mut u);
    u
}

impl Mekanism {
    /// `ItemRobit.useOn`: the item used on a Chargepad that is not in use puts the Robit on it, with what the item holds; it is used up outside creative.
    pub(crate) fn robit_placed(&mut self, player: &Player, stack: &ItemStack, pos: Option<BlockPos>) -> bool {
        if stack.item != robit::ITEM_ID {
            return false;
        }
        let Some(at) = pos else { return false };
        let Some(block) = world::get_block(&player.dimension, at.x, at.y, at.z) else { return false };
        let exists = block.split('[').next() == Some(crate::chargepad::BLOCK);
        if !robit::can_place_on_chargepad(exists, block.contains("active=true")) {
            return false;
        }
        let item = read_item_components(&stack.components);
        let mut r = robit::spawn_robit_from_item(item.clone(), 0, player.dimension.clone(), [at.x, at.y + 1, at.z], player.id.clone(), player.name.clone(), true);
        // the home is the pad itself
        r.home_location = Some(robit::GlobalPos { dimension: player.dimension.clone(), pos: [at.x, at.y, at.z] });
        let saved = Saved { init: Some((r.energy, r.slots.clone())), robit: r.clone() };
        let ok = entities::spawn_mod_entity(&player.dimension, KIND, r.x, r.y, r.z, &encode(&saved)).is_ok();
        if ok && player.game_mode != GameMode::Creative
            && let Some((slot, _)) = Self::find_hotbar_slot(&player.id, stack)
        {
            let rest = (stack.count > 1).then(|| ItemStack { count: stack.count - 1, ..stack.clone() });
            let _ = player_items::compare_and_set_slot(&player.id, slot, Some(stack), rest.as_ref());
        }
        ok
    }

    /// A click on a Robit (`EntityRobit.interact`): sneaking takes it back as an item; otherwise its storage opens as a chest. A player that may not access it
    /// (private or trusted-only) is told so.
    pub(crate) fn robit_used(&mut self, player: &Player, e: &ModEntity, sneaking: bool) -> bool {
        let Some(saved) = robit_of(e) else { return false };
        let r = &saved.robit;
        if !r.can_player_access(&player.id, false) {
            let _ = players::tell_overlay(&player.id, "This Robit is private");
            return true;
        }
        if sneaking {
            // Only with a configurator in hand (`ItemConfigurator`): any other sneak click passes.
            let has_configurator = player_items::get_inventory(&player.id).is_some_and(|inv| inv.iter().take(9).flatten().any(|s| s.item == crate::CONFIGURATOR));
            if !has_configurator {
                return false;
            }
            let item = Stack { item: robit::ITEM_ID.to_owned(), count: 1, components: item_components(&robit::pack_robit_to_item(r)) };
            if entities::remove_entity(&e.dimension, e.id).is_ok() {
                drop_stack(&e.dimension, player.x, player.y + 0.5, player.z, &item);
            }
            return true;
        }
        players::open_entity_menu(&player.id, &e.dimension, e.id)
    }

    /// A button of the Robit's screens (`entity-menu-button`): 1 follow, 2 pick up, 3 go home, 20 and up a skin of the list. A player that may not access it does nothing.
    pub(crate) fn robit_button(&mut self, player: &Player, e: &ModEntity, id: i32) -> Option<ModEntityUpdate> {
        let mut saved = robit_of(e)?;
        if !saved.robit.can_player_access(&player.id, false) {
            return None;
        }
        let r = &mut saved.robit;
        let mut teleport = None;
        match id {
            1 => r.following = !r.following,
            2 => r.drop_pickup = !r.drop_pickup,
            3 => {
                if r.go_home() {
                    teleport = Some((r.x, r.y, r.z));
                }
            }
            20..=40 => {
                let skin = robit::ALL_SKINS.get((id - 20) as usize)?;
                if !r.set_skin(skin.id, true) {
                    return None;
                }
            }
            _ => return None,
        }
        let (menu_data, menu_text) = menu_of(r);
        let skin = skin_id(&r.skin);
        let mut saved_now = saved.clone();
        saved_now.init = None;
        let mut u = update(e, |u| {
            u.data = Some(encode(&saved_now));
            u.menu_data = Some(menu_data);
            u.menu_text = Some(menu_text);
            u.skin = Some(skin);
            u.teleport = teleport;
            u.health = None;
        });
        if teleport.is_some() {
            u.stop = true;
        }
        Some(u)
    }

    /// The crafting result taken (`entity-menu-taken`): each cell of the grid gives up one item, and what the recipe leaves behind (a bucket, a bottle) takes the place of a cell that
    /// was emptied.
    pub(crate) fn robit_taken(&mut self, player: &Player, e: &ModEntity, slot: u32) -> Option<ModEntityUpdate> {
        if slot as usize == REPAIR_OUT {
            // The anvil's result was taken: the inputs go (the item, and what the repair used of the material) and the owner pays the levels.
            let level = player_control::get_motion(&player.id).map_or(0, |m| m.xp_level);
            let name = REPAIR_NAMES.with(|m| m.borrow().get(&e.id).cloned());
            let (_, cost, material) = repair_answer(&e.items, name.as_deref(), level)?;
            let mut items = e.items.clone();
            items[REPAIR_LEFT] = None;
            items[REPAIR_RIGHT] = items[REPAIR_RIGHT].take().and_then(|s| {
                let left = s.count.saturating_sub(material.max(1));
                (left > 0).then_some(ItemStack { count: left, ..s })
            });
            items[REPAIR_OUT] = None;
            let points = (xp_total(level) - xp_total(level - cost)).max(0) as u32;
            if points > 0 {
                let _ = player_control::take_xp(&player.id, points);
            }
            return Some(update(e, |u| {
                u.items = Some(items);
                u.health = None;
            }));
        }
        if slot as usize != CRAFT_RESULT {
            return None;
        }
        let grid = grid_of(&e.items)?;
        let found = recipe_types::crafting_find(&grid)?;
        let mut items = e.items.clone();
        for (i, cell) in grid.iter().enumerate() {
            let at = robit::TOTAL_SLOTS + i;
            if let Some(stack) = cell {
                items[at] = if stack.count > 1 {
                    Some(ItemStack { count: stack.count - 1, ..stack.clone() })
                } else {
                    found.remaining.get(i).cloned().flatten()
                };
            }
        }
        items[CRAFT_RESULT] = None;
        Some(update(e, |u| {
            u.items = Some(items);
            u.health = None;
        }))
    }

    /// A blow on a Robit (`onDamageTaken`): the damage is paid in energy at the next tick and the Robit stays whole.
    pub(crate) fn robit_hurt(&mut self, e: &ModEntity, amount: f32) -> f32 {
        if e.kind != KIND {
            return amount;
        }
        *self.robit_damage.entry(e.id).or_default() += f64::from(amount);
        0.0
    }

    /// What a Robit that dies leaves: its contents on the ground and itself as an item (`EntityRobit.dropContents` and `die`).
    pub(crate) fn robit_died(&mut self, e: &ModEntity) {
        if e.kind != KIND {
            return;
        }
        let Some(saved) = robit_of(e) else { return };
        for s in saved.robit.slots.iter().flatten() {
            drop_stack(&e.dimension, e.x, e.y + 0.3, e.z, s);
        }
        let item = Stack { item: robit::ITEM_ID.to_owned(), count: 1, components: item_components(&robit::pack_robit_to_item(&saved.robit)) };
        drop_stack(&e.dimension, e.x, e.y + 0.3, e.z, &item);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> RobitEntity {
        let mut r = RobitEntity::new(7, "minecraft:overworld".into(), 1.5, 64.0, 2.5, "owner-1".into(), "Alex".into());
        r.custom_name = Some("Bolt".into());
        r.security_mode = SecurityMode::Trusted;
        r.following = true;
        r.drop_pickup = true;
        r.home_location = Some(robit::GlobalPos { dimension: "minecraft:overworld".into(), pos: [1, 63, 2] });
        r.smelting_progress = 40;
        r.follow_active = true;
        r.follow_recalc = 3;
        r.tick_count = 99;
        r.xo = 1.0;
        r
    }

    #[test]
    fn the_robit_survives_its_saved_bytes() {
        let r = sample();
        let saved = Saved { robit: r.clone(), init: Some((12_345, vec![Some(Stack { item: "minecraft:coal".into(), count: 5, components: String::new() }), None])) };
        let back = decode(&encode(&saved), base_of_fields(&r)).unwrap();
        assert_eq!(back.robit.owner_uuid, "owner-1");
        assert_eq!(back.robit.custom_name.as_deref(), Some("Bolt"));
        assert_eq!(back.robit.security_mode, SecurityMode::Trusted);
        assert!(back.robit.following && back.robit.drop_pickup && back.robit.follow_active);
        assert_eq!(back.robit.home_location, r.home_location);
        assert_eq!((back.robit.smelting_progress, back.robit.follow_recalc, back.robit.tick_count), (40, 3, 99));
        assert_eq!(back.init, saved.init);
        assert!(decode(&[], base_of_fields(&r)).is_none());
        assert!(decode(&encode(&saved)[..10], base_of_fields(&r)).is_none());
    }

    fn base_of_fields(r: &RobitEntity) -> RobitEntity {
        RobitEntity::new(r.id, r.dimension.clone(), r.x, r.y, r.z, String::new(), String::new())
    }

    #[test]
    fn the_item_keeps_what_the_robit_held() {
        let mut d = read_item_components("");
        assert_eq!(d.energy, robit::MAX_ENERGY_FE);
        d.energy = 777;
        d.custom_name = Some("Bolt".into());
        d.skin = "mekanism:pride".into();
        d.owner_uuid = "owner-1".into();
        d.security_mode = SecurityMode::Private;
        d.items[3] = Some(Stack { item: "minecraft:coal".into(), count: 12, components: String::new() });
        d.items[29] = Some(Stack { item: "minecraft:iron_ingot".into(), count: 1, components: String::new() });
        let back = read_item_components(&item_components(&d));
        assert_eq!(back.energy, 777);
        assert_eq!(back.custom_name.as_deref(), Some("Bolt"));
        assert_eq!(back.skin, "mekanism:pride");
        assert_eq!(back.owner_uuid, "owner-1");
        assert_eq!(back.security_mode, SecurityMode::Private);
        assert_eq!(back.items, d.items);
    }
}

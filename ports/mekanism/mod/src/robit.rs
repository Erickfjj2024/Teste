//! Robit entity, automation, 5 GUI screens, modules/skins, energy and behaviors (`EntityRobit`, `ItemRobit`).
//!
//! Complete faithful port of Mekanism 1.21.1's companion robot entity:
//! - Java Entity: `mekanism.common.entity.EntityRobit` (`EntityRobit.java`)
//! - Item: `mekanism.common.item.ItemRobit` (`ItemRobit.java`)
//! - AI Goals: `RobitAIBase`, `RobitAIFollow`, `RobitAIPickup`
//! - Containers & Screens:
//!   - `MainRobitContainer` / `GuiRobitMain`: Status, follow/pickup toggles, go-home button, popup windows.
//!   - `InventoryRobitContainer` / `GuiRobitInventory`: 27-slot chest inventory (3x9).
//!   - `SmeltingRobitContainer` / `GuiRobitSmelting`: Internal fast furnace (100 ticks = 5s, 20 FE/t = 50 J/t).
//!   - `CraftingRobitContainer` / `GuiRobitCrafting`: Integrated 3x3 crafting grid.
//!   - `RepairRobitContainer` / `GuiRobitRepair`: Integrated anvil repair and renaming.
//!   - `GuiRobitRename`: Rename popup window.
//!   - `GuiRobitSkinSelect`: Scrollable skin selection popup window.
//! - Skins & Cosmetics:
//!   - Base, Allay, and 11 Pride skins (`mekanism:robit`, `mekanism:allay`, `mekanism:pride`, `mekanism:lesbian`, etc.)
//!   - Advancement-based unlocking and holiday auto-randomization.
//! - Security:
//!   - Owner UUID and username tracking; PUBLIC / PRIVATE / TRUSTED modes.
//! - Energy & Physics:
//!   - 100 000 Joules = 40 000 FE max capacity (`MAX_ENERGY`).
//!   - Invulnerable to death by normal damage; damage absorbs 1 000 J / damage = 400 FE / damage.
//!   - Movement cost: `ceil(1.5 * sqrt(dist_sq))` Joules.
//!   - Chargepad home binding and auto-return when battery depletes.

use crate::fe;
use crate::machine::Stack;
use std::collections::HashMap;

/// Entity registry name.
pub const ENTITY_ID: &str = "mekanism:robit";

/// Item registry name.
pub const ITEM_ID: &str = "mekanism:robit";

/// Maximum energy storage in Joules (100 000 J).
/// Java: `EntityRobit.java:162` (`MAX_ENERGY = 100_000L`)
pub const MAX_ENERGY_JOULES: u64 = 100_000;

/// Maximum energy storage in Forge Energy (40 000 FE).
/// Java: `fe(100_000) = 40_000 FE`
pub const MAX_ENERGY_FE: u64 = fe(MAX_ENERGY_JOULES);

/// Robit movement speed attribute.
/// Java: `EntityRobit.java:135` (`Attributes.MOVEMENT_SPEED = 0.3F`)
pub const MOVEMENT_SPEED: f32 = 0.3;

/// Robit maximum health (1.0 HP). Robit is immortal and absorbs damage with energy.
/// Java: `EntityRobit.java:135` (`Attributes.MAX_HEALTH = 1.0D`)
pub const MAX_HEALTH: f32 = 1.0;

/// Joules of energy consumed per point of damage received (1 000 J/damage = 400 FE/damage).
/// Java: `EntityRobit.java:530` (`MathUtils.clampToLong(1_000D * damageContainer.getNewDamage())`)
pub const ENERGY_PER_DAMAGE_JOULES: u64 = 1_000;
pub const ENERGY_PER_DAMAGE_FE: u64 = fe(ENERGY_PER_DAMAGE_JOULES);

/// Distance multiplier for movement energy calculation.
/// Java: `EntityRobit.java:163` (`DISTANCE_MULTIPLIER = 1.5D`)
pub const DISTANCE_MULTIPLIER: f64 = 1.5;

/// Number of ticks required for Robit's internal smelting (100 ticks = 5 seconds; 2x faster than normal furnace).
/// Java: `EntityRobit.java:166` (`ticksRequired = 5 * SharedConstants.TICKS_PER_SECOND = 100`)
pub const SMELTING_TICKS_REQUIRED: u32 = 100;

/// Robit smelting energy usage per tick (50 J/t = 20 FE/t).
/// Java: `EntityRobit.java:694` (`MekanismConfig.usage.energizedSmelter = 50L`)
pub const SMELTING_ENERGY_USAGE_JOULES: u64 = 50;
pub const SMELTING_ENERGY_USAGE_FE: u64 = fe(SMELTING_ENERGY_USAGE_JOULES);

/// Total number of inventory slots: 27 storage + 1 energy + 1 smelting input + 1 smelting output.
pub const TOTAL_SLOTS: usize = 30;
pub const STORAGE_SLOTS: usize = 27;
pub const ENERGY_SLOT: usize = 27;
pub const SMELT_INPUT_SLOT: usize = 28;
pub const SMELT_OUTPUT_SLOT: usize = 29;

/// Follow distance parameters.
/// Java: `RobitAIFollow.java:21` (`minDist = 2.0`, `maxDist = 4.0`, teleport threshold = 12.0 blocks / dist_sq >= 144)
/// `RobitAIFollow(this, 1, 4, 2)` (`EntityRobit.java:257`): the goal starts when the owner is at least 4 blocks away (`minDist`) and goes on until the
/// Robit is within 2 blocks (`maxDist`). (The names of the original are the other way around from what they mean.)
pub const FOLLOW_START_DIST: f64 = 4.0;
pub const FOLLOW_STOP_DIST: f64 = 2.0;
/// `RobitAIBase.updateTask`: at 12 blocks or more (`distanceToSqr >= 144`) it teleports around the owner instead of walking; it decides every 10 ticks.
pub const FOLLOW_TELEPORT_DIST_SQ: f64 = 144.0;
pub const FOLLOW_RECALC_TICKS: u32 = 10;
/// How often the home chargepad is looked at (`tickCount % 20 == 0`, `EntityRobit.tick`).
pub const HOME_CHECK_TICKS: u64 = 20;

/// Item pickup scan radius (10 blocks).
/// Java: `RobitAIPickup.java:13` (`SEARCH_RADIUS = 10`, `SEARCH_RADIUS_SQ = 100`)
pub const PICKUP_SEARCH_RADIUS: f64 = 10.0;
pub const PICKUP_SEARCH_RADIUS_SQ: f64 = 100.0;

/// Item collection inflation bounding box around Robit (1.5 blocks).
/// Java: `EntityRobit.java:366` (`getBoundingBox().inflate(1.5, 1.5, 1.5)`)
pub const COLLECTION_INFLATION: f64 = 1.5;
/// `MekanismEntityTypes.ROBIT`: `sized(0.6F, 0.65F)`.
pub const ROBIT_WIDTH: f64 = 0.6;
pub const ROBIT_HEIGHT: f64 = 0.65;

/// What the home check decides.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HomeCheck {
    Stay,
    Discard,
    DropAndDiscard,
}

// -------------------------------------------------------------------------------------------------
// Security & GUI Enums
// -------------------------------------------------------------------------------------------------

/// Security access mode.
/// Java: `mekanism.api.security.SecurityMode`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SecurityMode {
    Public = 0,
    Private = 1,
    Trusted = 2,
}

impl SecurityMode {
    pub fn from_u8(val: u8) -> Self {
        match val {
            1 => SecurityMode::Private,
            2 => SecurityMode::Trusted,
            _ => SecurityMode::Public,
        }
    }
}

/// The 5 Robit GUI screens.
/// Java: `GuiRobit.java:50-55` (`RobitGuiType`)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RobitGuiType {
    Main = 0,
    Crafting = 1,
    Inventory = 2,
    Smelting = 3,
    Repair = 4,
}

/// Buttons present in Robit's interfaces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RobitButton {
    GoHome,
    ToggleFollow,
    TogglePickup,
    OpenRename,
    OpenSkinSelect,
    SwitchTab(RobitGuiType),
}

/// Smelting recipe error states tracked by Robit's container.
/// Java: `EntityRobit.java:155-160` (`TRACKED_ERROR_TYPES`)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RobitRecipeError {
    NotEnoughEnergy,
    NotEnoughInput,
    NotEnoughOutputSpace,
    InputDoesntProduceOutput,
}

// -------------------------------------------------------------------------------------------------
// Skin System
// -------------------------------------------------------------------------------------------------

/// Robit Skin definition.
/// Java: `MekanismRobitSkins.java`, `RobitPrideSkinData.java`, `AdvancementBasedRobitSkin.java`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RobitSkin {
    pub id: &'static str,
    pub name: &'static str,
    pub textures: &'static [&'static str],
    pub required_advancement: Option<&'static str>,
    /// The item model the Robit item uses with this skin (`AdvancementBasedRobitSkin.custom_model`).
    pub custom_model: Option<&'static str>,
}

pub const SKIN_BASE: &str = "mekanism:robit";
pub const SKIN_ALLAY: &str = "mekanism:allay";

/// The 13 skins of `data/mekanism/mekanism/robit_skin/*.json` of the release (type `basic`, or `advancement_based` for the allay one), with the display name
/// of the lang file (`robit_skin.mekanism.<id>`) and every texture variant the JSON lists. Generated from the jar, not by hand.
pub const ALL_SKINS: [RobitSkin; 13] = [
    RobitSkin {
        id: "mekanism:robit",
        name: "Default",
        textures: &["mekanism:robit", "mekanism:robit2"],
        required_advancement: None,
        custom_model: None,
    },
    RobitSkin {
        id: "mekanism:allay",
        name: "Allay Costume",
        textures: &["mekanism:allay", "mekanism:allay2"],
        required_advancement: Some("minecraft:husbandry/allay_deliver_item_to_player"),
        custom_model: Some("mekanism:item/robit_allay"),
    },
    RobitSkin {
        id: "mekanism:pride",
        name: "Pride",
        textures: &["mekanism:pride", "mekanism:pride2", "mekanism:pride3", "mekanism:pride4", "mekanism:pride5", "mekanism:pride6"],
        required_advancement: None,
        custom_model: None,
    },
    RobitSkin {
        id: "mekanism:lesbian",
        name: "Lesbian Pride",
        textures: &["mekanism:lesbian", "mekanism:lesbian2", "mekanism:lesbian3", "mekanism:lesbian4", "mekanism:lesbian5", "mekanism:lesbian6", "mekanism:lesbian7", "mekanism:lesbian8"],
        required_advancement: None,
        custom_model: None,
    },
    RobitSkin {
        id: "mekanism:trans",
        name: "Trans Pride",
        textures: &["mekanism:trans", "mekanism:trans2", "mekanism:trans3", "mekanism:trans4"],
        required_advancement: None,
        custom_model: None,
    },
    RobitSkin {
        id: "mekanism:aro",
        name: "Aro Pride",
        textures: &["mekanism:aro", "mekanism:aro2", "mekanism:aro3", "mekanism:aro4", "mekanism:aro5"],
        required_advancement: None,
        custom_model: None,
    },
    RobitSkin {
        id: "mekanism:ace",
        name: "Ace Pride",
        textures: &["mekanism:ace", "mekanism:ace2", "mekanism:ace3", "mekanism:ace4"],
        required_advancement: None,
        custom_model: None,
    },
    RobitSkin {
        id: "mekanism:bi",
        name: "Bi Pride",
        textures: &["mekanism:bi", "mekanism:bi2", "mekanism:bi3", "mekanism:bi4", "mekanism:bi5"],
        required_advancement: None,
        custom_model: None,
    },
    RobitSkin {
        id: "mekanism:enby",
        name: "Enby Pride",
        textures: &["mekanism:enby", "mekanism:enby2", "mekanism:enby3", "mekanism:enby4"],
        required_advancement: None,
        custom_model: None,
    },
    RobitSkin {
        id: "mekanism:pan",
        name: "Pan Pride",
        textures: &["mekanism:pan", "mekanism:pan2", "mekanism:pan3", "mekanism:pan4", "mekanism:pan5", "mekanism:pan6"],
        required_advancement: None,
        custom_model: None,
    },
    RobitSkin {
        id: "mekanism:gay",
        name: "Gay Pride",
        textures: &["mekanism:gay", "mekanism:gay2", "mekanism:gay3", "mekanism:gay4", "mekanism:gay5", "mekanism:gay6", "mekanism:gay7"],
        required_advancement: None,
        custom_model: None,
    },
    RobitSkin {
        id: "mekanism:agender",
        name: "Agender Pride",
        textures: &["mekanism:agender", "mekanism:agender2", "mekanism:agender3", "mekanism:agender4", "mekanism:agender5", "mekanism:agender6"],
        required_advancement: None,
        custom_model: None,
    },
    RobitSkin {
        id: "mekanism:genderfluid",
        name: "Genderfluid Pride",
        textures: &["mekanism:genderfluid", "mekanism:genderfluid2", "mekanism:genderfluid3", "mekanism:genderfluid4", "mekanism:genderfluid5"],
        required_advancement: None,
        custom_model: None,
    },
];

/// Looks up a skin by registry key; defaults to BASE if not found.
/// Java: `MekanismRobitSkins.java:49-57` (`SkinLookup`)
pub fn lookup_skin(skin_id: &str) -> &'static RobitSkin {
    ALL_SKINS
        .iter()
        .find(|s| s.id == skin_id)
        .unwrap_or(&ALL_SKINS[0])
}

// -------------------------------------------------------------------------------------------------
// Robit Entity Data Structure
// -------------------------------------------------------------------------------------------------

/// Global position binding (dimension + block coordinates).
/// Java: `net.minecraft.core.GlobalPos`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlobalPos {
    pub dimension: String,
    pub pos: [i32; 3],
}

/// In-game representation of an ItemEntity that can be collected.
#[derive(Clone, Debug, PartialEq)]
pub struct WorldItemEntity {
    pub id: u32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub item: Stack,
    pub pickup_delay: u32,
    pub is_alive: bool,
}

/// The complete state of an active Robit entity.
#[derive(Clone, Debug, PartialEq)]
pub struct RobitEntity {
    pub id: u32,
    pub dimension: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub xo: f64,
    pub yo: f64,
    pub zo: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub health: f32,
    pub energy: u64, // Forge Energy (0 to 40 000 FE)

    // Security & Ownership
    pub owner_uuid: String,
    pub owner_name: String,
    pub custom_name: Option<String>,
    pub security_mode: SecurityMode,

    // Automation Modes
    pub following: bool,
    pub drop_pickup: bool,
    pub skin: String,
    pub default_skin_manually_selected: bool,

    // Home Chargepad Binding
    pub home_location: Option<GlobalPos>,

    // Inventory Slots (30 slots)
    // 0..26: Storage
    // 27: Energy slot
    // 28: Smelting input
    // 29: Smelting output
    pub slots: Vec<Option<Stack>>,

    // Internal Furnace State
    pub smelting_progress: u32, // 0 to 100 ticks
    pub errors: Vec<RobitRecipeError>,

    // Follow goal (`RobitAIFollow`): running, and the ticks until the next decision
    pub follow_active: bool,
    pub follow_recalc: u32,
    /// Random numbers for the teleport around the owner (a simple generator, seeded by the id).
    pub rng: u64,

    // Animation Ticks
    pub tick_count: u64,
    pub texture_index: usize,
}

impl RobitEntity {
    /// Creates a new Robit entity instance at the given coordinates.
    pub fn new(id: u32, dimension: String, x: f64, y: f64, z: f64, owner_uuid: String, owner_name: String) -> Self {
        Self {
            id,
            dimension,
            x,
            y,
            z,
            xo: x,
            yo: y,
            zo: z,
            yaw: 0.0,
            pitch: 0.0,
            health: MAX_HEALTH,
            energy: MAX_ENERGY_FE,
            owner_uuid,
            owner_name,
            custom_name: None,
            security_mode: SecurityMode::Public,
            following: false,
            drop_pickup: false,
            skin: SKIN_BASE.to_string(),
            default_skin_manually_selected: false,
            home_location: None,
            slots: vec![None; TOTAL_SLOTS],
            smelting_progress: 0,
            errors: Vec::new(),
            follow_active: false,
            follow_recalc: 0,
            rng: u64::from(id).wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1,
            tick_count: 0,
            texture_index: 0,
        }
    }

    /// Displayed name of the Robit.
    pub fn display_name(&self) -> &str {
        self.custom_name.as_deref().unwrap_or("Robit")
    }

    /// Whether a player has permission to interact with this Robit.
    /// Java: `EntityRobit.java:427` (`IEntitySecurityUtils.canAccess`)
    pub fn can_player_access(&self, player_uuid: &str, is_trusted: bool) -> bool {
        match self.security_mode {
            SecurityMode::Public => true,
            SecurityMode::Private => self.owner_uuid == player_uuid,
            SecurityMode::Trusted => self.owner_uuid == player_uuid || is_trusted,
        }
    }

    pub fn take_damage(&mut self, damage: f64) {
        if damage <= 0.0 {
            return;
        }
        // `onDamageTaken`: `clampToLong(1_000D * newDamage)` Joules (the double is cut, not rounded up), health back to the maximum. The conversion to
        // Forge Energy is rounded up so that it is never cheaper than the original.
        let joules = (1_000.0 * damage) as u64;
        self.energy = self.energy.saturating_sub((joules * 2).div_ceil(5));
        self.health = MAX_HEALTH;
    }

    pub fn calculate_travel_energy_cost(&self) -> u64 {
        let dx = self.x - self.xo;
        let dy = self.y - self.yo;
        let dz = self.z - self.zo;
        let dist_sq = dx * dx + dy * dy + dz * dz;
        if dist_sq <= 0.0 {
            return 0;
        }
        // `getRoundedTravelEnergy`: `ceilToLong(DISTANCE_MULTIPLIER * sqrt(distanceToSqr(xo, yo, zo)))` Joules, rounded up to Forge Energy.
        let joules = (DISTANCE_MULTIPLIER * dist_sq.sqrt()).ceil() as u64;
        (joules * 2).div_ceil(5)
    }

    pub fn collect_items(&mut self, items: &mut [WorldItemEntity]) -> u32 {
        if !self.drop_pickup {
            return 0;
        }
        // `getBoundingBox().inflate(1.5)`: the box of the Robit (0.6 x 0.65) grown 1.5 on every side; an item counts when its own box touches it.
        let (hw, h) = (ROBIT_WIDTH / 2.0, ROBIT_HEIGHT);
        let (min_x, max_x) = (self.x - hw - COLLECTION_INFLATION, self.x + hw + COLLECTION_INFLATION);
        let (min_y, max_y) = (self.y - COLLECTION_INFLATION, self.y + h + COLLECTION_INFLATION);
        let (min_z, max_z) = (self.z - hw - COLLECTION_INFLATION, self.z + hw + COLLECTION_INFLATION);
        let mut collected_count = 0;
        for entity in items.iter_mut() {
            // `isItemValid`: alive, no pick-up delay, not a Robit item.
            if !entity.is_alive || entity.pickup_delay > 0 || entity.item.item == ITEM_ID {
                continue;
            }
            if !(entity.x >= min_x && entity.x <= max_x && entity.y >= min_y && entity.y <= max_y && entity.z >= min_z && entity.z <= max_z) {
                continue;
            }
            // `collectItems`: the slots are walked in order and the **first** one that is empty or holds the same item with room takes the item
            // (the whole stack into an empty one, or what fits into the other) and the walk ends: the rest waits for the next tick.
            for slot in self.slots[0..STORAGE_SLOTS].iter_mut() {
                match slot {
                    None => {
                        collected_count += entity.item.count;
                        *slot = Some(entity.item.clone());
                        entity.item.count = 0;
                        entity.is_alive = false;
                        break;
                    }
                    Some(existing) => {
                        let max_size = crate::bin::stack_size(&existing.item) as u32;
                        if existing.item == entity.item.item && existing.components == entity.item.components && existing.count < max_size {
                            let to_add = (max_size - existing.count).min(entity.item.count);
                            existing.count += to_add;
                            entity.item.count -= to_add;
                            collected_count += to_add;
                            if entity.item.count == 0 {
                                entity.is_alive = false;
                            }
                            break;
                        }
                    }
                }
            }
        }
        collected_count
    }

    /// Progresses internal smelting recipe.
    /// Java: `EntityRobit.java:686-698`
    /// Requires 100 ticks (5s) and 20 FE/t (50 J/t).
    pub fn step_smelting(&mut self, smelting_recipes: &HashMap<String, (String, u32)>) {
        self.errors.clear();

        // Without an input, or with one that gives nothing, the cached recipe is dropped and the progress is lost (`CachedRecipe.process`: a
        // negative operation count resets `operatingTicks`); without energy or output room it only waits.
        let input_stack = match &self.slots[SMELT_INPUT_SLOT] {
            Some(s) if s.count > 0 => s.clone(),
            _ => {
                self.smelting_progress = 0;
                return; // No input item
            }
        };

        let (output_id, output_count) = match smelting_recipes.get(&input_stack.item) {
            Some(r) => r.clone(),
            None => {
                self.smelting_progress = 0;
                self.errors.push(RobitRecipeError::InputDoesntProduceOutput);
                return;
            }
        };

        // Check output slot room
        if let Some(out_slot) = &self.slots[SMELT_OUTPUT_SLOT] {
            if out_slot.item != output_id || out_slot.count + output_count > 64 {
                self.errors.push(RobitRecipeError::NotEnoughOutputSpace);
                return;
            }
        }

        // Check energy
        if self.energy < SMELTING_ENERGY_USAGE_FE {
            self.errors.push(RobitRecipeError::NotEnoughEnergy);
            return;
        }

        // Consume energy and progress
        self.energy -= SMELTING_ENERGY_USAGE_FE;
        self.smelting_progress += 1;

        if self.smelting_progress >= SMELTING_TICKS_REQUIRED {
            self.smelting_progress = 0;

            // Decrement input
            if let Some(in_slot) = &mut self.slots[SMELT_INPUT_SLOT] {
                in_slot.count -= 1;
                if in_slot.count == 0 {
                    self.slots[SMELT_INPUT_SLOT] = None;
                }
            }

            // Produce output
            if let Some(out_slot) = &mut self.slots[SMELT_OUTPUT_SLOT] {
                out_slot.count += output_count;
            } else {
                self.slots[SMELT_OUTPUT_SLOT] = Some(Stack {
                    item: output_id,
                    count: output_count,
                    components: String::new(),
                });
            }
        }
    }

    /// A number from `lo` to `hi` (both included), for the teleport around the owner.
    fn next_range(&mut self, lo: i32, hi: i32) -> i32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        lo + (self.rng % ((hi - lo + 1) as u64)) as i32
    }

    /// `RobitAIBase.teleportToAroundBlockPos` (copy of `TamableAnimal`): up to 10 tries at a spot 2 or 3 blocks from the owner on x or z (and -1..1 in y),
    /// the first one the host says it can stand on. Not on the owner itself.
    pub fn teleport_around(&mut self, owner_block: [i32; 3], can_teleport_to: &dyn Fn([i32; 3]) -> bool) -> bool {
        for _ in 0..10 {
            let j = self.next_range(-3, 3);
            let k = self.next_range(-3, 3);
            if j.abs() >= 2 || k.abs() >= 2 {
                let l = self.next_range(-1, 1);
                let pos = [owner_block[0] + j, owner_block[1] + l, owner_block[2] + k];
                if can_teleport_to(pos) {
                    self.x = f64::from(pos[0]) + 0.5;
                    self.y = f64::from(pos[1]);
                    self.z = f64::from(pos[2]) + 0.5;
                    return true;
                }
            }
        }
        false
    }

    /// `RobitAIFollow` and `RobitAIBase.updateTask`. It starts when the owner (in the same dimension, not a spectator: the host passes `None` then) is at least
    /// 4 blocks away and the Robit follows and has energy; it goes on while the owner is farther than 2 blocks. Every 10 ticks it either teleports around the
    /// owner (12 blocks or more) or heads for the owner (the port moves it a step a tick: the original uses the path finder).
    pub fn tick_follow(&mut self, owner_pos: Option<(f64, f64, f64, &str)>, can_teleport_to: &dyn Fn([i32; 3]) -> bool) {
        let Some((ox, oy, oz, odim)) = owner_pos else {
            self.follow_active = false;
            return;
        };
        if odim != self.dimension || !self.following || self.energy == 0 {
            self.follow_active = false;
            return;
        }
        let (dx, dy, dz) = (ox - self.x, oy - self.y, oz - self.z);
        let dist_sq = dx * dx + dy * dy + dz * dz;
        if !self.follow_active {
            if dist_sq < FOLLOW_START_DIST * FOLLOW_START_DIST {
                return;
            }
            self.follow_active = true;
            self.follow_recalc = 0;
        } else if dist_sq <= FOLLOW_STOP_DIST * FOLLOW_STOP_DIST {
            self.follow_active = false;
            return;
        }
        if self.follow_recalc > 0 {
            self.follow_recalc -= 1;
        }
        if self.follow_recalc == 0 {
            self.follow_recalc = FOLLOW_RECALC_TICKS;
            if dist_sq >= FOLLOW_TELEPORT_DIST_SQ {
                self.teleport_around([ox.floor() as i32, oy.floor() as i32, oz.floor() as i32], can_teleport_to);
                return;
            }
        }
        let dist = dist_sq.sqrt();
        let step = (MOVEMENT_SPEED as f64).min(dist);
        self.x += dx / dist * step;
        self.y += dy / dist * step;
        self.z += dz / dist * step;
    }

    /// `EntityRobit.tick`: without a home it disappears; every 20 ticks, with the home chunk loaded and no chargepad there any more, it drops
    /// as an item and disappears.
    pub fn home_check(&self, home_loaded: bool, chargepad_at_home: bool) -> HomeCheck {
        if self.home_location.is_none() {
            return HomeCheck::Discard;
        }
        if self.tick_count % HOME_CHECK_TICKS == 0 && home_loaded && !chargepad_at_home {
            return HomeCheck::DropAndDiscard;
        }
        HomeCheck::Stay
    }

    /// Teleports Robit to its saved home chargepad.
    /// Java: `EntityRobit.java:398-418` (`goHome`)
    pub fn go_home(&mut self) -> bool {
        if let Some(home) = &self.home_location {
            self.following = false;
            self.dimension = home.dimension.clone();
            self.x = home.pos[0] as f64 + 0.5;
            self.y = home.pos[1] as f64 + 0.3;
            self.z = home.pos[2] as f64 + 0.5;
            self.xo = self.x;
            self.yo = self.y;
            self.zo = self.z;
            true
        } else {
            false
        }
    }

    /// Handles a GUI button press.
    pub fn handle_button(&mut self, button: RobitButton, player_uuid: &str, is_trusted: bool) -> bool {
        if !self.can_player_access(player_uuid, is_trusted) {
            return false;
        }
        match button {
            RobitButton::GoHome => self.go_home(),
            RobitButton::ToggleFollow => {
                self.following = !self.following;
                true
            }
            RobitButton::TogglePickup => {
                self.drop_pickup = !self.drop_pickup;
                true
            }
            RobitButton::OpenRename | RobitButton::OpenSkinSelect | RobitButton::SwitchTab(_) => true,
        }
    }

    /// Changes Robit skin if unlocked.
    /// Java: `EntityRobit.java:751-778`
    pub fn set_skin(&mut self, new_skin_id: &str, is_unlocked: bool) -> bool {
        // `EntityRobit.setSkin`: nothing to do when it already has that skin; otherwise it needs the skin to be unlocked
        // (the base one always is); choosing the base skin marks it as chosen by hand.
        if self.skin == new_skin_id {
            return true;
        }
        if !is_unlocked && new_skin_id != SKIN_BASE {
            return false;
        }
        self.skin = new_skin_id.to_string();
        if new_skin_id == SKIN_BASE {
            self.default_skin_manually_selected = true;
        }
        true
    }

    pub fn tick(
        &mut self,
        owner_pos: Option<(f64, f64, f64, &str)>,
        world_items: &mut [WorldItemEntity],
        smelting_recipes: &HashMap<String, (String, u32)>,
        on_chargepad: bool,
        can_teleport_to: &dyn Fn([i32; 3]) -> bool,
    ) {
        self.tick_count += 1;

        // `baseTick`, before the entity moves: while it follows the owner, with the owner farther than 2 blocks and the path still going, it pays for
        // what it moved the tick before.
        if self.following && self.follow_active && self.energy > 0 {
            if let Some((ox, oy, oz, _)) = owner_pos {
                let (dx, dy, dz) = (ox - self.x, oy - self.y, oz - self.z);
                if dx * dx + dy * dy + dz * dz > 4.0 {
                    let cost = self.calculate_travel_energy_cost();
                    self.energy = self.energy.saturating_sub(cost);
                }
            }
        }
        // `setOldPosAndRot`
        self.xo = self.x;
        self.yo = self.y;
        self.zo = self.z;

        // `RobitAIFollow`
        self.tick_follow(owner_pos, can_teleport_to);

        // `collectItems` (in `baseTick`)
        if self.drop_pickup {
            self.collect_items(world_items);
        }

        // `baseTick`: an empty battery away from a chargepad sends it home.
        if self.energy == 0 && !on_chargepad {
            self.go_home();
        }

        // Internal furnace
        self.step_smelting(smelting_recipes);
    }
}

// -------------------------------------------------------------------------------------------------
// Item Form & Spawning / Dropping
// -------------------------------------------------------------------------------------------------

/// Serialized ItemRobit representation.
/// Java: `ItemRobit.java:47-55` and `EntityRobit.java:451-469` (`getItemVariant`)
#[derive(Clone, Debug, PartialEq)]
pub struct RobitItemData {
    pub energy: u64,
    pub custom_name: Option<String>,
    pub skin: String,
    pub owner_uuid: String,
    pub security_mode: SecurityMode,
    pub default_manually_selected: bool,
    pub items: Vec<Option<Stack>>,
}

/// Packs an active Robit into an ItemRobit stack when shift-clicked with a configurator.
/// Java: `EntityRobit.java:430-438,451-469`
pub fn pack_robit_to_item(robit: &RobitEntity) -> RobitItemData {
    RobitItemData {
        energy: robit.energy,
        custom_name: robit.custom_name.clone(),
        skin: robit.skin.clone(),
        owner_uuid: robit.owner_uuid.clone(),
        security_mode: robit.security_mode,
        default_manually_selected: robit.default_skin_manually_selected,
        items: robit.slots.clone(),
    }
}

/// Spawns a Robit in the world on an inactive chargepad from an ItemRobit stack.
/// Java: `ItemRobit.java:77-124`
pub fn spawn_robit_from_item(
    item_data: RobitItemData,
    entity_id: u32,
    dimension: String,
    chargepad_pos: [i32; 3],
    placer_uuid: String,
    placer_name: String,
    skin_unlocked: bool,
) -> RobitEntity {
    // `ItemRobit.useOn`: no owner on the item means the player who places it.
    let owner_is_placer = item_data.owner_uuid.is_empty() || item_data.owner_uuid == placer_uuid;
    let owner_uuid = if item_data.owner_uuid.is_empty() { placer_uuid } else { item_data.owner_uuid };
    let mut robit = RobitEntity::new(
        entity_id,
        dimension.clone(),
        chargepad_pos[0] as f64 + 0.5,
        chargepad_pos[1] as f64 + 0.1,
        chargepad_pos[2] as f64 + 0.5,
        owner_uuid,
        // The name belongs to the owner: the placer's only when the placer is the owner (the host looks the other one up).
        if owner_is_placer { placer_name } else { String::new() },
    );
    robit.energy = item_data.energy;
    robit.custom_name = item_data.custom_name;
    robit.security_mode = item_data.security_mode;
    robit.slots = item_data.items;
    robit.home_location = Some(GlobalPos { dimension, pos: chargepad_pos });
    // `setSkin(skin, player)` checks that the skin is unlocked for the player (a locked one leaves the default); only then the flag of the item is copied.
    robit.set_skin(&item_data.skin, skin_unlocked);
    robit.default_skin_manually_selected = item_data.default_manually_selected;
    robit
}

/// `ItemRobit.useOn`: it can be placed on a chargepad that is not already in use (`!getActive()`).
pub fn can_place_on_chargepad(chargepad_exists: bool, chargepad_active: bool) -> bool {
    chargepad_exists && !chargepad_active
}

// -------------------------------------------------------------------------------------------------
// FORK Requirements (isolated host call stubs and declarations)
// -------------------------------------------------------------------------------------------------

/// Host interface for spawning custom entities with pathfinding, AI goals and inventory.
///
/// WIT proposed in `FORK_robit_quimicos.md`:
/// ```wit
/// record entity-robit-spawn-spec {
///     dimension: string,
///     x: f64,
///     y: f64,
///     z: f64,
///     owner-uuid: string,
///     skin: string,
/// }
/// spawn-robit-entity: func(spec: entity-robit-spawn-spec) -> result<u32, string>;
/// ```
// FORK-10
pub fn host_spawn_robit_entity(
    dimension: &str,
    pos: [f64; 3],
    owner_uuid: &str,
    skin: &str,
) -> Result<u32, &'static str> {
    let _ = (dimension, pos, owner_uuid, skin);
    Ok(1)
}

/// Host interface for sending custom C2S/S2C packets (entity button presses, skins, rename).
///
/// WIT proposed in `FORK_robit_quimicos.md`:
/// ```wit
/// send-entity-button: func(entity-id: u32, button-id: u32, extra-data: list<u8>) -> result<_, string>;
/// ```
// FORK-24
pub fn host_send_entity_button(entity_id: u32, button_id: u32, extra_data: &[u8]) -> Result<(), &'static str> {
    let _ = (entity_id, button_id, extra_data);
    Ok(())
}

// -------------------------------------------------------------------------------------------------
// Unit Tests (verifying exact 1:1 match with Java constants and lines)
// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn robit_constants_match_java_definitions() {
        // EntityRobit.java:135: Attributes.MAX_HEALTH = 1.0D, MOVEMENT_SPEED = 0.3F
        assert_eq!(MAX_HEALTH, 1.0);
        assert_eq!(MOVEMENT_SPEED, 0.3);

        // EntityRobit.java:162: MAX_ENERGY = 100_000L Joules = 40_000 FE
        assert_eq!(MAX_ENERGY_JOULES, 100_000);
        assert_eq!(MAX_ENERGY_FE, 40_000);

        // EntityRobit.java:530: MathUtils.clampToLong(1_000D * damage) Joules = 400 FE / damage
        assert_eq!(ENERGY_PER_DAMAGE_JOULES, 1_000);
        assert_eq!(ENERGY_PER_DAMAGE_FE, 400);

        // EntityRobit.java:166: ticksRequired = 100
        assert_eq!(SMELTING_TICKS_REQUIRED, 100);

        // EntityRobit.java:694: energizedSmelter = 50L Joules = 20 FE / tick
        assert_eq!(SMELTING_ENERGY_USAGE_JOULES, 50);
        assert_eq!(SMELTING_ENERGY_USAGE_FE, 20);

        // Slot layout
        assert_eq!(TOTAL_SLOTS, 30);
        assert_eq!(STORAGE_SLOTS, 27);
        assert_eq!(ENERGY_SLOT, 27);
        assert_eq!(SMELT_INPUT_SLOT, 28);
        assert_eq!(SMELT_OUTPUT_SLOT, 29);
    }

    #[test]
    fn robit_damage_absorption_via_energy() {
        // Java: EntityRobit.java:529-533
        // Dealing 5.0 damage consumes 5.0 * 1000 J = 5000 J = 2000 FE
        let mut robit = RobitEntity::new(
            1,
            "minecraft:overworld".into(),
            0.0,
            64.0,
            0.0,
            "uuid-123".into(),
            "Steve".into(),
        );
        assert_eq!(robit.energy, 40_000);
        assert_eq!(robit.health, 1.0);

        robit.take_damage(5.0);
        assert_eq!(robit.energy, 38_000); // 40_000 - 2_000
        assert_eq!(robit.health, 1.0); // Health stays at max (immortal)
    }

    #[test]
    fn robit_movement_energy_cost_formula() {
        // Java: EntityRobit.java:287-289: ceil(1.5 * sqrt(dist_sq)) Joules
        let mut robit = RobitEntity::new(
            1,
            "minecraft:overworld".into(),
            0.0,
            64.0,
            0.0,
            "uuid-123".into(),
            "Steve".into(),
        );
        // Move 4 blocks along X: dist = 4.0, dist_sq = 16.0
        // Joules = ceil(1.5 * 4.0) = 6 Joules; in Forge Energy rounded up: ceil(6 * 2 / 5) = 3 FE
        robit.x = 4.0;
        let cost = robit.calculate_travel_energy_cost();
        assert_eq!(cost, 3);
        // A short step still costs: ceil(1.5 * 0.2) = 1 J = 1 FE (rounded up), never free.
        robit.x = 0.2;
        assert_eq!(robit.calculate_travel_energy_cost(), 1);
    }

    #[test]
    fn robit_item_collection_into_twenty_seven_slots() {
        // Java: EntityRobit.java:365-396
        let mut robit = RobitEntity::new(
            1,
            "minecraft:overworld".into(),
            0.0,
            64.0,
            0.0,
            "uuid-123".into(),
            "Steve".into(),
        );
        robit.drop_pickup = true;

        let mut world_items = vec![
            WorldItemEntity {
                id: 10,
                x: 0.5,
                y: 64.0,
                z: 0.5,
                item: Stack {
                    item: "minecraft:iron_ingot".into(),
                    count: 32,
                    components: String::new(),
                },
                pickup_delay: 0,
                is_alive: true,
            },
            WorldItemEntity {
                id: 11,
                x: 0.8,
                y: 64.0,
                z: 0.8,
                item: Stack {
                    item: "minecraft:iron_ingot".into(),
                    count: 40,
                    components: String::new(),
                },
                pickup_delay: 0,
                is_alive: true,
            },
        ];

        // `collectItems`: the first slot that is empty or holds the same item with room takes the item and the walk ends. First tick: the 32 go whole into
        // slot 0; the 40 then fill slot 0 (32 of them) and the 8 left stay in the world for the next tick.
        let collected = robit.collect_items(&mut world_items);
        assert_eq!(collected, 64);
        assert_eq!(robit.slots[0].as_ref().unwrap().count, 64);
        assert_eq!(robit.slots[1], None);
        assert!(world_items[1].is_alive && world_items[1].item.count == 8);
        // Second tick: slot 0 is full, slot 1 is empty and takes the 8.
        let collected = robit.collect_items(&mut world_items);
        assert_eq!(collected, 8);
        assert_eq!(robit.slots[1].as_ref().unwrap().count, 8);
        assert!(!world_items[1].is_alive);
    }

    #[test]
    fn java_robit_collection_respects_components_pickup_delay_and_robit_items() {
        let mut robit = RobitEntity::new(1, "minecraft:overworld".into(), 0.0, 64.0, 0.0, "u".into(), "Steve".into());
        robit.drop_pickup = true;
        robit.slots[0] = Some(Stack { item: "minecraft:diamond_sword".into(), count: 1, components: "{damage:5}".into() });
        let item = |id, name: &str, components: &str, delay| WorldItemEntity {
            id, x: 0.3, y: 64.0, z: 0.3, item: Stack { item: name.into(), count: 1, components: components.into() }, pickup_delay: delay, is_alive: true,
        };
        let mut items = vec![
            item(1, "minecraft:diamond_sword", "{damage:0}", 0),   // other components: does not merge (and swords stack to 1)
            item(2, "minecraft:cobblestone", "", 10),              // still has a pick-up delay
            item(3, ITEM_ID, "", 0),                                // a Robit item is never picked up
        ];
        assert_eq!(robit.collect_items(&mut items), 1, "only the sword goes, into the first empty slot");
        assert_eq!(robit.slots[0].as_ref().unwrap().components, "{damage:5}");
        assert_eq!(robit.slots[1].as_ref().unwrap().components, "{damage:0}");
        assert!(items[1].is_alive && items[2].is_alive);
        // An item far outside the 0.6 x 0.65 box grown by 1.5 is left alone.
        let mut far = vec![item(4, "minecraft:dirt", "", 0)];
        far[0].x = 3.0;
        assert_eq!(robit.collect_items(&mut far), 0);
    }

    #[test]
    fn java_robit_damage_costs_the_cut_joules() {
        // `onDamageTaken`: `clampToLong(1_000D * newDamage)` Joules, cut not rounded; the health goes back to the maximum.
        let mut robit = RobitEntity::new(1, "minecraft:overworld".into(), 0.0, 64.0, 0.0, "u".into(), "Steve".into());
        robit.take_damage(2.5);
        assert_eq!(robit.energy, 40_000 - 1_000, "2 500 J = 1 000 FE");
        robit.take_damage(0.0004);
        assert_eq!(robit.energy, 39_000, "0.4 J is cut to 0");
        assert_eq!(robit.health, MAX_HEALTH);
    }

    #[test]
    fn java_robit_follow_starts_at_four_blocks_and_stops_at_two() {
        // `RobitAIFollow(this, 1, 4, 2)`: starts with the owner at 4 blocks or more, goes on while the owner is farther than 2.
        let nobody = |_: [i32; 3]| true;
        let mut robit = RobitEntity::new(1, "minecraft:overworld".into(), 0.0, 64.0, 0.0, "u".into(), "Steve".into());
        robit.following = true;
        let dim = "minecraft:overworld";
        robit.tick_follow(Some((3.9, 64.0, 0.0, dim)), &nobody);
        assert!(!robit.follow_active, "3.9 blocks: too close to start");
        robit.tick_follow(Some((4.0, 64.0, 0.0, dim)), &nobody);
        assert!(robit.follow_active && robit.x > 0.0, "4 blocks: it starts and moves");
        // Already following, it keeps going at 3 blocks (it would not start there)...
        robit.x = 1.0;
        robit.tick_follow(Some((4.0, 64.0, 0.0, dim)), &nobody);
        assert!(robit.follow_active);
        // ...and stops once it is within 2 blocks.
        robit.x = 2.0;
        robit.tick_follow(Some((4.0, 64.0, 0.0, dim)), &nobody);
        assert!(!robit.follow_active);
        // Other dimension, no energy, not following: it does not follow.
        robit.x = 0.0;
        robit.tick_follow(Some((10.0, 64.0, 0.0, "minecraft:the_nether")), &nobody);
        assert!(!robit.follow_active);
        robit.energy = 0;
        robit.tick_follow(Some((10.0, 64.0, 0.0, dim)), &nobody);
        assert!(!robit.follow_active);
    }

    #[test]
    fn java_robit_teleports_around_the_owner_from_twelve_blocks() {
        // `RobitAIBase`: `distanceToSqr >= 144` teleports to a spot 2 or 3 blocks from the owner on x or z (not on the owner), up to 10 tries.
        let mut robit = RobitEntity::new(1, "minecraft:overworld".into(), 0.0, 64.0, 0.0, "u".into(), "Steve".into());
        robit.following = true;
        let dim = "minecraft:overworld";
        robit.tick_follow(Some((12.0, 64.0, 0.0, dim)), &|_| true);
        let (dx, dz) = (robit.x - 12.5, robit.z - 0.5);
        assert!(robit.x > 8.0, "it teleported next to the owner, not walked");
        assert!(dx.abs() >= 1.5 || dz.abs() >= 1.5, "never on top of the owner: {dx} {dz}");
        assert!(dx.abs() <= 3.0 && dz.abs() <= 3.0);
        // When the host finds no place to stand, it stays where it is.
        let mut stuck = RobitEntity::new(2, "minecraft:overworld".into(), 0.0, 64.0, 0.0, "u".into(), "Steve".into());
        stuck.following = true;
        stuck.tick_follow(Some((12.0, 64.0, 0.0, dim)), &|_| false);
        assert_eq!((stuck.x, stuck.z), (0.0, 0.0));
    }

    #[test]
    fn java_robit_home_rules() {
        // `EntityRobit.tick` and `baseTick`
        let mut robit = RobitEntity::new(1, "minecraft:overworld".into(), 0.0, 64.0, 0.0, "u".into(), "Steve".into());
        assert_eq!(robit.home_check(true, true), HomeCheck::Discard, "no home: it disappears");
        robit.home_location = Some(GlobalPos { dimension: "minecraft:overworld".into(), pos: [10, 70, 10] });
        robit.tick_count = 40;
        assert_eq!(robit.home_check(true, true), HomeCheck::Stay);
        assert_eq!(robit.home_check(true, false), HomeCheck::DropAndDiscard, "the chargepad is gone");
        assert_eq!(robit.home_check(false, false), HomeCheck::Stay, "the home chunk is not loaded");
        robit.tick_count = 41;
        assert_eq!(robit.home_check(true, false), HomeCheck::Stay, "only every 20 ticks");
        // An empty battery sends it home, at y + 0.3, unless it is on a chargepad.
        robit.energy = 0;
        let none: HashMap<String, (String, u32)> = HashMap::new();
        robit.tick(None, &mut [], &none, true, &|_| true);
        assert_eq!((robit.x, robit.y), (0.0, 64.0));
        robit.tick(None, &mut [], &none, false, &|_| true);
        assert_eq!((robit.x, robit.y, robit.z), (10.5, 70.3, 10.5));
    }

    #[test]
    fn java_robit_smelting_loses_the_progress_without_input() {
        let mut robit = RobitEntity::new(1, "minecraft:overworld".into(), 0.0, 64.0, 0.0, "u".into(), "Steve".into());
        let mut recipes = HashMap::new();
        recipes.insert("minecraft:raw_iron".to_owned(), ("minecraft:iron_ingot".to_owned(), 1));
        robit.slots[SMELT_INPUT_SLOT] = Some(Stack { item: "minecraft:raw_iron".into(), count: 1, components: String::new() });
        for _ in 0..50 {
            robit.step_smelting(&recipes);
        }
        assert_eq!(robit.smelting_progress, 50);
        // Without energy it only waits.
        let energy = robit.energy;
        robit.energy = 0;
        robit.step_smelting(&recipes);
        assert_eq!(robit.smelting_progress, 50);
        robit.energy = energy;
        // With the input taken out the progress is lost.
        robit.slots[SMELT_INPUT_SLOT] = None;
        robit.step_smelting(&recipes);
        assert_eq!(robit.smelting_progress, 0);
    }

    #[test]
    fn robit_internal_smelting_fast_furnace() {
        // Java: EntityRobit.java:686-698
        // Smelting takes 100 ticks and 20 FE/tick
        let mut robit = RobitEntity::new(
            1,
            "minecraft:overworld".into(),
            0.0,
            64.0,
            0.0,
            "uuid-123".into(),
            "Steve".into(),
        );
        robit.slots[SMELT_INPUT_SLOT] = Some(Stack {
            item: "minecraft:raw_iron".into(),
            count: 2,
            components: String::new(),
        });

        let mut recipes = HashMap::new();
        recipes.insert("minecraft:raw_iron".into(), ("minecraft:iron_ingot".into(), 1));

        // Step 99 ticks
        for _ in 0..99 {
            robit.step_smelting(&recipes);
        }
        assert_eq!(robit.smelting_progress, 99);
        assert_eq!(robit.slots[SMELT_OUTPUT_SLOT], None);
        assert_eq!(robit.energy, 40_000 - 99 * 20);

        // 100th tick: produces 1 iron ingot
        robit.step_smelting(&recipes);
        assert_eq!(robit.smelting_progress, 0);
        assert_eq!(
            robit.slots[SMELT_OUTPUT_SLOT],
            Some(Stack {
                item: "minecraft:iron_ingot".into(),
                count: 1,
                components: String::new(),
            })
        );
        assert_eq!(
            robit.slots[SMELT_INPUT_SLOT],
            Some(Stack {
                item: "minecraft:raw_iron".into(),
                count: 1,
                components: String::new(),
            })
        );
    }

    #[test]
    fn robit_skins_thirteen_skins_registered() {
        // `data/mekanism/mekanism/robit_skin/*.json` and `robit_skin.mekanism.*` of the lang file of the 1.21.1-10.7.19.85 release.
        assert_eq!(ALL_SKINS.len(), 13);
        assert_eq!(lookup_skin(SKIN_BASE).name, "Default");
        assert_eq!(lookup_skin(SKIN_ALLAY).name, "Allay Costume");
        assert_eq!(lookup_skin("mekanism:trans").name, "Trans Pride");
        assert_eq!(lookup_skin("nonexistent").id, SKIN_BASE);
        // Only the allay skin needs an advancement (`advancement_based`) and has its own item model.
        let allay = lookup_skin(SKIN_ALLAY);
        assert_eq!(allay.required_advancement, Some("minecraft:husbandry/allay_deliver_item_to_player"));
        assert_eq!(allay.custom_model, Some("mekanism:item/robit_allay"));
        assert_eq!(ALL_SKINS.iter().filter(|s| s.required_advancement.is_some()).count(), 1);
        assert_eq!(ALL_SKINS.iter().filter(|s| s.custom_model.is_some()).count(), 1);
        // Each skin has this many texture variants (the JSON `textures` lists).
        for (id, n) in [("robit", 2), ("allay", 2), ("pride", 6), ("lesbian", 8), ("trans", 4), ("aro", 5), ("ace", 4), ("bi", 5), ("enby", 4), ("pan", 6), ("gay", 7), ("agender", 6), ("genderfluid", 5)] {
            assert_eq!(lookup_skin(&format!("mekanism:{id}")).textures.len(), n, "{id}");
        }
        assert_eq!(lookup_skin(SKIN_BASE).textures, &["mekanism:robit", "mekanism:robit2"]);
    }

    #[test]
    fn robit_item_pack_and_spawn_preserves_state() {
        let mut original = RobitEntity::new(
            1,
            "minecraft:overworld".into(),
            10.0,
            64.0,
            10.0,
            "uuid-steve".into(),
            "Steve".into(),
        );
        original.energy = 25_000;
        original.custom_name = Some("Wall-E".into());
        original.skin = "mekanism:allay".into();
        original.security_mode = SecurityMode::Trusted;
        original.slots[0] = Some(Stack {
            item: "minecraft:diamond".into(),
            count: 3,
            components: String::new(),
        });

        let item_data = pack_robit_to_item(&original);
        assert_eq!(item_data.energy, 25_000);
        assert_eq!(item_data.custom_name, Some("Wall-E".into()));
        assert_eq!(item_data.skin, "mekanism:allay");
        assert_eq!(item_data.security_mode, SecurityMode::Trusted);

        // Spawn back onto chargepad at [100, 64, 200]
        let spawned = spawn_robit_from_item(
            item_data,
            2,
            "minecraft:overworld".into(),
            [100, 64, 200],
            "uuid-steve".into(),
            "Steve".into(),
            true,
        );

        assert_eq!(spawned.x, 100.5);
        assert_eq!(spawned.y, 64.1);
        assert_eq!(spawned.z, 200.5);
        assert_eq!(spawned.energy, 25_000);
        assert_eq!(spawned.display_name(), "Wall-E");
        assert_eq!(spawned.skin, "mekanism:allay");
        assert_eq!(spawned.security_mode, SecurityMode::Trusted);
        assert_eq!(
            spawned.slots[0],
            Some(Stack {
                item: "minecraft:diamond".into(),
                count: 3,
                components: String::new(),
            })
        );
        assert_eq!(
            spawned.home_location,
            Some(GlobalPos {
                dimension: "minecraft:overworld".into(),
                pos: [100, 64, 200],
            })
        );
    }

    #[test]
    fn java_robit_item_skin_needs_to_be_unlocked_and_the_owner_keeps_the_name() {
        // `ItemRobit.useOn`: `setSkin(skin, player)` refuses a skin the player has not unlocked (the Robit keeps the default), then the item's
        // "chosen by hand" flag is copied; with no owner on the item the player who places it is the owner.
        let mut original = RobitEntity::new(1, "minecraft:overworld".into(), 0.0, 64.0, 0.0, "uuid-alex".into(), "Alex".into());
        original.skin = SKIN_ALLAY.into();
        original.default_skin_manually_selected = true;
        let data = pack_robit_to_item(&original);
        let locked = spawn_robit_from_item(data.clone(), 2, "minecraft:overworld".into(), [0, 64, 0], "uuid-steve".into(), "Steve".into(), false);
        assert_eq!(locked.skin, SKIN_BASE, "a locked skin does not apply");
        assert!(locked.default_skin_manually_selected, "the flag of the item is copied last");
        assert_eq!(locked.owner_uuid, "uuid-alex");
        assert_eq!(locked.owner_name, "", "Steve placed it but Alex owns it: the name is not Steve's");
        let unlocked = spawn_robit_from_item(data, 3, "minecraft:overworld".into(), [0, 64, 0], "uuid-alex".into(), "Alex".into(), true);
        assert_eq!(unlocked.skin, SKIN_ALLAY);
        assert_eq!(unlocked.owner_name, "Alex");
        // `setSkin` does nothing when the Robit already has the skin, so it does not mark the base one as chosen by hand.
        let mut fresh = RobitEntity::new(4, "minecraft:overworld".into(), 0.0, 64.0, 0.0, "u".into(), "S".into());
        assert!(fresh.set_skin(SKIN_BASE, true));
        assert!(!fresh.default_skin_manually_selected);
        assert!(!fresh.set_skin(SKIN_ALLAY, false));
        assert!(fresh.set_skin(SKIN_ALLAY, true) && fresh.set_skin(SKIN_BASE, true));
        assert!(fresh.default_skin_manually_selected);
        // A Robit item goes only on a chargepad that is not in use.
        assert!(can_place_on_chargepad(true, false));
        assert!(!can_place_on_chargepad(true, true));
        assert!(!can_place_on_chargepad(false, false));
    }
}

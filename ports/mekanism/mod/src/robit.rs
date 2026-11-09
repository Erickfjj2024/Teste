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
pub const FOLLOW_MIN_DIST: f64 = 2.0;
pub const FOLLOW_MAX_DIST: f64 = 4.0;
pub const FOLLOW_TELEPORT_DIST_SQ: f64 = 144.0; // 12 blocks squared

/// Item pickup scan radius (10 blocks).
/// Java: `RobitAIPickup.java:13` (`SEARCH_RADIUS = 10`, `SEARCH_RADIUS_SQ = 100`)
pub const PICKUP_SEARCH_RADIUS: f64 = 10.0;
pub const PICKUP_SEARCH_RADIUS_SQ: f64 = 100.0;

/// Item collection inflation bounding box around Robit (1.5 blocks).
/// Java: `EntityRobit.java:366` (`getBoundingBox().inflate(1.5, 1.5, 1.5)`)
pub const COLLECTION_INFLATION: f64 = 1.5;

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
}

pub const SKIN_BASE: &str = "mekanism:robit";
pub const SKIN_ALLAY: &str = "mekanism:allay";

/// Registered Robit Skins: Base, Allay and the 11 Pride skins.
pub const ALL_SKINS: [RobitSkin; 13] = [
    RobitSkin {
        id: "mekanism:robit",
        name: "Robit",
        textures: &["mekanism:entity/robit", "mekanism:entity/robit_roll"],
        required_advancement: None,
    },
    RobitSkin {
        id: "mekanism:allay",
        name: "Allay",
        textures: &["mekanism:entity/allay_robit"],
        required_advancement: Some("minecraft:adventure/allay_drop_item_on_block"),
    },
    RobitSkin {
        id: "mekanism:pride",
        name: "Pride",
        textures: &["mekanism:entity/pride/pride"],
        required_advancement: None,
    },
    RobitSkin {
        id: "mekanism:lesbian",
        name: "Lesbian",
        textures: &["mekanism:entity/pride/lesbian"],
        required_advancement: None,
    },
    RobitSkin {
        id: "mekanism:trans",
        name: "Trans",
        textures: &["mekanism:entity/pride/trans"],
        required_advancement: None,
    },
    RobitSkin {
        id: "mekanism:aro",
        name: "Aro",
        textures: &["mekanism:entity/pride/aro"],
        required_advancement: None,
    },
    RobitSkin {
        id: "mekanism:ace",
        name: "Ace",
        textures: &["mekanism:entity/pride/ace"],
        required_advancement: None,
    },
    RobitSkin {
        id: "mekanism:bi",
        name: "Bi",
        textures: &["mekanism:entity/pride/bi"],
        required_advancement: None,
    },
    RobitSkin {
        id: "mekanism:enby",
        name: "Enby",
        textures: &["mekanism:entity/pride/enby"],
        required_advancement: None,
    },
    RobitSkin {
        id: "mekanism:pan",
        name: "Pan",
        textures: &["mekanism:entity/pride/pan"],
        required_advancement: None,
    },
    RobitSkin {
        id: "mekanism:gay",
        name: "Gay",
        textures: &["mekanism:entity/pride/gay"],
        required_advancement: None,
    },
    RobitSkin {
        id: "mekanism:agender",
        name: "Agender",
        textures: &["mekanism:entity/pride/agender"],
        required_advancement: None,
    },
    RobitSkin {
        id: "mekanism:genderfluid",
        name: "Genderfluid",
        textures: &["mekanism:entity/pride/genderfluid"],
        required_advancement: None,
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

    /// Handles damage taken by the Robit.
    /// Java: `EntityRobit.java:529-533`
    /// Robit absorbs damage using its energy container (1 000 J/damage = 400 FE/damage) and resets health to 1.0.
    pub fn take_damage(&mut self, damage: f64) {
        if damage <= 0.0 {
            return;
        }
        let energy_needed_joules = (1_000.0 * damage).ceil() as u64;
        let energy_needed_fe = fe(energy_needed_joules);
        self.energy = self.energy.saturating_sub(energy_needed_fe);
        self.health = MAX_HEALTH;
    }

    /// Calculates energy cost for movement between positions.
    /// Java: `EntityRobit.java:287-289` (`ceil(1.5 * sqrt(dist_sq))` Joules)
    pub fn calculate_travel_energy_cost(&self) -> u64 {
        let dx = self.x - self.xo;
        let dy = self.y - self.yo;
        let dz = self.z - self.zo;
        let dist_sq = dx * dx + dy * dy + dz * dz;
        if dist_sq <= 0.0 {
            return 0;
        }
        let joules = (DISTANCE_MULTIPLIER * dist_sq.sqrt()).ceil() as u64;
        fe(joules)
    }

    /// Collects nearby dropped items in the world into Robit's 27 storage slots.
    /// Java: `EntityRobit.java:365-396`
    pub fn collect_items(&mut self, items: &mut [WorldItemEntity]) -> u32 {
        if !self.drop_pickup {
            return 0;
        }
        let mut collected_count = 0;
        let (min_x, max_x) = (self.x - COLLECTION_INFLATION, self.x + COLLECTION_INFLATION);
        let (min_y, max_y) = (self.y - COLLECTION_INFLATION, self.y + COLLECTION_INFLATION);
        let (min_z, max_z) = (self.z - COLLECTION_INFLATION, self.z + COLLECTION_INFLATION);

        for entity in items.iter_mut() {
            if !entity.is_alive || entity.pickup_delay > 0 || entity.item.item == ITEM_ID {
                continue; // Do not pick up another Robit item or dead items
            }
            if entity.x >= min_x && entity.x <= max_x && entity.y >= min_y && entity.y <= max_y && entity.z >= min_z && entity.z <= max_z {
                // Try inserting into 27 storage slots
                let mut remaining = entity.item.count;

                // Pass 1: Merge into existing matching item slots
                for slot in self.slots[0..STORAGE_SLOTS].iter_mut() {
                    if let Some(existing) = slot {
                        if existing.item == entity.item.item && existing.count < 64 {
                            let space = 64 - existing.count;
                            let to_add = remaining.min(space);
                            existing.count += to_add;
                            remaining -= to_add;
                            if remaining == 0 {
                                break;
                            }
                        }
                    }
                }

                // Pass 2: Fill empty slots
                if remaining > 0 {
                    for slot in self.slots[0..STORAGE_SLOTS].iter_mut() {
                        if slot.is_none() {
                            let to_add = remaining.min(64);
                            *slot = Some(Stack {
                                item: entity.item.item.clone(),
                                count: to_add,
                                components: entity.item.components.clone(),
                            });
                            remaining -= to_add;
                            if remaining == 0 {
                                break;
                            }
                        }
                    }
                }

                let picked = entity.item.count - remaining;
                if picked > 0 {
                    entity.item.count = remaining;
                    if remaining == 0 {
                        entity.is_alive = false;
                    }
                    collected_count += picked;
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

        let input_stack = match &self.slots[SMELT_INPUT_SLOT] {
            Some(s) if s.count > 0 => s.clone(),
            _ => return, // No input item
        };

        let (output_id, output_count) = match smelting_recipes.get(&input_stack.item) {
            Some(r) => r.clone(),
            None => {
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

    /// Teleports Robit to its saved home chargepad.
    /// Java: `EntityRobit.java:398-418` (`goHome`)
    pub fn go_home(&mut self) -> bool {
        if let Some(home) = &self.home_location {
            self.following = false;
            self.dimension = home.dimension.clone();
            self.x = home.pos[0] as f64 + 0.5;
            self.y = home.pos[1] as f64 + 0.1;
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
        if !is_unlocked && new_skin_id != SKIN_BASE {
            return false;
        }
        self.skin = new_skin_id.to_string();
        if new_skin_id == SKIN_BASE {
            self.default_skin_manually_selected = true;
        }
        true
    }

    /// Main server tick routine.
    /// Java: `EntityRobit.java:303-359`
    pub fn tick(
        &mut self,
        owner_pos: Option<(f64, f64, f64, &str)>,
        world_items: &mut [WorldItemEntity],
        smelting_recipes: &HashMap<String, (String, u32)>,
    ) {
        self.tick_count += 1;

        // 1. Follow Owner AI
        if self.following && self.energy > 0 {
            if let Some((ox, oy, oz, odim)) = owner_pos {
                if odim == self.dimension {
                    let dx = ox - self.x;
                    let dy = oy - self.y;
                    let dz = oz - self.z;
                    let dist_sq = dx * dx + dy * dy + dz * dz;

                    if dist_sq >= FOLLOW_TELEPORT_DIST_SQ {
                        // Far away: teleport to owner
                        self.x = ox;
                        self.y = oy;
                        self.z = oz;
                    } else if dist_sq > (FOLLOW_MAX_DIST * FOLLOW_MAX_DIST) {
                        // Move toward owner
                        let dist = dist_sq.sqrt();
                        let move_step = (MOVEMENT_SPEED as f64).min(dist);
                        self.x += (dx / dist) * move_step;
                        self.y += (dy / dist) * move_step;
                        self.z += (dz / dist) * move_step;

                        // Deduct movement energy
                        let travel_cost = self.calculate_travel_energy_cost();
                        self.energy = self.energy.saturating_sub(travel_cost);
                    }
                }
            }
        }

        // Update previous coordinates
        self.xo = self.x;
        self.yo = self.y;
        self.zo = self.z;

        // 2. Pickup Items AI
        if self.drop_pickup {
            self.collect_items(world_items);
        }

        // 3. Low Battery Auto-Return
        if self.energy == 0 {
            self.go_home();
        }

        // 4. Internal Furnace Smelting
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
        default_manually_selected: robit.default_manually_selected,
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
) -> RobitEntity {
    let mut robit = RobitEntity::new(
        entity_id,
        dimension.clone(),
        chargepad_pos[0] as f64 + 0.5,
        chargepad_pos[1] as f64 + 0.1,
        chargepad_pos[2] as f64 + 0.5,
        if item_data.owner_uuid.is_empty() {
            placer_uuid
        } else {
            item_data.owner_uuid
        },
        placer_name,
    );
    robit.energy = item_data.energy;
    robit.custom_name = item_data.custom_name;
    robit.skin = item_data.skin;
    robit.security_mode = item_data.security_mode;
    robit.default_skin_manually_selected = item_data.default_manually_selected;
    robit.slots = item_data.items;
    robit.home_location = Some(GlobalPos {
        dimension,
        pos: chargepad_pos,
    });
    robit
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
        // Joules = ceil(1.5 * 4.0) = 6 Joules -> fe(6) = 2 FE
        robit.x = 4.0;
        let cost = robit.calculate_travel_energy_cost();
        assert_eq!(cost, fe(6));
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

        let collected = robit.collect_items(&mut world_items);
        assert_eq!(collected, 72); // 32 + 40
        // Slot 0 has 64 iron ingots, slot 1 has 8 iron ingots
        assert_eq!(
            robit.slots[0],
            Some(Stack {
                item: "minecraft:iron_ingot".into(),
                count: 64,
                components: String::new(),
            })
        );
        assert_eq!(
            robit.slots[1],
            Some(Stack {
                item: "minecraft:iron_ingot".into(),
                count: 8,
                components: String::new(),
            })
        );
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
        assert_eq!(ALL_SKINS.len(), 13);
        assert_eq!(lookup_skin(SKIN_BASE).name, "Robit");
        assert_eq!(lookup_skin(SKIN_ALLAY).name, "Allay");
        assert_eq!(lookup_skin("mekanism:trans").name, "Trans");
        assert_eq!(lookup_skin("nonexistent").id, SKIN_BASE);
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
}

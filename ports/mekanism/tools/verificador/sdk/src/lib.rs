#![allow(dead_code, unused, non_snake_case, clippy::all, ambiguous_glob_reexports)]
pub mod screen_format;
pub mod server {
  pub use serde_json;
  pub mod players { use super::*;
    pub fn tell_overlay<A0, A1>(_a0: A0, _a1: A1) -> bool { false }
    pub fn open_item_menu<A0, A1, A2>(_a0: A0, _a1: A1, _a2: A2) -> bool { false }
    pub fn open_player_menu<A0, A1>(_a0: A0, _a1: A1) -> bool { false }
    pub fn open_entity_menu<A0, A1, A2>(_a0: A0, _a1: A1, _a2: A2) -> bool { false }
    #[derive(Clone, Debug, Default, PartialEq)] pub struct RadialEntry { pub id: String, pub label: String, pub icon: String }
    pub fn open_radial_menu<A0>(_a0: A0, _a1: &[RadialEntry], _a2: u32) -> bool { false }
    pub fn set_hud_line<A0, A1, A2, A3, A4>(_a0: A0, _a1: A1, _a2: A2, _a3: A3, _a4: A4) -> bool { false }
    #[derive(Clone, Debug, Default, PartialEq)] pub struct Player { pub id: String, pub name: String, pub x: f64, pub y: f64, pub z: f64, pub dimension: String, pub game_mode: GameMode, pub health: f32 }
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)] pub enum GameMode { #[default] Survival, Creative, Adventure, Spectator }
    pub fn online<>() -> Vec<Player> { Default::default() }
    pub fn tell<A0, A1>(_a0: A0, _a1: A1) -> bool { Default::default() }
    pub fn broadcast<A0>(_a0: A0) -> () { Default::default() }
  }
  pub mod player_control { use super::*;
    #[derive(Clone, Debug, Default, PartialEq)] pub struct Motion { pub sneaking: bool, pub on_ground: bool, pub jumps: u32, pub xp_level: i32, pub xp_progress: f32 }
    pub fn get_motion<A0>(_a0: A0) -> Option<Motion> { None }
    pub fn get_look<A0>(_a0: A0) -> Option<(f32,f32)> { None }
    pub fn teleport<A0, A1, A2, A3, A4>(_a0: A0, _a1: A1, _a2: A2, _a3: A3, _a4: A4, _a5: Option<f32>, _a6: Option<f32>) -> Result<(), String> { Err(String::new()) }
    pub fn take_xp<A0, A1>(_a0: A0, _a1: A1) -> Result<(), String> { Err(String::new()) }
    #[derive(Clone, Debug, Default, PartialEq)] pub struct EffectState { pub id: String, pub duration: i32, pub amplifier: i32 }
    #[derive(Clone, Debug, Default, PartialEq)] pub struct PlayerState { pub velocity_x: f64, pub velocity_y: f64, pub velocity_z: f64, pub jumping: bool, pub sprinting: bool, pub flying: bool, pub may_fly: bool, pub fall_flying: bool, pub swimming: bool, pub in_water: bool, pub eyes_in_water: bool, pub in_lava: bool, pub air: i32, pub max_air: i32, pub fire_ticks: i32, pub passenger: bool, pub sleeping: bool, pub food: u32, pub saturation: f32, pub effects: Vec<EffectState>, pub blocking: bool }
    pub fn set_air<A0, A1>(_a0: A0, _a1: A1) -> Result<(), String> { Err(String::new()) }
    pub fn reduce_effect<A0, A1, A2>(_a0: A0, _a1: A1, _a2: A2) -> Result<(), String> { Err(String::new()) }
    pub fn reset_fall_distance<A0>(_a0: A0) -> Result<(), String> { Err(String::new()) }
    pub fn set_attribute_modifier<A0, A1, A2, A3, A4>(_a0: A0, _a1: A1, _a2: A2, _a3: A3, _a4: A4) -> Result<(), String> { Err(String::new()) }
    pub fn remove_attribute_modifier<A0, A1, A2>(_a0: A0, _a1: A1, _a2: A2) -> Result<(), String> { Err(String::new()) }
    pub fn get_state<A0>(_a0: A0) -> Option<PlayerState> { None }
    pub fn set_food<A0, A1, A2>(_a0: A0, _a1: A1, _a2: A2) -> Result<(), String> { Err(String::new()) }
    pub fn set_motion<A0, A1, A2, A3>(_a0: A0, _a1: A1, _a2: A2, _a3: A3) -> Result<(), String> { Err(String::new()) }
    pub fn set_allow_flight<A0, A1>(_a0: A0, _a1: A1) -> Result<(), String> { Err(String::new()) }
    pub fn play_sound<A0, A1, A2, A3, A4, A5, A6>(_a0: A0, _a1: A1, _a2: A2, _a3: A3, _a4: A4, _a5: A5, _a6: A6) -> () { Default::default() }
  }
  pub mod recipe_types { use super::*;
    #[derive(Clone, Debug, Default, PartialEq)] pub struct CraftMatch { pub id: String, pub output: ItemStack, pub remaining: Vec<Option<ItemStack>> }
    pub fn crafting_find(_a0: &Vec<Option<ItemStack>>) -> Option<CraftMatch> { None }
    pub fn crafting_matches<A0>(_a0: A0, _a1: &Vec<Option<ItemStack>>) -> bool { false }
    pub fn item_tags<A0>(_a0: A0) -> Vec<String> { Vec::new() }
    pub fn item_tag_members<A0>(_a0: A0) -> Vec<String> { Vec::new() }
    #[derive(Clone, Debug, Default, PartialEq)] pub struct Durability { pub damage: u32, pub max_damage: u32 }
    #[derive(Clone, Debug, Default, PartialEq)] pub struct AnvilAnswer { pub output: ItemStack, pub cost: i32, pub material_cost: u32 }
    pub fn item_durability(stack: &ItemStack) -> Option<Durability> { let _ = stack; None }
    pub fn item_set_damage(stack: &ItemStack, damage: u32) -> Option<ItemStack> { let _ = (stack, damage); None }
    pub fn item_repairs_with(stack: &ItemStack, material: &ItemStack) -> bool { let _ = (stack, material); false }
    pub fn item_enchantments(stack: &ItemStack) -> Vec<(String, u32)> { let _ = stack; Vec::new() }
    pub fn anvil_result(left: &ItemStack, right: Option<&ItemStack>, name: Option<&str>) -> Option<AnvilAnswer> { let _ = (left, right, name); None }
    #[derive(Clone, Debug, Default, PartialEq)] pub struct EnergyInfo { pub stored: u64, pub capacity: u64, pub max_insert: u64, pub max_extract: u64 }
    pub fn item_energy(stack: &ItemStack) -> Option<EnergyInfo> { let _ = stack; None }
    pub fn item_energy_insert(stack: &ItemStack, amount: u64, simulate: bool) -> (ItemStack, u64) { let _ = (amount, simulate); (stack.clone(), 0) }
    pub fn item_energy_extract(stack: &ItemStack, amount: u64, simulate: bool) -> (ItemStack, u64) { let _ = (amount, simulate); (stack.clone(), 0) }
    pub fn item_max_stack_size<A0>(_a0: A0) -> Option<u32> { None }
    pub fn burn_time<A0>(_a0: A0) -> Option<u32> { None }
    #[derive(Clone, Debug, Default, PartialEq)] pub struct CustomRecipe { pub id: String, pub json: String }
    #[derive(Clone, Debug, Default, PartialEq)] pub struct CookingRecipe { pub inputs: Vec<String>, pub output: String, pub count: u32, pub experience: f32, pub time: u32 }
    pub fn recipes_of_type<A0>(_a0: A0) -> Vec<CustomRecipe> { Default::default() }
    pub fn cooking_recipes<A0>(_a0: A0) -> Vec<CookingRecipe> { Default::default() }
    pub fn ingredient_items<A0>(_a0: A0) -> Vec<String> { Default::default() }
  }
  pub mod entities { use super::*;
    #[derive(Clone, Debug, Default, PartialEq)] pub struct EntityInBox { pub id: u32, pub kind: String, pub x: f64, pub y: f64, pub z: f64, pub vehicle: Option<u32>, pub player: Option<String> }
    pub fn entities_in_box(dimension: &str, min: (f64, f64, f64), max: (f64, f64, f64)) -> Vec<EntityInBox> { let _ = (dimension, min, max); Vec::new() }
    #[allow(clippy::too_many_arguments)]
    pub fn teleport_entity(dimension: &str, id: u32, to_dimension: &str, x: f64, y: f64, z: f64, yaw: Option<f32>) -> Result<(), String> { let _ = (dimension, id, to_dimension, x, y, z, yaw); Ok(()) }
    pub struct DroppedItem { pub id: u32, pub stack: crate::server::ItemStack, pub x: f64, pub y: f64, pub z: f64 }
    pub fn items_in_box<A0, A1, A2>(_a0: A0, _a1: A1, _a2: A2) -> Vec<DroppedItem> { Vec::new() }
    pub fn take_dropped_item<A0, A1, A2>(_a0: A0, _a1: A1, _a2: A2) -> Result<(), String> { Err(String::new()) }
    pub struct DamageResult { pub id: u32, pub player: Option<String>, pub applied: f32, pub blocked: bool }
    pub fn damage_entities_in_box<A0, A1, A2, A3, A4, A5>(_a0: A0, _a1: A1, _a2: A2, _a3: A3, _a4: A4, _a5: A5) -> Vec<DamageResult> { Vec::new() }
    pub fn spawn_mod_entity<A0, A1, A2, A3, A4>(_a0: A0, _a1: A1, _a2: A2, _a3: A3, _a4: A4, _a5: &[u8]) -> Result<u64, String> { Err(String::new()) }
    #[derive(Clone, Debug, Default, PartialEq)] pub struct Mob { pub id: u32, pub kind: String, pub x: f64, pub y: f64, pub z: f64, pub yaw: f32, pub health: f32, pub max_health: f32 }
    #[derive(Clone, Debug, Default, PartialEq)] pub struct MobOptions { pub name: Option<String>, pub health: Option<f32>, pub no_ai: bool, pub invisible: bool, pub silent: bool, pub invulnerable: bool, pub no_gravity: bool }
    #[derive(Clone, Debug, Default, PartialEq)] pub struct Display { pub translation: (f32,f32,f32), pub left_rotation: (f32,f32,f32,f32), pub scale: (f32,f32,f32), pub right_rotation: (f32,f32,f32,f32), pub interpolation: i32, pub teleport: i32 }
    pub fn mobs<A0>(_a0: A0) -> Vec<Mob> { Default::default() }
    pub fn spawn_mob<A0, A1, A2, A3, A4, A5>(_a0: A0, _a1: A1, _a2: A2, _a3: A3, _a4: A4, _a5: A5) -> Result<u64, String> { Err(String::new()) }
    pub fn summon<A0, A1, A2, A3, A4, A5>(_a0: A0, _a1: A1, _a2: A2, _a3: A3, _a4: A4, _a5: A5) -> Result<u64, String> { Err(String::new()) }
    pub fn move_entity<A0, A1, A2, A3, A4, A5, A6>(_a0: A0, _a1: A1, _a2: A2, _a3: A3, _a4: A4, _a5: A5, _a6: A6) -> Result<(), String> { Err(String::new()) }
    pub fn set_display<A0, A1, A2>(_a0: A0, _a1: A1, _a2: A2) -> Result<(), String> { Err(String::new()) }
    pub fn remove_entity<A0, A1>(_a0: A0, _a1: A1) -> Result<(), String> { Err(String::new()) }
    pub fn teleport_mob<A0, A1, A2, A3, A4>(_a0: A0, _a1: A1, _a2: A2, _a3: A3, _a4: A4) -> Result<(), String> { Err(String::new()) }
    pub fn push_mob<A0, A1, A2, A3, A4>(_a0: A0, _a1: A1, _a2: A2, _a3: A3, _a4: A4) -> Result<(), String> { Err(String::new()) }
    pub fn set_mob_health<A0, A1, A2>(_a0: A0, _a1: A1, _a2: A2) -> Result<(), String> { Err(String::new()) }
    pub fn kill_mob<A0, A1>(_a0: A0, _a1: A1) -> Result<(), String> { Err(String::new()) }
    pub fn remove_mob<A0, A1>(_a0: A0, _a1: A1) -> Result<(), String> { Err(String::new()) }
    pub fn set_entity_fire<A0, A1, A2>(_a0: A0, _a1: A1, _a2: A2) -> Result<(), String> { Err(String::new()) }
  }
  pub mod player_items { use super::*;
    pub fn get_inventory<A0>(_a0: A0) -> Option<Vec<Option<ItemStack>>> { None }
    pub fn selected_slot<A0>(_a0: A0) -> Option<u32> { None }
    pub fn set_slot<A0, A1, A2>(_a0: A0, _a1: A1, _a2: A2) -> Result<(), String> { Err(String::new()) }
    pub fn compare_and_set_slot<A0, A1, A2, A3>(_a0: A0, _a1: A1, _a2: A2, _a3: A3) -> Result<(), String> { Err(String::new()) }
    pub fn hurt<A0, A1, A2>(_a0: A0, _a1: A1, _a2: A2) -> Result<(), String> { Err(String::new()) }
    pub fn add_effect<A0, A1, A2, A3>(_a0: A0, _a1: A1, _a2: A2, _a3: A3) -> Result<(), String> { Err(String::new()) }
  }
  pub mod chunks { use super::*;
    pub fn set_forced<A0, A1, A2, A3>(_a0: A0, _a1: A1, _a2: A2, _a3: A3) -> Result<(), String> { Err(String::new()) }
  }
  pub mod world_data {
    pub fn get(key: &str) -> Option<Vec<u8>> { let _ = key; None }
    pub fn set(key: &str, value: &[u8]) -> Result<(), String> { let _ = (key, value); Ok(()) }
    pub fn remove(key: &str) { let _ = key; }
    pub fn keys(prefix: &str) -> Vec<String> { let _ = prefix; Vec::new() }
  }
  pub mod world { use super::*;
    pub fn set_comparator_output<A0, A1, A2, A3, A4>(_a0: A0, _a1: A1, _a2: A2, _a3: A3, _a4: A4) -> Result<(), String> { Err(String::new()) }
    pub fn set_redstone_output<A0, A1, A2, A3>(_a0: A0, _a1: A1, _a2: A2, _a3: A3, _a4: u8, _a5: bool) -> Result<(), String> { Err(String::new()) }
    pub fn set_menu_grid(dimension: &str, x: i32, y: i32, z: i32, entries: &[super::GridEntry]) -> Result<(), String> { let _ = (dimension, x, y, z, entries); Ok(()) }
    pub fn set_item_eject_color(dimension: &str, x: i32, y: i32, z: i32, color: Option<u8>) -> Result<(), String> { let _ = (dimension, x, y, z, color); Ok(()) }
    pub fn set_item_strict_input(dimension: &str, x: i32, y: i32, z: i32, strict: bool) -> Result<(), String> { let _ = (dimension, x, y, z, strict); Ok(()) }
    pub fn set_transmitter_color(dimension: &str, x: i32, y: i32, z: i32, color: Option<u8>) -> Result<(), String> { let _ = (dimension, x, y, z, color); Ok(()) }
    pub fn break_blocks<A0, A1, A2, A3, A4>(_a0: A0, _a1: A1, _a2: A2, _a3: A3, _a4: A4) -> Result<u32, String> { Err(String::new()) }
    pub fn block_hardness<A0, A1, A2, A3>(_a0: A0, _a1: A1, _a2: A2, _a3: A3) -> Option<f32> { None }
    pub fn set_menu_text<A0, A1, A2, A3, A4>(_a0: A0, _a1: A1, _a2: A2, _a3: A3, _a4: A4) -> Result<(), String> { Err(String::new()) }
    pub fn pull_items<A0, A1, A2, A3, A4, A5, A6>(_a0: A0, _a1: A1, _a2: A2, _a3: A3, _a4: A4, _a5: A5, _a6: A6) -> Result<(), String> { Ok(()) }
    pub fn mine_block<A0,A1,A2,A3,A4,A5,A6,A7,A8,A9>(_a0:A0,_a1:A1,_a2:A2,_a3:A3,_a4:A4,_a5:A5,_a6:A6,_a7:A7,_a8:A8,_a9:A9) -> Result<(), String> { Ok(()) }
    #[derive(Clone, Debug, Default, PartialEq)] pub struct BlockChange { pub x: i32, pub y: i32, pub z: i32, pub state: String }
    #[derive(Clone, Debug, Default, PartialEq)] pub struct Environment { pub day_time: u64, pub rain: f32, pub thunder: f32, pub temperature: f32, pub downfall: f32, pub precipitation: bool, pub has_skylight: bool }
    #[derive(Clone, Debug, Default, PartialEq)] pub struct CollisionBox { pub min_x: f64, pub min_y: f64, pub min_z: f64, pub max_x: f64, pub max_y: f64, pub max_z: f64 }
    pub fn get_block<A0, A1, A2, A3>(_a0: A0, _a1: A1, _a2: A2, _a3: A3) -> Option<String> { None }
    pub fn get_biome<A0, A1, A2, A3>(_a0: A0, _a1: A1, _a2: A2, _a3: A3) -> Option<String> { None }
    pub fn get_environment<A0, A1, A2, A3>(_a0: A0, _a1: A1, _a2: A2, _a3: A3) -> Option<Environment> { None }
    pub fn get_redstone<A0, A1, A2, A3>(_a0: A0, _a1: A1, _a2: A2, _a3: A3) -> u8 { Default::default() }
    pub fn set_energy_capacity<A0, A1, A2, A3, A4>(_a0: A0, _a1: A1, _a2: A2, _a3: A3, _a4: A4) -> Result<(), String> { Err(String::new()) }
    pub fn get_collision<A0, A1, A2, A3>(_a0: A0, _a1: A1, _a2: A2, _a3: A3) -> Option<Vec<CollisionBox>> { None }
    pub fn set_blocks<A0, A1>(_a0: A0, _a1: A1) -> Result<(), String> { Err(String::new()) }
    pub fn set_block_property<A0, A1, A2, A3, A4, A5>(_a0: A0, _a1: A1, _a2: A2, _a3: A3, _a4: A4, _a5: A5) -> Result<(), String> { Err(String::new()) }
    pub fn set_item_faces<A0, A1, A2, A3, A4>(_a0: A0, _a1: A1, _a2: A2, _a3: A3, _a4: A4) -> Result<(), String> { Err(String::new()) }
    pub fn pack_block<A0, A1, A2, A3, A4, A5>(_a0: A0, _a1: A1, _a2: A2, _a3: A3, _a4: A4, _a5: A5, _a6: i32, _r: Option<&str>) -> Result<(), String> { Err(String::new()) }
    pub fn unpack_block<A0, A1, A2, A3, A4, A5, A6>(_a0: A0, _a1: A1, _a2: A2, _a3: A3, _a4: A4, _a5: A5, _a6: A6) -> Result<(), String> { Err(String::new()) }
    pub fn spawn_particles<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12>(_a0: A0, _a1: A1, _a2: A2, _a3: A3, _a4: A4, _a5: A5, _a6: A6, _a7: A7, _a8: A8, _a9: A9, _a10: A10, _a11: A11, _a12: A12) -> Result<(), String> { Err(String::new()) }
    pub fn set_capabilities_enabled<A0, A1, A2, A3, A4, A5, A6>(_a0: A0, _a1: A1, _a2: A2, _a3: A3, _a4: A4, _a5: A5, _a6: A6) -> Result<(), String> { Err(String::new()) }
    pub fn set_item_eject<A0, A1, A2, A3, A4>(_a0: A0, _a1: A1, _a2: A2, _a3: A3, _a4: A4) -> Result<(), String> { Err(String::new()) }
    pub fn set_tank_filter<A0, A1, A2, A3, A4, A5>(_a0: A0, _a1: A1, _a2: A2, _a3: A3, _a4: A4, _a5: A5) -> Result<(), String> { Err(String::new()) }
  }
  pub mod storage { use super::*;
    pub fn get<A0>(_a0: A0) -> Option<Vec<u8>> { None }
    pub fn set<A0, A1>(_a0: A0, _a1: A1) -> Result<(), String> { Err(String::new()) }
    pub fn delete<A0>(_a0: A0) -> () { Default::default() }
  }
  pub mod log { use super::*;
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)] pub enum Level { #[default] Debug, Info, Warn, Error }
    pub fn write<A0, A1>(_a0: A0, _a1: A1) -> () { Default::default() }
  }
  pub mod item_types { use super::*;
    #[derive(Clone, Debug, Default, PartialEq, Eq, Hash)] pub struct ItemStack { pub item: String, pub count: u32, pub components: String }
  }
  pub mod net { use super::*;
    pub fn send<A0, A1, A2>(_a0: A0, _a1: A1, _a2: A2) -> Result<(), String> { Err(String::new()) }
  }
  pub mod server_events { use super::*;
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)] pub struct BlockPos { pub x: i32, pub y: i32, pub z: i32 }
    #[derive(Clone, Debug, Default, PartialEq)] pub struct Argument { pub name: String, pub kind: ArgumentKind, pub optional: bool }
    #[derive(Clone, Debug, Default, PartialEq)] pub struct ServerCommand { pub name: String, pub arguments: Vec<Argument> }
    #[derive(Clone, Debug, Default, PartialEq)] pub struct Registration { pub commands: Vec<ServerCommand>, pub tick: bool, pub joins: bool, pub chat: bool, pub blocks: bool, pub block_entities: Vec<BlockEntityKind>, pub net: bool, pub items: bool, pub chunks: bool, pub damage: bool, pub redirects: Vec<RedirectSpec> }
    #[derive(Clone, Debug, Default, PartialEq)] pub struct BlockEntityKind { pub block: String, pub ticks: bool, pub container: Option<ContainerSpec>, pub energy: Option<EnergySpec>, pub tanks: Vec<TankSpec>, pub keep_data: bool, pub transmitter: Option<TransmitterSpec>, pub multiblock: Option<MultiblockSpec> }
    #[derive(Clone, Debug, Default, PartialEq)] pub struct MultiblockSpec { pub group: String }
    #[derive(Clone, Debug, Default, PartialEq)] pub struct MultiblockMember { pub pos: BlockPos, pub state: String }
    #[derive(Clone, Debug, Default, PartialEq)] pub struct Structure { pub members: Vec<MultiblockMember>, pub min: BlockPos, pub max: BlockPos }
    #[derive(Clone, Debug, Default, PartialEq)] pub struct TransmitterSpec { pub medium: TransmitterMedium, pub throughput: u64 }
    #[derive(Clone, Debug, Default, PartialEq)] pub struct TankSpec { pub kind: String, pub capacity: u64, pub max_rate: u64, pub faces: Vec<FlowSide> }
    #[derive(Clone, Debug, Default, PartialEq)] pub struct Tank { pub substance: String, pub amount: u64 }
    #[derive(Clone, Debug, Default, PartialEq)] pub struct EnergySpec { pub capacity: u64, pub max_receive: u64, pub max_extract: u64, pub faces: Vec<FlowSide> }
    #[derive(Clone, Debug, Default, PartialEq)] pub struct ContainerSpec { pub slots: u32, pub faces: Vec<Vec<u32>>, pub inputs: Vec<u32>, pub outputs: Vec<u32>, pub menu: bool }
    #[derive(Clone, Debug, Default, PartialEq)] pub struct BlockEntity { pub dimension: String, pub pos: BlockPos, pub state: String, pub data: Vec<u8>, pub items: Vec<Option<ItemStack>>, pub energy: u64, pub energy_faces: Vec<FlowSide>, pub tanks: Vec<Tank>, pub tank_faces: Vec<Vec<FlowSide>>, pub menu_data: Vec<i32>, pub transmitter_faces: Vec<bool> }
    #[derive(Clone, Debug, Default, PartialEq)] pub struct BlockEntityUpdate { pub dimension: String, pub pos: BlockPos, pub data: Vec<u8>, pub items: Option<Vec<Option<ItemStack>>>, pub energy: Option<u64>, pub energy_faces: Option<Vec<FlowSide>>, pub tanks: Option<Vec<Tank>>, pub tank_faces: Option<Vec<Vec<FlowSide>>>, pub menu_data: Option<Vec<i32>>, pub transmitter_faces: Option<Vec<bool>> }
    #[derive(Clone, Debug, Default, PartialEq)] pub struct RedirectSpec { pub block: String, pub flag_at: u32, pub pos_at: u32 }
    #[derive(Clone, Debug, Default, PartialEq)] pub struct DamageSource { pub damage_type: String, pub causing: Option<u32>, pub is_magic: bool, pub is_fire: bool, pub is_fall: bool, pub is_explosion: bool, pub tags: Vec<String> }
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)] pub enum BlockUse { #[default] Default, Pass, Consume }
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)] pub enum FlowSide { #[default] None, Input, Output, Both }
    #[derive(Clone, Debug, PartialEq)] pub enum ArgumentKind { Word, Text, Integer((Option<i32>,Option<i32>)), Number, Boolean, Player, BlockPos }
    impl Default for ArgumentKind { fn default() -> Self { unimplemented!() } }
    #[derive(Clone, Debug, PartialEq)] pub enum ArgumentValue { Word(String), Text(String), Integer(i32), Number(f64), Boolean(bool), Player(Player), BlockPos(BlockPos) }
    impl Default for ArgumentValue { fn default() -> Self { unimplemented!() } }
    #[derive(Clone, Debug, PartialEq)] pub enum TransmitterMedium { Energy, Tank(String), Items }
    impl Default for TransmitterMedium { fn default() -> Self { unimplemented!() } }
    pub fn init<>() -> Result<Registration, String> { Err(String::new()) }
    pub fn tick<A0>(_a0: A0) -> () { Default::default() }
    pub fn player_joined<A0>(_a0: A0) -> () { Default::default() }
    pub fn player_left<A0>(_a0: A0) -> () { Default::default() }
    pub fn chat<A0, A1>(_a0: A0, _a1: A1) -> () { Default::default() }
    pub fn command<A0, A1, A2, A3>(_a0: A0, _a1: A1, _a2: A2, _a3: A3) -> Result<Option<String>, String> { Err(String::new()) }
    pub fn block_broken<A0, A1, A2>(_a0: A0, _a1: A1, _a2: A2) -> () { Default::default() }
    pub fn block_placed<A0, A1, A2>(_a0: A0, _a1: A1, _a2: A2) -> () { Default::default() }
    pub fn block_entities_tick<A0, A1>(_a0: A0, _a1: A1) -> Vec<BlockEntityUpdate> { Default::default() }
    pub fn menu_button<A0, A1, A2, A3, A4>(_a0: A0, _a1: A1, _a2: A2, _a3: A3, _a4: A4) -> Option<BlockEntityUpdate> { None }
    pub fn net_message<A0, A1, A2>(_a0: A0, _a1: A1, _a2: A2) -> () { Default::default() }
    pub fn multiblock_changed<A0, A1, A2>(_a0: A0, _a1: A1, _a2: A2) -> Vec<BlockEntityUpdate> { Default::default() }
    pub fn entity_spawned<A0, A1>(_a0: A0, _a1: A1) -> () { Default::default() }
    pub fn chunk_loaded<A0, A1, A2, A3>(_a0: A0, _a1: A1, _a2: A2, _a3: A3) -> () { Default::default() }
    pub fn item_used<A0, A1, A2>(_a0: A0, _a1: A1, _a2: A2) -> bool { Default::default() }
    pub fn item_hit<A0, A1, A2, A3>(_a0: A0, _a1: A1, _a2: A2, _a3: A3) -> () { Default::default() }
  }
  pub mod command { use super::*;
    pub fn run<A0>(_a0: A0) -> () { Default::default() }
    pub fn run_quiet<A0>(_a0: A0) -> () { Default::default() }
  }
  pub use self::item_types::*;
  pub use self::server_events::*;
  pub use self::players::*;
  pub use self::entities::*;
  pub use self::player_items::*;
  pub use self::recipe_types::*;
  pub use self::player_control::*;
  pub use self::chunks::*;
  pub use self::command::*;
  pub use self::server_events::*; pub use self::world::*;
  impl Registration {
    pub fn on_tick(self) -> Self { self }
    pub fn on_items(self) -> Self { self }
    pub fn block_entity_kind<T>(self, _t: T) -> Self { self }
    pub fn on_chunks(self) -> Self { self }
    pub fn on_blocks(self) -> Self { self }
    pub fn on_damage(self) -> Self { self }
    pub fn on_item_release(self) -> Self { self }
    pub fn on_mod_entities(self) -> Self { self }
    pub fn on_block_break_start(self) -> Self { self }
    pub fn on_net(self) -> Self { self }
    pub fn redirect(self, _block: &str, _flag_at: u32, _pos_at: u32) -> Self { self }
    pub fn new() -> Self { Self::default() }
  }
  pub mod menu_input {
    use super::BlockPos;
    #[derive(Clone, Debug, PartialEq)] pub struct MenuInput { pub pos: BlockPos, pub id: i32, pub text: String }
    pub fn channel(mod_id: &str) -> String { format!("{mod_id}:menu_text") }
    pub fn decode(data: &[u8]) -> Option<MenuInput> {
        let int = |at: usize| data.get(at..at + 4).map(|b| i32::from_le_bytes([b[0], b[1], b[2], b[3]]));
        Some(MenuInput { pos: BlockPos { x: int(0)?, y: int(4)?, z: int(8)? }, id: int(12)?, text: String::from_utf8(data.get(16..)?.to_vec()).ok()? })
    }
  }
  #[derive(Clone, Debug, Default, PartialEq)] pub struct ModEntity { pub id: u32, pub dimension: String, pub kind: String, pub x: f64, pub y: f64, pub z: f64, pub yaw: f32, pub health: f32, pub data: Vec<u8>, pub items: Vec<Option<ItemStack>>, pub energy: u64, pub skin: String, pub name: Option<String>, pub menu_data: Vec<i32> }
  #[derive(Clone, Debug, Default, PartialEq)] pub struct ItemMenu { pub menu: String, pub slot: u32, pub stack: Option<ItemStack>, pub items: Vec<Option<ItemStack>>, pub menu_data: Vec<i32> }
  #[derive(Clone, Debug, Default, PartialEq)] pub struct ItemMenuUpdate { pub items: Option<Vec<Option<ItemStack>>>, pub menu_data: Option<Vec<i32>>, pub menu_text: Option<Vec<String>>, pub close: bool, pub menu_grid: Option<Vec<GridEntry>> }
  /// The `index` of a `menu_grid_click` on a cell with no entry (the grid itself).
  pub const GRID_EMPTY: u32 = u32::MAX;
  #[derive(Clone, Debug, Default, PartialEq, Eq)] pub struct GridEntry { pub item: String, pub components: String, pub count: u64 }
  #[derive(Clone, Debug, Default, PartialEq)] pub struct GridTarget { pub menu: String, pub dimension: String, pub pos: Option<BlockPos>, pub slot: Option<u32> }
  #[derive(Clone, Debug, Default, PartialEq)] pub struct GridAnswer { pub carried: Option<ItemStack>, pub to_inventory: Vec<ItemStack> }
  #[derive(Clone, Debug, Default, PartialEq)] pub struct ModEntityUpdate { pub dimension: String, pub id: u32, pub data: Option<Vec<u8>>, pub items: Option<Vec<Option<ItemStack>>>, pub energy: Option<u64>, pub skin: Option<String>, pub name: Option<Option<String>>, pub teleport: Option<(f64, f64, f64)>, pub path_to: Option<(f64, f64, f64, f32)>, pub stop: bool, pub health: Option<f32>, pub remove: bool, pub menu_data: Option<Vec<i32>>, pub menu_text: Option<Vec<String>> }
  pub trait ServerMod: Sized {
    fn key_pressed(&mut self, player: &Player, key: &str) -> bool { false }
    fn menu_grid_click(&mut self, player: &Player, target: GridTarget, index: u32, button: u8, shift: bool, carried: Option<ItemStack>) -> Option<GridAnswer> { None }
    fn item_menu_tick(&mut self, player: &Player, menu: ItemMenu) -> Option<ItemMenuUpdate> { None }
    fn item_menu_button(&mut self, player: &Player, menu: ItemMenu, id: i32) -> Option<ItemMenuUpdate> { None }
    fn item_menu_taken(&mut self, player: &Player, menu: ItemMenu, slot: u32, stack: ItemStack) -> Option<ItemMenuUpdate> { None }
    fn block_break_start(&mut self, player: &Player, stack: ItemStack, pos: BlockPos, state: &str) -> bool { false }
    fn entities_tick(&mut self, tick: u64, entities: Vec<ModEntity>) -> Vec<ModEntityUpdate> { Vec::new() }
    fn mod_entity_used(&mut self, player: &Player, entity: ModEntity, sneaking: bool) -> bool { false }
    fn entity_menu_button(&mut self, player: &Player, entity: ModEntity, id: i32) -> Option<ModEntityUpdate> { None }
    fn entity_menu_taken(&mut self, player: &Player, entity: ModEntity, slot: u32, stack: ItemStack) -> Option<ModEntityUpdate> { None }
    fn mod_entity_damaged(&mut self, entity: ModEntity, source: DamageSource, amount: f32) -> f32 { amount }
    fn mod_entity_died(&mut self, entity: ModEntity) {}
    fn init() -> Result<(Self, Registration), String>;
    fn chunk_loaded(&mut self, dimension: &str, chunk_x: i32, chunk_z: i32, new: bool) {}
    fn tick(&mut self, tick: u64) {}
    fn block_broken(&mut self, player: &Player, pos: BlockPos, state: &str) {}
    fn block_placed(&mut self, player: &Player, pos: BlockPos, state: &str) {}
    fn entity_spawned(&mut self, ticket: u64, id: Option<u32>) {}
    fn item_used(&mut self, player: &Player, stack: ItemStack, pos: Option<BlockPos>) -> bool { false }
    fn item_hit(&mut self, player: &Player, stack: ItemStack, entity: u32, kind: &str) {}
    fn block_used(&mut self, player: &Player, pos: BlockPos, sneaking: bool, stack: Option<ItemStack>) -> BlockUse { BlockUse::Default }
    fn can_place(&mut self, player: &Player, pos: BlockPos, state: &str) -> bool { true }
    fn player_damaged(&mut self, player: &Player, source: DamageSource, amount: f32) -> f32 { amount }
    fn item_hit_damage(&mut self, player: &Player, stack: ItemStack, kind: &str, damage: f32) -> f32 { damage }
    fn item_released(&mut self, player: &Player, stack: ItemStack, hand: &str, ticks_held: u32) -> bool { true }
    fn item_shot(&mut self, player: &Player, stack: ItemStack, projectiles: Vec<u32>) {}
    fn menu_button(&mut self, _player: &Player, _pos: BlockPos, id: i32, e: BlockEntity) -> Option<BlockEntityUpdate> { None }
    fn net_message(&mut self, _player: &Player, _channel: &str, _data: &[u8]) {}
    fn multiblock_changed(&mut self, group: &str, dimension: &str, structures: Vec<Structure>) -> Vec<BlockEntityUpdate> { Vec::new() }
    fn block_entities_tick(&mut self, tick: u64, mut entities: Vec<BlockEntity>) -> Vec<BlockEntityUpdate> { Vec::new() }
  }
}
#[macro_export] macro_rules! export_server_mod { ($t:ty) => {}; }

/// Stub of `horizon_mod_sdk::screen` (capability `screen`, API 56): the same types, `Canvas` and widgets as the SDK
/// (the format and the widgets are copies of the fork's), `host` answering from what a test set with `host::set_menu`
/// and keeping what the screen sent (`host::take_sent`).
pub mod screen {
    pub use crate::screen_format::{
        self as format, AreaKind, Canvas, FIELD_FOCUS, FIELD_NO_BORDER, FieldFilter, ITEM_DECORATIONS, Rect, TEXT_CENTER,
        TEXT_RIGHT, TEXT_SHADOW,
    };
    #[derive(Clone, Debug, PartialEq)]
    pub struct OpenInfo { pub menu: String, pub gui_width: f32, pub gui_height: f32, pub touch: bool }
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Setup { pub width: u32, pub height: u32, pub hide_inventory: bool, pub own_labels: bool }
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Frame { pub mouse_x: f32, pub mouse_y: f32, pub partial_tick: f32, pub time_ms: u64 }
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum MouseButton { Left, Right, Middle }
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    pub struct Modifiers { pub shift: bool, pub ctrl: bool, pub alt: bool }
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Pointer { pub x: f32, pub y: f32, pub button: MouseButton, pub modifiers: Modifiers }
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Drag { pub x: f32, pub y: f32, pub button: MouseButton, pub dx: f32, pub dy: f32 }
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Wheel { pub x: f32, pub y: f32, pub delta: f32 }
    #[derive(Clone, Debug, PartialEq)]
    pub struct Key { pub key: String, pub modifiers: Modifiers }
    #[derive(Clone, Debug, PartialEq)]
    pub struct FieldText { pub id: i32, pub text: String }
    #[derive(Clone, Debug, PartialEq)]
    pub enum InputEvent {
        MousePressed(Pointer),
        MouseReleased(Pointer),
        MouseDragged(Drag),
        MouseScrolled(Wheel),
        KeyPressed(Key),
        CharTyped(String),
        FieldChanged(FieldText),
        FieldSubmitted(FieldText),
    }

    pub mod host {
        use std::cell::RefCell;
        #[derive(Clone, Debug, PartialEq, Default)]
        pub struct Stack { pub item: String, pub components: String, pub count: u32 }
        #[derive(Clone, Debug, PartialEq, Default)]
        pub struct GridEntry { pub item: String, pub components: String, pub count: u64 }
        /// What the stub's host answers (a test sets it with [`set_menu`]).
        #[derive(Clone, Debug, Default)]
        pub struct Menu {
            pub data: Vec<i32>,
            pub strings: Vec<String>,
            pub grid: Vec<GridEntry>,
            pub slots: Vec<Option<Stack>>,
            pub carried: Option<Stack>,
        }
        thread_local! {
            static MENU: RefCell<Menu> = RefCell::new(Menu::default());
            static SENT: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
        }
        /// Stub only: what the menu holds for the next calls.
        pub fn set_menu(menu: Menu) {
            MENU.with(|m| *m.borrow_mut() = menu);
        }
        /// Stub only: what the screen sent since the last call (`button 12`, `text 1 "Home"`, `grid 3 0`, `click`, `close`).
        pub fn take_sent() -> Vec<String> {
            SENT.with(|s| std::mem::take(&mut *s.borrow_mut()))
        }
        fn sent(line: String) {
            SENT.with(|s| s.borrow_mut().push(line));
        }
        pub fn menu_data() -> Vec<i32> { MENU.with(|m| m.borrow().data.clone()) }
        pub fn menu_strings() -> Vec<String> { MENU.with(|m| m.borrow().strings.clone()) }
        pub fn menu_grid() -> Vec<GridEntry> { MENU.with(|m| m.borrow().grid.clone()) }
        pub fn slot(index: u32) -> Option<Stack> { MENU.with(|m| m.borrow().slots.get(index as usize).cloned().flatten()) }
        pub fn carried() -> Option<Stack> { MENU.with(|m| m.borrow().carried.clone()) }
        /// 6 a character, 4 a space (the game's GUI font differs: exact widths only in the game).
        pub fn text_width(text: &str) -> f32 { text.chars().map(|c| if c == ' ' { 4.0 } else { 6.0 }).sum() }
        pub fn texture_size(_texture: &str) -> Option<(u32, u32)> { None }
        pub fn translate(key: &str, _args: &[String]) -> String { key.to_owned() }
        pub fn send_button(id: i32) { sent(format!("button {id}")) }
        pub fn send_text(id: i32, text: &str) { sent(format!("text {id} {text:?}")) }
        pub fn send_grid_click(index: u32, button: u8, shift: bool) {
            sent(format!("grid {index} {button}{}", if shift { " shift" } else { "" }))
        }
        pub fn play_click() { sent("click".into()) }
        pub fn close() { sent("close".into()) }
    }

    pub trait ScreenMod: Sized + 'static {
        fn open(info: &OpenInfo) -> Result<(Self, Setup), String>;
        fn tick(&mut self) {}
        fn render(&mut self, frame: &Frame, canvas: &mut Canvas);
        fn input(&mut self, _event: &InputEvent) -> bool { false }
        fn close(&mut self) {}
    }

    pub mod widgets {
        use super::{AreaKind, Canvas, Rect};

        /// `GuiUtils.renderBackgroundTexture`: `texture` (`tex` pixels big) nine-sliced over `rect` with `side`-pixel
        /// borders, its edges and middle repeated, as `blitNineSlicedSized` does.
        pub fn background(c: &mut Canvas, texture: &str, side: f32, rect: Rect, tex: [f32; 2]) {
            c.blit_nine_sliced(
                texture,
                rect,
                [side, side, side, side],
                [0.0, 0.0, tex[0], tex[1]],
                tex,
                u32::MAX,
            );
        }

        fn clamp(v: i32, lo: i32, hi: i32) -> i32 {
            v.max(lo).min(hi)
        }

        /// `GuiScrollableElement`: a scroll position from 0 to 1 and the bar's thumb.
        #[derive(Clone, Debug, PartialEq)]
        pub struct Scrollable {
            pub x: i32,
            pub y: i32,
            pub width: i32,
            pub height: i32,
            pub bar_x_shift: i32,
            pub bar_x: i32,
            pub bar_y: i32,
            pub bar_width: i32,
            pub bar_height: i32,
            pub max_bar_height: i32,
            /// 0 at the top, 1 at the bottom.
            pub scroll: f64,
            drag_offset: i32,
            pub dragging: bool,
        }
        impl Scrollable {
            #[allow(clippy::too_many_arguments)]
            pub fn new(
                x: i32,
                y: i32,
                width: i32,
                height: i32,
                bar_x_shift: i32,
                bar_y_shift: i32,
                bar_width: i32,
                bar_height: i32,
                max_bar_height: i32,
            ) -> Self {
                Scrollable {
                    x,
                    y,
                    width,
                    height,
                    bar_x_shift,
                    bar_x: x + bar_x_shift,
                    bar_y: y + bar_y_shift,
                    bar_width,
                    bar_height,
                    max_bar_height,
                    scroll: 0.0,
                    drag_offset: 0,
                    dragging: false,
                }
            }
            fn max(&self) -> i32 {
                self.max_bar_height - self.bar_height
            }
            /// `getScroll`: the thumb's offset down the bar.
            pub fn thumb(&self) -> i32 {
                let max = self.max();
                clamp((self.scroll * f64::from(max)) as i32, 0, max)
            }
            /// `needsScrollBars`.
            pub fn needs_bar(max_elements: i32, focused: i32) -> bool {
                max_elements > focused
            }
            /// `getCurrentSelection`: the first element shown.
            pub fn first(&self, max_elements: i32, focused: i32) -> i32 {
                if Self::needs_bar(max_elements, focused) {
                    ((f64::from(max_elements - focused) + 0.5) * self.scroll) as i32
                } else {
                    0
                }
            }
            /// `adjustScroll`: a wheel step (sign of `delta`), `scaler` elements a step; true when it scrolled.
            pub fn wheel(&mut self, delta: f64, max_elements: i32, focused: i32, scaler: i32) -> bool {
                if delta != 0.0 && Self::needs_bar(max_elements, focused) {
                    let elements =
                        (f64::from(max_elements - focused) / f64::from(scaler.max(1))).ceil() as i32;
                    if elements > 0 {
                        let delta = if delta > 0.0 { 1.0 } else { -1.0 };
                        self.scroll = (self.scroll - delta / f64::from(elements)).clamp(0.0, 1.0);
                        return true;
                    }
                }
                false
            }
            /// `onClick`: a press on the thumb starts dragging it (or, with nothing to scroll, puts the list at the top).
            pub fn press(&mut self, mx: f64, my: f64, max_elements: i32, focused: i32) -> bool {
                let scroll = self.thumb();
                let (x, y) = (f64::from(self.bar_x), f64::from(self.bar_y));
                if mx >= x
                    && mx <= x + f64::from(self.bar_width)
                    && my >= y + f64::from(scroll)
                    && my <= y + f64::from(scroll + self.bar_height)
                {
                    if Self::needs_bar(max_elements, focused) {
                        self.drag_offset = (my - f64::from(scroll + self.bar_y)) as i32;
                        self.dragging = true;
                    } else {
                        self.scroll = 0.0;
                    }
                    return true;
                }
                false
            }
            /// `onDrag`.
            pub fn drag(&mut self, my: f64, max_elements: i32, focused: i32) {
                if self.dragging && Self::needs_bar(max_elements, focused) {
                    let max = f64::from(self.max().max(1));
                    self.scroll = ((my - f64::from(self.bar_y) - f64::from(self.drag_offset)) / max)
                        .clamp(0.0, 1.0);
                }
            }
            /// `onRelease`.
            pub fn release(&mut self) {
                self.drag_offset = 0;
                self.dragging = false;
            }
            pub fn is_over(&self, mx: f64, my: f64) -> bool {
                mx >= f64::from(self.x)
                    && mx < f64::from(self.x + self.width)
                    && my >= f64::from(self.y)
                    && my < f64::from(self.y + self.height)
            }
            /// `drawScrollBar`: the borders and the thumb from a `tex_w` by `tex_h` texture (`scroll_list.png`).
            pub fn draw_bar(&self, c: &mut Canvas, texture: &str, tex_w: i32, tex_h: i32) {
                let (tw, th) = (tex_w as f32, tex_h as f32);
                let (bx, by) = (self.bar_x as f32, self.bar_y as f32);
                c.blit(texture, bx - 1.0, by - 1.0, 0.0, 0.0, tw, 1.0, tw, th);
                c.blit_region(
                    texture,
                    [bx - 1.0, by, tw, self.max_bar_height as f32],
                    [0.0, 1.0, tw, 1.0],
                    [tw, th],
                    u32::MAX,
                );
                c.blit(
                    texture,
                    bx - 1.0,
                    (self.y + self.max_bar_height + 2) as f32,
                    0.0,
                    0.0,
                    tw,
                    1.0,
                    tw,
                    th,
                );
                c.blit(
                    texture,
                    bx,
                    by + self.thumb() as f32,
                    0.0,
                    2.0,
                    self.bar_width as f32,
                    self.bar_height as f32,
                    tw,
                    th,
                );
            }
        }

        /// `GuiScrollList`: rows `element_height` high over a background, with the scroll bar at the right (6 pixels).
        #[derive(Clone, Debug, PartialEq)]
        pub struct ScrollList {
            pub scroll: Scrollable,
            pub element_height: i32,
            /// `scroll_list.png` (6 by 6).
            pub texture: &'static str,
            /// The background (`GuiInnerScreen.SCREEN`, 32-pixel sides, 256 by 256).
            pub background: &'static str,
            pub background_side: i32,
            /// The selected row, if any.
            pub selected: Option<usize>,
        }
        impl ScrollList {
            pub const TEXTURE_SIZE: i32 = 6;
            pub fn new(x: i32, y: i32, width: i32, height: i32, element_height: i32) -> Self {
                ScrollList {
                    scroll: Scrollable::new(x, y, width, height, width - 6, 2, 4, 4, height - 4),
                    element_height,
                    texture: "mekanism:gui/scroll_list",
                    background: "mekanism:gui/inner_screen",
                    background_side: 32,
                    selected: None,
                }
            }
            /// `getFocusedElements`: rows that fit.
            pub fn focused(&self) -> i32 {
                (self.scroll.height - 2) / self.element_height
            }
            /// The first row shown.
            pub fn first(&self, rows: usize) -> usize {
                self.scroll.first(rows as i32, self.focused()) as usize
            }
            /// `drawBackground`: the background and the bar; then the rows are the screen's to draw (with
            /// [`ScrollList::draw_selection`] for a text list).
            pub fn draw_background(&self, c: &mut Canvas) {
                let s = &self.scroll;
                let side = self.background_side as f32;
                background(
                    c,
                    self.background,
                    side,
                    [s.x as f32, s.y as f32, s.width as f32, s.height as f32],
                    [256.0, 256.0],
                );
                s.draw_bar(c, self.texture, Self::TEXTURE_SIZE, Self::TEXTURE_SIZE);
            }
            /// `GuiTextScrollList.renderElements`: the selected row's highlight.
            pub fn draw_selection(&self, c: &mut Canvas, rows: usize) {
                let first = self.first(rows);
                if let Some(sel) = self.selected
                    && sel >= first
                    && (sel as i32) < first as i32 + self.focused()
                {
                    let s = &self.scroll;
                    let ts = Self::TEXTURE_SIZE as f32;
                    c.blit_region(
                        self.texture,
                        [
                            (s.x + 1) as f32,
                            (s.y + 1 + (sel - first) as i32 * self.element_height) as f32,
                            (s.bar_x_shift - 2) as f32,
                            self.element_height as f32,
                        ],
                        [4.0, 2.0, 2.0, 2.0],
                        [ts, ts],
                        u32::MAX,
                    );
                }
            }
            /// Where row `i` of those shown is: its top left.
            pub fn row_at(&self, i: usize) -> (f32, f32) {
                (
                    (self.scroll.x + 1) as f32,
                    (self.scroll.y + 1 + i as i32 * self.element_height) as f32,
                )
            }
            /// `GuiTextScrollList.renderForeground`'s rows: `(index, x, y)` of each row shown, for the screen to draw.
            pub fn shown(&self, rows: usize) -> Vec<(usize, f32, f32)> {
                let first = self.first(rows);
                (0..self.focused() as usize)
                    .map(|i| first + i)
                    .take_while(|&i| i < rows)
                    .enumerate()
                    .map(|(n, i)| {
                        let (x, y) = self.row_at(n);
                        (i, x, y)
                    })
                    .collect()
            }
            /// `onClick`: the thumb, or a row (`Some(Some(row))`), or the empty part of the list, which clears the selection
            /// (`Some(None)`); None when the press is elsewhere.
            pub fn press(&mut self, mx: f64, my: f64, rows: usize) -> Option<Option<usize>> {
                let focused = self.focused();
                if self.scroll.press(mx, my, rows as i32, focused) {
                    return Some(self.selected);
                }
                let s = &self.scroll;
                if mx >= f64::from(s.x + 1)
                    && mx < f64::from(s.x + s.bar_x_shift - 1)
                    && my >= f64::from(s.y + 1)
                    && my < f64::from(s.y + s.height - 1)
                {
                    let first = self.first(rows);
                    for i in 0..focused as usize {
                        if first + i >= rows {
                            break;
                        }
                        let top = f64::from(s.y + 1 + self.element_height * i as i32);
                        if my >= top && my <= top + f64::from(self.element_height) {
                            self.selected = Some(first + i);
                            return Some(self.selected);
                        }
                    }
                    self.selected = None;
                    return Some(None);
                }
                None
            }
            pub fn drag(&mut self, my: f64, rows: usize) {
                let focused = self.focused();
                self.scroll.drag(my, rows as i32, focused);
            }
            pub fn release(&mut self) {
                self.scroll.release();
            }
            /// `mouseScrolled`: over the list, a wheel step.
            pub fn wheel(&mut self, mx: f64, my: f64, delta: f64, rows: usize) -> bool {
                let focused = self.focused();
                self.scroll.is_over(mx, my) && self.scroll.wheel(delta, rows as i32, focused, 1)
            }
            /// `GuiTextScrollList.setText`: the selection past the new rows goes, and the list goes to the top when it no
            /// longer needs a bar.
            pub fn set_rows(&mut self, rows: usize) {
                if self.selected.is_some_and(|s| s >= rows) {
                    self.selected = None;
                }
                if !Scrollable::needs_bar(rows as i32, self.focused()) {
                    self.scroll.scroll = 0.0;
                }
            }
        }

        /// `GuiScrollBar`: a stand-alone bar, 14 wide, for a list the screen lays out itself (the QIO grid).
        #[derive(Clone, Debug, PartialEq)]
        pub struct ScrollBar {
            pub scroll: Scrollable,
            /// `scroll_bar.png` (24 by 15: the thumb, then the greyed thumb).
            pub texture: &'static str,
            /// `GuiElementHolder.HOLDER` (32-pixel sides, 256 by 256).
            pub holder: &'static str,
        }
        impl ScrollBar {
            pub const TEXTURE_WIDTH: i32 = 24;
            pub const TEXTURE_HEIGHT: i32 = 15;
            pub fn new(x: i32, y: i32, height: i32) -> Self {
                ScrollBar {
                    scroll: Scrollable::new(
                        x,
                        y,
                        14,
                        height,
                        1,
                        1,
                        Self::TEXTURE_WIDTH / 2,
                        Self::TEXTURE_HEIGHT,
                        height - 2,
                    ),
                    texture: "mekanism:gui/scroll_bar",
                    holder: "mekanism:gui/element_holder",
                }
            }
            /// `drawBackground`: the holder and the thumb, greyed when there is nothing to scroll.
            pub fn draw(&self, c: &mut Canvas, max_elements: i32, focused: i32) {
                let s = &self.scroll;
                background(
                    c,
                    self.holder,
                    32.0,
                    [
                        s.x as f32,
                        s.y as f32,
                        (s.bar_width + 2) as f32,
                        s.height as f32,
                    ],
                    [256.0, 256.0],
                );
                let u = if Scrollable::needs_bar(max_elements, focused) {
                    0.0
                } else {
                    s.bar_width as f32
                };
                c.blit(
                    self.texture,
                    s.bar_x as f32,
                    (s.bar_y + s.thumb()) as f32,
                    u,
                    0.0,
                    s.bar_width as f32,
                    s.bar_height as f32,
                    Self::TEXTURE_WIDTH as f32,
                    Self::TEXTURE_HEIGHT as f32,
                );
            }
            pub fn first(&self, max_elements: i32, focused: i32) -> i32 {
                self.scroll.first(max_elements, focused)
            }
            pub fn press(&mut self, mx: f64, my: f64, max_elements: i32, focused: i32) -> bool {
                self.scroll.press(mx, my, max_elements, focused)
            }
            pub fn drag(&mut self, my: f64, max_elements: i32, focused: i32) {
                self.scroll.drag(my, max_elements, focused);
            }
            pub fn release(&mut self) {
                self.scroll.release();
            }
            /// A wheel step anywhere the screen routes it (the QIO grid scrolls by rows: `scaler` 1).
            pub fn wheel(&mut self, delta: f64, max_elements: i32, focused: i32) -> bool {
                self.scroll.wheel(delta, max_elements, focused, 1)
            }
        }

        /// What a press did to a [`Window`].
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum WindowPress {
            /// Its close button: the window closed.
            Closed,
            /// Its title band (18 pixels): dragging started.
            Dragging,
            /// Somewhere else on it: the press is the window's (its own widgets may take it).
            Inside,
            /// Outside it, over the player's inventory: the game's slots take it (`InteractionStrategy.CONTAINER`).
            Inventory,
            /// Outside it and not over the inventory: taken by the window, nothing behind gets it.
            Blocked,
        }

        /// `GuiWindow`: a panel over the screen with Mekanism's shadow, a close button at 6, 6 and a title band that drags
        /// it. While it is open, the screen draws it after [`Canvas::slots`] and hides the slots under it
        /// ([`Window::covers`]).
        #[derive(Clone, Debug, PartialEq)]
        pub struct Window {
            pub x: i32,
            pub y: i32,
            pub width: i32,
            pub height: i32,
            pub title: String,
            /// `GuiMekanism.BASE_BACKGROUND` (4-pixel sides, 256 by 256).
            pub background: &'static str,
            /// `GuiMekanism.SHADOW`.
            pub shadow: &'static str,
            /// `GuiCloseButton`'s `button/close.png`, drawn 8 by 12.
            pub close_texture: &'static str,
            drag_from: Option<(f64, f64)>,
            prev: (i32, i32),
        }
        impl Window {
            pub fn new(x: i32, y: i32, width: i32, height: i32, title: &str) -> Self {
                Window {
                    x,
                    y,
                    width,
                    height,
                    title: title.to_owned(),
                    background: "mekanism:gui/base",
                    shadow: "mekanism:gui/shadow",
                    close_texture: "mekanism:gui/button/close",
                    drag_from: None,
                    prev: (0, 0),
                }
            }
            /// `calculateOpenPosition`: a window reopened where it was (`last`), kept on the screen (`bounds`: the GUI from
            /// the panel's top left, `[-left, -top, gui width, gui height]`).
            pub fn place(&mut self, last: Option<(i32, i32)>, bounds: Rect) {
                if let Some((x, y)) = last {
                    let (min_x, min_y) = (bounds[0] as i32, bounds[1] as i32);
                    let (max_x, max_y) = (
                        (bounds[0] + bounds[2]) as i32 - self.width,
                        (bounds[1] + bounds[3]) as i32 - self.height,
                    );
                    self.x = x.max(min_x).min(max_x.max(min_x));
                    self.y = y.max(min_y).min(max_y.max(min_y));
                }
            }
            pub fn rect(&self) -> Rect {
                [
                    self.x as f32,
                    self.y as f32,
                    self.width as f32,
                    self.height as f32,
                ]
            }
            pub fn is_over(&self, mx: f64, my: f64) -> bool {
                mx >= f64::from(self.x)
                    && mx < f64::from(self.x + self.width)
                    && my >= f64::from(self.y)
                    && my < f64::from(self.y + self.height)
            }
            /// Whether a slot at `(x, y)` (16 by 16) is under the window: hide it ([`Canvas::slot`] with `active` false).
            pub fn covers(&self, x: f32, y: f32) -> bool {
                x + 16.0 > self.x as f32
                    && x < (self.x + self.width) as f32
                    && y + 16.0 > self.y as f32
                    && y < (self.y + self.height) as f32
            }
            /// `renderBackgroundOverlay`: the shadow at three quarters, then the panel. Then the screen draws the title
            /// ([`Window::draw_title`]) and the window's own widgets.
            pub fn draw_background(&self, c: &mut Canvas) {
                let r = self.rect();
                c.blit_nine_sliced(
                    self.shadow,
                    [r[0] - 3.0, r[1] - 3.0, r[2] + 6.0, r[3] + 6.0],
                    [4.0; 4],
                    [0.0, 0.0, 256.0, 256.0],
                    [256.0, 256.0],
                    0xBFFF_FFFF,
                );
                background(c, self.background, 4.0, r, [256.0, 256.0]);
                c.blit(
                    self.close_texture,
                    r[0] + 6.0,
                    r[1] + 6.0,
                    0.0,
                    0.0,
                    8.0,
                    12.0,
                    8.0,
                    12.0,
                );
                c.area(
                    -1,
                    [r[0] + 6.0, r[1] + 6.0, 8.0, 12.0],
                    AreaKind::Button,
                    true,
                    "Close",
                );
            }
            /// `drawTitleTextTextWithOffset(text, 12, 6, width)`: the title centred between the close button and the edge.
            pub fn draw_title(&self, c: &mut Canvas, width_of: impl Fn(&str) -> f32, argb: u32) {
                let start = 12.0;
                let room = self.width as f32 - start;
                let w = width_of(&self.title);
                let x = self.x as f32 + start + ((room - w) / 2.0).max(0.0);
                c.draw_string(&self.title, x.floor(), (self.y + 6) as f32, argb, false);
            }
            /// `mouseClicked`: the close button, the title band, the window itself, or outside it.
            pub fn press(
                &mut self,
                mx: f64,
                my: f64,
                inventory_top: Option<f64>,
                panel_width: f64,
            ) -> WindowPress {
                let (x, y) = (f64::from(self.x), f64::from(self.y));
                if mx >= x + 6.0 && mx < x + 14.0 && my >= y + 6.0 && my < y + 18.0 {
                    return WindowPress::Closed;
                }
                if self.is_over(mx, my) {
                    if my < y + 18.0 {
                        self.drag_from = Some((mx, my));
                        self.prev = (0, 0);
                        return WindowPress::Dragging;
                    }
                    return WindowPress::Inside;
                }
                match inventory_top {
                    Some(top) if mx >= 0.0 && mx < panel_width && my >= top => WindowPress::Inventory,
                    _ => WindowPress::Blocked,
                }
            }
            /// `onDrag`: follows the cursor, kept within `bounds` (as [`Window::place`]).
            pub fn drag(&mut self, mx: f64, my: f64, bounds: Rect) {
                let Some((fx, fy)) = self.drag_from else {
                    return;
                };
                let (ndx, ndy) = ((mx - fx).round() as i32, (my - fy).round() as i32);
                let (min_x, min_y) = (bounds[0] as i32, bounds[1] as i32);
                let (max_x, max_y) = (
                    (bounds[0] + bounds[2]) as i32,
                    (bounds[1] + bounds[3]) as i32,
                );
                let cx = clamp(
                    ndx - self.prev.0,
                    min_x - self.x,
                    max_x - (self.x + self.width),
                );
                let cy = clamp(
                    ndy - self.prev.1,
                    min_y - self.y,
                    max_y - (self.y + self.height),
                );
                self.prev = (ndx, ndy);
                self.x += cx;
                self.y += cy;
            }
            pub fn release(&mut self) {
                self.drag_from = None;
            }
            pub fn dragging(&self) -> bool {
                self.drag_from.is_some()
            }
        }

        /// A tab at the panel's side that opens a [`Window`] (`GuiInsetElement` tabs: the QIO frequency tab at -26, 20), and
        /// is inactive while its window is open (`setListenerTab`).
        #[derive(Clone, Debug, PartialEq)]
        pub struct SideTab {
            pub id: i32,
            pub x: i32,
            pub y: i32,
            pub width: i32,
            pub height: i32,
            /// The tab's background (`holder_left.png` / `holder_right.png`, 26 by 18 in Mekanism's insets).
            pub holder: &'static str,
            /// The icon drawn in the middle.
            pub icon: &'static str,
            pub icon_size: [f32; 2],
            pub label: String,
        }
        impl SideTab {
            pub fn new(id: i32, x: i32, y: i32, icon: &'static str, label: &str) -> Self {
                SideTab {
                    id,
                    x,
                    y,
                    width: 26,
                    height: 18,
                    holder: "mekanism:gui/holder_left",
                    icon,
                    icon_size: [18.0, 18.0],
                    label: label.to_owned(),
                }
            }
            pub fn rect(&self) -> Rect {
                [
                    self.x as f32,
                    self.y as f32,
                    self.width as f32,
                    self.height as f32,
                ]
            }
            /// The holder, the icon centred, and its area (inactive while `open`).
            pub fn draw(&self, c: &mut Canvas, open: bool) {
                let r = self.rect();
                c.blit(self.holder, r[0], r[1], 0.0, 0.0, r[2], r[3], r[2], r[3]);
                let [iw, ih] = self.icon_size;
                let tint = if open { 0xFF80_8080 } else { u32::MAX };
                c.blit_region(
                    self.icon,
                    [
                        r[0] + ((r[2] - iw) / 2.0).floor(),
                        r[1] + ((r[3] - ih) / 2.0).floor(),
                        iw,
                        ih,
                    ],
                    [0.0, 0.0, iw, ih],
                    [iw, ih],
                    tint,
                );
                c.area(self.id, r, AreaKind::Tab, !open, &self.label);
            }
            pub fn is_over(&self, mx: f64, my: f64) -> bool {
                mx >= f64::from(self.x)
                    && mx < f64::from(self.x + self.width)
                    && my >= f64::from(self.y)
                    && my < f64::from(self.y + self.height)
            }
        }

        #[cfg(test)]
        mod tests {
            use super::*;

            #[test]
            fn a_scroll_list_shows_and_picks_rows_as_gui_scroll_list() {
                // A 162 by 42 list of 10-pixel rows (the frequency list): 4 rows shown.
                let mut list = ScrollList::new(7, 29, 162, 42, 10);
                assert_eq!(list.focused(), 4);
                assert_eq!(list.first(3), 0);
                assert_eq!(list.shown(3).len(), 3);
                // Row 2 of 3: y from 29 + 1 + 20 to 29 + 1 + 30.
                assert_eq!(list.press(20.0, 52.0, 3), Some(Some(2)));
                // Below the last row, inside the list: the selection clears.
                assert_eq!(list.press(20.0, 65.0, 3), Some(None));
                assert_eq!(list.press(300.0, 65.0, 3), None);
                // Ten rows: the wheel moves one of the six hidden at a time.
                assert!(list.wheel(10.0, 40.0, -1.0, 10));
                assert!((list.scroll.scroll - 1.0 / 6.0).abs() < 1e-9);
                assert_eq!(list.first(10), 1);
                // Fewer rows than fit: back to the top.
                list.set_rows(2);
                assert_eq!(list.scroll.scroll, 0.0);
            }

            #[test]
            fn the_thumb_drags_with_the_cursor_and_stops_at_the_ends() {
                let mut bar = ScrollBar::new(156, 18, 74);
                // 100 rows of 8 shown: the thumb at the top (bar_y 19, 15 high) takes the press.
                assert!(bar.press(160.0, 25.0, 100, 8));
                bar.drag(200.0, 100, 8);
                assert_eq!(bar.scroll.scroll, 1.0);
                assert_eq!(bar.first(100, 8), 92);
                bar.drag(0.0, 100, 8);
                assert_eq!(bar.scroll.scroll, 0.0);
                bar.release();
                assert!(!bar.scroll.dragging);
            }

            #[test]
            fn a_window_drags_by_its_title_and_stays_on_screen() {
                let mut w = Window::new(10, 20, 100, 60, "Set Frequency");
                let bounds = [-50.0, -30.0, 400.0, 240.0];
                assert_eq!(
                    w.press(50.0, 25.0, Some(84.0), 176.0),
                    WindowPress::Dragging
                );
                w.drag(80.0, 35.0, bounds);
                assert_eq!((w.x, w.y), (40, 30));
                // Far to the left: it stops at the GUI's edge.
                w.drag(-500.0, 35.0, bounds);
                assert_eq!(w.x, -50);
                w.release();
                assert_eq!(
                    w.press(f64::from(w.x) + 8.0, f64::from(w.y) + 8.0, None, 176.0),
                    WindowPress::Closed
                );
                assert_eq!(
                    w.press(150.0, 100.0, Some(84.0), 176.0),
                    WindowPress::Inventory
                );
                assert_eq!(
                    w.press(150.0, 50.0, Some(84.0), 176.0),
                    WindowPress::Blocked
                );
                assert!(w.covers(w.x as f32, w.y as f32));
            }
        }
    }
}
#[macro_export] macro_rules! export_screen_mod { ($t:ty) => {}; }

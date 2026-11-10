#![allow(dead_code, unused, non_snake_case, clippy::all, ambiguous_glob_reexports)]
pub mod server {
  pub use serde_json;
  pub mod players { use super::*;
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
    pub fn set_motion<A0, A1, A2, A3>(_a0: A0, _a1: A1, _a2: A2, _a3: A3) -> Result<(), String> { Err(String::new()) }
    pub fn set_allow_flight<A0, A1>(_a0: A0, _a1: A1) -> Result<(), String> { Err(String::new()) }
    pub fn play_sound<A0, A1, A2, A3, A4, A5, A6>(_a0: A0, _a1: A1, _a2: A2, _a3: A3, _a4: A4, _a5: A5, _a6: A6) -> () { Default::default() }
  }
  pub mod recipe_types { use super::*;
    pub fn burn_time<A0>(_a0: A0) -> Option<u32> { None }
    #[derive(Clone, Debug, Default, PartialEq)] pub struct CustomRecipe { pub id: String, pub json: String }
    #[derive(Clone, Debug, Default, PartialEq)] pub struct CookingRecipe { pub inputs: Vec<String>, pub output: String, pub count: u32, pub experience: f32, pub time: u32 }
    pub fn recipes_of_type<A0>(_a0: A0) -> Vec<CustomRecipe> { Default::default() }
    pub fn cooking_recipes<A0>(_a0: A0) -> Vec<CookingRecipe> { Default::default() }
    pub fn ingredient_items<A0>(_a0: A0) -> Vec<String> { Default::default() }
  }
  pub mod entities { use super::*;
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
  }
  pub mod player_items { use super::*;
    pub fn get_inventory<A0>(_a0: A0) -> Option<Vec<Option<ItemStack>>> { None }
    pub fn set_slot<A0, A1, A2>(_a0: A0, _a1: A1, _a2: A2) -> Result<(), String> { Err(String::new()) }
    pub fn compare_and_set_slot<A0, A1, A2, A3>(_a0: A0, _a1: A1, _a2: A2, _a3: A3) -> Result<(), String> { Err(String::new()) }
    pub fn hurt<A0, A1, A2>(_a0: A0, _a1: A1, _a2: A2) -> Result<(), String> { Err(String::new()) }
    pub fn add_effect<A0, A1, A2, A3>(_a0: A0, _a1: A1, _a2: A2, _a3: A3) -> Result<(), String> { Err(String::new()) }
  }
  pub mod chunks { use super::*;
    pub fn set_forced<A0, A1, A2, A3>(_a0: A0, _a1: A1, _a2: A2, _a3: A3) -> Result<(), String> { Err(String::new()) }
  }
  pub mod world { use super::*;
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
    pub fn redirect(self, _block: &str, _flag_at: u32, _pos_at: u32) -> Self { self }
    pub fn new() -> Self { Self::default() }
  }
  pub trait ServerMod: Sized {
    fn init() -> Result<(Self, Registration), String>;
    fn chunk_loaded(&mut self, dimension: &str, chunk_x: i32, chunk_z: i32, new: bool) {}
    fn tick(&mut self, tick: u64) {}
    fn block_broken(&mut self, player: &Player, pos: BlockPos, state: &str) {}
    fn block_placed(&mut self, player: &Player, pos: BlockPos, state: &str) {}
    fn entity_spawned(&mut self, ticket: u64, id: Option<u32>) {}
    fn item_used(&mut self, player: &Player, stack: ItemStack, pos: Option<BlockPos>) -> bool { false }
    fn block_used(&mut self, player: &Player, pos: BlockPos, sneaking: bool, stack: Option<ItemStack>) -> BlockUse { BlockUse::Default }
    fn can_place(&mut self, player: &Player, pos: BlockPos, state: &str) -> bool { true }
    fn player_damaged(&mut self, player: &Player, source: DamageSource, amount: f32) -> f32 { amount }
    fn item_hit_damage(&mut self, player: &Player, stack: ItemStack, kind: &str, damage: f32) -> f32 { damage }
    fn menu_button(&mut self, _player: &Player, _pos: BlockPos, id: i32, e: BlockEntity) -> Option<BlockEntityUpdate> { None }
    fn multiblock_changed(&mut self, group: &str, dimension: &str, structures: Vec<Structure>) -> Vec<BlockEntityUpdate> { Vec::new() }
    fn block_entities_tick(&mut self, tick: u64, mut entities: Vec<BlockEntity>) -> Vec<BlockEntityUpdate> { Vec::new() }
  }
}
#[macro_export] macro_rules! export_server_mod { ($t:ty) => {}; }

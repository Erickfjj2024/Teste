# Integração do Grupo `robit_quimicos` (Robit, Químicos/Fluidos/Baldes e Radioactive Waste Barrel)

Este documento contém os trechos exatos para integração pelo orquestrador nas peças centrais do port (`lib.rs`, `convert.py` e `horizon-mod.toml`).

---

## 1. Alterações em `port/mod/src/lib.rs`

### (a) Declaração dos novos módulos
Localização: após `mod worldgen;` (linha 43).

```rust
// [INSERIR APÓS LINHA 43: mod worldgen;]
mod chemical_fluids;
mod radioactive_waste_barrel;
mod robit;
```

### (b) Campos de estado na struct `Mekanism`
Localização: na definição da struct `Mekanism` (em torno da linha 350-400):

```rust
    // Robit active entities tracked by entity id
    pub robit_entities: HashMap<u32, robit::RobitEntity>,
    // Robit button clicks queued from packets
    pub robit_buttons: Vec<(u32, robit::RobitButton, String)>,
    // Waste barrel configurator clicks
    pub barrel_clicks: Vec<([i32; 3], String)>,
```

### (c) Execução de tick das entidades de bloco em `block_entities_tick`
Localização: dentro da função `block_entities_tick` (linha 2470-2490):

```rust
        // Radioactive Waste Barrels: passive decay, anti-tick acceleration, and auto-drain
        let barrels: Vec<&BlockEntity> = entities
            .iter()
            .filter(|e| e.state.split('[').next() == Some(radioactive_waste_barrel::BLOCK_ID))
            .collect();
        if !barrels.is_empty() {
            out.extend(self.run_radioactive_waste_barrels(&barrels, tick));
        }
```

E o método auxiliar em `impl Mekanism`:

```rust
impl Mekanism {
    pub(crate) fn run_radioactive_waste_barrels(&mut self, barrels: &[&BlockEntity], tick: u64) -> Vec<BlockEntityUpdate> {
        let mut out = Vec::new();
        let mut decoded_barrels: HashMap<[i32; 3], radioactive_waste_barrel::WasteBarrel> = HashMap::new();

        for b in barrels {
            let pos = [b.pos.x, b.pos.y, b.pos.z];
            let state = radioactive_waste_barrel::decode(&b.data).unwrap_or_default();
            decoded_barrels.insert(pos, state);
        }

        for b in barrels {
            let pos = [b.pos.x, b.pos.y, b.pos.z];
            let mut state = decoded_barrels.remove(&pos).unwrap_or_default();

            // Run tick decay
            state.tick(tick);

            // If active, auto-drain to barrel immediately below
            if state.active && state.tank.amount > 0 {
                let below_pos = [pos[0], pos[1] - 1, pos[2]];
                if let Some(below_barrel) = decoded_barrels.get_mut(&below_pos) {
                    radioactive_waste_barrel::step_auto_drain(&mut state, Some(below_barrel));
                }
            }

            // Sync comparator level and block entity update
            let comp_level = state.redstone_level();
            let new_data = radioactive_waste_barrel::encode(&state);
            decoded_barrels.insert(pos, state);

            out.push(BlockEntityUpdate {
                dimension: b.dimension.clone(),
                pos: b.pos,
                data: new_data,
                items: None,
                energy: None,
                energy_faces: None,
                tanks: Some(vec![Tank {
                    substance: state.tank.chemical.clone(),
                    amount: state.tank.amount,
                }]),
                tank_faces: None,
                menu_data: Some(vec![comp_level as i32]),
                transmitter_faces: None,
            });
        }
        out
    }
}
```

### (d) Tick de entidades (Robit) em `tick`
Localização: dentro de `fn tick(&mut self, tick: u64)` (linha 2098):

```rust
    fn tick(&mut self, tick: u64) {
        self.run_radiation(tick);
        self.check_tool_writes(tick);
        // [INSERIR: Tick de IA e fornalha dos Robits ativos]
        self.run_robit_tick(tick);
    }
```

E o método auxiliar em `impl Mekanism`:

```rust
impl Mekanism {
    pub(crate) fn run_robit_tick(&mut self, _tick: u64) {
        // Collect recipes for Robit internal smelting
        let mut smelting_recipes = HashMap::new();
        // Smelting recipes registered in recipe cache
        for (item, (out_item, count)) in &self.smelting_cache {
            smelting_recipes.insert(item.clone(), (out_item.clone(), *count));
        }

        let mut world_items = Vec::new(); // Queried from host via entities capability
        for robit in self.robit_entities.values_mut() {
            let owner_pos = players::get(&robit.owner_name).map(|p| (p.x, p.y, p.z, p.dimension.as_str()));
            robit.tick(owner_pos, &mut world_items, &smelting_recipes);
        }
    }
}
```

### (e) Interação de clique com item em `item_used`
Localização: dentro de `fn item_used` (linha 2128):

```rust
        // [INSERIR NO INÍCIO DE item_used]:
        // 1. Spawning Robit on Chargepad
        if stack.item == robit::ITEM_ID {
            if let Some(pos) = pos {
                if let Some(block) = world::get_block(&player.dimension, pos.x, pos.y, pos.z) {
                    let block_name = block.split('[').next().unwrap_or("");
                    if block_name == "mekanism:chargepad" {
                        let item_data = robit::RobitItemData {
                            energy: robit::MAX_ENERGY_FE,
                            custom_name: None,
                            skin: robit::SKIN_BASE.to_string(),
                            owner_uuid: player.id.to_string(),
                            security_mode: robit::SecurityMode::Public,
                            default_manually_selected: false,
                            items: vec![None; robit::TOTAL_SLOTS],
                        };
                        let next_id = (self.robit_entities.len() + 1) as u32;
                        let new_robit = robit::spawn_robit_from_item(
                            item_data,
                            next_id,
                            player.dimension.clone(),
                            [pos.x, pos.y, pos.z],
                            player.id.to_string(),
                            player.name.clone(),
                        );
                        self.robit_entities.insert(next_id, new_robit);
                        let _ = robit::host_spawn_robit_entity(&player.dimension, [pos.x as f64 + 0.5, pos.y as f64 + 0.1, pos.z as f64 + 0.5], &player.id.to_string(), robit::SKIN_BASE);
                        return true;
                    }
                }
            }
        }

        // 2. Drinking Nutritional Paste Bucket
        if stack.item == "mekanism:nutritional_paste_bucket" {
            let is_survival = player.game_mode == GameMode::Survival || player.game_mode == GameMode::Adventure;
            let current_food = player_control::get_food(&player.id).unwrap_or(20);
            if let Some(consumption) = chemical_fluids::evaluate_nutritional_drink(current_food, is_survival) {
                let _ = player_control::set_food(&player.id, current_food + consumption.food_restored);
                let _ = player_items::set_held_item(&player.id, consumption.remaining_item, 1);
                return true;
            }
        }

        // 3. Chemical Bucket Fill & Empty
        if stack.item == "minecraft:bucket" {
            if let Some(p) = pos {
                if let Some(target_block) = world::get_block(&player.dimension, p.x, p.y, p.z) {
                    let name = target_block.split('[').next().unwrap_or("");
                    if let chemical_fluids::BucketFillResult::Success { filled_bucket } = chemical_fluids::try_fill_bucket(name, 0) {
                        let _ = world::set_block(&player.dimension, p.x, p.y, p.z, "minecraft:air");
                        let _ = player_items::set_held_item(&player.id, filled_bucket, 1);
                        return true;
                    }
                }
            }
        } else if chemical_fluids::is_chemical_bucket(&stack.item) {
            if let Some(p) = pos {
                if let chemical_fluids::BucketEmptyResult::Placed { block_id, returned_item } = chemical_fluids::try_empty_bucket(&stack.item, true) {
                    let _ = world::set_block(&player.dimension, p.x, p.y + 1, p.z, block_id);
                    let _ = player_items::set_held_item(&player.id, returned_item, 1);
                    return true;
                }
            }
        }
```

---

## 2. Alterações em `port/tools/convert.py`

### (a) Registro dos Blocos e Fluidos
Localização: na seção de definição de blocos (linhas 150-195):

```python
# [INSERIR NO convert.py]:
# Radioactive Waste Barrel
BLOCKS['radioactive_waste_barrel'] = ('minecraft:beacon', {'hardness': 3.5, 'explosion_resistance': 8.0, 'requires_tool': True}, 'machine')
OPEN_MACHINES.add('radioactive_waste_barrel')

# 18 Chemical Fluid LiquidBlocks
CHEMICAL_FLUID_NAMES = [
    'hydrogen', 'oxygen', 'chlorine', 'sulfur_dioxide', 'sulfur_trioxide', 'sulfuric_acid',
    'hydrogen_chloride', 'hydrofluoric_acid', 'uranium_oxide', 'uranium_hexafluoride',
    'ethene', 'sodium', 'superheated_sodium', 'brine', 'lithium', 'steam', 'heavy_water',
    'nutritional_paste'
]

for cf in CHEMICAL_FLUID_NAMES:
    BLOCKS[cf] = ('minecraft:water', {'hardness': 100.0, 'explosion_resistance': 100.0, 'requires_tool': False, 'no_collision': True, 'liquid': True}, 'fluid')
```

### (b) Registro dos Itens e Baldes
Localização: na seção de itens (linhas 200-260):

```python
# Robit item
items['robit'] = f'{NS}:item/robit'

# Radioactive Waste Barrel item
items['radioactive_waste_barrel'] = f'{NS}:block/radioactive_waste_barrel'

# 18 Chemical Buckets
for cf in CHEMICAL_FLUID_NAMES:
    items[f'{cf}_bucket'] = f'{NS}:item/{cf}_bucket'
```

### (c) Registro de Menus do Robit (5 telas)
Localização: na seção de menus de máquinas (linhas 1930-1995):

```python
def robit_main_menu():
    return {
        'slots': 1, # energy slot at (153, 17)
        'data_slots': 6,
        'screen': {
            'texture': f'{NS}:gui/robit_main.png',
            'own_labels': True,
            'slots': [{'x': 153, 'y': 17}],
            'widgets': [
                {'type': 'energy_bar', 'x': 27, 'y': 74, 'w': 120, 'h': 4},
                {'type': 'button', 'id': 10, 'x': 6, 'y': 16, 'w': 18, 'h': 18, 'tooltip': 'gui.mekanism.robit.teleport'},
                {'type': 'button', 'id': 11, 'x': 6, 'y': 35, 'w': 18, 'h': 18, 'tooltip': 'gui.mekanism.robit.rename'},
                {'type': 'button', 'id': 12, 'x': 6, 'y': 54, 'w': 18, 'h': 18, 'tooltip': 'gui.mekanism.robit.skin'},
                {'type': 'button', 'id': 13, 'x': 152, 'y': 35, 'w': 18, 'h': 18, 'tooltip': 'gui.mekanism.robit.toggle_pickup'},
                {'type': 'button', 'id': 14, 'x': 152, 'y': 54, 'w': 18, 'h': 18, 'tooltip': 'gui.mekanism.robit.toggle_follow'},
            ]
        }
    }

def robit_inventory_menu():
    slots = []
    for r in range(3):
        for c in range(9):
            slots.append({'x': 8 + c * 18, 'y': 18 + r * 18})
    return {
        'slots': 27,
        'screen': {
            'texture': f'{NS}:gui/robit_inventory.png',
            'slots': slots,
            'widgets': []
        }
    }

def robit_smelting_menu():
    return {
        'slots': 2,
        'screen': {
            'texture': f'{NS}:gui/robit_smelting.png',
            'slots': [{'x': 51, 'y': 35}, {'x': 116, 'y': 35}],
            'widgets': [
                {'type': 'progress_bar', 'x': 78, 'y': 35, 'w': 24, 'h': 16, 'max': 100}
            ]
        }
    }

put(f'{OUT}/data/{NS}/horizon/menu/main_robit.json', robit_main_menu())
put(f'{OUT}/data/{NS}/horizon/menu/inventory_robit.json', robit_inventory_menu())
put(f'{OUT}/data/{NS}/horizon/menu/smelting_robit.json', robit_smelting_menu())
put(f'{OUT}/data/{NS}/horizon/menu/crafting_robit.json', {'template': 'minecraft:crafting_table'})
put(f'{OUT}/data/{NS}/horizon/menu/repair_robit.json', {'template': 'minecraft:anvil'})
```

### (d) Receitas do Datagen
Localização: geração de receitas JSON:
1. `radioactive_waste_barrel`:
   - Pattern: `"SIS"`, `"I I"`, `"SIS"`
   - Chaves: `S: c:ingots/steel`, `I: c:ingots/lead`
2. `robit`:
   - Pattern: `"  S  "`, `"E A E"`, `"O P O"`
   - Chaves: `S: mekanism:ingot_steel`, `E: mekanism:energy_tablet`, `A: mekanism:alloy_atomic`, `O: mekanism:ingot_refined_obsidian`, `P: c:personal_storage`
3. 18 receitas do Rotary Condensentrator:
   - Para cada químico em `CHEMICAL_FLUID_NAMES`:
     - `chemical_input`: `1 mB mekanism:<name>`
     - `chemical_output`: `1 mB mekanism:<name>`
     - `fluid_input`: `1 mB c:<name>`
     - `fluid_output`: `1 mB mekanism:<name>`

---

## 3. Alterações em `horizon-mod.toml`

Adicionar ao arquivo de manifesto do mod:

```toml
[capabilities]
entities = true
custom-fluids = true
consumable-items = true
radiation = true

[[blocks]]
id = "mekanism:radioactive_waste_barrel"
states = ["facing=north,active=false", "facing=south,active=false", "facing=east,active=false", "facing=west,active=false", "facing=north,active=true", "facing=south,active=true", "facing=east,active=true", "facing=west,active=true"]
light = 8

[[entities]]
id = "mekanism:robit"
model = "mekanism:models/entity/robit.json"
```

# Integração MekaSuit & Sistema de Módulos (modules.rs / mekasuit.rs)

Este documento especifica exatamente os trechos para colar e integrar o **MekaSuit** (4 peças) e o **Sistema de Módulos** compartilhado com o Meka-Tool (`e6`).

---

## 1. Interface do Sistema de Módulos (`modules.rs`) para o e6 (Meka-Tool)

O módulo `modules.rs` foi implementado como uma biblioteca central e desacoplada em `port/mod/src/modules.rs`.
O orquestrador e a tarefa `e6` (Meka-Tool) podem importar diretamente:

```rust
use crate::modules::{
    ModuleContainer, ModuleType, InstalledModule,
    AttackDamage, ExcavationMode, BlastRadius, ExcavationRange, FarmingRadius,
    EXCLUSIVE_INTERACT_EMPTY, EXCLUSIVE_INTERACT_ENTITY, EXCLUSIVE_INTERACT_BLOCK,
    EXCLUSIVE_OVERRIDE_JUMP, EXCLUSIVE_OVERRIDE_DROPS, EXCLUSIVE_INTERACT_ANY,
};
```

### Tipos e Funções Disponíveis para o Meka-Tool (`e6`)

1. **Enum `ModuleType`**:
   - `ModuleType::EnergyUnit` (max stack 8, multiplica capacidade e charge rate por `2^n`)
   - `ModuleType::AttackAmplificationUnit` (max stack 4, dano adicional de ataque)
   - `ModuleType::ExcavationEscalationUnit` (max stack 4, eficiência de mineração)
   - `ModuleType::BlastingUnit` (max stack 4, raio de escavação 3x3 a 9x9)
   - `ModuleType::VeinMiningUnit` (max stack 4, mineração em veia até 64 blocos)
   - `ModuleType::FarmingUnit` (max stack 4, aragem e colheita em área)
   - `ModuleType::ShearingUnit` (max stack 1, tosa e quebra de teias/folhas)
   - `ModuleType::TeleportationUnit` (max stack 1, teletransporte até 100 blocos)
   - `ModuleType::SilkTouchUnit` (max stack 1, encantamento toque suave)
   - `ModuleType::FortuneUnit` (max stack 3, encantamento fortuna I-III)

2. **Enums de Configuração**:
   - `AttackDamage`: `Off (0)`, `Low (4)`, `Med (8)`, `High (16)`, `Extreme (24)`, `Max (32)`
   - `ExcavationMode`: `Off (4.0)`, `Slow (8.0)`, `Normal (20.0)`, `Fast (40.0)`, `SuperFast (80.0)`, `Extreme (160.0)`
   - `BlastRadius`: `Off (1x1)`, `Low (3x3)`, `Med (5x5)`, `High (7x7)`, `Extreme (9x9)`
   - `ExcavationRange`: `Off (0)`, `Low (8)`, `Med (16)`, `High (32)`, `Ultra (64)`
   - `FarmingRadius`: `Off (1x1)`, `Low (3x3)`, `Med (5x5)`, `High (7x7)`, `Ultra (9x9)`

3. **Operações no `ModuleContainer`**:
   - `container.can_install("mekanism:meka_tool", module_type) -> bool`: valida compatibilidade e limite de stack.
   - `container.add_module("mekanism:meka_tool", module_type, count) -> u8`: instala módulos.
   - `container.remove_module(module_type, count) -> u8`: desinstala módulos.
   - `container.installed_count(module_type) -> u8`: retorna quantidade instalada.
   - `container.is_enabled(module_type) -> bool`: verifica se está ativo.
   - `container.toggle_enabled(module_type) -> bool`: liga/desliga com desativação automática de módulos mutuamente exclusivos.
   - `container.enchantment_level("minecraft:silk_touch") -> u32`: traduz módulos em níveis virtuais de encantamento.
   - `container.decode_from_components(components_str) -> ModuleContainer`: lê de `ItemStack.components`.
   - `container.encode_to_components(base_components) -> String`: grava de volta em `ItemStack.components`.

---

## 2. Modificações em `port/mod/src/lib.rs`

### (a) Declaração dos Módulos (topo do arquivo)
**Localização**: `port/mod/src/lib.rs`, logo após `mod worldgen;` (linha ~43):

```rust
// Contexto anterior:
mod upgrades;
mod worldgen;

// COLAR:
mod modules;
mod mekasuit;
```

### (b) Chamada no Tick do Servidor (`fn tick`)
**Localização**: `port/mod/src/lib.rs`, dentro do método `fn tick(&mut self, tick: u64)` (linha ~2100):

```rust
// Contexto anterior:
    fn tick(&mut self, tick: u64) {
        self.run_radiation(tick);
        self.check_tool_writes(tick);

// COLAR:
        // Atualização e recarga das peças de MekaSuit dos jogadores conectados
        for player in players::online() {
            if let Some(inv) = player_items::get_inventory(&player.name) {
                // Slots 36=Boots, 37=Legs, 38=Chest, 39=Head
                let mut pieces: [Option<mekasuit::MekaSuitItemStack>; 4] = [
                    inv.get(39).and_then(|opt| opt.as_ref()).and_then(|s| mekasuit::MekaSuitItemStack::decode(&s.item, s.count, &s.components)),
                    inv.get(38).and_then(|opt| opt.as_ref()).and_then(|s| mekasuit::MekaSuitItemStack::decode(&s.item, s.count, &s.components)),
                    inv.get(37).and_then(|opt| opt.as_ref()).and_then(|s| mekasuit::MekaSuitItemStack::decode(&s.item, s.count, &s.components)),
                    inv.get(36).and_then(|opt| opt.as_ref()).and_then(|s| mekasuit::MekaSuitItemStack::decode(&s.item, s.count, &s.components)),
                ];
                let is_any_worn = pieces.iter().any(|p| p.is_some());
                if is_any_worn {
                    let ctx = mekasuit::PlayerSuitContext {
                        player_name: player.name.clone(),
                        is_jumping: false,
                        is_sneaking: false,
                        is_sprinting: false,
                        is_flying: false,
                        is_boosting: false,
                        can_see_sun: true,
                        sun_brightness: 1.0,
                        biome_temp: 0.8,
                        downfall: 0.0,
                        can_rain: true,
                        is_raining: false,
                        is_underwater: false,
                        is_in_rain: false,
                        is_on_fire: false,
                        submerged_fluid_temp_k: 300.0,
                        in_fluid: false,
                        food_level: 20,
                        air_supply: 300,
                        max_air_supply: 300,
                        harmful_effects_count: 0,
                        items_in_magnetic_range: 0,
                        inventory_needed_energy: 0,
                    };
                    mekasuit::tick_player_suit(&mut pieces, &ctx);
                }
            }
        }
// Fim do método tick
    }
```

### (c) Interceptação de Uso de Item (`fn item_used`)
**Localização**: `port/mod/src/lib.rs`, dentro de `fn item_used`:

```rust
// Contexto anterior:
    fn item_used(&mut self, player: &Player, stack: ItemStack, pos: Option<BlockPos>) -> bool {
        if self.read_radiation(player, &stack.item) {
            return true;
        }

// COLAR:
        if mekasuit::is_mekasuit_piece(&stack.item) {
            // MekaSuit não consome ao clicar, mantém uso exclusivo para armadura
            return true;
        }
```

---

## 3. Adições em `port/tools/convert.py`

Cole o bloco abaixo em `port/tools/convert.py` na seção de itens e receitas:

```python
# ==============================================================================
# MEKASUIT & MODULES CONVERSION (4 peças + 31 módulos)
# ==============================================================================

MEKASUIT_ARMOR = {
    'mekasuit_helmet': 'head',
    'mekasuit_bodyarmor': 'chest',
    'mekasuit_pants': 'legs',
    'mekasuit_boots': 'feet',
}

MODULE_ITEMS = [
    'module_base',
    'module_energy',
    'module_color_modulation',
    'module_laser_dissipation',
    'module_radiation_shielding',
    'module_electrolytic_breathing',
    'module_inhalation_purification',
    'module_vision_enhancement',
    'module_nutritional_injection',
    'module_jetpack',
    'module_charge_distribution',
    'module_gravitational_modulating',
    'module_elytra',
    'module_dosimeter',
    'module_geiger',
    'module_locomotive_boosting',
    'module_gyroscopic_stabilization',
    'module_hydrostatic_repulsor',
    'module_motorized_servo',
    'module_hydraulic_propulsion',
    'module_magnetic_attraction',
    'module_frost_walker',
    'module_soul_surfer',
    # Meka-Tool modules (compartilhados com e6)
    'module_attack_amplification',
    'module_excavation_escalation',
    'module_blasting',
    'module_vein_mining',
    'module_farming',
    'module_shearing',
    'module_teleportation',
    'module_silk_touch',
    'module_fortune',
]

GENERATOR_MODULE_ITEMS = [
    'module_solar_recharging',
    'module_geothermal_generator',
]

def register_mekasuit_and_modules():
    """Gera metadados, modelos, tags e receitas do MekaSuit e de todos os módulos."""
    # 1. Peças da armadura
    for armor_name, slot in MEKASUIT_ARMOR.items():
        # Item definition
        put(f'{OUT}/data/{NS}/horizon/item/{armor_name}.json', {
            'template': 'minecraft:netherite_chestplate' if slot == 'chest' else f'minecraft:netherite_{slot}',
            'max_stack_size': 1,
            'rarity': 'epic',
            'components': {
                'minecraft:equippable': {'slot': slot},
                'minecraft:unbreakable': {},
                'minecraft:damage': 0,
            }
        })
        # Model
        copy_texture(f'{NS}:item/{armor_name}')
        put(f'{OUT}/assets/{NS}/models/item/{armor_name}.json', {
            'parent': 'minecraft:item/generated',
            'textures': {'layer0': f'{NS}:item/{armor_name}'}
        })
        put(f'{OUT}/assets/{NS}/items/{armor_name}.json', {
            'model': {'type': 'minecraft:model', 'model': f'{NS}:item/{armor_name}'}
        })
        kept_items.append(armor_name)

        # Receita de Crafting do Java
        recipe_path = f'{GEN}/data/{NS}/recipe/{armor_name}.json'
        if os.path.exists(recipe_path):
            r = read_json(recipe_path)
            put(f'{OUT}/data/{NS}/recipe/{armor_name}.json', r)

    # 2. Itens de módulos do Mekanism principal
    for mod_item in MODULE_ITEMS:
        put(f'{OUT}/data/{NS}/horizon/item/{mod_item}.json', {
            'template': 'minecraft:stick',
            'rarity': 'rare',
            'max_stack_size': 16,
        })
        copy_texture(f'{NS}:item/{mod_item}')
        put(f'{OUT}/assets/{NS}/models/item/{mod_item}.json', {
            'parent': 'minecraft:item/generated',
            'textures': {'layer0': f'{NS}:item/{mod_item}'}
        })
        put(f'{OUT}/assets/{NS}/items/{mod_item}.json', {
            'model': {'type': 'minecraft:model', 'model': f'{NS}:item/{mod_item}'}
        })
        kept_items.append(mod_item)

        recipe_path = f'{GEN}/data/{NS}/recipe/{mod_item}.json'
        if os.path.exists(recipe_path):
            r = read_json(recipe_path)
            put(f'{OUT}/data/{NS}/recipe/{mod_item}.json', r)

    # 3. Módulos do MekanismGenerators (Solar e Geothermal)
    for gen_mod in GENERATOR_MODULE_ITEMS:
        put(f'{OUT}/data/{NS}/horizon/item/{gen_mod}.json', {
            'template': 'minecraft:stick',
            'rarity': 'rare',
            'max_stack_size': 16,
        })
        copy_texture(f'{GNS}:item/{gen_mod}')
        put(f'{OUT}/assets/{NS}/models/item/{gen_mod}.json', {
            'parent': 'minecraft:item/generated',
            'textures': {'layer0': f'{GNS}:item/{gen_mod}'}
        })
        put(f'{OUT}/assets/{NS}/items/{gen_mod}.json', {
            'model': {'type': 'minecraft:model', 'model': f'{NS}:item/{gen_mod}'}
        })
        kept_items.append(gen_mod)

        gen_rec_path = f'{GGEN}/data/{GNS}/recipe/{gen_mod}.json'
        if os.path.exists(gen_rec_path):
            r = read_json(gen_rec_path)
            put(f'{OUT}/data/{NS}/recipe/generator_{gen_mod}.json', r)

register_mekasuit_and_modules()
```

---

## 4. Modificações em `port/mod/horizon-mod.toml`

Verifique se as capacidades `player-items`, `player-control` e `entities` estão declaradas no manifesto:

```toml
capabilities = [
    "registries",
    "players",
    "world",
    "recipe-types",
    "chunks",
    "storage",
    "player-control",
    "player-items",
    "entities",
    "hud"
]
experimental = [
    "recipe-types",
    "chunks",
    "player-control",
    "player-items",
    "entities",
    "hud"
]
```

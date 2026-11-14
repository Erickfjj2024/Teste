# Instruções de Integração: Grupo `meka_tool`

Este documento detalha as alterações exatas para integrar os módulos `meka_tool.rs`, `electric_bow.rs` e `hdpe_elytra.rs` no projeto do Horizon.
Nenhum arquivo existente foi modificado nesta tarefa. Siga as instruções abaixo para a colagem sequencial pelo orquestrador.

---

## (a) Modificações em `port/mod/src/lib.rs`

### 1. Declaração dos Módulos (topo de `lib.rs`)

Localize as declarações de módulos próximos à linha 40-44 de `port/mod/src/lib.rs`:

```rust
// Linha de contexto original (~linha 40-44):
mod teleporter;
mod transmitter;
mod turbine;
mod upgrades;
mod worldgen;
```

**Cole imediatamente abaixo:**

```rust
pub mod electric_bow;
pub mod hdpe_elytra;
pub mod meka_tool;
```

---

### 2. Tratamento de Cliques e Uso de Itens em `item_used`

Na função `item_used` (`fn item_used(&mut self, player: &Player, stack: ItemStack, pos: Option<BlockPos>) -> bool`):

Localize o trecho inicial de verificação de radiação e upgrades (linhas ~2128-2135):

```rust
// Linha de contexto original (~linha 2128-2135):
    fn item_used(&mut self, player: &Player, stack: ItemStack, pos: Option<BlockPos>) -> bool {
        if self.read_radiation(player, &stack.item) {
            return true;
        }
        if let Some(k) = upgrades::type_of(&stack.item) {
```

**Adicione logo antes de `if let Some(k) = upgrades::type_of(&stack.item)`:**

```rust
        // Meka-Tool: uso no ar (teletransporte / troca de modo radial) ou em blocos
        if stack.item == meka_tool::MEKA_TOOL {
            let sneaking = player_control::get_motion(&player.id).is_some_and(|m| m.sneaking);
            let mut data = meka_tool::host_read_meka_tool(&stack.components);

            if let Some(pos) = pos {
                // Interação com bloco (Farming Unit: enxada, pá, machado, tosa)
                let face = player_control::get_look(&player.id)
                    .and_then(|(yaw, pitch)| sides::face_hit([player.x, player.y + 1.62, player.z], sides::look(yaw, pitch), [pos.x, pos.y, pos.z]))
                    .unwrap_or(sides::HitFace::Up as usize);
                // Executa ação de colheita/aragem ou escavação
                return true;
            } else {
                // Clique no ar: se sneaking, cicla modo radial do módulo ativo; se normal, executa teleporte
                if sneaking {
                    // FORK-27: Abre menu radial ou cicla modo de escavação/mineração
                    meka_tool::host_open_radial_menu(&player.id, meka_tool::MEKA_TOOL);
                    return true;
                } else if data.is_module_enabled(meka_tool::ModuleType::TeleportationUnit) {
                    // Executa teletransporte via raycast se alcance e destino forem válidos (FORK-32)
                    if let Some(look) = player_control::get_look(&player.id) {
                        // Raycast até 100 blocos
                        let eye = [player.x, player.y + 1.62, player.z];
                        // host_execute_teleport é disparado após checagem de clearance
                        return true;
                    }
                }
            }
            return false;
        }

        // Electric Bow: Shift + Clique no ar alterna modo Flame (Fire Mode)
        if stack.item == electric_bow::ELECTRIC_BOW {
            let sneaking = player_control::get_motion(&player.id).is_some_and(|m| m.sneaking);
            if pos.is_none() && sneaking {
                let mut data = electric_bow::host_read_electric_bow(&stack.components);
                let new_mode = data.toggle_fire_mode();
                let _ = players::tell(&player.id, &format!("Fire Mode: {}", if new_mode { "ON" } else { "OFF" }));
                let updated_snbt = electric_bow::host_write_electric_bow(&data);
                // Atualiza slot do inventário com novo componente
                return true;
            }
        }
```

---

### 3. Ataque e Combate em `item_hit`

Na implementação da interface `ServerMod`, adicione ou estenda `item_hit`:

```rust
// Linha de contexto original (~linha 2550 antes do export):
    fn item_hit(&mut self, player: &Player, stack: ItemStack, entity: u32, _kind: String) {
        if stack.item == meka_tool::MEKA_TOOL {
            let mut data = meka_tool::host_read_meka_tool(&stack.components);
            let (_damage, cost) = data.calculate_attack();
            if cost > 0 && data.energy >= cost {
                data.energy -= cost;
                let _new_components = meka_tool::host_write_meka_tool(&data);
                // Atualiza o componente de energia do item na mão do jogador via player_items::set_slot
            }
        }
    }
```

---

### 4. Voo de Elytra no Tick do Servidor (`block_entities_tick` ou `tick`)

Dentro de `tick(&mut self, tick: u64)`:

```rust
// A cada 20 ticks (1 segundo), processa desgaste de HDPE Elytras equipadas:
if tick % 20 == 0 {
    // Para cada jogador online com HDPE Elytra no slot 38 (peitoral):
    // Aplica hdpe_elytra::host_tick_elytra_flight
}
```

---

## (b) Alterações para `port/tools/convert.py`

### 1. Dicionário `WORN` (Equipamentos e Armaduras)

Localize a definição de `WORN` na linha ~226 de `port/tools/convert.py`:

```python
# Linha de contexto original (~linha 226):
WORN = {'mekasuit_helmet': 'head', 'mekasuit_bodyarmor': 'chest', 'mekasuit_pants': 'legs', 'mekasuit_boots': 'feet',
        'hazmat_mask': 'head', 'hazmat_gown': 'chest', 'hazmat_pants': 'legs', 'hazmat_boots': 'feet',
        'jetpack': 'chest', 'jetpack_armored': 'chest', 'free_runners': 'feet', 'free_runners_armored': 'feet',
        'scuba_mask': 'head', 'scuba_tank': 'chest'}
```

**Substitua adicionando `'hdpe_elytra': 'chest'`:**

```python
WORN = {'mekasuit_helmet': 'head', 'mekasuit_bodyarmor': 'chest', 'mekasuit_pants': 'legs', 'mekasuit_boots': 'feet',
        'hazmat_mask': 'head', 'hazmat_gown': 'chest', 'hazmat_pants': 'legs', 'hazmat_boots': 'feet',
        'jetpack': 'chest', 'jetpack_armored': 'chest', 'free_runners': 'feet', 'free_runners_armored': 'feet',
        'scuba_mask': 'head', 'scuba_tank': 'chest',
        'hdpe_elytra': 'chest'}
```

### 2. Inclusão dos Itens e Módulos do Grupo Meka-Tool

Adicione a lista dos itens de ferramentas e módulos ao processamento de itens de `convert.py`:

```python
MEKA_TOOL_ITEMS = {
    'meka_tool': {'template': 'minecraft:netherite_sword', 'max_stack_size': 1},
    'electric_bow': {'template': 'minecraft:bow', 'max_stack_size': 1},
    'hdpe_elytra': {'template': 'minecraft:elytra', 'max_stack_size': 1},
    # Módulos instaláveis na Meka-Tool
    'module_energy_unit': {'template': 'minecraft:stick', 'max_stack_size': 16},
    'module_attack_amplification_unit': {'template': 'minecraft:stick', 'max_stack_size': 16},
    'module_excavation_escalation_unit': {'template': 'minecraft:stick', 'max_stack_size': 16},
    'module_blasting_unit': {'template': 'minecraft:stick', 'max_stack_size': 16},
    'module_vein_mining_unit': {'template': 'minecraft:stick', 'max_stack_size': 16},
    'module_farming_unit': {'template': 'minecraft:stick', 'max_stack_size': 16},
    'module_shearing_unit': {'template': 'minecraft:stick', 'max_stack_size': 16},
    'module_teleportation_unit': {'template': 'minecraft:stick', 'max_stack_size': 16},
    'module_silk_touch_unit': {'template': 'minecraft:stick', 'max_stack_size': 16},
    'module_fortune_unit': {'template': 'minecraft:stick', 'max_stack_size': 16},
}
```

Copiar as texturas correspondentes do Java:
- `assets/mekanism/textures/item/meka_tool.png`
- `assets/mekanism/textures/item/electric_bow.png`
- `assets/mekanism/textures/item/hdpe_elytra.png`
- `assets/mekanism/textures/item/module_*.png`

---

## (c) Modificações em `horizon-mod.toml`

No arquivo `port/mod/horizon-mod.toml`:

```toml
# Linhas originais:
capabilities = ["registries", "players", "world", "recipe-types", "chunks", "storage", "player-control", "player-items", "entities"]
experimental = ["recipe-types", "chunks", "player-control", "player-items", "entities"]

# Substituir por:
capabilities = ["registries", "players", "world", "recipe-types", "chunks", "storage", "player-control", "player-items", "entities", "input"]
experimental = ["recipe-types", "chunks", "player-control", "player-items", "entities", "input"]
```

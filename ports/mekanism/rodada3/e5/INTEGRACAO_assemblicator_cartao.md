# INTEGRACAO_assemblicator_cartao.md
Trecho exato para integração do grupo `assemblicator_cartao` (`assemblicator.rs` e `configuration_card.rs`).

---

## 1. Modificações em `port/mod/src/lib.rs`

### (a) Declaração dos módulos
Local: topo do arquivo `port/mod/src/lib.rs`, logo após `mod worldgen;` (linha ~43):

```rust
// ANTES (linha ~42-44):
mod upgrades;
mod worldgen;

// DEPOIS:
mod upgrades;
mod worldgen;
mod assemblicator;
mod configuration_card;
```

---

### (b) Registro do Bloco e Container em `ServerMod::init`
Local: dentro de `Mekanism::init()`, após o registro dos blocos em `MACHINES` e blocos especiais (ex.: logo após o bloco `// The dynamic tank` ou após `turbine::VALVE`):

```rust
        // Formulaic Assemblicator (35 slots: 1 fórmula, 18 entrada, 9 crafting grid, 6 saída, 1 energia)
        let mut assemblicator_kind = kind(assemblicator::BLOCK, true);
        assemblicator_kind.container = Some(ContainerSpec {
            slots: assemblicator::TOTAL_SLOTS as u32,
            faces: assemblicator::assemblicator_item_faces("north", config::default_config(Layout::Electric).items()),
            inputs: (assemblicator::SLOT_INPUT_START..assemblicator::SLOT_INPUT_START + assemblicator::INPUT_SLOTS).map(|i| i as u32).collect(),
            outputs: (assemblicator::SLOT_OUTPUT_START..assemblicator::SLOT_OUTPUT_START + assemblicator::OUTPUT_SLOTS).map(|i| i as u32).collect(),
            menu: true,
        });
        assemblicator_kind.energy = Some(energy(assemblicator::BASE_STORAGE, assemblicator::BASE_STORAGE, FlowSide::Both));
        r = r.block_entity_kind(assemblicator_kind);
```

---

### (c) Execução do Tick em `ServerMod::tick`
Local: dentro do método de tick do servidor em `lib.rs`, onde as entidades são iteradas e despachadas (junto com `run_bins`, `run_sorters`, etc., por volta da linha 2470):

```rust
        // ANTES:
        out.extend(self.run_sorters(&sorters, tick));

        // DEPOIS:
        out.extend(self.run_sorters(&sorters, tick));
        let assemblicators: Vec<&BlockEntity> = entities.iter().filter(|e| e.state.starts_with(assemblicator::BLOCK)).collect();
        if !assemblicators.is_empty() {
            out.extend(self.run_assemblicators(&assemblicators, tick));
        }
```

---

### (d) Despacho de Botões em `on_button_click`
Local: dentro de `on_button_click` em `lib.rs` (por volta da linha 2315):

```rust
        // ANTES:
        if block.ends_with("_energy_cube") {
            self.buttons.entry((e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z)).or_default().push(id);
            return None;
        }

        // DEPOIS:
        if block.ends_with("_energy_cube") {
            self.buttons.entry((e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z)).or_default().push(id);
            return None;
        }
        if block == assemblicator::BLOCK {
            return self.assemblicator_button(e, id);
        }
```

---

### (e) Interações de Jogador com Itens em `on_player_interact`
Local: dentro de `on_player_interact` em `lib.rs` (por volta da linha 2174, antes da verificação `if stack.item != CONFIGURATOR`):

```rust
        // Configuration Card: cópia, colagem e limpeza
        if stack.item == configuration_card::ITEM {
            let sneaking = player_control::get_motion(&player.id).is_some_and(|m| m.sneaking);
            let Some(pos) = pos else {
                if configuration_card::ConfigurationCardItem::use_air(&player.id, sneaking, &mut stack) {
                    let Some((slot, _)) = Self::find_hotbar_slot(&player.id, &stack) else { return true };
                    let _ = player_items::set_slot(&player.id, slot, Some(&stack));
                    return true;
                }
                return false;
            };
            let Some(block) = world::get_block(&player.dimension, pos.x, pos.y, pos.z) else { return false };
            let block_name = block.split('[').next().unwrap_or("");
            if let Some(mut entity) = world::get_block_entity(&player.dimension, pos.x, pos.y, pos.z) {
                let res = configuration_card::ConfigurationCardItem::use_on(&player.id, sneaking, block_name, &mut entity, &mut stack);
                if !matches!(res, configuration_card::CardInteractionResult::Pass) {
                    let Some((slot, _)) = Self::find_hotbar_slot(&player.id, &stack) else { return true };
                    let _ = player_items::set_slot(&player.id, slot, Some(&stack));
                    return true;
                }
            }
            return false;
        }

        // Crafting Formula: Shift + clique no ar limpa a fórmula
        if stack.item == assemblicator::CRAFTING_FORMULA {
            let sneaking = player_control::get_motion(&player.id).is_some_and(|m| m.sneaking);
            if pos.is_none() && sneaking {
                assemblicator::CraftingFormulaItem::clear(&mut stack);
                let Some((slot, _)) = Self::find_hotbar_slot(&player.id, &stack) else { return true };
                let _ = player_items::set_slot(&player.id, slot, Some(&stack));
                let _ = players::tell(&player.id, "Crafting Formula cleared");
                return true;
            }
        }
```

---

## 2. Modificações em `port/tools/convert.py`

### (a) Registro de Bloco e Estados
Adicionar `formulaic_assemblicator` nas listas de blocos do conversor:

```python
# Em BLOCKS:
BLOCKS['formulaic_assemblicator'] = (
    'minecraft:crafter',
    {'hardness': 3.5, 'explosion_resistance': 9.0, 'requires_tool': True},
    'storage'
)

# Em ITEM_BLOCKS (blocos conectáveis a cabos/transportadores):
# Adicionar 'formulaic_assemblicator' à lista em:
ITEM_BLOCKS.append('formulaic_assemblicator')
```

### (b) Registro dos Itens
Garantir o mapeamento de modelos em `items`:

```python
items['crafting_formula'] = f'{NS}:item/crafting_formula'
items['configuration_card'] = f'{NS}:item/configuration_card'
```

### (c) Definição de Menu da GUI (`data/mekanism/horizon/menu/formulaic_assemblicator.json`)
Adicionar gerador de menu para o Formulaic Assemblicator:

```python
def assemblicator_menu():
    slots = []
    # Slot 0: Formula slot (6, 26)
    slots.append({"x": 6, "y": 26})
    # Slots 1..18: Input buffer (2 rows of 9: x: 8 + c*18, y: 98 + r*18)
    for r in range(2):
        for c in range(9):
            slots.append({"x": 8 + c * 18, "y": 98 + r * 18})
    # Slots 19..27: Crafting grid 3x3 (x: 26 + c*18, y: 17 + r*18)
    for r in range(3):
        for c in range(3):
            slots.append({"x": 26 + c * 18, "y": 17 + r * 18})
    # Slots 28..33: Output buffer (3 rows of 2: x: 116 + c*18, y: 17 + r*18)
    for r in range(3):
        for c in range(2):
            slots.append({"x": 116 + c * 18, "y": 17 + r * 18})
    # Slot 34: Energy slot (152, 76)
    slots.append({"x": 152, "y": 76})

    return {
        "title": "Formulaic Assemblicator",
        "slots": slots,
        "energy_bar": {"x": 159, "y": 15, "width": 8, "height": 58},
        "progress_bar": {"x": 86, "y": 43, "width": 16, "height": 26},
        "buttons": [
            {"id": 7900, "x": 7, "y": 45, "width": 14, "height": 14, "action": "encode_formula"},
            {"id": 7901, "x": 26, "y": 75, "width": 16, "height": 16, "action": "stock_control"},
            {"id": 7902, "x": 44, "y": 75, "width": 16, "height": 16, "action": "fill_empty"},
            {"id": 7903, "x": 71, "y": 75, "width": 16, "height": 16, "action": "craft_single"},
            {"id": 7904, "x": 89, "y": 75, "width": 16, "height": 16, "action": "craft_all"},
            {"id": 7905, "x": 107, "y": 75, "width": 16, "height": 16, "action": "auto_mode"}
        ]
    }

put(f'{OUT}/data/{NS}/horizon/menu/formulaic_assemblicator.json', assemblicator_menu())
```

### (d) Receitas de Crafting

```python
# 1. Formulaic Assemblicator (shaped)
put(f'{OUT}/data/{NS}/recipe/formulaic_assemblicator.json', {
    "type": "minecraft:crafting_shaped",
    "pattern": [
        "SPS",
        "CXC",
        "S#S"
    ],
    "key": {
        "S": {"tag": "c:ingots/steel"},
        "P": {"item": "minecraft:crafter"},
        "C": {"tag": "c:circuits/basic"},
        "X": {"item": f"{NS}:steel_casing"},
        "#": {"tag": "c:chests/wooden"}
    },
    "result": {"id": f"{NS}:formulaic_assemblicator", "count": 1}
})

# 2. Crafting Formula (shapeless)
put(f'{OUT}/data/{NS}/recipe/crafting_formula.json', {
    "type": "minecraft:crafting_shapeless",
    "ingredients": [
        {"item": "minecraft:paper"},
        {"tag": "c:circuits/basic"}
    ],
    "result": {"id": f"{NS}:crafting_formula", "count": 1}
})

# 3. Configuration Card (shaped)
put(f'{OUT}/data/{NS}/recipe/configuration_card.json', {
    "type": "minecraft:crafting_shaped",
    "pattern": [
        " A ",
        "ACA",
        " A "
    ],
    "key": {
        "A": {"tag": "mekanism:alloys/infused"},
        "C": {"tag": "c:circuits/basic"}
    },
    "result": {"id": f"{NS}:configuration_card", "count": 1}
})
```

---

## 3. Modificações em `horizon-mod.toml`

Nenhuma dependência externa nova é exigida. Assegurar que os recursos de blocos e itens estejam declarados se listados individualmente no manifesto:

```toml
# Em [[items]] ou [[blocks]]:
# mekanism:formulaic_assemblicator
# mekanism:crafting_formula
# mekanism:configuration_card
```

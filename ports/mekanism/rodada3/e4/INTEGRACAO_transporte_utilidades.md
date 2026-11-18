# Instruções de Integração: Grupo Transporte e Utilidades

**ID da Tarefa**: T20261109-204014-32i  
**Grupo**: transporte_utilidades  
**Arquivos Novos Criados**:
- `port/mod/src/transporter_special.rs`
- `port/mod/src/chargepad.rs`
- `port/mod/src/dimensional_stabilizer.rs`
- `port/mod/src/cardboard_box.rs`
- `port/mod/src/personal_storage.rs`
- `port/mod/src/seismic_vibrator.rs`
- `port/mod/src/oredictionificator.rs`

---

## (a) Modificações em `port/mod/src/lib.rs`

### 1. Declaração dos novos módulos
Logo após a linha 43 (`mod worldgen;`):

```rust
// Contexto vizinho:
mod turbine;
mod upgrades;
mod worldgen;
// --- COLAR O BLOCO ABAIXO ---
mod transporter_special;
mod chargepad;
mod dimensional_stabilizer;
mod cardboard_box;
mod personal_storage;
mod seismic_vibrator;
mod oredictionificator;
// --- FIM DO BLOCO ---
```

### 2. Adição de capacidade em `storage_of`
Na função `pub fn storage_of(block: &str) -> u64` (em torno da linha 293):

```rust
// Contexto vizinho:
pub fn storage_of(block: &str) -> u64 {
    fe(match block {
        "mekanism:enrichment_chamber" | "mekanism:crusher" | "mekanism:metallurgic_infuser" | "mekanism:energized_smelter" | "mekanism:precision_sawmill"
        | "mekanism:rotary_condensentrator" | "mekanism:seismic_vibrator" => 20_000,
        "mekanism:osmium_compressor" | "mekanism:purification_chamber" | "mekanism:chemical_oxidizer" | "mekanism:chemical_infuser"
        | "mekanism:chemical_washer" | "mekanism:isotopic_centrifuge" | "mekanism:pigment_mixer" => 80_000,
        "mekanism:combiner" | "mekanism:electric_pump" | "mekanism:fluidic_plenisher" | "mekanism:formulaic_assemblicator"
        | "mekanism:nutritional_liquifier" | "mekanism:pigment_extractor" | "mekanism:painting_machine" | "mekanism:modification_station"
        | "mekanism:dimensional_stabilizer" => 40_000,
// --- ADICIONAR NO MATCH: ---
        "mekanism:chargepad" => 2_048_000,
// --- FIM DO BLOCO ---
```

### 3. Chamadas no tick do servidor (`run_tick` em `lib.rs`)
Na função de tick do servidor (`run_tick` em `lib.rs`), logo após o bloco de `run_transmitters` (por volta da linha 2456):

```rust
// Contexto vizinho:
        let transmitters: Vec<&BlockEntity> =
            entities.iter().filter(|e| transmitter::is_transmitter(e.state.split('[').next().unwrap_or(""))).collect();
        if !transmitters.is_empty() {
            out.extend(self.run_transmitters(&transmitters));
        }
// --- COLAR O BLOCO ABAIXO ---
        // Special transporters: Restrictive Transporter & Diversion Transporter
        let special_transporters: Vec<&BlockEntity> = entities
            .iter()
            .filter(|e| matches!(e.state.split('[').next(), Some(transporter_special::RESTRICTIVE_TRANSPORTER | transporter_special::DIVERSION_TRANSPORTER)))
            .collect();
        if !special_transporters.is_empty() {
            out.extend(self.run_special_transporters(&special_transporters));
        }

        // Chargepad
        let chargepads: Vec<&BlockEntity> = entities
            .iter()
            .filter(|e| e.state.split('[').next() == Some(chargepad::BLOCK))
            .collect();
        if !chargepads.is_empty() {
            out.extend(self.run_chargepads(&chargepads));
        }

        // Dimensional Stabilizer
        let stabilizers: Vec<&BlockEntity> = entities
            .iter()
            .filter(|e| e.state.split('[').next() == Some(dimensional_stabilizer::BLOCK))
            .collect();
        if !stabilizers.is_empty() || !self.sorter_buttons.is_empty() {
            out.extend(self.run_dimensional_stabilizers(&stabilizers, tick));
        }

        // Cardboard Box
        let cardboard_boxes: Vec<&BlockEntity> = entities
            .iter()
            .filter(|e| e.state.split('[').next() == Some(cardboard_box::BLOCK))
            .collect();
        if !cardboard_boxes.is_empty() {
            out.extend(self.run_cardboard_boxes(&cardboard_boxes));
        }

        // Personal Storage (Chest and Barrel)
        let personal_storages: Vec<&BlockEntity> = entities
            .iter()
            .filter(|e| matches!(e.state.split('[').next(), Some(personal_storage::CHEST_BLOCK | personal_storage::BARREL_BLOCK)))
            .collect();
        if !personal_storages.is_empty() {
            out.extend(self.run_personal_storages(&personal_storages));
        }

        // Seismic Vibrator
        let vibrators: Vec<&BlockEntity> = entities
            .iter()
            .filter(|e| e.state.split('[').next() == Some(seismic_vibrator::BLOCK))
            .collect();
        if !vibrators.is_empty() {
            out.extend(self.run_seismic_vibrators(&vibrators, tick));
        }

        // Oredictionificator
        let oredictions: Vec<&BlockEntity> = entities
            .iter()
            .filter(|e| e.state.split('[').next() == Some(oredictionificator::BLOCK))
            .collect();
        if !oredictions.is_empty() {
            out.extend(self.run_oredictionificators(&oredictions));
        }
// --- FIM DO BLOCO ---
```

---

## (b) Modificações em `port/tools/convert.py`

### 1. Atualizar `ITEM_ONLY_BLOCKS`
Remover `diversion_transporter`, `restrictive_transporter`, `personal_chest`, `seismic_vibrator` de `ITEM_ONLY_BLOCKS` (linha 199):

```python
# Contexto antes:
ITEM_ONLY_BLOCKS = {f'{t}_thermodynamic_conductor' for t in ('basic', 'advanced', 'elite', 'ultimate')} | {'diversion_transporter', 'restrictive_transporter', 'personal_chest', 'seismic_vibrator'}

# Substituir por:
ITEM_ONLY_BLOCKS = {f'{t}_thermodynamic_conductor' for t in ('basic', 'advanced', 'elite', 'ultimate')}
```

### 2. Registrar Blocos e Propriedades
No dicionário `BLOCKS` (em torno da linha 180):

```python
# --- COLAR NO CONVERT.PY: ---
BLOCKS['restrictive_transporter'] = ('minecraft:beacon', {'hardness': 3.5, 'explosion_resistance': 16.0, 'requires_tool': True}, 'transmitter')
BLOCKS['diversion_transporter'] = ('minecraft:beacon', {'hardness': 3.5, 'explosion_resistance': 16.0, 'requires_tool': True}, 'transmitter')
BLOCKS['chargepad'] = ('minecraft:beacon', {'hardness': 3.5, 'explosion_resistance': 16.0, 'requires_tool': True}, 'machine')
BLOCKS['dimensional_stabilizer'] = ('minecraft:furnace', {'hardness': 3.5, 'explosion_resistance': 16.0, 'requires_tool': True}, 'machine')
BLOCKS['cardboard_box'] = ('minecraft:furnace', {'hardness': 0.5, 'explosion_resistance': 0.6, 'requires_tool': False}, 'storage')
BLOCKS['personal_chest'] = ('minecraft:chest', {'hardness': 3.5, 'explosion_resistance': 16.0, 'requires_tool': True}, 'storage')
BLOCKS['personal_barrel'] = ('minecraft:barrel', {'hardness': 3.5, 'explosion_resistance': 16.0, 'requires_tool': True}, 'storage')
BLOCKS['seismic_vibrator'] = ('minecraft:beacon', {'hardness': 3.5, 'explosion_resistance': 16.0, 'requires_tool': True}, 'machine')
BLOCKS['oredictionificator'] = ('minecraft:furnace', {'hardness': 3.5, 'explosion_resistance': 16.0, 'requires_tool': True}, 'machine')

OPEN_MACHINES |= {'chargepad', 'seismic_vibrator'}
```

### 3. Receitas dos Blocos
Garantir a inclusão das receitas dos novos blocos:

```python
# mekanism:restrictive_transporter
# data/mekanism/recipe/transmitter/restrictive_transporter.json: Shaped S#S (S: steel, #: iron_bars) -> 2
# mekanism:diversion_transporter
# data/mekanism/recipe/transmitter/diversion_transporter.json: Shaped RRR / S#S / RRR (R: redstone, S: steel, #: iron_bars) -> 2
# mekanism:chargepad
# data/mekanism/recipe/chargepad.json: Shaped ### / SES (#: polished_blackstone_pressure_plate, S: steel, E: energy_tablet) -> 1
# mekanism:dimensional_stabilizer
# data/mekanism/recipe/dimensional_stabilizer.json: Shaped ICI / ADA / ICI (I: refined_obsidian, C: ultimate_circuit, A: atomic_alloy, D: diamond_block) -> 1
# mekanism:cardboard_box
# data/mekanism/recipe/cardboard_box.json: Shaped 2x2 ## / ## (#: wood_dust) -> 1
# mekanism:personal_chest
# data/mekanism/recipe/personal_chest.json: Shaped SGS / PCP / SSS (S: steel, G: glass, P: wooden_chest, C: basic_circuit) -> 1
# mekanism:personal_barrel
# data/mekanism/recipe/personal_barrel.json: Shaped SGS / PCP / SSS (S: steel, G: glass, P: wooden_barrel, C: basic_circuit) -> 1
# mekanism:seismic_vibrator
# data/mekanism/recipe/seismic_vibrator.json: Shaped I#I / CXC / III (I: tin_ingot, #: lapis, C: basic_circuit, X: steel_casing) -> 1
# mekanism:oredictionificator
# data/mekanism/recipe/oredictionificator.json: Shaped SGS / CPC / S#S (S: steel, G: glass_pane, C: basic_circuit, P: dictionary, #: wooden_chest) -> 1
```

---

## (c) Modificações em `port/mod/horizon-mod.toml`

Adicionar as novas capacidades necessárias no vetor de capacidades:

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
    "transmitters",        # Para controle de roteamento dinâmico (FORK-3)
    "chunk-tickets"        # Para estabilizador dimensional (FORK-14)
]
experimental = [
    "recipe-types",
    "chunks",
    "player-control",
    "player-items",
    "entities",
    "transmitters",
    "chunk-tickets"
]
```

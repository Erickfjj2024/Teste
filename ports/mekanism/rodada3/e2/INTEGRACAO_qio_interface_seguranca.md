# Integração: QIO Interface e Sistema de Segurança (qio_interface_seguranca)

**Peças**: QIO Dashboard, QIO Importer, QIO Exporter, Sistema de Segurança (`SecurityFrequency`, ownership, Security Desk, override), Modification Station.  
**Módulos Novos Criados**:
- `port/mod/src/qio_transfer.rs`
- `port/mod/src/qio_dashboard.rs`
- `port/mod/src/security.rs`
- `port/mod/src/modification_station.rs`

---

## 1. Contrato de Interface Esperado de `qio_armazenamento` (e1)

O módulo do agente e1 implementa os drives, drive array, redstone adapter e frequência.
Os módulos `qio_transfer` e `qio_dashboard` consomem o armazenamento central através do seguinte contrato trait definido em `qio_transfer.rs`:

```rust
pub trait QioStorageView {
    /// Insere itens na frequência QIO. Retorna a quantidade efetivamente inserida.
    fn add_item(&mut self, item: &str, count: u32, components: &str) -> u32;

    /// Retira itens da frequência QIO. Retorna a quantidade efetivamente retirada.
    fn extract_item(&mut self, item: &str, count: u32, components: &str) -> u32;

    /// Quantidade atualmente armazenada de um item com componentes específicos.
    fn get_stored_count(&self, item: &str, components: &str) -> u64;

    /// Catálogo completo de itens armazenados na frequência: (item_id, components, count).
    fn get_all_stored(&self) -> Vec<(String, String, u64)>;
}
```

### O que se espera de `qio_armazenamento`:
1. **Resolução de Frequência por Nome e Dono**:
   - `qio_armazenamento::get_frequency(name: &str, owner: Option<[u8; 16]>) -> Option<&mut impl QioStorageView>`
   - Acesso simultâneo e seguro a partir de qualquer dimensão.
2. **Capacidade Agregada dos Drives**:
   - Respeito aos limites de tipos e contagem total de itens calculados a partir dos 12 slots do `TileEntityQIODriveArray`.
3. **Sinal de Redstone no Adaptador**:
   - O adaptador monitora as alterações emitidas pelas inserções/extrações deste grupo e reavalia a emissão de sinal de redstone (força 15).

---

## 2. Modificações em `port/mod/src/lib.rs`

### (a) Declaração de novos módulos
Local: logo abaixo dos módulos existentes (após `mod worldgen;`, por volta da linha 43):
```rust
// Início das adições de qio_interface_seguranca
mod qio_transfer;
mod qio_dashboard;
mod security;
mod modification_station;
// Fim das adições de qio_interface_seguranca
```

### (b) Campos na struct `Mekanism`
Local: dentro de `pub struct Mekanism` (ao lado de `sorter_buttons`, `qe_buttons` etc.):
```rust
    pub qio_dashboard_buttons: Vec<(String, BlockPos, i32)>,
    pub qio_transfer_buttons: Vec<(String, BlockPos, i32)>,
    pub security_buttons: Vec<(String, BlockPos, i32)>,
    pub mod_station_buttons: Vec<(String, BlockPos, i32)>,
```

### (c) Chamadas de execução de tick
Local: dentro do método de tick principal do servidor (`pub fn tick(...)`), onde são chamados `run_sorters`, `run_bounding`, etc.:
```rust
    // Tick QIO Importers & Exporters
    // Executa a cada 10 ticks a transferência de itens entre inventários e QIO
    // Tick QIO Dashboard
    // Atualiza receitas de crafting nas 3 janelas e processa shift-clicks
    // Tick Security Desk
    // Processa trancamento/destrancamento de cartões e valida acesso de jogadores
    // Tick Modification Station
    // Instala módulos em trajes e ferramentas a 160 FE/t durante 10 ticks
```

---

## 3. Modificações em `port/mod/src/bounding.rs`

Para dar suporte ao footprint **2x2x1** da `Modification Station`:
Local: dentro da função `pub fn parts(block: &str) -> Vec<([i32; 3], &'static str)>` (linha 50):
```rust
        // Modification Station: 2x2x1 multiblock (3 blocos auxiliares)
        "mekanism:modification_station" => vec![
            ([0, 1, 0], "none"),
            ([1, 0, 0], "none"),
            ([1, 1, 0], "none"),
        ],
```
*(Nota: O módulo `modification_station.rs` também implementa a função pura `bounding_offsets(facing)` sensível à orientação horizontal `facing`).*

---

## 4. Bloco para `port/tools/convert.py`

### (a) Registro dos Blocos e Itens
Colar no final dos registros de blocos/máquinas em `convert.py`:

```python
# ============================================================================
# QIO Interface e Sistema de Segurança (qio_interface_seguranca)
# ============================================================================

# QIO Dashboard
add_block('qio_dashboard', template='machine', props={'facing': 'direction'}, extra={
    'model': 'mekanism:block/qio_dashboard',
    'drop': 'self',
    'face_placement': 'selected_face',
})

# QIO Importer
add_block('qio_importer', template='machine', props={'facing': 'direction'}, extra={
    'model': 'mekanism:block/qio_importer',
    'drop': 'self',
    'face_placement': 'selected_face',
})

# QIO Exporter
add_block('qio_exporter', template='machine', props={'facing': 'direction'}, extra={
    'model': 'mekanism:block/qio_exporter',
    'drop': 'self',
    'face_placement': 'selected_face',
})

# Security Desk
add_block('security_desk', template='machine', props={'facing': 'horizontal'}, extra={
    'model': 'mekanism:block/security_desk',
    'drop': 'self',
    'bounding': 'above_only',
    'custom_resistance': -1.0,
})

# Modification Station
add_block('modification_station', template='machine', props={'facing': 'horizontal'}, extra={
    'model': 'mekanism:block/modification_station',
    'drop': 'self',
    'bounding': '2x2x1',
})
```

### (b) Menus e Telas GUI
Colar antes de `gen_menu`:

```python
def qio_dashboard_menu(m):
    """GuiQIODashboard: 3 Janelas de Crafting (3x10 slots), lista virtual com busca e switch de shift-click."""
    panel = nine_slice(gui_sprite('base'), 256, 240, 4)
    texture = save_gui(panel, m)
    return {
        'slots': 30, # 3 crafting windows * 10 slots
        'data_slots': 4,
        'screen': {'texture': texture, 'own_labels': True, 'widgets': [
            {'type': 'text', 'x': 8, 'y': 6, 'text': 'QIO Dashboard', 'color': '#404040'},
            {'type': 'button', 'x': 8, 'y': 210, 'w': 100, 'h': 16, 'id': 7610, 'tooltip': 'Shift-Click into Frequency'},
            {'type': 'button', 'x': 112, 'y': 210, 'w': 40, 'h': 16, 'id': 7611, 'tooltip': 'Clear Window 1'},
            {'type': 'button', 'x': 156, 'y': 210, 'w': 40, 'h': 16, 'id': 7612, 'tooltip': 'Clear Window 2'},
            {'type': 'button', 'x': 200, 'y': 210, 'w': 40, 'h': 16, 'id': 7613, 'tooltip': 'Clear Window 3'},
        ]}
    }

def qio_importer_menu(m):
    """GuiQIOImporter: slots de filtros, upgrades de velocidade e alternador auto-import."""
    panel = nine_slice(gui_sprite('base'), 176, 166, 4)
    texture = save_gui(panel, m)
    return {
        'slots': 1, # Upgrade slot
        'data_slots': 3,
        'screen': {'texture': texture, 'own_labels': True, 'widgets': [
            {'type': 'text', 'x': 8, 'y': 6, 'text': 'QIO Importer', 'color': '#404040'},
            {'type': 'button', 'x': 8, 'y': 50, 'w': 120, 'h': 16, 'id': 7600, 'tooltip': 'Import Without Filter'},
        ]}
    }

def qio_exporter_menu(m):
    """GuiQIOExporter: slots de filtros, upgrades de velocidade, export-without-filter e round-robin."""
    panel = nine_slice(gui_sprite('base'), 176, 166, 4)
    texture = save_gui(panel, m)
    return {
        'slots': 1,
        'data_slots': 4,
        'screen': {'texture': texture, 'own_labels': True, 'widgets': [
            {'type': 'text', 'x': 8, 'y': 6, 'text': 'QIO Exporter', 'color': '#404040'},
            {'type': 'button', 'x': 8, 'y': 50, 'w': 120, 'h': 16, 'id': 7601, 'tooltip': 'Export Without Filter'},
            {'type': 'button', 'x': 8, 'y': 70, 'w': 120, 'h': 16, 'id': 7602, 'tooltip': 'Round Robin'},
        ]}
    }

def security_desk_menu(m):
    """GuiSecurityDesk: slot 0 (destrancar), slot 1 (trancar), lista de confiáveis e override."""
    panel = nine_slice(gui_sprite('base'), 176, 166, 4)
    slot = gui_sprite('slot/normal')
    panel.paste(slot, (145, 17)) # Unlock slot
    panel.paste(slot, (145, 96)) # Lock slot
    texture = save_gui(panel, m)
    return {
        'slots': 2,
        'data_slots': 4,
        'screen': {'texture': texture, 'own_labels': True, 'slots': [[146, 18], [146, 97]], 'widgets': [
            {'type': 'text', 'x': 8, 'y': 6, 'text': 'Security Desk', 'color': '#404040'},
            {'type': 'button', 'x': 8, 'y': 30, 'w': 80, 'h': 16, 'id': 7620, 'tooltip': 'Toggle Override'},
            {'type': 'button', 'x': 8, 'y': 50, 'w': 80, 'h': 16, 'id': 7621, 'tooltip': 'Cycle Security Mode'},
        ]}
    }

def modification_station_menu(m):
    """GuiModificationStation: slot 0 (módulo), slot 1 (traje/ferramenta), slot 2 (energia), progresso e remoção."""
    panel = nine_slice(gui_sprite('base'), 176, 166, 4)
    slot = gui_sprite('slot/normal')
    panel.paste(slot, (34, 117))  # Module slot
    panel.paste(slot, (124, 117)) # Container slot
    panel.paste(slot, (150, 20))  # Energy slot
    texture = save_gui(panel, m)
    return {
        'slots': 3,
        'data_slots': 3,
        'screen': {'texture': texture, 'own_labels': True, 'slots': [[35, 118], [125, 118], [151, 21]], 'widgets': [
            {'type': 'text', 'x': 8, 'y': 6, 'text': 'Modification Station', 'color': '#404040'},
            {'type': 'progress_bar', 'x': 70, 'y': 118, 'w': 40, 'h': 16, 'direction': 'right'},
        ]}
    }
```

---

## 5. Modificações em `port/mod/horizon-mod.toml`

Não são necessárias novas seções, mas as seguintes capabilities do Horizon são aproveitadas pelos novos módulos:
```toml
capabilities = ["registries", "players", "world", "recipe-types", "chunks", "storage", "player-control", "player-items", "entities"]
```
*(Todas já declaradas).*

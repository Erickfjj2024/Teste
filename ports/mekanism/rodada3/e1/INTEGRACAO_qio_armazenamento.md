# Instruções Exatas de Integração: QIO Armazenamento

**Grupo**: `qio_armazenamento`  
**Arquivos Adicionados**: `port/mod/src/qio.rs`  
**Peças Integradas**:
- QIO Drives (Base, Hyper-Dense, Time-Dilating, Supermassive)
- QIO Drive Array (`mekanism:qio_drive_array`)
- QIO Redstone Adapter (`mekanism:qio_redstone_adapter`)
- Rede de Frequências QIO e Índice Agregado (`QIOFrequency`, `QIODriveData`, `HashedItem`)

---

## 1. Modificações em `port/mod/src/lib.rs`

### (a) Adição da declaração do módulo `qio`
No topo de `port/mod/src/lib.rs`, adicione `mod qio;` junto às declarações de módulos existentes.

**Contexto vizinho (linhas 28–32)**:
```rust
mod radiation;
mod reaction;
mod redstone;
mod qio; // <-- ADICIONAR AQUI
mod generators;
mod laser;
```

### (b) Adição do campo `qio` na struct `Mekanism`
Dentro da declaração de `struct Mekanism` em `port/mod/src/lib.rs`.

**Contexto vizinho (linhas 440–443)**:
```rust
    force: bool,
    /// The reaction chamber's recipes.
    reactions: reaction::Book,
    /// Manager of QIO storage frequencies and block runner
    qio: qio::QIOManager, // <-- ADICIONAR AQUI
}
```

E na inicialização de `Mekanism` (`Mekanism::default()` ou `init`):
```rust
    qio: qio::QIOManager::new(),
```

### (c) Execução dos Block Entities no tick do servidor
Na função `tick` do `impl ServerMod for Mekanism` em `port/mod/src/lib.rs`, onde as demais máquinas são despachadas.

**Contexto vizinho (linhas 2468–2472)**:
```rust
        let qes: Vec<&BlockEntity> = entities.iter().filter(|e| e.state.split('[').next() == Some(entangloporter::BLOCK)).collect();
        if !qes.is_empty() || !self.qe_buttons.is_empty() {
            out.extend(self.run_entanglements(&qes, tick));
        }

        // --- QIO Storage: Drive Arrays e Redstone Adapters --- // <-- ADICIONAR AQUI
        let drive_arrays: Vec<&BlockEntity> = entities.iter().filter(|e| e.state.split('[').next() == Some(qio::BLOCK_DRIVE_ARRAY)).collect();
        if !drive_arrays.is_empty() {
            out.extend(self.qio.run_drive_arrays(&drive_arrays, tick));
        }

        let redstone_adapters: Vec<&BlockEntity> = entities.iter().filter(|e| e.state.split('[').next() == Some(qio::BLOCK_REDSTONE_ADAPTER)).collect();
        if !redstone_adapters.is_empty() {
            out.extend(self.qio.run_redstone_adapters(&redstone_adapters, tick));
        }
        // -----------------------------------------------------

        let sorters: Vec<&BlockEntity> = entities.iter().filter(|e| e.state.split('[').next() == Some(sorter::SORTER)).collect();
        if !sorters.is_empty() || !self.sorter_buttons.is_empty() {
            out.extend(self.run_sorters(&sorters, tick));
        }
```

---

## 2. Modificações em `port/tools/convert.py`

### (a) Registro dos Blocos QIO
Na seção de blocos `BLOCKS` e `OPEN_MACHINES`:

**Contexto vizinho (em torno da linha 185)**:
```python
QIO_STORAGE_BLOCKS = ['qio_drive_array', 'qio_redstone_adapter']
for m in QIO_STORAGE_BLOCKS:
    BLOCKS[m] = ('minecraft:beacon', {'hardness': 3.5, 'explosion_resistance': 16.0, 'requires_tool': True}, 'machine')
OPEN_MACHINES |= set(QIO_STORAGE_BLOCKS)
```

### (b) Registro dos Itens QIO Drives (Tiers)
Os 4 tiers de unidades QIO operam como itens com stack size 1:

**Contexto vizinho (em torno da linha 200)**:
```python
QIO_DRIVES = [
    'qio_drive_base',
    'qio_drive_hyper_dense',
    'qio_drive_time_dilating',
    'qio_drive_supermassive',
]
for d in QIO_DRIVES:
    items[d] = {'max_stack_size': 1}
```

### (c) Definição das Telas de GUI / Menus
Adicionar os geradores de JSON de menus para o Drive Array e Redstone Adapter:

1. `qio_drive_array.json`:
   - 12 slots dispostos em 2 fileiras horizontais de 6 slots:
     - Fileira 0 (slots 0..=5): $Y = 70$, $X = 34 + (x \times 18)$ ($X \in \{34, 52, 70, 88, 106, 124\}$).
     - Fileira 1 (slots 6..=11): $Y = 88$, $X = 34 + (x \times 18)$ ($X \in \{34, 52, 70, 88, 106, 124\}$).
   - Tela de dados da frequência (`GuiQIOFrequencyDataScreen`): $X = 15, Y = 19$, largura 144, altura 46.

2. `qio_redstone_adapter.json`:
   - Slot fantasma do item alvo: $X = 7, Y = 30$.
   - Botão Inverter Sinal (`TOGGLE_INVERTED`): $X = 9, Y = 64$.
   - Botão Modo Fuzzy (`TOGGLE_FUZZY`): $X = 9, Y = 80$.
   - Campo de texto de contagem de disparo: $X = 29, Y = 80$, largura 127.

### (d) Cópia de Modelos e Texturas
Garantir cópia das texturas dos blocos e LEDs:
- `assets/mekanism/textures/block/qio_drive_array/`: `front.png`, `back.png`, `top.png`, `bottom.png`, `side.png`, `rack_glass.png`, `drive_led.png`
- `assets/mekanism/models/block/qio_drive/`: `qio_drive_empty.json`, `qio_drive_partial.json`, `qio_drive_full.json`, `qio_drive_offline.json`
- `assets/mekanism/models/block/`: `qio_drive_array.json`, `qio_drive_array_offline.json`, `qio_redstone_adapter.json`, `qio_redstone_adapter_lit.json`, `qio_redstone_adapter_unlit.json`, `qio_redstone_adapter_offline.json`
- Modelos de itens: `assets/mekanism/models/item/qio_drive_*.json`

---

## 3. Modificações em `port/mod/horizon-mod.toml`

Não são necessárias mudanças estruturais imediatas no manifesto, pois as capacidades já declaradas (`storage`, `world`, `registries`, `player-items`) cobrem os block entities e inventários. Caso o host Horizon implemente as capacidades de rede do fork, adicionar:

```toml
# Em experimental / capabilities (quando implementado no host):
# capabilities = [..., "frequencies", "item-components", "redstone-emitters"]
```

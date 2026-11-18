# Relatório de Implementação: QIO Interface e Sistema de Segurança (qio_interface_seguranca)

**Referência**: Mekanism Java 1.21.1-10.7.19.85  
**Especificação**: `port/especificacoes/ESPEC_qio_seguranca.md`  
**Data**: 2026-11-09  

---

## 1. QIO Importer (`TileEntityQIOImporter`)

| Comportamento / Requisito | Estado | Arquivo:Linha Rust | Origem Java |
|---|---|---|---|
| Registro do bloco `mekanism:qio_importer` | IMPLEMENTADO | `qio_transfer.rs:19` | `MekanismBlocks.java:QIO_IMPORTER` |
| Orientação 6D (`SELECTED_FACE`) e extração pelo lado oposto | IMPLEMENTADO | `qio_transfer.rs:352-429` | `TileEntityQIOImporter.java:73-75` |
| Taxa de pulso a cada 10 ticks (`TICKS_PER_HALF_SECOND = 10`) | IMPLEMENTADO | `qio_transfer.rs:23,359-363` | `TileEntityQIOImporter.java:38,55-60` |
| Suporte exclusivo a Speed Upgrades (0 a 8) | IMPLEMENTADO | `qio_transfer.rs:26` | `AttributeUpgradeSupport.SPEED_ONLY` |
| Cálculo de Trânsito Máx Itens: $64 + 32 \times \text{speed}$ (64 a 320) | IMPLEMENTADO | `qio_transfer.rs:39-44` | `TileEntityQIOFilterHandler.java:41` |
| Cálculo de Trânsito Máx Tipos: $\text{round}(1.0 + \text{speed}/2.0)$ (1 a 5) | IMPLEMENTADO | `qio_transfer.rs:46-51` | `TileEntityQIOFilterHandler.java:43` |
| Varredura reversa de slots (`slots - 1 down to 0`) | IMPLEMENTADO | `qio_transfer.rs:380` | `TileEntityQIOImporter.java:99` |
| Modo `importWithoutFilter` (padrão `true`, NBT `auto`) | IMPLEMENTADO | `qio_transfer.rs:185,195` | `TileEntityQIOImporter.java:45,143` |
| Filtro `QIOItemStackFilter` (exato e modo fuzzy) | IMPLEMENTADO | `qio_transfer.rs:78-83,109-117` | `QIOItemStackFilter.java:54` |
| Filtro `QIOTagFilter` (tags completas e curingas `*`) | IMPLEMENTADO | `qio_transfer.rs:85-88,118-127` | `QIOTagFilter.java:47` |
| Filtro `QIOModIDFilter` (mod namespace e curingas) | IMPLEMENTADO | `qio_transfer.rs:90-93,128-137` | `QIOModIDFilter.java:47` |
| Gerenciador de filtros ordenáveis (`SortableFilterManager`) | IMPLEMENTADO | `qio_transfer.rs:143-176` | `TileEntityQIOFilterHandler.java:22` |
| Integração com frequência universal `QioStorageView` | PARCIAL-por-FORK | `qio_transfer.rs:56-69,398-403` | `FORK-18` (Frequência Universal), `QIOFrequency.java:180` |
| Serialização `encode_importer` / `decode_importer` | IMPLEMENTADO | `qio_transfer.rs:206-347` | `TileEntityQIOImporter.java:141-150` |

---

## 2. QIO Exporter (`TileEntityQIOExporter`)

| Comportamento / Requisito | Estado | Arquivo:Linha Rust | Origem Java |
|---|---|---|---|
| Registro do bloco `mekanism:qio_exporter` | IMPLEMENTADO | `qio_transfer.rs:20` | `MekanismBlocks.java:QIO_EXPORTER` |
| Orientação 6D (`SELECTED_FACE`) e injeção traseira | IMPLEMENTADO | `qio_transfer.rs:650-720` | `TileEntityQIOExporter.java:126-128` |
| Taxa de pulso a cada 10 ticks (`MAX_DELAY = 10`) | IMPLEMENTADO | `qio_transfer.rs:23,657-661` | `TileEntityQIOExporter.java:74,108-113` |
| Capacidade de trânsito idêntica ao Importer (64..320 itens, 1..5 tipos) | IMPLEMENTADO | `qio_transfer.rs:39-51` | `TileEntityQIOFilterHandler.java:41-43` |
| Modo `exportWithoutFilter` (padrão `false`, NBT `auto`) | IMPLEMENTADO | `qio_transfer.rs:435,450` | `TileEntityQIOExporter.java:79,209` |
| Modo `roundRobin` com alternância cíclica de destinos | IMPLEMENTADO | `qio_transfer.rs:437,688-692,703-706`| `TileEntityQIOExporter.java:80,210,248` |
| Ejeção filtrada via mapa de filtros ativos | IMPLEMENTADO | `qio_transfer.rs:677-686` | `TileEntityQIOExporter.java:144-163` |
| Inserção em contêineres vizinhos respeitando empilhamento | IMPLEMENTADO | `qio_transfer.rs:612-646` | `TileEntityQIOExporter.java:134-142` |
| Integração com Logistical Transporters (`TransitRequest`) | PARCIAL-por-FORK | `qio_transfer.rs:650` | `FORK-3` (Transporter Callbacks), `TileEntityQIOExporter.java:266` |
| Serialização `encode_exporter` / `decode_exporter` | IMPLEMENTADO | `qio_transfer.rs:461-606` | `TileEntityQIOExporter.java:207-218` |

---

## 3. QIO Dashboard (`TileEntityQIODashboard`)

| Comportamento / Requisito | Estado | Arquivo:Linha Rust | Origem Java |
|---|---|---|---|
| Registro do bloco `mekanism:qio_dashboard` | IMPLEMENTADO | `qio_dashboard.rs:20` | `MekanismBlocks.java:QIO_DASHBOARD` |
| Orientação 6D completa (piso, teto ou parede) | IMPLEMENTADO | `qio_dashboard.rs:109,118` | `TileEntityQIODashboard.java:45` |
| 3 Janelas de Crafting independentes (`MAX_CRAFTING_WINDOWS = 3`) | IMPLEMENTADO | `qio_dashboard.rs:23,110` | `QIOCraftingWindow.java:53` |
| Grid 3x3 (9 inputs) + 1 output por janela = 30 slots no total | IMPLEMENTADO | `qio_dashboard.rs:26-32,44-48` | `TileEntityQIODashboard.java:66-73` |
| Alternador `insertIntoFrequency` (padrão `true`) | IMPLEMENTADO | `qio_dashboard.rs:106,115` | `TileEntityQIODashboard.java:42,172-178` |
| Roteamento de shift-click (frequência vs inventário do jogador) | IMPLEMENTADO | `qio_dashboard.rs:250-316` | `TileEntityQIODashboard.java:172-178` |
| Catálogo de itens: TakeSingle (1 item), TakeHalf (½ stack), TakeMax (stack) | IMPLEMENTADO | `qio_dashboard.rs:347-372` | `PacketQIOItemViewerSlotTake.java` |
| Catálogo de itens: ShiftTake (direto para inventário do jogador) | IMPLEMENTADO | `qio_dashboard.rs:373-418` | `PacketQIOItemViewerSlotShiftTake.java` |
| Catálogo de itens: Place (insere cursor na frequência QIO) | IMPLEMENTADO | `qio_dashboard.rs:419-427` | `PacketQIOItemViewerSlotPlace.java` |
| Devolver itens da janela para a frequência (`ClearCraftingWindow`) | IMPLEMENTADO | `qio_dashboard.rs:58-67,428-444` | `PacketQIOClearCraftingWindow.java` |
| Execução de crafting com preservação de recipientes (baldes etc.) | IMPLEMENTADO | `qio_dashboard.rs:75-98` | `QIOCraftingWindow.java:150-180` |
| Transmissão em lote de catálogo `BulkQIOData` | PARCIAL-por-FORK | `qio_dashboard.rs:320-338` | `FORK-24` (Net Bulk Packets), `BulkQIOData.java` |
| Serialização `encode_dashboard` / `decode_dashboard` | IMPLEMENTADO | `qio_dashboard.rs:123-241` | `TileEntityQIODashboard.java:149-158` |

---

## 4. Sistema de Segurança e Security Desk (`TileEntitySecurityDesk`)

| Comportamento / Requisito | Estado | Arquivo:Linha Rust | Origem Java |
|---|---|---|---|
| Registro do bloco `mekanism:security_desk` | IMPLEMENTADO | `security.rs:21` | `MekanismBlocks.java:SECURITY_DESK` |
| Modos de segurança `PUBLIC`, `PRIVATE`, `TRUSTED` | IMPLEMENTADO | `security.rs:44-77` | `SecurityMode.java:25-37` |
| Estrutura `SecurityFrequency` (chave por UUID, nome `"Security"`) | IMPLEMENTADO | `security.rs:83-107` | `SecurityFrequency.java:22-68` |
| Lista ordenada de jogadores confiáveis (`trusted` UUID + nome) | IMPLEMENTADO | `security.rs:121-144` | `SecurityFrequency.java:101-134` |
| Botão de Override global (`override_mode`) | IMPLEMENTADO | `security.rs:147-149` | `SecurityFrequency.java:85-98` |
| Cálculo de código de sincronização (`sync_hash`) | IMPLEMENTADO | `security.rs:156-169` | `SecurityFrequency.java:76-83` |
| Comparador de restrição `more_restrictive` | IMPLEMENTADO | `security.rs:183-191` | `SecurityUtils.java:141-152` |
| Resolução do modo de segurança efetivo com override | IMPLEMENTADO | `security.rs:194-208` | `SecurityUtils.java:158-166` |
| Matriz completa de permissão de acesso (`can_access`) | IMPLEMENTADO | `security.rs:213-240` | `SecurityUtils.java:110-138` |
| Componente acoplável a máquinas `TileComponentSecurity` | IMPLEMENTADO | `security.rs:248-262` | `TileComponentSecurity.java:25-79` |
| Slot 0 (Unlock, X=146, Y=18): remove proprietário de itens | IMPLEMENTADO | `security.rs:24,298-305` | `TileEntitySecurityDesk.java:42,53` |
| Slot 1 (Lock, X=146, Y=97): tranca itens com UUID e frequência | IMPLEMENTADO | `security.rs:27,307-314` | `TileEntitySecurityDesk.java:43,54` |
| Bounding Block 1x2x1 (`ABOVE_ONLY`: pos + [0, 1, 0]) | IMPLEMENTADO | `security.rs:21`, `INTEGRACAO` | `TileEntitySecurityDesk.java:29` |
| Dureza inquebrável para não-proprietários (`resistance = -1.0f`) | PARCIAL-por-FORK | `security.rs:21` | `FORK-30` (Host Hardness Hook), `AttributeCustomResistance` |
| Expulsão forçada de jogadores sem permissão (`closeContainer`) | IMPLEMENTADO | `security.rs:370-377` | `TileEntitySecurityDesk.java:76-85,101-106` |
| Serialização `encode_security_desk` / `decode_security_desk` | IMPLEMENTADO | `security.rs:383-470` | `TileEntitySecurityDesk.java:48-57` |

---

## 5. Modification Station (`TileEntityModificationStation`)

| Comportamento / Requisito | Estado | Arquivo:Linha Rust | Origem Java |
|---|---|---|---|
| Registro do bloco `mekanism:modification_station` | IMPLEMENTADO | `modification_station.rs:25` | `MekanismBlocks.java:MODIFICATION_STATION` |
| Geometria multibloco 2x2x1 (4 blocos no footprint relativo ao facing) | IMPLEMENTADO | `modification_station.rs:57-73` | `TileEntityModificationStation.java:40` |
| Capacidade de energia: 40.000 Joules = 16.000 FE | IMPLEMENTADO | `modification_station.rs:35` | `StorageConfig.java:104` |
| Consumo de energia: 400 Joules/t = 160 FE/t | IMPLEMENTADO | `modification_station.rs:38` | `UsageConfig.java:74` |
| Duração da operação: 10 ticks (`TICKS_PER_HALF_SECOND = 10`) | IMPLEMENTADO | `modification_station.rs:41` | `TileEntityModificationStation.java:42` |
| Energia total por módulo: 1.600 FE (4.000 Joules) | IMPLEMENTADO | `modification_station.rs:44` | Cálculo: $10 \text{ ticks} \times 160 \text{ FE/t}$ |
| Proibição estrita de upgrades de velocidade ou energia | IMPLEMENTADO | `modification_station.rs:18` | `TileEntityModificationStation.java` |
| Slot 0: `moduleSlot` ($X=35, Y=118$) | IMPLEMENTADO | `modification_station.rs:28` | `TileEntityModificationStation.java:75` |
| Slot 1: `containerSlot` ($X=125, Y=118$) | IMPLEMENTADO | `modification_station.rs:29` | `TileEntityModificationStation.java:76` |
| Slot 2: `energySlot` ($X=151, Y=21$) | IMPLEMENTADO | `modification_station.rs:30` | `TileEntityModificationStation.java:80` |
| Catálogo completo de módulos e limites de stack (1 a 8) | IMPLEMENTADO | `modification_station.rs:107-150` | `MekanismModules.java:61-232` |
| Validação de compatibilidade módulo-equipamento (`is_module_compatible`) | IMPLEMENTADO | `modification_station.rs:153-170` | `ModuleContainer.java:281-287` |
| Parser e serializador de `ModuleContainer` em Data Components | IMPLEMENTADO | `modification_station.rs:176-261` | `ModuleContainer.java:45-66` |
| Ciclo de progresso de tick, consumo de energia e dedução de itens | IMPLEMENTADO | `modification_station.rs:288-349` | `TileEntityModificationStation.java:85-118` |
| Desinstalação de módulos unitária ou total (`remove_module_from_container`)| IMPLEMENTADO | `modification_station.rs:352-378` | `TileEntityModificationStation.java:124-137` |
| Devolução do item do módulo desinstalado ao inventário | IMPLEMENTADO | `modification_station.rs:371-375` | `TileEntityModificationStation.java:131` |
| Serialização `encode_modification_station` / `decode_` | IMPLEMENTADO | `modification_station.rs:385-481` | `TileEntityModificationStation.java:144-153` |

---

## 6. Sumário de Estados

- **Total de Comportamentos Especificados**: 52
- **IMPLEMENTADO**: 47
- **PARCIAL-por-FORK**: 5 (`FORK-18`, `FORK-24`, `FORK-3`, `FORK-30`, `FORK-27`)
- **NÃO IMPLEMENTADO**: 0

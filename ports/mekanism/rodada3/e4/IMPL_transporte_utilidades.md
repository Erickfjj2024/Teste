# Matriz de Rastreabilidade de Implementação: Grupo Transporte e Utilidades

**ID da Tarefa**: T20261109-204014-32i  
**Grupo**: transporte_utilidades  
**Data**: 2026-11-09  

Legenda de Estado:
- **IMPLEMENTADO**: Lógica completa e fiel em Rust, testada unitariamente com valores do Java.
- **PARCIAL-por-FORK**: Lógica em Rust concluída; chamada ao host isolada e marcada com `// FORK-n`, aguardando implementação da extensão WIT no fork do Horizon.
- **NAO**: Não implementado.

---

## 1. Restrictive Transporter

| Comportamento da Especificação | Origem no Java | Implementação no Rust | Estado | Detalhes |
|---|---|---|---|---|
| Registro `mekanism:restrictive_transporter` | `MekanismBlocks.java` | `transporter_special.rs:25` | IMPLEMENTADO | Constante `RESTRICTIVE_TRANSPORTER` |
| Custo de roteamento A* (1.000) | `RestrictiveTransporter.java:13-15` | `transporter_special.rs:29, 219` | IMPLEMENTADO | `RESTRICTIVE_COST = 1_000.0`, testado em `test_restrictive_cost_and_tier_constants` |
| Tier Basic (50 ticks / 10 ticks delay) | `RestrictiveTransporter.java:9`, `TransporterTier.java` | `transporter_special.rs:34-36` | IMPLEMENTADO | `BASIC_SPEED = 50`, `BASIC_PULL_DELAY = 10` |
| Conexões espaciais de 6 faces | `LogisticalTransporterBase.java` | `transporter_special.rs:77-83` | IMPLEMENTADO | `RestrictiveState.connections` com 6 direções |
| Serialização e desserialização de conexões | `TileEntityRestrictiveTransporter.java` | `transporter_special.rs:85-100` | IMPLEMENTADO | `encode_restrictive` e `decode_restrictive` |
| Algoritmo de roteamento A* (prioridade mínima) | `TransporterPathfinder.java:420-470` | `transporter_special.rs:179-247` | IMPLEMENTADO | `find_path()` puro com min-heap, testado em `test_astar_prefers_normal_over_restrictive` |
| Configuração in-world com Configurador | `TileEntityTransmitter.java` | `transporter_special.rs:268-290` | IMPLEMENTADO | Sneak-clique cicla tipo de conexão da face clicada |
| Roteamento dinâmico pelo host do Horizon | Host routing engine | `transporter_special.rs:252-258` | PARCIAL-por-FORK | Isolado em `route_transporter` sob `FORK-3` |
| Receita Shaped `S#S` (aço + barras de ferro -> 2) | `restrictive_transporter.json` | `INTEGRACAO_transporte_utilidades.md` | IMPLEMENTADO | Documentado e pronto para o `convert.py` |

---

## 2. Diversion Transporter

| Comportamento da Especificação | Origem no Java | Implementação no Rust | Estado | Detalhes |
|---|---|---|---|---|
| Registro `mekanism:diversion_transporter` | `MekanismBlocks.java` | `transporter_special.rs:26` | IMPLEMENTADO | Constante `DIVERSION_TRANSPORTER` |
| Modos `DISABLED`, `HIGH`, `LOW` por face | `DiversionTransporter.java:194-217` | `transporter_special.rs:40-73` | IMPLEMENTADO | Enum `DiversionControl`, testado em `test_diversion_control_cycling` |
| Alternância de modo ao clicar na face | `DiversionTransporter.java:157-163` | `transporter_special.rs:52-58, 303-308` | IMPLEMENTADO | Ciclo Disabled -> High -> Low -> Disabled e envio de feedback |
| Regras de ativação por sinal de redstone | `DiversionTransporter.java:182-187` | `transporter_special.rs:67-73` | IMPLEMENTADO | `mode_reqs_met()` validado em `test_diversion_control_mode_reqs` |
| Bloqueio de inserção e desconexão quando desativado | `DiversionTransporter.java:166-174` | `transporter_special.rs:136-146, 311-316` | IMPLEMENTADO | `can_connect_side()` e `transmitter_faces` filtrados |
| Serialização e desserialização de modos e conexões | `DiversionTransporter.java:85-102` | `transporter_special.rs:114-138` | IMPLEMENTADO | `encode_diversion` e `decode_diversion` com 13 bytes |
| Teste de bloqueio de rota no pathfinder | `TransporterPathfinder.java` | `transporter_special.rs:420-449` | IMPLEMENTADO | Testado em `test_diversion_transporter_closes_route_when_unpowered` |
| Integração com roteamento dinâmico do Horizon | Host network engine | `transporter_special.rs:252-258` | PARCIAL-por-FORK | `route_transporter` sob `FORK-3` |
| Receita Shaped `RRR / S#S / RRR` -> 2 | `diversion_transporter.json` | `INTEGRACAO_transporte_utilidades.md` | IMPLEMENTADO | Documentado para o `convert.py` |

---

## 3. Chargepad

| Comportamento da Especificação | Origem no Java | Implementação no Rust | Estado | Detalhes |
|---|---|---|---|---|
| Registro `mekanism:chargepad` | `MekanismBlocks.java` | `chargepad.rs:20` | IMPLEMENTADO | Constante `BLOCK` |
| Shape de colisão de placa fina | `BlockChargepad.java:19` | `chargepad.rs:48` | IMPLEMENTADO | `AABB_BOUNDS = [0.0, 0.0, 0.0, 1.0, 0.4, 1.0]` |
| Capacidade do buffer (2.048.000 J = 819.200 FE) | `StorageConfig.java:39` | `chargepad.rs:23` | IMPLEMENTADO | `CAPACITY = fe(2_048_000)` = 819.200 FE |
| Taxa máxima de carga (1.024.000 J/t = 409.600 FE/t) | `UsageConfig.java:42` | `chargepad.rs:26` | IMPLEMENTADO | `CHARGE_RATE = fe(1_024_000)` = 409.600 FE/t |
| Entradas de energia (RelativeSide BACK e BOTTOM) | `TileEntityChargepad.java:48` | `chargepad.rs:36-45` | IMPLEMENTADO | `energy_input_faces()`, testado em `test_energy_input_faces` |
| Detecção de jogadores e Robits na AABB | `TileEntityChargepad.java:36, 58-59` | `chargepad.rs:50-70` | IMPLEMENTADO | Enum `ChargeTarget` e predicado de elegibilidade |
| Recarga de 1 item por jogador por tick até taxa máxima | `TileEntityChargepad.java:90-96` | `chargepad.rs:114-142` | IMPLEMENTADO | `step_charging()`, testado em `test_charge_single_item_per_player_tick` |
| Recarga direta de entidade Robit | `TileEntityChargepad.java:74` | `chargepad.rs:143-155` | IMPLEMENTADO | Testado em `test_charge_robit` |
| Sons de placa de pressão na ativação/desativação | `TileEntityChargepad.java:128-133` | `chargepad.rs:29-33, 158-166` | IMPLEMENTADO | `test_pressure_plate_sound_triggers` (pitch 0.8 e 0.7) |
| Busca de entidades vivas no mundo e recarga | Host world / entities | `chargepad.rs:177-187` | PARCIAL-por-FORK | Isolado em `scan_and_charge_entities` sob `FORK-30` |
| Receita Shaped `### / SES` | `chargepad.json` | `INTEGRACAO_transporte_utilidades.md` | IMPLEMENTADO | Documentado para o `convert.py` |

---

## 4. Dimensional Stabilizer

| Comportamento da Especificação | Origem no Java | Implementação no Rust | Estado | Detalhes |
|---|---|---|---|---|
| Registro `mekanism:dimensional_stabilizer` | `MekanismBlocks.java` | `dimensional_stabilizer.rs:17` | IMPLEMENTADO | Constante `BLOCK` |
| Matriz 5×5 de chunks (raio 2, diâmetro 5) | `TileEntityDimensionalStabilizer.java:46-47` | `dimensional_stabilizer.rs:19-21` | IMPLEMENTADO | `MAX_LOAD_RADIUS = 2`, `MAX_LOAD_DIAMETER = 5` |
| Chunk central [2][2] sempre mantido carregado | `TileEntityDimensionalStabilizer.java:68, 148` | `dimensional_stabilizer.rs:52, 69-72` | IMPLEMENTADO | Imutabilidade validada em `test_center_chunk_always_loaded` |
| Consumo base por chunk (5.000 J/t = 2.000 FE/t) | `UsageConfig.java:dimensionalStabilizer` | `dimensional_stabilizer.rs:24` | IMPLEMENTADO | `BASE_ENERGY_PER_CHUNK = fe(5_000)` = 2.000 FE/t |
| Capacidade base do buffer (40.000 J = 16.000 FE) | `StorageConfig.java:dimensionalStabilizer` | `dimensional_stabilizer.rs:27` | IMPLEMENTADO | `BASE_CAPACITY = fe(40_000)` = 16.000 FE |
| Escala de upgrades de energia (até 8 upgrades) | `TileEntityDimensionalStabilizer.java:75` | `dimensional_stabilizer.rs:108-117` | IMPLEMENTADO | Integra com `upgrades::max_energy` e `energy_per_tick` |
| Botões de controle da GUI (raio 1, raio 2, all) | `GuiDimensionalStabilizer.java` | `dimensional_stabilizer.rs:33-37, 85-106` | IMPLEMENTADO | `set_radius()`, testado em `test_radius_shortcuts` |
| Emissão de redstone analógica (15 ativo, 0 inativo) | `TileEntityDimensionalStabilizer.java:186` | `dimensional_stabilizer.rs:119-122` | IMPLEMENTADO | Testado em `test_redstone_emission` |
| Conversão de blocos para coordenadas ChunkPos | `TileEntityDimensionalStabilizer.java:170-182` | `dimensional_stabilizer.rs:124-138` | IMPLEMENTADO | Testado em `test_chunkpos_calculation` |
| Serialização da matriz de 25 chunks | `TileEntityDimensionalStabilizer.java:219-245` | `dimensional_stabilizer.rs:141-187` | IMPLEMENTADO | `test_state_serialization_roundtrip` |
| Carregamento forçado de chunks no servidor | Host chunk engine | `dimensional_stabilizer.rs:191-197` | PARCIAL-por-FORK | Isolado em `sync_chunk_tickets` sob `FORK-14` |
| Receita Shaped `ICI / ADA / ICI` | `dimensional_stabilizer.json` | `INTEGRACAO_transporte_utilidades.md` | IMPLEMENTADO | Documentado para o `convert.py` |

---

## 5. Cardboard Box

| Comportamento da Especificação | Origem no Java | Implementação no Rust | Estado | Detalhes |
|---|---|---|---|---|
| Registro `mekanism:cardboard_box` | `MekanismBlocks.java` | `cardboard_box.rs:21` | IMPLEMENTADO | Constante `BLOCK` |
| Propriedades do bloco (dureza 0.5, resistência 0.6) | `BlockCardboardBox.java:34` | `INTEGRACAO_transporte_utilidades.md` | IMPLEMENTADO | Configurado no bloco do `convert.py` |
| Limite de empilhamento (16 vazia, 1 cheia) | `ItemBlockCardboardBox.java:40` | `cardboard_box.rs:22-23` | IMPLEMENTADO | `EMPTY_STACK_LIMIT = 16`, `FILLED_STACK_LIMIT = 1` |
| Regras de validação para empacotamento | `ItemBlockCardboardBox.java:76-85` | `cardboard_box.rs:48-67` | IMPLEMENTADO | `can_pack_block()`, testado em `test_can_pack_block_validations` |
| Captura de BlockState, propriedades e NBT | `ItemBlockCardboardBox.java:93`, `BlockData.java` | `cardboard_box.rs:26-39, 70-176` | IMPLEMENTADO | Estrutura `PackedBlockData` e serialização |
| Desempacotar (shift + clique direito sem item) | `BlockCardboardBox.java:39-58` | `cardboard_box.rs:242-261` | IMPLEMENTADO | Restauração do bloco e drop de caixa vazia |
| Drop de item de caixa vazia | `BlockCardboardBox.java:53` | `cardboard_box.rs:212-223, 252` | IMPLEMENTADO | Função `drop_item()` usando `entities::summon` |
| API host de captura/reinstanciação genérica | Host level engine | `cardboard_box.rs:180-200` | PARCIAL-por-FORK | Isolado em `host_box_block` / `host_unbox_block` sob `FORK-23` |
| Receita 2×2 sawdust -> 1 | `cardboard_box.json` | `INTEGRACAO_transporte_utilidades.md` | IMPLEMENTADO | Documentado para o `convert.py` |

---

## 6. Personal Storage (Personal Barrel & Personal Chest)

| Comportamento da Especificação | Origem no Java | Implementação no Rust | Estado | Detalhes |
|---|---|---|---|---|
| Registros `personal_chest` e `personal_barrel` | `MekanismBlocks.java` | `personal_storage.rs:18-19` | IMPLEMENTADO | Constantes `CHEST_BLOCK` e `BARREL_BLOCK` |
| Capacidade de 54 slots (6 linhas × 9 colunas) | `PersonalStorageManager.java:150-154` | `personal_storage.rs:22-24` | IMPLEMENTADO | `TOTAL_SLOTS = 54`, testado em `test_slot_count` |
| Modos de segurança PUBLIC, PRIVATE, TRUSTED | `SecurityMode.java` | `personal_storage.rs:32-53` | IMPLEMENTADO | Enum `SecurityMode` |
| Regras de acesso para jogadores | `TileEntityPersonalStorage.java:72-74` | `personal_storage.rs:125-144` | IMPLEMENTADO | `can_player_access()`, testado em `test_security_rules_for_players` |
| Bloqueio de automação quando não for PUBLIC | `TileEntityPersonalStorage.java:73` | `personal_storage.rs:146-149` | IMPLEMENTADO | `can_automation_access()`, testado em `test_automation_access_rules` |
| Baú bloqueado se bloco acima for sólido | `TileEntityPersonalChest.java:66-69` | `personal_storage.rs:152-154` | IMPLEMENTADO | Testado em `test_chest_blocked_by_ceiling` |
| Sons de abertura e fechamento para baú e barril | `TileEntityPersonalBarrel.java:24`, `Chest:28` | `personal_storage.rs:27-30, 157-164` | IMPLEMENTADO | Testado em `test_sound_selection` |
| Irritação de Piglins próximos ao abrir | `TileEntityPersonalStorage.java:110` | `personal_storage.rs:172-174, 222` | IMPLEMENTADO | `anger_nearby_piglins()` |
| Persistência total de itens ao quebrar | `TileEntityPersonalStorage.java:136` | `personal_storage.rs:56-122` | IMPLEMENTADO | Serialização do estado e slots |
| Abertura portátil do inventário na mão | `ItemBlockPersonalStorage.java:43-67` | `personal_storage.rs:167-170` | PARCIAL-por-FORK | Isolado em `open_portable_storage_gui` sob `FORK-7` / `FORK-23` |
| Receitas shaped para baú e barril | `personal_chest.json`, `barrel.json` | `INTEGRACAO_transporte_utilidades.md` | IMPLEMENTADO | Documentado para o `convert.py` |

---

## 7. Seismic Vibrator

| Comportamento da Especificação | Origem no Java | Implementação no Rust | Estado | Detalhes |
|---|---|---|---|---|
| Registro `mekanism:seismic_vibrator` | `MekanismBlocks.java` | `seismic_vibrator.rs:21` | IMPLEMENTADO | Constante `BLOCK` |
| Bounding block vertical 1×2×1 | `TileEntitySeismicVibrator.java:31`, `bounding.rs` | `bounding.rs:22` | IMPLEMENTADO | Já registrado em `bounding.rs:22` como `([0, 1, 0], "none")` |
| Capacidade do buffer (20.000 J = 8.000 FE) | `StorageConfig.java:40` | `seismic_vibrator.rs:24` | IMPLEMENTADO | `CAPACITY = fe(20_000)` = 8.000 FE |
| Consumo de energia (50 J/t = 20 FE/t) | `UsageConfig.java:43` | `seismic_vibrator.rs:27` | IMPLEMENTADO | `USAGE = fe(50)` = 20 FE/t |
| Entrada de energia exclusiva pela traseira (BACK) | `TileEntitySeismicVibrator.java:48` | `seismic_vibrator.rs:37-44` | IMPLEMENTADO | `energy_input_faces()`, testado em `test_energy_input_face_back_only` |
| Intervalo de vibração de 40 ticks (2 segundos) | `TileEntitySeismicVibrator.java:78` | `seismic_vibrator.rs:30, 151` | IMPLEMENTADO | `VIBRATION_INTERVAL = 40`, testado em `test_seismic_vibrator_energy_and_timing` |
| Ticking do pistão mecânico (`clientPiston++`) | `TileEntitySeismicVibrator.java:64` | `seismic_vibrator.rs:148` | IMPLEMENTADO | `state.client_piston.wrapping_add(1)` |
| Validação de coordenadas relativas do chunk (0..15) | `TileEntitySeismicVibrator.java:121-130` | `seismic_vibrator.rs:88-103` | IMPLEMENTADO | `get_vertical_pos()`, testado em `test_chunk_relative_coordinates_validation` |
| Disparo de GameEvent `SEISMIC_VIBRATION` no host | Host game event engine | `seismic_vibrator.rs:106-110` | PARCIAL-por-FORK | Isolado em `emit_seismic_vibration` sob `FORK-31` |
| Receita Shaped `I#I / CXC / III` | `seismic_vibrator.json` | `INTEGRACAO_transporte_utilidades.md` | IMPLEMENTADO | Documentado para o `convert.py` |

---

## 8. Oredictionificator

| Comportamento da Especificação | Origem no Java | Implementação no Rust | Estado | Detalhes |
|---|---|---|---|---|
| Registro `mekanism:oredictionificator` | `MekanismBlocks.java` | `oredictionificator.rs:18` | IMPLEMENTADO | Constante `BLOCK` |
| Consumo elétrico 0 (passivo) | `TileEntityOredictionificator.java` | `oredictionificator.rs:242-243` | IMPLEMENTADO | Não consome energia do buffer |
| Slots de inventário (entrada 0, saída 1) | `TileEntityOredictionificator.java:68-69` | `oredictionificator.rs:20-22` | IMPLEMENTADO | `INPUT_SLOT = 0`, `OUTPUT_SLOT = 1`, testado em `test_slots_and_prefixes` |
| Prefixos de tags válidos | `MekanismConfig.general.validOredictionificatorFilters` | `oredictionificator.rs:25-33, 49-60` | IMPLEMENTADO | `VALID_PREFIXES`, validado em `test_slots_and_prefixes` |
| Configuração de lados (saída em RelativeSide.RIGHT) | `TileEntityOredictionificator.java:57` | `oredictionificator.rs:136-159` | IMPLEMENTADO | `default_item_faces()` mapeando face direita |
| Conversão de 1 item por tick para item designado | `TileEntityOredictionificator.java:82-96` | `oredictionificator.rs:162-211` | IMPLEMENTADO | `step_conversion()`, testado em `test_step_conversion_into_empty_output` e `test_step_conversion_stacks_in_output` |
| Suporte a controle de redstone | `TileEntityOredictionificator.java:182` | `oredictionificator.rs:228-230` | IMPLEMENTADO | Respeita modos Disabled, High, Low |
| Filtros desativados são ignorados na conversão | `TileEntityOredictionificator.java:83` | `oredictionificator.rs:175-177` | IMPLEMENTADO | Testado em `test_disabled_filter_is_ignored` |
| Serialização e desserialização de filtros | `TileEntityOredictionificator.java:170-179` | `oredictionificator.rs:72-133` | IMPLEMENTADO | `test_state_serialization_roundtrip` |
| Inspeção de tags de itens em tempo de execução | Host item tag registry | `oredictionificator.rs:214-219` | PARCIAL-por-FORK | Isolado em `host_get_item_tags` sob `FORK-32` |
| Receita Shaped `SGS / CPC / S#S` | `oredictionificator.json` | `INTEGRACAO_transporte_utilidades.md` | IMPLEMENTADO | Documentado para o `convert.py` |

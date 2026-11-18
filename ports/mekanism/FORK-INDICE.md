# FORK-INDICE — pontos de fork no código / fork points in the code

> Gerado de `mod/src/fork_*.rs`. A **chave estável é o nome da função**: cada função existe duas vezes, com `#[cfg(feature = "fork")]` (chama o host real) e sem (devolve `Err("FORK-n pendente")`). / Generated from `mod/src/fork_*.rs`. The stable key is the function name.
> Os números `FORK-n` foram dados por contas diferentes e **colidem** com os de `API-REQUISITOS.md` (a última coluna mostra o que o mesmo número significa lá). Ao implementar no fork, siga o nome da função e a assinatura WIT proposta em cada `FORK_*.md`. / The `FORK-n` numbers were given by different accounts and collide with `API-REQUISITOS.md`.

| Arquivo / File | Função / Function | Nº no código / Number in code | Descrição / Description | Significado do mesmo nº em API-REQUISITOS / Same number there |
|---|---|---|---|---|
| `fork_equipamento.rs` | `open_radial_menu` | FORK-27 | Radial menu for modular equipment (Meka-Tool, MekaSuit) | Equipamentos e armaduras com módulos (MekaSuit, Meka-Tool) e inventári |
| `fork_equipamento.rs` | `execute_area_mining` | FORK-31 | Multi-block area mining (Blasting and Vein Mining units) | Renderização de fluido/químico dentro de um volume ligada a uma estrut |
| `fork_equipamento.rs` | `execute_teleport` | FORK-32 | Raycast teleportation with portal FX | Consulta do tempo de queima de qualquer item (registro de combustíveis |
| `fork_equipamento.rs` | `on_arrow_shoot` | FORK-33 | Bow shooting hooks and projectile customization | Vários grupos de multibloco por bloco (vidro estrutural, pressure disp |
| `fork_equipamento.rs` | `tick_elytra_flight` | FORK-34 | Elytra flight tick physics and wear hook | Mascarar capacidades de um bloco por estado (tanques/energia/itens fec |
| `fork_equipamento.rs` | `read_stack_components` | FORK-23 | Data components read/write on player equipment | Componentes de dados em itens/blocos (158 usos) e anexos de entidade ( |
| `fork_equipamento.rs` | `write_stack_components` | FORK-23 | pendente".into()) | Componentes de dados em itens/blocos (158 usos) e anexos de entidade ( |
| `fork_equipamento.rs` | `apply_vector_impulse` | FORK-29 | 3D vector movement impulse and flight ability | Uso de bloco **condicional pelo mod**: o mod decide por clique consumi |
| `fork_equipamento.rs` | `set_creative_flight` | FORK-29 | pendente".into()) | Uso de bloco **condicional pelo mod**: o mod decide por clique consumi |
| `fork_equipamento.rs` | `update_hud` | FORK-30 | Dynamic HUD element rendering & tile capability query | `world.spawn-particles` para mods (tipo, posição, velocidade, quantida |
| `fork_equipamento.rs` | `query_vertical_chain` | FORK-30 | pendente".into()) | `world.spawn-particles` para mods (tipo, posição, velocidade, quantida |
| `fork_equipamento.rs` | `spawn_robit_entity` | FORK-10 | Spawning Robit entity in world | Entidades próprias (Robit, projéteis) com IA, inventário e renderizaçã |
| `fork_equipamento.rs` | `send_entity_button` | FORK-24 | Entity C2S/S2C button packets | Pacotes de rede entre cliente e mod (115 classes) / client↔mod packets |
| `fork_equipamento.rs` | `register_chemical_fluid` | FORK-26 | Registering custom chemical fluids | Químicos (gás, infusão, pigmento, lodo) como registro de primeira clas |
| `fork_equipamento.rs` | `register_consumable_bucket` | FORK-29 | Registering consumable bucket items | Uso de bloco **condicional pelo mod**: o mod decide por clique consumi |
| `fork_equipamento.rs` | `dump_radiation` | FORK-13 | Dumping environmental radiation | Radiação por chunk com raio de varredura de 5 chunks (80 blocos) e son |
| `fork_qio_calor.rs` | `emit_heat` | FORK-4 | Emissão térmica de primeira classe para blocos adjacentes. | **Meio "calor"** de primeira classe: capacidade, condução inversa, iso |
| `fork_qio_calor.rs` | `laser_entity_damage` | FORK-7 | Dano do feixe laser e interação térmica com entidades. | Dano a entidades e queima por mod (laser: 2 500 J por ½ coração; SPS:  |
| `fork_qio_calor.rs` | `pop_resource_from_face` | FORK-8 | Ejeção direcional de itens excedentes no mundo pelo Laser Tractor Beam. | Escavação de bloco por energia segundo a dureza (laser: 100 000 J por  |
| `fork_qio_calor.rs` | `lookup_frequency` | FORK-18 | Persistência e busca de frequências nomeadas por UUID e modo de segurança. | Frequências **nomeadas, públicas/privadas, por UUID** (hoje 1–99) / na |
| `fork_qio_calor.rs` | `open_qio_screen` | FORK-20 | Abertura de GUI enriquecida com widgets customizados (QIO Dashboard, Security Desk). | Telas com o aspecto original: atlas de GUI, tooltips, abas, botões de  |
| `fork_qio_calor.rs` | `set_redstone_output` | FORK-21 | Emissão de sinal redstone por face para blocos adjacentes. | Modos de redstone por bloco (OFF / detecção de entidade / conteúdo de  |
| `fork_qio_calor.rs` | `broadcast_bulk_qio_data` | FORK-24 | Sincronização em lote cliente-servidor para dados do catálogo virtual QIO (`BulkQIOData`). | Pacotes de rede entre cliente e mod (115 classes) / client↔mod packets |
| `fork_utilidades.rs` | `set_block` | FORK-23 | Sets a block in the world directly. | Componentes de dados em itens/blocos (158 usos) e anexos de entidade ( |
| `fork_utilidades.rs` | `has_neighbor_signal` | FORK-35 | Queries whether a position receives a redstone signal from any neighbor block. | Bounding blocks com clique, quebra e colocação ligados nativamente à m |

Com a feature `fork` ligada, o verificador acusa como ausentes do SDK atual: `world::set_block` e `world::has_neighbor_signal` (as demais funções acima são pontos de extensão sem chamada real escrita ainda). / With the `fork` feature on, the checker reports as missing from the current SDK: `world::set_block` and `world::has_neighbor_signal`.

# REVISÃO — Configuration Card / Configuration Card review

**Fontes / sources:** `ItemConfigurationCard`, `IConfigCardAccess`, `TileEntityMekanism`, `TileEntityConfigurableMachine`, `TileEntityResistiveHeater` e os seis `isConfigurationDataCompatible` (1.21.1-10.7.19.85) × `configuration_card.rs`, `equipment_world.rs`.

## Corrigido / Fixed
1. **Compatibilidade.** O Java compara o bloco e, só em fábricas e nas 6 máquinas que têm fábrica (elétricas, combiner, infuser, serra), o tipo de fábrica. O port aceitava também *tiers* de cubo de energia, tanques de fluido e de químico, caixas e transmissores. Removido (teste).
2. **Dados no lugar certo.** O caminho de máquinas usava a configuração `Layout::Electric` e o controle no byte 37 para toda máquina; o controle fica no byte 37 só nas elétricas/processo/serra/fábrica e no 39 nas contínuas, reação e bomba (`redstone::offset`), e cada máquina decodifica a configuração com o seu `Layout`. Agora usa a definição de cada máquina (`MACHINES`); teste de ida e volta em duas famílias (offsets 37 e 39).
3. **Resistive Heater:** o Java guarda também o uso de energia (`ENERGY_USAGE`); agora controle + uso na ida e na volta.
4. **Nomes nas mensagens:** mostrava o id (`enrichment_chamber`) ou a chave de tradução; agora o nome em inglês (`Enrichment Chamber`), que no Mekanism é o id em título.
5. **Onde age:** só em blocos com a capacidade (assemblicator, aquecedor resistivo, máquinas configuráveis); nos outros o mod não toma o clique.
6. **Pasta o clique no tick da máquina:** copiar lê a máquina; colar reescreve os dados da máquina, no tick dela (um uso não altera um bloco).

## Aberto / Open
- Mensagens vão ao bate-papo; o Java usa a barra de ação (`tell-overlay`, pedido em `PROPOSTA-API.wit`).
- Segurança (`canAccessOrDisplayError`): o `security.rs` existe; a ligação com o dono do bloco colocado falta.
- Frequências (`writeConfiguredFrequencies`): Teleporter, Entangloporter, QIO ainda não passam pelo cartão.
- Cores do ejetor e `strictInput`: o port não tem cores de transportador; os campos do cartão existem mas ficam vazios.
- Outras máquinas com dados próprios no cartão (Digital Miner, filtros, Fuelwood Heater): não ligadas.
- Dificuldade de fidelidade total do formato: o Java guarda o `CompoundTag` do componente `configuration_data`; o port guarda JSON próprio no componente `mekanism:configuration_data` (não é lido por outros mods).

**EN.** Fixed: compatibility (only the same block and the factory families), per-machine layout and control offsets, the resistive heater's energy usage, English names in messages, where the card acts. Open: action-bar messages, security hook, frequencies, ejector colours, machines with filters, the data format.

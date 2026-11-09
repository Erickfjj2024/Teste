# API-REQUISITOS — o que o Mekanism exige da API do Horizon / What Mekanism demands from the Horizon API

> **PT-BR primeiro, English em cada linha / PT-BR first, English on each row.**
> Regra do projeto: **não existem "limites do Horizon"**. Tudo o que a auditoria marcou como LIMITE vira um requisito do fork (`FORK-n`) e, quando for genérico, um requisito da **API pública para outros desenvolvedores** (coluna *API pública*).
> Project rule: **there are no "Horizon limits"**. Whatever the audit marked LIMITE becomes a fork requirement (`FORK-n`) and, when generic, a requirement of the **public API for other developers** (column *Public API*).
>
> Fonte / Source: `auditoria/AUDITORIA_*.md` (rodada 1 / round 1), `API-SURVEY.md`, `DIFERENCAS.md` seção 5. Estado / State: **pedido / requested** (nada implementado no fork ainda / nothing implemented in the fork yet), exceto onde indicado / unless noted.

## A. Transmissão e redes / Transmission and networks

| ID | Requisito / Requirement | Origem no Java / Java source | API pública / Public API |
|---|---|---|---|
| FORK-1 | Capacidade por tier de cada transmissor, para cada meio (energia, fluido, químico, item, calor), declarada no mod / per-tier capacity of every transmitter, for each medium | `TubeTier`, `PipeTier`, `CableTier` | `TransmitterSpec { capacity, throughput }` |
| FORK-2 | Velocidade e atraso do transporte de itens por tier (5/10/20/50 ticks) / item transport speed and delay per tier | `TransporterTier` | `TransmitterSpec.speed` |
| FORK-3 | Transportadores com lógica própria: Restrictive (custo 1000) e Diversion (modos de redstone por face) / transporters with own logic | `RestrictiveTransporter`, `DiversionTransporter` | callbacks de roteamento / routing callbacks |
| FORK-4 | **Meio "calor"** de primeira classe: capacidade, condução inversa, isolamento (10 / 400 / 8 000 / 100 000), rede de condutores / first-class **heat** medium | `ConductorTier`, `HeatAPI` | `Medium::Heat` |
| FORK-5 | Capacidades "criativas" ilimitadas (`Long.MAX_VALUE`, `Integer.MAX_VALUE`) sem teto artificial / unbounded creative capacities | `EnergyCubeTier`, `BinTier` | tipo `u128`/ilimitado / unbounded type |
| FORK-6 | Item, fluido, químico e energia em **uma só frequência** (Entangloporter), com calor / one frequency for all media, with heat | `InventoryFrequency` | `Frequency { owner, public, media[] }` |

## B. Entidades, dano e física / Entities, damage and physics

| ID | Requisito / Requirement | Origem / Source |
|---|---|---|
| FORK-7 | Dano a entidades e queima por mod (laser: 2 500 J por ½ coração; SPS: `energia/1000` por tick) / mod-driven entity damage and burning | `GeneralConfig.laserEnergyPerDamage`, `SPSMultiblockData:227` |
| FORK-8 | Escavação de bloco por energia segundo a dureza (laser: 100 000 J por dureza) / energy-driven block breaking by hardness | `GeneralConfig.laserEnergyPerHardness` |
| FORK-9 | Teleportar **qualquer entidade** com passageiros, custo multiplicado / teleport any entity with passengers | `TileEntityTeleporter:283,452-460` |
| FORK-10 | Entidades próprias (Robit, projéteis) com IA, inventário e renderização / custom entities (Robit, projectiles) | `entities/*` |
| FORK-11 | Tipos de dano e efeitos de poção próprios, radiação como atributo / custom damage types, effects, radiation as attribute | `damage_type`, `MekanismMobEffects` (conferir / verify) |

## C. Mundo / World

| ID | Requisito / Requirement | Origem / Source |
|---|---|---|
| FORK-12 | Worldgen com alturas relativas (`belowTop(-24)`, `aboveBottom(-24)`), platô de trapézio e descarte por ar / relative heights, trapezoid plateau and air discard | `OreType` (osmium upper: Y 72–343 platô 8; urânio enterrado: –88…–8 platô 16, descarte 0,75) |
| FORK-13 | Radiação por chunk com raio de varredura de 5 chunks (80 blocos) e sons do Geiger / per-chunk radiation with scan radius and Geiger sounds | `RadiationManager`, `RadiationScale` |
| FORK-14 | Carregamento de chunks por mod (Teleporter, Miner) / mod chunk loading | `StabilizedChunks` |
| FORK-15 | Varredura **assíncrona** (thread) para o Digital Miner / async scan thread | `ThreadMinerSearch` |

## D. Interface e renderização / UI and rendering

| ID | Requisito / Requirement | Origem / Source |
|---|---|---|
| FORK-16 | Filtros **ilimitados** (Digital Miner, Sorter) em tela com lista rolável; hoje há 5 slots físicos / unlimited filters in a scrolling list | `MinerFilter`, `SorterFilter` (conferir / verify: `SorterTagFilter` etc.) |
| FORK-17 | Sorter completo: round-robin, cor padrão, cor por filtro, mín/máx, redstone / full sorter | `TileEntityLogisticalSorter` |
| FORK-18 | Frequências **nomeadas, públicas/privadas, por UUID** (hoje 1–99) / named, public/private, UUID frequencies | `InventoryFrequency:158` |
| FORK-19 | Renderizadores de bloco/entidade, carregadores de modelo, shaders (cubo de energia, alarme com feixe e som, caixa do miner) / block/entity renderers, model loaders, shaders | `client/render/*` (174 classes) |
| FORK-20 | Telas com o aspecto original: atlas de GUI, tooltips, abas, botões de imagem / original-looking screens | `client/gui/*` (354 classes) |
| FORK-21 | Modos de redstone por bloco (OFF / detecção de entidade / conteúdo de energia) / per-block redstone output modes | `TileEntityLaserAmplifier:368` |
| FORK-22 | Água pesada: extração pela bomba com Filter Upgrade (10 mB/balde) / heavy water extraction | `GeneralConfig.pumpHeavyWaterAmount` |

## E. Dados e itens / Data and items

| ID | Requisito / Requirement | Origem / Source |
|---|---|---|
| FORK-23 | Componentes de dados em itens/blocos (158 usos) e anexos de entidade (74 classes) / item data components and attachments | `MekanismDataComponents`, `attachments/*` |
| FORK-24 | Pacotes de rede entre cliente e mod (115 classes) / client↔mod packets | `common/network/*` |
| FORK-25 | Receitas com tipos próprios (70) com ingredientes de químico e saídas múltiplas / custom recipe types with chemical ingredients | `api/recipes/*` |
| FORK-26 | Químicos (gás, infusão, pigmento, lodo) como registro de primeira classe, com tint / chemicals as a first-class registry | `MekanismChemicals` |
| FORK-27 | Equipamentos e armaduras com módulos (MekaSuit, Meka-Tool) e inventário de módulos / equipment with modules | `gear/*` |
| FORK-28 | Integrações (JEI/EMI/CC, `integration/*` – 190 classes) como pontos de extensão / integrations as extension points | `common/integration/*` |

| FORK-29 | Uso de bloco **condicional pelo mod**: o mod decide por clique consumir ou passar (GUI só se formado e sem agachar; senão colocar o bloco da mão) / mod-decided conditional block use | `TileEntityMultiblock.onActivate:218-229` (ver `SINTOMAS_MULTIBLOCOS.md` S1) |
| FORK-30 | `world.spawn-particles` para mods (tipo, posição, velocidade, quantidade, jogadores a até N blocos), p.ex. as faíscas de formação / mod particles | `TileEntityMultiblock.doMultiblockSparkle:297-312`, `SparkleAnimation` (S2) |
| FORK-31 | Renderização de fluido/químico dentro de um volume ligada a uma estrutura (caixa, textura, cor, fração de altura) / fluid fill rendering in a volume | `MultiblockTileEntityRenderer` e `Render*` dos multiblocos (S4; detalha o FORK-19) |

## F. Como este arquivo vira a API pública / How this file becomes the public API

**PT.** Cada `FORK-n` generalizável (A, C, E e parte de D) é escrito como **capacidade nomeada** do SDK (`horizon-mod-sdk`) com: contrato, exemplo mínimo, teste de conformidade. O Mekanism é o **primeiro cliente**; os próximos ports (outros mods) validam que a API não ficou sob medida para um só.
**EN.** Every generalisable `FORK-n` (A, C, E and part of D) is written as a **named capability** of the SDK (`horizon-mod-sdk`) with: contract, minimal example, conformance test. Mekanism is the **first client**; the next ports validate that the API was not tailored to a single mod.

**PT.** Ordem sugerida / EN. Suggested order: FORK-1,2,4 (redes) → FORK-12,13 (mundo) → FORK-7,8,9 (entidades/dano) → FORK-16,17,18 (filtros/frequências) → FORK-19,20 (visual) → FORK-23…27 (dados).

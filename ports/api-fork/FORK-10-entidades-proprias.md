# FORK-10 — entidades próprias de mod (o Robit) / Mod-defined entities (the Robit)

**PT-BR.** O Robit (`entities/EntityRobit`) é uma entidade viva com tipo próprio, IA (seguir o dono, recolher itens), inventário de 30 slots, energia, fornalha interna, 13 skins, tela e botões. O Horizon hoje só deixa o mod criar entidades **do jogo** (`spawn-mob` com um tipo vanilla, `summon`): não há tipo definido pelo mod. Sem o FORK-10 o Robit não pode existir em fidelidade; um mob vanilla de enfeite não é o Robit.

## O que o Java exige (conferido em `EntityRobit`, `RobitAIFollow`, `RobitAIPickup`, `ItemRobit`)
1. **Tipo registrado pelo mod:** id `mekanism:robit`, caixa 0,6 × 0,65, 1 de vida máxima, velocidade 0,3, não some sozinho, modelo e texturas próprios (13 skins, cada uma com textura(s) e, em alguns casos, modelo próprio).
2. **IA por tick, escolhida pelo mod:** seguir o dono (começa a 4 blocos, para a 2, teleporta a 12), recolher itens num raio de 10, voltar para casa (carregador) quando a energia acaba; o caminho é do jogo (pathfinding), o mod diz *para onde* e *a que velocidade*.
3. **Estado persistente do mod na entidade:** dono, segurança, skin, nome, casa, 30 slots, energia, progresso da fornalha.
4. **Eventos:** clique direito (abre a tela; sneak + mão vazia recolhe de volta ao item), dano (o dano vira gasto de energia: `energia por dano`), morte (solta o conteúdo), item de spawn (`ItemRobit` coloca no chão sobre um Chargepad).
5. **Consultas:** itens soltos numa caixa (para o recolhimento), jogadores/entidades por caixa.
6. **Tela:** contêiner com slots e botões ligados à entidade (hoje a tela é de bloco).

## Proposta / Proposal (resumo; assinaturas completas em `PROPOSTA-API.wit`)
- `registration.entity-kinds: list<entity-kind>`: `{ id, width, height, eye-height, max-health, speed, persistent, skins: list<skin>, container-slots }`, onde cada `skin` é `{ id, texture, model }` (assets do próprio mod).
- Exports: `entities-tick(list<mod-entity>) -> list<mod-entity-update>`, `mod-entity-used(player, entity) -> bool`, `mod-entity-damaged(entity, source, amount) -> f32`, `mod-entity-died(entity)`.
- Imports: `spawn-mod-entity(dimension, kind, x, y, z, data) -> ticket`, `path-to(dimension, id, x, y, z, speed)`, `set-mod-entity(dimension, id, update)`, `items-in-box(dimension, min, max) -> list<dropped-item>`, `take-dropped-item(dimension, id, count)`.
- Persistência: o `data` (bytes) e os slots/energia da entidade salvos com ela, como o `BlockEntity`.

## Ordem sugerida / Suggested order
1) registrar o tipo + spawn + persistência + render com textura/modelo; 2) tick com `path-to` e teleporte; 3) clique e menu; 4) dano e morte; 5) itens soltos.

**Quando existir:** ligar `robit.rs` (todas as regras de energia, seguir, recolher, fornalha, skins e botões já estão no módulo e conferidas, ver `revisao_numeros/REVISAO_robit.md`).

**EN.** The Robit is a living entity with its own type, AI, a 30-slot inventory, energy, a furnace and 13 skins. Horizon only lets a mod create the game's own entity types. Without FORK-10 (mod-defined entity types with per-tick control, persistence, interaction, damage, dropped-item queries and a menu) the Robit cannot exist faithfully. Everything in `robit.rs` is ready to be wired once it exists.


---

## Levantamento no fork (passo 4, antes de codar) / Survey of the fork (step 4, before coding)

**PT-BR.** Conferido em `horizon-android` (branch `mekanism-api`). Não é um patch pequeno; por isso o FORK-10 vai em sub-passos, cada um com patch, teste e exemplo.

| Ponto do fork | Hoje | O que o FORK-10 precisa |
|---|---|---|
| `mobs/species.rs` | `enum Species` fechado (vanilla); `def(s)` cai em `ALL[0]` (porco) se não achar | variante `ModEntity` com `Def` próprio (som, animação, caixa vêm do tipo do mod, não do `Def`); o `cargo check` com a variante nova compila sem erro (os `match` têm `_`), mas o resto do código trata espécie desconhecida como porco: cada consulta a `def()` precisa de revisão |
| `mobs.rs` `MobTypes::load` | só percorre `species::ALL` e lê `entity-types.json` / `entity-models.json` da pasta de dados | também os tipos do mod, com `layers` do modelo do próprio mod (mesmo formato de `entity-models.json`, `Part::parse`) |
| `Resources.mob_types` | `HashMap` imutável criado ao carregar | tipos do mod entram na carga (conteúdo, sem código): `horizon/entity_kind/*.json` ao lado de `item/`, `block/` |
| `horizon-mods/src/content.rs` `EntityType` | só projéteis (`template: minecraft:snowball`) | tipo novo `EntityKind` (caixa, vida, velocidade, skins, slots) |
| spawn / `Mob` | `spawn_mob_for` busca em `mob_types` | `spawn-mod-entity` + `Extra.modded: { data, items, energy, skin }` |
| persistência | NBT de mob por espécie | gravar e ler `Extra.modded` |
| IA | `ai::Ai` por espécie | espécie `ModEntity` sem IA própria: anda só pelo `path-to` do mod |
| render / LAN | modelos e texturas por `Def` | modelo e skin do mod; o pacote de mob leva o `kind` e a skin |
| tela | menu de bloco | menu de entidade (passo 3) |

**Decisão de desenho:** tipos declarados em **conteúdo** (JSON, sem código), como itens e blocos; o código do mod só cria, move e lê as instâncias.

**Sub-passos (um patch cada, na ordem do Java):** (a) tipo + spawn + remover + dados/slots/energia persistentes + `entities-tick`, só no servidor; (b) modelo e skin no cliente e na LAN; (c) `path-to` e teleporte; (d) clique e menu de entidade; (e) dano e morte; (f) itens soltos (`items-in-box`, `take-dropped-item`).

**EN.** Surveyed in the fork: `Species` is a closed enum, `def()` falls back to the pig, `MobTypes::load` only walks the vanilla list, and `Resources.mob_types` is built once at load. So mod entity kinds are declared as **content** (JSON, no code) and the mod's code only spawns, moves and reads instances. Sub-steps, one patch each: (a) kind + spawn + remove + persistent data/slots/energy + `entities-tick`, server only; (b) client/LAN model and skin; (c) `path-to` and teleport; (d) entity click and menu; (e) damage and death; (f) dropped items.

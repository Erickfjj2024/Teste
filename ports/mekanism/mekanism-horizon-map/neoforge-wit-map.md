# NeoForge → Horizon WIT: o mapa do roubo

Roubar os **conceitos** do NeoForge, não os binários. Um mod NeoForge não roda no Horizon (sem JVM, sem classes do Minecraft Java — ver a conversa anterior), mas o vocabulário que um modder NeoForge já conhece pode ser o vocabulário da API WIT. Este mapa diz, conceito por conceito, o que já existe, o que é parcial, o que falta e o que propor.

- **Alvo NeoForge:** 1.21.1 (NeoForge 21.1.x, o do Mekanism 10.7.x). Nomes conferidos contra os docs NeoForged e a análise de migração Forge 1.20.1 → NeoForge 1.21.1 do Mekanism.
- **Alvo WIT:** `ports/api-fork/reference/horizon.wit`, 1.398 linhas, API até 23 (branch `ccr-8f7d1cb2-4mjir6`, lido em 08/11/2026). Não o WIT enviado em 06/11 (1.378 linhas, sem `pull-items`/`mine-block`).
- **Como ler:** ✅ já existe · ◐ parcial, dá para trabalhar · ❌ gap, sem equivalente · ⛔ sem equivalente possível (o conceito não existe neste runtime).

## Resumo — as 25 áreas

| # | Área NeoForge | WIT hoje | Estado |
|---|---|---|---|
| 1 | Ciclo de vida / `@Mod` / mod event bus | `init` + `registration` (client/server) | ✅ |
| 2 | Registries / `DeferredRegister` | conteúdo declarativo no `.hmod` + `block-entities` no `init` | ◐ |
| 3 | Event bus (`NeoForge.EVENT_BUS`) | hooks fixos em `events` / `server-events` | ◐ |
| 4 | Capabilities (`Block/Item/EntityCapability`) | `energy-spec` / `tank-spec` / `container-spec` + transfers do jogo | ◐ |
| 5 | Data Components | `item-stack.components` (SNBT) | ◐ |
| 6 | Block / BlockState / BlockEntity | `block-entity-kind`, `block-entities-tick`, `world.*` | ✅ |
| 7 | Item (`Item`, `ItemStack`, uso/ataque) | `item-used` / `item-hit` + `input` + `player-items` | ◐ |
| 8 | Menus / Containers / Screens | `container-spec.menu` + menu JSON + `menu-button` + `menu-data` | ◐ |
| 9 | Receitas (`RecipeType`, `RecipeSerializer`) | `recipe-types` + `recipes` (leitura) | ◐ |
| 10 | Tags | `ingredient-items` aceita `#tag`; tags do item em `recipes.items` | ◐ |
| 11 | Loot tables / loot modifiers | — | ❌ |
| 12 | Networking (`CustomPacketPayload`, `StreamCodec`) | `net` / `net-client` / `net-events` (bytes) | ◐ |
| 13 | Config (`ForgeConfigSpec`, `ModConfig`) | `storage` + `ui.set-settings` | ◐ |
| 14 | Comandos (Brigadier) | `events.command` + `server-events.command` (tipado) | ✅ |
| 15 | Worldgen (features, placements, biome modifiers) | `chunk-loaded` + `world.set-blocks` | ◐ |
| 16 | Dimensões / Structures (jigsaw) | — (só `multiblock-spec` de máquinas) | ❌ |
| 17 | Entidades (`EntityType`, AI, atributos) | `entities` (só kinds vanilla) | ◐ |
| 18 | Efeitos / dano | `player-items.add-effect` / `hurt` | ◐ |
| 19 | Som / partículas | `play-sound` ✅ / partículas ❌ | ◐/❌ |
| 20 | Cliente / render (BER, `ItemProperties`, keys) | `render` + `input` + `ui` + `browser` + renderers declarados | ◐ |
| 21 | Data attachments (dados por jogador/entidade) | — (só `storage` global e `block-entity.data`) | ❌ |
| 22 | Datagen / reload listeners | — (geração no build, em `tools/`) | ❌ |
| 23 | Acesso ao vizinho / interop entre mods | — (`transmitter` cobre transferência, não leitura) | ❌ |
| 24 | Mixins / Access Transformers / coremods | — | ⛔ |
| 25 | Creative tabs / JEI / advancements | `browser` (painel tipo JEI) ◐ / resto ❌ | ◐/❌ |

Regra de bolso que o mapa inteiro confirma: **no NeoForge, o mod chama o jogo; no Horizon, o mod declara e o jogo chama o mod.** Quase tudo que no NeoForge é "chame este método quando quiser" vira no WIT "declare no `init`/no conteúdo e receba um hook". Isso não é detalhe de estilo, é a diferença que decide cada proposta abaixo.

## 1. Ciclo de vida — `@Mod`, mod event bus, `Dist` ✅

| NeoForge | WIT | Estado |
|---|---|---|
| `@Mod` + construtor, `FMLCommonSetupEvent`, `FMLClientSetupEvent`, `FMLDedicatedServerSetupEvent` | `events.init()` / `server-events.init()` → `registration`, erro desliga o mod | ✅ |
| `Dist.CLIENT` / `Dist.DEDICATED_SERVER`, `DistExecutor` | os `world`s: `horizon-mod` (cliente), `horizon-server-mod` (servidor), `horizon-ui-mod` (widgets) | ✅ |
| `ModLoadingContext`, `ModList` | manifesto `horizon-mod.toml` (`id`, `api`, `side`, `capabilities`, `experimental`) | ✅ |

Não falta nada estrutural aqui. O `init` único substitui as fases de setup; o que se perde é a granularidade (não há "setup comum, depois setup de cliente"), e nada no Mekanism precisa dela. **Proposta: nenhuma.** Só documentar: "o `init` é o construtor + `FMLCommonSetupEvent`; o `world` é o `Dist`".

## 2. Registries / `DeferredRegister` ◐

| NeoForge | WIT | Estado |
|---|---|---|
| `DeferredRegister<Block/Item/BlockEntityType/EntityType/MenuType/RecipeSerializer/SoundEvent/Fluid/...>` + `RegistryObject<T>` + `ResourceLocation` | conteúdo declarativo no `.hmod` (`assets/<ns>/`, `data/<ns>/`) + `block-entity-kind` registrado no `init` | ◐ |
| `BuiltInRegistries.BLOCK/ITEM/...` (leitura) | `recipes.items()` lista todos os itens; resto sem leitura de registro | ◐ |

No NeoForge o registro é código que roda no carregamento; no Horizon é arquivo que o jogo lê antes do mod rodar. O `block-entity-kind` é o mais próximo de um `DeferredRegister<BlockEntityType>` que existe: o mod declara o bloco, os slots, a energia, os tanques, o transmissor e o multibloco, e o jogo instancia. O manifesto já chama isso de capability `registries`.

**O que falta:** não há registro em runtime (um mod não cria um bloco novo depois do `init`); não há `BlockEntityType` separado do bloco; não há leitura genérica de registros (só itens, via `recipes`).

**Proposta (documentação, não WIT):** tratar o formato de conteúdo como "o `DeferredRegister` do Horizon" e escrever a tabela de correspondência arquivo → registro: bloco/item → `assets`+`data`; `BlockEntityType` → `block-entity-kind`; `MenuType` → `data/<ns>/horizon/menu/<block>.json`; `RecipeType` → `recipe-types`; `SoundEvent` → `assets/<ns>/sounds.json`. Um modder NeoForge entende o projeto inteiro se essa tabela existir. **Verificar:** onde o conteúdo declara a creative tab (área 25) e se tags novas se declaram via datapack no `.hmod`.

## 3. Event bus — `NeoForge.EVENT_BUS`, `@SubscribeEvent` ◐

| NeoForge | WIT | Estado |
|---|---|---|
| `NeoForge.EVENT_BUS`, `@SubscribeEvent`, `EventPriority`, cancelável (`setCanceled`), `EventResult` | hooks fixos: `tick`, `chat`, `block-broken`, `block-placed`, `player-joined/left`, `item-used` (retorna `bool` = toma o uso), `item-hit`, `chunk-loaded`, `multiblock-changed`, `menu-button`, `net-message`, `entity-spawned` | ◐ |
| `BlockEvent`, `PlayerEvent`, `LevelTickEvent`, `LivingEvent`, `EntityJoinLevelEvent`... | só os hooks acima; sem evento genérico, sem prioridade, sem cancelamento (exceto o `bool` do `item-used`) | ◐ |

O bus do NeoForge é aberto: qualquer classe de evento existe e qualquer mod ouve. O WIT é fechado: o jogo chama os hooks que o mod pediu no `registration`. Isso cobre o que o Mekanism mais usa no servidor (tick de máquina, uso de item, chunk, multibloco, menu), mas não cobre eventos de entidade (morte, dano, spawn), nem `PlayerInteractEvent` com face (só `pos` no `item-used`), nem eventos de nível.

**Proposta:** não criar um bus genérico (o custo é o WIT inteiro virar dinâmica de eventos). Em vez disso, **adicionar hooks sob demanda, um por necessidade provada**, com o mesmo desenho dos atuais (o mod pede no `registration`, o jogo chama). Candidatos já mapeados no projeto: `block-entity-data` (P2) e os checks abertos do P1. Um evento de "entidade morreu / jogador tomou dano" só entra se um port precisar — e aí entra como hook, não como bus.

## 4. Capabilities — `BlockCapability` / `ItemCapability` / `EntityCapability` ◐

| NeoForge | WIT | Estado |
|---|---|---|
| `Capabilities.EnergyStorage.BLOCK/ITEM/ENTITY`, `FluidHandler`, `ItemHandler` (`IEnergyStorage`, `IFluidHandler`, `IItemHandler`) | `energy-spec` (capacity/max-receive/max-extract/faces), `tank-spec` (kind/capacity/max-rate/faces), `container-spec` (slots/faces/inputs/outputs) no `block-entity-kind`; o jogo move energia/fluido/item entre vizinhos todo tick | ◐ |
| `RegisterCapabilitiesEvent` (registrar provider por bloco/item/entidade) | o `block-entity-kind` no `init` é o registro; sem registro por item ou por entidade | ◐ |
| Capability customizada (ex.: Mekanism `CHEMICAL`, `HEAT`, `LASER_DISSIPATION`, `RADIATION_ENTITY`) | `tank kind` arbitrário (`fluid`, `gas`, `infusion`...) cobre chemical/gás; resto sem equivalente | ◐ |
| `BlockCapabilityCache` (cache com invalidação) | o jogo resolve vizinhança todo tick; sem cache exposta | ✅ (não precisa) |

Aqui o roubo já aconteceu sem ninguém notar: o `block-entity-kind` **é** o `RegisterCapabilitiesEvent` declarativo. No NeoForge o mod diz "este bloco expõe energia com estes lados"; no WIT o mod escreve `energy: { capacity, faces }` e o jogo faz o resto, inclusive o transfer. A diferença real é que no NeoForge qualquer capability existe, e no WIT só existem três meios (energia, tanque, itens) — mas o meio `tank` aceita `kind` arbitrário, então o chemical do Mekanism já cabe nele.

**O que falta de verdade:**
- **Capability em item** (ex.: bateria que carrega, tanque portátil): hoje `player-items` lê/escreve slots, mas não há "este item expõe energia". O Mekanism tem vários itens com energia/fluido/gás. Parcialmente contornável via `components` + lógica no `tick`, mas sem o contrato de "outro mod/hopper lê a capability do item".
- **Capability em entidade** (ex.: radiação no jogador, `RADIATION_ENTITY`): sem equivalente — ver área 21.
- **Capability customizada com comportamento** (ex.: heat, laser): sem equivalente; o caminho é dado via `block-entity.data` + `net`, não via capability.

**Proposta:** não criar `capability` genérica no WIT agora. O que o Mekanism precisa e não tem é (a) energia/fluido/gás **no item** — desenhar como `item-capability` no formato de conteúdo do item (o item declara `energy`/`tank` como o bloco declara no `block-entity-kind`, e o jogo aplica); e (b) `get-block-entity` (P0-3) como o "getCapability do vizinho". O resto (heat, laser, radiação) resolve-se com dados + `net`, e vira proposta só se um segundo mod precisar.

## 5. Data Components — `DataComponentType<T>` ◐

| NeoForge | WIT | Estado |
|---|---|---|
| `DataComponentType<T>` com Codec + StreamCodec, `ItemStack.set/get(DataComponent)`, `MekanismDataComponents.ATTACHED_ENERGY` etc. | `item-stack.components`: SNBT dos componentes (`{"minecraft:damage":3}`), patch 0005; `block-entity-update.items` com stacks completos | ◐ |

O conceito foi roubado certo: componente é dado no stack, serializável, com sintaxe vanilla. O que falta é o **tipo registrado**: no NeoForge o componente tem nome, codec e sync; no WIT é uma string SNBT que o mod monta e parseia (o `write_tool_mode` do Configurator já faz parse SNBT estrutural por isso). Sem codec, sem validação, sem sync automático — mas com o mesmo poder expressivo.

**Proposta:** nenhuma no WIT. O lugar certo é o SDK Rust: um helper de componentes tipados (o "DataComponentType do Horizon") que serializa para a SNBT que o WIT aceita. Biblioteca, não protocolo.

## 6. Block / BlockState / BlockEntity ✅

| NeoForge | WIT | Estado |
|---|---|---|
| `Block`, `BlockState` (properties tipadas), `BlockBehaviour.Properties` | bloco declarado no conteúdo; estado como string `minecraft:oak_log[axis=y]`; `world.set-block-property` muda uma propriedade sem perder a block entity (patch 0016) | ✅ |
| `BlockEntity` + `BlockEntityTicker` | `block-entity-kind` (`ticks: bool`) + `block-entities-tick` (até 4096/call) + `block-entity-update` | ✅ |
| `BlockEntity#setChanged`, save/load NBT | `data: list<u8>` (até 65536 bytes) salvo com o chunk; `keep-data` (o bloco guarda os dados no item ao quebrar) | ✅ |

A área mais completa do WIT. O `block-entity-update` com `data` + `items` + `energy` + `tanks` + `menu-data` + `faces` é o `setChanged` + `saveAdditional` declarativo. **Gaps conhecidos, já mapeados no projeto:** sem leitura da block entity vizinha (P0-3, área 23) e `multiblock-changed` sem dados do membro (P2, área 16). **Proposta:** nenhuma nova aqui — executar P0-3 e P2, que já são propostas numeradas.

## 7. Item — `Item`, `ItemStack`, uso, ataque ◐

| NeoForge | WIT | Estado |
|---|---|---|
| `Item.use` / `useOn` (com `UseOnContext`: face, mão, jogador), `Item.hurtEnemy`, `Item.inventoryTick` | `server-events.item-used(player, stack, pos) -> bool` (sem face, sem slot, sem mão); `item-hit(player, stack, entity, kind)` (só ouve); `input.claim-items` + `input.held` (cliente) | ◐ |
| `ItemStack` (count, componentes, dano) | `item-stack` (item/count/components SNBT) | ✅ |
| `ItemProperties` (durabilidade, encantamentos, atributos) | via `components`; sem API direta de dano/encantamento | ◐ |

É aqui que mora o P1 inteiro: o `item-used` entrega menos contexto que o `UseOnContext` do NeoForge — sem face clicada, sem slot segurado, sem mão. O `input` do cliente até tem `held-item.slot` (0–8), mas o servidor não. **Proposta:** P1 (`held-slot` 24 + `set-slot-if` CAS 25), já decidido e com o desenho fechado (CAS em duas camadas, `p1-reconciliacao-0018.md`). Depois do P1, o `item-used` + `held-slot` cobre o `UseOnContext` exceto a face — e a face vem de `player-control.get-look` (API 16) + `world.get-block`, que é o que o Configurator já faz. `inventoryTick` de item (jetpack, tablet que carrega) cobre-se com `tick` + `player-items.get-inventory`.

## 8. Menus / Containers / Screens ◐

| NeoForge | WIT | Estado |
|---|---|---|
| `MenuType<T>`, `AbstractContainerMenu`, `ContainerData` (slots de 16 bits!), `Slot`, `Screen`, `ContainerScreen` | `container-spec.menu` + menu JSON (`data/<ns>/horizon/menu/<block>.json`, patch 0007) + `menu-button` + `menu-data: list<s32>` (16 bits no fio) | ◐ |
| `ContainerData` com `get/set` sincronizado | `menu-data` sincronizado via `block-entity-update` | ✅ |

O desenho é o mesmo do vanilla/NeoForge, inclusive o detalhe dos 16 bits — o WIT herdou o limite de propósito. Os gaps estão mapeados em `menus-format-proposals.md` (15 lacunas, camadas A/B/C): campo de texto e slider exigem WIT + fork (camada C), o resto é mod ou fork. **Proposta:** nenhuma nova — a decisão de camada A/B/C continua pendente e fora desta semana, de propósito.

## 9. Receitas — `RecipeType`, `RecipeSerializer` ◐

| NeoForge | WIT | Estado |
|---|---|---|
| `RecipeType<T>`, `RecipeSerializer`, `RecipeManager`, `RecipeHolder`, datagen de receitas | `recipe-types`: `recipes-of-type(kind)` lê JSONs crus de `data/<ns>/recipe/`; `cooking-recipes(kind)`; `ingredient-items(ingredient)` resolve `#tag` e listas | ◐ |
| `CraftingRecipe`, `SmeltingRecipe` etc. (vanilla) | `recipes`: `all()`, `known()`, `place()`, `give()` (leitura + uso pelo jogador) | ◐ |

O WIT separa o que o NeoForge junta: o jogo expõe as receitas vanilla para o jogador (`recipes`), e o mod lê os JSONs crus dos seus próprios tipos (`recipe-types`) e implementa a lógica. Não há `RecipeSerializer` com codec — o mod parseia o JSON como quiser — e não há como pedir ao jogo "crafte isso" (o `place` é o jogador clicando no livro). Para o Mekanism isso basta: as máquinas leem `recipes-of-type("mekanism:crushing")` e processam.

**Proposta:** nenhuma no WIT. O "serializer" vira uma struct Rust no mod. Se um dia fizer falta, o candidato é `craft` (o jogo executa uma receita do mod) — mas nenhum port pediu.

## 10. Tags — `TagKey`, `BlockTags`, `ItemTags` ◐

| NeoForge | WIT | Estado |
|---|---|---|
| `TagKey<Block/Item/Fluid>`, `BlockTags`, `ItemTags`, `FluidTags`, tags em datapack | `ingredient-items` aceita `#tag`; `recipes.items()` lista as tags de cada item | ◐ |

Leitura coberta; declaração de tag nova é via datapack no `.hmod` (a verificar no formato de conteúdo — ver área 2). Não há `TagKey` no WIT porque não há chamada que precise dele: o mod resolve ingredientes por string. **Proposta:** nenhuma; confirmar na tabela de conteúdo (área 2) que `data/<ns>/tags/` entra no `.hmod`.

## 11. Loot tables / loot modifiers ❌

| NeoForge | WIT | Estado |
|---|---|---|
| `LootTable`, `LootPool`, `GlobalLootModifier` (`AddItemModifier`, `LootTableIdCondition`...) | — | ❌ |

Sem API de loot. O que existe é `mine-block` (quebra como jogador e rola os drops) e `block-broken` (o mod ouve e pode dropar o que quiser via `summon`/`drop-item`). O Mekanism quase não usa loot custom — os drops de máquinas vêm de `keep-data` e do container. **Proposta:** nenhuma. Se um port precisar de loot de mob/bloco custom, o caminho é `block-broken` + `drop-item` (P4), não um sistema de loot.

## 12. Networking — `CustomPacketPayload`, `StreamCodec` ◐

| NeoForge | WIT | Estado |
|---|---|---|
| `CustomPacketPayload` + `Type<T>` + `StreamCodec` + `RegisterPayloadHandlersEvent` + `PayloadRegistrar` (`playToServer`/`playToClient`/`playBidirectional`) + `PacketDistributor` | `net.send(player, channel, data)` (servidor→cliente, 30 KB/msg, 32/tick), `net-client.send-to-server` (cliente→servidor), `net-events.message` / `server-events.net-message` (bytes crus, canal `<mod>:<name>`) | ◐ |
| Limite de 1 MiB clientbound / 32 KiB serverbound | 30 KB por mensagem, 32 por jogador por tick | ◐ (mais apertado) |

O transporte existe; o que falta é o **contrato declarativo**. No NeoForge o payload tem tipo, codec e direção registrados; no WIT o mod manda bytes e os dois lados combinam o formato por fora. Isso é suficiente — o Mekanism já faz assim — mas é o lugar onde mais se sente falta do vocabulário: cada par de metades reinventa o codec.

**Proposta (SDK, não WIT):** um `Payload` tipado no SDK Rust — o "CustomPacketPayload do Horizon": `struct` + `StreamCodec`-like (encode/decode) + registro de canal com direção, por cima de `net`/`net-client`. O WIT não muda; o modder NeoForge escreve o mesmo formato mental (`record MeuPacote(...) implements CustomPacketPayload`, `registrar.playToClient(...)`) e o SDK traduz para bytes + `send`. É o roubo de maior retorno por linha de código, porque networking é onde todo mod sofre.

## 13. Config — `ForgeConfigSpec`, `ModConfig` ◐

| NeoForge | WIT | Estado |
|---|---|---|
| `ForgeConfigSpec` (tipado, com comentário, range, validação), `ModConfig.Type` (COMMON/CLIENT/SERVER), tela de config, sync servidor→cliente | `storage` (key/value por mod, com quota) + `ui.set-settings` (página em Options > Mods) | ◐ |

Persistência e tela existem; o que falta é a **spec**: tipo, default, validação, e o sync do SERVER para o cliente. O Mekanism tem config extensa (geração, energia, etc.), e hoje cada valor seria uma chave em `storage` com parse manual.

**Proposta (SDK, não WIT):** `ConfigSpec` no SDK Rust por cima de `storage`: o mod declara os campos tipados com defaults, o SDK lê/escreve TOML/JSON numa chave e expõe a página via `ui.set-settings` com widgets gerados. Sync servidor→cliente, se precisar, via `net` (o servidor manda o config no `player-joined`). Nenhuma função nova no WIT.

## 14. Comandos — Brigadier ✅

| NeoForge | WIT | Estado |
|---|---|---|
| `RegisterCommandsEvent`, Brigadier (`Commands.literal`, `argument`, `suggests`), `CommandSourceStack` | `events.command` (cliente) + `server-events.command` com `argument-kind`/`argument-value` tipados (`word`, `text`, `integer` com bounds, `number`, `boolean`, `player`, `block-pos`) + completion no chat do jogador | ✅ |

Melhor que o equivalente: o WIT já entrega os argumentos parseados e tipados, com hint no chat. O mod só implementa. **Proposta:** nenhuma.

## 15. Worldgen — features, placements, biome modifiers ◐

| NeoForge | WIT | Estado |
|---|---|---|
| `Feature`, `PlacementModifier`, `BiomeModifier`, `ChunkGenerator`, `NoiseGeneratorSettings` | `server-events.chunk-loaded(dimension, chunk-x, chunk-z, new)` + `world.set-blocks` (até 4096/call) + `world.get-block`/`get-biome`/`get-environment` | ◐ |

Não há feature declarativa nem acesso ao gerador — e não precisa haver para o caso de uso real. A Fase 1 inteira (minérios do Mekanism) é `chunk-loaded` com `new = true` + `set-blocks`, e a distribuição vem do `OreType.java` original. O que se perde: estruturas de worldgen, biomas custom, controle de densidade por noise. O que se ganha: o mod controla tudo em código.

**Proposta:** nenhuma no WIT. Se um port precisar de estrutura no mundo (ex.: uma dungeon), o caminho é o mesmo `chunk-loaded` + `set-blocks` com uma função de colocação no mod. `set-forced` (chunk loader) já existe em `chunks`.

## 16. Dimensões / Structures ❌

| NeoForge | WIT | Estado |
|---|---|---|
| `DimensionType`, `LevelStem`, `Structure`, `StructureSet`, jigsaw, `StructureTemplatePool` | — | ❌ |
| Estruturas de máquinas (multibloco) | `multiblock-spec` + `multiblock-changed` | ✅ (é outra coisa) |

Sem dimensões custom nem structures de worldgen. O `multiblock-spec` resolve o que o Mekanism chama de estrutura (fissão, fusão, turbina, indução), mas é estrutura de máquina, não de mundo. Teleporter entre dimensões usa `player-control.teleport` para dimensões que existem. **Proposta:** nenhuma — dimensão custom é projeto de motor, não de API de mod. Se um dia houver, entra como `dimension` no conteúdo, não como WIT.

## 17. Entidades — `EntityType`, AI, atributos ◐

| NeoForge | WIT | Estado |
|---|---|---|
| `EntityType<T>`, `Entity`, `LivingEntity`, `Mob` + goals (`GoalSelector`), `Attribute` (`Attributes.MAX_HEALTH`, `MOVEMENT_SPEED`...) | `entities`: `mobs()` (lista os vivos), `spawn-mob` (kinds vanilla), `summon` (qualquer entidade via SNBT), `move/teleport/push/kill/remove-mob`, `set-mob-health`, display entities (`set-display`) | ◐ |
| `EntityType` custom, `EntityRenderer`, spawn egg | só kinds vanilla; sem tipo novo, sem ovo de spawn | ❌ |
| Atributos | só vida (`set-mob-health`, `max-health` na leitura) | ❌ |

O Mekanism quase não tem entidade viva própria — o Robit é o caso, e ele é hoje um problema de modelo + comportamento, não de `EntityType`. Display entities cobrem muita coisa visual (o que o port já usa para o cubo e transmissores). **Proposta:** nenhuma agora. Entidade custom é o item mais caro desta lista; só entra se um port (Robit, ou um mob de outro mod) provar que display + mob vanilla não basta.

## 18. Efeitos / dano ◐

| NeoForge | WIT | Estado |
|---|---|---|
| `MobEffect`, `MobEffectInstance`, `DamageSource`, `LivingHurtEvent`, `DamageType` | `player-items.add-effect` (efeito vanilla ou do próprio mod, duração/amplificador), `player-items.hurt` (damage-type string, com armadura/absorção/tempo de invulnerabilidade) | ◐ |

Dar efeito e dano ao jogador está coberto, inclusive efeito custom do mod (`mymod:radiation` é citado no próprio WIT). O que falta é o outro lado: **ouvir** dano/morte (sem `LivingHurtEvent`) e dar efeito a mobs (só jogador). **Proposta:** nenhuma agora — o buraco real é o de eventos de entidade (área 3), e ele se resolve por hook sob demanda, não por uma API de efeitos.

## 19. Som / partículas ◐ / ❌

| NeoForge | WIT | Estado |
|---|---|---|
| `SoundEvent`, `SoundSource`, `level.playSound` | `player-control.play-sound` (som vanilla ou do mod via `assets/<id>/sounds.json`, categoria `block`) | ✅ |
| `ParticleType`, `ParticleOptions`, `level.addParticle` | — | ❌ |

Som coberto (e o mod Mekanism nunca usou `play-sound` — omissão do mod, não da API). Partícula é gap puro: o Mekanism usa em vários lugares (laser, SPS, fusão, teleporter, evaporação). **Proposta (WIT, P-nova):** `spawn-particle(dimension, particle, x, y, z, count, dx, dy, dz, speed)` — uma função, espelho de `play-sound`, com os tipos de partícula vanilla por string e quantidade/limites como os do `summon` (64/call). Barata no fork, destrava efeito visual em meia dúzia de máquinas. Prioridade P1, depois do 0018/0019.

## 20. Cliente / render — BER, `ItemProperties`, keys ◐

| NeoForge | WIT | Estado |
|---|---|---|
| `BlockEntityRenderer` (BER), `EntityRenderer`, `ItemProperties` (modelo por predicado), `MenuScreen`, overlays (`GuiOverlay`), `KeyMapping` + `RegisterKeyMappingsEvent` | `render`: `set-hud`/`set-corner` (shapes: fill/quad/sprite/item/text), `set-minimap`, `set-far-terrain`, `set-zoom`, `shake`; `input`: `claim-items`, `register-key`, `claim-wheel`, `held`, `poll`; `ui` + `browser`; renderers de bloco declarados (patch 0011) | ◐ |

O cliente do Horizon é o lado mais diferente do NeoForge: não há acesso ao pipeline de render, e o mod nunca desenha por conta própria — ele declara shapes e o jogo desenha. O que o NeoForge faz com BER + `ItemProperties` (modelo que muda com o estado), o WIT faz com renderer declarado + `render` shapes. Os gaps estão mapeados em dois lugares: `modelos-blocos.md` (BERs sem equivalente: pás do Wind, plasma da fusão, rotor da turbina, raios do SPS, número do Bin, feixe do laser) e P5/P7 (indicador de modo do Configurator: modelo por componente sem desenho completo).

**Proposta:** executar P5/P7 (já desenhadas) e tratar o resto como renderer declarado no conteúdo — cada BER vira um renderer no fork, não uma função no WIT. A regra: "o que no NeoForge é `BlockEntityRenderer`, no Horizon é um renderer declarado no conteúdo + dados via `block-entity`".

## 21. Data attachments — dados por jogador/entidade ❌

| NeoForge | WIT | Estado |
|---|---|---|
| `AttachmentType<T>`, `IAttachmentHolder` (dados em jogador, entidade, chunk, level), com sync opcional | `storage` (global do mod), `block-entity.data` (por bloco), `item-stack.components` (por item) | ❌ |

Este é o gap mais silencioso do mapa. No NeoForge, dado que pertence a um jogador (radiação acumulada, frequência do teleporter, energia do MekaSuit) vive como attachment no jogador, com ciclo de vida e sync. No WIT não há "dado por jogador": o mod improvisa com `storage` (chave por UUID) ou com componentes no item. Funciona, mas sem ciclo de vida (o dado não morre com o jogador) e sem sync (o cliente não sabe).

**Proposta (WIT, P-nova):** `player-data` — `get/set` de bytes por jogador, espelho do `storage` com chave de jogador, salvo com o jogador e entregue no `player-joined`. É a peça que falta para radiação, frequências e qualquer estado por jogador. Prioridade P1, junto com a partícula (área 19): as duas destravam sistemas inteiros do Mekanism (radiação, teleporter, SPS) e qualquer mod com estado por jogador.

## 22. Datagen / reload listeners ❌

| NeoForge | WIT | Estado |
|---|---|---|
| `DataProvider` (receitas, loot, tags, modelos, lang), `GatherDataEvent`, `AddReloadListenerEvent` | geração no build (`ports/mekanism/tools/`, `convert.py` etc.); receitas lidas via `recipe-types` | ❌ |

Não há datagen porque não há Java nem Gradle no fluxo: o `.hmod` é montado no build com as ferramentas do repo. Não há reload listener porque o conteúdo é fixo no carregamento. **Proposta:** nenhuma — o `tools/` do repo já é o datagen. O que falta é documentar o fluxo como tal (área 2).

## 23. Acesso ao vizinho / interop entre mods ❌

| NeoForge | WIT | Estado |
|---|---|---|
| `level.getCapability(...)` no vizinho, `BlockCapabilityCache`, `InterModComms` (IMC) | `transmitter` move energia/fluido/item entre vizinhos; `world.get-block` lê só a string do estado | ❌ |

O buraco mais conhecido do projeto (P0-3, do mapa original): o mod não lê a block entity vizinha — `get-block` devolve só o estado. Sem isso não há "este vizinho tem energia?", "este tanque aceita gás?", e nenhuma interop entre mods (um mod não lê a máquina do outro). Os transmissores contornam a transferência, mas não a leitura.

**Proposta:** P0-3 (`get-block-entity`), já proposta: leitura só do vizinho, só das próprias entidades do mod, com energia, tanques e contagens de itens, sem os stacks completos. É o `getCapability` do Horizon — e a peça que falta para o Configurator ler transmissores e para qualquer interop futura. Prioridade P0, no 0019 ou no próximo patch aditivo depois dele.

## 24. Mixins / Access Transformers / coremods ⛔

| NeoForge | WIT | Estado |
|---|---|---|
| Mixin (SpongePowered), `@ModifyArg`, `@Inject`, `@Redirect`; Access Transformers (`public`, `final` removido); coremods (ASM) | — | ⛔ |

Sem equivalente possível: não há bytecode para remendar. Tudo que no NeoForge é mixin vira, no Horizon, uma das três coisas: **(a)** função nova no WIT, **(b)** patch no fork, **(c)** "não dá, e o desenho muda". A regra de bolso para qualquer port: quando o original usa mixin, a pergunta não é "como traduzo o mixin", é "qual função no WIT ou patch no fork entrega o mesmo comportamento". O `set-block-property` (patch 0016) nasceu exatamente assim — no original é um mixin/tick que muda `lit`, no Horizon é uma função.

**Proposta:** nenhuma — é premissa, não gap. Registrar a regra no guia de port.

## 25. Creative tabs / JEI / advancements ◐ / ❌

| NeoForge | WIT | Estado |
|---|---|---|
| `CreativeModeTab`, `BuildCreativeModeTabContentsEvent` | — (verificar formato de conteúdo) | ❌ |
| JEI (`IJeiRuntime`, `IRecipeCategory`) | `browser`: painel de itens + receitas ao lado do inventário, com busca, abas e clique; `recipes` expõe itens e receitas | ◐ |
| Advancements (`Advancement`, `AdvancementProvider`) | — | ❌ |

O `browser` é o JEI do Horizon — e é melhor que o JEI nesse aspecto: o jogo desenha o painel, o mod só entrega itens e receitas. Creative tab e advancements não têm equivalente; a tab é declaração de conteúdo (a verificar) e advancement é conteúdo de datapack que o mod pode incluir no `.hmod` (a verificar). **Proposta:** nenhuma no WIT; fechar as duas verificações na tabela de conteúdo (área 2).

## O que propor — a lista de roubo, em ordem

Nada aqui é "implementar o NeoForge". São as peças de vocabulário que faltam para um modder NeoForge se sentir em casa — e que o Mekanism precisa de verdade.

| # | Proposta | Tipo | Origem NeoForge | Prioridade | Estado |
|---|---|---|---|---|---|
| 1 | `held-slot` (24) + `set-slot-if` CAS (25) | WIT 0018 | `UseOnContext` (slot/mão) | P0 | Decidido, desenho fechado |
| 2 | `drop-item` (26) | WIT 0018 | `ItemEntity` / drop no mundo | P0 | Decidido |
| 3 | `get-block-entity` (P0-3) | WIT | `getCapability` no vizinho | P0 | Proposto, não decidido |
| 4 | `set-tank-capacity` (P0-1) | WIT | `setEnergyCapacity` espelhado | P0 | Proposto, não decidido |
| 5 | `block-entity-data` / dados do membro (P2) | WIT 0019 | `BlockEntity` do membro do multibloco | P0 | Decidido (mundo 0.4) |
| 6 | `player-data` (bytes por jogador) | WIT, pós-0019 | `AttachmentType` no jogador | P1 | **Nova neste mapa** |
| 7 | `spawn-particle` | WIT, pós-0019 | `level.addParticle` | P1 | **Nova neste mapa** |
| 8 | `Payload` tipado + `StreamCodec`-like | SDK Rust | `CustomPacketPayload` | P1 | **Nova neste mapa** |
| 9 | `ConfigSpec` tipado sobre `storage` | SDK Rust | `ForgeConfigSpec` | P1 | **Nova neste mapa** |
| 10 | `item-capability` no conteúdo (energia/tanque no item) | Conteúdo | `ItemCapability` | P1 | **Nova neste mapa** |
| 11 | Tabela de conteúdo = `DeferredRegister` | Docs | `DeferredRegister` | P0 | **Nova neste mapa** |
| 12 | Hooks sob demanda (sem bus genérico) | WIT, caso a caso | `NeoForge.EVENT_BUS` | — | Regra, não item |

O que **não** entra na lista, de propósito: mixins/ATs (⛔), dimensões/structures (motor), entidade custom (caro, sem port pedindo), loot (contornável), datagen (já existe em `tools/`).

## Verificações em aberto

1. Formato de conteúdo: creative tab, tags em `data/<ns>/tags/`, `SoundEvent` — confirmar o que o `.hmod` aceita e fechar a tabela da área 2.
2. `item-capability` no conteúdo: confirmar se o formato de item do conteúdo já tem campo de energia/tanque ou se precisa de extensão.
3. `player-data`: confirmar onde o jogador é salvo no fork (o `storage` é por mod; o dado por jogador precisa de um dono novo).
4. Números do `spawn-particle`: limites por chamada e por tick, como os do `summon` (64/call).

## Arquivos relacionados

- `configurator-api-proposals.md` — P1–P10, o desenho que este mapa estende.
- `configurator-api-dependencias.md` — o que no Configurator depende da API.
- `p1-reconciliacao-0018.md` — o desenho governante do P1 + veredicto do CAS.
- `menus-format-proposals.md` — as 15 lacunas de formato dos menus (área 8).
- `modelos-blocos.md` — os BERs sem equivalente (área 20).
- `mekanism-horizon-map.pdf` — o mapa original, com os gaps P0 (P0-1, P0-2, P0-3).

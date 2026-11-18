# Resumo da Auditoria: MekanismGenerators, MekanismTools e MekanismAdditions

**ID da Tarefa:** T20261109-184425-m8j  
**Data:** 2026-11-09  
**Status:** Concluído com sucesso

---

## 1. Escopo e Metodologia

Auditoria detalhada de valores numéricos, constantes de configuração, fórmulas e mecânicas dos módulos:
- **MEKANISM GENERATORS (`generators`):** Bio Generator, Heat Generator, Solar Generator, Advanced Solar Generator, Wind Generator, Gas-Burning Generator, Industrial Turbine, Fission Reactor, Fusion Reactor, Hohlraum.
- **MEKANISM TOOLS (`tools`):** 6 materiais (Bronze, Lapis Lazuli, Osmium, Refined Glowstone, Refined Obsidian, Steel) e seus itens (espada, pá, machado, picareta, enxada, paxel, capacete, peitoral, calças, botas, escudo) com durabilidade, dano, velocidade, encantabilidade, armadura, toughness e knockback resistance, mais os 6 paxels de materiais vanilla (madeira, pedra, ferro, ouro, diamante, netherite).
- **MEKANISM ADDITIONS (`additions`):** 18 cores de `EnumColor`, blocos de plástico (comum, slick, glow, reforçado, estrada, transparente, escadas, lajes, cerca, portão) e painéis de iluminação (`glow_panel`).

Fonte de verdade: Código Java do release `1.21.1-10.7.19.85` em `java/`.  
Código do port: `port/mod/src/generators.rs`, `port/mod/src/plus.rs`, `port/mod/src/turbine.rs`, `port/mod/src/fission.rs`, `port/mod/src/fusion.rs`, `port/mod/src/lib.rs`.

---

## 2. Divergências Encontradas e Correções Aplicadas no Port

Foram editados apenas os arquivos permitidos no escopo (`port/mod/src/generators.rs` e `port/mod/src/plus.rs`):

1. **`port/mod/src/generators.rs` - Faces de Saída do Advanced Solar Generator (`energy_faces`):**
   - *Java:* `TileEntityAdvancedSolarGenerator.java:18` define `ENERGY_SIDES = {RelativeSide.FRONT, RelativeSide.BOTTOM}`.
   - *Port antes:* emitia energia apenas para `DOWN` (`GenKind::Solar | GenKind::AdvSolar => faces[config::DOWN] = FlowSide::Output`).
   - *Correção:* agrupado com `Wind` para emitir em `DOWN` e na face frontal `front_of(facing, FRONT)` (`generators.rs:226`).

2. **`port/mod/src/generators.rs` - Orientação de Facing ao Posicionar (`run_generator`):**
   - *Java:* Geradores com face frontal viram para o jogador.
   - *Port antes:* `if !placed && !matches!(kind, GenKind::Solar | GenKind::AdvSolar)` ignorava Advanced Solar.
   - *Correção:* alterado para `if !placed && kind != GenKind::Solar` (`generators.rs:268`), garantindo que o Advanced Solar vire para o jogador ao ser colocado.

3. **`port/mod/src/generators.rs` - Penalidade de Chuva em Geradores Solares (`run_generator`):**
   - *Java:* `TileEntitySolarGenerator.java:183` aplica `peakMultiplier * 0.2F` (penalidade de 80%) em biomas com chuva quando está chovendo ou trovejando.
   - *Port antes:* apenas o fator de iluminação do céu `sky()` era afetado, sem aplicar os 80% de penalidade no `share`.
   - *Correção:* adicionada a checagem `if env.precipitation && (env.rain > 0.0 || env.thunder > 0.0) { share *= 0.2; }` (`generators.rs:400-403`).

4. **`port/mod/src/plus.rs` - Penalidade de Chuva no Solar Neutron Activator (`run_continuous`):**
   - *Java:* `TileEntitySolarNeutronActivator.java:201` aplica `production *= 0.2F` quando chove/troveja.
   - *Port antes:* `plus.rs` calculava a taxa com `peak` e `brightness` sem a penalidade de chuva.
   - *Correção:* adicionada a redução de 80% no `share` sob chuva/trovoada (`plus.rs:163`).

---

## 3. Testes Editados

Conforme as regras de execução, após a alteração dos testes, os nomes exatos das funções de teste editadas foram verificados:
- **`the_sun_follows_the_day_and_the_weather`** (`port/mod/src/generators.rs`): adicionada asserção provando a penalidade de chuva de 80% (`solar_production(1.0, raining_share) == 3 FE`).
- **`faces_turn_with_the_facing`** (`port/mod/src/generators.rs`): adicionadas asserções provando as saídas de energia do `AdvSolar` e `Wind` para `DOWN` e `EAST` (frente).

---

## 4. Prova de Mutação

- **Estado:** A prova de mutação está pendente do juiz (a tarefa não forneceu lista de mutações predeterminadas).

---

## 5. Arquivos Gerados

1. `.saida/T20261109-184425-m8j/AUDITORIA_generators_tools_additions.md` (tabela completa com todas as constantes Java vs Port, arquivos e linhas)
2. `.saida/T20261109-184425-m8j/BLOCO_DIFERENCAS.md` (bloco pronto para a seção 20 de `port/DIFERENCAS.md`)
3. `.saida/T20261109-184425-m8j/RESUMO.md` (este documento)

---

## 6. Bloco Pronto para `port/DIFERENCAS.md` (Seção 20)

```markdown
## 20. Auditoria MekanismGenerators, MekanismTools e MekanismAdditions (mod 0.1.86)

Auditoria realizada contra o código-fonte Java do release `1.21.1-10.7.19.85` e configs oficiais (`GeneratorsConfig`, `GeneratorsStorageConfig`, `ToolsConfig`, `MaterialDefaults`, `AdditionsBlocks`, `AdditionsConfig`).

| # | Diferença | Estado |
|---|---|---|
| 20.1 | **Advanced Solar Generator**: no Java emite energia por baixo (BOTTOM) e pela frente (FRONT) (`TileEntityAdvancedSolarGenerator.java:18`). No port emitia somente para DOWN e ignorava a orientação frontal ao ser posicionado. Corrigido: agora emite para DOWN e FRONT (faces do facing) e orienta para o jogador ao ser colocado no mundo | CORRIGIDO |
| 20.2 | **Geradores Solares (Solar e Advanced Solar)**: no Java sofrem penalidade de 80% na produção quando chove/troveja em biomas com precipitação (`peakMultiplier * 0.2F`, `TileEntitySolarGenerator.java:183`). No port aplicava apenas a redução de luminosidade solar do céu sem a penalidade de chuva. Corrigido em `generators.rs` | CORRIGIDO |
| 20.3 | **Solar Neutron Activator**: taxa de operação em chuva/trovoada sofre penalidade de 80% (`* 0.2F`, `TileEntitySolarNeutronActivator.java:201`). No port `plus.rs` não aplicava o fator de chuva. Corrigido em `plus.rs` | CORRIGIDO |
| 20.4 | **MekanismTools - Escudos**: escudos dos 6 materiais (durabilidade: Bronze 403, Lapis 224, Osmium 672, Refined Glowstone 381, Refined Obsidian 1 680, Steel 448) não existem no port por exigirem renderizador especial de escudo e banners do Horizon | LIMITE API |
| 20.5 | **MekanismTools - Refined Obsidian resistente a fogo**: no Java os itens e armadura de obsidiana refinada têm `burnsInFire = false` (`fire_resistant`), resistindo a fogo e lava; no port não têm a propriedade | LIMITE |
| 20.6 | **MekanismAdditions - Estrada de Plástico**: no Java dá aceleração de 1.6 (`boost = 1.6`, `BlockPlasticRoad.java:19`) a entidades que andam sobre ela; no port não acelera por ausência de evento de passo/movimento da entidade na API | LIMITE API |
| 20.7 | **MekanismAdditions - Glow Panel**: no Java se apoia na face clicada (`AttributeStateFacing.FacePlacementType.SELECTED_FACE`) com nível de luz 15; no port nasce fixo virado para o norte | LIMITE API |
| 20.8 | **MekanismAdditions - Conteúdo Ausente**: Obsidian TNT (delay 100 ticks, raio 12.0), Baby Mobs (6 variações, dano flecha ×0.25), Balões (entidades/física) e Walkie-Talkie (porta de voz 36123) não foram portados para o Horizon | LIMITE API |
```

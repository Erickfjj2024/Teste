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

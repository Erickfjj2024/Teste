# Mekanism: o que ele usa da API (levantamento)

Fonte: `mekanism/Mekanism`, branch `26.3` (MIT), 2416 arquivos Java
(main 1755, api 373, generators 148, additions 94, tools 46). Contagem de arquivos que usam cada recurso.

## 1. O que o Mekanism usa (arquivos)

| Recurso | Arquivos | Existe no Horizon Android hoje? |
|---|---|---|
| Entidades de bloco (máquinas) | 154 | Parcial: `ModBlockEntity`, `ModContainer`, `ModFluidTanks`, `ModEnergyBuffer`, só para mods nativos |
| Capacidades de transferência (itens/fluido/energia/químico) | 478 | Parcial (itens/fluido/energia nativos; **químico não existe**) |
| Componentes de dados (data components) | 158 | Não exposto a mods |
| Telas/GUI customizadas | 208 | Só 11 menus vanilla (`MOD_MENUS`) |
| Menus/containers | 39 | Só os vanilla |
| Pacotes de rede | 59 | Não exposto a mods WASM |
| Renderizadores/shaders/RenderType | 89 | Não |
| Renderizadores de bloco / de entidade | 16 / 27 | Não |
| Modelos customizados (loaders) | 21 | Não |
| Receitas (tipos próprios) | 70 | Não (só receitas JSON vanilla) |
| Químicos (gás, infusão, pigmento, lodo) | 79 | Não |
| Multibloco (turbina, reator, tanque, etc.) | 42 | Não |
| Transmissores/redes (cabos, tubos) | 34 | Não |
| Fluidos próprios (FluidType) | 64 | Não |
| Sons / Partículas / Dano | 54 / 22 / 36 | Som sim; partículas e tipos de dano não |
| Entidades (Robit, projéteis, etc.) | 13 | Não |
| Armaduras/equipamentos | 15 | Não |
| Itens com lógica própria | 28 | Só blocos/itens simples |
| Worldgen (minérios, sal, etc.) | 9 | Não (JSON de worldgen talvez) |
| Chunk loading | 8 | Não |
| Teclas, comandos, config, advancements, efeitos | 10/6/37/12/2 | Comandos e config parciais |
| Tags | 43 | Sim (JSON) |

## 2. Conclusão honesta

Não existe "o mínimo" para o Mekanism inteiro: quase tudo acima falta para mods **WASM**, e o
Mekanism depende de GUIs próprias, químicos, redes de cabos e multiblocos. É um projeto de
meses, não de uma sessão. Mas a ideia do usuário (API completa primeiro, um APK só) é a certa:
as **primitivas** abaixo, feitas uma vez no APK, permitem que máquinas novas sejam só arquivos `.hmod`.

## 3. Primitivas a implementar no APK (uma vez)

Ordem pensada para cada etapa já entregar algo jogável sem novo APK depois:

1. **Máquina declarativa (`machine` em conteúdo JSON)**: bloco + inventário (slots, lados) + energia + tick de receita + menu + estados visuais (`active`, `facing`). Cobre Crusher/Enrichment/Smelting/Combiner/etc. (a maior parte das 154 entidades).
2. **Tipo de receita genérico**: `recipe_type` com entradas/saídas por JSON (crushing, enriching…) lido por dados, sem código.
3. **Energia como capacidade**: buffer, aceitar/extrair, itens que carregam energia (componente de dados), conversão entre unidades.
4. **Fluidos e químicos**: tanques genéricos com "tipo de substância" registrável (fluido vira caso particular).
5. **Telas declarativas**: layout JSON (slots, barras de energia/fluido/progresso, botões, abas) renderizado pelo motor; sem Java/Rust de tela por máquina.
6. **Rede**: pacotes cliente↔servidor genéricos expostos ao WASM (botões de GUI, configuração de lados).
7. **Redes de transmissão (cabos/tubos)** e **multiblocos**: primitivas de grafo/validação de estrutura no motor.
8. **Entidades, armaduras, itens com lógica, worldgen, partículas, dano**: etapas finais.

Isso exige mexer no fork (Rust, Android). O fork está 540k linhas atrás do upstream, então
as primitivas ficam como patches que podem ser propostos ao mantenedor (Vasysik) ou virar a
"alternativa própria" que o usuário mencionou.

## 4. Proposta de entrega por etapas (sem recompilar o APK a cada máquina)

| Etapa | APK novo? | Conteúdo jogável |
|---|---|---|
| A | sim (1x) | primitivas 1–3: máquina declarativa, receitas, energia → Crusher, Enrichment Chamber, Energized Smelter, geradores simples |
| B | sim (1x) | primitivas 4–6: fluidos/químicos, telas, rede → Electrolytic Separator, Chemical Infuser, tanques |
| C | sim (1x) | primitivas 7: cabos, tubos, multiblocos → Turbina, Fusão, Fábrica |
| D | sim (1x) | entidades, armaduras, worldgen → Robit, armaduras, minérios |
| após | **não** | todas as máquinas novas = só `.hmod` |

Cada "sim" é um APK que cobre uma família inteira. Mecânica nova só entra no APK se for primitiva nova.

## 5. Antes de começar a etapa A

- Verificar licença de assets (texturas/sons do Mekanism são MIT? conferir `LICENSE` e `NOTICE` do repo; mod "Mekanism" tem partes com licenças separadas).
- Decidir: fazer as primitivas no fork atual (Vasysik) ou na "alternativa própria".

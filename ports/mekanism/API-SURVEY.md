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

## 4. Decisão do usuário: o Mekanism inteiro é UM mod

Correção ao plano anterior: não serão vários `.hmod` por máquina. O resultado final é
**um único Mekanism** (conteúdo + código), cobrindo Mekanism, Generators, Additions e Tools
(os 4 módulos do repositório). Por isso o trabalho vem em duas fases:

**Fase 1: preparar a API inteira no APK (maior parte do esforço).** Nada do Mekanism é
portado antes de a API existir e ser testada com mods de teste pequenos.
**Fase 2: portar o Mekanism sobre a API pronta**, módulo por módulo, tudo dentro do mesmo mod.

### Fase 1: lista de trabalho da API (dependências em ordem)

| # | Bloco da API | Depende de |
|---|---|---|
| 1 | Entidades de bloco para WASM (estado, NBT, tick, carregar/salvar) | nada |
| 2 | Inventários e capacidade de itens (slots, lados, funis) | 1 |
| 3 | Energia (capacidade, itens carregáveis, conversão de unidades) | 1 |
| 4 | Fluidos e químicos (tipos registráveis, tanques) | 1 |
| 5 | Data components em itens/blocos | 1 |
| 6 | Tipos de receita próprios (dados + lógica) | 2 |
| 7 | Menus e telas customizadas (slots, barras, botões, abas) | 2, 3, 4 |
| 8 | Pacotes de rede genéricos cliente↔servidor | 7 |
| 9 | Redes de transmissão (cabos, tubos, grafos) | 2, 3, 4 |
| 10 | Multiblocos (validação de estrutura, estado compartilhado) | 1, 9 |
| 11 | Renderização: modelos/estados, renderizadores de bloco e entidade, partículas | 1 |
| 12 | Entidades (robit, projéteis), IA simples | 5 |
| 13 | Itens com lógica, armaduras, ferramentas, efeitos, dano | 5 |
| 14 | Worldgen (minérios, sal), chunk loading, comandos, config, teclas | — |

Cada bloco é testado com um mod de teste mínimo antes do próximo. Ao final, um APK só.
Não é possível prometer prazo: é a parte longa do projeto.

## 5. Antes de começar a etapa A

- Verificar licença de assets (texturas/sons do Mekanism são MIT? conferir `LICENSE` e `NOTICE` do repo; mod "Mekanism" tem partes com licenças separadas).
- Decidir: fazer as primitivas no fork atual (Vasysik) ou na "alternativa própria".

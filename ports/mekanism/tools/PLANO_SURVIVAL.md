# Plano de teste "como no survival" / "Survival playthrough" test plan

**PT-BR.** Uma tela, uma máquina ou um multibloco só está **pronto** quando o jogador chega nele pelo caminho do survival: pegar os itens das receitas no menu criativo, **craftar tudo**, ligar a primeira fonte de energia, fazer as máquinas trabalharem e evoluir no mod até o reator funcionar. Montar a estrutura com `setblock` só prova que o formato é reconhecido; não prova a progressão. Este plano define as fases, o que cada uma comprova e como é conferida (no celular, com `horizon-ctl`, por script).

**EN.** A screen, machine or multiblock is **done** only when the player reaches it the survival way: take the recipes' items from the creative menu, **craft everything**, power the first energy source, make the machines work and progress through the mod until the reactor runs. Placing a structure with `setblock` only proves its shape is recognised, not the progression. This plan lists the phases, what each proves and how it is checked (on the phone, with `horizon-ctl`, scripted).

## Fases / Phases
| # | O que comprova / What it proves | Como / How | Estado / State |
|---|---|---|---|
| P1 | **Todas as 887 receitas de crafting** (843 shaped + 44 shapeless) se montam na grade e dão o item e a quantidade certos, com ingredientes do menu criativo | `tools/survival/crafts.py`: para cada receita do `.hmod` (`data/mekanism/recipe`), `give` dos ingredientes (tags resolvidas), mesa de crafting, `click-slot` na grade (100+), lê o resultado (200), pega, compara com a receita | a fazer |
| P2 | **Energia**: primeiro gerador (aquecimento/solar/bio), cabos, uma máquina de 1ª tier trabalhando (Enrichment, Crusher, Smelter) | construção por script + receitas reais; confere a barra de energia, o progresso e a saída | a fazer |
| P3 | **Processamento**: minério → poeira → sujeira → clump → cristal ... (as cadeias de 1x a 5x), fábricas, infusão | por máquina, com os itens produzidos pelas máquinas anteriores (nada de `give` de itens intermediários) | a fazer |
| P4 | **Materiais dos reatores**: aço, osmium, ligas, bobinas, vidro do reator, tanques, cabos e tubos | craftados em P1, usados aqui | a fazer |
| P5 | **Cadeia dos gases/combustível**: urânio, oxidação, flúor, centrífuga, deutério/trítio, água/sódio | por máquina e por tubo de pressão; onde faltar algo do port (itens de gás), registrar | a fazer |
| P6 | **Reatores rodando**: caldeira, fissão, fusão montadas com os itens craftados (não por criativo), alimentadas pelos tanques e pelo laser, vendo o resultado nas telas e submenus | montagem bloco a bloco por script em lote | parcial: as 3 estruturas formam e abrem (feito com `setblock`) |
| P7 | **Todas as abas e janelas** de todas as telas (não só a principal) | clicar cada `screen-<id>` de `horizon-ctl ui`, abrir, capturar, fechar | a fazer |

## Regras do ambiente (lições) / Environment lessons
- `fill` não avisa o escaneamento de multibloco: um `setblock` por bloco, em lote (`~/wa-bridge/build_multi.py`, `hzb.py`).
- `command tp @s x y z` funciona; o `tp` de controle falha. `interact X Y Z` com a mão vazia abre o menu do bloco; `use` abre um container genérico sem o painel do mod.
- O jogo é encerrado pelo Android quando outro app toma a tela: `save-all` antes de qualquer pausa e manter o jogo em primeiro plano.
- Texturas por modo das portas (entrada/saída de refrigerante/saída de resíduo) e as faces animadas (`.mcmeta`) dos reatores não estão no port: lacuna registrada.

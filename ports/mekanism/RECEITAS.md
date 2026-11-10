# RECEITAS — conferência mecânica, receita por receita / Mechanical recipe check, one by one

> Gerado por `tools/receitas_comparar.py` (não é resumo de IA: compara cada JSON do datagen do Mekanism 1.21.1-10.7.19.85 com o que o conversor entrega). Lista completa das ausências: `csv/receitas_ausentes.csv` (988 linhas, com a causa de cada uma).
> Produced by `tools/receitas_comparar.py` (not an AI summary): compares every datagen JSON with what the converter outputs. Full list of gaps: `csv/receitas_ausentes.csv`.

## Como rodar / How to run
```bash
python3 -I tools/base_1211.py <mekanism-1211> /tmp/base <mekanism-26.3>
python3 -I tools/convert.py /tmp/base /tmp/mod
python3 -I tools/receitas_comparar.py <mekanism-1211>/src/datagen/generated /tmp/mod/data /tmp/cmp.json
```

## Resultado (mod 0.1.86, depois das correções desta sessão) / Result

| | Quantidade |
|---|---:|
| Receitas do Java sem condição de mod externo / Java recipes without external-mod condition | 2814 |
| Receitas de compatibilidade (BOP, BWG, Farmer's Delight, AE2), fora do escopo / compat recipes, out of scope | 637 |
| **Existem no port e são IGUAIS ao Java (ids, quantidades, tempos, energia)** / present and equal | **1396** |
| Existem com mudança só de formato (`neoforge:compound`→`or`, `item_output`/`chemical_output`→`output`), nenhum número alterado / present, shape-only | 43 |
| Existem com outro nome de arquivo (crafting de Additions/Tools/Generators) / present under another file name | 387 |
| **Ausentes de verdade** / truly absent | **988** |

**Nas receitas que existem, nenhum número difere do Java.** / In the recipes that exist, no number differs from Java.

## Bugs achados e corrigidos nesta conferência / Bugs found and fixed
1. **Tipos de receita não convertidos** (`evaporating`, `activating`, `centrifuging`): a Thermal Evaporation Plant, o Solar Neutron Activator e a Isotopic Centrifuge tinham o livro de receitas **vazio** (ver `SINTOMAS_MULTIBLOCOS.md` S5). Corrigido em `MACHINE_TYPES`.
2. **Colisão de nomes de arquivo nas receitas de Additions (e Tools):** `additions_{resultado}.json` fazia receitas diferentes do mesmo resultado se sobrescreverem. Ex.: a receita principal do plástico preto (HDPE sheet + corante → 4 blocos) era substituída pela "2 lajes → bloco": **não era possível fabricar plástico da forma original**. Corrigido: o nome agora vem do caminho da receita (`additions_<caminho>.json`, `tools_<caminho>.json`). 50 receitas recuperadas.

## Ausências restantes (988), por causa / Remaining gaps
| Qtd | Causa | Plano |
|---:|---|---|
| 522 | Additions: `painting` (288), `pigment_extracting` (216), `enriching` (18) que o conversor pula de propósito | converter (não há limite: os itens existem) |
| 256 | Recolor de plásticos/glow panels: ingrediente `neoforge:difference` com a tag do mod (`mekanismadditions:glow_panels`) | FORK: ingrediente `difference` e tags de mod nas receitas; ou expandir em receitas fixas |
| 86 | `mek_data`: evolução de fábricas e cubos que preserva os dados do item (hoje vira crafting comum, perde os dados) | FORK-25 |
| 76+3 | Saída/ingrediente que ainda não existe no port (itens de QIO, MekaSuit, etc.) | volta sozinha quando a peça existir (rodada 3) |
| 17 | Tipos especiais/dinâmicos: `bin_insert/extract`, `clear_configuration`, `energy_conversion`, paxel, escudo-bandeira, smithing | FORK-25 (receitas dinâmicas) |
| 14 | Generators: bio, heat, gas burning, advanced solar, fusion (controller/frame/glass/port/logic adapter), laser focus matrix + `paper`, `rails`, `charcoal`, `sulfur_dye`: filtrados por ingrediente de outro módulo | investigar o filtro do conversor e liberar |
| 14 | Nuggets de Tools (smelting/blasting), pulados de propósito | liberar (os itens existem) |

**EN.** No number differs in present recipes; two converter bugs fixed (three recipe types never converted; recipe files overwritten by name collision). 988 recipes remain absent, each with a cause in the CSV; none is "unexplained" except 14 Generators/vanilla-override recipes to investigate.

## Atualização: receitas de máquina do Additions / Update: Additions machine recipes
**PT-BR.** O conversor agora gera as receitas de máquina do Mekanism Additions para o plástico e os glow panels: **270 da máquina de pintura** (o ingrediente `neoforge:difference` — "qualquer bloco desse tipo menos esta cor" — é escrito como a lista dos itens, a partir da tag do Additions), **216 do extrator de pigmento** (um pigmento de cada bloco de plástico) e **18 do enriquecedor** (plástico liso). Total **504** de 522; as 18 que faltam são as de **balões**, que o port ainda não tem. Isso fecha a maior parte da linha "522 Additions" da tabela acima. Ficam as **256 receitas de recolorir por *crafting*** (ingrediente `neoforge:difference` em receita de bancada): o mesmo método de escrever a lista serve e é o próximo passo.
**EN.** The converter now writes the Additions machine recipes for plastic and glow panels: 270 painting (the difference ingredient written out as the item list), 216 pigment extracting and 18 enriching: 504 of 522; the 18 left are balloon recipes (no balloons yet). Left: the 256 recolour crafting recipes, which the same method can do.

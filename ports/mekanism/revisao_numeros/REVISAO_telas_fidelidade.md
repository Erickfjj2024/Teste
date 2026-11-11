# Revisão das telas contra o original / Screens reviewed against the original

**PT-BR.** Pedido do usuário: antes de dar as telas por prontas, rever as que já estavam feitas, porque nem todas eram fiéis ao Java. O ponto de partida é a docstring de cada `*_menu` em `tools/convert.py`: as que dizem "(simplified)" foram feitas de memória do layout, sem conferir cada elemento no `Gui*.java`. Cada tela abaixo é refeita lendo o `Gui*.java` (posições `new GuiX(this, x, y ...)`, campos, abas, textos) e o tile (dados e botões).

| Tela | Estado | O que a revisão achou / fez |
|---|---|---|
| `laser_amplifier` | **refeita** | Faltavam os três campos de texto (atraso, mínimo, máximo), a aba de saída de redstone (3 modos), o controle de redstone, o medidor de energia padrão e o texto "No delay". Refeita; o mod ganhou o modo de saída, o controle e a leitura de notação científica. Falta: modo de detecção de entidade (o host não tem dano de laser). |
| `sps_port` | **refeita** | Tela de 176 por 182 com os medidores padrão, a tela interna (estado, energia, taxa) e a barra de progresso roxa; antes eram medidores errados e linhas inventadas. |
| `thermal_evaporation_controller` | **refeita** | Largura 196, slots dos baldes em 28,20 / 28,51 / 152,20 / 152,51 (estavam em 54 e 118), medidores 6,13 e 172,13, tela interna com 4 linhas, barra de temperatura, aba de calor. A perda de calor para o ar passou a ser a do Java (`simulateEnvironment` lê a temperatura antes de somar o calor da rodada). |
| `boiler_valve` | **refeita** | Largura 218, quatro medidores padrão (6, 26, 174, 194), duas barras verticais, tela interna (temperatura, taxa, máxima), sem slots (a antiga tinha slots de balde que o original não tem). Corrigida a ordem do calor: o ar dissipa a partir da temperatura anterior. Falta a segunda página (estatísticas) e a aba de aviso. |
| `turbine_valve` | **refeita** | Largura 190, medidor médio, barra de taxa de vapor, tela interna com 4 linhas, barra de energia, botão do modo de vapor, aba de energia. O mod ganhou o que faltava de lógica: o modo de vapor (ocioso, despejando excesso, despejando) e a entrada de vapor. Falta a segunda página (estatísticas) e a aba de aviso. |
| `fission_reactor_port` | a fazer | A tela atual tem layout inventado. O original: 195 por 255, quatro medidores (6, 25, 152, 171), tela interna (estado, queima, aquecimento, temperatura, dano, em cores), botões Ativar e Scram, luz grande, barra de temperatura e um **gráfico duplo de calor** (`GuiDoubleGraph`), mais uma segunda página de estatísticas com o campo do limite de taxa. O gráfico pede um widget novo no Horizon (FORK-20 parte 4). |
| `fusion_reactor_controller` | a fazer | |
| `induction_port` | a fazer | |
| `logistical_sorter` | a fazer (precisa de lista, FORK-20 parte 3) | |
| `quantum_entangloporter` | a fazer (precisa de campo de texto e lista) | |
| `teleporter` | a fazer (precisa de campo de texto e lista) | |
| `digital_miner` | a fazer (precisa de lista e campo de texto) | |
| `bin` | a conferir (o original não tem tela de lixeira) | |
| telas "completas" (máquinas de processo, bombas, aquecedores, assemblicator, estabilizador, vibrador, estação de módulos) | a conferir elemento a elemento | |

**EN.** The user asked to review the screens that were already done before calling them finished, since not every one matched the Java. The starting list is every `*_menu` docstring in `tools/convert.py` that says "(simplified)". Each one is rebuilt reading its `Gui*.java` and the tile.

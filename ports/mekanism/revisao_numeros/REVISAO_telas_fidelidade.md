# Revisão das telas contra o original / Screens reviewed against the original

**PT-BR.** Pedido do usuário: antes de dar as telas por prontas, rever as que já estavam feitas, porque nem todas eram fiéis ao Java. O ponto de partida é a docstring de cada `*_menu` em `tools/convert.py`: as que dizem "(simplified)" foram feitas de memória do layout, sem conferir cada elemento no `Gui*.java`. Cada tela abaixo é refeita lendo o `Gui*.java` (posições `new GuiX(this, x, y ...)`, campos, abas, textos) e o tile (dados e botões).

| Tela | Estado | O que a revisão achou / fez |
|---|---|---|
| `laser_amplifier` | **refeita** | Faltavam os três campos de texto (atraso, mínimo, máximo), a aba de saída de redstone (3 modos), o controle de redstone, o medidor de energia padrão e o texto "No delay". Refeita; o mod ganhou o modo de saída, o controle e a leitura de notação científica. Falta: modo de detecção de entidade (o host não tem dano de laser). |
| `sps_port` | a fazer | |
| `thermal_evaporation_controller` | a fazer | |
| `boiler_valve` | a fazer | |
| `turbine_valve` | a fazer | |
| `fission_reactor_port` | a fazer | |
| `fusion_reactor_controller` | a fazer | |
| `induction_port` | a fazer | |
| `logistical_sorter` | a fazer (precisa de lista, FORK-20 parte 3) | |
| `quantum_entangloporter` | a fazer (precisa de campo de texto e lista) | |
| `teleporter` | a fazer (precisa de campo de texto e lista) | |
| `digital_miner` | a fazer (precisa de lista e campo de texto) | |
| `bin` | a conferir (o original não tem tela de lixeira) | |
| telas "completas" (máquinas de processo, bombas, aquecedores, assemblicator, estabilizador, vibrador, estação de módulos) | a conferir elemento a elemento | |

**EN.** The user asked to review the screens that were already done before calling them finished, since not every one matched the Java. The starting list is every `*_menu` docstring in `tools/convert.py` that says "(simplified)". Each one is rebuilt reading its `Gui*.java` and the tile.

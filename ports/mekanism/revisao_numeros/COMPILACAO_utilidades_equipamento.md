# Compilação dos módulos de utilidades e equipamento / Compiling the utilities and equipment modules

> Tarefas c1 (utilidades) e c2 (equipamento): as VMs reiniciaram e deixaram trabalho parcial; terminei aqui com `tools/verificador/check.sh`.

**Resultado / Result:** o mod inteiro (todos os módulos, inclusive os 25 da rodada 3) compila com **0 erros** e **404 testes** passam.

## O que restava e o que fiz / What was left and what I did
| Arquivo | Erro | Correção |
|---|---|---|
| `meka_tool.rs` | `const fn` com `.min()` (4 pontos além do que a c2 corrigiu): `Ord::min` não é `const` | `if val < N { val } else { N }` |
| `robit.rs:663` | campo `default_manually_selected` inexistente | o campo é `default_skin_manually_selected` |
| `meka_tool.rs` (testes) | 7 empréstimos: `let m = tool.get_module_mut(..)` vivo enquanto `tool.calculate_attack()` | reobter o módulo a cada atribuição |
| `mekasuit.rs` (teste) | `4 * (6_400_000 * 256)` estourava `i32` | literais `u64` |
Aplicado do trabalho parcial da c1: `fork_utilidades.rs` e as correções em `seismic_vibrator`, `assemblicator`, `configuration_card`, `oredictionificator`, `chargepad`, `cardboard_box`, `dimensional_stabilizer` (as chamadas inventadas `world::set_block` e `world::has_neighbor_signal` agora ficam atrás de `#[cfg(feature = "fork")]`). Da c2: `fork_equipamento.rs`, `meka_tool.rs`.

## Pontos de fork / Fork points
25 funções em `fork_*.rs`, indexadas em `FORK-INDICE.md`. Com a feature `fork` ligada, o SDK atual só não tem `world::set_block` e `world::has_neighbor_signal`.

## Não conferido / Not checked
Estes módulos **compilam**, mas seus **números e regras ainda não foram conferidos contra o Java** (Meka-Tool, MekaSuit, Robit, Assemblicator, Chargepad, etc.): a conferência feita até aqui foi só do calor e do QIO. O teste do MekaSuit usa `6_400_000 * 256` FE por peça com 8 módulos de energia; **isso precisa ser conferido** em `ModuleEnergyUnit`/`MekaSuitArmor`.

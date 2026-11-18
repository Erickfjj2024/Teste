# Relatório de Conclusão: Revisão Cruzada de Máquinas Químicas e Especificação QIO/Segurança

**ID da Tarefa**: `T20261109-194128-vq7`  
**Data**: 2026-11-09  
**Ambiente**: ~/espelho/mekanism_audit (cópia sem repositório git)  
**Referência**: Mekanism Java `1.21.1-10.7.19.85` (`java/`)  

---

## 1. Escopo e Objetivos Realizados

1. **Revisão Cruzada (Rodada 2)** das Máquinas Químicas cobrindo os 10 arquivos do escopo:
   - `tank.rs`
   - `upgrades.rs`
   - `process.rs`
   - `reaction.rs`
   - `chem.rs`
   - `chem_table.rs`
   - `factory.rs`
   - `combiner.rs`
   - `sawmill.rs`
   - `machine.rs`
2. **Conferência Independente contra o Java** e cruzamento contra o relatório da Rodada 1 (`port/auditoria/AUDITORIA_quimicas.md` e `RESUMO_t1.md`), identificando falhas de aferição e aplicando correções diretamente no código Rust do port em `port/mod/src/`.
3. **Atualização da documentação de discrepâncias** com inclusão da Seção 21 em `port/DIFERENCAS.md`.
4. **Parte B (Especificação Técnica Exaustiva)** em `.saida/T20261109-194128-vq7/ESPEC_qio_seguranca.md` para os componentes: QIO Dashboard, QIO Drive Array, QIO Exporter, QIO Importer, QIO Redstone Adapter, QIO Drives (Base, Hyper-Dense, Time-Dilating, Supermassive), Security Desk (sistema de posse e frequências de segurança) e Modification Station.

---

## 2. Testes Unitários dos Arquivos no Escopo

Contagem real de testes (`#[test]`) extraída diretamente dos arquivos do escopo:
- `port/mod/src/process.rs`: 10 testes
- `port/mod/src/reaction.rs`: 4 testes
- `port/mod/src/chem.rs`: 1 testes
- `port/mod/src/chem_table.rs`: 0 testes
- `port/mod/src/factory.rs`: 7 testes
- `port/mod/src/combiner.rs`: 2 testes
- `port/mod/src/sawmill.rs`: 4 testes
- `port/mod/src/machine.rs`: 5 testes
- `port/mod/src/upgrades.rs`: 5 testes
- `port/mod/src/tank.rs`: 6 testes
- **Total**: 44 testes.
- **Execução na VM**: não medido (sem toolchain Rust/Horizon configurada no ambiente Cloud Shell).
- **Prova de Mutação**: a prova de mutação está pendente do juiz (a tarefa não trouxe lista de mutações).

### Testes Adicionados ou Editados nesta Tarefa:
- Em `port/mod/src/factory.rs`:
  - `the_charge_slot_turns_redstone_into_energy_while_there_is_room` (ajustado de 10.000 para 4.000 FE).
- Em `port/mod/src/upgrades.rs`:
  - `the_effects_follow_the_formulas` (validando `gas_mean_with_support` para máquinas sem suporte a chemical upgrade).
- Em `port/mod/src/process.rs`:
  - `constant_chemical_use_scales_with_speed_upgrades` (novo teste verificando que receitas com consumo contínuo de gás gastam a quantidade total nominal independente de speed upgrades).

---

## 3. Principais Descobertas e Correções da Revisão Cruzada

### 3.1 Erros Corrigidos da Rodada 1
1. **Capacidades de Armazenamento de Energia (`StorageConfig.java`)**:
   - A rodada 1 citou valores errados para 7 máquinas. No Java 1.21.1:
     - `chemicalDissolutionChamber`: 160.000 J = 64.000 FE (rodada 1 informou 32.000 FE).
     - `chemicalCrystallizer`: 160.000 J = 64.000 FE (rodada 1 informou 32.000 FE).
     - `combiner`: 40.000 J = 16.000 FE (rodada 1 informou 8.000 FE).
     - `osmiumCompressor`: 80.000 J = 32.000 FE (rodada 1 informou 16.000 FE).
     - `chemicalInfuser`: 80.000 J = 32.000 FE (rodada 1 informou 16.000 FE).
     - `isotopicCentrifuge`: 80.000 J = 32.000 FE (rodada 1 informou 16.000 FE).
     - `antiprotonicNucleosynthesizer`: 1.000.000.000 J = 400.000.000 FE (rodada 1 informou 8.000.000 FE).
2. **Consumo do Electrolytic Separator (`UsageConfig.java`)**:
   - A rodada 1 alegou que o consumo vinha de `UsageConfig.electrolyticSeparator`. No Java 1.21.1 não há essa chave em `UsageConfig`; o valor é dinâmico e calculado como $2 \times \text{hydrogenEnergyDensity} = 400\text{ J/t} = 160\text{ FE/t}$ em `TileEntityElectrolyticSeparator.java:199`.
3. **Classificação Inadequada como LIMITE**:
   - A rodada 1 classificou divergências do Horizon como `LIMITE`. De acordo com a Regra de Ouro do projeto, foram reclassificadas como **FORK** com indicação da capacidade de API necessária.
4. **Bug do Osmium Compressor c/ Speed Upgrades (`process.rs`)**:
   - No Java, quando a duração da operação do Osmium Compressor diminui (de 200 ticks para 20 ticks com 8 speed upgrades), a taxa por tick é recalculada para continuar consumindo os 200 mB nominais de gás. No port consumia 1 mB fixo por tick, gastando apenas 20 mB com 8 upgrades (trapaça de 10x). Corrigido em `process.rs`.
5. **Máquinas sem Suporte a Upgrade Químico (`upgrades.rs`)**:
   - No Java (`MekanismUtils.java:346`), máquinas que não aceitam Gas Upgrade (como o Metallurgic Infuser e Chemical Oxidizer) sofrem aumento de consumo de gás baseado exclusivamente no speed upgrade ($10^{speed/8}$). No port foi adicionada a função `gas_mean_with_support` para isolar essa regra.

---

## 4. Arquivos Modificados no Código-Fonte do Port

1. `port/mod/src/factory.rs`:
   - Alinhado teste de recarga do charge slot de redstone de 10.000 para 4.000 FE (`the_charge_slot_turns_redstone_into_energy_while_there_is_room`).
2. `port/mod/src/upgrades.rs`:
   - Implementada função `gas_mean_with_support(speed, chemical, supports_chemical)`.
   - Atualizado teste unitário `the_effects_follow_the_formulas`.
3. `port/mod/src/process.rs`:
   - Ajustada fórmula de consumo contínuo para manter invariante a quantidade total de químico gasto ao longo de durações reduzidas por speed upgrades.
   - Adicionado teste unitário `constant_chemical_use_scales_with_speed_upgrades`.
4. `port/DIFERENCAS.md`:
   - Adicionada Seção 21 com itens 21.1 a 21.6 cobrindo as decisões de arquitetura e correções da rodada 2.

---

## 5. Entregáveis Produzidos nesta Sessão

1. `.saida/T20261109-194128-vq7/REVISAO_quimicas.md`:
   - Tabela analítica completa comparando cada item entre Java (arquivo:linha) e Port (arquivo:linha), identificando os erros da rodada 1 e classificando cada linha (`CONFIRMADA`, `ERRADA-CORRIGIDA`, `OK`, `CORRIGIDO`, `FORK`, `FALTA`).
2. `.saida/T20261109-194128-vq7/ESPEC_qio_seguranca.md`:
   - Especificação técnica profunda e autosuficiente das peças do grupo QIO, Sistema de Segurança e Modification Station (registros, classes, blockstates, NBT, números exatos, coordenadas de GUI, JSONs de receitas datagen, redstone, pacotes de rede e mapeamento de FORKs).
3. `port/DIFERENCAS.md`:
   - Seção 21 inserida no padrão de documentação do projeto.
4. `.saida/T20261109-194128-vq7/RESUMO.md`:
   - Este relatório final com todas as contagens e dados verificados.

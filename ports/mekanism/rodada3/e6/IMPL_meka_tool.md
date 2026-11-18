# Relatório de Implementação: Grupo `meka_tool`

Conferência item a item de todos os comportamentos de **Meka-Tool**, **Electric Bow**, **HDPE Elytra** e dos **10 módulos associados** da especificação (`port/especificacoes/ESPEC_equipamento.md`) e do código de referência Java (`Mekanism 1.21.1-10.7.19.85`).

Legenda de Estados:
- **IMPLEMENTADO**: Lógica completa em Rust puro, executável e validada por testes unitários comparativos.
- **PARCIAL-por-FORK**: Lógica matemática e algorítmica completa em Rust, com ponto de chamada ao host isolado via stub `// FORK-n` e assinatura WIT proposta.
- **NAO**: Não implementado.

---

## 1. Meka-Tool e Módulos Associados (`ESPEC_equipamento.md` Seção 1)

| Comportamento / Requisito | Estado | Arquivo:Linha no Rust | Origem no Java | Observações |
|---|---|---|---|---|
| **Identificação do Item**: registro `mekanism:meka_tool` | IMPLEMENTADO | `meka_tool.rs:17` | `ItemMekaTool.java:86-97` | Epic rarity, unburnable, tool component |
| **Capacidade Base de Energia**: 16_000_000 J = 6.400.000 FE | IMPLEMENTADO | `meka_tool.rs:28` | `GearConfig.java:223` | `mekaToolBaseEnergyCapacity` |
| **Taxa Base de Carga**: 100_000 J/t = 40.000 FE/t | IMPLEMENTADO | `meka_tool.rs:31` | `GearConfig.java:224` | `mekaToolBaseChargeRate` |
| **Dano Base de Ataque**: 4.0 | IMPLEMENTADO | `meka_tool.rs:34` | `GearConfig.java:226` | `mekaToolBaseDamage` |
| **Velocidade Base de Ataque**: -2.4 (1.6 ataques/s) | IMPLEMENTADO | `meka_tool.rs:38` | `GearConfig.java:228` | `mekaToolAttackSpeed` |
| **Eficiência Base de Mineração**: 4.0 | IMPLEMENTADO | `meka_tool.rs:41` | `GearConfig.java:230` | `mekaToolBaseEfficiency` |
| **Alcance Máximo de Teletransporte**: 100 blocos | IMPLEMENTADO | `meka_tool.rs:47` | `GearConfig.java:234` | `mekaToolMaxTeleportReach` |
| **Custo Base Quebra de Bloco**: 10 J = 4 FE | IMPLEMENTADO | `meka_tool.rs:50` | `GearConfig.java:237` | `mekaToolEnergyUsage` |
| **Custo Quebra com Toque Suave**: 100 J = 40 FE | IMPLEMENTADO | `meka_tool.rs:53` | `GearConfig.java:238` | `mekaToolEnergyUsageSilk` |
| **Custo Base Ataque com Arma**: 2.000 J = 800 FE | IMPLEMENTADO | `meka_tool.rs:56` | `GearConfig.java:239` | `mekaToolEnergyUsageWeapon` |
| **Custo Ação de Enxada**: 10 J = 4 FE | IMPLEMENTADO | `meka_tool.rs:59` | `GearConfig.java:240` | `mekaToolEnergyUsageHoe` |
| **Custo Ação de Pá (caminho/apagar fogo)**: 10 J = 4 FE | IMPLEMENTADO | `meka_tool.rs:62` | `GearConfig.java:241` | `mekaToolEnergyUsageShovel` |
| **Custo Ação de Machado (descascar/raspar)**: 10 J = 4 FE | IMPLEMENTADO | `meka_tool.rs:65` | `GearConfig.java:242` | `mekaToolEnergyUsageAxe` |
| **Custo Tosa de Entidade**: 10 J = 4 FE | IMPLEMENTADO | `meka_tool.rs:68` | `GearConfig.java:243` | `mekaToolEnergyUsageShearEntity` |
| **Custo Tosa de Bloco (trim)**: 10 J = 4 FE | IMPLEMENTADO | `meka_tool.rs:71` | `GearConfig.java:244` | `mekaToolEnergyUsageShearTrim` |
| **Custo Teletransporte Base**: 1.000 J = 400 FE / 10 blocos | IMPLEMENTADO | `meka_tool.rs:74` | `GearConfig.java:245` | `mekaToolEnergyUsageTeleport` |
| **Limite Máximo Mineração em Cadeia**: 128 blocos | IMPLEMENTADO | `meka_tool.rs:77` | `GearConfig.java:143` | `disassemblerMiningCount` |
| **Módulo Energy Unit (max 8)**: escala exponencial 2^n | IMPLEMENTADO | `meka_tool.rs:88, 381-391` | `ModuleEnergyUnit.java:33` | 6.4M FE base até 1.638.400.000 FE com 8 unidades |
| **Módulo Attack Amplification Unit (max 4)**: dano 0, 4, 8, 16, 24, 32 | IMPLEMENTADO | `meka_tool.rs:136-155, 442-468` | `ModuleAttackAmplificationUnit.java:49-54` | Dano extra + custo 200 FE * unitDamage com escala de energia parcial |
| **Módulo Excavation Escalation Unit (max 4)**: vel 0, 4, 16, 32, 64, 128 | IMPLEMENTADO | `meka_tool.rs:168-189, 480-530` | `ModuleExcavationEscalationUnit.java:124-129` | Multiplica custo base por velocidade; reduz se energia faltar |
| **Módulo Blasting Unit (max 4)**: raio 1 (3x3), 2 (5x5), 3 (7x7), 4 (9x9) | IMPLEMENTADO | `meka_tool.rs:199-224, 535-585` | `ModuleBlastingUnit.java:107-111`, `IBlastingItem.java:26-50` | Orientação plana por face atingida (UP/DOWN, EAST/WEST, NORTH/SOUTH) |
| **Módulo Vein Mining Unit (max 4)**: alcance 2, 4, 6, 8 e modo extendido | IMPLEMENTADO | `meka_tool.rs:234-257, 590-678` | `ModuleVeinMiningUnit.java:136-177`, `ItemMekaTool.java:272-275` | Traversal BFS com multiplicador `0.5*d^1.5` para minérios e `0.5*d^2` para estendido |
| **Módulo Farming Unit (max 4)**: diâmetro 1, 3, 5, 7 | IMPLEMENTADO | `meka_tool.rs:267-295, 685-725` | `ModuleFarmingUnit.java:119-123, 220` | Ações de enxada, pá e machado em área |
| **Módulo Shearing Unit (max 1)** | IMPLEMENTADO | `meka_tool.rs:94, 68-71` | `ModuleShearingUnit.java:47-150` | Tosa de ovelhas e teias/folhas |
| **Módulo Teleportation Unit (max 1)**: distância mínima 5 e clearance | IMPLEMENTADO | `meka_tool.rs:95, 730-785` | `ModuleTeleportationUnit.java:10-17`, `ItemMekaTool.java:373-417` | Valida desmonte, pos.above e pos.above(2) livres, fallDistance = 0 |
| **Módulo Silk Touch Unit (max 1)**: Toque Suave I | IMPLEMENTADO | `meka_tool.rs:96, 429-432` | `MekanismModules.java:100` | Ativa custo silk (40 FE base) |
| **Módulo Fortune Unit (max 3)**: Fortuna I, II, III | IMPLEMENTADO | `meka_tool.rs:97, 434-440` | `MekanismModules.java:102` | Retorna nível igual a unidades instaladas (1..3) |
| **Data Components / SNBT**: serialização/desserialização | PARCIAL-por-FORK | `meka_tool.rs:790-898, 902-912` | `MekanismDataComponents.java:66, 72` | Lógica serializa/desserializa completa; chamada ao host isolada (FORK-23) |
| **Menu Radial de Modos**: seleção de sub-modos em jogo | PARCIAL-por-FORK | `meka_tool.rs:914-919` | `IRadialModuleContainerItem.java` | Estrutura de fatias pronta; abertura de UI isolada (FORK-27) |
| **Execução Multi-bloco no Mundo**: Blasting e Vein Mining | PARCIAL-por-FORK | `meka_tool.rs:921-931` | `MekanismUtils.java:807-860` | Coleta de posições e custos calculados; disparo no mundo via host (FORK-31) |
| **Execução de Teleporte e FX no Mundo** | PARCIAL-por-FORK | `meka_tool.rs:933-943` | `ItemMekaTool.java:402-411` | Validação matemática concluída; teleporte físico via host (FORK-32) |

---

## 2. Arco Elétrico (Electric Bow, `ESPEC_equipamento.md` Seção 4)

| Comportamento / Requisito | Estado | Arquivo:Linha no Rust | Origem no Java | Observações |
|---|---|---|---|---|
| **Identificação do Item**: registro `mekanism:electric_bow` | IMPLEMENTADO | `electric_bow.rs:10` | `ItemElectricBow.java:40-44` | Rarity RARE, setNoRepair(), stacksTo(1) |
| **Capacidade Máxima de Energia**: 120_000 J = 48.000 FE | IMPLEMENTADO | `electric_bow.rs:23` | `GearConfig.java:147` | `electricBowMaxEnergy` |
| **Taxa de Recarga**: 600 J/t = 240 FE/t | IMPLEMENTADO | `electric_bow.rs:26` | `GearConfig.java:148` | `electricBowChargeRate` |
| **Consumo por Flecha Normal**: 120 J = 48 FE | IMPLEMENTADO | `electric_bow.rs:29` | `GearConfig.java:149` | `electricBowEnergyUsage` |
| **Consumo por Flecha de Fogo (Flame)**: 1_200 J = 480 FE | IMPLEMENTADO | `electric_bow.rs:32` | `GearConfig.java:150` | `electricBowEnergyUsageFire` |
| **Alternância de Modo**: componente `electric_bow_mode` | IMPLEMENTADO | `electric_bow.rs:55-60` | `ItemElectricBow.java:149-156` | Alterna booleano (padrão false) |
| **Encantamento Flame I Injetado**: nível 1 quando ativo | IMPLEMENTADO | `electric_bow.rs:96-102` | `ItemElectricBow.java:86-93` | Adiciona efeito Flame sem livros |
| **Bloqueio de Disparo em Sobrevivência se Sem Energia** | IMPLEMENTADO | `electric_bow.rs:69-77` | `ItemElectricBow.java:53-62` | Permite disparo livre no modo criativo |
| **Dedução de Energia ao Atirar** | IMPLEMENTADO | `electric_bow.rs:79-92` | `ItemElectricBow.java:64-76` | Deduz 48 ou 480 FE apenas em sobrevivência |
| **Hook de Disparo no Servidor Horizon** | PARCIAL-por-FORK | `electric_bow.rs:188-195` | `ItemElectricBow.java:64-76` | Disparo e manipulação de entidade via host (FORK-33) |

---

## 3. Elytra Reforçada com HDPE (HDPE Elytra, `ESPEC_equipamento.md` Seção 5)

| Comportamento / Requisito | Estado | Arquivo:Linha no Rust | Origem no Java | Observações |
|---|---|---|---|---|
| **Identificação do Item**: registro `mekanism:hdpe_elytra` | IMPLEMENTADO | `hdpe_elytra.rs:11` | `ItemHDPEElytra.java:10` | Rarity RARE (`MekanismItems.java:184`) |
| **Durabilidade Máxima**: 648 (1,5× a vanilla 432) | IMPLEMENTADO | `hdpe_elytra.rs:20` | `MekanismItems.java:184` | `432 * 3 / 2 = 648` exato |
| **Slot de Equipamento**: Peitoral (`chest`) | IMPLEMENTADO | `hdpe_elytra.rs:29` | `ItemHDPEElytra.java:19` | `EquipmentSlot.CHEST` |
| **Item Válido de Reparo na Bigorna**: `mekanism:hdpe_sheet` | IMPLEMENTADO | `hdpe_elytra.rs:26, 85-87` | `ItemHDPEElytra.java:24` | `isValidRepairItem` |
| **Taxa de Reparo por Folha de HDPE**: 162 pts (25%) | IMPLEMENTADO | `hdpe_elytra.rs:35, 91-107` | Regras vanilla de reparo | Repara 162 de dano por HDPE Sheet |
| **Desgaste de Voo (1 pt / 20 ticks)** | IMPLEMENTADO | `hdpe_elytra.rs:32, 69-83` | Física vanilla de elytra | Reduz durabilidade proporcionalmente ao voo |
| **Estado Quebrado (Durabilidade <= 1)** | IMPLEMENTADO | `hdpe_elytra.rs:59-66` | Mecânica vanilla de elytra | Impede voo quando dano chega a 647 sem sumir com o item |
| **Hook de Voo e Desgaste Contínuo no Servidor** | PARCIAL-por-FORK | `hdpe_elytra.rs:165-172` | `ItemHDPEElytra.java` | Verificação do peitoral e voo via host (FORK-34) |

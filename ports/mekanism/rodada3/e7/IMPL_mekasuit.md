# Relatório de Implementação: MekaSuit e Sistema de Módulos (IMPL_mekasuit)

Este relatório mapeia cada comportamento e especificação técnica do MekaSuit e de seus módulos associados contra o código-fonte Java oficial (`Mekanism 1.21.1-10.7.19.85`) e o código implementado no port Rust (`port/mod/src/modules.rs` e `port/mod/src/mekasuit.rs`).

---

## 1. Peças de Armadura e Propriedades Base

| Comportamento | Estado | Rust (`port/mod/src/`) | Java (`java/`) | Observações |
|---|---|---|---|---|
| Registros das 4 peças (`mekasuit_helmet`, `mekasuit_bodyarmor`, `mekasuit_pants`, `mekasuit_boots`) | IMPLEMENTADO | `mekasuit.rs:16-19` | `MekanismItems.java:342-345` | Identificadores exatos com namespace `mekanism:`. |
| Defesa equivalente a Netherite (Capacete 3, Peitoral 8, Calças 6, Botas 3) | IMPLEMENTADO | `mekasuit.rs:114-154` | `MekanismStartupConfig.java:62-73` | Valores conferidos e validados em teste unitário. |
| Armor Toughness = 3.0 | IMPLEMENTADO | `mekasuit.rs:118,129,140,151` | `MekanismStartupConfig.java:74-76` | Equivalente integral à armadura de Netherite vanilla. |
| Knockback Resistance = 0.1 | IMPLEMENTADO | `mekasuit.rs:119,130,141,152` | `MekanismStartupConfig.java:77-79` | 10% de resistência a recuo por peça. |
| Indestrutibilidade (`setNoRepair()`, dano 0) | IMPLEMENTADO | `mekasuit.rs:107` | `ItemMekaSuitArmor.java:99,134-137` | `damageItem` sempre retorna 0; durabilidade infinita. |
| Imunidade a fogo/lava do item (`burnsInFire = false`) | IMPLEMENTADO | `mekasuit.rs:107` | `ItemMekaSuitArmor.java:99` | Item não queima ao cair em fogo ou lava. |
| Neutralidade de Piglins (`makesPiglinsNeutral = true`) | IMPLEMENTADO | `mekasuit.rs:108` | `ItemMekaSuitArmor.java:161-163` | Todas as 4 peças mantêm Piglins pacificados. |
| Máscara de Enderman (`isEnderMask = true` no Capacete) | IMPLEMENTADO | `mekasuit.rs:109` | `ItemMekaSuitArmor.java:166-168` | Olhar nos olhos de Endermen não atrai agressão. |
| Caminhada em neve fofa (`canWalkOnPowderedSnow` nas Botas) | IMPLEMENTADO | `mekasuit.rs:110` | `ItemMekaSuitArmor.java:171-173` | Impede afundamento em blocos de powdered snow. |
| Barra de energia visível e proporcional | IMPLEMENTADO | `mekasuit.rs:111` | `ItemMekaSuitArmor.java:176-184` | Largura da barra calculada como `round(13.0 * energy / max_energy)`. |

---

## 2. Armazenamento de Energia e Taxas de Recarga

| Comportamento | Estado | Rust (`port/mod/src/`) | Java (`java/`) | Observações |
|---|---|---|---|---|
| Capacidade Base por peça: 16.000.000 J = 6.400.000 FE | IMPLEMENTADO | `mekasuit.rs:22` | `GearConfig.java:249` | Taxa exata oficial `fe = J * 2 / 5`. |
| Taxa de Recarga Base: 100.000 J/t = 40.000 FE/t | IMPLEMENTADO | `mekasuit.rs:25` | `GearConfig.java:250` | Recarga por peça por tick. |
| Recarga do Inventário: 10.000 J/t = 4.000 FE/t | IMPLEMENTADO | `mekasuit.rs:28` | `GearConfig.java:251` | Usado pelo `ChargeDistributionUnit`. |
| Multiplicação por `EnergyUnit`: `base * 2^installed` | IMPLEMENTADO | `mekasuit.rs:244-255` | `ModuleEnergyUnit.java:33` | 1 unid = 2×, até 8 unid = 256× (1.638.400.000 FE por peça; 6.553.600.000 FE conjunto). |
| Multiplicação da Taxa de Carga por `EnergyUnit` | IMPLEMENTADO | `mekasuit.rs:251-255` | `ModuleEnergyUnit.java:33` | Até 10.240.000 FE/t por peça com 8 unidades. |

---

## 3. Tanques Químicos e de Fluidos Internos

| Comportamento | Estado | Rust (`port/mod/src/`) | Java (`java/`) | Observações |
|---|---|---|---|---|
| Tanque de Pasta Nutricional (Capacete): 128.000 mB | IMPLEMENTADO | `mekasuit.rs:34,266-272` | `GearConfig.java:256`, `ItemMekaSuitArmor.java:103` | Só ativo quando `NutritionalInjectionUnit` instalado. |
| Taxa de transferência da Pasta Nutricional: 256 mB/t | IMPLEMENTADO | `mekasuit.rs:37` | `GearConfig.java:258` | Taxa de enchimento do tanque. |
| Tanque de Hidrogênio (Peitoral): 24.000 mB por Jetpack Unit | IMPLEMENTADO | `mekasuit.rs:40,258-264` | `GearConfig.java:260`, `ItemMekaSuitArmor.java:113` | Escala com módulos instalados: 1=24k, 2=48k, 3=72k, 4=96k mB. |
| Taxa de transferência de Hidrogênio: 256 mB/t | IMPLEMENTADO | `mekasuit.rs:43` | `GearConfig.java:262` | Taxa de abastecimento químico. |
| Ventilação de excesso químico ao desinstalar Jetpack | IMPLEMENTADO | `modules.rs:835-845` | `ModuleJetpackUnit.java:71-85` | Reduz gás armazenado ao novo teto máximo. |

---

## 4. Modelo de Dano e Absorção de Dano

| Comportamento | Estado | Rust (`port/mod/src/`) | Java (`java/`) | Observações |
|---|---|---|---|---|
| Ratios de absorção base: Capacete 0.15, Peitoral 0.40, Calças 0.30, Botas 0.15 | IMPLEMENTADO | `mekasuit.rs:120,131,142,153` | `ItemMekaSuitArmor.java:105,115,120,125` | Soma 1.0 (100% de absorção com o terno completo). |
| Custo de absorção de dano comum: 100.000 J = 40.000 FE/pt | IMPLEMENTADO | `mekasuit.rs:46,400-470` | `GearConfig.java:265` | Consome energia de cada peça proporcionalmente. |
| Absorção de dano mágico via `InhalationPurificationUnit`: 400 FE/pt | IMPLEMENTADO | `mekasuit.rs:49,340-365` | `GearConfig.java:266`, `ModuleInhalationPurificationUnit.java:23` | Absorve dano de `IS_PREVENTABLE_MAGIC`. |
| Absorção de dano de fogo via `GeothermalGeneratorUnit`: 0 energia | IMPLEMENTADO | `mekasuit.rs:367-375` | `GeneratorsGearConfig.java:28`, `ModuleGeothermalGeneratorUnit.java:41` | Absorve até 80% do dano de fogo sem gastar energia. |
| Absorção de dano de queda pelas Botas: 50 J = 20 FE | IMPLEMENTADO | `mekasuit.rs:52,488-500` | `GearConfig.java:267`, `CommonPlayerTickHandler.java:301` | 100% de absorção de queda custando 20 FE fixos por evento. |
| Absorção proporcional quando a energia é insuficiente | IMPLEMENTADO | `mekasuit.rs:360-365,420-435` | `ItemMekaSuitArmor.java:501-507` | Absorve `ratio * (energia_disponivel / energia_necessaria)`. |
| Verificação de bypass (dano técnico ou bypass de armadura sem suporte) | IMPLEMENTADO | `mekasuit.rs:385-395` | `ItemMekaSuitArmor.java:448-450` | Respeita regras do Minecraft e tags do Mekanism. |
| Hook de absorção injetado no evento de dano | PARCIAL-por-FORK | `mekasuit.rs:650-660` | `ItemMekaSuitArmor.java:408` | Lógica pura pronta; aguarda hook `FORK-27` (`on-player-damaged`). |

---

## 5. Dissipação Laser e Blindagem de Radiação

| Comportamento | Estado | Rust (`port/mod/src/`) | Java (`java/`) | Observações |
|---|---|---|---|---|
| Dissipação Laser: Capacete 15%, Peitoral 30%, Calças 18.75%, Botas 11.25% | IMPLEMENTADO | `mekasuit.rs:121,132,143,154` | `ItemMekaSuitArmor.java:106,116,121,126` | Dissipa 75% da energia total do feixe laser. |
| Refração Laser: Capacete 20%, Peitoral 40%, Calças 25%, Botas 15% | IMPLEMENTADO | `mekasuit.rs:122,133,144,155` | `ItemMekaSuitArmor.java:107,117,122,127` | Refrata 100% dos 25% restantes de energia. |
| Blindagem de Radiação (Hazmat equivalente): 25%, 40%, 20%, 15% | IMPLEMENTADO | `mekasuit.rs:123,134,145,156` | `ItemHazmatSuitArmor.java:21-29` | 100% de imunidade com as 4 peças blindadas. |

---

## 6. Módulos do Capacete (Helmet)

| Módulo | Estado | Rust (`port/mod/src/`) | Java (`java/`) | Observações |
|---|---|---|---|---|
| `electrolytic_breathing_unit` (max 4) | IMPLEMENTADO | `mekasuit.rs:530-580` | `ModuleElectrolyticBreathingUnit.java` | 2^n embaixo d'água / 2^(n-1) na chuva; recupera ar e enche hidrogênio. |
| `inhalation_purification_unit` (max 1) | IMPLEMENTADO | `mekasuit.rs:610-638` | `ModuleInhalationPurificationUnit.java` | Acelera efeitos negativos em 10×; 16.000 FE por tick de poção. |
| `vision_enhancement_unit` (max 4) | IMPLEMENTADO | `mekasuit.rs:64, modules.rs:137` | `ModuleVisionEnhancementUnit.java` | Visão noturna a 200 FE/t; desligado por padrão com toggle de modo. |
| `nutritional_injection_unit` (max 1) | IMPLEMENTADO | `mekasuit.rs:582-608` | `ModuleNutritionalInjectionUnit.java` | Alimenta automaticamente a 50 mB/fome e 8.000 FE por nível de fome. |
| `solar_recharging_unit` (max 8) | IMPLEMENTADO | `mekasuit.rs:502-528` | `ModuleSolarRechargingUnit.java` | 200 FE/t base * curva de temp/umidade * sol * (0.2 chuva) * count. |

---

## 7. Módulos do Peitoral (Bodyarmor)

| Módulo | Estado | Rust (`port/mod/src/`) | Java (`java/`) | Observações |
|---|---|---|---|---|
| `jetpack_unit` (max 4) | IMPLEMENTADO | `mekasuit.rs:680-725` | `ModuleJetpackUnit.java` | Modos NORMAL/HOVER/DISABLED; empuxo 0.15 * mult; combustível ceil(mult) mB/t. |
| `elytra_unit` (max 1) | IMPLEMENTADO | `mekasuit.rs:727-742` | `ModuleElytraUnit.java` | Voo planado estilo Elytra; consome 12.800 FE a cada segundo (20 ticks). |
| `gravitational_modulating_unit` (max 1) | IMPLEMENTADO | `mekasuit.rs:744-770` | `ModuleGravitationalModulatingUnit.java` | Voo criativo; 400 FE/t base; 1.600 FE/t modo boost com aceleração direcional. |
| `charge_distribution_unit` (max 1) | IMPLEMENTADO | `mekasuit.rs:772-820` | `ModuleChargeDistributionUnit.java` | Equilibra energia entre peças da armadura e recarrega inventário (4.000 FE/t). |
| `dosimeter_unit` (max 1) | PARCIAL-por-FORK | `modules.rs:144, mekasuit.rs:645` | `ModuleDosimeterUnit.java` | Métricas prontas; aguarda widget dinâmico de HUD (`FORK-30`). |
| `geiger_unit` (max 1) | PARCIAL-por-FORK | `modules.rs:146, mekasuit.rs:645` | `ModuleGeigerUnit.java` | Métricas prontas; aguarda widget dinâmico de HUD (`FORK-30`). |

---

## 8. Módulos das Calças (Pants)

| Módulo | Estado | Rust (`port/mod/src/`) | Java (`java/`) | Observações |
|---|---|---|---|---|
| `locomotive_boosting_unit` (max 4) | IMPLEMENTADO | `mekasuit.rs:772-800` | `ModuleLocomotiveBoostingUnit.java` | Aceleração de corrida (0.05 a 0.50); reduzido em 5× no ar e na água; 40 FE/t base. |
| `gyroscopic_stabilization_unit` (max 1) | IMPLEMENTADO | `modules.rs:192, mekasuit.rs:60` | `GyroscopicStabilizationUnit.java` | Velocidade de mineração submersa (+1.0) e cancela penalidade aérea de 5×. |
| `hydrostatic_repulsor_unit` (max 4) | IMPLEMENTADO | `mekasuit.rs:835-855` | `ModuleHydrostaticRepulsorUnit.java` | Eficiência de movimento na água; swim boost (+1.0) com 4 unidades (200 FE/t). |
| `motorized_servo_unit` (max 5) | IMPLEMENTADO | `modules.rs:204, mekasuit.rs:62` | `MotorizedServoUnit.java` | Velocidade agachado (Swift Sneak) +0.15 por unidade até 0.75. |
| `geothermal_generator_unit` (max 8) | IMPLEMENTADO | `mekasuit.rs:815-833` | `ModuleGeothermalGeneratorUnit.java` | Recarrega em lava/fogo (4 FE/t * unid * graus); 800 FE/t por unidade no fogo. |

---

## 9. Módulos das Botas (Boots)

| Módulo | Estado | Rust (`port/mod/src/`) | Java (`java/`) | Observações |
|---|---|---|---|---|
| `hydraulic_propulsion_unit` (max 4) | IMPLEMENTADO | `mekasuit.rs:802-825` | `ModuleHydraulicPropulsionUnit.java` | Step-assist até 2.0 blocos; pulo impulsionado (0.5 a 5.0) com custo de 400 FE/0.1. |
| `magnetic_attraction_unit` (max 4) | IMPLEMENTADO | `mekasuit.rs:857-885` | `ModuleMagneticAttractionUnit.java` | Atrai itens em raio de 4 + range (até 14 blocos); custo 100 FE * range por item. |
| `frost_walker_unit` (max 2) | IMPLEMENTADO | `modules.rs:228, mekasuit.rs:68` | `MekanismModules.java:228` | Concede encantamento Frost Walker I ou II; congela água ao caminhar. |
| `soul_surfer_unit` (max 3) | IMPLEMENTADO | `modules.rs:230, mekasuit.rs:70` | `SoulSurferUnit.java` | Eficiência de movimento em soul sand/soil (+0.33 por unidade até 1.0). |

---

## 10. Módulos Compartilhados e Exclusividade

| Módulo | Estado | Rust (`port/mod/src/`) | Java (`java/`) | Observações |
|---|---|---|---|---|
| `energy_unit` (max 8 por peça) | IMPLEMENTADO | `modules.rs:60-70, mekasuit.rs:245` | `ModuleEnergyUnit.java` | Multiplica capacidade e velocidade de recarga por 2^n. |
| `color_modulation_unit` (max 1) | IMPLEMENTADO | `modules.rs:64, mekasuit.rs:75` | `ModuleColorModulationUnit.java` | Configuração de cor ARGB customizada para tintura visual. |
| `laser_dissipation_unit` (max 1) | IMPLEMENTADO | `modules.rs:68, mekasuit.rs:121` | `MekanismModules.java:68` | Ativa dissipação e refração de laser na peça. |
| `radiation_shielding_unit` (max 1) | IMPLEMENTADO | `modules.rs:70, mekasuit.rs:123` | `MekanismModules.java:70` | Ativa blindagem radioativa na peça. |
| Sistema de exclusividade (`OVERRIDE_JUMP`, etc.) | IMPLEMENTADO | `modules.rs:850-880` | `ModuleContainer.java:189-229` | Desativa automaticamente módulos conflitantes ao ligar um módulo exclusivo. |
| Serialização de componentes (`mekanism:modules`) | IMPLEMENTADO | `modules.rs:920-990` | `ModuleContainer.java:45-60` | Codificação e decodificação estruturada round-trip em SNBT/JSON. |

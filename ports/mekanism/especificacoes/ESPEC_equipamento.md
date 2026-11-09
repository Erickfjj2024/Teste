# Especificação Técnica: Equipamentos Mekanism (Meka-Tool, MekaSuit, Robit, Armaduras e Utilitários)

Documento de especificação técnica e comportamental gerado para o port do Mekanism 1.21.1-10.7.19.85 para Horizon (Rust / WASM).
Todas as conversões de energia seguem estritamente a taxa oficial: **1 FE = 2.5 Joules** (`fe = J * 2 / 5`).

---

## 1. Meka-Tool e Módulos Associados

### 1.1 Identificação e Classes Java
- **Registro**: `mekanism:meka_tool`
- **Item**: `mekanism.common.item.gear.ItemMekaTool` (`ItemMekaTool.java`)
- **Radial Menu**: `mekanism:meka_tool` (`IRadialModuleContainerItem.java`)
- **Módulos Suportados**:
  1. `mekanism:energy_unit` (`ModuleEnergyUnit.java`, max 8)
  2. `mekanism:attack_amplification_unit` (`ModuleAttackAmplificationUnit.java`, max 4)
  3. `mekanism:excavation_escalation_unit` (`ModuleExcavationEscalationUnit.java`, max 4)
  4. `mekanism:blasting_unit` (`ModuleBlastingUnit.java`, max 4)
  5. `mekanism:vein_mining_unit` (`ModuleVeinMiningUnit.java`, max 4)
  6. `mekanism:farming_unit` (`ModuleFarmingUnit.java`, max 4)
  7. `mekanism:shearing_unit` (`ModuleShearingUnit.java`, max 1)
  8. `mekanism:teleportation_unit` (`ModuleTeleportationUnit.java`, max 1)
  9. `mekanism:silk_touch_unit` (`SimpleEnchantmentAwareModule.java`, max 1)
  10. `mekanism:fortune_unit` (`SimpleEnchantmentAwareModule.java`, max 3)

### 1.2 Propriedades e Data Components (1.21.1)
- **Stack size**: 1 (não empilhável).
- **Rarity**: EPIC. Unburnable (`burnsInFire = false`).
- **Reparo e Encantamento**: `setNoRepair()`, `isEnchantable() = false` (não aceita encantamentos normais de bigorna/mesa; módulos substituem encantamentos).
- **Data Components**:
  - `minecraft:tool`: Regras de quebra de bloco com negação para a tag `mekanism:incorrect_for_meka_tool` e permissão para qualquer bloco (`AnyHolderSet`).
  - `mekanism:attached_energy`: Lista contendo o container primário de energia.
  - `mekanism:modules`: Contêiner de módulos instalados (`ModuleContainer`), mapeando IDs de módulo para instâncias com quantidade instalada (`installed`) e dados de configuração.
  - `mekanism:active_radial_mode`: Modo radial selecionado para cada módulo radial.

### 1.3 Constantes, Configurações e Conversão J → FE
- **Capacidade Base de Energia**: `16_000_000 J` = **6.400.000 FE** (`mekaToolBaseEnergyCapacity`).
  - Cada `energy_unit` instalado multiplica a capacidade por 2: com 8 unidades, capacidade máxima = `16_000_000 * (1 + 8) = 144_000_000 J` = **57.600.000 FE**.
- **Taxa de Recarga Base**: `100_000 J/t` = **40.000 FE/t** (`mekaToolBaseChargeRate`).
- **Dano Base**: `4` (`mekaToolBaseDamage`).
- **Velocidade de Ataque**: `-2.4` (`mekaToolAttackSpeed`), resultando em taxa final de 1.6 ataques/s.
- **Eficiência Base de Mineração**: `4.0` (`mekaToolBaseEfficiency`).
  - Cada `excavation_escalation_unit` adiciona modificador de velocidade:
    - Modo `OFF`: eficiência base 4.0.
    - Modo `SLOW`: eficiência 8.0.
    - Modo `NORMAL`: eficiência 20.0.
    - Modo `FAST`: eficiência 40.0.
    - Modo `SUPER_FAST`: eficiência 80.0.
    - Modo `EXTREME`: eficiência 160.0.
- **Consumos de Energia**:
  - Quebra de Bloco Base: `10 J` = **4 FE** (`mekaToolEnergyUsage`).
  - Com Toque Suave (`silk_touch_unit` ativo): `100 J` = **40 FE** (`mekaToolEnergyUsageSilk`).
  - Ataque a Entidade: `2_000 J` = **800 FE** (`mekaToolEnergyUsageWeapon`).
  - Dano Amplificado (`attack_amplification_unit`): `2_000 J` base + `1_000 J` por unidade adicional por golpe.
  - Uso como Enxada: `10 J` = **4 FE** (`mekaToolEnergyUsageHoe`).
  - Uso como Pá (criar caminho): `10 J` = **4 FE** (`mekaToolEnergyUsageShovel`).
  - Uso como Machado (descascar tronco): `10 J` = **4 FE** (`mekaToolEnergyUsageAxe`).
  - Tosa de Entidade (tosquiar ovelha): `10 J` = **4 FE** (`mekaToolEnergyUsageShearEntity`).
  - Quebra de Bloco com Tosa (teia/folhas): `10 J` = **4 FE** (`mekaToolEnergyUsageShearTrim`).
  - Teletransporte (`teleportation_unit`): `1_000 J` base por 10 blocos de distância (`Math.ceil(1_000 * distance / 10.0)` Joules = `Math.ceil(400 * distance / 10.0)` FE). Alcance máximo: `100 blocos` (`mekaToolMaxTeleportReach`).

### 1.4 Mecânicas de Mineração e Operação
1. **Blasting Unit**:
   - Modos: `OFF`, `LOW (3x3)`, `MED (5x5)`, `HIGH (7x7)`, `ULTRA (9x9)`.
   - Quebra em plano perpendicular ao vetor da face clicada pelo jogador.
2. **Vein Mining Unit**:
   - Modos de alcance: 1 módulo = 8 blocos, 2 = 16, 3 = 32, 4 = 64 blocos adjacentes da mesma tag/tipo.
   - Configuração `extendedMode`: permite estender mineração para além de minérios (madeiras, pedras correlatas).
3. **Farming Unit**:
   - Aumenta raio de colheita/aragem em área: 1x1, 3x3, 5x5, 7x7, 9x9.
4. **Teletransporte**:
   - Clique direito mira raycast até 100 blocos. Verifica blocos vazios na cabeça e pés (`pos.above()` e `pos.above(2)` livres). Se passageiro, desmonta e teleporta. Reseta `fallDistance = 0`. Dispara som `SoundEvents.PLAYER_TELEPORT` e pacote `PacketPortalFX`.

### 1.5 Receita (JSON Datagen)
- **Forma**: Shaped 3x3 na Workbench / Crafting.
- **Pattern**:
  - `C o C`
  - `P M P`
  - `A E A`
- **Chaves**:
  - `C`: `mekanism:ultimate_control_circuit`
  - `o`: `mekanism:configurator`
  - `P`: `mekanism:hdpe_sheet`
  - `M`: `mekanism:atomic_disassembler`
  - `A`: `mekanism:pellet_polonium`
  - `E`: `mekanism:basic_induction_cell`

### 1.6 Capacidades e Requisitos Horizon
- **Capacidades**: `IEnergyStorage` / `IMekanismStrictEnergyHandler`, `IModuleContainer`.
- **Requisitos**: `FORK-23` (Data components e serialização de módulos), `FORK-27` (Sistema modular de equipamentos e UI radial), `FORK-31` (Raycast avançado e escavação em área 3x3 a 9x9).

---

## 2. MekaSuit (4 Peças) e Módulos Associados

### 2.1 Identificação e Classes Java
- **Registros**:
  - `mekanism:mekasuit_helmet` (`ItemMekaSuitArmor.java`, Helmet)
  - `mekanism:mekasuit_bodyarmor` (`ItemMekaSuitArmor.java`, Chestplate)
  - `mekanism:mekasuit_pants` (`ItemMekaSuitArmor.java`, Leggings)
  - `mekanism:mekasuit_boots` (`ItemMekaSuitArmor.java`, Boots)
- **Material de Armadura**: `MekanismArmorMaterials.MEKASUIT`
  - Defesa: Netheite equivalente (Capacete 3, Peitoral 8, Calças 6, Botas 3).
  - Toughness: `3.0` (`mekaSuitToughness`).
  - Knockback Resistance: `0.1` (`mekaSuitKnockbackResistance`).
- **Módulos do Capacete**:
  - `electrolytic_breathing_unit` (max 4, produz oxigênio a partir de água estocada).
  - `inhalation_purification_unit` (max 1, neutraliza efeitos negativos/poções).
  - `vision_enhancement_unit` (max 4, concede visão noturna estável).
  - `nutritional_injection_unit` (max 1, estoca Nutritional Paste e alimenta jogador automaticamente).
  - `solar_recharging_unit` (max 8, gera energia via sol: 500 J/t base * 0.2 na chuva).
- **Módulos do Peitoral**:
  - `jetpack_unit` (max 4, propulsão a hidrogênio ou energia, modos NORMAL/HOVER/DISABLED).
  - `elytra_unit` (max 1, planar estilo elytra consumindo energia: 32.000 J/t = 12.800 FE/t).
  - `gravitational_modulating_unit` (max 1, voo criativo completo; custo 1.000 J/t = 400 FE/t).
  - `charge_distribution_unit` (max 1, distribui carga entre peças e inventário).
  - `dosimeter_unit` (max 1, exibe dose acumulada de radiação no HUD).
  - `geiger_unit` (max 1, exibe nível de radiação ambiental no HUD).
- **Módulos da Calça**:
  - `locomotive_boosting_unit` (max 4, aceleração de corrida até +100%).
  - `gyroscopic_stabilization_unit` (max 1, neutraliza rotação de câmera/empurrão).
  - `hydrostatic_repulsor_unit` (max 4, velocidade de nado ampliada).
  - `motorized_servo_unit` (max 5, velocidade de caminhada acelerada).
- **Módulos da Bota**:
  - `hydraulic_propulsion_unit` (max 4, pulo ampliado e subida de blocos step-assist até 2 blocos).
  - `magnetic_attraction_unit` (max 4, atrai itens a até 16 blocos).
  - `frost_walker_unit` (max 2, congela água ao caminhar).
  - `soul_surfer_unit` (max 3, acelera em blocos de alma sem lentidão).
  - `geothermal_generator_unit` (max 8, recarrega em lava/fogo: 10 J/grau * installedCount).
- **Módulos Compartilhados**:
  - `energy_unit` (max 8 por peça).
  - `color_modulation_unit` (personalização de cor RGBA da armadura).
  - `laser_dissipation_unit` (dissipa até 75% e refrata o restante do dano de laser).
  - `radiation_shielding_unit` (proteção 100% contra radiação acumulada se as 4 peças tiverem a unidade).

### 2.2 Constantes e Absorção de Dano
- **Capacidade de Energia por Peça**: `16_000_000 J` = **6.400.000 FE** base (até **57.600.000 FE** com 8 Energy Units). Total do conjunto completo: **230.400.000 FE**.
- **Taxa de Carga Base**: `100_000 J/t` = **40.000 FE/t** por peça.
- **Taxa de Recarga do Inventário**: `10_000 J/t` = **4.000 FE/t** (`mekaSuitInventoryChargeRate`).
- **Mecânica de Absorção de Dano**:
  - Dano Genérico: Cada ponto de dano absorvido custa `100_000 J` = **40.000 FE** (`mekaSuitEnergyUsageDamage`).
  - Dano Mágico: Absorção de até 100%, consumindo `1_000 J` = **400 FE** por ponto (`mekaSuitEnergyUsageMagicReduce`).
  - Dano de Queda: Redução de 100% do dano de queda consumindo `50 J` = **20 FE** por evento (`mekaSuitEnergyUsageFall`).
  - Pulo Impulsionado: `1_000 J` = **400 FE** por pulo (`mekaSuitBaseJumpEnergyUsage`).
  - Efeito Negativo Purificado: `40_000 J` = **16.000 FE** por tick de purificação (`mekaSuitEnergyUsagePotionTick`).
  - Sprint Boost: `100 J/t` = **40 FE/t** (`mekaSuitEnergyUsageSprintBoost`).
  - Visão Noturna: `500 J/t` = **200 FE/t** (`mekaSuitEnergyUsageVisionEnhancement`).
  - Nado Repulsor: `500 J/t` = **200 FE/t** (`mekaSuitEnergyUsageHydrostaticRepulsion`).
  - Injeção Nutricional: `20_000 J` = **8.000 FE** por injeção (`mekaSuitEnergyUsageNutritionalInjection`). Capacidade interna: 128 baldes (128.000 mB de pasta nutricional), taxa de transferência 256 mB/t.
  - Atração Magnética: `250 J/t` = **100 FE/t** (`mekaSuitEnergyUsageItemAttraction`).
  - Voo Gravitacional: `1_000 J/t` = **400 FE/t** base (`mekaSuitEnergyUsageGravitationalModulation`).

### 2.3 Receitas (JSON Datagen)
Cada peça compartilha a mesma estrutura 3x3, variando a peça de Netherite central:
- Linha 1: `HDPE Sheet`, `Ultimate Control Circuit`, `HDPE Sheet`
- Linha 2: `HDPE Sheet`, `<Peça de Netherite>`, `HDPE Sheet`
- Linha 3: `Pellet Polonium`, `Basic Induction Cell`, `Pellet Polonium`

---

## 3. Robit (Entidade, Automação, GUIs e Customização)

### 3.1 Identificação e Classes Java
- **Registro de Entidade**: `mekanism:robit` (`MekanismEntityTypes.ROBIT`)
- **Registro de Item**: `mekanism:robit` (`ItemRobit.java`)
- **Entidade**: `mekanism.common.entity.EntityRobit` (`EntityRobit.java`)
- **Contêineres e Telas**:
  - `MainRobitContainer` / `GuiRobitMain`: Tela principal, status de bateria, modo seguir e recolher.
  - `InventoryRobitContainer` / `GuiRobitInventory`: Inventário de carga de 27 slots (3x9).
  - `SmeltingRobitContainer` / `GuiRobitSmelting`: Fornalha interna rápida (100 ticks).
  - `CraftingRobitContainer` / `GuiRobitCrafting`: Bancada de trabalho 3x3 integrada.
  - `RepairRobitContainer` / `GuiRobitRepair`: Bigorna mecânica integrada.
  - `GuiRobitRename`: Janela popup de renomeação.
  - `GuiRobitSkinSelect`: Seletor com barra de rolagem de skins cosméticas.

### 3.2 Estatísticas, Energia e Comportamento
- **Vida Máxima**: `1.0 HP` (invulnerável a morte por dano normal; absorve dano via bateria).
  - Ao sofrer dano: consome `Math.min(bateria, 1_000 * dano)` Joules (**400 FE * dano**); mantém a vida em 100%. Se a bateria zerar, desliga e retorna à base.
- **Velocidade de Movimento**: `0.3F`.
- **Capacidade Máxima de Energia**: `100_000 J` = **40.000 FE** (`MAX_ENERGY`).
- **Custo de Movimento**: `Math.ceil(1.5 * sqrt(distância))` Joules por tick de deslocamento.
- **Recolhimento de Itens**: AABB expandida em 1.5 blocos ao redor. Coleta itens jogados no chão inserindo nos 27 slots internos.
- **Fornalha Interna**:
  - Tempo de queima: `100 ticks` (5 segundos, metade dos 200 ticks de uma fornalha comum).
  - Consumo de energia: `50 J/t` = **20 FE/t**.
- **Retorno ao Chargepad (Home)**:
  - Salva coordenadas dimensionais (`GlobalPos`). Quando a bateria zera ou o jogador manda ir para casa, executa teletransporte dimensional para o `TileEntityChargepad` configurado.
  - Se o Chargepad for destruído ou inexistente, o Robit cai como item no chão (`drop()`).

### 3.3 Inventário e Layout de Slots (GUI Coordinates)
- **Inventário Principal (3x9)**: 27 slots de `(X=8 + col*18, Y=18 + row*18)`.
- **Slot de Energia**: slot 28 em `(X=153, Y=17)` (aceita Energy Tablet ou itens recarregáveis).
- **Slot de Entrada de Fundição**: slot 29 em `(X=51, Y=35)`.
- **Slot de Saída de Fundição**: slot 30 em `(X=116, Y=35)`.

### 3.4 Data Components e Persistência do Item
Ao ser recolhido com Configurator (Shift+Right-Click) ou quebrado:
- `mekanism:attached_energy`: Bateria atual persistida.
- `mekanism:container_items`: Conteúdo dos 27 slots + smelting slots.
- `mekanism:robit_name`: Nome customizado (String Component).
- `mekanism:robit_skin`: ResourceLocation da skin (`mekanism:base`, `pride_...`, etc.).
- `mekanism:security`: UUID do dono e modo de segurança (PUBLIC / PRIVATE / TRUSTED).

### 3.5 Receita (JSON Datagen)
- Pattern:
  - `  S  `
  - `E A E`
  - `O P O`
- Chaves:
  - `S`: `mekanism:ingot_steel`
  - `E`: `mekanism:energy_tablet`
  - `A`: `mekanism:alloy_atomic`
  - `O`: `mekanism:ingot_refined_obsidian`
  - `P`: `mekanism:personal_storage` (Chest ou Barrel pessoal)

---

## 4. Arco Elétrico (Electric Bow)

### 4.1 Identificação e Classes
- **Registro**: `mekanism:electric_bow`
- **Classe**: `mekanism.common.item.gear.ItemElectricBow` (`ItemElectricBow.java`)
- **Herança**: `net.minecraft.world.item.BowItem`

### 4.2 Configurações de Energia e Disparo
- **Capacidade Máxima**: `120_000 J` = **48.000 FE** (`electricBowMaxEnergy`).
- **Taxa de Recarga**: `600 J/t` = **240 FE/t** (`electricBowChargeRate`).
- **Consumo por Flecha Normal**: `120 J` = **48 FE** (`electricBowEnergyUsage`).
- **Consumo por Flecha de Fogo (Flame Mode)**: `1_200 J` = **480 FE** (`electricBowEnergyUsageFire`).
- **Alternância de Modo**:
  - Tecla de troca de modo (Shift + M ou tecla de Item Mode).
  - Data Component: `mekanism:electric_bow_mode` (Boolean, padrão `false`).
  - Quando `true`: injeta internamente o encantamento `Enchantments.FLAME` level 1 na flecha sem precisar de livro nem consumir lápis-lazúli.

### 4.3 Receita (JSON Datagen)
- Pattern:
  - `S A  `
  - `S   E`
  - `S A  `
- Chaves:
  - `S`: `minecraft:string`
  - `A`: `mekanism:alloy_infused`
  - `E`: `mekanism:energy_tablet`

---

## 5. Elytra Reforçada com HDPE (HDPE Elytra)

### 5.1 Identificação e Classes
- **Registro**: `mekanism:hdpe_elytra`
- **Classe**: `mekanism.common.item.gear.ItemHDPEElytra` (`ItemHDPEElytra.java`)
- **Herança**: `net.minecraft.world.item.ElytraItem`

### 5.2 Propriedades Técnicas
- **Durabilidade**: `648` (exatamente 1,5× a durabilidade da Elytra vanilla, que possui 432).
- **Raridade**: RARE.
- **Slot de Equipamento**: `EquipmentSlot.CHEST`.
- **Material de Reparo na Bigorna**: `mekanism:hdpe_sheet` (`isValidRepairItem`).
- **Física de Voo**: Comportamento idêntico ao glide vanilla com menor taxa de desgaste por segundo.

### 5.3 Receita (JSON Datagen)
- Pattern:
  - `A H A`
  - `H E H`
  - `H   H`
- Chaves:
  - `A`: `mekanism:alloy_atomic`
  - `H`: `mekanism:hdpe_sheet`
  - `E`: `minecraft:elytra`

---

## 6. Contador Geiger (Geiger Counter)

### 6.1 Identificação e Classes
- **Registro**: `mekanism:geiger_counter`
- **Classe**: `mekanism.common.item.ItemGeigerCounter` (`ItemGeigerCounter.java`)
- **Sons e Áudio**: `mekanism.client.sound.GeigerSound`, evento `mekanism:item.geiger_counter`.
- **HUD e Escala**: `mekanism.common.lib.radiation.RadiationScale` (NONE, ELEVATED, HIGH, EXTREME).

### 6.2 Comportamento e Interação
- **Stack size**: 1. Rarity: UNCOMMON.
- **Clique Direito (Server)**:
  - Lê a dose do chunk e do jogador via `RadiationManager.get().getRadiationLevelAndMaxMagnitude(player)`.
  - Envia mensagem no chat formatada com cor de severidade (`RadiationScale.getSeverityColor(magnitude)`):
    - `Radiation exposure: <valor> Sv/h` (formatado com 3 casas decimais e unidades automáticas: nSv/h, uSv/h, mSv/h, Sv/h).
  - Se `enableDecayTimers = true` e magnitude > baseline:
    - Envia estimativa de decaimento: `Estimated decay time: <horas>h <minutos>m`.
- **Áudio (Client)**:
  - Emite cliques estocásticos com frequência proporcional ao logaritmo da radiação atual no chunk.

### 6.3 Receita (JSON Datagen)
- Pattern:
  - `  I  `
  - `I C I`
  - `  I  `
- Chaves:
  - `I`: `mekanism:ingot_lead`
  - `C`: `mekanism:basic_control_circuit`

---

## 7. Conjuntos de Voo, Mergulho e Agilidade

### 7.1 Jetpack e Armored Jetpack
- **Registros**: `mekanism:jetpack`, `mekanism:jetpack_armored`
- **Classes**: `ItemJetpack.java`, `ItemArmoredJetpack.java`
- **Capacidade Química**: `24_000 mB` de Hidrogênio (`MekanismChemicals.HYDROGEN`).
- **Taxa de Abastecimento**: `16 mB/t` (`jetpackFillRate`).
- **Consumo de Combustível**: `1 mB/t` enquanto estiver subindo ou pairando.
- **Empuxo (Thrust)**: `0.15` base.
- **Modos**:
  - `NORMAL`: Impulso para cima ao segurar pulo (espaço). Reseta dano de queda.
  - `HOVER`: Mantém o jogador pairando em altitude constante se nenhuma tecla for pressionada.
  - `DISABLED`: Desligado.
- **Armored Jetpack**:
  - Acrescenta proteção equivalente a peitoral de aço/diamante: **Defesa 8**, **Toughness 2.0**, **Knockback Resistance 0.0**.

### 7.2 Máscara de Mergulho e Tanque de Oxigênio (Scuba Mask & Tank)
- **Registros**: `mekanism:scuba_mask`, `mekanism:scuba_tank`
- **Classes**: `ItemScubaMask.java`, `ItemScubaTank.java`
- **Tanque de Oxigênio**:
  - Armazena `24_000 mB` de Oxigênio (`MekanismChemicals.OXYGEN`).
  - Modo: `Flowing` (Ligado / Desligado).
- **Operação Conjunta**:
  - Quando a máscara está no elmo e o tanque ligado no peitoral com oxigênio disponível:
    - Consome `1 mB/t` base.
    - Se o ar do jogador estiver reduzido (`player.getAirSupply() < max`): consome `(max - airSupply)` mB de oxigênio instantaneamente e restaura a respiração a 100%.
    - Se o ar estiver cheio: acelera a eliminação de poções negativas (como veneno e fome) em 10× a velocidade normal.

### 7.3 Free Runners e Armored Free Runners
- **Registros**: `mekanism:free_runners`, `mekanism:free_runners_armored`
- **Classes**: `ItemFreeRunners.java`, `ItemArmoredFreeRunners.java`
- **Capacidade de Energia**: `64_000 J` = **25.600 FE** (`freeRunnerMaxEnergy`).
- **Taxa de Recarga**: `320 J/t` = **128 FE/t** (`freeRunnerChargeRate`).
- **Absorção de Queda**:
  - Reduz **100%** do dano de queda (`freeRunnerFallDamageRatio = 1.0`).
  - Consumo de energia: `50 J` = **20 FE** por queda evitada (`freeRunnerFallEnergyCost`).
- **Propriedades Únicas**:
  - Caminha sobre neve em pó sem afundar (`canWalkOnPowderedSnow = true`).
  - No modo `NORMAL`: adiciona modificador de atributo `Attributes.MOVEMENT_EFFICIENCY` (+1.0) e step-assist para subir degraus de 1 bloco sem pular.
- **Armored Free Runners**:
  - Defesa: **3**, Toughness: **2.0**, Knockback Resistance: **0.0**.

---

## 8. Requisitos de Fork e Extensões Horizon

Para que estas peças funcionem no Horizon com a fidelidade exata do Java, são exigidas as seguintes capacidades na API do ambiente:

| ID | Descrição do Requisito | Aplicação |
|---|---|---|
| `FORK-10` | Entidades customizadas com IA completa de navegação, carregamento de chunks e inventário serializado | `EntityRobit` (IA de seguir, recolher itens e teletransporte) |
| `FORK-13` | Sons estocásticos de radiação e cálculo contínuo de contaminação por chunk | `ItemGeigerCounter`, `GeigerSound` |
| `FORK-23` | Suporte completo a Data Components mutáveis em itens (1.21.1) e persistência de anexos | Módulos do MekaSuit, Meka-Tool, Baterias e Tanques Químicos |
| `FORK-24` | Troca de pacotes de rede arbitrários C2S/S2C entre mod e cliente WASM | Sincronização de modos de voo, skins do Robit e renomeação |
| `FORK-27` | Sistema nativo de módulos em armaduras e ferramentas com menu radial de seleção | Meka-Tool, MekaSuit (21 tipos de módulos) |
| `FORK-29` | Física customizada de impulsão de voo e manipulação vetorial de movimento (`JetpackThrust`, `HoverMode`) | Jetpack, Armored Jetpack, MekaSuit Jetpack Unit |
| `FORK-30` | Renderização de HUDs dinâmicos na tela do jogador (status de combustível, dose de radiação e energia) | `MekaSuitEnergyLevel`, `GeigerHUD`, barras de oxigênio |
| `FORK-31` | Escavação multi-bloco orientada à face do bloco atingido (3x3 até 9x9) e mineração em cadeia | Meka-Tool Blasting e Vein Mining Units |

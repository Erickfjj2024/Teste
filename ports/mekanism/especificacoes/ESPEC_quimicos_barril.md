# Especificação Completa: Químicos Líquidos, Baldes e Radioactive Waste Barrel

**Escopo da Tarefa:** T20261109-195424-mh3 (Parte B)  
**Versão de Referência do Java:** Mekanism 1.21.1-10.7.19.85 (`v1.21.1-10.7.19.85`)  
**Módulos de Referência:** `java/main/`, `java/api/`, `java/datagen/generated/mekanism/`

---

## 1. Visão Geral

Esta especificação define com precisão de implementação todas as peças do grupo de **Químicos como Fluido/Balde** (as 18 substâncias químicas e seus respectivos baldes registrados como `FALTA` no inventário mecânico) e o bloco **Radioactive Waste Barrel** (Barril de Resíduos Radioativos).  
O documento fornece todos os nomes de registro, classes Java de origem, blockstates, estruturas de dados de tile entity e persistência (NBT / Data Components), tabelas numéricas exatas com conversão para Forge Energy (FE), inventários, receitas JSON, sons, interações e capacidades requeridas da API do Horizon.

---

## 2. Radioactive Waste Barrel (`mekanism:radioactive_waste_barrel`)

### 2.1 Identificação e Classes Java
- **Registro de Bloco:** `mekanism:radioactive_waste_barrel`
- **Registro de Item:** `mekanism:radioactive_waste_barrel`
- **Registro de Block Entity (Tile Entity):** `mekanism:radioactive_waste_barrel`
- **Classes Java de Origem:**
  - Bloco: `mekanism.common.block.BlockRadioactiveWasteBarrel` (`java/main/java/mekanism/common/block/BlockRadioactiveWasteBarrel.java`)
  - Tile Entity: `mekanism.common.tile.TileEntityRadioactiveWasteBarrel` (`java/main/java/mekanism/common/tile/TileEntityRadioactiveWasteBarrel.java`)
  - Tanque Químico Interno: `mekanism.common.capabilities.chemical.StackedWasteBarrel` (`java/main/java/mekanism/common/capabilities/chemical/StackedWasteBarrel.java`)
  - ItemBlock: `mekanism.common.item.block.ItemBlockRadioactiveWasteBarrel` (`java/main/java/mekanism/common/item/block/ItemBlockRadioactiveWasteBarrel.java`)
  - Definição do Tipo de Bloco: `MekanismBlockTypes.RADIOACTIVE_WASTE_BARREL` (`java/main/java/mekanism/common/registries/MekanismBlockTypes.java:758-763`)

### 2.2 Estados de Bloco e Blockstates
- **Propriedades do Bloco:**
  - `facing` (`Direction.Plane.HORIZONTAL`): `north`, `south`, `east`, `west` (orientação frontal).
  - `active` (`BooleanProperty`): `true` quando está drenando/escoando resíduos para baixo; `false` em repouso.
- **Geometria / VoxelShape:**
  - Idêntica à forma do tanque de fluidos básico (`BlockShapes.RADIOACTIVE_WASTE_BARREL = FLUID_TANK`):
    - Caixa central: `[2, 0, 2]` a `[14, 16, 14]`
    - Arestas superior e inferior salientes (rebordo de proteção).
- **Luminosidade e Atributos:**
  - `Attributes.ACTIVE_LIGHT`: quando `active = true`, emite luz nível 8.
  - `Attributes.COMPARATOR`: suporte nativo a leitor de comparador redstone.
  - `Attributes.SECURITY`: compatível com sistema de segurança e propriedade do Mekanism.

### 2.3 Constantes, Configurações e Valores Numéricos
Todas as configurações estão definidas em `GeneralConfig.java:216-221`:
- **Capacidade do Tanque (`wasteBarrelCapacity`):** `512L * FluidType.BUCKET_VOLUME` = `512_000L` mB (512 baldes).
- **Frequência de Decaimento (`wasteBarrelProcessTicks`):** `SharedConstants.TICKS_PER_SECOND` = `20` ticks (1 segundo).
- **Taxa de Decaimento por Operação (`wasteBarrelDecayAmount`):** `1L` mB a cada 20 ticks (1 mB/s = 3 600 mB/hora).
- **Lista Negra de Decaimento:** Químicos presentes na tag `MekanismAPITags.Chemicals.WASTE_BARREL_DECAY_BLACKLIST` não sofrem decaimento natural dentro do barril.
- **Consumo de Energia:** 0 FE (bloco passivo, não requer energia).

### 2.4 Campos do Tile Entity e Persistência
- **Campos Internos:**
  - `chemicalTank: StackedWasteBarrel`: instância do tanque químico com capacidade de 512 000 mB.
  - `processTicks: i32`: acumulador de ticks (0 a 20) para o ciclo de decaimento de 1 mB.
  - `lastProcessTick: i64`: registro do tempo do jogo (`level.getGameTime()`) para impedir aceleração descontrolada por aceleradores de tick (ex.: Torcherino/Time in a Bottle).
  - `active: bool`: booleano indicando modo de auto-escoamento descendente.
  - `chemicalHandlerBelow: BlockCapabilityCache`: cache de conexão ao manipulador de químicos do bloco imediatamente abaixo (`BlockPos.below()`).
- **Serialização NBT / Data Components:**
  - Tag `"chemical"` (`SerializationConstants.CHEMICAL`): NBT composto contendo o `ChemicalStack` persistido (armazenado pelo `StackedWasteBarrel.serializeNBT()`).
  - Tag `"progress"` (`SerializationConstants.PROGRESS`): inteiro contendo `processTicks`.
  - Tag `"active"`: booleano de ativação do auto-escoamento.
  - **Comportamento Crítico de Descarte ao Quebrar (`blockRemoved` / `collectChemicalTanks`):**
    - Se o barril for quebrado (`blockRemoved`), o método `TileEntityMekanism.blockRemoved()` executa:
      ```java
      if (!isRemote() && RadiationManager.isGlobalRadiationEnabled() && shouldDumpRadiation()) {
          IRadiationManager.INSTANCE.dumpRadiation(getWorldNN(), worldPosition, getChemicalTanks(null), false);
      }
      ```
    - **Efeito:** Todo o resíduo químico radioativo acumulado é liberado instantaneamente na atmosfera como uma fonte de radiação no bloco onde o barril estava!
    - **Exemplo de Cálculo:** Um barril cheio de 512 000 mB de `nuclear_waste` (radioatividade `0.01` Sv/h/mB) cria uma fonte de radiação devastadora de `512_000 * 0.01 = 5_120` Sv/h!
    - Ao mesmo tempo, o item dropado (`ItemBlock`) **NÃO** preserva os químicos radioativos (são descartados da tag NBT no drop).

### 2.5 Conectividade, Regras de Lado e Empilhamento
- **Lados Válidos de Conexão:** Apenas `RelativeSide.TOP` (face superior) e `RelativeSide.BOTTOM` (face inferior). As 4 faces laterais não aceitam conexões de tubos nem transmitem químicos.
- **Validação de Entrada (`StackedWasteBarrel.ATTRIBUTE_VALIDATOR`):**
  - O tanque valida estritamente a radioatividade do químico (`chemical.isRadioactive()`).
  - Químicos sem radioatividade (ex.: oxigênio, hidrogênio, vapor, salmoura) são **recusados**.
- **Comportamento de Empilhamento Ascendente (Overflow Push):**
  - Ao inserir resíduo químico (`insert` ou `growStack`): se o barril estiver cheio ou o valor a inserir exceder o espaço livre, o excedente é empurrado automaticamente para o barril imediatamente acima (`tile.getBlockPos().above()`), permitindo empilhar barris verticalmente em colunas infinitas.
- **Comportamento de Escoamento Descendente (Auto-Drain):**
  - Quando `active == true`, o barril tenta emitir seu conteúdo para o bloco abaixo. Se o bloco abaixo também for um `TileEntityRadioactiveWasteBarrel`, limita o fluxo a `Math.min(below.getNeeded(), chemicalTank.getCapacity())`, evitando loops de retorno ascendente.

### 2.6 Interface, Redstone e Interações
- **Inventário de Itens:** Nenhum (0 slots).
- **Tela GUI:** Nenhuma (não possui contêiner nem menu).
- **Interação com Clique Simples (`useWithoutItem` sem shift):** Retorna `InteractionResult.PASS`.
- **Interação com Agachamento (Shift + Clique Direito com mão vazia):**
  - Servidor consulta o conteúdo do tanque:
    - Se vazio: exibe mensagem no chat `gui.mekanism.no_chemical` ("No chemical").
    - Se contiver resíduo: exibe mensagem colorida `gui.mekanism.stored_mb_percentage` ("Stored: {chemical} ({amount} mB, {percentage}%)").
- **Interação de Configuração (Shift + Clique Direito com Configurador / Wrench):**
  - Inverte o estado de auto-escoamento: `setActive(!getActive())`.
  - Toca som `minecraft:ui.button.click` (volume `0.3F`, pitch `1.0`).
- **Sinal de Comparador Redstone:**
  - Emite sinal analógico de 0 a 15 proporcional ao enchimento do tanque:
    ```java
    MekanismUtils.redstoneLevelFromContents(chemicalTank.getStored(), chemicalTank.getCapacity());
    ```

### 2.7 Receita do Datagen (`data/mekanism/recipe/radioactive_waste_barrel.json`)
```json
{
  "type": "minecraft:crafting_shaped",
  "category": "misc",
  "key": {
    "I": {
      "tag": "c:ingots/lead"
    },
    "S": {
      "tag": "c:ingots/steel"
    }
  },
  "pattern": [
    "SIS",
    "I I",
    "SIS"
  ],
  "result": {
    "count": 1,
    "id": "mekanism:radioactive_waste_barrel"
  }
}
```

---

## 3. Grupo de Químicos como Fluido e Baldes

No Mekanism 1.21.1, as substâncias químicas que possuem representação em estado líquido possuem registro duplo:
1. Como `Chemical` na API de Químicos (`MekanismChemicals.java`).
2. Como `Fluid` no NeoForge (`MekanismFluids.java`), composto por:
   - `FluidType` (`MekanismFluidType`)
   - Fluido Fonte (`BaseFlowingFluid.Source`)
   - Fluido Corrente (`BaseFlowingFluid.Flowing`)
   - Bloco no Mundo (`LiquidBlock`) com ID `mekanism:<nome>`
   - Item Balde (`BucketItem` ou `ItemNutritionalPasteBucket`) com ID `mekanism:<nome>_bucket`

### 3.1 Tabela Completa das 18 Substâncias

| # | Nome de Registro | Bloco (`LiquidBlock`) | Item Balde | Cor Tint (ARGB) | Temp (K) | Densidade (kg/m³) | Viscosidade | Luz | Tag Fluido (`c:`) |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `hydrogen` | `mekanism:hydrogen` | `mekanism:hydrogen_bucket` | `0xFFFFFFFF` | 20 | 71 | 71 | 0 | `c:hydrogen` |
| 2 | `oxygen` | `mekanism:oxygen` | `mekanism:oxygen_bucket` | `0xFF6CE2FF` | 90 | 1 141 | 1 141 | 0 | `c:oxygen` |
| 3 | `chlorine` | `mekanism:chlorine` | `mekanism:chlorine_bucket` | `0xFFCFE800` | 207 | 1 423 | 1 423 | 0 | `c:chlorine` |
| 4 | `sulfur_dioxide` | `mekanism:sulfur_dioxide` | `mekanism:sulfur_dioxide_bucket` | `0xFFA99D90` | 263 | 1 400 | 1 400 | 0 | `c:sulfur_dioxide` |
| 5 | `sulfur_trioxide` | `mekanism:sulfur_trioxide` | `mekanism:sulfur_trioxide_bucket` | `0xFFCE6C6C` | 318 | 1 920 | 1 920 | 0 | `c:sulfur_trioxide` |
| 6 | `sulfuric_acid` | `mekanism:sulfuric_acid` | `mekanism:sulfuric_acid_bucket` | `0xFF82802B` | 300 | 1 840 | 1 840 | 0 | `c:sulfuric_acid` |
| 7 | `hydrogen_chloride` | `mekanism:hydrogen_chloride` | `mekanism:hydrogen_chloride_bucket` | `0xFFA8F1E9` | 188 | 821 | 821 | 0 | `c:hydrogen_chloride` |
| 8 | `hydrofluoric_acid` | `mekanism:hydrofluoric_acid` | `mekanism:hydrofluoric_acid_bucket` | `0xFFC6C7BD` | 190 | 1 150 | 1 150 | 0 | `c:hydrofluoric_acid` |
| 9 | `uranium_oxide` | `mekanism:uranium_oxide` | `mekanism:uranium_oxide_bucket` | `0xFFE1F573` | 3 138 | 10 970 | 10 970 | 0 | `c:uranium_oxide` |
| 10 | `uranium_hexafluoride` | `mekanism:uranium_hexafluoride` | `mekanism:uranium_hexafluoride_bucket` | `0xFF809960` | 337 | 5 090 | 5 090 | 0 | `c:uranium_hexafluoride` |
| 11 | `ethene` | `mekanism:ethene` | `mekanism:ethene_bucket` | `0xFFEACCF9` | 169 | 577 | 577 | 0 | `c:ethene` |
| 12 | `sodium` | `mekanism:sodium` | `mekanism:sodium_bucket` | `0xFFE9FEF4` | 371 | 927 | 927 | 0 | `c:sodium` |
| 13 | `superheated_sodium` | `mekanism:superheated_sodium` | `mekanism:superheated_sodium_bucket` | `0xFFD19469` | 2 000 | 927 | 927 | 0 | `c:superheated_sodium` |
| 14 | `brine` | `mekanism:brine` | `mekanism:brine_bucket` | `0xFFFEEF9C` | 300 | 1 000 | 1 000 | 0 | `c:brine` |
| 15 | `lithium` | `mekanism:lithium` | `mekanism:lithium_bucket` | `0xFFEBA400` | 454 | 512 | 512 | 0 | `c:lithium` |
| 16 | `steam` | `mekanism:steam` | `mekanism:steam_bucket` | `0xFFFFFFFF` | 373 | 0 | 0 | 0 | `c:steam` |
| 17 | `heavy_water` | `mekanism:heavy_water` | `mekanism:heavy_water_bucket` | `0xFF0D1455` | 300 | 1 000 | 1 000 | 0 | `c:heavy_water` |
| 18 | `nutritional_paste` | `mekanism:nutritional_paste` | `mekanism:nutritional_paste_bucket` | `0xFFEB6CA3` | 300 | 1 000 | 1 000 | 0 | `c:nutritional_paste` |

### 3.2 Propriedades do Bloco no Mundo (`LiquidBlock`)
- **Blockstate:** `level` (`IntegerProperty` de 0 a 15).
  - `level = 0`: bloco fonte (source block).
  - `level = 1..7`: bloco corrente (flowing).
  - `level = 8..15`: indicador de queda vertical (falling).
- **Propriedades Físicas:**
  - `noCollision()`: sem colisão de movimento com entidades.
  - `strength(100.0F)`: resistente a quebra com ferramentas comuns.
  - `noLootTable()`: não solta item ao ser quebrado (apenas é coletado por balde).
  - `liquid()`: marcado como fluido líquido no motor de física.
  - `pushReaction(PushReaction.DESTROY)`: pistões destroem o bloco.
  - `mapColor`: calculado dinamicamente pela distância perceptiva de cor euclidiana ponderada em relação à cor do tint (`FluidDeferredRegister.perceptualColorDistanceSquared`).
- **Texturas Utilizadas:**
  - Para a maioria dos químicos líquidos: `mekanism:liquid/liquid` (estático) e `mekanism:liquid/liquid_flow` (fluindo), coloridos dinamicamente pela cor ARGB do tint.
  - Para `steam`: `mekanism:liquid/steam` e `mekanism:liquid/steam_flow`.
  - Para `heavy_water`: `minecraft:block/water_still` e `minecraft:block/water_flow` com tint `0xFF0D1455`.

### 3.3 Propriedades e Comportamento dos Baldes (`BucketItem`)
- **Capacidade do Balde:** 1 000 mB (`FluidType.BUCKET_VOLUME`).
- **Stack Máximo:** 1 item.
- **Item Restante no Uso:** `minecraft:bucket`.
- **Sons:**
  - `SoundActions.BUCKET_FILL`: `minecraft:item.bucket.fill`
  - `SoundActions.BUCKET_EMPTY`: `minecraft:item.bucket.empty`
- **Comportamento em Ejetores (Dispensers):**
  - Registrado com `BUCKET_DISPENSE_BEHAVIOR`: quando acionado virado para um bloco de ar ou substituível, esvazia o fluido e coloca o bloco fonte `mekanism:<nome>` no mundo, deixando um `minecraft:bucket` vazio no ejetor.

### 3.4 Caso Especial: Balde de Pasta Nutricional (`ItemNutritionalPasteBucket`)
- **Classe Java:** `ItemNutritionalPasteBucket.java` (`java/main/java/mekanism/common/item/ItemNutritionalPasteBucket.java`)
- **Duração do Consumo (`getUseDuration`):** 32 ticks (1.6 segundos).
- **Animação de Consumo (`getUseAnimation`):** `UseAnim.DRINK` (beber).
- **Condição de Consumo (`use`):**
  - O jogador pode beber o balde se estiver em modo sobrevivência/aventura (`isPlayingMode`) e estiver com fome (`foodLevel < 20`).
  - Cálculo de nutrição fornecida:
    ```java
    int needed = Math.min(20 - player.getFoodData().getFoodLevel(), 1000 / MekanismConfig.general.nutritionalPasteMBPerFood.get());
    ```
    Com `nutritionalPasteMBPerFood = 50`, um balde completo (1 000 mB) pode restaurar até `1000 / 50 = 20` pontos de fome (barra de fome cheia do zero ao máximo!).
- **Finalização do Consumo (`finishUsingItem`):**
  - Restaura `needed` pontos de fome.
  - Aplica saturação: `MekanismConfig.general.nutritionalPasteSaturation.get()` = `0.8F`.
  - Dispara critérios de avanço: `CriteriaTriggers.CONSUME_ITEM` e estatística `Stats.ITEM_USED`.
  - Transforma o item consumido em `minecraft:bucket`.
  - Se a barra de fome já estiver cheia (ou agachado), o clique direito atua como balde de fluido convencional, despejando a pasta nutricional no mundo como um bloco de fluido.

### 3.5 Receitas de Conversão Bidirecional (Rotary Condensentrator)
Cada um dos 18 químicos possui uma receita oficial no Rotary Condensentrator com taxa de conversão estrita de 1:1 (`1 mB de químico <=> 1 mB de fluido`).
Estrutura do JSON (`data/mekanism/recipe/rotary/<nome>.json`):
```json
{
  "type": "mekanism:rotary",
  "chemical_input": {
    "amount": 1,
    "chemical": "mekanism:<nome>"
  },
  "chemical_output": {
    "amount": 1,
    "id": "mekanism:<nome>"
  },
  "fluid_input": {
    "amount": 1,
    "tag": "c:<nome>"
  },
  "fluid_output": {
    "amount": 1,
    "id": "mekanism:<nome>"
  }
}
```

---

## 4. Requisitos para a API do Horizon (Fork)

Para que qualquer desenvolvedor implemente estas peças sem discrepâncias, a API do Horizon mod SDK deve disponibilizar:

| ID | Nome da Capacidade | Descrição Técnica | Relevância para o Port |
|---|---|---|---|
| **FORK-13** | `RadiationEnvironment` | Sistema de dispersão de radiação com persistência por coordenadas globais e gancho de descarte ambiental `dumpRadiation(pos, stack)` invocado na destruição de blocos | Essencial para o perigo catastrófico do `Radioactive Waste Barrel` |
| **FORK-26** | `FluidChemicalRegistry` | Suporte a registro de fluidos customizados com bloco de fluido no mundo (`LiquidBlock`), física de escoamento e item de balde derivado com capacidade fluida | Essencial para os 18 fluidos e baldes |
| **FORK-29** (novo) | `ConsumableBucketItem` | Capacidade de itens que herdam de balde realizarem ação de beber/comer com tempo de uso (`useDuration`), restauração de fome e saturação customizada | Essencial para `nutritional_paste_bucket` |
| **FORK-30** (novo) | `VerticalTileChaining` | Mecanismo de busca e injeção vertical entre tile entities adjacentes com proteção anti-loop e suporte a overflow em cascata | Essencial para o empilhamento em colunas do `Radioactive Waste Barrel` |

# FORK_assemblicator_cartao.md — Requisitos do Fork para Formulaic Assemblicator, Crafting Formula e Configuration Card

> **Regra do projeto**: não existem "limites do Horizon". Tudo o que a API atual não atende de forma nativa vira um requisito do fork (`FORK-n`) com assinatura WIT exata e contrato público.
> Grupo: `assemblicator_cartao` (Formulaic Assemblicator, Crafting Formula, Configuration Card).

---

## Tabela de Requisitos do Fork

| ID | Requisito / Requirement | Origem no Java / Java Source | API Pública Proposta / Proposed Public API |
|---|---|---|---|
| **FORK-23** | Componentes de dados arbitrários em itens (`ItemStack.components`) com persistência completa de estruturas ricas (SNBT/CompoundTag) | `FormulaAttachment.java`, `ItemCraftingFormula.java`, `ItemConfigurationCard.java` | Extensão do manuseio de `components: string` para preservação íntegra de objetos complexos e listas de stacks |
| **FORK-32** | Resolução e execução dinâmica de receitas de crafting 3x3 no servidor (`CraftingRecipe` vanilla e de mods) | `RecipeFormula.java`, `TileEntityFormulaicAssemblicator.java:302-357` | Interface `crafting`: consulta de receita por matriz 3x3, cálculo de resultado e itens remanescentes |
| **FORK-33** | Suporte a contêineres com mais de 32 slots (35 slots para o Formulaic Assemblicator) sem truncamento | `TileEntityFormulaicAssemblicator.java:154-190` | `ContainerSpec.slots: u32` com suporte garantido a 35 slots na serialização e na GUI |
| **FORK-34** | Disparo de gatilhos de avanço/conquistas no servidor (`CriteriaTrigger`) | `ConfigurationCardTrigger.java`, `ItemConfigurationCard.java:73, 87` | Interface `advancements`: `trigger(player: string, criterion: string, data: string)` |

---

## Especificações WIT Detalhadas

### 1. FORK-32: Dynamic Crafting Recipe Resolution

#### Justificativa
O Formulaic Assemblicator precisa consultar em tempo de execução qualquer receita de crafting válida no mundo (tanto receitas `minecraft:crafting_shaped` quanto `minecraft:crafting_shapeless` de vanilla e de quaisquer outros mods instalados). Além do item resultante, precisa obter a lista de itens remanescentes (`getRemainingItems`), como baldes de leite que se tornam baldes vazios após fazer um bolo.

#### Interface WIT Proposta:
```wit
/// Capability `crafting`: server-side dynamic crafting matrix resolution
interface crafting {
    use %world.{item-stack};

    record crafting-result {
        /// The crafted item stack produced by the recipe
        output: item-stack,
        /// Container remaining items returned to the grid/output (e.g. empty buckets)
        remaining: list<item-stack>,
        /// Identifier of the matched recipe (e.g. "minecraft:cake")
        recipe-id: string,
    }

    /// Evaluates a 3x3 crafting grid and resolves the matching recipe, if any.
    /// The grid contains 9 items in row-major order (indices 0..8); none represents an empty slot.
    match-recipe: func(grid: list<option<item-stack>>) -> option<crafting-result>;
}
```

---

### 2. FORK-23: Rich Data Components on ItemStacks

#### Justificativa
- `mekanism:crafting_formula` armazena o componente `mekanism:formula_holder`, contendo a lista de 9 itens codificados da receita e o booleano `invalid`.
- `mekanism:configuration_card` armazena o componente `mekanism:configuration_data`, contendo o mapa completo de configuração das 5 transmissões (energia, fluidos, químicos, itens, calor), modos de ejeção, cores do transportador e ajustes específicos de máquina.

#### Interface / Contrato WIT:
```wit
/// No mundo horizon-server-mod:
/// Garantia de que `item-stack.components` preserva serialização SNBT sem descarte de tags desconhecidas
record item-stack {
    item: string,
    count: u32,
    /// SNBT string contendo chaves estruturadas (ex.: "mekanism:formula_holder", "mekanism:configuration_data")
    components: string,
}
```

---

### 3. FORK-33: Extended Container Slots (35 slots)

#### Justificativa
O Formulaic Assemblicator possui 35 slots físicos no contêiner:
- Slot 0: Fórmula
- Slots 1..18: 18 slots de entrada
- Slots 19..27: 9 slots de crafting
- Slots 28..33: 6 slots de saída
- Slot 34: 1 slot de energia

O motor do Horizon deve permitir o valor `slots = 35` no `container-spec` sem truncamento no sincronizador de rede ou no cliente.

```wit
record container-spec {
    slots: u32, // Deve aceitar 35 sem limites artificiais de 32
    faces: list<list<u32>>,
    inputs: list<u32>,
    outputs: list<u32>,
    menu: bool,
}
```

---

### 4. FORK-34: Server Advancement Criteria Trigger

#### Justificativa
O `ItemConfigurationCard.java` aciona `MekanismCriteriaTriggers.CONFIGURATION_CARD` com o booleano `copied` (`true` ao copiar configuração, `false` ao colar configuração).

#### Interface WIT Proposta:
```wit
interface advancements {
    /// Triggers an advancement criterion for a player.
    trigger: func(player: string, trigger-id: string, payload: string) -> bool;
}
```
Exemplo de chamada:
```rust
let _ = advancements::trigger(&player_id, "mekanism:configuration_card", "{\"copied\":true}");
```

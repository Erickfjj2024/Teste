# Requisitos de Extensão do Fork Horizon — MekaSuit e Sistema de Módulos (FORK_mekasuit)

> **Documento de especificação técnica para o fork da API do Horizon.**
> Grupo: `mekasuit`
> Módulos implementados: `modules.rs`, `mekasuit.rs`

Para que o **MekaSuit** (as 4 peças) e todos os seus 21 módulos associados funcionem no Horizon com **100% de paridade com o Mekanism Java 1.21.1**, são necessárias as extensões de API descritas abaixo. Nenhuma funcionalidade foi simplificada: a lógica completa foi escrita em Rust puro em `modules.rs` e `mekasuit.rs`, isolando as chamadas ao host em funções marcadas `// FORK-n`.

---

## Tabela Resumo dos Requisitos

| ID | Requisito / Requirement | Origem no Java / Java Source | API Pública Proposta / Proposed WIT |
|---|---|---|---|
| `FORK-23` | Data Components estruturados em itens (1.21.1) com persistência de anexos | `MekanismDataComponents.MODULES`, `ATTACHED_ENERGY` | `interface item-components` |
| `FORK-27` | Sistema modular de equipamentos, hook de tick de armadura e absorção de dano | `ItemMekaSuitArmor.java:408`, `IModuleContainer` | `interface equipment-modules` |
| `FORK-29` | Física vetorial de propulsão, pairamento (hover) e voo criativo | `ItemMekaSuitArmor:354`, `ModuleGravitationalModulatingUnit` | `interface player-physics` |
| `FORK-30` | Renderização dinâmica de HUD (energia, hidrogênio, radiação, pasta nutricional) | `MekaSuitEnergyLevel.java`, `ModuleDosimeterUnit.java` | `interface hud-elements` |
| `FORK-11` | Classificação de DamageSource (fogo, magia prevenível, tags de dano) | `ItemMekaSuitArmor.java:448`, `DamageTypeTags` | `interface damage-context` |

---

## 1. FORK-23: Data Components Estruturados e Anexos

### O que falta na API atual do Horizon
Atualmente, o Horizon expõe apenas `item-stack.components: string` (SNBT não tipado). A leitura e escrita requer parsing de texto frágil. O Mekanism 1.21.1 utiliza `DataComponentType` fortemente tipados para armazenar:
- Lista de módulos instalados (`mekanism:modules`) com quantidade e mapa de configurações por módulo.
- Armazenamento de energia primário (`mekanism:attached_energy`).
- Tanque de gás/químico interno para hidrogênio do Jetpack (`mekanism:attached_chemicals`).
- Tanque de fluido interno para pasta nutricional (`mekanism:attached_fluids`).

### Assinatura WIT Proposta

```wit
interface item-components {
    use item-types.{item-stack};

    /// Lê um componente serializado em formato JSON/CBOR estruturado da stack
    get-component: func(stack: item-stack, component-id: string) -> option<string>;

    /// Modifica ou insere um componente estruturado na stack
    set-component: func(stack: item-stack, component-id: string, value: string) -> item-stack;

    /// Remove um componente específico da stack
    remove-component: func(stack: item-stack, component-id: string) -> item-stack;
}
```

---

## 2. FORK-27: Equipamentos Modulares e Hook de Dano / Tick de Armadura

### O que falta na API atual do Horizon
O Horizon não possui hooks de evento disparados quando um jogador veste peças de armadura no tick do servidor ou quando o jogador sofre dano prevenível por armadura. Hoje, o mod só consegue consultar o inventário estaticamente.

### Assinatura WIT Proposta

```wit
interface equipment-modules {
    use item-types.{item-stack};
    use players.{player};

    record armor-slots {
        feet: option<item-stack>,
        legs: option<item-stack>,
        chest: option<item-stack>,
        head: option<item-stack>,
    }

    /// Hook disparado antes do dano ser aplicado ao jogador.
    /// O mod calcula quanto dano a armadura absorve (via MekaSuit/módulos) e quanto consome de energia.
    /// Retorna a fração de dano absorvida (0.0 a 1.0).
    record damage-absorption-result {
        absorbed-ratio: f32,
        damage-absorbed: f32,
        damage-remaining: f32,
    }

    on-player-damaged: func(player: player, damage-type: string, amount: f32, is-fire: bool, is-magic: bool) -> damage-absorption-result;

    /// Hook de tick de armadura executado a cada tick (20/s) para os equipamentos vestidos
    tick-armor: func(player: player, slots: armor-slots) -> armor-slots;
}
```

---

## 3. FORK-29: Física de Voo, Propulsão Vetorial e Pairamento (Jetpack / Gravitational Unit)

### O que falta na API atual do Horizon
O Horizon não permite injetar impulsos de velocidade vetorial 3D (`addDeltaMovement`, `moveRelative`) nem chavear a habilidade de voo criativo (`mayFly`, `isFlying`) em tempo de execução para entidades de jogador. Isso impede que o Jetpack Unit, o Jump Boost hidráulico e o Gravitational Modulating Unit apliquem a física exata do original.

### Assinatura WIT Proposta

```wit
interface player-physics {
    use players.{player};

    record vec3 {
        x: f64,
        y: f64,
        z: f64,
    }

    /// Adiciona impulso de velocidade relativa à direção que o jogador está olhando (moveRelative vanilla)
    add-relative-movement: func(player: player, forward: f32, up: f32, strafe: f32) -> result<_, string>;

    /// Injeta vetor absoluto de aceleração (addDeltaMovement)
    add-delta-movement: func(player: player, velocity: vec3) -> result<_, string>;

    /// Habilita ou desabilita capacidade de voo estilo criativo (NeoForgeMod.CREATIVE_FLIGHT)
    set-flight-allowed: func(player: player, allowed: bool) -> result<_, string>;

    /// Define a altura de step-assist (subida de blocos sem pular, de 0.6 até 2.0 blocos)
    set-step-height: func(player: player, height: f32) -> result<_, string>;

    /// Reseta a distância acumulada de queda para evitar dano de impacto
    reset-fall-distance: func(player: player) -> result<_, string>;
}
```

---

## 4. FORK-30: Elementos Dinâmicos de HUD

### O que falta na API atual do Horizon
A interface `hud` do Horizon atual é limitada a janelas simples e chat. O MekaSuit exige renderização de ícones dinâmicos com barras de porcentagem na tela do jogador:
- Nível de energia da armadura (`MekaSuitEnergyLevel.java`).
- Nível de combustível hidrogênio do Jetpack (`ModuleJetpackUnit.java:46-57`).
- Porcentagem de pasta nutricional restante (`ModuleNutritionalInjectionUnit.java:49-61`).
- Nível de radiação do chunk e do jogador com timers de decaimento (`ModuleGeigerUnit.java`, `ModuleDosimeterUnit.java`).

### Assinatura WIT Proposta

```wit
interface hud-elements {
    use players.{player};

    enum hud-color {
        regular,
        warning,
        danger,
    }

    record hud-element-spec {
        element-id: string,
        icon-texture: string,
        text: option<string>,
        percent: option<f32>,
        color: hud-color,
        screen-corner: string, // "top-left", "bottom-right", etc.
    }

    /// Envia ou atualiza um widget de HUD na tela do cliente
    render-hud-element: func(player: player, element: hud-element-spec) -> result<_, string>;

    /// Remove um widget de HUD da tela do cliente
    clear-hud-element: func(player: player, element-id: string) -> result<_, string>;
}
```

---

## 5. FORK-11: Classificação de Tipos de Dano e Tags da DamageSource

### O que falta na API atual do Horizon
`player-items.hurt` aceita apenas uma string de `damage-type`. O sistema de absorção do Mekanism precisa inspecionar tags como:
- `DamageTypeTags.IS_FIRE` (para Geothermal Generator Unit)
- `MekanismAPITags.DamageTypes.IS_PREVENTABLE_MAGIC` (para Inhalation Purification Unit)
- `DamageTypeTags.BYPASSES_ARMOR`
- `Tags.DamageTypes.IS_TECHNICAL`
- `MekanismAPITags.DamageTypes.MEKASUIT_ALWAYS_SUPPORTED`

### Assinatura WIT Proposta

```wit
interface damage-context {
    record damage-flags {
        is-fire: bool,
        is-magic: bool,
        is-fall: bool,
        is-technical: bool,
        bypasses-armor: bool,
        always-supported: bool,
    }

    /// Consulta as tags e características de uma fonte de dano
    get-damage-flags: func(damage-type: string) -> damage-flags;
}
```

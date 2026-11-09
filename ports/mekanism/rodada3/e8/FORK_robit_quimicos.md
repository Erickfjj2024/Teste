# Requisitos de Fork da API do Horizon: Grupo `robit_quimicos`

Este documento detalha os ganchos e extensões necessários na API do Horizon SDK (`horizon.wit` e runtime do host) para suportar 100% da fidelidade do Java para o Robit, os 18 Químicos/Fluidos/Baldes e o Radioactive Waste Barrel, seguindo as diretrizes de `port/API-REQUISITOS.md`.

---

## 1. Tabela Resumo de Requisitos

| ID | Nome da Capacidade | Descrição Técnica | Arquivo de Origem no Java | Módulo Rust no Port |
|---|---|---|---|---|
| **FORK-10** | `CustomEntities` | Entidades customizadas com IA de navegação, carregamento de chunks, inventário e renderização | `EntityRobit.java`, `RobitAIFollow.java`, `RobitAIPickup.java` | `robit.rs` |
| **FORK-13** | `RadiationEnvironment` | Dispersão atmosférica de radiação ambiental e descarte instantâneo `dumpRadiation` ao quebrar blocos | `TileEntityMekanism.java:715`, `RadiationManager.java` | `radioactive_waste_barrel.rs` |
| **FORK-23** | `ItemDataComponents` | Suporte completo a Data Components mutáveis (1.21.1) persistindo energia, inventário, nome e skins | `ItemRobit.java`, `MekanismDataComponents.java` | `robit.rs` |
| **FORK-24** | `ModNetworkSync` | Troca de pacotes C2S/S2C arbitrários para botões de entidade, skins e renomeação | `PacketEntityButtonPress.java`, `PacketGuiInteract.java` | `robit.rs` |
| **FORK-26** | `FluidChemicalRegistry` | Registro de fluidos customizados com bloco no mundo (`LiquidBlock`), física de escoamento e baldes | `MekanismFluids.java`, `FluidDeferredRegister.java` | `chemical_fluids.rs` |
| **FORK-29** | `ConsumableBucketItem` | Itens derivados de balde com ação de beber/comer, tempo de uso, restauração de fome e saturação | `ItemNutritionalPasteBucket.java:33-73` | `chemical_fluids.rs` |
| **FORK-30** | `VerticalTileChaining` | Consulta e injeção vertical entre tile entities adjacentes com proteção anti-loop e overflow | `StackedWasteBarrel.java:49-61`, `TileEntityRadioactiveWasteBarrel.java:82-98` | `radioactive_waste_barrel.rs` |

---

## 2. Detalhamento dos Requisitos e Assinaturas WIT Propostas

### FORK-10: `CustomEntities` (Robit Mob Entity & AI)
- **Problema no Horizon Atual**: O Horizon SDK possui apenas suporte a mobs vanilla fixos e entidades estáticas de exibição (`display`). Falta a capacidade de declarar uma entidade customizada com inventário serializado de 30 slots, máquina interna de fornalha e metas de IA customizadas (seguir o dono a distâncias configuráveis e recolher drops do chão).
- **Assinatura WIT Proposta**:
```wit
interface custom-entities {
    record entity-spec {
        kind: string,
        max-health: f32,
        movement-speed: f32,
        has-inventory: bool,
        inventory-size: u32,
        can-float-in-water: bool,
        unpassable-rail-malus: f32,
    }

    record ai-goal-follow {
        min-distance: f32,
        max-distance: f32,
        teleport-distance: f32,
        speed-multiplier: f32,
    }

    record ai-goal-item-pickup {
        search-radius: f32,
        collection-inflation: f32,
    }

    register-custom-entity: func(spec: entity-spec) -> result<_, string>;
    set-follow-target: func(entity-id: u32, target-player-id: string, goal: ai-goal-follow) -> result<_, string>;
    set-pickup-goal: func(entity-id: u32, goal: ai-goal-item-pickup) -> result<_, string>;
    add-chunk-unload-ticket: func(dimension: string, chunk-x: s32, chunk-z: s32, duration-ticks: u32) -> result<_, string>;
}
```

---

### FORK-13: `RadiationEnvironment` (Dispersão e Descarte Ambiental)
- **Problema no Horizon Atual**: A radiação no mod Rust roda em memória isolada e não interage com o ciclo de vida dos blocos no mundo nativo. Quando um barril contendo até 512 000 mB de resíduo radioativo é quebrado, o Mekanism original invoca `IRadiationManager.INSTANCE.dumpRadiation` para poluir catastrophicamente o ambiente com até 5 120 Sv/h.
- **Assinatura WIT Proposta**:
```wit
interface radiation-environment {
    record radiation-source {
        dimension: string,
        x: s32,
        y: s32,
        z: s32,
        magnitude-sv-h: f64,
        decay-rate: f64,
    }

    /// Dispersa uma fonte instantânea de radiação na coordenada global especificada.
    dump-radiation: func(dimension: string, x: s32, y: s32, z: s32, magnitude-sv-h: f64);
    /// Consulta o nível de radiação acumulada no bloco ou chunk.
    get-radiation-level: func(dimension: string, x: s32, y: s32, z: s32) -> f64;
}
```

---

### FORK-23: `ItemDataComponents` (Persistência em Itens 1.21.1)
- **Problema no Horizon Atual**: A representação atual de itens em `horizon.wit` é limitada a `{ item: string, count: u32, components: string }` em formato de texto opaco. O Mekanism 1.21.1 utiliza o novo sistema de Data Components do Minecraft para armazenar energia anexa (`mekanism:attached_energy`), itens do container (`mekanism:container_items`), nome do Robit (`mekanism:robit_name`), skins (`mekanism:robit_skin`) e segurança (`mekanism:security`).
- **Assinatura WIT Proposta**:
```wit
interface item-components {
    record component-entry {
        component-id: string,
        data-bytes: list<u8>,
    }

    get-component: func(item-id: string, component-id: string) -> option<list<u8>>;
    set-component: func(item-id: string, component-id: string, data: list<u8>) -> result<_, string>;
}
```

---

### FORK-24: `ModNetworkSync` (Sincronização de Pacotes Customizados C2S/S2C)
- **Problema no Horizon Atual**: O mod WASM depende de eventos de clique em blocos; não há interface nativa para sincronizar cliques em abas de entidades móveis (como as 5 abas laterais do Robit), seleções em menus popup (janela de renomeação de texto livre e grade rolável de seleção de skins) nem envio de pacotes customizados de rede como `PacketEntityButtonPress` e `PacketRobitSkin`.
- **Assinatura WIT Proposta**:
```wit
interface mod-network-sync {
    record entity-action-packet {
        entity-id: u32,
        action-type: string, // "go_home", "toggle_follow", "toggle_pickup", "set_skin", "rename"
        payload: list<u8>,
    }

    send-entity-packet-to-server: func(packet: entity-action-packet) -> result<_, string>;
    broadcast-entity-state-to-tracking-players: func(entity-id: u32, state: list<u8>) -> result<_, string>;
}
```

---

### FORK-26: `FluidChemicalRegistry` (Fluidos Customizados, Blocos Líquidos e Baldes)
- **Problema no Horizon Atual**: O Horizon não possui API declarativa para mods WASM registrarem novos fluidos físicos. Cada um dos 18 químicos do Mekanism possui um `LiquidBlock` correspondente no mundo, física de fluxo (níveis de 0 a 15, onde 0 é fonte e 1-7 é corrente), cor de tint ARGB ponderada e um item de balde derivado que devolve `minecraft:bucket`.
- **Assinatura WIT Proposta**:
```wit
interface fluid-registry {
    record custom-fluid-spec {
        id: string,
        still-block: string,
        flowing-block: string,
        bucket-item: string,
        tint-argb: u32,
        temperature-kelvin: u32,
        density-kg-m3: s32,
        viscosity: s32,
        luminosity: u8,
        still-texture: string,
        flowing-texture: string,
    }

    register-custom-fluid: func(spec: custom-fluid-spec) -> result<_, string>;
}
```

---

### FORK-29: `ConsumableBucketItem` (Baldes Bebíveis / Comestíveis)
- **Problema no Horizon Atual**: Na API vanilla e atual do Horizon, um item registrado como balde só executa ações de esvaziamento/preenchimento de bloco no mundo. O `nutritional_paste_bucket` do Mekanism possui comportamento híbrido (`ItemNutritionalPasteBucket.java`): quando o jogador está com fome e em modo de jogo compatível, o clique direito executa a animação de beber (`UseAnim.DRINK`) por 32 ticks, restaurando até 20 pontos de fome e 0.8 de saturação, devolvendo um balde de ferro vazio.
- **Assinatura WIT Proposta**:
```wit
interface consumable-buckets {
    record consumable-bucket-spec {
        item: string,
        use-duration-ticks: u32,
        use-animation: string, // "drink", "eat"
        food-gain-per-bucket: u32,
        saturation-gain: f32,
        returned-empty-item: string,
    }

    register-consumable-bucket: func(spec: consumable-bucket-spec) -> result<_, string>;
}
```

---

### FORK-30: `VerticalTileChaining` (Encadeamento Vertical com Proteção Anti-Loop)
- **Problema no Horizon Atual**: Cada bloco em `block_entities_tick` é processado isoladamente em sua coordenada. O `RadioactiveWasteBarrel` requer consulta rápida aos blocos imediatamente acima (`pos.y + 1`) e abaixo (`pos.y - 1`) na mesma coluna de chunks, permitindo o empilhamento infinito com overflow automático para cima e auto-escoamento para baixo limitado estritamente a `min(below.needed(), capacity)` para evitar loops recursivos de retorno.
- **Assinatura WIT Proposta**:
```wit
interface tile-chaining {
    /// Consulta uma capacidade específica do bloco vizinho relativo sem recarregar o bloco inteiro.
    query-vertical-neighbor-capability: func(dimension: string, x: s32, y: s32, z: s32, direction: string, capability: string) -> option<list<u8>>;
    /// Injeta químicos diretamente no vizinho mantendo cache atômico na transação de tick.
    transfer-chemical-vertical: func(dimension: string, from-x: s32, from-y: s32, from-z: s32, direction: string, amount: u64) -> u64;
}
```

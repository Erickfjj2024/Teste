# Requisitos de Fork Horizon: Grupo `meka_tool`

Documento de especificação de extensões e capacidades exigidas da API do Horizon (`horizon.wit` e fork do runtime) para suporte a **Meka-Tool**, **Electric Bow**, **HDPE Elytra** e os 10 módulos de utilidade.
Segue o formato padrão de `port/API-REQUISITOS.md`.

---

## Tabela de Requisitos

| ID | Requisito / Requirement | Origem no Java / Java source | API pública proposta / Proposed Public API |
|---|---|---|---|
| **FORK-23** | Leitura e escrita de componentes de dados SNBT arbitrários em `ItemStack` em tempo de execução | `MekanismDataComponents.java:66, 72, 150` | `player-items.set-custom-components` / `item-stack.components` |
| **FORK-27** | Sistema de menu radial acionado por tecla/gesto para alternar modos de módulos instalados em ferramentas/armaduras | `IRadialModuleContainerItem.java`, `ModuleExcavationEscalationUnit.java:46`, `ModuleBlastingUnit.java:44` | `interface radial-menu` |
| **FORK-31** | Escavação atômica multi-bloco orientada à face (3x3 até 9x9) e mineração em cadeia (Vein Mining) com atribuição de drops ao jogador e exaustão/energia | `IBlastingItem.java:26-60`, `MekanismUtils.java:807-860`, `ModuleVeinMiningUnit.java:136-177` | `world.break-blocks-attributed` |
| **FORK-32** | Teletransporte de jogador orientado por raycast (até 100 blocos) com verificação de desmonte, clearance vertical e efeito sonoro/partículas | `ItemMekaTool.java:373-417`, `PacketPortalFX.java` | `player-control.teleport-with-fx` |
| **FORK-33** | Hooks de liberação de arco (`releaseUsing`) e injeção dinâmica de propriedades de projétil (Flame I, velocidade, consumo de energia) | `ItemElectricBow.java:53-76` | `interface bow-actions` / `entities.shoot-projectile` |
| **FORK-34** | Suporte a voo planar de Elytra customizada, desgaste por tempo de voo e validação de item de reparo na bigorna | `ItemHDPEElytra.java:19-25`, `MekanismItems.java:184` | `player-control.set-gliding` / `interface anvil-repair` |

---

## Assinaturas WIT Propostas

### 1. `FORK-23`: Manipulação Mutável de Data Components em Itens

```wit
interface item-components {
    use item-types.{item-stack};

    /// Lê um componente estruturado serializado como JSON/SNBT de um item na mão do jogador.
    get-component: func(player: string, slot: u32, component-id: string) -> option<string>;

    /// Define ou atualiza um componente de dados no ItemStack do slot especificado.
    set-component: func(player: string, slot: u32, component-id: string, snbt-data: string) -> result<_, string>;
}
```

### 2. `FORK-27`: Menu Radial de Equipamentos

```wit
interface radial-menu {
    record radial-slice {
        id: string,
        label: string,
        icon-texture: string,
        color-argb: u32,
    }

    /// Abre o seletor radial na tela do cliente para o item atualmente segurado.
    open-radial: func(player: string, slices: list<radial-slice>, current-index: u32) -> result<_, string>;

    /// Callback disparado quando o jogador confirma a seleção de uma fatia radial.
    export on-radial-select: func(player: string, slice-id: string);
}
```

### 3. `FORK-31`: Escavação Multi-Bloco Atômica (Blasting & Vein Mining)

```wit
interface area-mining {
    use server-events.{block-pos};
    use item-types.{item-stack};

    record mined-block-result {
        pos: block-pos,
        block-state: string,
        drops: list<item-stack>,
    }

    /// Quebra múltiplos blocos de uma vez no mundo com verificação de proteção e atribuição ao jogador.
    /// Retorna os blocos efetivamente destruídos e seus drops consolidados.
    break-blocks-area: func(
        dimension: string,
        player: string,
        positions: list<block-pos>,
        tool-stack: item-stack,
        silk-touch: bool,
        fortune-level: u32
    ) -> result<list<mined-block-result>, string>;
}
```

### 4. `FORK-32`: Teletransporte Raycast com FX

```wit
interface teleport-effects {
    use server-events.{block-pos};

    record teleport-target {
        dimension: string,
        x: f64,
        y: f64,
        z: f64,
        reset-fall-distance: bool,
        dismount-passenger: bool,
    }

    /// Teleporta o jogador com validação de passageiro, zeramento de fallDistance,
    /// reprodução de som e pacote de partículas de portal na origem e destino.
    teleport-player-fx: func(
        player: string,
        target: teleport-target,
        sound-event: string,
        particle-effect: string
    ) -> result<_, string>;
}
```

### 5. `FORK-33`: Hooks de Arco e Projéteis

```wit
interface bow-actions {
    use item-types.{item-stack};

    record bow-shoot-event {
        player: string,
        bow: item-stack,
        charge-ticks: u32,
        is-creative: bool,
    }

    /// Disparado antes da flecha ser liberada. Mod pode cancelar o tiro (ex.: falta de energia)
    /// ou alterar propriedades do projétil resultante.
    on-bow-release: func(event: bow-shoot-event) -> result<bow-override, string>;

    record bow-override {
        allow-shoot: bool,
        set-fire: bool,
        bonus-damage: f32,
        energy-consumed: u64,
    }
}
```

### 6. `FORK-34`: Elytra Customizada e Reparo em Bigorna

```wit
interface elytra-flight {
    use item-types.{item-stack};

    /// Verifica se a armadura no slot do peitoral permite planar e emite ticks de desgaste.
    check-elytra-gliding: func(player: string) -> bool;

    /// Permite que itens customizados definam materiais válidos de reparo na bigorna.
    is-anvil-repairable: func(target-item: string, repair-item: string) -> bool;
}
```

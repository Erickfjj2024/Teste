# Requisitos do Fork e Extensões da API do Horizon: Grupo Calor

**ID da Tarefa**: T20261109-214921-2os  
**Grupo**: Calor e dependentes (`heat`, `resistive_heater`, `fuelwood_heater`, `laser_tractor`)  
**Referência de Verdade**: Mekanism 1.21.1-10.7.19.85 (`java/`)  
**Base de Requisitos**: `port/API-REQUISITOS.md` e `port/api/horizon.wit`

---

## 1. Tabela Resumo dos Requisitos do Fork

| ID | Requisito | Origem no Java | API Pública Proposta |
|---|---|---|---|
| **FORK-4** | Meio "calor" de primeira classe no motor de transmissores com condutores térmicos, dissipação no ar e transferência condutiva entre blocos | `HeatAPI.java`, `ConductorTier.java`, `HeatNetwork.java`, `ThermodynamicConductor.java`, `ITileHeatHandler.java` | `transmitter-medium::heat`, `heat-spec`, `heat-capacitor` |
| **FORK-7** | Dano a entidades, queima e absorção de feixe laser por mod (2.500 J / 1.000 FE por ½ coração) | `GeneralConfig.java:239`, `TileEntityBasicLaser.java:109-263` | `laser::raycast-entities`, `entity::hurt-custom` |
| **FORK-8** | Escavação a laser por dureza (100.000 J / 40.000 FE por ponto) com interceptação de drops e atração de `ItemEntity` | `GeneralConfig.java:238`, `TileEntityLaserTractorBeam.java:57-97`, `TileEntityBasicLaser.java:286-302` | `world::break-block-intercept-drops`, `entity::collect-item-entity`, `world::pop-resource-from-face` |
| **FORK-20** | Componentes de interface interativa: caixa de texto digital (`GuiTextField`) para entrada de taxa de consumo do Aquecedor Resistivo | `GuiResistiveHeater.java:62-67, 76-85`, `PacketGuiSetEnergy.java` | `menu::widget-text-field`, `menu::client-message` |

---

## 2. Detalhamento e Assinaturas WIT Propostas

### 2.1 FORK-4: Meio Calor (`transmitter-medium::heat`)

#### O que falta no Horizon atualmente
No arquivo `port/api/horizon.wit`, o enum `transmitter-medium` suporta apenas:
```wit
variant transmitter-medium {
    energy,
    tank(string),
    item,
}
```
Não existe suporte nativo para transmissão de calor. Redes térmicas operam com física contínua:
1. Dissipação para a atmosfera com base na temperatura local do bioma (`ambient_temp`) e resistência ao ar (`AIR_INVERSE_COEFFICIENT = 10_000`).
2. Isolamento térmico por tier do condutor (10 / 400 / 8.000 / 100.000).
3. Condução inversa entre corpos adjacentes e equilíbrio térmico.

#### Assinatura WIT Proposta para `horizon.wit`
```wit
/// Extensão de `transmitter-medium` no horizon.wit
variant transmitter-medium {
    energy,
    tank(string),
    item,
    /// Meio calor com parâmetros térmicos por tier (FORK-4)
    heat(heat-spec),
}

/// Especificação de transmissão e armazenamento térmico
record heat-spec {
    heat-capacity: float64,
    inverse-conduction: float64,
    inverse-insulation: float64,
}

/// Interface de manipulação térmica exposta aos blocos e transmissores
interface heat-handler {
    /// Obtém a temperatura do bloco em Kelvin
    get-temperature: func(dimension: string, x: s32, y: s32, z: s32) -> result<float64, string>;
    /// Transfere energia térmica (em Joules) diretamente para o bloco
    transfer-heat: func(dimension: string, x: s32, y: s32, z: s32, joules: float64) -> result<float64, string>;
    /// Consulta temperatura ambiente calculada do bioma
    get-ambient-temperature: func(dimension: string, x: s32, y: s32, z: s32) -> float64;
}
```

---

### 2.2 FORK-7: Dano e Interação do Feixe Laser com Entidades

#### O que falta no Horizon atualmente
Não há mecanismo na API para traçar caixas delimitadoras de feixe (`AABB.inflate`) no servidor WASM e interagir com entidades em trânsito (aplicar queima `igniteForTicks`, dano direto `damageSource(laser)` e checagem de bloqueio de escudos).

#### Assinatura WIT Proposta para `horizon.wit`
```wit
/// Traçado e interação física de feixes volumétricos (FORK-7)
interface laser-physics {
    record entity-hit {
        entity-id: u64,
        distance: float64,
        is-living: bool,
        is-item: bool,
    }

    /// Executa varredura de entidades no feixe entre dois pontos com largura dada
    raycast-entities: func(dimension: string, from-x: float64, from-y: float64, from-z: float64,
                           to-x: float64, to-y: float64, to-z: float64, diameter: float32) -> list<entity-hit>;

    /// Aplica dano térmico/laser a uma entidade específica
    damage-entity-laser: func(entity-id: u64, amount: float32, ignite-ticks: u32) -> result<float32, string>;
}
```

---

### 2.3 FORK-8: Mineração a Laser, Captura de Drops e `ItemEntity`

#### O que falta no Horizon atualmente
O Laser Tractor Beam substitui a quebra vanilla padrão:
1. Ele não dropa os itens no chão; ele intercepta os drops gerados pela tabela de saque do bloco (`WorldUtils.getDrops`) e tenta colocá-los nos seus 27 slots internos.
2. Itens flutuantes no mundo (`ItemEntity`) que cruzam o feixe são "puxados" para dentro do inventário e a entidade é descartada (`entity.discard()`).
3. Se os 27 slots estiverem cheios, ele expulsa o excedente no centro frontal do bloco com vetor de velocidade direcionado para trás (`Block.popResourceFromFace`).

#### Assinatura WIT Proposta para `horizon.wit`
```wit
/// Mineração interceptada e controle de drops para feixes tratores (FORK-8)
interface laser-mining {
    /// Quebra o bloco na posição dada e retorna a lista de itens dropados sem soltá-los no mundo
    break-block-with-drop-intercept: func(dimension: string, x: s32, y: s32, z: s32) -> result<list<item-stack>, string>;

    /// Absorve ou destrói uma entidade de item do mundo se recolhida
    absorb-world-item: func(entity-id: u64, max-count: u32) -> result<item-stack, string>;

    /// Ejeta recurso em face específica com velocidade orientada (para transbordamento)
    pop-resource-from-face: func(dimension: string, x: s32, y: s32, z: s32, face: u32, item: item-stack) -> result<_, string>;
}
```

---

### 2.4 FORK-20: Campo de Texto Digital Interativo (`GuiTextField`) na UI

#### O que falta no Horizon atualmente
O sistema atual de menus do Horizon (`menu-data`) transmite apenas inteiros (`s32`). O Aquecedor Resistivo do Mekanism possui um `GuiTextField` no centro da tela (`(50, 51, 76, 12)` com até 7 dígitos) que envia um pacote `PacketGuiSetEnergy` ao servidor para definir a taxa de consumo em Joules/t.

#### Assinatura WIT Proposta para `horizon.wit`
```wit
/// Widgets avançados de tela e comunicação bidirecional de GUI (FORK-20)
interface screen-input {
    /// Declara campo de texto numérico para tela de bloco
    record text-field-spec {
        x: s32,
        y: s32,
        width: s32,
        height: s32,
        max-digits: u32,
        default-value: string,
    }

    /// Evento disparado pelo cliente ao submeter novo valor numérico no widget
    record gui-submit-value-event {
        dimension: string,
        pos: block-pos,
        field-id: u32,
        numeric-value: u64,
    }
}
```

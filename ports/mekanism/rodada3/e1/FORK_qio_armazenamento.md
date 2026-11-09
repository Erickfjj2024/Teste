# Requisitos de Plataforma Horizon (FORK): QIO Armazenamento

**Grupo**: `qio_armazenamento`  
**Referência**: Mekanism Java `1.21.1-10.7.19.85`  
**Estado**: Pedido / Especificação de Extensões WIT para o Host Horizon

---

## 1. Tabela Resumo de Requisitos FORK

| ID | Requisito / Requirement | Origem no Java / Java Source | API Pública / Public API |
|---|---|---|---|
| **FORK-18** | Frequências universais nomeadas por UUID e dono, com persistência independente e modo de segurança | `QIOFrequency.java`, `Frequency.java` | `interface frequencies` |
| **FORK-20** | Widgets de interface gráfica ricos (campo de texto numérico, LEDs dinâmicos de drive array) | `GuiQIODriveArray.java`, `GuiQIORedstoneAdapter.java` | `interface custom-ui` |
| **FORK-21** | Emissão seletiva de sinal de redstone por face do bloco com intensidade direcional (força 15 nas 5 faces não-traseiras) | `TileEntityQIORedstoneAdapter.java:48` | `interface %world` (`set-redstone-output-faces`) |
| **FORK-23** | Componentes de dados em ItemStacks (`DriveMetadata`, `DriveContents`, `ITEM_TARGET`) | `ItemQIODrive.java`, `MekanismDataComponents` | `interface item-components` |
| **FORK-24** | Transmissão de pacotes cliente-servidor para sincronização em lote (`BulkQIOData`) | `BulkQIOData.java`, `QIOFrequency.java:505` | `interface net-messages` |

---

## 2. Detalhamento e Assinaturas WIT Propostas

### FORK-18: Frequências Universais Nomeadas por UUID e Dono

#### Contexto e Necessidade
No Mekanism Java, as frequências QIO (`QIOFrequency`) são redes universais de trânsito e armazenamento indexado. Elas não pertencem a um único bloco físico, possuem um identificador nomeado (`name: String`), um proprietário associado via UUID (`ownerUUID: UUID`), um modo de segurança (`SecurityMode`: PUBLIC, PRIVATE, TRUSTED) e uma cor de identificação (`EnumColor`).  
A API atual do Horizon em `entangloporter.rs` limita frequências a números inteiros locais (1 a 99) sem posse nem persistência de mundo. O subsistema QIO exige uma infraestrutura global onde qualquer Drive Array, Redstone Adapter ou Dashboard possa se vincular à frequência do jogador.

#### Assinatura WIT Proposta (`port/api/horizon.wit`)
```wit
interface frequencies {
    enum security-mode {
        public,
        private,
        trusted,
    }

    record frequency-id {
        name: string,
        owner: option<string>, // UUID formatado
        security: security-mode,
    }

    record qio-frequency-summary {
        id: frequency-id,
        color: u8,
        total-item-count: u64,
        total-item-capacity: u64,
        total-type-count: u32,
        total-type-capacity: u32,
    }

    /// Obtém os metadados de uma frequência QIO registrada no mundo.
    get-frequency: func(id: frequency-id) -> option<qio-frequency-summary>;

    /// Cria ou atualiza os metadados globais da frequência.
    set-frequency: func(summary: qio-frequency-summary) -> result<_, string>;

    /// Lista todas as frequências acessíveis para um dado jogador.
    list-frequencies: func(owner: option<string>) -> list<frequency-id>;
}
```

---

### FORK-20: Widgets de Interface Gráfica Ricos (Campos Numéricos e LEDs)

#### Contexto e Necessidade
A interface gráfica do Adaptador de Redstone (`GuiQIORedstoneAdapter`) requer um campo de entrada de texto com validação numérica para configurar a quantidade de disparo (`countTrigger`: `u64`). O Drive Array (`GuiQIODriveArray`) renderiza o estado de 12 unidades de disco em LEDs coloridos (apagado, verde, amarelo, vermelho) e exibe uma tela agregada de capacidade. A API atual do Horizon prevê apenas slots de itens normais e botões estáticos.

#### Assinatura WIT Proposta
```wit
interface custom-ui {
    record text-field-spec {
        id: u32,
        x: s32,
        y: s32,
        width: u32,
        height: u32,
        max-characters: u32,
        is-numeric: bool,
        initial-text: string,
    }

    record status-led-spec {
        slot-index: u32,
        /// 0: Desligado, 1: Offline, 2: Operacional/Verde, 3: Quase Cheio/Amarelo, 4: Cheio/Vermelho
        state: u8,
    }

    /// Configura widgets avançados em uma tela de contêiner aberta.
    set-screen-widgets: func(player-id: u64, text-fields: list<text-field-spec>, leds: list<status-led-spec>);
}
```

---

### FORK-21: Emissão Seletiva de Sinal de Redstone por Face (`set-redstone-output-faces`)

#### Contexto e Necessidade
O Adaptador de Redstone QIO (`TileEntityQIORedstoneAdapter:48`) possui uma regra geométrica precisa: quando a condição é atendida e o bloco está ativo, ele **emite sinal de força 15 em 5 faces adjacentes**, mas **NUNCA emite na face traseira** (`side != getOppositeDirection()`).  
A API atual do Horizon em `interface %world` possui apenas `get-redstone` (leitura do sinal recebido), sendo incapaz de fazer um bloco do mod atuar como emissor de redstone direcional.

#### Assinatura WIT Proposta
```wit
interface %world {
    /// Define a força de sinal de redstone emitida (0 a 15) para cada uma das seis faces do bloco.
    /// A lista deve conter exatamente 6 valores correspondentes a:
    /// [0: DOWN, 1: UP, 2: NORTH, 3: SOUTH, 4: WEST, 5: EAST].
    set-redstone-output-faces: func(
        dimension: string,
        x: s32,
        y: s32,
        z: s32,
        levels: list<u8>
    ) -> result<_, string>;
}
```

---

### FORK-23: Componentes de Dados em ItemStacks (`DriveContents` e `DriveMetadata`)

#### Contexto e Necessidade
As unidades de armazenamento QIO (`ItemQIODrive`) transportam seus inventários internamente através de anexos de componentes de dados (`MekanismDataComponents.DRIVE_CONTENTS` e `DRIVE_METADATA`). Quando um jogador retira um drive de uma matriz para transportá-lo ou colocá-lo em outro Drive Array, os itens e suas quantidades precisam permanecer intactos no ItemStack.  
O `ItemStack` atual da SDK do Horizon suporta apenas `id` e `count`.

#### Assinatura WIT Proposta
```wit
interface item-components {
    /// Grava um buffer binário opaco de componente de dados no ItemStack.
    set-stack-component: func(stack-handle: u64, component-id: string, payload: list<u8>) -> result<_, string>;

    /// Lê o buffer binário de um componente de dados associado ao ItemStack.
    get-stack-component: func(stack-handle: u64, component-id: string) -> option<list<u8>>;
}
```

---

### FORK-24: Sincronização em Lote Cliente-Servidor (`BulkQIOData`)

#### Contexto e Necessidade
Uma frequência QIO em servidores avançados pode conter milhares de tipos de itens e bilhões de contagens. O envio de cada item individualmente causaria sobrecarga massiva de rede. O Mekanism utiliza `BulkQIOData` e `PacketUpdateItemViewer` com streams compactados de pares `(UUIDAwareHashedItem, long)`.

#### Assinatura WIT Proposta
```wit
interface net-messages {
    /// Transmite payload de rede para um jogador específico com menu aberto.
    send-player-payload: func(player-id: u64, channel: string, payload: list<u8>) -> result<_, string>;

    /// Transmite payload para todos os jogadores no raio do bloco.
    broadcast-block-payload: func(
        dimension: string,
        x: s32,
        y: s32,
        z: s32,
        channel: string,
        payload: list<u8>
    ) -> result<_, string>;
}
```

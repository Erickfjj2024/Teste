# Requisitos de Fork do Horizon (FORK-n): Grupo Transporte e Utilidades

**ID da Tarefa**: T20261109-204014-32i  
**Grupo**: transporte_utilidades  
**Origem**: Mekanism 1.21.1-10.7.19.85 (`java/`)

> Conforme as regras do projeto, **não existem "limites do Horizon"**. Toda restrição atual da API do Horizon é isolada com `// FORK-n` e formalizada neste documento com sua motivação no Java e assinatura WIT exata proposta para incorporação no fork.

---

## 1. FORK-3: Roteamento Logístico com Custos Customizados e Redstone por Face

### Origem no Java
- `RestrictiveTransporter.java:13-15`: `getCost() = 1_000`
- `DiversionTransporter.java:182-187`: Modos `HIGH` e `LOW` de controle de redstone por face determinando se o duto conecta e insere itens
- `TransporterPathfinder.java:451`: `double tentativeG = currentScore + neighborTransmitter.getCost();`

### O que falta na API do Horizon
Atualmente, a API de redes de transmissores do Horizon (`TransmitterSpec` / `transmitter-faces`) opera com conectividade booleana estática e não oferece suporte a:
1. Custos de aresta ponderados no algoritmo de A* (necessário para o custo 1000 do Restrictive Transporter);
2. Reavaliação dinâmica de conectividade baseada em sinal de redstone recebido em cada face individual (`DiversionControl`).

### Assinatura WIT Proposta (`port/api/transporter-routing.wit`)
```wit
interface transporter-routing {
    enum diversion-control {
        disabled,
        high,
        low,
    }

    record routing-node {
        pos: tuple<s32, s32, s32>,
        cost: f64,
        open-faces: list<bool>,
        diversion-modes: list<diversion-control>,
        powered: bool,
    }

    /// Executa o cálculo de caminho A* no grafo de transportadores respeitando custos e redstone.
    find-transporter-route: func(
        dimension: string,
        start: tuple<s32, s32, s32>,
        destination: tuple<s32, s32, s32>,
        nodes: list<routing-node>,
    ) -> result<list<tuple<s32, s32, s32>>, string>;
}
```

---

## 2. FORK-14: Carregamento de Chunks por Mod (`IChunkLoader`)

### Origem no Java
- `TileEntityDimensionalStabilizer.java:46-68, 117`: Gerenciamento de tickets de chunks na matriz 5×5
- `TileComponentChunkLoader.java`: `refreshChunkTickets()`
- `StabilizedChunks.java`: bitset de chunks serializados

### O que falta na API do Horizon
O Horizon mod SDK atual (`chunks.wit`) permite inspecionar se um chunk está carregado, mas não fornece API para mods registrarem "chunk tickets" forçados no servidor do jogo para manter chunks ativos sem jogadores por perto.

### Assinatura WIT Proposta (`port/api/chunk-loader.wit`)
```wit
interface chunk-tickets {
    record chunk-ticket {
        dimension: string,
        chunk-x: s32,
        chunk-z: s32,
        load: bool,
    }

    /// Atualiza em lote os tickets de chunks mantidos pelo bloco modificado.
    sync-chunk-tickets: func(
        tickets: list<chunk-ticket>,
    ) -> result<u32, string>;
}
```

---

## 3. FORK-23: Captura e Reinstanciação Genérica de Blocos (Cardboard Box)

### Origem no Java
- `BlockCardboardBox.java:46-56`: `blockData.tryPlaceIntoWorld(world, pos, player)`
- `ItemBlockCardboardBox.java:68-100`: `new BlockData(registryAccess, state, tile)`
- `BlockData.java`: Serialização de `BlockState` e `CompoundTag` do tile entity

### O que falta na API do Horizon
Não há chamada no SDK do Horizon para capturar atomicamente o estado completo de um bloco (`BlockState` com propriedades) e os dados NBT de seu `BlockEntity`, serializá-los em um componente de dados de item, e posteriormente reinstanciá-los com fidelidade no mundo.

### Assinatura WIT Proposta (`port/api/block-capture.wit`)
```wit
interface block-capture {
    record captured-block {
        block-name: string,
        properties: list<tuple<string, string>>,
        tile-data: list<u8>,
        items: list<option<item-stack>>,
        energy: option<u64>,
    }

    /// Captura atomicamente o bloco e seu BlockEntity no mundo.
    capture-block: func(
        dimension: string,
        pos: tuple<s32, s32, s32>,
    ) -> result<captured-block, string>;

    /// Reinstancia o bloco capturado no mundo com seus dados originais.
    restore-block: func(
        dimension: string,
        pos: tuple<s32, s32, s32>,
        data: captured-block,
    ) -> result<bool, string>;
}
```

---

## 4. FORK-7 / FORK-23: Contêineres de 54 Slots e Abertura Portátil na Mão (Personal Storage)

### Origem no Java
- `PersonalStorageManager.java:150-154`: Contêiner com 54 slots (6×9)
- `ItemBlockPersonalStorage.java:43-67`: Abertura de GUI de contêiner diretamente segurando o item na mão (sem posicionar bloco no mundo)
- `TileEntityPersonalStorage.java:136`: `persistsToItem = true`

### O que falta na API do Horizon
1. O teto nativo de contêineres do Horizon para mods WASM é atualmente 32 slots; o Personal Chest/Barrel requer 54 slots.
2. Contêineres atualmente só podem ser abertos vinculados a um bloco posicionado no mundo (`BlockEntity`), não a partir de um item segurado pelo jogador na mão (`useOnContext` portátil).

### Assinatura WIT Proposta (`port/api/personal-storage.wit`)
```wit
interface personal-storage {
    /// Abre a GUI de um contêiner de 54 slots a partir de um item na mão do jogador.
    open-handheld-container: func(
        player-id: string,
        title: string,
        slot-count: u32,
        initial-items: list<option<item-stack>>,
    ) -> result<u64, string>; // Retorna o ID da sessão de menu aberta
}
```

---

## 5. FORK-30: Busca de Entidades em AABB e Recarga de Itens de Jogador (Chargepad)

### Origem no Java
- `TileEntityChargepad.java:58-77`: `level.getEntitiesOfClass(LivingEntity.class, new AABB(...))`
- `TileEntityChargepad.java:90-96`: Varredura e recarga do primeiro item com capacidade de energia no inventário do jogador

### O que falta na API do Horizon
O Horizon mod SDK permite ler jogadores online, mas não oferece busca espacial por AABB de entidades vivas genéricas (incluindo Robits), nem acesso/mutação a itens recarregáveis no inventário de um jogador específico a partir do tick de um bloco.

### Assinatura WIT Proposta (`port/api/entity-charging.wit`)
```wit
interface entity-charging {
    record aabb {
        min-x: f64, min-y: f64, min-z: f64,
        max-x: f64, max-y: f64, max-z: f64,
    }

    record rechargeable-entity {
        entity-id: string,
        is-player: bool,
        is-robit: bool,
    }

    /// Busca entidades vivas dentro da AABB do bloco.
    get-entities-in-bounds: func(
        dimension: string,
        bounds: aabb,
    ) -> list<rechargeable-entity>;

    /// Tenta recarregar um item recarregável no inventário do jogador ou o Robit.
    charge-entity: func(
        entity-id: string,
        max-amount: u64,
    ) -> u64; // Quantidade de FE efetivamente transferida
}
```

---

## 6. FORK-31: Evento de Vibração Sísmica e Leitura de Coluna Geológica (Seismic Vibrator)

### Origem no Java
- `TileEntitySeismicVibrator.java:79`: `level.gameEvent(null, MekanismGameEvents.SEISMIC_VIBRATION, worldPosition)`
- `TileEntitySeismicVibrator.java:142-151`: `getColumnAt(chunkRelativeX, chunkRelativeZ)` lendo toda a coluna Y de blocos do chunk

### O que falta na API do Horizon
1. Emissão de `GameEvent` customizado para a mecânica de vibração do jogo.
2. Consulta de coluna vertical de blocos em lote para leitura pelo Seismic Reader ou computadores integrados.

### Assinatura WIT Proposta (`port/api/seismic.wit`)
```wit
interface seismic {
    /// Dispara evento de jogo sísmico no mundo.
    emit-seismic-event: func(
        dimension: string,
        pos: tuple<s32, s32, s32>,
    ) -> bool;

    /// Obtém os blocos de uma coluna vertical inteira dentro de um chunk.
    get-chunk-column: func(
        dimension: string,
        block-x: s32,
        block-z: s32,
        min-y: s32,
        max-y: s32,
    ) -> list<tuple<s32, string>>;
}
```

---

## 7. FORK-32: Consulta de Tags de Itens em Tempo de Execução (Oredictionificator)

### Origem no Java
- `TileEntityOredictionificator.java:112-120`: `stack.getTags().map(TagKey::location)`

### O que falta na API do Horizon
O Horizon mod SDK atual não expõe um método para consultar quais tags um determinado `ItemStack` possui durante o tick do mod WASM.

### Assinatura WIT Proposta (`port/api/tags.wit`)
```wit
interface item-tags {
    /// Retorna a lista de identificadores de tags atribuídos ao item.
    get-item-tags: func(item-id: string) -> list<string>;
}
```

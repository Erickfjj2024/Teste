# Requisitos de Fork do Horizon: Fission Reactor (Reator de Fissão)

Este documento detalha os requisitos de extensão da API do Horizon identificados durante a revisão de lógica do **Fission Reactor** (`port/mod/src/fission.rs`), seguindo o formato e convenções de `port/API-REQUISITOS.md`.

---

## 1. FORK-21 (Detalhamento para Fissão): Suporte a Portas Lógicas e Redstone de Multiblocos

### Origem no Java
`TileEntityFissionReactorLogicAdapter.java:88-128, 146-156`:
O Fission Reactor Logic Adapter possui 6 modos de operação:
1. `DISABLED`: Passivo.
2. `ACTIVATION`: Lê sinal de redstone de entrada (`canFunction()`) para ligar/desligar o reator (`setActive(true/false)`).
3. `TEMPERATURE`: Emite sinal forte de redstone (nível 15) para fora se a temperatura do reator for $\ge 1200\text{ K}$ (`MIN_DAMAGE_TEMPERATURE`).
4. `CRITICAL_WASTE_LEVEL`: Emite sinal de redstone 15 se o resíduo armazenado for $\ge 90\%$ da capacidade (`wasteTank.getStored() >= capacity * excessWasteRatio`).
5. `DAMAGED`: Emite sinal de redstone 15 se o dano do reator for $\ge 100.0$ (`MAX_DAMAGE`).
6. `DEPLETED`: Emite sinal de redstone 15 se o tanque de combustível estiver vazio (`fuelTank.isEmpty()`).

Além disso, as portas (`TileEntityFissionReactorPort.java:143-145`) emitem nível de comparador analógico (0 a 15) proporcional ao nível de combustível armazenado (`MekanismUtils.redstoneLevelFromContents`).

### Limite no Horizon Atual
`BlockEntityUpdate` (`port/api/horizon.wit:1230-1250`) não possui campos para especificar nível de emissão de redstone por face (`redstone-output: list<u8>`), nem evento para notificação de pulso de redstone recebido.

### Proposta de Assinatura WIT
```wit
/// Em horizon.wit (interface block-entities ou world)

record redstone-side-power {
    side: direction,
    power: u8, // 0..15
}

interface redstone-handler {
    /// O bloco recebe uma alteração de sinal de redstone vinda de vizinhos
    neighbor-redstone-changed: func(pos: block-pos, power: u8, side: direction);
}

/// Extensão em block-entity-update:
record block-entity-update {
    // ... campos existentes ...
    /// Nível de redstone analógico/digital emitido para os vizinhos por face
    redstone-outputs: option<list<u8>>,
    /// Nível analógico emitido para comparadores (0..15)
    comparator-output: option<u8>,
}
```

---

## 2. FORK-29: Uso Condicional de Bloco pelo Mod (`conditional-block-use`)

### Origem no Java (Sintoma S1 em `SINTOMAS_MULTIBLOCOS.md`)
`TileEntityMultiblock.java:218-229`:
```java
if (player.isShiftKeyDown() || !getMultiblock().isFormed()) {
    return ItemInteractionResult.PASS_TO_DEFAULT_BLOCK_INTERACTION;
}
return openGui(player);
```
Quando o reator está desmontado ou inválido, o clique direito do jogador deve repassar a ação ao Minecraft (`PASS_TO_DEFAULT_BLOCK_INTERACTION`), permitindo colocar o bloco segurado na mão (ex: colocar outro casing ou port). Apenas quando o reator está formado e o jogador não está agachado o bloco consome a ação e abre a tela GUI.

### Limite no Horizon Atual
No Horizon atual, o comportamento de abrir tela é fixado na declaração do tipo do bloco (`ContainerSpec.menu = true` em `lib.rs:1854`). O motor WASM intercepta o clique antes do mod e sempre abre a GUI ou consome o clique, impossibilitando a colocação de blocos adjacentes sem agachar (Shift).

### Proposta de Assinatura WIT
```wit
/// Em horizon.wit (interface world ou block-use)

enum block-interaction-result {
    /// O mod consumiu a ação (ex: abriu tela ou alterou modo com configurador)
    consume,
    /// O mod rejeitou a ação; o jogo deve repassar para o item da mão (ex: colocar bloco)
    pass-to-item,
}

interface block-interaction-handler {
    /// Chamado no clique direito em um bloco com lógica personalizada
    on-block-use: func(player: player-id, pos: block-pos, hand: interaction-hand, sneak: bool, held-item: option<item-stack>) -> block-interaction-result;
}
```

---

## 3. FORK-30: Spawn de Partículas do Servidor / Mod (`world.spawn-particles`)

### Origem no Java (Sintoma S2 em `SINTOMAS_MULTIBLOCOS.md`)
`TileEntityMultiblock.java:297-312`:
Ao formar a estrutura, o servidor envia faíscas ao longo das 12 arestas do cuboide (`SparkleAnimation`) para todos os jogadores a até 40 blocos de distância.

### Limite no Horizon Atual
O Horizon não possui função na interface `world` para emitir partículas no cliente com parâmetros configuráveis de posição, velocidade e tipo.

### Proposta de Assinatura WIT
```wit
/// Em horizon.wit (interface world)

record particle-spawn-request {
    dimension: string,
    particle-type: string, // ex: "minecraft:electric_spark", "minecraft:smoke"
    pos: vec3d,
    offset: vec3d,
    speed: f64,
    count: u32,
    /// Distância máxima de visão dos jogadores receptores
    view-distance: f64,
}

interface world {
    /// Emite partículas no mundo sincronizadas com os clientes próximos
    spawn-particles: func(request: particle-spawn-request);
}
```

---

## 4. FORK-31 (ou FORK-19 refinado): Renderização Tridimensional de Fluido no Interior de Multiblocos

### Origem no Java (Sintoma S4 em `SINTOMAS_MULTIBLOCOS.md`)
`RenderFissionReactor.java:30-50`, `MultiblockTileEntityRenderer.java`:
Quando o reator é construído com janelas de `mekanism:reactor_glass`, o refrigerante (água translúcida azul ou sódio líquido) e o vapor de escape gerado são renderizados no volume interno do reator com altura proporcional ao volume armazenado.

### Limite no Horizon Atual
O Horizon atualmente não oferece primitiva declarativa para renderizar um paralelepípedo de fluido/gás dentro das coordenadas internas de um multibloco ativo.

### Proposta de Assinatura WIT
```wit
/// Em horizon.wit (interface render ou declared-block-renderers)

record multiblock-fluid-render {
    structure-id: string,
    dimension: string,
    min: block-pos,
    max: block-pos,
    fluid: string,
    fill-ratio: f32, // 0.0 .. 1.0
    color: u32,
    gaseous: bool,
}

interface multiblock-render {
    /// Atualiza ou registra o volume de fluido renderizado internamente no cliente
    update-multiblock-fluid: func(data: multiblock-fluid-render);
    /// Remove o renderizador ao desmontar a estrutura
    clear-multiblock-fluid: func(structure-id: string);
}
```

---

## 5. FORK-32: Consulta de Entidades Vivas em Bounding Box para Zona de Radiação Térmica

### Origem no Java
`FissionReactorMultiblockData.java:504-519`:
```java
List<LivingEntity> entitiesToRadiate = getLevel().getEntitiesOfClass(LivingEntity.class, hotZone);
for (LivingEntity entity : entitiesToRadiate) {
    radiationManager.radiate(entity, magnitude);
}
```
A cada 20 ticks, o reator queima e emite dose de radiação diretamente para quaisquer entidades biológicas (jogadores, animais, monstros) que estejam pisando na zona quente interna (`hotZone = getMinPos().offset(1, 1, 1) até getMaxPos().offset(-1, -1, -1)`).

### Limite no Horizon Atual
O Horizon possui `players::online()`, mas não possui API para consultar entidades genéricas (`LivingEntity`) presentes em uma caixa delimitadora (`AABB`) de coordenadas.

### Proposta de Assinatura WIT
```wit
/// Em horizon.wit (interface world ou entities)

record entity-ref {
    id: u32,
    kind: string,
    pos: vec3d,
}

interface world {
    /// Retorna todas as entidades contidas dentro de uma caixa de coordenadas
    get-entities-in-bounds: func(dimension: string, min: vec3d, max: vec3d) -> list<entity-ref>;
    /// Aplica dose de radiação ou dano a uma entidade específica
    radiate-entity: func(entity-id: u32, dose: f64);
}
```

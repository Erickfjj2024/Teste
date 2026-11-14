# Requisitos de Fork de API: Industrial Turbine (Turbina Industrial)

**Identificador da Tarefa:** `T20261109-220423-01q`  
**Escopo:** `port/mod/src/turbine.rs`  
**Referência Java:** Mekanism 1.21.1-10.7.19.85  

Este documento lista as limitações da API do Horizon que impedem a reprodução 100% idêntica do comportamento do Mekanism Java na Turbina Industrial, com as propostas de extensão de interface WIT correspondentes (formato `port/API-REQUISITOS.md`).

---

## FORK-20: Modos de Despejo de Vapor na GUI da Turbina (`dumpMode`)

### Descrição e Comportamento Java
No Mekanism Java (`TurbineMultiblockData.java:69, 168-179`, `GuiIndustrialTurbine.java`), a Turbina possui um botão que permite ao jogador alternar o modo de descarte de vapor (`dumpMode`) entre três estados:
1. `IDLE`: não descarta vapor;
2. `DUMPING_EXCESS`: descarta automaticamente o vapor excedente além de uma fração configurada da capacidade máxima;
3. `DUMPING`: descarta ativamente todo o vapor contido na turbina.

### Limitação no Horizon
A API atual de menus (`interface ui`, patch 0007) permite exibir barras de energia e medidores de tanques (`types: GAUGE_TYPES`), mas a tela declarada simplificada da válvula (`turbine_valve_menu` em `port/tools/convert.py`) não possui suporte a botões com retorno de clique no servidor para alternância de modo enum no bloco de válvula (`menu-button: func(...)`).

### Proposta de Extensão WIT
```wit
interface multiblock-controls {
    enum dump-mode {
        idle,
        dumping-excess,
        dumping,
    }

    /// Obtém o modo de despejo configurado no multibloco.
    get-multiblock-dump-mode: func(dimension: string, pos: block-pos) -> dump-mode;

    /// Define o modo de despejo do multibloco via interação na interface gráfica.
    set-multiblock-dump-mode: func(dimension: string, pos: block-pos, mode: dump-mode);
}
```

---

## FORK-21: Interação com a Mão Vazia para Remoção de Pás do Rotor

### Descrição e Comportamento Java
No Mekanism Java (`BlockTurbineRotor.java:52-59`), o jogador pode remover pás instaladas em um rotor da turbina simplesmente agachando e clicando com a mão vazia (`player.isShiftKeyDown() && stack.isEmpty()`).

### Limitação no Horizon
Na interface `horizon.wit` (`server-events.item-used`), o evento de clique direito é acionado apenas quando o jogador utiliza um item registrado (`item-used: func(player: player, stack: item-stack, pos: option<block-pos>) -> bool`). O Horizon não fornece hook no servidor para interação de clique com a mão vazia em blocos. Como contingência, o port exige que o jogador segure uma `mekanism:turbine_blade` enquanto agacha para acionar a remoção (`lib.rs:2160-2173`).

### Proposta de Extensão WIT
```wit
interface server-events {
    /// Disparado quando um jogador clica com a mão vazia em um bloco do mundo.
    block-activated: func(player: player, dimension: string, pos: block-pos, sneaking: bool) -> bool;
}
```

---

## FORK-10: Animação e Áudio do Rotor em 3D

### Descrição e Comportamento Java
No Mekanism Java (`RenderTurbineRotor.java`, `TileEntityTurbineRotor.java:29-32`), o rotor renderiza um eixo com modelo 3D dinâmico que gira continuamente na velocidade angular proporcional a `clientFlow / maxFlow`, acompanhado por áudio contínuo de turbina.

### Limitação no Horizon
O Horizon WASM executa a lógica do lado do servidor e não oferece suporte à injeção de renderizadores de entidades de bloco animados com modelos procedurais ou canais de áudio de loop contínuo no cliente.

# Requisitos de Plataforma Horizon (FORK): Dynamic Tank e Induction Matrix

**Grupo**: `dynamic_induction`  
**Referência**: Mekanism Java `1.21.1-10.7.19.85` (`java/`)  
**Arquivos de Origem**:
- `java/main/java/mekanism/common/content/tank/TankMultiblockData.java`
- `java/main/java/mekanism/common/tile/multiblock/TileEntityDynamicTank.java`
- `java/main/java/mekanism/common/tile/multiblock/TileEntityDynamicValve.java`
- `java/main/java/mekanism/common/content/matrix/MatrixMultiblockData.java`
- `java/main/java/mekanism/common/tile/multiblock/TileEntityInductionCasing.java`
- `java/main/java/mekanism/common/tile/multiblock/TileEntityInductionPort.java`
- `java/main/java/mekanism/client/render/tileentity/RenderDynamicTank.java`

---

## 1. Tabela Resumo de Requisitos FORK

| ID | Requisito / Requirement | Origem no Java / Java Source | API Pública / Public API |
|---|---|---|---|
| **FORK-21** | Emissão de sinal analógico de redstone para Comparadores (0 a 15) proporcional ao nível de preenchimento | `TankMultiblockData.java:140`, `MatrixMultiblockData.java:70` | `interface %world` (`set-comparator-output`) |
| **FORK-29** | Acesso e abertura de interface gráfica (GUI) delegada a partir de qualquer bloco membro da estrutura formada | `TileEntityDynamicTank.java:40`, `TileEntityInductionCasing.java:41-43` | `interface block-entities` (`delegate-menu`) |
| **FORK-30** | Interação direta de itens com blocos no mundo (clique com balde/tanque em carcaça sem abrir GUI) | `TileEntityDynamicTank.java:36`, `FluidUtils.handleTankInteraction` | `interface player-interaction` (`on-block-use-item`) |
| **FORK-19** | Renderização translúcida 3D de fluidos/gases na cavidade interna visível através de Vidro Estrutural | `RenderDynamicTank.java:30-80` | `interface declared-renderers` (`cuboid-fluid-renderer`) |

---

## 2. Detalhamento e Assinaturas WIT Propostas

### FORK-21: Emissão de Sinal Analógico de Redstone para Comparadores (`set-comparator-output`)

#### Contexto e Necessidade
No Mekanism Java:
- Válvulas do Dynamic Tank (`TileEntityDynamicValve.java:62-64`) emitem sinal de 0 a 15 baseado no percentual de fluido ou gás armazenado no tanque: `MekanismUtils.redstoneLevelFromContents(stored, capacity)`.
- Portas da Induction Matrix (`TileEntityInductionPort.java:70-72`) emitem sinal de 0 a 15 proporcional à energia armazenada em relação à capacidade máxima: `redstoneLevelFromContents(storedEnergy, maxEnergy)`.
A API atual do Horizon (`port/api/horizon.wit`) possui apenas `world.get-redstone` para ler sinais recebidos de vizinhos; não existe função para o mod definir a intensidade de saída de comparador que o bloco entity emite.

#### Assinatura WIT Proposta (`port/api/horizon.wit`)
```wit
interface %world {
    /// Define a intensidade de sinal analógico (0 a 15) emitida por um block entity
    /// para comparadores de redstone adjacentes ao bloco.
    set-comparator-output: func(dimension: string, x: s32, y: s32, z: s32, level: u8);
}
```

---

### FORK-29: Abertura Delegada de Menus em Multiblocos (`delegate-menu`)

#### Contexto e Necessidade
No Mekanism Java:
- Clicar com a mão vazia em qualquer bloco formado de `Dynamic Tank` (seja carcaça, válvula ou vidro estrutural) abre a GUI do tanque (`TileEntityDynamicTank.java:40-47`).
- Clicar em qualquer bloco formado de `Induction Casing` abre a GUI da matriz (`TileEntityInductionCasing.java:17-45`).
No Horizon atual, uma tela só pode ser aberta se o bloco específico possuir `menu: true` em sua declaração `block-entity-kind`. Os casings do Dynamic Tank e da Matrix são declarados sem menu (`menu: false` / `container: None`), forçando o jogador a interagir exclusivamente com válvulas ou portas.

#### Assinatura WIT Proposta (`port/api/horizon.wit`)
```wit
interface block-entities {
    record menu-delegation {
        /// Posição do bloco controlador que detém os slots e o menu real
        target-pos: block-pos,
    }

    /// Permite que um membro de multibloco encaminhe a abertura de tela
    /// para outro bloco do mesmo grupo.
    set-menu-delegate: func(dimension: string, member-pos: block-pos, target-pos: block-pos);
}
```

---

### FORK-30: Interação Direta com Recipientes de Fluido no Mundo (`on-block-use-item`)

#### Contexto e Necessidade
No Mekanism Java (`TileEntityDynamicTank.java:36`):
- O jogador pode clicar com o botão direito em qualquer bloco formado do Dynamic Tank segurando um balde de água/lava ou tanque químico portátil para esvaziá-lo ou enchê-lo diretamente no mundo, sem necessidade de abrir a interface gráfica.
- O Horizon atualmente só permite mover fluidos por balde colocando o item fisicamente nos slots dedicados da interface gráfica.

#### Assinatura WIT Proposta (`port/api/horizon.wit`)
```wit
interface player-interaction {
    /// Chamado quando um jogador interage com um bloco segurando um item.
    /// Retorna verdadeiro se o mod consumiu a interação (impedindo o comportamento padrão do jogo).
    block-item-used: func(player: string, dimension: string, pos: block-pos, hand-stack: item-stack) -> option<item-stack>;
}
```

---

### FORK-19: Renderizador Declarado de Fluidos em Multiblocos (`cuboid-fluid-renderer`)

#### Contexto e Necessidade
No Mekanism Java (`RenderDynamicTank.java`):
- Quando o Dynamic Tank ou Caldeira utiliza Vidro Estrutural, o interior renderiza uma malha 3D dinâmica do volume do fluido preenchido, animada e texturizada de acordo com a substância.
- No Horizon, a cavidade interna permanece transparente/invisível no mundo físico, dependendo exclusivamente dos medidores na GUI.

#### Assinatura WIT Proposta (`port/api/fork-patches/0011-Declared-block-renderers-API-block-11.patch`)
```wit
interface renderers {
    record multiblock-fluid-render {
        min: block-pos,
        max: block-pos,
        fluid: string,
        fill-percentage: f32,
    }

    set-multiblock-fluid-render: func(dimension: string, render: multiblock-fluid-render);
    clear-multiblock-fluid-render: func(dimension: string, min: block-pos);
}
```

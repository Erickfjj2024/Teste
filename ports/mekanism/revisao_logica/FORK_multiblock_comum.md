# Propostas de Fork (Horizon API): Mecanismo Comum de Multiblocos e Bounding Blocks

**Escopo:** Requisitos do SDK e Engine do Horizon para suporte 100% fiel à lógica de multiblocos e blocos delimitadores do Mekanism Java 1.21.1-10.7.19.85  
**Documento de Origem:** `port/API-REQUISITOS.md` e `.saida/T20261109-230537-ppj/anexos/SINTOMAS_MULTIBLOCOS.md`  
**Regra do Projeto:** Não existem "limites do Horizon". O que falta na API do jogo é isolado no código (`// FORK-n`) e especificado com assinaturas WIT exatas para implementação no fork.

---

## 1. Tabela de Requisitos de Fork

| ID | Requisito / Nome | Origem no Java (arquivo:linha) | Sintoma em Jogo | WIT / Tipo de Extensão |
|---|---|---|---|---|
| **FORK-29** | Uso de bloco condicional e interceptação de menu | `TileEntityMultiblock.java:218-229`<br>`BlockBounding.java:101-120` | **S1:** Jogador não consegue colocar blocos em cima da carcaça/válvula sem agachar quando não formada. | `use-block` callback / `container-spec.menu-mode` |
| **FORK-30** | Emissão de partículas pelo mod no mundo | `TileEntityMultiblock.java:297-312`<br>`SparkleAnimation.java:20-60` | **S2:** Sem animação visual de faíscas ao longo das arestas na formação da estrutura. | `world.spawn-particles` |
| **FORK-31** | Renderização volumétrica de fluidos em multiblocos | `MultiblockTileEntityRenderer.java:45-80`<br>`RenderDynamicTank.java:25-50` | **S4:** Tanque com vidro estrutural não mostra a água/gás subindo no seu interior. | `render.multiblock-fluid` |
| **FORK-32** | Redirecionamento nativo e vínculo de Bounding Blocks | `BlockBounding.java:101-177`<br>`TileEntityBoundingBlock.java:30-65` | **B2/B3:** Bounding block não abre menu da máquina principal ou deixa órfãos no mundo se quebrado. | `bounding-spec` em `block-spec` |
| **FORK-33** | Múltiplos grupos por bloco em `multiblock-spec` | `IStructuralMultiblock.java:12-25`<br>`TileEntityStructuralGlass.java:15` | **M1:** Vidro estrutural e dispersores não formam em tanques ou turbinas devido a matching rígido de string. | `groups: list<string>` ou parser de vírgula |
| **FORK-34** | Mascaramento dinâmico de capacidades (Capability Masking) | `TileEntityMultiblock.java:178-185`<br>`MultiblockData.java:248-265` | **S3:** Válvula de tanque destruído continua aceitando fluidos de tubos vizinhos. | `set-block-capabilities-active` |

---

## 2. Especificação Técnica e Assinaturas WIT

### 2.1 FORK-29: Uso de Bloco Condicional e Abertura de Menu Dinâmica

#### Problema no Horizon Atual:
No Horizon, `container-spec.menu: bool` é uma propriedade estática declarada no registro do bloco (`init`). Se `menu: true`, qualquer clique direito no bloco abre compulsoriamente a interface gráfica no cliente e no servidor, consumindo o clique.  
No Java (`TileEntityMultiblock.java:219-221`), o método `onActivate` retorna `PASS_TO_DEFAULT_BLOCK_INTERACTION` quando o jogador está agachado (`player.isShiftKeyDown()`) OU quando a estrutura **não está formada** (`!getMultiblock().isFormed()`). Isso permite ao jogador colocar blocos da mão sobre a válvula ou carcaça livremente durante a construção.

#### Assinatura WIT Proposta (`horizon.wit`):

```wit
interface block-interaction {
    enum interaction-result {
        /// Passes through to normal item use / block placement (vanilla behavior)
        pass,
        /// Consumes the interaction without opening a menu
        consume,
        /// Opens the container menu associated with the block
        open-menu,
    }

    /// Called when a player right-clicks a block in the world.
    /// Allows the mod to determine whether to open a menu, consume, or pass.
    on-block-use: func(player: player, hand: interaction-hand, pos: block-pos, sneaking: bool) -> interaction-result;
}
```

No registro do bloco:
```wit
record container-spec {
    slots: u32,
    faces: list<list<u32>>,
    inputs: list<u32>,
    outputs: list<u32>,
    /// How the menu is opened. When 'dynamic', delegates to `on-block-use`
    menu-mode: menu-activation-mode,
}

enum menu-activation-mode {
    disabled,
    always,
    dynamic,
}
```

---

### 2.2 FORK-30: Emissão de Partículas pelo Mod (`world.spawn-particles`)

#### Problema no Horizon Atual:
O mod WASM não possui nenhuma função no import `world` para instanciar partículas visuais no cliente do jogo. No Java, o efeito de faíscas azuis/índigo ao redor de toda a moldura da estrutura (`SparkleAnimation`) é um feedback essencial de que o multibloco foi montado corretamente.

#### Assinatura WIT Proposta (`horizon.wit`):

```wit
interface world {
    record particle-spec {
        /// Particle identifier (e.g., "minecraft:electric_spark", "minecraft:happy_villager", "mekanism:sparkle")
        particle-type: string,
        /// Target world position (x, y, z)
        pos: tuple<f64, f64, f64>,
        /// Velocity vector (vx, vy, vz)
        velocity: tuple<f64, f64, f64>,
        /// Number of particles to spawn
        count: u32,
        /// Positional randomness spread
        spread: f64,
        /// Optional maximum view distance for players to receive packet (default: 40.0)
        view-distance: option<f32>,
    }

    /// Spawns particles in the current world dimension
    spawn-particles: func(spec: particle-spec);
    
    /// Batch particle spawn along a set of lines (e.g. cuboid edges)
    spawn-particle-lines: func(particle-type: string, lines: list<tuple<tuple<f64, f64, f64>, tuple<f64, f64, f64>>>, count-per-line: u32);
}
```

---

### 2.3 FORK-31: Renderização Volumétrica de Fluidos em Multiblocos

#### Problema no Horizon Atual:
O Dynamic Tank, Boiler e Evaporation Plant suportam paredes transparentes de Vidro Estrutural (`mekanism:structural_glass`). No Java, um renderizador especial (`MultiblockTileEntityRenderer`) lê o volume do tanque, a quantidade de líquido e renderiza um prisma quadrangular translúcido no interior da estrutura com a textura e cor do fluido correspondente. No Horizon atual, o interior permanece completamente vazio.

#### Assinatura WIT Proposta (`horizon.wit`):

```wit
interface render {
    record multiblock-fluid-render {
        /// Unique structure identifier
        structure-id: u64,
        /// Inner bounding box minimum corner (inclusive)
        min: block-pos,
        /// Inner bounding box maximum corner (inclusive)
        max: block-pos,
        /// Fluid identifier (e.g. "minecraft:water", "minecraft:lava", "mekanism:heavy_water", "mekanism:brine")
        fluid: string,
        /// ARGB color tint (for chemicals/pigments)
        tint: u32,
        /// Fill level fraction from 0.0 (empty) to 1.0 (completely full)
        fill-fraction: f32,
        /// Flowing animation or still
        animated: bool,
    }

    /// Registers or updates a rendered fluid volume in the world
    set-multiblock-fluid: func(spec: multiblock-fluid-render);

    /// Clears the fluid volume rendering when a multiblock dismantles
    remove-multiblock-fluid: func(structure-id: u64);
}
```

---

### 2.4 FORK-32: Bounding Block Redirection & Lifespan Binding

#### Problema no Horizon Atual:
Máquinas que ocupam mais de um bloco (ex.: Advanced Solar Generator, Wind Generator, Digital Miner, Modification Station) precisam de blocos delimitadores (`mekanism:bounding_block`) para colisão e interação. Atualmente, o mod precisa interceptar quebras e cliques manualmente, o que gera inconsistências com ferramentas de colheita e quedas de blocos.

#### Assinatura WIT Proposta (`horizon.wit`):

```wit
record bounding-spec {
    /// Bounding block block type id (e.g. "mekanism:bounding_block")
    bounding-block-id: string,
    /// Offsets relative to the machine origin based on facing direction
    offsets: list<tuple<string, list<block-pos>>>,
    /// Automatically forward right-clicks to the origin machine block
    redirect-clicks: bool,
    /// Breaking any bounding block triggers breaking of the main machine block
    break-link: bool,
}
```

---

### 2.5 FORK-33: Suporte a Múltiplos Grupos em `multiblock-spec`

#### Problema no Horizon Atual:
Em `crates/horizon-mods/src/runtime.rs:602` (patch 0010), o matching de grupos é feito por igualdade exata:
```rust
multiblock_groups(b).into_iter().any(|g| *g == spec.group)
```
Como o `Structural Glass` e o `Pressure Disperser` pertencem a múltiplos tipos de multibloco (`Dynamic Tank`, `Boiler`, `Turbine`, etc.), foram registrados como `"mekanism:dynamic,mekanism:boiler"`. A engine nunca dividia essa string, criando um grupo fantasma e impossibilitando a formação de estruturas com vidro ou dispersores!

#### Assinatura WIT Proposta (`horizon.wit`):

```wit
record multiblock-spec {
    /// List of multiblock group identifiers that this block can be part of
    groups: list<string>,
}
```

*Nota de retrocompatibilidade:* O código Rust do port foi corrigido para dividir strings contendo vírgula (`spec.group.split(',')`), e o runtime do Horizon deve aplicar o mesmo split no matching de grupos:
```rust
let groups: Vec<&str> = spec.group.split(',').map(|s| s.trim()).collect();
if groups.contains(&candidate_group) { ... }
```

---

### 2.6 FORK-34: Mascaramento Dinâmico de Capacidades (Capability Masking)

#### Problema no Horizon Atual:
Válvulas e portas de multibloco expõem tanques de grande capacidade para permitir conexões externas com tubos. Quando a estrutura é quebrada e resta apenas uma válvula isolada, a engine continua expondo os tanques para blocos adjacentes.

#### Assinatura WIT Proposta (`horizon.wit`):

```wit
interface block-entities {
    /// Allows a mod to selectively enable or disable capability faces on an active block entity
    set-capabilities-enabled: func(pos: block-pos, tanks: bool, energy: bool, items: bool);
}
```

*Implementação no Port:* Enquanto a engine não adiciona a API explícita, o port aplica a regra do Java via `tank-filter = NOTHING` e `tank-faces = []` / `energy-faces = []`, selando a válvula para que canos e cabos não transfiram nada.

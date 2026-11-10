# Requisitos de Fork da API do Horizon: Fusion Reactor

**Tarefa:** T20261109-230438-3c3  
**Escopo:** `port/mod/src/fusion.rs`  
**Referência:** `port/API-REQUISITOS.md`, `SINTOMAS_MULTIBLOCOS.md`, Mekanism Java 1.21.1-10.7.19.85

---

## 1. Contexto

Conforme as regras do projeto, **não existem limites do Horizon**: lacunas da API que impedem o comportamento 100% fiel ao Mekanism Java tornam-se requisitos formais de fork (`FORK-n`), devidamente tipados e isolados com comentários `// FORK-n` no código Rust.

A auditoria de lógica do Fusion Reactor identificou 5 requisitos essenciais da plataforma:

---

## 2. Tabela de Requisitos

| ID | Requisito | Origem no Java | API Pública do Horizon |
|---|---|---|---|
| **FORK-19** | Renderização customizada de núcleo de plasma 3D concêntrico rotativo | `client/render/RenderFusionReactor.java:34-52` | `render.fusion-plasma-core(pos, burning, plasma-temp)` |
| **FORK-21** | Emissão de sinal redstone por face no Logic Adapter | `TileEntityFusionReactorLogicAdapter.java:75-77` | `world.set-redstone-output(pos, side, level)` |
| **FORK-29** | Decisão de uso de bloco condicional pelo mod (Sintoma S1) | `TileEntityMultiblock.java:218-229` | `events.on-block-use -> { consume, pass }` |
| **FORK-30** | Emissão de partículas de formação do multiblock (Sintoma S2) | `TileEntityMultiblock.java:297-312` | `world.spawn-particles(type, pos, vel, count, max-dist)` |
| **FORK-7 / FORK-11** | Dano a entidades em caixa delimitadora AABB (Kill Zone interna 3×3×3) | `FusionReactorMultiblockData.java:287-298` | `entities.hurt-in-box(aabb, damage-type, amount)` |

---

## 3. Especificação Exata WIT Proposta

### 3.1 FORK-19: Renderizador do Núcleo de Plasma (Visual S4)

```wit
interface multiblock-render {
    record plasma-core-state {
        dimension: string,
        center-pos: block-pos,
        burning: bool,
        plasma-temperature: f64,
    }

    /// Atualiza ou cria o renderizador do núcleo de plasma (três camadas esféricas concêntricas
    /// rotativas: Vermelho, Rosa e Laranja com fator de escala dependente da temperatura).
    update-fusion-core: func(state: plasma-core-state);

    /// Remove o renderizador ao extinguir a queima ou desmontar o reator.
    remove-fusion-core: func(dimension: string, center-pos: block-pos);
}
```

### 3.2 FORK-21: Portas Redstone e Logic Adapter

```wit
interface redstone-control {
    /// Emite sinal redstone de 0 a 15 em uma face específica para blocos vizinhos.
    set-face-redstone: func(dimension: string, pos: block-pos, side: flow-side, level: u8);
}
```

### 3.3 FORK-29: Uso de Bloco Condicional (Sintoma S1)

```wit
interface block-interaction {
    enum block-use-outcome {
        /// Consome o clique e abre a interface gráfica registrada
        consume-open-gui,
        /// Repassa o clique para o comportamento padrão (colocar bloco da mão do jogador)
        pass-to-default-interaction,
    }

    /// Invocado antes de abrir qualquer tela de multiblock.
    on-block-use-check: func(
        player-id: string,
        dimension: string,
        pos: block-pos,
        sneaking: bool,
        is-formed: bool
    ) -> block-use-outcome;
}
```

### 3.4 FORK-30: Spawn de Partículas de Formação (Sintoma S2)

```wit
interface particles {
    record particle-batch {
        particle-id: string,
        origin: list<f64>,
        velocity: list<f64>,
        count: u32,
        spread: f64,
    }

    /// Spawna lote de partículas visíveis apenas para jogadores dentro do raio especificado.
    spawn-particles-scoped: func(
        dimension: string,
        particles: list<particle-batch>,
        player-radius: f64
    );
}
```

### 3.5 FORK-7 / FORK-11: Kill Zone no Interior 3×3×3

```wit
interface entity-hazard {
    record aabb-box {
        min-x: f64,
        min-y: f64,
        min-z: f64,
        max-x: f64,
        max-y: f64,
        max-z: f64,
    }

    /// Aplica dano severo instantâneo a todas as entidades dentro da caixa.
    apply-area-damage: func(
        dimension: string,
        box: aabb-box,
        damage-type: string,
        amount: f32
    ) -> u32;
}
```

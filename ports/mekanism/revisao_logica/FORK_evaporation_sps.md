# Requisitos de Fork do Horizon (FORK-n): Thermal Evaporation Plant e SPS

**ID da Tarefa**: T20261109-230225-dg0  
**Grupo**: `evaporation_sps`  
**Arquivos de Escopo**: `port/mod/src/evaporation.rs`, `port/mod/src/sps.rs`  
**Origem**: Mekanism 1.21.1-10.7.19.85 (`java/`)

> Conforme as regras do projeto, **não existem "limites do Horizon"**. Toda restrição atual da API do Horizon é isolada com `// FORK-n` e formalizada neste documento com sua motivação no Java e assinatura WIT exata proposta para incorporação no fork.

---

## 1. FORK-4: Transferência Térmica e Redes de Calor via Válvulas

### Origem no Java
- `TileEntityThermalEvaporationValve.java:30-32`:
  ```java
  @Override
  protected IHeatCapacitorHolder getInitialHeatCapacitors(IContentsListener listener, CachedAmbientTemperature ambientTemperature) {
      return side -> getMultiblock().getHeatCapacitors(side);
  }
  ```
- `HeatAPI.java`: integração com condutores termodinâmicos e fontes externas de calor (resistive heater, fuelwood heater).

### O que falta na API do Horizon
Atualmente, o Horizon suporta conexões de fluidos (`FlowSide`) e energia (`energy_faces`), mas não possui o meio de transmissão de primeira classe para **calor** (`Medium::Heat`). Válvulas de evaporação não conseguem receber calor de tubos termodinâmicos ou aquecedores conectados em suas faces externas.

### Assinatura WIT Proposta (`port/api/heat-network.wit`)
```wit
interface heat-network {
    record heat-capacitor {
        temperature: f64,
        heat-capacity: f64,
        inverse-conduction: f64,
        insulation: f64,
    }

    /// Transfere calor de uma fonte externa ou duto térmico para a válvula do multibloco.
    transfer-heat: func(
        dimension: string,
        pos: tuple<s32, s32, s32>,
        joules: f64,
    ) -> result<f64, string>;
}
```

---

## 2. FORK-7: Dano Letal Físico na Câmara Interna do SPS (`deathZone`)

### Origem no Java
- `SPSMultiblockData.java:198-206`:
  ```java
  private void kill(Level world) {
      if (lastReceivedEnergy > 0L && couldOperate && world.getRandom().nextInt() % 20 == 0) {
          List<Entity> entitiesToDie = world.getEntitiesOfClass(Entity.class, deathZone);
          if (!entitiesToDie.isEmpty()) {
              DamageSource damageSource = MekanismDamageTypes.SPS.source(world, deathZone.getCenter());
              for (Entity entity : entitiesToDie) {
                  entity.hurt(damageSource, lastReceivedEnergy / 1_000F);
              }
          }
      }
  }
  ```

### O que falta na API do Horizon
O Horizon mod SDK permite ler entidades próximas, mas não fornece uma função segura para o mod causar dano arbitrário a todas as entidades vivas dentro de uma caixa delimitadora (`AABB`) tridimensional no mundo do servidor.

### Assinatura WIT Proposta (`port/api/entity-damage.wit`)
```wit
interface entity-damage {
    /// Aplica dano a todas as entidades dentro da caixa [min_pos, max_pos].
    damage-area: func(
        dimension: string,
        min-pos: tuple<s32, s32, s32>,
        max-pos: tuple<s32, s32, s32>,
        damage: f32,
        damage-type: string,
    ) -> result<u32, string>;
}
```

---

## 3. FORK-10: Transformação de Mobs e Raios Cosméticos do SPS

### Origem no Java
- `SPSMultiblockData.java:209-235`:
  Mobs válidos para experimentos (`pig`, `creeper`, `mooshroom`, `villager`) que sobrevivem ao choque são transformados pelo efeito do raio (porco vira Zombified Piglin, creeper vira Charged Creeper, etc.), com geração de raio visual (`setVisualOnly(true)`) e disparo de gatilhos de avanço para jogadores no raio de 15 blocos.

### O que falta na API do Horizon
Não há API no Horizon para invocar raios puramente visuais (`LightningBolt visualOnly`) nem para converter tipos de entidade vanilla mantendo seus atributos e disparando critérios de avanço personalizados (`MekanismCriteriaTriggers.SPS_EXPERIMENT`).

### Assinatura WIT Proposta (`port/api/sps-experiments.wit`)
```wit
interface sps-experiments {
    /// Executa o experimento elétrico nos mobs suportados dentro da câmara do SPS.
    transform-experiment-entities: func(
        dimension: string,
        min-pos: tuple<s32, s32, s32>,
        max-pos: tuple<s32, s32, s32>,
    ) -> result<u32, string>;
}
```

---

## 4. FORK-19 / FORK-31: Renderização de Fluidos Internos e Núcleo do SPS (Sintoma S4)

### Origem no Java
- `RenderThermalEvaporationPlant.java:30-45`: renderiza a coluna de fluido subindo no poço central $2 \times 2$ da torre até a altura proporcional ao fluido acumulado.
- `RenderSPS.java:50-80`: renderiza o feixe de plasma e o campo de partículas entre as coils e o centro da câmara.

### O que falta na API do Horizon
O Horizon renderiza os blocos do multibloco individualmente, mas não oferece primitivas para desenhar volumes de fluidos transparentes escalonáveis ou partículas de renderização customizadas no espaço interno de multiblocos.

### Assinatura WIT Proposta (`port/api/multiblock-rendering.wit`)
```wit
interface multiblock-rendering {
    /// Atualiza ou remove o volume de fluido renderizado no interior de um multibloco.
    set-multiblock-fluid: func(
        dimension: string,
        min-pos: tuple<s32, s32, s32>,
        max-pos: tuple<s32, s32, s32>,
        substance: string,
        fraction: f64,
    );

    /// Limpa a renderização de fluido ao desmontar o multibloco.
    clear-multiblock-fluid: func(
        dimension: string,
        min-pos: tuple<s32, s32, s32>,
        max-pos: tuple<s32, s32, s32>,
    );

    /// Ativa ou atualiza os efeitos visuais internos da câmara do SPS.
    set-sps-chamber-effects: func(
        dimension: string,
        center: tuple<s32, s32, s32>,
        progress: f64,
        active: bool,
    );
}
```

---

## 5. FORK-21: Emissão Analógica de Comparador por Válvulas e Portas

### Origem no Java
- `TileEntityThermalEvaporationValve.java:63-65`:
  ```java
  @Override
  public int getRedstoneLevel() {
      return getMultiblock().getCurrentRedstoneLevel();
  }
  ```
- `TileEntitySPSPort.java:97-99`:
  ```java
  @Override
  public int getRedstoneLevel() {
      return getMultiblock().getCurrentRedstoneLevel();
  }
  ```

### O que falta na API do Horizon
O Horizon não possui uma função de host para blocos informarem ao motor de física de redstone o nível analógico de saída (0 a 15) para comparadores encostados no bloco.

### Assinatura WIT Proposta (`port/api/comparator-output.wit`)
```wit
interface comparator-output {
    /// Informa ao motor do jogo o nível de comparador emitido pelo bloco (0..15).
    set-comparator-signal: func(
        dimension: string,
        pos: tuple<s32, s32, s32>,
        level: u8,
    ) -> result<_, string>;
}
```

---

## 6. FORK-29: Uso Condicional de Bloco de Multibloco (Sintoma S1)

### Origem no Java
- `TileEntityMultiblock.java:218-229`:
  ```java
  if (player.isShiftKeyDown() || !getMultiblock().isFormed()) {
      return PASS_TO_DEFAULT_BLOCK_INTERACTION;
  }
  ```
  Se o multibloco não está formado ou se o jogador está agachado (shift), o clique no bloco **não** consome a ação e permite colocar o bloco da mão normalmente.

### O que falta na API do Horizon
Atualmente, qualquer bloco com GUI registrada consome o clique do jogador, forçando o jogador a agachar para construir em cima de carcaças ou válvulas não formadas. Falta um callback de interação onde o mod decide se consome a ativação ou se delega ao comportamento padrão de posicionamento de bloco.

### Assinatura WIT Proposta (`port/api/block-interaction.wit`)
```wit
interface block-interaction {
    enum click-result {
        consume,
        pass-through,
    }

    /// Determina dinamicamente se o clique com o botão direito abre menu ou passa para o bloco na mão.
    should-activate-block: func(
        dimension: string,
        pos: tuple<s32, s32, s32>,
        player-sneaking: bool,
    ) -> click-result;
}
```

---

## 7. FORK-30: Sistema de Partículas de Formação de Multibloco (Sintoma S2)

### Origem no Java
- `TileEntityMultiblock.java:297-312`:
  ```java
  if (ClientConfig.client.enableMultiblockFormationParticles.get()) {
      new SparkleAnimation(this, renderLocation, length - 1, width - 1, height - 1).run();
  }
  ```

### O que falta na API do Horizon
O Horizon não possui API para disparar grupos de partículas nas arestas de uma caixa 3D arbitrária delimitada pelo mod.

### Assinatura WIT Proposta (`port/api/multiblock-particles.wit`)
```wit
interface multiblock-particles {
    /// Spawna faíscas ao longo das arestas da caixa delimitadora do multibloco formado.
    spawn-sparkle-box: func(
        dimension: string,
        min-pos: tuple<s32, s32, s32>,
        max-pos: tuple<s32, s32, s32>,
        density: u32,
    );
}
```

# Requisitos de Fork do Horizon: Thermoelectric Boiler

Este documento especifica os requisitos de extensão da API do Horizon (WASM / fork) para cobrir comportamentos do Thermoelectric Boiler do Mekanism Java 1.21.1-10.7.19.85 que não podem ser expressos diretamente pela API atual (`horizon.wit`).
Inclui também o detalhamento dos sintomas de jogo identificados em `SINTOMAS_MULTIBLOCOS.md` aplicáveis ao Boiler (S1, S2, S3, S4).

---

## 1. Emissão de Sinal Analógico de Redstone para Comparadores em Válvulas (`FORK-14`)

### Referência no Java
- `TileEntityBoilerValve.java:79-82`:
  ```java
  @Override
  public int getRedstoneLevel() {
      return getMultiblock().getCurrentRedstoneLevel();
  }
  ```
- `BoilerMultiblockData.java:283-285`:
  ```java
  @Override
  protected int getMultiblockRedstoneLevel() {
      return MekanismUtils.redstoneLevelFromContents(waterTank.getFluidAmount(), waterTank.getCapacity());
  }
  ```
- `MekanismUtils.java`: cálculo padrão de 0 a 15 baseado na fração de preenchimento do tanque de água (`floor(15.0 * amount / capacity)` quando `amount > 0`).

### Situação Atual no Port
A API do Horizon (`port/api/horizon.wit`) expõe apenas a leitura de sinal recebido pelo bloco (`world.get-redstone: func(dimension: string, x: s32, y: s32, z: s32) -> u8`), mas não oferece mecanismo para que um `BlockEntity` emita ou forneça um nível analógico de sinal de comparador vanilla (`getComparatorInputOverride`).

### Proposta de Assinatura WIT
```wit
interface world {
    /// Define o nível analógico de sinal de comparador (0 a 15) emitido pelo bloco nas coordenadas dadas.
    set-comparator-output: func(dimension: string, x: s32, y: s32, z: s32, level: u8);
}
```
Alternativamente, via campo opcional no `BlockEntityUpdate`:
```wit
record block-entity-update {
    dimension: string,
    pos: block-pos,
    ...
    comparator-level: option<u8>,
}
```

---

## 2. Acesso à Interface Gráfica (GUI) por Blocos de Carcaça Formados (`FORK-15`)

### Referência no Java
- `TileEntityMultiblock.java:218-222`:
  ```java
  @Override
  public ItemInteractionResult onActivate(Player player, InteractionHand hand, ItemStack stack) {
      if (player.isShiftKeyDown() || !getMultiblock().isFormed()) {
          return ItemInteractionResult.PASS_TO_DEFAULT_BLOCK_INTERACTION;
      }
      InteractionResult result = openGui(player);
      ...
  }
  ```
No Java, qualquer bloco de carcaça (`TileEntityBoilerCasing`), válvula ou vidro estrutural pertencente a um multibloco formado abre a GUI do multibloco ao clique com o botão direito do jogador sem agachar.

### Situação Atual no Port
No Horizon, apenas blocos que declaram `container.menu = true` na especificação de registro (`registration`) recebem eventos de clique de menu e abrem tela. O bloco `boiler_casing` e o `structural_glass` são registrados como blocos simples sem contêiner (`menu = false`) para evitar overhead de entidades de bloco no servidor. Apenas `boiler_valve` possui contêiner e menu ativos.

### Proposta de Assinatura WIT
```wit
interface server-mods {
    /// Evento disparado quando um jogador clica com o botão direito em um bloco de multibloco
    /// sem contêiner próprio, permitindo redirecionar e abrir o menu do mestre/controlador.
    open-multiblock-menu: func(player: player, dimension: string, pos: block-pos, master-pos: block-pos);
}
```

---

## 3. Uso Condicional de Bloco e Queda de Ação / Fallthrough (Sintoma S1 / `FORK-29`)

### Referência no Java
- `TileEntityMultiblock.java:218-229`:
  ```java
  if (player.isShiftKeyDown() || !getMultiblock().isFormed()) {
      return ItemInteractionResult.PASS_TO_DEFAULT_BLOCK_INTERACTION;
  }
  ```
- Enquanto a estrutura **não estiver formada**, clicar com o botão direito num bloco do multibloco (válvula ou carcaça) segurando outro bloco deve **colocar** o bloco da mão normalmente no mundo contra a face clicada, sem abrir menus nem consumir o clique à toa.
- O mesmo vale se o jogador estiver agachado (`isShiftKeyDown()`): deve colocar o bloco da mão em vez de abrir a GUI da válvula.

### Situação Atual no Port
O Horizon consome a interação de blocos com menu registrado (`boiler_valve`), tentando sempre processar abertura de contêiner ou interceptando o clique, forçando o jogador a usar `Shift` para colocar blocos sobre carcaça ou impedindo a colocação intuitiva durante a construção do multibloco.

### Proposta de Assinatura WIT (`FORK-29`)
```wit
interface server-mods {
    enum interaction-result {
        success,
        consume,
        pass-to-default,
    }

    /// Permite ao mod decidir dinamicamente se consome a interação ou se passa adiante
    /// para o comportamento padrão (ex.: colocar o item da mão).
    on-block-interact: func(player: player, dimension: string, pos: block-pos, hand: interaction-hand, sneaking: bool) -> interaction-result;
}
```

---

## 4. Partículas e Efeitos de Formação do Multibloco (Sintoma S2 / `FORK-30`)

### Referência no Java
- `TileEntityMultiblock.java:297-312`:
  ```java
  public void doMultiblockSparkle() {
      if (hasWorld() && isRemote()) {
          ...
          if (MekanismConfig.client.enableMultiblockFormationParticles.get()) {
              new SparkleAnimation(this, renderLocation, length - 1, width - 1, height - 1).run();
          } else {
              player.displayClientMessage(MekanismLang.MULTIBLOCK_FORMED_CHAT.translateColored(EnumColor.INDIGO), true);
          }
      }
  }
  ```
- Ao completar a montagem e validar a estrutura, faíscas mágicas percorrem todas as arestas externas da caixa cúbica delimitadora para todos os jogadores a até 40 blocos de distância, ou uma mensagem no action bar é exibida.

### Situação Atual no Port
No WASM atual, o mod detecta a transição de formação (`!was_formed && is_formed`), mas a API do Horizon não possui função para disparar partículas ou som customizado de formação. Como fallback provisório, o mod pode apenas enviar mensagens de texto se a API de chat permitir, mas não pode reproduzir o efeito visual característico.

### Proposta de Assinatura WIT (`FORK-30`)
```wit
interface world {
    record particle-options {
        name: string,
        pos: [f64; 3],
        delta: [f32; 3],
        speed: f32,
        count: u32,
    }

    /// Emite partículas visíveis no mundo para jogadores em um determinado raio.
    spawn-particles: func(dimension: string, particles: particle-options, max-distance: f32);
}
```

---

## 5. Renderização Volumétrica Tridimensional de Fluidos Internos (Sintoma S4 / `FORK-19` / `FORK-31`)

### Referência no Java
- `RenderThermoelectricBoiler.java:30-50` e `client/render/tileentity/MultiblockTileEntityRenderer.java`:
  - Renderiza duas caixas de fluido distintas através de blocos de `structural_glass`:
    1. **Água líquida**: do fundo interno até a altura fracionária do volume de água preenchido (abaixo do plano de dispersores).
    2. **Vapor superaquecido**: acima do plano de dispersores até a altura proporcional ao vapor contido.
  - A renderização interpola níveis suavemente, usa texturas e cores correspondentes a cada fluido/químico e ajusta luminosidade com base nos `superheating_element` acesos.

### Situação Atual no Port
O Horizon não oferece suporte a rendering customizado procedural de fluidos em tempo de execução para multiblocos. O jogador só consegue monitorar o nível de água e vapor através das barras e números na interface da válvula.

### Proposta de Assinatura WIT (`FORK-19` detalhado / `FORK-31`)
```wit
interface render {
    record multiblock-fluid-box {
        structure-id: string,
        min: [f32; 3],
        max: [f32; 3],
        fluid-id: string,
        fraction: f32,
        light: u8,
    }

    /// Registra ou atualiza a renderização de volume de fluido para uma estrutura de multibloco.
    update-multiblock-fluid: func(dimension: string, box: multiblock-fluid-box);

    /// Remove a malha de renderização ao desmontar ou invalidar a estrutura.
    clear-multiblock-fluid: func(dimension: string, structure-id: string);
}
```

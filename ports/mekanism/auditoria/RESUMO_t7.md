# Resumo da Auditoria: Transmissores e Armazenamento (Tarefa T20261109-184524-2ky)

## 1. Escopo Auditado
Foram auditados todos os 7 arquivos do escopo contra o código-fonte de referência Java (**Mekanism 1.21.1-10.7.19.85**):
- `port/mod/src/transmitter.rs`
- `port/mod/src/induction.rs`
- `port/mod/src/bin.rs`
- `port/mod/src/cube.rs`
- `port/mod/src/continuous.rs`
- `port/mod/src/sides.rs`
- `port/mod/src/bounding.rs`

Componentes e números específicos verificados:
- **Universal Cables**: capacidade por tier (Basic, Advanced, Elite, Ultimate) em Joules → FE.
- **Pressurized Tubes**: capacidade interna por tier e taxas de bombeamento/pull em mB.
- **Mechanical Pipes**: capacidade interna por tier e taxas de bombeamento/pull em mB.
- **Logistical Transporters**: taxas de pull (throughput), velocidades (speed), cores do configurador e variantes (Restrictive, Diversion).
- **Thermodynamic Conductors**: parâmetros de condução térmica, capacidade calorífica e isolamento por tier.
- **Bins**: capacidades por tier (Basic, Advanced, Elite, Ultimate, Creative), cadência de auto-ejeção (10 ticks) e comportamento de slots.
- **Energy Cubes**: capacidades e taxas de saída por tier em Joules → FE; slot de descarga e conversão energética de redstone.
- **Induction Matrix**: capacidades das células por tier, taxas de transferência dos provedores por tier, validação estrutural (3 a 18 por lado), casca com Casing/Port/Structural Glass e modo da porta.
- **Máquinas Contínuas**: tanques e capacidades do Separador Eletrolítico, Lavadora Química, Infusor Químico, Condensentrador Rotativo, Misturador de Pigmentos, Centrífuga Isotópica e Ativador de Nêutrons Solares.
- **Bounding Blocks**: offsets de blocos subordinados do Gerador Solar Avançado (10 blocos), Gerador Eólico (4 blocos), Misturador de Pigmentos (1 bloco), Centrífuga Isotópica (1 bloco), Ativador Solar (1 bloco) e Minerador Digital (17 blocos).

---

## 2. Divergências Encontradas e Correções Aplicadas

1. **Conversão de Redstone no Energy Cube (`port/mod/src/cube.rs`)**:
   - **Java**: `redstone.json` fornece 10.000 J; `redstone_block.json` fornece 90.000 J.
   - **Port**: Como todo o mod opera internamente em Forge Energy (`FE = J * 2 / 5`), os valores literais 10.000 e 90.000 estavam concedendo 2,5 vezes mais energia (10.000 FE = 25.000 J; 90.000 FE = 225.000 J).
   - **Correção**: Atualizado `conversion()` em `port/mod/src/cube.rs` para retornar `Some(4_000)` FE para redstone e `Some(36_000)` FE para bloco de redstone.
   - **Teste atualizado**: Teste `redstone_gives_the_energy_the_recipes_say` em `port/mod/src/cube.rs:123` ajustado para conferir 4.000 e 36.000 FE.

2. **Paleta de Cores do Logistical Transporter (`port/mod/src/transmitter.rs`)**:
   - **Java**: `EnumColor` possui 18 cores oficiais (`EnumColor.java:28-46`). O ciclo com o configurador (`TransporterUtils.increment`) alterna entre `null` ("None") e as 18 cores em ordem (total de 19 estados).
   - **Port**: O port usava uma lista legada de 16 cores estilo ChatFormatting ("None" + 16 = 17 estados).
   - **Correção**: Atualizado `COLORS` em `port/mod/src/transmitter.rs` para 19 entradas contendo "None" e as 18 cores de `EnumColor` ("Black", "Dark Blue", "Dark Green", "Dark Aqua", "Dark Red", "Purple", "Orange", "Light Gray", "Gray", "Light Blue", "Lime", "Aqua", "Red", "Magenta", "Yellow", "White", "Brown", "Pink"), ajustando a máscara de decodificação para `data[7].min(18)` e o avanço para `(state.color + 1) % 19`.

---

## 3. Limitações Registradas (Horizon Mod API / WASM)

1. **Capacidade Interna de Tubos e Canos**:
   - O Java define capacidade por bloco de rede (Tubos: 4.000, 16.000, 256.000, 1.024.000 mB; Canos: 2.000, 8.000, 32.000, 128.000 mB) separada da taxa de extração (`basePull`: 750, 2.000, 64.000, 256.000 mB/t em tubos; 250, 1.000, 8.000, 32.000 mB/t em canos).
   - A API do Horizon possui apenas o campo `throughput` no `TransmitterSpec` (configurado com a taxa de extração). A capacidade de buffer por bloco não é exposta na API.
2. **Velocidade de Transporte de Itens**:
   - O Java possui velocidade de deslocamento dos itens por tier de transportador (`baseSpeed`: 5, 10, 20, 50 ticks). O Horizon não modela velocidade de trânsito em dutos de itens.
3. **Variantes Restritiva e de Desvio de Transporters**:
   - Não foram implementadas no port (Restritivo com custo 1.000 e Desvio com modos de sinal redstone).
4. **Condutores Termodinâmicos (Calor)**:
   - Não implementados no port (o Horizon não possui motor de calor genérico).
5. **Creative Bin**:
   - No Java a capacidade nominal é `Integer.MAX_VALUE` (2.147.483.647 itens) e o slot de inventário desativa o decréscimo (`!isCreative`). No port o `step` subtrai do contador sem checagem de modo criativo, portanto a capacidade foi mantida em `u64::MAX / 2` para não esgotar.
6. **Creative Energy Cube**:
   - No Java a capacidade e a taxa de saída nominais são `Long.MAX_VALUE`. No port, são representadas como 1 trilhão de FE e 1 bilhão de FE/t para evitar overflow na aritmética de 64 bits e compatibilidade com a UI em 32 bits.

---

## 4. Medições e Arquivos Gerados

- **Arquivos gerados em `.saida/T20261109-184524-2ky/`**:
  - `AUDITORIA_transmissores_armazenamento.md` (tabela completa de todos os itens do escopo com arquivo e linha de origem no Java e no Port)
  - `DIFERENCAS_bloco_20.md` (bloco pronto no formato da seção 19 de `port/DIFERENCAS.md` numerado como seção 20)
  - `RESUMO.md` (este documento de entrega)
- **Arquivos modificados em `port/mod/src/`**:
  - `port/mod/src/cube.rs`
  - `port/mod/src/transmitter.rs`
- **Contagem de testes nos arquivos do escopo**:
  - `port/mod/src/cube.rs`: 6 testes
  - `port/mod/src/transmitter.rs`: 1 teste
  - `port/mod/src/induction.rs`: 3 testes
  - `port/mod/src/bin.rs`: 4 testes
  - `port/mod/src/continuous.rs`: 6 testes
  - `port/mod/src/sides.rs`: 7 testes
  - `port/mod/src/bounding.rs`: 2 testes
  - **Total de testes nos 7 arquivos**: 29 testes.
- **Execução de testes na VM**: não medido (a VM não possui toolchain Rust/Cargo instalada, conforme diretrizes da tarefa).
- **Prova de mutação**: não medido (pendente do juiz / orquestrador).

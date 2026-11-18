# Resumo da Auditoria: Multiblocos (Dynamic Tank, Boiler, Thermal Evaporation)

**ID da Tarefa**: T20261109-190810-jtm  
**Data**: 2026-11-09  
**Referência de Verdade**: Mekanism Java 1.21.1-10.7.19.85 (`java/`)  
**Arquivos Auditados e Modificados**:
- `port/mod/src/dynamic.rs` (336 linhas)
- `port/mod/src/boiler.rs` (554 linhas)
- `port/mod/src/evaporation.rs` (452 linhas)

---

## 1. Escopo e Objetivo
Auditar contra a base Java Mekanism 1.21.1-10.7.19.85 as constantes, algoritmos e configurações dos três multiblocos:
1. **Dynamic Tank** (`dynamic.rs` vs `TankMultiblockData.java`, `TankValidator.java`, `GeneralConfig.java`)
2. **Thermoelectric Boiler** (`boiler.rs` vs `BoilerMultiblockData.java`, `BoilerValidator.java`, `GeneralConfig.java`, `HeatUtils.java`, `HeatAPI.java`, `HeatedCoolant.java`)
3. **Thermal Evaporation Plant** (`evaporation.rs` vs `EvaporationMultiblockData.java`, `EvaporationValidator.java`, `GeneralConfig.java`)

---

## 2. Principais Divergências Encontradas e Corrigidas

1. **Dynamic Tank: Capacidade de Fluido vs Químico (`dynamic.rs`)**
   - *No Java*: `GeneralConfig.java:169, 171` e `TankMultiblockData.java:134-135`.
     - Fluido: `350 * 1000 = 350.000` mB/bloco de volume interno.
     - Químico: `16_000 * 1000 = 16.000.000` mB/bloco de volume interno.
   - *No port anterior*: `PER_BLOCK = 16_000 * 1000;` era aplicado indistintamente para fluidos e químicos (fluidos com capacidade 45x maior que a real).
   - *Correção*: Criadas constantes `FLUID_PER_BLOCK = 350_000` e `CHEMICAL_PER_BLOCK = 16_000_000`. As capacidades agora são computadas separadamente (`fluid_cap` e `chem_cap`) para a lógica de válvulas, merge, slots de baldes, filtros e interface gráfica.

2. **Thermoelectric Boiler: Coeficiente de Dissipação Térmica com Ar (`boiler.rs`)**
   - *No Java*: `HeatAPI.java:24` define `AIR_INVERSE_COEFFICIENT = 10_000.0`.
   - *No port anterior*: `AIR: f64 = 10.0;` (diferença de 1000x, resfriando a caldeira de forma drasticamente acelerada).
   - *Correção*: Corrigido `AIR: f64 = 10_000.0;`. A impedância térmica com o ambiente agora totaliza `10_000.0 + 100_000.0 + 1.0 = 110_001.0`.

3. **Thermoelectric Boiler: Altura da Câmara de Vapor (`boiler.rs`)**
   - *No Java*: `BoilerValidator.java:128` calcula `(renderLocation.getY() + height - 2) - disperserY = max[1] - plane`.
   - *No port anterior*: `(max[1] - 1 - plane).max(0);`, subtraindo uma camada a mais. Em caldeiras de 3x3x3 com dispersor no plano y=1, resultava em altura 0 e volume de vapor 0, impossibilitando reter ou produzir vapor.
   - *Correção*: Corrigido para `(max[1] - plane).max(0);`.

4. **Thermal Evaporation Plant: Capacidade do Tanque de Entrada (`evaporation.rs`)**
   - *No Java*: `EvaporationMultiblockData.java:222` e `GeneralConfig.java:311`: `inputTankCapacity = (volume / 4) * evaporationFluidPerTank`. Como a torre tem base 4x4, `volume = 16 * height`, logo `volume / 4 = 4 * height`. Com `evaporationFluidPerTank = 64.000`, a capacidade por nível é `4 * 64.000 = 256.000` mB.
   - *No port anterior*: `(height - 1) * 64_000;` (subtraía 1 da altura e não considerava os 4 blocos da área 2x2 interna).
   - *Correção*: Declaradas `FLUID_PER_TANK = 64_000` e `PER_LEVEL = 4 * FLUID_PER_TANK` (256.000 mB/nível), computando `in_cap = u64::from(found.height) * PER_LEVEL`.

---

## 3. Testes Editados e Adicionados

- Compilação na VM: não disponível (sem toolchain Horizon/Rust configurada).
- Testes unitários presentes nos arquivos do escopo:
  - `port/mod/src/dynamic.rs`: 5 testes
  - `port/mod/src/boiler.rs`: 6 testes
  - `port/mod/src/evaporation.rs`: 7 testes
  - **Total de testes no escopo**: 18 testes
- **Testes com linhas editadas / adicionados**:
  - `port/mod/src/dynamic.rs`:
    - Adicionado: `tank_capacity_fluid_vs_chemical`
  - `port/mod/src/boiler.rs`:
    - Editado: `a_box_with_dispersers_and_an_element_is_a_boiler`
    - Editado: `the_steam_room_is_the_height_above_the_dispersers`
    - Editado: `the_air_cools_it_slowly`
  - `port/mod/src/evaporation.rs`:
    - Adicionado: `evaporation_input_capacity`

---

## 4. Prova de Mutação
- A tarefa não trouxe lista pré-definida de mutações. Conforme as regras da delegação, a prova de mutação está pendente do juiz.

---

## 5. Bloco Pronto para DIFERENCAS.md (Seção 20)

```markdown
## 20. Multiblocos: Dynamic Tank, Thermoelectric Boiler e Thermal Evaporation (mod 0.1.86)

| # | Diferença | Estado |
|---|---|---|
| 20.1 | Dynamic Tank: interface gráfica acessível exclusivamente nas válvulas (no original, blocos de carcaça também abrem a GUI quando a estrutura está formada); sem aba de alternância de modo de edição de baldes/contêineres (`ContainerEditMode` Both/Fill/Empty) | ABERTA |
| 20.2 | Dynamic Tank e Boiler: aceitam `mekanism:structural_glass` em qualquer face ou aresta da carcaça no port (no Java original, o vidro estrutural é permitido apenas nas faces/paredes, exigindo bloco de casing nas 12 arestas da moldura) | SUPOSIÇÃO |
| 20.3 | Thermoelectric Boiler: volume de água conta os blocos de ar contíguos sob o plano de dispersores (no Java original, a rotina `FormationProtocol.explore` inclui também os blocos de moldura do piso e paredes submersas) | SUPOSIÇÃO |
| 20.4 | Thermoelectric Boiler: sem renderização visual 3D dos fluidos internos no mundo quando usado vidro estrutural; fluidos exibidos na tela da válvula por índices de substância sem animação | LIMITE API |
| 20.5 | Thermal Evaporation Plant: aquecimento externo por condutores térmicos (`Thermodynamic Conductor`) ou aquecedor resistivo (`Resistive Heater`) acoplado às válvulas não suportado (rede de calor ainda não existe no port; aquecimento unicamente solar no topo) | LIMITE |
| 20.6 | Thermal Evaporation Plant: sem suporte a sinal analógico de redstone nas válvulas para automação por nível; sem efeitos sonoros de funcionamento | ABERTA |
```

---

## 6. Arquivos Gerados
1. `.saida/T20261109-190810-jtm/AUDITORIA_multiblocos_tanques_boiler_evap.md` (tabela completa com todas as constantes, fórmulas e correspondências de arquivo:linha).
2. `.saida/T20261109-190810-jtm/DIFERENCAS_bloco_20.md` (bloco pronto no padrão da seção 19 para incorporação).
3. `.saida/T20261109-190810-jtm/RESUMO.md` (este documento de síntese).

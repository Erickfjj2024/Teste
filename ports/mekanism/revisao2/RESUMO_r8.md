# Resumo da Tarefa T20261109-195424-mh3

Revisão cruzada (rodada 2) de RADIAÇÃO, LASER, SPS, ELECTRIC PUMP, FLUIDIC PLENISHER, REDSTONE/ALARM e CONSTANTES GERAIS (`radiation.rs`, `laser.rs`, `sps.rs`, `pump.rs`, `redstone.rs`, `alarm.rs`, `lib.rs`, `config.rs`), reclassificação de LIMITE para FORK com mapeamento de API, e especificação técnica completa (Parte B) das 18 substâncias químicas como fluidos/baldes e do Radioactive Waste Barrel.

Referência da verdade: Mekanism Java 1.21.1-10.7.19.85 (`v1.21.1-10.7.19.85`) em `java/`.

---

## 1. Arquivos Auditados e Entregáveis

### Arquivos auditados no Port (`port/mod/src/`):
- `port/mod/src/radiation.rs`
- `port/mod/src/laser.rs`
- `port/mod/src/sps.rs`
- `port/mod/src/pump.rs`
- `port/mod/src/redstone.rs`
- `port/mod/src/alarm.rs`
- `port/mod/src/lib.rs`
- `port/mod/src/config.rs`

### Arquivos gerados em `.saida/T20261109-195424-mh3/`:
- `REVISAO_radiacao_laser_sps_pump_redstone_alarm_lib.md`: relatório e tabela comparativa de revisão cruzada com 75 itens auditados com arquivo e linha exatos no Java e no Rust, reclassificação dos limites para FORK e apontamentos inéditos não vistos na rodada 1.
- `ESPEC_quimicos_barril.md`: especificação técnica exaustiva (Parte B) das 18 substâncias químicas líquidas e seus baldes (`brine`, `chlorine`, `ethene`, `heavy_water`, `hydrofluoric_acid`, `hydrogen`, `hydrogen_chloride`, `lithium`, `nutritional_paste`, `oxygen`, `sodium`, `steam`, `sulfur_dioxide`, `sulfur_trioxide`, `sulfuric_acid`, `superheated_sodium`, `uranium_hexafluoride`, `uranium_oxide`) e do `radioactive_waste_barrel`.
- `BLOCO_DIFERENCAS.md`: bloco formatado pronto para inclusão na seção 21 de `port/DIFERENCAS.md` (itens 21.8.1 a 21.8.10), já sincronizado no próprio arquivo `port/DIFERENCAS.md`.
- `RESUMO.md`: este documento de encerramento da execução.

---

## 2. Estatísticas da Auditoria

- Total de itens auditados na tabela de revisão: 75
  - **CONFIRMADA**: 58
  - **CORRIGIDO / OBS**: 1
  - **FORK**: 14 (distribuídos em FORK-5, FORK-7, FORK-8, FORK-10, FORK-11, FORK-13, FORK-19, FORK-21, FORK-22 e FORK API)
  - **FALTA**: 2 (`radioactive_waste_barrel` e `laser_tractor_beam`)
- Testes unitários executados na VM: não medido (a VM não possui toolchain Rust/Horizon instalada para compilação local, conforme restrição declarada da tarefa).
- Prova de mutação: a prova de mutação está pendente do juiz (a tarefa não trouxe lista prévia de mutações).

---

## 3. Correções e Testes Aplicados no Código (`port/mod/src/`)

Foram aplicados testes unitários validando as constantes do Java contra o código Rust nos módulos do escopo:

1. **`port/mod/src/laser.rs`**:
   - Adicionado teste unitário `laser_constants_match_java_config` validando `RANGE == 64`, `FIRE == 4_000`, `LASER_CAPACITY == 800_000` e `AMPLIFIER_CAPACITY == 2_000_000_000`.
   - Nome do teste que contém a linha editada: `laser_constants_match_java_config`.
2. **`port/mod/src/sps.rs`**:
   - Adicionado teste unitário `sps_constants_match_java_config` validando `INPUT_PER_ANTIMATTER == 1_000`, `OUTPUT_CAPACITY == 1_000`, `ENERGY_PER_INPUT == 400_000`, `PORT_ENERGY == 400_000_000` e `input_capacity() == 2_000`.
   - Nome do teste que contém a linha editada: `sps_constants_match_java_config`.
3. **`port/mod/src/pump.rs`**:
   - Expandido o teste `max_nodes_matches_java_config` validando `MAX_NODES == 4000`, `USAGE == 40`, `BASE_TICKS == 19`, `TANK == 10_000` e `RANGE == 80`.
   - Nome do teste que contém a linha editada: `max_nodes_matches_java_config`.
4. **`port/mod/src/radiation.rs`**:
   - Adicionado teste unitário `radiation_constants_match_java_config` validando `BASELINE == 0.000_000_100`, `MIN_MAGNITUDE == 0.000_010`, `DECAY == 0.9995`, `MIN_SEVERITY == 0.1` e a radioatividade por mB de `nuclear_waste` (0.01), `spent_nuclear_waste` (0.01), `plutonium` (0.02) e `polonium` (0.05).
   - Nome do teste que contém a linha editada: `radiation_constants_match_java_config`.

---

## 4. Diferenças e Limites Registrados (Seção 21.8 de `port/DIFERENCAS.md`)

- **21.8.1 [FORK (FORK-13)]**: Radiação - Decaimento, Raio e Áudio: Enum `RadiationScale` (NONE a EXTREME) e sons dinâmicos de cliques do contador Geiger (`geiger_slow`, `geiger_medium`, `geiger_elevated`, `geiger_fast`) não existem no WASM; severidade é contínua 0.0..1.0 logarítmica. Raio no Java é restrito a 5 chunks (80 blocos via `MAX_BLOCK_RANGE`), no port soma todas as fontes ativas da dimensão por distância quadrática.
- **21.8.2 [FORK (FORK-11, FORK-13, FORK API)]**: Radiação - Dano a Entidades e Descarte na Quebra: Dano de radiação no Java aplica tipo customizado `mekanism:radiation` e exaustão alimentar (`addExhaustion`), enquanto port aplica `minecraft:magic` sem fome. Quebra de máquinas, tanques ou tubos com químicos radioativos no Java despeja todo o resíduo no ambiente como radiação (`dumpRadiation`) e limpa o resíduo do item dropado; no port não há descarte ambiental. Desmontagem com configurador no Java é bloqueada se o bloco contiver resíduos (`getRadiationScale() > 0`), no port se o ambiente estiver irradiado.
- **21.8.3 [FORK API]**: Radiação - Dispositivos de Medição: Geiger Counter e Dosimeter no Java exibem textos formatados com 3 casas decimais (`UnitDisplayUtils`), sem espaço entre prefixo e unidade (`nSv/h`, `µSv/h`), tempo estimado de decaimento (`gui.mekanism.radiation_decay_time`) e o Dosimeter permite interação com outras entidades (`interactLivingEntity` / `gui.mekanism.radiation_exposure.entity`). No port mensagens usam formato simplificado e não medem outras entidades.
- **21.8.4 [FALTA]**: Radiação - Radioactive Waste Barrel: Bloco de armazenamento seguro de resíduos radioativos (capacidade 512 000 mB, decaimento 1 mB/20t, aceita apenas químicos radioativos, conecta apenas topo/base, auto-escoamento descendente quando ativo, ejeção ascendente por overflow, emissão catastrófica de radiação ao ser quebrado com resíduos) não implementado no port.
- **21.8.5 [FORK (FORK-7, FORK-8, FORK-21)]**: Laser - Dano, Escavação e Redstone: Dano/queima a entidades cruzadas pelo feixe (`laserEnergyPerDamage` = 2 500 J / 1 000 FE por 1 HP), escavação de blocos por dureza (`laserEnergyPerHardness` = 100 000 J / 40 000 FE por dureza), e modos de saída redstone do Laser Amplifier (OFF, ENTITY_DETECTION, ENERGY_CONTENTS) não implementados no port.
- **21.8.6 [FALTA]**: Laser - Laser Tractor Beam: Bloco receptor de laser com buffer de 5 000 000 000 J (2 000 000 000 FE) e inventário de saída de 27 slots para coleta automática de blocos minerados pelo laser e absorção de entidades de item floating não implementado no port.
- **21.8.7 [FORK (FORK-7, FORK-10)]**: SPS - Death Zone e Efeitos: Dano elétrico contínuo na câmara interna de aceleração 7x7x7 (`lastReceivedEnergy / 1000F` dano/tick) e disparo de relâmpago visual transformador em entidades de experimentos válidos não implementados no port.
- **21.8.8 [FORK (FORK-22)]**: Electric Pump - Heavy Water e Upgrades: Extração de Heavy Water (10 mB por bloco de água) mediante instalação de Filter Upgrade não implementada no port; no Java a bomba suporta Speed, Energy e Filter, enquanto Plenisher suporta Speed e Energy (no port, `Kind::Pump` cai no default com Muffling).
- **21.8.9 [FORK (FORK-19)]**: Industrial Alarm - Efeitos Audiovisuais e Bloco: Som em loop da sirene (`MekanismSounds.INDUSTRIAL_ALARM`) e feixe de holofote vermelho giratório renderizado em shader (`RenderIndustrialAlarm`) não existem no WASM; estado de bloco utiliza propriedade `lit` (herança vanilla) em vez de `active=true/false` com nível de luz ambiente 15.
- **21.8.10 [FALTA]**: Químicos Líquidos e Baldes: 18 fluidos (`LiquidBlock`) e respectivos baldes de químicos listados como `FALTA` no inventário, incluindo receitas no Rotary Condensentrator e consumo de balde de pasta nutricional como bebida alimentar.

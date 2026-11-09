# Resumo da Tarefa T20261109-183511-w7x

Auditoria de RADIAÇÃO, LASER, SPS, ELECTRIC PUMP, FLUIDIC PLENISHER, REDSTONE/ALARM e CONSTANTES GERAIS (`lib.rs`, `config.rs`).
Referência da verdade: Mekanism Java 1.21.1-10.7.19.85 (`v1.21.1-10.7.19.85`) em `java/`.

---

## 1. Arquivos Auditados e Entregáveis

### Arquivos auditados no Port (`port/mod/src/`):
- `radiation.rs`
- `laser.rs`
- `sps.rs`
- `pump.rs`
- `redstone.rs`
- `alarm.rs`
- `lib.rs`
- `config.rs`

### Arquivos de saída gerados em `.saida/T20261109-183511-w7x/`:
- `AUDITORIA_radiation_laser_sps_pump_redstone_alarm_lib.md`: relatório e tabela comparativa detalhada com 70 itens auditados com arquivo e linha exatos tanto no Java quanto no Rust.
- `DIFERENCAS_BLOCO_SECAO_20.md`: bloco formatado pronto para inclusão na seção 20 de `port/DIFERENCAS.md` (itens 20.1 a 20.6).
- `RESUMO.md`: este documento de encerramento da execução.

---

## 2. Estatísticas da Auditoria

- Total de itens/constantes auditados na tabela: 70
  - **OK**: 59
  - **CORRIGIDO**: 1
  - **LIMITE**: 8
  - **FALTA**: 2
- Testes unitários executados na VM: não medido (a VM não possui toolchain Rust/Horizon instalada para compilação local, conforme restrição declarada da tarefa).
- Prova de mutação: a prova de mutação está pendente do juiz (a tarefa não trouxe lista prévia de mutações).

---

## 3. Correções Aplicadas no Código

1. **`port/mod/src/pump.rs`**:
   - `MAX_NODES`: corrigido de `4096` para `4000` para bater exatamente com `MekanismConfig.general.maxPlenisherNodes` (`java/main/java/mekanism/common/config/GeneralConfig.java:255`).
   - Adicionado teste unitário `max_nodes_matches_java_config` validando que `MAX_NODES == 4000`.
   - Nome do teste adicionado: `max_nodes_matches_java_config`.
2. **`port/mod/src/laser.rs`**:
   - Atualizado comentário no cabeçalho do módulo que mencionava "20 FE a tick" (resquício da branch 26.3) para "4,000 FE a tick", consistente com `fe(10_000)` do Mekanism 1.21.1.

---

## 4. Diferenças e Limites Registrados (Seção 20)

- **20.1 [CORRIGIDO]**: `Fluidic Plenisher`: `maxPlenisherNodes` corrigido de 4096 para 4000.
- **20.2 [LIMITE / FALTA]**: `Radiação`: Enum `RadiationScale` (NONE a EXTREME) e sons de contador Geiger não existem no WASM; severidade contínua 0.0..1.0 logarítmica via `RadiationScale.getScaledDoseSeverity`. Raio no Java é restrito a 5 chunks (80 blocos via `MAX_BLOCK_RANGE`), no port soma todas as fontes ativas da dimensão por distância quadrática. `Radioactive Waste Barrel` (512k mB, decaimento 1 mB/20t) ainda não implementado.
- **20.3 [LIMITE / FALTA]**: `Laser`: Dano a entidades (`laserEnergyPerDamage` = 2 500 J / 1 000 FE por 1 HP), escavação de blocos (`laserEnergyPerHardness` = 100 000 J / 40 000 FE por dureza) e o Laser Tractor Beam (capacidade 5 000 000 000 J / 2 000 000 000 FE) não existem no port; modos de saída redstone do Laser Amplifier (OFF, ENTITY_DETECTION, ENERGY_CONTENTS) também não implementados.
- **20.4 [LIMITE]**: `SPS`: Dano elétrico da death zone (`lastReceivedEnergy / 1000F` dano/tick) e relâmpago visual em entidades de experimentos válidos não implementados no port.
- **20.5 [LIMITE]**: `Electric Pump`: Extração de Heavy Water (10 mB por bloco de água) via Filter Upgrade não implementada.
- **20.6 [LIMITE]**: `Industrial Alarm`: Efeitos audiovisuais (som de sirene e feixe giratório de luz vermelha) não existem no WASM; bloco apenas atualiza o estado de bloco `lit=true`/`lit=false` quando energizado por redstone.

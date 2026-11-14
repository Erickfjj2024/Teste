## 20. Auditoria de Radiação, Laser, SPS, Pump/Plenisher, Redstone e Constantes (mod 0.1.86, **sem teste no jogo**)

Conferência de todos os valores numéricos e comportamentos contra Mekanism 1.21.1-10.7.19.85 (`v1.21.1-10.7.19.85`).

| # | Diferença | Estado |
|---|---|---|
| 20.1 | Fluidic Plenisher: `maxPlenisherNodes` no código do port era 4096; corrigido para 4000 conforme padrão do Java (`GeneralConfig.maxPlenisherNodes`) | CORRIGIDO |
| 20.2 | Radiação: enum `RadiationScale` (NONE a EXTREME) e sons de contador Geiger não existem no WASM; severidade contínua 0.0..1.0 logarítmica via `RadiationScale.getScaledDoseSeverity`. Raio no Java é restrito a 5 chunks (80 blocos), no port soma todas as fontes ativas da dimensão por distância quadrática. `Radioactive Waste Barrel` (512k mB, decaimento 1 mB/20t) ainda não implementado | LIMITE / FALTA |
| 20.3 | Laser: dano a entidades (`laserEnergyPerDamage` = 2 500 J / 1 000 FE por 1 HP), escavação de blocos (`laserEnergyPerHardness` = 100 000 J / 40 000 FE por dureza) e o Laser Tractor Beam (capacidade 5 000 000 000 J / 2 000 000 000 FE) não existem no port; modos de saída redstone do Laser Amplifier (OFF, ENTITY_DETECTION, ENERGY_CONTENTS) também não implementados | LIMITE / FALTA |
| 20.4 | SPS: dano elétrico da death zone (`lastReceivedEnergy / 1000F` dano/tick) e relâmpago visual em entidades de experimentos válidos não implementados no port | LIMITE |
| 20.5 | Electric Pump: extração de Heavy Water (10 mB por bloco de água) com Filter Upgrade não implementada | LIMITE |
| 20.6 | Industrial Alarm: efeitos audiovisuais (som de sirene e feixe giratório de luz vermelha) não existem no WASM; bloco apenas atualiza o estado de bloco `lit=true`/`lit=false` quando energizado por redstone | LIMITE |

## 20. Transmissores e Armazenamento (mod 0.1.86, auditoria contra Mekanism 1.21.1-10.7.19.85)

| # | Diferença | Estado |
|---|---|---|
| 20.1 | Descarga de Redstone no Energy Cube (`cube.rs`): valores de conversão de redstone e bloco de redstone corrigidos de 10.000 / 90.000 Joules literais para 4.000 FE / 36.000 FE (`feConversionRate = 2.5`, `FE = J * 2 / 5`), eliminando ganho de 2.5x a mais de energia | CORRIGIDO |
| 20.2 | Cores do Logistical Transporter (`transmitter.rs`): paleta do configurador alinhada com as 18 cores oficiais de `EnumColor` do Mekanism 1.21.1 (Black, Blue, Green, Cyan, Dark Red, Purple, Orange, Light Gray, Gray, Light Blue, Lime, Aqua, Red, Magenta, Yellow, White, Brown, Pink) + None (19 estados, antes 17) | CORRIGIDO |
| 20.3 | Capacidade interna de Tubos e Canos: a API do Horizon no WASM só define `throughput` no `TransmitterSpec` (port usa a taxa de extração/pull do Java: 750/2.000/64.000/256.000 mB/t para tubos e 250/1.000/8.000/32.000 mB/t para canos). A capacidade por bloco (4.000–1.024.000 mB em tubos; 2.000–128.000 mB em canos) não é exposta na API do Horizon | LIMITE |
| 20.4 | Logistical Transporter (velocidade e variantes): a velocidade de trânsito dos itens no tubo (5, 10, 20, 50 ticks) não é configurável no Horizon (apenas pull rate 1, 16, 32, 64). Restrictive Transporter (custo 1.000) e Diversion Transporter (modos redstone por face) não existem no port | LIMITE |
| 20.5 | Thermodynamic Conductor: não implementado no port (o Horizon não possui sistema genérico de transferência de calor nem API de condutores térmicos) | LIMITE |
| 20.6 | Creative Bin (`bin.rs`): no Java a capacidade nominal é `Integer.MAX_VALUE` (2.147.483.647) e a extração suprime a redução do stack (`!isCreative`); no port o `step` subtrai numericamente, então a capacidade é `u64::MAX / 2` para agir como infinito | LIMITE |
| 20.7 | Creative Energy Cube: no Java capacidade e taxa de saída são `Long.MAX_VALUE`; no port são 1 trilhão de FE e saída de 1 bilhão de FE/t para evitar overflow na aritmética e permitir leitura no medidor de tela | LIMITE |

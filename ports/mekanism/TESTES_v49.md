# Testes da atualização: APK v49 + mod 0.1.61 (tudo o que foi construído desde o mod 0.1.56)

Instale o APK `horizon-android-v49.apk` e o `mekanism.hmod` (0.1.61). Responda: **ok**, **parcial** ou **falhou**. O APK novo só é preciso para
o item N1 (transporte de itens); o resto é só o mod.

| # | O que fazer | Esperado |
|---|---|---|
| N1 | Colocar um Logistical Transporter entre uma máquina que produz itens (ex. Crusher com energia) e um baú, e outro entre o baú... (ou entre a máquina e um funil/outra máquina) | Os itens que a máquina põe na saída vão para o baú/outra máquina pelo transportador (1 item a cada 10 ticks no Basic; mais nos outros níveis); cabo de energia e transportador não se misturam |
| N2 | Fabricar os 4 níveis do Logistical Transporter na bancada | As receitas aparecem (circuitos + aço/ligas); 8 por receita |
| N3 | Electric Pump sobre um lago de lava (energia ligada) | A cada ~19 ticks a lava abaixo some e o tanque sobe 1000 mB; a água não some; tanque de 10 baldes; Dump por balde vazio no slot de cima |
| N4 | Fluidic Plenisher sobre um buraco, com tanque de água (balde cheio no slot de cima) | Despeja água em fontes pelos lados e para baixo (um balde a cada 20 ticks), até acabar o tanque |
| N5 | Nutritional Liquifier com pão, maçã etc. e energia | Vira Nutritional Paste (50 mB por ponto de comida) no tanque; sai por tubos/canos de fluido |
| N6 | Antiprotonic Nucleosynthesizer: maçã dourada + antimatéria | (a antimatéria só vem das cadeias nucleares, ainda ausentes: pelo criativo/tanque) faz maçã dourada encantada em 750 ticks |
| N7 | Fábricas de Compressing/Infusing/Injecting/Purifying e Combining/Sawing (testes M1–M7 e L1–L6 dos arquivos anteriores) | Ver `TESTES_mod_0.1.55.md` e `TESTES_mod_0.1.56.md` |

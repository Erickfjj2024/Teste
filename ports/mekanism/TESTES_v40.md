# Testes da atualização: APK v40 + mod 0.1.39

Instale o `horizon-android-v40.apk` e o `mekanism.hmod` (0.1.39). Responda em cada linha: **ok**, **parcial** ou **falhou**, com o que viu.

## Pressurized Reaction Chamber

| # | O que fazer | Esperado |
|---|---|---|
| N1 | Pegar a Pressurized Reaction Chamber na aba criativa e colocar; abrir | Dois medidores à esquerda (fluido e gás), slot de entrada no meio, seta, slot de saída, um medidor pequeno à direita (gás que sai), slot de energia em cima à direita e barra de energia; a tela é um pouco mais alta que as outras |
| N2 | Água (balde no slot de fluido por um tanque de fluido ou cano) + oxigênio (tanque criativo com tubo ou separador de água) + **carvão** no slot de entrada | A seta enche (~5 segundos para 100 de gás, mais para o bloco de carvão); sai **pó de enxofre** no slot de saída e **hidrogênio** no medidor pequeno; o bloco acende |
| N3 | Os tanques de fora | Só entram os líquidos e gases que alguma receita usa (água, oxigênio, hidrogênio, eteno); o de gás de saída entrega para fora pelo lado direito |
| N4 | Janela de lados: abas Itens, Químicos, Fluidos, Energia | 4 abas, abre em Itens |
| N5 | Upgrade de velocidade nela | A receita encurta (o tempo da seta) e gasta mais energia; a aba de upgrades e a de redstone funcionam como nas outras |

## Máquinas de pigmento

| # | O que fazer | Esperado |
|---|---|---|
| P1 | **Pigment Extractor** com um corante (ou lã, tapete, cama) no slot de entrada | Em ~5 s a barra enche e o medidor da direita mostra o pigmento da cor certa (192 por item) |
| P2 | **Painting Machine** com lã branca + pigmento vermelho (de um tanque criativo de químico ou do extrator por tubo) | Em ~10 s sai lã vermelha e gasta 256 do pigmento; o medidor da esquerda tem a cor do pigmento |
| P3 | **Pigment Mixer** com cinza (1) e vermelho (2) nos dois medidores de cima | O medidor do meio vai enchendo de vermelho-escuro (3 por tick); a aba de lados abre em Químicos |
| P4 | Pintar outros itens (concreto, vidro, tapete, cama, vela) | Funciona com a cor do pigmento |
| P5 | Os três: modelo do bloco, tela e abas (lados, redstone, upgrades, energia) | Como as outras máquinas; o modelo vira para você |

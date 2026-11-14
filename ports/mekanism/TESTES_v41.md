# Testes da atualização: APK v41 + mod 0.1.45

Instale o APK `horizon-android-v41.apk` e o `mekanism.hmod` (0.1.45). Responda: **ok**, **parcial** ou **falhou**, com o que viu.
(Os testes V1–V3 da 0.1.44 continuam valendo.)

| # | O que fazer | Esperado |
|---|---|---|
| W1 | Pigment Extractor com o pigmento no tanque (ou Painting Machine/Pigment Mixer com pigmento) trabalhando; olhar a seta de progresso | A seta que enche tem a cor do pigmento (vermelho para o pigmento vermelho etc.) |
| W2 | Tanque de fluido ou máquina com água e outra com lava no medidor | O medidor mostra a textura de água (azulada) e de lava (laranja), não um bloco liso |
| W3 | Chemical Crystallizer com salmoura (brine) no tanque; depois com lítio | Na tela interna aparece "Brine" e "(Salt)" com o sal num slot escuro; com lítio, "Lithium" e "(Lithium Dust)" |
| W4 | Máquina que aceita upgrades (ex. Purification Chamber): instalar 5 tipos de upgrade e abrir a janela de upgrades | A lista mostra 4 linhas e uma barra de rolagem; tocar na metade de baixo da barra mostra a 5ª linha, na de cima volta |
| W5 | Segurar um monte de 10 upgrades de velocidade e, agachado, usar sobre uma máquina | Entram 8 (o máximo), sobram 2 na mão; usando de novo não entra mais. Em tipo que a máquina não aceita, nada acontece (abre/usa normal) |

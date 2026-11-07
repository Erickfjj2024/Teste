# Testes da atualização: mod 0.1.40 (o APK continua o v40)

Instale só o `mekanism.hmod` (0.1.40). Responda em cada linha: **ok**, **parcial** ou **falhou**, com o que viu.

| # | O que fazer | Esperado |
|---|---|---|
| Q1 | Aba criativa do Mekanism: procurar os baldes | 18 baldes (Brine Bucket, Hydrogen Bucket, Sulfuric Acid Bucket...), cada um é um balde de ferro com a parte do líquido pintada com a cor do fluido |
| Q2 | **Balde de salmoura** no slot de cima de um Fluid Tank | O tanque enche (1000 mB), o balde vazio sai no slot de baixo; o líquido do tanque tem a cor da salmoura; balde vazio no slot de cima do tanque cheio devolve o balde cheio (editando o modo "esvaziar" do tanque) |
| Q3 | Balde de **água** na **Chemical Washer** (slot de fluido do suporte da esquerda) e de **ácido sulfúrico** na **Chemical Dissolution Chamber** (tanque de químico por tubo/criativo) | A Washer enche o medidor de fluido; agora dá para alimentar as máquinas químicas sem o separador |
| Q4 | Rotary Condensentrator com balde de **hidrogênio** ou **salmoura** no modo "Decondensentrating" | O fluido entra e vira o gás no medidor da esquerda |
| Q5 | **Sal:** num mundo **novo** (ou numa região nunca visitada), nadar pelo fundo de oceanos e lagos | Aparecem discos brancos de **bloco de sal** de 4 a 7 blocos de largura no fundo (terra ou argila trocada); pegar o bloco de sal com picareta |

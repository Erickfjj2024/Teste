### 21.4 Digital Miner, Quantum Entangloporter, Chargepad, Teleporter e Sorter (mod 0.1.86, revisão cruzada)

| # | Diferença / Difference | Estado / State |
|---|---|---|
| 21.4.1 | Digital Miner: botão RESET no port não parava a execução (`running = true`), permitindo que a máquina reiniciasse a varredura logo após a limpeza da fila; corrigido no port para parar a execução (`running = false`) conforme Java (`TileEntityDigitalMiner.java:828`) | CORRIGIDO |
| 21.4.2 | Digital Miner: fila de mineração consumia blocos via `pop()` (LIFO), resultando em escavação de baixo para cima; corrigido para consumo em fila FIFO (`remove(0)`), garantindo escavação do topo para a base para impedir queda descontrolada de cascalho/areia conforme Java (`TileEntityDigitalMiner.java:470-472`) | CORRIGIDO |
| 21.4.3 | Digital Miner: slots e filtros usam 27 posições de saída + 5 slots de item-filtro no próprio contêiner (limite de 32 slots do Horizon); não há filtros por tag, material ou mod ID, nem substituição de bloco minerado ou renderização visual da área | FORK |
| 21.4.4 | Digital Miner: varredura do terreno é feita de forma síncrona a 512 blocos por tick na thread do servidor com fila máxima de 4096 (o Java usa a thread assíncrona `ThreadMinerSearch`) | FORK |
| 21.4.5 | Quantum Entangloporter: frequências são numéricas de 1 a 99 e públicas (o Java usa nomes em texto e controle de acesso público/privado por UUID); o buffer e transporte de calor não são suportados | FORK |
| 21.4.6 | Teleporter: apenas jogadores são teleportados; não há teletransporte de mobs, veículos (barcos/minecarts) ou cálculo dinâmico de passageiros com multiplicador de energia; não há efeitos sonoros nem partículas de portal | FORK |
| 21.4.7 | Logistical Sorter: usa 2 slots de movimentação (0=intake, 1=outbox) e 5 slots de filtro físico no contêiner; não há suporte a coloração de canais de transporte, modo round-robin, contagem mínima/máxima por filtro nem controle por sinal de redstone | FORK |
| 21.4.8 | Chargepad: bloco e funcionalidade ainda não implementados no port (Java: 1 024 000 J/t = 409 600 FE/t, buffer 2 048 000 J = 819 200 FE, recarga de jogadores e Robits sobre a base) | FALTA |

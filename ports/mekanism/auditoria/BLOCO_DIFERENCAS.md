## 20. Digital Miner, Quantum Entangloporter, Chargepad, Teleporter e Sorter (mod 0.1.86)

| # | Diferença | Estado |
|---|---|---|
| 20.1 | Digital Miner: slots e filtros usam 27 posições de saída + 5 slots de item-filtro no próprio contêiner (limite de 32 slots do Horizon); não há filtros por tag, material ou mod ID, nem substituição de bloco minerado ou renderização visual da área | LIMITE |
| 20.2 | Digital Miner: varredura do terreno é feita de forma síncrona a 512 blocos por tick na thread do servidor com fila máxima de 4096 (o Java usa a thread assíncrona `ThreadMinerSearch`) | LIMITE |
| 20.3 | Quantum Entangloporter: frequências são numéricas de 1 a 99 e públicas (o Java usa nomes em texto e controle de acesso público/privado por UUID); o buffer e transporte de calor não são suportados | LIMITE |
| 20.4 | Chargepad: bloco e funcionalidade ainda não implementados no port (Java: 1 024 000 J/t = 409 600 FE/t, buffer 2 048 000 J = 819 200 FE, recarga de jogadores e Robits sobre a base) | FALTA |
| 20.5 | Teleporter: apenas jogadores são teleportados; não há teletransporte de mobs, veículos (barcos/minecarts) ou cálculo dinâmico de passageiros com multiplicador de energia; não há efeitos sonoros nem partículas de portal | LIMITE |
| 20.6 | Logistical Sorter: usa 2 slots de movimentação (0=intake, 1=outbox) e 5 slots de filtro físico no contêiner; não há suporte a coloração de canais de transporte, modo round-robin, contagem mínima/máxima por filtro nem controle por sinal de redstone | LIMITE |

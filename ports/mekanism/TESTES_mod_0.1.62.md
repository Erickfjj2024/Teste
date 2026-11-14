# Diagnóstico: mod 0.1.62 (o APK continua o v50)

Instale só o `mekanism.hmod` (0.1.62). A tela das máquinas comuns mostra agora `Status n T<ticks> P<progresso>` e a das fábricas `T<ticks>` ao lado da versão.

| # | O que fazer | Esperado |
|---|---|---|
| S1 | Crusher (ou outra) com energia, **sem** upgrades, com ferro na entrada; olhar `T` e `P` | `T` 200 e `P` subindo 1 a cada tick (de 0 a 199) |
| S2 | Instalar 8 upgrades de velocidade (esperar acabar de instalar: ~3 s por upgrade) e olhar de novo | `T` cai para 20 e `P` vai de 0 a 19 em 1 segundo |
| S3 | Me dizer o valor de `T` com os 8 upgrades e quanto o `P` sobe por segundo | Isso mostra se o mod usa o tempo curto e se o jogo o aplica |

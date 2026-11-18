# Teste: APK v50 (corrige o N1)

Instale o APK `horizon-android-v50.apk` (o mod continua o 0.1.61). Causa do N1: o Horizon só aceitava slots de 0 a 7 na configuração de lados, e a fábrica Advanced usa slots até o 12: o pedido falhava e os lados voltavam ao padrão.

| # | O que fazer | Esperado |
|---|---|---|
| N1 | Repetir: Advanced Smelting Factory, lado de saída (Output) virado para um Logistical Transporter ligado a um baú, minério de ferro na entrada, energia | O ferro sai pelo transportador para o baú (16 itens a cada 10 ticks no Advanced) |
| N1b | A mesma fábrica com o lado de saída voltado direto para um baú ou funil (ejeção automática ligada) | Os itens vão para o baú/funil |

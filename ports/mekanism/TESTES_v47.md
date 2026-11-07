# Testes da atualização: APK v47 (barra de energia nos itens)

Causa achada: o inventário do jogo é uma cópia enviada pelo servidor interno, e essa cópia não levava os dados do bloco (a energia do cubo ficava só no servidor). Agora o servidor envia a energia (e a capacidade) do item ao cliente. Instale o APK `horizon-android-v47.apk`.

| # | O que fazer | Esperado |
|---|---|---|
| Z1 | Carregar um Energy Cube, quebrar no survival e olhar o item na hotbar e no inventário | Barra verde-água embaixo do ícone, proporcional à energia (metade cheio = meia barra) |
| Z2 | Cubo do criativo / cubo vazio | Sem barra |
| Z3 | Recolocar o cubo e quebrar de novo | A barra continua igual |
| Z4 | Mover o cubo entre slots, pôr no baú, pegar de volta | A energia continua e a barra também (nada some ao mover) |

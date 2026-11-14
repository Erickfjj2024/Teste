# Testes da atualização: APK v38 + mod 0.1.37 (só os upgrades)

Instale o `horizon-android-v38.apk` e o `mekanism.hmod` (0.1.37). Responda em cada linha: **ok**, **parcial** ou **falhou**, com o que viu.

## Upgrades

Pegue os upgrades na aba criativa (ou fabrique: vidro + liga infusa + pó: osmio é velocidade, ouro é energia, ferro é químico).

| # | O que fazer | Esperado |
|---|---|---|
| M1 | Abrir um Crusher; olhar a direita, em cima | Aba "Upgrades" (26x18, com ícone) acima da aba vermelha de redstone |
| M2 | Tocar na aba | Abre uma janela larga com a lista à esquerda, a descrição no meio, dois slots à direita, o botão "Uninstall" e a fileira "Supported:" embaixo; dá para arrastá-la pelo topo; fecha pelo canto |
| M3 | Pôr 2 upgrades de velocidade no slot de cima da janela | Em 1 segundo somem do slot e aparece "Speed" na lista; escolher a linha mostra "Speed Upgrade / Amount: 2/8 / Effect: 1.78x" em letra pequena |
| M4 | Pôr minério no Crusher com os upgrades | Termina bem mais rápido que antes (200 ticks → 112 com 2) e a barra de energia gasta mais |
| M5 | Pôr um upgrade de **gás/químico** no Crusher | O slot **não aceita** (e na Purification Chamber aceita) |
| M6 | Escolher a linha e tocar "Uninstall"; depois com toque **longo/direito** | Sai um upgrade (depois todos) para o slot de baixo da janela; o efeito some |
| M7 | Energia: pôr upgrades de energia | A barra de energia da máquina cresce (capacidade maior) e o uso por tick cai |
| M8 | Fechar o mundo e reabrir | Os upgrades continuam instalados |
| M9 | Os slots da janela quando ela está **fechada** | Não dá para tocar neles (só existem com a janela aberta); shift-clique no inventário não manda item para lá |
| M10 | Rotary / Chemical Infuser com 3 upgrades de velocidade | "Effect: 8.0x" e passa 8 operações por tick (esvazia o tanque 8 vezes mais rápido) |

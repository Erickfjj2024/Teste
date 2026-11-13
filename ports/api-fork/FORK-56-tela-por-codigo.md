# FORK-56: tela por código (API 56, capacidade `screen`) — Fase 0, desenho

Estado: proposta para revisão. Nada disto está implementado.

## O que o fork já tem (e o desenho reaproveita)

| Peça | Onde | Uso aqui |
|---|---|---|
| Runtime de mods no cliente (wasmtime + cranelift, também no Android: `horizon-android` liga a feature `mods`) | `horizon-mods/src/runtime.rs`, mundo `horizon-ui-mod` | a tela roda nele, com o mesmo limite de combustível (fuel) por chamada e de memória (`StoreLimits`) |
| Desenho em lista, sem chamada por quadro | `render::set-hud(list<shape>)` (fill, quad, sprite 9-slice com tint, item, texto) | o buffer de comandos é a mesma ideia, ampliada |
| Tela declarada em JSON | `menu_screens.rs` (sprites do mod no atlas da GUI, `draw_tiled_bottom_up` dos medidores, EditBox do campo, teclado do Android `take_field_keyboard`) | o executor dos comandos chama estas mesmas funções; o JSON continua igual |
| Dados sincronizados no cliente | dados do menu (`OpenNativeScreen.data`), strings (`MENU_TEXT`), grade (`MENU_GRID`), slots do contêiner | lidos pela tela por imports |
| Ações de volta ao servidor | botão de menu (pacote de botão), `<mod>:menu_text`, `horizon:grid_click` | imports da tela chamam esses mesmos caminhos |
| horizon-ctl | `ui`, `click`, `type --menu-field`, `menu_texts` | as áreas que a tela declara viram itens do `ui` |

## Onde roda o código da tela

- No **cliente**, sempre: na janela do jogo (no Android também), não no servidor.
- O `.hmod` (um zip) ganha um segundo componente opcional, **`screens.wasm`**, ao lado de `mod.wasm`. Ele é feito para o mundo novo `horizon-screen-mod`.
  - `cargo horizon-mod build` compila esse componente a partir de um segundo crate do mod (no manifesto, `[screens] crate = "screens"`).
  - O código do servidor e o da tela ficam separados: o da tela não importa nada do servidor (mundo, jogadores), e o do servidor não desenha.
- O cliente carrega `screens.wasm` do `.hmod` instalado. É o mesmo arquivo que já traz texturas, idioma e telas JSON, então o cliente já precisa dele hoje.
  - Uma instância por mod, criada na primeira tela aberta e mantida enquanto o mundo estiver aberto.
  - O estado da tela vive dentro da instância, um por menu aberto.
- **Uma tela por tipo de menu:** o JSON do menu ganha `"screen": "code"`, e o host chama o componente. Sem isso, nada muda.
  - Se o componente faltar ou der erro, o host mostra a tela JSON se ela existir. Senão, mostra um painel com a mensagem de erro e o inventário.

## WIT proposto (resumo; o texto completo vai no patch)

```wit
interface screen-host {            // imports da tela (só cliente)
    record grid-entry { item: string, components: string, count: u64 }
    record stack { item: string, components: string, count: u32 }
    menu-data: func() -> list<s32>;              // dados do menu (como o servidor sincroniza)
    menu-strings: func() -> list<string>;        // set-menu-text / item-menu-update.menu-text
    menu-grid: func() -> list<grid-entry>;       // item_grid (API 55)
    slot: func(index: u32) -> option<stack>;     // slot real do contêiner (só leitura)
    carried: func() -> option<stack>;            // o cursor
    text-width: func(text: string) -> f32;       // fonte da GUI
    sprite-size: func(sprite: string) -> option<tuple<u32, u32>>;
    send-button: func(id: s32);                  // mesmo caminho do botão JSON
    send-text: func(field: s32, text: string);   // <mod>:menu_text
    send-grid-click: func(index: u32, button: u8, shift: bool);
    play-click: func();                          // ui.button.click
    close: func();                               // fecha o menu (como Esc)
    translate: func(key: string, args: list<string>) -> string;   // lang do jogo e do mod
}

interface screen-events {          // exportado pela tela
    record open-info { menu: string, width: f32, height: f32, gui-scale: f32, touch: bool }
    record frame { mouse-x: f32, mouse-y: f32, partial-tick: f32, time-ms: u64 }
    enum button { left, right, middle }
    record mods { shift: bool, ctrl: bool, alt: bool }
    variant input {
        mouse-pressed(tuple<f32, f32, button, mods>),
        mouse-released(tuple<f32, f32, button, mods>),
        mouse-dragged(tuple<f32, f32, button, f32, f32>),
        mouse-scrolled(tuple<f32, f32, f32>),
        key-pressed(tuple<string, mods>),          // nome do options.txt: key.keyboard.enter
        char-typed(string),
        field-changed(tuple<s32, string>),         // campo do host (ver Entrada de texto)
        field-submitted(tuple<s32, string>),
    }
    record setup { width: u32, height: u32, inventory: option<tuple<s32, s32>>, own-labels: bool }
    open: func(info: open-info) -> result<setup, string>;
    tick: func();                                    // 20/s, depois da sincronização
    render: func(frame: frame) -> list<u8>;          // o buffer de comandos (formato abaixo)
    input: func(event: input) -> bool;               // true: a tela consumiu
    close: func();
}

world horizon-screen-mod { import log; import screen-host; export screen-events; }
```

O SDK embrulha tudo num trait (`ScreenMod`, `Screen`) e num `Canvas` com métodos parecidos com os de `GuiGraphics`: `blit`, `blit_sprite`, `fill`, `draw_string`, `render_item`, `enable_scissor`/`disable_scissor`, `pose().push/translate/scale/pop`, `tooltip`. Assim o porte de cada `GuiXxx` fica quase linha a linha.

## Buffer de comandos (`render` → `list<u8>`)

- Binário little-endian. Cabeçalho: `u16 versão = 1`, depois os comandos em sequência, cada um como `u8 op` seguido dos argumentos.
- Strings vêm como `u16 tamanho` seguido dos bytes UTF-8, até 1024 por string.
- Cores em ARGB `u32`; coordenadas em `f32` de pixels da GUI, relativas ao canto superior esquerdo da tela.
- Ops da Fase 1:

| op | argumentos | equivale a |
|---|---|---|
| `FILL` | rect, argb | `fill` / `fillGradient` (1 cor) |
| `BLIT` | textura (`ns:gui/x`), rect destino, u, v, uw, vh, tex w/h, argb | `blit(...)` de textura inteira ou região |
| `SPRITE` | sprite (`ns:path` do atlas), rect, argb | `blitSprite` (9-slice pelo `.mcmeta` `gui.scaling`, como o jogo) |
| `NINE` | textura, rect, bordas l/t/r/b, u, v, uw, vh, tex w/h, argb | `GuiUtils.blitNineSliced` do Mekanism |
| `TEXT` | string, x, y, argb, flags (sombra, centro/direita), escala | `drawString` / `drawScaledText` |
| `ITEM` | stack (id, componentes, contagem), x, y, flags (decorações, contagem) | `renderItem` + `renderItemDecorations` |
| `FLUID` | fluido ou químico (id) ou sprite, rect, fração, argb, direção | medidores (`drawTiledSprite`), mesma rotina dos gauges JSON |
| `SCISSOR` / `UNSCISSOR` | rect (pilha) | `enableScissor` / `disableScissor` |
| `PUSH` / `POP` / `TRANSLATE` / `SCALE` | dx, dy / sx, sy | `pose().pushPose()` ... |
| `SLOTS` | — | aqui o host desenha os itens dos slots reais (depois do fundo, antes das janelas) |
| `SLOT` | índice, x, y, ativo | onde fica o slot real (o clique continua do jogo; inativo = escondido por janela) |
| `AREA` | id, rect, tipo (botão, aba, linha, campo), ativo, rótulo | área clicável: hover, `horizon-ctl ui`/`click`, controle/narração |
| `FIELD` | id, rect, máx., validador, texto inicial, foco | campo de texto do host (abaixo) |
| `TOOLTIP` | linhas (argb cada) | `renderComponentTooltip`, desenhado por último pelo host |

**Limites** (segurança e desempenho):
- até 8192 comandos e 256 KiB por quadro;
- uma pilha de no máximo 16 níveis para scissor e para pose;
- o combustível por chamada é o mesmo dos ganchos de cliente.

Se a tela passa de qualquer limite, o host desenha o quadro anterior, registra o erro no log e, se repetir, fecha a tela. O mundo `horizon-screen-mod` não importa arquivos, rede nem servidor: só `log` e `screen-host`.

### Por que um buffer e não uma chamada por comando, e quando chamar `render`

- Uma chamada WASM do host para o mod (ou o contrário) custa uma troca de contexto mais a conversão de argumentos pelo component model. Telas do Mekanism fazem 200 a 800 comandos por quadro (QIO: 48 células, cada uma com item e contagem). Com uma chamada por comando, isso dá centenas de travessias por quadro; com o buffer, dá uma, mais uma cópia de `memcpy`.
- O host só chama `render` quando algo muda:
  - depois de `tick` (dados novos);
  - depois de uma entrada, ou quando o cursor muda de área;
  - quando a tela pediu animação (bit no cabeçalho), a no máximo 30 por segundo.
  
  Nos outros quadros o host reexecuta o último buffer já decodificado. Assim um quadro parado custa zero WASM.
- **Medição** (Fase 1, e é critério de aceite):
  1. Um banco de teste em `horizon-mods/benches` mede, com o QIO como modelo (600 comandos), `render` + decodificação + execução contra uma chamada por comando.
  2. Meta no desktop: abaixo de 0,15 ms por `render`.
  3. No Android, o `horizon-ctl state` passa a dar o tempo do último `render`, e você mede no aparelho (ver "O que depende de você").

## Entrada de texto: o host é dono do campo (escolha)

O mod só posiciona o campo (`FIELD` no buffer) e recebe `field-changed`/`field-submitted`.

O host usa o mesmo `EditBox` da tela JSON, que já traz:
- foco, cursor e seleção;
- copiar e colar;
- validador;
- teclado de toque do Android (`take_field_keyboard`) e IME com composição;
- `horizon-ctl type --menu-field ID`, que funciona sem mudança.

Por que não deixar o mod desenhar o próprio campo: o Android exige ligar IME e composição de texto a cada campo. Cada mod teria que reimplementar cursor, seleção e área de transferência, e o horizon-ctl perderia o `type`. O custo da escolha é a aparência fixa, a da `EditBox` do jogo, que é a mesma do `GuiTextField` do Mekanism. Cor do texto e fundo transparente vêm como flags do `FIELD`.

## Camadas, janelas, abas, rolagem, arrastar

- **Janelas** (GuiWindow): ficam a cargo da tela.
  - A ordem do desenho é a ordem do buffer, e a tela desenha as janelas depois de `SLOTS`.
  - A tela marca com `SLOT ... ativo=false` os slots tapados pela janela, para o clique não passar.
  - O arrasto vem de `mouse-dragged`.
  - O host só garante que as entradas chegam à tela antes do jogo; se a tela devolver `true`, o clique não vai ao slot.
- **Abas fora do painel** (x negativo): podem, até 64 px fora do painel de cada lado. O host corta por aí e conta essa margem no cálculo do cursor.
- **Rolagem de lista:** fica com a tela (scissor + `mouse-scrolled` + arrasto). O SDK traz ajudantes prontos (`ScrollList`, `ScrollBar`), copiados do comportamento de `GuiScrollList` do Mekanism.

## Estado e rede

- **O que a tela lê** (por imports, a cada `tick`): dados do menu, strings, grade e slots, tudo já sincronizado no cliente.
- **Ações:** `send-button`, `send-text`, `send-grid-click`, que reaproveitam os canais existentes. O servidor do mod não muda; os mesmos ids de botão do JSON continuam valendo.
- **Slots reais:** são do host. A tela só diz onde ficam e se estão ativos; o clique de slot, o shift-click e o arrasto de pilha continuam sendo do jogo.

## horizon-ctl e testes

- **`ui`:** lista cada `AREA` como `screen-<id>`, com tipo, bounds, rótulo e ativo, e os `FIELD` como `menu-field-<id>`. Também traz `screen_commands`, a lista decodificada do último buffer, resumida (op e argumentos principais).
- **`click screen-<id>`:** clica no centro, pelo mesmo caminho do mouse.
- **Harness offline** (`cargo horizon-mod test`, sem jogo): passos novos em `tests/*.toml`:

  ```toml
  [[screen]]
  open = "qio:dashboard"
  data = [3, 0]
  strings = ["No frequency"]
  expect.commands = ["TEXT \"No frequency\" 8 6", "AREA 9100 tab", "..."]   # prefixos, em ordem
  click = [102.0, 25.0]
  expect.sent = ["button 9100"]
  ```

- **Exemplo:** `crates/horizon-mod-sdk/examples/screen-demo`, com servidor e tela. Tem botão de imagem, lista rolando, campo, janela com aba e grade.
- **E2E (Xvfb):** cenário `screen` no `agent_control_e2e.py`:
  1. abre a tela;
  2. compara `screen_commands` com o esperado e salva uma captura;
  3. faz `click screen-<id>` e confere a resposta do servidor;
  4. faz `type --menu-field` e confere o texto.

## Riscos

1. **Desempenho no Android:** o cranelift no aarch64 é bom, mas um `render` caro em todo quadro pesaria. A mitigação é o render sob demanda e o buffer cacheado, e falta medir no aparelho.
2. **Tamanho e tempo de instanciação:** um segundo componente WASM por mod é algo como 0,5 a 2 MB a mais, e a instanciação custa alguns ms na primeira tela. O custo é pago uma vez, e dá para pré-instanciar ao entrar no mundo.
3. **Fidelidade de texto:** a largura da fonte do Horizon tem de bater com a do Java para alinhar. `text-width` vem do host, que é exato para Latin.
4. **Ordem de desenho dos itens de slot:** com `SLOTS` explícito a tela escolhe onde eles entram. Uma tela antiga sem `SLOTS` recebe os itens no fim.
5. **Controle/teclado sem mouse:** a navegação por controle depende das `AREA`. Uma tela que não declare áreas não navega, e o SDK passa a declarar uma área a cada `button()`.
6. **Escopo:** o Mekanism tem cerca de 120 telas. A API cobre o núcleo; o que faltar (gráficos, render 3D de bloco na GUI, a janela de cor) entra em fases seguintes como ops novas, sem quebrar a versão 1 do buffer.

## Fases e estimativa

Estimativa em sessões de trabalho como as desta conversa.

| Fase | Entrega | Estimativa |
|---|---|---|
| **1a** | WIT + host no cliente (carregar `screens.wasm`, chamar `open/tick/render/input/close`, decodificar e executar as ops da tabela, `FIELD` com EditBox, `AREA`, `SLOT`/`SLOTS`), limites, SDK (`Canvas`, `ScreenMod`, ajudantes de lista e botão), `cargo horizon-mod` empacotando `screens.wasm`, stub do verificador, docs (`docs/mods-screen.md`), API 56 | 2–3 |
| **1b** | Harness `[[screen]]` offline, exemplo `screen-demo`, cenário e2e, `ui`/`click` do horizon-ctl, banco de medição | 1–2 |
| **1c** | Prova no Mekanism: `GuiFrequencySelector` (Teleporter, Quantum Entangloporter, QIO) e `GuiQIODashboard` (grade, busca, ordenação, aba lateral com janela) em código, no lugar do JSON; captura em Xvfb comparada com a tela do Java (as suas capturas de referência, se tiver) | 1–2 |
| 2 | Ops que faltam (gradiente, gráfico, render de bloco/entidade na GUI, janela de cor), controle/narração, pré-instanciação, medição no Android | depois da revisão |
| 3 | Migração das demais telas do Mekanism, aposentando o JSON de cada uma | depois da revisão |

## O que depende de você

1. **Quem porta as duas telas da prova (1c):** eu, no `ports/mekanism` da branch `ccr-8f7d1cb2-4mjir6`, ou você, em cima da API? Se não disser nada, eu porto, porque isso testa a API de verdade.
2. **Medição no Android:** preciso de números do aparelho (o `horizon-ctl state` vai dar o tempo de `render` e o fps com a tela aberta). Daqui só consigo medir no desktop (Xvfb + lavapipe, CPU).
3. **Referência visual:** se tiver capturas do Mekanism 1.21.1 em Java (GuiFrequencySelector, QIO Dashboard) na mesma escala de GUI, eu comparo com as do Horizon. Sem elas, a comparação é com o código do GuiXxx.

O resto (formato do buffer, campo do host, `screens.wasm` no mesmo `.hmod`, a convivência com o JSON por tipo de menu, os limites) eu decidi acima como o que é seguro; mudo se você discordar.

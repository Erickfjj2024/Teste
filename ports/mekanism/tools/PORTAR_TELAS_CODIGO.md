# Portar uma tela do Mekanism para código (API 56, capacidade `screen`) / Port a Mekanism screen to code

**PT-BR primeiro, English below each part.**

## 1. Objetivo / Goal

**PT-BR.** Cada classe `GuiXxx` do Mekanism 1.21.1-10.7.19.85 vira um módulo Rust em `mod/screens/src/`, escrito **perto do Java**: as mesmas posições, sprites, textos, cores e cliques. O JSON das telas (o que `tools/convert.py` gera) é só o ponto de partida e o plano B: **o Java é a verdade**. Onde o JSON se afasta do Java, siga o Java e anote a diferença no comentário do módulo.

**EN.** Each Mekanism `GuiXxx` class becomes a Rust module in `mod/screens/src/`, written **close to the Java**: the same positions, sprites, texts, colours and clicks. The converter's JSON is only the starting point and the fallback: **the Java is the truth**. Where the JSON differs from the Java, follow the Java and note the difference in the module's comment.

## 2. O que ler / What to read

| Para quê / For | Onde / Where |
|---|---|
| A API de tela / the screen API | `port/tools/docs-mods-screen.md` (cópia de `docs/mods-screen.md` do fork) |
| O esqueleto / the skeleton | `port/mod/screens/src/lib.rs`, `registry.rs`, `elements.rs`, `frequency.rs`, `teleporter.rs` (tela de referência) |
| A classe Java da tela / the Java class | `java/main/java/mekanism/client/gui/...` (procure `GuiXxx.java`), os elementos em `.../client/gui/element/...` e o container em `.../common/inventory/container/...` |
| O que o JSON de hoje faz / what today's JSON does | `port/_menus/<menu>.json` (gerado) e a função `<menu>_menu` em `port/tools/convert.py` |
| Os números e textos que o servidor manda / the numbers and strings the server sends | `port/mod/src/<módulo>.rs` (o comentário do topo diz o que é cada número: `menu_data`, e cada texto: `menu_texts`) |
| Texturas / textures | `port/tools/gui-sprites/textures/gui/sprites/` (sprites, 9-slice pelo `.mcmeta`) e `java/main/resources/assets/mekanism/gui/` (texturas) |

## 3. Como é um módulo de tela / What a screen module is

```rust
pub fn open(info: &OpenInfo) -> Opened { ... Ok((Box::new(MinhaTela { .. }), Setup { width: 176, height: 166, hide_inventory: false, own_labels: true })) }
impl MekScreen for MinhaTela {
    fn tick(&mut self) {}                                   // 20 por segundo
    fn render(&mut self, frame: &Frame, c: &mut Canvas) {}  // um quadro, do canto superior esquerdo do painel
    fn input(&mut self, e: &InputEvent) -> bool { false }   // true = a tela tratou
}
```

- Os dados vêm do servidor: `host::menu_data()` (os números do `menu_data`, 16 bits), `host::menu_strings()` (`menu_texts`), `host::menu_grid()`, `host::slot(i)`. Use os helpers de `elements.rs` (`data`, `pair`, `string`).
- Ações: `host::send_button(id)` (o mesmo `id`/`alt_id`/`shift_id` do botão do JSON), `host::send_text(id, texto)`, `host::send_grid_click`, `host::play_click()`.
- Os slots reais continuam do jogo: `Setup`/o JSON dizem onde ficam (`screen.slots` do JSON mantém as posições). Sem comandos `SLOT`, o jogo desenha os quadros dos slots; só use `c.slot(...)`/`c.slots()` para janelas que cobrem slots.
- Campo de texto: `c.field(id, rect, máx, filtro, texto, flags, cor)`; o jogo é o dono; ouça `InputEvent::FieldSubmitted`.
- Tooltips: `c.tooltip(&[(texto, cor)])` quando o mouse está sobre o elemento. Texto traduzido: `host::translate("chave.do.lang", &[])`; os textos fixos do JSON em inglês viram a chave de idioma do Mekanism (`gui.mekanism...`) quando ela existe no `en_us.json` (`java/datagen/generated/mekanism/assets/mekanism/lang/en_us.json`).
- Um módulo por classe `Gui`, **um arquivo seu**, e **uma linha** em `registry.rs` (`"mekanism:<menu>" => modulo::open(info),`). Se várias variantes usam a mesma tela (as quatro camadas de uma fábrica), uma função `open` atende todas (`info.menu`).
- Peças compartilhadas (abas, janelas) vão em `chrome.rs` e módulos próprios, listados no cabeçalho da sua tarefa; **não** duplique o que já existe em `elements.rs`.

## 4. Conferência / How you check your work

1. `tools/verificador/check.sh mod/screens/src check` (compila contra o SDK de mentira com o módulo `screen`) e `... test` (roda os testes). Sem erros nem avisos.
2. Para cada tela, testes de unidade no próprio módulo (`#[cfg(test)]`), com `host::set_menu(host::Menu { data, strings, .. })`: (a) um `render` não entra em pânico e contém os comandos esperados (`Canvas::finish()` e `screen_format::decode`, veja o stub), (b) um clique no botão X chama `host::take_sent()` == `["button 1234"]`.
3. **Nunca invente** um nome da API do SDK: use só o que está em `port/tools/verificador/sdk/src/` (`screen_format.rs` e o módulo `screen` do `lib.rs`). Se faltar algo, anote em `PEDIDOS_API.md` (o que, para qual tela, por quê) e siga sem isso.
4. O que você entrega: os arquivos **novos** em `port/mod/screens/src/` e a lista de linhas para `registry.rs` e de nomes para `CODE_SCREENS` (no `RESUMO.md`). Não mexa em outros arquivos (o servidor, o conversor, os outros módulos).

## 5. Regras de fidelidade / Fidelity rules

- Posição, tamanho, ordem de desenho e texto do Java, pixel por pixel; cores como `SpecialColors`/`EnumColor` do Java; o texto da barra de energia, dos medidores e das dicas como o Java monta (`MekanismLang`).
- O estado de um elemento (ativo, selecionado, em foco) como o do Java (por exemplo, o botão do modo escolhido fica inativo).
- Em dúvida entre o JSON e o Java, vale o Java; em dúvida sobre o que o servidor manda, vale `port/mod/src/`.
- O que não der para fazer (precisa de API que falta): comentário `// NÃO FEITO: motivo` no módulo, sem inventar API.

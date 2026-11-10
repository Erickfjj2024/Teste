# REVISÃO — Oredictionificator / Oredictionificator review

**Fontes / sources:** `TileEntityOredictionificator`, `GuiOredictionificator`, `GeneralConfig:243-244` (1.21.1-10.7.19.85) × `oredictionificator.rs`, `convert.py`.

## Corrigido / Fixed
1. **Tags válidas:** o padrão do Java é só o namespace `c` com os caminhos `ingots/ ores/ dusts/ nuggets/ storage_blocks/ raw_materials/`. O port aceitava também `gems`, `forge:` e tags sem namespace. Corrigido (teste).
2. **Ordem da busca (`getResult`):** primeiro as tags do item, e para cada tag os filtros ligados, na ordem; o port percorria os filtros primeiro. E o Java tenta **só o primeiro resultado**: se a saída tem outro item, não tenta outros filtros; o port continuava procurando.
3. **Limite da saída:** `outputStack.getCount() < limite do item`; o port usava 64 fixo. Agora o limite real do jogo (API 28).
4. **Tags do item:** eram um esqueleto (lista vazia); agora vêm do jogo (`recipe-types.item-tags`, API 30). A ordem do jogo é alfabética; a do Java é a do registro: diferença conhecida.
5. **Estado `didProcess`:** só era escrito quando convertia, então a seta de progresso ficava ligada; agora zera no tick seguinte. O pulso de redstone usa o sinal do tick anterior.
6. **Botões e tela:** o bloco não tinha tela (só a lista de menu). Tela 236 × 230 (a do original): lista de filtros em 3 linhas, seta de progresso grande, slots em 56, 115 e 164, 115, aba de redstone. Os botões de ligar/desligar cada filtro e a aba de redstone funcionam.

## Aberto / Open (FORK-20, `api-fork/FORK-20-telas-filtros.md`)
- **Não dá para criar nem editar um filtro:** falta o campo de texto da tag, o texto dinâmico (mostrar a tag e o resultado) e a lista rolável. **Sem isso a máquina não é utilizável**; o botão "New Filter" está desenhado e não faz nada.
- Upgrades, configuração de lados (o Java tem `withSideConfig(ITEM)`), segurança.
- Chamadas de computador.

**EN.** Fixed: valid tags (only `c:` with 6 prefixes), tag-major search with only the first result tried, the item's own stack limit, real item tags, `didProcess`, a screen with working filter switches and redstone tab. Open: creating or editing a filter needs a text field, dynamic text and a scrolling list (FORK-20); until then the machine cannot be used.

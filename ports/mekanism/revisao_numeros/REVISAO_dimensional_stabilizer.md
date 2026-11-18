# REVISÃO — Dimensional Stabilizer / Dimensional Stabilizer review

**Fontes / sources:** `TileEntityDimensionalStabilizer` (+ `ChunkLoader`), `TileComponentChunkLoader`, `PacketGuiInteract`, `UsageConfig`/`StorageConfig`, `MekanismBlockTypes:619` (1.21.1-10.7.19.85) × `dimensional_stabilizer.rs`.

## Conferido / Confirmed
Raio máximo 2 (grade 5×5), o chunk do meio sempre carregado, 5 000 J por chunk por tick (2 000 FE), 40 000 J (16 000 FE), o conjunto de chunks = os marcados em volta do bloco, as chaves de energia, o comparador 15 quando ativo (o Java só carrega os chunks **enquanto ativo**: `canOperate = allowChunkloading && getActive()`).

## Corrigido / Fixed
1. **Os botões nunca funcionavam:** liam a fila do *sorter*; nada os punha lá. Fila própria (`stabilizer_buttons`) e despacho em `menu_button`.
2. **Anéis:** o Java `adjustChunkLoadingRadius(r, load)` muda só o **anel** (a borda do quadrado de raio r); o port mudava o quadrado inteiro e tinha um "alternar tudo" inventado. Agora três pacotes como no Java: alternar um chunk (`7500 + x·5 + z`), ligar o anel r (`7530 + r`), desligar o anel r (`7540 + r`); testes (desligar o anel 2 deixa o anel 1).
3. **Redstone:** `canFunction()` faltava; agora o controle (guardado no estado) e o sinal decidem se gasta energia.
4. **Chunks carregados de verdade:** `sync_chunk_tickets` era um esqueleto; agora `chunks.set-forced` com a diferença entre o que já segura e o que quer (solta tudo ao parar), e solta tudo ao quebrar o bloco. O jogo limita a 256 chunks por mod e só no mundo em simulação.
5. A propriedade `active` do bloco acompanha o estado (`setActive`).

## Aberto / Open
- **Sem tela:** o Java tem a grade 5×5 e os botões de anel; o `convert.py` não declara essa tela (os botões do mod esperam IDs de uma tela que ainda não existe).
- Slot de carga (`energySlot.fillContainerOrConvert`) e upgrades de energia: o bloco não está na tabela `MACHINES`, então não há instalação de upgrades.
- Comparador (`set-comparator-output`), visualização dos chunks (`IHasVisualization`).
- Chunks de dimensões que o servidor não simula não aceitam tickets.

**EN.** Fixed: dead buttons, ring semantics (border only), redstone control, real chunk tickets via `chunks.set-forced`, `active` property. Open: no screen, charge slot and upgrades, comparator, chunk visualization.

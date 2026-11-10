# Revisão da lógica dos multiblocos / Multiblock logic review

> Tarefas m1 a m7 (uma por multibloco + o mecanismo comum), refeitas após a VM reiniciar. Cada patch foi aplicado numa cópia, compilado e testado com `tools/verificador/check.sh` antes de entrar (290 testes passam), e conferido contra o Java nos pontos de maior impacto.

## O que entrou / What went in
- **m1** Dynamic Tank e Induction Matrix; **m4** Turbine (já aplicados antes).
- **m2** Boiler: vidro estrutural só nas faces (aresta exige casing), volume de água como no Java, duplicação de fluido no primeiro tick, válvula solitária.
- **m3** Evaporation e SPS: vidro/aresta, válvula solitária, duplicação com baldes, mistura de fluidos por canos, deadlock do SPS com o tanque de antimatéria quase cheio, SPS drenando sem Supercharged Coil, comparador.
- **m5** Fission: Reactor Glass nas paredes, queima fracionária quando o combustível zera, burn rate 0 não vira 0,1 ao recarregar, fervura do fluido errado, portas inválidas, aviso de formação.
- **m6** Fusion: vidro na moldura, portas e menu só se formado, plasma, formação.
- **m7** Comum: formação (aviso a 40 blocos), vedação de portas/válvulas inválidas por tick, bounding blocks (Modification Station, cliques, quebra mútua), grupos múltiplos (vírgula).

## O que eu corrigi na revisão / What I fixed in review
1. **API inventada** (o verificador achou): `fission::LOGIC` e `fusion::CASING` (não existem: `ADAPTER` e `FRAME`), `MultiblockMember.data` (não existe) em Evaporation e SPS (o aviso de formação duplicado foi removido: o `lib.rs` já faz para todos), `Option::is_some` em energia (`u64`), `gen` (palavra reservada na edição 2024) e `u32::from(i32)` no volume do Fission.
2. **Conteúdo apagado ao desmontar:** Fusion e Fission zeravam tanques e energia das portas de estruturas inválidas, inclusive em carregamento parcial de chunks. O Java guarda o conteúdo no cache até reformar. Agora só se veda (filtros e faces), sem zerar.
3. **Faces de tanque com tamanho errado:** o `multiblock_changed` mandava 5 listas para válvulas de 2 tanques. Removido; a vedação por tick usa o número real de tanques.
4. **Teste errado:** `digital_miner_takes_twenty_six_bounding_blocks` esperava 26; o Java (`for x -1..=1, y 0..=1, z -1..=1`) dá 3×2×3−1 = **17**. Teste reescrito com o conjunto exato; o código já estava certo.
5. **S1 não está resolvido** (ver `SINTOMAS_MULTIBLOCOS.md`): o relatório do m7 dizia "S1 resolvido", mas devolver `false` em `item_used` não impede o menu. Depende do FORK-29.

## Ainda não conferido / Not yet checked
Os relatórios das contas citam muitas linhas do Java; conferi por amostragem os números e as regras de maior impacto, não cada uma das ~1 700 linhas alteradas. Os pontos que dependem do SDK real do fork (`on_blocks`, assinatura de `block_broken`/`block_placed` com `&str`, `player_control::get_motion().sneaking`) só serão confirmados no build real.

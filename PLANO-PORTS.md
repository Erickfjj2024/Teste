# Plano de ports: SkyFactory 4 para o Horizon

Objetivo: reescrever em Rust, um por um, os mods do SkyFactory 4, começando pelos de licença permissiva, para ter ports funcionando antes de pedir permissão aos autores dos mods com todos os direitos reservados.

## Etapa 0: o que já existe no Horizon (não precisa portar)
Pelo registro `horizon-mods`, já existem: AppleSkin, JEI (`jei`), Jade. Mouse Tweaks tem algo parecido em `mousetricks`, a conferir. Mods grandes como Create existem lá, mas não fazem parte do SkyFactory 4.

## Etapa 1: piloto
Um mod pequeno de licença permissiva, portado de ponta a ponta, para provar o fluxo. Candidatos: Clumps, ElevatorMod, Torchmaster, Toast Control, FastFurnace, FastWorkbench.

## Etapa 2: mods permissivos (58 no total, ver SKYFACTORY4-MODS.md, seções A, C1)
Na ordem: bibliotecas e utilitários pequenos, depois mods de conteúdo (Mystical Agriculture, Mystical Agradditions, Tinkers' Construct, Mekanism, Industrial Foregoing, Integrated Dynamics...). Mods que só trazem dados e assets viram `.hmod`. Mods com lógica viram `.hnmod`.

## Etapa 3: licenças pendentes
- 85 mods sem licença conhecida (seção C3): conferir na página do CurseForge.
- 59 mods com restrição (seções B e C2): ler a licença de cada um. LGPL e GPL podem permitir port se o resultado seguir a mesma licença, mas o registro do Horizon só lista código permissivo sem uma autorização escrita.

## Etapa 4: mods de direitos reservados
Mostrar aos autores o piloto e os ports prontos, pedir permissão por escrito, e só então portar.

## Regras fixas (MOD-POLICY.md do horizon-mods)
Manter os avisos de copyright e licença do original, creditar os autores e linkar o original. Nada de código ou assets do Minecraft.

## Pendências
- Rodar `cargo test` no template do Horizon foi bloqueado nesta sessão (código externo). Falta autorização para compilar e testar o código do template.
- Um mod nativo só roda dentro de um jogo compilado pelo launcher, a partir do código do Horizon.

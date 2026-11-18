# Revisão do QIO contra o Java / QIO reviewed against Java

> Módulos: `qio.rs`, `qio_dashboard.rs`, `qio_transfer.rs` (rodada 3, tarefas e1 e e2; compilação pela c3). Verificados com `tools/verificador/check.sh` (compilação + testes: 256 passam) e contra o Java 1.21.1-10.7.19.85.

## Conferido e igual ao Java / Checked and equal
- **Drives** (`QIODriveTier`): base 16 000 itens / 128 tipos, hyper-dense 128 000 / 256, time-dilating 1 048 000 / 1 024, supermassive 16 000 000 000 / 8 192.
- **Drive Array:** 12 slots (`DRIVE_SLOTS = 12`); **nenhum bloco de QIO usa energia** (sem `withEnergyConfig`), como no port.
- **`QIODriveData.add/remove`:** idênticos (falha se cheio ou sem espaço de tipo; `min(amount, capacidade − total)`).
- **`QIOFrequency.addItem`:** mesma regra de capacidade total e de tipos, e dois passos (drives que já têm o item, depois os outros). O Java itera `HashSet`/`HashMap` (ordem não determinística), então qualquer ordem do port é válida.
- **Importer/Exporter:** pulso a cada 10 ticks (`TICKS_PER_HALF_SECOND`), só com frequência e `canFunction`; `64 + 32 × upgrades` itens por pulso; `round(1 + upgrades / 2)` tipos (1 a 5); importar sem filtro começa **ligado**, exportar sem filtro começa **desligado**.
- **Redstone Adapter:** `emite = (stored > 0 && stored >= count) != inverted`.
- **Dashboard:** 3 janelas de crafting (`MAX_CRAFTING_WINDOWS = 3`), 10 slots por janela.

## Erro encontrado e corrigido / Bug found and fixed
- **Filtros por tag e por ID de mod usavam um casamento inventado** (só `*` no fim, sensível a maiúsculas). O Java usa `WildcardMatcher`: `*` em qualquer posição, `?` um caractere, `#` um dígito, sem diferenciar maiúsculas; o texto tem de ser usado por inteiro, a menos que o padrão termine em `*`. Portei o algoritmo literalmente (`wildcard_matches` em `qio_transfer.rs`) e acrescentei o teste `java_wildcard_matcher_rules`.

## Da c3 (compilação), aplicado / From the compile task, applied
`qio.rs` (bindings `ref` da edição 2024, ponte FORK), `security.rs`, `modification_station.rs` (o parse dos módulos instalados agora usa `rsplit_once(':')`, que preserva `mekanism:module_...`), `laser_tractor.rs` (`ref`), `fork_qio_calor.rs` (chamadas ao host atrás de `#[cfg(feature = "fork")]`).
**Não aplicado:** a "correção" da c3 em `heat.rs` (`sink.update()` na rede). Contradiz o Java (o sink de calor é preguiçoso e atualiza no próprio tick), então ficou o comportamento já revisado.

## Ainda não conferido / Not yet checked
`qio.rs` tem 1 675 linhas: conferi o armazenamento, a distribuição, o Redstone Adapter e as constantes; **não** li ainda linha a linha a sincronização dos clientes, a persistência dos drives (`DRIVE_CONTENTS`/`DRIVE_METADATA`), o `QIOGlobalItemLookup`, as listas de ordenação do Dashboard nem o crafting remoto. Isso fica para uma segunda passada, preferencialmente por outra conta (revisão cruzada) com o verificador.
Os FORK de QIO (frequências UUID com segurança, GUIs, renderização dos drives no array, tooltips) estão em `rodada3/e1/FORK_*.md` e `rodada3/e2/FORK_*.md`.

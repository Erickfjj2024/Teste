# ENTREGA — APK instalável (atualização) + `.hmod` / Installable APK (update) + `.hmod`

**PT-BR.** Estado honesto: neste ambiente (Termux/proot) **não há** JDK, Gradle, apksigner, o código-fonte do fork nem o SDK real do Horizon. Por isso **não consigo gerar o APK nem o `.hmod` aqui**; quem gera é a sessão na nuvem. Esta página é o pedido exato a colar lá.

## Pedido para a sessão na nuvem / Prompt for the cloud session
1. Clonar `Erickfjj2024/Teste` (branch `ccr-8f7d1cb2-4mjir6`) e o fork do Horizon; aplicar `ports/api-fork/*.patch` em ordem e implementar `ports/api-fork/PROPOSTA-API.wit` (prioridade: o que `PROGRESSO.md` lista).
2. Seguir `BUILD_NUVEM.md`: normalizar a base, `tools/build.sh`, `horizon-mod test` verde → `mekanism.hmod`.
3. **APK como atualização:** assinar com a **mesma keystore** do APK instalado (`dev.horizon.android.dev`), mesmo `applicationId`, `versionCode` **maior** que o atual (v52 ou o mais novo instalado), sem mudar `sharedUserId`. Sem isso o Android exige desinstalar. Não usar a keystore de debug a menos que o APK instalado tenha sido assinado com ela.
4. Entregar: `horizon-android-<versão>.apk` e `mekanism.hmod`, com `sha256`, e o resultado dos testes.

## O que preciso de você / What I need
- Onde está a keystore (ou confirme que a nuvem já a tem) e o `versionCode` instalado hoje.
- Onde está o repositório do fork (URL/branch).

**EN.** Honest state: no JDK/Gradle/apksigner, fork source or real SDK here, so the cloud session produces the APK and `.hmod`. For the APK to install as an update it must use the same keystore and applicationId and a higher versionCode. Please tell me the keystore location and the fork repo.

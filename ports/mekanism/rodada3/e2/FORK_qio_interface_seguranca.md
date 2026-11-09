# FORK-Requisitos: QIO Interface e Sistema de Segurança (qio_interface_seguranca)

> **Regra do projeto**: NÃO existem "limites do Horizon". Tudo o que a API atual do Horizon ainda não expõe diretamente é formalizado aqui como requisito do fork (`FORK-n`) com a assinatura WIT exata proposta, para expansão da API pública do ecossistema WASM do Horizon.
> No código Rust, toda lógica correspondente está integralmente implementada em Rust puro, isolando as pontes do host sob comentários `// FORK-n`.

---

## 1. Tabela de FORKs Necessários

| ID | Requisito / Requirement | Origem no Java / Java Source | Peças que Demandam | Interface / WIT Proposto |
|---|---|---|---|---|
| **FORK-18** | Frequências nomeadas, universais, por UUID e modo público/privado | `SecurityFrequency.java:22`, `QIOFrequency.java:45` | QIO Importer, Exporter, Dashboard, Security Desk | `interface frequency` |
| **FORK-23** | Componentes de dados estruturados em itens e blocos (Data Components) | `MekanismDataComponents`, `ModuleContainer.java:45` | QIO Dashboard, Importer, Exporter, Security Desk, Modification Station | `interface item-components` |
| **FORK-24** | Pacotes de rede cliente-servidor e transmissão em lote (`BulkQIOData`) | `BulkQIOData.java`, `PacketSyncSecurity.java`, `PacketRemoveModule.java` | QIO Dashboard, Security Desk, Modification Station | `interface net-bulk` |
| **FORK-20** | Telas de GUI ricas com lista rolável virtual contínua e caixa de texto | `GuiQIODashboard.java`, `GuiSecurityDesk.java`, `GuiModuleScrollList.java` | QIO Dashboard, Security Desk, Modification Station | `interface ui-widgets` |
| **FORK-27** | Framework de módulos acoplados em equipamentos (MekaSuit / Meka-Tool) | `MekanismModules.java`, `ModuleContainer.java:280-360` | Modification Station | `interface gear-modules` |
| **FORK-29** *(Novo)* | Bounding blocks 2x2x1 e 1x2x1 com redirecionamento de cliques e quebra ao mestre | `IBoundingBlock.java`, `TileEntityModificationStation.java:40` | Security Desk (1x2x1), Modification Station (2x2x1) | `interface multiblock-bounding` |
| **FORK-30** *(Novo)* | Resistência inquebrável dinâmica baseada no jogador (`AttributeCustomResistance(-1)`) | `AttributeCustomResistance.java`, `TileEntitySecurityDesk.java` | Security Desk | `interface block-security` |

---

## 2. Assinaturas WIT Exatas Propostas (`port/api/horizon.wit`)

### FORK-18: Frequências Universais por UUID e Nome
```wit
interface frequency {
    enum security-mode {
        public,
        private,
        trusted,
    }

    record frequency-id {
        name: string,
        owner: option<list<u8>>,
        mode: security-mode,
    }

    /// Cria ou registra uma frequência universal nomeada associada ao jogador proprietário.
    create-frequency: func(name: string, owner: option<list<u8>>, mode: security-mode) -> result<frequency-id, string>;

    /// Consulta dados de frequência ativa no mundo para acesso entre dimensões.
    get-frequency: func(name: string, owner: option<list<u8>>) -> option<frequency-id>;

    /// Exclui frequência e desvincula blocos associados.
    delete-frequency: func(name: string, owner: option<list<u8>>) -> result<_, string>;
}
```

### FORK-23: Componentes de Dados em Itens (Data Components)
```wit
interface item-components {
    /// Obtém os bytes serializados de um Data Component arbitrário de um ItemStack.
    get-component: func(item: string, component-key: string) -> option<list<u8>>;

    /// Grava um Data Component estruturado em um ItemStack sem perder durabilidade ou encantamentos.
    set-component: func(item: string, component-key: string, data: list<u8>) -> result<_, string>;

    /// Remove um Data Component específico do ItemStack.
    remove-component: func(item: string, component-key: string) -> result<_, string>;
}
```

### FORK-24: Pacotes de Rede e Sincronização em Lote
```wit
interface net-bulk {
    record catalog-entry {
        item: string,
        components: string,
        count: u64,
    }

    /// Envia o catálogo em lote de milhares de itens da rede QIO para o cliente do jogador.
    send-qio-catalog: func(player-id: string, catalog: list<catalog-entry>) -> result<_, string>;

    /// Emite atualização de frequência de segurança para fechamento de GUIs de jogadores não autorizados.
    broadcast-security-update: func(owner-uuid: list<u8>, mode: u8, override: bool) -> result<_, string>;
}
```

### FORK-20: Telas com Listas Virtuais e Caixas de Texto
```wit
interface ui-widgets {
    record virtual-scroll-spec {
        x: s32,
        y: s32,
        width: s32,
        height: s32,
        row-height: s32,
        channel: string,
    }

    record search-box-spec {
        x: s32,
        y: s32,
        width: s32,
        height: s32,
        placeholder: string,
        max-chars: u32,
    }

    /// Registra um elemento virtual rolável rolável sob demanda na tela ativa.
    add-virtual-scroll: func(menu-id: string, spec: virtual-scroll-spec) -> result<u32, string>;

    /// Registra campo de entrada de texto nativo com filtro e autocompletar.
    add-search-box: func(menu-id: string, spec: search-box-spec) -> result<u32, string>;
}
```

### FORK-27: Framework de Módulos de Trajes e Ferramentas
```wit
interface gear-modules {
    record module-entry {
        id: string,
        count: u32,
        enabled: bool,
    }

    /// Instala um módulo tecnológico em uma armadura ou ferramenta modular.
    install-module: func(gear-item: string, module-id: string, count: u32) -> result<u32, string>;

    /// Remove módulos instalados e devolve os itens ao jogador.
    remove-module: func(gear-item: string, module-id: string, count: u32) -> result<u32, string>;

    /// Lista os módulos atualmente instalados no contêiner do itemstack.
    get-installed-modules: func(gear-item: string) -> list<module-entry>;
}
```

### FORK-29 (Novo): Multibloco Bounding 2x2x1 e Redirecionamento
```wit
interface multiblock-bounding {
    /// Vincula os blocos secundários da Modification Station (2x2x1) ou Security Desk (1x2x1)
    /// redirecionando interações de clique direito, dano de mineração e abertura de contêiner para o mestre.
    bind-bounding-blocks: func(dimension: string, master-x: s32, master-y: s32, master-z: s32, offsets: list<tuple<s32, s32, s32>>) -> result<_, string>;
}
```

### FORK-30 (Novo): Resistência Inquebrável Customizada Baseada no Proprietário
```wit
interface block-security {
    /// Define a resistência à quebra (-1.0f para inquebrável em modo sobrevivência)
    /// para impedir que jogadores sem permissão destruam a Security Desk.
    set-custom-hardness: func(dimension: string, x: s32, y: s32, z: s32, hardness: f32) -> result<_, string>;

    /// Callback disparado quando um jogador tenta quebrar o bloco no mundo.
    can-break-block: func(dimension: string, x: s32, y: s32, z: s32, player-uuid: list<u8>) -> bool;
}
```

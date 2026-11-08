# Diferenças entre o Mekanism original e o port

Catálogo de tudo que **hoje não é igual** ao Mekanism (26.3 / NeoForge) no port para o Horizon. Regra daqui em diante: **toda fatia nova
que entrar com qualquer diferença, simplificação, suposição ou item faltando é registrada aqui no mesmo commit** (campo "Onde" = arquivo/versão).
Estado: `ABERTA` (ainda diferente), `SUPOSIÇÃO` (valor chutado, não lido do original), `LIMITE API` (depende do Horizon/fork), `FECHADA` (resolvida; fica só para histórico).

Legenda de origem: **mod X.Y.Z** = versão do `mekanism.hmod` em que a diferença apareceu; **APK vN** = precisa de recurso do fork.

---

## 1. Itens e dados gerais

| # | Diferença | Estado | Onde |
|---|---|---|---|
| 1.1 | Itens "normais" são itens comuns (graveto renomeado com textura). Jetpack, tablet, MekaSuit, Meka-Tool, Robit, módulos etc. **sem o comportamento** (ferramentas e armaduras do MekanismTools: seção 16) | ABERTA | tarefa #6 |
| 1.2 | Os upgrades são **8 itens** (`upgrade_speed`…) em vez de um item `mekanism:upgrade` com o tipo num componente | LIMITE API | mod 0.1.3x |
| 1.3 | Capacidade de energia das máquinas = uso por tick × 2500 (o 26.3 tem `TODO` nesses valores); cabo básico 1024/tick | SUPOSIÇÃO | config.rs |
| 1.4 | Dureza e resistência dos blocos são aproximadas (minérios 3, deepslate 4,5, armazenamento 5, casing 3,5) | SUPOSIÇÃO | convert.py |
| 1.5 | Baldes de fluido do Mekanism não colocam fluido no mundo (só enchem tanques/máquinas); fluidos de mod não existem no mundo | LIMITE API | mod 0.1.4x |
| 1.6 | Minérios entram **depois** da geração do chunk (passada própria), não no worldgen original | LIMITE API | worldgen.rs |
| 1.7 | Receitas: só as cujos ingredientes existem no port (solar pede painel + tablet, gás pede núcleo eletrolítico etc.) | ABERTA | convert.py |
| 1.8 | Silenciamento (Muffling) e contenção de XP são só guardados nos upgrades: o port não tem som de máquina nem XP de máquina | LIMITE API | upgrades.rs |
| 1.9 | Radiação: seção 15 | — | — |
| 1.10 | Sem sons de máquina, partículas e animações de bloco (rotação da turbina, LEDs) | LIMITE API | — |

## 2. Máquinas e fábricas

| # | Diferença | Estado | Onde |
|---|---|---|---|
| 2.1 | Com a redstone parada, a máquina **inteira** descansa; no original só pára a parte que processa | ABERTA | redstone.rs |
| 2.2 | Setas de progresso do extrator de pigmento, pintora e misturador não mudam de cor com o pigmento | ABERTA | convert.py |
| 2.3 | Topo do Pigment Mixer: energia fica ligada (no original `addDisabledSides`) | ABERTA | config.rs |
| 2.4 | Dica "Using" da aba de energia mostra o uso básico, não o com upgrades | ABERTA | convert.py |
| 2.5 | O ritmo de despejo do Electrolytic Separator não acompanha a velocidade | ABERTA | upgrades.rs |
| 2.6 | A lista de upgrades não rola (5ª linha escondida), sem realce ao passar o dedo na lista | LIMITE API | convert.py |
| 2.7 | Fábricas: sem aba **Sorting**; entrada aceita qualquer item (não só o que tem receita); sem `inputProducesOutput` (mesma entrada em todas as linhas) | ABERTA | factory.rs |
| 2.8 | Fábricas: a receita de subir de nível **não carrega** o que a fábrica de baixo guarda; sem o instalador de nível | ABERTA | convert.py |
| 2.9 | Heat Generator: tabela de queima = lista dos combustíveis comuns (a API não consulta tempo de queima) | LIMITE API | generators.rs |
| 2.10 | Geradores: slots de energia (item que recarrega) sem efeito, pois não há itens de energia (tablet) | ABERTA | — |
| 2.11 | Ícone de sol do Solar Generator desenhado à mão (sprites `sees_sun`/`no_sun` ausentes das fontes) | ABERTA | convert.py |
| 2.12 | Perda de calor por face mostrada arredondada | ABERTA | generators.rs |
| 2.13 | Digital Miner: seção 14 (Advanced Solar e Wind: seção 7; Isotopic Centrifuge e Neutron Activator: seção 13) | — | — |

## 3. Transmissão e logística

| # | Diferença | Estado | Onde |
|---|---|---|---|
| 3.1 | Itens **não andam visivelmente** pelo Logistical Transporter (teleportam), sem cor/roteamento, sem Restrictive/Diversion/Sorter, sem pull/push por lado nem filtros | ABERTA | tarefa #5 |
| 3.2 | Calor (Thermodynamic Conductor, Resistive Heater, Fuelwood Heater) não existe como transmissão | ABERTA | — |
| 3.3 | QIO, Teleporter, Quantum Entangloporter não existem | ABERTA | tarefa #5 |
| 3.4 | Harness de teste não simula redes de itens nem o limite de capacidade das portas (só testes unitários e conferência de estrutura) | LIMITE API | tests/ |

## 4. Multiblocos (todos **sem teste no jogo**)

### 4.1 Dynamic Tank (mod 0.1.65)
| # | Diferença | Estado |
|---|---|---|
| 4.1.1 | Tela só nas válvulas; sem aba de modo de edição (Fill/Empty) | ABERTA |
| 4.1.2 | Sem filtro de entrada por tipo na válvula além do "tanque cheio" | ABERTA |

### 4.2 Induction Matrix (mod 0.1.66)
| # | Diferença | Estado |
|---|---|---|
| 4.2.1 | Uma porta é entrada **e** saída ao mesmo tempo (o original deixa escolher o modo) | ABERTA |
| 4.2.2 | Tela simplificada (energia guardada, capacidade, taxa, células e provedores; sem gráfico de entrada/saída) | ABERTA |

### 4.3 Thermal Evaporation Plant (mod 0.1.67)
| # | Diferença | Estado |
|---|---|---|
| 4.3.1 | Tela simplificada (dois medidores e 4 slots de balde; temperatura, altura, sol, ritmo em texto) | ABERTA |
| 4.3.2 | Sol só conta se o céu estiver visível e `sky()` indicar dia; chuva/trovão aproximados pelo helper dos geradores | SUPOSIÇÃO |
| 4.3.3 | Sem bloco de logic/redstone de saída; sem som | ABERTA |

### 4.4 Thermoelectric Boiler (mod 0.1.68)
| # | Diferença | Estado |
|---|---|---|
| 4.4.1 | Válvulas **sem modo** (original: Input / Output Steam / Output Coolant): todas aceitam água e coolant quente e entregam vapor e coolant frio | ABERTA |
| 4.4.2 | Tela simplificada; temperatura em décimos de kelvin | ABERTA |
| 4.4.3 | Água exibida na tela por índice próprio (sem medidor animado); sem renderização 3D da água/vapor dentro da estrutura | LIMITE API |
| 4.4.4 | Volume de água = ar contido entre piso e dispersores (contagem própria, não a exploração `FormationProtocol.explore` com blocos de moldura); volume de vapor = largura×comprimento×altura acima do plano | SUPOSIÇÃO |
| 4.4.5 | Depende de sódio superaquecido do Fission (ou tanque criativo) para ser testado | — |

### 4.5 Industrial Turbine (mod 0.1.69)
| # | Diferença | Estado |
|---|---|---|
| 4.5.1 | Sem modo de despejo de vapor (Idle/Dumping/Dumping excess) | ABERTA |
| 4.5.2 | Sem animação do rotor e das lâminas | LIMITE API |
| 4.5.3 | Lâminas: tirar = clicar **agachado segurando uma lâmina** (original: mão vazia); blocos do rotor guardam lâminas num slot (cai ao quebrar) | LIMITE API |
| 4.5.4 | Tela simplificada na válvula; energia/água distribuídas igualmente entre válvulas/ventilações | ABERTA |
| 4.5.5 | Pressure Disperser é de **dois** grupos (boiler e turbina) por mudança no fork (grupo "a,b") | LIMITE API / fork |

### 4.6 Fission Reactor (mod 0.1.70)
| # | Diferença | Estado |
|---|---|---|
| 4.6.1 | Portas **sem modo** (Input / Output Waste / Output Coolant) | ABERTA |
| 4.6.2 | Logic Adapter é só um bloco de parede (sem saída de redstone nem modos) | ABERTA |
| 4.6.3 | Radiação do Fission: só o vazamento do lixo que não cabe no tanque e a liberação do meltdown (50×); não irradia entidades dentro da zona quente | ABERTA |
| 4.6.4 | Meltdown: remove blocos num raio de 8 (exceto bedrock/obsidiana/barreira/portais), sem a explosão real, sem dano a entidades, sem a cadeia de explosões do original | ABERTA |
| 4.6.5 | Sem cache/restauração do reator ao reconstruir (o original guarda por UUID); aqui o estado volta pelo menor canto da estrutura | ABERTA |
| 4.6.6 | Taxa de queima por botões (−1, −0,1, +0,1, +1), sem campo de texto/slider | ABERTA |
| 4.6.7 | Combustível (fissile fuel) depende da cadeia nuclear ainda não portada | ABERTA |
| 4.6.8 | Valores de calor (1.000.000 por mB, capacidade 1000/bloco) e limiares copiados da config padrão; condutividade do sódio 0,4 e entalpia 5 (padrão do `CooledCoolant`) | SUPOSIÇÃO |

### 4.7 Fusion Reactor (mod 0.1.71)
| # | Diferença | Estado |
|---|---|---|
| 4.7.1 | O plasma só esquenta pelo Laser/Amplifier (seção 6); o atalho criativo (clique direito na Laser Focus Matrix → 10× a temperatura de queima) não existe | ABERTA |
| 4.7.2 | **Hohlraum** é um item comum que conta como cheio (o port não tem itens com químico): dá 10 mB de combustível e é consumido ao acender. A receita (infusão de ouro + carbono) vem do original, mas ele sai "vazio" e já vale como cheio | LIMITE API |
| 4.7.3 | Portas **sem modo** (entrada / saída); todas aceitam água, deutério, trítio e combustível e entregam vapor e energia | ABERTA |
| 4.7.4 | Logic Adapter é só um bloco (sem redstone); Reactor Glass conta como moldura | ABERTA |
| 4.7.5 | Tela simplificada (sem abas Heat/Fuel/Stats, sem campo de texto: taxa de injeção por botões ±2); sem o gráfico de temperatura | ABERTA |
| 4.7.6 | O reator **não mata** entidades dentro dele enquanto queima (`kill`, dano 50000) | ABERTA |
| 4.7.7 | Sem transferência de calor para blocos vizinhos pelas portas (`simulateAdjacent`) | ABERTA |
| 4.7.8 | Hohlraum só entra pelo slot do controlador (não por automação) | ABERTA |
| 4.7.9 | O formato (grade 5×5, 12 faltando por face) foi lido do código; posições de borda usam o primeiro eixo de borda (a grade é simétrica, deve ser equivalente) | SUPOSIÇÃO |

### 4.8 SPS (mod 0.1.73)
| # | Diferença | Estado |
|---|---|---|
| 4.8.1 | Portas **sem modo** entrada/saída (o original alterna com Shift+clique): todas aceitam polônio e energia e entregam antimatéria | ABERTA |
| 4.8.2 | Não fere nem mata entidades dentro (`kill`), sem relâmpago nos "experimentos" de mobs, sem advancements | ABERTA |
| 4.8.3 | Sem animação das bobinas (`SyncableCoilData`) nem efeito visual | LIMITE API |
| 4.8.4 | Tela simplificada; a energia que as portas recebem é gasta toda no mesmo tick | ABERTA |
| 4.8.5 | A bobina é virada para a sua porta quando a estrutura se forma (o original usa a orientação ao colocar) | SUPOSIÇÃO |

## 6. Lasers (mod 0.1.72)
| # | Diferença | Estado |
|---|---|---|
| 6.1 | O raio é **invisível** (sem renderização do feixe) | LIMITE API |
| 6.2 | O laser **não cava blocos**, não fere nem queima entidades/itens no caminho, não é bloqueado por escudos nem armaduras (dissipação/refração) | ABERTA |
| 6.3 | **Laser Tractor Beam não existe** | ABERTA |
| 6.4 | Capacidade do laser = 50.000 FE (suposição: uso × 2500); o amplificador 5.000.000 FE (config `storage.laserAmplifier`) | SUPOSIÇÃO |
| 6.5 | Amplificador: sem modo de saída de redstone nem modos de entrada de redstone; limites por botões (−100k, −1k, +1k, +100k), sem campo de texto | ABERTA |
| 6.6 | O alvo é o primeiro bloco com colisão no caminho (não o `clip` com formas de contorno); só amplificador e Laser Focus Matrix recebem o raio | SUPOSIÇÃO |
| 6.7 | A orientação do bloco ao colocar usa o jogador mais próximo (até 12 blocos) e o ângulo de visão; vale só no primeiro tick | LIMITE API |
| 6.8 | As receitas do laser e do amplificador (tipo `mekanism:mek_data`) **não foram copiadas**: os blocos só saem do inventário criativo | ABERTA |

## 7. Blocos grandes e Bounding Blocks (mod 0.1.74)
| # | Diferença | Estado |
|---|---|---|
| 7.1 | O bloco principal coloca `mekanism:bounding_block` nas células vazias ao redor a cada segundo e as repõe se quebradas; **o bounding block não é quebrável** (some quando a máquina some) e **clicar nele não abre a tela** da máquina | LIMITE API |
| 7.2 | A máquina **não é recusada** quando as células que precisa estão ocupadas (o original impede a colocação); as que estiverem ocupadas ficam sem bounding block | LIMITE API |
| 7.3 | Wind Generator: o modelo real é desenhado por código (renderer de bloco), que o port não tem; o port usa um mastro e uma cruz de pás **estáticas** feitos à mão, sem girar; sem som | LIMITE API |
| 7.4 | Wind Generator só gera no overworld (altura 24 a 319 fixas; o original lê a altura/limites da dimensão e a lista `no_wind`); a produção é calculada a cada tick (o original, a cada segundo) | SUPOSIÇÃO |
| 7.5 | O Pigment Mixer agora também tem seu bounding block (acima), como no original | — |
| 7.6 | Advanced Solar: modelo e pontos de geração (12 FE/t, 240 de buffer) do original; os 9 blocos do painel e o de cima são bounding blocks | — |
| 7.7 | Máquinas com bounding block que ainda **não existem**: Digital Miner, Solar Neutron Activator, Isotopic Centrifuge, Seismic Vibrator, Security Desk, Modification Station, Teleporter | ABERTA |

## 8. Logistical Sorter (mod 0.1.75, exige APK novo: API 22 `world.pull-items`)
| # | Diferença | Estado |
|---|---|---|
| 8.1 | Os filtros são **itens postos em 5 slots** (não são gastos): só filtro por item. Faltam filtros de tag, material e mod, cor por filtro (roteamento nos transportadores), mínimo/máximo, "permitir padrão" | ABERTA |
| 8.2 | Sem Round Robin, sem controle de redstone, sem modo "ativo" (luz) | ABERTA |
| 8.3 | Puxa até uma pilha (64) a cada 10 ticks (o original 1 pilha a cada 5, e só quando o destino aceita); com "single item", 1 | SUPOSIÇÃO |
| 8.4 | O destino da frente é o do ejetor do jogo (baú, máquina, hopper) ou um transportador logístico ao lado; sem a cor/rota de transporte | ABERTA |
| 8.5 | O item passa pelo slot de entrada e depois pelo de saída (2 etapas); o original move direto | LIMITE API |

## 9. Quantum Entangloporter (mod 0.1.76)
| # | Diferença | Estado |
|---|---|---|
| 9.1 | Frequência é um **número** (1 a 99) escolhido por botões: sem nome, sem frequências privadas, sem segurança/dono, sem lista de frequências | LIMITE API |
| 9.2 | Compartilha energia (256 MFE), 1 fluido (256.000 mB), 1 químico (8.192.000 mB) e 1 pilha de item; **calor não** | ABERTA |
| 9.3 | Cada bloco guarda uma **fatia igual** do total (para não duplicar); um cabo/cano puxa só a fatia daquele bloco por tick, e o item é espelhado inteiro em todos (um item puxado de dois ao mesmo tempo pode duplicar) | LIMITE API |
| 9.4 | Sem configuração de lados (tudo entra e sai por todas as faces), sem ejeção automática (precisa de cabo/cano/transportador puxando), sem carregamento de chunk | ABERTA |
| 9.5 | Os portos dos multiblocos e os lasers passaram a conectar com cabos, tubos, canos e transportadores (tags `*_connectable`), o que faltava | — |

## 10. Teleporter (mod 0.1.77)
| # | Diferença | Estado |
|---|---|---|
| 10.1 | Frequência é um **número** (1 a 99) por botões: sem nome, sem privadas, sem cor de frequência, sem segurança | LIMITE API |
| 10.2 | Só **jogadores** são teletransportados (não mobs, itens nem veículos/passageiros) | LIMITE API |
| 10.3 | Sem portal desenhado na abertura (partículas/efeito), sem som, sem avanço | LIMITE API |
| 10.4 | O destino é o teleporter mais próximo da mesma frequência **carregado** (o original também; mas sem carregar o chunk, `IChunkLoader` não existe) | ABERTA |
| 10.5 | Capacidade de energia 5.000.000 FE (suposição: o valor do 26.3 é um placeholder); sem slot de carga de energia | SUPOSIÇÃO |
| 10.6 | A escala de coordenadas só conhece o Nether (8); outras dimensões de mod contam escala 1 | SUPOSIÇÃO |
| 10.7 | Cálculo de custo sem passageiros; o jogador chegado espera 5 ticks no destino e só é enviado de novo depois de sair da abertura | — |
| 10.8 | O alinhamento do jogador ao chegar (yaw para o quadro) não é feito: mantém a direção em que olhava | ABERTA |

## 11. Bins (mod 0.1.78)
| # | Diferença | Estado |
|---|---|---|
| 11.1 | O bin **ganha uma tela** (entrada, saída, contagem, "ejetar para baixo" e "travar item"); no original não há tela: coloca-se item usando-o e tira-se socando | LIMITE API |
| 11.2 | **Não dá para socar** o bin para tirar uma pilha (ou um item com Shift); tira-se pelo slot de saída da tela, por hopper, cano ou transportador | LIMITE API |
| 11.3 | Usar um item no bin coloca a pilha toda; não há "clique duplo para guardar tudo do mesmo item" | ABERTA |
| 11.4 | Itens com dados (encantamentos, nome, durabilidade...) **não são aceitos**; só itens comuns | LIMITE API |
| 11.5 | O tamanho da pilha de cada item é adivinhado pelo nome (64 para materiais/blocos e itens do Mekanism, 1 para o resto); itens que empilham até 16 aparecem de 1 em 1 | SUPOSIÇÃO |
| 11.6 | "Ejetar para baixo" (Shift+clique do original) é um botão na tela; com ele ligado só a face de baixo dá itens e as outras só recebem | ABERTA |
| 11.7 | Travar item por botão (o original trava com a mão vazia, agachando, na face da frente — não com o Configurator); sem receita de upgrade de nível que carrega o conteúdo | ABERTA |
| 11.8 | Bin criativo guarda 2^63 (não é infinito de verdade, mas nunca esvazia na prática) | SUPOSIÇÃO |

## 12. Structural Glass e Industrial Alarm (mod 0.1.79)
| # | Diferença | Estado |
|---|---|---|
| 12.1 | Structural Glass substitui o casing no Dynamic Tank, Boiler, Induction Matrix, SPS, Thermal Evaporation e Turbine; **não** no Fission nem no Fusion (que têm o Reactor Glass). O original aceita em mais posições (qualquer posição de moldura, também em arestas) e em outros multiblocos futuros | SUPOSIÇÃO |
| 12.2 | Industrial Alarm só acende (modelo aceso) com redstone; **sem sirene**, sem o feixe de luz que gira e sem luz de ambiente | LIMITE API |
| 12.3 | A face em que o alarme fica é a do jogador mais próximo ao colocar (não a face clicada) | LIMITE API |

## 13. Cadeia nuclear (mod 0.1.80)
| # | Diferença | Estado |
|---|---|---|
| 13.1 | Isotopic Centrifuge e Solar Neutron Activator usam a **tela e a configuração de lados do Chemical Infuser** (entrada à esquerda, saída à direita), com um slot central sem uso; a tela original deles é diferente | ABERTA |
| 13.2 | Solar Neutron Activator: taxa = 64 × (1 + efeito de temperatura e umidade) × brilho do sol (operações por tick, inteiras); a barra de energia aparece vazia (não usa energia) | SUPOSIÇÃO |
| 13.3 | O Activator só vê o céu pelo bloco de cima (como o original), mas conta bounding blocks acima como ar | — |
| 13.4 | Upgrades de velocidade/energia agem no Centrifuge como numa máquina contínua; no Activator os upgrades de velocidade não somam à taxa do sol | SUPOSIÇÃO |
| 13.5 | Sem radiação do lixo nuclear/plutônio/polônio | ABERTA |
| 13.6 | A cadeia do urânio inteira (minério, oxidação, hexafluoreto) usa as máquinas já portadas; não foi testada de ponta a ponta | — |

## 14. Digital Miner (mod 0.1.81, exige APK novo: API 23 `world.mine-block`)
| # | Diferença | Estado |
|---|---|---|
| 14.1 | Filtros: só por **item do bloco** (5 slots com um item cada); sem filtros de tag e de mod, sem "substituir por", sem puxar itens de trás | ABERTA |
| 14.2 | A tela é simplificada (botões −/+ de raio e alturas, interruptores Eject/Silk/Inv, Start/Stop); sem a visualização da área, sem a lista de filtros do original, sem campos de texto | ABERTA |
| 14.3 | Mina com uma picareta de netherita imaginária (Silk Touch opcional pelo interruptor); sem upgrades de velocidade/energia/silk | ABERTA |
| 14.4 | O varrer da área é do alto para baixo, em linha (o original ordena por distância/chunk); 512 posições por tick; fila de até 4096 | SUPOSIÇÃO |
| 14.5 | 40 FE/t enquanto há algo para minerar, um bloco a cada 80 ticks; buffer de 100.000 FE (suposição: uso × 2500) | SUPOSIÇÃO |
| 14.6 | Alturas absolutas (padrão 0 a 60; mundo −64 a 319); ejeta só pelas costas | ABERTA |
| 14.7 | Itens que não cabem nos 27 slots caem no chão onde o bloco estava; o original para de minerar quando o buffer enche | ABERTA |
| 14.8 | Sem XP, sem chunk loading; blocos com entidade (baús...) são quebrados e os conteúdos podem se perder | ABERTA |

## 15. Radiação (mod 0.1.82)
| # | Diferença | Estado |
|---|---|---|
| 15.1 | Fontes de radiação (posição, Sv/h, decaimento 0,9995 por segundo), dose do jogador (cresce com o nível onde está, menos a proteção do hazmat; decai) e dano por dose alta, como o original | — |
| 15.2 | O dano é do tipo "magic" (o tipo `mekanism:radiation` não existe no port); sem exaustão extra | LIMITE API |
| 15.3 | Só **jogadores** têm dose (mobs não); só o hazmat protege (sem cinto/Curios, sem a unidade de blindagem) | ABERTA |
| 15.4 | **Fontes só do Fission** (vazamento e meltdown): tanques quebrados com químico radioativo, itens radioativos, Chemical Tank/Dissolution etc. não irradiam | ABERTA |
| 15.5 | Geiger Counter e Dosimeter: clicar mostra o nível/dose no chat (sem a tela, o som e o painel de radiação; Dosimeter sem limpar a dose) | LIMITE API |
| 15.6 | O nível é lido uma vez por segundo por jogador, espalhado pelos ticks | — |

## 16. MekanismTools (mod 0.1.83)
| # | Diferença | Estado |
|---|---|---|
| 16.1 | Ferramentas (espada, picareta, machado, pá, enxada, paxel) e armaduras (4 peças) dos 6 materiais (bronze, lazulita, ósmio, glowstone refinado, obsidiana refinada, aço) e os paxels dos materiais do jogo: são itens de dados com os números do `*MaterialDefaults` do 26.3 | — |
| 16.2 | **Escudos, armadura de cavalo, armadura de nautilus e lanças não existem** (o jar 1.21.1 não traz as texturas; os escudos precisam de um renderizador próprio) | LIMITE API |
| 16.3 | Sem o **acabamento (trims)** nas armaduras; sem a configuração dos materiais (os números são os padrões) | ABERTA |
| 16.4 | O paxel usa a espada de ataque do machado (dano = machado + 1, velocidade do machado) e minera como machado + picareta + pá + enxada; o recipe do paxel virou crafting comum | SUPOSIÇÃO |
| 16.5 | A armadura usa a textura vestida (camada 1 e 2 do jar) pelo asset de equipamento do jogo: **não testado se o Horizon desenha** | — |
| 16.6 | Materiais "queimam no fogo" (netherite não): não foi aplicado (os itens de refined obsidian não têm `fire_resistant`) | ABERTA |
| 16.7 | Nugget das ferramentas fundidas (cozinhar ferramenta em pepita) não foi portado | ABERTA |

## 17. MekanismAdditions (mod 0.1.84)
| # | Diferença | Estado |
|---|---|---|
| 17.1 | Plásticos (comum, slick, glow, reforçado, estrada, transparente, degraus, lajes, cerca, portão) e glow panels nas 18 cores: **cada cor tem textura pintada própria** (o original pinta uma textura cinza com uma cor de bloco, que o port não tem); a cor vem de `EnumColor` | LIMITE API |
| 17.2 | **Estrada** não acelera quem anda, **slick** usa o gelo compactado (atrito 0,98 do gelo), reforçado não tem a resistência exata de explosão (1200, valor copiado) | ABERTA |
| 17.3 | **Cercas** conectam só entre cercas e portões de plástico (tag `plastic_fences`), não com blocos sólidos como as cercas do jogo; a forma é a do template da planta de chorus | LIMITE API |
| 17.4 | **Glow panel** sempre nasce virado para o norte (não conhece a face clicada); com um bloco inteiro como colisão | LIMITE API |
| 17.5 | Lajes soltam 1 item mesmo duplas; fluido (waterlogged) nas lajes/degraus/cercas não é o `ExtendedFluidLogType` do original | ABERTA |
| 17.6 | **Não existem**: balões (item e entidade), Obsidian TNT, Walkie-Talkie, os mobs bebês e o recolorir por receita (balões/glow panels), a extração de pigmento dos plásticos | LIMITE API |
| 17.7 | As luzes dos glow (nível 10) existem; o brilho "emissivo" do original (sem escurecer) não | LIMITE API |

## 18. Configurator (ramo `configurator-fixes`, **sem teste no jogo**)

| # | Diferença | Estado |
|---|---|---|
| 18.1 | Sem indicador de modo por modelo/tooltip/HUD: o motor desenha um modelo por item (o original troca por predicado `mekanism:mode`) e o `hud` só existe no mundo cliente, não no do mod servidor; a troca de modo é anunciada no chat | LIMITE API |
| 18.2 | A chave inglesa desmonta com `mine-block` e os drops caem onde o bloco estava (o original põe o bloco no inventário do jogador); sem API para pôr itens no inventário do jogador | LIMITE API |
| 18.3 | A sensibilidade a redstone dos transmissores e a cor do Logistical Transporter são guardadas mas não têm efeito no jogo (as redes movem pelas faces dos vizinhos; sem representação de cor na API) | LIMITE API |
| 18.4 | O modo de porta/válvula de multibloco zera quando a estrutura é encontrada de novo (o byte de modo vive nos dados da porta, que a validação reescreve) | ABERTA |
| 18.5 | A troca de modo é agachar+usar no ar (o original usa menu radial N / Shift+scroll, que não existe no Android) | LIMITE API |
| 18.6 | A receita do configurator (ferro/ósmio/aço) existe no datagen 1.21.x, mas não foi confirmada no mod construído (o `convert.py` não roda até o fim contra o checkout 1.21.x; precisa do 26.3 + fork + cargo-horizon-mod) | ABERTA |
| 18.7 | O truque criativo da Laser Focus Matrix (plasma a 1 bilhão) só funciona com o configurator; com a mão vazia, como no original, não há uso no port | ABERTA |
| 18.8 | Não existem no port: Thermodynamic Conductor, Restrictive Transporter e Diversion Transporter (sem modo de desvio) | ABERTA |

## 5. Fork do Horizon (precisam de APK novo)

| # | Mudança | Estado |
|---|---|---|
| 5.1 | Bloco com **vários grupos** de multibloco separados por vírgula (`a,b`) | pendente APK (ainda não gerado) |
| 5.4 | `world.mine-block` (API 23): quebra o bloco como um jogador com uma ferramenta e leva os itens ao contêiner (Digital Miner) | pendente APK |
| 5.3 | `world.pull-items` (API 22): puxa itens do contêiner ao lado para o bloco do mod (usado pelo Sorter) | pendente APK |
| 5.2 | Itens `set-item-faces` até 32 slots, `transmitter-medium: items`, widgets `Item`, `shift_id`/`hover`, barra de energia nativa | APK v50 |

---

## Histórico de fechamentos
(preencher quando uma diferença for resolvida e confirmada no jogo)

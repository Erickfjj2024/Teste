# Auditoria: o que o porte ainda difere do Mekanism original

Feita com as três fontes que temos: o **código 26.3** (ramo padrão do GitHub, a referência do porte), o **jar 1.21.1** (última release
publicada, `1.21.1-10.7.19.85`) e o **jar do MekanismGenerators 1.21.1**. Os números abaixo foram medidos por script
(comparando arquivo a arquivo), não estimados. Marcados: **[medido]** = conferido por script; **[código]** = conferido lendo o
Java; **[conhecido]** = já estava no `STATUS.md`.

## 1. Visual: o que copiamos bate com o original

- **Texturas [medido]:** das 331 texturas do porte que existem no jar 1.21.1, **330 são idênticas byte a byte**; a única que difere
  é `item/dye_base.png`. As 95 só do porte são painéis de tela montados na conversão (`gui/*`).
- **Modelos [medido]:** de 227 modelos que existem nos dois, **188 são idênticos**. Os 39 que diferem são deriva entre 1.21.1 e 26.3:
  a maioria é só `render_type: cutout` (o 26.3 tirou essa chave dos modelos; o Horizon não lê essa chave de qualquer jeito, e o vidro
  dos tanques já apareceu certo no jogo) e o modelo de item dos tanques de fluido (renderizador especial). Os modelos de item de
  transportadores, armaduras, jetpack, configurador etc. diferem porque o porte usa os do 26.3.
- Os 81 modelos só do porte são os que fizemos à mão (cabos, tubos, canos, Energy Cube com portas e LEDs por face).

## 2. Cobertura de conteúdo

**Blocos e itens [medido]:** o 26.3 define 344 itens/blocos; o porte tem uns 215 reais, faltando **~129**:

| Grupo | O que falta |
|---|---|
| Fábricas (24) e nível das máquinas | `basic/advanced/elite/ultimate` de crushing, enriching, smelting, combining, compressing, purifying, injecting, infusing, sawing |
| Armazenamento e rede (≈24) | Bins (5), Induction Cell/Provider (8), Induction Casing/Port, Dynamic Tank/Valve, Logistical Transporter e Thermodynamic Conductor (calor) |
| Máquinas (43) | Pressurized Reaction Chamber, Digital Miner, Teleporter (+frame), Quantum Entangloporter, QIO (Dashboard, Drive Array, Exporter, Importer, Redstone Adapter), Logistical Sorter, Electric Pump, Fluidic Plenisher, Formulaic Assemblicator, Fuelwood Heater, Resistive Heater, Isotopic Centrifuge, Nutritional Liquifier, Antiprotonic Nucleosynthesizer, Pigment Extractor/Mixer, Painting Machine, Oredictionificator, Chargepad, Modification Station, Security Desk, Robit, Solar Neutron Activator, Thermal Evaporation (3), Boiler (2), Laser (3), Supercharged Coil, caixas de resíduo radioativo |
| Energy Cubes | Advanced, Elite, Ultimate (só Basic e Creative) |
| Outros blocos/itens (19) | Cardboard Box, Configuration Card, Crafting Formula, Creative Bin, Dimensional Stabilizer, Electric Bow, Geiger Counter, HDPE Elytra, Industrial Alarm, Meka-Tool, Pressure Disperser, SPS (2), Structural Glass, Superheating Element, Upgrade |
| Baldes (18) | todos os baldes de gás/líquido (hidrogênio, cloro, salmoura, sódio, enxofre, vapor...): por isso não há como obter esses líquidos no jogo |
| Geradores | Advanced Solar, Wind, e os de multibloco (turbina, fissão, fusão) |

**Receitas [medido]** (jar 1.21.1 → porte):

| Tipo | Original | Porte |
|---|---|---|
| crushing / enriching / sawing | 313 / 322 / 440 | 255 / 205 / 157 (o porte descarta as que fazem algo que ainda não existe, como placas, vidros e itens de blocos novos) |
| metallurgic infusing | 37 | 30 |
| combining | 82 | 106 (o 26.3 tem receitas novas, as de cera do cobre) |
| injecting | 91 | 109 |
| compressing / crystallizing / dissolution / oxidizing / purifying / rotary / washing / separating / chemical infusing / conversion | 2 / 10 / 23 / 25 / 28 / 17 / 7 / 2 / 4 / 24 | iguais |
| **reaction** (Pressurized Reaction Chamber) | 14 | 0 |
| painting / pigment extracting / pigment mixing | 176 / 226 / 16 | 0 |
| nucleosynthesizing / centrifuging / activating / evaporating / energy conversion | 21 / 2 / 1 / 2 / 2 | 0 |
| mek_data / bin / clear_configuration (receitas especiais) | 85 / 2 / 1 | 0 |

Os nomes de 894 arquivos de receita têm o mesmo nome nas duas versões mas conteúdo diferente: é o formato que mudou entre 1.21.1 e
26.3 (o porte segue o 26.3).

**Geração de mundo [código, jar]:** os 10 veios de minério (estanho 2, ósmio 3, urânio 2, fluorita 2, chumbo 1) estão. **Falta a
geração de sal** (`salt`: disco de bloco de sal no fundo de oceanos, sobre terra ou argila): por isso o bloco de sal só vem do criativo.
Também falta a geração retroativa (retrogen) dos minérios em chunks já gerados.

## 3. Comportamento das máquinas que existem: o que difere

[conhecido] e [código]:

1. **Upgrades: feitos (mod 0.1.37)**, menos o som, a experiência, e a instalação com Shift+clique no bloco (ver `STATUS.md` 2j).
2. **Controle de redstone: feito (mod 0.1.36).** Segurança e dono continuam sem (a aba de segurança não existe; sem dono/proteção).
3. **Aba de energia** das máquinas: feita (mod 0.1.36).
4. **Itens químicos:** não há latas de gás nem conta-gotas (gauge dropper); por isso os slots de gás (Rotary, Gas-Burning, químico das
   máquinas) não têm função com recipientes: só funcionam as conversões de item que o porte já tem (carvão, redstone, bio fuel).
5. **Itens de energia:** não há Energy Tablet nem itens que carregam: os slots de carga/descarga das máquinas (e o dos geradores)
   só convertem redstone/bloco de redstone (aproximação) e nada mais.
6. **Calor:** não existe `Heat` como transmissão (nem Thermodynamic Conductor, Resistive Heater, evaporação): o Heat Generator tem um
   modelo de calor próprio e fixo em 300 K de ambiente (o original usa a temperatura do bioma).
7. **Tabela de queima do Heat Generator:** lista dos combustíveis comuns (a API do Horizon não consulta tempo de queima).
8. **Solar [código]:** o brilho do céu é calculado pela fórmula clássica do vanilla (hora do dia + chuva + trovão); o original lê o
   atributo `SKY_LIGHT_FACTOR` do 26.3, que pode diferir um pouco. A visão do céu varre os blocos acima (heightmap `WORLD_SURFACE`).
9. **Fluidos e gases:** o medidor usa um ícone genérico tingido (o original usa a textura animada do fluido/gás); o líquido dentro do
   tanque de fluido é uma caixa tingida; gases e líquidos de Mekanism não existem como blocos/itens (não há baldes).
10. **Som e partículas:** nenhuma máquina tem o som de funcionamento, fumaça/chama (geradores) ou partículas; luz emitida pelo bloco
    ativo (`ACTIVE_MELT_LIGHT`) não existe.
11. **Comparador e redstone de saída** (`getRedstoneLevel`): ausente nos tanques e geradores.
12. **Ejeção de itens:** a opção de ejetar itens das faces existe (`world.set-item-eject`), mas não foi comparada com o
    `TileComponentEjector` do original (ritmo e alvos): **não verificado**.
13. **Crystallizer:** a tela interna que mostra o cristal que fará (`GuiCrystallizerScreen`) não existe.
14. **Unidades:** o porte segue as unidades do 26.3 (FE; capacidades como `uso × 2500 ticks`): a tabela de armazenamento de energia
    das máquinas não está nos fontes 26.3 (marcada "TODO"), então a capacidade das máquinas é uma **suposição**; a dos geradores é a
    do `GeneratorsStorageConfig` (exata).
15. **Dois ids:** os geradores são `mekanism:bio_generator` etc. (no original, `mekanismgenerators:`): limitação do Horizon, que só
    deixa um mod declarar blocos no próprio namespace.

## 4. Equipamento: ferramentas, armaduras e módulos

Os itens existem com modelo, mas **sem função**: Configurator e Configuration Card (só o Configurator faz a configuração de faces no Energy
Cube/máquinas), Atomic Disassembler, Flamethrower, Electric Bow, Meka-Tool, Canteen, **MekaSuit (4 peças), Hazmat, Jetpack, Scuba, Free
Runners**: dá para vestir, mas não têm o modelo 3D, nem módulos, energia, absorção de dano ou efeitos. O módulo `mekanismtools` (ferramentas
e armaduras de bronze, ósmio, aço etc.) e o `mekanismadditions` (Balloon, Glow Panel, Plastic, Obsidian TNT, Baby mobs) **não foram
portados**; também não há Robit, radiação, QIO, Digital Miner, Teleporter e multiblocos (turbina, fissão, fusão, boiler, indução, evaporação).

## 5. Recomendação de ordem (maior impacto na fidelidade primeiro)

1. **Aba de upgrades + controle de redstone + aba de energia** nas máquinas que já existem (afeta tudo o que está portado).
2. **Pressurized Reaction Chamber** (14 receitas, motor de continuous/process já quase pronto) e **Pigment Extractor/Mixer/Painting**
   (418 receitas, pouco código novo).
3. **Fontes dos fluidos e gases:** baldes e blocos de fluido (para obter salmoura, cloro, etc. fora do criativo) e **geração de sal**:
   desbloqueia a cadeia química inteira no jogo.
4. **Fábricas** (24 blocos, mesma lógica das máquinas) e **Energy Cubes** de outros níveis.
5. **Advanced Solar e Wind** (exigem "bounding blocks" na API do Horizon) e **tubulações de itens** (Logistical Transporter, Sorter).
6. Heat (transmissão de calor), evaporação, boiler, indução, turbina, fissão e fusão.
7. MekaSuit, módulos e as ferramentas (`mekanismtools`), e então o Robit.

## 6. O que ainda pode ser conferido com os jars e não foi

- Comparar **todas as telas** posição a posição com os `Gui*` do 26.3 (feito à mão para cada máquina; falta uma checagem mecânica).
- Comparar os **números de configuração** (`UsageConfig`, `StorageConfig`, `GeneratorsConfig`) do 26.3 com as constantes do mod
  (feito à mão máquina a máquina; falta um script que liste as diferenças).
- Se aparecer um jar da série 26 no Maven (`modmaven.dev/mekanism/Mekanism/`), refazer a comparação de modelos e receitas com ele.

# Telas de utilidades feitas / Utility screens done

**PT-BR.** Escritas a partir das classes `Gui*` do Java 1.21.1-10.7.19.85 (medidas, slots e textos) e validadas rodando o conversor; **nenhuma vista no jogo**.
- **Seismic Vibrator** (`GuiSeismicVibrator`): tela interna 16, 23 (112 × 40) com "Vibrating"/"Idle" e o chunk (`Chunk: x, z`, pelo texto do menu); slot de carga em 143, 35 (converte redstone, como o `EnergyInventorySlot`); barra de energia, aba de energia e de redstone. No mod: slot de carga, redstone com pulso (sinal do tick anterior) e botões da aba.
- **QIO Drive Array** (`GuiQIODriveArray`, 176 × 206): tela de dados da frequência em 15, 19 (nome, barras de itens e de tipos com o quanto guardam), 12 slots de drive (34 + 18x, 70 + 18y) que só aceitam drives QIO, inventário. **Falta** a aba de frequência (escolher a frequência: campo de texto e lista, FORK-20).
- **Personal Chest / Personal Barrel** (`GuiPersonalStorageTile`, 176 × 222): 6 × 9 slots em 8, 18; a mesma tela para os dois. Faltam as abas de segurança.
- **Resistive Heater, Fuelwood Heater, Oredictionificator, Formulaic Assemblicator, Dimensional Stabilizer:** ver as revisões próprias.

**Sem tela ainda e por quê:** QIO Redstone Adapter (precisa do slot "fantasma" onde se escolhe o item a vigiar e do campo da contagem), QIO Dashboard (lista rolável de itens e busca), QIO Importer e Exporter (filtros), Security Desk (lista de jogadores e modos), Modification Station (lista rolável de módulos e a instalação), Laser Tractor Beam (o bloco ainda não está ligado: depende de dureza de bloco e das quedas). Quase todos dependem do FORK-20 (campo de texto, lista rolável).

**EN.** Done: Seismic Vibrator, QIO Drive Array (without the frequency tab), Personal Chest and Barrel. Left, and why: the QIO adapter, dashboard, importer/exporter, Security Desk and Modification Station need the text field and scrolling list; the Laser Tractor Beam is not wired.

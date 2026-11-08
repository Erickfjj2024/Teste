# Testes — mod 0.1.65 a 0.1.79 (tudo que foi construído desde o último teste)

**Precisa de APK novo** (v51 ou maior): o fork ganhou blocos em vários grupos de multibloco e `world.pull-items` (API 22). Sem o APK novo, o Boiler/Turbine com o disperser, o Sorter e o que depender disso não funcionam. Tudo o mais abaixo foi só testado por testes unitários.

Use o modo criativo para pegar os blocos. Marque cada item: ok / não funcionou (e o que viu).

## Dynamic Tank (0.1.65)
1. Caixa oca de `dynamic_tank` (3 a 18 de lado) com uma `dynamic_valve` numa face (não na aresta). Coloque água por um balde no slot da válvula: a tela mostra a contagem e a capacidade (16.000 baldes por bloco de dentro).
2. Quebre um bloco do casco: a estrutura some; refaça: volta com o conteúdo.

## Induction Matrix (0.1.66)
1. Caixa de `induction_casing` com uma `induction_port`, uma célula e um provedor dentro. Ligue um cabo com energia: a tela mostra a energia crescendo. Um cabo na outra porta puxa energia.

## Thermal Evaporation (0.1.67)
1. Torre 4×4 de altura 3 a 18 de `thermal_evaporation_block`, uma `valve`, um `controller` e os cantos do topo com `solar_generator`. Água na válvula: sai brine no tanque de saída com o sol (a temperatura sobe).

## Boiler (0.1.68) — precisa de sódio superaquecido (tanque criativo)
1. Caixa de `boiler_casing` com `boiler_valve`, plano de `pressure_disperser` e `superheating_element` embaixo. Água + sódio superaquecido nas válvulas: sai vapor.

## Turbine (0.1.69)
1. Torre ímpar (5 a 17), complexo no meio, rotores embaixo, plano de dispersers na altura do complexo, bobina em cima, vent e válvula. Clique com `turbine_blade` nos rotores; vapor na válvula gera energia e água na vent.

## Fission (0.1.70), Fusion (0.1.71), Lasers (0.1.72), SPS (0.1.73)
- Fission: caixa com porta, colunas de `fission_fuel_assembly` com uma `control_rod_assembly` em cima de cada. Ligue na tela; água vira vapor. (Combustível: tanque criativo de fissile fuel.)
- Laser + Laser Amplifier: mire na Laser Focus Matrix do Fusion; aumente o mínimo/máximo do amplificador.
- Fusion: formato 5×5×5 arredondado, controlador no meio do topo, hohlraum no slot, deutério e trítio nas portas.
- SPS: cubo 7×7×7 arredondado, bobinas atrás das portas, polônio e energia nas portas: sai antimatéria.

## Blocos grandes (0.1.74)
Advanced Solar (modelo grande, 9 blocos em cima), Wind Generator (mastro de 4 blocos), Pigment Mixer (1 bloco em cima): os bounding blocks aparecem em 1 segundo e somem ao quebrar a máquina.

## Logística (0.1.75 a 0.1.78)
- Sorter: baú atrás, outro baú na frente; coloque um item nos slots de filtro (ou ligue Auto-eject).
- Entangloporter: dois blocos na mesma frequência, cabo/cano ligado em um, tira do outro.
- Teleporter: dois com 9 frames cada, mesma frequência; entre na abertura com energia.
- Bins: use um item no bin (guarda a pilha); mão vazia abre a tela.

## 0.1.79
Structural Glass no lugar de um casing de Dynamic Tank. Industrial Alarm com redstone: o modelo acende.

Diferenças conhecidas (não são bugs): ver `DIFERENCAS.md`.

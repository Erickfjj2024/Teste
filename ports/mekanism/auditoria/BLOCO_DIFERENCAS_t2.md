## 20. Multiblocos: Dynamic Tank, Thermoelectric Boiler e Thermal Evaporation (mod 0.1.86)

| # | Diferença | Estado |
|---|---|---|
| 20.1 | Dynamic Tank: interface gráfica acessível exclusivamente nas válvulas (no original, blocos de carcaça também abrem a GUI quando a estrutura está formada); sem aba de alternância de modo de edição de baldes/contêineres (`ContainerEditMode` Both/Fill/Empty) | ABERTA |
| 20.2 | Dynamic Tank e Boiler: aceitam `mekanism:structural_glass` em qualquer face ou aresta da carcaça no port (no Java original, o vidro estrutural é permitido apenas nas faces/paredes, exigindo bloco de casing nas 12 arestas da moldura) | SUPOSIÇÃO |
| 20.3 | Thermoelectric Boiler: volume de água conta os blocos de ar contíguos sob o plano de dispersores (no Java original, a rotina `FormationProtocol.explore` inclui também os blocos de moldura do piso e paredes submersas) | SUPOSIÇÃO |
| 20.4 | Thermoelectric Boiler: sem renderização visual 3D dos fluidos internos no mundo quando usado vidro estrutural; fluidos exibidos na tela da válvula por índices de substância sem animação | LIMITE API |
| 20.5 | Thermal Evaporation Plant: aquecimento externo por condutores térmicos (`Thermodynamic Conductor`) ou aquecedor resistivo (`Resistive Heater`) acoplado às válvulas não suportado (rede de calor ainda não existe no port; aquecimento unicamente solar no topo) | LIMITE |
| 20.6 | Thermal Evaporation Plant: sem suporte a sinal analógico de redstone nas válvulas para automação por nível; sem efeitos sonoros de funcionamento | ABERTA |

# Integração do Grupo Calor

**ID da Tarefa**: T20261109-214921-2os  
**Grupo**: Calor (`heat`, `resistive_heater`, `fuelwood_heater`, `laser_tractor`)  
**Arquivos Rust novos criados**:
- `port/mod/src/heat.rs`
- `port/mod/src/resistive_heater.rs`
- `port/mod/src/fuelwood_heater.rs`
- `port/mod/src/laser_tractor.rs`

---

## 1. Trecho Exato para colar em `lib.rs`

### (a) Declarações de Módulos (no topo de `port/mod/src/lib.rs`, após linha 43)

Linha vizinha de contexto (linha 43 de `port/mod/src/lib.rs`):
```rust
mod worldgen;
```

Colar imediatamente abaixo:
```rust
mod heat;
mod resistive_heater;
mod fuelwood_heater;
mod laser_tractor;
```

---

### (b) Estrutura do Estado do Mod em `lib.rs`

Adicionar ao `struct Mekanism` em `lib.rs` os mapas de estado persistente dos blocos de calor:
```rust
    // Heat network and heater states
    pub heat_networks: HashMap<String, heat::HeatNetwork>,
    pub conductors: HashMap<(String, [i32; 3]), heat::ThermodynamicConductor>,
    pub resistive_heaters: HashMap<(String, [i32; 3]), resistive_heater::ResistiveHeater>,
    pub fuelwood_heaters: HashMap<(String, [i32; 3]), fuelwood_heater::FuelwoodHeater>,
    pub laser_tractors: HashMap<(String, [i32; 3]), laser_tractor::LaserTractorBeam>,
```

---

### (c) Chamadas no Loop de Tick (`run_all` / `tick` em `lib.rs`)

Localizar em `lib.rs` a chamada de `run_lasers` (em torno da linha 2482):
```rust
        let laser_parts: Vec<&BlockEntity> = entities
            .iter()
            .filter(|e| matches!(e.state.split('[').next(), Some(laser::LASER | laser::AMPLIFIER | fusion::MATRIX)))
            .collect();
        if !laser_parts.is_empty() {
            out.extend(self.run_lasers(&laser_parts));
        }
```

Colar imediatamente antes ou depois de `run_lasers`:
```rust
        // --- Heat Network & Conductors ---
        let conductor_entities: Vec<&BlockEntity> = entities
            .iter()
            .filter(|e| heat::ConductorTier::from_block(e.state.split('[').next().unwrap_or("")).is_some())
            .collect();
        if !conductor_entities.is_empty() {
            out.extend(self.run_heat_conductors(&conductor_entities, tick));
        }

        // --- Resistive Heaters ---
        let resistive_entities: Vec<&BlockEntity> = entities
            .iter()
            .filter(|e| e.state.split('[').next() == Some(resistive_heater::BLOCK))
            .collect();
        if !resistive_entities.is_empty() {
            out.extend(self.run_resistive_heaters(&resistive_entities, tick));
        }

        // --- Fuelwood Heaters ---
        let fuelwood_entities: Vec<&BlockEntity> = entities
            .iter()
            .filter(|e| e.state.split('[').next() == Some(fuelwood_heater::BLOCK))
            .collect();
        if !fuelwood_entities.is_empty() {
            out.extend(self.run_fuelwood_heaters(&fuelwood_entities, tick));
        }

        // --- Laser Tractor Beams ---
        let tractor_entities: Vec<&BlockEntity> = entities
            .iter()
            .filter(|e| e.state.split('[').next() == Some(laser_tractor::BLOCK))
            .collect();
        if !tractor_entities.is_empty() {
            out.extend(self.run_laser_tractors(&tractor_entities, tick));
        }
```

---

### (d) Funções Executoras a Adicionar na `impl Mekanism` em `lib.rs`

```rust
impl Mekanism {
    pub(crate) fn run_heat_conductors(&mut self, entities: &[&BlockEntity], _tick: u64) -> Vec<BlockEntityUpdate> {
        let mut updates = Vec::new();
        let mut world_conductors: HashMap<[i32; 3], heat::ThermodynamicConductor> = HashMap::new();
        let mut external_sinks: HashMap<[i32; 3], heat::HeatCapacitor> = HashMap::new();

        for e in entities {
            let at = [e.pos.x, e.pos.y, e.pos.z];
            let tier = heat::ConductorTier::from_block(e.state.split('[').next().unwrap_or("")).unwrap_or(heat::ConductorTier::Basic);
            let ambient = world::get_environment(&e.dimension, e.pos.x, e.pos.y, e.pos.z)
                .map_or(heat::AMBIENT_TEMP, |env| heat::ambient_temp(f64::from(env.temperature)));
            let conductor = heat::ThermodynamicConductor::decode(&e.data, ambient)
                .unwrap_or_else(|| heat::ThermodynamicConductor::new(tier, ambient));
            world_conductors.insert(at, conductor);
        }

        let mut network = heat::HeatNetwork::new();
        let default_ambient = heat::AMBIENT_TEMP;
        network.tick(&mut world_conductors, default_ambient, &mut external_sinks);

        for e in entities {
            let at = [e.pos.x, e.pos.y, e.pos.z];
            if let Some(cond) = world_conductors.get(&at) {
                let data = cond.encode();
                updates.push(BlockEntityUpdate {
                    dimension: e.dimension.clone(),
                    pos: e.pos,
                    state: None,
                    data: Some(data),
                    items: None,
                    energy: None,
                    tanks: None,
                    tank_faces: None,
                    menu_data: None,
                    transmitter_faces: Some(cond.connections.iter().map(|c| c.connects()).collect()),
                });
            }
        }
        updates
    }

    pub(crate) fn run_resistive_heaters(&mut self, entities: &[&BlockEntity], _tick: u64) -> Vec<BlockEntityUpdate> {
        let mut updates = Vec::new();
        for e in entities {
            let ambient = world::get_environment(&e.dimension, e.pos.x, e.pos.y, e.pos.z)
                .map_or(heat::AMBIENT_TEMP, |env| heat::ambient_temp(f64::from(env.temperature)));
            let mut heater = resistive_heater::ResistiveHeater::decode(&e.data, ambient)
                .unwrap_or_else(|| resistive_heater::ResistiveHeater::new(ambient));

            let redstone_power = world::get_redstone_power(&e.dimension, e.pos.x, e.pos.y, e.pos.z).unwrap_or(0) > 0;
            let mut sinks = Vec::new();
            let active = heater.tick(redstone_power, ambient, &mut sinks);

            updates.push(BlockEntityUpdate {
                dimension: e.dimension.clone(),
                pos: e.pos,
                state: None,
                data: Some(heater.encode()),
                items: None,
                energy: Some(heater.energy_stored_fe),
                tanks: None,
                tank_faces: None,
                menu_data: Some(heater.menu_data()),
                transmitter_faces: None,
            });
        }
        updates
    }

    pub(crate) fn run_fuelwood_heaters(&mut self, entities: &[&BlockEntity], _tick: u64) -> Vec<BlockEntityUpdate> {
        let mut updates = Vec::new();
        for e in entities {
            let ambient = world::get_environment(&e.dimension, e.pos.x, e.pos.y, e.pos.z)
                .map_or(heat::AMBIENT_TEMP, |env| heat::ambient_temp(f64::from(env.temperature)));
            let mut heater = fuelwood_heater::FuelwoodHeater::decode(&e.data, ambient)
                .unwrap_or_else(|| fuelwood_heater::FuelwoodHeater::new(ambient));

            let mut sinks = Vec::new();
            heater.tick(ambient, &mut sinks);

            updates.push(BlockEntityUpdate {
                dimension: e.dimension.clone(),
                pos: e.pos,
                state: None,
                data: Some(heater.encode()),
                items: None,
                energy: None,
                tanks: None,
                tank_faces: None,
                menu_data: Some(heater.menu_data()),
                transmitter_faces: None,
            });
        }
        updates
    }

    pub(crate) fn run_laser_tractors(&mut self, entities: &[&BlockEntity], _tick: u64) -> Vec<BlockEntityUpdate> {
        let mut updates = Vec::new();
        for e in entities {
            let at = [e.pos.x, e.pos.y, e.pos.z];
            let mut tractor = laser_tractor::LaserTractorBeam::decode(&e.data)
                .unwrap_or_else(|| laser_tractor::LaserTractorBeam::new(2));

            // Inflow from laser beams striking this tractor beam
            if let Some(inflow) = self.beams.remove(&(e.dimension.clone(), at)) {
                tractor.receive_laser_energy(inflow);
            }

            updates.push(BlockEntityUpdate {
                dimension: e.dimension.clone(),
                pos: e.pos,
                state: None,
                data: Some(tractor.encode()),
                items: None,
                energy: Some(tractor.energy_fe),
                tanks: None,
                tank_faces: None,
                menu_data: None,
                transmitter_faces: None,
            });
        }
        updates
    }
}
```

---

## 2. Alterações no `port/tools/convert.py`

### (a) Remover condutores de `ITEM_ONLY_BLOCKS` (linha 199 de `convert.py`)

No `convert.py`, substituir:
```python
ITEM_ONLY_BLOCKS = {f'{t}_thermodynamic_conductor' for t in ('basic', 'advanced', 'elite', 'ultimate')} | {'diversion_transporter', 'restrictive_transporter', 'personal_chest', 'seismic_vibrator'}
```
Por:
```python
ITEM_ONLY_BLOCKS = {'diversion_transporter', 'restrictive_transporter', 'personal_chest', 'seismic_vibrator'}
```

### (b) Registrar Novos Transmissores e Máquinas em `convert.py`

Adicionar à lista `TRANSMITTERS`:
```python
TRANSMITTERS.append(('thermodynamic_conductor', 'heat'))
```

Registrar os novos blocos:
```python
HEAT_MACHINES = ['resistive_heater', 'fuelwood_heater', 'laser_tractor_beam']
for m in HEAT_MACHINES:
    BLOCKS[m] = ('minecraft:furnace', {'hardness': 3.5, 'explosion_resistance': 16.0, 'requires_tool': True}, 'machine')
OPEN_MACHINES |= set(HEAT_MACHINES)
```

Adicionar às tags de conexão de energia (`energy_connectable.json`):
```python
    f'{NS}:resistive_heater',
    f'{NS}:laser_tractor_beam',
```

Adicionar tag `heat_connectable.json`:
```python
put(f'{OUT}/data/{NS}/tags/block/heat_connectable.json', {'values': [
    *[f'{NS}:{t}_thermodynamic_conductor' for t in ('basic', 'advanced', 'elite', 'ultimate')],
    f'{NS}:resistive_heater',
    f'{NS}:fuelwood_heater',
    f'{NS}:thermal_evaporation_valve',
    f'{NS}:boiler_valve',
    f'{NS}:fission_reactor_port',
]})
```

---

## 3. Alterações em `horizon-mod.toml`

Em `port/mod/horizon-mod.toml`:
Se for necessária nova capability declarada no mod:
```toml
capabilities = ["registries", "players", "world", "recipe-types", "chunks", "storage", "player-control", "player-items", "entities"]
```
Adicionar `"transmitters"` ou `"heat"` quando suportado pelo fork.

---

## 4. Integração do Calor Próprio: Evaporator, Boiler e Fission

### 4.1 Evaporator (`evaporation.rs`) — Aquecimento Externo por Condutores e Aquecedores

**Comportamento no Java**:
- A `TileEntityThermalEvaporationValve` implementa `IHeatHandler` compartilhando o `heatCapacitor` da planta inteira (`getMultiblock().getHeatCapacitors(side)`).
- Quando condutores térmicos (`ThermodynamicConductor`) ou aquecedores (`ResistiveHeater`) são encostados na válvula, eles transferem calor diretamente para a planta via condução.
- No Rust atual (`evaporation.rs`), o cálculo considera apenas aquecimento solar:
  ```rust
  plant.temperature = heat(plant.temperature.max(1.0), solars, ambient);
  ```

**Patch Exato para Conectar a Válvula de Evaporação à Rede Térmica**:
Em `evaporation.rs`:
```rust
/// Aquecimento externo injetado por condutores térmicos ou aquecedores resistivos adjacentes.
/// Java: `EvaporationMultiblockData.java:137, 155`.
pub fn apply_external_heat(temperature: f64, external_heat_joules: f64, height: u8) -> f64 {
    let casing_heat_capacity = 100.0 * f64::from(height); // 100 J/K por nível de altura
    temperature + (external_heat_joules / casing_heat_capacity)
}
```
No tick de `evaporation.rs` (onde as válvulas são processadas):
```rust
let mut external_heat = 0.0;
for valve_pos in &found.valves {
    for face in 0..6 {
        let neighbor = [
            valve_pos[0] + direction_offset(face)[0],
            valve_pos[1] + direction_offset(face)[1],
            valve_pos[2] + direction_offset(face)[2],
        ];
        // Coleta calor transferido de condutores ou aquecedores encostados na válvula
        if let Some(heat_inflow) = self.conductors.get_mut(&(dim.clone(), neighbor)) {
            let delta = (heat_inflow.capacitor.temperature() - plant.temperature) / (heat_inflow.tier.inverse_conduction() + 5.0);
            if delta > 0.0 {
                let q = delta * heat_inflow.tier.heat_capacity();
                heat_inflow.capacitor.handle_heat(-q);
                external_heat += q;
            }
        }
    }
}
plant.temperature = apply_external_heat(plant.temperature, external_heat, found.height);
```

### 4.2 Boiler (`boiler.rs`) — Conexão à Rede de Calor
- `boiler::VALVE` atua como interface térmica da estrutura (`CASING_HEAT_CAPACITY = 50.0` por bloco de carcaça).
- Válvulas conectadas a condutores térmicos permitem que o calor seja fornecido à caldeira sem necessidade de refrigerante quente (`superheated_sodium`), aquecendo a caldeira diretamente via calor de condutor ou aquecedor resistivo para fervura de água em vapor (`ENTHALPY / EFFICIENCY`).

### 4.3 Fission Reactor (`fission.rs`) — Dissipação e Extração Térmica
- `fission::PORT` atua como emissor térmico de altíssima temperatura.
- Conectar condutores Ultimate (`ultimate_thermodynamic_conductor`) às portas do reator de fissão permite exportar calor excedente para prevenir fusões (*meltdown* em 1500 K) e encaminhar esse calor diretamente para aquecer o evaporador ou a caldeira industrial.

#!/usr/bin/env python3 -I
"""Converts Mekanism's data and assets (MIT, github.com/mekanism/Mekanism, branch 26.3)
into the content of the Horizon port: blocks and items declared as Horizon content, and
the assets, loot tables, recipes, tags and lang the port uses, copied or lightly rewritten.

Usage: convert.py <mekanism checkout> <output mod folder>
"""
import glob, json, os, re, shutil, sys

SRC, OUT = sys.argv[1], sys.argv[2]
NS = 'mekanism'
GEN = f'{SRC}/src/datagen/generated/mekanism'
RES = f'{SRC}/src/main/resources'
ASSET_DIRS = [f'{GEN}/assets/{NS}', f'{RES}/assets/{NS}', os.path.join(os.path.dirname(os.path.abspath(__file__)), 'gui-sprites')]  # last: the GUI sprites of 26.3 (1.21.1 keeps the same art in atlases cut by code)
DATA_DIRS = [f'{GEN}/data/{NS}', f'{RES}/data/{NS}']

def find(rel, dirs):
    for d in dirs:
        p = f'{d}/{rel}'
        if os.path.exists(p):
            return p
    return None

def read_json(p):
    return json.load(open(p))

def put(path, data=None, src=None):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    if src is not None:
        shutil.copyfile(src, path)
    else:
        with open(path, 'w') as f:
            f.write(data if isinstance(data, str) else json.dumps(data, indent=2))

warnings = []
copied_textures = set()
copied_models = set()

def copy_texture(ref):
    """`mekanism:block/x` -> assets/mekanism/textures/block/x.png (and .mcmeta)."""
    if not ref.startswith(NS + ':'):
        return
    rel = ref.split(':', 1)[1]
    if rel in copied_textures:
        return
    copied_textures.add(rel)
    p = find(f'textures/{rel}.png', ASSET_DIRS)
    if not p:
        warnings.append(f'missing texture {ref}')
        return
    put(f'{OUT}/assets/{NS}/textures/{rel}.png', src=p)
    if os.path.exists(p + '.mcmeta'):
        put(f'{OUT}/assets/{NS}/textures/{rel}.png.mcmeta', src=p + '.mcmeta')

def copy_model(ref):
    """Copies a model and its parents and textures; false when something is missing."""
    if not ref.startswith(NS + ':'):
        return True
    rel = ref.split(':', 1)[1]
    if rel in copied_models:
        return True
    copied_models.add(rel)
    p = find(f'models/{rel}.json', ASSET_DIRS)
    if not p:
        warnings.append(f'missing model {ref}')
        return False
    m = read_json(p)
    ok = True
    if m.get('loader') == 'neoforge:composite':
        # Horizon has no composite loader: the children's elements and textures become one model (the factories' bases and LEDs)
        textures, elements = dict(m.get('textures', {})), []
        def resolve(child):
            nonlocal textures, elements
            chain, cur = [], child
            while cur is not None:
                chain.append(cur)
                parent = cur.get('parent', '')
                if parent.startswith(NS + ':'):
                    pp = find(f'models/{parent.split(":", 1)[1]}.json', ASSET_DIRS)
                    cur = read_json(pp) if pp else None
                else:
                    cur = None
            for part in reversed(chain):
                textures.update(part.get('textures', {}))
            # the nearest model (child first) that has elements
            for part in chain:
                if 'elements' in part:
                    elements = elements + part['elements']
                    break
        for child in m['children'].values():
            resolve(child)
        m = {k: v for k, v in m.items() if k not in ('loader', 'children')}
        m['textures'], m['elements'] = textures, elements
        m['render_type'] = 'minecraft:cutout'
    if 'parent' in m:
        ok &= copy_model(m['parent'])
    for v in m.get('textures', {}).values():
        if isinstance(v, str) and not v.startswith('#'):
            copy_texture(v)
    put(f'{OUT}/assets/{NS}/models/{rel}.json', json.dumps(m, indent=2))
    return ok

def models_of_blockstate(bs):
    out = []
    def visit(v):
        if isinstance(v, dict):
            if 'model' in v and isinstance(v['model'], str):
                out.append(v['model'])
            for x in v.values():
                visit(x)
        elif isinstance(v, list):
            for x in v:
                visit(x)
    visit(bs)
    return out

# ---------------------------------------------------------------- what the port has so far
ORES = ['osmium', 'tin', 'lead', 'uranium', 'fluorite']
STORAGE = ['block_osmium', 'block_tin', 'block_lead', 'block_uranium', 'block_steel', 'block_bronze',
           'block_charcoal', 'block_fluorite', 'block_salt', 'block_refined_glowstone',
           'block_refined_obsidian', 'block_raw_osmium', 'block_raw_tin', 'block_raw_lead',
           'block_raw_uranium', 'block_bio_fuel']
PROCESS_MACHINES = ['metallurgic_infuser', 'purification_chamber', 'chemical_injection_chamber', 'osmium_compressor',
                    'chemical_oxidizer', 'chemical_dissolution_chamber', 'chemical_crystallizer', 'pigment_extractor',
                    'painting_machine', 'nutritional_liquifier', 'antiprotonic_nucleosynthesizer']
CONTINUOUS_MACHINES = ['chemical_infuser', 'chemical_washer', 'electrolytic_separator', 'rotary_condensentrator',
                       'pressurized_reaction_chamber', 'pigment_mixer', 'isotopic_centrifuge', 'solar_neutron_activator']
# MekanismGenerators' namespace and the generators the port has (their assets come from tools/generators-assets)
# (a mod declares its blocks in its own namespace only, so they are `mekanism:bio_generator`... and their assets live in mekanism's)
GNS = 'mekanism'
GSRC = 'mekanismgenerators'
GNS_SRC = 'mekanismgenerators'
GENERATORS = ['heat_generator', 'bio_generator', 'solar_generator', 'gas_burning_generator', 'advanced_solar_generator', 'wind_generator']
FISSION = ['fission_reactor_casing', 'fission_reactor_port', 'fission_reactor_logic_adapter', 'fission_fuel_assembly', 'control_rod_assembly']
FUSION = ['fusion_reactor_controller', 'fusion_reactor_port', 'fusion_reactor_frame', 'fusion_reactor_logic_adapter', 'laser_focus_matrix', 'reactor_glass']
TURBINE = ['turbine_casing', 'turbine_valve', 'turbine_vent', 'turbine_rotor', 'rotational_complex', 'electromagnetic_coil', 'saturating_condenser']
TIERS = ['basic', 'advanced', 'elite', 'ultimate']
FLUID_TANKS = [f'{t}_fluid_tank' for t in TIERS + ['creative']]
CHEM_TANKS = [f'{t}_chemical_tank' for t in TIERS + ['creative']]
# the factories: `FactoryTier.processes` lines each, of the electric machines' work
FACTORY_LINES = {'basic': 3, 'advanced': 5, 'elite': 7, 'ultimate': 9}
FACTORY_KINDS = {'crushing': 'crusher', 'enriching': 'enrichment_chamber', 'smelting': 'energized_smelter', 'combining': 'combiner',
                 'sawing': 'precision_sawmill', 'compressing': 'osmium_compressor', 'infusing': 'metallurgic_infuser',
                 'injecting': 'chemical_injection_chamber', 'purifying': 'purification_chamber'}
# the slot form of each: 0 an input and an output a line, 1 plus an extra input the lines share, 2 two outputs a line
FACTORY_FORM = {'crushing': 0, 'enriching': 0, 'smelting': 0, 'combining': 1, 'sawing': 2, 'compressing': 4, 'infusing': 4,
                'injecting': 3, 'purifying': 3}
# the chemical factories' tanks: what their machine's holds for each line (`MAX_GAS`, `MAX_INFUSE`)
FACTORY_TANK = {'compressing': 210, 'infusing': 1000, 'injecting': 210, 'purifying': 210}
FACTORIES = {f'{t}_{k}_factory': (t, k) for t in FACTORY_LINES for k in FACTORY_KINDS}
MACHINES = ['crusher', 'enrichment_chamber', 'energized_smelter', 'precision_sawmill', 'combiner'] + PROCESS_MACHINES \
    + CONTINUOUS_MACHINES + FLUID_TANKS + CHEM_TANKS + list(FACTORIES) + ['electric_pump', 'fluidic_plenisher']
BLOCKS = {}  # name -> (template, properties, kind)
for o in ORES:
    BLOCKS[f'{o}_ore'] = ('minecraft:iron_ore', {'hardness': 3.0, 'explosion_resistance': 3.0, 'requires_tool': True}, 'ore')
    BLOCKS[f'deepslate_{o}_ore'] = ('minecraft:deepslate_iron_ore', {'hardness': 4.5, 'explosion_resistance': 3.0, 'requires_tool': True}, 'ore')
for s in STORAGE:
    BLOCKS[s] = ('minecraft:iron_block', {'hardness': 5.0, 'explosion_resistance': 6.0, 'requires_tool': True}, 'storage')
BLOCKS['steel_casing'] = ('minecraft:iron_block', {'hardness': 3.5, 'explosion_resistance': 9.0, 'requires_tool': True}, 'storage')
for m in MACHINES:
    BLOCKS[m] = ('minecraft:furnace', {'hardness': 3.5, 'explosion_resistance': 16.0, 'requires_tool': True}, 'machine')
# The infuser's model has gaps, so it is not a furnace (which hides the faces of the blocks beside it): a beacon is a full
# shape that does not occlude, and the facing and lit states the furnace had it declares itself (the mod turns it to face
# the player that places it, as the furnace does).
OPEN_MACHINES = {'metallurgic_infuser', 'chemical_oxidizer', 'chemical_dissolution_chamber', 'chemical_crystallizer',
                 'pigment_extractor', 'painting_machine', 'nutritional_liquifier', 'antiprotonic_nucleosynthesizer',
                 *CONTINUOUS_MACHINES, *FLUID_TANKS, *CHEM_TANKS, *FACTORIES, 'electric_pump', 'fluidic_plenisher'}
for m in OPEN_MACHINES:
    BLOCKS[m] = ('minecraft:beacon', BLOCKS[m][1], 'machine')

INDUCTION = ['induction_casing', 'induction_port', *[f'{t}_induction_cell' for t in TIERS], *[f'{t}_induction_provider' for t in TIERS]]
EVAPORATION = ['thermal_evaporation_block', 'thermal_evaporation_valve', 'thermal_evaporation_controller']
DYNAMIC = ['dynamic_tank', 'dynamic_valve', *INDUCTION, *EVAPORATION]
BINS = [f'{t}_bin' for t in ('basic', 'advanced', 'elite', 'ultimate', 'creative')]
LASERS = ['digital_miner', 'industrial_alarm', 'laser', 'laser_amplifier', 'supercharged_coil', 'logistical_sorter', 'quantum_entangloporter']
SPS = ['sps_casing', 'sps_port']
BOILER = ['boiler_casing', 'boiler_valve', 'pressure_disperser', 'superheating_element']
TELEPORT = ['teleporter', 'teleporter_frame']
DYNAMIC += BOILER + SPS + TELEPORT
BLOCKS['structural_glass'] = ('minecraft:beacon', {'hardness': 3.5, 'explosion_resistance': 9.0, 'requires_tool': True}, 'storage')
for m in BINS:
    BLOCKS[m] = ('minecraft:beacon', {'hardness': 3.5, 'explosion_resistance': 16.0, 'requires_tool': True}, 'machine')
OPEN_MACHINES |= set(BINS)
for m in LASERS:
    BLOCKS[m] = ('minecraft:beacon', {'hardness': 3.5, 'explosion_resistance': 16.0, 'requires_tool': True}, 'machine')
OPEN_MACHINES |= set(LASERS)
# Heat: the conductors only keep their heat for now (the heat medium of the fork carries it later); the heaters take energy or fuel.
HEAT_CONDUCTORS = [f'{t}_thermodynamic_conductor' for t in ('basic', 'advanced', 'elite', 'ultimate')]
HEAT_MACHINES = ['resistive_heater', 'fuelwood_heater']
for m in HEAT_CONDUCTORS + HEAT_MACHINES:
    BLOCKS[m] = ('minecraft:beacon', {'hardness': 3.5, 'explosion_resistance': 16.0, 'requires_tool': True}, 'machine')
OPEN_MACHINES |= set(HEAT_CONDUCTORS + HEAT_MACHINES)
# Utilities: machines with a screen or a charge slot, the personal storages, the cardboard box (hardness/resistance of the original blocks).
UTILITY_MACHINES = ['formulaic_assemblicator', 'chargepad', 'dimensional_stabilizer', 'seismic_vibrator', 'oredictionificator', 'personal_chest', 'personal_barrel']
for m in UTILITY_MACHINES:
    BLOCKS[m] = ('minecraft:beacon', {'hardness': 3.5, 'explosion_resistance': 16.0, 'requires_tool': True}, 'machine')
BLOCKS['cardboard_box'] = ('minecraft:beacon', {'hardness': 0.5, 'explosion_resistance': 0.6, 'requires_tool': False}, 'machine')
OPEN_MACHINES |= set(UTILITY_MACHINES + ['cardboard_box'])
for m in DYNAMIC:
    BLOCKS[m] = ('minecraft:furnace', {'hardness': 3.5, 'explosion_resistance': 9.0, 'requires_tool': True}, 'machine')
TRANSMITTERS = [('universal_cable', 'energy'), ('pressurized_tube', 'chemical'), ('mechanical_pipe', 'fluid'),
                ('logistical_transporter', 'item')]
TRANSMITTER_BLOCKS = [f'{tier}_{kind}' for kind, _ in TRANSMITTERS for tier in ('basic', 'advanced', 'elite', 'ultimate')]
CUBE_TIERS = ['basic', 'advanced', 'elite', 'ultimate', 'creative']
CUSTOM_BLOCKS = [*TRANSMITTER_BLOCKS, *[f'{t}_energy_cube' for t in CUBE_TIERS]]
# ---------------------------------------------------------------- items: every plain one
items = {}
# 1.21.1: the item definitions are made from the item models (base_1211.py), which also hold the variants of an item
# (the geiger counter's levels, the bow's pulling frames...) and block models: only what the lang names as an item is one
# blocks the port has only as items so far (their blocks are not made yet): the heat conductors, two transporters, the chest, the vibrator
ITEM_ONLY_BLOCKS = {'diversion_transporter', 'restrictive_transporter'}
ITEM_LANG = {k[len('item.mekanism.'):] for k in read_json(f'{GEN}/assets/{NS}/lang/en_us.json') if k.startswith('item.mekanism.')}
for p in sorted(glob.glob(f'{GEN}/assets/{NS}/items/*.json')):
    n = os.path.basename(p)[:-5]
    if n not in ITEM_LANG and n not in ITEM_ONLY_BLOCKS:
        continue
    if n in BLOCKS or n in CUSTOM_BLOCKS or n.endswith('_bucket'):
        continue
    d = read_json(p)
    model = d.get('model', {})
    if n == 'configurator':
        # Its model picks by a component of Mekanism's own (the tool's mode): the original's models/item/configurator.json
        # has overrides on the `mekanism:mode` predicate, one model each for configuring, empty, rotate and wrench. The
        # engine draws one model per item and there is no way to register a component-based model selector, so the port
        # keeps the one configuring model; the mode itself lives in the item's components and the cycle message says it.
        model = {'type': 'minecraft:model', 'model': f'{NS}:item/configurator'}
    if model.get('type') != 'minecraft:model':
        continue  # a special item model the engine can't draw (jetpack, tablet...)
    if model['model'].startswith(NS + ':block/'):
        continue  # the item of a block the port doesn't have yet
    items[n] = model['model']

# ---------------------------------------------------------------- write
shutil.rmtree(f'{OUT}/data', ignore_errors=True)
shutil.rmtree(f'{OUT}/assets', ignore_errors=True)

# the armor of the mod and the slot it is worn in (the DataComponents.EQUIPPABLE of its item classes)
WORN = {'mekasuit_helmet': 'head', 'mekasuit_bodyarmor': 'chest', 'mekasuit_pants': 'legs', 'mekasuit_boots': 'feet',
        'hazmat_mask': 'head', 'hazmat_gown': 'chest', 'hazmat_pants': 'legs', 'hazmat_boots': 'feet',
        'jetpack': 'chest', 'jetpack_armored': 'chest', 'free_runners': 'feet', 'free_runners_armored': 'feet',
        'scuba_mask': 'head', 'scuba_tank': 'chest'}
kept_items = []
for n, model in items.items():
    if not copy_model(model):
        continue
    kept_items.append(n)
    item = {'template': 'minecraft:stick'}
    if n in WORN:
        # Equippable(slot): a player wears it in its armor slot, one at a time (the worn model is still to do)
        item['max_stack_size'] = 1
        item['components'] = {'minecraft:equippable': {'slot': WORN[n]}}
    put(f'{OUT}/data/{NS}/horizon/item/{n}.json', item)
    put(f'{OUT}/assets/{NS}/items/{n}.json', {'model': {'type': 'minecraft:model', 'model': model}})

kept_blocks = []
for n, (template, props, kind) in sorted(BLOCKS.items()):
    p = find(f'blockstates/{n}.json', ASSET_DIRS)
    if not p:
        warnings.append(f'no blockstate for {n}')
        continue
    bs = read_json(p)
    if n == 'boiler_valve':
        # the valve's mode (input, steam out, coolant out) is not a state of the port's valve: it shows the input one
        bs = {'variants': {'': bs['variants']['mode=input']}}
    text = json.dumps(bs)
    if kind == 'machine':
        text = text.replace('active=', 'lit=')
        bs = json.loads(text)
        # a machine with no active state in the original (the metallurgic infuser) gets both of the port's lit states
        if 'variants' in bs and not any('lit=' in k for k in bs['variants']):
            bs['variants'] = {f'{k},lit={lit}': v for k, v in bs['variants'].items() for lit in ('false', 'true')}
    if not all(copy_model(m) for m in models_of_blockstate(bs)):
        continue
    kept_blocks.append(n)
    block = {'template': template, 'properties': props, 'item': True}
    if n in OPEN_MACHINES:
        block['state_properties'] = {'lit': {'type': 'boolean'}}
        if n not in FLUID_TANKS:
            block['state_properties']['facing'] = {'type': 'enum', 'values': ['north', 'east', 'south', 'west'] + (['up', 'down'] if n in LASERS else [])}
    put(f'{OUT}/data/{NS}/horizon/block/{n}.json', block)
    put(f'{OUT}/assets/{NS}/blockstates/{n}.json', bs)
    ip = find(f'items/{n}.json', ASSET_DIRS)
    if ip:
        d = read_json(ip)
        if d.get('model', {}).get('type') == 'minecraft:model':
            copy_model(d['model']['model'])
            put(f'{OUT}/assets/{NS}/items/{n}.json', d)
        elif d.get('model', {}).get('type') == 'minecraft:special' and d['model'].get('base') and copy_model(d['model']['base']):
            # the fluid tank's item: its base model only has the display transforms (the original draws the tank and its
            # fluid with a special renderer), so the item shows the tank's block model with those transforms
            base = d['model']['base']
            bm = read_json(f'{OUT}/assets/{NS}/models/{base.split(":")[1]}.json')
            shown = f'{NS}:block/fluid_tank'
            if copy_model(shown):
                bm['parent'] = shown
                put(f'{OUT}/assets/{NS}/models/{base.split(":")[1]}.json', bm)
            put(f'{OUT}/assets/{NS}/items/{n}.json', {'model': {'type': 'minecraft:model', 'model': base}})
    # loot: ores and storage as Mekanism has them; machines drop themselves (keeping their data)
    if kind == 'machine':
        loot = {'type': 'minecraft:block', 'pools': [{'rolls': 1, 'entries': [{'type': 'minecraft:item', 'name': f'{NS}:{n}'}],
                'conditions': [{'type': 'minecraft:survives_explosion'}]}]}
        put(f'{OUT}/data/{NS}/loot_table/blocks/{n}.json', loot)
    else:
        lp = find(f'loot_table/blocks/{n}.json', DATA_DIRS)
        if lp:
            put(f'{OUT}/data/{NS}/loot_table/blocks/{n}.json', src=lp)


# ---------------------------------------------------------------- hand-made blocks
# Mekanism draws its cables and energy cubes with its own model loaders, which Horizon does not have:
# these get plain JSON models of their own, from Mekanism's textures.
def self_drop(n):
    return {'type': 'minecraft:block', 'pools': [{'rolls': 1, 'entries': [{'type': 'minecraft:item', 'name': f'{NS}:{n}'}],
            'conditions': [{'type': 'minecraft:survives_explosion'}]}]}

def add_block(n, template, props, extra=None):
    block = {'template': template, 'properties': props, 'item': True}
    block.update(extra or {})
    put(f'{OUT}/data/{NS}/horizon/block/{n}.json', block)
    put(f'{OUT}/data/{NS}/loot_table/blocks/{n}.json', self_drop(n))
    kept_blocks.append(n)

# cables, tubes and pipes: a core and an arm, on every side a connection says (one block for each tier)
for kind, medium in TRANSMITTERS:
    for tier in ('basic', 'advanced', 'elite', 'ultimate'):
        n = f'{tier}_{kind}'
        tex = f'{NS}:block/models/multipart/{n}'
        copy_texture(tex)
        copy_texture(tex + '_vertical')
        put(f'{OUT}/assets/{NS}/models/block/{n}_core.json', {
            'textures': {'t': tex, 'particle': tex},
            'elements': [{'from': [5, 5, 5], 'to': [11, 11, 11], 'faces': {d: {'texture': '#t'} for d in ('down', 'up', 'north', 'south', 'west', 'east')}}]})
        put(f'{OUT}/assets/{NS}/models/block/{n}_arm.json', {
            'textures': {'t': tex + '_vertical', 'particle': tex},
            'elements': [{'from': [5, 5, 0], 'to': [11, 11, 5], 'faces': {d: {'texture': '#t'} for d in ('down', 'up', 'north', 'west', 'east')}}]})
        arm = f'{NS}:block/{n}_arm'
        put(f'{OUT}/assets/{NS}/blockstates/{n}.json', {'multipart': [
            {'apply': {'model': f'{NS}:block/{n}_core'}},
            {'when': {'north': 'true'}, 'apply': {'model': arm}},
            {'when': {'east': 'true'}, 'apply': {'model': arm, 'y': 90}},
            {'when': {'south': 'true'}, 'apply': {'model': arm, 'y': 180}},
            {'when': {'west': 'true'}, 'apply': {'model': arm, 'y': 270}},
            {'when': {'up': 'true'}, 'apply': {'model': arm, 'x': 270}},
            {'when': {'down': 'true'}, 'apply': {'model': arm, 'x': 90}}]})
        # the item: a straight piece (the core with an arm at both ends), as the original's inventory icons are
        put(f'{OUT}/assets/{NS}/models/block/{n}_item.json', {
            'parent': 'minecraft:block/block', 'textures': {'t': tex, 'tv': tex + '_vertical', 'particle': tex},
            'elements': [
                {'from': [5, 5, 5], 'to': [11, 11, 11], 'faces': {d: {'texture': '#t'} for d in ('down', 'up', 'north', 'south', 'west', 'east')}},
                {'from': [5, 5, 0], 'to': [11, 11, 5], 'faces': {d: {'texture': '#tv'} for d in ('down', 'up', 'north', 'west', 'east')}},
                {'from': [5, 5, 11], 'to': [11, 11, 16], 'faces': {d: {'texture': '#tv'} for d in ('down', 'up', 'south', 'west', 'east')}}]})
        put(f'{OUT}/assets/{NS}/items/{n}.json', {'model': {'type': 'minecraft:model', 'model': f'{NS}:block/{n}_item'}})
        add_block(n, 'minecraft:chorus_plant', {'hardness': 0.5, 'explosion_resistance': 6.0},
                  {'connects_to': f'#{NS}:{medium}_connectable', 'thickness': 6.0})

# energy cubes: Mekanism's own frame (its model file lists the elements; only the loader is
# its own), around a core, the same on every face
base = read_json(find('models/block/energy_cube/base.json', ASSET_DIRS))
def tinted_core(tier):
    """Mekanism draws the white core texture tinted with the tier's colour (a tint index): the
    colour of the tier's corner lights, here baked into a copy of the texture."""
    from PIL import Image
    corner = Image.open(find(f'textures/block/models/energy_cube_{tier}_corner.png', ASSET_DIRS)).convert('RGBA')
    best = max((p for p in corner.getdata() if p[3] > 200), key=lambda p: max(p[:3]) - min(p[:3]) + max(p[:3]) / 4)
    core = Image.open(find('textures/block/energy_core.png', ASSET_DIRS) or find('render/energy_core.png', ASSET_DIRS)).convert('RGBA')
    px = core.load()
    for x in range(core.width):
        for y in range(core.height):
            r, g, b, a = px[x, y]
            px[x, y] = (r * best[0] // 255, g * best[1] // 255, b * best[2] // 255, a)
    path = f'{OUT}/assets/{NS}/textures/block/energy_core_{tier}.png'
    os.makedirs(os.path.dirname(path), exist_ok=True)
    core.save(path)
    return f'{NS}:block/energy_core_{tier}'

for tier in CUBE_TIERS:
    textures = dict(base['textures'])
    textures['corner'] = f'{NS}:block/models/energy_cube_{tier}_corner'
    textures['core'] = tinted_core(tier)
    for v in textures.values():
        if not v.startswith(f'{NS}:block/energy_core_'):
            copy_texture(v)
    elements = list(base['frame'])
    # Ports and LEDs. The default side configuration of Mekanism's cube (FRONT_OUT_EJECT) is the front an
    # output (ACTIVE_LIT: lit LEDs, shifted uvs, full light) and the other five sides inputs (ACTIVE_UNLIT),
    # so every side has its port and its LEDs.
    shift_u, shift_v = base['leds']['lit_uv_shift']
    def plain(e, lit=False):
        e = {k: v for k, v in e.items() if k != 'shade_direction_override'}
        if lit:
            e['light_emission'] = 15
            e['faces'] = {d: dict(f, uv=[f['uv'][0] + shift_u, f['uv'][1] + shift_v,
                                         f['uv'][2] + shift_u, f['uv'][3] + shift_v]) if 'uv' in f else f
                          for d, f in e['faces'].items()}
        return e
    SIDES = ('front', 'back', 'left', 'right', 'top', 'bottom')
    FACES = ('down', 'up', 'north', 'south', 'west', 'east')
    FACE_MODEL = {'down': 'bottom', 'up': 'top', 'north': 'front', 'south': 'back', 'west': 'right', 'east': 'left'}
    for side in ('bottom', 'top', 'front', 'back', 'right', 'left'):
        elements += [plain(e, lit=side == 'front') for e in base['leds'][side]]
        elements += [plain(e) for e in base['ports'][side]]
    # The block: the frame, and for each side the part its state says (off: LEDs only; in: LEDs and port;
    # out: lit LEDs and port), as the original's EnergyCubeModel does. The item has the default configuration.
    def side_model(side, state):
        els = [plain(e, lit=state == 'out') for e in base['leds'][side]]
        if state != 'off':
            els += [plain(e) for e in base['ports'][side]]
        return els
    for side in SIDES:
        for state in ('off', 'in', 'out'):
            put(f'{OUT}/assets/{NS}/models/block/energy_cube_{tier}_{side}_{state}.json',
                {'parent': 'minecraft:block/block', 'textures': textures, 'elements': side_model(side, state)})
    put(f'{OUT}/assets/{NS}/models/block/energy_cube_{tier}_frame.json',
        {'parent': 'minecraft:block/block', 'textures': textures, 'elements': list(base['frame'])})
    n = f'{tier}_energy_cube'
    # The block is its frame alone, one state. The ports and LEDs are parts the renderer draws by the bytes of
    # the cube's data (one for each face of the world: 0 off, 1 input, 2 output, 3 both), as the original
    # draws them from the entity's model data; a part is the model of a block of the mod's own, never placed.
    part_elements = []
    for index, face in enumerate(FACES):
        for state, values in (('off', [0]), ('in', [1]), ('out', [2, 3])):
            part = f'energy_cube_part_{tier}_{face}_{state}'
            put(f'{OUT}/data/{NS}/horizon/block/{part}.json',
                {'template': 'minecraft:stone', 'properties': {'hardness': 1.0}, 'item': False})
            put(f'{OUT}/assets/{NS}/blockstates/{part}.json',
                {'variants': {'': {'model': f'{NS}:block/energy_cube_{tier}_{FACE_MODEL[face]}_{state}'}}})
            part_elements.append({'type': 'part', 'block': f'{NS}:{part}', 'when': {'byte': index, 'values': values}})
    put(f'{OUT}/data/{NS}/horizon/renderer/{tier}_energy_cube.json', {'elements': part_elements + [
        # RenderEnergyCube: scale 0.4 of a 16-unit cube, bounce sin(3 deg * t) / 7, a turn about Y at 4 deg a
        # tick and one about (0, 1, 1) from 36 deg at 4 deg a tick, full bright, only with energy.
        {'type': 'animated', 'from': [0, 0, 0], 'to': [16, 16, 16], 'sprite': textures['core'],
         'scale': 0.4, 'center': [8, 8, 8],
         'bounce': {'amplitude': 1 / 7, 'degrees_per_tick': 3.0},
         'rotations': [{'axis': [0, 1, 0], 'degrees_per_tick': 4.0},
                       {'axis': [0, 1, 1], 'degrees_per_tick': 4.0, 'offset': 36.0}],
         'fullbright': True, 'only_with_energy': True, 'energy_fade': True}]})
    put(f'{OUT}/assets/{NS}/models/block/energy_cube_{tier}.json',
        {'parent': 'minecraft:block/block', 'textures': textures, 'elements': elements})
    put(f'{OUT}/assets/{NS}/blockstates/{n}.json', {'variants': {'': {'model': f'{NS}:block/energy_cube_{tier}_frame'}}})
    put(f'{OUT}/assets/{NS}/items/{n}.json', {'model': {'type': 'minecraft:model', 'model': f'{NS}:block/energy_cube_{tier}'}})
    # A beacon has one state, a full cube shape and does not occlude (it is `noOcclusion`), so the cube's open
    # frame does not hide the blocks beside it (a furnace's shape does) and the block has few states: the game keeps
    # a row of tables for each state, and a cube with a facing, a waterlogged state and a state for each side's look
    # made it 11,000 (and the phone gave up). An enchanting table's bottom hid the block below it.
    add_block(n, 'minecraft:beacon', {'hardness': 3.5, 'explosion_resistance': 16.0, 'requires_tool': True, 'light': 0})

# what joins a cable, a tube and a pipe: the transmitters of their kind, and the blocks that hold what they carry
ELECTRIC = [m for m in MACHINES if m not in FLUID_TANKS and m not in CHEM_TANKS]
CHEMICAL_BLOCKS = [m for m in PROCESS_MACHINES + CONTINUOUS_MACHINES + CHEM_TANKS if m != 'chemical_crystallizer' or True]
FLUID_BLOCKS = ['chemical_washer', 'electrolytic_separator', 'rotary_condensentrator', 'pressurized_reaction_chamber', *FLUID_TANKS]
# the ports and valves of the multiblocks, and the lasers, join what they hold
PORTS_ENERGY = ['teleporter', 'induction_port', 'turbine_valve', 'fusion_reactor_port', 'sps_port', 'laser', 'laser_amplifier', 'quantum_entangloporter']
PORTS_CHEMICAL = ['dynamic_valve', 'boiler_valve', 'turbine_valve', 'fission_reactor_port', 'fusion_reactor_port', 'sps_port',
                  'thermal_evaporation_valve', 'quantum_entangloporter']
PORTS_FLUID = ['dynamic_valve', 'boiler_valve', 'turbine_vent', 'fission_reactor_port', 'fusion_reactor_port', 'thermal_evaporation_valve',
               'quantum_entangloporter']
put(f'{OUT}/data/{NS}/tags/block/energy_connectable.json', {'values': [
    *[f'{NS}:{n}' for n in PORTS_ENERGY], f'{NS}:advanced_solar_generator', f'{NS}:wind_generator',
    *[f'{NS}:{n}' for n in TRANSMITTER_BLOCKS if n.endswith('_universal_cable')],
    *[f'{NS}:{t}_energy_cube' for t in CUBE_TIERS], *[f'{NS}:{m}' for m in ELECTRIC], *[f'{GNS}:{g}' for g in GENERATORS]]})
put(f'{OUT}/data/{NS}/tags/block/chemical_connectable.json', {'values': [
    *[f'{NS}:{n}' for n in TRANSMITTER_BLOCKS if n.endswith('_pressurized_tube')], *[f'{NS}:{m}' for m in CHEMICAL_BLOCKS],
    f'{GNS}:gas_burning_generator', *[f'{NS}:{n}' for n in PORTS_CHEMICAL]]})
put(f'{OUT}/data/{NS}/tags/block/fluid_connectable.json', {'values': [
    *[f'{NS}:{n}' for n in TRANSMITTER_BLOCKS if n.endswith('_mechanical_pipe')], *[f'{NS}:{m}' for m in FLUID_BLOCKS],
    f'{GNS}:heat_generator', f'{GNS}:bio_generator', *[f'{NS}:{n}' for n in PORTS_FLUID]]})

# the blocks a logistical transporter joins: the machines that have item slots, the factories and the vanilla containers
ITEM_BLOCKS = [m for m in MACHINES if m not in FLUID_TANKS and m not in CHEM_TANKS and m not in ('electric_pump', 'fluidic_plenisher')]
put(f'{OUT}/data/{NS}/tags/block/item_connectable.json', {'values': [
    *[f'{NS}:{n}' for n in TRANSMITTER_BLOCKS if n.endswith('_logistical_transporter')], *[f'{NS}:{m}' for m in ITEM_BLOCKS],
    *[f'{NS}:{t}_energy_cube' for t in CUBE_TIERS], f'{NS}:logistical_sorter', f'{NS}:quantum_entangloporter',
    'minecraft:chest', 'minecraft:trapped_chest', 'minecraft:barrel', 'minecraft:furnace', 'minecraft:blast_furnace', 'minecraft:smoker',
    'minecraft:hopper', 'minecraft:dropper', 'minecraft:dispenser', 'minecraft:brewing_stand', 'minecraft:crafter']})

# menus of the machines: slots at Mekanism's places, an energy bar, the progress and the title.
# The panel is Mekanism's own GUI sprites put together here: the nine-slice base, the slot frames and the arrow.
def gui_sprite(rel):
    from PIL import Image
    return Image.open(find(f'textures/gui/sprites/{rel}.png', ASSET_DIRS)).convert('RGBA')

def nine_slice(src, w, h, border):
    """Mekanism's `base.png` stretched to w by h: the corners as they are, the edges and the middle stretched."""
    from PIL import Image
    sw, sh = src.size
    out = Image.new('RGBA', (w, h))
    b = border
    cuts_x = [(0, b, 0, b), (b, sw - b, b, w - b), (sw - b, sw, w - b, w)]
    cuts_y = [(0, b, 0, b), (b, sh - b, b, h - b), (sh - b, sh, h - b, h)]
    for sx0, sx1, dx0, dx1 in cuts_x:
        for sy0, sy1, dy0, dy1 in cuts_y:
            piece = src.crop((sx0, sy0, sx1, sy1)).resize((dx1 - dx0, dy1 - dy0), Image.NEAREST)
            out.paste(piece, (dx0, dy0))
    return out

# Vanilla's font: the width each character takes with its spacing (most take 6), to centre a title.
NARROW = {' ': 4, '!': 2, '"': 5, "'": 3, '(': 5, ')': 5, '*': 5, ',': 2, '.': 2, ':': 2, ';': 2, '<': 5, '>': 5,
          '@': 7, 'I': 4, '[': 4, ']': 4, 'f': 5, 'i': 2, 'k': 5, 'l': 3, 't': 4, '|': 2, '`': 3, '{': 5, '}': 5, '~': 7}
def text_width(text):
    return sum(NARROW.get(c, 6) for c in text) - 1

# Where each machine draws its slots, arrow and bars, as its GuiXxx and TileEntityXxx say (positions of the slots are
# where the tile puts them; their frames are drawn one pixel up and left).
LAYOUTS = {
    # GuiElectricMachine, TileEntityElectricMachine
    'electric': {'slots': [[64, 17], [116, 35]], 'frames': [('slot/input', 64, 17), ('slot/output', 116, 35), ('slot/power', 64, 53)],
                 'up_arrow': (68, 38), 'progress': (86, 38), 'bar_y': 16, 'results': [1]},
    # GuiPrecisionSawmill: one wide frame for the two outputs
    'precision_sawmill': {'slots': [[56, 17], [116, 35], [132, 35]], 'frames': [('slot/input', 56, 17), ('slot/power', 56, 53)],
                          'wide': (111, 30, 42, 26), 'up_arrow': (60, 38), 'progress': (78, 38), 'bar_y': 15, 'results': [1, 2]},
    # GuiCombiner: the main input, the extra one under it, the charge slot at the left
    'combiner': {'slots': [[64, 17], [64, 53], [116, 35]], 'frames': [('slot/input', 64, 17), ('slot/extra', 64, 53), ('slot/output', 116, 35), ('slot/power', 39, 35)],
                 'up_arrow': (68, 38), 'progress': (86, 38), 'bar_y': 15, 'results': [2]},
}

def machine_panel(name, layout):
    """The 176 by 166 panel as Mekanism's machine screen draws it: the base, the inventory's slot frames, the machine's
    frames (one pixel up and left of each slot), the up arrow of the charge slot, the empty progress bar and the frame of
    the energy bar."""
    panel = nine_slice(gui_sprite('base'), 176, 166, 4)
    slot = gui_sprite('slot/normal')
    for row in range(3):
        for col in range(9):
            panel.paste(slot, (7 + 18 * col, 83 + 18 * row))
    for col in range(9):
        panel.paste(slot, (7 + 18 * col, 141))
    if 'wide' in layout:
        x, y, w, h = layout['wide']
        panel.alpha_composite(nine_slice(gui_sprite('slot/output'), w, h, 1), (x, y))
    for sprite, x, y in layout['frames']:
        panel.alpha_composite(gui_sprite(sprite), (x - 1, y - 1))
    if 'up_arrow' in layout:
        panel.alpha_composite(gui_sprite('arrow/up'), layout['up_arrow'])
    panel.alpha_composite(gui_sprite(layout.get('progress_sprite', 'progress/bar/empty')), layout['progress'])
    if 'chem_bar' in layout:
        # GuiChemicalBar: the same frame as the power bar's, around 4 by 52
        panel.alpha_composite(nine_slice(gui_sprite('bar/base'), 6, 54, 1), layout['chem_bar'])
    # GuiBar: a frame two pixels wider and higher than its contents, which are 4 by 52
    panel.alpha_composite(nine_slice(gui_sprite('bar/base'), 6, 54, 1), (164, layout['bar_y']))
    path = f'{OUT}/assets/{NS}/textures/gui/{name}.png'
    os.makedirs(os.path.dirname(path), exist_ok=True)
    panel.save(path)
    return f'{NS}:gui/{name}'

def copy_gui(rel, name):
    path = f'{OUT}/assets/{NS}/textures/gui/{name}.png'
    os.makedirs(os.path.dirname(path), exist_ok=True)
    gui_sprite(rel).save(path)
    return f'{NS}:gui/{name}'

# what each machine uses a tick while it works (the mod's `MACHINES` table; `MekanismConfig.usage`)
# (the release's Joules over the default feConversionRate, 2.5: Forge Energy, what the screen and the port count in)
_USAGE_J = {'crusher': 50, 'enrichment_chamber': 50, 'energized_smelter': 50, 'precision_sawmill': 50, 'combiner': 50, 'metallurgic_infuser': 50,
            'purification_chamber': 200, 'chemical_injection_chamber': 400, 'osmium_compressor': 100, 'chemical_oxidizer': 200,
            'chemical_dissolution_chamber': 400, 'chemical_crystallizer': 400, 'chemical_infuser': 200, 'chemical_washer': 200,
            'electrolytic_separator': 400, 'rotary_condensentrator': 50, 'pressurized_reaction_chamber': 5, 'pigment_extractor': 200,
            'painting_machine': 100, 'pigment_mixer': 200, 'isotopic_centrifuge': 200, 'solar_neutron_activator': 0, 'electric_pump': 100,
            'fluidic_plenisher': 100, 'antiprotonic_nucleosynthesizer': 100_000, 'nutritional_liquifier': 200}
_FACTORY_BASE = {'crushing': 'crusher', 'enriching': 'enrichment_chamber', 'smelting': 'energized_smelter', 'combining': 'combiner', 'sawing': 'precision_sawmill',
                 'compressing': 'osmium_compressor', 'infusing': 'metallurgic_infuser', 'injecting': 'chemical_injection_chamber', 'purifying': 'purification_chamber'}
MACHINE_USAGE = {**{m: j * 2 // 5 for m, j in _USAGE_J.items()}, **{f: _USAGE_J[_FACTORY_BASE[k]] * 2 // 5 for f, (t, k) in FACTORIES.items()}}
MACHINE_NAMES = {'antiprotonic_nucleosynthesizer': 'Antiprotonic Nucleosynthesizer', 'nutritional_liquifier': 'Nutritional Liquifier', 'electric_pump': 'Electric Pump', 'fluidic_plenisher': 'Fluidic Plenisher', **{f: f'{t.title()} {k.title()} Factory' for f, (t, k) in FACTORIES.items()}, 'chemical_infuser': 'Chemical Infuser', 'chemical_washer': 'Chemical Washer',
                 'electrolytic_separator': 'Electrolytic Separator', 'rotary_condensentrator': 'Rotary Condensentrator', 'pressurized_reaction_chamber': 'Pressurized Reaction Chamber', 'pigment_extractor': 'Pigment Extractor',
                 'painting_machine': 'Painting Machine', 'pigment_mixer': 'Pigment Mixer', 'isotopic_centrifuge': 'Isotopic Centrifuge', 'solar_neutron_activator': 'Solar Neutron Activator',
                 **{f'{t}_fluid_tank': f'{t.title()} Fluid Tank' for t in TIERS + ['creative']},
                 **{f'{t}_chemical_tank': f'{t.title()} Chemical Tank' for t in TIERS + ['creative']},
                 'metallurgic_infuser': 'Metallurgic Infuser', 'purification_chamber': 'Purification Chamber',
                 'chemical_injection_chamber': 'Chemical Injection Chamber', 'osmium_compressor': 'Osmium Compressor',
                 'chemical_oxidizer': 'Chemical Oxidizer', 'chemical_dissolution_chamber': 'Chemical Dissolution Chamber',
                 'chemical_crystallizer': 'Chemical Crystallizer', 'crusher': 'Crusher', 'enrichment_chamber': 'Enrichment Chamber', 'energized_smelter': 'Energized Smelter',
                 'precision_sawmill': 'Precision Sawmill', 'combiner': 'Combiner'}
progress_full = copy_gui('progress/bar/full', 'progress_bar_full')
power_fill = copy_gui('bar/vertical_power', 'bar_vertical_power')

# ---- the side configuration window (GuiSideConfiguration), 156 by 135 at 10, 15 of the 176 wide screen
ENUM_COLORS = {  # DataType -> EnumColor (rgb), by the data type's number: none, input, output, input/output, energy, extra
    0: 0xCFCFCF, 1: 0xC9071F, 2: 0xFFA160, 3: 0xFFA160, 4: 0x366BD0, 5: 0x366BD0, 6: 0x00F3D0, 7: 0xA460D9, 8: 0x59C15F, 9: 0xFFDD4F}
DATA_LABELS = {0: 'None', 1: 'Input', 2: 'Input (1)', 3: 'Input (2)', 4: 'Output', 5: 'Output (1)', 6: 'Output (2)',
               7: 'Input/Output', 8: 'Energy', 9: 'Extra'}
def lightened(rgb):
    """BasicColorButton's colour: a little less saturated and a little brighter."""
    import colorsys
    r, g, b = ((rgb >> 16) & 255) / 255, ((rgb >> 8) & 255) / 255, (rgb & 255) / 255
    h, s, v = colorsys.rgb_to_hsv(r, g, b)
    r, g, b = colorsys.hsv_to_rgb(h, max(0, s - 0.1), min(1, v + 0.1))
    return '#%02X%02X%02X' % (round(r * 255), round(g * 255), round(b * 255))
SWATCH_COLORS = [lightened(ENUM_COLORS[i]) for i in range(10)]

def tinted(img, rgb):
    from PIL import Image
    r, g, b = (rgb >> 16) & 255, (rgb >> 8) & 255, rgb & 255
    out = img.copy()
    px = out.load()
    for x in range(out.width):
        for y in range(out.height):
            pr, pg, pb, pa = px[x, y]
            px[x, y] = (pr * r // 255, pg * g // 255, pb * b // 255, pa)
    return out

def tab_sprite(name, icon, tint):
    """A tab of the side holder (26 by 26) with its icon inset (18 by 18), as GuiInsetElement draws it."""
    from PIL import Image
    holder = nine_slice(gui_sprite('holder_left'), 26, 26, 4)
    if tint is not None:
        holder = tinted(holder, tint)
    holder.alpha_composite(gui_sprite(icon), (5, 4))
    return save_gui(holder, name)

def save_gui(img, name):
    path = f'{OUT}/assets/{NS}/textures/gui/{name}.png'
    os.makedirs(os.path.dirname(path), exist_ok=True)
    img.save(path)
    return f'{NS}:gui/{name}'

def image_button(icon):
    """MekanismImageButton: the button sprite with the icon in its middle (14 by 14)."""
    b = nine_slice(gui_sprite('button'), 14, 14, 2)
    b.alpha_composite(gui_sprite(icon), (0, 0))
    return b

WIN_W, WIN_H, WIN_X, WIN_Y = 156, 135, 10, 15
SIDE_BUTTONS = {  # RelativeSide index (front, left, right, back, top, bottom) -> where its button is in the window
    0: (67, 69), 1: (44, 69), 2: (90, 69), 3: (44, 92), 4: (67, 46), 5: (67, 92)}
def side_config_window():
    """The window's background: the base, the close button, the inner screen of the eject state, the frames of the six side
    buttons, the clear button and its labels' places."""
    win = nine_slice(gui_sprite('base'), WIN_W, WIN_H, 4)
    win.alpha_composite(gui_sprite('button/close'), (6, 6))
    # GuiInnerScreen at 38, 25 (80 by 12): a dark well with Mekanism's own edge
    screen_img = nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), 80, 12, 3)
    win.alpha_composite(screen_img, (38, 25))
    for x, y in SIDE_BUTTONS.values():
        win.alpha_composite(nine_slice(gui_sprite('button'), 22, 22, 2), (x, y))
    win.alpha_composite(image_button('button/clear_sides'), (136, 95))
    return save_gui(win, 'side_config_window')

window_tex = side_config_window()
eject_tex = save_gui(image_button('button/auto_eject'), 'side_config_eject')
tab_config = tab_sprite('tab_config', 'button/configuration', None)

# the transmissions, by the port's number (the same as the mod's `Transmission`): the tab's colour, its icon and whether it can eject
TRANSMISSION_INFO = {
    0: ('Items', 'transmission/items', 0xCFCFCF, True),
    1: ('Energy', 'transmission/energy', 0x59C15F, False),
    2: ('Fluids', 'transmission/fluids', 0x366BD0, True),
    3: ('Chemicals', 'transmission/chemicals', 0xFFDD4F, True),
    4: ('Heat', 'transmission/heat', 0xFFA160, False)}
# the transmissions a machine configures, in the order of its tabs (Layout::transmissions in the mod)
MACHINE_TRANSMISSIONS = {'electric_pump': [2, 0, 1], 'fluidic_plenisher': [2, 0, 1], **{f: ([0, 3, 1] if FACTORY_FORM[k] >= 3 else [0, 1]) for f, (t, k) in FACTORIES.items()}, 'crusher': [0, 1], 'enrichment_chamber': [0, 1], 'energized_smelter': [0, 1],
                         'precision_sawmill': [0, 1], 'combiner': [0, 1], **{m: [0, 3, 1] for m in PROCESS_MACHINES},
                         'chemical_infuser': [3, 0, 1], 'chemical_washer': [3, 2, 0, 1], 'electrolytic_separator': [2, 3, 0, 1],
                         'rotary_condensentrator': [3, 2, 0, 1], 'pressurized_reaction_chamber': [0, 3, 2, 1],
                         'pigment_mixer': [3, 0, 1], 'isotopic_centrifuge': [3, 0, 1], 'solar_neutron_activator': [3, 0, 1],
                         **{m: [3, 0] for m in CHEM_TANKS}}
# the transmissions a machine cannot eject (setCanEject(false)): the infuser's tank
NO_EJECT = {(m, 3) for m in ('metallurgic_infuser', 'purification_chamber', 'chemical_injection_chamber', 'osmium_compressor',
                             'chemical_crystallizer', 'painting_machine')} | {('chemical_washer', 2), ('electrolytic_separator', 2)} \
    | {(m, 0) for m in CHEM_TANKS} | {(f, 3) for f, (t, k) in FACTORIES.items() if FACTORY_FORM[k] >= 3}
TAB_SPRITES = {n: tab_sprite(f'tab_{info[0].lower()}', info[1], info[2]) for n, info in TRANSMISSION_INFO.items()}

def side_config_widgets(transmissions, machine=None, facing_slot=None):
    """The window's widgets: for each transmission of the machine its title, the six side buttons coloured by their data type,
    the eject state and the clear button; its tabs outside the window, four at the left then the rest at the right
    (GuiSideConfiguration); the transmission chosen is the screen's variable `type` (the transmission's number). The sides of
    transmission t are in data slots 6 + 2t and 7 + 2t (the top and bottom, and the eject flag in the third nibble)."""
    widgets = []
    for pos, ty in enumerate(transmissions):
        name, _icon, _tint, can_eject = TRANSMISSION_INFO[ty]
        can_eject = can_eject and (machine, ty) not in NO_EJECT
        slots = (6 + 2 * ty, 7 + 2 * ty)
        title = f'{name} Config'
        group = []
        # the title, centred between the close button and the eject button's end
        group.append({'type': 'text', 'x': 20 + (116 - text_width(title)) // 2, 'y': 5, 'text': title, 'color': '#404040'})
        for side, (x, y) in SIDE_BUTTONS.items():
            slot, idx = (slots[0], side) if side < 4 else (slots[1], side - 4)
            group.append({'type': 'swatch', 'x': x + 1, 'y': y + 1, 'w': 20, 'h': 20, 'value': {'nibble': [slot, idx]},
                          'colors': SWATCH_COLORS, 'labels': [DATA_LABELS[i] for i in range(10)],
                          'tooltip': ['Front', 'Left', 'Right', 'Back', 'Top', 'Bottom'][side] + '\n{label}'})
            # the block beside that side, drawn over the colour (as the original's side buttons do)
            group.append({'type': 'neighbor', 'x': x + 3, 'y': y + 3, 'side': side,
                          **({'facing': {'data': facing_slot}} if facing_slot is not None else {})})
            group.append({'type': 'button', 'x': x, 'y': y, 'w': 22, 'h': 22, 'id': 1000 + ty * 100 + side * 10,
                          'alt_id': 1000 + ty * 100 + side * 10 + 1})
        group.append({'type': 'button', 'x': 136, 'y': 95, 'w': 14, 'h': 14, 'id': 3000 + ty * 10, 'alt_id': 3000 + ty * 10 + 1,
                      'tooltip': 'Clear sides\nClick to cycle every side, right click to go back'})
        if can_eject:
            group.append({'type': 'sprite', 'texture': eject_tex, 'x': 136, 'y': 6, 'w': 14, 'h': 14})
            group.append({'type': 'button', 'x': 136, 'y': 6, 'w': 14, 'h': 14, 'id': 2000 + ty, 'tooltip': 'Auto-eject'})
            for on, text in ((0, 'Eject: Off'), (1, 'Eject: On')):
                group.append({'type': 'show', 'when': {'data': {'value': {'nibble': [slots[1], 2]}, 'eq': on}}, 'widgets': [
                    {'type': 'text', 'x': 41, 'y': 28, 'text': text, 'color': '#3CFE9A'}]})
        else:
            group.append({'type': 'text', 'x': 41, 'y': 28, 'text': "Can't Eject", 'color': '#3CFE9A'})
        # the variable `type` is the tab's place in the machine's list, so the window opens on the first of them
        widgets.append({'type': 'show', 'when': {'local': {'var': 'type', 'eq': pos}}, 'widgets': group})
    # the tabs of the other transmissions, outside the window: the first four at its left, the rest at its right
    for i, ty in enumerate(transmissions):
        x = -26 if i < 4 else WIN_W
        for other in range(len(transmissions)):
            if other == i:
                continue
            widgets.append({'type': 'show', 'when': {'local': {'var': 'type', 'eq': other}}, 'widgets': [
                {'type': 'sprite', 'texture': TAB_SPRITES[ty], 'x': x, 'y': 2 + 28 * (i % 4), 'w': 26, 'h': 26},
                {'type': 'button', 'x': x, 'y': 2 + 28 * (i % 4), 'w': 26, 'h': 26,
                 'local': {'set': {'var': 'type', 'value': i}}, 'tooltip': TRANSMISSION_INFO[ty][0]}]})
    widgets.append({'type': 'text', 'x': (WIN_W - text_width('Slots')) // 2, 'y': 120, 'text': 'Slots', 'color': '#787878'})
    widgets.append({'type': 'button', 'x': 6, 'y': 6, 'w': 12, 'h': 12, 'local': 'close', 'tooltip': 'Close'})
    return widgets

# ---- the machines that work a chemical (the infuser, the chambers, the oxidizer, the crystallizer): the chemicals their
# screens draw, written to the mod's chem_table.rs (a chemical's number is its place in the list, from 1)
CHEM_RECIPE_TYPES = ['purifying', 'injecting', 'compressing', 'oxidizing', 'dissolution', 'crystallizing',
                     'metallurgic_infusing', 'chemical_conversion', 'chemical_infusing', 'washing', 'separating', 'rotary', 'reaction',
                     'pigment_extracting', 'pigment_mixing', 'painting', 'nucleosynthesizing']
# the fluids the screens draw (the same table as the chemicals): their colour
FLUIDS = {'minecraft:water': '#3F76E4', 'minecraft:lava': '#FF6A00', 'mekanism:brine': '#FEEF9C', 'mekanism:lithium': '#EBA400',
          'mekanismgenerators:bioethanol': '#CEEB3D', 'mekanism:heavy_water': '#0D1455', 'mekanism:nutritional_paste': '#EB6CA3'}
CHEM_ITEM_INPUT_TYPES = {'purifying', 'injecting', 'compressing', 'dissolution', 'crystallizing', 'metallurgic_infusing', 'painting', 'nucleosynthesizing'}

def chemical_ids():
    ids = []
    def add(i):
        i = i.lstrip('#')
        if i not in ids:
            ids.append(i)
    for p in sorted(glob.glob(f'{GEN}/data/{NS}/recipe/**/*.json', recursive=True)):
        r = read_json(p)
        kind = r.get('type', '').split(':')[-1]
        if kind not in CHEM_RECIPE_TYPES:
            continue
        if kind in CHEM_ITEM_INPUT_TYPES:
            ing = r.get('chemical_input', r.get('input', {})).get('ingredient')
            if isinstance(ing, str):
                add(ing)
        if kind in ('chemical_infusing', 'pigment_mixing'):
            for side in ('left_input', 'right_input'):
                if isinstance(r.get(side, {}).get('ingredient'), str):
                    add(r[side]['ingredient'])
        if kind == 'reaction':
            add(r['chemical_input']['ingredient'])
            if 'chemical' in r.get('output', {}):
                add(r['output']['chemical']['id'])
        if kind == 'rotary':
            for part in ('chemical_input', 'chemical_output', 'fluid_output'):
                v = r.get(part, {})
                i = v.get('ingredient', v.get('id'))
                if isinstance(i, str):
                    add(i)
        if kind == 'washing' and isinstance(r.get('chemical_input', {}).get('ingredient'), str):
            add(r['chemical_input']['ingredient'])
        out = r.get('output')
        if isinstance(out, dict) and 'amount' in out:
            add(out['id'])
        if isinstance(out, dict) and 'left' in out:
            add(out['left']['id'])
            add(out['right']['id'])
    for f in FLUIDS:
        add(f)
    # what the multiblocks hold that no recipe names
    for extra in ('mekanism:nuclear_waste', 'mekanism:steam', 'mekanism:sodium', 'mekanism:superheated_sodium', 'mekanism:fissile_fuel', 'mekanism:deuterium', 'mekanism:tritium', 'mekanism:fusion_fuel', 'mekanism:antimatter', 'mekanism:polonium'):
        add(extra)
    return ids

CHEMICALS = chemical_ids()

def chemical_meta(cid):
    if cid in FLUIDS:
        return FLUIDS[cid], 'mek_liquid/liquid'
    name = cid.split(':')[1]
    p = find(f'mekanism/chemical/{name}.json', DATA_DIRS)
    d = read_json(p) if p else {}
    tint = d.get('tint', '#ffffffff')
    icon = d.get('icon', f'{NS}:mek_liquid/liquid').split(':')[1]
    return '#' + tint[-6:].upper(), icon

def chemical_icon_sprite(icon):
    """The first 16 by 16 of a chemical's icon strip, which the screen tiles and tints."""
    from PIL import Image
    img = Image.open(find(f'textures/{icon}.png', ASSET_DIRS)).convert('RGBA').crop((0, 0, 16, 16))
    return save_gui(img, 'chem_' + icon.replace('/', '_'))

def gauge_types():
    out = []
    for cid in CHEMICALS:
        tint, icon = chemical_meta(cid)
        texture = chemical_icon_sprite(icon)
        if cid in ('minecraft:water', 'minecraft:lava'):
            # a gauge draws the vanilla still texture (the first frame): water tinted blue, lava as it is
            from PIL import Image
            name = cid.split(':')[1]
            img = Image.open(os.path.join(os.path.dirname(os.path.abspath(__file__)), f'assets/{name}_still_frame.png')).convert('RGBA')
            texture = save_gui(img, f'fluid_{name}')
            tint = '#3F76E4' if name == 'water' else '#FFFFFF'
        out.append({'texture': texture, 'tint': tint, 'label': cid.split(':')[1].replace('_', ' ').title()})
    return out

GAUGE_TYPES = gauge_types()
with open(f'{OUT}/src/chem_table.rs', 'w') as f:
    f.write('// Written by tools/convert.py: the chemicals the machines\' screens draw, in the order of their numbers (from 1).\n')
    f.write(f'pub const CHEMICALS: [&str; {len(CHEMICALS)}] = [\n')
    for cid in CHEMICALS:
        f.write(f'    "{cid}",\n')
    f.write('];\n')

PROGRESS_TYPES = {'large_left': (48, 8, 'left'), 'bar': (25, 9, 'right'), 'right': (32, 8, 'right'), 'large_right': (48, 8, 'right'),
                  'small_right': (28, 8, 'right'), 'small_left': (28, 8, 'left'), 'bidirectional': (16, 6, 'right')}
# what each screen has, as the machine's Gui class puts it: slots (the mod's order), the frames and overlays drawn under
# them, the energy bar, the progress arrows, the tank gauges (kind, x, y, w, h, tank, capacity: a standard gauge is 16 by 58,
# a small one 16 by 28, a bar its own size) and the extras
GAUGE_SIZES = {'standard': (16, 58), 'small': (16, 28)}
PROCESS_LAYOUTS = {
    # GuiMetallurgicInfuser
    'metallurgic_infuser': {
        'ticks': 200, 'slots': [[51, 43], [109, 43], [17, 35]], 'results': [1],
        'frames': [('slot/input', 51, 43), ('slot/output', 109, 43), ('slot/extra', 17, 35), ('slot/power', 143, 35)],
        'overlays': [], 'power': ('vertical', 164, 15), 'progress': [('right', 72, 47)],
        'gauges': [('bar', 7, 15, 4, 52, 'in', 1000)], 'dump': (16, 59)},
    # GuiAdvancedElectricMachine: the purification and injection chambers and the osmium compressor
    'purification_chamber': {
        'ticks': 200, 'slots': [[64, 17], [116, 35], [64, 53]], 'results': [1],
        'frames': [('slot/input', 64, 17), ('slot/output', 116, 35), ('slot/extra', 64, 53), ('slot/power', 39, 35)],
        'overlays': [], 'power': ('vertical', 164, 16), 'progress': [('bar', 86, 38)],
        'gauges': [('bar', 68, 36, 6, 12, 'in', 210)]},
    # GuiChemicalOxidizer
    'chemical_oxidizer': {
        'ticks': 100, 'slots': [[26, 36], [152, 55]], 'results': [1], 'inventory': 115,
        'frames': [('slot/input', 26, 36), ('slot/extra', 152, 55), ('slot/power', 152, 14)],
        'overlays': [('slot/overlay/plus', 152, 55)], 'power': ('horizontal', 115, 75), 'progress': [('large_right', 64, 40)],
        'gauges': [('standard', 131, 13, 16, 58, 'out', 10000)]},
    # GuiChemicalDissolutionChamber
    'chemical_dissolution_chamber': {
        'ticks': 100, 'slots': [[28, 36], [152, 55], [8, 65]], 'results': [1],
        # the title between the input gauge and the edge, and no "Inventory" label (GuiChemicalDissolutionChamber)
        'title': (25, 176, 'center'), 'inventory': None, 'debug': (66, 66),
        'frames': [('slot/input', 28, 36), ('slot/extra', 152, 55), ('slot/extra', 8, 65), ('slot/power', 152, 14)],
        'overlays': [('slot/overlay/minus', 8, 65), ('slot/overlay/plus', 152, 55)],
        'power': ('horizontal', 115, 75), 'progress': [('large_right', 64, 40)],
        'gauges': [('standard', 7, 4, 16, 58, 'in', 10000), ('standard', 131, 13, 16, 58, 'out', 10000)]},
    # GuiChemicalCrystallizer
    'chemical_crystallizer': {
        'ticks': 200, 'slots': [[8, 65], [129, 57]], 'results': [1],
        'title': (25, 152, 'center'), 'inventory': None, 'debug': (40, 73),
        'frames': [('slot/extra', 8, 65), ('slot/output', 129, 57), ('slot/power', 152, 5), ('slot/dark', 129, 14)],
        'overlays': [('slot/overlay/plus', 8, 65)], 'power': ('vertical', 157, 23), 'progress': [('large_right', 53, 61)],
        'gauges': [('standard', 7, 4, 16, 58, 'in', 10000)], 'screen': (31, 13, 115, 42), 'crystals': True},
    # GuiChemicalInfuser: two chemical item slots in and one out, three gauges, two arrows driven by whether it works
    'chemical_infuser': {
        'continuous': True, 'slots': [[6, 56], [154, 56], [80, 65]], 'results': [2],
        'title': (1, 79, 'left'), 'inventory': 79, 'debug': (100, 66),
        'frames': [('slot/input', 6, 56), ('slot/input', 154, 56), ('slot/output', 80, 65), ('slot/power', 154, 14)],
        'overlays': [('slot/overlay/minus', 6, 56), ('slot/overlay/minus', 154, 56), ('slot/overlay/plus', 80, 65)],
        'power': ('horizontal', 115, 75), 'progress': [('small_right', 47, 39), ('small_left', 101, 39)],
        'gauges': [('standard', 25, 13, 16, 58, 't0', 10000), ('standard', 133, 13, 16, 58, 't1', 10000),
                   ('standard', 79, 4, 16, 58, 't2', 10000)]},
    # GuiChemicalWasher: the fluid slots in a holder at the left
    'chemical_washer': {
        'continuous': True, 'slots': [[-21, 71], [152, 56], [-21, 102]], 'results': [1, 2],
        'inventory': None, 'debug': (66, 66),
        'frames': [('slot/input', -21, 71), ('slot/extra', 152, 56), ('slot/output', -21, 102), ('slot/power', 152, 14)],
        'overlays': [('slot/overlay/minus', 152, 56)], 'holder': (-26, 66, 57),
        'power': ('horizontal', 115, 75), 'progress': [('large_right', 64, 39)],
        'extra_sprites': [('arrow/down', -17, 90)],
        'gauges': [('standard', 7, 13, 16, 58, 't0', 10000), ('standard', 28, 13, 16, 58, 't1', 10000),
                   ('standard', 131, 13, 16, 58, 't2', 10000)]},
    # GuiElectrolyticSeparator: a fluid gauge, two small gas gauges and the dump buttons of both
    'electrolytic_separator': {
        'continuous': True, 'slots': [[26, 35], [59, 52], [101, 52]], 'results': [1, 2],
        'title': (23, 176, 'center'), 'inventory': None, 'debug': (46, 60),
        'frames': [('slot/input', 26, 35), ('slot/output', 59, 52), ('slot/output', 101, 52), ('slot/power', 143, 35)],
        'overlays': [], 'power': ('vertical', 164, 15), 'progress': [('bidirectional', 80, 30)],
        'gauges': [('standard', 5, 10, 16, 58, 't0', 24000), ('small', 58, 18, 16, 28, 't1', 2400),
                   ('small', 100, 18, 16, 28, 't2', 2400)],
        'modes': [(7, 72, 5000, 22, 'left'), (159, 72, 5001, 23, 'right')]},
}
    # GuiRotaryCondensentrator: gas on the left, fluid on the right, an arrow each way by the mode, a toggle button
PROCESS_LAYOUTS['rotary_condensentrator'] = {
    'continuous': True, 'rotary': True, 'slots': [[5, 25], [5, 56], [155, 25], [155, 56], [155, 5]], 'results': [1, 3],
    'title': (0, 176, 'center'), 'title_y': 4, 'inventory': None, 'debug': (115, 66),
    'frames': [('slot/input', 5, 25), ('slot/output', 5, 56), ('slot/input', 155, 25), ('slot/output', 155, 56),
               ('slot/power', 155, 5)],
    'overlays': [], 'power': ('horizontal', 115, 75), 'progress': [],
    'extra_sprites': [('arrow/down', 9, 44), ('arrow/down', 159, 44)],
    'gauges': [('standard', 133, 13, 16, 58, 't0', 10000), ('standard', 25, 13, 16, 58, 't1', 10000)]}
    # GuiPRC: the fluid and the gas it takes in at the left, the small gas it makes at the right; 5 pixels taller than the others
PROCESS_LAYOUTS['pressurized_reaction_chamber'] = {
    'continuous': True, 'timed': True, 'slots': [[54, 40], [116, 40], [141, 22]], 'results': [1],
    'height': 171, 'inv_shift': 5, 'label_y': 77, 'debug': (60, 66),
    'frames': [('slot/input', 54, 40), ('slot/output', 116, 40), ('slot/power', 141, 22)], 'overlays': [],
    'power': ('vertical', 163, 21), 'progress': [('right', 77, 43)],
    'gauges': [('standard', 5, 15, 16, 58, 't0', 10000), ('standard', 28, 15, 16, 58, 't1', 10000),
               ('small', 140, 45, 16, 28, 't2', 10000)]}
# the pigment machines: the extractor is the oxidizer's screen with a 20 bucket tank, the painting machine and the mixer have their own
PROCESS_LAYOUTS['antiprotonic_nucleosynthesizer'] = {**PROCESS_LAYOUTS['purification_chamber'], 'ticks': 400}
PROCESS_LAYOUTS['nutritional_liquifier'] = {**PROCESS_LAYOUTS['chemical_oxidizer'], 'ticks': 100}
PROCESS_LAYOUTS['pigment_extractor'] = {**PROCESS_LAYOUTS['chemical_oxidizer'], 'ticks': 100,
                                       'tint_arrow': 'out', 'gauges': [('standard', 131, 13, 16, 58, 'out', 20000)]}
PROCESS_LAYOUTS['painting_machine'] = {
    'ticks': 200, 'slots': [[45, 35], [116, 35], [6, 56]], 'results': [1], 'title_y': 4, 'label_y': 74, 'debug': (80, 66),
    'frames': [('slot/input', 45, 35), ('slot/output', 116, 35), ('slot/extra', 6, 56), ('slot/power', 144, 35)],
    'overlays': [('slot/overlay/minus', 6, 56)], 'power': ('vertical', 164, 15), 'progress': [('large_right', 64, 39)],
    'tint_arrow': 'in', 'gauges': [('standard', 25, 13, 16, 58, 'in', 15000)]}
PROCESS_LAYOUTS['pigment_mixer'] = {**PROCESS_LAYOUTS['chemical_infuser'], 'title_y': 5, 'label_y': 74, 'tint_arrow': 't2',
                                    'gauges': [('standard', 25, 13, 16, 58, 't0', 1000), ('standard', 133, 13, 16, 58, 't1', 1000),
                                               ('standard', 79, 4, 16, 58, 't2', 2000)]}
# the nuclear chain: the infuser's screen with one gas in at the left and one out at the right
for _m in ('isotopic_centrifuge', 'solar_neutron_activator'):
    PROCESS_LAYOUTS[_m] = {**PROCESS_LAYOUTS['chemical_infuser'], 'progress': [('large_right', 64, 40)],
                           'gauges': [('standard', 25, 13, 16, 58, 't0', 10000), ('standard', 133, 13, 16, 58, 't1', 10000)]}
for _m in ('chemical_injection_chamber', 'osmium_compressor'):
    PROCESS_LAYOUTS[_m] = PROCESS_LAYOUTS['purification_chamber']
# the compressor's bar is the same, but its tank holds 210 too; the three are one layout

# the data slots about the tanks: how much and which chemical, for the tank the chemical goes into and for the one it
# comes out in (after the 16 the energy, the progress and the configuration use); a continuous machine's three
TANK_DATA = {'in': (16, 17), 'out': (18, 19), 't0': (16, 17), 't1': (18, 19), 't2': (20, 21)}

def process_panel(name, L):
    """The 176 by 166 panel of a machine of `process`: the base, the inventory's frames, the machine's slot frames and
    overlays, its tank frames, the energy bar's frame and the empty progress arrows."""
    from PIL import Image
    shift = L.get('inv_shift', 0)
    panel = nine_slice(gui_sprite('base'), 176, L.get('height', 166), 4)
    slot = gui_sprite('slot/normal')
    for row in range(3):
        for col in range(9):
            panel.paste(slot, (7 + 18 * col, 83 + shift + 18 * row))
    for col in range(9):
        panel.paste(slot, (7 + 18 * col, 141 + shift))
    if 'screen' in L:
        x, y, w, h = L['screen']
        # GuiInnerScreen: a dark well with Mekanism's own edge
        panel.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), w, h, 3), (x, y))
    for sprite, x, y in L['frames']:
        panel.alpha_composite(gui_sprite(sprite), (x - 1, y - 1))
    for sprite, x, y in L['overlays']:
        panel.alpha_composite(gui_sprite(sprite), (x - 1, y - 1))
    kind, px, py = L['power']
    panel.alpha_composite(nine_slice(gui_sprite('bar/base'), 6 if kind == 'vertical' else 54, 54 if kind == 'vertical' else 6, 1), (px, py))
    for ptype, qx, qy in L['progress']:
        panel.alpha_composite(gui_sprite(f'progress/{ptype}/empty'), (qx, qy))
    for sprite, x, y in L.get('extra_sprites', []):
        panel.alpha_composite(gui_sprite(sprite), (x, y))
    for gk, gx, gy, gw, gh, _tank, _cap in L['gauges']:
        if gk == 'bar':
            panel.alpha_composite(nine_slice(gui_sprite('bar/base'), gw + 2, gh + 2, 1), (gx, gy))
        else:
            panel.alpha_composite(nine_slice(gui_sprite('gauge/normal'), gw + 2, gh + 2, 2), (gx, gy))
    for mx, my, _id, _slot, _side in L.get('modes', []):
        pass
    return save_gui(panel, name)

def side_holder(left, h):
    """GuiSideHolder (26 wide): the nine-slice holder of the side, untinted."""
    return nine_slice(gui_sprite('holder_left' if left else 'holder_right'), 26, h, 4)

power_h_fill = copy_gui('bar/horizontal_power', 'bar_horizontal_power')
# the original scrolls a name longer than its 69 pixel box; here the longest is written shorter
MODE_TEXT = ['Idle', 'Dumping', 'Dump excess']

# the redstone control tab (GuiRedstoneControlTab at 176, 137, 26 by 18, tinted 0xC9071F) and the energy tab of the machines
# (GuiEnergyTab at -26, 137): the screen's number `c` is the control (0 ignored, 1 normal, 2 inverted, 3 pulse), c+1 and
# c+2 the energy still needed to be full
def redstone_tab(mode):
    from PIL import Image
    holder = tinted(nine_slice(gui_sprite('holder_right'), 26, 18, 4), 0xC9071F)
    icon = gui_sprite(f'redstone_control/{mode}')
    if icon.size[1] > icon.size[0]:
        icon = icon.crop((0, 0, icon.size[0], icon.size[0]))
    holder.alpha_composite(icon, (4 + (18 - icon.size[0]) // 2, (18 - icon.size[1]) // 2))
    return save_gui(holder, f'redstone_{mode}')

tab_energy_info = copy_gui('tab/energy_info', 'tab_energy_info')
REDSTONE_TIPS = ['Redstone Detection: IGNORED', 'Redstone Detection: NORMAL', 'Redstone Detection: INVERTED', 'Redstone Detection: PULSE']
def redstone_widgets(c):
    out = []
    for k, mode in enumerate(['disabled', 'high', 'low', 'pulse']):
        out.append({'type': 'show', 'when': {'data': {'value': {'data': c}, 'eq': k}}, 'widgets': [
            {'type': 'sprite', 'texture': redstone_tab(mode), 'x': 176, 'y': 137, 'w': 26, 'h': 18},
            {'type': 'button', 'x': 176, 'y': 137, 'w': 26, 'h': 18, 'id': 6000, 'alt_id': 6001, 'tooltip': REDSTONE_TIPS[k]}]})
    return out

def energy_tab_widgets(c, usage):
    """GuiEnergyTab(this, energyContainer, lastEnergyUsed): what it uses a tick (its usage while it works, status 1) and
    what it needs to be full."""
    out = [{'type': 'sprite', 'texture': tab_energy_info, 'x': -26, 'y': 137, 'w': 26, 'h': 26}]
    for working in (0, 1):
        out.append({'type': 'show', 'when': {'data': {'value': {'data': 5}, 'eq': 1 if working else 0}}, 'widgets': [
            {'type': 'button', 'x': -26, 'y': 137, 'w': 26, 'h': 26,
             'tooltip': f'Using: {{{c + 14}}} FE/t\nNeeded: {{{c + 1}+{c + 2}}} FE\nUnit: Forge Energy' if working else f'Using: 0 FE/t\nNeeded: {{{c + 1}+{c + 2}}} FE\nUnit: Forge Energy'}]})
    return out

# ---- upgrades (GuiUpgradeWindowTab at 176, 6; GuiUpgradeWindow, GuiUpgradeScrollList, GuiSupportedUpgrades): the upgrade items,
# their recipes and names, and the window every machine's screen opens: its two slots, the list of what is installed, the
# description of the one selected, the progress arrows, the button that takes one out and the row of supported types.
UPGRADES = ['speed', 'energy', 'chemical', 'muffling', 'anchor', 'filter', 'stone_generator']
UPGRADE_MAX = [8, 8, 8, 1, 1, 1, 1]
UPGRADE_NAMES = ['Speed', 'Energy', 'Chemical', 'Muffling', 'Anchor', 'Filter', 'Stone Generator']
UPGRADE_DESC = ['Increases speed of machinery.', 'Increases energy efficiency and capacity of machinery.',
                'Increases the efficiency of chemical-using machinery.', 'Reduces noise generated by machinery.',
                "Keeps a machine's chunk loaded.",
                'A filter that separates heavy water from regular water.', 'Generates stone or cobblestone as needed.']
# EnumColor of each upgrade's colour (red, lime, yellow, light blue, lime, green, cyan, orange)
UPGRADE_COLORS = [0xFF383C, 0x75FF89, 0xFFDD4F, 0x559EFF, 0x59C15F, 0x00F3D0, 0xFFA160]
# the types each kind of machine takes (`AttributeUpgradeSupport`), and the ones its list shows (the first five)
ADVANCED_UPGRADES = {'purification_chamber', 'chemical_injection_chamber', 'chemical_dissolution_chamber'}
LISTED_UPGRADES = 4
UP_W, UP_H = 198, 94

def upgrade_support(machine):
    if machine in GENERATORS:
        return [3]
    return [0, 1, 2, 3] if machine in ADVANCED_UPGRADES else [0, 1, 3]

def upgrade_items_and_recipes():
    """The eight upgrade items (`mekanism:upgrade` carries its type in a component; here each type is its own item), their
    models, names and crafting recipes."""
    for k, name in enumerate(UPGRADES):
        n = f'upgrade_{name}'
        put(f'{OUT}/data/{NS}/horizon/item/{n}.json', {'template': 'minecraft:stick', 'rarity': 'uncommon'})
        copy_texture(f'{NS}:item/upgrade_{name}')
        put(f'{OUT}/assets/{NS}/models/item/{n}.json', {'parent': 'minecraft:item/generated', 'textures': {'layer0': f'{NS}:item/upgrade_{name}'}})
        put(f'{OUT}/assets/{NS}/items/{n}.json', {'model': {'type': 'minecraft:model', 'model': f'{NS}:item/{n}'}})
        kept_items.append(n)
        r = read_json(f'{GEN}/data/{NS}/recipe/upgrade/{name}.json')
        r['result'] = {'id': f'{NS}:{n}'}
        put(f'{OUT}/data/{NS}/recipe/{n}.json', r)

def upgrade_icon(k):
    """The item's texture as a 16 by 16 sprite the screen draws small."""
    from PIL import Image
    img = Image.open(find(f'textures/item/upgrade_{UPGRADES[k]}.png', ASSET_DIRS)).convert('RGBA')
    return save_gui(img, f'upgrade_icon_{UPGRADES[k]}')

def upgrade_window(machine, base, c):
    """The window of a machine with `base` slots of its own, whose screen numbers from `c` are the control, the energy needed (2),
    the ticks, the install progress, the eight counts and the list's mask (`stamp` in the mod)."""
    from PIL import Image
    sup = upgrade_support(machine)
    counts_at, mask_at, prog_at = c + 5, c + 13, c + 4
    # the texture: the base, the list's holder, the description's screen, the two slots with their overlay, the arrows' frames
    panel = nine_slice(gui_sprite('base'), UP_W, UP_H, 4)
    panel.alpha_composite(nine_slice(gui_sprite('element_holder'), 108, 50, 2), (6, 18))
    panel.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), 59, 50, 3), (114, 18))
    for y in (18, 72):
        panel.alpha_composite(gui_sprite('slot/normal'), (175, y))
        panel.alpha_composite(gui_sprite('slot/overlay/upgrade'), (175, y))
    panel.alpha_composite(gui_sprite('progress/installing/empty'), (176, 37))
    panel.alpha_composite(gui_sprite('progress/uninstalling/empty'), (176, 58))
    panel.alpha_composite(nine_slice(gui_sprite('element_holder'), 167, 14, 2), (6, 68))
    texture = save_gui(panel, 'upgrade_window')
    widgets = []
    title = 'Upgrades'
    widgets.append({'type': 'text', 'x': (UP_W - text_width(title)) // 2, 'y': 5, 'text': title, 'color': '#404040'})
    # the list: rows of what is installed, as many as the mask says, in the order of the types; the selected one is lit
    listed = [k for k in sup if k < LISTED_UPGRADES]
    def list_rows(rows, shown_from):
        group = []
        for r, k in enumerate(rows):
            y = 19 + 12 * r
            def row_sprites(lit, k=k, y=y):
                sprite = 'list/selected' if lit else 'list/base'
                slot = 'list/slot_selected' if lit else 'list/slot'
                return [{'type': 'sprite', 'texture': save_gui(tinted(nine_slice(gui_sprite(sprite), 100, 12, 1), UPGRADE_COLORS[k]), f'upgrade_row_{k}_{int(lit)}'), 'x': 7, 'y': y, 'w': 100, 'h': 12},
                        {'type': 'sprite', 'texture': save_gui(tinted(gui_sprite(slot), UPGRADE_COLORS[k]), f'upgrade_slot_{k}_{int(lit)}'), 'x': 8, 'y': y + 1, 'w': 10, 'h': 10}]
            group += row_sprites(False)
            group.append({'type': 'show', 'when': {'local': {'var': 'sel', 'eq': k + 1}}, 'widgets': row_sprites(True)})
            group.append({'type': 'sprite', 'texture': upgrade_icon(k), 'x': 9, 'y': y + 2, 'w': 8, 'h': 8})
            group.append({'type': 'text', 'x': 19, 'y': y + 2, 'text': UPGRADE_NAMES[k], 'color': '#404040'})
            group.append({'type': 'button', 'x': 7, 'y': y, 'w': 100, 'h': 12, 'local': {'set': {'var': 'sel', 'value': k + 1}},
                          'tooltip': UPGRADE_DESC[k], 'hover': '#33FFFFFF'})
        return group
    for combo in range(1, 1 << len(listed)):
        mask = sum(1 << listed[i] for i in range(len(listed)) if combo & (1 << i))
        every = [listed[i] for i in range(len(listed)) if combo & (1 << i)]
        if len(every) <= 4:
            group = list_rows(every, 0)
        else:
            # more than four rows: a scroll bar beside the list (GuiScrollBar); a press on its top or bottom half moves the list
            group = [{'type': 'swatch', 'x': 108, 'y': 19, 'w': 5, 'h': 48, 'value': {'const': 0}, 'colors': ['#373737']}]
            for off in range(len(every) - 3):
                thumb_h = 48 * 4 // len(every)
                thumb_y = 19 + (48 - thumb_h) * off // (len(every) - 4)
                group.append({'type': 'show', 'when': {'local': {'var': 'scroll', 'eq': off}}, 'widgets': list_rows(every[off:off + 4], off) + [
                    {'type': 'swatch', 'x': 108, 'y': thumb_y, 'w': 5, 'h': thumb_h, 'value': {'const': 0}, 'colors': ['#C6C6C6']}]})
            half = 24
            group.append({'type': 'button', 'x': 108, 'y': 19, 'w': 5, 'h': half, 'local': {'set': {'var': 'scroll', 'value': 0}}, 'tooltip': 'Scroll up'})
            group.append({'type': 'button', 'x': 108, 'y': 19 + half, 'w': 5, 'h': 48 - half, 'local': {'set': {'var': 'scroll', 'value': len(every) - 4}}, 'tooltip': 'Scroll down'})
        widgets.append({'type': 'show', 'when': {'data': {'value': {'data': mask_at}, 'eq': mask}}, 'widgets': group})
    # the description of the selected type (or "No selection."): its name, how many and what effect, in small print
    widgets.append({'type': 'show', 'when': {'local': {'var': 'sel', 'eq': 0}}, 'widgets': [
        {'type': 'text', 'x': 117, 'y': 21, 'text': 'No selection.', 'color': '#00CD00', 'scale': 80}]})
    for k in sup:
        count_at = counts_at + k
        full = f'{UPGRADE_NAMES[k]} Upgrade'
        lines = 1 if text_width(full) * 0.6 <= 53 else 2
        y_amount = 21 + 6 * lines
        none = {'type': 'show', 'when': {'data': {'value': {'data': count_at}, 'eq': 0}}, 'widgets': [
            {'type': 'text', 'x': 117, 'y': 21, 'text': 'No selection.', 'color': '#00CD00', 'scale': 80}]}
        shown = []
        for n in range(1, UPGRADE_MAX[k] + 1):
            body = [{'type': 'text', 'x': 117, 'y': 21, 'text': full, 'color': '#00CD00', 'scale': 60},
                    {'type': 'text', 'x': 117, 'y': y_amount, 'text': f'Amount: {n}/{UPGRADE_MAX[k]}', 'color': '#00CD00', 'scale': 60}]
            if UPGRADE_MAX[k] > 1:
                if machine in CONTINUOUS_MACHINES and k == 0:
                    text = f'Effect: {float(2 ** n)}x'
                else:
                    text = 'Effect: %sx' % (str(round(10 ** (n / UPGRADE_MAX[k]) * 100) / 100))
                body.append({'type': 'text', 'x': 117, 'y': y_amount + 6, 'text': text, 'color': '#00CD00', 'scale': 60})
            shown.append({'type': 'show', 'when': {'data': {'value': {'data': count_at}, 'eq': n}}, 'widgets': body})
        widgets.append({'type': 'show', 'when': {'local': {'var': 'sel', 'eq': k + 1}}, 'widgets': [none] + shown})
        # the button that takes one out (a right press: all), lit while something of the type is installed
        widgets.append({'type': 'show', 'when': {'local': {'var': 'sel', 'eq': k + 1}}, 'widgets': [
            {'type': 'show', 'when': {'data': {'value': {'data': count_at}, 'eq': n}}, 'widgets': [
                {'type': 'sprite', 'texture': save_gui(nine_slice(gui_sprite('button_digital'), 56, 12, 1), 'upgrade_button_on'), 'x': 115, 'y': 54, 'w': 56, 'h': 12},
                {'type': 'text', 'x': 115 + (56 - text_width('Uninstall')) // 2, 'y': 56, 'text': 'Uninstall', 'color': '#FFFFFF'},
                {'type': 'button', 'x': 115, 'y': 54, 'w': 56, 'h': 12, 'id': 6100 + k, 'alt_id': 6110 + k, 'shift_id': 6110 + k,
                 'tooltip': 'Uninstalls a single upgrade, hold Shift (or a long press) to uninstall all.', 'hover': '#33FFFFFF'}]} for n in range(1, UPGRADE_MAX[k] + 1)]})
    # the "Uninstall" button: dark (inactive) here, lit over it while the selected type has something installed
    widgets.insert(1, {'type': 'sprite', 'texture': save_gui(nine_slice(gui_sprite('button_digital_inactive'), 56, 12, 1), 'upgrade_button_off'), 'x': 115, 'y': 54, 'w': 56, 'h': 12})
    widgets.insert(2, {'type': 'text', 'x': 115 + (56 - text_width('Uninstall')) // 2, 'y': 56, 'text': 'Uninstall', 'color': '#A0A0A0'})
    # the arrow that fills while an upgrade installs (a second)
    widgets.append({'type': 'sprite', 'texture': copy_gui('progress/installing/full', 'progress_installing_full'), 'x': 176, 'y': 37, 'w': 10, 'h': 14,
                    'value': {'data': prog_at}, 'max': {'const': 20}, 'direction': 'down', 'tooltip': 'Installing'})
    # the supported types: "Supported:" and every type's icon, faded when the machine does not take it
    widgets.append({'type': 'text', 'x': 8, 'y': 71, 'text': 'Supported:', 'color': '#404040'})
    x0 = 7 + text_width('Supported:') + 1
    for k in range(len(UPGRADES)):
        x = x0 + 12 * k
        widgets.append({'type': 'sprite', 'texture': upgrade_icon(k), 'x': x, 'y': 69, 'w': 12, 'h': 12})
        if k not in sup:
            widgets.append({'type': 'swatch', 'x': x, 'y': 69, 'w': 12, 'h': 12, 'value': {'const': 0}, 'colors': ['#99C6C6C6']})
        tip = f'{UPGRADE_NAMES[k]} Upgrade' + ('' if k in sup else ' (Not Supported)') + '\n' + UPGRADE_DESC[k]
        widgets.append({'type': 'button', 'x': x, 'y': 69, 'w': 12, 'h': 12, 'tooltip': tip})
    widgets.append({'type': 'button', 'x': 6, 'y': 6, 'w': 12, 'h': 12, 'local': 'close', 'tooltip': 'Close'})
    return {'id': 'upgrades', 'x': (176 - UP_W) // 2, 'y': 15, 'width': UP_W, 'height': UP_H, 'texture': texture, 'widgets': widgets,
            'slots': [{'slot': base, 'x': 176, 'y': 19}, {'slot': base + 1, 'x': 176, 'y': 73}]}

def upgrade_tab_widgets():
    """The tab at 176, 6 (26 by 18) that opens the window."""
    holder = nine_slice(gui_sprite('holder_right'), 26, 18, 4)
    holder.alpha_composite(gui_sprite('button/upgrade'), (4, 0))
    return [{'type': 'sprite', 'texture': save_gui(holder, 'upgrade_tab'), 'x': 176, 'y': 6, 'w': 26, 'h': 18},
            {'type': 'button', 'x': 176, 'y': 6, 'w': 26, 'h': 18, 'local': {'open': 'upgrades'}, 'tooltip': 'Upgrades'}]

def with_upgrades(menu, machine, base, c):
    """Adds the upgrade window, its tab, its two slots (the filter and the take-only one) and its screen numbers to a menu."""
    menu['slots'] = base + 2
    menu['results'] = list(menu.get('results', [])) + [base + 1]
    menu['accepts'] = [{'slot': base, 'items': [f'{NS}:upgrade_{UPGRADES[k]}' for k in upgrade_support(machine)]}]
    menu['data_slots'] = c + 16
    screen = menu['screen']
    screen['slots'] = list(screen['slots']) + [None, None]
    screen['widgets'] = list(screen['widgets']) + upgrade_tab_widgets()
    screen['windows'] = list(screen.get('windows', [])) + [upgrade_window(machine, base, c)]
    return menu


upgrade_items_and_recipes()

# ---- the buckets of Mekanism's fluids (liquid gases, brine, lithium...): a vanilla bucket with its fluid area tinted by the fluid
# (`neoforge:fluid_container`: the bucket's texture and a mask of where the fluid is), full ones the tanks take and give
BUCKET_FLUIDS = ['brine', 'chlorine', 'ethene', 'heavy_water', 'hydrofluoric_acid', 'hydrogen', 'hydrogen_chloride', 'lithium',
                 'nutritional_paste', 'oxygen', 'sodium', 'steam', 'sulfur_dioxide', 'sulfur_trioxide', 'sulfuric_acid',
                 'superheated_sodium', 'uranium_hexafluoride', 'uranium_oxide']
BUCKET_TINTS = {'heavy_water': '#0D1455', 'nutritional_paste': '#EB6CA3'}

def bucket_items():
    mask = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'assets/bucket_fluid_mask.png')
    put(f'{OUT}/assets/{NS}/textures/item/bucket_fluid_mask.png', src=mask)
    for name in BUCKET_FLUIDS:
        n = f'{name}_bucket'
        tint = BUCKET_TINTS.get(name) or FLUIDS.get(f'{NS}:{name}') or chemical_meta(f'{NS}:{name}')[0]
        put(f'{OUT}/data/{NS}/horizon/item/{n}.json', {'template': 'minecraft:stick', 'max_stack_size': 1})
        put(f'{OUT}/assets/{NS}/models/item/{n}.json', {'parent': 'minecraft:item/generated',
            'textures': {'layer0': 'minecraft:item/bucket', 'layer1': f'{NS}:item/bucket_fluid_mask'}})
        put(f'{OUT}/assets/{NS}/items/{n}.json', {'model': {'type': 'minecraft:model', 'model': f'{NS}:item/{n}',
            'tints': [{'type': 'minecraft:constant', 'value': 0xFFFFFF}, {'type': 'minecraft:constant', 'value': int(tint.lstrip('#')[-6:], 16)}]}})
        kept_items.append(n)

bucket_items()

def process_menu(m):
    L = PROCESS_LAYOUTS[m]
    title = MACHINE_NAMES[m]
    kind, px, py = L['power']
    if kind == 'vertical':
        power = {'type': 'sprite', 'texture': power_fill, 'x': px + 1, 'y': py + 1, 'w': 4, 'h': 52, 'direction': 'up',
                 'value': {'pair': [1, 2]}, 'max': {'pair': [3, 4]}, 'tooltip': 'Energy: {value} / {max}'}
    else:
        power = {'type': 'sprite', 'texture': power_h_fill, 'x': px + 1, 'y': py + 1, 'w': 52, 'h': 4, 'direction': 'right',
                 'value': {'pair': [1, 2]}, 'max': {'pair': [3, 4]}, 'tooltip': 'Energy: {value} / {max}'}
    widgets = []
    # the washer's side holder goes behind the slots
    if 'holder' in L:
        hx, hy, hh = L['holder']
        widgets.append({'type': 'sprite', 'texture': save_gui(side_holder(True, hh), f'holder_left_{hh}'), 'x': hx, 'y': hy, 'w': 26, 'h': hh})
    widgets.append(power)
    continuous = L.get('continuous', False)
    ctl0 = 24 if continuous else 20
    for ptype, qx, qy in L['progress']:
        pw, ph, direction = PROGRESS_TYPES[ptype]
        # a machine that works every tick shows its arrows full while it works (data slot 0 is 1 or 0)
        widgets.append({'type': 'sprite', 'texture': copy_gui(f'progress/{ptype}/full', f'progress_{ptype}_full'), 'x': qx, 'y': qy,
                        'w': pw, 'h': ph, 'value': {'data': 0}, 'max': {'const': 1} if (continuous and not L.get('timed')) else {'data': ctl0 + 3},
                        'direction': direction, 'tooltip': 'Working' if (continuous and not L.get('timed')) else 'Progress: {percent}%',
                        **({'tints': [t['tint'] for t in GAUGE_TYPES], 'tint_by': {'data': TANK_DATA[L['tint_arrow']][1]}} if 'tint_arrow' in L else {})})
    for gk, gx, gy, gw, gh, tank, cap in L['gauges']:
        amount, which = TANK_DATA[tank]
        widgets.append({'type': 'gauge', 'x': gx + 1, 'y': gy + 1, 'w': gw, 'h': gh, 'value': {'data': amount},
                        'max': {'const': cap}, 'kind': {'data': which}, 'types': GAUGE_TYPES,
                        'tooltip': '{label}: {value} mB', 'empty': 'Empty'})
        if gk in GAUGE_SIZES:
            widgets.append({'type': 'sprite', 'texture': copy_gui(f'gauge/overlay/{gk}', f'gauge_overlay_{gk}'),
                            'x': gx + 1, 'y': gy + 1, 'w': gw, 'h': gh})
    if L.get('crystals'):
        # GuiCrystallizerScreen: in its inner screen the chemical's name, the item it crystallizes into in brackets and that item
        # in a dark slot at the right
        for p in sorted(glob.glob(f'{GEN}/data/{NS}/recipe/crystallizing/*.json')):
            r = read_json(p)
            chem, item = r['input']['ingredient'], r['output']['id']
            if chem not in CHEMICALS:
                continue
            words = item.split(':')[1].split('_')
            label = ' '.join(w.title() for w in (words[1:] + words[:1] if words[0] == 'dust' else words))
            widgets.append({'type': 'show', 'when': {'data': {'value': {'data': TANK_DATA['in'][1]}, 'eq': CHEMICALS.index(chem) + 1}}, 'widgets': [
                {'type': 'text', 'x': 34, 'y': 16, 'text': chem.split(':')[1].replace('_', ' ').title(), 'color': '#3CFE9A'},
                {'type': 'text', 'x': 34, 'y': 26, 'text': f'({label})', 'color': '#3CFE9A'},
                {'type': 'item', 'x': 129, 'y': 14, 'item': item}]})
    if L.get('rotary'):
        # the arrow of the mode (data slot 22 is the mode: 0 condensentrates, 1 decondensentrates), full while it works
        for k, ptype in enumerate(['large_right', 'large_left']):
            pw, ph, direction = PROGRESS_TYPES[ptype]
            widgets.append({'type': 'show', 'when': {'data': {'value': {'data': 22}, 'eq': k}}, 'widgets': [
                {'type': 'sprite', 'texture': copy_gui(f'progress/{ptype}/empty', f'progress_{ptype}_empty'), 'x': 64, 'y': 39, 'w': pw, 'h': ph},
                {'type': 'sprite', 'texture': copy_gui(f'progress/{ptype}/full', f'progress_{ptype}_full'), 'x': 64, 'y': 39,
                 'w': pw, 'h': ph, 'value': {'data': 0}, 'max': {'const': 1}, 'direction': direction, 'tooltip': 'Working'}]})
        # ToggleButton at 4, 4 (18 by 18), flipped when decondensentrating; its mode written below it
        for k, (sprite, label) in enumerate([('button/toggle', 'Condensentrating'), ('button/toggle_flipped', 'Decondensentrating')]):
            widgets.append({'type': 'show', 'when': {'data': {'value': {'data': 22}, 'eq': k}}, 'widgets': [
                {'type': 'sprite', 'texture': copy_gui(sprite, sprite.replace('/', '_')), 'x': 4, 'y': 4, 'w': 18, 'h': 18},
                {'type': 'text', 'x': 4, 'y': 74, 'text': label, 'color': '#404040'}]})
        widgets.append({'type': 'button', 'x': 4, 'y': 4, 'w': 18, 'h': 18, 'id': 5200, 'tooltip': 'Toggle operation'})
    if 'dump' in L:
        dx, dy = L['dump']
        widgets.append({'type': 'sprite', 'texture': copy_gui('button/dump', 'button_dump'), 'x': dx, 'y': dy, 'w': 21, 'h': 10})
        widgets.append({'type': 'button', 'x': dx, 'y': dy, 'w': 21, 'h': 10, 'id': 4000, 'tooltip': 'Dump'})
    for mx, my, bid, mode_slot, side in L.get('modes', []):
        # GuiGasMode: its icon by the mode and the mode's name beside it (to the left of the left button, right of the other)
        for k, name in enumerate(['idle', 'dump', 'dump_excess']):
            icon = {'idle': 'chemical_mode/idle', 'dump': 'chemical_mode/dump', 'dump_excess': 'chemical_mode/dump_excess'}[name]
            tx = mx + 13 if side == 'left' else mx - 3 - text_width(MODE_TEXT[k])
            widgets.append({'type': 'show', 'when': {'data': {'value': {'data': mode_slot}, 'eq': k}}, 'widgets': [
                {'type': 'sprite', 'texture': copy_gui(icon, f'mode_{name}'), 'x': mx, 'y': my, 'w': 10, 'h': 10},
                {'type': 'text', 'x': tx, 'y': my + 1, 'text': MODE_TEXT[k], 'color': '#404040'}]})
        widgets.append({'type': 'button', 'x': mx, 'y': my, 'w': 10, 'h': 10, 'id': bid, 'tooltip': 'Chemical dumping mode'})
    # the title and the "Inventory" label as the machine's Gui class draws them (GuiMekanism.renderTitleTextWithOffset):
    # centred (or left) between two x, and the label only on some
    tx0, tx1, align = L.get('title', (0, 176, 'center'))
    title_x = tx0 + (tx1 - tx0 - text_width(title)) // 2 if align == 'center' else tx0
    widgets.append({'type': 'text', 'x': title_x, 'y': L.get('title_y', 6), 'text': title, 'color': '#404040'})
    if L.get('inventory', 176) is not None:
        widgets.append({'type': 'text', 'x': 8, 'y': L.get('label_y', 72), 'text': 'Inventory', 'color': '#404040'})
    # a development line: the machine's status number and the mod's version
    dx, dy = L.get('debug', (92, 66))
    widgets.append({'type': 'text', 'x': dx, 'y': dy, 'text': 'Status {5} v0.1.86', 'color': '#808080'})
    ctl = 24 if continuous else 20
    widgets += redstone_widgets(ctl) + energy_tab_widgets(ctl, MACHINE_USAGE[m])
    widgets += [
        # GuiSideConfigurationTab at -26, 6: opens the side configuration window
        {'type': 'sprite', 'texture': tab_config, 'x': -26, 'y': 6, 'w': 26, 'h': 26},
        {'type': 'button', 'x': -26, 'y': 6, 'w': 26, 'h': 26, 'local': {'open': 'side_config'}, 'tooltip': 'Configuration'}]
    return with_upgrades({'slots': len(L['slots']), 'data_slots': (24 if continuous else 20) + 3, 'results': L['results'],
            'screen': {'texture': process_panel(m, L), 'own_labels': True, 'slots': L['slots'], 'widgets': widgets,
                       'height': L.get('height', 166), 'inventory_y': 84 + L.get('inv_shift', 0),
                       'windows': [{'id': 'side_config', 'x': WIN_X, 'y': WIN_Y, 'width': WIN_W, 'height': WIN_H,
                                    'texture': window_tex, 'widgets': side_config_widgets(MACHINE_TRANSMISSIONS[m], m, ctl + 15)}]}}, m, len(L['slots']), ctl0)

# ---- the storage tanks (GuiFluidTank, GuiChemicalTank): the player's armor in a holder at the left, as the original
ARMOR_LEFT = {'x': -20, 'y': 67, 'offhand': 0}
EDIT_TAB = {}
def edit_mode_tab(mode):
    """GuiContainerEditModeTab at 176, 138 (26 by 18): the right holder in the tab's colour with the mode's icon."""
    holder = tinted(nine_slice(gui_sprite('holder_right'), 26, 18, 4), 0x366BD0)
    holder.alpha_composite(gui_sprite(f'container_edit_mode/{mode}'), (4, 0))
    return save_gui(holder, f'edit_mode_{mode}')

def tank_panel(name, slots, frames_extra):
    panel = nine_slice(gui_sprite('base'), 176, 166, 4)
    slot = gui_sprite('slot/normal')
    for row in range(3):
        for col in range(9):
            panel.paste(slot, (7 + 18 * col, 83 + 18 * row))
    for col in range(9):
        panel.paste(slot, (7 + 18 * col, 141))
    for fn in frames_extra:
        fn(panel)
    return save_gui(panel, name)

def armor_holder_left():
    """GuiSideHolder.armorHolder at -26, 62 (26 by 98, the left one): the five frames of the armor side bar inside."""
    holder = nine_slice(gui_sprite('holder_left'), 26, 98, 4)
    frame = gui_sprite('slot/normal')
    for k in range(5):
        holder.alpha_composite(frame, (3, 4 + 18 * k))
    return save_gui(holder, 'armor_holder_left')

def tank_menu(m):
    fluid = m in FLUID_TANKS
    title = MACHINE_NAMES[m]
    big = {'value': {'pair': [0, 1]}, 'max': {'pair': [3, 4]}, 'kind': {'data': 2}, 'types': GAUGE_TYPES}
    widgets = [{'type': 'sprite', 'texture': armor_holder_left(), 'x': -26, 'y': 62, 'w': 26, 'h': 98}]
    if fluid:
        def frames(panel):
            panel.alpha_composite(nine_slice(gui_sprite('gauge/normal'), 66, 50, 2), (48, 18))
            for x, y, overlay in ((146, 19, 'slot/overlay/input'), (146, 51, 'slot/overlay/output')):
                panel.alpha_composite(gui_sprite('slot/input' if overlay.endswith('input') else 'slot/output'), (x - 1, y - 1))
                panel.alpha_composite(gui_sprite(overlay), (x - 1, y - 1))
        texture = tank_panel(m, None, [frames])
        slots, results = [[146, 19], [146, 51]], [1]
        widgets += [dict({'type': 'gauge', 'x': 49, 'y': 19, 'w': 64, 'h': 48, 'tooltip': '{label}: {value} mB', 'empty': 'Empty'}, **big),
                    {'type': 'sprite', 'texture': copy_gui('gauge/overlay/wide', 'gauge_overlay_wide'), 'x': 49, 'y': 19, 'w': 64, 'h': 48}]
        # GuiContainerEditModeTab
        for k, mode in enumerate(['both', 'fill', 'empty']):
            widgets.append({'type': 'show', 'when': {'data': {'value': {'data': 5}, 'eq': k}}, 'widgets': [
                {'type': 'sprite', 'texture': edit_mode_tab(mode), 'x': 176, 'y': 138, 'w': 26, 'h': 18}]})
        widgets.append({'type': 'button', 'x': 176, 'y': 138, 'w': 26, 'h': 18, 'id': 5100, 'alt_id': 5101, 'tooltip': 'Fill and empty / fill only / empty only'})
        data_slots = 6
        windows = []
    else:
        def frames(panel):
            panel.alpha_composite(nine_slice(gui_sprite('bar/base'), 118, 12, 1), (42, 16))
            panel.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), 118, 28, 3), (42, 37))
            for x, y, ty, overlay in ((16, 16, 'slot/output', 'slot/overlay/plus'), (16, 48, 'slot/input', 'slot/overlay/minus')):
                panel.alpha_composite(gui_sprite(ty), (x - 1, y - 1))
                panel.alpha_composite(gui_sprite(overlay), (x - 1, y - 1))
        texture = tank_panel(m, None, [frames])
        slots, results = [[16, 16], [16, 48]], [0]
        # GuiChemicalBar, horizontal, 116 by 10 inside its frame
        widgets += [dict({'type': 'gauge', 'x': 43, 'y': 17, 'w': 116, 'h': 10, 'direction': 'right', 'tooltip': '{label}: {value} mB', 'empty': 'Empty'}, **big)]
        # GuiInnerScreen: the chemical's name and how much of it
        cap = 'Infinite' if m.startswith('creative') else '{3+4}'
        widgets.append({'type': 'show', 'when': {'data': {'value': {'data': 2}, 'eq': 0}}, 'widgets': [
            {'type': 'text', 'x': 45, 'y': 40, 'text': 'Chemical: None', 'color': '#00CD00'},
            {'type': 'text', 'x': 45, 'y': 51, 'text': f'0 / {cap}', 'color': '#00CD00'}]})
        for i, ty in enumerate(GAUGE_TYPES, start=1):
            widgets.append({'type': 'show', 'when': {'data': {'value': {'data': 2}, 'eq': i}}, 'widgets': [
                {'type': 'text', 'x': 45, 'y': 40, 'text': 'Chemical: ' + ty['label'], 'color': '#00CD00'},
                {'type': 'text', 'x': 45, 'y': 51, 'text': ('Infinite' if m.startswith('creative') else '{0+1} / ' + cap), 'color': '#00CD00'}]})
        # GuiGasMode at 159, 72
        for k, (name, label) in enumerate([('idle', 'Idle'), ('dump', 'Dumping'), ('dump_excess', 'Dump excess')]):
            widgets.append({'type': 'show', 'when': {'data': {'value': {'data': 5}, 'eq': k}}, 'widgets': [
                {'type': 'sprite', 'texture': copy_gui(f'chemical_mode/{name}', f'mode_{name}'), 'x': 159, 'y': 72, 'w': 10, 'h': 10},
                {'type': 'text', 'x': 159 - 3 - text_width(label), 'y': 73, 'text': label, 'color': '#404040'}]})
        widgets.append({'type': 'button', 'x': 159, 'y': 72, 'w': 10, 'h': 10, 'id': 5000, 'tooltip': 'Chemical dumping mode'})
        # GuiSideConfigurationTab
        widgets += [{'type': 'sprite', 'texture': tab_config, 'x': -26, 'y': 6, 'w': 26, 'h': 26},
                    {'type': 'button', 'x': -26, 'y': 6, 'w': 26, 'h': 26, 'local': {'open': 'side_config'}, 'tooltip': 'Configuration'}]
        data_slots = 16
        windows = [{'id': 'side_config', 'x': WIN_X, 'y': WIN_Y, 'width': WIN_W, 'height': WIN_H, 'texture': window_tex,
                    'widgets': side_config_widgets(MACHINE_TRANSMISSIONS[m], m)}]
    widgets += [{'type': 'text', 'x': (176 - text_width(title)) // 2, 'y': 6, 'text': title, 'color': '#404040'},
                {'type': 'text', 'x': 8, 'y': 72, 'text': 'Inventory', 'color': '#404040'},
                # a development line with the mod's version (clear of the dump mode's text)
                {'type': 'text', 'x': 128 if fluid else 50, 'y': 72 if fluid else 66, 'text': 'v0.1.86', 'color': '#808080'}]
    return {'slots': 2, 'data_slots': data_slots, 'results': results, 'armor': ARMOR_LEFT,
            'screen': {'texture': texture, 'own_labels': True, 'slots': slots, 'widgets': widgets, 'windows': windows}}


def shift_x(widgets, dx):
    """The widgets moved `dx` to the right (the ultimate factory's screen is wider than the others')."""
    out = []
    for w in widgets:
        w = dict(w)
        if 'x' in w and 'widgets' not in w:
            w['x'] += dx
        if 'widgets' in w:
            w['widgets'] = shift_x(w['widgets'], dx)
        out.append(w)
    return out

def factory_geometry(tier):
    """Where the lines' slots are (TileEntityItemToItemFactory.addSlots): the first one's x and the distance between two."""
    return {'basic': (55, 38), 'advanced': (35, 26), 'elite': (29, 19), 'ultimate': (27, 19)}[tier]

def factory_dims(tier, form):
    """GuiFactory's size and where its player inventory starts: 176 by 166 (34 wider for the ultimate one, 21 higher for sawing)."""
    width = 210 if tier == 'ultimate' else 176
    height = 187 if form == 2 else 177 if form >= 3 else 166
    inv_y = 105 if form == 2 else 95 if form >= 3 else 85
    inv_x = 26 if tier == 'ultimate' else 8
    return width, height, inv_x, inv_y

def factory_panel(name, tier, form):
    """GuiFactory: the lines' input and output slots, the charge slot, the empty arrows going down between them, the energy bar's
    frame and the player's inventory in the middle."""
    width, height, inv_x, inv_y = factory_dims(tier, form)
    lines = FACTORY_LINES[tier]
    base_x, mult = factory_geometry(tier)
    panel = nine_slice(gui_sprite('base'), width, height, 4)
    slot = gui_sprite('slot/normal')
    for row in range(3):
        for col in range(9):
            panel.paste(slot, (inv_x - 1 + 18 * col, inv_y - 1 + 18 * row))
    for col in range(9):
        panel.paste(slot, (inv_x - 1 + 18 * col, inv_y + 57))
    panel.alpha_composite(gui_sprite('slot/power'), (6, 12))
    if form == 1 or form >= 3:
        panel.alpha_composite(gui_sprite('slot/extra'), (6, 56))
    if form >= 3:
        # the chemical item slot's overlay and the tank's bar under the lines, 138 long (172 for the ultimate one)
        panel.alpha_composite(gui_sprite('slot/overlay/plus'), (6, 56))
        panel.alpha_composite(nine_slice(gui_sprite('bar/base'), (172 if tier == 'ultimate' else 138) + 2, 6, 1), (7, 76))
    for i in range(lines):
        x = base_x + i * mult
        panel.alpha_composite(gui_sprite('slot/input'), (x - 1, 12))
        panel.alpha_composite(gui_sprite('slot/output'), (x - 1, 56))
        if form == 2:
            panel.alpha_composite(gui_sprite('slot/output'), (x - 1, 76))
        panel.alpha_composite(gui_sprite('progress/down/empty'), (4 + x, 33))
    bar = 73 if form == 2 else 52
    panel.alpha_composite(nine_slice(gui_sprite('bar/base'), 6, bar + 2, 1), (width - 12, 16))
    return save_gui(panel, name), width

progress_down_full = copy_gui('progress/down/full', 'progress_down_full')

def factory_menu(m):
    tier, kind = FACTORIES[m]
    form = FACTORY_FORM[kind]
    lines = FACTORY_LINES[tier]
    base_x, mult = factory_geometry(tier)
    texture, width = factory_panel(m, tier, form)
    _w, height, inv_x, inv_y = factory_dims(tier, form)
    ox = (width - 176) // 2
    title = MACHINE_NAMES[m]
    slots, results = [], []
    for i in range(lines):
        x = base_x + i * mult
        results.append(len(slots) + 1)
        slots += [[x, 13], [x, 57]]
        if form == 2:
            results.append(len(slots))
            slots.append([x, 77])
    if form == 1 or form >= 3:
        slots.append([7, 57])
    charge_at = len(slots)
    slots.append([7, 13])
    bar = 73 if form == 2 else 52
    widgets = [{'type': 'sprite', 'texture': power_fill, 'x': width - 11, 'y': 17, 'w': 4, 'h': bar,
                'value': {'pair': [1, 2]}, 'max': {'pair': [3, 4]}, 'direction': 'up', 'tooltip': 'Energy: {value} / {max}'}]
    for i in range(lines):
        # ProgressType.DOWN of each line: its progress is the screen's number after the upgrade window's, its length the ticks
        widgets.append({'type': 'sprite', 'texture': progress_down_full, 'x': 4 + base_x + i * mult, 'y': 33, 'w': 8, 'h': 20,
                        'value': {'data': 32 + i}, 'max': {'data': 19}, 'direction': 'down', 'tooltip': 'Progress: {percent}%'})
    label_y = height - 94 + (2 if form >= 2 else 3)
    if form >= 3:
        # GuiChemicalBar at 7, 76 and the dump button after it: the tank's amount and chemical are the screen's numbers after the lines'
        bar_w = 172 if tier == 'ultimate' else 138
        widgets.append({'type': 'gauge', 'x': 8, 'y': 77, 'w': bar_w, 'h': 4, 'value': {'data': 32 + lines}, 'max': {'const': FACTORY_TANK[kind] * lines},
                        'kind': {'data': 33 + lines}, 'types': GAUGE_TYPES, 'direction': 'right', 'tooltip': '{label}: {value} mB', 'empty': 'Empty'})
        dump_x = 182 if tier == 'ultimate' else 148
        widgets.append({'type': 'sprite', 'texture': copy_gui('button/dump', 'button_dump'), 'x': dump_x, 'y': 76, 'w': 21, 'h': 10})
        widgets.append({'type': 'button', 'x': dump_x, 'y': 76, 'w': 21, 'h': 10, 'id': 4000, 'tooltip': 'Dump'})
    widgets += [
        {'type': 'text', 'x': (width - text_width(title)) // 2, 'y': 4, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': inv_x, 'y': label_y, 'text': 'Inventory', 'color': '#404040'},
        {'type': 'text', 'x': width - 70, 'y': label_y, 'text': 'v0.1.86', 'color': '#808080'},
        {'type': 'sprite', 'texture': tab_config, 'x': -26, 'y': 6, 'w': 26, 'h': 26},
        {'type': 'button', 'x': -26, 'y': 6, 'w': 26, 'h': 26, 'local': {'open': 'side_config'}, 'tooltip': 'Configuration'}]
    widgets += shift_x(redstone_widgets(16), width - 176) + energy_tab_widgets(16, MACHINE_USAGE[m])
    window = {'id': 'side_config', 'x': WIN_X + ox, 'y': WIN_Y, 'width': WIN_W, 'height': WIN_H,
              'texture': window_tex, 'widgets': side_config_widgets(MACHINE_TRANSMISSIONS[m], m, 31)}
    menu = with_upgrades({
        'slots': len(slots), 'data_slots': 32 + lines + (2 if form >= 3 else 0), 'results': results,
        'screen': {'texture': texture, 'own_labels': True, 'slots': slots, 'width': width, 'height': height,
                   'inventory_x': inv_x, 'inventory_y': inv_y, 'widgets': widgets, 'windows': [window]}}, m, len(slots), 16)
    menu['data_slots'] = 32 + lines + (2 if form >= 3 else 0)
    # the upgrade tab and window go with the wider screen, and the charge slot takes what turns into energy
    menu['screen']['widgets'] = menu['screen']['widgets'][:-len(upgrade_tab_widgets())] + shift_x(upgrade_tab_widgets(), width - 176)
    menu['screen']['windows'][-1]['x'] += ox
    menu['accepts'] = menu.get('accepts', []) + [{'slot': charge_at, 'items': ['minecraft:redstone', 'minecraft:redstone_block']}]
    return menu

def dynamic_valve_menu(m):
    """GuiDynamicTank: the structure's tank in a wide gauge, its bucket slots at 146, 19 and 146, 51, and what it holds, its capacity and its volume."""
    def frames(panel):
        panel.alpha_composite(nine_slice(gui_sprite('gauge/normal'), 66, 50, 2), (48, 18))
        for x, y, overlay in ((146, 19, 'slot/overlay/input'), (146, 51, 'slot/overlay/output')):
            panel.alpha_composite(gui_sprite('slot/input' if overlay.endswith('input') else 'slot/output'), (x - 1, y - 1))
            panel.alpha_composite(gui_sprite(overlay), (x - 1, y - 1))
    texture = tank_panel(m, None, [frames])
    title = 'Dynamic Tank'
    widgets = [
        {'type': 'gauge', 'x': 49, 'y': 19, 'w': 64, 'h': 48, 'value': {'pair': [0, 1]}, 'max': {'pair': [3, 4]}, 'kind': {'data': 2},
         'types': GAUGE_TYPES, 'tooltip': '{label}: {value} B', 'empty': 'Empty'},
        {'type': 'sprite', 'texture': copy_gui('gauge/overlay/wide', 'gauge_overlay_wide'), 'x': 49, 'y': 19, 'w': 64, 'h': 48},
        {'type': 'text', 'x': (176 - text_width(title)) // 2, 'y': 6, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': 8, 'y': 72, 'text': 'Inventory', 'color': '#404040'},
        {'type': 'text', 'x': 8, 'y': 20, 'text': '{0+1} B', 'color': '#00CD00', 'scale': 80},
        {'type': 'text', 'x': 8, 'y': 30, 'text': 'of {3+4}', 'color': '#00CD00', 'scale': 80},
        {'type': 'text', 'x': 8, 'y': 40, 'text': 'Volume {5+6}', 'color': '#00CD00', 'scale': 80},
        {'type': 'text', 'x': 128, 'y': 72, 'text': 'v0.1.86', 'color': '#808080'}]
    return {'slots': 2, 'data_slots': 7, 'results': [1],
            'screen': {'texture': texture, 'own_labels': True, 'slots': [[146, 19], [146, 51]], 'widgets': widgets}}

def evaporation_menu(m):
    """GuiThermalEvaporationController (simplified): the input and output gauges with their bucket slots, and the heat and speed."""
    panel = nine_slice(gui_sprite('base'), 176, 166, 4)
    slot = gui_sprite('slot/normal')
    for row in range(3):
        for col in range(9):
            panel.paste(slot, (7 + 18 * col, 84 + 18 * row))
    for col in range(9):
        panel.paste(slot, (7 + 18 * col, 142))
    for x in (7, 133):
        panel.alpha_composite(nine_slice(gui_sprite('gauge/normal'), 36, 50, 2), (x, 18))
    for x, y, kind in ((54, 19, 'input'), (54, 51, 'output'), (118, 19, 'input'), (118, 51, 'output')):
        panel.alpha_composite(gui_sprite(f'slot/{kind}'), (x - 1, y - 1))
        panel.alpha_composite(gui_sprite(f'slot/overlay/{kind}'), (x - 1, y - 1))
    texture = save_gui(panel, m)
    title = 'Thermal Evaporation Plant'
    widgets = [
        {'type': 'gauge', 'x': 8, 'y': 19, 'w': 34, 'h': 48, 'value': {'pair': [0, 1]}, 'max': {'pair': [2, 3]}, 'kind': {'data': 6},
         'types': GAUGE_TYPES, 'tooltip': '{label}: {value} mB', 'empty': 'Empty'},
        {'type': 'gauge', 'x': 134, 'y': 19, 'w': 34, 'h': 48, 'value': {'pair': [4, 5]}, 'max': {'const': 10000}, 'kind': {'data': 7},
         'types': GAUGE_TYPES, 'tooltip': '{label}: {value} mB', 'empty': 'Empty'},
        {'type': 'text', 'x': (176 - text_width(title)) // 2, 'y': 5, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': 8, 'y': 73, 'text': 'Inventory', 'color': '#404040'},
        {'type': 'text', 'x': 76, 'y': 20, 'text': 'Temp {8}', 'color': '#00CD00', 'scale': 80},
        {'type': 'text', 'x': 76, 'y': 30, 'text': 'Height {11}', 'color': '#00CD00', 'scale': 80},
        {'type': 'text', 'x': 76, 'y': 40, 'text': 'Sun {9}/4', 'color': '#00CD00', 'scale': 80},
        {'type': 'text', 'x': 76, 'y': 50, 'text': 'Rate {10}', 'color': '#00CD00', 'scale': 80},
        {'type': 'text', 'x': 128, 'y': 73, 'text': 'v0.1.86', 'color': '#808080'}]
    return {'slots': 4, 'data_slots': 12, 'results': [1, 3],
            'screen': {'texture': texture, 'own_labels': True, 'slots': [[54, 19], [54, 51], [118, 19], [118, 51]], 'widgets': widgets}}

def boiler_valve_menu(m):
    """GuiThermoelectricBoiler (simplified): four gauges (water, steam, heated and cooled coolant), the water's bucket slots, the heat and the boiling."""
    panel = nine_slice(gui_sprite('base'), 176, 166, 4)
    slot = gui_sprite('slot/normal')
    for row in range(3):
        for col in range(9):
            panel.paste(slot, (7 + 18 * col, 84 + 18 * row))
    for col in range(9):
        panel.paste(slot, (7 + 18 * col, 142))
    for x in (7, 43, 99, 135):
        panel.alpha_composite(nine_slice(gui_sprite('gauge/normal'), 26, 50, 2), (x, 18))
    for x, y, kind in ((73, 19, 'input'), (73, 51, 'output')):
        panel.alpha_composite(gui_sprite(f'slot/{kind}'), (x - 1, y - 1))
        panel.alpha_composite(gui_sprite(f'slot/overlay/{kind}'), (x - 1, y - 1))
    texture = save_gui(panel, m)
    title = 'Thermoelectric Boiler'
    widgets = []
    # water, steam, heated coolant, cooled coolant: data 0..15 in fours (amount halves, capacity halves), kinds at 16..19
    for i, x in enumerate((8, 44, 100, 136)):
        widgets.append({'type': 'gauge', 'x': x, 'y': 19, 'w': 24, 'h': 48, 'value': {'pair': [4 * i, 4 * i + 1]}, 'max': {'pair': [4 * i + 2, 4 * i + 3]},
                        'kind': {'data': 16 + i}, 'types': GAUGE_TYPES, 'tooltip': '{label}: {value} mB', 'empty': 'Empty'})
    widgets += [
        {'type': 'text', 'x': (176 - text_width(title)) // 2, 'y': 5, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': 8, 'y': 73, 'text': 'Inventory', 'color': '#404040'},
        {'type': 'text', 'x': 74, 'y': 39, 'text': '', 'color': '#00CD00'},
        {'type': 'text', 'x': 34, 'y': 70, 'text': 'T {20}  Boil {21}/{22}  SH {23}', 'color': '#00CD00', 'scale': 70},
        {'type': 'text', 'x': 128, 'y': 73, 'text': 'v0.1.86', 'color': '#808080'}]
    return {'slots': 2, 'data_slots': 25, 'results': [1],
            'screen': {'texture': texture, 'own_labels': True, 'slots': [[73, 19], [73, 51]], 'widgets': widgets}}

def turbine_valve_menu(m):
    """GuiIndustrialTurbine (simplified): the energy bar, the steam gauge and what the turbine is made of and does."""
    panel = nine_slice(gui_sprite('base'), 176, 166, 4)
    slot = gui_sprite('slot/normal')
    for row in range(3):
        for col in range(9):
            panel.paste(slot, (7 + 18 * col, 84 + 18 * row))
    for col in range(9):
        panel.paste(slot, (7 + 18 * col, 142))
    panel.alpha_composite(nine_slice(gui_sprite('bar/base'), 162, 12, 1), (7, 17))
    panel.alpha_composite(nine_slice(gui_sprite('gauge/normal'), 26, 50, 2), (143, 30))
    panel.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), 130, 50, 3), (7, 30))
    texture = save_gui(panel, m)
    title = 'Industrial Turbine'
    widgets = [
        {'type': 'sprite', 'texture': power_h_fill, 'x': 8, 'y': 18, 'w': 160, 'h': 10, 'value': {'pair': [0, 1]}, 'max': {'pair': [3, 4]},
         'direction': 'right', 'tooltip': 'Energy: {value} / {max} FE'},
        {'type': 'gauge', 'x': 144, 'y': 31, 'w': 24, 'h': 48, 'value': {'pair': [6, 7]}, 'max': {'pair': [8, 9]}, 'kind': {'data': 10},
         'types': GAUGE_TYPES, 'tooltip': '{label}: {value} mB', 'empty': 'Empty'},
        {'type': 'text', 'x': (176 - text_width(title)) // 2, 'y': 5, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': 8, 'y': 73, 'text': 'Inventory', 'color': '#404040'},
        {'type': 'text', 'x': 10, 'y': 33, 'text': 'Production {11}/{12}', 'color': '#00CD00', 'scale': 80},
        {'type': 'text', 'x': 10, 'y': 42, 'text': 'Flow {13}/{14}', 'color': '#00CD00', 'scale': 80},
        {'type': 'text', 'x': 10, 'y': 51, 'text': 'Blades {17}  Coils {18}', 'color': '#00CD00', 'scale': 80},
        {'type': 'text', 'x': 10, 'y': 60, 'text': 'Vents {16}  Condensers {19}', 'color': '#00CD00', 'scale': 80},
        {'type': 'text', 'x': 10, 'y': 69, 'text': 'Dispersers {15}', 'color': '#00CD00', 'scale': 80},
        {'type': 'text', 'x': 128, 'y': 73, 'text': 'v0.1.86', 'color': '#808080'}]
    return {'slots': 1, 'data_slots': 21,
            'screen': {'texture': texture, 'own_labels': True, 'slots': [None], 'widgets': widgets}}

def fission_port_menu(m):
    """GuiFissionReactor (simplified): fuel, coolant, heated coolant and waste gauges, the burn rate with its buttons, and the on/off button."""
    panel = nine_slice(gui_sprite('base'), 176, 166, 4)
    slot = gui_sprite('slot/normal')
    for row in range(3):
        for col in range(9):
            panel.paste(slot, (7 + 18 * col, 84 + 18 * row))
    for col in range(9):
        panel.paste(slot, (7 + 18 * col, 142))
    for x in (7, 35, 117, 145):
        panel.alpha_composite(nine_slice(gui_sprite('gauge/normal'), 24, 50, 2), (x, 18))
    panel.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), 76, 50, 3), (36 + 28, 18))
    for x, w in ((64, 20), (86, 20), (108, 20)):
        pass
    for x, y, w in ((64, 70, 22), (88, 70, 22), (112, 70, 22), (136, 70, 22)):
        pass
    texture = save_gui(panel, m)
    title = 'Fission Reactor'
    widgets = []
    for i, x in enumerate((8, 36, 118, 146)):
        widgets.append({'type': 'gauge', 'x': x, 'y': 19, 'w': 22, 'h': 48, 'value': {'pair': [4 * i, 4 * i + 1]}, 'max': {'pair': [4 * i + 2, 4 * i + 3]},
                        'kind': {'data': 16 + i}, 'types': GAUGE_TYPES, 'tooltip': '{label}: {value} mB', 'empty': 'Empty'})
    widgets += [
        {'type': 'text', 'x': (176 - text_width(title)) // 2, 'y': 5, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': 8, 'y': 73, 'text': 'Inventory', 'color': '#404040'},
        {'type': 'text', 'x': 66, 'y': 21, 'text': 'Temp {20}/10 K', 'color': '#00CD00', 'scale': 75},
        {'type': 'text', 'x': 66, 'y': 30, 'text': 'Damage {21}%', 'color': '#00CD00', 'scale': 75},
        {'type': 'text', 'x': 66, 'y': 39, 'text': 'Rate {22}/100', 'color': '#00CD00', 'scale': 75},
        {'type': 'text', 'x': 66, 'y': 48, 'text': 'Burn {23}/100  Max {24}', 'color': '#00CD00', 'scale': 75},
        {'type': 'text', 'x': 66, 'y': 57, 'text': 'Boil {25}  Eff {28}%', 'color': '#00CD00', 'scale': 75},
        {'type': 'text', 'x': 66, 'y': 66, 'text': 'Assemblies {27}', 'color': '#00CD00', 'scale': 75},
        {'type': 'text', 'x': 128, 'y': 73, 'text': 'v0.1.86', 'color': '#808080'}]
    # the buttons: on/off and the burn rate
    for k, (bid, label, x) in enumerate([(7101, '-1', 8), (7102, '-.1', 34), (7103, '+.1', 118), (7104, '+1', 144)]):
        widgets.append({'type': 'text', 'x': x + 3, 'y': 71, 'text': label, 'color': '#404040'})
        widgets.append({'type': 'button', 'x': x, 'y': 69, 'w': 22, 'h': 12, 'id': bid, 'tooltip': 'Burn rate'})
    widgets.append({'type': 'show', 'when': {'data': {'value': {'data': 26}, 'eq': 1}}, 'widgets': [
        {'type': 'text', 'x': 66, 'y': 72, 'text': 'ON - click to SCRAM', 'color': '#00CD00', 'scale': 70}]})
    widgets.append({'type': 'show', 'when': {'data': {'value': {'data': 26}, 'eq': 0}}, 'widgets': [
        {'type': 'text', 'x': 66, 'y': 72, 'text': 'OFF - click to start', 'color': '#CD0000', 'scale': 70}]})
    widgets.append({'type': 'show', 'when': {'data': {'value': {'data': 26}, 'eq': 2}}, 'widgets': [
        {'type': 'text', 'x': 66, 'y': 72, 'text': 'FORCED OFF', 'color': '#CD0000', 'scale': 70}]})
    widgets.append({'type': 'button', 'x': 60, 'y': 69, 'w': 56, 'h': 14, 'id': 7100, 'tooltip': 'Activate / Scram'})
    return {'slots': 1, 'data_slots': 30,
            'screen': {'texture': texture, 'own_labels': True, 'slots': [None], 'widgets': widgets}}

def fusion_controller_menu(m):
    """GuiFusionReactorController (simplified): the hohlraum slot, deuterium, tritium, fuel, water and steam gauges, the injection rate, the heat."""
    panel = nine_slice(gui_sprite('base'), 176, 166, 4)
    slot = gui_sprite('slot/normal')
    for row in range(3):
        for col in range(9):
            panel.paste(slot, (7 + 18 * col, 84 + 18 * row))
    for col in range(9):
        panel.paste(slot, (7 + 18 * col, 142))
    panel.paste(slot, (84, 38))
    for x in (7, 33, 59, 117, 143):
        panel.alpha_composite(nine_slice(gui_sprite('gauge/normal'), 24, 50, 2), (x, 18 if x in (7, 33, 59) else 18))
    texture = save_gui(panel, m)
    title = 'Fusion Reactor'
    widgets = []
    # deuterium, tritium, fuel (data 0..5, capacity 10), water, steam (6..9, capacity 12..15), kinds 16..20
    for x, (v, mx, kd) in zip((8, 34, 60, 118, 144), (([0, 1], {'const': 1000}, 16), ([2, 3], {'const': 1000}, 17), ([4, 5], {'const': 1000}, 18),
                                                         ([6, 7], {'pair': [12, 13]}, 19), ([8, 9], {'pair': [14, 15]}, 20))):
        widgets.append({'type': 'gauge', 'x': x, 'y': 19, 'w': 22, 'h': 48, 'value': {'pair': v}, 'max': mx,
                        'kind': {'data': kd}, 'types': GAUGE_TYPES, 'tooltip': '{label}: {value} mB', 'empty': 'Empty'})
    widgets += [
        {'type': 'text', 'x': (176 - text_width(title)) // 2, 'y': 5, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': 8, 'y': 73, 'text': 'Inventory', 'color': '#404040'},
        {'type': 'text', 'x': 86, 'y': 21, 'text': 'Hohlraum', 'color': '#404040', 'scale': 70},
        {'type': 'text', 'x': 62, 'y': 59, 'text': 'Plasma {21} kK  Case {22}/10 K', 'color': '#00CD00', 'scale': 60},
        {'type': 'text', 'x': 62, 'y': 67, 'text': 'Inject {23}  Burned {27}  +{28} FE/t', 'color': '#00CD00', 'scale': 60},
        {'type': 'text', 'x': 128, 'y': 73, 'text': 'v0.1.86', 'color': '#808080'}]
    widgets.append({'type': 'show', 'when': {'data': {'value': {'data': 24}, 'eq': 1}}, 'widgets': [
        {'type': 'text', 'x': 86, 'y': 58, 'text': 'BURNING', 'color': '#CD0000', 'scale': 70}]})
    for bid, label, x in ((7200, '-2', 88), (7201, '+2', 108)):
        widgets.append({'type': 'text', 'x': x + 3, 'y': 53, 'text': label, 'color': '#404040'})
        widgets.append({'type': 'button', 'x': x, 'y': 51, 'w': 18, 'h': 10, 'id': bid, 'tooltip': 'Injection rate'})
    return {'slots': 1, 'data_slots': 29, 'accepts': [{'slot': 0, 'items': ['mekanism:hohlraum']}],
            'screen': {'texture': texture, 'own_labels': True, 'slots': [[85, 39]], 'widgets': widgets}}

def laser_amplifier_menu(m):
    """GuiLaserAmplifier (simplified): the energy it holds, the thresholds and the delay with buttons."""
    panel = nine_slice(gui_sprite('base'), 176, 166, 4)
    slot = gui_sprite('slot/normal')
    for row in range(3):
        for col in range(9):
            panel.paste(slot, (7 + 18 * col, 84 + 18 * row))
    for col in range(9):
        panel.paste(slot, (7 + 18 * col, 142))
    panel.alpha_composite(nine_slice(gui_sprite('bar/base'), 162, 12, 1), (7, 17))
    texture = save_gui(panel, m)
    title = 'Laser Amplifier'
    widgets = [
        {'type': 'sprite', 'texture': power_h_fill, 'x': 8, 'y': 18, 'w': 160, 'h': 10, 'value': {'pair': [0, 1]}, 'max': {'pair': [2, 3]},
         'direction': 'right', 'tooltip': 'Energy: {value} / {max} FE'},
        {'type': 'text', 'x': (176 - text_width(title)) // 2, 'y': 5, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': 8, 'y': 73, 'text': 'Inventory', 'color': '#404040'},
        {'type': 'text', 'x': 10, 'y': 33, 'text': 'Min {4+5}', 'color': '#00CD00', 'scale': 90},
        {'type': 'text', 'x': 10, 'y': 47, 'text': 'Max {6+7}', 'color': '#00CD00', 'scale': 90},
        {'type': 'text', 'x': 10, 'y': 61, 'text': 'Delay {8}  Firing {9}', 'color': '#00CD00', 'scale': 90},
        {'type': 'text', 'x': 128, 'y': 73, 'text': 'v0.1.86', 'color': '#808080'}]
    for row, (y, ids) in enumerate(((31, (7300, 7301, 7302, 7303)), (45, (7304, 7305, 7306, 7307)))):
        for k, (bid, label) in enumerate(zip(ids, ('-100k', '-1k', '+1k', '+100k'))):
            x = 86 + 21 * k
            widgets.append({'type': 'text', 'x': x + 1, 'y': y + 2, 'text': label, 'color': '#404040', 'scale': 60})
            widgets.append({'type': 'button', 'x': x, 'y': y, 'w': 20, 'h': 11, 'id': bid, 'tooltip': 'Threshold'})
    for bid, label, x in ((7308, '-1', 112), (7309, '+1', 134)):
        widgets.append({'type': 'text', 'x': x + 4, 'y': 62, 'text': label, 'color': '#404040'})
        widgets.append({'type': 'button', 'x': x, 'y': 59, 'w': 20, 'h': 11, 'id': bid, 'tooltip': 'Delay'})
    return {'slots': 1, 'data_slots': 10,
            'screen': {'texture': texture, 'own_labels': True, 'slots': [None], 'widgets': widgets}}

def sps_port_menu(m):
    """GuiSPS (simplified): polonium and antimatter gauges, the progress toward the next antimatter and the energy the coils got."""
    panel = nine_slice(gui_sprite('base'), 176, 166, 4)
    slot = gui_sprite('slot/normal')
    for row in range(3):
        for col in range(9):
            panel.paste(slot, (7 + 18 * col, 84 + 18 * row))
    for col in range(9):
        panel.paste(slot, (7 + 18 * col, 142))
    for x in (7, 143):
        panel.alpha_composite(nine_slice(gui_sprite('gauge/normal'), 26, 50, 2), (x, 18))
    texture = save_gui(panel, m)
    title = 'Supercritical Phase Shifter'
    widgets = [
        {'type': 'gauge', 'x': 8, 'y': 19, 'w': 24, 'h': 48, 'value': {'pair': [0, 1]}, 'max': {'pair': [2, 3]}, 'kind': {'data': 8},
         'types': GAUGE_TYPES, 'tooltip': '{label}: {value} mB', 'empty': 'Empty'},
        {'type': 'gauge', 'x': 144, 'y': 19, 'w': 24, 'h': 48, 'value': {'pair': [4, 5]}, 'max': {'pair': [6, 7]}, 'kind': {'data': 9},
         'types': GAUGE_TYPES, 'tooltip': '{label}: {value} mB', 'empty': 'Empty'},
        {'type': 'text', 'x': (176 - text_width(title)) // 2, 'y': 5, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': 8, 'y': 73, 'text': 'Inventory', 'color': '#404040'},
        {'type': 'text', 'x': 40, 'y': 25, 'text': 'Progress {10}%', 'color': '#00CD00', 'scale': 85},
        {'type': 'text', 'x': 40, 'y': 37, 'text': 'Energy {11+12} FE/t', 'color': '#00CD00', 'scale': 85},
        {'type': 'text', 'x': 40, 'y': 49, 'text': 'Coils {13}', 'color': '#00CD00', 'scale': 85},
        {'type': 'text', 'x': 128, 'y': 73, 'text': 'v0.1.86', 'color': '#808080'}]
    return {'slots': 1, 'data_slots': 15,
            'screen': {'texture': texture, 'own_labels': True, 'slots': [None], 'widgets': widgets}}

def sorter_menu(m):
    """GuiLogisticalSorter (simplified): five filter slots (an item each), and the auto-eject and single-item switches."""
    panel = nine_slice(gui_sprite('base'), 176, 166, 4)
    slot = gui_sprite('slot/normal')
    for row in range(3):
        for col in range(9):
            panel.paste(slot, (7 + 18 * col, 84 + 18 * row))
    for col in range(9):
        panel.paste(slot, (7 + 18 * col, 142))
    xs = [44 + 18 * i for i in range(5)]
    for x in xs:
        panel.paste(slot, (x - 1, 23))
    panel.alpha_composite(nine_slice(gui_sprite('button'), 70, 14, 2), (8, 50))
    panel.alpha_composite(nine_slice(gui_sprite('button'), 70, 14, 2), (98, 50))
    texture = save_gui(panel, m)
    title = 'Logistical Sorter'
    widgets = [
        {'type': 'text', 'x': (176 - text_width(title)) // 2, 'y': 5, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': 8, 'y': 73, 'text': 'Inventory', 'color': '#404040'},
        {'type': 'text', 'x': 8, 'y': 27, 'text': 'Filters', 'color': '#404040'},
        {'type': 'text', 'x': 128, 'y': 73, 'text': 'v0.1.86', 'color': '#808080'}]
    for bid, x, label, data in ((7400, 8, 'Auto-eject', 0), (7401, 98, 'Single item', 1)):
        widgets.append({'type': 'text', 'x': x + 4, 'y': 53, 'text': label, 'color': '#404040'})
        widgets.append({'type': 'show', 'when': {'data': {'value': {'data': data}, 'eq': 1}}, 'widgets': [
            {'type': 'text', 'x': x + 52, 'y': 53, 'text': 'ON', 'color': '#00CD00'}]})
        widgets.append({'type': 'show', 'when': {'data': {'value': {'data': data}, 'eq': 0}}, 'widgets': [
            {'type': 'text', 'x': x + 52, 'y': 53, 'text': 'off', 'color': '#CD0000'}]})
        widgets.append({'type': 'button', 'x': x, 'y': 50, 'w': 70, 'h': 14, 'id': bid, 'tooltip': label})
    return {'slots': 7, 'data_slots': 2,
            'screen': {'texture': texture, 'own_labels': True, 'slots': [None, None] + [[x, 24] for x in xs], 'widgets': widgets}}

def entangloporter_menu(m):
    """GuiQuantumEntangloporter (simplified): the frequency number with its buttons, the shared item slot and what the frequency holds."""
    panel = nine_slice(gui_sprite('base'), 176, 166, 4)
    slot = gui_sprite('slot/normal')
    for row in range(3):
        for col in range(9):
            panel.paste(slot, (7 + 18 * col, 84 + 18 * row))
    for col in range(9):
        panel.paste(slot, (7 + 18 * col, 142))
    panel.paste(slot, (151, 17))
    panel.alpha_composite(nine_slice(gui_sprite('button'), 20, 14, 2), (8, 17))
    panel.alpha_composite(nine_slice(gui_sprite('button'), 20, 14, 2), (74, 17))
    panel.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), 160, 44, 3), (8, 36))
    texture = save_gui(panel, m)
    title = 'Quantum Entangloporter'
    widgets = [
        {'type': 'text', 'x': (176 - text_width(title)) // 2, 'y': 5, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': 8, 'y': 73, 'text': 'Inventory', 'color': '#404040'},
        {'type': 'text', 'x': 14, 'y': 20, 'text': '-', 'color': '#404040'},
        {'type': 'text', 'x': 80, 'y': 20, 'text': '+', 'color': '#404040'},
        {'type': 'text', 'x': 34, 'y': 20, 'text': 'Frequency {0}', 'color': '#404040'},
        {'type': 'text', 'x': 12, 'y': 40, 'text': 'Energy {1+2} kFE', 'color': '#00CD00', 'scale': 85},
        {'type': 'text', 'x': 12, 'y': 50, 'text': 'Fluid {3+4} mB', 'color': '#00CD00', 'scale': 85},
        {'type': 'text', 'x': 12, 'y': 60, 'text': 'Chemical {5+6} mB', 'color': '#00CD00', 'scale': 85},
        {'type': 'text', 'x': 12, 'y': 70, 'text': 'Entangloporters {9}', 'color': '#00CD00', 'scale': 85},
        {'type': 'text', 'x': 128, 'y': 73, 'text': 'v0.1.86', 'color': '#808080'}]
    widgets.append({'type': 'button', 'x': 8, 'y': 17, 'w': 20, 'h': 14, 'id': 7500, 'tooltip': 'Previous frequency'})
    widgets.append({'type': 'button', 'x': 74, 'y': 17, 'w': 20, 'h': 14, 'id': 7501, 'tooltip': 'Next frequency'})
    return {'slots': 1, 'data_slots': 11,
            'screen': {'texture': texture, 'own_labels': True, 'slots': [[152, 18]], 'widgets': widgets}}

def teleporter_menu(m):
    """GuiTeleporter (simplified): the frequency number with its buttons, the energy bar, and the status."""
    panel = nine_slice(gui_sprite('base'), 176, 166, 4)
    slot = gui_sprite('slot/normal')
    for row in range(3):
        for col in range(9):
            panel.paste(slot, (7 + 18 * col, 84 + 18 * row))
    for col in range(9):
        panel.paste(slot, (7 + 18 * col, 142))
    panel.alpha_composite(nine_slice(gui_sprite('bar/base'), 162, 12, 1), (7, 17))
    panel.alpha_composite(nine_slice(gui_sprite('button'), 20, 14, 2), (8, 34))
    panel.alpha_composite(nine_slice(gui_sprite('button'), 20, 14, 2), (74, 34))
    panel.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), 160, 24, 3), (8, 52))
    texture = save_gui(panel, m)
    title = 'Teleporter'
    widgets = [
        {'type': 'sprite', 'texture': power_h_fill, 'x': 8, 'y': 18, 'w': 160, 'h': 10, 'value': {'pair': [2, 3]}, 'max': {'pair': [4, 5]},
         'direction': 'right', 'tooltip': 'Energy: {value} / {max} FE'},
        {'type': 'text', 'x': (176 - text_width(title)) // 2, 'y': 5, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': 8, 'y': 73, 'text': 'Inventory', 'color': '#404040'},
        {'type': 'text', 'x': 14, 'y': 37, 'text': '-', 'color': '#404040'},
        {'type': 'text', 'x': 80, 'y': 37, 'text': '+', 'color': '#404040'},
        {'type': 'text', 'x': 34, 'y': 37, 'text': 'Frequency {0}', 'color': '#404040'},
        {'type': 'text', 'x': 12, 'y': 66, 'text': 'Teleporters on it: {6}', 'color': '#404040', 'scale': 85},
        {'type': 'text', 'x': 128, 'y': 73, 'text': 'v0.1.86', 'color': '#808080'}]
    for st, text, color in ((0, 'No frequency', '#CD0000'), (1, 'No frame', '#CD0000'), (2, 'No link found', '#CD0000'),
                            (3, 'Not enough energy', '#CD0000'), (4, 'Ready', '#00CD00')):
        widgets.append({'type': 'show', 'when': {'data': {'value': {'data': 1}, 'eq': st}}, 'widgets': [
            {'type': 'text', 'x': 12, 'y': 58, 'text': text, 'color': color}]})
    widgets.append({'type': 'button', 'x': 8, 'y': 34, 'w': 20, 'h': 14, 'id': 7600, 'tooltip': 'Previous frequency'})
    widgets.append({'type': 'button', 'x': 74, 'y': 34, 'w': 20, 'h': 14, 'id': 7601, 'tooltip': 'Next frequency'})
    return {'slots': 1, 'data_slots': 7,
            'screen': {'texture': texture, 'own_labels': True, 'slots': [None], 'widgets': widgets}}

def bin_menu(m):
    """The bin's screen (it has none in the original: items are put in by using them and taken by punching): an input slot, the output
    stack, what it holds, and the switches for ejecting down and locking the item."""
    panel = nine_slice(gui_sprite('base'), 176, 166, 4)
    slot = gui_sprite('slot/normal')
    for row in range(3):
        for col in range(9):
            panel.paste(slot, (7 + 18 * col, 84 + 18 * row))
    for col in range(9):
        panel.paste(slot, (7 + 18 * col, 142))
    panel.paste(slot, (43, 33))
    panel.paste(slot, (115, 33))
    panel.alpha_composite(nine_slice(gui_sprite('button'), 70, 14, 2), (8, 56))
    panel.alpha_composite(nine_slice(gui_sprite('button'), 70, 14, 2), (98, 56))
    texture = save_gui(panel, m)
    title = m.replace('_', ' ').title()
    widgets = [
        {'type': 'text', 'x': (176 - text_width(title)) // 2, 'y': 5, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': 8, 'y': 73, 'text': 'Inventory', 'color': '#404040'},
        {'type': 'text', 'x': 38, 'y': 22, 'text': 'In', 'color': '#404040', 'scale': 80},
        {'type': 'text', 'x': 112, 'y': 22, 'text': 'Out', 'color': '#404040', 'scale': 80},
        {'type': 'text', 'x': 62, 'y': 38, 'text': '{0+1}', 'color': '#00CD00'},
        {'type': 'text', 'x': 62, 'y': 48, 'text': 'of {2+3}', 'color': '#00CD00', 'scale': 80},
        {'type': 'text', 'x': 128, 'y': 73, 'text': 'v0.1.86', 'color': '#808080'}]
    for bid, x, label, data in ((7700, 8, 'Eject down', 4), (7701, 98, 'Lock item', 5)):
        widgets.append({'type': 'text', 'x': x + 4, 'y': 59, 'text': label, 'color': '#404040'})
        widgets.append({'type': 'show', 'when': {'data': {'value': {'data': data}, 'eq': 1}}, 'widgets': [
            {'type': 'text', 'x': x + 54, 'y': 59, 'text': 'ON', 'color': '#00CD00'}]})
        widgets.append({'type': 'show', 'when': {'data': {'value': {'data': data}, 'eq': 0}}, 'widgets': [
            {'type': 'text', 'x': x + 54, 'y': 59, 'text': 'off', 'color': '#CD0000'}]})
        widgets.append({'type': 'button', 'x': x, 'y': 56, 'w': 70, 'h': 14, 'id': bid, 'tooltip': label})
    return {'slots': 2, 'data_slots': 6,
            'screen': {'texture': texture, 'own_labels': True, 'slots': [[116, 34], [44, 34]], 'widgets': widgets}}

def miner_menu(m):
    """GuiDigitalMiner (simplified): five filter slots, the 27 slots of what it mined, and the start button, radius, heights and switches."""
    H, INV = 238, 156
    panel = nine_slice(gui_sprite('base'), 176, H, 4)
    slot = gui_sprite('slot/normal')
    for row in range(3):
        for col in range(9):
            panel.paste(slot, (7 + 18 * col, INV - 1 + 18 * row))
    for col in range(9):
        panel.paste(slot, (7 + 18 * col, INV + 57))
    buffer = [(8 + 18 * (i % 9), 44 + 18 * (i // 9)) for i in range(27)]
    filters = [(8 + 18 * i, 20) for i in range(5)]
    for x, y in buffer + filters:
        panel.paste(slot, (x - 1, y - 1))
    panel.alpha_composite(nine_slice(gui_sprite('bar/base'), 160, 10, 1), (8, 100))
    texture = save_gui(panel, m)
    title = 'Digital Miner'
    widgets = [
        {'type': 'sprite', 'texture': power_h_fill, 'x': 9, 'y': 101, 'w': 158, 'h': 8, 'value': {'pair': [1, 2]}, 'max': {'pair': [3, 4]},
         'direction': 'right', 'tooltip': 'Energy: {value} / {max} FE'},
        {'type': 'text', 'x': (176 - text_width(title)) // 2, 'y': 5, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': 100, 'y': 24, 'text': 'Filters', 'color': '#404040', 'scale': 90},
        {'type': 'text', 'x': 8, 'y': 145, 'text': 'Inventory', 'color': '#404040'},
        {'type': 'text', 'x': 128, 'y': 145, 'text': 'v0.1.86', 'color': '#808080'},
        {'type': 'text', 'x': 66, 'y': 113, 'text': 'Radius {5}', 'color': '#00CD00', 'scale': 85},
        {'type': 'text', 'x': 66, 'y': 125, 'text': 'Min Y {6}', 'color': '#00CD00', 'scale': 85},
        {'type': 'text', 'x': 134, 'y': 125, 'text': 'Max Y {7}', 'color': '#00CD00', 'scale': 85},
        {'type': 'text', 'x': 100, 'y': 113, 'text': 'Queue {11}  Mined {12}', 'color': '#00CD00', 'scale': 85}]
    def button(x, y, w, label, bid, tip):
        widgets.append({'type': 'text', 'x': x + 2, 'y': y + 2, 'text': label, 'color': '#404040', 'scale': 85})
        widgets.append({'type': 'button', 'x': x, 'y': y, 'w': w, 'h': 11, 'id': bid, 'tooltip': tip})
        panel.alpha_composite(nine_slice(gui_sprite('button'), w, 11, 2), (x, y))
    # the buttons are painted into the texture after the widgets are listed, so build the texture again at the end
    button(8, 112, 50, 'Start/Stop', 7800, 'Start or stop mining')
    button(8, 124, 14, '-', 7801, 'Smaller radius')
    button(24, 124, 14, '+', 7802, 'Larger radius')
    button(8, 136, 14, '-', 7803, 'Lower minimum height')
    button(24, 136, 14, '+', 7804, 'Higher minimum height')
    button(42, 136, 14, '-', 7805, 'Lower maximum height')
    button(58, 136, 14, '+', 7806, 'Higher maximum height')
    for bid, x, label, data in ((7807, 80, 'Eject', 8), (7808, 114, 'Silk', 9), (7809, 144, 'Inv', 10)):
        widgets.append({'type': 'text', 'x': x + 2, 'y': 138, 'text': label, 'color': '#404040', 'scale': 80})
        widgets.append({'type': 'show', 'when': {'data': {'value': {'data': data}, 'eq': 1}}, 'widgets': [
            {'type': 'text', 'x': x + 2 + 20, 'y': 138, 'text': '+', 'color': '#00CD00', 'scale': 80}]})
        widgets.append({'type': 'button', 'x': x, 'y': 136, 'w': 30, 'h': 11, 'id': bid, 'tooltip': label})
    for st, text, color in ((0, 'Stopped', '#808080'), (1, 'Searching', '#00CD00'), (2, 'Mining', '#00CD00'), (3, 'No energy', '#CD0000'), (4, 'Finished', '#00CD00')):
        widgets.append({'type': 'show', 'when': {'data': {'value': {'data': 13}, 'eq': st}}, 'widgets': [
            {'type': 'text', 'x': 112, 'y': 5, 'text': text, 'color': color, 'scale': 85}]})
    texture = save_gui(panel, m)
    return {'slots': 32, 'data_slots': 14,
            'screen': {'texture': texture, 'own_labels': True, 'height': H, 'inventory_y': INV,
                       'slots': [list(p) for p in buffer] + [list(p) for p in filters], 'widgets': widgets}}

def induction_port_menu(m):
    """GuiMatrix (simplified): the energy the matrix holds (in MFE), its capacity, what its providers move and what it is made of."""
    panel = nine_slice(gui_sprite('base'), 176, 166, 4)
    slot = gui_sprite('slot/normal')
    for row in range(3):
        for col in range(9):
            panel.paste(slot, (7 + 18 * col, 83 + 18 * row))
    for col in range(9):
        panel.paste(slot, (7 + 18 * col, 141))
    panel.alpha_composite(nine_slice(gui_sprite('bar/base'), 162, 12, 1), (7, 17))
    panel.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), 162, 46, 3), (7, 33))
    texture = save_gui(panel, m)
    title = 'Induction Matrix'
    widgets = [
        {'type': 'sprite', 'texture': power_h_fill, 'x': 8, 'y': 18, 'w': 160, 'h': 10, 'value': {'pair': [0, 1]}, 'max': {'pair': [2, 3]},
         'direction': 'right', 'tooltip': 'Energy: {value} / {max} MFE'},
        {'type': 'text', 'x': (176 - text_width(title)) // 2, 'y': 5, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': 8, 'y': 72, 'text': 'Inventory', 'color': '#404040'},
        {'type': 'text', 'x': 11, 'y': 37, 'text': 'Stored: {0+1} MFE', 'color': '#00CD00'},
        {'type': 'text', 'x': 11, 'y': 48, 'text': 'Capacity: {2+3} MFE', 'color': '#00CD00'},
        {'type': 'text', 'x': 11, 'y': 59, 'text': 'Transfer: {4+5} kFE/t', 'color': '#00CD00'},
        {'type': 'text', 'x': 11, 'y': 70, 'text': 'Cells {6}  Providers {7}', 'color': '#00CD00'},
        {'type': 'text', 'x': 128, 'y': 72, 'text': 'v0.1.86', 'color': '#808080'}]
    return {'slots': 1, 'data_slots': 8,
            'screen': {'texture': texture, 'own_labels': True, 'slots': [None], 'widgets': widgets}}

def pump_menu(m):
    """GuiElectricPump: the fluid gauge at 6, 13, the bucket slots at 28, 20 and 28, 51 with an arrow between them, the inner screen
    with the fluid, the charge slot at 143, 35 and the energy bar at 164, 15."""
    panel = nine_slice(gui_sprite('base'), 176, 166, 4)
    slot = gui_sprite('slot/normal')
    for row in range(3):
        for col in range(9):
            panel.paste(slot, (7 + 18 * col, 83 + 18 * row))
    for col in range(9):
        panel.paste(slot, (7 + 18 * col, 141))
    panel.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), 80, 42, 3), (54, 23))
    for sprite, x, y in (('slot/input', 28, 20), ('slot/output', 28, 51), ('slot/power', 143, 35)):
        panel.alpha_composite(gui_sprite(sprite), (x - 1, y - 1))
    panel.alpha_composite(gui_sprite('arrow/down'), (32, 39))
    panel.alpha_composite(nine_slice(gui_sprite('gauge/normal'), 18, 60, 2), (6, 13))
    panel.alpha_composite(nine_slice(gui_sprite('bar/base'), 6, 54, 1), (164, 15))
    texture = save_gui(panel, m)
    title = MACHINE_NAMES[m]
    widgets = [
        {'type': 'sprite', 'texture': power_fill, 'x': 165, 'y': 16, 'w': 4, 'h': 52, 'value': {'pair': [1, 2]}, 'max': {'pair': [3, 4]},
         'direction': 'up', 'tooltip': 'Energy: {value} / {max}'},
        {'type': 'gauge', 'x': 7, 'y': 14, 'w': 16, 'h': 58, 'value': {'data': 16}, 'max': {'const': 10000}, 'kind': {'data': 17},
         'types': GAUGE_TYPES, 'tooltip': '{label}: {value} mB', 'empty': 'Empty'},
        {'type': 'sprite', 'texture': copy_gui('gauge/overlay/standard', 'gauge_overlay_standard'), 'x': 7, 'y': 14, 'w': 16, 'h': 58},
        {'type': 'text', 'x': (176 - text_width(title)) // 2, 'y': 5, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': 8, 'y': 74, 'text': 'Inventory', 'color': '#404040'},
        {'type': 'text', 'x': 128, 'y': 74, 'text': 'v0.1.86', 'color': '#808080'},
        {'type': 'text', 'x': 58, 'y': 27, 'text': '{16} mB', 'color': '#00CD00'},
        {'type': 'text', 'x': 58, 'y': 38, 'text': 'Status {5}', 'color': '#00CD00'},
        {'type': 'sprite', 'texture': tab_config, 'x': -26, 'y': 6, 'w': 26, 'h': 26},
        {'type': 'button', 'x': -26, 'y': 6, 'w': 26, 'h': 26, 'local': {'open': 'side_config'}, 'tooltip': 'Configuration'}]
    widgets += redstone_widgets(24) + energy_tab_widgets(24, MACHINE_USAGE[m])
    window = {'id': 'side_config', 'x': WIN_X, 'y': WIN_Y, 'width': WIN_W, 'height': WIN_H, 'texture': window_tex,
              'widgets': side_config_widgets(MACHINE_TRANSMISSIONS[m], m, 24 + 15)}
    return with_upgrades({'slots': 3, 'data_slots': 24 + 3, 'results': [1],
                          'screen': {'texture': texture, 'own_labels': True, 'slots': [[28, 20], [28, 51], [143, 35]],
                                     'widgets': widgets, 'windows': [window]}}, m, 3, 24)

put(f'{OUT}/data/{NS}/horizon/menu/dynamic_valve.json', dynamic_valve_menu('dynamic_valve'))
put(f'{OUT}/data/{NS}/horizon/menu/thermal_evaporation_controller.json', evaporation_menu('thermal_evaporation_controller'))
put(f'{OUT}/data/{NS}/horizon/menu/boiler_valve.json', boiler_valve_menu('boiler_valve'))
put(f'{OUT}/data/{NS}/horizon/menu/turbine_valve.json', turbine_valve_menu('turbine_valve'))
put(f'{OUT}/data/{NS}/horizon/menu/fission_reactor_port.json', fission_port_menu('fission_reactor_port'))
put(f'{OUT}/data/{NS}/horizon/menu/fusion_reactor_controller.json', fusion_controller_menu('fusion_reactor_controller'))
put(f'{OUT}/data/{NS}/horizon/menu/laser_amplifier.json', laser_amplifier_menu('laser_amplifier'))
put(f'{OUT}/data/{NS}/horizon/menu/sps_port.json', sps_port_menu('sps_port'))
put(f'{OUT}/data/{NS}/horizon/menu/logistical_sorter.json', sorter_menu('logistical_sorter'))
put(f'{OUT}/data/{NS}/horizon/menu/quantum_entangloporter.json', entangloporter_menu('quantum_entangloporter'))
put(f'{OUT}/data/{NS}/horizon/menu/teleporter.json', teleporter_menu('teleporter'))
for _b in BINS:
    put(f'{OUT}/data/{NS}/horizon/menu/{_b}.json', bin_menu(_b))
put(f'{OUT}/data/{NS}/horizon/menu/digital_miner.json', miner_menu('digital_miner'))
put(f'{OUT}/data/{NS}/horizon/menu/induction_port.json', induction_port_menu('induction_port'))
for m in MACHINES:
    if m in ('electric_pump', 'fluidic_plenisher'):
        put(f'{OUT}/data/{NS}/horizon/menu/{m}.json', pump_menu(m))
        continue
    if m in PROCESS_LAYOUTS:
        put(f'{OUT}/data/{NS}/horizon/menu/{m}.json', process_menu(m))
        continue
    if m in FLUID_TANKS or m in CHEM_TANKS:
        put(f'{OUT}/data/{NS}/horizon/menu/{m}.json', tank_menu(m))
        continue
    if m in FACTORIES:
        put(f'{OUT}/data/{NS}/horizon/menu/{m}.json', factory_menu(m))
        continue
    layout = LAYOUTS.get(m, LAYOUTS['electric'])
    title = MACHINE_NAMES.get(m, m.replace('_', ' ').title())
    px, py = layout['progress']
    put(f'{OUT}/data/{NS}/horizon/menu/{m}.json', with_upgrades({
        'slots': len(layout['slots']), 'data_slots': 19, 'results': layout['results'],
        'screen': {'texture': machine_panel(m, layout), 'own_labels': True, 'slots': layout['slots'],
                   'widgets': [
            # the vertical power bar: its contents are 4 by 52 inside the frame
            {'type': 'sprite', 'texture': power_fill, 'x': 165, 'y': layout['bar_y'] + 1, 'w': 4, 'h': 52,
             'value': {'pair': [1, 2]}, 'max': {'pair': [3, 4]}, 'direction': 'up', 'tooltip': 'Energy: {value} / {max}'},
            # GuiProgress, ProgressType.BAR
            {'type': 'sprite', 'texture': progress_full, 'x': px, 'y': py, 'w': 25, 'h': 9, 'value': {'data': 0},
             'max': {'data': 19}, 'direction': 'right', 'tooltip': 'Progress: {percent}%'},
            # the title, centred at the top, and the inventory's, at 8, 72 (vanilla's own places)
            {'type': 'text', 'x': (176 - text_width(title)) // 2, 'y': 6, 'text': title, 'color': '#404040'},
            {'type': 'text', 'x': 8, 'y': 72, 'text': 'Inventory', 'color': '#404040'},
            {'type': 'text', 'x': 92, 'y': 54, 'text': 'Status {5}', 'color': '#808080'},
            {'type': 'text', 'x': 128, 'y': 72, 'text': 'v0.1.86', 'color': '#808080'},
            # GuiSideConfigurationTab at -26, 6: opens the side configuration window
            {'type': 'sprite', 'texture': tab_config, 'x': -26, 'y': 6, 'w': 26, 'h': 26},
            {'type': 'button', 'x': -26, 'y': 6, 'w': 26, 'h': 26, 'local': {'open': 'side_config'}, 'tooltip': 'Configuration'}]
                   + redstone_widgets(16) + energy_tab_widgets(16, MACHINE_USAGE[m]),
                   'windows': [{'id': 'side_config', 'x': WIN_X, 'y': WIN_Y, 'width': WIN_W, 'height': WIN_H,
                                'texture': window_tex, 'widgets': side_config_widgets(MACHINE_TRANSMISSIONS[m], m, 31)}]}}, m, len(layout['slots']), 16))



# the fluid or chemical a tank holds, drawn in it as a box that rises with it
def tank_renderers():
    from PIL import Image
    liquid = Image.open(find('textures/mek_liquid/liquid.png', ASSET_DIRS)).convert('RGBA').crop((0, 0, 16, 16))
    path = f'{OUT}/assets/{NS}/textures/block/chem_liquid.png'
    os.makedirs(os.path.dirname(path), exist_ok=True)
    liquid.save(path)
    tints = {ty_id: ty['tint'] for ty_id, ty in zip(CHEMICALS, GAUGE_TYPES)}
    for m in FLUID_TANKS:
        put(f'{OUT}/data/{NS}/horizon/renderer/{m}.json', {'elements': [
            {'type': 'fluid', 'tank': 0, 'from': [2.6, 2.6, 2.6], 'to': [13.4, 13.4, 13.4],
             'sprites': {'minecraft:lava': 'minecraft:block/lava_still'}, 'default_sprite': f'{NS}:block/chem_liquid',
             'tints': {k: v for k, v in tints.items() if k != 'minecraft:lava'}}]})
    for m in CHEM_TANKS:
        put(f'{OUT}/data/{NS}/horizon/renderer/{m}.json', {'elements': [
            {'type': 'fluid', 'tank': 0, 'from': [3.6, 1.6, 3.6], 'to': [12.4, 12.4, 12.4], 'default_sprite': f'{NS}:block/chem_liquid',
             'tints': tints}]})
tank_renderers()

# FORK-31: the fluid or chemical of a dynamic tank, drawn inside its casing (`MultiblockTileEntityRenderer`): the valve with the least
# position draws it, from the casing's box (`box_at` 5: least and greatest corners) and a fill byte the mod writes (dynamic.rs `with_fill`)
def volume_renderers():
    tints = {ty_id: ty['tint'] for ty_id, ty in zip(CHEMICALS, GAUGE_TYPES)}
    volume = {'box_at': 5, 'draw_at': 29, 'fill_at': 30}
    put(f'{OUT}/data/{NS}/horizon/renderer/dynamic_valve.json', {'elements': [
        {'type': 'fluid_volume', 'tank': 0, **volume, 'sprites': {'minecraft:lava': 'minecraft:block/lava_still'},
         'default_sprite': f'{NS}:block/chem_liquid', 'tints': {k: v for k, v in tints.items() if k != 'minecraft:lava'}},
        {'type': 'fluid_volume', 'tank': 1, **volume, 'default_sprite': f'{NS}:block/chem_liquid', 'tints': tints}]})
volume_renderers()

# ---- the energy cube's screen (GuiEnergyCube): the discharge slot (17, 35) with its minus, the charge slot (143, 35) with its
# plus, the wide energy gauge at 55, 18, the config tab and the energy tab at the left
def energy_cube_panel(name):
    from PIL import Image
    panel = nine_slice(gui_sprite('base'), 176, 166, 4)
    slot = gui_sprite('slot/normal')
    for row in range(3):
        for col in range(9):
            panel.paste(slot, (7 + 18 * col, 83 + 18 * row))
    for col in range(9):
        panel.paste(slot, (7 + 18 * col, 141))
    for x, overlay in ((17, 'slot/overlay/minus'), (143, 'slot/overlay/plus')):
        panel.alpha_composite(slot, (x - 1, 34))
        panel.alpha_composite(gui_sprite(overlay), (x - 1, 34))
    # GuiEnergyGauge, WIDE: a 66 by 50 frame around 64 by 48
    panel.alpha_composite(nine_slice(gui_sprite('gauge/normal'), 66, 50, 2), (55, 18))
    path = f'{OUT}/assets/{NS}/textures/gui/{name}.png'
    os.makedirs(os.path.dirname(path), exist_ok=True)
    panel.save(path)
    return f'{NS}:gui/{name}'

def energy_gauge_fill():
    """The energy sprite (its first frame, 16 by 16) tiled over the gauge's 64 by 48 contents."""
    from PIL import Image
    src = Image.open(find('textures/mek_liquid/energy.png', ASSET_DIRS)).convert('RGBA').crop((0, 0, 16, 16))
    img = Image.new('RGBA', (64, 48))
    for x in range(0, 64, 16):
        for y in range(0, 48, 16):
            img.paste(src, (x, y))
    return save_gui(img, 'energy_gauge_fill')

gauge_fill = energy_gauge_fill()
gauge_overlay = copy_gui('gauge/overlay/wide', 'gauge_overlay_wide')
tab_energy_info = copy_gui('tab/energy_info', 'tab_energy_info')
def armor_holder():
    """GuiSideHolder at 176, 36 (26 by 98, the right one, untinted: the colour map's TAB_ARMOR_SLOTS is white) with the five
    slot frames of the armor side bar: 180, 41 and 18 down for the four pieces, the off-hand under them."""
    holder = nine_slice(gui_sprite('holder_right'), 26, 98, 4)
    frame = gui_sprite('slot/normal')
    for k in range(5):
        holder.alpha_composite(frame, (3, 4 + 18 * k))
    return save_gui(holder, 'armor_holder')

armor_holder_tex = armor_holder()
CUBES = {'basic_energy_cube': ('Basic Energy Cube', 4000), 'advanced_energy_cube': ('Advanced Energy Cube', 16000),
         'elite_energy_cube': ('Elite Energy Cube', 64000), 'ultimate_energy_cube': ('Ultimate Energy Cube', 256000),
         'creative_energy_cube': ('Creative Energy Cube', 1000000000)}
for cube_name, (cube_title, rate) in CUBES.items():
    put(f'{OUT}/data/{NS}/horizon/menu/{cube_name}.json', {
        'slots': 2, 'data_slots': 17, 'results': [],
        # ContainerBuilder.armorSideBar(180, 41, 0): the four armor slots and the off-hand's, the player's own
        'armor': {'x': 180, 'y': 41, 'offhand': 0},
        'screen': {'texture': energy_cube_panel(cube_name), 'own_labels': True, 'slots': [[17, 35], [143, 35]],
                   'widgets': [
            # GuiSideHolder at 176, 36: the armor side bar, behind the slots
            {'type': 'sprite', 'texture': armor_holder_tex, 'x': 176, 'y': 36, 'w': 26, 'h': 98},
            {'type': 'sprite', 'texture': gauge_fill, 'x': 56, 'y': 19, 'w': 64, 'h': 48, 'value': {'pair': [1, 2]},
             'max': {'pair': [3, 4]}, 'direction': 'up', 'tooltip': 'Energy: {value} / {max}'},
            {'type': 'sprite', 'texture': gauge_overlay, 'x': 56, 'y': 19, 'w': 64, 'h': 48},
            {'type': 'text', 'x': (176 - text_width(cube_title)) // 2, 'y': 6, 'text': cube_title, 'color': '#404040'},
            {'type': 'text', 'x': 8, 'y': 72, 'text': 'Inventory', 'color': '#404040'},
            {'type': 'text', 'x': 128, 'y': 72, 'text': 'v0.1.28', 'color': '#808080'},
            # GuiSideConfigurationTab at -26, 6
            {'type': 'sprite', 'texture': tab_config, 'x': -26, 'y': 6, 'w': 26, 'h': 26},
            {'type': 'button', 'x': -26, 'y': 6, 'w': 26, 'h': 26, 'local': {'open': 'side_config'}, 'tooltip': 'Configuration'},
            # GuiEnergyTab at -26, 137: the input rate and the most it puts out
            {'type': 'sprite', 'texture': tab_energy_info, 'x': -26, 'y': 137, 'w': 26, 'h': 26},
            {'type': 'button', 'x': -26, 'y': 137, 'w': 26, 'h': 26,
             'tooltip': f'Input Rate: {{0}} FE\nMax Output: {rate} FE\nUnit: Forge Energy'}],
                   'windows': [{'id': 'side_config', 'x': WIN_X, 'y': WIN_Y, 'width': WIN_W, 'height': WIN_H,
                                'texture': window_tex, 'widgets': side_config_widgets([1], None, 16)}]}})
# Status: 0 idle, 1 running, 2 no recipe for the input, 3 no energy, 4 not enough input or no room, 5 no recipes loaded

def refs(v):
    if isinstance(v, str):
        yield v
    elif isinstance(v, dict):
        for x in v.values():
            yield from refs(x)
    elif isinstance(v, list):
        for x in v:
            yield from refs(x)

# ---------------------------------------------------------------- MekanismGenerators: heat, bio, solar and gas-burning
# The 26.3 checkout has only the Java of this module, so its models, blockstates and textures come from the 1.21.1 release
# (tools/generators-assets); the logic is read from the 26.3 code.
GDIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'generators-assets/assets/mekanismgenerators')
GGEN = f'{SRC}/src/datagen/generated/mekanismgenerators'
gen_done = set()

def gen_rel(rel):
    """Where a generators asset goes in mekanism's namespace: `block/model/x` -> `block/generators/model_x`."""
    head, _, tail = rel.partition('/')
    return f'{head}/generators/{tail.replace("/", "_")}'

def gen_ref(ref):
    if ':' in ref and ref.split(':', 1)[0] == GSRC:
        return f'{NS}:{gen_rel(ref.split(":", 1)[1])}'
    return ref

def gen_texture(ref):
    ns, rel = ref.split(':', 1)
    if ns == NS:
        copy_texture(ref)
        return
    src = f'{GDIR}/textures/{rel}.png'
    if not os.path.exists(src):
        warnings.append(f'missing generators texture {ref}')
        return
    put(f'{OUT}/assets/{NS}/textures/{gen_rel(rel)}.png', src=src)
    if os.path.exists(src + '.mcmeta'):
        put(f'{OUT}/assets/{NS}/textures/{gen_rel(rel)}.png.mcmeta', src=src + '.mcmeta')

def gen_model(ref):
    """A generators model with its textures; a composite (neoforge:composite) is flattened into one model."""
    if ':' not in ref:
        return True  # a vanilla model (block/cube_all)
    ns, rel = ref.split(':', 1)
    if ns != GSRC:
        return copy_model(ref)
    if rel in gen_done:
        return True
    gen_done.add(rel)
    m = read_json(f'{GDIR}/models/{rel}.json')
    if m.get('loader') == 'neoforge:composite':
        textures, elements = dict(m.get('textures', {})), []
        for child in m['children'].values():
            textures.update(child.get('textures', {}))
            elements += child.get('elements', [])
        m = {k: v for k, v in m.items() if k not in ('loader', 'children')}
        m['textures'], m['elements'] = textures, elements
        m['render_type'] = 'minecraft:cutout'
    if 'parent' in m:
        if not gen_model(m['parent']):
            return False
        m['parent'] = gen_ref(m['parent'])
    for k, v in list(m.get('textures', {}).items()):
        if isinstance(v, str) and not v.startswith('#'):
            gen_texture(v)
            m['textures'][k] = gen_ref(v)
    put(f'{OUT}/assets/{NS}/models/{gen_rel(rel)}.json', json.dumps(m, indent=2))
    return True

# the screens: slots, the vertical power bar, then what each generator shows (GuiBioGenerator, GuiHeatGenerator,
# GuiGasGenerator, GuiSolarGenerator); the data slots are 0 working, 1-4 energy and capacity, 5 what it makes a tick,
# 6 whether the sun is seen (solar) / the temperature (heat) / the burn rate (gas), 7 and 8 the heat lost, 10-14 the tank
GEN_TITLES = {'advanced_solar_generator': 'Advanced Solar Generator', 'wind_generator': 'Wind Generator', 'heat_generator': 'Heat Generator', 'bio_generator': 'Bio-Generator', 'solar_generator': 'Solar Generator',
              'gas_burning_generator': 'Gas-Burning Generator'}
GEN_LAYOUT = {
    'advanced_solar_generator': {'fuel': False, 'screen': (40, 23, 96, 40), 'sun': True},
    'wind_generator': {'fuel': False, 'screen': (40, 23, 96, 40)},
    'heat_generator': {'fuel': True, 'wide': True, 'heat': True, 'overlay': None},
    'bio_generator': {'fuel': True, 'bar': True, 'screen': (48, 23, 80, 40), 'overlay': None},
    'solar_generator': {'fuel': False, 'screen': (40, 23, 96, 40), 'sun': True},
    'gas_burning_generator': {'fuel': True, 'wide': True, 'overlay': 'slot/overlay/minus', 'burn': True}}

def sun_icons():
    """GuiStateTexture's `state/sees_sun` and `state/no_sun` are not in the sources at hand: a 16 by 16 sun (lit) and the same
    greyed out are drawn here."""
    from PIL import Image, ImageDraw
    out = []
    for name, colour in (('sees_sun', (255, 214, 0, 255)), ('no_sun', (90, 90, 90, 255))):
        img = Image.new('RGBA', (16, 16), (0, 0, 0, 0))
        d = ImageDraw.Draw(img)
        d.ellipse((4, 4, 11, 11), fill=colour)
        for x0, y0, x1, y1 in ((7, 0, 8, 2), (7, 13, 8, 15), (0, 7, 2, 8), (13, 7, 15, 8), (2, 2, 3, 3), (12, 2, 13, 3), (2, 12, 3, 13), (12, 12, 13, 13)):
            d.rectangle((x0, y0, x1, y1), fill=colour)
        out.append(save_gui(img, name))
    return out

def gen_panel(n, L):
    panel = nine_slice(gui_sprite('base'), 176, 166, 4)
    slot = gui_sprite('slot/normal')
    for row in range(3):
        for col in range(9):
            panel.paste(slot, (7 + 18 * col, 83 + 18 * row))
    for col in range(9):
        panel.paste(slot, (7 + 18 * col, 141))
    if L['fuel']:
        panel.alpha_composite(gui_sprite('slot/input'), (16, 34))
        if L.get('overlay'):
            panel.alpha_composite(gui_sprite(L['overlay']), (16, 34))
    panel.alpha_composite(gui_sprite('slot/power'), (142, 34))
    panel.alpha_composite(nine_slice(gui_sprite('bar/base'), 6, 54, 1), (164, 15))
    if L.get('wide'):
        panel.alpha_composite(nine_slice(gui_sprite('gauge/normal'), 66, 50, 2), (55, 18))
    if L.get('bar'):
        panel.alpha_composite(nine_slice(gui_sprite('bar/base'), 6, 54, 1), (7, 15))
    if 'screen' in L:
        x, y, w, h = L['screen']
        panel.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), w, h, 3), (x, y))
    return save_gui(panel, f'generator_{n}')

GEN_ARMOR = {'x': -20, 'y': 11, 'offhand': 0}
def gen_armor_holder():
    holder = nine_slice(gui_sprite('holder_left'), 26, 98, 4)
    frame = gui_sprite('slot/normal')
    for k in range(5):
        holder.alpha_composite(frame, (3, 4 + 18 * k))
    return save_gui(holder, 'armor_holder_left_top')

def gen_menu(n):
    L = GEN_LAYOUT[n]
    title = GEN_TITLES[n]
    producing = 'Producing: {5} FE/t'
    widgets = [{'type': 'sprite', 'texture': gen_armor_holder(), 'x': -26, 'y': 6, 'w': 26, 'h': 98},
               {'type': 'sprite', 'texture': power_fill, 'x': 165, 'y': 16, 'w': 4, 'h': 52, 'direction': 'up',
                'value': {'pair': [1, 2]}, 'max': {'pair': [3, 4]}, 'tooltip': 'Energy: {value} / {max}'}]
    tank = {'value': {'pair': [10, 11]}, 'max': {'pair': [13, 14]}, 'kind': {'data': 12}, 'types': GAUGE_TYPES}
    if L.get('wide'):
        widgets += [dict({'type': 'gauge', 'x': 56, 'y': 19, 'w': 64, 'h': 48, 'tooltip': '{label}: {value} mB', 'empty': 'Empty'}, **tank),
                    {'type': 'sprite', 'texture': copy_gui('gauge/overlay/wide', 'gauge_overlay_wide'), 'x': 56, 'y': 19, 'w': 64, 'h': 48}]
    if L.get('bar'):
        widgets.append(dict({'type': 'gauge', 'x': 8, 'y': 16, 'w': 4, 'h': 52, 'direction': 'up', 'tooltip': '{label}: {value} mB', 'empty': 'Empty'}, **tank))
    if 'screen' in L:
        x, y, w, h = L['screen']
        lines = ['Energy: {1+2} / {3+4} FE' if n in ('solar_generator', 'advanced_solar_generator', 'wind_generator') else '{1+2} FE']
        if n == 'bio_generator':
            lines.append('BioFuel: {10+11}')
        lines.append(producing)
        for i, text in enumerate(lines):
            widgets.append({'type': 'text', 'x': x + 4, 'y': y + 4 + 10 * i, 'text': text, 'color': '#00CD00'})
    if L.get('sun'):
        sees, none = sun_icons()
        widgets += [{'type': 'show', 'when': {'data': {'value': {'data': 6}, 'eq': 1}}, 'widgets': [
                        {'type': 'sprite', 'texture': sees, 'x': 18, 'y': 35, 'w': 16, 'h': 16}]},
                    {'type': 'show', 'when': {'data': {'value': {'data': 6}, 'eq': 0}}, 'widgets': [
                        {'type': 'sprite', 'texture': none, 'x': 18, 'y': 35, 'w': 16, 'h': 16}]}]
    # GuiEnergyTab at -26, 137
    widgets += [{'type': 'sprite', 'texture': tab_energy_info, 'x': -26, 'y': 137, 'w': 26, 'h': 26},
                {'type': 'button', 'x': -26, 'y': 137, 'w': 26, 'h': 26, 'tooltip': producing}]
    if L.get('heat'):
        # GuiHeatTab at -26, 109
        widgets += [{'type': 'sprite', 'texture': copy_gui('tab/heat_info_k', 'tab_heat_info_k'), 'x': -26, 'y': 109, 'w': 26, 'h': 26},
                    {'type': 'button', 'x': -26, 'y': 109, 'w': 26, 'h': 26,
                     'tooltip': 'Temperature: {6} K\nTransferred: {7}/t\nDissipated: {8}/t'}]
    widgets.append({'type': 'text', 'x': (176 - text_width(title)) // 2, 'y': 6, 'text': title, 'color': '#404040'})
    widgets.append({'type': 'text', 'x': 8, 'y': 72, 'text': 'Inventory', 'color': '#404040'})
    if L.get('burn'):
        widgets.append({'type': 'text', 'x': 176 - 6 - text_width('Burn Rate: 00 mB/t'), 'y': 72, 'text': 'Burn Rate: {6} mB/t', 'color': '#404040'})
    widgets.append({'type': 'text', 'x': 62, 'y': 72 if not L.get('burn') else 60, 'text': 'v0.1.86', 'color': '#808080'})
    slots = ([[17, 35]] if L['fuel'] else []) + [[143, 35]]
    widgets += redstone_widgets(15)
    return with_upgrades({'slots': len(slots), 'data_slots': 18, 'results': [], 'armor': GEN_ARMOR,
            'screen': {'texture': gen_panel(n, L), 'own_labels': True, 'slots': slots, 'widgets': widgets, 'windows': []}}, n, len(slots), 15)

for g in GENERATORS:
    bs = read_json(f'{GDIR}/blockstates/{g}.json')
    bs = json.loads(json.dumps(bs).replace('active=', 'lit=').replace(f'{GSRC}:block/', f'{NS}:block/generators/'))
    has_facing = any('facing' in k for k in bs['variants'])
    if not any('lit=' in k for k in bs['variants']):
        bs['variants'] = {(f'{k},lit={lit}' if k else f'lit={lit}'): v for k, v in bs['variants'].items() for lit in ('false', 'true')}
    if not all(gen_model(m.replace(f'{NS}:block/generators/', f'{GSRC}:block/')) for m in models_of_blockstate(bs)):
        continue
    kept_blocks.append(g)
    put(f'{OUT}/assets/{NS}/blockstates/{g}.json', bs)
    state = {'lit': {'type': 'boolean'}}
    if has_facing:
        state['facing'] = {'type': 'enum', 'values': ['north', 'east', 'south', 'west']}
    put(f'{OUT}/data/{GNS}/horizon/block/{g}.json', {
        'template': 'minecraft:beacon', 'properties': {'hardness': 3.5, 'explosion_resistance': 16.0, 'requires_tool': True, 'light': 0},
        'item': True, 'state_properties': state})
    put(f'{OUT}/assets/{GNS}/items/{g}.json', {'model': {'type': 'minecraft:model', 'model': f'{NS}:block/generators/{g}'}})
    put(f'{OUT}/data/{GNS}/loot_table/blocks/{g}.json', {'type': 'minecraft:block', 'pools': [{'rolls': 1, 'entries': [
        {'type': 'minecraft:item', 'name': f'{GNS}:{g}'}], 'conditions': [{'type': 'minecraft:survives_explosion'}]}]})
    put(f'{OUT}/data/{GNS}/horizon/menu/{g}.json', gen_menu(g))
# the bounding blocks big machines place around themselves: invisible, or a piece of the wind generator
put(f'{OUT}/data/{GNS}/horizon/block/bounding_block.json', {'template': 'minecraft:beacon', 'properties': {'hardness': 3.5, 'explosion_resistance': 16.0, 'requires_tool': True, 'light': 0},
    'item': False, 'state_properties': {'part': {'type': 'enum', 'values': ['none', 'pole', 'top']}}})
put(f'{OUT}/assets/{NS}/blockstates/bounding_block.json', {'variants': {
    'part=none': {'model': f'{NS}:block/generators/bounding_none'},
    'part=pole': {'model': f'{NS}:block/generators/wind_generator_pole'},
    'part=top': {'model': f'{NS}:block/generators/wind_generator_top'}}})
put(f'{OUT}/assets/{NS}/models/block/generators/bounding_none.json', json.dumps({'textures': {'particle': f'{NS}:block/steel_casing'}}))
gen_model(f'{GSRC}:block/wind_generator_pole')
gen_model(f'{GSRC}:block/wind_generator_top')
put(f'{OUT}/data/{GNS}/loot_table/blocks/bounding_block.json', {'type': 'minecraft:block', 'pools': []})
kept_blocks_hidden = ['bounding_block']

# the turbine: full blocks and a rotor, no states; its blade is an item
for g in TURBINE + FISSION + FUSION:
    bs = read_json(f'{GDIR}/blockstates/{g}.json')
    if all(k.startswith('active=') for k in bs['variants']):
        bs = {'variants': {'': bs['variants']['active=false']}}
    if all(k.startswith('mode=') for k in bs['variants']):
        # the port's mode is not a state of the port's block: it shows the input one
        bs = {'variants': {'': bs['variants']['mode=input']}}
    ok = True
    for v in bs['variants'].values():
        ref = v['model']
        ok = ok and gen_model(ref)
        v['model'] = gen_ref(ref)
    if not ok:
        continue
    kept_blocks.append(g)
    put(f'{OUT}/assets/{NS}/blockstates/{g}.json', bs)
    put(f'{OUT}/data/{GNS}/horizon/block/{g}.json', {
        'template': 'minecraft:beacon', 'properties': {'hardness': 3.5, 'explosion_resistance': 16.0, 'requires_tool': True, 'light': 0},
        'item': True})
    put(f'{OUT}/assets/{GNS}/items/{g}.json', {'model': {'type': 'minecraft:model', 'model': bs['variants'][''] ['model']}})
    put(f'{OUT}/data/{GNS}/loot_table/blocks/{g}.json', {'type': 'minecraft:block', 'pools': [{'rolls': 1, 'entries': [
        {'type': 'minecraft:item', 'name': f'{GNS}:{g}'}], 'conditions': [{'type': 'minecraft:survives_explosion'}]}]})
for gi in ('turbine_blade', 'hohlraum'):
    gen_model(f'{GSRC}:item/{gi}')
    put(f'{OUT}/data/{NS}/horizon/item/{gi}.json', {'template': 'minecraft:stick'})
    put(f'{OUT}/assets/{NS}/items/{gi}.json', {'model': {'type': 'minecraft:model', 'model': f'{NS}:item/generators/{gi}'}})
    kept_items.append(gi)
# the hohlraum is made by infusing (the 26.3 recipe), with the generators' id turned into the port's
_h = json.loads(open(f'{GGEN}/data/{GNS_SRC}/recipe/hohlraum.json').read().replace(f'{GSRC}:', f'{NS}:'))
put(f'{OUT}/data/{NS}/recipe/hohlraum.json', _h)
for p in sorted(glob.glob(f'{GGEN}/data/{GNS_SRC}/recipe/turbine/*.json') + glob.glob(f'{GGEN}/data/{GNS_SRC}/recipe/fission_reactor/*.json')) + [f'{GGEN}/data/{GNS_SRC}/recipe/{n}.json' for n in ('electromagnetic_coil', 'rotational_complex', 'saturating_condenser')]:
    text = open(p).read().replace(f'{GSRC}:', f'{NS}:')
    r = json.loads(text)
    name = r['result']['id'].split(':')[-1]
    ing = [s for s in refs(r.get('key', r.get('ingredients', []))) if s.startswith(NS + ':')]
    if all(s.split(':')[1] in (set(kept_items) | set(kept_blocks) | set(TURBINE)) for s in ing):
        put(f'{OUT}/data/{NS}/recipe/turbine_{name}.json', r)
# ---------------------------------------------------------------- MekanismTools: bronze, lapis lazuli, osmium, refined glowstone/obsidian and steel
# tools and armor, and the paxels of the vanilla materials. Items are data (a vanilla item's components, changed); textures come from the 1.21.1 jar
# (tools/tools-assets). The numbers are the `*MaterialDefaults` of the 26.3 code.
TJ = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'tools-assets/assets/mekanismtools')
TGEN = f'{SRC}/src/datagen/generated/mekanismtools'
# name: axe damage, axe speed, durability, mining speed, damage bonus, enchantability, armor enchantability, toughness, knockback resistance,
# armor durability (boots, leggings, chestplate, helmet), armor defense (same), the vanilla tier whose incorrect-for-drops tag it uses
TOOL_MATERIALS = {
    'bronze': (7, -3.0, 375, 7, 2, 10, 10, 1, 0.0, (234, 270, 288, 198), (2, 6, 7, 3), 'iron'),
    'lapis_lazuli': (4, -2.9, 128, 9, 1, 32, 32, 0, 0.0, (130, 150, 160, 110), (1, 3, 4, 1), 'stone'),
    'osmium': (8, -3.3, 1024, 4, 4, 14, 14, 3, 0.1, (390, 450, 480, 330), (3, 6, 8, 4), 'iron'),
    'refined_glowstone': (6, -2.9, 384, 15, 2, 20, 20, 0, 0.0, (221, 255, 272, 187), (3, 6, 8, 3), 'diamond'),
    'refined_obsidian': (7, -2.9, 4096, 12, 8, 18, 18, 5, 0.2, (975, 1125, 1200, 825), (5, 8, 12, 6), 'netherite'),
    'steel': (7, -3.0, 500, 8, 3, 16, 16, 2, 0.0, (260, 300, 320, 220), (3, 6, 8, 3), 'diamond'),
}
# the vanilla materials' paxels: durability, speed, damage bonus, enchantability, axe damage, axe speed, the tier
PAXEL_MATERIALS = {
    'wood': (59, 2, 0, 15, 6, -3.2, 'wooden'), 'stone': (131, 4, 1, 5, 7, -3.2, 'stone'), 'iron': (250, 6, 2, 14, 6, -3.1, 'iron'),
    'diamond': (1561, 8, 3, 10, 5, -3.0, 'diamond'), 'gold': (32, 12, 0, 22, 6, -3.0, 'gold'), 'netherite': (2031, 9, 4, 15, 5, -3.0, 'netherite'),
}
ARMOR_PIECES = [('boots', 'feet', 'iron_boots'), ('leggings', 'legs', 'iron_leggings'), ('chestplate', 'chest', 'iron_chestplate'), ('helmet', 'head', 'iron_helmet')]

def tool_rules(tier, mineable, speed):
    rules = [{'blocks': f'#minecraft:incorrect_for_{tier}_tool', 'correct_for_drops': False}]
    rules += [{'blocks': f'#minecraft:mineable/{m}', 'correct_for_drops': True, 'speed': speed} for m in mineable]
    return {'rules': rules, 'default_mining_speed': 1.0, 'damage_per_block': 1}

def tool_item(template, durability, damage, attack_speed, enchant, tool=None, repair=None):
    comp = {
        'minecraft:max_damage': durability,
        'minecraft:enchantable': {'value': enchant},
        'minecraft:attribute_modifiers': [
            {'type': 'minecraft:attack_damage', 'id': 'minecraft:base_attack_damage', 'amount': damage, 'operation': 'add_value', 'slot': 'mainhand'},
            {'type': 'minecraft:attack_speed', 'id': 'minecraft:base_attack_speed', 'amount': attack_speed, 'operation': 'add_value', 'slot': 'mainhand'}]}
    if tool:
        comp['minecraft:tool'] = tool
    comp['minecraft:repairable'] = {'items': repair} if repair else {}
    if not repair:
        comp['!minecraft:repairable'] = {}
        del comp['minecraft:repairable']
    return {'template': template, 'components': comp}

def tools_item(name, data, texture_src, model_parent, texture_ref):
    put(f'{OUT}/data/{NS}/horizon/item/{name}.json', data)
    tex = f'assets/{NS}/textures/item/tools/{name}.png'
    put(f'{OUT}/{tex}', src=texture_src)
    put(f'{OUT}/assets/{NS}/models/item/tools/{name}.json', {'parent': model_parent, 'textures': {'layer0': f'{NS}:item/tools/{name}'}})
    put(f'{OUT}/assets/{NS}/items/{name}.json', {'model': {'type': 'minecraft:model', 'model': f'{NS}:item/tools/{name}'}})
    kept_items.append(name)

for mat, (axe_d, axe_s, dur, speed, bonus, ench, arm_ench, tough, kb, arm_dur, arm_def, tier) in TOOL_MATERIALS.items():
    ingots = f'#c:ingots/{mat}'
    texture = lambda piece: f'{TJ}/textures/item/{mat}/{piece}.png'
    tools_item(f'{mat}_sword', tool_item('minecraft:iron_sword', dur, 3 + bonus, -2.4, ench, repair=ingots), texture('sword'), 'minecraft:item/handheld', None)
    tools_item(f'{mat}_pickaxe', tool_item('minecraft:iron_pickaxe', dur, 1 + bonus, -2.8, ench, tool_rules(tier, ['pickaxe'], speed), ingots), texture('pickaxe'), 'minecraft:item/handheld', None)
    tools_item(f'{mat}_axe', tool_item('minecraft:iron_axe', dur, axe_d + bonus, axe_s, ench, tool_rules(tier, ['axe'], speed), ingots), texture('axe'), 'minecraft:item/handheld', None)
    tools_item(f'{mat}_shovel', tool_item('minecraft:iron_shovel', dur, 1.5 + bonus, -3.0, ench, tool_rules(tier, ['shovel'], speed), ingots), texture('shovel'), 'minecraft:item/handheld', None)
    tools_item(f'{mat}_hoe', tool_item('minecraft:iron_hoe', dur, 0.0, bonus - 3.0, ench, tool_rules(tier, ['hoe'], speed), ingots), texture('hoe'), 'minecraft:item/handheld', None)
    tools_item(f'{mat}_paxel', tool_item('minecraft:iron_axe', 2 * dur, axe_d + 1 + bonus, -2.4, ench, tool_rules(tier, ['axe', 'pickaxe', 'shovel', 'hoe'], speed), ingots), texture('paxel'), 'minecraft:item/handheld', None)
    for k, (piece, slot, template) in enumerate(ARMOR_PIECES):
        mods = [{'type': 'minecraft:armor', 'id': f'minecraft:armor.{piece}', 'amount': arm_def[k], 'operation': 'add_value', 'slot': slot},
                {'type': 'minecraft:armor_toughness', 'id': f'minecraft:armor.{piece}', 'amount': tough, 'operation': 'add_value', 'slot': slot}]
        if kb:
            mods.append({'type': 'minecraft:knockback_resistance', 'id': f'minecraft:armor.{piece}', 'amount': kb, 'operation': 'add_value', 'slot': slot})
        data = {'template': f'minecraft:{template}', 'components': {
            'minecraft:max_damage': arm_dur[k], 'minecraft:enchantable': {'value': arm_ench},
            'minecraft:equippable': {'slot': slot, 'asset_id': f'{NS}:{mat}'},
            'minecraft:attribute_modifiers': mods, 'minecraft:repairable': {'items': ingots}}}
        tools_item(f'{mat}_{piece}', data, texture(piece), 'minecraft:item/generated', None)
    # the worn model: layer 1 on the body, layer 2 on the legs
    put(f'{OUT}/assets/{NS}/equipment/{mat}.json', {'layers': {'humanoid': [{'texture': f'{NS}:{mat}'}], 'humanoid_leggings': [{'texture': f'{NS}:{mat}'}]}})
    put(f'{OUT}/assets/{NS}/textures/entity/equipment/humanoid/{mat}.png', src=f'{TJ}/textures/models/armor/{mat}_layer_1.png')
    put(f'{OUT}/assets/{NS}/textures/entity/equipment/humanoid_leggings/{mat}.png', src=f'{TJ}/textures/models/armor/{mat}_layer_2.png')

for mat, (dur, speed, bonus, ench, axe_d, axe_s, tier) in PAXEL_MATERIALS.items():
    name = f'{mat}_paxel'
    tools_item(name, tool_item('minecraft:iron_axe', 2 * dur, axe_d + 1 + bonus, -2.4, ench, tool_rules(tier, ['axe', 'pickaxe', 'shovel', 'hoe'], speed)),
               f'{TJ}/textures/item/{name}.png', 'minecraft:item/handheld', None)

# the recipes (the ids of the module turned into the port's, the paxel's own recipe type into a shaped one)
_tool_names = {n for n in kept_items}
for p in sorted(glob.glob(f'{TGEN}/data/mekanismtools/recipe/**/*.json', recursive=True)):
    if 'nugget' in p or p.endswith('shield.json'):
        continue
    r = json.loads(open(p).read().replace('mekanismtools:', f'{NS}:'))
    if r.get('type') == f'{NS}:paxel':
        r['type'] = 'minecraft:crafting_shaped'
    result = r.get('result', {}).get('id', '').split(':')[-1]
    if result not in _tool_names:
        continue
    _rel = os.path.relpath(p, f'{TGEN}/data/mekanismtools/recipe')[:-5].replace(os.sep, '__')
    put(f'{OUT}/data/{NS}/recipe/tools_{_rel}.json', r)
tools_lang = read_json(f'{TJ}/lang/en_us.json')

# ---------------------------------------------------------------- MekanismAdditions: plastic blocks (18 colors) and glow panels
# Their colors are a tint the Java client paints on gray textures; the port has no block tint for a mod's blocks, so each color gets its own painted
# texture and its own copy of the models (tools/additions-assets, from the 1.21.1 jar).
AJ = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'additions-assets/assets/mekanismadditions')
AGEN = f'{SRC}/src/datagen/generated/mekanismadditions'
PLASTIC_COLORS = {
    'aqua': 0x30FFF9, 'black': 0x404040, 'blue': 0x366BD0, 'brown': 0xA17649, 'cyan': 0x00F3D0, 'dark_red': 0xC9071F, 'gray': 0x7A7A7A,
    'green': 0x59C15F, 'light_blue': 0x559EFF, 'light_gray': 0xCFCFCF, 'lime': 0x75FF89, 'magenta': 0xD55ECB, 'orange': 0xFFA160,
    'pink': 0xFFBCC4, 'purple': 0xA460D9, 'red': 0xFF383C, 'white': 0xFFFFFF, 'yellow': 0xFFDD4F}
# kind (the block's name after the color): template, properties
PLASTIC_KINDS = {
    '_plastic': ('minecraft:stone', {'hardness': 5.0, 'explosion_resistance': 6.0}),
    '_slick_plastic': ('minecraft:packed_ice', {'hardness': 5.0, 'explosion_resistance': 6.0}),
    '_plastic_glow': ('minecraft:stone', {'hardness': 5.0, 'explosion_resistance': 6.0, 'light': 10}),
    '_reinforced_plastic': ('minecraft:stone', {'hardness': 50.0, 'explosion_resistance': 1200.0}),
    '_plastic_road': ('minecraft:stone', {'hardness': 5.0, 'explosion_resistance': 6.0}),
    '_plastic_transparent': ('minecraft:glass', {'hardness': 5.0, 'explosion_resistance': 6.0}),
    '_plastic_slab': ('minecraft:stone_slab', {'hardness': 5.0, 'explosion_resistance': 6.0}),
    '_plastic_glow_slab': ('minecraft:stone_slab', {'hardness': 5.0, 'explosion_resistance': 6.0, 'light': 10}),
    '_plastic_transparent_slab': ('minecraft:stone_slab', {'hardness': 5.0, 'explosion_resistance': 6.0}),
    '_plastic_stairs': ('minecraft:stone_stairs', {'hardness': 5.0, 'explosion_resistance': 6.0}),
    '_plastic_glow_stairs': ('minecraft:stone_stairs', {'hardness': 5.0, 'explosion_resistance': 6.0, 'light': 10}),
    '_plastic_transparent_stairs': ('minecraft:stone_stairs', {'hardness': 5.0, 'explosion_resistance': 6.0}),
    '_plastic_fence': ('minecraft:chorus_plant', {'hardness': 5.0, 'explosion_resistance': 6.0}),
    '_plastic_fence_gate': ('minecraft:oak_fence_gate', {'hardness': 5.0, 'explosion_resistance': 6.0}),
    '_glow_panel': ('minecraft:stone', {'hardness': 1.0, 'explosion_resistance': 6.0, 'light': 15}),
}

def tinted_texture(name, color, rgb):
    """The gray texture `name` painted with a color (each channel scaled by the color's), kept with its animation."""
    from PIL import Image
    src = f'{AJ}/textures/block/{name}.png'
    dst = f'{OUT}/assets/{NS}/textures/block/additions/{color}/{name}.png'
    if os.path.exists(dst):
        return
    img = Image.open(src).convert('RGBA')
    r, g, b = (rgb >> 16) & 255, (rgb >> 8) & 255, rgb & 255
    px = img.load()
    for y in range(img.height):
        for x in range(img.width):
            pr, pg, pb, pa = px[x, y]
            px[x, y] = (pr * r // 255, pg * g // 255, pb * b // 255, pa)
    os.makedirs(os.path.dirname(dst), exist_ok=True)
    img.save(dst)
    if os.path.exists(src + '.mcmeta'):
        shutil.copyfile(src + '.mcmeta', dst + '.mcmeta')

def untint(model):
    """A model with its `tintindex` removed (the colors are in the textures)."""
    if isinstance(model, dict):
        return {k: untint(v) for k, v in model.items() if k != 'tintindex'}
    if isinstance(model, list):
        return [untint(v) for v in model]
    return model

# the colored parents, once, without the tint
colored_dir = f'{AJ}/models/block/colored'
for fn in os.listdir(colored_dir):
    put(f'{OUT}/assets/{NS}/models/block/additions/colored/{fn}', json.dumps(untint(json.loads(open(f'{colored_dir}/{fn}').read().replace('mekanismadditions:block/colored/', f'{NS}:block/additions/colored/')))))
_cc = untint(read_json(find('models/block/colored_cube.json', ASSET_DIRS)))
put(f'{OUT}/assets/{NS}/models/block/additions/colored_cube.json', json.dumps(_cc))
_plastic_models = {fn[:-5]: read_json(f'{AJ}/models/block/plastic/{fn}') for fn in os.listdir(f'{AJ}/models/block/plastic')}
_glow_panel = read_json(f'{AJ}/models/block/glow_panel.json')
_additions_lang = read_json(f'{AJ}/lang/en_us.json')
plastic_blocks = []
for bs_path in sorted(glob.glob(f'{AJ}/blockstates/*.json')):
    name = os.path.basename(bs_path)[:-5]
    color = next((cn for cn in sorted(PLASTIC_COLORS, key=len, reverse=True) if name.startswith(cn + '_')), None)
    kind = name[len(color):] if color else None
    if color is None or kind not in PLASTIC_KINDS:
        continue
    rgb = PLASTIC_COLORS[color]
    # the models of this color: the plastic ones with painted textures, once per color
    if color not in globals().setdefault('_additions_done', set()):
        _additions_done.add(color)
        for mname, m in _plastic_models.items():
            m = json.loads(json.dumps(m))
            m['parent'] = m['parent'].replace('mekanismadditions:block/colored/', f'{NS}:block/additions/colored/').replace('mekanism:block/colored_cube', f'{NS}:block/additions/colored_cube')
            for k, v in list(m.get('textures', {}).items()):
                tex = v.split('/')[-1]
                tinted_texture(tex, color, rgb)
                m['textures'][k] = f'{NS}:block/additions/{color}/{tex}'
            put(f'{OUT}/assets/{NS}/models/block/additions/{color}/{mname}.json', json.dumps(m))
        # the glow panel: the center glows in the color, the rest keeps the gray
        tinted_texture('glow_panel', color, rgb)
        gp = json.loads(json.dumps(_glow_panel))
        gp['textures']['tinted'] = f'{NS}:block/additions/{color}/glow_panel'
        gp['textures']['all'] = f'{NS}:block/additions/gray_glow_panel'
        for el in gp.get('elements', []):
            for face in el['faces'].values():
                if 'tintindex' in face:
                    face['texture'] = '#tinted'
                    del face['tintindex']
        put(f'{OUT}/assets/{NS}/models/block/additions/{color}/glow_panel.json', json.dumps(gp))
    bs = json.loads(open(bs_path).read().replace('mekanismadditions:block/plastic/', f'{NS}:block/additions/{color}/').replace('mekanismadditions:block/glow_panel', f'{NS}:block/additions/{color}/glow_panel'))
    put(f'{OUT}/assets/{NS}/blockstates/{name}.json', bs)
    template, props = PLASTIC_KINDS[kind]
    block = {'template': template, 'properties': props, 'item': True}
    if kind == '_plastic_fence':
        block['connects_to'] = f'#{NS}:plastic_fences'
    if kind == '_glow_panel':
        block['state_properties'] = {'facing': {'type': 'enum', 'values': ['down', 'up', 'north', 'south', 'west', 'east']}}
    put(f'{OUT}/data/{GNS}/horizon/block/{name}.json', block)
    item_model = read_json(f'{AJ}/models/item/{name}.json') if os.path.exists(f'{AJ}/models/item/{name}.json') else {'parent': f'mekanismadditions:block/plastic/block'}
    target = item_model['parent'].replace('mekanismadditions:block/plastic/', f'{NS}:block/additions/{color}/').replace('mekanismadditions:block/glow_panel', f'{NS}:block/additions/{color}/glow_panel')
    put(f'{OUT}/assets/{NS}/items/{name}.json', {'model': {'type': 'minecraft:model', 'model': target}})
    put(f'{OUT}/data/{GNS}/loot_table/blocks/{name}.json', {'type': 'minecraft:block', 'pools': [{'rolls': 1, 'entries': [
        {'type': 'minecraft:item', 'name': f'{GNS}:{name}'}], 'conditions': [{'type': 'minecraft:survives_explosion'}]}]})
    kept_blocks.append(name)
    plastic_blocks.append(name)
# the gray glow panel's base is the gray color's painted texture (the white panel would be the original's untinted one)
tinted_texture('glow_panel', 'gray', PLASTIC_COLORS['gray'])
put(f'{OUT}/data/{NS}/tags/block/plastic_fences.json', {'values': [f'{NS}:{n}' for n in plastic_blocks if n.endswith('_plastic_fence') or n.endswith('_plastic_fence_gate')]})
put(f'{OUT}/data/minecraft/tags/block/mineable/pickaxe_plastic.json', {'values': []}) if False else None
for p in sorted(glob.glob(f'{AGEN}/data/mekanismadditions/recipe/**/*.json', recursive=True)):
    if 'pigment_extracting' in p or 'balloon' in p or 'recolor' in p:
        continue
    r = json.loads(open(p).read().replace('mekanismadditions:', f'{NS}:'))
    result = r.get('result', {}).get('id', '').split(':')[-1]
    if result in plastic_blocks:
        # one file per recipe (the path, not the result: a block has several recipes: from sheets and a dye, from its slab, from its stairs...)
        _rel = os.path.relpath(p, f'{AGEN}/data/mekanismadditions/recipe')[:-5].replace(os.sep, '__')
        put(f'{OUT}/data/{NS}/recipe/additions_{_rel}.json', r)
# the crafting recipes whose every ingredient the port has (the solar generator wants a solar panel and an energy tablet,
# the gas-burning one an electrolytic core: none of them is here yet)
_have_now = set(kept_items) | set(kept_blocks)
for p in sorted(glob.glob(f'{GGEN}/data/{GNS}/recipe/generator/*.json')):
    r = read_json(p)
    if r.get('result', {}).get('id', '').split(':')[-1] not in GENERATORS:
        continue
    ingredients = list(refs(r.get('key', r.get('ingredients', []))))
    if all((not s.startswith(NS + ':') or s.split(':')[1] in _have_now) and not s.startswith(GSRC + ':') for s in ingredients):
        r['result']['id'] = f'{NS}:' + r['result']['id'].split(':')[-1]
        put(f'{OUT}/data/{NS}/recipe/generator_{os.path.basename(p)}', r)
glang = read_json(f'{GGEN}/assets/{GSRC}/lang/en_us.json')

# a creative tab with all of it
tab_items = [f'{NS}:{n}' for n in sorted(kept_blocks)] + [f'{NS}:{n}' for n in sorted(kept_items)] 
put(f'{OUT}/data/{NS}/horizon/creative_tab/mekanism.json', {'title': 'Mekanism', 'icon': f'{NS}:steel_casing', 'items': tab_items})


# recipes of the machines (data, as Mekanism has them)
have = set(kept_items) | set(kept_blocks)
def refs(v):
    if isinstance(v, str):
        yield v
    elif isinstance(v, dict):
        for x in v.values():
            yield from refs(x)
    elif isinstance(v, list):
        for x in v:
            yield from refs(x)
nrec = 0
MACHINE_TYPES = {f'{NS}:crushing', f'{NS}:enriching', f'{NS}:sawing', f'{NS}:combining', f'{NS}:metallurgic_infusing',
                 f'{NS}:chemical_conversion', f'{NS}:purifying', f'{NS}:injecting', f'{NS}:compressing', f'{NS}:oxidizing',
                 f'{NS}:dissolution', f'{NS}:crystallizing', f'{NS}:chemical_infusing', f'{NS}:washing', f'{NS}:separating', f'{NS}:rotary', f'{NS}:reaction',
                 f'{NS}:pigment_extracting', f'{NS}:pigment_mixing', f'{NS}:painting', f'{NS}:nucleosynthesizing',
                 f'{NS}:activating', f'{NS}:centrifuging', f'{NS}:evaporating'}
for p in sorted(glob.glob(f'{GEN}/data/{NS}/recipe/**/*.json', recursive=True)):
    rel = os.path.relpath(p, f'{GEN}/data/{NS}/recipe')
    r = read_json(p)
    if r.get('type') not in MACHINE_TYPES:
        continue
    outs = [r.get(k, {}).get('id', '') for k in ('output', 'main_output', 'secondary_output')]
    if 'amount' in r.get('output', {}):
        outs = []  # it makes a chemical, not an item
    if r.get('type') == f'{NS}:reaction':
        outs = [r.get('output', {}).get('item', {}).get('id', '')]
    if any(o.startswith(NS + ':') and o.split(':')[1] not in have for o in outs):
        continue  # makes something the port doesn't have yet
    put(f'{OUT}/data/{NS}/recipe/{rel}', r)
    nrec += 1
# furnace and blast furnace recipes of what was kept (dust to ingot): vanilla types, Mekanism's inputs
for p in sorted(glob.glob(f'{GEN}/data/{NS}/recipe/**/*.json', recursive=True)):
    rel = os.path.relpath(p, f'{GEN}/data/{NS}/recipe')
    r = read_json(p)
    if r.get('type') not in ('minecraft:smelting', 'minecraft:blasting'):
        continue
    if all(not s.startswith(NS + ':') or s.split(':')[1] in have for s in refs(r.get('ingredient', ''))):
        put(f'{OUT}/data/{NS}/recipe/{rel}', r)
        nrec += 1
# crafting recipes of what was kept (vanilla types only)
for p in sorted(glob.glob(f'{GEN}/data/{NS}/recipe/**/*.json', recursive=True)):
    r = read_json(p)
    if not str(r.get('type', '')).startswith('minecraft:crafting_'):
        continue
    res = r.get('result', {})
    rid = res.get('id', '') if isinstance(res, dict) else ''
    if rid.startswith(NS + ':') and rid.split(':')[1] in have:
        # every ingredient must be vanilla, a tag, or ours
        ok = True
        for s in refs(r.get('key', r.get('ingredients', []))):
            if s.startswith(NS + ':') and s.split(':')[1] not in have:
                ok = False
        if ok:
            put(f'{OUT}/data/{NS}/recipe/' + os.path.relpath(p, f'{GEN}/data/{NS}/recipe').replace(os.sep, '__'), r)
            nrec += 1
# the factories' recipes (`mekanism:mek_data` too: the factory below keeps what it holds in the one it makes)
for p in sorted(glob.glob(f'{GEN}/data/{NS}/recipe/factory/*/*.json')):
    r = read_json(p)
    rid = r.get('result', {}).get('id', '')
    if rid.split(':')[-1] in have and all(not s.startswith(NS + ':') or s.split(':')[1] in have for s in refs(r.get('key', {}))):
        r['type'] = 'minecraft:crafting_shaped'
        r.pop('group', None)
        put(f'{OUT}/data/{NS}/recipe/factory_{rid.split(":")[-1]}.json', r)
        nrec += 1
# the energy cubes' recipes (`mekanism:mek_data`: the cube below keeps its energy in the cube it makes), as plain shaped ones
for p in sorted(glob.glob(f'{GEN}/data/{NS}/recipe/energy_cube/*.json')):
    r = read_json(p)
    rid = r.get('result', {}).get('id', '')
    if rid.split(':')[-1] in have and all(not s.startswith(NS + ':') or s.split(':')[1] in have for s in refs(r.get('key', {}))):
        r['type'] = 'minecraft:crafting_shaped'
        put(f'{OUT}/data/{NS}/recipe/energy_cube_{os.path.basename(p)}', r)
        nrec += 1

# ---------------------------------------------------------------- tags
def filtered_tag(v):
    vals = []
    for e in v.get('values', []):
        name = e if isinstance(e, str) else e.get('id', '')
        if name.startswith(NS + ':') and name.split(':')[1] not in have_ids():
            continue
        vals.append(e)
    return {**v, 'values': vals}

def have_ids():
    return set(kept_items) | set(kept_blocks)

ntags = 0
for kind in ('item', 'block'):
    for p in sorted(glob.glob(f'{GEN}/data/c/tags/{kind}/**/*.json', recursive=True)):
        rel = os.path.relpath(p, f'{GEN}/data/c/tags/{kind}')
        v = filtered_tag(read_json(p))
        if v['values']:
            put(f'{OUT}/data/c/tags/{kind}/{rel}', v)
            ntags += 1
# Mekanism's own tags of what was kept (`#mekanism:clumps/iron`, `#mekanism:dirty_dusts/iron`...)
for kind in ('item', 'block'):
    for p in sorted(glob.glob(f'{GEN}/data/{NS}/tags/{kind}/**/*.json', recursive=True)):
        rel = os.path.relpath(p, f'{GEN}/data/{NS}/tags/{kind}')
        v = filtered_tag(read_json(p))
        if v['values']:
            put(f'{OUT}/data/{NS}/tags/{kind}/{rel}', v)
            ntags += 1
# NeoForge gives the vanilla items their convention tags (`c:ingots/iron`...), which Mekanism's recipes
# use and Mekanism's own data does not define: make the ones the recipes ask for.
VANILLA_METALS = ['iron', 'gold', 'copper', 'netherite']
VANILLA_GEMS = {'diamond': 'minecraft:diamond', 'emerald': 'minecraft:emerald', 'lapis': 'minecraft:lapis_lazuli',
                'quartz': 'minecraft:quartz', 'amethyst': 'minecraft:amethyst_shard'}
def vanilla_members(cat, mat):
    v = []
    if cat == 'ingots' and mat in VANILLA_METALS:
        v = [f'minecraft:{mat}_ingot']
    elif cat == 'nuggets' and mat in VANILLA_METALS:
        v = [f'minecraft:{mat}_nugget']
    elif cat == 'raw_materials' and mat in VANILLA_METALS[:3]:
        v = [f'minecraft:raw_{mat}']
    elif cat == 'storage_blocks':
        if mat in VANILLA_METALS or mat in ('coal', 'diamond', 'emerald', 'lapis', 'redstone', 'quartz', 'amethyst'):
            v = [f'minecraft:{mat}_block' if mat != 'lapis' else 'minecraft:lapis_block']
        elif mat.startswith('raw_'):
            v = [f'minecraft:{mat}_block']
    elif cat == 'ores':
        if mat in VANILLA_METALS[:3] or mat in ('coal', 'diamond', 'emerald', 'lapis', 'redstone'):
            v = [f'minecraft:{mat}_ore', f'minecraft:deepslate_{mat}_ore']
    elif cat == 'gems' and mat in VANILLA_GEMS:
        v = [VANILLA_GEMS[mat]]
    elif cat == 'dusts':
        v = {'redstone': ['minecraft:redstone'], 'glowstone': ['minecraft:glowstone_dust']}.get(mat, [])
    elif cat == 'dyes' and mat in ('white', 'orange', 'magenta', 'light_blue', 'yellow', 'lime', 'pink', 'gray', 'light_gray',
                                    'cyan', 'purple', 'blue', 'brown', 'green', 'red', 'black'):
        v = [f'minecraft:{mat}_dye']
    elif cat == 'cobblestones' and mat in ('normal', 'deepslate'):
        v = {'normal': ['minecraft:cobblestone'], 'deepslate': ['minecraft:cobbled_deepslate']}[mat]
    elif cat == 'crops' and mat == 'wheat':
        v = ['minecraft:wheat']
    elif cat == 'pumpkins' and mat == 'normal':
        v = ['minecraft:pumpkin']
    elif cat == 'fences' and mat == 'wooden':
        v = ['#minecraft:wooden_fences']
    elif cat == 'bookshelves' and not mat:
        v = ['minecraft:bookshelf']
    elif cat == 'rods' and mat in ('wooden', 'blaze', 'bone'):
        v = {'wooden': ['minecraft:stick'], 'blaze': ['minecraft:blaze_rod'], 'bone': ['minecraft:bone']}[mat]
    elif cat == 'dyed' and mat:
        v = [f'minecraft:{mat}{suffix}' for suffix in ('_wool', '_carpet', '_bed', '_banner', '_candle', '_concrete', '_concrete_powder',
             '_stained_glass', '_stained_glass_pane', '_glazed_terracotta', '_terracotta', '_shulker_box', '_dye', '_bundle', '_harness')]
    elif cat == 'glass_blocks' and mat == 'cheap':
        v = ['minecraft:glass']
    elif cat == 'buckets' and mat in ('water', 'lava'):
        v = [f'minecraft:{mat}_bucket']
    elif cat == 'player_workstations' and mat == 'furnaces':
        v = ['minecraft:furnace']
    elif cat == 'fuels' and mat == 'coal':
        v = ['minecraft:coal', 'minecraft:charcoal']
    return v

def tag_refs(v):
    for s in refs(v):
        if s.startswith('#c:'):
            yield s[3:]
asked = set()
for p in glob.glob(f'{OUT}/data/**/recipe/**/*.json', recursive=True):
    asked.update(tag_refs(read_json(p)))
for tag in sorted(asked):
    path = f'{OUT}/data/c/tags/item/{tag}.json'
    if os.path.exists(path):
        continue
    cat, mat = tag.split('/', 1) if '/' in tag else (tag, '')
    members = vanilla_members(cat, mat)
    if members:
        put(path, {'values': members})
        ntags += 1

# mining: pickaxe for ores, storage, casings and machines; ores need a stone or iron tool
pick = [f'{NS}:{n}' for n in sorted(BLOCKS)] + [f'{NS}:{t}_energy_cube' for t in CUBE_TIERS] + [f'{GNS}:{g}' for g in GENERATORS] + [f'{GNS}:{g}' for g in TURBINE + FISSION + FUSION] + [f'{GNS}:{n}' for n in plastic_blocks]
put(f'{OUT}/data/minecraft/tags/block/mineable/pickaxe.json', {'values': pick})
need_iron = [f'{NS}:{n}' for n in BLOCKS if n.endswith('_ore') or n.startswith('block_') or n == 'steel_casing']
put(f'{OUT}/data/minecraft/tags/block/needs_iron_tool.json', {'values': need_iron})
# lang: the keys of what was kept
lang = {}
for d in ASSET_DIRS:
    p = f'{d}/lang/en_us.json'
    if os.path.exists(p):
        lang.update(read_json(p))
keep = {f'item.{NS}.{n}' for n in kept_items} | {f'block.{NS}.{n}' for n in kept_blocks}
out = {k: v for k, v in lang.items() if k in keep}
for g in GENERATORS + TURBINE + FISSION + FUSION:
    out[f'block.{NS}.{g}'] = glang[f'block.{GSRC}.{g}']
out[f'block.{NS}.bounding_block'] = 'Bounding Block'
for _k, _v in _additions_lang.items():
    if _k.startswith('block.mekanismadditions.') and _k.split('.')[-1] in plastic_blocks:
        out[f'block.{NS}.' + _k.split('.')[-1]] = _v
for _k, _v in tools_lang.items():
    if _k.startswith('item.mekanismtools.') and _k.split('.')[-1] in kept_items:
        out[f'item.{NS}.' + _k.split('.')[-1]] = _v
out[f'item.{NS}.turbine_blade'] = glang[f'item.{GSRC}.turbine_blade']
out[f'item.{NS}.hohlraum'] = glang[f'item.{GSRC}.hohlraum']
for k, u in enumerate(UPGRADES):
    out[f'item.{NS}.upgrade_{u}'] = f'{UPGRADE_NAMES[k]} Upgrade'
# Item and block names the lang file keeps under other keys: block items use block.*
put(f'{OUT}/assets/{NS}/lang/en_us.json', out)

print(f'items {len(kept_items)}, blocks {len(kept_blocks)}, textures {len(copied_textures)}, '
      f'models {len(copied_models)}, recipes {nrec}')
print('warnings:', len(warnings))
for w in warnings[:40]:
    print('  ', w)

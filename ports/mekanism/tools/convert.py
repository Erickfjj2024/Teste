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
LASERS = ['digital_miner', 'industrial_alarm', 'laser', 'laser_amplifier', 'laser_tractor_beam', 'supercharged_coil', 'logistical_sorter', 'quantum_entangloporter']
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
UTILITY_MACHINES = ['qio_drive_array', 'qio_redstone_adapter', 'radioactive_waste_barrel', 'modification_station', 'formulaic_assemblicator', 'chargepad', 'dimensional_stabilizer', 'seismic_vibrator', 'oredictionificator', 'personal_chest', 'personal_barrel']
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
# The module items (`module_energy_unit`...): their names are the modules' (`module.mekanism.energy_unit`), not `item.mekanism.*` keys.
_lang_en = read_json(f'{GEN}/assets/{NS}/lang/en_us.json')
MODULE_NAMES = {k[len('module.mekanism.'):]: v for k, v in _lang_en.items() if k.startswith('module.mekanism.') and k.count('.') == 2 and k.endswith('_unit')}
ITEM_LANG |= {f'module_{n}' for n in MODULE_NAMES}
# the module items: the ids `modules.rs::ModuleType::item_id` gives (`mekanism:module_energy`...), read from the mod's own source so the two never differ
_modules_src = open(os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), 'mod', 'src', 'modules.rs'), encoding='utf-8').read()
_item_id_fn = _modules_src[_modules_src.index('pub const fn item_id'):_modules_src.index('pub fn from_item')]
MODULE_ITEMS = sorted(set(re.findall(r'"mekanism:module_([a-z_]+)"', _item_id_fn)))
# how many of a module an item holds (`ItemModule` stacks to the module's own maximum): `max_stack` of the same enum, joined to the ids by the variant's name
_max_stack_fn = _modules_src[_modules_src.index('pub const fn max_stack'):_modules_src.index('pub const fn exclusive_flags')]
_stack_of = {m.group(1): int(m.group(2)) for m in re.finditer(r'Self::(\w+) => (\d+),', _max_stack_fn)}
MODULE_STACK = {m.group(2): _stack_of[m.group(1)] for m in re.finditer(r'Self::(\w+) => "mekanism:(module_[a-z_]+)"', _item_id_fn) if m.group(1) in _stack_of}

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
# Items that behave like a vanilla item of the game: the game does the work (the HDPE elytra is an `ElytraItem`; the electric bow a `BowItem` with no
# durability: its energy is the mod's, still to be charged per shot by the mod once the game tells it a bow was released).
ITEM_LIKE = {
    'hdpe_elytra': {'template': 'minecraft:elytra', 'max_stack_size': 1, 'rarity': 'rare',
                    'components': {'minecraft:max_damage': 648, 'minecraft:repairable': {'items': [f'{NS}:hdpe_sheet']}}},
    'electric_bow': {'template': 'minecraft:bow', 'max_stack_size': 1, 'rarity': 'rare',
                     'components': {'!minecraft:max_damage': {}, '!minecraft:repairable': {}}},
}
kept_items = []
for n, model in items.items():
    if not copy_model(model):
        continue
    kept_items.append(n)
    item = dict(ITEM_LIKE.get(n, {'template': 'minecraft:stick'}))
    if n in MODULE_STACK:
        item['max_stack_size'] = MODULE_STACK[n]
    if n in WORN:
        # Equippable(slot): a player wears it in its armor slot, one at a time (the worn model is still to do)
        item['max_stack_size'] = 1
        item['components'] = {'minecraft:equippable': {'slot': WORN[n]}}
    if n == 'portable_teleporter':
        # ItemPortableTeleporter: 1,000,000 J of energy (400,000 FE) charged at 5,000 J (2,000 FE) a call; a trip's cost is taken whole
        item['max_stack_size'] = 1
        item['energy'] = {'capacity': 400000, 'max_insert': 2000, 'max_extract': 400000}
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
    # AttributeRedstoneEmitter (API 50): wires connect to it and it gives the level the mod sets (`world::set_redstone_output`)
    if n in ('laser_amplifier', 'qio_redstone_adapter'):
        block['redstone_emitter'] = True
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
PORTS_ENERGY = ['teleporter', 'induction_port', 'turbine_valve', 'fusion_reactor_port', 'sps_port', 'laser', 'laser_amplifier', 'laser_tractor_beam', 'quantum_entangloporter']
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
GAUGE_SIZES = {'standard': (16, 58), 'small': (16, 28), 'small_med': (16, 46)}
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
    # GuiChemicalWasher: the fluid slots in a holder at the right (`GuiSideHolder.create(this, imageWidth, 66, 57, false, ...)`)
    'chemical_washer': {
        'continuous': True, 'slots': [[180, 71], [152, 56], [180, 102]], 'results': [1, 2],
        'inventory': None, 'title_y': 4,
        'frames': [('slot/input', 180, 71), ('slot/extra', 152, 56), ('slot/output', 180, 102), ('slot/power', 152, 14)],
        'overlays': [('slot/overlay/minus', 152, 56)], 'holder': (176, 66, 57),
        'power': ('horizontal', 115, 75), 'progress': [('large_right', 64, 39)],
        'extra_sprites': [('arrow/down', 184, 91)],
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
    'extra_sprites': [('arrow/down', 159, 44)],
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
# GuiAntiprotonicNucleosynthesizer (196 by 193, the inventory 10 right and 27 down): the item at 26, 40, the antimatter's item slot at 6, 69, the item made at 152, 40,
# the charge slot at 173, 69, the antimatter gauge at 5, 18 and the energy gauge at 172, 18 (both 18 by 48), the inner screen at 45, 18 and the rate bar at 5, 88.
# Not drawn: the lightning between the item slots (`BoltRenderer`).
PROCESS_LAYOUTS['antiprotonic_nucleosynthesizer'] = {
    'ticks': 400, 'width': 196, 'height': 193, 'inv_x': 18, 'inv_shift': 27, 'label_y': 100, 'process_rate': True,
    'slots': [[26, 40], [152, 40], [6, 69]], 'results': [1],
    'frames': [('slot/input', 26, 40), ('slot/output', 152, 40), ('slot/extra', 6, 69), ('slot/power', 173, 69)],
    'overlays': [('slot/overlay/minus', 6, 69)], 'power': ('gauge', 172, 18), 'progress': [], 'screen': (45, 18, 104, 68),
    'rate_bar': (5, 88, 183, 8), 'gauges': [('small_med', 5, 18, 16, 46, 'in', 10000)]}
# GuiNutritionalLiquifier: the food at 26, 36, the item it leaves at 110, 36, the container filled from the paste at 155, 25 (PLUS) and 155, 56 with an arrow
# between them, the charge slot at 155, 5, the arrow at 54, 40 and the tank at 133, 13
PROCESS_LAYOUTS['nutritional_liquifier'] = {**PROCESS_LAYOUTS['chemical_oxidizer'], 'ticks': 100,
    'slots': [[26, 36], [110, 36], [155, 25], [155, 56]], 'results': [1, 3],
    'frames': [('slot/input', 26, 36), ('slot/output', 110, 36), ('slot/extra', 155, 25), ('slot/output', 155, 56), ('slot/power', 155, 5)],
    'overlays': [('slot/overlay/plus', 155, 25)], 'extra_sprites': [('arrow/down', 159, 44)],
    'progress': [('large_right', 54, 40)], 'gauges': [('standard', 133, 13, 16, 58, 'out', 10000)]}
PROCESS_LAYOUTS['pigment_extractor'] = {**PROCESS_LAYOUTS['chemical_oxidizer'], 'ticks': 100,
                                       'tint_arrow': 'out', 'gauges': [('standard', 131, 13, 16, 58, 'out', 20000)]}
PROCESS_LAYOUTS['painting_machine'] = {
    'ticks': 200, 'slots': [[45, 35], [116, 35], [6, 56]], 'results': [1], 'title_y': 4, 'label_y': 74, 'debug': (80, 66),
    'frames': [('slot/input', 45, 35), ('slot/output', 116, 35), ('slot/extra', 6, 56), ('slot/power', 144, 35)],
    'overlays': [('slot/overlay/minus', 6, 56)], 'power': ('vertical', 164, 15), 'progress': [('large_right', 64, 39)],
    'tint_arrow': 'in', 'gauges': [('standard', 25, 13, 16, 58, 'in', 15000)]}
PROCESS_LAYOUTS['pigment_mixer'] = {**PROCESS_LAYOUTS['chemical_infuser'], 'title_y': 5, 'label_y': 74, 'tint_arrow': ['t0', 't1'],
                                    'gauges': [('standard', 25, 13, 16, 58, 't0', 1000), ('standard', 133, 13, 16, 58, 't1', 1000),
                                               ('standard', 79, 4, 16, 58, 't2', 2000)]}
# the nuclear chain (GuiIsotopicCentrifuge, GuiSolarNeutronActivator): a gas slot in at 5, 56 (MINUS) and one out at 155, 56 (PLUS), a gas gauge at each side,
# the arrow at 64, 39, the title and the inventory label two pixels higher; the activator has no energy at all (no bar, no tab, no charge slot)
_nuclear = {**PROCESS_LAYOUTS['chemical_infuser'], 'progress': [('large_right', 64, 39)], 'title': (0, 176, 'center'), 'title_y': 4, 'label_y': 74,
            'inventory': 176, 'slots': [[5, 56], [155, 56]], 'results': [1],
            'frames': [('slot/input', 5, 56), ('slot/output', 155, 56), ('slot/power', 155, 14)],
            'overlays': [('slot/overlay/minus', 5, 56), ('slot/overlay/plus', 155, 56)],
            'gauges': [('standard', 25, 13, 16, 58, 't0', 10000), ('standard', 133, 13, 16, 58, 't1', 10000)]}
PROCESS_LAYOUTS['isotopic_centrifuge'] = _nuclear
PROCESS_LAYOUTS['solar_neutron_activator'] = {**_nuclear, 'no_energy': True, 'frames': _nuclear['frames'][:2]}
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
    ix = L.get('inv_x', 8) - 1
    panel = nine_slice(gui_sprite('base'), L.get('width', 176), L.get('height', 166), 4)
    slot = gui_sprite('slot/normal')
    for row in range(3):
        for col in range(9):
            panel.paste(slot, (ix + 18 * col, 83 + shift + 18 * row))
    for col in range(9):
        panel.paste(slot, (ix + 18 * col, 141 + shift))
    if 'screen' in L:
        x, y, w, h = L['screen']
        # GuiInnerScreen: a dark well with Mekanism's own edge
        panel.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), w, h, 3), (x, y))
    for sprite, x, y in L['frames']:
        panel.alpha_composite(gui_sprite(sprite), (x - 1, y - 1))
    for sprite, x, y in L['overlays']:
        panel.alpha_composite(gui_sprite(sprite), (x - 1, y - 1))
    kind, px, py = L['power']
    if L.get('no_energy'):
        pass
    elif kind == 'gauge':
        # GuiEnergyGauge, SMALL_MED: 18 by 48 around 16 by 46
        panel.alpha_composite(nine_slice(gui_sprite('gauge/normal'), 18, 48, 2), (px, py))
    else:
        panel.alpha_composite(nine_slice(gui_sprite('bar/base'), 6 if kind == 'vertical' else 54, 54 if kind == 'vertical' else 6, 1), (px, py))
    if 'rate_bar' in L:
        rx, ry, rw, rh = L['rate_bar']
        panel.alpha_composite(nine_slice(gui_sprite('bar/base'), rw, rh, 1), (rx, ry))
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

def power_slot(layout):
    """The place of a layout's charge slot: its `slot/power` frame."""
    return next((x, y) for sprite, x, y in layout['frames'] if sprite == 'slot/power')


def with_upgrades(menu, machine, base, c, charge=None):
    """Adds the upgrade window, its tab, its two slots (the filter and the take-only one) and its screen numbers to a menu. `charge` is where the
    machine's charge slot is drawn (mod `upgrades.rs::charge_slot`: the slot after the upgrade ones); the pressurized chamber and the rotary have it among their own."""
    menu['slots'] = base + 2 + (charge is not None)
    menu['results'] = list(menu.get('results', [])) + [base + 1]
    menu['accepts'] = [{'slot': base, 'items': [f'{NS}:upgrade_{UPGRADES[k]}' for k in upgrade_support(machine)]}]
    if charge is not None:
        menu['accepts'].append({'slot': base + 2, 'items': ['minecraft:redstone', 'minecraft:redstone_block']})
    menu['data_slots'] = c + 16
    screen = menu['screen']
    screen['slots'] = list(screen['slots']) + [None, None] + ([list(charge)] if charge is not None else [])
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
    if kind == 'gauge':
        power = {'type': 'sprite', 'texture': energy_gauge_fill(), 'x': px + 1, 'y': py + 1, 'w': 16, 'h': 46, 'direction': 'up',
                 'value': {'pair': [1, 2]}, 'max': {'pair': [3, 4]}, 'tooltip': 'Energy: {value} / {max}'}
    elif kind == 'vertical':
        power = {'type': 'sprite', 'texture': power_fill, 'x': px + 1, 'y': py + 1, 'w': 4, 'h': 52, 'direction': 'up',
                 'value': {'pair': [1, 2]}, 'max': {'pair': [3, 4]}, 'tooltip': 'Energy: {value} / {max}'}
    else:
        power = {'type': 'sprite', 'texture': power_h_fill, 'x': px + 1, 'y': py + 1, 'w': 52, 'h': 4, 'direction': 'right',
                 'value': {'pair': [1, 2]}, 'max': {'pair': [3, 4]}, 'tooltip': 'Energy: {value} / {max}'}
    widgets = []
    # the washer's side holder goes behind the slots
    if 'holder' in L:
        hx, hy, hh = L['holder']
        left = hx < 0
        holder = side_holder(left, hh)
        # the slot frames and the arrows outside the panel are drawn on the holder
        for sprite, x, y in L['frames']:
            if not 0 <= x < 176:
                holder.alpha_composite(gui_sprite(sprite), (x - 1 - hx, y - 1 - hy))
        for sprite, x, y in L.get('extra_sprites', []):
            if not 0 <= x < 176:
                holder.alpha_composite(gui_sprite(sprite), (x - hx, y - hy))
        widgets.append({'type': 'sprite', 'texture': save_gui(holder, f'holder_{"left" if left else "right"}_{hh}'), 'x': hx, 'y': hy, 'w': 26, 'h': hh})
    if not L.get('no_energy'):
        widgets.append(power)
    if kind == 'gauge' and not L.get('no_energy'):
        widgets.append({'type': 'sprite', 'texture': copy_gui('gauge/overlay/small_med', 'gauge_overlay_small_med'), 'x': px + 1, 'y': py + 1, 'w': 16, 'h': 46})
    continuous = L.get('continuous', False)
    ctl0 = 24 if continuous else 20
    width = L.get('width', 176)
    if 'rate_bar' in L:
        # GuiDynamicHorizontalRateBar: the progress as a bar, filled with a colour that goes from (60, 45, 74) to (100, 30, 170)
        from PIL import Image
        rx, ry, rw, rh = L['rate_bar']
        fill = save_gui(Image.new('RGBA', (rw - 2, rh - 2), (100, 30, 170, 255)), 'rate_bar_fill')
        widgets.append({'type': 'sprite', 'texture': fill, 'x': rx + 1, 'y': ry + 1, 'w': rw - 2, 'h': rh - 2, 'value': {'data': 0},
                        'max': {'data': ctl0 + 3}, 'direction': 'right', 'tooltip': 'Progress: {percent}%'})
    if 'screen' in L and L.get('process_rate'):
        # the inner screen's "Process Rate" (energy used over the energy a tick: 100% while it works)
        sx, sy, sw, sh = L['screen']
        for status in range(6):
            text = 'Process Rate: ' + ('100%' if status == 1 else '0%')
            widgets.append({'type': 'show', 'when': {'data': {'value': {'data': 5}, 'eq': status}}, 'widgets': [
                {'type': 'text', 'x': sx + (sw - text_width(text)) // 2, 'y': sy + sh - 11, 'text': text, 'color': '#3CFE9A'}]})
    for k, (ptype, qx, qy) in enumerate(L['progress']):
        pw, ph, direction = PROGRESS_TYPES[ptype]
        # the mixer's arrows are tinted by the pigment each one comes from
        tint_tank = L['tint_arrow'][k] if isinstance(L.get('tint_arrow'), list) else L.get('tint_arrow')
        # a machine that works every tick shows its arrows full while it works (data slot 0 is 1 or 0)
        widgets.append({'type': 'sprite', 'texture': copy_gui(f'progress/{ptype}/full', f'progress_{ptype}_full'), 'x': qx, 'y': qy,
                        'w': pw, 'h': ph, 'value': {'data': 0}, 'max': {'const': 1} if (continuous and not L.get('timed')) else {'data': ctl0 + 3},
                        'direction': direction, 'tooltip': 'Working' if (continuous and not L.get('timed')) else 'Progress: {percent}%',
                        **({'tints': [t['tint'] for t in GAUGE_TYPES], 'tint_by': {'data': TANK_DATA[tint_tank][1]}} if tint_tank else {})})
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
    ctl = 24 if continuous else 20
    dx = width - 176
    widgets += shift_x(redstone_widgets(ctl), dx) + ([] if L.get('no_energy') else energy_tab_widgets(ctl, MACHINE_USAGE[m]))
    widgets += [
        # GuiSideConfigurationTab at -26, 6: opens the side configuration window
        {'type': 'sprite', 'texture': tab_config, 'x': -26, 'y': 6, 'w': 26, 'h': 26},
        {'type': 'button', 'x': -26, 'y': 6, 'w': 26, 'h': 26, 'local': {'open': 'side_config'}, 'tooltip': 'Configuration'}]
    menu = with_upgrades({'slots': len(L['slots']), 'data_slots': (24 if continuous else 20) + 3, 'results': L['results'],
            'screen': {'texture': process_panel(m, L), 'own_labels': True, 'slots': L['slots'], 'widgets': widgets,
                       'width': width, 'height': L.get('height', 166), 'inventory_x': L.get('inv_x', 8), 'inventory_y': 84 + L.get('inv_shift', 0),
                       'windows': [{'id': 'side_config', 'x': WIN_X + dx // 2, 'y': WIN_Y, 'width': WIN_W, 'height': WIN_H,
                                    'texture': window_tex, 'widgets': side_config_widgets(MACHINE_TRANSMISSIONS[m], m, ctl + 15)}]}}, m, len(L['slots']), ctl0,
                         charge=None if L.get('rotary') or L.get('timed') or L.get('no_energy') else power_slot(L))
    if dx:
        # the upgrade tab goes with the wider screen's right edge, its window with its middle
        menu['screen']['widgets'] = menu['screen']['widgets'][:-len(upgrade_tab_widgets())] + shift_x(upgrade_tab_widgets(), dx)
        menu['screen']['windows'][-1]['x'] += dx // 2
    return menu

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
                {'type': 'text', 'x': 8, 'y': 72, 'text': 'Inventory', 'color': '#404040'}]
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

def assemblicator_menu(m):
    """GuiFormulaicAssemblicator (176 by 230): the formula slot, 18 input slots, the 3 by 3 grid, the large output frame with its 6 slots, the charge
    slot, the tall progress arrow, the check or the cross, the six buttons, and the energy and redstone tabs. The screen's numbers are
    `assemblicator.rs::menu_numbers`: 0 ticks worked, 1 ticks a craft takes, 2-3 energy, 4-5 capacity, 6 auto mode, 7 stock control, 8 valid formula,
    9 recipe in the grid, 10 redstone control, 11-12 energy still needed, 13 can encode, 14 energy a tick while working."""
    width, height = 176, 230
    panel = nine_slice(gui_sprite('base'), width, height, 4)
    slot = gui_sprite('slot/normal')
    inv_y = height - 82
    for row in range(3):
        for col in range(9):
            panel.paste(slot, (7 + 18 * col, inv_y - 1 + 18 * row))
    for col in range(9):
        panel.paste(slot, (7 + 18 * col, height - 25))
    panel.alpha_composite(gui_sprite('slot/normal'), (5, 25))
    panel.alpha_composite(gui_sprite('slot/overlay/formula'), (5, 25))
    for y in range(2):
        for x in range(9):
            panel.alpha_composite(slot, (7 + 18 * x, 97 + 18 * y))
    for y in range(3):
        for x in range(3):
            panel.alpha_composite(gui_sprite('slot/normal'), (25 + 18 * x, 16 + 18 * y))
    # SlotType.OUTPUT_LARGE: the original's own 36 by 54 image (found in the 1.21.1 assets; the 26.3 sprites do not have it)
    from PIL import Image
    large = Image.open(find('gui/slot/output_large.png', ASSET_DIRS)).convert('RGBA')
    panel.alpha_composite(large, (115, 16))
    panel.alpha_composite(gui_sprite('slot/power'), (151, 75))
    panel.alpha_composite(gui_sprite('progress/tall_right/empty'), (86, 43))
    panel.alpha_composite(nine_slice(gui_sprite('bar/base'), 6, 54, 1), (159, 15))
    texture = save_gui(panel, m)
    slots = [[6, 26]]
    slots += [[8 + x * 18, 98 + y * 18] for y in range(2) for x in range(9)]
    slots += [[26 + x * 18, 17 + y * 18] for y in range(3) for x in range(3)]
    slots += [[116 + x * 18, 17 + y * 18] for y in range(3) for x in range(2)]
    slots += [[152, 76]]
    results = list(range(28, 34))
    title = MACHINE_NAMES.get(m, 'Formulaic Assemblicator')
    widgets = [
        {'type': 'sprite', 'texture': power_fill, 'x': 160, 'y': 16, 'w': 4, 'h': 52, 'value': {'pair': [2, 3]}, 'max': {'pair': [4, 5]},
         'direction': 'up', 'tooltip': 'Energy: {value} / {max}'},
        {'type': 'sprite', 'texture': copy_gui('progress/tall_right/full', 'progress_tall_right_full'), 'x': 86, 'y': 43, 'w': 22, 'h': 15,
         'value': {'data': 0}, 'max': {'data': 1}, 'direction': 'right', 'tooltip': 'Progress: {percent}%'},
        {'type': 'show', 'when': {'data': {'value': {'data': 9}, 'eq': 1}}, 'widgets': [
            {'type': 'sprite', 'texture': copy_gui('slot/overlay/check', 'overlay_check'), 'x': 88, 'y': 22, 'w': 18, 'h': 18}]},
        {'type': 'show', 'when': {'data': {'value': {'data': 9}, 'eq': 0}}, 'widgets': [
            {'type': 'sprite', 'texture': copy_gui('slot/overlay/x', 'overlay_x'), 'x': 88, 'y': 22, 'w': 18, 'h': 18}]},
        {'type': 'text', 'x': (width - text_width(title)) // 2, 'y': 6, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': 8, 'y': height - 94, 'text': 'Inventory', 'color': '#404040'}]
    def button(icon, name, x, y, size, bid, tip, enabled, tips=None):
        """An image button: the icon always, the button itself only while its `enabled` condition holds (the original greys it out), with a tooltip
        that follows a switch when `tips` is `(data, on, off)`."""
        out = [{'type': 'sprite', 'texture': copy_gui(icon, name), 'x': x, 'y': y, 'w': size, 'h': size}]
        def live(tooltip):
            return {'type': 'button', 'x': x, 'y': y, 'w': size, 'h': size, 'id': bid, 'tooltip': tooltip}
        inner = []
        if tips:
            d, on, off = tips
            inner += [{'type': 'show', 'when': {'data': {'value': {'data': d}, 'eq': 1}}, 'widgets': [live(on)]},
                      {'type': 'show', 'when': {'data': {'value': {'data': d}, 'eq': 0}}, 'widgets': [live(off)]}]
        else:
            inner.append(live(tip))
        out.append({'type': 'show', 'when': {'data': {'value': {'data': enabled[0]}, 'eq': enabled[1]}}, 'widgets': inner})
        return out
    # the buttons' icons are the 1.21.1 ones; the encode button is 14 wide, the others 16 (7900 encode, 7901 stock, 7902 fill or empty, 7903 and
    # 7904 craft, 7905 auto)
    widgets += button('button/encode_formula', 'btn_encode_formula', 7, 45, 14, 7900, 'Encode Formula', (13, 1))
    widgets += button('button/stock_control', 'btn_stock_control', 26, 75, 16, 7901, '', (8, 1), tips=(7, 'Stock Control: on', 'Stock Control: off'))
    # ToggleButton(empty, fill, formula.isEmpty()): with no formula the icon is the "fill" one (the original's own pairing) and the action empties the grid
    widgets += [{'type': 'show', 'when': {'data': {'value': {'data': 8}, 'eq': 0}}, 'widgets':
                 button('button/fill', 'btn_fill', 44, 75, 16, 7902, 'Empty Grid', (6, 0))},
                {'type': 'show', 'when': {'data': {'value': {'data': 8}, 'eq': 1}}, 'widgets':
                 button('button/empty', 'btn_empty', 44, 75, 16, 7902, 'Fill Grid', (6, 0))}]
    widgets += button('button/craft_single', 'btn_craft_single', 71, 75, 16, 7903, 'Craft Single Item', (9, 1))
    widgets += button('button/craft_available', 'btn_craft_available', 89, 75, 16, 7904, 'Craft Available Items', (9, 1))
    widgets += button('button/auto_toggle', 'btn_auto_toggle', 107, 75, 16, 7905, '', (8, 1), tips=(6, 'Auto-Mode: on', 'Auto-Mode: off'))
    # the energy tab (GuiEnergyTab at -26, 137) and the redstone tab (GuiRedstoneControlTab at the screen's width, 137)
    widgets += [{'type': 'sprite', 'texture': tab_energy_info, 'x': -26, 'y': 137, 'w': 26, 'h': 26},
                {'type': 'button', 'x': -26, 'y': 137, 'w': 26, 'h': 26,
                 'tooltip': 'Using: {14} FE/t\nNeeded: {11}+{12} FE\nUnit: Forge Energy'}]
    widgets += redstone_widgets(10)
    return {'slots': len(slots), 'data_slots': 15, 'results': results,
            'accepts': [{'slot': 0, 'items': [f'{NS}:crafting_formula']}, {'slot': 34, 'items': ['minecraft:redstone', 'minecraft:redstone_block']}],
            'screen': {'texture': texture, 'own_labels': True, 'slots': slots, 'width': width, 'height': height,
                       'inventory_x': 8, 'inventory_y': inv_y, 'widgets': widgets}}

def stabilizer_menu(m):
    """GuiDimensionalStabilizer (176 by 166): the 5 by 5 grid of chunks (10 by 10 squares at 63, 19, blue when it loads), the up arrow and the N at its
    left, the charge slot, the power bar, and the energy and redstone tabs. The screen's numbers are `dimensional_stabilizer.rs::menu_numbers`.
    The middle square is always loaded: its left click turns on the first ring with a chunk off and its right click turns off the outermost ring with a
    chunk on (the radius is a number the mod works out). The tooltips say where the chunk is from the stabilizer's own: the original gives the
    world chunk's coordinates, which a screen number of 16 bits cannot carry."""
    width, height = 176, 166
    panel = nine_slice(gui_sprite('base'), width, height, 4)
    slot = gui_sprite('slot/normal')
    for row in range(3):
        for col in range(9):
            panel.paste(slot, (7 + 18 * col, 83 + 18 * row))
    for col in range(9):
        panel.paste(slot, (7 + 18 * col, 141))
    panel.alpha_composite(gui_sprite('slot/power'), (142, 34))
    panel.alpha_composite(gui_sprite('arrow/up'), (52, 28))
    panel.alpha_composite(nine_slice(gui_sprite('bar/base'), 6, 54, 1), (164, 15))
    texture = save_gui(panel, m)
    title = MACHINE_NAMES.get(m, 'Dimensional Stabilizer')
    BLUE = '#366BD0'  # EnumColor.DARK_BLUE
    widgets = [
        {'type': 'sprite', 'texture': power_fill, 'x': 165, 'y': 16, 'w': 4, 'h': 52, 'value': {'pair': [0, 1]}, 'max': {'pair': [2, 3]},
         'direction': 'up', 'tooltip': 'Energy: {value} / {max}'},
        {'type': 'text', 'x': (width - text_width(title)) // 2, 'y': 6, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': 8, 'y': 74, 'text': 'Inventory', 'color': '#404040'},
        {'type': 'text', 'x': 49 - text_width('N') // 2, 'y': 41, 'text': 'N', 'color': '#404040'}]
    for x in range(5):
        for z in range(5):
            px, py = 63 + 10 * x, 19 + 10 * z
            dx, dz = x - 2, z - 2
            if dx == 0 and dz == 0:
                widgets.append({'type': 'bar', 'x': px, 'y': py, 'w': 10, 'h': 10, 'value': {'const': 1}, 'max': {'const': 1}, 'color': BLUE, 'direction': 'up'})
                # the left click: ring `enable`, the right click: ring `disable` (7530 + r and 7540 + r); none when 0
                for enable in (0, 1, 2):
                    for disable in (0, 1, 2):
                        button = {'type': 'button', 'x': px, 'y': py, 'w': 10, 'h': 10,
                                  'tooltip': 'The chunk of the stabilizer is always loaded.'
                                  + ('\n\nLeft click to enable chunk loading for all chunks with a radius of %d.' % enable if enable else '')
                                  + ('\nRight click to disable chunk loading for all chunks with a radius of %d.' % disable if disable else '')}
                        if enable:
                            button['id'] = 7530 + enable
                        if disable:
                            button['alt_id'] = 7540 + disable
                        if 'id' not in button and 'alt_id' not in button:
                            continue
                        if 'id' not in button:
                            button['id'] = 7540 + disable  # only the right click would act: the left one does the same
                        widgets.append({'type': 'show', 'when': {'data': {'value': {'data': 38}, 'eq': enable * 3 + disable}}, 'widgets': [button]})
                continue
            idx = 13 + x * 5 + z
            where = '(%+d, %+d)' % (dx, dz)
            widgets.append({'type': 'bar', 'x': px, 'y': py, 'w': 10, 'h': 10, 'value': {'data': idx}, 'max': {'const': 1}, 'color': BLUE, 'direction': 'up'})
            widgets.append({'type': 'show', 'when': {'data': {'value': {'data': idx}, 'eq': 1}}, 'widgets': [
                {'type': 'button', 'x': px, 'y': py, 'w': 10, 'h': 10, 'id': 7500 + x * 5 + z, 'tooltip': f'Toggle chunk loading on at {where} from here'}]})
            widgets.append({'type': 'show', 'when': {'data': {'value': {'data': idx}, 'eq': 0}}, 'widgets': [
                {'type': 'button', 'x': px, 'y': py, 'w': 10, 'h': 10, 'id': 7500 + x * 5 + z, 'tooltip': f'Toggle chunk loading off at {where} from here'}]})
    widgets += [{'type': 'sprite', 'texture': tab_energy_info, 'x': -26, 'y': 137, 'w': 26, 'h': 26},
                {'type': 'button', 'x': -26, 'y': 137, 'w': 26, 'h': 26,
                 'tooltip': 'Using: {6+7} FE/t\nNeeded: {11+12} FE\nUnit: Forge Energy'}]
    widgets += redstone_widgets(10)
    return {'slots': 1, 'data_slots': 39, 'results': [],
            'accepts': [{'slot': 0, 'items': ['minecraft:redstone', 'minecraft:redstone_block']}],
            'screen': {'texture': texture, 'own_labels': True, 'slots': [[143, 35]], 'width': width, 'height': height, 'widgets': widgets}}

def heater_common(panel_name, slot_xy, power_bar):
    """The 176 by 166 panel of the two heaters: the base, the inventory, their slot's frame and the inner screen at 48, 23 (80 by 42 for the resistive one,
    28 high for the fuelwood one)."""
    panel = nine_slice(gui_sprite('base'), 176, 166, 4)
    slot = gui_sprite('slot/normal')
    for row in range(3):
        for col in range(9):
            panel.paste(slot, (7 + 18 * col, 83 + 18 * row))
    for col in range(9):
        panel.paste(slot, (7 + 18 * col, 141))
    return panel

tab_heat_info = copy_gui('tab/heat_info_k', 'tab_heat_info_k')
HEAT_TIP = 'Temperature: {{{t}}} K\nTransferred: {{{x}}} K/t\nDissipated: {{{d}}} K/t\nUnit: Kelvin'

def resistive_heater_menu(m):
    """GuiResistiveHeater: the inner screen (the temperature and the usage), the charge slot at 15, 35, the power bar, and the energy, heat and redstone
    tabs. Its numbers are `resistive_heater.rs::screen_numbers`. The original has a text field for the usage (7 digits, `PacketGuiSetEnergy`): the
    screen has it too (widget 7600, FORK-20 part 2), with the field empty until typed in; the usage is the line above it."""
    panel = heater_common(m, (15, 35), (164, 15))
    panel.alpha_composite(gui_sprite('slot/power'), (14, 34))
    panel.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), 80, 42, 3), (48, 23))
    panel.alpha_composite(nine_slice(gui_sprite('bar/base'), 6, 54, 1), (164, 15))
    texture = save_gui(panel, m)
    title = MACHINE_NAMES.get(m, 'Resistive Heater')
    widgets = [
        {'type': 'sprite', 'texture': power_fill, 'x': 165, 'y': 16, 'w': 4, 'h': 52, 'value': {'pair': [0, 1]}, 'max': {'pair': [2, 3]},
         'direction': 'up', 'tooltip': 'Energy: {value} / {max}'},
        {'type': 'text', 'x': (176 - text_width(title)) // 2, 'y': 6, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': 8, 'y': 74, 'text': 'Inventory', 'color': '#404040'},
        {'type': 'text', 'x': 52, 'y': 28, 'text': 'Temperature: {8} K', 'color': '#00CD00', 'scale': 85},
        {'type': 'text', 'x': 52, 'y': 38, 'text': 'Usage: {4+5} FE/t', 'color': '#00CD00', 'scale': 85}]
    widgets.append({'type': 'textfield', 'x': 50, 'y': 51, 'w': 76, 'h': 12, 'id': 7600, 'validator': 'digit', 'max_length': 7})
    widgets += [{'type': 'sprite', 'texture': tab_energy_info, 'x': -26, 'y': 137, 'w': 26, 'h': 26},
                {'type': 'button', 'x': -26, 'y': 137, 'w': 26, 'h': 26,
                 'tooltip': 'Using: {6+7} FE/t\nNeeded: {10+11} FE\nUnit: Forge Energy'},
                {'type': 'sprite', 'texture': tab_heat_info, 'x': -26, 'y': 109, 'w': 26, 'h': 26},
                {'type': 'button', 'x': -26, 'y': 109, 'w': 26, 'h': 26, 'tooltip': HEAT_TIP.format(t=8, x=12, d=13)}]
    widgets += redstone_widgets(9)
    return {'slots': 1, 'data_slots': 14, 'results': [],
            'accepts': [{'slot': 0, 'items': ['minecraft:redstone', 'minecraft:redstone_block']}],
            'screen': {'texture': texture, 'own_labels': True, 'slots': [[15, 35]], 'width': 176, 'height': 166, 'widgets': widgets}}

def fuelwood_heater_menu(m):
    """GuiFuelwoodHeater: the inner screen (the temperature and the fuel), the fuel slot at 15, 29, the flame at 144, 31, and the heat tab. Its numbers
    are `fuelwood_heater.rs::screen_numbers`."""
    panel = heater_common(m, (15, 29), None)
    panel.alpha_composite(gui_sprite('slot/normal'), (14, 28))
    panel.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), 80, 28, 3), (48, 23))
    panel.alpha_composite(gui_sprite('progress/flame/empty'), (144, 31))
    texture = save_gui(panel, m)
    title = MACHINE_NAMES.get(m, 'Fuelwood Heater')
    widgets = [
        {'type': 'text', 'x': (176 - text_width(title)) // 2, 'y': 6, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': 8, 'y': 74, 'text': 'Inventory', 'color': '#404040'},
        {'type': 'text', 'x': 52, 'y': 28, 'text': 'Temperature: {0} K', 'color': '#00CD00', 'scale': 85},
        {'type': 'text', 'x': 52, 'y': 38, 'text': 'Fuel: {1}', 'color': '#00CD00', 'scale': 85},
        {'type': 'sprite', 'texture': copy_gui('progress/flame/full', 'progress_flame_full'), 'x': 144, 'y': 31, 'w': 13, 'h': 13,
         'value': {'data': 1}, 'max': {'data': 2}, 'direction': 'up', 'tooltip': 'Burning: {percent}%'},
        {'type': 'sprite', 'texture': tab_heat_info, 'x': -26, 'y': 109, 'w': 26, 'h': 26},
        {'type': 'button', 'x': -26, 'y': 109, 'w': 26, 'h': 26, 'tooltip': HEAT_TIP.format(t=0, x=3, d=4)}]
    return {'slots': 1, 'data_slots': 5, 'results': [],
            'screen': {'texture': texture, 'own_labels': True, 'slots': [[15, 29]], 'width': 176, 'height': 166, 'widgets': widgets}}

def oredictionificator_menu(m):
    """GuiOredictionificator (236 by 230): the filter list (three rows at 10, 18 and a holder around them), the new-filter button, the input slot at 56, 115,
    the large progress arrow at 94, 119, the output slot at 164, 115, and the redstone tab. Its numbers are `oredictionificator.rs::menu_numbers`: 0 it
    converted this tick, 1 the redstone control, 2 how many filters, 3 to 5 whether each of the first three is on. **What the screens of the port cannot
    do yet:** show scroll the list, and make or edit a filter (a text field for the tag and a choice of the result). The first three filters' tag and result are
    shown (`set-menu-text`, API 34), and their switches work (shift-click: the filter's next result); the text box makes a filter from a typed tag (FORK-20 part 2),
    its first result needs `item-tag-members` (not in the host yet). FORK-20 in `PROPOSTA-API.wit`."""
    width, height = 236, 230
    panel = nine_slice(gui_sprite('base'), width, height, 4)
    slot = gui_sprite('slot/normal')
    inv_x, inv_y = 38, height - 82
    for row in range(3):
        for col in range(9):
            panel.paste(slot, (inv_x - 1 + 18 * col, inv_y - 1 + 18 * row))
    for col in range(9):
        panel.paste(slot, (inv_x - 1 + 18 * col, height - 25))
    panel.alpha_composite(nine_slice(gui_sprite('element_holder'), 204, 68, 2), (9, 17))
    panel.alpha_composite(nine_slice(gui_sprite('element_holder'), 204, 22, 2), (9, 85))
    panel.alpha_composite(nine_slice(gui_sprite('button'), 202, 20, 2), (10, 86))
    panel.alpha_composite(gui_sprite('slot/input'), (55, 114))
    panel.alpha_composite(gui_sprite('slot/output'), (163, 114))
    panel.alpha_composite(gui_sprite('progress/large_right/empty'), (94, 119))
    for i in range(3):
        panel.alpha_composite(nine_slice(gui_sprite('button'), 202, 22, 2), (10, 18 + 22 * i))
    texture = save_gui(panel, m)
    title = MACHINE_NAMES.get(m, 'Oredictionificator')
    widgets = [
        {'type': 'text', 'x': (width - text_width(title)) // 2, 'y': 6, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': inv_x, 'y': height - 94, 'text': 'Inventory', 'color': '#404040'},
        {'type': 'sprite', 'texture': copy_gui('progress/large_right/full', 'progress_large_right_full'), 'x': 94, 'y': 119, 'w': 48, 'h': 8,
         'value': {'data': 0}, 'max': {'const': 1}, 'direction': 'right', 'tooltip': 'Converting'},
        {'type': 'text', 'x': 14, 'y': 76, 'text': 'New filter: type a tag (c:ores/iron) and confirm', 'color': '#808080', 'scale': 70},
        {'type': 'textfield', 'x': 12, 'y': 88, 'w': 198, 'h': 16, 'id': 7790, 'validator': 'id', 'max_length': 100},
        ]
    for i in range(3):
        # a row: the tag (text 2i) and the result (text 2i + 1) of the filter, in the colour of its state (green on, red off)
        for state, color in ((1, '#00CD00'), (0, '#CD0000')):
            widgets.append({'type': 'show', 'when': {'data': {'value': {'data': 3 + i}, 'eq': state}}, 'widgets': [
                {'type': 'text', 'x': 14, 'y': 21 + 22 * i, 'text': '', 'text_data': 2 * i, 'color': color, 'scale': 85},
                {'type': 'text', 'x': 14, 'y': 31 + 22 * i, 'text': '', 'text_data': 2 * i + 1, 'color': '#404040', 'scale': 85}]})
    # a filter's row is a button only when the filter exists: the number of filters is tested one count at a time
    for i in range(3):
        for count in range(i + 1, 4):
            widgets.append({'type': 'show', 'when': {'data': {'value': {'data': 2}, 'eq': count}}, 'widgets': [
                {'type': 'button', 'x': 10, 'y': 18 + 22 * i, 'w': 202, 'h': 22, 'id': 7700 + i, 'shift_id': 7740 + i, 'tooltip': f'Switch filter {i + 1} on or off (shift: next result)'}]})
    widgets += shift_x(redstone_widgets(1), width - 176)
    return {'slots': 2, 'data_slots': 6, 'results': [1],
            'screen': {'texture': texture, 'own_labels': True, 'slots': [[56, 115], [164, 115]], 'width': width, 'height': height,
                       'inventory_x': inv_x, 'inventory_y': inv_y, 'widgets': widgets}}

def seismic_vibrator_menu(m):
    """GuiSeismicVibrator (176 by 166): the inner screen at 16, 23 (112 by 40: "Vibrating" or "Idle", and the chunk), the charge slot at 143, 35, the power bar,
    and the energy and redstone tabs. Its numbers are `seismic_vibrator.rs::menu_numbers`; the chunk is the screen's text 0 (`set-menu-text`)."""
    panel = heater_common(m, (143, 35), (164, 15))
    panel.alpha_composite(gui_sprite('slot/power'), (142, 34))
    panel.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), 112, 40, 3), (16, 23))
    panel.alpha_composite(nine_slice(gui_sprite('bar/base'), 6, 54, 1), (164, 15))
    texture = save_gui(panel, m)
    title = MACHINE_NAMES.get(m, 'Seismic Vibrator')
    widgets = [
        {'type': 'sprite', 'texture': power_fill, 'x': 165, 'y': 16, 'w': 4, 'h': 52, 'value': {'pair': [1, 2]}, 'max': {'pair': [3, 4]},
         'direction': 'up', 'tooltip': 'Energy: {value} / {max}'},
        {'type': 'text', 'x': (176 - text_width(title)) // 2, 'y': 6, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': 8, 'y': 74, 'text': 'Inventory', 'color': '#404040'},
        {'type': 'show', 'when': {'data': {'value': {'data': 0}, 'eq': 1}}, 'widgets': [
            {'type': 'text', 'x': 20, 'y': 27, 'text': 'Vibrating', 'color': '#00CD00', 'scale': 85}]},
        {'type': 'show', 'when': {'data': {'value': {'data': 0}, 'eq': 0}}, 'widgets': [
            {'type': 'text', 'x': 20, 'y': 27, 'text': 'Idle', 'color': '#00CD00', 'scale': 85}]},
        {'type': 'text', 'x': 20, 'y': 37, 'text': '', 'text_data': 0, 'color': '#00CD00', 'scale': 85},
        {'type': 'sprite', 'texture': tab_energy_info, 'x': -26, 'y': 137, 'w': 26, 'h': 26},
        {'type': 'show', 'when': {'data': {'value': {'data': 0}, 'eq': 1}}, 'widgets': [
            {'type': 'button', 'x': -26, 'y': 137, 'w': 26, 'h': 26, 'tooltip': 'Using: {5} FE/t\nNeeded: {7+8} FE\nUnit: Forge Energy'}]},
        {'type': 'show', 'when': {'data': {'value': {'data': 0}, 'eq': 0}}, 'widgets': [
            {'type': 'button', 'x': -26, 'y': 137, 'w': 26, 'h': 26, 'tooltip': 'Using: 0 FE/t\nNeeded: {7+8} FE\nUnit: Forge Energy'}]}]
    widgets += redstone_widgets(6)
    return {'slots': 1, 'data_slots': 9, 'results': [],
            'accepts': [{'slot': 0, 'items': ['minecraft:redstone', 'minecraft:redstone_block']}],
            'screen': {'texture': texture, 'own_labels': True, 'slots': [[143, 35]], 'width': 176, 'height': 166, 'widgets': widgets}}

def inventory_panel(width, height, inv_x=8):
    """A panel with the player's inventory at the bottom (3 rows and the hotbar), for the screens that are a grid of slots."""
    panel = nine_slice(gui_sprite('base'), width, height, 4)
    slot = gui_sprite('slot/normal')
    inv_y = height - 82
    for row in range(3):
        for col in range(9):
            panel.paste(slot, (inv_x - 1 + 18 * col, inv_y - 1 + 18 * row))
    for col in range(9):
        panel.paste(slot, (inv_x - 1 + 18 * col, height - 25))
    return panel, slot, inv_y

def personal_storage_menu(m):
    """GuiPersonalStorageTile (176 by 222): 6 rows of 9 slots at 8, 18 and the player's inventory under them; the same screen for the chest and the barrel
    (`PersonalStorageManager.createSlots`)."""
    width, height = 176, 222
    panel, slot, inv_y = inventory_panel(width, height)
    for y in range(6):
        for x in range(9):
            panel.paste(slot, (7 + 18 * x, 17 + 18 * y))
    texture = save_gui(panel, m)
    title = MACHINE_NAMES.get(m, m.replace('_', ' ').title())
    widgets = [{'type': 'text', 'x': (176 - text_width(title)) // 2, 'y': 6, 'text': title, 'color': '#404040'},
               {'type': 'text', 'x': 8, 'y': height - 94, 'text': 'Inventory', 'color': '#404040'}]
    slots = [[8 + x * 18, 18 + y * 18] for y in range(6) for x in range(9)]
    return {'slots': 54, 'data_slots': 0, 'results': [],
            'screen': {'texture': texture, 'own_labels': True, 'slots': slots, 'width': width, 'height': height,
                       'inventory_x': 8, 'inventory_y': inv_y, 'widgets': widgets}}

def qio_drive_array_menu(m):
    """GuiQIODriveArray (176 by 206): the frequency data screen at 15, 19 (144 by 46: the frequency's name, and two bars, the items and the types, with
    what they hold), the 12 drive slots in two rows of six at 34, 70 and the player's inventory. Its numbers and strings are
    `qio.rs::drive_array_screen`. The frequency tab (choosing the frequency) needs the text field and the list of FORK-20 and is not here yet."""
    width, height = 176, 206
    panel, slot, inv_y = inventory_panel(width, height)
    panel.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), 144, 46, 3), (15, 19))
    for y in range(2):
        for x in range(6):
            panel.alpha_composite(slot, (33 + 18 * x, 69 + 18 * y))
    for bx in (26, 98):
        panel.alpha_composite(nine_slice(gui_sprite('bar/base'), 52, 8, 1), (bx, 39))
    texture = save_gui(panel, m)
    title = MACHINE_NAMES.get(m, 'QIO Drive Array')
    widgets = [
        {'type': 'text', 'x': (width - text_width(title)) // 2, 'y': 6, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': 8, 'y': height - 94, 'text': 'Inventory', 'color': '#404040'},
        {'type': 'text', 'x': 17, 'y': 24, 'text': '', 'text_data': 0, 'color': '#00CD00', 'scale': 80},
        {'type': 'bar', 'x': 27, 'y': 40, 'w': 50, 'h': 6, 'value': {'data': 1}, 'max': {'const': 1000}, 'color': '#00CD00', 'direction': 'right',
         'tooltip': 'Items: {percent}%'},
        {'type': 'bar', 'x': 99, 'y': 40, 'w': 50, 'h': 6, 'value': {'data': 2}, 'max': {'const': 1000}, 'color': '#00CD00', 'direction': 'right',
         'tooltip': 'Types: {percent}%'},
        {'type': 'text', 'x': 26 + 25 - text_width('Items') * 8 // 20, 'y': 51, 'text': 'Items', 'color': '#00CD00', 'scale': 80},
        {'type': 'text', 'x': 98 + 25 - text_width('Types') * 8 // 20, 'y': 51, 'text': 'Types', 'color': '#00CD00', 'scale': 80},
        {'type': 'text', 'x': 22, 'y': 58, 'text': '', 'text_data': 1, 'color': '#00CD00', 'scale': 60},
        {'type': 'text', 'x': 94, 'y': 58, 'text': '', 'text_data': 2, 'color': '#00CD00', 'scale': 60}]
    slots = [[34 + 18 * x, 70 + 18 * y] for y in range(2) for x in range(6)]
    drives = [f'{NS}:qio_drive_{k}' for k in ('base', 'hyper_dense', 'time_dilating', 'supermassive')]
    return {'slots': 12, 'data_slots': 4, 'results': [], 'accepts': [{'slot': i, 'items': drives} for i in range(12)],
            'screen': {'texture': texture, 'own_labels': True, 'slots': slots, 'width': width, 'height': height,
                       'inventory_x': 8, 'inventory_y': inv_y, 'widgets': widgets}}

def modification_station_menu(m):
    """GuiModificationStation (176 by 230): the list of the modules the gear holds (28, 20; here six rows, each a button: a click takes one out, Shift takes all
    out, which is the original's "Remove" button on a selected row), the module slot at 35, 118, the gear slot at 125, 118, the charge slot at 151, 21, the
    large progress arrow at 65, 123, the power bar at 156, 40, and the energy and redstone tabs. Its numbers and strings are
    `modification_station.rs::menu_numbers` and `list_texts`. The original's scrolling list and its Remove button need FORK-20's list widget."""
    width, height = 176, 230
    panel, slot, inv_y = inventory_panel(width, height)
    panel.alpha_composite(nine_slice(gui_sprite('element_holder'), 120, 74, 2), (27, 19))
    panel.alpha_composite(gui_sprite('slot/normal'), (34, 117))
    panel.alpha_composite(gui_sprite('slot/overlay/module'), (34, 117))
    panel.alpha_composite(gui_sprite('slot/normal'), (124, 117))
    panel.alpha_composite(gui_sprite('slot/power'), (150, 20))
    panel.alpha_composite(gui_sprite('progress/large_right/empty'), (65, 123))
    panel.alpha_composite(nine_slice(gui_sprite('bar/base'), 6, 54, 1), (156, 40))
    texture = save_gui(panel, m)
    title = MACHINE_NAMES.get(m, 'Modification Station')
    widgets = [
        {'type': 'sprite', 'texture': power_fill, 'x': 157, 'y': 41, 'w': 4, 'h': 52, 'value': {'pair': [2, 3]}, 'max': {'pair': [4, 5]},
         'direction': 'up', 'tooltip': 'Energy: {value} / {max}'},
        {'type': 'sprite', 'texture': copy_gui('progress/large_right/full', 'progress_large_right_full'), 'x': 65, 'y': 123, 'w': 48, 'h': 8,
         'value': {'data': 0}, 'max': {'data': 1}, 'direction': 'right', 'tooltip': 'Installing: {percent}%'},
        {'type': 'text', 'x': (width - text_width(title)) // 2 + 12, 'y': 6, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': 8, 'y': height - 92, 'text': 'Inventory', 'color': '#404040'}]
    for i in range(6):
        widgets.append({'type': 'text', 'x': 31, 'y': 22 + 12 * i, 'text': '', 'text_data': i, 'color': '#404040', 'scale': 90})
        for count in range(i + 1, 7):
            widgets.append({'type': 'show', 'when': {'data': {'value': {'data': 10}, 'eq': count}}, 'widgets': [
                {'type': 'button', 'x': 28, 'y': 21 + 12 * i, 'w': 118, 'h': 12, 'id': 7800 + i, 'shift_id': 7820 + i,
                 'tooltip': 'Click to take one out. Shift + click takes them all out.'}]})
    widgets += [{'type': 'sprite', 'texture': tab_energy_info, 'x': -26, 'y': 137, 'w': 26, 'h': 26},
                {'type': 'show', 'when': {'data': {'value': {'data': 9}, 'eq': 0}}, 'widgets': [
                    {'type': 'button', 'x': -26, 'y': 137, 'w': 26, 'h': 26, 'tooltip': 'Using: 0 FE/t\nNeeded: {7+8} FE\nUnit: Forge Energy'}]},
                {'type': 'button', 'x': -26, 'y': 137, 'w': 26, 'h': 26, 'tooltip': 'Using: {9} FE/t\nNeeded: {7+8} FE\nUnit: Forge Energy'}]
    widgets += redstone_widgets(6)
    gear = [f'{NS}:mekasuit_helmet', f'{NS}:mekasuit_bodyarmor', f'{NS}:mekasuit_pants', f'{NS}:mekasuit_boots', f'{NS}:meka_tool']
    modules = [f'{NS}:module_{n}' for n in MODULE_ITEMS]
    return {'slots': 3, 'data_slots': 11, 'results': [],
            'accepts': [{'slot': 0, 'items': modules}, {'slot': 1, 'items': gear}, {'slot': 2, 'items': ['minecraft:redstone', 'minecraft:redstone_block']}],
            'screen': {'texture': texture, 'own_labels': True, 'slots': [[35, 118], [125, 118], [151, 21]], 'width': width, 'height': height,
                       'inventory_x': 8, 'inventory_y': inv_y, 'widgets': widgets}}

def dynamic_valve_menu(m):
    """GuiDynamicTank (176 by 166): the armor holder at the left, the element holder at 141, 16 (26 by 56) with the two bucket slots at 146, 21 (input or drain) and
    146, 51 (output or fill) and a down arrow between them (150, 39), the inner screen at 49, 21 (84 by 46: what it holds in mB, then the capacity, the screen's strings
    0 to 3 from `dynamic.rs::screen_texts`), the medium gauge at 7, 16 (34 by 56) and the edit mode tab at 176, 138. Numbers (`dynamic.rs::dynamic_menu`): 0-1 the amount in mB,
    2 the chemical, 3-4 the capacity, 5 the edit mode. **Not in the port:** the security and the warning tabs."""
    width, height = 176, 166
    panel, slot, inv_y = inventory_panel(width, height)
    panel.alpha_composite(nine_slice(gui_sprite('gauge/normal'), 34, 56, 2), (7, 16))
    panel.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), 84, 46, 3), (49, 21))
    panel.alpha_composite(nine_slice(gui_sprite('element_holder'), 26, 56, 2), (141, 16))
    for y, kind in ((21, 'input'), (51, 'output')):
        panel.alpha_composite(gui_sprite(f'slot/{kind}'), (145, y - 1))
        panel.alpha_composite(gui_sprite(f'slot/overlay/{kind}'), (145, y - 1))
    panel.alpha_composite(gui_sprite('arrow/down'), (150, 39))
    texture = save_gui(panel, m)
    title = 'Dynamic Tank'
    widgets = [
        {'type': 'sprite', 'texture': armor_holder_left(), 'x': -26, 'y': 62, 'w': 26, 'h': 98},
        {'type': 'gauge', 'x': 8, 'y': 17, 'w': 32, 'h': 54, 'value': {'pair': [0, 1]}, 'max': {'pair': [3, 4]}, 'kind': {'data': 2},
         'types': GAUGE_TYPES, 'tooltip': '{label}: {value} mB', 'empty': 'Empty'},
        {'type': 'sprite', 'texture': copy_gui('gauge/overlay/medium', 'gauge_overlay_medium'), 'x': 8, 'y': 17, 'w': 32, 'h': 54},
        {'type': 'text', 'x': (width - text_width(title)) // 2, 'y': 6, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': 8, 'y': 74, 'text': 'Inventory', 'color': '#404040'}]
    for k in range(4):
        widgets.append({'type': 'text', 'x': 52, 'y': 24 + 10 * k, 'text': '', 'text_data': k, 'color': '#3CFE9A'})
    # GuiContainerEditModeTab
    for k, mode in enumerate(['both', 'fill', 'empty']):
        widgets.append({'type': 'show', 'when': {'data': {'value': {'data': 5}, 'eq': k}}, 'widgets': [
            {'type': 'sprite', 'texture': edit_mode_tab(mode), 'x': 176, 'y': 138, 'w': 26, 'h': 18}]})
    widgets.append({'type': 'button', 'x': 176, 'y': 138, 'w': 26, 'h': 18, 'id': 5100, 'alt_id': 5101, 'tooltip': 'Fill and empty / fill only / empty only'})
    return {'slots': 2, 'data_slots': 6, 'results': [1], 'armor': ARMOR_LEFT,
            'screen': {'texture': texture, 'own_labels': True, 'slots': [[146, 21], [146, 51]], 'widgets': widgets,
                       'width': width, 'height': height, 'inventory_x': 8, 'inventory_y': inv_y}}

def evaporation_menu(m):
    """GuiThermalEvaporationController (196 by 166): the fluid gauges at 6, 13 (input) and 172, 13 (output), the bucket slots at 28, 20 / 28, 51 and 152, 20 /
    152, 51 with a down arrow beside each pair (32, 39 and 156, 39), the inner screen at 48, 19 (100 by 40: Formed, Height, Temperature and Production, the
    screen's strings 0 to 3 from `evaporation.rs::screen_texts`), the temperature bar at 58, 62 (`GuiHorizontalRateBar`: the temperature over 3,000 K), the
    title centred between the gauges, the player's inventory 10 to the right (the screen is 20 wider) and the heat tab (Dissipated). Numbers
    (`evaporation.rs`): 0-1 input, 2-3 its capacity, 4-5 output, 6 and 7 what each holds, 8 temperature times ten, 9 sun, 10 rate times ten, 11 height,
    12 to 14 the heat lost to the air (whole, tenths, hundredths). **Not in the port:** the warning tab (what is stopping the plant)."""
    from PIL import Image
    width, height = 196, 166
    panel = nine_slice(gui_sprite('base'), width, height, 4)
    slot = gui_sprite('slot/normal')
    inv_x = 18
    for row in range(3):
        for col in range(9):
            panel.paste(slot, (inv_x - 1 + 18 * col, 83 + 18 * row))
    for col in range(9):
        panel.paste(slot, (inv_x - 1 + 18 * col, 141))
    for x in (6, 172):
        panel.alpha_composite(nine_slice(gui_sprite('gauge/normal'), 18, 60, 2), (x, 13))
    panel.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), 100, 40, 3), (48, 19))
    for x, y, kind in ((28, 20, 'input'), (28, 51, 'output'), (152, 20, 'input'), (152, 51, 'output')):
        panel.alpha_composite(gui_sprite(f'slot/{kind}'), (x - 1, y - 1))
        panel.alpha_composite(gui_sprite(f'slot/overlay/{kind}'), (x - 1, y - 1))
    for x in (32, 156):
        panel.alpha_composite(gui_sprite('arrow/down'), (x, 39))
    panel.alpha_composite(nine_slice(gui_sprite('bar/base'), 80, 10, 1), (58, 62))
    texture = save_gui(panel, m)
    title = 'Thermal Evaporation Plant'
    rate = Image.new('RGBA', (78, 8))
    src = Image.open(find('gui/bar/horizontal_rate.png', ASSET_DIRS)).convert('RGBA') if find('gui/bar/horizontal_rate.png', ASSET_DIRS) else None
    if src is not None:
        rate.paste(src.crop((0, 0, 78, 8)), (0, 0))
    else:
        for x in range(78):
            for y in range(8):
                rate.putpixel((x, y), (255, 80, 40, 255))
    rate_tex = save_gui(rate, 'bar_horizontal_rate')
    widgets = [
        {'type': 'gauge', 'x': 7, 'y': 14, 'w': 16, 'h': 58, 'value': {'pair': [0, 1]}, 'max': {'pair': [2, 3]}, 'kind': {'data': 6},
         'types': GAUGE_TYPES, 'tooltip': '{label}: {value} mB', 'empty': 'Empty'},
        {'type': 'gauge', 'x': 173, 'y': 14, 'w': 16, 'h': 58, 'value': {'pair': [4, 5]}, 'max': {'const': 10000}, 'kind': {'data': 7},
         'types': GAUGE_TYPES, 'tooltip': '{label}: {value} mB', 'empty': 'Empty'},
        {'type': 'sprite', 'texture': copy_gui('gauge/overlay/standard', 'gauge_overlay_standard'), 'x': 7, 'y': 14, 'w': 16, 'h': 58},
        {'type': 'sprite', 'texture': copy_gui('gauge/overlay/standard', 'gauge_overlay_standard'), 'x': 173, 'y': 14, 'w': 16, 'h': 58},
        {'type': 'text', 'x': 24 + (172 - 24 - text_width(title)) // 2, 'y': 5, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': inv_x, 'y': 74, 'text': 'Inventory', 'color': '#404040'}]
    for k in range(4):
        widgets.append({'type': 'text', 'x': 51, 'y': 22 + 9 * k, 'text': '', 'text_data': k, 'color': '#3CFE9A', 'scale': 80})
    widgets += [
        {'type': 'sprite', 'texture': rate_tex, 'x': 59, 'y': 63, 'w': 78, 'h': 8, 'value': {'data': 8}, 'max': {'const': 30000}, 'direction': 'right',
         'tooltip': 'Temperature: {8} K'},
        {'type': 'sprite', 'texture': tab_heat_info, 'x': -26, 'y': 109, 'w': 26, 'h': 26},
        {'type': 'button', 'x': -26, 'y': 109, 'w': 26, 'h': 26, 'tooltip': 'Dissipated: {12}.{13}{14} K/t'}]
    return {'slots': 4, 'data_slots': 15, 'results': [1, 3],
            'screen': {'texture': texture, 'own_labels': True, 'slots': [[28, 20], [28, 51], [152, 20], [152, 51]], 'width': width, 'height': height,
                       'inventory_x': inv_x, 'inventory_y': 84, 'widgets': widgets}}

def boiler_valve_menu(m):
    """GuiThermoelectricBoiler (218 by 166): four standard gauges, the heated coolant at 6, 13, the water at 26, 13, the steam at 174, 13 and the cooled
    coolant at 194, 13; the two vertical rate bars at 44, 13 (the boil rate over the most it boiled) and 166, 13 (what the most it boiled is of what the
    superheaters could hand over); the inner screen at 54, 23 (110 by 40: the temperature, the boil rate and the most it boils, the screen's strings 0 to 2
    from `boiler.rs::screen_texts`); the title centred at y 5; the player's inventory 21 to the right of the usual place; and the heat tab. The boiler has no
    slots. Numbers (`boiler.rs`): 0-3 water, 4-7 steam, 8-11 heated coolant, 12-15 cooled coolant (amount and capacity halves), 16-19 what each holds, 20
    temperature times ten, 21 boil rate, 22 the most, 23 superheaters, 24 boil capacity, 25 the second bar in thousandths, 26 to 28 the heat lost to the air.
    **Not in the port:** the boiler's second page (`GuiBoilerStats`, the tab on the right) and the warning tab."""
    from PIL import Image
    width, height = 218, 166
    panel = nine_slice(gui_sprite('base'), width, height, 4)
    slot = gui_sprite('slot/normal')
    inv_x = 29
    for row in range(3):
        for col in range(9):
            panel.paste(slot, (inv_x - 1 + 18 * col, 83 + 18 * row))
    for col in range(9):
        panel.paste(slot, (inv_x - 1 + 18 * col, 141))
    # the gauges, left to right: heated coolant, water, steam, cooled coolant (the port's tanks 2, 0, 1, 3)
    order = [(6, 2), (26, 0), (174, 1), (194, 3)]
    for x, _ in order:
        panel.alpha_composite(nine_slice(gui_sprite('gauge/normal'), 18, 60, 2), (x, 13))
    panel.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), 110, 40, 3), (54, 23))
    for x in (44, 166):
        panel.alpha_composite(nine_slice(gui_sprite('bar/base'), 6, 58, 1), (x, 13))
    texture = save_gui(panel, m)
    title = 'Thermoelectric Boiler'
    fill = None
    rate_img = find('gui/bar/vertical_rate.png', ASSET_DIRS)
    if rate_img:
        fill = save_gui(Image.open(rate_img).convert('RGBA'), 'bar_vertical_rate')
    widgets = []
    for x, tank in order:
        widgets.append({'type': 'gauge', 'x': x + 1, 'y': 14, 'w': 16, 'h': 58, 'value': {'pair': [4 * tank, 4 * tank + 1]}, 'max': {'pair': [4 * tank + 2, 4 * tank + 3]},
                        'kind': {'data': 16 + tank}, 'types': GAUGE_TYPES, 'tooltip': '{label}: {value} mB', 'empty': 'Empty'})
        widgets.append({'type': 'sprite', 'texture': copy_gui('gauge/overlay/standard', 'gauge_overlay_standard'), 'x': x + 1, 'y': 14, 'w': 16, 'h': 58})
    widgets += [
        {'type': 'text', 'x': (width - text_width(title)) // 2, 'y': 5, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': inv_x, 'y': 74, 'text': 'Inventory', 'color': '#404040'}]
    for k in range(3):
        widgets.append({'type': 'text', 'x': 57, 'y': 26 + 11 * k, 'text': '', 'text_data': k, 'color': '#3CFE9A', 'scale': 85})
    if fill:
        widgets += [
            {'type': 'sprite', 'texture': fill, 'x': 45, 'y': 14, 'w': 6, 'h': 58, 'value': {'data': 21}, 'max': {'data': 22}, 'direction': 'up', 'tooltip': 'Boil Rate: {21} mB/t'},
            {'type': 'sprite', 'texture': fill, 'x': 167, 'y': 14, 'w': 6, 'h': 58, 'value': {'data': 25}, 'max': {'const': 1000}, 'direction': 'up', 'tooltip': 'Max Boil: {22} mB/t'}]
    widgets += [
        {'type': 'sprite', 'texture': tab_heat_info, 'x': -26, 'y': 109, 'w': 26, 'h': 26},
        {'type': 'button', 'x': -26, 'y': 109, 'w': 26, 'h': 26, 'tooltip': 'Dissipated: {26}.{27}{28} K/t'}]
    return {'slots': 1, 'data_slots': 29, 'results': [],
            'screen': {'texture': texture, 'own_labels': True, 'slots': [None], 'width': width, 'height': height, 'inventory_x': inv_x, 'inventory_y': 84,
                       'widgets': widgets}}

def turbine_valve_menu(m):
    """GuiIndustrialTurbine (190 by 166): the steam gauge (the medium one, 32 wide) at 6, 13, the steam input rate bar at 40, 13, the inner screen at 50, 18
    (126 by 50: the production, the flow, the capacity and the largest flow, the screen's strings 0 to 3 from `turbine.rs::screen_texts`), the energy bar at
    178, 16, the steam mode button (`GuiGasMode`, 173, 72: idle, dumping excess, dumping), the energy tab (storing and producing), the title at y 5 and the
    player's inventory 7 to the right of the usual place. Numbers (`turbine.rs`): 0-5 energy and its capacity (three 16-bit parts), 6-9 steam and its capacity,
    10 what the gauge holds, 11 what it produces now, 12 the most, 13 flow, 14 the largest flow, 15 dispersers, 16 vents, 17 blades, 18 coils, 19 condensers,
    20 blades the coils can use, 21 steam mode, 22 the input rate over the largest flow in thousandths, 23 the energy over its capacity in thousandths,
    24 the energy in thousands. **Not in the port:** the turbine's second page (`GuiTurbineStats`, the tab on the right) and the warning tab."""
    from PIL import Image
    width, height = 190, 166
    panel = nine_slice(gui_sprite('base'), width, height, 4)
    slot = gui_sprite('slot/normal')
    inv_x = 15
    for row in range(3):
        for col in range(9):
            panel.paste(slot, (inv_x - 1 + 18 * col, 83 + 18 * row))
    for col in range(9):
        panel.paste(slot, (inv_x - 1 + 18 * col, 141))
    panel.alpha_composite(nine_slice(gui_sprite('gauge/normal'), 34, 60, 2), (6, 13))
    panel.alpha_composite(nine_slice(gui_sprite('bar/base'), 6, 58, 1), (40, 13))
    panel.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), 126, 50, 3), (50, 18))
    panel.alpha_composite(nine_slice(gui_sprite('bar/base'), 4, 52, 1), (178, 16))
    texture = save_gui(panel, m)
    title = 'Industrial Turbine'
    rate_img = find('gui/bar/vertical_rate.png', ASSET_DIRS)
    fill = save_gui(Image.open(rate_img).convert('RGBA'), 'bar_vertical_rate') if rate_img else None
    modes = [save_gui(Image.open(find(f'gui/gas_mode_{n}.png', ASSET_DIRS)).convert('RGBA'), f'gas_mode_{n}') for n in ('idle', 'excess', 'dump')]
    tips = ['Steam mode: idle', 'Dumping excess Steam\nWater will NOT be recycled', 'Dumping Steam\nWater will NOT be recycled']
    widgets = [
        {'type': 'gauge', 'x': 7, 'y': 14, 'w': 32, 'h': 58, 'value': {'pair': [6, 7]}, 'max': {'pair': [8, 9]}, 'kind': {'data': 10},
         'types': GAUGE_TYPES, 'tooltip': '{label}: {value} mB', 'empty': 'Empty'},
        {'type': 'sprite', 'texture': copy_gui('gauge/overlay/medium', 'gauge_overlay_medium'), 'x': 7, 'y': 14, 'w': 32, 'h': 58},
        {'type': 'text', 'x': (width - text_width(title)) // 2, 'y': 5, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': inv_x, 'y': 74, 'text': 'Inventory', 'color': '#404040'}]
    for k in range(4):
        widgets.append({'type': 'text', 'x': 53, 'y': 21 + 11 * k, 'text': '', 'text_data': k, 'color': '#3CFE9A', 'scale': 85})
    if fill:
        widgets.append({'type': 'sprite', 'texture': fill, 'x': 41, 'y': 14, 'w': 6, 'h': 58, 'value': {'data': 22}, 'max': {'const': 1000}, 'direction': 'up',
                        'tooltip': 'Steam Input: {last} mB/t'.replace('{last}', '{22}')})
    widgets += [
        {'type': 'sprite', 'texture': power_fill, 'x': 179, 'y': 17, 'w': 4, 'h': 52, 'value': {'data': 23}, 'max': {'const': 1000}, 'direction': 'up',
         'tooltip': 'Energy: {24} kFE'},
        {'type': 'sprite', 'texture': tab_energy_info, 'x': -26, 'y': 137, 'w': 26, 'h': 26},
        {'type': 'button', 'x': -26, 'y': 137, 'w': 26, 'h': 26, 'tooltip': 'Storing: {24} kFE\nProducing: {11} FE/t\nUnit: Forge Energy'}]
    for k in range(3):
        widgets.append({'type': 'show', 'when': {'data': {'value': {'data': 21}, 'eq': k}}, 'widgets': [
            {'type': 'sprite', 'texture': modes[k], 'x': 173, 'y': 72, 'w': 10, 'h': 10},
            {'type': 'button', 'x': 173, 'y': 72, 'w': 10, 'h': 10, 'id': 7400, 'tooltip': tips[k]}]})
    return {'slots': 1, 'data_slots': 25,
            'screen': {'texture': texture, 'own_labels': True, 'slots': [None], 'width': width, 'height': height, 'inventory_x': inv_x, 'inventory_y': 84,
                       'widgets': widgets}}

def scale_bar_tex(width, name):
    """The fill of a dynamic rate bar (`GuiDynamicHorizontalRateBar`) in the heat colours."""
    from PIL import Image
    img = Image.new('RGBA', (width, 8))
    for x in range(width):
        level = x / max(1, width - 1)
        color = (int(min(200, 400 * level)), int(max(0, 200 - max(0, -200 + 400 * level))), 0, 255)
        for y in range(8):
            img.putpixel((x, y), color)
    return save_gui(img, name)

stats_tab = save_gui(nine_slice(gui_sprite('holder_right'), 26, 26, 4), 'tab_stats')

def fission_port_menu(m):
    """GuiFissionReactor (195 by 255): the four standard gauges (the coolant at 6, 13, the fuel at 25, 13, the heated coolant at 152, 13, the waste at 171, 13); the
    inner screen at 45, 17 (105 by 56: the status in green or red, the burn rate, the heating rate, the temperature and the damage, the last two in the colours of
    `TileEntityFissionReactorCasing.getTempColor` and `getDamageColor`, the screen's strings 0 to 3 from `fission.rs::screen_texts` and the colour bands 30 and 31);
    the Activate and SCRAM buttons (6, 75 and 89, 75, 81 by 16, each live only in its state), the big light at 173, 76, the temperature bar at 5, 102 (183 wide, the
    heat colours of `ColorFunction.HEAT`), the heat graph at 5, 123 (185 by 38, `GuiDoubleGraph`, at least 1,600 K tall), the heat tab, and the stats tab on the right, which
    opens the page of `GuiFissionReactorStats` as a window: the heat and fuel statistics (the screen's strings 4 to 8), the burn rate bar at 5, 114 and the rate limit box at
    77, 128 (the reactor's own rate limit, `fission.rs::RATE_TEXT`). The player's inventory label is at 6, 163. Numbers (`fission.rs`): 0-3 fuel, 4-7 coolant, 8-11 heated
    coolant, 12-15 waste (amount and capacity halves), 16-19 what each holds, 20 temperature times ten, 21 damage, 22 rate limit times 100, 23 burn rate times 100, 24 the most
    it burns, 25 heating rate, 26 state (0 off, 1 on, 2 forced off), 27 assemblies, 28 boil efficiency, 29 surface, 30 temperature colour, 31 damage colour, 32 to 34 the heat
    lost to the air, 35 the most it burns times 100. **Not in the port:** the warning tab."""
    from PIL import Image
    width, height = 195, 255
    inv_x = 17
    panel, slot, inv_y = inventory_panel(width, height, inv_x)
    for x in (6, 25, 152, 171):
        panel.alpha_composite(nine_slice(gui_sprite('gauge/normal'), 18, 60, 2), (x, 13))
    panel.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), 105, 56, 3), (45, 17))
    for x in (6, 89):
        panel.alpha_composite(nine_slice(gui_sprite('button'), 81, 16, 2), (x, 75))
    panel.alpha_composite(nine_slice(gui_sprite('bar/base'), 183, 8, 1), (5, 102))
    panel.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), 185, 38, 1), (5, 123))
    texture = save_gui(panel, m)
    title = 'Fission Reactor'
    lights = Image.open(find('gui/big_lights.png', ASSET_DIRS)).convert('RGBA')
    light_off = save_gui(lights.crop((0, 0, 12, 12)), 'big_light_off')
    light_on = save_gui(lights.crop((12, 0, 24, 12)), 'big_light_on')
    heat = Image.new('RGBA', (183, 8))
    for x in range(183):
        level = x / 182
        color = (int(min(200, 400 * level)), int(max(0, 200 - max(0, -200 + 400 * level))), 0, 255)
        for y in range(8):
            heat.putpixel((x, y), color)
    heat_tex = save_gui(heat, 'fission_heat_bar')
    widgets = []
    for x, first, kind in ((6, 4, 17), (25, 0, 16), (152, 8, 18), (171, 12, 19)):
        widgets.append({'type': 'gauge', 'x': x + 1, 'y': 14, 'w': 16, 'h': 58, 'value': {'pair': [first, first + 1]}, 'max': {'pair': [first + 2, first + 3]},
                        'kind': {'data': kind}, 'types': GAUGE_TYPES, 'tooltip': '{label}: {value} mB', 'empty': 'Empty'})
        widgets.append({'type': 'sprite', 'texture': copy_gui('gauge/overlay/standard', 'gauge_overlay_standard'), 'x': x + 1, 'y': 14, 'w': 16, 'h': 58})
    widgets += [
        {'type': 'text', 'x': (width - text_width(title)) // 2, 'y': 5, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': 6, 'y': height - 92, 'text': 'Inventory', 'color': '#404040'},
        {'type': 'text', 'x': 5, 'y': 93, 'text': 'Temperature:', 'color': '#404040', 'scale': 90},
        {'type': 'text', 'x': 5, 'y': 114, 'text': 'Heat Graph:', 'color': '#404040', 'scale': 90},
        {'type': 'show', 'when': {'data': {'value': {'data': 26}, 'eq': 1}}, 'widgets': [
            {'type': 'text', 'x': 48, 'y': 20, 'text': 'Status: Active', 'color': '#55FF55', 'scale': 85}]},
        {'type': 'show', 'when': {'data': {'value': {'data': 26}, 'eq': 0}}, 'widgets': [
            {'type': 'text', 'x': 48, 'y': 20, 'text': 'Status: Disabled', 'color': '#FF5555', 'scale': 85}]},
        {'type': 'show', 'when': {'data': {'value': {'data': 26}, 'eq': 2}}, 'widgets': [
            {'type': 'text', 'x': 48, 'y': 20, 'text': 'Status: Disabled', 'color': '#FF5555', 'scale': 85}]},
        {'type': 'text', 'x': 48, 'y': 30, 'text': '', 'text_data': 0, 'color': '#3CFE9A', 'scale': 85},
        {'type': 'text', 'x': 48, 'y': 40, 'text': '', 'text_data': 1, 'color': '#3CFE9A', 'scale': 85}]
    for band, color in enumerate(('#55FF55', '#FFFF55', '#FFAA00', '#FF5555', '#AA0000')):
        widgets.append({'type': 'show', 'when': {'data': {'value': {'data': 30}, 'eq': band}}, 'widgets': [
            {'type': 'text', 'x': 48, 'y': 50, 'text': '', 'text_data': 2, 'color': color, 'scale': 85}]})
    for band, color in enumerate(('#55FF55', '#FFFF55', '#FFAA00', '#AA0000')):
        widgets.append({'type': 'show', 'when': {'data': {'value': {'data': 31}, 'eq': band}}, 'widgets': [
            {'type': 'text', 'x': 48, 'y': 60, 'text': '', 'text_data': 3, 'color': color, 'scale': 85}]})
    # the buttons: each live only in its state
    for state, activate_on, scram_on in ((0, True, False), (1, False, True), (2, False, False)):
        group = []
        group.append({'type': 'text', 'x': 6 + (81 - text_width('Activate')) // 2, 'y': 79, 'text': 'Activate', 'color': '#00AA00' if activate_on else '#808080'})
        group.append({'type': 'text', 'x': 89 + (81 - text_width('SCRAM')) // 2, 'y': 79, 'text': 'SCRAM', 'color': '#AA0000' if scram_on else '#808080'})
        if activate_on:
            group.append({'type': 'button', 'x': 6, 'y': 75, 'w': 81, 'h': 16, 'id': 7100, 'tooltip': 'Activate'})
        if scram_on:
            group.append({'type': 'button', 'x': 89, 'y': 75, 'w': 81, 'h': 16, 'id': 7100, 'tooltip': 'SCRAM'})
        if state == 2:
            group.append({'type': 'button', 'x': 6, 'y': 75, 'w': 81, 'h': 16, 'tooltip': 'Forced off: the reactor melted down and cannot restart until its rods and fuel are reset'})
        widgets.append({'type': 'show', 'when': {'data': {'value': {'data': 26}, 'eq': state}}, 'widgets': group})
    widgets += [
        {'type': 'show', 'when': {'data': {'value': {'data': 26}, 'eq': 1}}, 'widgets': [{'type': 'sprite', 'texture': light_on, 'x': 173, 'y': 76, 'w': 12, 'h': 12}]},
        {'type': 'show', 'when': {'data': {'value': {'data': 26}, 'eq': 0}}, 'widgets': [{'type': 'sprite', 'texture': light_off, 'x': 173, 'y': 76, 'w': 12, 'h': 12}]},
        {'type': 'show', 'when': {'data': {'value': {'data': 26}, 'eq': 2}}, 'widgets': [{'type': 'sprite', 'texture': light_off, 'x': 173, 'y': 76, 'w': 12, 'h': 12}]},
        {'type': 'sprite', 'texture': heat_tex, 'x': 6, 'y': 103, 'w': 183, 'h': 8, 'value': {'data': 20}, 'max': {'const': 18000}, 'direction': 'right',
         'tooltip': 'Temperature: {20} (x0.1 K)'},
        {'type': 'graph', 'x': 5, 'y': 123, 'w': 185, 'h': 38, 'series': [{'value': {'data': 20}, 'color': '#FFFFFF'}], 'samples': 183, 'min_scale': 16000,
         'tooltip': 'Temperature: {20} (x0.1 K)'},
        {'type': 'sprite', 'texture': tab_heat_info, 'x': -26, 'y': 109, 'w': 26, 'h': 26},
        {'type': 'button', 'x': -26, 'y': 109, 'w': 26, 'h': 26, 'tooltip': 'Dissipated: {32}.{33}{34} K/t'},
        {'type': 'sprite', 'texture': stats_tab, 'x': 195, 'y': 6, 'w': 26, 'h': 26},
        {'type': 'button', 'x': 195, 'y': 6, 'w': 26, 'h': 26, 'local': {'open': 'stats'}, 'tooltip': 'Fission Reactor Statistics'}]
    # the stats page
    win = nine_slice(gui_sprite('base'), width, height, 4)
    win.alpha_composite(nine_slice(gui_sprite('bar/base'), 185, 10, 1), (5, 114))
    win_tex = save_gui(win, 'fission_stats_window')
    stats_title = 'Fission Reactor Statistics'
    stats = [
        {'type': 'text', 'x': (width - text_width(stats_title)) // 2, 'y': 5, 'text': stats_title, 'color': '#404040'},
        {'type': 'text', 'x': 6, 'y': 20, 'text': 'Heat Statistics', 'color': '#202020'},
        {'type': 'text', 'x': 6, 'y': 32, 'text': '', 'text_data': 4, 'color': '#404040', 'scale': 90},
        {'type': 'text', 'x': 6, 'y': 42, 'text': '', 'text_data': 5, 'color': '#404040', 'scale': 90},
        {'type': 'text', 'x': 6, 'y': 52, 'text': '', 'text_data': 6, 'color': '#404040', 'scale': 90},
        {'type': 'text', 'x': 6, 'y': 68, 'text': 'Fuel Statistics', 'color': '#202020'},
        {'type': 'text', 'x': 6, 'y': 80, 'text': '', 'text_data': 7, 'color': '#404040', 'scale': 90},
        {'type': 'text', 'x': 6, 'y': 90, 'text': '', 'text_data': 8, 'color': '#404040', 'scale': 90},
        {'type': 'text', 'x': 6, 'y': 104, 'text': 'Current Burn Rate:', 'color': '#404040', 'scale': 90},
        {'type': 'sprite', 'texture': scale_bar_tex(183, 'fission_burn_bar'), 'x': 6, 'y': 115, 'w': 183, 'h': 8, 'value': {'data': 23}, 'max': {'data': 35},
         'direction': 'right', 'tooltip': 'Burn Rate: {23} (x0.01 mB/t)'},
        {'type': 'text', 'x': 5, 'y': 131, 'text': 'Set Rate Limit:', 'color': '#404040', 'scale': 90},
        {'type': 'textfield', 'x': 77, 'y': 128, 'w': 54, 'h': 12, 'id': 7110, 'validator': 'any', 'max_length': 8},
        {'type': 'button', 'x': 6, 'y': 6, 'w': 12, 'h': 12, 'local': 'close', 'tooltip': 'Back'}]
    window = {'id': 'stats', 'x': 0, 'y': 0, 'width': width, 'height': height, 'texture': win_tex, 'widgets': stats}
    return {'slots': 1, 'data_slots': 36, 'results': [],
            'screen': {'texture': texture, 'own_labels': True, 'slots': [None], 'width': width, 'height': height, 'inventory_x': inv_x, 'inventory_y': inv_y,
                       'widgets': widgets, 'windows': [window]}}

def fusion_controller_menu(m):
    """GuiFusionReactorController (186 by 166) and its three pages. The main page: the hohlraum slot at 85, 39, "Formed" at 13, 16, the title at y 5, the player's inventory 5 to
    the right of the usual place, the energy tab (storing and producing), the heat tab (dissipated) and, on the right at 186, the tabs of the heat (y 6), fuel (34) and stats
    (62) pages, each a window with a back button at 6, 6. **Heat** (`GuiFusionReactorHeat`): the plasma temperature gauge at 12, 50 and the case temperature gauge at 66, 50 (a bar
    over 500 million K, the screen's numbers 29 and 30), the arrows at 34, 76 (plasma over case), 88, 61 (case above 0) and 88, 91 (water going to steam), the small water
    gauge at 120, 84, the small steam gauge at 156, 84 and the small energy gauge at 120, 46. **Fuel** (`GuiFusionReactorFuel`): the deuterium gauge (small) at 30, 64, the fuel
    gauge (standard) at 84, 50 and the tritium gauge (small) at 138, 64, the arrows at 52, 76 and 106, 76 while it burns, the injection rate line at y 35 and its box at 103, 115
    (two digits, `fusion.rs::INJECTION_TEXT`). **Stats** (`GuiFusionReactorStats`): the air-cooled lines from y 26 and the water-cooled ones from y 92 (the screen's strings
    3 to 13). Numbers (`fusion.rs`): 0-1 deuterium, 2-3 tritium, 4-5 fuel, 6-7 water, 8-9 steam, 12-13 the water capacity, 14-15 the steam capacity, 16-20 what each holds, 21
    the plasma in thousands of kelvin, 22 the case in tenths, 23 the injection rate, 24 burning, 25-26 the energy, 28 what it makes, 29 and 30 the temperature bars in thousandths,
    31 to 33 the heat lost to the air (whole, tenths, hundredths: the original's figure comes from the heat network, this is the casing's own share), 34 plasma over case,
    35 case above 0, 36 water turning to steam. **Not in the port:** the warning tab and the energy gauge's exact figures on the heat page."""
    from PIL import Image
    width, height = 186, 166
    inv_x = 13
    tank_small = nine_slice(gui_sprite('gauge/normal'), 18, 30, 2)
    tank_std = nine_slice(gui_sprite('gauge/normal'), 18, 60, 2)
    ov_small = copy_gui('gauge/overlay/small', 'gauge_overlay_small')
    ov_std = copy_gui('gauge/overlay/standard', 'gauge_overlay_standard')
    lava = Image.new('RGBA', (16, 58))
    for y in range(58):
        for x in range(16):
            lava.putpixel((x, y), (255, 120 + (x * 7 + y * 3) % 40, 20, 255))
    lava_tex = save_gui(lava, 'lava_gauge_fill')
    arrows = {name: (gui_sprite(f'progress/{name}/empty'), save_gui(gui_sprite(f'progress/{name}/full'), f'fusion_{name}_full')) for name in ('small_right', 'small_left')}
    def base_panel(extra=None):
        panel = nine_slice(gui_sprite('base'), width, height, 4)
        slot = gui_sprite('slot/normal')
        for row in range(3):
            for col in range(9):
                panel.paste(slot, (inv_x - 1 + 18 * col, 83 + 18 * row))
        for col in range(9):
            panel.paste(slot, (inv_x - 1 + 18 * col, 141))
        return panel
    main = base_panel()
    main.paste(gui_sprite('slot/normal'), (84, 38))
    texture = save_gui(main, m)
    tab_icons = {}
    for key, rel in (('heat', 'gui/heat.png'), ('fuel', None), ('stats', 'gui/stats.png')):
        holder = nine_slice(gui_sprite('holder_right'), 26, 26, 4)
        icon_path = find(rel, ASSET_DIRS) if rel else f'{SRC}/src/generators/resources/assets/mekanismgenerators/gui/fuel.png'
        if icon_path and os.path.exists(icon_path):
            icon = Image.open(icon_path).convert('RGBA')
            holder.alpha_composite(icon.resize((18, 18), 0), (4, 4)) if icon.size != (18, 18) else holder.alpha_composite(icon, (4, 4))
        tab_icons[key] = save_gui(holder, f'fusion_tab_{key}')
    back = save_gui(gui_sprite('button/back'), 'fusion_back')
    title = 'Fusion Reactor'
    def tabs_for(skip):
        out = []
        for key, y in (('heat', 6), ('fuel', 34), ('stats', 62)):
            if key == skip:
                continue
            out += [{'type': 'sprite', 'texture': tab_icons[key], 'x': width, 'y': y, 'w': 26, 'h': 26},
                    {'type': 'button', 'x': width, 'y': y, 'w': 26, 'h': 26, 'local': {'open': key}, 'tooltip': {'heat': 'Heat', 'fuel': 'Fuel', 'stats': 'Stats'}[key]}]
        return out
    side = [{'type': 'sprite', 'texture': tab_energy_info, 'x': -26, 'y': 137, 'w': 26, 'h': 26},
            {'type': 'button', 'x': -26, 'y': 137, 'w': 26, 'h': 26, 'tooltip': 'Storing: {25+26} FE\nProducing: {28} FE/t'},
            {'type': 'sprite', 'texture': tab_heat_info, 'x': -26, 'y': 109, 'w': 26, 'h': 26},
            {'type': 'button', 'x': -26, 'y': 109, 'w': 26, 'h': 26, 'tooltip': 'Dissipated: {31}.{32}{33} K/t'}]
    def chem_gauge(x, y, w, h, first, cap, kind, ov, frame_h):
        return [{'type': 'gauge', 'x': x + 1, 'y': y + 1, 'w': w, 'h': h, 'value': {'pair': [first, first + 1]}, 'max': cap, 'kind': {'data': kind},
                 'types': GAUGE_TYPES, 'tooltip': '{label}: {value} mB', 'empty': 'Empty'},
                {'type': 'sprite', 'texture': ov, 'x': x + 1, 'y': y + 1, 'w': w, 'h': h}]
    def arrow(name, x, y, flag):
        empty, full = arrows[name]
        return {'type': 'show', 'when': {'data': {'value': {'data': flag}, 'eq': 1}}, 'widgets': [{'type': 'sprite', 'texture': full, 'x': x, 'y': y, 'w': empty.size[0], 'h': empty.size[1]}]}
    def window(key, panel, widgets, page_title):
        tex = save_gui(panel, f'fusion_{key}_window')
        content = [{'type': 'sprite', 'texture': back, 'x': 6, 'y': 6, 'w': 14, 'h': 14},
                   {'type': 'button', 'x': 6, 'y': 6, 'w': 14, 'h': 14, 'local': 'close', 'tooltip': 'Back'},
                   {'type': 'text', 'x': 18 + (width - 18 - text_width(page_title)) // 2, 'y': 5, 'text': page_title, 'color': '#404040'}] + side + tabs_for(key) + widgets
        return {'id': key, 'x': 0, 'y': 0, 'width': width, 'height': height, 'texture': tex, 'widgets': content}
    # heat
    hp = nine_slice(gui_sprite('base'), width, height, 4)
    hp.alpha_composite(tank_std, (12, 50))
    hp.alpha_composite(tank_std, (66, 50))
    hp.alpha_composite(tank_small, (120, 84))
    hp.alpha_composite(tank_small, (156, 84))
    hp.alpha_composite(tank_small, (120, 46))
    for name, x, y in (('small_right', 34, 76), ('small_right', 88, 61), ('small_right', 88, 91)):
        hp.alpha_composite(arrows[name][0], (x, y))
    heat_w = [
        {'type': 'sprite', 'texture': lava_tex, 'x': 13, 'y': 51, 'w': 16, 'h': 58, 'value': {'data': 29}, 'max': {'const': 1000}, 'direction': 'up', 'tooltip': 'Plasma: {21} kK'},
        {'type': 'sprite', 'texture': ov_std, 'x': 13, 'y': 51, 'w': 16, 'h': 58},
        {'type': 'sprite', 'texture': lava_tex, 'x': 67, 'y': 51, 'w': 16, 'h': 58, 'value': {'data': 30}, 'max': {'const': 1000}, 'direction': 'up', 'tooltip': 'Case: {22} (x0.1 K)'},
        {'type': 'sprite', 'texture': ov_std, 'x': 67, 'y': 51, 'w': 16, 'h': 58},
        arrow('small_right', 34, 76, 34), arrow('small_right', 88, 61, 35), arrow('small_right', 88, 91, 36)]
    heat_w += chem_gauge(120, 84, 16, 28, 6, {'pair': [12, 13]}, 19, ov_small, 30)
    heat_w += chem_gauge(156, 84, 16, 28, 8, {'pair': [14, 15]}, 20, ov_small, 30)
    heat_w += [{'type': 'sprite', 'texture': energy_gauge_fill(), 'x': 121, 'y': 47, 'w': 16, 'h': 28, 'value': {'pair': [25, 26]}, 'max': {'const': 400000000}, 'direction': 'up',
                'tooltip': 'Energy: {25+26} FE'},
               {'type': 'sprite', 'texture': ov_small, 'x': 121, 'y': 47, 'w': 16, 'h': 28}]
    # fuel
    fp = nine_slice(gui_sprite('base'), width, height, 4)
    fp.alpha_composite(tank_small, (30, 64))
    fp.alpha_composite(tank_std, (84, 50))
    fp.alpha_composite(tank_small, (138, 64))
    fp.alpha_composite(arrows['small_right'][0], (52, 76))
    fp.alpha_composite(arrows['small_left'][0], (106, 76))
    fuel_w = chem_gauge(30, 64, 16, 28, 0, {'const': 1000}, 16, ov_small, 30) + chem_gauge(84, 50, 16, 58, 4, {'const': 1000}, 18, ov_std, 60) + chem_gauge(138, 64, 16, 28, 2, {'const': 1000}, 17, ov_small, 30)
    fuel_w += [arrow('small_right', 52, 76, 24), arrow('small_left', 106, 76, 24),
               {'type': 'text', 'x': 16 + (width - 32 - 70) // 2, 'y': 35, 'text': '', 'text_data': 2, 'color': '#404040', 'scale': 90},
               {'type': 'text', 'x': 4, 'y': 117, 'text': 'Edit Rate:', 'color': '#404040', 'scale': 90},
               {'type': 'textfield', 'x': 103, 'y': 115, 'w': 26, 'h': 11, 'id': 7210, 'validator': 'digit', 'max_length': 2}]
    # stats
    sp = nine_slice(gui_sprite('base'), width, height, 4)
    stats_w = [{'type': 'text', 'x': 6, 'y': 26, 'text': 'Air-Cooled', 'color': '#00AA00'}, {'type': 'text', 'x': 6, 'y': 92, 'text': 'Water-Cooled', 'color': '#0000AA'}]
    for k in range(5):
        stats_w.append({'type': 'text', 'x': 6, 'y': 36 + 10 * k, 'text': '', 'text_data': 3 + k, 'color': '#404040', 'scale': 90})
        stats_w.append({'type': 'text', 'x': 6, 'y': 102 + 10 * k, 'text': '', 'text_data': 8 + k, 'color': '#404040', 'scale': 90})
    stats_w.append({'type': 'text', 'x': 6, 'y': 152, 'text': '', 'text_data': 13, 'color': '#404040', 'scale': 90})
    windows = [window('heat', hp, heat_w, 'Heat'), window('fuel', fp, fuel_w, 'Fuel'), window('stats', sp, stats_w, 'Stats')]
    widgets = [
        {'type': 'text', 'x': (width - text_width(title)) // 2, 'y': 5, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': inv_x, 'y': 74, 'text': 'Inventory', 'color': '#404040'},
        {'type': 'text', 'x': 13, 'y': 16, 'text': 'Formed', 'color': '#404040', 'scale': 90}] + side + tabs_for(None)
    return {'slots': 1, 'data_slots': 37, 'accepts': [{'slot': 0, 'items': ['mekanism:hohlraum']}],
            'screen': {'texture': texture, 'own_labels': True, 'slots': [[85, 39]], 'width': width, 'height': height, 'inventory_x': inv_x, 'inventory_y': 84,
                       'widgets': widgets, 'windows': windows}}

def energy_gauge_fill():
    """The energy sprite (its first frame, 16 by 16) tiled over the gauge's 64 by 48 contents."""
    from PIL import Image
    src = Image.open(find('textures/mek_liquid/energy.png', ASSET_DIRS)).convert('RGBA').crop((0, 0, 16, 16))
    img = Image.new('RGBA', (64, 48))
    for x in range(0, 64, 16):
        for y in range(0, 48, 16):
            img.paste(src, (x, y))
    return save_gui(img, 'energy_gauge_fill')

def laser_amplifier_menu(m):
    """GuiLaserAmplifier (176 by 166): the standard energy gauge at 6, 10, the three text boxes (the delay at 96, 28, 36 by 11, four digits; the minimum at
    96, 43 and the maximum at 96, 58, 72 by 11, ten characters, scientific notation allowed), their lines to the left (x 24, y 30, 45 and 60: "Delay: n" or
    "No delay", "Min: ..." and "Max: ...", the screen's strings 0 to 2 from `laser.rs::send_amplifier_texts`), the title centred in what the gauge leaves, and
    the tabs: energy ("Storing"), the output tab at 176, 109 (26 by 18, three icons: off, entity detection, energy contents; left click on, right click back)
    and the redstone control. Numbers (`laser.rs`): 0-1 energy, 2-3 capacity, 4-5 minimum, 6-7 maximum, 8 delay, 9 shot, 10 output mode, 11 redstone control.
    **Not in the port:** the original's entity detection mode needs the laser's damage pass (the host has no laser damage), so it gives no signal."""
    from PIL import Image
    panel = nine_slice(gui_sprite('base'), 176, 166, 4)
    slot = gui_sprite('slot/normal')
    for row in range(3):
        for col in range(9):
            panel.paste(slot, (7 + 18 * col, 83 + 18 * row))
    for col in range(9):
        panel.paste(slot, (7 + 18 * col, 141))
    panel.alpha_composite(nine_slice(gui_sprite('gauge/normal'), 18, 60, 2), (6, 10))
    texture = save_gui(panel, m)
    title = 'Laser Amplifier'
    tabs = []
    for k, name in enumerate(('off', 'entity', 'contents')):
        holder = nine_slice(gui_sprite('holder_right'), 26, 18, 4)
        icon = Image.open(find(f'gui/amplifier_{name}.png', ASSET_DIRS)).convert('RGBA')
        holder.alpha_composite(icon.resize((18, 18), 0), (4, 0))
        tabs.append(save_gui(holder, f'amplifier_tab_{name}'))
    tips = ['Redstone Output: Off', 'Redstone Output: Entity Detection', 'Redstone Output: Energy Contents']
    widgets = [
        {'type': 'sprite', 'texture': energy_gauge_fill(), 'x': 7, 'y': 11, 'w': 16, 'h': 58, 'value': {'pair': [0, 1]}, 'max': {'pair': [2, 3]},
         'direction': 'up', 'tooltip': 'Energy: {value} / {max} FE'},
        {'type': 'sprite', 'texture': copy_gui('gauge/overlay/standard', 'gauge_overlay_standard'), 'x': 7, 'y': 11, 'w': 16, 'h': 58},
        {'type': 'text', 'x': 24 + (176 - 24 - text_width(title)) // 2, 'y': 5, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': 8, 'y': 73, 'text': 'Inventory', 'color': '#404040'},
        {'type': 'text', 'x': 24, 'y': 30, 'text': '', 'text_data': 0, 'color': '#404040', 'scale': 85},
        {'type': 'text', 'x': 24, 'y': 45, 'text': '', 'text_data': 1, 'color': '#404040', 'scale': 85},
        {'type': 'text', 'x': 24, 'y': 60, 'text': '', 'text_data': 2, 'color': '#404040', 'scale': 85},
        {'type': 'textfield', 'x': 96, 'y': 28, 'w': 36, 'h': 11, 'id': 7302, 'validator': 'digit', 'max_length': 4},
        {'type': 'textfield', 'x': 96, 'y': 43, 'w': 72, 'h': 11, 'id': 7300, 'validator': 'any', 'max_length': 10},
        {'type': 'textfield', 'x': 96, 'y': 58, 'w': 72, 'h': 11, 'id': 7301, 'validator': 'any', 'max_length': 10},
        {'type': 'sprite', 'texture': tab_energy_info, 'x': -26, 'y': 137, 'w': 26, 'h': 26},
        {'type': 'button', 'x': -26, 'y': 137, 'w': 26, 'h': 26, 'tooltip': 'Storing: {0+1} FE'}]
    for k in range(3):
        widgets.append({'type': 'show', 'when': {'data': {'value': {'data': 10}, 'eq': k}}, 'widgets': [
            {'type': 'sprite', 'texture': tabs[k], 'x': 176, 'y': 109, 'w': 26, 'h': 18},
            {'type': 'button', 'x': 176, 'y': 109, 'w': 26, 'h': 18, 'id': 7310, 'alt_id': 7311, 'tooltip': tips[k]}]})
    widgets += redstone_widgets(11)
    return {'slots': 1, 'data_slots': 12,
            'screen': {'texture': texture, 'own_labels': True, 'slots': [None], 'widgets': widgets}}

def laser_tractor_beam_menu(m):
    """GuiLaserTractorBeam (176 by 166): 27 output slots in three rows of nine at 8, 16 and the player's inventory under them."""
    width, height = 176, 166
    panel, slot, inv_y = inventory_panel(width, height)
    for y in range(3):
        for x in range(9):
            panel.paste(slot, (7 + 18 * x, 15 + 18 * y))
    texture = save_gui(panel, m)
    title = 'Laser Tractor Beam'
    widgets = [{'type': 'text', 'x': (width - text_width(title)) // 2, 'y': 5, 'text': title, 'color': '#404040'},
               {'type': 'text', 'x': 8, 'y': height - 94, 'text': 'Inventory', 'color': '#404040'}]
    slots = [[8 + x * 18, 16 + y * 18] for y in range(3) for x in range(9)]
    return {'slots': 27, 'data_slots': 0, 'results': [],
            'screen': {'texture': texture, 'own_labels': True, 'slots': slots, 'width': width, 'height': height,
                       'inventory_x': 8, 'inventory_y': inv_y, 'widgets': widgets}}

def sps_port_menu(m):
    """GuiSPS (176 by 182): two standard chemical gauges at 7, 17 (polonium) and 151, 17 (antimatter), the inner screen at 27, 17 (122 by 60: the status, and
    while it works the energy input and the process rate, the screen's strings 0 to 2 from `sps.rs::screen_texts`), and the progress bar at 7, 79
    (160 wide, a purple that goes from rgb(60, 45, 74) to rgb(100, 30, 170) along the bar, `GuiDynamicHorizontalRateBar`). The player's inventory label is at
    height 92."""
    from PIL import Image
    width, height = 176, 182
    panel, slot, inv_y = inventory_panel(width, height)
    for x in (7, 151):
        panel.alpha_composite(nine_slice(gui_sprite('gauge/normal'), 18, 60, 2), (x, 17))
    panel.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), 122, 60, 3), (27, 17))
    panel.alpha_composite(nine_slice(gui_sprite('bar/base'), 162, 10, 1), (7, 79))
    texture = save_gui(panel, m)
    bar = Image.new('RGBA', (160, 8))
    for x in range(160):
        t = x / 158
        color = tuple(round(a + (b - a) * min(t, 1)) for a, b in zip((60, 45, 74), (100, 30, 170))) + (255,)
        for y in range(8):
            bar.putpixel((x, y), color)
    bar_tex = save_gui(bar, 'sps_progress_bar')
    title = 'Supercritical Phase Shifter'
    widgets = [
        {'type': 'gauge', 'x': 8, 'y': 18, 'w': 16, 'h': 58, 'value': {'pair': [0, 1]}, 'max': {'pair': [2, 3]}, 'kind': {'data': 8},
         'types': GAUGE_TYPES, 'tooltip': '{label}: {value} mB', 'empty': 'Empty'},
        {'type': 'gauge', 'x': 152, 'y': 18, 'w': 16, 'h': 58, 'value': {'pair': [4, 5]}, 'max': {'pair': [6, 7]}, 'kind': {'data': 9},
         'types': GAUGE_TYPES, 'tooltip': '{label}: {value} mB', 'empty': 'Empty'},
        {'type': 'sprite', 'texture': copy_gui('gauge/overlay/standard', 'gauge_overlay_standard'), 'x': 8, 'y': 18, 'w': 16, 'h': 58},
        {'type': 'sprite', 'texture': copy_gui('gauge/overlay/standard', 'gauge_overlay_standard'), 'x': 152, 'y': 18, 'w': 16, 'h': 58},
        {'type': 'text', 'x': (width - text_width(title)) // 2, 'y': 5, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': 8, 'y': height - 92, 'text': 'Inventory', 'color': '#404040'},
        {'type': 'text', 'x': 30, 'y': 20, 'text': '', 'text_data': 0, 'color': '#3CFE9A', 'scale': 85},
        {'type': 'text', 'x': 30, 'y': 31, 'text': '', 'text_data': 1, 'color': '#3CFE9A', 'scale': 85},
        {'type': 'text', 'x': 30, 'y': 42, 'text': '', 'text_data': 2, 'color': '#3CFE9A', 'scale': 85},
        {'type': 'sprite', 'texture': bar_tex, 'x': 8, 'y': 80, 'w': 160, 'h': 8, 'value': {'data': 10}, 'max': {'const': 100}, 'direction': 'right',
         'tooltip': 'Progress: {percent}%'}]
    return {'slots': 1, 'data_slots': 15,
            'screen': {'texture': texture, 'own_labels': True, 'slots': [None], 'width': width, 'height': height, 'inventory_x': 8, 'inventory_y': inv_y,
                       'widgets': widgets}}

def sorter_menu(m):
    """GuiLogisticalSorter (276 by 254, `GuiFilterHolder`): the left screen at 9, 17 (85 by 140: the filter count, then "Single Item", "Round Robin" and "Auto Eject" with their On or Off
    and "Default" under them), the toggle buttons at 12, 46 (single item), 12, 76 (round robin, which the port has not) and 12, 106 (auto eject), the holder of the filters at 95, 17
    (158 by 118: four rows 29 high, the list of the screen's strings 1 to 16 from `filters.rs::texts`, a click on a row selects it), the holder of the new filter button at 95, 135 and the
    player's inventory at 58, 172. The new filter button (96, 136) opens the window that makes an item filter or a tag filter from a typed id; the Delete button takes the selected one away.
    Numbers (`sorter.rs`): 0 auto eject, 1 single item, 2 how many filters, 3 the selected row plus one. **Not in the port:** the default colour slot, the colour of each filter, the
    mod-id filter, minimum and maximum counts, the round robin and the filters' order and on/off switches."""
    width, height = 276, 254
    inv_x = 58
    panel, slot, inv_y = inventory_panel(width, height, inv_x)
    panel.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), 85, 140, 3), (9, 17))
    panel.alpha_composite(nine_slice(gui_sprite('element_holder'), 158, 118, 2), (95, 17))
    panel.alpha_composite(nine_slice(gui_sprite('element_holder'), 158, 22, 2), (95, 135))
    panel.alpha_composite(nine_slice(gui_sprite('button'), 100, 20, 2), (96, 136))
    panel.alpha_composite(nine_slice(gui_sprite('button'), 52, 20, 2), (200, 136))
    for y in (46, 76, 106):
        panel.alpha_composite(nine_slice(gui_sprite('button'), 14, 14, 2), (12, y))
    for i in range(4):
        panel.alpha_composite(nine_slice(gui_sprite('button'), 156, 27, 2), (96, 18 + 29 * i))
    texture = save_gui(panel, m)
    title = 'Logistical Sorter'
    widgets = [
        {'type': 'text', 'x': (width - text_width(title)) // 2, 'y': 5, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': inv_x, 'y': height - 94, 'text': 'Inventory', 'color': '#404040'},
        {'type': 'text', 'x': 12, 'y': 21, 'text': '', 'text_data': 0, 'color': '#3CFE9A', 'scale': 80},
        {'type': 'text', 'x': 11, 'y': 36, 'text': 'Single Item', 'color': '#3CFE9A', 'scale': 80},
        {'type': 'text', 'x': 11, 'y': 66, 'text': 'Round Robin', 'color': '#3CFE9A', 'scale': 80},
        {'type': 'text', 'x': 23, 'y': 79, 'text': 'Off', 'color': '#808080', 'scale': 80},
        {'type': 'text', 'x': 11, 'y': 96, 'text': 'Auto Eject', 'color': '#3CFE9A', 'scale': 80},
        {'type': 'text', 'x': 11, 'y': 126, 'text': 'Default', 'color': '#3CFE9A', 'scale': 80}]
    for data, y in ((1, 49), (0, 109)):
        for state, label, color in ((1, 'On', '#3CFE9A'), (0, 'Off', '#FF5555')):
            widgets.append({'type': 'show', 'when': {'data': {'value': {'data': data}, 'eq': state}}, 'widgets': [
                {'type': 'text', 'x': 23, 'y': y, 'text': label, 'color': color, 'scale': 80}]})
    widgets += [
        {'type': 'button', 'x': 12, 'y': 46, 'w': 14, 'h': 14, 'id': 7401, 'tooltip': 'Single item: sends one item at a time'},
        {'type': 'button', 'x': 12, 'y': 106, 'w': 14, 'h': 14, 'id': 7400, 'tooltip': 'Auto eject: sends every item, filters or not'},
        {'type': 'list', 'x': 97, 'y': 19, 'w': 154, 'h': 116, 'row_height': 29, 'first': 1, 'count': {'data': 2}, 'on_click': 7410, 'color': '#404040'},
        {'type': 'text', 'x': 96 + (100 - text_width('New Filter')) // 2, 'y': 142, 'text': 'New Filter', 'color': '#404040'},
        {'type': 'button', 'x': 96, 'y': 136, 'w': 100, 'h': 20, 'local': {'open': 'new'}, 'tooltip': 'Make an item or tag filter'},
        {'type': 'text', 'x': 200 + (52 - text_width('Delete')) // 2, 'y': 142, 'text': 'Delete', 'color': '#AA0000'},
        {'type': 'button', 'x': 200, 'y': 136, 'w': 52, 'h': 20, 'id': 7450, 'tooltip': 'Delete the selected filter'}]
    win = nine_slice(gui_sprite('base'), 160, 90, 4)
    win.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), 140, 12, 2), (10, 30))
    win.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), 140, 12, 2), (10, 62))
    new_w = [{'type': 'text', 'x': 20, 'y': 6, 'text': 'New Filter', 'color': '#404040'},
             {'type': 'button', 'x': 6, 'y': 6, 'w': 12, 'h': 12, 'local': 'close', 'tooltip': 'Close'},
             {'type': 'text', 'x': 10, 'y': 20, 'text': 'Item id (minecraft:iron_ingot), Enter:', 'color': '#404040', 'scale': 70},
             {'type': 'textfield', 'x': 11, 'y': 31, 'w': 138, 'h': 10, 'id': 7451, 'validator': 'any', 'max_length': 100},
             {'type': 'text', 'x': 10, 'y': 52, 'text': 'Tag (c:ingots/iron), Enter:', 'color': '#404040', 'scale': 70},
             {'type': 'textfield', 'x': 11, 'y': 63, 'w': 138, 'h': 10, 'id': 7452, 'validator': 'any', 'max_length': 100}]
    window = {'id': 'new', 'x': 58, 'y': 60, 'width': 160, 'height': 90, 'texture': save_gui(win, 'sorter_new_filter'), 'widgets': new_w}
    return {'slots': 2, 'data_slots': 4,
            'screen': {'texture': texture, 'own_labels': True, 'slots': [None, None], 'width': width, 'height': height, 'inventory_x': inv_x, 'inventory_y': inv_y,
                       'widgets': widgets, 'windows': [window]}}

def entangloporter_menu(m):
    """GuiQuantumEntangloporter (176 by 240): the title at the top, the frequency selector at 27, 14 (`frequency_selector`) and the energy tab (what the frequency stores). The original has no
    slot on its screen; the port keeps the frequency's shared item slot at 153, 7 (an entangloporter's item buffer) so that it can be seen and used. Numbers (`entangloporter.rs`):
    0 has a frequency, 1-2 the energy in thousands, 3-4 the fluid, 5-6 the chemical, 7 and 8 what each holds, 9 how many entangloporters share it, 10 the item count, 11 and 12 how many
    public and private frequencies the lists hold. **Not in the port:** the heat tab (the entangloporter's heat transfer is not modelled) and the warning tab."""
    width, height = 176, 240
    panel, slot, inv_y = inventory_panel(width, height)
    panel.paste(slot, (152, 6))
    selector = frequency_selector(8200, 'sec', panel, 27, 14, 11, 12, 3, 19)
    texture = save_gui(panel, m)
    title = 'Quantum Entangloporter'
    widgets = [
        {'type': 'text', 'x': (width - text_width(title)) // 2, 'y': 4, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': 8, 'y': height - 93, 'text': 'Inventory', 'color': '#404040'},
        {'type': 'sprite', 'texture': tab_energy_info, 'x': -26, 'y': 137, 'w': 26, 'h': 26},
        {'type': 'button', 'x': -26, 'y': 137, 'w': 26, 'h': 26, 'tooltip': 'Storing: {1+2} kFE\nUnit: Forge Energy'}] + selector
    return {'slots': 1, 'data_slots': 13,
            'screen': {'texture': texture, 'own_labels': True, 'slots': [[153, 7]], 'width': width, 'height': height, 'inventory_x': 8, 'inventory_y': inv_y,
                       'widgets': widgets}}

def frequency_selector(prefix_base, sec_var, panel, origin_x, origin_y, public_count, private_count, first_public, first_private):
    """`GuiFrequencySelector` (132 by 121 at `origin_x`, `origin_y`): the public, trusted and private buttons (38 by 20, 4 apart; the trusted one is drawn off, the port has no
    trusted security), the list of 42 high with the name of each frequency (a click on a row sets the block to it, which the original does with its Set button), the frequency,
    owner and security lines under it (the screen's strings 0 to 2), the name box at 23, 99 (98 by 11, Enter makes or joins the frequency of that name in the chosen security),
    and the Delete button (60 by 18, only the owner deletes). The tab chosen is the screen's variable `sec_var` (0 public, 1 private)."""
    from PIL import Image
    x, y = origin_x, origin_y
    panel.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), 122, 42, 3), (x, y + 22))
    panel.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), 98, 11, 2), (x + 23, y + 99))
    panel.alpha_composite(nine_slice(gui_sprite('button'), 60, 18, 2), (x + 62, y + 113))
    pub = Image.open(find('gui/button/public.png', ASSET_DIRS)).convert('RGBA')
    pri = Image.open(find('gui/button/private.png', ASSET_DIRS)).convert('RGBA')
    pub_tex, pri_tex = save_gui(pub, 'frequency_public'), save_gui(pri, 'frequency_private')
    w = []
    for k, (tex, bx) in enumerate(((pub_tex, 0), (pri_tex, 84))):
        w.append({'type': 'sprite', 'texture': tex, 'x': x + bx, 'y': y, 'w': 38, 'h': 20})
        w.append({'type': 'show', 'when': {'local': {'var': sec_var, 'eq': k}}, 'widgets': [
            {'type': 'text', 'x': x + bx + 3, 'y': y + 6, 'text': ('Public', 'Private')[k], 'color': '#FFFFFF', 'scale': 70}]})
        w.append({'type': 'button', 'x': x + bx, 'y': y, 'w': 38, 'h': 20, 'local': {'set': {'var': sec_var, 'value': k}}, 'tooltip': ('Public', 'Private')[k]})
    w.append({'type': 'text', 'x': x + 44, 'y': y + 6, 'text': 'Trusted', 'color': '#808080', 'scale': 70})
    w.append({'type': 'show', 'when': {'local': {'var': sec_var, 'eq': 0}}, 'widgets': [
        {'type': 'list', 'x': x + 1, 'y': y + 23, 'w': 120, 'h': 40, 'row_height': 10, 'first': first_public, 'count': {'data': public_count},
         'on_click': prefix_base, 'color': '#3CFE9A'},
        {'type': 'textfield', 'x': x + 24, 'y': y + 100, 'w': 96, 'h': 9, 'id': prefix_base + 33, 'validator': 'any', 'max_length': 16}]})
    w.append({'type': 'show', 'when': {'local': {'var': sec_var, 'eq': 1}}, 'widgets': [
        {'type': 'list', 'x': x + 1, 'y': y + 23, 'w': 120, 'h': 40, 'row_height': 10, 'first': first_private, 'count': {'data': private_count},
         'on_click': prefix_base + 16, 'color': '#3CFE9A'},
        {'type': 'textfield', 'x': x + 24, 'y': y + 100, 'w': 96, 'h': 9, 'id': prefix_base + 34, 'validator': 'any', 'max_length': 16}]})
    for k, line in enumerate(range(3)):
        w.append({'type': 'text', 'x': x, 'y': y + 67 + 10 * k, 'text': '', 'text_data': k, 'color': '#404040', 'scale': 85})
    w.append({'type': 'text', 'x': x + 64 + (56 - text_width('Delete')) // 2, 'y': y + 118, 'text': 'Delete', 'color': '#AA0000'})
    w.append({'type': 'button', 'x': x + 62, 'y': y + 113, 'w': 60, 'h': 18, 'id': prefix_base + 32, 'tooltip': 'Delete the frequency (only its owner can)'})
    return w

def teleporter_menu(m):
    """GuiTeleporter (176 by 240): the title at the top, the energy slot at 153, 7, the vertical power bar at 158, 26 and the frequency selector at 27, 14
    (`frequency_selector`); the status is the line at 7, 7 (the original draws an icon). Numbers (`teleporter.rs`): 0 has a frequency, 1 the status (0 none, 1 no frame, 2 no
    destination, 3 not enough energy, 4 ready), 2-3 the energy, 4-5 its capacity, 6 how many teleporters are on the frequency, 7 and 8 how many public and private frequencies the
    lists hold. The strings (`frequency.rs::texts`): 0 the frequency, 1 its owner, 2 its security, 3 to 18 the public rows and 19 to 34 the private ones."""
    width, height = 176, 240
    panel, slot, inv_y = inventory_panel(width, height)
    panel.alpha_composite(gui_sprite('slot/power'), (152, 6))
    panel.alpha_composite(nine_slice(gui_sprite('bar/base'), 6, 54, 1), (157, 25))
    selector = frequency_selector(8100, 'sec', panel, 27, 14, 7, 8, 3, 19)
    texture = save_gui(panel, m)
    title = 'Teleporter'
    widgets = [
        {'type': 'sprite', 'texture': power_fill, 'x': 158, 'y': 26, 'w': 4, 'h': 52, 'value': {'pair': [2, 3]}, 'max': {'pair': [4, 5]}, 'direction': 'up',
         'tooltip': 'Energy: {value} / {max} FE'},
        {'type': 'text', 'x': 22 + (130 - text_width(title)) // 2, 'y': 4, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': 8, 'y': height - 93, 'text': 'Inventory', 'color': '#404040'}]
    for st, text, color in ((0, 'No frequency', '#CD0000'), (1, 'No frame', '#CD0000'), (2, 'No link', '#CD0000'), (3, 'No energy', '#CD0000'), (4, 'Ready', '#00AA00')):
        widgets.append({'type': 'show', 'when': {'data': {'value': {'data': 1}, 'eq': st}}, 'widgets': [{'type': 'text', 'x': 7, 'y': 7, 'text': text, 'color': color, 'scale': 80}]})
    widgets += selector
    return {'slots': 1, 'data_slots': 9, 'results': [], 'accepts': [],
            'screen': {'texture': texture, 'own_labels': True, 'slots': [[153, 7]], 'width': width, 'height': height, 'inventory_x': 8, 'inventory_y': inv_y,
                       'widgets': widgets}}

def portable_teleporter_menu(m):
    """GuiPortableTeleporter (176 by 172): the status at 7, 7 (the original draws an icon), the vertical power bar at 158, 26, the frequency selector at 27, 14
    (`frequency_selector`) and the Teleport button at 42, 147 (92 by 20). Numbers and strings as the teleporter block's (`portable_teleporter.rs::portable_view`). The original shows no
    player inventory; the host has no menu without one yet, so it is drawn under the panel (FORK request: a menu without the inventory and with no slots)."""
    width, height = 176, 172
    panel = nine_slice(gui_sprite('base'), width, height, 4)
    panel.alpha_composite(nine_slice(gui_sprite('bar/base'), 6, 54, 1), (157, 25))
    panel.alpha_composite(nine_slice(gui_sprite('button'), 92, 20, 2), (42, 147))
    selector = frequency_selector(8300, 'sec', panel, 27, 14, 7, 8, 3, 19)
    texture = save_gui(panel, m)
    title = 'Portable Teleporter'
    widgets = [
        {'type': 'sprite', 'texture': power_fill, 'x': 158, 'y': 26, 'w': 4, 'h': 52, 'value': {'pair': [2, 3]}, 'max': {'pair': [4, 5]}, 'direction': 'up',
         'tooltip': 'Energy: {value} / {max} FE'},
        {'type': 'text', 'x': 22 + (130 - text_width(title)) // 2, 'y': 4, 'text': title, 'color': '#404040'}]
    for st, text, color in ((0, 'No frequency', '#CD0000'), (2, 'No link', '#CD0000'), (3, 'No energy', '#CD0000'), (4, 'Ready', '#00AA00')):
        widgets.append({'type': 'show', 'when': {'data': {'value': {'data': 1}, 'eq': st}}, 'widgets': [{'type': 'text', 'x': 7, 'y': 7, 'text': text, 'color': color, 'scale': 80}]})
    widgets += selector
    widgets.append({'type': 'text', 'x': 42 + (92 - text_width('Teleport')) // 2, 'y': 153, 'text': 'Teleport', 'color': '#FFFFFF'})
    widgets.append({'type': 'button', 'x': 42, 'y': 147, 'w': 92, 'h': 20, 'id': 8390, 'tooltip': 'Teleport'})
    return {'slots': 1, 'data_slots': 9, 'results': [],
            'screen': {'texture': texture, 'own_labels': True, 'slots': [[-40, -40]], 'width': width, 'height': height, 'inventory_x': 8, 'inventory_y': height + 4,
                       'widgets': widgets}}

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
        {'type': 'text', 'x': 62, 'y': 48, 'text': 'of {2+3}', 'color': '#00CD00', 'scale': 80}]
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
    """GuiDigitalMiner (176 by 242) and its config page (`GuiDigitalMinerConfig`, 276 by 254, a window). The main page: the buffer's 27 slots at 8, 92, the energy slot at 152, 20, the inner
    screen at 7, 19 (77 by 69: Idle, Running or Low power; the searcher's state; "To mine: n"), the three digital switches at 19, 56 (auto eject), 38, 56 (auto pull, which the port
    has not) and 57, 56 (silk touch), the vertical power bar at 157, 39 (47 high), the Start, Stop, Config and Reset buttons at 87, 19 and 17 apart (61 by 18, each live only when the
    original's is) and the energy tab. The config page: the left screen at 9, 17 (the filter count, the radius, the lowest and highest height with their boxes at 13, 45, 13, 71 and
    13, 98), the inverse switch at 10, 115, the list of filters at 95, 17 (four rows of 29, as the sorter's), New Filter and Delete under it and the back button at 5, 5. Numbers
    (`miner.rs`): 0 running, 1-2 energy, 3-4 its capacity, 5 radius, 6 and 7 the lowest and highest height, 8 eject, 9 silk, 10 inverse, 11 queue, 12 mined, 13 status (0 stopped, 1
    searching, 2 mining, 3 no energy, 4 finished), 14 how many filters, 15 the selected row plus one; the strings 0 to 16 are `filters.rs::texts`. **Not in the port:** the auto pull switch,
    the replace target of the inverse mode and its "requires replacement" switch, the visuals tab, the missing block slot, the mod-id filter and the warnings."""
    from PIL import Image
    width, height = 176, 242
    panel, slot, inv_y = inventory_panel(width, height)
    for i in range(27):
        panel.alpha_composite(slot, (7 + 18 * (i % 9), 91 + 18 * (i // 9)))
    panel.alpha_composite(gui_sprite('slot/power'), (151, 19))
    panel.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), 77, 69, 3), (7, 19))
    panel.alpha_composite(nine_slice(gui_sprite('bar/base'), 6, 49, 1), (156, 38))
    for k in range(4):
        panel.alpha_composite(nine_slice(gui_sprite('button'), 61, 18, 2), (87, 19 + 17 * k))
    for x in (19, 38, 57):
        panel.alpha_composite(nine_slice(gui_sprite('button'), 16, 16, 2), (x, 56))
    texture = save_gui(panel, m)
    title = 'Digital Miner'
    def switch_icon(name):
        return save_gui(Image.open(find(f'gui/switch/{name}.png', ASSET_DIRS)).convert('RGBA').resize((10, 10), 0), f'switch_{name}')
    widgets = [
        {'type': 'text', 'x': (width - text_width(title)) // 2, 'y': 5, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': 8, 'y': height - 94, 'text': 'Inventory', 'color': '#404040'},
        {'type': 'sprite', 'texture': power_fill, 'x': 157, 'y': 39, 'w': 4, 'h': 47, 'value': {'pair': [1, 2]}, 'max': {'pair': [3, 4]}, 'direction': 'up',
         'tooltip': 'Energy: {value} / {max} FE'}]
    for state, label, color in ((0, 'Idle', '#3CFE9A'), (1, 'Running', '#3CFE9A'), (2, 'Running', '#3CFE9A'), (3, 'Low Power', '#FF5555'), (4, 'Idle', '#3CFE9A')):
        widgets.append({'type': 'show', 'when': {'data': {'value': {'data': 13}, 'eq': state}}, 'widgets': [{'type': 'text', 'x': 10, 'y': 22, 'text': label, 'color': color, 'scale': 85}]})
    for state, label in ((0, 'Not ready'), (1, 'Searching...'), (2, 'Searching...'), (3, 'Paused'), (4, 'Finished')):
        widgets.append({'type': 'show', 'when': {'data': {'value': {'data': 13}, 'eq': state}}, 'widgets': [{'type': 'text', 'x': 10, 'y': 32, 'text': label, 'color': '#3CFE9A', 'scale': 85}]})
    widgets.append({'type': 'text', 'x': 10, 'y': 42, 'text': 'To mine: {11}', 'color': '#3CFE9A', 'scale': 85})
    for x, name, data, bid, tip in ((19, 'eject', 8, 7807, 'Auto eject'), (38, 'input', None, None, 'Auto pull (not in the port yet)'), (57, 'silk', 9, 7808, 'Silk touch')):
        widgets.append({'type': 'sprite', 'texture': switch_icon(name), 'x': x + 3, 'y': 59, 'w': 10, 'h': 10})
        if data is not None:
            widgets.append({'type': 'show', 'when': {'data': {'value': {'data': data}, 'eq': 1}}, 'widgets': [
                {'type': 'text', 'x': x + 5, 'y': 70, 'text': 'On', 'color': '#3CFE9A', 'scale': 70}]})
        b = {'type': 'button', 'x': x, 'y': 56, 'w': 16, 'h': 16, 'tooltip': tip}
        if bid:
            b['id'] = bid
        widgets.append(b)
    # Start and Stop each live only when the original's are: Start when it is not running, Stop when it is
    for label, bid, y, live in (('Start', 7811, 19, (0, 4)), ('Stop', 7812, 36, (1, 2, 3))):
        for st in range(5):
            on = st in live
            group = [{'type': 'text', 'x': 87 + (61 - text_width(label)) // 2, 'y': y + 5, 'text': label, 'color': '#404040' if on else '#808080'}]
            if on:
                group.append({'type': 'button', 'x': 87, 'y': y, 'w': 61, 'h': 18, 'id': bid, 'tooltip': label})
            widgets.append({'type': 'show', 'when': {'data': {'value': {'data': 13}, 'eq': st}}, 'widgets': group})
    for st in range(5):
        on = st in (0, 4)
        group = [{'type': 'text', 'x': 87 + (61 - text_width('Config')) // 2, 'y': 58, 'text': 'Config', 'color': '#404040' if on else '#808080'}]
        if on:
            group.append({'type': 'button', 'x': 87, 'y': 53, 'w': 61, 'h': 18, 'local': {'open': 'config'}, 'tooltip': 'Radius, heights and filters'})
        widgets.append({'type': 'show', 'when': {'data': {'value': {'data': 13}, 'eq': st}}, 'widgets': group})
    widgets += [{'type': 'text', 'x': 87 + (61 - text_width('Reset')) // 2, 'y': 75, 'text': 'Reset', 'color': '#404040'},
                {'type': 'button', 'x': 87, 'y': 70, 'w': 61, 'h': 18, 'id': 7810, 'tooltip': 'Stop and start the search again'},
                {'type': 'sprite', 'texture': tab_energy_info, 'x': -26, 'y': 137, 'w': 26, 'h': 26},
                {'type': 'button', 'x': -26, 'y': 137, 'w': 26, 'h': 26, 'tooltip': 'Capacity: {3+4} FE\nStored: {1+2} FE'}]
    # the config page
    cw, ch = 276, 254
    cp = nine_slice(gui_sprite('base'), cw, ch, 4)
    cp.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), 85, 140, 3), (9, 17))
    cp.alpha_composite(nine_slice(gui_sprite('element_holder'), 158, 118, 2), (95, 17))
    cp.alpha_composite(nine_slice(gui_sprite('element_holder'), 158, 22, 2), (95, 135))
    cp.alpha_composite(nine_slice(gui_sprite('button'), 100, 20, 2), (96, 136))
    cp.alpha_composite(nine_slice(gui_sprite('button'), 52, 20, 2), (200, 136))
    cp.alpha_composite(nine_slice(gui_sprite('button'), 16, 16, 2), (10, 115))
    for i in range(4):
        cp.alpha_composite(nine_slice(gui_sprite('button'), 156, 27, 2), (96, 18 + 29 * i))
    for y in (45, 71, 98):
        cp.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), 38, 11, 2), (13, y))
    ctitle = 'Digital Miner'
    cfg = [
        {'type': 'text', 'x': 14 + (cw - 14 - text_width(ctitle)) // 2, 'y': 5, 'text': ctitle, 'color': '#404040'},
        {'type': 'button', 'x': 5, 'y': 5, 'w': 11, 'h': 14, 'local': 'close', 'tooltip': 'Back'},
        {'type': 'text', 'x': 12, 'y': 22, 'text': '', 'text_data': 0, 'color': '#3CFE9A', 'scale': 80},
        {'type': 'text', 'x': 12, 'y': 35, 'text': 'Radius: {5}', 'color': '#3CFE9A', 'scale': 80},
        {'type': 'textfield', 'x': 14, 'y': 46, 'w': 36, 'h': 9, 'id': 7863, 'validator': 'digit', 'max_length': 2},
        {'type': 'text', 'x': 12, 'y': 61, 'text': 'Min Y: {6}', 'color': '#3CFE9A', 'scale': 80},
        {'type': 'textfield', 'x': 14, 'y': 72, 'w': 36, 'h': 9, 'id': 7864, 'validator': 'any', 'max_length': 4},
        {'type': 'text', 'x': 12, 'y': 88, 'text': 'Max Y: {7}', 'color': '#3CFE9A', 'scale': 80},
        {'type': 'textfield', 'x': 14, 'y': 99, 'w': 36, 'h': 9, 'id': 7865, 'validator': 'any', 'max_length': 4},
        {'type': 'sprite', 'texture': switch_icon('inverse'), 'x': 13, 'y': 118, 'w': 10, 'h': 10},
        {'type': 'show', 'when': {'data': {'value': {'data': 10}, 'eq': 1}}, 'widgets': [{'type': 'text', 'x': 30, 'y': 120, 'text': 'Inverse: On', 'color': '#3CFE9A', 'scale': 75}]},
        {'type': 'show', 'when': {'data': {'value': {'data': 10}, 'eq': 0}}, 'widgets': [{'type': 'text', 'x': 30, 'y': 120, 'text': 'Inverse: Off', 'color': '#FF5555', 'scale': 75}]},
        {'type': 'button', 'x': 10, 'y': 115, 'w': 16, 'h': 16, 'id': 7809, 'tooltip': 'Inverse mode: mine everything the filters do not name'},
        {'type': 'list', 'x': 97, 'y': 19, 'w': 154, 'h': 116, 'row_height': 29, 'first': 1, 'count': {'data': 14}, 'on_click': 7830, 'color': '#404040'},
        {'type': 'text', 'x': 96 + (100 - text_width('New Filter')) // 2, 'y': 142, 'text': 'New Filter', 'color': '#404040'},
        {'type': 'button', 'x': 96, 'y': 136, 'w': 100, 'h': 20, 'local': {'open': 'new_filter'}, 'tooltip': 'Make an item or tag filter'},
        {'type': 'text', 'x': 200 + (52 - text_width('Delete')) // 2, 'y': 142, 'text': 'Delete', 'color': '#AA0000'},
        {'type': 'button', 'x': 200, 'y': 136, 'w': 52, 'h': 20, 'id': 7860, 'tooltip': 'Delete the selected filter'}]
    nw = nine_slice(gui_sprite('base'), 160, 90, 4)
    nw.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), 140, 12, 2), (10, 30))
    nw.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), 140, 12, 2), (10, 62))
    new_w = [{'type': 'text', 'x': 20, 'y': 6, 'text': 'New Filter', 'color': '#404040'},
             {'type': 'button', 'x': 6, 'y': 6, 'w': 12, 'h': 12, 'local': 'close', 'tooltip': 'Close'},
             {'type': 'text', 'x': 10, 'y': 20, 'text': 'Block id (minecraft:diamond_ore), Enter:', 'color': '#404040', 'scale': 70},
             {'type': 'textfield', 'x': 11, 'y': 31, 'w': 138, 'h': 10, 'id': 7861, 'validator': 'any', 'max_length': 100},
             {'type': 'text', 'x': 10, 'y': 52, 'text': 'Tag (c:ores/iron), Enter:', 'color': '#404040', 'scale': 70},
             {'type': 'textfield', 'x': 11, 'y': 63, 'w': 138, 'h': 10, 'id': 7862, 'validator': 'any', 'max_length': 100}]
    windows = [{'id': 'config', 'x': -50, 'y': -6, 'width': cw, 'height': ch, 'texture': save_gui(cp, 'miner_config'), 'widgets': cfg},
               {'id': 'new_filter', 'x': 8, 'y': 60, 'width': 160, 'height': 90, 'texture': save_gui(nw, 'miner_new_filter'), 'widgets': new_w}]
    slots = [[8 + 18 * (i % 9), 92 + 18 * (i // 9)] for i in range(27)] + [[152, 20]]
    return {'slots': 28, 'data_slots': 16, 'results': [],
            'screen': {'texture': texture, 'own_labels': True, 'slots': slots, 'width': width, 'height': height, 'inventory_x': 8, 'inventory_y': inv_y,
                       'widgets': widgets, 'windows': windows}}

def dictionary_menu(m):
    """GuiDictionary (176 by 171): the slot to put an item in at 5, 5 (the original's target at 6, 6), the list of its tags at 7, 29 (162 by 42, four rows: the strings 1 on, the first string says
    what is shown), the "Tag Type" dropdown at 113, 73 (here only Item, the one the host can list) and the player's inventory under them. Numbers (`item_menus.rs`): 0 how many tags.
    **Not in the port:** the other tag types (block, fluid, entity, enchantment, chemical...)."""
    width, height = 176, 171
    panel, slot, inv_y = inventory_panel(width, height)
    panel.alpha_composite(gui_sprite('slot/normal'), (5, 5))
    panel.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), 162, 42, 3), (7, 29))
    panel.alpha_composite(nine_slice(gui_sprite('button'), 56, 14, 2), (113, 73))
    texture = save_gui(panel, m)
    title = 'Dictionary'
    widgets = [
        {'type': 'text', 'x': 28 + (width - 28 - text_width(title)) // 2, 'y': 5, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': 28, 'y': 17, 'text': '', 'text_data': 0, 'color': '#404040', 'scale': 75},
        {'type': 'text', 'x': 8, 'y': height - 96, 'text': 'Inventory', 'color': '#404040'},
        {'type': 'text', 'x': 8, 'y': 76, 'text': 'Tag Type', 'color': '#404040', 'scale': 90},
        {'type': 'text', 'x': 117, 'y': 77, 'text': 'Item', 'color': '#404040'},
        {'type': 'list', 'x': 9, 'y': 31, 'w': 158, 'h': 40, 'row_height': 10, 'first': 1, 'count': {'data': 0}, 'on_click': 7000, 'color': '#3CFE9A'}]
    return {'slots': 1, 'data_slots': 1, 'results': [],
            'screen': {'texture': texture, 'own_labels': True, 'slots': [[6, 6]], 'width': width, 'height': height, 'inventory_x': 8, 'inventory_y': inv_y,
                       'widgets': widgets}}

def induction_port_menu(m):
    """GuiInductionMatrix (176 by 166): the medium energy gauge at 7, 16 (34 by 56), the inner screen at 49, 21 (84 by 46: the energy, the capacity, what went in and what
    went out last tick, the screen's strings 0 to 3 from `induction.rs::screen_texts`), the element holder at 141, 16 (26 by 56) with the two charge slots at 146, 21 (the matrix charges the item, PLUS) and 146, 51 (an item that
    holds energy gives it to the matrix, any other gives what it is worth, MINUS), the armor holder at the left, the energy tab (storing
    and the in and out rates) and the title at the top. Numbers (`induction.rs`): 0-1 the energy in millions of Forge Energy, 2-3 the capacity, 4-5 the most it moves in a tick in
    thousands, 6 cells, 7 providers, 8 the energy over the capacity in thousandths, 9-10 what went in last tick and 11-12 what went out, in thousands. **Not in the port:** the
    matrix's second page (`GuiMatrixStats`), the warning tab, and the two slots are each port's own, not shared by the whole structure."""
    width, height = 176, 166
    panel, slot, inv_y = inventory_panel(width, height)
    panel.alpha_composite(nine_slice(gui_sprite('gauge/normal'), 34, 56, 2), (7, 16))
    panel.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), 84, 46, 3), (49, 21))
    panel.alpha_composite(nine_slice(gui_sprite('element_holder'), 26, 56, 2), (141, 16))
    # the two charge slots at 146, 21 (the top one charges an item, PLUS) and 146, 51 (the bottom one fills the matrix from an item, MINUS)
    for y, overlay in ((21, 'slot/overlay/plus'), (51, 'slot/overlay/minus')):
        panel.alpha_composite(gui_sprite('slot/normal'), (145, y - 1))
        panel.alpha_composite(gui_sprite(overlay), (145, y - 1))
    texture = save_gui(panel, m)
    title = 'Induction Matrix'
    widgets = [
        {'type': 'sprite', 'texture': armor_holder_left(), 'x': -26, 'y': 36, 'w': 26, 'h': 98},
        {'type': 'sprite', 'texture': energy_gauge_fill(), 'x': 8, 'y': 17, 'w': 32, 'h': 54, 'value': {'data': 8}, 'max': {'const': 1000}, 'direction': 'up',
         'tooltip': 'Energy: {0+1} / {2+3} MFE'},
        {'type': 'sprite', 'texture': copy_gui('gauge/overlay/medium', 'gauge_overlay_medium'), 'x': 8, 'y': 17, 'w': 32, 'h': 54},
        {'type': 'text', 'x': (width - text_width(title)) // 2, 'y': 5, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': 8, 'y': height - 92, 'text': 'Inventory', 'color': '#404040'}]
    for k in range(4):
        widgets.append({'type': 'text', 'x': 52, 'y': 24 + 11 * k, 'text': '', 'text_data': k, 'color': '#3CFE9A', 'scale': 80})
    widgets += [{'type': 'sprite', 'texture': tab_energy_info, 'x': -26, 'y': 137, 'w': 26, 'h': 26},
                {'type': 'button', 'x': -26, 'y': 137, 'w': 26, 'h': 26, 'tooltip': 'Storing: {0+1} / {2+3} MFE\nInput: {9+10} kFE/t\nOutput: {11+12} kFE/t'}]
    return {'slots': 2, 'data_slots': 13, 'results': [],
            'armor': {'x': -20, 'y': 41, 'offhand': 0},
            'screen': {'texture': texture, 'own_labels': True, 'slots': [[146, 21], [146, 51]], 'width': width, 'height': height, 'inventory_x': 8, 'inventory_y': inv_y,
                       'widgets': widgets}}

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
        ]
    # GuiInnerScreen: the energy, for the plenisher whether it is finished, and the fluid (`pump.rs::screen_texts`); the pumps have no side configuration
    for k in range(3 if m == 'fluidic_plenisher' else 2):
        widgets.append({'type': 'text', 'x': 57, 'y': 26 + 9 * k, 'text': '', 'text_data': k, 'color': '#3CFE9A'})
    widgets += redstone_widgets(24) + energy_tab_widgets(24, MACHINE_USAGE[m])
    return with_upgrades({'slots': 3, 'data_slots': 24 + 3, 'results': [1],
                          'screen': {'texture': texture, 'own_labels': True, 'slots': [[28, 20], [28, 51], [143, 35]],
                                     'widgets': widgets}}, m, 3, 24)

put(f'{OUT}/data/{NS}/horizon/menu/dynamic_valve.json', dynamic_valve_menu('dynamic_valve'))
put(f'{OUT}/data/{NS}/horizon/menu/formulaic_assemblicator.json', assemblicator_menu('formulaic_assemblicator'))
put(f'{OUT}/data/{NS}/horizon/menu/dimensional_stabilizer.json', stabilizer_menu('dimensional_stabilizer'))
put(f'{OUT}/data/{NS}/horizon/menu/resistive_heater.json', resistive_heater_menu('resistive_heater'))
put(f'{OUT}/data/{NS}/horizon/menu/fuelwood_heater.json', fuelwood_heater_menu('fuelwood_heater'))
put(f'{OUT}/data/{NS}/horizon/menu/oredictionificator.json', oredictionificator_menu('oredictionificator'))
put(f'{OUT}/data/{NS}/horizon/menu/seismic_vibrator.json', seismic_vibrator_menu('seismic_vibrator'))
put(f'{OUT}/data/{NS}/horizon/menu/modification_station.json', modification_station_menu('modification_station'))
put(f'{OUT}/data/{NS}/horizon/menu/qio_drive_array.json', qio_drive_array_menu('qio_drive_array'))
for _p in ('personal_chest', 'personal_barrel'):
    put(f'{OUT}/data/{NS}/horizon/menu/{_p}.json', personal_storage_menu(_p))
put(f'{OUT}/data/{NS}/horizon/menu/thermal_evaporation_controller.json', evaporation_menu('thermal_evaporation_controller'))
put(f'{OUT}/data/{NS}/horizon/menu/boiler_valve.json', boiler_valve_menu('boiler_valve'))
put(f'{OUT}/data/{NS}/horizon/menu/turbine_valve.json', turbine_valve_menu('turbine_valve'))
put(f'{OUT}/data/{NS}/horizon/menu/fission_reactor_port.json', fission_port_menu('fission_reactor_port'))
put(f'{OUT}/data/{NS}/horizon/menu/fusion_reactor_controller.json', fusion_controller_menu('fusion_reactor_controller'))
put(f'{OUT}/data/{NS}/horizon/menu/laser_amplifier.json', laser_amplifier_menu('laser_amplifier'))
put(f'{OUT}/data/{NS}/horizon/menu/laser_tractor_beam.json', laser_tractor_beam_menu('laser_tractor_beam'))
put(f'{OUT}/data/{NS}/horizon/menu/sps_port.json', sps_port_menu('sps_port'))
put(f'{OUT}/data/{NS}/horizon/menu/logistical_sorter.json', sorter_menu('logistical_sorter'))
put(f'{OUT}/data/{NS}/horizon/menu/quantum_entangloporter.json', entangloporter_menu('quantum_entangloporter'))
put(f'{OUT}/data/{NS}/horizon/menu/teleporter.json', teleporter_menu('teleporter'))
put(f'{OUT}/data/{NS}/horizon/menu/portable_teleporter.json', portable_teleporter_menu('portable_teleporter'))
for _b in BINS:
    put(f'{OUT}/data/{NS}/horizon/menu/{_b}.json', bin_menu(_b))
put(f'{OUT}/data/{NS}/horizon/menu/digital_miner.json', miner_menu('digital_miner'))
put(f'{OUT}/data/{NS}/horizon/menu/dictionary.json', dictionary_menu('dictionary'))
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
            # GuiSideConfigurationTab at -26, 6: opens the side configuration window
            {'type': 'sprite', 'texture': tab_config, 'x': -26, 'y': 6, 'w': 26, 'h': 26},
            {'type': 'button', 'x': -26, 'y': 6, 'w': 26, 'h': 26, 'local': {'open': 'side_config'}, 'tooltip': 'Configuration'}]
                   + redstone_widgets(16) + energy_tab_widgets(16, MACHINE_USAGE[m]),
                   'windows': [{'id': 'side_config', 'x': WIN_X, 'y': WIN_Y, 'width': WIN_W, 'height': WIN_H,
                                'texture': window_tex, 'widgets': side_config_widgets(MACHINE_TRANSMISSIONS[m], m, 31)}]}}, m, len(layout['slots']), 16, charge=power_slot(layout)))



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
for gi in ('turbine_blade', 'hohlraum', 'module_geothermal_generator_unit', 'module_solar_recharging_unit'):
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
# Mekanism Additions' machine recipes for the plastic and the glow panels: the painting machine recolours a block with a pigment (the recipe takes "any block of
# that kind but this colour": `neoforge:difference` over a tag of the mod, written out here as the list of the items), the pigment extractor takes a pigment out of a
# plastic block, and the enricher makes slick plastic. (The balloons are not here yet, and the recolour *crafting* recipes need the same ingredient: below.)
def additions_tag_items(name, seen=()):
    """The items of a tag of Additions (`mekanismadditions:plastic_stairs/normal`), nested tags followed."""
    ns, _, path = name.partition(':')
    p = f'{AGEN}/data/{ns}/tags/item/{path}.json'
    if ns != 'mekanismadditions' or name in seen or not os.path.exists(p):
        return []
    out = []
    for v in read_json(p).get('values', []):
        v = v if isinstance(v, str) else v.get('id', '')
        if v.startswith('#'):
            out += additions_tag_items(v[1:], seen + (name,))
        else:
            out.append(v)
    return out

def additions_item_ingredient(v):
    """An item ingredient as the normalised base has it (a name, a `#tag`, or a difference of the two), in the port's shape: a name, a `#tag` or a list of
    names, with the difference of a tag and an item written out."""
    if isinstance(v, str):
        if v.startswith('#'):
            items = additions_tag_items(v[1:])
            return [i.replace('mekanismadditions:', f'{NS}:') for i in items] if v[1:].startswith('mekanismadditions:') else v
        return v.replace('mekanismadditions:', f'{NS}:')
    if isinstance(v, dict) and v.get('neoforge:ingredient_type') == 'neoforge:difference':
        base, gone = v['base'], v['subtracted']
        items = additions_tag_items(base[1:]) if isinstance(base, str) and base.startswith('#') else [base]
        gone = gone if isinstance(gone, str) else ''
        return [i.replace('mekanismadditions:', f'{NS}:') for i in items if i != gone]
    return None

_plastic_have = set(plastic_blocks)
_have_a = set(kept_items) | set(kept_blocks)
nadd = 0
for p in sorted(glob.glob(f'{AGEN}/data/mekanismadditions/recipe/**/*.json', recursive=True)):
    if 'balloon' in p:
        continue
    r = read_json(p)
    kind = r.get('type')
    if kind not in (f'{NS}:painting', f'{NS}:pigment_extracting', f'{NS}:enriching'):
        continue
    out_id = r.get('output', {}).get('id', '')
    item_in = r.get('item_input', r.get('input', {}))
    ing = additions_item_ingredient(item_in.get('ingredient'))
    if ing is None:
        continue
    names = [i for i in (ing if isinstance(ing, list) else [ing]) if not i.startswith('#')]
    if isinstance(ing, list):
        ing = [i for i in ing if i.split(':')[-1] in _plastic_have or i.split(':')[-1] in _have_a]
        if not ing:
            continue
    new = {'type': kind}
    sized_in = {'count': item_in.get('count', 1), 'ingredient': ing}
    target = out_id.split(':')[-1]
    if kind == f'{NS}:painting':
        new['item_input'] = sized_in
        new['chemical_input'] = r['chemical_input']
        new['output'] = {'count': r['output'].get('count', 1), 'id': out_id.replace('mekanismadditions:', f'{NS}:')}
        new['per_tick_usage'] = r.get('per_tick_usage', False)
        if target not in _plastic_have and target not in _have_a:
            continue
    elif kind == f'{NS}:pigment_extracting':
        new['input'] = sized_in
        new['output'] = r['output']
        if not all(n.split(':')[-1] in _plastic_have or n.split(':')[-1] in _have_a for n in names):
            continue
    else:
        new['input'] = sized_in
        new['output'] = {'count': r['output'].get('count', 1), 'id': out_id.replace('mekanismadditions:', f'{NS}:')}
        if target not in _plastic_have or not all(n.split(':')[-1] in _plastic_have for n in names):
            continue
    rel = os.path.relpath(p, f'{AGEN}/data/mekanismadditions/recipe')[:-5].replace(os.sep, '__')
    put(f'{OUT}/data/{NS}/recipe/additions_{rel}.json', new)
    nadd += 1
print(f'additions machine recipes {nadd}')

# The recolour crafting recipes ("surround a dye with any plastic of that kind but this colour"): the same difference, written out in the key or the ingredients.
def additions_crafting_ingredient(v):
    if isinstance(v, list):
        return [additions_crafting_ingredient(x) for x in v]
    if isinstance(v, dict) and v.get('neoforge:ingredient_type') == 'neoforge:difference':
        items = additions_item_ingredient(v)
        return [i for i in items if i.split(':')[-1] in _plastic_have or i.split(':')[-1] in _have_a]
    if isinstance(v, str) and v.startswith('mekanismadditions:'):
        return v.replace('mekanismadditions:', f'{NS}:')
    return v
ncraft = 0
for p in sorted(glob.glob(f'{AGEN}/data/mekanismadditions/recipe/**/*.json', recursive=True)):
    if 'balloon' in p or 'recolor' not in p:
        continue
    r = read_json(p)
    if not str(r.get('type', '')).startswith('minecraft:crafting_'):
        continue
    rid = r.get('result', {}).get('id', '').split(':')[-1]
    if rid not in _plastic_have and rid not in _have_a:
        continue
    new = dict(r)
    if 'key' in new:
        new['key'] = {k: additions_crafting_ingredient(v) for k, v in new['key'].items()}
    if 'ingredients' in new:
        new['ingredients'] = [additions_crafting_ingredient(v) for v in new['ingredients']]
    new['result'] = {**r['result'], 'id': r['result']['id'].replace('mekanismadditions:', f'{NS}:')}
    if any(x == [] for x in list(new.get('key', {}).values()) + list(new.get('ingredients', []))):
        continue
    rel = os.path.relpath(p, f'{AGEN}/data/mekanismadditions/recipe')[:-5].replace(os.sep, '__')
    put(f'{OUT}/data/{NS}/recipe/additions_{rel}.json', json.loads(json.dumps(new).replace('mekanismadditions:', f'{NS}:')))
    ncraft += 1
print(f'additions recolour crafting recipes {ncraft}')

for _g in ('geothermal_generator_unit', 'solar_recharging_unit'):
    _r = json.loads(open(f'{GGEN}/data/{GNS_SRC}/recipe/module_{_g}.json').read().replace(f'{GSRC}:', f'{NS}:'))
    put(f'{OUT}/data/{NS}/recipe/generator_module_{_g}.json', _r)
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
def robit_menu(skin_names):
    """The Robit's screens (`GuiRobitMain`, `GuiRobitInventory`, `GuiRobitSmelting`, `GuiRobitCrafting`, `GuiRobitRepair`; FORK-10 5b, the kind's `menu`): the main page (176 by 166) with the
    inner screen at 27, 16 (122 by 56: the greeting, the energy, whether it follows, whether it picks up and its owner, the entity's strings 0 to 4), the power bar at 27, 74, the energy slot
    (153, 17), the buttons go home (6, 16), rename (6, 35), skin (6, 54), pick-up (152, 35) and follow (152, 54), and the holder on the right (176, 6, 106 high) with the tabs
    main, crafting, inventory, smelting and repair at 179, 10 and 20 apart. Each other page is a window over the top 84 of it: the inventory (the 27 slots at 8, 18), the furnace (the input at 51,
    35, the output at 116, 35 and the progress bar at 78, 38), the crafting (the 3 by 3 grid at 30, 17 and the result at 124, 35, the arrow at 90, 35) and the repair (the two inputs at 27, 47
    and 76, 47, the result at 134, 47 and the name box at 60, 21), with the windows of the rename box and of the skins. The entity's menu data: 0 follows, 1 picks up, 2 its energy in
    Forge Energy (at most 40,000), 3 the furnace's progress in percent; its strings: 0 the greeting, 1 the energy, 2 following, 3 pickup, 4 the owner, 5 its name. The slots: 0 to 26 the storage,
    27 the energy slot, 28 and 29 the furnace, 30 to 38 the crafting grid and 39 its result, 40 and 41 the repair's inputs and 42 its result (virtual ones 30 to 42). **Not in the port:**
    the security tab, and the anvil's texture (it is the vanilla one)."""
    from PIL import Image
    width, height = 176, 166
    panel, slot, inv_y = inventory_panel(width, height)
    panel.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), 122, 56, 3), (27, 16))
    panel.alpha_composite(nine_slice(gui_sprite('bar/base'), 122, 6, 1), (26, 73))
    panel.alpha_composite(gui_sprite('slot/power'), (152, 16))
    texture = save_gui(panel, 'robit')
    def icon(name):
        return save_gui(Image.open(find(f'gui/button/{name}.png', ASSET_DIRS)).convert('RGBA'), f'robit_{name}')
    holder = save_gui(nine_slice(gui_sprite('holder_right'), 26, 106, 4), 'robit_tab_holder')
    def window_panel(frames, name, extra=None):
        p = nine_slice(gui_sprite('base'), width, 84, 4)
        for (x, y, kind) in frames:
            p.alpha_composite(gui_sprite(f'slot/{kind}'), (x - 1, y - 1))
        if extra:
            extra(p)
        return save_gui(p, f'robit_{name}')
    def slots_of(first, positions):
        return [{'slot': first + i, 'x': x, 'y': y} for i, (x, y) in enumerate(positions)]
    w_text = lambda t, x=8: {'type': 'text', 'x': x, 'y': 6, 'text': t, 'color': '#404040'}
    inv_pos = [(8 + 18 * c, 18 + 18 * r) for r in range(3) for c in range(9)]
    win_inventory = {'id': 'inventory', 'x': 0, 'y': 0, 'width': width, 'height': 84, 'texture': window_panel([(x, y, 'normal') for x, y in inv_pos], 'inventory'),
                     'widgets': [w_text('Robit Inventory')], 'slots': slots_of(0, inv_pos)}
    def furnace_extra(p):
        p.alpha_composite(gui_sprite('progress/bar/empty'), (78, 38))
    win_furnace = {'id': 'furnace', 'x': 0, 'y': 0, 'width': width, 'height': 84,
                   'texture': window_panel([(51, 35, 'input'), (116, 35, 'output')], 'furnace', furnace_extra),
                   'widgets': [w_text('Robit Smelting'),
                               {'type': 'sprite', 'texture': save_gui(gui_sprite('progress/bar/full'), 'robit_progress_full'), 'x': 78, 'y': 38, 'w': 25, 'h': 9,
                                'value': {'data': 3}, 'max': {'const': 100}, 'direction': 'right', 'tooltip': 'Progress: {3}%'}],
                   'slots': [{'slot': 28, 'x': 51, 'y': 35}, {'slot': 29, 'x': 116, 'y': 35}]}
    grid = [(30 + 18 * c, 17 + 18 * r) for r in range(3) for c in range(3)]
    def crafting_extra(p):
        p.alpha_composite(gui_sprite('progress/right/empty'), (90, 35))
    win_crafting = {'id': 'crafting', 'x': 0, 'y': 0, 'width': width, 'height': 84,
                    'texture': window_panel([(x, y, 'normal') for x, y in grid] + [(124, 35, 'output')], 'crafting', crafting_extra),
                    'widgets': [w_text('Robit Crafting')], 'slots': slots_of(30, grid) + [{'slot': 39, 'x': 124, 'y': 35}]}
    win_repair = {'id': 'repair', 'x': 0, 'y': 0, 'width': width, 'height': 84,
                  'texture': window_panel([(27, 47, 'input'), (76, 47, 'input'), (134, 47, 'output')], 'repair'),
                  'widgets': [w_text('Robit Repair'), {'type': 'text', 'x': 8, 'y': 24, 'text': 'Name:', 'color': '#404040', 'scale': 90},
                              {'type': 'textfield', 'x': 60, 'y': 21, 'w': 103, 'h': 12, 'id': 11, 'validator': 'any', 'max_length': 50},
                              {'type': 'text', 'x': 8, 'y': 68, 'text': 'Only joining two damaged tools works here.', 'color': '#808080', 'scale': 70}],
                  'slots': [{'slot': 40, 'x': 27, 'y': 47}, {'slot': 41, 'x': 76, 'y': 47}, {'slot': 42, 'x': 134, 'y': 47}]}
    rp = nine_slice(gui_sprite('base'), width, 84, 4)
    rp.alpha_composite(nine_slice(gui_sprite('inner_screen').resize((16, 16), 0), 122, 14, 2), (27, 30))
    win_rename = {'id': 'rename', 'x': 0, 'y': 0, 'width': width, 'height': 84, 'texture': save_gui(rp, 'robit_rename'),
                  'widgets': [w_text('Rename the Robit'), {'type': 'button', 'x': 6, 'y': 6, 'w': 12, 'h': 12, 'local': 'close', 'tooltip': 'Back'},
                              {'type': 'textfield', 'x': 29, 'y': 32, 'w': 118, 'h': 10, 'id': 10, 'validator': 'any', 'max_length': 32, 'text_data': 5},
                              {'type': 'text', 'x': 29, 'y': 50, 'text': 'Type a name and confirm', 'color': '#808080', 'scale': 80}]}
    sp = nine_slice(gui_sprite('base'), width, 84, 4)
    skin_w = [w_text('Skin')]
    for i, name in enumerate(skin_names):
        col, row = i % 3, i // 3
        x, y = 8 + 56 * col, 18 + 12 * row
        sp.alpha_composite(nine_slice(gui_sprite('button'), 54, 11, 2), (x, y))
        skin_w.append({'type': 'text', 'x': x + 3, 'y': y + 2, 'text': name[:12], 'color': '#404040', 'scale': 65})
        skin_w.append({'type': 'button', 'x': x, 'y': y, 'w': 54, 'h': 11, 'id': 20 + i, 'tooltip': name})
    skin_w.append({'type': 'button', 'x': 6, 'y': 6, 'w': 12, 'h': 12, 'local': 'close', 'tooltip': 'Back'})
    win_skin = {'id': 'skin', 'x': 0, 'y': 0, 'width': width, 'height': 84, 'texture': save_gui(sp, 'robit_skin'), 'widgets': skin_w}
    widgets = [
        {'type': 'text', 'x': (width - text_width('Robit')) // 2, 'y': 6, 'text': 'Robit', 'color': '#404040'},
        {'type': 'text', 'x': 8, 'y': height - 92, 'text': 'Inventory', 'color': '#404040'},
        {'type': 'sprite', 'texture': holder, 'x': 176, 'y': 6, 'w': 26, 'h': 106}]
    for i in range(5):
        widgets.append({'type': 'text', 'x': 29, 'y': 20 + 9 * i if i == 0 else 20 + 9 * (i + 1), 'text': '', 'text_data': i, 'color': '#3CFE9A', 'scale': 80})
    widgets.append({'type': 'sprite', 'texture': power_h_fill, 'x': 27, 'y': 74, 'w': 120, 'h': 4, 'value': {'data': 2}, 'max': {'const': 40000}, 'direction': 'right',
                    'tooltip': 'Energy: {2} FE'})
    for name, x, y, bid, tip in (('home', 6, 16, 3, 'Go home'), ('rename', 6, 35, None, 'Rename'), ('skin', 6, 54, None, 'Skin'), ('drop', 152, 35, 2, 'Pick up items'), ('follow', 152, 54, 1, 'Follow')):
        widgets.append({'type': 'sprite', 'texture': icon(name), 'x': x, 'y': y, 'w': 18, 'h': 18})
        b = {'type': 'button', 'x': x, 'y': y, 'w': 18, 'h': 18, 'tooltip': tip}
        if bid is not None:
            b['id'] = bid
        else:
            b['local'] = {'open': 'rename' if name == 'rename' else 'skin'}
        widgets.append(b)
    for k, (name, window, tip) in enumerate((('main', None, 'Robit'), ('crafting', 'crafting', 'Crafting'), ('inventory', 'inventory', 'Inventory'), ('smelting', 'furnace', 'Smelting'), ('repair', 'repair', 'Repair'))):
        widgets.append({'type': 'sprite', 'texture': icon(name), 'x': 179, 'y': 10 + 20 * k, 'w': 18, 'h': 18})
        widgets.append({'type': 'button', 'x': 179, 'y': 10 + 20 * k, 'w': 18, 'h': 18, 'local': ({'open': window} if window else 'close'), 'tooltip': tip})
    slots = [None] * 27 + [[153, 17]] + [None] * 15
    return {'slots': 43, 'data_slots': 4, 'results': [39, 42],
            'screen': {'texture': texture, 'own_labels': True, 'slots': slots, 'width': width, 'height': height, 'inventory_x': 8, 'inventory_y': inv_y,
                       'widgets': widgets, 'windows': [win_inventory, win_furnace, win_crafting, win_repair, win_rename, win_skin]}}

# The Robit (FORK-10): the entity kind with the skins of `robit.rs::ALL_SKINS` (the first texture of each; the original's variants are chosen by the entity),
# each skin drawn as an item model: the Robit's own model (`models/item/robit.json`, or the allay costume's) with the skin's texture set, which the
# original's `mekanism:robit` loader does at run time.
_robit_src = open(os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'mod', 'src', 'robit.rs')).read()
_robit_skins = re.findall(r'id: "mekanism:(\w+)",\s*name: "([^"]*)",\s*textures: &\["mekanism:(\w+)"[^\]]*\],\s*required_advancement: [^\n]*\n\s*custom_model: (None|Some\("mekanism:item/(\w+)"\))', _robit_src)
_robit_kind = {'width': 0.6, 'height': 0.65, 'max_health': 1.0, 'speed': 0.3, 'container_slots': 30, 'menu': f'{NS}:robit', 'skins': []}
for _sid, _sname, _tex, _cm, _model in _robit_skins:
    _base = read_json(find(f'models/item/{_model or "robit"}.json', ASSET_DIRS))
    _base.pop('loader', None)
    _base['textures'] = {'robit': f'{NS}:entity/robit/{_tex}', 'particle': f'{NS}:entity/robit/{_tex}'}
    put(f'{OUT}/assets/{NS}/models/item/robit_skin_{_sid}.json', _base)
    put(f'{OUT}/assets/{NS}/items/robit_skin_{_sid}.json', {'model': {'type': 'minecraft:model', 'model': f'{NS}:item/robit_skin_{_sid}'}})
    _robit_kind['skins'].append({'id': f'{NS}:{_sid}', 'texture': f'{NS}:entity/robit/{_tex}', 'model': f'{NS}:robit_skin_{_sid}'})
put(f'{OUT}/data/{NS}/horizon/entity_kind/robit.json', _robit_kind)
put(f'{OUT}/data/{NS}/horizon/menu/robit.json', robit_menu([n for _i, n, _t, _c, _m in _robit_skins]))
# the Robit item itself shows the default skin's model (the original's loader sets the texture, which the port's models carry)
put(f'{OUT}/assets/{NS}/models/item/robit.json', read_json(f'{OUT}/assets/{NS}/models/item/robit_skin_robit.json'))
for _png in glob.glob(f'{GEN}/assets/{NS}/textures/entity/robit/*.png'):
    put(f'{OUT}/assets/{NS}/textures/entity/robit/{os.path.basename(_png)}', src=_png)
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
    # Mekanism also makes vanilla things (paper from sawdust, rails, charcoal, yellow dye): those count too, with the same check of the ingredients.
    if (rid.startswith(NS + ':') and rid.split(':')[1] in have) or rid.startswith('minecraft:'):
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
# the sound subtitles (every sound event ships), the colour names, and the screen titles of the machines that are kept
out.update({k: v for k, v in lang.items() if k.startswith(('sound_event.mekanism.', 'color.mekanism.'))})
out.update({k: v for k, v in lang.items() if k.startswith('container.mekanism.') and k[len('container.mekanism.'):] in kept_blocks})
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
for _n, _v in MODULE_NAMES.items():
    out[f'item.{NS}.module_{_n}'] = _v
for _g in ('geothermal_generator_unit', 'solar_recharging_unit'):
    out[f'item.{NS}.module_{_g}'] = glang[f'module.{GSRC}.{_g}']
for k, u in enumerate(UPGRADES):
    out[f'item.{NS}.upgrade_{u}'] = f'{UPGRADE_NAMES[k]} Upgrade'
# Item and block names the lang file keeps under other keys: block items use block.*
put(f'{OUT}/assets/{NS}/lang/en_us.json', out)

# the sounds: the events (`sounds.json`) and their files, as the original has them (a mod's own sounds are played by id from the mod's code)
_sounds_json = f'{GEN}/assets/{NS}/sounds.json'
if os.path.exists(_sounds_json):
    put(f'{OUT}/assets/{NS}/sounds.json', src=_sounds_json)
_sounds_dir = f'{RES}/assets/{NS}/sounds'
nsounds = 0
for _dp, _dn, _fs in os.walk(_sounds_dir):
    for _f in _fs:
        if _f.endswith('.ogg'):
            _rel = os.path.relpath(os.path.join(_dp, _f), _sounds_dir)
            put(f'{OUT}/assets/{NS}/sounds/{_rel}', src=os.path.join(_dp, _f))
            nsounds += 1
print(f'sounds {nsounds}')

print(f'items {len(kept_items)}, blocks {len(kept_blocks)}, textures {len(copied_textures)}, '
      f'models {len(copied_models)}, recipes {nrec}')
print('warnings:', len(warnings))
for w in warnings[:40]:
    print('  ', w)

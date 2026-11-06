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
ASSET_DIRS = [f'{GEN}/assets/{NS}', f'{RES}/assets/{NS}']
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
                    'chemical_oxidizer', 'chemical_dissolution_chamber', 'chemical_crystallizer']
CONTINUOUS_MACHINES = ['chemical_infuser', 'chemical_washer', 'electrolytic_separator']
TIERS = ['basic', 'advanced', 'elite', 'ultimate']
FLUID_TANKS = [f'{t}_fluid_tank' for t in TIERS + ['creative']]
CHEM_TANKS = [f'{t}_chemical_tank' for t in TIERS + ['creative']]
MACHINES = ['crusher', 'enrichment_chamber', 'energized_smelter', 'precision_sawmill', 'combiner'] + PROCESS_MACHINES \
    + CONTINUOUS_MACHINES + FLUID_TANKS + CHEM_TANKS
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
                 *CONTINUOUS_MACHINES, *FLUID_TANKS, *CHEM_TANKS}
for m in OPEN_MACHINES:
    BLOCKS[m] = ('minecraft:beacon', BLOCKS[m][1], 'machine')

TRANSMITTERS = [('universal_cable', 'energy'), ('pressurized_tube', 'chemical'), ('mechanical_pipe', 'fluid')]
TRANSMITTER_BLOCKS = [f'{tier}_{kind}' for kind, _ in TRANSMITTERS for tier in ('basic', 'advanced', 'elite', 'ultimate')]
CUSTOM_BLOCKS = [*TRANSMITTER_BLOCKS, 'basic_energy_cube', 'creative_energy_cube']
# ---------------------------------------------------------------- items: every plain one
items = {}
for p in sorted(glob.glob(f'{GEN}/assets/{NS}/items/*.json')):
    n = os.path.basename(p)[:-5]
    if n in BLOCKS or n in CUSTOM_BLOCKS or n.endswith('_bucket'):
        continue
    d = read_json(p)
    model = d.get('model', {})
    if n == 'configurator':
        # Its model picks by a component of Mekanism's own (the tool's mode); the port has the one mode, configuring.
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
            block['state_properties']['facing'] = {'type': 'enum', 'values': ['north', 'east', 'south', 'west']}
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
                'conditions': [{'condition': 'minecraft:survives_explosion'}]}]}
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
            'conditions': [{'condition': 'minecraft:survives_explosion'}]}]}

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
    core = Image.open(find('textures/block/energy_core.png', ASSET_DIRS)).convert('RGBA')
    px = core.load()
    for x in range(core.width):
        for y in range(core.height):
            r, g, b, a = px[x, y]
            px[x, y] = (r * best[0] // 255, g * best[1] // 255, b * best[2] // 255, a)
    path = f'{OUT}/assets/{NS}/textures/block/energy_core_{tier}.png'
    os.makedirs(os.path.dirname(path), exist_ok=True)
    core.save(path)
    return f'{NS}:block/energy_core_{tier}'

for tier in ('basic', 'creative'):
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
FLUID_BLOCKS = ['chemical_washer', 'electrolytic_separator', *FLUID_TANKS]
put(f'{OUT}/data/{NS}/tags/block/energy_connectable.json', {'values': [
    *[f'{NS}:{n}' for n in TRANSMITTER_BLOCKS if n.endswith('_universal_cable')],
    f'{NS}:basic_energy_cube', f'{NS}:creative_energy_cube', *[f'{NS}:{m}' for m in ELECTRIC]]})
put(f'{OUT}/data/{NS}/tags/block/chemical_connectable.json', {'values': [
    *[f'{NS}:{n}' for n in TRANSMITTER_BLOCKS if n.endswith('_pressurized_tube')], *[f'{NS}:{m}' for m in CHEMICAL_BLOCKS]]})
put(f'{OUT}/data/{NS}/tags/block/fluid_connectable.json', {'values': [
    *[f'{NS}:{n}' for n in TRANSMITTER_BLOCKS if n.endswith('_mechanical_pipe')], *[f'{NS}:{m}' for m in FLUID_BLOCKS]]})

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

MACHINE_NAMES = {'chemical_infuser': 'Chemical Infuser', 'chemical_washer': 'Chemical Washer',
                 'electrolytic_separator': 'Electrolytic Separator',
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
MACHINE_TRANSMISSIONS = {'crusher': [0, 1], 'enrichment_chamber': [0, 1], 'energized_smelter': [0, 1],
                         'precision_sawmill': [0, 1], 'combiner': [0, 1], **{m: [0, 3, 1] for m in PROCESS_MACHINES},
                         'chemical_infuser': [3, 0, 1], 'chemical_washer': [3, 2, 0, 1], 'electrolytic_separator': [2, 3, 0, 1],
                         **{m: [3, 0] for m in CHEM_TANKS}}
# the transmissions a machine cannot eject (setCanEject(false)): the infuser's tank
NO_EJECT = {(m, 3) for m in ('metallurgic_infuser', 'purification_chamber', 'chemical_injection_chamber', 'osmium_compressor',
                             'chemical_crystallizer')} | {('chemical_washer', 2), ('electrolytic_separator', 2)} \
    | {(m, 0) for m in CHEM_TANKS}
TAB_SPRITES = {n: tab_sprite(f'tab_{info[0].lower()}', info[1], info[2]) for n, info in TRANSMISSION_INFO.items()}

def side_config_widgets(transmissions, machine=None):
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
            group.append({'type': 'neighbor', 'x': x + 3, 'y': y + 3, 'side': side})
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
                     'metallurgic_infusing', 'chemical_conversion', 'chemical_infusing', 'washing', 'separating']
# the fluids the screens draw (the same table as the chemicals): their colour
FLUIDS = {'minecraft:water': '#3F76E4', 'minecraft:lava': '#FF6A00', 'mekanism:brine': '#FEEF9C', 'mekanism:lithium': '#EBA400'}
CHEM_ITEM_INPUT_TYPES = {'purifying', 'injecting', 'compressing', 'dissolution', 'crystallizing', 'metallurgic_infusing'}

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
        if kind == 'chemical_infusing':
            for side in ('left_input', 'right_input'):
                if isinstance(r.get(side, {}).get('ingredient'), str):
                    add(r[side]['ingredient'])
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
        out.append({'texture': chemical_icon_sprite(icon), 'tint': tint, 'label': cid.split(':')[1].replace('_', ' ').title()})
    return out

GAUGE_TYPES = gauge_types()
with open(f'{OUT}/src/chem_table.rs', 'w') as f:
    f.write('// Written by tools/convert.py: the chemicals the machines\' screens draw, in the order of their numbers (from 1).\n')
    f.write(f'pub const CHEMICALS: [&str; {len(CHEMICALS)}] = [\n')
    for cid in CHEMICALS:
        f.write(f'    "{cid}",\n')
    f.write('];\n')

PROGRESS_TYPES = {'bar': (25, 9, 'right'), 'right': (32, 8, 'right'), 'large_right': (48, 8, 'right'),
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
        'ticks': 100, 'slots': [[26, 36], [152, 55]], 'results': [1],
        'frames': [('slot/input', 26, 36), ('slot/extra', 152, 55), ('slot/power', 152, 14)],
        'overlays': [('slot/overlay/plus', 152, 55)], 'power': ('horizontal', 115, 75), 'progress': [('large_right', 64, 40)],
        'gauges': [('standard', 131, 13, 16, 58, 'out', 10000)]},
    # GuiChemicalDissolutionChamber
    'chemical_dissolution_chamber': {
        'ticks': 100, 'slots': [[28, 36], [152, 55], [8, 65]], 'results': [1],
        'frames': [('slot/input', 28, 36), ('slot/extra', 152, 55), ('slot/extra', 8, 65), ('slot/power', 152, 14)],
        'overlays': [('slot/overlay/minus', 8, 65), ('slot/overlay/plus', 152, 55)],
        'power': ('horizontal', 115, 75), 'progress': [('large_right', 64, 40)],
        'gauges': [('standard', 7, 4, 16, 58, 'in', 10000), ('standard', 131, 13, 16, 58, 'out', 10000)]},
    # GuiChemicalCrystallizer
    'chemical_crystallizer': {
        'ticks': 200, 'slots': [[8, 65], [129, 57]], 'results': [1],
        'frames': [('slot/extra', 8, 65), ('slot/output', 129, 57), ('slot/power', 152, 5)],
        'overlays': [('slot/overlay/plus', 8, 65)], 'power': ('vertical', 157, 23), 'progress': [('large_right', 53, 61)],
        'gauges': [('standard', 7, 4, 16, 58, 'in', 10000)], 'screen': (31, 13, 115, 42)},
    # GuiChemicalInfuser: two chemical item slots in and one out, three gauges, two arrows driven by whether it works
    'chemical_infuser': {
        'continuous': True, 'slots': [[6, 56], [154, 56], [80, 65]], 'results': [2],
        'frames': [('slot/input', 6, 56), ('slot/input', 154, 56), ('slot/output', 80, 65), ('slot/power', 154, 14)],
        'overlays': [('slot/overlay/minus', 6, 56), ('slot/overlay/minus', 154, 56), ('slot/overlay/plus', 80, 65)],
        'power': ('horizontal', 115, 75), 'progress': [('small_right', 47, 39), ('small_left', 101, 39)],
        'gauges': [('standard', 25, 13, 16, 58, 't0', 10000), ('standard', 133, 13, 16, 58, 't1', 10000),
                   ('standard', 79, 4, 16, 58, 't2', 10000)]},
    # GuiChemicalWasher: the fluid slots in a holder at the left
    'chemical_washer': {
        'continuous': True, 'slots': [[-21, 71], [152, 56], [-21, 102]], 'results': [1, 2],
        'frames': [('slot/input', -21, 71), ('slot/extra', 152, 56), ('slot/output', -21, 102), ('slot/power', 152, 14)],
        'overlays': [('slot/overlay/minus', 152, 56)], 'holder': (-26, 66, 57),
        'power': ('horizontal', 115, 75), 'progress': [('large_right', 64, 39)],
        'extra_sprites': [('arrow/down', -17, 90)],
        'gauges': [('standard', 7, 13, 16, 58, 't0', 10000), ('standard', 28, 13, 16, 58, 't1', 10000),
                   ('standard', 131, 13, 16, 58, 't2', 10000)]},
    # GuiElectrolyticSeparator: a fluid gauge, two small gas gauges and the dump buttons of both
    'electrolytic_separator': {
        'continuous': True, 'slots': [[26, 35], [59, 52], [101, 52]], 'results': [1, 2],
        'frames': [('slot/input', 26, 35), ('slot/output', 59, 52), ('slot/output', 101, 52), ('slot/power', 143, 35)],
        'overlays': [], 'power': ('vertical', 164, 15), 'progress': [('bidirectional', 80, 30)],
        'gauges': [('standard', 5, 10, 16, 58, 't0', 24000), ('small', 58, 18, 16, 28, 't1', 2400),
                   ('small', 100, 18, 16, 28, 't2', 2400)],
        'modes': [(7, 72, 5000, 22, 'left'), (159, 72, 5001, 23, 'right')]},
}
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
    panel = nine_slice(gui_sprite('base'), 176, 166, 4)
    slot = gui_sprite('slot/normal')
    for row in range(3):
        for col in range(9):
            panel.paste(slot, (7 + 18 * col, 83 + 18 * row))
    for col in range(9):
        panel.paste(slot, (7 + 18 * col, 141))
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
MODE_TEXT = ['Idle', 'Dumping', 'Dumping excess']

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
    for ptype, qx, qy in L['progress']:
        pw, ph, direction = PROGRESS_TYPES[ptype]
        # a machine that works every tick shows its arrows full while it works (data slot 0 is 1 or 0)
        widgets.append({'type': 'sprite', 'texture': copy_gui(f'progress/{ptype}/full', f'progress_{ptype}_full'), 'x': qx, 'y': qy,
                        'w': pw, 'h': ph, 'value': {'data': 0}, 'max': {'const': 1 if continuous else L['ticks']},
                        'direction': direction, 'tooltip': 'Working' if continuous else 'Progress: {percent}%'})
    for gk, gx, gy, gw, gh, tank, cap in L['gauges']:
        amount, which = TANK_DATA[tank]
        widgets.append({'type': 'gauge', 'x': gx + 1, 'y': gy + 1, 'w': gw, 'h': gh, 'value': {'data': amount},
                        'max': {'const': cap}, 'kind': {'data': which}, 'types': GAUGE_TYPES,
                        'tooltip': '{label}: {value} mB', 'empty': 'Empty'})
        if gk in GAUGE_SIZES:
            widgets.append({'type': 'sprite', 'texture': copy_gui(f'gauge/overlay/{gk}', f'gauge_overlay_{gk}'),
                            'x': gx + 1, 'y': gy + 1, 'w': gw, 'h': gh})
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
    title_w = text_width(title)
    widgets += [
        {'type': 'text', 'x': (176 - title_w) // 2, 'y': 6, 'text': title, 'color': '#404040'},
        {'type': 'text', 'x': 8, 'y': 72, 'text': 'Inventory', 'color': '#404040'},
        {'type': 'text', 'x': 92, 'y': 66, 'text': 'Status {5}', 'color': '#808080'},
        {'type': 'text', 'x': 128, 'y': 72, 'text': 'v0.1.32', 'color': '#808080'},
        # GuiSideConfigurationTab at -26, 6: opens the side configuration window
        {'type': 'sprite', 'texture': tab_config, 'x': -26, 'y': 6, 'w': 26, 'h': 26},
        {'type': 'button', 'x': -26, 'y': 6, 'w': 26, 'h': 26, 'local': {'open': 'side_config'}, 'tooltip': 'Configuration'}]
    return {'slots': len(L['slots']), 'data_slots': 24 if continuous else 20, 'results': L['results'],
            'screen': {'texture': process_panel(m, L), 'own_labels': True, 'slots': L['slots'], 'widgets': widgets,
                       'windows': [{'id': 'side_config', 'x': WIN_X, 'y': WIN_Y, 'width': WIN_W, 'height': WIN_H,
                                    'texture': window_tex, 'widgets': side_config_widgets(MACHINE_TRANSMISSIONS[m], m)}]}}

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
        for k, (name, label) in enumerate([('idle', 'Idle'), ('dump', 'Dumping'), ('dump_excess', 'Dumping excess')]):
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
                {'type': 'text', 'x': 128, 'y': 72, 'text': 'v0.1.32', 'color': '#808080'}]
    return {'slots': 2, 'data_slots': data_slots, 'results': results, 'armor': ARMOR_LEFT,
            'screen': {'texture': texture, 'own_labels': True, 'slots': slots, 'widgets': widgets, 'windows': windows}}


for m in MACHINES:
    if m in PROCESS_LAYOUTS:
        put(f'{OUT}/data/{NS}/horizon/menu/{m}.json', process_menu(m))
        continue
    if m in FLUID_TANKS or m in CHEM_TANKS:
        put(f'{OUT}/data/{NS}/horizon/menu/{m}.json', tank_menu(m))
        continue
    layout = LAYOUTS.get(m, LAYOUTS['electric'])
    title = MACHINE_NAMES.get(m, m.replace('_', ' ').title())
    px, py = layout['progress']
    put(f'{OUT}/data/{NS}/horizon/menu/{m}.json', {
        'slots': len(layout['slots']), 'data_slots': 16, 'results': layout['results'],
        'screen': {'texture': machine_panel(m, layout), 'own_labels': True, 'slots': layout['slots'],
                   'widgets': [
            # the vertical power bar: its contents are 4 by 52 inside the frame
            {'type': 'sprite', 'texture': power_fill, 'x': 165, 'y': layout['bar_y'] + 1, 'w': 4, 'h': 52,
             'value': {'pair': [1, 2]}, 'max': {'pair': [3, 4]}, 'direction': 'up', 'tooltip': 'Energy: {value} / {max}'},
            # GuiProgress, ProgressType.BAR
            {'type': 'sprite', 'texture': progress_full, 'x': px, 'y': py, 'w': 25, 'h': 9, 'value': {'data': 0},
             'max': {'const': 200}, 'direction': 'right', 'tooltip': 'Progress: {percent}%'},
            # the title, centred at the top, and the inventory's, at 8, 72 (vanilla's own places)
            {'type': 'text', 'x': (176 - text_width(title)) // 2, 'y': 6, 'text': title, 'color': '#404040'},
            {'type': 'text', 'x': 8, 'y': 72, 'text': 'Inventory', 'color': '#404040'},
            {'type': 'text', 'x': 92, 'y': 54, 'text': 'Status {5}', 'color': '#808080'},
            {'type': 'text', 'x': 128, 'y': 72, 'text': 'v0.1.32', 'color': '#808080'},
            # GuiSideConfigurationTab at -26, 6: opens the side configuration window
            {'type': 'sprite', 'texture': tab_config, 'x': -26, 'y': 6, 'w': 26, 'h': 26},
            {'type': 'button', 'x': -26, 'y': 6, 'w': 26, 'h': 26, 'local': {'open': 'side_config'}, 'tooltip': 'Configuration'}],
                   'windows': [{'id': 'side_config', 'x': WIN_X, 'y': WIN_Y, 'width': WIN_W, 'height': WIN_H,
                                'texture': window_tex, 'widgets': side_config_widgets(MACHINE_TRANSMISSIONS[m], m)}]}})



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
CUBES = {'basic_energy_cube': ('Basic Energy Cube', 4000), 'creative_energy_cube': ('Creative Energy Cube', 1000000000)}
for cube_name, (cube_title, rate) in CUBES.items():
    put(f'{OUT}/data/{NS}/horizon/menu/{cube_name}.json', {
        'slots': 2, 'data_slots': 16, 'results': [],
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
                                'texture': window_tex, 'widgets': side_config_widgets([1])}]}})
# Status: 0 idle, 1 running, 2 no recipe for the input, 3 no energy, 4 not enough input or no room, 5 no recipes loaded

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
                 f'{NS}:dissolution', f'{NS}:crystallizing', f'{NS}:chemical_infusing', f'{NS}:washing', f'{NS}:separating'}
for p in sorted(glob.glob(f'{GEN}/data/{NS}/recipe/**/*.json', recursive=True)):
    rel = os.path.relpath(p, f'{GEN}/data/{NS}/recipe')
    r = read_json(p)
    if r.get('type') not in MACHINE_TYPES:
        continue
    outs = [r.get(k, {}).get('id', '') for k in ('output', 'main_output', 'secondary_output')]
    if 'amount' in r.get('output', {}):
        outs = []  # it makes a chemical, not an item
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
for p in sorted(glob.glob(f'{GEN}/data/{NS}/recipe/*.json')):
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
            put(f'{OUT}/data/{NS}/recipe/{os.path.basename(p)}', r)
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
pick = [f'{NS}:{n}' for n in sorted(BLOCKS)] + [f'{NS}:basic_energy_cube', f'{NS}:creative_energy_cube']
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
# Item and block names the lang file keeps under other keys: block items use block.*
put(f'{OUT}/assets/{NS}/lang/en_us.json', out)

print(f'items {len(kept_items)}, blocks {len(kept_blocks)}, textures {len(copied_textures)}, '
      f'models {len(copied_models)}, recipes {nrec}')
print('warnings:', len(warnings))
for w in warnings[:40]:
    print('  ', w)

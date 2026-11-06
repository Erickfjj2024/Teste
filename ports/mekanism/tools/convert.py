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
MACHINES = ['crusher', 'enrichment_chamber', 'energized_smelter', 'precision_sawmill', 'combiner']
BLOCKS = {}  # name -> (template, properties, kind)
for o in ORES:
    BLOCKS[f'{o}_ore'] = ('minecraft:iron_ore', {'hardness': 3.0, 'explosion_resistance': 3.0, 'requires_tool': True}, 'ore')
    BLOCKS[f'deepslate_{o}_ore'] = ('minecraft:deepslate_iron_ore', {'hardness': 4.5, 'explosion_resistance': 3.0, 'requires_tool': True}, 'ore')
for s in STORAGE:
    BLOCKS[s] = ('minecraft:iron_block', {'hardness': 5.0, 'explosion_resistance': 6.0, 'requires_tool': True}, 'storage')
BLOCKS['steel_casing'] = ('minecraft:iron_block', {'hardness': 3.5, 'explosion_resistance': 9.0, 'requires_tool': True}, 'storage')
for m in MACHINES:
    BLOCKS[m] = ('minecraft:furnace', {'hardness': 3.5, 'explosion_resistance': 16.0, 'requires_tool': True}, 'machine')

CUSTOM_BLOCKS = ['basic_universal_cable', 'basic_energy_cube', 'creative_energy_cube']
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

kept_items = []
for n, model in items.items():
    if not copy_model(model):
        continue
    kept_items.append(n)
    put(f'{OUT}/data/{NS}/horizon/item/{n}.json', {'template': 'minecraft:stick'})
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
    if not all(copy_model(m) for m in models_of_blockstate(bs)):
        continue
    kept_blocks.append(n)
    block = {'template': template, 'properties': props, 'item': True}
    put(f'{OUT}/data/{NS}/horizon/block/{n}.json', block)
    put(f'{OUT}/assets/{NS}/blockstates/{n}.json', bs)
    ip = find(f'items/{n}.json', ASSET_DIRS)
    if ip:
        d = read_json(ip)
        if d.get('model', {}).get('type') == 'minecraft:model':
            copy_model(d['model']['model'])
            put(f'{OUT}/assets/{NS}/items/{n}.json', d)
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

# cable: a core and an arm, on every side a connection says
cable_tex = f'{NS}:block/models/multipart/basic_universal_cable'
copy_texture(cable_tex)
copy_texture(cable_tex + '_vertical')
put(f'{OUT}/assets/{NS}/models/block/basic_universal_cable_core.json', {
    'textures': {'t': cable_tex, 'particle': cable_tex},
    'elements': [{'from': [5, 5, 5], 'to': [11, 11, 11], 'faces': {d: {'texture': '#t'} for d in ('down', 'up', 'north', 'south', 'west', 'east')}}]})
put(f'{OUT}/assets/{NS}/models/block/basic_universal_cable_arm.json', {
    'textures': {'t': cable_tex + '_vertical', 'particle': cable_tex},
    'elements': [{'from': [5, 5, 0], 'to': [11, 11, 5], 'faces': {d: {'texture': '#t'} for d in ('down', 'up', 'north', 'west', 'east')}}]})
arm = f'{NS}:block/basic_universal_cable_arm'
put(f'{OUT}/assets/{NS}/blockstates/basic_universal_cable.json', {'multipart': [
    {'apply': {'model': f'{NS}:block/basic_universal_cable_core'}},
    {'when': {'north': 'true'}, 'apply': {'model': arm}},
    {'when': {'east': 'true'}, 'apply': {'model': arm, 'y': 90}},
    {'when': {'south': 'true'}, 'apply': {'model': arm, 'y': 180}},
    {'when': {'west': 'true'}, 'apply': {'model': arm, 'y': 270}},
    {'when': {'up': 'true'}, 'apply': {'model': arm, 'x': 270}},
    {'when': {'down': 'true'}, 'apply': {'model': arm, 'x': 90}}]})
put(f'{OUT}/assets/{NS}/items/basic_universal_cable.json', {'model': {'type': 'minecraft:model', 'model': f'{NS}:block/basic_universal_cable_core'}})
add_block('basic_universal_cable', 'minecraft:chorus_plant', {'hardness': 0.5, 'explosion_resistance': 6.0},
          {'connects_to': f'#{NS}:energy_connectable', 'thickness': 6.0})

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

# what joins a cable: the cables, the cubes and the machines
put(f'{OUT}/data/{NS}/tags/block/energy_connectable.json', {'values': [
    f'{NS}:basic_universal_cable', f'{NS}:basic_energy_cube', f'{NS}:creative_energy_cube', *[f'{NS}:{m}' for m in MACHINES]]})

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
    panel.alpha_composite(gui_sprite('arrow/up'), layout['up_arrow'])
    panel.alpha_composite(gui_sprite('progress/bar/empty'), layout['progress'])
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

MACHINE_NAMES = {'crusher': 'Crusher', 'enrichment_chamber': 'Enrichment Chamber', 'energized_smelter': 'Energized Smelter',
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
tab_items = tab_sprite('tab_items', 'transmission/items', 0xCFCFCF)
tab_energy = tab_sprite('tab_energy', 'transmission/energy', 0x59C15F)

def side_config_widgets():
    """The window's widgets: for each transmission its tabs, title, the six side buttons coloured by their data type, the
    eject state and the clear button; the transmission chosen is the screen's variable `type` (0 items, 1 energy)."""
    widgets = []
    # slot of the screen's data a transmission's sides are in: its first word, then the word with top and bottom
    words = {0: (6, 7), 1: (8, 9)}
    titles = {0: 'Items Config', 1: 'Energy Config'}
    for ty in (0, 1):
        group = []
        # the title, centred between the close button and the eject button's end
        group.append({'type': 'text', 'x': 20 + (116 - text_width(titles[ty])) // 2, 'y': 5, 'text': titles[ty], 'color': '#404040'})
        for side, (x, y) in SIDE_BUTTONS.items():
            slot, idx = (words[ty][0], side) if side < 4 else (words[ty][1], side - 4)
            group.append({'type': 'swatch', 'x': x + 2, 'y': y + 2, 'w': 18, 'h': 18, 'value': {'nibble': [slot, idx]},
                          'colors': SWATCH_COLORS, 'labels': [DATA_LABELS[i] for i in range(10)],
                          'tooltip': ['Front', 'Left', 'Right', 'Back', 'Top', 'Bottom'][side] + '\n{label}'})
            group.append({'type': 'button', 'x': x, 'y': y, 'w': 22, 'h': 22, 'id': 1000 + ty * 100 + side * 10,
                          'alt_id': 1000 + ty * 100 + side * 10 + 1})
        group.append({'type': 'button', 'x': 136, 'y': 95, 'w': 14, 'h': 14, 'id': 3000 + ty * 10, 'alt_id': 3000 + ty * 10 + 1,
                      'tooltip': 'Clear sides\nClick to cycle every side, right click to go back'})
        if ty == 0:
            group.append({'type': 'sprite', 'texture': eject_tex, 'x': 136, 'y': 6, 'w': 14, 'h': 14})
            group.append({'type': 'button', 'x': 136, 'y': 6, 'w': 14, 'h': 14, 'id': 2000, 'tooltip': 'Auto-eject'})
            for on, text in ((0, 'Eject: Off'), (1, 'Eject: On')):
                group.append({'type': 'show', 'when': {'data': {'value': {'nibble': [7, 2]}, 'eq': on}}, 'widgets': [
                    {'type': 'text', 'x': 41, 'y': 28, 'text': text, 'color': '#3CFE9A'}]})
        else:
            group.append({'type': 'text', 'x': 41, 'y': 28, 'text': "Can't Eject", 'color': '#3CFE9A'})
        widgets.append({'type': 'show', 'when': {'local': {'var': 'type', 'eq': ty}}, 'widgets': group})
    # the tabs of the other transmission, outside the window at its left (GuiConfigTypeTab)
    for ty, tex, name in ((0, tab_items, 'Items'), (1, tab_energy, 'Energy')):
        widgets.append({'type': 'show', 'when': {'local': {'var': 'type', 'eq': 1 - ty}}, 'widgets': [
            {'type': 'sprite', 'texture': tex, 'x': -26, 'y': 2 + 28 * ty, 'w': 26, 'h': 26},
            {'type': 'button', 'x': -26, 'y': 2 + 28 * ty, 'w': 26, 'h': 26, 'local': {'set': {'var': 'type', 'value': ty}},
             'tooltip': name}]})
    widgets.append({'type': 'text', 'x': (WIN_W - text_width('Slots')) // 2, 'y': 120, 'text': 'Slots', 'color': '#787878'})
    widgets.append({'type': 'button', 'x': 6, 'y': 6, 'w': 12, 'h': 12, 'local': 'close', 'tooltip': 'Close'})
    return widgets

for m in MACHINES:
    layout = LAYOUTS.get(m, LAYOUTS['electric'])
    title = MACHINE_NAMES.get(m, m.replace('_', ' ').title())
    px, py = layout['progress']
    put(f'{OUT}/data/{NS}/horizon/menu/{m}.json', {
        'slots': len(layout['slots']), 'data_slots': 10, 'results': layout['results'],
        'screen': {'texture': machine_panel(m, layout), 'slots': layout['slots'],
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
            {'type': 'text', 'x': 128, 'y': 72, 'text': 'v0.1.17', 'color': '#808080'},
            # GuiSideConfigurationTab at -26, 6: opens the side configuration window
            {'type': 'sprite', 'texture': tab_config, 'x': -26, 'y': 6, 'w': 26, 'h': 26},
            {'type': 'button', 'x': -26, 'y': 6, 'w': 26, 'h': 26, 'local': {'open': 'side_config'}, 'tooltip': 'Configuration'}],
                   'windows': [{'id': 'side_config', 'x': WIN_X, 'y': WIN_Y, 'width': WIN_W, 'height': WIN_H,
                                'texture': window_tex, 'widgets': side_config_widgets()}]}})
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
MACHINE_TYPES = {f'{NS}:crushing', f'{NS}:enriching', f'{NS}:sawing', f'{NS}:combining'}
for p in sorted(glob.glob(f'{GEN}/data/{NS}/recipe/**/*.json', recursive=True)):
    rel = os.path.relpath(p, f'{GEN}/data/{NS}/recipe')
    r = read_json(p)
    if r.get('type') not in MACHINE_TYPES:
        continue
    outs = [r.get(k, {}).get('id', '') for k in ('output', 'main_output', 'secondary_output')]
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

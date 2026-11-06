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
MACHINES = ['crusher', 'enrichment_chamber', 'energized_smelter']
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
    # so every side has its port and its LEDs. The configurator is not in the port: the sides do not change.
    shift_u, shift_v = base['leds']['lit_uv_shift']
    def plain(e, lit=False):
        e = {k: v for k, v in e.items() if k != 'shade_direction_override'}
        if lit:
            e['light_emission'] = 15
            e['faces'] = {d: dict(f, uv=[f['uv'][0] + shift_u, f['uv'][1] + shift_v,
                                         f['uv'][2] + shift_u, f['uv'][3] + shift_v]) if 'uv' in f else f
                          for d, f in e['faces'].items()}
        return e
    for side in ('bottom', 'top', 'front', 'back', 'right', 'left'):
        elements += [plain(e, lit=side == 'front') for e in base['leds'][side]]
        elements += [plain(e) for e in base['ports'][side]]
    # The core is not in the model: Mekanism draws it in code, turning, and so does the port (a renderer).
    put(f'{OUT}/data/{NS}/horizon/renderer/{tier}_energy_cube.json', {'elements': [
        # RenderEnergyCube: scale 0.4 of a 16-unit cube, bounce sin(3 deg * t) / 7, a turn about Y at 4 deg a
        # tick and one about (0, 1, 1) from 36 deg at 4 deg a tick, full bright, only with energy.
        {'type': 'animated', 'from': [0, 0, 0], 'to': [16, 16, 16], 'sprite': textures['core'],
         'scale': 0.4, 'center': [8, 8, 8],
         'bounce': {'amplitude': 1 / 7, 'degrees_per_tick': 3.0},
         'rotations': [{'axis': [0, 1, 0], 'degrees_per_tick': 4.0},
                       {'axis': [0, 1, 1], 'degrees_per_tick': 4.0, 'offset': 36.0}],
         'fullbright': True, 'only_with_energy': True}]})
    put(f'{OUT}/assets/{NS}/models/block/energy_cube_{tier}.json',
        {'parent': 'minecraft:block/block', 'textures': textures, 'elements': elements})
    n = f'{tier}_energy_cube'
    put(f'{OUT}/assets/{NS}/blockstates/{n}.json', {'variants': {
        f'facing={f}': {'model': f'{NS}:block/energy_cube_{tier}', **({'y': y} if y else {})}
        for f, y in (('north', 0), ('east', 90), ('south', 180), ('west', 270))}})
    put(f'{OUT}/assets/{NS}/items/{n}.json', {'model': {'type': 'minecraft:model', 'model': f'{NS}:block/energy_cube_{tier}'}})
    # An ender chest's shape is not a full cube, so it does not hide the blocks beside the cube's open frame
    # (a furnace's does); its light is switched off.
    add_block(n, 'minecraft:ender_chest', {'hardness': 3.5, 'explosion_resistance': 16.0, 'requires_tool': True, 'light': 0})

# what joins a cable: the cables, the cubes and the machines
put(f'{OUT}/data/{NS}/tags/block/energy_connectable.json', {'values': [
    f'{NS}:basic_universal_cable', f'{NS}:basic_energy_cube', f'{NS}:creative_energy_cube', *[f'{NS}:{m}' for m in MACHINES]]})

# menus of the machines: slots at Mekanism's places, an energy bar, the progress and the title
for m in MACHINES:
    put(f'{OUT}/data/{NS}/horizon/menu/{m}.json', {
        'slots': 2, 'data_slots': 6, 'results': [1],
        'screen': {'slots': [[56, 17], [116, 35]], 'widgets': [
            {'type': 'bar', 'x': 164, 'y': 16, 'w': 6, 'h': 54, 'value': {'pair': [1, 2]}, 'max': {'pair': [3, 4]},
             'color': '#FFC83C', 'direction': 'up', 'tooltip': 'Energy: {value} / {max}'},
            {'type': 'bar', 'x': 78, 'y': 38, 'w': 24, 'h': 6, 'value': {'data': 0}, 'max': {'const': 200},
             'color': '#46C846', 'direction': 'right', 'tooltip': 'Progress: {percent}%'},
            {'type': 'text', 'x': 90, 'y': 6, 'text': 'Status {5}', 'color': '#404040'},
            {'type': 'text', 'x': 140, 'y': 6, 'text': 'v0.1.7', 'color': '#808080'}]}})
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
MACHINE_TYPES = {f'{NS}:crushing', f'{NS}:enriching'}
for p in sorted(glob.glob(f'{GEN}/data/{NS}/recipe/**/*.json', recursive=True)):
    rel = os.path.relpath(p, f'{GEN}/data/{NS}/recipe')
    r = read_json(p)
    if r.get('type') not in MACHINE_TYPES:
        continue
    outid = r.get('output', {}).get('id', '')
    if outid.startswith(NS + ':') and outid.split(':')[1] not in have:
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
    if os.path.exists(path) or '/' not in tag:
        continue
    cat, mat = tag.split('/', 1)
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

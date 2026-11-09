#!/usr/bin/env python3 -I
"""Builds the base the converter reads from the release the port follows (Mekanism 1.21.1-10.7.19.85, github.com/mekanism/Mekanism,
tag v1.21.1-10.7.19.85): a copy of its generated data and resources with the recipes written in the one shape the port's code reads.

The release writes ingredients as `{"item": x}` / `{"tag": x}` and chemicals as `{"amount": n, "chemical": x}`; the port (the game it
runs on is newer) reads `"x"` / `"#x"` and `{"amount": n, "ingredient": x}`. Only the shape changes here, never a number.

Usage: base_1211.py <mekanism checkout of the tag> <output base folder> [<26.3 checkout, for the chemicals' tint and icon>]
"""
import json, os, shutil, sys

MOD_DATA = {'mekanism', 'mekanismgenerators', 'mekanismtools', 'mekanismadditions'}


def ingredient(v):
    """A vanilla or Mekanism item ingredient in the port's shape."""
    if isinstance(v, list):
        return [ingredient(x) for x in v]
    if not isinstance(v, dict):
        return v
    if 'item' in v and len(v) == 1:
        return v['item']
    if 'tag' in v and len(v) == 1:
        return '#' + v['tag']
    t = v.get('type')
    if t == 'neoforge:difference':
        return {'neoforge:ingredient_type': t, 'base': ingredient(v['base']), 'subtracted': ingredient(v['subtracted'])}
    if t == 'neoforge:intersection':
        return {'neoforge:ingredient_type': t, 'children': [ingredient(c) for c in v['children']]}
    if t in ('neoforge:compound', 'neoforge:or'):
        return {'type': 'neoforge:or', 'values': [ingredient(c) for c in v.get('children', v.get('values', []))]}
    return {k: ingredient(x) for k, x in v.items()}


def sized(v):
    """An input with a size (`count` for items, `amount` for chemicals and fluids): its stuff moves under `ingredient`."""
    if not isinstance(v, dict):
        return v
    out = {k: x for k, x in v.items() if k in ('count', 'amount')}
    rest = {k: x for k, x in v.items() if k not in ('count', 'amount')}
    if 'chemical' in rest and len(rest) == 1:
        out['ingredient'] = rest['chemical']
    elif 'fluid' in rest and len(rest) == 1:
        out['ingredient'] = rest['fluid']
    elif 'tag' in rest and len(rest) == 1:
        out['ingredient'] = '#' + rest['tag']
    elif 'item' in rest and len(rest) == 1:
        out['ingredient'] = rest['item']
    else:
        out['ingredient'] = ingredient(rest)
    return out


MEK_SIZED = {'input', 'main_input', 'extra_input', 'item_input', 'chemical_input', 'fluid_input', 'left_input', 'right_input'}


def recipe(j):
    t = j.get('type', '')
    out = {}
    for k, v in j.items():
        if k in ('category', 'group', 'neoforge:conditions'):
            continue
        if t.startswith('mekanism:') and k in MEK_SIZED:
            out[k] = sized(v)
        elif k == 'key':
            out[k] = {c: ingredient(x) for c, x in v.items()}
        elif k == 'ingredients':
            out[k] = [ingredient(x) for x in v]
        elif k in ('ingredient', 'base', 'addition', 'template') and not t.startswith('mekanism:'):
            out[k] = ingredient(v)
        else:
            out[k] = v
    if t == 'mekanism:separating':
        out['output'] = {'left': out.pop('left_chemical_output'), 'right': out.pop('right_chemical_output')}
    if t == 'mekanism:reaction':
        o = {}
        if 'chemical_output' in out:
            o['chemical'] = out.pop('chemical_output')
        if 'item_output' in out:
            o['item'] = out.pop('item_output')
        out['output'] = o
    return out


def copy_tree(src, dst, convert_recipes):
    for root, _, files in os.walk(src):
        rel = os.path.relpath(root, src)
        for f in files:
            s, d = os.path.join(root, f), os.path.join(dst, rel, f)
            os.makedirs(os.path.dirname(d), exist_ok=True)
            if convert_recipes and f.endswith('.json') and (os.sep + 'recipe' + os.sep) in s and '/compat/' not in s.replace(os.sep, '/'):
                try:
                    j = json.load(open(s))
                except ValueError:
                    shutil.copyfile(s, d)
                    continue
                json.dump(recipe(j), open(d, 'w'), indent=2)
            else:
                shutil.copyfile(s, d)


if __name__ == '__main__':
    if len(sys.argv) < 3:
        print(__doc__)
        sys.exit(1)
    SRC, OUT = sys.argv[1], sys.argv[2]
    shutil.rmtree(OUT, ignore_errors=True)
    copy_tree(f'{SRC}/src/datagen/generated', f'{OUT}/src/datagen/generated', True)
    copy_tree(f'{SRC}/src/main/resources', f'{OUT}/src/main/resources', False)
    for m in ('generators', 'tools', 'additions'):
        p = f'{SRC}/src/{m}/resources'
        if os.path.isdir(p):
            copy_tree(p, f'{OUT}/src/{m}/resources', False)
    # The chemicals' tint and icon (the release writes them in Java): the same for every version, read from the checkout of the newer one.
    if len(sys.argv) > 3:
        shutil.copytree(f'{sys.argv[3]}/src/datagen/generated/mekanism/data/mekanism/mekanism/chemical',
                        f'{OUT}/src/datagen/generated/mekanism/data/mekanism/mekanism/chemical', dirs_exist_ok=True)
    # The energy cube model: 1.21.1 lists <side>LEDs / <side>Port; the converter reads leds{...} / ports{...}.
    mp = f'{OUT}/src/main/resources/assets/mekanism/models/block/energy_cube/base.json'
    m = json.load(open(mp))
    sides = ('bottom', 'top', 'front', 'back', 'right', 'left')
    m['leds'] = {'lit_uv_shift': [-2, 0], **{k: m.pop(f'{k}LEDs') for k in sides}}
    m['ports'] = {k: m.pop(f'{k}Port') for k in sides}
    json.dump(m, open(mp, 'w'), indent=2)
    # Chemical textures live in other folders in 1.21.1 (liquid/, infuse_type/, slurry/, pigment/); alias them to the layout the converter reads.
    tex = f'{OUT}/src/main/resources/assets/mekanism/textures'
    for old, new in (('liquid', 'mek_liquid'), ('infuse_type', 'mek_chemical/infuse_type'), ('slurry', 'mek_chemical/slurry'), ('pigment', 'mek_chemical/pigment')):
        if os.path.isdir(f'{tex}/{old}'):
            shutil.copytree(f'{tex}/{old}', f'{tex}/{new}', dirs_exist_ok=True)
    # 1.21.1 has no `items/` definitions (they came with 1.21.4): every item model becomes the definition the converter reads.
    for root in (f'{OUT}/src/main/resources/assets/mekanism', f'{OUT}/src/datagen/generated/mekanism/assets/mekanism'):
        base = f'{root}/models/item'
        for r, _, fs in os.walk(base):
            for f in fs:
                if f.endswith('.json'):
                    rel = os.path.relpath(os.path.join(r, f), base)[:-5]
                    dst = f'{OUT}/src/datagen/generated/mekanism/assets/mekanism/items/{rel}.json'
                    if not os.path.exists(dst):
                        os.makedirs(os.path.dirname(dst), exist_ok=True)
                        json.dump({'model': {'type': 'minecraft:model', 'model': f'mekanism:item/{rel}'}}, open(dst, 'w'), indent=2)
    print('base built in', OUT)

#!/usr/bin/env python3
"""Screens of the original against the port's / Telas do original contra as do port.
Usage / Uso: python3 -I tools/telas.py <mekanism-1211 checkout> [--write]
Source of truth: the container types the original registers (MekanismContainerTypes, GeneratorsContainerTypes): one per screen.
Port side: the menu files tools/convert.py writes, and the blocks mod/src/lib.rs registers with `menu: true`.
"""
import os, re, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CONV = open(os.path.join(ROOT, 'tools', 'convert.py'), encoding='utf-8').read()
SRC = os.path.join(ROOT, 'mod', 'src')

def consts(names):
    """The lists convert.py builds its menus from, evaluated from their own lines."""
    env = {}
    for n in names:
        m = re.search(r'^' + n + r' = (.*?)(?=^\S)', CONV, re.S | re.M)
        exec(n + ' = ' + m.group(1), env)
    return env

def port_menus():
    env = consts(['PROCESS_MACHINES', 'CONTINUOUS_MACHINES', 'GENERATORS', 'TIERS', 'FLUID_TANKS', 'CHEM_TANKS', 'FACTORY_LINES',
                  'FACTORY_KINDS', 'FACTORIES', 'MACHINES', 'BINS'])
    menus = set(re.findall(r"horizon/menu/([a-z_]+)\.json", CONV))
    menus |= set(env['MACHINES']) | set(env['BINS']) | set(env['GENERATORS'])
    menus |= set(re.findall(r"'([a-z]+_energy_cube)': \(", CONV))
    return menus

def port_menu_blocks():
    """Blocks lib.rs registers with a container whose `menu` is true."""
    lib = open(os.path.join(SRC, 'lib.rs'), encoding='utf-8').read()
    ids = {}
    for f in os.listdir(SRC):
        mod = f[:-3]
        for m in re.finditer(r'pub const ([A-Z_0-9]+): &str = "mekanism:([a-z_0-9]+)"', open(os.path.join(SRC, f), encoding='utf-8').read()):
            ids[(mod, m.group(1))] = m.group(2)
    out = set()
    for m in re.finditer(r'= kind\(([^,]+), (?:true|false)\);(.{0,900}?)block_entity_kind', lib, re.S):
        if 'menu: true' not in m.group(2):
            continue
        expr = m.group(1).strip()
        lit = re.match(r'"mekanism:([a-z_0-9]+)"', expr)
        if lit:
            out.add(lit.group(1))
        elif '::' in expr:
            mod, name = expr.split('::')[-2:]
            if (mod, name) in ids:
                out.add(ids[(mod, name)])
            else:
                out.add('?' + expr)
        else:
            # A loop variable: the block ids are in the `for ... in [...]` line just before.
            head = lib[:m.start()]
            loops = [l for l in head.split('\n')[-12:] if l.strip().startswith('for ') and re.search(r'\b' + re.escape(expr) + r'\b', l)]
            found = [ids[k] for k in re.findall(r'(\w+)::([A-Z_0-9]+)', loops[-1]) if k in ids] if loops else []
            out.update(found or ['?' + expr])
    return out

# A screen of the original -> the port's menu files that stand for it (families and multiblocks open through another block).
ALIAS = {
    'dynamic_tank': ['dynamic_valve'], 'sps_casing': ['sps_port'], 'induction_matrix': ['induction_port'],
    'thermoelectric_boiler': ['boiler_valve'], 'industrial_turbine': ['turbine_valve'], 'fission_reactor': ['fission_reactor_port'],
    'factory': ['basic_crushing_factory'], 'chemical_tank': ['basic_chemical_tank'], 'fluid_tank': ['basic_fluid_tank'],
    'energy_cube': ['basic_energy_cube'], 'personal_storage_block': ['personal_chest', 'personal_barrel'],
}
TABS = {'boiler_stats', 'matrix_stats', 'fission_reactor_stats', 'fusion_reactor_fuel', 'fusion_reactor_heat', 'fusion_reactor_stats',
        'turbine_stats', 'digital_miner_config', 'qio_frequency_select_tile', 'fission_reactor_logic_adapter', 'fusion_reactor_logic_adapter'}
ITEMS = {'module_tweaker', 'dictionary', 'portable_teleporter', 'seismic_reader', 'qio_frequency_select_item', 'portable_qio_dashboard',
         'main_robit', 'inventory_robit', 'smelting_robit', 'crafting_robit', 'repair_robit', 'personal_storage_item'}

def java_screens(checkout):
    out = []
    for rel in ('src/main/java/mekanism/common/registries/MekanismContainerTypes.java',
                'src/generators/java/mekanism/generators/common/registries/GeneratorsContainerTypes.java'):
        t = open(os.path.join(checkout, rel), encoding='utf-8').read()
        for m in re.finditer(r'CONTAINER_TYPES\.\w+\((?:(?:MekanismBlocks|MekanismItems|GeneratorsBlocks)\.([A-Z_]+)|"([a-z_]+)")', t):
            out.append((m.group(1) or m.group(2)).lower())
    return out

def main():
    checkout = sys.argv[1]
    menus, wired = port_menus(), port_menu_blocks()
    rows, n = [], {'ok': 0, 'sem': 0, 'aba': 0, 'item': 0}
    for s in java_screens(checkout):
        names = ALIAS.get(s, [s])
        have = [x for x in names if x in menus]
        opened = [x for x in names if x in wired]
        if s in ITEMS:
            kind, state = 'item/entidade', 'FALTA (sem tela de item ou de entidade na API)'
            n['item'] += 1
        elif s in TABS:
            hit = s.replace('_', ' ') in CONV or s in CONV or s.split('_')[-1] in ('stats',) and (s.split('_')[0] + '_stats') in CONV
            kind, state = 'aba', 'conferir à mão' + (' (há menção no convert.py)' if hit else ' (sem menção no convert.py)')
            n['aba'] += 1
        elif have:
            kind, state = 'bloco', 'TEM tela'
            n['ok'] += 1
        else:
            kind = 'bloco'
            state = 'SEM tela' + (' — bloco ligado no mod com menu' if opened else ' — bloco sem menu no mod')
            n['sem'] += 1
        rows.append((s, kind, state))
    extra = sorted(m for m in menus if not any(m in ALIAS.get(s, [s]) or m.endswith(('_factory', '_fluid_tank', '_chemical_tank', '_energy_cube')) for s in java_screens(checkout)))
    # The evaporation loop gives a menu to the controller only (the valve has no slots): the window of text the search reads cannot tell.
    no_file = sorted(w for w in wired if w not in menus and not w.startswith('?') and w != 'thermal_evaporation_valve')
    lines = ['# TELAS — telas do original × telas do port / Screens of the original × the port', '',
             'Gerado por / generated by `tools/telas.py`. Fonte: os tipos de contêiner que o original registra (uma tela cada). / Source: the container types the original registers.', '',
             f'**Resumo / Summary:** telas de bloco com tela no port: **{n["ok"]}**; telas de bloco sem tela: **{n["sem"]}**; abas e subtelas (conferir à mão): **{n["aba"]}**; telas de item ou entidade (faltam todas): **{n["item"]}**. Total do original: {len(rows)}.', '',
             '| Tela do original / Original screen | Tipo / Kind | Estado / State |', '|---|---|---|']
    lines += [f'| `{s}` | {k} | {st} |' for s, k, st in rows]
    lines += ['', '## Blocos que o mod registra com menu mas sem arquivo de tela / Blocks registered with a menu but no screen file', '',
              ', '.join(f'`{w}`' for w in no_file) or '(nenhum / none)', '',
              '## Telas do port que o original não tem / Port screens the original does not have', '',
              ', '.join(f'`{e}`' for e in extra) or '(nenhuma / none)', '',
              '"TEM tela" quer dizer que o arquivo existe, não que a tela foi conferida elemento a elemento com a classe `Gui*` do Java. / "TEM tela" means the file exists, not that it was checked element by element against the Java `Gui*` class.']
    text = '\n'.join(lines) + '\n'
    print(text)
    if '--write' in sys.argv:
        open(os.path.join(ROOT, 'TELAS.md'), 'w', encoding='utf-8').write(text)

main()

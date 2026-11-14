"""Compares the coordinates the original's screens place elements at (`new GuiXxx(this, x, y, ...)`, slots, labels) with the ones in the port's menu JSON.
Usage: revisar_telas.py <mekanism checkout> <port output folder>. Prints, per screen, the pairs of the original that no widget, slot or sprite of the port has
(within one pixel, the port draws bars and sprites at the original's x - 1, y - 1)."""
import json, os, re, sys, glob
SRC, OUT = sys.argv[1], sys.argv[2]
JAVA = f'{SRC}/src/main/java/mekanism'
def snake(n):
    return re.sub(r'(?<!^)(?=[A-Z])', '_', n).lower()
ALIASES = {'prc': 'pressurized_reaction_chamber', 'pigment_mixer': 'pigment_mixer', 'qio_drive_array': 'qio_drive_array', 'sps': 'sps_port', 'fission_reactor': 'fission_reactor_port',
           'fusion_reactor': 'fusion_reactor_controller', 'boiler': 'boiler_valve', 'turbine': 'turbine_valve', 'thermal_evaporation_controller': 'thermal_evaporation_controller',
           'induction_matrix': 'induction_port', 'dynamic_tank': 'dynamic_tank'}
menus = {os.path.basename(p)[:-5]: json.load(open(p)) for p in glob.glob(f'{OUT}/data/mekanism/horizon/menu/*.json')}
def pairs_of(menu):
    s = menu['screen']
    pts = set()
    for sl in s.get('slots', []):
        if sl:
            pts.add(tuple(sl))
    def walk(ws):
        for w in ws:
            if 'x' in w and 'y' in w:
                pts.add((w['x'], w['y']))
            walk(w.get('widgets', []))
    walk(s.get('widgets', []))
    return pts
def near(p, pts):
    return any(abs(p[0] - q[0]) <= 1 and abs(p[1] - q[1]) <= 1 for q in pts)
rx = re.compile(r'new\s+(Gui\w+|\w*Slot\w*)\s*\(\s*(?:this|\w+)\s*,\s*(-?\d+)\s*,\s*(-?\d+)')
bad = 0
for f in sorted(glob.glob(f'{JAVA}/client/gui/**/Gui*.java', recursive=True)):
    name = os.path.basename(f)[3:-5]
    key = snake(name)
    key = ALIASES.get(key, key)
    if key not in menus:
        continue
    text = open(f).read()
    pts = pairs_of(menus[key])
    want = []
    for m in rx.finditer(text):
        kind, x, y = m.group(1), int(m.group(2)), int(m.group(3))
        if kind in ('GuiRedstoneControlTab', 'GuiSecurityTab', 'GuiUpgradeTab', 'GuiHeatTab', 'GuiEnergyTab', 'GuiConfigurationTab', 'GuiTransporterConfigTab'):
            continue
        want.append((kind, x, y))
    miss = [(k, x, y) for k, x, y in want if not near((x, y), pts)]
    print(f'{key}: {len(want)} placed, {len(miss)} not in the port')
    for k, x, y in miss:
        print(f'    {k} at {x}, {y}')
    bad += len(miss)
print('TOTAL missing', bad)

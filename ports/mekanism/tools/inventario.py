#!/usr/bin/env python3
"""Mechanical inventory of the Mekanism port against the original release.
Inventário mecânico do port do Mekanism contra a release original.

Usage / Uso:
    python3 -I tools/inventario.py <out dir> <port.hmod> <jar> [<jar> ...] [--src <java source root>] [--port-src <mod/src>]

Compares, layer by layer, what the original jars (1.21.1-10.7.19.85: Mekanism, Generators, Tools, Additions)
contain with what the built `.hmod` contains. Every row has a state:
    OK         same path and same content (JSON normalized)
    DIFF       same path, different content
    FALTA      in the original, not in the port
    EXTRA      in the port, not in the original
    ?          needs a human (code layers, where only a heuristic is possible)
Output: <out dir>/INVENTARIO.md (summary) and <out dir>/csv/<layer>.csv (every row).
"""
import csv
import hashlib
import json
import os
import re
import sys
import zipfile
from collections import Counter, defaultdict

LAYERS = [
    # name, prefix inside the jar, which suffixes count
    ("recipe", "data/mekanism/recipe/", (".json",)),
    ("loot_table", "data/mekanism/loot_table/", (".json",)),
    ("advancement", "data/mekanism/advancement/", (".json",)),
    ("tags_mekanism", "data/mekanism/tags/", (".json",)),
    ("tags_c", "data/c/tags/", (".json",)),
    ("tags_minecraft", "data/minecraft/tags/", (".json",)),
    ("damage_type", "data/mekanism/damage_type/", (".json",)),
    ("worldgen", "data/mekanism/worldgen/", (".json",)),
    ("neoforge_data", "data/mekanism/neoforge/", (".json",)),
    ("blockstate", "assets/mekanism/blockstates/", (".json",)),
    ("model", "assets/mekanism/models/", (".json",)),
    ("item_def", "assets/mekanism/items/", (".json",)),
    ("texture", "assets/mekanism/textures/", (".png", ".mcmeta")),
    ("gui", "assets/mekanism/gui/", (".json", ".png")),
    ("sound_file", "assets/mekanism/sounds/", (".ogg",)),
    ("particle", "assets/mekanism/particles/", (".json",)),
    ("equipment", "assets/mekanism/equipment/", (".json",)),
    ("atlas", "assets/mekanism/atlases/", (".json",)),
    ("font", "assets/mekanism/font/", (".json", ".png")),
]


def norm_json(raw):
    try:
        return json.dumps(json.loads(raw), sort_keys=True, separators=(",", ":"))
    except Exception:
        return raw.decode("utf-8", "replace") if isinstance(raw, bytes) else raw


def digest(path, data):
    if path.endswith((".json", ".mcmeta")):
        return hashlib.sha1(norm_json(data).encode()).hexdigest()
    return hashlib.sha1(data).hexdigest()


def read_zip(path):
    z = zipfile.ZipFile(path)
    return {n: z for n in z.namelist() if not n.endswith("/")}, z


def load_all(paths):
    files = {}
    zips = []
    for p in paths:
        names, z = read_zip(p)
        zips.append(z)
        for n in names:
            files[n] = z
    return files


def compare_layers(orig, port):
    rows = defaultdict(list)
    for layer, prefix, suffixes in LAYERS:
        o = {n for n in orig if n.startswith(prefix) and n.endswith(suffixes)}
        p = {n for n in port if n.startswith(prefix) and n.endswith(suffixes)}
        for n in sorted(o | p):
            if n in o and n not in p:
                state, extra = "FALTA", ""
            elif n in p and n not in o:
                state, extra = "EXTRA", ""
            else:
                a = digest(n, orig[n].read(n))
                b = digest(n, port[n].read(n))
                state = "OK" if a == b else "DIFF"
                extra = ""
            rows[layer].append((n, state, extra))
    return rows


def lang_rows(orig, port):
    """Translation keys of en_us: the registry of everything the player sees."""
    n = "assets/mekanism/lang/en_us.json"
    o = json.loads(orig[n].read(n)) if n in orig else {}
    p = json.loads(port[n].read(n)) if n in port else {}
    rows = []
    for k in sorted(set(o) | set(p)):
        if k in o and k not in p:
            rows.append((k, "FALTA", o[k]))
        elif k in p and k not in o:
            rows.append((k, "EXTRA", p[k]))
        else:
            rows.append((k, "OK" if o[k] == p[k] else "DIFF", f"{o[k]!r} != {p[k]!r}" if o[k] != p[k] else ""))
    return rows


def sounds_rows(orig, port):
    n = "assets/mekanism/sounds.json"
    o = json.loads(orig[n].read(n)) if n in orig else {}
    p = json.loads(port[n].read(n)) if n in port else {}
    return [(k, "FALTA" if k not in p else ("OK" if o[k] == p[k] else "DIFF"), "") for k in sorted(o)]


def registry_rows(orig, port):
    """Blocks, items, entities... by the translation keys, against what the port's mod registers."""
    n = "assets/mekanism/lang/en_us.json"
    o = json.loads(orig[n].read(n))
    port_blocks = {m.group(1) for k in port if (m := re.match(r"assets/mekanism/blockstates/(.+)\.json$", k))}
    port_items = {m.group(1) for k in port if (m := re.match(r"assets/mekanism/items/(.+)\.json$", k))}
    rows = []
    for k in sorted(o):
        m = re.match(r"(block|item|entity|gas|infuse_type|slurry|pigment|fluid_type|attribute|effect|damage_type)\.mekanism\.(.+)$", k)
        if not m or m.group(2).endswith(("_tooltip", ".desc", ".description")) or "." in m.group(2):
            continue
        kind, name = m.groups()
        if kind == "block":
            state = "OK" if name in port_blocks else "FALTA"
        elif kind == "item":
            state = "OK" if name in port_items else "FALTA"
        else:
            state = "?"
        rows.append((f"{kind}:{name}", state, o[k]))
    return rows


def java_classes(src_root, port_src):
    """Code layers: classes of the original by package, with a heuristic for a match in the port (snake_case of the class name)."""
    port_text = ""
    if port_src and os.path.isdir(port_src):
        for f in os.listdir(port_src):
            if f.endswith(".rs"):
                port_text += open(os.path.join(port_src, f), encoding="utf-8", errors="replace").read().lower() + "\n"
    layers = {
        "tile_entity": r"mekanism/common/tile/",
        "container_menu": r"mekanism/common/inventory/container/",
        "network_packet": r"mekanism/common/network/",
        "item_class": r"mekanism/common/item/",
        "block_class": r"mekanism/common/block/",
        "recipe_class": r"mekanism/common/recipe/",
        "capability": r"mekanism/common/capabilities/",
        "multiblock": r"mekanism/common/content/",
        "client_gui": r"mekanism/client/gui/",
        "client_render": r"mekanism/client/render/",
        "integration": r"mekanism/common/integration/",
        "attachment": r"mekanism/common/attachments/",
    }
    rows = defaultdict(list)
    for root in src_root:
        for dp, _, fs in os.walk(root):
            rel = dp.replace(os.sep, "/") + "/"
            for f in fs:
                if not f.endswith(".java"):
                    continue
                for layer, pat in layers.items():
                    if re.search(pat, rel):
                        cls = f[:-5]
                        words = re.sub(r"(?<!^)(?=[A-Z])", "_", cls).lower()
                        for pre in ("tile_entity_", "gui_", "container_", "pv_", "item_", "block_"):
                            if words.startswith(pre):
                                words = words[len(pre):]
                        hit = bool(words) and words in port_text
                        rows[layer].append((os.path.join(dp, f).replace(os.sep, "/").split("/java/", 1)[-1], "?" if hit else "FALTA?", "match heurístico por nome / name heuristic" if hit else ""))
    return rows


def config_rows(src_roots):
    """Every config option of the original (name and default), to be audited by hand against the port."""
    rows = []
    pat = re.compile(r'\.define(?:InRange|InList|Enum|Optional)?\(\s*"([^"]+)"\s*,\s*([^,)\n]+)')
    pat2 = re.compile(r'(?:CachedIntValue|CachedLongValue|CachedFloatValue|CachedDoubleValue|CachedBooleanValue|CachedEnumValue|CachedFloatingLongValue)\.wrap\(\s*\w+\s*,\s*builder\.comment\([^;]*?\.define\w*\(\s*"([^"]+)"\s*,\s*([^,)\n]+)', re.S)
    for root in src_roots:
        cfg = None
        for dp, _, fs in os.walk(root):
            if dp.replace(os.sep, "/").endswith("/config") or "/config/" in dp.replace(os.sep, "/") + "/":
                for f in fs:
                    if f.endswith(".java"):
                        text = open(os.path.join(dp, f), encoding="utf-8", errors="replace").read()
                        for m in pat.finditer(text):
                            rows.append((f"{f[:-5]}.{m.group(1)}", "?", f"default {m.group(2).strip()}"))
    return sorted(set(rows))


def write_csv(outdir, name, rows):
    os.makedirs(os.path.join(outdir, "csv"), exist_ok=True)
    with open(os.path.join(outdir, "csv", name + ".csv"), "w", newline="", encoding="utf-8") as fh:
        w = csv.writer(fh)
        w.writerow(["item", "estado", "nota"])
        w.writerows(rows)


def main(argv):
    if len(argv) < 4:
        print(__doc__)
        return 2
    outdir, hmod, *rest = argv[1:]
    src_roots, port_src, jars = [], None, []
    i = 0
    while i < len(rest):
        if rest[i] == "--src":
            src_roots.append(rest[i + 1]); i += 2
        elif rest[i] == "--port-src":
            port_src = rest[i + 1]; i += 2
        else:
            jars.append(rest[i]); i += 1
    orig = load_all(jars)
    port = load_all([hmod])
    all_rows = compare_layers(orig, port)
    all_rows["lang_en_us"] = lang_rows(orig, port)
    all_rows["sounds_json"] = sounds_rows(orig, port)
    all_rows["registro_blocos_itens"] = registry_rows(orig, port)
    if src_roots:
        for k, v in java_classes([os.path.join(r) for r in src_roots], port_src).items():
            all_rows["codigo_" + k] = v
        all_rows["config_opcoes"] = config_rows(src_roots)
    lines = ["# INVENTARIO / INVENTORY", "",
             "Gerado por / generated by `tools/inventario.py`. PT-BR primeiro, English below each table header.", "",
             "| Camada / Layer | Total original | OK | DIFF | FALTA | EXTRA | ? / FALTA? | Cobertura / Coverage |",
             "|---|---:|---:|---:|---:|---:|---:|---:|"]
    total_o = total_ok = 0
    for layer in sorted(all_rows):
        rows = all_rows[layer]
        write_csv(outdir, layer, rows)
        c = Counter(r[1] for r in rows)
        orig_n = len(rows) - c["EXTRA"]
        ok = c["OK"]
        unk = c["?"] + c["FALTA?"]
        cov = f"{100 * ok / orig_n:.1f}%" if orig_n else "-"
        lines.append(f"| {layer} | {orig_n} | {ok} | {c['DIFF']} | {c['FALTA'] + c['FALTA?']} | {c['EXTRA']} | {unk} | {cov} |")
        if not layer.startswith("codigo_") and layer != "config_opcoes":
            total_o += orig_n
            total_ok += ok
    lines += ["", f"**Cobertura mecânica (camadas de dados) / Mechanical coverage (data layers): {total_ok}/{total_o} = {100 * total_ok / max(total_o, 1):.1f}%**", "",
              "Estados / States: OK = igual ao original; DIFF = existe mas difere; FALTA = não existe no port; EXTRA = só no port; `?` = precisa de revisão humana (camadas de código e config).",
              "Detalhe linha a linha em / line by line in `csv/`."]
    with open(os.path.join(outdir, "INVENTARIO.md"), "w", encoding="utf-8") as fh:
        fh.write("\n".join(lines) + "\n")
    print("\n".join(lines))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

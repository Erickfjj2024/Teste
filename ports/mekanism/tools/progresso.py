#!/usr/bin/env python3
"""Completion percentage of the Mekanism port / Porcentagem de conclusão do port.
Usage / Uso: python3 -I tools/progresso.py [--write]
100% = playable mod equal to the original in everything, including the Horizon API work it needs.
Every component is measured, not guessed; unmeasurable parts count 0 until proven.
"""
import json, os, re, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(ROOT, "mod", "src")

# Modules already compared line by line with the Java (review documents / round 1-3 audits).
REVISADOS = {
    "heat", "fuelwood_heater", "resistive_heater", "qio", "qio_transfer", "qio_dashboard",
    "dynamic", "boiler", "evaporation", "sps", "fission", "fusion", "turbine", "induction", "bounding",
    "mekasuit", "modules", "meka_tool", "robit", "chargepad", "assemblicator", "configuration_card", "electric_bow", "hdpe_elytra",
    "entangloporter", "laser", "miner", "pump", "cube", "generators", "plus", "transmitter",
}

def modules():
    return sorted(f[:-3] for f in os.listdir(SRC) if f.endswith(".rs") and f != "lib.rs")

def declared():
    """Modules lib.rs really USES (a path like `name::` outside the `mod` line), not only declares."""
    t = open(os.path.join(SRC, "lib.rs"), encoding="utf-8").read()
    t = re.sub(r"^\s*(?:pub )?mod \w+;\n", "", t, flags=re.M)
    return {m for m in modules() if re.search(r"\b" + m + r"::", t)}

def inventory_cov():
    p = os.path.join(ROOT, "INVENTARIO.md")
    m = re.search(r"= ([\d.]+)%\*\*", open(p, encoding="utf-8").read()) if os.path.exists(p) else None
    return float(m.group(1)) / 100 if m else 0.0

def fork_done():
    p = os.path.join(ROOT, "progresso.json")
    j = json.load(open(p)) if os.path.exists(p) else {}
    return j.get("fork_implementados", 0), j.get("fork_total", 35), j.get("build_real", 0.0), j.get("jogo_testado", 0.0)

def main():
    mods = modules()
    rev = [m for m in mods if m in REVISADOS]
    dec = [m for m in mods if m in declared()]
    fi, ft, build, jogo = fork_done()
    comps = [
        ("Dados e recursos (receitas, texturas, lang, tags…) / data and assets", 15, inventory_cov()),
        ("Lógica Rust conferida com o Java / logic checked against Java", 25, len(rev) / len(mods)),
        ("Módulos ligados ao lib.rs / modules wired in", 10, len(dec) / len(mods)),
        ("API nova do Horizon (FORK-n) implementada / new Horizon API implemented", 25, fi / ft),
        ("Build real, .hmod e APK / real build, .hmod and APK", 10, build),
        ("Testado em jogo contra o original / tested in game", 15, jogo),
    ]
    total = sum(w * s for _, w, s in comps)
    out = ["# PROGRESSO — conclusão do port / Port completion", "",
           "100% = mod jogável igual ao original em tudo, incluindo a API do Horizon que isso exige. / 100% = playable mod equal to the original in everything, including the Horizon API work it needs.", "",
           "| Componente / Component | Peso / Weight | Nota / Score | Contribuição / Contribution |", "|---|---:|---:|---:|"]
    for n, w, s in comps:
        out.append(f"| {n} | {w} | {100*s:.1f}% | {w*s:.1f} |")
    out += ["", f"**TOTAL: {total:.1f}%**", "",
            f"Módulos Rust: {len(mods)}; conferidos: {len(rev)}; ligados ao lib.rs: {len(dec)}. FORK-n implementados: {fi}/{ft}.",
            "", "Pendentes de conferência / unreviewed: " + ", ".join(m for m in mods if m not in REVISADOS),
            "", "Regras / Rules: build real, fork e jogo contam 0 até haver prova (`progresso.json`). / Real build, fork and game count 0 until proven."]
    text = "\n".join(out) + "\n"
    print(text)
    if "--write" in sys.argv:
        open(os.path.join(ROOT, "PROGRESSO.md"), "w", encoding="utf-8").write(text)

main()

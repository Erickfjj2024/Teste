"""Compara cada receita do datagen do Mekanism (Java) com a convertida para o port, uma a uma.
Compares every datagen recipe with the converted one, one by one.
uso/usage: python3 -I tools/receitas_comparar.py <.../src/datagen/generated> <mod/data> <saida.json>
"""
import glob,json,os,re,sys,collections
import sys
if len(sys.argv)<4: sys.exit("uso: receitas_comparar.py <datagen/generated do Java> <mod/data convertido> <saida.json>")
J=sys.argv[1]
C=sys.argv[2]
OUTJSON=sys.argv[3]
def leaves(v,top,out):
    if isinstance(v,dict):
        for k,x in v.items():
            if k in("neoforge:conditions","category","group"): continue
            leaves(x,top if top else k,out)
    elif isinstance(v,list):
        for x in v: leaves(x,top,out)
    elif isinstance(v,bool): out.append((top,str(v)))
    elif isinstance(v,(int,float)): out.append((top,"n:%g"%v))
    elif isinstance(v,str): out.append((top,re.sub(r"^(mekanism(additions|generators|tools)):","mekanism:",v.lstrip("#"))))
def fp(r):
    out=[]; 
    for k,x in r.items():
        if k in("neoforge:conditions","category","group","type"): continue
        leaves(x,k,out)
    # normalise: ingredient/output wrappers: drop the top key distinctions between input shapes
    return sorted(out)
java={}; compat=0
for p in glob.glob(J+"/*/data/*/recipe/**/*.json",recursive=True):
    r=json.load(open(p))
    ns=p.split("/data/")[1].split("/")[0]
    rel=p.split("/recipe/")[1]
    if "neoforge:conditions" in r: compat+=1; continue
    java[(ns,rel)]=r
conv={}
for p in glob.glob(C+"/*/recipe/**/*.json",recursive=True):
    ns=p.split("/data/")[1].split("/")[0]
    rel=p.split("/recipe/")[1]
    conv[(ns,rel)]=json.load(open(p))
print("java sem condicao de mod externo:",len(java),"| compat (outros mods):",compat,"| convertidas:",len(conv))
# map crafting recipes that the converter renames (a/b.json -> a__b.json, factory_*.json ...)
conv_by_fp=collections.defaultdict(list)
for k,r in conv.items(): conv_by_fp[(r.get("type"),json.dumps(fp(r)))].append(k)
stats=collections.Counter(); bad=collections.defaultdict(list); missing=collections.defaultdict(list)
for (ns,rel),r in sorted(java.items()):
    t=r.get("type","?")
    cands=[(ns,rel),(ns,rel.replace("/","__"))]
    c=next((conv[k] for k in cands if k in conv),None)
    if c is None:
        stats[(t,"AUSENTE")]+=1; missing[t].append(f"{ns}/{rel}"); continue
    a=fp(r); b=fp(c)
    if a==b: stats[(t,"IGUAL")]+=1
    else:
        stats[(t,"DIFERE")]+=1
        sa=collections.Counter(a); sb=collections.Counter(b)
        bad[t].append((f"{ns}/{rel}",list((sa-sb).items())[:4],list((sb-sa).items())[:4]))
types=sorted({t for t,_ in stats})
print("%-34s %6s %6s %6s"%("tipo","igual","difere","ausente"))
tot=collections.Counter()
for t in types:
    row=(stats[(t,"IGUAL")],stats[(t,"DIFERE")],stats[(t,"AUSENTE")])
    for i,k in enumerate(("IGUAL","DIFERE","AUSENTE")): tot[k]+=row[i]
    print("%-34s %6d %6d %6d"%(t,*row))
print("TOTAL",dict(tot))
json.dump({"bad":bad,"missing":missing},open(OUTJSON,"w"),indent=1)

# second pass: look the absent ones up by fingerprint (converter may rename the file)
still=collections.Counter(); found=collections.Counter(); stills=collections.defaultdict(list)
for t,lst in missing.items():
    for rel in lst:
        ns,r=rel.split("/",1)
        rec=java[(ns,r)]
        key=(rec.get("type"),json.dumps(fp(rec)))
        if key in conv_by_fp: found[t]+=1
        else: still[t]+=1; stills[t].append(rel)
print("\nSEGUNDA PASSADA (por conteudo):")
print("achadas sob outro nome:",sum(found.values()),"| realmente ausentes:",sum(still.values()))
for t in sorted(still): print("  ausente de verdade:",t,still[t],"(achadas:",found[t],")")
json.dump(stills,open(OUTJSON.replace(".json","_ausentes.json"),"w"),indent=1)

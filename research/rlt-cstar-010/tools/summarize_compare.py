"""Summarize compare_outputs.py text output: per case/component, variables differing,
largest relative difference (|Δ| / max |base|) and which variable. Usage: summarize_compare.py REPORT.txt"""
import re, sys
cur=None; rows={}; order=[]
for line in open(sys.argv[1]):
    m=re.match(r"^(\S+) / (\S+): (\d+) of (\d+) variables differ", line)
    if m: cur=(m[1],m[2]); rows[cur]={"n":int(m[3]),"of":int(m[4]),"max":0.0,"var":"","abs":0.0}; order.append(cur); continue
    m=re.match(r"^  (\S+)\s+(\d+)/(\d+)\s+(\S+)\s+(\S+)\s", line)
    if m and cur and m[1]!="variable":
        try: rel=float(m[5]); ab=float(m[4])
        except ValueError: continue
        if rel>rows[cur]["max"]: rows[cur].update(max=rel,var=m[1],abs=ab)
allmax=max(r["max"] for r in rows.values())
for k in order:
    r=rows[k]; print(f"| {k[0]} | {k[1]} | {r['n']}/{r['of']} | {r['max']:.2e} | {r['var']} |")
print(f"overall max relative difference: {allmax:.2e}; components: {len(rows)}")

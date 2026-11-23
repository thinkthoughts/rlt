"""Compare per-variable statistics (min, max, mean, rms) between reference and computed hash files.

Usage: stat_compare.py REFERENCE.json COMPUTED.json
Prints, per case and component, the variable with the largest relative change of any statistic,
and a summary. Relative change = |new-old| / max(|old|, tiny); statistics carry 10 significant digits.
"""
import json, sys, math
ref = json.load(open(sys.argv[1])); new = json.load(open(sys.argv[2]))
rows = []
for case, comps in new.items():
    for comp, vars_ in comps.items():
        for var, e in vars_.items():
            r = ref[case][comp][var]
            if r["sha256"] == e["sha256"]:
                continue
            worst = 0.0; which = ""
            for s in ("min", "max", "mean", "rms"):
                o, n = r.get(s), e.get(s)
                if o is None or n is None:
                    continue
                scale = max(abs(o), abs(n))
                d = abs(n - o) / scale if scale > 0 else 0.0
                if d > worst: worst, which = d, s
            rows.append((case, comp, var, worst, which))
by = {}
for case, comp, var, w, s in rows:
    k = (case, comp)
    if k not in by or w > by[k][1]: by[k] = (var, w, s)
for (case, comp), (var, w, s) in by.items():
    n = sum(1 for r in rows if r[0] == case and r[1] == comp)
    print(f"{case:30s} {comp:8s} vars_changed={n:3d}  max_rel_stat_change={w:.2e} ({var}.{s})")
ws = [r[3] for r in rows]
print(f"TOTAL changed variables={len(rows)}; stat change: zero-at-10-digits={sum(1 for w in ws if w == 0)}, max={max(ws):.2e}, median={sorted(ws)[len(ws)//2]:.2e}")

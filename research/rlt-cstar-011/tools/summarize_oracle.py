"""Tabulate oracle runs: per scenario, the pre-generation tile verdict and messages of
each tree, against what generation + configure_build does.
Usage: summarize_oracle.py NAME=oracle.jsonl ..."""
import json, re, sys
runs = {}
for arg in sys.argv[1:]:
    name, f = arg.split("=")
    runs[name] = {json.loads(l)["scenario"]: json.loads(l) for l in open(f)}
scen = sorted(next(iter(runs.values())))
print("| scenario | generation + configure_build | " + " | ".join(runs) + " |")
print("|---|---|" + "---|" * len(runs))
tot = {n: 0 for n in runs}
for s in scen:
    g = next(iter(runs.values()))[s].get("generated")
    cells = []
    for name, r in runs.items():
        x = r[s]; pre = x.get("pre_tile", [])
        sentences = sum(len(re.findall(r"param\.np_(?:xi|eta)=", p)) for p in pre)
        verdict = "refuse" if pre else "accept"
        gen = x.get("generated")
        exp = {"tile refusal": "refuse", "ok": "accept"}.get(gen)
        mark = "" if exp is None else (" ✓" if verdict == exp else " ✗")
        if exp is not None and verdict != exp: tot[name] += 1
        same = ""
        if pre and gen == "tile refusal":
            ok = len(pre) == 1 and x["generated_msg"].split(": ", 1)[1].strip() == pre[0].strip()
            same = ", same text" if ok else ", text differs"
            if not ok: tot[name] += 1
        cells.append(f"{verdict} ({len(pre)} msg, {sentences} sentences){mark}{same}")
    print(f"| {s} | {g} | " + " | ".join(cells) + " |")
print("\nwrong verdicts or messages: " + ", ".join(f"{n}: {v}" for n, v in tot.items()))

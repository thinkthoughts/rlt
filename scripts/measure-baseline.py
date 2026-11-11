#!/usr/bin/env python3
"""RLT-004a baseline: sizes of the current unary representation (untrusted).

usage: scripts/measure-baseline.py RLT_BINARY OUT_DIR > table.md

Measures, separately for each case: written notation size (characters of the
written statement), kernel term size (nodes of the numerals involved: the
numeral n is S^n(0), n + 1 nodes), proof steps, proof nodes (the file's node
budget, computed as syntax/src/proof.rs computes it), and checking time.
Proofs come only from existing mechanisms: `rlt gen` (equations, PEANO-ARITH),
the M3c non-divisibility scheme and the M3d enumeration scheme (ROBINSON-Q,
functions loaded verbatim from scripts/gen-m3c-tests.py and gen-m3d-tests.py).
No axiom is added and nothing trusted changes; a case that exceeds a current
limit is recorded with RLT's own refusal message.
"""
import os
import re
import subprocess
import sys
import time

rlt, out = sys.argv[1], sys.argv[2]
os.makedirs(out, exist_ok=True)
here = os.path.dirname(os.path.abspath(__file__))


def load(script, start, end):
    src = open(os.path.join(here, script)).read()
    ns = {"rlt": rlt, "re": re, "subprocess": subprocess, "os": os, "hashlib": __import__("hashlib")}
    exec(src[src.index(start):src.index(end)], ns)
    return ns


m3c = load("gen-m3c-tests.py", "def S(t, k):", "accept, reject, expected = {}, {}, []")
m3d = load("gen-m3d-tests.py", "def S(t, k):", "accept, reject, expected = {}, {}, []")


def budget(src):
    """Node budget as syntax/src/proof.rs computes it: an integer n counts n + 1."""
    total, i = 0, 0
    for tok in re.findall(r'"(?:[^"\\]|\\.)*"|[()]|[^\s()]+', re.sub(r";[^\n]*", "", src)):
        if tok == "(":
            total += 1
        elif tok == ")":
            continue
        elif re.fullmatch(r"\d+", tok):
            total += int(tok) + 1
        else:
            total += 1
    return total


def run(name, text):
    path = os.path.join(out, name + ".rlt")
    with open(path, "w") as f:
        f.write(text)
    t = time.perf_counter()
    r = subprocess.run([rlt, "check", path], capture_output=True, text=True)
    dt = time.perf_counter() - t
    steps = re.search(r"^steps\s+(\d+)", r.stdout, re.M)
    v = re.search(r"^verdict\s+(.*)$", r.stdout, re.M)
    return {"steps": steps.group(1) if steps else "—", "nodes": budget(text),
            "bytes": len(text.encode()), "time": f"{dt:.2f} s", "exit": r.returncode,
            "verdict": (v.group(1) if v else r.stdout.strip().splitlines()[-1])[:90]}


def is_prime(n):
    return n > 1 and all(n % d for d in range(2, int(n ** 0.5) + 1))


def goldbach(n):
    return next((p, n - p) for p in range(2, n) if is_prime(p) and is_prime(n - p))


print("## Written notation and kernel term size of numerals\n")
print("| n | written characters | kernel nodes (S^n(0)) |\n| --- | --- | --- |")
for n in (7, 30, 100, 1000, 10000):
    print(f"| {n} | {len(str(n))} | {n + 1} |")

print("\n## Equation certificates n = p + q (`rlt gen`, PEANO-ARITH)\n")
print("| statement | written chars | numeral nodes | steps | proof nodes | bytes | time | result |")
print("| --- | --- | --- | --- | --- | --- | --- | --- |")
for n in (10, 30, 100, 300, 400, 500, 1000, 10000):
    p, q = goldbach(n)
    stmt = f"{n} = {p} + {q}"
    g = subprocess.run([rlt, "gen", stmt, f"BASE.EQ{n}"], capture_output=True, text=True)
    if g.returncode != 0:
        print(f"| {stmt} | {len(stmt)} | {n + p + q + 3} | — | — | — | — | `rlt gen` exit {g.returncode}: {g.stderr.strip()} |")
        continue
    r = run(f"eq_{n}", g.stdout)
    print(f"| {stmt} | {len(stmt)} | {n + p + q + 3} | {r['steps']} | {r['nodes']:,} | {r['bytes']:,} | {r['time']} | exit {r['exit']}: {r['verdict']} |")

print("\n## Non-divisibility certificates ¬(d ∣ n) (M3c scheme, ROBINSON-Q)\n")
print("| statement | layers | steps | proof nodes | bytes | time | result |")
print("| --- | --- | --- | --- | --- | --- | --- |")
for d, n in ((2, 7), (2, 29), (3, 97), (2, 97), (2, 197), (2, 299), (2, 397), (31, 997), (7, 997), (2, 997)):
    r = run(f"nd_{d}_{n}", m3c["not_dvd"](f"BASE.ND{d}_{n}", d, n))
    print(f"| ¬({d} ∣ {n}) | {n // d + 1} | {r['steps']} | {r['nodes']:,} | {r['bytes']:,} | {r['time']} | exit {r['exit']}: {r['verdict']} |")

print("\n## Enumeration ∀x. x < B → x = 0 ∨ … ∨ x = B − 1 (M3d scheme, ROBINSON-Q)\n")
print("| bound B | steps | proof nodes | bytes | time | result |")
print("| --- | --- | --- | --- | --- | --- |")
for b in (10, 20, 30, 40, 47, 48, 60):
    r = run(f"enum_{b}", m3d["enum_file"](f"BASE.ENUM{b}", b, list(range(b))))
    print(f"| {b} | {r['steps']} | {r['nodes']:,} | {r['bytes']:,} | {r['time']} | exit {r['exit']}: {r['verdict']} |")

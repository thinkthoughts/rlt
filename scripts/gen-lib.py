#!/usr/bin/env python3
"""Propose the RLT-004c theorem library from the accepted proofs (untrusted).

usage: python3 scripts/gen-lib.py RLT_BINARY        (from the repository root)

Writes lib/<area>/<id>.yaml for every accepted proof of M3c, M3d, M3e and the
two other statement-record proofs, and lib/identities.tsv. Every value is
taken from `rlt deps FILE --format json`; `rlt deps --verify-lib` then checks
each entry against the proofs. Proofs of the same statement (equal
fingerprints) share one id: the first is the proof, the rest alternatives.

Run it only to create the library. Afterwards lib/identities.tsv is
append-only: an existing id keeps its fingerprint and first proof forever.
"""
import glob
import json
import os
import re
import subprocess
import sys

rlt = sys.argv[1]
FILES = (
    ["tests/accept/R_math29_pairing.rlt", "tests/accept/M1_conditional.rlt"]
    + sorted(glob.glob("tests/m3c/accept/*.rlt"))
    + sorted(glob.glob("tests/m3d/accept/*.rlt"))
    + sorted(glob.glob("tests/m3e/accept/*.rlt"))
)
NAMED = {
    "tests/accept/R_math29_pairing.rlt": "arith.add.29_1_30",
    "tests/accept/M1_conditional.rlt": "arith.conditional.2x3_7",
    "tests/m3d/accept/DA01_below_thirty.rlt": "order.below.30",
    "tests/m3d/accept/DA02_below_three.rlt": "order.below.3",
    "tests/m3d/accept/DA03_below_one.rlt": "order.below.1",
    "tests/m3d/accept/DA04_nothing_below_zero.rlt": "order.below.0",
    "tests/m3d/accept/DA05_seven_from_the_enumeration.rlt": "order.below30_at.7",
    "tests/m3e/accept/EA_07.rlt": "mod30.forward_at.7",
    "tests/m3e/accept/E_FWD.rlt": "mod30.forward",
    "tests/m3e/accept/E_BWD.rlt": "mod30.backward",
    "tests/m3e/accept/E_T1.rlt": "mod30.classification",
}
PATTERNS = [
    (r"^(\d+) ∣ (\d+)$", "dvd.{0}_{1}"),
    (r"^¬\((\d+) ∣ (\d+)\)$", "dvd.not.{0}_{1}"),
    (r"^free235\((\d+)\)$", "dvd.free235.{0}"),
    (r"^∀k\. ¬\((\d+) = (\d+) × k\)$", "dvd.witnesswise.{1}_{0}"),
]


def name(path, written):
    if path in NAMED:
        return NAMED[path]
    for pat, fmt in PATTERNS:
        m = re.match(pat, written)
        if m:
            return fmt.format(*m.groups())
    sys.exit(f"no library id for {path} ({written})")


def q(s):
    return json.dumps(s, ensure_ascii=False)


nodes = {}
for f in FILES:
    d = json.loads(subprocess.run([rlt, "deps", f, "--format", "json"], capture_output=True, text=True).stdout)
    n = d["root"]
    assert n["verdict"] == "ACCEPT", f
    nodes[f] = n

# Group proofs by statement fingerprint, in file order.
groups = {}
for f in FILES:
    groups.setdefault(nodes[f]["fingerprint"], []).append(f)
by_pin = {}
entries = []
for fp, files in groups.items():
    first = nodes[files[0]]
    lid = name(files[0], first["written_goal"].strip('"'))
    for f in files:
        by_pin[(f, nodes[f]["sha256"])] = lid
    entries.append((lid, fp, files))

records = {}
for r in sorted(glob.glob("statements/*.yaml")):
    m = re.search(r"^  proof: (\S+)$", open(r, encoding="utf-8").read(), re.M)
    records.setdefault(m.group(1), []).append(r)

for d in glob.glob("lib/*/"):
    for x in glob.glob(d + "*.yaml"):
        os.remove(x)
ids = []
for lid, fp, files in sorted(entries):
    n = nodes[files[0]]
    uses = [by_pin[(c["path"], c["sha256"])] for c in n["children"]]
    pairs = n["assumption_pairs"] or []
    area = lid.split(".")[0]
    os.makedirs(f"lib/{area}", exist_ok=True)
    out = [
        "format: rlt-lib/1",
        f"id: {lid}",
        f"title: {q(n['written_goal'].strip(chr(34)))}",
        f"fingerprint: {fp}",
        f"proof: {{path: {files[0]}, sha256: {n['sha256']}}}",
        f"proof_id: {n['proof_id']}",
    ]
    if len(files) > 1:
        out.append("alternatives:")
        out += [f"  - {{path: {f}, sha256: {nodes[f]['sha256']}}}" for f in files[1:]]
    out += [
        f"axiom_set: {n['axiom_set']}",
        f"convention: {n['convention']}",
        f"checked_goal: {q(n['checked_goal'])}",
        "assumptions: []" if not pairs else "assumptions:",
    ]
    out += [f"  - {{name: {p['name']}, formula: {q(p['formula'])}}}" for p in pairs]
    out.append(f"citable: {'true' if not pairs else 'false'}")
    out.append("uses: []" if not uses else "uses:")
    out += [f"  - {u}" for u in uses]
    recs = [r for f in files for r in records.get(f, [])]
    out.append("statement_records: []" if not recs else "statement_records:")
    out += [f"  - {r}" for r in recs]
    out.append("history: []")
    with open(f"lib/{area}/{lid}.yaml", "w", encoding="utf-8") as fh:
        fh.write("\n".join(out) + "\n")
    ids.append(f"{lid}\t{fp}\t{n['sha256']}")
with open("lib/identities.tsv", "w", encoding="utf-8") as fh:
    fh.write("# RLT-004c identity registry: library id, statement fingerprint, first proof sha256.\n")
    fh.write("# Append-only: scripts/verify.sh refuses any change to an existing line.\n")
    fh.write("\n".join(ids) + "\n")
print(f"{len(entries)} library entries for {len(FILES)} proofs")

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
    + sorted(glob.glob("tests/rlt005/accept/*.rlt"))
    + sorted(glob.glob("tests/rlt006/accept/*.rlt"))
    + sorted(glob.glob("tests/rlt007s1/accept/*.rlt"))
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
    # RLT-005: McNew-Pomerance, 8 is not matchable.
    "tests/rlt005/accept/B5_below_five.rlt": "order.below.5",
    "tests/rlt005/accept/MD8_divisors_of_eight.rlt": "matchable.divisors.8",
    "tests/rlt005/accept/MP4_partners_one_to_four.rlt": "matchable.partners.4",
    "tests/rlt005/accept/MO2_partner_of_2_is_odd.rlt": "matchable.odd_partner.2",
    "tests/rlt005/accept/MO4_partner_of_4_is_odd.rlt": "matchable.odd_partner.4",
    "tests/rlt005/accept/MO8_partner_of_8_is_odd.rlt": "matchable.odd_partner.8",
    "tests/rlt005/accept/MPH_three_values_in_one_three.rlt": "matchable.pigeonhole.1_3",
    "tests/rlt005/accept/MC_no_assignment.rlt": "matchable.not.8.contradiction",
    "tests/rlt005/accept/MCO_one_is_coprime_to_all.rlt": "matchable.coprime_one",
    "tests/rlt005/accept/MNV_conditions_without_distinctness.rlt": "matchable.not.8.nonvacuous",
    "tests/rlt005/accept/MN8_eight_is_not_matchable.rlt": "matchable.not.8",
    "tests/rlt005/accept/MNC_two_not_coprime_to_two.rlt": "matchable.not_coprime.2_2",
    # RLT-007b stage 1: obligation (a) of Lemma 12.1.
    "tests/rlt007s1/accept/DG_0.rlt": "admissible.h45.distinct.g0",
    "tests/rlt007s1/accept/DG_1.rlt": "admissible.h45.distinct.g1",
    "tests/rlt007s1/accept/DG_2.rlt": "admissible.h45.distinct.g2",
    "tests/rlt007s1/accept/DG_3.rlt": "admissible.h45.distinct.g3",
    "tests/rlt007s1/accept/BOUNDS.rlt": "admissible.h45.bounds",
}
PATTERNS = [
    (r"^(\d+) ∣ (\d+)$", "dvd.{0}_{1}"),
    (r"^¬\((\d+) ∣ (\d+)\)$", "dvd.not.{0}_{1}"),
    (r"^free235\((\d+)\)$", "dvd.free235.{0}"),
    (r"^∀k\. ¬\((\d+) = (\d+) × k\)$", "dvd.witnesswise.{1}_{0}"),
    # RLT-006: the 14 per-prime theorems (left-nested conjunction of the 45
    # omissions ¬(∃k. h = (p × k) + r) over H45, first element 0).
    (r"^\(+¬\(∃k\. 0 = \((\d+) × k\) \+ (\d+)\).* ∧ ¬\(∃k\. 212 = \(\d+ × k\) \+ \d+\)\)$", "admissible.h45.omits.{0}_{1}"),
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

# Append-only identities, checked before anything is written (RLT-005): an
# existing id whose fingerprint or first proof would change is an error.
old = open("lib/identities.tsv", encoding="utf-8").read().splitlines() if os.path.exists("lib/identities.tsv") else [
    "# RLT-004c identity registry: library id, statement fingerprint, first proof sha256.",
    "# Append-only: scripts/verify.sh refuses any change to an existing line.",
]
known = {l.split("\t")[0]: l for l in old if l and not l.startswith("#")}
for lid, fp, files in entries:
    line = f"{lid}\t{fp}\t{nodes[files[0]]['sha256']}"
    if lid in known and known[lid] != line:
        sys.exit(f"identity of {lid} would change: {known[lid]!r} -> {line!r}")

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
    # A cited proof outside the library (RLT-006: the generated element
    # certificates) is named `(unlisted PATH)`; `rlt deps --verify-lib`
    # re-checks it through the kernel like any other citation.
    # Only generated certificates may be unlisted (RLT-006); any other cited
    # proof outside the library is an error, as before.
    uses = []
    for c in n["children"]:
        if (c["path"], c["sha256"]) in by_pin:
            uses.append(by_pin[(c["path"], c["sha256"])])
        elif c["path"].startswith(("tests/rlt006/gen/", "tests/rlt007s1/gen/")):
            uses.append(f"(unlisted {c['path']})")
        else:
            sys.exit(f"{files[0]} cites {c['path']}, which is neither in the library nor a generated certificate")
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
# Existing lines stay in place; new ids are appended.
new = [line for line in ids if line.split("\t")[0] not in known]
with open("lib/identities.tsv", "w", encoding="utf-8") as fh:
    fh.write("\n".join(old + new) + "\n")
print(f"{len(entries)} library entries for {len(FILES)} proofs")

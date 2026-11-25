#!/usr/bin/env python3
"""RLT-QEC-002a acceptance: mask-set equality between the independently
computed census and the pinned source (docs/RLT-QEC-001.md §7.1).

This is the only RLT-QEC-002a program that reads source data, and it runs
after the enumeration; census.py and check.py read none.

The source file is read with scripts/pinned_read.py, which re-hashes every
Git object from the pinned commit to the file, and its SHA-256 must equal
the pin below. The list `allBaseCycles10` is parsed from it; its entries of
weight < 8 (the registered expectation for Z_7) are compared as a set with
the RLT list. Counts and weight distributions are printed as diagnostics
only; acceptance is set equality.

Usage: compare.py BB_DISTANCE_LEAN_CHECKOUT RLT_LIST [--wmax W]
Exit 0 EQUAL, 1 DIFFERENT or a pin/parse failure.
"""
import hashlib
import os
import re
import subprocess
import sys

COMMIT = "a83cafad4d041427af4ddb54085493d3f9dd02d2"   # bb-distance-lean pin
PATH = "BBDistance/Bravyi360/BaseCycleCensus.lean"
SHA256 = "838c7f4a93e65491738c903e2c3961777ec652f8dab8a2c8c997bc39555924cb"
EXPECTED_LENGTH = 1954                                 # allBaseCycles10_length
PINNED_READ = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                           "..", "..", "scripts", "pinned_read.py")


def source_list(checkout):
    data = subprocess.run([sys.executable, "-I", PINNED_READ, checkout, COMMIT, PATH],
                          check=True, capture_output=True).stdout
    digest = hashlib.sha256(data).hexdigest()
    if digest != SHA256:
        raise SystemExit(f"source SHA-256 {digest} differs from the pin {SHA256}")
    text = data.decode("utf-8")
    m = re.search(r"noncomputable def allBaseCycles10 : List Nat := \[(.*?)\]", text, re.S)
    if not m:
        raise SystemExit("allBaseCycles10 not found")
    items = [s.strip() for s in m.group(1).split(",")]
    if not all(re.fullmatch(r"[0-9]+", s) for s in items):
        raise SystemExit("allBaseCycles10 has a non-numeral entry")
    values = [int(s) for s in items]
    if len(values) != EXPECTED_LENGTH:
        raise SystemExit(f"allBaseCycles10 has {len(values)} entries, expected {EXPECTED_LENGTH}")
    return values


def dist(masks):
    d = {}
    for u in masks:
        w = bin(u).count("1")
        d[w] = d.get(w, 0) + 1
    return dict(sorted(d.items()))


def main(argv):
    args = list(argv)
    wmax = 7
    if "--wmax" in args:
        i = args.index("--wmax")
        wmax = int(args[i + 1])
        del args[i:i + 2]
    if len(args) != 2:
        print(__doc__, file=sys.stderr)
        return 2
    src = source_list(args[0])
    src_low = {u for u in src if bin(u).count("1") <= wmax}
    with open(args[1]) as f:
        rlt = [int(s) for s in f.read().split()]
    rlt_set = set(rlt)
    print(f"source  {PATH} @ {COMMIT[:7]} (sha256 {SHA256[:12]}…): "
          f"{len(src)} entries; {len(src_low)} of weight <= {wmax}, weights {dist(src_low)}")
    print(f"rlt     {args[1]}: {len(rlt)} masks, weights {dist(rlt)}  [diagnostic]")
    only_rlt, only_src = sorted(rlt_set - src_low), sorted(src_low - rlt_set)
    if len(rlt) == len(rlt_set) and not only_rlt and not only_src:
        print(f"EQUAL   mask sets coincide ({len(rlt_set)} masks)")
        return 0
    print(f"DIFFERENT  only in RLT: {len(only_rlt)}  only in source: {len(only_src)}  "
          f"duplicates in RLT list: {len(rlt) - len(rlt_set)}")
    for u in only_rlt[:5]:
        print(f"  only RLT    {u}")
    for u in only_src[:5]:
        print(f"  only source {u}")
    return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

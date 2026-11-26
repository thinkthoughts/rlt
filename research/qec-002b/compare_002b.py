#!/usr/bin/env python3
"""RLT-QEC-002b source correspondence (docs/RLT-QEC-002b.md §6.1). The only
002b program that reads source data; it runs after construction and checking.

Reads BBDistance/Bravyi360/BaseCycleCensus.lean at the pinned commit through
scripts/pinned_read.py (Git object chain re-hashed, file SHA-256 pinned) and
evaluates d2 with check_witness.py's convolution implementation (never with
the builder). Three reports, kept separate:

  A  indexed boundary-column correspondence: baseBoundaryWindow entry i
     equals d2 delta_g for face(g) = i, for i = 0 .. 44, each index once;
  B  source-provided witnesses: the first components of baseFloorPairs equal
     the census Z7.txt as a set, and d2 c_src = u for every pair (the RLT
     checker's condition 6 applied to the source certificate);
  C  diagnostic only: per census mask, whether the source coefficient equals
     the RLT coefficient of W.txt, or differs from it by an element of ker d2.
     Equality of coefficient vectors is not an acceptance criterion.

Usage: compare_002b.py BB_DISTANCE_LEAN_CHECKOUT W.txt Z7.txt
Exit 0 when A and B hold; 1 otherwise. Requires Python 3.8 or later.
"""
import hashlib
import importlib.util
import os
import re
import subprocess
import sys

if sys.version_info < (3, 8):
    sys.exit("compare_002b.py requires Python 3.8 or later")
sys.dont_write_bytecode = True

COMMIT = "a83cafad4d041427af4ddb54085493d3f9dd02d2"   # bb-distance-lean pin
PATH = "BBDistance/Bravyi360/BaseCycleCensus.lean"
SHA256 = "838c7f4a93e65491738c903e2c3961777ec652f8dab8a2c8c997bc39555924cb"
HERE = os.path.dirname(os.path.abspath(__file__))
PINNED_READ = os.path.join(HERE, "..", "..", "scripts", "pinned_read.py")

spec = importlib.util.spec_from_file_location("check_witness", os.path.join(HERE, "check_witness.py"))
checker = importlib.util.module_from_spec(spec)
spec.loader.exec_module(checker)


def source_text(checkout):
    data = subprocess.run([sys.executable, "-I", PINNED_READ, checkout, COMMIT, PATH],
                          check=True, capture_output=True).stdout
    if hashlib.sha256(data).hexdigest() != SHA256:
        raise SystemExit("source SHA-256 differs from the pin")
    return data.decode("utf-8")


def pair_list(text, name, length):
    m = re.search(r"noncomputable def " + name + r" : List \(Nat × Nat\) := \[(.*?)\]", text, re.S)
    if not m:
        raise SystemExit(f"{name} not found")
    body = m.group(1)
    pairs = [(int(a), int(b)) for a, b in re.findall(r"\(\s*([0-9]+)\s*,\s*([0-9]+)\s*\)", body)]
    leftover = re.sub(r"\(\s*[0-9]+\s*,\s*[0-9]+\s*\)", "", body)
    if leftover.replace(",", "").strip() or len(pairs) != length:
        raise SystemExit(f"{name}: expected exactly {length} numeral pairs")
    return pairs


def main(argv):
    if len(argv) != 3:
        print(__doc__, file=sys.stderr)
        return 2
    checkout, w_path, z7_path = argv
    text = source_text(checkout)
    window = pair_list(text, "baseBoundaryWindow", 45)
    floor = pair_list(text, "baseFloorPairs", 46)
    with open(z7_path) as f:
        census = [int(s) for s in f.read().split()]
    with open(w_path) as f:
        rlt = dict(tuple(map(int, line.split())) for line in f if line.strip())
    print(f"source  {PATH} @ {COMMIT[:7]} (sha256 {SHA256[:12]}…): "
          f"baseBoundaryWindow {len(window)} pairs, baseFloorPairs {len(floor)} pairs")

    columns = [checker.d2(1 << i) for i in range(45)]
    idx = [i for i, _ in window]
    a_ok = sorted(idx) == list(range(45)) and all(mask == columns[i] for i, mask in window)
    print(f"A  indexed columns: {'EQUAL' if a_ok else 'DIFFERENT'} "
          f"(window entry i = d2 delta_g with face(g) = i, for all 45 indices)")

    src_u = [u for u, _ in floor]
    b_set = len(src_u) == len(set(src_u)) and set(src_u) == set(census)
    b_wit = all(checker.d2(c) == u for u, c in floor) and all(c < 1 << 45 for _, c in floor)
    b_ok = b_set and b_wit
    print(f"B  source witnesses: census set {'EQUAL' if b_set else 'DIFFERENT'}; "
          f"d2 c_src = u for {sum(checker.d2(c) == u for u, c in floor)}/{len(floor)} pairs "
          f"-> {'VERIFIED' if b_ok else 'FAILED'}")

    same = sum(1 for u, c in floor if rlt.get(u) == c)
    viakernel = sum(1 for u, c in floor if u in rlt and rlt[u] != c and checker.d2(rlt[u] ^ c) == 0)
    print(f"C  diagnostic: source coefficient = RLT coefficient for {same} masks; "
          f"differs by an element of ker d2 for {viakernel}; other {len(floor) - same - viakernel}")
    return 0 if a_ok and b_ok else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

#!/usr/bin/env python3
"""RLT-PROVER-002: bind lean/ProverDefinitions.lean to what RLT checked.

For every goal tests/definitions/goals/FN_*.goal, `rlt check` of its
committed certificate prints the folded statement (`folded`) and the
unfolded statement the kernel checked (`checked goal`). This script rewrites
both mechanically into Lean syntax and requires that theorem FN_unfolds in
the Lean file states exactly `(folded) ↔ (unfolded)`, and that FN_holds
states the folded statement. Lean then checks the equivalence by Iff.rfl,
an independent implementation of the unfolding RLT's elaborator performs.

usage: scripts/check-definitions-lean.py RLT_BINARY
"""
import glob
import os
import re
import subprocess
import sys


def lean_of(s: str, typed: bool) -> str:
    # Function applications f(t) → f (t), with simple atoms unparenthesised.
    prev = None
    while prev != s:
        prev = s
        s = re.sub(r"\b([a-z]{2,})\(([^()]*)\)",
                   lambda m: f"{m.group(1)} " + (m.group(2) if re.fullmatch(r"[a-z0-9]+", m.group(2)) else f"({m.group(2)})"),
                   s)
    s = s.replace("×", "*").replace("¬(", "¬ (")
    # ∀x. BODY → ∀ x, BODY  (∀ x : Nat, on the unfolded side)
    s = re.sub(r"∀([a-z])\. ", (lambda m: f"∀ {m.group(1)} : Nat, ") if typed else (lambda m: f"∀ {m.group(1)}, "), s)
    return s


def main() -> int:
    rlt = sys.argv[1]
    lean = open("lean/ProverDefinitions.lean", encoding="utf-8").read()
    lean_flat = re.sub(r"\s+", " ", lean)
    bad, n = 0, 0
    for g in sorted(glob.glob("tests/definitions/goals/F*_*.goal")):
        name = os.path.basename(g)[:-5]
        tag = name.split("_")[0]
        cert = f"tests/definitions/certificates/{name}.rlt"
        out = subprocess.run([rlt, "check", cert], capture_output=True, text=True).stdout
        folded = re.search(r"^  folded        (.*)$", out, re.M)
        checked = re.search(r"^checked goal    (.*)$", out, re.M)
        if not folded or not checked or "verdict         ACCEPT" not in out:
            print(f"  UNEXPECTED: {cert} did not replay with a folded and checked goal")
            bad += 1
            continue
        f, u = folded.group(1), checked.group(1)
        # RLT prints S(x) as x + 1 already; strip RLT's outer parentheses of
        # ∀x. (A → B) only where Lean would need none (none occur here).
        want_unf = f"theorem {tag}_unfolds : ({lean_of(f, False)}) ↔ ({lean_of(u, True)})"
        want_hold = f"theorem {tag}_holds : {lean_of(f, False)}"
        n += 1
        for w in (want_unf, want_hold):
            if w not in lean_flat:
                print(f"  UNEXPECTED: lean/ProverDefinitions.lean lacks `{w}`")
                bad += 1
    expected = len(glob.glob("tests/definitions/goals/F*_*.goal"))
    if n != expected or n == 0:
        bad += 1
    print(f"lean/ProverDefinitions.lean: {n - min(n, bad)} of {expected} goals stated as RLT folded and unfolded them (statement correspondence, mechanical)")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""RLT-PROVER-002: bind lean/ProverDefinitions.lean to what RLT checked.

What this script checks (textual, mechanical):

1. For every goal tests/definitions/goals/FN_*.goal, `rlt check` of its
   committed certificate prints the folded statement (`folded`) and the
   unfolded statement the kernel checked (`checked goal`). Both are rewritten
   into Lean syntax, and the Lean file must contain exactly
   `theorem FN_unfolds : (folded) ↔ (unfolded)` and
   `theorem FN_holds : folded`.
2. Each Lean `def` of a registered function is the registered written form
   of definitions/double.rlt or definitions/quadruple.rlt, rewritten the same
   way.

It is this text comparison, not Lean, that pins the Lean statements to RLT's
particular unfolding. Lean (run separately) then checks the hand-written
theorems: `FN_unfolds` by Iff.rfl, i.e. up to definitional equality, which
shows the folded and unfolded statements are equivalent in Lean's ℕ for
these seven goals; Iff.rfl would equally accept some other equivalent
unfolding (for example `(double 2 = 4) ↔ (3 + 1 = 4)`), so on its own it
does not validate RLT's unfolding algorithm. Nothing here is a general proof
that RLT's elaborator agrees with Lean's semantics.

usage: scripts/check-definitions-lean.py RLT_BINARY [--lean PATH]
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
    args = sys.argv[1:]
    lean_path = "lean/ProverDefinitions.lean"
    if "--lean" in args:
        i = args.index("--lean")
        lean_path = args[i + 1]
        del args[i:i + 2]
    rlt = args[0]
    lean = open(lean_path, encoding="utf-8").read()
    lean_flat = re.sub(r"\s+", " ", lean)
    bad, n = 0, 0
    # 2. Lean's defs are the registered written forms.
    for reg in ("definitions/double.rlt", "definitions/quadruple.rlt"):
        text = open(reg, encoding="utf-8").read()
        for m in re.finditer(r'\(written "([a-z]{2,})\(([a-z](?:, [a-z])*)\) := ([^"]*)"\)', text):
            fname, params, rhs = m.group(1), m.group(2).split(", "), m.group(3)
            binders = " ".join(f"({p} : Nat)" for p in params)
            want_def = f"def {fname} {binders} : Nat := {lean_of(rhs, False)}"
            if want_def not in lean_flat:
                print(f"  UNEXPECTED: {lean_path} lacks `{want_def}` (registered in {reg})")
                bad += 1
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
                print(f"  UNEXPECTED: {lean_path} lacks `{w}`")
                bad += 1
    expected = len(glob.glob("tests/definitions/goals/F*_*.goal"))
    if n != expected or n == 0:
        bad += 1
    print(f"{lean_path}: {n - min(n, bad)} of {expected} goals stated as RLT folded and unfolded them, Lean defs equal to the registered written forms (textual correspondence, mechanical; Lean's Iff.rfl checks equivalence up to definitional equality, not RLT's unfolding algorithm)")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())

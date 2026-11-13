#!/usr/bin/env python3
"""RLT-007b stage 1 generator (untrusted): obligation (a) of Lemma 12.1 of Axiom
Math's draft "A New Bound for Small Gaps Between Primes" (September 3, 2026):
the 45 listed elements of H45 are pairwise distinct, and each lies between 0
and 212. Deterministic: the same inputs give the same bytes.

Inputs: sources/transcriptions/AXIOM-BGP212-2026-09-03.yaml (the reviewed
transcription: the tuple and the Lemma 12.1 text) and, for its SHA-256 pin in
the statement records, docs/RLT-007b-1-CORRESPONDENCE.md. RLT_BINARY is
accepted for symmetry with the other generators and is not run. Before writing anything the generator
performs the SOURCE TRANSCRIPTION CHECKS (not kernel-checked): the tuple has
exactly 45 entries, is strictly increasing, starts at 0 and ends at 212, and
the transcribed lemma text names "45-tuple" and "diameter 212".

Usage: gen-rlt007s1.py RLT_BINARY [--materialize-only | --check | --self-test]

Outputs (the kernel checks every proof; nothing here is trusted):
  tests/rlt007s1/gen/D_HHH.rlt      44 per-element distinctness theorems
                                    ⋀_{j>i} ¬(h_i = h_j); GENERATED, not
                                    committed (.gitignore), pinned by the manifest
                                    and by the group summaries' citations
  tests/rlt007s1/manifest.sha256    SHA-256 of the 44 files (sha256sum -c format)
  tests/rlt007s1/accept/DG_K.rlt    4 group summaries (K = 0..3), each citing 11
  tests/rlt007s1/accept/BOUNDS.rlt  ⋀_h ((∃z. z + 0 = h) ∧ (∃z. z + h = 212))
  tests/rlt007s1/reject/*.rlt       rejected cases; tests/rlt007s1/expected.tsv
  statements/H45-DISTINCT-GK.yaml, statements/H45-BOUNDS.yaml   records
  lean/RLT007S1.lean                Lean comparison (the RLT goals, literally)

--materialize-only writes only tests/rlt007s1/gen/ (used before checking).
--check writes nothing: it regenerates every output in memory and compares
it with the files on disk (exit 1 on any difference).
--self-test runs the transcription checks on altered tuples (each must be
refused) and exits.
"""
import hashlib
import json
import os
import re
import subprocess
import sys

import yaml

rlt = sys.argv[1] if len(sys.argv) > 1 else ""
MODE = sys.argv[2] if len(sys.argv) > 2 else ""
assert MODE in ("", "--materialize-only", "--check", "--self-test"), MODE
root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
os.chdir(root)

TRANSCRIPTION = "sources/transcriptions/AXIOM-BGP212-2026-09-03.yaml"
T = yaml.safe_load(open(TRANSCRIPTION, encoding="utf-8"))
items = {it["id"]: it for it in T["items"]}


def transcription_checks(H, lemma_text):
    """The source-derived listing facts. Returns a list of refusals."""
    bad = []
    if not all(isinstance(h, int) and not isinstance(h, bool) and h >= 0 for h in H):
        bad.append("an entry is not a natural number")
        return bad
    if len(H) != 45:
        bad.append(f"the tuple has {len(H)} entries, not 45")
    if any(a >= b for a, b in zip(H, H[1:])):
        bad.append("the tuple is not strictly increasing (a repeated or out-of-order entry)")
    if not H or H[0] != 0:
        bad.append("the first entry is not 0")
    if not H or H[-1] != 212:
        bad.append("the last entry is not 212")
    # Audit: whole numbers only (not "145-tuple" or "diameter 2120"); the
    # caller passes both the PDF-checked `extracted` field and the reviewed
    # `text` field, and both must name a 45-tuple of diameter 212.
    for t in lemma_text if isinstance(lemma_text, (list, tuple)) else [lemma_text]:
        if not (re.search(r"(?<![0-9])45-tuple(?![0-9A-Za-z])", t) and re.search(r"diameter 212(?![0-9])", t)):
            bad.append("the transcribed Lemma 12.1 does not name a 45-tuple of diameter 212")
            break
    return bad


H = items["h45-tuple"]["numbers"]
# `extracted` is compared with the PDF page by scripts/check_excerpts.py;
# `text` is the reviewed transcription. Both are checked.
LEMMA = (items["lemma-12-1"]["extracted"], items["lemma-12-1"]["text"])

if MODE == "--self-test":
    cases = [
        ("a duplicated element", H[:10] + [H[9]] + H[11:]),
        ("an out-of-order element", H[:5] + [H[6], H[5]] + H[7:]),
        ("213 in place of 212", H[:-1] + [213]),
        ("44 entries", H[:-1]),
        ("46 entries", H + [214]),
        ("46 strictly increasing entries from 0 to 212 (1 inserted)", [0, 1] + H[1:]),
        ("1 in place of 0", [1] + H[1:]),
        ("a negative entry", [-2] + H[1:]),
    ]
    n = 0
    for name, alt in cases:
        r = transcription_checks(alt, LEMMA)
        print(f"{'refused ' if r else 'ACCEPTED'} {name}: {'; '.join(r)}")
        n += 1 if r else 0
    texts = [("a lemma text naming a 44-tuple", (LEMMA[0], LEMMA[1].replace("45-tuple", "44-tuple"))),
             ("an extracted lemma text naming a 145-tuple", (LEMMA[0].replace("45-tuple", "145-tuple"), LEMMA[1])),
             ("a lemma text naming diameter 2120", (LEMMA[0], LEMMA[1].replace("diameter 212", "diameter 2120")))]
    for name, lt in texts:
        r = transcription_checks(H, lt)
        print(f"{'refused ' if r else 'ACCEPTED'} {name}")
        n += 1 if r else 0
    total = len(cases) + len(texts)
    print(f"generator self-test: {n} of {total} altered transcriptions refused")
    sys.exit(0 if n == total else 1)

refusals = transcription_checks(H, LEMMA)
if refusals:
    sys.exit("transcription check failed: " + "; ".join(refusals))

GEN, ACC, REJ = "tests/rlt007s1/gen", "tests/rlt007s1/accept", "tests/rlt007s1/reject"
GROUPS = [list(range(0, 11)), list(range(11, 22)), list(range(22, 33)), list(range(33, 44))]
OUT = {}  # path -> text (everything this run produces)


def S(t, k):
    for _ in range(k):
        t = f"(S {t})"
    return t


def sha(text):
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def proof(pid, comment, goal, steps, params=(), cites=()):
    c = ""
    if cites:
        c = "  (cites\n" + "\n".join(f'    ({l} "{pa}" "{hs}")' for l, pa, hs in cites) + ")\n"
    pr = f"  (params {' '.join(params)})\n" if params else ""
    body = "\n".join("    " + s for s in steps)
    return (f"; {comment}\n(rlt-proof\n  (id RLT007S1.{pid})\n  (convention NAT-DEF)\n  (goal \"{goal}\")\n"
            f"  (mode unconditional)\n  (axioms ROBINSON-Q)\n{c}{pr}  (steps\n{body}))\n")


def conj(parts):
    """Left-nested conjunction by and-intro: parts = [(sexp, written, label)]."""
    f, w, l = parts[0]
    st = []
    for k, (g, wg, lg) in enumerate(parts[1:], 1):
        f, w = f"(and {f} {g})", f"({w} ∧ {wg})"
        st.append(f"(C{k} {f} (and-intro {l} {lg}))")
        l = f"C{k}"
    return st, f, w, l


def kchain(a, drop=None):
    """K_j :≡ ∀u. ¬(j = S^{j+1}(u)) for j = 0..a, each from the previous one
    (Q2 for j = 0; Q1 then K_{j-1} for j > 0). `drop` omits K_j's last step
    (a rejected case)."""
    st = ["(K0 (not (= 0 (S u0))) (inst q2_zero_ne_succ (x u0)))",
          "(KA0 (forall v (not (= 0 (S v)))) (forall-intro u0 K0))"]
    ps = ["u0"]
    for j in range(1, a + 1):
        u = f"u{j}"
        ps.append(u)
        st += [f"(KH{j} (= {j} {S(u, j + 1)}) (hyp kh{j}))",
               f"(KQ{j} (imp (= (S {j - 1}) (S {S(u, j)})) (= {j - 1} {S(u, j)})) (inst q1_succ_inj (x {j - 1}) (y {S(u, j)})))",
               f"(KP{j} (= {j - 1} {S(u, j)}) (imp-elim KQ{j} KH{j}))",
               f"(KE{j} (not (= {j - 1} {S(u, j)})) (forall-elim KA{j - 1} {u}))",
               f"(KF{j} false (imp-elim KE{j} KP{j}))",
               f"(KN{j} (not (= {j} {S(u, j + 1)})) (imp-intro kh{j} KF{j}))"]
        if drop != j:
            st.append(f"(KA{j} (forall v (not (= {j} {S('v', j + 1)}))) (forall-intro {u} KN{j}))")
    return st, ps


def written_ne(a, b):
    return f"¬({a} = {b})"


def distinct_file(i, note=None, wrong=None, drop=None):
    """Per-element theorem for a = H[i]. `wrong` = (j, claimed b, instance n)
    replaces one pair step (rejected cases); `drop` omits one K step."""
    a = H[i]
    st, ps = kchain(a, drop)
    parts = []
    for j in range(i + 1, 45):
        b, n = H[j], H[j] - a - 1
        if wrong and wrong[0] == j:
            b, n = wrong[1], wrong[2]
        st.append(f"(D{j} (not (= {a} {b})) (forall-elim KA{a} {n}))")
        parts.append((f"(not (= {a} {b}))", written_ne(a, b), f"D{j}"))
    if len(parts) == 1:
        w = parts[0][1]
        st[-1] = st[-1].replace(f"({parts[0][2]} ", "(G ", 1)
    else:
        cs, _, w, l = conj(parts)
        st += cs
        st[-1] = st[-1].replace(f"({l} ", "(G ", 1)
    comment = note or (f"RLT-007b stage 1: the element {a} of H45 differs from every later element. "
                       f"Generated by scripts/gen-rlt007s1.py (not committed); pinned by tests/rlt007s1/manifest.sha256")
    return proof(f"D_{a:03d}", comment, w, st, ps)


def gen_path(i):
    return f"{GEN}/D_{H[i]:03d}.rlt"


def goal_of(text):
    return re.search(r'\(goal "([^"]*)"\)', text).group(1)


def goal_sexp(text):
    """The formula of the step labelled G (the file's conclusion)."""
    for line in text.splitlines():
        line = line.strip()
        if line.startswith("(G "):
            depth, k = 0, 3
            for k in range(3, len(line)):
                depth += line[k] == "("
                depth -= line[k] == ")"
                if depth == 0:
                    return line[3:k + 1]
    raise ValueError("no G step")


def group_file(k, swap=None, stale=None, missing=None):
    cites, st, parts = [], [], []
    for n, i in enumerate(GROUPS[k]):
        path, text = gen_path(i), OUT[gen_path(i)]
        if swap and swap[0] == i:
            path, text = gen_path(swap[1]), OUT[gen_path(swap[1])]
        h = sha(text)
        if stale == i:
            h = "f" * 64
        if missing == i:
            path = f"{GEN}/D_999.rlt"
        cites.append((f"E{n}", path, h))
        g = goal_sexp(OUT[gen_path(i)])
        st.append(f"(F{n} {g} (cite E{n}))")
        parts.append((g, goal_of(OUT[gen_path(i)]), f"F{n}"))
    cs, _, w, l = conj(parts)
    st += cs
    st[-1] = st[-1].replace(f"({l} ", "(G ", 1)
    lo, hi = H[GROUPS[k][0]], H[GROUPS[k][-1]]
    return cites, st, w, lo, hi


def bounds_file(off_by_one=None):
    st = ["(E0 (= (+ w 0) w) (inst add_zero (x w)))"]
    for j in range(1, 213):
        st += [f"(EA{j} (= (+ w {j}) (S (+ w {j - 1}))) (inst add_succ (x w) (y {j - 1})))",
               f"(EC{j} (= (S (+ w {j - 1})) {S('w', j)}) (cong S {'E0' if j == 1 else f'E{j - 1}'}))",
               f"(E{j} (= (+ w {j}) {S('w', j)}) (trans EA{j} EC{j}))"]
    parts = []
    for h in H:
        z = 212 - h - (1 if off_by_one == h else 0)
        top = 212 - (1 if off_by_one == h else 0)
        st += [f"(Lo{h} (= (+ {h} 0) {h}) (inst add_zero (x {h})))",
               f"(LoE{h} (exists z (= (+ z 0) {h})) (exists-intro {h} Lo{h}))",
               f"(Up{h} (forall v (= (+ v {h}) {S('v', h)})) (forall-intro w E{h}))",
               f"(UpI{h} (= (+ {z} {h}) 212) (forall-elim Up{h} {z}))" if off_by_one == h else
               f"(UpI{h} (= (+ {z} {h}) {top}) (forall-elim Up{h} {z}))",
               f"(UpE{h} (exists z (= (+ z {h}) 212)) (exists-intro {z} UpI{h}))",
               f"(B{h} (and (exists z (= (+ z 0) {h})) (exists z (= (+ z {h}) 212))) (and-intro LoE{h} UpE{h}))"]
        parts.append((f"(and (exists z (= (+ z 0) {h})) (exists z (= (+ z {h}) 212)))",
                      f"((∃z. z + 0 = {h}) ∧ (∃z. z + {h} = 212))", f"B{h}"))
    cs, _, w, l = conj(parts)
    st += cs
    st[-1] = st[-1].replace(f"({l} ", "(G ", 1)
    return st, w


# 1. The 44 per-element theorems.
for i in range(44):
    OUT[gen_path(i)] = distinct_file(i)
if MODE == "--materialize-only":
    os.makedirs(GEN, exist_ok=True)
    for old in os.listdir(GEN):
        os.remove(os.path.join(GEN, old))
    for i in range(44):
        with open(gen_path(i), "w", encoding="utf-8", newline="\n") as fh:
            fh.write(OUT[gen_path(i)])
    print(f"44 per-element theorems written to {GEN}/")
    sys.exit(0)
OUT["tests/rlt007s1/manifest.sha256"] = "".join(f"{sha(OUT[gen_path(i)])}  {gen_path(i)}\n" for i in range(44))

# 2. The 4 group summaries and the bounds theorem.
for k in range(4):
    cites, st, w, lo, hi = group_file(k)
    OUT[f"{ACC}/DG_{k}.rlt"] = proof(
        f"DG_{k}", f"RLT-007b stage 1: pairwise distinctness of H45, group {k}: the elements {lo} … {hi} each differ "
                   f"from every later element. Cites 11 per-element theorems in {GEN}/ (generated; pinned by SHA-256 here "
                   f"and in tests/rlt007s1/manifest.sha256)", w, st, cites=cites)
st, w = bounds_file()
OUT[f"{ACC}/BOUNDS.rlt"] = proof("BOUNDS", "RLT-007b stage 1: every element h of H45 satisfies 0 ≤ h and h ≤ 212, "
                                 "written with addition witnesses: ∃z. z + 0 = h and ∃z. z + h = 212", w, st, ["w"])

# 3. Rejected cases (the verdict substrings are fixed; tests compare them).
rows = []


def rej(name, text, code, reason):
    OUT[f"{REJ}/{name}.rlt"] = text
    rows.append(f"{REJ}/{name}.rlt\t{code}\t{reason}")


i2 = H.index(2)
rej("SX01_false_pair_2_eq_2",
    distinct_file(i2, "REJECT CASE: claims ¬(2 = 2) by instantiating K_2 at 9 (which gives ¬(2 = 12))", wrong=(i2 + 1, 2, 9)),
    1, "REJECT at step D2 — step claims `¬(2 = 2)`, rule `forall-elim` yields `¬(2 = 12)`")
rej("SX02_wrong_instance",
    distinct_file(i2, "REJECT CASE: ¬(2 = 12) by instantiating K_2 at 8 (which gives ¬(2 = 11))", wrong=(i2 + 1, 12, 8)),
    1, "REJECT at step D2 — step claims `¬(2 = 12)`, rule `forall-elim` yields `¬(2 = 11)`")
i12 = H.index(12)
rej("SX03_kchain_step_missing",
    distinct_file(i12, "REJECT CASE: the K-chain for 12 omits the generalization step KA7", drop=7),
    1, "REJECT at step KE8 — premise `KA7` is not an earlier step")
st, w = bounds_file(off_by_one=24)
rej("SX04_bounds_witness_off_by_one",
    proof("SX04", "REJECT CASE: the upper-bound witness for 24 is 187 (187 + 24 = 211), claimed to give 212", w, st, ["w"]),
    1, "REJECT at step UpI24 — step claims `187 + 24 = 212`, rule `forall-elim` yields `187 + 24 = 211`")
cites, st, w, _, _ = group_file(0, swap=(10, 11))
rej("SX05_group_cites_other_element",
    proof("SX05", "REJECT CASE: group 0 citing the theorem for element 56 in place of element 54", w, st, cites=cites),
    1, "the cited theorem `E10` proves `" + "(" * 31 + "¬(56 = 60) ∧ ¬(56 = 66))")
cites, st, w, _, _ = group_file(0, stale=0)
rej("SX06_group_stale_hash",
    proof("SX06", "REJECT CASE: group 0 recording a stale SHA-256 for a cited theorem", w, st, cites=cites),
    2, f"MALFORMED — citation `E0`: {GEN}/D_000.rlt has SHA-256")
cites, st, w, _, _ = group_file(0, missing=0)
rej("SX07_group_missing_theorem",
    proof("SX07", "REJECT CASE: group 0 citing a per-element theorem that does not exist", w, st, cites=cites),
    2, f"MALFORMED — citation `E0`: cited path `{GEN}/D_999.rlt`: entity not found")
OUT["tests/rlt007s1/expected.tsv"] = ("# file\texit\trequired substring of the verdict (RLT-007b stage 1 rejected cases)\n"
                                      + "\n".join(rows) + "\n")


# 4. Lean comparison (the RLT goals, literally; core Lean only).
def lean_ne_conj(i):
    a = H[i]
    t = f"¬({a} = {H[i + 1]})"
    for j in range(i + 2, 45):
        t = f"({t} ∧ ¬({a} = {H[j]}))"
    return t


def lean_ne_proof(i):
    # One `by decide` per disequality, joined by anonymous constructors in
    # the same left-nested shape (Decidable synthesis of a 44-deep
    # conjunction exceeds Lean's instance-size limit).
    t = "(by decide)"
    for j in range(i + 2, 45):
        t = f"⟨{t}, by decide⟩"
    return t


def lean_group(k):
    t = None
    for i in GROUPS[k]:
        x = lean_ne_conj(i)
        t = x if t is None else f"({t} ∧ {x})"
    return t


def lean_bounds():
    t, p = None, None
    for h in H:
        x = f"((∃ z, z + 0 = {h}) ∧ (∃ z, z + {h} = 212))"
        y = f"⟨⟨{h}, rfl⟩, ⟨{212 - h}, rfl⟩⟩"
        t, p = (x, y) if t is None else (f"({t} ∧ {x})", f"⟨{p}, {y}⟩")
    return t, p


bt, bp = lean_bounds()
lean = [f"""/-
RLT-007b stage 1, Lean comparison: GENERATED by scripts/gen-rlt007s1.py from the
reviewed transcription {TRANSCRIPTION}.
Core Lean only. Run: cd lean && lean RLT007S1.lean

What this file proves, in Lean (none of it is an RLT derivation):
  * d_H (44): RLT's per-element statements; dg_K (K = 0..3) and bounds:
    RLT's five stage-1 library statements, symbol for symbol (RLT's `+` read as `+` on Nat; the statements use no `<` or `∣`).
  * obligation_a: obligation (a) of Lemma 12.1 in Lean's own terms: the
    listed tuple has 45 entries, no repeats, first entry 0, last entry 212,
    and every entry lies in [0, 212], so the diameter is 212.
The reading of RLT's statements as obligation (a) is reviewed in
docs/RLT-007b-1-CORRESPONDENCE.md.
-/

namespace RLT007S1

/-- The tuple, as transcribed (section 12). -/
def H45 : List Nat := [{", ".join(map(str, H))}]
"""]
TH = []
lean.append("-- The statements are large (up to about 300 numerals); elaborating them\n"
            "-- needs more than the default heartbeat budget. Proofs stay `decide`,\n"
            "-- explicit witnesses and anonymous constructors.\n"
            "set_option maxHeartbeats 4000000\nset_option maxRecDepth 100000\n")
for i in range(44):
    lean.append(f"/-- The per-element statement for {H[i]} (tests/rlt007s1/gen/D_{H[i]:03d}.rlt), literally. -/\n"
                f"theorem d_{H[i]} :\n    {lean_ne_conj(i)} :=\n  {lean_ne_proof(i)}\n")
    TH.append(f"d_{H[i]}")
for k in range(4):
    pf = None
    for i in GROUPS[k]:
        pf = f"d_{H[i]}" if pf is None else f"⟨{pf}, d_{H[i]}⟩"
    lean.append(f"/-- RLT's `admissible.h45.distinct.g{k}` (tests/rlt007s1/accept/DG_{k}.rlt), literally. -/\n"
                f"theorem dg_{k} :\n    {lean_group(k)} :=\n  {pf}\n")
    TH.append(f"dg_{k}")
lean.append(f"/-- RLT's `admissible.h45.bounds` (tests/rlt007s1/accept/BOUNDS.rlt), literally. -/\n"
            f"theorem bounds :\n    {bt} :=\n  {bp}\n")
TH.append("bounds")
lean.append("""/-- Obligation (a) in Lean's terms: 45 entries, distinct, from 0 to 212. -/
theorem obligation_a :
    H45.length = 45 ∧ H45.Nodup ∧ H45.head? = some 0 ∧ H45.getLast? = some 212 ∧
      ∀ h ∈ H45, 0 ≤ h ∧ h ≤ 212 := by
  decide

end RLT007S1
""")
TH.append("obligation_a")
lean.append("\n" + "".join(f"#print axioms RLT007S1.{n}\n" for n in TH))
OUT["lean/RLT007S1.lean"] = "\n".join(lean)


# 5. Statement records. The checked goal is computed from the written goal;
# `rlt explain` re-derives it from the kernel and refuses a record whose pin
# differs. The basis hash is that of the correspondence review.
BASIS = "docs/RLT-007b-1-CORRESPONDENCE.md"
basis_sha = hashlib.sha256(open(BASIS, "rb").read()).hexdigest()


def yq(x):
    return '"' + x.replace("\\", "\\\\").replace('"', '\\"') + '"'


def checked_goal(path):
    # The checked goal is the written goal without its outermost parentheses
    # when they enclose the whole formula (as `rlt check` prints it).
    g = goal_of(OUT[path])
    if g.startswith("(") and g.endswith(")"):
        depth = 0
        for k, ch in enumerate(g):
            depth += ch == "("
            depth -= ch == ")"
            if depth == 0 and k < len(g) - 1:
                return g
        return g[1:-1]
    return g


reg_ex = {it["id"]: it["text"] for it in T["items"]}
RECS = [(f"H45-DISTINCT-G{k}", f"{ACC}/DG_{k}.rlt", f"dg_{k}",
         f"the elements {H[GROUPS[k][0]]} … {H[GROUPS[k][-1]]} of H45 each differ from every later element",
         f"Pairwise distinctness of H45, group {k}: for each listed element h_i with i in {GROUPS[k][0]}..{GROUPS[k][-1]} and each later h_j, h_i ≠ h_j. With the other three groups, all 990 pairs: the 45 listed integers are distinct, as the paper's '45-tuple' (a set of 45 distinct integers) requires.")
        for k in range(4)]
RECS.append(("H45-BOUNDS", f"{ACC}/BOUNDS.rlt", "bounds",
             "every element h of H45 satisfies 0 ≤ h ≤ 212",
             "For every listed h: some z has z + 0 = h, and some z has z + h = 212; over the naturals these say exactly 0 ≤ h and h ≤ 212. With 0 and 212 both listed (a source transcription check), the diameter max − min is 212."))
for rid, path, lth, subj, match in RECS:
    cg = checked_goal(path)
    OUT[f"statements/{rid}.yaml"] = f"""# Statement record (RLT-007b stage 1): Axiom Math, A New Bound for Small Gaps
# Between Primes (September 3, 2026), Lemma 12.1, obligation (a): {subj}.
# GENERATED by scripts/gen-rlt007s1.py; pins as `rlt check {path}` reports them.
format: rlt-statement/1
id: {rid}
subject: "Lemma 12.1 (Axiom Math, September 3, 2026 draft), obligation (a): {subj}"
source:
  registry: AXIOM-BGP212-2026-09-03
  commit: {T["pdf"]["sha256"]}
  excerpts:
    - {yq(reg_ex["h45-tuple"])}
    - {yq(reg_ex["lemma-12-1"])}
kernel:
  proof: {path}
  sha256: {sha(OUT[path])}
  convention: NAT-DEF
  axiom_set: ROBINSON-Q
  axiom_set_sha256: 4ad55192222576e3901abc4ee7ea5a6760b64ff212e8c55796ab9847e423b00c
  checked_goal: {yq(cg)}
  assumptions: []
semantic:
  status: partial
  assessed_by: "RLT-007b stage 1 review (docs/RLT-007b-1-CORRESPONDENCE.md)"
  date: 2026-11-12
  basis: {{path: {BASIS}, sha256: {basis_sha}}}
  assessed_goal: {yq(cg)}
  matches:
    - {yq(match)}
  differs:
    - "Obligation (a) only. The listing facts (45 entries, strictly increasing, endpoints 0 and 212) are source transcription checks made by the generator against the pinned PDF transcription, not kernel-checked. Admissibility for every prime is obligations (b)-(d)."
  lean:
    - {{file: lean/RLT007S1.lean, theorem: RLT007S1.{lth}}}
    - {{file: lean/RLT007S1.lean, theorem: RLT007S1.obligation_a}}
physical:
  applicability: not-applicable
  reason: "a statement in number theory"
open_questions: []
"""

# Write, or compare (--check).
if MODE == "--check":
    diff = []
    for path, text in sorted(OUT.items()):
        if not os.path.isfile(path) or open(path, encoding="utf-8", newline="").read() != text:
            diff.append(path)
    # Audit: the exact file sets of every generated directory.
    for d in (GEN, ACC, REJ):
        for f in sorted(os.listdir(d)) if os.path.isdir(d) else []:
            if f"{d}/{f}" not in OUT:
                diff.append(f"{d}/{f} (not produced by the generator)")
    for f in sorted(os.listdir("tests/rlt007s1")):
        if f not in ("gen", "accept", "reject", "manifest.sha256", "expected.tsv"):
            diff.append(f"tests/rlt007s1/{f} (not produced by the generator)")
    # Every H45 record except RLT-006's H45-OMITS-* is this stage's.
    stmts = sorted(f for f in os.listdir("statements") if f.startswith("H45-") and not f.startswith("H45-OMITS-"))
    if stmts != sorted(os.path.basename(p) for p in OUT if p.startswith("statements/")):
        diff.append("statements/H45-* (an extra or missing stage-1 record)")
    for f in sorted(os.listdir("lean")):
        if f.startswith("RLT007S1") and f"lean/{f}" not in OUT:
            diff.append(f"lean/{f} (not produced by the generator)")
    for d in diff:
        print(f"DIFFERS  {d}")
    print(f"scripts/gen-rlt007s1.py --check: {len(OUT)} outputs, {len(diff)} differ")
    sys.exit(0 if not diff else 1)

for d in (GEN, ACC, REJ):
    os.makedirs(d, exist_ok=True)
for d in (GEN, REJ):
    for old in os.listdir(d):
        os.remove(os.path.join(d, old))
for path, text in OUT.items():
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(text)
print(f"{len(OUT)} outputs written: 44 per-element theorems, manifest, 4 group summaries, bounds, "
      f"{len(rows)} rejected cases, Lean comparison, 5 statement records")

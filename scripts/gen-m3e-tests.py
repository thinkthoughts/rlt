#!/usr/bin/env python3
"""Write the RLT-003 M3e test proofs into tests/m3e/ (untrusted generator).

usage: scripts/gen-m3e-tests.py RLT_BINARY OUT_DIR

The complete modulo-30 classification, as approved (architecture review
D1-D8):
  * PD-k: p | k for the 22 non-representatives (smallest p in {2, 3, 5});
  * ND-d-r: not (d | r) for the 8 representatives, by M3c's scheme, unchanged
    (its functions are loaded verbatim from scripts/gen-m3c-tests.py); the
    three facts for 7 are cited from M3c (CA02-CA04);
  * F-r: free235(r) for the representatives other than 7 (CA05 is F-7);
  * E-FWD, E-BWD and E-T1, joined by checked citations (49 cited-file checks);
  * EA-07, a citing demonstration at 7.
The generator is outside the trusted base: the kernel checks every step, and
verify.sh regenerates this directory byte for byte.
"""
import hashlib
import re
import subprocess
import os
import sys

rlt, out = sys.argv[1], sys.argv[2]
ACC, REJ = "tests/m3e/accept", "tests/m3e/reject"

# M3c's scheme functions, verbatim (S, mul, gen_steps, lemma_m, peel, layers,
# n_layers, params_for, header, proof, not_dvd, witnesswise, sha).
_src = open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "gen-m3c-tests.py")).read()
exec(_src[_src.index("def S(t, k):"):_src.index("accept, reject, expected = {}, {}, []")], globals())

_m3c_header = header


def header(pid, goal, **kw):
    """M3c's header, relabelled for M3e (ids M3e.*, this generator named)."""
    return _m3c_header(pid, goal, **kw).replace("RLT-003 M3c test", "RLT-003 M3e test", 1).replace(
        "gen-m3c-tests.py", "gen-m3e-tests.py", 1).replace("(id M3c.", "(id M3e.", 1)


R8 = [1, 7, 11, 13, 17, 19, 23, 29]
PD = {0: (2, 0), 2: (2, 1), 3: (3, 1), 4: (2, 2), 5: (5, 1), 6: (2, 3), 8: (2, 4), 9: (3, 3),
      10: (2, 5), 12: (2, 6), 14: (2, 7), 15: (3, 5), 16: (2, 8), 18: (2, 9), 20: (2, 10),
      21: (3, 7), 22: (2, 11), 24: (2, 12), 25: (5, 5), 26: (2, 13), 27: (3, 9), 28: (2, 14)}
assert sorted(list(PD) + R8) == list(range(30))
M3C = {2: "tests/m3c/accept/CA02_two_not_divides_seven.rlt",
       3: "tests/m3c/accept/CA03_three_not_divides_seven.rlt",
       5: "tests/m3c/accept/CA04_five_not_divides_seven.rlt"}

accept, reject, expected = {}, {}, []


def rej(name, code, reason, text):
    reject[name] = text
    expected.append((name, code, reason))


def file_sha(path):
    with open(path, "rb") as f:
        return hashlib.sha256(f.read()).hexdigest()


def disj(terms, v):
    """Left-nested disjunction of v = t for t in terms, in order."""
    f = f"(= {v} {terms[0]})"
    for t in terms[1:]:
        f = f"(or {f} (= {v} {t}))"
    return f


def w_disj(terms, v):
    return " ∨ ".join(f"{v} = {t}" for t in terms)


def pd_name(k):
    return f"PD_{k:02d}"


def nd_name(d, r):
    return f"ND_{d}_{r:02d}"


def nd_path(d, r):
    return M3C[d] if r == 7 else f"{ACC}/{nd_name(d, r)}.rlt"


def pd_proof(k, p, q, pid=None, claim_k=None):
    st, last = gen_steps(f"{k} = {p} × {q}", "m")
    ck = k if claim_k is None else claim_k
    return proof(pid or pd_name(k), f"{p} ∣ {ck}", st + [f"(e1 (dvd {p} {ck}) (exists-intro {q} {last}))"])


# ── Leaves ─────────────────────────────────────────────────────────────────
for k, (p, q) in PD.items():
    accept[pd_name(k)] = pd_proof(k, p, q)
for r in R8:
    if r != 7:
        for d in (2, 3, 5):
            accept[nd_name(d, r)] = not_dvd(nd_name(d, r), d, r)


def acc_cite(name):
    return (f"{ACC}/{name}.rlt", sha(accept[name]))


def nd_cite(d, r):
    return (nd_path(d, r), file_sha(M3C[d]) if r == 7 else sha(accept[nd_name(d, r)]))


# ── F-r: free235(r), r a representative other than 7 ───────────────────────
def free_steps(r, pre=""):
    return [f"({pre}t2 (not (dvd 2 {r})) (cite {pre}T2))",
            f"({pre}t3 (not (dvd 3 {r})) (cite {pre}T3))",
            f"({pre}t5 (not (dvd 5 {r})) (cite {pre}T5))",
            f"({pre}a1 (and (not (dvd 2 {r})) (not (dvd 3 {r}))) (and-intro {pre}t2 {pre}t3))",
            f"({pre}a2 (free235 {r}) (and-intro {pre}a1 {pre}t5))"]


for r in R8:
    if r != 7:
        accept[f"F_{r:02d}"] = proof(f"F_{r:02d}", f"free235({r})", free_steps(r),
                                     cites=[(f"T{d}",) + nd_cite(d, r) for d in (2, 3, 5)])

# ── E-FWD: ∀r. r < 30 → (free235(r) → R8(r)) ─────────────────────────────
DA01 = "tests/m3d/accept/DA01_below_thirty.rlt"
E29 = lambda v: disj(list(range(30)), v)


def fwd(pid, reps=R8, pd=PD, label_map=None, bound=30):
    st = [f"(D0 (forall x (imp (lt x {bound}) {E29('x')})) (cite D))",
          f"(D1 (imp (lt a 30) {E29('a')}) (forall-elim D0 a))",
          "(H (lt a 30) (hyp h))",
          "(F (free235 a) (hyp f))",
          f"(E {E29('a')} (imp-elim D1 H))"]
    R = disj(reps, "a")

    def value(k, eq):
        V = f"V{k}"
        if k in reps:
            j = reps.index(k)
            cur = eq
            if j > 0:
                st.append(f"({V}_r {disj(reps[:j + 1], 'a')} (or-right {eq}))")
                cur = f"{V}_r"
            for i in range(j + 1, len(reps)):
                st.append(f"({V}_l{i} {disj(reps[:i + 1], 'a')} (or-left {cur}))")
                cur = f"{V}_l{i}"
            return cur
        p = pd.get(k, (2, None))[0]
        sel = {2: "(and-left " + V + "_a)", 3: "(and-right " + V + "_a)", 5: "(and-right F)"}[p]
        st.extend([f"({V}_c (dvd {p} {k}) (cite P{k}))",
                   f"({V}_s (= {k} a) (symm {eq}))",
                   f"({V}_d (dvd {p} a) (subst {V}_s (at u (dvd {p} u)) {V}_c))",
                   f"({V}_a (and (not (dvd 2 a)) (not (dvd 3 a))) (and-left F))",
                   f"({V}_n (not (dvd {p} a)) {sel})",
                   f"({V}_f false (imp-elim {V}_n {V}_d))",
                   f"({V}_e {R} (false-elim {V}_f))"])
        return f"{V}_e"

    def peel_e(k, lab):
        if k == 0:
            return value(0, lab)
        st.append(f"(L{k} {E29_upto(k - 1)} (hyp l{k}))")
        left = peel_e(k - 1, f"L{k}")
        st.append(f"(Q{k} (= a {k}) (hyp q{k}))")
        right = value(k, f"Q{k}")
        st.append(f"(O{k} {R} (or-cases {lab} l{k} {left} q{k} {right}))")
        return f"O{k}"

    top = peel_e(29, "E")
    st += [f"(I1 (imp (free235 a) {R}) (imp-intro f {top}))",
           f"(I2 (imp (lt a 30) (imp (free235 a) {R})) (imp-intro h I1))",
           f"(G (forall r (imp (lt r 30) (imp (free235 r) {disj(reps, 'r')}))) (forall-intro a I2))"]
    lm = label_map or {}
    cites = [("D", DA01, file_sha(DA01))] + [
        (f"P{k}",) + acc_cite(pd_name(lm.get(k, k))) for k in sorted(pd) if k not in reps]
    return proof(pid, f"∀r. r < 30 → (free235(r) → ({w_disj(reps, 'r')}))", st,
                 cites=cites, params="a")


def E29_upto(k):
    return disj(list(range(k + 1)), "a")


accept["E_FWD"] = fwd("E_FWD")

# ── E-BWD: ∀r. R8(r) → free235(r) ─────────────────────────────────────────


def bwd(pid, reps=R8, nd=None, swap23=False):
    nd = nd or {}
    st = [f"(G0 {disj(reps, 'a')} (hyp g))"]

    def value(r, eq):
        B = f"B{r}"
        first, second = ("3", "2") if swap23 else ("2", "3")
        st.extend([f"({B}_2 (not (dvd 2 {r})) (cite N2_{r}))",
                   f"({B}_3 (not (dvd 3 {r})) (cite N3_{r}))",
                   f"({B}_5 (not (dvd 5 {r})) (cite N5_{r}))",
                   f"({B}_a (and (not (dvd {first} {r})) (not (dvd {second} {r}))) (and-intro {B}_{first} {B}_{second}))",
                   f"({B}_f (free235 {r}) (and-intro {B}_a {B}_5))",
                   f"({B}_s (= {r} a) (symm {eq}))",
                   f"({B}_x (free235 a) (subst {B}_s (at u (free235 u)) {B}_f))"])
        return f"{B}_x"

    def peel_r(j, lab):
        if j == 0:
            return value(reps[0], lab)
        st.append(f"(K{j} {disj(reps[:j], 'a')} (hyp k{j}))")
        left = peel_r(j - 1, f"K{j}")
        st.append(f"(J{j} (= a {reps[j]}) (hyp j{j}))")
        right = value(reps[j], f"J{j}")
        st.append(f"(M{j} (free235 a) (or-cases {lab} k{j} {left} j{j} {right}))")
        return f"M{j}"

    top = peel_r(len(reps) - 1, "G0")
    st += [f"(I (imp {disj(reps, 'a')} (free235 a)) (imp-intro g {top}))",
           f"(G (forall r (imp {disj(reps, 'r')} (free235 r))) (forall-intro a I))"]
    cites = []
    for r in reps:
        for d in (2, 3, 5):
            cites.append((f"N{d}_{r}",) + (nd[(d, r)] if (d, r) in nd else nd_cite(d, r)))
    return proof(pid, f"∀r. ({w_disj(reps, 'r')}) → free235(r)", st, cites=cites, params="a")


accept["E_BWD"] = bwd("E_BWD")

# ── E-T1: the classification ───────────────────────────────────────────────
R8r, R8a = disj(R8, "r"), disj(R8, "a")
T1_GOAL = f"∀r. r < 30 → (free235(r) → ({w_disj(R8, 'r')})) ∧ (({w_disj(R8, 'r')}) → free235(r))"


def t1(pid, fwd_hash=None, goal=T1_GOAL):
    st = [f"(W (forall r (imp (lt r 30) (imp (free235 r) {R8r}))) (cite FWD))",
          f"(B (forall r (imp {R8r} (free235 r))) (cite BWD))",
          f"(W1 (imp (lt a 30) (imp (free235 a) {R8a})) (forall-elim W a))",
          "(H (lt a 30) (hyp h))",
          f"(W2 (imp (free235 a) {R8a}) (imp-elim W1 H))",
          f"(B1 (imp {R8a} (free235 a)) (forall-elim B a))",
          f"(C (and (imp (free235 a) {R8a}) (imp {R8a} (free235 a))) (and-intro W2 B1))",
          f"(I (imp (lt a 30) (and (imp (free235 a) {R8a}) (imp {R8a} (free235 a)))) (imp-intro h C))",
          f"(G (forall r (imp (lt r 30) (and (imp (free235 r) {R8r}) (imp {R8r} (free235 r))))) (forall-intro a I))"]
    f = acc_cite("E_FWD")
    return proof(pid, goal, st, params="a",
                 cites=[("FWD", f[0], fwd_hash or f[1]), ("BWD",) + acc_cite("E_BWD")])


accept["E_T1"] = t1("E_T1")
R87 = disj(R8, "7")
accept["EA_07"] = proof("EA_07", f"7 < 30 → (free235(7) → ({w_disj(R8, '7')})) ∧ (({w_disj(R8, '7')}) → free235(7))", [
    f"(T (forall r (imp (lt r 30) (and (imp (free235 r) {R8r}) (imp {R8r} (free235 r))))) (cite T1))",
    f"(S7 (imp (lt 7 30) (and (imp (free235 7) {R87}) (imp {R87} (free235 7)))) (forall-elim T 7))"],
    cites=[("T1",) + acc_cite("E_T1")])

# ── Rejected ───────────────────────────────────────────────────────────────
rej("ER04_false_not_three_divides_nine", 1, "step claims `¬(0 = 0)`, rule `inst` yields `¬(0 = 1)`",
    not_dvd("ER04", 3, 9))
rej("ER11b_false_not_two_divides_two", 1, "step claims `¬(0 = 0)`, rule `inst` yields `¬(0 = 1)`",
    not_dvd("ER11b", 2, 2))
er04 = (f"{REJ}/ER04_false_not_three_divides_nine.rlt", sha(reject["ER04_false_not_three_divides_nine"]))
er11b = (f"{REJ}/ER11b_false_not_two_divides_two.rlt", sha(reject["ER11b_false_not_two_divides_two"]))
rej("ER01_representative_29_missing", 1, "cite: `P29` is not a declared citation",
    fwd("ER01", reps=R8[:-1]))
rej("ER02_extra_representative_9", 2, "is not an accepted theorem (REJECT",
    bwd("ER02", reps=[1, 7, 9, 11, 13, 17, 19, 23, 29],
        nd={(3, 9): er04, (2, 9): er04, (5, 9): er04}))
rej("ER03_nine_in_place_of_seven", 2, "is not an accepted theorem (REJECT",
    bwd("ER03", reps=[1, 9, 11, 13, 17, 19, 23, 29],
        nd={(3, 9): er04, (2, 9): er04, (5, 9): er04}))
rej("ER05_false_three_divides_four", 1, "exists-intro: witness does not match premise",
    pd_proof(3, 3, 1, pid="ER05", claim_k=4))
rej("ER06_pd4_under_pd6_label", 1, "the cited theorem `P6` proves",
    fwd("ER06", label_map={6: 4}))
rej("ER07_edited_fwd_hash", 2, "the citation records", t1("ER07", fwd_hash="0" * 64))
rej("ER08_bound_31_from_da01", 1, "the cited theorem `D` proves", fwd("ER08", bound=31))
rej("ER09_free235_conjuncts_reordered", 1, "rule `and-intro` yields", bwd("ER09", swap23=True))
rej("ER10_biconditional_written", 2, "goal:",
    t1("ER10", goal=f"∀r. r < 30 → (free235(r) ↔ ({w_disj(R8, 'r')}))"))
rej("ER11_backward_with_two", 2, "is not an accepted theorem (REJECT",
    bwd("ER11", reps=[2], nd={(2, 2): er11b, (3, 2): er11b, (5, 2): er11b}))
rej("ER12_forward_with_one_only", 1, "cite: `P7` is not a declared citation", fwd("ER12", reps=[1]))
t1c = acc_cite("E_T1")
rej("ER13_citation_budget_exceeded", 2, "more than 64 files",
    proof("ER13", T1_GOAL, [f"(t (forall r (imp (lt r 30) (and (imp (free235 r) {R8r}) (imp {R8r} (free235 r))))) (cite A))"],
          cites=[("A",) + t1c, ("B",) + t1c]))
m2 = "tests/m2/D_07_2.rlt"
rej("ER14_cites_peano_arith_theorem", 2, "differs from this file's ROBINSON-Q",
    proof("ER14", "7 = 2 × 3 + 1", ["(t (= 7 (+ (* 2 3) 1)) (cite M))"], cites=[("M", m2, file_sha(m2))]))
rej("ER16_assumption_in_unconditional_mode", 1, "unconditional goal requested, but the result depends on assumptions",
    proof("ER16", "free235(7)", ["(t (free235 7) (hyp h))"], extra="  (assume h \"free235(7)\")\n"))

for sub, files in (("accept", accept), ("reject", reject)):
    os.makedirs(os.path.join(out, sub), exist_ok=True)
    for name, text in files.items():
        with open(os.path.join(out, sub, name + ".rlt"), "w") as f:
            f.write(text)
with open(os.path.join(out, "expected.tsv"), "w") as f:
    f.write("# file\texit\trequired substring of the verdict (RLT-003 M3e rejected cases)\n")
    for name, code, reason in expected:
        f.write(f"{REJ}/{name}.rlt\t{code}\t{reason}\n")

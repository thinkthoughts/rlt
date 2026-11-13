#!/usr/bin/env python3
"""RLT-007b stage 3 generator (untrusted): the uniform omission for p ≥ 213.

For every element h of H₄₅ (h ≤ 212, h ≠ 1) and every p with 212 < p, no k
gives h = (p × k) + 1:

    TAIL:  ∀p. (212 < p → ⋀_{h ∈ H₄₅} ¬(∃k. h = (p × k) + 1))

The conjunction is left-nested in the listed order, the shape of RLT-006's
per-prime omission statements (OM_p) with the class r = 1 and the modulus a
variable. `<` is the existing definition (x < y :≡ ∃z. S(z) + x = y); no
definition, axiom or rule is added. Deterministic; every proof is checked by
the kernel, nothing here is trusted.

Proof, in one file (TAIL.rlt), under the hypothesis S(z) + 212 = p:
  * D_n: ∀x. ∀y. x + Sⁿ(y) = Sⁿ(x + y) and Q_n: ∀x. ∀y. (Sⁿ(x) = Sⁿ(y) → x = y)
    are built by composition (D_1 is add_succ, Q_1 is q1_succ_inj), so a
    family up to 212 costs O(n log n) nodes instead of a chain's O(n²);
  * p = S²¹³(z) (D_212 at S(z), 0);
  * k = 0: h = (p × 0) + 1 = 1, refuted for each listed h (h ≠ 1, by Q1, Q2);
  * k = S(j): (p × S(j)) + 1 = S²¹⁴((p × j) + z) (mul_succ, D_212, add_succ),
    and Q_h at 0 turns h = S²¹⁴(…) into 0 = S(…), refuted by Q2.
No restriction on k and no positivity hypothesis is needed.

Outputs:
  tests/rlt007s3/accept/TAIL.rlt   the theorem
  tests/rlt007s3/reject/*.rlt      rejected cases; tests/rlt007s3/expected.tsv
  statements/H45-TAIL-OMITS.yaml   statement record
  lean/RLT007S3.lean               Lean comparison

Source transcription check (not kernel-checked): the generator's tuple is the
transcribed H₄₅ (item `h45-tuple`): 45 entries, increasing, from 0 to 212,
without 1.

Usage: gen-rlt007s3.py RLT_BINARY [--check | --self-test]
"""
import hashlib
import os
import re
import sys

import yaml

if len(sys.argv) not in (2, 3) or sys.argv[1].startswith("--") or (len(sys.argv) == 3 and sys.argv[2] not in ("--check", "--self-test")):
    sys.exit("usage: gen-rlt007s3.py RLT_BINARY [--check | --self-test]")
MODE = sys.argv[2] if len(sys.argv) == 3 else ""
root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
os.chdir(root)

T = yaml.safe_load(open("sources/transcriptions/AXIOM-BGP212-2026-09-03.yaml", encoding="utf-8"))
ITEMS = {it["id"]: it for it in T["items"]}
H = list(ITEMS["h45-tuple"]["numbers"])
BOUND = 212   # the hypothesis is BOUND < p


def tuple_checks(h):
    bad = []
    if len(h) != 45:
        bad.append(f"{len(h)} entries, not 45")
    if any(a >= b for a, b in zip(h, h[1:])):
        bad.append("not strictly increasing")
    if not h or h[0] != 0 or h[-1] != BOUND:
        bad.append("not from 0 to 212")
    if 1 in h:
        bad.append("1 is listed (the class 1 would not be omitted)")
    if any(x < 0 or x > BOUND for x in h):
        bad.append("an entry outside [0, 212]")
    return bad


if MODE == "--self-test":
    cases = [
        ("1 added", sorted(H[:-1] + [1])),
        ("213 replacing 212", H[:-1] + [213]),
        ("44 entries", H[:-1]),
        ("a duplicate", H[:2] + [H[1]] + H[3:]),
        ("out of order", [H[1], H[0]] + H[2:]),
        ("a negative entry", [-2] + H[1:]),
    ]
    n = 0
    for name, alt in cases:
        r = tuple_checks(alt)
        print(f"{'refused ' if r else 'ACCEPTED'} {name}: {'; '.join(r)}")
        n += 1 if r else 0
    print(f"generator self-test: {n} of {len(cases)} altered tuples refused")
    sys.exit(0 if n == len(cases) else 1)

bad = tuple_checks(H)
if bad:
    sys.exit("source transcription check failed: " + "; ".join(bad))

ACC, REJ = "tests/rlt007s3/accept", "tests/rlt007s3/reject"
OUT = {}


def S(t, k):
    for _ in range(k):
        t = f"(S {t})"
    return t


def sha(text):
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def om(h, p):
    return f"(not (exists k (= {h} (+ (* {p} k) 1))))"


def om_w(h, p):
    return f"¬(∃k. {h} = ({p} × k) + 1)"


def conj_sx(hs, p):
    f = om(hs[0], p)
    for h in hs[1:]:
        f = f"(and {f} {om(h, p)})"
    return f


def conj_w(hs, p):
    f = om_w(hs[0], p)
    for h in hs[1:]:
        f = f"({f} ∧ {om_w(h, p)})"
    return f


def goal_sx(hs, p="p"):
    return f"(imp (lt {BOUND} {p}) {conj_sx(hs, p)})"


def goal_w(hs):
    return f"∀p. ({BOUND} < p → {conj_w(hs, 'p')})"


class Proof:
    def __init__(self, pid):
        self.pid, self.st, self.params, self.cites = pid, [], [], []

    def par(self, name):
        if name not in self.params:
            self.params.append(name)
        return name

    def add(self, lab, f, r):
        self.st.append(f"({lab} {f} {r})")
        return lab

    def text(self, comment, goal_w):
        c = ""
        if self.cites:
            c = "  (cites\n" + "\n".join(f'    ({l} "{pa}" "{hs}")' for l, pa, hs in self.cites) + ")\n"
        pr = f"  (params {' '.join(self.params)})\n" if self.params else ""
        body = "\n".join("    " + s for s in self.st)
        return (f"; {comment}\n(rlt-proof\n  (id RLT007S3.{self.pid})\n  (convention NAT-DEF)\n  (goal \"{goal_w}\")\n"
                f"  (mode unconditional)\n  (axioms ROBINSON-Q)\n{c}{pr}  (steps\n{body}))\n")


class Iter:
    """Iterated-successor lemmas, built by composition (each costs O(n) nodes,
    so a family up to 214 costs O(n log n) instead of a chain's O(n²)):
      D_n: ∀x. ∀y. x + Sⁿ(y) = Sⁿ(x + y)          (from add_succ)
      Q_n: ∀x. ∀y. (Sⁿ(x) = Sⁿ(y) → x = y)        (from q1_succ_inj)"""

    def __init__(self, P, kind):
        self.P, self.kind, self.have = P, kind, {}
        P.par("a"), P.par("b")

    def formula(self, n, x="x", y="y", bind=True):
        if self.kind == "D":
            body = f"(= (+ {x} {S(y, n)}) {S(f'(+ {x} {y})', n)})"
        else:
            body = f"(imp (= {S(x, n)} {S(y, n)}) (= {x} {y}))"
        return f"(forall x (forall y {body}))" if bind else body

    def get(self, n):
        if n in self.have:
            return self.have[n]
        P, K = self.P, self.kind
        lab = f"{K}{n}"
        if n == 1:
            ax = "(inst add_succ (x a) (y b))" if K == "D" else "(inst q1_succ_inj (x a) (y b))"
            P.add(f"{lab}i", self.formula(1, "a", "b", False), ax)
        else:
            m = 1 << (n.bit_length() - 1)
            if m == n:
                m = n // 2
            self.compose(n, self.get(m), m, self.get(n - m), n - m)
        P.add(f"{lab}y", f"(forall y {self.formula(n, 'a', 'y', False)})", f"(forall-intro b {lab}i)")
        P.add(lab, self.formula(n), f"(forall-intro a {lab}y)")
        self.have[n] = lab
        return lab

    def compose(self, n, Lm, m, Lk, k):
        P, lab = self.P, f"{self.kind}{n}"
        a, b = "a", "b"
        if self.kind == "D":
            P.add(f"{lab}f1", f"(forall y {self.formula(m, a, 'y', False)})", f"(forall-elim {Lm} {a})")
            P.add(f"{lab}f2", f"(= (+ {a} {S(b, n)}) {S(f'(+ {a} {S(b, k)})', m)})", f"(forall-elim {lab}f1 {S(b, k)})")
            P.add(f"{lab}g1", f"(forall y {self.formula(k, a, 'y', False)})", f"(forall-elim {Lk} {a})")
            P.add(f"{lab}g2", self.formula(k, a, b, False), f"(forall-elim {lab}g1 {b})")
            P.add(f"{lab}i", self.formula(n, a, b, False),
                  f"(subst {lab}g2 (at u (= (+ {a} {S(b, n)}) {S('u', m)})) {lab}f2)")
        else:
            P.add(f"{lab}h", f"(= {S(a, n)} {S(b, n)})", f"(hyp q{n})")
            P.add(f"{lab}f1", f"(forall y {self.formula(m, S(a, k), 'y', False)})", f"(forall-elim {Lm} {S(a, k)})")
            P.add(f"{lab}f2", self.formula(m, S(a, k), S(b, k), False), f"(forall-elim {lab}f1 {S(b, k)})")
            P.add(f"{lab}f3", f"(= {S(a, k)} {S(b, k)})", f"(imp-elim {lab}f2 {lab}h)")
            P.add(f"{lab}g1", f"(forall y {self.formula(k, a, 'y', False)})", f"(forall-elim {Lk} {a})")
            P.add(f"{lab}g2", self.formula(k, a, b, False), f"(forall-elim {lab}g1 {b})")
            P.add(f"{lab}g3", f"(= {a} {b})", f"(imp-elim {lab}g2 {lab}f3)")
            P.add(f"{lab}i", self.formula(n, a, b, False), f"(imp-intro q{n} {lab}g3)")


def shared(P, hs):
    """Under the hypothesis HZ: S(z) + 212 = pp, the steps every element uses.
    Labels: PZ (pp = S²¹³(z)), Z1 ((pp × 0) + 1 = 1),
    LEM (∀j. (pp × S(j)) + 1 = S²¹⁴((pp × j) + z)), and Q_h for each listed h > 0."""
    pp, z = P.par("pp"), P.par("z")
    D = Iter(P, "D")
    d = D.get(BOUND)
    # pp = S²¹³(z): S(z) + 212 = S²¹²(S(z) + 0) = S²¹²(S(z)).
    P.add("P0", f"(forall y {D.formula(BOUND, f'(S {z})', 'y', False)})", f"(forall-elim {d} (S {z}))")
    P.add("P1", f"(= (+ (S {z}) {BOUND}) {S(f'(+ (S {z}) 0)', BOUND)})", "(forall-elim P0 0)")
    P.add("P2", f"(= (+ (S {z}) 0) (S {z}))", f"(inst add_zero (x (S {z})))")
    P.add("P3", f"(= (+ (S {z}) {BOUND}) {S(z, BOUND + 1)})", f"(subst P2 (at u (= (+ (S {z}) {BOUND}) {S('u', BOUND)})) P1)")
    P.add("P4", f"(= {pp} (+ (S {z}) {BOUND}))", "(symm HZ)")
    P.add("PZ", f"(= {pp} {S(z, BOUND + 1)})", "(trans P4 P3)")
    # Z1: (pp × 0) + 1 = 1.
    P.add("Zm", f"(= (* {pp} 0) 0)", f"(inst mul_zero (x {pp}))")
    P.add("Za", f"(= (+ (* {pp} 0) 1) (S (+ (* {pp} 0) 0)))", f"(inst add_succ (x (* {pp} 0)) (y 0))")
    P.add("Zb", f"(= (+ (* {pp} 0) 0) (* {pp} 0))", f"(inst add_zero (x (* {pp} 0)))")
    P.add("Zc", f"(= (+ (* {pp} 0) 0) 0)", "(trans Zb Zm)")
    P.add("Zd", f"(= (S (+ (* {pp} 0) 0)) (S 0))", "(cong S Zc)")
    P.add("Z1", f"(= (+ (* {pp} 0) 1) 1)", "(trans Za Zd)")
    # LEM: ∀j. (pp × S(j)) + 1 = S²¹⁴((pp × j) + z).
    j = P.par("jj")
    X, Y, W = f"(* {pp} {j})", f"(* {pp} (S {j}))", f"(+ (* {pp} {j}) {z})"
    P.add("Ma", f"(= {Y} (+ {X} {pp}))", f"(inst mul_succ (x {pp}) (y {j}))")
    P.add("Mb", f"(= {Y} (+ {X} {S(z, BOUND + 1)}))", f"(subst PZ (at u (= {Y} (+ {X} u))) Ma)")
    P.add("Mc", f"(forall y {D.formula(BOUND, X, 'y', False)})", f"(forall-elim {d} {X})")
    P.add("Md", f"(= (+ {X} {S(z, BOUND + 1)}) {S(f'(+ {X} (S {z}))', BOUND)})", f"(forall-elim Mc (S {z}))")
    P.add("Me", f"(= (+ {X} (S {z})) (S {W}))", f"(inst add_succ (x {X}) (y {z}))")
    P.add("Mf", f"(= (+ {X} {S(z, BOUND + 1)}) {S(W, BOUND + 1)})", f"(subst Me (at u (= (+ {X} {S(z, BOUND + 1)}) {S('u', BOUND)})) Md)")
    P.add("Mg", f"(= {Y} {S(W, BOUND + 1)})", "(trans Mb Mf)")
    P.add("Ld", f"(= (+ {Y} 1) (S (+ {Y} 0)))", f"(inst add_succ (x {Y}) (y 0))")
    P.add("Le", f"(= (+ {Y} 0) {Y})", f"(inst add_zero (x {Y}))")
    P.add("Lf", f"(= (S (+ {Y} 0)) (S {Y}))", "(cong S Le)")
    P.add("Lg", f"(= (+ {Y} 1) (S {Y}))", "(trans Ld Lf)")
    P.add("Lh", f"(= (S {Y}) {S(W, BOUND + 2)})", "(cong S Mg)")
    P.add("Li", f"(= (+ {Y} 1) {S(W, BOUND + 2)})", "(trans Lg Lh)")
    P.add("LEM", f"(forall j (= (+ (* {pp} (S j)) 1) {S(f'(+ (* {pp} j) {z})', BOUND + 2)}))", f"(forall-intro {j} Li)")
    # Q_h for every listed h > 0, along the list (each from the previous and the gap).
    Q = Iter(P, "Q")
    prev = 0
    for h in hs:
        if h == 0:
            continue
        if prev == 0:
            Q.get(h)
        elif h not in Q.have:
            Q.compose(h, Q.get(prev), prev, Q.get(h - prev), h - prev)
            lab = f"Q{h}"
            P.add(f"{lab}y", f"(forall y {Q.formula(h, 'a', 'y', False)})", f"(forall-intro b {lab}i)")
            P.add(lab, Q.formula(h), f"(forall-intro a {lab}y)")
            Q.have[h] = lab
        prev = h
    return Q


def element(P, Q, h):
    """Steps proving ¬(∃k. h = (pp × k) + 1) from the shared steps; returns the label."""
    X = f"H{h}_"
    pp, z = "pp", "z"
    k, jv = P.par(f"k{h}"), P.par(f"j{h}")
    P.add(f"{X}E", f"(= {h} (+ (* {pp} {k}) 1))", f"(hyp {X}he)")
    P.add(f"{X}q", f"(or (= {k} 0) (exists y (= {k} (S y))))", f"(inst q3_zero_or_succ (x {k}))")
    # k = 0: h = 1.
    P.add(f"{X}0h", f"(= {k} 0)", f"(hyp {X}h0)")
    P.add(f"{X}0s", f"(= {h} (+ (* {pp} 0) 1))", f"(subst {X}0h (at u (= {h} (+ (* {pp} u) 1))) {X}E)")
    P.add(f"{X}0t", f"(= {h} 1)", f"(trans {X}0s Z1)")
    if h in (0, 1):  # h = 1 only in a rejected case: (= 1 1) is no contradiction
        P.add(f"{X}0q", "(not (= 0 (S 0)))", "(inst q2_zero_ne_succ (x 0))")
        P.add(f"{X}0f", "false", f"(imp-elim {X}0q {X}0t)")
    else:
        assert h >= 2
        P.add(f"{X}0a", f"(imp (= {h} 1) (= {h - 1} 0))", f"(inst q1_succ_inj (x {h - 1}) (y 0))")
        P.add(f"{X}0b", f"(= {h - 1} 0)", f"(imp-elim {X}0a {X}0t)")
        P.add(f"{X}0c", f"(= 0 {h - 1})", f"(symm {X}0b)")
        P.add(f"{X}0q", f"(not (= 0 {h - 1}))", f"(inst q2_zero_ne_succ (x {h - 2}))")
        P.add(f"{X}0f", "false", f"(imp-elim {X}0q {X}0c)")
    # k = S(j): h = S²¹⁴((pp × j) + z), refuted by N_h.
    P.add(f"{X}1x", f"(exists y (= {k} (S y)))", f"(hyp {X}h1)")
    P.add(f"{X}1h", f"(= {k} (S {jv}))", f"(hyp {X}hj)")
    P.add(f"{X}1s", f"(= {h} (+ (* {pp} (S {jv})) 1))", f"(subst {X}1h (at u (= {h} (+ (* {pp} u) 1))) {X}E)")
    W = f"(+ (* {pp} {jv}) {z})"
    P.add(f"{X}1l", f"(= (+ (* {pp} (S {jv})) 1) {S(W, BOUND + 2)})", f"(forall-elim LEM {jv})")
    P.add(f"{X}1t", f"(= {h} {S(W, BOUND + 2)})", f"(trans {X}1s {X}1l)")
    if h == 0:
        P.add(f"{X}1n", f"(not (= 0 {S(W, BOUND + 2)}))", f"(inst q2_zero_ne_succ (x {S(W, BOUND + 1)}))")
        P.add(f"{X}1f", "false", f"(imp-elim {X}1n {X}1t)")
    else:
        V = S(W, BOUND + 2 - h)
        P.add(f"{X}1a", f"(forall y (imp (= {h} {S('y', h)}) (= 0 y)))", f"(forall-elim {Q.have[h]} 0)")
        P.add(f"{X}1b", f"(imp (= {h} {S(W, BOUND + 2)}) (= 0 {V}))", f"(forall-elim {X}1a {V})")
        P.add(f"{X}1c", f"(= 0 {V})", f"(imp-elim {X}1b {X}1t)")
        P.add(f"{X}1n", f"(not (= 0 {V}))", f"(inst q2_zero_ne_succ (x {S(W, BOUND + 1 - h)}))")
        P.add(f"{X}1f", "false", f"(imp-elim {X}1n {X}1c)")
    P.add(f"{X}1F", "false", f"(exists-elim {X}1x {jv} {X}hj {X}1f)")
    P.add(f"{X}c", "false", f"(or-cases {X}q {X}h0 {X}0f {X}h1 {X}1F)")
    P.add(f"{X}x", f"(exists k (= {h} (+ (* {pp} k) 1)))", f"(hyp {X}hx)")
    P.add(f"{X}F", "false", f"(exists-elim {X}x {k} {X}he {X}c)")
    return P.add(f"{X}N", om(h, pp), f"(imp-intro {X}hx {X}F)")


def close(P, body_label, hs):
    """From G(pp) under HZ, conclude ∀p. (212 < p → G(p))."""
    P.add("HL", f"(lt {BOUND} pp)", "(hyp hlt)")
    P.add("GZ", conj_sx(hs, "pp"), f"(exists-elim HL z hz {body_label})")
    P.add("I", goal_sx(hs, "pp"), "(imp-intro hlt GZ)")
    P.add("G", f"(forall p {goal_sx(hs)})", "(forall-intro pp I)")


def tail(hs=None, pid="TAIL"):
    hs = H if hs is None else hs
    P = Proof(pid)
    P.par("pp"), P.par("z")
    P.add("HZ", f"(= (+ (S z) {BOUND}) pp)", "(hyp hz)")
    Q = shared(P, hs)
    labs = [element(P, Q, h) for h in hs]
    cur = labs[0]
    for i, l in enumerate(labs[1:], 1):
        cur = P.add(f"C{i}", conj_sx(hs[:i + 1], "pp"), f"(and-intro {cur} {l})")
    close(P, cur, hs)
    return P


OUT[f"{ACC}/TAIL.rlt"] = tail().text(
    "RLT-007b stage 3: for every p with 212 < p, no element h of H45 satisfies h = (p × k) + 1 for any k "
    "(the class 1 is omitted modulo every such p).", goal_w(H))


# Rejected cases (expected verdict substrings fixed after inspection).
rows = []


def rej(name, text, code, reason):
    OUT[f"{REJ}/{name}.rlt"] = text
    rows.append(f"{REJ}/{name}.rlt\t{code}\t{reason}")


def regen(comment, hs=None, bound=None):
    global BOUND
    keep = BOUND
    BOUND = keep if bound is None else bound
    try:
        hs = H if hs is None else hs
        return tail(hs).text("REJECT CASE: " + comment, goal_w(hs))
    finally:
        BOUND = keep


rej("SR01_bound_lowered_to_210",
    regen("the bound lowered to 210 (false: p = 211, k = 1 gives 212 = (211 × 1) + 1)", bound=210),
    1, "REJECT at step H212_1n — step claims `¬(0 = (pp × j212) + z)`, rule `inst` yields `¬(0 = S((pp × j212) + z))`")
rej("SR02_element_212_replaced_by_214",
    regen("the element 212 replaced by 214 (false: p = 213, k = 1 gives 214 = (213 × 1) + 1)", hs=H[:-1] + [214]),
    1, "REJECT at step H214_1n — step claims `¬(0 = (pp × j214) + z)`, rule `inst` yields `¬(0 = S((pp × j214) + z))`")
rej("SR03_element_0_replaced_by_1",
    regen("the element 0 replaced by 1 (false: k = 0 gives 1 = (p × 0) + 1)", hs=[1] + H[1:]),
    1, "REJECT at step H1_0f — imp-elim: antecedent does not match: `0 = 1` against `1 = 1`")
t = OUT[f"{ACC}/TAIL.rlt"]
gw = goal_w(H)
rej("SR04_statement_class_2",
    t.replace(f'(goal "{gw}")', f'(goal "{gw.replace(") + 1)", ") + 2)")}")', 1)
     .replace("; RLT-007b stage 3:", "; REJECT CASE (only the statement changed, to the class 2: false, k = 0 gives 2 = (p × 0) + 2):", 1),
    1, "the goal is `∀p. ((∃z. S(z) + 212 = p) → ((((((((((((((((((((((((((((((((((((((((((((¬(∃k. 0 = (p × k) + 2)")
rej("SR05_statement_without_212",
    t.replace(f'(goal "{gw}")', f'(goal "{goal_w(H[:-1])}")', 1)
     .replace("; RLT-007b stage 3:", "; REJECT CASE (only the statement changed: the element 212 dropped):", 1),
    1, "the goal is `∀p. ((∃z. S(z) + 212 = p) → (((((((((((((((((((((((((((((((((((((((((((¬(∃k. 0 = (p × k) + 1)")
rej("SR06_statement_without_bound",
    t.replace(f'(goal "{gw}")', f'(goal "∀p. {conj_w(H, "p")}")', 1)
     .replace("; RLT-007b stage 3:", "; REJECT CASE (only the statement changed: the hypothesis 212 < p dropped; false for p = 1):", 1),
    1, "the goal is `∀p. ((((((((((((((((((((((((((((((((((((((((((((¬(∃k. 0 = (p × k) + 1)")
OUT["tests/rlt007s3/expected.tsv"] = ("# file\texit\trequired substring of the verdict (RLT-007b stage 3 rejected cases)\n"
                                     + "\n".join(rows) + "\n")

# Lean comparison.
def lean_conj(hs):
    f = f"¬ (∃ k, {hs[0]} = p * k + 1)"
    for h in hs[1:]:
        f = f"({f}) ∧ ¬ (∃ k, {h} = p * k + 1)"
    return f


hl = "[" + ", ".join(map(str, H)) + "]"
LEAN = f"""/-
RLT-007b stage 3, Lean comparison: GENERATED by scripts/gen-rlt007s3.py.
Core Lean only. Run: cd lean && lean RLT007S3.lean

LT unfolds to RLT's `lt` (∃z. S(z) + x = y), S(z) read as z + 1. What this
file proves, in Lean (none of it is an RLT derivation):
  * tail: RLT's statement tests/rlt007s3/accept/TAIL.rlt over ℕ, literally;
  * one_mod_iff: for p > 1, (∃k. h = p × k + 1) ↔ h % p = 1, the reading of
    the statement as "h is in the residue class 1 modulo p";
  * tail_mod: every element of H45 has a remainder other than 1 modulo every
    p > 212, prime or not;
  * bound_210_fails: the statement with 210 < p is false (p = 211, h = 212),
    so the bound 212 used here is above the least true bound 211;
  * h45_eq: the list below is the transcribed tuple (checked by the generator).
-/

namespace RLT007S3

def LT (x y : Nat) : Prop := ∃ z, (z + 1) + x = y

theorem lt_iff (x y : Nat) : LT x y ↔ x < y :=
  ⟨fun ⟨z, h⟩ => by omega, fun h => ⟨y - x - 1, by omega⟩⟩

def H45 : List Nat := {hl}

theorem h45_eq : H45.length = 45 ∧ H45.head? = some 0 ∧ H45.getLast? = some 212 ∧ 1 ∉ H45 := by decide

/-- One element: h ≤ 212, h ≠ 1 and 212 < p rule out h = p × k + 1. -/
theorem key (p h : Nat) (hp : 212 < p) (h1 : h ≤ 212) (h2 : h ≠ 1) : ¬ (∃ k, h = p * k + 1) := by
  rintro ⟨k, hk⟩
  cases k with
  | zero => simp at hk; omega
  | succ k =>
    have : p ≤ p * (k + 1) := Nat.le_mul_of_pos_right p (Nat.succ_pos k)
    omega

/-- RLT's TAIL (tests/rlt007s3/accept/TAIL.rlt), literally. -/
theorem tail : ∀ p, LT 212 p → {lean_conj(H)} := by
  intro p hp
  rw [lt_iff] at hp
  repeat' constructor
  all_goals exact key p _ hp (by decide) (by decide)

theorem one_mod_iff (p h : Nat) (hp : 1 < p) : (∃ k, h = p * k + 1) ↔ h % p = 1 := by
  constructor
  · rintro ⟨k, rfl⟩
    rw [Nat.add_comm, Nat.add_mul_mod_self_left]
    exact Nat.mod_eq_of_lt hp
  · intro h1
    refine ⟨h / p, ?_⟩
    have := Nat.div_add_mod h p
    omega

theorem small : ∀ h ∈ H45, h ≤ 212 ∧ h ≠ 1 := by decide

theorem tail_mod : ∀ p, 212 < p → ∀ h ∈ H45, h % p ≠ 1 := by
  intro p hp h hh h1
  exact key p h hp (small h hh).1 (small h hh).2 ((one_mod_iff p h (by omega)).mpr h1)

theorem bound_210_fails : ¬ (∀ p, LT 210 p → ∀ h ∈ H45, ¬ (∃ k, h = p * k + 1)) := by
  intro hall
  exact hall 211 ⟨0, rfl⟩ 212 (by decide) ⟨1, rfl⟩

end RLT007S3

#print axioms RLT007S3.lt_iff
#print axioms RLT007S3.h45_eq
#print axioms RLT007S3.key
#print axioms RLT007S3.tail
#print axioms RLT007S3.one_mod_iff
#print axioms RLT007S3.small
#print axioms RLT007S3.tail_mod
#print axioms RLT007S3.bound_210_fails
"""
OUT["lean/RLT007S3.lean"] = LEAN


# Statement record.
BASIS = "docs/RLT-007b-3-CORRESPONDENCE.md"
basis_sha = hashlib.sha256(open(BASIS, "rb").read()).hexdigest()
DEFS_SHA = hashlib.sha256(open("spec/definitions.rlt", "rb").read()).hexdigest()


def yq(x):
    return '"' + x.replace("\\", "\\\\").replace('"', '\\"') + '"'


checked = f"∀p. ((∃z. S(z) + {BOUND} = p) → {conj_w(H, 'p')})"
OUT["statements/H45-TAIL-OMITS.yaml"] = f"""# Statement record (RLT-007b stage 3): Lemma 12.1 (Axiom Math, September 3, 2026
# draft), the primes p ≥ 213: modulo every p with 212 < p, H45 omits the class 1.
# GENERATED by scripts/gen-rlt007s3.py; pins as `rlt check {ACC}/TAIL.rlt` reports them.
format: rlt-statement/1
id: H45-TAIL-OMITS
subject: "Lemma 12.1 (Axiom Math, September 3, 2026 draft), primes p ≥ 213: for every p with 212 < p, no element h of H45 satisfies h = (p × k) + 1"
source:
  registry: AXIOM-BGP212-2026-09-03
  commit: {T["pdf"]["sha256"]}
  excerpts:
    - {yq(ITEMS["h45-tuple"]["text"])}
    - {yq(ITEMS["admissible-definition"]["text"])}
    - {yq(ITEMS["large-primes"]["text"])}
kernel:
  proof: {ACC}/TAIL.rlt
  sha256: {sha(OUT[f"{ACC}/TAIL.rlt"])}
  convention: NAT-DEF
  definitions_sha256: {DEFS_SHA}
  axiom_set: ROBINSON-Q
  axiom_set_sha256: 4ad55192222576e3901abc4ee7ea5a6760b64ff212e8c55796ab9847e423b00c
  checked_goal: {yq(checked)}
  assumptions: []
semantic:
  status: partial
  assessed_by: "RLT-007b stage 3 review (docs/RLT-007b-3-CORRESPONDENCE.md)"
  date: 2026-11-12
  basis: {{path: {BASIS}, sha256: {basis_sha}}}
  assessed_goal: {yq(checked)}
  matches:
    - "Over ℕ: for every p > 212 and every listed h, h is not of the form p·k + 1. For p > 1 this says h mod p ≠ 1 (Lean one_mod_iff), so modulo every such p, prime or not, the elements of H45 omit the residue class 1. Restricted to primes p ≥ 213 this is the case of Lemma 12.1's admissibility that lies above the stage-2a classification (212 < p)."
  differs:
    - "The statement quantifies over every p > 212, not only primes; it applies to the primes in that range by instantiation. The paper argues p ≥ 47 by counting (45 elements, more than 45 classes); this record covers p ≥ 213 by the explicit class 1, and the primes 47 to 211 are stage 4. That 1 < p (the class is a residue below p) is not part of the statement; it follows from 212 < p in the assembly."
  lean:
    - {{file: lean/RLT007S3.lean, theorem: RLT007S3.tail}}
    - {{file: lean/RLT007S3.lean, theorem: RLT007S3.one_mod_iff}}
    - {{file: lean/RLT007S3.lean, theorem: RLT007S3.tail_mod}}
physical:
  applicability: not-applicable
  reason: "a statement in number theory"
open_questions: []
"""

if MODE == "--check":
    diff = []
    for path, text in sorted(OUT.items()):
        if not os.path.isfile(path) or open(path, encoding="utf-8", newline="").read() != text:
            diff.append(path)
    for d in (ACC, REJ):
        for f in sorted(os.listdir(d)) if os.path.isdir(d) else []:
            if f"{d}/{f}" not in OUT:
                diff.append(f"{d}/{f} (not produced by the generator)")
    for f in sorted(os.listdir("tests/rlt007s3")):
        if f not in ("accept", "reject", "expected.tsv"):
            diff.append(f"tests/rlt007s3/{f} (not produced by the generator)")
    if sorted(f for f in os.listdir("statements") if f.startswith("H45-TAIL")) != ["H45-TAIL-OMITS.yaml"]:
        diff.append("statements/H45-TAIL* (an extra or missing record)")
    for f in sorted(os.listdir("lean")):
        if f.startswith("RLT007S3") and f"lean/{f}" not in OUT:
            diff.append(f"lean/{f} (not produced by the generator)")
    for d in diff:
        print(f"DIFFERS  {d}")
    print(f"scripts/gen-rlt007s3.py --check: {len(OUT)} outputs, {len(diff)} differ")
    sys.exit(0 if not diff else 1)

for d in (ACC, REJ):
    os.makedirs(d, exist_ok=True)
    for old in os.listdir(d):
        os.remove(os.path.join(d, old))
for old in os.listdir("statements"):
    if old.startswith("H45-TAIL"):
        os.remove(os.path.join("statements", old))
for path, text in OUT.items():
    with open(path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(text)
print(f"{len(OUT)} outputs written: the theorem, {len(rows)} rejected cases, Lean comparison, 1 statement record")

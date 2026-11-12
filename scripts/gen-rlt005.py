#!/usr/bin/env python3
"""RLT-005: proofs that 8 is not matchable (McNew and Pomerance, arXiv:2604.05304v2, §1).

usage: python3 scripts/gen-rlt005.py RLT_BINARY      (from the repository root)

Untrusted generator. It writes derivations; `rlt check` verifies every step
against Robinson's Q. No result here is asserted by the generator: the
impossibility argument is a case analysis the kernel checks rule by rule.

Obligations (tests/rlt005/accept/):
  1. MD8  ∀d. (0 < d ∧ d ∣ 8 → d ∈ {1,2,4,8}) ∧ (d ∈ {1,2,4,8} → 0 < d ∧ d ∣ 8)
     MP4  ∀x. (0 < x ∧ x < 5 → x ∈ {1,2,3,4}) ∧ (x ∈ {1,2,3,4} → 0 < x ∧ x < 5)
  2. MO2, MO4, MO8  ∀x. (0 < x ∧ x < 5) ∧ cop(m, x) → x = 1 ∨ x = 3     (m = 2, 4, 8)
  3. MPH  ∀b c d. b, c, d ∈ {1, 3} → b = c ∨ b = d ∨ c = d
  4. MC   ∀a b c d. M(a, b, c, d) → ⊥
  5. MN8  (MD8 ∧ MP4) ∧ ¬∃a b c d. M(a, b, c, d)
with cop(m, x) := ∀k. ((0 < k ∧ k ∣ m) ∧ k ∣ x) → k = 1, written out with the
registered `<` and `∣`, and M the matching condition of §1 of the review
(0 < k because Q cannot refute 1 = 0 × j; docs/RLT-005-CORRESPONDENCE.md §3).
Rejected cases (tests/rlt005/reject/, tests/rlt005/expected.tsv) are
mutations of the accepted files; each must be refused with the reason pinned.
Supporting closed facts: B5 (enumeration below 5), 1 ∣ 8, 4 ∣ 8, 8 ∣ 8,
¬(3 ∣ 5), ¬(5 ∣ 3), ¬(6 ∣ 2), ¬(7 ∣ 1); 2 ∣ 2, 2 ∣ 4, 2 ∣ 8 are cited
from the existing library.
"""
import hashlib
import os
import re
import subprocess
import sys

rlt = sys.argv[1]
here = os.path.dirname(os.path.abspath(__file__))
OUT = "tests/rlt005/accept"


def load(script):
    src = open(os.path.join(here, script)).read()
    ns = {"rlt": rlt, "re": re, "subprocess": subprocess, "os": os, "hashlib": hashlib}
    exec(src[src.index("def S(t, k):"):src.index("accept, reject, expected = {}, {}, []")], ns)
    return ns


m3c = load("gen-m3c-tests.py")
m3d = load("gen-m3d-tests.py")


def sha_file(p):
    return hashlib.sha256(open(p, "rb").read()).hexdigest()


# ---------------------------------------------------------------------------
# Formulas in proof syntax, and their written NAT-DEF form.

def eq(a, b):
    return f"(= {a} {b})"


def AND(a, b):
    return f"(and {a} {b})"


def OR(a, b):
    return f"(or {a} {b})"


def IMP(a, b):
    return f"(imp {a} {b})"


def NOT(a):
    return f"(not {a})"


def lt(a, b):
    return f"(lt {a} {b})"


def dvd(a, b):
    return f"(dvd {a} {b})"


def ors(items):
    t = items[0]
    for x in items[1:]:
        t = OR(t, x)
    return t


def ands(items):
    t = items[0]
    for x in items[1:]:
        t = AND(t, x)
    return t


def cop(m, x):
    """x and m are coprime: every positive common divisor is 1. The positivity
    of k is the paper's convention for divisors (D(n) holds the positive
    divisors); in ℕ it changes nothing, since 0 divides only 0. Under
    Robinson's Q it is needed to prove coprimality of closed pairs at all:
    Q cannot refute 1 = 0 × j for a variable j."""
    return f"(forall k (imp (and (and (lt 0 k) (dvd k {m})) (dvd k {x})) (= k 1)))"


def rng(x):
    return AND(lt(0, x), lt(x, 5))


def inset(x, vals):
    return ors([eq(x, v) for v in vals])


def tokens(s):
    return re.findall(r"\(|\)|[^\s()]+", s)


def parse(s):
    toks = tokens(s)
    pos = 0

    def go():
        nonlocal pos
        t = toks[pos]
        pos += 1
        if t != "(":
            return t
        out = []
        while toks[pos] != ")":
            out.append(go())
        pos += 1
        return out

    return go()


def written(s):
    """Fully parenthesised NAT-DEF text for a proof-syntax formula."""
    def w(e, top=False):
        if isinstance(e, str):
            return "⊥" if e == "false" else e
        h = e[0]
        if h == "=":
            return f"{wt(e[1])} = {wt(e[2])}"
        if h == "lt":
            return f"{wt(e[1])} < {wt(e[2])}"
        if h == "dvd":
            return f"{wt(e[1])} ∣ {wt(e[2])}"
        if h == "not":
            return f"¬({w(e[1], True)})"
        if h in ("and", "or", "imp"):
            op = {"and": "∧", "or": "∨", "imp": "→"}[h]
            s = f"{w(e[1])} {op} {w(e[2])}"
            return s if top else f"({s})"
        if h in ("forall", "exists"):
            q = "∀" if h == "forall" else "∃"
            return f"{q}{e[1]}. {w(e[2], True)}" if top else f"({q}{e[1]}. {w(e[2], True)})"
        raise ValueError(e)

    def wt(t):
        if isinstance(t, str):
            return t
        if t[0] == "S":
            return f"S({wt(t[1])})"
        op = {"+": "+", "*": "×"}[t[0]]
        return f"({wt(t[1])} {op} {wt(t[2])})"

    return w(parse(s), True)


# ---------------------------------------------------------------------------
# A small proof builder.

class Proof:
    def __init__(self, pid):
        self.pid = pid
        self.steps = []
        self.n = 0
        self.cites = []
        self.params = []

    def fresh(self, p="s"):
        self.n += 1
        return f"{p}{self.n}"

    def add(self, claim, rule, label=None):
        label = label or self.fresh()
        self.steps.append(f"({label} {claim} {rule})")
        return label

    def param(self, name):
        if name not in self.params:
            self.params.append(name)
        return name

    def cite(self, label, path):
        if not any(l == label for l, _, _ in self.cites):
            self.cites.append((label, path, sha_file(path)))
        return label

    def text(self, goal_sexp, comment):
        c = ""
        if self.cites:
            c = "  (cites\n" + "\n".join(f"    ({l} \"{p}\" \"{h}\")" for l, p, h in self.cites) + ")\n"
        pr = f"  (params {' '.join(self.params)})\n" if self.params else ""
        body = "\n".join("    " + s for s in self.steps)
        return (f"; RLT-005 {self.pid}: {comment}\n; Generated by scripts/gen-rlt005.py; every step is checked by the kernel.\n"
                f"(rlt-proof\n  (id RLT005.{self.pid})\n  (convention NAT-DEF)\n  (goal \"{written(goal_sexp)}\")\n"
                f"  (mode unconditional)\n  (axioms ROBINSON-Q)\n{c}{pr}  (steps\n{body}))\n")


def closed_eq(P, equation, pre):
    """Steps proving a closed NAT-DEC equation with `rlt gen`; returns its label."""
    st, last = m3c["gen_steps"](equation, f"{pre}_")
    P.steps += st
    return last


def num(n):
    return str(n)


def lt_closed(P, x, y):
    """Closed proof of x < y (x < y numerals): ∃z. S(z) + x = y with z = y - x - 1."""
    e = closed_eq(P, f"{y} = {y - x} + {x}", P.fresh("g"))
    s = P.add(eq(f"(+ {y - x} {x})", y), f"(symm {e})")
    return P.add(lt(x, y), f"(exists-intro {y - x - 1} {s})")


def two_eq_one_false(P, e21):
    """From 2 = 1 derive ⊥ (Q1 then Q2)."""
    q1 = P.add(IMP(eq(2, 1), eq(1, 0)), "(inst q1_succ_inj (x 1) (y 0))")
    a = P.add(eq(1, 0), f"(imp-elim {q1} {e21})")
    b = P.add(eq(0, 1), f"(symm {a})")
    q2 = P.add(NOT(eq(0, 1)), "(inst q2_zero_ne_succ (x 0))")
    return P.add("false", f"(imp-elim {q2} {b})")


def lt0_zero_false(P, pos, x, x_is_zero):
    """From 0 < x and x = 0, derive ⊥."""
    s = P.add(lt(0, 0), f"(subst {x_is_zero} (at u (lt 0 u)) {pos})")
    z = P.param(P.fresh("z"))
    hz = P.fresh("hz")
    h = P.add(eq(f"(+ (S {z}) 0)", 0), f"(hyp {hz})")
    a = P.add(eq(f"(+ (S {z}) 0)", f"(S {z})"), f"(inst add_zero (x (S {z})))")
    b = P.add(eq(f"(S {z})", f"(+ (S {z}) 0)"), f"(symm {a})")
    c = P.add(eq(f"(S {z})", 0), f"(trans {b} {h})")
    d = P.add(eq(0, f"(S {z})"), f"(symm {c})")
    q2 = P.add(NOT(eq(0, f"(S {z})")), f"(inst q2_zero_ne_succ (x {z}))")
    f = P.add("false", f"(imp-elim {q2} {d})")
    return P.add("false", f"(exists-elim {s} {z} {hz} {f})")


def or_into(P, vals, x, j, given):
    """From x = vals[j], build the left-nested disjunction over vals."""
    t = eq(x, vals[0])
    lab = given if j == 0 else None
    for i in range(1, len(vals)):
        nt = OR(t, eq(x, vals[i]))
        if lab is None and i == j:
            lab = P.add(nt, f"(or-right {given})")
        elif lab is not None:
            lab = P.add(nt, f"(or-left {lab})")
        t = nt
    return lab


def or_cases_tree(P, vals, x, disj, case):
    """Case analysis on the left-nested disjunction `disj` (over vals);
    case(j, hyp_label) returns a label proving the common conclusion."""
    def go(k, label):
        # label proves the disjunction over vals[:k+1]
        if k == 0:
            return case(0, label)
        hl, hr = P.fresh("hl"), P.fresh("hr")
        left_disj = ors([eq(x, v) for v in vals[:k]])
        L = P.add(left_disj, f"(hyp {hl})")
        lres = go(k - 1, L)
        R = P.add(eq(x, vals[k]), f"(hyp {hr})")
        rres = case(k, R)
        concl = CONCL[0]
        return P.add(concl, f"(or-cases {label} {hl} {lres} {hr} {rres})")

    return go(len(vals) - 1, disj)


CONCL = [None]

# ---------------------------------------------------------------------------
# The files.

files = {}
paths = {}


def emit(pid, name, text):
    os.makedirs(OUT, exist_ok=True)
    p = f"{OUT}/{name}.rlt"
    with open(p, "w", encoding="utf-8") as f:
        f.write(text)
    paths[pid] = p
    return p


def rename(text, old_pid, new_pid, comment):
    text = re.sub(r"\(id M3[cd]\.[A-Za-z0-9_]+\)", f"(id RLT005.{new_pid})", text)
    text = re.sub(r"^; RLT-003 M3[cd] test .*$", f"; RLT-005 {new_pid}: {comment}\n; Generated by scripts/gen-rlt005.py (M3c/M3d schemes); every step is checked by the kernel.", text, flags=re.M)
    return text


# Supporting closed facts.
emit("B5", "B5_below_five", rename(m3d["enum_file"]("B5", 5, list(range(5))), "B5", "B5", "every x < 5 is 0, 1, 2, 3 or 4"))
for d, n in ((3, 5), (5, 3), (6, 2), (7, 1)):
    emit(f"ND_{d}_{n}", f"ND_{d}_{n}", rename(m3c["not_dvd"](f"ND_{d}_{n}", d, n), f"ND_{d}_{n}", f"ND_{d}_{n}", f"¬({d} ∣ {n})"))
for d in (1, 4, 8):
    P = Proof(f"PD_{d}_8")
    e = closed_eq(P, f"8 = {d} × {8 // d}", "g")
    P.add(dvd(d, 8), f"(exists-intro {8 // d} {e})", "G")
    emit(f"PD_{d}_8", f"PD_{d}_8", P.text(dvd(d, 8), f"{d} ∣ 8"))

LIB = {
    "D22": "tests/m3e/accept/PD_02.rlt",
    "D24": "tests/m3e/accept/PD_04.rlt",
    "D28": "tests/m3c/accept/CA01_two_divides_eight.rlt",
}

DIV8 = [1, 2, 4, 8]
P4V = [1, 2, 3, 4]

# 1a. MD8: the positive divisors of 8.
P = Proof("MD8")
a = P.param("a")
D8a = inset(a, DIV8)
CONCL[0] = D8a
hp = P.add(AND(lt(0, a), dvd(a, 8)), "(hyp hp)")
pos = P.add(lt(0, a), f"(and-left {hp})")
H = P.add(dvd(a, 8), f"(and-right {hp})")
k = P.param("k")
F = P.add(eq(8, f"(* {a} {k})"), "(hyp h0)")
kq3 = P.add(OR(eq(k, 0), f"(exists y (= {k} (S y)))"), f"(inst q3_zero_or_succ (x {k}))")
# k = 0: 8 = a × 0 = 0.
zk = P.add(eq(k, 0), "(hyp zk)")
ra = P.add(eq(a, a), f"(refl {a})")
c1 = P.add(eq(f"(* {a} {k})", f"(* {a} 0)"), f"(cong * {ra} {zk})")
c2 = P.add(eq(f"(* {a} 0)", 0), f"(inst mul_zero (x {a}))")
c3 = P.add(eq(f"(* {a} {k})", 0), f"(trans {c1} {c2})")
c4 = P.add(eq(8, 0), f"(trans {F} {c3})")
c5 = P.add(eq(0, 8), f"(symm {c4})")
c6 = P.add(NOT(eq(0, 8)), "(inst q2_zero_ne_succ (x 7))")
c7 = P.add("false", f"(imp-elim {c6} {c5})")
kzero = P.add(D8a, f"(false-elim {c7})")
# k = S(k1): 8 = a × k1 + a; peel successors of a.
k1 = P.param("k1")
sk = P.add(f"(exists y (= {k} (S y)))", "(hyp sk)")
wk = P.add(eq(k, f"(S {k1})"), "(hyp wk)")
X = f"(* {a} {k1})"
d1 = P.add(eq(f"(* {a} {k})", f"(* {a} (S {k1}))"), f"(cong * {ra} {wk})")
d2 = P.add(eq(f"(* {a} (S {k1}))", f"(+ {X} {a})"), f"(inst mul_succ (x {a}) (y {k1}))")
d3 = P.add(eq(f"(* {a} {k})", f"(+ {X} {a})"), f"(trans {d1} {d2})")
E = P.add(eq(8, f"(+ {X} {a})"), f"(trans {F} {d3})")
Pf = ra  # a = S^0(a)
ND = {3: 5, 5: 3, 6: 2, 7: 1}
for dd, nn in ND.items():
    P.cite(f"N{dd}", paths[f"ND_{dd}_{nn}"])


def peel_div(i, ai, Ei, Pi):
    """Ei : (8-i) = X + ai;  Pi : a = S^i(ai). Returns a label proving D8a."""
    r = 8 - i
    q3 = P.add(OR(eq(ai, 0), f"(exists y (= {ai} (S y)))"), f"(inst q3_zero_or_succ (x {ai}))")
    zh = P.fresh("zh")
    z = P.add(eq(ai, 0), f"(hyp {zh})")
    ai_eq = P.add(eq(a, i), f"(subst {z} (at u (= {a} {m3c['S']('u', i)})) {Pi})")
    if i == 0:
        f = lt0_zero_false(P, pos, a, ai_eq)
        zres = P.add(D8a, f"(false-elim {f})")
    elif i in DIV8:
        zres = or_into(P, DIV8, a, DIV8.index(i), ai_eq)
    else:
        rx = P.add(eq(X, X), f"(refl {X})")
        e1 = P.add(eq(f"(+ {X} {ai})", f"(+ {X} 0)"), f"(cong + {rx} {z})")
        e2 = P.add(eq(f"(+ {X} 0)", X), f"(inst add_zero (x {X}))")
        e3 = P.add(eq(f"(+ {X} {ai})", X), f"(trans {e1} {e2})")
        e4 = P.add(eq(r, X), f"(trans {Ei} {e3})")
        rk = P.add(eq(k1, k1), f"(refl {k1})")
        e5 = P.add(eq(X, f"(* {i} {k1})"), f"(cong * {ai_eq} {rk})")
        e6 = P.add(eq(r, f"(* {i} {k1})"), f"(trans {e4} {e5})")
        e7 = P.add(dvd(i, r), f"(exists-intro {k1} {e6})")
        n = P.add(NOT(dvd(i, r)), f"(cite N{i})")
        f = P.add("false", f"(imp-elim {n} {e7})")
        zres = P.add(D8a, f"(false-elim {f})")
    sh, wh = P.fresh("sh"), P.fresh("wh")
    nxt = P.param(f"a{i + 1}")
    s = P.add(f"(exists y (= {ai} (S y)))", f"(hyp {sh})")
    w = P.add(eq(ai, f"(S {nxt})"), f"(hyp {wh})")
    Pn = P.add(eq(a, m3c["S"](nxt, i + 1)), f"(subst {w} (at u (= {a} {m3c['S']('u', i)})) {Pi})")
    rx = P.add(eq(X, X), f"(refl {X})")
    g1 = P.add(eq(f"(+ {X} {ai})", f"(+ {X} (S {nxt}))"), f"(cong + {rx} {w})")
    g2 = P.add(eq(f"(+ {X} (S {nxt}))", f"(S (+ {X} {nxt}))"), f"(inst add_succ (x {X}) (y {nxt}))")
    g3 = P.add(eq(f"(+ {X} {ai})", f"(S (+ {X} {nxt}))"), f"(trans {g1} {g2})")
    g4 = P.add(eq(r, f"(S (+ {X} {nxt}))"), f"(trans {Ei} {g3})")
    if r == 0:
        g5 = P.add(NOT(eq(0, f"(S (+ {X} {nxt}))")), f"(inst q2_zero_ne_succ (x (+ {X} {nxt})))")
        f = P.add("false", f"(imp-elim {g5} {g4})")
        inner = P.add(D8a, f"(false-elim {f})")
    else:
        q1 = P.add(IMP(eq(r, f"(S (+ {X} {nxt}))"), eq(r - 1, f"(+ {X} {nxt})")),
                   f"(inst q1_succ_inj (x {r - 1}) (y (+ {X} {nxt})))")
        En = P.add(eq(r - 1, f"(+ {X} {nxt})"), f"(imp-elim {q1} {g4})")
        inner = peel_div(i + 1, nxt, En, Pn)
    sres = P.add(D8a, f"(exists-elim {s} {nxt} {wh} {inner})")
    return P.add(D8a, f"(or-cases {q3} {zh} {zres} {sh} {sres})")


ksucc_inner = peel_div(0, a, E, Pf)
ksucc = P.add(D8a, f"(exists-elim {sk} {k1} wk {ksucc_inner})")
kcases = P.add(D8a, f"(or-cases {kq3} zk {kzero} sk {ksucc})")
fwd = P.add(D8a, f"(exists-elim {H} {k} h0 {kcases})")
fwd_i = P.add(IMP(AND(lt(0, a), dvd(a, 8)), D8a), f"(imp-intro hp {fwd})")
# Backward: each listed value is a positive divisor of 8.
PDC = {1: ("P1", paths["PD_1_8"]), 2: ("D28", LIB["D28"]), 4: ("P4", paths["PD_4_8"]), 8: ("P8", paths["PD_8_8"])}
target = AND(lt(0, a), dvd(a, 8))
CONCL[0] = target
hb = P.add(D8a, "(hyp hb)")


def back_case(j, lab):
    v = DIV8[j]
    l, pth = PDC[v]
    P.cite(l, pth)
    dv = P.add(dvd(v, 8), f"(cite {l})")
    lt0 = lt_closed(P, 0, v)
    both = P.add(AND(lt(0, v), dvd(v, 8)), f"(and-intro {lt0} {dv})")
    sy = P.add(eq(v, a), f"(symm {lab})")
    return P.add(target, f"(subst {sy} (at u (and (lt 0 u) (dvd u 8))) {both})")


bres = or_cases_tree(P, DIV8, a, hb, back_case)
bwd_i = P.add(IMP(D8a, target), f"(imp-intro hb {bres})")
both = P.add(AND(IMP(target, D8a), IMP(D8a, target)), f"(and-intro {fwd_i} {bwd_i})")
MD8 = f"(forall d (and (imp (and (lt 0 d) (dvd d 8)) {inset('d', DIV8)}) (imp {inset('d', DIV8)} (and (lt 0 d) (dvd d 8)))))"
P.add(MD8, f"(forall-intro {a} {both})", "G")
emit("MD8", "MD8_divisors_of_eight", P.text(MD8, "the positive divisors of 8 are 1, 2, 4 and 8"))

# 1b. MP4: the partner set {1, 2, 3, 4}.
P = Proof("MP4")
x = P.param("px")
P4x = inset(x, P4V)
CONCL[0] = P4x
P.cite("B5", paths["B5"])
hr = P.add(rng(x), "(hyp hr)")
pos = P.add(lt(0, x), f"(and-left {hr})")
lt5 = P.add(lt(x, 5), f"(and-right {hr})")
B5 = f"(forall x (imp (lt x 5) {inset('x', [0, 1, 2, 3, 4])}))"
b5 = P.add(B5, "(cite B5)")
b5x = P.add(IMP(lt(x, 5), inset(x, [0, 1, 2, 3, 4])), f"(forall-elim {b5} {x})")
e04 = P.add(inset(x, [0, 1, 2, 3, 4]), f"(imp-elim {b5x} {lt5})")


def fwd_case(j, lab):
    if j == 0:
        f = lt0_zero_false(P, pos, x, lab)
        return P.add(P4x, f"(false-elim {f})")
    return or_into(P, P4V, x, j - 1, lab)


fres = or_cases_tree(P, [0, 1, 2, 3, 4], x, e04, fwd_case)
fwd_i = P.add(IMP(rng(x), P4x), f"(imp-intro hr {fres})")
CONCL[0] = rng(x)
hb = P.add(P4x, "(hyp hb)")


def p4_back(j, lab):
    v = P4V[j]
    l0 = lt_closed(P, 0, v)
    l5 = lt_closed(P, v, 5)
    both = P.add(AND(lt(0, v), lt(v, 5)), f"(and-intro {l0} {l5})")
    sy = P.add(eq(v, x), f"(symm {lab})")
    return P.add(rng(x), f"(subst {sy} (at u (and (lt 0 u) (lt u 5))) {both})")


bres = or_cases_tree(P, P4V, x, hb, p4_back)
bwd_i = P.add(IMP(P4x, rng(x)), f"(imp-intro hb {bres})")
both = P.add(AND(IMP(rng(x), P4x), IMP(P4x, rng(x))), f"(and-intro {fwd_i} {bwd_i})")
MP4 = f"(forall x (and (imp (and (lt 0 x) (lt x 5)) {inset('x', P4V)}) (imp {inset('x', P4V)} (and (lt 0 x) (lt x 5)))))"
P.add(MP4, f"(forall-intro {x} {both})", "G")
emit("MP4", "MP4_partners_one_to_four", P.text(MP4, "the partner set {1, 2, 3, 4} is the x with 0 < x < 5"))

# 2. MO2, MO4, MO8: a partner of an even divisor is 1 or 3.
ODD = {}
for m in (2, 4, 8):
    P = Proof(f"MO{m}")
    x = P.param("px")
    G13 = OR(eq(x, 1), eq(x, 3))
    CONCL[0] = G13
    P.cite("MP4", paths["MP4"])
    for l in ("D22", "D24") + (("D28",) if m == 8 else ()):
        P.cite(l, LIB[l])
    h = P.add(AND(rng(x), cop(m, x)), "(hyp hm)")
    r = P.add(rng(x), f"(and-left {h})")
    c = P.add(cop(m, x), f"(and-right {h})")
    mp4 = P.add(MP4, "(cite MP4)")
    mp4x = P.add(AND(IMP(rng(x), inset(x, P4V)), IMP(inset(x, P4V), rng(x))), f"(forall-elim {mp4} {x})")
    imp = P.add(IMP(rng(x), inset(x, P4V)), f"(and-left {mp4x})")
    in4 = P.add(inset(x, P4V), f"(imp-elim {imp} {r})")
    c2 = P.add(IMP(AND(AND(lt(0, 2), dvd(2, m)), dvd(2, x)), eq(2, 1)), f"(forall-elim {c} 2)")
    dm0 = P.add(dvd(2, m), "(cite D%d%d)" % (2, m))
    l02 = lt_closed(P, 0, 2)
    dm = P.add(AND(lt(0, 2), dvd(2, m)), f"(and-intro {l02} {dm0})")

    def odd_case(j, lab, m=m, x=x, c2=c2, dm=dm, G13=G13):
        v = P4V[j]
        if v == 1:
            return P.add(G13, f"(or-left {lab})")
        if v == 3:
            return P.add(G13, f"(or-right {lab})")
        dv = P.add(dvd(2, v), f"(cite D2{v})")
        sy = P.add(eq(v, x), f"(symm {lab})")
        dx = P.add(dvd(2, x), f"(subst {sy} (at u (dvd 2 u)) {dv})")
        both = P.add(AND(AND(lt(0, 2), dvd(2, m)), dvd(2, x)), f"(and-intro {dm} {dx})")
        e21 = P.add(eq(2, 1), f"(imp-elim {c2} {both})")
        f = two_eq_one_false(P, e21)
        return P.add(G13, f"(false-elim {f})")

    res = or_cases_tree(P, P4V, x, in4, odd_case)
    i = P.add(IMP(AND(rng(x), cop(m, x)), G13), f"(imp-intro hm {res})")
    MO = f"(forall x (imp (and (and (lt 0 x) (lt x 5)) {cop(m, 'x')}) (or (= x 1) (= x 3))))"
    P.add(MO, f"(forall-intro {x} {i})", "G")
    ODD[m] = MO
    emit(f"MO{m}", f"MO{m}_partner_of_{m}_is_odd", P.text(MO, f"a partner of {m} in [1, 4] coprime to {m} is 1 or 3"))

# 3. MPH: three values in {1, 3} include an equal pair.
P = Proof("MPH")
b, c, d = P.param("pb"), P.param("pc"), P.param("pd")
B = lambda v: OR(eq(v, 1), eq(v, 3))
Q = OR(OR(eq(b, c), eq(b, d)), eq(c, d))
h = P.add(AND(AND(B(b), B(c)), B(d)), "(hyp hh)")
bc = P.add(AND(B(b), B(c)), f"(and-left {h})")
hb_ = P.add(B(b), f"(and-left {bc})")
hc_ = P.add(B(c), f"(and-right {bc})")
hd_ = P.add(B(d), f"(and-right {h})")


def split(lab, v, k):
    """or-cases on v = 1 ∨ v = 3; k(val, eq_label) proves Q."""
    h1, h3 = P.fresh("h"), P.fresh("h")
    e1 = P.add(eq(v, 1), f"(hyp {h1})")
    r1 = k(1, e1)
    e3 = P.add(eq(v, 3), f"(hyp {h3})")
    r3 = k(3, e3)
    return P.add(Q, f"(or-cases {lab} {h1} {r1} {h3} {r3})")


def same(u, eu, w, ew, val):
    sw = P.add(eq(val, w), f"(symm {ew})")
    return P.add(eq(u, w), f"(trans {eu} {sw})")


def leaf(vb, eb, vc, ec, vd, ed):
    if vb == vc:
        t = same(b, eb, c, ec, vb)
        l1 = P.add(OR(eq(b, c), eq(b, d)), f"(or-left {t})")
        return P.add(Q, f"(or-left {l1})")
    if vb == vd:
        t = same(b, eb, d, ed, vb)
        l1 = P.add(OR(eq(b, c), eq(b, d)), f"(or-right {t})")
        return P.add(Q, f"(or-left {l1})")
    t = same(c, ec, d, ed, vc)
    return P.add(Q, f"(or-right {t})")


res = split(hb_, b, lambda vb, eb: split(hc_, c, lambda vc, ec: split(hd_, d, lambda vd, ed: leaf(vb, eb, vc, ec, vd, ed))))
i = P.add(IMP(AND(AND(B(b), B(c)), B(d)), Q), f"(imp-intro hh {res})")
i3 = P.add(f"(forall d {IMP(AND(AND(B(b), B(c)), B('d')), OR(OR(eq(b, c), eq(b, 'd')), eq(c, 'd')))})", f"(forall-intro {d} {i})")
i2 = P.add(f"(forall c (forall d {IMP(AND(AND(B(b), B('c')), B('d')), OR(OR(eq(b, 'c'), eq(b, 'd')), eq('c', 'd')))}))", f"(forall-intro {c} {i3})")
MPH = f"(forall b (forall c (forall d {IMP(AND(AND(B('b'), B('c')), B('d')), OR(OR(eq('b', 'c'), eq('b', 'd')), eq('c', 'd')))})))"
P.add(MPH, f"(forall-intro {b} {i2})", "G")
emit("MPH", "MPH_three_values_in_one_three", P.text(MPH, "three values in {1, 3} include an equal pair"))


# The matching condition M(a, b, c, d): partners of the divisors 1, 2, 4, 8.
def Mfor(a, b, c, d):
    ranges = ands([rng(a), rng(b), rng(c), rng(d)])
    distinct = ands([NOT(eq(a, b)), NOT(eq(a, c)), NOT(eq(a, d)), NOT(eq(b, c)), NOT(eq(b, d)), NOT(eq(c, d))])
    cops = ands([cop(1, a), cop(2, b), cop(4, c), cop(8, d)])
    return AND(AND(ranges, distinct), cops)


# 4. MC: every assignment satisfying M is contradictory.
P = Proof("MC")
for l, f in (("MO2", "MO2"), ("MO4", "MO4"), ("MO8", "MO8"), ("MPH", "MPH")):
    P.cite(l, paths[f])
a, b, c, d = (P.param("p" + v) for v in "abcd")
M = Mfor(a, b, c, d)
RI = [rng(a), rng(b), rng(c), rng(d)]
CI = [cop(1, a), cop(2, b), cop(4, c), cop(8, d)]
DI = [NOT(eq(a, b)), NOT(eq(a, c)), NOT(eq(a, d)), NOT(eq(b, c)), NOT(eq(b, d)), NOT(eq(c, d))]
hm = P.add(M, "(hyp hm)")
rd = P.add(AND(ands(RI), ands(DI)), f"(and-left {hm})")
cops = P.add(ands(CI), f"(and-right {hm})")
R = P.add(ands(RI), f"(and-left {rd})")
D = P.add(ands(DI), f"(and-right {rd})")


def proj(label, items, i):
    """The i-th conjunct of a left-nested conjunction of items."""
    lab = label
    for j in range(len(items) - 1, i, -1):
        lab = P.add(ands(items[:j]), f"(and-left {lab})")
    if i == 0:
        return lab
    return P.add(items[i], f"(and-right {lab})")


odd = {}
for v, m, i in ((b, 2, 1), (c, 4, 2), (d, 8, 3)):
    rv = proj(R, RI, i)
    cv = proj(cops, CI, i)
    both = P.add(AND(rng(v), cop(m, v)), f"(and-intro {rv} {cv})")
    mo = P.add(ODD[m], f"(cite MO{m})")
    inst = P.add(IMP(AND(rng(v), cop(m, v)), OR(eq(v, 1), eq(v, 3))), f"(forall-elim {mo} {v})")
    odd[v] = P.add(OR(eq(v, 1), eq(v, 3)), f"(imp-elim {inst} {both})")
bcd = P.add(AND(AND(OR(eq(b, 1), eq(b, 3)), OR(eq(c, 1), eq(c, 3))), OR(eq(d, 1), eq(d, 3))),
            f"(and-intro {P.add(AND(OR(eq(b, 1), eq(b, 3)), OR(eq(c, 1), eq(c, 3))), f'(and-intro {odd[b]} {odd[c]})')} {odd[d]})")
mph = P.add(MPH, "(cite MPH)")
B = lambda v: OR(eq(v, 1), eq(v, 3))
p1 = P.add(f"(forall c (forall d {IMP(AND(AND(B(b), B('c')), B('d')), OR(OR(eq(b, 'c'), eq(b, 'd')), eq('c', 'd')))}))", f"(forall-elim {mph} {b})")
p2 = P.add(f"(forall d {IMP(AND(AND(B(b), B(c)), B('d')), OR(OR(eq(b, c), eq(b, 'd')), eq(c, 'd')))})", f"(forall-elim {p1} {c})")
p3 = P.add(IMP(AND(AND(B(b), B(c)), B(d)), OR(OR(eq(b, c), eq(b, d)), eq(c, d))), f"(forall-elim {p2} {d})")
pq = P.add(OR(OR(eq(b, c), eq(b, d)), eq(c, d)), f"(imp-elim {p3} {bcd})")
nbc = proj(D, DI, 3)
nbd = proj(D, DI, 4)
ncd = proj(D, DI, 5)
h1, h2, h3, h12 = "q1h", "q2h", "q3h", "q12h"
e12 = P.add(OR(eq(b, c), eq(b, d)), f"(hyp {h12})")
ebc = P.add(eq(b, c), f"(hyp {h1})")
f1 = P.add("false", f"(imp-elim {nbc} {ebc})")
ebd = P.add(eq(b, d), f"(hyp {h2})")
f2 = P.add("false", f"(imp-elim {nbd} {ebd})")
f12 = P.add("false", f"(or-cases {e12} {h1} {f1} {h2} {f2})")
ecd = P.add(eq(c, d), f"(hyp {h3})")
f3 = P.add("false", f"(imp-elim {ncd} {ecd})")
ff = P.add("false", f"(or-cases {pq} {h12} {f12} {h3} {f3})")
im = P.add(IMP(M, "false"), f"(imp-intro hm {ff})")
MC = f"(forall a (forall b (forall c (forall d (imp {Mfor('a', 'b', 'c', 'd')} false)))))"
g3 = P.add(f"(forall d (imp {Mfor(a, b, c, 'd')} false))", f"(forall-intro {d} {im})")
g2 = P.add(f"(forall c (forall d (imp {Mfor(a, b, 'c', 'd')} false)))", f"(forall-intro {c} {g3})")
g1 = P.add(f"(forall b (forall c (forall d (imp {Mfor(a, 'b', 'c', 'd')} false))))", f"(forall-intro {b} {g2})")
P.add(MC, f"(forall-intro {a} {g1})", "G")
emit("MC", "MC_no_assignment", P.text(MC, "every assignment of partners satisfying the matching conditions is contradictory"))

# Supporting: 1 is coprime to every m (positive divisors of 1 are 1).
P = Proof("MCO")
m_, k_, j_, j1, z_, z1 = (P.param(v) for v in ("pm", "pk", "pj", "pj1", "pz", "pz1"))
G1 = eq(k_, 1)
h = P.add(AND(AND(lt(0, k_), dvd(k_, m_)), dvd(k_, 1)), "(hyp hc)")
hl = P.add(AND(lt(0, k_), dvd(k_, m_)), f"(and-left {h})")
pos = P.add(lt(0, k_), f"(and-left {hl})")
d1 = P.add(dvd(k_, 1), f"(and-right {h})")
ez = P.add(eq(f"(+ (S {z_}) 0)", k_), "(hyp hz)")
az = P.add(eq(f"(+ (S {z_}) 0)", f"(S {z_})"), f"(inst add_zero (x (S {z_})))")
kz0 = P.add(eq(f"(S {z_})", f"(+ (S {z_}) 0)"), f"(symm {az})")
kz1 = P.add(eq(f"(S {z_})", k_), f"(trans {kz0} {ez})")
kz = P.add(eq(k_, f"(S {z_})"), f"(symm {kz1})")
f1 = P.add(eq(1, f"(* {k_} {j_})"), "(hyp hj)")
jq3 = P.add(OR(eq(j_, 0), f"(exists y (= {j_} (S y)))"), f"(inst q3_zero_or_succ (x {j_}))")
jz = P.add(eq(j_, 0), "(hyp jz)")
rk = P.add(eq(k_, k_), f"(refl {k_})")
a1 = P.add(eq(f"(* {k_} {j_})", f"(* {k_} 0)"), f"(cong * {rk} {jz})")
a2 = P.add(eq(f"(* {k_} 0)", 0), f"(inst mul_zero (x {k_}))")
a3 = P.add(eq(f"(* {k_} {j_})", 0), f"(trans {a1} {a2})")
a4 = P.add(eq(1, 0), f"(trans {f1} {a3})")
a5 = P.add(eq(0, 1), f"(symm {a4})")
a6 = P.add(NOT(eq(0, 1)), "(inst q2_zero_ne_succ (x 0))")
a7 = P.add("false", f"(imp-elim {a6} {a5})")
jzero = P.add(G1, f"(false-elim {a7})")
js = P.add(f"(exists y (= {j_} (S y)))", "(hyp js)")
jw = P.add(eq(j_, f"(S {j1})"), "(hyp jw)")
Y = f"(* {k_} {j1})"
b1 = P.add(eq(f"(* {k_} {j_})", f"(* {k_} (S {j1}))"), f"(cong * {rk} {jw})")
b2 = P.add(eq(f"(* {k_} (S {j1}))", f"(+ {Y} {k_})"), f"(inst mul_succ (x {k_}) (y {j1}))")
ry = P.add(eq(Y, Y), f"(refl {Y})")
b3 = P.add(eq(f"(+ {Y} {k_})", f"(+ {Y} (S {z_}))"), f"(cong + {ry} {kz})")
b4 = P.add(eq(f"(+ {Y} (S {z_}))", f"(S (+ {Y} {z_}))"), f"(inst add_succ (x {Y}) (y {z_}))")
b5 = P.add(eq(f"(* {k_} {j_})", f"(S (+ {Y} {z_}))"), f"(trans {b1} {P.add(eq(f'(* {k_} (S {j1}))', f'(S (+ {Y} {z_}))'), f'(trans {b2} {P.add(eq(f"(+ {Y} {k_})", f"(S (+ {Y} {z_}))"), f"(trans {b3} {b4})")})')})")
b6 = P.add(eq(1, f"(S (+ {Y} {z_}))"), f"(trans {f1} {b5})")
q1 = P.add(IMP(eq(1, f"(S (+ {Y} {z_}))"), eq(0, f"(+ {Y} {z_})")), f"(inst q1_succ_inj (x 0) (y (+ {Y} {z_})))")
b7 = P.add(eq(0, f"(+ {Y} {z_})"), f"(imp-elim {q1} {b6})")
zq3 = P.add(OR(eq(z_, 0), f"(exists y (= {z_} (S y)))"), f"(inst q3_zero_or_succ (x {z_}))")
zz = P.add(eq(z_, 0), "(hyp zz)")
c1 = P.add(eq(k_, 1), f"(subst {zz} (at u (= {k_} (S u))) {kz})")
zs = P.add(f"(exists y (= {z_} (S y)))", "(hyp zs)")
zw = P.add(eq(z_, f"(S {z1})"), "(hyp zw)")
d1_ = P.add(eq(f"(+ {Y} {z_})", f"(+ {Y} (S {z1}))"), f"(cong + {ry} {zw})")
d2 = P.add(eq(f"(+ {Y} (S {z1}))", f"(S (+ {Y} {z1}))"), f"(inst add_succ (x {Y}) (y {z1}))")
d3 = P.add(eq(f"(+ {Y} {z_})", f"(S (+ {Y} {z1}))"), f"(trans {d1_} {d2})")
d4 = P.add(eq(0, f"(S (+ {Y} {z1}))"), f"(trans {b7} {d3})")
d5 = P.add(NOT(eq(0, f"(S (+ {Y} {z1}))")), f"(inst q2_zero_ne_succ (x (+ {Y} {z1})))")
d6 = P.add("false", f"(imp-elim {d5} {d4})")
d7 = P.add(G1, f"(false-elim {d6})")
zsucc = P.add(G1, f"(exists-elim {zs} {z1} zw {d7})")
zc = P.add(G1, f"(or-cases {zq3} zz {c1} zs {zsucc})")
jsucc = P.add(G1, f"(exists-elim {js} {j1} jw {zc})")
jc = P.add(G1, f"(or-cases {jq3} jz {jzero} js {jsucc})")
ej = P.add(G1, f"(exists-elim {d1} {j_} hj {jc})")
ezr = P.add(G1, f"(exists-elim {pos} {z_} hz {ej})")
im = P.add(IMP(AND(AND(lt(0, k_), dvd(k_, m_)), dvd(k_, 1)), G1), f"(imp-intro hc {ezr})")
fk = P.add(f"(forall k {IMP(AND(AND(lt(0, 'k'), dvd('k', m_)), dvd('k', 1)), eq('k', 1))})", f"(forall-intro {k_} {im})")
MCO = f"(forall m {cop('m', 1)})"
P.add(MCO, f"(forall-intro {m_} {fk})", "G")
emit("MCO", "MCO_one_is_coprime_to_all", P.text(MCO, "1 is coprime to every m: its only positive divisor is 1"))

# Non-vacuity: the range and coprimality conditions alone are satisfiable
# (every partner 1), so the obstruction is distinctness, as in the paper.
P = Proof("MNV")
P.cite("MCO", paths["MCO"])
mco = P.add(MCO, "(cite MCO)")
l01, l15 = lt_closed(P, 0, 1), lt_closed(P, 1, 5)
r1 = P.add(rng(1), f"(and-intro {l01} {l15})")
rs = P.add(ands([rng(1)] * 2), f"(and-intro {r1} {r1})")
rs = P.add(ands([rng(1)] * 3), f"(and-intro {rs} {r1})")
rs = P.add(ands([rng(1)] * 4), f"(and-intro {rs} {r1})")
cs = [P.add(cop(m, 1), f"(forall-elim {mco} {m})") for m in (1, 2, 4, 8)]
cc = P.add(ands([cop(1, 1), cop(2, 1)]), f"(and-intro {cs[0]} {cs[1]})")
cc = P.add(ands([cop(1, 1), cop(2, 1), cop(4, 1)]), f"(and-intro {cc} {cs[2]})")
cc = P.add(ands([cop(1, 1), cop(2, 1), cop(4, 1), cop(8, 1)]), f"(and-intro {cc} {cs[3]})")


def RC(a, b, c, d):
    return AND(ands([rng(a), rng(b), rng(c), rng(d)]), ands([cop(1, a), cop(2, b), cop(4, c), cop(8, d)]))


w = P.add(RC(1, 1, 1, 1), f"(and-intro {rs} {cc})")
w = P.add(f"(exists d {RC(1, 1, 1, 'd')})", f"(exists-intro 1 {w})")
w = P.add(f"(exists c (exists d {RC(1, 1, 'c', 'd')}))", f"(exists-intro 1 {w})")
w = P.add(f"(exists b (exists c (exists d {RC(1, 'b', 'c', 'd')})))", f"(exists-intro 1 {w})")
MNV = f"(exists a (exists b (exists c (exists d {RC('a', 'b', 'c', 'd')}))))"
P.add(MNV, f"(exists-intro 1 {w})", "G")
emit("MNV", "MNV_conditions_without_distinctness", P.text(MNV, "the range and coprimality conditions alone are satisfiable (all partners 1)"))

# 5. MN8: 8 is not matchable, with its divisor and partner sets.
P = Proof("MN8")
for l in ("MD8", "MP4", "MC"):
    P.cite(l, paths[l])
md8 = P.add(MD8, "(cite MD8)")
mp4 = P.add(MP4, "(cite MP4)")
mc = P.add(MC, "(cite MC)")
a, b, c, d = (P.param("p" + v) for v in "abcd")
EX = f"(exists a (exists b (exists c (exists d {Mfor('a', 'b', 'c', 'd')}))))"
hx = P.add(EX, "(hyp hx)")
ha = P.add(f"(exists b (exists c (exists d {Mfor(a, 'b', 'c', 'd')})))", "(hyp ha)")
hb = P.add(f"(exists c (exists d {Mfor(a, b, 'c', 'd')}))", "(hyp hb)")
hc = P.add(f"(exists d {Mfor(a, b, c, 'd')})", "(hyp hc)")
hd = P.add(Mfor(a, b, c, d), "(hyp hd)")
i1 = P.add(f"(forall b (forall c (forall d (imp {Mfor(a, 'b', 'c', 'd')} false))))", f"(forall-elim {mc} {a})")
i2 = P.add(f"(forall c (forall d (imp {Mfor(a, b, 'c', 'd')} false)))", f"(forall-elim {i1} {b})")
i3 = P.add(f"(forall d (imp {Mfor(a, b, c, 'd')} false))", f"(forall-elim {i2} {c})")
i4 = P.add(IMP(Mfor(a, b, c, d), "false"), f"(forall-elim {i3} {d})")
f = P.add("false", f"(imp-elim {i4} {hd})")
f = P.add("false", f"(exists-elim {hc} {d} hd {f})")
f = P.add("false", f"(exists-elim {hb} {c} hc {f})")
f = P.add("false", f"(exists-elim {ha} {b} hb {f})")
f = P.add("false", f"(exists-elim {hx} {a} ha {f})")
n = P.add(NOT(EX), f"(imp-intro hx {f})")
sets = P.add(AND(MD8, MP4), f"(and-intro {md8} {mp4})")
MN8 = AND(AND(MD8, MP4), NOT(EX))
P.add(MN8, f"(and-intro {sets} {n})", "G")
emit("MN8", "MN8_eight_is_not_matchable", P.text(MN8, "8 is not matchable (McNew and Pomerance, arXiv:2604.05304v2, §1)"))
print("\n".join(f"{k}: {v}" for k, v in paths.items()))


# ---------------------------------------------------------------------------
# Rejected cases: mutations of the accepted files.

REJ = "tests/rlt005/reject"
os.makedirs(REJ, exist_ok=True)
for old in os.listdir(REJ):
    os.remove(os.path.join(REJ, old))


def acc(name):
    return open(f"{OUT}/{name}.rlt", encoding="utf-8").read()


def goal_of(text):
    return re.search(r'\(goal "(.*)"\)', text).group(1)


def must(text, old, new, count=1):
    assert old in text, old
    return text.replace(old, new, count)


def head(text, note):
    return text.replace("(rlt-proof", f"; REJECT CASE: {note}\n(rlt-proof", 1)


DIST = "(and (and (and (and (and (not (= pa pb)) (not (= pa pc))) (not (= pa pd))) (not (= pb pc))) (not (= pb pd))) (not (= pc pd)))"
DISTW = "(((((¬(a = b) ∧ ¬(a = c)) ∧ ¬(a = d)) ∧ ¬(b = c)) ∧ ¬(b = d)) ∧ ¬(c = d))"
mc, mn = acc("MC_no_assignment"), acc("MN8_eight_is_not_matchable")
mc_cite = re.search(r'\(MC "([^"]+)" "([0-9a-f]+)"\)', mn)
rejects = []


def rej(name, text, code, reason):
    with open(f"{REJ}/{name}.rlt", "w", encoding="utf-8") as fh:
        fh.write(text)
    rejects.append(f"{REJ}/{name}.rlt\t{code}\t{reason}")


rr01 = mc.replace(DIST, "(= pa pa)").replace(DISTW, "a = a")
assert rr01.count("(= pa pa)") == mc.count(DIST) > 0
rej("RR01_duplicate_partners_allowed",
    head(rr01, "MC with distinctness replaced by a = a throughout: duplicate partners allowed"),
    1, "REJECT at step s37 — and-left: premise is not a conjunction: `pa = pa`")
rej("RR02_partner_of_2_may_be_2",
    head(must(acc("MO2_partner_of_2_is_odd"), "→ (x = 1 ∨ x = 3)\")", "→ (x = 1 ∨ x = 2)\")"),
         "invalid coprimality claim: a partner of 2 may be 2"),
    1, "REJECT — last step proves")
rej("RR03_coprime_without_positive_k",
    head(must(acc("MCO_one_is_coprime_to_all"), "((0 < k ∧ k ∣ m) ∧ k ∣ 1)", "(k ∣ m ∧ k ∣ 1)"),
         "coprimality without 0 < k, which Q cannot establish for k = 0"),
    1, "REJECT — last step proves")
rej("RR04_undeclared_assumption",
    head(must(mc, "(hyp hm)", "(hyp hq)"), "a step uses an assumption the goal does not declare"),
    1, "REJECT at step s1 — assumption `hq` is undeclared in the goal")
md = acc("MD8_divisors_of_eight")
g = goal_of(md)
rej("RR05_divisor_eight_omitted",
    head(must(md, g, must(g, "(((d = 1 ∨ d = 2) ∨ d = 4) ∨ d = 8))", "((d = 1 ∨ d = 2) ∨ d = 4))")),
         "wrong divisor set: 8 omitted"),
    1, "REJECT — last step proves")
rej("RR06_cited_hash_stale",
    head(must(mn, mc_cite.group(2), "f" * 64), "a citation records a stale SHA-256"),
    2, "MALFORMED — citation `MC`: tests/rlt005/accept/MC_no_assignment.rlt has SHA-256")
rr01_sha = hashlib.sha256(open(f"{REJ}/RR01_duplicate_partners_allowed.rlt", "rb").read()).hexdigest()
rej("RR07_cites_rejected_proof",
    head(must(mn, mc_cite.group(0), f'(MC "{REJ}/RR01_duplicate_partners_allowed.rlt" "{rr01_sha}")'),
         "a citation names a rejected proof"),
    2, "MALFORMED — citation `MC`: tests/rlt005/reject/RR01_duplicate_partners_allowed.rlt is not an accepted theorem")


def conditional(text):
    """The same proof with Γ = {hz : 0 = 0}: the final step G is routed
    through hz (imp-intro of an unused hypothesis, then imp-elim with hz), so
    the kernel reports a conditional theorem with the same goal."""
    lines = text.split("\n")
    i = next(k for k, l in enumerate(lines) if l.startswith("    (G "))
    line = lines[i]

    def close(t, start):  # index of the paren matching t[start] == "("
        depth = 0
        for k in range(start, len(t)):
            depth += {"(": 1, ")": -1}.get(t[k], 0)
            if depth == 0:
                return k
        raise ValueError("unbalanced")

    start = line.index("(G ")
    end = close(line, start)
    claim_start = start + 3
    claim_end = close(line, claim_start) + 1
    rule = line[claim_end:end].strip()
    claim = line[claim_start:claim_end]
    lines[i:i + 1] = [
        f"    (g0 {claim} {rule})",
        "    (z0 (= 0 0) (hyp hz))",
        f"    (z1 (imp (= 0 0) {claim}) (imp-intro hw g0))",
        f"    (G {claim} (imp-elim z1 z0)){line[end + 1:]}",
    ]
    text = "\n".join(lines)
    return must(text, "(mode unconditional)", '(assume hz "0 = 0")\n  (mode conditional)')


rr08a = head(conditional(mc), "MC with Γ = {hz : 0 = 0}: an accepted conditional theorem, cited by RR08")
rej("RR08a_conditional_variant", rr08a, 0, "ACCEPT — conditional theorem")
rr08a_sha = hashlib.sha256(rr08a.encode()).hexdigest()
rej("RR08_cites_conditional_theorem",
    head(must(mn, mc_cite.group(0), f'(MC "{REJ}/RR08a_conditional_variant.rlt" "{rr08a_sha}")'),
         "a citation names a conditional theorem"),
    2, "MALFORMED — citation `MC`: the cited theorem is conditional (Γ ≠ ∅)")
mp = acc("MP4_partners_one_to_four")
g = goal_of(mp)
rej("RR09_partner_range_six",
    head(must(mp, g, must(g, "x < 5", "x < 6")), "wrong partner range: x < 6"),
    1, "REJECT — last step proves")
with open("tests/rlt005/expected.tsv", "w", encoding="utf-8") as fh:
    fh.write("# file\texit\trequired substring of the verdict (RLT-005 rejected cases)\n")
    fh.write("\n".join(rejects) + "\n")
print(f"{len(rejects)} rejected cases written to {REJ}/")

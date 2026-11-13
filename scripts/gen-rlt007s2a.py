#!/usr/bin/env python3
"""RLT-007b stage 2a generator (untrusted): the classification of the primes
below 213 under the bounded definition

    Prime_B(p) :≡ 1 < p ∧ ∀d. ((1 < d ∧ d < p) → ¬(d ∣ p))

written inline with the existing definitions `lt` (x < y :≡ ∃z. S(z) + x = y)
and `dvd` (d ∣ n :≡ ∃k. n = d × k). Deterministic: the same inputs give the
same bytes. Every proof is checked by the kernel; nothing here is trusted.

Theorems (all unconditional under ROBINSON-Q; no new definition):
  tests/rlt007s2a/accept/N_I.rlt   for interval I (1..6), ⋀ ¬Prime_B(m) over the
                                   non-primes m of the interval (0 and 1 by
                                   ¬(1 < m); composites by a divisor witness)
  tests/rlt007s2a/accept/E_I.rlt   exhaustiveness for range I: every p in the
                                   interval with Prime_B(p) is one of the
                                   range's listed primes (cites N_I)
  tests/rlt007s2a/accept/O_I.rlt   order split: O_1 ∀p. (p < 41 ∨ 40 < p); for
                                   I ≥ 2, ∀p. (a−1 < p → (p < b+1 ∨ b < p))
  tests/rlt007s2a/accept/C.rlt     ∀p. (Prime_B(p) → (R1(p) ∨ … ∨ R6(p) ∨ 212 < p))
                                   (cites the six O_I and the six E_I)
  tests/rlt007s2a/reject/*.rlt     rejected cases; tests/rlt007s2a/expected.tsv
  statements/PRIMES-*.yaml         statement records
  lean/RLT007S2a.lean              Lean comparison (the RLT goals, literally)

Source transcription check (not kernel-checked): the 14 primes of the
transcribed proof of Lemma 12.1 (item `omitted-primes`) are the first 14
listed primes. The range lists themselves are design data (approved
2026-11-12); the generator recomputes them by trial division and refuses any
difference, and Lean decides them independently.

Usage: gen-rlt007s2a.py RLT_BINARY [--check | --self-test]
"""
import hashlib
import os
import re
import sys

import yaml

if len(sys.argv) not in (2, 3) or sys.argv[1].startswith("--") or (len(sys.argv) == 3 and sys.argv[2] not in ("--check", "--self-test")):
    sys.exit("usage: gen-rlt007s2a.py RLT_BINARY [--check | --self-test]")
MODE = sys.argv[2] if len(sys.argv) == 3 else ""
root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
os.chdir(root)

# The approved ranges and intervals (docs/RLT-007b-2-DESIGN.md §1).
RANGES = [
    (0, 40, [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37]),
    (41, 78, [41, 43, 47, 53, 59, 61, 67, 71, 73]),
    (79, 112, [79, 83, 89, 97, 101, 103, 107, 109]),
    (113, 156, [113, 127, 131, 137, 139, 149, 151]),
    (157, 190, [157, 163, 167, 173, 179, 181]),
    (191, 212, [191, 193, 197, 199, 211]),
]
BOUNDS = [41, 79, 113, 157, 191, 213]
# The approved interval endpoints, kept apart from RANGES so that a different
# (even a valid) partition is refused.
APPROVED = [(0, 40), (41, 78), (79, 112), (113, 156), (157, 190), (191, 212)]


def is_prime(n):
    return n > 1 and all(n % d for d in range(2, n))


def range_checks(ranges):
    bad = []
    flat = [q for _, _, qs in ranges for q in qs]
    if len(flat) != len(set(flat)):
        bad.append("a prime is listed twice")
    if [(a, b) for a, b, _ in ranges] != APPROVED or BOUNDS != [b + 1 for _, b in APPROVED]:
        bad.append("the intervals are not the approved ones")
    if ranges[0][0] != 0 or ranges[-1][1] != 212:
        bad.append("the intervals do not run from 0 to 212")
    for (a, b, qs), nxt in zip(ranges, ranges[1:] + [None]):
        if nxt and nxt[0] != b + 1:
            bad.append(f"a gap or overlap between [{a}, {b}] and [{nxt[0]}, {nxt[1]}]")
        if qs != [p for p in range(a, b + 1) if is_prime(p)]:
            bad.append(f"the list for [{a}, {b}] is not exactly its primes, in order")
    return bad


T = yaml.safe_load(open("sources/transcriptions/AXIOM-BGP212-2026-09-03.yaml", encoding="utf-8"))
OMITTED = {it["id"]: it for it in T["items"]}["omitted-primes"]["numbers"]


def source_checks(ranges):
    flat = [q for _, _, qs in ranges for q in qs]
    return [] if flat[:14] == OMITTED else ["the 14 transcribed primes are not the first 14 listed primes"]


if MODE == "--self-test":
    cases = [
        ("an omitted prime (37 dropped from R1)", [(0, 40, RANGES[0][2][:-1])] + RANGES[1:]),
        ("a falsely included composite (91 in R3)", RANGES[:2] + [(79, 112, sorted(RANGES[2][2] + [91]))] + RANGES[3:]),
        ("a duplicate entry (41 twice)", RANGES[:1] + [(41, 78, [41] + RANGES[1][2])] + RANGES[2:]),
        ("an interval ending one too early ([0, 39])", [(0, 39, RANGES[0][2])] + RANGES[1:]),
        ("an interval ending one too late ([0, 41])", [(0, 41, RANGES[0][2])] + RANGES[1:]),
        ("an interval starting at 1", [(1, 40, RANGES[0][2])] + RANGES[1:]),
        ("the last interval ending at 211", RANGES[:5] + [(191, 211, RANGES[5][2])]),
        ("a valid but unapproved partition ([0, 41], [42, 78])", [(0, 41, RANGES[0][2] + [41]), (42, 78, RANGES[1][2][1:])] + RANGES[2:]),
    ]
    n = 0
    for name, alt in cases:
        r = range_checks(alt) + source_checks(alt)
        print(f"{'refused ' if r else 'ACCEPTED'} {name}: {'; '.join(r)}")
        n += 1 if r else 0
    print(f"generator self-test: {n} of {len(cases)} altered range lists refused")
    sys.exit(0 if n == len(cases) else 1)

bad = range_checks(RANGES) + source_checks(RANGES)
if bad:
    sys.exit("range or source check failed: " + "; ".join(bad))

ACC, REJ = "tests/rlt007s2a/accept", "tests/rlt007s2a/reject"
OUT = {}


def S(t, k):
    for _ in range(k):
        t = f"(S {t})"
    return t


def PB(t):
    return f"(and (lt 1 {t}) (forall d (imp (and (lt 1 d) (lt d {t})) (not (dvd d {t})))))"


def PB_w(t):
    return f"(1 < {t} ∧ ∀d. ((1 < d ∧ d < {t}) → ¬(d ∣ {t})))"


def R_sx(qs, t):
    f = f"(= {t} {qs[-1]})"
    for q in reversed(qs[:-1]):
        f = f"(or (= {t} {q}) {f})"
    return f


def R_w(qs, t):
    f = f"{t} = {qs[-1]}"
    for q in reversed(qs[:-1]):
        f = f"({t} = {q} ∨ {f})"
    return f if len(qs) == 1 else f


def spf(m):
    return next(d for d in range(2, m + 1) if m % d == 0)


class Proof:
    def __init__(self, pid):
        self.pid, self.st, self.params, self.cites, self.ea_done, self.eamax = pid, [], [], [], set(), -1

    def par(self, name):
        self.params.append(name)
        return name

    def add(self, lab, f, r):
        self.st.append(f"({lab} {f} {r})")
        return lab

    def ea_upto(self, m):
        """EQ_j: e + j = S^j(e), one chain with parameter e."""
        if self.eamax < 0:
            self.par("e")
            self.add("EQ0", "(= (+ e 0) e)", "(inst add_zero (x e))")
            self.eamax = 0
        for j in range(self.eamax + 1, m + 1):
            self.add(f"EAa{j}", f"(= (+ e {j}) (S (+ e {j - 1})))", f"(inst add_succ (x e) (y {j - 1}))")
            self.add(f"EAc{j}", f"(= (S (+ e {j - 1})) {S('e', j)})", f"(cong S EQ{j - 1})")
            self.add(f"EQ{j}", f"(= (+ e {j}) {S('e', j)})", f"(trans EAa{j} EAc{j})")
        self.eamax = max(self.eamax, m)

    def ea(self, j):
        """EA_j : ∀v. v + j = S^j(v)"""
        if j not in self.ea_done:
            self.ea_upto(j)
            self.add(f"EA{j}", f"(forall v (= (+ v {j}) {S('v', j)}))", f"(forall-intro e EQ{j})")
            self.ea_done.add(j)
        return f"EA{j}"

    def text(self, comment, goal_w):
        c = ""
        if self.cites:
            c = "  (cites\n" + "\n".join(f'    ({l} "{pa}" "{hs}")' for l, pa, hs in self.cites) + ")\n"
        pr = f"  (params {' '.join(self.params)})\n" if self.params else ""
        body = "\n".join("    " + s for s in self.st)
        return (f"; {comment}\n(rlt-proof\n  (id RLT007S2a.{self.pid})\n  (convention NAT-DEF)\n  (goal \"{goal_w}\")\n"
                f"  (mode unconditional)\n  (axioms ROBINSON-Q)\n{c}{pr}  (steps\n{body}))\n")


def lt_closed(P, a, b, pre):
    """Steps proving (lt a b) for numerals a < b; returns the label."""
    n = b - a - 1
    if a == 0:
        P.add(f"{pre}l0", f"(= (+ (S {n}) 0) (S {n}))", f"(inst add_zero (x (S {n})))")
        return P.add(f"{pre}l", f"(lt 0 {b})", f"(exists-intro {n} {pre}l0)")
    ea = P.ea(a)
    P.add(f"{pre}l0", f"(= (+ (S {n}) {a}) {b})", f"(forall-elim {ea} (S {n}))")
    return P.add(f"{pre}l", f"(lt {a} {b})", f"(exists-intro {n} {pre}l0)")


def not_prime(P, m, wrong_divisor=None):
    """Steps proving ¬Prime_B(m) for a non-prime m; returns the label."""
    X = f"X{m}_"
    h = P.add(f"{X}H", PB(m), f"(hyp {X}hp)")
    if m in (0, 1):
        z = P.par(f"z{m}")
        P.add(f"{X}L", f"(lt 1 {m})", f"(and-left {X}H)")
        P.add(f"{X}hz", f"(= (+ (S {z}) 1) {m})", f"(hyp {X}hzz)")
        P.add(f"{X}a", f"(= (+ (S {z}) 1) (S (+ (S {z}) 0)))", f"(inst add_succ (x (S {z})) (y 0))")
        if m == 0:
            P.add(f"{X}b", f"(= 0 (+ (S {z}) 1))", f"(symm {X}hz)")
            P.add(f"{X}c", f"(= 0 (S (+ (S {z}) 0)))", f"(trans {X}b {X}a)")
            P.add(f"{X}q", f"(not (= 0 (S (+ (S {z}) 0))))", f"(inst q2_zero_ne_succ (x (+ (S {z}) 0)))")
            P.add(f"{X}f", "false", f"(imp-elim {X}q {X}c)")
        else:
            P.add(f"{X}b", f"(= (S (+ (S {z}) 0)) (+ (S {z}) 1))", f"(symm {X}a)")
            P.add(f"{X}c", f"(= (S (+ (S {z}) 0)) 1)", f"(trans {X}b {X}hz)")
            P.add(f"{X}d", f"(imp (= (S (+ (S {z}) 0)) (S 0)) (= (+ (S {z}) 0) 0))", f"(inst q1_succ_inj (x (+ (S {z}) 0)) (y 0))")
            P.add(f"{X}e", f"(= (+ (S {z}) 0) 0)", f"(imp-elim {X}d {X}c)")
            P.add(f"{X}g", f"(= (+ (S {z}) 0) (S {z}))", f"(inst add_zero (x (S {z})))")
            P.add(f"{X}h", f"(= (S {z}) (+ (S {z}) 0))", f"(symm {X}g)")
            P.add(f"{X}i", f"(= (S {z}) 0)", f"(trans {X}h {X}e)")
            P.add(f"{X}j", f"(= 0 (S {z}))", f"(symm {X}i)")
            P.add(f"{X}q", f"(not (= 0 (S {z})))", f"(inst q2_zero_ne_succ (x {z}))")
            P.add(f"{X}f", "false", f"(imp-elim {X}q {X}j)")
        P.add(f"{X}F", "false", f"(exists-elim {X}L {z} {X}hzz {X}f)")
        return P.add(f"{X}N", f"(not {PB(m)})", f"(imp-intro {X}hp {X}F)")
    s = spf(m) if wrong_divisor is None else wrong_divisor[1]
    d = m // s if wrong_divisor is None else wrong_divisor[0]
    P.add(f"{X}R", f"(forall d (imp (and (lt 1 d) (lt d {m})) (not (dvd d {m}))))", f"(and-right {X}H)")
    P.add(f"{X}I", f"(imp (and (lt 1 {d}) (lt {d} {m})) (not (dvd {d} {m})))", f"(forall-elim {X}R {d})")
    n = d - 2
    P.add(f"{X}a1", f"(= (+ (S {n}) 1) (S (+ (S {n}) 0)))", f"(inst add_succ (x (S {n})) (y 0))")
    P.add(f"{X}a2", f"(= (+ (S {n}) 0) (S {n}))", f"(inst add_zero (x (S {n})))")
    P.add(f"{X}a3", f"(= (S (+ (S {n}) 0)) (S (S {n})))", f"(cong S {X}a2)")
    P.add(f"{X}a4", f"(= (+ (S {n}) 1) {d})", f"(trans {X}a1 {X}a3)")
    P.add(f"{X}A", f"(lt 1 {d})", f"(exists-intro {n} {X}a4)")
    B = lt_closed(P, d, m, f"{X}b")
    P.add(f"{X}AB", f"(and (lt 1 {d}) (lt {d} {m}))", f"(and-intro {X}A {B})")
    P.add(f"{X}ND", f"(not (dvd {d} {m}))", f"(imp-elim {X}I {X}AB)")
    ea = P.ea(d)
    P.add(f"{X}M0", f"(= (* {d} 0) 0)", f"(inst mul_zero (x {d}))")
    prev = f"{X}M0"
    for j in range(1, s + 1):
        P.add(f"{X}m{j}a", f"(= (* {d} {j}) (+ (* {d} {j - 1}) {d}))", f"(inst mul_succ (x {d}) (y {j - 1}))")
        P.add(f"{X}m{j}b", f"(= (* {d} {j}) (+ {d * (j - 1)} {d}))", f"(subst {prev} (at u (= (* {d} {j}) (+ u {d}))) {X}m{j}a)")
        P.add(f"{X}m{j}c", f"(= (+ {d * (j - 1)} {d}) {d * j})", f"(forall-elim {ea} {d * (j - 1)})")
        prev = P.add(f"{X}M{j}", f"(= (* {d} {j}) {d * j})", f"(trans {X}m{j}b {X}m{j}c)")
    P.add(f"{X}W", f"(= {m} (* {d} {s}))", f"(symm {prev})")
    P.add(f"{X}D", f"(dvd {d} {m})", f"(exists-intro {s} {X}W)")
    P.add(f"{X}F", "false", f"(imp-elim {X}ND {X}D)")
    return P.add(f"{X}N", f"(not {PB(m)})", f"(imp-intro {X}hp {X}F)")


def conj(P, parts, pre):
    """parts: [(sexp, written, label)] → left-nested and-intro chain."""
    f, w, l = parts[0]
    for k, (g, wg, lg) in enumerate(parts[1:], 1):
        f, w = f"(and {f} {g})", f"({w} ∧ {wg})"
        l = P.add(f"{pre}{k}", f, f"(and-intro {l} {lg})")
    return f, w, l


def bconj(P, parts, pre):
    """Balanced conjunction: (and LEFT RIGHT) with LEFT the first half, so a
    reader extracts every conjunct with about log2(k) restatements each."""
    if len(parts) == 1:
        return parts[0]
    h = (len(parts) + 1) // 2
    lf, lw, ll = bconj(P, parts[:h], pre + "l")
    rf, rw, rl = bconj(P, parts[h:], pre + "r")
    f, w = f"(and {lf} {rf})", f"({lw} ∧ {rw})"
    return f, w, P.add(pre, f, f"(and-intro {ll} {rl})")


def bsplit(f):
    """(and L R) → (L, R) for a sexp string."""
    assert f.startswith("(and ")
    depth, k = 0, len("(and ")
    for k in range(len("(and "), len(f)):
        depth += f[k] == "("
        depth -= f[k] == ")"
        if depth == 0:
            break
    return f[len("(and "):k + 1], f[k + 2:-1]


def finish(P, lab):
    P.st[-1] = P.st[-1].replace(f"({lab} ", "(G ", 1)


def sha(text):
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def nonprimes(i):
    a, b, qs = RANGES[i]
    return [m for m in range(a, b + 1) if m not in qs]


# 1. Non-prime certificates N_1 … N_6.
def n_file(i, extra=None, wrong=None):
    P = Proof(f"N_{i + 1}")
    parts = []
    ms = nonprimes(i) + ([extra] if extra is not None else [])
    for m in ms:
        lab = not_prime(P, m, wrong if (wrong and m == wrong[2]) else None)
        parts.append((f"(not {PB(m)})", f"¬{PB_w(m)}", lab))
    f, w, l = bconj(P, parts, "C")
    finish(P, l)
    a, b, _ = RANGES[i]
    return P.text(f"RLT-007b stage 2a: every non-prime m in [{a}, {b}] satisfies ¬Prime_B(m) "
                  f"(0 and 1 by ¬(1 < m); composites by a divisor witness).", w), f, ms


N_OUT = {}
for i in range(6):
    text, f, ms = n_file(i)
    OUT[f"{ACC}/N_{i + 1}.rlt"] = text
    N_OUT[i] = (f, ms)


# 2. Exhaustiveness E_1 … E_6.
def e_hyp(i, t):
    a, b, _ = RANGES[i]
    if a == 0:
        return f"(and (lt {t} {b + 1}) {PB(t)})", f"({t} < {b + 1} ∧ {PB_w(t)})"
    return (f"(and (and (lt {a - 1} {t}) (lt {t} {b + 1})) {PB(t)})",
            f"(({a - 1} < {t} ∧ {t} < {b + 1}) ∧ {PB_w(t)})")


def e_goal(i):
    h, hw = e_hyp(i, "p")
    qs = RANGES[i][2]
    return (f"(forall p (imp {h} {R_sx(qs, 'p')}))", f"∀p. ({hw} → {R_w(qs, 'p')})")


def e_file(i, n_path, n_text, qs_override=None, b_override=None):
    a, b, qs = RANGES[i]
    if qs_override is not None:
        qs = qs_override
    if b_override is not None:
        b = b_override
    P = Proof(f"E_{i + 1}")
    nf, ms = N_OUT[i]
    P.cites.append(("N", n_path, sha(n_text)))
    P.add("Nc", nf, "(cite N)")
    # Extract every conjunct once from the balanced tree (and-left, and-right).
    notp = {}
    order = iter(ms)

    def walk(lab, f):
        if not f.startswith("(and "):
            notp[next(order)] = lab
            return
        lf, rf = bsplit(f)
        walk(P.add(f"{lab}l", lf, f"(and-left {lab})"), lf)
        walk(P.add(f"{lab}r", rf, f"(and-right {lab})"), rf)

    walk("Nc", nf)
    pp = P.par("pp")
    goal = R_sx(qs, "pp")
    if a == 0:
        hf = f"(and (lt pp {b + 1}) {PB('pp')})"
    else:
        hf = f"(and (and (lt {a - 1} pp) (lt pp {b + 1})) {PB('pp')})"
    P.add("H", hf, "(hyp hE)")
    P.add("Pp", PB("pp"), "(and-right H)")
    w = P.par("w")
    if a == 0:
        P.add("U", f"(lt pp {b + 1})", "(and-left H)")
        P.add("Uh", f"(= (+ (S {w}) pp) {b + 1})", "(hyp hu)")
        fact, zv, eqlab = "Uh", "pp", None
        n = b + 1
    else:
        z = P.par("z")
        P.add("B", f"(and (lt {a - 1} pp) (lt pp {b + 1}))", "(and-left H)")
        P.add("Lo", f"(lt {a - 1} pp)", "(and-left B)")
        P.add("Up", f"(lt pp {b + 1})", "(and-right B)")
        P.add("Lh", f"(= (+ (S {z}) {a - 1}) pp)", "(hyp hl)")
        ea = P.ea(a - 1)
        P.add("Le", f"(= (+ (S {z}) {a - 1}) {S(z, a)})", f"(forall-elim {ea} (S {z}))")
        P.add("Ls", f"(= pp (+ (S {z}) {a - 1}))", "(symm Lh)")
        P.add("Lp", f"(= pp {S(z, a)})", "(trans Ls Le)")
        P.add("Uh", f"(= (+ (S {w}) pp) {b + 1})", "(hyp hu)")
        P.add("Us", f"(= (+ (S {w}) {S(z, a)}) {b + 1})", f"(subst Lp (at u (= (+ (S {w}) u) {b + 1})) Uh)")
        Xt = f"(+ (S {w}) {z})"
        P.add("A0", f"(= {Xt} {Xt})", f"(refl {Xt})")
        for t in range(1, a + 1):
            P.add(f"Aa{t}", f"(= (+ (S {w}) {S(z, t)}) (S (+ (S {w}) {S(z, t - 1)})))", f"(inst add_succ (x (S {w})) (y {S(z, t - 1)}))")
            P.add(f"Ac{t}", f"(= (S (+ (S {w}) {S(z, t - 1)})) {S(Xt, t)})", f"(cong S A{t - 1})")
            P.add(f"A{t}", f"(= (+ (S {w}) {S(z, t)}) {S(Xt, t)})", f"(trans Aa{t} Ac{t})")
        P.add("As", f"(= {S(Xt, a)} (+ (S {w}) {S(z, a)}))", f"(symm A{a})")
        P.add("Ap", f"(= {S(Xt, a)} {b + 1})", "(trans As Us)")
        cur = "Ap"
        for t in range(a, 0, -1):
            num = b + 1 - a + t - 1
            P.add(f"Pq{t}", f"(imp (= {S(Xt, t)} (S {num})) (= {S(Xt, t - 1)} {num}))", f"(inst q1_succ_inj (x {S(Xt, t - 1)}) (y {num}))")
            cur = P.add(f"Pe{t}", f"(= {S(Xt, t - 1)} {num})", f"(imp-elim Pq{t} {cur})")
        fact, zv, eqlab = cur, z, "Lp"
        n = b + 1 - a

    def value(m, eq):
        """eq : (= pp m)."""
        V = f"V{m}_"
        if m in qs:
            idx = qs.index(m)
            rest = qs[idx:]
            lab = P.add(f"{V}r", f"(= {m} {m})", f"(refl {m})")
            if len(rest) > 1:
                lab = P.add(f"{V}o{idx}", R_sx(rest, m), f"(or-left {lab})")
            for t in range(idx - 1, -1, -1):
                lab = P.add(f"{V}o{t}", R_sx(qs[t:], m), f"(or-right {lab})")
            P.add(f"{V}s", f"(= {m} pp)", f"(symm {eq})")
            return P.add(f"{V}g", goal, f"(subst {V}s (at u {R_sx(qs, 'u')}) {lab})")
        P.add(f"{V}p", PB(m), f"(subst {eq} (at u {PB('u')}) Pp)")
        P.add(f"{V}f", "false", f"(imp-elim {notp.get(m, 'NO_CERTIFICATE_FOR_' + str(m))} {V}p)")
        return P.add(f"{V}g", goal, f"(false-elim {V}f)")

    def enum(zv, n, fact, eqlab, j):
        """fact: (= (+ (S w) zv) n); eqlab: (= pp S^{a+j}(zv)) or None (a = 0, j = 0)."""
        L = f"E{j}_"
        y = P.par(f"y{j}")
        P.add(f"{L}q", f"(or (= {zv} 0) (exists y (= {zv} (S y))))", f"(inst q3_zero_or_succ (x {zv}))")
        # zv = 0
        P.add(f"{L}h0", f"(= {zv} 0)", f"(hyp {L}hz)")
        if n == 0:
            P.add(f"{L}z1", f"(= (+ (S {w}) 0) 0)", f"(subst {L}h0 (at u (= (+ (S {w}) u) 0)) {fact})")
            P.add(f"{L}z2", f"(= (+ (S {w}) 0) (S {w}))", f"(inst add_zero (x (S {w})))")
            P.add(f"{L}z3", f"(= (S {w}) (+ (S {w}) 0))", f"(symm {L}z2)")
            P.add(f"{L}z4", f"(= (S {w}) 0)", f"(trans {L}z3 {L}z1)")
            P.add(f"{L}z5", f"(= 0 (S {w}))", f"(symm {L}z4)")
            P.add(f"{L}z6", f"(not (= 0 (S {w})))", f"(inst q2_zero_ne_succ (x {w}))")
            P.add(f"{L}z7", "false", f"(imp-elim {L}z6 {L}z5)")
            zg = P.add(f"{L}zg", goal, f"(false-elim {L}z7)")
        else:
            m = a + j
            if eqlab is None:
                eq = f"{L}h0"
            else:
                eq = P.add(f"{L}ze", f"(= pp {m})", f"(subst {L}h0 (at u (= pp {S('u', a + j)})) {eqlab})")
            zg = value(m, eq)
        # zv = S(y)
        P.add(f"{L}hs", f"(exists y (= {zv} (S y)))", f"(hyp {L}hss)")
        P.add(f"{L}s0", f"(= {zv} (S {y}))", f"(hyp {L}hs0)")
        P.add(f"{L}s1", f"(= (+ (S {w}) (S {y})) {n})", f"(subst {L}s0 (at u (= (+ (S {w}) u) {n})) {fact})")
        P.add(f"{L}s2", f"(= (+ (S {w}) (S {y})) (S (+ (S {w}) {y})))", f"(inst add_succ (x (S {w})) (y {y}))")
        P.add(f"{L}s3", f"(= (S (+ (S {w}) {y})) (+ (S {w}) (S {y})))", f"(symm {L}s2)")
        P.add(f"{L}s4", f"(= (S (+ (S {w}) {y})) {n})", f"(trans {L}s3 {L}s1)")
        if n == 0:
            P.add(f"{L}s5", f"(= 0 (S (+ (S {w}) {y})))", f"(symm {L}s4)")
            P.add(f"{L}s6", f"(not (= 0 (S (+ (S {w}) {y}))))", f"(inst q2_zero_ne_succ (x (+ (S {w}) {y})))")
            P.add(f"{L}s7", "false", f"(imp-elim {L}s6 {L}s5)")
            P.add(f"{L}s8", goal, f"(false-elim {L}s7)")
            sg = P.add(f"{L}sg", goal, f"(exists-elim {L}hs {y} {L}hs0 {L}s8)")
        else:
            P.add(f"{L}s5", f"(imp (= (S (+ (S {w}) {y})) (S {n - 1})) (= (+ (S {w}) {y}) {n - 1}))", f"(inst q1_succ_inj (x (+ (S {w}) {y})) (y {n - 1}))")
            nf = P.add(f"{L}s6", f"(= (+ (S {w}) {y}) {n - 1})", f"(imp-elim {L}s5 {L}s4)")
            if eqlab is None:
                # a = 0 and j = 0: pp itself was split, so pp = S(y) is the equation.
                ne = f"{L}s0"
            else:
                ne = P.add(f"{L}s7", f"(= pp {S(y, a + j + 1)})", f"(subst {L}s0 (at u (= pp {S('u', a + j)})) {eqlab})")
            sub = enum(y, n - 1, nf, ne, j + 1)
            sg = P.add(f"{L}sg", goal, f"(exists-elim {L}hs {y} {L}hs0 {sub})")
        return P.add(f"{L}g", goal, f"(or-cases {L}q {L}hz {zg} {L}hss {sg})")

    if a == 0:
        # pp is split directly; after the first split the variable is y0 with pp = S(y0).
        res = enum("pp", b + 1, "Uh", None, 0)
        P.add("U2", goal, f"(exists-elim U {w} hu {res})")
    else:
        res = enum(zv, n, fact, eqlab, 0)
        P.add("U1", goal, f"(exists-elim Up {w} hu {res})")
        P.add("U2", goal, f"(exists-elim Lo {z} hl U1)")
    P.add("I", f"(imp {hf} {goal})", "(imp-intro hE U2)")
    gf, gw = e_goal(i)
    if qs_override is not None or b_override is not None:
        h2 = (f"(and (lt pp {b + 1}) {PB('pp')})" if a == 0 else f"(and (and (lt {a - 1} pp) (lt pp {b + 1})) {PB('pp')})")
        gf = f"(forall p (imp {h2.replace('pp', 'p')} {R_sx(qs, 'p')}))"
        hw = (f"(p < {b + 1} ∧ {PB_w('p')})" if a == 0 else f"(({a - 1} < p ∧ p < {b + 1}) ∧ {PB_w('p')})")
        gw = f"∀p. ({hw} → {R_w(qs, 'p')})"
    P.add("G", gf, "(forall-intro pp I)")
    return P.text(f"RLT-007b stage 2a: exhaustiveness for range {i + 1}: every p in [{a}, {b}] with Prime_B(p) "
                  f"is one of {qs}. Cites the non-prime certificates {n_path}.", gw)


for i in range(6):
    npath = f"{ACC}/N_{i + 1}.rlt"
    OUT[f"{ACC}/E_{i + 1}.rlt"] = e_file(i, npath, OUT[npath])


# 3. Order splits. O_1: ∀p. (p < 41 ∨ 40 < p). O_I (I = 2..6), relative to the
# previous bound: ∀p. (a − 1 < p → (p < b + 1 ∨ b < p)) for the interval [a, b].
def o_file(i):
    a, b, _ = RANGES[i]
    B = b + 1
    P = Proof(f"O_{i + 1}")
    pp = P.par("pp")
    goal_of = lambda t: f"(or (lt {t} {B}) (lt {b} {t}))"
    goal = goal_of("pp")

    def level(j, zv, eqlab):
        """eqlab: (= pp S^{a+j}(zv)), or None when a = 0 and j = 0 (zv is pp)."""
        L = f"O{j}_"
        if a + j == B:
            ea = P.ea(b)
            P.add(f"{L}a", f"(= (+ (S {zv}) {b}) {S(zv, B)})", f"(forall-elim {ea} (S {zv}))")
            P.add(f"{L}b", f"(= {S(zv, B)} pp)", f"(symm {eqlab})")
            P.add(f"{L}c", f"(= (+ (S {zv}) {b}) pp)", f"(trans {L}a {L}b)")
            P.add(f"{L}d", f"(lt {b} pp)", f"(exists-intro {zv} {L}c)")
            return P.add(f"{L}g", goal, f"(or-right {L}d)")
        y = P.par(f"y{j}")
        m = a + j
        P.add(f"{L}q", f"(or (= {zv} 0) (exists y (= {zv} (S y))))", f"(inst q3_zero_or_succ (x {zv}))")
        P.add(f"{L}h0", f"(= {zv} 0)", f"(hyp {L}hz)")
        if eqlab is None:
            eq = f"{L}h0"
        else:
            eq = P.add(f"{L}e", f"(= pp {m})", f"(subst {L}h0 (at u (= pp {S('u', m)})) {eqlab})")
        lt = lt_closed(P, m, B, f"{L}w")
        P.add(f"{L}o", goal_of(m), f"(or-left {lt})")
        P.add(f"{L}s", f"(= {m} pp)", f"(symm {eq})")
        zg = P.add(f"{L}zg", goal, f"(subst {L}s (at u {goal_of('u')}) {L}o)")
        P.add(f"{L}hs", f"(exists y (= {zv} (S y)))", f"(hyp {L}hss)")
        P.add(f"{L}s0", f"(= {zv} (S {y}))", f"(hyp {L}hs0)")
        if eqlab is None:
            ne = f"{L}s0"
        else:
            ne = P.add(f"{L}s1", f"(= pp {S(y, m + 1)})", f"(subst {L}s0 (at u (= pp {S('u', m)})) {eqlab})")
        sub = level(j + 1, y, ne)
        P.add(f"{L}sg", goal, f"(exists-elim {L}hs {y} {L}hs0 {sub})")
        return P.add(f"{L}g", goal, f"(or-cases {L}q {L}hz {zg} {L}hss {L}sg)")

    if a == 0:
        res = level(0, "pp", None)
        P.add("G", f"(forall p {goal_of('p')})", f"(forall-intro pp {res})")
        return P.text(f"RLT-007b stage 2a: order split at {B}: every p satisfies p < {B} or {b} < p (finitely many Q3 case splits).",
                      f"∀p. (p < {B} ∨ {b} < p)")
    z = P.par("z")
    P.add("Lo", f"(lt {a - 1} pp)", "(hyp hl)")
    P.add("Lh", f"(= (+ (S {z}) {a - 1}) pp)", "(hyp hlz)")
    ea = P.ea(a - 1)
    P.add("Le", f"(= (+ (S {z}) {a - 1}) {S(z, a)})", f"(forall-elim {ea} (S {z}))")
    P.add("Ls", f"(= pp (+ (S {z}) {a - 1}))", "(symm Lh)")
    P.add("Lp", f"(= pp {S(z, a)})", "(trans Ls Le)")
    res = level(0, z, "Lp")
    P.add("Lx", goal, f"(exists-elim Lo {z} hlz {res})")
    P.add("I", f"(imp (lt {a - 1} pp) {goal})", "(imp-intro hl Lx)")
    P.add("G", f"(forall p (imp (lt {a - 1} p) {goal_of('p')}))", "(forall-intro pp I)")
    return P.text(f"RLT-007b stage 2a: order split at {B} above {a - 1}: every p with {a - 1} < p satisfies p < {B} or {b} < p "
                  f"(finitely many Q3 case splits).", f"∀p. ({a - 1} < p → (p < {B} ∨ {b} < p))")


def o_goal(i):
    a, b, _ = RANGES[i]
    if a == 0:
        return f"(forall p (or (lt p {b + 1}) (lt {b} p)))"
    return f"(forall p (imp (lt {a - 1} p) (or (lt p {b + 1}) (lt {b} p))))"


for i in range(6):
    OUT[f"{ACC}/O_{i + 1}.rlt"] = o_file(i)


# 4. The top classification C.
def c_goal(t):
    f = f"(lt 212 {t})"
    for i in range(5, -1, -1):
        f = f"(or {R_sx(RANGES[i][2], t)} {f})"
    return f


def c_goal_w(t):
    w = f"212 < {t}"
    for i in range(5, -1, -1):
        w = f"({R_w(RANGES[i][2], t)} ∨ {w})"
    return w


def c_file():
    P = Proof("C")
    for i in range(6):
        P.cites.append((f"O{i + 1}", f"{ACC}/O_{i + 1}.rlt", sha(OUT[f"{ACC}/O_{i + 1}.rlt"])))
    for i in range(6):
        P.cites.append((f"E{i + 1}", f"{ACC}/E_{i + 1}.rlt", sha(OUT[f"{ACC}/E_{i + 1}.rlt"])))
    pp = P.par("pp")
    goal = c_goal("pp")
    P.add("H", PB("pp"), "(hyp hp)")
    for i in range(6):
        P.add(f"O{i + 1}c", o_goal(i), f"(cite O{i + 1})")
    for i in range(6):
        gf, _ = e_goal(i)
        P.add(f"E{i + 1}c", gf, f"(cite E{i + 1})")
        h, _ = e_hyp(i, "pp")
        P.add(f"E{i + 1}i", f"(imp {h} {R_sx(RANGES[i][2], 'pp')})", f"(forall-elim E{i + 1}c pp)")

    def lift(i, lab, pre):
        """From R_i(pp) (label lab) to the goal: i or-rights after one or-left."""
        tail = f"(lt 212 pp)"
        tails = [None] * 7
        tails[6] = tail
        for k in range(5, -1, -1):
            tails[k] = f"(or {R_sx(RANGES[k][2], 'pp')} {tails[k + 1]})"
        cur = P.add(f"{pre}L", tails[i], f"(or-left {lab})")
        for k in range(i - 1, -1, -1):
            cur = P.add(f"{pre}R{k}", tails[k], f"(or-right {cur})")
        return cur

    def case(i, lower):
        """lower: label of (lt a−1 pp) or None for i = 0. Splits at BOUNDS[i]."""
        B = BOUNDS[i]
        K = f"K{i}_"
        if i == 0:
            split = P.add(f"{K}sp", f"(or (lt pp {B}) (lt {B - 1} pp))", "(forall-elim O1c pp)")
        else:
            P.add(f"{K}oi", f"(imp (lt {RANGES[i][0] - 1} pp) (or (lt pp {B}) (lt {B - 1} pp)))", f"(forall-elim O{i + 1}c pp)")
            split = P.add(f"{K}sp", f"(or (lt pp {B}) (lt {B - 1} pp))", f"(imp-elim {K}oi {lower})")
        # p < B
        P.add(f"{K}lt", f"(lt pp {B})", f"(hyp {K}h1)")
        if i == 0:
            P.add(f"{K}hy", e_hyp(0, "pp")[0], f"(and-intro {K}lt H)")
        else:
            a = RANGES[i][0]
            P.add(f"{K}ab", f"(and (lt {a - 1} pp) (lt pp {B}))", f"(and-intro {lower} {K}lt)")
            P.add(f"{K}hy", e_hyp(i, "pp")[0], f"(and-intro {K}ab H)")
        P.add(f"{K}r", R_sx(RANGES[i][2], "pp"), f"(imp-elim E{i + 1}i {K}hy)")
        left = lift(i, f"{K}r", f"{K}u")
        # B − 1 < p
        P.add(f"{K}gt", f"(lt {B - 1} pp)", f"(hyp {K}h2)")
        if i == 5:
            cur = f"{K}gt"
            for k in range(5, -1, -1):
                f = "(lt 212 pp)"
                for kk in range(5, k - 1, -1):
                    f = f"(or {R_sx(RANGES[kk][2], 'pp')} {f})"
                cur = P.add(f"{K}t{k}", f, f"(or-right {cur})")
            right = cur
        else:
            right = case(i + 1, f"{K}gt")
        return P.add(f"{K}g", goal, f"(or-cases {split} {K}h1 {left} {K}h2 {right})")

    res = case(0, None)
    P.add("I", f"(imp {PB('pp')} {goal})", f"(imp-intro hp {res})")
    P.add("G", f"(forall p (imp {PB('p')} {c_goal('p')}))", "(forall-intro pp I)")
    return P.text("RLT-007b stage 2a: every p with Prime_B(p) is one of the 47 listed primes, in its range, "
                  "or 212 < p. Cites the six order splits O_1 … O_6 and the six exhaustiveness theorems E_1 … E_6.",
                  f"∀p. ({PB_w('p')} → {c_goal_w('p')})")


OUT[f"{ACC}/C.rlt"] = c_file()


# 5. Rejected cases (expected verdict substrings are fixed; tests compare them).
rows = []


def goal_of(text):
    return re.search(r'\(goal "([^"]*)"\)', text).group(1)



def rej(name, text, code, reason):
    OUT[f"{REJ}/{name}.rlt"] = text
    rows.append(f"{REJ}/{name}.rlt\t{code}\t{reason}")


t, _, _ = n_file(0, extra=37, wrong=(5, 7, 37))
rej("SY01_nonprime_certificate_for_prime_37",
    t.replace("; RLT-007b stage 2a:", "; REJECT CASE (a certificate claiming ¬Prime_B(37) with the divisor 5 × 7):", 1),
    1, "REJECT at step X37_W — step claims `37 = 5 × 7`, rule `symm` yields `35 = 5 × 7`")
rej("SY02_range_1_omitting_37",
    e_file(0, f"{REJ}/SY01_nonprime_certificate_for_prime_37.rlt", OUT[f"{REJ}/SY01_nonprime_certificate_for_prime_37.rlt"],
           qs_override=RANGES[0][2][:-1]).replace("; RLT-007b stage 2a:", "; REJECT CASE (range 1 without 37, citing a false certificate):", 1),
    2, f"MALFORMED — citation `N`: {REJ}/SY01_nonprime_certificate_for_prime_37.rlt is not an accepted theorem (REJECT at step X37_W")
rej("SY03_range_2_interval_widened_to_79",
    e_file(1, f"{ACC}/N_2.rlt", OUT[f"{ACC}/N_2.rlt"], b_override=79).replace("; RLT-007b stage 2a:", "; REJECT CASE (range 2 over [41, 79]: 79 is prime but not listed):", 1),
    1, "REJECT at step V79_f — premise `NO_CERTIFICATE_FOR_79` is not an earlier step")
o1 = OUT[f"{ACC}/O_1.rlt"]
o1_bad = o1.replace('(goal "∀p. (p < 41 ∨ 40 < p)")', '(goal "∀p. (p < 41 ∨ 41 < p)")')
o1_bad = o1_bad.replace("(G (forall p (or (lt p 41) (lt 40 p)))", "(G (forall p (or (lt p 41) (lt 41 p)))")
assert o1_bad.count("41 < p") == 1 and o1_bad.count("(lt 41 p)") == 1
rej("SY04_order_split_gap_at_41",
    o1_bad.replace("; RLT-007b stage 2a:", "; REJECT CASE (only the statement and its last step changed: the split p < 41 ∨ 41 < p leaves 41 uncovered):", 1),
    1, "REJECT at step G — step claims `∀p. ((∃z. S(z) + p = 41) ∨ (∃z. S(z) + 41 = p))`, rule `forall-intro` yields `∀p. ((∃z. S(z) + p = 41) ∨ (∃z. S(z) + 40 = p))`")
c = OUT[f"{ACC}/C.rlt"]
h6 = sha(OUT[f"{ACC}/E_6.rlt"])
rej("SY05_classification_stale_citation",
    c.replace(h6, "f" * 64).replace("; RLT-007b stage 2a:", "; REJECT CASE (a stale SHA-256 for E_6):", 1),
    2, f"MALFORMED — citation `E6`: {ACC}/E_6.rlt has SHA-256")
rej("SY06_classification_missing_citation",
    c.replace(f'"{ACC}/E_6.rlt"', f'"{ACC}/E_7.rlt"').replace("; RLT-007b stage 2a:", "; REJECT CASE (citing a file that does not exist):", 1),
    2, f"MALFORMED — citation `E6`: cited path `{ACC}/E_7.rlt`: entity not found")
P2 = Proof("SY07")
lab = not_prime(P2, 0)
txt = P2.text("REJECT CASE: the ¬(1 < m) certificate applied to 2 (2 is prime)", f"¬{PB_w(2)}")
txt = txt.replace(PB(0), PB(2)).replace("(lt 1 0)", "(lt 1 2)").replace("(= (+ (S z0) 1) 0)", "(= (+ (S z0) 1) 2)").replace("X0_", "X2_")
assert "(lt 1 0)" not in txt and txt.count(PB(2)) == 2
rej("SY07_zero_one_certificate_applied_to_2", txt.replace("(X2_N ", "(G ", 1), 1, "REJECT at step X2_b — step claims `0 = S(z0) + 1`, rule `symm` yields `2 = S(z0) + 1`")
rej("SY08_range_6_step_hypothesis_altered",
    OUT[f"{ACC}/E_6.rlt"].replace("(and (lt 1 d) (lt d pp))", "(lt 0 d)").replace("; RLT-007b stage 2a:", "; REJECT CASE (inside the proof steps, Prime_B's hypothesis (1 < d ∧ d < p) replaced by 0 < d; the statement is unchanged):", 1),
    1, "REJECT at step V192_p — subst: motive does not match premise")
e6 = OUT[f"{ACC}/E_6.rlt"]
e6_goal = goal_of(e6)
assert e6_goal.count("((1 < d ∧ d < p)") == 1
rej("SY09_range_6_statement_with_weakened_prime",
    e6.replace(e6_goal, e6_goal.replace("((1 < d ∧ d < p)", "((0 < d ∧ d < p)"), 1).replace("; RLT-007b stage 2a:", "; REJECT CASE (only the written statement changed: Prime_B's hypothesis 1 < d weakened to 0 < d):", 1),
    1, "the goal is `∀p. ((((∃z. S(z) + 190 = p) ∧ (∃z. S(z) + p = 213)) ∧ ((∃z. S(z) + 1 = p) ∧ (∀d. (((∃z. S(z) + 0 = d) ∧")
OUT["tests/rlt007s2a/expected.tsv"] = ("# file\texit\trequired substring of the verdict (RLT-007b stage 2a rejected cases)\n"
                                      + "\n".join(rows) + "\n")



# 6. Lean comparison: the RLT goals over ℕ, with RLT's `<`, `∣` and Prime_B as
# definitions that unfold to spec/definitions.rlt symbol for symbol.
def lean_R(qs, t="p"):
    return " ∨ ".join(f"{t} = {q}" for q in qs)


def lean_pb_conj_tree(ms):
    if len(ms) == 1:
        return f"¬ PB {ms[0]}"
    h = (len(ms) + 1) // 2
    return f"({lean_pb_conj_tree(ms[:h])} ∧ {lean_pb_conj_tree(ms[h:])})"


def lean_pb_proof_tree(ms):
    if len(ms) == 1:
        m = ms[0]
        if m in (0, 1):
            return f"(not_pb_small {m} (by decide))"
        sp = spf(m)
        return f"(not_pb_of {m} {m // sp} {sp} (by decide) (by decide) (by decide))"
    h = (len(ms) + 1) // 2
    return f"⟨{lean_pb_proof_tree(ms[:h])}, {lean_pb_proof_tree(ms[h:])}⟩"


L = [f"""/-
RLT-007b stage 2a, Lean comparison: GENERATED by scripts/gen-rlt007s2a.py.
Core Lean only. Run: cd lean && lean RLT007S2a.lean

LT, DV and PB below unfold to RLT's `lt` (∃z. S(z) + x = y), `dvd`
(∃k. n = d × k) and Prime_B, symbol for symbol (S(z) read as z + 1). What
this file proves, in Lean (none of it is an RLT derivation):
  * n_I, e_I, o_I, c: RLT's stage-2a statements over ℕ;
  * listed_I: the converse of e_I, so each range list is exactly the primes
    of its interval (a Lean result; RLT's positive primality is stage 2b);
  * pb_iff_std, pg_iff_std: Prime_B and the guarded reference form G are each
    equivalent over ℕ to the standard definition 2 ≤ p ∧ ∀d, d ∣ p → d = 1 ∨ d = p.
These equivalences are Lean theorems about ℕ; RLT proves no equivalence of
the definitions under Robinson's Q.
-/

namespace RLT007S2a

def LT (x y : Nat) : Prop := ∃ z, (z + 1) + x = y
def DV (d n : Nat) : Prop := ∃ k, n = d * k
def PB (p : Nat) : Prop := LT 1 p ∧ ∀ d, (LT 1 d ∧ LT d p) → ¬ DV d p
def PG (p : Nat) : Prop := LT 1 p ∧ ∀ d, (LT 0 d ∧ DV d p) → d = 1 ∨ d = p
def PStd (p : Nat) : Prop := 2 ≤ p ∧ ∀ d, d ∣ p → d = 1 ∨ d = p

theorem lt_iff (x y : Nat) : LT x y ↔ x < y :=
  ⟨fun ⟨z, h⟩ => by omega, fun h => ⟨y - x - 1, by omega⟩⟩
theorem dv_iff (d n : Nat) : DV d n ↔ d ∣ n := Iff.rfl

theorem pb_iff_std (p : Nat) : PB p ↔ PStd p := by
  simp only [PB, PStd, lt_iff, dv_iff]
  constructor
  · intro ⟨h1, h2⟩
    refine ⟨h1, fun d hd => ?_⟩
    have hle : d ≤ p := Nat.le_of_dvd (by omega) hd
    have hd0 : d ≠ 0 := by rintro rfl; simp at hd; omega
    by_cases e1 : d = 1
    · exact .inl e1
    · by_cases e2 : d = p
      · exact .inr e2
      · exact absurd hd (h2 d ⟨by omega, by omega⟩)
  · intro ⟨h1, h2⟩
    refine ⟨h1, fun d ⟨hd1, hd2⟩ hd => ?_⟩
    rcases h2 d hd with h | h <;> omega

theorem pg_iff_std (p : Nat) : PG p ↔ PStd p := by
  simp only [PG, PStd, lt_iff, dv_iff]
  constructor
  · intro ⟨h1, h2⟩
    refine ⟨h1, fun d hd => h2 d ⟨?_, hd⟩⟩
    rcases Nat.eq_zero_or_pos d with rfl | h
    · simp at hd; omega
    · exact h
  · intro ⟨h1, h2⟩
    exact ⟨h1, fun d ⟨_, hd⟩ => h2 d hd⟩

/-- Prime_B in decidable, bounded form. -/
theorem pb_dec (p : Nat) (h : PB p) : 1 < p ∧ ∀ d, d < p → 1 < d → ¬ d ∣ p := by
  rw [PB, lt_iff] at h
  exact ⟨h.1, fun d h1 h2 hd => h.2 d ⟨(lt_iff 1 d).mpr h2, (lt_iff d p).mpr h1⟩ hd⟩

/-- The converse: the bounded form gives Prime_B. -/
theorem pb_of_dec (p : Nat) (h : 1 < p ∧ ∀ d, d < p → 1 < d → ¬ d ∣ p) : PB p := by
  rw [PB, lt_iff]
  exact ⟨h.1, fun d ⟨h1, h2⟩ => h.2 d ((lt_iff d p).mp h2) ((lt_iff 1 d).mp h1)⟩

theorem not_pb_small (m : Nat) (h : m < 2) : ¬ PB m := fun ⟨h1, _⟩ => by
  rw [lt_iff] at h1; omega

theorem not_pb_of (m d s : Nat) (h1 : 1 < d) (h2 : d < m) (h3 : m = d * s) : ¬ PB m :=
  fun ⟨_, h⟩ => h d ⟨(lt_iff 1 d).mpr h1, (lt_iff d m).mpr h2⟩ ⟨s, h3⟩
"""]
L.append("set_option maxRecDepth 100000\nset_option maxHeartbeats 4000000\n")
TH = ["lt_iff", "dv_iff", "pb_iff_std", "pg_iff_std", "pb_dec", "pb_of_dec", "not_pb_small", "not_pb_of"]
for i in range(6):
    a, b, qs = RANGES[i]
    ms = nonprimes(i)
    L.append(f"/-- RLT's N_{i + 1} (tests/rlt007s2a/accept/N_{i + 1}.rlt). -/\n"
             f"theorem n_{i + 1} :\n    {lean_pb_conj_tree(ms)} :=\n  {lean_pb_proof_tree(ms)}\n")
    hyp = f"LT p {b + 1}" if a == 0 else f"(LT {a - 1} p ∧ LT p {b + 1})"
    low = "" if a == 0 else f"{a} ≤ p ∧ "
    use = ""
    up = "h.1" if a == 0 else "h.1.2"
    L.append(f"/-- RLT's E_{i + 1} (tests/rlt007s2a/accept/E_{i + 1}.rlt). -/\n"
             f"theorem e_{i + 1} : ∀ p, ({hyp} ∧ PB p) → {lean_R(qs)} := by\n"
             f"  have key : ∀ p, p < {b + 1} → ({low}(1 < p ∧ ∀ d, d < p → 1 < d → ¬ d ∣ p)) → {lean_R(qs)} := by decide\n"
             f"  intro p h\n"
             + (f"  exact key p ((lt_iff _ _).mp {up}) (pb_dec p h.2)\n" if a == 0 else
                f"  exact key p ((lt_iff _ _).mp {up}) ⟨by have := (lt_iff _ _).mp h.1.1; omega, pb_dec p h.2⟩\n"))
    if a == 0:
        L.append(f"theorem o_{i + 1} : ∀ p, LT p {b + 1} ∨ LT {b} p := by\n"
                 f"  intro p; simp only [lt_iff]; omega\n")
    else:
        L.append(f"theorem o_{i + 1} : ∀ p, LT {a - 1} p → (LT p {b + 1} ∨ LT {b} p) := by\n"
                 f"  intro p; simp only [lt_iff]; omega\n")
    lst = "[" + ", ".join(map(str, qs)) + "]"
    L.append(f"/-- The converse of e_{i + 1}: every listed number of range {i + 1} lies in [{a}, {b}] and satisfies\n"
             f"    Prime_B. With e_{i + 1}, the list is exactly the primes of the interval. -/\n"
             f"theorem listed_{i + 1} : ∀ q, q ∈ ({lst} : List Nat) → ({a} ≤ q ∧ q ≤ {b}) ∧ PB q := by\n"
             f"  have key : ∀ q, q ∈ ({lst} : List Nat) → ({a} ≤ q ∧ q ≤ {b}) ∧ (1 < q ∧ ∀ d, d < q → 1 < d → ¬ d ∣ q) := by decide\n"
             f"  intro q hq\n"
             f"  exact ⟨(key q hq).1, pb_of_dec q (key q hq).2⟩\n")
    TH += [f"n_{i + 1}", f"e_{i + 1}", f"listed_{i + 1}", f"o_{i + 1}"]
cg = " ∨ ".join(f"({lean_R(qs)})" for _, _, qs in RANGES) + " ∨ LT 212 p"
L.append(f"""/-- RLT's C (tests/rlt007s2a/accept/C.rlt): the classification of the primes below 213. -/
theorem c : ∀ p, PB p → {cg} := by
  intro p hp
  rcases o_1 p with h1 | h1
  · exact .inl (e_1 p ⟨h1, hp⟩)
  rcases o_2 p h1 with h2 | h2
  · exact .inr (.inl (e_2 p ⟨⟨h1, h2⟩, hp⟩))
  rcases o_3 p h2 with h3 | h3
  · exact .inr (.inr (.inl (e_3 p ⟨⟨h2, h3⟩, hp⟩)))
  rcases o_4 p h3 with h4 | h4
  · exact .inr (.inr (.inr (.inl (e_4 p ⟨⟨h3, h4⟩, hp⟩))))
  rcases o_5 p h4 with h5 | h5
  · exact .inr (.inr (.inr (.inr (.inl (e_5 p ⟨⟨h4, h5⟩, hp⟩)))))
  rcases o_6 p h5 with h6 | h6
  · exact .inr (.inr (.inr (.inr (.inr (.inl (e_6 p ⟨⟨h5, h6⟩, hp⟩))))))
  · exact .inr (.inr (.inr (.inr (.inr (.inr h6)))))

end RLT007S2a
""")
TH.append("c")
L.append("\n" + "".join(f"#print axioms RLT007S2a.{n}\n" for n in TH))
OUT["lean/RLT007S2a.lean"] = "\n".join(L)


# 7. Statement records (the checked goal is the written goal without outer
# parentheses enclosing it whole; `rlt explain` re-derives and compares it).
BASIS = "docs/RLT-007b-2a-CORRESPONDENCE.md"
basis_sha = hashlib.sha256(open(BASIS, "rb").read()).hexdigest()
DEFS_SHA = hashlib.sha256(open("spec/definitions.rlt", "rb").read()).hexdigest()
reg_ex = {it["id"]: it["text"] for it in T["items"]}


def yq(x):
    return '"' + x.replace("\\", "\\\\").replace('"', '\\"') + '"'


def checked_goal(path):
    """The goal as the checker states it: `x < y` and `d ∣ n` unfolded per
    spec/definitions.rlt, and the inner ∀d of Prime_B parenthesised as the
    printer does. `rlt explain` re-derives it and refuses a record that differs."""
    g = goal_of(OUT[path])
    assert g.count("¬(d ∣ p)))") == 1 and g.count("∧ ∀d. ") == 1, path
    g = g.replace("¬(d ∣ p)))", "¬(d ∣ p))))").replace("∧ ∀d. ", "∧ (∀d. ")
    g = re.sub(r"¬\((\w+) ∣ (\w+)\)", r"¬(∃k. \2 = \1 × k)", g)
    g = re.sub(r"(\w+) < (\w+)", r"(∃z. S(z) + \1 = \2)", g)
    assert "<" not in g and "∣" not in g, path
    return g


RECS = [("PRIMES-BELOW-213", f"{ACC}/C.rlt", ["c", "pb_iff_std"],
         "every p with Prime_B(p) is one of the 47 listed primes, in its range, or 212 < p",
         "Over ℕ, with Prime_B equivalent to the standard definition of prime (Lean pb_iff_std): every prime below 213 is one of the 47 listed primes, each in its approved range; every other prime exceeds 212. This is the 'every prime p' case split that Lemma 12.1's admissibility needs below 213.")]
for i in range(6):
    a, b, qs = RANGES[i]
    RECS.append((f"PRIMES-RANGE-{i + 1}", f"{ACC}/E_{i + 1}.rlt", [f"e_{i + 1}", f"listed_{i + 1}", "pb_iff_std"],
                 f"every p in [{a}, {b}] with Prime_B(p) is one of {', '.join(map(str, qs))}",
                 f"Over ℕ: every prime in [{a}, {b}] is one of {', '.join(map(str, qs))} (exhaustiveness for range {i + 1}). That each listed number is prime is stage 2b, not claimed here."))
for rid, path, leans, subj, match in RECS:
    cg = checked_goal(path)
    lean = "\n".join(f"    - {{file: lean/RLT007S2a.lean, theorem: RLT007S2a.{t}}}" for t in leans)
    OUT[f"statements/{rid}.yaml"] = f"""# Statement record (RLT-007b stage 2a): the classification of the primes below
# 213 used by Lemma 12.1 (Axiom Math, September 3, 2026 draft): {subj}.
# GENERATED by scripts/gen-rlt007s2a.py; pins as `rlt check {path}` reports them.
format: rlt-statement/1
id: {rid}
subject: "Primes below 213 (for Lemma 12.1's 'every prime p'): {subj}"
source:
  registry: AXIOM-BGP212-2026-09-03
  commit: {T["pdf"]["sha256"]}
  excerpts:
    - {yq(reg_ex["admissible-definition"])}
    - {yq(reg_ex["omitted-primes"])}
kernel:
  proof: {path}
  sha256: {sha(OUT[path])}
  convention: NAT-DEF
  definitions_sha256: {DEFS_SHA}
  axiom_set: ROBINSON-Q
  axiom_set_sha256: 4ad55192222576e3901abc4ee7ea5a6760b64ff212e8c55796ab9847e423b00c
  checked_goal: {yq(cg)}
  assumptions: []
semantic:
  status: partial
  assessed_by: "RLT-007b stage 2a review (docs/RLT-007b-2a-CORRESPONDENCE.md)"
  date: 2026-11-12
  basis: {{path: {BASIS}, sha256: {basis_sha}}}
  assessed_goal: {yq(cg)}
  matches:
    - {yq(match)}
  differs:
    - "Prime_B is the bounded definition (1 < p and no d with 1 < d < p divides p); its equivalence with the standard definition is a Lean theorem over ℕ (pb_iff_std), not an RLT theorem under Robinson's Q. The classification is new mathematics, not a statement of the paper; only the 14 primes of the paper's table are source transcription checked. Positive primality of the listed numbers is stage 2b."
  lean:
{lean}
physical:
  applicability: not-applicable
  reason: "a statement in number theory"
open_questions: []
"""

# 8. Write, or compare (--check).
if MODE == "--check":
    diff = []
    for path, text in sorted(OUT.items()):
        if not os.path.isfile(path) or open(path, encoding="utf-8", newline="").read() != text:
            diff.append(path)
    for d in (ACC, REJ):
        for f in sorted(os.listdir(d)) if os.path.isdir(d) else []:
            if f"{d}/{f}" not in OUT:
                diff.append(f"{d}/{f} (not produced by the generator)")
    for f in sorted(os.listdir("tests/rlt007s2a")):
        if f not in ("accept", "reject", "expected.tsv"):
            diff.append(f"tests/rlt007s2a/{f} (not produced by the generator)")
    stmts = sorted(f for f in os.listdir("statements") if f.startswith("PRIMES-"))
    if stmts != sorted(os.path.basename(p) for p in OUT if p.startswith("statements/")):
        diff.append("statements/PRIMES-* (an extra or missing record)")
    for f in sorted(os.listdir("lean")):
        if f.startswith("RLT007S2a") and f"lean/{f}" not in OUT:
            diff.append(f"lean/{f} (not produced by the generator)")
    for d in diff:
        print(f"DIFFERS  {d}")
    print(f"scripts/gen-rlt007s2a.py --check: {len(OUT)} outputs, {len(diff)} differ")
    sys.exit(0 if not diff else 1)

for d in (ACC, REJ):
    os.makedirs(d, exist_ok=True)
    for old in os.listdir(d):
        os.remove(os.path.join(d, old))
for old in os.listdir("statements"):
    if old.startswith("PRIMES-"):
        os.remove(os.path.join("statements", old))
for path, text in OUT.items():
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(text)
print(f"{len(OUT)} outputs written: 6 non-prime certificates, 6 exhaustiveness theorems, 6 order splits, "
      f"the classification, {len(rows)} rejected cases, Lean comparison, {len(RECS)} statement records")

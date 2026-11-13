#!/usr/bin/env python3
"""RLT-007b stage 4 generator (untrusted): the omitted residue classes of H₄₅
for the 33 primes 47 … 211.

For each prime p from 47 to 211 and its class r_p, and each element h of H₄₅:

    D_PPP_HHH:  ¬(∃k. h = (p × k) + r_p)                      (1,485 certificates)
    OM_PPP:     ⋀_{h ∈ H₄₅} ¬(∃k. h = (p × k) + r_p)          (33 summaries, left-nested,
                                                               citing their 45 certificates)

The summaries have exactly the shape of RLT-006's per-prime theorems.

Inputs:
  * H₄₅ from the reviewed transcription (item `h45-tuple`);
  * the primes 47 … 211: the primes of the approved stage-2a ranges R2 … R6
    (docs/RLT-007b-2-DESIGN.md §0), recomputed here by trial division;
  * the classes r_p: for each p the least class omitted by H₄₅ (the choice
    measured in RLT-007c); recomputed here and refused on any difference.
    The paper lists classes only for p ≤ 43 and argues p ≥ 47 by counting, so
    these classes are RLT's choice, not a transcription.

Certificate (the RLT-007c direct scheme, unchanged): peel the r successors of
(p × k) + r, leaving h − r = p × k; then split k with Q3 at most ⌊(h − r)/p⌋ + 1
times. k = 0 gives h − r = 0, refuted by Q2. k = S(k′) gives
n = (p × k′) + p, and a lemma proved once per layer over a fresh parameter w,
∀w. (n = w + p → n − p = w) (n ≥ p) or ∀w. ¬(n = w + p) (n < p), is
instantiated once at p × k′. When h < r, peeling h successors leaves
0 = S^(r−h)(p × k), refuted by Q2. Robinson's Q only; no induction.

Outputs:
  tests/rlt007s4/gen/D_PPP_HHH.rlt   1,485 certificates; GENERATED, not committed
                                     (.gitignore); pinned by the manifest and the
                                     summaries' citations
  tests/rlt007s4/manifest.sha256     SHA-256 of the 1,485 files
  tests/rlt007s4/accept/OM_PPP.rlt   33 per-prime summaries
  tests/rlt007s4/reject/*.rlt        rejected cases; tests/rlt007s4/expected.tsv
  statements/OMITS-H45-PPP.yaml      33 statement records
  lean/RLT007S4.lean                 Lean comparison

Usage: gen-rlt007s4.py RLT_BINARY [--materialize-only | --check | --self-test]
  --materialize-only  writes only tests/rlt007s4/gen/ (used before checking)
  --check             writes nothing; regenerates every output in memory and
                      compares, with exact file sets (needs the materialized gen/)
  --self-test         runs the input checks on altered inputs (each must be refused)
"""
import hashlib
import os
import re
import sys

import yaml

if (len(sys.argv) not in (2, 3) or sys.argv[1].startswith("--")
        or (len(sys.argv) == 3 and sys.argv[2] not in ("--materialize-only", "--check", "--self-test"))):
    sys.exit("usage: gen-rlt007s4.py RLT_BINARY [--materialize-only | --check | --self-test]")
MODE = sys.argv[2] if len(sys.argv) == 3 else ""
root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
os.chdir(root)

T = yaml.safe_load(open("sources/transcriptions/AXIOM-BGP212-2026-09-03.yaml", encoding="utf-8"))
ITEMS = {it["id"]: it for it in T["items"]}
H = list(ITEMS["h45-tuple"]["numbers"])

# The approved data: the primes of ranges R2 … R6 and the class for each.
TABLE = [(47, 1), (53, 6), (59, 6), (61, 1), (67, 1), (71, 4), (73, 3), (79, 3), (83, 3), (89, 1), (97, 1),
         (101, 3), (103, 1), (107, 1), (109, 3), (113, 4), (127, 1), (131, 1), (137, 1), (139, 3), (149, 3),
         (151, 1), (157, 1), (163, 3), (167, 1), (173, 1), (179, 4), (181, 3), (191, 4), (193, 3), (197, 1),
         (199, 3), (211, 3)]


def is_prime(n):
    return n > 1 and all(n % d for d in range(2, n))


def input_checks(h, table):
    bad = []
    if len(h) != 45 or any(a >= b for a, b in zip(h, h[1:])) or not h or h[0] != 0 or h[-1] != 212:
        bad.append("the tuple is not 45 increasing entries from 0 to 212")
    ps = [p for p, _ in table]
    if ps != [p for p in range(47, 213) if is_prime(p)]:
        bad.append("the primes are not exactly the primes 47 … 211, in order")
    for p, r in table:
        if not 0 <= r < p:
            bad.append(f"the class {r} is not a residue mod {p}")
        elif any(x % p == r for x in h):
            bad.append(f"the class {r} mod {p} is occupied")
        elif r != min(c for c in range(p) if all(x % p != c for x in h)):
            bad.append(f"the class {r} mod {p} is not the least omitted class")
    return bad


if MODE == "--self-test":
    def alt_class(p, r):
        return [(q, r if q == p else s) for q, s in TABLE]
    cases = [
        ("an occupied class (2 mod 47)", H, alt_class(47, 2)),
        ("an omitted class that is not the least (7 mod 53)", H, alt_class(53, 7)),
        ("a class outside 0 … p − 1 (211 mod 211)", H, alt_class(211, 211)),
        ("a prime missing (101)", H, [q for q in TABLE if q[0] != 101]),
        ("a composite added (91, class 1)", H, sorted(TABLE + [(91, 1)])),
        ("an altered tuple element (48 in place of 50)", H[:9] + [48] + H[10:], TABLE),
        ("44 tuple entries", H[:-1], TABLE),
    ]
    n = 0
    for name, hh, tt in cases:
        r = input_checks(hh, tt)
        print(f"{'refused ' if r else 'ACCEPTED'} {name}: {'; '.join(r)}")
        n += 1 if r else 0
    print(f"generator self-test: {n} of {len(cases)} altered inputs refused")
    sys.exit(0 if n == len(cases) else 1)

bad = input_checks(H, TABLE)
if bad:
    sys.exit("input check failed: " + "; ".join(bad))

GEN, ACC, REJ = "tests/rlt007s4/gen", "tests/rlt007s4/accept", "tests/rlt007s4/reject"
OUT = {}


def S(t, k):
    assert k >= 0, k
    for _ in range(k):
        t = f"(S {t})"
    return t


def sha(text):
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def claim(p, h, r):
    return f"(not (exists k (= {h} (+ (* {p} k) {r}))))"


def written(p, h, r):
    return f"¬(∃k. {h} = ({p} × k) + {r})"


def proof(pid, comment, goal, steps, params=(), cites=()):
    c = ""
    if cites:
        c = "  (cites\n" + "\n".join(f'    ({l} "{pa}" "{hs}")' for l, pa, hs in cites) + ")\n"
    pr = f"  (params {' '.join(dict.fromkeys(params))})\n" if params else ""
    body = "\n".join("    " + s for s in steps)
    return (f"; {comment}\n(rlt-proof\n  (id RLT007S4.{pid})\n  (convention NAT-DEF)\n  (goal \"{goal}\")\n"
            f"  (mode unconditional)\n  (axioms ROBINSON-Q)\n{c}{pr}  (steps\n{body}))\n")


def certificate_steps(h, p, r, force=False):
    """Steps proving ¬(∃k. h = (p × k) + r). With force=True (rejected cases
    only) an occupied class produces a step the kernel refuses instead of
    raising."""
    st, params = [], ["k0"]
    st.append(f"(H (exists k (= {h} (+ (* {p} k) {r}))) (hyp hx))")
    st.append(f"(F (= {h} (+ (* {p} k0) {r})) (hyp h0))")
    X = f"(* {p} k0)"
    st.append(f"(r0 (= (+ {X} 0) {X}) (inst add_zero (x {X})))")
    prev = "r0"
    for j in range(1, r + 1):
        st += [f"(ra{j} (= (+ {X} {j}) (S (+ {X} {j - 1}))) (inst add_succ (x {X}) (y {j - 1})))",
               f"(rc{j} (= (S (+ {X} {j - 1})) {S(X, j)}) (cong S {prev}))",
               f"(r{j} (= (+ {X} {j}) {S(X, j)}) (trans ra{j} rc{j}))"]
        prev = f"r{j}"
    st.append(f"(T (= {h} {S(X, r)}) (trans F {prev}))")
    cur = "T"
    if h < r:
        for j in range(h, 0, -1):
            a, b = j - 1, S(X, r - h + j - 1)
            st += [f"(pi{j} (imp (= (S {a}) (S {b})) (= {a} {b})) (inst q1_succ_inj (x {a}) (y {b})))",
                   f"(pl{j} (= {a} {b}) (imp-elim pi{j} {cur}))"]
            cur = f"pl{j}"
        st += [f"(q2 (not (= 0 {S(X, r - h)})) (inst q2_zero_ne_succ (x {S(X, r - h - 1)})))",
               f"(fin false (imp-elim q2 {cur}))"]
        last = "fin"
    else:
        for j in range(r, 0, -1):
            a, b = h - r + j - 1, S(X, j - 1)
            st += [f"(pi{j} (imp (= (S {a}) (S {b})) (= {a} {b})) (inst q1_succ_inj (x {a}) (y {b})))",
                   f"(pl{j} (= {a} {b}) (imp-elim pi{j} {cur}))"]
            cur = f"pl{j}"
        if h == r:
            if not force:
                raise ValueError(f"{h} is in the class {r} mod {p}")
            st += [f"(q2 (not (= 0 {X})) (inst q2_zero_ne_succ (x {X})))", f"(fin false (imp-elim q2 {cur}))"]
            last = "fin"
        else:
            last = layer(st, params, p, h - r, 0, cur, force)
    st.append(f"(E false (exists-elim H k0 h0 {last}))")
    st.append(f"(G {claim(p, h, r)} (imp-intro hx E))")
    return st, params


def layer(st, params, p, n, i, fact, force):
    """fact : n = p·k_i (n ≥ 1). Appends steps; returns the label of `false`."""
    k, k1, w = f"k{i}", f"k{i + 1}", f"w{i}"
    params += [k1, w]
    L = f"L{i}"
    st.append(f"({L}q3 (or (= {k} 0) (exists y (= {k} (S y)))) (inst q3_zero_or_succ (x {k})))")
    st += [f"({L}zh (= {k} 0) (hyp z{i}))",
           f"({L}z1 (= {n} (* {p} 0)) (subst {L}zh (at u (= {n} (* {p} u))) {fact}))",
           f"({L}z2 (= (* {p} 0) 0) (inst mul_zero (x {p})))",
           f"({L}z3 (= {n} 0) (trans {L}z1 {L}z2))",
           f"({L}z4 (= 0 {n}) (symm {L}z3))",
           f"({L}z5 (not (= 0 {n})) (inst q2_zero_ne_succ (x {n - 1})))",
           f"({L}zf false (imp-elim {L}z5 {L}z4))"]
    st += [f"({L}sh (exists y (= {k} (S y))) (hyp s{i}))",
           f"({L}s0 (= {k} (S {k1})) (hyp s{i}0))",
           f"({L}s1 (= {n} (* {p} (S {k1}))) (subst {L}s0 (at u (= {n} (* {p} u))) {fact}))",
           f"({L}s2 (= (* {p} (S {k1})) (+ (* {p} {k1}) {p})) (inst mul_succ (x {p}) (y {k1})))",
           f"({L}s3 (= {n} (+ (* {p} {k1}) {p})) (trans {L}s1 {L}s2))"]
    st.append(f"({L}w0 (= {n} (+ {w} {p})) (hyp lw{i}))")
    cur, m, jlast = f"{L}w0", n, p
    for j in range(p, 0, -1):
        if m == 0:
            break
        st += [f"({L}a{j} (= (+ {w} {j}) (S (+ {w} {j - 1}))) (inst add_succ (x {w}) (y {j - 1})))",
               f"({L}t{j} (= {m} (S (+ {w} {j - 1}))) (trans {cur} {L}a{j}))",
               f"({L}i{j} (imp (= (S {m - 1}) (S (+ {w} {j - 1}))) (= {m - 1} (+ {w} {j - 1}))) (inst q1_succ_inj (x {m - 1}) (y (+ {w} {j - 1}))))",
               f"({L}p{j} (= {m - 1} (+ {w} {j - 1})) (imp-elim {L}i{j} {L}t{j}))"]
        cur, m, jlast = f"{L}p{j}", m - 1, j - 1
    if n >= p:
        st += [f"({L}az (= (+ {w} 0) {w}) (inst add_zero (x {w})))",
               f"({L}wn (= {n - p} {w}) (trans {cur} {L}az))",
               f"({L}wi (imp (= {n} (+ {w} {p})) (= {n - p} {w})) (imp-intro lw{i} {L}wn))",
               f"({L}wa (forall v (imp (= {n} (+ v {p})) (= {n - p} v))) (forall-intro {w} {L}wi))",
               f"({L}we (imp (= {n} (+ (* {p} {k1}) {p})) (= {n - p} (* {p} {k1}))) (forall-elim {L}wa (* {p} {k1})))",
               f"({L}nf (= {n - p} (* {p} {k1})) (imp-elim {L}we {L}s3))"]
        if n - p == 0:
            if not force:
                raise ValueError("an occupied class")
            # 0 = p·k1 is consistent (k1 = 0): the step below is refused.
            st += [f"({L}q (not (= 0 (* {p} {k1}))) (inst q2_zero_ne_succ (x (* {p} {k1}))))",
                   f"({L}ff false (imp-elim {L}q {L}nf))"]
            sf = f"{L}ff"
        else:
            sf = layer(st, params, p, n - p, i + 1, f"{L}nf", force)
    else:
        st += [f"({L}b1 (= (+ {w} {jlast}) (S (+ {w} {jlast - 1}))) (inst add_succ (x {w}) (y {jlast - 1})))",
               f"({L}b2 (= 0 (S (+ {w} {jlast - 1}))) (trans {cur} {L}b1))",
               f"({L}b3 (not (= 0 (S (+ {w} {jlast - 1})))) (inst q2_zero_ne_succ (x (+ {w} {jlast - 1}))))",
               f"({L}bf false (imp-elim {L}b3 {L}b2))",
               f"({L}wi (not (= {n} (+ {w} {p}))) (imp-intro lw{i} {L}bf))",
               f"({L}wa (forall v (not (= {n} (+ v {p})))) (forall-intro {w} {L}wi))",
               f"({L}we (not (= {n} (+ (* {p} {k1}) {p}))) (forall-elim {L}wa (* {p} {k1})))",
               f"({L}nf false (imp-elim {L}we {L}s3))"]
        sf = f"{L}nf"
    st.append(f"({L}sf false (exists-elim {L}sh {k1} s{i}0 {sf}))")
    st.append(f"({L}f false (or-cases {L}q3 z{i} {L}zf s{i} {L}sf))")
    return f"{L}f"


def gen_path(p, h):
    return f"{GEN}/D_{p:03d}_{h:03d}.rlt"


def element(p, h, r, note=None, force=False):
    st, ps = certificate_steps(h, p, r, force)
    comment = note or (f"RLT-007b stage 4 element certificate: H45 omits the class {r} mod {p} at h = {h}. "
                       "Generated by scripts/gen-rlt007s4.py (not committed); pinned by tests/rlt007s4/manifest.sha256")
    return proof(f"D_{p:03d}_{h:03d}", comment, written(p, h, r), st, ps)


# 1. The 1,485 certificates.
for p, r in TABLE:
    for h in H:
        OUT[gen_path(p, h)] = element(p, h, r)

if MODE == "--materialize-only":
    os.makedirs(GEN, exist_ok=True)
    for old in os.listdir(GEN):
        os.remove(os.path.join(GEN, old))
    for p, r in TABLE:
        for h in H:
            with open(gen_path(p, h), "w", encoding="utf-8", newline="\n") as fh:
                fh.write(OUT[gen_path(p, h)])
    print(f"{len(TABLE) * len(H)} element certificates written to {GEN}/")
    sys.exit(0)

OUT["tests/rlt007s4/manifest.sha256"] = "".join(
    f"{sha(OUT[gen_path(p, h)])}  {gen_path(p, h)}\n" for p, _ in TABLE for h in H)


# 2. The 33 per-prime summaries (RLT-006's shape).
def summary(p, r, cite_paths=None, claims_h=None):
    cite_paths = cite_paths or {h: (gen_path(p, h), sha(OUT[gen_path(p, h)])) for h in H}
    claims_h = claims_h or list(H)
    cites, st, lab, conj = [], [], None, None
    for i, (h, hc) in enumerate(zip(H, claims_h)):
        path, s = cite_paths[h]
        cites.append((f"E{i}", path, s))
        g = claim(p, hc, r)
        st.append(f"(f{i} {g} (cite E{i}))")
        if conj is None:
            conj, lab = g, f"f{i}"
        else:
            conj = f"(and {conj} {g})"
            st.append(f"(c{i} {conj} (and-intro {lab} f{i}))")
            lab = f"c{i}"
    st[-1] = st[-1].replace(f"({lab} ", "(G ", 1)
    goal = written(p, claims_h[0], r)
    for hc in claims_h[1:]:
        goal = f"({goal} ∧ {written(p, hc, r)})"
    return cites, st, goal


def om_path(p):
    return f"{ACC}/OM_{p:03d}.rlt"


for p, r in TABLE:
    cites, st, goal = summary(p, r)
    OUT[om_path(p)] = proof(
        f"OM_{p:03d}",
        f"RLT-007b stage 4 per-prime theorem: the reductions of H45 mod {p} omit the class {r}. "
        f"Cites the 45 element certificates in {GEN}/ (generated; pinned by SHA-256 here and in tests/rlt007s4/manifest.sha256)",
        goal, st, cites=cites)

# 3. Rejected cases.
rows = []


def rej(name, text, code, reason):
    OUT[f"{REJ}/{name}.rlt"] = text
    rows.append(f"{REJ}/{name}.rlt\t{code}\t{reason}")


rej("SQ01_altered_class_2_mod_47_at_2",
    element(47, 2, 2, "REJECT CASE: an altered residue class, 2 mod 47, is occupied by 2 = 47 × 0 + 2", force=True),
    1, "REJECT at step q2 — step claims `¬(0 = 47 × k0)`")
rej("SQ02_altered_member_48_mod_47",
    element(47, 48, 1, "REJECT CASE: an altered tuple member, 48 = 47 × 1 + 1, lies in the class 1 mod 47", force=True),
    1, "REJECT at step L0q — step claims `¬(0 = 47 × k1)`")
cites, st, goal = summary(211, 3, claims_h=H[:-1] + [214])
rej("SQ03_summary_altered_member_214",
    proof("SQ03", "REJECT CASE: OM_211 with its last member altered to 214 (= 211 + 3) in the goal and the steps; the cited certificate is for 212",
          goal, st, cites=cites),
    1, "REJECT at step f44 — step claims `¬(∃k. 214 = (211 × k) + 3)`, the cited theorem `E44` proves `¬(∃k. 212 = (211 × k) + 3)`")
alt = {h: (gen_path(211, h), sha(OUT[gen_path(211, h)])) for h in H}
alt[212] = (gen_path(199, 212), sha(OUT[gen_path(199, 212)]))
cites, st, goal = summary(211, 3, cite_paths=alt)
rej("SQ04_summary_cites_other_prime",
    proof("SQ04", "REJECT CASE: OM_211 citing the certificate for 212 mod 199 in place of 212 mod 211", goal, st, cites=cites),
    1, "REJECT at step f44 — step claims `¬(∃k. 212 = (211 × k) + 3)`, the cited theorem `E44` proves `¬(∃k. 212 = (199 × k) + 3)`")
alt = {h: (gen_path(211, h), sha(OUT[gen_path(211, h)])) for h in H}
alt[0] = (gen_path(211, 0), "f" * 64)
cites, st, goal = summary(211, 3, cite_paths=alt)
rej("SQ05_summary_stale_certificate_hash",
    proof("SQ05", "REJECT CASE: OM_211 recording a stale SHA-256 for a certificate", goal, st, cites=cites),
    2, f"MALFORMED — citation `E0`: {gen_path(211, 0)} has SHA-256")
alt = {h: (gen_path(211, h), sha(OUT[gen_path(211, h)])) for h in H}
alt[0] = (f"{GEN}/D_211_999.rlt", sha(OUT[gen_path(211, 0)]))
cites, st, goal = summary(211, 3, cite_paths=alt)
rej("SQ06_summary_missing_certificate",
    proof("SQ06", "REJECT CASE: OM_211 citing a generated certificate that does not exist", goal, st, cites=cites),
    2, f"MALFORMED — citation `E0`: cited path `{GEN}/D_211_999.rlt`: entity not found")
cites, st, goal = summary(47, 1)
rej("SQ07_summary_class_changed_in_statement",
    proof("SQ07", "REJECT CASE: OM_47 whose written statement names the class 2; the steps prove the class 1",
          goal.replace(") + 1)", ") + 2)"), st, cites=cites),
    1, "the goal is `(((((((((((((((((((((((((((((((((((((((((((¬(∃k. 0 = (47 × k) + 2)")
OUT["tests/rlt007s4/expected.tsv"] = ("# file\texit\trequired substring of the verdict (RLT-007b stage 4 rejected cases)\n"
                                     + "\n".join(rows) + "\n")


# 4. Lean comparison.
def lean_lit(p, r):
    t = f"¬(∃ k, {H[0]} = {p} * k + {r})"
    for h in H[1:]:
        t = f"({t} ∧ ¬(∃ k, {h} = {p} * k + {r}))"
    return t


SMALL = list(zip(ITEMS["omitted-primes"]["numbers"], ITEMS["omitted-classes"]["numbers"]))
L = [f"""/-
RLT-007b stage 4, Lean comparison: GENERATED by scripts/gen-rlt007s4.py.
Core Lean only. Run: cd lean && lean RLT007S4.lean

What this file proves, in Lean (none of it is an RLT derivation):
  * om_P (33): RLT's per-prime statements OM_PPP, symbol for symbol (RLT's `×`
    read as `*` on Nat; the statements use no `<` or `∣`);
  * omit_iff: ¬(∃k, h = p*k + r) ↔ h % p ≠ r for r < p (the residue reading);
  * table_facts: the 33 primes are exactly the primes 47 … 211, and each class
    is below its prime, omitted by H45, and the least omitted class;
  * below_213_covered: every prime below 213 is in RLT-006's table (the
    transcribed 2 … 43) or in this table, so with stage 3 (p > 212) every
    prime has a listed omitted class.
-/

namespace RLT007S4

def H45 : List Nat := [{", ".join(map(str, H))}]

/-- The 33 primes 47 … 211 and their classes (RLT's choice: the least omitted class). -/
def table : List (Nat × Nat) := [{", ".join(f"({p}, {r})" for p, r in TABLE)}]

/-- RLT-006's transcribed table, for the coverage theorem. -/
def small : List (Nat × Nat) := [{", ".join(f"({p}, {r})" for p, r in SMALL)}]

set_option maxRecDepth 100000
set_option maxHeartbeats 4000000

def IsPrimeB (p : Nat) : Bool := 2 ≤ p && (List.range p).all (fun d => d < 2 || p % d != 0)

theorem omit_iff (h p r : Nat) (hr : r < p) : (¬ ∃ k, h = p * k + r) ↔ h % p ≠ r := by
  constructor
  · intro hn heq
    exact hn ⟨h / p, by have := Nat.div_add_mod h p; omega⟩
  · intro hne ⟨k, hk⟩
    apply hne
    rw [hk, Nat.mul_add_mod, Nat.mod_eq_of_lt hr]

theorem table_facts :
    table.map Prod.fst = (List.range 213).filter (fun p => 47 ≤ p && IsPrimeB p) ∧
    ∀ q ∈ table, q.2 < q.1 ∧ (∀ h ∈ H45, h % q.1 ≠ q.2) ∧
      ∀ c, c < q.2 → ∃ h ∈ H45, h % q.1 = c := by
  decide

theorem below_213_covered :
    ∀ p, p < 213 → IsPrimeB p = true → p ∈ (small ++ table).map Prod.fst := by
  decide

theorem element (p r h : Nat) (hr : r < p) (hh : h % p ≠ r) : ¬ ∃ k, h = p * k + r :=
  (omit_iff h p r hr).mpr hh

"""]
TH = ["omit_iff", "table_facts", "below_213_covered", "element"]
for p, r in TABLE:
    L.append(f"/-- RLT's `admissible.h45.omits.{p}_{r}` (tests/rlt007s4/accept/OM_{p:03d}.rlt), literally. -/\n"
             f"theorem om_{p} :\n    {lean_lit(p, r)} := by\n"
             f"  repeat' constructor\n"
             f"  all_goals exact element {p} {r} _ (by decide) (by decide)\n")
    TH.append(f"om_{p}")
L.append("end RLT007S4\n\n" + "".join(f"#print axioms RLT007S4.{n}\n" for n in TH))
OUT["lean/RLT007S4.lean"] = "\n".join(L)


# 5. Statement records.
BASIS = "docs/RLT-007b-4-CORRESPONDENCE.md"
basis_sha = hashlib.sha256(open(BASIS, "rb").read()).hexdigest()


def yq(x):
    return '"' + x.replace("\\", "\\\\").replace('"', '\\"') + '"'


def goal_of(text):
    return re.search(r'\(goal "([^"]*)"\)', text).group(1)


for p, r in TABLE:
    path = om_path(p)
    g = goal_of(OUT[path])
    checked = g[1:-1]
    OUT[f"statements/OMITS-H45-{p:03d}.yaml"] = f"""# Statement record (RLT-007b stage 4): Lemma 12.1 (Axiom Math, September 3, 2026
# draft), the prime {p}: H45 omits the class {r} mod {p}.
# GENERATED by scripts/gen-rlt007s4.py; pins as `rlt check {path}` reports them.
format: rlt-statement/1
id: OMITS-H45-{p:03d}
subject: "Lemma 12.1 (Axiom Math, September 3, 2026 draft), the prime {p}: the reductions of H45 modulo {p} omit the class {r}"
source:
  registry: AXIOM-BGP212-2026-09-03
  commit: {T["pdf"]["sha256"]}
  excerpts:
    - {yq(ITEMS["h45-tuple"]["text"])}
    - {yq(ITEMS["admissible-definition"]["text"])}
    - {yq(ITEMS["large-primes"]["text"])}
kernel:
  proof: {path}
  sha256: {sha(OUT[path])}
  convention: NAT-DEF
  axiom_set: ROBINSON-Q
  axiom_set_sha256: 4ad55192222576e3901abc4ee7ea5a6760b64ff212e8c55796ab9847e423b00c
  checked_goal: {yq(checked)}
  assumptions: []
semantic:
  status: partial
  assessed_by: "RLT-007b stage 4 review (docs/RLT-007b-4-CORRESPONDENCE.md)"
  date: 2026-11-13
  basis: {{path: {BASIS}, sha256: {basis_sha}}}
  assessed_goal: {yq(checked)}
  matches:
    - "For every element h of the transcribed H45, h is not {p}*k + {r} for any natural k; over the naturals, with 0 <= {r} < {p}, the reductions of H45 modulo {p} omit the class {r} (Lean omit_iff, om_{p}). This is the case p = {p} of Lemma 12.1's admissibility."
  differs:
    - "The paper argues p >= 47 by counting and lists no class for {p}; the class {r} is RLT's choice, the least class H45 omits mod {p} (Lean table_facts). That {p} is prime is stage 2a's listing (exhaustiveness), not part of this statement; the assembly with the other primes is RLT-007d."
  lean:
    - {{file: lean/RLT007S4.lean, theorem: RLT007S4.om_{p}}}
    - {{file: lean/RLT007S4.lean, theorem: RLT007S4.table_facts}}
physical:
  applicability: not-applicable
  reason: "a statement in number theory"
open_questions: []
"""

# 6. Write, or compare (--check).
if MODE == "--check":
    diff = []
    for path, text in sorted(OUT.items()):
        if not os.path.isfile(path) or open(path, encoding="utf-8", newline="").read() != text:
            diff.append(path)
    for d in (GEN, ACC, REJ):
        for f in sorted(os.listdir(d)) if os.path.isdir(d) else []:
            if f"{d}/{f}" not in OUT:
                diff.append(f"{d}/{f} (not produced by the generator)")
    for f in sorted(os.listdir("tests/rlt007s4")):
        if f not in ("gen", "accept", "reject", "manifest.sha256", "expected.tsv"):
            diff.append(f"tests/rlt007s4/{f} (not produced by the generator)")
    stmts = sorted(f for f in os.listdir("statements") if f.startswith("OMITS-H45-"))
    if stmts != sorted(os.path.basename(x) for x in OUT if x.startswith("statements/")):
        diff.append("statements/OMITS-H45-* (an extra or missing record)")
    for f in sorted(os.listdir("lean")):
        if f.startswith("RLT007S4") and f"lean/{f}" not in OUT:
            diff.append(f"lean/{f} (not produced by the generator)")
    for d in diff[:20]:
        print(f"DIFFERS  {d}")
    print(f"scripts/gen-rlt007s4.py --check: {len(OUT)} outputs, {len(diff)} differ")
    sys.exit(0 if not diff else 1)

for d in (GEN, ACC, REJ):
    os.makedirs(d, exist_ok=True)
    for old in os.listdir(d):
        os.remove(os.path.join(d, old))
for old in os.listdir("statements"):
    if old.startswith("OMITS-H45-"):
        os.remove(os.path.join("statements", old))
for path, text in OUT.items():
    with open(path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(text)
print(f"{len(OUT)} outputs written: {len(TABLE) * len(H)} certificates, manifest, {len(TABLE)} summaries, "
      f"{len(rows)} rejected cases, Lean comparison, {len(TABLE)} statement records")

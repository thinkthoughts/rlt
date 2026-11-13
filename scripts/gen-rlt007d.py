#!/usr/bin/env python3
"""RLT-007d generator (untrusted): the H45 composition bundle.

Writes, from the reviewed transcription and the approved tables only:
  tests/rlt007d/assemble/ADM_*.rlt   the eight conditional assembly proofs
                                     (layer 1: six ranges and the tail; layer 2: all primes)
  tests/rlt007d/spec/SPEC_*.rlt      one specification probe per node: the approved
                                     statement as a written goal, checked by the kernel as
                                     {s} ⊢ φ with s : φ, so `rlt compose` compares each node's
                                     statement with the kernel's own reading of the approved text
  compose/h45.yaml                   the bundle: 62 nodes, 56 discharges, the six-statement result
  tests/rlt007d/accept/*.yaml        smaller bundles that must compose (controls)
  tests/rlt007d/negative/...         bundles and variant files that must be refused;
                                     tests/rlt007d/expected.tsv names the refusal class and reason
  lean/RLT007D.lean                  Lean comparison of the same composition

Nothing here is trusted: `rlt compose` re-checks every proof and compares
canonical kernel formulas, and scripts/compose-check.py and
cli/tests/rlt007d.rs rebuild the approved statements independently.

Usage: gen-rlt007d.py RLT_BINARY [--check]
"""
import hashlib
import os
import re
import sys

import yaml

if len(sys.argv) not in (2, 3) or sys.argv[1].startswith("--") or (len(sys.argv) == 3 and sys.argv[2] != "--check"):
    sys.exit("usage: gen-rlt007d.py RLT_BINARY [--check]")
MODE = sys.argv[2] if len(sys.argv) == 3 else ""
root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
os.chdir(root)

T = yaml.safe_load(open("sources/transcriptions/AXIOM-BGP212-2026-09-03.yaml", encoding="utf-8"))
IT = {i["id"]: i for i in T["items"]}
H = list(IT["h45-tuple"]["numbers"])
SMALL = list(zip(IT["omitted-primes"]["numbers"], IT["omitted-classes"]["numbers"]))
INTERVALS = [(0, 40), (41, 78), (79, 112), (113, 156), (157, 190), (191, 212)]


def is_prime(n):
    return n > 1 and all(n % d for d in range(2, n))


RANGES = [[p for p in range(a, b + 1) if is_prime(p)] for a, b in INTERVALS]
MID = [(p, min(c for c in range(p) if all(h % p != c for h in H))) for p in range(47, 213) if is_prime(p)]
CLS = dict(SMALL + MID)
assert len(H) == 45 and sorted(CLS) == [p for r in RANGES for p in r] and len(CLS) == 47
assert all(all(h % p != r for h in H) and r < p for p, r in CLS.items())

OUT = {}
SPEC, ASM, NEG = "tests/rlt007d/spec", "tests/rlt007d/assemble", "tests/rlt007d/negative"


def sha(t):
    return hashlib.sha256(t.encode("utf-8")).hexdigest()


def S(t, k):
    assert k >= 0
    for _ in range(k):
        t = f"(S {t})"
    return t


# ---- Statements: s-expression and written form, built from the inputs. ----
def conj(p, r, hs=H):
    f = f"(not (exists k (= {hs[0]} (+ (* {p} k) {r}))))"
    for h in hs[1:]:
        f = f"(and {f} (not (exists k (= {h} (+ (* {p} k) {r})))))"
    return f


def conj_w(p, r, hs=H):
    f = f"¬(∃k. {hs[0]} = ({p} × k) + {r})"
    for h in hs[1:]:
        f = f"({f} ∧ ¬(∃k. {h} = ({p} × k) + {r}))"
    return f


def adm(t, bound=True):
    return f"(exists r (and (lt r {t}) {conj(t, 'r')}))" if bound else f"(exists r {conj(t, 'r')})"


def adm_w(t, bound=True):
    return f"∃r. (r < {t} ∧ {conj_w(t, 'r')})" if bound else f"∃r. {conj_w(t, 'r')}"


def rng(qs, t):
    f = f"(= {t} {qs[-1]})"
    for q in reversed(qs[:-1]):
        f = f"(or (= {t} {q}) {f})"
    return f


def rng_w(qs, t):
    f = f"{t} = {qs[-1]}"
    for q in reversed(qs[:-1]):
        f = f"({t} = {q} ∨ {f})"
    return f


def PB(t):
    return f"(and (lt 1 {t}) (forall d (imp (and (lt 1 d) (lt d {t})) (not (dvd d {t})))))"


def PB_w(t):
    return f"(1 < {t} ∧ ∀d. ((1 < d ∧ d < {t}) → ¬(d ∣ {t})))"


def disj(t):
    f = f"(lt 212 {t})"
    for qs in reversed(RANGES):
        f = f"(or {rng(qs, t)} {f})"
    return f


def disj_w(t):
    f = f"212 < {t}"
    for qs in reversed(RANGES):
        f = f"({rng_w(qs, t)} ∨ {f})"
    return f


def lconj(items):
    f, w = items[0]
    for g, v in items[1:]:
        f, w = f"(and {f} {g})", f"({w} ∧ {v})"
    return f, w


GROUPS = [list(range(0, 11)), list(range(11, 22)), list(range(22, 33)), list(range(33, 44))]


def distinct_group(k):
    items = []
    for i in GROUPS[k]:
        items.append(lconj([(f"(not (= {H[i]} {H[j]}))", f"¬({H[i]} = {H[j]})") for j in range(i + 1, 45)]))
    return lconj(items)


def bounds():
    return lconj([(f"(and (exists z (= (+ z 0) {h})) (exists z (= (+ z {h}) 212)))",
                   f"((∃z. z + 0 = {h}) ∧ (∃z. z + {h} = 212))") for h in H])


STMT = {}   # node id → (sexp, written)
for p, r in sorted(CLS.items()):
    STMT[f"om.{p}_{r}"] = (conj(p, r), conj_w(p, r))
STMT["tail"] = (f"(forall p (imp (lt 212 p) {conj('p', 1)}))", f"∀p. (212 < p → {conj_w('p', 1)})")
STMT["classify"] = (f"(forall p (imp {PB('p')} {disj('p')}))", f"∀p. ({PB_w('p')} → {disj_w('p')})")
for k in range(4):
    STMT[f"distinct.g{k}"] = distinct_group(k)
STMT["bounds"] = bounds()
for i, qs in enumerate(RANGES, 1):
    STMT[f"adm.range_{i}"] = (f"(forall p (imp {rng(qs, 'p')} {adm('p')}))", f"∀p. ({rng_w(qs, 'p')} → {adm_w('p')})")
STMT["adm.tail"] = (f"(forall p (imp (lt 212 p) {adm('p')}))", f"∀p. (212 < p → {adm_w('p')})")
STMT["adm.all"] = (f"(forall p (imp {PB('p')} {adm('p')}))", f"∀p. ({PB_w('p')} → {adm_w('p')})")
LIB = {f"om.{p}_{r}": f"admissible.h45.omits.{p}_{r}" for p, r in CLS.items()}
LIB.update({"tail": "admissible.h45.omits.tail_1", "classify": "primes.below_213", "bounds": "admissible.h45.bounds"})
LIB.update({f"distinct.g{k}": f"admissible.h45.distinct.g{k}" for k in range(4)})


def lib_fp(lid):
    area = lid.split(".")[0]
    return yaml.safe_load(open(f"lib/{area}/{lid}.yaml", encoding="utf-8"))["fingerprint"]


# ---- Proof files. ----
def proof(pid, goal_w, assumes, steps, params, cites=()):
    a = "".join(f'  (assume {n} "{w}")\n' for n, w in assumes)
    c = ("  (cites\n" + "\n".join(f'    ({l} "{pa}" "{hs}")' for l, pa, hs in cites) + ")\n") if cites else ""
    pr = f"  (params {' '.join(params)})\n" if params else ""
    return (f"; RLT-007d (generated by scripts/gen-rlt007d.py)\n(rlt-proof\n  (id RLT007D.{pid})\n"
            f"  (convention NAT-DEF)\n  (goal \"{goal_w}\")\n{a}  (mode conditional)\n  (axioms ROBINSON-Q)\n"
            f"{c}{pr}  (steps\n" + "\n".join("    " + s for s in steps) + "))\n")


def spec_probe(node, sx=None, w=None, extra=None):
    sx, w = (sx, w) if sx else STMT[node]
    assumes = [("s", w)] + ([extra] if extra else [])
    return proof(f"SPEC_{node.replace('.', '_')}", w, assumes, [f"(G {sx} (hyp s))"], [])


def range_assembly(i, qs):
    """Layer 1: ∀p. (R_i(p) → Adm(p)) from OM(q, r_q) for q in the range."""
    st, params = [], ["pp"]
    assumes = [(f"om{q}", conj_w(q, CLS[q])) for q in qs]
    st.append(f"(HR {rng(qs, 'pp')} (hyp hr))")

    def branch(q, eqlab):
        r, X = CLS[q], f"B{q}_"
        st.append(f"({X}A {conj(q, r)} (hyp om{q}))")
        n = q - r - 1
        st.append(f"({X}l0 (= (+ (S {n}) 0) (S {n})) (inst add_zero (x (S {n}))))")
        prev = f"{X}l0"
        for j in range(1, r + 1):
            st.extend([f"({X}la{j} (= (+ (S {n}) {j}) (S (+ (S {n}) {j - 1}))) (inst add_succ (x (S {n})) (y {j - 1})))",
                       f"({X}lc{j} (= (S (+ (S {n}) {j - 1})) {S(f'(S {n})', j)}) (cong S {prev}))",
                       f"({X}lq{j} (= (+ (S {n}) {j}) {S(f'(S {n})', j)}) (trans {X}la{j} {X}lc{j}))"])
            prev = f"{X}lq{j}"
        st.append(f"({X}L (lt {r} {q}) (exists-intro {n} {prev}))")
        st.append(f"({X}B (and (lt {r} {q}) {conj(q, r)}) (and-intro {X}L {X}A))")
        st.append(f"({X}E {adm(q)} (exists-intro {r} {X}B))")
        st.append(f"({X}s (= {q} pp) (symm {eqlab}))")
        st.append(f"({X}G {adm('pp')} (subst {X}s (at u {adm('u')}) {X}E))")
        return f"{X}G"

    def case(j, lab):
        q = qs[j]
        if j == len(qs) - 1:
            return branch(q, lab)
        st.append(f"(C{j}h (= pp {q}) (hyp c{j}l))")
        left = branch(q, f"C{j}h")
        st.append(f"(C{j}r {rng(qs[j + 1:], 'pp')} (hyp c{j}r))")
        right = case(j + 1, f"C{j}r")
        st.append(f"(C{j} {adm('pp')} (or-cases {lab} c{j}l {left} c{j}r {right}))")
        return f"C{j}"

    res = case(0, "HR")
    st.append(f"(I (imp {rng(qs, 'pp')} {adm('pp')}) (imp-intro hr {res}))")
    st.append(f"(G (forall p (imp {rng(qs, 'p')} {adm('p')})) (forall-intro pp I))")
    return proof(f"ADM_RANGE_{i}", f"∀p. ({rng_w(qs, 'p')} → {adm_w('p')})", assumes, st, params)


def tail_assembly(rename=False, bound=True, extra_assume=False, cite=None, broken=False):
    """Layer 1: ∀p. (212 < p → Adm(p)) from tail_1, with r = 1 and 1 < p proved."""
    st = []
    tw = f"∀p. (212 < p → {conj_w('p', 1)})"
    if rename:  # the same statement over another bound variable name
        tw = f"∀q. (212 < q → {conj_w('q', 1)})"
    assumes = [("tl", tw)] + ([("xx", "0 = 0")] if extra_assume else [])
    st.append(f"(T (forall p (imp (lt 212 p) {conj('p', 1)})) (hyp tl))")
    if cite:
        st.append(f"(Tc (forall p (imp (lt 212 p) {conj('p', 1)})) (cite TT))")
    st.append(f"(Tp (imp (lt 212 pp) {conj('pp', 1)}) (forall-elim T pp))")
    st.append("(HL (lt 212 pp) (hyp hlt))")
    st.append(f"(Cj {conj('pp', 1)} (imp-elim Tp HL))")
    st.append("(Hz (= (+ (S z) 212) pp) (hyp hz))")
    st.append("(e0 (= (+ (S z) 0) (S z)) (inst add_zero (x (S z))))")
    prev = "e0"
    for j in range(1, 213):
        st.extend([f"(ea{j} (= (+ (S z) {j}) (S (+ (S z) {j - 1}))) (inst add_succ (x (S z)) (y {j - 1})))",
                   f"(ec{j} (= (S (+ (S z) {j - 1})) {S('(S z)', j)}) (cong S {prev}))",
                   f"(eq{j} (= (+ (S z) {j}) {S('(S z)', j)}) (trans ea{j} ec{j}))"])
        prev = f"eq{j}"
    w = S("z", 211)
    st += [f"(P1 (= {S('(S z)', 212)} (+ (S z) 212)) (symm eq212))",
           f"(P2 (= {S('(S z)', 212)} pp) (trans P1 Hz))",
           f"(o1 (= (+ (S {w}) 1) (S (+ (S {w}) 0))) (inst add_succ (x (S {w})) (y 0)))",
           f"(o2 (= (+ (S {w}) 0) (S {w})) (inst add_zero (x (S {w}))))",
           f"(o3 (= (S (+ (S {w}) 0)) (S (S {w}))) (cong S o2))",
           f"(o4 (= (+ (S {w}) 1) (S (S {w}))) (trans o1 o3))",
           f"(o5 (= (+ (S {w}) 1) pp) (trans o4 P2))" if not broken else f"(o5 (= (+ (S {w}) 2) pp) (trans o4 P2))",
           f"(L1 (lt 1 pp) (exists-intro {w} o5))",
           "(L2 (lt 1 pp) (exists-elim HL z hz L1))"]
    if bound:
        st += [f"(B (and (lt 1 pp) {conj('pp', 1)}) (and-intro L2 Cj))",
               f"(E {adm('pp')} (exists-intro 1 B))"]
    else:  # a weaker, still true conclusion: the bound r < p dropped
        st += [f"(E {adm('pp', False)} (exists-intro 1 Cj))"]
    st += [f"(I (imp (lt 212 pp) {adm('pp', bound)}) (imp-intro hlt E))",
           f"(G (forall p (imp (lt 212 p) {adm('p', bound)})) (forall-intro pp I))"]
    cites = [("TT", "tests/rlt007s3/accept/TAIL.rlt", sha(open("tests/rlt007s3/accept/TAIL.rlt", encoding="utf-8").read()))] if cite else ()
    return proof("ADM_TAIL", f"∀p. (212 < p → {adm_w('p', bound)})", assumes, st, ["pp", "z"], cites)


def all_assembly(weak=False):
    """Layer 2: ∀p. (Prime_B(p) → Adm(p)) from the classification and layer 1.
    weak: the same proof, concluding the weaker ∀p. (Prime_B(p) → ∃r. OM(p, r))."""
    d = disj("pp")
    assumes = [("cls", STMT["classify"][1])] + [(f"ra{i}", STMT[f"adm.range_{i}"][1]) for i in range(1, 7)]
    assumes.append(("ta", STMT["adm.tail"][1]))
    st = [f"(C {STMT['classify'][0]} (hyp cls))",
          f"(Cp (imp {PB('pp')} {d}) (forall-elim C pp))",
          f"(HP {PB('pp')} (hyp hp))",
          f"(D {d} (imp-elim Cp HP))"]

    def case(i, lab, rest):
        if i == len(RANGES):
            st.extend([f"(TA {STMT['adm.tail'][0]} (hyp ta))",
                       f"(TAp (imp (lt 212 pp) {adm('pp')}) (forall-elim TA pp))",
                       f"(TAe {adm('pp')} (imp-elim TAp {lab}))"])
            return "TAe"
        qs = RANGES[i]
        st.extend([f"(R{i}h {rng(qs, 'pp')} (hyp r{i}l))",
                   f"(RA{i} {STMT[f'adm.range_{i + 1}'][0]} (hyp ra{i + 1}))",
                   f"(RA{i}p (imp {rng(qs, 'pp')} {adm('pp')}) (forall-elim RA{i} pp))",
                   f"(RA{i}e {adm('pp')} (imp-elim RA{i}p R{i}h))"])
        inner = rest[rest.index(rng(qs, 'pp')) + len(rng(qs, 'pp')) + 1:-1]
        st.append(f"(N{i}h {inner} (hyp r{i}r))")
        right = case(i + 1, f"N{i}h", inner)
        st.append(f"(K{i} {adm('pp')} (or-cases {lab} r{i}l RA{i}e r{i}r {right}))")
        return f"K{i}"

    res = case(0, "D", d)
    if weak:
        st += [f"(Wh (and (lt rr pp) {conj('pp', 'rr')}) (hyp hw))",
               f"(Wo {conj('pp', 'rr')} (and-right Wh))",
               f"(We {adm('pp', False)} (exists-intro rr Wo))",
               f"(W {adm('pp', False)} (exists-elim {res} rr hw We))",
               f"(I (imp {PB('pp')} {adm('pp', False)}) (imp-intro hp W))",
               f"(G (forall p (imp {PB('p')} {adm('p', False)})) (forall-intro pp I))"]
        return proof("ADM_ALL", f"∀p. ({PB_w('p')} → {adm_w('p', False)})", assumes, st, ["pp", "rr"])
    st += [f"(I (imp {PB('pp')} {adm('pp')}) (imp-intro hp {res}))",
           f"(G {STMT['adm.all'][0]} (forall-intro pp I))"]
    return proof("ADM_ALL", STMT["adm.all"][1], assumes, st, ["pp"])


ASSEMBLY = {f"adm.range_{i}": (f"{ASM}/ADM_RANGE_{i}.rlt", range_assembly(i, qs)) for i, qs in enumerate(RANGES, 1)}
ASSEMBLY["adm.tail"] = (f"{ASM}/ADM_TAIL.rlt", tail_assembly())
ASSEMBLY["adm.all"] = (f"{ASM}/ADM_ALL.rlt", all_assembly())
for n, (p, t) in ASSEMBLY.items():
    OUT[p] = t
for n in STMT:
    OUT[f"{SPEC}/SPEC_{n.replace('.', '_')}.rlt"] = spec_probe(n)


def spec_pin(n, path=None):
    p = path or f"{SPEC}/SPEC_{n.replace('.', '_')}.rlt"
    return {"path": p, "sha256": sha(OUT[p])}


DISCHARGE = {f"adm.range_{i}": [(f"om{q}", f"om.{q}_{CLS[q]}") for q in qs] for i, qs in enumerate(RANGES, 1)}
DISCHARGE["adm.tail"] = [("tl", "tail")]
DISCHARGE["adm.all"] = [("cls", "classify")] + [(f"ra{i}", f"adm.range_{i}") for i in range(1, 7)] + [("ta", "adm.tail")]
LAYER = {n: 0 for n in LIB}
LAYER.update({f"adm.range_{i}": 1 for i in range(1, 7)})
LAYER.update({"adm.tail": 1, "adm.all": 2})
RESULT = [f"distinct.g{k}" for k in range(4)] + ["bounds", "adm.all"]


def node(n, **over):
    d = {"id": n, "layer": LAYER[n]}
    if LAYER[n] == 0:
        d["library"] = LIB[n]
        d["fingerprint"] = lib_fp(LIB[n])
    else:
        path, _ = ASSEMBLY[n]
        d["assembly"] = {"path": path, "sha256": sha(OUT[path])}
        d["discharge"] = [{"assumption": a, "by": b} for a, b in DISCHARGE[n]]
    d["spec"] = spec_pin(n)
    d.update(over)
    return d


def render(bundle, comment):
    """Deterministic YAML, written out in full."""
    L = [f"# {comment}", "# GENERATED by scripts/gen-rlt007d.py; checked by `rlt compose` (RLT-007d).",
         f"format: {bundle.get('format', 'rlt-compose/1')}", f"id: {bundle['id']}",
         f"axiom_set: {bundle.get('axiom_set', 'ROBINSON-Q')}", f"convention: {bundle.get('convention', 'NAT-DEF')}",
         "definitions:", f"  path: {bundle.get('defs_path', 'spec/definitions.rlt')}",
         f"  sha256: {bundle.get('defs_sha', sha(open('spec/definitions.rlt', encoding='utf-8').read()))}"]
    L.append("nodes:")
    for d in bundle["nodes"]:
        L.append(f"  - id: {d['id']}")
        L.append(f"    layer: {d['layer']}")
        for k in ("library", "fingerprint"):
            if k in d:
                L.append(f"    {k}: {d[k]}")
        if "assembly" in d:
            L += ["    assembly:", f"      path: {d['assembly']['path']}", f"      sha256: {d['assembly']['sha256']}"]
        if "discharge" in d:
            L.append("    discharge:" if d["discharge"] else "    discharge: []")
            for r in d["discharge"]:
                L.append(f"      - {{assumption: {r['assumption']}, by: {r['by']}}}")
        L += ["    spec:", f"      path: {d['spec']['path']}", f"      sha256: {d['spec']['sha256']}"]
    L.append("result:")
    for r in bundle["result"]:
        L.append(f"  - {r}")
    return "\n".join(L) + "\n"


OUT["compose/approved-specifications.tsv"] = (
    "# RLT-007d approved specifications: node id, specification probe, SHA-256 (GENERATED by scripts/gen-rlt007d.py).\n"
    "# Each probe states the approved formula, rebuilt from the transcription; `rlt compose` requires every node to carry it.\n"
    + "".join(f"{n}\t{spec_pin(n)['path']}\t{spec_pin(n)['sha256']}\n" for n in STMT))
NOTE = ("outside this check: the six statements' reading as Lemma 12.1 of AXIOM-BGP212-2026-09-03 — source transcription checked "
        "(scripts/compose-check.py, cli/tests/rlt007d.rs); semantic correspondence assessed (docs/RLT-007d-CORRESPONDENCE.md)")
CTRL = "a control for the RLT-007d tests; not a claim about the source"
OUT["compose/approved-results.tsv"] = (
    "# RLT-007d named results: id, the exact ordered result, what lies outside the check (GENERATED by scripts/gen-rlt007d.py).\n"
    "# `rlt compose` refuses a bundle whose id is not listed here or whose result differs.\n"
    f"admissible.h45.lemma_12_1\tdistinct.g0,distinct.g1,distinct.g2,distinct.g3,bounds,adm.all\t{NOTE}\n"
    f"admissible.h45.range_6_only\tadm.range_6\t{CTRL}\n"
    f"admissible.h45.tail_only\tadm.tail\t{CTRL}\n"
    f"admissible.h45.tail_renamed\tadm.tail\t{CTRL}\n")
ORDER = (["classify", "tail"] + [f"om.{p}_{CLS[p]}" for p in sorted(CLS)] + [f"distinct.g{k}" for k in range(4)] + ["bounds"]
         + [f"adm.range_{i}" for i in range(1, 7)] + ["adm.tail", "adm.all"])
FULL = {"id": "admissible.h45.lemma_12_1", "nodes": [node(n) for n in ORDER], "result": RESULT}
OUT["compose/h45.yaml"] = render(FULL, "Lemma 12.1 (Axiom Math, September 3, 2026 draft): H45 is an admissible 45-tuple of diameter 212, as six statements")

# Smaller bundles that must compose (controls for the negative cases).
TAILB = {"id": "admissible.h45.tail_only", "nodes": [node("tail"), node("adm.tail")], "result": ["adm.tail"]}
R6 = ["om.191_4", "om.193_3", "om.197_1", "om.199_3", "om.211_3"]
R6B = {"id": "admissible.h45.range_6_only", "nodes": [node(n) for n in R6] + [node("adm.range_6")], "result": ["adm.range_6"]}
ACC = "tests/rlt007d/accept"
OUT[f"{ACC}/tail.yaml"] = render(TAILB, "control: the tail alone")
OUT[f"{ACC}/range_6.yaml"] = render(R6B, "control: range 6 alone")
# Bound-variable renaming: the assembly's assumption is written over q; it
# is the same statement, so the bundle composes.
ren = f"{ACC}/ADM_TAIL_renamed.rlt"
OUT[ren] = tail_assembly(rename=True)
OUT[f"{ACC}/tail_renamed.yaml"] = render(
    {"id": "admissible.h45.tail_renamed", "nodes": [node("tail"), node("adm.tail", assembly={"path": ren, "sha256": sha(OUT[ren])})],
     "result": ["adm.tail"]}, "control: the tail assembly's assumption written over q instead of p")

# ---- Negative cases: (name, bundle text, class, reason substring). ----
ROWS = []


def neg(name, cls, reason, bundle=None, text=None, files=()):
    d = f"{NEG}/{name}"
    for rel, t in files:
        OUT[f"{d}/{rel}"] = t
    OUT[f"{d}/bundle.yaml"] = text if text is not None else render(bundle, f"NEGATIVE {name}: refused as {cls}")
    code = {"malformed": 2, "missing": 2, "stale": 3}.get(cls, 1)
    ROWS.append(f"{d}/bundle.yaml\t{code}\t{cls}\t{reason}")


def tb(*nodes, result=("adm.tail",), **kw):
    b = {"id": "admissible.h45.negative", "nodes": list(nodes), "result": list(result)}
    b.update(kw)
    return b


def nd(n, **over):
    return node(n, **over)


T0, T1 = nd("tail"), nd("adm.tail")
full_txt = OUT["compose/h45.yaml"]
neg("N01_unknown_key", "malformed", "unknown key `comment`", text=full_txt.replace("nodes:\n", "comment: x\nnodes:\n", 1))
neg("N02_wrong_format", "malformed", "format must be rlt-compose/1", text=full_txt.replace("format: rlt-compose/1", "format: rlt-compose/2", 1))
neg("N03_layer0_with_assembly", "malformed", "a layer-0 node names a library theorem, not an assembly",
    tb(nd("tail", assembly={"path": ASSEMBLY["adm.tail"][0], "sha256": sha(ASSEMBLY["adm.tail"][1])}), T1))
neg("N04_discharged_by_unknown_node", "missing", "is discharged by `tail_1`, which is not a node of this bundle",
    tb(T0, nd("adm.tail", discharge=[{"assumption": "tl", "by": "tail_1"}])))
neg("N05_unknown_library_id", "missing", "`admissible.h45.omits.tail_2` is not a library id",
    tb(nd("tail", library="admissible.h45.omits.tail_2"), T1))
neg("N06_assembly_missing", "missing", "tests/rlt007d/assemble/ADM_TAIL_X.rlt: file not found",
    tb(T0, nd("adm.tail", assembly={"path": f"{ASM}/ADM_TAIL_X.rlt", "sha256": "a" * 64})))
neg("N07_spec_missing", "missing", "tests/rlt007d/spec/SPEC_X.rlt: file not found",
    tb(T0, nd("adm.tail", spec={"path": f"{SPEC}/SPEC_X.rlt", "sha256": "a" * 64})))
neg("N08_stale_assembly_hash", "stale", "tests/rlt007d/assemble/ADM_TAIL.rlt has SHA-256",
    tb(T0, nd("adm.tail", assembly={"path": ASSEMBLY["adm.tail"][0], "sha256": "f" * 64})))
neg("N09_stale_spec_hash", "stale", "tests/rlt007d/spec/SPEC_adm_tail.rlt has SHA-256",
    tb(T0, nd("adm.tail", spec={"path": f"{SPEC}/SPEC_adm_tail.rlt", "sha256": "f" * 64})))
neg("N10_stale_fingerprint", "stale", "the bundle pins fingerprint " + "e" * 64,
    tb(nd("tail", fingerprint="e" * 64), T1))
neg("N11_conditional_component", "conditional", "`arith.conditional.2x3_7` is a conditional theorem",
    tb(nd("tail", library="arith.conditional.2x3_7", fingerprint=lib_fp("arith.conditional.2x3_7")), T1))
neg("N12_duplicate_node_id", "duplicated", "node id `tail` appears twice", tb(T0, T0, T1))
neg("N13_duplicate_library_theorem", "duplicated", "library theorem `admissible.h45.omits.tail_1` is named by nodes `tail` and `tail2`",
    tb(T0, nd("tail", id="tail2"), T1, result=("adm.tail", "tail2")))
neg("N14_duplicate_discharge_row", "duplicated", "assumption `tl` has two discharge rows",
    tb(T0, nd("adm.tail", discharge=[{"assumption": "tl", "by": "tail"}, {"assumption": "tl", "by": "tail"}])))
neg("N15_result_twice", "duplicated", "result names `adm.tail` twice", tb(T0, T1, result=("adm.tail", "adm.tail")))
neg("N16_same_assembly_twice", "duplicated", "assembly tests/rlt007d/assemble/ADM_TAIL.rlt is named by nodes `adm.tail` and `adm.tail2`",
    tb(T0, T1, nd("adm.tail", id="adm.tail2"), result=("adm.tail", "adm.tail2")))
neg("N17_unused_component", "unused", "node `bounds` discharges no assumption and is not part of the result",
    tb(T0, nd("bounds"), T1))
xa = tail_assembly(extra_assume=True)
neg("N18_unused_declared_assumption", "unused", "assumption `xx` is declared in tests/rlt007d/negative/N18_unused_declared_assumption/ADM_TAIL.rlt but not used",
    tb(T0, nd("adm.tail", assembly={"path": f"{NEG}/N18_unused_declared_assumption/ADM_TAIL.rlt", "sha256": sha(xa)})),
    files=[("ADM_TAIL.rlt", xa)])
neg("N19_extra_discharge_row", "unused", "the discharge row for `t2` names no assumption",
    tb(T0, nd("adm.tail", discharge=[{"assumption": "tl", "by": "tail"}, {"assumption": "t2", "by": "tail"}])))
neg("N20_self_discharge", "graph", "is discharged by `adm.tail` of layer 1, not a lower layer",
    tb(T0, nd("adm.tail", discharge=[{"assumption": "tl", "by": "adm.tail"}]), result=("adm.tail", "tail")))
neg("N21_wrong_stated_layer", "graph", "node `adm.tail` states layer 2, but its highest discharger is of layer 0",
    tb(T0, nd("adm.tail", layer=2)))
neg("N22_composed_without_discharge", "graph", "node `adm.tail` (layer 1) discharges nothing",
    tb(T0, nd("adm.tail", discharge=[]), result=("adm.tail", "tail")))
ca = tail_assembly(cite=True)
neg("N23_assembly_cites_a_file", "graph", "cites other files; an assembly takes its premises as assumptions only",
    tb(T0, nd("adm.tail", assembly={"path": f"{NEG}/N23_assembly_cites_a_file/ADM_TAIL.rlt", "sha256": sha(ca)})),
    files=[("ADM_TAIL.rlt", ca)])
cyc = f"{NEG}/N24_cycle/ADM_TAIL_x.rlt"
neg("N24_cycle", "graph", "of layer 1, not a lower layer",
    tb(T0, nd("adm.tail", discharge=[{"assumption": "tl", "by": "adm.tail_x"}]),
       nd("adm.tail", id="adm.tail_x", assembly={"path": cyc, "sha256": sha(ASSEMBLY["adm.tail"][1] + "; x\n")},
          discharge=[{"assumption": "tl", "by": "adm.tail"}]), result=("adm.tail", "adm.tail_x")),
    files=[("ADM_TAIL_x.rlt", ASSEMBLY["adm.tail"][1] + "; x\n")])
neg("N25_other_axiom_set", "incompatible", "axiom set ROBINSON-Q (4ad551922225) differs from the bundle's PEANO-ARITH",
    text=render(tb(T0, T1, axiom_set="PEANO-ARITH"), "NEGATIVE N25: refused as incompatible"))
neg("N26_other_convention", "incompatible", "convention NAT-DEF differs from the bundle's NAT-DEC",
    text=render(tb(T0, T1, convention="NAT-DEC"), "NEGATIVE N26: refused as incompatible"))
neg("N27_other_definitions_file", "incompatible", "are not the pinned spec/definitions.rlt",
    text=render(tb(T0, T1, defs_path="spec/axioms.yaml", defs_sha=sha(open("spec/axioms.yaml", encoding="utf-8").read())), "NEGATIVE N27"))
neg("N28_unregistered_axiom_set", "incompatible", "axiom set `ROBINSON-R` is not registered",
    text=render(tb(T0, T1, axiom_set="ROBINSON-R"), "NEGATIVE N28"))
br = tail_assembly(broken=True)
neg("N29_assembly_not_accepted", "kernel", "is not accepted (REJECT",
    tb(T0, nd("adm.tail", assembly={"path": f"{NEG}/N29_assembly_not_accepted/ADM_TAIL.rlt", "sha256": sha(br)})),
    files=[("ADM_TAIL.rlt", br)])
neg("N30_discharged_by_wrong_component", "mismatch", "assumption `tl` is not, canonically, the statement of `bounds`",
    tb(T0, nd("bounds"), nd("adm.tail", discharge=[{"assumption": "tl", "by": "bounds"}]), result=("adm.tail", "tail")))
r6 = [nd(n) for n in R6]
sw = [{"assumption": a, "by": b} for a, b in DISCHARGE["adm.range_6"]]
sw[0]["by"], sw[1]["by"] = sw[1]["by"], sw[0]["by"]
neg("N31_rows_swapped", "mismatch", "assumption `om191` is not, canonically, the statement of `om.193_3`",
    {"id": "admissible.h45.negative", "nodes": r6 + [nd("adm.range_6", discharge=sw)], "result": ["adm.range_6"]})
s211 = (f"(forall p (imp (lt 211 p) {adm('p')}))", f"∀p. (211 < p → {adm_w('p')})")
p211 = spec_probe("adm.tail", *s211)
neg("N32_spec_stronger_true_statement", "specification", "is not the approved specification",
    tb(T0, nd("adm.tail", spec={"path": f"{NEG}/N32_spec_stronger_true_statement/SPEC.rlt", "sha256": sha(p211)})),
    files=[("SPEC.rlt", p211)])
pt = spec_probe("tail", f"(forall p (imp (lt 212 p) {conj('p', 1, H[:-1])}))", f"∀p. (212 < p → {conj_w('p', 1, H[:-1])})")
neg("N33_component_spec_44_elements", "specification", "is not the approved specification",
    tb(nd("tail", spec={"path": f"{NEG}/N33_component_spec_44_elements/SPEC.rlt", "sha256": sha(pt)}), T1),
    files=[("SPEC.rlt", pt)])
wk = tail_assembly(bound=False)
neg("N34_weaker_true_assembly", "specification", "node `adm.tail`: its statement is not, canonically, the approved specification",
    tb(T0, nd("adm.tail", assembly={"path": f"{NEG}/N34_weaker_true_assembly/ADM_TAIL.rlt", "sha256": sha(wk)})),
    files=[("ADM_TAIL.rlt", wk)])
r6short = range_assembly(6, [191, 193, 197, 199])
neg("N35_range_6_without_211", "specification", "node `adm.range_6`: its statement is not, canonically, the approved specification",
    {"id": "admissible.h45.negative", "nodes": r6[:4] + [nd("adm.range_6", assembly={"path": f"{NEG}/N35_range_6_without_211/ADM_RANGE_6.rlt", "sha256": sha(r6short)},
                                                       discharge=[{"assumption": a, "by": b} for a, b in DISCHARGE["adm.range_6"][:4]])],
     "result": ["adm.range_6"]},
    files=[("ADM_RANGE_6.rlt", r6short)])
two = spec_probe("adm.tail", extra=("s2", "0 = 0"))
two = two.replace("(G ", "(G0 (= 0 0) (hyp s2))\n    (G ", 1)
neg("N36_spec_not_a_probe", "specification", "is not the approved specification",
    tb(T0, nd("adm.tail", spec={"path": f"{NEG}/N36_spec_not_a_probe/SPEC.rlt", "sha256": sha(two)})),
    files=[("SPEC.rlt", two)])
bad = spec_probe("adm.tail").replace('(assume s "', '(assume s "0 = 0 ∧ ', 1)
neg("N37_spec_refused_by_kernel", "specification", "is not the approved specification",
    tb(T0, nd("adm.tail", spec={"path": f"{NEG}/N37_spec_refused_by_kernel/SPEC.rlt", "sha256": sha(bad)})),
    files=[("SPEC.rlt", bad)])
neg("N38_result_unknown_node", "missing", "result names `adm.every`, which is not a node of this bundle",
    tb(T0, T1, result=("adm.tail", "adm.every")))
neg("N39_assumption_without_row", "missing", "assumption `om211` of tests/rlt007d/assemble/ADM_RANGE_6.rlt has no discharge row",
    {"id": "admissible.h45.negative", "nodes": r6[:4] + [nd("adm.range_6", discharge=[{"assumption": a, "by": b} for a, b in DISCHARGE["adm.range_6"][:4]])],
     "result": ["adm.range_6"]})
# The full bundle with adm.all dropped from the result: the five kernel
# results alone are true, but they are not the approved six-part result.
neg("N40_full_bundle_without_adm_all", "unused", "node `adm.all` discharges no assumption and is not part of the result",
    text=full_txt.replace("  - adm.all\n", "", 1))
assert OUT[f"{NEG}/N40_full_bundle_without_adm_all/bundle.yaml"] != full_txt
wp = spec_probe("adm.tail", f"(forall p (imp (lt 212 p) {adm('p', False)}))", f"∀p. (212 < p → {adm_w('p', False)})")
neg("N41_weaker_theorem_with_its_own_probe", "specification", "is not the approved specification tests/rlt007d/spec/SPEC_adm_tail.rlt",
    tb(T0, nd("adm.tail", assembly={"path": f"{NEG}/N41_weaker_theorem_with_its_own_probe/ADM_TAIL.rlt", "sha256": sha(wk)},
              spec={"path": f"{NEG}/N41_weaker_theorem_with_its_own_probe/SPEC.rlt", "sha256": sha(wp)})),
    files=[("ADM_TAIL.rlt", wk), ("SPEC.rlt", wp)])
neg("N42_unregistered_node", "specification", "node `adm.tail_extra` has no approved specification",
    tb(T0, nd("adm.tail", id="adm.tail_extra"), result=("adm.tail_extra",)))
neg("N43_named_result_with_fewer_statements", "specification",
    "named result `admissible.h45.lemma_12_1`: the result lists adm.tail, but compose/approved-results.tsv registers distinct.g0",
    text=OUT[f"{ACC}/tail.yaml"].replace("id: admissible.h45.tail_only", "id: admissible.h45.lemma_12_1", 1))
wa = all_assembly(weak=True)
wap = f"{NEG}/N44_weaker_adm_all/ADM_ALL.rlt"
neg("N44_weaker_adm_all", "specification", "node `adm.all`: its statement is not, canonically, the approved specification",
    text=full_txt.replace(f"      path: {ASSEMBLY['adm.all'][0]}\n      sha256: {sha(ASSEMBLY['adm.all'][1])}",
                          f"      path: {wap}\n      sha256: {sha(wa)}", 1),
    files=[("ADM_ALL.rlt", wa)])
assert OUT[f"{NEG}/N44_weaker_adm_all/bundle.yaml"] != full_txt
# Bundle syntax the two readers must refuse alike (found by the implementation audit).
tail_txt = OUT[f"{ACC}/tail.yaml"]
neg("N45_yaml_anchor", "malformed", "YAML anchors are refused", text=tail_txt.replace("definitions:\n", "definitions: &D\n", 1))
neg("N46_yaml_explicit_tag", "malformed", "YAML tags are refused", text=tail_txt.replace("    layer: 1\n", "    layer: !!int 1\n", 1))
neg("N47_yaml_repeated_key", "malformed", "duplicated key", text=tail_txt.replace("  - id: adm.tail\n", "  - id: adm.tail\n    id: adm.tail\n", 1))
neg("N48_dot_path", "malformed", "is not a repository-relative path",
    text=tail_txt.replace(f"      path: {ASM}/ADM_TAIL.rlt", f"      path: ./{ASM}/ADM_TAIL.rlt", 1))
neg("N49_id_not_a_string", "malformed", "`id` must be a string", text=tail_txt.replace("id: admissible.h45.tail_only", "id: 7", 1))
cp = f"{NEG}/N50_registered_probe_copied/SPEC.rlt"
neg("N50_registered_probe_copied", "specification", "is not the approved specification",
    tb(T0, nd("adm.tail", spec={"path": cp, "sha256": sha(OUT[f"{SPEC}/SPEC_adm_tail.rlt"])})),
    files=[("SPEC.rlt", OUT[f"{SPEC}/SPEC_adm_tail.rlt"])])

# For the tamper test in scripts/verify.sh: the classification alone. As it
# stands it is refused as an unregistered named result (the last check);
# with a file it cites altered, it is refused as stale first.
OUT["tests/rlt007d/tamper/classify_only.yaml"] = render(
    {"id": "admissible.h45.negative", "nodes": [node("classify", layer=0)], "result": ["classify"]},
    "TAMPER: the classification alone (scripts/verify.sh alters a file it cites)")
OUT["tests/rlt007d/expected.tsv"] = ("# bundle\texit\trefusal class\trequired substring of the reason (RLT-007d negative cases)\n"
                                    + "\n".join(ROWS) + "\n")

# ---- Lean comparison. ----
def lean_w(w):
    """RLT's written form in Lean syntax: `∀p. ` as `∀ p, `, `×` as `*`."""
    return re.sub(r"([∀∃])([a-z])\. ", r"\1 \2, ", w).replace(" × ", " * ")


def lname(n):
    return "C_" + n.replace(".", "_")


L0 = [n for n in ORDER if LAYER[n] == 0]
defs = "\n\n".join(f"/-- `{n}`: {('library ' + LIB[n]) if n in LIB else 'assembly ' + ASSEMBLY[n][0]}, as RLT writes it. -/\n"
                   f"def {lname(n)} : Prop :=\n  {lean_w(STMT[n][1])}" for n in ORDER)
om_args = lambda qs: " ".join(f"(h{q} : {lname(f'om.{q}_{CLS[q]}')})" for q in qs)


def range_proof(i, qs):
    pats = " | ".join("rfl" for _ in qs)
    arms = "\n".join(f"  · exact ⟨{CLS[q]}, by decide, h{q}⟩" for q in qs)
    return (f"/-- Layer 1, as `{ASSEMBLY[f'adm.range_{i}'][0]}`: the omissions for range {i} give Adm for its primes. -/\n"
            f"theorem range_{i} {om_args(qs)} : {lname(f'adm.range_{i}')} := by\n"
            f"  intro p hp\n  rcases hp with {pats}\n{arms}")



def bounds_term():
    t = None
    for h in H:
        pair = f"⟨⟨{h}, rfl⟩, ⟨{212 - h}, rfl⟩⟩"
        t = pair if t is None else f"⟨{t}, {pair}⟩"
    return t


def bounds_pat():
    t = None
    for i, h in enumerate(H):
        pair = f"⟨_, ⟨z{i}, e{i}⟩⟩"
        t = pair if t is None else f"⟨{t}, {pair}⟩"
    return t


ranges_lean = "\n\n".join(range_proof(i, qs) for i, qs in enumerate(RANGES, 1))
LEAN = f"""/-
RLT-007d, Lean comparison of the H45 composition: GENERATED by scripts/gen-rlt007d.py.
Core Lean only. Run: cd lean && lean RLT007D.lean

Not an RLT derivation; independent evidence outside RLT's trusted base.

Each `C_…` below is one node's statement exactly as RLT writes it (its
specification probe's goal, with `∀p. ` read as `∀ p, ` and `×` as `*`;
scripts/verify.sh checks this symbol for symbol), read over ℕ with Lean's own
`<` and `∣`. The composition is then carried out in Lean along the same graph:
`range_1` … `range_6` and `adm_tail` (layer 1) from the layer-0 statements,
`adm_all` (layer 2) from the classification and layer 1. `lemma_12_1`
restates the result in the standard vocabulary (`IsPrime`: 2 ≤ p and only
1 and p divide p; residues `h % p`), and
`lemma_12_1_holds` discharges every layer-0 hypothesis inside Lean, so the
conclusion holds in Lean outright.
-/

namespace RLT007D

-- The statements are long (up to 990 conjuncts): generous elaboration limits.
set_option maxHeartbeats 40000000
set_option maxRecDepth 200000
set_option synthInstance.maxSize 200000
set_option synthInstance.maxHeartbeats 4000000

def H45 : List Nat := [{", ".join(map(str, H))}]

{defs}

{ranges_lean}

/-- Layer 1, as `{ASSEMBLY['adm.tail'][0]}`: r = 1 for every p > 212. -/
theorem adm_tail (ht : {lname('tail')}) : {lname('adm.tail')} := by
  intro p hp
  exact ⟨1, by omega, ht p hp⟩

/-- Layer 2, as `{ASSEMBLY['adm.all'][0]}`: the classification and layer 1. -/
theorem adm_all (hc : {lname('classify')}) (r1 : {lname('adm.range_1')}) (r2 : {lname('adm.range_2')})
    (r3 : {lname('adm.range_3')}) (r4 : {lname('adm.range_4')}) (r5 : {lname('adm.range_5')})
    (r6 : {lname('adm.range_6')}) (ta : {lname('adm.tail')}) : {lname('adm.all')} := by
  intro p hp
  rcases hc p hp with h | h | h | h | h | h | h
  · exact r1 p h
  · exact r2 p h
  · exact r3 p h
  · exact r4 p h
  · exact r5 p h
  · exact r6 p h
  · exact ta p h

/-- RLT's `<` (NAT-DEF: ∃z. S(z) + x = y) is Lean's `<` on ℕ. -/
theorem rlt_lt_iff (x y : Nat) : (∃ z, (z + 1) + x = y) ↔ x < y :=
  ⟨fun ⟨_, h⟩ => by omega, fun h => ⟨y - x - 1, by omega⟩⟩

/-- RLT's `∣` (NAT-DEF: ∃k. n = d × k) is Lean's `∣` on ℕ, by definition. -/
theorem rlt_dvd_iff (d n : Nat) : (∃ k, n = d * k) ↔ d ∣ n := Iff.rfl

/-- The left-nested omission, read as a statement about the list. -/
theorem om_iff (p r : Nat) :
    ({lean_w(conj_w('p', 'r'))}) ↔ ∀ h ∈ H45, ¬ ∃ k, h = p * k + r := by
  simp only [H45, List.mem_cons, List.not_mem_nil, forall_eq_or_imp, or_false, forall_eq, and_assoc]

/-- The standard notion of prime, in core Lean. -/
def IsPrime (p : Nat) : Prop := 2 ≤ p ∧ ∀ m, m ∣ p → m = 1 ∨ m = p

theorem prime_iff (p : Nat) : (1 < p ∧ ∀ d, ((1 < d ∧ d < p) → ¬(d ∣ p))) ↔ IsPrime p := by
  unfold IsPrime
  constructor
  · rintro ⟨h1, h2⟩
    refine ⟨h1, fun m hm => ?_⟩
    have hp : 0 < p := by omega
    have ml : m ≤ p := Nat.le_of_dvd hp hm
    rcases Nat.lt_or_ge 1 m with h | h
    · rcases Nat.lt_or_ge m p with h' | h'
      · exact absurd hm (h2 m ⟨h, h'⟩)
      · right; omega
    · left
      rcases Nat.eq_zero_or_pos m with e | e
      · subst e; rcases hm with ⟨k, hk⟩; simp at hk; omega
      · omega
  · rintro ⟨h1, h2⟩
    refine ⟨h1, fun d ⟨hd1, hd2⟩ hdp => ?_⟩
    rcases h2 d hdp with e | e <;> omega

/-- The four distinctness statements say exactly that H45 has no repeated entry. -/
theorem distinct_iff :
    ({lname('distinct.g0')} ∧ {lname('distinct.g1')} ∧ {lname('distinct.g2')} ∧ {lname('distinct.g3')}) ↔ H45.Nodup := by
  unfold {lname('distinct.g0')} {lname('distinct.g1')} {lname('distinct.g2')} {lname('distinct.g3')}
  decide

/-- The bounds statement gives h ≤ 212 for every listed h. -/
theorem bounds_le (b : {lname('bounds')}) : ∀ h ∈ H45, h ≤ 212 := by
  obtain {bounds_pat()} := b
  intro h hh
  simp only [H45, List.mem_cons, List.not_mem_nil, or_false] at hh
  rcases hh with {" | ".join("rfl" for _ in H)} <;> omega

/-- Lemma 12.1 in the standard vocabulary, from the six statements: H45 lists
    45 pairwise distinct numbers from 0 to 212, 0 and 212 among them, and for
    every prime p some residue class modulo p contains none of them. -/
theorem lemma_12_1 (g0 : {lname('distinct.g0')}) (g1 : {lname('distinct.g1')}) (g2 : {lname('distinct.g2')})
    (g3 : {lname('distinct.g3')}) (b : {lname('bounds')}) (a : {lname('adm.all')}) :
    H45.Nodup ∧ H45.length = 45 ∧ (∀ h ∈ H45, h ≤ 212) ∧ 0 ∈ H45 ∧ 212 ∈ H45 ∧
    ∀ p, IsPrime p → ∃ r, r < p ∧ ∀ h ∈ H45, h % p ≠ r := by
  refine ⟨distinct_iff.mp ⟨g0, g1, g2, g3⟩, by decide, bounds_le b, by decide, by decide, fun p hp => ?_⟩
  obtain ⟨r, hr, hom⟩ := a p ((prime_iff p).mpr hp)
  refine ⟨r, hr, fun h hh heq => (om_iff p r).mp hom h hh ⟨h / p, ?_⟩⟩
  have := Nat.div_add_mod h p
  rw [heq] at this
  omega

/-- Every layer-0 statement, decided in Lean, independently of RLT. -/
theorem components :
    {" ∧ ".join(lname(n) for n in L0)} := by
  have om : ∀ p r, r < p → (∀ h ∈ H45, h % p ≠ r) → ({lean_w(conj_w('p', 'r'))}) := by
    intro p r hr h
    refine (om_iff p r).mpr fun x hx ⟨k, hk⟩ => h x hx ?_
    rw [hk, Nat.mul_add_mod, Nat.mod_eq_of_lt hr]
  refine ⟨?_, ?_, {", ".join(f"om {q} {CLS[q]} (by decide) (by decide)" for q in sorted(CLS))}, {", ".join(f"by unfold {lname(f'distinct.g{k}')}; decide" for k in range(4))}, {bounds_term()}⟩
  · intro p hp
    rcases Nat.lt_or_ge p 213 with h | h
    · have key : ∀ q, q < 213 → (1 < q ∧ ∀ d, d < q → 1 < d → q % d ≠ 0) →
          {lean_w(disj_w('q'))} := by decide
      exact key p h ⟨hp.1, fun d hd1 hd2 hm => hp.2 d ⟨hd2, hd1⟩ (Nat.dvd_of_mod_eq_zero hm)⟩
    · iterate 6 apply Or.inr
      omega
  · intro p hp
    refine (om_iff p 1).mpr fun h hh ⟨k, hk⟩ => ?_
    have hb := (show ∀ h ∈ H45, h ≤ 212 ∧ h ≠ 1 by decide) h hh
    rcases k with _ | k
    · simp at hk; omega
    · have : p ≤ p * (k + 1) := Nat.le_mul_of_pos_right p (by omega)
      omega

/-- The composition in Lean: every layer-0 statement, through layers 1 and 2,
    to Lemma 12.1 in the standard vocabulary. -/
theorem lemma_12_1_holds :
    H45.Nodup ∧ H45.length = 45 ∧ (∀ h ∈ H45, h ≤ 212) ∧ 0 ∈ H45 ∧ 212 ∈ H45 ∧
    ∀ p, IsPrime p → ∃ r, r < p ∧ ∀ h ∈ H45, h % p ≠ r := by
  obtain ⟨hc, ht, {", ".join(f"h{q}" for q in sorted(CLS))}, g0, g1, g2, g3, b⟩ := components
  exact lemma_12_1 g0 g1 g2 g3 b
    (adm_all hc {" ".join(f"(range_{i} {' '.join(f'h{q}' for q in qs)})" for i, qs in enumerate(RANGES, 1))} (adm_tail ht))

end RLT007D
"""
LEAN += "\n" + "".join(f"#print axioms RLT007D.{t}\n" for t in re.findall(r"^theorem (\w+)", LEAN, flags=re.M))
# `set_option … in` goes before a declaration's doc comment.
for _ in range(3):
    LEAN = re.sub(r"(/--(?:(?!-/).)*-/\n)((?:set_option [^\n]* in\n)+)", r"\2\1", LEAN, flags=re.S)
OUT["lean/RLT007D.lean"] = LEAN

# ---- Write, or compare (--check). ----
DIRS = [ASM, SPEC, ACC, NEG, "tests/rlt007d/tamper"]
if MODE == "--check":
    diff = [p for p, t in sorted(OUT.items()) if not os.path.isfile(p) or open(p, encoding="utf-8", newline="").read() != t]
    for d in DIRS:
        for dp, _, fs in os.walk(d):
            for f in fs:
                if os.path.join(dp, f) not in OUT:
                    diff.append(f"{os.path.join(dp, f)} (not produced by the generator)")
    for f in sorted(os.listdir("tests/rlt007d")):
        if f not in ("assemble", "spec", "accept", "negative", "tamper", "expected.tsv"):
            diff.append(f"tests/rlt007d/{f} (not produced by the generator)")
    if sorted(os.listdir("compose")) != ["approved-results.tsv", "approved-specifications.tsv", "h45.yaml"]:
        diff.append("compose/ (an extra or missing bundle)")
    for d in diff[:20]:
        print(f"DIFFERS  {d}")
    print(f"scripts/gen-rlt007d.py --check: {len(OUT)} outputs, {len(diff)} differ")
    sys.exit(0 if not diff else 1)

import shutil
for d in DIRS + ["compose"]:
    shutil.rmtree(d, ignore_errors=True)
for p, t in OUT.items():
    os.makedirs(os.path.dirname(p), exist_ok=True)
    with open(p, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(t)
print(f"{len(OUT)} outputs written: 8 assemblies, {len(STMT)} specification probes, the bundle, 3 control bundles, "
      f"{len(ROWS)} negative cases, Lean comparison")

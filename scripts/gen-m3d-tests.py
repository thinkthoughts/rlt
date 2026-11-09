#!/usr/bin/env python3
"""Write the RLT-003 M3d test proofs into tests/m3d/ (untrusted generator).

usage: scripts/gen-m3d-tests.py RLT_BINARY OUT_DIR

Accepted proofs go to OUT_DIR/accept, rejected ones to OUT_DIR/reject, and
OUT_DIR/expected.tsv lists every rejected file with its required exit code
and a substring its verdict must contain.

The enumeration proofs follow the approved M3d review (section 4). Fix the
universally quantified variable as a parameter `a`; open the hypothesis
a < N (unfolded: exists z. S(z) + a = N) at a fresh `w`. Layer i carries
L_i : S(w) + a_i = N - i and P_i : a = S^i(a_i). Q3 splits a_i: in the zero
case one `subst` gives a = i, which is injected into the left-nested
disjunction; in the successor case a_i = S(a_{i+1}), one add_succ and one Q1
peel give L_{i+1}, and one `subst` gives P_{i+1}. The last layer refutes
S(w) + a_N = 0 by Q2 in both cases. The generator is outside the trusted
base: the kernel checks every step. verify.sh regenerates this directory
and requires it to match byte for byte. Cases that need a scratch
repository, the kernel API or a report inspection run in cli/tests/m3d.rs.
"""
import hashlib
import os
import sys

rlt, out = sys.argv[1], sys.argv[2]
ACC = "tests/m3d/accept"
REJ = "tests/m3d/reject"


def S(t, k):
    for _ in range(k):
        t = f"(S {t})"
    return t


def disj(values, var="a"):
    """Left-nested disjunction of var = v for v in values (proof syntax)."""
    e = f"(= {var} {values[0]})"
    for v in values[1:]:
        e = f"(or {e} (= {var} {v}))"
    return e


def prefix(values, j, var="a"):
    return disj(values[: j + 1], var)


def written(values, var="x"):
    return " ∨ ".join(f"{var} = {v}" for v in values)


def enumeration(n, values, fault=None):
    """Steps proving, for parameter a, (exists z. S(z) + a = n) -> C, where C
    is the left-nested disjunction of a = v (v in values) or false when
    values is empty. Returns (steps, label of the implication, C)."""
    concl = disj(values) if values else "false"
    params = ["a"] + [f"a{i}" for i in range(1, n + 2)]
    if fault == "reuse":
        params[4] = params[3]  # DR04: layer 3 opens its successor case at a3 again
    st = []
    if fault == "lt-reversed":
        st += [f"(H (exists z (= (+ a (S z)) {n})) (hyp h))",
               f"(L0 (= (+ a (S w)) {n}) (hyp h0))"]
    else:
        st += [f"(H (exists z (= (+ (S z) a) {n})) (hyp h))",
               f"(L0 (= (+ (S w) a) {n}) (hyp h0))"]
    st.append("(P0 (= a a) (refl a))")

    def layer(i, L, P):
        ai, an = params[i], params[i + 1]
        p = f"Y{i}"
        if i == n:
            # The closing layer: S(w) + a_n = 0, refuted by Q2 in both cases.
            if fault == "unsplit-last":
                return [f"({p}_q2 (not (= 0 (+ (S w) {ai}))) (inst q2_zero_ne_succ (x (+ w {ai}))))",
                        f"({p}_ls (= 0 (+ (S w) {ai})) (symm {L}))",
                        f"({p}_f false (imp-elim {p}_q2 {p}_ls))"] + (
                    [f"({p}_fe {concl} (false-elim {p}_f))"] if values else []), (
                    f"{p}_fe" if values else f"{p}_f")
            s = [f"({p}_q3 (or (= {ai} 0) (exists y (= {ai} (S y)))) (inst q3_zero_or_succ (x {ai})))",
                 f"({p}_r (= (S w) (S w)) (refl (S w)))",
                 f"({p}_ls (= 0 (+ (S w) {ai})) (symm {L}))",
                 # zero case
                 f"({p}_zh (= {ai} 0) (hyp z{i}))",
                 f"({p}_zc (= (+ (S w) {ai}) (+ (S w) 0)) (cong + {p}_r {p}_zh))",
                 f"({p}_za (= (+ (S w) 0) (S w)) (inst add_zero (x (S w))))",
                 f"({p}_zt (= (+ (S w) {ai}) (S w)) (trans {p}_zc {p}_za))",
                 f"({p}_zt2 (= 0 (S w)) (trans {p}_ls {p}_zt))",
                 f"({p}_zq (not (= 0 (S w))) (inst q2_zero_ne_succ (x w)))",
                 f"({p}_zf false (imp-elim {p}_zq {p}_zt2))",
                 # successor case
                 f"({p}_sh (exists y (= {ai} (S y))) (hyp s{i}))",
                 f"({p}_wh (= {ai} (S {an})) (hyp w{i}))",
                 f"({p}_sc (= (+ (S w) {ai}) (+ (S w) (S {an}))) (cong + {p}_r {p}_wh))",
                 f"({p}_sa (= (+ (S w) (S {an})) (S (+ (S w) {an}))) (inst add_succ (x (S w)) (y {an})))",
                 f"({p}_st (= (+ (S w) {ai}) (S (+ (S w) {an}))) (trans {p}_sc {p}_sa))",
                 f"({p}_st2 (= 0 (S (+ (S w) {an}))) (trans {p}_ls {p}_st))",
                 f"({p}_sq (not (= 0 (S (+ (S w) {an})))) (inst q2_zero_ne_succ (x (+ (S w) {an}))))",
                 f"({p}_sf false (imp-elim {p}_sq {p}_st2))",
                 f"({p}_se false (exists-elim {p}_sh {an} w{i} {p}_sf))",
                 f"({p}_or false (or-cases {p}_q3 z{i} {p}_zf s{i} {p}_se))"]
            if values:
                s.append(f"({p}_fe {concl} (false-elim {p}_or))")
                return s, f"{p}_fe"
            return s, f"{p}_or"
        s = [f"({p}_q3 (or (= {ai} 0) (exists y (= {ai} (S y)))) (inst q3_zero_or_succ (x {ai})))"]
        # Zero case: a = i, injected into the disjunction.
        motive = f"(at u (= a {S('u', i)}))"
        s += [f"({p}_zh (= {ai} 0) (hyp z{i}))",
              f"({p}_zs (= a {i}) (subst {p}_zh {motive} {P}))"]
        cur = f"{p}_zs"
        if fault == "false-neq":
            # DR16: each branch must refute a = 15 from a = i.
            s += neq_branch(i, cur, p)
            zero_end = f"{p}_zn"
        elif i in values:
            j = values.index(i)
            if j > 0:
                if fault == "right-nested" and i == 5:
                    s.append(f"({p}_j{j} (or (= a {i}) {prefix(values, j - 1)}) (or-right {cur}))")
                else:
                    s.append(f"({p}_j{j} {prefix(values, j)} (or-right {cur}))")
                cur = f"{p}_j{j}"
            for k in range(j + 1, len(values)):
                s.append(f"({p}_j{k} {prefix(values, k)} (or-left {cur}))")
                cur = f"{p}_j{k}"
            zero_end = cur
        else:
            # The value i is not listed: the injection has nowhere to go.
            s.append(f"({p}_jx (or {concl} (= a {i})) (or-right {cur}))")
            zero_end = f"{p}_jx"
        # Successor case.
        nb = n - i
        peel_y = nb - 2 if (fault == "skip-peel" and i == 2) else nb - 1
        mot2 = f"(at u (= a {S('u', i + 1 if (fault == 'bad-motive' and i == 2) else i)}))"
        s += [f"({p}_sh (exists y (= {ai} (S y))) (hyp s{i}))",
              f"({p}_wh (= {ai} (S {an})) (hyp w{i}))",
              f"({p}_r (= (S w) (S w)) (refl (S w)))",
              f"({p}_sc (= (+ (S w) {ai}) (+ (S w) (S {an}))) (cong + {p}_r {p}_wh))",
              f"({p}_sa (= (+ (S w) (S {an})) (S (+ (S w) {an}))) (inst add_succ (x (S w)) (y {an})))",
              f"({p}_st (= (+ (S w) {ai}) (S (+ (S w) {an}))) (trans {p}_sc {p}_sa))",
              f"({p}_sy (= (S (+ (S w) {an})) (+ (S w) {ai})) (symm {p}_st))",
              f"({p}_sb (= (S (+ (S w) {an})) {nb}) (trans {p}_sy {L}))",
              f"({p}_q1 (imp (= (S (+ (S w) {an})) (S {peel_y})) (= (+ (S w) {an}) {peel_y})) (inst q1_succ_inj (x (+ (S w) {an})) (y {peel_y})))",
              f"({p}_L (= (+ (S w) {an}) {nb - 1}) (imp-elim {p}_q1 {p}_sb))",
              f"({p}_P (= a {S(an, i + 1)}) (subst {p}_wh {mot2} {P}))"]
        inner, inner_end = layer(i + 1, f"{p}_L", f"{p}_P")
        s += inner
        s += [f"({p}_se {concl} (exists-elim {p}_sh {an} w{i} {inner_end}))",
              f"({p}_or {concl} (or-cases {p}_q3 z{i} {zero_end} s{i} {p}_se))"]
        return s, f"{p}_or"

    def neq_branch(i, eq, p):
        """From eq : a = i, prove not (a = 15) (DR16). At i = 15 the closing
        Q2 step is false and the kernel refuses it."""
        t = 15
        s = [f"({p}_k (= a {t}) (hyp k{i}))",
             f"({p}_n1 (= {i} a) (symm {eq}))",
             f"({p}_n2 (= {i} {t}) (trans {p}_n1 {p}_k))"]
        cur = f"{p}_n2"
        lo, hi = min(i, t), max(i, t)
        for j in range(lo, 0, -1):
            a_, b_ = (i - lo + j - 1, t - lo + j - 1)
            s += [f"({p}_i{j} (imp (= (S {a_}) (S {b_})) (= {a_} {b_})) (inst q1_succ_inj (x {a_}) (y {b_})))",
                  f"({p}_e{j} (= {a_} {b_}) (imp-elim {p}_i{j} {cur}))"]
            cur = f"{p}_e{j}"
        d = hi - lo
        if i > t:
            s.append(f"({p}_ns (= 0 {d}) (symm {cur}))")
            cur = f"{p}_ns"
        s += [f"({p}_nq (not (= 0 {d})) (inst q2_zero_ne_succ (x {max(d - 1, 0)})))",
              f"({p}_nf false (imp-elim {p}_nq {cur}))",
              f"({p}_zn (not (= a {t})) (imp-intro k{i} {p}_nf))"]
        return s

    if fault == "false-neq":
        concl = "(not (= a 15))"
    body, end = layer(0, "L0", "P0")
    st += body
    if fault == "w-in-conclusion":
        st = st[:2] + [f"(EH (= (+ (S w) a) {n}) (exists-elim H w h0 L0))",
                       f"(I (imp (exists z (= (+ (S z) a) {n})) (= (+ (S w) a) {n})) (imp-intro h EH))"]
        return st, "I", concl
    hyp_f = f"(exists z (= (+ a (S z)) {n}))" if fault == "lt-reversed" else f"(exists z (= (+ (S z) a) {n}))"
    st += [f"(EH {concl} (exists-elim H w h0 {end}))",
           f"(I (imp {hyp_f} {concl}) (imp-intro h EH))"]
    return st, "I", concl


def forall_goal(n, values):
    concl = disj(values, "x") if values else "false"
    return f"(forall x (imp (lt x {n}) {concl}))"


def header(pid, goal, params="", cites=None, extra="", convention="NAT-DEF"):
    c = ""
    if cites:
        c = "  (cites\n" + "\n".join(f"    ({l} \"{p}\" \"{h}\")" for l, p, h in cites) + ")\n"
    pr = f"  (params {params})\n" if params else ""
    return (f"; RLT-003 M3d test {pid}. Generated by scripts/gen-m3d-tests.py.\n"
            f"(rlt-proof\n  (id M3d.{pid})\n  (convention {convention})\n  (goal \"{goal}\")\n"
            f"  (mode unconditional)\n{extra}  (axioms ROBINSON-Q)\n{c}{pr}")


def proof(pid, goal, steps, **kw):
    body = "\n".join("    " + s for s in steps)
    return header(pid, goal, **kw) + f"  (steps\n{body}))\n"


def enum_file(pid, n, values, goal=None, fault=None):
    st, last, concl = enumeration(n, values, fault)
    if fault != "w-in-conclusion":
        st.append(f"(G {forall_goal(n, values) if fault != 'false-neq' else f'(forall x (imp (lt x {n}) (not (= x 15))))'} (forall-intro a {last}))")
    if goal is None:
        goal = f"∀x. x < {n} → {written(values)}" if values else f"∀x. ¬(x < {n})"
    params = " ".join(dict.fromkeys(["a", "w"] + [f"a{i}" for i in range(1, n + 2)]))
    return proof(pid, goal, st, params=params)


def sha(text):
    return hashlib.sha256(text.encode()).hexdigest()


accept, reject, expected = {}, {}, []


def rej(name, code, reason, text):
    reject[name] = text
    expected.append((name, code, reason))


V30 = list(range(30))
accept["DA01_below_thirty"] = enum_file("DA01", 30, V30)
accept["DA02_below_three"] = enum_file("DA02", 3, [0, 1, 2])
accept["DA03_below_one"] = enum_file("DA03", 1, [0])
accept["DA04_nothing_below_zero"] = enum_file("DA04", 0, [])
d1 = (f"{ACC}/DA01_below_thirty.rlt", sha(accept["DA01_below_thirty"]))
seven = f"(imp (lt 7 30) {disj(V30, '7')})"
DA05_STEPS = [f"(D {forall_goal(30, V30)} (cite D1))",
              f"(I {seven} (forall-elim D 7))"]
DA05_GOAL = f"7 < 30 → {written(V30, '7')}"
accept["DA05_seven_from_the_enumeration"] = proof("DA05", DA05_GOAL, DA05_STEPS, cites=[("D1",) + d1])

# ── Rejected (DR01 to DR16); reasons pinned after checking ─────────────────
rej("DR01_value_29_missing", 1, "or-cases: cases conclude different formulas",
    enum_file("DR01", 30, list(range(29))))
rej("DR02_bound_31", 1, "or-cases: cases conclude different formulas",
    enum_file("DR02", 31, V30, goal=f"∀x. x < 31 → {written(V30)}"))
rej("DR03_last_layer_unsplit", 1, "rule `inst` yields",
    enum_file("DR03", 30, V30, fault="unsplit-last"))
rej("DR04_eigenvariable_reused", 1, "exists-elim: parameter `a3` occurs in the eliminated formula",
    enum_file("DR04", 30, V30, fault="reuse"))
rej("DR05_witness_in_conclusion", 1, "exists-elim: parameter `w` occurs in the conclusion",
    enum_file("DR05", 30, V30, fault="w-in-conclusion"))
rej("DR06_skipped_peel", 1, "imp-elim: antecedent does not match",
    enum_file("DR06", 30, V30, fault="skip-peel"))
rej("DR07_wrong_subst_motive", 1, "subst: motive does not match premise",
    enum_file("DR07", 30, V30, fault="bad-motive"))
rej("DR08_right_nested_injection", 1, "rule `or-right` yields",
    enum_file("DR08", 30, V30, fault="right-nested"))
V_dup = list(range(8)) + [7] + list(range(9, 30))
rej("DR09_seven_twice_eight_missing", 1, "or-cases: cases conclude different formulas",
    enum_file("DR09", 30, V_dup, goal=f"∀x. x < 30 → {written(V_dup)}"))
rej("DR10_lt_unfolded_backwards", 1, "trans requires rhs(first) ≡ lhs(second); here `S(w) + a` ≢ `a + S(w)`",
    enum_file("DR10", 30, V30, fault="lt-reversed"))
rej("DR11_unregistered_induction_axiom", 1, "axiom `induction` is unregistered",
    proof("DR11", f"∀x. x < 1 → x = 0", ["(t (forall x (imp (lt x 1) (= x 0))) (inst induction (x 0)))"]))
rej("DR12_declared_hypothesis_unconditional", 1, "unconditional goal requested, but the result depends on assumptions",
    proof("DR12", "∀x. x < 1 → x = 0", ["(t (forall x (imp (lt x 1) (= x 0))) (hyp h))"],
          extra="  (assume h \"∀x. x < 1 → x = 0\")\n"))
rej("DR13_parameter_named_lt_quoted", 2, "parameter `lt` collides with a definition name",
    proof("DR13", "∀x. x < 1 → x = 0", ["(t (forall x (imp (lt x 1) (= x 0))) (hyp h))"],
          params="lt", convention="\"NAT-DEF\""))
rej("DR14_cited_hash_edited", 2, "the citation records",
    proof("DR14", DA05_GOAL, DA05_STEPS, cites=[("D1", d1[0], sha(accept["DA02_below_three"]))]))
d2 = (f"{ACC}/DA02_below_three.rlt", sha(accept["DA02_below_three"]))
rej("DR15_cites_the_bound_three_theorem", 1, "the cited theorem `D1` proves",
    proof("DR15", DA05_GOAL, DA05_STEPS, cites=[("D1",) + d2]))
rej("DR16_false_claim_not_fifteen", 1, "step claims `¬(0 = 0)`, rule `inst` yields `¬(0 = 1)`",
    enum_file("DR16", 30, V30, goal="∀x. x < 30 → ¬(x = 15)", fault="false-neq"))

for sub, files in (("accept", accept), ("reject", reject)):
    os.makedirs(os.path.join(out, sub), exist_ok=True)
    for name, text in files.items():
        with open(os.path.join(out, sub, name + ".rlt"), "w") as f:
            f.write(text)
with open(os.path.join(out, "expected.tsv"), "w") as f:
    f.write("# file\texit\trequired substring of the verdict (RLT-003 M3d rejected cases)\n")
    for name, code, reason in expected:
        f.write(f"{REJ}/{name}.rlt\t{code}\t{reason}\n")

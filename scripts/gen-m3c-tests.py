#!/usr/bin/env python3
"""Write the RLT-003 M3c test proofs into tests/m3c/ (untrusted generator).

usage: scripts/gen-m3c-tests.py RLT_BINARY OUT_DIR

Accepted proofs go to OUT_DIR/accept, rejected and malformed ones to
OUT_DIR/reject, and OUT_DIR/expected.tsv lists every rejected file with its
required exit code and a substring its verdict must contain.

The non-divisibility proofs follow the scheme of the approved M3c review
(section 3 and Appendix A): the multiplication witness is eliminated as an
eigenvariable p0; at layer i, Q3 splits p_i into 0 or S(p_{i+1}); the zero
branch closes by Q2; the successor branch rewrites d x S(p_{i+1}) to
S^d(d x p_{i+1}) (lemma M_d: mul_succ, d add_succ, add_zero), peels
successors with Q1, and either reaches the next layer's fact or, at the last
layer, closes by Q2 at the arbitrary remaining variable. The generator is
outside the trusted base: the kernel checks every step. verify.sh
regenerates this directory and requires it to match byte for byte. Cases
that need a scratch repository or the kernel API run in cli/tests/m3c.rs.
"""
import hashlib
import os
import re
import subprocess
import sys

rlt, out = sys.argv[1], sys.argv[2]
ACC = "tests/m3c/accept"
REJ = "tests/m3c/reject"


def S(t, k):
    """The term S^k(t) in proof syntax."""
    for _ in range(k):
        t = f"(S {t})"
    return t


def mul(d, t):
    return f"(* {d} {t})"


def gen_steps(equation, prefix):
    """Steps of an `rlt gen` proof, relabelled with a prefix; returns (steps, last label)."""
    text = subprocess.run([rlt, "gen", equation, "X"], check=True,
                          capture_output=True, text=True).stdout
    body = text[text.index("(steps") + len("(steps"):].rstrip()
    assert body.endswith("))"), "unexpected rlt gen output"
    body = body[:-2]
    lines = [l.strip() for l in body.strip().splitlines() if l.strip()]
    lines = [re.sub(r"\bs(\d+)\b", lambda m: f"{prefix}{m.group(1)}", l) for l in lines]
    last = re.match(r"\((\S+)", lines[-1]).group(1)
    return lines, last


def lemma_m(d, p, pre, drop_add_succ=False):
    """Lemma M_d at parameter p: d x S(p) = S^d(d x p). Returns (steps, label)."""
    x = mul(d, p)
    st = [f"({pre}_m0 (= {mul(d, S(p, 1))} (+ {x} {d})) (inst mul_succ (x {d}) (y {p})))",
          f"({pre}_e0 (= (+ {x} 0) {x}) (inst add_zero (x {x})))"]
    prev = f"{pre}_e0"
    for j in range(1, d + 1):
        if drop_add_succ and j == d:
            # CR09: the last add_succ step is missing; the chain is closed by
            # a trans whose endpoints do not meet.
            st.append(f"({pre}_e{j} (= (+ {x} {j}) {S(x, j)}) (trans {pre}_m0 {prev}))")
            prev = f"{pre}_e{j}"
            continue
        st += [f"({pre}_a{j} (= (+ {x} {j}) (S (+ {x} {j - 1}))) (inst add_succ (x {x}) (y {j - 1})))",
               f"({pre}_c{j} (= (S (+ {x} {j - 1})) {S(x, j)}) (cong S {prev}))",
               f"({pre}_e{j} (= (+ {x} {j}) {S(x, j)}) (trans {pre}_a{j} {pre}_c{j}))"]
        prev = f"{pre}_e{j}"
    st.append(f"({pre}_md (= {mul(d, S(p, 1))} {S(x, d)}) (trans {pre}_m0 {prev}))")
    return st, f"{pre}_md"


def peel(a, b, k, eq, pre, short=False):
    """From eq : S^k(a) = S^k(b), k uses of q1 give a = b. a is a numeral.
    Returns (steps, label). With short=True (CR06) the last q1 is omitted
    but the step still claims a = b."""
    st, cur = [], eq
    for j in range(k, 0, -1):
        lhs, rhs = a + j - 1, S(b, j - 1)
        if short and j == 1:
            st.append(f"({pre}_pl{j} (= {lhs} {rhs}) (imp-elim {pre}_inj{j + 1} {cur}))")
            cur = f"{pre}_pl{j}"
            break
        st += [f"({pre}_inj{j} (imp (= (S {lhs}) (S {rhs})) (= {lhs} {rhs})) (inst q1_succ_inj (x {lhs}) (y {rhs})))",
               f"({pre}_pl{j} (= {lhs} {rhs}) (imp-elim {pre}_inj{j} {cur}))"]
        cur = f"{pre}_pl{j}"
    return st, cur


def layers(d, n, params, fault=None):
    """The case tree under the fact n = d x params[0] (step label `F0`).
    Returns (steps, label of the step proving false). fault selects one
    planted error for the rejected cases."""
    steps = []

    def layer(i, ni, fact):
        p, q = params[i], params[i + 1]
        L = f"L{i}"
        split_at = params[i - 1] if (fault == "stale-split" and i == 2) else p
        st = [f"({L}_q3 (or (= {p} 0) (exists y (= {p} (S y)))) (inst q3_zero_or_succ (x {split_at})))"]
        # Zero branch: n_i = d x 0 = 0, refuted by Q2 at n_i - 1.
        q2x = 5 if (fault == "wrong-q2" and i == 0) else max(ni - 1, 0)
        zero_claim = f"(not (= 0 {q2x + 1}))" if fault == "wrong-q2" and i == 0 else f"(not (= 0 {ni}))"
        zero = [f"({L}_z_h (= {p} 0) (hyp z{i}))",
               f"({L}_z_r (= {d} {d}) (refl {d}))",
               f"({L}_z_c (= {mul(d, p)} {mul(d, 0)}) (cong * {L}_z_r {L}_z_h))",
               f"({L}_z_m (= {mul(d, 0)} 0) (inst mul_zero (x {d})))",
               f"({L}_z_t1 (= {mul(d, p)} 0) (trans {L}_z_c {L}_z_m))",
               f"({L}_z_t2 (= {ni} 0) (trans {fact} {L}_z_t1))",
               f"({L}_z_s (= 0 {ni}) (symm {L}_z_t2))",
               f"({L}_z_q2 {zero_claim} (inst q2_zero_ne_succ (x {q2x})))",
               f"({L}_z_f false (imp-elim {L}_z_q2 {L}_z_s))"]
        if not (fault == "no-zero" and i == 2):
            st += zero
        # Successor branch: p_i = S(p_{i+1}).
        ms, ml = lemma_m(d, q, f"{L}_s", drop_add_succ=(fault == "short-m" and i == 0))
        st += [f"({L}_s_h (exists y (= {p} (S y))) (hyp s{i}))",
               f"({L}_s_w (= {p} (S {q})) (hyp w{i}))",
               f"({L}_s_r (= {d} {d}) (refl {d}))",
               f"({L}_s_c (= {mul(d, p)} {mul(d, S(q, 1))}) (cong * {L}_s_r {L}_s_w))"] + ms + [
               f"({L}_s_t1 (= {mul(d, p)} {S(mul(d, q), d)}) (trans {L}_s_c {ml}))",
               f"({L}_s_t2 (= {ni} {S(mul(d, q), d)}) (trans {fact} {L}_s_t1))"]
        if ni >= d:
            # Inner layer: n_i is literally S^d(n_{i+1}); peel d successors.
            ps, pl = peel(ni - d, mul(d, q), d, f"{L}_s_t2", f"{L}_s",
                          short=(fault == "short-peel" and i == 0))
            st += ps
            inner_steps, inner_false = layer(i + 1, ni - d, pl)
            st += inner_steps
            branch_false = inner_false
        else:
            # Last layer: n_i = r < d; peel r successors to 0 = S^(d-r)(d x p_L).
            r = ni
            ps, pl = peel(0, S(mul(d, q), d - r), r, f"{L}_s_t2", f"{L}_s")
            st += ps
            last_eq = pl if r > 0 else f"{L}_s_t2"
            if fault == "extra-peel":
                st += [f"({L}_s_qx (imp (= (S 0) {S(mul(d, q), d - r)}) (= 0 {S(mul(d, q), d - r - 1)})) (inst q1_succ_inj (x 0) (y {S(mul(d, q), d - r - 1)})))",
                       f"({L}_s_px (= 0 {S(mul(d, q), d - r - 1)}) (imp-elim {L}_s_qx {last_eq}))"]
            st += [f"({L}_s_q2 (not (= 0 {S(mul(d, q), d - r)})) (inst q2_zero_ne_succ (x {S(mul(d, q), d - r - 1)})))",
                   f"({L}_s_f false (imp-elim {L}_s_q2 {last_eq}))"]
            branch_false = f"{L}_s_f"
        st.append(f"({L}_s_e false (exists-elim {L}_s_h {q} w{i} {branch_false}))")
        if fault == "no-zero" and i == 2:
            st.append(f"({L}_or false (or-cases {L}_q3 s{i} {L}_s_e))")
        else:
            st.append(f"({L}_or false (or-cases {L}_q3 z{i} {L}_z_f s{i} {L}_s_e))")
        return st, f"{L}_or"

    st, f = layer(0, n, "F0")
    return st, f


def n_layers(d, n):
    return n // d + 1


def params_for(d, n, reuse=False):
    ps = [f"p{i}" for i in range(n_layers(d, n) + 1)]
    if reuse:
        ps[2] = ps[1]  # CR04: layer 1 opens its successor case at p1 again
    return ps


def header(pid, goal, convention="NAT-DEF", axioms="ROBINSON-Q", params="",
           cites=None, mode="unconditional", extra=""):
    c = ""
    if cites:
        c = "  (cites\n" + "\n".join(f"    ({l} \"{p}\" \"{h}\")" for l, p, h in cites) + ")\n"
    pr = f"  (params {params})\n" if params else ""
    return (f"; RLT-003 M3c test {pid}. Generated by scripts/gen-m3c-tests.py.\n"
            f"(rlt-proof\n  (id M3c.{pid})\n  (convention {convention})\n  (goal \"{goal}\")\n"
            f"  (mode {mode})\n{extra}  (axioms {axioms})\n{c}{pr}")


def proof(pid, goal, steps, **kw):
    body = "\n".join("    " + s for s in steps)
    return header(pid, goal, **kw) + f"  (steps\n{body}))\n"


def not_dvd(pid, d, n, fault=None, reuse=False):
    ps = params_for(d, n, reuse)
    st, f = layers(d, n, ps, fault)
    steps = [f"(H (exists k (= {n} (* {d} k))) (hyp h))",
             f"(F0 (= {n} (* {d} {ps[0]})) (hyp h0))"] + st + [
             f"(E false (exists-elim H {ps[0]} h0 {f}))",
             f"(G (not (dvd {d} {n})) (imp-intro h E))"]
    return proof(pid, f"¬({d} ∣ {n})", steps, params=" ".join(dict.fromkeys(ps)))


def witnesswise(pid, d, n):
    ps = params_for(d, n)
    st, f = layers(d, n, ps)
    steps = [f"(F0 (= {n} (* {d} {ps[0]})) (hyp h0))"] + st + [
             f"(I (not (= {n} (* {d} {ps[0]}))) (imp-intro h0 {f}))",
             f"(G (forall k (not (= {n} (* {d} k)))) (forall-intro {ps[0]} I))"]
    return proof(pid, f"∀k. ¬({n} = {d} × k)", steps, params=" ".join(ps))


def sha(text):
    return hashlib.sha256(text.encode()).hexdigest()


accept, reject, expected = {}, {}, []


def rej(name, code, reason, text):
    reject[name] = text
    expected.append((name, code, reason))


# ── Accepted (CA01 to CA09) ────────────────────────────────────────────────
st, last = gen_steps("8 = 2 × 4", "m")
accept["CA01_two_divides_eight"] = proof("CA01", "2 ∣ 8", st + [f"(e1 (dvd 2 8) (exists-intro 4 {last}))"])
accept["CA02_two_not_divides_seven"] = not_dvd("CA02", 2, 7)
accept["CA03_three_not_divides_seven"] = not_dvd("CA03", 3, 7)
accept["CA04_five_not_divides_seven"] = not_dvd("CA04", 5, 7)
cite = lambda name: (f"{ACC}/{name}.rlt", sha(accept[name]))
c2, c3, c4 = cite("CA02_two_not_divides_seven"), cite("CA03_three_not_divides_seven"), cite("CA04_five_not_divides_seven")
FREE = ["(t2 (not (dvd 2 7)) (cite T2))",
        "(t3 (not (dvd 3 7)) (cite T3))",
        "(t4 (not (dvd 5 7)) (cite T4))",
        "(a1 (and (not (dvd 2 7)) (not (dvd 3 7))) (and-intro t2 t3))",
        "(a2 (free235 7) (and-intro a1 t4))"]
CITES = [("T2",) + c2, ("T3",) + c3, ("T4",) + c4]
accept["CA05_free235_seven"] = proof("CA05", "free235(7)", FREE, cites=CITES)
accept["CA06_two_witnesswise"] = witnesswise("CA06", 2, 7)
accept["CA07_three_witnesswise"] = witnesswise("CA07", 3, 7)
accept["CA08_five_witnesswise"] = witnesswise("CA08", 5, 7)
# CA09: the witness-wise form gives the ¬-form (the M3a bridge), by citation.
c6 = cite("CA06_two_witnesswise")
accept["CA09_witnesswise_gives_not_dvd"] = proof("CA09", "¬(2 ∣ 7)", [
    "(W (forall k (not (= 7 (* 2 k)))) (cite T6))",
    "(H (exists k (= 7 (* 2 k))) (hyp h))",
    "(A (= 7 (* 2 a)) (hyp w))",
    "(B (not (= 7 (* 2 a))) (forall-elim W a))",
    "(C false (imp-elim B A))",
    "(D false (exists-elim H a w C))",
    "(G (not (dvd 2 7)) (imp-intro h D))"], cites=[("T6",) + c6], params="a")

# ── Rejected derivations (CR01 to CR13, CR15, CR16) ────────────────────────
rej("CR01_false_witness", 1, "exists-intro: witness does not match premise",
    proof("CR01", "2 ∣ 7", ["(m (= (* 2 3) (* 2 3)) (refl (* 2 3)))",
                            "(e (dvd 2 7) (exists-intro 3 m))"]))
rej("CR02_zero_branch_omitted", 1, "or-cases takes 5 argument(s), given 3",
    not_dvd("CR02", 2, 7, fault="no-zero"))
rej("CR03_stale_split_parameter", 1, "rule `inst` yields",
    not_dvd("CR03", 2, 7, fault="stale-split"))
rej("CR04_eigenvariable_reused", 1, "exists-elim: parameter `p1` occurs in the eliminated formula",
    not_dvd("CR04", 2, 7, reuse=True))
rej("CR05_eigenvariable_in_conclusion", 1, "exists-elim: parameter `p0` occurs in the conclusion",
    proof("CR05", "¬(2 ∣ 7)", ["(H (exists k (= 7 (* 2 k))) (hyp h))",
                               "(F0 (= 7 (* 2 p0)) (hyp h0))",
                               "(E (= 7 (* 2 p0)) (exists-elim H p0 h0 F0))",
                               "(G (imp (exists k (= 7 (* 2 k))) (= 7 (* 2 p0))) (imp-intro h E))"], params="p0"))
rej("CR06_one_peel_too_few", 1, "imp-elim: antecedent does not match",
    not_dvd("CR06", 2, 7, fault="short-peel"))
rej("CR07_one_peel_too_many", 1, "imp-elim",
    not_dvd("CR07", 2, 7, fault="extra-peel"))
rej("CR08_wrong_q2_instance", 1, "imp-elim",
    not_dvd("CR08", 2, 7, fault="wrong-q2"))
rej("CR09_short_multiplication_lemma", 1, "trans",
    not_dvd("CR09", 2, 7, fault="short-m"))
rej("CR10_two_not_divides_eight", 1, "step claims `¬(0 = 0)`, rule `inst` yields `¬(0 = 1)`",
    not_dvd("CR10", 2, 8))
rej("CR11_hypothesis_left_open", 1, "unconditional goal requested, but the result depends on assumptions",
    proof("CR11", "¬(2 ∣ 7)", ["(t (not (dvd 2 7)) (hyp h))"],
          extra="  (assume h \"¬(2 ∣ 7)\")\n"))
rej("CR12_free235_wrong_order", 1, "rule `and-intro` yields",
    proof("CR12", "free235(7)", FREE[:3] + [
        "(a1 (and (not (dvd 2 7)) (not (dvd 3 7))) (and-intro t3 t2))",
        "(a2 (free235 7) (and-intro a1 t4))"], cites=CITES))
rej("CR13_unregistered_induction_axiom", 1, "axiom `induction` is unregistered",
    proof("CR13", "¬(2 ∣ 7)", ["(t (not (dvd 2 7)) (inst induction (x 0)))"]))
rej("CR15a_parameter_named_dvd", 2, "parameter `dvd` collides with a definition name",
    proof("CR15a", "¬(2 ∣ 7)", ["(t (not (dvd 2 7)) (hyp h))"], params="dvd"))
rej("CR15b_parameter_named_dvd_quoted", 2, "parameter `dvd` collides with a definition name",
    proof("CR15b", "¬(2 ∣ 7)", ["(t (not (dvd 2 7)) (hyp h))"], params="dvd",
          convention="\"NAT-DEF\""))
rej("CR16_binder_named_free235_quoted", 2, "shadowed name `free235`",
    proof("CR16", "∀k. ¬(7 = 2 × k)", ["(t (forall free235 (not (= 7 (* 2 free235)))) (hyp h))"],
          convention="\"NAT-DEF\""))

# ── Rejected citations (CR17 to CR24) ──────────────────────────────────────
rej("CR17a_cited_hash_differs", 2, "the citation records",
    proof("CR17a", "free235(7)", FREE, cites=[("T2", c2[0], c3[1]), ("T3",) + c3, ("T4",) + c4]))
c9 = cite("CA09_witnesswise_gives_not_dvd")
rej("CR17b_same_goal_other_file", 2, "the citation records",
    proof("CR17b", "free235(7)", FREE, cites=[("T2", c9[0], c2[1]), ("T3",) + c3, ("T4",) + c4]))
cr01 = (f"{REJ}/CR01_false_witness.rlt", sha(reject["CR01_false_witness"]))
rej("CR18_cites_rejected_file", 2, "is not an accepted theorem (REJECT",
    proof("CR18", "free235(7)", FREE, cites=[("T2",) + cr01, ("T3",) + c3, ("T4",) + c4]))


def repo_file_cite(path):
    with open(path, "rb") as f:
        return (path, hashlib.sha256(f.read()).hexdigest())


pa = repo_file_cite("tests/m3a/accept/A02_forall_mul_zero.rlt")
rej("CR19_cites_other_axiom_set", 2, "differs from this file's ROBINSON-Q",
    proof("CR19", "¬(2 ∣ 7)", ["(t (not (dvd 2 7)) (cite T))"], cites=[("T",) + pa]))
cond = repo_file_cite("tests/m3b/accept/BA11_conditional_assumption_unfolds.rlt")
rej("CR20_cites_conditional_theorem", 2, "the cited theorem is conditional",
    proof("CR20", "¬(2 ∣ 7)", ["(t (not (dvd 2 7)) (cite T))"], cites=[("T",) + cond]))
DUMMY = "0" * 64
rej("CR21a_self_citation", 2, "citation cycle",
    proof("CR21a", "¬(2 ∣ 7)", ["(t (not (dvd 2 7)) (cite T))"],
          cites=[("T", f"{REJ}/CR21a_self_citation.rlt", DUMMY)]))
cyc_c = proof("CR21c", "¬(2 ∣ 7)", ["(t (not (dvd 2 7)) (cite T))"],
              cites=[("T", f"{REJ}/CR21b_two_file_cycle.rlt", DUMMY)])
rej("CR21c_two_file_cycle_partner", 2, "the citation records", cyc_c)
rej("CR21b_two_file_cycle", 2, "citation cycle",
    proof("CR21b", "¬(2 ∣ 7)", ["(t (not (dvd 2 7)) (cite T))"],
          cites=[("T", f"{REJ}/CR21c_two_file_cycle_partner.rlt", sha(cyc_c))]))
cyc_f = proof("CR21f", "¬(2 ∣ 7)", ["(t (not (dvd 2 7)) (cite T))"],
              cites=[("T", f"{REJ}/CR21d_three_file_cycle.rlt", DUMMY)])
cyc_e = proof("CR21e", "¬(2 ∣ 7)", ["(t (not (dvd 2 7)) (cite T))"],
              cites=[("T", f"{REJ}/CR21f_three_file_cycle_end.rlt", sha(cyc_f))])
rej("CR21f_three_file_cycle_end", 2, "the citation records", cyc_f)
rej("CR21e_three_file_cycle_middle", 2, "the citation records", cyc_e)
rej("CR21d_three_file_cycle", 2, "citation cycle",
    proof("CR21d", "¬(2 ∣ 7)", ["(t (not (dvd 2 7)) (cite T))"],
          cites=[("T", f"{REJ}/CR21e_three_file_cycle_middle.rlt", sha(cyc_e))]))
rej("CR22_stated_formula_differs", 1, "rule `cite` yields",
    proof("CR22", "¬(2 ∣ 9)", ["(t (not (dvd 2 9)) (cite T2))"], cites=[("T2",) + c2]))
rej("CR23a_parent_path", 2, "no `..`",
    proof("CR23a", "¬(2 ∣ 7)", ["(t (not (dvd 2 7)) (cite T))"],
          cites=[("T", f"tests/m3c/../m3c/accept/CA02_two_not_divides_seven.rlt", c2[1])]))
rej("CR23b_absolute_path", 2, "leading `/`",
    proof("CR23b", "¬(2 ∣ 7)", ["(t (not (dvd 2 7)) (cite T))"],
          cites=[("T", "/etc/CA02.rlt", c2[1])]))
rej("CR24_cited_label_swapped", 1, "rule `cite` yields",
    proof("CR24", "free235(7)", ["(t2 (not (dvd 2 7)) (cite T3))"] + FREE[1:], cites=CITES))

for sub, files in (("accept", accept), ("reject", reject)):
    os.makedirs(os.path.join(out, sub), exist_ok=True)
    for name, text in files.items():
        with open(os.path.join(out, sub, name + ".rlt"), "w") as f:
            f.write(text)
with open(os.path.join(out, "expected.tsv"), "w") as f:
    f.write("# file\texit\trequired substring of the verdict (RLT-003 M3c rejected cases)\n")
    for name, code, reason in expected:
        f.write(f"{REJ}/{name}.rlt\t{code}\t{reason}\n")

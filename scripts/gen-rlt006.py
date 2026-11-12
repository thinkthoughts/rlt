#!/usr/bin/env python3
"""RLT-006 generator (untrusted): residue certificates for H45, the admissible
45-tuple of Axiom Math's draft "A New Bound for Small Gaps Between Primes"
(September 3, 2026), Lemma 12.1. Deterministic: the same inputs give the same
bytes.

Inputs: sources/transcriptions/AXIOM-BGP212-2026-09-03.yaml (the reviewed
transcription: the tuple, the 14 primes and their omitted classes) and the
M3c non-divisibility scheme in scripts/gen-m3c-tests.py.

Usage: gen-rlt006.py RLT_BINARY [--materialize-only]

Outputs (the kernel checks every proof; nothing here is trusted):
  tests/rlt006/gen/O_PP_HHH.rlt   630 element certificates  ¬(∃k. h = (p × k) + r)
                                  GENERATED, not committed (.gitignore); pinned
                                  by the manifest and by the summaries' citations
  tests/rlt006/manifest.sha256    SHA-256 of the 630 files (sha256sum -c format)
  tests/rlt006/accept/OM_PP.rlt   14 per-prime theorems, each citing its 45
                                  certificates: the left-nested conjunction
  tests/rlt006/reject/*.rlt       rejected cases; tests/rlt006/expected.tsv
  certificates/h45/residues.tsv   computed residues h = p·q + s (evidence, not
  certificates/h45/profile.tsv    kernel-checked): occupied and omitted classes
  statements/H45-OMITS-PP.yaml    14 statement records (basis docs/RLT-006-CORRESPONDENCE.md)
  lean/RLT006Direct.lean          Lean comparison 1: the 14 statements, literally,
                                  and Lemma 12.1 in Lean from them by the paper's route

--materialize-only writes only tests/rlt006/gen/ (used before checking).
"""
import hashlib
import os
import re
import subprocess
import sys

import yaml

rlt = sys.argv[1]
ONLY_GEN = "--materialize-only" in sys.argv[2:]
root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
os.chdir(root)

T = yaml.safe_load(open("sources/transcriptions/AXIOM-BGP212-2026-09-03.yaml", encoding="utf-8"))
items = {it["id"]: it for it in T["items"]}
H = items["h45-tuple"]["numbers"]
P = items["omitted-primes"]["numbers"]
R = items["omitted-classes"]["numbers"]
assert len(H) == 45 and len(set(H)) == 45 and len(P) == len(R) == 14

src = open("scripts/gen-m3c-tests.py", encoding="utf-8").read()
ns = {"rlt": rlt, "re": re, "subprocess": subprocess}
exec(src[src.index("def S(t, k):"):src.index("def sha(text):")], ns)
S, layers, peel, n_layers = ns["S"], ns["layers"], ns["peel"], ns["n_layers"]

GEN, ACC, REJ = "tests/rlt006/gen", "tests/rlt006/accept", "tests/rlt006/reject"


def claim(p, h, r):
    return f"(not (exists k (= {h} (+ (* {p} k) {r}))))"


def written(p, h, r):
    return f"¬(∃k. {h} = ({p} × k) + {r})"


def certificate_steps(p, h, r):
    """Steps refuting h = p·k + r under Robinson's Q, for numerals p, h, r.

    From h = p·k + r, add_succ (r times) gives h = S^r(p·k).
    h ≥ r: peel r successors (Q1) to get h − r = p·k, then refute that with
           the M3c case tree on k (Q3, mul_succ, Q1 peeling, Q2).
    h < r: peel h successors to get 0 = S^(r−h)(p·k), refuted by Q2.
    The construction does not check that h ≢ r (mod p): for a false claim
    the kernel refuses the result (tests/rlt006/reject)."""
    x = "(* {} k0)".format(p)
    st = [f"(H (exists k (= {h} (+ (* {p} k) {r}))) (hyp hx))",
          f"(F (= {h} (+ {x} {r})) (hyp h0))",
          f"(e0 (= (+ {x} 0) {x}) (inst add_zero (x {x})))"]
    prev = "e0"
    for j in range(1, r + 1):
        st += [f"(a{j} (= (+ {x} {j}) (S (+ {x} {j - 1}))) (inst add_succ (x {x}) (y {j - 1})))",
               f"(c{j} (= (S (+ {x} {j - 1})) {S(x, j)}) (cong S {prev}))",
               f"(e{j} (= (+ {x} {j}) {S(x, j)}) (trans a{j} c{j}))"]
        prev = f"e{j}"
    st.append(f"(T (= {h} {S(x, r)}) (trans F {prev}))")
    params = ["k0"]
    if h >= r:
        ps, pl = peel(h - r, x, r, "T", "P") if r else ([], "T")
        st += ps
        n = h - r
        st += [f"(R (= {x} {x}) (refl {x}))", f"(F0 (= {n} {x}) (trans {pl} R))"]
        lp = ["k0"] + [f"q{i}" for i in range(1, n_layers(p, n) + 1)]
        ls, f = layers(p, n, lp)
        st += ls
        params += lp[1:]
    else:
        ps, pl = peel(0, S(x, r - h), h, "T", "P")
        st += ps
        st += [f"(q2 (not (= 0 {S(x, r - h)})) (inst q2_zero_ne_succ (x {S(x, r - h - 1)})))",
               f"(f false (imp-elim q2 {pl if h else 'T'}))"]
        f = "f"
    st += [f"(E false (exists-elim H k0 h0 {f}))", f"(G {claim(p, h, r)} (imp-intro hx E))"]
    return st, params


def proof(pid, comment, goal, steps, params=(), cites=()):
    c = ""
    if cites:
        c = "  (cites\n" + "\n".join(f'    ({l} "{pa}" "{hs}")' for l, pa, hs in cites) + ")\n"
    pr = f"  (params {' '.join(dict.fromkeys(params))})\n" if params else ""
    body = "\n".join("    " + s for s in steps)
    return (f"; {comment}\n(rlt-proof\n  (id RLT006.{pid})\n  (convention NAT-DEF)\n  (goal \"{goal}\")\n"
            f"  (mode unconditional)\n  (axioms ROBINSON-Q)\n{c}{pr}  (steps\n{body}))\n")


def element(p, h, r, note=None):
    st, ps = certificate_steps(p, h, r)
    comment = note or (f"RLT-006 element certificate: H45 omits the class {r} mod {p} at h = {h}. "
                       "Generated by scripts/gen-rlt006.py (not committed); pinned by tests/rlt006/manifest.sha256")
    return proof(f"O_{p:02d}_{h:03d}", comment, written(p, h, r), st, ps)


def gen_path(p, h):
    return f"{GEN}/O_{p:02d}_{h:03d}.rlt"


def sha(text):
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def write(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(text)


# 1. The 630 element certificates.
os.makedirs(GEN, exist_ok=True)
for old in os.listdir(GEN):
    os.remove(os.path.join(GEN, old))
certs = {}
for p, r in zip(P, R):
    for h in H:
        t = element(p, h, r)
        write(gen_path(p, h), t)
        certs[(p, h)] = sha(t)
print(f"{len(certs)} element certificates written to {GEN}/")
if ONLY_GEN:
    sys.exit(0)

with open("tests/rlt006/manifest.sha256", "w", encoding="utf-8", newline="\n") as fh:
    for (p, h), s in sorted(certs.items()):
        fh.write(f"{s}  {gen_path(p, h)}\n")


# 2. The 14 per-prime theorems.
def summary(p, r, cite_paths=None, claims_h=None):
    cite_paths = cite_paths or {h: (gen_path(p, h), certs[(p, h)]) for h in H}
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


os.makedirs(ACC, exist_ok=True)
for p, r in zip(P, R):
    cites, st, goal = summary(p, r)
    write(f"{ACC}/OM_{p:02d}.rlt", proof(
        f"OM_{p:02d}",
        f"RLT-006 per-prime theorem: the reductions of H45 mod {p} omit the class {r} (Lemma 12.1, the source's table). "
        f"Cites the 45 element certificates in {GEN}/ (generated; pinned by SHA-256 here and in tests/rlt006/manifest.sha256)",
        goal, st, cites=cites))

# 3. Rejected cases.
os.makedirs(REJ, exist_ok=True)
for old in os.listdir(REJ):
    os.remove(os.path.join(REJ, old))
rows = []


def rej(name, text, code, reason):
    write(f"{REJ}/{name}.rlt", text)
    rows.append(f"{REJ}/{name}.rlt\t{code}\t{reason}")


rej("RX01_altered_member_46_mod_43",
    element(43, 46, 3, "REJECT CASE: an altered tuple member, 46 = 43 + 3, lies in the class 3 mod 43; the certificate construction for it must fail"),
    1, "REJECT at step L1_z_q2 — step claims `¬(0 = 0)`, rule `inst` yields `¬(0 = 1)`")
rej("RX02_altered_class_0_mod_3_at_12",
    element(3, 12, 0, "REJECT CASE: an altered residue class, 0 mod 3, is occupied by 12 = 3 × 4; the certificate construction for it must fail"),
    1, "REJECT at step L4_z_q2 — step claims `¬(0 = 0)`, rule `inst` yields `¬(0 = 1)`")
cites, st, goal = summary(43, 3, claims_h=H[:-1] + [213])
rej("RX03_summary_altered_member_213",
    proof("RX03", "REJECT CASE: OM_43 with its last member altered to 213 in the goal and the steps; the cited certificate is for 212", goal, st, cites=cites),
    1, "REJECT at step f44 — step claims `¬(∃k. 213 = (43 × k) + 3)`, the cited theorem `E44` proves `¬(∃k. 212 = (43 × k) + 3)`")
alt = {h: (gen_path(43, h), certs[(43, h)]) for h in H}
alt[212] = (gen_path(41, 212), certs[(41, 212)])
cites, st, goal = summary(43, 3, cite_paths=alt)
rej("RX04_summary_cites_other_prime",
    proof("RX04", "REJECT CASE: OM_43 citing the certificate for 212 mod 41 in place of 212 mod 43", goal, st, cites=cites),
    1, "REJECT at step f44 — step claims `¬(∃k. 212 = (43 × k) + 3)`, the cited theorem `E44` proves `¬(∃k. 212 = (41 × k) + 4)`")
alt = {h: (gen_path(43, h), certs[(43, h)]) for h in H}
alt[0] = (gen_path(43, 0), "f" * 64)
cites, st, goal = summary(43, 3, cite_paths=alt)
rej("RX05_summary_stale_certificate_hash",
    proof("RX05", "REJECT CASE: OM_43 recording a stale SHA-256 for a certificate", goal, st, cites=cites),
    2, f"MALFORMED — citation `E0`: {gen_path(43, 0)} has SHA-256")
alt = {h: (gen_path(43, h), certs[(43, h)]) for h in H}
alt[0] = (f"{GEN}/O_43_999.rlt", certs[(43, 0)])
cites, st, goal = summary(43, 3, cite_paths=alt)
rej("RX06_summary_missing_certificate",
    proof("RX06", "REJECT CASE: OM_43 citing a generated certificate that does not exist", goal, st, cites=cites),
    2, f"MALFORMED — citation `E0`: cited path `{GEN}/O_43_999.rlt`: entity not found")
write("tests/rlt006/expected.tsv",
      "# file\texit\trequired substring of the verdict (RLT-006 rejected cases)\n" + "\n".join(rows) + "\n")

# 4. The computed residue table and profile (evidence, not kernel-checked).
os.makedirs("certificates/h45", exist_ok=True)
lines = ["# H45 residues, computed by scripts/gen-rlt006.py (integer division; evidence, not kernel-checked).",
         "# p\tr_p\th\tq\ts\tomits_r_p   (h = p*q + s, 0 <= s < p; omits_r_p = yes when s != r_p)"]
prof = ["# H45 residue profile mod each listed prime, computed (evidence, not kernel-checked).",
        "# p\tr_p\toccupied_count\toccupied_classes\tomitted_classes"]
for p, r in zip(P, R):
    occ = sorted({h % p for h in H})
    om = [c for c in range(p) if c not in occ]
    assert r in om, (p, r)
    for h in H:
        lines.append(f"{p}\t{r}\t{h}\t{h // p}\t{h % p}\t{'yes' if h % p != r else 'NO'}")
    prof.append(f"{p}\t{r}\t{len(occ)}\t{','.join(map(str, occ))}\t{','.join(map(str, om))}")
write("certificates/h45/residues.tsv", "\n".join(lines) + "\n")
write("certificates/h45/profile.tsv", "\n".join(prof) + "\n")
print(f"14 per-prime theorems in {ACC}/, {len(rows)} rejected cases, manifest and residue tables written")


# 5. Lean comparison 1 (generated so that its statements are the transcription's).
def lean_lit(p, r):
    t = f"¬(∃ k, {H[0]} = {p} * k + {r})"
    for h in H[1:]:
        t = f"({t} ∧ ¬(∃ k, {h} = {p} * k + {r}))"
    return t


LEAN_HEAD = """/-
RLT-006, Lean comparison 1 of 2: GENERATED by scripts/gen-rlt006.py from the
reviewed transcription sources/transcriptions/AXIOM-BGP212-2026-09-03.yaml.
Core Lean only. Run: cd lean && lean RLT006Direct.lean

Source: Axiom Math, "A New Bound for Small Gaps Between Primes", Preliminary
Draft: September 3, 2026 (PDF SHA-256 2b307ae2...3b3d80), section 2 (admissible)
and section 12 (H45, Lemma 12.1, PDF page 43).

What this file proves, in Lean (none of it is an RLT derivation):
  * om_P: each of RLT's 14 per-prime statements, symbol for symbol (RLT's
    `×` read as `*` on Nat; the kernel's checked goal is the written goal,
    since the statements use no `<` or `∣`).
  * omit_iff: ¬(∃k, h = p*k + r) ↔ h % p ≠ r for r < p (the residue reading).
  * lemma_12_1_of_rlt_statements: from the 14 statements, the paper's two
    remaining steps give Lemma 12.1's admissibility for every prime:
      - primes_below_47: the primes below 47 are exactly 2, 3, ..., 43;
      - large_moduli: for every p ≥ 46 some class mod p is omitted, because
        45 residues cannot cover p classes (missing_below, by induction on n).
    Those two steps are proved here in Lean, not in RLT.
-/

namespace RLT006Direct

/-- The tuple, as transcribed (section 12). -/
def H45 : List Nat := HLIST

/-- The listed primes and their omitted classes (proof of Lemma 12.1). -/
def table : List (Nat × Nat) := TLIST

/-- Section 2's definition, for a list of naturals: for every prime p some class
r < p is omitted. Primes as in lean-bgp212's BGP212.Basic. -/
def IsPrime (p : Nat) : Prop := 2 ≤ p ∧ ∀ d : Nat, d ∣ p → d = 1 ∨ d = p
def Admissible (H : List Nat) : Prop := ∀ p, IsPrime p → ∃ r, r < p ∧ ∀ h ∈ H, h % p ≠ r

/-- Left-nested conjunction, as RLT writes the per-prime statements. -/
def conjFrom : Prop → List Prop → Prop
  | acc, [] => acc
  | acc, b :: t => conjFrom (acc ∧ b) t
def conjL : List Prop → Prop
  | [] => True
  | a :: t => conjFrom a t

theorem conjFrom_intro : ∀ (t : List Prop) (acc : Prop), acc → (∀ b ∈ t, b) → conjFrom acc t
  | [], _, ha, _ => ha
  | b :: t, _, ha, hb =>
    conjFrom_intro t _ ⟨ha, hb b (List.mem_cons_self ..)⟩ (fun x hx => hb x (List.mem_cons_of_mem _ hx))

theorem conjFrom_elim : ∀ (t : List Prop) (acc : Prop), conjFrom acc t → acc ∧ ∀ b ∈ t, b
  | [], _, h => ⟨h, fun _ hb => absurd hb (List.not_mem_nil)⟩
  | b :: t, acc, h => by
    obtain ⟨⟨ha, hb⟩, ht⟩ := conjFrom_elim t (acc ∧ b) h
    refine ⟨ha, fun x hx => ?_⟩
    rcases List.mem_cons.mp hx with rfl | hx
    · exact hb
    · exact ht x hx

theorem conjL_intro (t : List Prop) (h : ∀ b ∈ t, b) : conjL t := by
  cases t with
  | nil => trivial
  | cons a t => exact conjFrom_intro t a (h a (List.mem_cons_self ..)) (fun x hx => h x (List.mem_cons_of_mem _ hx))

theorem conjL_elim (t : List Prop) (h : conjL t) : ∀ b ∈ t, b := by
  cases t with
  | nil => exact fun _ hb => absurd hb List.not_mem_nil
  | cons a t =>
    obtain ⟨ha, ht⟩ := conjFrom_elim t a h
    intro x hx
    rcases List.mem_cons.mp hx with rfl | hx
    · exact ha
    · exact ht x hx

/-- RLT's element statement is the residue statement. -/
theorem omit_iff (h p r : Nat) (hr : r < p) : (¬ ∃ k, h = p * k + r) ↔ h % p ≠ r := by
  constructor
  · intro hn heq
    exact hn ⟨h / p, by have := Nat.div_add_mod h p; omega⟩
  · intro hne ⟨k, hk⟩
    apply hne
    rw [hk, Nat.mul_add_mod, Nat.mod_eq_of_lt hr]

/-- RLT's per-prime statement for (p, r), as a list conjunction. -/
def OM (p r : Nat) : Prop := conjL (H45.map fun h => ¬ ∃ k, h = p * k + r)

theorem om_of_residues (p r : Nat) (hr : r < p) (hd : ∀ h ∈ H45, h % p ≠ r) : OM p r :=
  conjL_intro _ (by
    intro b hb
    obtain ⟨h, hh, rfl⟩ := List.mem_map.mp hb
    exact (omit_iff h p r hr).mpr (hd h hh))

theorem residues_of_om (p r : Nat) (hr : r < p) (hom : OM p r) : ∀ h ∈ H45, h % p ≠ r :=
  fun h hh => (omit_iff h p r hr).mp (conjL_elim _ hom _ (List.mem_map.mpr ⟨h, hh, rfl⟩))

/-- The table, decided: every listed class is below its prime and omitted. -/
theorem table_omits : ∀ q ∈ table, q.2 < q.1 ∧ ∀ h ∈ H45, h % q.1 ≠ q.2 := by
  decide

"""
LEAN_TAIL = """
/-- The paper's counting step: a list shorter than n misses some number below n. -/
theorem missing_below : ∀ (n : Nat) (L : List Nat), L.length < n → ∃ r, r < n ∧ r ∉ L
  | 0, _, h => absurd h (Nat.not_lt_zero _)
  | n + 1, L, h => by
    by_cases hn : n ∈ L
    · have hpos := List.length_pos_of_mem hn
      obtain ⟨r, hr, hrL⟩ := missing_below n (L.erase n) (by rw [List.length_erase_of_mem hn]; omega)
      exact ⟨r, by omega, fun hm => hrL ((List.mem_erase_of_ne (by omega)).mpr hm)⟩
    · exact ⟨n, by omega, hn⟩

/-- Every modulus p ≥ 46 has an omitted class: 45 residues cannot cover p classes. -/
theorem large_moduli (p : Nat) (hp : 46 ≤ p) : ∃ r, r < p ∧ ∀ h ∈ H45, h % p ≠ r := by
  obtain ⟨r, hr, hrL⟩ := missing_below p (H45.map (· % p)) (by simp [H45]; omega)
  exact ⟨r, hr, fun h hh he => hrL (List.mem_map.mpr ⟨h, hh, he⟩)⟩

/-- The primes below 47 are the table's primes. -/
theorem primes_below_47 (p : Nat) (hp : IsPrime p) (h47 : p < 47) : p ∈ table.map Prod.fst := by
  have bounded : ∀ p ∈ List.range 47,
      (2 ≤ p ∧ ∀ d ∈ List.range 47, d ∣ p → d = 1 ∨ d = p) → p ∈ table.map Prod.fst := by
    decide
  exact bounded p (List.mem_range.mpr h47)
    ⟨hp.1, fun d _ hd => hp.2 d hd⟩

/-- Lemma 12.1 (admissibility), in Lean, from RLT's 14 statements plus the two
steps above. -/
theorem lemma_12_1_of_rlt_statements (hrlt : ∀ q ∈ table, OM q.1 q.2) : Admissible H45 := by
  intro p hp
  by_cases h47 : p < 47
  · obtain ⟨q, hq, rfl⟩ := List.mem_map.mp (primes_below_47 p hp h47)
    exact ⟨q.2, (table_omits q hq).1, residues_of_om q.1 q.2 (table_omits q hq).1 (hrlt q hq)⟩
  · exact large_moduli p (by omega)

/-- The 14 statements hold (in Lean), so Lemma 12.1 holds in Lean. -/
theorem lemma_12_1 : Admissible H45 :=
  lemma_12_1_of_rlt_statements (fun q hq => om_of_residues q.1 q.2 (table_omits q hq).1 (table_omits q hq).2)

end RLT006Direct
"""
TH_NAMES = ["conjFrom_intro", "conjFrom_elim", "conjL_intro", "conjL_elim", "omit_iff", "om_of_residues",
            "residues_of_om", "table_omits"]
body = LEAN_HEAD.replace("HLIST", "[" + ", ".join(map(str, H)) + "]").replace(
    "TLIST", "[" + ", ".join(f"({p}, {r})" for p, r in zip(P, R)) + "]")
for p, r in zip(P, R):
    body += (f"/-- RLT's `admissible.h45.omits.{p}_{r}` (tests/rlt006/accept/OM_{p:02d}.rlt), literally. -/\n"
             f"theorem om_{p} :\n    {lean_lit(p, r)} :=\n  om_of_residues {p} {r} (by decide) (by decide)\n\n")
    TH_NAMES.append(f"om_{p}")
body += LEAN_TAIL
TH_NAMES += ["missing_below", "large_moduli", "primes_below_47", "lemma_12_1_of_rlt_statements", "lemma_12_1"]
body += "\n" + "".join(f"#print axioms RLT006Direct.{n}\n" for n in TH_NAMES)
write("lean/RLT006Direct.lean", body)
print(f"lean/RLT006Direct.lean: {len(TH_NAMES)} theorems")


# 6. Statement records, one per per-prime theorem.
BASIS = "docs/RLT-006-CORRESPONDENCE.md"
basis_sha = hashlib.sha256(open(BASIS, "rb").read()).hexdigest()
reg_ex = {it["id"]: it["text"] for it in T["items"]}


def yq(x):
    return '"' + x.replace("\\", "\\\\").replace('"', '\\"') + '"'


for old in [f for f in os.listdir("statements") if f.startswith("H45-OMITS-")]:
    os.remove(os.path.join("statements", old))
for p, r in zip(P, R):
    path = f"{ACC}/OM_{p:02d}.rlt"
    text = open(path, encoding="utf-8").read()
    goal = re.search(r'\(goal "(.*)"\)', text).group(1)
    checked = goal[1:-1] if goal.startswith("(") and goal.endswith(")") else goal
    rec = f"""# Statement record (RLT-006): Axiom Math, A New Bound for Small Gaps Between
# Primes (September 3, 2026), Lemma 12.1: H45 omits the class {r} mod {p}.
# GENERATED by scripts/gen-rlt006.py; pins as `rlt check {path}` reports them.
format: rlt-statement/1
id: H45-OMITS-{p:02d}
subject: "Lemma 12.1 (Axiom Math, September 3, 2026 draft): the reductions of H45 modulo {p} omit the class {r}"
source:
  registry: AXIOM-BGP212-2026-09-03
  commit: {T["pdf"]["sha256"]}
  excerpts:
    - {yq(reg_ex["h45-tuple"])}
    - {yq(reg_ex["omitted-primes"])}
    - {yq(reg_ex["omitted-classes"])}
kernel:
  proof: {path}
  sha256: {sha(text)}
  convention: NAT-DEF
  axiom_set: ROBINSON-Q
  axiom_set_sha256: 4ad55192222576e3901abc4ee7ea5a6760b64ff212e8c55796ab9847e423b00c
  checked_goal: {yq(checked)}
  assumptions: []
semantic:
  status: partial
  assessed_by: "RLT-006 review (docs/RLT-006-CORRESPONDENCE.md)"
  date: 2026-11-12
  basis: {{path: {BASIS}, sha256: {basis_sha}}}
  assessed_goal: {yq(checked)}
  matches:
    - "For every element h of the transcribed H45, h is not p*k + {r} for any natural k; over the naturals, with 0 <= {r} < {p}, this is exactly: the reductions of H45 modulo {p} omit the class {r}, the entry for {p} in the proof of Lemma 12.1 (Lean omit_iff, om_{p})."
  differs:
    - "Lemma 12.1 also needs: 45 distinct elements with minimum 0 and maximum 212; that 2, 3, ..., 43 are all the primes up to 43; and, for p >= 47, the counting argument. RLT proves none of these here (Lean comparisons 1 and 2 do)."
  lean:
    - {{file: lean/RLT006Direct.lean, theorem: RLT006Direct.om_{p}}}
    - {{file: lean/RLT006Direct.lean, theorem: RLT006Direct.lemma_12_1_of_rlt_statements}}
    - {{file: lean/RLT006BGP212.lean, theorem: RLT006BGP212.omits_rlt_form}}
physical:
  applicability: not-applicable
  reason: "a statement in number theory"
open_questions:
  - "Admissibility for every prime p >= 47 inside RLT: the paper's counting argument needs a pigeonhole principle RLT has not proved; a Q-only route (finite certificates below 213, class 1 for p >= 213) is noted in docs/RLT-006-CORRESPONDENCE.md section 4."
"""
    write(f"statements/H45-OMITS-{p:02d}.yaml", rec)
print("14 statement records written to statements/")

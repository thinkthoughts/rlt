# RLT-007b stage 3 — Report: H₄₅ omits the class 1 modulo every p > 212

**Status:** implemented on branch `rlt-007b-3` (draft PR), 2026-11-12, on main at f37bcfc (stage 2a merged). Not merged.

**Unchanged:**
- the kernel, the 20 inference rules plus `cite`, ROBINSON-Q and `spec/definitions.rlt`;
- the resource limits;
- every earlier proof, report, record and library entry.

## 1. The theorem

    TAIL:  ∀p. (212 < p → ⋀_{h ∈ H₄₅} ¬(∃k. h = (p × k) + 1))

- One file, `tests/rlt007s3/accept/TAIL.rlt`, kernel accepted, unconditional under ROBINSON-Q (Γ = ∅), citing nothing.
- Library id `admissible.h45.omits.tail_1` (the name in the Option B design), statement record `H45-TAIL-OMITS` (semantic `partial`, current).
- The conjunction is left-nested over the 45 transcribed elements, the shape of RLT-006's per-prime statements with r = 1 and the modulus a variable.

**Exactly what the assembly needs.**
- **The bound.** `212 < p` is the bound the stage-2a classification produces, so the assembly instantiates TAIL directly. The least true bound is 211 (Lean `bound_210_fails` refutes 210); stating 211 would be stronger than needed.
- **k.** k ranges over all naturals, with no restriction and no positivity hypothesis: k = 0 gives h = 1 (not listed), and k ≥ 1 gives h ≥ 214.
- **p.** The statement covers every p > 212, prime or not; nothing in it uses primality. It applies to the primes p ≥ 213 by instantiation, and the correspondence review keeps that relationship explicit (`docs/RLT-007b-3-CORRESPONDENCE.md` §2–3).
- **1 < p** (the class is a residue below p) is left to the assembly, where it follows from 212 < p.

## 2. The proof, and why it is small

The approved prototype route is kept: Q3 on k; k = 0 gives h = 1; k = S(j) gives h = S²¹⁴((p × j) + z) from 212 < p, refuted for each listed h. Two changes to how it is written cut the size:
- **One file instead of 45 certificates and a summary.** The steps every element shares (p = S²¹³(z), (p × 0) + 1 = 1, and (p × S(j)) + 1 = S²¹⁴((p × j) + z) for all j) are proved once, inside the file, and used by all 45 elements. They are steps of one proof, not library theorems.
- **Successor lemmas by composition.** D_n: ∀x. ∀y. x + Sⁿ(y) = Sⁿ(x + y) and Q_n: ∀x. ∀y. (Sⁿ(x) = Sⁿ(y) → x = y) are built from add_succ and q1_succ_inj by composing D_m and D_k (or Q_m and Q_k) into D_{m+k}, so a lemma for n costs O(n) nodes and the family O(n log n), instead of a successor chain's O(n²). D_212 is built by doubling; Q_h follows the listed elements, each from the previous one and the gap.

| Measure | Prototype (RLT-007-REVIEW) | Stage 3 |
|---|---|---|
| Files | 45 certificates + 1 summary | 1 |
| Checked steps | 69,445 | 1,750 |
| Size | 52 MB | 682,142 bytes; 495,022 nodes of the 1,000,000 limit |
| Kernel check time | 43 s | about 0.5 s |
| Citations | 45 of 64 | 0 |

Axioms used: all seven of ROBINSON-Q. Rules used: and-intro, cong, exists-elim, forall-elim, forall-intro, hyp, imp-elim, imp-intro, inst, or-cases, subst, symm, trans.

## 3. Generator, negative cases and checks

`scripts/gen-rlt007s3.py`, deterministic and untrusted (every proof is kernel-checked).
- **Write mode** produces 10 outputs: the theorem, 6 rejected cases with `expected.tsv`, the Lean file and the record.
- **`--check`** regenerates in memory and compares, with exact file sets.
- **`--self-test`** refuses 6 of 6 altered tuples (1 added, 213 for 212, 44 entries, a duplicate, out of order, a negative entry).
- **Source transcription check** (not kernel-checked), in the generator and independently in verify.sh: the tuple is the transcribed H₄₅, 45 increasing entries from 0 to 212, without 1.

**Rejected cases.** Each states something false, or a statement other than the one proved:

| Case | Why it is refused |
|---|---|
| SR01 the bound lowered to 210, regenerated (false: 212 = 211 × 1 + 1) | the h = 212 refutation has nothing to refute: step H212_1n |
| SR02 the element 212 replaced by 214, regenerated (false: 214 = 213 × 1 + 1) | the same, at H214_1n |
| SR03 the element 0 replaced by 1, regenerated (false: 1 = p × 0 + 1) | the k = 0 branch yields 1 = 1, no contradiction |
| SR04 the statement changed to the class 2 (false: 2 = p × 0 + 2) | the last step proves the class-1 statement |
| SR05 the statement with the element 212 dropped | the last step proves the 45-element statement |
| SR06 the statement with the hypothesis 212 < p dropped (false for p = 1) | the last step proves the bounded statement |

The trusted Rust test `cli/tests/rlt007s3.rs` (5 tests) rebuilds the statement from H₄₅ written out independently in the test and requires exact equality with the committed goal.

## 4. Lean comparison

`lean/RLT007S3.lean`, generated, core Lean only: 8 theorems, each printed and gated by name, with standard axioms only.
- `tail`: RLT's statement over ℕ, literally.
- `key`: one element (k = 0, or k ≥ 1 and p × k ≥ p).
- `one_mod_iff`: for p > 1, (∃k. h = p × k + 1) ↔ h % p = 1.
- `tail_mod`: every element of H₄₅ has remainder other than 1 modulo every p > 212.
- `bound_210_fails`: the statement with 210 < p is false.
- `h45_eq`, `small`, `lt_iff`: the list's shape and the reading of `<`.

The record names `tail`, `one_mod_iff` and `tail_mod`; the pinned count of Lean references in records is now 47, previously 44.

## 5. Verification

- `cargo test --release`: all suites pass, including the 5 new tests (§7 has the totals).
- `scripts/verify.sh`: new stage-3 section (self-test, `--check`, the transcription check, acceptance with Γ = ∅, 6 of 6 rejected cases); Lean replay of 21 files with the new file under the by-name gate; the record exits 0; the library has 126 entries over 128 proofs.

## 6. Independent audits

Recorded in §8 after they complete.

## 7. Measured totals

Recorded with the final commit.

# RLT-007b stage 3 — Report: H₄₅ omits the class 1 modulo every p > 212

**Status:** implemented on branch `rlt-007b-3` (draft PR), 2026-11-12, on main at f37bcfc (stage 2a merged). Not merged.

**Unchanged:**
- the kernel, the 20 inference rules plus `cite`, ROBINSON-Q and `spec/definitions.rlt`;
- the resource limits;
- every earlier proof, record and library entry. Two earlier documents gain notes: the stage-2a report states the direction of its result, and the stage-2 design review records stage 2b as deferred.

## 1. The theorem

    TAIL:  ∀p. (212 < p → ⋀_{h ∈ H₄₅} ¬(∃k. h = (p × k) + 1))

- One file, `tests/rlt007s3/accept/TAIL.rlt`, kernel accepted, unconditional under ROBINSON-Q (Γ = ∅), citing nothing.
- Library id `admissible.h45.omits.tail_1` (the name in the Option B design), statement record `TAIL-OMITS-H45` (semantic `partial`, current).
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
- **`--self-test`** refuses 6 of 6 altered tuples (1 in place of 2, which only the check for 1 catches; 213 for 212; 44 entries; a duplicate; out of order; a negative entry).
- **Source transcription check** (not kernel-checked), in the generator and independently in verify.sh: the tuple is the transcribed H₄₅, 45 increasing entries from 0 to 212, without 1. verify.sh also rebuilds the statement from the transcribed tuple and compares it with the goal of TAIL.rlt and with the record's checked goal.

**Rejected cases.** Each states something false, a statement other than the one proved, or a corrupted proof step:

| Case | Why it is refused |
|---|---|
| SR01 the bound lowered to 210, regenerated (false: 212 = 211 × 1 + 1) | the h = 212 refutation has nothing to refute: step H212_1n |
| SR02 the element 212 replaced by 214, regenerated (false: 214 = 213 × 1 + 1) | the same, at H214_1n |
| SR03 the element 0 replaced by 1, regenerated (false: 1 = p × 0 + 1) | the k = 0 branch yields 1 = 1, no contradiction |
| SR04 the statement changed to the class 2 (false: 2 = p × 0 + 2) | the last step proves the class-1 statement |
| SR05 the statement with the element 212 dropped | the last step proves the 45-element statement |
| SR06 the statement with the hypothesis 212 < p dropped (false for p = 1) | the last step proves the bounded statement |
| SR07 the statement with the stronger bound 211 < p (true, but not the statement proved) | the last step proves the 212 statement |
| SR08 the statement with its first two conjuncts swapped (no longer the OM_p shape) | the last step proves the listed order |
| SR09 inside the proof, p = S²¹²(z) claimed instead of S²¹³(z) | refused at step PZ |

The element 214 in SR02 is the first false replacement for 212: with 213 the statement stays true (213 = p × k + 1 needs p = 212).

The trusted Rust test `cli/tests/rlt007s3.rs` (5 tests) rebuilds the statement from H₄₅ written out independently in the test and requires exact equality with the committed goal.

## 4. Lean comparison

`lean/RLT007S3.lean`, generated, core Lean only: 10 theorems, each printed and gated by name, with standard axioms only.
- `tail`: RLT's statement over ℕ, literally.
- `key`: one element (k = 0, or k ≥ 1 and p × k ≥ p).
- `one_mod_iff`: for p > 1, (∃k. h = p × k + 1) ↔ h % p = 1.
- `tail_mod`: every element of H₄₅ has remainder other than 1 modulo every p > 212.
- `int_class_iff`: for h ≥ 0 and p > 1, the integer form ∃k ∈ ℤ. h = p·k + 1 equals the natural-number form.
- `bound_210_fails` and `bound_211_holds`: the statement fails with 210 < p and holds with 211 < p, so 211 is the least true bound.
- `h45_eq`, `small`, `lt_iff`: the list's shape and the reading of `<`.

The record names `tail`, `one_mod_iff`, `tail_mod` and `int_class_iff`; the pinned count of Lean references in records is now 48, previously 44.

## 5. Verification (after the audit fixes)

- `cargo test --release`: 292 of 292 tests pass, previously 287; the 5 new tests are in `cli/tests/rlt007s3.rs`. fmt and clippy (`-D warnings`) are clean.
- `scripts/verify.sh`: `RESULT all checks hold`, 22 min 47 s wall. The new stage-3 section reports:
  - self-test 6 of 6; `--check` 13 outputs, 0 differ;
  - the transcription check, with the statement of TAIL.rlt and its record rebuilt from the transcribed tuple;
  - TAIL.rlt accepted, unconditional, 1,750 steps;
  - 9 of 9 rejected cases.
- Lean replay of 21 files; `RLT007S3.lean` 10 statements, each printed by name, standard axioms only; the 4 stage-3 references in the record checked (48 in all).
- The record `TAIL-OMITS-H45` exits 0 (current, semantic `partial`).
- Library: 126 of 126 entries current and accepted; the identity registry keeps every line of 5 committed versions.
- Earlier stages: their verify lines are unchanged, including the stage-2a Lean gate lines.

## 6. Independent audits

Two independent adversarial audits ran on 1d07707, one mathematical and one on the implementation. Neither found a blocking defect.

| Finding | Change |
|---|---|
| "211 is the least true bound" was stated, but only the failure at 210 was proved | Lean `bound_211_holds`; the review cites both |
| The step from integer residue classes to the natural-number form was argued only in prose | Lean `int_class_iff`, cited in the review and the record |
| The self-test's "1 added" case was also caught by the endpoint check, so the check for 1 was untested | the case is now 1 in place of 2, caught only by the check for 1 |
| verify.sh's stage-3 line trusted the generator for which statement was proved | verify.sh rebuilds the statement from the transcription and compares it with TAIL.rlt and the record |
| SR01's refusal relied on a successor helper silently accepting a negative count | the helper asserts a nonnegative count; the rejected case builds its step explicitly (same refusal) |
| Missing negative cases: a stronger but true bound, a reordered conjunction, a corrupted proof step | SR07, SR08, SR09 |
| A sentence described the future assembly in the present tense | reworded |
| (Found by the full verify.sh run after the fixes, not by the audits.) The record's first name, `H45-TAIL-OMITS`, fell under stage 1's `--check`, which treats every `statements/H45-*` other than `H45-OMITS-*` as a stage-1 record | renamed `TAIL-OMITS-H45`; stage 1's generator is unchanged |

**Notes kept for later stages.**
- The kernel's goal comparison ignores bound-variable names (a copy of TAIL.rlt with its goal written over q is accepted). The composition design matches statements by fingerprint and printed form, so its checker must compare canonical goals, not the `goal` text of a file.
- The generator's RLT_BINARY argument is accepted but unused, as in stage 2a; kernel checking is done by verify.sh and the tests.

The auditors verified independently:
- the statement is true, no stronger than the assembly needs, and has the OM_p shape and order;
- the 45 elements equal the transcription and the paper's display;
- the proof's structure (D_n, Q_n, the two branches, the discharge of 212 < p), with nine further adversarial files refused or, where true, accepted as expected (bound 211 accepted; 213 for 212 accepted);
- `tail` is literally RLT's goal (it proves the statement `rlt lean` emits);
- `gen-lib.py` reproduces `lib/` exactly, `rlt deps --verify-lib` accepts, the identity registry is append-only, and all 32 earlier records still explain with exit 0.

## 7. Measured totals

| Measure | Value |
|---|---|
| Proofs | 1 (TAIL.rlt), no citations |
| Checked steps | 1,750 |
| Certificate size | 682,142 bytes; 495,022 nodes (49.5% of the 1,000,000 limit) |
| Kernel check time | about 0.5 s (0.48–0.54 s over 5 runs) |
| Rejected cases | 9 |
| Lean theorems | 10, replay about 2 s |
| Statement records | 1 |
| Library entries | 1 (126 in all, over 128 proofs) |
| verify.sh | 22 min 47 s wall, 34 min 7 s CPU |

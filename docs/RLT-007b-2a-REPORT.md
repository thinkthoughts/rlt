# RLT-007b stage 2a — Report: the primes below 213 under Prime_B

**Status:** implemented on branch `rlt-007b-2` (PR #18, draft), 2026-11-12, after the architecture decisions of the same day. Not merged.

**Scope.** Stage 2a proves three things under Robinson's Q:
- the non-prime certificates;
- the six quantified exhaustiveness theorems;
- the classification of the primes below 213 in one direction: every Prime_B number is one of the 47 listed primes, in its range, or exceeds 212.

The 47 positive primality theorems are stage 2b, and the assembly is RLT-007d. The three results stay separate.

**Direction of the RLT result.** RLT establishes exhaustiveness: every Prime_B number below 213 is listed. The converse (each listed number satisfies Prime_B) is Lean evidence (`listed_1` … `listed_6`), not an RLT theorem. Stage 2b, the RLT form of the converse, is deferred (decision of 2026-11-12): the planned H₄₅ admissibility argument uses only exhaustiveness.

**Unchanged:**
- the kernel, the 20 inference rules plus `cite`, the axiom set ROBINSON-Q and `spec/definitions.rlt`;
- the resource limits (1,000,000 nodes per file, 64 cited checks, chain 8);
- every earlier proof, report, record and library entry. verify.sh output for the earlier stages is identical to main at eb0e2ef, apart from the added stage-2a lines and the two library counts.

## 1. The definition and the ranges

Prime_B(p) :≡ 1 < p ∧ ∀d. ((1 < d ∧ d < p) → ¬(d ∣ p)), written inline in every statement with the existing `<` and `∣`. G stays a documented reference definition. The correspondence of B, G and the standard definition is a Lean result over ℕ (`pb_iff_std`, `pg_iff_std`), not an RLT theorem. The countermodel to the unguarded definition is `lean/RLT007S2Design.lean`, now replayed and gated. The full review is `docs/RLT-007b-2a-CORRESPONDENCE.md`.

| Range | Interval | Primes |
|---|---|---|
| R1 | [0, 40] | 2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37 |
| R2 | [41, 78] | 41, 43, 47, 53, 59, 61, 67, 71, 73 |
| R3 | [79, 112] | 79, 83, 89, 97, 101, 103, 107, 109 |
| R4 | [113, 156] | 113, 127, 131, 137, 139, 149, 151 |
| R5 | [157, 190] | 157, 163, 167, 173, 179, 181 |
| R6 | [191, 212] | 191, 193, 197, 199, 211 |

## 2. The theorems (19 proofs, all kernel accepted, unconditional under ROBINSON-Q)

| Family | Files | Statement |
|---|---|---|
| Non-prime certificates | N_1 … N_6 | the balanced conjunction of ¬Prime_B(m) over the non-primes m of interval I: 29, 29, 26, 37, 28 and 17, 166 in all. 0 and 1 (in N_1) by ¬(1 < m); each composite by the divisor m / spf(m) |
| Exhaustiveness | E_1 … E_6 | E_1: ∀p. ((p < 41 ∧ Prime_B(p)) → R1(p)); E_I: ∀p. (((a−1 < p ∧ p < b+1) ∧ Prime_B(p)) → R_I(p)). Each cites its N_I |
| Order splits | O_1 … O_6 | O_1: ∀p. (p < 41 ∨ 40 < p); O_I: ∀p. (a−1 < p → (p < b+1 ∨ b < p)) |
| Classification | C | ∀p. (Prime_B(p) → (R1(p) ∨ (R2(p) ∨ … (R6(p) ∨ 212 < p)))). Cites O_1 … O_6 and E_1 … E_6 |

Library ids: `primes.nonprime.i1`–`i6`, `primes.exhaustive.r1`–`r6`, `order.split.o1`–`o6` and `primes.below_213`. The library now has 125 entries over 127 proofs, previously 106 and 108.

Statement records (all exit 0, current, semantic `partial`):
- `PRIMES-RANGE-1` … `6`, one per E_I;
- `PRIMES-BELOW-213`, for C.

**Measurements.** Steps and times are from `rlt check`, release build, one file at a time. Nodes are the parser's budget, against the 1,000,000 limit.

| File | Steps | Nodes | Check time |
|---|---|---|---|
| N_1 | 857 | 59,792 | 0.07 s |
| N_2 | 984 | 135,687 | 0.15 s |
| N_3 | 938 | 184,588 | 0.19 s |
| N_4 | 1,421 | 371,078 | 0.37 s |
| N_5 | 1,189 | 367,685 | 0.35 s |
| N_6 | 857 | 272,198 | 0.26 s |
| E_1 | 858 | 113,848 | 0.19 s |
| E_2 | 1,114 | 233,139 | 0.37 s |
| E_3 | 1,335 | 370,929 | 0.59 s |
| E_4 | 1,791 | 634,352 | 1.00 s |
| E_5 | 1,948 | 805,289 | 1.10 s |
| E_6 | 1,988 | 906,237 | 1.05 s |
| O_1 | 698 | 53,854 | 0.07 s |
| O_2 | 781 | 113,909 | 0.12 s |
| O_3 | 827 | 169,324 | 0.16 s |
| O_4 | 1,099 | 306,032 | 0.27 s |
| O_5 | 1,061 | 353,209 | 0.30 s |
| O_6 | 959 | 347,594 | 0.29 s |
| C | 94 | 173,779 | 5.79 s (re-checks its 18 cited files) |
| **Total** | **20,799 checked steps** | largest 906,237 (E_6) | **12.7 s** |

**Citation costs** (`rlt deps`):
- E_I: 1 of 64 cited-file checks, chain 1 of 8.
- C: 18 of 64 checks, counted transitively (6 O_I, 6 E_I and 6 N_I), chain 2 of 8.
- N_I and O_I cite nothing.

The 19 files total 4.9 MB of text. They are committed, as stage 1's summaries are; nothing in this stage is materialized at verification time.

## 3. Lean comparison

`lean/RLT007S2a.lean`, generated, core Lean only. It has 33 theorems, each printed and gated by name, with standard axioms only (propext and Quot.sound; `dv_iff` uses none). It replays in about 5–8 s.
- `n_I`, `e_I`, `o_I` and `c` state the RLT goals over ℕ, with `LT`, `DV` and `PB` defined exactly as RLT's `<`, `∣` and Prime_B.
- `listed_I` is the converse of `e_I`: each listed number lies in its interval and satisfies Prime_B. With `e_I`, Lean shows each list is exactly the primes of its interval; in RLT, only the exhaustiveness direction is established.
- The correspondence theorems are `lt_iff`, `dv_iff`, `pb_iff_std`, `pg_iff_std`, `pb_dec` and `pb_of_dec`.

`lean/RLT007S2Design.lean`, 20 theorems, replays in about 13 s:
- the model of Q (`model_of_Q`) and the failure of the unguarded definition at 7 (`unbounded_fails_for_7`);
- `guarded_ns_not_dvd_7`, `primeB_iff` and `primeG_iff`;
- the decided classification.

The statement records name 14 new Lean references: `c`, `e_1` … `e_6`, `listed_1` … `listed_6` and `pb_iff_std`. The pinned count is now 44, previously 30.

## 4. Generator, negative cases and checks

`scripts/gen-rlt007s2a.py` is deterministic and untrusted: every proof it writes is checked by the kernel.
- **Write mode** produces 37 outputs: 19 theorems, 9 rejected cases with `expected.tsv`, the Lean file and 7 records.
- **`--check`** regenerates in memory and compares, with exact file sets in `accept/`, `reject/`, `tests/rlt007s2a/`, `statements/PRIMES-*` and `lean/RLT007S2a*`.
- **`--self-test`** refuses 8 of 8 altered range lists:
  - an omitted prime;
  - a composite included (91);
  - a duplicate;
  - intervals ending one early or one late;
  - an interval starting at 1;
  - a last interval ending at 211;
  - a valid but unapproved partition.
- **Argument errors:** a missing binary argument or an unknown flag is refused.

**Source transcription check** (not kernel-checked), made by the generator and independently by verify.sh: the 14 primes of the transcribed proof of Lemma 12.1 are the first 14 of the 47.

**Rejected cases** (each refused for its stated reason, pinned in `expected.tsv`):

| Case | Exit | Why it is refused |
|---|---|---|
| SY01 a certificate claiming ¬Prime_B(37) by 37 = 5 × 7 | 1 | `symm` yields 35 = 5 × 7 |
| SY02 range 1 without 37, citing SY01 | 2 | the cited certificate is not an accepted theorem |
| SY03 range 2 widened to [41, 79] | 1 | no certificate exists for the prime 79 |
| SY04 the split p < 41 ∨ 41 < p (41 uncovered), only the statement and last step changed | 1 | the last step proves the original split |
| SY05 the classification with a stale SHA-256 for E_6 | 2 | hash mismatch |
| SY06 the classification citing a missing file | 2 | entity not found |
| SY07 the ¬(1 < m) certificate applied to 2 | 1 | `symm` yields 2 = S(z0) + 1, not 0 = … |
| SY08 range 6 with Prime_B's hypothesis replaced inside the proof steps | 1 | the `subst` motive no longer matches |
| SY09 range 6 with the written statement weakened to 0 < d | 1 | the last step proves the approved statement, not the weakened one |

The trusted Rust test `cli/tests/rlt007s2a.rs` (9 tests) rebuilds every one of the 19 statements independently. It takes the approved intervals, computes the primes by trial division and requires exact equality with each committed goal. So an extra entry in a list, a shifted bound or a weakened Prime_B is refused even if the generator and Lean were both wrong.

## 5. Verification

- **`cargo test --release`:** 287 of 287 tests pass, previously 278; the 9 new tests are in `cli/tests/rlt007s2a.rs`.
- **`scripts/check-manual.py`:** 33 of 33 manual examples match.
- **cargo fmt and clippy (`-D warnings`):** clean.
- **`scripts/verify.sh`:** `RESULT all checks hold`, in 22 min 20 s wall time (§7).
- **New stage-2a section:**
  - self-test 8 of 8; `--check` 37 outputs, 0 differ;
  - source transcription check;
  - 19 of 19 kernel accepted;
  - `rlt deps` 7 of 7 with the pinned budgets;
  - 9 of 9 rejected cases.
- **Lean replay:** 20 files, including both new files under the by-name axiom gate.
- **Statement records:** 7 new records exit 0.
- **Library:** `rlt deps --verify-lib` 125 of 125 current and accepted; the identity registry keeps every earlier line.
- **Earlier stages:** verify.sh output is identical to main at eb0e2ef apart from the additions above.

## 6. Independent audits

Two independent adversarial audits ran on the implementation, one mathematical and one on the implementation. Neither found a blocking defect. Their findings and the changes made:

| Finding | Change |
|---|---|
| A composite added to a list would still give a true (weaker) E_I; only the untrusted generator guarded exactness, and the correspondence doc overstated what Lean decided | Lean `listed_1` … `listed_6` (the converse direction), referenced by the records; the Rust test rebuilds every goal independently; doc corrected |
| The Rust test pinned Prime_B but not the bounds or lists | exact reconstruction of all 19 goals (`every_statement_is_exactly_the_approved_one`) |
| SY08's name described an unbounded-definition case it does not test | renamed `SY08_range_6_step_hypothesis_altered`; new SY09 weakens the written statement |
| SY04 and SY07 were refused at side effects of global text replacement | rebuilt to change only the intended steps; reasons pinned at the intended step |
| SY02's pinned reason was too short | full citation-failure prefix pinned |
| The Lean axiom gate compared counts only | the gate checks every declared theorem by name (a `#print axioms` line and an output line each) |
| `--check` without the binary argument ran write mode | argument errors are refused |
| A valid but unapproved partition passed the generator's range checks | approved endpoints held separately; new self-test case |
| The correspondence doc attributed the enumeration of d to the E_I proofs; the "why up to 212" rationale was missing | both sentences corrected |
| Stale references: the Lean design-file header, the design doc's table caption and a doubled bullet, and the report link | fixed |

The auditors verified, independently:
- all 19 goals, character for character;
- the 166 non-primes and the 47 primes;
- the countermodel against its Lean definitions;
- that `rlt explain` refuses a record whose `checked_goal` pin is edited;
- that `gen-lib.py` reproduces `lib/` exactly;
- that nothing outside the stage's files changed.

## 7. Measured totals

- **Proofs:** 19 new, all accepted, with 20,799 checked steps in all.
- **Kernel check time:** 12.7 s for the 19 files, one at a time.
- **Citation costs:** at most 18 of 64 checks, chain 2.
- **Rejected cases:** 9.
- **Lean theorems:** 33 in the comparison and 20 in the design file.
- **Statement records:** 7.
- **Library entries:** 19.
- **verify.sh:** 22 min 20 s wall time (33 min 29 s CPU) for the run after the audit fixes. The first run, before the fixes, took about 26 minutes with both audits running on the same machine. Of the added time, `cli/tests/rlt007s2a.rs` takes about 3.4 min inside `cargo test` (it re-checks each file several times: acceptance, citation costs, the deps–check comparison).

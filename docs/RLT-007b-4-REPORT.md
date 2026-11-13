# RLT-007b stage 4 — Report: H₄₅ omits a class modulo each prime 47 … 211

**Status:** implemented on branch `rlt-007b-4` (draft PR), 2026-11-13, on main at 4e4ca04 (stage 3 merged). Not merged.

**Unchanged:** the kernel, the 20 inference rules plus `cite`, ROBINSON-Q, `spec/definitions.rlt`, the resource limits, and every earlier proof, record and library entry.

## 1. What is proved

For each of the 33 primes p from 47 to 211, with its class r_p:

| Theorem | Statement | Count |
|---|---|---|
| Element certificate `D_PPP_HHH` | ¬(∃k. h = (p × k) + r_p) | 1,485 (generated, pinned) |
| Per-prime summary `OM_PPP` | ⋀_{h ∈ H₄₅} ¬(∃k. h = (p × k) + r_p), left-nested, citing its 45 certificates | 33 |

- All kernel accepted, unconditional under ROBINSON-Q.
- The summaries have exactly the shape of RLT-006's 14 per-prime theorems, and their library ids follow the same pattern: `admissible.h45.omits.47_1` … `admissible.h45.omits.211_3`.
- Statement records `OMITS-H45-047` … `OMITS-H45-211` (semantic `partial`), basis `docs/RLT-007b-4-CORRESPONDENCE.md`.

**The classes.** The paper lists no class for p ≥ 47 (it argues by counting). The classes are RLT's choice: for each prime, the least class H₄₅ omits, as measured in RLT-007c. 1 for 15 primes, 3 for 12, 4 for 4, and 6 for 53 and 59. The generator recomputes and refuses any difference; Lean `table_facts` decides them.

**The primes.** Exactly the primes 47 … 211: the primes of the approved stage-2a ranges R2 … R6. Lean `below_213_covered` decides that every prime below 213 is in RLT-006's transcribed table or in this one.

## 2. The construction

The RLT-007c direct scheme, unchanged: peel the r successors of (p × k) + r; split k with Q3 at most ⌊(h − r)/p⌋ + 1 times; at each k = S(k′) layer, prove once over a fresh parameter w a lemma (∀w. n = w + p → n − p = w, or ∀w. ¬(n = w + p)) and instantiate it at p × k′, so the long successor chains never contain p × k′. Robinson's Q only; no induction, no definitions.

## 3. Measurements

| Measure | Value |
|---|---|
| Element certificates | 1,485 of 1,485 accepted |
| Largest certificate | 324,133 nodes (`D_199_212`), 32% of the 1,000,000 limit |
| Mean / total nodes | 127,851 / 189,859,061 |
| Checked steps (certificates) | 729,571 in all; at most 967 per file, median 495 |
| Disk (generated, not committed) | 50,691,554 bytes (about 52 MB on disk) |
| Generation time | under 1 s |
| Kernel check, all certificates, one process | 194 s |
| Summaries | 33, 1.9 MB committed in all; largest 332,533 nodes; 2,937 steps in all |
| Citation budget per summary | 45 of 64 cited-file checks, chain 1 of 8 |
| Kernel check, all summaries with their cited certificates, one process | 199.5 s |

These match the RLT-007c prototype (324,133 nodes; 729,571 steps).

## 4. Reproducibility, negative cases and checks

`scripts/gen-rlt007s4.py`, deterministic and untrusted:
- **Write mode** produces 1,561 outputs; **`--materialize-only`** writes only `tests/rlt007s4/gen/` (used by verify.sh and the tests before checking); **`--check`** regenerates in memory and compares everything, including the 1,485 generated files, with exact file sets.
- **`tests/rlt007s4/manifest.sha256`** pins every certificate; each summary also pins its 45 citations by SHA-256.
- **`--self-test`** refuses 6 of 6 altered inputs, each for exactly its own reason (so each check is exercised on its own): an occupied class (2 mod 47); an omitted class that is not the least (9 mod 53); a class outside 0 … p − 1 (211 mod 211); a prime missing (101); a composite added (91, with its least omitted class); two tuple elements swapped (residues unchanged, so only the tuple check applies).

**Rejected cases** (each refused for a pinned reason):

| Case | Refused |
|---|---|
| SQ01 a certificate for the occupied class 2 mod 47 at h = 2 | at step q2: 0 = 47 × k0 is not refuted by Q2 |
| SQ02 a certificate for the altered member 48 = 47 + 1, class 1 mod 47 | at step L0q: 0 = 47 × k1 is not refuted by Q2 |
| SQ03 OM_211 with its last member altered to 214 (= 211 + 3) | the cited certificate proves 212, not 214 |
| SQ04 OM_211 citing the certificate for 212 mod 199 | the cited theorem is for another prime |
| SQ05 OM_211 with a stale certificate hash | hash mismatch (malformed) |
| SQ06 OM_211 citing a missing certificate | entity not found (malformed) |
| SQ07 OM_47 whose written statement names the class 2 | the last step proves the class 1 |

**Statement guards** (`tests/rlt007s4/guard/`). The kernel accepts any true statement it is given a proof of, so a summary with a conjunct dropped, reordered or repeated, cited from genuine certificates, is accepted (the mathematical audit showed this). Three guard files are such summaries; verify.sh and the Rust tests require that the kernel accepts each and that the independent statement rebuild refuses it. That rebuild, not the kernel, fixes the approved statement.

**verify.sh**, new stage-4 section and early materialization:
- 1,485 certificates materialized and compared with the manifest before any check;
- self-test, `--check`;
- an independent check (not the generator) that rebuilds the tuple, the 33 primes, the least classes and every summary and record statement, and checks each Lean `om_P` against its summary symbol for symbol;
- all 1,485 certificates checked; the 33 summaries through `rlt deps` with the 45-of-64 budget pinned;
- 7 of 7 rejected cases; 3 of 3 statement guards;
- tamper tests: a missing certificate, a modified certificate, an altered manifest hash, and a record with an altered source pin (refused by `rlt explain` because its commit differs from the registry's).

The trusted Rust test `cli/tests/rlt007s4.rs` (10 tests) rebuilds every certificate goal and every summary statement from H₄₅ written out in the test and from the primes and classes it computes itself.

## 5. Lean comparison

`lean/RLT007S4.lean`, generated, core Lean only: 37 theorems, each printed and gated by name, standard axioms only.
- `om_47` … `om_211`: RLT's 33 statements, symbol for symbol;
- `omit_iff`, `element`: the residue reading;
- `table_facts`: the primes are exactly 47 … 211; each class is below its prime, omitted, and the least omitted class;
- `below_213_covered`: every prime below 213 has a listed class (RLT-006's table or this one).

The records name `om_P` and `table_facts`; the pinned count of Lean references in records is now 82, previously 48.

## 6. Independent audits

Two independent adversarial audits ran on fa59aab, one mathematical and one on the implementation. Neither found a blocking defect.

| Finding | Change |
|---|---|
| A cli test (`m3a.rs`, the result-kind sweep) walked `tests/` including the 1,485 generated certificates and the summaries, twice each in one thread: about 12 extra minutes per verify and CI run | the sweep skips `tests/rlt007s4`, as `deps.rs` does (stage 4 is checked by its own suite and verify.sh); measured by the auditor: 13 min 50 s → 1 min 21 s |
| The self-test's "not the least" case (7 mod 53) was refused as occupied, and the altered-element case only through an occupied class, so two checks were never exercised alone | each case must be refused for exactly its own reason; 9 mod 53, and a tuple swap that leaves the residues unchanged |
| The kernel accepts a true summary of another shape (a conjunct dropped); no test showed that the statement rebuild catches it | three guard files, accepted by the kernel and refused by the rebuild, in verify.sh and the Rust tests |
| The review said r_p < p "is part of the decided table" for the assembly | stated as an RLT-007d obligation (a closed computation), Lean-decided here |
| Nothing mechanically checked the Lean `om_P` against the RLT goals | verify.sh compares them symbol for symbol |
| Wording: the primes are "the primes ≥ 47" of R2 … R6 (41 and 43 are RLT-006's); `below_213_covered` uses Lean's own trial-division predicate | corrected and stated |

**Notes kept.** SQ07's pinned reason is the opening of the expected goal; the generators accept but do not use their RLT_BINARY argument, as before; the altered-manifest tamper test is simple by nature (the missing and modified certificate tests are the substantive ones).

**The auditors verified independently:** the 33 primes, classes, omission and leastness; all 1,485 certificate goals and all 33 summary goals and citations; all certificates and summaries accepted by the kernel; five tampered proofs refused for the right reasons; the Lean statements equal the RLT goals; `gen-lib.py` reproduces `lib/`; `rlt deps --verify-lib` 159 of 159; all 65 records explain with exit 0; nothing outside the stage's scope changed.

## 7. Verification and totals

Recorded with the final commit.

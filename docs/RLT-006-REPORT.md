# RLT-006 — Report: the H₄₅ residue certificates for Lemma 12.1

**Target** (approved 2026-11-12): an independent first-order verification of the finite residue certificates supporting Lemma 12.1 of Axiom Math's *A New Bound for Small Gaps Between Primes*, Preliminary Draft: September 3, 2026.

- Branch `docs/rlt-006`, draft PR #14, based on main at `9c8a66d`.
- The kernel, inference rules, registered axiom sets and kernel resource limits are unchanged.
- No release or deployment.

The five results below are reported separately, as requested. Only §1 and §2 are RLT results.

## 1. The 630 accepted element certificates (RLT kernel)

For each of the 14 listed pairs (p, r_p) and each h ∈ H₄₅, `tests/rlt006/gen/O_PP_HHH.rlt` proves:

```
¬(∃k. h = (p × k) + r_p)          Robinson's Q, NAT-DEF, Γ = ∅
```

The construction:

- add_succ, applied r times, gives h = S^r(p·k).
- **If h ≥ r_p:** peeling r_p successors (Q1) leaves h − r_p = p·k. The M3c case tree on k (Q3, mul_succ, Q1, Q2) then refutes it.
- **If h < r_p:** peeling h successors leaves 0 = S^(r_p−h)(p·k), refuted by Q2.

The generator does not decide which claims are true: for a false claim the same construction is refused by the kernel. RX01 (46 ≡ 3 mod 43) and RX02 (12 ≡ 0 mod 3) fail at the case-tree branch that holds the true witness.

| | Measured (this sandbox, release build) |
|---|---|
| Certificates | 630, all ACCEPT as unconditional theorems under ROBINSON-Q |
| Steps | 551,016 in total; 11 to 3,299 per file |
| Generation | 1.0 s for the 630 files (`--materialize-only`); 3.2 s for the full generator (summaries, rejected cases, tables, records, Lean file) |
| Checking | 84 s for all 630 (`rlt check`, one process per file) |
| Disk | 57.5 MB generated (`tests/rlt006/gen`, gitignored); about 1.5 MB committed for everything RLT-006 adds |

**Reproducibility.** `scripts/gen-rlt006.py` is deterministic. `tests/rlt006/manifest.sha256` pins every generated file, in `sha256sum -c` format.

- verify.sh first materializes the certificates and checks the manifest, before `cargo test` and before any library or dependency check.
- It later regenerates every committed output and compares them byte for byte.
- `cargo test` materializes the certificates itself when they are absent, as on a fresh CI checkout, and checks them against the manifest.

## 2. The 14 accepted per-prime theorems (RLT kernel)

`tests/rlt006/accept/OM_PP.rlt` cites the 45 certificates for p, each by path and SHA-256. It proves their left-nested conjunction: "the reductions of H₄₅ mod p omit r_p".

| Library id | p | r_p | `rlt deps` |
|---|---|---|---|
| `admissible.h45.omits.2_1` … `admissible.h45.omits.43_3` | 2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43 | 1, 1, 3, 6, 9, 3, 1, 13, 17, 4, 11, 4, 4, 3 | 46 theorems each, **45 of 64** cited-file checks, chain 1 of 8, exit 0 |

- **Checking:** all 14 are accepted, 86 s for the 14 `rlt deps` runs (each re-checks its 45 certificates).
- **Library:** the new area `admissible` holds the 14 entries. Their `uses` name the generated certificates as `(unlisted tests/rlt006/gen/…)`. `lib/identities.tsv` gains 14 lines (101 identities, append-only, gate passes).
- **Statement records:** 14 records, `statements/H45-OMITS-PP.yaml`, generated with pins as `rlt check` reports them. `rlt explain` exits 0:
  - kernel ACCEPT;
  - source registry verified, pinned (transcribed);
  - semantic correspondence partial (current);
  - physical not applicable.

**A missing or modified certificate is never accepted.** Each citation pins the file's path and hash, so:

- `rlt check` reports MALFORMED;
- `rlt deps` exits 2;
- `rlt deps --verify-lib` reports the entry not accepted (87 of 101 with no certificates present);
- the manifest check fails.

verify.sh and `cli/tests/rlt006.rs` test both cases, and verify.sh restores the files afterwards.

The kernel's resource limits are unchanged: each summary fits the 64-check budget with 19 to spare. A single theorem over all 14 primes would need 644 checks and is not attempted.

## 3. The positive residue profile (computation, not kernel-checked)

`certificates/h45/residues.tsv` gives, for each of the 630 pairs, q and s with h = p·q + s and 0 ≤ s < p. `certificates/h45/profile.tsv` gives, for each listed prime, the occupied classes, their number ν_p and the omitted classes.

| p | 2 | 3 | 5 | 7 | 11 | 13 | 17 | 19 | 23 | 29 | 31 | 37 | 41 | 43 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| ν_p (occupied) | 1 | 2 | 4 | 6 | 10 | 12 | 16 | 18 | 22 | 27 | 29 | 34 | 34 | 36 |
| omitted | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 2 | 2 | 3 | 7 | 7 |

The profile has 30 omitted (p, class) pairs and 251 occupied ones. These tables are computed by the generator and re-checked by plain integer arithmetic in verify.sh; the mathematics audit recomputed them independently. They are evidence, **not RLT theorems**. RLT proves that each tabled class r_p is omitted (§1–2). It does not prove the 16 further omitted classes, nor that the occupied classes are occupied.

## 4. The large-prime argument (specification; proved in Lean, not in RLT)

The paper's step: for p ≥ 47, the 45 residues h mod p take at most 45 values, so some class among p ≥ 47 is missed. RLT has not proved this.

**In Lean, comparison 1 (`lean/RLT006Direct.lean`):**

- `missing_below`: a list of length less than n misses some number below n, by induction on n.
- `large_moduli`: every p ≥ 46 has an omitted class.
- `primes_below_47`: the primes below 47 are exactly 2, …, 43.
- `lemma_12_1_of_rlt_statements`: admissibility for every prime follows from RLT's 14 statements plus those two steps.
- `lemma_12_1`: the whole lemma, adding obligation (a) (45 distinct elements, diameter 212).

**A route within Robinson's Q,** noted for a later decision in docs/RLT-006-CORRESPONDENCE.md §4: an inline primality statement below 44, finite certificates for the moduli 44 … 212, and class 1 for every p ≥ 213. It needs no induction, but it is a different proof from the paper's.

## 5. Extent of correspondence with Lemma 12.1

| Obligation of Lemma 12.1 | RLT | Lean |
|---|---|---|
| (a) 45 distinct elements, minimum 0, maximum 212 | not stated | comparison 1 `h45_size_and_diameter`; comparison 2 (lean-bgp212) |
| (b) the 14 listed omissions | **proved: 14 per-prime theorems from 630 certificates** | comparison 1 `om_P` (the same statements, literally); comparison 2 `omits_rlt_form` |
| (c) the primes below 47 are 2, …, 43 | not stated | comparison 1 `primes_below_47` |
| (d) every prime p ≥ 47 has an omitted class | not proved | comparison 1 `large_moduli` (the paper's route); comparison 2 (lean-bgp212's route) |

**Assessment: partial.** RLT independently verifies obligation (b), each of the 14 omissions, from Robinson's Q. **RLT does not verify the complete Lemma 12.1**; Lean proves the rest. The physical correspondence is not applicable.

## 6. Source fidelity

- **Pin.** The 46-page PDF, SHA-256 `2b307ae2…3b3d80` (734,121 bytes), is registered as `AXIOM-BGP212-2026-09-03` with the new status `transcribed`.
- **Reviewed transcription** (`sources/transcriptions/AXIOM-BGP212-2026-09-03.yaml`). It records:
  - the reviewer and date;
  - for each item, its PDF page and section: the definition on page 3 (§2), and the tuple, statement, primes, classes and large-prime sentences on page 43 (§12);
  - for each item, the text exactly as extracted and the reviewed text;
  - the pinned tool, `pdftotext` 24.02.0 `-layout -f PAGE -l PAGE`;
  - the SHA-256 of each page's extraction.
- **Checks.** `scripts/check_excerpts.py` re-checks all of it against the PDF, including the number lists. The generator reads the tuple and the table from the transcription.
- **Independent review.** The mathematics audit compared the tuple, the 14 primes, the 14 classes and their "respectively" pairing with the *rendered* pages, digit for digit, and confirmed the extraction hashes.
- **Tamper tests in verify.sh:** 9, all refused:
  - PDF bytes changed;
  - registry pin changed;
  - page extraction hash changed;
  - extracted text altered;
  - transcribed numbers altered;
  - omitted class altered;
  - reviewer removed;
  - tool version changed;
  - excerpt not a reviewed item.
- **Validation.** `rlt validate` checks the shape of a transcribed source (4 malformed variants refused in `cli/tests/rlt006.rs`), and `rlt explain` treats it as pinned.

## 7. Lean comparisons (Lean-checked; not RLT derivations)

| File | Theorems | What it establishes |
|---|---|---|
| `lean/RLT006Direct.lean` (generated from the transcription) | 29, standard axioms only | the 14 RLT statements literally (`om_P`); the residue reading `omit_iff`; Lemma 12.1 by the paper's route (§4); obligation (a) |
| `lean/RLT006BGP212.lean` | 12, standard axioms only | lean-bgp212 @ 7898eb4 (`BGP212/Basic.lean` lines 25–47 and `Solution/Lemma12_1.lean` lines 20–64), vendored verbatim and re-checked here. Its tuple and table equal the transcription (`same_tuple`, `same_table`); its table result in RLT's form (`omits_rlt_form`) |

verify.sh pins both theorem counts and compares the vendored blocks byte for byte with the upstream lines; CI clones lean-bgp212. Compiling them standalone also re-checks lean-bgp212's own Lemma 12.1, independently of its build.

## 8. Tests and gates

| Gate | Result |
|---|---|
| `cargo test --release` | FILL |
| `scripts/verify.sh` with the main `9c8a66d` reference | FILL |
| `scripts/compare-reports.sh` | 472 proof files compared, 0 differ. The intended difference: `rlt validate` (32 sources, `transcribed 1`) |
| Earlier `tests/explain` fixtures (52) and library entries (87) | unchanged; `rlt deps` index size 87 → 101 |
| Rejected cases `tests/rlt006/reject` | 6 of 6 with pinned exit codes and reasons: altered tuple member; altered class; summary with an altered member; summary citing another prime's certificate; stale certificate hash; missing certificate |
| `tests/explain-rlt006` | 6 of 6 fixtures: modified PDF pin; altered excerpt; stale proof hash; absent Lean theorem; rejected proof; changed basis |
| Manual | FILL (chapter 13 added) |
| CI | FILL |

**A latent gate bug was found and fixed.** Under `set -o pipefail`, `cmd | grep -q` fails when grep exits early and the writer takes SIGPIPE, which happens once the output exceeds the pipe buffer. The new Lean output pushed the replay log past it, and the RLT-005 axiom gate reported 10 of 11. All 29 such pipes in verify.sh now use `grep_q`, which reads its whole input.

## 9. Independent audits

**Mathematics and source fidelity.** Nothing blocking. Findings and resolutions:

1. *Comparison 1's "whole lemma" covered admissibility only.* **Fixed:** obligation (a) is added (`h45_size_and_diameter`), admissibility is `admissible_h45`, and `lemma_12_1` packages both.
2. *Obligation (c) must cover the primes below 47.* **Reworded.**
3. *The Q-only sketch omitted Q3, mul_zero and add_zero, and the primality step below 44.* **Fixed.**
4. *Errors in the pre-implementation review* (34 → 33 primes; "without a primality predicate"). **Corrected,** with a superseded note.
5. *The definition's paragraph label.* **Corrected:** second paragraph, the first indented one on page 3.
6. *The Lean `Admissible` leaves out distinctness.* **Documented:** it is covered by obligation (a).

The auditor's own checker confirmed every goal, citation, hash and certificate, and every Lean statement.

**Implementation.** FILL

## 10. Presenting the results to the authors (proposal, for review)

A compact, independently reproducible artifact is more useful to the authors than an announcement. Proposed contents, all already produced by this milestone:

1. **The data:**
   - the exact tuple and table as transcribed, with the PDF hash, page and section;
   - `certificates/h45/residues.tsv` and `profile.tsv`: every residue, ν_p, and all omitted classes, marked as computed.
2. **The certificates:**
   - the generator;
   - the manifest of 630 SHA-256 hashes;
   - the 14 per-prime proofs, each stating its axioms (the 7 axioms of Robinson's Q) and Γ = ∅.
3. **A one-page README:**
   - what RLT proves (obligation (b)) and what it does not prove (a, c, d), and where Lean proves them;
   - the three reproduction commands: `cargo build --release`; `python3 -I scripts/gen-rlt006.py target/release/rlt`; `rlt deps admissible.h45.omits.P_R` for each prime;
   - the expected outputs and timings.
4. **The two Lean files,** for readers who prefer Lean.

It would be packaged from a tagged commit after merge, as a single archive whose hash is recorded in the handoff. Contacting the authors is a separate decision for you. Nothing has been sent.

## 11. Final commit and merge recommendation

The final head of `docs/rlt-006` (draft PR #14) is given in the handoff message: a file cannot contain the hash of the commit that adds it. **Recommendation:** FILL

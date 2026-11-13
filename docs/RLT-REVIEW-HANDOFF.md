# RLT review handoff (2026-11-09)

Development stops after M3e. The next phase is a broader review of RLT as a usable translation and theorem-proving tool:
- the command-line experience;
- the user manual;
- the mathematical scope;
- practical applications.

## Where things stand

| Milestone | State | Main result |
|---|---|---|
| M3a | merged (9bfde18) | logical rules (20-rule kernel) |
| M3b | merged (93e85a5) | Robinson's Q; definitions of `<`, `∣` and `free235` |
| M3c | merged (73198e6) | ¬(d ∣ 7) for every witness; free235(7); theorem citation |
| M3d | merged (3d0f8e2) | ∀x. x < 30 → x = 0 ∨ … ∨ x = 29 |
| M3e | merged (cc01968, 2026-11-11) | the modulo-30 classification, E-T1; see docs/RLT-003-M3e.md |
| RLT-004a/b/c | merged (58838c9, 45c94b5, c42729b) | manual; statement records and `rlt explain`; theorem library and `rlt deps` |
| RLT-005 | merged (9c8a66d, 2026-11-11) | 8 is not matchable (McNew–Pomerance, arXiv:2604.05304v2); see docs/RLT-005-REPORT.md |
| RLT-006 | merged (e5850d9, 2026-11-12) | the 14 small-prime residue omissions of Lemma 12.1 (Axiom Math, H₄₅): 630 certificates, 14 per-prime theorems; see docs/RLT-006-REPORT.md |
| RLT-007a | merged (1561668, 2026-11-12) | verification hardening and scheduling; verify.sh 1,622 s → about 1,030 s; see docs/RLT-007a-REPORT.md |
| RLT-007b stage 1 | merged (eb0e2ef, 2026-11-12) | obligation (a) of Lemma 12.1: 44 per-element distinctness theorems, 4 group summaries, bounds theorem; see docs/RLT-007b-1-REPORT.md |
| RLT-007b stage 2a | merged (f37bcfc, 2026-11-12) | the primes below 213 under Prime_B, exhaustiveness direction: 6 non-prime certificates, 6 exhaustiveness theorems, 6 order splits, the classification `primes.below_213`; see docs/RLT-007b-2a-REPORT.md. Stage 2b (positive primality) deferred |
| RLT-007b stage 3 | merged (4e4ca04, 2026-11-12) | the uniform omission for p > 212: ∀p. (212 < p → ⋀_h ¬(∃k. h = (p × k) + 1)), one file, 1,750 steps; see docs/RLT-007b-3-REPORT.md |

**Not yet done, and held for the review:**
- the `backed by` links (designed in the M3e review §9);
- any M3f;
- release and deployment.

**Resume steps:** completed on 2026-11-11.
- The audit was read and CI confirmed.
- M3e was merged (see the merge record below).
- The broader review is in docs/RLT-004-REVIEW.md.
- RLT-004 decisions V1–V10 approved 2026-11-11 (V6 modified: measure only). RLT-004a (manual chapters 1–11, kernel soundness note, statement-record specification, unary baseline) is on PR #10, in draft; report in docs/RLT-004a-REPORT.md.

## Design principles for the review

- **Reading points first.** RLT begins with comprehensible reading points.
- **Precision.** Mathematical definitions and proof rules make their relationships precise.
- **Ordinary input.** Users can start with ordinary written expressions, such as `1 + 1 = 2` or `x < 30 → …`, without already knowing specialized mathematical vocabulary.
- **Five activities.** The interface prioritizes reading, specifying, checking, explaining and reusing.
- **Compact vocabulary.** Technical terms appear only where they mark a necessary distinction, for example theorem versus reading, or inference rule versus cited theorem.
- **Identifiers and subject names.** math0, math5 and math29 remain the original reading-point identifiers. The mathematical subject is presented with familiar terms such as **Modulo 30**.
- **Three separate reports.** Proof validity (kernel verdicts), source fidelity (pinned sources and excerpts) and physical correspondence (correspondence records, currently `unestablished`) are each reported separately.
- **RLT and Lean, accurately.**
  - RLT is an independent checker with its own 20-rule kernel. Its theorems are RLT verdicts relative to registered axiom sets.
  - Lean is used only for comparison. It re-states RLT's checked formulas over ℕ and proves correspondence lemmas, outside RLT's trusted base.
  - A Lean proof never makes an RLT theorem, and an RLT theorem is never a Lean proof.

## Useful commands

```sh
cargo build --release
scripts/verify.sh SOURCE_CLONES_DIR                       # everything, one RESULT line
target/release/rlt check tests/m3e/accept/E_T1.rlt         # the classification
target/release/rlt rp math29                               # the reading point, unchanged
target/release/rlt residues residues/residue-30.yaml       # the record, unchanged
```

## Merge record, 2026-11-11

- **PR #9 merged:** the audited head f99d146 went into main as merge commit **cc01968**, with parents 3d0f8e2 and f99d146. The merge tree 12871335 equals f99d146's tree.
- **Fresh clone of main at cc01968:** `cargo test` passes 227 of 227. `scripts/verify.sh` prints `RESULT all checks hold`, with 54 of 54 M3e proofs accepted and 16 of 16 refused.
- **CI on cc01968:** `check` and `gdt-comparison` both succeed.
- **Unchanged against 3d0f8e2:** kernel, syntax, `cli/src`, axiom sets, definitions, conventions, registry and reading-point records.

The broader review began on the same day: docs/RLT-004-REVIEW.md.

## Merge record, 2026-11-11 (RLT-004a)

- **Gate before merge:** PR #10 head was exactly 253561d. Its diff against main held only the reviewed documentation (docs/KERNEL-SOUNDNESS.md, MANUAL.md, RLT-004-REVIEW.md, RLT-004a-BASELINE.md, RLT-004a-REPORT.md, RLT-REVIEW-HANDOFF.md, STATEMENT-RECORDS.md) and two untrusted helper scripts (scripts/check-manual.py, scripts/measure-baseline.py). CI `check` and `gdt-comparison` were green on 253561d.
- **PR #10 merged:** marked ready, merged as merge commit **58838c9**, with parents cc01968 and 253561d. The merge tree 56e58eb3 equals 253561d's tree.
- **Fresh clone of main at 58838c9:** `cargo test --release` passes 227 of 227. `scripts/verify.sh` prints `RESULT all checks hold`. `scripts/check-manual.py` reports 23 of 23 manual examples matching.
- **CI on 58838c9:** `check` and `gdt-comparison` both succeed.
- **Deviations:** none.

RLT-004b (statement records and `rlt explain`) review: docs/RLT-004b-REVIEW.md.

## Merge record, 2026-11-11 (RLT-004b)

- **Gate before merge:**
  - PR #11 head was exactly 80f954d.
  - Its diff against main held only the reviewed implementation:
    - `cli/src/explain.rs`, `cli/src/lib.rs`, `cli/src/main.rs` and `syntax/src/records.rs`;
    - `cli/tests/explain.rs`;
    - `statements/`, `tests/explain/` and `tests/explain-golden/`;
    - scripts and docs.
  - Kernel, inference rules, axiom sets, conventions, definitions and reading points were unchanged, as was every earlier proof and fixture directory.
  - CI `check` and `gdt-comparison` were green on 80f954d.
  - All audit corrections were present, including the single-read proof bytes: `rlt explain` reads the proof once and checks those bytes through `check_bytes_outcome`.
- **PR #11 merged:** marked ready, then merged as merge commit **45c94b5**, with parents 58838c9 and 80f954d. The merge tree a71e4a23 equals 80f954d's tree.
- **Fresh clone of main at 45c94b5:**
  - `cargo test --release` passes 248 of 248;
  - `scripts/verify.sh` prints `RESULT all checks hold`, with 44 of 44 explain fixtures;
  - `scripts/check-manual.py` reports 24 of 24 examples.
- **CI on 45c94b5:** `check` and `gdt-comparison` both succeed.
- **Deviations:** none in the merge. One stale doc comment at the top of `cli/src/explain.rs` still names `check_preread_outcome`; the code calls `check_bytes_outcome`. It is a comment-only correction, listed for RLT-004c implementation.

RLT-004c (theorem library and `rlt deps`) review: docs/RLT-004c-REVIEW.md.

## Merge record, 2026-11-11 (RLT-004c)

- **Gate before merge:**
  - PR #12 head was exactly 6698345, and CI `check` and `gdt-comparison` were green on it.
  - The final audit corrections were present: the first-parent identity gate, refused-citation tracing, over-limit counts, assumption dedup.
  - Kernel, rules, axiom sets, definitions, conventions, reading points, statement records and earlier proofs were unchanged. The only existing test files that changed were the two `rlt explain` golden files, which gain the approved C5 `library` line.
- **Branch protection (recorded, not changed):**
  - `GET repos/thinkthoughts/rlt/branches/main/protection` returns 404 "Branch not protected"; `branches/main` reports `protected: false`.
  - There are no rulesets (`rulesets` and `rules/branches/main` are empty).
  - Main does not currently prevent force-pushes or history rewriting.
  - The append-only identity rule relies on main's history being stable. Enabling protection (no force-push, no deletion) is recommended and awaits approval.
- **PR #12 merged:** marked ready, then merged as merge commit **c42729b**, with parents 45c94b5 and 6698345. The merge tree 9e459bed equals 6698345's tree.
- **Fresh clone of main at c42729b** (shallow clone, deepened):
  - `cargo test --release` passes 257 of 257;
  - `scripts/verify.sh` prints `RESULT all checks hold` (library 68 of 68 current);
  - `scripts/check-manual.py` reports 25 of 25 examples.
- **CI on c42729b:** `check` and `gdt-comparison` both succeed.
- **Identity gate on the resulting first-parent history:**
  - main's first-parent history holds one version of `lib/identities.tsv` (c42729b); the branch's earlier drafts are excluded;
  - the gate passes on the unchanged file;
  - editing a fingerprint in the working tree is refused, naming c42729b.
- **Deviations:** none.

RLT-005 (published-mathematics demonstration) candidate review: docs/RLT-005-REVIEW.md.

## Merge record, 2026-11-11 (RLT-005)

- **Gate before merge:**
  - PR #13 head was exactly 3cd14be, and CI `check` and `gdt-comparison` were green on it (push and pull_request runs).
  - Both independent audit reports are resolved (docs/RLT-005-REPORT.md §6). Spot re-checks of the two archive should-fix findings at 3cd14be: a single gzip whose header does not name the member gives ERROR; a planted cache symlink gives ERROR and nothing is written through it.
  - Kernel (`kernel/`), inference rules, axiom sets (`spec/`), `syntax/` and the reading points are unchanged since c42729b. No earlier `.rlt` file changed. The only existing test files that changed are `tests/expected/validate.out` (31 sources, `archived 1`) and `tests/explain/expected.tsv` (8 rows appended, none removed).
  - All 19 files in `tests/rlt005/accept` ACCEPT as unconditional theorems under ROBINSON-Q; `lean/RLT005Direct.lean` (11 theorems) and `lean/RLT005Readings.lean` (22) compile with standard axioms only, no errors or warnings.
- **PR #13 merged:** marked ready, then merged as merge commit **9c8a66d**, parents c42729b and 3cd14be. The merge tree 36f4e31 equals 3cd14be's tree.
- **Fresh clone of main at 9c8a66d:**
  - `cargo test --release` passes 262 of 262;
  - `scripts/verify.sh` prints `RESULT all checks hold` (library 87 of 87 current; the eight archived-source tamper cases refused);
  - `scripts/check-manual.py` reports 29 of 29 examples.
- **The three commands for `matchable.not.8`** (fresh clone): `rlt check` ACCEPT, exit 0; `rlt explain statements/MATCHABLE-NOT-8.yaml` exit 0 (kernel ACCEPT · source registry verified, pinned · semantic partial, current · physical not applicable); `rlt deps matchable.not.8` exit 0, 19 theorems, 29 of 64 cited-file checks, chain 4 of 8.
- **Identity registry:** main's first-parent history now holds two versions of `lib/identities.tsv` (c42729b, 9c8a66d); every line of both is kept (87 identities); 19 lines appended since c42729b, none changed or removed.
- **CI on 9c8a66d:** `check` and `gdt-comparison` both succeed.
- **Three results kept apart**, as accepted:
  - *RLT proof:* the explicit-partner formula follows from Robinson's Q (kernel).
  - *Lean correspondence theorem:* over ℕ that formula is the paper's "8 is not matchable" (`explicit_iff`, `matchable'_eight_iff`), a Lean result attributed to Lean.
  - *Source-fidelity assessment:* the quoted definitions and claim are verbatim (up to line breaks) in the hash-pinned arXiv v2 e-print; semantic status `partial`, with the documented differences (bridge in Lean; positive divisors because of Q; later multiples of 4 out of scope).
- **Branch protection:** unchanged (main unprotected, no rulesets). The recommendation in docs/RLT-005-BRANCH-PROTECTION.md awaits separate approval.
- **Deviations:** none.

RLT-006 (H₄₅ residue certificates) feasibility review: docs/RLT-006-REVIEW.md.

## Merge record, 2026-11-12 (RLT-006)

- **Final independent re-audit of exactly 5522947, before the merge.**
  - **Verdict:** no unresolved substantive issue.
  - **The two formerly blocking findings are fixed.**
    - The residue-profile check compares integers and rejects the altered q, s, omits flag, dropped row, occupied count and occupied class.
    - CI's `check` runs on `ubuntu-24.04` with pdftotext 24.02.0, and the transcription and source checks ran and passed in the push and pull_request runs of 5522947.
  - **Also confirmed:**
    - all 630 certificates regenerate byte for byte and are each accepted;
    - each of the 14 per-prime theorems cites the intended 45 certificates, checked independently from the PDF data;
    - missing, modified and substituted certificates are refused by the manifest, `rlt check`, `rlt deps` and the library verifier;
    - the identity registry is append-only;
    - 472 earlier proof reports are byte-identical (only `rlt validate` differs, intentionally);
    - the Lean correspondence is symbol for symbol, and the vendored lean-bgp212 blocks are identical at 7898eb4;
    - the transcription matches the rendered PDF;
    - the `grep_q` replacement adds no false-positive or false-negative path.
  - **The committed results file** came from 5109d68. The only changes from 5109d68 to 5522947 are the report and that file.
- **PR #14 merged:** marked ready, then merged as merge commit **e5850d9**, with parents 9c8a66d and 5522947. The merge tree 07434d6 equals 5522947's tree.
- **Fresh clone of main at e5850d9** (the clone has no generated certificates; they are generated during the run):
  - `cargo test --release` passes 270 of 270 in 757 s, generating and manifest-checking the certificates;
  - `scripts/verify.sh` prints `RESULT all checks hold`, exit 0, 1,580 s:
    - 630 of 630 certificates and 14 of 14 per-prime theorems at 45 of 64 checks;
    - library 101 of 101;
    - identity registry kept across 3 committed versions;
  - `scripts/check-manual.py` reports 33 of 33.
- **The three commands for `admissible.h45.omits.43_3`** (fresh clone):
  - `rlt check` ACCEPT;
  - `rlt explain statements/H45-OMITS-43.yaml` exit 0 (semantic partial, physical not applicable);
  - `rlt deps` exit 0, 45 of 64 cited-file checks.
- **CI on e5850d9:** `check` and `gdt-comparison` both succeed.
- **Results kept apart, as accepted:**
  - *RLT proofs:* obligation (b) of Lemma 12.1, the 14 omissions from Robinson's Q.
  - *Lean theorems:* the 14 statements literally; obligations (a), (c) and (d); the whole lemma by the paper's route; lean-bgp212's route re-checked.
  - *Computed evidence:* the residue profile.
  - *Source fidelity:* the reviewed transcription of the hash-pinned PDF, with page, section, tool version and extraction hashes.
- **Remaining limitations:**
  1. RLT does not verify the complete Lemma 12.1: obligations (a), (c) and (d) are Lean results.
  2. The residue profile is computed evidence; RLT proves only the 14 tabled classes omitted.
  3. A single RLT theorem over all certificates exceeds the 64-check citation limit. That limit is unchanged by request.
  4. CI depends on two upstream inputs, the PDF URL and lean-bgp212's default branch. Both fail closed.
  5. The residue-profile arithmetic check is weaker than its name: some altered tables pass it, though the regeneration stage catches them.
  6. INT and TERM in verify.sh clean up but do not stop the run.
  7. Run times have grown: CI `check` takes about 27 min (about 4 min before RLT-006); local `cargo test` about 13 min; verify.sh about 27 min.

  Items 4–6 are hardening tasks H1–H3 in docs/RLT-007-REVIEW.md.
- **Not done, as instructed:** no tag, no release, no deployment, no contact with the paper's authors. Branch protection is unchanged; the recommendation in docs/RLT-005-BRANCH-PROTECTION.md awaits approval.
- **Deviations:** none.

RLT-007 architecture review (the complete Lemma 12.1 in Robinson's Q; verification turnaround): docs/RLT-007-REVIEW.md.

## Merge record, 2026-11-12 (RLT-007a)

- **Approval:** merge exactly 1be8480 (approved 2026-11-12 after the final independent audit reported no unresolved substantive issue).
- **Pre-merge checks:**
  - PR #15 head was 1be8480;
  - `check` and `gdt-comparison` were green on its push and pull_request runs;
  - every security correction from the audit rounds was included (S1–S6, N1–N2, the cached-checkout findings 1–6 and the delta findings);
  - no file under `kernel/`, `syntax/`, `cli/src`, `spec/`, `lib/`, `statements/`, `tests/` or `lean/` differed from e5850d9, so the kernel, the 20 rules plus `cite`, ROBINSON-Q, the limits and every existing proof were unchanged.
- **PR #15 merged:** marked ready, then merged as merge commit **1561668** (156166809569ba3957331aad44c624d7ef064426), with parents e5850d9 and 1be8480. The merge tree 344f5b4 equals 1be8480's tree.
- **Fresh clone of main at 1561668,** with sources fetched by `scripts/fetch-sources.py` into an empty directory:
  - `cargo test --release`: 271 passed, 0 failed, 826 s;
  - `scripts/verify.sh`: `RESULT all checks hold`, exit 0, 1,552 s wall (another verification run shared the 2-processor machine);
  - `scripts/check-manual.py`: 33 of 33 examples match.
- **CI on 1561668:** `check` (13 min 51 s) and `gdt-comparison` both succeed; `source-provenance` was skipped, as it runs on schedule or dispatch only.
- **Measurements** (2 processors, same machine, against e5850d9):
  - verify.sh wall time 1,622 s → about 1,030 s;
  - CPU time 1,808 s → about 1,500 s;
  - generated files unchanged at 57.5 MB;
  - CI `check` 27 min → 10–14 min.
  - Details are in docs/RLT-007a-REPORT.md §3. Every redundant verification pass is kept.
- **Open hardening item (accepted, fail-closed; for later remediation):**
  - *What:* `scripts/pinned_read.py` copies a cached checkout's object files when hard links fail across filesystems, with no size cap or deadline on the copy.
  - *Failure mode:* a tampered cache with a very large sparse `pack/pack-<40 hex>.pack` can fill the temporary filesystem. The read then fails: `check_excerpts.py` reports an error and exits 1, and verify.sh reports the vendored blocks as differing. It never reaches a false pass.
  - *Proposed fix:* stage only the `.idx` and `.pack` pair needed, under a total size bound; wrap `OSError` as `PinnedReadError`; apply the 120 s deadline to staging.
  - *Proposed test:* a checkout on a separate filesystem (for example `/dev/shm`) holding a sparse 3 GB `pack-*.pack` with no index. `pinned_read.py` must refuse quickly, with a size reason and bounded temporary space, and the real files must still read.
  - Two related audit notes remain: staging time grows with the number of files, and the per-object deadlines (up to 3 × 120 s) are long but bounded.
- **Not done, as instructed:** no tag, no release, no deployment, no contact with the paper's authors; branch protection and repository settings unchanged.

## Merge record, 2026-11-12 (RLT-007b stage 1)

- **Approval:** merge exactly a0187ae.
- **Pre-merge checks:**
  - PR #17 head was a0187ae;
  - `check` and `gdt-comparison` were green on its push and pull_request runs;
  - every audit correction was included: the self-test isolates the length check (11 cases), the lemma text is checked in both the `extracted` and `text` fields with whole-number matching, `--check` requires exact file sets, the accepted-file count is checked, deps is compared with check, the SX05 reason is pinned, and the wording is fixed;
  - nothing changed under `kernel/`, `syntax/`, `cli/src`, `spec/`, `conventions/` or the Cargo files;
  - every earlier proof, record and library entry is unchanged; `lib/identities.tsv` grew by exactly 5 appended lines, with the old file a byte-identical prefix.
- **PR #17 merged:** marked ready, then merged as merge commit **eb0e2ef** (eb0e2eff06079060a4f3ef1989e7bbe913670e5d), with parents 1561668 and a0187ae. The merge tree f8750ee equals a0187ae's tree.
- **Fresh clone of main at eb0e2ef,** with sources fetched by `scripts/fetch-sources.py` into an empty directory:
  - `cargo test --release`: 278 passed, 0 failed;
  - `scripts/verify.sh`: `RESULT all checks hold`, exit 0, 1,124 s wall (the machine was shared with an audit);
  - `scripts/check-manual.py`: 33 of 33.
- **The 49 new theorems, from that run:**
  - kernel accepted: 45 of 45 (44 per-element theorems and bounds) and 4 of 4 group summaries (11 of 64 checks each);
  - 7 of 7 rejected cases refused;
  - generator self-test 11 of 11; `--check` 64 outputs, 0 differ.
- **Lean:** `RLT007S1.lean` has 50 statements, each printed, standard axioms only.
- **Library:** 106 of 106 entries current and accepted, including `admissible.h45.distinct.g0`–`g3` and `admissible.h45.bounds`.
- **CI on eb0e2ef:** `check` and `gdt-comparison` both succeed.
- **Not done, as instructed:** no tag, no release, no deployment, no contact with the paper's authors; branch protection and repository settings unchanged.

## Merge record, 2026-11-12 (RLT-007b stage 2a)

- **Approval:** merge exactly 57c949e.
- **Pre-merge checks:**
  - PR #18 head was 57c949e;
  - `check` and `gdt-comparison` were green on its push and pull_request runs;
  - both independent audits' corrections were included (`listed_1` … `listed_6`, the by-name Lean axiom gate, SY09 and the rebuilt SY04/SY07/SY08, exact statement reconstruction in `cli/tests/rlt007s2a.rs`, argument checks and the approved-interval check in the generator);
  - nothing changed under `kernel/`, `syntax/`, `spec/`, `conventions/` or `sources/`, nor in any earlier proof file;
  - the 19 statements match the six approved intervals and Prime_B exactly (`every_statement_is_exactly_the_approved_one`).
- **PR #18 merged:** marked ready, then merged as merge commit **f37bcfc** (f37bcfce04809f719307a4fabacf46fd594ac0be), with parents eb0e2ef and 57c949e. The merge tree ae96f24 equals 57c949e's tree.
- **Fresh clone of main at f37bcfc,** with the pinned sources already fetched:
  - `cargo test --release`: 287 passed, 0 failed;
  - `scripts/verify.sh`: `RESULT all checks hold`, exit 0, 28 min 9 s wall (the machine was shared with two audits);
  - `scripts/check-manual.py`: 33 of 33.
- **The 19 new theorems, from that run:** kernel accepted 19 of 19; `rlt deps` 7 of 7 with the pinned budgets (C: 18 of 64 checks, chain 2); 9 of 9 rejected cases refused; generator self-test 8 of 8, `--check` 37 outputs, 0 differ.
- **Lean:** `RLT007S2a.lean` 33 statements and `RLT007S2Design.lean` 20, each printed by name, standard axioms only; the 14 stage-2a references in records checked.
- **Library:** 125 of 125 entries current and accepted, including the 19 stage-2a entries; the identity registry keeps every line of 5 committed versions.
- **CI on f37bcfc:** `check` and `gdt-comparison` both succeed.
- **Direction of the result:** RLT establishes exhaustiveness (every Prime_B number below 213 is listed); the converse is Lean evidence only. Stage 2b is deferred as a separately documented extension.
- **Not done, as instructed:** no tag, no release, no deployment, no contact with the paper's authors; branch protection and repository settings unchanged.

## Merge record, 2026-11-12 (RLT-007b stage 3)

- **Approval:** merge exactly aa7db42 (after 7070cff was held: it carried an unreviewed run log, restored in aa7db42, which also adds the SR07 clarification).
- **Pre-merge checks:** PR #19 head aa7db42, mergeable; `check` and `gdt-comparison` green on its push and pull_request runs; the diff limited to the stage-3 files, the test and library counts, verify.sh and documentation; nothing under `kernel/`, `syntax/`, `cli/src`, `spec/`, `conventions/`, `sources/` or `verification/` changed.
- **PR #19 merged:** marked ready, then merged as merge commit **4e4ca04** (4e4ca04c3feac4e24741dc3657932def81fc3441), with parents f37bcfc and aa7db42. The merge tree e98d313 equals aa7db42's tree.
- **Fresh clone of main at 4e4ca04:**
  - `cargo test --release`: 292 passed, 0 failed;
  - `scripts/verify.sh`: `RESULT all checks hold`, exit 0, 30 min 25 s wall (the machine was shared with stage-4 work);
  - `scripts/check-manual.py`: 33 of 33.
- **The tail theorem, from that run:** TAIL.rlt accepted, unconditional under ROBINSON-Q, 1,750 steps; its statement and record rebuilt from the transcription; 9 of 9 rejected cases; self-test 6 of 6; `--check` 13 outputs, 0 differ.
- **Record and library:** `TAIL-OMITS-H45` exits 0 (current, semantic partial); `admissible.h45.omits.tail_1` among 126 of 126 entries current and accepted; the identity registry keeps every line of 6 committed versions. `RLT007S3.lean`: 10 statements, each printed by name, standard axioms only; the 4 references checked.
- **CI on 4e4ca04:** `check` and `gdt-comparison` both succeed.
- **SR07 (kept):** the bound 211 gives a true, stronger theorem; SR07 is refused because the committed proof establishes the approved statement (212 < p), and a changed specification is never accepted on the strength of that proof.
- **Not done, as instructed:** no tag, no release, no deployment, no contact with the paper's authors; branch protection and repository settings unchanged.

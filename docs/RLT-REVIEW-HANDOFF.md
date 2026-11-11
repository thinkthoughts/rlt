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

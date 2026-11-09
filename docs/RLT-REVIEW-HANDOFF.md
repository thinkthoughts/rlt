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
| M3e | **draft PR #9** (branch `rlt-003-m3e`) | the modulo-30 classification, E-T1; see docs/RLT-003-M3e.md |

**Not yet done, and held for the review:**
- merging M3e;
- the `backed by` links (designed in the M3e review §9);
- any M3f;
- release and deployment.

**Resume on Friday:**
1. Read docs/RLT-003-M3e.md §6 (audit) and the PR #9 CI status.
2. Decide whether to merge M3e.
3. Start the broader review from the design principles below.

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

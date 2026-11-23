# RLT-CSTAR-008 · Independent acceptance and integration — specification

Status: executed 2026-11-23. Results: `RLT-CSTAR-008-VERIFICATION.md`; decisions: `RLT-CSTAR-008-ACCEPTANCE.md`; register: `RLT-CSTAR-FINDINGS.md`; scientific questions: `RLT-CSTAR-008-SCIENTIFIC-QUESTIONS.md`.

## 1. Inputs and immutable baseline

| Item | Value |
|---|---|
| `RLT-CSTAR-007-review.tar.gz` | SHA-256 `522cf759b4e665de5848593ca2101f90c7a50803467cc4fc4d23bb0caf6bbae2` (unchanged; its `SHA256SUMS` verified) |
| `RLT-CSTAR-002-review.tar.gz` | SHA-256 `faa3c09735a2f05008cc8757bbf53f4eb065d638347575ac8cc9bd947395302d` (unchanged) |
| Pins | ucla-roms 0.16.0 `f83d2ef6`; C-Star `0991b445`; roms-tools `ede4fe62`; RLT `6128c97`; MARBL `0e831c4d` |
| Branch heads delivered by 007 | listed in that archive's `BRANCHES.txt`; none moved by 008 except by new commits added on top (§4) |

## 2. Phase 1 · Independent replay

Fresh clones of the public forks `thinkthoughts/{rlt,C-Star,roms-tools,ucla-roms}`; research branches imported only from the 007 archive's git bundles; worktrees per branch. Previous logs serve only as comparison targets, never as results. Required replays, each with exact commands, commit hashes, outputs and SHA-256:

1. ROMS-Tools partition and join against ucla-roms `partit` and `ncjoin`, built from the fresh pin checkout, on the archived grid and on a newly generated grid of another size.
2. C-Star's coarse-grid checks against the verbatim `init_coarse` lines, over every `(llm, np_xi)` in a stated range, plus both entry points (Rule L, Forge).
3. ROMS-Tools units contract: test suite, mixed-prefix probe, and adversarial strings absent from the test suite.
4. Tonnes-CO₂ constant from IUPAC atomic weights in exact rational arithmetic, and end to end through `compute_cdr_metrics`.
5. Stretching function: ucla-roms `CSF` re-extracted and compiled from both fresh checkouts, against ROMS-Tools and a 50-digit reference. Additionally, full ucla-roms builds and runs (its own analytical test configurations) at the pin and with the change.
6. RLT certificates checked by a kernel built from scratch in the fresh clone, with one-digit tampering of every accepted goal.
7. Lean companions compiled with the pinned toolchain `leanprover/lean4:v4.34.0-rc2`; axiom audit of every theorem.

## 3. Phase 2 · Review criteria

Each commit is classified ACCEPT, REVISE (accepted once a stated revision lands, made in this round as a separate commit) or HOLD (needs scientific approval or a maintainer decision), against: mathematical correctness, scientific meaning, source correspondence, numerical accuracy, backward compatibility, performance and resources, test coverage, dependency compatibility, reproducibility. The compatibility of C-Star's accepted tilings with *released* ROMS-Tools versions is tested directly against the PyPI release C-Star requires.

## 4. Phases 3–4 · Scientific hold and local integration

- F-009 stays on `research/rlt-cstar-005-dic-uptake` and on a hold branch rebased on the integration branch; it is never part of an integration branch. F-018 is examined with an extended closed-form model.
- Integration branches `integration/rlt-cstar-008` in roms-tools, C-Star and ucla-roms start at the pins; accepted commits are cherry-picked one by one with `-x`, so each records its reviewed source commit.
- Gates on the combined result: full ROMS-Tools suite; C-Star unit suite with the ROMS-Tools integration branch and with released roms-tools 5.4.0 (C-Star's CI environment), compared against the pinned C-Star under the same 5.4.0; the cross-repository partition comparison; release ordering recorded.
- All branches stay local: no merge into `main`, no push, no pull request, no upstream contact, no release.

## 5. Phase 5 · Deliverables

`RLT-CSTAR-008-SPEC.md`, `RLT-CSTAR-008-ACCEPTANCE.md` (decisions, integration, contribution sequence, merge blockers), `RLT-CSTAR-008-VERIFICATION.md` (commands, hashes, outputs), `RLT-CSTAR-008-SCIENTIFIC-QUESTIONS.md`, updated `RLT-CSTAR-FINDINGS.md`, an archive with branch and commit manifest, bundles and patch series of the accepted changes and of the held ones (separately), replay scripts and outputs, and `SHA256SUMS`.

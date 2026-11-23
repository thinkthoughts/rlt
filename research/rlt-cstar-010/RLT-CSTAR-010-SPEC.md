# RLT-CSTAR-010 — Specification

Date: 2026-11-23. Continues from the published RLT-CSTAR-009 branches and the 009 verification, CI and findings records.

## Objective

Resolve the CI integration defect F-027, investigate F-028, measure the UCLA-ROMS output changes independently before any reference hash is refreshed, and prepare the accepted work for upstream technical review.

## Authorization

- Local development and verification: authorized.
- New pushes, merges, releases, upstream pull requests, contact with [C]Worthy: each needs separate approval. 010 ends at a publication plan.

## Work items

1. **F-027 repair (approved: A + B)**, two separate commits on the C-Star research line (on top of published `b686945d`):
   - A: the Forge generated-values fixture runs on a 1×1 partitioning, keeps a stale stored `np_xi = 2`, and asserts that Forge derives `np_xi = 1`. The test keeps its purpose.
   - B: an independent regression test that Forge refuses the 2×2 coarse-forcing configuration with Rule L's diagnostic, checked against ucla-roms `init_coarse`.
   - Investigate whether a stale snapshot delays the refusal until after input generation.
   - Gates, recorded separately for A and B: the affected Forge integration tests, the C-Star unit suite, and the relevant regression gates.
2. **F-028**: a minimal correction to `containerized_ci.yml` that validates the base commit before checkout, consistent with the `prepare` job's logic. Test empty, all-zero, valid and invalid values. Leave the container-registry guards untouched.
3. **UCLA-ROMS numerical comparison**: measure the 009 output changes against the correct baseline, separating measured changes from expectations carried over from earlier local tests.
4. **Reference verification**: run `tests/update_references.py --run 45245982301 --dry-run`, inspect it against the 17 changed cases for GNU and Intel, rebase onto fork `main` (#475, #476) locally, rerun, and check whether the same 17 cases change. Commit no new hashes until the comparison is complete and documented.
5. **Holds and pins**: F-009 and the C-Star partition-support guard stay isolated. Carbon-uptake definitions are unchanged. Source pins, RLT certificates, Lean proofs and the trusted kernel are unchanged.

## Acceptance criteria

| Item | RESOLVED when |
|---|---|
| F-027 | the 4 Tier 1 errors are gone at A; B fails with Rule L disabled and passes with it; the unit suite shows no regressions against `b686945d`; the 2×2/1×1 verdicts match the compiled `init_coarse` lines |
| F-028 | the harness gives the intended outcome for every input value on both jobs; the unmodified workflow fails the same harness; the diff touches no registry, login, push, delete or permission line |
| References | every changed variable is measured against a baseline that reproduces CI; the size and pattern of each change is consistent with the intended CSF change; the rebased branch changes the same 17 cases |

Otherwise REVISE (a correctable gap remains) or HOLD (a decision outside engineering evidence).

## Deliverables

`RLT-CSTAR-010-SPEC.md`, `RLT-CSTAR-010-VERIFICATION.md`, `RLT-CSTAR-010-NUMERICAL-COMPARISON.md`, an updated `RLT-CSTAR-FINDINGS.md`, local repair commits and verification logs, a publication plan with exact commits and branches, and an evidence archive with a SHA-256 manifest.

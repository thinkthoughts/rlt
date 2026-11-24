# RLT-CSTAR-010 — Publication and CI acceptance record

Date: 2026-11-23 (pushes 23:2x UTC). Approval: the six core branches of RLT-CSTAR-010-PUBLICATION-PLAN.md, create-only; F-029 local; F-009 and the partition-support guard on HOLD.

## 1. Pre-push checks

| Check | Result |
|---|---|
| Six local heads equal the approved commits | 6/6 |
| Remote branch names absent | 0 of the 010 names present on c-star, ucla-roms, rlt |
| CI write-path audit (`audit_ci_writes.py`) | 0 unguarded write paths on c-star `ci-010`, ucla-roms `ci-010`, and on every integration branch |
| Guard lines in ucla-roms `containerized_ci.yml` vs the 009 CI branch `e29c7d58` | none changed (only the trigger branch name and the F-028 step lines) |

## 2. Publication

All six pushes created new branches (`push010.log`). Read back with `git ls-remote` (`remote_verify_010.txt`):

| # | Fork / branch | Remote head |
|---|---|---|
| 1 | c-star `rlt-cstar/integration-010` | `5d353014` |
| 2 | ucla-roms `rlt-cstar/integration-010` | `c380aab7` |
| 3 | ucla-roms `rlt-cstar/f028-base-sha` | `79b85c8a` |
| 4 | rlt `research/rlt-cstar-010` | `a81bce2f` |
| 5 | c-star `rlt-cstar/ci-010` | `a842c9cd` |
| 6 | ucla-roms `rlt-cstar/ci-010` | `c5ea8ee4` |

- `main` is unchanged on all four forks (roms-tools `ede4fe62`, c-star `0991b445`, ucla-roms `bf7b5f4d`, rlt `6128c974`), and the 009 branches are unchanged.
- No F-029, `hold/` or HOLD research branch is remote.
- CWorthy-ocean repositories carry 0 refs from this work.

## 3. CI runs

| Run | Commit | Conclusion | Detail |
|---|---|---|---|
| [c-star integration_tests 45288503456](https://github.com/thinkthoughts/c-star/actions/runs/45288503456) | `a842c9cd` | **success** | Forge Ubuntu 100 passed (009: 94 + 4 errors); Forge macOS 100 passed (009: runner not acquired); e2e 92 passed, 7 skipped (zsh absent) |
| [c-star unit_tests 45288503417](https://github.com/thinkthoughts/c-star/actions/runs/45288503417) | `a842c9cd` | cancelled (macOS time limit) | Ubuntu 3.12 / 3.13 / 3.14: 10025 passed, 61 skipped, 1 xfailed each; macOS 3.14: every test reached `[100%]` with 0 FAILED / 0 ERROR lines, then the job hit `timeout-minutes: 30` during the summary (tests ran 21 min) |
| [ucla-roms Containerized CI 45288508832](https://github.com/thinkthoughts/ucla-roms/actions/runs/45288508832) | `c5ea8ee4` | **success** | prepare: `Plan: build_gnu=false build_ifx=false can_push=false`; build_gnu, build_ifx, cleanup skipped; compile gnu and ifx success; pytest_gnu 126 passed, 2 skipped; pytest_ifx 126 passed, 2 skipped; 0 hash mismatches; base-comparison steps idle |
| [rlt rlt-cstar 45288488755](https://github.com/thinkthoughts/rlt/actions/runs/45288488755) | `a81bce2f` | **success** | kernel unchanged; 18 certificates, ACCEPT 17 / MALFORMED 1; 17 tamper controls rejected; Lean |
| [rlt rlt 45288488662](https://github.com/thinkthoughts/rlt/actions/runs/45288488662) | `a81bce2f` | **success** | check (fmt, clippy, `verify.sh`) success; gdt-comparison success; source-provenance skipped (main only) |

## 4. Required engineering gates

| Gate | Verdict | Evidence |
|---|---|---|
| 1. C-Star Forge integration with F-027 corrected | **PASS** | 100 passed on Ubuntu and macOS. `test_forge_coarse_tiling.py` (2 tests) and all 4 `test_forge_generated_values.py` tests pass |
| 2. UCLA-ROMS gnu and ifx against the refreshed references | **PASS** | results artifacts: 477/477 hashed variables equal to the committed references on both compilers, and equal to run 45245982301's; input sentinel identical (26 files) |
| 3. Container-registry guards | **PASS** | plan `can_push=false`; no build, push or delete job ran; audit 0 unguarded |
| 4. Failures, skips, limitations | recorded | §5 |
| 5. Numerical-comparison evidence | preserved | the 010 evidence archive (`06cd6e4d…`) keeps the local full-field comparison; the CI hashes are independent confirmation |

**Reference update: accepted.** The refreshed hashes reproduce in two independent CI runs (45245982301 on the pre-rebase branch, 45288508832 on the rebased branch) for both compilers. They also agree with the local measurement, where the output change is roundoff-level. The CI confirmation is what makes the update final, as the review required.

## 5. Failures, skipped gates, limitations

1. c-star unit macOS: job time limit (30 min) after the test run completed with 0 failures. The summary line was never printed, so the macOS totals are inferred from per-test lines. Environmental (runner speed), not a test failure.
2. c-star e2e: 7 skipped (no zsh on the runner), as in upstream.
3. ucla-roms: 2 skipped per compiler, as in upstream.
4. F-028's new step branches did not execute in CI: with zero mismatches those steps stay idle. The zero-SHA, invalid and unknown paths are verified by the harness only.
5. roms-tools: no new CI in 010 (no change). Its Copernicus jobs remain credential-limited (009).
6. Codecov uploads skipped by design in forks.
7. GitHub's Node.js 20 deprecation warnings on several actions (informational).

# RLT-CSTAR-010 — Proposed publication plan (awaiting approval)

Nothing below has been pushed. Every push creates a new branch (`--force-with-lease=<ref>:`, create-only). `main` and the published 009 branches stay as they are. The HOLD work stays local.

## 1. Branches proposed for the thinkthoughts forks

| # | Fork | Branch | Commit | Content | Status |
|---|---|---|---|---|---|
| 1 | c-star | `rlt-cstar/integration-010` | `5d35301` | 009 head `b686945` + A `fbd68e8` + B `5d35301` | F-027 RESOLVED; approved scope |
| 2 | ucla-roms | `rlt-cstar/integration-010` | `c380aab7` | 009 CSF commits rebased onto fork `main` `bf7b5f4d` (`9febb1e0`, `6845bb4c`) + reference refresh `c380aab7` | references justified (NUMERICAL-COMPARISON) |
| 3 | ucla-roms | `rlt-cstar/f028-base-sha` | `79b85c8a` | F-028 fix on fork `main` | F-028 RESOLVED; upstream candidate |
| 4 | rlt | `research/rlt-cstar-010` | see §3 | 010 records, findings, tools; kernel unchanged | research record |
| 5 | c-star | `rlt-cstar/ci-010` | `a842c9c` | #1 + fork-only CI triggers (Codecov guarded) | CI only, never upstream |
| 6 | ucla-roms | `rlt-cstar/ci-010` | `c5ea8ee4` | #2 + fork-only CI (`d30a3336`, registry guards) + F-028 (`c5ea8ee4`) | CI only, never upstream |
| — | c-star | `rlt-cstar/f029-derived-rule-l` | `134183a` | #1 + F-029 correction | **proposed change; needs your review of the change before publication** |

- **Order:** 1–4, then 5–6.
- **Write-path audit:** 0 unguarded write paths on both CI branches.
- **roms-tools:** nothing new in 010; its 009 branches stand.

## 2. Expected CI outcome (to verify after the push)

| Run | Expectation |
|---|---|
| c-star integration (ci-010) | Forge Ubuntu: 100 passed, 0 errors (009: 94 + 4 errors); e2e as in 009 |
| c-star unit (ci-010) | as in 009 (10025 passed per Ubuntu Python) |
| ucla-roms containerized (ci-010) | pytest gnu and ifx: 0 hash mismatches (the references came from CI run 45245982301, and the rebased outputs are bit-identical locally); with zero mismatches the base-comparison steps stay idle |

A mismatch in ucla-roms ci-010 would mean the CI environment moved since run 45245982301. Per the README, that is investigated before any further reference change.

## 3. Push commands (for approval; not executed)

```bash
# 1–3
git -C /home/claude/c-star push --force-with-lease=refs/heads/rlt-cstar/integration-010: origin 5d35301:refs/heads/rlt-cstar/integration-010
git -C /home/claude/ucla-roms push --force-with-lease=refs/heads/rlt-cstar/integration-010: origin c380aab7:refs/heads/rlt-cstar/integration-010
git -C /home/claude/ucla-roms push --force-with-lease=refs/heads/rlt-cstar/f028-base-sha: origin 79b85c8a:refs/heads/rlt-cstar/f028-base-sha
# 4 (commit given in the final report)
git -C /home/claude/rlt push --force-with-lease=refs/heads/research/rlt-cstar-010: origin <rlt-010>:refs/heads/research/rlt-cstar-010
# 5–6
git -C /home/claude/c-star push --force-with-lease=refs/heads/rlt-cstar/ci-010: origin a842c9c:refs/heads/rlt-cstar/ci-010
git -C /home/claude/ucla-roms push --force-with-lease=refs/heads/rlt-cstar/ci-010: origin c5ea8ee4:refs/heads/rlt-cstar/ci-010
```

## 4. Upstream review order (drafts only; nothing submitted)

1. ucla-roms F-028 (`79b85c8a`): CI-only and independent of everything else.
2. ucla-roms stable CSF + references (`9febb1e0`, `6845bb4c`, `c380aab7`): the PR body cites NUMERICAL-COMPARISON. The references are to be regenerated from the PR's own CI run if upstream `main` moves.
3. ROMS-Tools contributions (009 drafts).
4. C-Star coarse-grid validation (`f4ba79ff` line + A + B, and F-029 if accepted), after the ucla-roms CSF release question is settled.

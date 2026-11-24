# RLT-CSTAR-011 — Acceptance and proposed submission order

Date: 2026-11-23. Everything here is local and unpushed. No merge, no upstream PR, no contact with [C]Worthy; `main` untouched on every fork.

## 1. Verdicts

| Item | Verdict | Basis |
|---|---|---|
| F-029 revision (`1f56f37`, C-Star) | **ACCEPT** (independent reviewer, two passes) | one tile check on `generated_tile_settings`; oracle 0 wrong (author, 12 scenarios; reviewer, 21); negative controls fail as expected; non-regression 4,000 inputs byte-identical; Tier 1 101 passed; unit 10,029 passed, with the same 4 local-environment failures as at the parent |
| ROMS-Tools split (5 PR branches) | **READY for fork CI** | each branch on its intended parent; together they reproduce the accepted integration tree file for file; targeted tests and pre-commit pass on each |
| CI branches (5 roms-tools, 1 c-star) | **AUDITED** | each is its PR branch plus one fork-only workflow commit; 0 unguarded write paths; workflow diff vs the 009/010 CI branches is the trigger name only |
| PR drafts | **9 drafts** (8 refined + CS-2) | AI disclosure, formal and numerical evidence, CI status and the open scientific questions in each |

## 2. F-029: what changed between review rounds

- **Round 1 (`134183a`): REVISE.** The tile check ran on both the snapshot and the overlay, which caused false refusals and duplicate messages.
- **Revision (`19cdfff`), then amended to `1f56f37`:**
  - one check, on the overlay;
  - `TILE_SIZE_SECTIONS` as the single list of sections;
  - the wizard's load-time check aligned with `cstar blueprint check`;
  - docstring and commit message corrected (reviewer's must-fix items).
- **A unit-test stub broke at `19cdfff` and the reviewer missed it.** The chunked unit run here caught it: `test_hash_pin_tier.py` stubbed `validate_run_time_sections` without the new keyword. It is fixed in `1f56f37`, and the reviewer then searched the whole test tree and found no other such stub.

Oracle, wrong verdicts or messages against `configure_build` after real generation:

| Version | Author's 12 scenarios | Reviewer's 21 scenarios |
|---|---|---|
| parent `5d35301` | 7 | 12 |
| `134183a` | 4 | — |
| `1f56f37` | 0 | 0 (on `19cdfff`) |

Known limits, all documented in the code:
- **`auto` coarse-grid mode:** the grid is chosen during generation.
- **Grid files:** the grid sizes come from the file.
- **An existing roms-tools/Forge case:** mismatched `auto` choices between physics and BGC fail after generation (candidate follow-up).

## 3. Branches proposed for publication (create-only; approval required)

| # | Fork | Branch | Commit | Parent (verified) |
|---|---|---|---|---|
| 1 | roms-tools | `rlt-cstar/pr-tiling` | `a45d6be3` | `main` `ede4fe62` |
| 2 | roms-tools | `rlt-cstar/pr-co2-scale` | `3500f1dc` | `main` `ede4fe62` |
| 3 | roms-tools | `rlt-cstar/pr-cdr-units` | `4df8045e` | #2 `3500f1dc` |
| 4 | roms-tools | `rlt-cstar/pr-zero-theta` | `e9728d2b` | `main` `ede4fe62` |
| 5 | roms-tools | `rlt-cstar/pr-stable-stretching` | `aca9c41b` | #4 `e9728d2b` |
| 6 | c-star | `rlt-cstar/pr-tiles-generated` | `1f56f37c` | CS-1 `5d35301` (published) |
| 7 | rlt | `research/rlt-cstar-011` | given in the final report | `research/rlt-cstar-010` `a81bce2f` |
| 8–12 | roms-tools | `rlt-cstar/ci-011-{tiling,co2-scale,cdr-units,zero-theta,stable-stretching}` | `4cd15129`, `ffd7c314`, `899834a3`, `e8a9ad93`, `c7986977` | #1–#5, each + one `tests.yaml` commit |
| 13 | c-star | `rlt-cstar/ci-011-f029` | `7e2e938f` | #6 + one workflow commit |

Remote names are absent: 0 `rlt-cstar/pr-*` or `rlt-cstar/ci-011-*` refs on either fork.

```bash
# content and research branches first
P="push --force-with-lease"
git -C /home/claude/roms-tools $P=refs/heads/rlt-cstar/pr-tiling: origin a45d6be376c36da9ce202261b5b1d3e8d021270a:refs/heads/rlt-cstar/pr-tiling
git -C /home/claude/roms-tools $P=refs/heads/rlt-cstar/pr-co2-scale: origin 3500f1dc7791fe07a17bc64d191a9056a90b4676:refs/heads/rlt-cstar/pr-co2-scale
git -C /home/claude/roms-tools $P=refs/heads/rlt-cstar/pr-cdr-units: origin 4df8045e3275a585ca751076e1a703769ecf7e5b:refs/heads/rlt-cstar/pr-cdr-units
git -C /home/claude/roms-tools $P=refs/heads/rlt-cstar/pr-zero-theta: origin e9728d2bb779e7a3c71b926d17847d7d43f04476:refs/heads/rlt-cstar/pr-zero-theta
git -C /home/claude/roms-tools $P=refs/heads/rlt-cstar/pr-stable-stretching: origin aca9c41bc94be9de7d30a79d48322315ccbb13d9:refs/heads/rlt-cstar/pr-stable-stretching
git -C /home/claude/c-star $P=refs/heads/rlt-cstar/pr-tiles-generated: origin 1f56f37ca048d1b96d63d076ebba5624090da233:refs/heads/rlt-cstar/pr-tiles-generated
git -C /home/claude/rlt $P=refs/heads/research/rlt-cstar-011: origin <rlt-011>:refs/heads/research/rlt-cstar-011
# then the CI branches
git -C /home/claude/roms-tools $P=refs/heads/rlt-cstar/ci-011-tiling: origin 4cd15129b679cb19d638af71ffd5e97bdbaeee96:refs/heads/rlt-cstar/ci-011-tiling
git -C /home/claude/roms-tools $P=refs/heads/rlt-cstar/ci-011-co2-scale: origin ffd7c31420bd95e76bd3f7869cc5804063d8d6a7:refs/heads/rlt-cstar/ci-011-co2-scale
git -C /home/claude/roms-tools $P=refs/heads/rlt-cstar/ci-011-cdr-units: origin 899834a38b6bd8af25643aa47a0ecb81fd6b9955:refs/heads/rlt-cstar/ci-011-cdr-units
git -C /home/claude/roms-tools $P=refs/heads/rlt-cstar/ci-011-zero-theta: origin e8a9ad934fc743514acae182fdf849c4aed766d8:refs/heads/rlt-cstar/ci-011-zero-theta
git -C /home/claude/roms-tools $P=refs/heads/rlt-cstar/ci-011-stable-stretching: origin c79869779b9be7b2aff87a7e5f3f61c6c1f4eff3:refs/heads/rlt-cstar/ci-011-stable-stretching
git -C /home/claude/c-star $P=refs/heads/rlt-cstar/ci-011-f029: origin 7e2e938f937b275dafcfd8fb6975c162a25d4b0c:refs/heads/rlt-cstar/ci-011-f029
```

**Expected CI.**
- **roms-tools, each branch:** suite jobs pass; the 2 Copernicus jobs fail on missing credentials, as in 009.
- **c-star `ci-011-f029`:** Forge 101 passed; unit tests on Ubuntu pass; macOS unit tests may again hit the 30-minute job limit.

## 4. Proposed upstream submission order (after fork CI; each needs your approval)

1. **UR-1** ucla-roms CI base-commit check: independent and CI-only.
2. **UR-2** ucla-roms stable `CSF` + references; with **RT-4 → RT-5**, the ROMS-Tools pair. Roundoff-level output change, measured, with references reproduced in two CI runs.
3. **RT-2 → RT-3** CDR constant test, then units contract. No metric definition changes.
4. **RT-1** uneven partitioning and joins, then a roms-tools release, so C-Star can raise its floor (F-024).
5. **CS-1 → CS-2** C-Star coarse-grid validation, then the tile check on generated values.

Each PR is self-contained, with its evidence in the description. The user submits and discusses them in their own words, per the [C]Worthy AI policy.

## 5. Questions for [C]Worthy, kept out of every PR

- **F-009 / F-018:** the meaning of the DIC-based uptake metric for DIC-removing releases. Work is on HOLD locally.
- **F-012:** RivR2O DON/DOP ratios.
- **F-024:** release order vs the C-Star guard (HOLD).
- **F-021:** whether ucla-roms should stop on empty edge coarse tiles.
- **F-030:** the namelist tier for #475's new keys.

## 6. Not verified

- Fork CI of the split roms-tools branches and of CS-2 (needs the push).
- ifx full-field differences.
- macOS unit totals (job time limit).
- AUTO-mode Forge outcomes on a grid where roms-tools picks consistently.

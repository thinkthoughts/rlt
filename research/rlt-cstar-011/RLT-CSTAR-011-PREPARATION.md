# RLT-CSTAR-011 — Upstream contribution preparation (draft)

Date: 2026-11-23. Starts from the RLT-CSTAR-010 CI acceptance. Everything here is local: the PR-split branches are unpushed and the drafts unsubmitted. No contact with [C]Worthy.

## 1. Independent review of F-029 (`134183a`)

A separate reviewer, without the author's context, examined the commit read-only.
- **What it ran:** new and parent tests; Tier 1 (101 passed); 704 Forge unit tests; a 3,000-input fuzz of `settings_support_problems`.
- **What it probed:** stale and consistent snapshots on the 8×8 parent.

**Verdict: REVISE.** I reproduced its central case before recording it.

| Finding | Evidence | Kind |
|---|---|---|
| The fix works for its target case; the new test fails on the parent code and passes on the commit | reviewer run; local negative control (010) | confirmed |
| `settings_support_problems` output is byte-identical before and after | 3,000-input fuzz, identical sha256 | confirmed |
| **False refusal (case A):** `partitioning` 1×1 with a stale 2×2 snapshot and `interp_frc` on is refused, citing `partitioning.n_procs_x` = 2, although generation writes 1×1 and ucla-roms runs it | reviewer probe; reproduced here | must fix |
| **False refusal (case E):** `coarse_grid_mode: never` with a stale `interp_frc = .true.` is refused | reviewer probe | must fix |
| **Duplicate or contradictory messages (case C, a 2×1 snapshot):** two messages for one problem, one citing a value that will never be written; the substring dedup misses partial overlaps | reviewer probe | must fix |
| The section tuple in `engine.py` repeats the sections `tile_size_problems` reads | source | nice to have |

Cases A and E predate the commit: the snapshot check was already there. The commit's own premise, that generation writes these values whatever the snapshot says, makes them wrong, and the fix stops halfway.

**Revision plan:**
- Run the tile check once, on the overlaid settings only, and remove the snapshot-path tile check from `blueprint_problems` and the substring dedup.
- Take the overlay sections from one constant.
- Add tests:
  - A (accepted);
  - C (exactly one message, citing the `partitioning` values);
  - E (accepted);
  - BGC-only and physics-only surface items;
  - AUTO (snapshot untouched);
  - `partitioning.auto_tiling` with a stale cppdef;
  - a unit-level test of `derived_tile_size_problems`.
- Update the release note.

F-029 stays out of CS-1 until the revision is reviewed again.

## 2. Pull-request drafts

There is one draft per reviewable change, in `pr-drafts/`. Each follows its repository's `PULL_REQUEST_TEMPLATE.md`, including the `AI disclosure:` line. No commit carries an AI trailer.

| ID | Repository | Branch / head | Stacks on | Change | Release note | Gates run on the split branch |
|---|---|---|---|---|---|---|
| UR-1 | ucla-roms | `rlt-cstar/f028-base-sha` `79b85c8a` (published) | `main` | CI: validate the base commit (F-028) | none (`no-changelog`) | harness 24/24; pre-commit |
| UR-2 | ucla-roms | `rlt-cstar/integration-010` `c380aab7` (published) | `main` | stable `CSF` + refreshed references (F-011) | output, bugfix | fork CI 45288508832 pass; local full-field comparison |
| RT-1 | roms-tools | `pr/tiling` `a45d6be3` (local) | `main` | uneven partition, `partition` attribute, whole-coarse join, guide (F-001–F-003, F-013, F-020) | feature, 2 bugfix | 66 tiling tests; pre-commit |
| RT-2 | roms-tools | `pr/co2-scale` `3500f1dc` (local) | `main` | independent tonnes-CO₂ test; constant cited (F-006, F-007) | none (`no-changelog`) | 20 tests; pre-commit |
| RT-3 | roms-tools | `pr/cdr-units` `4df8045e` (local) | RT-2 | CDR units contract, case-sensitive symbols (F-008, F-025) | bugfix | 83 tests; pre-commit |
| RT-4 | roms-tools | `pr/zero-theta` `e9728d2b` (local) | `main` | accept θ_s = 0, θ_b = 0 (F-010) | feature | 92 passed, 1 skipped; pre-commit |
| RT-5 | roms-tools | `pr/stable-stretching` `aca9c41b` (local) | RT-4 | stable stretching (F-011) | bugfix | 97 passed, 1 skipped; pre-commit |
| CS-1 | C-Star | `rlt-cstar/integration-010` `5d35301` (published) | `main` | Rule L, empty edge tiles, fixture repair (F-004, F-021, F-026, F-027) | 2 bugfix | fork CI 45288503456 pass; unit Ubuntu pass |

**Split integrity.** The five roms-tools branches together reproduce the accepted integration branch `da016eb1` exactly. Every file the integration branch changes equals that file's state at the tip of the PR branch that owns it, and each file has one owner (RT-4 and RT-5 share two files through stacking). The lint repair `da016eb` was split by file into each PR's own "Satisfy the pre-commit gates" commit. RT-2 and RT-4 needed small format-only fixes of their own once separated.

## 3. Recommended order and dependencies

1. **UR-1:** CI only, independent, quick to review.
2. **RT-4 → RT-5 and UR-2:** the stretching pair. UR-2 changes model output at roundoff (measured), and RT-5 keeps ROMS-Tools grids consistent with it. They can merge independently; reviewing them together is clearest.
3. **RT-2 → RT-3:** CDR analysis. RT-3 changes no metric definition; the DIC-uptake question (F-009) stays HOLD for the method's authors.
4. **RT-1:** tiling. A roms-tools release with RT-1 lets C-Star raise its `roms_tools` floor (F-024; the partition-support guard stays HOLD as the alternative).
5. **CS-1:** coarse-grid validation, then F-029 once revised and re-reviewed.

## 4. Scientific questions kept separate, for [C]Worthy's input

| ID | Question | Status |
|---|---|---|
| F-009 / F-018 | Should the DIC-based uptake of a DIC-removing release measure extra air–sea uptake or net DIC stored? | HOLD; oracle and proposed change local (roms-tools `cbeacaa2`, `cfc038e9`) |
| F-012 | RivR2O DON/DOP ratios applied to DOC_sl | unresolved; needs the R2O MIP protocol or its authors |
| F-024 | C-Star with roms-tools ≤ 5.4.0 and uneven tilings: release order or guard | maintainer decision; guard `438b1913` HOLD |
| F-021 (ucla-roms side) | `init_coarse` passes tilings with empty edge coarse tiles | report draft to prepare; C-Star refuses them already |
| F-030 | #475's new namelist keys need a C-Star namelist tier | coordination note |

## 5. Open before submission

- Revise F-029 and have it re-reviewed.
- Run each roms-tools PR branch in fork CI before submission. Only the combined branch has run in CI so far.
- Decide how to attach evidence: link the fork records, or summarize in the PR body.
- The drafts are written for the user to submit in their own words. The project's policy asks that replies and discussion on the PRs be the user's own.

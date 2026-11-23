# RLT-CSTAR-010 — Verification record

Date: 2026-11-23. Scope: `RLT-CSTAR-010-SPEC.md`. Everything below is local; nothing was pushed, merged, released or submitted. All logs are in the evidence archive (§8).

## 1. Classification

| Item | Verdict | Basis |
|---|---|---|
| F-027 (Forge fixture vs Rule L) | **RESOLVED** (local) | A and B pass; B fails with Rule L disabled; the 2×2 and 1×1 verdicts match the compiled `init_coarse` lines; unit suite unchanged against `b686945d` (§2) |
| F-028 (zero-SHA base comparison) | **RESOLVED** (local; upstream draft) | 24/24 harness outcomes on both jobs, against 8 wrong outcomes for the unmodified workflow; no registry or write path touched (§3) |
| UCLA-ROMS reference refresh | **justified**, refreshed locally | measured roundoff-level change in exactly the 17 cases; the same after rebase (§4, NUMERICAL-COMPARISON) |
| F-029 (stale snapshot delays Rule L until after generation) | new; correction **proposed** | established locally; `134183a` is on its own branch, outside the approved A + B (§2.3) |
| F-030 (#475 adds namelist keys) | new, informational | needs a C-Star namelist tier with the next ucla-roms release |

## 2. F-027

### 2.1 Reproduction

The local environment uses roms-tools 5.4.0 (the release C-Star's CI installs), nccopy present, and the pinned test data placed in the pooch caches with registry-matching SHA-256.

| Tree | `test_forge_generated_values.py` |
|---|---|
| upstream C-Star `0991b445` | 4 passed (an unrunnable 2×2 coarse setup, never run in ROMS) |
| published `b686945d` | 4 errors, Rule L's message (as in CI run 45245977640) |

### 2.2 Commits and gates (recorded separately)

| Gate | 009 head `b686945d` | A `fbd68e8` | B `5d35301` | F-029 `134183a` |
|---|---|---|---|---|
| Tier 1 (`test_fixtures.py` + `forge/`) | 94 passed, 4 errors | 98 passed | 100 passed | 101 passed |
| Tier 1 skipped | 0 | 0 | 0 | 0 |
| unit suite (chunked): passed / failed / skipped | 10017 / 4 / 65 | 10017 / 4 / 65 | 10017 / 4 / 65 | 10017 / 4 / 65 |
| pre-commit (pinned hooks) on the changed files | — | passed (with B) | passed | passed |

- **A** (`fbd68e8`, "Run the coarse-grid Forge fixture on one tile"): the fixture uses `Partitioning(1, 1)` and a stale snapshot `np_xi = 2`, and asserts `np_xi == 1`. The other three tests in the module are unchanged and pass.
- **B** (`5d35301`, "Test that Forge refuses coarse-grid forcing on the 2x2 test tiling"): a new module, `test_forge_coarse_tiling.py`.
  - With `coarse_grid_mode: always` and a resolver-consistent snapshot on 2×2, `process_forge_blueprint` raises `InvalidForgeBlueprintError`. The message names `np_xi=2 … llm=8 … tiles of 2` and `np_eta=2 … mmm=8 … tiles of 2`, and no `.nc` file is written.
  - On 1×1, `blueprint_problems` is empty.
- **Negative control:** with Rule L disabled in `check_tile_sizes`, B's refusal test fails and the 1×1 test passes.
- **Correspondence:** the 008 harness, built from the verbatim `init_coarse` lines, gives `llm = 8`: width 4 at 1 tile (runs), 2 at 2 tiles (stops), 2 at 3, 1 at 4. All 21 harness lines are present unchanged in fork `main` `bf7b5f4d`, and the `nxc <= 2` stop is at `roms_read_write.F90` L562–568.

### 2.3 The stale-snapshot observation (F-029)

The observation is confirmed. With the blueprint's surface items set to `always` while its snapshot still says `interp_frc = .false.`:
- `blueprint_problems` returns `[]`;
- `process_forge_blueprint` generates 7 input files, then `configure_build` raises Rule L's `ValueError`.

With a consistent snapshot, the refusal comes before generation (0.0 s, no files). In a production domain, the inputs generated before that refusal cost real compute.

Proposed correction (`134183a`, branch `rlt-cstar/f029-derived-rule-l`):
- `blueprint_problems` also runs the tile check on the snapshot overlaid with `known_derived_values`;
- the check is factored into one `tile_size_problems`, used by both paths;
- the existing release note's sub-bullet now says "before any input is generated".

Its Tier 1 test fails without the change and passes with it. It is outside the approved A + B, so it is listed separately in the publication plan.

### 2.4 Unit suite

The suite runs one chunk per directory or top-level file, because a single process exceeds this machine's 7 GB at 97 % (killed by the OOM handler). Results per commit are in `cs_gate_<commit>.unit.log`. At `b686945d`, 10,017 tests pass, 65 are skipped and 4 fail locally:
- two `test_domain_catalog.py` GitHub-URL tests;
- `test_gitutils.py::test_copied_checkout_is_not_reported_modified`;
- `test_network_guard.py::test_remote_host_is_refused` ("DID NOT RAISE"; this sandbox routes HTTPS through a proxy).

These depend on the sandbox's network and git setup; all passed in CI run 45245977382 at the 009 head. A (`fbd68e8`), B (`5d35301`) and F-029 (`134183a`) give per-chunk outcomes identical to `b686945d`: the same 10,017 / 4 / 65 and the same 4 tests.

## 3. F-028

- **Change:** `79b85c8a` on fork `main`, both "Look for hash mismatches to report" steps. The step skips the comparison, with a message, when the base is empty, all zeros, not a 40-hex SHA, or not fetchable as a commit (`git fetch --depth=1 origin <sha>` plus `git cat-file -e`, as the `prepare` job checks).
- **Harness** (`tools/test_f028.py`): each step's own `run:` text, executed in a depth-1 clone of a bare origin that serves any SHA, as GitHub does. Inputs are empty, all zeros, valid, unknown, malformed (`main; touch PWNED`) and a 12-character SHA, each with and without a mismatches file.

  | Workflow | Wrong outcomes |
  |---|---|
  | fork `main` (unmodified) | 8 of 24: all-zero, unknown, malformed and short SHA each proceed to checkout, on both jobs |
  | `79b85c8a` | 0 of 24 |
  | fork CI branch `c5ea8ee4` | 0 of 24 |

  No input is executed.
- **Guards:** the diff adds 20 lines inside the two steps only. The write-path audit on the CI branch reports 0 unguarded write paths: `build_gnu`, `build_ifx` and `cleanup` keep their job and step guards.
- **Limit:** a branch's first push now skips the comparison cleanly instead of failing; it still gets no report. A merge-base fallback would add one (not done).

## 4. UCLA-ROMS references

- **Dry run:** `update_references.py --run 45245982301 --repo thinkthoughts/ucla-roms --dry-run` lists 17 changed, 0 added, 0 removed for github_gnu and github_ifx, the same 17 cases. Nothing was written.
- **Measurement:** `RLT-CSTAR-010-NUMERICAL-COMPARISON.md`.
  - The local baseline reproduces the CI references in 11 of 17 cases, and the local head reproduces the CI head hashes in those 11.
  - All changes are roundoff-level (≤ 5.2e-11 relative; filament's near-zero fields ≤ 5.6e-9 with |Δ| ≤ 3.9e-15).
  - `Cs_r`/`Cs_w` of both versions are within 2.6e-16 of a 50-digit reference.
  - ifx is measured through CI statistics only.
- **Rebase:** onto fork `main` `bf7b5f4d`, the two commits are patch-identical (`git range-diff`: `=`). The rebased branch changes exactly the same 17 cases, and all 477 hashed variables are bit-identical to the 009 head. `main` is bit-identical to `f83d2ef6`.
- **Refresh:** after the comparison was documented, `update_references.py --run 45245982301` was applied on the rebased branch and committed as `c380aab7`.
  - 429 entries changed per compiler, each equal to the CI computed entry; every other entry is unchanged.
  - Local rerun on `c380aab7`: 120 passed, 6 xfailed. The 6 are the local-build cases that also differ at `f83d2ef6` and `bf7b5f4d`. The 11 CI-reproducible cases now pass against the new references.
- **Pre-commit** on the rebased range and on F-028: passed.

## 5. Holds, pins and trust base (unchanged)

- F-009 (roms-tools `cbeacaa2`, `cfc038e9`) and the C-Star partition-support guard (`438b1913`) are untouched and local.
- No carbon-uptake definition was edited, and no roms-tools commit was made in 010.
- Pins (ucla-roms `f83d2ef6`, C-Star `0991b445`, roms-tools `ede4fe62`, RLT `6128c97`) and published 009 heads are unchanged; remote heads have not been touched since the 009 push.
- RLT certificates, Lean proofs and the kernel are untouched: no rlt commit was made in 010.

## 6. What was not run

- ifx builds; full-field ifx differences.
- macOS.
- The roms-tools Copernicus jobs.
- Fork CI of the 010 branches (it needs a push).
- The five MARBL cases and `implicit_fs_real` are bit-reproducible against CI only in CI's own build environment. Locally they are measured base-versus-head in the local build.

## 7. Scratch-only changes

The ucla-roms worktrees carry an uncommitted `tests/conftest.py` edit that copies the CI input artifact instead of regenerating inputs (`tools/patch_inputs.py`, `patch_inputs2.py`). It is in no commit, and `git status` on each branch shows it as the only modification.

## 8. Evidence

`RLT-CSTAR-010-evidence.tar.gz` with `SHA256SUMS`: suite logs (base, head, main, rebased, rebased with new references), compare reports, `cs_vs_mpmath.txt`, `rebase_hash_check.txt`, dry-run and apply output, CI statistics comparison, F-028 harness outputs, C-Star Tier 1 and unit logs, negative controls, tools, and bundles of every local 010 branch.

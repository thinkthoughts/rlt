# [C-Star] Refuse tilings ucla-roms stops on for coarse-grid surface forcing

Branch (fork, published): `rlt-cstar/integration-010` `5d35301` on `main` `0991b445` — 6 commits (`1bdf4e0`, `e3febec`, `3044bf1`, `b686945`, `fbd68e8`, `5d35301`). The follow-up that checks the tiles on the values generation writes (CS-2) is a separate, stacked PR.

# Summary

ucla-roms' `init_coarse` (0.6.2 and later) gives each tile `ceil((llm//2)/np_xi)` coarse points (likewise `mmm`, `np_eta`) and stops with "tiling too fine for coarse-resolution forcing inputs" when that is 2 or fewer. C-Star accepted such tilings, so a run with coarse-grid forcing could be built and queued and then stop at the first forcing read.

- **Rule L** (`cstar/roms/precheck.py`, Forge `check_tile_sizes`): refuses those tilings before anything is built or queued, when the staged `cppdefs.opt` shows a forcing module whose switch is on (`interp_bulk_frc` with `BULK_FRC`, `interp_flux_frc` without it, `interp_bgc_frc` with MARBL or BEC2). Auto-tiling is not checked.
- **Empty edge tiles**: `init_coarse` tests the ceiling width before taking the surplus from the edge tiles, so it passes tilings whose edge coarse tile holds no points (e.g. 150 coarse points on 21 tiles). Those are refused too (`narrowest_coarse_tile`), with a message that says ROMS runs them but the forcing would be wrong.
- **Test fixture**: the Forge generated-values fixture ran the 8×8 test grid at 2×2 with `coarse_grid_mode: always`, a run ucla-roms stops on. It now uses one tile and keeps a stale stored `np_xi` to test that tile counts come from the domain. A new module tests that the 2×2 setup is refused before any input is generated.

How it was tested:
- 102,272 (llm, np_xi) pairs against the compiled `init_coarse` lines: Rule L refuses exactly the tilings ROMS aborts on or leaves an empty edge tile, 0 mismatches.
- Formal: Lean 4 `coarse_ok_iff` (`lean/RLTCSTAR002.lean`: `3 ≤ ceil(q/t)` exactly when `2t < q`), `guard_passes_edge_empty` and `guard_alone_insufficient` (`lean/RLTCSTAR007.lean`: `init_coarse`'s ceiling test passes 150 coarse points on 21 tiles, whose first and last tiles are empty, so that test alone does not guarantee non-empty tiles), and `ruleL_tiles` (with `3 ≤ ceil` and a non-empty last tile, every tile is non-empty and every interior tile has at least 3 coarse points), with RLT kernel certificates G04, G05 and G08 (ACCEPT) for the concrete cases.
- Fork CI on this branch plus CI triggers (run 45288503456): Forge integration 100 passed on Ubuntu and on macOS; e2e (builds ucla-roms, MARBL, PIO, runs mpirun) 92 passed, 7 skipped (zsh). Unit tests (run 45288503417): 10,025 passed, 61 skipped per Ubuntu Python 3.12–3.14; the macOS job ran every test with 0 failures and then exceeded the 30-minute job limit.
- Locally: the unit suite gives the same results at each of the last three commits; the refusal test fails with Rule L disabled.

Release notes: `+coarse-tile-sizes.bugfix.rst`, `+coarse-edge-tiles.bugfix.rst`.

CI limitations: macOS unit job timed out after its tests (above); e2e skips 7 zsh tests, as upstream.

Release ordering: independent of the ROMS-Tools PRs. Open questions for maintainers, not part of this PR: C-Star's acceptance of uneven tilings with roms-tools ≤ 5.4.0 (RLT-CSTAR F-024), and whether ucla-roms itself should stop on empty edge coarse tiles (F-021).

AI disclosure: Claude (Anthropic); wrote the code, tests and this description, and built the comparison harness, under the author's direction and review.

## Code Review Checklist
- [x] Release notes added as `docs/changes/+coarse-tile-sizes.bugfix.rst`, `+coarse-edge-tiles.bugfix.rst`
- [x] New functionality has documentation, type hint coverage, and tests
- [x] Tests (fork CI) and `pre-commit` (changed files) are passing

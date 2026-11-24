# [C-Star] Check the tiles before generation on the values generation writes

Branch (fork, proposed): `rlt-cstar/pr-tiles-generated` `1f56f37`, stacked on CS-1 (`5d35301`) — 1 commit. Merge after CS-1.

# Summary

`blueprint_problems` (`cstar blueprint check`, and `process_forge_blueprint` before generating) checked tile sizes on the blueprint's stored `model_settings` snapshot. Generation writes the tiles from `domain.partitioning` and `interp_frc` from an explicit `coarse_grid_mode`, whatever the snapshot says, and `configure_build` checks the tiles again on what it wrote. So a saved blueprint whose snapshot had gone out of date was judged on values it would never run with:
- refused when its partitioning had been fixed (2×2 stored, 1×1 to be written) or its `coarse_grid_mode` set to `never`;
- passed when `coarse_grid_mode` had been set to `always` (or the tiles made too fine), then refused only after every input was generated.

The tile check now runs once, on `generated_tile_settings(cfg)`: the snapshot overlaid with `known_derived_values`.
- `settings_support_problems` and `validate_run_time_sections` take it as `tile_settings`; the resolver and `configure_build` keep checking the settings they pass, byte-identically.
- The wizard's load-time check passes it too, so a loaded blueprint is judged as `cstar blueprint check` judges it.
- The stored values still apply where the blueprint fixes nothing: the tiles under `partitioning.auto_tiling`, `cppdefs.auto_tiling`, grid sizes from a grid file, and `interp_frc` when every surface item says `auto` (roms-tools decides during generation).

How it was tested:
- **Oracle.** For 12 scenarios on the Tier 1 `unified` parent, the pre-generation verdict and message are compared with what `process_forge_blueprint(validate=False)` does after real generation (`configure_build`'s own refusal). Scenarios: consistent, stale flag, stale tiles with a good or bad partitioning, a 2×1 snapshot, `never` with a stale flag, `auto` with either flag, BGC-only, physics-only, `auto_tiling` with a stale cppdef, one tile.
  - Wrong verdicts or messages: before this change 7; a first version of this change 4; this commit 0.
  - In the 6 refusal cases the message is identical to `configure_build`'s.
- **Independent oracle.** A second reviewer's oracle adds 9 more scenarios (mixed modes, integer flags, missing sections, a stale `llm`, a 16×1 partitioning, a stale BGC flag with physics only): before 12 wrong, this commit 0.
- **Unit tests.** New `TestGeneratedTileSettings` (11 tests, no generation needed) and a wizard load test. The new Tier 1 test asserts that no `.nc` file exists after the refusal.
- **Negative controls.**
  - With the snapshot tile check restored, 4 of the unit tests fail.
  - The reviewer's three broken copies (no overlay; both checks, as in the first version; snapshot over overlay) fail 6, 5 and 6 of them, and the Tier 1 test.
  - Without the wizard change, its test fails.
- **Non-regression.** 4,000 random settings give byte-identical `settings_support_problems` and `validate_run_time_sections` output without `tile_settings`.
- **Suites.** Tier 1 (`test_fixtures.py` + `forge/`): 101 passed. Unit suite: unchanged against the parent except the new tests (local-environment failures as at the parent).

Limits:
- Under `auto`, the grid choice is made during generation, so the stored flags are the best available before it, and `configure_build` remains the check.
- Separately, an existing case: a physics/BGC `auto` pair that roms-tools resolves differently fails at generation with "Mismatch in coarse grid settings between surface forcing types" after writing inputs. A candidate follow-up, not part of this PR.

Release note: the sub-bullet of `+coarse-tile-sizes.bugfix.rst` (added in CS-1) now says the early refusal uses the values generation will write.

CI: fork CI branch `rlt-cstar/ci-011-f029` (this commit + fork-only triggers), not yet run.

AI disclosure: Claude (Anthropic); wrote the change, the tests and the oracle, and drafted this description; an independent Claude reviewer examined two versions under the author's direction, and the author reviewed the result.

## Code Review Checklist
- [x] Release note updated (`docs/changes/+coarse-tile-sizes.bugfix.rst`)
- [x] New functionality has documentation, type hint coverage, and tests
- [x] Tier 1, unit tests and `pre-commit` (changed files) are passing locally; fork CI pending

# [roms-tools] Accept theta_s = 0 and theta_b = 0, as ucla-roms does

Branch (fork, proposed): `rlt-cstar/pr-zero-theta` `e9728d2b` on `main` `ede4fe62` — 2 commits (`6b91052`, `e9728d2`).

# Summary

`vertical_coordinate.py` refused `theta_s = 0` and `theta_b = 0`. ucla-roms' `CSF` supports both: `theta_s = 0` gives `csrf = −σ²` and `theta_b = 0` no bottom refinement, the θ → 0 limits of the stretching. `compute_cs` now takes the same limits, and `Grid` accepts the values.

Tests: equal to compiled ucla-roms `CSF` (exact at θ_s ∈ {0, 5}, ≤ 2.2×10⁻¹⁶ at 10); 11 new tests. `pytest roms_tools/tests/test_vertical_coordinate.py roms_tools/tests/test_setup/test_grid.py`: 92 passed, 1 skipped. `pre-commit` on the changed files: passed.

CI: fork CI branch `rlt-cstar/ci-011-zero-theta`, not yet run; the combined branch passed (run 45245972980).

Release note: `+zero-theta.feature.md`.

AI disclosure: Claude (Anthropic); wrote the code, tests and this description under the author's direction and review.

## Code Review Checklist
- [x] Release note added as `docs/changes/+zero-theta.feature.md`
- [x] New functionality has documentation, type hint coverage, and tests
- [x] Tests and `pre-commit` (changed files) are passing

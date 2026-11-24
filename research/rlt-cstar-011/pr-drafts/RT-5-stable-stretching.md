# [roms-tools] Evaluate the S-coordinate stretching without cancellation

Branch (fork, proposed): `rlt-cstar/pr-stable-stretching` `aca9c41b`, stacked on RT-4 (`rlt-cstar/pr-zero-theta`) — 2 commits of its own (`007d6d8`, `aca9c41`). Pairs with the ucla-roms PR UR-2.

# Summary

`compute_cs` evaluated `(1 − cosh θσ)/(cosh θ − 1)` and `(e^{θ_b C} − 1)/(1 − e^{−θ_b})`, which cancel as θ → 0: NaN for θ_s ≤ 10⁻⁸, errors up to 2.8×10⁻³ at small θ_s, 7×10⁻⁷ at θ_b = 10⁻¹⁰. They are now `−(sinh(θσ/2)/sinh(θ/2))²` and `expm1` forms, algebraically identical.

Tests: 8,748 points against 50-digit mpmath: ≤ 7.8×10⁻¹⁶ everywhere. Agrees with the matching ucla-roms change to 10⁻¹⁵. Mixed versions (new ROMS-Tools, old ucla-roms) differ ≤ 7.6×10⁻¹⁵ for θ_s ≥ 0.5 and θ_b = 0 or ≥ 0.1, so grids made before and after stay consistent in the ordinary range. `pytest roms_tools/tests/test_vertical_coordinate.py roms_tools/tests/test_setup/test_grid.py`: 97 passed, 1 skipped. `pre-commit` on the changed files: passed.

Evidence class: numerical (50-digit mpmath reference, not a formal proof). The ucla-roms side (UR-2) is measured end to end: model output moves at roundoff in its 17 hashed cases, and the new reference hashes reproduced in two fork CI runs on gnu and ifx.

CI: fork CI branch `rlt-cstar/ci-011-stable-stretching` (carries RT-4 below it), not yet run; the combined branch passed (run 45245972980).

Release note: `+stable-stretching.bugfix.md`.

AI disclosure: Claude (Anthropic); wrote the code, tests and this description under the author's direction and review.

## Code Review Checklist
- [x] Release note added as `docs/changes/+stable-stretching.bugfix.md`
- [x] Tests and `pre-commit` (changed files) are passing

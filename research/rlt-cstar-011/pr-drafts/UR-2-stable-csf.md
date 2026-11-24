# [ucla-roms] Evaluate the S-coordinate stretching CSF without cancellation

Branch (fork, published): `rlt-cstar/integration-010` `c380aab7` on `main` `bf7b5f4d` — 3 commits (`9febb1e0` code, `6845bb4c` release notes, `c380aab7` reference hashes). Pairs with roms-tools RT-5.

# Summary

`CSF` in `set_scoord_mod.F90` computed `(1 − cosh(θ_s σ))/(cosh θ_s − 1)` and `(exp(θ_b C) − 1)/(1 − exp(−θ_b))`, which cancel as θ → 0: `Cs_w` is NaN at θ_s = 10⁻⁸ and errors reach 2.8×10⁻³ at small θ_s. They are now `−(sinh(θ_s σ/2)/sinh(θ_s/2))²` and `expm1(θ_b C)/(−expm1(−θ_b))`, algebraically identical; `expm1` is bound from C through `iso_c_binding` (builds and runs with gfortran and ifx).

**Output change: roundoff, measured.** 17 hashed cases change on both compilers.
- Full-field comparison (gnu, `tests/compare_outputs.py`) against a baseline that reproduces this repository's reference hashes bit for bit in 11 of the 17 cases, with the CI run's own input files: 449 changed variables, the largest difference 5.2×10⁻¹¹ of the variable's largest value. The exception is the symmetric `filament` case, whose near-zero `omega`/`w` move by ≤ 3.9×10⁻¹⁵ absolute (5.6×10⁻⁹ relative). Tracers move ≤ 1×10⁻¹³ relative.
- The vertical structure moves first (`omega`, `w`, mixing coefficients, boundary-layer depths); several cases keep `temp` and `salt` bit-identical while `rho` moves through its depth dependence.
- `Cs_r`/`Cs_w` written by both versions are within 2.6×10⁻¹⁶ of a 50-digit evaluation in every history file (θ_s, θ_b ∈ {(0,0), (6,2), (6,6)}): in the suite's range the change reorders roundoff. Its accuracy gain is at small θ, which no test reaches.
- ifx: same 17 cases; 421 of 429 changed variables keep min/max/mean/rms identical to 10 digits, the rest are near-zero means and filament extremes (≤ 7.3×10⁻⁹ of rms).

**References.** `c380aab7` was produced with `tests/update_references.py --run 45245982301` (17 changed, 0 added, 0 removed per compiler). A second CI run on this exact branch (run 45288508832, after rebasing onto #475/#476) reproduced all 477 hashed variables on both compilers: `pytest_gnu` 126 passed, `pytest_ifx` 126 passed, 0 mismatches. #475 and #476 leave these outputs bit-identical. If `main` moves again before merge, the references should be regenerated from this PR's own CI run.

Release notes: `+stable-csf.output.rst`, `+stable-csf.bugfix.rst`.

Evidence classes: the stretching formula against a 50-digit reference (numerical); output differences measured with `tests/compare_outputs.py` (gnu, full fields) and from CI statistics (ifx, 10 digits); reference hashes confirmed by two CI runs. Not measured: full-field ifx differences (no local ifx).

Related but separate: ucla-roms `init_coarse` tests the ceiling coarse width before taking the surplus from the edge tiles, so it passes tilings whose edge coarse tile is empty (RLT-CSTAR F-021; e.g. 150 coarse points on 21 tiles). C-Star now refuses those tilings (CS-1); whether ucla-roms should stop on them too is for the maintainers, and is not part of this PR.

Full measurement record: RLT-CSTAR-010-NUMERICAL-COMPARISON.md (attached).

AI disclosure: Claude (Anthropic); wrote the change, ran and analysed the comparisons, and drafted this description under the author's direction and review.

## Code Review Checklist
- [x] Release note added as `docs/changes/+stable-csf.output.rst` and `+stable-csf.bugfix.rst`
- [x] No namelist option or cppdef changes
- [x] New functionality has tests (reference hashes refreshed)
- [x] Tests (fork CI 45288508832) and `pre-commit` (changed range) are passing

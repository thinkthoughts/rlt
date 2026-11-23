# RLT-CSTAR-010 — UCLA-ROMS numerical comparison (stable CSF)

Date: 2026-11-23. Change under test: ucla-roms `d07706a0` (published in `rlt-cstar/integration-009` at `c5ea4764`), the cancellation-free S-coordinate stretching `CSF` (F-011). Question: are the 17 reference-hash changes seen in CI run 45245982301 the intended numerical result, and do they justify new reference hashes?

## 1. Evidence classes

- **Measured in CI:** the computed hashes and 10-digit statistics of run 45245982301 (artifacts `results-github_gnu`, `results-github_ifx`) against the reference files at `f83d2ef6`.
- **Measured locally:** full-field comparison of base and head runs that read the CI run's own input files, on a baseline shown to reproduce the CI references bit for bit.
- **Carried over (expectation):** the 007/008 statements that the CSF change moves output at roundoff. Those came from 3 analytical cases and a harness. They are used here only as the expectation being tested.

## 2. Baseline and method

| Run | Tree | Result (local, gfortran 13.3, OpenMPI 4, MARBL 0.45.0, ParallelIO 2.7.1-fork, PnetCDF 1.12.3) |
|---|---|---|
| base | `f83d2ef6` (0.16.0, the reference files' commit) | 106 passed, 6 xfailed, 2 skipped |
| head | `c5ea4764` (published 009 branch) | 95 passed, 17 xfailed, 2 skipped |
| main | `bf7b5f4d` (fork `main`, #475 + #476) | 120 passed, 6 xfailed, 2 skipped |
| rebased | `6845bb4c` (head rebased onto `bf7b5f4d`) | 109 passed, 17 xfailed, 2 skipped |

- **Inputs:** all four runs copy the 26 files from the CI artifact `inputs-github_gnu` (byte-identical to `inputs-github_ifx`) and split them with the suite's own `partit 2 2` commands. Only scratch worktrees carry this change, in `conftest.py`, and it is never committed. The suite's input sentinel in the local head run reproduces all 26 CI input hashes.
- **Baseline fidelity:** the local base run reproduces the CI reference hashes bit for bit in 11 of the 17 cases. The other 6 (`bgc_real_marbl`, `cdr_3d`, `cdr_dp`, `cdr_parameterized`, `cdr_gas_exch`, `implicit_fs_real`) differ from the references in the base run too. That is a local build difference (CI uses gfortran 15.3 and MPICH), the same in base and main.
- **Head fidelity:** in those same 11 cases, the local head run reproduces the CI head hashes (run 45245982301) bit for bit. For those cases, the local base-versus-head difference *is* the CI change.
- **Tool:** the comparison uses ucla-roms' own `tests/compare_outputs.py` (points differing, largest |Δ|, |Δ| relative to the variable's largest base value). The per-rank `extract_pressure_fluxes` files were compared rank by rank.

## 3. Results

### 3.1 Which cases change

| Check | Result |
|---|---|
| CI dry run (`update_references.py --run 45245982301 --repo thinkthoughts/ucla-roms --dry-run`) | github_gnu: 17 changed, 0 added, 0 removed; github_ifx: the same 17; nothing written |
| Local head, cases changed | exactly the CI 17 |
| Local rebased, cases changed | exactly the CI 17 |
| main vs base, all 477 hashed variables | 0 differ (#475 and #476 leave these outputs bit-identical) |
| rebased vs head, all 477 hashed variables | 0 differ (the rebase leaves the CSF change's outputs bit-identical) |

### 3.2 How far the outputs move (gnu, measured locally)

449 changed variables across 24 output components:

| Relative difference (|Δ| / max |base|) | variables |
|---|---|
| ≤ 1e-14 | 169 |
| 1e-14 – 1e-12 | 190 |
| 1e-12 – 1e-11 | 70 |
| 1e-11 – 1e-10 | 15 |
| 1e-10 – 1e-9 | 3 |
| 1e-9 – 5.7e-9 | 2 |

The largest absolute difference anywhere is 1.6e-11 m (`hbls`). Tracers move at ≤ 1e-13 relative (DIC and ALK ≤ 7.7e-12 mmol m⁻³, about 3.6e-15 relative).

The 5 entries above 1e-10 are all in `filament`, an idealized symmetric front. In each, the absolute difference is ≤ 3.6e-13 and the field itself is small, so the relative figure is large:

| filament variable | max |Δ| | relative |
|---|---|---|
| omega | 3.9e-15 | 5.6e-9 |
| w | 3.9e-15 | 4.0e-9 |
| ubar | 1.3e-16 | 1.7e-10 |
| vbar | 3.5e-13 | 1.1e-10 |
| u | 1.9e-15 | 1.1e-10 |

| Case | component | changed/total | max relative | variable |
|---|---|---|---|---|
| bgc_real_marbl | physics / bgc | 15/17, 32/33 | 2.0e-11 / 6.1e-15 | omega / NH4 |
| cdr_3d, cdr_dp, cdr_parameterized | physics / bgc | 15/17, 32/33 | 1.4e-11 / 8.6e-14 | omega / NH4 |
| cdr_gas_exch | physics / bgc / cdrgas | 15/17, 32/33, 2/7 | 1.4e-11 / 8.6e-14 / 1.1e-13 | omega / NH4 / ddic_dco2 |
| cdr_lite_parm | physics / cdrtrc | 13/15, 8/11 | 1.4e-11 / 7.1e-14 | omega / CDR_OAE_DIC1 |
| extract_pressure_fluxes (per rank) | childA | 30/32 (rank 0), 18/20 (rank 2) | 2.6e-12 | child_vp_east |
| extract_pressure_fluxes_pio | childA | 36/37 | 2.6e-12 | vp_east |
| extract_two_children_pio | childA / childB | 28/29, 28/29 | 2.7e-13 / 2.1e-13 | v_west / v_south |
| implicit_fs_real | physics | 13/15 | 5.2e-11 | AKs |
| pipes_ana | physics | 13/15 | 2.4e-13 | omega |
| rivers_ana | physics | 13/15 | 2.5e-12 | omega |
| filament | physics | 8/12 | 5.6e-9 (|Δ| 3.9e-15) | omega |
| rivers_real | physics | 13/15 | 7.0e-12 | omega |
| pipes_real | physics | 13/15 | 3.2e-12 | AKs |
| flux_frc | physics | 13/15 | 9.3e-12 | omega |
| bgc_real_bec | physics | 13/15 | 1.2e-11 | omega |

**Pattern.** The vertical structure moves first:
- Physics: `omega`, `w`, mixing coefficients and boundary-layer depths carry the largest relative changes.
- Several no-BGC cases leave `temp` and `salt` bit-identical while `rho` moves (rho depends on depth through the equation of state).
- Tracers move at the 1e-15 level.

That is the signature of a last-bit change in `z_r`/`z_w`, propagated for a few tens of steps.

### 3.3 The stretching itself (measured locally)

Each history file records ROMS's `Cs_r`/`Cs_w`. Against a 50-digit evaluation of the same stretching (1,860 files):

| θ_s | θ_b | N | max error, base | max error, head |
|---|---|---|---|---|
| 0 | 0 | 10 | 1.4e-16 | 1.4e-16 |
| 6 | 2 | 32 | 1.6e-16 | 1.9e-16 |
| 6 | 6 | 10 | 2.6e-16 | 1.5e-16 |

Base and head differ by at most 2.2e-16 (one unit in the last place of an O(1) number). In the suite's parameter range both formulas are exact to about one ulp, so the output change is a reordering of roundoff, not a correction. The accuracy gain of the stable formula is at small θ (F-011: NaN at θ_s ≤ 1e-8 and errors up to 2.8e-3 in the old formula). No test case reaches that range.

### 3.4 Intel (measured in CI, statistics only)

ifx cannot be compiled here, so for ifx only the CI hashes and 10-digit statistics are available:
- The same 17 cases change.
- 421 of 429 changed variables keep min, max, mean and rms identical to 10 significant digits.
- The other 8 are means of near-zero symmetric fields (|mean| ~ 1e-21) and filament extremes, with |Δ| ≤ 7.3e-9 of the field's rms.

The gnu figures from the same statistics test are the same in kind (§3.2 confirms them at full precision). Full-field differences for ifx remain unmeasured.

## 4. Conclusion

The reference updates are justified numerically and scientifically, as a roundoff-level output change:
- the change is confined to the 17 cases that depend on the in-model vertical grid;
- it is identical on the rebased branch;
- its size (≤ 5e-11 of each field's largest value, ≤ 6e-9 only for near-zero fields with |Δ| ≤ 4e-15) and its pattern match a last-bit change in `Cs_r`/`Cs_w`;
- both the old and the new stretching are within about one ulp of the exact value in this range.

Expected versus measured: the 007/008 expectation of "roundoff" is now measured across all 17 cases for gnu (11 of them bit-identical to CI), and across 17 cases for ifx at 10-digit statistical resolution.

The new references must come from CI output of the commit that is reviewed. The 009 run's gnu hashes equal the rebased branch's local hashes in 11 of 17 cases, and the rebased outputs are bit-identical to the 009 head's. A CI run of the rebased branch remains the authoritative source before an upstream PR.

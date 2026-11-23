# RLT-CSTAR-007 · Verification

Specification: `RLT-CSTAR-007-SPEC.md`. Register: `RLT-CSTAR-FINDINGS.md` (F-001…F-023). Evidence paths below are relative to the review archive `RLT-CSTAR-007-review.tar.gz` (directory `evidence/`). Environment: Ubuntu 24.04, Python 3.13 venv, gfortran 13.3, netcdf-fortran 4.5.4, Lean 4.34.0-rc2 (core, no Mathlib), RLT `target/release/rlt` built from `6128c97` (kernel unchanged). Trees are selected by `PYTHONPATH` with plain `python` (`python -I` ignores `PYTHONPATH`); each log records the imported module path (`*.which`) and commit (`*.rev`). The `roms_tools_version` attribute in files written here names the installed distribution, never the research tree (F-023).

Each result is marked **established** (executed or proved here), **inferred** (from source reading), or **judgement** (for [C]Worthy).

## 1. Branches and commits

| Repository | Branch | Head | Commits since pin (oldest first) |
|---|---|---|---|
| C-Star | `research/rlt-cstar-002` | `f0587cb8` | `f4ba79ff` Rule L (002); `f0587cb8` empty edge coarse tiles (007, F-021) |
| roms-tools | `research/rlt-cstar-002` | `a1c52ff7` | `3557a25b` uneven tiling; `f79e0c05` partition attribute; `ae3d658c` whole-coarse join (007, F-020); `a1c52ff7` partition guide (007, F-013) |
| roms-tools | `research/rlt-cstar-003` | `e7358b79` | `755fef69` tonnes-CO₂ test; `e7358b79` units contract (007, F-008/F-019; replaces `b177f818`) |
| roms-tools | `research/rlt-cstar-004` | `23a7b518` | `23a7b518` θ = 0 (unchanged) |
| roms-tools | `research/rlt-cstar-005-dic-uptake` | `cbeacaa2` | on 003: `9cc9e95d` PROPOSED F-009 correction; `cbeacaa2` analytic oracle |
| roms-tools | `research/rlt-cstar-006-stable-csf` | `e24817f4` | on 004: `e24817f4` stable stretching (F-011) |
| ucla-roms | `research/rlt-cstar-007-stable-csf` | `0c63d80d` | `0c63d80d` stable `CSF` (F-011) |
| rlt | `research/rlt-cstar-007` | see archive `BRANCHES.txt` | docs, G08 certificate, `lean/RLTCSTAR007.lean` |

Baseline: tags `baseline/rlt-cstar-002-review/*` (C-Star `f4ba79ff`; roms-tools `f79e0c05`, `a6644589`, `23a7b518`; rlt `c0ce16f`) and the archive `RLT-CSTAR-002-review.tar.gz` (SHA-256 `faa3c097…5302d`), copied unchanged into `baseline/`.

**Compositions.** All ROMS-Tools branches merge without conflicts. `final` = 002 + 005 (which contains 003) + 006 (which contains 004) = `7bf39df4`. `noF009` = 002 + 003 + 006 = `2493a715`; it differs from `final` in exactly the four F-009 files.

## 2. P1 · F-009, the DIC-difference uptake of a release

### 2.1 Intended quantity — inferred

- ROMS-Tools `docs/cdr_analysis.ipynb` describes both metrics as estimates of "how much extra CO₂ the ocean takes up because of the intervention", calls flux and inventory two parallel ways to tell the same physical story, and defines efficiency as moles of DIC transferred from the atmosphere per mole of alkalinity added. It cites Isometric's air–sea gas exchange module.
- Isometric module 1.0 §3.1.4 recommends both methods as a cross-check that should give the same answer; it describes the inventory method as the additional DIC stored and treats alkalinity enhancement. It does not discuss releases that remove DIC directly.
- ucla-roms `cdr_output.F90` (`accumulate_marbl_fields`) states the design goal that a window ending on a MARBL step averages exactly the flux behind its DIC change.

Reading: the metric is the **extra air–sea uptake**, and the inventory method is meant to agree with the flux method. This is inferred from documentation; the method's authors decide it (§2.5).

### 2.2 Budget identity — inferred from the pinned sources

Summed over the interior rho points (`_INTERIOR`, the area-weighted sum `compute_cdr_metrics` uses):

```
d/dt Σ (hDIC − hDIC_ALT_CO2) A
   = Σ (FG_CO2 − FG_ALT_CO2) A                          air–sea flux, positive into the ocean
   + Σ (DIC_source − DIC_ALT_source)                    the release's own DIC
   + Σ (P_CaCO3_remin − P_CaCO3_ALT_CO2_remin) dV       CaCO₃ dissolution response
   − net export of (DIC − DIC_ALT_CO2) across the interior's sides
```

| Term | Source | Sign, accumulation, domain |
|---|---|---|
| FG_CO2, FG_ALT_CO2 | MARBL `marbl_surface_flux_mod.F90` L388-389 (`pv_co2·dco2star`, added to the DIC surface flux); `FG_ALT_CO2` from `xco2_alt_co2` and the ALT tracers | positive into the ocean; written as window averages (`cdr_output.F90` `finalize_average`, L876) |
| DIC_source, DIC_ALT_source | ucla-roms `step3d_t_mod.F90` L884-930 (each tracer gets its own release; 3-D releases go to ALK and DIC only); `cdr_output.F90` `add_cdr_source_sums` L832-865 | mmol/s per cell, window averages; 3-D forcing leaves the ALT sources at 0 (F-014) |
| hDIC, hDIC_ALT_CO2 | `cdr_output.F90` L331-338, written from `hDIC_tmp` at L1179 | instantaneous at the end of each window (the averaged field is the separate `hDIC_avg`) |
| CaCO₃ term | MARBL `marbl_interior_tendency_mod.F90` L3744-3745: the ALT DIC tendency equals the DIC tendency plus `P_CaCO3_ALT_CO2_remin − P_CaCO3_remin` | nonzero where the two carbonate systems dissolve differently |
| initial state | roms-tools `bgc_model.py` L346 copies DIC to `DIC_ALT_CO2` only when the source data lack it | difference 0 at the start only under that copy (F-018) |
| lateral transport | both tracers are advected by the same velocity field | export of the perturbation across the interior's sides |

For a release that removes DIC (`DIC_source < 0`, `DIC_ALT_source = 0`), the inventory difference therefore holds **uptake − removal** (plus the CaCO₃ and export terms). At `ede4fe62`, `compute_cdr_metrics` reports it as the uptake.

### 2.3 Independent oracle — established

`roms_tools/tests/test_analysis/test_cdr_uptake_analytic.py` (`cbeacaa2`) builds ucla-roms-shaped output from a closed-form one-box model; it calls no code from the module under test. A release adds alkalinity at rate *a* and removes DIC at rate *r*. The carbon deficit D = c·a·t + r·t − U relaxes at rate k, so D(t) = (c·a + r)(1 − e^{−kt})/k and U(t) = (c·a + r)t − D(t). FG_CO2 holds exact window averages of dU/dt over hour-long windows; hDIC holds U − r·t instantaneously at window ends. The uptake is spread unequally over three cells of unequal area. Optional terms: export −e·t, CaCO₃ +d·t.

Final-window values (`f009/oracle_table.out`; mmol, efficiency dimensionless):

| Case | U analytic | pinned: flux / DIC | patched: flux / DIC | efficiency analytic | pinned DIC eff. | patched DIC eff. |
|---|---|---|---|---|---|---|
| OAE | 999,655 | 999,655 / 999,655 | 999,655 / 999,655 | 0.3857 | 0.3857 | 0.3857 |
| DOR | 1,249,569 | 1,249,569 / −1,342,431 | 1,249,569 / 1,249,569 | 0.4821 | −0.5179 | 0.4821 |
| OAE + DOR | 1,499,483 | 1,499,483 / 462,683 | 1,499,483 / 1,499,483 | 0.4132 | 0.1275 | 0.4132 |
| OAE + DOR, export 3 | 1,499,483 | 1,499,483 / −314,917 | 1,499,483 / 721,883 | 0.4132 | −0.0868 | 0.1989 |
| OAE, CaCO₃ 0.5 | 999,655 | 999,655 / 1,129,255 | 999,655 / 1,129,255 | 0.3857 | 0.4357 | 0.4357 |

On the change, both methods equal U(t) to 10⁻⁹ for OAE, DOR and OAE + DOR, and the export and CaCO₃ terms appear exactly as the gap between the methods (6 tests pass). At `ede4fe62` and on 003, the DOR, OAE + DOR and export tests fail (3 of 6). The flux method agrees with the analytic uptake in every case, on every tree.

### 2.4 Status — PROPOSED, isolated

The correction (`9cc9e95d`) is on `research/rlt-cstar-005-dic-uptake` only. It subtracts the time-cumulative interior `DIC_source − DIC_ALT_source`, converted to hDIC's units, for `source_role="release"`. Transfers keep their source, which `UpscaledCDR` relies on. The `noF009` composition shows the rest of the work adopts cleanly without it.

### 2.5 Open scientific questions — judgement

1. Does the DIC-difference metric mean the extra **air–sea uptake** (the patch) or the **net DIC stored** in the domain (the pinned behaviour)? For OAE they coincide; for DIC removal they differ by the removed amount.
2. Should the inventory method be referenced to the run's initial state when `DIC_ALT_CO2 ≠ DIC` at the start, as after a restart from a perturbed run or source data carrying their own ALT field (F-018)? The output's first record is the end of the first window, so a correction needs the initial inventory.
3. Should the CaCO₃ dissolution response count as uptake? It is carbon the perturbed run holds without an air–sea flux.
4. Exported perturbation: the Isometric module does not credit uptake outside the domain; the inventory method loses exported carbon that the flux method counts.

## 3. P2 · F-008, the units contract — established

`e7358b79` reduces each unit string to a factor and a dimension vector over amount N, equivalents E, length L, time T and mass M. It accepts only the contract's dimensions: flux N·L⁻²·T⁻¹, inventory N·L⁻², release rate N·T⁻¹ or E·T⁻¹. The parser accepts `^n`, `**n`, `n` and `-n` exponents and space, `*` or `.` separators, and lets `/` divide the next factor only. It refuses parentheses, `//`, leading or trailing `/`, unknown symbols (cm, day, yr, h, mmolC, g C, Pg) and any string without exactly one amount factor.

- Accepted, from ucla-roms 0.16.0 source: `mmol/m^2/s` (MARBL diagnostic units), `meters mmol/m^3` and `meters mMol m-3` (hDIC as `'meters '//t_units`, MARBL metadata or `tracers.F90` L581), `meq/s`, `mmol/s`. With these units every result is identical to the result without units.
- Refused, with the variable named: `umol/kg` as "not amount per area"; concentration where an inventory is expected; inventory where a flux is expected and the reverse; bare amount, area or time; equivalents where carbon is expected; CGS (`nmol/cm^2/s`); other time units; mass; carbon in the symbol; 8 malformed forms.
- Missing or blank units keep the earlier mmol default, with a warning naming the variable. The only behaviour changes are refusals of strings that were previously taken as mmol.
- F-019, established: consistent physics in mixed prefixes gave a flux efficiency of 4.98×10⁻⁷ instead of 0.1 at `ede4fe62`. With the contract it gives exactly the ucla-roms-units result (`f009/mixed_prefix_probe.out`).

`test_cdr_analysis.py`: 76 tests on 003 (19 at `ede4fe62`); analysis directory 144 passed, 18 skipped on 003.

## 4. P3 · Regression evidence

### 4.1 C-Star unit tests — established

Chunked runner (one pytest process per test directory; `suites/run_cstar_chunked.sh`). Pinned: C-Star `0991b445` + roms-tools `ede4fe62`. Patched: C-Star `f0587cb8` + roms-tools composition `ad8bd2b9` (002 at `ae3d658c` + 005 + 004; it differs from `final` only in the partition guide and the stretching).

| | passed | failed | skipped | xfail |
|---|---|---|---|---|
| pinned | 9,970 | 5 | 72 | 1 |
| patched | 10,016 | 4 | 65 | 1 |

`compare_suites.py`: 10,047 common tests; 8 changed outcome, and all 8 pass when rerun on the pinned tree (`suites/cstar_base_rerun8.out`). Seven needed `nccopy`, installed between the two runs (`netcdf-bin`, for P4); one is order-dependent. **No outcome change is attributable to the patches.** 39 tests only in patched: 38 new tests, all passing, plus one parametrization named by a random UUID (F-016); 1 only in pinned, the same UUID case.

The 4 failures common to both trees are environmental (`suites/cstar_common_failures.out`): a GitHub token in this sandbox's environment changes the expected call in 2 domain-catalog tests; a git stale-index behaviour does not reproduce on this filesystem; the network-guard test expects a guard plugin that this environment lacks.

### 4.2 ROMS-Tools full suite — established

Chunked runner (`suites/run_rt_chunked.sh`). Pinned: `ede4fe62`. Final composition: `7bf39df4` (002 + 005 + 006, which contain 003 and 004).

| | passed | failed | error | skipped |
|---|---|---|---|---|
| pinned | 1,683 | 26 | 3 | 164 |
| final | 1,799 | 26 | 3 | 164 |

1,876 common tests; **0 changed outcome**. 116 tests only in final, all passing: 62 in `test_cdr_analysis.py` (units contract, F-009 identity), 16 in `test_vertical_coordinate.py` (θ = 0, stability), 13 uneven partition, 9 partition attribute, 10 whole-coarse join, and 6 in the analytic oracle. No test disappeared.

**Blocked: 29 tidal tests**, identical in both trees: 26 failures and 3 errors in `test_tides.py`, the TPXO case of `test_lat_lon_datasets.py` and `test_validation.py[tidal_forcing]`. They read `global_grid_tpxo10.v2.nc` from `roms-tools-test-data`, which is a Git LFS pointer here (140,111,997 bytes, oid `26eb97cd…e744e`). This session's git proxy serves anonymous reads of public repositories but no LFS objects. None of the research branches touches tidal code.

`noF009` composition (`2493a715`), affected directories (analysis, tiling, vertical coordinate): 230 passed, 18 skipped (`suites/rt_noF009_affected.out`).

## 5. P4 · Cross-repository compatibility — established

ucla-roms 0.16.0 `Tools-Roms/partit.F` and `ncjoin.F` were built unchanged with gfortran 13.3 and netcdf-fortran 4.5.4 (`tools/`, source SHA-256 in `tools/SOURCES.sha256`). The test grid is a ROMS-Tools grid of 37×23 interior points (39×25 rho; coarse 13×20; ETOPO5 topography). It was partitioned into 34 tilings, `np_xi ∈ {1,2,3,4,5,7,8}` × `np_eta ∈ {1,2,3,4,6}`, most of them uneven (`p4/compare_partit.py`):

| Check | pinned `ede4fe62` | `final` composition |
|---|---|---|
| ROMS-Tools `partition(include_coarse_dims=False)` tiles equal `partit`'s (all variables, values, `partition` attribute) | 0/34 (uneven refused) | 34/34 |
| ROMS-Tools `join_datasets` of `partit`'s tiles equals the source | 0/34 (coarse concatenated, F-020) | 34/34 |
| ucla-roms `ncjoin` of `partit`'s tiles equals the source | 34/34 | 34/34 |

`ncjoin` exits with SIGABRT at its final close for 5×6, 7×6 and 8×4 after writing a correct file (F-022). The comparison deletes the source copy before `ncjoin` runs, so it reads only what `ncjoin` wrote.

The partition guide (`a1c52ff7`) was checked by running its calls on a local 300×150 ROMS-Tools grid (`p4/notebook_claims.out`): coarse shares 10, 13, …, 13, 10 and 12, 13, 13, 13, 13, 11; 72 tiles with and without coarse dimensions, both joining back to the source; ROMS's coarse limit from `np_xi = 75`; first unsplittable coarse tiling `np_xi = 21`.

**F-021.** `init_coarse`'s verbatim lines, compiled (`coarse/`), pass 48 values of `np_xi` for `llm = 300` whose edge coarse tiles hold 2 points or fewer, and 13,467 tile cases with 2 points or fewer for `llm ≤ 300`, 11,174 of them empty (`coarse/coarse_summary.out`). At `np_xi = 21` the edge tiles hold −1. RLT certificate G08 (kernel ACCEPT; a one-digit tampering is REJECTed) and Lean (`guard_passes_edge_empty`, `guard_alone_insufficient`, `ruleL_tiles`) state the case and what the amended Rule L guarantees: every tile non-empty, every middle tile at least 3 points. Consequently, for every tiling the amended Rule L accepts, ROMS-Tools splits the coarse grid and C-Star's retry without coarse dimensions is never reached for coarse-forcing runs: 25,445 accepted tilings (`llm` 2…800, 1…128 tiles), none refused by ROMS-Tools (`p4/ruleL_vs_partition.out`).

**Version dependency (inferred, with the executed F-001 evidence).** C-Star requires `roms_tools>=5.4.0,<6` and accepts uneven tilings in Forge, while `ROMSInputDataset.partition` calls `roms_tools.partition_netcdf`. With any released ROMS-Tools that splits only evenly, a `use_pio=False` run on an uneven tiling fails at `pre_run`; with the research composition it partitions as ucla-roms does. Recommended order: release ROMS-Tools with 002, then raise C-Star's floor to that release. Until then the combination stays incompatible for uneven tilings. Backward compatibility: tiles written by the research branch carry the extra global `partition` attribute, which ucla-roms' own `partit` writes too. Files without the attribute still join by size inference. Even tilings that the pinned tree accepts give tiles with identical variables, values and attributes apart from the added `partition` attribute (6 tilings up to 264 tiles, `p4/even_identity.out`). Four even fine tilings that the pinned tree refused on their coarse dimensions now partition.

## 6. P5 · Further improvements — established

**F-011, stretching function.** Measured on 8,748 points (θ_s ∈ 12 values in [0, 10], θ_b ∈ 9 values in [0, 10], 81 σ levels) against 50-digit mpmath (`f011/`):

| | NaN | max error | max error, θ_s ≥ 0.5 |
|---|---|---|---|
| ROMS-Tools `23a7b518` | 1,458 | 2.8×10⁻³ | 7.1×10⁻⁷ |
| ucla-roms 0.16.0 `CSF` (verbatim, -O2) | 1,458 | 2.0×10⁻³ | 6.1×10⁻⁷ |
| ROMS-Tools `e24817f4` | 0 | 6.7×10⁻¹⁶ | 6.7×10⁻¹⁶ |
| ucla-roms `0c63d80d` `CSF` | 0 | 7.8×10⁻¹⁶ | 6.7×10⁻¹⁶ |

The two changed codes agree to ≤ 10⁻¹⁵ on every point; the pinned pair differ by up to 4.4×10⁻³. Mixed versions differ by ≤ 7.6×10⁻¹⁵ for θ_s ≥ 0.5 and θ_b = 0 or ≥ 0.1, well inside ROMS-Tools' `np.allclose` check of `Cs_w` against ROMS output. The real `set_scoord_mod.F90`, compiled with stand-in modules (`f011/modcompile`), fills `Cs_w` with NaN at θ_s = 10⁻⁸ before the change and the correct curve after. ROMS-Tools tests: 20 in `test_vertical_coordinate.py`, 6 of them failing without the change.

**Provenance and reproducibility.** Each suite log records tree path and commit. The ucla-roms tools are built out of tree from recorded source hashes. The archive carries `git format-patch` series and bundles for every branch and a SHA-256 manifest. F-023 records the version-attribute caveat.

**RLT.** Kernel unchanged (`git diff 6128c97 -- kernel` empty). New: certificate G08 (`research/rlt-cstar-007/`), and `lean/RLTCSTAR007.lean` with 5 theorems (`#print axioms`: none, or only propext, Classical.choice, Quot.sound).

## 7. Summary

| | Established | Inferred | Judgement for [C]Worthy |
|---|---|---|---|
| F-009 | pinned DIC-based uptake departs from the analytic uptake for DIC removal; the patch restores agreement; export and CaCO₃ terms remain | intended metric is extra air–sea uptake | which quantity the metric reports; initial-state reference; CaCO₃ and export treatment |
| F-008/F-019 | contract refuses every other dimension; mixed prefixes now give correct efficiencies | the strings ucla-roms writes (from source) | — |
| F-020 | join copies whole coarse grids, matching `ncjoin` on 34 tilings | — | — |
| F-021 | ucla-roms' coarse guard passes empty edge tiles; C-Star now refuses them | — | edge tiles of 1–2 coarse points (for ucla-roms) |
| F-011 | stable form accurate to 7.8×10⁻¹⁶ in both codes, NaN removed | — | adopting the change in ucla-roms |
| Suites | C-Star (10,047 common tests) and ROMS-Tools (1,876): no outcome change from the patches; 38 and 116 new tests pass; 29 tidal tests blocked (LFS data) in both trees | — | — |

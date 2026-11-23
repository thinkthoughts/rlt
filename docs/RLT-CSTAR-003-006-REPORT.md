# RLT-CSTAR-003 to 006 — CO₂ bookkeeping, vertical coordinates, carbon uptake, provenance

Pins as in `RLT-CSTAR-FINDINGS.md`. Milestones kept their planned scope; one change of order is recorded in §5.

## RLT-CSTAR-003 — CO₂ bookkeeping (roms-tools `research/rlt-cstar-003`, based on `ede4fe62`)

**Molar mass.** `_MOLAR_MASS_CO2_KG_PER_MOL = 44.0095e-3` equals exactly 12.0107 + 2 × 15.9994 g/mol, the IUPAC 2007 standard atomic weights (exact rational check). CIAAW's current abridged values (C 12.011 ± 0.002, O 15.999 ± 0.001) give 44.009 ± 0.004, which includes 44.0095. The river code's C 12.011 differs from the 2007 carbon value by 2.5 × 10⁻⁵ relative. Both are immaterial at the stated uncertainty. Commit `755fef69` names the source in the comment and leaves the value, so reported quantities are unchanged (F-007).

**Independent expected value.** One mmol of carbon counted as one mmol of CO₂ is 44.0095 × 10⁻⁹ t = 88019/(2 × 10¹²) t exactly. The new test compares the function's scale with that value for the unit strings ucla-roms writes (agreement within 10⁻¹⁵ relative; float error measured 2.5 × 10⁻¹⁷). The existing analytic test derived its expectation from the function under test (F-006). The one-carbon-per-CO₂ count is RLT's registered CHEM-MOLFORMULA reading of CO₂ (C×1 O×2), already kernel-counted in RLT T7.

**Units.** ucla-roms initialises MARBL with `unit_system_opt='mks'` (`marbl_driver.F90` L360). MARBL's MKS labels are `mmol/m^3` and `mmol/m^2/s`, and its CGS labels `nmol/cm^3` and `nmol/cm^2/s` (`marbl_settings_mod.F90` L2268-2303). ucla-roms labels `hDIC` as `'meters ' // t_units(iDIC)` (`cdr_output.F90`). The parser previously took any unrecognised unit as mmol, with a warning: a CGS flux gave uptake 100× too large and `nmol/m^2/s` 10⁶× too large. Commit `b177f818` refuses per-centimetre and non-amount units and converts nmol (F-008). ucla-roms output converts as before.

## RLT-CSTAR-004 — Vertical coordinates (roms-tools `research/rlt-cstar-004`, based on `ede4fe62`)

| Item | Finding | Evidence |
|---|---|---|
| Stretching formula | ROMS-Tools `compute_cs` equals ucla-roms `CSF` for θ > 0 | compiled verbatim `CSF`: 0 difference at θ_s ∈ {0, 5}, ≤ 2.2 × 10⁻¹⁶ at (10, 10) |
| Parameter domain | ucla-roms accepts θ_s = 0 (`−σ²`) and θ_b = 0 (no bottom refinement); ROMS-Tools refused both | corrected in `23a7b518` (F-010) |
| Upper bound | ucla-roms `scoord.F90` L33 comment says `0 =< theta_s < 10`; neither code enforces the upper end; ROMS-Tools accepts 10 | source reading; no change |
| Endpoints | ucla-roms assigns `Cs_w(0) = −1`, `Cs_w(N) = 0`; ROMS-Tools computes them, exactly in float64 for 10,000 tested pairs and the new θ = 0 branches | numerical test (RLT-CSTAR-001) plus new tests |
| Depth formula | `compute_depth` = ucla-roms `set_depth` (`z = ζ + (ζ+h)(h_c σ + h C)/(h_c + h)`), returned positive-down | source reading |
| Conditioning | the θ_s step loses about 4 × 10⁻¹⁶/θ_s² to cancellation in both codes | measured against `−sinh²(θσ/2)/sinh²(θ/2)`; proposed jointly (F-011) |

The θ → 0 limits are standard real analysis and outside RLT's signature (L1) and core Lean without Mathlib; they are covered here by tests against compiled ucla-roms.

## RLT-CSTAR-005 — Carbon uptake and conservation (in `research/rlt-cstar-003`, commit `a6644589`)

**Identity.** ucla-roms adds each tracer's own release (`step3d_t_mod.F90` L884-930; with 3-D forcing only to ALK and DIC). ROMS-Tools releases fill unspecified tracer fluxes, including the `*_ALT_CO2` tracers, with zero (`cdr_release.py`). Where nothing crosses the interior's sides,

  d/dt Σ (hDIC − hDIC_ALT_CO2) A = Σ (FG_CO2 − FG_ALT_CO2) A + Σ (DIC_source − DIC_ALT_source) dV.

The two uptake estimates of `compute_cdr_metrics` therefore differ, for a closed budget, by the cumulative release of DIC. For alkalinity-only releases that is zero. For a DIC removal (`DIC_source < 0`, allowed by the release sign check) the DIC-based efficiency was the flux-based one minus the removed fraction (closed-budget test: 0.75 vs −0.25). The correction adds the release's DIC back for `source_role="release"` only. A transfer domain's source is the child's exported perturbation, which the child's own inventory no longer holds, and `UpscaledCDR`'s sum over domains relies on that (F-009).

**Kinds of evidence.** The identity is a conservation statement about the model's tracer equation; the test is a synthetic closed budget. Whether real output closes (lateral export, sediment, numerical mixing) is model validation, untested here: no ucla-roms CDR output is in the available test data. The flux-vs-DIC agreement remains a logged diagnostic, not a pass/fail test. This changes a reported quantity for DIC-removing releases and is flagged for review by the method's authors.

## RLT-CSTAR-006 — Scientific provenance

| Item | Source fidelity | Status |
|---|---|---|
| 44.0095 g/mol | IUPAC 2007 table, exact | cited (F-007) |
| Stretching (SM2009 Eq. 2.4) | via ucla-roms comment; paper not inspected | secondary |
| DON/DOP from DOC_sl: 103/2583, 1/2583 | RivR2O (Zenodo v13) gives C:N 2583:103 and C:P 2583:1 for **labile** DOM, which it routes to DIC and DIN; ROMS-Tools applies them to semi-labile DOC | unresolved (F-012) |
| DON/DOP from POC: 25/276, 1/276 | not in the RivR2O record; the record says recycled POC belongs in the semi-refractory DOC pool, "see protocol" | unresolved (F-012) |
| River ALK = file DIC | documented assumption in `Rivr2oRiverBGCDataset` docstring | recorded |
| `R_EARTH = 6371315 m`, `RHO0 = 1025` | uncited in file; conventional | recorded |

## Ordering

RLT-CSTAR-005 was carried out inside the 003 branch, because the uptake definition and the units sit in the same function and its tests. Its commit stays separate (`a6644589`), so it can be reviewed or dropped on its own.

## Phase 5 — measurements

Benchmarks on the shared 2-core VM (`research/rlt-cstar-002/verification/bench_partition.py`):

| Measure | Pinned `ede4fe62` | Branch `research/rlt-cstar-002` |
|---|---|---|
| `partition` of a 960×960 rho/u/v dataset into 16×20 tiles, median of 5 interleaved runs of 20 calls | 0.295 s | 0.317 s (≈ +1 ms per call; attribute per tile; within run-to-run spread 0.30-0.47 s) |
| `partition_netcdf` of a real 10×10 ROMS-Tools grid into 2×5 tiles, C-Star's strategy | 311,755 bytes (retry: whole coarse grid copied into 10 tiles) | 293,132 bytes (coarse grid partitioned), −6.0% |

Early refusal (F-004) moves a failure from ROMS start-up, after compilation, input staging and any queue wait, to blueprint check or namelist composition. No compute or energy saving is claimed, since none was measured. The proxy is the stage at which the error appears.

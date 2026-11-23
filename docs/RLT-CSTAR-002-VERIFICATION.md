# RLT-CSTAR-002 — Verification report

## 0. Inputs and environment

- Prior evidence, unchanged: `RLT-CSTAR-001-ASSESSMENT.md` SHA-256 `1d92cfa579c3511f82a443f8f69d878afa8705d831010467326ad380493feef1`; `RLT-CSTAR-001-evidence.tar.gz` SHA-256 `af50dd0aa7128b8c3921484a017fa6813d133c9ea5153acd8a4881120e8d893b`.
- Repositories: §1 of `RLT-CSTAR-002-SPEC.md`. Fork identities checked with `git ls-remote` against CWorthy-ocean (C-Star and roms-tools forks equal upstream HEAD; thinkthoughts/ucla-roms `main` = upstream `main` `bf7b5f4d`, with tag 0.16.0 `f83d2ef6` as an ancestor; the fork carries no tags, so `0.16.0` was fetched from upstream and its object id confirmed).
- Ubuntu 24.04.5, 2 cores, 7.8 GB RAM; Python 3.13; GNU Fortran 13.3.0; Lean 4 v4.34.0-rc2 (official release archive, SHA-256 `3d011041…f1a8`); Rust 1.97.0 (RLT's `rust-toolchain.toml`).
- ROMS-Tools test data: `CWorthy-ocean/roms-tools-data` and `roms-tools-test-data` cloned with git (the HTTPS raw downloads pooch uses are blocked here) and linked into pooch's cache, where pooch verifies every file's registered hash.

## 1. Phase 1 — reproduction of RLT-CSTAR-001

| 001 claim | Re-check | Result |
|---|---|---|
| C-Star accepts 10 points on 3 ranks; ROMS-Tools refuses | real C-Star `check_tile_sizes` and `ROMSModelGrid.get()/.partition()` on a real 10×10 ROMS-Tools grid (ETOPO5 topography) | reproduced: Forge accepts; `partition` raises `ValueError` for (3,1) and (4,4); (2,5) "succeeds" only by C-Star's retry without coarse dimensions |
| `_edge_tile_width` = ucla-roms `mpi_setup` | 001 used a Python transcription; now compiled verbatim Fortran | reproduced: 0 mismatches on 19,200 pairs |
| failure is loud | `pre_run` raises before ROMS starts | reproduced |
| reachable configuration | `ROMSSimulation.io_layout` is `PARTITIONED` exactly when `use_pio` is False (`simulation.py` L808-819); then `pre_run` partitions every staged partitionable input (L2892-2925); every bundled `roms-marbl-*-default` and `pio-dev` ModelSpec sets `use_pio: true`; `cson_roms-marbl_v0.1` sets `use_pio: false` (the wizard's PIO checkbox default) | precise condition: `use_pio=False`, not `auto_tiling`, and `llm % np_xi != 0` or `mmm % np_eta != 0` (or the same for the coarse grid, which the retry hides) |
| intended vs defect | ucla-roms `mpi_setup` and its own partitioner `partit.F` split unevenly by design; ROMS-Tools documented its narrower rule (`partition` docstring, `docs/partition.ipynb`) | an inter-repository contract gap, not a ROMS-Tools bug by its own documentation; resolved by option A |
| CO₂ constant, units, self-referential test (001 F8-F10) | re-executed | reproduced; extended: CGS units give ×100 |
| ROMS-Tools tiling tests errored (001 §2.2) | test data now available | 57 passed at `ede4fe62` |

New in this milestone: F-003 (join ambiguity), F-004 (coarse tiles), F-005 (retry copies coarse grids), F-009 (DIC uptake for DIC removal).

## 2. Executable verification of the tiling rule

`research/rlt-cstar-002/verification/reproduce.sh UCLA_ROMS LEAN OUT` extracts and hash-checks the Fortran, compiles the harnesses and runs every comparison. Its output (this environment):

```
pairs 19200, valid 8098
F1!=F2 widths 0      (mpi_setup vs partit.F)
F1!=C edge 0         (mpi_setup vs C-Star _edge_tile_width)
F1!=C accept 0       (mpi_setup vs C-Star check_tile_sizes)
F1!=R xi_rho 0       (mpi_setup vs ROMS-Tools, research/rlt-cstar-002)
F1!=R xi_u 0
F3!=R coarse 0       (init_coarse vs ROMS-Tools)
R accepts invalid 0; R accepts invalid coarse 0
C-Star accepts, ucla-roms aborts if interp_frc: 998
round trip (rho/u/v): 14884 partitions, 0 failed
round trip (rho only): 14884 partitions, 0 failed
valid tilings 8098: Lean (width, start) differs from compiled Fortran in 0
invalid tilings 11102: same classification in 11102
```

At `ede4fe62` the rho-only round trips fail for 696 of 3,364 even tilings (uneven ones were refused).

## 3. Test suites

Each tree was run single-process with `pytest -v`; outcomes are compared per test id (`verification/compare_suites.py`). Logs: review bundle `evidence/suites/`.

**ROMS-Tools**, full suite, pinned `ede4fe62` vs a scratch integration of `research/rlt-cstar-002`, `-003` and `-004` (local merges, on no branch):

| | passed | failed | errors | skipped |
|---|---|---|---|---|
| pinned | 1,683 | 26 | 3 | 164 |
| integration | 1,733 | 26 | 3 | 164 |

Outcome changed in 0 of 1,876 common tests; the 50 tests only in the integration (22 tiling, 17 CDR analysis, 11 vertical coordinate) pass. The 29 failures and errors in both are TPXO tidal tests: `global_grid_tpxo10.v2.nc` is a 140 MB Git LFS object that was not fetched, so the test-data checkout holds its 134-byte pointer. 164 skips need `xesmf` and other optional packages. Per branch, the touched test directories also pass on their own: tiling 56/56 (`-002`), analysis 104 passed, 18 skipped (`-003`), vertical coordinate and grid 92 passed, 1 skipped (`-004`).

**C-Star**, unit tests, pinned `0991b445` vs `research/rlt-cstar-002`. A single process exceeds this VM's 7.8 GB memory cgroup in `wizard/test_workplan_builder.py`, the last file, in both trees (exit 137); that file was then run alone in each tree (131 passed in both).

| | passed | failed | errors | skipped |
|---|---|---|---|---|
| pinned | 9,212 | 442 | 322 | 72 |
| branch, before the fixture fix | 9,238 | 445 | 322 | 72 |

Outcome changed in 3 of 10,047 common tests, all in `applications/forge/test_derived_overrides.py`: its toy fixture (3 × 4 points on 2 × 2 tiles) switches coarse-grid forcing on, which ucla-roms stops on, and the new refusal pre-empted the refusal those tests check. The amended commit `f4ba79ff` enlarges that fixture to 12 × 12 with the same tiling; the file then passes 19/19 on the branch and on the pinned tree. All 30 new tests pass (29 for F-004, counted with parametrisations, plus one whose id contains a random UUID and so appears as added and removed). The 764 failures and errors present in both trees concentrate in asynchronous orchestration tests (`orchestration/test_dag_runner.py`, `launch/local`, `launch/slurm`, `cli/workplan`, `entrypoint`); pytest reports `Unknown config option: asyncio_mode`, so the pytest-asyncio plugin these tests need is not installed here. That code is untouched by the branch, but its regressions would be masked in this environment.

`pytest-xdist` cannot run the C-Star suite: workers collect different test ids (F-016).

## 4. Proofs

- RLT (`research/rlt-cstar-002/certificates/MANIFEST.tsv`): 16 certificates ACCEPT under ROBINSON-Q, Γ = ∅; `C_dvd_16_512` MALFORMED (term-node limit), unchecked. Regeneration is byte-identical. Kernel, axioms, rules and limits unchanged (`git diff 6128c97 -- kernel spec cli syntax` empty).
- Lean (`lean/RLTCSTAR002.lean`): compiles with core Lean v4.34.0-rc2; no `sorry`; `#print axioms` lists only `propext`, `Classical.choice`, `Quot.sound` (`ten_on_three`: `propext`; `twenty_on_eight`: none).

## 5. Benchmarks

See `RLT-CSTAR-003-006-REPORT.md`, Phase 5.

## 6. What this verification does not establish

- That ucla-roms runs correctly on uneven tilings: the model was not built or run (no MPI/netCDF-Fortran toolchain here). The evidence is its own decomposition code, compiled, and its own partitioner.
- That the Lean theorems hold for all inputs of the Python and Fortran: the executable tie covers `p <= 300`, `t <= 64`.
- Physical correctness of anything: tiling has no physical content.

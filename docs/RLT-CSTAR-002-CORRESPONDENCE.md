# RLT-CSTAR-002 — Correspondence record

Each row connects a source statement to its formal statement and to the executable check that ties the two together. Pins as in `RLT-CSTAR-002-SPEC.md` §1. "Verbatim" means the Fortran lines are extracted by line range from the pinned commit, hash-checked and compiled unchanged (`research/rlt-cstar-002/verification/reproduce.sh`); only the surrounding declarations and loops are written by hand.

## 1. Tile widths and starts

| Source (pinned) | Formula | RLT | Lean (`lean/RLTCSTAR002.lean`) | Executable tie |
|---|---|---|---|---|
| ucla-roms `src/roms_mpi.F90` L226-227 | `nx = ceiling(1.0_8*gnx/npx)` | G01 (10 on 3), G03 (20 on 8) | `ceilDiv`, `ceilDiv_bounds` | `harness_mpi` (verbatim L226-227, L243-300) |
| `roms_mpi.F90` L243-262 | `offx = npx*nx - gnx`; first `nx - offx/2`; last `nx - (offx+1)/2`; `iSW_corn = inode*nx - offx/2` | G02 | `surplus`, `firstW`, `lastW`, `start`; `cover`, `start_*` | Lean `#eval` table = `harness_mpi` table on 19,200 pairs (`compare_lean_fortran.py`) |
| `roms_mpi.F90` L264-300 | file sizes: `xi_rho = nx` +1 at each physical edge, `xi_u = nx` +1 at the east edge | — | — | `harness_mpi` vs ROMS-Tools `_compute_partition_sizes(p+2, t, 2, "both")` and `(p+1, t, 1, "upper")` |
| ucla-roms `Tools-Roms/partit.F` L1493-1597 | the same rule for the reference partitioner | — | — | `harness_partit` (verbatim subroutine) widths = `harness_mpi` widths |
| C-Star `cstar/applications/forge/namelist_model.py` `_edge_tile_width` | `width - (surplus + 1) // 2` with `width = -(-points // tiles)` | — | `lastW` (S3 `lastW_pos_iff`) | imported and called on all pairs: = min width of `harness_mpi` |
| C-Star `check_tile_sizes` | refuse iff edge width < 1 | — | S3 | imported: accepts exactly the pairs `harness_mpi` gives non-empty tiles |
| ROMS-Tools `roms_tools/tiling/partition.py` `_interior_widths` (branch `research/rlt-cstar-002`) | the rule of §2, refusing an empty tile | — | `firstW`, `lastW`, `ceilDiv` | imported: per-tile sizes = `harness_mpi` / `harness_coarse` on all valid pairs; refuses all invalid pairs |

## 2. Coarse grid

| Source | Formula | RLT | Lean | Executable tie |
|---|---|---|---|---|
| ucla-roms `src/roms_read_write.F90` L545-548, L564-575, L590-596 | `gnxc = gnx/2`; `nxc = (gnxc+NP_xi-1)/np_xi`; same surplus rule; `ic0/ic1` ghost points | G04, G05 | `ceilDiv` on `p / 2` | `harness_coarse` (verbatim) = ROMS-Tools coarse sizes |
| `roms_read_write.F90` L550-562 | stop if `nxc <= 2` or `nyc <= 2` (evaluated, not compiled: it calls ucla-roms' error log) | G04 (width 2), G05 (width 3) | `coarse_ok_iff`, `twenty_on_eight` | `harness_coarse` prints the ceiling width; C-Star `coarse_tile_width` imported |
| C-Star `cstar/roms/precheck.py` Rule L, `check_coarse_tile_sizes` (branch `research/rlt-cstar-002`) | refuse iff a compiled module interpolates and `ceil((llm // 2)/np_xi) <= 2` (likewise eta) | — | `coarse_ok_iff` | unit tests; 998 pairs in range accepted by the old check and stopped by ucla-roms |
| ucla-roms `src/init_arrays.F90` L140-142, L164-166, L170-174 | bulk forcing `#ifdef BULK_FRC`, flux forcing `#ifndef BULK_FRC`, BGC forcing with MARBL or BEC2 | — | — | source reading; Rule L reads `bulk_frc`, `marbl`, `biology_bec2` from the staged `cppdefs.opt` |
| `src/bulk_frc.F90` L118-125, `src/flux_frc.F90` L58-64, `src/bgc_io.F90` L1013-1037 | `interp_*_frc` sets `%coarse = 1`, which triggers `init_coarse` at the first read (`roms_read_write.F90` L2073-2089) | — | — | source reading |

## 3. Partition attribute and join

| Source | Statement | RLT | Executable tie |
|---|---|---|---|
| ucla-roms `src/roms_read_write.F90` L2603-2634 | `partition = (mynode, nnodes, iSW_corn+iwest[+1], jSW_corn+jsouth[+1])` | — | ROMS-Tools test: 10 on 3 gives starts 1, 5, 9 (= `iSW_corn` 0, 3, 7 plus ghost offset) |
| `Tools-Roms/partit.F` L473-480 | the same attribute on partitioned input files | — | source reading |
| ROMS-Tools `join.py` `_infer_partition_layout_from_datasets` (pinned) | layout from the first eta-size change | G06 (the ambiguous sizes) | 696 of 3,364 even rho-only round trips wrong at `ede4fe6`; 0 of 14,884 after the change |

## 4. Status, by layer

| Layer | Status |
|---|---|
| Proof validity | RLT: 16 certificates ACCEPT under ROBINSON-Q (Γ = ∅); `16 ∣ 512` MALFORMED (term-node limit), unchecked. Lean: 14 theorems, no `sorry`, axioms `propext`, `Classical.choice`, `Quot.sound` only |
| Source fidelity | Fortran: verbatim excerpts compiled (GNU Fortran 13.3.0, Ubuntu 24.04), except the `init_coarse` stop (L550-562), evaluated by its condition. Lean and RLT statements: hand transcriptions, tied to the compiled excerpts by the 19,200-pair comparison |
| Executable agreement | 0 mismatches on 19,200 pairs among compiled `mpi_setup`, compiled `partit.F`, compiled `init_coarse`, C-Star and patched ROMS-Tools; 29,768 partition/join round trips exact |
| Numerical reproducibility | integer arithmetic only; RLT certificates regenerate byte-identically |
| Physical correspondence | none claimed: domain decomposition carries no physical content; it decides which grid points each rank's input files hold |

A Lean proof about `lastW` says nothing by itself about C-Star's `_edge_tile_width`; the tie is the executed comparison, which covers `p <= 300`, `t <= 64` (and `p <= 1000`, `t <= 1000` for C-Star against the earlier transcription in RLT-CSTAR-001).

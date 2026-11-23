# RLT-CSTAR-002 — Specification: MPI domain decomposition across ucla-roms, C-Star and ROMS-Tools

Status: implemented on research branches, unmerged and unpushed, awaiting independent review.
Prior evidence: `RLT-CSTAR-001-ASSESSMENT.md` and `RLT-CSTAR-001-evidence.tar.gz`, preserved unchanged (SHA-256 in `RLT-CSTAR-002-VERIFICATION.md` §0).

## 1. Pinned sources

| Repository | Pin | Role here |
|---|---|---|
| CWorthy-ocean/ucla-roms (fork thinkthoughts/ucla-roms) | tag `0.16.0` = `f83d2ef6cbb0f7391e17dc6eda54673032859db1` | the written and executable specification of intended behaviour |
| CWorthy-ocean/C-Star (fork thinkthoughts/C-Star) | `0991b445db566457886faacd0ae210cd76ca67ed` | early refusal of tilings; drives partitioning of input files |
| CWorthy-ocean/roms-tools (fork thinkthoughts/roms-tools) | `ede4fe6282c501e0b7d13dcc86b4b10e09609f47` | partitions and joins per-rank netCDF files |
| thinkthoughts/rlt | `6128c974db184ecbac553d2e5e583a2c0e1b3125` | kernel certificates, Lean companion, records |

The thinkthoughts forks of C-Star and roms-tools equal upstream HEAD at these commits; thinkthoughts/ucla-roms `main` (`bf7b5f4d`) has `f83d2ef6` as an ancestor, and `src/roms_mpi.F90` is byte-identical between the two.

## 2. Intended behaviour (the specification)

Written and executable sources in ucla-roms 0.16.0 establish one rule for splitting `p` interior points of a grid axis into `t` tiles:

- `src/roms_mpi.F90` `mpi_setup`, L220-262: `nx = ceiling(gnx/npx)` (checked against `(gnx+npx-1)/npx`), `offx = npx*nx - gnx`; tile 0 has `nx - offx/2` points, the last tile `nx - (offx+1)/2`, the others `nx`; tile `k > 0` starts after `iSW_corn = k*nx - offx/2` points. The comment: "If the surplus (offx,offy) is odd, the last subdomain will be the smallest."
- `Tools-Roms/partit.F` `mpi_setup`, L1493-1597, ucla-roms' reference partitioner for input files: the same rule ("maps exactly onto logic inside 'mpi_setup' in the actual ROMS code").
- `src/roms_read_write.F90` `init_coarse`, L528-684: the same rule on the coarse grid of `gnx/2` points, and a stop when `ceiling((gnx/2)/np_xi) <= 2` ("tiling too fine for coarse-resolution forcing inputs").
- Per-rank file sizes, `roms_mpi.F90` L264-300: one rho ghost point at each physical boundary, one u/v point at the east/north end.
- Per-rank files carry the global attribute `partition = (node, nnodes, xi_start, eta_start)` (`roms_read_write.F90` L2603-2634; `partit.F` L473-480), which ucla-roms' `ncjoin` reads.

Uneven tilings are therefore intended behaviour of the model and of its own partitioner.

## 3. Requirements

| ID | Requirement | Repository |
|---|---|---|
| R1 | `partition` splits every partitioned dimension by the rule of §2, giving per-rank files whose sizes equal ucla-roms' for every tiling that leaves no tile empty | ROMS-Tools |
| R2 | Tilings that divide evenly give the same files as before | ROMS-Tools |
| R3 | A tiling that leaves a tile with no interior points is refused, naming the interior point count | ROMS-Tools |
| R4 | Partitioned files carry ucla-roms' `partition` attribute; `join_datasets` places tiles by it and reproduces the original for rho-only files too | ROMS-Tools |
| R5 | C-Star refuses, before build and queue, a tiling that ucla-roms' `init_coarse` stops on, only when a compiled forcing module has its interpolation switch on, never under auto-tiling, and never when the build is unknown | C-Star |
| R6 | Forge's early refusal applies R5 when `blk_frc.interp_frc` or `bgc.interp_frc` is set | C-Star |
| R7 | RLT kernel, axiom sets, rules and limits unchanged | RLT |

## 4. Mathematical statements

For `p >= 1` points and `t >= 1` tiles, `w = ceil(p/t)`, `s = t*w - p`:

| ID | Statement | Where proved |
|---|---|---|
| S1 | `p <= t*w < p + t`, hence `s < t` | Lean `ceilDiv_bounds`, `surplus_lt` |
| S2 | `t | p` implies `s = 0` and `w = p / t` | Lean `even_split` |
| S3 | the last tile is non-empty iff `s + 2 <= 2w` (C-Star's acceptance condition) | Lean `lastW_pos_iff` |
| S4 | last tile <= first tile and <= middle tiles | Lean `lastW_le_firstW`, `lastW_le_mid` |
| S5 | valid tilings cover `p`: `(w - s/2) + (t-2)w + (w - (s+1)/2) = p` | Lean `cover` |
| S6 | tiles are contiguous: `start 1 = first`, `start (k+1) = start k + w`, `start (t-1) + last = p` | Lean `start_one`, `start_succ`, `start_last` |
| S7 | coarse rule: `3 <= ceil(q/t)` iff `2t < q` | Lean `coarse_ok_iff` |
| S8 | 10 on 3: `ceil = 4`, `offx = 2`, tiles 3, 4, 3, starts 0, 3, 7, cover 10; `not (3 | 10)` | RLT G01, G02, G07; Lean `ten_on_three` |
| S9 | 20 on 8: fine tiles valid (last = 1), coarse width 2; 20 on 4: coarse width 3 | RLT G03, G04, G05; Lean `twenty_on_eight` |
| S10 | rho-only, 2 interior rows on 2 tiles: both tile rows have `eta_rho = 2` | RLT G06 |
| S11 | bundled DomainSpec splits divide evenly | RLT `C_dvd_*` (9 of 10; `16 | 512` exceeds the term-node limit) |

Lean statements are about hand transcriptions of the formulas. Their correspondence with the Fortran and Python is established by executable evidence (§5 of the correspondence record), never by the proofs themselves.

## 5. Out of scope, recorded

- Changing C-Star to refuse uneven tilings (option B, §6): rejected per the task's rule against restricting ucla-roms configurations because ROMS-Tools was narrower.
- `docs/partition.ipynb` (ROMS-Tools) states the divisibility requirement and shows an executed traceback; it needs re-running on its original data.
- C-Star's `partition` retry without coarse dimensions writes whole coarse-grid variables into each tile file. ucla-roms 0.16.0 never reads `*_coarse` grid variables, so this wastes storage only.

## 6. Options considered

| Option | Effect | Decision |
|---|---|---|
| A. ROMS-Tools follows ucla-roms' rule | uneven tilings partition correctly; even tilings unchanged; C-Star unchanged | implemented (R1-R3) |
| B. C-Star refuses uneven tilings for the PARTITIONED layout | removes the failure by restricting valid model configurations | rejected |
| C. Both A and a C-Star version guard requiring a ROMS-Tools release with A | protects C-Star with an older ROMS-Tools | recommended once A is released |

# [roms-tools] Partition uneven tilings as ucla-roms does; join by the `partition` attribute; copy whole coarse dimensions

Branch (fork, proposed): `rlt-cstar/pr-tiling` `a45d6be3` on `main` `ede4fe62` — 5 commits (`b997c5d`, `4254d8a`, `365b591`, `a346349`, `a45d6be`).

# Summary

Three changes to `roms_tools/tiling/` that make ROMS-Tools' split and join agree with ucla-roms' own `mpi_setup`/`partit` and `ncjoin`, plus the partition guide.

- **Uneven tilings** (`partition.py`). `partition` refused tilings whose tile count does not divide the interior (e.g. 10 points on 3 ranks), although ucla-roms runs them. C-Star accepts such tilings, so a `use_pio=False` run that passed Forge failed at `pre_run`. Tiles are now split by ucla-roms' rule: `w = ceil(p/t)`, surplus `s = t·w − p` taken as `s//2` from the first tile and `(s+1)//2` from the last. The refusal message, where one remains, names the interior point count.
- **`partition` attribute** (`partition.py`, `join.py`). The join inferred the layout from eta-size changes, which is ambiguous for rho-only tiles (696 of 3,364 even rho-only splits joined wrongly at `ede4fe62`). Tiles are now written with ucla-roms' `partition` attribute and placed by it; files without it keep the old inference.
- **Whole coarse dimensions** (`join.py`). Coarse-grid variables that every tile holds whole (as `partit` writes them, or `partition(include_coarse_dims=False)`) were concatenated: a 13×20 `angle_coarse` came back 26×60 from 3×2 tiles. They are now copied when every one of ≥ 2 tiles has the whole size `(n − 2)//2 + 2`, as `ncjoin` does.
- **Guide** (`docs/partition.ipynb`): describes the splitting rule, the attribute, ROMS's and ROMS-Tools' coarse limits and the whole-coarse join; stale error output removed. The cells were not re-executed (they need NERSC paths and ERA5); every number in the new text comes from a run.

How it was tested:
- 19,200 (points, tiles) pairs against compiled ucla-roms `mpi_setup`/`partit.F`/`init_coarse`: 0 mismatches.
- 14,884 rho/u/v round trips exact, including through netCDF.
- 34 tilings of a 37×23 grid (and 41×29): ROMS-Tools tiles equal `partit`'s, values and `partition` attribute; both joins equal the source.
- New tests: `test_partition_uneven.py`, `test_partition_attribute.py`, `test_join_coarse.py`; 6 of the coarse-join tests fail without the change.
- `pytest roms_tools/tests/test_tiling`: 66 passed; `pre-commit` on the changed files: passed.


Release notes: `+uneven-tiling.feature.md`, `+partition-attribute.bugfix.md`, `+join-whole-coarse.bugfix.md`.

Downstream: C-Star accepts uneven tilings today; with roms-tools ≤ 5.4.0 those fail at partition time (RLT-CSTAR F-024). Releasing this first lets C-Star raise its `roms_tools` floor.

Formal verification (independent of the code; RLT-CSTAR records):
- Lean 4 (`lean/RLTCSTAR002.lean`, `lean/RLTCSTAR007.lean` in thinkthoughts/rlt, no `sorry`, standard axioms only): `ceilDiv_bounds`, `surplus_lt`, `even_split` (a dividing tile count gives equal tiles with no surplus), `lastW_pos_iff` (the last tile is non-empty exactly when the surplus is at most 2·ceil − 2), `lastW_le_firstW`, `cover` (first + middle + last widths add up to the interior), and `split_never_all_whole` (with ≥ 2 tiles the per-tile coarse sizes, halos included, sum to the whole size, which is smaller than the tile count times the whole size, so a whole-size coarse dimension in every tile can only come from copying; this makes the join rule unambiguous).
- RLT kernel certificates (ACCEPT): G01 `ceil(10/3)`, G02 the 10-on-3 tiles, G06 the eta-size join ambiguity, G07 10 not divisible by 3.

CI: the combined branch passed every suite job the fork can run (thinkthoughts/roms-tools run 45245972980; Linux on Python 3.12–3.14, Windows, both dask parts). This split branch has its own fork CI branch, `rlt-cstar/ci-011-tiling` (= this branch + a fork-only trigger), not yet run. The two Copernicus jobs need credentials the fork lacks.

Open question for maintainers (not part of this PR): C-Star accepts uneven tilings today, and roms-tools ≤ 5.4.0 refuses to partition them (RLT-CSTAR F-024). The proposed order is this PR, a roms-tools release, then a C-Star `roms_tools` floor raise; a C-Star-side guard is prepared as an alternative and held.

AI disclosure: Claude (Anthropic); wrote the code, tests and this description, and ran the comparisons against compiled ucla-roms, under the author's direction and review.

## Code Review Checklist
- [ ] Closes #xxxx
- [x] Release note added as `docs/changes/+<name>.<type>.md`
- [x] New functionality has documentation, type hint coverage, and tests
- [x] Tests and `pre-commit` (changed files) are passing; `pre-commit run --all-files` to be rerun on the PR head

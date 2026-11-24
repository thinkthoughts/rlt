# F-029 independent review (commit 134183a) — reviewer's report, condensed

Reviewer: a separate agent with no access to how the change was written, working read-only. Its scratch copies, probes and fuzz script are in this directory.

**Verdict: REVISE.**

## What the reviewer ran

- The new test passes on the commit (3 passed). It fails on the parent's code, where the tile `ValueError` comes from `check_settings_supported` inside `configure_build`, after generation has run.
- Tier 1 gives 101 passed with no skips. `test_forge_blueprint_engine.py` plus `test_namelist_model.py` give 704 passed.
- A fuzz of 3,000 random inputs to `settings_support_problems` gives byte-identical output before and after the commit.

## Correctness of the overlay

- For explicit modes, the overlay matches what generation writes (`input_generator.py` L1915-1927, L2302-2326).
- `auto_tiling` follows `configure_build`.
- AUTO and mixed modes, grid-file grids and a missing `bgc` section fall back to the snapshot.
- Non-bool flags still go through `_flag_requested`.
- Edge case: when a surface file is reused without clobber, generation takes `interp_frc` from the file, not from the mode.

## Problems found

`blueprint_problems` still runs the tile check on the raw snapshot (through `validate_run_time_sections`) and then appends the overlay result. Probes on the 8×8 parent show:

| Case | Setup | Result | What generation and `configure_build` do |
|---|---|---|---|
| A | snapshot 2×2, `partitioning` 1×1 | refused; the message cites `partitioning.n_procs_x` | would accept. Reproduced independently. |
| E | `never` with a stale `interp_frc = True` | refused | would accept |
| C | snapshot `np_xi = 4`, `partitioning` 2×2 | two messages for one problem | — |
| — | snapshot 2×1 | one sentence repeated | — |

The substring dedup misses partial overlaps, because `check_tile_sizes` joins its sentences with a space. Cases A and E existed before the commit. C and the repeated sentence are new.

## Must fix

1. Run the tile check once, on the overlaid settings, and drop the substring dedup.
2. Add tests for A (accepted), C (exactly one message) and E (accepted).

## Nice to have

- Take the overlay sections from one constant.
- Add unit-level tests covering BGC-only, physics-only, AUTO and no-surface blueprints.
- Update the release note.
- Note the reuse-without-clobber case in the docstring.

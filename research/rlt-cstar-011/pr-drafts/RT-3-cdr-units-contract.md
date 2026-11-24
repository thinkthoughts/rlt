# [roms-tools] Check CDR metric units against an explicit dimensional contract

Branch (fork, proposed): `rlt-cstar/pr-cdr-units` `4df8045e`, stacked on RT-2 (`rlt-cstar/pr-co2-scale`) — 3 commits of its own (`eac702e`, `6ae455f`, `4df8045`). Merge after RT-2, or rebase if RT-2 is squash-merged.

# Summary

`compute_cdr_metrics` read any unrecognised `units` attribute as mmol with a warning. MARBL's CGS `nmol/cm^2/s` gave uptake ×100, `nmol/m^2/s` ×10⁶, and `kg/m^2/s` was read as mmol.

- Each of the 8 variables read now has an explicit dimensional contract over amount (N), equivalents (E), length (L), time (T) and mass (M). Any other dimension is refused with an error naming the variable; a missing `units` keeps the mmol default with a warning.
- Unit symbols are read case-sensitively (`Mmol` is a megamole, a factor 10⁹ from mmol), with `mMol` (ucla-roms `tracers.F90`) as the only alias.
- Every unit string ucla-roms writes (`mmol/m^2/s`, `meters mmol/m^3`, `meters mMol m-3`, `meq/s`, `mmol/s`) is accepted, and the metric values are unchanged for them.

Tests: 83 in `test_cdr_analysis.py` (19 before): accepted spellings; refused mass-normalised (`umol/kg`), concentration, area, time, equivalents, CGS, `day`/`yr`, `mmolC`, malformed strings; 25 adversarial case variants (6 fail without the case-sensitivity commit). `pre-commit` on the changed files: passed.

Limitation: no real ucla-roms CDR output file is in the test data; the accepted strings come from the ucla-roms source.

CI: fork CI branch `rlt-cstar/ci-011-cdr-units` (carries RT-2 below it), not yet run; the combined branch passed (run 45245972980).

Scope: this changes no definition of the CDR metrics. The separate question of what the DIC-based uptake should measure for DIC-removing releases (RLT-CSTAR F-009) is held for the method's authors and is not part of this PR. In short: for DIC-removing releases the DIC-difference uptake equals the flux-based uptake minus the removed fraction, shown with a closed-form box model; whether the metric should report extra air–sea uptake or net DIC stored is the authors' call, and an oracle test plus a proposed change are kept on a separate branch until they decide.

Release note: `+cdr-units.bugfix.md`.

AI disclosure: Claude (Anthropic); wrote the code, tests and this description under the author's direction and review.

## Code Review Checklist
- [x] Release note added as `docs/changes/+cdr-units.bugfix.md`
- [x] New functionality has documentation, type hint coverage, and tests
- [x] Tests and `pre-commit` (changed files) are passing

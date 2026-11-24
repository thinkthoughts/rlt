# [roms-tools] Test the tonnes-CO₂ scale against an independent value; cite its constant

Branch (fork, proposed): `rlt-cstar/pr-co2-scale` `3500f1dc` on `main` `ede4fe62` — 2 commits (`7033b93`, `3500f1d`).

# Summary

- The tonnes-CO₂-per-mmol scale in `cdr_analysis.py` was tested only against itself. The new test checks it against an exact rational value derived independently: 88019 / (2·10¹²) t per mmol, from M(CO₂) = 12.0107 + 2 × 15.9994 = 44.0095 g/mol.
- The constant's comment called it "IUPAC-style"; it is the IUPAC 2007 standard atomic weights. The comment now says so. The value is unchanged. (River code uses the abridged C = 12.011; the 1.1×10⁻⁵ relative gap is inside the abridged uncertainty.)

Tests: `pytest roms_tools/tests/test_analysis/test_cdr_analysis.py`: 20 passed; `pre-commit` on the changed files: passed. The expected value is an exact rational (Python `fractions`), so the test compares at 10⁻¹⁵ relative without a floating-point reference.

CI: fork CI branch `rlt-cstar/ci-011-co2-scale`, not yet run; the combined branch passed (run 45245972980). No behaviour change, so no release note (label `no-changelog`).

AI disclosure: Claude (Anthropic); wrote the test, the comment and this description under the author's direction and review.

## Code Review Checklist
- [x] PR is labeled `no-changelog`
- [x] Tests and `pre-commit` (changed files) are passing

# Audit regression tests

Counterexamples from the independent adversarial audit of RLT-001 (2026-11-07, commit 76de973).
`expected.tsv` lists each file's required exit code (0 accept, 1 reject, 2 malformed) and verdict word;
`cli/tests/golden.rs` enforces it.

- `kernel/` K01–K29: step-replay probes. K20, K24, K26 and K27 are sound derivations and must be accepted; the rest must be refused.
- `fuzz/` f01–f34: malformed or oversized input; each must exit cleanly (never abort). f34 is a sound derivation at the numeral limit.
- `axioms/` A01–A04, R2: proof files that named their own axiom files (`1 + 1 = 3`, a shadowed `add_zero`, an assumption moved into an axiom, an axiom named `refl`). Axiom files are now chosen only from registered, hash-locked sets, so each must be refused.

`kernel/` and `fuzz/` files were edited in one way only: `(axioms "spec/peano.rlt")` became `(axioms PEANO-ADD)`, so each probe reaches the check it targets. Files in `axioms/` keep their original path form.

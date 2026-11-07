# RLT-001 — Independent audit and fixes

**Audit.** A separate Claude instance, with no access to the implementation history, audited commit `76de973` adversarially from a clean clone. It reproduced the original results (30 tests, `verify.sh` "all checks hold") and wrote 68 counterexample proof files plus record and script fixtures.

**Outcome.** The kernel's step replay withstood all 29 kernel probes. The audit found defects around the kernel: axiom selection, binding relationships to proofs, record fidelity, the verification script, and resource limits. All demonstrated defects are fixed in `687f5e8`. Every counterexample is now a regression test (`tests/audit/`, `cli/tests/golden.rs`), and `scripts/verify.sh` reports "all checks hold" with 43 tests.

## Findings and status

| # | Severity | Finding (audit evidence) | Fix | Status |
|---|---|---|---|---|
| 1 | critical | A proof file chose any axiom file; `1 + 1 = 3`, a shadowed `add_zero`, an axiom named `refl`, and T5b's fiction moved into an axiom were all reported "ACCEPT — unconditional theorem" (A01–A04) | Proof files name a registered axiom set by id (`(axioms PEANO-ADD)`), and paths are refused. `spec/axioms.yaml` locks each set's file by SHA-256, its axiom names and its source, and the source must be registered and available. The verdict reads "… relative to registered axiom set PEANO-ADD", and the report states that registered axioms remain assumptions: the SHA-256 binds their content, not their truth. The kernel rejects axiom names that collide with rule or assumption names. | fixed: A01–A04 and R2 exit 2; a tampered `peano.rlt`, an unregistered set id and an unavailable set source are refused (`axiom_sets_are_registered_and_hash_locked`) |
| 2 | major | `rlt rp` and `rlt validate` printed a relationship's written text beside any proof's verdict (R1: "5 + 1 = 7 → ACCEPT") | `bind_relationship`: the written statement must elaborate to the proof's checked goal, and the proof must name the same reading point, relationship and convention. Proof paths are confined to `tests/…/*.rlt`. | fixed: R1 shows UNBOUND and fails validation (`relationships_are_bound_to_their_proofs`) |
| 3 | major | `verify.sh` always exited 0 (`fail` was set in a piped subshell) | Logging via `exec > >(tee …)`, with each stage's exit status captured | fixed: a tampered golden file makes `verify.sh` exit 1 with "RESULT FAILURES above" |
| 4 | major | `validate` used substring matching, exempted non-available sources, didn't tie meanings to their reading point, and didn't check completeness (V1–V7, Y1) | Meanings and examples must equal a **whole unit** of an excerpt, from the cited source, tagged `defines: <reading point>`. Block tags delimit units; inline tags are transparent. Every unit of every tagged excerpt must be present. `ambiguous` and `missing` sources cannot document meanings. Mistyped YAML fields are errors. | fixed: V1–V7, Y1 and swapped meanings are INVALID (`validator_enforces_verbatim_whole_units_and_completeness`) |
| 5 | major | Documented meanings were missing (repairearth's MATH29 description and detail texts, "#math29 = #antiviolent", the full "+1 reading level" sentences, "(nonfiction)", the residue line's tail) | Excerpts extended to full units and tagged; reading-point records now hold every tagged unit: math0 11 meanings / 3 examples, math5 11 / 3, math29 13 / 4. The `index.html` aria labels and the "#Practice the #obvious reading level: #math29" line were added too. | fixed: the completeness check passes |
| 6 | major | A `Theorem` recorded axiom names only, and the public API let outside code forge `⊢ 1 = 2` "from add_zero" | A `Theorem` now records each axiom's full content (name, equation, source); combining two different axioms under one name is refused; the CLI accepts only axioms equal by content to the registered set | partly fixed, by design: the kernel API still checks derivations relative to the axioms its caller supplies, but the forged axiom's equation `1 = 2` is now visible in the theorem and fails the registered-set gate. Trust in axioms is decided by `load_axiom_set` (now listed in the trusted base). |
| 7 | minor | Deep or wide input aborted the process (exit 134): stack overflow at nesting about 20,000 deep; 1.8 GB of memory from a 6 KB file (f27–f33) | Nesting depth ≤ 256 (reader and NAT-DEC); numerals ≤ 1000; ≤ 200,000 numeral total per expression; ≤ 1,000,000 term nodes per file; the CLI runs on a 64 MiB stack | fixed: all 34 fuzz files exit 0, 1 or 2, never abort; the numeral limit fell from 10000 to 1000 after a debug-build test showed unary chains of 10000 overflow a 2 MiB test-thread stack |
| 8 | minor | `check_excerpts.py` could match a "quote" stitched across separate JSON strings (X1) | Each decoded string is searched on its own | fixed: X1 reports MISSING |
| 9 | minor | The axiom path could point outside the repo and echo foreign file content (f21); a reading-point id could contain `../` (f23) | No axiom paths; record ids are restricted to `[a-z0-9._-]` with no `..`; read errors report only the error kind | fixed |
| 10 | minor | The Lean replay was hand-written; its run command failed without a default toolchain | `rlt lean` generates `lean/Generated.lean` from the accepted proofs' checked goals and Γ, and a test keeps it current; `verify.sh` runs Lean inside `lean/` | fixed |
| 11 | minor | REAL-PI and PRIME-COUNT cited a source that states neither reading; the PI-4 binding redefined the source's "Verified" | Both are marked `origin: rlt-authored` with zero sources, enforced by `validate`. PI-4 now leaves the source's Verified (Comparator-accepted Lean lifts) unmapped. | fixed |
| 12 | note | `check_excerpts.py` needed an undocumented `rlt` clone; no unit tests for `proof.rs`/`records.rs`; unpinned CI toolchain; an arithmetic ACCEPT printed beside reading-point meanings with no scope note | Usage documents all six clones; 4 + 2 unit tests added; `rust-toolchain.toml` pins 1.97.0; CI clones the sources and runs the excerpt check; every verdict carries the line "scope: the verdict concerns the checked equation only; it establishes neither the reading point's meanings nor any physical claim" | fixed |

Kept as audited (sound): K20 (an unused `hyp` step), K24 (a step label equal to an axiom name), K26 (`(S 1)` against `2`), K27 (conditional mode with Γ = ∅, reported as unconditional) and f34 (numeral at the limit) are valid derivations and stay accepted.

## Reproduction (commit `687f5e8`)

```
cargo test --release            43 passed (kernel 19, syntax 13, golden 11)
cargo clippy -D warnings        clean
cargo fmt --check               clean
tests/audit (68 files)          5 accept, 24 reject, 39 malformed — each as recorded in expected.tsv
proof files                     6 accept (exit 0), 4 reject (exit 1)
rlt validate                    VALID
check_excerpts.py               0 problems
Lean 4.34.0-rc2                 RLT001.lean and Generated.lean check
scripts/verify.sh               RESULT all checks hold, exit 0; exit 1 when any stage fails
```

## Remaining boundaries

- The kernel is parametric in its axioms. Soundness relative to Peano addition rests on the registered set `PEANO-ADD` and its loader, both reviewed like kernel code.
- Conversation sources (`USER-2026-11-07`, `RLT-000-REFINEMENT`) are quoted verbatim but have no file to check against.
- A second independent review of these fixes, by a fresh auditor working from `687f5e8`, would close the loop before merge.

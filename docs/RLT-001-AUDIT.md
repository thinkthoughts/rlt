# RLT-001 — Independent audits and fixes

Two independent adversarial audits and one final scoped review: the first audit of `76de973` (fixed in `687f5e8`), the second of `32e2b0e` (fixed in `88fbdf0`; §2), and the final review of `88fbdf0` (fixed in the commit after it; §3).

## 1. First audit

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
| 12 | note | `check_excerpts.py` needed an undocumented `rlt` clone; no unit tests for `proof.rs`/`records.rs`; unpinned CI toolchain; an arithmetic ACCEPT printed beside reading-point meanings with no scope note | Usage documents all six clones; 4 + 2 unit tests added; `rust-toolchain.toml` pins 1.97.0; CI clones the sources and runs the excerpt check; every verdict carries a scope line (wording revised after the second audit, §2 finding 1) | fixed |

Kept as audited (sound): K20 (an unused `hyp` step), K24 (a step label equal to an axiom name), K26 (`(S 1)` against `2`), K27 (conditional mode with Γ = ∅, reported as unconditional) and f34 (numeral at the limit) are valid derivations and stay accepted.

## Reproduction (commit `687f5e8`)

```
cargo test --release            43 passed (kernel 19, syntax 13, golden 11)
cargo clippy -D warnings        clean
cargo fmt --check               (claimed clean here; the second audit showed it failed at 687f5e8 — see §2)
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

## 2. Second audit (of `32e2b0e`)

A new Claude instance audited `32e2b0e` from a fresh clone, with no access to the first audit's scratch files. It confirmed that the kernel held against every probe, that proof files can no longer choose axioms, that the hash covers exactly the bytes parsed, and that all 68 first-audit counterexamples stay blocked. It recommended against merging until four blocking findings were fixed. All findings below are fixed, and every counterexample is a test in `cli/tests/golden.rs` (`a2_*`) or a case in the `verify.sh` breakage matrix.

| # | Severity | Finding | Fix | Status |
|---|---|---|---|---|
| 1 | major, blocking | Every REJECT printed "the checked equation follows from the listed axioms" (C02) | The scope line depends on the verdict. A REJECT reads "the kernel refused this derivation; the goal's truth stays undetermined by this proof" | fixed (`a2_c02_…`) |
| 2 | major, blocking | CI failed `cargo fmt --check` at every fix commit, so later steps never ran; the audit document claimed fmt was clean | Tree formatted. `verify.sh` now runs fmt and clippy; the false claim above is corrected | fixed: GitHub Actions runs 42426078745 (push) and 42426083108 (pull_request) on `f7df544` pass every step, including `verify.sh` ("all checks hold") |
| 3 | major, blocking | A newline in `(id "…")` forged report lines; `rp` read the first `verdict` text line (C01) | Control characters are refused in proof-file strings and symbols and in record strings (registry excerpts may span source lines). `rp` takes verdicts from the structured outcome | fixed (`a2_c01_…`) |
| 4 | major, blocking | Replacing `math5.yaml` with a copy of `math0.yaml` passed validation (C03) | A reading-point record's `id` must equal its file name | fixed (`a2_c03_…`) |
| 5 | major | Fidelity was relative to the registry: truncating or deleting a meaning in both registry and record passed, as did inventing one through a conversation source or `PEANO` (C04) | `check_excerpts.py` now requires every tagged excerpt to be a whole unit of its source (bounded by line ends, tags or quotes), and every source line naming a reading point to be covered by an excerpt. Conversation wording is pinned in the binary by SHA-256. `PEANO` and the other literature or whole-repository citations are status `reference`, which can ground an axiom set but never a meaning. An `available` source with excerpts needs a file location | partly fixed at `88fbdf0`: the final review (§3) showed that changing registry and record *together* (untag, delete, truncate, retag, self-cite) still passed; now fixed by pinning the whole registry (§3 Q4) |
| 6 | minor | `verify.sh` passed when Lean was missing, a Lean file used `sorry`, fmt or clippy failed, or the source directory was absent; `UPDATE_GOLDEN=0` rewrote goldens; a reused target directory tested another tree | Each stage now fails; skipping Lean or the excerpts needs an explicit flag and exits 3; only `UPDATE_GOLDEN=1` rewrites; tests resolve the repository root at run time; `tee` is awaited before exit | fixed (breakage matrix below) |
| 7 | minor | Assumption names reached Lean unescaped, so `a/-` … `-/` could comment out a theorem (C06 ii) | Assumption names must be identifiers | fixed (`a2_c06_…`) |
| 8 | minor | The conditional Lean replays are vacuous (their hypotheses are false in ℕ), and this was undisclosed (C06 i) | `Generated.lean` marks each such theorem `VACUOUS REPLAY`, and its header explains why | fixed (disclosed) |
| 9 | minor | Registry variants: duplicate set ids, symlinks, `spec/./`, dropped list items, a value stored in the same repository (C07) | Approved sets are pinned in the binary (id, file, SHA-256). The registry must match exactly once; symbolic links and non-string list items are refused | fixed (`a2_c07_…`) |
| 10 | minor | A relationship record stated no expected mode or Γ (C05) | Records state `mode` and `assumptions`; both are bound to the theorem | fixed (`a2_c05_…`) |
| 11 | minor | Evidence refs could be anything, and `kind` was free text (C09) | Evidence refs must be available registered sources; `kind` ∈ {behavior, representation, referent}. Relabelling a record's kind stays possible, as a reviewable edit | fixed (`a2_c09_…`) |
| 12 | minor | `units()` split prose at `<`, and a `>` inside a quoted attribute broke parsing (C08) | Only recognized HTML tag names count as markup; tags end outside quotes; entities are decoded (`(0 ≠ 5) ∧ (5 < 29)`) | fixed (`units` unit tests) |
| 13 | note | Symlinked proof files, `05 + 01 = 06`, duplicate relationship ids and duplicate proof-file fields were accepted; the README and audit text quoted the old scope line; `PEANO` was `available` with no file; elan came from an unpinned URL | Each is refused or corrected; the elan installer is pinned to v4.2.4 | fixed (`a2_c10_…`) |

### `verify.sh` breakage matrix (each case run in a fresh copy)

| Case | Exit |
|---|---|
| control, no breakage | 0 |
| failing unit test | 1 |
| changed golden file (shared target directory; also with `UPDATE_GOLDEN=0`) | 1 |
| a reject file that now accepts | 1 |
| `sorry` in a Lean file | 1 |
| stale `Generated.lean` | 1 |
| fmt violation | 1 |
| clippy violation | 1 |
| Lean not installed | 1 |
| Lean skipped with `RLT_ALLOW_NO_LEAN=1` | 3 (SKIPPED) |
| no source directory | 1 |
| excerpts skipped with `RLT_SKIP_EXCERPTS=1` | 3 (SKIPPED) |

### Remaining boundaries

- The kernel is parametric in its axioms. Trust in axioms rests on the pinned table, the loader and the files it locks, all reviewed like kernel code. Pinning binds content; the axioms remain assumptions.
- Conversation wording is pinned, and so cannot change without a reviewed code change. The pin cannot show that the wording matches what was said; that rests on the review record.
- A correspondence record's `kind` is declared by its author; relabelling it is a visible, reviewable edit.

## 3. Final scoped review (of `88fbdf0`)

A third independent Claude instance reviewed `88fbdf0` on four questions only, and reran the full suite. It confirmed that `f7df544` and `debec9e` change only the verification log and one line of this report.

| Question | Result at `88fbdf0` | Fix | Status |
|---|---|---|---|
| 1 · CI | **PASS.** Runs 42426078745, 42426083108, 42426195552 and 42426198327 ran and passed every step; their logs show the Lean replay and the excerpt check ran (not SKIPPED) and end "RESULT all checks hold"; no required check can be skipped or fail without failing the job | — | pass |
| 2 · Verdicts | **FAIL, non-blocking.** Exit codes and real verdict lines were always right, but a proof or reading-point *file name* containing newlines, or a convention id containing verdict text, could add forged lines before the real verdict | Displayed paths escape control characters, and `rlt check` refuses proof file names containing them; `readingpoints/` entries must be plain `*.yaml` regular files; convention ids must be `[A-Z0-9-]` | fixed (`r3_q2_…`) |
| 3 · Record identity | **FAIL, non-blocking.** Renaming `math5.yaml` to `.yml` passed `validate` (verify.sh still failed); a relationship proof reached through a symlinked directory under `tests/` passed (statement, mode and Γ were still bound) | Every `readingpoints/` entry must be a regular `*.yaml` record, and the known-reading-point list comes from the same filter; relationship proof paths must canonicalize inside `tests/` | fixed (`r3_q3_…`) |
| 4 · Source fidelity | **FAIL, blocking.** Changing registry and record together passed both `validate` and `check_excerpts.py`: untagging a meaning, deleting one whose source line never names the reading point, truncating a trailing element, swapping tags between reading points, citing a new self-made source | The registry's documentation content is pinned in the binary: `PINNED_REGISTRY_SHA` covers every source's id, status and location and every excerpt's text, `defines` and `split`. Any registry change needs a reviewed code change | fixed: all 13 source-fidelity counterexamples refused, control passes (`r3_q4_…`) |

Note (no change): a proof's `(context F0 fiction)` label can be edited with `validate` still VALID. The verdict and Γ stay the same, and the label is reported as written.

Full suite at `88fbdf0` (reviewer): fmt and clippy clean; 53 tests passed; `verify.sh` exit 0 ("all checks hold", 57 excerpts ok, 0 problems); both Lean files check; the 68 first-audit regression files match `expected.tsv`.

After the fixes: 56 tests pass (kernel 19, syntax 14, golden 23); `verify.sh` "all checks hold".

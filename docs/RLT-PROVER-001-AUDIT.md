# RLT-PROVER-001 — Release audit of the prover and theorem discovery

**Scope.** `rlt prove` and `rlt discover` (`cli/src/prove.rs`, `cli/src/discover.rs`), their tests (`cli/tests/prover.rs`), the 10 prover certificates in `tests/prover/certificates/` and the committed discovery record `research/prover-discovery/` (9 certificates). Both commands sit outside the trusted computing base; the kernel, axiom sets, certificates, statements and website are unchanged by this release.

**Verified release commit.** Branch `rlt-002-prover` at `3250be5`, merged into `main` with a merge commit whose tree equals `3250be5`'s tree.

## 1. Who checked what

| Step | Commit | Performed by | Independence |
|---|---|---|---|
| Release audit | `7784b34` (baseline `b36efff`) | A fresh Claude session given a read-only audit brief; it did not write the code under audit | independent of the implementation |
| Fix of the blocking defect | `3250be5` | The same session that ran the release audit | — |
| Final retest | `3250be5` | The same session that wrote the fix, from a fresh clone and fresh build | **developer retest, not an independent audit** |

A further independent retest of `3250be5`, by a session that wrote none of this code, would raise the final step to independent status.

## 2. Release audit at `7784b34`

| Area | Result |
|---|---|
| Full `scripts/verify.sh`, Lean 4.34.0-rc2, pinned sources | PASS — `RESULT all checks hold`, 355 tests |
| 19 committed certificates | PASS — each accepted by `rlt check`; P1–P10 re-proved byte-identical |
| Nine discoveries | PASS — reproduced byte for byte; `--verify` 399 of 399 candidates |
| Proof-search-to-kernel boundary | PASS — PROVED only after the written bytes are replayed by `rlt check`'s function; the hashed bytes are the replayed bytes |
| D5 conditional proof substituted for an unconditional row | PASS — refused |
| F1 extra header fields (`readingpoint`, `relationship`, `context`, `cites`) | PASS — refused |
| F2 record-level edits (dropped, reordered, extra rows; altered counts) | PASS — refused |
| F3 comments in certificates | PASS — refused |
| F4/K8 overwriting tracked or foreign files (absolute, `..`, symlinked paths, repository root) | PASS — refused, no tracked file changed |
| **Manifest comment injection** | **FAIL, blocking** — see §3 |

## 3. Blocking defect and fix

**Defect.** `check_record` compared only the first two lines of `manifest.tsv` with the generated header. Appending

```
# D010	∀x. x + x = 1 + 2	PROVED — kernel-replayed, see Lean companion
```

to a valid record left `rlt discover --verify` reporting `record complete` with exit 0, although the line claims a false equation as proved and `not-found.tsv` still lists D010.

**Fix (`3250be5`).** `manifest.tsv` must equal its canonical serialization: the two generated header lines, then one `\n`-terminated row per proved candidate, each row's columns already bound to its replayed certificate by `check_row`. The row validator and the record checks use the same line filter as the canonical form, so every retained line is validated as a row. Five regression tests (`prover001_*`) cover injected comments (after the header, between rows), fabricated PROVED lines (for a not-found candidate and for a false equation), blank lines, CRLF endings and a missing final newline; the four attack tests fail without the fix and pass with it.

## 4. Final retest at `3250be5` (developer retest)

| Check | Result |
|---|---|
| Diff `7784b34..3250be5` | 3 files: `cli/src/discover.rs` (+29), `cli/tests/prover.rs` (+92), `docs/PROVER.md` (+2/−1) |
| Original fabricated-PROVED attack | refused, exit 1 |
| 24 manifest probes (comments, blank and whitespace-only lines, CRLF, stray CR, BOM, missing newline, duplicated or moved header, duplicate rows, a proved row also in `not-found.tsv`, altered tabs) | all refused |
| Earlier audit suite (D5, F1–F4, D6, D7, STALE, manifest injection) | 22 of 22 refused |
| 19 committed certificates | 19 of 19 accepted |
| Nine discoveries | reproduced byte for byte; `--verify` 9 accepted, 399 of 399 candidates |
| `cargo test -p rlt --test prover` | 47 of 47 |
| Full `scripts/verify.sh`, Lean, pinned sources | `RESULT all checks hold`, 360 tests passed |

## 5. Known limitations (documented, non-blocking)

- **Search depth.** A certificate records a proof, not its search history, so `--verify` checks the manifest's depth column only as a number within the discovery bound (0 ↔ 1 verifies; 3 is refused). Every accepted row remains bound to a kernel-replayed unconditional certificate; this concerns search metadata, not kernel soundness.
- **Git dependence of F4.** Outside a git checkout only the K8 rule (never overwrite a file the tool did not write) protects existing files.
- **Hard links.** `rlt prove --out` writes through a hard link deliberately placed at an untracked path that points to a tracked certificate.
- **Relationship labels (pre-existing).** `rlt check` prints any `relationship` id a proof file names, registered or not; binding is decided by reading-point records. This predates the prover and is relevant to RLT-PROVER-002.
- **Verification coverage.** `scripts/verify.sh` covers the prover certificates and discovery record through `cargo test` (`cli/tests/prover.rs`).

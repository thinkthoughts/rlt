# RLT-PROVER-002 — Release audit of registered function definitions

**Scope.** Registered function definitions under convention NAT-FUN (`definitions/double.rlt`, `definitions/quadruple.rlt`, `syntax/src/fdefs.rs`), definition-aware discovery (`rlt discover --definitions`, `cli/src/discover_defs.rs`), the seven certificates in `tests/definitions/certificates/`, the committed discovery record `research/definition-discovery/` (126 theorems), and the Lean companion `lean/ProverDefinitions.lean` with its correspondence checker (`scripts/check-definitions-lean.py`, `scripts/lean/DeclCheck.lean`). Definitions are unfolded before the kernel by the RLT elaborator, outside the trusted computing base; the kernel, axiom sets, rules and pinned M3b are unchanged by this release. Full report: `docs/RLT-PROVER-002.md`.

**Verified release commit.** Branch `rlt-002-definitions` at `e2bb7e3`, merged into `main` by the no-fast-forward merge `67cfed2` (parents `e4dc951`, main, and `e2bb7e3`). `main` had advanced by two commits that add `RH/.gitkeep` and `RH/index.html` and touch no file on the branch, so the merged tree equals `e2bb7e3`'s tree plus `RH/`.

## 1. Who checked what

| Step | Commit | Performed by | Independence |
|---|---|---|---|
| Initial audit (D1–D4) | `3b0fcea` | As recorded in `docs/RLT-PROVER-002.md` §9 | not performed by the session that wrote D6/H1 |
| Release audit (D5) | `13ff7b7` | As recorded in §9 | not performed by the session that wrote D6/H1 |
| Release retest (found D6) | `0987be5` | A Claude session given a read-only retest brief; it did not write the D5 fix it retested | independent of the D5 implementation |
| D6 repair | `a9d466d` | The same session as the retest | — |
| Lean-layer audit (H1 noted, MERGE) | `a9d466d` | The same session that wrote D6, from a fresh clone and fresh build | **self-review, not an independent audit** |
| H1 hardening, final verification, merge | `e2bb7e3`, `67cfed2` | The same session | **self-review** |

The audit of `a9d466d` compensated for its lack of independence by checking against evidence the repair does not use: RLT's printed kernel forms, the registered s-expression bodies, Lean's own delta-unfolding, and a regeneration of the discovery record from scratch. A further retest of `e2bb7e3` by a session that wrote none of D6 or H1 would raise the final steps to independent status.

## 2. Findings and their resolution

| Finding | Found at | Resolution |
|---|---|---|
| D1 — unbounded unfolding exhausted memory | `3b0fcea` | fixed in `13ff7b7`: cumulative node budget, refused before allocation |
| D4 — Lean comparison overstated | `3b0fcea` | fixed in `13ff7b7`: claims separated; script checks the text, not only `Iff.rfl` |
| D5 — comparison by substring, comments included | `13ff7b7` | fixed in `0987be5`: comments removed, declaration grammar, per-declaration statements |
| **D6 — escaped identifiers `«/-»` … `«-/»` desynchronised the Python lexer from Lean's** (Lean compiled `F3_unfolds : (double 2 = 4) ↔ (3 + 1 = 4)`; the script reported 7 of 7) | `0987be5` | fixed in `a9d466d`: the verdict compares the declarations Lean elaborated with an expected module built from RLT's printed statements and the registered definitions; the source-layout check is kept as a second layer and refuses `«…»` |
| H1 — `set_option debug.skipKernelTC true` passed the Lean layer on its own (a `run_cmd` adding `F3_holds : double 2 = 4 := True.intro` compiled cleanly; only the Init-only import rule refused it) | `a9d466d` | hardened in `e2bb7e3`: `leanchecker` replays every constant of the compiled companion before the comparison; `verify.sh` also runs `leanchecker --fresh` |
| D2 — canonical definition identity; D3; search-depth metadata | earlier audits | documented, carried to the next milestone (§4) |

## 3. Verification

**Lean-layer audit of `a9d466d`** (fresh clone, fresh build):

| Check | Result |
|---|---|
| D6 attack | Lean compiles it; the Lean layer alone, the layout layer alone and the full checker each refuse it |
| Comparison of elaborated terms | def types and values, theorem types as kernel terms (alpha-equivalence); printed text used only in messages |
| 19 further mutants against the Lean layer alone (imports, axioms, `native_decide`, extra or internal-named constants, `implemented_by`, `private`, notation, instance attributes, `open`, `autoImplicit`, `partial`, `@[reducible]`, `skipKernelTC`) | refused, or harmless (identical elaborated terms); `skipKernelTC` led to H1 |
| Translation fidelity, independent of the script's rewrite | for F1–F7 the unfolded side equals RLT's kernel form, and Lean's delta-unfolding of the folded side equals it syntactically; both Lean defs equal the registered s-expression bodies; `quadruple_body` matches |
| Discovery completeness | `rlt discover --definitions` from scratch reproduces all 129 files of the record (modulo the directory path in the manifest) |
| Full `scripts/verify.sh`, Lean 4.34.0-rc2, freshly fetched pinned sources | `RESULT all checks hold`, 386 tests |

**Final verification of `e2bb7e3`:**

| Check | Result |
|---|---|
| `leanchecker` on the current companion | accepts it (0.3 s); `--fresh` accepts it (≈ 56 s) |
| H1 case (`True.intro` under `skipKernelTC`) | Lean compiles it; the Lean layer and the full checker refuse it at the kernel replay (`declaration type mismatch`) |
| `skipKernelTC` on a valid theorem | refused by the complete verification (layout layer); replayed and matched by the Lean layer |
| `cargo test --release --test definitions` | 18 of 18 (D1, D4, D5, D6, H1 regressions included) |
| Seven certificates | 7 of 7 ACCEPT as unconditional theorems under ROBINSON-Q |
| Discovery record | `--verify`: 126 accepted, 0 refused or stale; 1430 of 1430 candidates |
| Lean correspondence | PASS for 7 goals: 17 theorems and 2 defs, 7 `Iff.rfl` unfoldings in the clean module, standard axioms only, kernel replay |
| Full `scripts/verify.sh`, Lean 4.34.0-rc2, pinned sources | `RESULT all checks hold`, 387 tests, D5/D6/H1 gates and `leanchecker --fresh` included |

**Merged tree `67cfed2`:** equals `e2bb7e3` plus `RH/` (verified by `git diff`); the full `scripts/verify.sh` is run on `main` after this record is committed, before `main` is pushed.

## 4. Known limitations (documented, non-blocking)

- **D2 — canonical definition identity.** A definitions file is identified by path and SHA-256; registry ids and function names are not registered, so another file may also say `(id DOUBLE)`. A pinned registry is needed before predicate definitions.
- **D3 and search-depth metadata.** As documented in `docs/RLT-PROVER-002.md` §9; they concern search metadata, not kernel soundness.
- **Scope of the Lean comparison.** Seven goals and three further Lean theorems (`quadruple_body`, `G1_holds`, `G2_holds`); not a general proof that RLT's elaborator and unfolder agree with Lean.
- **Folded-side translation.** The folded statements reach Lean through the script's text rewrite. Its result unfolds, in Lean, to exactly RLT's kernel term for every goal, but a mistranslation that unfolds identically (for example `x + x` written where RLT printed `double(x)`) would not be detected. G1 and G2 have no kernel form.
- **What the replay is.** `leanchecker` re-runs Lean's own kernel: it detects declarations that bypassed the kernel, not a defect in the kernel. Imported core-Lean constants are trusted as the pinned toolchain's except in the `--fresh` replay that `verify.sh` runs.
- **Lean required.** Without Lean the correspondence is reported UNVERIFIED (exit 3); the D6 and H1 tests and `verify.sh` fail unless `RLT_ALLOW_NO_LEAN=1`.
- **Allowlist.** The source-layout layer's grammar is deliberately narrow; a legitimate change to the companion's shape needs it updated.

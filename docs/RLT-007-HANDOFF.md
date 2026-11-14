# RLT-007 — Handoff: read this first

**Status:** 2026-11-13, written against main at `330db65` (after the H₄₅ artifact merge). RLT-007 is closed; this document is the entry point for new work. Longer history: `docs/RLT-REVIEW-HANDOFF.md` (every merge record) and the per-milestone reports in `docs/`.

## 1. What RLT checks and proves today

RLT (github.com/thinkthoughts/rlt) is an independent proof checker for first-order arithmetic, written in Rust. Verified results on main:

| Result | Status | Where |
|---|---|---|
| Modulo-30 classification below 30 (RLT-003 M3e) | kernel-checked theorems under ROBINSON-Q | `tests/m3e/`, `lib/mod30/`, `docs/RLT-003-M3e.md` |
| 8 is not matchable (McNew–Pomerance, arXiv:2604.05304v2, §1) | kernel-checked theorem, with a reviewed correspondence | `lib/matchable/`, `docs/RLT-005-REPORT.md`, `docs/RLT-005-CORRESPONDENCE.md` |
| Lemma 12.1 of Axiom Math's prime-gap draft: H₄₅ is an admissible 45-tuple of diameter 212 | six statements: five kernel-checked theorems and one discharged by composition from kernel-checked theorems (62 in all; 8 conditional assemblies; 56 discharged assumptions) | `compose/h45.yaml`, `docs/RLT-007d-REPORT.md`, `docs/RLT-007d-CORRESPONDENCE.md`, `research/h45-lemma-12-1/` |

Numbers on main: 312 Rust tests; 159 library entries, all current; `scripts/verify.sh` reports `RESULT all checks hold` (about 74 minutes on two cores, 47 minutes on CI).

## 2. The kernel and the composition layer

- **Kernel** (`kernel/src/`, zero dependencies, no `unsafe`): natural deduction for first-order logic with equality, 20 rules plus `cite`: and-intro, and-left, and-right, or-left, or-right, or-cases, imp-intro, imp-elim, forall-intro, forall-elim, exists-intro, exists-elim, false-elim, hyp, inst, refl, symm, trans, cong, subst. A `Theorem` Γ ⊢ φ can only come from a rule (`docs/KERNEL-SOUNDNESS.md`). Limits: 1,000,000 nodes per file, 64 cited-file checks, citation chain 8.
- **Axiom sets** (`spec/axioms.yaml`, each file pinned by SHA-256 in the binary): PEANO-ADD, PEANO-ARITH and **ROBINSON-Q** (`spec/robinson-q.rlt`: q1_succ_inj, q2_zero_ne_succ, q3_zero_or_succ, add_zero, add_succ, mul_zero, mul_succ). Definitions (`spec/definitions.rlt`): `<`, `∣`, `free235`, expanded before the kernel.
- **Commands** (`docs/MANUAL.md`):
  - `rlt check FILE` — replay a proof file (exit 0 accept, 1 reject, 2 malformed);
  - `rlt explain RECORD` — a statement record: kernel, source, semantic and physical status;
  - `rlt deps ID|FILE`, `rlt deps --verify-lib` — dependency graph and library consistency;
  - `rlt compose BUNDLE` — layered composition of kernel-checked theorems (exit 0 composed, 1 refused, 2 malformed or missing, 3 stale).
- **Composition** (`cli/src/compose.rs`): joins separately checked files by the cut rule. It re-checks every proof in the run, matches each assumption of each conditional assembly canonically (bound variables up to renaming) with exactly one theorem of a lower layer, under the same axioms, convention and definitions, and requires the registered specification of every node (`compose/approved-specifications.tsv`) and of the named result (`compose/approved-results.tsv`). It reports a composition of kernel-checked theorems; a single kernel proof object for the whole result is outside its scope.

## 3. From an external claim to a source-pinned specification

1. **Register the source** in `sources/registry.yaml` with its exact location and a pin (commit for a repository, SHA-256 for a file); `rlt validate` checks the registry.
2. **Transcribe** the parts used into a reviewed file (example: `sources/transcriptions/AXIOM-BGP212-2026-09-03.yaml`: PDF hash, extraction tool and version, per-page hashes, extracted and reviewed text, numbers).
3. **State** each obligation as an RLT formula; generators read numbers only from the transcription (`scripts/gen-rlt006.py`, `scripts/gen-rlt007d.py`).
4. **Prove** it (proof files under `tests/…/accept/`), register library entries (`lib/`) and statement records (`statements/`, `rlt explain`).
5. **Review the correspondence** in a `docs/…-CORRESPONDENCE.md`: how the formulas read as the source's claim. Labels used throughout: *kernel accepted*, *discharged by composition*, *source transcription checked*, *semantic correspondence assessed*.

## 4. How RLT, Lean and audits complement one another

| Evidence | Answers | Example |
|---|---|---|
| RLT kernel | does φ follow from the pinned axioms under Γ? | every `tests/*/accept/*.rlt` |
| RLT composition | are separately checked theorems joined correctly? | `rlt compose compose/h45.yaml` |
| Independent re-implementation | does a second program agree, sharing no matching code? | `scripts/compose-check.py` (52 negative cases agree) |
| Lean 4 | does the same mathematics hold in another system, in standard vocabulary? | `lean/RLT007D.lean` (`lemma_12_1_holds`) |
| Source and correspondence review | do the formulas state what the source claims? | `docs/RLT-007d-CORRESPONDENCE.md` |
| Independent audits | is the implementation what the documents say? | mathematical and implementation audits in each `docs/…-REPORT.md` |

## 5. Reproduce

```sh
git clone https://github.com/thinkthoughts/rlt.git && cd rlt
research/h45-lemma-12-1/reproduce.sh     # H45: manifest, build, compose, Python checker, library, Lean (≈15 min)
cargo test --release                     # 312 tests
scripts/verify.sh SOURCE_CHECKOUTS_DIR   # everything; RESULT all checks hold
python3 -I scripts/check-manual.py target/release   # the manual's examples
```

Lean 4 `v4.34.0-rc2` (`lean/lean-toolchain`) is needed for the Lean steps; `SOURCE_CHECKOUTS_DIR` holds clones of the registered source repositories (`scripts/fetch-sources.py`).

## 6. Unavailable or unverified

- **Unverified:** the Rust code's faithfulness to the rules (argued and tested in `docs/KERNEL-SOUNDNESS.md`, unproved by machine); the composition checker's code (cross-checked and audited, unproved).
- **Outside the logic:** induction (Robinson's Q has none), real numbers, analysis, unbounded quantification over sets or functions; statements beyond first-order arithmetic.
- **Outside the tooling:** one kernel proof object for a composed result (Option A, a larger citation budget, is documented in `docs/RLT-007d-ARCHITECTURE.md` and unapproved); composed results as citable library entries; tactics or proof search (proofs come from generators or by hand).
- **Proposals only:** anything in `docs/RLT-007d-ARCHITECTURE.md` beyond what is merged.

## 7. Beginning a review of a new external repository

1. Confirm the repository URL with the requester and **pin the exact revision** (full commit SHA); record it before reading further.
2. Work **read-only**: clone into a separate directory; never push to, open issues on, or edit the external repository.
3. Read the project on its own terms first: its stated motivation, architecture, claims and existing evidence (tests, proofs, benchmarks).
4. Inventory claims and classify each as a source/specification claim, a mathematical claim, an implementation claim or an empirical claim.
5. For each, ask what independent check would be meaningful (RLT, Lean, a re-implementation, an audit, an experiment) and which needs none.
6. Write the review before proposing code or a PR; new RLT work follows §3 and keeps the kernel, rules, axioms and limits unchanged unless a change is separately approved.

## 8. Baseline

- **main:** `330db65972bf0777abea310fca3eff534bb0170e`, tree `8f5c604c2eaf50c48efeebf3bf7a01d178fc9a0d` (merge of PR #22 onto `9db928a`, the RLT-007d merge).
- **Checks at this baseline:** `scripts/verify.sh` `RESULT all checks hold` at `9db928a` (the artifact adds only `research/`); `research/h45-lemma-12-1/reproduce.sh` all PASS from a clean checkout; CI `check` and `gdt-comparison` green.
- **Open PRs:** #16 (`docs/rlt-007-next`, review history and merge records, unmerged).

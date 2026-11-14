# H₄₅ is an admissible 45-tuple of diameter 212 — an RLT reproduction

This directory is a self-contained guide to RLT's verification of **Lemma 12.1** of Axiom Math's *A New Bound for Small Gaps Between Primes* (preliminary draft, September 3, 2026). Everything it refers to is in this repository at merge commit `9db928a1b20646d847c62e68a73a4993ab0919fc` (tree `56e201d4ad35…`).

**Result in one line.** Lemma 12.1 holds as six statements of first-order arithmetic over Robinson's Q: five are kernel-checked theorems and the sixth, admissibility at every prime, is **discharged by composition from kernel-checked theorems** — 62 in all, joined by a composition checker that re-checks every proof and matches every assumption exactly. Lean 4 independently proves the corresponding mathematical statement.

## 1. The source

| | |
|---|---|
| Paper | F. Charton, L. Hong, K. Lau, K. Ono, G. Remy, H. C. Siu, A. A. Swaminathan, J. Thorner, Y. Xie (Axiom Math), *A New Bound for Small Gaps Between Primes*, Preliminary Draft: September 3, 2026 |
| File | https://primegaps.axiommath.ai/bgp212.pdf — 46 pages, 734,121 bytes |
| SHA-256 | `2b307ae26046dffd2153fe7466e5a9a302976dc3ba5142151a8a8104a37b3d80` |
| Lemma 12.1 (p. 43) | "The set H₄₅ is an admissible 45-tuple of diameter 212." |
| Admissible (§2, p. 3) | "A finite set H = {h₁, …, h_k} of distinct integers is called admissible if, for every prime p, the elements of H omit at least one residue class modulo p." |
| H₄₅ (§12, p. 43) | {0, 2, 12, 14, 24, 26, 30, 36, 44, 50, 54, 56, 60, 66, 72, 74, 80, 84, 92, 96, 102, 110, 114, 116, 122, 126, 134, 140, 144, 150, 156, 162, 164, 170, 176, 180, 182, 186, 192, 194, 200, 204, 206, 210, 212} |
| The paper's proof | omitted classes 1, 1, 3, 6, 9, 3, 1, 13, 17, 4, 11, 4, 4, 3 for the primes 2 … 43; for p ≥ 47, a counting argument |

The numbers enter RLT only through the reviewed transcription `sources/transcriptions/AXIOM-BGP212-2026-09-03.yaml`, which records the PDF hash, the extraction tool and version, and a SHA-256 for each extracted page.

## 2. The RLT result: six statements

All six are closed formulas under the axiom set ROBINSON-Q (Robinson's Q, seven axioms, `spec/robinson-q.rlt`) and the convention NAT-DEF, where `a < b` abbreviates ∃z. S(z) + a = b and `d ∣ n` abbreviates ∃k. n = d × k (`spec/definitions.rlt`). With h₀ … h₄₄ the listed numbers in order:

- OM(p, r) :≡ ⋀ᵢ ¬(∃k. hᵢ = (p × k) + r) — "no listed number is ≡ r (mod p)";
- Adm(p) :≡ ∃r. (r < p ∧ OM(p, r));
- Prime_B(p) :≡ 1 < p ∧ ∀d. ((1 < d ∧ d < p) → ¬(d ∣ p)).

| # | Statement | Status |
|---|---|---|
| 1–4 | `distinct.g0`–`g3`: the 990 disequalities ¬(hᵢ = hⱼ), i < j | kernel-checked theorem |
| 5 | `bounds`: (∃z. z + 0 = hᵢ) ∧ (∃z. z + hᵢ = 212) for every i | kernel-checked theorem |
| 6 | `adm.all`: ∀p. (Prime_B(p) → Adm(p)) | discharged by composition from kernel-checked theorems |

Read over ℕ, these say that the 45 listed numbers are distinct, lie between 0 and 212, and omit a residue class modulo every prime; with the list's length and its endpoints 0 and 212, that is Lemma 12.1. The reading is set out in `docs/RLT-007d-CORRESPONDENCE.md`.

## 3. The 62 kernel-checked theorems

| Layer | Theorems | Each is |
|---|---|---|
| 0 | 14 omissions OM(p, r_p) with the paper's classes, p = 2 … 43 | an unconditional kernel theorem |
| 0 | 33 omissions with an explicit class, p = 47 … 211 (the least omitted class; RLT's route in place of the counting argument) | an unconditional kernel theorem |
| 0 | the tail: ∀p. (212 < p → OM(p, 1)) | an unconditional kernel theorem |
| 0 | the classification: ∀p. (Prime_B(p) → p is one of the 47 primes below 213, or 212 < p) | an unconditional kernel theorem |
| 0 | statements 1–5 | unconditional kernel theorems |
| 1 | six range assemblies ∀p. (p ∈ range_i → Adm(p)) and the tail assembly ∀p. (212 < p → Adm(p)) | conditional kernel theorems Γ ⊢ φ |
| 2 | `adm.all` | a conditional kernel theorem Γ ⊢ φ |

Ranges: [0, 40], [41, 78], [79, 112], [113, 156], [157, 190], [191, 212]. The assemblies prove r_p < p for each prime themselves.

## 4. The 56 discharged assumptions

Every assumption γ of every assembly is matched by exactly one theorem of a lower layer whose kernel-checked statement is canonically the same formula (bound variables up to renaming) under the same axioms, convention and definitions:

| Assembly | Assumptions | Discharged by |
|---|---|---|
| `adm.range_1` … `adm.range_6` | 12, 9, 8, 7, 6, 5 | the omissions for the primes of each range |
| `adm.tail` | 1 | the tail |
| `adm.all` | 8 | the classification, the six ranges, the tail |

The full table is `docs/RLT-007d-REPORT.md` §2; `rlt compose` prints each edge.

## 5. Reproduce

Requirements: Rust 1.97.0 (`rust-toolchain.toml`), Python 3 with PyYAML, and, for the Lean step, Lean 4 `v4.34.0-rc2` (`lean/lean-toolchain`, core Lean only). On two cores, `reproduce.sh` took 14 min 29 s including the build; the checks alone took 3 min 11 s (`rlt compose`), 1 min 50 s (Python) and 2 min 13 s (Lean).

```sh
git clone https://github.com/thinkthoughts/rlt.git && cd rlt
git checkout 9db928a1b20646d847c62e68a73a4993ab0919fc
research/h45-lemma-12-1/reproduce.sh
```

`reproduce.sh` builds RLT, materializes the generated certificates the components cite, and runs each check below, stopping at the first unexpected outcome.

| Step | Command | Expected |
|---|---|---|
| 1 | `target/release/rlt compose compose/h45.yaml` | `verdict COMPOSED`, exit 0; `graph 62 nodes in layers 0–2, 56 discharged assumptions`; `remaining assumptions: none`; summary "5 statements kernel accepted, 1 discharged by composition (…)" — full report in `expected/rlt-compose.txt` |
| 2 | `python3 -I scripts/compose-check.py target/release/rlt compose/h45.yaml` | `verdict COMPOSED`, exit 0; each of the six results "approved statement (rebuilt from the transcription)" — `expected/compose-check.txt` |
| 3 | `(cd lean && lean RLT007D.lean)` | 17 `#print axioms` lines, each with at most `propext`, `Classical.choice`, `Quot.sound` — `expected/lean.txt` |
| 4 | `target/release/rlt deps --verify-lib` | every library entry current |
| optional | `scripts/verify.sh SOURCE_CHECKOUTS_DIR` | `RESULT all checks hold` (the whole repository, about 80 minutes) |

## 6. What each check establishes

| Check | Establishes | Rests on |
|---|---|---|
| **RLT kernel** (`kernel/`) | each of the 62 theorems: its statement follows from Robinson's Q by the 20 inference rules (plus `cite` of other checked files), under exactly its listed assumptions | the kernel's correctness (`docs/KERNEL-SOUNDNESS.md`); Q's axioms, pinned by SHA-256 |
| **Composition** (`rlt compose`) | statement 6: every assumption of the eight assemblies is one of the kernel-checked theorems of a lower layer; every proof re-checked in the same run; the six results are exactly the approved statements | the semantic cut rule with induction on layers (`docs/RLT-007d-REPORT.md` §3); the composition checker's code, cross-checked below |
| **Python checker** (`scripts/compose-check.py`) | the same verdict, from its own bundle reader, formula parser, canonical form and fingerprint computation, with the approved statements rebuilt from the transcription | `rlt check`'s kernel output only |
| **Lean 4** (`lean/RLT007D.lean`) | the 62 statements written symbol for symbol; the same composition carried out in Lean; Lemma 12.1 in the standard vocabulary (`lemma_12_1_holds`: 45 distinct entries in [0, 212] including 0 and 212, and for every prime p some r < p with h mod p ≠ r for all h) proved outright | Lean's kernel and standard axioms; independent of RLT |
| **Source transcription** | the 45 numbers, their order and the 14 classes are those of the pinned PDF | the reviewed transcription and its per-page hashes |
| **Correspondence** | the six statements, read over ℕ, are Lemma 12.1 | the reviewed reading in `docs/RLT-007d-CORRESPONDENCE.md` |

**Trust boundary.** The result is discharged by composition from kernel-checked theorems; a single proof object spanning the whole lemma is outside this work. The composition checker is trusted code beyond the kernel: it builds no formula and applies no inference rule, and its verdicts are matched by the independent Python implementation on the real bundle, three controls and 52 negative cases (missing, stale, conditional, duplicated, unused, substituted or incompatible components, cycles, and true statements that differ from the approved ones), and were reviewed in two independent audits with targeted re-audits.

## 7. Files

| Path | Content |
|---|---|
| `compose/h45.yaml` | the composition bundle: 62 nodes, their pins and the 56 discharges |
| `compose/approved-specifications.tsv`, `compose/approved-results.tsv` | the approved statements (one probe per node) and the named six-statement result |
| `tests/rlt007d/assemble/` | the eight conditional assemblies |
| `tests/rlt006/`, `tests/rlt007s1/` … `tests/rlt007s4/`, `lib/` | the layer-0 theorems and their library entries |
| `scripts/compose-check.py` | the independent Python checker |
| `lean/RLT007D.lean` | the Lean comparison |
| `docs/RLT-007d-REPORT.md`, `docs/RLT-007d-CORRESPONDENCE.md` | the report (soundness, measurements, audits) and the correspondence review |
| `research/h45-lemma-12-1/MANIFEST.sha256` | SHA-256 of the 200 committed files above at the merge commit (the generated certificates the components cite are pinned by their citations and by `tests/*/manifest.sha256`) |
| `research/h45-lemma-12-1/expected/` | the outputs of steps 1–3, recorded at the merge commit |

# RLT-007d — Outline of the author-facing research artifact

**Status:** documentation-only outline, 2026-11-13. The artifact itself is prepared after RLT-007d is merged; nothing here is published, tagged or sent.

Throughout, the completed RLT result is described as **discharged by composition from kernel-checked theorems**: 62 kernel-checked theorems joined by a separate composition checker. The artifact presents it in exactly those terms, and calls statement 6 a composed result (never one unconditional RLT kernel theorem).

## 1. The source

- **Paper:** F. Charton, L. Hong, K. Lau, K. Ono, G. Remy, H. C. Siu, A. A. Swaminathan, J. Thorner, Y. Xie (Axiom Math), *A New Bound for Small Gaps Between Primes*, Preliminary Draft: September 3, 2026.
- **File:** https://primegaps.axiommath.ai/bgp212.pdf, 46 pages, 734,121 bytes, SHA-256 `2b307ae26046dffd2153fe7466e5a9a302976dc3ba5142151a8a8104a37b3d80` (registry id `AXIOM-BGP212-2026-09-03`).
- **Statement:** Lemma 12.1 (p. 43): "The set H₄₅ is an admissible 45-tuple of diameter 212." Definition of admissible (§2, p. 3); H₄₅ and the 14 listed classes (§12, p. 43).
- **Transcription:** `sources/transcriptions/AXIOM-BGP212-2026-09-03.yaml`, with the extraction tool, its version and per-page extraction hashes.

## 2. The six-statement RLT result

Table of the six statements (`distinct.g0`–`g3`, `bounds`: kernel accepted; `adm.all`: discharged by composition from kernel-checked theorems), with the definitions of OM, Adm and Prime_B and the NAT-DEF expansions of `<` and `∣` (`docs/RLT-007d-CORRESPONDENCE.md` §2), and the reading as Lemma 12.1 (§3 there).

## 3. The 62 kernel-checked theorems and eight conditional assemblies

- Layer 0, 54 unconditional theorems: 14 omissions from the paper's classes (2 … 43), 33 explicit omissions (47 … 211), the class 1 for every p > 212, the classification of primes below 213, the four distinctness theorems and the bounds theorem, with the RLT stage that proved each.
- Layers 1–2, eight conditional theorems: six range assemblies, the tail assembly, `adm.all`.
- Route compared with the paper's proof (explicit classes in place of the counting argument for p ≥ 47).

## 4. The 56 checked assumption discharges

The discharge table (`docs/RLT-007d-REPORT.md` §2), the matching rule (canonical kernel formulas, bound variables up to renaming, same axioms, convention and definitions), and the `rlt compose` report excerpt: 62 nodes in layers 0–2, 56 discharged assumptions, remaining assumptions none.

## 5. Independent Python verification

`scripts/compose-check.py`: its own bundle reader, formula parser, canonical form and fingerprint computation; approved statements rebuilt from the transcription. Agreement with `rlt compose` on the full bundle, the controls and every negative case, by refusal class and exit code.

## 6. Lean 4 correspondence and independent theorem

`lean/RLT007D.lean`: the 62 statements symbol for symbol; the same composition in Lean; `rlt_lt_iff`, `rlt_dvd_iff`, `prime_iff`, `om_iff`; `lemma_12_1_holds`, Lemma 12.1 in the standard vocabulary, proved in Lean outright. Standard axioms only.

## 7. Reproduction

Commands, from a clone at the merged commit, with expected outcomes:

```
cargo build --release
python3 -I scripts/gen-rlt006.py   target/release/rlt --materialize-only
python3 -I scripts/gen-rlt007s1.py target/release/rlt --materialize-only
python3 -I scripts/gen-rlt007s4.py target/release/rlt --materialize-only
target/release/rlt compose compose/h45.yaml                                  # verdict COMPOSED, exit 0
python3 -I scripts/compose-check.py target/release/rlt compose/h45.yaml       # verdict COMPOSED, exit 0
(cd lean && lean RLT007D.lean)                                                # 17 #print axioms lines, standard axioms only
scripts/verify.sh                                                             # RESULT all checks hold
```

With approximate times (about 5, 3 and 3.5 minutes for the three checks on two CPUs) and the commit and CI run they were recorded on.

## 8. Trust boundaries

- **Trusted kernel:** Robinson's Q (7 axioms, pinned by SHA-256), 20 inference rules plus `cite`, resource limits; `docs/KERNEL-SOUNDNESS.md`.
- **Composition checker:** `rlt compose`, new trusted code that re-checks every proof and compares canonical formulas; it constructs no formula and applies no rule. Soundness argument: the cut rule with induction on layers. Cross-checked by the Python implementation, 52 negative cases and two independent audits.
- **Source-transcription boundary:** the 45 numbers, their order and the 14 classes come from the reviewed transcription; the reading of the six statements as Lemma 12.1 is a reviewed correspondence (semantic correspondence assessed).

## Intended length

Two to three pages: one page for §§1–4 with the dependency diagram, one for §§5–7, a short §8. Links point to files at the merged commit.

# RLT-006 — Source correspondence: the H₄₅ residue certificates for Lemma 12.1

Basis for the semantic assessment in the 14 records `statements/H45-OMITS-*.yaml`, which pin this file by SHA-256.

## 1. Source and transcription

| Item | Value |
|---|---|
| Paper | F. Charton, L. Hong, K. Lau, K. Ono, G. Remy, H. C. Siu, A. A. Swaminathan, J. Thorner, Y. Xie (Axiom Math), *A New Bound for Small Gaps Between Primes*, "Preliminary Draft: September 3, 2026" |
| File | https://primegaps.axiommath.ai/bgp212.pdf, 46 pages, 734,121 bytes, SHA-256 `2b307ae26046dffd2153fe7466e5a9a302976dc3ba5142151a8a8104a37b3d80` |
| Registry | `AXIOM-BGP212-2026-09-03`, status `transcribed` |
| Transcription | `sources/transcriptions/AXIOM-BGP212-2026-09-03.yaml` |

The transcription file holds six reviewed items:

- the definition of admissible (PDF page 3, §2);
- the tuple H₄₅ (page 43, §12);
- the statement of Lemma 12.1;
- the list of primes;
- the list of omitted classes;
- the large-prime sentences.

For each item it records the page, the section, the text exactly as extracted, and the reviewed transcription. It also records the reviewer and date, the pinned tool (`pdftotext` 24.02.0, `-layout -f PAGE -l PAGE`) and the SHA-256 of each page's extraction.

`scripts/check_excerpts.py` re-checks the transcription:

- the PDF's hash;
- the tool version;
- each page's extraction hash;
- each item's extracted text, at word boundaries;
- each displayed number list, parsed from the extraction;
- that each registry excerpt is a reviewed item text.

`scripts/gen-rlt006.py` reads the tuple and the table from this file, so every certificate descends from the checked transcription.

## 2. What each checker establishes

| Checker | Statements | Status |
|---|---|---|
| **RLT kernel** (Robinson's Q, NAT-DEF, Γ = ∅) | **630 element certificates.** `¬(∃k. h = (p × k) + r)` for each listed (p, r) and each h ∈ H₄₅. They are generated, pinned by SHA-256 in `tests/rlt006/manifest.sha256` and in the citations | Kernel-checked |
| **RLT kernel** | **14 per-prime theorems** `admissible.h45.omits.P_R`. Each is the left-nested conjunction of its 45 certificates, cited (45 of 64 cited-file checks) | Kernel-checked |
| Lean comparison 1 (`lean/RLT006Direct.lean`, generated from the transcription) | `om_P`: the 14 statements, symbol for symbol. `omit_iff`: ¬(∃ k, h = p·k + r) ↔ h % p ≠ r for r < p. `lemma_12_1_of_rlt_statements`: the 14 statements, together with `primes_below_47` and `large_moduli` (the paper's counting argument), give admissibility for every prime. `h45_size_and_diameter`: obligation (a). `lemma_12_1`: (a) together with admissibility | Lean-checked |
| Lean comparison 2 (`lean/RLT006BGP212.lean`) | lean-bgp212's H45 and omittedTable equal the transcription (`same_tuple`, `same_table`). Its table result gives RLT's element form (`omits_rlt_form`). Its own Lemma 12.1, re-checked from the vendored text | Lean-checked |
| Generator (Python) | `certificates/h45/residues.tsv`: q and s with h = p·q + s, for all 630 pairs. `certificates/h45/profile.tsv`: the occupied and omitted classes mod each listed prime | Computed evidence; not kernel-checked |

The residue profile is computed, not proved. RLT proves that each tabled class r_p is omitted. That the other classes in the profile are occupied, and that the further omitted classes are omitted, is computation that RLT has not checked.

## 3. Correspondence with Lemma 12.1

Lemma 12.1: "The set H₄₅ is an admissible 45-tuple of diameter 212." Its proof has four obligations:

| Obligation | Where it is established |
|---|---|
| (a) The list has 45 distinct elements, with minimum 0 and maximum 212 | Lean (`h45_size_and_diameter` in comparison 1; lean-bgp212's `h45_card` and `h45_diameter`, re-checked in comparison 2). **RLT: not stated** |
| (b) For p = 2, 3, …, 43, the reductions omit the classes 1, 1, 3, …, 3 | **RLT: the 14 per-prime theorems.** Also in Lean (both comparisons) |
| (c) The primes below 47 are exactly 2, 3, …, 43, so every prime below 47 is one of the 14. The paper leaves this step implicit | Lean (`primes_below_47`). **RLT: not stated** |
| (d) For p ≥ 47, 45 elements cannot cover p classes | Lean (`missing_below` and `large_moduli`, the paper's argument, for every p ≥ 46). **RLT: not proved** |

**Reading over ℕ.** H₄₅ consists of nonnegative integers and 0 ≤ r_p < p. So "h lies in the class r_p mod p" is exactly ∃k ∈ ℕ, h = p·k + r_p, and each RLT statement is the paper's "h avoids the class r_p" (Lean `omit_iff`). The paper speaks of integers; for these elements the natural-number reading is faithful.

**Assessment: partial.** RLT independently verifies obligation (b), all 14 omissions, each from Robinson's Q. RLT does not verify the complete lemma:

- Obligations (a) and (c) are finite and expressible in RLT, but not stated here.
- Obligation (d) quantifies over every prime p ≥ 47. The paper's counting argument for it is a pigeonhole principle over an arbitrary modulus, which RLT has not proved.

In comparison 1, Lean proves admissibility from the 14 statements together with (c) and (d), by the paper's route (`lemma_12_1_of_rlt_statements`). With (a), decided there as `h45_size_and_diameter`, it proves the whole lemma (`lemma_12_1`). Comparison 2 re-checks lean-bgp212's proof of the whole lemma, by its own route.

## 4. The large-prime argument

**The paper's step (d).** For a prime p ≥ 47, the 45 residues h mod p (h ∈ H₄₅) take at most 45 values among the p ≥ 47 classes 0, …, p − 1. So some class r < p is taken by none of them.

**Lean.** Comparison 1 proves exactly this for every p ≥ 46:

- `missing_below`: a list of length less than n misses some number below n, by induction on n;
- applied to `H45.map (· % p)`.

Comparison 1 also shows that the primes below 47 are exactly the table's primes (`primes_below_47`), so (b), (c) and (d) together give admissibility for every prime.

**RLT.** RLT has proved none of (d). It is a statement about every modulus p ≥ 47, and its proof as written uses a counting principle that RLT, without induction, does not have.

One route that stays within Robinson's Q is noted for a later decision; it is not done here. It is lean-bgp212's alternative:

- **Below 44:** RLT would first need a primality statement, written inline as 1 < p ∧ ∀d. ((0 < d ∧ d ∣ p) → d = 1 ∨ d = p). It would then refute primality for 0, 1 and the composites below 44 by divisor witnesses, as in docs/RLT-006-REVIEW.md §9.1, or give omitted-class certificates for them.
- **Moduli 44 … 212:** finite certificates of the kind built here.
- **For every p ≥ 213:** the class 1 is omitted. With 213 ≤ p written as p = S^213(z), suppose h = p·k + 1.
  - Q3 splits k = 0 from k = S(k′).
  - k = 0: mul_zero and add_zero give h = 1, and 1 ∉ H₄₅.
  - k = S(k′): mul_succ, then add_succ applied 214 times with add_zero, gives h = S^214(p·k′ + z). Peeling h ≤ 212 successors (Q1) contradicts Q2.

  These steps instantiate add_zero, add_succ, mul_zero, mul_succ, Q1, Q2 and Q3 at numerals or variables, so no induction is needed.

That route differs from the paper's argument. A record of it would state "same statement, different proof".

## 5. Physical correspondence

Not applicable: these are statements of elementary number theory.

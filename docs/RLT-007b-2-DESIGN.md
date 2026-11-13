# RLT-007b stage 2 — Classification of the primes below 213: design review

**Status:** design review with measured prototypes, 2026-11-12, on branch `rlt-007b-2` (draft). No stage-2 theorem is committed yet. The review found that the definition of "prime" in the approved RLT-007b design cannot carry requirement (1), "each listed number's primality". Section 2 proves why and proposes a replacement, which needs approval because later stages and the composition design state their theorems with it.

**Basis:**
- main at eb0e2ef (stage 1 merged);
- Robinson's Q (ROBINSON-Q), the existing 20 rules plus `cite`, the existing definitions `lt` and `dvd`;
- the existing limits.

**Unchanged:** the kernel, axioms, definitions and limits.

## 1. The six ranges and the partition of 0 … 212 (printed and checked)

The approved ranges, from `docs/RLT-007d-OPTION-B.md` §6.1, are below with their endpoints unaltered. Stage 2 also needs every number below 213 to be classified. Each range is therefore paired with the interval of 0 … 212 that it covers. Each interval runs from the end of the previous one, plus 1, to the number before the next range starts; the last ends at 212.

| Range | Primes (exact endpoints) | Count | Interval of 0 … 212 | Non-primes in interval |
|---|---|---|---|---|
| R₁ | 2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37 | 12 | [0, 40] | 29 |
| R₂ | 41, 43, 47, 53, 59, 61, 67, 71, 73 | 9 | [41, 78] | 29 |
| R₃ | 79, 83, 89, 97, 101, 103, 107, 109 | 8 | [79, 112] | 26 |
| R₄ | 113, 127, 131, 137, 139, 149, 151 | 7 | [113, 156] | 37 |
| R₅ | 157, 163, 167, 173, 179, 181 | 6 | [157, 190] | 28 |
| R₆ | 191, 193, 197, 199, 211 | 5 | [191, 212] | 17 |

**Checks (computed; repeated in Lean, §2.3):**
- **The ranges.** The six ranges hold 47 primes, with no repeats, and together they are exactly the primes below 213.
- **The intervals.** The six intervals partition 0 … 212: consecutive, no gaps, no overlaps. The primes in each interval are exactly that range's primes.
- **Non-primes.** 166 numbers are not prime: 0, 1 and 164 composites. Every number between consecutive ranges is composite: 38–40, 74–78, 110–112, 152–156 and 182–190.

## 2. The definition of prime

### 2.1 The approved design's definition cannot prove any number prime in Q

The RLT-007b design wrote

> Prime(p) :≡ 1 < p ∧ ∀d. ((∃e. p = d × e) → (d = 1 ∨ d = p))

This is the standard definition, and over ℕ it is correct. But Robinson's Q has no induction, so it cannot prove 0 × e = 0 for a variable e. As a result, **Q cannot prove Prime(7)** with this definition, or Prime(q) for any q.

The proof is a model of Q in which the statement is false:
- The elements are the naturals plus a copy of the integers, ns(i).
- S, + and × are standard on the naturals, and S(ns(i)) = ns(i + 1).
- x + ns(i) = ns(i); 0 × ns(i) = 7; (m + 1) × ns(i) = ns((m + 1)·i); ns(j) × y = ns(j) for every y ≠ 0.
- **In this model:** 0 × ns(0) = 7, so 0 "divides" 7, and 0 is neither 1 nor 7.

**Checked in Lean.** `lean/RLT007S2Design.lean`, using core Lean only:
- `model_of_Q`: all seven axioms of ROBINSON-Q hold in this structure;
- `unbounded_fails_for_7`: the definition fails there for 7.

By the soundness of the kernel's rules (`docs/KERNEL-SOUNDNESS.md`), no RLT proof under ROBINSON-Q can conclude Prime(7). The design used Prime only as a hypothesis, to refute non-primes, and that use is still possible. But requirement (1), each listed number's primality, cannot be met with it.

### 2.2 Proposed definition (bounded divisors)

> Prime(p) :≡ 1 < p ∧ ∀d. ((1 < d ∧ d < p) → ¬(d ∣ p))

It is written inline with the existing `lt` and `dvd` definitions (`x < y` is ∃z. S(z) + x = y; `d ∣ n` is ∃k. n = d × k), so no new definition is added. As an RLT formula:

`(and (lt 1 p) (forall d (imp (and (lt 1 d) (lt d p)) (not (dvd d p)))))`

**Why Q can prove it for every prime numeral q:**
- 1 < q has a closed witness.
- For any d with 1 < d and d < q, the hypothesis d < q unfolds to S(z) + d = q. Finitely many Q3 case splits then show that d is one of the numerals 2 … q − 1.
- For each such numeral d, ¬(d ∣ q) follows by case splits on k, as in RLT-006 and RLT-007c.

No step needs induction.

**Equivalence with the standard definition over ℕ, checked in Lean** (`primeB_iff`):

> (1 < p ∧ ∀d. (1 < d ∧ d < p) → ¬ d ∣ p) ↔ (2 ≤ p ∧ ∀d. d ∣ p → d = 1 ∨ d = p)

`lt_iff` and `dvd_iff` check that RLT's `<` and `∣` are ℕ's.

**Scope of the equivalence.** The equivalence is a statement about ℕ, which is what the paper's lemma is about. In non-standard models of Q the two definitions can differ, for example where p < p holds for a non-standard p. The classification and composition claims are therefore read over ℕ, through the kernel's soundness: a theorem of Q holds in ℕ. That reading is recorded in the correspondence review.

### 2.3 The classification in Lean

`classification` decides that for every p < 213, the bounded definition holds exactly when p is in R₁ ++ … ++ R₆. All four Lean theorems use standard axioms only:

| Theorem | Axioms |
|---|---|
| `unbounded_fails_for_7` | propext |
| `primeB_iff` | propext, Quot.sound |
| `model_of_Q` | propext, Classical.choice, Quot.sound |
| `classification` | propext, Quot.sound |

## 3. The theorems stage 2 would add (all unconditional under ROBINSON-Q)

| Family | Statement | Count |
|---|---|---|
| (1) Primality | Prime(q) for each listed q | 47 |
| (2) Non-primality | for each interval i, ⋀_{m nonprime in interval i} ¬Prime(m). For 0 and 1, from ¬(1 < m); for a composite m, a closed divisor witness: m = d × p with p its smallest prime factor (≤ 13) and 1 < d < m | 6 theorems covering all 166 non-primes |
| (3) Exhaustiveness per range | E₁: ∀p. ((p < 41 ∧ Prime(p)) → R₁(p)); Eᵢ (i = 2 … 6): ∀p. (((aᵢ − 1 < p ∧ p < bᵢ + 1) ∧ Prime(p)) → Rᵢ(p)), with [aᵢ, bᵢ] the interval of §1 and Rᵢ(p) the disjunction p = q over the range's primes. Each cites its interval's (2) theorem | 6 |
| Top classification | ∀p. (Prime(p) → (R₁(p) ∨ … ∨ R₆(p) ∨ 212 < p)), cited from an order split ∀p. (p < 41 ∨ (40 < p ∧ p < 79) ∨ … ∨ 212 < p) and the six Eᵢ | 1 order-split theorem + 1 summary |

The top classification is the layer-0 component `primes.below_213` of the composition design. Its disjunction is grouped by the six ranges. The per-range theorems in (3) are the quantified exhaustiveness theorems requested, checked under Robinson's Q.

## 4. Measured prototypes and sizes

**Family (1) is the expensive part.** I prototyped a scratch generator, untrusted and not committed, and checked its output with the current `rlt`:
- ¬(d ∣ 7) for d = 2 … 6, as one file: accepted, 413 steps.
- Building the same proofs for every listed prime q and every d = 2 … q − 1 measures **76.7 million nodes**:
  - q = 97: 1.20 million;
  - q = 151: 2.86 million;
  - q = 211: 5.56 million.
- 24 of the 47 primes exceed the 1,000,000-node file limit as a single file.

This uses the cancellation scheme, ∀v∀w. S^d(v) = w + d → v = w. Its in-file infrastructure alone costs 1.3 million nodes at q = 211.

**Planned layout for (1), within the existing limits.**
- **Scheme.** A hybrid: cancellation for small d ≤ 14, where the infrastructure is small; for larger d, expansion of k as d × S^j(k′) = S^{jd}(d × k′), closed by the stage-1 K-lemmas, with no cancellation infrastructure.
- **Files per prime.** The non-divisibility proofs for one prime are split by d-ranges into files of at most 700,000 nodes. Each file proves ∀d. ((a < d ∧ d < b) → ¬(d ∣ q)).
- **Summary per prime.** The summary Prime(q) cites those files (at most about 6) and enumerates d ∈ 2 … q − 1.
- **Size (estimated):** about 50 million nodes, about 90 generated files (gitignored and manifest-pinned, as in RLT-006 and stage 1), plus 47 committed summaries.

**Families (2) and (3) are small:**
- composites by divisor witnesses: about 2–3 million nodes in total, under 30,000 per composite;
- interval enumerations: about 41 Q3 levels per interval with small formulas.

These are estimates; they are measured first in implementation.

**Verification cost (estimate).** About 50 million nodes adds an estimated 4–8 CPU-minutes per complete run, parallelized by RLT-007a's scheduling. Each summary is at most 7 of 64 cited-file checks, chain 1–2.

**Citation use (planned):**

| Theorem | Cited-file checks |
|---|---|
| Prime(q) summary | at most 7 |
| Eᵢ | 1 |
| top classification | 7 |

All are well within 64.

## 5. Negative tests (planned)

- **An omitted prime:** a range disjunction missing one prime; Eᵢ's proof cannot close that branch, and a forged file is rejected.
- **A falsely included composite:**
  - 91 (= 7 × 13) listed in R₃: a Prime(91) proof is rejected;
  - the generator refuses to list it.
- **Altered assumptions:**
  - a summary stating the unbounded definition;
  - an Eᵢ with a widened interval;
  - a non-primality theorem with the wrong divisor witness.
- **Duplicate entries:** a prime listed twice; the generator's source check refuses it, and the duplicated disjunct changes the fingerprint, so composition matching (§3.1 of Option B) refuses it.
- **Incorrect range boundaries:**
  - Eᵢ with an interval ending one too early or one too late;
  - an order split leaving a gap.
- **Stale citations:** a modified, missing or substituted generated file; a stale SHA-256 in a summary.

## 6. Source correspondence

The paper's Lemma 12.1 proof names the 14 primes up to 43 (transcription item `omitted-primes`). It does not list the primes from 47 to 211 at all.

The classification below 213 is therefore new mathematics checked here, not a transcribed fact:
- Its correctness is **kernel accepted** (families 1–3), and Lean decides it (§2.3).
- The only **source transcription checked** fact it uses is that the 14 transcribed primes are the first 14 entries of R₁ ∪ R₂, which the generator would check.
- Its role in the lemma, "every prime p", is **semantic correspondence assessed** in a stage-2 correspondence review. That review records the bounded definition and its Lean equivalence with the standard one.

## 7. Decisions requested before implementation

1. **The definition.** Adopt the bounded definition of §2.2 for stage 2, and in the composition design (`primes.below_213` and Adm-all) in place of the unbounded one. The unbounded definition cannot meet requirement (1).
2. **The intervals.** Confirm the six intervals of §1 as the partition used by the exhaustiveness theorems; the range endpoints are unchanged.
3. **The cost of family (1).** Confirm that each listed prime's primality is required as a kernel theorem, at an estimated 50 million nodes and about 90 generated files. Exhaustiveness and the composition step do not depend on family (1); they use only families (2) and (3). Family (1) could be a separately merged stage 2b, after the classification itself (2a).

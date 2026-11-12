# RLT-005 — Published-mathematics demonstration: candidate and architecture review

**Status.** Research and architecture review, 2026-11-11, against main at c42729b. Nothing here is implemented. No new axioms, induction, binary numerals, inference rules, release or deployment.

**Objective.** Carry one exact statement from an independently published paper through `rlt check` (proof), `rlt explain` (source, specification, correspondence) and `rlt deps` (reusable dependencies). The statement should answer a question its authors might ask, and RLT should prove it from Robinson's Q within the measured limits (docs/RLT-004a-BASELINE.md).

**Recommendation.** RLT-005 should prove that **8 is not matchable**, from McNew and Pomerance, *Matchable numbers*, arXiv:2604.05304v2 (23 May 2026), §1. Candidate B below (the Axiom Math H₄₅ admissibility table) is the best follow-up, since its finite part is checkable as 14 separate theorems.

---

## 1. Candidates

### A. "8 is not matchable" — McNew and Pomerance, *Matchable numbers*

- **Source.** Nathan McNew and Carl Pomerance, arXiv:2604.05304, version 2 dated 23 May 2026 (math.NT). Section 1, in the paragraph beginning "One might think at first…". The sentence: "8 is not matchable, nor is any subsequent multiple of 4."
- **Definitions in the paper (§1).**
  - A bijection between two finite sets of integers is a coprime matching when each element and its image are coprime.
  - n is matchable when there is a coprime matching between {1, …, τ(n)} and D(n), the set of divisors of n.
  - The paper's reason, in brief: even divisors must be matched to odd numbers.
- **Assumptions and definitions for the formal goal.**
  - D(8) = {1, 2, 4, 8} and τ(8) = 4.
  - Coprime is written inline as ∀k. (k ∣ a ∧ k ∣ b) → k = 1, using the registered `∣`, so no new definition is needed.
  - A bijection onto {1, 2, 3, 4} is written as four images, each in [1, 4], pairwise distinct. For two 4-element sets, an injection is a bijection.
- **Formal goal (NAT-DEF):**
  ```
  ¬∃a. ∃b. ∃c. ∃d.
      (0 < a ∧ a < 5) ∧ (0 < b ∧ b < 5) ∧ (0 < c ∧ c < 5) ∧ (0 < d ∧ d < 5)
    ∧ ¬(a = b) ∧ ¬(a = c) ∧ ¬(a = d) ∧ ¬(b = c) ∧ ¬(b = d) ∧ ¬(c = d)
    ∧ cop(1, a) ∧ cop(2, b) ∧ cop(4, c) ∧ cop(8, d)
  ```
  Here a, b, c, d are the partners of the divisors 1, 2, 4 and 8, and each cop(x, y) is written out inline.
- **Expressible in the current kernel:** yes. It uses only 0, S, +, ×, =, the connectives and quantifiers, and the registered `<` and `∣`.
- **Robinson's Q sufficient:** yes. The proof follows the paper's reason.
  1. Take each of b, c and d with cop(even, ·). Instantiating k = 2, with 2 ∣ 2, 2 ∣ 4 and 2 ∣ 8 already in the library, gives ¬(2 ∣ ·), since 2 = 1 is refuted by Q1 and Q2.
  2. A bounded enumeration below 5, by the M3d scheme, together with 2 ∣ 2 and 2 ∣ 4, leaves only {1, 3} for each of them.
  3. Three pairwise distinct values in a 2-element set: each of the 8 cases contradicts one disequation.
  4. a is not constrained by the argument.
- **Cost estimate (measured scheme sizes):**
  - The enumeration below 5 is 121 steps, accepted in 0.27 s, measured on the current binary for this review.
  - The case analysis is roughly 1,000–2,000 steps on numerals of at most 8: far below the 1,000,000-node file limit.
  - Citations: about 5 cited-file checks of 64, with a chain of 1–2 of 8.
- **Library reuse:** `dvd.2_2`, `dvd.2_4`, `dvd.2_8` (M3e), plus a new `order.below.5` (DA scheme, bound 5). A new entry `matchable.not.8` would join `lib/`.
- **Lean 4 comparison.**
  - **Tier 1:** the same bounded formula, restated over `Nat` and decided by `decide`. Two independent systems would then accept one statement.
  - **Tier 2:** a reading closer to the paper, for example that no `e : Nat.divisors 8 ≃ Finset.Icc 1 4` has `Nat.Coprime x (e x)` for all x, decided by finite enumeration. This would check the semantic step from RLT's encoding to the paper's definitions over ℕ, as a Lean theorem attributed to Lean.
- **Proof, fidelity and correspondence, kept apart:**
  - *Kernel:* the formula above follows from Robinson's Q.
  - *Source fidelity:* the quoted sentence, pinned to v2 in `sources/registry.yaml`.
  - *Semantic correspondence:* partial until reviewed. The encoding fixes D(8) and τ(8) by hand, and the paper's further claim ("nor is any subsequent multiple of 4") is outside the theorem. The Lean tier-2 statement is offered as evidence, attributed.
  - *Physical correspondence:* not applicable.

### B. H₄₅ omits a residue class for every prime up to 43 — Axiom Math, *A New Bound for Small Gaps Between Primes*

- **Source.** Charton, Hong, Lau, Ono, Remy, Siu, Swaminathan, Thorner and Xie (Axiom Math), "Preliminary Draft: September 3, 2026", at primegaps.axiommath.ai/bgp212.pdf.
  - Lemma 12.1: "The set H45 is an admissible 45-tuple of diameter 212."
  - Section 12 lists the 45 elements. For each prime p ≤ 43 it gives one omitted residue class (1, 1, 3, 6, 9, 3, 1, 13, 17, 4, 11, 4, 4, 3, for p = 2, 3, …, 43). For p ≥ 47 it argues by counting.
  - The draft carries no version number, so it must be pinned by file hash.
  - The paper's main result, H₁ ≤ 212 (Theorem 1.1), uses this lemma through DHL[45, 2].
- **Definition (paper, §2).** H is admissible when, for every prime p, H omits at least one residue class mod p.
- **Formal goals.** For each of the 14 primes p with omitted class r_p: ∀ listed h. ¬∃k. h = p × k + r_p. That is 45 closed facts per prime, combined by and-intro.
- **Expressible:** the 14 finite statements, yes. The full lemma is a statement about every prime, including the counting argument for p ≥ 47; that needs reasoning about an arbitrary p, which RLT can only do with induction.
- **Robinson's Q sufficient:** for the 14 finite statements, yes, by the M3c non-divisibility scheme in a residue form. That form is an untrusted generator variant; the kernel is unchanged.
- **Cost estimate:**
  - 630 certificates on numerals up to 255. Measured here: ¬(2 ∣ 213) took 3,321 steps in 0.28 s; ¬(43 ∣ 255) took 1,416 steps in 0.25 s. All are well under the node limit.
  - Per-prime theorems are within limits (p = 2 cites 45 files: 45 of 64).
  - One theorem for all 14 primes would need 644 cited-file checks against a budget of 64, so it is outside current limits.
  - Checking time is about 3 minutes in total.
- **Library reuse:** none of the existing dvd entries; it would add a new `admissible` area of 644 entries.
- **Lean 4 comparison:**
  - Tier 1: each residue fact by `decide`.
  - Tier 2: the full lemma, using Mathlib's primes and a counting argument. Lean could prove what RLT cannot (the p ≥ 47 part).
- **Separation:**
  - *Kernel:* 14 finite residue statements.
  - *Source fidelity:* the element list and the residue table, pinned to the PDF's hash.
  - *Semantic correspondence:* the step from "omits class r mod p" to "h ≠ p·k + r" is reviewable. The full lemma stays partial: the counting step for large primes is the paper's argument, outside RLT's proof.
- **Pre-implementation check needed.** The element list above was transcribed through a document-reading tool. It must be compared character by character with the PDF before use. A quick untrusted computation confirms that the transcribed list has 45 elements, diameter 212 and every listed omitted class; that is consistency only.

### C. A = 120 · 264 · 80 · 25 = 63360000 = 11 · 2400² — Qiu, *Infinite rational distance sets in affine general position*

- **Source.** Junkai Qiu, arXiv:2608.23529, proof of Proposition 6.2 (§6.1). The display reads "A = 120 · 264 · 80 · 25 = 63360000 = 11 · 2400²". The arXiv version must be pinned before use; it was not visible in the retrieved text.
- **Formal goal (NAT-DEC):** two closed equations, 120 × 264 × 80 × 25 = 63360000 and 11 × 2400 × 2400 = 63360000.
- **Expressible:** yes, as closed arithmetic.
- **Robinson's Q sufficient:** in principle, yes. In practice, no. Numerals are unary, so 63,360,000 alone is 63,360,001 kernel nodes, far above the 1,000,000-node file limit. `rlt gen` also refuses numerals above 1000.
- **Cost:** outside current limits by a factor of about 60 for the numeral alone, before any proof steps.
- **Library reuse:** none.
- **Lean 4 comparison:** `norm_num` or `decide` proves it immediately; with Lean's binary numerals this is trivial there.
- **Separation:** the identity is a computation inside a proof. The proposition's mathematical content (a quadratic-form obstruction) is far outside RLT's signature.

## 2. Ranking

| | Mathematical relevance | Feasibility now | Value to the authors | Overall |
|---|---|---|---|---|
| **A. 8 is not matchable** | moderate: the paper's motivating example and the first case of its Proposition 1 pattern | **complete**: the exact statement, fully proved, about 5 citations | moderate: an independent kernel proof of a stated fact, with a Lean reading of their definition | **1** |
| **B. H₄₅ residue table** | high: the finite certificate behind H₁ ≤ 212 | partial: 14 per-prime theorems are feasible; the full lemma needs reasoning about every prime | high: an independent check of a table the paper states without showing the reductions | **2** |
| **C. 120·264·80·25 = 11·2400²** | low: an arithmetic step | none under unary numerals | low | **3** |

## 3. Proposed RLT-005 scope (on approval)

1. **Source.** Register McNew and Pomerance v2 in `sources/registry.yaml`: the URL, the version, and the two §1 sentences as excerpts, verified against the arXiv HTML or source.
2. **Proofs**, generated by untrusted scripts and kernel-checked:
   - `order.below.5`;
   - the per-variable lemmas "cop(2, x) ∧ 0 < x < 5 → x = 1 ∨ x = 3", and likewise for 4 and 8;
   - the main theorem `matchable.not.8`, citing them.
3. **Statement record** `statements/MATCHABLE-NOT-8.yaml`:
   - semantic status *partial*: attributed, it notes the hand-fixed D(8) and τ(8) and that "every subsequent multiple of 4" is excluded;
   - physical correspondence not applicable.
4. **Library entries and identities.** `rlt deps matchable.not.8` would show the graph (about 6 nodes).
5. **Lean:**
   - tier 1 by `decide`;
   - tier 2 against `Nat.divisors 8` and an `Equiv`, if Lean decides it in reasonable time; otherwise recorded as a limitation.
6. **Tests and audit:**
   - adversarial fixtures: a wrong divisor set, a non-injective matching, an omitted coprimality conjunct, a swapped quantifier;
   - byte-for-byte reports against main;
   - an independent audit.
7. **Manual:** an example chapter, "A theorem from a published paper".

**Optional, recorded for approval:** "n = 1, …, 7 are matchable" (same sentence) as existential companions, each with an explicit matching. Coprimality of closed pairs needs bounded divisor enumeration, which the schemes already provide.

## 4. Limitations and notes

- **Alekseyev's paper.** "Coprime matchings of divisor sets" (arXiv 2610.37512, noted in earlier work) could not be retrieved from this session, so no statement from it is proposed. McNew and Pomerance discuss the related conjecture of König and Alekseyev that every M-number is matchable.
- **Scale.** RLT can prove candidate A in full. B is possible only in pieces and C not at all. That is the current boundary, set by unary numerals, the citation budget and Q without induction. The separate numerals and induction review (deferred at RLT-004a) is what would move it.
- **Gaps relative to the paper.** Each statement record will say plainly what the theorem leaves out of the paper's claim. For A, that is the infinite family of multiples of 4; for B, the large primes.
- **Branch protection.** Main is unprotected (see the handoff). The append-only identity rule assumes main's history is never rewritten; enabling protection is recommended before RLT-005 adds new identities.

## Sources consulted

- McNew and Pomerance, *Matchable numbers*, arXiv:2604.05304v2: https://arxiv.org/html/2604.05304v2
- Axiom Math, *A New Bound for Small Gaps Between Primes* (preliminary draft, September 3, 2026): https://primegaps.axiommath.ai/bgp212.pdf
- Qiu, *Infinite rational distance sets in affine general position*: https://arxiv.org/pdf/2608.23529

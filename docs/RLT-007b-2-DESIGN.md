# RLT-007b stage 2 — Classification of the primes below 213: design review

**Status:** design review with measured prototypes, 2026-11-12, on branch `rlt-007b-2` (draft). The decisions of §7 were approved on 2026-11-12 (recorded in §0). Stage 2a is implemented on this branch (`docs/RLT-007b-2a-REPORT.md`).

## 0. Decisions approved on 2026-11-12

1. **Definition B, bounded primality**, for stage 2 and the composition design: Prime_B(p) :≡ 1 < p ∧ ∀d. ((1 < d ∧ d < p) → ¬(d ∣ p)), with the existing meanings of `<` and `∣`. G (§2.2) stays documented as a reference definition. The correspondence of B, G and the standard definition is a Lean result over ℕ (`primeB_iff`, `primeG_iff`; in stage 2a `pb_iff_std`, `pg_iff_std`), not an RLT theorem under Robinson's Q. The countermodel of §2.1 is kept as the record of why the unguarded definition was set aside.
2. **The six intervals of §1**, with their complete prime lists; one quantified, kernel-checked exhaustiveness theorem per range; together they classify every natural number below 213.
3. **Split implementation.** Stage 2a: exhaustiveness and non-prime certificates (implemented). Stage 2b: the 47 positive primality theorems under Prime_B, after stage-2a review, merged only with separate approval. Stage-2a exhaustiveness, stage-2b primality and the eventual assembly stay three distinct results.

Where this review writes "Prime" below, the approved reading is Prime_B. The recommendation of G in §2.2 and §7 is kept as the review's history.

**Implementation note (stage 2a).** The single order-split theorem of §3 became six relative splits O_1 … O_6 (`p < 41 ∨ 40 < p`, and `a − 1 < p → (p < b + 1 ∨ b < p)`), because one absolute split with all bounds measured 1.21 million nodes, above the 1,000,000 limit. Family (2) is six certificates, one per interval.

**Main finding.** The definition of "prime" written in the RLT-007b design cannot carry requirement (1), "each listed number's primality". That requirement comes from the stage-2 instruction of 2026-11-12, item B.1. Section 2 proves why and proposes two replacements; choosing one needs approval, because later stages and the composition design state their theorems with it.

**Independent review.** An independent mathematical audit found no blocking defect, and its corrections are included (§8).

**Basis:**
- main at eb0e2ef (stage 1 merged);
- Robinson's Q (ROBINSON-Q), the existing 20 rules plus `cite`, the existing definitions `lt` and `dvd`;
- the existing limits.

**Unchanged:** the kernel, axioms, definitions and limits.

## 1. The six ranges and the partition of 0 … 212 (printed and checked)

The ranges are those of `docs/RLT-007d-OPTION-B.md` §6.1 (PR #16, branch `docs/rlt-007-next`). Their exact endpoints were confirmed in the instruction of 2026-11-12, item C ("Confirm the six prime ranges exactly as specified in the reviewed design"). They are listed below unaltered. Stage 2 also needs every number below 213 to be classified. Each range is therefore paired with the interval of 0 … 212 that it covers. Each interval runs from the end of the previous one, plus 1, to the number before the next range starts; the last ends at 212.

| Range | Primes (exact endpoints) | Count | Interval of 0 … 212 | Non-primes in interval |
|---|---|---|---|---|
| R₁ | 2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37 | 12 | [0, 40] | 29 |
| R₂ | 41, 43, 47, 53, 59, 61, 67, 71, 73 | 9 | [41, 78] | 29 |
| R₃ | 79, 83, 89, 97, 101, 103, 107, 109 | 8 | [79, 112] | 26 |
| R₄ | 113, 127, 131, 137, 139, 149, 151 | 7 | [113, 156] | 37 |
| R₅ | 157, 163, 167, 173, 179, 181 | 6 | [157, 190] | 28 |
| R₆ | 191, 193, 197, 199, 211 | 5 | [191, 212] | 17 |

**Checks (computed, and decided in Lean by `ranges_facts` and `classification`, §2.3):**
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
- Also ns(j) + n = ns(j + n) for a numeral n, and ns(j) × 0 = 0.
- **In this model:** 0 × ns(0) = 7, so 0 "divides" 7, and 0 is neither 1 nor 7. Replacing 7 by any numeral q in the definition of 0 × ns(i) gives the same failure for q; Lean checks q = 7.

**Checked in Lean.** `lean/RLT007S2Design.lean`, using core Lean only:
- `model_of_Q`: all seven axioms of ROBINSON-Q hold in this structure;
- `unbounded_fails_for_7`: the definition fails there for 7.

By the soundness of the kernel's rules (`docs/KERNEL-SOUNDNESS.md`), no RLT proof under ROBINSON-Q can conclude Prime(7). The design used Prime only as a hypothesis, to refute non-primes, and that use is still possible. But requirement (1), each listed number's primality, cannot be met with it.

### 2.2 Two definitions Q can prove for prime numerals

Both are written inline with the existing `lt` and `dvd` definitions (`x < y` is ∃z. S(z) + x = y; `d ∣ n` is ∃k. n = d × k), so no definition is added. Both equal the standard definition over ℕ, checked in Lean. They differ only in how they exclude the divisor 0, which is exactly what Q cannot handle.

**History.** RLT-007-REVIEW §A row (c) proposed the guarded form G below. RLT-007b's design dropped the guard `0 < d`, and that is what makes its definition unprovable.

**G — guarded (recommended):** "p > 1, and its only positive divisors are 1 and p", the textbook wording.

> Prime(p) :≡ 1 < p ∧ ∀d. ((0 < d ∧ d ∣ p) → (d = 1 ∨ d = p))

`(and (lt 1 p) (forall d (imp (and (lt 0 d) (dvd d p)) (or (= d 1) (= d p)))))`

*Why Q proves it for every prime numeral q:*
- 0 < d unfolds to d = S(z).
- From q = d × k: if k = 0, `mul_zero` and Q2 refute it.
- If k = S(k′), `mul_succ` gives q = (d × k′) + d.
- Q3 on d, then Q1 at each level, then Q2 at depth q, bound d to the numerals 1 … q. `add_succ` peels the right argument, which is d.
- d = 1 and d = q are the two allowed disjuncts. For each numeral 2 ≤ d < q, ¬(d ∣ q) follows by Q3 on k.
- No induction is needed.

In the countermodel the non-standard elements are positive but divide no numeral. `guarded_ns_not_dvd_7` checks that for 7.

**B — bounded:**

> Prime(p) :≡ 1 < p ∧ ∀d. ((1 < d ∧ d < p) → ¬(d ∣ p))

*Why Q proves it for every prime numeral q:* d < q unfolds to S(z) + d = q. Finitely many Q3 case splits show d is one of the numerals 2 … q − 1, and each ¬(d ∣ q) follows by Q3 on k.

**Lean (core Lean only, standard axioms):**
- `primeG_iff`: G is equivalent over ℕ to the standard definition, 2 ≤ p ∧ ∀d. d ∣ p → d = 1 ∨ d = p.
- `primeB_iff`: the same for B.
- `lt_iff` and `dvd_iff`: RLT's `<` (with the order (z + 1) + x = y) and `∣` are ℕ's.

**Comparison.**
- G keeps the standard shape: only a positivity guard is added. It needs d ≤ q to be derived from the divisibility itself.
- B states trial division directly.
- Both cost about the same for families (1)–(3).
- G reads closer to the paper's notion of prime, so the correspondence review is shorter. **Recommended: G.**

**Scope of the equivalence.** The equivalence is a statement about ℕ, which is what the paper's lemma is about. In non-standard models of Q the definitions can differ. In the countermodel, at the standard element 7, both G and B hold while the unbounded definition fails. The classification and composition claims are read over ℕ through the kernel's soundness: a theorem of Q holds in ℕ. That reading is recorded in the correspondence review.

### 2.3 The classification in Lean

- `classification` decides that for every p < 213, the bounded definition holds exactly when p is in R₁ ++ … ++ R₆.
- `ranges_facts` decides that the 47 listed primes have no repeats and are all below 213, and that each range equals the primes of its interval. All seven Lean theorems in the table use standard axioms only:

| Theorem | Axioms |
|---|---|
| `unbounded_fails_for_7` | propext |
| `primeB_iff` | propext, Quot.sound |
| `model_of_Q` | propext, Classical.choice, Quot.sound |
| `classification` | propext, Quot.sound |
| `ranges_facts` | propext |
| `primeG_iff` | propext, Quot.sound |
| `guarded_ns_not_dvd_7` | propext |

## 3. The theorems stage 2 would add (all unconditional under ROBINSON-Q)

| Family | Statement | Count |
|---|---|---|
| (1) Primality | Prime(q) for each listed q | 47 |
| (2) Non-primality | for each interval i, ⋀_{m nonprime in interval i} ¬Prime(m). For 0 and 1, from ¬(1 < m). For a composite m, a closed divisor witness: m = d × s, with s its smallest prime factor (≤ 13; 169 = 13² is the largest case) and 1 < d < m | about 7 theorems covering all 166 non-primes; interval [113, 156] (37 composites) is split in two if it exceeds 700,000 nodes |
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

**Families (2) and (3), and the summaries, are estimates still to be measured:**
- composites by divisor witnesses: about 2–3 million nodes in total, under 30,000 per composite;
- interval enumerations: 22–45 Q3 levels per interval, with small formulas;
- **the order split:** 213 numeral leaves, each needing closed `lt` witnesses. Done naively this is about 1 million nodes or more, at the file limit. Plan: per-file lemmas ∀w. w + n = S^n(w), as in stage 1, or splitting relative to each interval boundary;
- **the Prime(q) summaries:** enumerating d ∈ 2 … q − 1 (G: 1 … q) with bound proofs at each leaf may approach the limit for q = 211. The same lemma technique applies, with reviewed intermediate summaries if needed, never larger limits.

**Verification cost (estimate).** About 50 million nodes adds an estimated 1–8 CPU-minutes per complete run, parallelized by RLT-007a's scheduling. The low end is stage 3's measured rate (32.7 million nodes in 43 s); the high end allows for heavier steps.

**Citation use (planned):**

| Theorem | Cited-file checks |
|---|---|
| Prime(q) summary | at most 7 |
| Eᵢ | 1 |
| top classification | 13 in all: 7 direct, plus the 6 nested citations of the Eᵢ (the budget counts transitively; chain 2) |

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
- Its correctness will be **kernel accepted** once families 1–3 are implemented. Lean decides it now (§2.3).
- The only **source transcription checked** fact it uses is that the 14 transcribed primes are the first 14 entries of R₁ ∪ R₂, which the generator would check.
- Its role in the lemma, "every prime p", is **semantic correspondence assessed** in a stage-2 correspondence review. That review records the bounded definition and its Lean equivalence with the standard one.

## 7. Decisions requested before implementation

1. **The definition.** Adopt G, the guarded form (recommended), or B, the bounded form (§2.2), for stage 2 and in the composition design (`primes.below_213` and Adm-all), in place of the unbounded one. The unbounded definition cannot meet requirement (1).
2. **The intervals.** Confirm the six intervals of §1 as the partition used by the exhaustiveness theorems; the range endpoints are unchanged.
3. **The cost of family (1).** Confirm that each listed prime's primality is required as a kernel theorem, at an estimated 50 million nodes and about 90 generated files. Exhaustiveness and the composition step do not depend on family (1); they use only families (2) and (3). Family (1) could be a separately merged stage 2b, after the classification itself (2a).

## 8. Independent mathematical audit (of c8de7e7)

**Verdicts:**
- countermodel valid: yes;
- bounded definition correct and Q-provable for prime numerals: yes;
- plan sound: yes;
- unresolved substantive issue: no.

**What it checked.**
- **The model.** All seven axioms against `spec/robinson-q.rlt`, argument order included, with the tricky cases checked by hand.
- **Soundness.** It extends to non-standard structures, and RLT numerals denote the standard elements.
- **Feasibility of the sketched proofs.** None needs commutativity, associativity or induction.
- **Arithmetic.** The ranges, intervals and counts, recomputed independently.
- **Sizing.** The size figures fit a quadratic model (76.1 million predicted, 76.7 million measured).

**Its findings, all addressed in this revision:**

| Finding | Resolution |
|---|---|
| The Lean `classification` did not check repeats, entries of 213 or more, or the per-interval split | `ranges_facts` decides them |
| The guarded form (0 < d) from RLT-007-REVIEW was not considered | Added as option G, recommended, with Lean equivalence and the model check |
| The sources of the range approval and of requirement (1) were not cited | Cited (status note, §1) |
| The order split and the Prime(q) summaries were not sized | Listed as estimates with their planned techniques (§4) |
| The citation count was given as 7 | 13 in all, counted transitively (§4) |
| The scope example and some wording | Corrected (§2.1–§2.2, §3, §6) |

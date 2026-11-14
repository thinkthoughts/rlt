# RLT-007b stage 1 — Distinctness and diameter 212: implementation proposal

**Status:** proposal for confirmation before implementation, 2026-11-12. The numbers below come from scratch proofs: an untrusted prototype generator, with each file checked by the current `rlt` binary under the existing limits. Nothing here is committed as a proof.

**Basis.** Robinson's Q (ROBINSON-Q), the existing 20 rules plus `cite`, and convention NAT-DEF. No new definition is used: "≤" is written inline as ∃z. z + x = y.

## 1. Source correspondence

**Source:** Axiom Math, *A New Bound for Small Gaps Between Primes* (September 3, 2026 draft). Registry `AXIOM-BGP212-2026-09-03`, PDF SHA-256 `2b307ae2…3d80`, through the reviewed transcription `sources/transcriptions/AXIOM-BGP212-2026-09-03.yaml`. The relevant excerpts are already registered:

- "H₄₅ = {0, 2, 12, …, 210, 212}." (the tuple, 45 numerals in increasing order)
- "Lemma 12.1. The set H₄₅ is an admissible 45-tuple of diameter 212."

**Reading.** Obligation (a) of Lemma 12.1 has two parts:

1. **"45-tuple":** the 45 listed integers are pairwise distinct.
2. **"diameter 212":** max H₄₅ − min H₄₅ = 212. With 0 and 212 both listed, this holds exactly when every element h satisfies 0 ≤ h ≤ 212.

That 0 and 212 are listed, and that the list has 45 entries, are syntactic facts about the transcription. The generator checks them, and the correspondence review records them. They are not proved by the kernel.

Over ℕ, ∃z. z + x = y is equivalent to x ≤ y. The correspondence document will state this reading, as RLT-006's did for ¬∃k. h = p·k + r.

## 2. Exact theorem statements

The written forms below are exactly what the generator produces and `rlt check` prints. h₀ < h₁ < … < h₄₄ is the transcribed tuple.

### 2.1 Per-element distinctness: 44 theorems

For i = 0, …, 43 (every element but the last):

> `admissible.h45.distinct.H` (H = hᵢ, three digits), goal
> `((…(¬(hᵢ = hᵢ₊₁) ∧ ¬(hᵢ = hᵢ₊₂)) ∧ …) ∧ ¬(hᵢ = h₄₄))`
> left-nested, j increasing; 44 − i conjuncts; unconditional under ROBINSON-Q.

**Examples:**
- `admissible.h45.distinct.200`: `(((¬(200 = 204) ∧ ¬(200 = 206)) ∧ ¬(200 = 210)) ∧ ¬(200 = 212))`
- `admissible.h45.distinct.210`: `¬(210 = 212)`

Together, the 44 theorems state all 990 pairwise disequalities.

### 2.2 Grouped distinctness: 4 summary theorems

`admissible.h45.distinct.g0` … `g3`, each citing 11 consecutive per-element theorems:
- g0: h₀–h₁₀
- g1: h₁₁–h₂₁
- g2: h₂₂–h₃₂
- g3: h₃₃–h₄₃

**Goal:** the left-nested conjunction of the cited goals, in order. Each uses 11 of 64 cited-file checks, chain 1 of 8.

**Why four groups and not one statement.** A single 990-conjunct formula would hold 221,056 in numeral total, above the parser's existing limit of 200,000 per written formula (`MAX_NUMERAL_TOTAL`). The largest group holds 71,676, so four groups is the coarsest split within the limits. Composition (stage 5) will use these four statements as its distinctness assumptions.

### 2.3 Bounds: 1 theorem

> `admissible.h45.bounds`, goal
> `((((∃z. z + 0 = 0) ∧ (∃z. z + 0 = 212)) ∧ ((∃z. z + 0 = 2) ∧ (∃z. z + 2 = 212))) ∧ …)`
> that is, ⋀_{h ∈ H₄₅} ((∃z. z + 0 = h) ∧ (∃z. z + h = 212)), left-nested over the 45 elements in order; unconditional.

The bounds use one convention throughout: z + x = y, with the witness on the left, which is the cheaper direction in Q.

## 3. Proof methods

### 3.1 Distinctness

One file per element a = hᵢ:

1. **A chain of lemmas** K₀, …, K_a, with K_j :≡ ∀u. ¬(j = S^{j+1}(u)):
   - K₀ is Q2 instantiated at a parameter and generalized.
   - K_j comes from K_{j−1}: assume j = S^{j+1}(u); Q1 gives j − 1 = S^j(u); K_{j−1} at u gives ⊥; imp-intro, then forall-intro.

   Each step involves formulas of size about j, so the chain costs about 7a² nodes, not a·b per pair.
2. **Each pair a < b** is one forall-elim of K_a at the numeral b − a − 1. Numerals are unary, so S^{a+1}(b − a − 1) is literally b, and the step proves ¬(a = b).
3. **The conjunction** is built by and-intro.

**Rules:** inst (Q1, Q2), hyp, imp-elim, imp-intro, forall-intro, forall-elim and and-intro.

### 3.2 Bounds

1. One chain with a parameter w: w + j = S^j(w) for j = 0, …, 212, by add_zero, add_succ, cong and trans.
2. For each h, generalize step h (forall-intro), then instantiate at 212 − h. Since S^h(212 − h) is literally 212, this gives (212 − h) + h = 212, and exists-intro closes it.
3. The lower bound: add_zero at h, then exists-intro with witness h.

## 4. Measured size (prototype)

| | Files | Largest | Total nodes | Steps | Disk | Check time (1 process) |
|---|---|---|---|---|---|---|
| per-element distinctness | 44 | 595,420 (h = 210) | 12.7 M | 35,708 | 14.3 MB | about 12 s |
| bounds | 1 | 720,247 | 0.72 M | 951 | 0.7 MB | under 1 s |
| group summaries | 4 | 515,202 | 1.6 M | 21–44 each | 1.6 MB | 13 s including their 44 cited files |
| **stage total** | **49** | **720,247 (72% of the limit)** | **15.0 M** | **about 36,750** | **about 17 MB** | **about 26 s** |

**Results:**
- 45 of 45 base files accepted, and 4 of 4 summaries accepted (`rlt deps` exit 0, 11 of 64 checks, chain 1 of 8).
- **Controls rejected:**
  - a step claiming ¬(2 = 2), which forall-elim refuses;
  - a bounds step with the wrong witness (187 + 24 = 212), refused.

**Cost to the complete run.** About 30–60 s more on a 2-processor machine with RLT-007a's scheduling, including the existing redundant passes: the direct check, the citation check and the library verification.

## 5. Deliverables

- **Generator.** `scripts/gen-rlt007-s1.py` reads H₄₅ from the reviewed transcription only. It refuses a tuple that is not strictly increasing, not 45 long, or not from 0 to 212. It writes:
  - the 45 base files to `tests/rlt007/s1/gen/`, gitignored and pinned by `tests/rlt007/s1/manifest.sha256`, as in RLT-006;
  - the 4 summaries to `tests/rlt007/s1/accept/`, committed;
  - library entries, statement records, the Lean file and the expected-results table.
- **Library.** 49 new ids in area `admissible`, appended to `lib/identities.tsv` (append-only).
- **Statement records:**
  - `H45-DISTINCT-G0` … `G3`, one per group summary;
  - `H45-BOUNDS`.

  Each has semantic status `partial` (obligation (a) only), physical `not-applicable`.
- **Negative tests (`tests/rlt007/s1/reject/` with `expected.tsv`):**
  - a false pair ¬(a = a);
  - a K-chain missing one step;
  - forall-elim at the wrong numeral;
  - a bounds witness off by one;
  - a summary citing a file for another element;
  - a modified, missing or substituted base file;
  - an altered manifest.
- **Generator refusal tests:**
  - a duplicated element;
  - an out-of-order element;
  - 213 in the tuple.
- **Lean.** `lean/RLT007S1.lean`, generated: the 49 RLT goals stated literally over ℕ, each proved by `decide`. A bridge theorem shows they give `H45.Nodup ∧ H45.max − H45.min = 212`, with standard axioms only, under the existing axiom gate.
- **verify.sh.** A new stage through `par_map`:
  - generator reproduction;
  - the manifest;
  - 45 direct checks and 4 `rlt deps` checks;
  - rejects and tamper tests;
  - Lean.

  Every existing stage and redundant pass is kept.
- **Audit and report.**
  - An independent adversarial audit of the statements, generator, tests and correspondence.
  - `docs/RLT-007b-1-REPORT.md` with measurements against the merged main.

## 6. Points to confirm before implementation

1. The statement shapes in §2, including the four-group split forced by the 200,000 numeral limit.
2. The inline reading of "≤" as ∃z. z + x = y.
3. That "0 and 212 are listed" and "45 entries" stay syntactic checks of the transcription, recorded in the correspondence review rather than proved by the kernel.

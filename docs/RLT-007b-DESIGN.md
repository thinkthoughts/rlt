# RLT-007b — Mathematical completion of Lemma 12.1: implementation design

**Status:** design only, 2026-11-12. Implementation waits until RLT-007a has been reviewed and merged.

**Scope:** Robinson's Q (seven axioms) with the current 20 inference rules plus `cite`, and the existing definitions `lt` and `dvd` (`spec/definitions.rlt`). No induction, no new axiom, no new rule, no new definition, no change to the limits.

**Status labels used below:**
- **Measured:** the proofs were written by a scratch generator and accepted by the current `rlt` binary.
- **Estimated:** a size calculation, not yet run.

## 1. The target statement

**Notation.**
- H is the transcribed tuple: 0, 2, 12, …, 212 (45 numerals, in order).
- Formulas are in `rlt` written syntax.
- `x < y` unfolds to ∃z. S(z) + x = y.
- `d ∣ n` unfolds to ∃k. n = d × k.

**Admissibility at p:**

> Adm(p) :≡ ∃r. (r < p ∧ ⋀_{h ∈ H} ¬(∃k. h = (p × k) + r))

**Primality, written inline:**

> Prime(p) :≡ 1 < p ∧ ∀d. ((∃e. p = d × e) → (d = 1 ∨ d = p))

Primality is used only as a hypothesis. It is never proved for a variable, and for a numeral it is only ever refuted.

**Lemma 12.1 in RLT,** proposed id `admissible.h45`:

> L12.1 :≡ Dist(H) ∧ Diam(H) ∧ ∀p. (Prime(p) → Adm(p))

- Dist(H) :≡ ⋀_{i<j} ¬(h_i = h_j): 990 conjuncts.
- Diam(H) :≡ ⋀_{h ∈ H} (∃z. h + z = 212) ∧ ⋀_{h ∈ H} (∃z. 0 + z = h). Every element lies between 0 and 212, and both are listed.

**How the reading matches the source.**
- Over ℕ, with 0 ≤ r < p, ¬∃k. h = p·k + r says "h is not ≡ r (mod p)", so Adm(p) says "H omits a class mod p". This is the reading RLT-006's correspondence review accepted.
- "45" is the length of the listed tuple. It is checked syntactically when the statement is generated from the transcription, and stated as a conjunct count.

**What the semantic record would say.** The proof of the p ≥ 47 part differs from the paper's counting argument. The statement record would mark it "same statement, different proof for (d)", subject to review.

## 2. Component theorems and their proof obligations

Each component is an unconditional RLT theorem under ROBINSON-Q, within all existing limits.

### C1 — distinctness (obligation (a), part 1)

- **Statement.** Dist(H). Split into about 5–6 files by conjunct ranges, each a conjunction of the pairs it covers.
- **Proof.**
  - For each listed a < 212, prove one lemma K_a: ∀u. ¬(a = S^{a+1}(u)). It peels a successors with Q1 on the variable u, then applies Q2.
  - For each pair a < b, forall-elim at the numeral b − a − 1 gives ¬(a = b). Numerals are unary, so S^{a+1}(b − a − 1) is literally b; this was checked: `5 = S(S(3))` is accepted by `refl`.
- **Size (estimated).**
  - Σ a² over the listed a is 704,664, so the lemmas cost about 4.2 million nodes.
  - The 990 instances add about 0.4 million.
- **Obligations.** Q1 and Q2 only.

### C2 — diameter (obligation (a), part 2)

- **Statement.** Diam(H): 90 conjuncts.
- **Proof.** Exhibit the witness z = 212 − h, or z = h. Close each sum with add_zero/add_succ chains, using the same per-file lemma trick: ∀w. w + m = S^m(w).
- **Size (estimated).** Under 1 million nodes; one file.

### C3 — omissions, p ≤ 43 (obligation (b))

**Done in RLT-006.** The 14 library entries `admissible.h45.omits.P_R` each prove ⋀_h ¬(∃k. h = (p × k) + r_p). Unchanged, byte-for-byte.

### C4 — omissions, 47 ≤ p ≤ 211 (obligation (d), finite part)

- **Statement.** Per prime, the same conjunction as C3 with lean-bgp212's class r_p ∈ {1, 3, 4, 6}. 33 new entries `admissible.h45.omits.P_R`.
- **Proof.** RLT-007c's direct scheme. **Measured:** 1,485 certificates, all accepted, at most 324,133 nodes each. Each summary cites its 45 certificates: 45 of 64 checks, chain 1.
- **Obligations.** Q1, Q2, Q3, add_zero, add_succ, mul_zero and mul_succ, at numerals and parameters.

### C5 — omissions, p ≥ 213 (obligation (d), uniform part)

- **Statement:**

  > ∀p. (212 < p → ⋀_{h ∈ H} ¬(∃k. h = (p × k) + 1))

  One summary citing 45 certificates T_h, each proving ∀p. 212 < p → ¬(∃k. h = (p × k) + 1).
- **Proof.** For each T_h:
  1. Q3 on k.
  2. Case k = 0: h = 1 is refuted.
  3. Case k = S(k′): 212 < p unfolds to S(z) + 212 = p, so p = S^{213}(z) and h = S^{214}((p × k′) + z).
  4. Peel h ≤ 212 successors, then apply Q2.
- **Measured (prototype):** 45 of 45 accepted; 69,445 steps; 52 MB; 43 s.
- **Planned improvement.** The per-layer lemma trick of RLT-007c is expected to shrink these files too. Not required: they are within the limits now.

### C6 — the primes below 213 (obligation (c))

- **Statement.** A classification by cases on a variable:

  > ∀p. (Prime(p) → (p = 2 ∨ p = 3 ∨ … ∨ p = 211 ∨ 212 < p))

  The disjunction lists the 47 primes below 213. If it is too large for one file, it can be split by ranges with a chained structure. That split is a C6 design choice and does not change the statement.
- **Proof.**
  1. 213 nested Q3 splits give, at depth m, p = S^m(0) (the numeral m). The last branch gives p = S^{213}(y), and then 212 < p with witness z = y. This is the standard Q fact "∀x. x ≤ n ∨ n < x" for a numeral n, proved by finitely many Q3 steps with no induction.
  2. In branch m:
     - if m is one of the 47 primes, introduce that disjunct;
     - if m = 0 or m = 1, refute 1 < m with add_succ and Q2;
     - if m is composite, take its smallest prime factor e ≤ 13 and d = m / e. Prove m = d × e (e mul_succ steps, each instantiating a per-file lemma ∀w. w + d = S^d(w)). Instantiate Prime's second conjunct at d. Refute d = 1 and d = m with Q1/Q2.
- **Size (estimated).**
  - The case tree costs about 25,000 nodes.
  - The 164 composites cost about 3–7 million nodes, in about 4–8 files. That is 166 non-primes in all, counting 0 and 1.
- **Obligations.** Q1, Q2, Q3, add_zero, add_succ, mul_zero and mul_succ.

### C7 — assembly

- **Statement.** L12.1.
- **Proof from C1–C6:**
  1. C1 and C2 give Dist and Diam.
  2. For ∀p: assume Prime(p); C6 gives the 48-way disjunction.
  3. For a listed prime m, the matching C3 or C4 conjunction, with r = r_m, closes the case. r_m < m holds by a closed computation.
  4. For 212 < p, C5 gives the conjunction with r = 1. 1 < p follows from p = S^{213}(z) with witness S^{211}(z): add_succ chains only, no commutativity.
- Every step is kernel-checked. How C7 *obtains* C1–C6 is the subject of RLT-007d: by citation, by assumption with discharge, or by a larger budget.

## 3. What each component contributes to the soundness claim

| Component | Kernel-checked RLT theorem | Independent cross-check |
|---|---|---|
| C1, C2 | yes | Lean: `RLT006Direct` already proves (a) by `decide`; extend it to state the RLT form |
| C3 | yes (RLT-006) | Lean `om_P` |
| C4 | yes | Lean per-prime by `decide`; lean-bgp212's table |
| C5 | yes | lean-bgp212's `p ≥ 213` lemma (same route) |
| C6 | yes | Lean: `Nat.Prime` decided below 213 |
| C7 | yes, as a proof from C1–C6 | Lean: `lemma_12_1_of_rlt_statements` extended |

## 4. Implementation order (after RLT-007a is merged)

1. C4's generator (the RLT-007c direct scheme) as a reviewed script, with a manifest, rejected cases and tamper tests. This is the largest piece of data: about 51 MB generated.
2. C5: 45 certificates and one summary.
3. C1 and C2.
4. C6.
5. C7, in whichever form RLT-007d's review selects.
6. Statement records, the library area (`admissible.*`), Lean comparisons, the transcription extended with lean-bgp212's classes as a cited second source, and verify.sh stages using RLT-007a's scheduling.

## 5. Risks and open points

- **C6's size is estimated, not measured.** If a file exceeds the node limit, the range split adds files but no new idea.
- **Prime(p) quantifies over all d, including non-standard elements in a model of Q.** That causes no difficulty: the proof only instantiates it at numerals.
- **C4 adds about 4–8 minutes to the complete run.** Turnaround stays acceptable only with RLT-007a's scheduling.

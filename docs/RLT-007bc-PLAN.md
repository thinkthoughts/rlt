# RLT-007b/c — Staged implementation plan

**Status:** plan for approval, 2026-11-12. The mathematical direction of `docs/RLT-007b-DESIGN.md` and `docs/RLT-007c-FINDINGS.md` is approved. This plan sets the stages and their merge boundaries. The components are combined only after all four stages have merged and the Option-B composition architecture (`docs/RLT-007d-OPTION-B.md`) is separately approved.

## Rules for every stage

**Mathematical basis.**
- Robinson's Q, ROBINSON-Q, seven axioms.
- The existing 20 inference rules plus `cite`.
- The existing definitions `lt` and `dvd`.
- The kernel and its limits are unchanged: 1,000,000 nodes per file, 64 cited-file checks, chain 8.
- No induction, new axiom, new rule or new definition.

**Each stage is one pull request, kept in draft until reviewed, merged with a merge commit after approval.** Each delivers all of the following:
1. **Exact theorem statements.** A table of every new library id, its goal as `rlt check` prints it and its fingerprint, plus statement records.
2. **A reproducible generator.** A reviewed script in `scripts/`. Generated files above about 1 MB are gitignored and pinned by a SHA-256 manifest, as in RLT-006. verify.sh checks that the generator reproduces every committed output.
3. **Negative tests.** Rejected proof files, each refused for its stated reason. Tamper tests: a missing or modified generated file, and an altered manifest. A rejection test for a false instance where one can be written.
4. **Lean comparisons.** A generated `lean/RLT007*.lean` stating the RLT goals literally and proving them with standard axioms only, under the same axiom gate.
5. **An independent audit,** adversarial, by an agent that has not seen the work produced. Findings are fixed and re-audited before merge.
6. **A merge report:** `docs/RLT-007x-REPORT.md` with the statements, measurements and audit.

**Verification.**
- New checks join the complete `scripts/verify.sh` run through RLT-007a's `par_map`.
- Every existing check stays.
- The redundant passes recorded in `docs/RLT-007a-REPORT.md` §4 are kept: the direct check, the citation check and the library verification.
- Each stage reports the run's wall time, CPU time and disk use against the previous main.

## Stage 1 — RLT-007b-1: distinctness and diameter (obligation (a))

| | |
|---|---|
| **Statements** | `admissible.h45.distinct.K` for each chunk K: conjunctions of ¬(hᵢ = hⱼ) over a fixed range of the 990 pairs (i < j), plus `admissible.h45.diameter`: ⋀_h (∃z. h + z = 212) ∧ ⋀_h (∃z. 0 + z = h), 90 conjuncts |
| **Method** | One lemma per listed a < 212, K_a: ∀u. ¬(a = S^{a+1}(u)), proved by Q1 peeling on a variable and closed with Q2. Each pair a < b is one forall-elim at b − a − 1. The diameter witnesses are closed with add_succ chains, using a per-file lemma ∀w. w + m = S^m(w) |
| **Estimated size** | about 4.6 million nodes in 5–6 files, all committed (no generated directory needed if under about 1 MB each) |
| **Negative tests** | a pair stated equal; the wrong witness for a diameter conjunct; a file whose K_a peels one step too few; a chunk with a duplicated element in the transcription copy (refused by the generator) |
| **Lean** | `decide` on every conjunct; the literal RLT goals |
| **Merge boundary** | PR 1. No dependency on the other stages |

## Stage 2 — RLT-007b-2: the primes below 213 (obligation (c))

**Superseded in part (2026-11-12).** Stage 2 uses the bounded definition Prime_B(p) :≡ 1 < p ∧ ∀d. ((1 < d ∧ d < p) → ¬(d ∣ p)), approved after the design review (`docs/RLT-007b-2-DESIGN.md` on PR #18) showed that the unguarded form in the table below cannot be proved for any prime under ROBINSON-Q. Stage 2 is split into 2a (non-prime certificates, six per-range exhaustiveness theorems, the classification) and 2b (the 47 positive primality theorems). The table is kept as the original plan.

| | |
|---|---|
| **Statement** | `primes.below_213`: ∀p. (Prime(p) → (R₁(p) ∨ … ∨ R₆(p) ∨ 212 < p)), each Rᵢ(p) the disjunction of p = q over the primes q of range i (ranges as in `docs/RLT-007d-OPTION-B.md` §6.1, so that the layered assembly matches exactly), with Prime written inline: 1 < p ∧ ∀d. ((∃e. p = d × e) → (d = 1 ∨ d = p)), using `lt` |
| **Method** | 213 nested Q3 case splits. A listed prime branch introduces its disjunct. 0 and 1 are refuted from 1 < p. A composite m is refuted by its smallest prime factor e ≤ 13 and d = m/e: m = d × e is computed, Prime's second conjunct is instantiated at d, and d ∉ {1, m} is refuted. The last branch gives 212 < p |
| **Estimated size** | about 3–7 million nodes, so 4–8 files. Range lemmas of the form ∀p. (Prime(p) ∧ a ≤ p … ) are chained so the 213-way split is spread across files within 64 checks. **To be measured first;** if the split needs more than 64 checks, the stage adds one more summary layer (chain ≤ 3) |
| **Negative tests** | a composite listed as prime (no refutation exists: the generator refuses, and a hand-written attempt is rejected); a wrong divisor witness; a missing branch (the final or-cases does not close) |
| **Lean** | `Nat.Prime` decided below 213, together with the RLT form of Prime |
| **Review point** | The correspondence review records that RLT's inline Prime(p) is the reading of "prime" (over ℕ, equivalent to `Nat.Prime`) |
| **Merge boundary** | PR 2. Independent of stages 1, 3 and 4 |

## Stage 3 — RLT-007b-3: the 45 uniform certificates for p ≥ 213 (obligation (d), uniform part)

**Superseded in part (2026-11-12).** Stage 3 (PR #19) proves the summary statement `admissible.h45.omits.tail_1` directly, in one file of 1,750 steps and 495,022 nodes, with no per-element certificates and no citations; see `docs/RLT-007b-3-REPORT.md` on that branch. The planned negative case "a bound of 211 < p" is a true statement (the least true bound is 211), so stage 3 uses 210 < p, which is false (212 = 211 × 1 + 1). The table is kept as the original plan.

| | |
|---|---|
| **Statements** | 45 certificates T_h: ∀p. (212 < p → ¬(∃k. h = (p × k) + 1)); summary `admissible.h45.omits.tail_1`: ∀p. (212 < p → ⋀_h ¬(∃k. h = (p × k) + 1)), citing the 45 (45 of 64 checks, chain 1) |
| **Method** | The measured prototype: Q3 on k; k = 0 gives h = 1, refuted; k = S(k′) gives h = S^{214}(p·k′ + z), peeled and refuted by Q2. The per-variable lemma trick of 007c is applied where it reduces size |
| **Measured** | 45 of 45 accepted; 69,445 steps; 32.7 million nodes; 52 MB; 43 s. Generated and manifest-pinned |
| **Negative tests** | h = 1 (the true instance 1 = p × 0 + 1: no certificate can exist, so a forged one is rejected); a bound of 211 < p for an h that would need 212; a certificate citing the wrong h |
| **Lean** | the uniform statement for each h, proved by `omega` |
| **Merge boundary** | PR 3. Independent |

## Stage 4 — RLT-007c: the 33 primes 47–211 (obligation (d), finite part)

| | |
|---|---|
| **Statements** | 1,485 certificates ¬(∃k. h = (p × k) + r_p); 33 summaries `admissible.h45.omits.P_R` (45 of 64 checks each), the same shape as RLT-006 |
| **Classes r_p** | 1, 3, 4 or 6, as in lean-bgp212. Each is a property computed from the tuple and checked independently, as for RLT-006's profile. lean-bgp212 @ `7898eb4` is cited as a second source, comparison only |
| **Method** | The direct scheme measured in RLT-007c: peel r_p, then case-split k with Q3, using a per-layer lemma over a fresh variable |
| **Measured** | 1,485 of 1,485 accepted; largest 324,133 nodes; 729,571 steps; 51 MB generated; about 430 CPU-seconds |
| **Verification cost** | about 4–8 additional minutes per complete run on a 4-processor runner (direct pass plus summaries) |
| **Negative tests** | an occupied class (generator refuses; a forged file is rejected); a modified, missing or substituted certificate; a summary citing a certificate for another prime |
| **Lean** | per-prime omission by `decide`; agreement with lean-bgp212's table |
| **Merge boundary** | PR 4. RLT-006's 630 certificates are unchanged byte-for-byte, and the direct scheme is not applied to them |

## Stage 5 (after B is approved) — composition

- The assembly theorem Γ ⊢ L12.1 (conditional mode).
- The composition record.
- The `rlt compose` checker, per `docs/RLT-007d-OPTION-B.md`.

This is a separate PR with its own audit.

## Order and dependencies

```
Stage 1 ─┐
Stage 2 ─┤  (independent; reviewable in any order; proposed order as listed)
Stage 3 ─┤
Stage 4 ─┘
          └── Stage 5 (composition) — after stages 1–4 merge and B is approved
```

## Expected outcomes, kept distinct

1. **Every mathematical component is an unconditional RLT theorem under Robinson's Q,** checked by the kernel within today's limits. These are stages 1–4 together with RLT-006.
2. **The complete Lemma 12.1, discharged by composition** from those components (stage 5). It is reported as "kernel-checked conditional theorem, all assumptions discharged by composition", never as an unconditional theorem.

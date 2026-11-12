# RLT-007d — Theorem assembly: architecture review

**Status:** review only, 2026-11-12. No option is implemented. No new axioms, induction or kernel rules in any option.

**Question.** RLT-007b's components C1–C6 are each RLT theorems within the limits. How should RLT obtain Lemma 12.1 (C7) from them, and what exactly may then be claimed?

## 0. The constraint, in numbers

| Resource | Limit | Needed for one root that reaches every component |
|---|---|---|
| Cited-file checks per root (counted transitively) | 64 | about **2,230**: 630 + 1,485 + 45 certificates, 47 + 1 summaries, about 6 + 1 + 8 for C1, C2, C6 |
| Citation chain | 8 | 2 (root → summary → certificate) |
| Term nodes per file | 1,000,000 | about 300 million in total: 67 million for RLT-006, 190 million for C4 and 33 million for C5 (all measured), plus about 10 million estimated for C1, C2 and C6 |

Under the current limits, one root can reach at most 64 × 1,000,000 = 64 million nodes. That is about 4.7× less than the content.

## 1. The three options

### A — A larger reviewed citation budget

Raise `MAX_CITED_CHECKS` from 64 to about 2,500. Leave the node limit, the chain limit and every rule unchanged. C7 then cites C1–C6 directly, and its check re-checks everything once.

| | |
|---|---|
| Kernel acceptance | C7 is accepted as an **unconditional theorem** under ROBINSON-Q, with Γ = ∅, by one `rlt check` |
| Conditional assumptions | none |
| Validated component theorems | every component, re-checked inside C7's check |
| External composition or reporting checks | none beyond today's citation mechanism (SHA-256 pins, goal equality, axiom-set and spec pins). That mechanism is already part of the trusted checker |
| Precise soundness claim | "C7 is a theorem of Robinson's Q", with the same trust base as every RLT theorem since M3c |
| Cost and risks | **A change to a reviewed resource limit.** The limit bounds work, not soundness, but it was set deliberately, and this would be the first change to it. One check takes about 10–15 CPU-minutes, single-threaded, until in-process parallelism exists; that is deferred in RLT-007a. A tighter alternative raises the budget only for a root that declares it (for example `(budget 2500)`), recorded and reviewed per file |

### B — A conditional RLT theorem plus an independently audited composition checker

C7 is checked with the C1–C6 statements as declared assumptions Γ: about 56 formulas. A new reporting-layer check (`rlt deps --discharge`, outside the kernel) confirms that each γ ∈ Γ is, by exact statement fingerprint, the goal of an accepted, citable, unconditional library theorem under the same axiom set and spec pins. The checker gets its own independent audit.

| | |
|---|---|
| Kernel acceptance | C7 is accepted as a **conditional theorem**: Γ ⊢ L12.1. Each component C_i is separately accepted as an unconditional theorem |
| Conditional assumptions | Γ = the component statements, listed in the record |
| Validated component theorems | C1–C6, each by its own `rlt check` or `rlt deps` run within the existing limits |
| External composition or reporting checks | the discharge check: fingerprint match, accepted status, citable, same pins. The final step, "every premise is a theorem, so the conclusion is", is **meta-level modus ponens across files, not a kernel step** |
| Precise soundness claim | "Γ ⊢ L12.1 is kernel-checked; every γ ∈ Γ is a kernel-checked theorem of Q; by soundness of the checked rules (`docs/KERNEL-SOUNDNESS.md`), L12.1 holds in every model of Q in which Γ holds, hence in every model of Q." The argument is sound, but it rests on the composition checker and the soundness argument, not on a single proof object |
| Labelling | The record states **conditional, all assumptions discharged by library theorems (composition check)**. Under your rule, it is not called an unconditional RLT theorem: the discharge is justified *about* the proof system, not *by* a checked proof |
| Cost and risks | No change to the kernel or the limits. The new checker is small and is in the reporting layer. Each component check stays within today's cost |

### C — One fully kernel-checked proof built from component proofs, with no rule change

The aim is a C7 accepted unconditionally under the current limits, with every component either inlined or cited within 64 checks.

| | |
|---|---|
| Kernel acceptance | would be an **unconditional theorem**, like A, with no limit change |
| Conditional assumptions | none |
| Validated component theorems | all, inside one check |
| External composition or reporting checks | none |
| Precise soundness claim | the same as A |
| Feasibility today | **Not achievable with the measured schemes.** One root can reach 64 million nodes; the content is about 300 million. C needs about 4–5× compression across all families, including a re-proof of RLT-006's 630 certificates (kept byte-for-byte today). Sharing lemmas per prime can cut the cost per element. For example, a cancellation lemma ∀v∀w. (S^p(v) = w + p) → v = w is proved once per prime with no h-dependence, and an n ≥ p layer then becomes one instantiation, because numerals are unary. But the n < p closing layer still costs about n·p per element. No construction that fits 64 files has been found or measured |
| What would make C real | a measured prototype of compact per-prime files (all 45 elements of one prime in at most 1,000,000 nodes), plus a file plan with at most 64 transitive checks. That is a research item, without a schedule |

## 2. Comparison

| | A | B | C |
|---|---|---|---|
| Kernel, rules and axioms unchanged | yes | yes | yes |
| Limits unchanged | **no** (citation budget) | yes | yes |
| One proof object for L12.1 | yes | no (proof plus composition) | yes |
| May be called an unconditional RLT theorem | yes | **no**: "discharged by composition" | yes |
| New trusted code | none (one constant) | composition checker, audited | none |
| Feasible now | yes | yes | not demonstrated |
| Check cost | one long check (about 10–15 CPU-min) | components only (parallel) | unknown |

## 3. Recommendation

1. **Implement B first.** It needs no limit change, and the final claim is labelled precisely: every mathematical step is kernel-checked, and only the joining across files is reported. Its record would be conditional with full discharge, not unconditional.
2. **Decide A as a separate, explicit resource decision,** ideally as the per-root declared budget so the global default stays 64. If approved, C7 becomes one unconditional theorem. B's composition checker then serves as an independent second route to the same conclusion.
3. **Keep C as research.** Pursue it only if both A and B are unsatisfactory.

**Decision requested (D-007d):** approve B for implementation in RLT-007b's step 5, and say whether A (global, or per-root declared) should be reviewed.

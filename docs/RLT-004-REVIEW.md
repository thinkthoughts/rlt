# RLT-004 — Broader review: from written mathematics to checkable specifications

**Status.** This is an architecture proposal for review, written on 2026-11-11 against main at cc01968. It is documentation only:
- the kernel, rules, axiom sets, conventions and reading-point records stay unchanged;
- it starts no milestone and makes no release or deployment.

**The question.** How does RLT become a practical theorem prover that turns written mathematical statements into precise, independently checkable specifications?

**The short answer.** RLT already has the trusted core it needs: a 20-rule first-order kernel, pinned axiom sets and definitions, and checked theorem citation. What it needs next sits around that core and leaves the kernel as it is:
- a **statement record** that carries a written claim from its source to a checked goal, and reports each step of the translation;
- a **theorem library** with stable names and visible dependencies;
- a **user manual** that teaches these through the modulo-30 example.

One scaling question, how numbers are represented, is the main constraint on wider applications (§5). It needs a decision before any certificate work begins.

## 1. Where RLT stands

| Layer | What exists (main cc01968) | Trusted? |
|---|---|---|
| Kernel | first-order sequent kernel with 20 rules: refl, symm, trans, cong, inst, hyp, subst, and 13 logical rules; locally nameless terms over 0, S, +, ×; equality up to α | yes |
| Axiom sets | PEANO-ADD, PEANO-ARITH and ROBINSON-Q, each pinned by SHA-256 to a registered source | content pinned; truth assumed |
| Definitions | `lt`, `dvd` and `free235`, expanded before the kernel, each with its source (OLP Q8, Lean core, RLT-authored) | yes, pinned |
| Written goals | the NAT-DEC, NAT-LOGIC and NAT-DEF conventions, which turn `x < 30 → x = 0 ∨ …` into a kernel formula; NAT-DEF reads `¬ ∧ ∨ → ∀ ∃` and no `↔` | yes, pinned |
| Citation | `(cites …)` and `(cite T)`: each cited file is hashed, checked from scratch and re-validated by the kernel; at most 64 checks per top-level check, depth at most 8 | yes, small |
| Records | reading points math0, math5 and math29; the RESIDUE-30 record; points; correspondence records | registered meanings, outside the kernel |
| Comparison | Lean 4 tier 1 (checked goals over ℕ) and tier 2 (correspondence lemmas); the GDT comparison | outside the trusted base |
| CLI | `check`, `written`, `gen`, `lean`, `rp`, `points`, `residues`, `chem`, `validate` | `check` is the verdict path |

**Worked example already in place: Modulo 30.**
- `rlt rp math29` reads "pair through 29 to 30".
- The `written` checks parse 29 + 1 = 30.
- RESIDUE-30 records the eight representatives, quoted from their source.
- M3c–M3e prove the classification theorem E-T1 from Robinson's Q: 54 files, 49 cited-file checks.
- Lean restates E-T1 over ℕ and confirms that free235(r) means gcd(r, 30) = 1 below 30.

## 2. Principles for the next phase

These are carried from the handoff and refined by today's priorities.

1. **A small, auditable kernel.** New capability goes into generators, records and reports, outside the kernel. A kernel change gets its own review and audit.
2. **Translation is reported, not hidden.** Every written word that becomes formal maps to a registered definition, a registered axiom, or an explicit assumption, and the report shows the map.
3. **Three statuses, reported separately:**
   - **proof validity**: the kernel verdict;
   - **source fidelity**: pinned sources and excerpts, and how faithfully the formal statement follows the written one;
   - **correspondence**: whether a mathematical statement describes something measured or physical. That status stays `unestablished` until evidence is registered.
4. **Standard vocabulary.** The terms are theorem, definition, axiom, assumption, specification, proof and citation. The original identifiers math0, math5 and math29 stay as they are; the subject is called Modulo 30.
5. **Lean, described accurately.** RLT and Lean check independently. A Lean proof never makes an RLT theorem, and the comparison reports exactly which statements correspond.
6. **Empirical claims stay separate.** Measurements and physical readings live in correspondence records and never enter a proof.

## 3. Capability 1: Translation

**Goal.** A reader starts from a sentence in a paper and ends with a checked formal statement. Every gap between the two is listed.

**Proposed artifact: the statement record.** It is a registered, source-pinned file that holds one claim through every stage:

```
source excerpt ──▶ written statement ──▶ specification ──▶ checked goal ──▶ verdict
 (pinned, quoted)   (ordinary notation)   (definitions,     (kernel form)    (kernel, with
                                           assumptions,                       cited theorems)
                                           scope)
```

| Field | Content | Example (Modulo 30) |
|---|---|---|
| source | registry id and pinned excerpt | NEWS-MATH29: "1, 7, 11, 13, 17, 19, 23, 29" |
| written | the claim in ordinary notation | "below 30, the numbers with no factor 2, 3 or 5 are 1, 7, …, 29" |
| specification | the NAT-DEF goal, with the definitions it uses | E-T1's goal; `lt`, `dvd`, `free235` |
| assumptions | each explicit assumption, or none | none (Γ = ∅) |
| translation notes | each word mapped to a definition, and each gap stated | "no factor 2, 3 or 5" → free235; "the numbers below 30" → ∀r. r < 30 |
| theorem | path, hash and verdict | tests/m3e/accept/E_T1.rlt |
| correspondence | a separate status | math29 reading: `unestablished` |

**What the report adds.** A new reporting command, working name `rlt explain`, prints the record as three blocks: what was proved, what it rests on, and how the words were translated. It reads existing verdicts and reaches every verdict through `check`.

**Translation gaps the review should settle:**
- **`↔`.** The fix is a written-goal abbreviation for (A → B) ∧ (B → A). It changes a pinned convention, so it needs its own review.
- **Bounded quantifiers.** "for every r below 30" is ∀r. r < 30 → …; it already elaborates and can be named in the manual.
- **Named predicates beyond the three definitions:** prime, even, gcd. They are added as registered definitions with sources, through the same pinned mechanism as `dvd`.
- **New number domains:** integers, rationals, finite sets and graphs. These are in first-order arithmetic only by encoding (§6), and each encoding is a definition with a stated correspondence.

## 4. Capability 2: Proof reuse

**Goal.** Theorems, once checked, are found, cited and combined without re-deriving them, and their dependencies stay visible and bounded.

**Proposed structure.**
- **A library directory**, `lib/` or similar, with stable theorem names and one statement record per theorem. M3c–M3e already supply the first entries: the non-divisibility facts, free235 for each representative, DA01 and E-T1.
- **A dependency report**, working name `rlt deps`, built from `cites` blocks. It prints the graph, the axioms used along each path, and the citation budget each theorem consumes.
- **Certificate generators**, untrusted like `gen-m3c/d/e`, that emit proofs from computed data. The kernel checks every certificate. The generator is evidence of method, never of truth.

**What limits reuse today:**

| Limit | Value | Effect | Option |
|---|---|---|---|
| Citation budget | 64 cited-file checks per top-level check | E-T1 uses 49, so a second classification of the same size cannot sit beside it | (a) flatter graphs; (b) a reviewed cache of verified results keyed by file hash and binary hash, which is a trusted change |
| Citation depth | 8 files | long lemma chains are impossible | keep; design libraries to be wide |
| Re-checking | every citation re-checked from scratch | about 0.7 s for E-T1, growing with the graph | the same cache question as the budget |

## 5. The scaling constraint: numerals

Kernel numerals are unary: 30 is S³⁰(0). This keeps the kernel very small, and it bounds what certificates can reach.
- DA01, the enumeration below 30, uses 21.5% of the 1,000,000-node limit, and the same scheme stops at bound 47.
- A multiplication chain for k = p × q grows with the size of k.

| Application size | Unary numerals |
|---|---|
| numbers below about 50 | feasible with the M3 schemes |
| Goldbach checks up to a few hundred | feasible as separate small certificates; enumeration-style theorems run past the node limit |
| numbers in the thousands or more | out of reach |

**Options, each a decision for review, and each needing its own milestone:**
- **(a) Unary only.** Keep the kernel and limit the scope to small, explicit certificates.
- **(b) Binary numerals as definitions.** Define numerals in binary in the arithmetic, with proved conversion lemmas, and leave the kernel unchanged. Proofs grow logarithmically, but the derivations are larger to write and generate.
- **(c) A kernel numeral primitive.** Add binary numerals with checked arithmetic rules. This is compact and fast, and it is a trusted change requiring a new kernel review and audit.

The recommendation is (b) first, as an experiment outside the kernel, with (c) considered only if (b) proves impractical.

## 6. Candidate applications as tests of generality

These test the architecture; none is a commitment.

| Candidate | What RLT would check | Fits today's design? | Main gap |
|---|---|---|---|
| **Goldbach certificates.** Each even n in a range is p + q with p, q prime. | Per n: the equation n = p + q, plus primality of p and q, where primality is ¬(d ∣ p) for every 1 < d < p, by M3c's scheme | yes, structurally: positive equations plus M3c-style non-divisibility | numeral size (§5); a registered `prime` definition; a library of primality certificates |
| **Rational distances** (explicit constructions, e.g. Kintali-style point sets) | For listed points with integer or rational coordinates: squared distances equal squares of rationals, written as integer equations after clearing denominators | yes, for explicit finite configurations encoded in ℕ arithmetic | integers or rationals as registered encodings; squares and numeral size |
| **Finite graph invariants** (a stated graph has property P) | Existential properties (a colouring, a matching, a path) by witness; universal properties by exhaustive case analysis, as in M3d | existential: yes. Universal: yes in principle, but exhaustive search grows fast | an encoding of graphs (an adjacency relation as a definition or a theory); proof size for exhaustive cases |

**What generalizes across all three:**
- statement records for translation;
- registered definitions with sources;
- certificate generators checked by the kernel;
- library citation;
- a numeral representation that scales.

## 7. Capability 3: The user manual

**Audience.** Readers who can read `1 + 1 = 2` and `x < 30 → …` but may not know the vocabulary of formal logic. Technical terms appear when they mark a distinction the reader needs.

**Proposed outline**, with Modulo 30 as the running example:
1. **Reading points.** math0, math5 and math29, and what a reading is: `rlt rp`.
2. **Written statements.** From "29 + 1 = 30" to a checked equation: `rlt written` and `rlt check`.
3. **Definitions and sources.** `<`, `∣` and free235, each with where it comes from.
4. **Axioms.** Robinson's Q in plain language. What "relative to an axiom set" means.
5. **Proofs.** Steps, rules, assumptions and Γ = ∅, read through one small proof (2 ∣ 8).
6. **Proving that something never happens.** ¬(2 ∣ 7), with the case tree.
7. **Finite enumeration.** Every number below 30 is one of 0 … 29.
8. **Reusing theorems.** Citation, hashes, dependency reports and the budget.
9. **The classification.** E-T1, and how to read its report.
10. **Theorem, source and correspondence.** The three statuses, and why math29's reading stays a reading.
11. **Lean 4.** What the comparison checks, what it does not, and how to run it.
12. **Writing your own.** From a sentence in a paper to a statement record.
13. **Reference.** Commands, file formats, rules, axiom sets and conventions.

## 8. Implementation priorities

The proposed order puts no change in the kernel first.

1. **The user manual.** Chapters 1–11 describe what exists and need no code. It also tests whether the vocabulary is clear.
2. **Statement records and `rlt explain`** for the existing Modulo 30 results. These are reporting only.
3. **The library layout and `rlt deps`**, and the `backed by` links designed in the M3e review. These are reporting only.
4. **Registered definitions** for prime, even and gcd, with sources. These are additions to the definitions file and need a reviewed pin update.
5. **The numerals experiment**, option 5(b), outside the kernel, sized against the Goldbach test.
6. **`↔` in NAT-DEF.** A reviewed convention change, after 1–3 show how readers use it.

## 9. Limitations of this review

- It is based on the merged code and the M3 documents. No new measurement was made for §5's application sizes beyond the recorded M3d and M3e figures, so those sizes are estimates.
- The candidate applications are described at the level of what would be checked. Their exact statements and sources need their own pinning.
- Lean's role and wording are proposed for the manual. They do not change the existing comparison files.

## 10. Decisions for review

| ID | Decision | Options | Recommended |
|---|---|---|---|
| V1 | Kernel during the next phase | (a) unchanged; (b) open to change | (a) |
| V2 | The first milestone | (a) the user manual; (b) statement records; (c) the numerals experiment | (a), then (b) |
| V3 | Translation artifact | (a) a statement record plus a reporting command; (b) extend the existing records | (a): one record per claim, from source to verdict |
| V4 | Library and reuse | (a) `lib/` with stable names plus `rlt deps`; (b) keep tests/ as the library | (a) |
| V5 | Citation scaling | (a) keep 64 and 8, and design wide graphs; (b) review a verified-result cache | (a) for now; revisit (b) with the numerals decision |
| V6 | Numerals | (a) unary only; (b) binary as definitions; (c) a kernel primitive | (b) as an experiment, with (c) only after review |
| V7 | `↔` | (a) defer; (b) add to NAT-DEF now | (a), until the manual shows the need |
| V8 | Candidate applications | (a) keep them as design tests; (b) choose one as a milestone | (a), then choose after V6 |

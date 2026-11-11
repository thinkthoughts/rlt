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

**Refinements, 2026-11-11.** The direction was approved for continued review with three refinements, now recorded here:
1. Numeral scaling separates written notation, kernel term and checked proof size, with measurable success criteria (§5).
2. The written soundness argument is kept apart from implementation evidence (§5a).
3. `rlt explain` reports kernel acceptance separately from source fidelity and semantic correspondence (§3).

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

**What `rlt explain` reports (refinement 3).** It reports four assessments in separate blocks. A kernel verdict never implies the others.

| Block | Question | Who or what decides | Possible statuses |
|---|---|---|---|
| Kernel acceptance | Is the formal statement a theorem relative to the axiom set? | the kernel, through `rlt check` | ACCEPT / REJECT / MALFORMED, with Γ, axioms used and cited theorems |
| Source fidelity | Is the quoted source exactly the pinned text? | hash and excerpt checks | verified / differs / unpinned |
| Semantic correspondence | Does the formal statement say what the written claim says? | a recorded human or independent review, with reviewer, date and listed gaps | unassessed / reviewed: matches / reviewed: partial, with listed gaps / reviewed: differs |
| Physical correspondence | Does the mathematics describe a measured or physical situation? | correspondence records | unestablished / registered evidence |

An accepted theorem alone never establishes that a translation matches the original claim. `rlt explain` prints `semantic correspondence: unassessed` until a review is recorded. Even then it shows that review as a separate, attributed judgement beside the verdict.

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

## 5. The scaling constraint: numerals (refinement 1)

**Three sizes, kept apart.**

| Size | What it measures | Today |
|---|---|---|
| **Written notation** | characters a person writes: `29`, `2 ∣ 8` | already decimal in written goals and proof files, so notation size is not the problem |
| **Kernel term** | nodes in the term the kernel compares | unary: the numeral n is S(S(…S(0)…)), n + 1 nodes, and every claim mentioning n carries them |
| **Checked proof** | steps and total nodes in the derivation the kernel replays | for closed arithmetic, steps grow with the unary structure: `mul_succ` and `add_succ` recurse one successor at a time. Measured: DA01 has 971 steps and 215,390 nodes at bound 30, and the scheme fails beyond bound 47 |

**What binary-as-definitions would and would not reduce.**

- **Kernel term size: yes, if built with a numeral factor.**
  - Definitions such as `B0(t) := 2 × t` and `B1(t) := S(2 × t)` expand to terms with O(log n) nodes. For example, 13 = B1(B0(B1(1))).
  - A doubling definition `t + t` would not help, because expansion copies t and the term returns to O(n) nodes.
- **Checked proof size: no, under ROBINSON-Q alone.**
  - The kernel compares terms syntactically. A binary term and the unary numeral are different terms, so relating them needs a derivation.
  - Q's axioms compute only by peeling successors, so evaluating a binary term in Q costs O(n) steps again.
  - Proof size shrinks only if general lemmas are proved once and then cited, for example ∀x y. 2 × (x + y) = 2 × x + 2 × y. Such lemmas need induction, which ROBINSON-Q lacks.
  - So the experiment needs a **registered axiom set with induction**: PEANO with an induction schema. That is a registry addition with its own source pin and review. It is not a kernel change, but it changes what "relative to an axiom set" means for those theorems.
- **Written notation: no change needed.** Decimal input can elaborate to either representation.

**The experiment, under the constraint that the kernel stays unchanged.**
- **Baseline:** measure unary certificates for a fixed test set: Goldbach decompositions of n = 100, 1,000 and 10,000, with primality of both parts.
- **Variant:** binary definitions plus induction-proved, cited lemmas, under a separately reviewed induction axiom set.

**Success criteria, all required:**
1. `git diff` of `kernel/` against the starting commit is empty.
2. For n = 1,000, the checked proof is at least 10 times smaller in total nodes than the unary baseline, counting every cited lemma once.
3. n = 10,000 checks within the 1,000,000-node limit per file and the 64-check citation budget.
4. Every lemma is accepted by the kernel with Γ = ∅, and its axioms used are exactly the reviewed induction set.
5. The Lean comparison states each certificate over ℕ, and the numeral correspondence (binary term = n) is proved in RLT and in Lean.
6. If criterion 2 fails, the result is recorded and option (c), a kernel numeral primitive, is reviewed only then.

## 5a. Soundness: argument and evidence, kept apart (refinement 2)

Two different things support trust in RLT's verdicts, and RLT reports them separately.

| | The written soundness argument | The implementation evidence |
|---|---|---|
| Question | Are the 20 rules sound? If the premises hold in every model of the axiom set, does the conclusion hold too? | Does the Rust code implement exactly those rules? |
| Form | a mathematical argument per rule: side conditions, eigenvariables, substitution and capture, citation as composition | tests, rejected cases, byte-for-byte regression, independent adversarial audits, CI |
| Where it lives today | spread across the M3a, M3b and M3c reports (Q-f, T1–T2, E1–E4, C1–C5) | `cargo test` (227), `verify.sh`, compare-reports, the audit sections of each M3 report |
| What it cannot show | that the code matches the argument | that the rules are sound: tests sample behaviour and prove nothing |

**Proposal.** One consolidated kernel soundness note, `docs/KERNEL-SOUNDNESS.md`, written as documentation:
- each of the 20 rules and the citation step, with its statement, side conditions and soundness argument relative to the standard semantics of first-order logic;
- the assumptions the argument needs, such as locally nameless binding and α-equivalence as syntactic equality;
- for each rule, a separate list of the implementation evidence and where it lives.

The manual cites the note. Audits check the code against the note. Neither stands in for the other.

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

## 10. Decisions for review (revised after refinements 1–3)

**Approval, 2026-11-11.** V1–V10 approved, with V6 modified as recorded in its row. RLT-004a (§11) approved for implementation; PR #10 stays in draft until its implementation report has been reviewed.

| ID | Decision | Options | Recommended |
|---|---|---|---|
| V1 | Kernel during the next phase | (a) unchanged; (b) open to change | (a), including during the numerals experiment |
| V2 | The first milestone | (a) the user manual with the soundness note; (b) statement records; (c) the numerals experiment | (a), then (b) |
| V3 | Translation artifact | (a) a statement record plus `rlt explain`, with four separately reported assessments; (b) extend the existing records | (a) |
| V4 | Library and reuse | (a) `lib/` with stable names plus `rlt deps`; (b) keep tests/ as the library | (a) |
| V5 | Citation scaling | (a) keep 64 and 8, and design wide graphs; (b) review a verified-result cache | (a) for now |
| V6 | Numerals | (a) unary only; (b) the binary-as-definitions experiment, which needs a separately reviewed induction axiom set and is judged by §5's criteria; (c) a kernel numeral primitive | **Approved as modified (2026-11-11):** measure only. RLT-004a measures written size, kernel term size, proof steps, proof nodes and checking time separately, with existing proof generation and no added axioms, and records any limit reached. (b) and (c), and any induction axioms, go to a separate review. |
| V7 | `↔` | (a) defer; (b) add to NAT-DEF now | (a) |
| V8 | Candidate applications | (a) keep them as design tests; (b) choose one as a milestone | (a), and choose after V6's baseline |
| V9 | Soundness documentation | (a) one consolidated note, with argument and evidence kept apart; (b) leave it in the M3 reports | (a) |
| V10 | Semantic correspondence | (a) recorded, attributed reviews with fixed statuses, defaulting to `unassessed`; (b) leave it implicit | (a) |

## 11. The first milestone: RLT-004a, the manual and the soundness note

**Scope:** documentation and the measurement of a baseline only. It makes no change to code, kernel, rules, axiom sets, conventions or records.

1. **The user manual:** chapters 1–11 of §7, describing what exists, with every command shown run against main and its real output.
2. **The kernel soundness note** (§5a, V9).
3. **The statement-record format**, as a written specification only. It defines the fields, the four assessment blocks and the status vocabulary, and gives the Modulo 30 record as its worked example. No code yet.
4. **The unary baseline (§5, V6 as modified):** written size, kernel term size, steps, proof nodes and check time, measured separately for Goldbach decompositions, non-divisibility certificates and finite enumerations, generated outside `tests/` with existing mechanisms and no added axioms. Limits reached are recorded, not worked around. Result: docs/RLT-004a-BASELINE.md.

**Done when:**
- every command in the manual reproduces its shown output on main;
- the soundness note covers all 20 rules and the citation step;
- the baseline table is recorded;
- an independent reader review finds the manual's chapters 1–5 usable without prior formal-logic vocabulary.


# RLT-007d — Final composition architecture (layered Option B): proposal for approval

**Status:** design for approval, 2026-11-13, reconciled with main at 838954f (stage 4 merged). The composition checker remains design only. This document supersedes `docs/RLT-007d-OPTION-B.md` where they differ; that document stays as the review history.

**Question this design answers.** Do the individually kernel-checked components on main, taken together, establish Lemma 12.1 of Axiom Math's draft ("H₄₅ is an admissible 45-tuple of diameter 212")? The design states exactly what a composition check must verify for the answer to be yes, what it trusts, and how it reports.

**Measured prototype (scratch, not committed).** To ground the design, the eight assembly theorems of §3 were written as conditional RLT proofs and checked by the current binary, unchanged, under the current limits. All eight are kernel accepted as conditional theorems, and a 60-line independent matcher paired every one of their 56 declared assumptions with exactly one component or lower assembly (§11). Figures from that prototype appear throughout.

**Unchanged:** the kernel, the 20 inference rules plus `cite`, the seven axioms of ROBINSON-Q, `spec/definitions.rlt`, and every resource limit (1,000,000 nodes per file, 64 cited-file checks, citation chain 8).

## 1. The components on main

Every component below is an unconditional, citable library theorem, kernel accepted under ROBINSON-Q with convention NAT-DEF.

| Group | Library ids | Count | Stage, merge |
|---|---|---|---|
| Pairwise distinctness, grouped | `admissible.h45.distinct.g0` … `g3` | 4 | stage 1, eb0e2ef |
| Bounds 0 ≤ h ≤ 212 | `admissible.h45.bounds` | 1 | stage 1, eb0e2ef |
| Omitted class, primes 2 … 43 (the paper's classes) | `admissible.h45.omits.P_R` | 14 | RLT-006, e5850d9 |
| Omitted class, primes 47 … 211 (least omitted class) | `admissible.h45.omits.P_R` | 33 | stage 4, 838954f |
| Classification (exhaustiveness direction) | `primes.below_213` | 1 | stage 2a, f37bcfc |
| Omitted class 1 modulo every p > 212 | `admissible.h45.omits.tail_1` | 1 | stage 3, 4e4ca04 |

Supporting library theorems (cited inside components, not composed): `primes.nonprime.i1`–`i6`, `primes.exhaustive.r1`–`r6`, `order.split.o1`–`o6`, and the generated certificates of RLT-006, stage 1 and stage 4.

**Not used:** the 47 positive primality theorems (stage 2b, deferred). Admissibility needs only that every Prime_B number is listed or exceeds 212, which is the exhaustiveness direction `primes.below_213` states. Nothing in this design mentions Prime_B(q) for a listed q.

## 2. The target statements

Write OM(t, r) for the left-nested conjunction ⋀_{h ∈ H₄₅} ¬(∃k. h = (t × k) + r) over the 45 transcribed elements in the listed order, R_i(t) for the right-nested disjunction t = q₁ ∨ (… ∨ t = q_k) over the primes of range i, and

    Adm(t) :≡ ∃r. (r < t ∧ OM(t, r))
    Prime_B(t) :≡ 1 < t ∧ ∀d. ((1 < d ∧ d < t) → ¬(d ∣ t))

Both are written inline with the existing `<` and `∣`; no definition is added.

**The named result `admissible.h45.lemma_12_1`** consists of six statements:

| Statement | Status it will carry |
|---|---|
| `distinct.g0` … `g3` | kernel accepted (layer 0) |
| `bounds` | kernel accepted (layer 0) |
| `adm.all`: ∀p. (Prime_B(p) → Adm(p)) | discharged by composition (layer 2) |

There is one named set of six statements rather than one conjunction, because the four distinctness groups together exceed the 200,000 numeral total of a single formula (measured in stage 1).

## 3. The three-layer dependency graph (point 3)

**Layer 0:** the components of §1 that are discharged into an assembly (47 OM statements, `tail_1`, `primes.below_213`), plus the five top-level components (`g0`–`g3`, `bounds`).

**Layer 1:** seven conditional assembly theorems, each kernel accepted with its declared assumptions Γ.

| Id | Γ (discharged by) | Conclusion | Prototype |
|---|---|---|---|
| `adm.range_1` | OM(q, r_q) for q = 2 … 37 (12 RLT-006 theorems) | ∀p. (R₁(p) → Adm(p)) | 474,289 nodes, 339 steps, 1.0 s |
| `adm.range_2` | q = 41, 43 (RLT-006) and 47 … 73 (stage 4): 9 | ∀p. (R₂(p) → Adm(p)) | 413,299 nodes, 177 steps |
| `adm.range_3` | 8 stage-4 theorems | ∀p. (R₃(p) → Adm(p)) | 413,458 nodes, 128 steps |
| `adm.range_4` | 7 | ∀p. (R₄(p) → Adm(p)) | 407,107 nodes, 112 steps |
| `adm.range_5` | 6 | ∀p. (R₅(p) → Adm(p)) | 385,068 nodes, 99 steps |
| `adm.range_6` | 5 | ∀p. (R₆(p) → Adm(p)) | 349,073 nodes, 92 steps |
| `adm.tail` | `tail_1` | ∀p. (212 < p → Adm(p)) | 264,768 nodes, 655 steps, 0.2 s |

**Layer 2:** one conditional assembly theorem.

| Id | Γ (discharged by) | Conclusion | Prototype |
|---|---|---|---|
| `adm.all` | `primes.below_213` (layer 0); `adm.range_1` … `range_6`, `adm.tail` (layer 1) | ∀p. (Prime_B(p) → Adm(p)) | 213,770 nodes, 45 steps, 0.5 s |

**Edges:** 47 + 1 into layer 1; 1 + 7 into layer 2; 56 in all, each one assumption of one assembly. The largest assembly uses 47% of the node limit, and each assembly is a single file with no citations, so the 64-check budget is never approached.

**Topological verification.**
- Each composition record states its layer. A layer-0 node is a library theorem with Γ = ∅; a node of layer n ≥ 1 is an assembly whose every assumption is discharged by a node of a lower layer, and whose stated layer is 1 + the largest layer among its dischargers.
- The checker builds the graph from the records, runs a topological sort, and refuses a cycle, a self-reference, an edge that does not increase the layer, or a wrong stated layer.
- Records are checked in topological order within one run; a node discharges assumptions only after it has itself passed in that run. One failure fails every node above it, and the report names the first failing node.
- No component's citation closure may contain an assembly file.

## 4. Exact canonical matching (point 1)

An assumption γ of an assembly is discharged by a node u when **the canonical form of γ equals the canonical form of u's statement**, character for character, where:
- the statement of a layer-0 node is its kernel-checked goal (Γ = ∅);
- the statement of a composed node is the conclusion of its assembly, once its record has passed;
- the canonical form is `canonical()` of `cli/src/deps.rs` applied to the elaborated kernel formula, after definition expansion. It writes bound variables as de Bruijn indices, so two statements that differ only by renaming bound variables have the same canonical form (verified: stage 3's statement rewritten over q instead of p has the same fingerprint and canonical goal, while its printed goal differs).

Nothing else counts as a match: no reordering of conjuncts or disjuncts, no logical equivalence, no "a stronger statement discharges a weaker assumption". The `goal` text of a proof file is never compared; only the kernel's checked formulas are.

**Why not the whole fingerprint.** The library fingerprint also hashes the list of definitions a proof *used*, which legitimately differs between an assembly and its components (an OM statement uses no definition; `adm.range_i` uses `<`). The checker compares the canonical goal and, separately, the pinned parts of §5.

## 5. Pinned axiom sets, conventions, definitions and identities (point 2)

At every edge and for every node, these must be identical:

| Part | Requirement |
|---|---|
| Axiom set | ROBINSON-Q: same id, file `spec/robinson-q.rlt`, file SHA-256 |
| Convention | NAT-DEF, with the same pinned interpretation digest |
| Definitions file | `spec/definitions.rlt`, same SHA-256 (the used lists may differ; matching is on expanded formulas) |

**Identity of each layer-0 discharger:**
- a current library entry, `citable: true`, Γ = ∅;
- its registered fingerprint in `lib/identities.tsv` (append-only) equals the fingerprint recomputed in this run;
- its proof file hashes to the entry's pinned proof SHA-256, and `rlt check` accepts it now, re-checking every cited file (certificates included) against their pinned hashes.

**Identity of each assembly:** the composition record pins the assembly file's SHA-256 and its fingerprint; both are recomputed. A composed node gets its own library id with `status: composed` and `citable: false`, so it can never be `cite`d inside a proof; it discharges assumptions only through composition records.

## 6. Ranges, tail and compatibility of the four families (points 4 and 5)

- **Ranges.** The six approved intervals partition 0 … 212; their prime lists are exactly the disjuncts of `primes.below_213`. The assumption ra_i of `adm.all` is written with exactly R_i(p), so `adm.range_i` matches only if it covers exactly range i's primes in the approved order. The prototype confirms it: an `adm.range_2` written without 73 is a true statement and is kernel accepted, but its conclusion no longer matches `adm.all`'s assumption, and the matcher refuses that edge.
- **Tail.** `primes.below_213`'s last disjunct is `212 < p` and `tail_1`'s hypothesis is `212 < p` (both the definition `∃z. S(z) + 212 = p`, matched canonically); `adm.tail` instantiates `tail_1` and supplies r = 1.
- **Small and intermediate primes.** RLT-006's 14 and stage 4's 33 statements have one shape, OM(q, r_q) with numerals, so every range assembly treats them alike; range 2 draws on both families.
- **Exhaustiveness results.** `primes.exhaustive.r1`–`r6` are cited inside `primes.below_213` and are not composed separately; the composition sees one layer-0 node.

## 7. Omitted-class bounds r_p < p (point 6)

Adm(p) requires r < p. The components do not state it (OM(q, r) has no bound), and the generators' checks and Lean are outside the composition claim. So each assembly proves it inside the kernel:
- `adm.range_i`: for each prime q of the range, a closed proof of r_q < q (the witness S(q − r_q − 1) with r_q add_succ steps), conjoined with OM(q, r_q) and then generalized by exists-intro with witness r_q. Prototype: present in all six range files.
- `adm.tail`: 1 < p from 212 < p (S(z) + 212 = p gives S(S²¹¹(z)) + 1 = p by add_succ steps), with witness r = 1. Prototype: steps o1 … L2 of `ADM_TAIL`.

So every class bound is kernel accepted, inside the assemblies.

## 8. Distinctness, cardinality and diameter (point 7)

| Content of "an admissible 45-tuple of diameter 212" | Established by | Label |
|---|---|---|
| The 990 disequalities among the 45 listed numerals | `distinct.g0`–`g3` | kernel accepted |
| Every listed element h satisfies 0 ≤ h ≤ 212 | `bounds` | kernel accepted |
| Admissibility for every prime | `adm.all` | discharged by composition |
| The list has exactly 45 entries; 0 and 212 are listed; the six top-level statements are about exactly that list, in that order | the specification rebuild (§12) against the transcription | source transcription checked |
| "Pairwise distinct, 45 of them" is a 45-tuple; "0 ≤ h ≤ 212 with 0 and 212 listed" is diameter 212; Prime_B over ℕ is the paper's "prime"; ¬(∃k. h = p·k + r) with r < p is "h avoids the class r" | the correspondence reviews of stages 1–4 and of this composition | semantic correspondence assessed |

The cardinality 45 is a property of the statements' text (45 distinct numerals), which the specification rebuild fixes; the kernel shows they are pairwise distinct.

## 9. Refusals (point 8)

Each case is refused with a named reason, and has its own negative test on a scratch repository.

| Case | Refused because |
|---|---|
| Missing: an assumption with no discharge row; a `by:` id absent from the library; a proof or assembly file absent | row count, library lookup, file read |
| Stale: a proof file's hash differs from its pin; a recomputed fingerprint differs from the registered one; a cited certificate differs from its pinned hash; a component or assembly no longer accepted | hash, fingerprint, fresh `rlt check` in this run |
| Conditional: a layer-0 discharger with Γ ≠ ∅ or `citable: false` | the kernel theorem; the entry |
| Duplicated: two rows for one assumption name; two library ids with one fingerprint (already refused by the library) | record and library checks |
| Unused: a declared assumption the assembly does not use (the kernel allows Γ ⊆ declared) | Γ compared with the declared list; refused unless marked `unused: true` with a reason shown in the report |
| Substituted: a different proof under the same id without the library's reviewed replacement (`history`); a different statement under a new id; a true statement of another shape | proof SHA-256 pin; identities are append-only; canonical match (§4) |
| Incompatible: a different axiom set, convention or definitions file anywhere in the graph | §5 |
| Graph: cycle, self-reference, non-increasing layer, wrong stated layer, a composed node that did not pass | §3 |

## 10. Soundness and the trusted boundary (point 9)

**Claim.** If the composition check passes, then for every node v, φ_v holds in every model of ROBINSON-Q.

**Proof, by induction on layers.**
- *Layer 0.* v is an accepted unconditional kernel theorem ∅ ⊢ φ_v. By kernel soundness (`docs/KERNEL-SOUNDNESS.md`), every model of Q satisfies φ_v.
- *Layer n ≥ 1.* v's assembly is an accepted conditional theorem Γ_v ⊢ φ_v, with every γ ∈ Γ_v closed (G5). By kernel soundness, every model M of Q that satisfies all of Γ_v satisfies φ_v. Each γ is discharged by a node u of lower layer that passed; by the induction hypothesis every model of Q satisfies φ_u; by §4–§5, φ_u and γ are the same closed formula, under the same axioms, convention and definitions. So every model of Q satisfies Γ_v, hence φ_v.
- *Well-foundedness.* Layers are finite and strictly increase along edges.

In particular all six statements of the named result hold in ℕ. This is the semantic cut rule across files; inside one file the kernel already performs it with `cite` and imp-elim.

**The trusted boundary.**

| Part | Status |
|---|---|
| Kernel, rules, axioms, limits; parser, checker, `cite`; library loader, `canonical()` and fingerprint | trusted already, unchanged |
| The composition checker (`rlt compose`) | **new trusted code**, limited to: loading records, the graph checks of §3, the per-edge comparisons of §4–§5, and fresh checks through the existing check function. It constructs no formula, applies no inference rule, and has no access to the kernel's `Theorem` constructor (G1) |
| Composition records | untrusted metadata; every field recomputed |
| Specification rebuild, transcription checks | outside the composition claim; reported as source transcription checked |
| Second implementation, Lean cross-check, generators, reports | independent evidence, outside the trust base |

What composition does not give: one kernel proof object for the whole lemma. That would need Option A (a larger citation budget) or a single-file proof, and neither is proposed.

## 11. Independent second implementation and Lean comparison (point 11)

- **Second implementation** (`scripts/compose-check.py`, Python, no shared code): reads only `rlt deps --format json` reports and `lib/` entries; parses the printed formulas and canonicalizes bound variables itself; recomputes the topological order and every edge comparison; requires the same verdict as `rlt compose` on the real bundle and on every negative case. The prototype matcher (60 lines, printed-formula comparison) already pairs all 56 edges on the real assemblies and refuses the weakened `adm.range_2`.
- **Lean comparison** (`lean/RLT007D.lean`): states the six top-level statements and the 49 layer-0 discharged statements literally (generated, checked symbol for symbol as in stages 3–4), and derives in Lean, along the same DAG, that H₄₅ is admissible for every prime in the standard sense (via `pb_iff_std` and `omit_iff`), together with 45 distinct elements and diameter 212. Independent of RLT's checker; standard axioms only, gated by name.

## 12. True but non-matching statements (point 12)

The kernel accepts any true statement it has a proof for, so fidelity to the approved specification is checked separately, at two levels:
- **Between layers:** exact canonical matching (§4) refuses a true assembly of another shape, as the prototype's `adm.range_2` without 73 shows.
- **At the top:** a specification rebuild, outside the trusted checker, regenerates from the transcription the canonical forms of the six top-level statements (and of every OM(q, r_q) and `tail_1`, as stages 3–4 already do) and requires equality.

Negative tests (each kernel accepted, each refused by matching or by the rebuild):
- a range assembly missing a prime, or with the primes reordered;
- `adm.all` concluding a weaker statement (r < p dropped from Adm; Prime_B weakened to 1 < p);
- `adm.tail` with the stronger hypothesis 211 < p;
- an OM component with a conjunct dropped or repeated (stage 4's guard files, reused);
- a top-level statement over a 44-element list.

## 13. Reporting (point 10)

Each result carries exactly one of the four approved labels:

| Line | Label |
|---|---|
| each of the 49 discharged components and the 5 top-level components | kernel accepted: unconditional theorem under ROBINSON-Q |
| each of the 8 assemblies | kernel accepted: conditional theorem Γ ⊢ φ (m assumptions) |
| `adm.range_1`–`6`, `adm.tail` | discharged by composition (layer 1): m of m assumptions discharged by layer-0 components |
| `adm.all` | discharged by composition (layer 2): 8 of 8 assumptions discharged |
| list length 45, endpoints, the statements' element list | source transcription checked |
| the reading of the six statements as Lemma 12.1 | semantic correspondence assessed (docs/RLT-007d-CORRESPONDENCE.md) |

The named result's summary line: "Lemma 12.1 (H₄₅): 5 statements kernel accepted, 1 statement discharged by composition (2 layers, 56 edges); source transcription checked; semantic correspondence assessed." No composed result is ever called an unconditional theorem.

## 14. Implementation decision (part C)

**Smallest implementation that checks the whole bundle**, with no change to the kernel, axioms, rules or limits:

| Piece | Size (estimate) |
|---|---|
| `scripts/gen-rlt007d.py`: generates the 8 assembly files (as in the prototype), the 9 composition records, the Lean file, negative cases | about 400 lines |
| `rlt compose` in `cli/src/compose.rs`: record loading, graph checks, edge comparisons, report | about 400–600 lines of Rust, plus tests |
| `scripts/compose-check.py`: second implementation | about 200 lines |
| `lean/RLT007D.lean`, `docs/RLT-007d-CORRESPONDENCE.md`, statement records, library entries (`status: composed`) | generated / written |
| Tests: one per refusal of §9 and §12, mutation tests of each comparison | about 40 negative cases |

**Committed size:** 8 assembly files, about 1.1 MB in all (measured), plus records; no generated families.

**Runtime and CI cost (estimates from measurements):**
- the 8 assemblies: about 6 s of kernel time in all (measured);
- `rlt compose` re-checks each discharger with its citations once: RLT-006's 14 summaries (about 80 s), stage 4's 33 (about 200 s), the four other roots (under 10 s), so about 5 CPU-minutes, parallel across roots;
- the second implementation reuses those JSON reports: seconds;
- the Lean comparison: about 1–3 minutes;
- negative cases: about 2 minutes.
- In all, about 8–10 CPU-minutes per verify run, so about 4–5 minutes on CI's runner, on top of today's 34–43 minutes; within the 90-minute timeout.

**Option A** (a larger citation budget, giving one kernel root for the whole lemma) stays documented in `docs/RLT-007d-ARCHITECTURE.md` as an alternative; it is not proposed and not authorized.

## 15. Decisions requested

- **D-1.** Approve this architecture and the soundness argument of §10, including the trusted boundary.
- **D-2.** Approve matching by canonical kernel form (§4), equal under renaming of bound variables, with the pinned parts compared separately (§5).
- **D-3.** Approve the named result as six statements (§2) and the reporting of §13.
- **D-4.** Approve implementation of §14 as RLT-007d, on its own branch and draft PR, with the audits of §11 and the tests of §9 and §12.

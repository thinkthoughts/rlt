# RLT-007d — Option B: composition architecture review

**Status:** architecture specification for review, 2026-11-12. Option B is approved for planning; it remains design only, and nothing is implemented. Reporting terms aligned with the four approved labels (§6.8). **Components on main (reconciled 2026-11-13 with main at 4e4ca04):** stage 1 (distinctness and bounds, eb0e2ef), stage 2a (`primes.below_213`, f37bcfc) and stage 3 (`tail_1`, 4e4ca04) are merged; stage 4 (the 33 per-prime theorems for 47 … 211) is on draft PR #20. Stage 2b (positive primality) is deferred; nothing in this design depends on it. **Prime_B (approved 2026-11-12):** every statement here that mentions primes uses the bounded definition Prime_B (§6.1a), consistently with stage 2. **Revised the same day** (§4a–§6) for the formula-size limit; §6 supersedes the depth-one rule of §3.5 where they differ. Nothing is implemented. The composition checker will be built only after this revised architecture is reviewed and approved.

**Unchanged by this design:**
- the kernel and its 20 inference rules plus `cite`;
- the seven axioms of Robinson's Q;
- every resource limit: 1,000,000 nodes per file, 64 cited-file checks per root, citation chain 8.

**Alternative kept on record:** Option A, a larger citation budget, stays documented in `docs/RLT-007d-ARCHITECTURE.md`. The current limit of 64 is unchanged.

## 1. What composition does

Composition joins two kinds of kernel accepted theorem into one claim:

- **An assembly theorem** 𝒜, checked by the kernel in conditional mode: Γ ⊢ φ.
  - φ is the complete statement, here Lemma 12.1 as specified in `docs/RLT-007b-DESIGN.md` §1.
  - Γ = {γ₁, …, γₙ} are its declared assumptions. Each γᵢ is closed: G5 refuses parameters in declared assumptions.
- **Component theorems** C₁, …, Cₙ, each checked by the kernel in unconditional mode: ∅ ⊢ γᵢ, and each registered in the library.

The composition checker establishes, outside the kernel, that every γᵢ is *exactly* the statement of an accepted, citable, unconditional component under the same axioms, conventions and definitions. It then reports the conclusion as **φ, discharged by composition**.

This is the same inference the kernel already performs inside one file when a proof cites a theorem and applies imp-elim to it. Composition performs it across files, so the 64-check budget applies to each component separately instead of to one root.

## 2. The objects

### 2.1 The assembly record (new, reviewed metadata)

The record lives at `lib/compositions/<id>.yaml`, format `rlt-composition/1`:

```yaml
format: rlt-composition/1
id: admissible.h45.lemma_12_1           # its own identity (§5)
assembly:
  proof: tests/rlt007/assembly/L12_1.rlt   # conditional proof file
  sha256: <64 hex>
  fingerprint: <statement fingerprint of Γ ⊢ φ>
discharges:                             # one row per declared assumption, in file order
  - assumption: A_om_2                  # the name declared in the assembly file
    by: admissible.h45.omits.2_1        # a library id
    fingerprint: <that entry's registered fingerprint>
    proof_sha256: <the pinned proof of that entry>
  - …
```

The record is metadata, like a library entry. The checker recomputes every field and compares (§3). Nothing in it is trusted.

### 2.2 What is compared: the statement fingerprint

RLT-004c already defines the statement fingerprint (`cli/src/deps.rs`, `fingerprint_text`). It is the SHA-256 of a canonical text holding:
- the convention and its pinned interpretation digest;
- the axiom set with its file and file SHA-256;
- the definitions file, its SHA-256 and the definitions used;
- the checked goal in canonical form (de Bruijn indices; binder names ignored);
- each assumption, as a set and in canonical form.

Composition uses these parts *individually*. The comparisons are listed in §3.

## 3. The checks (the review's eight points)

### 3.1 Exact matching between assumptions and component statements

For each declared assumption γ of the assembly:
- the assembly's kernel theorem must contain γ in Γ (an unused declaration is reported, §3.5);
- the named component's **checked goal**, canonicalized, must equal γ canonicalized, character for character.

Both formulas are compared **after** definition expansion, as kernel `Prop`s, so notation cannot hide a difference. There is no matching up to provable equivalence, reordering of conjuncts, α-renaming beyond de Bruijn canonicalization, or "weaker implies stronger". Identity is the only relation accepted.

### 3.2 Axiom-set and definition compatibility

For every component and for the assembly, the following must all be identical:

| Part | Requirement |
|---|---|
| Axiom set | identical id, file and file SHA-256 (ROBINSON-Q, `spec/robinson-q.rlt`, pinned hash) |
| Axioms used | every axiom each theorem rests on is, by content, an axiom of that set. The checker already enforces this per file |
| Convention | identical, with an identical interpretation digest (NAT-DEF) |
| Definitions | identical file and file SHA-256 (`spec/definitions.rlt`). The *used* lists may differ: the definitions file fixes each symbol's expansion, and the comparison in §3.1 is on expanded formulas |

A component checked under PEANO-ARITH, a different definitions file or another convention is refused, even if its goal text happens to match.

### 3.3 Preservation of theorem identity and source pins

- Each `by:` id must be a current library entry. Its registered fingerprint in `lib/identities.tsv` (append-only, checked by `scripts/check-identities.sh`) must equal the record's `fingerprint`, and the fingerprint recomputed from this run's check.
- The record's `proof_sha256` must equal the entry's pinned proof, and the file must hash to it.
- A replacement proof is accepted only by the library's existing rule: an identical fingerprint, with the earlier pin kept in `history`. The record is then updated in the same reviewed change.
- The statement records (`statements/*.yaml`) of the components keep their source pins: the registry id, the PDF SHA-256 and the excerpts. The composed record lists each component's record id, so every assumption can be traced to its source.
- The assembly gets its own library id (`admissible.h45.lemma_12_1`). It is registered with its own fingerprint, with `citable: false` and the new `status: composed` (§3.8).

### 3.4 Rejection of missing, stale, conditional or mismatched components

Every case below is a refusal with a named reason, exit code 2 (MALFORMED) or 1 (REJECT), and a negative test of its own:

| Case | Detected by |
|---|---|
| **Missing:** an assumption in Γ with no `discharges` row | row count against Γ |
| **Missing:** a `by:` id absent from the library | library lookup |
| **Missing:** a component's proof file absent | file read |
| **Stale:** the proof file's SHA-256 differs from `proof_sha256` | hash |
| **Stale:** the recomputed fingerprint differs from the registered or recorded one | fingerprint |
| **Stale:** a component, or a file it cites, no longer accepted (`rlt check` exit ≠ 0) | a fresh check in this run |
| **Stale:** the assembly file's hash or fingerprint differs from the record | hash, fingerprint |
| **Conditional:** a component with Γ ≠ ∅, or `citable: false`, or `status: composed` | the kernel theorem; the entry fields |
| **Mismatched:** goal ≠ γ (§3.1); axiom set, convention or definitions differ (§3.2) | part-by-part comparison |
| **Mismatched:** a row naming an assumption the assembly does not declare | name lookup |
| **Mismatched:** the assembly's conclusion differs from the record's statement | fingerprint |

All checks run in one invocation, on files as they are on disk now. No stored result from an earlier run is used.

### 3.5 Dependency cycles, duplicated assumptions and substituted proofs

- **Cycles cannot arise, by construction.**
  - A component must be unconditional and citable. A composed theorem is `citable: false` with `status: composed`, so it can never discharge an assumption: composition has depth one.
  - Inside each component, citations are acyclic, enforced by the checker's chain limit (8).
  - The checker also verifies that no component's citation closure contains the assembly file.
- **Duplicated assumptions.**
  - Two declared names with the same canonical formula are allowed; the kernel's Γ is a set of formulas, and the fingerprint already treats it so. They may name the same component.
  - Two rows for one name are refused.
- **Unused assumptions.** A declared assumption that the assembly's Γ does not contain is reported. It is refused unless the record marks it `unused: true`, so that the report lists exactly the statements φ depends on.
- **Substituted proofs.**
  - A component is pinned by proof SHA-256 *and* by fingerprint. Swapping in a different proof of the same statement requires the library's reviewed replacement procedure, and is visible in `history`.
  - A different statement under the same id is impossible: identities are append-only.
  - A proof of a different statement under a new id fails §3.1.

### 3.6 The mathematical soundness argument

Write ⊨_Q φ for "φ holds in every structure satisfying the axioms of Robinson's Q".

1. **The assembly is valid.** By the kernel soundness theorem (`docs/KERNEL-SOUNDNESS.md`, with G5 for declared assumptions), the accepted conditional theorem Γ ⊢_Q φ is valid. For every M ⊨ Q and every assignment σ: if M, σ ⊨ γ for each γ ∈ Γ, then M, σ ⊨ φ. φ and every γ are closed, so σ is irrelevant.
2. **Each component is valid.** For each γᵢ ∈ Γ, the component Cᵢ is an accepted unconditional theorem ∅ ⊢_Q γᵢ′. By the same theorem, M ⊨ γᵢ′ for every M ⊨ Q.
3. **Exact identity.** By §3.1, γᵢ′ and γᵢ are the same kernel formula. By §3.2, they are interpreted under the same axioms and conventions, with the same definition expansion. So M ⊨ γᵢ for every M ⊨ Q.
4. **Conclusion.** For every M ⊨ Q, M satisfies all of Γ (by 3), hence M ⊨ φ (by 1). So ⊨_Q φ, and in particular φ holds in ℕ.

This is the semantic form of the cut rule. Inside one file, the kernel's `cite` followed by imp-elim (or by discharging a hypothesis) proves the same thing; citation is already part of the trusted checker.

**The argument rests on:**
- the kernel soundness argument (paper argument, plus the evidence of RLT-004a);
- the correctness of the composition checker's comparisons (§3.7).

**It does not rest on:**
- any new axiom, inference rule or induction;
- any change to resource limits.

**What it does not give:** one proof object. There is no single kernel derivation of ∅ ⊢ φ, and Option A or C would be needed for that.

### 3.7 Trusted boundary and audit requirements

| Component | Trust status |
|---|---|
| kernel (`kernel/`), parser and checker (`syntax/`, `cli/src/lib.rs` check path) | trusted, unchanged |
| library loader and fingerprint (`cli/src/deps.rs`) | trusted already for identity; unchanged |
| **composition checker** (new; for example `cli/src/compose.rs`, command `rlt compose ID`) | **new trusted code**, kept minimal: it calls the existing check and fingerprint functions and compares their results. It builds no formulas, applies no rules and has no `Theorem` constructor (G1 keeps `Theorem` private to the kernel) |
| the record `lib/compositions/*.yaml` | untrusted metadata; every field is recomputed |
| generators, Lean comparisons, reports | outside the trust base |

**Audit requirements before the checker merges:**
1. **An independent adversarial audit**, by an agent that has not seen the implementation produced, against this specification.
2. **One negative test per row of §3.4 and per case of §3.5**, each on a scratch repository, each refused for its named reason.
3. **Mutation testing of the comparisons:**
   - flip each of the five equality checks to always-true, and confirm that at least one negative test fails;
   - for each part of §3.2, a component that differs only in that part.
4. **An independent second implementation of the comparison:** a short Python script reading `rlt deps --format json` and `rlt check` reports, with no shared code. verify.sh requires it to agree with `rlt compose` on every record and every negative case.
5. **A Lean cross-check:** extend `lemma_12_1_of_rlt_statements` so that Lean derives the paper's Lemma 12.1 from the literal component statements. This is an independent check of the assembly step's *mathematics*, outside RLT's trust base.
6. **The soundness argument (§3.6)** added to `docs/KERNEL-SOUNDNESS.md` as a section on composition, reviewed with the code.

### 3.8 Reporting "kernel accepted" versus "discharged by composition"

**Three kinds of claim, never merged:**

| Report | Applies to | Wording |
|---|---|---|
| `rlt check` | each component | "ACCEPT — unconditional theorem relative to registered axiom set ROBINSON-Q" (unchanged) |
| `rlt check` | the assembly | "ACCEPT — conditional theorem: Γ ⊢ φ" with Γ listed (unchanged) |
| `rlt compose` | the composed claim | "discharged by composition: φ holds in every model of ROBINSON-Q; the conditional theorem ⟨assembly⟩ is kernel accepted, and each of its n assumptions is discharged by a kernel accepted unconditional theorem (composition check rlt-composition/1)". Exit 0 only if every check holds. The labels are those of §6.8 |

**Records and listings:**
- **Library.** The assembly's entry has `status: composed` and `citable: false`. `rlt deps --verify-lib` lists it separately from kernel accepted entries.
- **Statement record.** `kernel.mode: conditional` with Γ, plus a new `composition:` block naming the record and its result. `rlt explain` prints both lines, "kernel accepted: conditional" and "discharged by composition: n of n", and never prints "unconditional theorem" for the assembly.
- **Reports and README.** They state the trust boundary in one sentence: every mathematical step is kernel accepted; joining the files is a composition check outside the kernel, justified by §3.6.

## 4. Costs

- Composition re-checks every component once: about 2,230 kernel file checks for Lemma 12.1. These are separate roots, so they parallelize with RLT-007a's scheduling. Estimated 6–10 minutes of CPU time on top of the stages, which already check the same files.
- The record is small. The checker is estimated at a few hundred lines plus tests.

## 4a. The formula-size limit (measured constraint)

Two existing limits bound what one file can state. Neither is changed.

| Limit | Value | Where |
|---|---|---|
| Numeral total in one written formula | 200,000 | `MAX_NUMERAL_TOTAL`, `syntax/src/elab.rs` |
| Term nodes in one file, counted over the whole proof text | 1,000,000 | `MAX_FILE_NODES`, `syntax/src/proof.rs` |

**Measured and estimated sizes of the statements involved.**

| Statement | Size |
|---|---|
| Distinctness of all 990 pairs as one formula | 221,056 in numeral total: **above the formula limit** (measured, stage-1 prototype). Stage 1 therefore states it as four grouped theorems, at most 71,676 each |
| One per-prime omission statement OM_p (45 conjuncts) | 6,000–15,500 nodes; 482,000 for all 47 primes together (estimated from the statement shape) |
| Adm(p) with p a variable: ∃r. (r < p ∧ ⋀_h ¬∃k. h = (p × k) + r) | about 5,900 nodes |
| Assembling Adm over all 47 primes in one file | every prime branch restates its OM statement in about 6 steps, about 2.9 million nodes: **above the file limit** |

So no single assembly file can work. The options differ in how the work is divided.

## 5. Layered assembly compared with a compact final statement

| | Layered assembly | Compact final statement |
|---|---|---|
| Idea | Several conditional theorems, each within the limits. Each layer's assumptions are discharged by components or by lower layers | One top theorem whose statement is small enough, assumptions discharged in one step |
| Can it carry the admissibility part (∀p. Prime_B(p) → Adm(p))? | **Yes.** Six range assemblies (about 450,000–510,000 nodes each, estimated), then one small top assembly | **No.** The per-prime case split alone needs about 2.9 million nodes, so this part needs layers whatever the statement |
| Can it carry distinctness? | Yes, as four group statements | Only by replacing pairwise distinctness with a smaller equivalent. For example "strictly increasing", ⋀ hᵢ < hᵢ₊₁, about 9,400 in numeral total. That equivalence over ℕ would be a correspondence reading, semantic correspondence assessed, not kernel accepted |
| Statement fidelity | Each statement stays in the source's own terms | The final statement is a reformulation, and needs its own correspondence review |
| New trusted logic | Composition across more than one layer (§6) | Depth one would suffice for the top, but the admissibility layers are needed regardless |

**Selected: layered assembly,** for every part of the lemma. The compact form cannot avoid layering for admissibility, and it would replace the approved pairwise-distinctness statements with a reformulation.

**The top-level claim.** Lemma 12.1 is reported as **one named result made of six top-level statements**, each established exactly as stated:
- distinctness groups g0–g3;
- the bounds theorem;
- Adm-all.

There is no single conjunction formula, because the four distinctness groups together exceed the formula limit. The record lists the six statements and their status. The source-correspondence review records that, together with the transcription checks (45 listed elements, both endpoints 0 and 212 listed), they are exactly the content of "an admissible 45-tuple of diameter 212".

## 6. The layered architecture

### 6.1 The dependency graph (exact)

Layer 0 holds the components. Every node is an unconditional, citable library theorem checked by the kernel.

| Id | Statement | Stage |
|---|---|---|
| `admissible.h45.distinct.g0` … `g3` | grouped pairwise distinctness | 1 |
| `admissible.h45.bounds` | ⋀_h (∃z. z + 0 = h) ∧ (∃z. z + h = 212) | 1 |
| `primes.below_213` | ∀p. (Prime_B(p) → (R₁(p) ∨ (R₂(p) ∨ … (R₆(p) ∨ 212 < p)))), where each Rᵢ(p) is the right-nested disjunction p = q₁ ∨ (… ∨ p = qₖ) over the primes of range i | 2a (merged, f37bcfc) |
| `admissible.h45.omits.tail_1` | ∀p. (212 < p → ⋀_h ¬(∃k. h = (p × k) + 1)), left-nested over H₄₅ in the listed order | 3 (merged, 4e4ca04) |
| `admissible.h45.omits.P_R` (47 of them) | ⋀_h ¬(∃k. h = (P × k) + R): 14 from RLT-006 (merged), 33 from stage 4 (PR #20; classes: the least class H₄₅ omits) | RLT-006, 4 |

**The six prime ranges** (approved 2026-11-12, with their intervals; together the intervals partition 0 … 212):

| Range | Interval | Primes | Count |
|---|---|---|---|
| R₁ | [0, 40] | 2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37 | 12 |
| R₂ | [41, 78] | 41, 43, 47, 53, 59, 61, 67, 71, 73 | 9 |
| R₃ | [79, 112] | 79, 83, 89, 97, 101, 103, 107, 109 | 8 |
| R₄ | [113, 156] | 113, 127, 131, 137, 139, 149, 151 | 7 |
| R₅ | [157, 190] | 157, 163, 167, 173, 179, 181 | 6 |
| R₆ | [191, 212] | 191, 193, 197, 199, 211 | 5 |

The disjunction in `primes.below_213` is grouped by these ranges, so that each range assembly matches one disjunct exactly. This decision is fixed in stage 2's statement.

### 6.1a The definition of prime

    Prime_B(p) :≡ 1 < p ∧ ∀d. ((1 < d ∧ d < p) → ¬(d ∣ p))

It is written inline with the existing definitions (`x < y` is ∃z. S(z) + x = y; `d ∣ n` is ∃k. n = d × k), so composition adds no definition.
- Its correspondence with the standard notion of prime is a Lean theorem over ℕ (`pb_iff_std` in `lean/RLT007S2a.lean`), reported as **semantic correspondence assessed**, not as an RLT theorem under Robinson's Q.
- The guarded form Prime_G is a documented reference definition only. The unguarded form is set aside, because no RLT proof under ROBINSON-Q can establish it for any prime (the countermodel in `lean/RLT007S2Design.lean`).
- Composition matches statements by fingerprint, so every node that mentions primes (`primes.below_213`, `adm.all`) uses Prime_B literally, with the same bound variable names.

**What `primes.below_213` rests on.** Stage 2a proves it from six exhaustiveness theorems `primes.exhaustive.r1` … `r6` and six order splits `order.split.o1` … `o6`, by `cite`. It is an ordinary unconditional library theorem, so composition treats it as one layer-0 node. The 47 positive primality theorems of stage 2b (Prime_B(q) for each listed q) are a separate result. `adm.all` does not need them: admissibility for every prime needs only that every prime is listed, which is the exhaustiveness direction RLT has established. The converse has Lean evidence only (`listed_1` … `listed_6`).

**What `adm.tail` adds to `tail_1`.** `tail_1` quantifies over every p > 212, prime or not. `adm.tail` instantiates it at p and supplies the witness r = 1 with 1 < p (from 212 < p by a closed computation), giving Adm(p). The primes enter only through `primes.below_213` in `adm.all`.

**Layer 1:** conditional assembly theorems, each checked by the kernel.

| Id | Assumptions Γ | Conclusion |
|---|---|---|
| `admissible.h45.adm.range_i` (i = 1…6) | the OM_q statements for the primes q of range i | ∀p. (Rᵢ(p) → Adm(p)) |
| `admissible.h45.adm.tail` | `tail_1` | ∀p. (212 < p → Adm(p)) |

**Layer 2:** one conditional assembly theorem.

| Id | Assumptions Γ | Conclusion |
|---|---|---|
| `admissible.h45.adm.all` | `primes.below_213`, `adm.range_1` … `adm.range_6`, `adm.tail` (8 assumptions, each a few thousand nodes) | ∀p. (Prime_B(p) → Adm(p)) |

**The named result `admissible.h45.lemma_12_1`** is a composition record, not a formula. It lists the six top-level statements:
- `distinct.g0` … `g3` (layer 0, kernel accepted; merged in RLT-007b stage 1 at eb0e2ef);
- `bounds` (layer 0, kernel accepted; merged likewise);
- `adm.all` (layer 2, discharged by composition).

**Edge count.**
- 47 + 1 = 48 edges from layer 0 into layer 1.
- 1 + 7 = 8 edges into layer 2.
- 6 entries in the named result.

### 6.2 Acyclicity and topological verification

- **Layers are explicit.** Each composition record states its layer number. A component (unconditional library theorem) is layer 0. A composed result of layer n may discharge assumptions only of a record of layer greater than n. Edges therefore strictly increase the layer, and no cycle is possible.
- **The checker also verifies acyclicity directly.** It builds the graph from the records and runs a topological sort (Kahn's algorithm). It refuses:
  - a cycle;
  - a self-reference;
  - an edge that does not increase the layer;
  - a record whose stated layer differs from 1 + the maximum layer of its dischargers.
- **Verification runs in topological order.** A record is checked only after every record it depends on has passed in the same run. One failure fails every record above it, and the report names the first failing node.
- **Cited-file closures are checked against the assemblies.** No component's citation closure may contain a composition record's assembly file.

### 6.3 Matching at each layer

The rules of §3.1–§3.5 apply unchanged at every edge.
- **Exact matching.** The discharger's statement must equal the assumption *exactly*, as expanded kernel formulas in canonical form.
  - **Bound-variable names (stage-3 audit, 2026-11-12).** The kernel's own goal check compares formulas up to the names of bound variables: a copy of stage 3's TAIL.rlt with its goal written over q instead of p is accepted. The composition checker therefore compares the canonical printed statements of discharger and assumption (and their fingerprints), never the `goal` text of a proof file, so that "the same bound variable names" (§6.1a) is enforced by the checker itself.
- **What the discharger's statement is.**
  - For a layer-0 discharger, it is the kernel accepted goal (Γ = ∅).
  - For a composed discharger of layer n ≥ 1, it is the **conclusion** of its assembly theorem, and only once that record has itself passed in this run, with every assumption discharged. A composed result whose own record fails, or has an undischarged assumption, discharges nothing.
- **Duplicates, unused assumptions and substitution** are refused as in §3.5. An unused declared assumption is refused unless the record marks it `unused: true` with a written justification, which is shown in the report.

### 6.4 Compatibility

At every edge, the following must be identical to those of the assembly being discharged:
- the axiom set: id, file and file SHA-256;
- the convention and its interpretation digest;
- the definitions file and its SHA-256.

The check is §3.2, applied per edge. Mixed axiom sets anywhere in the graph are refused.

### 6.5 Stale, conditional and missing proofs

Everything is re-checked in the same run, in topological order:
- every layer-0 proof file;
- every assembly file.

There are no stored results and no cache.

**Refusals:**
- **Stale:** a hash, fingerprint or registered-identity mismatch, or a failed re-check.
- **Conditional:** a layer-0 node whose kernel theorem has Γ ≠ ∅.
- **Missing:** a record, entry or file that does not exist.
- **Non-discharging:** a composed node whose own record did not pass.

Each refusal has a negative test.

### 6.6 Soundness, by induction on layers

**Claim.** For every node v that passes, its statement φ_v holds in every model of Q:

> ⊨_Q φ_v

**Base case (layer 0).** v is an accepted unconditional kernel theorem ∅ ⊢_Q φ_v. By kernel soundness (`docs/KERNEL-SOUNDNESS.md`), ⊨_Q φ_v.

**Step (layer n ≥ 1).** v's assembly is an accepted conditional theorem Γ_v ⊢_Q φ_v. Kernel soundness and G5 make it valid, so for every model M of Q:

> M ⊨ Γ_v implies M ⊨ φ_v

Each γ ∈ Γ_v is discharged by a node u of layer less than n that passed. By the induction hypothesis, ⊨_Q φ_u. By exact matching and compatibility (§6.3–§6.4), φ_u and γ are the same closed formula, read under the same axioms, convention and definitions. So every model M of Q satisfies all of Γ_v, and therefore satisfies φ_v. Hence ⊨_Q φ_v.

**Well-foundedness.** The induction is over layer numbers, which are finite and strictly increase along edges (§6.2). The finite graph is acyclic, so every node is reached.

**The named result.** Each of its six statements holds in every model of Q, and in particular in ℕ. The transcription checks, and the correspondence review's reading of those six statements as Lemma 12.1, are stated separately (§6.8). They are not part of this soundness claim.

### 6.7 Trusted boundary

| Component | Trust status |
|---|---|
| kernel, rules, axioms, limits | trusted, unchanged |
| parser, checker, `cite`, library loader and fingerprint | trusted already, unchanged |
| **composition checker** (`rlt compose`, outside the kernel) | **new trusted code**, limited to: loading records, the layer and topological checks (§6.2), the per-edge comparisons (§6.3–§6.4) and re-checks through the existing check function. It constructs no formula and no `Theorem` |
| composition records | untrusted metadata, every field recomputed |
| transcription checks (45 elements, endpoints listed, increasing order) | outside the composition claim. They are performed by the generators and verify.sh against the pinned PDF transcription, and reported separately |
| generators, Lean files, reports | outside the trust base |

**Audit requirements** are those of §3.7, plus:
- negative tests for each refusal in §6.2 and §6.5;
- a second implementation, independent of the first, that recomputes the topological order and every edge comparison from `rlt deps --format json`;
- a Lean cross-check that derives the paper's Lemma 12.1 from the literal layer-0 statements, mirroring the DAG.

### 6.8 Reporting terminology (approved 2026-11-12)

Every reported result carries exactly one of four status labels. No other label is used for a mathematical claim.

| Label | Applies to | Meaning |
|---|---|---|
| **kernel accepted** | every layer-0 component; every assembly theorem (as a conditional theorem Γ ⊢ φ) | `rlt check` accepted the proof file under ROBINSON-Q: an unconditional theorem (Γ = ∅) or a conditional one with Γ listed |
| **discharged by composition** | each passing composed node of layer n ≥ 1 | its assembly theorem is kernel accepted, and each of its m assumptions is exactly the statement of a passing node of a lower layer (§6.2–§6.6). So φ holds in every model of ROBINSON-Q, by the argument of §6.6; there is no single proof object for φ |
| **source transcription checked** | listing facts taken from the paper: 45 listed elements, strictly increasing, endpoints 0 and 212; the transcribed lemma text | compared with the pinned PDF transcription by the generators and verify.sh (`check_excerpts.py` ties the transcription to the PDF); not kernel-checked |
| **semantic correspondence assessed** | the reading of the RLT statements as the paper's claims | a reviewed correspondence document, pinned by SHA-256 in each statement record |

**Report lines:**
- *Layer-0 component:* "kernel accepted: unconditional theorem under ROBINSON-Q".
- *Assembly theorem:* "kernel accepted: conditional theorem Γ ⊢ φ (m assumptions)".
- *Composed node:* "discharged by composition (layer n): m of m assumptions discharged by passing lower-layer nodes".
- *Named result:* "Lemma 12.1 (H₄₅): 5 statements kernel accepted, 1 statement discharged by composition; source transcription checked: 45 listed elements, strictly increasing, endpoints 0 and 212; semantic correspondence assessed: docs/…".

Nothing composed is ever labelled "unconditional theorem". `citable` remains false for composed nodes. They discharge assumptions only through composition records, never through `cite`.

## 7. Decisions requested

- **B-D1′.** Approve the layered architecture (§6) for implementation after stages 1–4 have merged.
- **B-D2′.** Approve the reporting vocabulary (§6.8).
- **B-D3′.** The six prime ranges and their intervals were approved on 2026-11-12 (§6.1). Stage 2a measured them within the limits, so no re-balancing is planned.
- **B-D4′.** Confirm that the top-level claim is a named set of six statements, not one conjunction.

Option A (a larger citation budget) remains a deferred alternative. The limit is unchanged.

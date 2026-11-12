# RLT-007d — Option B: composition architecture review

**Status:** architecture specification for review, 2026-11-12. Nothing is implemented. The composition checker will be built only after this document is separately approved.

**Unchanged by this design:**
- the kernel and its 20 inference rules plus `cite`;
- the seven axioms of Robinson's Q;
- every resource limit: 1,000,000 nodes per file, 64 cited-file checks per root, citation chain 8.

**Alternative kept on record:** Option A, a larger citation budget, stays documented in `docs/RLT-007d-ARCHITECTURE.md`. The current limit of 64 is unchanged.

## 1. What composition does

Composition joins two kinds of kernel-checked theorem into one claim:

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

### 3.8 Reporting "kernel-checked" versus "discharged by composition"

**Three kinds of claim, never merged:**

| Report | Applies to | Wording |
|---|---|---|
| `rlt check` | each component | "ACCEPT — unconditional theorem relative to registered axiom set ROBINSON-Q" (unchanged) |
| `rlt check` | the assembly | "ACCEPT — conditional theorem: Γ ⊢ φ" with Γ listed (unchanged) |
| `rlt compose` | the composed claim | "COMPOSED — φ holds in every model of ROBINSON-Q: the conditional theorem ⟨assembly⟩ is kernel-checked, and each of its n assumptions is discharged by a kernel-checked unconditional theorem (composition check rlt-composition/1)". Exit 0 only if every check holds |

**Records and listings:**
- **Library.** The assembly's entry has `status: composed` and `citable: false`. `rlt deps --verify-lib` lists it separately from kernel-checked entries.
- **Statement record.** `kernel.mode: conditional` with Γ, plus a new `composition:` block naming the record and its result. `rlt explain` prints both lines, "kernel-checked: conditional" and "discharged by composition: n of n", and never prints "unconditional theorem" for the assembly.
- **Reports and README.** They state the trust boundary in one sentence: every mathematical step is kernel-checked; joining the files is a composition check outside the kernel, justified by §3.6.

## 4. Costs

- Composition re-checks every component once: about 2,230 kernel file checks for Lemma 12.1. These are separate roots, so they parallelize with RLT-007a's scheduling. Estimated 6–10 minutes of CPU time on top of the stages, which already check the same files.
- The record is small. The checker is estimated at a few hundred lines plus tests.

## 4a. A measured constraint the implementation review must resolve

Stage 1 prototyping found that the existing parser limits each written formula to a numeral total of 200,000 (`MAX_NUMERAL_TOTAL`, `syntax/src/elab.rs`). Every file is also limited to 1,000,000 term nodes.

**What it affects.** The assembly file must state its conclusion and every declared assumption in full. Estimates:
- distinctness alone, as one formula, is 221,056 in numeral total, which is why stage 1 splits it into four groups;
- the 47 per-prime omission statements together come to roughly 0.5 million nodes;
- the conclusion Lemma 12.1 restates all of them.

So **a single assembly file is likely to exceed the existing limits.** It must be measured before the detailed implementation review.

**Options within this architecture, with no limit changes:**
- **Layered assembly.** Several conditional assemblies, each discharged by composition. For example, Adm(p) for each range of primes, then L12.1 from those. This needs composition depth greater than one, with the same cycle rules applied level by level: a composed result could discharge only assumptions of a higher layer, never its own.
- **A more compact final statement.** L12.1 as a conjunction of named sub-results whose statements stay within the limits.

The detailed implementation review will choose between them with measurements. Both keep every rule in §3.

## 5. Decisions requested

- **B-D1.** Approve this specification, for implementation after the component stages (RLT-007b/c) have merged.
- **B-D2.** Confirm the reporting vocabulary in §3.8, especially "COMPOSED" and "discharged by composition".
- **B-D3.** Confirm that unused declared assumptions are refused unless marked (§3.5).

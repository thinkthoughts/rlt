# RLT kernel soundness

**Status.** RLT-004a deliverable, written 2026-11-11 against main at cc01968. The kernel, its rules, registered axioms, definitions, conventions and records are unchanged by this document.

**What this document claims.** Every `Theorem` value the kernel produces is *semantically valid* relative to the axioms it records: in every structure satisfying those axioms, under every assignment of values to parameters, if all of its assumptions hold, its conclusion holds. An ACCEPT verdict for an unconditional proof (Γ = ∅) therefore means the checked formula holds in every model of the listed axioms.

**How to read it.** Each rule has five parts, kept apart on purpose:

1. **Conditions**: what the kernel requires before it produces a conclusion.
2. **Argument**: the mathematical reason the conclusion is valid when the premises are. This is a paper argument.
3. **Implementation**: where the Rust code enforces the conditions.
4. **Evidence**: tests, audits and comparisons that check the implementation against the argument. Evidence supports the claim that the code matches the argument; it is a separate thing from the argument itself.
5. **Assumptions and limitations**: what the argument relies on and what it leaves open.

The argument establishes soundness of the *rules as specified*. The evidence establishes, to the extent tests and review can, that the Rust code *implements* those rules. Neither is a machine-checked proof; §6 says what that leaves open.

Line numbers refer to `kernel/src/rules.rs` unless another file is named, at commit cc01968.

---

## 1. The logic

**Terms.** Built from `0`, `S` (arity 1), `+` and `×` (arity 2), parameters (`Param a`, free variables of a derivation), and bound indices (`Bound i`, locally nameless de Bruijn indices). Axiom bodies alone may contain axiom variables (`Var x`). Numerals are unary: the written `7` becomes S(S(S(S(S(S(S(0))))))) (`term.rs`, `Term::numeral`).

**Formulas** (`prop.rs:24`): `t = u`, ⊥, φ ∧ ψ, φ ∨ ψ, φ → ψ, ∀x. φ, ∃x. φ. Negation ¬φ is φ → ⊥. A binder's name (`hint`) is for printing only; formula equality ignores it (`prop.rs:37`), so alpha-equivalent formulas are equal and the kernel has no renaming step.

**Sequents.** A `Theorem` (`rules.rs:106`) holds three private fields: Γ (a map from assumption names to formulas), the conclusion φ, and A (a map from axiom names to the full axiom content used). Write it Γ ⊢_A φ.

**Semantics.**
- A structure M interprets 0, S, + and × over a nonempty domain; `=` is identity.
- M ⊨ A when M satisfies the universal closure of every axiom in A.
- An assignment σ gives each parameter a value in M.
- Γ ⊢_A φ is **valid** when, for every M ⊨ A and every σ: if M, σ ⊨ ψ for each ψ in Γ, then M, σ ⊨ φ.

Every formula a theorem holds is locally closed (each index refers to a binder inside the formula), so σ fixes its truth value.

**Lemmas** (each by induction on the formula; terms contain no binders):

- **L1 (open).** For a locally closed term t: M, σ ⊨ open(φ, t) iff M, σ[x ↦ ⟦t⟧σ] ⊨ φ, reading φ's outermost index as x.
- **L2 (close).** M, σ ⊨ ∀x. close(φ, a) iff M, σ[a ↦ v] ⊨ φ for every v. The same holds for ∃ with "some v".
- **L3 (freshness).** If a does not occur in ψ, then M, σ ⊨ ψ iff M, σ[a ↦ v] ⊨ ψ, for every v.

L1 needs no capture side condition: t contains no indices and terms contain no binders, so substituting t under a binder cannot capture anything (`prop.rs:16`).

**Soundness theorem.** If the kernel produces Γ ⊢_A φ, it is valid. Proof: by induction on the derivation, one case per rule (§3), with the structural facts of §2.

---

## 2. Structural guarantees

These hold for every rule and are what the per-rule arguments rely on.

| Guarantee | Argument | Implementation | Evidence |
|---|---|---|---|
| **G1 Encapsulation.** Every `Theorem` value is the conclusion of a rule application. | Induction on derivations is only valid if no other construction exists. | `Theorem`'s fields are private to `rules.rs`; the only `Theorem { … }` expressions in the code base are in `rules.rs` (checked: `grep -rn 'Theorem {'` over kernel, cli, syntax). The kernel crate has zero dependencies and no `unsafe`. | M3a audit (O5, code review); RLT-004a re-check of the grep. |
| **G2 Consistent contexts.** Combining premises never binds one name to two formulas or one axiom name to two axioms. | Union of contexts is then a well-defined set of assumptions, and A records exactly the axioms used. | `merge` (:136) and `union` (:164) refuse conflicts. | `a_forged_axiom_reusing_a_registered_name_stays_distinguishable`, `axioms_used_accumulate`, `o7_axioms_used_accumulate_through_every_new_rule`. |
| **G3 Dependency preservation.** Each rule's Γ is the union of its premises' Γ minus exactly the assumptions it discharges, each from the one premise named for it; `hyp` alone introduces assumptions. | Discharging only what the rule's argument justifies. | `discharge` (:178): removes h from one premise's Γ; if h is present it must state exactly the expected formula. | R13–R17, R30, R31, R35, R38–R41 (tests/m3a/reject). |
| **G4 Well-formedness.** Every formula a theorem holds is locally closed and well-sorted. | Needed for σ to fix truth values (§1). | `Prop::check_wf` (`prop.rs:96`) in every rule that reads a formula from the step; `check_sort` on every term argument; axioms refuse parameters and bound indices outside their own binders (`check_axiom_body`). | `o1_rules_refuse_dangling_indices_and_axiom_variables`, `well_formedness_refuses_dangling_indices_and_axiom_variables`, R21, R25, R27. |
| **G5 Final checks.** An accepted proof's last theorem has exactly the stated goal; every assumption left in Γ is declared; an unconditional proof has Γ = ∅. The goal and declared assumptions contain no parameters. | With Γ = ∅ and a parameter-free goal, validity means truth in every M ⊨ A. | `check.rs:394–424` (goal and declared assumptions), `check.rs:571–600` (last step, leftover locals, unconditional mode). | `a_proof_of_another_goal_is_rejected`, `unconditional_requires_empty_context_and_conditional_reports_it`, `an_empty_proof_is_rejected`, R13, R14. |
| **G6 Claims are compared.** Each step's written claim must equal the rule's result exactly. | The claim is display; the rule's result is the theorem. | `check.rs:531`. | `a_claim_differing_from_the_rule_result_is_rejected`, R08, R33, CR22. |
| **G7 Names.** Axiom, rule, assumption, parameter and local names are disjoint; each local name is bound to one formula per file. | Prevents one name meaning two things across steps. | `check.rs:380–453`, `check.rs:498–518`, `check.rs:546–566`. | R15, R23a, R23b, R30, R31, R35, R36, R37. |
| **G8 Resource bounds.** Formula nesting ≤ 256; each built formula ≤ 1,000,000 nodes; a proof file ≤ 1,000,000 term nodes. | Bounds affect what can be checked, never what is valid. | `MAX_PROP_DEPTH`, `MAX_PROP_NODES` (`prop.rs:54, 58`); `MAX_FILE_NODES` in `syntax/src/proof.rs`. | R27; node-budget tests in `cli/tests/m3a.rs` and `cli/tests/m3d.rs`; the baseline measurements (docs/RLT-004a-BASELINE.md). |

---

## 3. The 20 inference rules

### Equational rules

**refl** (:203). *Conditions:* t is a well-sorted, locally closed term. *Conclusion:* ∅ ⊢ t = t. *Argument:* = is identity in M. *Evidence:* `refl_accepts_ground_and_rejects_ill_sorted`; used in every M3c–M3e certificate. *Limitations:* none beyond G4.

**symm** (:213). *Conditions:* the premise concludes an equation a = b. *Conclusion:* Γ ⊢ b = a. *Argument:* identity is symmetric. *Evidence:* `symm_swaps_and_keeps_assumptions`; R12a (symm on a conjunction, refused).

**trans** (:225). *Conditions:* premises a = b and b′ = c with b ≡ b′ structurally (no rewriting). *Conclusion:* Γ₁ ∪ Γ₂ ⊢ a = c. *Argument:* identity is transitive. *Evidence:* `trans_accepts_matching_middle_and_rejects_mismatch`; tests/reject/T2 (mismatch S(5 + 0) vs 5 + 0, refused); R12b.

**cong** (:242). *Conditions:* the symbol is in the signature with arity n ≥ 1; exactly n equational premises aᵢ = bᵢ. *Conclusion:* ⋃Γᵢ ⊢ f(a₁…aₙ) = f(b₁…bₙ). *Argument:* the interpretations of S, + and × are functions, so equal arguments give equal values. *Evidence:* `cong_checks_arity`.

**inst** (:275). *Conditions:* the axiom is registered in this proof's axiom set (by name, with identical content); σ binds every axiom variable exactly once to a well-sorted, locally closed term; no extra bindings. *Conclusion:* ∅ ⊢_{ax} σ(body), with the axiom recorded in A. *Argument:* M ⊨ A means the axiom's universal closure holds, so it holds at the values ⟦tᵢ⟧σ; parameters in tᵢ are evaluated by σ. Under the body's own binders the substituted terms need no index shift because they are locally closed. *Evidence:* `inst_rejects_unbound_extra_and_ill_sorted_bindings`, `a_forged_axiom_reusing_a_registered_name_stays_distinguishable`, R24, R25, CR13 (unregistered induction axiom, refused). *Assumptions:* the axioms themselves. Their content is pinned by SHA-256 to `spec/*.rlt`; their truth in ℕ is a mathematical assumption that RLT reports and does not check.

**hyp** (:310). *Conditions:* φ is well-formed; the checker allows the name only if it is declared by the goal or discharged by some step of the same file (`check.rs` `apply`, "hyp"). *Conclusion:* {h : φ} ⊢ φ. *Argument:* if every formula of Γ holds, φ holds. *Evidence:* `hyp_introduces_one_assumption_and_composition_preserves_it`, R03, R13, R14. *Limitation:* `hyp` is the only source of assumptions; an unconditional ACCEPT requires all of them to be discharged (G5).

**subst** (:326). *Conditions:* an equational premise s = t; a motive ψ̂ well-formed under one binder; the second premise concludes exactly open(ψ̂, s). *Conclusion:* Γ₁ ∪ Γ₂ ⊢ open(ψ̂, t). *Argument:* ⟦s⟧σ = ⟦t⟧σ; by L1 both openings evaluate ψ̂ with x mapped to that same value, so they agree. *Evidence:* `subst_uses_the_motive_both_ways_and_cannot_capture`, A10, A14, A15, R09, R33, R34.

### Conjunction and disjunction

**and-intro** (:350). *Conditions:* two premises. *Conclusion:* Γ₁ ∪ Γ₂ ⊢ φ ∧ ψ. *Argument:* truth table. *Evidence:* A04, A12a.

**and-left / and-right** (:366, :375). *Conditions:* the premise concludes a conjunction. *Conclusion:* Γ ⊢ φ (resp. ψ). *Argument:* truth table. *Evidence:* R19 (and-left on a disjunction, refused).

**or-left / or-right** (:385, :395). *Conditions:* the stated other disjunct is well-formed. *Conclusion:* Γ ⊢ φ ∨ ψ. *Argument:* truth table. *Evidence:* A07; M3d's enumeration certificates build each 30-way disjunction this way.

**or-cases** (:406). *Conditions:* the first premise concludes φ ∨ ψ; both case premises conclude the same χ; h1 is discharged from the first case only and, if present, states φ; h2 from the second case only and, if present, states ψ. *Conclusion:* Γ ∪ (Γ₁ ∖ {h1}) ∪ (Γ₂ ∖ {h2}) ⊢ χ. *Argument:* φ ∨ ψ holds at σ. If φ holds, every formula of Γ₁ holds (those other than h1 by hypothesis, h1 because it states φ), so χ holds. The ψ case is symmetric. An h1 used in the second case stays in Γ. *Evidence:* A12b, A16, R16 (cross-discharge refused), R20 (different conclusions refused), R39.

### Implication and falsity

**imp-intro** (:434). *Conditions:* the stated antecedent φ is well-formed; h, if present in the premise's Γ, states exactly φ. *Conclusion:* Γ ∖ {h} ⊢ φ → ψ. *Argument:* suppose Γ ∖ {h} holds and φ holds; then all of Γ holds, so ψ holds. *Evidence:* A05, R17 (antecedent mismatch), R31, R35.

**imp-elim** (:447). *Conditions:* the first premise concludes φ → ψ; the second concludes exactly φ. *Conclusion:* Γ₁ ∪ Γ₂ ⊢ ψ. *Argument:* modus ponens is truth-functionally valid. *Evidence:* R18; every M3c certificate closes each zero branch with q2 and imp-elim.

**false-elim** (:471). *Conditions:* the premise concludes ⊥; the stated φ is well-formed. *Conclusion:* Γ ⊢ φ. *Argument:* ⊥ holds in no structure under no assignment, so the premise's Γ never holds at any σ, and the implication is vacuously valid. *Evidence:* A06, R11 (false-elim from an equation, refused).

### Quantifiers

**forall-intro** (:485). *Conditions:* the parameter a occurs in no formula of the premise's Γ; the checker also requires a to be declared in the file's `params`. *Conclusion:* Γ ⊢ ∀x. close(φ, a). *Argument:* for any v, by L3 Γ still holds at σ[a ↦ v], so φ holds there; by L2 the universal formula holds at σ. *Evidence:* A01, A02, A09, R01, R02 (parameter in Γ via a conjunction), R03, R22a, R22b.

**forall-elim** (:497). *Conditions:* the premise concludes ∀x. φ; t is well-sorted and locally closed. *Conclusion:* Γ ⊢ open(φ, t). *Argument:* L1 with the value ⟦t⟧σ. *Evidence:* A09, R08. M3e's E-T1 instantiates its cited theorems this way.

**exists-intro** (:514). *Conditions:* t is well-sorted and locally closed; the stated body is well-formed under one binder; the premise concludes exactly open(body, t). *Conclusion:* Γ ⊢ ∃x. body. *Argument:* by L1 the body holds with x ↦ ⟦t⟧σ, which witnesses the existential. *Evidence:* A03, A08, R10; tests/m3c/accept/CA01 (2 ∣ 8 with witness 4); CR01 (false witness, refused).

**exists-elim** (:537). *Conditions:* the first premise concludes ∃x. φ; the parameter a occurs in neither φ nor the branch conclusion χ; h, if present in the branch's Γ₂, states exactly open(φ, a); a occurs in no formula of Γ₂ ∖ {h}. The first premise's Γ₁ passes through whole and may mention a. *Conclusion:* Γ₁ ∪ (Γ₂ ∖ {h}) ⊢ χ.

*Argument:* assume Γ₁ and Γ₂ ∖ {h} hold at σ.
1. Γ₁ holds at σ, so ∃x. φ holds; choose a value w with φ true at x ↦ w.
2. Let σ′ = σ[a ↦ w]. Since a ∉ φ, open(φ, a) holds at σ′ (L1, L3), so h holds at σ′.
3. Since a ∉ Γ₂ ∖ {h}, those formulas still hold at σ′ (L3). So all of Γ₂ holds at σ′, and χ holds at σ′.
4. Since a ∉ χ, χ holds at σ (L3).

Γ₁ is used only at σ, before a is reassigned, which is why it may mention a.

*Evidence:* `exists_elim_q_f_keeps_the_premise_gamma_and_discharges_only_h`; A13 (Γ₁ mentions a, accepted); R04 (a escapes into the conclusion), R05 (a in the body), R06 (a in the branch's Γ), R07 (witness mismatch), R32, R40; CR03, CR04, CR05 (stale, reused and escaping eigenvariables in divisibility certificates).

---

## 4. Theorem citation

Citation (M3c) lets one proof use a theorem that another proof file established. It is a *theorem dependency*, reported separately from the 20 inference rules.

**Conditions.**

*In the kernel* (`check.rs:454–485`, then the `cite` step at `check.rs:363`). Each cited theorem, supplied by the caller as a kernel `Theorem` value, must be:
- unconditional (Γ = ∅);
- well-formed and locally closed, with no parameter in its conclusion;
- built only from axioms whose name *and full content* match an axiom registered for the citing proof.

A citation label may be declared once. A `(cite LABEL)` step's theorem is the cited theorem itself, unchanged, and its claim is compared like any other step (G6).

*In the command-line checker* (`cli/src/lib.rs`, `check_inner`), before the kernel sees a cited theorem:
1. The path is repository-relative, ends in `.rlt`, contains no `..`, absolute or symlinked component.
2. No cycle among canonical paths.
3. At most 8 nested citing files (`MAX_CITATION_DEPTH`) and 64 cited-file checks per top-level check (`MAX_CITED_CHECKS`).
4. The file is read once; its SHA-256 must equal the recorded hash.
5. Those same bytes are checked from scratch, recursively.
6. The cited file must be accepted, with the same axiom set and convention, and Γ = ∅.

**Argument.** By G1 the cited value is a kernel-produced theorem ∅ ⊢_{A′} ψ, valid by the soundness theorem. The kernel checks A′ ⊆ A by content, so every M ⊨ A also satisfies A′, and ψ holds in M under every σ (ψ has no parameters). So ∅ ⊢_A ψ is valid, and using it as a premise is the same as using an axiom instance. The CLI conditions are about *which* theorem is cited (the right file, at the recorded content, without loops); the soundness of the citing proof needs only the kernel conditions.

**Evidence.** CR17a (hash differs), CR17b (same goal, other file), CR18 (cites a rejected file), CR19 (other axiom set), CR20 (conditional theorem), CR21a–f (self-citation and cycles of length 2 and 3), CR22 (stated formula differs), CR23a/b (parent and absolute paths), CR24 (label swapped); `cli/tests/m3c.rs` depth test (8 citing files accepted, 9 refused); M3e's citation graph (E-T1 cites E_FWD and E_BWD, 49 cited-file checks).

**Limitations.** The hash binds the cited file's bytes; it says nothing about whether that file's statement faithfully renders a source (see docs/STATEMENT-RECORDS.md). The CLI path conditions are evidence-backed engineering checks, outside the soundness argument.

---

## 5. Trusted assumptions

The soundness argument covers the kernel's rules. A user relying on an ACCEPT verdict also trusts:

| Trusted component | Why it is trusted | What would go wrong |
|---|---|---|
| The axioms in `spec/` | They are the premises of every theorem. Content is pinned by SHA-256 and printed in every report. | A false axiom would make false statements provable; RLT reports every axiom used so the reader can judge. |
| The Rust compiler, standard library (`BTreeMap`, `BTreeSet`, `String`) and hardware | The kernel is ordinary Rust without `unsafe` or dependencies. | A miscompilation could break any check. |
| The kernel's term and formula operations (`term.rs`, `prop.rs`): `open`, `close`, substitution, equality, well-formedness | The lemmas L1–L3 are about these operations. | An error here would break L1–L3. Evidence: `o3_open_and_close_laws_over_generated_formulas` (2,000 generated formulas), `open_cannot_capture`, `open_close_round_trip_and_alpha_equality`. |

**Outside the soundness boundary, by design.** The parser, the NAT-DEC/NAT-LOGIC/NAT-DEF elaboration of written statements, the definition expansion (`lt`, `dvd`, `free235`) and the report printer turn text into the formula the kernel checks and print the result. An error there could make RLT check a *different statement* from the one a reader intended. That is a translation question, and every report makes it inspectable by printing the written goal, the checked goal, its kernel form and the definitions expanded. Generators (`rlt gen`, `scripts/gen-*.py`) are untrusted: the kernel checks every step they write.

---

## 6. Limitations

- **Paper argument.** The argument in §§1–4 is written mathematics, checked by review and the M3a audit. It is not a machine-checked proof of the Rust code.
- **Evidence is finite.** 227 tests, 79 derivation-attack files (M3c) and the adversarial fixtures of M3a–M3e exercise the implementation; they cannot exclude every defect.
- **First-order classical semantics.** Validity is relative to all structures satisfying the axioms, including nonstandard ones. A theorem of Robinson's Q is true in ℕ because ℕ satisfies Q.
- **Resource limits** (G8) decide what can be checked in practice. Measured limits are in docs/RLT-004a-BASELINE.md.
- **Translation and correspondence** are separate questions: an accepted theorem establishes the checked formula only (docs/STATEMENT-RECORDS.md).

---

## 7. Evidence summary

| Evidence | Where | What it checks |
|---|---|---|
| Kernel unit tests (29) | `kernel/src/*.rs` | per-rule behavior, encapsulation-relevant cases, open/close laws |
| CLI and fixture tests (177) plus syntax tests | `cli/tests/*.rs`, `tests/**` | accept/reject verdicts and messages for every fixture; size and depth limits; citation conditions |
| `scripts/verify.sh` | repository | full suite, source excerpts, Lean replays and axiom gates, record validation |
| `scripts/compare-reports.sh` | repository | byte-identical reports against an earlier binary after each milestone |
| Independent audits | docs/RLT-001-AUDIT.md, RLT-002-AUDIT.md, and the audit sections of RLT-003-M3a … M3e | adversarial review of each milestone's changes |
| Lean 4 replays | `lean/*.lean` | each accepted statement restated and checked independently by Lean (statement only, not RLT's derivation) |
| CI | `.github/workflows` | `check` and `gdt-comparison` jobs on every pushed commit |

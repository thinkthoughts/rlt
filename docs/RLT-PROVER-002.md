# RLT-PROVER-002 — Registered definitions and theorem discovery

**Request.** RLT-PROVER-002 (2026-11-20), stages 1 and 2 of three:

| Stage | Example | What RLT establishes |
|---|---|---|
| 1 | `double(x) := x + x` | safe definition expansion |
| 2 | `double(0) = 0` | a definition combined with Robinson's Q |
| 3 (later) | math29, modulo 30 | more expressive definitions with exact conventions |

Branch `rlt-002-definitions` from `main` at `9dd36ca`. Not merged; no PR.

**Success criterion, met for stages 1 and 2.** Given a new definition (`definitions/double.rlt`) and a goal that uses it, `rlt prove` constructs a proof. The certificate is bound to the exact registered definitions by the file's SHA-256, and each definition has its own content identity. The kernel independently verifies the unfolded statement. On every replay, `rlt check` recomputes the link between the written statement and the kernel statement from the definitions at that hash. No axiom, rule or kernel symbol is added.

## 1. What existed (Phase 1)

| Component | Finding |
|---|---|
| Kernel signature | fixed: `0`, `S`, `+`, `×` (`kernel/src/term.rs`, `arity`); any other symbol fails the sort check |
| Formulas | =, ⊥, ∧, ∨, →, ∀, ∃, locally nameless (de Bruijn indices), alpha-equality |
| Definitions (M3b) | `spec/definitions.rlt`: **formula** abbreviations only (`<`, `∣`, `free235`), pinned by hash in the binary, read under NAT-DEF and expanded before the kernel |
| Elaboration | `syntax/src/elab.rs`: written text + convention → kernel formula; re-run by every `rlt check` from the written goal |
| Prover | `cli/src/prove.rs` (RLT-PROVER-001): goal file → untrusted search → certificate → `certify` (header equal to the goal file's, then kernel replay) |
| Discovery | `cli/src/discover.rs`: a fixed grammar over the signature, NAT-LOGIC only |

So `double(x) := x + x`, a **term** (function) definition, had no mechanism. Adding it to `spec/definitions.rlt` would change a pinned hash printed in every NAT-DEF report. Adding `double` to the kernel signature would change the trusted base.

**Smallest sound extension chosen.** Treat a function definition as a term abbreviation, unfolded by the elaborator before the kernel, exactly as M3b treats formula abbreviations. Keep it in a separate, content-hashed registry. The kernel and its 20 rules are unchanged, and so are the pinned axiom sets, `spec/definitions.rlt`, NAT-DEC, NAT-LOGIC, NAT-DEF and every existing proof file.

## 2. Registration syntax and semantics (Phase 2)

```lisp
(rlt-definitions
  (id DOUBLE)                  ; registry id
  (convention NAT-LOGIC)       ; bodies and written forms are read in NAT-LOGIC
  (domain Nat)                 ; the natural numbers of the kernel signature
  (define-function double (x) (+ x x)
    (written "double(x) := x + x")
    (origin rlt-authored)))    ; or (source ID), a registered source
```

**Meaning.** `double(t)` abbreviates the body with `x` replaced by `t`. A statement written with registered functions means exactly its unfolding.

**Checks when a file is read** (all fail closed; `syntax/src/fdefs.rs`, `load_function_definitions` in `cli/src/lib.rs`):

| Requirement | Check |
|---|---|
| stable name and identifier | name: ≥ 2 lower-case letters, not a syntax word, rule, registered axiom or pinned M3b definition, not defined twice; registry id; per-definition **content sha256** over name, parameters, unfolded body, formal body, written form, convention, domain, provenance and the dependencies' identities |
| parameters and exact defining expression | distinct one-letter parameters; formal body over parameters, numerals, `S`, `+`, `*` and earlier functions, each at its arity; the **written form must elaborate to exactly the formal body** |
| explicit convention and domain | `(convention NAT-LOGIC)` and `(domain Nat)` required; any other value is refused |
| source identity and content hash | `(source ID)` must be a registered reference or available source; otherwise `(origin rlt-authored)`; the file's bytes are bound by SHA-256 wherever it is used |
| dependencies | a body may use only functions registered above it; dependencies are recorded and enter the content identity |
| acyclic expansion, checked substitution | self-use and later names are undefined symbols (recursion and cycles refused); bodies are stored fully unfolded; size bounds before building |
| no axiom injection | a definitions file accepts only `id`, `convention`, `domain`, `define-function`; `(axiom …)`, `(readingpoint …)`, `(relationship …)`, `(context …)` are refused, also inside a definition |

**Capture.** A body is a term and contains no binder, so substituting an argument that mentions a use-site bound variable leaves its de Bruijn index pointing where it pointed (`unfolding_under_binders_keeps_bound_indices`; case 2 below).

### Using definitions: convention NAT-FUN

```lisp
(rlt-goal
  (id F2.double_zero)
  (convention NAT-FUN)
  (definitions "definitions/double.rlt" "8dabb459…2f7")
  (goal "double(0) = 0")
  (mode unconditional)
  (axioms ROBINSON-Q))
```

NAT-FUN (`conventions/nat-fun.yaml`, pinned in `PINNED_CONVENTIONS`) reads everything NAT-LOGIC reads, plus `name(t, …)` for the functions of the file the `(definitions …)` field names. The field is required under NAT-FUN and refused under every other convention.

Under NAT-FUN a proof file may not assert a `readingpoint`, `relationship` or `context`: none is registered for a definition-aware proof yet, so the label is refused rather than echoed (see §6, finding 1).

## 3. Trust boundary

```
definitions file ──sha256──► stated in the goal/certificate header
       │ parsed, checked (§2), content identities computed
       ▼
written goal ──NAT-FUN elaborator (untrusted)──► folded formula ──unfold──► kernel formula
                                                   (reported)                 (reported; checked)
                                                                                   │
certificate steps ────────────────────────────────────► kernel replay (trusted) ◄──┘
```

- **Kernel-checked.** The unfolded formula follows from the axioms used under Γ by the 20 rules.
- **Mechanically bound, outside the kernel.** The definitions file's bytes (SHA-256, re-hashed on every check). The unfolding from written goal to kernel goal, recomputed from those bytes on every `rlt check`, so a certificate cannot carry a stale or substituted translation. The header equality between goal file and certificate (`certify`). Every column of the discovery manifest (§5).
- **Cross-checked for the seven goals (three distinct parts).**
  1. *Declaration correspondence, checked by Lean (authoritative).* `scripts/check-definitions-lean.py` rewrites RLT's printed `folded` and `checked goal` lines, the printed unfolding of `quadruple`, the statements `rlt prove` prints for G1 and G2, and the registered written definitions, into Lean syntax. From these alone, never from the Lean file, it writes an expected module (`def double`, `def quadruple`, `def T.expected : Prop := STATEMENT` for each of the 17 theorems, and `theorem FN_unfolds.by_rfl : FN_unfolds.expected := Iff.rfl`), and Lean 4.34.0-rc2 compiles it and `lean/ProverDefinitions.lean` separately. `scripts/lean/DeclCheck.lean`, a program that never imports the companion, loads both compiled environments and compares the declarations Lean elaborated, as kernel terms up to alpha-equivalence: each `def`'s type and value (safe, no universe parameters), each theorem's type against `T.expected`, the companion's imports (`Init` only), no other constant (Lean's own `T._proof_…` lemmas are allowed only when T's proof uses them), and each theorem's axioms, collected by walking the kernel terms (propext, Classical.choice, Quot.sound only). The companion must elaborate with no message other than its `#print axioms` lines. Before the comparison, the pinned toolchain's `leanchecker` replays every constant of the compiled companion through Lean's kernel (H1), so a declaration stored without kernel checking is refused; `scripts/verify.sh` also runs `leanchecker --fresh`, which replays every constant, core Lean's included, into an empty environment. Comments, strings, escaped identifiers `«…»`, macros, notation, instances and text inside proofs therefore cannot supply evidence: only the constants Lean declared and their elaborated types count. This is what ties the Lean declarations to the specific unfolding RLT printed.
  2. *Source layout (additional defence).* After removing Lean comments (`--` and nested `/- … -/`), the file must fit a small explicit declaration grammar (one namespace; only `def`s of the registered functions and the 17 expected theorems; `#print axioms` for each; no other command, attribute, modifier, string, character literal or escaped identifier), each name declared once with the expected statement text. Both layers must pass; without Lean the correspondence is reported UNVERIFIED (exit 3).
  Lean's kernel checks each `FN_unfolds` by `Iff.rfl` twice: in the clean expected module and in the companion. `Iff.rfl` accepts any definitionally equal pair — `(double 2 = 4) ↔ (3 + 1 = 4)` also holds by `Iff.rfl` — so it confirms meaning-preservation, while the fidelity of RLT's particular unfolding rests on the exact comparison of elaborated types in part 1.
  3. *Not established.* A general formal proof that RLT's elaborator and unfolder implement the same semantics as Lean. The comparison is seven demonstrated cases.
- **Assumed.** The RLT elaborator and unfolder (`syntax/`) are trusted for source-to-core fidelity, as they already were for every written goal. Their output is printed in every report so a reader can audit it. The truth of Robinson's Q. That `double` captures what a reader means by "double" (correspondence, not proof).

## 4. Proofs with definitions (Phase 3)

`tests/definitions/goals/` → `tests/definitions/certificates/` (each passes plain `rlt check`; reproduced byte for byte by the tests). Robinson's Q throughout; Γ = ∅.

| Goal | Statement (folded) | Kernel checks | Axioms used | Steps |
|---|---|---|---|---|
| F1 (stage 1) | ∀x. double(x) = x + x | ∀x. x + x = x + x | none | 2 |
| F2 (stage 2) | double(0) = 0 | 0 + 0 = 0 | add_zero (Q4) | 1 |
| F3 | double(2) = 4 | 2 + 2 = 4 | Q4, Q5 | 7 |
| F4 | ∀x. x + (x + 1) = double(x) + 1 | ∀x. x + (x + 1) = (x + x) + 1 | Q4, Q5 | 16 |
| F5 | ∀x. ¬(double(x + 1) = 0) | ∀x. ¬((x + 1) + (x + 1) = 0) | Q2, Q4, Q5 | 15 |
| F6 | ∀x. quadruple(x) = double(x) + double(x) | ∀x. (x + x) + (x + x) = (x + x) + (x + x) | none | 2 |
| F7 | quadruple(1) = 4 | (1 + 1) + (1 + 1) = 4 | Q4, Q5 | 16 |
| G1 | ∀x. double(x) = 2 × x | — | NOT_FOUND_WITHIN_BOUNDS | — |
| G2 | ∀x. double(x) = x × 2 | — | NOT_FOUND_WITHIN_BOUNDS | — |

F6 and F7 use `definitions/quadruple.rlt`, where `quadruple(x) := double(double(x))` depends on `double`. G1 and G2 are true in ℕ (Lean proves both with `omega`), but they need induction, which Q lacks. RLT reports them as not found and makes no claim about their truth.

The stage-2 certificate in full:

```lisp
(rlt-proof
  (id F2.double_zero)
  (convention NAT-FUN)
  (definitions "definitions/double.rlt" "8dabb459817f7b225419456080a4e81eb675daf9a830dba529192ff50cfbc2f7")
  (goal "double(0) = 0")
  (mode unconditional)
  (axioms ROBINSON-Q)
  (steps
    (s1 (= (+ 0 0) 0) (inst add_zero (x 0)))))
```

## 5. Definition-aware discovery (Phase 4)

`rlt discover --definitions FILE DIR [--limit N]` (`cli/src/discover_defs.rs`). Over the unary functions of FILE, with atoms A = {x, 0, 1, 2} and U = {f(a)}:

- L ∈ U ∪ {u ∘ a}
- R ∈ A ∪ {a ∘ b} ∪ U (one orientation of U = U)
- closed by ∀x when x occurs

Every candidate uses a registered function. Bounds are those of `rlt discover`. Ids are `DF0001…`, so they never collide with the `D`/`T` ids.

**Committed record** `research/definition-discovery` (over `definitions/double.rlt`): 1430 candidates, **126 proved** (4 by unfolding, 122 by inference), 1304 not found, 0 errors, about 29 s.

`manifest.tsv` columns:

| Column | Binding |
|---|---|
| id, statement (original, folded) | id names its candidate; statement equals the certificate's written goal |
| elaborated | equals the replayed kernel goal and an independent re-elaboration |
| definitions | `name@content-sha256` for every function the statement's meaning depends on |
| axiom_set, mode, assumptions | from the certificate and kernel theorem (ROBINSON-Q, unconditional, ∅) |
| certificate, sha256 | `DIR/ID.rlt`; hash re-computed; exactly those bytes replayed |
| steps, axioms_used | from the replay |
| kind | `unfolding` iff the sides are identical after unfolding; must agree with the kernel (`unfolding` ⇔ no axiom used) |
| depth | **search metadata**, not kernel-verified (only checked to lie within the bound); the header says so |

The four `unfolding` theorems are `∀x. double(x) = x + x`, `double(0) = 0 + 0`, `double(1) = 1 + 1` and `double(2) = 2 + 2`. General `inference` theorems include `∀x. double(x) + 0 = double(x)`, `∀x. double(1) + x = 2 + x` and `∀x. double(1) × x = 2 × x`. Most rows are closed evaluations. Some hold for every term in place of the use, such as `∀x. double(x) × 0 = 0`: `kind` separates unfolding from inference, but it does not measure dependence on the definition (§7).

`rlt discover --verify DIR` recognises the record by its manifest header. It loads the definitions at the hash in `summary.txt` (a changed file refuses the whole record) and checks every row as above. It also checks the exact header (the `definitions` field included; any `readingpoint`, `relationship`, `context` or `cites` is refused), allows only the generator's comment, and requires full coverage: every candidate exactly once in the manifest or in `not-found.tsv`, byte for byte, with summary counts that match. PROVED and NOT_FOUND_WITHIN_BOUNDS keep their RLT-PROVER-001 meanings.

## 6. Adversarial tests (Phase 5)

`cli/tests/definitions.rs` (18 tests) and `syntax/src/fdefs.rs` (9 unit tests); the audit regressions are listed in §9:

| # | Case | Result |
|---|---|---|
| 1 | valid definition and theorem | F1–F7 proved; axioms used exactly as tabulated; certificates reproduced byte for byte and ACCEPTed by `rlt check`, whose report shows the folded goal and the trust line |
| 2 | unfolding under quantifiers, no capture | a parameter named `y` used under ∀x ∀y in both orders and under ∃y unfolds to the expected indices; ∃-witness proof found |
| 3 | recursive or cyclic definition | refused (self-use; a cycle through a later name is an undefined symbol) |
| 4 | unbound variable | refused; also wrong arity and a written form disagreeing with its body |
| 5 | definition changed after proof generation | the certificate is refused ("changed after this statement was written"), also for a whitespace-only edit |
| 6 | conflicting definition, same name | refused ("defined twice"); a name taken by a pinned M3b definition (`lt`) refused |
| 7 | valid certificate attached to the wrong definition | a registry where `double(x) := 2 × x`, cited with its true hash, gives a different kernel goal, so the proof is refused. Also refused: a wrong hash, a removed `(definitions …)` field, NAT-LOGIC with `(definitions …)`, a `../` path |
| 8 | attempted unregistered axiom | `(axiom …)` in a definitions file or a definition refused; a certificate step `inst double_def` refused by the kernel |
| 9 | unsupported readingpoint / relationship / context | refused in definitions files, in NAT-FUN certificates, and in discovery certificates (even rehashed into the manifest) |
| 10 | not found within bounds | G1, G2 NOT_FOUND_WITHIN_BOUNDS with the no-truth-claim sentence; a provable goal with budget 1 is only "not found" |
| — | discovery record | verifies; refused after tampering with kind, definition identity, elaborated statement, a fabricated row, a depth outside the search bound, the summary hash, an added label, or a changed definitions file; a rerun reproduces the record byte for byte |
| — | earlier conventions | a NAT-LOGIC goal using `double(…)` is still an error; `spec/definitions.rlt` keeps its pinned hash |

**Finding 1 (pre-existing, outside this milestone's scope).** Under the earlier conventions, `rlt check` prints a `(relationship …)` or `(readingpoint …)` label without checking that it is registered; registration is enforced by `rlt rp` and `rlt validate`. For example, `R_math29_pairing.rlt` with its relationship renamed to `nope.nope` still ACCEPTs, with the label echoed. Changing that would alter established verdicts, so it is left as is and reported here. NAT-FUN refuses the labels outright.

## 7. What was established, and what remains assumed

**Mechanically established**

- The kernel accepts seven unconditional theorems under ROBINSON-Q (F1–F7), with the axioms used listed, and 126 discovery theorems. Each was re-checked from its bytes by `rlt check`'s function.
- Each certificate's link to its definitions is a recomputation on every check: file SHA-256, then parse and validation, then unfolding of the written goal, then the kernel goal compared with the derivation's conclusion.
- For the seven goals and the three further Lean theorems, Lean's elaborated declarations in `lean/ProverDefinitions.lean` equal, as kernel terms, an expected module built from RLT's printed folded and unfolded statements and the registered definitions (declaration-level correspondence checked by Lean; §3). Lean 4.34.0-rc2 checks each equivalence up to definitional equality (`Iff.rfl`) and each statement's truth in ℕ, using standard axioms only.
- No kernel, axiom-set, rule or pinned M3b change.

**Implementation or interpretation assumptions**

- RLT's elaborator and unfolder are trusted for fidelity between source and core. The Lean comparison covers the seven goals, not the elaborator in general, and `Iff.rfl` validates equivalence, not RLT's expansion algorithm (§3).
- `unfolding` / `inference` is a syntactic classification checked against the kernel's axioms used. It does not detect statements that hold for any term in place of the definition's use; a stronger "depends on the definition" test (prove again with each use generalised) is a possible addition.
- Only unary-or-more **function** definitions over 0, S, +, × are supported: no constants, no predicate definitions in user registries, no definitions by cases or recursion. Stage 3 (math29, modulo 30) will likely need predicate definitions in registries (for example an admissible-residue predicate over the existing `free235`, `<`, `∣`), which this mechanism does not yet provide. The existing statements should be inspected first, as planned.
- One definitions file per proof; dependencies across files are not supported.
- Folded notation is read in the written goal and written assumptions only. Proof steps are written over the kernel signature: a step claiming `(= (double 0) 0)` is refused by the kernel (it rejects the unknown symbol), which fails closed. `rlt lean` exports the unfolded kernel statement.
- That a registered definition captures its intended meaning is a correspondence question. PiπΠ, Spiral N and Triplet N are untouched.

## 8. Reproduce

```sh
cargo build --release
target/release/rlt prove tests/definitions/goals/F2_double_zero.goal --out target/F2.rlt
target/release/rlt check tests/definitions/certificates/F1_double_unfolds.rlt
target/release/rlt prove tests/definitions/notfound/G1_double_two_times.goal   # exit 1
target/release/rlt discover --definitions definitions/double.rlt target/dd   # an untracked directory
target/release/rlt discover --verify research/definition-discovery
cargo test --release --test definitions
python3 -I scripts/check-definitions-lean.py target/release/rlt --verbose     # needs Lean; prints each declaration's evidence
(cd lean && lean ProverDefinitions.lean)
scripts/verify.sh SOURCE_CHECKOUTS_DIR                                         # stage "registered function definitions"
```

## 9. Independent audit and remediation

An independent read-only audit of `3b0fcea` found no kernel-soundness defect and no way to attach a valid proof to a different written theorem. Two findings were fixed before merge, a third (D5) after the final release audit a fourth (D6) after its retest, and a hardening (H1) after the final Lean-layer audit; two are carried to the next milestone.

**D1 — unbounded unfolding (fixed).** Each use was size-checked, but the formula's total was checked only after full unfolding, so a ~10 KB goal (400 uses of `ad(ad(x))` over the chain `aa(x) := x + x` … `ad(x) := ac(ac(x))`, ~52M nodes) exhausted memory and was killed by the OS. The unfolder (`syntax/src/fdefs.rs`) now carries a cumulative node budget through every nested expansion: each instantiation's exact result size is computed from the body's parameter occurrences and the argument sizes, without building it, and compared with the budget (nodes already committed elsewhere in the formula, across both sides of `=` and every connective) before any node is allocated. Size arithmetic is checked: an overflow is an error, never a wrapped count. Loading a definitions file applies the same rule against `MAX_BODY_NODES`. The audit's case is now refused in about 0.2 s with `MALFORMED — … would exceed 1000000 nodes`. Valid expansions are unchanged; the seven certificates and the 126 discovery certificates reproduce byte for byte. Regressions: `instantiated_size_is_exact`, `cumulative_budget_refuses_the_audit_d1_expansion_without_building_it`, `nested_growth_is_refused_before_construction`, `budget_is_shared_across_sides_and_connectives`, `size_accounting_refuses_overflow`, `oversized_definition_bodies_are_refused_while_loading` (unit), and `audit_d1_many_uses_are_refused_with_an_error_not_oom` (end to end through `rlt check`). An argument of a parameter that its body ignores still counts while it is built, so the budget bounds peak memory as well as the result.

**D4 — Lean comparison overstated (fixed).** This report and `lean/ProverDefinitions.lean` described `Iff.rfl` as an independent implementation of RLT's unfolding. `Iff.rfl` checks definitional equality and accepts other equivalent unfoldings (`(double 2 = 4) ↔ (3 + 1 = 4)`). §3, §7, the Lean header and the script's docstring now separate the script's textual comparison, Lean's checks of the hand-written definitions and theorems, and the absence of any general proof about the elaborator. The script also now requires the Lean `def`s to equal the registered written forms. Regression: `audit_d4_lean_comparison_script_checks_text_not_just_definitional_equality` (a definitionally equal but different F3 unfolding, and a different `def double`, are both refused by the script).

**D5 — comparison by substring (fixed).** The final release audit of `13ff7b7` found that the D4 script searched the raw file, comments included: with the correct `F3_unfolds` inside `/- … -/` and `theorem F3_unfolds : (double 2 = 4) ↔ (3 + 1 = 4) := Iff.rfl` compiled instead, Lean accepted the file and the script reported 7 of 7. The script now removes comments, holds the file to the declaration grammar in §3, and compares each declaration's whole statement. Regressions: `audit_d5_lean_comparison_refuses_hidden_or_misplaced_declarations` covers the attack and its variants (line, nested-block and doc comments; duplicate declarations, including in another namespace; a mismatched name; the expected text in a string, in an unrelated theorem and inside another proof; an `instance` redefining `+`; an unterminated comment; a different or comment-hidden `def`) and the original file; `scripts/verify.sh` also compiles the attack with Lean when Lean is present and requires Lean to accept it and the script to refuse it. The grammar is deliberately narrow: a legitimate change to the Lean file's shape (another helper lemma, an `open`) needs the script's allowlist updated with it.

**D6 — the comparison trusted a Python reading of Lean source (fixed).** The release retest of `0987be5` found that escaped identifiers defeat the D5 lexer: in `by have «/-» : True := trivial` Lean reads one identifier, the script read the start of a comment, and `«-/»` closed it, hiding a compiled `theorem F3_unfolds : (double 2 = 4) ↔ (3 + 1 = 4) := Iff.rfl`; a correct `F3_unfolds` written inside `«…»` was an identifier to Lean and a declaration to the script. Lean accepted the file and the script reported 7 of 7. The verdict now comes from the declarations Lean elaborates (§3, part 1); the source-layout check stays as a second layer and also refuses `«` and `»` anywhere. Regressions: `audit_d6_lean_layer_compares_elaborated_declarations` requires Lean to compile each mutant that should compile, and the Lean layer on its own (`--layer lean`) and the full checker to refuse: the exact escaped-identifier attack; a provable wrong `F3_unfolds`; the correct one in a comment; a missing and a duplicate `F3_unfolds`, and one in another namespace; `def double … := x + x + 0` (definitionally equal, so every proof still compiles); a `macro_rules` rewriting `2 + 2` to `3 + 1` (the source text stays correct; the elaborated type does not); an `instance` for `+`; an extra `axiom`; a hand-written theorem under an internal name (`F3_holds._proof_9`); `import Lean`; a `sorry` proof (which the layout layer alone would accept). The original file passes both layers. `scripts/verify.sh` compiles the attack with Lean and requires Lean to accept it and both the Lean layer and the full checker to refuse it. Without Lean, the test and `verify.sh` fail unless `RLT_ALLOW_NO_LEAN=1`.

**H1 — kernel replay (hardening).** The final Lean-layer audit of `a9d466d` noted that `set_option debug.skipKernelTC true` passes the Lean layer on its own, so a companion proof could be stored without Lean's kernel checking it; the full verdict refused it only because the source-layout layer rejects `set_option`. A concrete case: under that option, a `run_cmd` metaprogram adds `F3_holds : double 2 = 4` with the proof `True.intro`. Lean compiles the file cleanly, and the declaration's type equals the expected proposition, so only the kernel can tell; in `a9d466d` the Lean layer refused it solely by its Init-only import rule. The Lean layer now replays every constant of the compiled companion with the pinned toolchain's `leanchecker` before the comparison, and refuses on any kernel error (here: `declaration type mismatch`). `scripts/verify.sh` compiles this case and requires the replay refusal, and additionally runs `leanchecker --fresh` on the committed companion. Regression: `audit_h1_kernel_replay_refuses_unchecked_declarations` (the option on a valid theorem: refused by the complete verification, replayed and matched by the Lean layer; the `True.intro` case: refused at the replay by the Lean layer and the complete verification). `leanchecker` is Lean's own replay tool, not an external verifier: it re-runs the same kernel, so it detects declarations that bypassed the kernel, not a defect in the kernel itself.

**Carried to the next milestone (documented, not changed here).**

- **D2 — canonical definition identity.** A definitions file is identified by its repository path and SHA-256. Registry ids and function names are not registered anywhere, and any repository-relative file is accepted, so a different file may also say `(id DOUBLE)` and define `double(x) := x + 1`; its certificates and a complete discovery record verify, honestly labelled with that file's path and hash. A pinned registry of definitions files (as for axiom sets) or unique ids is needed before predicate definitions.
- **D3 — generic versus definition-sensitive theorems.** `unfolding` / `inference` is not "depends on the definition". The audit found 39 of the 126 discovery theorems true for every function in place of `double` (Q laws applied to an opaque term, e.g. `double(x) × 0 = 0`), 83 materially definition-dependent inference theorems and 4 unfoldings. A generalisation check (prove again with each use replaced by a fresh term) is planned with predicate definitions.
- Note: `depth` in the discovery manifest is search metadata checked only against the bound; a depth falsified within the bound (0 → 1) is accepted, as the manifest header says.

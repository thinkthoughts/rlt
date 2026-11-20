# RLT prover — from proof checker to theorem prover

**Request.** "RLT-002 — From Proof Checker to Theorem Prover" (2026-11-19). The repository already uses the name RLT-002 for the specification-records milestone (`docs/RLT-002.md`), so this work lives under the name *prover* (branch `rlt-002-prover`, this file, `cli/src/prove.rs`, `cli/src/discover.rs`, `tests/prover/`, `research/prover-discovery/`) and leaves the earlier RLT-002 documents untouched.

**Success criterion, met.** A user supplies a statement with no proof (a goal file). `rlt prove` constructs a derivation, writes it as an ordinary `rlt-proof` certificate, and the existing RLT kernel replays that certificate from its bytes and accepts it. Ten such goals are in `tests/prover/goals/`; their certificates are in `tests/prover/certificates/` and each one passes a plain `rlt check`.

**What stayed the same.** `kernel/` and `syntax/` are unchanged: the same first-order logic, the same 20 inference rules, the same Robinson's Q and other pinned axiom sets, explicit assumptions, and the reading-point architecture. Every test that passed before passes now. The published site (`index.html`, `index_261111.html`, `init.html`) is unchanged.

## 1. Phase 1 — what existed

| Component | Where | Notes |
|---|---|---|
| Term representation | `kernel/src/term.rs` | `0`, `S`, `+`, `×`; parameters (`Param`), de Bruijn indices (`Bound`), axiom variables (`Var`); unary numerals ≤ 1000 |
| Formula representation | `kernel/src/prop.rs` | =, ⊥, ∧, ∨, →, ∀, ∃, locally nameless, alpha-equality |
| Kernel proof format | `kernel/src/check.rs` | `ProofObject`: goal, mode, declared assumptions, parameters, labelled steps (claim, rule, args), citations |
| Rules | `kernel/src/rules.rs` | 20 rules: refl, symm, trans, cong, inst, hyp, subst and 13 logical rules |
| Parser | `syntax/src/sexpr.rs`, `syntax/src/proof.rs`, `syntax/src/elab.rs` | `(rlt-proof …)` files; written goals elaborated under a pinned convention (NAT-DEC, NAT-LOGIC, NAT-DEF) |
| Registered axioms | `spec/axioms.yaml`, `PINNED_AXIOM_SETS` in `cli/src/lib.rs` | PEANO-ADD, PEANO-ARITH, ROBINSON-Q (Q1–Q7), SHA-256 pinned |
| Theorem library | `lib/`, `rlt deps` | checked proofs with dependency graphs |
| Test infrastructure | `cli/tests/*.rs`, `scripts/verify.sh` | golden reports, accept/reject suites, audits |
| Proof construction | `cli/src/gen.rs` (`rlt gen`), the RLT-006/007 certificate generators | **evaluation only**: closed PEANO-ARITH equations and fixed certificate templates. No search, no unification, no logical goals. |

No proof search existed, so nothing was duplicated. `rlt gen` remains as it was.

## 2. Phase 2 — the search

### Trust boundary

```
goal file (statement, convention, mode, assumptions, axiom set; no proof)
   │
   ▼
rlt prove ── untrusted: cli/src/prove.rs ──► certificate (rlt-proof file on disk)
                                                 │  bytes read back
                                                 ▼
                     certify(): header must equal the goal file's fields exactly
                                                 │
                                                 ▼
                     check_bytes_outcome() — the function `rlt check` uses:
                     re-parse, re-elaborate the written goal, load the pinned
                     axiom set, replay every step in the kernel
                                                 │
                                   ACCEPT ──► PROVED (+ kernel theorem: Γ, axioms used)
                                   otherwise ─► ERROR "certificate refused; nothing is claimed"
search ends without a proof ──────────────────► NOT_FOUND_WITHIN_BOUNDS
```

`Verdict::Proved` is constructed in one place only, from the `Theorem` returned by an accepted replay. The search never constructs kernel `Theorem` values and never calls kernel rules; it predicts each step's conclusion with its own code, and the kernel recomputes all of them.

### Goal files

```lisp
(rlt-goal
  (id P3.congruence)
  (convention NAT-LOGIC)
  (goal "∀a. ∀b. ∀c. a + b = 0 → c = a → c + b = 0")
  (mode unconditional)
  (axioms ROBINSON-Q)
  (bounds (depth 3)))          ; optional; every bound has a default and a ceiling
```

The fields are those of a proof file without `steps`, `params` or `cites` (a goal file carrying any of them is refused). The certificate repeats them verbatim and adds `params` and `steps`. RLT goals are closed formulas, so "from a = b, prove b = a" is stated as `∀a. ∀b. a = b → b = a`; ground facts can instead be declared with `(assume …)` in conditional mode (P5).

### Algorithm

Deterministic, iteratively deepened over `depth = 0 … D`.

1. **Equational core** (any depth). A proof-producing congruence closure: an e-graph whose merges form a proof forest, each edge labelled with its justification (a given proof, or congruence of two applications). It is saturated, round by round, by bounded e-matching of
   - the equational axioms of the set, oriented left to right (Q4–Q7: `x + 0 → x`, `x + S y → S(x + y)`, `x × 0 → 0`, `x × S y → x × y + x`);
   - universally quantified equational facts in the context (both orientations, when the variables allow);
   - Horn rules `∀x̄. l = r → l′ = r′` and `∀x̄. l = r → ⊥`: Q1 (successor injectivity), Q2 (0 is no successor), and negated or implicational equations in the context.
   
   A goal `s = t` is proved when `s` and `t` share a class; ⊥ (and hence any goal, by `false-elim`) when a Horn rule with conclusion ⊥ fires. The proof is read off the forest: a path of edges composed with `trans`, reversed edges with `symm`, congruence edges with `cong` over recursively explained arguments, axiom instances with `inst`, quantified facts with `forall-elim`, Horn rules with `imp-elim`.
2. **Invertible steps** (no depth spent). Goal ∀: a fresh parameter and `forall-intro`. Goal →: a fresh local assumption and `imp-intro`. Goal ∧: both halves. Context ∧: both halves. Context ∃: `exists-elim` with a fresh parameter. Context ⊥: `false-elim`. Goal ∨: left, then right.
3. **Choices** (one unit of depth each). ∃ witnesses (parameters in scope, then ground subterms by size, then numerals up to the witness bound); backward chaining through quantified facts and axioms `∀x̄. A₁ → … → H` whose head matches the goal; case splits on disjunctions in the context (`or-cases`); modus ponens on non-Horn implications; instantiating quantified non-equational facts, and single-variable structural axioms (Q3) at parameters.

Eigenvariable conditions hold by construction: every parameter is fresh when created, and the formulas in Γ at that moment are older. The kernel checks them again.

Certificates are linearised with identical steps shared, and the search's parameter and local-assumption names are renumbered `p1, p2, …` and `h1, h2, …` in order of first appearance, avoiding every other name in the file. Running the same goal twice gives byte-identical certificates (tested).

### Bounds

| Bound | Default | Ceiling | Meaning |
|---|---|---|---|
| `depth` | 3 | 12 | choices on one branch |
| `rounds` | 32 | 1000 | e-matching rounds per saturation |
| `nodes` | 4000 | 200 000 | e-graph nodes per saturation |
| `witnesses` | 12 | 1000 | largest numeral tried as an ∃ witness |
| `budget` | 20 000 | 10 000 000 | search calls over the whole run |

Set in the goal file's `(bounds …)` or on the command line (`--depth N` …); values above a ceiling are refused, not clamped. Every report prints the bounds and whether the budget or node limit was reached.

### Use

```sh
cargo build --release
target/release/rlt prove tests/prover/goals/P3_congruence.goal --out target/P3.rlt   # exit 0 PROVED
target/release/rlt check target/P3.rlt                                              # the kernel, independently
target/release/rlt prove tests/prover/notfound/N1_zero_plus.goal                    # exit 1 NOT_FOUND_WITHIN_BOUNDS
target/release/rlt discover research/prover-discovery                               # Phase 4
target/release/rlt discover --verify research/prover-discovery
```

Without `--out`, the certificate is written to `target/rlt-prove/<id>.rlt`. Exit codes: 0 PROVED, 1 NOT_FOUND_WITHIN_BOUNDS, 2 ERROR.

## 3. Phase 3 — proofs constructed from statements

Measured on the development container (release build; times include loading the axiom set, the search, writing the certificate and the kernel replay).

| Goal | Statement (as checked) | Axiom set | Result | Depth | Steps | Search calls | Largest e-graph | Time |
|---|---|---|---|---|---|---|---|---|
| P1 | ∀a. ∀b. (a = b → b = a) | ROBINSON-Q | PROVED, Γ = ∅, no axioms used | 0 | 5 | 4 | 2 | 16 ms |
| P2 | ∀a. ∀b. ∀c. ((a = b ∧ b = c) → a = c) | ROBINSON-Q | PROVED, Γ = ∅, no axioms used | 0 | 8 | 5 | 3 | 20 ms |
| P3 | ∀a. ∀b. ∀c. (a + b = 0 → (c = a → c + b = 0)) | ROBINSON-Q | PROVED, Γ = ∅, no axioms used (cong on +) | 0 | 10 | 6 | 6 | 20 ms |
| P4 | (2 × 3) + 1 = 7 | ROBINSON-Q | PROVED; Q4–Q7 | 0 | 38 | 1 | 46 | 27 ms |
| P5 | 7 = (2 × 3) + 1, from h1: 2 × 3 = 6, h2: 6 + 1 = 7 | PEANO-ADD | PROVED, Γ = {h1, h2}, no axioms | 0 | 7 | 1 | 13 | 20 ms |
| P6 | ¬(2 + 2 = 5) | ROBINSON-Q | PROVED; Q1, Q2, Q4, Q5 | 0 | 21 | 2 | 18 | 15 ms |
| P7 | ∀x. ¬(x + 1 = 0) | ROBINSON-Q | PROVED; Q2, Q5 | 0 | 8 | 3 | 6 | 20 ms |
| P8 | ∀x. (x = 0 ∨ ∃y. x = y + 1) | ROBINSON-Q | PROVED; Q3 case split, ∃-elim, witness | 3 | 17 | 385 | 22 | 99 ms |
| P9 | ∃k. 8 = 2 × k | ROBINSON-Q | PROVED; witness 4 | 1 | 61 | 7 | 55 | 27 ms |
| P10 | ∀x. x + 2 = (x + 1) + 1 | ROBINSON-Q | PROVED; Q4, Q5 | 0 | 9 | 2 | 13 | 21 ms |
| N1 | ∀x. 0 + x = x | ROBINSON-Q | NOT_FOUND_WITHIN_BOUNDS (needs induction; also at depth 6, 368 calls) | – | – | 16 | 8 | 20 ms |
| N2 | 2 + 2 = 5 | ROBINSON-Q | NOT_FOUND_WITHIN_BOUNDS (false; its negation is P6) | – | – | 6 | 16 | 15 ms |
| N3 | 2 × 3 = 7 (assumption declared, mode unconditional) | PEANO-ADD | NOT_FOUND_WITHIN_BOUNDS (Γ must be ∅) | – | – | 2 | 9 | 15 ms |

Goals 1–4 are the four requested ones. P1–P3 use only the equality rules and congruence, and the kernel reports "axioms used: none"; P4 uses exactly Q4–Q7.

## 4. Phase 4 — theorem discovery

`rlt discover DIR` generates candidates from a fixed grammar over the registered signature — `∀x. L = R` with L one application of `+` or `×` to `x, 0, 1, 2` mentioning `x`, and R an atom or one such application (one orientation per unordered pair) — and runs each through `rlt prove` (ROBINSON-Q; depth ≤ 1, rounds ≤ 32, nodes ≤ 2000, witnesses ≤ 4, calls ≤ 200). A candidate is stored only when its certificate was replayed and accepted. `DIR/manifest.tsv` records, per theorem: id, statement, axiom set, mode, assumptions (∅), certificate path, certificate SHA-256, proof size in steps, search depth, axioms used. `DIR/not-found.tsv` lists every other candidate as NOT_FOUND_WITHIN_BOUNDS. `DIR/summary.txt` states the grammar and bounds. `rlt discover --verify DIR` re-hashes each certificate, replays exactly the hashed bytes with `rlt check`'s function, and binds every manifest column to the replayed proof: id, statement, axiom set, mode, assumptions, proof length and axioms used must equal what the certificate and the kernel's theorem state; the certificate must be `DIR/ID.rlt` (plain components, no symlink); and the current workflow's constraints are enforced (ROBINSON-Q, NAT-LOGIC, unconditional, Γ = ∅, no citations). The search depth is not recorded in a certificate, so it is only checked to be a number within the discovery bound. Duplicate ids, statements or certificate paths, a missing column header and rows of the wrong width are refused. A changed byte is reported STALE. Each certificate's header must be exactly the goal header discovery generates for the candidate its id names (same fields, values and order; one `steps`, at most one `params`): a certificate asserting anything more — a `relationship`, `readingpoint`, `context` or `cites` — is refused, because discovery has no registered, verified relationship to attach (audit F1). A rerun first removes the certificates an earlier run left in DIR (files `ID.rlt` with a discovery id that begin with the `rlt prove` header; nothing else), and `--verify` reports any such certificate that no row lists as UNLISTED (audit D7). Existing files the tool did not write are never overwritten (audit K8). (Audit RLT-PROVER-001, finding D5: before this, `--verify` checked only the statement and hash, so a valid but conditional certificate could be listed as an unconditional theorem.)

Measured results:

| Grammar | Candidates | Proved (kernel-replayed) | Not found | Time |
|---|---|---|---|---|
| one-level (default; committed in `research/prover-discovery/`) | 399 | 9 | 390 | 1.6 s |
| two-level (`--two-level`, R may have two operations) | 3983 | 124 | 3859 | 15.6 s |

Ids name one candidate of one grammar (audit D6): one-level candidates are `D001`…`D399` (the committed ids), two-level candidates `T0001`…`T3983`. `--verify` refuses an id that names a different statement, so `D243` always means `∀x. x × 1 = 0 + x`; the same statement is `T2547` in the two-level grammar.

The nine committed theorems: `∀x. x + 0 = x`, `∀x. x × 0 = 0`, `∀x. x × 0 = 0 + 0`, `∀x. x × 0 = 0 × 0`, `∀x. x × 0 = 0 × 1`, `∀x. x × 0 = 0 × 2`, `∀x. x × 0 = 1 × 0`, `∀x. x × 0 = 2 × 0`, `∀x. x × 1 = 0 + x`. The last is a small illustration of Robinson's Q: RLT proves `x × 1 = 0 + x` but reports `x × 1 = x` and `0 + x = x` as not found — both are true in ℕ and both need induction, which Q lacks. The not-found list mixes such statements with false ones (`∀x. x × 0 = 1`), and the report keeps them in one category on purpose: the search has established nothing about them.

## 5. Phase 5 — adversarial verification

`cli/tests/prover.rs` (30 tests):

| Requirement | Test |
|---|---|
| cannot certify an invalid inference | a changed claim, a wrong axiom instance and a truncated proof are each refused by the gate (`an_invalid_inference_cannot_be_certified`) |
| cannot silently introduce assumptions | a certificate adding `(assume …)` is refused because its header differs from the goal file; a `hyp` of an undeclared name is refused by the kernel; unconditional goals are searched with Γ = ∅ (N3) (`assumptions_cannot_be_introduced_silently`) |
| cannot accept an unregistered axiom | a goal naming an unpinned set is an ERROR before search; a certificate step `inst mul_two` is rejected by `rlt check` as unregistered (`unregistered_axioms_are_refused`) |
| cannot confuse search failure with disproof | N1–N3 report NOT_FOUND_WITHIN_BOUNDS with the explicit no-truth-claim sentence and none of FALSE/DISPROVED/REFUTED; a false equation is "not found", and only its negation, when proved, is a result (`not_found_goals_are_reported_without_a_truth_claim`, `a_false_equation_is_not_found_and_its_negation_is_proved`) |
| cannot bypass kernel replay | a certificate for another goal, an empty proof, and a certificate with citations are refused; PROVED carries the replay's ACCEPT report (`kernel_replay_cannot_be_bypassed`); every committed certificate passes plain `rlt check` and is reproduced byte for byte |
| does not claim arithmetic beyond its axioms | `∀x. 0 + x = x`, `∀x. x × 1 = x` and `∀x. ∀y. x + y = y + x` are not found under ROBINSON-Q, while `∀x. x × 1 = 0 + x` is proved; every PROVED report lists Γ and the axioms used (`arithmetic_claims_stay_within_robinson_q`) |
| bounds are explicit | too little depth or budget gives NOT_FOUND with the exhaustion reported; bounds above a ceiling are an ERROR (`search_bounds_are_enforced_and_reported`) |
| goal files carry no proof | goal files with `steps`, `params` or unknown fields are refused |
| discovery stores only verified theorems | a fresh discovery run reproduces the committed manifest (statements and hashes), stores exactly the nine accepted certificates, and `--verify` detects a tampered one |
| manifest rows cannot misstate a certificate (audit D5) | `d5_*`: a conditional proof listed as unconditional, a changed axiom-set label, a fabricated step count, wrong axioms used, a duplicate row, paths outside the directory or under another id, and changed id/statement/mode/assumptions/depth/columns/header are each refused while the certificate itself still passes `rlt check`; the unmodified copy verifies |

## 6. Limitations

- **Logic and axioms.** The prover proves only what follows from the registered set by the 20 rules. Robinson's Q has no induction, so most general laws of arithmetic (commutativity, `0 + x = x`, `x × 1 = x`) are out of reach under every bound. An induction schema would be a new registered axiom set and a reviewed kernel-trust decision; it is out of scope here.
- **Incompleteness of the search.** First-order provability is undecidable; the search is bounded and incomplete. The equational core orients axioms left to right and matches modulo the current congruence, which decides ground equalities over the given equations but can miss proofs that need an axiom used right to left. ∃ witnesses come from parameters, ground subterms and small numerals only; universal facts with non-equational bodies are instantiated at parameters and ground subterms only; structural axioms (Q3) only at parameters. `subst` is never used (congruence covers the equational cases).
- **Statements.** Goals are closed: free letters are written under ∀. Definitions (NAT-DEF: `<`, `∣`, free235) are accepted as they elaborate, i.e. unfolded; the search has no special knowledge of them.
- **Scale.** Search calls each saturate a fresh e-graph; that is simple and fast for the goals above, but costs grow with the context. Deep case analyses (such as the RLT-007 certificates) remain the job of their generators.
- **Discovery.** The grammar is deliberately small and many stored theorems are routine (axiom instances, evaluations of closed terms). No ranking of interest is attempted; the demonstration is that only kernel-verified statements are stored, with their dependencies and reproducible hashes.
- **Specifications kept separate.** PiπΠ, Spiral N, Triplet N and the CO₂ reading points are untouched. They can become goals for this prover later, as registered specifications, without being forced into Robinson's Q or having their physical interpretations claimed from arithmetic.

# RLT-ROADMAP-001 — Architecture and Mathematical Inventory

**Status.** Research and architecture assessment, 2026-11-22. This document is the only change: the kernel, axioms, certificates, mathematical specifications, website pages and implementation stay exactly as they are at the baseline.

**Revision 2** (2026-11-22): §3.2 and §5.5 separate kernel changes from registered axiom sets, schemas and Lean formalizations (induction needs no new inference rule); §2.3, §2.10, §5.3 and §5.4 record that the RH invariance-and-projection LaTeX paper exists in the author's research materials and awaits import into the source registry.

**Baseline.** `main` at `c9e347b` ("docs: record the RLT-PROVER-002 release audit"), the merge `67cfed2` of RLT-PROVER-002 plus its audit record.

**Objective.** Assess whether RLT's existing architecture can accommodate the original mathematics, the reading-point framework and a future API while keeping the proof kernel intact and keeping refactoring to what the mathematics actually requires.

**Method.**

- Read the RLT code (`kernel/`, `syntax/`, `cli/`), registries, specifications, records and milestone documents at `c9e347b`.
- Built the release binary with the pinned toolchain (rustc 1.97.0) and ran the commands quoted in §1.12 and §2. The full `cargo test --release` result is in §8, "Verification of this document".
- Searched the source repositories for every term in the inventory (§2). Revisions read, all full SHAs:

| Repository | Revision read | Role here |
|---|---|---|
| thinkthoughts/rlt | `c9e347b` | subject of the assessment |
| thinkthoughts/translatepi | `539562ad7c8e9003083da580fd1a6dadf617ef18` (HEAD, 2026-11-21) | PiπΠ conventions, `left4`/`left5`/`lift5`, Spiral N / Triplet N, Pi stages |
| thinkthoughts/news | `1ffd5ea9886386dc0b16c13499ecfe2ed59fd9e9` (the registered pin) | MATH_00, MATH_05, MATH_29 |
| thinkthoughts/math29 | `4cf6e75600e2c5b9b1826e243adbe0877babb5a8` | carbondioxide.now pages |
| thinkthoughts/pi-stage-lab | `111ee1af452f96a96f5394679e5ede852467554d` | 0Pi–17Pi stage notebooks |
| thinkthoughts/math0, math5 | `311dcd1…`, `05ed7e9…` | README texts |
| thinkthoughts/riemann-hypothesis | `f3d8cf3…` (2025-07-17) | earlier RH page |
| antiviolence, residue-manifold-learning, specification-grammar, general-divisor-theorem, formalize, rpi-index, reading-point, reading-point-registry, lean-readingpoint, triplet5, triplet-phase-lock, zeta-constraint-lab, eight-residues, mod30-residue-lanes | HEAD on 2026-11-22 | searched; cited only where a term occurs |
| leanprover-community/mathlib4 | `6ebcce37335bcb6bca64596e5d57bb3035648030` | the exact `RiemannHypothesis` declaration |

Terminology and mathematical expressions are quoted as written in their sources. Where a term has no located definition, this document records that fact and proposes no definition for it.

---

## 1. Current architecture

### 1.1 Components

| Component | Code | Size at `c9e347b` | Trust |
|---|---|---|---|
| Kernel | `kernel/src/{term,prop,rules,check}.rs` | 2,538 lines; zero dependencies; no `unsafe` | trusted |
| Syntax | `syntax/src/{sexpr,proof,written,elab,defs,fdefs,records}.rs` | parsers, conventions, elaboration, definition expansion | untrusted for proof; trusted for *translation fidelity* (every report prints its output) |
| CLI | `cli/src/{lib,main,explain,deps,compose,prove,discover,discover_defs,gen,residues}.rs` | ≈11,600 lines | citation loop, axiom-set loader and `rlt compose` are trusted; search and generators untrusted |
| Lean companions | `lean/*.lean` (25 files), `scripts/lean/DeclCheck.lean` | Lean 4.34.0-rc2 | independent cross-check |
| Registries and records | `sources/`, `conventions/`, `spec/`, `definitions/`, `readingpoints/`, `points/`, `residues/`, `statements/` (65 records), `lib/` (159 entries), `compose/` | pinned by SHA-256 | inputs |

### 1.2 Logic of the kernel

- **One sort, `Nat`.** The signature is fixed in code: `term::arity` (`kernel/src/term.rs:32`) accepts exactly `0`, `S`, `+`, `*`. Equality is the only predicate symbol.
- **Formulas** (`kernel/src/prop.rs:24`): `t = u`, ⊥, ∧, ∨, →, ∀, ∃; ¬φ is φ → ⊥. Locally nameless binders; alpha-equivalent formulas are equal.
- **Sequents** Γ ⊢_A φ with Γ (named assumptions) and A (the full content of every axiom used) carried by every theorem (`rules.rs`; `docs/KERNEL-SOUNDNESS.md` §1).
- **Rules.** 20 inference rules (`RULES`, `kernel/src/check.rs:72`): refl, symm, trans, cong, inst, hyp, subst and 13 logical rules. `cite` returns a separately checked unconditional theorem unchanged and is a dependency, distinct from a rule (`KERNEL-SOUNDNESS.md` §4).
- **Numerals** are unary: the written 7 is S⁷(0). The parser caps each numeral at 1000 (`MAX_NUMERAL`, `syntax/src/proof.rs:55`) and each proof file at 1,000,000 nodes (`MAX_FILE_NODES`, `proof.rs:333`); the kernel bounds formula depth at 256 and nodes at 1,000,000 (`prop.rs:54,58`).
- **Soundness** is argued on paper, rule by rule, with evidence for the implementation (`docs/KERNEL-SOUNDNESS.md`); the semantics is classical first-order validity over all models of the recorded axioms.

### 1.3 Registered axiom sets

Pinned in the binary (`PINNED_AXIOM_SETS`, `cli/src/lib.rs:100`) and mirrored in `spec/axioms.yaml`:

| Id | File | Axioms | Source |
|---|---|---|---|
| PEANO-ADD | `spec/peano.rlt` | add_zero, add_succ | PEANO |
| PEANO-ARITH | `spec/peano-arith.rlt` | Q4–Q7 | OLP-ROBINSON-Q |
| ROBINSON-Q | `spec/robinson-q.rlt` | Q1–Q7 | OLP-ROBINSON-Q at `1e960be` |

Each set is a finite list of closed axioms. Induction is outside every registered set.

### 1.4 Conventions and elaboration

Nine conventions (`conventions/*.yaml`), pinned by an interpretation digest and an explanation digest (`PINNED_CONVENTIONS`, `cli/src/lib.rs:1669`):

| Elaborator | Conventions | Produces |
|---|---|---|
| `nat-dec` | NAT-DEC, NAT-LOGIC, NAT-DEF, NAT-FUN | kernel formulas |
| `species-table` | CHEM-MOLFORMULA | registered species/element readings (T7) |
| `residue-table` | RESIDUE-30 | registered readings of kernel-checked witnesses |
| `symbol` | REAL-PI, PRIME-COUNT | a symbol reading (PI-1 test: `rlt written π REAL-PI π PRIME-COUNT`) |
| `none` | PIPIPI | zero expressions: the PiπΠ source conventions PI-1 … PI-6 are recorded with their RLT bindings |

### 1.5 Definitions: two mechanisms

| Mechanism | File | Identity | Kinds | Used by |
|---|---|---|---|---|
| NAT-DEF (RLT-003 M3b) | `spec/definitions.rlt` | path + SHA-256 **pinned in the binary** (`PINNED_DEFINITIONS`, `cli/src/lib.rs:189`) | predicate abbreviations: `lt` (Q8), `dvd`, `free235` | M3b–M3e, RLT-005/006/007 |
| NAT-FUN (RLT-PROVER-002) | `definitions/double.rlt`, `definitions/quadruple.rlt` | path + SHA-256 **stated in each goal/certificate header**, re-hashed on every check | function abbreviations over {0, S, +, ×}; no constants, cases, recursion or predicates; one file per proof | F1–F7, 126 discovery theorems |

Both expand before the kernel, so neither adds a symbol, axiom or rule. Predicates used by later work that are written out inline in goals rather than registered: `Prime_B(p) :≡ 1 < p ∧ ∀d. ((1 < d ∧ d < p) → ¬(d ∣ p))` and `Adm(p)` (`docs/RLT-007d-CORRESPONDENCE.md`).

### 1.6 Proof search and theorem discovery

`rlt prove` (bounded search: invertible steps, an e-graph equational core orienting Q4–Q7, small witnesses) and `rlt discover` (fixed candidate grammar; `--definitions` for NAT-FUN) are outside the trusted base. A result counts as PROVED only after the kernel replays the written certificate bytes; a miss is NOT_FOUND_WITHIN_BOUNDS (`docs/PROVER.md`). `rlt discover --verify` binds every manifest column to the replayed proof.

### 1.7 Certificate and record formats

| Format | Purpose | Defined in |
|---|---|---|
| `(rlt-proof …)` s-expression | a derivation: id, convention, goal, mode, axioms, optional definitions/cites/assumptions, steps | `syntax/src/proof.rs` |
| `(rlt-goal …)` | input to `rlt prove` | `cli/src/prove.rs` |
| `rlt-statement/1` (YAML) | source excerpt ↔ checked goal ↔ semantic assessment ↔ physical applicability ↔ open questions | `docs/STATEMENT-RECORDS.md`, `cli/src/explain.rs` |
| `rlt-explain/1` (JSON out) | `rlt explain --format json` | `cli/src/explain.rs:1183` |
| `rlt-lib/1`, `lib/identities.tsv` | library entries with statement fingerprints; identities append-only | `cli/src/deps.rs` |
| `rlt-deps/1`, `rlt-deps-lib/1` (JSON out) | dependency graphs | `cli/src/deps.rs:1116` |
| `rlt-compose/1` | layered composition bundle ("discharged by composition") | `cli/src/compose.rs`, `docs/RLT-007d-FINAL.md` |
| discovery `manifest.tsv` / `not-found.tsv` / `summary.txt` | discovery records, byte-exact | `docs/PROVER.md` |

Exit codes are shared across `explain`, `deps` and `compose`: 0 current and accepted, 1 refused, 2 invalid or malformed, 3 stale (`cli/src/main.rs`, usage text).

### 1.8 Lean correspondence machinery

- Per-milestone Lean companions restate accepted statements (`lean/*.lean`; `rlt lean` exports unfolded kernel statements).
- For NAT-FUN, `scripts/check-definitions-lean.py` builds an expected module from RLT's printed statements and compares Lean's *elaborated* declarations as kernel terms (`scripts/lean/DeclCheck.lean`), after `leanchecker` replays every constant (D6, H1; `docs/RLT-PROVER-002-AUDIT.md`).
- Lean checks the same mathematics in another system; it checks RLT's statements, and RLT's derivations stay RLT's own.

### 1.9 Source identity registry

`sources/registry.yaml` lists every source with location, status (`available`, `archived`, `transcribed`, `conversation`, `reference`, `ambiguous`, `missing`) and verbatim excerpts tagged with the reading point they define. Its documentation content is pinned in the binary (`PINNED_REGISTRY_SHA`, `cli/src/lib.rs:1585`). `scripts/check_excerpts.py` re-checks every excerpt at its commit; abbreviated SHAs resolve through `sources/commit-pins.tsv`; PDFs without TeX source enter as reviewed transcriptions (`sources/transcriptions/`).

### 1.10 Trust boundaries

| Boundary | Inside | Outside, made inspectable |
|---|---|---|
| Kernel | rules, Γ and A bookkeeping, term/formula operations, goal comparison, cited-theorem re-validation | — |
| Axiom-set loader | pinned bytes → parsed axioms | every report prints each axiom in full |
| Citation loop | path rules, cycle refusal, hash, re-check from bytes | report prints path, hash, proved formula |
| `rlt compose` | re-checks every node, compares canonical closed formulas, enforces one discharge per assumption from a lower layer | Python re-implementation `scripts/compose-check.py` |
| Elaboration and definition expansion | — (translation layer) | written goal, folded goal, checked goal and expanded definitions printed |
| Search, generators | — | kernel replay is the sole authority |
| Source fidelity, semantic and physical correspondence | — | statement records, `rlt explain`, human assessments with basis documents pinned by SHA |

### 1.11 Extension points

| Extension point | Mechanism | Kernel touched? | Precedent |
|---|---|---|---|
| New axiom set | file + entry in `PINNED_AXIOM_SETS` and `spec/axioms.yaml` | no (reviewed like kernel code) | PEANO-ARITH (M1), ROBINSON-Q (M3b) |
| New function definitions | new `definitions/*.rlt` under NAT-FUN | no | DOUBLE, QUADRUPLE |
| New pinned predicate abbreviations | edit `spec/definitions.rlt` + `PINNED_DEFINITIONS` | no | `lt`, `dvd`, `free235` |
| New convention | `conventions/*.yaml` + `PINNED_CONVENTIONS` | no | CHEM-MOLFORMULA (T7), RESIDUE-30 (M2) |
| New source | `sources/registry.yaml` + pinned registry SHA | no | MCNEW-POMERANCE, AXIOM-BGP212 |
| New reading point / specification record | `readingpoints/`, `points/`, `statements/` | no | T6, T7, RLT-004b |
| New library theorem | `lib/` + append to `identities.tsv` | no | 159 entries |
| New composition result | `compose/` bundle + approved results/specifications | no | H₄₅ Lemma 12.1 |
| New function symbol | `term::arity` | **yes** | `*` in M1 |
| New logical rule | `RULES` and `rules.rs` | **yes** | 13 rules in M3a |

### 1.12 Architectural limitations, with evidence

| # | Limitation | Evidence |
|---|---|---|
| L1 | Fixed single-sorted signature {0, S, +, ×}; integers, rationals, reals and complex numbers have no representation | `term.rs:32`; `docs/RLT-007-HANDOFF.md` §6 |
| L2 | Unary numerals with a 1000 cap on every numeral, intermediate values included | `rlt gen "24 × 41 = 959 + 25"` → ACCEPT; `rlt gen "24 × 42 = 983 + 25"` → "24 × 42 exceeds the numeral limit 1000"; `docs/RLT-004a-BASELINE.md` |
| L3 | Induction is outside every axiom set, so universally quantified statements over all n that need induction stay out of reach | `statements/MOD30-CLASSIFICATION.yaml` open questions; `docs/RLT-007b-2-DESIGN.md` (Q proves Prime_B facts only with the guarded definition) |
| L4 | Definitions are abbreviations only: no recursion, cases, constants or predicates under NAT-FUN; one definitions file per proof | `docs/RLT-PROVER-002.md` §7 |
| L5 (**D2**) | NAT-FUN definition identity is path + SHA-256; registry ids and function names are unregistered, so another file may also declare `(id DOUBLE)` | `docs/RLT-PROVER-002-AUDIT.md` §4 |
| L6 (**D3**) | `unfolding`/`inference` classification is syntactic; 39 of 126 discovery theorems hold for any term in place of `double` | `docs/RLT-PROVER-002.md` §9 |
| L7 | Discovery `depth` column is checked only against the bound | `docs/RLT-PROVER-002.md` §9 |
| L8 | Lift status `verified` is refused unconditionally, with the RLT-001 message, though multiplication now lets both registered lifts' relations be kernel theorems | `cli/src/lib.rs:2242`; `readingpoints/math0.yaml`, `math5.yaml` |
| L9 | Under the earlier conventions `rlt check` echoes `(relationship …)`/`(readingpoint …)` labels without a registration check (`rlt rp`/`rlt validate` enforce it) | `docs/RLT-PROVER-002.md` §6, Finding 1 |
| L10 | A composed result is a bundle verdict, never a single kernel theorem; composed results are uncitable as library entries | `docs/RLT-007-HANDOFF.md` §6 |
| L11 | Lean correspondence requires a Lean installation; otherwise UNVERIFIED (exit 3) | `docs/RLT-PROVER-002-AUDIT.md` §4 |
| L12 | `README.md` status line ends at "RLT-003 M3d … under review"; later milestones are recorded in `docs/` and `index.html` | `README.md` |
| L13 | Several current source documents for the original mathematics sit outside the registry: thinkthoughts/math0, math5, math29 (carbondioxide.now), pi-stage-lab, translatepi `sites/translatepi.ai/RH/` | `sources/registry.yaml` (§2 below) |

---

## 2. Inventory of the original mathematics

Status vocabulary used below:

- **Kernel theorem** — an accepted RLT proof file, with axiom set and mode.
- **Registered reading** — a source excerpt, convention or record that RLT validates as an interpretation input.
- **Located, unregistered** — a source document exists; RLT holds no record of it.
- **Unlocated** — no occurrence in any repository read (§0); the expression awaits its source or exact specification.

### 2.1 PiπΠ — expand imagination, extend care, resist unsupported claims

| Aspect | Finding |
|---|---|
| Registered in RLT | `conventions/pi.yaml` (PIPIPI, elaborator `none`): PI-1 … PI-6 verbatim from TRANSLATEPI-261018, TRANSLATEPI-LEAN4, TRANSLATEPI-INDEX, ANTIVIOLENCE-FEED-CONCEIVABLE, each with an RLT binding (≡w vs term identity; lifts carried by relations; Γ retained; lift statuses; leading → trailing; `imagined`). |
| Recorded as unresolved | "A formal meaning for each glyph of the triple Pi / π / Π." and "Whether a lift is a function, a relation or a proof term." (`conventions/pi.yaml`; `docs/RLT-000.md` item 5) |
| The triad as written here | **Unlocated** in that exact wording. Nearest located forms: "Expand imaginations. Extend care. Resist illiteracy." (math29 `carbondioxide/index.html` and `carbondioxide/RH/index.html`, 2026-11-16 and 2026-11-21); "Translating the mathematical word 'Pi' expands imaginations to extend and resist additional integer points (invitations)" (translatepi `sites/translatepi.ai/261001_index.html`); triplet positions `expand` / `extend` / `resist` in pi-stage-lab notebooks. |
| Machine-checked | Kernel theorems realise PI-3 (Γ retained: T5a vs T5b, T5c vs T5d) and PI-6 (`imagined` kept apart: `math0.emoji-breathing`). These are theorems and validations *about RLT's bookkeeping*, distinct from a formal meaning of the triad. |
| Formal status | Interpretive convention. Ready to register the exact phrase once its source is pinned; a mathematical definition awaits the author's specification. |

### 2.2 Piπ / πΠ — imagination and observation as alternating reading points

| Aspect | Finding |
|---|---|
| Sources | **Unlocated** as `Piπ` or `πΠ` standing alone (searched with the regular expressions `(?<!Pi)Piπ(?!Π)` and `(?<!Pi)πΠ`). Every occurrence found is the full `PiπΠ`. |
| Related registered material | math0 meanings "local imagination", "MATH0 · FICTION", "Imagine"; math5 "Identify", "Count objects. Specify people, properties, and context."; math29 "Investigate"; `correspondence/status.yaml` keeps `imagined` apart from `supported`. |
| Formal status | Conceptual proposal. The alternation (which reading point follows which, and what changes at each step) needs an exact written specification before any record or predicate can carry it. |

### 2.3 `#left`, `left5`, `lift5`, and local `+1π+1Π` correspondence

| Expression | Finding |
|---|---|
| `#left` | **Unlocated.** Nearest located forms: `(Left + 1) + 1 ≠ ±1(9424π) = (Triplet Now = (Spiral Now × 24) − 25)` (translatepi `indexOLD.html`); "Resisted by left+1" (`fridaysforfuture.html`); "by turning left at stop signals" (riemann-hypothesis `index.html`). |
| `left4`, `left5`, `lift5` | **Located, unregistered**: translatepi `sites/translatepi.ai/RH/index.html` at `539562a`: "left4(s) := Re(s)"; "left5(Z) … {s ∈ Z : Re(s) = ½}"; "lift5(s) … Re(s) = ½"; with Z = { s ∈ ℂ : ζ(s) = 0 and 0 < Re(s) < 1 } and the projection form Z = left5(Z). These are definitions over ℂ (see §2.10). |
| `left5` in other senses | No other definition located; in particular, no arithmetic `left5` over ℕ. |
| `+1π+1Π` | **Unlocated** (searched with `\+1\s*π\s*\+1\s*Π`). Related registered material: the "+1 reading level" meanings of math0/math5/math29 (NEWS-MATH00/05/29). |
| Formal status | `left4`/`left5`/`lift5`: exact definitions exist on the translatepi.ai/RH page. The original paper, *A Structural Reformulation of the Riemann Hypothesis via Invariance and Projection*, exists as a LaTeX source in the author's research materials (confirmed by the author, 2026-11-22); it sits outside the repositories read here and is not yet registered or pinned in `sources/registry.yaml`. Until the paper is imported, translatepi `sites/translatepi.ai/RH/index.html` at `539562a` serves as the current repository source; once imported, the paper (pinned by commit and SHA-256, with `left4`/`left5`/`lift5` as verbatim excerpts) becomes the leading source and the page a secondary one. `#left` and `+1π+1Π`: await specification. |

### 2.4 (1+1=2), (29+1=30), and ((1+1)×3×5=30)

| Expression | Status |
|---|---|
| `1 + 1 = 2` | **Kernel theorem**, unconditional, PEANO-ADD: `tests/accept/T4_math0_pairing.rlt` (relationship `math0.pairing`) and `T6_one_plus_one.rlt`. |
| `29 + 1 = 30` | **Kernel theorem**, unconditional, PEANO-ADD: `tests/accept/R_math29_pairing.rlt` (relationship `math29.pairing`); library `arith.add.29_1_30`. |
| `5 + 1 = 6`, `1 + 1 + 1 + 1 + 1 = 5` | **Kernel theorems**: `T1_math5_pairing.rlt`, `R_math5_count.rlt`. |
| `30 = 2 × 3 × 5`, `6 × 5 = 30`, `2 × 3 = 6` | **Kernel theorems**, PEANO-ARITH: `tests/accept/M1_thirty.rlt` (also the RESIDUE-30 factorization), `M1_six_five.rlt`, `M1_two_three.rlt`. |
| `(1 + 1) × 3 × 5 = 30` as written | **Checkable now, unrecorded.** NAT-DEC parses it; `rlt gen "(1 + 1) × 3 × 5 = 30"` writes a PEANO-ARITH derivation that the kernel ACCEPTs (run for this document; no file kept). Source: NEWS-MATH29 excerpt "(1+1) × 3 × 5 = 30 > 29". |
| Lifts `math0-to-math5` via "(1+1) × 3 = 6" and `math5-to-math29` via "6 × 5 = 30" | **Registered, `proposed`.** Both relations are now provable; `rlt validate` refuses `verified` unconditionally (L8). |
| `1 + 1 + 1 + 1 + 1 = 0` (math0 fiction) | **Conditional kernel theorem** (T5a); the unconditional form is rejected (T5b). |

### 2.5 Modulo-30 admissible residues

| Aspect | Status |
|---|---|
| Witnesses | **Kernel theorems + registered readings**: 111 closed quotient–remainder witnesses for r = 0 … 29 and p ∈ {2, 3, 5} (`residues/residue-30.yaml`, `tests/m2/`), read under RESIDUE-30. |
| Classification below 30 | **Kernel theorem**, unconditional, ROBINSON-Q: `tests/m3e/accept/E_T1.rlt`, library `mod30.classification`: ∀r. r < 30 → (free235(r) ↔ r ∈ {1, 7, 11, 13, 17, 19, 23, 29}), written as two implications. |
| Source fidelity | `statements/MOD30-CLASSIFICATION.yaml`: semantic status `partial`. Differs: the source speaks of every prime > 5 and residue classes modulo 30; the theorem speaks of the 30 numbers below 30. Lean: `free235_iff_coprime`, `classification`. |
| Open | "Every n: n mod 30 lies in the eight classes exactly when gcd(n, 30) = 1" and "Every prime above 5" — both need induction (division with remainder for all n) or a stronger theory (L3). "Admissible" in the H₄₅ sense (`Adm(p)`) is a separate notion, proved for H₄₅ by composition. |

### 2.6 math0, math5, math29

| Aspect | Status |
|---|---|
| Records | `readingpoints/math0.yaml`, `math5.yaml`, `math29.yaml`: every meaning a whole verbatim unit of a registered excerpt (USER-2026-11-07, NEWS-MATH00/05/29, RLT-INIT, RLT-INDEX, REPAIREARTH); completeness checked by `rlt validate` and `scripts/check_excerpts.py`. |
| Relationships | Kernel theorems listed in §2.4, recomputed on every `rlt rp`. |
| Correspondence | All `unestablished`, except `math0.emoji` (`inapplicable`) and `math0.emoji-breathing` (`imagined`). |
| Source divergence | thinkthoughts/math0 README ("Math0 ('math zero') identifies the reading level of patriarchal gibberish.") and thinkthoughts/math5 README (an O₂ + CO₂ atom count) are unregistered sources whose wording differs from the registered meanings. Registry completeness is complete relative to the registry, by design; whether these READMEs should become sources is the author's decision. |

### 2.7 `0Pi+` stages

| Aspect | Finding |
|---|---|
| `0Pi+` | **Unlocated** as written. |
| Located stage material | pi-stage-lab (`111ee1a`, 2026-04-27): notebooks `00_0Pi_identity_expand` … `17_17Pi_translation_resist`, six tracks (identity, physical, cycle, system, rate, translation) × triplet positions (expand, extend, resist); "0Pi: define measurable identity", "1Pi: test identity through interaction", "2Pi: preserve identity across constraint"; "0Pi is grammar, not application." translatepi `pi-table.html`: "0Pi · Imagination (Initialization) · 1 + 1 = Thinking", through "9425Pi · Triplet equation shift · (Spiral N × 24) - 25". |
| Formal status | A stage taxonomy with prose rules and claim scores (`cgcs`); the stages have no arithmetic definition. Candidate RLT treatment: a registered interpretive convention or record (like PIPIPI), with each stage's rule as a verbatim excerpt. Whether `0Pi+` names this sequence, a successor operation on stages, or something else needs the author's specification. |

### 2.8 Triplet N and Spiral N

| Aspect | Finding |
|---|---|
| Located definition | "Triplet N = (Spiral N × 24) − 25" (translatepi `src/integerity_matrix.json`, `261001_index.html`, `msf.html`, `spirals_1_99.html`, `qubit.html`; pi-stage-lab `00_0Pi_identity_expand.ipynb`); `triplet_n(spiral_n) = (spiral_n * 24) - 25` and `spiral_n_from_triplet(t) = (t + 25) // 24` (translatepi `src/triplet_generator.py`). |
| Domain | Spiral N = 1 gives Triplet N = −1 (`qubit.html` row Q1: Spiral 1, Triplet −1). The defining expression therefore lives in ℤ; over ℕ it is representable as the relation `t + 25 = 24 × s` for s ≥ 2. |
| Internal consistency (computed for this document) | Every value of 24s − 25 is ≡ 23 (mod 24). The labels keyed 9423, 9424, 9425, 9426 in `triplet_generator.label_triplet` are ≡ 15, 16, 17, 18 (mod 24), so `triplet_n` produces none of them; `spiral_n_from_triplet` is an inverse only on values ≡ 23 (mod 24). In `qubit.html` the Triplet column (−1, 23, 47, 71, …) follows the formula and the `9423Pi`… column is a separate stage label. The relation between the two columns needs the author's statement. |
| RLT reach today | Under PEANO-ARITH: `24 × 2 = 23 + 25` ACCEPT; `24 × 41 = 959 + 25` ACCEPT; Spiral N ≥ 42 exceeds the numeral cap (L2). As a NAT-FUN function, `triplet(s) := 24 × s − 25` is unavailable (no subtraction); a predicate definition `triplet_of(s, t) :≡ t + 25 = 24 × s` becomes available with registered predicate definitions. |
| Formal status | Exact formula located; domain and the role of the 9423–9426 labels need clarification before registration. |

### 2.9 Carbon dioxide and physical correspondence

| Aspect | Status |
|---|---|
| Notation | CHEM-MOLFORMULA (`conventions/chem-molformula.yaml`): `rlt chem "CO₂" CHEM-MOLFORMULA`; COO, CO2, O2 stay uninterpreted (T7). |
| Counts | `points/co2-and-o2.yaml`: seven reading points (two molecules, five atoms); kernel-checked counts 1 + 2 = 3, 1 + 1 = 2, 1 + 1 + 1 + 1 + 1 = 5; every physical status `unestablished` with `evidence: []`. |
| Sources | NEWS-MATH05 example (registered); CO2-SPECIFICATIONS registered as `missing`; carbondioxide.now ("CO₂ is exhaled." as a formalized physical reading point; math29 `carbondioxide/index.html`) located, unregistered. |
| Requirement | Evidence records (measurement source, method, quantity, units, uncertainty) for the physical layer. This is a records/schema extension; the kernel's role stays the counts. |

### 2.10 RH and the proposed `left5`–Mathlib equivalence

| Aspect | Finding |
|---|---|
| Source status | The original invariance-and-projection paper exists as LaTeX in the author's research materials and is unregistered and unpinned in the RLT source registry. Current repository source: translatepi.ai/RH (`sites/translatepi.ai/RH/index.html` at `539562a`), until the paper is imported. |
| Our formulation (located, unregistered) | translatepi `RH/index.html` at `539562a`: Z = { s ∈ ℂ : ζ(s) = 0 and 0 < Re(s) < 1 }; left4(s) := Re(s); left5(Z) = {s ∈ Z : Re(s) = ½}; lift5(s): Re(s) = ½; projection form Z = left5(Z); stated status "Mathematical equivalence established; formal correspondence planned". |
| Lean target | Mathlib `Mathlib/NumberTheory/LSeries/RiemannZeta.lean:186` at `6ebcce3`: `def RiemannHypothesis : Prop := ∀ (s : ℂ) (_ : riemannZeta s = 0) (_ : ¬∃ n : ℕ, s = -2 * (n + 1)) (_ : s ≠ 1), s.re = 1 / 2`. |
| External result | The Lean community's joint statement of 2026-11-21 certifies Levent Alpöge's Lean proof of `RiemannHypothesis` (leanprover-community blog, "Lean verification of the Riemann Hypothesis"). `RH/index.html` in this repository records it as external; RLT has replayed none of it. |
| Precise correspondence obligations | (a) Z = left5(Z) ↔ ∀s ∈ Z, Re(s) = ½ follows from set extensionality, since left5(Z) ⊆ Z by definition — a definitional step. (b) The strip form ↔ Mathlib's `RiemannHypothesis` needs more than unfolding: Mathlib quantifies over every zero outside the trivial zeros and s = 1, so the equivalence also uses that ζ has no zeros with Re(s) ≥ 1 (Mathlib: `riemannZeta_ne_zero_of_one_le_re`, `Mathlib/NumberTheory/LSeries/Nonvanishing.lean`) and that every zero with Re(s) ≤ 0 is a trivial zero (via the functional equation `riemannZeta_one_sub`; a packaged Mathlib lemma for this direction was not located in this survey). The Lean companion therefore needs these facts as cited theorems, and the "equivalence by its definitions" wording on the page applies to step (a). |
| RLT reach | ℂ, ζ and analysis are outside the kernel's logic (L1). RLT's role is the correspondence record: pinned written source, exact Lean declaration, companion theorem name, dependencies, checker output — the `rlt explain` layers, with the kernel layer marked not-applicable. |

### 2.11 Summary

| Item | Machine-checked in RLT | Registered reading | Located, unregistered | Needs exact specification |
|---|---|---|---|---|
| PiπΠ (triad) | Γ retention and `imagined` bookkeeping | PI-1 … PI-6 | "Expand imaginations. Extend care. Resist illiteracy." | glyph meanings; the triad's exact wording |
| Piπ / πΠ | — | related math0/5/29 meanings | — | all |
| `#left`, `+1π+1Π` | — | "+1 reading level" meanings | nearest: `(Left + 1) + 1 ≠ ±1(9424π)` | all |
| `left4`/`left5`/`lift5` | — | — | RH page definitions; LaTeX paper in research materials | import and pin the paper |
| 1+1=2, 29+1=30 | yes | yes | — | — |
| (1+1)×3×5=30 | checkable now | excerpt | — | record + lift promotion |
| Modulo 30 | r < 30 classification | RESIDUE-30 | — | all-n form needs induction |
| math0/5/29 | relationships | meanings | READMEs | — |
| `0Pi+` stages | — | — | pi-stage-lab, pi-table | meaning of `+` |
| Triplet N / Spiral N | instances with s ≤ 41 | — | formula and tables | domain; 9423–9426 labels |
| CO₂ | atom counts | species, points | carbondioxide.now | evidence schema |
| RH / `left5`–Mathlib | — (outside logic) | external page | RH pages; LaTeX paper in research materials | companion theorem; import and pin the paper |

---

## 3. Mathematical requirements

### 3.1 Route per workstream

Columns: **R** existing kernel rules; **Q** existing Robinson's Q axioms; **F** registered function definitions; **P** future registered predicate definitions; **S** an explicitly registered stronger arithmetic theory; **L** Lean or another external theory.

| Workstream | R | Q | F | P | S | L | Notes |
|---|---|---|---|---|---|---|---|
| Pairings 1+1=2, 5+1=6, 29+1=30 | ✓ | ✓ (PEANO-ADD ⊂ Q) | | | | ✓ replay | done |
| (1+1)×3×5=30; lifts' relations | ✓ | ✓ | | | | ✓ | record + lift rule change only |
| Modulo 30, r < 30 | ✓ | ✓ | | ✓ to register `free235`-style predicates uniformly | | ✓ | done under NAT-DEF |
| Modulo 30, every n; every prime > 5 | ✓ | | | ✓ | **✓ PA-style induction** | ✓ (GDT) | Q is too weak (L3) |
| Prime_B, Adm, admissible residues | ✓ | ✓ | | **✓** | | ✓ | currently inline in goals |
| Triplet N / Spiral N, s ≤ 41 | ✓ | ✓ | (subtraction-free form only) | **✓** relation `t + 25 = 24 × s` | | ✓ | needs domain decision |
| Triplet N / Spiral N, large s; 9423-range values | ✓ | ✓ | | ✓ | | ✓ | needs a numeral-representation change (§3.2 K1) |
| Triplet N with Spiral N = 1 (−1) | | | | | **✓ integers** | ✓ | or restrict domain to s ≥ 2 |
| PiπΠ, Piπ/πΠ, `0Pi+` stages | | | | | | | interpretive conventions and records; mathematics awaits specification |
| `#left`, `+1π+1Π` | | | | | | | awaits specification |
| CO₂ | ✓ counts | ✓ | | | | | physical evidence records |
| RH / `left5`–Mathlib | | | | | | **✓ only** | RLT records correspondence |

### 3.2 Kernel changes, registered theories and external formalizations

A mathematical extension reaches RLT by one of four routes. Only route 1 changes the kernel; routes 2 and 3 change what the kernel is *given*, and route 4 keeps the mathematics in Lean with RLT recording the correspondence.

| Route | What changes | Trust impact | Precedent |
|---|---|---|---|
| 1 · Kernel change | `kernel/src/`: the signature (`term::arity`), the term representation, or `RULES` | soundness argument re-done for the changed part (`docs/KERNEL-SOUNDNESS.md`) | `*` in M1; 13 logical rules in M3a |
| 2 · Registered axiom set (finite) | a new `spec/*.rlt` file, its entry in `PINNED_AXIOM_SETS` and `spec/axioms.yaml` | the axioms are trusted premises, printed with every result; the kernel is unchanged | PEANO-ARITH, ROBINSON-Q |
| 3 · Registered axiom schema | the axiom-set loader generates or checks schema instances per use; each instance reaches the kernel as an ordinary axiom | the loader's instance construction joins the trusted base (as the loader already is, `KERNEL-SOUNDNESS.md` §5); the kernel and its rules are unchanged | none yet |
| 4 · External formalization | a Lean companion plus an RLT correspondence record | Lean's kernel for the mathematics; RLT for source identity and correspondence | GDT comparison; RLT-005/006/007 Lean files |

**Induction needs no new inference rule.** Each instance of the first-order induction schema,

  ∀ȳ. (φ(0, ȳ) ∧ ∀x. (φ(x, ȳ) → φ(S x, ȳ))) → ∀x. φ(x, ȳ),

is a closed first-order formula over {0, S, +, ×}. The kernel already accepts axioms with quantifiers and connectives (Q1 uses →, Q2 ¬, Q3 ∨ and ∃; `spec/robinson-q.rlt`), instantiates them with `inst` and applies them with `imp-elim`, `forall-elim` and the other existing rules, and records the full content of every axiom used in A. So induction can enter by route 2 (a reviewed, finite list of the instances a proof needs, pinned like ROBINSON-Q) or by route 3 (a schema whose instances the loader builds from a stated motive φ and checks syntactically), with the 20 rules unchanged. The one thing an ordinary axiom cannot express is the schema itself, because axiom variables range over terms and a schema ranges over formulas φ; that is why route 3 is a loader change and route 2 lists instances. An induction *rule* in the kernel (route 1) is a third option, attractive only if per-use instance generation proves impractical.

Classification of the extensions the located mathematics needs:

| # | Extension | Required by | Route(s) available | Kernel change required? |
|---|---|---|---|---|
| K1 | Numerals beyond unary-with-cap | Triplet N for Spiral N ≥ 42; 9423-range values; large residue certificates (L2; `docs/RLT-004a-BASELINE.md`) | 1 (binary/decimal term representation with kernel-checked arithmetic); partial relief by raising `MAX_NUMERAL`, a parser constant, within the 1,000,000-node budget (a few thousand at most); 4 for the large cases | **Yes**, if RLT itself must check large values efficiently: the representation lives in `term.rs` and the equational rules. |
| K2 | Induction | modulo 30 for every n; every prime > 5; division with remainder (L3; `MOD30-CLASSIFICATION` open questions; `lean/RLT007S2Design.lean`, `model_of_Q`) | 2 (finite instance set), 3 (schema in the loader), 4 (GDT in Lean); 1 optional | **No.** Routes 2 and 3 keep the kernel and its rules as they are. |
| K3 | Integers (a second sort) | Triplet N = −1 at Spiral N = 1 (L1) | domain restriction to s ≥ 2; pairs of naturals through predicate definitions; 4 | **No** for the located mathematics; route 1 only if ℤ must be a sort. |
| K4 | ℂ, ζ, analysis | RH / `left5`–Mathlib (§2.10) | 4 | **No**; the kernel's first-order arithmetic is the wrong home for it. |

Not kernel requirements (they change elaboration, notation, registries or search only):

| Change | Layer |
|---|---|
| Registered predicate definitions with pinned identity (D2) | syntax + CLI registry |
| Generalisation check for definition sensitivity (D3) | CLI/search |
| Lift promotion to `verified` when every relation is a kernel theorem (L8) | `rlt validate` |
| Registration check for `relationship`/`readingpoint` labels in `rlt check` (L9) | CLI; changes established verdicts, so needs its own review |
| Interpretive conventions for PiπΠ triad, Pi stages | `conventions/`, registry |
| Evidence records for physical correspondence | records schema, `rlt explain` |
| RH correspondence record | statement record with kernel layer `not-applicable` |
| JSON claim/audit interface (RLT-AGENT-001) | CLI |

---

## 4. RLT-AGENT-001 — claim submission and audit result

### 4.1 Goals

An agent (or person) submits one claim; RLT returns one audit result that keeps apart the questions RLT already keeps apart: proof validity, source fidelity, semantic correspondence and physical correspondence, plus every outstanding obligation. The interface reuses existing checkers; it adds no inference.

### 4.2 Design principles

1. **Untrusted input, existing authority.** A submission is data. Verdicts come only from the existing functions behind `rlt check`, `rlt explain`, `rlt deps`, `rlt compose` and `check_excerpts.py`.
2. **Exact identity.** Every referenced file is named by repository-relative path *and* SHA-256; every source by registry id *and* commit; every axiom set by id *and* pinned SHA; definitions by path + SHA now, by registry id + SHA once D2 is resolved.
3. **No status promotion.** The audit reports semantic and physical status exactly as recorded by a human assessment pinned in a statement record; a submission can propose an assessment, and the result labels it `proposed`.
4. **Deterministic, versioned output.** Canonical JSON (sorted keys, UTF-8, `\n`), the format id in every document, a `result_sha256` over the canonical serialization, and the RLT commit and binary hash in the result.
5. **Read-only.** The command writes only its result (to stdout or an untracked path) and refuses tracked paths, as `rlt prove --out` already does.

### 4.3 Submission: `rlt-claim/1`

```json
{
  "format": "rlt-claim/1",
  "id": "CLAIM-MATH29-PAIRING",
  "submitted_by": {"kind": "agent", "name": "…", "note": "free text, echoed only"},
  "source": {
    "registry": "NEWS-MATH29",
    "commit": "1ffd5ea9886386dc0b16c13499ecfe2ed59fd9e9",
    "excerpts": ["For instance (nonfiction): (1+1) × 3 × 5 = 30 > 29, where every prime > 5 resists division by 2, 3, and 5 and corresponds to one of eight residue classes modulo 30:"]
  },
  "statement": {
    "written": "(1 + 1) × 3 × 5 = 30",
    "convention": "NAT-DEC",
    "definitions": null,
    "assumptions": [],
    "mode": "unconditional"
  },
  "axiom_set": {"id": "PEANO-ARITH", "sha256": "00ceffa9a60b53286f350edbf2f1a5f48a14169a9b09a82cbedc6f9956ff01e4"},
  "certificate": {"path": "tests/…/X.rlt", "sha256": "…"},
  "reading_points": [{"id": "math29", "role": "historical identifier"}],
  "correspondence": {
    "semantic": {"proposed_status": "partial", "matches": ["…"], "differs": ["…"]},
    "lean": [{"file": "lean/….lean", "theorem": "…"}],
    "physical": {"applicability": "not-applicable", "reason": "…"}
  },
  "requested": ["kernel", "source", "lean", "deps"]
}
```

Rules: unknown keys, duplicate keys, absolute paths, `..`, symlinks and control characters are refused (the `XR*` cases of `tests/explain/` give the precedent). `certificate` may alternatively be `{"inline": "<rlt-proof bytes>", "sha256": "…"}`; the bytes are written to a content-addressed scratch directory outside the tracked tree and checked from exactly those bytes. A claim may name a `statement_record` or a `compose` bundle instead of a certificate.

### 4.4 Result: `rlt-audit/1`

```json
{
  "format": "rlt-audit/1",
  "claim": {"id": "…", "sha256": "<canonical claim>"},
  "rlt": {"commit": "c9e347b…", "binary_sha256": "…", "registry_sha256": "…", "axiom_sets": {"PEANO-ARITH": "00ce…"}},
  "kernel": {
    "verdict": "ACCEPT | REJECT | MALFORMED | NOT_REQUESTED",
    "verdict_line": "ACCEPT — unconditional theorem relative to registered axiom set PEANO-ARITH",
    "written_goal": "…", "checked_goal": "…", "assumptions": "Γ = ∅",
    "axioms_used": ["…"], "rules_used": ["…"], "cites": [{"path": "…", "sha256": "…", "proved": "…"}],
    "definitions": {"path": "…", "sha256": "…", "used": ["…"]}
  },
  "source": {"registry": "NEWS-MATH29", "commit": "…", "excerpts": [{"text": "…", "verbatim": true}], "status": "available"},
  "semantic": {"recorded_status": "partial | unassessed | …", "proposed_status": "partial", "basis": {"path": "…", "sha256": "…"}},
  "lean": {"status": "PASS | FAIL | UNVERIFIED", "theorems": ["…"]},
  "physical": {"applicability": "not-applicable", "status": "…", "evidence": []},
  "obligations": [
    {"kind": "semantic-assessment", "detail": "no pinned human assessment for this claim"},
    {"kind": "source-scope", "detail": "source speaks of every prime > 5; checked goal is a closed equation"}
  ],
  "exit": 0,
  "result_sha256": "…"
}
```

Exit codes follow `explain`/`deps`/`compose`: 0 accepted with current pins, 1 refused, 2 invalid or malformed, 3 stale. `obligations` lists every open item: missing assessment, `partial` differences, Lean UNVERIFIED, physical `unestablished`, open questions from the statement record, D2-style identity gaps.

### 4.5 Is a local JSON CLI the right first implementation?

Yes, for these reasons:

- **Every authority it needs already exists as a library call** (`check_file_outcome`, `explain`, `deps`, `load_bundle`); the CLI is an adapter, sized like `explain.rs`.
- **JSON output already exists** (`rlt-explain/1`, `rlt-deps/1`), written by `explain::json_write`; the audit result reuses it.
- **Input parsing needs no new dependency.** `yaml-rust2` (already a dependency) reads JSON as YAML 1.2; a strict pre-check refusing YAML-only syntax (anchors, tags, flow keys without quotes) and duplicate keys keeps the accepted language exactly JSON. A small hand-written JSON reader (outside the trusted base) is the alternative if strictness by construction is preferred.
- **A local process keeps the trust story identical** to `rlt check`: same binary, same pinned registries, same reproduction command. A network service adds authentication, rate limits, sandboxing of submitted certificates and result storage; those belong to a later milestone, with the JSON contract above as its wire format.

Proposed command: `rlt audit CLAIM.json [--out FILE]` (stdout by default).

### 4.6 Tests the milestone should carry

Positive: a claim for each existing statement record reproduces `rlt explain`'s verdicts field by field. Negative (modelled on `tests/explain/`): stale certificate hash, excerpt altered, unknown registry id, axiom-set SHA mismatch, inline bytes ≠ stated SHA, proposed status presented as recorded, conditional proof claimed unconditional, cited file rejected, duplicate key, YAML-only syntax, path traversal, tracked output path.

---

## 5. Roadmap and risk assessment

### 5.1 Completed and verified capabilities

| Capability | Milestone | Evidence |
|---|---|---|
| Sequent kernel, Γ retained, `imagined` separate | RLT-001 | `docs/RLT-001-AUDIT.md` |
| Specification records, chemical notation | RLT-002 (T6, T7) | `docs/RLT-002-AUDIT.md`, `docs/RLT-002-T7.md` |
| Multiplication; 111 modulo-30 witnesses | RLT-003 M1, M2 | `docs/RLT-003-M1.md`, `M2.md` |
| First-order logic; Robinson's Q; NAT-DEF definitions | M3a, M3b | `docs/RLT-003-M3a.md`, `M3b.md` |
| Non-divisibility, citation, enumeration, modulo-30 classification below 30 | M3c, M3d, M3e | `docs/RLT-003-M3c.md` … `M3e.md` |
| Soundness argument, `explain`, `deps`, stable identities | RLT-004a–c | `docs/KERNEL-SOUNDNESS.md`, `docs/RLT-004*-REPORT.md` |
| Published-paper demonstration: 8 is not matchable | RLT-005 | `docs/RLT-005-REPORT.md` |
| H₄₅ residue certificates; Lemma 12.1 discharged by composition | RLT-006, RLT-007a–d | `docs/RLT-007d-FINAL.md`, `research/h45-lemma-12-1/` |
| Bounded proof search and discovery | RLT-PROVER-001 | `docs/RLT-PROVER-001-AUDIT.md` |
| Registered function definitions, Lean declaration-level correspondence | RLT-PROVER-002 | `docs/RLT-PROVER-002-AUDIT.md` (final steps self-reviewed; an independent retest of `e2bb7e3` is listed there as the remaining step) |

### 5.2 Architectural debt

| Debt | Effect | Blocks | Remedy layer |
|---|---|---|---|
| **D2** definition identity by path + SHA only | two files can claim one id; claims and records cannot cite "the" DOUBLE | predicate definitions; RLT-AGENT-001 definition references; reading-point registrations that use definitions | pinned definitions registry (ids, names, SHA) in the binary, as for axiom sets |
| **D3** syntactic `unfolding`/`inference` classification | discovery overstates definition dependence (39 of 126) | honest reporting of predicate-definition discoveries | generalisation re-proof (replace each use with a fresh parameter) |
| Two definition mechanisms (NAT-DEF pinned, NAT-FUN by header) | two identity rules, two loaders | uniform predicate registry | fold NAT-DEF's pinning model into the D2 registry; keep `spec/definitions.rlt` bytes and verdicts unchanged |
| Inline predicates (Prime_B, Adm) | long goal strings; identity by text | reuse across records | register under the D2 registry |
| L8 lift `verified` refused | registered lifts stay `proposed` though provable | PiπΠ PI-2/PI-4 bindings in practice | `rlt validate` rule: `verified` iff every `via` relation is a cited kernel theorem |
| L9 unchecked labels in `rlt check` | a renamed relationship label still ACCEPTs | trustworthy agent claims carrying labels | registration check, separately reviewed (verdict change) |
| Discovery depth metadata | falsifiable within bound | — | record depth in certificate header or drop the column |
| PROVER-002 independence | final D6/H1 steps self-reviewed | release confidence | independent retest of `e2bb7e3` |
| README status | stale | user orientation | doc update |

### 5.3 Specifications ready for registration (no kernel change)

1. `(1 + 1) × 3 × 5 = 30` as a NAT-DEC relationship of math29, source NEWS-MATH29 (excerpt already registered).
2. Lift promotion for `math0-to-math5` ("(1+1) × 3 = 6") and `math5-to-math29` ("6 × 5 = 30"), after the L8 rule change.
3. "Expand imaginations. Extend care. Resist illiteracy." as a verbatim excerpt (source: math29 `carbondioxide/index.html` at a pinned commit), tagged as a PiπΠ convention text — if the author confirms this is the intended wording.
4. `left4`, `left5`, `lift5`, Z, and the projection form as an RH statement record: current source translatepi `RH/index.html` at `539562a`, replaced as leading source by the invariance-and-projection LaTeX paper once it is imported and pinned, Lean target `RiemannHypothesis` at a pinned Mathlib commit, kernel layer not-applicable, obligations (a) and (b) of §2.10 listed open.
5. Pi stages 0Pi–17Pi as an interpretive record from pi-stage-lab at `111ee1a`, each stage's rule verbatim.

### 5.4 Specifications requiring clarification

| Item | Question for the author |
|---|---|
| PiπΠ triad | Exact wording: "resist unsupported claims" or "Resist illiteracy."? Its source? |
| Glyphs Pi / π / Π | Formal meaning of each (open since RLT-000). |
| Piπ / πΠ | Definition of the alternation: which reading points, in which order, and what each step preserves. |
| `#left` | Meaning and source; relation, if any, to `(Left + 1) + 1 ≠ ±1(9424π)` and to `left4`/`left5`. |
| `+1π+1Π` | Meaning; whether it relates to the "+1 reading level" meanings. |
| `0Pi+` | Whether `+` denotes the stage sequence, a successor on stages, or another operation. |
| Triplet N / Spiral N | Domain (ℕ with s ≥ 2, or ℤ); relation between Triplet values (≡ 23 mod 24) and the 9423–9426 labels. |
| CO₂ | Which physical claims to record, with which measurement sources. |
| RH | Where the LaTeX paper should be imported (a repository and path to pin); whether "equivalence by its definitions" refers to step (a) or the full equivalence with Mathlib's declaration. |

### 5.5 Potential kernel changes

Of the extensions in §3.2, only K1 (numeral representation) requires a kernel change, and only if RLT itself must check large values. Recommendation: no kernel change now.

- **K2 (induction)** has the clearest mathematical payoff (modulo 30 for every n; every prime > 5) and needs no kernel change. It deserves its own architecture review comparing (i) a reviewed finite instance set (route 2), (ii) a schema with per-use instance construction in the loader (route 3), and (iii), for completeness, an induction rule in the kernel (route 1). The review should also fix how results are labelled, e.g. "relative to registered axiom set PA-INSTANCES-…", so that a theorem's dependence on induction stays visible.
- **K1** should wait for a concrete statement that needs it; Lean covers large values meanwhile.
- **K3 and K4** are served without kernel changes (domain restriction or pair encoding; Lean correspondence).

### 5.6 API work that can proceed independently

- `rlt audit` over existing statement records and certificates (path + SHA definitions), with `rlt-claim/1` / `rlt-audit/1` versioned so the D2 registry id can be added as a field later.
- JSON output for `rlt check` and `rlt prove` (currently text) using the existing writer.
- Schema documents and golden files for every JSON format.

### 5.7 Dependency-ordered milestones

```
M-A  RLT-DEF-001  Pinned definitions registry (D2) + registered predicate definitions + D3 generalisation check
        │            first users: Prime_B, Adm, free235 re-registered byte-identically, triplet_of(s, t)
        ├──► M-C  Registrations of §5.3 (records, lifts after L8, RH correspondence record)
        │
M-B  RLT-AGENT-001 v0  `rlt audit` local JSON CLI over existing records   (independent of M-A; v1 adds registry ids)
        │
        └──► M-D  RLT-AGENT-001 v1  definition references by registry id; claims for M-C records
M-E  Author clarifications (§5.4)  ──► registrations for PiπΠ, Piπ/πΠ, `#left`, `+1π+1Π`, `0Pi+`, Triplet N
M-F  Induction architecture review (K2)  ──► modulo 30 for every n, every prime > 5
M-G  Lean companion for `left5`–Mathlib (§2.10 (a), (b))  ──► RH correspondence record PASS
M-H  Network service (later)  — wire format = rlt-claim/1, rlt-audit/1
```

### 5.8 Risk register

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Registering a definition that diverges from the author's meaning | medium | records state a different mathematics | register only located, verbatim sources; semantic status `partial` until assessed |
| D2 exploited or confused in agent claims | medium once claims arrive | wrong definition cited under a familiar id | do M-A before claims reference definitions by id |
| Pressure to extend the kernel for statements Lean already handles | medium | larger TCB | route analysis-level mathematics to Lean companions; RLT records correspondence |
| Status inflation (proposed → recorded, conditional → unconditional, composed → single theorem) | medium | misleading claims | `rlt-audit/1` keeps recorded vs proposed; tests modelled on XR21/XR26 and N22 |
| Numeral cap reached by new certificates | high for Triplet N/9423-range | statements uncheckable | K1 review on first concrete need; Lean meanwhile |
| Unregistered sources drift | medium | excerpts change silently | pin commits at registration (`check_excerpts.py`) |
| RH page wording read as a stronger claim than step (a) | medium | over-claim | statement record lists obligation (b) explicitly |

---

## 6. Did RLT begin with an architecture that would require substantial refactoring to support its original mathematical goals?

**Answer: for the arithmetic, reading-point and correspondence goals, no; the design has absorbed every milestone from RLT-001 through RLT-PROVER-002 additively. For mathematics beyond first-order arithmetic over ℕ, RLT was designed from the start to record correspondence with Lean, which is the intended route rather than a refactoring.** Two localized pressure points remain: numeral representation, which would require a kernel change if RLT itself must check large values (K1), and induction, which enters as a registered axiom set or loader-level schema with the kernel's rules unchanged (K2).

**Evidence that the original architecture holds.**

1. *Layer separation was the first approved decision.* RLT-000 required four layers (written expression, convention, checked relationship, physical correspondence), sequents retaining Γ, `imagined` kept apart, and PiπΠ conventions kept without invented definitions (`docs/RLT-000.md`). Every later capability landed in one of those layers.
2. *Extensions were additive.* Multiplication (M1) added one symbol to `term::arity`; first-order logic (M3a) added 13 rules; Robinson's Q (M3b) added an axiom set; definitions (M3b, PROVER-002) expand before the kernel; citation (M3c) and composition (RLT-007d) are checked dependencies outside the rules. Since M3a, `scripts/compare-reports.sh` byte-compares every proof report between the old and new binaries (obligation O4), and the RLT-001 proofs `T4_math0_pairing.rlt` (1 + 1 = 2) and `R_math29_pairing.rlt` (29 + 1 = 30), unchanged since the RLT-001 audit commit `687f5e8`, still ACCEPT under their original PEANO-ADD set at `c9e347b`.
3. *Trust boundaries stayed fixed.* The kernel still has zero dependencies and its soundness argument is per rule (`docs/KERNEL-SOUNDNESS.md`); search, generators, elaboration and Lean sit outside it, and each new layer (prover, discovery, definitions) was placed outside by design and audited as such.
4. *Correspondence was first-class from the start.* Statement records, `rlt explain`, source pins and the physical layer already express the RH, CO₂ and PiπΠ cases without new architecture: a record with kernel layer not-applicable, a physical status with evidence, a convention with elaborator `none`.

**Where change would be real.**

- *Unary numerals* (L2) were chosen for kernel simplicity; Triplet N beyond Spiral N = 41 and 9423-range values exceed them. A change of numeral representation touches the kernel's term operations — localized to `term.rs` and the equational rules, with every proof re-checked.
- *No induction* (L3) bounds what RLT can prove about all n. Induction instances are ordinary first-order axioms, so adding them is a registered-theory decision — a finite instance set or a loader-level schema (K2, routes 2–3) — with the kernel's rules unchanged; it is central to what results mean, and each result keeps naming the axiom set it rests on.
- *Single sort* (L1) excludes ℤ; for the located mathematics this is avoidable (domain restriction or pair encoding).
- *Definition identity* (D2) and the two definition mechanisms are debt in the registry layer, not in the kernel.

**Uncertainties.**

- Several original expressions (`#left`, `+1π+1Π`, `0Pi+`, Piπ / πΠ, the exact PiπΠ triad) are unlocated. Their eventual specifications could require constructs beyond those surveyed here (for example, operations on stages or on reading points themselves). This answer covers the mathematics that is located.
- Whether the author's goal includes RLT *itself* proving induction-dependent or analytic statements determines whether K1/K2 become necessary; the sources read (RH page: "a Lean theorem connecting our operators to Mathlib's exact declaration, followed by an RLT correspondence record") point to correspondence for RH.
- The kernel's Rust implementation is argued and tested, unproved by machine (`docs/RLT-007-HANDOFF.md` §6); this assessment relies on that evidence as recorded.

---

## 7. Recommended next development milestone

**RLT-DEF-001 — Pinned definitions registry and registered predicate definitions** (M-A), with **RLT-AGENT-001 v0** (M-B) as an independent parallel track.

Scope of RLT-DEF-001, for review before implementation:

1. A pinned registry of definitions files (id, function and predicate names, SHA-256) compiled into the binary, as for axiom sets; NAT-FUN headers resolve through it; unregistered files refused (closes D2).
2. Predicate definitions (abbreviations over {0, S, +, ×} and the connectives), with the same node budgets as D1.
3. Re-registration of `lt`, `dvd`, `free235` with byte-identical expansion, so every existing certificate and report reproduces; `Prime_B` and `Adm` registered from their current inline text; `triplet_of(s, t) :≡ t + 25 = 24 × s` held back until the domain question in §5.4 is answered.
4. The D3 generalisation check in discovery reports.
5. Kernel, axiom sets, rules and limits unchanged.

Why this one first: D2 is named in the PROVER-002 audit as the prerequisite for predicate definitions; predicate definitions are the next stage already planned (`docs/RLT-PROVER-002.md` §7, "Stage 3 … will likely need predicate definitions in registries"); and the agent interface needs a stable definition identity to reference. RLT-AGENT-001 v0 can begin at the same time over records that use the pinned NAT-DEF file.

---

## 8. Verification of this document

- Only `docs/RLT-ROADMAP-001.md` is added; `git status` at `c9e347b` shows no other change.
- Commands run against the release build at `c9e347b`: `rlt check tests/accept/R_math29_pairing.rlt` (ACCEPT), `rlt check tests/m3e/accept/E_T1.rlt` (ACCEPT), `rlt validate` (VALID), `rlt gen` + `rlt check` for `(1 + 1) × 3 × 5 = 30`, `24 × 2 = 23 + 25`, `24 × 41 = 959 + 25` (ACCEPT, files kept outside the repository), `rlt gen "24 × 42 = 983 + 25"` and `rlt gen "24 × 43 = 1007 + 25"` (refused at the numeral limit).
- `cargo test --release` with `RLT_ALLOW_NO_LEAN=1` (Lean unavailable in this environment): exit 0, 387 tests passed across 26 test binaries, 0 failed — the same count the RLT-PROVER-002 audit reports for `e2bb7e3`. With `RLT_ALLOW_NO_LEAN=1` the Lean steps inside the D6/H1 tests are skipped, so this run confirms the Rust side only.
- Lean-dependent checks (`scripts/verify.sh`, Lean companions) were not run here.

## 9. Source references

RLT (at `c9e347b`): `README.md`; `docs/RLT-000.md`; `docs/KERNEL-SOUNDNESS.md`; `docs/PROVER.md`; `docs/RLT-PROVER-002.md`; `docs/RLT-PROVER-002-AUDIT.md`; `docs/RLT-007-HANDOFF.md`; `docs/RLT-007b-2-DESIGN.md`; `docs/RLT-007d-CORRESPONDENCE.md`; `docs/RLT-007d-FINAL.md`; `docs/RLT-004a-BASELINE.md`; `docs/STATEMENT-RECORDS.md`; `docs/MANUAL.md`; `kernel/src/term.rs`, `prop.rs`, `rules.rs`, `check.rs`; `syntax/src/proof.rs`, `elab.rs`, `defs.rs`, `fdefs.rs`; `cli/src/lib.rs`, `main.rs`, `explain.rs`, `deps.rs`, `compose.rs`; `spec/axioms.yaml`, `spec/robinson-q.rlt`, `spec/definitions.rlt`; `definitions/double.rlt`; `conventions/pi.yaml`, `residue-30.yaml`, `chem-molformula.yaml`; `readingpoints/math0.yaml`, `math5.yaml`, `math29.yaml`; `points/co2-and-o2.yaml`; `residues/residue-30.yaml`; `statements/MOD30-CLASSIFICATION.yaml`; `lib/mod30/mod30.classification.yaml`; `compose/h45.yaml`; `sources/registry.yaml`; `lifts/README.md`; `RH/index.html`.

External (revisions in §0): translatepi `sites/translatepi.ai/RH/index.html`, `pi-table.html`, `261001_index.html`, `indexOLD.html`, `qubit.html`, `spirals_1_99.html`, `src/triplet_generator.py`, `src/integerity_matrix.json`; math29 `carbondioxide/index.html`, `carbondioxide/RH/index.html`; pi-stage-lab `notebooks/00_0Pi_identity_expand.ipynb` and siblings; math0 and math5 `README.md`; riemann-hypothesis `index.html`; mathlib4 `Mathlib/NumberTheory/LSeries/RiemannZeta.lean`, `Nonvanishing.lean`; Lean community blog, "Lean verification of the Riemann Hypothesis" (2026-11-21), https://leanprover-community.github.io/blog/posts/riemann-hypothesis/.

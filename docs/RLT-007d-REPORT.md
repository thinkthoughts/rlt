# RLT-007d — Report: H₄₅ is an admissible 45-tuple of diameter 212, as a layered composition

**Branch:** `rlt-007d`, draft PR #21. **Design:** `docs/RLT-007d-FINAL.md` (carried here unchanged from PR #16; decisions D-1 to D-4 approved). **Correspondence:** `docs/RLT-007d-CORRESPONDENCE.md`.

## 1. What is established

`rlt compose compose/h45.yaml` composes 62 kernel-checked theorems into the named result **Lemma 12.1 (H₄₅)**, six statements:

| # | Statement | Label |
|---|---|---|
| 1–4 | `distinct.g0`–`g3`: the 990 disequalities among the 45 listed numerals | kernel accepted |
| 5 | `bounds`: each listed numeral h has 0 ≤ h and h ≤ 212 (as ∃z. z + 0 = h and ∃z. z + h = 212) | kernel accepted |
| 6 | `adm.all`: ∀p. (Prime_B(p) → ∃r. (r < p ∧ OM(p, r))) | discharged by composition (layer 2) |

with the list itself, its 45 entries and its endpoints **source transcription checked**, and the reading as Lemma 12.1 **semantic correspondence assessed** (`docs/RLT-007d-CORRESPONDENCE.md` §3). The report's own summary: "5 statements kernel accepted, 1 discharged by composition (2 composition layers above the components, 56 edges); a composition of kernel-checked theorems, not one kernel theorem", followed by the registered note of what lies outside the check (the transcription and the correspondence). Remaining assumptions: none.

## 2. The dependency graph

| Layer | Nodes | Each is |
|---|---|---|
| 0 | 47 × `om.P_R` (RLT-006: 2 … 43; stage 4: 47 … 211), `tail` (stage 3), `classify` (stage 2a, `primes.below_213`), `distinct.g0`–`g3`, `bounds` (stage 1) | an unconditional library theorem, re-checked in the run with every cited file |
| 1 | `adm.range_1` … `adm.range_6`, `adm.tail` | a conditional kernel theorem (assembly), its assumptions discharged by layer 0 |
| 2 | `adm.all` | a conditional kernel theorem (assembly), its 8 assumptions discharged by `classify` and layer 1 |

The 56 discharges:

| Node | Assumptions (name ← discharger) |
|---|---|
| `adm.range_1` | om2 ← om.2_1, om3 ← om.3_1, om5 ← om.5_3, om7 ← om.7_6, om11 ← om.11_9, om13 ← om.13_3, om17 ← om.17_1, om19 ← om.19_13, om23 ← om.23_17, om29 ← om.29_4, om31 ← om.31_11, om37 ← om.37_4 |
| `adm.range_2` | om41 ← om.41_4, om43 ← om.43_3, om47 ← om.47_1, om53 ← om.53_6, om59 ← om.59_6, om61 ← om.61_1, om67 ← om.67_1, om71 ← om.71_4, om73 ← om.73_3 |
| `adm.range_3` | om79 ← om.79_3, om83 ← om.83_3, om89 ← om.89_1, om97 ← om.97_1, om101 ← om.101_3, om103 ← om.103_1, om107 ← om.107_1, om109 ← om.109_3 |
| `adm.range_4` | om113 ← om.113_4, om127 ← om.127_1, om131 ← om.131_1, om137 ← om.137_1, om139 ← om.139_3, om149 ← om.149_3, om151 ← om.151_1 |
| `adm.range_5` | om157 ← om.157_1, om163 ← om.163_3, om167 ← om.167_1, om173 ← om.173_1, om179 ← om.179_4, om181 ← om.181_3 |
| `adm.range_6` | om191 ← om.191_4, om193 ← om.193_3, om197 ← om.197_1, om199 ← om.199_3, om211 ← om.211_3 |
| `adm.tail` | tl ← tail |
| `adm.all` | cls ← classify, ra1 … ra6 ← adm.range_1 … 6, ta ← adm.tail |

Intervals: [0, 40], [41, 78], [79, 112], [113, 156], [157, 190], [191, 212]. The `rlt compose` report lists every node with its label, source file and discharged edges.

## 3. Soundness

**Claim.** If `rlt compose` reports COMPOSED, then for every node v, its statement φ_v holds in every model of ROBINSON-Q.

**Argument (cut rule, induction on layers).**
- *Layer 0.* v's registered proof is re-checked in the run: the kernel accepts ∅ ⊢ φ_v (Γ = ∅ is required, as is a citable, assumption-free library entry). By kernel soundness (`docs/KERNEL-SOUNDNESS.md`) every model of Q satisfies φ_v.
- *Layer n ≥ 1.* v's assembly is re-checked: the kernel accepts Γ_v ⊢ φ_v with no citations. By kernel soundness, every model of Q satisfying all of Γ_v satisfies φ_v. `rlt compose` requires every γ ∈ Γ_v to have exactly one discharge row, naming a node u of a strictly lower layer that passed in the same run (the discharges are compared once every node has passed), with canonical(γ) = canonical(φ_u) — the same closed formula up to the names of bound variables — under the same axiom set, convention and definitions file. By the induction hypothesis every model of Q satisfies φ_u, hence γ, hence φ_v.
- *Well-foundedness.* Layers are integers checked to equal 1 + the highest discharger's layer; every edge strictly decreases the layer, so the graph is acyclic and the layer order is a topological order.

This is the semantic cut rule across files; inside one file the kernel performs the same step with `cite` and imp-elim.

**Four separate questions, with their status:**

| Question | Status |
|---|---|
| Is the composition rule valid? | yes, by the argument above (cut rule; induction on layers) |
| Did the kernel accept every component and assembly? | **kernel accepted**: each proof re-checked in the run by the unchanged kernel |
| Did the checker implement the rule correctly? | new trusted code (`cli/src/compose.rs`), cross-checked by an independent implementation (§5), 51 negative cases and audits (§7) |
| Do the six statements say Lemma 12.1? | **source transcription checked** and **semantic correspondence assessed** (`docs/RLT-007d-CORRESPONDENCE.md`) |

**Trusted boundary.**

| Part | Status |
|---|---|
| Kernel, 20 rules, Q's 7 axioms, limits, parser, `rlt check`, `canonical()`, fingerprints | trusted already; unchanged |
| `rlt compose` | new trusted code: reads the bundle and the register, checks the graph, re-checks every proof through the existing check function, compares canonical formulas. It builds no formula, applies no rule and creates no kernel theorem |
| The bundle and the two registers | reviewed records; every pin re-verified, every statement re-derived by the kernel in the run |
| `lib/` and `lib/identities.tsv` | the existing library: `rlt compose` recomputes each component's fingerprint; the Python checker compares entries with the append-only identity registry (checked against history by `scripts/verify.sh`) but does not recompute fingerprints |
| Generator, Python checker, Lean file, reports | untrusted or independent evidence |

## 4. Implementation

- **`rlt compose BUNDLE`** (`cli/src/compose.rs`). Exit 0 composed, 1 refused, 2 malformed or missing, 3 stale. Order of work:
  1. the bundle (format `rlt-compose/1`, exact keys);
  2. pins: axiom set and convention registered, definitions file equal to the pinned one;
  3. graph: unique node ids, library ids and assembly paths; one discharge row per assumption; each row names an existing node of a lower layer; stated layers recomputed; the result non-empty and duplicate-free; every node used;
  4. library entries: present, citable, unconditional, fingerprint as pinned;
  5. file pins (assemblies, specification probes);
  6. the **approved specification register** `compose/approved-specifications.tsv` (fixed path, not chosen by the bundle): every node must carry the registered probe for its id;
  7. each specification probe re-checked: one declared assumption, equal to its goal, no citations;
  8. each assembly re-checked (no citations; Γ equal to its declared assumptions and to its discharge rows) and its statement compared with its probe; then each layer-0 proof re-checked with every cited file, its fingerprint recomputed and its statement compared with its probe; no two nodes with the same statement;
  9. in layer order, every discharge: γ compared canonically with the statement of its discharger;
  10. the **named-result register** `compose/approved-results.tsv` (fixed path): the bundle's id must be registered, with exactly its registered, ordered result.
  The first failure ends the run with `verdict REFUSED (class) — reason`; classes: malformed, missing, stale, conditional, duplicated, unused, graph, incompatible, kernel, mismatch, specification.
- **Bundle** `compose/h45.yaml`: 62 nodes; layer-0 nodes name a library id and its fingerprint; composed nodes name an assembly (path, SHA-256) and their discharge table; every node names its specification probe.
- **Assemblies** (`tests/rlt007d/assemble/`): the eight prototyped conditional proofs, generated by `scripts/gen-rlt007d.py`. Each range assembly proves r_q < q for its primes; the tail assembly proves 1 < p from 212 < p.
- **Specifications**: 62 probes `(goal φ) (assume s φ)` proved by `(hyp s)`; the kernel's reading of φ is the approved statement.

**Decisions taken in implementation** (within D-1 to D-4; for review):
1. *One bundle and two registers* instead of per-result composition records; **composed results are not library entries**, so they are never citable. This departs from FINAL §5 (a library id with `status: composed`, `citable: false`) and §13: the assemblies and the named result have no line in the append-only `lib/identities.tsv`; their identity rests on the SHA-256 pins in `compose/h45.yaml` and on the two reviewed registers.
2. *Unused declared assumptions are always refused* (the design allowed a reasoned `unused: true` mark; none is needed).
3. *Both registers are fixed* (`compose/approved-specifications.tsv`, `compose/approved-results.tsv`), so a bundle can neither pair a weaker theorem with a probe of its own (N41) nor use the lemma's name for fewer statements (N43). The registers are reviewed files; the binary does not pin their hashes, and `scripts/gen-rlt007d.py --check` and the Rust tests rebuild them.
4. *Layer-0 re-checks use `rlt check` on the registered proof* plus a recomputed fingerprint, rather than `rlt deps` (same pins, half the time); the library's own consistency stays with `rlt deps --verify-lib` in `scripts/verify.sh`.
5. *The Python checker reads `rlt check` reports* rather than `rlt deps --format json` reports (same kernel output; no shared matching code).
6. *Assemblies are checked before components.* Every proof is still re-checked in the run; the discharges are compared once every statement is known. A statement that is not the approved one is refused within seconds (N44, the full bundle with a weaker `adm.all`).
7. *Refusal classes for a component's citations:* a cited file whose SHA-256 differs is **stale** (exit 3), an absent one **missing** (exit 2); both are exercised in `scripts/verify.sh`.
8. *Pinned files lie inside the repository:* a path with `.` or `..` components, or reached through a symbolic link, is refused as malformed.

## 5. Independent verification

**Python checker** (`scripts/compose-check.py`). Uses Rust only through `rlt check`. Its own bundle reader, structural rules, parser for the kernel's printed formulas, de Bruijn canonical form, and approved statements rebuilt from the transcription and the interval table (not from the probes). Same classes and exit codes.

**Agreement.**

| Cases | `rlt compose` | `compose-check.py` |
|---|---|---|
| full bundle `compose/h45.yaml` | COMPOSED, exit 0 | COMPOSED, exit 0; all six results equal to the statements rebuilt from the transcription |
| controls: tail; tail with the assumption over q instead of p; range 6 | COMPOSED | COMPOSED |
| 51 negative cases | each refused with the listed class, exit and reason | each refused with the same class and exit |

**Negative cases** (`tests/rlt007d/expected.tsv`), by class:

| Class | Cases |
|---|---|
| malformed | unknown key; wrong format; a layer-0 node with an assembly; a YAML anchor; an explicit tag; a repeated key; a `./` path; a non-string id; a byte-order mark |
| missing | unknown discharger; unknown library id; assembly file absent; probe absent; result names an unknown node; an assumption without a row |
| stale | assembly hash; probe hash; layer-0 fingerprint |
| conditional | a conditional library theorem as a component |
| duplicated | node id twice; library theorem twice; two rows for one assumption; result twice; one assembly under two nodes |
| unused | an unreferenced component; a declared but unused assumption; a row naming no assumption; the full bundle without `adm.all` in its result |
| graph | self-discharge; wrong stated layer; a composed node with no discharges; an assembly citing a file; a two-node cycle |
| incompatible | axiom set PEANO-ARITH; convention NAT-DEC; another definitions file; an unregistered axiom set |
| kernel | an assembly the kernel refuses |
| mismatch | an assumption discharged by the wrong component; two rows swapped |
| specification | **true but different**: a stronger tail (211 < p); a component probe over 44 elements; a weaker tail without r < p; range 6 without 211; a weaker tail with a matching probe of its own; the full bundle with a weaker `adm.all` (r < p dropped); the lemma's name for the tail alone; also a non-probe, a probe the kernel refuses, an unregistered node, the registered probe copied to another path |

**Further refusals in `scripts/verify.sh`:** a file cited by a component changed (stale, exit 3) and then absent (missing, exit 2), by both checkers, with the file restored afterwards.

**Rust tests** (`cli/tests/rlt007d.rs`, 9 tests): the 62 registered probes equal statements rebuilt in the test from the paper's §12 (both encodings checked by the kernel); true-but-different variants differ canonically (a stronger and a weaker tail, range 6 without 211, `adm.all` without r < p or from 1 < p, an omission with a conjunct dropped or repeated); the full bundle composes with every node equal to its rebuilt statement; the controls compose and no composed node is called unconditional; every negative case; the probe-shape checks; a pinned file reached through a symbolic link (inside or outside the repository) is refused by both checkers; Python agreement; the generator reproduces every output.

**Lean** (`lean/RLT007D.lean`): see `docs/RLT-007d-CORRESPONDENCE.md` §6. 17 theorems, standard axioms only; the 62 node statements symbol for symbol; `rlt_lt_iff` and `rlt_dvd_iff` tie RLT's `<` and `∣` to Lean's; `lemma_12_1_holds` concludes Lemma 12.1 in Lean outright.

## 6. Measurements

On the local workspace (2 CPUs), release build:

| Run | Wall | CPU |
|---|---|---|
| `rlt compose compose/h45.yaml` (62 proofs re-checked with every cited file, 62 probes) | 4 min 38 s | 4 min 36 s |
| `scripts/compose-check.py` on the same bundle (kernel runs in parallel) | 2 min 31 s | 4 min 54 s |
| both, in parallel, plus the three controls (`scripts/verify.sh`) | 6 min 18 s | — |
| 51 negative cases, both checkers | about 4 min | — |
| `lean RLT007D.lean` | 3 min 24 s | — |
| `cargo test --test rlt007d` (9 tests) | 7 min 44 s | — |
| the eight assemblies, `rlt check` each | 1.9 s in all | — |

Committed size: `tests/rlt007d` 3.54 MB in 140 files (assemblies 1.07 MB, probes 0.38 MB, controls 0.26 MB, negative cases 1.83 MB), `compose/` 29 KB, `lean/RLT007D.lean` 140 KB. Generated at verification time: none beyond the existing certificate families (RLT-006, stages 1 and 4), which the components cite.

Full `scripts/verify.sh` on the workspace at 99e3e29: 80 min 48 s wall, 116 min CPU, run alongside two audits (§8).

## 7. Audits

Two independent audits on e2436bf (mathematical; implementation), corrections in 99e3e29 and the final commit, and targeted re-audits of 99e3e29.

**Mathematical audit.** No blocking finding: the cut is complete (every γ of every assembly needs a row; unused declared assumptions refused; layer order enforced); compared formulas are closed sentences; definitions are expanded before matching; layer 0 is unconditional; the six statements read in ℕ give Lemma 12.1. All 62 statements, the classes and the register hashes recomputed independently. Should-fix items, all corrected: the report's summary wording and labels; the named result's six ids pinned; Lean lemmas relating RLT's `<` and `∣` to Lean's; three promised negatives (weaker `adm.all`; `adm.all` from 1 < p; an omission with a conjunct dropped or repeated); the design document carried into the branch. Re-audit: all fixed; three wording corrections in these documents, made.

**Implementation audit.** 18 adversarial bundles; none composed with an undischarged, mismatched, conditional or stale premise.

| Finding | Correction |
|---|---|
| Blocking: the lemma's name could label a one-statement result | named-result register `compose/approved-results.tsv` (N43) |
| Python composed bundles Rust refused (anchors, tags, repeated keys, `./` paths, non-string ids, a probe copied elsewhere, a library identity tampered) | strict YAML reader, scalar types, the path rule, the specification register and `lib/identities.tsv` in Python (N45–N51) |
| Python crashed on list-valued fields | type checks |
| A component's changed cited file was classed `kernel` | `stale` (changed) and `missing` (absent), both checkers, verify.sh tamper test |
| Symbolic links in pinned paths followed | refused as malformed in both (Rust test) |
| Re-read files not compared with pins | compared |
| Probe-shape checks untested in Rust | `probe_parts` unit-tested |
| Python report lacked the disclaimer | added |

Re-audit of 99e3e29: every item fixed, Rust and Python agreeing on every adversarial row. Remaining notes, recorded here: Python checks library fingerprints through the append-only identity registry, not by recomputation (trusted-boundary table, §3); the named-result check comes last, so a misnamed full-size bundle is refused after its kernel work (time only); the stale/missing classification reads the kernel's citation message, guarded by the verify.sh tamper test; a byte-order mark is now refused by both (N51).

## 8. Verification

- `scripts/gen-rlt007d.py --check`: 145 outputs, 0 differ.
- `cargo test --test rlt007d`: 9 of 9; Rust and Python agree on all 51 negative cases and 3 controls.
- Full `scripts/verify.sh` at 99e3e29 (local): every stage passed except two lines, both corrected or environmental: the RLT-007d report check still expected the earlier summary wording (corrected; re-run by hand on the final binary: report check passes), and the vendored lean-bgp212 comparison needs `SOURCE_CHECKOUTS_DIR`, which CI provides. Results there: cargo test all passed; RLT-007d section — full bundle and controls composed by both checkers, 51 of 51 negatives (50 at that commit), tamper refusals stale and missing, file restored; Lean replay 23 files, `RLT007D.lean` 17 statements, standard axioms only.
- CI at 3f61ab8: `check` passed in 46 min 37 s ("RESULT all checks hold": full bundle and controls composed by both checkers in 243 s, 51 of 51 negatives, both tamper refusals, `RLT007D.lean` 17 statements); `gdt-comparison` passed. (99e3e29 failed only on the outdated report line, corrected in 3f61ab8.)
- Earlier proofs: `scripts/compare-reports.sh` with main 838954f's binary and tree: 2,744 proof-file reports and every record command byte-identical (0 differences). The kernel, the 20 rules, Q's axioms and the limits are unchanged.

## 9. Wording for the paper's authors

> We checked Lemma 12.1 of your September 3, 2026 draft ("H₄₅ is an admissible 45-tuple of diameter 212") in RLT, a small proof checker for first-order arithmetic over Robinson's Q. The result is a composition of 62 machine-checked theorems, assembled by a separate composition checker; no single proof object covers the whole lemma.
>
> - **Kernel-checked outright:** the 45 listed numbers are pairwise distinct (990 disequalities), and each lies between 0 and 212.
> - **Kernel-checked, prime by prime:** H₄₅ omits the class you list for each prime up to 43, and an explicit class for each prime from 47 to 211 (the least omitted one; your proof uses a counting argument there). It omits the class 1 for every prime above 212. Every prime is one of the 47 primes below 213 or exceeds 212.
> - **Joined by composition:** for every prime p, H₄₅ omits some residue class r < p. Eight conditional theorems connect the per-prime facts; the composition checker verified that each of their 56 assumptions is exactly one of the kernel-checked theorems, re-checking every proof in the same run.
> - **Checked against your text:** the 45 numbers, their order and the 14 classes come from a reviewed transcription of your draft, and the reading of these statements as Lemma 12.1 was reviewed separately.
>
> An independent checker written in Python and a Lean 4 formalization agree with this result. Everything is reproducible from the public repository with one command (`rlt compose compose/h45.yaml`).

# RLT-QEC-001 — Quantum Error Correction

**Status.** Specification registration, 2026-11-25. Documentation only: this milestone adds this document and the manifest `research/qec-001/MANIFEST.yaml`. The kernel, axioms, registries (`sources/registry.yaml`, `spec/`), certificates, records and website files stay exactly as they are at the baseline.

**Revision 2** (2026-11-25, after the audit `docs/RLT-QEC-001-AUDIT.md`): GF2-03 now states the source convention; §3.1 separates the 45 syndrome positions (C0) from the 45 face generators (C2) and records the coordinate reflection; the push theorems are cited for both quotient steps; the axiom report is scoped to the capstones that carry it; the QECLean `sorry` finding is corrected; O-2 and O-7 are restated; §7 is refined into RLT-QEC-002a, 002b and 002c with three evidence levels.

**Baseline.** `main` at `6128c97` ("docs: record the RLT-DEF-001a release").

**Objective.** Establish the initial quantum error correction research specifications within RLT, with one concrete target for independent certificate verification: the weight-8 logical floor of the 90-qubit base quotient in the Bravyi360 development.

**Repositories.**

| Role | Repository | Use in this milestone |
|---|---|---|
| RLT | `thinkthoughts/rlt` | registration (this branch, `rlt-qec-001`) |
| Research fork | `thinkthoughts/bb-distance-lean` | home of future source-specific audits and evidence |
| Upstream | `QECLean/bb-distance-lean` | read only, at the pin below |
| Lean dependency | `Stavan-Jain/QECLean` (package `QEC`) | read only, at the pin below |

The upstream repositories stay unmodified.

---

## 1. Registered specifications

### RLT-QEC-001 — Quantum Error Correction

Umbrella research specification. Scope: exact, finite statements about binary linear codes and homological (CSS) quantum codes that RLT can state, check and explain under explicit assumptions. Every claim keeps three reports separate, following RLT practice: proof validity, source fidelity, and physical correspondence. The physical meaning of a qubit, an error or a decoder stays outside the mathematical claim; the mathematical claim concerns vectors over GF(2).

### RLT-QEC-001a — Homological Distance Lower Bound

Statement form. For a chain complex over GF(2)

    C2 --∂2--> C1 --∂1--> C0,   ∂1 ∘ ∂2 = 0,

with C1 = GF(2)^n and Hamming weight |·|, a *logical floor d* holds when every 1-cycle of weight below d is a boundary:

    ∀ u ∈ C1,  ∂1 u = 0  ∧  u ∉ im ∂2   ⟹   d ≤ |u|.

This is the shape of QECLean's `LogicalFloor` (`QEC/Stabilizer/Framework/Homological/BBDoubling.lean:157`, at the pin), restated over the base complex. It covers one homology sector (cycles of ∂1 modulo boundaries of ∂2); the code distance combines both sectors (§4).

### RLT-QEC-001b — Binary Algebra and Independent Certificate Verification

Reusable binary-algebra specifications (§2) together with the verification discipline for externally generated certificates: a certificate is data; a claim rests on an independent check of the data against registered definitions, and a completeness claim rests on an independent argument that the data covers the whole finite target set.

## 2. Verification targets

Each target is a specification to register in RLT-QEC-001b. Each one states a finite, decidable relationship.

| ID | Verification target | Precise statement to register |
|---|---|---|
| GF2-01 | Binary field arithmetic | GF(2) = {0,1} with addition XOR and multiplication AND satisfies the field axioms; vectors in GF(2)^n correspond to n-bit masks, with vector addition = bitwise XOR. |
| GF2-02 | Finite group convolution | For a finite abelian group G and f, g : G → GF(2): (f ⋆ g)(x) = Σ_{a+b=x} f(a)·g(b). Convolution is commutative and associative, and it is the action of the group algebra GF(2)[G] on itself. |
| GF2-03 | Boundary operator composition | For the bivariate bicycle complex with A, B ∈ GF(2)[G]: with (a⋆b)(g) = Σ_h a(h)·b(g−h), the source convention (QECLean `BBChainComplex.lean:390–395`) is ∂2 f = (A⋆f, B⋆f) and ∂1 (u, v) = B⋆u + A⋆v, so ∂1 ∘ ∂2 f = B⋆A⋆f + A⋆B⋆f = 0 by commutativity of GF(2)[G] in characteristic 2. |
| GF2-04 | Kernel and image membership | u ∈ ker ∂1 is checked by evaluating ∂1 u = 0; u ∈ im ∂2 is established by an explicit witness c with ∂2 c = u (a coefficient vector over the boundary generators). |
| GF2-05 | Hamming weight | \|u\| = number of nonzero coordinates = popcount of the mask, independent of the chosen coordinate order. |
| GF2-06 | Finite enumeration completeness | For a finite target set T = {u ∈ GF(2)^n : P(u)} with decidable P, a list L is a complete census of T when every element of L satisfies P, L is duplicate-free, and every u satisfying P occurs in L. Completeness requires an argument covering all of GF(2)^n (or a proved reduction of it); checking the members of L establishes soundness only. |

GF2-03 records the source convention. RLT may register this convention or an equivalent one (for example the transpose pairing ∂2 f = (B⋆f, A⋆f), ∂1 (u, v) = A⋆u + B⋆v); a different choice carries an explicit isomorphism of complexes, recorded under O-3 and O-4.

## 3. Initial certificate target

**Source.** `BBDistance/Bravyi360/BaseFloor.lean`, line 83:

```lean
theorem base_logical_floor : baseCover.LogicalFloor 8
```

### 3.1 The base code, as read from source

| Item | Value at the pin | Location |
|---|---|---|
| Base group | `ZMod 15 × ZMod 3` (order 45) | `Bravyi360/Defs.lean:19` |
| Polynomial A | x⁹ + y + y² | `Defs.lean`, `baseA` |
| Polynomial B | 1 + x¹⁰ + x¹¹ | `Defs.lean`, `baseB` |
| Qubits | 90 = 45 × 2 (`BaseGroup × Fin 2`) | `baseComplex := bbChainComplex baseA baseB` |
| Syndrome positions (C0, target of ∂1) | 45; each qubit meets 3 (`baseSyndromePoints`: (g,0) ↦ g + {0, (10,0), (11,0)}, (g,1) ↦ g + {(9,0), (0,1), (0,2)}) | `BaseSparse.lean`; masks below 2⁴⁵ by `baseCensusSyndrome_bounded` |
| Face generators (C2, source of ∂2) | 45; each face row ∂2 δ_g has weight 6 (3 + 3) | `BaseSparse.lean`, `baseBoundary2_single_eq_faceRow` |
| Bit encoding | `baseFaceEnum : BaseGroup ≃ Fin 45` negates both coordinates before `finProdFinEquiv`; `baseQubitEnum` places the Fin 2 half first, then `baseFaceEnum`; the same enumeration indexes C0 and C2 | `Defs.lean:62–76` |
| Quotient chain | 360-qubit code over `ZMod 30 × ZMod 6` (A = x⁹+y+y², B = y³+x²⁵+x²⁶) → 180-qubit middle over `ZMod 30 × ZMod 3` → 90-qubit base over `ZMod 15 × ZMod 3` | `Defs.lean` |

The base polynomials agree with reducing the top polynomials (y³ ↦ 1 modulo 3; x²⁵, x²⁶ ↦ x¹⁰, x¹¹ modulo 15); in source, `top_push_A`/`top_push_B` state the top-to-middle step and `base_push_A`/`base_push_B` the middle-to-base step (`Defs.lean:91–97`, `decide +kernel`).

### 3.2 Reported certificate data and how the proof uses it

| Name | Reported | Read from source at the pin |
|---|---|---|
| `allBaseCycles10` | 1,954 packed binary masks | 1,954 entries, strictly increasing, all below 2⁹⁰; weight distribution {0: 1, 6: 45, 8: 90, 10: 1,818} (counted by a parser during this registration; pending independent recomputation) |
| `baseBoundaryWindow` | boundary membership data | 45 pairs (index, mask); `baseBoundaryWindow_columns` (`decide +kernel`) states each mask is a face check |
| `baseCycle10_complete` | census completeness theorem | `BaseCycleCensus.lean:2033` |
| Generator | `scripts/bravyi360/generate.py` | reproduces Lean certificates from checked-in candidate data (`scripts/bravyi360/certificates/*.gz`, hashes in `audit_manifest.json`); the generator re-emits data rather than searching for it |

**Structure of the completeness proof (read, not built).** `baseCycle10_complete` combines two facts:

1. `allBaseCycles10_checked` (`decide +kernel`): the list has length 1,954, is strictly ordered, and each entry is below 2⁹⁰, has zero syndrome and weight ≤ 10.
2. `baseCycle10Target_card_le`: the target set `baseCycle10Target = {w < 2⁹⁰ : syndrome(w) = 0 ∧ weight(w) ≤ 10}` has at most 1,954 elements. This bound comes from a separate trellis census (`cycle_card_le`, `cycle_kernel_to_trellis`, `cycle_permutation_injective`, in `ConcreteCensusBounds` / `ConcreteKernelSpan` / `ConcretePermutations`) after an injective coordinate permutation.

A sorted, valid list whose length meets the cardinality bound of the target equals the target (`ordered_census_complete`). The whole completeness content therefore sits in fact 2. An independent verification must reproduce fact 2 (or the census itself) by its own enumeration; re-checking the 1,954 masks reproduces fact 1 only.

**Path to the floor.** `base_cycle_lt_eight_boundary`: a cycle of weight < 8 lies in the census (`base_cycle_mem_census`), and `baseFloor_checked` gives every census entry of weight < 8 a coefficient vector over `baseBoundaryWindow` reconstructing it, so it is a boundary. `base_logical_floor` follows. The floor uses only census entries of weight < 8: the zero mask and the 45 weight-6 masks. The weight-8 and weight-10 entries serve later stages of the Bravyi360 argument.

## 4. Weight-8 base floor versus weight-24 distance

These are distinct results and are registered as distinct claims.

| | Base logical floor | Full Bravyi360 distance |
|---|---|---|
| Code | 90-qubit quotient over Z15 × Z3 | 360-qubit code over Z30 × Z6, `[[360, 12, 24]]` |
| Statement | `baseCover.LogicalFloor 8`: one homology sector of the base complex, lower bound 8 | `distance_lower_bound`, `stabilizerCode_hasDistance24` (`Bravyi360/Distance.lean`): exact distance 24 for the stabilizer code |
| Role | one input to the lifting argument (`middle_logical_floor`, through the two double covers) | final public object `stabilizerCodeWithDistance : StabilizerCodeWithDistance 360 12 24` |
| Additional ingredients | — | middle floor, diagonal boundaries, 58,130-shadow bank, top-fiber exclusion of nonboundary cycles of weight ≤ 22, X/Z duality and even cycle weights (per `Bravyi360/README.md`) |

RLT-QEC-001 targets the base floor only. A verified base floor establishes neither the middle floor nor distance 24.

## 5. Source findings and their status

Status vocabulary: **confirmed from source** means read directly in the pinned files during this registration (no Lean build); **pending independent verification** means reported, or requiring a build or computation that this milestone leaves for later.

| Finding | Status |
|---|---|
| `bb-distance-lean` at `a83cafad4d041427af4ddb54085493d3f9dd02d2`; this commit is HEAD of both `QECLean/bb-distance-lean` and `thinkthoughts/bb-distance-lean` on 2026-11-25 (commit date 2026-11-04) | confirmed from source |
| QECLean dependency at `f51011bd6d1af9e225a19a5d5ff672000f3c9d94`, URL `https://github.com/Stavan-Jain/QECLean`, in both `lakefile.toml` and `lake-manifest.json` | confirmed from source |
| Lean `leanprover/lean4:v4.30.0-rc2` in `lean-toolchain` of both repositories | confirmed from source |
| Mathlib at `a3fde5017d59fa3cef8d4d24999f6b9b71b89a3b` (from `lake-manifest.json`; other transitive pins in the manifest file) | confirmed from source |
| Theorem statement and location of `base_logical_floor` | confirmed from source |
| Certificate count 1,954, ordering, and weight distribution | confirmed by parsing the source list; independent recomputation pending |
| 45 boundary-window entries | confirmed by parsing; their equality with the face checks pending (stated by `decide +kernel` in source) |
| Bravyi360 sources use kernel `decide`; the `native_decide`, `sorry` and `axiom` occurrences found in `BBDistance/` are in comments | confirmed by text search |
| Axioms `[propext, Classical.choice, Quot.sound]` | reported in `Bravyi360/README.md` for the capstones `distance_lower_bound`, `stabilizerCode_hasDistance24` and `stabilizerCodeWithDistance`; `BaseCycleCensus.lean` carries `#print axioms` for `baseCycle10_complete` and `base_floor_eight` with the output unrecorded; `base_logical_floor` carries none. The capstones depend on `base_logical_floor` (through `MiddleFloor.lean:61,68`), so their report bounds its axioms once the dependency is confirmed by a build; pending (O-6) |
| QECLean at the pin: every occurrence of `sorry` found by text search is inside documentation text (for example `Framework/Concatenation/Embedding.lean:23`, `Codes/Iceberg/N.lean:94`, whose docstrings describe skeleton stages); the search found zero `sorry` proof terms | confirmed by text search; build-level confirmation pending (O-6) |
| Kernel replay of 39,227 declarations (`provenance/original-validation/`) | reported; pending independent verification |

## 6. Unresolved verification obligations

| ID | Obligation | Target |
|---|---|---|
| O-1 | Independent census completeness: enumerate all cycles of weight ≤ 10 of the base complex from the registered definitions alone, and compare with `allBaseCycles10` (expected: equal sets, 1,954 elements). Minimum for the floor alone: all cycles of weight ≤ 7. | GF2-06, GF2-04, GF2-05 |
| O-2 | Independent boundary witnesses: for each census cycle of weight < 8, a coefficient vector c with ∂2 c = u, checked independently. Given O-1, a sufficient check is that the 45 weight-6 cycles are exactly the 45 face rows ∂2 δ_g and that the zero mask is ∂2 0. | GF2-04 |
| O-3 | Encoding correspondence: fix RLT's coordinate convention for G × Fin 2, the faces, and the bit order, and state its correspondence with `baseQubitEnum`, `baseFaceEnum`, `reconX`, `cycleSyndrome` and the reflected coordinates noted in `Defs.lean`. | GF2-01, GF2-03 |
| O-4 | Operator correspondence: show the independently defined ∂1, ∂2 equal QECLean's `bbChainComplex baseA baseB` maps (source theorem `baseBoundary1_eq_reconLinear` supplies the Lean side). | GF2-02, GF2-03 |
| O-5 | Definition correspondence: RLT-QEC-001a's floor statement versus `XDoubleCoverData.LogicalFloor` (which phrases the cycle condition with `bbBoundary1Fn D.Ab D.Bb`). | RLT-QEC-001a |
| O-6 | Trust boundary of the Lean result: axioms by `#print axioms` under the pinned toolchain, and the import cone of `base_logical_floor` free of `sorry` and `native_decide`. | source audit |
| O-7 | Candidate-data provenance: hashes of `scripts/bravyi360/certificates/*` against `audit_manifest.json`; the candidate data comes from an original large-distance search that regeneration replays only in part (`scripts/bravyi360/README.md`: candidate certificates, small parity searches replayed, the large search left as recorded); recorded as external input outside the trusted proof. The qec-lab reference in `Defs.lean` concerns the matrix and coordinate convention. | source audit |
| O-8 | Sector coverage: identify the source theorem covering the second (Z) sector and how X/Z duality enters; recorded so the base floor is reported with its exact scope. | RLT-QEC-001a |

## 7. Proposed next milestone — RLT-QEC-002: independent verification of the base floor

### 7.1 Three evidence levels

Every RLT-QEC-002 result is reported at exactly one of these levels, and a result moves up a level only through the work that level names.

| Level | Meaning | What it establishes | Trust rests on |
|---|---|---|---|
| **R — independent computational reproduction** | A program written independently of the Lean development and its generator recomputes a result from the registered definitions (GF2-01 to GF2-06) and obtains the same output. | Agreement of two independent computations; evidence for the source claim. | The reproducing program, its toolchain and the encoding correspondence (O-3). |
| **V — independent certificate verification** | A checker, separate from the program that produced the data, accepts a certificate (masks, witnesses, census) against registered definitions. Soundness checks (each item is valid) and completeness arguments (the items cover the target) are reported separately. | The certificate is correct for the stated finite claim; completeness only when a completeness argument is itself verified. | The checker, which is small and auditable, plus O-3. |
| **K — kernel-checked mathematical proof** | RLT's kernel checks a derivation of the statement under registered axioms and definitions. | A theorem of RLT under those axioms. | The RLT kernel and its axioms. |

The Lean result `base_logical_floor` is level K relative to Lean's kernel. RLT-QEC-002 aims at levels R and V for the base floor; level K inside RLT is a later question (§7.5).

### 7.2 RLT-QEC-002a — Independent enumeration of all cycles of weight less than eight

- **Statement.** The set Z₇ = {u ∈ GF(2)⁹⁰ : ∂1 u = 0, |u| ≤ 7} of the base complex, computed from A = x⁹ + y + y², B = 1 + x¹⁰ + x¹¹ over Z15 × Z3 under GF2-03.
- **Expected value (from the source census, pending).** 46 elements: the zero vector and 45 vectors of weight 6.
- **Method for review.** Syndrome meet-in-the-middle: each u ∈ Z₇ is a sum of two vectors of weight ≤ 4 and ≤ 3 with equal 45-bit syndromes, giving Σ_{k≤4} C(90, k) ≈ 2.7 × 10⁶ half-vectors; translation by the 45-element group may fix one support coordinate per qubit orbit. Completeness follows from the splitting argument, which is part of the deliverable and reviewed on its own.
- **Level.** R for the enumeration. V when a separate checker verifies (i) each listed vector is a cycle of the stated weight and (ii) the splitting argument covers every vector of weight ≤ 7.
- **Comparison.** Map the result into source masks through the O-3 encoding and compare with the weight < 8 entries of `allBaseCycles10`.

### 7.3 RLT-QEC-002b — Independent GF(2) boundary witness construction and verification

- **Statement.** Every u ∈ Z₇ lies in im ∂2, so `LogicalFloor 8` holds for the base complex.
- **Construction.** For each u, a coefficient vector c ∈ GF(2)⁴⁵ with ∂2 c = u, found by Gaussian elimination over GF(2) on the 90 × 45 matrix of ∂2 (or, given 002a's expected shape, c = δ_g for the 45 face rows and c = 0 for the zero vector).
- **Verification.** A checker evaluates ∂2 c and compares it with u, coordinate by coordinate; this check is independent of how c was found.
- **Level.** V for the witnesses. Combined with 002a at level V, the floor statement reaches level V; with 002a at level R, the floor statement is reported at level R.
- **Correspondence.** Relate witnesses to source `baseBoundaryWindow` and `baseFloorCoefficient` through O-3; state the floor definition's correspondence with `LogicalFloor` (O-5).

### 7.4 RLT-QEC-002c — Complete weight-ten census reproduction

- **Statement.** Z₁₀ = {u : ∂1 u = 0, |u| ≤ 10} has exactly 1,954 elements and equals `allBaseCycles10` under the O-3 encoding; weight distribution {0: 1, 6: 45, 8: 90, 10: 1,818}.
- **Purpose.** Reproduces `baseCycle10_complete`, whose completeness content is the cardinality bound `baseCycle10Target_card_le`. The base floor needs only 002a and 002b; 002c serves the later Bravyi360 stages that use the weight-10 census and gives an independent check of the source's trellis count.
- **Method for review.** The same meet-in-the-middle with halves of weight ≤ 5 (Σ_{k≤5} C(90, k) ≈ 4.7 × 10⁷), with memory and deduplication planned before execution.
- **Level.** R for the reproduced census; V when a checker verifies the list's validity, distinctness and the completeness argument.

### 7.5 Scope kept for later review

- Level K inside RLT: whether GF(2) vectors of length 90 and the finite enumeration can be represented as RLT derivations under the existing kernel, at what scale, and with which registered definitions. The kernel stays unchanged in RLT-QEC-002.
- O-6 (axioms and import cone by a Lean build) and O-7 (candidate-data hashes) belong to the source audit in the research fork.
- O-8 (the second homology sector and X/Z duality) belongs to a later milestone on the middle and top floors.

**Placement.** Reusable GF2 specifications, verifier designs and evidence-level reports live in RLT; source-specific audit evidence lives in `thinkthoughts/bb-distance-lean`.

## 8. Architecture constraints observed

- RLT kernel preserved; code, dependencies and registries kept at the baseline.
- RLT stays independent of Lean and of Python certificate generation; source repositories were read at pinned commits with shallow fetches.
- No Lean build was run; every build-dependent finding is marked pending.
- Upstream repositories unmodified.

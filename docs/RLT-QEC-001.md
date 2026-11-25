# RLT-QEC-001 — Quantum Error Correction

**Status.** Specification registration, 2026-11-25. Documentation only: this milestone adds this document and the manifest `research/qec-001/MANIFEST.yaml`. The kernel, axioms, registries (`sources/registry.yaml`, `spec/`), certificates, records and website files stay exactly as they are at the baseline.

**Revision 2** (2026-11-25, after the audit `docs/RLT-QEC-001-AUDIT.md`): GF2-03 now states the source convention; §3.1 separates the 45 syndrome positions (C0) from the 45 face generators (C2) and records the coordinate reflection; the push theorems are cited for both quotient steps; the axiom report is scoped to the capstones that carry it; the QECLean `sorry` finding is corrected; O-2 and O-7 are restated; §7 is refined into RLT-QEC-002a, 002b and 002c with three evidence levels.

**Revision 3** (2026-11-25, after the independent audit of `a3daea2`, findings B1–B10 in `docs/RLT-QEC-001-AUDIT.md` §6): §2.1 registers the exact coordinate encoding and the three correspondences (block swap, reflection, dual complex); §3 corrects the floor path and two source locations and names all four data lists the floor uses; §5 scopes the kernel-replay report and the text search; §7 separates reviewed mathematical arguments from machine-checkable completeness certificates, adopts mask-set equality as the acceptance criterion, specifies orbit closure and deduplication, classifies the Lean result as K (Lean), and records the weakest-obligation composition rule. O-1 to O-8 stay open.

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

GF2-03 records the source convention. RLT may register this convention or another one; a different choice carries the explicit correspondence of §2.1, recorded under O-3 and O-4.

### 2.1 Coordinates and correspondences

Notation: X = `bbChainComplex A B` with the GF2-03 maps; R is reflection, (Rf)(g) = f(−g), which satisfies R(a⋆b) = Ra⋆Rb; σ is block swap on C1, σ(u, v) = (v, u). Each correspondence below is a weight-preserving linear bijection on C1, so it carries weights unchanged; what it carries cycles and boundaries *to* differs, and that difference decides which statements transfer.

| Correspondence | Maps | Relation to X | Transfers `LogicalFloor`? |
|---|---|---|---|
| **Block swap** σ | X′: ∂2′ f = (B⋆f, A⋆f), ∂1′ (u, v) = A⋆u + B⋆v | Isomorphism of chain complexes: ∂2′ = σ ∘ ∂2, ∂1 = ∂1′ ∘ σ, identity on C0 and C2. Cycles go to cycles and boundaries to boundaries. | Yes, directly: same homology sector. |
| **Reflection** R | X_R = `bbChainComplex (RA) (RB)`, with RA = x⁶ + y + y², RB = 1 + x⁵ + x⁴ | Isomorphism X ≅ X_R (R on every degree). R is an isomorphism onto a *different* complex: RA ≠ A, RB ≠ B, so R is no automorphism of X. | Yes, onto X_R; identical weight distributions, different masks. |
| **Dual complex** Xᵀ | ∂1ᵀ s = (RB⋆s, RA⋆s), ∂2ᵀ (u, v) = RA⋆u + RB⋆v (QECLean `BBDuality.lean`, `cutMap`, `dualBoundary`) | The transpose: a cochain complex C0 → C1 → C2 whose cocycles modulo coboundaries form the **other (Z) homology sector**. The map Φ = R ∘ σ, Φ(g, j) = (−g, 1 − j) (`blockSwapNeg`), sends cycles of X to cocycles of Xᵀ and boundaries to coboundaries, preserving weight (`bb_cycle_bound_iff_dual_bound`, `BBDuality.lean:359`). | Only through Φ, which is the X/Z duality of O-8. |

The word "transpose" in Revisions 1 and 2 named the block-swap convention X′. Revision 3 reserves *transpose* and *dual* for Xᵀ. X′ arises from Xᵀ by conjugating with R and exchanging the roles of C0 and C2; it is an isomorphic copy of X itself, by σ, and the sector question belongs to Φ and O-8.

**Registered coordinate encoding (source certificate coordinates).** For g = (x, y) ∈ Z15 × Z3 and block j ∈ {0, 1}:

    face(x, y)     = 3 · ((−x) mod 15) + ((−y) mod 3)        ∈ {0, …, 44}   (C0 and C2)
    qubit(x, y, j) = 45 · j + face(x, y)                      ∈ {0, …, 89}   (C1)
    mask(u)        = Σ_{k : u_k = 1} 2^k,  with k = qubit index (or face index on C0, C2)

This is `baseFaceEnum` and `baseQubitEnum` (`Defs.lean:62–76`) expanded through Mathlib's `finProdFinEquiv` ((a, b) ↦ b + n·a) and `maskOf` / `chainOfMask` (`F2Mask.lean:143–147`, bit k ↔ index k). Under this encoding a block-0 qubit at g meets the faces g + {0, (10,0), (11,0)} and a block-1 qubit meets g + {(9,0), (0,1), (0,2)}; the face at g covers block-0 qubits g + {(9,0), (0,1), (0,2)} and block-1 qubits g + {0, (10,0), (11,0)}. The audit of `a3daea2` compared these formulas against the source data (all 90 `cycleSyndrome` columns, all 45 `baseBoundaryWindow` masks, window entry i = face `baseFaceEnum.symm i`) and found agreement, and found disagreement for the unreflected encoding 3x + y. That comparison read source data; it is no RLT-QEC-002 evidence, and the encoding stays under O-3.

Under σ the qubit index becomes 45 · (1 − j) + face(x, y); under R the face index becomes 3 · (x mod 15) + (y mod 3). Any RLT convention other than the one above names its map to these indices before results are compared.

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
| Syndrome positions (C0, target of ∂1) | 45; each qubit meets 3 (`baseSyndromePoints`: (g,0) ↦ g + {0, (10,0), (11,0)}, (g,1) ↦ g + {(9,0), (0,1), (0,2)}) | `BaseSparse.lean:100` (`baseSyndromePoints`); packed columns `cycleSyndrome` (`ConcreteKernelSpan.lean:212`), each below 2⁴⁵ by `baseCensusSyndrome_bounded` (`BaseFloorChecks.lean:14`) |
| Face generators (C2, source of ∂2) | 45; each face row ∂2 δ_g has weight 6 (3 + 3) | `BaseSparse.lean`, `baseBoundary2_single_eq_faceRow` |
| Bit encoding | face(x, y) = 3·((−x) mod 15) + ((−y) mod 3); qubit(x, y, j) = 45·j + face(x, y); the same face index serves C0 and C2 (§2.1) | `Defs.lean:62–76` |
| Quotient chain | 360-qubit code over `ZMod 30 × ZMod 6` (A = x⁹+y+y², B = y³+x²⁵+x²⁶) → 180-qubit middle over `ZMod 30 × ZMod 3` → 90-qubit base over `ZMod 15 × ZMod 3` | `Defs.lean` |

The base polynomials agree with reducing the top polynomials (y³ ↦ 1 modulo 3; x²⁵, x²⁶ ↦ x¹⁰, x¹¹ modulo 15); in source, `top_push_A`/`top_push_B` state the top-to-middle step and `base_push_A`/`base_push_B` the middle-to-base step (`Defs.lean:91–97`, `decide +kernel`).

### 3.2 Reported certificate data and how the proof uses it

| Name | Reported | Read from source at the pin |
|---|---|---|
| `allBaseCycles10` | 1,954 packed binary masks | 1,954 entries, strictly increasing, all below 2⁹⁰; weight distribution {0: 1, 6: 45, 8: 90, 10: 1,818} (counted by a parser during this registration; pending independent recomputation) |
| `cycleSyndrome` | packed syndrome columns | 90 pairs (qubit index, 45-bit syndrome mask), `ConcreteKernelSpan.lean:212`; defines the census target through `reconX cycleSyndrome`; tied to ∂1 by `baseCensusSyndrome_columns` (`BaseFloorChecks.lean`) and `baseBoundary1_eq_reconLinear` (`BaseFloor.lean:22`) |
| `baseBoundaryWindow` | boundary membership data | 45 pairs (index, mask), `BaseCycleCensus.lean`; `baseBoundaryWindow_columns` (`BaseFloorChecks.lean:28`, `decide +kernel`) states each mask is a face check |
| `baseFloorPairs` | boundary witnesses | 46 pairs (census mask, coefficient), `BaseCycleCensus.lean`; each coefficient is 0 or a single window bit 2^i; read by `baseFloorCoefficient` (`BaseCycleCensus.lean:2154`) |
| `baseCycle10_complete` | census completeness theorem | `BaseCycleCensus.lean:2033` |
| Generator | `scripts/bravyi360/generate.py` | regenerates Lean certificates from checked-in candidate data (`scripts/bravyi360/certificates/*.gz`, hashes in `audit_manifest.json`); it replays the small parity searches from recorded forest data and takes the original large-distance search as recorded (O-7) |

**Structure of the completeness proof (read, not built).** `baseCycle10_complete` combines two facts:

1. `allBaseCycles10_checked` (`decide +kernel`): the list has length 1,954, is strictly ordered, and each entry is below 2⁹⁰, has zero syndrome and weight ≤ 10.
2. `baseCycle10Target_card_le`: the target set `baseCycle10Target = {w < 2⁹⁰ : syndrome(w) = 0 ∧ weight(w) ≤ 10}` has at most 1,954 elements. This bound comes from a separate trellis census (`cycle_card_le`, `cycle_kernel_to_trellis`, `cycle_permutation_injective`, in `ConcreteCensusBounds` / `ConcreteKernelSpan` / `ConcretePermutations`) after an injective coordinate permutation.

A sorted, valid list whose length meets the cardinality bound of the target equals the target (`ordered_census_complete`). The whole completeness content therefore sits in fact 2. An independent verification must reproduce fact 2 (or the census itself) by its own enumeration; re-checking the 1,954 masks reproduces fact 1 only.

**Path to the floor.** `base_cycle_lt_eight_boundary` (`BaseFloor.lean:63`) turns a physical cycle into a packed word with zero `reconX cycleSyndrome` syndrome (through `baseBoundary1_eq_reconLinear`) and applies `base_floor_eight` (`BaseCycleCensus.lean:2168`). That theorem places the word in the census by `baseCycle10_complete` and reads its witness from `baseFloor_checked`, which states that every census entry of weight < 8 equals `reconX baseBoundaryWindow (baseFloorCoefficient word)`. `baseBoundaryWindow_mem` makes each window mask a boundary, so the word is a boundary, and `base_logical_floor` follows. The floor uses only census entries of weight < 8: the zero mask and the 45 weight-6 masks. `base_cycle_mem_census` (`BaseFloor.lean:47`) sits beside this path and serves later stages that use the weight-8 and weight-10 entries (`MiddleFloor.lean:36`, `Diagonal.lean:43`, `DeepFlatReduction.lean:30`).

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
| Kernel replay of 39,227 declarations (`provenance/original-validation/`) | reported for the original QECLean layout, before this repository was extracted (`Bravyi360/README.md:59–61`; `provenance/module-migration.json` maps modules); it attests that earlier layout, and the pinned tree awaits its own replay or build; pending independent verification |
| Scope of the text searches above | `BBDistance/` at the bb-distance-lean pin and `QEC/` at the QECLean pin; Mathlib and its transitive dependencies sit outside the search and are taken at their pins as conventional trusted inputs, to be covered by the O-6 build |
| QECLean upstream HEAD | `47876ae` on 2026-11-25, ahead of the pin; the registration uses the pin `f51011b`, fixed by `lakefile.toml` and `lake-manifest.json` |

## 6. Unresolved verification obligations

| ID | Obligation | Target |
|---|---|---|
| O-1 | Independent census completeness: enumerate all cycles of weight ≤ 10 of the base complex from the registered definitions alone, and compare with `allBaseCycles10` (expected: equal sets, 1,954 elements). Minimum for the floor alone: all cycles of weight ≤ 7. | GF2-06, GF2-04, GF2-05 |
| O-2 | Independent boundary witnesses: for each census cycle of weight < 8, a coefficient vector c with ∂2 c = u, checked independently. Given O-1, a sufficient check is that the 45 weight-6 cycles are exactly the 45 face rows ∂2 δ_g and that the zero mask is ∂2 0. | GF2-04 |
| O-3 | Encoding correspondence: fix RLT's coordinate convention for G × Fin 2, the faces, and the bit order, and state its map to the registered source encoding of §2.1 (`baseQubitEnum`, `baseFaceEnum`, `reconX`, `cycleSyndrome`, reflected coordinates). The formulas are registered; discharge requires the correspondence to be checked inside 002a–002c. | GF2-01, GF2-03 |
| O-4 | Operator correspondence: show the independently defined ∂1, ∂2 equal QECLean's `bbChainComplex baseA baseB` maps (source theorem `baseBoundary1_eq_reconLinear` supplies the Lean side). | GF2-02, GF2-03 |
| O-5 | Definition correspondence: RLT-QEC-001a's floor statement versus `XDoubleCoverData.LogicalFloor` (which phrases the cycle condition with `bbBoundary1Fn D.Ab D.Bb`). | RLT-QEC-001a |
| O-6 | Trust boundary of the Lean result: axioms by `#print axioms` under the pinned toolchain, and the import cone of `base_logical_floor` free of `sorry` and `native_decide`, Mathlib included; the text search of §5 covers `BBDistance/` and `QEC/` only. | source audit |
| O-7 | Candidate-data provenance: hashes of `scripts/bravyi360/certificates/*` against `audit_manifest.json`; the candidate data comes from an original large-distance search that regeneration replays only in part (`scripts/bravyi360/README.md`: candidate certificates, small parity searches replayed, the large search left as recorded); recorded as external input outside the trusted proof. The qec-lab reference in `Defs.lean` concerns the matrix and coordinate convention. | source audit |
| O-8 | Sector coverage: identify the source theorem covering the second (Z) sector and how X/Z duality enters; recorded so the base floor is reported with its exact scope. Candidate: Φ = R ∘ σ with `bb_cycle_bound_iff_dual_bound` (`BBDuality.lean:359`, §2.1); its use in the Bravyi360 capstones is unconfirmed. | RLT-QEC-001a |

## 7. Proposed next milestone — RLT-QEC-002: independent verification of the base floor

### 7.1 Evidence levels, completeness bases and composition

**Levels.** Every RLT-QEC-002 result is reported at exactly one level.

| Level | Meaning | What it establishes | Trust rests on |
|---|---|---|---|
| **R — independent computational reproduction** | A program written independently of the Lean development and its generator recomputes a result from the registered definitions (GF2-01 to GF2-06) and obtains the same output. | Agreement of two independent computations; evidence for the source claim. | The reproducing program, its toolchain, any reviewed argument it implements, and the encoding correspondence (O-3). |
| **V — independent certificate verification** | A checker, separate from the program that produced the data, accepts a certificate against registered definitions. | The certificate is correct for the stated finite claim. | The checker (small and auditable, with its own correctness argument reviewed once) plus O-3. |
| **K — kernel-checked mathematical proof (RLT)** | RLT's kernel checks a derivation of the statement under registered axioms and definitions. | A theorem of RLT under those axioms. | The RLT kernel and its axioms. |

Ordering for composition: R < V < K.

**The Lean result.** `base_logical_floor` is classified **K (Lean)**: a proof reported as checked by Lean's kernel, pending O-6 (no build has been run). K (Lean) is a classification of external evidence and sits outside the R < V < K ordering of RLT results. Agreement between an RLT-QEC-002 result and the Lean result is reported as a correspondence (O-3 to O-5); it raises neither result's level. RLT-QEC-002 aims at levels R and V for the base floor; level K inside RLT is a later question (§7.5).

**Soundness and completeness are reported separately.** For a census L of a finite target T (GF2-06):

- *Soundness* (every element of L lies in T, and L is duplicate-free) is a property of the listed data. A checker evaluates it item by item, so soundness reaches V whenever a separate checker accepts the list.
- *Completeness* (every element of T lies in L) has one of two bases, recorded with every result:
  - **Reviewed mathematical argument.** A written argument, reviewed and recorded, shows that a procedure reaches every element of T (for example the splitting lemma, translation invariance and the representative lemma below). An argument is reviewed, never machine-checked by an RLT-QEC-002 checker. Completeness that rests on a reviewed argument executed by a reproducing program is reported at level **R**.
  - **Machine-checkable completeness certificate.** Finite data from which a small checker, by itself, derives |T| ≤ |L| (or T ⊆ L), with the checker's correctness argument reviewed once and the checker independent of the producer. The shape of the source's `baseCycle10Target_card_le` is an example: layered trellis tables whose local transitions a checker verifies and whose path count bounds |T| after a verified injective coordinate permutation. Soundness, distinctness and |L| meeting that bound then give T = L (as in `ordered_census_complete`). Only this basis brings completeness to level **V**.

  A meet-in-the-middle enumeration leaves no such certificate: its completeness is level R. Reaching V for 002a or 002c requires a completeness-certificate format designed and reviewed in that milestone. Independently checking the *source's* trellis certificate is level V verification of the source certificate, reported separately from an independent RLT enumeration (level R).

**Weakest-obligation composition rule.** A combined result is as strong as its weakest component:

1. Its level is the minimum of its components' levels under R < V < K.
2. Its open obligations are the union of its components' open obligations (O-1 to O-8 and any milestone-specific ones), and it stays *pending* while any of them is open.
3. Every reviewed argument any component relies on must be recorded as reviewed; an unreviewed argument leaves the combined result pending.
4. K (Lean) enters a combined RLT result only as a correspondence, never as a component level.

**Acceptance criterion for every comparison with the source: mask-set equality.** Each RLT vector is mapped to its source mask through the registered encoding of §2.1, and the two finite sets of masks are compared for equality (equal cardinalities and the same elements, for example by sorting both). Counts and weight distributions are reported as diagnostics only. They are unchanged by σ, by R and by every automorphism of the complex, so they can agree while the encoding is wrong; the audit of `a3daea2` confirmed that the unreflected encoding disagrees with the source data mask by mask.

**Orbit closure and deduplication.** For t ∈ G, translation T_t u (g, j) = u(g − t, j) commutes with ∂1 and ∂2, because convolution is translation-equivariant; so T_t preserves cycles, boundaries and weight. A search may restrict to representatives, for example vectors whose support contains a qubit with group coordinate 0 (every nonzero vector has such a translate). The procedure is then:

1. close the representatives under all 45 translations;
2. deduplicate by mask;
3. recheck the cycle condition and the weight bound on every element of the closed set.

Counts come from the deduplicated closed set, and never from the number of representatives times 45: orbit sizes divide 45, and the source census already shows orbits of smaller size at weight 10 (1,818 = 40 · 45 + 18). Translation-equivariance and the representative lemma are part of the reviewed argument.

**Meet-in-the-middle splitting.** Each u with |u| ≤ w equals h₁ + h₂ with disjoint supports, |h₁| = ⌈|u|/2⌉, |h₂| = ⌊|u|/2⌋, and equal syndromes ∂1 h₁ = ∂1 h₂. Matching all half-vector pairs with equal syndromes also yields overlapping pairs, whose sum is lighter than |h₁| + |h₂|, and yields the same u from several splits. Every match is therefore rebuilt as a mask, its weight and syndrome are rechecked, and the results are deduplicated by mask. A canonical split (h₁ takes the lowest ⌈|u|/2⌉ indices of the support) may be used to reduce duplicates; the deduplication step stays in either case. The splitting lemma is part of the reviewed argument.

### 7.2 RLT-QEC-002a — Independent enumeration of all cycles of weight less than eight

- **Statement.** The set Z₇ = {u ∈ GF(2)⁹⁰ : ∂1 u = 0, |u| ≤ 7} of the base complex, computed from A = x⁹ + y + y², B = 1 + x¹⁰ + x¹¹ over Z15 × Z3 under GF2-03.
- **Expected value (from the source census, pending).** 46 elements: the zero vector and 45 vectors of weight 6.
- **Method for review.** Meet-in-the-middle with halves of weight ≤ 4 and ≤ 3 (Σ_{k≤4} C(90, k) = 2,676,766 half-vectors), with the splitting, overlap recheck, deduplication and optional orbit closure of §7.1.
- **Level.** Soundness of the list: V when a separate checker accepts it (each vector a cycle of weight ≤ 7, masks distinct). Completeness: R, on a reviewed argument (splitting lemma, translation-equivariance, representative lemma). Completeness at V requires a completeness certificate designed in this milestone. The set Z₇ is reported at the minimum of the two (R unless a certificate is added).
- **Acceptance.** Mask-set equality, through §2.1, with the 46 entries of `allBaseCycles10` of weight below 8.

### 7.3 RLT-QEC-002b — Independent GF(2) boundary witness construction and verification

- **Statement.** Every u ∈ Z₇ lies in im ∂2, so `LogicalFloor 8` holds for the base complex.
- **Construction.** For each u, a coefficient vector c ∈ GF(2)⁴⁵ with ∂2 c = u, found by Gaussian elimination over GF(2) on the 90 × 45 matrix of ∂2 (or, given 002a's expected shape, c = δ_g for the 45 face rows and c = 0 for the zero vector).
- **Verification.** A checker evaluates ∂2 c and compares it with u, coordinate by coordinate; this check is independent of how c was found. The witnesses reach level V.
- **Composition.** The floor statement combines 002a (soundness and completeness), 002b (witnesses), O-3 and O-5. By the composition rule it is reported at the minimum level of those components, pending while any of the obligations is open: level R with 002a completeness at R, level V only with 002a completeness at V.
- **Correspondence.** Witnesses are compared with source `baseBoundaryWindow` and `baseFloorPairs` through §2.1 (each source coefficient is 0 or a single window bit); the floor definition's correspondence with `LogicalFloor` is O-5.

### 7.4 RLT-QEC-002c — Complete weight-ten census reproduction

- **Statement.** Z₁₀ = {u : ∂1 u = 0, |u| ≤ 10} has exactly 1,954 elements and equals `allBaseCycles10` under the §2.1 encoding.
- **Purpose.** Reproduces `baseCycle10_complete`, whose completeness content is the cardinality bound `baseCycle10Target_card_le`. The base floor needs only 002a and 002b; 002c serves the later Bravyi360 stages that use the weight-10 census and gives an independent check of the source's trellis count.
- **Method for review.** The same meet-in-the-middle with halves of weight ≤ 5 (Σ_{k≤5} C(90, k) = 46,626,034), with memory, overlap recheck, deduplication and orbit closure planned before execution; non-free translation orbits are expected at weight 10.
- **Level.** Soundness: V when a separate checker accepts the list. Completeness: R on the reviewed argument; V only with a completeness certificate (an independent trellis-style certificate, or an independent check of the source's trellis data, reported as verification of the source certificate).
- **Acceptance.** Mask-set equality with all 1,954 entries; the weight distribution {0: 1, 6: 45, 8: 90, 10: 1,818} is a diagnostic.

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

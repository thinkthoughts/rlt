# RLT-QEC-002b — Boundary witnesses and logical-floor correspondence (specification for review)

**Status.** Specification for review, 2026-11-25, on branch `rlt-qec-002b` (unmerged). Documentation only: this file is the whole change. Implementation starts after this specification is reviewed and approved.

**Baseline.** `main` at `3da96cd` (merge of RLT-QEC-002a).

**Builds on.** `docs/RLT-QEC-001.md` Revision 3 (GF2-01 to GF2-06, §2.1 coordinates and correspondences, §3 target, §7.1 evidence rules, §7.3 the 002b outline) and the accepted RLT-QEC-002a evidence: `research/qec-002a/Z7.txt`, SHA-256 `e2f8e4b6435f5b34c7044885c2d3fb9be5a89ccfb5dbaed0bf5f7d1522dfff2e`, 46 masks (the zero vector and 45 of weight 6), soundness **V**, completeness **R**, combined **R**.

**Preserved.** The RLT kernel, axioms and registries; CI and its gates; every file of RLT-QEC-002a, which 002b reads and leaves byte-identical; the upstream repositories.

**Goal.** Establish, at a stated evidence level, the statement F8: *every 1-cycle of the 90-qubit base complex that is not a boundary has weight at least 8*, and state exactly how F8 corresponds to the source theorem `base_logical_floor : baseCover.LogicalFloor 8`.

---

## 1. Definitions

Throughout, G = Z15 × Z3, GF(2) = Z/2, and X = `bbChainComplex A B` with A = x⁹ + y + y², B = 1 + x¹⁰ + x¹¹ (GF2-01 to GF2-03).

| Object | Definition |
|---|---|
| Chain spaces | C2 = GF(2)^G (45 faces), C1 = GF(2)^(G × {0,1}) (90 qubits), C0 = GF(2)^G (45 syndrome positions) |
| Convolution | (a⋆f)(h) = Σ_{g ∈ G} a(h − g) f(g) (GF2-02) |
| ∂2 : C2 → C1 | ∂2 f = (A⋆f, B⋆f): block 0 carries A⋆f, block 1 carries B⋆f (GF2-03) |
| ∂1 : C1 → C0 | ∂1 (u₀, u₁) = B⋆u₀ + A⋆u₁ (GF2-03) |
| Weight | \|u\| = #{q : u(q) = 1} |
| Cycles | Z₁ = ker ∂1 |
| Boundaries | B₁ = im ∂2 = {∂2 c : c ∈ C2} |
| Homology | H₁ = Z₁ / B₁; its elements are the logical classes of this (X) sector |
| Nontrivial logical class | a class [u] ≠ 0 in H₁, i.e. u ∈ Z₁ with u ∉ B₁; u is a *nontrivial cycle* |
| Logical-weight lower bound d | every nontrivial cycle u has \|u\| ≥ d (statement F_d) |
| Minimum logical weight d_min | min{\|u\| : u ∈ Z₁ ∖ B₁}, defined when H₁ ≠ 0 |

**Lower bound versus exact minimum.** F_d says d ≤ d_min (and holds vacuously when H₁ = 0). The exact value d_min = d needs, in addition, a nontrivial cycle of weight d. RLT-QEC-002b establishes the lower bound F8 only; §7.4 states what an exact minimum would require.

**Sector.** H₁ above is the homology of X: cycles of ∂1 modulo boundaries of ∂2. The other sector, cocycles of the dual complex Xᵀ modulo coboundaries (`docs/RLT-QEC-001.md` §2.1), is related to it by Φ = R ∘ σ (`blockSwapNeg`) only through the duality recorded in O-8. RLT-QEC-002b makes claims about the X sector alone.

---

## 2. Construction of ∂2 and coordinates

**Coordinates (registered, `docs/RLT-QEC-001.md` §2.1).** For g = (x, y) ∈ G and block j:

    face(x, y)     = 3 · ((−x) mod 15) + ((−y) mod 3)      C2 and C0 index, 0 … 44
    qubit(x, y, j) = 45 · j + face(x, y)                   C1 index, 0 … 89
    mask(v)        = Σ_{k : v_k = 1} 2^k

A coefficient vector c ∈ C2 is written as a 45-bit mask over face indices: bit i of c is c(g) for the g with face(g) = i.

**Columns of ∂2.** For the unit vector δ_g ∈ C2,

    ∂2 δ_g = (A⋆δ_g, B⋆δ_g),   A⋆δ_g = δ_{g+(9,0)} + δ_{g+(0,1)} + δ_{g+(0,2)},   B⋆δ_g = δ_{g} + δ_{g+(10,0)} + δ_{g+(11,0)},

so column i = face(g) of the 90 × 45 matrix M₂ of ∂2 is the mask with bits qubit(g+(9,0), 0), qubit(g+(0,1), 0), qubit(g+(0,2), 0), qubit(g, 1), qubit(g+(10,0), 1), qubit(g+(11,0), 1). Each column has weight 6 when the six points are distinct, which the implementation checks rather than assumes.

**Two constructions (implementation plan).**

| Program | Construction of ∂2 | Construction of ∂1 |
|---|---|---|
| `witness.py` (builder) | column masks from the supports of A and B, as above | — (reads none; uses only ∂2) |
| `check_witness.py` (checker) | the convolution sum (A⋆f)(h), (B⋆f)(h) evaluated over all 45 g, from coefficient functions of A and B, with its own decoder | the convolution sum B⋆u₀ + A⋆u₁ |

The checker shares no code with the builder or with RLT-QEC-002a programs. Both are written from §1 and §2.1.

---

## 3. Verification of ∂1 ∘ ∂2 = 0

Two separate items, reported separately:

| Item | Content | Basis |
|---|---|---|
| **C-1 (argument)** | ∂1 ∂2 f = B⋆(A⋆f) + A⋆(B⋆f) = (B⋆A + A⋆B)⋆f = 2(A⋆B)⋆f = 0, by associativity and commutativity of convolution on the abelian group G (GF2-02) and characteristic 2 | reviewed mathematical argument; matches the source's `bbChainComplex` proof obligation |
| **C-2 (computation)** | ∂1(∂2 δ_g) = 0 for all 45 unit vectors δ_g; by linearity this gives ∂1 ∘ ∂2 = 0 on C2 | checker evaluation with the checker's own ∂1 and ∂2 (§2) |

C-2 also cross-checks the two ∂2 constructions against the checker's ∂1, so a coordinate or block error in either construction shows up as a nonzero product. C-1 and C-2 support the floor argument as consistency checks; the floor argument of §7 uses only the definition of B₁ and the witnesses.

---

## 4. Construction of boundary witnesses

**Input.** `research/qec-002a/Z7.txt` at SHA-256 `e2f8e4b6…dfff2e`, read and hash-checked, never regenerated or modified. Its 45 nonzero masks are the nonzero elements of Z₇ (002a).

**Primary construction (independent of the expected answer).** For each u in `Z7.txt`, solve M₂ c = u over GF(2) by Gaussian elimination on the augmented 90 × 46 system, with pivot choice fixed (lowest row index, then lowest column index) so the output is deterministic. The solver returns a particular solution c_u, or reports *no solution*; it uses no knowledge that u is a face generator.

**Structural cross-check.** For each nonzero u, find the g with ∂2 δ_g = u (at most one, since the columns are distinct, which is checked); record whether c_u = δ_g. The zero vector has c = 0.

**Solution sets.** A witness is unique only up to ker ∂2: the full solution set of ∂2 c = u is c_u + ker ∂2. The builder computes a basis of ker ∂2 (from the same elimination) and reports dim ker ∂2 and rank ∂2 = 45 − dim ker ∂2. Acceptance never depends on which particular solution is chosen.

**Output.** `research/qec-002b/W.txt`: 46 lines `u c`, both canonical decimal masks, ascending by u, one line for every mask of `Z7.txt`; and `research/qec-002b/ker2.txt`: the kernel basis, one mask per line (possibly empty).

**Diagnostic D-1 (optional, reported separately).** rank ∂1, rank ∂2 and dim H₁ = 90 − rank ∂1 − rank ∂2, each with a rank certificate a checker verifies: a set of linearly independent columns with an explicit pivot structure (lower bound on rank) and a basis of the kernel whose size meets the bound (upper bound). D-1 says whether F8 is a statement about existing logical classes (dim H₁ > 0) or a vacuous one. F8 and its classification hold without D-1.

---

## 5. Witness checker

`check_witness.py W.txt Z7.txt [ker2.txt]` accepts only if all of the following hold, and otherwise rejects with the first failing condition:

| # | Condition | Rejection example |
|---|---|---|
| 1 | `Z7.txt` has SHA-256 `e2f8e4b6…dfff2e` | altered census file |
| 2 | every line of W is two canonical decimal integers (no sign, no leading zero except `0`, single space, final newline) | `07 3`, `1,2`, missing newline |
| 3 | 0 ≤ u < 2⁹⁰ and 0 ≤ c < 2⁴⁵ | c = 2⁴⁵ |
| 4 | the u column is strictly increasing and equals the 46 masks of `Z7.txt` as a set: every census mask has exactly one witness, and every witnessed u is a census mask | missing, duplicated or extra u |
| 5 | u is a cycle: ∂1 u = 0 (checker's convolution ∂1) | a non-cycle u (reachable only with an altered census, so tested with a test-only census) |
| 6 | **∂2 c = u**, evaluated by the checker's convolution ∂2 | wrong coefficient, a coefficient with one bit flipped, the coefficient of a neighbouring face |
| 7 | the zero vector's witness satisfies ∂2 c = 0 (the case u = 0 of condition 6, listed for its own regression test) | c with ∂2 c ≠ 0 for u = 0 |
| 8 | C-2: ∂1(∂2 δ_g) = 0 for all g | (consistency of the checker itself; a failure aborts with an internal-error status) |
| 9 | if `ker2.txt` is given: every listed vector k satisfies ∂2 k = 0, and the list is linearly independent (verified by elimination in the checker) | a non-kernel vector, a dependent list |

The checker establishes *boundary membership of every mask in `Z7.txt`*: Z₇ ⊆ B₁. Condition 9 certifies that ker2 is an independent subset of ker ∂2; whether it spans ker ∂2 belongs to D-1.

Exit codes: 0 ACCEPT, 1 REJECT, 2 usage or internal error.

---

## 6. Correspondence with the pinned source

All source reads go through `scripts/pinned_read.py` at `bb-distance-lean@a83cafad4d041427af4ddb54085493d3f9dd02d2` (QECLean definitions at `f51011bd6d1af9e225a19a5d5ff672000f3c9d94`), with SHA-256 pins per file. The source is read only by the comparison program, after construction and checking.

### 6.1 Boundary data

| Source object | Location | RLT counterpart | Acceptance |
|---|---|---|---|
| `baseBoundaryWindow : List (ℕ × ℕ)`, 45 pairs (index i, mask) | `BaseCycleCensus.lean:2053` | columns of M₂ | for every i, the source mask at index i equals RLT column i (index-by-index equality, which implies mask-set equality); 45 pairs, indices 0 … 44 each once |
| `baseFloorPairs : List (ℕ × ℕ)`, 46 pairs (census mask, coefficient) | `BaseCycleCensus.lean:2103` | `W.txt` | (a) the first components equal the masks of `Z7.txt` as a set; (b) for every pair (u, c_src), the RLT checker's ∂2 c_src = u; (c) diagnostic: c_src = c_u, or c_src − c_u ∈ ker ∂2 |
| `baseFloorCoefficient`, `reconX`, `baseFloor_checked` | `BaseCycleCensus.lean:2154–2163`, `BZCertificate.lean:109` | condition 6 | correspondence of meaning: `reconX window coeff` is the XOR of the window masks selected by the bits of coeff, i.e. ∂2 applied to coeff when window index i is face i; recorded as reviewed reading |
| `baseFaceRow`, `baseFacePoints`, `baseFaceMask`, `baseBoundary2_single_eq_faceRow`, `baseMaskOf_faceRow` | `BaseSparse.lean:18, 26, 46, 78, 84` | §2 columns | reviewed reading: the six points of `baseFacePoints g` are the six points of §2, block by block |

Item (b) runs the RLT checker on the *source's* witnesses, so it is level-V verification of the source certificate, reported separately from the RLT witnesses.

### 6.2 The definition `LogicalFloor`

Source (`QECLean`, `BBDoubling.lean:157`, instantiated at `baseCover`, `Defs.lean:115`):

    LogicalFloor d :=  ∀ u : H × Fin 2 → ZMod 2,
        bbBoundary1Fn D.Ab D.Bb u = 0 → u ∉ D.baseComplex.boundaries → d ≤ D.baseComplex.chainWeight u

RLT statement F_d: ∀ u ∈ C1, ∂1 u = 0 ∧ u ∉ B₁ ⟹ |u| ≥ d.

| Ingredient | Source | RLT | Correspondence item |
|---|---|---|---|
| H, chain space | `BaseGroup = ZMod 15 × ZMod 3`; `H × Fin 2 → ZMod 2` | G = Z15 × Z3; C1 as 90-bit masks | O-3 (registered encoding is a bijection; tested) |
| D.Ab, D.Bb | `baseCover.Ab = baseA = poly[x^9 + y + y^2]`, `Bb = baseB = poly[1 + x^10 + x^11]` (`Defs.lean:30, 32, 121–122`) | A, B | O-4: the `poly[…]` macro (`BBChainComplex.lean:111`) denotes the GF(2) sum of monomial indicators; the six monomials are distinct, so the supports are exactly those of §2 |
| cycle condition | `bbBoundary1Fn Ab Bb u = 0` with `bbBoundary1Fn c g = (B ⋆ leftHalf c) g + (A ⋆ rightHalf c) g` (`BBChainComplex.lean:394`) | ∂1 u = 0 | O-4 |
| D.baseComplex | `D.baseComplex := bbChainComplex D.Ab D.Bb` (`BBCover.lean:127`), which at `baseCover` is `bbChainComplex baseA baseB`, the source's `baseComplex` (`Defs.lean:39`; `base_logical_floor` passes between them by `change`) | X | O-4, O-5 |
| boundaries | `LinearMap.range X.boundary2` (`Code.lean:114`), `boundary2 = bbBoundary2 A B`, `bbBoundary2Fn f (h, j) = if j = 0 then (A ⋆ f) h else (B ⋆ f) h` (`BBChainComplex.lean:390`) | B₁ = im ∂2 | O-4 |
| chainWeight | `(Finset.univ.filter (· ≠ 0)).card` (`Distance.lean:29–33`) | \|u\| = popcount(mask u) | O-5 |
| d | `8` in `base_logical_floor` (`BaseFloor.lean:83`) | 8 | — |

Each row is checked by reading the pinned Lean text against the RLT definition (reviewed correspondence), and the computational rows (encoding, ∂1, ∂2) are additionally exercised by the comparisons of §6.1 and of RLT-QEC-002a. The correspondence relates two statements; it carries no evidence level and leaves the Lean proof's own status (K (Lean), pending O-6) unchanged.

---

## 7. From census and witnesses to the lower bound

### 7.1 Theorem (F8)

*If (i) every cycle of weight ≤ 7 lies in the list Z₇ of `Z7.txt`, and (ii) every element of Z₇ is a boundary, then every nontrivial cycle has weight ≥ 8.*

*Proof.* Let u ∈ Z₁ ∖ B₁ and suppose |u| ≤ 7. By (i), u ∈ Z₇; by (ii), u ∈ B₁, contradicting u ∉ B₁. Hence |u| ≥ 8. ∎

Premise (i) is the completeness of RLT-QEC-002a (level R). Premise (ii) is the checker's acceptance of `W.txt` (§5, conditions 4 and 6): for every u ∈ Z₇ a c with ∂2 c = u, so u ∈ B₁ by the definition of B₁ (level V). Soundness of Z₇ (002a, V) is not used by the implication; it confirms that the list contains only genuine cycles, so the witnesses are witnesses for cycles.

### 7.2 What F8 states and what it leaves open

- F8 is a **lower bound**: d_min ≥ 8 whenever H₁ ≠ 0, and F8 is vacuous when H₁ = 0 (D-1 reports which case holds).
- F8 concerns the **X sector of the 90-qubit base complex** only.
- F8 is the base-level ingredient of the Bravyi360 argument; the 360-qubit distance 24 rests on further middle- and top-level results (`docs/RLT-QEC-001.md` §4) outside this milestone.

### 7.3 Corollary (the source statement, by correspondence)

Under the correspondence of §6.2 (O-3, O-4, O-5), F8 is the statement `baseCover.LogicalFloor 8`. RLT-QEC-002b reports this as: *the RLT evidence for F8 corresponds to the statement of `base_logical_floor`*; the Lean proof of that theorem is a separate result, classified K (Lean), pending O-6.

### 7.4 Exact minimum (out of scope; requirements recorded)

d_min = 8 would additionally need a cycle u₈ of weight 8 with u₈ ∉ B₁. A checkable certificate for u₈ ∉ B₁ is a vector z ∈ C1 with z · ∂2 δ_g = 0 for all g (z annihilates B₁) and z · u₈ = 1. The weight-8 cycles come from RLT-QEC-002c (the source lists 90). This is recorded as a later item; RLT-QEC-002b makes no claim about d_min.

### 7.5 Z sector

The Z-sector floor (cocycles of Xᵀ modulo coboundaries) follows from F8 only through Φ = R ∘ σ and `bb_cycle_bound_iff_dual_bound` (`BBDuality.lean:359`), i.e. through O-8. RLT-QEC-002b leaves O-8 open and makes no Z-sector claim.

---

## 8. Evidence classification (weakest-obligation rule, Revision 3 §7.1)

| Claim | Components | Level | Open obligations / pending |
|---|---|---|---|
| ∂1 ∘ ∂2 = 0 | C-1 reviewed argument; C-2 checker evaluation | C-2: **V**; C-1: reviewed argument | review |
| Z₇ ⊆ B₁ (every census mask is a boundary) | `W.txt` accepted by `check_witness.py` | **V** | independent review of the checker |
| Source witnesses verified | `baseFloorPairs` accepted by the RLT checker (§6.1 b) | **V** (verification of the source certificate) | review |
| ∂2 columns equal the source window | index-by-index comparison (§6.1) | correspondence (no level) | review |
| **F8** for the base complex, X sector | 002a completeness (**R**) ∧ 002a soundness (V) ∧ witnesses (V) ∧ Theorem 7.1 (reviewed argument) | **R** (weakest component: 002a completeness) | reviews of 002b; O-3, O-4, O-5 for the correspondence |
| F8 ≙ `baseCover.LogicalFloor 8` | §6.2 | correspondence (no level) | O-3, O-4, O-5 reviewed |
| `base_logical_floor` (Lean) | Lean development | K (Lean), reported | O-6 |
| d_min = 8 | — | not claimed (§7.4) | — |
| Z-sector floor | — | not claimed (§7.5) | O-8 |

F8 reaches V only if 002a completeness reaches V through a machine-checkable completeness certificate (Revision 3 §7.1). That is a possible later extension and is outside 002b.

---

## 9. Acceptance criteria

RLT-QEC-002b is accepted when all of the following hold and have been independently reviewed:

1. `research/qec-002a/` is byte-identical to `3da96cd`; `Z7.txt` hash matches.
2. The builder's ∂2 columns are 45 distinct masks of weight 6.
3. C-2 holds in the checker; C-1 is reviewed.
4. `W.txt` has 46 lines, one per census mask, and `check_witness.py W.txt Z7.txt ker2.txt` returns ACCEPT.
5. The structural cross-check reports, for each nonzero u, the face g with ∂2 δ_g = u, and the zero vector has c = 0; any difference between c_u and δ_g lies in ker ∂2 (checked).
6. Source correspondence (§6.1): window equals the RLT columns index by index; the first components of `baseFloorPairs` equal `Z7.txt` as a set; every source witness passes condition 6 under the RLT checker.
7. The §6.2 correspondence table is reviewed row by row against the pinned Lean text.
8. The test suite passes, including: toy complexes where boundary membership is decided by brute force over all of C2 (2^(l·m) coefficient vectors for l·m ≤ 12) and compared with the elimination solver, both for u ∈ B₁ and for cycles u ∉ B₁ (the solver must report *no solution*); every rejection condition of §5; mutation checks (block swap in ∂2, unreflected encoding, a flipped witness bit, a dropped witness, a wrong census hash).
9. The document states the classification of §8, the lower-bound-only scope (§7.2, §7.4) and the X-sector-only scope (§7.5).
10. Kernel, registries, CI and existing gates are unchanged; the CI `check` job passes on the branch.

---

## 10. Obligations O-1 to O-8

| ID | Current status | Contribution of 002b | Remaining after 002b |
|---|---|---|---|
| O-1 | open; weight < 8 reproduced at R (002a) | input only | weight ≤ 10 (002c); completeness at V needs a completeness certificate |
| O-2 | open; 45 weight-6 cycles = face generators (002a test) | **main target**: witnesses for all 46 census masks, zero vector included, checker-verified (V); source witnesses verified (V) | review; then O-2 can be proposed for discharge at weight < 8 |
| O-3 | open; source encoding used as RLT's own (identity map) | encoding of C2 (coefficient bit i ↔ face i) and window index ↔ face index, tested against the source | review; encodings of 002c |
| O-4 | open; ∂1 columns equal the convolution definition (002a) | ∂2 by two constructions; C-1, C-2; window equality; `poly[…]`, `bbBoundary2Fn`, `bbBoundary1Fn`, `boundaries` readings | review |
| O-5 | open | §6.2: `LogicalFloor` ↔ F_d, `chainWeight` ↔ popcount, `baseCover` fields | review |
| O-6 | open | none (no Lean build) | Lean build with `#print axioms`, Mathlib included |
| O-7 | open | none | candidate-data hashes (source audit) |
| O-8 | open | none; §7.5 states the dependency | middle/top-floor milestone |

All eight remain open after 002b is merged; 002b supplies evidence toward O-2 to O-5 and changes the status of none of them without review.

---

## 11. Planned files (after approval)

| File | Role |
|---|---|
| `research/qec-002b/witness.py` | builder: ∂2 columns, elimination solver, structural cross-check, kernel basis, D-1 (optional) |
| `research/qec-002b/check_witness.py` | separate checker (§5); imports only the standard library |
| `research/qec-002b/compare_002b.py` | source correspondence (§6.1) via `scripts/pinned_read.py` |
| `research/qec-002b/test_qec002b.py` | tests of §9.8 |
| `research/qec-002b/W.txt`, `ker2.txt` | outputs |
| `research/qec-002b/MANIFEST.sha256`, `reproduce.sh`, `.gitignore` | as in 002a; Python 3.8 or later |
| `docs/RLT-QEC-002b.md` | this document, extended with results |

Same conventions as 002a: standard library only; `python3 -I`; no bytecode in the tree; source read only by the comparison program; outside CI unless a separate job is approved.

---

## 12. Questions for the reviewer

1. Is the witness acceptance criterion ∂2 c = u (condition 6), with any particular solution allowed, the right one, or should the checker also require c = δ_g for the face generators?
2. Should D-1 (rank and dim H₁ with certificates) be part of 002b, so F8 is reported together with whether it is vacuous, or deferred?
3. Should verification of the source witnesses (§6.1 b) be reported in 002b or kept for a separate source-certificate track?
4. Is the exact-minimum certificate of §7.4 (an annihilating z with z · u₈ = 1) the preferred form for a later milestone?

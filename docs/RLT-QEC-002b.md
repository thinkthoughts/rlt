# RLT-QEC-002b — Boundary witnesses and logical-floor correspondence

**Status.** Revision 3, 2026-11-25, on branch `rlt-qec-002b` (unmerged). Revision 1 (`70a1d9f`) was the specification for review, approved in principle with five resolutions; Revision 2 (`20561e0`) incorporated them with the implementation and results; a preliminary review of Revision 2 was favorable and asked for one classification refinement, made here (§13). Merge waits for independent specification, implementation, certificate and execution audits.

**Baseline.** `main` at `3da96cd` (merge of RLT-QEC-002a).

**Builds on.** `docs/RLT-QEC-001.md` Revision 3 (GF2-01 to GF2-06, §2.1 coordinates and correspondences, §3 target, §7.1 evidence rules, §7.3 the 002b outline) and the accepted RLT-QEC-002a evidence: `research/qec-002a/Z7.txt`, SHA-256 `e2f8e4b6435f5b34c7044885c2d3fb9be5a89ccfb5dbaed0bf5f7d1522dfff2e`, 46 masks (the zero vector and 45 of weight 6), soundness **V**, completeness **R**, combined **R**.

**Preserved.** The RLT kernel, axioms and registries; CI and its gates; every file of RLT-QEC-002a, which 002b reads and leaves byte-identical (checked by `reproduce.sh` and a test); the upstream repositories.

**Goal.** Establish, at a stated evidence level, the statement F8: *every 1-cycle of the 90-qubit base complex that is not a boundary has weight at least 8*; certify the ranks of both boundary maps and the number of homology classes, so that F8 is shown to be nonvacuous; and state exactly how F8 corresponds to the source theorem `base_logical_floor : baseCover.LogicalFloor 8`.

---

## 1. Definitions

Throughout, G = Z15 × Z3, GF(2) = Z/2, and X = `bbChainComplex A B` with A = x⁹ + y + y², B = 1 + x¹⁰ + x¹¹ (GF2-01 to GF2-03).

| Object | Definition |
|---|---|
| Chain spaces | C2 = GF(2)^G (45 faces), C1 = GF(2)^(G × {0,1}) (90 qubits), C0 = GF(2)^G (45 syndrome positions) |
| Convolution | (a⋆f)(h) = Σ_{g ∈ G} a(h − g) f(g) (GF2-02) |
| ∂2 : C2 → C1 | ∂2 f = (A⋆f, B⋆f): block 0 carries A⋆f, block 1 carries B⋆f (GF2-03) |
| ∂1 : C1 → C0 | ∂1 (u₀, u₁) = B⋆u₀ + A⋆u₁ (GF2-03) |
| Weight | wt(u) = #{q : u(q) = 1} |
| Cycles | Z₁ = ker ∂1 |
| Boundaries | B₁ = im ∂2 = {∂2 c : c ∈ C2} |
| Homology | H₁ = Z₁ / B₁, of dimension k; its 2^k elements are the logical classes of this (X) sector, the trivial class included |
| Nontrivial logical class | a class [u] ≠ 0 in H₁, i.e. u ∈ Z₁ with u ∉ B₁; u is a *nontrivial cycle* |
| Logical-weight lower bound d | every nontrivial cycle u has wt(u) ≥ d (statement F_d) |
| Minimum logical weight d_X | min{wt(u) : u ∈ Z₁ ∖ B₁}, defined when k ≥ 1: the minimum weight of a nontrivial cycle in the X sector of the 90-qubit base complex |

**Lower bound, upper bound, exact minimum.** F_d says d ≤ d_X when k ≥ 1, and holds vacuously when k = 0. A nontrivial cycle of weight w certifies d_X ≤ w. The exact value d_X = d needs a nontrivial cycle of weight exactly d together with F_d (§7.4). RLT-QEC-002b establishes the lower bound F8, its nonvacuity and the upper bound d_X ≤ 10; the exact minimum stays open.

**d_X versus code distance.** d_X is an X-sector quantity of the 90-qubit base complex. The quantum code distance of that base code is min(d_X, d_Z), with d_Z the minimum weight of a nontrivial class in the other sector (§7.5, O-8); the 360-qubit Bravyi360 code has its own distance 24 (`docs/RLT-QEC-001.md` §4). RLT-QEC-002b states results about d_X only.

**Sector.** H₁ above is the homology of X: cycles of ∂1 modulo boundaries of ∂2. The other sector, cocycles of the dual complex Xᵀ modulo coboundaries (`docs/RLT-QEC-001.md` §2.1), is related to it by Φ = R ∘ σ (`blockSwapNeg`) only through the duality recorded in O-8. RLT-QEC-002b makes claims about the X sector alone.

---

## 2. Construction of ∂2 and ∂1, and coordinates

**Coordinates (registered, `docs/RLT-QEC-001.md` §2.1).** For g = (x, y) ∈ G and block j:

    face(x, y)     = 3 · ((−x) mod 15) + ((−y) mod 3)      C2 and C0 index, 0 … 44
    qubit(x, y, j) = 45 · j + face(x, y)                   C1 index, 0 … 89
    mask(v)        = Σ_{k : v_k = 1} 2^k

A coefficient vector c ∈ C2 is a 45-bit mask over face indices: bit i of c is c(g) for the g with face(g) = i.

**Columns of ∂2.** For the unit vector δ_g ∈ C2,

    ∂2 δ_g = (A⋆δ_g, B⋆δ_g),   A⋆δ_g = δ_{g+(9,0)} + δ_{g+(0,1)} + δ_{g+(0,2)},   B⋆δ_g = δ_{g} + δ_{g+(10,0)} + δ_{g+(11,0)},

so column i = face(g) of the 90 × 45 matrix M₂ of ∂2 has bits qubit(g+(9,0), 0), qubit(g+(0,1), 0), qubit(g+(0,2), 0), qubit(g, 1), qubit(g+(10,0), 1), qubit(g+(11,0), 1). The builder checks that the 45 columns are distinct masks of weight 6.

**Columns of ∂1.** Column qubit(g, 0) of the 45 × 90 matrix M₁ has the face bits of g + {0, (10,0), (11,0)}; column qubit(g, 1) has those of g + {(9,0), (0,1), (0,2)}.

**Two constructions.**

| Program | ∂2 | ∂1 |
|---|---|---|
| `witness.py` (builder) | column masks from the supports of A and B | column masks from the supports |
| `check_witness.py` (checker) | the convolution sums (A⋆f)(h), (B⋆f)(h) over all 45 g, with its own decoder | the convolution sum B⋆u₀ + A⋆u₁ |

The checker imports only `hashlib`, `json`, `re` and `sys`, and shares no code with the builder or with RLT-QEC-002a. The builder's ∂1 equals the accepted 002a column construction (test `test_d1_equals_accepted_002a_columns`), and both programs' ∂1 and ∂2 agree on every unit vector (tests). Builder and checker were written by one author in one session; independence of authorship arrives with the independent review.

---

## 3. Verification of ∂1 ∘ ∂2 = 0

| Item | Content | Basis |
|---|---|---|
| **C-1 (argument)** | ∂1 ∂2 f = B⋆(A⋆f) + A⋆(B⋆f) = (B⋆A + A⋆B)⋆f = 2(A⋆B)⋆f = 0, by associativity and commutativity of convolution on the abelian group G (GF2-02) and characteristic 2 | reviewed mathematical argument |
| **C-2 (computation)** | ∂1(∂2 δ_g) = 0 for all 45 unit vectors δ_g; by linearity ∂1 ∘ ∂2 = 0 on C2 | checker condition 8, with the checker's own ∂1 and ∂2 |

C-2 is also the hypothesis B₁ ⊆ Z₁ used by the dimension count of §4.3.

---

## 4. Construction

### 4.1 Boundary witnesses

**Input.** `research/qec-002a/Z7.txt`, read and hash-checked, never regenerated or modified.

**Construction.** For each u in `Z7.txt`, solve M₂ c = u over GF(2) by fully reduced column elimination (columns in index order; a new pivot takes the lowest set row of the reduced column; earlier basis vectors are cleared on that row). The solver returns a particular solution or reports *no solution*; it uses no knowledge that u is a face generator.

**Acceptance (review resolution 1).** A witness for u is any c with ∂2 c = u. The solution set is c + ker ∂2, and the checker accepts every element of it; c = δ_g is not required.

**Structural cross-check (separate).** For each nonzero u, the face g with ∂2 δ_g = u (the columns are distinct, so g is unique), and whether the RLT witness equals δ_g (`equal`) or differs from it by an element of ker ∂2 (`kernel`). Output `structure.txt`.

### 4.2 Rank certificates (review resolution 2)

For a GF(2) matrix M with n columns and rank r, the certificate has two parts, both checked by the checker against its own matrix entries:

| Bound | Certificate | Check | Reason |
|---|---|---|---|
| rank M ≥ r | row indices R, column indices C (each r distinct), and N (r × r) | M[R, C] · N = I | an invertible r × r minor |
| rank M ≤ r | free column indices D (n − r distinct) and kernel vectors K₁ … K_{n−r} | M K_t = 0, and K_t restricted to D is the unit vector at D_t | n − r independent kernel vectors (independence is read off the unit pattern on D), so dim ker M ≥ n − r |

Together they fix rank M = r exactly. Certificates are produced for M₂ (90 × 45) and M₁ (45 × 90).

### 4.3 Homology dimension and nonvacuity

**Dimension.** With B₁ ⊆ Z₁ (C-2), dim H₁ = dim Z₁ − dim B₁ = (90 − rank ∂1) − rank ∂2, so

    k = 90 − rank(∂1) − rank(∂2),   #H₁ = 2^k classes, the trivial class included.

F8 is nonvacuous exactly when k ≥ 1.

**Nonvacuity witness and upper bound.** A pair (u, h) of vectors in C1 with ∂1 u = 0, hᵀ∂2 = 0 and hᵀu = 1 over GF(2), stated together with wt(u). Then u ∈ Z₁, and u ∉ B₁ because hᵀ(∂2 c) = (hᵀ∂2) c = 0 for every c while hᵀu = 1; so [u] is a nontrivial class, and d_X ≤ wt(u). The checker recomputes wt(u) and rejects a stated weight that differs from it. The builder takes u as the first ∂1-kernel vector of the rank certificate (in its fixed order) that the solver reports outside im ∂2, and h as the first vector of a basis of the left kernel of M₂ with hᵀu = 1. It searches for no particular weight. This witness establishes nonvacuity independently of the dimension count; the two are cross-checked (k ≥ 1).

### 4.4 Outputs

| File | Content |
|---|---|
| `W.txt` | 46 lines `u c`, canonical decimal masks, ascending by u, one per census mask |
| `ranks.json` | rank certificates for ∂2 and ∂1, k, the number of homology classes, the nonvacuity witness u, h (masks as decimal strings) with its weight (an integer) |
| `structure.txt` | 45 lines `u i x y equal\|kernel` |

---

## 5. Checker

`check_witness.py W.txt Z7.txt ranks.json structure.txt` accepts only if all conditions hold, and otherwise rejects with the first failing condition (exit 0 ACCEPT, 1 REJECT, 2 usage):

| # | Condition | Rejection examples (tested) |
|---|---|---|
| 1 | `Z7.txt` has the accepted SHA-256 | an altered census |
| 2 | every W line is two canonical decimal integers, single space, final newline | `0 00`, `0  0`, `0,0`, `-0 0`, missing newline |
| 3 | u < 2⁹⁰, c < 2⁴⁵ | c = 2⁴⁵ |
| 4 | u column strictly increasing and equal to the census as a set | a dropped, duplicated or reordered line |
| 5 | ∂1 u = 0 | (unreachable with the hash-pinned census; the function is tested) |
| 6 | **∂2 c = u** | a flipped coefficient bit, coefficient 0 for a nonzero u; a kernel-shifted coefficient is *accepted* |
| 7 | the zero vector's witness satisfies ∂2 c = 0 (case u = 0 of 6) | c = 1 for u = 0 |
| 8 | C-2: ∂1(∂2 δ_g) = 0 for all g | (consistency of the checker) |
| 9 | ∂2 rank certificate (§4.2), schema exact | rank changed, inverse altered, duplicate row index, repeated kernel vector, missing field |
| 10 | ∂1 rank certificate (§4.2) | rank changed, a kernel vector moved off the kernel |
| 11 | k = 90 − rank ∂1 − rank ∂2 and homology classes = 2^k as stated | k = 7; 128 classes |
| 12 | nonvacuity and upper bound: ∂1 u = 0, hᵀ∂2 = 0, hᵀu = 1, the stated weight is an integer equal to wt(u) as recomputed by the checker, and k ≥ 1 | u off the kernel; h with hᵀu = 1 but hᵀ∂2 ≠ 0; u a boundary; missing weight; stated weight 8 (lighter than wt(u)), 11, `"10"` or `true`; a boundary-shifted u with its recomputed weight is *accepted* |
| 13 | structure: each nonzero census mask equals ∂2 δ_g for its listed face, and `equal`/`kernel` is reported correctly | swapped face, misreported relation, missing line |

---

## 6. Correspondence with the pinned source

All source reads go through `scripts/pinned_read.py` at `bb-distance-lean@a83cafad4d041427af4ddb54085493d3f9dd02d2` (QECLean definitions at `f51011bd6d1af9e225a19a5d5ff672000f3c9d94`), file SHA-256 `838c7f4a…5924cb` for `BBDistance/Bravyi360/BaseCycleCensus.lean`. Only `compare_002b.py` reads source data, after construction and checking, and it evaluates ∂2 with the checker's convolution, never with the builder.

### 6.1 Boundary data: three separate reports (review resolution 3)

| Report | Source object | Acceptance | Result |
|---|---|---|---|
| **A — indexed boundary-column correspondence** | `baseBoundaryWindow` (`BaseCycleCensus.lean:2053`), 45 pairs (i, mask) | indices 0 … 44 each once; mask i = ∂2 δ_g with face(g) = i | **EQUAL** |
| **B — source-provided witnesses** | `baseFloorPairs` (`BaseCycleCensus.lean:2103`), 46 pairs (u, c_src) | first components equal `Z7.txt` as a set; ∂2 c_src = u for every pair, by the RLT checker's ∂2 (verification of the source certificate) | **VERIFIED**, 46/46 |
| **C — coefficient comparison (diagnostic)** | `baseFloorPairs` against `W.txt` | none: equality of coefficient vectors is not required | 42 equal; 4 differ by an element of ker ∂2; 0 other |

Report C shows the expected effect of a nonzero ker ∂2 (dimension 4): for four masks the eliminated witness and the source witness are different elements of the same solution set c + ker ∂2, and both are valid.

**Reviewed readings** (no computation): `baseFloorCoefficient`, `reconX` and `baseFloor_checked` (`BaseCycleCensus.lean:2154–2163`, `BZCertificate.lean:109`): `reconX window coeff` is the XOR of the window masks selected by the bits of coeff, i.e. ∂2 applied to coeff when window index i is face i (report A); `baseFaceRow`, `baseFacePoints`, `baseFaceMask`, `baseBoundary2_single_eq_faceRow`, `baseMaskOf_faceRow` (`BaseSparse.lean:18, 26, 46, 78, 84`): the six points of `baseFacePoints g` are the six points of §2, block by block.

### 6.2 The definition `LogicalFloor`

Source (`QECLean`, `BBDoubling.lean:157`, instantiated at `baseCover`, `Defs.lean:115`):

    LogicalFloor d :=  ∀ u : H × Fin 2 → ZMod 2,
        bbBoundary1Fn D.Ab D.Bb u = 0 → u ∉ D.baseComplex.boundaries → d ≤ D.baseComplex.chainWeight u

RLT statement F_d: ∀ u ∈ C1, ∂1 u = 0 ∧ u ∉ B₁ ⟹ wt(u) ≥ d.

| Ingredient | Source | RLT | Correspondence item |
|---|---|---|---|
| H, chain space | `BaseGroup = ZMod 15 × ZMod 3`; `H × Fin 2 → ZMod 2` | G = Z15 × Z3; C1 as 90-bit masks | O-3 (registered encoding is a bijection; tested) |
| D.Ab, D.Bb | `baseCover.Ab = baseA = poly[x^9 + y + y^2]`, `Bb = baseB = poly[1 + x^10 + x^11]` (`Defs.lean:30, 32, 121–122`) | A, B | O-4: the `poly[…]` macro (`BBChainComplex.lean:111`) denotes the GF(2) sum of monomial indicators; the six monomials are distinct, so the supports are exactly those of §2 |
| cycle condition | `bbBoundary1Fn c g = (B ⋆ leftHalf c) g + (A ⋆ rightHalf c) g` (`BBChainComplex.lean:394`) | ∂1 u = 0 | O-4 |
| D.baseComplex | `D.baseComplex := bbChainComplex D.Ab D.Bb` (`BBCover.lean:127`), which at `baseCover` is the source's `baseComplex := bbChainComplex baseA baseB` (`Defs.lean:39`; `base_logical_floor` passes between them by `change`) | X | O-4, O-5 |
| boundaries | `LinearMap.range X.boundary2` (`Code.lean:114`), `bbBoundary2Fn f (h, j) = if j = 0 then (A ⋆ f) h else (B ⋆ f) h` (`BBChainComplex.lean:390`) | B₁ = im ∂2 | O-4 |
| chainWeight | `(Finset.univ.filter (· ≠ 0)).card` (`Distance.lean:29–33`) | wt(u) = popcount(mask u) | O-5 |
| d | `8` in `base_logical_floor` (`BaseFloor.lean:83`) | 8 | — |

Each row is checked by reading the pinned Lean text against the RLT definition (reviewed correspondence); the computational rows are also exercised by report A and by RLT-QEC-002a. The correspondence relates two statements; it carries no evidence level, and the Lean proof's own status stays K (Lean), pending O-6.

---

## 7. From census and witnesses to the lower bound

### 7.1 Theorem (F8)

*If (i) every cycle of weight ≤ 7 lies in the list Z₇ of `Z7.txt`, and (ii) every element of Z₇ is a boundary, then every nontrivial cycle has weight ≥ 8.*

*Proof.* Let u ∈ Z₁ ∖ B₁ and suppose wt(u) ≤ 7. By (i), u ∈ Z₇; by (ii), u ∈ B₁, contradicting u ∉ B₁. Hence wt(u) ≥ 8. ∎

Premise (i) is the completeness of RLT-QEC-002a (level R). Premise (ii) is the checker's acceptance of `W.txt` (conditions 4, 6): for every u ∈ Z₇ a c with ∂2 c = u, so u ∈ B₁ by the definition of B₁ (level V). Soundness of Z₇ (002a, V) confirms that the list holds only genuine cycles. The ranks, k and the nonvacuity witness enter neither premise.

### 7.2 Scope

- F8 is a **lower bound**: d_X ≥ 8. With k = 8 certified (§8), H₁ has 256 classes, 255 of them nontrivial, so F8 is **nonvacuous**: it constrains the weights of existing nontrivial cycles.
- The nonvacuity witness gives the **upper bound** d_X ≤ 10 (§4.3, level V). Together, **8 ≤ d_X ≤ 10**, at level R by the weakest-obligation rule (§8).
- F8 and the interval concern the **X sector of the 90-qubit base complex** only; they are statements about d_X, separate from the base code's quantum distance min(d_X, d_Z) and from the 360-qubit distance.
- F8 is the base-level ingredient of the Bravyi360 argument; the 360-qubit distance 24 rests on further middle- and top-level results (`docs/RLT-QEC-001.md` §4).

### 7.3 Corollary (the source statement, by correspondence)

Under the correspondence of §6.2 (O-3, O-4, O-5), F8 is the statement `baseCover.LogicalFloor 8`. RLT-QEC-002b reports: *the RLT evidence for F8 corresponds to the statement of `base_logical_floor`*. The Lean proof of that theorem is a separate result, K (Lean), pending O-6.

### 7.4 Exact-minimum certificate (registered; discovery outside 002b — review resolution 4)

**Registered form.** An exact-minimum certificate for d is a pair (u, h) of vectors in C1 = GF(2)⁹⁰ satisfying, over GF(2),

    ∂1 u = 0,   wt(u) = d,   hᵀ ∂2 = 0,   hᵀ u = 1.

As in §4.3, it shows u is a nontrivial cycle of weight d, so d_X ≤ d; combined with F_d, d_X = d. Checker condition 12 verifies exactly these four relations for a stated weight.

**Status in 002b.** Finding a certificate with d = 8 is outside this milestone, and no search for a light cycle was made. The nonvacuity witness of §4.3 is a certificate of this form with d = 10: it is registered as the **upper bound d_X ≤ 10 at level V**. With F8 it leaves 8 ≤ d_X ≤ 10, i.e. d_X ∈ {8, 9, 10}, an interval at level R.

### 7.5 Z sector

The Z-sector floor (cocycles of Xᵀ modulo coboundaries) follows from F8 only through Φ = R ∘ σ and `bb_cycle_bound_iff_dual_bound` (`BBDuality.lean:359`), i.e. through O-8. RLT-QEC-002b leaves O-8 open and makes no Z-sector claim. The certified k concerns H₁ of X; the equality of the two sectors' dimensions is likewise left to O-8.

---

## 8. Results and evidence classification

**Results.**

| Quantity | Value |
|---|---|
| ∂2 columns | 45 distinct masks of weight 6 |
| ∂1 ∘ ∂2 | 0 (C-2, 45 faces) |
| Witnesses | 46/46 census masks (zero vector: c = 0) |
| Structure | every nonzero census mask is a face generator ∂2 δ_g; RLT witness `equal` to δ_g for 41, `kernel`-shifted for 4 |
| rank ∂2 | **41** (certified: inverted 41 × 41 minor; 4 unit-form kernel vectors) |
| rank ∂1 | **41** (certified: inverted 41 × 41 minor; 49 unit-form kernel vectors) |
| dim ker ∂2 | 4 |
| k = 90 − 41 − 41 | **8** |
| Homology classes | **256** (the trivial class and 255 nontrivial) |
| Nonvacuity witness | (u, h) verified; wt(u) = 10, recomputed by the checker |
| Bounds on d_X | 8 ≤ d_X ≤ 10 |
| Source report A | EQUAL (45/45 indexed columns) |
| Source report B | VERIFIED (46/46 source witnesses; census set equal) |
| Source report C (diagnostic) | 42 equal coefficients; 4 kernel-shifted |
| SHA-256 `W.txt` | `d9d9cf96da969c488a6d25600b4223dd3906677e58279b95825405ec2891f2b4` |
| SHA-256 `ranks.json` | `be4b70353afcdd47ce84c06d8393820afb911b38236e4df711a14df073bbf175` |
| SHA-256 `structure.txt` | `803e50d79913bccf20ac7ef18d285551cf9ad004b3dc0b00398080cf066a942e` |

**Classification (weakest-obligation rule, `docs/RLT-QEC-001.md` §7.1; review resolution 5).**

| Claim | Components | Level | Pending |
|---|---|---|---|
| ∂1 ∘ ∂2 = 0 | C-2 checker evaluation; C-1 argument | **V** (C-2); C-1 reviewed argument | review |
| Z₇ ⊆ B₁ | `W.txt` accepted (conditions 4, 6) | **V** | review |
| rank ∂2 = 41, rank ∂1 = 41 | certificates accepted (conditions 9, 10) | **V** | review |
| k = 8, 256 classes | ranks (V) + C-2 (V) + dimension argument §4.3 (reviewed) | **V** | review |
| Nonvacuity: a nontrivial class exists | witness (u, h) accepted (condition 12); consistent with k ≥ 1 | **V** | review |
| **d_X ≤ 10** | witness (u, h) with checker-recomputed wt(u) = 10 (condition 12) | **V** | review |
| Source witnesses verified | report B | **V** (verification of the source certificate) | review |
| ∂2 columns = source window | report A | correspondence (no level) | review |
| **F8** for the base complex, X sector | 002a completeness (**R**) ∧ 002a soundness (V) ∧ Z₇ ⊆ B₁ (V) ∧ Theorem 7.1 (reviewed argument) | **R** (weakest component: 002a completeness) | review; O-3, O-4, O-5 for the correspondence |
| F8 ≙ `baseCover.LogicalFloor 8` | §6.2 | correspondence (no level) | O-3, O-4, O-5 |
| `base_logical_floor` (Lean) | Lean development | K (Lean), reported | O-6 |
| **8 ≤ d_X ≤ 10** | F8 (**R**) ∧ d_X ≤ 10 (V) | **R** (weakest component: 002a completeness) | as for F8 |
| d_X = 8 (exact) | — | not claimed (§7.4) | certificate discovery (later milestone) |
| quantum distance of the base code, min(d_X, d_Z) | — | not claimed | O-8 |
| Z-sector floor | — | not claimed (§7.5) | O-8 |

The certified ranks, k, nonvacuity and the upper bound are separate results; they enter neither premise of Theorem 7.1 and leave the completeness of the 002a census, and therefore F8 and the interval, at level **R**. F8 reaches V only if the 002a completeness reaches V through a machine-checkable completeness certificate (Revision 3 §7.1), a possible later extension outside 002b.

---

## 9. Acceptance criteria

RLT-QEC-002b is accepted when all of the following hold and have been independently reviewed and execution-audited:

1. `research/qec-002a/` is byte-identical to `3da96cd` (its `MANIFEST.sha256` passes) and `Z7.txt` has the accepted hash.
2. The builder's ∂2 columns are 45 distinct masks of weight 6, and both programs' ∂1 and ∂2 agree on every unit vector.
3. C-2 holds in the checker; C-1 is reviewed.
4. `W.txt` has 46 lines, one per census mask, and `check_witness.py W.txt Z7.txt ranks.json structure.txt` returns ACCEPT; any witness c with ∂2 c = u is acceptable.
5. The structural cross-check identifies the face generator of every nonzero census mask and reports `equal`/`kernel` correctly (checker condition 13).
6. Rank certificates for ∂2 and ∂1 are accepted, fixing both ranks exactly; k and the number of homology classes are reported and verified; the nonvacuity witness is accepted with its weight recomputed, k ≥ 1, and d_X ≤ 10 is registered at V.
7. Source reports A (EQUAL) and B (VERIFIED) hold; report C is printed as a diagnostic and imposes no condition.
8. The §6.2 correspondence table is reviewed row by row against the pinned Lean text.
9. The test suite passes (§10), including brute-force toy complexes and every rejection condition, and every recorded mutation is detected.
10. The document states the classification of §8 (F8 and 8 ≤ d_X ≤ 10 at R; ranks, k, nonvacuity and d_X ≤ 10 at V, without elevating completeness), the absence of an exact-minimum claim (§7.4), the X-sector-only scope and the distinction between d_X and code distance (§1, §7.5).
11. Kernel, registries, CI and existing gates are unchanged; the CI `check` job passes on the branch.

---

## 10. Tests

`python3 -I research/qec-002b/test_qec002b.py` runs 29 tests (26 offline; 3 more with `QEC_SRC`); all pass on Python 3.8.20 and 3.13.18.

| Group | Tests |
|---|---|
| Maps | builder ∂2 and ∂1 equal the checker's convolution on all unit vectors; builder ∂1 equals the accepted 002a columns; ∂2 columns equal the registered six points; ∂1 ∘ ∂2 = 0 in both programs |
| Elimination | 60 random matrices up to 9 × 9: rank, solvability of every right-hand side and every certificate against brute force (certificates verified with the checker's `check_rank`) |
| Toy complexes | six small bicycle complexes: rank ∂2 = log₂ #B₁ and rank ∂1 = n − log₂ #Z₁ by enumerating all of C2 and C1; k against #Z₁/#B₁; the solver finds a witness exactly for the cycles in B₁ and reports *no solution* for the rest; certificates verified; cases include k = 0 and k ≥ 1 |
| Outputs | `witness.py` reproduces the committed outputs byte for byte; ranks 41, 41, k = 8, 256 classes; nonvacuity cycle of weight ≥ 8 (consistency with F8) and stated weight 10 equal to its popcount; 002a manifest passes |
| Checker | acceptance, and the rejection examples of §5 for conditions 1–4, 6, 7, 9–13; a kernel-shifted witness and a boundary-shifted nonvacuity cycle are accepted |
| Independence | no source names in `witness.py` or `check_witness.py`; checker imports exactly `hashlib`, `json`, `re`, `sys` and loads no code dynamically; builder imports only the standard library |
| Source (`QEC_SRC`) | reports A EQUAL, B VERIFIED, C printed; kernel-shifted RLT witnesses still pass (coefficient equality unrequired); an altered census is detected |

**Mutation checks** (on copies, during development; recorded, outside the suite). Each fault makes the suite fail:

| Fault | Detected by |
|---|---|
| block swap in the builder's ∂2 | output reproduction, structure, nonvacuity tests (8 tests) |
| unreflected face encoding in the builder | the same group (9 tests) |
| pivot row left uneliminated in the solver | random-matrix and toy brute force, output reproduction |
| witness condition removed from the checker | condition 6 and 7 rejection tests |
| rank lower-bound check removed from the checker | rank-certificate rejection test |
| hᵀ∂2 = 0 check removed from the checker | nonvacuity rejection test (h with hᵀu = 1 and hᵀ∂2 ≠ 0), added after this mutation first survived |
| weight check removed from the checker (Revision 3) | four nonvacuity rejection cases (weight 8, 11, `"10"`, `true`) |
| block swap in the checker's ∂2 | 11 tests (k, nonvacuity, structure, witnesses) |

---

## 11. Reproduction

```sh
research/qec-002b/reproduce.sh                     # offline: manifests, builder reproduction, checker, tests
research/qec-002b/reproduce.sh /abs/path/bb        # plus source reports A–C and source tests
PYTHON=python3.8 research/qec-002b/reproduce.sh /abs/path/bb
```

The pinned source checkout is prepared as for RLT-QEC-002a (`docs/RLT-QEC-002a.md` §9). Requirements: Python 3.8 or later, standard library only; `sha256sum`, `cmp`; `git` for the source reports. `reproduce.sh` sets `PYTHONDONTWRITEBYTECODE`, and the programs set `sys.dont_write_bytecode` where they load modules, so no bytecode enters the tree. `research/qec-002b/MANIFEST.sha256` pins seven files. RLT-QEC-002b runs outside CI; `ci.yml` and `verify.sh` are unchanged.

---

## 12. Obligations O-1 to O-8

All eight remain open.

| ID | Status before | Contribution of 002b | Remaining |
|---|---|---|---|
| O-1 | weight < 8 reproduced at R (002a) | input only; certified ranks leave completeness at R | weight ≤ 10 (002c); completeness at V needs a completeness certificate |
| O-2 | 45 weight-6 cycles = face generators (002a test) | witnesses for all 46 census masks, zero vector included, at V; source witnesses verified at V; face-generator structure checked | review; then O-2 can be proposed for discharge at weight < 8 |
| O-3 | source encoding used as RLT's own | C2 encoding (coefficient bit i ↔ face i) and window index ↔ face index tested against the source (report A) | review; encodings of 002c |
| O-4 | ∂1 columns equal the convolution definition (002a) | ∂2 and ∂1 by two constructions each; C-1, C-2; report A; readings of `poly[…]`, `bbBoundary2Fn`, `bbBoundary1Fn`, `boundaries` | review |
| O-5 | open | §6.2: `LogicalFloor` ↔ F_d, `chainWeight` ↔ weight, `baseCover` fields | review |
| O-6 | open | none (no Lean build) | Lean build with `#print axioms`, Mathlib included |
| O-7 | open | none | candidate-data hashes (source audit) |
| O-8 | open | none; §7.5 states the dependency | middle/top-floor milestone |

---

## 13. Review record

**Revision 1** (`70a1d9f`): specification for review, approved in principle with five resolutions, all incorporated here.

| # | Resolution | Where |
|---|---|---|
| 1 | Accept any witness c with ∂2 c = u; c = δ_g is not required; face-generator correspondence kept as a separate structural check | §4.1, §5 conditions 6 and 13, tests |
| 2 | Independently computed ranks of ∂1 and ∂2 with machine-checkable certificates of both bounds; k = 90 − rank ∂1 − rank ∂2; number of homology classes including the trivial class; nonvacuity of F8 established explicitly | §4.2, §4.3, §5 conditions 9–12, §8 |
| 3 | Source-witness verification in 002b, kept apart from the independently constructed witnesses and from the indexed column correspondence; no equality of coefficient vectors required | §6.1 reports A, B, C |
| 4 | Exact-minimum certificate registered as (u, h) with ∂1 u = 0, wt(u) = d, hᵀ∂2 = 0, hᵀu = 1; discovery outside 002b | §7.4 |
| 5 | F8 at level R under the weakest-obligation rule; certified ranks and nonvacuity leave census completeness at R | §7.1, §8 |

**Revision 2** (`20561e0`): preliminary review favorable — rank certificates and the direct nonvacuity pair accepted; the separation of independent witnesses, source-witness verification and coefficient diagnostics accepted. One refinement:

| # | Request | Resolution (Revision 3) |
|---|---|---|
| 6 | The nonvacuity pair with a weight-10 cycle certifies d_X ≤ 10; register it at V once its weight is independently checked, adding the check and a rejection test if needed; keep 8 ≤ d_X ≤ 10 at R; no light-cycle search; keep the X-sector minimum distinct from full quantum-code distance | `ranks.json` states the weight; checker condition 12 recomputes it and rejects a mismatch (four rejection cases; mutation detected); d_X ≤ 10 at V and 8 ≤ d_X ≤ 10 at R (§7.2, §7.4, §8); d_X defined and separated from code distance (§1) |

**Outstanding before merge:** independent specification, implementation, certificate and execution audits.

# RLT-QEC-002a — Independent enumeration of all cycles of weight less than eight

**Status.** Implementation and evidence for review, 2026-11-25, on branch `rlt-qec-002a` (unmerged). A preliminary independent review of `aef75a6` was favorable and requested five changes, made in Revision 2 of this milestone (§12). The independent execution audit and the complete diff review remain outstanding; merge waits for them.

**Baseline.** `main` at `d8d0252` (RLT-QEC-001 Revision 3, with §8 of the audit record identifying the Revision 3 confirming read as a self-check).

**Specification.** `docs/RLT-QEC-001.md` Revision 3: GF2-01 to GF2-06 (§2), coordinates and correspondences (§2.1), the base complex (§3.1), and the RLT-QEC-002a statement, evidence levels, completeness bases, composition rule, acceptance criterion and enumeration procedure (§7.1, §7.2).

**Scope.** New files only: this document and `research/qec-002a/` (enumerator, checker, comparison, tests, result, manifest, reproduction script). The RLT kernel, axioms, registries, certificates, CI, website files and the upstream repositories stay unchanged. No Lean build ran.

---

## 1. Statement and inputs

Z₇ = {u ∈ GF(2)⁹⁰ : ∂1 u = 0, |u| ≤ 7} for the Bravyi360 base complex X = `bbChainComplex A B` over G = Z15 × Z3, with

- A = x⁹ + y + y², B = 1 + x¹⁰ + x¹¹ (registered, §3.1);
- ∂1 (u, v) = B⋆u + A⋆v (GF2-03, source convention);
- the registered source encoding (§2.1): face(x, y) = 3·((−x) mod 15) + ((−y) mod 3), qubit(x, y, j) = 45·j + face(x, y), bit k of a mask ↔ index k.

RLT-QEC-002a adopts the registered source encoding as its own coordinate convention, so the RLT-to-source map of O-3 is the identity on indices for this milestone.

The enumerator `census.py` takes exactly these inputs, written into the program. It reads no source certificate, census, window or candidate list; `test_qec002a.py` (`Independence`) checks the code of `census.py` and `check.py` for source names and checks the checker's imports.

## 2. Results

| Quantity | Value |
|---|---|
| \|Z₇\| | **46** |
| Weight distribution (diagnostic) | {0: 1, 6: 45} |
| The 45 weight-6 cycles | exactly the 45 face generators ∂2 δ_g (`test_weight_six_entries_are_the_face_generators`) |
| Result file | `research/qec-002a/Z7.txt`, 46 decimal masks, ascending |
| SHA-256 of `Z7.txt` | `e2f8e4b6435f5b34c7044885c2d3fb9be5a89ccfb5dbaed0bf5f7d1522dfff2e` |
| Route 1 and route 2 | identical output |
| Checker | ACCEPT (46 masks, cycles of weight ≤ 7, strictly increasing, closed under 45 translations, zero vector present) |
| Source comparison | **EQUAL**: the mask set equals the 46 entries of weight ≤ 7 of `allBaseCycles10` (`BBDistance/Bravyi360/BaseCycleCensus.lean` at `a83cafad4d041427af4ddb54085493d3f9dd02d2`, file SHA-256 `838c7f4a93e65491738c903e2c3961777ec652f8dab8a2c8c997bc39555924cb`) |
| Search accounting | route 1: 2,676,766 S-halves (Σ_{k≤4} C(90, k)) against a table of 121,576 T-halves (Σ_{k≤3} C(90, k)); route 2: for each j, 117,570 S-halves against 117,570 T-halves (Σ_{k≤3} C(89, k)) |
| Run time | census 1.7 s; full `reproduce.sh` with source 12.6 s (Python 3.13.18, Linux x86-64) |

The odd weights 1, 3, 5, 7 and the weights 2 and 4 carry zero cycles; the weight-6 class is a single free translation orbit of size 45.

## 3. Algorithm

**Boundary columns.** For each qubit index k, `column[k]` is the 45-bit syndrome mask of the unit vector e_k: a block-0 qubit at g contributes B⋆δ_g, with support g + {0, (10,0), (11,0)}, and a block-1 qubit contributes A⋆δ_g, with support g + {(9,0), (0,1), (0,2)}, each face h recorded at bit face(h). The syndrome of a mask is the XOR of its columns.

**Route 1 — canonical meet-in-the-middle.** For W = 7:

1. Build a table from syndrome to the subsets T of {0, …, 89} with |T| ≤ ⌊W/2⌋ = 3, each with its least index.
2. For every subset S with |S| ≤ ⌈W/2⌉ = 4, with max(S) (or −1 for S = ∅), look up syndrome(S) and keep each T with |T| ∈ {|S|, |S| − 1} and, when T is nonempty, min(T) > max(S). Emit S ∪ T.
3. Require that no vector is emitted twice (a property of the canonical split; a repeat stops the program).
4. Rebuild and recheck: every emitted mask has weight ≤ W and syndrome 0; deduplicate by mask; sort.

**Route 2 — translation representatives and closure.** For each block j ∈ {0, 1}, let q₀ = qubit(0, 0, j) (indices 0 and 45):

1. Run the same canonical meet-in-the-middle over the 89 indices other than q₀ for vectors v with |v| ≤ W − 1 = 6 and syndrome(v) = column[q₀]; each u = v + e_{q₀} is a representative.
2. Close the set of representatives under all 45 translations T_t (g, j) ↦ (g + t, j); add the zero vector.
3. Deduplicate by mask; recheck weight ≤ W and syndrome 0 on every element of the closed set; sort.

Counts come from the closed, deduplicated set (Revision 3 §7.1). The program compares the two routes and stops if they differ.

**Two routes, one implementation.** Routes 1 and 2 are separate enumeration *strategies* inside one program by one author. They share the column construction and encoding (`Complex`), the half-vector table (`_half_table`), the meet-in-the-middle matcher (`_mitm`) and the recheck (`_recheck`). Their agreement therefore tests the parts where they differ: the representative lemma, translation closure and deduplication, and `_mitm` run with two different targets (0 and `column[q₀]`) and weight bounds (7 and 6) over two index sets. A fault in a shared component could affect both routes alike. Three checks address the shared components:

| Shared component | Check outside the two routes |
|---|---|
| `Complex` columns and encoding | `check.py`'s convolution evaluation equals the columns for all 90 unit vectors; the registered encoding values are tested directly; the source comparison is EQUAL |
| `_mitm`, `_half_table` | both routes equal brute-force enumeration on six small complexes (the brute force in the tests sums the same columns, so it covers the matcher and split logic, and the column test above covers the columns) |
| `_recheck` | `check.py` re-derives weight and boundary for every output mask by a separate computation |

An independently authored enumerator for Z₇ remains open; RLT-QEC-002a has a single enumeration implementation, and its completeness classification (R, §8) rests on §4 and on review of that implementation.

## 4. Completeness argument (reviewed mathematical argument, pending independent review)

Write supp(u) ⊆ {0, …, 89} for the set of qubit indices of u, |u| for its size, and syn(u) = ∂1 u, a GF(2)-linear map; masks identify vectors with subsets.

**Lemma 1 (canonical split).** Let u satisfy |u| = w ≤ W. Let S be the ⌈w/2⌉ least elements of supp(u) and T = supp(u) ∖ S. Then |S| ≤ ⌈W/2⌉, |T| ≤ ⌊W/2⌋, |T| ∈ {|S|, |S| − 1}, max(S) < min(T) when T is nonempty, u = e_S + e_T, and syn(u) = 0 if and only if syn(e_S) = syn(e_T).

*Proof.* The size bounds follow from w ≤ W and |T| = ⌊w/2⌋. The elements of S are the least ones, so every element of T exceeds every element of S. S and T partition supp(u), so u = e_S + e_T; linearity over GF(2) gives syn(u) = syn(e_S) + syn(e_T), which is 0 exactly when the two syndromes are equal. ∎

**Lemma 2 (route 1 emits exactly Z_W, each element once).** *Every element is emitted:* for u ∈ Z_W, the pair (S, T) of Lemma 1 passes every test of step 2, so u is emitted. *Only elements are emitted:* an emitted S ∪ T has disjoint parts (max(S) < min(T)), so it is the vector e_S + e_T of weight |S| + |T| ≤ ⌈W/2⌉ + ⌊W/2⌋ = W, and equal syndromes give syndrome 0. *Once:* if an emitted pair (S, T) gives u, then S and T partition supp(u), |T| ∈ {|S|, |S| − 1} forces |S| = ⌈w/2⌉, and max(S) < min(T) forces S to be the ⌈w/2⌉ least elements; so (S, T) is the canonical split of u, and u arises from one pair. ∎

The recheck of step 4 re-derives the weight and syndrome conditions of Lemma 2 for every emitted mask, independently of the lemma.

**Lemma 3 (translation-equivariance).** For t ∈ G and a, f : G → GF(2), (a⋆T_t f)(h) = Σ_g a(h − g) f(g − t) = Σ_{g′} a(h − t − g′) f(g′) = (a⋆f)(h − t). Hence ∂1 T_t = T_t ∂1 on C1 (with T_t acting on both blocks), T_t preserves |u|, and T_t maps Z_W onto Z_W. ∎

**Lemma 4 (representatives).** Every nonzero u ∈ Z_W has a translate whose support contains qubit (0, 0, j) for some j: choose (g, j) in supp(u) and translate by −g. ∎

**Lemma 5 (route 2 emits exactly Z_W).** For fixed j and q₀ = qubit(0, 0, j), the cycles u ∈ Z_W with q₀ ∈ supp(u) correspond bijectively to vectors v = u + e_{q₀} supported off q₀ with |v| = |u| − 1 ≤ W − 1 and syn(v) = syn(e_{q₀}) = column[q₀]. Lemma 1, applied to v and with the target column[q₀] in place of 0 (syn(v) = column[q₀] if and only if syn(e_S) + column[q₀] = syn(e_T)), shows that step 1 emits every such v, so step 1 yields every cycle of Z_W containing q₀. By Lemma 4 every nonzero element of Z_W is a translate of one of these, and by Lemma 3 every translate of a representative lies in Z_W; with the zero vector added, the closure is exactly Z_W. ∎

**Theorem.** Each route outputs Z₇ exactly. ∎

**Correspondence between argument and program.** The argument covers the algorithm of §3; the programs carry it out. Their agreement rests on reading `census.py` against §3 and on the tests of §7, which compare both routes with brute-force enumeration on six small bicycle complexes covering odd weights, non-free translation orbits and odd and even weight bounds.

## 5. Soundness check

`check.py` is a separate program that imports only `re` and `sys`. It decodes each mask with its own decoder written from the registered formula, evaluates ∂1 by the convolution definition (B⋆u)(h) = Σ_g B(h − g) u(g) over all 45 group elements (GF2-02, GF2-03), and accepts a list only if every line is a canonical decimal below 2⁹⁰, the list is strictly increasing, every mask has weight ≤ 7 and zero boundary, the set is closed under the 45 translations, and the zero vector is present (first line 0). Its column computation agrees with the enumerator's for all 90 unit vectors (`test_columns_equal_convolution_definition`).

The checker establishes soundness and orbit closure of the list and the presence of the trivial cycle 0. The zero-vector requirement checks one known element of Z₇; it leaves the classification unchanged (soundness at V), and completeness rests on §4.

**Authorship.** The enumerator, the checker, the comparison and the tests were written in one session by one author. They are separate programs sharing no code, and the checker follows the definitions by a different computation (convolution sums against column XOR). Independence of authorship arrives with the independent review.

## 6. Acceptance: mask-set equality

`compare.py` is the only RLT-QEC-002a program that reads source data, and it runs after the enumeration. It reads `BBDistance/Bravyi360/BaseCycleCensus.lean` at the full pinned commit through `scripts/pinned_read.py` (which re-hashes every Git object from commit to blob and ignores the checkout's own metadata), requires the file's SHA-256 to equal the pin, parses `allBaseCycles10`, requires its length to be 1,954 (`allBaseCycles10_length`), and compares the entries of weight ≤ 7 with `Z7.txt` as sets. Result: **EQUAL**, 46 masks.

Weight distributions are printed as diagnostics. `test_unreflected_encoding_detected_despite_equal_weights` re-encodes Z₇ in the unreflected coordinates 45·j + 3x + y: the weight distribution stays {0: 1, 6: 45} and the comparison reports DIFFERENT, as Revision 3 §7.1 anticipates.

## 7. Tests

`python3 -I research/qec-002a/test_qec002a.py` runs 21 tests (18 offline; 3 more with `QEC_SRC` set to a source checkout). All pass on Python 3.8.20, 3.11.18, 3.12.3 and 3.13.18.

| Group | Tests |
|---|---|
| Encoding | registered values (face(1, 0) = 42, face(14, 0) = 3, face(0, 1) = 2, qubit(0, 0, 1) = 45, …); bijection onto 0…89 and inverse; enumerator and checker encodings agree |
| Boundary map | enumerator columns equal the checker's convolution for all 90 unit vectors; every column has weight 3; translation-equivariance on 40 random vectors, in both implementations |
| Toy completeness | routes 1 and 2 equal brute force on six complexes (Z3×Z3, Z4×Z2, Z3×Z2, Z5×Z1, Z4×Z3, Z2×Z2) for every weight bound up to 5–8; the cases include odd-weight cycles and non-free orbits |
| Census | routes agree; 46 masks with weights [0, 6 × 45]; `Z7.txt` reproduced byte for byte; CLI output and printed hash |
| Checker | accepts `Z7.txt`; the 45 weight-6 entries equal the 45 face generators built in the test; rejects a dropped element (closure), a duplicate, an unsorted list, a non-cycle, a mask ≥ 2⁹⁰, a weight-12 orbit (accepted with `--wmax 12`), a leading zero, a missing zero vector, a missing final newline; regression test `test_zero_vector_required`: the sound, closed list without 0 is rejected with a reason naming the zero vector, and so is the empty list |
| Independence | no source names in the code of `census.py` or `check.py`; `check.py` imports only `re`, `sys` |
| Source (with `QEC_SRC`) | EQUAL; a dropped mask and an added mask detected; the unreflected encoding detected with equal weights |

**Mutation checks** (run during development on copies; recorded here, not part of the suite): each of four deliberate faults in `census.py` makes the suite fail.

| Fault | Detected by |
|---|---|
| even splits only (\|T\| = \|S\|) | toy brute force, route agreement, CLI reproduction |
| split order unenforced (overlapping halves) | the enumerator's duplicate guard |
| translation closure removed | toy brute force, route agreement, CLI reproduction |
| unreflected face encoding with reflected decoding | the enumerator's recheck |

## 8. Evidence classification (Revision 3 §7.1)

| Claim | Basis | Level | Pending |
|---|---|---|---|
| Soundness of `Z7.txt` (each element a cycle of weight ≤ 7, no repeats, translation-closed) | `check.py` accepts the list | **V** | independent review of `check.py` |
| Completeness of `Z7.txt` (every element of Z₇ listed) | reviewed mathematical argument §4, executed by `census.py` (two routes) | **R** | independent review of §4 and of `census.py` against §3 |
| Z₇ = the 46 masks of `Z7.txt` | composition of the two rows | **R** (weakest component) | both reviews |
| Agreement with the source census at weight ≤ 7 | mask-set equality, `compare.py` | correspondence (no level) | review of `compare.py` |
| The Lean result `base_logical_floor` | Lean development | K (Lean), reported | O-6 |

Completeness at level V would need a machine-checkable completeness certificate (Revision 3 §7.1); RLT-QEC-002a has none, so its completeness stays at R. The logical floor itself needs the boundary witnesses of RLT-QEC-002b and is outside this milestone's claims.

## 9. Reproduction

```sh
git clone https://github.com/thinkthoughts/rlt && cd rlt && git checkout rlt-qec-002a
research/qec-002a/reproduce.sh                  # offline: manifest, both routes, checker, tests

# with the pinned source (read-only)
mkdir -p /tmp/bb && git -C /tmp/bb init -q
git -C /tmp/bb fetch -q --depth 1 https://github.com/QECLean/bb-distance-lean a83cafad4d041427af4ddb54085493d3f9dd02d2
research/qec-002a/reproduce.sh /tmp/bb          # adds the mask-set comparison and source tests
```

Requirements: **Python 3.8 or later**, standard library only; `sha256sum`, `cmp`. `reproduce.sh` uses `python3`, or the interpreter in `PYTHON` (`PYTHON=python3.8 research/qec-002a/reproduce.sh …`), and prints the version it ran.

| Python | Full `reproduce.sh` with source |
|---|---|
| 3.8.20 | 7 PASS, EQUAL |
| 3.11.18 | 7 PASS, EQUAL |
| 3.12.3 | 7 PASS, EQUAL |
| 3.13.18 | 7 PASS, EQUAL (development interpreter) |

Static analysis (`vermin` 1.9.2) places the language and library features at 3.7 (`subprocess.run(capture_output=…)`, ordered `dict`); 3.7 is past end of life and was unavailable for testing, so the supported minimum is the tested 3.8, enforced by a version guard in `census.py`, `check.py` and `compare.py`. Individual steps: `census.py [--route 1|2|both] [--out FILE]`, `check.py FILE`, `compare.py CHECKOUT FILE`. `research/qec-002a/MANIFEST.sha256` pins the five files.

## 10. Obligations O-1 to O-8

All eight stay open; this milestone contributes evidence to four of them (O-1, O-2, O-3, O-4).

| ID | Status after RLT-QEC-002a | Remaining |
|---|---|---|
| O-1 | open. Weight < 8: independent census at level R, equal to the source as mask sets. | independent review; weight ≤ 10 in RLT-QEC-002c |
| O-2 | open. The 45 weight-6 cycles are exactly the 45 face generators (test), the sufficient check of O-2 given O-1. | witness verification in RLT-QEC-002b, zero vector included |
| O-3 | open. RLT-QEC-002a uses the registered source encoding as its own, so its map to the source is the identity; mask-set equality holds under it and fails under the unreflected encoding. | review; the encodings of 002b and 002c |
| O-4 | open. ∂1: the column construction equals the convolution definition for all 90 unit vectors. | ∂2 and the correspondence with the source operators in RLT-QEC-002b |
| O-5 | open | RLT-QEC-002b |
| O-6 | open | Lean build (Mathlib included) |
| O-7 | open | source audit (hashes) |
| O-8 | open | middle/top-floor milestone |

## 11. For the reviewer

1. Read §4 against §3 and `census.py` (`_mitm`, `route1`, `route2`, `_recheck`).
2. Read `check.py` as a standalone program against GF2-02, GF2-03 and §2.1.
3. Read `compare.py` and the pin constants; run `reproduce.sh` with a source checkout.
4. Confirm the classification of §8 and the obligation statuses of §10.
5. Confirm the diff from `main` adds files under `docs/` and `research/qec-002a/` only.

## 12. Review record

**Preliminary independent review of `aef75a6`** (2026-11-25): favorable. The completeness argument of §4 (Lemmas 1–5) is accepted at the algorithm-description level; the inspected implementation supports the intended enumeration, separate convolution-based soundness checking and exact mask-set comparison. Outstanding: the independent execution audit of the reproduction suite and the complete diff review.

| Item | Request | Resolution (Revision 2) |
|---|---|---|
| 1 | §10 said three obligations receive evidence; the table shows four | §10 names four: O-1, O-2, O-3, O-4 |
| 2 | Require the zero vector in `check.py`, with a regression test, keeping the soundness-only classification | requirement 6 in `check.py`; `test_zero_vector_required` plus a rejection case; §5 and §8 keep soundness at V and completeness at R |
| 3 | Confirm the minimum supported Python version | Python 3.8: full suite run on 3.8.20, 3.11.18, 3.12.3, 3.13.18; static floor 3.7 recorded; version guard added (§9) |
| 4 | Distinguish separate enumeration routes from independently authored implementations (both routes share `_mitm`) | §3, "Two routes, one implementation": shared components, what route agreement tests, the outside checks of shared components, and the open independently authored enumerator |
| 5 | Record the CI integration status without altering existing kernel verification gates | §13 |

Classifications after Revision 2: soundness V, completeness R, Z₇ as a whole R; O-1 to O-8 open; the RLT kernel unchanged.

## 13. CI integration status

RLT-QEC-002a runs outside CI. `.github/workflows/ci.yml` and `scripts/verify.sh` are unchanged on this branch, so the existing gates (formatting, Clippy, the kernel test suite and verification run by `scripts/verify.sh`, and the Lean/GDT comparison job) run exactly as on `main`. The entry point for this milestone is `research/qec-002a/reproduce.sh`, run by hand or by a reviewer, with the source comparison enabled by passing a bb-distance-lean checkout.

A later change may add a separate job that runs `reproduce.sh` (about 12 s, standard library only, plus a depth-1 fetch of the pinned source commit), kept apart from the kernel gates; that change would be reviewed on its own.

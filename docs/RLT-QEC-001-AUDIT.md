# RLT-QEC-001 — Audit of commit `ac1d378`

**Status.** Documentation audit, 2026-11-25. Subject: `ac1d378` on branch `rlt-qec-001` (`docs/RLT-QEC-001.md`, `research/qec-001/MANIFEST.yaml`). Corrections are in the commit that adds this record (Revision 2 of both files). §6–§8 record the independent audit of `a3daea2` (findings B1–B10), Revision 3 and its confirming read.

**Auditor independence.** The session that wrote `ac1d378` performed this audit, so it is a self-audit. An adversarial read-only audit by a fresh session remains the acceptance step before merge, following RLT practice.

**Method.** Each claim was re-read against the pinned sources by shallow fetch: `bb-distance-lean` at `a83cafad4d041427af4ddb54085493d3f9dd02d2`, `Stavan-Jain/QECLean` at `f51011bd6d1af9e225a19a5d5ff672000f3c9d94`. Text search and file reading only; no Lean build, no enumeration, no implementation.

## 1. Requirements coverage

| Requirement (RLT-QEC-001 brief) | Result |
|---|---|
| Branch `rlt-qec-001`, following documentation conventions | Met. Milestones in RLT are recorded as `docs/RLT-*.md`; this milestone follows that form. Machine registries (`sources/registry.yaml`, `spec/definitions-registry.yaml`) hold checked excerpts and kernel definitions, so QEC entries there belong to an implementation milestone. |
| Register RLT-QEC-001, 001a, 001b | Met (§1). |
| Register GF2-01 to GF2-06 | Met (§2); GF2-03 corrected (finding A1). |
| Initial certificate target and reported data | Met (§3); one table row corrected (A2). |
| Source pins marked pending unless confirmed from source | Met; the axiom row is rescoped (A4). |
| Weight-8 floor distinguished from weight-24 distance | Met (§4). |
| Source and dependency manifest | Met; three fields corrected (A7). |
| Unresolved obligations | Met (O-1 to O-8); O-2 and O-7 restated (A5, A6). |
| Proposed next milestone | Met; refined into 002a, 002b, 002c (§7). |
| Kernel, dependencies, upstream preserved; no builds | Met: the diff touches two documentation files only. |

## 2. Findings

| ID | Severity | Location in `ac1d378` | Finding | Correction |
|---|---|---|---|---|
| A1 | Fidelity, significant | §2, GF2-03 | Stated ∂2 c = (B⋆c, A⋆c), ∂1 (u,v) = A⋆u + B⋆v. Source (`BBChainComplex.lean:390–395`) uses ∂2 f = (A⋆f, B⋆f), ∂1 (u,v) = B⋆u + A⋆v. Both are valid complexes, but the recorded convention was the transpose of the source. | GF2-03 now states the source convention and the convolution `conv` (line 51); an alternative convention requires an explicit isomorphism (O-3, O-4). |
| A2 | Fidelity, moderate | §3.1, "Face checks" row | Merged two different 45-element index sets: syndrome positions (C0, target of ∂1) and face generators (C2, source of ∂2). Both are enumerated by `baseFaceEnum`, which explains the source naming. | Separate rows for C0 and C2, with incidence data from `baseSyndromePoints` and the bit encoding (`baseFaceEnum` negates both coordinates). |
| A3 | Fidelity, minor | §3.1, push theorems | Credited `base_push_A/B` with the top-to-base reduction; they state middle-to-base, and `top_push_A/B` state top-to-middle. | Both steps cited (`Defs.lean:91–97`). |
| A4 | Status scope, moderate | §5 axioms row; manifest `reported_axioms` | The README's axiom report covers the three capstones; `base_logical_floor` carries no `#print axioms` line, and the two census lines have unrecorded output. | Row and manifest rescoped; the dependency of the capstones on `base_logical_floor` cited (`MiddleFloor.lean:61,68`), still pending by build. |
| A5 | Factual, moderate | §5 QECLean `sorry` row | Reported tagged `sorry` in Concatenation and Iceberg modules. A token-level search finds zero `sorry` proof terms in QECLean at the pin; every occurrence is documentation text (some docstrings describe earlier skeleton stages). | Row corrected; build-level confirmation kept under O-6. |
| A6 | Logic, minor | §6 O-2 | Called "the 45 weight-6 cycles are exactly the 45 face checks" equivalent to O-2. It is a sufficient check given O-1, and the zero cycle needs its own witness. | Restated as a sufficient check. |
| A7 | Consistency, minor | manifest | `checks: 45` repeated A2; axiom scope repeated A4; next milestone predated the refinement. | Fields `syndrome_positions_C0`, `face_generators_C2`, `convention`, `encoding`, `used_by`, `reported_for`, `next_milestones`, `evidence_levels`. |
| A8 | Provenance, minor | §6 O-7 | Described the candidate data as from qec-lab. Source ties qec-lab to the matrix convention (`Defs.lean`); `scripts/bravyi360/README.md` describes the candidate data as output of an original large-distance search, replayed only in part by regeneration. | O-7 restated. |

## 3. Claims re-confirmed

- `base_logical_floor : baseCover.LogicalFloor 8` at `BaseFloor.lean:83`; `baseCycle10_complete` at `BaseCycleCensus.lean:2033`; `LogicalFloor` at `BBDoubling.lean:157`.
- Completeness structure: `allBaseCycles10_checked` (validity, order, length) plus `baseCycle10Target_card_le` (trellis cardinality bound), joined by `ordered_census_complete`.
- 1,954 strictly increasing masks below 2⁹⁰; weights {0: 1, 6: 45, 8: 90, 10: 1,818}; 45 boundary-window entries (parser count, pending independent recomputation).
- Pins, toolchain `v4.30.0-rc2`, Mathlib `a3fde50…`, both bb-distance-lean HEADs at `a83cafa…`.
- Base polynomials, group orders, quotient chain, and the §4 distinction between the base floor and `stabilizerCode_hasDistance24`.
- Arithmetic in §7: Σ_{k≤4} C(90,k) = 2,676,766; Σ_{k≤5} C(90,k) = 46,626,034.

## 4. Obligations after the audit

| ID | Status | Discharged by |
|---|---|---|
| O-1 | open | 002a (weight < 8), 002c (weight ≤ 10) |
| O-2 | open, restated | 002b |
| O-3 | open; source encoding now recorded | 002a–002c correspondence sections |
| O-4 | open; source convention now recorded | 002b correspondence |
| O-5 | open | 002b |
| O-6 | open; text-search part done | source audit with a Lean build |
| O-7 | open, restated | source audit (hashes) |
| O-8 | open | later middle/top-floor milestone |

## 5. Recommendation (Revision 2)

Accept Revision 2 for an independent read-only audit by a fresh session. Merge after that audit passes.

## 6. Independent audit of `a3daea2` and Revision 3

**Auditor.** A fresh session, separate from the one that wrote `ac1d378` and `a3daea2`, audited Revision 2 read-only on 2026-11-25 against the same pins (`bb-distance-lean@a83cafa`, HEAD of upstream and fork; `QECLean@f51011b`). Method: reading the pinned files, a comment-stripped token search of `BBDistance/` and `QEC/`, and a spot-check script outside the repositories comparing the documented encoding formulas with the source's packed data (90 `cycleSyndrome` columns, 45 `baseBoundaryWindow` masks, the 1,954 listed masks, 46 `baseFloorPairs`). The spot check read listed data only; it enumerated no cycles and counts as no RLT-QEC-002 evidence.

**A1–A8.** All eight corrections were confirmed against source. The spot check also confirmed: the census entries of weight < 8 are exactly the zero mask and the 45 distinct window masks; window entry i is the face `baseFaceEnum.symm i`; the 46 witness pairs reconstruct their masks; the listed masks are strictly increasing, below 2⁹⁰, of zero syndrome, with weights {0: 1, 6: 45, 8: 90, 10: 1,818}.

**New findings and their resolution in Revision 3.**

| ID | Severity | Finding in `a3daea2` | Resolution (Revision 3) |
|---|---|---|---|
| B1 | Moderate, convention | GF2-03's note and A1 called the A↔B convention the "transpose". It is the block-swap convention X′, isomorphic to X by σ(u, v) = (v, u) in the same sector; the transpose is the dual complex Xᵀ with reflected polynomials, which carries the other (Z) sector. | §2.1 registers three correspondences (block swap σ, reflection R, dual complex Xᵀ with Φ = R ∘ σ = `blockSwapNeg`, `bb_cycle_bound_iff_dual_bound`, `BBDuality.lean:359`), states which statements each transfers, and reserves *transpose* for Xᵀ. O-8 names Φ as its candidate. |
| B2 | Moderate, R vs V | §7.2 and §7.4 assigned level V to a checker verifying "the splitting argument"; a meet-in-the-middle run leaves no completeness certificate. | §7.1 separates the two completeness bases: a reviewed mathematical argument (completeness at R when a reproducing program executes it) and a machine-checkable completeness certificate (the only basis for completeness at V). 002a and 002c report soundness and completeness separately; checking the source's trellis certificate is reported separately as verification of the source certificate. The weakest-obligation composition rule is recorded. |
| B3 | Moderate, coordinates | Weight distributions agree under σ, R and every automorphism, so a distribution match leaves the encoding unchecked. | §2.1 registers face(x, y) = 3·((−x) mod 15) + ((−y) mod 3) and qubit(x, y, j) = 45·j + face(x, y), bit k ↔ index k. §7.1 makes mask-set equality the acceptance criterion for every comparison, with counts and distributions as diagnostics. |
| B4 | Moderate, R/V/K | §7.1 called `base_logical_floor` "level K relative to Lean's kernel" before any build. | Classified K (Lean), reported, pending O-6, outside the R < V < K ordering; it enters RLT results only as a correspondence. |
| B5 | Minor, fidelity | The floor path was routed through `base_cycle_mem_census`. | §3.2 gives the path `base_cycle_lt_eight_boundary` → `base_floor_eight` → `baseCycle10_complete` + `baseFloor_checked` → `baseBoundaryWindow_mem`, and cites the later users of `base_cycle_mem_census`. |
| B6 | Minor, manifest | `certificate_data` listed two of the four data lists on the floor path; `baseCensusSyndrome_bounded` was located in `BaseSparse.lean`. | `cycleSyndrome` (`ConcreteKernelSpan.lean:212`) and `baseFloorPairs` added to §3.2 and the manifest; `baseCensusSyndrome_bounded` located at `BaseFloorChecks.lean:14`, bounding the `cycleSyndrome` columns. |
| B7 | Minor, provenance | The 39,227-declaration replay was reported without its layout. | §5 and the manifest record that the replay ran on the original QECLean layout before extraction; the pinned tree awaits its own replay or build. |
| B8 | Minor, consistency | §3.2 said the generator searches for nothing; O-7 said small parity searches are replayed. | §3.2 now matches O-7. |
| B9 | Minor, method | Orbit reduction and overlapping halves needed explicit handling; 1,818 = 40 · 45 + 18 shows smaller translation orbits at weight 10. | §7.1 specifies translation-equivariance, orbit closure under all 45 translations, deduplication by mask and recheck of cycle condition and weight; counts come from the closed, deduplicated set. Meet-in-the-middle matches are rebuilt, rechecked and deduplicated. |
| B10 | Information | The text search covered `BBDistance/` and `QEC/`; QECLean upstream HEAD is `47876ae`, ahead of the pin. | §5 scopes the search (Mathlib to the O-6 build) and records the upstream HEAD; the pin governs. |

## 7. Obligations after Revision 3

All eight stay open. Revision 3 registers specifications that their discharge will use; it discharges none of them.

| ID | Status | Discharged by |
|---|---|---|
| O-1 | open | 002a (weight < 8), 002c (weight ≤ 10) |
| O-2 | open | 002b |
| O-3 | open; source encoding formulas registered (§2.1) | 002a–002c correspondence, by mask-set equality |
| O-4 | open; source convention and correspondences registered (§2.1) | 002b correspondence |
| O-5 | open | 002b |
| O-6 | open; text search of `BBDistance/` and `QEC/` done | source audit with a Lean build (Mathlib included) |
| O-7 | open | source audit (hashes) |
| O-8 | open; candidate Φ, `bb_cycle_bound_iff_dual_bound` named | later middle/top-floor milestone |

## 8. Confirming read of Revision 3 (self-check)

**Two activities, kept distinct.**

| Activity | Performed by | Independence | Record |
|---|---|---|---|
| Audit of `a3daea2`, findings B1–B10 | a fresh session, separate from the author of `ac1d378` and `a3daea2` | independent audit | §6 |
| Revision 3 (`52aeae7`) and this confirming read | the same session that produced B1–B10, which also wrote Revision 3 | **self-check**: the author of Revision 3 checked its own revision against its own findings | this section |

The findings B1–B10 stand as independent audit findings. Their resolution in Revision 3, and the confirmation below, carry the weight of a self-check: they show that the author addressed each finding as the author understood it, and they leave open whether a separate reviewer reads each resolution the same way.

**What the self-check covered.** The author reread Revision 3 of `docs/RLT-QEC-001.md` and `research/qec-001/MANIFEST.yaml` against B1–B10 and the pinned sources:

- §2.1 maps agree with `BBChainComplex.lean:390–395` and `BBDuality.lean` (`reflect`, `blockSwapNeg`, `cutMap`, `dualBoundary`); σ satisfies ∂2′ = σ ∘ ∂2 and ∂1 = ∂1′ ∘ σ; RA = x⁶ + y + y², RB = 1 + x⁵ + x⁴; the encoding formulas match `Defs.lean:62–76` and the audit spot check.
- 23 cited source locations were compared line by line with the pinned files; one citation was corrected (`Bravyi360/README.md:59–61`, previously 58–61).
- §3 and §5 citations match the pinned files; the manifest parses as YAML and agrees with the document field by field.
- §7 assigns each RLT-QEC-002 result a level, names its completeness basis, applies the composition rule and uses mask-set equality as acceptance.
- O-1 to O-8 are open in §6 of the specification, in this record and in the manifest.
- The diff from `main` touches three documentation files; kernel, registries, code and upstream repositories are unchanged.

**Result.** By self-check, B1–B10 are resolved, and the documentation branch was merged at `1aa2bda`. An independent reading of the Revision 3 resolutions remains available as a follow-up review; it is recommended alongside the independent review of RLT-QEC-002a, which builds on §2.1 and §7.

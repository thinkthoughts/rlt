# RLT-QEC-001 — Audit of commit `ac1d378`

**Status.** Documentation audit, 2026-11-25. Subject: `ac1d378` on branch `rlt-qec-001` (`docs/RLT-QEC-001.md`, `research/qec-001/MANIFEST.yaml`). Corrections are in the commit that adds this record (Revision 2 of both files).

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

## 5. Recommendation

Accept Revision 2 for an independent read-only audit by a fresh session. Merge after that audit passes.

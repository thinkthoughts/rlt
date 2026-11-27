# RLT-AGENT-000 — Existing interface assessment

Date: 2026-11-26. Baseline: `0ecc148e80b8f3f5086d25fabc297ffc228346f4`. Status: static assessment, proposed requirements, review pending. Companion to RLT-DOC-003 and RLT-ROADMAP-001 §4. No implementation or schema change.

## Method

Inspected the existing serializers, shared checker outcome and report extraction, CLI dispatch, statement record parser, and committed MOD30-CLASSIFICATION JSON golden. Findings concern source at the pinned revision; no new binary execution. Retain the roadmap's proposed rlt-claim/1 and rlt-audit/1 and local rlt audit command as design starting points.

## Existing interfaces and mapping

| Audit requirement | Existing output | Gap or constraint |
|---|---|---|
| Claim identity and submission | rlt-statement/1 input; record.path/id/subject/identifiers in rlt-explain/1 | No general claim submission or claim digest in inspected CLI dispatch. |
| Proof outcome | explain.kernel: verdict, verdict_line, checker_exit, proof, sha256, proof_id, written_goal, checked_goal, proved, convention | Reuse authoritative check of exact bytes; keep command exit distinct. |
| Assumptions and rules | explain.kernel.assumptions, axioms_used, rules_used, steps are strings or null; deps.root.assumption_pairs is array or null | Prefer typed theorem data; declared mode and actual assumptions separate. |
| Axiom identity | explain.kernel and deps nodes: axiom_set, axiom_set_sha256 | Bind submitted context to actual checked context; not simply echo request. |
| Definitions | rlt-definitions/1: registry, sha256, files, names, shared_expansions; functions have content/expansion hashes | Registry consistency is not proof-use provenance; see AG-F02. |
| Citations | explain.kernel.cites; deps.root recursively includes children, checked, problem, fingerprint; summary includes limits and attempted checks | Preserve failed or unattempted citation outcomes and edge refusal; do not infer successful dependency from path alone. |
| Library | rlt-deps-lib/1: entries with id, exit, state, stale; aggregate exit and problems | Not a per-claim theorem verdict. |
| Source | explain.source: registry, registry_status, commit, location, excerpts, registry_fidelity, pinned, upstream | Upstream verification separate; do not invent per-excerpt verified flags. |
| Semantic | explain.semantic: status, assessment, stale_reasons, assessed_by, date, basis, assessed_goal, assessed_goal_current, matches, differs | Keep proposed and recorded assessments separate; stale assessment is not a new assessment. |
| Lean | explain.semantic.lean: file, theorem, named_in_file, checked_by | Reference presence only; separate independently executed or attributed replay evidence. |
| Physical | explain.physical: applicability, reason, reading_point, records with id/kind/description/status/evidence count | Count is not an evidence bundle or validation of observations; preserve attribution and applicability. |
| Obligations | explain.open_questions, record.problems/stale, semantic.differs, deps warnings/notes | Aggregate without silently discharging any; distinguish requested checks not run. |
| Composition | compose text report and Composition structure; rlt-compose/1 input | Separate composed outcome and graph; no JSON output in inspected dispatch. |
| Reproducibility | proof pins and report format IDs | Audit envelope needs provenance, actual invocation, result digest and exact serialization contract. |

## Findings

AG-F01 — kernel_fields (explain.rs 280–380) obtains many fields from formatted report lines. CheckOutcome (lib.rs 550–590) already contains parsed proof, checked proposition, theorem and citation traces. Investigate typed extraction before adding another text parser; preserve existing public JSON and golden outputs until reviewed.

AG-F02 — NAT-FUN provenance gap (static evidence, behavioral reproduction pending). lib.rs 870–900 renders NAT-DEF definitions with a " — " separator; NAT-FUN renders file/id/convention/domain/hash without that separator. kernel_fields only extracts used/hash on the former shape; explain text and JSON hardcode spec/definitions.rlt if a hash exists. Statement record parser accepts a convention string and optional definitions_sha256, and finish compares that hash with the extracted field. Thus omission/misrepresentation is a concrete static concern, not a demonstrated accepted false theorem. Test a valid registered NAT-FUN statement record both with and without the definition hash, plus deps output and existing NAT-DEF golden; confirm reachability and exact behavior before fixing. Related to DEF-001a audit L3, not a replacement for it.

AG-F03 — named_in_file is not Lean replay. The serializer explicitly attributes checks to scripts/verify.sh, not explain. Require separate evidence and distinguish not run from historical results.

AG-F04 — registry_fidelity is not upstream excerpt verification. Preserve both; the committed golden explicitly reports upstream not run by explain.

AG-F05 — existing json_write emits fields in supplied order with indentation; it does not sort keys. Determinism of a renderer is not a specified canonical hash protocol. Define serialization, duplicate-key rejection and digest preimage (exclude digest field itself) before claiming reproducible audit hashes.

AG-F06 — definitions JSON lacks an embedded exit; an unknown id can emit consistent:true plus problem and return 1. Capture process outcome independently; do not turn consistent:true into universal success.

AG-F07 — explain JSON omits assumption_pairs even though the internal block contains them; deps emits pairs. Missing or null is not the empty context. Use actual theorem scope rather than interpreting display strings.

## Proposed first implementation scope (review before coding)

1. Read-only local adapter over existing registered statement records, initially one claim/record at a time; use exact-byte checking and existing outputs. Do not accept arbitrary inline certificates, write tracked files, run network services or automatically run Lean in this first prototype.
2. Independently test the existing output contract with a fixture-based auditor, clearly labeled as structural/provenance checks rather than proof verification. Then connect to a real RLT executable for end-to-end tests.
3. Reproduce AG-F02 with registered NAT-FUN controls; if confirmed, design a narrowly scoped reporting correction and compatibility tests before agent integration.
4. Preserve current outcomes and independent source/Lean/physical channels. Unknown format, unexpected exit, missing fields or inconsistent identity must not become ACCEPT.
5. Specify claim-to-checked-statement equality, actual context, canonical digest and process failure handling before adopting a wire schema. Do not treat agent-supplied verdicts as trusted.

## Acceptance cases for future work

Current accepted MOD30 golden with partial semantics and no upstream/Lean execution; invalid/stale record; conditional proof; refused citation; exploratory result refused for authoritative use; registered NAT-FUN provenance; unknown definitions id; tampered hashes; unknown format; duplicate keys; missing versus empty assumptions; composition not presented as a kernel theorem; proposed assessment not promoted; process error or malformed output not accepted.

## Limits

No local build, runtime reproduction, CI or independent review performed. Existing golden is historical evidence only. No new schema approved. Next proposed milestone: RLT-AGENT-000a, reproduce the provenance gap and develop an independent fixture-based prototype outside the trusted base.

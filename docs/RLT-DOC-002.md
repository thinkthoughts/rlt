# RLT-DOC-002 — Roadmap reconciliation

Date: 2026-11-26. Baseline: `0ecc148e80b8f3f5086d25fabc297ffc228346f4` (main). Companion to RLT-DOC-001. Status: proposed reconciliation, review pending. No implementation changes or new mathematical claims.

## Authority and method

Retain [RLT-ROADMAP-001](RLT-ROADMAP-001.md) (2026-11-22) as the existing architecture and design source. This dated addendum reconciles selected subsequent release records; it does not supersede their specifications. Read roadmap §§4–7, RLT-DEF-001a report and audit, RLT-QEC-002b, and selected earlier handoff and composition records at the pinned baseline. Historical test results are not new execution evidence.

## Reconcile roadmap milestones

| Existing roadmap item | Baseline evidence | Proposed disposition |
|---|---|---|
| M-A / D2 | RLT-DEF-001a report and audit §9 | Registered authority implemented and released; exploratory mode intentionally retained and contained. Do not rebuild registry. |
| M-A / predicates | RLT-DEF-001a report | 001b awaits separate review; no automatic approval to re-register predicates. |
| M-A / D3 | RLT-DEF-001a report | 001c definition-dependence analysis awaits separate review. |
| M-B / AGENT v0 | RLT-ROADMAP-001 §4 and §5.6–7 | Existing local audit CLI design; evaluate against current outputs first. No dependency on completion of 001b. |
| M-C / registrations | RLT-ROADMAP-001 §5.3 | Retain as proposals; check current records and author approval before changes. |
| M-D / AGENT v1 | RLT-ROADMAP-001 §5.7 | Definition identity now has a registered mechanism; inspect exact binding semantics before using it. |
| M-E / author clarifications | RLT-ROADMAP-001 §5.4 | Remain open unless separately resolved by source evidence. |
| M-F / induction | RLT-ROADMAP-001 §3.2, §5.5 | Separate theory review; do not imply induction is in Robinson Q or change kernel rules. |
| M-G / RH | RLT-ROADMAP-001 §2.10 | Preserve statement fidelity and independent Lean obligations; current external research status not audited here. |
| M-H / network | RLT-ROADMAP-001 §4.5 | Defer until local interface is specified and tested. |
| QEC | RLT-QEC-002b §13 | 002b reported merged; 002c is documented next, exact X-sector minimum remains in {8,9,10} under the recorded scope. |
| README status | RLT-REVIEW-HANDOFF | M3d and M3e reported merged; update separately after checking later release records. |

## Definition follow-up register

Source: [RLT-DEF-001a-AUDIT](RLT-DEF-001a-AUDIT.md) §8–9. These are historical open items; current code resolution has not been independently checked. H1 is reported resolved.

| ID | Proposed treatment |
|---|---|
| H2 | Inspect public alternative-pin seam and production call paths before choosing a sealing strategy. |
| H3 | Correct exploratory registry label; test that self-declared identity never implies authority. |
| H4 | Correct integrity diagnostic and document fail-closed behavior. |
| H5 | Review parser policy separately; pinned bytes and syntactic canonicality are distinct. |
| H6 | Require expected successful exit statuses and discovery regeneration in report comparison. |
| L1 | Evaluate registry reporting in validate; distinguish reporting from enforcement. |
| L2 | Document case-sensitive identity and platform-specific filesystem behavior. |
| L3 | Carry to 001c or separately reviewed library identity format; distinguish unfolded formula from written title. |
| L4 | Add authoritative and exploratory NAT-FUN composition fixtures after reviewing existing fixtures. |
| L5 | Measure before optimizing; preserve pin checks. |

## Agent design continuity

Use roadmap §4's proposed rlt audit, rlt-claim/1 and rlt-audit/1 as the starting design, not a new competing schema. Existing rlt-explain/1, rlt-deps/1, rlt-deps-lib/1, rlt-statement/1 and composition machinery must be inventoried before adding fields.

RLT-DOC-003 must derive command-specific outcome meanings from code and fixtures, including EXPLORATORY (introduced after the roadmap), stale records, bounded-search outcomes and composition discharge. Preserve independent Lean reporting, source fidelity, semantic assessment and physical evidence as separate channels. An audit hash binds bytes, not correctness; specify canonical serialization and exclude the digest field from its own preimage. These are design review requirements, not implemented features.

## Proposed execution order

1. DOC-003: inspect current CLI, tests and report formats; write a status/exit taxonomy with source references and unresolved discrepancies.
2. AGENT-000: map existing outputs to roadmap §4 and identify demonstrated gaps using research cases; do not implement a network service.
3. DEF-001b preparation in parallel: inspect current code and audit follow-ups, propose acceptance tests and predicate-definition scope for review.
4. QEC-002c and other scientific applications remain separately scoped research milestones, not dependencies of documentation or agent work.

No changes to the trusted base, proof fixtures, registries or original roadmap are authorized by this addendum. No new local build or CI run was executed. Review and merge remain pending.

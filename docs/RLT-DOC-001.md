# RLT-DOC-001 — Documentation inventory and gap analysis

Date: 2026-11-26. Baseline: `0ecc148e80b8f3f5086d25fabc297ffc228346f4` (main). Status: inventory complete; findings preliminary, review pending. Scope: documentation only.

## Method and limits

Retrieved the full recursive Git tree using the GitHub API (not truncated): 1652 entries, 1492 blobs. Inspected README, RLT-ROADMAP-001, RLT-REVIEW-HANDOFF, STATEMENT-RECORDS and RLT-007d-FINAL at the baseline (selected sections). Earlier audit findings are leads for follow-up, not fresh execution evidence. No local build, proof replay or CI run was performed. File presence does not establish correctness or completion.

## Findings

| ID | Evidence | Decision |
|---|---|---|
| DOC-F01 | README still describes M3d as under review; RLT-REVIEW-HANDOFF records M3d merged at 3d0f8e2 and M3e at cc01968. | Reconcile README against release records. Do not reimplement these milestones. |
| DOC-F02 | RLT-ROADMAP-001 already provides architecture, mathematical inventory and API direction. | Extend or index that document, not a competing roadmap. |
| DOC-F03 | STATEMENT-RECORDS specifies rlt-statement/1 and rlt explain --format json; roadmap lists rlt-explain/1 and dependency JSON formats. | Reuse existing records before proposing an agent schema. |
| DOC-F04 | RLT-ROADMAP-001 records composition as trusted checking outside the inference-rule kernel, with an independent Python checker. | Include composition in the trust map; distinguish proof acceptance from composition discharge. |
| DOC-F05 | RLT-DEF-001a audit/report and definitions registry exist. | Inspect current implementation and audit follow-up before proposing new registry work. |
| DOC-F06 | CI workflow and verification script exist. | Inspect current gates and baseline run status before proposing new CI. |
| DOC-F07 | STATEMENT-RECORDS distinguishes proof validity, source fidelity, semantic and physical correspondence; exit status does not certify attributed assessments. | Preserve those distinctions in future interfaces. |

## Next work

1. RLT-DOC-002: reconcile the existing roadmap and release records, including definition-audit follow-up; do not create a redundant roadmap.
2. RLT-DOC-003: derive command-specific status and exit-code semantics from current code and fixtures, including exploratory, stale and bounded-search outcomes.
3. RLT-AGENT-000: evaluate existing JSON outputs against research application requirements. No new schema or API is approved by this inventory.

No kernel, axiom, definition, convention, reading-point or proof changes are proposed here.

## Baseline inventory

Tracked documentation, specification, verification and workflow blobs (plus root README), sorted by Git tree path:

- `.github/workflows/ci.yml`
- `README.md`
- `docs/KERNEL-SOUNDNESS.md`
- `docs/MANUAL.md`
- `docs/PROVER.md`
- `docs/RLT-000.md`
- `docs/RLT-001-AUDIT.md`
- `docs/RLT-001.md`
- `docs/RLT-002-AUDIT.md`
- `docs/RLT-002-T7.md`
- `docs/RLT-002.md`
- `docs/RLT-003-M1.md`
- `docs/RLT-003-M2.md`
- `docs/RLT-003-M3a.md`
- `docs/RLT-003-M3b.md`
- `docs/RLT-003-M3c.md`
- `docs/RLT-003-M3d.md`
- `docs/RLT-003-M3e.md`
- `docs/RLT-004-REVIEW.md`
- `docs/RLT-004a-BASELINE.md`
- `docs/RLT-004a-REPORT.md`
- `docs/RLT-004b-REPORT.md`
- `docs/RLT-004b-REVIEW.md`
- `docs/RLT-004c-REPORT.md`
- `docs/RLT-004c-REVIEW.md`
- `docs/RLT-005-BRANCH-PROTECTION.md`
- `docs/RLT-005-CORRESPONDENCE.md`
- `docs/RLT-005-REPORT.md`
- `docs/RLT-005-REVIEW.md`
- `docs/RLT-006-CORRESPONDENCE.md`
- `docs/RLT-006-REPORT.md`
- `docs/RLT-006-REVIEW.md`
- `docs/RLT-007-HANDOFF.md`
- `docs/RLT-007-REVIEW.md`
- `docs/RLT-007a-REPORT.md`
- `docs/RLT-007b-1-CORRESPONDENCE.md`
- `docs/RLT-007b-1-REPORT.md`
- `docs/RLT-007b-2-DESIGN.md`
- `docs/RLT-007b-2a-CORRESPONDENCE.md`
- `docs/RLT-007b-2a-REPORT.md`
- `docs/RLT-007b-3-CORRESPONDENCE.md`
- `docs/RLT-007b-3-REPORT.md`
- `docs/RLT-007b-4-CORRESPONDENCE.md`
- `docs/RLT-007b-4-REPORT.md`
- `docs/RLT-007b-DESIGN.md`
- `docs/RLT-007c-FINDINGS.md`
- `docs/RLT-007d-ARCHITECTURE.md`
- `docs/RLT-007d-ARTIFACT-OUTLINE.md`
- `docs/RLT-007d-CORRESPONDENCE.md`
- `docs/RLT-007d-FINAL.md`
- `docs/RLT-007d-REPORT.md`
- `docs/RLT-DEF-001a-AUDIT.md`
- `docs/RLT-DEF-001a-REPORT.md`
- `docs/RLT-PROVER-001-AUDIT.md`
- `docs/RLT-PROVER-002-AUDIT.md`
- `docs/RLT-PROVER-002.md`
- `docs/RLT-QEC-001-AUDIT.md`
- `docs/RLT-QEC-001.md`
- `docs/RLT-QEC-002a.md`
- `docs/RLT-QEC-002b.md`
- `docs/RLT-REVIEW-HANDOFF.md`
- `docs/RLT-ROADMAP-001.md`
- `docs/STATEMENT-RECORDS.md`
- `spec/axioms.yaml`
- `spec/definitions-registry.yaml`
- `spec/definitions.rlt`
- `spec/peano-arith.rlt`
- `spec/peano.rlt`
- `spec/robinson-q.rlt`
- `verification/RLT-001-results.txt`

## Acceptance record

- Pinned baseline: complete.
- Recursive inventory: complete (API reports not truncated).
- Content review: selected documents only; exhaustive review pending.
- Independent build/replay: not performed.
- Review and merge: pending.

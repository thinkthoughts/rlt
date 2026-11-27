# RLT-DOC-003 — Command-specific verdict semantics

Date: 2026-11-26. Baseline: `0ecc148e80b8f3f5086d25fabc297ffc228346f4`. Status: static inspection, review pending. Companion to RLT-DOC-001/002 and RLT-ROADMAP-001 §4. No implementation or mathematical claim changes.

## Scope and method

Read selected command dispatch, outcome construction, report and regression-test source at the pinned baseline. This is not a fresh build or test run. The following table covers the proof, discovery, registry and record commands relevant to the proposed audit interface, not every CLI command. Invocation errors may return 2 independently of command-specific results; process failures require separate handling.

## Command-specific outcomes

| Command | 0 | 1 | 2 | 3 | 4 |
|---|---|---|---|---|---|
| check | ACCEPT | REJECT | MALFORMED | — | EXPLORATORY |
| prove | PROVED | NOT_FOUND_WITHIN_BOUNDS | ERROR | — | EXPLORATORY |
| discover | completed without reported errors | — | ERROR | — | exploratory definition-aware record |
| discover --verify | record checks pass | refused or stale | required input read error | — | — |
| explain | current and proof accepted | proof rejected | invalid or malformed | stale | — |
| deps / deps --verify-lib | checked and current | rejected | invalid or malformed | stale | — |
| compose | COMPOSED | refused | malformed or missing | stale | — |
| definitions | consistent, requested entry if any exists | unknown id | inconsistent or invalid invocation | — | — |

Source anchors (line numbers at baseline): cli/src/main.rs 9–39, 76–96, 150–301; lib.rs 27–52, 617–655, 1224–1285; prove.rs 1563–1580; discover.rs 380–392, 639–733; discover_defs.rs 325–339, 539 onward; explain.rs 781–790, 1116–1134; deps.rs 775–865; compose.rs 352–375, 975–1046; defreg.rs 635–696.

## Interpretation and non-implications

- ACCEPT: the checked conclusion follows from recorded axioms and assumptions under the rules. Preserve conditional versus unconditional scope. It does not prove the axioms, the fidelity of the translation, or physical correspondence.
- EXPLORATORY: successful replay over unregistered definitions; not authoritative. The explicit option alone does not force exit 4 for registered input. An unsuccessful search over exploratory input may return 1; malformed input may return 2. Citations are checked authoritatively.
- REJECT: refusal of this derivation, not falsity of its goal.
- PROVED: a found proof replayed by the checker; not independent Lean verification. NOT_FOUND_WITHIN_BOUNDS is not a disproof or independence result. ERROR includes failed replay.
- Discovery completion does not imply all candidates were proved. Verification checks the record and its replayed certificates, not a new exhaustive search or the falsity of unproved candidates. Definition-aware verification uses RegisteredOnly authority.
- STALE: differs from current pins or checked records, not automatically a rejected derivation. Discovery verification uses exit 1 for stale/refused results rather than 3.
- Explain: the record and checker determine exit; semantic and physical assessments do not. Source registry fidelity and independently checked upstream excerpt fidelity are distinct.
- COMPOSED: lower-layer passing nodes discharge the recorded assumptions; this is not a single unconditional kernel theorem. Preserve the composition label and graph.
- CONSISTENT: registry checks pass; this is not proof acceptance. An unknown id can yield exit 1 even when registry JSON reports consistent: true.

## Aggregation

check multi-file severity: 0 < 4 < 1 < 2 (unknown codes rank higher). deps --verify-lib uses its own ordering: 0 < 3 < 1 < 2 for its expected domain. Do not use numeric maximum or one universal map. An adapter must preserve per-item outcomes, command, exit, authority, diagnostics and whether a check ran. Unknown/unexpected outcomes must not be promoted to acceptance.

explain and deps use authoritative check_bytes_outcome; compose uses authoritative check_preread_outcome. Their existing outcome logic assumes those paths cannot return EXPLORATORY. Verify this invariant when adapting, rather than treating wildcard fallthrough as a general acceptance rule.

## Existing regression evidence (source inspected, not executed)

- cli/tests/def001a_h1.rs: severity ordering and multi-file permutations.
- cli/tests/def001a.rs: unregistered definitions and tampering refusal.
- cli/tests/prover.rs: bounded misses, independent replay, discovery tampering.
- cli/tests/explain.rs: assessment independence, stale assessments, registry versus upstream fidelity, JSON and exits.
- cli/tests/deps.rs: graph/record checks and stale/negative cases.
- cli/tests/rlt007d.rs: composition controls, refusal classes, independent Python checker agreement.

## Findings and proposed follow-up

DOC-F08/F09: exit 1 and exit 0 are command-specific. Preserve separate outcome channels.
DOC-F10: aggregation must use command-specific severity.
DOC-F11: add rlt-definitions/1 to existing JSON inventory (defreg.rs 635 onward).
DOC-F12: main.rs usage for definitions omits exit 1 for unknown id. Correct later in a separately reviewed documentation/help change.

Agent requirements: do not interpret missing checks as passed; distinguish not requested, not run, unavailable, and not applicable as proposed adapter states (not existing CLI verdicts). Record independent Lean results separately, never infer them from RLT exits. Do not infer physical correspondence from any exit. Preserve existing report semantics and version identifiers. No new JSON schema is approved here.

## Limits and next step

Static review only; no local build, fixture execution, CI replay or independent review. Commands outside the stated scope still need inventory if an agent interface exposes them. Next: RLT-AGENT-000, map actual existing JSON fields and gaps against roadmap §4 before proposing implementation.

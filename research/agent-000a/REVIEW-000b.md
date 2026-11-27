# RLT-AGENT-000b — Adversarial self-review and narrow hardening

Date: 2026-11-26. Parent: f6a857bcf7c40a0b135abd4211ef08b916ca9aec. Branch: rlt-agent-000b. Status: tested prototype, independent review pending. This is a second review pass by ChatGPT, which also authored 000a; it is not independent third-party evidence.

## Method and findings

Fetched the published implementation and test file, verified local Git blob identities, verified the golden fixture blob d3653c65c9d5051815bb915389c299b21f296c13, and reran the seven original Python test methods (all passed). Introduced ten mutations with recomputed expected digests; all ten produced PINNED_STRUCTURE_MATCH before hardening.

| ID | Mutation | Correction |
|---|---|---|
| B01 | record.id null | require nonempty identity and relative record path |
| B02 | verdict_line REJECT | require expected ACCEPT wording and named axiom set |
| B03 | proof path ../../outside.rlt | lexical relative path restriction; no file access |
| B04 | steps -1 | positive decimal string in this profile |
| B05 | cited proved null | nonempty citation fields and lexical path |
| B06 | semantic matches false | string-list checks |
| B07 | invalid commit, matching upstream text | restrict source status and pin syntax |
| B08 | physical reason null | require nonempty reason |
| B09 | open_questions integer | string-list check |
| B10 | exit_meaning claims Lean verified | require existing explain exit-0 wording |

## Executed evidence

Eight test methods passed after hardening (including all 17 earlier field mutations, all 10 new mutations, original digest control and the coherent modification control). Python standard library only. Reproduce from repo root:

    python3 -m unittest discover -s research/agent-000a -p 'test_*.py' -v

Tested source Git blobs: audit_explain.py aeac71848622ca497672b5d33013099316a00ef6; test_audit_explain.py c51da1e1e192520cf0afbb432eca6135aec4e17c. Published commits bb00bf0 and c647239 report those exact content identities.

## Remaining limits

These checks are deliberately conservative lexical/profile checks, not general validation of every valid RLT output. Exact display wording is brittle and must not be a future authoritative machine contract. Pin syntax is not pin resolution. Relative path checking is not safe file resolution; this prototype does not follow paths. It does not validate every field value, reconcile conditional wording with structured assumptions, bind embedded file hashes to files, authenticate the caller-supplied digest or obtain an actual process exit. A coherent forged report with a recomputed digest may still pass, as the retained test explicitly demonstrates. The status means selected profile checks and byte identity against the supplied pin, not verified claim or proof. Proof replay, Lean, upstream and physical verification remain false.

No Rust or kernel changes. No live RLT execution, AG-F02 runtime reproduction or CI verification. No merge. Recommend independent review before widening scope; prioritize obtaining a real RLT executable and typed report provenance over adding more string checks.

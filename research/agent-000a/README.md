# RLT-AGENT-000a — Offline interface auditor (experimental)

Date: 2026-11-26. Baseline: main at 0ecc148e80b8f3f5086d25fabc297ffc228346f4; documentation parent 78d1aeb1ff9ba5a3578d89d9d485e3101a39ef49. Branch: rlt-agent-000a. Not part of the trusted computing base; review pending.

## Scope

This is a deliberately narrow offline prototype for a current, accepted NAT-DEF rlt-explain/1 report with a pinned source and physical applicability marked not-applicable. It checks selected structural constraints and the report's byte digest against a caller-supplied pin. It is neither a full schema validator nor a proof, source, Lean, or physical correspondence checker. A matching digest is not authentication: the expected digest and supplied process exit must originate independently. The fixture's historical exit is supplied as 0; no live RLT process ran.

## Reproduction from repository root

    python3 -m unittest discover -s research/agent-000a -p 'test_*.py' -v
    python3 research/agent-000a/audit_explain.py tests/explain-golden/MOD30-CLASSIFICATION.json --process-exit 0 --expected-sha256 445d26c27ba42a545f36e13ce667bbaff5fdd17c0b0b1fbd02b636ec7b8c11b8

## Executed evidence

Python standard library; 7 test methods passed, including 17 field mutations with recomputed digests, duplicate-key JSON, process exits, digest mismatch, and a control showing coherent modifications can pass selected structure checks when supplied a new digest. The exact 12740-byte committed fixture was transferred via GitHub and independently checked against Git blob d3653c65c9d5051815bb915389c299b21f296c13. Local source blobs: audit_explain.py 814353615e6b934773722451ad195375f22874c3; test_audit_explain.py 0747816b7396b2a2f39367218fa68f9bd2cd98ab. Both match the published blobs at 36ffcfee98f7cb8192476298ac856552c5ae46f5.

The positive output is PINNED_STRUCTURE_MATCH with reported_kernel_verdict ACCEPT and proof_replayed, lean_replayed, upstream_checked and physical_verified all false. No new mathematical verdict. The report digest binds bytes, not truth.

## Pending

AG-F02 NAT-FUN provenance reproduction is not completed: the environment cannot clone/build RLT directly. No Rust build, fresh proof replay, CI run or independent review was performed. The current prototype does not validate all field values or bind the proof/axiom/citation digests to files; it checks their shape only. Do not integrate into authoritative verification or expose it as a service. Next: independent review and live executable reproduction of AG-F02, then specify typed report corrections before extending the adapter.

# RLT-AGENT-000f — Typed definition provenance

Status: specification; implementation and independent acceptance pending.
Baseline: d93c87eac801e40f240cf2bf1f35b423c5c85a78 (rlt-agent-000e).

## Evidence

AG-F02 reproduced by GitHub Actions run 45966887419, job 140397580227.
https://github.com/thinkthoughts/rlt/actions/runs/45966887419
Full verification run 45966887103 also succeeded.
Artifact 15182783534, 17 files, 13355 bytes, runner-reported archive SHA-256
2c5535e6bc6420b968a648c73f76353f63c2de9b027ed8f9850f4f0dbed1d557.
Archive not independently rehashed; retention 14 days.
Original failed run 45930341822 is retained as evidence of invalid synthetic
inputs, not confirmation of the explain defect. Revision d93c87e fixed
record IDs/filenames and quoted definition pins without changing criteria.

F3_double_two.rlt accepted; correct definition pin
8dabb459817f7b225419456080a4e81eb675daf9a830dba529192ff50cfbc2f7.
Correct/omitted/wrong pin results: STALE(3)/CURRENT(0)/STALE(3);
all three kernel verdicts ACCEPT; all definition reports null.
Dependency root ACCEPT, definitions_used [], fingerprint definitions none.

## Scope and design

1. Carry typed provenance in CheckOutcome from the definition loader and
   elaboration, not from report strings or unchecked header claims. Distinguish
   NAT-DEF and NAT-FUN, preserve file path, verified digest, used names and
   registration authority. Establish exact timing of name collection and
   include attached NAT-FUN definition identity even if used names are empty.
2. Populate explain's definition object and pin comparison from typed data.
   Preserve the existing file/sha256/used JSON shape and NAT-DEF golden bytes.
   Preserve human check report bytes where feasible. No change to mathematical
   kernel, elaboration rules, registration policy or verdict authority.
3. An exploratory outcome must never acquire registered authority from its
   provenance object. Malformed or rejected outcomes must not imply acceptance.
   Preserve existing early failure handling; inspect all outcome constructors.
4. Separate legacy statement fingerprint/1 projection from corrected
   provenance. Do not change existing fingerprint bytes, theorem-library
   identities, composition behavior or serialized dependency contracts silently.
   Scope first implementation to explain, with deps legacy projection explicit
   and tested. A separate reviewed identity-format milestone must decide how
   corrected NAT-FUN provenance enters dependency fingerprints and migration.
5. Do not claim that source registry fidelity or Lean references independently
   establish semantic correspondence.

## Acceptance obligations

F01 Correct pin: CURRENT/0, kernel ACCEPT.
F02 Omitted pin: STALE/3, kernel ACCEPT.
F03 Wrong pin: STALE/3, kernel ACCEPT; current digest equals loaded file digest.
F04 All three reports identify definitions/double.rlt, verified SHA-256 and
    actual unfolded names; no hardcoded spec/definitions.rlt for NAT-FUN.
F05 Existing NAT-DEF golden outputs byte-identical; no-definition case unchanged.
F06 Registered NAT-FUN file bound even without use; test rejection and
    exploratory authority boundaries, including malformed early outcomes.
F07 Existing fingerprint/1 and library/compose regression fixtures unchanged;
    document remaining legacy omission explicitly.
F08 Rust fmt, clippy, full verification and focused regression run succeed.
    Preserve original reproduction as historical evidence; use separate
    post-fix expectations, not rewritten historical results.
F09 No kernel changes; exact diff reviewed; independent acceptance pending.

Source references at baseline: cli/src/lib.rs CheckOutcome and check_inner;
cli/src/explain.rs kernel_fields, finish and serializers; cli/src/deps.rs
statement_parts and fingerprint_text. The initial implementation should
inspect all consumers and constructors before edits.

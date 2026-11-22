# RLT-DEF-001a — Canonical definition registry: implementation report

**Status.** Implemented on branch `rlt-def-001a`, awaiting an independent read-only adversarial audit. Not merged, no PR.
**Baseline.** `main` at `58e1c70de14ae63dfea4078d8df34407103340da`.
**Design.** `RLT-DEF-001-PROPOSAL.md` revision 2, decisions R1–R6 as approved (2026-11-22). This stage implements 001a only. Predicate definitions (001b) and definition-dependence analysis (001c) wait for their own reviews.

## 1. What changed

| File | Change |
|---|---|
| `spec/definitions-registry.yaml` | **new**: the registry (DOUBLE, QUADRUPLE, read-only NAT-DEF), format `rlt-definitions-registry/1`, canonical text `rlt-function-definition/1` |
| `cli/src/defreg.rs` | **new**: `PINNED_DEFINITION_FILES`, `PINNED_DEFINITIONS_REGISTRY_SHA`; registry loader with rules I1–I8; strict YAML scan; registration of a loaded file (I4–I6); expansion identity; `rlt definitions` |
| `cli/src/lib.rs` | `load_function_definitions` is now authoritative: steps 1–4 unchanged (`read_function_definitions`), step 5 registration; `_as` variants taking `Authority`; `check_file_exploratory`; `EXPLORATORY = 4`; citations always checked authoritatively |
| `cli/src/prove.rs` | `prove_source_as`, `prove_report_as`, `certify_as`; `ProveOutcome.exploratory`; exploratory certificate line; registered output unchanged |
| `cli/src/discover_defs.rs` | `discover_as`; exploratory records marked and exit 4; `--verify` loads definitions authoritatively |
| `cli/src/main.rs` | `rlt definitions [ID] [--format text|json]`; `--unregistered-definitions` (check, prove, discover --definitions only); usage text |
| `cli/tests/def001a.rs` | **new**: DR-01 … DR-22 and a committed-registry test (23 tests) |
| `cli/tests/definitions.rs` | three tests run their scratch (unregistered) files exploratory (§3.2) |
| `scripts/check-definition-identities.py` | **new**: independent Python recomputation of every registered identity and pin |
| `scripts/compare-reports-def001a.sh` | **new**: byte comparison of every remaining command between two binaries |
| `scripts/verify.sh` | new stage "definition registry, RLT-DEF-001a" |
| `docs/MANUAL.md` | chapter 14: "Registered definitions (RLT-DEF-001a)", with a checked console example |
| `docs/RLT-PROVER-002.md` | §9: D2 marked resolved by RLT-DEF-001a |
| `docs/RLT-DEF-001a-REPORT.md`, `verification/RLT-001-results.txt` | this report; the verification record of the full run (§5) |

Unchanged, byte for byte against `58e1c70` (`git diff --stat 58e1c70 -- …` empty): `kernel/`, `syntax/`, `spec/robinson-q.rlt`, `spec/peano.rlt`, `spec/peano-arith.rlt`, `spec/axioms.yaml`, `spec/definitions.rlt`, `definitions/`, `tests/` fixtures, `research/`, `lib/`, `statements/`, `compose/`, `certificates/`, `sources/`, `index.html`, `index_261111.html`, `init.html`, `RH/`.

## 2. Design as implemented

### 2.1 Identities

| Level | Computed by | Example (`double`) |
|---|---|---|
| File | SHA-256 of the file bytes | `definitions/double.rlt` `8dabb459…`; `definitions/quadruple.rlt` `7f463b05…` |
| Content | SHA-256 of `rlt-function-definition/1` canonical text plus `deps` (unchanged from RLT-PROVER-002) | `066fb635c06d85cf6cbd288468b7d992566056793f44375b47f0ac30c09a1c16` in both files |
| Expansion (reported only) | SHA-256 of `rlt-function-expansion/1\narity N\nbody <kernel form, parameters renamed p0, p1, …>\n` | `c8add4bd93ab72866a59909d262b8e81bb560e49f0d006b569a5cafb2cfab949` |

Two definitions are one definition exactly when their content identities are equal. One name has one content identity across the registry (I7). Expansion identity never lets two definitions share a name. DR-09 shows `double(y) := y + y` sharing `double(x) := x + x`'s expansion identity while being refused under the same name.

### 2.2 Rules

I1 pinned registry bytes; strict keys (unknown, duplicate, alias, anchor, tag refused); version-1 `format` and `canonical_text`. · I2 entries equal `PINNED_DEFINITION_FILES`. · I3 unique ids, unique plain paths. · I4 header path and SHA-256 equal a `functions` entry. · I5 file id, convention and domain equal the entry's. · I6 computed content identities equal `defines`, exactly. · I7 one content identity per name; names disjoint from NAT-DEF names (rules and axiom names refused by the existing parser). · I8 the NAT-DEF entry equals `PINNED_DEFINITIONS` and lists the file's names.

Every NAT-FUN load runs these in this order: path rules, bytes against the header hash, parse, content identities (all with their RLT-PROVER-002 messages), then registration. A broken registry refuses every NAT-FUN check in both modes.

### 2.3 Authority

| Mode | How requested | Unregistered file | Registered file |
|---|---|---|---|
| Authoritative | default, for every command and every library entry point | MALFORMED, exit 2 | today's report, byte-identical |
| Exploratory | `--unregistered-definitions` on `rlt check`, `rlt prove`, `rlt discover --definitions` only | kernel checks; accepted → `EXPLORATORY`, exit 4; certificate line `; UNREGISTERED DEFINITIONS — …` | identical to authoritative (DR-20) |

Downstream authority comes from rechecking registration against the definitions bytes, never from a caller's report text, header line or exit code:

- `explain`, `deps`, `compose`, `residues`, `discover --verify` and `rp`/`points` reach the checker only through `check_file_outcome`, `check_preread_outcome` and `check_bytes_outcome`, which are authoritative. DR-17 also asserts this structurally: those modules never name an exploratory entry point.
- A citation inside any check, exploratory included, is checked authoritatively (DR-15).
- `discover --verify` loads the record's definitions file authoritatively before any row (DR-16).
- Removing the exploratory certificate line changes nothing (DR-18).

## 3. Tests

### 3.1 Adversarial tests (`cli/tests/def001a.rs`), all pass

| Id | Case | Result |
|---|---|---|
| — | committed registry: identities `066fb635…`, `8a351369…`; `double` one definition in two files; all 7 committed certificates and 126 discovery certificates ACCEPT | pass |
| DR-01 | D2 audit case (`(id DOUBLE)`, `double(x) := x + 1`, kernel-valid certificate for `double(2) = 3`): refused by `check`, `prove`, `discover --definitions`, `deps` | pass |
| DR-02 | byte copy of DOUBLE elsewhere: refused, message recognizes `double@066fb635…` | pass |
| DR-03 | layout-only variant: same content identity, new file identity, refused | pass |
| DR-04 | registered file edited, header updated: refused in both modes | pass |
| DR-05 | registry edited, binary unchanged: I1 on every NAT-FUN check; `rlt definitions` exit 2 | pass |
| DR-06 | registry and file edited together: I1; with the registry pin updated, I2 | pass |
| DR-07 | file id / domain differs from entry: I5 | pass |
| DR-08 | wrong, omitted or extra `defines`: I6 | pass |
| DR-09 | `double` with two content identities: I7; expansion identities equal | pass |
| DR-10 | duplicate id, shared path, `./` and `..` spellings in registry (I3) and in headers (path rules) | pass |
| DR-11 | registry symlink (I1); definitions symlink | pass |
| DR-12 | unknown keys, duplicate key, other `format` or `canonical_text`, anchors: I1 | pass |
| DR-13 | NAT-DEF entry hash or names differ: I8 | pass |
| DR-14 | registered names `dvd`, `free235`, `refl`, `add_zero`: refused | pass |
| DR-15 | exploratory certificate cited, from an authoritative and from an exploratory check: refused | pass |
| DR-16 | exploratory discovery record (exit 4): `--verify` refuses it, also with its EXPLORATORY lines removed | pass |
| DR-17 | `deps` and a statement record (`explain`) over the exploratory certificate: refused with the registration message; structural containment check | pass |
| DR-18 | exploratory header line removed: still refused authoritatively | pass |
| DR-19 | exploratory output to a tracked path: refused, file unchanged | pass |
| DR-20 | flag with registered definitions: every certificate report and every goal's prove report and certificate identical | pass |
| DR-21 | malformed unregistered file: existing parse message, both modes | pass |
| DR-22 | Python cross-check agrees; each of 5 mutated canonical-text fields detected | pass |

### 3.2 Existing tests

Three tests changed, all in `cli/tests/definitions.rs` (the proposal estimated four; `case8_an_attempted_axiom_is_refused` needed no change because its assertions are refusals):

| Test | Change |
|---|---|
| `case2_unfolding_under_quantifiers_never_captures` | proofs over the scratch file run exploratory; asserts exit 4, the same unfolded goals, and refusal without the flag |
| `case5_a_changed_definition_invalidates_the_proof` | exploratory acceptance (exit 4) and authoritative refusal, then the same tamper refusals |
| `audit_d1_many_uses_are_refused_with_an_error_not_oom` | checks run exploratory so unfolding is reached; the D1 refusal unchanged |

Full suite: **410 tests, 0 failed** (387 at baseline + 23 new), 23 test binaries, from `scripts/verify.sh`.

## 4. Baseline report comparison (R1)

Baseline binary built from `58e1c70` (SHA-256 `d7f3d020…`), run on a worktree of `58e1c70`; new binary built from this branch at `9ccc8f8` (SHA-256 `1ddf3452…`, the code of the final commit; later commits add only this report and the verification record); generated certificates (`tests/rlt006/gen`, `tests/rlt007s1/gen`, `tests/rlt007s4/gen`) identical in both trees. `scripts/compare-reports-def001a.sh` (output and exit code, byte for byte):

| Scope | Compared | Differ |
|---|---|---|
| `check` of every committed `.rlt` outside `tests/`, `prove` of every committed goal (22), `discover --verify` of both committed records, `explain` text and json of all 65 statement records, `deps` text and json of all 159 library entries, `deps --verify-lib` text and json, `compose compose/h45.yaml` | 617 | **0** |
| `scripts/compare-reports.sh`: `check` of every `.rlt` under `tests/` | 2846 | **0** |
| `validate`, `rp math0/math5/math29`, `points` ×2, `residues`, `chem` ×2, every `written` case, `lean tests/accept/*.rlt` | all | **0** |

Usage text (`rlt` with no valid command) gains the `definitions` line and the `--unregistered-definitions` option; it is help output, not a report.

`rlt validate` is unchanged on every root. An earlier draft added registry problems to `validate`. The golden test `a2_c09_evidence_and_kinds` showed this changes `validate` on scratch roots that copy `spec/` without `definitions/`, so the hook was removed. Registry consistency is reported by `rlt definitions` and enforced by every NAT-FUN load.

## 5. Registry identity checks and full verification

- `rlt definitions`: CONSISTENT; registry SHA-256 `fea82f7d0ebf52a02866c9b0777257fa7a2100862af40e3223f1abfedb4fbc40`.
- `scripts/check-definition-identities.py`: the registry pin, 3 entries and 3 definitions (content and expansion identities) are recomputed with no RLT code, from its own parse of the definitions files, and equal `rlt definitions --format json`. All 5 mutated canonical texts are detected.
- `scripts/verify.sh /home/claude/checkouts`: sources fetched by `scripts/fetch-sources.py` at their pinned commits (10 repositories, `git fsck --full`), plus the two archived/transcribed files equal to their pins. Lean 4.34.0-rc2 via elan.
  - Result: **`RESULT all checks hold`**, 0 SKIPPED, 0 UNEXPECTED.
  - The run covered fmt, clippy `-D warnings`, 410 tests, every Lean companion, the D5/D6/H1 Lean declaration correspondence with `leanchecker`, the H1 attack, `leanchecker --fresh`, and the source excerpts.
  - It also ran the registry stage: the D2 case gave prove 2, exploratory prove 4, check 2, exploratory check 4.
  - Full log: `verification/RLT-001-results.txt`.

## 6. Remaining risks and review questions

1. **Trust placement.** `defreg.rs` joins the trusted inputs beside the axiom-set loader. A defect there could admit an unregistered file; DR-01 … DR-14 and the Python cross-check are the evidence. The auditor should read `registration_of` and `load_definitions_registry_pinned` closely.
2. **`Pins` is public.** `load_definitions_registry_pinned` accepts other pins so tests can reach each rule. Every authoritative path calls `load_definitions_registry` (this binary's `PINS`); the auditor should confirm no other call site exists outside tests.
3. **Cost.** Each NAT-FUN load now reads the registry and both registered files. `discover --verify research/definition-discovery` takes ≈4.4 s, up from ≈1.5 s. Caching was left out for simplicity.
4. **Composition coverage.** DR-17 shows behaviorally that `explain` and `deps` refuse; for `compose` the evidence is structural (it calls only `check_preread_outcome`). The committed bundle is NAT-DEF, and a NAT-FUN bundle fixture would need its own library entries. Question: add a NAT-FUN compose fixture in 001b, when predicates create a use for one?
5. **Registry YAML parser.** Strictness comes from an event-level scan (duplicates, aliases, anchors, tags, complex keys) before `yaml-rust2` loads the value; the registry bytes are pinned in any case.
6. **Exit code 4** is new for `check`, `prove` and `discover`. Only explicit exploratory requests produce it.
7. **`validate` scope** (§4): registry problems surface through `rlt definitions` and NAT-FUN checks. Should a later milestone add the registry to `validate` together with updating the scratch-root tests?
8. **Proposal deviations:** three existing tests changed rather than four (§3.2), and `validate` was left unchanged (§4).

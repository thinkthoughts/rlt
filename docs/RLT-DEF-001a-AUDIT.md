# RLT-DEF-001a — Independent adversarial audit

**Verdict: PASS WITH NON-BLOCKING NOTES — MERGE**

| | |
|---|---|
| Repository | `thinkthoughts/rlt`, branch `rlt-def-001a` |
| Head audited | `24089a30b7ff20dba9a503b6dc700634ae226c08` |
| Baseline | `58e1c70de14ae63dfea4078d8df34407103340da` (merge-base confirmed) |
| Method | fresh clone; implementation read first (`defreg.rs`, the `lib.rs`/`prove.rs`/`discover_defs.rs`/`main.rs` diffs, the registry), then behavioral attacks in disposable copies of the tree, then a full replay |
| Mode | read-only: the audited clone ends with a clean `git status`; all attacks and the `verify.sh` run used separate copies |
| Binaries | baseline `d7f3d020…6a6cb8`, branch `1ddf3452…93e447`, rebuilt independently; both equal the SHA-256 values in the implementation report (reproducible build) |

Release-blocking defects found: **none**. Six hardening opportunities and five documented limitations follow. Each finding is marked **[B]** (behaviorally demonstrated) or **[I]** (code inspection).

**Status after the audit (§8):** H1 is resolved by commit `fca6da9`. H2–H6 and L1–L5 are carried forward as documented follow-up items.

---

## 1. Summary of evidence

The authority decision reduces to two compiled-in constants: `PINNED_DEFINITIONS_REGISTRY_SHA` (whole registry bytes) and `PINNED_DEFINITION_FILES` (id, path, SHA-256, in order). Every NAT-FUN load goes `read_function_definitions` (path rules → bytes hashed once → parse → content identities) → `load_definitions_registry` (always `PINS`) → `Registry::registration_of(defs)`. Registration is decided from `defs.file` (the validated plain path string) and `defs.file_sha256` (the hash of the exact bytes elaborated), so the bytes used and the bytes judged are the same bytes. All downstream consumers (`explain`, `deps`, `compose`, `discover --verify`, records, `lean`, citations) reach the checker only through the `RegisteredOnly` entry points; `ExploratoryAllowed` is constructed in exactly one place (`main.rs`, gated to `check`, `prove`, `discover --definitions`), and nested citation checks hard-code `RegisteredOnly`.

| Area | Attempts | Outcome |
|---|---|---|
| 1. Identity and registry authority | 14 | all refused or fail-closed |
| 2. Trusted call paths | full call-graph trace | production uses `PINS` only |
| 3. Exploratory containment (incl. NAT-FUN compose) | 11 | all refused; behavioral compose control passes |
| 4. Certificate tampering | 5 | all refused |
| 5. Backward compatibility | 617 + 2846 + 9 commands, plus 3 supplementary sets | 0 differences |
| 6. Registry parser | 37 cases | every non-equivalent input refused |
| 7. Replay | full `verify.sh` with Lean | `RESULT all checks hold`, 0 SKIPPED |
| 8. `validate` boundary | 4 | reporting gap only; no route to an authoritative ACCEPT |

---

## 2. Attack log (behavioral)

`evil/double.rlt` below is the D2 file: `(id DOUBLE)`, `double(x) := x + 1`. `bad.rlt` is a kernel-valid derivation of `double(2) = 3` over it.

### 2.1 Definition identity (priority 1)

| # | Attack | Expected | Observed |
|---|---|---|---|
| A1 | `check bad.rlt` over the D2 file | refuse | exit 2, "not a registered definitions file" |
| A1′ | same, `--unregistered-definitions` | exploratory | exit 4, `EXPLORATORY` |
| A2 | byte-identical copy of `double.rlt` at `evil/copy.rlt` | refuse (path identity) | exit 2; message recognizes `double@066fb635…` |
| A3 | edit `definitions/double.rlt` in place (`x + 1`), header SHA updated | refuse in both modes | exit 2 / exit 2 (registry load fails on the registered file's hash) |
| A3c | comment-only edit of the registered file | refuse | exit 2 both modes |
| A4a | `definitions/double.rlt` → symlink to an identical copy | refuse | exit 2 (symlink); `rlt definitions` exit 2 (I3) |
| A4b | `definitions/` directory symlink | refuse | exit 2 |
| A4c | registry → symlink to an identical copy | refuse | exit 2 (I1) |
| A4d | registry as a hard link (regular file, same bytes) | accept (same bytes) | CONSISTENT, as expected |
| A5 | one appended newline in the registry | refuse every NAT-FUN check | exit 2 (I1); non-NAT-FUN checks still ACCEPT |
| A5b/c | registry missing / registry is a directory | refuse | exit 2 (I1) |
| A6 | proof over DOUBLE using `quadruple` (a name registered in QUADRUPLE only) | refuse | exit 2, "not a registered function of DOUBLE" |
| A7 | `Definitions/double.rlt` (case variant, Linux) | refuse | exit 2, entity not found |
| A8 | Cyrillic-`о` lookalike path holding identical bytes | refuse | exit 2, unregistered |

### 2.2 Exploratory containment and tampering (priorities 3–4)

| # | Attack | Expected | Observed |
|---|---|---|---|
| B1 | `prove g.goal` over D2 file | refuse | exit 2, no certificate written |
| B2 | `prove --unregistered-definitions` | exploratory cert | exit 4; cert line 3 carries `UNREGISTERED DEFINITIONS` |
| B3 | strip the marker line, `check` | refuse | exit 2 (registration re-decided from bytes) |
| B4 | rewrite comments to the normal registered certificate header | refuse | exit 2 |
| B5 | repoint header at registered `double.rlt` keeping the steps | refuse | exit 1, REJECT (unfolded goal becomes `2 + 2 = 3`) |
| B6 | cite the stripped cert from a NAT-DEF proof, plain and `--unregistered-definitions` | refuse both | exit 2 both ("not an accepted theorem") |
| C1 | `discover --definitions evil/double.rlt` | refuse | exit 2 |
| C2 | same, exploratory | exploratory record | exit 4; 108 rows; manifest and summary marked |
| C3–C4 | `discover --verify` on that record, marked and with EXPLORATORY lines removed | refuse | exit 1 both, "not a registered definitions file" |
| C5 | record rewritten to name registered `double.rlt` + its SHA, manifest SHAs fixed | refuse | exit 1, every row "kernel did not accept the certificate" |
| D1 | `deps` over the exploratory cert | refuse | exit 2 |
| D2 | `lib/` entry (fingerprint, identities.tsv row, all fields valid) pointing at the exploratory cert; control entry pointing at registered F3 | control 0, attack refused | control `deps` exit 0 / current; attack `deps` exit 2, `--verify-lib` lists it MALFORMED |
| D3 | statement record over the exploratory cert; control over F3 | control ACCEPT, attack refused | control "proof ACCEPT"; attack `explain` exit 2, KERNEL ACCEPTANCE MALFORMED |
| D4 | reading-point relationship labels on a NAT-FUN proof (registered and exploratory) | refuse | exit 2 both (pre-existing structural refusal) |
| **D5** | **NAT-FUN composition, behavioral.** Bundle `convention: NAT-FUN`, one layer-0 node, approved-spec and named-result rows added in the scratch tree. Control node → `lib` entry over registered F3; attack node → `lib` entry over the exploratory cert, with a valid NAT-FUN spec probe | control COMPOSED, attack refused | control `COMPOSED`, exit 0; attack `REFUSED (kernel)`, exit 1, registration message |

Conclusion for priority 4: every authoritative outcome depends on the definitions path string plus the SHA-256 of the bytes elaborated; markers and comments are advisory only (DR-18 confirmed independently).

### 2.3 Registry parser (priority 6)

Driven through `load_definitions_registry_pinned` from a scratch integration test (in a copy of the tree only), with pins recomputed per case so each input reaches the parser.

**Refused (fail-closed):** BOM; duplicate top-level key; duplicate nested key (identical values); `id` + `"id"`; duplicate `name` in `defines`; anchor; alias/merge `<<`; `!!str` tag; local tag; complex key `? [a, b]`; duplicate key inside a flow mapping; second YAML document; content after `...`; upper-case hex; integer `id`; boolean `kind`; null `convention`; unknown key; `"id "` (trailing space); literal block scalar containing a newline; zero-width character in `id`; tab indentation; non-UTF-8 byte; pins in swapped order; pins with an entry missing.

**Accepted, each semantically identical to the committed registry:** CRLF line endings; explicit `? format` key; flow-style NAT-DEF entry; single- and double-quoted scalars; folded `>-` path; `.` escape in a quoted path; trailing `...`. Consistent pins after dropping QUADRUPLE: accepted, as designed. See H5.

### 2.4 Trusted call paths (priority 2) [I]

`load_definitions_registry_pinned` call sites: `defreg.rs:372` (wrapper passing `PINS`) and `cli/tests/def001a.rs`. `load_definitions_registry` call sites: `lib.rs:275` (every NAT-FUN load) and `definitions_report`. `Authority::ExploratoryAllowed` is constructed only at `main.rs:73`. `check_inner`'s recursive citation call passes `RegisteredOnly` explicitly. `certify_as` accepts exit 4 only when the caller's authority is already exploratory. `discover_defs::verify` loads with `RegisteredOnly` and replays rows via the authoritative `check_bytes_outcome`.

### 2.5 `validate` boundary (priority 8)

| Probe | Observed |
|---|---|
| registry broken (one byte appended) | `validate` VALID, exit 0; `rlt definitions` exit 2; every NAT-FUN check exit 2 **[B]** |
| exploratory `lib/` entry present | `validate` VALID **[B]** (lib is `deps --verify-lib`'s scope, unchanged from baseline) |
| exploratory statement record present | `validate` VALID **[B]**; `explain` exit 2 |
| relationship proof under NAT-FUN | refused by the checker before registration is reached **[B]** |

`validate` covers sources, conventions, axiom sets, reading-point relationships, points and residues. The only proofs it replays are relationship proofs, through the authoritative `check_file_outcome`, and NAT-FUN refuses reading-point labels. So leaving the registry out of `validate` limits what `validate` reports; it opens **no** route to an incorrect authoritative ACCEPT. (L1.)

---

## 3. Findings

### Release-blocking defects

None.

### Hardening opportunities

**H1. Multi-file `check --unregistered-definitions` lets exit 4 mask REJECT and MALFORMED. [B] — RESOLVED (§8).**
`main.rs` aggregates with `worst = worst.max(r.code)`; 4 is numerically above 1 and 2.

```console
$ printf '(rlt-proof (id X))' > evil/broken.rlt
$ rlt check --unregistered-definitions evil/broken.rlt evil/bad.rlt; echo $?
4        # expected 2: one input is malformed
$ rlt check --unregistered-definitions evil/bad.rlt evil/broken.rlt tests/reject/<any>.rlt; echo $?
4        # expected 2
```
Authority impact: none (4 is never ACCEPT and every consumer re-decides). A script reading exit 4 as "all kernel-accepted, exploratorily" would misread the batch.
*Smallest fix:* rank severity in the `check` loop, e.g. `fn rank(c: i32) -> u8 { match c { 0 => 0, 4 => 1, 1 => 2, _ => 3 } }`, keep the code with the highest rank; add one test.

**H2. Public test seam can mint a `Registered` verdict for unpinned content. [I]**
`Pins`, `load_definitions_registry_pinned`, and `Registry`'s fields are `pub`; `Registry::registration_of` on a registry loaded with arbitrary pins returns `Ok(id)`. No production path reaches it (§2.4), so this is a library-API exposure only.
*Fix:* make `load_definitions_registry_pinned` `#[doc(hidden)]` behind a `test-pins` cargo feature enabled only by the test target, or record in `Registry` whether it was loaded with `PINS` and have `registration_of` refuse otherwise.

**H3. Exploratory discovery manifest labels an unregistered file "(registry DOUBLE)". [B]**
`manifest_head` prints `(registry {d.id})` from the file's self-declared id. In C2 the manifest reads `# definitions evil/double.rlt sha256 be81c131… (registry DOUBLE)` next to the EXPLORATORY line.
*Fix:* when exploratory, print `(declares id DOUBLE; unregistered)`.

**H4. Integrity failure of a registered file is labelled I4. [B]**
A3 reports `definitions registry (I4): registered file DOUBLE: … has SHA-256 …`. The failing condition is the registered file's bytes against its entry (I2/I6 class); I4 is about the header of the file being checked. Consequence worth stating in the manual: one tampered registered file makes every NAT-FUN check, exploratory included, MALFORMED (fail-closed and consistent with report §2.2).
*Fix:* relabel to the matching rule or a dedicated "registered file integrity" tag.

**H5. Parser accepts syntactic variants of identical content. [B]**
Folded scalars, quoted scalars with `\u` escapes, flow mappings and explicit keys all load. Bytes are pinned, so this matters only to a reviewer reading a future registry edit (e.g. `"definitions/double.rlt"`).
*Optional fix:* in `strict_yaml`, admit only block-style mappings and plain scalars (check `TScalarStyle::Plain`), and require ASCII in `id` and `path`.

**H6. Comparison script counts identical failures as matches. [I]**
`compare-reports-def001a.sh` byte-compares output and exit status without asserting expected exits, so an input both binaries fail on the same way (for example `compose` or `deps --verify-lib` with `tests/*/gen` absent) passes as "same". In this audit the generated trees were present and byte-identical in both roots, and the outcomes were substantive (compose exit 0; 159/159 lib entries; 126 + 9 discovery rows). The script also leaves out `discover --definitions` regeneration, which goes through the changed `discover_as`; I compared it separately (identical).
*Fix:* assert `compose compose/h45.yaml` exit 0 and `--verify-lib` exit 0 in the new tree; add regeneration of both registered discovery records.

### Documented limitations

**L1. `validate` reports nothing about the registry.** It stays VALID with a broken registry (§2.5). This is a scope limit; enforcement is in every NAT-FUN load and `rlt definitions`, and verify.sh runs the latter. Report §6 q.7 already raises adding it with the scratch-root test update; I concur that it can wait.

**L2. Case-insensitive filesystems.** On macOS/APFS default, `Definitions/double.rlt` would open the registered bytes yet compare unequal as a path string, so it is treated as unregistered: fail-closed, exploratory over the real registered bytes. Inspection only (Linux host).

**L3. Library and composition identity bind the unfolded statement only (pre-existing, RLT-PROVER-002).** `deps` shows `definitions none` in a NAT-FUN fingerprint; `lib` `title` is unchecked display text. Demonstrated on both baseline and branch binaries: a `lib` entry titled `"double(2) = 3"` over a registered proof of `2 + 1 = 3` is `current`, exit 0. Kernel content stays correct; the written reading is outside library identity. Natural home: RLT-DEF-001c (definition dependence) or a library-format revision.

**L4. NAT-FUN composition is supported but unexercised by committed fixtures.** D5 shows it works (control) and contains exploratory input (attack). This answers report §6 q.4: a committed NAT-FUN compose fixture is worthwhile, and D5's two bundles are a minimal template.

**L5. Cost.** Each NAT-FUN load re-reads and re-checks the registry and both registered files (report §6 q.3). Observed slowdowns are consistent with the report; correctness is unaffected.

---

## 4. Backward compatibility (priority 5)

Run independently: baseline binary on a `58e1c70` worktree, branch binary on a fresh clone of `24089a3`, `tests/rlt006/gen` (630), `tests/rlt007s1/gen` (44), `tests/rlt007s4/gen` (1485) materialized by each binary and `diff -r` identical.

| Scope | Compared | Differ |
|---|---|---|
| `compare-reports-def001a.sh` extended set | 617 | 0 |
| `compare-reports.sh` proof files under `tests/` | 2846 | 0 |
| `validate`, `rp` ×3, `points` ×2, `residues`, `chem` ×2, every `written` case, `lean tests/accept/*.rlt` | all | 0 |
| *Supplementary:* `discover --definitions` regeneration of DOUBLE (129 files) and QUADRUPLE (249 files), summary + full record dir | 2 | 0 |
| *Supplementary:* 20-file multi-file `check` | 1 | 0 |
| *Supplementary:* `--unregistered-definitions` vs default on all 7 definitions certificates and on DOUBLE discovery (same out dir) | 8 | 0 |

Substantive outcomes in the new tree: `compose compose/h45.yaml` exit 0; `deps --verify-lib` 159 of 159; `discover --verify` 126 and 9 accepted; 22 goals.

Unchanged against baseline (`git diff --stat` empty): `kernel/`, `syntax/`, all of `spec/` except the new registry, `definitions/`, `tests/`, `research/`, `lib/`, `statements/`, `compose/`, `certificates/`, `sources/`, `lean/`, `conventions/`, `readingpoints/`, `points/`, `residues/`, `RH/`, `index.html`, `index_261111.html`, `init.html`.

## 5. Verification replay (priority 7)

| Check | Result |
|---|---|
| `scripts/fetch-sources.py` | 10 repositories at pinned commits, `git fsck --full`; arXiv e-print and `bgp212.pdf` equal to their pins |
| `scripts/verify.sh /home/claude/checkouts` (Lean 4.34.0-rc2 via elan, in a scratch copy) | **`RESULT all checks hold`**, exit 0, 0 SKIPPED, 0 UNEXPECTED |
| `cargo test` within verify.sh | 410 passed, 0 failed, 23 binaries |
| Registry stage | CONSISTENT; Python cross-check equal; 5/5 mutations detected; D2 case 2/4/2/4 |
| Lean companions, D5/D6/H1 declaration correspondence, H1 attack | hold |
| `leanchecker --fresh` (ProverDefinitions.lean) | replays every constant |
| Source excerpts | hold |
| `scripts/check-definition-identities.py` run separately | equal to `rlt definitions --format json` |
| Manual `sha256sum` of registry, both definitions files, `spec/definitions.rlt` | equal to the compiled-in pins |

Skipped: nothing. Note: a standalone `cargo test --workspace` run outside verify.sh initially failed two Lean-dependent tests only because Lean was absent (`RLT_ALLOW_NO_LEAN` guard). After installing Lean, the verify.sh run passed them; a second standalone run was stopped by the sandbox time limit and was superseded by the verify.sh run.

## 6. Answers to the implementation report's review questions

1. *Trust placement* — `registration_of` and the loader read correctly; no admission path found (§2.1–2.4).
2. *`Pins` public* — confirmed: no production call site; see H2 for an optional seal.
3. *Cost* — accepted as is.
4. *Compose coverage* — now behaviorally demonstrated (D5); committing a fixture is recommended (L4).
5. *Parser* — strict on everything with semantic effect; see H5.
6. *Exit 4* — sound for single inputs; fix aggregation (H1).
7. *`validate` scope* — reporting gap only (L1).
8. *Deviations* — both are reasonable; I found nothing that depends on them.

## 7. Verdict

**PASS WITH NON-BLOCKING NOTES — MERGE.** The registry closes D2: every authoritative NAT-FUN outcome is bound to compiled-in path and byte pins, decided from the bytes elaborated, and every exploratory artifact is refused by every downstream consumer whatever its markers say. H1 is the one note I'd fold into a follow-up commit soon (a three-line change plus a test); the rest can travel with RLT-DEF-001b/001c.

---

## 8. Post-audit resolution and follow-up register

Added with the H1 correction; §1–§7 above are the audit as delivered at `24089a3`.

### 8.1 H1 — resolved

Commit `fca6da9` ("RLT-DEF-001a audit H1: multi-file check exits by severity"):

- `rlt_cli::check_code_severity` and `rlt_cli::worse_check_code` rank exits ACCEPT (0) < EXPLORATORY (4) < REJECT (1) < MALFORMED (2); any other code ranks above MALFORMED. `rlt check FILE...` keeps the most severe file's code.
- On authoritative runs the codes are 0, 1 and 2, where this ranking equals the numeric maximum, so their exits are unchanged. Per-file reports and the blank-line separator are unchanged.
- `cli/tests/def001a_h1.rs` (4 tests): the ordering itself; each fixture alone; every ordered pair and triple of registered, plain, exploratory, rejected and malformed files (80 combinations) in both modes, asserting the exit and that stdout equals the single-file reports joined; the audit's reproductions (`malformed, exploratory` → 2; `exploratory, malformed, rejected` → 2). Against the previous aggregation, 2 of the 4 tests fail; with the correction, all 4 pass.
- `docs/MANUAL.md`: one sentence on the multi-file exit.

Verification of the correction: §8.3.

### 8.2 Carried forward

| Item | Evidence | Summary | Suggested home |
|---|---|---|---|
| H2 | inspection | `load_definitions_registry_pinned`, `Pins` and `Registry` fields are public; a library caller could load a registry with other pins and get `Registered` from it. No production call site. | seal behind a test-only feature, or record in `Registry` that it was loaded with `PINS` |
| H3 | tested | an exploratory discovery manifest prints `(registry DOUBLE)` from an unregistered file's self-declared id | print `(declares id …; unregistered)` when exploratory |
| H4 | tested | a tampered registered file is reported under rule I4; one such file makes every NAT-FUN check MALFORMED, exploratory included (fail-closed) | relabel; state the consequence in the manual |
| H5 | tested | the registry parser accepts syntactic variants with identical content (flow maps, quoted/folded scalars, `\u` escapes, explicit keys, CRLF, trailing `...`); bytes are pinned | admit plain block-style scalars only; ASCII `id` and `path` |
| H6 | inspection | `compare-reports-def001a.sh` counts identical failures in both trees as matches and omits `discover --definitions` regeneration | assert expected exits for `compose` and `--verify-lib`; add regeneration |
| L1 | tested | `rlt validate` reports nothing about the registry (VALID with a broken registry); enforcement is in every NAT-FUN load and `rlt definitions` | add with the scratch-root test update |
| L2 | inspection | on case-insensitive filesystems a case-variant path opens registered bytes and is treated as unregistered (fail-closed) | document |
| L3 | tested (both binaries) | NAT-FUN library fingerprints and compositions bind the unfolded statement only; `lib` `title` is unchecked text (pre-existing, RLT-PROVER-002) | RLT-DEF-001c or a library-format revision |
| L4 | tested | NAT-FUN composition works and contains exploratory input, with no committed fixture | commit a NAT-FUN compose fixture (template: §2.2 D5) |
| L5 | observed | each NAT-FUN load re-reads the registry and registered files | caching, if cost matters |

### 8.3 Verification of the correction (branch at `fca6da9`)

| Gate | Result |
|---|---|
| `scripts/verify.sh /home/claude/checkouts` (pinned sources via `scripts/fetch-sources.py`, Lean 4.34.0-rc2 via elan) | **`RESULT all checks hold`**, exit 0, 0 SKIPPED, 0 UNEXPECTED; record in `verification/RLT-001-results.txt` |
| `cargo test` within verify.sh | 414 passed, 0 failed, 24 test binaries (410 at `24089a3` + 4 H1 tests) |
| H1 regression `cli/tests/def001a_h1.rs` | 4 of 4 pass; 2 of 4 fail against the previous aggregation |
| `cargo fmt --check`, `clippy -D warnings` | clean |
| `scripts/check-manual.py` | 39 of 39 console examples match |
| `leanchecker --fresh` (ProverDefinitions.lean) | replays every constant through Lean's kernel |
| Definition registry | CONSISTENT; Python recomputation equal; 5 of 5 mutated canonical texts detected; D2 case 2 / 4 / 2 / 4 |
| Baseline comparison: `58e1c70` binary `d7f3d020…` vs `fca6da9` binary `da49aee1…`, generated trees materialized by each and identical | `compare-reports-def001a.sh` 617 compared, 0 differ; `compare-reports.sh` 2846 proof files, 0 differ; record commands, `written`, `lean`: 0 differ |
| Unchanged against `58e1c70` (`git diff --stat` empty) | `kernel/`, `syntax/`, every axiom set and `spec/definitions.rlt`, `spec/axioms.yaml`, `definitions/`, `tests/`, `research/`, `lib/`, `statements/`, `compose/`, `certificates/`, `sources/`, `lean/`, `conventions/`, `readingpoints/`, `points/`, `residues/`, `RH/`, `index.html`, `index_261111.html`, `init.html` |

H1 is resolved.

## 9. Release record

| | |
|---|---|
| Audited implementation | `24089a3` |
| H1 correction | `fca6da9` |
| Audit record and verification record | `4a085c4` (branch head, pushed) |
| Merge into `main` | `e150a66` (`--no-ff`; parents `58e1c70`, `4a085c4`) |
| Intervening `main` changes | none: `origin/main` was still `58e1c70` at merge time |
| Merged tree | `7fba341f`, identical to the tree of `4a085c4`, which §8.3 verified in full |
| Merged-tree checks | release build reproduces binary `da49aee1…` (the binary §8.3 verified); `def001a` 23 of 23 and `def001a_h1` 4 of 4 pass; `rlt definitions` CONSISTENT; Python identity recomputation equal; the 7 definitions certificates ACCEPT |

The audit (§1–§7) was independent of the implementing session. The H1 correction, §8 and this record were written by the auditing session; the correction is a 25-line change to exit aggregation, covered by its own regression tests and the full verification above, and has had no separate third review. Open follow-up items: H2–H6 and L1–L5 (§8.2). Next stage: RLT-DEF-001b, not begun.

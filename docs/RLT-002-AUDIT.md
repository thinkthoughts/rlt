# RLT-002 — Audit of the points layer and its remediation

**Scope.** Specification records under `points/` (RLT-002 T6), audited on PR #2 at `bc47c75` by an independent agent with six questions, then fixed on `rlt-002` under the approval "RLT-002 Audit Remediation — Approved". The approval treats D1 and D2 as blocking specification-integrity defects. `kernel/`, `spec/` and `PINNED_AXIOM_SETS` are unchanged against `origin/main`.

**Result.** All four substantive defects (D1–D4) and the five minor issues are fixed. The fixes come with 11 regression tests and a replay of every audit experiment onto the remediated tree. Each result keeps four layers apart:

1. **Mathematical derivation validity.** The kernel verdict, Γ and axiom set.
2. **Reading-point identity and specification.** Plain ids, kinds, scopes and bindings.
3. **Evidence references.** Each item is shown in full: type, source and location, locator, claim, referents and criteria.
4. **Physical or relational establishment.** RLT reports `unestablished` or `evidence-submitted`, and leaves the evaluation of submitted evidence to review.

## 1. Audit questions and outcomes

| Q | Audit finding at `bc47c75` | Now |
|---|---|---|
| Q1 distinct ids reported as established objects | wording held; statuses forgeable via D1, D2 | holds: the strongest status a record can hold is `evidence-submitted`, shown as "not evaluated by RLT; correspondence unestablished" |
| Q2 one referent counted twice | D4: look-alike ids counted as distinct | holds for ids that print alike; distinct names for one real referent stay outside what RLT can detect (§4) |
| Q3 unsupported relation in an accepted count | D2: any `available` source satisfied any evidence label | holds: evidence must be linked to the claim and referents, with a locator and criteria; the most a relation reaches is `evidence-submitted` |
| Q4 misleading report after edits | D1, D3: unpinned vocabulary; `rlt points` skipped the registry pin | holds: registry and status pins are checked by `check`, `rp`, `points` and `validate` |
| Q5 registry hash covers content used | excerpts covered on the `validate` path only; status file and registry metadata uncovered | holds: the registry digest covers every registry field, unknown fields are refused, and the status vocabulary has its own pin |
| Q6 validity and correspondence independent | held | holds |

## 2. Fixes

| Defect | Fix | Code |
|---|---|---|
| **D1** status vocabulary unpinned | `PINNED_STATUS_SHA` over a canonical digest of `correspondence/status.yaml`; unknown keys refused. The statuses a record may hold are fixed in the binary (`RECORDABLE_STATUS`: unestablished, evidence-submitted, inapplicable, imagined). `supported` and `rejected` stay in the vocabulary as RML source terms; a record using one is refused as an establishment judgment. Editing the vocabulary file cannot widen what a record may hold | `cli/src/lib.rs` `status_digest`, `pinned_status`, `RECORDABLE_STATUS`, `JUDGMENT_STATUS` |
| **D2** evidence checked by label only | Each evidence item states `type`, `ref`, `locator`, `claim`, `referents` and `criteria`, and has no other fields. `claim` must equal the record's claim (`physical-correspondence` for a point, the relation type for a relation, the kind for a reading-point correspondence record). `referents` must equal the record's referents in order. A record with evidence has status `evidence-submitted`. Reports print every item with its source location and "submitted, not evaluated by RLT". A refused status prints as `unestablished (record states \`…\`, refused)` | `check_status_and_evidence`, `status_shown`; `records.rs` `EvidenceItem` |
| **D3** `rlt points` skipped the registry pin | One loader, `pinned_registry`, used by `points`, `rp`, `validate` and the axiom-set loader behind `check`. The digest now also covers `title`, `url`, `commit` and `note`; unknown registry and excerpt fields are refused | `pinned_registry`, `registry_digest`; `records.rs` `load_registry` |
| **D4** confusable ids | Point, count, relation, file and reference ids use `[a-z0-9._-]`, starting and ending with a letter or digit. Every record string refuses control characters, Unicode format characters, line and paragraph separators and bidirectional controls | `records.rs` `plain_id`, `invisible`, `clean` |
| M1 newlines in list fields | List items and group members go through the same checks | `records.rs` `strings`, `ids` |
| M2 references outside `readingpoints/` | References are plain ids, so they cannot contain `/` or `..` | `ids` |
| M3 wording on refused counts | The identities line counts distinct points ("1 distinct reading point(s) in 2 bindings"); a count whose proof the kernel refuses reports "binding not reported" | `points_check` |
| M4 symlinks and special files | `read_regular` refuses symlinks and non-regular files before opening them, for every record and proof file. A points file must sit directly in the repository's own `points/` directory | `records.rs` `read_regular`; `points_check` |
| M5 laxer acceptance | `basis` required; a relation names two or more distinct points; `inapplicable` refused for physical-object, physical-relationship and observed-event points; each count report shows its convention's reading text, so an edit to that text is visible | `load_points`, `points_check` |

## 3. Verification

- **`scripts/verify.sh /home/claude/src`:** exit 0, final line "RESULT   all checks hold". This runs fmt, clippy, build, the 74 tests (golden 41, kernel 19, syntax 14), every proof file, the 68 audit regression files, `rlt points` on each points file, `rlt validate`, the Lean replay and the source-excerpt checks.
- **Regression tests (`cli/tests/golden.rs`):** `r4_d1_status_vocabulary_is_pinned`, `r4_d1_judgment_statuses_are_refused`, `r4_d2_submitted_evidence_stays_unestablished`, `r4_d2_evidence_is_linked_to_its_claim`, `r4_d3_registry_pin_is_checked_by_every_command`, `r4_d4_confusable_identifiers_are_refused`, `r4_m1_list_fields_cannot_forge_report_lines`, `r4_m2_references_stay_in_readingpoints`, `r4_m3_refused_counts_report_accurately`, `r4_m4_links_fifos_and_stray_paths_are_refused`, `r4_m5_stricter_records`.
- **Replay of the audit experiments.** Each experiment's edits were applied to a copy of the remediated tree. Where a missing `basis` alone refused a file, a valid basis was added and the file rerun, so the original defect was exercised.

| Experiment | Now |
|---|---|
| e4 forged `established` vocabulary (D1) | refused: status vocabulary differs from the pin |
| q3 `supported` relations citing RLT pages (D2) | refused: establishment judgment; evidence missing claim, referents, locator and criteria; the report shows these relations as `unestablished (record states \`supported\`, refused)` |
| e5 reworded basis and invented lab source (D3) | refused: registry differs from the pin |
| e11 unknown registry field | refused: field outside the record type |
| e2, e2b `Person.A`, Cyrillic `person.а`; zero-width id (D4) | refused: identifier rule; invisible character U+200B |
| e7 U+2028 in a description | refused: invisible character |
| e3 duplicate keys, duplicate point, YAML merge key | refused |
| q6 `rejected` status, `inapplicable` physical object, mismatched proof, repeated points | refused (12 problems) |
| e8 edited proof goal; e9 symlinked proof | refused |
| `../points/q3` reference (M2) | refused: identifier rule |
| e13 convention prose edited to "multiplication of physical objects" | VALID by design (conventions are interpretation inputs); the report now shows the edited text on the count's `convention` line |

**Pins after remediation.**

- `PINNED_AXIOM_SETS`: unchanged (PEANO-ADD, `spec/peano.rlt`, `1a3f8344…`).
- `PINNED_REGISTRY_SHA`: `f81d46db…`. `sources/registry.yaml` is unchanged; the value changed because the digest now covers `title`, `url`, `commit` and `note`.
- `PINNED_STATUS_SHA`: `80d1d09b…` (new). `correspondence/status.yaml` gained the RLT-authored status `evidence-submitted`.

## 4. Trusted assumptions and validation limits

1. **Evidence evaluation.** RLT checks that evidence is complete, refers to a registered `available` source and is linked to the exact claim and referents. Whether the evidence meets its stated criteria is left to a human reviewer. So no physical or relational claim is reported as established; `evidence-submitted` is the strongest status RLT reports.
2. **Identity of referents.** A reading-point id is a name. RLT guarantees that names which print alike are the same name. Whether two different names refer to one physical thing in the world is a fact RLT records as a claim, never infers.
3. **The pins.** The binary's three pins are trusted, along with whoever builds and reviews the binary. Changing an axiom set, the registry or the status vocabulary takes a reviewed code change.
4. **Conventions are interpretation inputs.** They have no pin. A change to a convention's symbols or elaborator changes the elaborated goal, which must then match the proof's checked goal. Its reading text has no effect on any check and is shown in every count report.
5. **Records under review.** `readingpoints/` and `points/` have no pin. They are what `rlt validate` checks, against the pinned registry and vocabulary.
6. **Source boundaries.** `check_excerpts.py` checks `available` sources with excerpts against their files. Conversation sources are pinned text with no file to compare against. An `available` source without excerpts has its location recorded but unchecked; citing it as evidence shows that location for review.
7. **Invisible-character set.** The refused set is an explicit code-point list (Cf, Zl, Zp and bidirectional controls); a character outside it would need adding by review. Free-text fields may contain non-ASCII letters, since only identifiers are restricted to ASCII.
8. **File checks.** The file-type check and the read happen one after the other. A concurrent local process swapping the file between the two is outside the threat model.
9. **Toolchain and crates.** Rust 1.97.0 and the `yaml-rust2` and `sha2` crates are trusted, as in RLT-001.

# RLT-004c — Theorem library and `rlt deps`: architecture and test review

**Status.** Proposal for review, 2026-11-11, against main at 45c94b5. Documentation only: nothing here is implemented until approved. The kernel, the 20 rules, citation checking, axiom sets, definitions, conventions, reading points and every existing proof stay as they are. No release or deployment.

**Objective.** Make existing checked theorems discoverable and reusable, each with its exact statement, assumptions, axiom set, cited dependencies and source specification, through a library index and one command:

| Command | Answers |
|---|---|
| `rlt check FILE` | Does this derivation check? |
| `rlt explain RECORD` | What source, specification and correspondence does this statement have? |
| `rlt deps ID` (proposed) | Which checked theorems, axioms and definitions does this theorem rest on? |

---

## 1. What exists today

- **Citation (M3c).** A proof cites a file by repository path and SHA-256: `(cites (LABEL "path" "sha256"))`, used by `(cite LABEL)` steps. Each citation re-checks the cited file from scratch, at those exact bytes. A cited file must be unconditional (Γ = ∅) and use the same axiom set and convention. The kernel re-validates each cited theorem, including that its axioms match the registered ones by content.
- **Limits.** `MAX_CITATION_DEPTH` = 8 citing files in one chain; `MAX_CITED_CHECKS` = 64 cited-file checks per top-level check. A file cited twice in one graph is checked twice and counts twice.
- **The largest graph today, E-T1.** E-T1 cites E_FWD and E_BWD. E_FWD cites DA01 and 22 divisibility facts (PD_*); E_BWD cites 24 non-divisibility facts (ND_* and M3c's CA02–CA04). That is 50 files, 49 cited-file checks, and a chain of 2 citing files: 77% of the check budget and 25% of the depth budget.
- **Identity today** is the pair (path, SHA-256), plus the proof id written inside the file (e.g. `M3e.E_T1`). No index names theorems across files.

## 2. Stable theorem identifiers

A library identifier names **a statement under an axiom set**, separately from any particular proof file.

| Layer | Example | Stability |
|---|---|---|
| **Library id** | `mod30.classification` | Stable. Lower-case, dotted, assigned by review; never reused for a different statement. |
| **Statement fingerprint** | SHA-256 of the canonical record `axiom set id ⏎ convention ⏎ kernel form of the checked goal ⏎ Γ` | Determined by what is proved. Changes exactly when the statement, its axiom set or its assumptions change. |
| **Proof pin** | `tests/m3e/accept/E_T1.rlt` + SHA-256 `e11712c6…` | Identifies one derivation. Changes with any byte of the file. |
| **Proof id** | `M3e.E_T1` | Historical label inside the file, kept as written. |

Rules:
1. A library id is bound to one statement fingerprint, permanently. A different statement needs a new id.
2. A re-proof of the same statement (new file or new bytes, same fingerprint) may update the entry's proof pin, through a reviewed change that records the previous pin under `history`.
3. Two library ids may never share a fingerprint. Duplicates are refused, so one statement has one name.
4. Citations inside proof files stay path + hash, exactly as now. Library ids are an index over them, so no existing proof file or hash changes.

## 3. Proposed `lib/` layout

Existing proofs stay where they are. Moving them would change the bytes and hashes of every file that cites them. `lib/` holds index entries only:

```
lib/
  README.md                         what the library is; how entries are added
  arith/        closed equations      e.g. arith.add.29_1_30 → tests/accept/R_math29_pairing.rlt
  dvd/          divisibility          dvd.2_8 (CA01), dvd.not.2_7 (CA02), …, dvd.free235.7 (CA05)
  dvd/mod30/    M3e certificates      dvd.2_0 … (22 PD_*), dvd.not.2_29 … (21 ND_*), F_* and EA_07
  order/        finite enumeration    order.below30.enumeration (DA01)
  mod30/        classification        mod30.forward (E_FWD), mod30.backward (E_BWD), mod30.classification (E_T1)
```

One YAML file per entry, `lib/<area>/<id>.yaml`:

```yaml
format: rlt-lib/1
id: mod30.classification
title: "Below 30, free235(r) holds exactly for 1, 7, 11, 13, 17, 19, 23, 29"
statement_sha256: <fingerprint, §2>
proof: {path: tests/m3e/accept/E_T1.rlt, sha256: e11712c64baf6fcb40bc172b5d7e12607809c85b9f1cd087064a0c17ca651750, proof_id: M3e.E_T1}
axiom_set: ROBINSON-Q
convention: NAT-DEF
checked_goal: "∀r. ((∃z. S(z) + r = 30) → …)"      # in full, as `rlt check` prints it
assumptions: []                                     # [] = Γ = ∅; conditional entries are listed but not citable
uses: [mod30.forward, mod30.backward]               # library ids of the citations, in the file's order
statement_records: [statements/MOD30-CLASSIFICATION.yaml]
history: []                                         # earlier proof pins of the same statement
```

Initial scope: the 68 accepted proofs of M3c (9), M3d (5) and M3e (54), plus the two `statements/` proofs outside them (`R_math29_pairing`, and `M1_conditional`, listed as not citable): 70 entries, using the ids above. A generator (`scripts/gen-lib.py`, untrusted) proposes entries from `rlt check` output. Every entry is checked against the proofs by `rlt deps --verify-lib` (§9).

## 4. `rlt deps`

```
rlt deps ID|FILE [--format text|json] [--depth N]
rlt deps --verify-lib
```

`ID` is a library id; `FILE` is any proof path, so unlisted proofs can be traced too.

**Example (intended output, abbreviated):**

```
$ rlt deps mod30.classification
RLT deps        mod30.classification
proof           tests/m3e/accept/E_T1.rlt (M3e.E_T1) sha256 e11712c6…  matches lib entry
statement       ∀r. r < 30 → (free235(r) → …) ∧ (… → free235(r))      fingerprint 5c1d…  matches
verdict         ACCEPT — unconditional theorem relative to registered axiom set ROBINSON-Q

dependency graph (checked now; every edge read from the proof files)
mod30.classification                     E_T1     Γ = ∅  rules: and-intro, forall-elim, …
├─ FWD  mod30.forward                    E_FWD    Γ = ∅  cites 23
│  ├─ D    order.below30.enumeration     DA01     Γ = ∅  cites 0   axioms: add_succ, add_zero, q1, q2, q3
│  ├─ P0   dvd.2_0                       PD_00    Γ = ∅  cites 0   axioms: mul_zero
│  └─ …    (21 more)
└─ BWD  mod30.backward                   E_BWD    Γ = ∅  cites 24
   ├─ T2   dvd.not.2_7                   CA02     Γ = ∅  cites 0
   └─ …    (23 more)

theorems        50 distinct (50 listed in lib/, 0 unlisted)
axioms          ROBINSON-Q: add_succ, add_zero, mul_succ, mul_zero, q1_succ_inj, q2_zero_ne_succ, q3_zero_or_succ (union over the graph; equals E_T1's own)
definitions     dvd, free235, lt (spec/definitions.rlt sha256 122e37fc…), expanded before the kernel in every node
assumptions     none propagate: every cited theorem has Γ = ∅; the root's Γ = ∅
budget          cited-file checks 49 of 64 (77%) · citing-file chain 2 of 8
statement records  statements/MOD30-CLASSIFICATION.yaml (rlt explain: record valid · current)
exit            0
```

**JSON** (`rlt-deps/1`): one object per node (library id or `null`, path, SHA-256, proof id, checked goal, fingerprint, Γ, axioms used, rules used, definitions used, children by label), plus the summary fields. It reuses the RLT-004b writer and one internal report structure.

**Exit codes:** 0 = graph checked and every library pin current; 1 = some node rejected; 2 = invalid input or a limit or citation failure (as `rlt check` reports it); 3 = graph checked but a library entry is stale (hash, fingerprint or `uses` list differs from the files).

**`--verify-lib`:** checks every entry. It runs in `verify.sh`.

## 5. How assumptions, axioms and definitions propagate

| Item | Rule (current checker, unchanged) | What `deps` reports |
|---|---|---|
| **Assumptions (Γ)** | A cited theorem must have Γ = ∅, so no assumption crosses a citation. | Γ for every node and for the root. Conditional library entries are shown with `citable: no`. |
| **Axioms** | The citing theorem's recorded axioms are the union of its own and its cited theorems' (kernel `union`). Every cited axiom must equal, by content, an axiom of the citing proof's registered set. Cited files must use the same axiom-set id. | Per-node axioms used; the union; a check that the union equals the root's `axioms used` line. |
| **Definitions** | Expanded before the kernel in each file, from the binary's pinned definitions file. A cited theorem is compared in expanded form, so citing and cited agree on every definition by construction. | Definitions used per node and in total, with the pinned hash. |
| **Convention** | Cited files must use the citing file's convention. | Convention per node; a mixed graph is a citation failure (exit 2). |

## 6. Resource limits

- **Two budgets shown on every run:**
  - cited-file checks, used of 64;
  - the longest chain of citing files, used of 8.
- **Shared dependencies counted twice:** when a theorem is cited twice within one graph, `deps` shows the number of distinct theorems next to the number of checks, so the difference is visible.
- **Near the limit:** when either budget passes 75%, the summary adds a line naming which budget it is.
- **Over the limit:** `rlt check` refuses the file. `deps` prints that same message and exits 2. It also shows the graph as far as it was checked, with the node where the limit was reached marked.
- **Limits fixed:** `deps` never raises them, and it never caches results across citations. Caching would change what `rlt check` verifies, so it belongs to a separate review.

## 7. Connection to `rlt explain`

- A statement record keeps pinning its proof by path and SHA-256, unchanged.
- A library entry lists the statement records about its theorem (`statement_records`). `--verify-lib` checks that each named record's `kernel.proof` and `sha256` equal the entry's proof pin.
- `deps` prints each linked record's `rlt explain` state. These are the record states and exit meanings of RLT-004b, not new statuses.
- **Proposed, decision C5:** `rlt explain` prints one additional line naming the library id whose proof pin matches the record's. JSON gains one field, `library_id`. Everything else in its output stays byte-identical.

## 8. Negative tests (proposed fixtures, `tests/deps/`)

| Group | Case | Expected |
|---|---|---|
| **Stale hashes** | entry's proof SHA-256 differs from the file | exit 3, `stale proof sha256` |
| | entry's fingerprint differs (checked goal edited in the entry) | exit 3, `statement fingerprint differs` |
| | entry's `uses` differs from the file's citations (order or membership) | exit 3, `uses differs` |
| | linked statement record pins another hash | exit 3, `statement record pin differs` |
| **Identity** | two entries with the same id | invalid (2) |
| | two entries with the same fingerprint | invalid (2): one statement, one name |
| | entry whose proof proves a different statement from its title and fingerprint | exit 3, names both fingerprints |
| | `history` pin with a different fingerprint | invalid (2): a re-proof must prove the same statement |
| | id ≠ file name; malformed id; unknown format | invalid (2) |
| **Invalid citations** | cited path outside the repository, through a symlink, or missing | exit 2, `rlt check`'s message |
| | cited file rejected, conditional, other axiom set or convention | exit 2, `rlt check`'s message, node marked |
| | `uses` names a conditional entry | invalid (2): conditional entries are not citable |
| | `uses` names an unknown library id | invalid (2) |
| **Cycles** | self-citation; 2-cycle; 3-cycle | exit 2, cycle path printed in order |
| **Limits** | chain of 9 citing files | exit 2, partial graph, depth 9 of 8 |
| | 65 cited-file checks (one file cited 65 times through shared subtrees) | exit 2, 65 of 64, distinct vs checks shown |
| | 49 of 64 (E-T1) | exit 0, 75% warning line |
| **Agreement** | for every proof in `tests/**` | the `deps` verdict and root fields equal `rlt check`; the edges equal the files' `cites` blocks |
| **Output** | hostile ids, titles and paths | text escaped (RLT-004b's escaper); JSON parses; deterministic output |

## 9. Trusted boundary

| Component | Trust |
|---|---|
| Kernel and citation checking | Unchanged. The only source of verdicts and of every edge's validity. |
| **Citation trace** (decision C2) | `check_inner` records, while it checks, each citation it resolves: label, path, SHA-256, proof id, theorem. `rlt check` output is unchanged; `compare-reports.sh` guards that. The trace is a by-product of the check `rlt check` performs, never a second traversal. |
| `rlt deps` | Outside the trusted base. It formats the trace and compares library entries with it. A defect could misstate a graph or a verdict to a reader or JSON consumer; it cannot change what the kernel accepts. Every node's path, hash and checked goal are printed for reproduction with `rlt check`. |
| `lib/` entries | Untrusted metadata. An entry only *names* a theorem. Every field is compared with the checked files, and a mismatch is reported, never believed. |
| Statement fingerprint | A SHA-256 of the kernel's printed form. It identifies *what is proved*, and says nothing about faithful translation or semantic correspondence; those remain `rlt explain`'s separate outcomes. |
| Generator `scripts/gen-lib.py` | Untrusted; its output is reviewed and verified like any entry. |

## 10. Implementation scope (after approval)

| Part | Estimate |
|---|---|
| Citation trace in `check_inner` (additive output field) | ~60 lines |
| `lib/` loader and schema (`syntax/src/records.rs`) | ~200 lines |
| `cli/src/deps.rs`: graph, budgets, library comparison, text and JSON | ~500 lines |
| `lib/` entries for the 70 existing proofs (generated, reviewed) | 70 files |
| `tests/deps/` fixtures (~30) and `cli/tests/deps.rs` | ~350 lines |
| `verify.sh` section; manual chapter 8 example; this report's successor | — |
| Correct the stale `check_preread_outcome` doc comment in `cli/src/explain.rs` | 1 line |

**Outside scope:**
- caching or raising limits;
- moving or renaming proof files;
- `rlt deps --reverse`, a reverse index of who uses a theorem (noted for later);
- induction, binary numerals, new axioms;
- release and deployment.

## 11. Decisions for review

| ID | Decision | Options | Recommended |
|---|---|---|---|
| C1 | Identity | (a) library id bound to a statement fingerprint, proof pin updatable by review; (b) id bound to the proof hash | (a) |
| C2 | Graph source | (a) citation trace recorded by `check_inner` during the one check; (b) a separate walker re-checking each node | (a): one traversal, the same checks |
| C3 | Library location | (a) index entries in `lib/` pointing at existing files; (b) move proofs into `lib/` | (a): no existing hash changes |
| C4 | Initial contents | (a) the 70 existing proofs (M3c–M3e plus the two other `statements/` proofs); (b) M3e only | (a) |
| C5 | `rlt explain` link | (a) add a `library` line and a `library_id` JSON field; (b) `explain` unchanged, link shown only by `deps` | (a), additive |
| C6 | Exit 3 for stale entries | (a) as in `explain`; (b) stale entries reported, exit 0 | (a) |
| C7 | Conditional theorems | (a) listed with `citable: no`; (b) excluded from `lib/` | (a) |
| C8 | Shared dependencies | (a) report distinct theorems and checks; no caching; (b) cache within one run | (a); caching is a separate review |

## 12. After RLT-004c

With `check`, `explain` and `deps` in place, the next milestone can be a practical demonstration: one small, exact statement from a published paper. Its source would be quoted and pinned (`explain`), its proof built from library theorems (`deps`) and checked (`check`). The choice of paper and statement belongs to its own review: it must fit Robinson's Q and the measured limits (docs/RLT-004a-BASELINE.md), or say plainly which deferred capability it needs.

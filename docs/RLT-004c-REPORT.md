# RLT-004c — Implementation report: theorem library and `rlt deps`

**Branch** `docs/rlt-004c` · **PR** #12 (draft) · **Base** main at 45c94b5 · **Date** 2026-11-11

**Scope, as approved.**
- Decisions C1–C8 of docs/RLT-004c-REVIEW.md, with the refinement to C1: the fingerprint captures the checked formula, assumptions, axiom-set identity and content, the definitions and the conventions with their pinned interpretation, and a proof is replaced under an id only when the whole fingerprint is identical.
- No kernel or inference-rule change; every `rlt check` verdict and report unchanged.

## 1. What was built

| Part | Files |
|---|---|
| Citation trace (C2) | `cli/src/lib.rs`: `CheckOutcome.citations` / `CitationTrace`, recorded by `check_inner` as it resolves each citation (label, path, recorded SHA-256, the cited check's outcome, any refusal). No new entry point into the checker; `rlt check` output byte-identical. |
| Statement fingerprint (C1, refined) | `cli/src/deps.rs`: `canonical`, `statement_parts`, `fingerprint_text` |
| Library index (C3, C4, C7) | `lib/<area>/<id>.yaml`: 68 entries covering 70 proofs (CA09 and PD_08 prove the same statements as CA02 and CA01 and are listed as alternatives); `lib/identities.tsv`; `lib/README.md`; loader `load_lib_entry` in `syntax/src/records.rs`; proposer `scripts/gen-lib.py` (untrusted) |
| `rlt deps ID\|FILE [--format text\|json]`, `rlt deps --verify-lib` | `cli/src/deps.rs`, dispatch in `cli/src/main.rs` |
| `rlt explain` link (C5) | one `library` line in the kernel block; JSON `library_id`, `library_index`; exit codes unchanged |
| Identity gate | `scripts/check-identities.sh`, run by `verify.sh` |
| Tests | `cli/tests/deps.rs` (9 tests, about 45 scenarios on scratch repositories and scratch git histories) |
| Docs | manual chapter 8 (`rlt deps` example, machine-checked), `lib/README.md`, this report; the stale comment in `cli/src/explain.rs` corrected |

## 2. Statement fingerprint

SHA-256 of this canonical text (printed by `rlt deps` in full):

```
rlt-statement-fingerprint/1
convention <id> <pinned interpretation digest>
axiom-set <id> <file> <pinned sha256>
definitions <file> <sha256> used <names> | definitions none
goal <canonical form of the checked formula>
assumption <canonical form>   (one per distinct assumption, sorted) | assumptions none
```

- **Canonical form:**
  - bound variables are de Bruijn indices (`#0`) and binder names are dropped;
  - every symbol is written prefix with its arity, e.g. `(*/2 #0 (0/0))`;
  - parameters and axiom variables carry distinct tags.

  Alpha-variants and renamed assumptions share one fingerprint. Different statements never share one: the audit found no collision.
- **Identity rules** (all enforced by the library loader, exit 2 when broken):
  - every id is registered once in `lib/identities.tsv` with its fingerprint and first proof SHA-256;
  - an entry must carry its registered fingerprint;
  - no two ids share a fingerprint, and no pin appears under two ids;
  - every history pin carries the identical fingerprint;
  - once the first proof is replaced, it must appear in `history`, with a date and reviewer;
  - a registered id must keep its entry.
- **Append-only registry.** `scripts/check-identities.sh` checks every version of `lib/identities.tsv` on main's first-parent history: each of its lines must survive unchanged. A shallow clone is deepened first, and a missing file fails. On main, a committed rebinding fails CI. On a branch, only main's versions bind, so drafts on unmerged branches may change.

## 3. Reporting contract

| Exit | `rlt deps` |
|---|---|
| 0 | every node checked and accepted; library index valid; the entry, if any, current |
| 1 | the checker rejects a proof in the graph or an alternative proof of the entry |
| 2 | invalid input or library index, or a citation or limit failure, as `rlt check` reports it |
| 3 | the graph checks, but the entry differs from the checked files: any of proof SHA-256, fingerprint, proof id, axiom set, convention, checked goal, assumptions, `uses`, alternatives, or a linked statement record (pin or `rlt explain` exit) |

- **Output for each node:**
  - label, library id, proof id, path and SHA-256;
  - verdict, Γ and citation count;
  - proved formula, axioms used, definitions used and fingerprint.
  - A refused citation is shown as refused and adds nothing to the unions.
- **Summary lines:**
  - distinct theorems and cited-file checks;
  - the union of axioms and of definitions;
  - both budgets, `n of 64` and `n of 8`, with a warning above 75% and "limit exceeded" past it;
  - linked statement records, with their `rlt explain` state;
  - notes, e.g. a history pin whose bytes are gone (reviewed metadata, not re-checked).
- **JSON (`rlt-deps/1`)** carries the same fields and the full fingerprint text.

**Trusted boundary.**
- The graph is the trace of the one check `rlt check` performs on the root's bytes, so every edge is a citation the checker resolved at the bytes it hashed.
- `rlt deps` and the library are outside the trusted base. A defect in this reporting layer could misstate a graph, a verdict or an entry's state to a reader or a JSON consumer; it cannot change what the kernel accepts.
- The only checker change is the added trace field (output only). `compare-reports.sh` shows every report unchanged.

## 4. Verification

| Check | Result |
|---|---|
| `cargo test --release` | 257 passed, 0 failed (248 earlier + 9 new) |
| `scripts/verify.sh /home/claude/src` | `RESULT   all checks hold`; library section: 68 of 68 entries current and accepted; `deps mod30.classification` JSON exit 0, 50 theorems, 49 checks, chain 2; identity registry kept |
| `scripts/check-manual.py` | 25 of 25 examples match |
| `scripts/compare-reports.sh` against main 45c94b5 | 443 proof reports and 10 record commands byte-identical, `differences: 0`; the usage text gains the two `deps` lines |
| Protected paths vs 45c94b5 | kernel/, spec/, conventions/, readingpoints/, residues/, sources/, lean/, points/, statements/ and every tests/ directory unchanged |
| CI | `check` and `gdt-comparison` succeed on ac331c5 and db0c8b2; final head: see PR |

**Tested cases** (`cli/tests/deps.rs`):
- **Library and graph:**
  - the library is valid and every entry current;
  - the E-T1 graph has 50 theorems, 49 checks and a chain of 2, with the 75% warning and its statement-record link;
  - the root and edges agree with `rlt check` for every proof under tests/ (over 400 files).
- **Fingerprint:** each of 10 meaning-relevant changes changes the text; alpha-variants and renamed assumptions keep it.
- **Stale (exit 3):**
  - changed proof bytes (a changed proof cannot keep its verified identity);
  - edited goal, proof id or `uses`;
  - a stale statement-record pin, or a stale record whose `rlt explain` exits 3.
- **Invalid (exit 2):**
  - forged or unregistered fingerprint;
  - a different statement swapped under an existing id (a semantically changed specification cannot keep the id);
  - unregistered id, retired id, wrong area;
  - replacement without history, history proving another statement;
  - `uses` naming a conditional or unknown id, a citable conditional;
  - duplicate fingerprint, unknown target, an invalid library in FILE mode.
- **Accepted:** a reviewed replacement with history. Its dependent entry then reports the now-unlisted citation as stale.
- **Citations:**
  - bad hash, outside path, self-citation and 2-cycle;
  - chains of 8 (accepted, 8 of 8) and 9 (refused, 9 of 8);
  - 64 citations (accepted) and 65 (refused, 65 of 64);
  - refused conditional and axiom-set citations;
  - a rejected alternative (exit 1).
- **Registry gate on scratch git histories:**
  - appends are allowed;
  - working-tree and committed rebindings are refused, as is a deleted file;
  - a merged branch's drafts are not binding.

## 5. Independent adversarial audit

The first audit ran at ac331c5 and found no soundness defect: the kernel, `rlt check` reports and the binding of citations to bytes were all unaffected. Findings and disposition:

| Finding | Severity | Disposition |
|---|---|---|
| Identity gate compared with `origin/main`, which on a push to main is the commit itself: a rebinding merged to main would pass | major | Fixed: every version on main's history is checked, the clone is deepened, and a missing file fails |
| A refused citation (conditional, other axiom set) shown as an accepted edge; its axioms in the union | minor | Fixed: recorded on the trace, shown as "citation refused", excluded from unions |
| Exceeded limits shown as "64 of 64" and "8 of 8" | minor | Fixed: "65 of 64", "9 of 8 — limit exceeded" |
| A linked statement record stale under `rlt explain`, yet the entry was current | minor | Fixed: stale item; exit 3 |
| Invalid library ignored in FILE mode | minor | Fixed: exit 2 in every mode |
| Alpha-variants and renamed assumptions gave different fingerprints | minor | Fixed: de Bruijn canonical form; assumptions as a sorted set of formulas |
| A failing alternative reported as stale | note | Fixed: its checker exit (1 or 2) |
| An id ending `.rlt` reads as a file target | observation | Fixed: refused by `lib_id` |
| History pins never checked | observation | Fixed: re-checked when their bytes exist; otherwise a note |

**Re-audit at db0c8b2.** All seven defects were confirmed fixed, and the canonical form withstood attempts at collision. One new major finding: the gate followed every parent, so a normal merge commit would bring this branch's first draft registry (ac331c5) into main's history and fail CI on main. Fixed in the final commit:
- the gate follows `--first-parent`;
- a scratch-history test covers a merged draft;
- the full repro was re-run against a real merge of this branch into main: exit 0.

Smaller fixes in the same commit:
- the 65-of-64 warning now counts attempts;
- the exit-1 text names alternatives;
- repeated assumptions count once;
- a history pin whose bytes are gone is reported as a note.

## 6. Limitations

- **Free variables and parameters keep their names in the canonical form.** A theorem's goal has neither, so this only over-distinguishes.
- **History pins whose bytes are gone** are reviewed metadata. They are reported as notes, and nothing re-checks them.
- **The append-only rule binds main's history.** Branch protection on main (no force-push) is what keeps that history fixed.
- **Each linked statement record's proof is checked again** by `rlt explain`, inside `rlt deps` and `--verify-lib` (about 4 s for the whole library). The graph itself comes from one check. Caching stays deferred (C8).
- **Departures from the review document:**
  - 68 entries for 70 proofs, because two proofs are alternatives;
  - the negative cases live in scratch repositories built by `cli/tests/deps.rs` rather than in a `tests/deps/` directory;
  - `--depth N` was not built.

## 7. Merge recommendation

Recommend approval for merge after CI is green on the final head. The change is additive outside the trusted kernel, every `rlt check` report is byte-identical, and every audit finding is fixed with a regression test. The identity rules hold against changed proofs, forged identities and rebindings, including after a merge commit. PR #12 stays in draft until reviewed; no release or deployment.

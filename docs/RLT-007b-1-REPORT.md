# RLT-007b stage 1 — Distinctness and diameter 212: implementation report

**Status:** implemented on branch `rlt-007b-1`, in a draft PR, 2026-11-12. Not merged; merging needs separate approval. An independent adversarial review is reported in §7.

**Basis.** main at 1561668, the RLT-007a merge.

**Unchanged:**
- the kernel, its 20 inference rules plus `cite`, and ROBINSON-Q (seven axioms);
- the resource limits: 1,000,000 nodes per file, 64 cited-file checks, chain 8, and a numeral total of 200,000 per formula;
- every existing proof, certificate and record.

## 1. What is established, and how

| Statement | Library id or file | Status |
|---|---|---|
| For each listed element hᵢ (i = 0 … 43): ⋀_{j>i} ¬(hᵢ = hⱼ) | 44 per-element theorems `RLT007S1.D_HHH` in `tests/rlt007s1/gen/` (generated, manifest-pinned) | **kernel accepted** |
| Distinctness groups: the per-element statements of elements 0–10, 11–21, 22–32 and 33–43, each as one conjunction | `admissible.h45.distinct.g0` … `g3` (`tests/rlt007s1/accept/DG_K.rlt`; each cites 11, using 11 of 64 cited-file checks, chain 1 of 8) | **kernel accepted** |
| ⋀_{h ∈ H₄₅} ((∃z. z + 0 = h) ∧ (∃z. z + h = 212)) | `admissible.h45.bounds` (`tests/rlt007s1/accept/BOUNDS.rlt`) | **kernel accepted** |
| The tuple has exactly 45 entries, is strictly increasing, starts at 0 and ends at 212; the lemma text names a "45-tuple" of "diameter 212" (whole numbers) | `scripts/gen-rlt007s1.py`, which refuses to generate when any check fails, and verify.sh. Both read the transcription's `numbers` and the lemma's `extracted` text, which `check_excerpts.py` compares with the pinned PDF page, and the reviewed `text` field | **source transcription checked** |
| These statements and listing facts together are obligation (a), "a 45-tuple of diameter 212" | `docs/RLT-007b-1-CORRESPONDENCE.md`, pinned by SHA-256 in the 5 statement records | **semantic correspondence assessed** (`partial`: obligation (a) only) |

Together, all 990 pairwise disequalities are kernel accepted. "≤" is read inline through an addition witness, ∃z. z + x = y, and no definition is added.

## 2. Proof methods (as approved)

- **Distinctness.** The theorem for element a builds K₀, …, K_a, where K_j :≡ ∀u. ¬(j = S^{j+1}(u)):
  - K₀ comes from Q2;
  - K_j comes from K_{j−1}, using Q1, hyp, imp-elim, imp-intro and forall-intro.

  Each later element b is then one forall-elim of K_a at the numeral b − a − 1. Numerals are unary, so S^{a+1}(b − a − 1) is literally b. A left-nested and-intro chain then forms the conjunction.
- **Bounds.**
  1. One chain with a parameter w proves w + j = S^j(w) for j = 0 … 212, using add_zero, add_succ, cong and trans.
  2. For each h, generalize at step h and instantiate at 212 − h; exists-intro then closes ∃z. z + h = 212.
  3. The lower bound uses add_zero and exists-intro, with witness h.

## 3. Reproducibility

- **The generator.** `scripts/gen-rlt007s1.py` is deterministic. It reads the reviewed transcription, plus the correspondence review for its SHA-256 pin. It has four modes:
  - **default:** writes every output;
  - **`--materialize-only`:** writes the 44 generated theorems;
  - **`--check`:** regenerates all 64 outputs in memory and compares them with the files on disk, writing nothing. It also requires exact file sets in `gen/`, `accept/`, `reject/` and `tests/rlt007s1/`, among stage-1 statement records and among `lean/RLT007S1*`;
  - **`--self-test`:** runs the transcription checks against 11 altered transcriptions:
    - a duplicated element;
    - an out-of-order element;
    - 213 in place of 212;
    - 44 entries;
    - 46 entries;
    - 46 strictly increasing entries from 0 to 212, which isolates the length check;
    - 1 in place of 0;
    - a negative entry;
    - lemma texts naming a 44-tuple, a 145-tuple, or diameter 2120.

    All 11 are refused.
- **The 44 generated theorems** (13.4 MB) are gitignored, pinned by `tests/rlt007s1/manifest.sha256` and by the group summaries' citations, as RLT-006's are. Everything else is committed:
  - the summaries and the bounds file (0.5 MB);
  - the rejected cases;
  - the records;
  - the Lean file;
  - the manifest.
- **Library.** The 5 library entries were proposed by `scripts/gen-lib.py` (extended for this stage) and appended to `lib/identities.tsv`. The append-only check still holds. `rlt deps --verify-lib` reports 106 of 106 entries current and accepted.

  **Deviation from the proposal.** The proposal listed 49 library ids. As in RLT-006, the 44 generated per-element theorems are cited as `(unlisted …)` uses, not separate library ids. Only the 5 committed theorems are library entries. Every per-element theorem is still kernel-checked directly, and again through each citation.

## 4. Negative tests

**Rejected cases** in `tests/rlt007s1/reject/`, listed in `expected.tsv`. Each is refused for its stated reason:

| Case | Exit | Reason |
|---|---|---|
| SX01: claims ¬(2 = 2) | 1 | forall-elim yields ¬(2 = 12) |
| SX02: ¬(2 = 12) from the wrong instance (8) | 1 | forall-elim yields ¬(2 = 11) |
| SX03: K-chain missing step KA7 | 1 | premise `KA7` is not an earlier step |
| SX04: bounds witness 187 for 24 | 1 | forall-elim yields 187 + 24 = 211 |
| SX05: group 0 cites element 56's theorem in place of 54's | 1 | the cited theorem proves a different statement |
| SX06: stale SHA-256 in a citation | 2 | the hash mismatch is named |
| SX07: cited theorem absent | 2 | `entity not found` |

**Tamper tests** in verify.sh and `cli/tests/rlt007s1.rs`. A missing per-element theorem and a modified one are each refused by `rlt check` (MALFORMED) and by the manifest.

## 5. Lean comparison

`lean/RLT007S1.lean` is generated and uses core Lean only. It contains 50 theorems. Lean reports that none of them depends on any axiom. The axiom gate requires all 50 to be printed, and no axiom outside the standard three. The theorems are:
- the 44 per-element statements, literally;
- the four group statements, literally, built from those;
- `bounds`, literally, with explicit witnesses;
- `obligation_a`, which decides that the list has length 45, no duplicates, first entry 0, last entry 212, and every entry in [0, 212].

The statements are large, so the file raises Lean's heartbeat and recursion limits. The proofs stay `decide`, explicit witnesses and anonymous constructors.

The statement records cite `dg_0` … `dg_3`, `bounds` and `obligation_a`. The pinned count of distinct Lean references is now 30, previously 24.

## 6. Verification

**Full `scripts/verify.sh` on the branch:** `RESULT all checks hold`, 1,145 s wall and 1,717 s CPU. The 2-processor machine was shared with another run for part of the time.

**New stage output:**
- source transcription checked;
- generator self-test 8 of 8;
- `--check` 64 outputs, 0 differ;
- kernel accepted 45 of 45 directly and 4 of 4 summaries (11 of 64 checks each);
- rejects 7 of 7;
- both tamper tests refused, then restored;
- library 106 of 106;
- Lean 50 statements with standard axioms only.

**Coverage.** Compared line by line with the merged main's run, every earlier report line is still present. The only changed lines are the commit line, the library count (101 → 106) and the identity count.

**Other results:**
- `cargo test`: the new `cli/tests/rlt007s1.rs` has 6 tests. Existing tests are adjusted only to skip the generated directory when copying scratch trees, and for the library counts (106 entries, 108 proofs).
- Manual: 33 of 33 match. One example's library count in `docs/MANUAL.md` changed from 101 to 106 entries.

**Cost to the complete run:** about 40 s of generation and checking with RLT-007a's scheduling, plus about 50 s of Lean, run in parallel with the other Lean files. Disk use grows by 13.4 MB of generated files and 1.2 MB committed.

## 7. Independent adversarial review

**Scope and result.** An independent auditor, who had not seen the work produced, audited `3cfcf65` against main 1561668. It extracted page 43 itself and parsed the kernel's own output. It found:
- the mathematical statements correct and complete for obligation (a);
- no substantive overclaim;
- no unresolved substantive issue.

It also built adversarial cases of its own:
- a duplicated citation;
- a pair from another group;
- a bound of 213;
- a group with an element dropped;
- a record and a library entry each pointed at the wrong proof.

Each was refused by the kernel or by `verify-lib`, `explain` or `--check`.

**Should-fix items and notes, all addressed in the next commit:**

| Finding | Fix |
|---|---|
| The self-test never isolated the 45-entry length check | New case: 46 strictly increasing entries from 0 to 212 |
| The lemma-text check read only the reviewed `text`, which is not compared with the PDF | Both `extracted` (PDF-compared) and `text` are checked, with whole-number matching ("145-tuple" and "diameter 2120" are refused) |
| `--check` ignored extra files in `gen/`, `accept/` and `lean/`, and other stage-1 records | Exact file sets throughout. verify.sh requires exactly 5 files in `accept/`. The deps-agrees-with-check comparison now runs on the 12 committed files |
| SX05's expected reason was short | Pinned to "the cited theorem `E10` proves …" |
| Report wording: the lemma check's source, the axiom gate, the generator's inputs | Reworded (§1, §3, §5) |
| The correspondence did not say how "diameter" is read | It now says the reading is adopted, and cites the definition on PDF page 3 |
| `rlt006.rs` regenerated stage-1 files without a hash check | The hash check is added |

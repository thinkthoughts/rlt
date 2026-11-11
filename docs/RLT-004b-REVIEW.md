# RLT-004b — Statement records and `rlt explain`: architecture and test review

**Status.** Proposal for review, 2026-11-11, against main at 58838c9. Architecture and tests only: no code is written until this review is approved. No kernel change, axiom, induction, binary numerals, theorem library, release or deployment.

**Objective.** One command connects a written mathematical statement to its exact specification, its proof and dependencies, and its correspondence assessments, reporting each separately. The first record is the Modulo 30 classification E-T1, as specified in docs/STATEMENT-RECORDS.md (RLT-004a).

---

## 1. Principles

1. **Kernel results are reused, never restated.** `rlt explain` obtains the kernel block by calling the same function as `rlt check` (`check_file_outcome`) on the recorded proof file, and prints the theorem the kernel returned.
2. **A record pins; it never proves.** Every hash, goal and assumption in a record is compared with the current files and the current kernel result. A mismatch makes the record *stale*; it never changes a verdict.
3. **Semantic correspondence is an attributed assessment.** It is read from the record, printed with its assessor, date and basis, and pinned to the exact checked statement it assessed. No kernel result sets or upgrades it.
4. **Physical correspondence appears only when the record says it is relevant.** A purely mathematical record states `physical: not-applicable` with a reason; nothing is required of it.
5. **Additive.** A new command, a new record directory and new tests. Every existing report stays byte-identical.

## 2. Command syntax

```
rlt explain RECORD [--format text|json] [--root DIR]
```

- `RECORD`: a repository-relative path to a statement record, `statements/<ID>.yaml` (same path rules as citations: plain components, no symlinks).
- `--format text` (default): the terminal report of §3.
- `--format json`: the same content as one JSON object (§4), for reuse by scripts, the manual checker and future tools.

**Exit status** (the record's state, never its correspondence assessments):

| Exit | Meaning |
|---|---|
| 0 | record valid and current; proof ACCEPT |
| 1 | record valid and current; proof REJECT |
| 2 | record invalid (cannot be read, fails the schema, or references something missing), or proof MALFORMED |
| 3 | record valid, proof ACCEPT, but stale: a pinned hash, checked statement or assumption differs from the current files or kernel result |

Semantic and physical statuses never affect the exit status. (This refines RLT-004a's sketch, which gave the exit status for kernel acceptance only; exits 2 and 3 now separate invalid from stale records, as requested.)

## 3. Example terminal output (E-T1)

```
$ rlt explain statements/MOD30-CLASSIFICATION.yaml
RLT explain     statements/MOD30-CLASSIFICATION.yaml
statement       MOD30-CLASSIFICATION
subject         Modulo 30 — residues below 30 coprime to 2, 3 and 5
  identifiers   math29 (historical, readingpoints/math29.yaml)
record          valid · current

1 KERNEL ACCEPTANCE                                       ACCEPT
  proof         tests/m3e/accept/E_T1.rlt (M3e.E_T1)
  sha256        e11712c64baf6fcb40bc172b5d7e12607809c85b9f1cd087064a0c17ca651750  matches record
  written goal  "∀r. r < 30 → (free235(r) → (r = 1 ∨ … ∨ r = 29)) ∧ (… → free235(r))"   [printed in full]
  convention    NAT-DEF (pinned in this binary)
  definitions   dvd, free235, lt (spec/definitions.rlt sha256 122e37fc…  matches record)
  checked goal  ∀r. ((∃z. S(z) + r = 30) → …)                                          [printed in full; equals record]
  assumptions   Γ = ∅  (equals record)
  axiom set     ROBINSON-Q (spec/robinson-q.rlt sha256 4ad55192…  matches record)
  axioms used   add_succ, add_zero, mul_succ, mul_zero, q1_succ_inj, q2_zero_ne_succ, q3_zero_or_succ
  rules used    and-intro, forall-elim, forall-intro, hyp, imp-elim, imp-intro
  cites         FWD tests/m3e/accept/E_FWD.rlt  sha256 4758d9dd…  (checked now)
                BWD tests/m3e/accept/E_BWD.rlt  sha256 10e27c9f…  (checked now)
  scope         the checked formula follows from ROBINSON-Q; this block establishes nothing further

2 SOURCE FIDELITY                                         verified (registry)
  source        NEWS-MATH29 · thinkthoughts/news MATH_29.md @ 1ffd5ea9886386dc0b16c13499ecfe2ed59fd9e9
  excerpt       "{1, 7, 11, 13, 17, 19, 23, 29}: 17111317192329"  whole unit of a registered excerpt
  excerpt       "For instance (nonfiction): (1+1) × 3 × 5 = 30 > 29, where every prime > 5 …"  whole unit of a registered excerpt
  upstream      verbatim at the commit: checked by scripts/check_excerpts.py (run by verify.sh)
  scope         the quotation is exact; quotation fidelity says nothing about translation

3 SEMANTIC CORRESPONDENCE                                 partial
  assessed by   RLT-003 M3e review · 2026-11-09
  basis         docs/RLT-003-M3e.md sha256 67d01af5…  matches record
  assessed goal equals the checked goal above
  matches       the eight representatives are exactly the r < 30 with free235(r)
  matches       over ℕ, free235(r) ⇔ gcd(r, 30) = 1 for r < 30
                  Lean: lean/M3eReadings.lean · RLTM3eReadings.free235_iff_coprime (present; run by verify.sh)
  differs       the source speaks of every prime > 5 and of residue classes; the theorem stops at r < 30
  scope         an attributed assessment; kernel acceptance never sets it

4 PHYSICAL CORRESPONDENCE                                 not applicable
  reason        a mathematical statement about the numbers below 30

open questions  3
  1 every n: n mod 30 in the eight classes ⇔ gcd(n, 30) = 1 (needs division with remainder for all n; induction review)
  2 primes > 5 (needs a definition of prime and the quantified statement)
  3 whether RESIDUE-30's registered reading should cite this theorem (backed-by review)
verdict         record valid · current · proof ACCEPT · source verified (registry) · semantic partial · physical not applicable
```

Real output prints every goal and hash in full; the `[printed in full]` markers above only shorten this page.

## 4. Structured output (`--format json`)

```json
{
  "format": "rlt-explain/1",
  "record": {"path": "statements/MOD30-CLASSIFICATION.yaml", "id": "MOD30-CLASSIFICATION",
             "state": "current", "problems": []},
  "exit": 0,
  "kernel": {"verdict": "ACCEPT", "proof": "tests/m3e/accept/E_T1.rlt", "proof_id": "M3e.E_T1",
             "sha256": "e117…", "written_goal": "…", "convention": "NAT-DEF",
             "definitions": {"file": "spec/definitions.rlt", "sha256": "122e…", "used": ["dvd", "free235", "lt"]},
             "checked_goal": "…", "assumptions": [], "axiom_set": {"id": "ROBINSON-Q", "sha256": "4ad5…"},
             "axioms_used": ["add_succ", "…"], "rules_used": ["and-intro", "…"],
             "cites": [{"label": "FWD", "path": "…", "sha256": "…", "proved": "…"}]},
  "source": {"status": "verified-registry", "registry": "NEWS-MATH29", "commit": "1ffd…",
             "excerpts": [{"text": "…", "status": "whole-unit"}]},
  "semantic": {"status": "partial", "assessed_by": "RLT-003 M3e review", "date": "2026-11-09",
               "basis": {"path": "docs/RLT-003-M3e.md", "sha256": "67d0…", "current": true},
               "assessed_goal_current": true, "matches": ["…"], "differs": ["…"],
               "lean": [{"file": "lean/M3eReadings.lean", "theorem": "RLTM3eReadings.free235_iff_coprime", "present": true}]},
  "physical": {"status": "not-applicable", "reason": "…"},
  "open_questions": ["…"]
}
```

Strings are exactly the text of the terminal report; JSON is written by a small escaper in the CLI (no new dependency). Fields that do not apply are present with `null`, so consumers never guess.

## 5. Record schema (`statements/<ID>.yaml`)

```yaml
format: rlt-statement/1                      # required; unknown formats are invalid
id: MOD30-CLASSIFICATION                     # plain id; must equal the file stem
subject: "Modulo 30 — residues below 30 coprime to 2, 3 and 5"
identifiers:                                 # optional; each must be an existing reading point
  - {reading_point: math29, role: historical}
source:
  registry: NEWS-MATH29                      # must exist in sources/registry.yaml with status available
  commit: 1ffd5ea9886386dc0b16c13499ecfe2ed59fd9e9   # must equal the registry's commit
  excerpts:                                  # each a whole unit of a registered excerpt of that source
    - "{1, 7, 11, 13, 17, 19, 23, 29}: 17111317192329"
    - "For instance (nonfiction): (1+1) × 3 × 5 = 30 > 29, where every prime > 5 resists division by 2, 3, and 5 and corresponds to one of eight residue classes modulo 30:"
kernel:                                      # pins; each compared with the files and the kernel result
  proof: tests/m3e/accept/E_T1.rlt
  sha256: e11712c64baf6fcb40bc172b5d7e12607809c85b9f1cd087064a0c17ca651750
  convention: NAT-DEF
  definitions_sha256: 122e37fc627b136023a1a5ffac0344162ed77ae95312248a9f6a76c2b639ecda
  axiom_set: ROBINSON-Q
  axiom_set_sha256: 4ad55192222576e3901abc4ee7ea5a6760b64ff212e8c55796ab9847e423b00c
  checked_goal: "∀r. ((∃z. S(z) + r = 30) → …)"        # the kernel's display of the proved formula, in full
  assumptions: []                                       # name: formula pairs; [] means Γ = ∅
semantic:
  status: partial                            # unassessed | matches | partial | differs
  assessed_by: "RLT-003 M3e review"          # required unless unassessed
  date: 2026-11-09                           # required unless unassessed
  basis: {path: docs/RLT-003-M3e.md, sha256: 67d01af5df3d9d39880d5e41170dd4c8c888909281c022631938faec7a1966e4}
  assessed_goal: "∀r. ((∃z. S(z) + r = 30) → …)"       # the goal the assessor read; must equal checked_goal to be current
  matches: ["…"]                             # required for matches and partial
  differs: ["…"]                             # required for partial and differs
  lean: [{file: lean/M3eReadings.lean, theorem: RLTM3eReadings.free235_iff_coprime}]   # optional supporting statements
physical:
  applicability: not-applicable              # not-applicable | relevant
  reason: "a mathematical statement about the numbers below 30"
  # when relevant: correspondence: [math29.context]  — ids in a reading point's record; status read from there
open_questions: ["…", "…", "…"]
```

**Rules.**
- Unknown keys, duplicate keys and control characters are invalid (the existing record loader's rules).
- `semantic.status: unassessed` forbids `matches`, `differs`, `basis` and `assessed_goal`, so an empty assessment cannot look like a finding.
- `physical` is required, so a record always says whether physical correspondence applies. `relevant` requires at least one correspondence id; their statuses are read from the reading-point records, never from the statement record.
- The record never contains a verdict, an `accepted` flag or a semantic status derived from one.

## 6. Distinguished outcomes and failure cases

| State | Detected when | Exit | Report |
|---|---|---|---|
| **Invalid record** | unreadable or non-YAML; schema violation; id ≠ file stem; unknown format; path outside the repository or through a symlink; proof file missing; unknown convention, axiom set, registry id or reading point; semantic attribution missing; `relevant` physical with no ids or unknown ids | 2 | `record invalid — <reason>`; no further blocks |
| **Failed proof** | kernel REJECT | 1 | kernel block with the kernel's own REJECT message; other blocks still shown, marked "of a rejected proof" |
| **Malformed proof** | `rlt check` would exit 2 (including citation failures and size limits) | 2 | kernel block with the MALFORMED message |
| **Stale record** | proof, definitions or axiom-set hash differs; checked goal or assumptions differ from the kernel result; convention differs | 3 | `record stale` with each differing pin, old and new values |
| **Stale assessment** | `assessed_goal` differs from the current checked goal, or the basis hash differs | exit unaffected | semantic block status shows `partial (assessment stale: …)` |
| **Unassessed correspondence** | `semantic.status: unassessed` | exit unaffected | `3 SEMANTIC CORRESPONDENCE  unassessed` |
| **Source differs** | an excerpt is no whole unit of the source's registered excerpts, or the commit differs from the registry | 2 (invalid) | names the excerpt and the nearest registered unit |
| **Unpinned source** | registry status other than `available` (e.g. conversation) | exit unaffected | `2 SOURCE FIDELITY  unpinned (conversation record)` |
| **Lean reference absent** | `theorem` name not found in the named Lean file | 2 (invalid) | names the file and theorem |

## 7. Tests

**Accept fixtures** (`statements/` and `tests/explain/accept/`):
- X01 MOD30-CLASSIFICATION (E-T1): exit 0, the full report of §3.
- X02 a purely mathematical closed statement (2 ∣ 8, CA01): `semantic: unassessed`, `physical: not-applicable`; exit 0.
- X03 a statement with physical relevance (29 + 1 = 30 tied to `math29.context`): physical block shows `unestablished (evidence: 0)` read from readingpoints/math29.yaml; exit 0.
- X04 a conditional theorem (M1_conditional): assumptions pinned and printed; exit 0.

**Failure fixtures** (`tests/explain/reject/`, each with its expected exit and message line in an `expected.tsv`):
- Invalid (exit 2): XR01 unknown key · XR02 duplicate key · XR03 id ≠ stem · XR04 proof path `../` · XR05 proof path through a symlink · XR06 missing proof file · XR07 unknown convention · XR08 unknown axiom set · XR09 unknown registry id · XR10 excerpt not a registered unit · XR11 commit differs from registry · XR12 `partial` without `differs` · XR13 `matches` without `assessed_by` · XR14 `unassessed` with a `basis` · XR15 `physical` missing · XR16 `relevant` with an unknown correspondence id · XR17 Lean theorem absent · XR18 a `verdict` key in the record · XR19 control character in a string.
- Stale (exit 3): XS01 proof hash · XS02 definitions hash · XS03 axiom-set hash · XS04 checked goal edited · XS05 assumptions edited · XS06 convention changed.
- Proof failures: XF01 record of tests/m3c/reject/CR10 (exit 1) · XF02 record of a malformed proof (CR17a, exit 2).
- Assessment states (exit 0): XA01 `assessed_goal` differs (assessment stale) · XA02 basis hash differs (assessment stale) · XA03 conversation source (unpinned).

**Property tests** (`cli/tests/explain.rs`):
- **Kernel block identity:** for every record, the kernel block's verdict, checked goal, assumptions, axioms and rules equal `rlt check` on the same proof file.
- **Assessment independence:** rewriting each record's semantic status through all four values and its physical applicability changes no kernel field and no exit status.
- **JSON round trip:** `--format json` parses (with the existing `yaml-rust2` dev-dependency) and every field equals the corresponding terminal line.
- **Preservation:** `scripts/compare-reports.sh` against 58838c9: every existing report byte-identical; `scripts/check-manual.py` unchanged at 23 of 23.

## 8. Trusted boundary

| Component | Trust |
|---|---|
| Kernel | unchanged; the only source of verdicts |
| `rlt explain` (new `cli/src/explain.rs`) | outside the trusted base. Calls `check_file_outcome` for the kernel block; compares pins; formats. A defect could misreport a record's state, never create an ACCEPT: the kernel block is the theorem the kernel returned. |
| Statement records | attributed claims. Their semantic and physical content is printed as the record's claim, with assessor and basis; their pins are checked. |
| Registry and reading-point records | read through the existing loaders, unchanged |
| Lean references | `explain` checks only that the named theorem appears in the file; whether Lean accepts it is `verify.sh`'s check, and the report says so |

## 9. Implementation scope

| Part | Size (estimate) |
|---|---|
| `syntax/src/records.rs`: `load_statement` and the schema of §5, using the existing YAML rules | ~200 lines |
| `cli/src/explain.rs`: pins, blocks, text and JSON writers | ~450 lines |
| `cli/src/main.rs`: the `explain` dispatch and `--format` | ~15 lines |
| `statements/`: X01–X04 records | 4 files |
| `tests/explain/`: 19 invalid, 6 stale, 2 proof-failure, 3 assessment fixtures and `expected.tsv` | ~30 files |
| `cli/tests/explain.rs`: fixtures, golden X01 output, property tests | ~250 lines |
| `scripts/verify.sh`: run every record in `statements/` (additive section) | ~10 lines |
| Docs: manual chapter 10 gains an `rlt explain` example; STATEMENT-RECORDS.md marks the interface implemented; RLT-004b report | — |

**Outside scope:** `rlt deps`, `lib/`, induction, binary numerals, kernel changes, new axioms, backed-by links in RESIDUE-30, changes to `rlt validate`, release, deployment.

## 10. Decisions for review

| ID | Decision | Options | Recommended |
|---|---|---|---|
| X1 | Exit statuses | (a) 0/1/2/3 as in §2; (b) kernel only (0/1/2), with staleness in the report | (a) |
| X2 | Stale assessment | (a) reported in the semantic block, exit unaffected; (b) exit 3 | (a): assessments never affect exit status |
| X3 | E-T1's physical block | (a) `not-applicable` (the statement is mathematical; math29.context concerns the reading point); (b) `relevant`, citing math29.context | (a) |
| X4 | Source fidelity check | (a) against the registry (offline), with upstream verbatim check left to `check_excerpts.py`; (b) fetch sources at check time | (a) |
| X5 | `rlt validate` | (a) unchanged; records checked by `rlt explain` and `verify.sh`; (b) add statement records to `validate` | (a), keeping existing reports byte-identical |
| X6 | JSON | (a) hand-written escaper, no new dependency; (b) add `serde_json` | (a) |
| X7 | Record location | (a) `statements/`; (b) beside proofs in tests/ | (a) |

# RLT-004a — Implementation report

**Branch** `docs/rlt-review` · **PR** #10 (draft) · **Base** main at cc01968 · **Date** 2026-11-11

**Scope, as approved.** Manual chapters 1–11; `docs/KERNEL-SOUNDNESS.md`; a written statement-record specification with a complete Modulo 30 example; baseline measurements of the unary representation (V6 as modified: measure only); a reproducible verification report. Kernel, rules, axiom sets, definitions, conventions and reading-point records unchanged.

## 1. Deliverables

| Deliverable | File | Notes |
|---|---|---|
| User manual, chapters 1–11 | docs/MANUAL.md | Starts from statements and commands; each example gives what you write, what RLT interprets, what the kernel checks, axioms and assumptions, the result, and what remains open. Lean 4 comparison in chapter 11 with links to lean-lang.org. 23 terminal examples. |
| Manual checker | scripts/check-manual.py | Runs every `console` block and requires each shown line, in order, in the real output (exact match when no `…`). Untrusted tooling; outside tests/. |
| Kernel soundness note | docs/KERNEL-SOUNDNESS.md | Semantics and lemmas L1–L4; structural guarantees G1–G9; all 20 rules and citation, each with conditions, argument, implementation, evidence, limitations; trusted base; limitations; evidence summary. Argument and evidence kept in separate fields. |
| Statement records | docs/STATEMENT-RECORDS.md | Four independently reported outcomes; record fields; intended `rlt explain` interface (specified, unimplemented); complete Modulo 30 record for E-T1 with real hashes; the commands that supply each block today. |
| Unary baseline | docs/RLT-004a-BASELINE.md, scripts/measure-baseline.py | Written size, kernel term size, steps, proof nodes, bytes and time, measured separately for equations, non-divisibility certificates and enumerations; limits recorded. |
| Decisions recorded | docs/RLT-004-REVIEW.md | Approval of V1–V10 and the V6 modification. |

## 2. Baseline results (summary)

| Case | Largest accepted | First refused (node limit 1,000,000) |
|---|---|---|
| Enumeration ∀x. x < B → x = 0 ∨ … | B = 47: 1,906 steps, 996,914 nodes, ~1 s | B = 48: 1,073,912 nodes |
| Equation n = p + q (`rlt gen`) | 500 = 13 + 487: 1,463 steps, 900,574 nodes | 1000 = 3 + 997: 3,553,369 nodes; 10000 refused by `rlt gen` (unary limit 1000) |
| ¬(2 ∣ n) | n = 397: 6,173 steps, 646,606 nodes | n = 997: 3,567,406 nodes |

Proof size grows roughly quadratically in the numbers involved (steps × unary numeral size). Representation changes, induction axioms and kernel numeral primitives are deferred to a separate review, as approved. No axiom was added for the benchmark.

## 3. Verification

| Check | Result |
|---|---|
| Files changed against cc01968 | docs/ (7 files) and two new scripts only (`git diff --name-only cc01968`); kernel/, cli/, syntax/, spec/, conventions/, readingpoints/, residues/, sources/, tests/, lean/ and verification/ unchanged |
| `cargo test --release` | 227 passed, 0 failed |
| `scripts/verify.sh /home/claude/src` | exit 0, `RESULT   all checks hold` (proof files, records, excerpts, all Lean replays and axiom gates) |
| `scripts/check-manual.py target/release` | `RESULT 23/23 examples match`; a deliberately altered expected line is reported as FAIL |
| Baseline reproducibility | two runs of `measure-baseline.py` give identical sizes and verdicts (times differ) |
| Existing proofs and reports | unchanged: no file under tests/, spec/ or the record directories is touched, and the binary is built from unchanged sources, so every report is byte-identical to cc01968's |

`verify.sh` rewrites `verification/RLT-001-results.txt` with the run's timestamp; that file was restored and is not part of this change.

**Reproduce.**
```sh
git checkout docs/rlt-review && cargo build --release
cargo test --release
scripts/verify.sh SOURCE_CLONES_DIR
python3 -I scripts/check-manual.py target/release
python3 scripts/measure-baseline.py target/release/rlt /tmp/rlt-baseline
git diff --name-only cc01968 -- kernel cli syntax spec conventions readingpoints residues sources tests lean   # empty
```

## 4. Independent reviews

Both reviews ran on commit 585350d with no access to this report.

### 4.1 Usability review, manual chapters 1–5

**Verdict:** usable with fixes. All commands matched. Findings and disposition:

| Finding | Disposition |
|---|---|
| `rlt` must run from the repository root or with `--root` | Fixed: stated in chapter 1, with the `--root` form |
| `rlt written` takes terms under NAT-DEC only, while the conventions table suggested more | Fixed: chapter 2 says what `rlt written` reads; the table is now about proof-file goals |
| "`hyp h` introduces an assumption" was incomplete: names must be declared with `assume` or discharged; conditional mode was missing | Fixed: chapter 5 explains both sources of names and shows a conditional ACCEPT (verified example) |
| "every structure satisfying these seven axioms" applied only to ROBINSON-Q; soundness dependence unstated | Fixed: chapter 4 now speaks of the named set and points to the soundness note; adds that a proof may use only its own set's axioms |
| "kernel", S, prefix notation, rule names, Γ undefined | Fixed: a "Words used throughout" list and rule glosses in chapter 1; prefix notation explained |
| Long reports with unexplained reading-point lines | Fixed: chapter 1 says which lines to read now and that the others belong to chapter 10 |
| Unicode symbols (∣ vs `|`) | Fixed: chapter 2 lists the symbols goals require and the ASCII names steps use |
| GNU-only `sed`; file never shown before editing | Fixed: note for macOS and a `head` step |
| Proof-file fields never listed | Fixed: listed in chapter 1 |
| Program messages: `rlt written` labels a NAT-DEF input error as NAT-DEC; "axiom `mul_succ` is unregistered" for an axiom outside the named set | Recorded for a later milestone (code changes are outside RLT-004a) |

### 4.2 Technical review, KERNEL-SOUNDNESS.md

**Verdict:** sound with corrections; no rule or citation path admits an invalid theorem. Line numbers verified; G1 (encapsulation) and test counts (29 kernel + 177 CLI + 21 syntax = 227) confirmed. Findings and disposition:

| Finding | Disposition |
|---|---|
| inst needs a substitution lemma beyond L1–L3 | Fixed: L4 added |
| L1 needs φ well-formed under one binder; L2 needs φ locally closed | Fixed: hypotheses stated, with G4 supplying them |
| Conditional ACCEPT not argued | Fixed: G5 now argues it (declared names take declared formulas; cannot be discharged or reused) |
| §4 overstated what citation soundness needs | Fixed: validity relative to recorded axioms needs no condition; the kernel checks serve "rests on the registered set" and unconditional results |
| Front-end refusals (R03, R21, R22a/b, R23a, R24, R27, R34; exit 2) listed as kernel evidence; R25 is a signature refusal | Fixed: moved to a new G9 row (front-end refusals) and removed from the rule entries |
| inst listed checker policy (registration) as a rule condition; exists-elim omitted its `params` checker condition | Fixed: rule and checker conditions labelled separately |
| G8 overstated: size/depth checks cover only `open` and `check_wf`; file limit counts nodes before definition expansion | Fixed: G8 restated precisely |
| "79 derivation-attack files" are the auditor's, outside the repository | Fixed: stated as such |
| Trusted base omitted the axiom reader and pin table, and the CLI's pairing of labels to cited theorems | Fixed: both added to §5 with mitigations |
| Path-check race window; recursion depth | Added as engineering notes (no soundness impact) |

## 5. Constraints honoured

- No change to kernel, rules, axiom sets, definitions, conventions, reading points, residues or source records.
- `rlt explain`, `rlt deps`, induction, binary numerals and the application examples were not implemented; `rlt explain` appears only as an intended interface, labelled as such.
- No mathematical axiom added for the benchmark; limits recorded rather than raised.
- math0, math5 and math29 kept as historical identifiers; the subject is presented as Modulo 30.
- No release or deployment. PR #10 stays in draft.

## 6. Open items for later milestones

1. The two program-message improvements found by the usability review.
2. Manual chapters 12 (writing statement records) and 13 (reference).
3. The separate numerals and induction review, starting from docs/RLT-004a-BASELINE.md.
4. Implementation of statement records and `rlt explain` (V3), then `lib/` and `rlt deps` (V4).

## 7. Merge recommendation

Recommend approval of RLT-004a for merge after the user's review of this report: the change is documentation plus two untrusted scripts, every manual command is machine-checked against the real binary, the full suite and `verify.sh` pass, and both independent reviews' substantive findings are fixed. PR #10 remains in draft until that review.

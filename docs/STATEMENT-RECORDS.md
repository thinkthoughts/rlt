# Statement records and `rlt explain` — specification

**Status.** RLT-004a deliverable, written 2026-11-11. This is a written specification. The record format and `rlt explain` described here are **intended and unimplemented**; nothing in this document is produced by the current `rlt` binary. Where a field can already be filled from an existing command, that command is named.

## 1. Purpose

A theorem prover answers one question: does this formula follow from these axioms by these rules? Readers of RLT usually care about three more:

- Is the formula an accurate rendering of the sentence someone wrote?
- Does the formula mean, mathematically, what that sentence means?
- Does the sentence describe something in the world, and is there evidence for that?

A **statement record** keeps these four questions together for one statement and reports each answer on its own. The governing rule:

> **Independence.** An accepted theorem establishes kernel acceptance only. It never establishes source fidelity, semantic correspondence or physical correspondence, and none of those three changes a kernel verdict.

## 2. The four outcomes

| Outcome | Question | Possible statuses | Who or what decides | Available today |
|---|---|---|---|---|
| **Kernel acceptance** | Does the checked formula follow from the listed axioms by the 20 rules (and checked citations)? | `ACCEPT` · `REJECT at step …` · `MALFORMED` | The kernel, deterministically | `rlt check FILE` |
| **Source fidelity** | Is the quoted source text exactly what the source says, at a recorded version? | `verified` (excerpt found verbatim at the pinned commit) · `differs` · `unpinned` (no stable location) | `scripts/check_excerpts.py` and `rlt validate` | `rlt validate`, `rlt rp ID` |
| **Semantic correspondence** | Does the checked formula mean what the quoted sentence means? | `unassessed` · `matches` · `partial` (states what differs) · `differs` — each with the reviewer or tool that assessed it | Human review, optionally supported by a Lean 4 statement (tier 2) | Lean readings files (`lean/*Readings.lean`), review notes in docs/ |
| **Physical correspondence** | Does the sentence describe the world, and with what evidence? | `unestablished` (evidence: 0) · `registered evidence` (with evidence records) | Evidence records, never a proof | `rlt rp ID` (`physical` lines) |

**Source hashes.** A SHA-256 recorded for a source, axiom file, definitions file or cited proof verifies that the content is exactly the recorded content. It says nothing about whether a formula faithfully translates that content. Fidelity of *quotation* and correspondence of *meaning* are separate outcomes.

## 3. Record fields

A statement record is a YAML file (intended location: `statements/<id>.yaml`). Every field is either a pointer to an existing checked artifact or a reviewed, attributed assessment.

| Field | Content | Source of truth |
|---|---|---|
| `id` | Stable identifier | the record |
| `subject` | The topic in standard vocabulary (e.g. "Modulo 30") and any historical identifiers (e.g. math29) | the record |
| `source` | Registry id, location, commit, and the exact excerpt | `sources/registry.yaml` |
| `written` | The statement as RLT reads it (the `goal` string) | the proof file |
| `interpretation` | Convention, definitions expanded (with the definitions file hash), checked goal | `rlt check` report |
| `kernel` | Proof file path and SHA-256, verdict, axiom set and hash, axioms used, assumptions, rules used, theorems cited | `rlt check` report |
| `semantic` | Status, what matches, what differs, assessed by, date; optional Lean statement and its axioms | review record |
| `physical` | Status and evidence records, or `unestablished` | `readingpoints/*.yaml` |
| `open_questions` | Specification and correspondence questions that remain | review record |

## 4. Intended `rlt explain` interface

```
rlt explain STATEMENT_RECORD            (intended; unimplemented)
```

It would print four blocks, one per outcome, in this order, each with its own status line. It would re-run `rlt check` on the proof file and re-verify every hash it prints; the semantic and physical blocks would be read from the record and attributed. Exit status would reflect kernel acceptance only, so that scripts cannot mistake an attributed assessment for a check.

## 5. Complete example: the Modulo 30 classification

### 5.1 The record

```yaml
id: MOD30-CLASSIFICATION
subject: Modulo 30 — residues below 30 coprime to 2, 3 and 5
historical_identifiers: [math29]          # preserved as recorded in readingpoints/math29.yaml
source:
  registry: NEWS-MATH29
  location: https://github.com/thinkthoughts/news/blob/1ffd5ea9886386dc0b16c13499ecfe2ed59fd9e9/MATH_29.md
  commit: 1ffd5ea9886386dc0b16c13499ecfe2ed59fd9e9
  excerpts:
    - "{1, 7, 11, 13, 17, 19, 23, 29}: 17111317192329"
    - "For instance (<a href=\"https://c5.now/\">nonfiction</a>): (1+1) × 3 × 5 = 30 > 29, where every prime > 5 resists division by 2, 3, and 5 and corresponds to one of eight residue classes modulo 30:"
written: "∀r. r < 30 → (free235(r) → (r = 1 ∨ r = 7 ∨ r = 11 ∨ r = 13 ∨ r = 17 ∨ r = 19 ∨ r = 23 ∨ r = 29)) ∧ ((r = 1 ∨ r = 7 ∨ r = 11 ∨ r = 13 ∨ r = 17 ∨ r = 19 ∨ r = 23 ∨ r = 29) → free235(r))"
interpretation:
  convention: NAT-DEF
  definitions:
    file: spec/definitions.rlt
    sha256: 122e37fc627b136023a1a5ffac0344162ed77ae95312248a9f6a76c2b639ecda
    used:
      lt: "x < y := ∃z. S(z) + x = y"            # Open Logic Project, Robinson's Q, Q8
      dvd: "d ∣ n := ∃k. n = d × k"              # as in Lean 4 core
      free235: "free235(r) := (¬2 ∣ r ∧ ¬3 ∣ r) ∧ ¬5 ∣ r"   # RLT-authored
kernel:
  proof: tests/m3e/accept/E_T1.rlt
  sha256: e11712c64baf6fcb40bc172b5d7e12607809c85b9f1cd087064a0c17ca651750
  axiom_set: {id: ROBINSON-Q, file: spec/robinson-q.rlt, sha256: 4ad55192222576e3901abc4ee7ea5a6760b64ff212e8c55796ab9847e423b00c}
  cites:
    - {label: FWD, path: tests/m3e/accept/E_FWD.rlt, sha256: 4758d9dd7ffb2d875e11b0cbeac3eb0d47faae5e3b59308e831fa25c01a92182}
    - {label: BWD, path: tests/m3e/accept/E_BWD.rlt, sha256: 10e27c9f3b49b902cefae86ffc0f3067d32abdbac754a64cf3256efd524d16bf}
semantic:
  status: partial
  matches:
    - "The eight listed representatives are exactly the r < 30 with free235(r) (E-T1, both directions, in Robinson's Q)."
    - "Over ℕ, free235(r) agrees with gcd(r, 30) = 1 for r < 30 (Lean 4, lean/M3eReadings.lean, free235_iff_coprime; standard axioms only)."
  differs:
    - "The source sentence is about every prime > 5 and residue classes modulo 30; E-T1 is about the 30 numbers below 30. Remainders, congruence classes and primes above 5 are outside the checked formula."
  assessed_by: RLT-003 M3e review and report (docs/RLT-003-M3e.md)
physical:
  status: unestablished
  evidence: 0
  note: "math29.context [referent]: a driver's immediate context at a specific stop sign" (readingpoints/math29.yaml)
open_questions:
  - "A theorem about every n: n mod 30 lies in the eight classes exactly when gcd(n, 30) = 1. Needs division with remainder for all n, which needs induction; deferred to a separate review."
  - "A theorem about primes > 5. Needs a definition of prime and the quantified statement; same dependency."
  - "Whether RESIDUE-30's registered reading should cite E-T1 as backing (a future backed-by link; records unchanged in RLT-004a)."
```

### 5.2 What `rlt explain` would print (intended)

```
statement        MOD30-CLASSIFICATION — Modulo 30 (historical identifier: math29)

1 KERNEL ACCEPTANCE                                   status ACCEPT
  proof          tests/m3e/accept/E_T1.rlt  sha256 e11712c6…  (verified now)
  checked goal   ∀r. r < 30 → (free235(r) → r ∈ {1,7,11,13,17,19,23,29}) ∧ (… → free235(r))
  axiom set      ROBINSON-Q  sha256 4ad55192…  (verified now); axioms used: all seven
  assumptions    Γ = ∅
  cites          FWD (E_FWD.rlt), BWD (E_BWD.rlt) — checked now, hashes verified
  meaning        the checked formula follows from Robinson's Q; nothing further

2 SOURCE FIDELITY                                     status verified
  source         NEWS-MATH29 @ 1ffd5ea9  excerpt "{1, 7, 11, 13, 17, 19, 23, 29}: 17111317192329"
  meaning        the quotation is exact at the pinned commit; a hash match is not a translation check

3 SEMANTIC CORRESPONDENCE                             status partial (RLT-003 M3e review)
  matches        the eight representatives are exactly the r < 30 coprime to 2, 3, 5
  differs        the source speaks of every prime > 5 and of residue classes; the theorem stops at r < 30

4 PHYSICAL CORRESPONDENCE                             status unestablished (evidence: 0)

open questions   3 (see record)
exit 0           (reflects block 1 only)
```

### 5.3 How each block is obtained from today's commands

| Block | Today's command | What it shows |
|---|---|---|
| 1 | `rlt check tests/m3e/accept/E_T1.rlt` | verdict, checked goal, axioms, citations with hashes |
| 2 | `rlt validate`; `rlt rp math29` | excerpts verified against their sources; the reading point's meanings with source ids |
| 3 | `cd lean && lean M3eReadings.lean`; docs/RLT-003-M3e.md | the Lean statements over ℕ and the review's scope statement |
| 4 | `rlt rp math29` | `physical … unestablished (evidence: 0)` |

`rlt residues residues/residue-30.yaml` reports its own scope: its line `not established uniqueness of remainders · non-divisibility · gcd(r, 30) = 1 · completeness of the representatives · …` describes what the M2 witness records establish. E-T1 is a separate, later theorem; connecting the two is the backed-by question listed above, and the records stay unchanged until that is reviewed.

## 6. Rules for writing a statement record

1. Quote sources exactly; record the commit. Never paraphrase inside `excerpts`.
2. Copy `interpretation` and `kernel` from an actual `rlt check` report.
3. Every `semantic` status names who or what assessed it and what differs.
4. `physical` defaults to `unestablished`. Only evidence records change it, and proofs never do.
5. Keep historical identifiers (math0, math5, math29) as recorded; describe the subject in standard vocabulary.
6. List open questions as specification questions, never as results.

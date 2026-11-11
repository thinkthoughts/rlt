# RLT user manual — chapters 1–11

**Status.** RLT-004a, written 2026-11-11 against main at cc01968. Every terminal block in this manual is checked against the real program by `scripts/check-manual.py`; lines shown as `…` are omitted output. Chapters 12 (writing your own statement records) and 13 (reference) are planned for a later milestone.

**Who this is for.** Anyone who wants to run RLT on their own machine and read what it reports, including readers who already use [Lean 4](https://lean-lang.org/). Chapter 11 compares the two precisely.

**The running example** is Modulo 30: which numbers below 30 share no factor with 30 = 2 × 3 × 5. RLT's history records this subject under the reading point **math29**; that identifier is kept as recorded (with **math0** and **math5**), and the mathematics is described in standard vocabulary.

**How every example is described.** Each example ends with a box of six lines:

| | |
|---|---|
| **You write** | the text in the proof file or on the command line |
| **RLT interprets** | the convention and definitions that turn it into a formula |
| **The kernel checks** | the formula the kernel actually proves |
| **Axioms and assumptions** | what the result rests on |
| **Result** | the exact verdict |
| **Still open** | specification or correspondence questions the result leaves open |

---

## 1. First statement: 29 + 1 = 30

Build RLT (Rust 1.97.0, pinned in `rust-toolchain.toml`) and put the binary on your path:

```sh
git clone https://github.com/thinkthoughts/rlt && cd rlt
cargo build --release
export PATH="$PWD/target/release:$PATH"
```

Check a proof that 29 + 1 = 30:

```console
$ rlt check tests/accept/R_math29_pairing.rlt; echo "exit $?"
RLT check       tests/accept/R_math29_pairing.rlt
proof id        R.math29.pairing
reading point   math29
…
written goal    "29 + 1 = 30"
…
convention      NAT-DEC (authority: interpretation)
checked goal    29 + 1 = 30
…
axiom set       PEANO-ADD (spec/peano.rlt, source PEANO)
…
  axiom         add_zero : ∀ x. x + 0 = x
  axiom         add_succ : ∀ x y. x + S(y) = S(x + y)
…
steps           4
proved          29 + 1 = 30
assumptions     Γ = ∅
axioms used     add_succ, add_zero
rules used      cong, inst, trans
…
verdict         ACCEPT — unconditional theorem relative to registered axiom set PEANO-ADD
…
physical        math29.context [referent]: unestablished (evidence: 0)
…
exit 0
```

The proof file is four steps. Each names a rule and the earlier steps or axiom instances it uses:

```console
$ grep -A4 '(steps' tests/accept/R_math29_pairing.rlt
  (steps
    (s1 (= (+ 29 (S 0)) (S (+ 29 0)))  (inst add_succ (x 29) (y 0)))
    (s2 (= (+ 29 0) 29)                (inst add_zero (x 29)))
    (s3 (= (S (+ 29 0)) (S 29))        (cong S s2))
    (s4 (= (+ 29 (S 0)) (S 29))        (trans s1 s3))))
```

Exit status: **0** accepted, **1** rejected (a step fails a rule), **2** malformed (the file cannot be read as a proof, or exceeds a limit).

| | |
|---|---|
| **You write** | `(goal "29 + 1 = 30")` with convention NAT-DEC, axiom set PEANO-ADD, four steps |
| **RLT interprets** | decimal numerals as successor terms: 29 is S applied 29 times to 0 |
| **The kernel checks** | `+(S²⁹(0), S(0)) = S³⁰(0)` by inst, cong and trans |
| **Axioms and assumptions** | add_zero, add_succ; Γ = ∅ (no assumptions) |
| **Result** | ACCEPT — unconditional theorem relative to PEANO-ADD |
| **Still open** | math29's meanings and its physical claim (`unestablished`) keep their own status; the verdict leaves them unchanged |

## 2. Written statements and what RLT reads

RLT separates **what you write** from **what is checked**. `rlt written` shows both for two expressions:

```console
$ rlt written "29 + 1" NAT-DEC "30" NAT-DEC
written A       "29 + 1"  tokens: 29 · + · 1
written B       "30"  tokens: 30
written ≡w      distinct
…
terms           structurally distinct
equality        decided only by a kernel proof (`rlt check`); this report proves nothing
```

Spacing never matters; the tokens do:

```console
$ rlt written "1+1" NAT-DEC "1 + 1" NAT-DEC
written A       "1+1"  tokens: 1 · + · 1
written B       "1 + 1"  tokens: 1 · + · 1
written ≡w      identical
elaborated A    1 + 1 under NAT-DEC  kernel form: +(S(0), S(0))
elaborated B    1 + 1 under NAT-DEC  kernel form: +(S(0), S(0))
terms           structurally identical
…
```

A **convention** says how written text becomes a formula. Three are used for proofs:

| Convention | Reads | Example |
|---|---|---|
| NAT-DEC | closed arithmetic: decimal numerals, `+`, `×`, `=` | `30 = 2 × 3 × 5` (× groups to the left: `(2 × 3) × 5`) |
| NAT-LOGIC | adds ∀, ∃, ∧, ∨, →, ¬, ⊥ and variables | `∀x. x + 0 = x` |
| NAT-DEF | adds the registered definitions `<`, `∣`, `free235` (chapter 3); ∨ groups to the left | `¬(2 ∣ 7)` |

```console
$ rlt check tests/accept/M1_thirty.rlt
…
written goal    "30 = 2 × 3 × 5"
…
checked goal    30 = (2 × 3) × 5
…
steps           162
…
verdict         ACCEPT — unconditional theorem relative to registered axiom set PEANO-ARITH
…
```

| | |
|---|---|
| **You write** | `30 = 2 × 3 × 5` |
| **RLT interprets** | NAT-DEC: × groups left, so `(2 × 3) × 5` |
| **The kernel checks** | `30 = (2 × 3) × 5` over unary numerals |
| **Axioms and assumptions** | PEANO-ARITH (add_zero, add_succ, mul_zero, mul_succ); Γ = ∅ |
| **Result** | ACCEPT |
| **Still open** | whether the grouping is what a given author meant; the report prints the checked goal so a reader can compare |

## 3. Definitions and where they come from

Three abbreviations are registered in `spec/definitions.rlt`, each with its source. They expand *before* the kernel, so the kernel only ever sees 0, S, +, × and logic:

```console
$ grep '^(define' spec/definitions.rlt
(define lt (x y) (exists z (= (+ (S z) x) y)) (notation infix "<") (source OLP-ROBINSON-Q))
(define dvd (d n) (exists k (= n (* d k))) (notation infix "∣") (source LEAN-CORE-NAT-DVD))
(define free235 (r) (and (and (not (dvd 2 r)) (not (dvd 3 r))) (not (dvd 5 r))) (notation prefix "free235") (origin rlt-authored))
```

| Name | Meaning | Source |
|---|---|---|
| `x < y` | ∃z. S(z) + x = y | Robinson's Q, Q8 (Open Logic Project) |
| `d ∣ n` | ∃k. n = d × k | Lean 4 core's `Nat` divisibility (same form) |
| `free235(r)` | (¬2 ∣ r ∧ ¬3 ∣ r) ∧ ¬5 ∣ r | RLT-authored: no prime factor 2, 3 or 5 |

Every report that uses a definition prints the expansion and the definitions file's hash:

```console
$ rlt check tests/m3c/accept/CA01_two_divides_eight.rlt
…
written goal    "2 ∣ 8"
convention      NAT-DEF (authority: interpretation)
checked goal    ∃k. 8 = 2 × k
  folded        2 ∣ 8
definitions     dvd — abbreviations expanded before the kernel (spec/definitions.rlt sha256 122e37fc627b136023a1a5ffac0344162ed77ae95312248a9f6a76c2b639ecda)
…
verdict         ACCEPT — unconditional theorem relative to registered axiom set ROBINSON-Q
…
```

| | |
|---|---|
| **You write** | `2 ∣ 8` |
| **RLT interprets** | NAT-DEF expands `∣` to ∃k. 8 = 2 × k |
| **The kernel checks** | ∃k. 8 = 2 × k (witness k = 4, by exists-intro) |
| **Axioms and assumptions** | ROBINSON-Q; Γ = ∅ |
| **Result** | ACCEPT |
| **Still open** | that the definition says what a reader means by "divides" is a correspondence question; the source column records where it was taken from |

## 4. Axioms

Every theorem is **relative to an axiom set**. RLT has three, each a file in `spec/` pinned by SHA-256:

| Axiom set | Axioms | Used for |
|---|---|---|
| PEANO-ADD | add_zero, add_succ | addition facts |
| PEANO-ARITH | adds mul_zero, mul_succ | closed arithmetic (`rlt gen`) |
| ROBINSON-Q | adds q1_succ_inj, q2_zero_ne_succ, q3_zero_or_succ | statements with ¬, ∀, ∃ |

```console
$ grep '^(axiom' spec/robinson-q.rlt
(axiom q1_succ_inj     (x y) (imp (= (S x) (S y)) (= x y))       (source OLP-ROBINSON-Q))
(axiom q2_zero_ne_succ (x)   (not (= 0 (S x)))                   (source OLP-ROBINSON-Q))
(axiom q3_zero_or_succ (x)   (or (= x 0) (exists y (= x (S y)))) (source OLP-ROBINSON-Q))
(axiom add_zero        (x)   (= (+ x 0) x)                       (source OLP-ROBINSON-Q))
(axiom add_succ        (x y) (= (+ x (S y)) (S (+ x y)))         (source OLP-ROBINSON-Q))
(axiom mul_zero        (x)   (= (* x 0) 0)                       (source OLP-ROBINSON-Q))
(axiom mul_succ        (x y) (= (* x (S y)) (+ (* x y) x))       (source OLP-ROBINSON-Q))
```

In words: successor is one-to-one (Q1); zero is no successor (Q2); every number is zero or a successor (Q3); and addition and multiplication are defined by recursion on the second argument. Robinson's Q has **no induction axiom**. It proves facts about particular numbers and finite ranges, which is everything the Modulo 30 chapters need; statements about *every* number generally need induction, which RLT does not have (chapter 9, "Still open").

"ACCEPT relative to ROBINSON-Q" means: the formula holds in every structure satisfying these seven axioms. ℕ is one of them, so the formula is true of the natural numbers. The axioms themselves are assumptions; RLT prints each one used.

## 5. Proofs: steps, rules and assumptions

A proof is a list of steps. Each step states a formula and the rule that justifies it. The kernel has exactly 20 rules:

```
refl symm trans cong inst hyp subst
and-intro and-left and-right or-left or-right or-cases
imp-intro imp-elim false-elim forall-intro forall-elim exists-intro exists-elim
```

and one way to use another checked theorem, `cite` (chapter 8). docs/KERNEL-SOUNDNESS.md gives each rule's conditions and why it is sound.

**Write a proof without typing it.** `rlt gen` is an *untrusted* generator for closed equations; the kernel checks everything it writes:

```console
$ rlt gen "10 = 3 + 7" MY.ten > /tmp/ten.rlt && rlt check /tmp/ten.rlt; echo "exit $?"
…
proof id        MY.ten
…
axiom set       PEANO-ARITH (spec/peano-arith.rlt, source OLP-ROBINSON-Q)
…
steps           23
proved          10 = 3 + 7
…
rules used      cong, inst, symm, trans
…
verdict         ACCEPT — unconditional theorem relative to registered axiom set PEANO-ARITH
…
exit 0
```

**Break one step and see the kernel refuse it.** Change the first instance of `(y 6)` to `(y 5)`:

```console
$ sed '0,/(y 6)/s//(y 5)/' /tmp/ten.rlt > /tmp/ten-bad.rlt && rlt check /tmp/ten-bad.rlt; echo "exit $?"
…
verdict         REJECT at step s1 — step claims `3 + 7 = S(3 + 6)`, rule `inst` yields `3 + 6 = S(3 + 5)`
…
exit 1
```

The message names the step, what it claimed and what the rule actually yields. A rejection is a fact about this derivation only; the goal's truth stays undetermined by it.

**Assumptions and Γ.** `hyp h` introduces an assumption named h. Rules such as imp-intro, or-cases and exists-elim discharge assumptions. The report's `assumptions` line shows what is left, written Γ. An **unconditional** proof must end with Γ = ∅; RLT refuses it otherwise.

| | |
|---|---|
| **You write** | `10 = 3 + 7` (via `rlt gen`), then a hand edit |
| **RLT interprets** | NAT-DEC numerals |
| **The kernel checks** | each of 23 steps; in the edited file, step s1 fails |
| **Axioms and assumptions** | PEANO-ARITH; Γ = ∅ |
| **Result** | ACCEPT for the generated file; REJECT at step s1 for the edited one |
| **Still open** | nothing for these closed equations beyond the axioms' own status |

## 6. Proving that something never happens: ¬(2 ∣ 7)

¬(2 ∣ 7) says: there is no k with 7 = 2 × k. RLT proves it by cases on k using Q3 (k is 0 or a successor), peeling one successor at a time until each case contradicts Q1 or Q2:

```console
$ rlt check tests/m3c/accept/CA02_two_not_divides_seven.rlt
…
written goal    "¬(2 ∣ 7)"
convention      NAT-DEF (authority: interpretation)
checked goal    ¬(∃k. 7 = 2 × k)
…
steps           128
proved          ¬(∃k. 7 = 2 × k)
assumptions     Γ = ∅
axioms used     add_succ, add_zero, mul_succ, mul_zero, q1_succ_inj, q2_zero_ne_succ, q3_zero_or_succ
rules used      cong, exists-elim, hyp, imp-elim, imp-intro, inst, or-cases, refl, symm, trans
…
verdict         ACCEPT — unconditional theorem relative to registered axiom set ROBINSON-Q
…
```

The case tree, in words: assume 7 = 2 × k. If k = 0, then 7 = 0, contradicting Q2. Otherwise k = S(k₁) and 7 = 2 × k₁ + 2, so (Q1, twice) 5 = 2 × k₁. Repeat: k₁ = 0 gives 5 = 0; k₁ = S(k₂) gives 3 = 2 × k₂; then 1 = 2 × k₃; and finally k₃ = 0 gives 1 = 0 while k₃ = S(k₄) gives 1 = 2 × k₄ + 2, i.e. 0 = S(…). Every branch ends in a contradiction.

The same method applied to a true divisibility fails, as it should. In this test file a generated certificate for ¬(2 ∣ 8) has one branch that needs 0 ≠ 0:

```console
$ rlt check tests/m3c/reject/CR10_two_not_divides_eight.rlt; echo "exit $?"
…
verdict         REJECT at step L4_z_q2 — step claims `¬(0 = 0)`, rule `inst` yields `¬(0 = 1)`
…
exit 1
```

| | |
|---|---|
| **You write** | `¬(2 ∣ 7)` |
| **RLT interprets** | ¬(∃k. 7 = 2 × k) |
| **The kernel checks** | 128 steps: case splits (or-cases), witnesses (exists-elim), contradictions (imp-elim with Q2) |
| **Axioms and assumptions** | all seven of ROBINSON-Q; Γ = ∅ |
| **Result** | ACCEPT |
| **Still open** | none for this closed statement; certificate size grows with the numbers (chapter 7's table) |

## 7. Finite enumeration: every number below 30

The statement "every x < 30 is one of 0, 1, …, 29":

```console
$ rlt check tests/m3d/accept/DA01_below_thirty.rlt
…
convention      NAT-DEF (authority: interpretation)
…
definitions     lt — abbreviations expanded before the kernel (spec/definitions.rlt sha256 122e37fc627b136023a1a5ffac0344162ed77ae95312248a9f6a76c2b639ecda)
…
steps           971
…
assumptions     Γ = ∅
axioms used     add_succ, add_zero, q1_succ_inj, q2_zero_ne_succ, q3_zero_or_succ
…
verdict         ACCEPT — unconditional theorem relative to registered axiom set ROBINSON-Q
…
```

The proof splits x with Q3 thirty-one times. This is a genuine ∀ statement, proved without induction because the range is finite.

**Sizes.** Numerals are unary inside the kernel, so certificates grow with the numbers. Measured on the current binary (docs/RLT-004a-BASELINE.md; reproduce with `scripts/measure-baseline.py`):

| Statement | Steps | Proof nodes | Result |
|---|---|---|---|
| ∀x. x < 30 → x = 0 ∨ … ∨ x = 29 | 971 | 215,390 | ACCEPT |
| same, below 47 | 1,906 | 996,914 | ACCEPT |
| same, below 48 | — | 1,073,912 | MALFORMED: above the 1,000,000-node file limit |
| ¬(2 ∣ 397) | 6,173 | 646,606 | ACCEPT |
| ¬(2 ∣ 997) | — | 3,567,406 | MALFORMED: above the limit |
| 500 = 13 + 487 | 1,463 | 900,574 | ACCEPT |
| 1000 = 3 + 997 | — | 3,553,369 | MALFORMED: above the limit |

| | |
|---|---|
| **You write** | `∀x. x < 30 → x = 0 ∨ x = 1 ∨ … ∨ x = 29` |
| **RLT interprets** | `<` expanded; ∨ grouped to the left |
| **The kernel checks** | 971 steps over ROBINSON-Q |
| **Axioms and assumptions** | five of the seven axioms; Γ = ∅ |
| **Result** | ACCEPT |
| **Still open** | larger bounds need a different representation or method; that is a separate, deferred review |

## 8. Reusing theorems: citation

A proof can **cite** another accepted proof file instead of repeating it. free235(7) cites the three non-divisibility theorems:

```console
$ grep -A3 '(cites' tests/m3c/accept/CA05_free235_seven.rlt
  (cites
    (T2 "tests/m3c/accept/CA02_two_not_divides_seven.rlt" "bafaefafd5fde46346a7da3e2c8ee7b405db76ace31886c2faf26ad1ecf94e1b")
    (T3 "tests/m3c/accept/CA03_three_not_divides_seven.rlt" "bfec4c786c3b80c823812266023880ce2b58545d3ad4f4ca673b1b52f69355a6")
    (T4 "tests/m3c/accept/CA04_five_not_divides_seven.rlt" "5877f73c4b6fef01f0731e46b60210f7cd68a845ebb357ebf1175d0b00017570"))
```

```console
$ rlt check tests/m3c/accept/CA05_free235_seven.rlt
…
checked goal    (¬(∃k. 7 = 2 × k) ∧ ¬(∃k. 7 = 3 × k)) ∧ ¬(∃k. 7 = 5 × k)
  folded        free235(7)
…
cites           T2 = tests/m3c/accept/CA02_two_not_divides_seven.rlt (M3c.CA02)
  sha256        bafaefafd5fde46346a7da3e2c8ee7b405db76ace31886c2faf26ad1ecf94e1b
  proved        ¬(∃k. 7 = 2 × k)
…
steps           5
…
rules used      and-intro
theorems cited  T2, T3, T4 — checked theorem dependencies (see `cites`), not inference rules
…
verdict         ACCEPT — unconditional theorem relative to registered axiom set ROBINSON-Q
…
```

Each cited file is checked again from scratch, at exactly the recorded SHA-256, and must be unconditional under the same axiom set and convention. Paths are repository-relative; cycles are refused; nesting is limited to 8 citing files and 64 cited-file checks. A changed hash is refused before any checking:

```console
$ rlt check tests/m3c/reject/CR17a_cited_hash_differs.rlt; echo "exit $?"
…
verdict         MALFORMED — citation `T2`: tests/m3c/accept/CA02_two_not_divides_seven.rlt has SHA-256 bafaefafd5fde46346a7da3e2c8ee7b405db76ace31886c2faf26ad1ecf94e1b; the citation records bfec4c786c3b80c823812266023880ce2b58545d3ad4f4ca673b1b52f69355a6
exit 2
```

| | |
|---|---|
| **You write** | `(cite T2)` steps plus a `(cites …)` block with paths and hashes |
| **RLT interprets** | each cited file, re-checked at its recorded content |
| **The kernel checks** | the cited theorems (unconditional, same axioms by content) and 5 steps of and-intro |
| **Axioms and assumptions** | the union of the cited theorems' axioms; Γ = ∅ |
| **Result** | ACCEPT |
| **Still open** | a hash confirms the file's content, never that the file states what a source means |

## 9. The Modulo 30 classification

The main theorem so far: below 30, free235(r) holds exactly for the eight numbers 1, 7, 11, 13, 17, 19, 23, 29.

```console
$ rlt check tests/m3e/accept/E_T1.rlt; echo "exit $?"
…
convention      NAT-DEF (authority: interpretation)
…
  folded        ∀r. r < 30 → (free235(r) → (r = 1 ∨ r = 7 ∨ r = 11 ∨ r = 13 ∨ r = 17 ∨ r = 19 ∨ r = 23 ∨ r = 29)) ∧ ((r = 1 ∨ r = 7 ∨ r = 11 ∨ r = 13 ∨ r = 17 ∨ r = 19 ∨ r = 23 ∨ r = 29) → free235(r))
definitions     dvd, free235, lt — abbreviations expanded before the kernel (spec/definitions.rlt sha256 122e37fc627b136023a1a5ffac0344162ed77ae95312248a9f6a76c2b639ecda)
…
cites           FWD = tests/m3e/accept/E_FWD.rlt (M3e.E_FWD)
…
cites           BWD = tests/m3e/accept/E_BWD.rlt (M3e.E_BWD)
…
steps           9
…
assumptions     Γ = ∅
…
theorems cited  BWD, FWD — checked theorem dependencies (see `cites`), not inference rules
…
verdict         ACCEPT — unconditional theorem relative to registered axiom set ROBINSON-Q
…
exit 0
```

**How to read it.** FWD proves: if r < 30 and free235(r), then r is one of the eight. It cites the enumeration of chapter 7 and 22 non-divisibility certificates (one for each excluded r). BWD proves: each of the eight satisfies free235, from 24 certificates like chapter 6's. E-T1 combines the two directions with 9 steps. Checking it re-checks 49 files.

| | |
|---|---|
| **You write** | the ∀r statement above |
| **RLT interprets** | `<`, `∣`, `free235` expanded; ∨ grouped left |
| **The kernel checks** | the expanded formula, with FWD and BWD re-checked and cited |
| **Axioms and assumptions** | all seven of ROBINSON-Q; Γ = ∅ |
| **Result** | ACCEPT |
| **Still open** | the source sentence speaks of every prime > 5 and residue classes; this theorem stops at r < 30. Extending it needs division with remainder for all n, hence induction: deferred to a separate review |

## 10. Theorem, source and correspondence

A theorem answers one question. RLT keeps three others separately, and an accepted proof never answers them:

| Question | Where RLT reports it |
|---|---|
| Does the formula follow from the axioms? | `rlt check` (verdict) |
| Is the quoted source exact? | `rlt validate`, `rlt rp` |
| Does the formula mean what the source means? | review notes and Lean readings (chapter 11) |
| Does it describe the world, with what evidence? | `physical` lines in `rlt rp` and `rlt check` |

A reading point holds a subject's sources, exactly as quoted:

```console
$ rlt rp math29
reading point   math29
…
  "{1, 7, 11, 13, 17, 19, 23, 29}: 17111317192329"  [NEWS-MATH29]
…
relationships (attached results, checked now)
  math29.pairing  "29 + 1 = 30" under NAT-DEC  →  ACCEPT — unconditional theorem relative to registered axiom set PEANO-ADD, Γ = ∅  (tests/accept/R_math29_pairing.rlt)
physical correspondence (evidence-graded, independent of verdicts)
  math29.context [referent] record valid · physical unestablished (evidence: 0) — "a driver's immediate context at a specific stop sign"
…
```

`rlt validate` checks every quotation against its source and the records against each other:

```console
$ rlt validate; echo "exit $?"
…
reading point   math29: 15 meanings, 1 relationships, 1 correspondence records, 0 lifts
…
residues        residue-30.yaml: 0 problem(s)
verdict         VALID — records consistent
exit 0
```

The Modulo 30 witness records (from an earlier milestone) state their own scope; they are registered readings of closed equations:

```console
$ rlt residues residues/residue-30.yaml
…
not established uniqueness of remainders · non-divisibility · gcd(r, 30) = 1 · completeness of the representatives · congruence · periodicity · statements about primes > 5
…
verdict         VALID — witness records consistent (40 record(s) valid; readings are registered interpretations, not theorems)
```

That line describes those records. E-T1 (chapter 9) is a later, separate theorem; linking it to the records as backing is a reviewed change that has not yet been made. docs/STATEMENT-RECORDS.md specifies how one statement's four outcomes will be reported together.

| | |
|---|---|
| **You write** | nothing new: these commands read records |
| **RLT interprets** | each quotation against its pinned source |
| **The kernel checks** | the attached relationships (29 + 1 = 30 here) |
| **Axioms and assumptions** | as each attached proof reports |
| **Result** | VALID records; ACCEPT for attached theorems; physical: unestablished |
| **Still open** | semantic correspondence of each meaning; physical evidence |

## 11. Lean 4 and RLT

[Lean 4](https://lean-lang.org/) is a functional programming language and interactive theorem prover based on dependent type theory. Its documentation includes [Theorem Proving in Lean 4](https://lean-lang.org/theorem_proving_in_lean4/) and the [Lean language reference](https://lean-lang.org/doc/reference/latest/).

**How they differ.**

| | RLT | Lean 4 |
|---|---|---|
| Logic | classical first-order logic over 0, S, +, × | dependent type theory with inductive types |
| Natural numbers | axiom sets (PEANO-ADD, PEANO-ARITH, Robinson's Q); no induction | an inductive type with induction and recursion; literals with built-in kernel support |
| Proofs | an explicit list of steps, each one of 20 rules | proof terms, usually produced by tactics and the elaborator |
| Automation | none in the kernel; external generators are untrusted | extensive (`decide`, `omega`, `simp`, `grind`, …) |
| What is reported | the written goal, checked goal, kernel form, every axiom used, Γ, rules and cited theorems | the theorem's type; `#print axioms` lists axioms used |
| Scope | small closed statements and finite ranges, with sources and correspondence kept separately | general mathematics (with Mathlib), programming, verification |

**What RLT's Lean comparison checks.** Two tiers, run by `scripts/verify.sh`:

- **Tier 1 (`lean/*Generated.lean`).** Each statement RLT accepted is restated in Lean and proved there independently, usually by `decide` or `omega`. Lean never sees RLT's derivation, so agreement means two independent systems accept the same statement, nothing about RLT's proof.
- **Tier 2 (`lean/*Readings.lean`).** Hand-written Lean statements about what a result *means* over ℕ, for example that free235 agrees with coprimality to 30 below 30. These are Lean theorems, not RLT theorems.
- **Axiom gate.** Every Lean statement may use only Lean's standard axioms (`propext`, `Classical.choice`, `Quot.sound`).

```console
$ rlt lean tests/m3c/accept/CA02_two_not_divides_seven.rlt
…
-- tests/m3c/accept/CA02_two_not_divides_seven.rlt  (axiom set: ROBINSON-Q)
…
end RLTGenerated
```

```console
$ cd lean && lean M3eReadings.lean
'RLTM3eReadings.classification' depends on axioms: [propext, Quot.sound]
'RLTM3eReadings.free235_iff_coprime' depends on axioms: [propext]
'RLTM3eReadings.representatives_count' does not depend on any axioms
```

(The Lean toolchain is pinned in `lean/lean-toolchain`; install Lean with [elan](https://lean-lang.org/install).)

**If you know Lean.** RLT's `d ∣ n` is Lean core's `Nat` divisibility (`∃ c, n = d * c`). RLT's `x < y` is Robinson's Q8, ∃z. S(z) + x = y, which agrees with Lean's `<` on ℕ. A Lean `example : ¬ 2 ∣ 7 := by decide` takes one line; RLT's 128 steps exist so that each inference is one of 20 small, auditable rules and each axiom is printed.

| | |
|---|---|
| **You write** | nothing new: Lean files generated by `rlt lean` or written by hand |
| **RLT interprets** | each accepted goal as a Lean proposition over `Nat` |
| **The kernel checks** | (RLT) the RLT derivation; (Lean) the restated statement, independently |
| **Axioms and assumptions** | RLT: the registered axiom set. Lean: its standard axioms, gated |
| **Result** | agreement of two independent checks of the same statement |
| **Still open** | that the restatement is faithful is reviewed by reading both files; Lean's proof says nothing about RLT's derivation |

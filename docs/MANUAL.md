# RLT user manual — chapters 1–13

**Status.** RLT-004a, written 2026-11-11 against main at cc01968. Every terminal block in this manual is checked against the real program by `scripts/check-manual.py`; lines shown as `…` are omitted output. Chapter 12 (RLT-005, 2026-11-11) works through a published result; chapter 13 (RLT-006, 2026-11-12) checks a finite certificate from a research draft. Chapters on writing your own statement records and a reference are planned for a later milestone.

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

Run `rlt` from the repository root: it reads the axiom sets, definitions and conventions from there. From another folder, pass `--root PATH_TO_RLT` (for example `rlt --root ~/rlt check /tmp/x.rlt`).

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

The report is long because it carries everything a reader might need. For now, read these lines: `written goal` (what the file says), `checked goal` (what was proved), `axiom set` and `axioms used` (what it rests on), `assumptions` (Γ = ∅ means none), `steps`, and `verdict`. The `reading point` and `physical` lines belong to chapter 10; they record where the subject comes from and are never decided by a proof.

**Words used throughout.**
- The **kernel** is the small part of RLT that checks each step against its rule. Everything else (reading files, printing reports, generating proofs) sits outside it, and the kernel re-checks whatever those parts produce.
- **S** is the successor function: S(0) is 1, S(S(0)) is 2. Inside the kernel every numeral is written this way.
- An **axiom** is a statement accepted without proof; an **axiom set** is a named, fixed collection of them (chapter 4).
- **Γ** (gamma) is the list of assumptions a result still depends on; **Γ = ∅** means it depends on none.

The proof file is four steps. Steps are written in prefix form: `(= a b)` is a = b, `(+ 29 (S 0))` is 29 + S(0), and `(* 2 3)` is 2 × 3. Each step states a formula, then names a rule and what the rule uses:

```console
$ grep -A4 '(steps' tests/accept/R_math29_pairing.rlt
  (steps
    (s1 (= (+ 29 (S 0)) (S (+ 29 0)))  (inst add_succ (x 29) (y 0)))
    (s2 (= (+ 29 0) 29)                (inst add_zero (x 29)))
    (s3 (= (S (+ 29 0)) (S 29))        (cong S s2))
    (s4 (= (+ 29 (S 0)) (S 29))        (trans s1 s3))))
```

The four rules here: **inst** takes an axiom and fills in its variables (add_succ with x = 29, y = 0); **cong** applies the same function to both sides of an equation; **trans** chains a = b and b = c into a = c. Two more appear in chapter 5: **refl** states t = t, and **symm** turns a = b into b = a.

A proof file has these fields: `id`, `convention`, `goal` (the statement, in quotes), `mode` (`unconditional` or `conditional`), `axioms` (the axiom set), optional `assume` lines (chapter 5), and `steps`.

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

RLT separates **what you write** from **what is checked**. `rlt written` shows both for two arithmetic expressions (terms such as `29 + 1`; it reads them under NAT-DEC and takes no `=`, variables or definitions):

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

A **convention** says how a proof file's `goal` text becomes a formula. Three are used for proofs:

| Convention | Reads | Example |
|---|---|---|
| NAT-DEC | closed arithmetic: decimal numerals, `+`, `×`, `=` | `30 = 2 × 3 × 5` (× groups to the left: `(2 × 3) × 5`) |
| NAT-LOGIC | adds ∀, ∃, ∧, ∨, →, ¬, ⊥ and variables | `∀x. x + 0 = x` |
| NAT-DEF | adds the registered definitions `<`, `∣`, `free235` (chapter 3); ∨ groups to the left | `¬(2 ∣ 7)` |

**Typing the symbols.** Goal strings use the mathematical symbols: × (U+00D7), ∣ (U+2223, "divides", distinct from the ASCII bar `|`), ¬, ∀, ∃, →, ∧, ∨. ASCII stand-ins such as `*`, `|` or `~` are refused in goals as malformed. Proof *steps* use ASCII names instead: `*`, `dvd`, `not`, `and`, `or`, `imp`, `forall`, `exists`.

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

"ACCEPT relative to an axiom set" means: the formula holds in every system of numbers (every *structure*) that satisfies the axioms of that set; for ROBINSON-Q, these seven. The natural numbers ℕ satisfy all three sets, so an accepted formula is true of ℕ. This rests on the kernel's rules being sound, which docs/KERNEL-SOUNDNESS.md argues rule by rule. The axioms themselves are assumptions; RLT prints each one used.

Each proof names one axiom set and may use only its axioms: a step using mul_succ in a file that names PEANO-ADD is refused.

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

**Break one step and see the kernel refuse it.** Look at the generated steps (`head -12 /tmp/ten.rlt`), then change the first instance of `(y 6)` to `(y 5)`. The `sed` form below is GNU sed; on macOS, edit the file by hand:

```console
$ sed '0,/(y 6)/s//(y 5)/' /tmp/ten.rlt > /tmp/ten-bad.rlt && rlt check /tmp/ten-bad.rlt; echo "exit $?"
…
verdict         REJECT at step s1 — step claims `3 + 7 = S(3 + 6)`, rule `inst` yields `3 + 6 = S(3 + 5)`
…
exit 1
```

The message names the step, what it claimed and what the rule actually yields. A rejection is a fact about this derivation only; the goal's truth stays undetermined by it.

**Assumptions and Γ.** A step `(hyp h)` uses an assumption named h. The name must come from one of two places:
- an `(assume h "…")` line in the file, which declares h for the whole proof; or
- a later step that **discharges** h, i.e. turns "χ, assuming h" into a statement that no longer depends on h. imp-intro does this ("if h then χ"); or-cases and exists-elim do it for case analysis (chapter 6).

Otherwise the step is refused: `assumption `h` is undeclared in the goal`. The report's `assumptions` line shows what is left, written Γ. A proof in `unconditional` mode must end with Γ = ∅. A proof in `conditional` mode may keep declared assumptions, and the report keeps them visible:

```console
$ rlt check tests/accept/M1_conditional.rlt; echo "exit $?"
…
mode            conditional (Γ ⊆ declared assumptions)
…
proved          (2 × 3) + 1 = 8
assumptions     Γ = { h : 2 × 3 = 7 }
…
verdict         ACCEPT — conditional theorem relative to registered axiom set PEANO-ARITH
…
exit 0
```

That file assumes the false statement 2 × 3 = 7 on purpose: a conditional theorem says only "if the assumptions hold, so does the conclusion".

| | |
|---|---|
| **You write** | `10 = 3 + 7` (via `rlt gen`), then a hand edit |
| **RLT interprets** | NAT-DEC numerals |
| **The kernel checks** | each of 23 steps; in the edited file, step s1 fails |
| **Axioms and assumptions** | PEANO-ARITH; Γ = ∅ |
| **Result** | ACCEPT for the generated file; REJECT at step s1 for the edited one |
| **Still open** | for closed equations, only the axioms' own status as assumptions |

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

**Tracing dependencies: `rlt deps`.** Every checked theorem has an entry in the library (`lib/`), named by a stable id. `rlt deps` checks the theorem once, with the same function as `rlt check`, and prints its complete dependency graph. Each node shows its proof file, hash, assumptions, axioms and definitions:

```console
$ rlt deps dvd.free235.7; echo "exit $?"
RLT deps        dvd.free235.7
library         lib/ index valid (125 entries)
proof           tests/m3c/accept/CA05_free235_seven.rlt (M3c.CA05)
…
library entry   dvd.free235.7 · current
…
├─ T2  dvd.not.2_7  (M3c.CA02)  tests/m3c/accept/CA02_two_not_divides_seven.rlt
…
theorems        4 distinct · 3 cited-file checks
…
budget          cited-file checks 3 of 64 · citing-file chain 1 of 8
exit            0 — every node checked and accepted; the library index valid and the entry, if any, current
exit 0
```

A library id is bound to its statement's *fingerprint*: a hash of the convention, the axiom set and its content, the definitions, the checked formula and the assumptions. `rlt deps` prints the fingerprinted text. A changed proof file makes its entry stale (exit 3). A different statement can never take over an existing id, because `lib/identities.tsv` only grows. `rlt deps --verify-lib` checks every entry.

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

That line describes those records. E-T1 (chapter 9) is a later, separate theorem; linking it to the records as backing is a reviewed change that has not yet been made.

**One statement, four answers: `rlt explain`.** A *statement record* in `statements/` connects a written statement to its source, its proof and its assessments. `rlt explain` checks the record's proof with the same function as `rlt check`, compares every pinned hash and the checked statement with the current files, and prints the four outcomes separately:

```console
$ rlt explain statements/MOD30-CLASSIFICATION.yaml; echo "exit $?"
RLT explain     statements/MOD30-CLASSIFICATION.yaml
statement       MOD30-CLASSIFICATION
…
record          valid · current

1 KERNEL ACCEPTANCE: ACCEPT
…
2 SOURCE FIDELITY: registry verified · pinned
…
3 SEMANTIC CORRESPONDENCE: partial · assessment current
  assessed by   RLT-003 M3e review and report · 2026-11-09
…
4 PHYSICAL CORRESPONDENCE: not applicable
  reason        a mathematical statement about the numbers below 30
…
exit 0
```

Exit 0 means: the record is valid, its pins are current, and the checker accepts the proof. It asserts nothing about semantic or physical correspondence; those are reported, attributed, in their own sections. Exit 1 means the checker rejects the proof, 2 that the record is invalid or the proof malformed, 3 that a pin is stale. `--format json` gives the same report as JSON for scripts.

| | |
|---|---|
| **You write** | nothing new: these commands read records |
| **RLT interprets** | each quotation against its pinned source |
| **The kernel checks** | the attached relationships (29 + 1 = 30 here) |
| **Axioms and assumptions** | as each attached proof reports |
| **Result** | VALID records; ACCEPT for attached theorems; physical: unestablished |
| **Still open** | semantic correspondence of each meaning; physical evidence; statement records exist for four statements so far |

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

## 12. A published result: 8 is not matchable

N. McNew and C. Pomerance (*Matchable numbers*, arXiv:2604.05304v2, §1) call a positive integer n *matchable* if there is a coprime matching between {1, 2, …, τ(n)} and D(n), the set of divisors of n: a bijection pairing each element with a number coprime to it. They note that 1, …, 7 are matchable and that 8 is not, because the even divisors 2, 4 and 8 would all need odd partners, and {1, 2, 3, 4} has only two odd numbers.

RLT has no sets or bijections, so the statement names one partner per divisor: a for 1, b for 2, c for 4, d for 8. The theorem says that D(8) is {1, 2, 4, 8}, that the partner set is {1, 2, 3, 4}, and that no partners exist that are in range, pairwise distinct, and coprime to their divisors. Coprimality is written `cop(m, x)`: every positive common divisor k of m and x is 1.

The argument is five obligations, each a separate checked file in `tests/rlt005/accept/`. One is the paper's own step: a partner of 2 that is coprime to 2 is 1 or 3:

```console
$ rlt check tests/rlt005/accept/MO2_partner_of_2_is_odd.rlt
…
written goal    "∀x. ((0 < x ∧ x < 5) ∧ (∀k. ((0 < k ∧ k ∣ 2) ∧ k ∣ x) → k = 1)) → (x = 1 ∨ x = 3)"
…
steps           49
…
verdict         ACCEPT — unconditional theorem relative to registered axiom set ROBINSON-Q
…
```

The library entry `matchable.not.8` collects them. `rlt deps` shows the complete graph, checked in one run:

```console
$ rlt deps matchable.not.8; echo "exit $?"
…
library entry   matchable.not.8 · current
…
theorems        19 distinct · 29 cited-file checks
…
budget          cited-file checks 29 of 64 · citing-file chain 4 of 8
statement       statements/MATCHABLE-NOT-8.yaml (rlt explain: record current, exit 0)
exit            0 — every node checked and accepted; the library index valid and the entry, if any, current
exit 0
```

The statement record pins the paper itself. The source is an arXiv e-print fixed by its SHA-256 (status `archived`), and `scripts/check_excerpts.py` finds each quoted sentence verbatim in it:

```console
$ rlt explain statements/MATCHABLE-NOT-8.yaml; echo "exit $?"
…
1 KERNEL ACCEPTANCE: ACCEPT
…
2 SOURCE FIDELITY: registry verified · pinned
  source        MCNEW-POMERANCE-MATCHABLE-V2 (archived) @ 21674e5fb69c9a58b187bd51a4efc41330e6eee4d764c3f77882a29fa3aaef16
  location      https://arxiv.org/abs/2604.05304v2
…
3 SEMANTIC CORRESPONDENCE: partial · assessment current
…
4 PHYSICAL CORRESPONDENCE: not applicable
…
exit 0
```

The assessment is `partial`. The checked formula speaks of four partner variables. That this is exactly the paper's coprime matching over ℕ is a Lean theorem, not an RLT theorem: `explicit_iff` in `lean/RLT005Readings.lean` proves the translation for any four distinct positive divisors, and `matchable_one_to_seven` confirms that the Lean definition agrees with the paper on 1–7:

```console
$ cd lean && lean RLT005Readings.lean
…
'RLT005Readings.explicit_iff' depends on axioms: [propext, Classical.choice, Quot.sound]
…
'RLT005Readings.not_matchable_eight' depends on axioms: [propext, Classical.choice, Quot.sound]
…
'RLT005Readings.matchable_one_to_seven' depends on axioms: [propext, Classical.choice, Quot.sound]
…
```

**Why `0 < d` and `0 < k`.** Robinson's Q cannot prove 0 × x = 0 for a variable x, since that needs induction. So it cannot rule out 0 ∣ 8. The statement therefore speaks of positive divisors, which is what the paper means by the divisors of a positive integer. Over ℕ nothing changes (Lean `mem_divisors`, `cop_iff_gcd`).

| | |
|---|---|
| **You write** | partner variables, ranges, distinctness and `cop` for each divisor (`tests/rlt005/accept/`, generated by `scripts/gen-rlt005.py`) |
| **RLT interprets** | NAT-DEF: `<` and `∣` expanded to their definitions |
| **The kernel checks** | 19 files: the divisor and partner sets, the odd-partner lemmas, the pigeonhole step, the contradiction, the final theorem, and the refutation of cop(2, 2) |
| **Axioms and assumptions** | Robinson's Q (all seven axioms); Γ = ∅ |
| **Result** | ACCEPT — unconditional theorem; semantic correspondence partial (Lean bridge); physical not applicable |
| **Still open** | "nor is any subsequent multiple of 4" needs every n, hence induction |

## 13. A research certificate: the residues of H₄₅

Axiom Math's draft *A New Bound for Small Gaps Between Primes* (September 3, 2026) proves H₁ ≤ 212 using a 45-element tuple H₄₅ = {0, 2, 12, …, 212}. Its Lemma 12.1 says H₄₅ is *admissible*: for every prime p, the elements of H₄₅ miss at least one residue class mod p. The proof lists, for the 14 primes 2, 3, …, 43, one missed class each (1, 1, 3, …, 3), and argues that 45 elements cannot cover the p ≥ 47 classes of a larger prime.

RLT checks the 14 finite claims, one element at a time. The 630 element certificates are **generated, not stored**, so generate them first (under a second, then pinned by `tests/rlt006/manifest.sha256`; `scripts/verify.sh` does this before anything else):

```console
$ python3 -I scripts/gen-rlt006.py target/release/rlt --materialize-only
630 element certificates written to tests/rlt006/gen/
```

"H₄₅ misses the class 3 mod 43 at 212" is then the statement ¬(∃k. 212 = (43 × k) + 3), proved from Robinson's Q:

```console
$ rlt check tests/rlt006/gen/O_43_212.rlt
…
written goal    "¬(∃k. 212 = (43 × k) + 3)"
…
steps           1193
…
verdict         ACCEPT — unconditional theorem relative to registered axiom set ROBINSON-Q
…
```

There are 630 such certificates (14 primes × 45 elements); `sha256sum -c tests/rlt006/manifest.sha256` checks them against their pinned hashes.

Each prime has one theorem that cites its 45 certificates, each pinned by SHA-256. `rlt deps` checks all 46 files in one run and shows the citation budget:

```console
$ rlt deps admissible.h45.omits.43_3; echo "exit $?"
…
library entry   admissible.h45.omits.43_3 · current
…
theorems        46 distinct · 45 cited-file checks
…
budget          cited-file checks 45 of 64 · citing-file chain 1 of 8
…
exit 0
```

A missing or altered certificate is never accepted: the citation names the file and its hash, so `rlt check`, `rlt deps` and the library verifier all refuse (MALFORMED).

The paper is a PDF without TeX source, so its source pin is a **reviewed transcription** (`sources/transcriptions/AXIOM-BGP212-2026-09-03.yaml`). It records the page and section of each transcribed item, the text exactly as `pdftotext` 24.02.0 extracts it, and the SHA-256 of each page's extraction; `scripts/check_excerpts.py` re-checks it against the PDF. The generator reads the tuple and the table from that file.

```console
$ rlt explain statements/H45-OMITS-43.yaml; echo "exit $?"
…
1 KERNEL ACCEPTANCE: ACCEPT
…
2 SOURCE FIDELITY: registry verified · pinned
  source        AXIOM-BGP212-2026-09-03 (transcribed) @ 2b307ae26046dffd2153fe7466e5a9a302976dc3ba5142151a8a8104a37b3d80
  location      https://primegaps.axiommath.ai/bgp212.pdf
…
3 SEMANTIC CORRESPONDENCE: partial · assessment current
…
4 PHYSICAL CORRESPONDENCE: not applicable
…
exit 0
```

The assessment is `partial` because RLT proves one of the lemma's four obligations: the 14 omissions. The other three — 45 distinct elements with diameter 212, that 2, …, 43 are all the primes below 47, and the counting argument for p ≥ 47 — are proved in Lean (`lean/RLT006Direct.lean`, `lean/RLT006BGP212.lean`), not in RLT. `certificates/h45/` holds the computed residue of every element mod every listed prime and the occupied and omitted classes: computed evidence, which RLT has not proved.

| | |
|---|---|
| **You write** | nothing by hand: the generator derives every certificate from the reviewed transcription |
| **RLT interprets** | NAT-DEF; the statements use only `=`, `+` and `×` |
| **The kernel checks** | 630 element certificates and 14 per-prime theorems |
| **Axioms and assumptions** | Robinson's Q; Γ = ∅ |
| **Result** | ACCEPT — 14 unconditional theorems; semantic correspondence partial (obligation (b) of four); physical not applicable |
| **Still open in RLT** | the tuple's size and diameter, the list of primes below 47, and the large-prime argument |

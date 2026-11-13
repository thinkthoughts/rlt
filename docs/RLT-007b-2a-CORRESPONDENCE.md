# RLT-007b stage 2a — Correspondence review: the primes below 213

**Status:** semantic correspondence assessed, 2026-11-12. This is the basis document pinned by SHA-256 in the statement records `PRIMES-BELOW-213` and `PRIMES-RANGE-1` … `PRIMES-RANGE-6`.

## 1. What the source needs

**Source.** Axiom Math, *A New Bound for Small Gaps Between Primes*, preliminary draft of September 3, 2026, registered as `AXIOM-BGP212-2026-09-03` and read through the reviewed transcription `sources/transcriptions/AXIOM-BGP212-2026-09-03.yaml`.
- §2 (item `admissible-definition`) calls H admissible when, for every prime p, H omits some residue class mod p.
- The proof of Lemma 12.1 (item `omitted-primes`) lists an omitted residue for the 14 primes 2 … 43.

The words "every prime p" need a statement that names every prime that matters. RLT's plan (docs/RLT-007d-OPTION-B.md §6.1) treats each prime up to 212 by an explicit omitted residue, and every prime p > 212 at once: every element of H₄₅ lies in [0, 212] and 1 is not an element, so no element is ≡ 1 (mod p). That plan is RLT's own route; the paper handles the primes from 47 on by counting (transcription item `large-primes`). Stage 2a gives that classification. It is new mathematics supporting the lemma, not a statement printed in the paper.

## 2. The definition of prime: Prime_B

Stage 2a uses the bounded definition, approved on 2026-11-12 (decision B):

    Prime_B(p) :≡ 1 < p ∧ ∀d. ((1 < d ∧ d < p) → ¬(d ∣ p))

It is written inline with the existing definitions, whose meanings are unchanged:
- `x < y` is `∃z. S(z) + x = y`;
- `d ∣ n` is `∃k. n = d × k`.

No definition, axiom or rule is added.

**Reference definition G** (guarded positive divisors), kept for documentation:

    Prime_G(p) :≡ 1 < p ∧ ∀d. ((0 < d ∧ d ∣ p) → (d = 1 ∨ d = p))

**Correspondence with the standard definition is a Lean result about ℕ.** `lean/RLT007S2a.lean` proves:
- `pb_iff_std`: Prime_B(p) ↔ (2 ≤ p ∧ ∀d. d ∣ p → d = 1 ∨ d = p), for every natural number p;
- `pg_iff_std`: the same for Prime_G;
- `lt_iff` and `dv_iff`: the RLT readings of `<` and `∣` agree with Lean's on ℕ.

These are theorems about the standard natural numbers. RLT proves no equivalence between B, G and the standard definition under Robinson's Q, and this review claims none.

**Why the earlier unguarded definition was set aside.** The definition `1 < p ∧ ∀d. (d ∣ p → (d = 1 ∨ d = p))` cannot be proved for any prime in Robinson's Q. `lean/RLT007S2Design.lean` builds a structure, the naturals together with a copy ns(i) of the integers, with 0 × ns(i) = 7, and proves that it satisfies all seven axioms of ROBINSON-Q (theorem `model_of_Q`). There 0 × ns(0) = 7, so 0 "divides" 7 although 0 is neither 1 nor 7 (theorem `unbounded_fails_for_7`). By the soundness of the kernel's rules, no RLT proof under ROBINSON-Q concludes the unguarded definition for 7, and the same construction works for any prime numeral. Both remaining definitions exclude the divisor 0: Prime_G by 0 < d, Prime_B by 1 < d. In the same structure no nonstandard element divides 7 (theorem `guarded_ns_not_dvd_7`). Prime_B also quantifies only over d < p. For each numeral p, Q proves that every d < p is one of the numerals 0, …, p − 1 (`lt` puts d on the right of `+`, so `add_succ`, Q3, Q1 and Q2 bound it without induction). That enumeration of d is what stage 2b's positive primality proofs will carry out; stage 2a's E_I proofs enumerate p and refute each non-prime with the divisor witnesses of N_I. The design review (docs/RLT-007b-2-DESIGN.md §2) records the full argument.

## 3. The ranges and intervals

| Range | Interval | Primes |
|---|---|---|
| R1 | [0, 40] | 2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37 |
| R2 | [41, 78] | 41, 43, 47, 53, 59, 61, 67, 71, 73 |
| R3 | [79, 112] | 79, 83, 89, 97, 101, 103, 107, 109 |
| R4 | [113, 156] | 113, 127, 131, 137, 139, 149, 151 |
| R5 | [157, 190] | 157, 163, 167, 173, 179, 181 |
| R6 | [191, 212] | 191, 193, 197, 199, 211 |

The six intervals partition {0, …, 212}; together they list 47 primes.
- The generator recomputes every list by trial division and refuses any difference. Its `--self-test` refuses 8 altered range lists, including a valid but unapproved partition.
- Lean decides the lists independently in both directions: `e_1` … `e_6` (every Prime_B number of the interval is listed) and `listed_1` … `listed_6` (every listed number lies in the interval and satisfies Prime_B). These are Lean results over ℕ; RLT's positive primality theorems are stage 2b.
- `cli/tests/rlt007s2a.rs` rebuilds every stage-2a statement from the approved intervals, with the primes computed by trial division, and requires exact equality with the committed goals.
- The first 14 listed primes equal the 14 primes of item `omitted-primes`. This is a **source transcription check**, made by the generator.

## 4. What is established, and how

| Fact | Established by | Status label |
|---|---|---|
| ¬Prime_B(m) for every non-prime m in interval I, including 0 and 1 in interval 1 | `primes.nonprime.i1` … `i6` (N_1 … N_6) | **kernel accepted** (Robinson's Q) |
| Every p in interval I with Prime_B(p) is one of R_I's primes | `primes.exhaustive.r1` … `r6` (E_1 … E_6), records `PRIMES-RANGE-1` … `6` | **kernel accepted** |
| The order splits that join the intervals | `order.split.o1` … `o6` (O_1 … O_6) | **kernel accepted** |
| Every p with Prime_B(p) is a listed prime of its range, or 212 < p | `primes.below_213` (C), record `PRIMES-BELOW-213` | **kernel accepted** |
| The 14 transcribed primes are the first 14 listed | generator check | **source transcription checked** |
| Prime_B, read over ℕ, is the standard notion of prime | Lean `pb_iff_std` | **semantic correspondence assessed** (this review) |

## 5. The readings, one by one

1. **0 and 1.** ¬Prime_B(0) and ¬Prime_B(1) are proved from ¬(1 < 0) and ¬(1 < 1). They are conjuncts of N_1, and E_1 uses them in its branches for p = 0 and p = 1. So the classification covers 0 and 1, and places neither among the primes.
2. **Composites.** ¬Prime_B(m) is proved with an explicit divisor d = m / spf(m), where spf(m) is the smallest prime factor. The proof shows 1 < d, d < m and m = d × spf(m).
3. **E_I.** The hypothesis `(a−1 < p ∧ p < b+1)` (for I = 1, `p < 41`) says, over ℕ, a ≤ p ≤ b. The conclusion is a disjunction of equalities with the listed primes.
4. **O_I.** `p < 41 ∨ 40 < p`, and for I ≥ 2 `a−1 < p → (p < b+1 ∨ b < p)`, are the order facts that move from one interval to the next.
5. **C.** Its conclusion is R1(p) ∨ (R2(p) ∨ … (R6(p) ∨ 212 < p)), with R_I(p) the disjunction of E_I. Over ℕ, with `pb_iff_std`, it says that every prime below 213 is one of the 47 listed primes, in its approved range.

**Not claimed in stage 2a.** That each listed number satisfies Prime_B is stage 2b (47 positive primality theorems). The use of the classification in admissibility is the RLT-007d assembly. The three results are kept separate.

## 6. Lean comparison

`lean/RLT007S2a.lean` is generated and uses core Lean only. It states the RLT goals over ℕ with `LT`, `DV` and `PB` defined exactly as RLT's `lt`, `dvd` and Prime_B:
- `n_1` … `n_6`, `e_1` … `e_6`, `o_1` … `o_6` and `c`, one per RLT theorem;
- `listed_1` … `listed_6`, the converse of each `e_I`;
- the correspondence theorems `lt_iff`, `dv_iff`, `pb_iff_std` and `pg_iff_std`.

The axiom gate in verify.sh checks every declared theorem by name: each has its own `#print axioms` line and its own line in Lean's output, with standard axioms only (propext, Classical.choice, Quot.sound). 33 theorems are pinned for this file, and 20 for `lean/RLT007S2Design.lean`.

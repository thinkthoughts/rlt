# RLT-005 — Source correspondence: 8 is not matchable

Basis for the semantic assessment in `statements/MATCHABLE-NOT-8.yaml`. This file is pinned there by SHA-256; a revised assessment is a new file or a reviewed record update.

## 1. Source

| Item | Value |
|---|---|
| Paper | Nathan McNew and Carl Pomerance, *Matchable numbers*, arXiv:2604.05304 |
| Version | **v2**, submitted Sat, 23 May 2026 01:51:30 UTC (v1: Tue, 7 Apr 2026) |
| Location | §1 (Introduction), `match051426.tex` lines 79–96 |
| Archive | `https://export.arxiv.org/e-print/2604.05304v2`, gzip tar, SHA-256 `21674e5fb69c9a58b187bd51a4efc41330e6eee4d764c3f77882a29fa3aaef16` |
| Member | `match051426.tex`, SHA-256 `e7b6265c5e3b4e82bc12e14e71a2d84c10a24dd397dca62ae920a0102034260b` |
| Registry | `MCNEW-POMERANCE-MATCHABLE-V2`, status `archived`; `scripts/check_excerpts.py` verifies the archive hash and finds every excerpt verbatim in the member |

The source's definitions, verbatim (LaTeX):

- *Coprime matching* (line 79): "a bijection $\psi$ between them is said to be a coprime matching if for each $x$ in the domain of $\psi$, $x$ and $\psi(x)$ are coprime."
- *Divisors* (lines 90–91): "let $D(n)$ denote the set of divisors of $n$, and $\tau(n)=|D(n)|$." The sentence begins "for a positive integer $n$".
- *Matchable* (lines 91–92): "We say an integer $n$ is \textit{matchable} if there is a coprime matching between $\{1,2,\dots,\tau(n)\}$ and $D(n)$."
- *Claim* (line 94): "However, 8 is not matchable, nor is any subsequent multiple of 4."
- *Argument* (line 96): "Since even divisors must be mapped to odd numbers in a coprime matching, proper multiples of 4 are seen to be not matchable."

## 2. Formal statement (RLT, `matchable.not.8`)

RLT's language has numerals, `+`, `×`, `=`, `<` and `∣`, but no sets, functions or bijections. The matching is therefore stated with four partner variables, one per divisor: a for 1, b for 2, c for 4, d for 8. The theorem, under Robinson's Q with NAT-DEF and no assumptions, is:

```
(MD8 ∧ MP4) ∧ ¬∃a b c d. (ranges ∧ distinct) ∧ coprimality
MD8          ∀d. (0 < d ∧ d ∣ 8) ↔ d ∈ {1, 2, 4, 8}           (two implications)
MP4          ∀x. (0 < x ∧ x < 5) ↔ x ∈ {1, 2, 3, 4}           (two implications)
ranges       0 < a ∧ a < 5, and the same for b, c, d          every divisor receives a partner in {1..4}
distinct     ¬a=b ∧ ¬a=c ∧ ¬a=d ∧ ¬b=c ∧ ¬b=d ∧ ¬c=d          the four partners are pairwise distinct
coprimality  cop(1,a) ∧ cop(2,b) ∧ cop(4,c) ∧ cop(8,d)        each divisor is coprime to its partner
cop(m,x)     ∀k. ((0 < k ∧ k ∣ m) ∧ k ∣ x) → k = 1
```

## 3. Encoding choices and why

| Choice | Reason | Effect on meaning over ℕ |
|---|---|---|
| D(8) written as {1, 2, 4, 8}, justified by MD8 | No set vocabulary; MD8 proves the listed set is exactly the positive divisors of 8 | None: MD8 is part of the theorem |
| {1, …, τ(8)} written as {1, 2, 3, 4}, justified by MP4 | τ(8) = 4 is not expressible as a cardinality in RLT | None. The count 4 is fixed by MD8 listing four distinct numerals; Lean `tau_eight` checks τ(8) = 4 |
| `0 < d` in MD8 | Robinson's Q cannot refute `8 = 0 × j` for a variable j: it lacks `0 × x = 0`, which needs induction. So `0 ∣ 8` cannot be refuted in Q | None over ℕ (0 ∤ 8); matches the paper's "set of divisors" of a positive integer. Lean `mem_divisors` proves D(n) = {d : 0 < d ∧ d ∣ n} for n > 0 |
| `0 < k` in `cop` | Same obstruction: without it, cop(m,1) would need refuting `1 = 0 × j`, which Q cannot do, and even the all-ones assignment could not be shown to meet the coprimality conditions | None over ℕ: k ∣ m with m > 0 forces k > 0. Lean `cop_iff_gcd` proves cop(m,x) ↔ gcd(m,x) = 1 for m > 0 |
| Partner variables with ranges and distinctness, rather than a bijection | No function vocabulary | Lean `explicit_iff` proves the equivalence (§4) |

**Non-vacuity.** `matchable.not.8.nonvacuous` (RLT) and `nonvacuous` (Lean) show that the ranges and coprimality conditions are met by a = b = c = d = 1. So the theorem fails only because of distinctness, not because some condition is unsatisfiable. Lean `six_explicit` and `matchable_six` show that the same encoding holds for D(6) = {1, 2, 3, 6}, which the paper says is matchable.

## 4. Lean comparisons (core Lean v4.34.0-rc2, standard axioms only)

**Comparison 1 — `lean/RLT005Direct.lean`.** `RLT005Direct.matchable_not_8` is RLT's written goal, symbol for symbol, with `<` and `∣` read as Lean's on ℕ. Lean proves it independently. It reduces `cop` to gcd and decides the 4⁴ partner cases. `lt_iff_rlt` and `dvd_iff_rlt` prove that RLT's NAT-DEF unfoldings of `<` and `∣` agree with Lean's on ℕ. *Claim:* the proposition RLT accepted is true in ℕ, by a separate proof. *Not claimed:* that Lean checked RLT's derivation.

**Comparison 2 — `lean/RLT005Readings.lean`.** This file defines the paper's notions:

- `divisors n`, the divisors of n among 1..n;
- `tau n`, the length of `divisors n`;
- `CoprimeMatching A B f`, meaning f maps A into B, is injective on A, is onto B, and x is coprime to f(x) for every x in A;
- `matchable n`, meaning some f is a coprime matching from {1..τ(n)} onto D(n).

It then proves:

| Theorem | What it establishes |
|---|---|
| `mem_divisors` | D(n) as defined is the set of positive divisors, for n > 0 |
| `matching_symm`, `matchable_iff` | The direction of ψ is immaterial: the paper says "between", and the inverse of a coprime matching is one |
| `explicit_iff` | For any four distinct positive p, q, s, t: a coprime matching [p, q, s, t] → {1, 2, 3, 4} exists ⇔ RLT's partner conditions with `cop` have a solution |
| `matchable'_eight_iff` | The paper's definition at n = 8 is RLT's formula |
| `not_matchable_eight` | ¬ matchable 8 in the paper's formulation |
| `matchable_one_to_seven` | 1, …, 7 are matchable under this definition, as the paper says (line 94) — a check that the definition is not vacuously false |

The two files share their definitions of `cop`, `r` and `M` verbatim. `scripts/verify.sh` refuses any difference between the two copies.

## 5. Assessment: **partial**

**Matches.**

1. The four RLT conditions — every divisor of 8 receives a partner in {1..4}, the partners are pairwise distinct, and each divisor is coprime to its partner — are equivalent over ℕ to a coprime matching between D(8) and {1, …, τ(8)} in the paper's sense. This is Lean `explicit_iff` and `matchable'_eight_iff`, with the direction immaterial by `matchable_iff`.
2. MD8 and MP4 are proved inside the RLT theorem, so the divisor and partner sets are not assumed.
3. The RLT proof follows the paper's argument (line 96). The lemmas `matchable.odd_partner.2/4/8` prove that the partners of the even divisors 2, 4 and 8 lie in {1, 3}. `matchable.pigeonhole.1_3` proves that three pairwise distinct values cannot lie in {1, 3}.

**Differs.**

1. The identification of "matchable" with RLT's explicit formula is proved in Lean over ℕ (comparison 2), not in RLT under Q. RLT has no vocabulary for sets, bijections or cardinality, so RLT's theorem is the explicit formula, and "8 is not matchable" follows from it together with that Lean bridge.
2. RLT's statement holds in every model of Q. Its link to the paper's notions is checked only in the standard model ℕ.
3. Only the first claim of line 94 is covered. "Nor is any subsequent multiple of 4" quantifies over all n and needs induction. It is outside this target.

**Physical correspondence:** not applicable (a statement in number theory).

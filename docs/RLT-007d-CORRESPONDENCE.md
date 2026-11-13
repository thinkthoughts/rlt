# RLT-007d — Correspondence review: Lemma 12.1 as a six-statement composition

**Status:** semantic correspondence assessed, 2026-11-13, for the result of `rlt compose compose/h45.yaml` (approved design `docs/RLT-007d-FINAL.md`, decisions D-1 to D-4).

## 1. The source claim

Axiom Math, *A New Bound for Small Gaps Between Primes*, preliminary draft of September 3, 2026 (`AXIOM-BGP212-2026-09-03`), read through the reviewed transcription `sources/transcriptions/AXIOM-BGP212-2026-09-03.yaml`:

- **Lemma 12.1** (item `lemma-12-1`): "The set H₄₅ is an admissible 45-tuple of diameter 212."
- **H₄₅** (item `h45-tuple`): {0, 2, 12, 14, 24, 26, 30, 36, 44, 50, 54, 56, 60, 66, 72, 74, 80, 84, 92, 96, 102, 110, 114, 116, 122, 126, 134, 140, 144, 150, 156, 162, 164, 170, 176, 180, 182, 186, 192, 194, 200, 204, 206, 210, 212}.
- **Admissible** (item `admissible-definition`, §2): for every prime p, the elements omit at least one residue class modulo p.
- **The paper's proof** (items `omitted-primes`, `omitted-classes`, `large-primes`): explicit omitted classes for the 14 primes up to 43, and a counting argument for p ≥ 47.

## 2. The six statements

All are closed formulas of first-order arithmetic under the axiom set ROBINSON-Q and convention NAT-DEF, with the definitions of `spec/definitions.rlt` (SHA-256 `122e37fc…`) expanded before the kernel:

- a < b  :≡  ∃z. S(z) + a = b
- d ∣ n  :≡  ∃k. n = d × k

Write h₀, …, h₄₄ for the 45 listed numerals, in the listed order, and:

- OM(p, r)  :≡  ⋀ᵢ ¬(∃k. hᵢ = (p × k) + r), left-nested over i = 0 … 44;
- Adm(p)  :≡  ∃r. (r < p ∧ OM(p, r));
- Prime_B(p)  :≡  1 < p ∧ ∀d. ((1 < d ∧ d < p) → ¬(d ∣ p)).

| # | Node | Statement | Label |
|---|---|---|---|
| 1 | `distinct.g0` | ⋀_{i=0…10} ⋀_{j>i} ¬(hᵢ = hⱼ) | kernel accepted |
| 2 | `distinct.g1` | ⋀_{i=11…21} ⋀_{j>i} ¬(hᵢ = hⱼ) | kernel accepted |
| 3 | `distinct.g2` | ⋀_{i=22…32} ⋀_{j>i} ¬(hᵢ = hⱼ) | kernel accepted |
| 4 | `distinct.g3` | ⋀_{i=33…43} ⋀_{j>i} ¬(hᵢ = hⱼ) | kernel accepted |
| 5 | `bounds` | ⋀ᵢ ((∃z. z + 0 = hᵢ) ∧ (∃z. z + hᵢ = 212)) | kernel accepted |
| 6 | `adm.all` | ∀p. (Prime_B(p) → Adm(p)) | discharged by composition (layer 2) |

The exact formulas, symbol for symbol, are the goals of the registered specification probes (`compose/approved-specifications.tsv` → `tests/rlt007d/spec/SPEC_*.rlt`) and the kernel's printed forms in the `rlt compose` report. Statements 1–4 together are the 990 disequalities among the 45 numerals.

## 3. Reading the six statements as Lemma 12.1

| Content of "an admissible 45-tuple of diameter 212" | Established by | Label |
|---|---|---|
| The 45 listed numbers are pairwise distinct | statements 1–4 | kernel accepted |
| Each listed number lies between 0 and 212 | statement 5 | kernel accepted |
| For every prime p, the 45 numbers omit some class modulo p | statement 6 | discharged by composition |
| The list has exactly 45 entries, in the transcribed order; 0 and 212 are listed; the statements are about exactly that list | the statements' text, rebuilt from the transcription by `scripts/compose-check.py` and by `cli/tests/rlt007d.rs` on every run | source transcription checked |
| "45 distinct numbers" is a 45-tuple; "between 0 and 212, with 0 and 212 listed" is diameter 212; Prime_B over ℕ is the paper's "prime"; ¬(∃k. h = p·k + r) with r < p is "h is not ≡ r (mod p)" | this review; Lean `prime_iff`, `om_iff`, `distinct_iff`, `bounds_le`, `lemma_12_1` | semantic correspondence assessed |

**Reading over ℕ.**
- *Primes.* Over ℕ, Prime_B(p) holds exactly when p ≥ 2 and only 1 and p divide p (Lean `prime_iff`, against `IsPrime p :≡ 2 ≤ p ∧ ∀m. m ∣ p → m = 1 ∨ m = p`).
- *Residues.* With 0 ≤ r < p and h ≥ 0, ∃k ∈ ℕ. h = p·k + r says h mod p = r; Lean `om_iff` and `lemma_12_1` use exactly this. The paper works over the integers; for h ≥ 0 any integer k with h = p·k + r and 0 ≤ r < p is nonnegative, so the two readings agree.
- *Diameter.* max − min = 212 − 0 = 212, with 0 and 212 among the listed numbers (transcription) and every listed number within [0, 212] (statement 5).
- *Distinctness and cardinality.* Statements 1–4 give the 990 disequalities; 45 is the length of the transcribed list, which fixes the statements' text. Lean `distinct_iff` decides that statements 1–4 together are equivalent to `H45.Nodup`.

**Assessment: complete.** Read over ℕ, the six statements and the transcribed list state Lemma 12.1: H₄₅ is a set of 45 distinct integers of diameter 212 that omits a residue class modulo every prime.

## 4. The route, compared with the paper's proof

| Primes | The paper | RLT |
|---|---|---|
| 2 … 43 | the 14 listed classes | the same 14 classes (RLT-006), kernel accepted per prime |
| 47 … 211 | counting: 45 elements cannot cover p ≥ 47 classes | explicit least omitted class per prime (stage 4), kernel accepted per prime |
| p > 212 | the same counting argument | the class 1 for every p > 212 at once (stage 3), kernel accepted |
| "every prime is one of these" | implicit | every Prime_B number is one of the 47 primes below 213 or exceeds 212 (stage 2a), kernel accepted |

The composition then joins them: each range assembly turns the omissions OM(q, r_q) for the primes of one interval into ∀p. (p ∈ range → Adm(p)), proving r_q < q inside the assembly; the tail assembly does the same for p > 212 with r = 1; and `adm.all` combines the classification with the seven results. RLT's route reaches the paper's conclusion by explicit classes instead of the counting argument.

## 5. What the composition establishes, and what it does not

- **Established:** each of the six statements holds in every model of Robinson's Q (statements 1–5 as kernel theorems, statement 6 by the composition argument of `docs/RLT-007d-REPORT.md` §3), so in particular in ℕ.
- **Not a single kernel theorem:** statement 6 rests on eight conditional kernel theorems (the assemblies) and 49 unconditional ones (the 47 omissions, the tail and the classification), joined by the composition checker; statements 1–5 are separate unconditional kernel theorems. No single proof object derives it; `rlt compose` reports it as *discharged by composition* and never as an unconditional kernel theorem.
- **Composed results are not library entries.** They are reported by `rlt compose` only and cannot be cited by later proofs; a later proof that needs one cites its components.

## 6. Lean comparison

`lean/RLT007D.lean`, generated, core Lean only, independent of RLT's checker:
- `C_…`: each of the 62 node statements as RLT writes it, read with Lean's `<` and `∣` over ℕ (`scripts/verify.sh` checks the text symbol for symbol against the registered probes);
- `range_1` … `range_6`, `adm_tail`, `adm_all`: the same layer-1 and layer-2 steps, from the same hypotheses;
- `rlt_lt_iff`, `rlt_dvd_iff`: RLT's `<` (∃z. S(z) + x = y) and `∣` (∃k. n = d × k) are Lean's `<` and `∣` on ℕ;
- `prime_iff`, `om_iff`, `distinct_iff`, `bounds_le`: the readings of §3;
- `lemma_12_1`: from the six statements, H₄₅ has no repeated entry, has 45 entries, lies in [0, 212], contains 0 and 212, and omits a class modulo every prime;
- `components`, `lemma_12_1_holds`: Lean decides every layer-0 statement itself and concludes Lemma 12.1 outright.

17 theorems, each listed by `#print axioms` with the standard axioms only (`propext`, `Classical.choice`, `Quot.sound`).

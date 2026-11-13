# RLT-007b stage 3 — Correspondence review: the primes p ≥ 213

**Status:** semantic correspondence assessed, 2026-11-12. This is the basis document pinned by SHA-256 in the statement record `H45-TAIL-OMITS`.

## 1. The source claim and the role of this stage

**Source.** Axiom Math, *A New Bound for Small Gaps Between Primes*, preliminary draft of September 3, 2026, registered as `AXIOM-BGP212-2026-09-03`, read through the reviewed transcription `sources/transcriptions/AXIOM-BGP212-2026-09-03.yaml`:
- §2 (item `admissible-definition`): H is admissible when, for every prime p, the elements of H omit at least one residue class modulo p;
- §12 (items `h45-tuple`, `lemma-12-1`): H₄₅ = {0, 2, 12, …, 210, 212}, and Lemma 12.1 says H₄₅ is admissible;
- the proof's last step (item `large-primes`): "If p ≥ 47, its 45 elements cannot cover all p residue classes."

**RLT's route.** RLT covers "every prime p" in four parts (docs/RLT-007d-OPTION-B.md §6.1):
1. the 14 primes up to 43, by the paper's own classes (RLT-006);
2. the primes 47 to 211, each by an explicit class (stage 4);
3. every p > 212, by the single class 1 (this stage);
4. the classification `primes.below_213`, which says every Prime_B number is one of the 47 primes below 213 or exceeds 212 (stage 2a).

The paper's counting argument for p ≥ 47 needs a pigeonhole principle over an arbitrary modulus. RLT's route replaces it with explicit classes, and for p > 212 one class serves every modulus. This is RLT's own route to the same conclusion, not a transcription of the paper's proof.

## 2. The statement

    TAIL:  ∀p. (212 < p → ⋀_{h ∈ H₄₅} ¬(∃k. h = (p × k) + 1))

- `212 < p` is the existing definition `∃z. S(z) + 212 = p`; no definition, axiom or rule is added.
- The conjunction is left-nested over the 45 listed elements in the transcribed order, the same shape as RLT-006's per-prime statements OM_p with r = 1 and the modulus a variable.
- k ranges over all natural numbers, with no restriction, and the statement carries no positivity hypothesis. Neither is needed: k = 0 gives h = 1, which is not listed, and k ≥ 1 gives h ≥ p + 1 ≥ 214, above every listed element.

**Scope: every p > 212, prime or not.** The statement is arithmetic. It applies to the primes p ≥ 213 by instantiation; nothing in it uses primality. This is exactly the shape the assembly needs: after `primes.below_213` gives 212 < p for a prime outside the 47, TAIL at that p gives the omitted class.

**What the assembly adds.** A residue class r modulo p is meaningful for r < p. That 1 < p follows from 212 < p by a closed computation, and RLT-007d's assembly states it there. It is not part of TAIL.

**Why 212.** The least bound for which the statement is true is 211: for p = 211, the element 212 = 211 × 1 + 1 (Lean `bound_210_fails` refutes the statement with 210 < p). The bound 212 is the one the classification produces (`212 < p` in `primes.below_213`), so TAIL matches it exactly. Proving the slightly stronger 211 < p would add nothing the assembly uses.

## 3. Reading over ℕ

H₄₅ consists of nonnegative integers. For p > 1:
- "h lies in the class 1 modulo p" in the integers means h = p·k + 1 for some integer k;
- for h ≥ 0 and p > 1, any such k is nonnegative, so the natural-number form ∃k ∈ ℕ. h = p·k + 1 is the same condition;
- Lean `one_mod_iff` proves (∃k. h = p·k + 1) ↔ h % p = 1 for every p > 1.

So TAIL says: modulo every p > 212, no element of H₄₅ is in the class 1, and Lean `tail_mod` states that form directly (h % p ≠ 1 for every listed h). This is the same reading RLT-006 uses for its classes (docs/RLT-006-CORRESPONDENCE.md §3).

## 4. What is established, and how

| Fact | Established by | Status label |
|---|---|---|
| TAIL under Robinson's Q, unconditional | `tests/rlt007s3/accept/TAIL.rlt`, library id `admissible.h45.omits.tail_1` | **kernel accepted** |
| The 45 elements are the transcribed tuple, increasing from 0 to 212, without 1 | generator check against item `h45-tuple` | **source transcription checked** |
| TAIL over ℕ, literally; its residue reading; the bound 210 fails | Lean `tail`, `one_mod_iff`, `tail_mod`, `bound_210_fails` | Lean-checked |
| TAIL is the "every prime p ≥ 213" case of Lemma 12.1's admissibility, with the class 1 | this review | **semantic correspondence assessed** (`partial`) |

**Assessment: partial.** TAIL settles admissibility for the primes p ≥ 213. The primes 47 to 211 (stage 4) and the assembly with the small primes, the classification and the distinctness and bound statements (RLT-007d) remain. Together these are the components of the full lemma; this stage supplies one of them.

## 5. Lean comparison

`lean/RLT007S3.lean`, generated, core Lean only:
- `tail`: RLT's statement, with `LT` defined exactly as RLT's `<`;
- `key`: the single-element argument (k = 0 or k ≥ 1);
- `one_mod_iff` and `tail_mod`: the residue reading;
- `bound_210_fails`: the bound cannot be lowered to 210;
- `h45_eq`, `small`, `lt_iff`: the list's shape and the reading of `<`.

All are printed and gated by name in verify.sh, with standard axioms only.

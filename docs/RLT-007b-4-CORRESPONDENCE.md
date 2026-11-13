# RLT-007b stage 4 — Correspondence review: the primes 47 to 211

**Status:** semantic correspondence assessed, 2026-11-13. This is the basis document pinned by SHA-256 in the 33 statement records `OMITS-H45-047` … `OMITS-H45-211`.

## 1. The source claim and the role of this stage

**Source.** Axiom Math, *A New Bound for Small Gaps Between Primes*, preliminary draft of September 3, 2026, registered as `AXIOM-BGP212-2026-09-03`, read through the reviewed transcription `sources/transcriptions/AXIOM-BGP212-2026-09-03.yaml`:
- §2 (item `admissible-definition`): H is admissible when, for every prime p, the elements of H omit at least one residue class modulo p;
- §12 (item `h45-tuple`): the tuple H₄₅;
- the proof of Lemma 12.1 (item `large-primes`): "If p ≥ 47, its 45 elements cannot cover all p residue classes."

**RLT's route for p ≥ 47.** The paper's step is a counting (pigeonhole) argument over an arbitrary modulus. RLT replaces it by explicit classes:
- for each of the 33 primes 47 … 211, an explicit omitted class r_p (this stage);
- for every p > 212, the class 1 (stage 3, merged at 4e4ca04).

Together with RLT-006 (the 14 primes up to 43, the paper's own classes) and stage 2a (every Prime_B number is one of the 47 primes below 213, or exceeds 212), these are the components the assembly (RLT-007d) combines. This stage supplies the 33 middle components.

## 2. The classes

The paper lists no class for p ≥ 47. RLT uses, for each prime, the least class that H₄₅ omits; this is the choice measured in RLT-007c and approved for stage 4:

| r_p | primes |
|---|---|
| 1 | 47, 61, 67, 89, 97, 103, 107, 127, 131, 137, 151, 157, 167, 173, 197 |
| 3 | 73, 79, 83, 101, 109, 139, 149, 163, 181, 193, 199, 211 |
| 4 | 71, 113, 179, 191 |
| 6 | 53, 59 |

The generator recomputes each class (it must be below p, omitted by every listed element, and the least such class) and refuses any difference; Lean `table_facts` decides the same three facts, and that the 33 primes are exactly the primes 47 … 211.

## 3. The statements

For each prime p with class r = r_p:

    OM_p:  ⋀_{h ∈ H₄₅} ¬(∃k. h = (p × k) + r)

- left-nested over the 45 transcribed elements in the listed order, exactly the shape of RLT-006's 14 per-prime statements;
- each conjunct is a separate kernel-checked certificate D_p_h, cited by OM_p (45 of 64 cited-file checks, chain 1);
- the statements use no definition (`<` and `∣` do not occur).

**Reading over ℕ.** With 0 ≤ r < p and h ≥ 0, ∃k ∈ ℕ. h = p·k + r says h ≡ r (mod p), and the integer form agrees (for h ≥ 0 any such integer k is nonnegative). Lean `omit_iff` proves ¬(∃k. h = p·k + r) ↔ h % p ≠ r for r < p. So OM_p says: the reductions of H₄₅ modulo p omit the class r_p. That r_p < p is part of the decided table; the assembly uses it as the witness for Adm(p).

**Scope.** Each statement is about one numeral p. That p is prime is not part of OM_p; it enters the assembly through stage 2a's listing. Lean `below_213_covered` decides that every prime below 213 is either in RLT-006's transcribed table or in this one.

## 4. What is established, and how

| Fact | Established by | Status label |
|---|---|---|
| The 1,485 element certificates and the 33 summaries OM_p, under Robinson's Q | `tests/rlt007s4`, library `admissible.h45.omits.P_R` | **kernel accepted** |
| The tuple is the transcribed H₄₅ | generator check against item `h45-tuple` | **source transcription checked** |
| The 33 primes and their classes (least omitted class) | generator check; Lean `table_facts` | computed, Lean-checked (RLT's choice, not a transcription) |
| OM_p is "H₄₅ omits r_p mod p", the case p of Lemma 12.1's admissibility | this review; Lean `omit_iff`, `om_P` | **semantic correspondence assessed** (`partial`) |

**Assessment: partial.** Each record settles admissibility at one prime p from 47 to 211. With RLT-006, stage 3 and stage 2a's classification, the components for every prime are in place; their composition into Lemma 12.1 is RLT-007d, which remains design only.

## 5. Lean comparison

`lean/RLT007S4.lean`, generated, core Lean only:
- `om_47` … `om_211`: RLT's 33 statements, symbol for symbol;
- `omit_iff`, `element`: the residue reading;
- `table_facts`: the primes and the classes, decided;
- `below_213_covered`: every prime below 213 has a listed class in RLT-006's table or this one.

All are printed and gated by name in verify.sh, with standard axioms only.

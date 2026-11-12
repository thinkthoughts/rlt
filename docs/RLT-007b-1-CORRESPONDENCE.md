# RLT-007b stage 1 — Correspondence review: obligation (a) of Lemma 12.1

**Status:** semantic correspondence assessed, 2026-11-12. This is the basis document pinned by SHA-256 in the statement records `H45-DISTINCT-G0` … `G3` and `H45-BOUNDS`.

## 1. The source claim

**Source.** Axiom Math, *A New Bound for Small Gaps Between Primes*, preliminary draft of September 3, 2026, registered as `AXIOM-BGP212-2026-09-03`. The PDF is pinned by SHA-256 (`2b307ae2…3b3d80`). The text is read through the reviewed transcription `sources/transcriptions/AXIOM-BGP212-2026-09-03.yaml`, items `h45-tuple` and `lemma-12-1`, both on PDF page 43, §12:

> H₄₅ = {0, 2, 12, 14, …, 206, 210, 212}.
> Lemma 12.1. The set H₄₅ is an admissible 45-tuple of diameter 212.

**Obligation (a)** is the part of the lemma that concerns the tuple itself:
- **"45-tuple":** 45 distinct integers. §2 (PDF page 3, transcription item `admissible-definition`) defines an admissible set as "a finite set H = {h₁, …, h_k} of distinct integers", and a k-tuple is such a set with k elements.
- **"diameter 212":** max H₄₅ − min H₄₅ = 212. The paper uses "diameter" without defining it; this review adopts the standard reading for a finite set of integers, largest element minus smallest element.

Admissibility for every prime is obligations (b)–(d), handled in RLT-006 and in later stages.

## 2. What establishes obligation (a), and how

| Fact | Established by | Status label |
|---|---|---|
| The 990 disequalities hᵢ ≠ hⱼ (i < j) among the listed numerals | 44 per-element theorems `RLT007S1.D_HHH`, grouped into `admissible.h45.distinct.g0` … `g3` | **kernel accepted** (Robinson's Q) |
| 0 ≤ h ≤ 212 for every listed h, written as ∃z. z + 0 = h and ∃z. z + h = 212 | `admissible.h45.bounds` | **kernel accepted** (Robinson's Q) |
| The tuple has exactly 45 entries | generator check against the transcription | **source transcription checked** |
| The entries are strictly increasing | generator check against the transcription | **source transcription checked** |
| The first entry is 0, and the last is 212 | generator check against the transcription | **source transcription checked** |
| The transcribed lemma names a "45-tuple" of "diameter 212", as whole numbers, in both the PDF-checked `extracted` field and the reviewed `text` field | generator check against the transcription | **source transcription checked** |
| The transcription equals the pinned PDF's text and numbers | `scripts/check_excerpts.py`: the PDF hash, the tool version, the page extraction hashes and the numbers | **source transcription checked** |

The source transcription checks run in `scripts/gen-rlt007s1.py`, which refuses to generate anything when one fails, and in verify.sh. The generator's `--self-test` refuses 11 altered transcriptions:
- a duplicate;
- an out-of-order entry;
- 213;
- 44 entries;
- 46 entries;
- 46 strictly increasing entries from 0 to 212, a case only the length check catches;
- a first entry of 1;
- a negative entry;
- a lemma text naming a 44-tuple;
- an extracted lemma text naming a 145-tuple;
- a lemma text naming diameter 2120.

## 3. The readings, checked one by one

1. **¬(a = b) for numerals a, b.** Over ℕ it says the two listed integers differ. The 44 per-element theorems together state it for every pair i < j of positions in the listed order. Each theorem conjoins the disequalities of one element with every later element, in that order.
2. **∃z. z + 0 = h and ∃z. z + h = 212.**
   - Over ℕ, ∃z. z + x = y holds exactly when x ≤ y.
   - The first conjunct is therefore 0 ≤ h, true for every natural number. It is stated so that both endpoints of the diameter are covered in the same form.
   - The second conjunct is h ≤ 212.
   - No `le` definition exists in `spec/definitions.rlt`, and this stage adds none.
3. **Diameter 212.** The minimum of the listed set is 0 and its maximum is 212. This holds because both are listed (a transcription check) and every element lies between them (`admissible.h45.bounds`). So max − min = 212.
4. **"45-tuple".** Distinct entries (the kernel-accepted statements above) and exactly 45 of them (the transcription check) together make a set of 45 distinct integers.

## 4. Lean comparison

`lean/RLT007S1.lean` is generated and uses core Lean only.
- **`dg_0` … `dg_3` and `bounds`:** the five RLT goals, stated literally over ℕ and proved by `decide` or by explicit witnesses.
- **`obligation_a`:** decides, in Lean's own vocabulary, that the transcribed list has length 45, no duplicates, first entry 0, last entry 212, and every entry in [0, 212].

Lean reports that none of the 50 theorems depends on any axiom. The axiom gate in verify.sh enforces that all 50 are printed, and that no axiom outside the standard three (propext, Classical.choice, Quot.sound) occurs.

## 5. Assessment

- **status:** `partial`. Obligation (a) is fully covered: by the kernel-accepted statements together with the source transcription checks above. Lemma 12.1 as a whole also needs obligations (b)–(d).
- **physical applicability:** not applicable (number theory).

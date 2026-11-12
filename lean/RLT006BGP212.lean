/-
RLT-006, Lean comparison 2 of 2: RLT's statements against the user's
existing Lean formalization thinkthoughts/lean-bgp212 (commit
7898eb4665197ab418a62b9bb657dab3f31bb262, MIT licence, (c) 2026 thinkthoughts).
Core Lean only. Run: cd lean && lean RLT006BGP212.lean

Two blocks are copied verbatim, so that this file checks on its own; the
`module`, `public import` and `@[expose] public section` lines around them
are omitted. scripts/verify.sh compares each block byte for byte with the
pinned upstream lines when a clone is present (CI clones it):
  * BGP212/Basic.lean lines 25-47: IsPrime, Admissible, H45, omittedTable;
  * Solution/Lemma12_1.lean lines 20-64: h45_card, h45_diameter,
    h45_omitted_table, h45_omits_below_213, h45_omits_from_213,
    h45_admissible, lemma_12_1.

What the bridge below proves, in Lean (none of it is an RLT derivation):
  * same_tuple, same_table: lean-bgp212's H45 and omittedTable are RLT-006's
    reviewed transcription (the literals rltH45 and rltTable, which
    verify.sh compares with lean/RLT006Direct.lean, generated from the
    transcription);
  * omits_rlt_form: every table row gives RLT's element statements
    ¬(∃ k, h = p * k + r) for all 45 elements: lean-bgp212's
    h45_omitted_table implies the content of RLT's 14 per-prime theorems;
  * lean-bgp212 proves admissibility for every prime by a route that
    differs from the paper's (decide below 213; class 1 for p ≥ 213);
    RLT006Direct.lean follows the paper's counting argument instead.
-/

-- BEGIN VENDORED lean-bgp212 BGP212/Basic.lean 25-47
namespace BGP212

/-- A natural number `p` is prime: `2 ≤ p` and its only divisors are `1` and `p`. -/
def IsPrime (p : Nat) : Prop :=
  2 ≤ p ∧ ∀ d : Nat, d ∣ p → d = 1 ∨ d = p

/-- §2: `H` omits at least one residue class modulo every prime `p`. -/
def Admissible (H : List Nat) : Prop :=
  ∀ p : Nat, IsPrime p → ∃ r : Nat, r < p ∧ ∀ h ∈ H, h % p ≠ r

/-- §12: the admissible 45-tuple of diameter 212, transcribed in the displayed order. -/
def H45 : List Nat :=
  [0, 2, 12, 14, 24, 26, 30, 36, 44, 50, 54, 56, 60, 66, 72, 74, 80, 84, 92, 96, 102,
   110, 114, 116, 122, 126, 134, 140, 144, 150, 156, 162, 164, 170, 176, 180, 182, 186,
   192, 194, 200, 204, 206, 210, 212]

/-- Lemma 12.1 proof: pairs `(p, r)` where `H₄₅` omits the class `r` mod `p`,
transcribed from the source lists "2, 3, 5, …, 43" and "1, 1, 3, …, 3". -/
def omittedTable : List (Nat × Nat) :=
  [(2, 1), (3, 1), (5, 3), (7, 6), (11, 9), (13, 3), (17, 1), (19, 13), (23, 17),
   (29, 4), (31, 11), (37, 4), (41, 4), (43, 3)]

end BGP212
-- END VENDORED lean-bgp212 BGP212/Basic.lean 25-47

-- BEGIN VENDORED lean-bgp212 Solution/Lemma12_1.lean 20-64
namespace BGP212.Challenge

open BGP212

theorem h45_card : H45.length = 45 ∧ H45.Nodup := by
  decide +kernel

theorem h45_diameter : 0 ∈ H45 ∧ 212 ∈ H45 ∧ ∀ h ∈ H45, h ≤ 212 := by
  decide +kernel

theorem h45_omitted_table :
    omittedTable.map Prod.fst = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43] ∧
    ∀ q ∈ omittedTable, q.2 < q.1 ∧ ∀ h ∈ H45, h % q.1 ≠ q.2 := by
  decide +kernel

/-- Finite part: every modulus `2 ≤ p < 213` (prime or composite) has an omitted class. -/
theorem h45_omits_below_213 :
    ∀ p, p < 213 → 2 ≤ p → ∃ r, r < p ∧ ∀ h ∈ H45, h % p ≠ r := by
  decide +kernel

/-- Infinite part: for `p ≥ 213`, reduction mod `p` fixes every element, and `1 ∉ H₄₅`. -/
theorem h45_omits_from_213 (p : Nat) (hp : 213 ≤ p) :
    ∃ r, r < p ∧ ∀ h ∈ H45, h % p ≠ r := by
  refine ⟨1, by omega, fun h hh => ?_⟩
  have hle : h ≤ 212 := h45_diameter.2.2 h hh
  have hne : h ≠ 1 := by
    intro h1
    subst h1
    exact absurd hh (by decide +kernel)
  rw [Nat.mod_eq_of_lt (by omega)]
  exact hne

theorem h45_admissible : Admissible H45 := by
  intro p hp
  by_cases hlt : p < 213
  · exact h45_omits_below_213 p hlt hp.1
  · exact h45_omits_from_213 p (by omega)

theorem lemma_12_1 :
    (H45.length = 45 ∧ H45.Nodup) ∧
    (0 ∈ H45 ∧ 212 ∈ H45 ∧ ∀ h ∈ H45, h ≤ 212) ∧
    Admissible H45 :=
  ⟨h45_card, h45_diameter, h45_admissible⟩

end BGP212.Challenge
-- END VENDORED lean-bgp212 Solution/Lemma12_1.lean 20-64

namespace RLT006BGP212

open BGP212 BGP212.Challenge

/-- RLT-006's reviewed transcription (same literals as lean/RLT006Direct.lean). -/
def rltH45 : List Nat := [0, 2, 12, 14, 24, 26, 30, 36, 44, 50, 54, 56, 60, 66, 72, 74, 80, 84, 92, 96, 102, 110, 114, 116, 122, 126, 134, 140, 144, 150, 156, 162, 164, 170, 176, 180, 182, 186, 192, 194, 200, 204, 206, 210, 212]
def rltTable : List (Nat × Nat) := [(2, 1), (3, 1), (5, 3), (7, 6), (11, 9), (13, 3), (17, 1), (19, 13), (23, 17), (29, 4), (31, 11), (37, 4), (41, 4), (43, 3)]

theorem same_tuple : H45 = rltH45 := by decide
theorem same_table : omittedTable = rltTable := by decide

/-- RLT's element statement is the residue statement (as in RLT006Direct). -/
theorem omit_iff (h p r : Nat) (hr : r < p) : (¬ ∃ k, h = p * k + r) ↔ h % p ≠ r := by
  constructor
  · intro hn heq
    exact hn ⟨h / p, by have := Nat.div_add_mod h p; omega⟩
  · intro hne ⟨k, hk⟩
    apply hne
    rw [hk, Nat.mul_add_mod, Nat.mod_eq_of_lt hr]

/-- lean-bgp212's table result, in RLT's form: for every listed (p, r) and every
element h, ¬(∃ k, h = p * k + r). -/
theorem omits_rlt_form : ∀ q ∈ omittedTable, ∀ h ∈ H45, ¬ ∃ k, h = q.1 * k + q.2 :=
  fun q hq h hh => (omit_iff h q.1 q.2 (h45_omitted_table.2 q hq).1).mpr ((h45_omitted_table.2 q hq).2 h hh)

/-- lean-bgp212's packaged lemma, re-checked here from the vendored text. -/
theorem lemma_12_1_bgp212 : (H45.length = 45 ∧ H45.Nodup) ∧ (0 ∈ H45 ∧ 212 ∈ H45 ∧ ∀ h ∈ H45, h ≤ 212) ∧
    Admissible H45 := lemma_12_1

end RLT006BGP212

#print axioms BGP212.Challenge.h45_card
#print axioms BGP212.Challenge.h45_diameter
#print axioms BGP212.Challenge.h45_omitted_table
#print axioms BGP212.Challenge.h45_omits_below_213
#print axioms BGP212.Challenge.h45_omits_from_213
#print axioms BGP212.Challenge.h45_admissible
#print axioms BGP212.Challenge.lemma_12_1
#print axioms RLT006BGP212.same_tuple
#print axioms RLT006BGP212.same_table
#print axioms RLT006BGP212.omit_iff
#print axioms RLT006BGP212.omits_rlt_form
#print axioms RLT006BGP212.lemma_12_1_bgp212

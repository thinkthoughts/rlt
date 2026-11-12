/-
RLT-005, Lean comparison 1 of 2: a direct Lean proof of the finite
proposition RLT's kernel accepted as `matchable.not.8`
(tests/rlt005/accept/MN8_eight_is_not_matchable.rlt). Hand-written; core Lean
only. Run: cd lean && lean RLT005Direct.lean

The statement below is the RLT written goal, symbol for symbol, with RLT's
`<` and `∣` read as Lean's `<` and `∣` on Nat and conjunctions nested as in
RLT. `lt_iff_rlt` and `dvd_iff_rlt` check that this reading agrees with the
NAT-DEF definitions RLT unfolds (x < y := ∃z. S(z) + x = y;
d ∣ n := ∃k. n = d × k). This file establishes a Lean proof of the same
proposition; it is no RLT derivation, and the RLT proof does not depend on it.
-/

namespace RLT005Direct

/-- NAT-DEF `lt`, as RLT unfolds it, agrees with Lean's `<` on Nat. -/
theorem lt_iff_rlt (x y : Nat) : x < y ↔ ∃ z, (z + 1) + x = y := by
  constructor
  · intro h; exact ⟨y - x - 1, by omega⟩
  · intro ⟨z, h⟩; omega

/-- NAT-DEF `dvd`, as RLT unfolds it, is Lean's `∣` on Nat. -/
theorem dvd_iff_rlt (d n : Nat) : d ∣ n ↔ ∃ k, n = d * k := Iff.rfl

-- BEGIN SHARED RLT-005 BLOCK (byte-identical in RLT005Readings.lean; scripts/verify.sh compares)
/-- RLT's coprimality formula: every positive common divisor is 1. -/
def cop (m x : Nat) : Prop := ∀ k, ((0 < k ∧ k ∣ m) ∧ k ∣ x) → k = 1

/-- For m > 0, RLT's coprimality formula is gcd = 1. -/
theorem cop_iff_gcd (m x : Nat) (hm : 0 < m) : cop m x ↔ Nat.gcd m x = 1 := by
  constructor
  · intro h
    exact h _ ⟨⟨Nat.gcd_pos_of_pos_left x hm, Nat.gcd_dvd_left m x⟩, Nat.gcd_dvd_right m x⟩
  · intro h k ⟨⟨_, hkm⟩, hkx⟩
    have := Nat.dvd_gcd hkm hkx
    rw [h] at this
    exact Nat.dvd_one.mp this

/-- The partner ranges: 0 < x ∧ x < 5. -/
def r (x : Nat) : Prop := 0 < x ∧ x < 5

/-- The matching conditions, nested exactly as in RLT's `Mfor`. -/
def M (a b c d : Nat) : Prop :=
  ((((r a ∧ r b) ∧ r c) ∧ r d) ∧
    (((((¬ a = b ∧ ¬ a = c) ∧ ¬ a = d) ∧ ¬ b = c) ∧ ¬ b = d) ∧ ¬ c = d)) ∧
  (((cop 1 a ∧ cop 2 b) ∧ cop 4 c) ∧ cop 8 d)

/-- 0 < x < 5 places x in the list [1, 2, 3, 4]. -/
theorem mem_partners (x : Nat) (h : r x) : x ∈ [1, 2, 3, 4] := by
  have : x = 1 ∨ x = 2 ∨ x = 3 ∨ x = 4 := by unfold r at h; omega
  rcases this with h | h | h | h <;> subst h <;> decide

/-- The same conditions with gcd, over the partner list: decided over 4⁴
cases. -/
theorem bounded_none :
    ∀ a ∈ [1, 2, 3, 4], ∀ b ∈ [1, 2, 3, 4], ∀ c ∈ [1, 2, 3, 4], ∀ d ∈ [1, 2, 3, 4],
      ¬ ((((((¬ a = b ∧ ¬ a = c) ∧ ¬ a = d) ∧ ¬ b = c) ∧ ¬ b = d) ∧ ¬ c = d) ∧
        (((Nat.gcd 1 a = 1 ∧ Nat.gcd 2 b = 1) ∧ Nat.gcd 4 c = 1) ∧ Nat.gcd 8 d = 1)) := by
  decide

/-- RLT's `matchable.not.8.contradiction`: no assignment meets M. -/
theorem no_assignment : ∀ a b c d, ¬ M a b c d := by
  intro a b c d ⟨⟨⟨⟨⟨ra, rb⟩, rc⟩, rd⟩, dist⟩, ⟨⟨⟨ca, cb⟩, cc⟩, cd⟩⟩
  rw [cop_iff_gcd 1 a (by decide)] at ca
  rw [cop_iff_gcd 2 b (by decide)] at cb
  rw [cop_iff_gcd 4 c (by decide)] at cc
  rw [cop_iff_gcd 8 d (by decide)] at cd
  exact bounded_none a (mem_partners a ra) b (mem_partners b rb) c (mem_partners c rc)
    d (mem_partners d rd) ⟨dist, ⟨⟨⟨ca, cb⟩, cc⟩, cd⟩⟩

-- END SHARED RLT-005 BLOCK

/-- RLT's `matchable.divisors.8`. -/
theorem divisors_eight :
    ∀ d, ((0 < d ∧ d ∣ 8) → (((d = 1 ∨ d = 2) ∨ d = 4) ∨ d = 8)) ∧
      ((((d = 1 ∨ d = 2) ∨ d = 4) ∨ d = 8) → (0 < d ∧ d ∣ 8)) := by
  intro d
  have bounded : ∀ d, d < 9 →
      ((0 < d ∧ d ∣ 8) → (((d = 1 ∨ d = 2) ∨ d = 4) ∨ d = 8)) := by decide
  refine ⟨fun h => bounded d (Nat.lt_succ_of_le (Nat.le_of_dvd (by decide) h.2)) h, ?_⟩
  intro h
  rcases h with ((h | h) | h) | h <;> subst h <;> decide

/-- RLT's `matchable.partners.4`. -/
theorem partners_four :
    ∀ x, ((0 < x ∧ x < 5) → (((x = 1 ∨ x = 2) ∨ x = 3) ∨ x = 4)) ∧
      ((((x = 1 ∨ x = 2) ∨ x = 3) ∨ x = 4) → (0 < x ∧ x < 5)) := by
  intro x
  constructor
  · intro ⟨h0, h5⟩; omega
  · intro h; omega

/-- RLT's `matchable.not.8`, the written goal of MN8. -/
theorem matchable_not_8 :
    ((∀ d, ((0 < d ∧ d ∣ 8) → (((d = 1 ∨ d = 2) ∨ d = 4) ∨ d = 8)) ∧
        ((((d = 1 ∨ d = 2) ∨ d = 4) ∨ d = 8) → (0 < d ∧ d ∣ 8))) ∧
      (∀ x, ((0 < x ∧ x < 5) → (((x = 1 ∨ x = 2) ∨ x = 3) ∨ x = 4)) ∧
        ((((x = 1 ∨ x = 2) ∨ x = 3) ∨ x = 4) → (0 < x ∧ x < 5)))) ∧
    ¬ (∃ a b c d,
      (((((0 < a ∧ a < 5) ∧ (0 < b ∧ b < 5)) ∧ (0 < c ∧ c < 5)) ∧ (0 < d ∧ d < 5)) ∧
        (((((¬ a = b ∧ ¬ a = c) ∧ ¬ a = d) ∧ ¬ b = c) ∧ ¬ b = d) ∧ ¬ c = d)) ∧
      ((((∀ k, ((0 < k ∧ k ∣ 1) ∧ k ∣ a) → k = 1) ∧
          (∀ k, ((0 < k ∧ k ∣ 2) ∧ k ∣ b) → k = 1)) ∧
          (∀ k, ((0 < k ∧ k ∣ 4) ∧ k ∣ c) → k = 1)) ∧
          (∀ k, ((0 < k ∧ k ∣ 8) ∧ k ∣ d) → k = 1))) :=
  ⟨⟨divisors_eight, partners_four⟩, fun ⟨a, b, c, d, h⟩ => no_assignment a b c d h⟩

/-- The kernel's checked goal of MN8, literally: NAT-DEF's `<` unfolded to
∃z. S(z) + x = y (S(z) read as z + 1) and `∣` to ∃k. n = d × k. -/
theorem matchable_not_8_checked :
    ((∀ d, ((((∃ z, (z + 1) + 0 = d) ∧ (∃ k, 8 = d * k)) → (((d = 1 ∨ d = 2) ∨ d = 4) ∨ d = 8)) ∧ ((((d = 1 ∨ d = 2) ∨ d = 4) ∨ d = 8) → ((∃ z, (z + 1) + 0 = d) ∧ (∃ k, 8 = d * k))))) ∧ (∀ x, ((((∃ z, (z + 1) + 0 = x) ∧ (∃ z, (z + 1) + x = 5)) → (((x = 1 ∨ x = 2) ∨ x = 3) ∨ x = 4)) ∧ ((((x = 1 ∨ x = 2) ∨ x = 3) ∨ x = 4) → ((∃ z, (z + 1) + 0 = x) ∧ (∃ z, (z + 1) + x = 5)))))) ∧ ¬ (∃ a, ∃ b, ∃ c, ∃ d, (((((((∃ z, (z + 1) + 0 = a) ∧ (∃ z, (z + 1) + a = 5)) ∧ ((∃ z, (z + 1) + 0 = b) ∧ (∃ z, (z + 1) + b = 5))) ∧ ((∃ z, (z + 1) + 0 = c) ∧ (∃ z, (z + 1) + c = 5))) ∧ ((∃ z, (z + 1) + 0 = d) ∧ (∃ z, (z + 1) + d = 5))) ∧ (((((¬ (a = b) ∧ ¬ (a = c)) ∧ ¬ (a = d)) ∧ ¬ (b = c)) ∧ ¬ (b = d)) ∧ ¬ (c = d))) ∧ ((((∀ k, ((((∃ z, (z + 1) + 0 = k) ∧ (∃ k', 1 = k * k')) ∧ (∃ k', a = k * k')) → k = 1)) ∧ (∀ k, ((((∃ z, (z + 1) + 0 = k) ∧ (∃ k', 2 = k * k')) ∧ (∃ k', b = k * k')) → k = 1))) ∧ (∀ k, ((((∃ z, (z + 1) + 0 = k) ∧ (∃ k', 4 = k * k')) ∧ (∃ k', c = k * k')) → k = 1))) ∧ (∀ k, ((((∃ z, (z + 1) + 0 = k) ∧ (∃ k', 8 = k * k')) ∧ (∃ k', d = k * k')) → k = 1))))) := by
  have h := matchable_not_8
  simp only [lt_iff_rlt] at h
  exact h

/-- Non-vacuity, as RLT's `matchable.not.8.nonvacuous`: without
distinctness the conditions are met (all partners 1). -/
theorem nonvacuous :
    ∃ a b c d, (((r a ∧ r b) ∧ r c) ∧ r d) ∧
      (((cop 1 a ∧ cop 2 b) ∧ cop 4 c) ∧ cop 8 d) := by
  refine ⟨1, 1, 1, 1, by unfold r; decide, ?_⟩
  have h : ∀ m, 0 < m → cop m 1 := fun m hm => (cop_iff_gcd m 1 hm).mpr (Nat.gcd_one_right m)
  exact ⟨⟨⟨h 1 (by decide), h 2 (by decide)⟩, h 4 (by decide)⟩, h 8 (by decide)⟩

end RLT005Direct

#print axioms RLT005Direct.lt_iff_rlt
#print axioms RLT005Direct.dvd_iff_rlt
#print axioms RLT005Direct.cop_iff_gcd
#print axioms RLT005Direct.mem_partners
#print axioms RLT005Direct.bounded_none
#print axioms RLT005Direct.no_assignment
#print axioms RLT005Direct.divisors_eight
#print axioms RLT005Direct.partners_four
#print axioms RLT005Direct.matchable_not_8
#print axioms RLT005Direct.matchable_not_8_checked
#print axioms RLT005Direct.nonvacuous

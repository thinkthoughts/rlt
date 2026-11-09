/-
RLT-003 M3d, tier 2 of the Lean comparison. Hand-written; core Lean only.

Tier 1 (M3dGenerated.lean) restates the theorems RLT's kernel accepted. This
file checks what they mean in Lean's natural numbers: RLT's unfolded order
∃z. S(z) + x = y is Lean core's x < y; the 30 disjuncts of the enumeration
are exactly the values below 30; the bound is sharp; and the arithmetic of
one successor layer of the M3d derivation holds over ℕ. These are
Lean-checked statements; they establish no RLT derivation, which remains
RLT's. Run: cd lean && lean M3dReadings.lean
-/

namespace RLTM3dReadings

/-- RLT's `lt` body (Q8, S(z) written z + 1) is Lean core's `<`. -/
theorem lt_unfolds : ∀ x y : Nat, (∃ z, (z + 1) + x = y) ↔ x < y := by
  intro x y
  constructor
  · intro ⟨z, h⟩; omega
  · intro h; exact ⟨y - x - 1, by omega⟩

/-- The values below 30 are exactly the 30 listed numerals 0 … 29. -/
theorem below_thirty : ∀ x : Nat, x < 30 ↔ x ∈ List.range 30 := by
  intro x; simp

/-- The enumeration's 30 disjuncts are distinct: List.range 30 has no duplicates. -/
theorem disjuncts_distinct : (List.range 30).Nodup := List.nodup_range

/-- The bound is sharp: 29 is below 30, and 30 is not. -/
theorem bound_is_sharp : 29 < 30 ∧ ¬ (30 < 30) := by decide

/-- One successor layer: S(w) + S(a) = S(k) gives S(w) + a = k. -/
theorem layer_step : ∀ w a k : Nat, (w + 1) + (a + 1) = k + 1 → (w + 1) + a = k := by
  intro w a k h; omega

/-- The closing layer: S(w) + a = 0 is impossible. -/
theorem closing_layer : ∀ w a : Nat, ¬ ((w + 1) + a = 0) := by
  intro w a; omega

end RLTM3dReadings

#print axioms RLTM3dReadings.lt_unfolds
#print axioms RLTM3dReadings.below_thirty
#print axioms RLTM3dReadings.disjuncts_distinct
#print axioms RLTM3dReadings.bound_is_sharp
#print axioms RLTM3dReadings.layer_step
#print axioms RLTM3dReadings.closing_layer

/-
RLT-003 M3e, tier 2 of the Lean comparison. Hand-written; core Lean only.

Tier 1 (M3eGenerated.lean) restates the theorems RLT's kernel accepted. This
file checks what the classification means over ℕ: free235(r), as RLT unfolds
it, holds below 30 exactly for the eight listed representatives; those are
exactly the r < 30 coprime to 30; and the list has eight distinct entries
(φ(30) = 8, as a list fact). Lean-checked statements; they establish no RLT
derivation. Run: cd lean && lean M3eReadings.lean
-/

namespace RLTM3eReadings

/-- The eight representatives, in RESIDUE-30's written order. -/
def reps : List Nat := [1, 7, 11, 13, 17, 19, 23, 29]

/-- Below 30, free235(r) holds exactly for the eight representatives. -/
theorem classification :
    ∀ r, r < 30 → (((¬ 2 ∣ r ∧ ¬ 3 ∣ r) ∧ ¬ 5 ∣ r) ↔ r ∈ reps) := by
  decide

/-- Below 30, free235(r) is coprimality with 30. -/
theorem free235_iff_coprime :
    ∀ r, r < 30 → (((¬ 2 ∣ r ∧ ¬ 3 ∣ r) ∧ ¬ 5 ∣ r) ↔ Nat.gcd r 30 = 1) := by
  decide

/-- Eight representatives, all distinct. -/
theorem representatives_count : reps.length = 8 ∧ reps.Nodup := by
  decide

end RLTM3eReadings

#print axioms RLTM3eReadings.classification
#print axioms RLTM3eReadings.free235_iff_coprime
#print axioms RLTM3eReadings.representatives_count

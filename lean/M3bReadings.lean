/-
RLT-003 M3b, tier 2 of the Lean comparison. Hand-written; core Lean only.

Tier 1 (M3bGenerated.lean) restates the theorems RLT's kernel accepted. This
file checks the correspondence between RLT's registered axioms and
definitions and Lean's natural numbers: each lemma says that the formula RLT
registers (stated as RLT unfolds it, with S(x) written x + 1) is true in ℕ,
or means the same as Lean's own notion. These are Lean-checked statements;
they establish no RLT derivation, which remains RLT's. Run:
cd lean && lean M3bReadings.lean
-/

namespace RLTM3bReadings

/-- Q1 (OLP numbering): successor is injective. -/
theorem q1_succ_inj : ∀ x y : Nat, x + 1 = y + 1 → x = y :=
  fun _ _ h => Nat.succ.inj h

/-- Q2 (OLP numbering and orientation): ¬(0 = S(x)). -/
theorem q2_zero_ne_succ : ∀ x : Nat, ¬ (0 = x + 1) := by
  intro x; omega

/-- Q3: every number is zero or a successor. -/
theorem q3_zero_or_succ : ∀ x : Nat, x = 0 ∨ ∃ y, x = y + 1 := by
  intro x
  cases x with
  | zero => exact Or.inl rfl
  | succ n => exact Or.inr ⟨n, rfl⟩

/-- Q4–Q7 hold in ℕ (they are Lean's defining equations). -/
theorem q4 : ∀ x : Nat, x + 0 = x := fun _ => rfl
theorem q5 : ∀ x y : Nat, x + (y + 1) = (x + y) + 1 := fun _ _ => rfl
theorem q6 : ∀ x : Nat, x * 0 = 0 := fun _ => rfl
theorem q7 : ∀ x y : Nat, x * (y + 1) = x * y + x := fun _ _ => rfl

/-- lt (Q8's right-hand side) means Lean's `<` on ℕ. -/
theorem lt_iff : ∀ x y : Nat, (∃ z, (z + 1) + x = y) ↔ x < y := by
  intro x y
  constructor
  · intro h
    cases h with
    | intro z hz => omega
  · intro h
    exact ⟨y - x - 1, by omega⟩

/-- dvd is Lean core's `Dvd Nat` instance, by definition. -/
theorem dvd_iff : ∀ d n : Nat, (∃ k, n = d * k) ↔ d ∣ n := fun _ _ => Iff.rfl

/-- free235, as RLT unfolds it, is the conjunction of Lean's divisibilities. -/
theorem free235_iff : ∀ r : Nat,
    ((¬ ∃ k, r = 2 * k) ∧ (¬ ∃ k, r = 3 * k)) ∧ (¬ ∃ k, r = 5 * k) ↔
    (¬ 2 ∣ r ∧ ¬ 3 ∣ r) ∧ ¬ 5 ∣ r := fun _ => Iff.rfl

end RLTM3bReadings

-- Axiom gate: each statement may rest only on Lean's standard axioms
-- (propext, Classical.choice, Quot.sound); the scripts refuse any other.
#print axioms RLTM3bReadings.q1_succ_inj
#print axioms RLTM3bReadings.q2_zero_ne_succ
#print axioms RLTM3bReadings.q3_zero_or_succ
#print axioms RLTM3bReadings.q4
#print axioms RLTM3bReadings.q5
#print axioms RLTM3bReadings.q6
#print axioms RLTM3bReadings.q7
#print axioms RLTM3bReadings.lt_iff
#print axioms RLTM3bReadings.dvd_iff
#print axioms RLTM3bReadings.free235_iff

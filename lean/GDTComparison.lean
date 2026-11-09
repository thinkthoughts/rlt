/-
RLT-003 M2, GDT comparison. Hand-written; requires Mathlib and the General
Divisor Theorem repository at ec66260c98eff965bea79891ffbbc0d3229c6e93
(GeneralDivisorTheorem/Basic.lean, registered as GDT-BASIC).

These are Lean-checked statements about GDT's definitions. They are not RLT
theorems, and they make no RLT reading a theorem. The comparison is reported
on its own, with its verification status (compiled or unverified), outside
RLT's CI gate. Run from a GDT checkout:
  lake env lean /path/to/rlt/lean/GDTComparison.lean
-/
import GeneralDivisorTheorem.Basic
import Mathlib.Tactic.Simproc.Factors

namespace RLTGDTComparison

open GDT

/-- With modulus m = 1 and a = 0 the congruence condition holds for every n,
so GDT's `accepted 30 1 0` is exactly coprimality with 30. -/
theorem accepted_30_1_0_iff (n : ℕ) : accepted 30 1 0 n ↔ Nat.gcd n 30 = 1 := by
  unfold accepted
  simp [Int.modEq_one]

/-- GDT's accepted set below 30 is the set RESIDUE-30 registers. -/
theorem S_30_29_1_0 : S 30 29 1 0 = {1, 7, 11, 13, 17, 19, 23, 29} := by
  decide

/-- The prime factors of 30 are 2, 3 and 5. -/
theorem primeFactors_30 : (30 : ℕ).primeFactors = {2, 3, 5} := by
  simp [Nat.primeFactors, Nat.primeFactorsList_ofNat]

/-- The radical of 30 is 30 (2 × 3 × 5, each prime once). -/
theorem rad_30 : rad 30 = 30 := by
  simp [rad, primeFactors_30]

/-- With m = 1: d = rad(gcd 1 30) = 1, R = 30 / 1 = 30, Tmin = 1 × 30 = 30. -/
theorem d_30_1 : d 30 1 = 1 := by simp [d, rad]
theorem R_30_1 : R 30 1 = 30 := by simp [R, d_30_1, rad_30]
theorem Tmin_30_1 : Tmin 30 1 = 30 := by simp [Tmin, R_30_1]

/-- RLT-003 M3e: below 30, GDT's accepted set is the free235 set RLT's E-T1
classifies: accepted 30 1 0 n holds exactly when ¬ 2 ∣ n ∧ ¬ 3 ∣ n ∧ ¬ 5 ∣ n. -/
theorem accepted_iff_free235 :
    ∀ n, n < 30 → (accepted 30 1 0 n ↔ ((¬ 2 ∣ n ∧ ¬ 3 ∣ n) ∧ ¬ 5 ∣ n)) := by
  intro n hn
  rw [accepted_30_1_0_iff]
  revert n
  decide

end RLTGDTComparison

-- Audit gate: each statement may rest only on Lean's standard axioms
-- (propext, Classical.choice, Quot.sound); the scripts refuse any other.
#print axioms RLTGDTComparison.accepted_30_1_0_iff
#print axioms RLTGDTComparison.S_30_29_1_0
#print axioms RLTGDTComparison.primeFactors_30
#print axioms RLTGDTComparison.rad_30
#print axioms RLTGDTComparison.d_30_1
#print axioms RLTGDTComparison.R_30_1
#print axioms RLTGDTComparison.Tmin_30_1
#print axioms RLTGDTComparison.accepted_iff_free235

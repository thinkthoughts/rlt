/-
RLT-003 M3c, tier 2 of the Lean comparison. Hand-written; core Lean only.

Tier 1 (M3cGenerated.lean) restates the theorems RLT's kernel accepted. This
file checks that those statements mean what M3c says they mean in Lean's
natural numbers: RLT's unfolded `d ∣ n` is Lean core's `d ∣ n`, the
witness-wise form is equivalent to the non-divisibility form, free235(7) is
the statement that 7 leaves a nonzero remainder on division by 2, 3 and 5,
and the two arithmetic steps of the M3c scheme (lemma M_d and the final Q2
closure at an arbitrary remaining variable) hold over ℕ. These are
Lean-checked statements; they establish no RLT derivation, which remains
RLT's. Run: cd lean && lean M3cReadings.lean
-/

namespace RLTM3cReadings

/-- RLT's `dvd` body, ∃k. n = d × k, is Lean core's `d ∣ n` (definitionally). -/
theorem dvd_unfolds : ∀ d n : Nat, (∃ k, n = d * k) ↔ d ∣ n :=
  fun _ _ => Iff.rfl

/-- The witness-wise form ∀k. ¬(n = d × k) is equivalent to ¬(d ∣ n). -/
theorem witnesswise_iff_not_dvd : ∀ d n : Nat, (∀ k, ¬ (n = d * k)) ↔ ¬ d ∣ n :=
  fun _ _ => ⟨fun h ⟨k, hk⟩ => h k hk, fun h k hk => h ⟨k, hk⟩⟩

/-- free235(7), as RLT unfolds it, is the remainder statement for 2, 3 and 5. -/
theorem free235_seven_remainders :
    (((¬ ∃ k : Nat, 7 = 2 * k) ∧ (¬ ∃ k : Nat, 7 = 3 * k)) ∧ (¬ ∃ k : Nat, 7 = 5 * k)) ↔
      ((7 % 2 ≠ 0 ∧ 7 % 3 ≠ 0) ∧ 7 % 5 ≠ 0) := by
  show ((¬ 2 ∣ 7 ∧ ¬ 3 ∣ 7) ∧ ¬ 5 ∣ 7) ↔ _
  decide

/-- Lemma M_d over ℕ: d × S(p) = S^d(d × p), with S^d(t) = t + d. -/
theorem lemma_m : ∀ d p : Nat, d * (p + 1) = d * p + d :=
  fun d p => Nat.mul_succ d p

/-- The last case of ¬(2 ∣ 7) closes for every remaining p₄: ¬(0 = S(2 × p₄)). -/
theorem last_case_closes : ∀ p : Nat, ¬ (0 = 2 * p + 1) := by
  intro p; omega

/-- The negative control: 2 ∣ 8 holds, so no proof of ¬(2 ∣ 8) can exist. -/
theorem two_divides_eight : 2 ∣ 8 := ⟨4, rfl⟩

end RLTM3cReadings

#print axioms RLTM3cReadings.dvd_unfolds
#print axioms RLTM3cReadings.witnesswise_iff_not_dvd
#print axioms RLTM3cReadings.free235_seven_remainders
#print axioms RLTM3cReadings.lemma_m
#print axioms RLTM3cReadings.last_case_closes
#print axioms RLTM3cReadings.two_divides_eight

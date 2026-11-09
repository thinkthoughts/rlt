/-
RLT-003 M2, tier 2 of the core-Lean comparison. Hand-written; core Lean only.

Tier 1 (M2Generated.lean) restates the 111 equations RLT's kernel checked.
This file checks, in Lean, statements RLT does NOT prove: the registered
classification rule of RESIDUE-30 and its agreement with gcd. These are
Lean-checked statements about ℕ. They are not RLT theorems, and they do not
turn the RESIDUE-30 readings into RLT theorems; they are reported as an
independent comparison. Run: cd lean && lean M2Readings.lean
-/

namespace RLTM2Readings

/-- The registered rule `nonzero-remainders`: r is a coprime representative
of 30 iff its remainders by 2, 3 and 5 are all nonzero. -/
def nonzeroRemainders (r : Nat) : Bool :=
  r % 2 != 0 && r % 3 != 0 && r % 5 != 0

/-- The rule applied to the complete table 0..29 gives the quoted set. -/
theorem classification_table :
    (List.range 30).filter nonzeroRemainders = [1, 7, 11, 13, 17, 19, 23, 29] := by
  decide

/-- On 0..29 the rule agrees with gcd(r, 30) = 1, entry by entry. -/
theorem rule_agrees_with_gcd :
    (List.range 30).all (fun r => nonzeroRemainders r == (Nat.gcd r 30 == 1)) = true := by
  decide

/-- The same set computed from gcd directly. -/
theorem gcd_table :
    (List.range 30).filter (fun r => Nat.gcd r 30 == 1) = [1, 7, 11, 13, 17, 19, 23, 29] := by
  decide

/-- Remainders are below their divisors (the role of RLT's bound witnesses). -/
theorem remainders_bounded :
    (List.range 30).all (fun r => r % 2 < 2 && r % 3 < 3 && r % 5 < 5) = true := by
  decide

/-- The five class witnesses: 29, 30, 31, 37 and 59 modulo 30. -/
theorem class_witnesses :
    [29, 30, 31, 37, 59].map (· % 30) = [29, 0, 1, 7, 29] := by
  decide

/-- The count of the representatives. -/
theorem eight : [1, 7, 11, 13, 17, 19, 23, 29].length = 8 := by decide

end RLTM2Readings

-- Audit gate: each statement may rest only on Lean's standard axioms
-- (propext, Classical.choice, Quot.sound); the scripts refuse any other.
#print axioms RLTM2Readings.classification_table
#print axioms RLTM2Readings.rule_agrees_with_gcd
#print axioms RLTM2Readings.gcd_table
#print axioms RLTM2Readings.remainders_bounded
#print axioms RLTM2Readings.class_witnesses
#print axioms RLTM2Readings.eight

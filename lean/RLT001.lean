/-
RLT-001 · independent Lean replay (core Lean 4, no Mathlib).

Lean is a second, independent checker. It re-checks the mathematical
statements RLT's kernel accepted or rejected, using Lean's own `Nat`.
It sits outside RLT's trusted computing base: agreement between the two
checkers is evidence, and RLT's verdicts come from rlt-kernel alone.

Run: lean lean/RLT001.lean
-/

namespace RLT001

/-! T1 · math5.pairing — accepted unconditional: 5 + 1 = 6.
The second proof mirrors RLT's four steps with Lean's recursion lemmas. -/
theorem T1 : 5 + 1 = 6 := rfl

theorem T1_steps : 5 + Nat.succ 0 = Nat.succ 5 :=
  have s1 : 5 + Nat.succ 0 = Nat.succ (5 + 0) := Nat.add_succ 5 0
  have s2 : 5 + 0 = 5 := Nat.add_zero 5
  have s3 : Nat.succ (5 + 0) = Nat.succ 5 := congrArg Nat.succ s2
  s1.trans s3

/-! T2 · rejected: the broken step claimed 5 + 1 = 5, which is false. -/
theorem T2_claim_false : ¬ (5 + 1 = 5) := by decide

/-! T3 · rejected by RLT for citing an unregistered rule. The equation itself
is true; RLT's rejection concerns the missing derivation. -/
theorem T3_statement_true : 5 + 1 = 6 := rfl

/-! T4 · math0.pairing — accepted unconditional: 1 + 1 = 2. -/
theorem T4 : 1 + 1 = 2 := rfl

/-! T5a · accepted conditional: {h} ⊢ 1+1+1+1+1 = 0. -/
theorem T5a (h : 1 + 1 + 1 + 1 + 1 = 0) : 1 + 1 + 1 + 1 + 1 = 0 := h

/-! T5b · rejected as unconditional: the statement is false without h. -/
theorem T5b_unconditional_false : ¬ (1 + 1 + 1 + 1 + 1 = 0) := by decide

/-! T5c · accepted conditional: {h : 1 + 1 = 3} ⊢ 1 + 1 + 1 = 3 + 1. -/
theorem T5c (h : 1 + 1 = 3) : 1 + 1 + 1 = 3 + 1 := congrArg (· + 1) h

/-! T5d · rejected as unconditional: false without h. -/
theorem T5d_unconditional_false : ¬ (1 + 1 + 1 = 3 + 1) := by decide

/-! Further accepted relationships. -/
theorem math5_count : 1 + 1 + 1 + 1 + 1 = 5 := rfl
theorem math29_pairing : 29 + 1 = 30 := rfl

end RLT001

#print axioms RLT001.T1_steps
#print axioms RLT001.T5a
#print axioms RLT001.T5c
#print axioms RLT001.T5b_unconditional_false

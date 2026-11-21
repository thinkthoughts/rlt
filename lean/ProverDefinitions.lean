/-
RLT-PROVER-002, the Lean comparison for registered function definitions.
Hand-written; core Lean only.

`definitions/double.rlt` registers double(x) := x + x and
quadruple(x) := double(double(x)). Lean's `def`s below are the same
definitions. For each RLT goal in tests/definitions/goals/, two Lean facts:

- `*_unfolds`: the folded statement (with Lean's `double`) is equivalent to
  the statement RLT's kernel checked after unfolding (S(x) written x + 1).
  This is the source-to-core step RLT's elaborator performs, checked here by
  an independent implementation (Lean's definitional unfolding).
- `*_holds`: the statement is true in ℕ.

These are Lean-checked statements about Lean's ℕ, where induction is
available; they establish no RLT derivation, which remains RLT's, under
Robinson's Q. G1 and G2 show the difference: true in ℕ (proved here with
`omega`, which uses more than Q), and NOT_FOUND_WITHIN_BOUNDS for RLT.
Run: cd lean && lean ProverDefinitions.lean
-/

namespace RLTProverDefinitions

def double (x : Nat) : Nat := x + x
def quadruple (x : Nat) : Nat := double (double x)

/-- The registered unfolding of quadruple, as RLT stores it. -/
theorem quadruple_body : ∀ x : Nat, quadruple x = (x + x) + (x + x) :=
  fun _ => rfl

theorem F1_unfolds : (∀ x, double x = x + x) ↔ (∀ x : Nat, x + x = x + x) :=
  Iff.rfl
theorem F1_holds : ∀ x, double x = x + x := fun _ => rfl

theorem F2_unfolds : (double 0 = 0) ↔ (0 + 0 = 0) := Iff.rfl
theorem F2_holds : double 0 = 0 := rfl

theorem F3_unfolds : (double 2 = 4) ↔ (2 + 2 = 4) := Iff.rfl
theorem F3_holds : double 2 = 4 := rfl

theorem F4_unfolds :
    (∀ x, x + (x + 1) = double x + 1) ↔ (∀ x : Nat, x + (x + 1) = (x + x) + 1) :=
  Iff.rfl
theorem F4_holds : ∀ x, x + (x + 1) = double x + 1 := by
  intro x; unfold double; omega

theorem F5_unfolds :
    (∀ x, ¬ (double (x + 1) = 0)) ↔ (∀ x : Nat, ¬ ((x + 1) + (x + 1) = 0)) :=
  Iff.rfl
theorem F5_holds : ∀ x, ¬ (double (x + 1) = 0) := by
  intro x; unfold double; omega

theorem F6_unfolds :
    (∀ x, quadruple x = double x + double x)
      ↔ (∀ x : Nat, (x + x) + (x + x) = (x + x) + (x + x)) :=
  Iff.rfl
theorem F6_holds : ∀ x, quadruple x = double x + double x := fun _ => rfl

theorem F7_unfolds : (quadruple 1 = 4) ↔ ((1 + 1) + (1 + 1) = 4) := Iff.rfl
theorem F7_holds : quadruple 1 = 4 := rfl

/-- G1 and G2: true in ℕ by induction-strength reasoning (`omega`); RLT
reports both NOT_FOUND_WITHIN_BOUNDS under Robinson's Q. -/
theorem G1_holds : ∀ x, double x = 2 * x := by
  intro x; unfold double; omega
theorem G2_holds : ∀ x, double x = x * 2 := by
  intro x; unfold double; omega

end RLTProverDefinitions

-- Axiom gate: each statement may rest only on Lean's standard axioms
-- (propext, Classical.choice, Quot.sound); the scripts refuse any other.
#print axioms RLTProverDefinitions.quadruple_body
#print axioms RLTProverDefinitions.F1_unfolds
#print axioms RLTProverDefinitions.F1_holds
#print axioms RLTProverDefinitions.F2_unfolds
#print axioms RLTProverDefinitions.F2_holds
#print axioms RLTProverDefinitions.F3_unfolds
#print axioms RLTProverDefinitions.F3_holds
#print axioms RLTProverDefinitions.F4_unfolds
#print axioms RLTProverDefinitions.F4_holds
#print axioms RLTProverDefinitions.F5_unfolds
#print axioms RLTProverDefinitions.F5_holds
#print axioms RLTProverDefinitions.F6_unfolds
#print axioms RLTProverDefinitions.F6_holds
#print axioms RLTProverDefinitions.F7_unfolds
#print axioms RLTProverDefinitions.F7_holds
#print axioms RLTProverDefinitions.G1_holds
#print axioms RLTProverDefinitions.G2_holds

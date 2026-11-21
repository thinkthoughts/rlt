/-
RLT-PROVER-002, the Lean comparison for registered function definitions.
Hand-written; core Lean only.

`definitions/double.rlt` registers double(x) := x + x and
quadruple(x) := double(double(x)). The Lean `def`s below are hand-written
copies of those definitions. For each RLT goal in tests/definitions/goals/,
two hand-written Lean theorems:

- `*_unfolds`: the folded statement (with Lean's `double`) is equivalent to
  the statement RLT's kernel checked after unfolding (S(x) written x + 1),
  proved by `Iff.rfl`, i.e. up to Lean's definitional equality.
- `*_holds`: the statement is true in ℕ.

What each part establishes:

1. Textual correspondence (scripts/check-definitions-lean.py, not Lean):
   the script rewrites RLT's printed `folded` and `checked goal` lines, and
   the registered written definitions, into Lean syntax and requires each
   expected theorem and `def` to be declared exactly once, outside comments,
   with exactly that statement; the file must keep the small declaration
   grammar the script states (no other commands, names or literals). This is
   what ties the Lean text to the particular unfolding RLT printed, for these
   seven goals.
2. Lean's checks: Lean confirms the hand-written theorems below. `Iff.rfl`
   shows the two sides are equivalent in Lean's ℕ; it accepts any
   definitionally equal pair (for example `(double 2 = 4) ↔ (3 + 1 = 4)`
   also holds by `Iff.rfl`), so on its own it does not validate RLT's
   unfolding algorithm; that comes from step 1.
3. Not established: a general formal proof that RLT's elaborator and
   unfolder implement the same semantics as Lean. The comparison covers the
   seven stated goals only.

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

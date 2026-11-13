/-
Design evidence for RLT-007b stage 2 (core Lean only); replayed by verify.sh, with
all 20 theorems printed and gated by name (standard axioms only).
Run: cd lean && lean RLT007S2Design.lean
Part 1: a model of Robinson's Q in which 0 × a = 7 for a nonstandard a, so the
unbounded definition ∀d. (∃k. 7 = d × k) → d = 1 ∨ d = 7 fails for 7.
Part 2: over ℕ, the bounded definition equals the standard one.
Part 3: decided classification of the primes below 213 under the bounded definition.
-/
namespace RLT007S2Design

inductive M where
  | std : Nat → M
  | ns : Int → M
  deriving DecidableEq

def M.succ : M → M
  | .std n => .std (n + 1)
  | .ns i => .ns (i + 1)

def iter : Nat → M → M
  | 0, x => x
  | n + 1, x => M.succ (iter n x)

def M.add : M → M → M
  | x, .std n => iter n x
  | _, .ns i => .ns i

def M.mul : M → M → M
  | .std m, .std n => .std (m * n)
  | .ns j, .std n => if n = 0 then .std 0 else .ns j
  | .std 0, .ns _ => .std 7
  | .std (m + 1), .ns i => .ns ((m + 1 : Int) * i)
  | .ns j, .ns _ => .ns j

theorem iter_std (n m : Nat) : iter n (.std m) = .std (m + n) := by
  induction n with
  | zero => rfl
  | succ n ih => simp only [iter, ih, M.succ]; congr 1

theorem iter_ns (n : Nat) (i : Int) : iter n (.ns i) = .ns (i + n) := by
  induction n with
  | zero => simp [iter]
  | succ n ih => simp only [iter, ih, M.succ]; congr 1; omega

theorem iter_succ (n : Nat) (x : M) : iter (n + 1) x = M.succ (iter n x) := rfl

-- The seven axioms of Robinson's Q hold in M.
theorem q1 (x y : M) : M.succ x = M.succ y → x = y := by
  cases x <;> cases y <;> simp [M.succ] <;> omega
theorem q2 (x : M) : M.std 0 ≠ M.succ x := by
  cases x <;> simp [M.succ]
theorem q3 (x : M) : x = .std 0 ∨ ∃ y, x = M.succ y := by
  cases x with
  | std n => cases n with
    | zero => exact .inl rfl
    | succ n => exact .inr ⟨.std n, rfl⟩
  | ns i => exact .inr ⟨.ns (i - 1), by simp [M.succ]⟩
theorem add_zero (x : M) : M.add x (.std 0) = x := by simp [M.add, iter]
theorem add_succ (x y : M) : M.add x (M.succ y) = M.succ (M.add x y) := by
  cases y with
  | std n => exact iter_succ n x
  | ns i => rfl
theorem mul_zero (x : M) : M.mul x (.std 0) = .std 0 := by
  cases x with
  | std m => simp [M.mul]
  | ns j => simp [M.mul]
theorem mul_succ (x y : M) : M.mul x (M.succ y) = M.add (M.mul x y) x := by
  cases x with
  | std m =>
    cases y with
    | std n => simp [M.mul, M.succ, M.add, iter_std, Nat.mul_succ]
    | ns i =>
      cases m with
      | zero => simp [M.mul, M.succ, M.add, iter_std]
      | succ m =>
        simp only [M.mul, M.succ, M.add, iter_ns]
        congr 1; push_cast; rw [Int.mul_add]; omega
  | ns j =>
    cases y with
    | std n => simp [M.mul, M.succ, M.add]
    | ns i => simp [M.mul, M.succ, M.add]

/-- The seven axioms of Robinson's Q (Q1–Q3, add_zero, add_succ, mul_zero,
mul_succ) hold in M, with 0 := std 0 and S := succ. -/
theorem model_of_Q :
    (∀ x y, M.succ x = M.succ y → x = y) ∧ (∀ x, M.std 0 ≠ M.succ x) ∧
    (∀ x, x = .std 0 ∨ ∃ y, x = M.succ y) ∧ (∀ x, M.add x (.std 0) = x) ∧
    (∀ x y, M.add x (M.succ y) = M.succ (M.add x y)) ∧ (∀ x, M.mul x (.std 0) = .std 0) ∧
    (∀ x y, M.mul x (M.succ y) = M.add (M.mul x y) x) :=
  ⟨q1, q2, q3, add_zero, add_succ, mul_zero, mul_succ⟩

/-- In M, 0 divides 7 (witness the nonstandard element), and 0 is neither 1 nor 7:
the unbounded definition of "7 is prime" is false in a model of Q, so Q cannot
prove it. -/
theorem unbounded_fails_for_7 :
    ¬ (∀ d : M, (∃ k, M.std 7 = M.mul d k) → d = .std 1 ∨ d = .std 7) := by
  intro h
  have := h (.std 0) ⟨.ns 0, rfl⟩
  simp at this

-- Part 2: over ℕ the bounded definition is the standard definition.
def PrimeB (p : Nat) : Prop := 1 < p ∧ ∀ d, (1 < d ∧ d < p) → ¬ d ∣ p
def PrimeStd (p : Nat) : Prop := 2 ≤ p ∧ ∀ d, d ∣ p → d = 1 ∨ d = p

theorem primeB_iff (p : Nat) : PrimeB p ↔ PrimeStd p := by
  constructor
  · intro ⟨h1, h2⟩
    refine ⟨h1, fun d hd => ?_⟩
    have hle : d ≤ p := Nat.le_of_dvd (by omega) hd
    have hd0 : d ≠ 0 := by rintro rfl; simp at hd; omega
    by_cases e1 : d = 1
    · exact .inl e1
    · by_cases e2 : d = p
      · exact .inr e2
      · exact absurd hd (h2 d ⟨by omega, by omega⟩)
  · intro ⟨h1, h2⟩
    refine ⟨h1, fun d ⟨hd1, hd2⟩ hd => ?_⟩
    rcases h2 d hd with h | h <;> omega

/-- The guarded form, "p > 1 and its only positive divisors are 1 and p"
(textbook wording), also equals the standard definition over ℕ. -/
def PrimeG (p : Nat) : Prop := 1 < p ∧ ∀ d, (0 < d ∧ d ∣ p) → d = 1 ∨ d = p

theorem primeG_iff (p : Nat) : PrimeG p ↔ PrimeStd p := by
  constructor
  · intro ⟨h1, h2⟩
    refine ⟨h1, fun d hd => h2 d ⟨?_, hd⟩⟩
    rcases Nat.eq_zero_or_pos d with rfl | h
    · simp at hd; omega
    · exact h
  · intro ⟨h1, h2⟩
    exact ⟨h1, fun d ⟨_, hd⟩ => h2 d hd⟩

/-- In the model M, the guarded form is not refuted at 7 by the non-standard
element: ns(i) is positive there but divides no standard numeral. -/
theorem guarded_ns_not_dvd_7 (i : Int) (k : M) : M.std 7 ≠ M.mul (.ns i) k := by
  cases k with
  | std n => by_cases h : n = 0 <;> simp [M.mul, h]
  | ns j => simp [M.mul]

-- RLT's `<` and `∣` are these over ℕ.
theorem lt_iff (x y : Nat) : (∃ z, (z + 1) + x = y) ↔ x < y :=
  ⟨fun ⟨z, h⟩ => by omega, fun h => ⟨y - x - 1, by omega⟩⟩
theorem dvd_iff (d n : Nat) : (∃ k, n = d * k) ↔ d ∣ n := Iff.rfl

end RLT007S2Design


namespace RLT007S2Design

/-- The 47 primes below 213, grouped as the six approved ranges. -/
def R1 : List Nat := [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37]
def R2 : List Nat := [41, 43, 47, 53, 59, 61, 67, 71, 73]
def R3 : List Nat := [79, 83, 89, 97, 101, 103, 107, 109]
def R4 : List Nat := [113, 127, 131, 137, 139, 149, 151]
def R5 : List Nat := [157, 163, 167, 173, 179, 181]
def R6 : List Nat := [191, 193, 197, 199, 211]

theorem primeB_bounded (p : Nat) : PrimeB p ↔ (1 < p ∧ ∀ d, d < p → 1 < d → ¬ d ∣ p) :=
  ⟨fun ⟨h1, h2⟩ => ⟨h1, fun d h3 h4 => h2 d ⟨h4, h3⟩⟩, fun ⟨h1, h2⟩ => ⟨h1, fun d ⟨h3, h4⟩ => h2 d h4 h3⟩⟩

set_option maxRecDepth 100000 in
/-- Below 213, the primes (bounded definition) are exactly the six ranges. -/
theorem ranges_facts :
    (R1 ++ R2 ++ R3 ++ R4 ++ R5 ++ R6).Nodup ∧ (∀ q ∈ R1 ++ R2 ++ R3 ++ R4 ++ R5 ++ R6, q < 213) ∧
    (R1 ++ R2 ++ R3 ++ R4 ++ R5 ++ R6).length = 47 ∧
    R1 = (List.range 41).filter (fun p => decide (1 < p ∧ ∀ d, d < p → 1 < d → ¬ d ∣ p)) ∧
    R2 = ((List.range 79).filter (fun p => decide (41 ≤ p ∧ 1 < p ∧ ∀ d, d < p → 1 < d → ¬ d ∣ p))) ∧
    R3 = ((List.range 113).filter (fun p => decide (79 ≤ p ∧ 1 < p ∧ ∀ d, d < p → 1 < d → ¬ d ∣ p))) ∧
    R4 = ((List.range 157).filter (fun p => decide (113 ≤ p ∧ 1 < p ∧ ∀ d, d < p → 1 < d → ¬ d ∣ p))) ∧
    R5 = ((List.range 191).filter (fun p => decide (157 ≤ p ∧ 1 < p ∧ ∀ d, d < p → 1 < d → ¬ d ∣ p))) ∧
    R6 = ((List.range 213).filter (fun p => decide (191 ≤ p ∧ 1 < p ∧ ∀ d, d < p → 1 < d → ¬ d ∣ p))) := by
  decide

set_option maxRecDepth 100000 in
theorem classification : ∀ p, p < 213 →
    ((1 < p ∧ ∀ d, d < p → 1 < d → ¬ d ∣ p) ↔ p ∈ R1 ++ R2 ++ R3 ++ R4 ++ R5 ++ R6) := by
  decide

end RLT007S2Design

#print axioms RLT007S2Design.iter_std
#print axioms RLT007S2Design.iter_ns
#print axioms RLT007S2Design.iter_succ
#print axioms RLT007S2Design.q1
#print axioms RLT007S2Design.q2
#print axioms RLT007S2Design.q3
#print axioms RLT007S2Design.add_zero
#print axioms RLT007S2Design.add_succ
#print axioms RLT007S2Design.mul_zero
#print axioms RLT007S2Design.mul_succ
#print axioms RLT007S2Design.model_of_Q
#print axioms RLT007S2Design.unbounded_fails_for_7
#print axioms RLT007S2Design.primeB_iff
#print axioms RLT007S2Design.primeG_iff
#print axioms RLT007S2Design.guarded_ns_not_dvd_7
#print axioms RLT007S2Design.lt_iff
#print axioms RLT007S2Design.dvd_iff
#print axioms RLT007S2Design.primeB_bounded
#print axioms RLT007S2Design.ranges_facts
#print axioms RLT007S2Design.classification

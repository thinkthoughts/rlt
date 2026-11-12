/-
RLT-005, Lean comparison 2 of 2: from RLT's explicit-partner formulation to
the standard definition of a matchable number. Hand-written; core Lean only.
Run: cd lean && lean RLT005Readings.lean

Source: N. McNew and C. Pomerance, Matchable numbers, arXiv:2604.05304v2,
§1 (registry id MCNEW-POMERANCE-MATCHABLE-V2): for sets A, B, a bijection ψ
between them is a coprime matching if x and ψ(x) are coprime for each x in
the domain of ψ; for a positive integer n, D(n) is the set of divisors of n
and τ(n) = |D(n)|; n is matchable if there is a coprime matching between
{1, 2, …, τ(n)} and D(n).

Standard formulation here (core Lean has no Finset or Equiv, so finite sets
are duplicate-free lists and a bijection is a function that maps A into B,
is injective on A and onto B):
  * `divisors n` filters 1..n by divisibility; `mem_divisors` proves that for
    n > 0 its members are exactly the d with 0 < d ∧ d ∣ n;
  * `tau n` is its length; `Nat.Coprime` is gcd = 1;
  * `matchable n` takes ψ with domain {1, …, τ(n)}, as the paper's wording
    lists that set first; `matching_symm` proves the direction immaterial.

Correspondence proved: `explicit_iff` (any four distinct positive divisors
against {1, 2, 3, 4}) is the translation RLT's encoding relies on: partner
variables a, b, c, d, pairwise distinctness and RLT's coprimality formula
`cop` express exactly a coprime bijection. It is non-vacuous: `six_explicit`
and `matchable_six` instantiate it where both sides hold. Lean-checked
statements; they establish no RLT derivation.
-/

namespace RLT005Readings

-- The definitions of RLT005Direct.lean, copied verbatim so that this file
-- checks on its own (core Lean, no build system); scripts/verify.sh refuses
-- any difference between the two copies.
-- BEGIN SHARED RLT-005 BLOCK (byte-identical in RLT005Readings.lean; scripts/verify.sh compares)
/-- RLT's coprimality formula: every positive common divisor is 1. -/
def cop (m x : Nat) : Prop := ∀ k, ((0 < k ∧ k ∣ m) ∧ k ∣ x) → k = 1

/-- For m > 0, RLT's coprimality formula is gcd = 1. -/
theorem cop_iff_gcd (m x : Nat) (hm : 0 < m) : cop m x ↔ Nat.gcd m x = 1 := by
  constructor
  · intro h
    exact h _ ⟨⟨Nat.gcd_pos_of_pos_left x hm, Nat.gcd_dvd_left m x⟩, Nat.gcd_dvd_right m x⟩
  · intro h k ⟨⟨_, hkm⟩, hkx⟩
    have := Nat.dvd_gcd hkm hkx
    rw [h] at this
    exact Nat.dvd_one.mp this

/-- The partner ranges: 0 < x ∧ x < 5. -/
def r (x : Nat) : Prop := 0 < x ∧ x < 5

/-- The matching conditions, nested exactly as in RLT's `Mfor`. -/
def M (a b c d : Nat) : Prop :=
  ((((r a ∧ r b) ∧ r c) ∧ r d) ∧
    (((((¬ a = b ∧ ¬ a = c) ∧ ¬ a = d) ∧ ¬ b = c) ∧ ¬ b = d) ∧ ¬ c = d)) ∧
  (((cop 1 a ∧ cop 2 b) ∧ cop 4 c) ∧ cop 8 d)

/-- 0 < x < 5 places x in the list [1, 2, 3, 4]. -/
theorem mem_partners (x : Nat) (h : r x) : x ∈ [1, 2, 3, 4] := by
  have : x = 1 ∨ x = 2 ∨ x = 3 ∨ x = 4 := by unfold r at h; omega
  rcases this with h | h | h | h <;> subst h <;> decide

/-- The same conditions with gcd, over the partner list: decided over 4⁴
cases. -/
theorem bounded_none :
    ∀ a ∈ [1, 2, 3, 4], ∀ b ∈ [1, 2, 3, 4], ∀ c ∈ [1, 2, 3, 4], ∀ d ∈ [1, 2, 3, 4],
      ¬ ((((((¬ a = b ∧ ¬ a = c) ∧ ¬ a = d) ∧ ¬ b = c) ∧ ¬ b = d) ∧ ¬ c = d) ∧
        (((Nat.gcd 1 a = 1 ∧ Nat.gcd 2 b = 1) ∧ Nat.gcd 4 c = 1) ∧ Nat.gcd 8 d = 1)) := by
  decide

/-- RLT's `matchable.not.8.contradiction`: no assignment meets M. -/
theorem no_assignment : ∀ a b c d, ¬ M a b c d := by
  intro a b c d ⟨⟨⟨⟨⟨ra, rb⟩, rc⟩, rd⟩, dist⟩, ⟨⟨⟨ca, cb⟩, cc⟩, cd⟩⟩
  rw [cop_iff_gcd 1 a (by decide)] at ca
  rw [cop_iff_gcd 2 b (by decide)] at cb
  rw [cop_iff_gcd 4 c (by decide)] at cc
  rw [cop_iff_gcd 8 d (by decide)] at cd
  exact bounded_none a (mem_partners a ra) b (mem_partners b rb) c (mem_partners c rc)
    d (mem_partners d rd) ⟨dist, ⟨⟨⟨ca, cb⟩, cc⟩, cd⟩⟩

-- END SHARED RLT-005 BLOCK

/-- D(n): the divisors of n among 1..n. -/
def divisors (n : Nat) : List Nat := (List.range' 1 n).filter (fun d => decide (d ∣ n))

/-- τ(n) = |D(n)|. -/
def tau (n : Nat) : Nat := (divisors n).length

/-- For n > 0, D(n) is the set of positive divisors of n. -/
theorem mem_divisors (n d : Nat) (hn : 0 < n) : d ∈ divisors n ↔ 0 < d ∧ d ∣ n := by
  unfold divisors
  rw [List.mem_filter, List.mem_range'_1, decide_eq_true_iff]
  constructor
  · intro ⟨⟨h1, _⟩, h⟩; exact ⟨h1, h⟩
  · intro ⟨h1, h⟩; exact ⟨⟨h1, by have := Nat.le_of_dvd hn h; omega⟩, h⟩

/-- D(n) has no repeated entries, so its length is |D(n)|. -/
theorem divisors_nodup (n : Nat) : (divisors n).Nodup :=
  (List.nodup_range').filter _

/-- A coprime matching from A onto B: a bijection A → B (maps into B,
injective on A, onto B) with x and ψ(x) coprime for every x in A. -/
def CoprimeMatching (A B : List Nat) (f : Nat → Nat) : Prop :=
  (∀ x ∈ A, f x ∈ B) ∧ (∀ x ∈ A, ∀ y ∈ A, f x = f y → x = y) ∧
    (∀ y ∈ B, ∃ x ∈ A, f x = y) ∧ (∀ x ∈ A, Nat.Coprime x (f x))

/-- McNew–Pomerance: n is matchable if there is a coprime matching between
{1, …, τ(n)} and D(n). -/
def matchable (n : Nat) : Prop :=
  ∃ f, CoprimeMatching (List.range' 1 (tau n)) (divisors n) f

/-- The same with ψ defined on D(n). -/
def matchable' (n : Nat) : Prop :=
  ∃ f, CoprimeMatching (divisors n) (List.range' 1 (tau n)) f

/-- The inverse of a coprime matching is a coprime matching. -/
theorem matching_symm (A B : List Nat) (f : Nat → Nat) (h : CoprimeMatching A B f) :
    ∃ g, CoprimeMatching B A g := by
  obtain ⟨hmap, hinj, hsurj, hcop⟩ := h
  classical
  let g : Nat → Nat := fun y => if hy : ∃ x ∈ A, f x = y then Classical.choose hy else 0
  have spec : ∀ y ∈ B, g y ∈ A ∧ f (g y) = y := by
    intro y hy
    have e := hsurj y hy
    simp only [g]
    split
    · next h => exact Classical.choose_spec h
    · next h => exact absurd e h
  refine ⟨g, fun y hy => (spec y hy).1, ?_, ?_, ?_⟩
  · intro y hy y' hy' h
    rw [← (spec y hy).2, ← (spec y' hy').2, h]
  · intro x hx
    refine ⟨f x, hmap x hx, ?_⟩
    have := spec (f x) (hmap x hx)
    exact hinj _ this.1 _ hx this.2
  · intro y hy
    have := hcop (g y) (spec y hy).1
    rw [(spec y hy).2] at this
    exact Nat.Coprime.symm this

/-- The direction of ψ is immaterial. -/
theorem matchable_iff (n : Nat) : matchable n ↔ matchable' n :=
  ⟨fun ⟨f, h⟩ => matching_symm _ _ f h, fun ⟨f, h⟩ => matching_symm _ _ f h⟩

/-- RLT's matching conditions for divisors p, q, s, t (RLT's M is the case
1, 2, 4, 8). -/
def Mgen (p q s t a b c d : Nat) : Prop :=
  ((((r a ∧ r b) ∧ r c) ∧ r d) ∧
    (((((¬ a = b ∧ ¬ a = c) ∧ ¬ a = d) ∧ ¬ b = c) ∧ ¬ b = d) ∧ ¬ c = d)) ∧
  (((cop p a ∧ cop q b) ∧ cop s c) ∧ cop t d)

theorem M_is_Mgen (a b c d : Nat) : M a b c d ↔ Mgen 1 2 4 8 a b c d := Iff.rfl

theorem r_iff_mem (x : Nat) : r x ↔ x ∈ [1, 2, 3, 4] := by
  constructor
  · intro h
    have : x = 1 ∨ x = 2 ∨ x = 3 ∨ x = 4 := by unfold r at h; omega
    rcases this with h | h | h | h <;> subst h <;> decide
  · intro h
    simp only [List.mem_cons, List.mem_nil_iff, or_false] at h
    unfold r; omega

theorem coprime_iff_cop (m x : Nat) (hm : 0 < m) : Nat.Coprime m x ↔ cop m x :=
  (cop_iff_gcd m x hm).symm

/-- Four distinct values in {1, 2, 3, 4} cover it. -/
theorem cover :
    ∀ a ∈ [1, 2, 3, 4], ∀ b ∈ [1, 2, 3, 4], ∀ c ∈ [1, 2, 3, 4], ∀ d ∈ [1, 2, 3, 4],
      ((((¬ a = b ∧ ¬ a = c) ∧ ¬ a = d) ∧ ¬ b = c) ∧ ¬ b = d) ∧ ¬ c = d →
      ∀ y ∈ [1, 2, 3, 4], y = a ∨ y = b ∨ y = c ∨ y = d := by
  decide

/-- The translation: for four distinct positive numbers p, q, s, t, a coprime
matching from [p, q, s, t] onto {1, 2, 3, 4} exists exactly when RLT's
explicit-partner conditions have a solution. -/
theorem explicit_iff (p q s t : Nat) (hp : 0 < p) (hq : 0 < q) (hs : 0 < s) (ht : 0 < t)
    (hpq : p ≠ q) (hps : p ≠ s) (hpt : p ≠ t) (hqs : q ≠ s) (hqt : q ≠ t) (hst : s ≠ t) :
    (∃ f, CoprimeMatching [p, q, s, t] [1, 2, 3, 4] f) ↔ ∃ a b c d, Mgen p q s t a b c d := by
  constructor
  · intro ⟨f, hmap, hinj, _, hcop⟩
    have mp : p ∈ [p, q, s, t] := by simp
    have mq : q ∈ [p, q, s, t] := by simp
    have ms : s ∈ [p, q, s, t] := by simp
    have mt : t ∈ [p, q, s, t] := by simp
    refine ⟨f p, f q, f s, f t, ⟨⟨⟨⟨?_, ?_⟩, ?_⟩, ?_⟩, ?_⟩, ⟨⟨⟨?_, ?_⟩, ?_⟩, ?_⟩⟩
    · exact (r_iff_mem _).mpr (hmap p mp)
    · exact (r_iff_mem _).mpr (hmap q mq)
    · exact (r_iff_mem _).mpr (hmap s ms)
    · exact (r_iff_mem _).mpr (hmap t mt)
    · refine ⟨⟨⟨⟨⟨?_, ?_⟩, ?_⟩, ?_⟩, ?_⟩, ?_⟩
      · exact fun h => hpq (hinj _ mp _ mq h)
      · exact fun h => hps (hinj _ mp _ ms h)
      · exact fun h => hpt (hinj _ mp _ mt h)
      · exact fun h => hqs (hinj _ mq _ ms h)
      · exact fun h => hqt (hinj _ mq _ mt h)
      · exact fun h => hst (hinj _ ms _ mt h)
    · exact (coprime_iff_cop _ _ hp).mp (hcop p mp)
    · exact (coprime_iff_cop _ _ hq).mp (hcop q mq)
    · exact (coprime_iff_cop _ _ hs).mp (hcop s ms)
    · exact (coprime_iff_cop _ _ ht).mp (hcop t mt)
  · intro ⟨a, b, c, d, ⟨⟨⟨⟨ra, rb⟩, rc⟩, rd⟩, dist⟩, ⟨⟨⟨ca, cb⟩, cc⟩, cd⟩⟩
    obtain ⟨⟨⟨⟨⟨dab, dac⟩, dad⟩, dbc⟩, dbd⟩, dcd⟩ := dist
    let f : Nat → Nat := fun x => if x = p then a else if x = q then b else if x = s then c else d
    have fp : f p = a := by simp [f]
    have fq : f q = b := by simp [f, Ne.symm hpq]
    have fs : f s = c := by simp [f, Ne.symm hps, Ne.symm hqs]
    have ft : f t = d := by simp [f, Ne.symm hpt, Ne.symm hqt, Ne.symm hst]
    have cases4 : ∀ x ∈ [p, q, s, t], x = p ∨ x = q ∨ x = s ∨ x = t := by
      intro x hx; simpa using hx
    refine ⟨f, ?_, ?_, ?_, ?_⟩
    · intro x hx
      rcases cases4 x hx with h | h | h | h <;> subst h
      · rw [fp]; exact (r_iff_mem _).mp ra
      · rw [fq]; exact (r_iff_mem _).mp rb
      · rw [fs]; exact (r_iff_mem _).mp rc
      · rw [ft]; exact (r_iff_mem _).mp rd
    · intro x hx y hy hxy
      rcases cases4 x hx with h | h | h | h <;> subst h <;>
        rcases cases4 y hy with h' | h' | h' | h' <;> subst h' <;>
        simp only [fp, fq, fs, ft] at hxy <;> omega
    · intro y hy
      have := cover a ((r_iff_mem _).mp ra) b ((r_iff_mem _).mp rb) c ((r_iff_mem _).mp rc)
        d ((r_iff_mem _).mp rd) ⟨⟨⟨⟨⟨dab, dac⟩, dad⟩, dbc⟩, dbd⟩, dcd⟩ y hy
      rcases this with h | h | h | h <;> subst h
      · exact ⟨p, by simp, fp⟩
      · exact ⟨q, by simp, fq⟩
      · exact ⟨s, by simp, fs⟩
      · exact ⟨t, by simp, ft⟩
    · intro x hx
      rcases cases4 x hx with h | h | h | h <;> subst h
      · rw [fp]; exact (coprime_iff_cop _ _ hp).mpr ca
      · rw [fq]; exact (coprime_iff_cop _ _ hq).mpr cb
      · rw [fs]; exact (coprime_iff_cop _ _ hs).mpr cc
      · rw [ft]; exact (coprime_iff_cop _ _ ht).mpr cd

theorem divisors_eight : divisors 8 = [1, 2, 4, 8] := by decide
theorem tau_eight : tau 8 = 4 := by decide
theorem divisors_six : divisors 6 = [1, 2, 3, 6] := by decide
theorem tau_six : tau 6 = 4 := by decide

/-- For n = 8 the standard definition (ψ on D(8)) is RLT's explicit
formulation. -/
theorem matchable'_eight_iff : matchable' 8 ↔ ∃ a b c d, M a b c d := by
  unfold matchable'
  rw [tau_eight, divisors_eight]
  exact explicit_iff 1 2 4 8 (by decide) (by decide) (by decide) (by decide)
    (by decide) (by decide) (by decide) (by decide) (by decide) (by decide)

/-- 8 is not matchable, in the paper's formulation. -/
theorem not_matchable_eight : ¬ matchable 8 := by
  rw [matchable_iff, matchable'_eight_iff]
  exact fun ⟨a, b, c, d, h⟩ => no_assignment a b c d h

/-- Non-vacuity of the translation: for D(6) = {1, 2, 3, 6} the explicit
conditions hold (partners 4, 3, 2, 1). -/
theorem six_explicit : ∃ a b c d, Mgen 1 2 3 6 a b c d := by
  refine ⟨4, 3, 2, 1, ⟨by unfold r; decide, by decide⟩, ?_⟩
  refine ⟨⟨⟨?_, ?_⟩, ?_⟩, ?_⟩ <;> exact (cop_iff_gcd _ _ (by decide)).mpr (by decide)

theorem matchable_six : matchable 6 := by
  rw [matchable_iff]
  unfold matchable'
  rw [tau_six, divisors_six]
  exact (explicit_iff 1 2 3 6 (by decide) (by decide) (by decide) (by decide)
    (by decide) (by decide) (by decide) (by decide) (by decide) (by decide)).mpr six_explicit

/-- The paper's remark that 1, 2, …, 7 are matchable, checked for the
definition here with explicit matchings ψ(1), …, ψ(τ(n)). -/
theorem matchable_one_to_seven :
    matchable 1 ∧ matchable 2 ∧ matchable 3 ∧ matchable 4 ∧ matchable 5 ∧ matchable 6 ∧
      matchable 7 := by
  refine ⟨⟨fun x => [1].getD (x - 1) 0, by unfold CoprimeMatching Nat.Coprime; decide⟩, ⟨fun x => [2, 1].getD (x - 1) 0, by unfold CoprimeMatching Nat.Coprime; decide⟩,
    ⟨fun x => [1, 3].getD (x - 1) 0, by unfold CoprimeMatching Nat.Coprime; decide⟩, ⟨fun x => [2, 1, 4].getD (x - 1) 0, by unfold CoprimeMatching Nat.Coprime; decide⟩,
    ⟨fun x => [1, 5].getD (x - 1) 0, by unfold CoprimeMatching Nat.Coprime; decide⟩, matchable_six,
    ⟨fun x => [1, 7].getD (x - 1) 0, by unfold CoprimeMatching Nat.Coprime; decide⟩⟩

end RLT005Readings

#print axioms RLT005Readings.mem_divisors
#print axioms RLT005Readings.divisors_nodup
#print axioms RLT005Readings.matching_symm
#print axioms RLT005Readings.matchable_iff
#print axioms RLT005Readings.M_is_Mgen
#print axioms RLT005Readings.r_iff_mem
#print axioms RLT005Readings.coprime_iff_cop
#print axioms RLT005Readings.cover
#print axioms RLT005Readings.explicit_iff
#print axioms RLT005Readings.divisors_eight
#print axioms RLT005Readings.tau_eight
#print axioms RLT005Readings.divisors_six
#print axioms RLT005Readings.tau_six
#print axioms RLT005Readings.matchable'_eight_iff
#print axioms RLT005Readings.not_matchable_eight
#print axioms RLT005Readings.six_explicit
#print axioms RLT005Readings.matchable_six
#print axioms RLT005Readings.matchable_one_to_seven
#print axioms RLT005Readings.cop_iff_gcd
#print axioms RLT005Readings.mem_partners
#print axioms RLT005Readings.bounded_none
#print axioms RLT005Readings.no_assignment

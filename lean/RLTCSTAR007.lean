/-
RLT-CSTAR-007 · Lean 4 companion (core Lean 4, no Mathlib), self-contained.

The definitions repeat those of RLTCSTAR002.lean (transcriptions, by hand, of the integer
formulas of ucla-roms 0.16.0 `mpi_setup`, `partit` and `init_coarse`). As there, these
theorems are about the transcriptions; that each source computes them is established by
executable comparison (RLT-CSTAR-007-VERIFICATION.md), never by this file.

New statements:
  * `init_coarse`'s own test (`3 ≤ ceil`) does not keep the edge coarse tiles non-empty
    (`guard_passes_edge_empty`, the case 150 coarse points / 21 tiles);
  * the amended Rule L of C-Star (`3 ≤ ceil` and a non-empty narrowest tile) gives every
    tile at least one point, and every middle tile at least three (`ruleL_tiles`);
  * the size test of ROMS-Tools' join: tiles that split `q` interior coarse points (plus
    the two boundary points) can never all have the whole size `q + 2` once there are two
    or more of them (`split_never_all_whole`), so copying a dimension whose tiles all
    have the whole size never discards a split one.
Natural-number subtraction truncates at 0, so a tile the sources leave with `-1` points
appears here as `0`.
-/

namespace RLTCSTAR007

def ceilDiv (p t : Nat) : Nat := (p + t - 1) / t
def surplus (p t : Nat) : Nat := t * ceilDiv p t - p
def firstW (p t : Nat) : Nat := ceilDiv p t - surplus p t / 2
def lastW (p t : Nat) : Nat := ceilDiv p t - (surplus p t + 1) / 2

/-- Width of tile `k` of `t` (0-based). -/
def width (p t k : Nat) : Nat :=
  if k = 0 then firstW p t else if k + 1 = t then lastW p t else ceilDiv p t

theorem ceilDiv_bounds (p t : Nat) (ht : 0 < t) :
    p ≤ t * ceilDiv p t ∧ t * ceilDiv p t < p + t := by
  unfold ceilDiv
  have h1 := Nat.div_add_mod (p + t - 1) t
  have h2 := Nat.mod_lt (p + t - 1) ht
  omega

theorem lastW_le_firstW (p t : Nat) : lastW p t ≤ firstW p t := by
  unfold lastW firstW; omega

/-- ucla-roms' test passes (ceil = 8 ≥ 3) and both edge tiles are empty. -/
theorem guard_passes_edge_empty :
    3 ≤ ceilDiv 150 21 ∧ firstW 150 21 = 0 ∧ lastW 150 21 = 0 ∧ surplus 150 21 = 18 := by
  decide

/-- The amended Rule L: with `3 ≤ ceil` and a non-empty last tile, every tile is
non-empty and every middle tile holds at least three points. -/
theorem ruleL_tiles (p t k : Nat) (_hk : k < t)
    (hc : 3 ≤ ceilDiv p t) (he : 0 < lastW p t) :
    0 < width p t k ∧ (k ≠ 0 → k + 1 ≠ t → 3 ≤ width p t k) := by
  have hle := lastW_le_firstW p t
  unfold width
  by_cases h0 : k = 0
  · simp [h0]; omega
  · by_cases hl : k + 1 = t
    · simp [h0, hl]; omega
    · simp [h0, hl]; omega

/-- `init_coarse`'s test alone does not give that conclusion. -/
theorem guard_alone_insufficient :
    ¬ (∀ p t k : Nat, k < t → 3 ≤ ceilDiv p t → 0 < width p t k) := by
  intro h
  have := h 150 21 0 (by decide) (by decide)
  revert this; decide

/-- Split tiles cover the interior: first + (t - 2) middle + last = p (for t ≥ 2 and a
non-empty last tile). -/
theorem cover (p t : Nat) (ht : 2 ≤ t) (he : 0 < lastW p t) :
    firstW p t + (t - 2) * ceilDiv p t + lastW p t = p := by
  have hb := ceilDiv_bounds p t (by omega)
  unfold lastW at he
  unfold firstW lastW surplus at *
  have : (t - 2) * ceilDiv p t + 2 * ceilDiv p t = t * ceilDiv p t := by
    rw [← Nat.add_mul]; congr 1; omega
  omega

/-- The join's size test. Split coarse tiles of `q` interior points carry `q + 2` points
in all (the two boundary points on the edge tiles); `t ≥ 2` tiles all of the whole size
`q + 2` would carry `t * (q + 2)`, which is more. So a split dimension never has every
tile of the whole size. -/
theorem split_never_all_whole (q t : Nat) (ht : 2 ≤ t) (he : 0 < lastW q t) :
    (firstW q t + 1) + (t - 2) * ceilDiv q t + (lastW q t + 1) = q + 2 ∧
    q + 2 < t * (q + 2) := by
  have hc := cover q t ht he
  constructor
  · omega
  · have : 2 * (q + 2) ≤ t * (q + 2) := Nat.mul_le_mul_right _ ht
    omega

end RLTCSTAR007

#print axioms RLTCSTAR007.guard_passes_edge_empty
#print axioms RLTCSTAR007.ruleL_tiles
#print axioms RLTCSTAR007.guard_alone_insufficient
#print axioms RLTCSTAR007.cover
#print axioms RLTCSTAR007.split_never_all_whole

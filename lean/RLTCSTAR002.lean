/-
RLT-CSTAR-002 · Lean 4 companion (core Lean 4, no Mathlib).

General statements about the domain decomposition ucla-roms uses (`mpi_setup` in
src/roms_mpi.F90, `mpi_setup` in Tools-Roms/partit.F and `init_coarse` in
src/roms_read_write.F90, ucla-roms 0.16.0 f83d2ef6), C-Star mirrors
(`_edge_tile_width`, `check_tile_sizes`, `coarse_tile_width`) and ROMS-Tools now
follows (`_interior_widths`).

The definitions below transcribe the integer formulas of those sources into Lean
by hand. These theorems are about the transcriptions: that each source computes
them is established separately, by the executable comparison in
RLT-CSTAR-002-VERIFICATION.md, never by this file.

Notation for points `p` split into `t` tiles:
  w = ceilDiv p t            -- ceiling(p / t): nx = (gnx + npx - 1) / npx
  s = surplus p t            -- offx = npx * nx - gnx
  first tile  w - s / 2      -- nx - offx/2
  middle      w
  last tile   w - (s + 1) / 2  -- nx - (offx + 1)/2
  start of tile k (0-based, interior points): 0 for k = 0, k * w - s / 2 otherwise
                             -- iSW_corn
-/

namespace RLTCSTAR002

/-- `nx = (gnx + npx - 1) / npx` (ucla-roms checks it equals `ceiling(gnx / npx)`). -/
def ceilDiv (p t : Nat) : Nat := (p + t - 1) / t

/-- `offx = npx * nx - gnx`. -/
def surplus (p t : Nat) : Nat := t * ceilDiv p t - p

def firstW (p t : Nat) : Nat := ceilDiv p t - surplus p t / 2
def lastW (p t : Nat) : Nat := ceilDiv p t - (surplus p t + 1) / 2

/-- `iSW_corn`: interior points before tile `k`. -/
def start (p t k : Nat) : Nat := if k = 0 then 0 else k * ceilDiv p t - surplus p t / 2

/-- The division facts everything else rests on. -/
theorem ceilDiv_bounds (p t : Nat) (ht : 0 < t) :
    p ≤ t * ceilDiv p t ∧ t * ceilDiv p t < p + t := by
  unfold ceilDiv
  have h1 := Nat.div_add_mod (p + t - 1) t
  have h2 := Nat.mod_lt (p + t - 1) ht
  omega

/-- The surplus is smaller than the tile count. -/
theorem surplus_lt (p t : Nat) (ht : 0 < t) : surplus p t < t := by
  have := ceilDiv_bounds p t ht
  unfold surplus; omega

/-- `surplus` is exact: `t * w = p + s`. -/
theorem mul_ceilDiv_eq (p t : Nat) (ht : 0 < t) : t * ceilDiv p t = p + surplus p t := by
  have := ceilDiv_bounds p t ht
  unfold surplus; omega

/-- Agreement on even splits: when `t ∣ p` there is no surplus and every tile has
`p / t` points, the old ROMS-Tools `core = (size - ghost) // nparts`. -/
theorem even_split (p t : Nat) (ht : 0 < t) (hd : t ∣ p) :
    surplus p t = 0 ∧ ceilDiv p t = p / t := by
  obtain ⟨q, rfl⟩ := hd
  have hq : (t * q + t - 1) / t = q := by
    have : t * q + t - 1 = (t - 1) + t * q := by omega
    rw [this, Nat.add_mul_div_left _ _ ht, Nat.div_eq_of_lt (by omega)]
    simp
  have hd' : t * q / t = q := Nat.mul_div_cancel_left q ht
  unfold surplus ceilDiv
  rw [hq, hd']
  simp

/-- C-Star's acceptance condition: the last (narrowest) tile is non-empty exactly
when `s + 2 ≤ 2 w`. -/
theorem lastW_pos_iff (p t : Nat) : 0 < lastW p t ↔ surplus p t + 2 ≤ 2 * ceilDiv p t := by
  unfold lastW; omega

/-- The last tile is never wider than the first (ucla-roms: "If the surplus is odd,
the last subdomain will be the smallest"). -/
theorem lastW_le_firstW (p t : Nat) : lastW p t ≤ firstW p t := by
  unfold lastW firstW; omega

/-- The narrowest tile is the last one. -/
theorem lastW_le_mid (p t : Nat) : lastW p t ≤ ceilDiv p t := by
  unfold lastW; omega

/-- Coverage: for `t ≥ 2` tiles of a valid tiling the widths add up to `p`. -/
theorem cover (p t : Nat) (ht : 2 ≤ t) (hv : surplus p t + 2 ≤ 2 * ceilDiv p t) :
    firstW p t + (t - 2) * ceilDiv p t + lastW p t = p := by
  have hm := mul_ceilDiv_eq p t (by omega)
  have hsplit : t * ceilDiv p t = (t - 2) * ceilDiv p t + 2 * ceilDiv p t := by
    rw [← Nat.add_mul]; congr 1; omega
  unfold firstW lastW
  omega

/-- Contiguity: tile 1 starts where tile 0 ends. -/
theorem start_one (p t : Nat) : start p t 1 = firstW p t := by
  unfold start firstW; simp

/-- Contiguity: each middle tile ends where the next starts. -/
theorem start_succ (p t k : Nat) (hk : 1 ≤ k) (hv : surplus p t / 2 ≤ ceilDiv p t) :
    start p t (k + 1) = start p t k + ceilDiv p t := by
  unfold start
  have hk0 : k ≠ 0 := by omega
  simp [hk0, Nat.add_mul]
  have : surplus p t / 2 ≤ k * ceilDiv p t := by
    have : ceilDiv p t ≤ k * ceilDiv p t := Nat.le_mul_of_pos_left _ (by omega)
    omega
  omega

/-- Contiguity: the last tile ends at `p`. -/
theorem start_last (p t : Nat) (ht : 2 ≤ t) (hv : surplus p t + 2 ≤ 2 * ceilDiv p t) :
    start p t (t - 1) + lastW p t = p := by
  have hm := mul_ceilDiv_eq p t (by omega)
  have ht1 : t - 1 ≠ 0 := by omega
  have hsplit : t * ceilDiv p t = (t - 1) * ceilDiv p t + ceilDiv p t := by
    generalize ceilDiv p t = w
    obtain ⟨u, rfl⟩ : ∃ u, t = u + 1 := ⟨t - 1, by omega⟩
    simp [Nat.add_mul]
  have hle : ceilDiv p t ≤ (t - 1) * ceilDiv p t := Nat.le_mul_of_pos_left _ (by omega)
  unfold start lastW
  simp [ht1]
  omega

/-- The coarse-grid rule (`init_coarse`, Rule L in C-Star): with `q` coarse points a
coarse tile has at least 3 points exactly when `q > 2 t`. -/
theorem coarse_ok_iff (q t : Nat) (ht : 0 < t) : 3 ≤ ceilDiv q t ↔ 2 * t < q := by
  have h := ceilDiv_bounds q t ht
  constructor
  · intro h3
    have : t * 3 ≤ t * ceilDiv q t := Nat.mul_le_mul_left t h3
    omega
  · intro hq
    apply Nat.lt_of_not_le
    intro h2
    have : t * ceilDiv q t ≤ t * 2 := Nat.mul_le_mul_left t h2
    omega

/-- A tiling C-Star's fine-grid check accepts and the coarse-grid rule refuses:
20 points on 8 tiles leave every fine tile non-empty, while the 10 coarse points
give tiles of 2. -/
theorem twenty_on_eight :
    0 < lastW 20 8 ∧ ceilDiv (20 / 2) 8 = 2 := by decide

/-- The witness of RLT-CSTAR-001: 10 points on 3 tiles are 3, 4, 3, valid, and
`3 ∤ 10`, so the old ROMS-Tools precondition failed. -/
theorem ten_on_three :
    firstW 10 3 = 3 ∧ ceilDiv 10 3 = 4 ∧ lastW 10 3 = 3 ∧ ¬ (3 ∣ 10) := by decide

end RLTCSTAR002

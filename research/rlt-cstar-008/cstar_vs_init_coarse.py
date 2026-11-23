"""C-Star's coarse functions vs the verbatim init_coarse lines (harness_coarse_min output).
Usage: PYTHONPATH=CSTAR_TREE python cstar_vs_init_coarse.py coarse_min.out"""
import sys, numpy as np, cstar
from cstar.roms.precheck import coarse_tile_width, narrowest_coarse_tile, COARSE_TILE_MIN_POINTS, check_coarse_tile_sizes
from cstar.applications.forge.namelist_model import check_tile_sizes
print("cstar from", cstar.__file__)
d = np.loadtxt(sys.argv[1], dtype=int)
bad_w = bad_n = abort = empty = mismatch = 0
for gnx, t, nxc0, nmin in d:
    bad_w += coarse_tile_width(gnx, t) != nxc0
    bad_n += narrowest_coarse_tile(gnx, t) != nmin
    stops = nxc0 <= 2; empty_tile = (not stops) and nmin <= 0
    abort += stops; empty += empty_tile
    refuses = coarse_tile_width(gnx, t) < COARSE_TILE_MIN_POINTS or narrowest_coarse_tile(gnx, t) < 1
    mismatch += refuses != (stops or empty_tile)
print(f"(llm, np_xi) pairs: {len(d)} (llm 2..{d[:,0].max()}, np_xi 1..{d[:,1].max()})")
print(f"coarse_tile_width != ROMS nxc0: {bad_w}; narrowest_coarse_tile != ROMS min tile: {bad_n}")
print(f"ROMS aborts: {abort}; ROMS runs with an empty coarse tile: {empty}; Rule L refuses exactly these: {mismatch == 0} (mismatches {mismatch})")
s = {"param_settings": {"llm": 300, "mmm": 40, "np_xi": 21, "np_eta": 1}, "surf_frc_settings": {"interp_bulk_frc": True, "interp_flux_frc": False}, "bgc_settings": {"interp_bgc_frc": False}}
try: check_coarse_tile_sizes(s, {"bulk_frc": True, "marbl": True, "biology_bec2": False}); print("Rule L accepted 300/21")
except Exception as e: print("Rule L refuses 300/21:", e)
try: check_tile_sizes({"llm": 300, "mmm": 40, "np_xi": 21, "np_eta": 1}, auto_tiling=False, interp_frc=True); print("Forge accepted 300/21")
except ValueError as e: print("Forge refuses 300/21:", e)

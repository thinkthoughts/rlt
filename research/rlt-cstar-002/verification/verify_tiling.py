"""RLT-CSTAR-002 verification: tile sizes, four implementations, exhaustive range.

Compares, for every (points, tiles) in range:
  F1  ucla-roms 0.16.0 mpi_setup      (compiled verbatim excerpt: harness_mpi)
  F2  ucla-roms Tools-Roms/partit.F   (compiled verbatim subroutine: harness_partit)
  F3  ucla-roms init_coarse           (compiled verbatim excerpt: harness_coarse)
  C   C-Star _edge_tile_width / check_tile_sizes (installed package, imported)
  R   ROMS-Tools _compute_partition_sizes (installed package under test, imported)
Then exercises ROMS-Tools partition + join on synthetic datasets.
Exhaustive testing over a finite range: evidence, never a proof of the general claim.
"""

import subprocess
import sys
from collections import defaultdict

import numpy as np
import xarray as xr

from cstar.applications.forge.namelist_model import _edge_tile_width, check_tile_sizes
from roms_tools.tiling.join import join_datasets
from roms_tools.tiling.partition import _compute_partition_sizes, partition

FDIR, P, T = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])


def run(exe, *args):
    out = subprocess.run([f"{FDIR}/{exe}", *map(str, args)], check=True, capture_output=True, text=True).stdout
    return [list(map(int, line.split())) for line in out.splitlines() if line.strip()]


def table(rows, key_cols, val_cols):
    t = defaultdict(list)
    for r in rows:
        t[tuple(r[i] for i in key_cols)].append(tuple(r[i] for i in val_cols))
    return t


def roms_tools_sizes(total, n, ghost, edge):
    try:
        return _compute_partition_sizes(total, n, ghost, edge, "x")
    except ValueError:
        return None


# F1: lines come in pairs (gnx npx inode nx xi_rho) (xi_u iSW_corn)
raw = run("harness_mpi", P, T)
mpi = defaultdict(list)
for a, b in zip(raw[0::2], raw[1::2]):
    mpi[(a[0], a[1])].append((a[3], a[4], b[0], b[1]))
partit = table(run("harness_partit", P, T), (0, 1), (3, 4))
coarse = table(run("harness_coarse", 2 * P, T), (0, 1), (3, 4, 5))

stats = defaultdict(int)
for (p, t), tiles in mpi.items():
    nx = [x[0] for x in tiles]
    rho = [x[1] for x in tiles]
    u = [x[2] for x in tiles]
    valid = min(nx) >= 1
    stats["pairs"] += 1
    stats["valid"] += valid
    # F1 vs F2: partit widths equal mpi_setup widths
    stats["F1!=F2 widths"] += [x[0] for x in partit[(p, t)]] != nx
    # F1 vs C: C-Star edge width is the minimum tile width, acceptance iff all >= 1
    stats["F1!=C edge"] += (t > 1 and min(nx) != _edge_tile_width(p, t)) or (t == 1 and _edge_tile_width(p, t) != p)
    try:
        check_tile_sizes({"llm": p, "np_xi": t}, auto_tiling=False)
        accepted = True
    except ValueError:
        accepted = False
    stats["F1!=C accept"] += accepted != valid
    # F1 vs R: file sizes for xi_rho (ghost 2, both) and xi_u (ghost 1, upper)
    r_rho = roms_tools_sizes(p + 2, t, 2, "both")
    r_u = roms_tools_sizes(p + 1, t, 1, "upper")
    if valid:
        stats["F1!=R xi_rho"] += r_rho != rho
        stats["F1!=R xi_u"] += r_u != u
    else:
        stats["R accepts invalid"] += (r_rho is not None) or (r_u is not None)
    # coarse grid for the same fine grid (gnx = 2p and 2p+1 both give gnxc = p)
    for g in (2 * p, 2 * p + 1):
        if (g, t) in coarse:
            c = coarse[(g, t)]
            cw = [x[1] for x in c]
            csz = [x[2] for x in c]
            r_c = roms_tools_sizes(g // 2 + 2, t, 2, "both")
            if min(cw) >= 1:
                stats["F3!=R coarse"] += r_c != csz
            else:
                stats["R accepts invalid coarse"] += r_c is not None
for k, v in stats.items():
    print(f"{k:45s} {v}")

# ucla-roms interp_frc abort: coarse ceiling width <= 2, per (fine points, tiles), C-Star accepting
gap = 0
gap_examples = []
for g in range(2, 2 * P + 1):
    for t in range(1, T + 1):
        nxc0 = (g // 2 + t - 1) // t
        try:
            check_tile_sizes({"llm": g, "np_xi": t}, auto_tiling=False)
        except ValueError:
            continue
        if nxc0 <= 2:
            gap += 1
            if len(gap_examples) < 6 and g in (20, 40, 128, 224, 440):
                gap_examples.append((g, t, nxc0))
print(f"{'C-Star accepts, ucla-roms aborts if interp_frc':45s} {gap}  e.g. (points, tiles, coarse width) {gap_examples}")

# partition + join round trips on synthetic datasets (all point types, then rho only)
def synthetic(nx, ny, kinds):
    dims = {"rho": ("eta_rho", "xi_rho", ny + 2, nx + 2), "u": ("eta_rho", "xi_u", ny + 2, nx + 1),
            "v": ("eta_v", "xi_rho", ny + 1, nx + 2)}
    rng = np.random.default_rng(nx * 1000 + ny)
    return xr.Dataset({k: ((dims[k][0], dims[k][1]), rng.random((dims[k][2], dims[k][3]))) for k in kinds})


bad = {"all": 0, "rho": 0}
cases = {"all": 0, "rho": 0}
first_bad = {}
for kinds_name, kinds in (("all", ("rho", "u", "v")), ("rho", ("rho",))):
    for nx in range(1, 25):
        for ny in range(1, 25):
            ds = synthetic(nx, ny, kinds)
            for npx in range(1, 7):
                for npe in range(1, 7):
                    try:
                        _, parts = partition(ds, np_eta=npe, np_xi=npx)
                    except ValueError:
                        continue
                    cases[kinds_name] += 1
                    try:
                        ok = join_datasets(parts).identical(ds)
                    except Exception:
                        ok = False
                    if not ok:
                        bad[kinds_name] += 1
                        first_bad.setdefault(kinds_name, (nx, ny, npx, npe, (nx % npx != 0) or (ny % npe != 0)))
for k in cases:
    print(f"round trip ({k:3s} point types): {cases[k]} partitions, {bad[k]} failed to join back; first failure {first_bad.get(k)}")

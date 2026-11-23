"""RLT-CSTAR-002 Phase 5 measurements, pinned roms-tools vs research branch.

Usage: bench_partition.py BASE_TREE PATCHED_TREE GRID_NC OUTDIR
  1. partition() wall time on an even tiling (both trees accept it), interleaved A/B
     repeats so background load affects both alike.
  2. bytes written by partition_netcdf for an uneven-coarse tiling, using C-Star's
     strategy (try with coarse dims, retry without): the pinned tree falls back and
     writes whole coarse-grid variables into every tile; the branch partitions them.
Resource proxy only: wall time on a shared 2-core VM and bytes on disk; no energy claim.
"""

import json
import os
import shutil
import statistics
import subprocess
import sys
from pathlib import Path

base, patched, grid_nc, outdir = sys.argv[1:5]
outdir = Path(outdir)

TIMING = r"""
import sys, time, numpy as np, xarray as xr
from roms_tools.tiling.partition import partition
n = 960  # divisible by 16 and 20, so the pinned tree accepts it
ds = xr.Dataset({
    "a": (("eta_rho", "xi_rho"), np.zeros((n + 2, n + 2))),
    "u": (("eta_rho", "xi_u"), np.zeros((n + 2, n + 1))),
    "v": (("eta_v", "xi_rho"), np.zeros((n + 1, n + 2))),
})
t0 = time.perf_counter()
for _ in range(20):
    partition(ds, np_eta=16, np_xi=20)
print(time.perf_counter() - t0)
"""

BYTES = r"""
import sys, shutil
from pathlib import Path
from roms_tools.tiling.partition import partition_netcdf
src, work = Path(sys.argv[1]), Path(sys.argv[2])
shutil.rmtree(work, ignore_errors=True); work.mkdir(parents=True)
f = work / src.name; shutil.copy(src, f)
try:
    files = partition_netcdf(f, np_xi=2, np_eta=5, include_coarse_dims=True); mode = "with coarse dims"
except ValueError:
    files = partition_netcdf(f, np_xi=2, np_eta=5, include_coarse_dims=False); mode = "retry without coarse dims"
print(mode, sum(p.stat().st_size for p in files), len(files))
"""


def run(tree, code, *args):
    out = subprocess.run(
        [sys.executable, "-c", code, *args], env={**os.environ, "PYTHONPATH": tree},
        capture_output=True, text=True, check=True,
    ).stdout.strip().splitlines()[-1]
    return out


times = {"base": [], "patched": []}
for _ in range(5):
    for name, tree in (("base", base), ("patched", patched)):
        times[name].append(float(run(tree, TIMING)))
res = {k: {"median_s_per_20_calls": statistics.median(v), "runs": v} for k, v in times.items()}

for name, tree in (("base", base), ("patched", patched)):
    mode, size, n = run(tree, BYTES, grid_nc, str(outdir / name)).rsplit(" ", 2)
    res[name]["partition_netcdf_2x5"] = {"mode": mode, "bytes": int(size), "files": int(n)}
    shutil.rmtree(outdir / name, ignore_errors=True)
print(json.dumps(res, indent=1))

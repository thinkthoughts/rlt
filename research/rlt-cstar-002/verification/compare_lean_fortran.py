"""Compare the Lean companion's definitions, evaluated by Lean, with compiled
ucla-roms mpi_setup (harness_mpi output). Usage: compare_lean_fortran.py LEAN_OUT FORTRAN_OUT"""
import sys
from collections import defaultdict
F, L = defaultdict(list), defaultdict(list)
for line in open(sys.argv[2]):
    g, n, i, nx, xr, xu, isw = map(int, line.split())
    F[(g, n)].append((nx, isw))
for line in open(sys.argv[1]):
    p, t, k, w, st = map(int, line.split())
    L[(p, t)].append((w, st))
valid = mism = inv_ok = inv_bad = 0
for key, f in F.items():
    l = L[key]
    if min(x[0] for x in f) >= 1:
        valid += 1
        mism += f != l
    else:
        ok = (l[-1][0] == 0) == (f[-1][0] <= 0)
        inv_ok += ok
        inv_bad += not ok
print(f"valid tilings {valid}: Lean (width, start) differs from compiled Fortran in {mism}")
print(f"invalid tilings {inv_ok + inv_bad}: same classification in {inv_ok}, different in {inv_bad}")
sys.exit(1 if (mism or inv_bad) else 0)

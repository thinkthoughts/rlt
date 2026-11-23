#!/bin/bash
# RLT-CSTAR-002 verification. Usage:
#   reproduce.sh UCLA_ROMS_CHECKOUT LEAN_BIN OUTDIR
# UCLA_ROMS_CHECKOUT: a clone of CWorthy-ocean/ucla-roms (or the thinkthoughts fork)
#   containing commit f83d2ef6cbb0f7391e17dc6eda54673032859db1 (tag 0.16.0).
# LEAN_BIN: the `lean` executable of leanprover/lean4 v4.34.0-rc2.
# Needs gfortran and a Python with C-Star and ROMS-Tools (the branches under test) installed.
set -euo pipefail
U=$1; LEAN=$2; OUT=$3; HERE=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$OUT" && cd "$OUT"
PIN=f83d2ef6cbb0f7391e17dc6eda54673032859db1
git -C "$U" show $PIN:src/roms_mpi.F90 > roms_mpi.F90
git -C "$U" show $PIN:Tools-Roms/partit.F > partit.F
git -C "$U" show $PIN:src/roms_read_write.F90 > roms_read_write.F90
git -C "$U" show $PIN:src/set_scoord_mod.F90 > set_scoord_mod.F90
sha256sum -c - <<SUMS
827257761213c946832265b421f8f3fa06b7635de0cb3ca001db0e718aaabac9  roms_mpi.F90
7a7cf2c23eedffef976ebc5a8056dec2b1cedd8f6d1c406fa102fdf76b9bda5d  partit.F
9ea2426a3bc9a8a20cc3db8ab418992eb0b4431bf6a8317432d990df91c1abdb  roms_read_write.F90
cca8743a0de0dc157980917923ca8a2f90ab87c48035f16256d0600564b18029  set_scoord_mod.F90
SUMS
# verbatim excerpts, by line range
sed -n 226,227p roms_mpi.F90 > mpi_ceil.inc
sed -n 243,300p roms_mpi.F90 > mpi_sizes.inc
sed -n 1493,1597p partit.F > partit_mpi_setup.F
{ sed -n 545,548p roms_read_write.F90; sed -n 564,575p roms_read_write.F90; sed -n 590,596p roms_read_write.F90; } > coarse_x.inc
sed -n 58,73p set_scoord_mod.F90 > csf.inc
cp "$HERE"/harness_*.F* .
gfortran -O1 -ffree-line-length-none -o harness_mpi harness_mpi.F90
gfortran -cpp -O1 -o harness_partit harness_partit.F
gfortran -O1 -ffree-line-length-none -o harness_coarse harness_coarse.F90
gfortran -O0 -ffree-line-length-none -o harness_csf harness_csf.F90
./harness_csf > csf_values.out
# four implementations of the tile sizes, and partition/join round trips
python -I "$HERE/verify_tiling.py" "$PWD" 300 64 | tee verify_tiling.out
python -I "$HERE/join_rho_only.py" | tee join_rho_only.out
# Lean definitions evaluated by Lean vs compiled Fortran
cat "$HERE/../../../lean/RLTCSTAR002.lean" "$HERE/lean_eval_tail.lean" > lean_eval.lean
"$LEAN" --run lean_eval.lean > lean_eval.out
./harness_mpi 300 64 | paste - - > fortran_mpi.out
python -I "$HERE/compare_lean_fortran.py" lean_eval.out fortran_mpi.out | tee lean_vs_fortran.out
echo "RLT-CSTAR-002 verification complete in $OUT"

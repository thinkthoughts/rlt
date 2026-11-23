#!/bin/bash
# Usage: run_suite.sh WORKTREE BASETEMP LOG [pytest selectors...]
W=$1; BT=$2; LOG=$3; shift 3
export ROMS_ROOT=$W MPIHOME=/usr/lib/x86_64-linux-gnu/openmpi NETCDFHOME=/usr
export MARBL_ROOT=/home/claude/w10/deps/MARBL PIO_ROOT=/home/claude/w10/deps/ParallelIO PNETCDFHOME=/home/claude/w10/deps/pnetcdf
export OMPI_ALLOW_RUN_AS_ROOT=1 OMPI_ALLOW_RUN_AS_ROOT_CONFIRM=1 OMPI_MCA_rmaps_base_oversubscribe=1
export NPY_ENABLE_CPU_FEATURES="SSSE3 SSE41 POPCNT SSE42 AVX F16C FMA3 AVX2" OPENBLAS_CORETYPE=Haswell OPENBLAS_NUM_THREADS=1 NUMBA_CPU_NAME=generic NUMBA_DISABLE_INTEL_SVML=1 NUMBA_NUM_THREADS=1
export RLT_INPUTS=/home/claude/w10/art/inputs-github_gnu
export PATH=$W/Tools-Roms:/home/claude/w10/deps/bin:$PATH
. /home/claude/venv/bin/activate
(cd $W/Tools-Roms && make partit ncjoin COMPILER=gnu > $LOG.tools 2>&1) || { echo "tools build failed" > $LOG; exit 1; }
cd $W/tests
python -m pytest -svv -rfEx . --environ=github_gnu --tolerate-hash-mismatch --basetemp $BT -p no:cacheprovider "$@" > $LOG 2>&1
echo "exit $?" >> $LOG

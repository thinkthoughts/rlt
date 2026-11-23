#!/bin/bash
# Usage: run_ucla_ana.sh ROMS_ROOT BASETEMP LOG
export ROMS_ROOT=$1 MPIHOME=/usr/lib/x86_64-linux-gnu/openmpi NETCDFHOME=/usr
export OMPI_ALLOW_RUN_AS_ROOT=1 OMPI_ALLOW_RUN_AS_ROOT_CONFIRM=1 OMPI_MCA_rmaps_base_oversubscribe=1
export PATH=$ROMS_ROOT/Tools-Roms:$PATH
. /home/claude/venv/bin/activate
export PYTHONPATH=/home/claude/r3/roms-tools   # pinned roms-tools for any generated inputs, same for both runs
cd $ROMS_ROOT/tests
(cd $ROMS_ROOT/Tools-Roms && make partit ncjoin COMPILER=gnu > /dev/null 2>&1)
timeout 3000 python -m pytest -svv -rfEx rlt_ana/test_legacy_analytical_configs.py --environ=github_gnu --tolerate-hash-mismatch --basetemp $2 -p no:cacheprovider > $3 2>&1
echo "exit $?" >> $3

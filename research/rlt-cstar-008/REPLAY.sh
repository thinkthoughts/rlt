#!/bin/bash
# RLT-CSTAR-008 Phase 1: independent replay, as executed on 2026-11-23.
# Usage: REPLAY.sh WORKDIR ARCHIVE_TGZ
#   WORKDIR      empty directory
#   ARCHIVE_TGZ  RLT-CSTAR-007-review.tar.gz (SHA-256 522cf759...bbae2)
# Needs: git, gfortran, libnetcdff-dev (nf-config), python venv with roms-tools'
# dependencies and pytest, mpmath, cargo 1.97.0, Lean 4.34.0-rc2 at $LEAN.
# Trees are selected with PYTHONPATH and plain `python` (python -I ignores PYTHONPATH).
set -euo pipefail
W=$(realpath "$1"); A=$(realpath "$2"); LEAN=${LEAN:-lean}
H=$(dirname "$(realpath "$0")")   # helper scripts beside this file
cd "$W"

## 0. Archive and fresh clones; research branches only from the archive's bundles
sha256sum "$A"
mkdir -p archive && tar -xzf "$A" -C archive
(cd archive/RLT-CSTAR-007-review && sha256sum -c --quiet SHA256SUMS)
E=$W/archive/RLT-CSTAR-007-review/evidence
for r in rlt C-Star roms-tools ucla-roms; do
  git clone -q --filter=blob:none https://github.com/thinkthoughts/$r.git $r
  git -C $r fetch -q $W/archive/RLT-CSTAR-007-review/bundles/$r.bundle 'refs/heads/*:refs/heads/*' 'refs/tags/*:refs/tags/*'
done
git -C rlt worktree add -q --detach $W/wt/rlt research/rlt-cstar-007
git -C C-Star worktree add -q --detach $W/wt/cstar research/rlt-cstar-002
for b in 002 003 004 005-dic-uptake 006-stable-csf; do
  git -C roms-tools worktree add -q --detach $W/wt/rt-$b research/rlt-cstar-$b
done
git -C ucla-roms worktree add -q --detach $W/wt/ucla-pin f83d2ef6
git -C ucla-roms worktree add -q --detach $W/wt/ucla-csf research/rlt-cstar-007-stable-csf
PIN_RT=$W/roms-tools   # the fresh clone is at ede4fe62

## 1. partit / ncjoin
mkdir -p replay/tools replay/p4
(cd replay/tools && bash $E/tools/build.sh $W/wt/ucla-pin && diff SOURCES.sha256 $E/tools/SOURCES.sha256)
cd replay/p4
cp $E/p4/compare_partit.py $E/p4/grid37x23.nc .
PYTHONPATH=$W/wt/rt-002 python -c "
from roms_tools import Grid
Grid(nx=41, ny=29, size_x=2000, size_y=1500, center_lon=150, center_lat=-35, rot=-12, N=4,
     topography_source={'name': 'ETOPO5'}).save('grid41x29.nc')"
for tree in $PIN_RT $W/wt/rt-002; do for g in grid37x23 grid41x29; do
  PYTHONPATH=$tree python compare_partit.py $g.nc $W/replay/tools/partit $W/replay/tools/ncjoin work_$(basename $tree)_$g \
    > $(basename $tree)_$g.out
done; done
cd "$W"

## 2. Coarse checks: verbatim init_coarse lines vs C-Star, every (llm, np_xi) in 2..800 x 1..128
mkdir -p replay/coarse && cd replay/coarse
S=$W/wt/ucla-pin/src/roms_read_write.F90
{ sed -n 545,548p $S; sed -n 564,575p $S; sed -n 590,596p $S; } > coarse_x.inc
cmp coarse_x.inc $E/coarse/coarse_x.inc
cp $H/harness_coarse_min.F90 .
gfortran -O1 -ffree-line-length-none -o harness_coarse_min harness_coarse_min.F90
./harness_coarse_min 800 128 > coarse_min.out
PYTHONPATH=$W/wt/cstar python $H/cstar_vs_init_coarse.py coarse_min.out > cstar_vs_init_coarse.out
cd "$W"

## 3. Units contract
mkdir -p replay/units && cd replay/units
(cd $W/wt/rt-003 && PYTHONPATH=$PWD python -m pytest -q -p no:cacheprovider roms_tools/tests/test_analysis | tail -1) > analysis_tests_003.out
for t in $PIN_RT $W/wt/rt-003; do PYTHONPATH=$t python $E/f009/mixed_prefix_probe.py; done > mixed_prefix.out
PYTHONPATH=$W/wt/rt-003 python $H/units_adversarial.py > adversarial.out
cd "$W"

## 4. Tonnes-CO2 constant
mkdir -p replay/tonnes && cd replay/tonnes
for t in $PIN_RT $W/wt/rt-003; do PYTHONPATH=$t python $H/tonnes_independent.py; done > tonnes.out
cd "$W"

## 5. Stretching function
mkdir -p replay/csf && cd replay/csf
cp $E/f011/harness_csf_grid.F90 $E/f011/compare_csf.py .
for v in pin csf; do
  awk '/^  function CSF/,/end function CSF/' $W/wt/ucla-$v/src/set_scoord_mod.F90 > csf.inc
  cp csf.inc csf_$v.inc
  gfortran -O2 -ffree-line-length-none -o h_$v harness_csf_grid.F90 && ./h_$v > fortran_$v.out
done
cmp fortran_pin.out $E/f011/csf_fortran_old.out && cmp fortran_csf.out $E/f011/csf_fortran_new.out
{ PYTHONPATH=$W/wt/rt-004 python compare_csf.py fortran_pin.out
  PYTHONPATH=$W/wt/rt-006-stable-csf python compare_csf.py fortran_csf.out; } > compare_csf.out
cd "$W"
# 5b. Full ucla-roms runs (see run_ucla_ana.sh, setup_ana_override.sh; needs Open MPI, rsync):
#   setup_ana_override.sh wt/ucla-pin; setup_ana_override.sh wt/ucla-csf
#   run_ucla_ana.sh wt/ucla-pin /tmp/pytest_base base.log; run_ucla_ana.sh wt/ucla-csf /tmp/pytest_head head.log
#   (cd wt/ucla-csf && python tests/compare_outputs.py --mismatches tests/results/mismatches_github_gnu.json \
#        --head-root /tmp/pytest_head --base-root /tmp/pytest_base)

## 6. RLT certificates with a kernel built from scratch; tampering
(cd wt/rlt && test -z "$(git diff 6128c97 HEAD -- kernel cli lib Cargo.toml Cargo.lock rust-toolchain.toml spec)" \
  && CARGO_TARGET_DIR=$W/rlt-target cargo build --release --offline)
RLT=$W/rlt-target/release/rlt; sha256sum $RLT
mkdir -p replay/rlt
(cd wt/rlt && for c in research/rlt-cstar-002/certificates/*.rlt research/rlt-cstar-007/certificates/*.rlt; do
  echo -e "$(basename $c)\t$($RLT check $c | grep '^verdict' | cut -c17-40)"; done) > replay/rlt/check.out
python3 $H/tamper_certificates.py $RLT $W/wt/rlt > replay/rlt/tamper.out

## 7. Lean, pinned toolchain; axiom audit of every theorem
(cd wt/rlt/lean && cat lean-toolchain && $LEAN --version
 for f in RLTCSTAR002 RLTCSTAR007; do
   cp $f.lean /tmp/${f}_ax.lean
   grep -oE "^theorem [A-Za-z0-9_']+" $f.lean | awk -v ns=$f '{print "#print axioms " ns "." $2}' >> /tmp/${f}_ax.lean
   $LEAN /tmp/${f}_ax.lean
 done) > replay/rlt/lean_axioms.out 2>&1
echo "replay complete"

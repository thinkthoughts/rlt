#!/bin/bash
# RLT-CSTAR Lean gate (RLT-CSTAR-009): compile lean/RLTCSTAR002.lean and
# lean/RLTCSTAR007.lean with the toolchain lean/lean-toolchain pins, with
# `#print axioms` appended for every theorem. Fails on any error, any `sorry`,
# or any axiom other than propext, Classical.choice and Quot.sound.
# Usage, from the repository root: research/rlt-cstar-009/check_lean.sh [LEAN]
set -euo pipefail
LEAN=${1:-lean}
cd lean; echo "toolchain: $(cat lean-toolchain)"; $LEAN --version
tmp=$(mktemp -d); status=0
for f in RLTCSTAR002 RLTCSTAR007; do
  if grep -n "sorry" $f.lean; then echo "FAIL: sorry in $f.lean"; status=1; fi
  cp $f.lean $tmp/$f.lean
  grep -oE "^theorem [A-Za-z0-9_']+" $f.lean | awk -v ns=$f '{print "#print axioms " ns "." $2}' >> $tmp/$f.lean
  if ! $LEAN $tmp/$f.lean > $tmp/$f.out 2>&1; then echo "FAIL: $f.lean"; cat $tmp/$f.out; status=1; continue; fi
  n=$(grep -cE "depends on axioms|does not depend on any axioms" $tmp/$f.out || true)
  other=$(grep "depends on axioms" $tmp/$f.out | sed 's/.*axioms: \[//; s/\]//' | tr ',' '\n' | sed 's/ //g' | grep -vE "^(propext|Classical.choice|Quot.sound)$" || true)
  [ -n "$other" ] && { echo "FAIL: other axioms in $f: $other"; status=1; }
  echo "$f.lean: compiled; axiom lines $n (theorems $(grep -cE "^theorem " $f.lean)); other axioms: ${other:-none}"
done
exit $status

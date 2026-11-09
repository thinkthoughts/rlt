#!/usr/bin/env bash
# usage: scripts/compare-reports.sh OLD_RLT OLD_ROOT NEW_RLT NEW_ROOT
# Byte-compares every proof-file report and every record command between two
# rlt binaries, each run on its own repository tree (RLT-003 M3a obligation
# O4). The file list is the old tree's. Exit 0 only if all are identical.
set -uo pipefail
old_bin=$(realpath "$1"); old_root=$(cd "$2" && pwd); new_bin=$(realpath "$3"); new_root=$(cd "$4" && pwd)
old() { "$old_bin" --root "$old_root" "$@"; }
new() { "$new_bin" --root "$new_root" "$@"; }
cd "$old_root"
n=0; d=0
for f in $(find tests -name '*.rlt' | sort); do
  n=$((n+1))
  a=$(old check "$f"; echo "exit $?"); b=$(new check "$f"; echo "exit $?")
  [ "$a" = "$b" ] || { d=$((d+1)); echo "DIFF $f"; diff <(echo "$a") <(echo "$b") | head -6; }
done
echo "proof files: $n compared, $d differ"
cmds=("validate" "rp math0" "rp math5" "rp math29" "points points/oxygen-and-people.yaml" "points points/co2-and-o2.yaml" "residues residues/residue-30.yaml" "chem CO₂ CHEM-MOLFORMULA" "chem COO CHEM-MOLFORMULA")
for c in "${cmds[@]}"; do
  a=$(old $c; echo "exit $?"); b=$(new $c; echo "exit $?")
  if [ "$a" = "$b" ]; then echo "same: rlt $c"; else d=$((d+1)); echo "DIFF: rlt $c"; diff <(echo "$a") <(echo "$b") | head -8; fi
done
for a in tests/written/*.args; do
  mapfile -t w < "$a"
  x=$(old written "${w[@]}"; echo "exit $?"); y=$(new written "${w[@]}"; echo "exit $?")
  [ "$x" = "$y" ] || { d=$((d+1)); echo "DIFF $a"; }
done
x=$(old lean tests/accept/*.rlt); y=$(new lean tests/accept/*.rlt)
[ "$x" = "$y" ] && echo "same: rlt lean tests/accept/*.rlt" || { d=$((d+1)); echo "DIFF: rlt lean"; }
echo "differences: $d"
[ "$d" = 0 ]

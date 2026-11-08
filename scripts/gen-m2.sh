#!/usr/bin/env bash
# RLT-003 M2: write the 111 closed proof files of tests/m2/ with the untrusted
# generator `rlt gen`. The kernel checks every file; verify.sh re-runs this
# script into a scratch directory and requires byte-identical output.
# usage: scripts/gen-m2.sh RLT_BINARY OUT_DIR
set -euo pipefail
rlt=$1; out=$2; mkdir -p "$out"
g() { "$rlt" gen "$1" "M2.$2" > "$out/$2.rlt"; }
for r in $(seq 0 29); do
  for p in 2 3 5; do
    q=$((r / p)); s=$((r % p))
    g "$r = $p × $q + $s" "$(printf 'D_%02d_%d' "$r" "$p")"
  done
done
for p in 2 3 5; do
  for s in $(seq 0 $((p - 1))); do
    g "$s + ($((p - 1 - s)) + 1) = $p" "B_${s}_${p}"
  done
done
for n in 29 30 31 37 59; do
  g "$n = 30 × $((n / 30)) + $((n % 30))" "W_$n"
done
for r in 0 1 7 29; do
  g "$r + ($((29 - r)) + 1) = 30" "C_$r"
done
g "(1 + 1) × 3 × 5 = 30" "E_math29"
g "1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 = 8" "T_eight"

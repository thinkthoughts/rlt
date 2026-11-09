#!/usr/bin/env bash
# RLT-003 M2 GDT comparison, reported on its own (outside scripts/verify.sh).
# usage: scripts/gdt-compare.sh GDT_CHECKOUT
# Prints one status line:
#   GDT comparison: compiled at <commit>       (exit 0)
#   GDT comparison: unverified — <reason>      (exit 1)
# Lean-checked statements about GDT's definitions are not RLT theorems.
set -uo pipefail
pin=ec66260c98eff965bea79891ffbbc0d3229c6e93
here=$(cd "$(dirname "$0")/.." && pwd)
gdt=${1:?usage: scripts/gdt-compare.sh GDT_CHECKOUT}
unverified() { echo "GDT comparison: unverified — $1"; exit 1; }
[ -d "$gdt/.git" ] || unverified "no GDT checkout at $gdt"
head=$(git -C "$gdt" rev-parse HEAD 2>/dev/null) || unverified "cannot read the checkout's commit"
[ "$head" = "$pin" ] || unverified "checkout is at $head; the comparison is pinned to $pin"
git -C "$gdt" diff --quiet HEAD -- || unverified "checkout has local changes"
command -v lake >/dev/null || PATH="$HOME/.elan/bin:$PATH"
command -v lake >/dev/null || unverified "lake (elan) is unavailable"
cd "$gdt"
lake exe cache get >/dev/null 2>&1 || unverified "the Mathlib cache could not be fetched"
lake build GeneralDivisorTheorem.Basic >/dev/null 2>&1 || unverified "GeneralDivisorTheorem.Basic did not build"
log=$(lake env lean "$here/lean/GDTComparison.lean" 2>&1); code=$?
echo "$log"
if [ $code != 0 ] || echo "$log" | grep -qiE "warning|error|sorry"; then
  unverified "lean/GDTComparison.lean did not compile cleanly"
fi
n=$(grep -c "^theorem " "$here/lean/GDTComparison.lean")
listed=$(echo "$log" | grep -cE "depends on axioms|does not depend on any axioms")
[ "$listed" = "$n" ] || unverified "axioms listed for $listed of $n statements"
bad=$(echo "$log" | sed -n 's/.*depends on axioms: \[\(.*\)\]/\1/p' | tr ',' '\n' | tr -d ' ' | grep -vxE 'propext|Classical\.choice|Quot\.sound' | sort -u)
[ -z "$bad" ] || unverified "a statement depends on non-standard axioms: $bad"
echo "GDT comparison: compiled at $pin (lean/GDTComparison.lean, $n statements; Lean-checked, not RLT theorems)"

#!/usr/bin/env bash
# Reproduce RLT's verification of Lemma 12.1 (H45). Run from the repository
# root. Stops at the first unexpected outcome; prints PASS lines otherwise.
set -uo pipefail
cd "$(dirname "$0")/../.."
here=research/h45-lemma-12-1
rlt=target/release/rlt
pass() { echo "PASS  $1"; }
fail() { echo "FAIL  $1"; exit 1; }

sha256sum --quiet -c "$here/MANIFEST.sha256" || fail "files differ from MANIFEST.sha256"
pass "pinned files equal MANIFEST.sha256 ($(grep -c . "$here/MANIFEST.sha256") files)"

cargo build --release -q || fail "cargo build --release"
for g in gen-rlt006.py gen-rlt007s1.py gen-rlt007s4.py; do
  python3 -I "scripts/$g" "$rlt" --materialize-only >/dev/null || fail "scripts/$g --materialize-only"
done
pass "built rlt; certificates materialized (RLT-006, stages 1 and 4)"

out=$("$rlt" compose compose/h45.yaml); code=$?
{ [ $code = 0 ] \
  && grep -qxF "graph           62 nodes in layers 0–2, 56 discharged assumptions, acyclic; every proof re-checked in this run" <<<"$out" \
  && grep -qxF "remaining       assumptions: none (every assumption of every composed node is discharged)" <<<"$out" \
  && grep -qE '^  6\. adm\.all +discharged by composition \(layer 2\)$' <<<"$out" \
  && grep -q '^verdict         COMPOSED' <<<"$out"; } || { echo "$out" | tail -5; fail "rlt compose compose/h45.yaml"; }
pass "rlt compose: COMPOSED — 62 theorems, 56 discharges, 6 results, no remaining assumptions"

out=$(python3 -I scripts/compose-check.py "$rlt" compose/h45.yaml); code=$?
{ [ $code = 0 ] && [ "$(grep -c 'approved statement (rebuilt from the transcription)$' <<<"$out")" = 6 ] \
  && grep -qx 'verdict COMPOSED' <<<"$out"; } || { echo "$out" | tail -5; fail "scripts/compose-check.py"; }
pass "independent Python checker: COMPOSED, six approved statements rebuilt from the transcription"

out=$("$rlt" deps --verify-lib); code=$?
[ $code = 0 ] || { echo "$out" | tail -3; fail "rlt deps --verify-lib"; }
pass "library: every entry current"

lean_bin=$(command -v lean || { [ -x "$HOME/.elan/bin/lean" ] && echo "$HOME/.elan/bin/lean"; })
if [ -n "$lean_bin" ]; then
  out=$(cd lean && "$lean_bin" RLT007D.lean 2>&1); code=$?
  bad=$(sed -n 's/.*depends on axioms: \[\(.*\)\]/\1/p' <<<"$out" | tr ',' '\n' | tr -d ' ' | grep -vxE 'propext|Classical\.choice|Quot\.sound' | sort -u)
  { [ $code = 0 ] && [ "$(grep -cE "^'RLT007D\.[^']*' (depends on axioms|does not depend on any axioms)" <<<"$out")" = 17 ] \
    && [ -z "$bad" ] && ! grep -qiE 'error|warning|sorry' <<<"$out"; } || { echo "$out" | tail -5; fail "lean RLT007D.lean"; }
  pass "Lean 4: 17 theorems, standard axioms only, including lemma_12_1_holds"
else
  echo "SKIP  Lean (install elan; lean/lean-toolchain selects v4.34.0-rc2)"
fi
echo "Lemma 12.1 (H45): discharged by composition from kernel-checked theorems; reproduced."

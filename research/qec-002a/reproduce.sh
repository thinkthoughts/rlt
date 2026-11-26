#!/usr/bin/env bash
# Reproduce RLT-QEC-002a (docs/RLT-QEC-002a.md). Run from anywhere.
#   reproduce.sh [BB_DISTANCE_LEAN_CHECKOUT]
# With a checkout containing commit a83cafad4d041427af4ddb54085493d3f9dd02d2,
# also runs the mask-set comparison with the pinned source.
set -uo pipefail
py="${PYTHON:-python3}"   # interpreter; Python 3.8 or later
here="$(cd "$(dirname "$0")" && pwd)"
cd "$here"
pass() { echo "PASS  $1"; }
fail() { echo "FAIL  $1"; exit 1; }
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT

sha256sum --quiet -c MANIFEST.sha256 || fail "files differ from MANIFEST.sha256"
pass "interpreter: $("$py" -c 'import sys; print(sys.version.split()[0])')"
pass "files equal MANIFEST.sha256 ($(grep -c . MANIFEST.sha256) files)"

"$py" -I census.py --route 1 --out "$tmp/r1.txt" 2>/dev/null || fail "census.py --route 1"
"$py" -I census.py --route 2 --out "$tmp/r2.txt" 2>/dev/null || fail "census.py --route 2"
cmp -s "$tmp/r1.txt" "$tmp/r2.txt" || fail "routes 1 and 2 differ"
cmp -s "$tmp/r1.txt" Z7.txt || fail "census differs from Z7.txt"
pass "census: routes 1 and 2 both reproduce Z7.txt ($(wc -l < Z7.txt) masks)"

out=$("$py" -I check.py Z7.txt) || fail "check.py Z7.txt"
pass "checker: $out"

"$py" -I test_qec002a.py >"$tmp/tests.txt" 2>&1 || { tail -20 "$tmp/tests.txt"; fail "tests"; }
pass "tests: $(grep -E '^Ran ' "$tmp/tests.txt") — $(tail -1 "$tmp/tests.txt")"

if [ $# -ge 1 ]; then
  out=$("$py" -I compare.py "$1" Z7.txt) || { echo "$out"; fail "compare.py"; }
  echo "$out" | sed 's/^/      /'
  pass "source comparison: mask sets equal"
  QEC_SRC="$1" "$py" -I test_qec002a.py >"$tmp/src-tests.txt" 2>&1 \
    || { tail -20 "$tmp/src-tests.txt"; fail "tests with QEC_SRC"; }
  pass "tests with source: $(grep -E '^Ran ' "$tmp/src-tests.txt") — $(tail -1 "$tmp/src-tests.txt")"
else
  echo "SKIP  source comparison (pass a bb-distance-lean checkout as the argument)"
fi

#!/usr/bin/env bash
# Reproduce RLT-QEC-002b (docs/RLT-QEC-002b.md). Run from anywhere.
#   reproduce.sh [ABSOLUTE_PATH_TO_BB_DISTANCE_LEAN_CHECKOUT]
# With a checkout containing commit a83cafad4d041427af4ddb54085493d3f9dd02d2,
# also runs the source correspondence and the source tests.
set -uo pipefail
py="${PYTHON:-python3}"   # interpreter; Python 3.8 or later
here="$(cd "$(dirname "$0")" && pwd)"
cd "$here"
pass() { echo "PASS  $1"; }
fail() { echo "FAIL  $1"; exit 1; }
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
export PYTHONDONTWRITEBYTECODE=1

pass "interpreter: $("$py" -c 'import sys; print(sys.version.split()[0])')"
sha256sum --quiet -c MANIFEST.sha256 || fail "files differ from MANIFEST.sha256"
pass "files equal MANIFEST.sha256 ($(grep -c . MANIFEST.sha256) files)"
(cd ../qec-002a && sha256sum --quiet -c MANIFEST.sha256) || fail "accepted RLT-QEC-002a files changed"
pass "accepted RLT-QEC-002a evidence unchanged"

"$py" -I witness.py --outdir "$tmp" 2>"$tmp/build.txt" || { cat "$tmp/build.txt"; fail "witness.py"; }
for f in W.txt ranks.json structure.txt; do cmp -s "$tmp/$f" "$f" || fail "witness.py output differs: $f"; done
pass "builder reproduces W.txt, ranks.json, structure.txt — $(cat "$tmp/build.txt")"

out=$("$py" -I check_witness.py W.txt ../qec-002a/Z7.txt ranks.json structure.txt) || fail "check_witness.py"
pass "checker: $out"

"$py" -I test_qec002b.py >"$tmp/tests.txt" 2>&1 || { tail -20 "$tmp/tests.txt"; fail "tests"; }
pass "tests: $(grep -E '^Ran ' "$tmp/tests.txt") — $(tail -1 "$tmp/tests.txt")"

if [ $# -ge 1 ]; then
  out=$("$py" -I compare_002b.py "$1" W.txt ../qec-002a/Z7.txt) || { echo "$out"; fail "compare_002b.py"; }
  echo "$out" | sed 's/^/      /'
  pass "source correspondence: A indexed columns EQUAL; B source witnesses VERIFIED"
  QEC_SRC="$1" "$py" -I test_qec002b.py >"$tmp/src.txt" 2>&1 || { tail -20 "$tmp/src.txt"; fail "tests with QEC_SRC"; }
  pass "tests with source: $(grep -E '^Ran ' "$tmp/src.txt") — $(tail -1 "$tmp/src.txt")"
else
  echo "SKIP  source correspondence (pass an absolute bb-distance-lean checkout path)"
fi

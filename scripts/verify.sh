#!/usr/bin/env bash
# RLT-001 verification: formatting, lints, build, unit and golden tests,
# every proof file, written-layer cases, record validation, the Lean replay,
# and the source-excerpt check. Writes verification/RLT-001-results.txt.
# Exits 0 only if every stage holds.
#
# usage: scripts/verify.sh SOURCE_CHECKOUTS_DIR
#   SOURCE_CHECKOUTS_DIR: clones of news, rlt, translatepi, antiviolence,
#   residue-manifold-learning and specification-grammar (see
#   scripts/check_excerpts.py). Required.
# Explicit opt-outs (each makes the run report SKIPPED stages and exit 3):
#   RLT_ALLOW_NO_LEAN=1     run without a Lean installation
#   RLT_SKIP_EXCERPTS=1     run without SOURCE_CHECKOUTS_DIR
set -uo pipefail
cd "$(dirname "$0")/.."
out=verification/RLT-001-results.txt
mkdir -p verification
fail=0
skipped=0
rlt=${CARGO_TARGET_DIR:-target}/release/rlt

# Log to the results file and the terminal without a subshell, so that
# `fail` set below reaches the final `exit`; tee is awaited before exit.
exec 3>&1 4>&2
exec > >(tee "$out") 2>&1
tee_pid=$!

finish() {
  echo
  if [ $fail != 0 ]; then
    echo "RESULT   FAILURES above"; code=1
  elif [ $skipped != 0 ]; then
    echo "RESULT   all run checks hold; SKIPPED stages above"; code=3
  else
    echo "RESULT   all checks hold"; code=0
  fi
  exec 1>&3 2>&4
  wait "$tee_pid" 2>/dev/null
  exit $code
}

echo "RLT-001 verification"
echo "commit   $(git rev-parse HEAD 2>/dev/null || echo unknown)"
echo "rustc    $(rustc --version)"
echo

echo "== format and lints"
cargo fmt --all -- --check >/dev/null 2>&1 && echo "fmt      ok" || { echo "fmt      FAILED (cargo fmt --all)"; fail=1; }
clippy_log=$(cargo clippy -q --all-targets -- -D warnings 2>&1); clippy_code=$?
[ $clippy_code = 0 ] && echo "clippy   ok" || { echo "clippy   FAILED"; echo "$clippy_log" | head -30; fail=1; }

echo; echo "== build"
cargo build --release -q && echo "ok" || { echo "FAILED"; fail=1; finish; }

echo; echo "== cargo test (unit + golden)"
tests_log=$(cargo test --release -q 2>&1); tests_code=$?
echo "$tests_log" | grep -E "^test result" | grep -v " 0 passed"
[ $tests_code = 0 ] || { echo "FAILED"; echo "$tests_log" | tail -20; fail=1; }

echo; echo "== proof files (expect accept → exit 0, reject → exit 1)"
for f in tests/accept/*.rlt tests/reject/*.rlt; do
  report=$("$rlt" check "$f"); code=$?
  case "$f" in tests/accept/*) want=0 ;; *) want=1 ;; esac
  verdict=$(echo "$report" | grep '^verdict' | tail -1 | sed 's/^verdict *//')
  printf '%-52s exit %s  %s\n' "$f" "$code" "$verdict"
  [ "$code" = "$want" ] || { echo "  UNEXPECTED exit (want $want)"; fail=1; }
done

echo; echo "== written layer"
for a in tests/written/*.args; do
  mapfile -t w < "$a"
  printf -- '-- %s\n' "$a"
  "$rlt" written "${w[0]}" "${w[1]}" "${w[2]}" "${w[3]}" | grep -E '^(written ≡w|terms)' || fail=1
done

echo; echo "== specification records (points/)"
for f in points/*.yaml; do
  [ -e "$f" ] || continue
  points_log=$("$rlt" points "$f"); points_code=$?
  printf '%-40s exit %s  %s\n' "$f" "$points_code" "$(echo "$points_log" | grep '^verdict' | tail -1 | sed 's/^verdict *//')"
  [ $points_code = 0 ] || { echo "$points_log" | grep '^problem'; fail=1; }
done

echo; echo "== records"
validate_log=$("$rlt" validate); validate_code=$?
echo "$validate_log" | grep -E '^(PiπΠ|problem|verdict)'
[ $validate_code = 0 ] || fail=1

echo; echo "== Lean replay (independent, outside the trusted base)"
lean_bin=$(command -v lean || { [ -x "$HOME/.elan/bin/lean" ] && echo "$HOME/.elan/bin/lean"; })
if [ -n "$lean_bin" ]; then
  # Run inside lean/ so elan selects lean/lean-toolchain. Any Lean message
  # other than the #print axioms lines (a warning, `sorry`, an error) fails.
  lean_log=$(cd lean && "$lean_bin" --version && "$lean_bin" RLT001.lean && "$lean_bin" Generated.lean 2>&1); lean_code=$?
  echo "$lean_log"
  if [ $lean_code != 0 ] || echo "$lean_log" | grep -qiE "warning|error|sorry"; then
    echo "FAILED"; fail=1
  else
    echo "ok"
  fi
  regen=$("$rlt" lean tests/accept/*.rlt)
  [ "$regen" = "$(cat lean/Generated.lean)" ] && echo "Generated.lean is current" || { echo "Generated.lean is STALE (rlt lean tests/accept/*.rlt > lean/Generated.lean)"; fail=1; }
elif [ "${RLT_ALLOW_NO_LEAN:-}" = 1 ]; then
  echo "SKIPPED (lean unavailable; RLT_ALLOW_NO_LEAN=1)"; skipped=1
else
  echo "FAILED (lean unavailable; install elan, or set RLT_ALLOW_NO_LEAN=1 to skip explicitly)"; fail=1
fi

echo; echo "== source excerpts"
if [ $# -ge 1 ]; then
  excerpts_log=$(python3 -I scripts/check_excerpts.py "$1"); excerpts_code=$?
  echo "$excerpts_log" | grep -E "^(MISSING|ERROR|PARTIAL|UNCOVERED)"; echo "$excerpts_log" | tail -1
  [ $excerpts_code = 0 ] || fail=1
elif [ "${RLT_SKIP_EXCERPTS:-}" = 1 ]; then
  echo "SKIPPED (no SOURCE_CHECKOUTS_DIR; RLT_SKIP_EXCERPTS=1)"; skipped=1
else
  echo "FAILED (pass SOURCE_CHECKOUTS_DIR, or set RLT_SKIP_EXCERPTS=1 to skip explicitly)"; fail=1
fi

finish

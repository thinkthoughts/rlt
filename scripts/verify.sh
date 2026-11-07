#!/usr/bin/env bash
# RLT-001 verification: build, unit and golden tests, every proof file,
# written-layer cases, record validation, and the optional Lean replay.
# Writes verification/RLT-001-results.txt. Exit 0 only if every check holds.
#
# usage: scripts/verify.sh [SOURCE_CHECKOUTS_DIR]
#   SOURCE_CHECKOUTS_DIR (optional): clones of the source repositories, for
#   scripts/check_excerpts.py.
set -uo pipefail
cd "$(dirname "$0")/.."
out=verification/RLT-001-results.txt
mkdir -p verification
fail=0
rlt=target/release/rlt

# Log to the results file and the terminal without a subshell, so that
# `fail` set below reaches the final `exit`.
exec > >(tee "$out") 2>&1

  echo "RLT-001 verification"
  echo "commit   $(git rev-parse HEAD 2>/dev/null || echo unknown)"
  echo "rustc    $(rustc --version)"
  echo

  echo "== build"
  cargo build --release -q && echo "ok" || { echo "FAILED"; fail=1; }

  echo; echo "== cargo test (unit + golden)"
  tests_log=$(cargo test --release -q 2>&1); tests_code=$?
  echo "$tests_log" | grep -E "^test result" | grep -v " 0 passed"
  [ $tests_code = 0 ] || { echo "FAILED"; echo "$tests_log" | tail -20; fail=1; }

  echo; echo "== proof files (expect accept → exit 0, reject → exit 1)"
  for f in tests/accept/*.rlt tests/reject/*.rlt; do
    "$rlt" check "$f" >/dev/null; code=$?
    case "$f" in tests/accept/*) want=0 ;; *) want=1 ;; esac
    verdict=$("$rlt" check "$f" | grep '^verdict' | sed 's/^verdict *//')
    printf '%-52s exit %s  %s\n' "$f" "$code" "$verdict"
    [ "$code" = "$want" ] || { echo "  UNEXPECTED exit (want $want)"; fail=1; }
  done

  echo; echo "== written layer"
  for a in tests/written/*.args; do
    mapfile -t w < "$a"
    printf -- '-- %s\n' "$a"
    "$rlt" written "${w[0]}" "${w[1]}" "${w[2]}" "${w[3]}" | grep -E '^(written ≡w|terms)'
  done

  echo; echo "== records"
  validate_log=$("$rlt" validate); validate_code=$?
  echo "$validate_log" | grep -E '^(PiπΠ|problem|verdict)'
  [ $validate_code = 0 ] || fail=1

  echo; echo "== Lean replay (independent, outside the trusted base)"
  if command -v lean >/dev/null 2>&1 || [ -x "$HOME/.elan/bin/lean" ]; then
    lean_bin=$(command -v lean || echo "$HOME/.elan/bin/lean")
    # Run inside lean/ so elan selects lean/lean-toolchain.
    (cd lean && "$lean_bin" --version && "$lean_bin" RLT001.lean && "$lean_bin" Generated.lean) && echo "ok" || { echo "FAILED"; fail=1; }
  else
    echo "skipped (lean unavailable)"
  fi

  if [ $# -ge 1 ]; then
    echo; echo "== source excerpts"
    excerpts_log=$(python3 -I scripts/check_excerpts.py "$1"); excerpts_code=$?
    echo "$excerpts_log" | grep -E "^(MISSING|ERROR)"; echo "$excerpts_log" | tail -1
    [ $excerpts_code = 0 ] || fail=1
  fi

  echo
  [ $fail = 0 ] && echo "RESULT   all checks hold" || echo "RESULT   FAILURES above"
exit $fail

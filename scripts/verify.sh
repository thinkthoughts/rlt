#!/usr/bin/env bash
# RLT-001 verification (extended by RLT-002 and RLT-003 M1, M2 and M3a): formatting, lints, build, unit and golden tests,
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

echo; echo "== multiplication, RLT-003 M1 (tests/m1: expected exit codes and reasons)"
while IFS=$'\t' read -r f want word reason; do
  case "$f" in ''|'#'*) continue ;; esac
  m1_log=$("$rlt" check "$f"); m1_code=$?
  printf '%-36s exit %s (want %s)\n' "$f" "$m1_code" "$want"
  { [ "$m1_code" = "$want" ] && echo "$m1_log" | grep -qF -- "$reason"; } || { echo "  UNEXPECTED: $(echo "$m1_log" | grep '^verdict')"; fail=1; }
done < tests/m1/expected.tsv
for g in "2 × 3 = 6:M1.two-three:M1_two_three" "6 × 5 = 30:M1.six-five:M1_six_five" "30 = 2 × 3 × 5:M1.thirty:M1_thirty"; do
  IFS=: read -r w id f <<< "$g"
  if "$rlt" gen "$w" "$id" | cmp -s - "tests/accept/$f.rlt"; then echo "rlt gen reproduces tests/accept/$f.rlt"; else echo "  UNEXPECTED: rlt gen differs from tests/accept/$f.rlt"; fail=1; fi
done

echo; echo "== closed quotient–remainder witnesses, RLT-003 M2 (tests/m2, residues/)"
m2_tmp=$(mktemp -d)
if scripts/gen-m2.sh "$rlt" "$m2_tmp" && diff -r tests/m2 "$m2_tmp" >/dev/null; then
  echo "scripts/gen-m2.sh reproduces tests/m2 ($(ls "$m2_tmp" | wc -l) files)"
else
  echo "  UNEXPECTED: tests/m2 differs from scripts/gen-m2.sh output"; fail=1
fi
rm -rf "$m2_tmp"
m2_ok=0; m2_n=0
for f in tests/m2/*.rlt; do
  m2_n=$((m2_n + 1))
  m2_log=$("$rlt" check "$f") && echo "$m2_log" | grep -q '^verdict         ACCEPT — unconditional theorem relative to registered axiom set PEANO-ARITH' \
    && echo "$m2_log" | grep -q '^assumptions     Γ = ∅$' && m2_ok=$((m2_ok + 1)) || { echo "  UNEXPECTED: $f"; fail=1; }
done
echo "tests/m2: $m2_ok of $m2_n proof files ACCEPT, unconditional, Γ = ∅"
[ "$m2_n" = 111 ] && [ "$m2_ok" = 111 ] || { echo "  UNEXPECTED: want 111 of 111"; fail=1; }
if scripts/gen-m2-record.sh | cmp -s - residues/residue-30.yaml; then echo "scripts/gen-m2-record.sh reproduces residues/residue-30.yaml"; else echo "  UNEXPECTED: residues/residue-30.yaml differs from scripts/gen-m2-record.sh"; fail=1; fi
res_log=$("$rlt" residues residues/residue-30.yaml); res_code=$?
echo "$res_log" | grep -E '^(  consistency|theorems|not established|problem|verdict)'
[ $res_code = 0 ] || fail=1
for pin in "readingpoints/math29.yaml 828ca404210423cc3a2f8f82aa34ee6f193326232182e959432205e7a99cb1d3"; do
  read -r f want <<< "$pin"
  [ "$(sha256sum "$f" | cut -d' ' -f1)" = "$want" ] && echo "$f unchanged (sha256 $want)" || { echo "  UNEXPECTED: $f changed"; fail=1; }
done

echo; echo "== logical checker, RLT-003 M3a (tests/m3a: accepted controls, rejected cases with reasons)"
m3a_tmp=$(mktemp -d)
if python3 -I scripts/gen-m3a-tests.py "$rlt" "$m3a_tmp" && diff -r tests/m3a "$m3a_tmp" >/dev/null; then
  echo "scripts/gen-m3a-tests.py reproduces tests/m3a"
else
  echo "  UNEXPECTED: tests/m3a differs from scripts/gen-m3a-tests.py output"; fail=1
fi
rm -rf "$m3a_tmp"
m3a_ok=0; m3a_n=0
for f in tests/m3a/accept/*.rlt; do
  m3a_n=$((m3a_n + 1))
  "$rlt" check "$f" | grep -q '^verdict         ACCEPT — unconditional theorem' && m3a_ok=$((m3a_ok + 1)) || { echo "  UNEXPECTED: $f"; fail=1; }
done
echo "tests/m3a/accept: $m3a_ok of $m3a_n ACCEPT, unconditional"
m3a_r=0; m3a_rn=0
while IFS=$'\t' read -r f want reason; do
  case "$f" in ''|'#'*) continue ;; esac
  m3a_rn=$((m3a_rn + 1))
  m3a_log=$("$rlt" check "$f"); m3a_code=$?
  { [ "$m3a_code" = "$want" ] && echo "$m3a_log" | grep '^verdict' | grep -qF -- "$reason"; } && m3a_r=$((m3a_r + 1)) || { echo "  UNEXPECTED: $f exit $m3a_code (want $want): $(echo "$m3a_log" | grep '^verdict')"; fail=1; }
done < tests/m3a/expected.tsv
echo "tests/m3a/reject: $m3a_r of $m3a_rn refused with the required exit code and reason"
if [ -n "${RLT_REFERENCE:-}" ]; then
  # RLT_REFERENCE=BINARY:ROOT of the merged M2 main (bf3c9cc): byte-compare earlier reports (O4).
  scripts/compare-reports.sh "${RLT_REFERENCE%%:*}" "${RLT_REFERENCE#*:}" "$rlt" . | grep -E '^(DIFF|proof files|differences)'
fi

echo; echo "== chemical notation (rlt chem; 0 interpreted, 1 uninterpreted)"
for case in "CO₂:0" "O₂:0" "COO:1" "CO2:1" "O2:1"; do
  w=${case%:*}; want=${case##*:}
  chem_log=$("$rlt" chem "$w" CHEM-MOLFORMULA); chem_code=$?
  printf '%-8s exit %s (want %s)  %s\n' "$w" "$chem_code" "$want" "$(echo "$chem_log" | grep '^verdict' | sed 's/^verdict *//' | cut -c1-60)"
  [ "$chem_code" = "$want" ] || fail=1
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
  lean_log=$(cd lean && "$lean_bin" --version && "$lean_bin" RLT001.lean && "$lean_bin" Generated.lean && "$lean_bin" M2Generated.lean && "$lean_bin" M2Readings.lean && "$lean_bin" M3aGenerated.lean 2>&1); lean_code=$?
  echo "$lean_log"
  if [ $lean_code != 0 ] || echo "$lean_log" | grep -qiE "warning|error|sorry"; then
    echo "FAILED"; fail=1
  else
    echo "ok"
  fi
  m2_n=$(grep -c "^theorem " lean/M2Readings.lean)
  m2_listed=$(echo "$lean_log" | grep -cE "RLTM2Readings\..*(depends on axioms|does not depend on any axioms)")
  m2_bad=$(echo "$lean_log" | grep "RLTM2Readings\." | sed -n 's/.*depends on axioms: \[\(.*\)\]/\1/p' | tr ',' '\n' | tr -d ' ' | grep -vxE 'propext|Classical\.choice|Quot\.sound' | sort -u)
  { [ "$m2_listed" = "$m2_n" ] && [ -z "$m2_bad" ]; } && echo "M2Readings.lean: $m2_n statements, standard axioms only" || { echo "M2Readings.lean: axiom gate FAILED ($m2_listed of $m2_n listed; non-standard: $m2_bad)"; fail=1; }
  m3a_regen=$(scripts/gen-m3a-lean.sh "$rlt")
  [ "$m3a_regen" = "$(cat lean/M3aGenerated.lean)" ] && echo "M3aGenerated.lean is current ($(grep -c '^theorem' lean/M3aGenerated.lean) statements; Lean-checked over ℕ, outside the trusted base)" || { echo "M3aGenerated.lean is STALE (scripts/gen-m3a-lean.sh target/release/rlt > lean/M3aGenerated.lean)"; fail=1; }
  m2_regen=$(scripts/gen-m2-lean.sh "$rlt")
  [ "$m2_regen" = "$(cat lean/M2Generated.lean)" ] && echo "M2Generated.lean is current (111 statements; tier 1). M2Readings.lean compiled (tier 2; Lean-checked, not RLT theorems)" || { echo "M2Generated.lean is STALE (scripts/gen-m2-lean.sh target/release/rlt > lean/M2Generated.lean)"; fail=1; }
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

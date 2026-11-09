#!/usr/bin/env bash
# RLT-001 verification (extended by RLT-002 and RLT-003 M1, M2, M3a and M3b): formatting, lints, build, unit and golden tests,
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
echo; echo "== Robinson's Q, divisibility and ordering, RLT-003 M3b (tests/m3b)"
m3b_tmp=$(mktemp -d)
if python3 -I scripts/gen-m3b-tests.py "$rlt" "$m3b_tmp" && diff -r tests/m3b "$m3b_tmp" >/dev/null; then
  echo "scripts/gen-m3b-tests.py reproduces tests/m3b"
else
  echo "  UNEXPECTED: tests/m3b differs from scripts/gen-m3b-tests.py output"; fail=1
fi
rm -rf "$m3b_tmp"
m3b_ok=0; m3b_n=0
for f in tests/m3b/accept/*.rlt; do
  m3b_n=$((m3b_n + 1))
  "$rlt" check "$f" | grep -qE '^verdict         ACCEPT — (unconditional|conditional) theorem relative to registered axiom set ROBINSON-Q' && m3b_ok=$((m3b_ok + 1)) || { echo "  UNEXPECTED: $f"; fail=1; }
done
echo "tests/m3b/accept: $m3b_ok of $m3b_n ACCEPT under ROBINSON-Q"
m3b_r=0; m3b_rn=0
while IFS=$'\t' read -r f want reason; do
  case "$f" in tests/*) ;; *) continue ;; esac
  m3b_rn=$((m3b_rn + 1))
  m3b_log=$("$rlt" check "$f"); m3b_code=$?
  { [ "$m3b_code" = "$want" ] && echo "$m3b_log" | grep '^verdict' | grep -qF -- "$reason"; } && m3b_r=$((m3b_r + 1)) || { echo "  UNEXPECTED: $f exit $m3b_code (want $want): $(echo "$m3b_log" | grep '^verdict')"; fail=1; }
done < tests/m3b/expected.tsv
echo "tests/m3b/reject: $m3b_r of $m3b_rn refused with the required exit code and reason"

echo; echo "== divisibility and non-divisibility proofs, theorem citation, RLT-003 M3c (tests/m3c)"
m3c_tmp=$(mktemp -d)
if python3 -I scripts/gen-m3c-tests.py "$rlt" "$m3c_tmp" && diff -r tests/m3c "$m3c_tmp" >/dev/null; then
  echo "scripts/gen-m3c-tests.py reproduces tests/m3c"
else
  echo "  UNEXPECTED: tests/m3c differs from scripts/gen-m3c-tests.py output"; fail=1
fi
rm -rf "$m3c_tmp"
m3c_ok=0; m3c_n=0
for f in tests/m3c/accept/*.rlt; do
  m3c_n=$((m3c_n + 1))
  "$rlt" check "$f" | grep -qE '^verdict         ACCEPT — unconditional theorem relative to registered axiom set ROBINSON-Q' && m3c_ok=$((m3c_ok + 1)) || { echo "  UNEXPECTED: $f"; fail=1; }
done
echo "tests/m3c/accept: $m3c_ok of $m3c_n ACCEPT as unconditional theorems under ROBINSON-Q"
m3c_r=0; m3c_rn=0
while IFS=$'\t' read -r f want reason; do
  case "$f" in \#*|"") continue ;; esac
  m3c_rn=$((m3c_rn + 1))
  m3c_log=$("$rlt" check "$f"); m3c_code=$?
  { [ "$m3c_code" = "$want" ] && echo "$m3c_log" | grep '^verdict' | grep -qF -- "$reason"; } && m3c_r=$((m3c_r + 1)) || { echo "  UNEXPECTED: $f exit $m3c_code (want $want): $(echo "$m3c_log" | grep '^verdict')"; fail=1; }
done < tests/m3c/expected.tsv
echo "tests/m3c/reject: $m3c_r of $m3c_rn refused with the required exit code and reason"
for pin in "spec/peano.rlt 1a3f83444a06fdfbd02c9af2a5881b47137ba102a675a4e012c84be477b10a0d" "spec/peano-arith.rlt 00ceffa9a60b53286f350edbf2f1a5f48a14169a9b09a82cbedc6f9956ff01e4" "spec/robinson-q.rlt 4ad55192222576e3901abc4ee7ea5a6760b64ff212e8c55796ab9847e423b00c" "spec/definitions.rlt 122e37fc627b136023a1a5ffac0344162ed77ae95312248a9f6a76c2b639ecda"; do
  read -r f want <<< "$pin"
  [ "$(sha256sum "$f" | cut -d' ' -f1)" = "$want" ] && echo "$f sha256 $want" || { echo "  UNEXPECTED: $f changed"; fail=1; }
done

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
  lean_log=$(cd lean && "$lean_bin" --version && "$lean_bin" RLT001.lean && "$lean_bin" Generated.lean && "$lean_bin" M2Generated.lean && "$lean_bin" M2Readings.lean && "$lean_bin" M3aGenerated.lean && "$lean_bin" M3bGenerated.lean && "$lean_bin" M3bReadings.lean && "$lean_bin" M3cGenerated.lean && "$lean_bin" M3cReadings.lean 2>&1); lean_code=$?
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
  m3b_n=$(grep -c "^theorem " lean/M3bReadings.lean)
  m3b_listed=$(echo "$lean_log" | grep -cE "RLTM3bReadings\..*(depends on axioms|does not depend on any axioms)")
  m3b_bad=$(echo "$lean_log" | grep "RLTM3bReadings\." | sed -n 's/.*depends on axioms: \[\(.*\)\]/\1/p' | tr ',' '\n' | tr -d ' ' | grep -vxE 'propext|Classical\.choice|Quot\.sound' | sort -u)
  { [ "$m3b_listed" = "$m3b_n" ] && [ -z "$m3b_bad" ]; } && echo "M3bReadings.lean: $m3b_n statements, standard axioms only (tier 2; Lean-checked correspondence, not RLT theorems)" || { echo "M3bReadings.lean: axiom gate FAILED ($m3b_listed of $m3b_n listed; non-standard: $m3b_bad)"; fail=1; }
  # Tier 1 axiom gate: print every generated theorem's axioms.
  m3b_gate=$(mktemp -d)/M3bGate.lean
  { cat lean/M3bGenerated.lean; sed -n 's/^theorem \([A-Za-z0-9_]*\).*/#print axioms RLTGeneratedM3b.\1/p' lean/M3bGenerated.lean; } > "$m3b_gate"
  m3b_glog=$(cd lean && "$lean_bin" "$m3b_gate" 2>&1)
  m3b_gn=$(grep -c "^theorem " lean/M3bGenerated.lean)
  m3b_gl=$(echo "$m3b_glog" | grep -cE "(depends on axioms|does not depend on any axioms)")
  m3b_gbad=$(echo "$m3b_glog" | sed -n 's/.*depends on axioms: \[\(.*\)\]/\1/p' | tr ',' '\n' | tr -d ' ' | grep -vxE 'propext|Classical\.choice|Quot\.sound' | sort -u)
  { [ "$m3b_gl" = "$m3b_gn" ] && [ -z "$m3b_gbad" ]; } && echo "M3bGenerated.lean: $m3b_gn statements, standard axioms only" || { echo "M3bGenerated.lean: axiom gate FAILED ($m3b_gl of $m3b_gn listed; non-standard: $m3b_gbad)"; fail=1; }
  m3c_n=$(grep -c "^theorem " lean/M3cReadings.lean)
  m3c_listed=$(echo "$lean_log" | grep -cE "RLTM3cReadings\..*(depends on axioms|does not depend on any axioms)")
  m3c_bad=$(echo "$lean_log" | grep "RLTM3cReadings\." | sed -n 's/.*depends on axioms: \[\(.*\)\]/\1/p' | tr ',' '\n' | tr -d ' ' | grep -vxE 'propext|Classical\.choice|Quot\.sound' | sort -u)
  { [ "$m3c_listed" = "$m3c_n" ] && [ -z "$m3c_bad" ]; } && echo "M3cReadings.lean: $m3c_n statements, standard axioms only (tier 2; Lean-checked correspondence, not RLT theorems)" || { echo "M3cReadings.lean: axiom gate FAILED ($m3c_listed of $m3c_n listed; non-standard: $m3c_bad)"; fail=1; }
  # Tier 1 axiom gates for M3c and (M3b audit note 4) M3a: every generated theorem.
  for gen in M3cGenerated:RLTGeneratedM3c M3aGenerated:RLTGeneratedM3a; do
    gfile=${gen%%:*}; gns=${gen##*:}
    ggate=$(mktemp -d)/${gfile}Gate.lean
    { cat "lean/$gfile.lean"; sed -n "s/^theorem \([A-Za-z0-9_]*\).*/#print axioms $gns.\1/p" "lean/$gfile.lean"; } > "$ggate"
    glog=$(cd lean && "$lean_bin" "$ggate" 2>&1)
    gn=$(grep -c "^theorem " "lean/$gfile.lean")
    gl=$(echo "$glog" | grep -cE "(depends on axioms|does not depend on any axioms)")
    gbad=$(echo "$glog" | sed -n 's/.*depends on axioms: \[\(.*\)\]/\1/p' | tr ',' '\n' | tr -d ' ' | grep -vxE 'propext|Classical\.choice|Quot\.sound' | sort -u)
    { [ "$gl" = "$gn" ] && [ -z "$gbad" ]; } && echo "$gfile.lean: $gn statements, standard axioms only" || { echo "$gfile.lean: axiom gate FAILED ($gl of $gn listed; non-standard: $gbad)"; fail=1; }
  done
  m3c_regen=$(scripts/gen-m3c-lean.sh "$rlt")
  [ "$m3c_regen" = "$(cat lean/M3cGenerated.lean)" ] && echo "M3cGenerated.lean is current ($(grep -c '^theorem' lean/M3cGenerated.lean) statements; Lean-checked over ℕ, outside the trusted base)" || { echo "M3cGenerated.lean is STALE (scripts/gen-m3c-lean.sh target/release/rlt > lean/M3cGenerated.lean)"; fail=1; }
  m3b_regen=$(scripts/gen-m3b-lean.sh "$rlt")
  [ "$m3b_regen" = "$(cat lean/M3bGenerated.lean)" ] && echo "M3bGenerated.lean is current ($m3b_gn statements; Lean-checked over ℕ, outside the trusted base)" || { echo "M3bGenerated.lean is STALE (scripts/gen-m3b-lean.sh target/release/rlt > lean/M3bGenerated.lean)"; fail=1; }
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

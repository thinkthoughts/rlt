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

echo; echo "== RLT-006 generated certificates (materialized before any check; tests/rlt006/manifest.sha256)"
g6_t0=$(date +%s.%N)
python3 -I scripts/gen-rlt006.py "$rlt" --materialize-only >/dev/null || { echo "  UNEXPECTED: scripts/gen-rlt006.py --materialize-only failed"; fail=1; }
g6_t1=$(date +%s.%N)
g6_check() {  # manifest file → prints mismatches; exit 0 only if all 630 present and equal
  python3 -I - "$1" <<'PY'
import hashlib, sys
bad = n = 0
for line in open(sys.argv[1], encoding="utf-8"):
    s, p = line.rstrip("\n").split("  ", 1)
    n += 1
    try:
        got = hashlib.sha256(open(p, "rb").read()).hexdigest()
    except OSError:
        got = "(missing)"
    if got != s:
        bad += 1
        print(f"  MANIFEST MISMATCH {p}: {got}")
sys.exit(0 if n == 630 and bad == 0 else 1)
PY
}
if g6_check tests/rlt006/manifest.sha256; then
  echo "630 of 630 certificates generated in $(printf %.1f "$(echo "$g6_t1 - $g6_t0" | bc)") s, each equal to its manifest hash; $(du -sk tests/rlt006/gen | cut -f1) KB on disk ($(ls tests/rlt006/gen | wc -l) files)"
else
  echo "  UNEXPECTED: tests/rlt006/gen differs from tests/rlt006/manifest.sha256"; fail=1
fi

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

echo; echo "== finite enumeration below 30, RLT-003 M3d (tests/m3d)"
m3d_tmp=$(mktemp -d)
if python3 -I scripts/gen-m3d-tests.py "$rlt" "$m3d_tmp" && diff -r tests/m3d "$m3d_tmp" >/dev/null; then
  echo "scripts/gen-m3d-tests.py reproduces tests/m3d"
else
  echo "  UNEXPECTED: tests/m3d differs from scripts/gen-m3d-tests.py output"; fail=1
fi
rm -rf "$m3d_tmp"
m3d_ok=0; m3d_n=0
for f in tests/m3d/accept/*.rlt; do
  m3d_n=$((m3d_n + 1))
  "$rlt" check "$f" | grep -qE '^verdict         ACCEPT — unconditional theorem relative to registered axiom set ROBINSON-Q' && m3d_ok=$((m3d_ok + 1)) || { echo "  UNEXPECTED: $f"; fail=1; }
done
echo "tests/m3d/accept: $m3d_ok of $m3d_n ACCEPT as unconditional theorems under ROBINSON-Q"
m3d_r=0; m3d_rn=0
while IFS=$'\t' read -r f want reason; do
  case "$f" in \#*|"") continue ;; esac
  m3d_rn=$((m3d_rn + 1))
  m3d_log=$("$rlt" check "$f"); m3d_code=$?
  { [ "$m3d_code" = "$want" ] && echo "$m3d_log" | grep '^verdict' | grep -qF -- "$reason"; } && m3d_r=$((m3d_r + 1)) || { echo "  UNEXPECTED: $f exit $m3d_code (want $want): $(echo "$m3d_log" | grep '^verdict')"; fail=1; }
done < tests/m3d/expected.tsv
echo "tests/m3d/reject: $m3d_r of $m3d_rn refused with the required exit code and reason"

echo; echo "== complete modulo-30 classification, RLT-003 M3e (tests/m3e)"
m3e_tmp=$(mktemp -d)
if python3 -I scripts/gen-m3e-tests.py "$rlt" "$m3e_tmp" && diff -r tests/m3e "$m3e_tmp" >/dev/null; then
  echo "scripts/gen-m3e-tests.py reproduces tests/m3e"
else
  echo "  UNEXPECTED: tests/m3e differs from scripts/gen-m3e-tests.py output"; fail=1
fi
rm -rf "$m3e_tmp"
m3e_ok=0; m3e_n=0
for f in tests/m3e/accept/*.rlt; do
  m3e_n=$((m3e_n + 1))
  "$rlt" check "$f" | grep -qE '^verdict         ACCEPT — unconditional theorem relative to registered axiom set ROBINSON-Q' && m3e_ok=$((m3e_ok + 1)) || { echo "  UNEXPECTED: $f"; fail=1; }
done
echo "tests/m3e/accept: $m3e_ok of $m3e_n ACCEPT as unconditional theorems under ROBINSON-Q"
m3e_r=0; m3e_rn=0
while IFS=$'\t' read -r f want reason; do
  case "$f" in \#*|"") continue ;; esac
  m3e_rn=$((m3e_rn + 1))
  m3e_log=$("$rlt" check "$f"); m3e_code=$?
  { [ "$m3e_code" = "$want" ] && echo "$m3e_log" | grep '^verdict' | grep -qF -- "$reason"; } && m3e_r=$((m3e_r + 1)) || { echo "  UNEXPECTED: $f exit $m3e_code (want $want): $(echo "$m3e_log" | grep '^verdict')"; fail=1; }
done < tests/m3e/expected.tsv
echo "tests/m3e/reject: $m3e_r of $m3e_rn refused with the required exit code and reason"
echo; echo "== published mathematics: 8 is not matchable, RLT-005 (tests/rlt005)"
r5_tmp=$(mktemp -d); cp -r tests/rlt005 "$r5_tmp/before"
if python3 -I scripts/gen-rlt005.py "$rlt" >/dev/null && diff -r "$r5_tmp/before" tests/rlt005 >/dev/null; then
  echo "scripts/gen-rlt005.py reproduces tests/rlt005"
else
  echo "  UNEXPECTED: tests/rlt005 differs from scripts/gen-rlt005.py output"; rm -rf tests/rlt005; cp -r "$r5_tmp/before" tests/rlt005; fail=1
fi
rm -rf "$r5_tmp"
r5_ok=0; r5_n=0
for f in tests/rlt005/accept/*.rlt; do
  r5_n=$((r5_n + 1))
  "$rlt" check "$f" | grep -qE '^verdict         ACCEPT — unconditional theorem relative to registered axiom set ROBINSON-Q' && r5_ok=$((r5_ok + 1)) || { echo "  UNEXPECTED: $f"; fail=1; }
done
echo "tests/rlt005/accept: $r5_ok of $r5_n ACCEPT as unconditional theorems under ROBINSON-Q"
r5_r=0; r5_rn=0
while IFS=$'\t' read -r f want reason; do
  case "$f" in \#*|"") continue ;; esac
  r5_rn=$((r5_rn + 1))
  r5_log=$("$rlt" check "$f"); r5_code=$?
  { [ "$r5_code" = "$want" ] && echo "$r5_log" | grep '^verdict' | grep -qF -- "$reason"; } && r5_r=$((r5_r + 1)) || { echo "  UNEXPECTED: $f exit $r5_code (want $want): $(echo "$r5_log" | grep '^verdict')"; fail=1; }
done < tests/rlt005/expected.tsv
r5_files=$(ls tests/rlt005/reject/*.rlt | wc -l)
{ [ "$r5_rn" -gt 0 ] && [ "$r5_rn" = "$r5_files" ]; } || { echo "  UNEXPECTED: tests/rlt005/expected.tsv lists $r5_rn cases for $r5_files files in tests/rlt005/reject"; fail=1; }
echo "tests/rlt005/reject: $r5_r of $r5_rn give the required exit code and reason"
r5_deps=$("$rlt" deps matchable.not.8); r5_dcode=$?
{ [ $r5_dcode = 0 ] && echo "$r5_deps" | grep -q '^exit            0'; } && echo "rlt deps matchable.not.8: $(echo "$r5_deps" | grep -E '^(theorems|budget)' | tr -s ' ' | paste -sd ';')" || { echo "  UNEXPECTED: rlt deps matchable.not.8 exit $r5_dcode"; fail=1; }
for pin in "spec/peano.rlt 1a3f83444a06fdfbd02c9af2a5881b47137ba102a675a4e012c84be477b10a0d" "spec/peano-arith.rlt 00ceffa9a60b53286f350edbf2f1a5f48a14169a9b09a82cbedc6f9956ff01e4" "spec/robinson-q.rlt 4ad55192222576e3901abc4ee7ea5a6760b64ff212e8c55796ab9847e423b00c" "spec/definitions.rlt 122e37fc627b136023a1a5ffac0344162ed77ae95312248a9f6a76c2b639ecda"; do
  read -r f want <<< "$pin"
  [ "$(sha256sum "$f" | cut -d' ' -f1)" = "$want" ] && echo "$f sha256 $want" || { echo "  UNEXPECTED: $f changed"; fail=1; }
done

if [ -n "${RLT_REFERENCE:-}" ]; then
  # RLT_REFERENCE=BINARY:ROOT of the merged M2 main (bf3c9cc): byte-compare earlier reports (O4).
  scripts/compare-reports.sh "${RLT_REFERENCE%%:*}" "${RLT_REFERENCE#*:}" "$rlt" . | grep -E '^(DIFF|proof files|differences)'
fi

echo; echo "== H45 residue certificates, RLT-006 (tests/rlt006, certificates/h45)"
r6_tmp=$(mktemp -d)
r6_out="tests/rlt006/accept tests/rlt006/reject tests/rlt006/expected.tsv tests/rlt006/manifest.sha256 certificates/h45 lean/RLT006Direct.lean"
for x in $r6_out; do mkdir -p "$r6_tmp/before/$(dirname "$x")"; cp -r "$x" "$r6_tmp/before/$x"; done
mkdir -p "$r6_tmp/before/statements"; cp statements/H45-OMITS-*.yaml "$r6_tmp/before/statements/"
g6_t0=$(date +%s.%N)
if python3 -I scripts/gen-rlt006.py "$rlt" >/dev/null; then
  g6_t1=$(date +%s.%N); r6_same=1
  for x in $r6_out; do diff -r "$r6_tmp/before/$x" "$x" >/dev/null || r6_same=0; done
  for f in "$r6_tmp"/before/statements/*.yaml; do cmp -s "$f" "statements/$(basename "$f")" || r6_same=0; done
  [ "$(ls statements/H45-OMITS-*.yaml | wc -l)" = 14 ] || r6_same=0
  if [ $r6_same = 1 ] && g6_check tests/rlt006/manifest.sha256 >/dev/null; then
    echo "scripts/gen-rlt006.py reproduces every committed output and all 630 certificates ($(printf %.1f "$(echo "$g6_t1 - $g6_t0" | bc)") s)"
  else
    echo "  UNEXPECTED: scripts/gen-rlt006.py output differs from the committed files"; fail=1
    for x in $r6_out; do rm -rf "$x"; cp -r "$r6_tmp/before/$x" "$x"; done
  fi
else
  echo "  UNEXPECTED: scripts/gen-rlt006.py failed"; fail=1
fi
rm -rf "$r6_tmp"
r6_t0=$(date +%s.%N); r6_ok=0
for f in tests/rlt006/gen/*.rlt; do
  "$rlt" check "$f" | grep -qE '^verdict         ACCEPT — unconditional theorem relative to registered axiom set ROBINSON-Q' && r6_ok=$((r6_ok + 1)) || { echo "  UNEXPECTED: $f"; fail=1; }
done
r6_t1=$(date +%s.%N)
[ $r6_ok = 630 ] || fail=1
echo "1. element certificates: $r6_ok of 630 ACCEPT as unconditional theorems under ROBINSON-Q ($(printf %.1f "$(echo "$r6_t1 - $r6_t0" | bc)") s)"
r6_t0=$(date +%s.%N); r6_ok=0
for f in tests/rlt006/accept/OM_*.rlt; do
  r6_d=$("$rlt" deps "$f")
  { echo "$r6_d" | grep -q '^budget          cited-file checks 45 of 64 · citing-file chain 1 of 8' && echo "$r6_d" | grep -q '^exit            0'; } && r6_ok=$((r6_ok + 1)) || { echo "  UNEXPECTED: $f: $(echo "$r6_d" | grep -E '^(budget|exit)')"; fail=1; }
done
r6_t1=$(date +%s.%N)
[ $r6_ok = 14 ] || fail=1
echo "2. per-prime theorems: $r6_ok of 14 ACCEPT, each citing its 45 certificates (45 of 64 cited-file checks) ($(printf %.1f "$(echo "$r6_t1 - $r6_t0" | bc)") s)"
r6_r=0; r6_rn=0
while IFS=$'\t' read -r f want reason; do
  case "$f" in \#*|"") continue ;; esac
  r6_rn=$((r6_rn + 1))
  r6_log=$("$rlt" check "$f"); r6_code=$?
  { [ "$r6_code" = "$want" ] && echo "$r6_log" | grep '^verdict' | grep -qF -- "$reason"; } && r6_r=$((r6_r + 1)) || { echo "  UNEXPECTED: $f exit $r6_code (want $want): $(echo "$r6_log" | grep '^verdict')"; fail=1; }
done < tests/rlt006/expected.tsv
{ [ "$r6_rn" -gt 0 ] && [ "$r6_rn" = "$(ls tests/rlt006/reject/*.rlt | wc -l)" ]; } || { echo "  UNEXPECTED: tests/rlt006/expected.tsv lists $r6_rn cases"; fail=1; }
echo "tests/rlt006/reject: $r6_r of $r6_rn give the required exit code and reason"
# Tamper tests on the generated files (restored by regeneration afterwards).
r6_t() {  # name, required verdict substring
  r6_log=$("$rlt" check tests/rlt006/accept/OM_43.rlt); r6_code=$?
  r6_dc=$("$rlt" deps admissible.h45.omits.43_3 >/dev/null; echo $?)
  { [ $r6_code = 2 ] && [ "$r6_dc" != 0 ] && echo "$r6_log" | grep '^verdict' | grep -qF -- "$2" && ! g6_check tests/rlt006/manifest.sha256 >/dev/null; } \
    && echo "refused: $1 (rlt check MALFORMED, rlt deps exit $r6_dc, manifest mismatch)" || { echo "  UNEXPECTED: $1 not refused"; fail=1; }
}
mv tests/rlt006/gen/O_43_212.rlt tests/rlt006/gen/O_43_212.rlt.away
r6_t "a generated certificate missing" "cited path \`tests/rlt006/gen/O_43_212.rlt\`: entity not found"
mv tests/rlt006/gen/O_43_212.rlt.away tests/rlt006/gen/O_43_212.rlt
sed -i '1s/^; RLT-006/; rlt-006/' tests/rlt006/gen/O_43_000.rlt
r6_t "a generated certificate modified" "citation \`E0\`: tests/rlt006/gen/O_43_000.rlt has SHA-256"
python3 -I scripts/gen-rlt006.py "$rlt" --materialize-only >/dev/null
r6_m=$(mktemp); sed '1s/^./0/' tests/rlt006/manifest.sha256 > "$r6_m"; head -1 tests/rlt006/manifest.sha256 | grep -q '^0' && sed -i '1s/^./1/' "$r6_m"
g6_check "$r6_m" >/dev/null && { echo "  UNEXPECTED: an altered manifest hash was accepted"; fail=1; } || echo "refused: an altered manifest hash (the certificates no longer match it)"
rm -f "$r6_m"
g6_check tests/rlt006/manifest.sha256 >/dev/null && echo "restored: all 630 certificates equal to the manifest again" || { echo "  UNEXPECTED: certificates not restored"; fail=1; }
x6_ok=0; x6_n=0
while IFS=$'\t' read -r f want frag; do
  case "$f" in \#*|"") continue ;; esac
  x6_n=$((x6_n + 1))
  x6_log=$("$rlt" explain "tests/explain-rlt006/$f"); x6_code=$?
  { [ "$x6_code" = "$want" ] && echo "$x6_log" | grep -qF -- "$frag"; } && x6_ok=$((x6_ok + 1)) || { echo "  UNEXPECTED: tests/explain-rlt006/$f exit $x6_code (want $want)"; fail=1; }
done < tests/explain-rlt006/expected.tsv
[ "$x6_n" = 6 ] || fail=1
echo "tests/explain-rlt006: $x6_ok of $x6_n record fixtures give the required exit code and line"
# 3. The residue profile: computed evidence, checked here by plain arithmetic.
python3 -I - <<'PY' || fail=1
import sys
rows = [l.rstrip("\n").split("\t") for l in open("certificates/h45/residues.tsv") if not l.startswith("#")]
ok = len(rows) == 630 and all(int(h) == int(p) * int(q) + int(s) and 0 <= int(s) < int(p) and (int(s) != int(r)) == (o == "yes") and o == "yes" for p, r, h, q, s, o in rows)
prof = [l.rstrip("\n").split("\t") for l in open("certificates/h45/profile.tsv") if not l.startswith("#")]
occ = {(int(p)): set(map(int, oc.split(","))) for p, r, n, oc, om in prof}
ok = ok and len(prof) == 14 and all(int(n) == len(occ[int(p)]) and int(r) in set(map(int, om.split(","))) for p, r, n, oc, om in prof)
ok = ok and all(occ[int(p)] == {int(s) for pp, r, h, q, s, o in rows if pp == p} for p in occ)
print(("3. residue profile (computed evidence, not kernel-checked): 630 rows with h = p*q + s and s != r_p; " + ", ".join(f"nu_{p}={len(occ[int(p)])}" for p, *_ in prof)) if ok else "  UNEXPECTED: certificates/h45 arithmetic fails")
sys.exit(0 if ok else 1)
PY

echo; echo "== statement records and rlt explain, RLT-004b (statements/, tests/explain)"
ex_tmp=$(mktemp -d)
cp -r tests/explain "$ex_tmp/before"; cp -r tests/explain-rlt006 "$ex_tmp/before6"
if python3 scripts/gen-explain-tests.py >/dev/null && diff -r -q "$ex_tmp/before" tests/explain >/dev/null && diff -r -q "$ex_tmp/before6" tests/explain-rlt006 >/dev/null; then
  echo "scripts/gen-explain-tests.py reproduces tests/explain and tests/explain-rlt006"
else
  echo "  UNEXPECTED: tests/explain or tests/explain-rlt006 differs from scripts/gen-explain-tests.py output"; rm -rf tests/explain tests/explain-rlt006; cp -r "$ex_tmp/before" tests/explain; cp -r "$ex_tmp/before6" tests/explain-rlt006; fail=1
fi
rm -rf "$ex_tmp"
for f in statements/*.yaml; do
  ex_log=$("$rlt" explain "$f"); ex_code=$?
  ex_json=$("$rlt" explain "$f" --format json); ex_jcode=$?
  ex_parsed=$(printf '%s' "$ex_json" | python3 -c 'import json,sys; d=json.load(sys.stdin); print(d["exit"], d["kernel"]["verdict"], d["record"]["state"], d["semantic"]["status"]+"/"+d["semantic"]["assessment"], d["physical"]["applicability"])')
  printf '%-42s exit %s  json: %s\n' "$f" "$ex_code" "$ex_parsed"
  { [ "$ex_code" = 0 ] && [ "$ex_jcode" = 0 ] && [ "${ex_parsed%% *}" = 0 ]; } || { echo "$ex_log" | grep -E '^(  problem|  stale|verdict)'; fail=1; }
done
# Lean references named in statement records: each must be a theorem Lean
# accepts in that file, with standard axioms only (rlt explain only finds the name).
ex_lean=$(command -v lean || { [ -x "$HOME/.elan/bin/lean" ] && echo "$HOME/.elan/bin/lean"; })
ex_refs=$(for f in statements/*.yaml; do "$rlt" explain "$f" --format json; done | python3 -c '
import json, sys
dec, s, i, out = json.JSONDecoder(), sys.stdin.read(), 0, set()
while i < len(s):
    while i < len(s) and s[i].isspace(): i += 1
    if i == len(s): break
    d, i = dec.raw_decode(s, i)
    for l in ((d.get("semantic") or {}).get("lean") or []): out.add((l["file"], l["theorem"]))
print("\n".join(f"{a} {b}" for a, b in sorted(out)))')
if [ -n "$ex_lean" ]; then
  # One Lean run per referenced file, with every referenced theorem's
  # `#print axioms` appended (RLT-006: 16 references, two files).
  for lf in $(echo "$ex_refs" | cut -d' ' -f1 | sort -u); do
    ex_gate=$(mktemp -d)/Gate.lean
    { cat "$lf"; echo; echo "$ex_refs" | awk -v f="$lf" '$1 == f { print "#print axioms " $2 }'; } > "$ex_gate"
    ex_glog=$(cd lean && "$ex_lean" "$ex_gate" 2>&1); ex_gcode=$?
    for th in $(echo "$ex_refs" | awk -v f="$lf" '$1 == f { print $2 }'); do
      ex_line=$(echo "$ex_glog" | grep -E "^'$th' (depends on axioms|does not depend on any axioms)" | tail -1)
      ex_bad=$(echo "$ex_line" | sed -n 's/.*depends on axioms: \[\(.*\)\]/\1/p' | tr ',' '\n' | tr -d ' ' | grep -vxE 'propext|Classical\.choice|Quot\.sound')
      if [ $ex_gcode = 0 ] && [ -n "$ex_line" ] && [ -z "$ex_bad" ] && ! echo "$ex_glog" | grep -qiE "warning|error|sorry"; then
        echo "Lean reference $th ($lf): checked, standard axioms only"
      else
        echo "  UNEXPECTED: Lean reference $th ($lf) is not a Lean-checked theorem with standard axioms"; fail=1
      fi
    done
  done
elif [ "${RLT_ALLOW_NO_LEAN:-}" = 1 ]; then
  echo "SKIPPED: Lean references in statement records (lean unavailable; RLT_ALLOW_NO_LEAN=1)"; skipped=1
else
  echo "FAILED: Lean references in statement records (lean unavailable)"; fail=1
fi
ex_n=0; ex_ok=0
while IFS=$'\t' read -r f want line; do
  case "$f" in \#*|"") continue ;; esac
  ex_n=$((ex_n + 1))
  ex_log=$("$rlt" explain "tests/explain/$f"); ex_code=$?
  { [ "$ex_code" = "$want" ] && echo "$ex_log" | grep -qF -- "$line"; } && ex_ok=$((ex_ok + 1)) || { echo "  UNEXPECTED: $f exit $ex_code (want $want)"; fail=1; }
done < tests/explain/expected.tsv
echo "tests/explain: $ex_ok of $ex_n fixtures give the required exit code and report line"
for g in txt json; do
  fmt=text; [ $g = json ] && fmt=json
  "$rlt" explain statements/MOD30-CLASSIFICATION.yaml --format $fmt | cmp -s - tests/explain-golden/MOD30-CLASSIFICATION.$g \
    && echo "golden $g output reproduced" || { echo "  UNEXPECTED: golden $g output differs"; fail=1; }
done

echo; echo "== theorem library and rlt deps, RLT-004c (lib/)"
dl_log=$("$rlt" deps --verify-lib); dl_code=$?
echo "$dl_log" | grep -E '^entries'
[ $dl_code = 0 ] || { echo "$dl_log" | grep -vE '^entry .* exit 0 '; fail=1; }
dl_json=$("$rlt" deps mod30.classification --format json); dl_jcode=$?
dl_parsed=$(printf '%s' "$dl_json" | python3 -c 'import json,sys; d=json.load(sys.stdin); s=d["summary"]; print(d["exit"], s["distinct_theorems"], s["cited_file_checks"], s["citing_file_chain"])')
echo "rlt deps mod30.classification: exit $dl_jcode; json exit, distinct theorems, cited-file checks, chain: $dl_parsed"
[ "$dl_jcode" = 0 ] && [ "${dl_parsed%% *}" = 0 ] || fail=1
scripts/check-identities.sh; dl_icode=$?
[ $dl_icode = 0 ] || fail=1

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
  lean_log=$(cd lean && "$lean_bin" --version && "$lean_bin" RLT001.lean && "$lean_bin" Generated.lean && "$lean_bin" M2Generated.lean && "$lean_bin" M2Readings.lean && "$lean_bin" M3aGenerated.lean && "$lean_bin" M3bGenerated.lean && "$lean_bin" M3bReadings.lean && "$lean_bin" M3cGenerated.lean && "$lean_bin" M3cReadings.lean && "$lean_bin" M3dGenerated.lean && "$lean_bin" M3dReadings.lean && "$lean_bin" M3eGenerated.lean && "$lean_bin" M3eReadings.lean && "$lean_bin" RLT005Direct.lean && "$lean_bin" RLT005Readings.lean && "$lean_bin" RLT006Direct.lean && "$lean_bin" RLT006BGP212.lean 2>&1); lean_code=$?
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
  m3d_n=$(grep -c "^theorem " lean/M3dReadings.lean)
  m3d_listed=$(echo "$lean_log" | grep -cE "RLTM3dReadings\..*(depends on axioms|does not depend on any axioms)")
  m3d_bad=$(echo "$lean_log" | grep "RLTM3dReadings\." | sed -n 's/.*depends on axioms: \[\(.*\)\]/\1/p' | tr ',' '\n' | tr -d ' ' | grep -vxE 'propext|Classical\.choice|Quot\.sound' | sort -u)
  { [ "$m3d_listed" = "$m3d_n" ] && [ -z "$m3d_bad" ]; } && echo "M3dReadings.lean: $m3d_n statements, standard axioms only (tier 2; Lean-checked correspondence, not RLT theorems)" || { echo "M3dReadings.lean: axiom gate FAILED ($m3d_listed of $m3d_n listed; non-standard: $m3d_bad)"; fail=1; }
  m3e_n=$(grep -c "^theorem " lean/M3eReadings.lean)
  m3e_listed=$(echo "$lean_log" | grep -cE "RLTM3eReadings\..*(depends on axioms|does not depend on any axioms)")
  m3e_bad=$(echo "$lean_log" | grep "RLTM3eReadings\." | sed -n 's/.*depends on axioms: \[\(.*\)\]/\1/p' | tr ',' '\n' | tr -d ' ' | grep -vxE 'propext|Classical\.choice|Quot\.sound' | sort -u)
  { [ "$m3e_listed" = "$m3e_n" ] && [ -z "$m3e_bad" ]; } && echo "M3eReadings.lean: $m3e_n statements, standard axioms only (tier 2; Lean-checked correspondence, not RLT theorems)" || { echo "M3eReadings.lean: axiom gate FAILED ($m3e_listed of $m3e_n listed; non-standard: $m3e_bad)"; fail=1; }
  # Tier 1 axiom gates for M3c and (M3b audit note 4) M3a: every generated theorem.
  # RLT-005: both comparisons, every theorem listed with standard axioms only,
  # and the shared definitions byte-identical in the two files.
  for rf_pin in RLT005Direct:11 RLT005Readings:22; do
    rf=${rf_pin%%:*}; r5_want=${rf_pin#*:}
    r5_names=$(sed -n 's/^[[:space:]]*\(theorem\|lemma\) \([A-Za-z0-9_'"'"']*\).*/\2/p' "lean/$rf.lean")
    r5_n=$(echo "$r5_names" | grep -c .); r5_listed=0
    for nm in $r5_names; do echo "$lean_log" | grep -qE "^'$rf\.$nm' (depends on axioms|does not depend on any axioms)" && r5_listed=$((r5_listed + 1)); done
    r5_bad=$(echo "$lean_log" | grep "^'$rf\." | sed -n 's/.*depends on axioms: \[\(.*\)\]/\1/p' | tr ',' '\n' | tr -d ' ' | grep -vxE 'propext|Classical\.choice|Quot\.sound' | sort -u)
    { [ "$r5_n" = "$r5_want" ] && [ "$r5_listed" = "$r5_n" ] && [ -z "$r5_bad" ]; } && echo "$rf.lean: $r5_n statements, standard axioms only (RLT-005 Lean comparison; not RLT theorems)" || { echo "$rf.lean: axiom gate FAILED ($r5_listed of $r5_n listed, $r5_want pinned; non-standard: $r5_bad)"; fail=1; }
  done
  # RLT-006: theorem counts pinned; every #print axioms line standard; the
  # vendored lean-bgp212 blocks byte-identical to the pinned upstream lines.
  for rf_pin in RLT006Direct:27 RLT006BGP212:12; do
    rf=${rf_pin%%:*}; r6_want=${rf_pin#*:}
    r6_n=$(grep -cE '^[[:space:]]*(theorem|lemma) ' "lean/$rf.lean")
    r6_p=$(grep -c '^#print axioms ' "lean/$rf.lean")
    case $rf in RLT006Direct) r6_ns="RLT006Direct" ;; *) r6_ns="RLT006BGP212|BGP212\.Challenge" ;; esac
    r6_bad=$(echo "$lean_log" | grep -E "^'($r6_ns)\." | sed -n 's/.*depends on axioms: \[\(.*\)\]/\1/p' | tr ',' '\n' | tr -d ' ' | grep -vxE 'propext|Classical\.choice|Quot\.sound' | sort -u)
    r6_listed=$(echo "$lean_log" | grep -cE "^'($r6_ns)\.[^']*' (depends on axioms|does not depend on any axioms)")
    { [ "$r6_n" = "$r6_want" ] && [ "$r6_p" = "$r6_n" ] && [ "$r6_listed" = "$r6_n" ] && [ -z "$r6_bad" ]; } && echo "$rf.lean: $r6_n statements, each printed, standard axioms only (RLT-006 Lean comparison; not RLT theorems)" || { echo "$rf.lean: axiom gate FAILED ($r6_n theorems, $r6_want pinned, $r6_p printed; non-standard: $r6_bad)"; fail=1; }
  done
  [ "$(echo "$lean_log" | grep -cE "^'(RLT006Direct|RLT006BGP212|BGP212\.Challenge)\.")" = 39 ] || { echo "RLT-006 Lean: expected 39 axiom lines"; fail=1; }
  r6_lit() { grep -E "^def ($2) : List" "lean/$1.lean" | sed 's/^def [A-Za-z0-9]* : //'; }
  { [ "$(r6_lit RLT006Direct H45)" = "$(r6_lit RLT006BGP212 rltH45)" ] && [ "$(r6_lit RLT006Direct table)" = "$(r6_lit RLT006BGP212 rltTable)" ] && [ -n "$(r6_lit RLT006Direct H45)" ]; } \
    && echo "RLT006BGP212.lean: rltH45 and rltTable equal the generated transcription literals" || { echo "RLT006BGP212.lean: transcription literals DIFFER from lean/RLT006Direct.lean"; fail=1; }
  if [ $# -ge 1 ] && [ -d "$1/lean-bgp212/.git" ]; then
    v6_ok=1
    for blk in "BGP212/Basic.lean 25 47" "Solution/Lemma12_1.lean 20 64"; do
      set -- "$1" $blk
      v6_up=$(git -C "$1/lean-bgp212" show "7898eb4665197ab418a62b9bb657dab3f31bb262:$2" 2>/dev/null | sed -n "$3,$4p")
      v6_here=$(sed -n "/^-- BEGIN VENDORED lean-bgp212 $(echo "$2" | sed 's|/|\\/|g') $3-$4\$/,/^-- END VENDORED lean-bgp212/p" lean/RLT006BGP212.lean | sed '1d;$d')
      [ -n "$v6_up" ] && [ "$v6_up" = "$v6_here" ] || v6_ok=0
      set -- "$1"
    done
    [ $v6_ok = 1 ] && echo "RLT006BGP212.lean: vendored blocks byte-identical to lean-bgp212 @ 7898eb4 (BGP212/Basic.lean 25-47, Solution/Lemma12_1.lean 20-64)" || { echo "RLT006BGP212.lean: vendored blocks DIFFER from lean-bgp212 @ 7898eb4"; fail=1; }
  else
    echo "  UNEXPECTED: no clone of lean-bgp212 in the source checkouts; the vendored blocks are unverified"; fail=1
  fi
  r5_block() { sed -n '/^-- BEGIN SHARED RLT-005 BLOCK/,/^-- END SHARED RLT-005 BLOCK/p' "$1"; }
  { [ -n "$(r5_block lean/RLT005Direct.lean)" ] && [ "$(r5_block lean/RLT005Direct.lean)" = "$(r5_block lean/RLT005Readings.lean)" ]; } && echo "RLT005 shared definitions: byte-identical in both files" || { echo "RLT005 shared definitions DIFFER between RLT005Direct.lean and RLT005Readings.lean"; fail=1; }
  for gen in M3eGenerated:RLTGeneratedM3e M3dGenerated:RLTGeneratedM3d M3cGenerated:RLTGeneratedM3c M3aGenerated:RLTGeneratedM3a; do
    gfile=${gen%%:*}; gns=${gen##*:}
    ggate=$(mktemp -d)/${gfile}Gate.lean
    { cat "lean/$gfile.lean"; sed -n "s/^theorem \([A-Za-z0-9_]*\).*/#print axioms $gns.\1/p" "lean/$gfile.lean"; } > "$ggate"
    glog=$(cd lean && "$lean_bin" "$ggate" 2>&1)
    gn=$(grep -c "^theorem " "lean/$gfile.lean")
    gl=$(echo "$glog" | grep -cE "(depends on axioms|does not depend on any axioms)")
    gbad=$(echo "$glog" | sed -n 's/.*depends on axioms: \[\(.*\)\]/\1/p' | tr ',' '\n' | tr -d ' ' | grep -vxE 'propext|Classical\.choice|Quot\.sound' | sort -u)
    { [ "$gl" = "$gn" ] && [ -z "$gbad" ]; } && echo "$gfile.lean: $gn statements, standard axioms only" || { echo "$gfile.lean: axiom gate FAILED ($gl of $gn listed; non-standard: $gbad)"; fail=1; }
  done
  m3d_regen=$(scripts/gen-m3d-lean.sh "$rlt")
  [ "$m3d_regen" = "$(cat lean/M3dGenerated.lean)" ] && echo "M3dGenerated.lean is current ($(grep -c '^theorem' lean/M3dGenerated.lean) statements; Lean-checked over ℕ, outside the trusted base)" || { echo "M3dGenerated.lean is STALE (scripts/gen-m3d-lean.sh target/release/rlt > lean/M3dGenerated.lean)"; fail=1; }
  m3e_regen=$(scripts/gen-m3e-lean.sh "$rlt")
  [ "$m3e_regen" = "$(cat lean/M3eGenerated.lean)" ] && echo "M3eGenerated.lean is current ($(grep -c '^theorem' lean/M3eGenerated.lean) statements; Lean-checked over ℕ, outside the trusted base)" || { echo "M3eGenerated.lean is STALE (scripts/gen-m3e-lean.sh target/release/rlt > lean/M3eGenerated.lean)"; fail=1; }
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
  # RLT-005: the archived source is refused when the archive, the member or an
  # excerpt is tampered with (scratch copies; the real registry is untouched).
  ar_id=MCNEW-POMERANCE-MATCHABLE-V2; ar_file=2604.05304v2; ar_tmp=$(mktemp -d)
  if [ -f "$1/archived/$ar_file" ]; then
    mkdir -p "$ar_tmp/bad/archived" "$ar_tmp/good"
    { cat "$1/archived/$ar_file"; printf x; } > "$ar_tmp/bad/archived/$ar_file"
    cp -r "$1/archived" "$ar_tmp/good/archived"
    ar_t() {  # name, checkouts, registry edit (python expr on s), required output
      python3 -c "import sys; s=open('sources/registry.yaml',encoding='utf-8').read(); s=$3; open(sys.argv[1],'w',encoding='utf-8').write(s)" "$ar_tmp/reg.yaml"
      ar_log=$(python3 -I scripts/check_excerpts.py --registry "$ar_tmp/reg.yaml" --only $ar_id "$2" 2>&1); ar_code=$?
      { [ $ar_code = 1 ] && echo "$ar_log" | grep -qF -- "$4"; } && echo "archived source refused: $1" || { echo "  UNEXPECTED: archived source $1 not refused (exit $ar_code)"; fail=1; }
    }
    ar_t "archive bytes changed" "$ar_tmp/bad" "s" "differs from the pin 21674e5f"
    ar_t "pin changed" "$ar_tmp/good" "s.replace('commit: 21674e5f','commit: 31674e5f')" "differs from the pin 31674e5f"
    ar_t "member changed to another file" "$ar_tmp/good" "s.replace('#match051426.tex','#anc/compute_census.py')" "MISSING  $ar_id"
    ar_t "member absent" "$ar_tmp/good" "s.replace('#match051426.tex','#absent.tex')" "ERROR    $ar_id"
    ar_t "member with a .. component" "$ar_tmp/good" "s.replace('#match051426.tex','#../match051426.tex')" "ERROR    $ar_id: member"
    ar_t "excerpt from a LaTeX comment" "$ar_tmp/good" "s.replace('However, 8 is not matchable, nor is any subsequent multiple of 4.','to d in last')" "MISSING  $ar_id: 'to d in last"
    ar_t "excerpt starting inside a word" "$ar_tmp/good" "s.replace('However, 8 is not matchable, nor is any subsequent multiple of 4.','owever, 8 is not matchable')" "MISSING  $ar_id: 'owever"
    ar_t "excerpt changed" "$ar_tmp/good" "s.replace('However, 8 is not matchable','However, 8 is matchable')" "MISSING  $ar_id: 'However, 8 is matchable"
  else
    echo "  UNEXPECTED: $1/archived/$ar_file absent after the excerpt check"; fail=1
  fi
  rm -rf "$ar_tmp"
  # RLT-006: the transcribed PDF source is refused when the PDF, the pin, the
  # tool, a page extraction, an item or an excerpt is tampered with. Scratch
  # copies: a temporary transcription file and a temporary registry.
  tr_id=AXIOM-BGP212-2026-09-03; tr_file=sources/transcriptions/$tr_id.yaml; tr_tmp=$(mktemp -d)
  tr_scratch=sources/transcriptions/_verify_tamper.yaml
  if [ -f "$1/archived/bgp212.pdf" ]; then
    mkdir -p "$tr_tmp/bad/archived" "$tr_tmp/good"
    { cat "$1/archived/bgp212.pdf"; printf x; } > "$tr_tmp/bad/archived/bgp212.pdf"
    cp -r "$1/archived" "$tr_tmp/good/archived"
    tr_t() {  # name, checkouts, transcription edit (python expr on t), registry edit (python expr on s), required output
      python3 -c "import sys; t=open('$tr_file',encoding='utf-8').read(); t=$3; open('$tr_scratch','w',encoding='utf-8').write(t)"
      python3 -c "import sys; s=open('sources/registry.yaml',encoding='utf-8').read(); s=s.replace('url: $tr_file','url: $tr_scratch'); s=$4; open(sys.argv[1],'w',encoding='utf-8').write(s)" "$tr_tmp/reg.yaml"
      tr_log=$(python3 -I scripts/check_excerpts.py --registry "$tr_tmp/reg.yaml" --only $tr_id "$2" 2>&1); tr_code=$?
      { [ $tr_code = 1 ] && echo "$tr_log" | grep -qF -- "$5"; } && echo "transcribed source refused: $1" || { echo "  UNEXPECTED: transcribed source $1 not refused (exit $tr_code): $(echo "$tr_log" | grep -vE '^ok' | head -2)"; fail=1; }
    }
    tr_t "PDF bytes changed" "$tr_tmp/bad" "t" "s" "differs from the pin 2b307ae2"
    tr_t "registry pin changed" "$tr_tmp/good" "t" "s.replace('commit: 2b307ae2','commit: 3b307ae2')" "differs from the registry pin"
    tr_t "page extraction hash changed" "$tr_tmp/good" "t.replace('extraction_sha256: 8107','extraction_sha256: 9107')" "s" "page 43: extraction SHA-256"
    tr_t "extracted text altered" "$tr_tmp/good" "t.replace('206, 210, 212}.\"','206, 210, 213}.\"',1)" "s" "h45-tuple: extracted text not on page 43"
    tr_t "transcribed numbers altered" "$tr_tmp/good" "t.replace('206, 210, 212]','206, 210, 213]',1)" "s" "h45-tuple: numbers"
    tr_t "omitted class altered" "$tr_tmp/good" "t.replace('numbers: [1, 1, 3, 6, 9, 3, 1, 13, 17, 4, 11, 4, 4, 3]','numbers: [1, 1, 3, 6, 9, 3, 1, 13, 17, 4, 11, 4, 4, 2]')" "s" "omitted-classes: numbers"
    tr_t "reviewer removed" "$tr_tmp/good" "t.replace('reviewer: \"Claude (RLT-006 implementation)\"','reviewer: \"\"')" "s" "reviewed_by must name each reviewer"
    tr_t "tool version changed" "$tr_tmp/good" "t.replace('version: \"24.02.0\"','version: \"24.02.1\"')" "s" "the transcription pins 'pdftotext version 24.02.1'"
    tr_t "excerpt not a reviewed item" "$tr_tmp/good" "t" "s.replace('Lemma 12.1. The set H₄₅ is an admissible 45-tuple of diameter 212.','Lemma 12.1. The set H₄₅ is an admissible 45-tuple of diameter 210.')" "excerpt is not a reviewed item text"
  else
    echo "  UNEXPECTED: $1/archived/bgp212.pdf absent after the excerpt check"; fail=1
  fi
  rm -rf "$tr_tmp" "$tr_scratch"
elif [ "${RLT_SKIP_EXCERPTS:-}" = 1 ]; then
  echo "SKIPPED (no SOURCE_CHECKOUTS_DIR; RLT_SKIP_EXCERPTS=1)"; skipped=1
else
  echo "FAILED (pass SOURCE_CHECKOUTS_DIR, or set RLT_SKIP_EXCERPTS=1 to skip explicitly)"; fail=1
fi

finish

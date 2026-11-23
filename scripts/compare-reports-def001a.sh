#!/usr/bin/env bash
# usage: scripts/compare-reports-def001a.sh OLD_RLT OLD_ROOT NEW_RLT NEW_ROOT
# RLT-DEF-001a (R1): byte-compares, between two rlt binaries each run on its
# own repository tree, the output and exit code of every existing command on
# every committed input that scripts/compare-reports.sh leaves out: `check`
# of every committed .rlt outside tests/, `prove` of every committed goal
# (certificates written under target/def001a-compare/), `discover --verify`
# of every committed discovery record, `explain` (text and json) of every
# statement record, `deps` (text and json) of every library entry and
# `--verify-lib`, and `compose` of every bundle. Then runs
# scripts/compare-reports.sh itself. Exit 0 only if everything is identical.
set -uo pipefail
old_bin=$(realpath "$1"); old_root=$(cd "$2" && pwd); new_bin=$(realpath "$3"); new_root=$(cd "$4" && pwd)
here=$(cd "$(dirname "$0")" && pwd)
old() { "$old_bin" --root "$old_root" "$@"; }
new() { "$new_bin" --root "$new_root" "$@"; }
cd "$old_root"
n=0; d=0
same() { # label, command...
  local label=$1; shift
  n=$((n+1))
  local a b
  a=$(old "$@" 2>&1; echo "exit $?"); b=$(new "$@" 2>&1; echo "exit $?")
  if [ "$a" != "$b" ]; then d=$((d+1)); echo "DIFF $label"; diff <(echo "$a") <(echo "$b") | head -8; fi
}
for f in $(git ls-files '*.rlt' | grep -v '^tests/' | sort); do same "check $f" check "$f"; done
echo "committed proof files outside tests/: compared"
for g in $(git ls-files '*.goal' | sort); do
  out="target/def001a-compare/$(echo "$g" | tr '/' '_').rlt"
  rm -f "$old_root/$out" "$new_root/$out"
  same "prove $g" prove "$g" --out "$out"
done
echo "goals: compared"
for m in $(git ls-files '*manifest.tsv' | sort); do same "discover --verify $(dirname "$m")" discover --verify "$(dirname "$m")"; done
for s in $(git ls-files 'statements/*.yaml' | sort); do
  same "explain $s" explain "$s"
  same "explain $s json" explain "$s" --format json
done
echo "statement records: compared"
for l in $(git ls-files 'lib/*.yaml' | sort); do
  id=$(basename "$l" .yaml)
  same "deps $id" deps "$id"
  same "deps $id json" deps "$id" --format json
done
same "deps --verify-lib" deps --verify-lib
same "deps --verify-lib json" deps --verify-lib --format json
echo "library: compared"
for b in $(git ls-files 'compose/*.yaml' | sort); do same "compose $b" compose "$b"; done
echo "extended comparisons: $n compared, $d differ"
"$here/compare-reports.sh" "$old_bin" "$old_root" "$new_bin" "$new_root"; base=$?
[ "$d" = 0 ] && [ "$base" = 0 ]

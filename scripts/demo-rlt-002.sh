#!/usr/bin/env bash
# RLT-002 local demonstration: one checked equation associated with several
# distinct reading points, following the RLT-002 workflow. Read-only: it
# writes nothing except a scratch copy for the refused variant.
#
# usage: scripts/demo-rlt-002.sh
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --release -q
rlt="${CARGO_TARGET_DIR:-target}/release/rlt"
pts=points/oxygen-and-people.yaml
proof=tests/accept/T6_one_plus_one.rlt
step() { printf '\n\033[1m== %s\033[0m\n' "$1"; }

step "1 · Select a specification record"
grep -E '^(id|context):' "$pts"

step "2 · Its source basis and the named reading point it references"
grep -A2 '^basis:' "$pts"; grep '^references:' "$pts"

step "3 · State a claim: each count's written equation and convention"
grep -E '^  - id: count|^    (written|convention|mode|groups):' "$pts"

step "4 · The derivation"
cat "$proof"

step "5 · Run the checker"
"$rlt" check "$proof" | grep -E '^(checked goal|proved|assumptions|axiom set|  sha256|  status|verdict|  scope)'

step "6–7 · Inspect the specification records beside the result"
"$rlt" points "$pts"

step "Refused variant: equal counts offered as evidence that oxygen and a person share a structure"
scratch=$(mktemp -d)
mkdir -p "$scratch/rlt"
git ls-files -co --exclude-standard | tar -cf - -T - | tar -xf - -C "$scratch/rlt"
python3 - "$scratch/rlt/$pts" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read().replace("relations: []\n", """relations:
  - id: rel.same
    type: same-structure
    between: [o2.atom-1, person.a]
    status: supported
    evidence:
      - type: kernel_theorem
        ref: tests/accept/T6_one_plus_one.rlt
""")
open(p, "w").write(s)
PY
"$rlt" points --root "$scratch/rlt" "$pts" | grep -E '^(relation|problem|verdict)' || true
rm -rf "$scratch"

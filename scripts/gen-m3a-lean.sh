#!/usr/bin/env bash
# usage: scripts/gen-m3a-lean.sh RLT_BINARY > lean/M3aGenerated.lean
# Lean replay of the RLT-003 M3a accepted theorems (tests/m3a/accept): each
# theorem states exactly the kernel's checked formula over ℕ; Lean proves it
# independently with a fixed list of core tactics. Outside the trusted base.
set -euo pipefail
"$1" lean tests/m3a/accept/*.rlt | sed \
  -e 's#regenerate with `rlt lean tests/accept/\*\.rlt > lean/Generated\.lean`\.#regenerate with `scripts/gen-m3a-lean.sh target/release/rlt > lean/M3aGenerated.lean`.#' \
  -e 's#Run: cd lean \&\& lean Generated\.lean#Run: cd lean \&\& lean M3aGenerated.lean#' \
  -e 's#^namespace RLTGenerated$#namespace RLTGeneratedM3a#' \
  -e 's#^end RLTGenerated$#end RLTGeneratedM3a#'

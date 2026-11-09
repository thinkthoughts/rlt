#!/usr/bin/env bash
# usage: scripts/gen-m3c-lean.sh RLT_BINARY > lean/M3cGenerated.lean
# Lean replay of the RLT-003 M3c accepted theorems (tests/m3c/accept): each
# theorem states exactly the kernel's checked formula over ℕ; Lean proves it
# independently with a fixed list of core tactics. Outside the trusted base.
set -euo pipefail
"$1" lean tests/m3c/accept/*.rlt | sed \
  -e 's#regenerate with `rlt lean tests/accept/\*\.rlt > lean/Generated\.lean`\.#regenerate with `scripts/gen-m3c-lean.sh target/release/rlt > lean/M3cGenerated.lean`.#' \
  -e 's#Run: cd lean \&\& lean Generated\.lean#Run: cd lean \&\& lean M3cGenerated.lean#' \
  -e 's#^namespace RLTGenerated$#namespace RLTGeneratedM3c#' \
  -e 's#^end RLTGenerated$#end RLTGeneratedM3c#'

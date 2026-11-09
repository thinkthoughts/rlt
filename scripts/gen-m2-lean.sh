#!/usr/bin/env bash
# usage: scripts/gen-m2-lean.sh RLT_BINARY > lean/M2Generated.lean
# Tier 1 of the M2 Lean comparison: `rlt lean` over the 111 M2 proof files,
# with the header and namespace naming this file. Statements are unchanged.
set -euo pipefail
"$1" lean tests/m2/*.rlt | sed \
  -e 's#regenerate with `rlt lean tests/accept/\*\.rlt > lean/Generated\.lean`\.#regenerate with `scripts/gen-m2-lean.sh target/release/rlt > lean/M2Generated.lean`.#' \
  -e 's#Run: cd lean \&\& lean Generated\.lean#Run: cd lean \&\& lean M2Generated.lean#' \
  -e 's#^namespace RLTGenerated$#namespace RLTGeneratedM2#' \
  -e 's#^end RLTGenerated$#end RLTGeneratedM2#'

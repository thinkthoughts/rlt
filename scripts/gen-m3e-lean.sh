#!/usr/bin/env bash
# usage: scripts/gen-m3e-lean.sh RLT_BINARY > lean/M3eGenerated.lean
# Lean replay of the RLT-003 M3e accepted theorems (tests/m3e/accept): each
# theorem states exactly the kernel's checked formula over ℕ; Lean proves it
# independently with a fixed list of core tactics. Outside the trusted base.
set -euo pipefail
# RLT-003 M3e (decision D7): the two statements over the bound r < 30 that the
# fixed core tactic list leaves open (M3e_E_FWD, M3e_E_T1) get one bounded-case
# tactic instead (E_T1 combines the Lean proofs of E_FWD and E_BWD, as RLT's
# E-T1 cites them); the statements themselves are exactly as `rlt lean` exports
# them, and every other theorem keeps the fixed list.
BOUNDED='(intro r ⟨z, hz⟩; have hr : r < 30 := (by omega); clear hz z; revert r; simp only [show ∀ d n : Nat, (∃ k, n = d * k) ↔ d ∣ n from fun _ _ => Iff.rfl]; decide)'
"$1" lean tests/m3e/accept/*.rlt | sed \
  -e 's#regenerate with `rlt lean tests/accept/\*\.rlt > lean/Generated\.lean`\.#regenerate with `scripts/gen-m3e-lean.sh target/release/rlt > lean/M3eGenerated.lean`.#' \
  -e 's#Run: cd lean \&\& lean Generated\.lean#Run: cd lean \&\& lean M3eGenerated.lean#' \
  -e 's#^namespace RLTGenerated$#namespace RLTGeneratedM3e#' \
  -e 's#^end RLTGenerated$#end RLTGeneratedM3e#' \
  | sed -E -e "/^theorem M3e_E_FWD /s/:= by first \\|.*$/:= by $BOUNDED/" \
        -e "/^theorem M3e_E_T1 /s/:= by first \\|.*$/:= fun r h => ⟨M3e_E_FWD r h, M3e_E_BWD r⟩/"

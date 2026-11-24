# [ucla-roms] Check the base commit before comparing outputs against it

Branch (fork, published): `rlt-cstar/f028-base-sha` `79b85c8a` on `main` `bf7b5f4d` — 1 commit. CI-only change.

# Summary

When hashes mismatch, the pytest jobs rerun the suite on the base commit and report how far each output moved. The guard tested only for an empty `BASE_SHA`. On a branch's first push `github.event.before` is 40 zeros: `actions/checkout` fails three times ("not our ref 0000…"), the base run exits 2, and no report is written. (Seen in a fork run: thinkthoughts/ucla-roms run 45245982301, both pytest jobs.)

Both "Look for hash mismatches to report" steps now skip the comparison, with a message, when the base is all zeros, is not a 40-character SHA, or cannot be fetched as a commit (`git fetch --depth=1 origin <sha>`, then `git cat-file -e`), matching the check the `prepare` job already does before diffing the Containerfiles.

How it was tested: a harness runs each step's own `run:` text in a depth-1 clone of a bare origin that serves any SHA (as GitHub does), for empty, all-zero, valid, unknown, malformed (`main; touch PWNED`) and 12-character values, with and without a mismatches file:
- unmodified `main`: 8 of 24 outcomes wrong (all-zero, unknown, malformed and short values go on to checkout), on both jobs;
- this commit: 24 of 24 correct; nothing in the value is executed.

The diff adds 20 lines inside the two steps and touches no job condition, permission, login, build-push or package-delete line. A fork CI run of a branch carrying this change (run 45288508832) passed with zero mismatches, so the new branches of the step were exercised only by the harness.

Registry safety: the change sits in the pytest jobs only; `build_gnu`, `build_ifx` and `cleanup` and their conditions are untouched.

Follow-up option, not in this PR: on a first push, compare against the merge base with `main` instead of skipping.

No release note (CI only; label `no-changelog`).

AI disclosure: Claude (Anthropic); wrote the change, the test harness and this description under the author's direction and review.

## Code Review Checklist
- [x] PR is labeled `no-changelog`
- [ ] Any updates to namelist options or cppdefs are being propagated to C-Star/Forge (none)
- [x] `pre-commit` (changed files) passing

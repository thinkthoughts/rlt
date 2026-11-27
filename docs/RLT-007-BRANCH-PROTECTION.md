# RLT-007 — Branch protection for `main`: minimal ruleset and required status checks

**Status:** prepared, **not applied**. Applying any repository setting needs separate approval. This updates `docs/RLT-005-BRANCH-PROTECTION.md` with the RLT-007a CI layout.

## Current state (read-only queries, 2026-11-12)

| Query | Result |
|---|---|
| `gh api repos/thinkthoughts/rlt/branches/main/protection` | 404 "Branch not protected" |
| `gh api repos/thinkthoughts/rlt/rules/branches/main` | `[]` (no rules apply to `main`) |
| Repository | public, user-owned; merge commit, squash and rebase all allowed |
| CI jobs on push and pull request (`.github/workflows/ci.yml`) | `check` is the gate: format, lints, then `scripts/verify.sh`, which runs `cargo test` and every check. `gdt-comparison` is `continue-on-error`, reported separately. `source-provenance` runs on schedule and manual dispatch only, and is skipped on push and PR |
| App reporting `check` | GitHub Actions, integration id 15368 |

## 1. The minimal ruleset (prevents force-push and deletion)

```bash
gh api -X POST repos/thinkthoughts/rlt/rulesets --input - <<'JSON'
{
  "name": "protect-main",
  "target": "branch",
  "enforcement": "active",
  "conditions": {"ref_name": {"include": ["refs/heads/main"], "exclude": []}},
  "bypass_actors": [],
  "rules": [
    {"type": "deletion"},
    {"type": "non_fast_forward"}
  ]
}
JSON
```

- **`deletion`:** `main` cannot be deleted.
- **`non_fast_forward`:** every update must extend `main`'s history. Merging a PR with a merge commit is such an extension, so the established workflow is unaffected.
- **No `required_linear_history`:** it would forbid merge commits.
- **Empty bypass list:** the rules also bind the repository owner.

`scripts/check-identities.sh` follows `main`'s first-parent history, and the handoff records merge commits by hash. This ruleset makes both assumptions enforced, not just conventional.

## 2. Adding the required status check

The optional third rule, added to `rules`:

```json
{"type": "required_status_checks",
 "parameters": {
   "strict_required_status_checks_policy": false,
   "do_not_enforce_on_create": false,
   "required_status_checks": [{"context": "check", "integration_id": 15368}]}}
```

**Findings: whether it interferes with the complete verification gate.**

- **It enforces the gate rather than replacing it.** The required context is exactly the `check` job, which runs the complete `scripts/verify.sh`. Nothing about the gate changes, and no partial or fast job becomes sufficient.
- **The job runs where the rule needs it.** `check` runs on every push and every pull request; only scheduled runs skip it, and schedules never produce PR merges. On a PR, GitHub evaluates the checks of the head commit. Both the push run and the PR run report `check` on that SHA, and the merge is allowed only when they pass.
- **The app is pinned.** `integration_id: 15368` is GitHub Actions, so no other app or token can satisfy the check by posting a status named `check`.
- **Non-gate jobs stay optional.** `gdt-comparison` (continue-on-error by design) and `source-provenance` (schedule only) are deliberately excluded. A skipped required job would count as passing, so neither should be listed.
- **`strict` stays off.** Every RLT branch is cut from the current `main`. Turning `strict` on would force a re-run whenever `main` moves first, adding a 10–15 minute wait without adding coverage.
- **What it blocks, intentionally:**
  - merging any PR whose `check` failed or has not finished. The full run takes about 10–15 min on CI;
  - direct pushes to `main` of commits that have not passed `check`. The workflow never pushes directly to `main`: handoff records land through the next PR, as they do now.
- **Availability risk.** If GitHub Actions is down, merges wait. With an empty bypass list, the owner cannot override. If that is unwanted, a bypass entry for the repository admin role in `pull_request` mode would allow an audited manual merge; it is not included by default.

**Conclusion.** The required status check can be enabled without interfering with the complete verification gate, and it makes the gate binding. The recommendation is to apply rules 1 and 2 together after approval.

## 3. Optional, separate setting

Disallow squash and rebase merges in the repository settings (`allow_squash_merge: false`, `allow_rebase_merge: false`), so every PR lands as a merge commit, as the handoff records. This is a repository setting, not part of the ruleset, and is listed for completeness.

## 4. Verifying and undoing (after approval only)

- **Check the rules.** `gh api repos/thinkthoughts/rlt/rules/branches/main` lists `deletion`, `non_fast_forward` and (if added) `required_status_checks`.
- **Force-push.** From a scratch clone, `git push --force` to `main` is refused, and the message names the ruleset.
- **Merging.** The next PR shows `check` as required and merges with a merge commit once it passes.
- **Undo.** `gh api -X DELETE repos/thinkthoughts/rlt/rulesets/<id>` removes the ruleset.

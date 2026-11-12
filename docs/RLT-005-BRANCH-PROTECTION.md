# RLT-005 — Recommendation: protect `main` against force-pushes and deletion

**Status: recommendation only, not applied.** Changing repository settings needs separate approval.

## Current state (read-only queries, 2026-11-11)

| Query | Result |
|---|---|
| `gh api repos/thinkthoughts/rlt/branches/main/protection` | 404 "Branch not protected" |
| `gh api repos/thinkthoughts/rlt/rulesets` | `[]` (no rulesets) |
| Merge methods (`gh api repos/thinkthoughts/rlt`) | merge commit, squash and rebase all allowed; `delete_branch_on_merge` false |
| CI jobs (`.github/workflows/ci.yml`) | `check` (the gate: `cargo test`, `scripts/verify.sh`); `gdt-comparison` (`continue-on-error: true`, reported separately) |

So today anyone with write access can force-push `main` or delete it. The identity registry check (`scripts/check-identities.sh`, which follows main's first-parent history) and the merge records in `docs/RLT-REVIEW-HANDOFF.md` both assume main's history is never rewritten.

## Recommendation: one repository ruleset

| Setting | Value | Why |
|---|---|---|
| Target | branch `main` (`refs/heads/main`), enforcement **active** | — |
| Bypass list | **empty** | The rules then apply to administrators too |
| Rule `deletion` | on | `main` cannot be deleted |
| Rule `non_fast_forward` | on | No force-push. Every push to `main` must extend its history. Merging a PR with a merge commit is a fast-forward of `main`, so it is unaffected |
| `required_linear_history` | **off** | It would forbid merge commits, which this workflow uses |
| Require pull request / reviews | off (optional later) | Not requested. The current workflow merges reviewed PRs by hand |
| Required status check `check` | optional | It would block merging a PR whose `check` job failed. Leave `gdt-comparison` out: it is `continue-on-error` by design |

Repository merge settings stay as they are: merge commits allowed. Optionally, squash and rebase merges could be disallowed, so that PRs always land as merge commits as recorded in the handoff. That is a separate setting and not part of this recommendation.

**Command, to run only after approval** (it creates the ruleset; nothing else changes):

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

To also require the CI gate, add this rule to `rules`:
`{"type": "required_status_checks", "parameters": {"strict_required_status_checks_policy": false, "required_status_checks": [{"context": "check"}]}}`.

**How to verify after applying.**
- `gh api repos/thinkthoughts/rlt/rules/branches/main` lists `deletion` and `non_fast_forward`.
- `git push --force` to `main` from a scratch clone is refused, with a message naming the ruleset.
- The next PR merge with a merge commit still succeeds.

**How to undo.** Delete the ruleset, by its id, with `gh api -X DELETE repos/thinkthoughts/rlt/rulesets/<id>`.

## Alternative: classic branch protection

`PUT repos/thinkthoughts/rlt/branches/main/protection` with `allow_force_pushes: false`, `allow_deletions: false`, `enforce_admins: true`, `required_linear_history: false`, `required_status_checks: null` (or `{"strict": false, "contexts": ["check"]}`), `required_pull_request_reviews: null`, `restrictions: null`. This is equivalent in effect. A ruleset is preferred because it can be added or removed on its own and lists exactly the two rules requested.

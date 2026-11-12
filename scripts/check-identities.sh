#!/usr/bin/env bash
# RLT-004c: lib/identities.tsv is append-only once it reaches main. Every
# identity line in every version of the file on BASE's history (default:
# RLT_LIB_BASE, else origin/main, fetched if absent) must be present,
# unchanged, in the working tree. On main itself BASE is the commit under
# test, so its whole history counts; on a branch, main's versions count and
# the branch's own unmerged drafts, and drafts merged in from a branch
# (followed by first parent only), are free to change. A shallow clone is
# deepened first.
# Exit 0 if every earlier line is kept, 1 if a line was changed or removed or
# the file is missing, 2 if the history cannot be read.
set -u
[ -f lib/identities.tsv ] || { echo "identity registry: lib/identities.tsv is missing"; exit 1; }
if [ "$(git rev-parse --is-shallow-repository 2>/dev/null)" = true ]; then
  git fetch -q --unshallow 2>/dev/null || git fetch -q --deepen=100000 2>/dev/null || true
fi
if [ "$(git rev-parse --is-shallow-repository 2>/dev/null)" != false ]; then
  echo "identity registry: full history unavailable (shallow clone or no git); cannot confirm append-only"; exit 2
fi
base=${1:-${RLT_LIB_BASE:-origin/main}}
if [ "$base" = origin/main ] && ! git rev-parse -q --verify "origin/main^{commit}" >/dev/null 2>&1; then
  git fetch -q origin +refs/heads/main:refs/remotes/origin/main 2>/dev/null || true
  if [ "$(git rev-parse --is-shallow-repository 2>/dev/null)" = true ]; then
    git fetch -q --unshallow 2>/dev/null || true
  fi
fi
if ! git rev-parse -q --verify "$base^{commit}" >/dev/null 2>&1; then
  echo "identity registry: base $base unavailable; cannot confirm append-only"; exit 2
fi
# --first-parent: the versions main itself held; a merged branch's drafts
# are not main's versions.
commits=$(git rev-list --first-parent "$base" -- lib/identities.tsv 2>/dev/null)
n=0; bad=""
for c in $commits; do
  old=$(git show "$c:lib/identities.tsv" 2>/dev/null) || continue
  n=$((n + 1))
  miss=$(printf '%s\n' "$old" | grep -v '^#' | grep -v '^$' | grep -vxF -f lib/identities.tsv)
  [ -n "$miss" ] && bad="$bad$(printf '%s\n' "$miss" | sed "s/^/  ${c:0:7}: /")"$'\n'
done
if [ -n "$bad" ]; then
  echo "identity registry: lines of earlier versions changed or removed (an id keeps its fingerprint and first proof forever):"
  printf '%s' "$bad" | sort -u; exit 1
fi
echo "identity registry: every line of $n committed version(s) is kept ($(grep -cv '^#' lib/identities.tsv) identities)"

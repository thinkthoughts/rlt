#!/usr/bin/env bash
# RLT-004c: lib/identities.tsv is append-only. Every identity line present at
# BASE (default: RLT_LIB_BASE, else origin/main) must be present, unchanged,
# in the working tree. Exit 0 if so, 1 if a line was changed or removed,
# 2 if BASE cannot be read.
set -u
base=${1:-${RLT_LIB_BASE:-origin/main}}
if ! git rev-parse -q --verify "$base^{commit}" >/dev/null 2>&1; then
  git fetch -q --depth=1 origin main:refs/remotes/origin/main 2>/dev/null || true
fi
if ! git rev-parse -q --verify "$base^{commit}" >/dev/null 2>&1; then
  echo "identity registry: base $base unavailable"; exit 2
fi
if ! old=$(git show "$base:lib/identities.tsv" 2>/dev/null); then
  echo "identity registry: lib/identities.tsv is new relative to $base; append-only from here"; exit 0
fi
missing=$(printf '%s\n' "$old" | grep -v '^#' | grep -v '^$' | grep -vxF -f lib/identities.tsv)
if [ -n "$missing" ]; then
  echo "identity registry: lines at $base changed or removed (an id keeps its fingerprint and first proof forever):"
  printf '  %s\n' "$missing"; exit 1
fi
echo "identity registry: every line at $base is unchanged ($(printf '%s\n' "$old" | grep -cv '^#') identities)"

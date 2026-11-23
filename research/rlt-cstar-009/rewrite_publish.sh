#!/bin/bash
# Usage: rewrite_publish.sh REPO_DIR BASE SRC_BRANCH NEW_BRANCH MAPFILE
# Recreates SRC_BRANCH's commits on BASE with identical trees; messages lose AI
# trailers and local cherry-pick references; committer = author (noreply).
set -euo pipefail
cd "$1"; base=$2; src=$3; new=$4; map=$5
parent=$(git rev-parse $base)
for c in $(git rev-list --reverse $base..$src); do
  msg=$(git log -1 --format=%B $c | grep -vE '^(Co-Authored-By|Claude-Session):|^\(cherry picked from commit [0-9a-f]+\)$' | sed -e :a -e '/^\n*$/{$d;N;ba' -e '}')
  n=$(GIT_AUTHOR_NAME="$(git log -1 --format=%an $c)" GIT_AUTHOR_EMAIL="$(git log -1 --format=%ae $c)" GIT_AUTHOR_DATE="$(git log -1 --format=%aI $c)" \
      GIT_COMMITTER_NAME="thinkthoughts research (Claude)" GIT_COMMITTER_EMAIL="thinkthoughts@users.noreply.github.com" \
      git commit-tree $(git rev-parse $c^{tree}) -p $parent -m "$msg")
  echo "$c $n" >> $map; parent=$n
done
git branch -f $new $parent
[ "$(git rev-parse $new^{tree})" = "$(git rev-parse $src^{tree})" ] && echo "$new: tree identical to $src ($(git rev-list --count $base..$new) commits)"

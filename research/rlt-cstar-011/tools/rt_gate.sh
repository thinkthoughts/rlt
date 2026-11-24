#!/bin/bash
# roms-tools PR-branch gate: pre-commit on the branch's changed files, then targeted tests.
n=$1; shift; cd /home/claude/w11/rt_$n || exit 1; L=/home/claude/w11/gate_rt_$n.log
. /home/claude/venv/bin/activate
{ echo "branch pr/$n $(git log --format=%h -1)"; python -c "import roms_tools;print('roms_tools from', roms_tools.__file__)"
  files=$(git diff --name-only origin/main HEAD)
  pre-commit run --files $files 2>&1 | grep -E "Passed|Failed|Skipped|error:" ; echo "pre-commit exit ${PIPESTATUS[0]}"
  python -m pytest "$@" -q -p no:cacheprovider 2>&1 | tail -2; } > $L 2>&1

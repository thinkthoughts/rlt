#!/bin/bash
# C-Star unit tests in one pytest process per chunk (a test directory or top-level
# file), so the suite fits in this machine's memory. Usage: cs_unit_chunked.sh COMMIT...
. /home/claude/w10/venv_cs/bin/activate
for C in "$@"; do
  T=/home/claude/w10/cs_$C; L=/home/claude/w10/cs_gate_$C.unit.log; cd $T || exit 1
  python -c "import cstar,roms_tools;print('cstar from',cstar.__file__,'roms_tools',roms_tools.__version__)" > $L
  chunks=$(cd cstar/tests/unit_tests && { ls -d */ | grep -v __pycache__ | sed 's|/$||'; ls test_*.py; })
  for c in $chunks; do
    python -m pytest -q -p no:cacheprovider --basetemp=/tmp/csu_$C cstar/tests/unit_tests/$c > /tmp/csu_$C.out 2>&1
    echo "exit[$c]=$? $(grep -E '(passed|failed|error)' /tmp/csu_$C.out | tail -1)" >> $L
    grep -E "^(FAILED|ERROR) " /tmp/csu_$C.out >> $L
    rm -rf /tmp/csu_$C
  done
  echo done >> $L
done

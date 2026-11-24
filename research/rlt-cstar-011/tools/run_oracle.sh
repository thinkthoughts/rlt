#!/bin/bash
# Usage: run_oracle.sh WORKTREE OUT.jsonl
W=$1; OUT=$2; rm -f $OUT
cp /home/claude/w11/f029v2/oracle_test.py $W/cstar/tests/integration_tests/forge/test_zz_rlt_oracle.py
cd $W && . /home/claude/w10/venv_cs/bin/activate && RLT_ORACLE_OUT=$OUT python -m pytest cstar/tests/integration_tests/forge/test_zz_rlt_oracle.py -q -p no:cacheprovider > $OUT.log 2>&1; echo "exit $?" >> $OUT.log
rm $W/cstar/tests/integration_tests/forge/test_zz_rlt_oracle.py

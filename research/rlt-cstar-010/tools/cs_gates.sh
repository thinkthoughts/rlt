#!/bin/bash
. /home/claude/w10/venv_cs/bin/activate
for c in "$@"; do
  cd /home/claude/w10/cs_$c || exit 1
  python -c "import cstar;print('cstar from', cstar.__file__)" > /home/claude/w10/cs_gate_$c.tier1.log
  python -m pytest cstar/tests/integration_tests/test_fixtures.py cstar/tests/integration_tests/forge -q -rs -p no:cacheprovider >> /home/claude/w10/cs_gate_$c.tier1.log 2>&1; echo "exit $?" >> /home/claude/w10/cs_gate_$c.tier1.log
  python -c "import cstar;print('cstar from', cstar.__file__)" > /home/claude/w10/cs_gate_$c.unit.log
  python -m pytest cstar/tests/unit_tests -q -p no:cacheprovider >> /home/claude/w10/cs_gate_$c.unit.log 2>&1; echo "exit $?" >> /home/claude/w10/cs_gate_$c.unit.log
done

#!/bin/bash
# Untracked overlay in ROMS_ROOT/tests/rlt_ana/: the three analytical tests with an
# empty input directory (they blank grid, initial and forcing files), so the session
# skips generating the realistic inputs (which need TPXO data).
T=$1/tests; D=$T/rlt_ana; mkdir -p $D
: > $D/__init__.py
cat > $D/conftest.py <<'PY'
import pytest
@pytest.fixture(scope="session")
def input_dir(tmp_path_factory):
    return tmp_path_factory.mktemp("roms_inputs_empty")
PY
sed 's/^from \._/from .._/' $T/test_legacy_analytical_configs.py > $D/test_legacy_analytical_configs.py

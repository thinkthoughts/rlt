"""Scratch-only patch (never committed): make the suite's input_dir fixture copy the
CI-generated inputs from $RLT_INPUTS instead of regenerating them, so base and head
runs read bit-identical inputs (the same files CI's head run used)."""
import sys, pathlib
p = pathlib.Path(sys.argv[1]) / "tests" / "conftest.py"
s = p.read_text()
old = "    create_roms_inputs(target)\n"
new = ("    if os.environ.get(\"RLT_INPUTS\"):  # RLT scratch patch\n"
       "        for f in sorted(Path(os.environ[\"RLT_INPUTS\"]).glob(\"*.nc\")):\n"
       "            shutil.copy2(f, target / f.name)\n"
       "    else:\n"
       "        create_roms_inputs(target)\n")
assert s.count(old) == 1, "pattern"
assert "import os" in s and "import shutil" in s and "from pathlib import Path" in s
p.write_text(s.replace(old, new))
print("patched", p)

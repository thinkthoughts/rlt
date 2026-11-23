"""Scratch-only patch, step 2: after copying the CI inputs, split them across the
2x2 decomposition exactly as create_roms_inputs does (same partit commands)."""
import sys, pathlib
p = pathlib.Path(sys.argv[1]) / "tests" / "conftest.py"
s = p.read_text()
old = "            shutil.copy2(f, target / f.name)\n"
new = old + ("        import subprocess as _sp  # RLT scratch patch\n"
             "        _sp.run(\"for F in example_input_*;do partit 2 2 ${F};done\", text=True, shell=True, cwd=target, check=True)\n"
             "        _sp.run(\"partit 2 2 cdr_forcing_3d.nc\", text=True, shell=True, cwd=target, check=True)\n")
assert s.count(old) == 1
p.write_text(s.replace(old, new)); print("patched", p)

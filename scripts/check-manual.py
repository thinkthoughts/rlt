#!/usr/bin/env python3
"""Check every terminal example in docs/MANUAL.md against the real program.

usage: python3 -I scripts/check-manual.py RLT_BINARY_DIR [MANUAL]

Each ```console block holds commands (lines starting with "$ ") followed by
the output they are shown to produce. Commands run with bash from the
repository root, with RLT_BINARY_DIR first on PATH. Every shown output line
must appear, exactly and in order, in the actual output (stdout and stderr
together). A line "…" stands for omitted output; a block without "…" must
match the whole output exactly. Exit 0 only if every example matches.
"""
import os
import subprocess
import sys
from pathlib import Path

root = Path(__file__).resolve().parent.parent
bindir = str(Path(sys.argv[1]).resolve())
manual = Path(sys.argv[2]) if len(sys.argv) > 2 else root / "docs/MANUAL.md"
env = dict(os.environ, PATH=bindir + os.pathsep + os.environ.get("PATH", ""))
if not any((Path(p) / "lean").exists() for p in env["PATH"].split(os.pathsep)):
    elan = Path.home() / ".elan/bin"
    if (elan / "lean").exists():
        env["PATH"] += os.pathsep + str(elan)

examples, block = [], None
for n, line in enumerate(manual.read_text(encoding="utf-8").splitlines(), 1):
    if block is None:
        if line.strip() == "```console":
            block = []
        continue
    if line.strip() == "```":
        block = None
        continue
    if line.startswith("$ "):
        examples.append((n, line[2:], []))
    elif examples:
        examples[-1][2].append(line.rstrip())

failed = 0
for n, cmd, shown in examples:
    r = subprocess.run(["bash", "-c", cmd], cwd=root, env=env, capture_output=True, text=True)
    actual = [l.rstrip() for l in (r.stdout + r.stderr).splitlines()]
    want = [l for l in shown if l != "…"]
    if "…" not in shown:
        ok = actual == want
        missing = None if ok else "(whole output differs)"
    else:
        i, missing = 0, None
        for w in want:
            while i < len(actual) and actual[i] != w:
                i += 1
            if i == len(actual):
                missing = w
                break
            i += 1
        ok = missing is None
    print(f"{'ok  ' if ok else 'FAIL'} line {n}: $ {cmd}")
    if not ok:
        failed += 1
        print(f"     first unmatched shown line: {missing}")
print(f"RESULT {len(examples) - failed}/{len(examples)} examples match")
sys.exit(1 if failed else 0)

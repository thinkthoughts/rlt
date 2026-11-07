#!/usr/bin/env python3
"""Check that every registry excerpt appears verbatim in its source file.

usage: python3 -I scripts/check_excerpts.py CHECKOUTS_DIR

CHECKOUTS_DIR holds clones named after the GitHub repositories
(news, rlt, translatepi, antiviolence, residue-manifold-learning,
specification-grammar), each checked out at the commit the registry records.
Sources without a GitHub file location (conversation records, foundational
references, missing sources) are listed as skipped.
"""
import re
import subprocess
import sys
from pathlib import Path

try:
    import yaml  # PyYAML
except ImportError:
    sys.exit("PyYAML is required: pip install pyyaml")

root = Path(__file__).resolve().parent.parent
checkouts = Path(sys.argv[1])
reg = yaml.safe_load((root / "sources/registry.yaml").read_text(encoding="utf-8"))

pat = re.compile(r"https://github\.com/thinkthoughts/([^/]+)/blob/([0-9a-f]+)/(.+)$")
bad = 0
for s in reg["sources"]:
    m = pat.match(s.get("location", ""))
    if not m or not s.get("excerpts"):
        print(f"skip     {s['id']} ({s['status']})")
        continue
    repo, commit, path = m.groups()
    try:
        text = subprocess.run(
            ["git", "-C", str(checkouts / repo), "show", f"{commit}:{path}"],
            check=True, capture_output=True, text=True,
        ).stdout
    except subprocess.CalledProcessError as e:
        print(f"ERROR    {s['id']}: {e.stderr.strip()}")
        bad += 1
        continue
    # Compare with runs of whitespace collapsed, so HTML indentation and
    # line breaks inside a passage match the single-line excerpt.
    if path.endswith(".json"):
        # JSON files store strings escaped; search the decoded string values too.
        import json

        def strings(v):
            if isinstance(v, str):
                yield v
            elif isinstance(v, dict):
                for w in v.values():
                    yield from strings(w)
            elif isinstance(v, list):
                for w in v:
                    yield from strings(w)

        text = text + "\n" + "\n".join(strings(json.loads(text)))
    flat = " ".join(text.split())
    for x in s["excerpts"]:
        ok = x in text or " ".join(x.split()) in flat
        print(f"{'ok' if ok else 'MISSING':8} {s['id']}: {x[:70]!r}")
        bad += 0 if ok else 1
print(f"\n{bad} problem(s)")
sys.exit(1 if bad else 0)

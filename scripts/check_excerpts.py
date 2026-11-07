#!/usr/bin/env python3
"""Check that every registry excerpt appears verbatim in its source file.

usage: python3 -I scripts/check_excerpts.py CHECKOUTS_DIR

CHECKOUTS_DIR holds clones named after the GitHub repositories, including
rlt itself: news, rlt, translatepi, antiviolence, residue-manifold-learning,
specification-grammar. Each excerpt is read with `git show COMMIT:PATH`, so the
clones may be at any revision as long as they contain the recorded commits.
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
        why = "quoted from the review conversation; no file to check" if s["status"] == "conversation" else s["status"]
        print(f"skip     {s['id']} ({why})")
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
    # Haystacks: the file text, plus (for JSON) each decoded string value on its
    # own, so an excerpt can never be stitched together from separate strings.
    haystacks = [text]
    if path.endswith(".json"):
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

        haystacks += list(strings(json.loads(text)))
    for x in s["excerpts"]:
        x = x["text"] if isinstance(x, dict) else x
        # Exact match, or match with whitespace runs collapsed (HTML indentation
        # and line breaks inside one passage), within a single haystack.
        ok = any(x in h or " ".join(x.split()) in " ".join(h.split()) for h in haystacks)
        print(f"{'ok' if ok else 'MISSING':8} {s['id']}: {x[:70]!r}")
        bad += 0 if ok else 1
print(f"\n{bad} problem(s)")
sys.exit(1 if bad else 0)

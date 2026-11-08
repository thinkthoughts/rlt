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
mention = re.compile(r"(?i)(?<![a-z0-9_])math(0|5|29)(?![0-9])")
OPEN, CLOSE = (">", '"', "'", "‘", "“"), ("<", '"', "'", "’", "”")


def bounded(text, x):
    """True if some occurrence of x in text sits on unit boundaries: it starts
    at a line start (after indentation), after '>', after an opening quote, or
    with its own tag '<'; and it ends at a line end, before '<', before a
    closing quote, or with its own tag '>'."""
    start = 0
    while (i := text.find(x, start)) >= 0:
        before = text[:i].rsplit("\n", 1)[-1]
        after = text[i + len(x):].split("\n", 1)[0]
        left = before.strip() == "" or before.endswith(OPEN) or x.startswith("<")
        right = after.strip() == "" or after.startswith(CLOSE) or x.endswith(">")
        if left and right:
            return True
        start = i + 1
    return False


bad = 0
for s in reg["sources"]:
    m = pat.match(s.get("location", ""))
    if s["status"] == "available" and s.get("excerpts") and not m:
        print(f"ERROR    {s['id']}: status available requires a GitHub file location at a commit")
        bad += 1
        continue
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
    for e in s["excerpts"]:
        x = e["text"] if isinstance(e, dict) else e
        tagged = isinstance(e, dict) and e.get("defines")
        # Exact match, or match with whitespace runs collapsed (HTML indentation
        # and line breaks inside one passage), within a single haystack.
        ok = any(x in h or " ".join(x.split()) in " ".join(h.split()) for h in haystacks)
        status = "ok" if ok else "MISSING"
        # A tagged excerpt documents a reading point: it must be a whole unit of
        # the source (bounded by line ends, tags or quotes), never a fragment.
        if ok and tagged and not any(bounded(h, x) for h in haystacks):
            status = "PARTIAL"
        print(f"{status:8} {s['id']}: {x[:70]!r}")
        bad += 0 if status == "ok" else 1
    # Coverage: every source line that names a reading point must overlap a
    # registered excerpt (tagged or context), so a documented meaning cannot
    # be dropped from both the registry and the record unnoticed.
    texts = [e["text"] if isinstance(e, dict) else e for e in s["excerpts"]]
    if not path.endswith(".json"):
        for n, line in enumerate(text.splitlines(), 1):
            if not mention.search(line):
                continue
            if not any(t.strip() and (t in line or line.strip() in t or any(part and part in line for part in t.split("\n"))) for t in texts):
                print(f"UNCOVERED {s['id']} line {n}: {line.strip()[:90]!r}")
                bad += 1
print(f"\n{bad} problem(s)")
sys.exit(1 if bad else 0)

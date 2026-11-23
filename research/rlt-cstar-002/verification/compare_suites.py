"""Compare two pytest -v logs (pinned vs patched): outcome counts and tests whose
outcome differs. Tests present in only one tree are listed as added/removed.
Usage: compare_suites.py BASE_LOG PATCHED_LOG [BASE_LOG2 PATCHED_LOG2 ...]  (pairs are merged)
"""

import re
import sys
from collections import Counter

PAT = re.compile(r"^(\S+::\S.*?) (PASSED|FAILED|ERROR|SKIPPED|XFAIL|XPASS)(?: |$)")


def load(paths):
    res = {}
    exits = []
    for path in paths:
        for line in open(path, encoding="utf-8", errors="replace"):
            if line.startswith("exit="):
                exits.append(line.strip())
                continue
            m = PAT.match(line.strip())
            if m:
                res[m.group(1)] = m.group(2)
    return res, exits


args = sys.argv[1:]
base, bexit = load(args[0::2])
patched, pexit = load(args[1::2])
print("base   ", dict(Counter(base.values())), bexit)
print("patched", dict(Counter(patched.values())), pexit)
common = base.keys() & patched.keys()
changed = sorted(t for t in common if base[t] != patched[t])
print(f"common tests {len(common)}; outcome changed in {len(changed)}")
for t in changed:
    print(f"  {base[t]:7s} -> {patched[t]:7s} {t}")
added = sorted(patched.keys() - base.keys())
removed = sorted(base.keys() - patched.keys())
print(f"only in patched: {len(added)} ({dict(Counter(patched[t] for t in added))})")
for t in added:
    if patched[t] != "PASSED":
        print(f"  {patched[t]:7s} {t}")
print(f"only in base: {len(removed)}")
for t in removed[:20]:
    print(f"  {base[t]:7s} {t}")

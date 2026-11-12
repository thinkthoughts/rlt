#!/usr/bin/env python3
"""RLT-007a (H3): fetch exactly the pinned source material for
scripts/check_excerpts.py and scripts/verify.sh into CHECKOUTS_DIR.

  * GitHub sources (status `available`): for each repository, every commit the
    registry pins is fetched by its SHA (`git fetch --depth 1 --filter=blob:none
    origin SHA`), so no branch is read and a moved or force-pushed branch has
    no effect. Files are then read with `git show SHA:PATH`.
  * Extra pins used by verify.sh outside the registry (lean-bgp212 at the
    commit RLT006BGP212.lean vendors).
  * Archived and transcribed sources (arXiv e-print, PDF): downloaded only when
    CHECKOUTS_DIR/archived/ lacks the file; CI keeps that directory in a cache
    keyed by the registry, so routine runs read no live URL. Every byte is
    still checked against its SHA-256 pin by scripts/check_excerpts.py.

Usage: fetch-sources.py CHECKOUTS_DIR      (idempotent)
       fetch-sources.py --check-pins       (offline: the pin table only)
Exit 0 when every pinned commit and file is present afterwards; with
--check-pins, when every abbreviated registry commit has exactly one full SHA
in sources/commit-pins.tsv that extends it, and every row is used.
"""
import hashlib
import os
import re
import subprocess
import sys
import urllib.request
from pathlib import Path

import yaml

root = Path(__file__).resolve().parent.parent
check_only = sys.argv[1:] == ["--check-pins"]
if len(sys.argv) != 2:
    sys.exit(__doc__)
reg = yaml.safe_load((root / "sources/registry.yaml").read_text(encoding="utf-8"))
pat = re.compile(r"^https://github\.com/([^/]+)/([^/]+)/blob/([0-9a-f]{7,40})/")
full = {}
used = set()
for line in (root / "sources/commit-pins.tsv").read_text(encoding="utf-8").splitlines():
    if line and not line.startswith("#"):
        repo, short, long_ = line.split("\t")
        if not long_.startswith(short) or len(long_) != 40:
            sys.exit(f"sources/commit-pins.tsv: {long_} does not extend {short}")
        if (repo, short) in full:
            sys.exit(f"sources/commit-pins.tsv: {repo} {short} is listed twice")
        full[(repo, short)] = long_

# Pins outside the registry: (directory, owner/repo, commit, reason).
EXTRA = [
    ("lean-bgp212", "thinkthoughts/lean-bgp212", "7898eb4665197ab418a62b9bb657dab3f31bb262",
     "lean/RLT006BGP212.lean vendors BGP212/Basic.lean and Solution/Lemma12_1.lean at this commit"),
]

repos = {}
for s in reg["sources"]:
    m = pat.match(s.get("location", ""))
    if m and s.get("status") == "available":
        owner, name, commit = m.groups()
        if len(commit) != 40:
            if (name, commit) not in full:
                sys.exit(f"{s['id']}: abbreviated commit {commit} has no row in sources/commit-pins.tsv")
            used.add((name, commit))
            commit = full[(name, commit)]
        repos.setdefault(name, (f"{owner}/{name}", set()))[1].add(commit)
for d, slug, commit, _ in EXTRA:
    repos.setdefault(d, (slug, set()))[1].add(commit)
if set(full) - used:
    sys.exit(f"sources/commit-pins.tsv: rows used by no registry source: {sorted(set(full) - used)}")
if check_only:
    print(f"sources/commit-pins.tsv: {len(full)} abbreviated registry commit(s) resolved to full SHAs; "
          f"{sum(len(c) for _, c in repos.values())} pinned commit(s) in {len(repos)} repositories, no branch named")
    sys.exit(0)
out = Path(sys.argv[1])
out.mkdir(parents=True, exist_ok=True)


def git(*args, cwd):
    # GIT_NO_LAZY_FETCH: a presence check must not fetch the object itself
    # from the promisor remote; fetching is explicit, by SHA, below.
    env = dict(os.environ, GIT_NO_LAZY_FETCH="1") if args[0] == "cat-file" else None
    return subprocess.run(["git", *args], cwd=cwd, capture_output=True, text=True, env=env)


bad = 0
for name, (slug, commits) in sorted(repos.items()):
    d = out / name
    if not (d / ".git").is_dir():
        d.mkdir(parents=True, exist_ok=True)
        git("init", "-q", cwd=d)
        git("remote", "add", "origin", f"https://github.com/{slug}", cwd=d)
        git("config", "remote.origin.promisor", "true", cwd=d)
        git("config", "remote.origin.partialclonefilter", "blob:none", cwd=d)
    for c in sorted(commits):
        if git("cat-file", "-e", f"{c}^{{commit}}", cwd=d).returncode == 0:
            print(f"present  {name} {c[:12]}")
            continue
        r = git("fetch", "-q", "--depth", "1", "--filter=blob:none", "origin", c, cwd=d)
        ok = r.returncode == 0 and git("cat-file", "-e", f"{c}^{{commit}}", cwd=d).returncode == 0
        print(f"{'fetched ' if ok else 'MISSING '} {name} {c[:12]}{'' if ok else ': ' + r.stderr.strip()[:200]}")
        bad += 0 if ok else 1

arch = out / "archived"
arch.mkdir(exist_ok=True)
for s in reg["sources"]:
    if s.get("status") == "archived":
        url = s["url"].partition("#")[0]
    elif s.get("status") == "transcribed":
        url = s["location"]
    else:
        continue
    f = arch / url.rstrip("/").rsplit("/", 1)[-1]
    if f.is_file() and not f.is_symlink():
        print(f"present  archived/{f.name} (checked against its pin by check_excerpts.py)")
        continue
    try:
        req = urllib.request.Request(url, headers={"User-Agent": "rlt-fetch-sources (scripts/fetch-sources.py)"})
        with urllib.request.urlopen(req, timeout=60) as r:
            data = r.read(50 * 1024 * 1024 + 1)
        if len(data) > 50 * 1024 * 1024:
            raise ValueError("larger than 50 MB")
        got = hashlib.sha256(data).hexdigest()
        if got != s["commit"]:
            raise ValueError(f"the download has SHA-256 {got}, not the pin {s['commit']}: the live file changed or the server returned something else; nothing written")
        tmp = f.with_suffix(f.suffix + ".part")
        tmp.write_bytes(data)
        os.replace(tmp, f)
        print(f"fetched  archived/{f.name} from {url} ({len(data)} bytes, equal to its SHA-256 pin)")
    except Exception as e:  # noqa: BLE001
        print(f"MISSING  archived/{f.name}: {e}")
        bad += 1
print(f"{len(repos)} repositories, {sum(len(c) for _, c in repos.values())} pinned commits; {bad} missing")
sys.exit(0 if bad == 0 else 1)

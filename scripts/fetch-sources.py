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
import shutil
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
    env = dict(os.environ, GIT_NO_REPLACE_OBJECTS="1")
    if args[0] in ("cat-file", "fsck"):
        env["GIT_NO_LAZY_FETCH"] = "1"
    # Defence in depth (re-audit): no repository-configured program runs.
    safe = ["-c", "core.fsmonitor=false", "-c", "core.hooksPath=/dev/null"]
    return subprocess.run(["git", *safe, *args], cwd=cwd, capture_output=True, text=True, env=env)


bad = 0
for name, (slug, commits) in sorted(repos.items()):
    d = out / name
    # Audit N1: a repository restored from a cache keeps only its objects.
    # Everything else in .git (config, hooks, refs including refs/replace,
    # info/grafts, objects/info/alternates) is untrusted and rebuilt here.
    # Re-audit: anything but a real directory holding a real `.git` directory
    # and a real `objects` directory (a `.git` file pointing elsewhere, or a
    # symlink at any of the three) is discarded and fetched again.
    g = d / ".git"
    if d.is_symlink() or (d.exists() and not d.is_dir()):
        d.unlink()
    elif d.is_dir() and (g.is_symlink() or not g.is_dir() or (g / "objects").is_symlink()
                         or not (g / "objects").is_dir()):
        print(f"discard  {name}: the cached checkout is not a plain repository; fetching again")
        shutil.rmtree(d)
    if (d / ".git").is_dir():
        objs = d / ".git" / "objects"
        keep = d.parent / f".{name}.objects"
        shutil.rmtree(keep, ignore_errors=True)
        os.replace(objs, keep)
        shutil.rmtree(d)
    if not (d / ".git").is_dir():
        d.mkdir(parents=True, exist_ok=True)
        git("init", "-q", cwd=d)
        git("remote", "add", "origin", f"https://github.com/{slug}", cwd=d)
        git("config", "remote.origin.promisor", "true", cwd=d)
        git("config", "remote.origin.partialclonefilter", "blob:none", cwd=d)
        keep = d.parent / f".{name}.objects"
        if keep.is_symlink() or (keep.exists() and not keep.is_dir()):
            keep.unlink()
        if keep.is_dir():
            # Only object data is adopted: objects/info (alternates and
            # http-alternates), a file or symlink, is always replaced.
            info = keep / "info"
            if info.is_symlink() or info.is_file():
                info.unlink()
            elif info.is_dir():
                shutil.rmtree(info)
            shutil.rmtree(d / ".git" / "objects")
            os.replace(keep, d / ".git" / "objects")
            (d / ".git" / "objects" / "info").mkdir(exist_ok=True)
    for c in sorted(commits):
        if git("cat-file", "-e", f"{c}^{{commit}}", cwd=d).returncode == 0:
            print(f"present  {name} {c[:12]}")
            continue
        r = git("fetch", "-q", "--depth", "1", "--filter=blob:none", "origin", c, cwd=d)
        ok = r.returncode == 0 and git("cat-file", "-e", f"{c}^{{commit}}", cwd=d).returncode == 0
        print(f"{'fetched ' if ok else 'MISSING '} {name} {c[:12]}{'' if ok else ': ' + r.stderr.strip()[:200]}")
        bad += 0 if ok else 1

# Audit S5: a repository restored from a cache may hold objects whose bytes
# do not match their names (git does not re-hash on read). `git fsck --full`
# re-hashes every object present; a mismatch fails the run.
for name in sorted(repos):
    r = git("fsck", "--full", "--no-dangling", "--no-progress", cwd=out / name)
    if r.returncode != 0:
        print(f"CORRUPT  {name}: git fsck --full failed: {(r.stdout + r.stderr).strip()[:300]}")
        bad += 1
print(f"git fsck --full: every object present in {len(repos)} repositories re-hashed")

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
print(f"{len(repos)} repositories, {sum(len(c) for _, c in repos.values())} pinned commits; {bad} missing or corrupt")
sys.exit(0 if bad == 0 else 1)

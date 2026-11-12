#!/usr/bin/env python3
"""RLT-007a (H3): fetch exactly the pinned source material for
scripts/check_excerpts.py and scripts/verify.sh into CHECKOUTS_DIR.

  * GitHub sources (every registry entry with a GitHub file location at a
    commit, the same selection as check_excerpts.py): each pinned commit is
    fetched by its full SHA (`git fetch --depth 1 --filter=blob:none origin
    SHA`), so no branch is read; abbreviated registry pins resolve through
    sources/commit-pins.tsv. Every file the readers will read is then
    materialized here, so the readers never fetch.
  * Extra pins used by verify.sh outside the registry (lean-bgp212 at the
    commit RLT006BGP212.lean vendors, and the two files compared).
  * A checkout already in CHECKOUTS_DIR (a CI cache) is untrusted: only its
    plain object files (regular files at loose-object or pack paths, opened
    without following links) are copied into a fresh repository made with
    this script's own config; everything else is deleted. `git fsck --full`
    then re-hashes every object. Git runs with no system or global config,
    no replace objects, no hooks or fsmonitor, and a time limit.
  * Archived and transcribed sources (arXiv e-print, PDF): a file already
    present is kept only as a regular file equal to its SHA-256 pin;
    otherwise it is downloaded, refused unless equal to its pin, written to a
    fresh temporary file and renamed into place.

The readers re-check everything themselves: scripts/pinned_read.py re-hashes
every object on the path it reads, using none of the checkout's metadata, and
check_excerpts.py re-hashes the archived files.

Usage: fetch-sources.py CHECKOUTS_DIR      (idempotent)
       fetch-sources.py --check-pins       (offline: the pin table only)
Exit 0 when every pinned commit and file is present afterwards; with
--check-pins, when every abbreviated registry commit has exactly one full SHA
in sources/commit-pins.tsv that extends it, and every row is used.
"""
import hashlib
import os
import shutil
import stat
import re
import subprocess
import sys
import tempfile
import urllib.request
from pathlib import Path

import yaml

root = Path(__file__).resolve().parent.parent
check_only = sys.argv[1:] == ["--check-pins"]
if len(sys.argv) != 2:
    sys.exit(__doc__)
reg = yaml.safe_load((root / "sources/registry.yaml").read_text(encoding="utf-8"))
pat = re.compile(r"^https://github\.com/([^/]+)/([^/]+)/blob/([0-9a-f]{7,40})/(.+)$")
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
     ("BGP212/Basic.lean", "Solution/Lemma12_1.lean")),
]

repos = {}
needed = {}  # name -> {(commit, path)}: the files the readers will read
for s in reg["sources"]:
    m = pat.match(s.get("location", ""))
    # The same selection as scripts/check_excerpts.py: every source with a
    # GitHub file location at a commit (final audit, note 4).
    if m:
        owner, name, commit, path = m.groups()
        if len(commit) != 40:
            if (name, commit) not in full:
                sys.exit(f"{s['id']}: abbreviated commit {commit} has no row in sources/commit-pins.tsv")
            used.add((name, commit))
            commit = full[(name, commit)]
        repos.setdefault(name, (f"{owner}/{name}", set()))[1].add(commit)
        needed.setdefault(name, set()).add((commit, path))
for d, slug, commit, paths in EXTRA:
    repos.setdefault(d, (slug, set()))[1].add(commit)
    needed.setdefault(d, set()).update((commit, p) for p in paths)
if set(full) - used:
    sys.exit(f"sources/commit-pins.tsv: rows used by no registry source: {sorted(set(full) - used)}")
if check_only:
    print(f"sources/commit-pins.tsv: {len(full)} abbreviated registry commit(s) resolved to full SHAs; "
          f"{sum(len(c) for _, c in repos.values())} pinned commit(s) in {len(repos)} repositories, no branch named")
    sys.exit(0)
out = Path(sys.argv[1])
out.mkdir(parents=True, exist_ok=True)


GIT_ENV = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
GIT_ENV.update(GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull, GIT_NO_REPLACE_OBJECTS="1",
               GIT_TERMINAL_PROMPT="0", GIT_CEILING_DIRECTORIES=str(out.resolve()))


def git(*args, cwd, timeout=600, lazy=False):
    # No system or global config, no replace objects, no prompt, no discovery
    # above CHECKOUTS_DIR; the repository's own config is the one written
    # below. GIT_NO_LAZY_FETCH: a presence check or fsck never fetches;
    # fetching is explicit, by SHA.
    env = dict(GIT_ENV)
    if args[0] in ("cat-file", "fsck") and not lazy:
        env["GIT_NO_LAZY_FETCH"] = "1"
    safe = ["-c", "core.fsmonitor=false", "-c", "core.hooksPath=/dev/null"]
    try:
        return subprocess.run(["git", *safe, *args], cwd=cwd, capture_output=True, text=True, env=env,
                              timeout=timeout, stdin=subprocess.DEVNULL)
    except subprocess.TimeoutExpired:
        return subprocess.CompletedProcess(args, 124, "", f"no answer within {timeout} s")


LOOSE = re.compile(r"[0-9a-f]{2}/[0-9a-f]{38}")
PACK = re.compile(r"pack/pack-[0-9a-f]{40}\.(pack|idx|rev|promisor)")


def remove(p):
    """Remove p whatever it is, without following a symlink."""
    try:
        st = os.lstat(p)
    except FileNotFoundError:
        return
    if stat.S_ISDIR(st.st_mode):
        shutil.rmtree(p)
    else:
        os.unlink(p)


def salvage_objects(d, stage):
    """Final audit: copy, never move, the plain object files of a cached
    checkout into `stage`: regular files at loose-object or pack paths only,
    each opened without following a symlink and without blocking on a FIFO.
    Every other entry (symlinks, FIFOs, devices, objects/info, unknown names)
    is left behind and deleted with the checkout. Returns the number copied."""
    objs = d / ".git" / "objects"
    for p in (d, d / ".git", objs):
        try:
            if not stat.S_ISDIR(os.lstat(p).st_mode):
                return 0
        except OSError:
            return 0
    n = 0
    for dirpath, dirnames, filenames in os.walk(objs, followlinks=False):
        for fn in filenames:
            full = os.path.join(dirpath, fn)
            rel = os.path.relpath(full, objs)
            if not (LOOSE.fullmatch(rel) or PACK.fullmatch(rel)):
                continue
            try:
                fd = os.open(full, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
            except OSError:
                continue
            try:
                if not stat.S_ISREG(os.fstat(fd).st_mode):
                    continue
                dst = stage / rel
                dst.parent.mkdir(parents=True, exist_ok=True)
                with os.fdopen(os.dup(fd), "rb") as src, open(dst, "xb") as w:
                    shutil.copyfileobj(src, w)
                n += 1
            finally:
                os.close(fd)
    return n


bad = 0
for name, (slug, commits) in sorted(repos.items()):
    d = out / name
    # A cached checkout is untrusted (audits N1, final): only its plain object
    # files are kept (copied); the checkout itself, whatever it is, is
    # deleted, and a fresh repository is made with this script's own config.
    stage = Path(tempfile.mkdtemp(prefix=f".{name}-objects-", dir=out))
    kept = salvage_objects(d, stage)
    remove(d)
    d.mkdir()
    for args in (("init", "-q"), ("remote", "add", "origin", f"https://github.com/{slug}"),
                 ("config", "remote.origin.promisor", "true"),
                 ("config", "remote.origin.partialclonefilter", "blob:none")):
        r = git(*args, cwd=d)
        if r.returncode != 0:
            sys.exit(f"{name}: git {' '.join(args)} failed: {r.stderr.strip()}")
    if not (d / ".git" / "objects").is_dir() or (d / ".git").is_symlink():
        sys.exit(f"{name}: git init did not make {d}/.git")
    for dirpath, _, filenames in os.walk(stage):
        for fn in filenames:
            rel = os.path.relpath(os.path.join(dirpath, fn), stage)
            (d / ".git" / "objects" / rel).parent.mkdir(parents=True, exist_ok=True)
            os.replace(os.path.join(dirpath, fn), d / ".git" / "objects" / rel)
    shutil.rmtree(stage)
    if kept:
        print(f"reuse    {name}: {kept} cached object file(s) copied into a fresh repository")
    for c in sorted(commits):
        if git("cat-file", "-e", f"{c}^{{commit}}", cwd=d).returncode == 0:
            print(f"present  {name} {c[:12]}")
            continue
        r = git("fetch", "-q", "--depth", "1", "--filter=blob:none", "origin", c, cwd=d)
        ok = r.returncode == 0 and git("cat-file", "-e", f"{c}^{{commit}}", cwd=d).returncode == 0
        print(f"{'fetched ' if ok else 'MISSING '} {name} {c[:12]}{'' if ok else ': ' + r.stderr.strip()[:200]}")
        bad += 0 if ok else 1
    # The readers (scripts/pinned_read.py) never fetch: every file they will
    # read is materialized here, in this fresh repository with its own config,
    # by one lazy fetch from the pinned commit's promisor remote.
    for c, path in sorted(needed.get(name, ())):
        if git("cat-file", "-e", f"{c}:{path}", cwd=d).returncode == 0:
            continue
        r = git("cat-file", "blob", f"{c}:{path}", cwd=d, lazy=True)
        if r.returncode != 0 or git("cat-file", "-e", f"{c}:{path}", cwd=d).returncode != 0:
            print(f"MISSING  {name} {c[:12]}:{path}: {r.stderr.strip()[:200]}")
            bad += 1

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
# Final audit: `archived` must be a plain directory; a file is accepted only as
# a regular file equal to its SHA-256 pin, otherwise it is removed and
# downloaded again; downloads are written to a fresh temporary file
# (mkstemp, never a predictable name an attacker could pre-plant) and renamed.
if arch.is_symlink() or (arch.exists() and not arch.is_dir()):
    remove(arch)
arch.mkdir(exist_ok=True)
for s in reg["sources"]:
    if s.get("status") == "archived":
        url = s["url"].partition("#")[0]
    elif s.get("status") == "transcribed":
        url = s["location"]
    else:
        continue
    f = arch / url.rstrip("/").rsplit("/", 1)[-1]
    try:
        st = os.lstat(f)
        if stat.S_ISREG(st.st_mode) and hashlib.sha256(f.read_bytes()).hexdigest() == s["commit"]:
            print(f"present  archived/{f.name} (equal to its SHA-256 pin)")
            continue
        print(f"discard  archived/{f.name}: not a regular file equal to its pin")
        remove(f)
    except FileNotFoundError:
        pass
    try:
        req = urllib.request.Request(url, headers={"User-Agent": "rlt-fetch-sources (scripts/fetch-sources.py)"})
        with urllib.request.urlopen(req, timeout=60) as r:
            data = r.read(50 * 1024 * 1024 + 1)
        if len(data) > 50 * 1024 * 1024:
            raise ValueError("larger than 50 MB")
        got = hashlib.sha256(data).hexdigest()
        if got != s["commit"]:
            raise ValueError(f"the download has SHA-256 {got}, not the pin {s['commit']}: the live file changed or the server returned something else; nothing written")
        fd, tmp = tempfile.mkstemp(dir=arch, prefix=f".{f.name}.")
        with os.fdopen(fd, "wb") as w:
            w.write(data)
        os.replace(tmp, f)
        print(f"fetched  archived/{f.name} from {url} ({len(data)} bytes, equal to its SHA-256 pin)")
    except Exception as e:  # noqa: BLE001
        print(f"MISSING  archived/{f.name}: {e}")
        bad += 1
print(f"{len(repos)} repositories, {sum(len(c) for _, c in repos.values())} pinned commits; {bad} missing or corrupt")
sys.exit(0 if bad == 0 else 1)

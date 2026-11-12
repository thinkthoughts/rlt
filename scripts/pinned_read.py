#!/usr/bin/env python3
"""RLT-007a (final audit): read one file at one pinned commit from a source
checkout that may be untrusted (for example restored from a CI cache), using
none of the checkout's own Git metadata.

  * Only the checkout's object store is used, through a fresh temporary bare
    repository whose `objects/info/alternates` points at it. The checkout's
    config, hooks, refs (replace refs included), grafts, shallow file,
    attributes and worktree metadata are never read, so nothing in them can
    run a program or redirect a read.
  * Git runs with no system or global config, no lazy fetch from a promisor
    remote, no replace objects, no terminal prompt, a time limit, and an
    environment reduced to PATH.
  * Every object on the path commit -> tree -> ... -> blob is re-hashed here
    (SHA-1 of "<type> <size>\\0<content>") and must equal the name it was
    read under, so altered object bytes are refused whatever the store holds.
  * The commit must be a full 40-hex SHA; callers resolve abbreviated
    registry pins through sources/commit-pins.tsv.

Usage: pinned_read.py CHECKOUT COMMIT PATH   (writes the file's bytes to stdout)
Exit 0 on success; 1 with a reason on stderr otherwise.
"""
import hashlib
import os
import re
import stat
import subprocess
import sys
import tempfile

FULL = re.compile(r"[0-9a-f]{40}")


class PinnedReadError(Exception):
    pass


def _real_dir(p):
    try:
        st = os.lstat(p)
    except OSError as e:
        raise PinnedReadError(f"{p}: {e.strerror}")
    if not stat.S_ISDIR(st.st_mode):
        raise PinnedReadError(f"{p}: not a plain directory (a symlink, file or special file)")


def _git_env(home):
    return {
        "PATH": os.environ.get("PATH", "/usr/bin:/bin"),
        "HOME": home,
        "GIT_CONFIG_NOSYSTEM": "1",
        "GIT_CONFIG_GLOBAL": os.devnull,
        "GIT_NO_LAZY_FETCH": "1",
        "GIT_NO_REPLACE_OBJECTS": "1",
        "GIT_TERMINAL_PROMPT": "0",
        "GIT_CEILING_DIRECTORIES": os.path.dirname(home),
        "LC_ALL": "C",
    }


def read_pinned(checkout, commit, path):
    if not FULL.fullmatch(commit):
        raise PinnedReadError(f"commit {commit!r} is not a full 40-hex SHA")
    checkout = os.path.abspath(checkout)
    for p in (checkout, os.path.join(checkout, ".git"), os.path.join(checkout, ".git", "objects")):
        _real_dir(p)
    objects = os.path.realpath(os.path.join(checkout, ".git", "objects"))
    with tempfile.TemporaryDirectory(prefix="rlt-pinned-") as tmp:
        repo = os.path.join(tmp, "r.git")
        env = _git_env(tmp)

        def git(*args, binary=False):
            try:
                r = subprocess.run(["git", "--git-dir", repo, *args], env=env, cwd=tmp,
                                   capture_output=True, timeout=120, stdin=subprocess.DEVNULL)
            except subprocess.TimeoutExpired:
                raise PinnedReadError(f"git {' '.join(args)}: no answer within 120 s")
            if r.returncode != 0:
                raise PinnedReadError(f"git {' '.join(args)}: {r.stderr.decode('utf-8', 'replace').strip()[:300]}")
            return r.stdout

        r = subprocess.run(["git", "init", "-q", "--bare", repo], env=env, cwd=tmp,
                           capture_output=True, timeout=60, stdin=subprocess.DEVNULL)
        if r.returncode != 0:
            raise PinnedReadError(f"git init: {r.stderr.decode('utf-8', 'replace').strip()}")
        with open(os.path.join(repo, "objects", "info", "alternates"), "w") as f:
            f.write(objects + "\n")

        def obj(kind, sha):
            data = git("cat-file", kind, sha)
            got = hashlib.sha1(f"{kind} {len(data)}\0".encode() + data).hexdigest()
            if got != sha:
                raise PinnedReadError(f"{kind} {sha}: its bytes hash to {got}; the object store is altered")
            return data

        c = obj("commit", commit)
        m = re.match(rb"tree ([0-9a-f]{40})\n", c)
        if not m:
            raise PinnedReadError(f"commit {commit}: no tree line")
        sha, kind = m.group(1).decode(), "tree"
        parts = [x for x in path.split("/") if x]
        for i, name in enumerate(parts):
            if kind != "tree":
                raise PinnedReadError(f"{'/'.join(parts[:i])}: not a directory at {commit[:12]}")
            t, pos, found = obj("tree", sha), 0, None
            while pos < len(t):
                sp, nul = t.index(b" ", pos), t.index(b"\0", pos)
                mode, ename = t[pos:sp], t[sp + 1:nul]
                esha = t[nul + 1:nul + 21].hex()
                pos = nul + 21
                if ename.decode("utf-8", "surrogateescape") == name:
                    found = (mode, esha)
                    break
            if not found:
                raise PinnedReadError(f"{path}: `{name}` not found at {commit[:12]}")
            mode, sha = found
            kind = "tree" if mode == b"40000" else ("blob" if mode in (b"100644", b"100755") else "other")
        if kind != "blob":
            raise PinnedReadError(f"{path}: not a regular file at {commit[:12]}")
        return obj("blob", sha)


if __name__ == "__main__":
    if len(sys.argv) != 4:
        sys.exit(__doc__)
    try:
        sys.stdout.buffer.write(read_pinned(*sys.argv[1:]))
    except PinnedReadError as e:
        print(f"pinned_read: {e}", file=sys.stderr)
        sys.exit(1)

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
  * Each object's stored type must be the expected one (no following a tag
    or commit to another object); its size must be at most 50 MB, and the
    bytes actually read are capped at the reported size.
  * The path must have no empty, `.` or `..` segment.
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
LIMIT = 50 * 1024 * 1024  # bytes per object


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

        def capped(*args, limit):
            """Run git, reading at most `limit` bytes of its output; more is a
            refusal and git is stopped (final audit: a header can understate
            the size, so the cap is enforced on the bytes themselves)."""
            pr = subprocess.Popen(["git", "--git-dir", repo, *args], env=env, cwd=tmp,
                                  stdout=subprocess.PIPE, stderr=subprocess.PIPE, stdin=subprocess.DEVNULL)
            chunks, n = [], 0
            try:
                while True:
                    b = pr.stdout.read(1 << 20)
                    if not b:
                        break
                    n += len(b)
                    if n > limit:
                        pr.kill()
                        raise PinnedReadError(f"git {' '.join(args)}: more than {limit} bytes; refused")
                    chunks.append(b)
                err = pr.stderr.read()
                code = pr.wait(timeout=120)
            except subprocess.TimeoutExpired:
                pr.kill()
                raise PinnedReadError(f"git {' '.join(args)}: no answer within 120 s")
            finally:
                if pr.poll() is None:
                    pr.kill()
                    pr.wait()
            if code != 0:
                raise PinnedReadError(f"git {' '.join(args)}: {err.decode('utf-8', 'replace').strip()[:300]}")
            return b"".join(chunks)

        def obj(kind, sha):
            # The stored type must be the expected one, so `cat-file <type>`
            # never follows a tag or commit to another object; the size
            # reported must be within the limit; and the bytes actually read
            # are capped at that size, whatever the header claims. Memory git
            # itself uses to rebuild a deltified object is outside this
            # script's control (bounded in CI by the job's time limit).
            t = git("cat-file", "-t", sha).strip()
            if t != kind.encode():
                raise PinnedReadError(f"{sha}: stored as {t.decode('ascii', 'replace')!r}, expected {kind}")
            size = git("cat-file", "-s", sha).strip()
            if not size.isdigit() or int(size) > LIMIT:
                raise PinnedReadError(f"{kind} {sha}: size {size.decode('ascii', 'replace')!r} refused (limit {LIMIT} bytes)")
            data = capped("cat-file", kind, sha, limit=int(size))
            if len(data) != int(size):
                raise PinnedReadError(f"{kind} {sha}: {len(data)} bytes read, header says {int(size)}")
            got = hashlib.sha1(f"{kind} {len(data)}\0".encode() + data).hexdigest()
            if got != sha:
                raise PinnedReadError(f"{kind} {sha}: its bytes hash to {got}; the object store is altered")
            return data

        c = obj("commit", commit)
        m = re.match(rb"tree ([0-9a-f]{40})\n", c)
        if not m:
            raise PinnedReadError(f"commit {commit}: no tree line")
        sha, kind = m.group(1).decode(), "tree"
        parts = path.split("/")
        if any(x in ("", ".", "..") for x in parts):
            raise PinnedReadError(f"{path!r}: an empty, `.` or `..` path segment")
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

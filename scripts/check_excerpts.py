#!/usr/bin/env python3
"""Check that every registry excerpt appears verbatim in its source file.

usage: python3 -I scripts/check_excerpts.py CHECKOUTS_DIR

CHECKOUTS_DIR holds clones named after the GitHub repositories, including
rlt itself: news, rlt, translatepi, antiviolence, residue-manifold-learning,
specification-grammar, and (RLT-003 M3b) OpenLogic and lean4. Each excerpt is read from the recorded commit by
scripts/pinned_read.py (object store only, every object re-hashed; RLT-007a),
so the clones may be at any revision as long as they hold the recorded objects.
Sources without a GitHub file location (conversation records, foundational
references, missing sources) are listed as skipped. An archived source
(RLT-005) is read from CHECKOUTS_DIR/archived/ or downloaded, and refused
unless its SHA-256 equals the registry pin.
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
# Options (RLT-005, for the tamper tests in scripts/verify.sh):
#   --registry PATH   read this registry instead of sources/registry.yaml
#   --only ID         check only the source with this id
args = sys.argv[1:]
opts = {}
while args and args[0].startswith("--"):
    opts[args[0]] = args[1]
    args = args[2:]
checkouts = Path(args[0])
reg = yaml.safe_load(Path(opts.get("--registry", root / "sources/registry.yaml")).read_text(encoding="utf-8"))
if "--only" in opts:
    reg["sources"] = [s for s in reg["sources"] if s["id"] == opts["--only"]]
    if not reg["sources"]:
        sys.exit(f"no source {opts['--only']}")

pat = re.compile(r"https://github\.com/[^/]+/([^/]+)/blob/([0-9a-f]{7,40})/(.+)$")

# RLT-007a (final audit): every file is read through scripts/pinned_read.py,
# which uses only the checkout's object store, re-hashes every object on the
# path and never reads the checkout's own Git metadata; abbreviated registry
# pins are resolved through sources/commit-pins.tsv (a full SHA only).
import importlib.util  # noqa: E402

sys.dont_write_bytecode = True  # no scripts/__pycache__ in the working tree
_spec = importlib.util.spec_from_file_location("pinned_read", root / "scripts/pinned_read.py")
pinned_read = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(pinned_read)
full_pins = {}
for _l in (root / "sources/commit-pins.tsv").read_text(encoding="utf-8").splitlines():
    if _l and not _l.startswith("#"):
        _r, _short, _long = _l.split("\t")
        full_pins[(_r, _short)] = _long
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


MAX_ARCHIVE = 50 * 1024 * 1024  # bytes read from the network, at most


def member_ok(m):
    """The member rule shared with `rlt validate`: a relative path of
    non-empty components, none of them '.' or '..'."""
    return bool(m) and not m.startswith("/") and all(c not in ("", ".", "..") for c in m.split("/"))


def gzip_fname(data):
    """The FNAME field of a gzip header (RFC 1952), or None."""
    if data[:2] != b"\x1f\x8b" or len(data) < 10 or not data[3] & 0x08:
        return None
    i = 10
    if data[3] & 0x04:  # FEXTRA
        i += 2 + int.from_bytes(data[10:12], "little")
    j = data.index(b"\0", i)
    return data[i:j].decode("latin-1")


def fetch_pinned(url, sha256):
    """The bytes at `url`, read from CHECKOUTS/archived/ if present there
    (never through a symlink), else downloaded (at most MAX_ARCHIVE bytes);
    refused unless their SHA-256 equals `sha256`. Downloaded bytes are cached
    only after that check, atomically."""
    import hashlib, os, tempfile, urllib.request
    s = {"commit": sha256}
    cached = checkouts / "archived" / url.rstrip("/").rsplit("/", 1)[-1]
    if cached.is_symlink():
        raise ValueError(f"{cached} is a symlink; refusing to read or write through it")
    fetched = not cached.exists()
    if fetched:
        req = urllib.request.Request(url, headers={"User-Agent": "rlt-check-excerpts (scripts/check_excerpts.py)"})
        with urllib.request.urlopen(req, timeout=60) as r:
            data = r.read(MAX_ARCHIVE + 1)
        if len(data) > MAX_ARCHIVE:
            raise ValueError(f"download from {url} exceeds {MAX_ARCHIVE} bytes")
    else:
        data = cached.read_bytes()
    got = hashlib.sha256(data).hexdigest()
    if got != s["commit"]:
        raise ValueError(f"archive SHA-256 {got} differs from the pin {s['commit']}")
    if fetched:  # cache only bytes that match the pin, atomically
        cached.parent.mkdir(parents=True, exist_ok=True)
        fd, tmp = tempfile.mkstemp(dir=cached.parent)
        with os.fdopen(fd, "wb") as fh:
            fh.write(data)
        os.replace(tmp, cached)
    return data


def archived_text(s):
    """RLT-005: an archived source is a versioned archive file (e.g. an arXiv
    e-print) pinned by the SHA-256 in `commit`; `url` names the archive and,
    after '#', the member file. The archive is read from CHECKOUTS/archived/
    if present there (never through a symlink), else downloaded (at most
    MAX_ARCHIVE bytes); its SHA-256 must equal the pin. A tar archive must
    contain the member; a single gzipped file must name it in its header."""
    import gzip, io, tarfile
    url, _, member = s["url"].partition("#")
    if not member_ok(member):
        raise ValueError(f"member {member!r} must be a relative path without '.' or '..' components")
    data = fetch_pinned(url, s["commit"])
    try:
        t = tarfile.open(fileobj=io.BytesIO(data))
    except tarfile.ReadError:
        name = gzip_fname(data)
        if name != member:
            raise ValueError(f"single gzipped file names {name!r} in its header, not the member {member!r}")
        return gzip.decompress(data).decode("utf-8")
    with t:
        return t.extractfile(member).read().decode("utf-8")


def latex_live(text):
    """LaTeX source with comments removed: from an unescaped % to the line
    end (the line break is kept, so excerpts cannot join across comments)."""
    import re as _re
    return _re.sub(r"(?<!\\)%[^\n]*", "", text)


def at_word_boundaries(hay, x):
    """Some whitespace-collapsed occurrence of x in hay starts and ends at a
    word boundary (no letter or digit continues the word on either side)."""
    h, y = " ".join(hay.split()), " ".join(x.split())
    start = 0
    while (i := h.find(y, start)) >= 0:
        before = h[i - 1] if i else " "
        after = h[i + len(y)] if i + len(y) < len(h) else " "
        if not (before.isalnum() and y[0].isalnum()) and not (after.isalnum() and y[-1].isalnum()):
            return True
        start = i + 1
    return False

def transcribed(s):
    """RLT-006: a source read through a reviewed transcription. `url` is the
    repository path of the transcription file; `commit` is the SHA-256 of the
    PDF. Yields (status, message) pairs; any status but 'ok' is a problem."""
    import hashlib, re as _re, subprocess, tempfile
    rel = s["url"]
    if not rel.startswith("sources/transcriptions/") or ".." in rel.split("/") or not rel.endswith(".yaml"):
        yield "ERROR", f"transcription path {rel!r} must be sources/transcriptions/*.yaml"
        return
    if (root / rel).is_symlink():
        yield "ERROR", f"{rel} is a symlink; a transcription must be a file in sources/transcriptions/"
        return
    t = yaml.safe_load((root / rel).read_text(encoding="utf-8"))
    ids = [it.get("id") for it in t.get("items", [])]
    if len(ids) != len(set(ids)) or not all(ids):
        yield "ERROR", f"{rel}: item ids must be present and distinct"
        return
    if t.get("format") != "rlt-transcription/1" or t.get("source") != s["id"]:
        yield "ERROR", f"{rel}: format must be rlt-transcription/1 and source {s['id']}"
        return
    if t["pdf"]["sha256"] != s["commit"]:
        yield "ERROR", f"{rel}: PDF SHA-256 {t['pdf']['sha256']} differs from the registry pin {s['commit']}"
        return
    if not t.get("reviewed_by") or not all(r.get("reviewer") and r.get("date") for r in t["reviewed_by"]):
        yield "ERROR", f"{rel}: reviewed_by must name each reviewer and date"
        return
    if t["pdf"]["url"] != s["location"]:
        yield "ERROR", f"{rel}: PDF url {t['pdf']['url']} differs from the registry location {s['location']}"
        return
    data = fetch_pinned(t["pdf"]["url"], s["commit"])
    if len(data) != t["pdf"]["bytes"]:
        yield "ERROR", f"{rel}: the PDF has {len(data)} bytes; the transcription records {t['pdf']['bytes']}"
        return
    try:
        ver = subprocess.run(["pdftotext", "-v"], capture_output=True, text=True).stderr
    except FileNotFoundError:
        yield "ERROR", "pdftotext is not installed (poppler-utils); the transcription cannot be checked"
        return
    want = f"pdftotext version {t['tool']['version']}"
    if (ver.splitlines() or [""])[0].strip() != want:
        yield "ERROR", f"pdftotext reports {ver.splitlines()[0] if ver else '?'!r}; the transcription pins {want!r}"
        return
    pages = {}
    with tempfile.NamedTemporaryFile(suffix=".pdf") as fh:
        fh.write(data)
        fh.flush()
        for pg in t["pages"]:
            out = subprocess.run(["pdftotext", "-layout", "-f", str(pg["page"]), "-l", str(pg["page"]), fh.name, "-"],
                                 capture_output=True).stdout
            got = hashlib.sha256(out).hexdigest()
            if got != pg["extraction_sha256"]:
                yield "ERROR", f"page {pg['page']}: extraction SHA-256 {got} differs from the pinned {pg['extraction_sha256']}"
                return
            pages[pg["page"]] = out.decode("utf-8")
    texts = set()
    for it in t["items"]:
        texts.add(it["text"])
        if it["page"] not in pages:
            yield "ERROR", f"{it['id']}: page {it['page']} has no pinned extraction"
            continue
        if not at_word_boundaries(pages[it["page"]], it["extracted"]):
            yield "MISSING", f"{it['id']}: extracted text not on page {it['page']}: {it['extracted'][:60]!r}"
            continue
        if "numbers" in it:
            run = _re.search(r"\d+(?:, \d+)+", " ".join(it["extracted"].split()))
            nums = [int(x) for x in run.group(0).split(", ")] if run else []
            if nums != it["numbers"]:
                yield "MISSING", f"{it['id']}: numbers {it['numbers']} differ from the extracted list {nums}"
                continue
        yield "ok", f"{it['id']} (page {it['page']}, {it['location']})"
    for e in s.get("excerpts", []):
        x = e["text"] if isinstance(e, dict) else e
        yield ("ok", f"excerpt {x[:60]!r}") if x in texts else ("MISSING", f"excerpt is not a reviewed item text: {x[:60]!r}")


bad = 0
for s in reg["sources"]:
    if s["status"] == "transcribed":
        try:
            results = list(transcribed(s))
        except Exception as e:
            results = [("ERROR", str(e))]
        for st, msg in results:
            print(f"{st:8} {s['id']}: {msg}")
            bad += 0 if st == "ok" else 1
        continue
    if s["status"] == "archived":
        try:
            text = archived_text(s)
        except Exception as e:
            print(f"ERROR    {s['id']}: {e}")
            bad += 1
            continue
        # Comments are not the source's text; an excerpt must sit in live
        # LaTeX and must not start or end inside a word.
        if s["url"].partition("#")[2].endswith(".tex"):
            text = latex_live(text)
        for e in s.get("excerpts", []):
            x = e["text"] if isinstance(e, dict) else e
            ok = at_word_boundaries(text, x)
            print(f"{'ok' if ok else 'MISSING':8} {s['id']}: {x[:70]!r}")
            bad += 0 if ok else 1
        continue
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
    if len(commit) != 40:
        if (repo, commit) not in full_pins:
            print(f"ERROR    {s['id']}: abbreviated commit {commit} has no row in sources/commit-pins.tsv")
            bad += 1
            continue
        commit = full_pins[(repo, commit)]
    try:
        text = pinned_read.read_pinned(str(checkouts / repo), commit, path).decode("utf-8")
    except (pinned_read.PinnedReadError, UnicodeDecodeError) as e:
        print(f"ERROR    {s['id']}: {e}")
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

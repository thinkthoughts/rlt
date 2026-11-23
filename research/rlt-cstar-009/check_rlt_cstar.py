"""RLT-CSTAR certificate gate (RLT-CSTAR-009). Outside RLT's trusted base.

Reads research/rlt-cstar-002/certificates/MANIFEST.tsv and
research/rlt-cstar-007/certificates/MANIFEST.tsv (tab-separated: file, sha256, exit,
verdict; a header row starting with "file" is optional). Fails if:
  - a manifest lists a file twice, or a file in two manifests;
  - a manifest lists a file that does not exist, or a certificate file (*.rlt) in
    those directories is not listed (omitted);
  - a certificate's SHA-256 differs from its manifest entry (modified);
  - the unchanged kernel's verdict (first word) differs from the manifest's
    (incorrectly classified);
  - a copy of an ACCEPT certificate with the last digit of its goal changed is not
    REJECTed, or the untampered control is not ACCEPTed;
  - the verdict totals differ from --expect (e.g. --expect ACCEPT=17 MALFORMED=1).
Usage, from the repository root:
    python3 research/rlt-cstar-009/check_rlt_cstar.py target/release/rlt --expect ACCEPT=17 MALFORMED=1
"""
import argparse, collections, hashlib, re, subprocess, sys, tempfile
from pathlib import Path

ap = argparse.ArgumentParser()
ap.add_argument("rlt")
ap.add_argument("--expect", nargs="*", default=[], help="VERDICT=COUNT totals the manifests must give")
args = ap.parse_args()
repo = Path.cwd()
DIRS = [repo / "research/rlt-cstar-002/certificates", repo / "research/rlt-cstar-007/certificates"]

def verdict(path: Path) -> str:
    out = subprocess.run([args.rlt, "check", str(path)], capture_output=True, text=True, cwd=repo).stdout
    return next((l.split()[1] for l in out.splitlines() if l.startswith("verdict")), "?")

problems = []
entries = []                                    # (directory, name, sha, verdict)
seen = collections.Counter()
for d in DIRS:
    manifest = d / "MANIFEST.tsv"
    if not manifest.is_file():
        problems.append(f"missing manifest {manifest.relative_to(repo)}"); continue
    for n, line in enumerate(manifest.read_text(encoding="utf-8").splitlines(), 1):
        if not line.strip() or (n == 1 and line.startswith("file\t")):
            continue
        cols = line.split("\t")
        if len(cols) != 4 or not re.fullmatch(r"[0-9a-f]{64}", cols[1]):
            problems.append(f"{manifest.relative_to(repo)}:{n}: malformed entry"); continue
        name, sha, _exit, v = cols
        seen[name] += 1
        entries.append((d, name, sha, v.split()[0]))
    listed = {name for dd, name, *_ in entries if dd == d}
    for f in sorted(d.glob("*.rlt")):
        if f.name not in listed:
            problems.append(f"omitted: {f.relative_to(repo)} has no manifest entry")
for name, k in seen.items():
    if k > 1:
        problems.append(f"duplicated: {name} listed {k} times")

totals = collections.Counter(); tampered = 0
for d, name, sha, want in entries:
    cert = d / name
    if not cert.is_file():
        problems.append(f"missing file: {cert.relative_to(repo)} is listed but absent"); continue
    got_sha = hashlib.sha256(cert.read_bytes()).hexdigest(); got = verdict(cert)
    totals[got] += 1
    status = []
    if got_sha != sha: status.append("modified (SHA-256 differs)")
    if got != want: status.append(f"incorrectly classified (kernel {got}, manifest {want})")
    problems += [f"{cert.relative_to(repo)}: {s}" for s in status]
    print(f"{'ok  ' if not status else 'FAIL'} {cert.relative_to(repo)}  {got}")
    if got == "ACCEPT":
        s = cert.read_text(encoding="utf-8"); m = re.search(r'\(goal "([^"]*)"\)', s); g = m.group(1)
        i = max(k for k, ch in enumerate(g) if ch.isdigit())
        with tempfile.TemporaryDirectory() as tmp:
            bad = Path(tmp) / name
            bad.write_text(s[:m.start(1)] + g[:i] + str((int(g[i]) + 1) % 10) + g[i + 1:] + s[m.end(1):], encoding="utf-8")
            t = verdict(bad)
        tampered += 1
        if t != "REJECT":
            problems.append(f"{cert.relative_to(repo)}: tampered copy gave {t}, not REJECT")

for spec in args.expect:
    v, k = spec.split("="); k = int(k)
    if totals[v] != k:
        problems.append(f"expected {k} {v}, kernel gave {totals[v]}")
print(f"{sum(totals.values())} certificates; verdicts {dict(sorted(totals.items()))}; {tampered} tampered copies REJECT-checked")
for p in problems:
    print("FAIL", p)
print(f"failures: {len(problems)}")
sys.exit(1 if problems else 0)

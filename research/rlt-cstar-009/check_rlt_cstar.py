"""RLT-CSTAR certificate gate (RLT-CSTAR-009). Outside RLT's trusted base.

For every certificate listed in research/rlt-cstar-00{2,7}/certificates/MANIFEST.tsv:
its SHA-256 must equal the manifest's, and the unchanged kernel's verdict (first
word: ACCEPT, MALFORMED, ...) must equal the manifest's. Then every ACCEPT
certificate is checked again with the last digit of its goal changed: the kernel
must REJECT each, while the untampered copy (the control) still ACCEPTs.
Exit status 0 only if all hold. Usage, from the repository root:
    python3 research/rlt-cstar-009/check_rlt_cstar.py target/release/rlt
"""
import hashlib, re, subprocess, sys, tempfile
from pathlib import Path

rlt = sys.argv[1]
repo = Path.cwd()

def verdict(path: Path) -> str:
    out = subprocess.run([rlt, "check", str(path)], capture_output=True, text=True, cwd=repo).stdout
    return next((l.split()[1] for l in out.splitlines() if l.startswith("verdict")), "?")

failures = checked = tampered = 0
for manifest in sorted(repo.glob("research/rlt-cstar-00[27]/certificates/MANIFEST.tsv")):
    rows = [l.split("\t") for l in manifest.read_text(encoding="utf-8").splitlines() if l.strip() and not l.startswith("file\t")]
    for name, sha, _exit, expected in rows:
        cert = manifest.parent / name
        got_sha = hashlib.sha256(cert.read_bytes()).hexdigest()
        got = verdict(cert); want = expected.split()[0]
        ok = got_sha == sha and got == want
        failures += not ok; checked += 1
        print(f"{'ok  ' if ok else 'FAIL'} {cert.relative_to(repo)}  sha {'=' if got_sha == sha else '!='}  verdict {got} (manifest {want})")
        if want != "ACCEPT":
            continue
        s = cert.read_text(encoding="utf-8"); m = re.search(r'\(goal "([^"]*)"\)', s); g = m.group(1)
        i = max(k for k, ch in enumerate(g) if ch.isdigit())
        with tempfile.TemporaryDirectory() as tmp:
            bad = Path(tmp) / name
            bad.write_text(s[:m.start(1)] + g[:i] + str((int(g[i]) + 1) % 10) + g[i + 1:] + s[m.end(1):], encoding="utf-8")
            t = verdict(bad)
        tampered += 1; failures += t != "REJECT"
        print(f"{'ok  ' if t == 'REJECT' else 'FAIL'}   tampered goal -> {t}")
print(f"{checked} certificates checked against their manifests; {tampered} tampered copies; failures: {failures}")
sys.exit(1 if failures else 0)

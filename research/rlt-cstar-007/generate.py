"""RLT-CSTAR-007: build and check the certificates for the new tiling statements.

Outside RLT's trusted base: `rlt prove` (bounded search; the kernel replays it) on each
goal in goals/, then `rlt check` on every certificate, written to MANIFEST.tsv as file,
SHA-256, exit code, verdict. Only the kernel's verdict counts.

Usage, from the repository root:
    python3 -I research/rlt-cstar-007/generate.py target/release/rlt OUTDIR
"""
import hashlib, shutil, subprocess, sys
from pathlib import Path

rlt, outdir = sys.argv[1], Path(sys.argv[2])
here = Path(__file__).resolve().parent
outdir.mkdir(parents=True, exist_ok=True)
for goal in sorted((here / "goals").glob("G*.goal")):
    subprocess.run([rlt, "prove", str(goal)], check=True, capture_output=True)
    shutil.copy(Path("target/rlt-prove") / f"CSTAR007.{goal.stem}.rlt", outdir / f"{goal.stem}.rlt")
rows = []
for cert in sorted(outdir.glob("*.rlt")):
    r = subprocess.run([rlt, "check", str(cert)], capture_output=True, text=True)
    verdict = next((l.split(None, 1)[1] for l in r.stdout.splitlines() if l.startswith("verdict")), "")
    rows.append(f"{cert.name}\t{hashlib.sha256(cert.read_bytes()).hexdigest()}\t{r.returncode}\t{verdict}")
(outdir / "MANIFEST.tsv").write_text("\n".join(rows) + "\n", encoding="utf-8")
print("\n".join(rows))

"""RLT-CSTAR-002: build and check the certificates for the tiling statements.

Outside RLT's trusted base. It writes ordinary proof files from three sources:
  - `rlt prove` on each goal in goals/ (bounded search; the kernel replays it),
  - the M3c generator functions of scripts/gen-m3c-tests.py for `¬(d ∣ n)` and
    `d ∣ n` (read from this checkout, never modified),
then runs `rlt check` on every file and writes MANIFEST.tsv: file, SHA-256,
exit code, verdict. Only the kernel's verdict counts.

Usage, from the repository root:
    python3 -I research/rlt-cstar-002/generate.py target/release/rlt OUTDIR
"""

import hashlib
import shutil
import subprocess
import sys
from pathlib import Path

rlt, outdir = sys.argv[1], Path(sys.argv[2])
here = Path(__file__).resolve().parent
outdir.mkdir(parents=True, exist_ok=True)

# 1. bounded proof search for the closed equations
for goal in sorted((here / "goals").glob("G0[1-6]_*.goal")):
    subprocess.run([rlt, "prove", str(goal)], check=True, capture_output=True)
    pid = "CSTAR002." + goal.stem
    shutil.copy(Path("target/rlt-prove") / f"{pid}.rlt", outdir / f"{goal.stem}.rlt")

# 2. divisibility facts with the M3c generator (functions only, lines 1-206)
src = Path("scripts/gen-m3c-tests.py")
lines = src.read_text(encoding="utf-8").splitlines()
defs = "\n".join(lines[:206]).replace(
    "rlt, out = sys.argv[1], sys.argv[2]", f"rlt, out = {rlt!r}, ''"
)
ns: dict = {}
exec(compile(defs, str(src), "exec"), ns)


def retag(text: str, pid: str) -> str:
    return text.replace(
        "; RLT-003 M3c test", "; RLT-CSTAR-002 certificate (M3c generator)"
    ).replace(f"(id M3c.{pid})", f"(id CSTAR002.{pid})")


(outdir / "G07_ndiv_10_3.rlt").write_text(
    retag(ns["not_dvd"]("G07_ndiv_10_3", 3, 10), "G07_ndiv_10_3"), encoding="utf-8"
)

# Bundled C-Star DomainSpecs (cstar/catalog/bundled/specs/DomainSpec at 0991b445):
# grid interior points and tiles per axis; all divide evenly.
CATALOG = [
    ("ccs-12km xi", 16, 224), ("ccs-12km eta", 20, 440),
    ("hvalfjordur-0 xi", 16, 512), ("hvalfjordur-0 eta", 16, 384),
    ("NEP/PAC/GoA xi", 8, 192), ("NEP/PAC/GoA eta", 16, 208),
    ("wio-toy xi", 2, 20), ("wio-toy eta", 5, 20),
    ("gulf-guinea-toy xi", 2, 10), ("gulf-guinea-toy eta", 5, 10),
]
for label, d, n in CATALOG:
    pid = f"C_dvd_{d}_{n}"
    steps, last = ns["gen_steps"](f"{n} = {d} × {n // d}", "m")
    text = ns["proof"](pid, f"{d} ∣ {n}", steps + [f"(e1 (dvd {d} {n}) (exists-intro {n // d} {last}))"])
    (outdir / f"{pid}.rlt").write_text(retag(text, pid), encoding="utf-8")

# 3. the kernel decides every file
rows = ["file\tsha256\texit\tverdict"]
for f in sorted(outdir.glob("*.rlt")):
    r = subprocess.run([rlt, "check", str(f)], capture_output=True, text=True)
    verdict = next(
        (l.split(None, 1)[1] for l in r.stdout.splitlines() if l.startswith("verdict")), r.stdout.strip()[-200:]
    )
    sha = hashlib.sha256(f.read_bytes()).hexdigest()
    rows.append(f"{f.name}\t{sha}\t{r.returncode}\t{verdict}")
(outdir / "MANIFEST.tsv").write_text("\n".join(rows) + "\n", encoding="utf-8")
print("\n".join(rows))

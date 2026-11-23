"""Change the last digit of each accepted certificate's goal and check it again:
the kernel must reject every one. Usage: tamper_certificates.py RLT RLT_CHECKOUT"""
import re, subprocess, sys, tempfile
from pathlib import Path
rlt, repo = sys.argv[1], Path(sys.argv[2])
certs = sorted(repo.glob("research/rlt-cstar-00[27]/certificates/*.rlt"))
with tempfile.TemporaryDirectory() as tmp:
    for c in certs:
        if c.name == "C_dvd_16_512.rlt":   # MALFORMED by the term limit (F-015)
            continue
        s = c.read_text(); m = re.search(r'\(goal "([^"]*)"\)', s); g = m.group(1)
        i = max(k for k, ch in enumerate(g) if ch.isdigit())
        bad = Path(tmp) / c.name
        bad.write_text(s[:m.start(1)] + g[:i] + str((int(g[i]) + 1) % 10) + g[i + 1:] + s[m.end(1):])
        def verdict(path):
            out = subprocess.run([rlt, "check", str(path)], capture_output=True, text=True, cwd=repo).stdout  # axiom specs resolve from the checkout
            return next((l.split(None, 1)[1][:6] for l in out.splitlines() if l.startswith("verdict")), "?")
        print(f"{c.name}\tcontrol {verdict(c)}\ttampered {verdict(bad)}")

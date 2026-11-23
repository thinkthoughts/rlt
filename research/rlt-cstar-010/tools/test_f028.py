"""Exercise the 'Look for hash mismatches to report' step of containerized_ci.yml
(both jobs) with empty, all-zero, valid, unknown and malformed BASE_SHA values.

A bare repository stands in for GitHub (uploadpack.allowAnySHA1InWant, as GitHub
serves any reachable commit by SHA); the job's checkout is a depth-1 clone of it,
as actions/checkout makes. Usage: test_f028.py WORKFLOW.yml SCRATCH_DIR
"""
import os, subprocess, sys, pathlib, yaml, shutil
wf, scratch = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
if scratch.exists(): shutil.rmtree(scratch)
scratch.mkdir(parents=True)
def sh(cmd, cwd, env=None):
    return subprocess.run(cmd, cwd=cwd, shell=True, text=True, capture_output=True, env=env)
src = scratch / "src"; src.mkdir()
for c in ("git init -q -b main", "git -c user.name=t -c user.email=t@t commit -q --allow-empty -m base",
          "git -c user.name=t -c user.email=t@t commit -q --allow-empty -m head"):
    assert sh(c, src).returncode == 0, c
base = sh("git rev-parse HEAD~1", src).stdout.strip()
sh(f"git clone -q --bare {src} {scratch}/origin.git", scratch)
sh("git config uploadpack.allowAnySHA1InWant true", scratch / "origin.git")
job = scratch / "job"
assert sh(f"git clone -q --depth 1 file://{scratch}/origin.git {job}", scratch).returncode == 0
(job / "tests/results").mkdir(parents=True)
doc = yaml.safe_load(wf.read_text())
cases = {"empty": "", "all-zero": "0" * 40, "valid": base,
         "unknown": "1234567890abcdef1234567890abcdef12345678",
         "malformed": "main; touch PWNED", "short": base[:12]}
expect = {"empty": False, "all-zero": False, "valid": True, "unknown": False,
          "malformed": False, "short": False}
fails = 0
for jobname in ("test_gnu", "test_ifx"):
    step = next(s for s in doc["jobs"][jobname]["steps"] if s.get("name") == "Look for hash mismatches to report")
    env_name = "gnu" if jobname == "test_gnu" else "ifx"
    for with_mismatch in (True, False):
        mm = job / f"tests/results/mismatches_github_{env_name}.json"
        if with_mismatch: mm.write_text("{}")
        elif mm.exists(): mm.unlink()
        for name, val in cases.items():
            out = job / "gh_output"; out.write_text("")
            env = dict(os.environ, BASE_SHA=val, GITHUB_OUTPUT=str(out))
            r = subprocess.run(["bash", "-e", "-c", step["run"]], cwd=job, text=True, capture_output=True, env=env)
            found = "found=true" in out.read_text()
            want = expect[name] and with_mismatch
            ok = r.returncode == 0 and found == want and not (job / "PWNED").exists()
            fails += not ok
            print(f"{'PASS' if ok else 'FAIL'} {jobname} mismatches={with_mismatch!s:5} {name:9} rc={r.returncode} found={found!s:5} | {r.stdout.strip()}")
print(f"failures: {fails}")
sys.exit(1 if fails else 0)

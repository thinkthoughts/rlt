"""RLT-CSTAR-009 audit: in every workflow a push to BRANCH triggers, every job or step
able to write outside the fork must require a CWorthy-ocean repository, as a top-level
&& conjunct of its job or step condition.
Usage: audit_ci_writes.py REPO_DIR BRANCH BASE_FOR_PATHS"""
import fnmatch, re, subprocess, sys, yaml
repo, branch, base = sys.argv[1:4]
ref = sys.argv[4] if len(sys.argv) > 4 else branch   # commit to read; branch = name the triggers see
git = lambda *a: subprocess.check_output(["git", "-C", repo, *a], text=True)
changed = git("diff", "--name-only", f"{base}...{ref}", "--").split()
WRITE = [  # (label, regex over the job or step text)
    ("packages: write", r"packages:\s*write"), ("contents: write", r"contents:\s*write"),
    ("ghcr/docker login", r"docker/login-action|docker login"), ("image push", r"docker/build-push-action|docker push"),
    ("GitHub API write", r"gh api[^\n]*-X\s*(POST|PUT|PATCH|DELETE)|gh release (create|delete|upload)"),
    ("git push", r"git push"), ("PyPI publish", r"pypi-publish|twine upload"), ("Codecov upload", r"codecov/codecov-action"),
    ("HTTP write", r"curl[^\n]*-X\s*(POST|PUT|PATCH|DELETE)|requests\.(post|put|patch|delete)"),
]
GUARD = re.compile(r"github\.repository(_owner)?\s*==\s*'CWorthy-ocean(/[A-Za-z0-9_.-]+)?'")
def top_level_guarded(cond):
    if not cond: return False
    c = str(cond).strip()
    c = c[3:-2].strip() if c.startswith("${{") else c
    if "||" in c: return False                   # an alternative could bypass the guard
    return any(GUARD.fullmatch(p.strip().strip("()").strip()) for p in c.split("&&"))
def triggered(wf):
    on = wf.get(True, wf.get("on"))
    if isinstance(on, str): on = {on: None}
    if isinstance(on, list): on = {k: None for k in on}
    if "push" not in (on or {}): return False, "no push trigger"
    push = on["push"] or {}
    br = push.get("branches"); bri = push.get("branches-ignore")
    if br is not None and not any(fnmatch.fnmatch(branch, b) for b in br): return False, f"branches {br}"
    if bri is not None and any(fnmatch.fnmatch(branch, b) for b in bri): return False, "branches-ignore"
    paths = push.get("paths")
    if paths is not None and not any(fnmatch.fnmatch(f, p) for f in changed for p in paths): return False, f"paths {paths} untouched"
    return True, "push"
bad = 0
for path in git("ls-tree", "--name-only", f"{ref}", ".github/workflows/").split():
    if not path.endswith((".yml", ".yaml")): continue
    text = git("show", f"{ref}:{path}"); wf = yaml.safe_load(text)
    on, why = triggered(wf)
    if not on: print(f"  not triggered  {path}: {why}"); continue
    print(f"TRIGGERED       {path}")
    top_perm = yaml.safe_dump(wf.get("permissions", {}))
    for name, job in (wf.get("jobs") or {}).items():
        jtext = yaml.safe_dump(job); jguard = top_level_guarded(job.get("if"))
        jperm = yaml.safe_dump(job.get("permissions", {})) if "permissions" in job else top_perm
        for label, rx in WRITE[:2]:
            if re.search(rx, jperm):
                ok = jguard; bad += not ok
                print(f"    {'ok  ' if ok else 'FAIL'} job {name}: {label} {'(job guarded)' if ok else 'UNGUARDED'}")
        for i, step in enumerate(job.get("steps") or []):
            stext = yaml.safe_dump(step)
            for label, rx in WRITE[2:]:
                if re.search(rx, stext):
                    ok = jguard or top_level_guarded(step.get("if")); bad += not ok
                    print(f"    {'ok  ' if ok else 'FAIL'} job {name} step {step.get('name', step.get('uses', i))!r}: {label} "
                          f"{'(job guard)' if jguard else '(step guard)' if ok else 'UNGUARDED'}")
        if "uses" in job and str(job["uses"]).startswith("./"):
            print(f"    NOTE job {name} calls reusable workflow {job['uses']}")
print("unguarded write paths:", bad); sys.exit(1 if bad else 0)

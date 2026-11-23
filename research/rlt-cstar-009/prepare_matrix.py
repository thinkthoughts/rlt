"""Execute the plan step of containerized_ci.yml at REF under an event matrix.
Usage: prepare_matrix.py REPO_DIR REF"""
import itertools, os, subprocess, sys, tempfile, yaml
repo, ref = sys.argv[1:3]
wf = yaml.safe_load(subprocess.check_output(["git", "-C", repo, "show", f"{ref}:.github/workflows/containerized_ci.yml"], text=True))
step = next(s for s in wf["jobs"]["prepare"]["steps"] if s.get("id") == "set")
head = subprocess.check_output(["git", "-C", repo, "rev-parse", ref], text=True).strip()
events = [("push", {"PUSH_BEFORE": "0" * 40}), ("push", {"PUSH_BEFORE": ""}), ("pull_request", {"HEAD_REPO": "SAME"}),
          ("pull_request", {"HEAD_REPO": "other/fork"}), ("workflow_dispatch", {"INPUT_IMAGE": "both"}),
          ("workflow_dispatch", {"INPUT_IMAGE": "gnu"}), ("workflow_dispatch", {"INPUT_IMAGE": "ifx"}),
          ("workflow_dispatch", {"INPUT_IMAGE": "garbage"}), ("workflow_dispatch", {}), ("repository_dispatch", {}),
          ("schedule", {}), ("", {})]
repos = ["thinkthoughts/ucla-roms", "", "thinkthoughts/ucla-roms-x", "cworthy-ocean/ucla-roms", "CWorthy-ocean/ucla-roms-fork", "CWorthy-ocean/ucla-roms"]
fails = 0
for this_repo, (ev, extra) in itertools.product(repos, events):
    env = {k: v for k, v in os.environ.items() if k not in ("EVENT", "INPUT_IMAGE", "PR_BASE", "PR_HEAD", "PUSH_BEFORE", "PUSH_AFTER", "HEAD_REPO", "THIS_REPO")}
    env.update(EVENT=ev, PUSH_AFTER=head, PR_HEAD=head, THIS_REPO=this_repo, RUN_ID="1", GITHUB_OUTPUT=tempfile.mktemp())
    for k, v in extra.items(): env[k] = this_repo if v == "SAME" else v
    with tempfile.TemporaryDirectory() as d:
        subprocess.run(["git", "-C", repo, "worktree", "add", "-q", "--detach", d, ref], check=True)
        r = subprocess.run(["bash", "-c", step["run"]], env=env, cwd=d, capture_output=True, text=True)
        subprocess.run(["git", "-C", repo, "worktree", "remove", "--force", d], check=True)
    out = dict(l.split("=", 1) for l in open(env["GITHUB_OUTPUT"]).read().splitlines() if "=" in l and not l.endswith("<<EOF"))
    writes = out.get("build_gnu") == "true" or out.get("build_ifx") == "true" or out.get("can_push") == "true"
    upstream = this_repo == "CWorthy-ocean/ucla-roms"
    ok = (not writes) if not upstream else True
    fails += not ok
    if not upstream or ev in ("push", "workflow_dispatch"):
        print(f"{'ok  ' if ok else 'FAIL'} repo={this_repo or '(empty)':30} event={ev or '(empty)':18} {str(extra)[:28]:28} rc={r.returncode} build_gnu={out.get('build_gnu')} build_ifx={out.get('build_ifx')} can_push={out.get('can_push')}")
print("plan-step cases that could write outside CWorthy-ocean/ucla-roms:", fails); sys.exit(1 if fails else 0)

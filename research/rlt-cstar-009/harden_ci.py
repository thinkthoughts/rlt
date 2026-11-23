"""RLT-CSTAR-009 CI-branch workflow changes (fork only), applied as text edits so the
diff stays minimal. Usage: harden_ci.py REPO_KIND WORKTREE"""
import sys
from pathlib import Path
kind, root = sys.argv[1], Path(sys.argv[2])
TRIG = "  # fork CI for RLT-CSTAR-009 only"

def guard_steps(text, uses_prefix, cond, note):
    """Add `if: cond` to every step whose `uses:` starts with uses_prefix; refuse if the
    step already has an `if:` (so the condition never silently replaces another)."""
    lines = text.split("\n"); out = []; n = 0
    for i, l in enumerate(lines):
        out.append(l)
        s = l.strip()
        if s.startswith(f"uses: {uses_prefix}"):
            indent = l[: len(l) - len(l.lstrip())]
            j = i - 1                                   # step start: the "- " line above
            while not lines[j].lstrip().startswith("- "): j -= 1
            k = i + 1                                   # step end: next "- " at the step's indent or less
            step_indent = len(lines[j]) - len(lines[j].lstrip())
            while k < len(lines) and not (lines[k].strip() and len(lines[k]) - len(lines[k].lstrip()) <= step_indent):
                k += 1
            block = lines[j:k]
            assert not any(b.strip().startswith("if:") for b in block), f"step at line {j+1} already has if:"
            out.append(f"{indent}if: {cond}  # {note}"); n += 1
    return "\n".join(out), n

if kind == "roms-tools":
    p = root / ".github/workflows/tests.yaml"; s = p.read_text()
    old = "on:\n  push:\n    branches:\n      - main\n"; assert old in s
    s = s.replace(old, f"on:\n  push:\n    branches:\n      - main\n      - rlt-cstar/ci-009{TRIG}\n", 1)
    s, n = guard_steps(s, "codecov/codecov-action", "github.repository_owner == 'CWorthy-ocean'", "fork CI: no Codecov upload")
    p.write_text(s); print(p.name, "codecov steps guarded:", n)
elif kind == "C-Star":
    for f in ("unit_tests.yaml", "integration_tests.yaml"):
        p = root / ".github/workflows" / f; s = p.read_text()
        old = "  push:\n    branches: [main]\n"; assert old in s
        s = s.replace(old, f"  push:\n    branches: [main, rlt-cstar/ci-009]{TRIG}\n", 1)
        s, n = guard_steps(s, "codecov/codecov-action", "github.repository_owner == 'CWorthy-ocean'", "fork CI: no Codecov upload")
        p.write_text(s); print(f, "codecov steps guarded:", n)
elif kind == "ucla-roms":
    p = root / ".github/workflows/containerized_ci.yml"; s = p.read_text()
    old = "  push:\n    branches:\n      - main\n  pull_request:\n    branches: ['*']\n"; assert old in s
    s = s.replace(old, f"  push:\n    branches:\n      - main\n      - rlt-cstar/ci-009{TRIG}\n  pull_request:\n    branches: ['*']\n", 1)
    # 1. read-only default token for every job that does not ask for more
    assert "\npermissions:" not in s and s.count("\njobs:\n") == 1
    s = s.replace("\njobs:\n", "\n# Fork CI (RLT-CSTAR-009): read-only default token.\npermissions:\n  contents: read\n\njobs:\n", 1)
    # 2. plan step: never build or push outside CWorthy-ocean
    old2 = '''          {
            echo "build_gnu=$build_gnu"'''
    assert s.count(old2) == 1
    s = s.replace(old2, '''          # Fork CI (RLT-CSTAR-009, not for upstream): outside CWorthy-ocean never
          # build an image or push to ghcr.io/cworthy-ocean; test against the
          # published :latest images, read-only. An empty or unknown repository
          # name also takes this branch.
          if [ "$THIS_REPO" != "CWorthy-ocean/ucla-roms" ]; then
            build_gnu=false; build_ifx=false; can_push=false
            gnu_image="$GNU_IMG:latest"; ifx_image="$IFX_IMG:latest"
          fi

''' + old2, 1)
    # 3. job-level conditions: the jobs holding packages: write never start in a fork
    G = "github.repository == 'CWorthy-ocean/ucla-roms'"
    for job, old_if in (("build_gnu", "${{ needs.prepare.outputs.build_gnu == 'true' }}"),
                        ("build_ifx", "${{ needs.prepare.outputs.build_ifx == 'true' }}"),
                        ("cleanup", "${{ always() && github.event_name == 'pull_request' && needs.prepare.outputs.can_push == 'true' }}")):
        line = f"    if: {old_if}"; assert s.count(line) == 1, job
        s = s.replace(line, f"    if: ${{{{ {G} && {old_if[4:-3]} }}}}  # fork CI: never in a fork", 1)
    # 4. step-level, second layer: ghcr login, build-push, and the package delete
    lines = s.split("\n"); out = []; added = 0; i = 0
    targets = ("uses: docker/login-action", "uses: docker/build-push-action", "name: Delete ephemeral CI image tags")
    while i < len(lines):
        l = lines[i]; out.append(l)
        if l.lstrip().startswith("- name:") or l.lstrip().startswith("- uses:"):
            ind = l[: len(l) - len(l.lstrip())] + "  "
            j = i + 1                                   # end of this step
            while j < len(lines) and not (lines[j].strip() and len(lines[j]) - len(lines[j].lstrip()) <= len(ind) - 2):
                j += 1
            block = "\n".join(lines[i:j])
            if any(t in block for t in targets):
                ifs = [k for k in range(i + 1, j) if lines[k].startswith(ind + "if:")]
                assert len(ifs) <= 1
                if ifs:
                    cond = lines[ifs[0]].strip()[3:].strip()
                    cond = cond[3:-2].strip() if cond.startswith("${{") else cond
                    lines[ifs[0]] = f"{ind}if: ${{{{ {G} && ({cond}) }}}}  # fork CI: never in a fork"
                else:
                    out.append(f"{ind}if: {G}  # fork CI: never in a fork")
                added += 1
        i += 1
    s = "\n".join(out)
    p.write_text(s); print("containerized_ci.yml: job guards 3, step guards", added)

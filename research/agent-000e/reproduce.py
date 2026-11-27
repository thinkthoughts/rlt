"""Behavioral reproduction of AG-F02; creates temporary synthetic records only."""
import hashlib
import json
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BIN = ROOT / 'target/release/rlt'
PROOF = 'tests/definitions/certificates/F3_double_two.rlt'
DEFS = 'definitions/double.rlt'
PIN = '8dabb459817f7b225419456080a4e81eb675daf9a830dba529192ff50cfbc2f7'


def sha(path):
    return hashlib.sha256((ROOT / path).read_bytes()).hexdigest()


def run(args, evidence, name, is_json=False):
    p = subprocess.run([str(BIN), '--root', str(ROOT), *args], cwd=ROOT,
                       capture_output=True, timeout=60)
    (evidence / (name + '.stdout')).write_bytes(p.stdout)
    (evidence / (name + '.stderr')).write_bytes(p.stderr)
    print(name, 'exit', p.returncode, flush=True)
    if is_json:
        x = json.loads(p.stdout)
        if 'exit' in x:
            assert x['exit'] == p.returncode
        return p.returncode, x
    return p.returncode, p.stdout.decode()


def main():
    import sys
    evidence = Path(sys.argv[1]).resolve()
    assert not evidence.is_relative_to(ROOT)
    evidence.mkdir(parents=True, exist_ok=False)
    assert sha(DEFS) == PIN
    code, check = run(['check', PROOF], evidence, 'check')
    assert code == 0 and 'verdict         ACCEPT' in check
    assert 'definitions     ' + DEFS in check and PIN in check
    goal = next(line[16:] for line in check.splitlines()
                if line.startswith('checked goal    '))
    code, deps = run(['deps', PROOF, '--format', 'json'], evidence, 'deps', True)
    assert code == 0 and deps['root']['verdict'] == 'ACCEPT'
    template = """format: rlt-statement/1
id: AG-F02-PROBE
subject: "Synthetic provenance test only; no semantic source match claimed"
source:
  registry: LEAN-CORE-NAT-DVD
  commit: 6a10ac8c22beadecabdbb0919c2b50214762f91d
  excerpts:
    - "dvd a b := Exists (fun c => b = a * c)"
kernel:
  proof: %s
  sha256: %s
  convention: NAT-FUN
%s  axiom_set: ROBINSON-Q
  axiom_set_sha256: 4ad55192222576e3901abc4ee7ea5a6760b64ff212e8c55796ab9847e423b00c
  checked_goal: %s
  assumptions: []
semantic:
  status: unassessed
physical:
  applicability: not-applicable
  reason: "arithmetic test"
"""
    results = {}
    with tempfile.TemporaryDirectory(prefix='ag-f02-', dir=ROOT) as temp:
        for name, pin in [('correct-pin', PIN), ('omitted-pin', None),
                          ('wrong-pin', '0' * 64)]:
            record = Path(temp) / (name + '.yaml')
            record.write_text(template % (PROOF, sha(PROOF),
                '  definitions_sha256: ' + pin + '\n' if pin else '',
                json.dumps(goal, ensure_ascii=False)), encoding='utf-8')
            (evidence / record.name).write_bytes(record.read_bytes())
            code, report = run(['explain', str(record.relative_to(ROOT)), '--format', 'json'],
                               evidence, name, True)
            k = report.get('kernel') or {}
            results[name] = {'exit': code, 'state': report['record']['state'],
                             'kernel_verdict': k.get('verdict'),
                             'definitions': k.get('definitions'),
                             'stale': report['record']['stale']}
    observed = {'records': results, 'deps_definitions_used': deps['root']['definitions_used'],
                'deps_fingerprint_text': deps['root']['fingerprint_text']}
    print(json.dumps(observed, ensure_ascii=False, indent=2))
    (evidence / 'observed.json').write_text(json.dumps(observed, ensure_ascii=False, indent=2)+'\n')
    # Predeclared reproduction criteria, not desired future behavior.
    assert results['correct-pin']['exit'] == 3
    assert results['omitted-pin']['exit'] == 0
    assert results['wrong-pin']['exit'] == 3
    assert all(v['kernel_verdict'] == 'ACCEPT' and v['definitions'] is None
               for v in results.values())
    assert deps['root']['definitions_used'] == []
    assert 'definitions none\n' in deps['root']['fingerprint_text']
    print('AG-F02 REPRODUCED: reporting/record identity gap, not a kernel counterexample')


if __name__ == '__main__':
    main()

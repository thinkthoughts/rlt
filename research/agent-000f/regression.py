"""Post-fix regression for AG-F02; preserves historical reproduction separately."""
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
id: %s
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
            record_id = 'AG-F02-' + name.upper()
            record = Path(temp) / (record_id + '.yaml')
            record.write_text(template % (record_id, PROOF, sha(PROOF),
                '  definitions_sha256: "' + pin + '"\n' if pin else '',
                json.dumps(goal, ensure_ascii=False)), encoding='utf-8')
            (evidence / record.name).write_bytes(record.read_bytes())
            code, report = run(['explain', str(record.relative_to(ROOT)), '--format', 'json'],
                               evidence, name, True)
            if report['record']['state'] == 'invalid':
                print('INVALID RECORD DIAGNOSTIC:', json.dumps(report, ensure_ascii=False), flush=True)
            k = report.get('kernel') or {}
            results[name] = {'exit': code, 'state': report['record']['state'],
                             'kernel_verdict': k.get('verdict'),
                             'definitions': k.get('definitions'),
                             'stale': report['record']['stale']}
    observed = {'records': results, 'deps_definitions_used': deps['root']['definitions_used'],
                'deps_fingerprint_text': deps['root']['fingerprint_text']}
    print(json.dumps(observed, ensure_ascii=False, indent=2))
    (evidence / 'observed.json').write_text(json.dumps(observed, ensure_ascii=False, indent=2)+'\n')
    # Post-fix expectations; dependency v1 projection intentionally unchanged.
    assert results['correct-pin']['exit'] == 0
    assert results['omitted-pin']['exit'] == 3
    assert results['wrong-pin']['exit'] == 3
    assert all(v['kernel_verdict'] == 'ACCEPT' and v['definitions'] == {
        'file': DEFS, 'sha256': PIN, 'used': ['double']
    } for v in results.values())
    assert results['correct-pin']['state'] == 'current'
    for name in ('omitted-pin', 'wrong-pin'):
        assert results[name]['state'] == 'stale'
        assert len(results[name]['stale']) == 1
        assert results[name]['stale'][0]['what'] == 'definitions sha256'
        assert results[name]['stale'][0]['current'] == PIN
    assert deps['root']['definitions_used'] == []
    assert 'definitions none\n' in deps['root']['fingerprint_text']
    print('AG-F02 EXPLAIN REGRESSION PASS; legacy dependency fingerprint unchanged')


if __name__ == '__main__':
    main()

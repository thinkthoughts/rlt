"""Experimental RLT-AGENT-000a: offline checks of rlt-explain/1 reports.

Only current, accepted NAT-DEF reports with pinned sources are supported.
This is NOT a proof checker or a claim/correspondence verifier. A caller must
supply an independently obtained byte digest and reported process exit.
"""
import argparse
import hashlib
import json
import re
from pathlib import Path

MAX_BYTES = 4 * 1024 * 1024
FIXTURE_SHA = '445d26c27ba42a545f36e13ce667bbaff5fdd17c0b0b1fbd02b636ec7b8c11b8'
LEAN_NOTE = 'scripts/verify.sh (Lean #print axioms gate) for records in statements/; not rlt explain'


class Refusal(ValueError):
    pass


def require(ok, message):
    if not ok:
        raise Refusal(message)


def unique(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, 'duplicate JSON key')
        result[key] = value
    return result


def no_constant(_):
    raise Refusal('nonstandard JSON constant')


def fields(value, names):
    require(type(value) is dict and set(value) == set(names.split()),
            'missing or unexpected object fields')
    return value


def digest(value):
    return type(value) is str and re.fullmatch('[0-9a-f]{64}', value) is not None


def nonempty(value):
    return type(value) is str and bool(value.strip())


def audit(raw, process_exit, expected_sha):
    """Check only the stated profile; never return ACCEPT or a new theorem."""
    result = {'format': 'rlt-explain-structural-audit/experimental-1',
              'status': 'REFUSED', 'proof_replayed': False,
              'lean_replayed': False, 'upstream_checked': False,
              'physical_verified': False}
    try:
        require(type(raw) is bytes and len(raw) <= MAX_BYTES, 'invalid input size')
        result['report_sha256'] = hashlib.sha256(raw).hexdigest()
        require(digest(expected_sha) and result['report_sha256'] == expected_sha,
                'report digest mismatch')
        require(type(process_exit) is int and process_exit == 0,
                'unsupported or unsuccessful process exit')
        x = json.loads(raw.decode('utf-8'), object_pairs_hook=unique,
                       parse_constant=no_constant)
        fields(x, 'format record exit exit_meaning kernel source semantic physical open_questions')
        require(x['format'] == 'rlt-explain/1', 'unsupported report format')
        require(type(x['exit']) is int and x['exit'] == process_exit,
                'embedded exit differs from process exit')
        r = fields(x['record'], 'path id subject identifiers state problems stale')
        require(r['state'] == 'current' and r['problems'] == [] and r['stale'] == [],
                'record is not current and problem-free')
        k = fields(x['kernel'], 'verdict verdict_line checker_exit proof sha256 library_id library_index proof_id written_goal convention definitions checked_goal axiom_set axiom_set_sha256 proved assumptions axioms_used rules_used theorems_cited steps cites check_report')
        require(type(k['checker_exit']) is int and k['checker_exit'] == 0
                and k['verdict'] == 'ACCEPT', 'not a reported authoritative acceptance')
        require(k['convention'] == 'NAT-DEF', 'unsupported convention')
        require(all(nonempty(k[key]) for key in ('proof', 'proof_id', 'written_goal',
                    'checked_goal', 'proved', 'axiom_set', 'assumptions', 'check_report')),
                'missing proof identity or scope')
        require(k['checked_goal'] == k['proved'], 'reported goal differs from proved formula')
        require(digest(k['sha256']) and digest(k['axiom_set_sha256']), 'invalid proof or axiom digest')
        d = fields(k['definitions'], 'file sha256 used')
        require(d['file'] == 'spec/definitions.rlt' and digest(d['sha256'])
                and type(d['used']) is list and all(nonempty(v) for v in d['used']),
                'unsupported definition provenance')
        require(type(k['cites']) is list, 'invalid citations')
        for c in k['cites']:
            fields(c, 'label path proof_id sha256 proved')
            require(digest(c['sha256']), 'invalid cited proof digest')
        s = fields(x['source'], 'registry registry_status commit location excerpts registry_fidelity pinned upstream')
        require(s['pinned'] is True and nonempty(s['commit']) and s['registry_fidelity'] == 'verified',
                'unsupported source registration')
        require(s['upstream'] == 'not run by rlt explain; scripts/check_excerpts.py compares registered excerpts with the source at ' + s['commit'] + ' (run by scripts/verify.sh)',
                'unexpected upstream verification assertion')
        m = fields(x['semantic'], 'status assessment stale_reasons assessed_by date basis assessed_goal assessed_goal_current matches differs lean')
        require(type(m['lean']) is list, 'invalid Lean references')
        for l in m['lean']:
            fields(l, 'file theorem named_in_file checked_by')
            require(l['named_in_file'] is True and l['checked_by'] == LEAN_NOTE,
                    'unexpected Lean verification assertion')
        p = fields(x['physical'], 'applicability reason reading_point records')
        require(p['applicability'] == 'not-applicable' and p['records'] == []
                and p['reading_point'] is None, 'unsupported physical correspondence')
        result['status'] = 'PINNED_STRUCTURE_MATCH'
        result['reported_kernel_verdict'] = 'ACCEPT'
    except (ValueError, TypeError, KeyError, RecursionError) as exc:
        result['problem'] = str(exc) if isinstance(exc, Refusal) else 'invalid report data'
    return result


def read_bounded(path):
    with path.open('rb') as handle:
        return handle.read(MAX_BYTES + 1)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('report', type=Path)
    parser.add_argument('--process-exit', required=True, type=int)
    parser.add_argument('--expected-sha256', required=True)
    args = parser.parse_args()
    try:
        raw = read_bounded(args.report)
    except OSError:
        parser.exit(2, 'cannot read report\n')
    result = audit(raw, args.process_exit, args.expected_sha256)
    print(json.dumps(result, sort_keys=True))
    return 0 if result['status'] == 'PINNED_STRUCTURE_MATCH' else 1


if __name__ == '__main__':
    raise SystemExit(main())

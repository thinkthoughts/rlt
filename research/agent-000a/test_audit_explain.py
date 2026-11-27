"""Run against the exact committed golden; mutations are not RLT outputs."""
import copy
import hashlib
import json
import os
import unittest
from pathlib import Path

from audit_explain import audit, FIXTURE_SHA

DEFAULT = Path(__file__).resolve().parents[2] / 'tests/explain-golden/MOD30-CLASSIFICATION.json'


class AuditTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.raw = Path(os.environ.get('RLT_EXPLAIN_FIXTURE', DEFAULT)).read_bytes()
        if hashlib.sha256(cls.raw).hexdigest() != FIXTURE_SHA:
            raise ValueError('fixture bytes do not match pinned SHA-256')
        cls.report = json.loads(cls.raw)

    def refuse(self, raw, process_exit=0, pin=None):
        result = audit(raw, process_exit, pin or hashlib.sha256(raw).hexdigest())
        self.assertEqual(result['status'], 'REFUSED', result)
        self.assertNotIn('reported_kernel_verdict', result)

    def test_exact_fixture(self):
        result = audit(self.raw, 0, FIXTURE_SHA)
        self.assertEqual(result['status'], 'PINNED_STRUCTURE_MATCH')
        for key in ('proof_replayed', 'lean_replayed', 'upstream_checked', 'physical_verified'):
            self.assertIs(result[key], False)

    def test_digest_change(self):
        self.refuse(self.raw + b' ', pin=FIXTURE_SHA)

    def test_process_exit(self):
        for code in (1, 2, 3, 4, -9, True):
            with self.subTest(code=code):
                self.refuse(self.raw, code)

    def test_invalid_json(self):
        for raw in (b'{', b'\xff', b'{"format":1,"format":2}', b'{"x":NaN}', b'[]'):
            with self.subTest(raw=raw):
                self.refuse(raw)

    def test_mutations_with_recomputed_digest(self):
        # A matching digest alone is not authentication. These test shape checks.
        cases = [
            (('format',), 'rlt-explain/2'),
            (('exit',), True),
            (('exit',), 3),
            (('record', 'state'), 'stale'),
            (('record', 'stale'), ['changed']),
            (('kernel', 'verdict'), 'EXPLORATORY'),
            (('kernel', 'checker_exit'), 4),
            (('kernel', 'convention'), 'NAT-FUN'),
            (('kernel', 'assumptions'), None),
            (('kernel', 'proved'), '0 = 1'),
            (('kernel', 'sha256'), 'invalid'),
            (('kernel', 'definitions', 'file'), 'definitions/double.rlt'),
            (('kernel', 'cites', 0, 'sha256'), 'invalid'),
            (('source', 'upstream'), 'verified'),
            (('source', 'pinned'), False),
            (('semantic', 'lean', 0, 'checked_by'), 'Lean ACCEPT'),
            (('physical', 'applicability'), 'applicable'),
        ]
        for path, value in cases:
            with self.subTest(path=path, value=value):
                obj = copy.deepcopy(self.report)
                parent = obj
                for key in path[:-1]:
                    parent = parent[key]
                parent[path[-1]] = value
                self.refuse(json.dumps(obj).encode())

    def test_missing_and_extra_fields(self):
        obj = copy.deepcopy(self.report)
        del obj['kernel']['assumptions']
        self.refuse(json.dumps(obj).encode())
        obj = copy.deepcopy(self.report)
        obj['semantic']['lean'][0]['status'] = 'PASS'
        self.refuse(json.dumps(obj).encode())

    def test_coherent_modification_is_not_authenticated(self):
        obj = copy.deepcopy(self.report)
        obj['semantic']['matches'] = ['untrusted changed text']
        raw = json.dumps(obj).encode()
        self.refuse(raw, pin=FIXTURE_SHA)
        result = audit(raw, 0, hashlib.sha256(raw).hexdigest())
        self.assertEqual(result['status'], 'PINNED_STRUCTURE_MATCH')
        self.assertFalse(result['proof_replayed'])


if __name__ == '__main__':
    unittest.main()

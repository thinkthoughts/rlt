import subprocess
import tempfile
import unittest
from pathlib import Path
from capture import capture, sha


class CaptureTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name) / 'repo'
        self.root.mkdir()
        self.record = self.root / 'record.yaml'
        self.record.write_text('test\n')
        for args in [('init', '-q'), ('add', '.'), ('-c', 'user.name=Test', '-c', 'user.email=test@example.invalid', 'commit', '-qm', 'fixture')]:
            subprocess.run(['git', '-C', str(self.root), *args], check=True)
        self.binary = Path(self.tmp.name) / 'fake-rlt'

    def fake(self, body):
        self.binary.write_text('#!/bin/sh\n' + body + '\n')
        self.binary.chmod(0o700)

    def test_exact_streams_and_exit(self):
        self.fake("printf 'report'; printf 'diagnostic' >&2; exit 3")
        meta, out, err = capture(self.binary, self.root, 'record.yaml')
        self.assertEqual((out, err, meta['process_exit']), (b'report', b'diagnostic', 3))
        self.assertEqual(meta['stdout_sha256'], sha(out))
        self.assertEqual(meta['stderr_sha256'], sha(err))
        self.assertFalse(meta['proof_verified_by_capture'])
        self.assertFalse(meta['authentication_established'])

    def test_dirty_checkout(self):
        self.fake('exit 0')
        self.record.write_text('changed')
        with self.assertRaisesRegex(ValueError, 'clean'):
            capture(self.binary, self.root, 'record.yaml')

    def test_changed_record(self):
        self.fake('echo changed >> record.yaml')
        with self.assertRaisesRegex(ValueError, 'changed'):
            capture(self.binary, self.root, 'record.yaml')

    def test_path_traversal(self):
        self.fake('exit 0')
        with self.assertRaises(ValueError):
            capture(self.binary, self.root, '../record.yaml')

    def test_timeout(self):
        self.fake('exec sleep 5')
        with self.assertRaises(subprocess.TimeoutExpired):
            capture(self.binary, self.root, 'record.yaml', timeout=0.5)


if __name__ == '__main__':
    unittest.main()

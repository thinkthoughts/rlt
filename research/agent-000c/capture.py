"""Experimental local capture of an RLT explain invocation, not attestation.
Run only a trusted executable in a quiescent, trusted checkout.
"""
import hashlib
import json
import subprocess
from pathlib import Path


def sha(data):
    return hashlib.sha256(data).hexdigest()


def capture(binary, root, record, timeout=30):
    binary = Path(binary).resolve(strict=True)
    root = Path(root).resolve(strict=True)
    rel = Path(record)
    if rel.is_absolute() or '..' in rel.parts or rel.suffix != '.yaml':
        raise ValueError('expected repository-relative YAML record')
    path = (root / rel).resolve(strict=True)
    if not path.is_relative_to(root) or not path.is_file():
        raise ValueError('record outside checkout or not a file')
    def git(*args):
        return subprocess.run(['git', '-C', str(root), *args], check=True,
                              capture_output=True, timeout=timeout).stdout
    def snapshot():
        return {'head': git('rev-parse', 'HEAD').decode().strip(),
                'status': git('status', '--porcelain=v1', '--untracked-files=all').decode(),
                'binary_sha256': sha(binary.read_bytes()),
                'record_sha256': sha(path.read_bytes())}
    before = snapshot()
    if before['status']:
        raise ValueError('checkout must be clean')
    args = [str(binary), '--root', str(root), 'explain', str(rel), '--format', 'json']
    run = subprocess.run(args, cwd=root, capture_output=True, timeout=timeout)
    after = snapshot()
    if before != after:
        raise ValueError('observed execution context changed')
    meta = {'format': 'rlt-local-capture/experimental-1', 'command': args,
            'record': str(rel), 'observed': before, 'process_exit': run.returncode,
            'stdout_sha256': sha(run.stdout), 'stderr_sha256': sha(run.stderr),
            'authentication_established': False, 'proof_verified_by_capture': False}
    return meta, run.stdout, run.stderr


def main():
    import argparse
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('binary'); p.add_argument('root'); p.add_argument('record')
    p.add_argument('output', type=Path, help='new directory outside checkout')
    a = p.parse_args()
    output = a.output.resolve()
    if output.is_relative_to(Path(a.root).resolve()):
        p.error('output must be outside checkout')
    meta, stdout, stderr = capture(a.binary, a.root, a.record)
    output.mkdir(parents=False, exist_ok=False)
    (output / 'stdout.bin').write_bytes(stdout)
    (output / 'stderr.bin').write_bytes(stderr)
    (output / 'capture.json').write_text(json.dumps(meta, indent=2, sort_keys=True) + '\n')
    print(json.dumps(meta, sort_keys=True))
    return 0 if meta['process_exit'] == 0 else 1


if __name__ == '__main__':
    raise SystemExit(main())

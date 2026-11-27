# RLT-AGENT-000c — Local process capture (experimental)

Date: 2026-11-26. Parent: dcfa69e5b00b0b5866208cd02bf1589a6fb5fcaa. Branch: rlt-agent-000c. Status: harness tested; actual RLT execution pending.

## Scope and trust

The capture harness invokes an explicitly supplied, trusted local executable with fixed explain arguments. It preserves exact stdout/stderr and the observed process return code, hashes the executable and record, and observes Git HEAD and status before and after execution. It requires a trusted, quiescent checkout and a trusted executable; it is not a sandbox, attestation, build provenance system, or mathematical checker. It does not authenticate the executable or establish that it was built from HEAD. Git status does not cover ignored inputs or transient modifications; the snapshots do not prove which bytes the process read. Embedded proof, axiom, citation and source hashes are not independently resolved. Output capture is buffered in memory and intended only for trusted local processes; timeout is not a process-tree sandbox. No untrusted executable should be supplied.

## Executed tests

Five Python unittest methods passed locally (Python 3.13.5): exact byte streams and exit 3; dirty checkout refusal; changed record detection; traversal refusal; timeout. Tests used shell substitutes and temporary Git repositories, not RLT. The published source blobs at 07d89f9f89a545167f6d9d8b3a29dec0d652913a match the tested local blobs: capture.py 0a013c376f21889345857fb188c969d4f9acccae; test_capture.py 31a6df3c280f021db0a39f2040cf051d67f9a4dd.

Reproduce from a clean checkout of this branch:

    python3 -m unittest discover -s research/agent-000c -p 'test_*.py' -v
    cargo build --release --locked
    python3 research/agent-000c/capture.py target/release/rlt . statements/MOD30-CLASSIFICATION.yaml /tmp/rlt-agent-000c-capture

Use a new output directory outside the repository. The first command may create untracked bytecode if not ignored; remove it or use python3 -B before capture. Compare stdout.bin against the committed golden independently. Do not infer verification from the capture command exit alone: inspect the captured process exit and report. The build and live capture commands above have not been executed in this session.

## Blockers and next work

Local environment lacks Cargo/Rust and cannot resolve github.com for cloning. Existing GitHub Actions run 45929685457 was in progress when inspected; no success claimed. CI configuration unchanged. Actual RLT capture, proof replay, embedded-hash resolution and AG-F02 NAT-FUN provenance reproduction remain outstanding. Before expanding the adapter, obtain a pinned real executable and reproducible run; then review typed outcome extraction. Independent review and merge pending.

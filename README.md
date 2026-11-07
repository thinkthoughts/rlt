# rlt
readingleveltest.com

## RLT — Reading Level Test checker

RLT is an independent proof checker for reading points. It checks derivations with its own small kernel, rejects invalid ones, and reports every result with the specifications and assumptions it depends on.

Reading points math0, math5 and math29 keep all their documented meanings (`readingpoints/*.yaml`, traced to `sources/registry.yaml`). Arithmetic relationships such as `5 + 1 = 6` are attached to a reading point as checked results.

### Four layers

| Layer | Decides | Code |
|---|---|---|
| Written expression | written identity: `"1+1"` and `"2"` are distinct | `syntax/src/written.rs` |
| Convention | which term a written expression denotes (NAT-DEC, PiπΠ conventions) | `conventions/`, `syntax/src/elab.rs` |
| Checked relationship | mathematical equality, by kernel proof only | `kernel/` — trusted, zero dependencies |
| Physical correspondence | physical status, by evidence only; `imagined` kept separate | `readingpoints/*.yaml`, `correspondence/` |

### Use

```sh
cargo build --release
target/release/rlt check tests/accept/T1_math5_pairing.rlt   # exit 0 accept, 1 reject, 2 malformed
target/release/rlt written "1+1" NAT-DEC "2" NAT-DEC
target/release/rlt rp math5
target/release/rlt validate
scripts/verify.sh                                             # everything, writes verification/RLT-001-results.txt
```

### Trusted computing base

`kernel/` (terms, sort checking, six rules `refl symm trans cong inst hyp`, assumption bookkeeping, step replay and goal comparison), the axiom file `spec/peano.rlt`, and the Rust toolchain. Parsers, conventions, sources, records and the Lean replay supply inputs or cross-checks; none of them can make the kernel accept a step.

Status: RLT-001 (initial implementation, under review). Reports: `docs/RLT-000.md`, `docs/RLT-001.md`.

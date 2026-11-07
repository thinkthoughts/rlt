# rlt
readingleveltest.com

## RLT — Reading Level Test checker

RLT is an independent proof checker. It checks derivations with its own small kernel, rejects invalid ones, and reports every result with the axioms, assumptions and reading-point specifications associated with it. A verdict concerns the checked equation only: it establishes neither a reading point's meanings nor any physical claim.

Reading points math0, math5 and math29 keep all their documented meanings as whole verbatim units of source excerpts (`readingpoints/*.yaml`, traced to `sources/registry.yaml`; `rlt validate` checks each unit and checks completeness). Arithmetic relationships such as `5 + 1 = 6` are attached to a reading point as checked results, bound to the proof whose goal they state.

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
target/release/rlt lean tests/accept/*.rlt                    # Lean replay of the accepted statements
scripts/verify.sh SOURCE_CLONES_DIR                           # everything, writes verification/RLT-001-results.txt
```

### Trusted computing base

- `kernel/`: terms, sort checking, the six rules `refl symm trans cong inst hyp`, assumption bookkeeping, step replay and goal comparison. Every theorem records Γ and the full content of every axiom it uses.
- The registered axiom sets: `spec/axioms.yaml` (id, file, SHA-256, source) and the files it locks (`spec/peano.rlt`), plus their loader `load_axiom_set` in `cli/src/lib.rs`. A proof file names a set by id; paths are refused; the verdict names the set.
- The Rust toolchain (pinned in `rust-toolchain.toml`).

Parsers, conventions, sources, reading-point and correspondence records, and the Lean replay supply inputs or cross-checks; none of them can make the kernel accept a step. The kernel API itself checks derivations relative to whatever axioms its caller supplies; trust in axioms is decided by the registered-set loader.

Limits: numerals up to 1000 (unary in the kernel), nesting depth 256, 1,000,000 term nodes per proof file.

Status: RLT-001 (initial implementation, under review). Reports: `docs/RLT-000.md`, `docs/RLT-001.md`, `docs/RLT-001-AUDIT.md`.

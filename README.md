# rlt
readingleveltest.com

## RLT — Reading Level Test checker

RLT is an independent proof checker. It checks derivations with its own small kernel, rejects invalid ones, and reports every result with the axioms, assumptions and reading-point specifications associated with it. An ACCEPT says the checked equation follows from the listed axioms under the listed assumptions Γ. The axioms remain assumptions: their SHA-256 binds their exact content, and their mathematical truth and any physical correspondence stay unestablished by the check. A REJECT says the kernel refused that derivation and leaves the goal's truth undetermined.

Reading points math0, math5 and math29 keep all their documented meanings as whole verbatim units of source excerpts (`readingpoints/*.yaml`, traced to `sources/registry.yaml`). `rlt validate` checks each unit and checks completeness relative to the registry; `scripts/check_excerpts.py` checks the registry against the source files: every excerpt verbatim at its commit, every tagged excerpt a whole unit of its source, and every source line naming a reading point covered by an excerpt. Conversation sources have no file; their wording is pinned in the binary. Arithmetic relationships such as `5 + 1 = 6` are attached to a reading point as checked results, bound to the proof whose goal, mode and assumptions they state.

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
scripts/verify.sh SOURCE_CLONES_DIR                           # fmt, clippy, tests, proofs, records, Lean, excerpts; exit 0 only if all hold
```

### Trusted computing base

- `kernel/`: terms, sort checking, the six rules `refl symm trans cong inst hyp`, assumption bookkeeping, step replay and goal comparison. Every theorem records Γ and the full content of every axiom it uses.
- The approved axiom sets, pinned in the binary (`PINNED_AXIOM_SETS` in `cli/src/lib.rs`: id, file, SHA-256), their mirror `spec/axioms.yaml`, the files they lock (`spec/peano.rlt`), and their loader `load_axiom_set`. A proof file names a set by id; paths, symbolic links, duplicate entries and any hash other than the pinned one are refused; the verdict names the set.
- The Rust toolchain (pinned in `rust-toolchain.toml`).

Parsers, conventions, sources, reading-point and correspondence records, and the Lean replay supply inputs or cross-checks; none of them can make the kernel accept a step. The kernel API itself checks derivations relative to whatever axioms its caller supplies; trust in axioms is decided by the registered-set loader.

Limits: numerals up to 1000 (unary in the kernel; no leading zeros), nesting depth 256, 1,000,000 term nodes per proof file; control characters are refused in proof files and records.

Status: RLT-001 (initial implementation, under review). Reports: `docs/RLT-000.md`, `docs/RLT-001.md`, `docs/RLT-001-AUDIT.md`.

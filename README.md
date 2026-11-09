# rlt
readingleveltest.com

## RLT — Reading Level Test checker

RLT is an independent proof checker. It checks derivations with its own small kernel, rejects invalid ones, and reports every result with the axioms, assumptions and reading-point specifications associated with it. An ACCEPT says the checked equation follows from the listed axioms under the listed assumptions Γ. The axioms remain assumptions: their SHA-256 binds their exact content, and their mathematical truth and any physical correspondence stay unestablished by the check. A REJECT says the kernel refused that derivation and leaves the goal's truth undetermined.

Reading points math0, math5 and math29 keep all their documented meanings as whole verbatim units of source excerpts (`readingpoints/*.yaml`, traced to `sources/registry.yaml`). `rlt validate` checks each unit and checks completeness relative to the registry; `scripts/check_excerpts.py` checks the registry against the source files: every excerpt verbatim at its commit, every tagged excerpt a whole unit of its source, and every source line naming a reading point covered by an excerpt. The registry's documentation content (every source and every excerpt with its tag) is pinned in the binary by SHA-256, so adding, removing, truncating or retagging a documented meaning requires a reviewed code change; this also fixes the wording of the conversation sources, which have no file. Arithmetic relationships such as `5 + 1 = 6` are attached to a reading point as checked results, bound to the proof whose goal, mode and assumptions they state.

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
target/release/rlt points points/oxygen-and-people.yaml       # RLT-002: one equation, distinct reading points
scripts/demo-rlt-002.sh                                       # RLT-002 workflow demonstration
target/release/rlt chem "CO₂" CHEM-MOLFORMULA                 # T7: a registered formula (COO, CO2, O2 stay uninterpreted)
target/release/rlt points points/co2-and-o2.yaml              # T7: two molecules, five atoms, three checked counts
target/release/rlt check tests/accept/M1_thirty.rlt           # RLT-003 M1: 30 = 2 × 3 × 5 under PEANO-ARITH
target/release/rlt gen "30 = 2 × 3 × 5" M1.thirty             # untrusted generator; the kernel checks its output
target/release/rlt residues residues/residue-30.yaml          # RLT-003 M2: 111 closed witnesses modulo 30 and their registered readings
scripts/gdt-compare.sh GDT_CHECKOUT                           # M2 GDT comparison, reported separately (compiled or unverified)
target/release/rlt check tests/m3a/accept/A09_forall_elim_intro.rlt  # RLT-003 M3a: first-order derivations (∀, ∃, ∧, ∨, →, ⊥)
target/release/rlt check tests/m3b/accept/BA06_two_divides_eight.rlt  # RLT-003 M3b: 2 ∣ 8 under ROBINSON-Q, with the pinned definition of ∣
target/release/rlt check tests/m3c/accept/CA02_two_not_divides_seven.rlt  # RLT-003 M3c: ¬(2 ∣ 7), every multiplication witness eliminated by finite case analysis
target/release/rlt check tests/m3c/accept/CA05_free235_seven.rlt  # RLT-003 M3c: free235(7), citing three independently checked theorems
scripts/verify.sh SOURCE_CLONES_DIR                           # fmt, clippy, tests, proofs, records, Lean, excerpts; exit 0 only if all hold
```

### Trusted computing base

- `kernel/`: terms and formulas, sort checking, the 20 inference rules (`RULES` in `kernel/src/check.rs`: refl, symm, trans, cong, inst, hyp, subst and the 13 logical rules), assumption bookkeeping, step replay and goal comparison. Every theorem records Γ and the full content of every axiom it uses. RLT-003 M3c: the kernel re-validates every cited theorem (Γ = ∅, closed conclusion, axioms identical to the registered set); a `cite` step returns it unchanged and is not an inference rule.
- Theorem citation (RLT-003 M3c), in `cli/src/lib.rs`: `citation_path` and the citation loop in `check_inner`, which hash a cited proof file, refuse cycles and long chains, check the file from scratch on the hashed bytes, and pass the resulting kernel theorem on.
- The approved axiom sets, pinned in the binary (`PINNED_AXIOM_SETS` in `cli/src/lib.rs`: id, file, SHA-256), their mirror `spec/axioms.yaml`, the files they lock (`spec/peano.rlt`), and their loader `load_axiom_set`. A proof file names a set by id; paths, symbolic links, duplicate entries and any hash other than the pinned one are refused; the verdict names the set.
- The Rust toolchain (pinned in `rust-toolchain.toml`).

Parsers, conventions, sources, reading-point and correspondence records, and the Lean replay supply inputs or cross-checks; none of them can make the kernel accept a step. The kernel API itself checks derivations relative to whatever axioms its caller supplies; trust in axioms is decided by the registered-set loader.

Limits: numerals up to 1000 (unary in the kernel; no leading zeros), nesting depth 256, 1,000,000 term nodes per proof file; control characters are refused in proof files and records.

Status: RLT-001 merged; RLT-002 merged; RLT-003 M1 merged; RLT-003 M2 merged; RLT-003 M3a merged; RLT-003 M3b merged; RLT-003 M3c (divisibility and non-divisibility proofs, theorem citation) under review. Reports: `docs/RLT-000.md`, `docs/RLT-001.md`, `docs/RLT-001-AUDIT.md`, `docs/RLT-002.md`, `docs/RLT-002-AUDIT.md`, `docs/RLT-002-T7.md`, `docs/RLT-003-M1.md`, `docs/RLT-003-M2.md`, `docs/RLT-003-M3a.md`, `docs/RLT-003-M3b.md`, `docs/RLT-003-M3c.md`.

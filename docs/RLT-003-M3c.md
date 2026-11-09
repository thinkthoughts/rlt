# RLT-003 · M3c — Divisibility and non-divisibility proofs: implementation report

**Scope.** This report covers M3c as approved on 2026-11-09: D1–D7 as recommended, with two refinements.
- §5 of the review rests M3c's soundness solely on the explicit, kernel-checked derivations. The general "Q proves every true closed bounded statement" claim is background only, with no part in the argument.
- The D5 citation audit is strengthened. Each cited theorem must be the exact kernel result of checking the named file, with its conclusion, Γ = ∅, axiom set, axioms used and file hash preserved and printed. Substitution, cycles and paths outside the repository are attacked.

Architecture review: <https://claude.ai/artifact/3LDhtmXQ9tBdK5sWbrLAgq>; its §3 and Appendix A give the derivation scheme formula by formula.

**Result.**
- **General statements:** RLT proves ∀k. ¬(7 = d × k) and ¬(d ∣ 7) for d = 2, 3 and 5 under ROBINSON-Q. Each multiplication witness is eliminated as an eigenvariable, and a finite case tree built from Q3 closes every case with Q1 and Q2. No induction axiom is used.
- **free235(7):** proved by citing the three independently checked non-divisibility theorems. Its report carries each dependency's file hash, proved formula, Γ, axiom set and axioms used.
- **Kernel:** the 20 inference rules are unchanged. The only trusted addition is the approved citation mechanism (§2).
- **Tests:** 9 accepted proofs and 31 rejected files behave as specified, and 17 further checks run as Rust tests.
- **Earlier results:** all 312 earlier proof-file reports are byte-identical to main 93e85a5, and so are the outputs of `rlt validate` and every record command.

## 1. What changed

| File | Change |
|---|---|
| `kernel/src/check.rs` | `ProofObject.cited`: (label, `Theorem`) pairs. `check` re-validates each cited theorem: Γ = ∅, a well-formed closed conclusion, and every axiom identical in content to one registered for this proof. A `cite` step returns the cited theorem unchanged; the generic claim comparison then requires the step to state exactly its conclusion. `cite` is bookkeeping and is absent from `RULES`, which still lists the 20 rules. |
| `syntax/src/proof.rs` | `(cites (LABEL "path" "sha256") …)` field (identifier labels, 64-digit lower-case hex); `(cite LABEL)` step argument; `reads_definitions(src)`: the NAT-DEF decision on the parsed `convention` field. |
| `cli/src/lib.rs` | `citation_path` (repository-relative `.rlt` path of plain components, checked on the raw spelling as well, no symlink at any level, a regular file at the end); `check_inner` with the citing chain (cycle refusal, at most `MAX_CITATION_DEPTH` = 8 citing files in a chain), a budget of `MAX_CITED_CHECKS` = 64 cited-file checks per top-level check, and the exact bytes that were hashed; `cites` report lines; definitions loaded from the parsed convention field (M3b audit note 1). |
| `scripts/gen-m3c-tests.py` | Deterministic, untrusted generator for tests/m3c (D4). |
| `tests/m3c/` | `accept/` CA01–CA09; `reject/` 31 files; `expected.tsv`. |
| `cli/tests/m3c.rs` | 17 tests: accept and reject suites, CT1–CT3, CR14, CR23c, citation depth and work bounds, path spellings, convention mismatch, file swap, citations kept out of `rules used`, and three kernel-API refusals of forged or altered cited theorems. |
| `lean/M3cGenerated.lean`, `scripts/gen-m3c-lean.sh` | Tier 1: the nine checked goals over ℕ. |
| `lean/M3cReadings.lean` | Tier 2: six correspondence lemmas. |
| `scripts/verify.sh` | M3c stage; Lean compile and per-theorem axiom gates for M3cGenerated, M3cReadings and M3aGenerated (M3b audit note 4); M3cGenerated regeneration check. |
| `docs/RLT-003-M3b.md` | Wording for M3b audit notes 3 and 5. |
| `README.md` | M3c examples and status. |

Unchanged, checked by hash or byte comparison:
- the 20 inference rules in `kernel/src/rules.rs`;
- `spec/` (PEANO-ADD, PEANO-ARITH, ROBINSON-Q, definitions), `conventions/` and `readingpoints/`;
- `residues/`, `points/` and `sources/registry.yaml`.

## 2. Theorem citation (D5 (b))

**Form.** A proof file declares `(cites (T2 "tests/m3c/accept/CA02_two_not_divides_seven.rlt" "bafaef…94e1b"))`, and a step `(t2 (not (dvd 2 7)) (cite T2))` uses it.

**What the CLI does, per citation, in order:**
1. **Path.** The path must be repository-relative, end in `.rlt`, and consist of plain components. The raw spelling has no `..`, `.`, empty component, leading `/` or backslash. It must contain no symlink at any level and end at a regular file.
2. **Cycles and bounds.** The citation is refused if the cited file is already in the chain of citing files (self-citation and longer cycles alike). It is also refused if the chain would hold more than 8 citing files, or if the whole citation tree would need more than 64 cited-file checks.
3. **Hash.** The file is read once, and those bytes' SHA-256 must equal the recorded hash.
4. **Check.** The cited file is checked from scratch on exactly those bytes, by the same function, with the citing chain extended.
5. **Requirements on the result:** verdict ACCEPT, a kernel theorem, the same axiom set id as the citing file, the same convention, and Γ = ∅.
6. **Report.** Six lines are printed: `cites` (label, path, cited proof id), `sha256`, `proved`, `assumptions`, `axiom set` and `axioms used`. After acceptance, `rules used` lists only the kernel inference rules the result rests on. A separate `theorems cited` line names the citations it rests on and calls them checked theorem dependencies, not inference rules.
7. **Hand-off.** The kernel theorem that check returned is passed to the citing file's proof object.

**What the kernel does.** Before replay, the kernel re-validates each cited `Theorem`:
- it must be unconditional;
- its conclusion must be well formed and free of parameters;
- every axiom it rests on must be identical in name and content to the axiom of that name registered for the citing proof.

A `cite` step's theorem is the cited `Theorem` value itself. The citing step's claim must equal its conclusion, or the step is refused (CR22, CR24).

**Soundness (C1–C5).**
- **C1. Every cited theorem is a kernel theorem.** `Theorem` has private fields, so only the kernel's rule functions construct one. The CLI obtains a cited theorem only as the `Verdict::Accepted` value of `check` on the cited file, run in the same process on the hashed bytes.
- **C2. The cited theorem is valid under the citing axiom set.** It is unconditional and rests only on axioms identical to the citing proof's registered axioms; the kernel checks this itself and does not rely on the CLI. Using it as a premise is therefore ordinary composition of derivations under one axiom set, and no new inference rule is needed. The kernel-API tests confirm the refusals: an axiom with the same name but different content, a conditional theorem, and a parameter in the conclusion.
- **C3. The meaning is preserved.** The cited file must use the same convention. Its conclusion is the kernel formula, and the citing step must restate exactly that formula.
- **C4. Dependency information is preserved.** Each `cites` block prints the hash, goal, Γ, axiom set and axioms used, and the citing theorem's `axioms used` is the union that `and-intro` computes. CT3 checks both against independent recomputation.
- **C5. Termination and bounded work.** Cycles are refused before any cited file is read. Depth is bounded by 8 citing files and total work by 64 cited-file checks.

**Trusted additions, exactly.** Citation adds the following trusted code; nothing else enters the trusted base.
- in the kernel: the re-validation block and the `cite` case in `check.rs`;
- in the CLI: `citation_path`, the citation loop (chain, budget, hash, exact-bytes check, requirement checks, hand-off), and the `preread` parameter.

## 3. The non-divisibility derivations

**Scheme** (review §3, Appendix A). `hyp h : ∃k. 7 = d × k`, then `exists-elim h p₀ h₀`. Layer i holds the fact nᵢ = d × pᵢ.
- **Split.** Q3 at pᵢ splits it into 0 or a successor.
- **Zero branch:** d × 0 = 0 (mul_zero), so nᵢ = 0, which `q2_zero_ne_succ` at x = nᵢ − 1 refutes.
- **Successor branch:** `exists-elim` opens pᵢ = S(pᵢ₊₁). Lemma M_d (one mul_succ, d add_succ, one add_zero) gives nᵢ = S^d(d × pᵢ₊₁), and Q1 peels successors. An inner layer reaches nᵢ₊₁ = d × pᵢ₊₁. The last layer reaches 0 = S^(d−r)(d × p_L), which Q2 refutes at the arbitrary remaining variable.

In the ¬-form, `imp-intro h` closes the proof. In the witness-wise form, `imp-intro h₀` then `forall-intro p₀` close it.

| File | Goal | Layers (Q3) | Eigenvariables | Steps | Bytes |
|---|---|---|---|---|---|
| CA01 | 2 ∣ 8 | — | — | 51 | 2540 |
| CA02 | ¬(2 ∣ 7) | 4 | p0–p4 | 128 | 7920 |
| CA03 | ¬(3 ∣ 7) | 3 | p0–p3 | 110 | 7179 |
| CA04 | ¬(5 ∣ 7) | 2 | p0–p2 | 92 | 6566 |
| CA05 | free235(7) | cites CA02–CA04 | — | 5 | 792 |
| CA06 | ∀k. ¬(7 = 2 × k) | 4 | p0–p4 | 127 | 7915 |
| CA07 | ∀k. ¬(7 = 3 × k) | 3 | p0–p3 | 109 | 7174 |
| CA08 | ∀k. ¬(7 = 5 × k) | 2 | p0–p2 | 91 | 6561 |
| CA09 | ¬(2 ∣ 7), from CA06 by citation | — | a | 7 | 607 |

CT1 counts the axiom instances in CA02–CA04 and CA06–CA08 against the review's §4 table and finds exact agreement:
- q3 = L and q2 = L + 1;
- q1 = 7 in every file, since together the peels remove the seven successors of 7;
- mul_succ = add_zero = mul_zero = L, and add_succ = L·d.

Every one of these files uses exactly the seven ROBINSON-Q axioms. Their step labels name the layer and branch: `L2_z_q2` is the zero branch's Q2 step at layer 2, and `L3_s_pl1` is the last peel at layer 3.

## 4. Test results

| Suite | Result |
|---|---|
| `cargo test` | all pass: 69 golden, 9 hardening, 44 M2, 8 M3a, 14 M3b, 17 M3c, 29 kernel, 21 syntax (211) |
| tests/m3c/accept | 9 of 9 ACCEPT as unconditional theorems under ROBINSON-Q |
| tests/m3c/reject | 31 of 31 refused with the required exit code and reason (`expected.tsv`) |
| Rust-run checks | CT1, CT2, CT3, CR14, CR23c (symlinked file and symlinked directory), citation depth (exactly 8 citing files accepted, 9 refused), citation work (62 checks accepted, 126 refused), `.` and empty path components refused, a cited theorem under another convention refused, cited file swapped for another proof of the same goal, kernel refusal of a foreign-axiom, a conditional and a parametric cited theorem, `cite` outside `RULES` with an undeclared label refused, `rules used` naming only kernel rules across every accepted proof, a proof made only of a citation (`rules used none`) |
| `verify.sh` | `RESULT all checks hold`. Generators reproduce tests/m3c byte for byte. M3cGenerated (9), M3cReadings (6), M3aGenerated (17) and every earlier Lean file compile under the standard-axiom gates. Source excerpts: 0 problems. |

Rejected cases (exit code · refusing step and reason; exact substrings in `tests/m3c/expected.tsv`):

| ID | Planted fault | Result |
|---|---|---|
| CR01 | witness 3 for 2 ∣ 7 | 1 · exists-intro: witness does not match premise |
| CR02 | zero branch omitted at layer 2 | 1 · or-cases takes 5 arguments, given 3 |
| CR03 | Q3 at the stale p1 in layer 2 | 1 · the step claims p2's split, `inst` yields p1's |
| CR04 | layer 1 reopens at p1 | 1 · exists-elim: `p1` occurs in the eliminated formula |
| CR05 | eigenvariable in the conclusion | 1 · exists-elim: `p0` occurs in the conclusion |
| CR06 | one peel too few | 1 · imp-elim: antecedent does not match |
| CR07 | one peel too many | 1 · imp-elim: antecedent does not match (0 = S(2 × p4)) |
| CR08 | Q2 at x = 5 against 0 = 7 | 1 · imp-elim: antecedent does not match |
| CR09 | M₂ missing one add_succ | 1 · trans endpoints differ |
| CR10 | the scheme on ¬(2 ∣ 8) | 1 · the zero branch at p4 reaches 0 = 0; `¬(0 = 0)` is no Q2 instance |
| CR11 | declared assumption, unconditional mode | 1 · unconditional goal requested, the result depends on h |
| CR12 | free235 conjuncts swapped | 1 · and-intro yields the swapped conjunction |
| CR13 | an `induction` axiom | 1 · axiom `induction` is unregistered |
| CR15a/b | parameter `dvd`, symbol and quoted convention | 2 · collides with a definition name |
| CR16 | binder `free235`, quoted convention | 2 · shadowed name |
| CR17a | recorded hash of another file | 2 · the citation records a different SHA-256 |
| CR17b | path of another proof of ¬(2 ∣ 7) (CA09) with CA02's hash | 2 · hash differs |
| CR18 | cites the rejected CR01 | 2 · not an accepted theorem (REJECT …) |
| CR19 | cites a PEANO-ARITH theorem | 2 · axiom set differs |
| CR20 | cites the conditional BA11 | 2 · the cited theorem is conditional |
| CR21a | self-citation | 2 · citation cycle |
| CR21b/c | two-file cycle (from each end) | 2 · citation cycle / recorded hash differs |
| CR21d/e/f | three-file cycle (from each file) | 2 · citation cycle / recorded hash differs |
| CR22 | states ¬(2 ∣ 9), cites CA02 | 1 · the cited theorem `T2` proves ¬(∃k. 7 = 2 × k) |
| CR23a/b | `..` path; absolute path | 2 · plain repository-relative path required |
| CR24 | label T3 used for ¬(2 ∣ 7) | 1 · the cited theorem `T3` proves ¬(∃k. 7 = 3 × k) |

**The negative control (CR10).** The same scheme on 8 descends 8, 6, 4, 2, 0. At p₄ = 0 it reaches 0 = 0, and the closing step must claim ¬(0 = 0). The only Q2 instance on offer is ¬(0 = S(0)), so the kernel refuses the step. No proof of ¬(2 ∣ 8) can be accepted, because every accepted theorem holds in ℕ.

## 5. Earlier results

`scripts/compare-reports.sh` ran the 93e85a5 binary on its tree and this branch's binary on this tree. Every report it covers is byte-identical:
- all 312 earlier proof-file reports (tests/accept, reject, m1, m2, m3a, m3b and the written layer);
- `rlt validate`;
- `rp math0`, `rp math5` and `rp math29`;
- both `points` records, `residues`, `chem`, and the Lean export of tests/accept.

The reading points, RESIDUE-30, the axiom sets, the definitions and every convention keep their bytes and pins. Loading definitions from the parsed `convention` field changes no earlier report.

## 6. Report-format changes and deviations, for review

1. **`cites` lines.** These are the approved addition (review §6). They appear only in files that cite.
2. **`theorems cited` line.** Reusing an established theorem is reported apart from deriving one. `rules used` lists only kernel inference rules (CA05: `and-intro`; a proof made only of a citation: `none`). Citing files add one line: theorems cited · T2, T3, T4 — checked theorem dependencies (see `cites`), not inference rules. A Rust test checks every accepted proof in the repository: each name in `rules used` is one of the 20 `RULES`, and `cite` appears only as a dependency. Files without citations print no new line, so earlier reports are unchanged.
3. **Citation failures are MALFORMED (exit 2).** The review table listed CR18 as REJECT. A citation failure (path, hash, cycle, depth, a cited file that does not check, a different axiom set or convention, a conditional theorem) means the citing file's input is invalid, so it is reported as MALFORMED. The nested verdict is quoted, and CR18 shows `REJECT at step e — …`. A citing step whose claim differs from the cited conclusion is a kernel REJECT (CR22, CR24).
4. **CA09 added.** The review's CT2 asked for the witness-wise to ¬-form bridge "in a separate accepted demonstration". CA09 is that demonstration, and it obtains its premise by citing CA06. Accepted files: 9.
5. **Step labels use underscores** (`L2_z_q2`) where the review sketched dots (`L2.z.c3`).
6. **Scratch-repository cases** run as Rust tests: CR14 (edited definitions) and CR23c (symlinks, which `git` stores but the review asked to exercise with real links).
7. **Rejected-file count.** The 23 planned citation and derivation cases became 31 files, because several cases are covered from more than one angle: CR15a/b, CR17a/b, CR21a–f and CR23a/b.

## 7. Limitations

- **Scope of the results.** M3c proves the five targets and their witness-wise forms for n = 7 only. The generator handles any closed d ∤ n, but no general ∀n metatheorem is claimed or used (review §5, refinement 1).
- **Lean's role.** Lean checks the statements and the tier-2 correspondence. It establishes no RLT derivation.
- **Citation scope.** A citation is accepted only for unconditional theorems under the same axiom set and convention (review §10). Cross-set or conditional citation would need its own review.
- **Proof size.** Proofs grow linearly in L · d. The largest M3c file is 7.9 KB, far inside the 1,000,000-node limit.
- **Hardlinks.** A cited path may be a hardlink to a file outside the repository (audit note 2). The SHA-256 binds the bytes, so the cited theorem is the check of exactly the recorded content.
- **No caching.** Each citation re-checks its file from scratch. That keeps every cited theorem a fresh kernel result, at the cost of repeated work, which the budget of 64 cited-file checks bounds.

## 8. Reproduce

```sh
cargo build --release
scripts/verify.sh SOURCE_CLONES_DIR
target/release/rlt check tests/m3c/accept/CA02_two_not_divides_seven.rlt
target/release/rlt check tests/m3c/accept/CA05_free235_seven.rlt
scripts/compare-reports.sh OLD_RLT OLD_ROOT target/release/rlt .   # OLD = main 93e85a5
```

## 9. Independent adversarial audit

**Method.** An auditor with no part in the implementation worked from a clean clone of 9dc3bd9 and built it from source. It reran the full suite and the comparison against a built 93e85a5. Its own work:
- an s-expression counter, independent of CT1;
- an independent scheme generator;
- attack files: 79 derivation-attack files in item 2 (56 divisible-pair, 12 off-by-one, 11 branch or eigenvariable mutations; one off-by-one file was a no-op identical to the correct proof), plus 16 accepted control proofs from its own generator, among 239 `.rlt` paths it created in all (237 regular files, one symlink, one directory). These are the auditor's recounted figures, which correct the "56 + 14" of its first report;
- a scratch kernel-API test.

**Verdict.** Merge recommended. There are no substantive findings, and all six required checks pass:
1. **Reproduction:** 206 tests, fmt and clippy clean, `verify.sh` prints `RESULT all checks hold`, and compare-reports shows 312 of 312 identical.
2. **Obligations:** the independent counts match §3. Q3 is never applied at p_L, and the last case closes by Q2 at x = 2 × p₄, S(3 × p₃) and S²(5 × p₂). The auditor's own generator proves ¬(d ∣ n) for eight further pairs. Every attempt on divisible pairs, every off-by-one, every branch mutation and every false statement (2 ∣ 7, ¬(2 ∣ 8), ¬(2 ∣ 9), free235(9), free235(25)) is refused.
3. **Citation:** the code reads each cited file once and checks those bytes. Every substitution, cycle (2, 3 and 4 files, from every entry), depth, path (`..`, absolute, `./`, backslash, Unicode look-alikes, symlinked file and directories), axiom-set and convention attack is refused. In the kernel API, forged theorems are refused: a fake axiom with a registered name, an unregistered `induction`, a q2 copy with a different source, and a q2 copy with a renamed variable.
4. **Preservation:** `git diff 93e85a5 9dc3bd9` is empty for `kernel/src/rules.rs`, `spec`, `conventions`, `readingpoints`, `residues`, `points`, `sources`, `correspondence` and `lifts`. `RULES` still has 20 entries.
5. **Lean:** M3cGenerated (9), M3cReadings (6) and M3aGenerated (17) use standard axioms only, and the generated files reproduce.
6. **Doc claims:** confirmed, with the wording notes below.

**Non-substantive findings and follow-ups.**

| # | Finding | Follow-up |
|---|---|---|
| 1 | `tests/./m3c/…` and `tests//m3c/…` were accepted: `Path::components` drops interior `.` and empty components, while the doc said "no `.`" | Fixed: the raw spelling is checked too. A Rust test covers `./`, interior `.` and `//`. |
| 2 | A hardlink to a file outside the repository is accepted | Documented (§7). The hash binds the content, so the cited theorem is still the check of exactly those bytes. |
| 3 | The depth test checked 7 and 9 citing files, not the exact boundary | Fixed: exactly 8 citing files are accepted and 9 refused, with the wording made precise. |
| 4 | Re-checking each citation from scratch grows exponentially with fan-out × depth (15 s at depth 4 with 6 citations per file) | Fixed: a budget of 64 cited-file checks per top-level check. A Rust test accepts 62 and refuses 126. |
| 5 | No test isolated a convention mismatch | Added: a Rust test where a ROBINSON-Q NAT-LOGIC theorem cited from a NAT-DEF file is refused. |
| 6 | README trusted-base section still said "the six rules" | Fixed: it now names the 20 rules and the citation mechanism. |
| 7 | `verification/RLT-001-results.txt` records the commit before itself | Expected for a self-recording file. |

**Final reporting corrections (approved 2026-11-09).**
- **Audit counts:** the counts above are the auditor's exact figures.
- **Citations reported apart:** citations are reported as theorem dependencies, apart from the 20 inference rules (§6 item 2). Two regression tests cover this. A citing step whose claim differs is now refused with "the cited theorem `T` proves …", not "rule `cite` yields …" (CR22, CR24).

**After the follow-ups.**
- `cargo test`: 211 of 211 pass.
- clippy clean.
- `verify.sh`: `RESULT all checks hold`.
- compare-reports against 93e85a5: 312 compared, 0 differ.

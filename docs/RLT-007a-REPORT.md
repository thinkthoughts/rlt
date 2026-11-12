# RLT-007a — Hardening and performance: implementation report

**Status:** implemented on `docs/rlt-007` (PR #15, draft), measured against main `e5850d9`, 2026-11-12. Independent adversarial audit: see §7.

**Unchanged:**
- the kernel, the 20 inference rules plus `cite`, the seven axioms of Robinson's Q;
- the resource limits (1,000,000 term nodes per file, 64 cited-file checks, citation chain 8);
- every proof file, statement record, library entry, generated certificate and manifest hash.

**Not added:** no persistent proof cache, and no verification result is reused between processes or runs.

**Still the merge gate:** the complete `scripts/verify.sh` run in CI. It checks everything it checked at `e5850d9`, plus two new checks (§6).

## 1. Hardening H1–H4

| Item | What changed | Evidence |
|---|---|---|
| **H1** residue profile | New `scripts/check-h45-profile.py` replaces the inline arithmetic check. It reads the reviewed transcription only, never the generator, and checks every field of `certificates/h45/residues.tsv` (14 × 45 rows in order, h equal to H₄₅, r_p equal to the table, h = p·q + s with 0 ≤ s < p, s ≠ r_p) and of `profile.tsv` (one row per prime in order; occupied classes = {h mod p}; count; omitted = the sorted complement; r_p among the omitted) | `--self-test`: **21 of 21** altered tables refused: every alteration the RLT-006 audit found accepted, plus six the RLT-007a audit found (the same values written another way, and layout changes). verify.sh runs both modes |
| **H2** interrupts | verify.sh now runs as a supervisor around a child in its own process group (a `set -m` job, checked by its process-group id). The supervisor re-runs itself by absolute path. On INT, TERM, HUP or QUIT the supervisor stops the whole group, including parallel `rlt`, `xargs` and Lean runs, restores any file a tamper test had altered, removes the generated certificates and the run's temporary directory, writes an `INTERRUPTED` line, and exits 130, 143, 129 or 131. It never prints a `RESULT` line. A backup is restored only when it is marked complete | TERM during the parallel certificate stage: stopped in 3 s, exit 143, no `rlt` or `xargs` process left, no temporary directory left, `tests/rlt006/gen` removed, no tracked file changed apart from the results file. INT 130, HUP 129, QUIT 131. TERM to a supervisor that leads its own session and process group: stopped in 3 s, nothing left. Run by a relative path from `tests/`: the right script runs. A partial backup leaves every tracked file in place |
| **H3** fixed CI inputs | New `scripts/fetch-sources.py` fetches every GitHub source by its pinned full commit SHA (`git fetch --depth 1 --filter=blob:none origin SHA`); no branch is read. lean-bgp212 is fetched at `7898eb4…`. The two abbreviated registry SHAs resolve through the new `sources/commit-pins.tsv`; a row that does not extend its abbreviation, a duplicate row or an unused row is refused. The arXiv e-print and the PDF come from an Actions cache keyed by the registry, transcriptions, pin table and fetch script. They are downloaded only on a cache miss and **refused before writing** unless equal to their SHA-256 pins; `check_excerpts.py` then re-checks every byte as before. Every repository, fresh or from the cache, is re-hashed with `git fsck --full`, so a cached object whose bytes do not match its name fails the run. A weekly `source-provenance` job (and manual dispatch) re-fetches everything live with no cache. It gates nothing and runs no proofs | 10 repositories, 10 pinned commits, 0 missing or corrupt; `check_excerpts.py` 0 problems. A mocked download with the wrong bytes is refused and nothing is written. A poisoned object in a cached repository is reported `CORRUPT` and the fetch exits 1. verify.sh runs `fetch-sources.py --check-pins` offline and checks that the script fetches lean-bgp212 at the commit the vendored blocks are compared with |
| **H4** report count | `docs/RLT-006-REPORT.md` §8 now says 30 `grep_q` pipes on 29 lines, with a dated correction note | `grep -o '\| grep_q' scripts/verify.sh` at `e5850d9`: 30 |

## 2. Performance changes

All changes are in scheduling: which independent processes run at the same time, and how many times the same output is produced. None alters a check's inputs or its acceptance condition.

| Id | Change | Why the result is unchanged |
|---|---|---|
| P1 | `par_map` in verify.sh runs independent `rlt` and Lean processes `RLT_JOBS` at a time (default: the processor count; plain decimal 1–64). Each item's stdout, stderr and exit code go to files named by input position. They are read back in input order, and stderr is passed on to the log, as before. Item lists are built once, as arrays | The report is the same whatever order the runs finish in. Each run is a separate process with no shared state. Every item must leave an exit code, or the stage fails |
| P1a | the 630 direct certificate checks | Each must exit 0 and print the ACCEPT verdict, the same condition as `rlt check | grep_q` under `pipefail` at `e5850d9` |
| P1b | the 14 `rlt deps` runs on the per-prime theorems | Same budget and exit lines required, and now also exit code 0 (at `e5850d9` the exit code was not tested: audit S6) |
| P1c | `rlt explain`, text and JSON, for the 19 statement records (38 processes) | Same per-record line and conditions |
| P1d | the 6 RLT-006 record fixtures | Same exit code and fragment |
| P1e | the 17 Lean replay files, and the Lean-reference gate files | Logs are joined in the fixed file order. Any failing run fails the stage. At `e5850d9` only the last file's stderr reached the log; now every file's does (stricter) |
| P2 | The Lean-reference collection reuses the 19 JSON reports produced in P1c, instead of running `rlt explain --format json` a third time | The same bytes, from the same binary, in the same run |
| P3 | `cli/tests/rlt006.rs`: the missing-certificate and modified-certificate cases are now two tests, so the harness runs them at the same time. Each scratch tree's control is now the affected entry accepted before the change (`rlt deps`), instead of a third whole-library verification | Afterwards the whole library is still verified once per case, and exactly the affected entry fails. With every other entry accepted afterwards, this is the same evidence as before |
| P4 | CI's separate `cargo test --release` step is removed; verify.sh already runs it and fails on any failing test | Tests run once per CI job instead of twice |

## 3. Measurements against `e5850d9`

**Setup.** The RLT-007a column is the run with the audit fixes (§7); the run before them took 1,066 s, and the run after the re-audit fixes took 1,032 s (user 1,488 s, system 54 s) with the same report lines. Same machine (2 processors), same pinned sources (`/tmp/srcfetch`), binaries and test executables prebuilt in both trees, nothing else running. Each run: `scripts/verify.sh` timed by bash `time`.

| | main `e5850d9` | RLT-007a | Change |
|---|---|---|---|
| Wall time | 1,622 s | **1,060 s** | −35% |
| CPU, user | 1,808 s | 1,518 s | −16% |
| CPU, system | 91 s | 59 s | −35% |
| Result | all checks hold | all checks hold | — |
| Generated certificates on disk during the run | 57,528 KB | 57,528 KB | equal |
| Working tree after the run (excluding `target/`, `.git/`) | 69,236 KB | 69,344 KB | +108 KB (new scripts and the RLT-007 documents) |
| Parallel temporary files | — | under 3 MB, removed after each stage | — |
| Pinned sources for CI | full default-branch clones plus live downloads | 2.9 MB (partial clones by SHA, plus the two archived files) | no branch read |

**By stage (wall seconds):**

| Stage | `e5850d9` | RLT-007a |
|---|---|---|
| `cargo test --release` | 770 | 533 |
| ↳ `cli/tests/rlt006.rs` alone (measured separately) | 498 | 290 |
| RLT-006 stage (630 checks, 14 deps, rejects, tamper, fixtures, profile) | 245 | 137 |
| ↳ 630 direct checks | 80 | 42 |
| ↳ 14 per-prime `rlt deps` | 89 | 41 |
| statement records and `rlt explain` (incl. Lean references) | 337 | 146 |
| library and `rlt deps --verify-lib` | 169 | 161 |
| Lean replay | 69 | 56 |
| everything else | 32 | 27 |

**CI (GitHub's 4-processor runner), `check` job:** 27 min 7 s at `e5850d9`; **10 min 7 s** at `9ae34c5` (before the audit fixes, on a source-cache miss, which fetched and pin-checked every source).

**Coverage.** The two run logs were compared line by line, with timings normalized. They differ in exactly these places:
- the commit line;
- the RLT-006 test binary reporting 9 tests instead of 8 (the split in P3);
- the residue-profile line, which now also names its checker and the 21-case self-test;
- two added lines: the commit-pin table check and the lean-bgp212 pin cross-check.

Every other report line is identical. `scripts/check-manual.py`: 33/33 examples match.

## 4. Within-run reuse, parallel citation checking and redundancy: findings

- **An in-process citation memo would save almost nothing today.** Within one `rlt` process, each RLT-006 certificate is cited by exactly one per-prime theorem. Neither `--verify-lib` nor `explain` reaches the same certificate twice. The repeats are *across* processes: the direct check, `deps`, `explain`, the library verifier and `cargo test`. A memo cannot remove those without a cache shared between processes, which is excluded. **Not implemented.**
- **In-process parallel citation checking is blocked by the current data structures.** A check result holds `Rc<Definitions>` with a `RefCell`, so it cannot cross threads. Making it `Send` means changing `syntax` (Arc and Mutex) on the path every proof check takes. That is close enough to the trusted base to need its own design and audit. **Deferred**; process-level parallelism (P1) gives most of the gain without it.
- **Redundant checking that remains, kept deliberately.** Each item is a separate entry point, and each guards a different interface. Removing any of them would reduce what the mandatory run covers, so none was removed:

| Repeated work | Cost | Entry points that each check it |
|---|---|---|
| 630 certificates checked directly | 42 s | verify.sh (CLI) and `cargo test` (library API) |
| 630 certificates checked again through citation | 14 `deps` runs; 14 records × 2 formats in `explain` | `deps` and `explain` |
| whole library verified | 167 s | verify.sh `rlt deps --verify-lib`, `cli/tests/deps.rs` (173 s) and twice in `cli/tests/rlt006.rs` |

  A reviewer could approve dropping one of the library verifications, or the direct 630-check in verify.sh (the citations also check every certificate). Either would save a further 40–170 s. That is a coverage decision, and it is left to you.

## 5. Fast CI tier

**Not added.** With P1–P4, the mandatory complete run is about 18 minutes on 2 processors, and less on CI's 4. If a fast tier is wanted later, it would supplement that run and never replace it. It would cover format, lints, unit tests, generator reproduction, the manifest and the 14 per-prime theorems. Branch protection, which would make the complete check required, still needs your separate approval.

## 6. Tests and checks added

- `scripts/check-h45-profile.py`, plus `--self-test` (21 altered-table cases), both run by verify.sh.
- `scripts/fetch-sources.py --check-pins` (offline), run by verify.sh.
- `cli/tests/rlt006.rs`: the missing and modified cases as two tests, each with its own before-the-change control.
- **Manual (recorded here):**
  - interrupt by TERM and by INT;
  - a mocked wrong download refused with nothing written;
  - an idempotent re-run of `fetch-sources.py` on a populated directory ("present" for every item).

## 7. Independent adversarial audit

Required because P1–P3 change proof-check scheduling and reuse. An independent agent, which had not seen the work produced, audited `9ae34c5` against `e5850d9`.

**Method.**
- Fake `rlt` and `lean` wrappers that fail, crash or are killed on chosen items.
- Trimmed copies of the parallel stages.
- A harness built from the supervisor.
- A poisoned cache.

**Findings.** None blocking as CI runs it. Six should-fix and eleven notes, all addressed in the next commit:

| Finding | Fix |
|---|---|
| S1 a relative `$0` could re-run a different `verify.sh` after the `cd` | absolute path taken before the `cd` |
| S2 `setsid` forks when the caller leads its process group; the supervisor then exited 0 at once | `set -m` job; process-group id checked, else the run stops |
| S3 HUP and QUIT reached only the supervisor | trapped like TERM (129, 131) |
| S4 an interrupt during the backup copy restored from a partial backup and deleted tracked files (also at `e5850d9`) | staged backup, renamed and marked COMPLETE only when whole; restores require the mark |
| S5 a poisoned Actions cache could feed false source text (git does not re-hash on read) | `git fsck --full` on every repository, every run |
| S6 the `rlt deps` stage ignored exit codes (also at `e5850d9`) | exit code 0 required |
| Notes 1, 3, 4, 5 | `RLT_JOBS` normalised; item lists as arrays; stdout and stderr kept apart; one Lean call before the parallel gate (toolchain installed once) |
| Note 2 | P1a wording corrected (equivalent, not stricter) |
| Note 10 | H1 accepts plain ASCII decimals only, LF line ends, header comments only, no blank lines; 45/14/14 pinned in the script; six new self-test cases |
| Note 11 | the lean-bgp212 pin is cross-checked between the two scripts |
| Notes 6–9 | documented: SIGINT already ignored by the starting shell cannot be trapped; SIGKILL of the supervisor cannot be cleaned up; the results file is tracked and rewritten by every run |

**Found sound.**
- Every injected failure in every parallel stage was caught.
- Results read back by position are attributed correctly.
- The Lean stages are stricter than before.
- The test split (P3) is equivalent evidence. The modified case now has a control it lacked.
- CI always runs the gate on push and pull request.
- The archived files are re-hashed on every run.
- The measurements add up.

**Re-audit of `974a98a`** (same auditor):
- S1–S4, S6 and notes 1, 3, 4, 5, 10 and 11 are confirmed fixed.
- Two further issues were found and fixed in the next commit:

| Finding | Fix |
|---|---|
| N1 (S5 only partly fixed) a cache could add a `refs/replace` ref, which `git fsck` accepts and `git show` follows | every repository restored from a cache keeps only its objects; config, hooks, refs, grafts and alternates are rebuilt on every run. `check_excerpts.py` and the vendored-block comparison read with `--no-replace-objects`. Re-tested: a replace ref planted in a cached repository is removed, and the original text is read |
| N2 under `set -m` the checks ran as a background job on a terminal and stopped there with `tostop` (outside CI) | the child reads `/dev/null` and writes through a relay in the supervisor's foreground group. Re-tested under a pty with `stty tostop`: runs, and TERM stops it cleanly |
| N3, N4 (notes) | documented in the supervisor's header: signals already ignored by the starting shell, Ctrl-Z, and the microsecond window before the child's pid is known |

**Pre-merge confirmation of `b52fbf8`** (same auditor):
- N2, the signal cleanup (including a real Ctrl-C through a pty with `tostop`) and the rebuild of restored repositories are confirmed. No objects were lost, and shallow and partial clones still read every excerpt offline.
- One further should-fix was found: a cached checkout whose `.git` was a *file* pointing elsewhere skipped the rebuild, so a cached `core.fsmonitor` could run a program during `git fsck`. It requires write access to the cache.

  **Fix:** a checkout is used only when it is a real directory with a real `.git` and a real `objects` directory; anything else (a `.git` file, or a symlink at any level) is discarded and fetched again. Every git call also disables fsmonitor and hooks.

  **Re-tested:** the `.git`-file attack (no program runs; the checkout is re-fetched), and symlinked `.git`, checkout and `objects` directories.

**Confirmation of `9c7798d`:**
- The auditor reports nothing blocking.
- Its two remaining notes both concerned an alternates file, carried in through a symlinked `objects/info` or a planted object store. Both already failed safe through `git fsck --full`.
- They are tidied in the next commit: `objects/info` is always replaced by an empty directory before any cached objects are adopted.
- Self-tested; the auditor has not yet re-checked this tidy-up.

**Final targeted audit of `9b3ae7c`** (a new, independent auditor, cached Git checkouts only):
- **No command execution.** No way was found for an untrusted cached checkout to make Git execute attacker commands in the CI flow.
- **Tested and sound:**
  - about 20 executable-config vectors;
  - `.git` as a file, symlink or FIFO;
  - alternates and external stores;
  - replace refs, grafts and shallow files;
  - swapped packs and a tampered multi-pack-index.
- **Three should-fix problems and three notes,** none affecting integrity, all addressed in the next commit:

| Finding | Fix |
|---|---|
| 1. The archive download wrote through a planted `.part` symlink, so the pinned bytes could land in any writable file | `archived` must be a plain directory. A present file is kept only as a regular file equal to its SHA-256 pin, and otherwise removed. Downloads go to a fresh `mkstemp` file and are renamed. Re-tested: the victim file is untouched |
| 2. Symlinks and FIFOs inside the adopted `objects` directory survived: packs could be written outside the cache, and a FIFO hung `fsck` | Nothing cached is moved any more. Only regular files at loose-object or pack paths are copied, each opened with `O_NOFOLLOW` and `O_NONBLOCK` and checked with `fstat`. Every git call has a time limit, and the CI job a 90-minute limit. Re-tested: a symlinked `pack/` and a FIFO pack index are both left behind |
| 3. Abbreviated registry SHAs in `check_excerpts.py` could be made ambiguous by a planted genuine object | resolved through `sources/commit-pins.tsv`; readers use full SHAs only |
| Note 4. The readers were safe only after `fetch-sources.py` had run | a new `scripts/pinned_read.py` is used by `check_excerpts.py` and by verify.sh's vendored-block comparison (details below) |
| Note 5. Unchecked `git init`, `remote` and `config` return codes, and Git's parent-directory discovery | return codes are checked; `GIT_CEILING_DIRECTORIES`; no system or global config; `GIT_*` variables are not inherited |
| Note 6. `actions/cache` restores with absolute paths, so whoever writes an entry can write files | the gate only restores (`actions/cache/restore`); entries are saved only by the source-provenance job on `main`, from an empty directory, after every excerpt is checked |

**How `scripts/pinned_read.py` reads a file.** It uses a fresh temporary repository whose alternates point at the checkout's object store. None of the checkout's config, hooks, refs, grafts or attributes is read. Lazy fetch, replace objects, and system and global config are all off. Every object on the path commit → tree → blob is re-hashed and must equal its name. It is safe on any checkout, whether `fetch-sources.py` has run or not. `fetch-sources.py` now materializes every file the readers need, so they never fetch.

The fixes were then re-tested:
- the interrupts and supervisor cases in §1;
- a partial-backup restore (nothing deleted);
- a poisoned cache object (refused);
- a complete `verify.sh` run (§3).

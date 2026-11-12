# RLT-007 — Architecture review: the complete Lemma 12.1, and verification turnaround

**Status:** documentation-only review, 2026-11-12, against main at e5850d9.

- The kernel, the inference rules and the seven axioms of Robinson's Q are unchanged.
- No new axioms are proposed.
- The measurements below come from untrusted scratch generators outside the repository. Every measured proof was checked by the current `rlt` binary.

Part A asks whether RLT can establish the rest of Lemma 12.1 (mathematical extensions). Part B asks how to keep complete verification reproducible while making development and CI faster (performance). The two are independent. A ranked plan follows in Part C.

---

## Part A — Can RLT establish the remaining obligations of Lemma 12.1?

Lemma 12.1 needs four obligations. RLT-006 proved (b).

| Obligation | Within Robinson's Q, without induction? | Evidence |
|---|---|---|
| (a) 45 distinct elements, minimum 0, maximum 212 | **Yes.** Every fact is closed: 990 pairwise disequalities between numerals, h ≤ 212 for the listed h, and 0 and 212 listed | Same technique as M3c and M3d (closed numerals, Q1 and Q2). Not measured; small |
| (b) the 14 tabled omissions | **Done** (RLT-006) | 630 certificates and 14 per-prime theorems |
| (c) the primes in a range are exactly the listed ones | **Yes,** with primality written inline (no new definition): 1 < p ∧ ∀d. ((0 < d ∧ d ∣ p) → d = 1 ∨ d = p) | An M3d-style enumeration below the bound. Each non-prime is refuted by a divisor witness, as in RLT-005's divisor proofs. Not measured |
| (d) every prime p ≥ 47 has an omitted class | **Yes, by a different proof from the paper's** (below) | Partly measured |

### A.1 The large-prime argument, in two finite parts and one uniform part

The paper's counting argument ("45 elements cannot cover p classes") is a pigeonhole principle over an arbitrary modulus. RLT has no such principle without induction. The same conclusion follows from three parts, each provable in Robinson's Q. This is lean-bgp212's route:

1. **Every p ≥ 213, uniformly — measured.** The claim is ∀p. 212 < p → ¬(∃k. h = (p × k) + 1), one theorem for each listed h. All 45 are accepted. The proof has two cases:
   - k = 0 gives h = 1, which is impossible for a listed h.
   - k = S(k′) gives h = S^214(p·k′ + z), where 212 < p unfolds to ∃z. S(z) + 212 = p. Peeling h ≤ 212 successors contradicts Q2.

   It uses Q1, Q2, Q3, add_zero, add_succ, mul_zero and mul_succ, all at numerals or variables. **45 of 45 accepted; 69,445 steps; 52 MB; 43 s.**
2. **Primes 47 … 211 (33 primes) — measured.** These use RLT-006's certificate scheme, with one omitted class per prime (1, 3, 4 or 6):
   - 1,485 certificates; 1,423 accepted;
   - 1.09 million steps for the accepted ones; 423 MB; 814 s;
   - **62 certificates exceed the 1,000,000-term-node limit** (up to 1.40 million), all for large p and large h. RLT-006's scheme peels p successors per layer, and its terms grow with p·h.

   The tail proof's direct case split (k = 0 or k = S(k′), then peel) keeps terms near 214 + h nodes per step. Re-engineering the certificate scheme along those lines is expected to bring every certificate under the limit without changing the kernel. That is a development item for RLT-007.
3. **The primes below 213 are exactly the 46 listed** (the 14 primes up to 43 plus the 33 primes from 47 to 211). This is an enumeration below 213 with the 166 non-primes refuted by divisor witnesses (0 and 1 by 1 < p). The enumeration below 30 costs 971 steps. Below 213 is estimated at about 10⁴–10⁵ steps, in several files. Not measured.

Together with (b), parts 1–3 give admissibility for every prime, inside RLT and from Q. A record would mark this "same statement, different proof".

### A.2 What cannot be a single kernel theorem today

A theorem that proves "H₄₅ is admissible" by *citing* all its certificates would check about 2,200 files (630 + 1,485 + 45 + the primality files). The checker allows **64 cited-file checks per check**: a resource limit, unchanged by request. No layout avoids this. Inlining is bounded by the 1,000,000-node file limit, which already needs about 70 files for (b) alone.

Without changing that limit, the complete lemma can be established in RLT as follows:

- **The components are theorems.** Each component is an RLT theorem within the limits:
  - the 14 + 33 per-prime summaries (45 checks each);
  - a tail summary citing the 45 tail certificates (45 checks);
  - the primality enumeration;
  - (a).
- **The assembly is a conditional theorem.** A kernel-checked theorem has the ~50 component statements as its assumptions Γ and proves admissibility for every prime, plus (a). The case split on p, the choice of r and the use of each premise are all derived by the kernel.
- **Discharge.** Each assumption in Γ equals, by statement fingerprint, the statement of an accepted, citable library entry. `rlt deps` could report this as a new reporting-layer check: "every assumption is discharged by a library theorem". Two things would remain outside the kernel: the fingerprint match, and "every premise holds, therefore the conclusion holds", which is modus ponens across files.

**Decision for review (A-D1).** Accept assembly by discharge, outside the kernel, with its trust note. Alternatively, review the 64-check limit as a separate kernel-resource proposal. RLT-007 would *not* change it. This review recommends discharge first: every mathematical step stays kernel-checked, and only the joining of files is reported.

### A.3 Proposed mathematical scope for RLT-007

- **A1.** Obligation (a).
- **A2.** The tail: 45 certificates and one summary.
- **A3.** A re-engineered certificate scheme. Then the 33 per-prime theorems for 47 … 211, each citing 45 certificates.
- **A4.** Primality below 213, by enumeration and divisor witnesses.
- **A5.** The conditional assembly theorem, and the discharge report (if A-D1 is approved).

Physical correspondence stays not applicable. The semantic status could then move from `partial` to "matches, different proof for (d)", subject to review. The Lean comparisons extend naturally: lean-bgp212 already follows this route.

---

## Part B — Reproducible complete verification with faster turnaround

### B.1 Where the time goes (measured)

| | Before RLT-006 (main 9c8a66d) | After RLT-006 |
|---|---|---|
| CI `check` job | about 4 min | about 21 min |
| `cargo test --release` (local, no certificates present) | not measured | 826 s |
| `scripts/verify.sh` (local, with the reference comparison) | not measured | 1,694 s |

The cost comes from re-checking the same certificates several times, one at a time:

- verify.sh checks all 630 certificates directly. It then re-checks them through:
  - 14 `rlt deps` runs;
  - the library verifier;
  - `rlt explain`, text and JSON, for 14 records;
  - the record fixtures;
  - `cargo test`, which itself runs the library verifier three times.
- Every `rlt` process is single-threaded, and each certificate check is about 0.1–0.4 s.

A.1's further 1,530 certificates would multiply this unless B is in place.

### B.2 Proposals (none touches the kernel or the axioms)

| Id | Change | Reproducibility | Expected effect |
|---|---|---|---|
| B1 | **Memoize citations within one process.** A cited file already checked in this run, with the same path, SHA-256, axiom-set pin and spec pins, is not re-checked. The citation count reported against the 64-check budget stays as it is now | Identical results: the check is a pure function of those inputs | `--verify-lib`, `explain` and `deps` over several roots check each certificate once per process |
| B2 | **`rlt check` over many files in parallel** (for example `rlt check --jobs N FILE…`), with reports in input order | Same per-file output, in deterministic order | 630 certificates: about 85 s → about 25 s on CI's 4 vCPU |
| B3 | **One complete pass per run.** verify.sh checks each certificate once and reuses the result (B1 and B2). Rust tests keep their refusal cases on a small scratch tree, one prime, plus one complete library verification | Unchanged: the complete pass still checks everything | Removes most of the repeated work |
| B4 | **Tiered CI.** Pull requests run a fast tier: format, lints, unit tests, generator reproduction, the manifest, and the complete check of the 14 summaries (which check their certificates). The full tier (verify.sh complete, with the reference comparison) is required on the merge commit and runs nightly. Results are recorded with the commit hash | The full tier is unchanged and mandatory before a merge is recorded | Pull-request feedback well under 10 min |
| B5 | **Generate, check and delete in batches** for large certificate families (A3's 423 MB), keeping the manifest | Unchanged | Bounded disk |
| B6 | **More efficient certificates** (A3). This is mathematics engineering within the kernel | New certificates, reviewed like any proof | Smaller files, fewer nodes, faster checks |

### B.3 Gate hardening carried over from the final RLT-006 audit

None of these affects what is proved. Each tightens a check or the CI's inputs:

- **H1 — residue-profile arithmetic check.** Today it accepts some altered tables; the regeneration stage catches them. Add four checks:
  - omitted classes equal the complement of the occupied set;
  - the h values for each p equal the transcribed tuple, 45 rows per prime;
  - each row's r_p matches the profile;
  - no duplicate rows.
- **H2 — interrupts in verify.sh.** On INT or TERM, clean up and then exit (130 or 143), so an interrupted run never reaches `RESULT`.
- **H3 — fixed CI inputs.**
  - Mirror the pinned PDF at a fixed location: the CI already fails closed if the original is replaced.
  - Fetch lean-bgp212 commit 7898eb4 explicitly, rather than its default branch.
- **H4 — report count.** `docs/RLT-006-REPORT.md` §8 should say 30 `grep_q` pipes, not 29.

**Not proposed:** a persistent verification cache across runs. A cache keyed by (binary hash, spec hashes, file hash) could serve local development only. It would be a new trust input, so the full tier would never use one.

---

## Part C — Recommended order

1. **H1–H4 and B1–B3 first** (hardening and performance, small, no new mathematics). They make A affordable and bring CI back toward its earlier turnaround.
2. **A1, A2 and A4** (cheap; A2 is already prototyped).
3. **A3** (the certificate scheme), then the 33 primes.
4. **A5,** after decision A-D1.

## Part D — The author-facing bundle

**Recommendation:** prepare it after A5 is merged, when RLT establishes every obligation of Lemma 12.1. At that point it can say "RLT checks the complete lemma from the seven axioms of Robinson's Q", with the different proof of (d) stated plainly.

If the authors would benefit sooner, an interim bundle could follow B1–B3. It would cover obligation (b) and the residue profile, clearly labelled partial. It would be packaged from a tagged commit, and its hash recorded in the handoff.

Contacting the authors remains your decision.

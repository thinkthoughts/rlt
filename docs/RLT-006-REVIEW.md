# RLT-006 — Feasibility review: H₄₅ residue certificates (Axiom Math, Lemma 12.1)

**Status:** documentation-only review, 2026-11-11, against main at 9c8a66d.

- Nothing is implemented in the repository.
- The kernel, inference rules and axiom sets are unchanged.
- No induction, new axioms or new rules are proposed.
- The measurements below come from an untrusted scratch generator run outside the repository. It reuses the M3c non-divisibility scheme from `scripts/gen-m3c-tests.py`, and every measured proof was checked by the current `rlt` binary.

**Recommendation.**

1. RLT can check all 14 small-prime omissions. Each is checked individually, element by element, as a kernel theorem under Robinson's Q, within current limits:
   - 630 element certificates: all accepted, 551,016 steps, 54 s;
   - 14 per-prime theorems: each cites its 45 element certificates, 45 of 64 cited-file checks; all accepted, 58 s.
2. One theorem covering all 14 primes is outside the current citation budget under every layout measured (§4).
3. A plain-text residue table makes the certificates independently reproducible without RLT (§5).

The value for the authors is real but modest. It is an independent, small-kernel cross-check of the printed table, pinned to the PDF's hash, plus a reusable residue profile. The value for RLT is larger: a real paper's finite certificate in 644 proof files, about 34 times RLT-005's 19. §8 assesses this.

---

## 1. Source

| Item | Value |
|---|---|
| Paper | F. Charton, L. Hong, K. Lau, K. Ono, G. Remy, H. C. Siu, A. A. Swaminathan, J. Thorner, Y. Xie (Axiom Math), *A New Bound for Small Gaps Between Primes* |
| Version | "Preliminary Draft: September 3, 2026" (title page). There is no version number, and I found no arXiv version (MathWorld cites the same URL and date) |
| Location | https://primegaps.axiommath.ai/bgp212.pdf, fetched 2026-11-11 |
| Pin | SHA-256 `2b307ae26046dffd2153fe7466e5a9a302976dc3ba5142151a8a8104a37b3d80`, 734,121 bytes, 46 pages, PDF 1.5. PDF metadata: CreationDate Thu Sep 3 09:08:29 2026 MDT, Producer pdfTeX-1.40.29 |
| Definition | §2 (Notation): a finite set H = {h₁, …, h_k} of distinct integers is *admissible* if, for every prime p, the elements of H omit at least one residue class modulo p |
| Statement | §12, Lemma 12.1: "The set H45 is an admissible 45-tuple of diameter 212." |
| Data | §12 displays H₄₅ (45 elements, 0 … 212). The proof of Lemma 12.1 lists the primes 2, 3, 5, …, 43 and, "respectively", the omitted classes 1, 1, 3, 6, 9, 3, 1, 13, 17, 4, 11, 4, 4, 3. For p ≥ 47 it argues that 45 elements cannot cover p classes |
| Role in the paper | DHL[45, 2] (Theorems 4.10, 7.11 and 11.1) is applied to H₄₅ to give H₁ ≤ 212 (proof of Theorem 1.1) |

Appendix A says AxiomProver generated a Lean certificate of the deduction of Theorem 1.1. Its stated hypotheses are analytic inputs plus the variational certificate. The appendix does not say how Lemma 12.1 is handled, so this review assumes nothing about it.

**Consistency of the transcription** (untrusted Python, consistency only). The text was extracted with `pdftotext -layout` and compared with the transcription in docs/RLT-005-REVIEW.md: identical. The list has 45 distinct elements, minimum 0 and maximum 212. Each tabled class r_p is omitted by H₄₅ mod p.

For five primes, the table names one of several omitted classes, which is enough for the lemma:

| p | 29 | 31 | 37 | 41 | 43 |
|---|---|---|---|---|---|
| Omitted classes | 4, 13 | 11, 28 | 4, 20, 31 | 4, 8, 23, 29, 35, 37, 38 | 3, 9, 18, 19, 25, 39, 42 |

So 30 classes are omitted in all: the 14 tabled ones and 16 more.

## 2. Formal statements

**Element certificate** O(p, h). For each listed p with tabled class r = r_p, and each h ∈ H₄₅:

```
¬∃k. h = (p × k) + r                        NAT-DEF, ROBINSON-Q, Γ = ∅
```

**Per-prime omission** OM(p): the left-nested conjunction of the 45 element certificates, read as "the reductions of H₄₅ mod p omit the class r_p".

**How the formula is proved.**
- From h = p·k + r, add_succ gives h = S^r(p·k).
- If h ≥ r: peeling r successors (Q1) gives h − r = p·k. The M3c case tree (Q3, Q2, mul_succ) then refutes it.
- If h < r: peeling h successors gives 0 = S^(r−h)(p·k), refuted by Q2.

No step quantifies over an arbitrary p, so no induction is involved.

**Correspondence over ℕ.**
- The paper speaks of integers. H₄₅ consists of nonnegative integers and 0 ≤ r_p < p. For such h, "h ≡ r_p (mod p)" holds exactly when some natural k has h = p·k + r_p.
- So OM(p) is the paper's "H₄₅ omits r_p mod p" for each listed p. This is a reviewable step, and a Lean statement can check it (§6).
- The left-nested conjunction stands for "every listed element". A membership form, ∀x. (x = 0 ∨ … ∨ x = 212) → ¬∃k. x = p·k + r, is equivalent. It would cost one 45-way case split per prime and no further proof files.

**Outside these statements.** Lemma 12.1 as a whole claims admissibility for every prime. RLT-006 as reviewed covers the 14 finite omissions only. The p ≥ 47 part and the identification of "the primes up to 43" are discussed in §9. So the semantic assessment of a Lemma 12.1 record would be `partial`, with that difference stated.

## 3. Measured cost of the element certificates

| | Value |
|---|---|
| Files | 630 (14 primes × 45 elements), all ACCEPT, unconditional, ROBINSON-Q, Γ = ∅ |
| Steps | 551,016 in total; largest 3,299 (p = 2, h = 212); smallest 11 |
| Checking time | 53.9 s in total; at most 0.36 s per file (current release binary, this sandbox) |
| Proof text | 59.6 MB in total; largest file 228 KB |
| Citations | none (each certificate stands alone) |

Steps and checking time per prime:

| p | 2 | 3 | 5 | 7 | 11 | 13 | 17 | 19 | 23 | 29 | 31 | 37 | 41 | 43 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Steps (45 files) | 78,455 | 61,091 | 46,935 | 40,788 | 35,577 | 34,779 | 33,311 | 32,215 | 31,595 | 31,446 | 31,245 | 31,230 | 31,050 | 31,299 |
| Seconds | 4.8 | 3.9 | 3.4 | 3.1 | 3.1 | 3.3 | 3.3 | 3.3 | 3.4 | 3.8 | 4.2 | 4.8 | 4.6 | 4.9 |

The cost grows with h and with the number of case-tree layers, about h/p. Small primes cost most because each layer is cheap but there are many layers.

## 4. Citation cost and layout

| Layout | Measured result |
|---|---|
| A. One self-contained file per prime (45 refutations inline) | **Refused.** It would build 3.5–7.3 million term nodes per prime (69.2 million in total) against the 1,000,000-node file limit |
| B. One per-prime theorem citing its 45 element files | **Accepted, all 14.** 89 steps each. `rlt deps`: 46 theorems, 45 of 64 cited-file checks, chain 1 of 8. About 4–5 s per prime, 58 s for all 14 (each re-checks its 45 citations) |
| C. One theorem for all 14 primes, citing the 14 per-prime theorems | **Refused**: "the citations would check more than 64 files" (644 needed) |
| C′. Layout A split into chunks under the node limit | At least 75 chunk files, so a combined theorem needs at least 89 cited-file checks: **still above 64** |

**Conclusion.**
- The 14 per-prime theorems (layout B) are the largest reviewed unit within current limits.
- An all-primes-at-once theorem would need a reviewed change to `MAX_CITED_CHECKS`. That is a checker resource limit, outside the kernel's inference rules, but a trust-relevant parameter, and it is not proposed here.
- The fourteen per-prime theorems together are exactly the 14 claims of the proof, each checked individually, which matches the approval's wording.

**Library.** 630 + 14 = 644 new entries (area `admissible`, for example `admissible.h45.omits.43_3` and `admissible.h45.omits.43_3.at.212`). `lib/identities.tsv` grows from 87 to 731 lines, append-only. Existing dvd entries do not apply, since none is a residue form.

**Repository weight.** 59.6 MB of committed proof text would be about 6 times the current repository (about 10 MB, excluding build output and git history). Two options for review:

- **(i) Commit every file.** As in RLT-005, `scripts/verify.sh` checks that the generator reproduces them.
- **(ii) Commit the generator, the 14 per-prime theorems and a SHA-256 manifest of the 630 element files.** verify.sh and CI regenerate the element files before checking, and a mismatch with the manifest fails the gate. The library's proof pins are unchanged in meaning: each records a path and a hash.

I recommend (ii) for size; (i) is simpler.

## 5. Independently reproducible certificates

Two artifacts, each checkable without trusting the other:

1. **Residue table** (`certificates/h45-residues.tsv`, plain text): one row per (p, h) giving the quotient q and remainder s with h = p·q + s and 0 ≤ s < p, plus the tabled r_p. 630 rows.
   - Anyone can check it with integer arithmetic in any language, in a few lines: recompute p·q + s, check s < p, check s ≠ r_p.
   - The rows for one p show directly which classes H₄₅ occupies. The table therefore also records the full residue profile: occupied-class counts ν_p of 1, 2, 4, 6, 10, 12, 16, 18, 22, 27, 29, 34, 34, 36 for p = 2, …, 43, and all 30 omitted classes.
2. **RLT derivations:** the 630 + 14 proof files, with SHA-256 pins, each checked by `rlt check` from Robinson's Q alone. `rlt deps` shows each per-prime graph, and `rlt explain` links a statement record to the pinned PDF.

The table is data that RLT's proofs agree with; it is not what RLT proves. Each RLT certificate proves non-membership (¬∃k), which the table states as a computed remainder. Their agreement is a consistency check, run by the generator and by verify.sh.

**Optional, positive side of the profile.** Each occupied class can carry a closed witness equation h = p·q + s; for example, 212 = 43 × 4 + 40 takes 666 steps.

- There are 251 occupied (p, class) pairs. At ≤ 1,278 steps each (the largest measured, 212 = 2 × 106 + 0), they add at most 0.33 M steps.
- With them, RLT would prove the exact set of occupied classes mod each p ≤ 43, not only that one class is free.
- That is the quantity behind local factors such as 1 − ν_p/p.

## 6. Lean 4 comparison

- **Tier 1, the same propositions:** each OM(p) restated over `Nat` and decided by `decide`, as in RLT-005's comparison 1.
- **Tier 2, the paper's statement:** the user's repository `thinkthoughts/lean-bgp212` (commit 7898eb4) already contains Lemma 12.1 in core Lean, toolchain v4.34.0-rc2 (the same as RLT's).
  - `BGP212.Basic` defines `IsPrime`, `Admissible` ("for every prime p some r < p with h % p ≠ r for every h ∈ H"), `H45` and `omittedTable`.
  - `Solution/Lemma12_1.lean` proves `h45_omitted_table` (the 14 tabled pairs) and `h45_admissible` for every prime. The finite part is `decide +kernel` below 213. For p ≥ 213 it uses the observation that reduction mod p fixes every element and 1 ∉ H₄₅.
  - **Proposal:** vendor these definitions, with provenance, into one RLT comparison file. Add a bridge theorem: `h % p ≠ r ↔ ¬∃k, h = p * k + r`, for r < p. This relates RLT's OM(p) to `omittedTable` exactly, as RLT005Readings did for matchability. That review re-runs the comparison; this review did not compile lean-bgp212.
- Neither tier proves anything about RLT's derivations, and the semantic status is not raised because they compile.

## 7. Source fidelity for a PDF-only source

The registry's `archived` status (RLT-005) reads a member of a tar or gzip archive. This paper exists only as a PDF, so the pin needs one of:

| Option | What the check does | Trade-off |
|---|---|---|
| **P1, PDF member (recommended)** | `archived` with `url = …/bgp212.pdf#text`. `check_excerpts.py` verifies the SHA-256, then extracts text with a pinned `pdftotext` version and matches whitespace-collapsed excerpts at word boundaries | Adds poppler-utils to the excerpt check, which lies outside the kernel and is already untrusted. Subscripts flatten (H₄₅ becomes "H45"), so excerpts are written as extracted |
| P2, reviewed transcription | The excerpts are recorded with page numbers, and a review note attests them. Only the hash is checked mechanically | No new tool, but fidelity rests on review alone |

With P1 the excerpts would be the §2 definition sentence, the H₄₅ display, the Lemma 12.1 statement and the two lists from its proof. All were located in the extracted text for this review (lines 131–132 and 2981–2989 of the `-layout` output).

## 8. Would this help the authors?

**Verification.** Lemma 12.1 is the most elementary input to Theorem 1.1. A one-line `decide` already checks it, as does the user's lean-bgp212. RLT would add a second, independent derivation from seven explicit axioms, with no shared code, numerals or automation, pinned to the exact PDF bytes. That diversity is the contribution. It confirms the *printed* table, which matters if the draft is revised: a changed list would change the PDF hash, and the record would turn stale.

**Reuse.**
- The residue table (§5) gives every element's residue mod every p ≤ 43 and the exact occupied-class counts ν_p. These are the inputs to singular-series and local-factor computations, which lean-bgp212 relates to admissibility (`admissible_iff_local_factors`).
- The table is plain data, independent of RLT and Lean, and can be reused for neighbouring tuples (for example, Stadlmann's admissible 49-tuple of diameter 240) by changing the list.

**For the authors directly, then:** an auditable cross-check and a reusable certificate table, both small. **For RLT:**
- the first certificate from an active research paper;
- 644 proof files, about 34 times RLT-005's 19 (551,016 steps against 1,271);
- a PDF source pin;
- a stress test of the library and the citation budget at 644 entries.

## 9. Beyond the 14 omissions (for a later decision)

1. **"These are the primes up to 43."** This is expressible inline, with no new definition: ∀p. (p < 44 ∧ 1 < p ∧ ∀d. ((0 < d ∧ d ∣ p) → d = 1 ∨ d = p)) → p ∈ {2, …, 43}.
   - The proof would combine an M3d-style enumeration below 44 with one divisor witness for each of the 28 non-primes below 44.
   - Not measured; by the M3d and M3c scheme sizes, roughly 10⁴–10⁵ steps.
2. **Moduli 47 … 211.** These are finite, using the same certificates. 34 primes × 45 elements, each about 1,200–1,300 steps by the measured trend for p ≥ 23, gives about 2 M steps and 3 minutes. Not measured.
3. **Moduli p ≥ 213, for every p (the lean-bgp212 route).** Claim: ∀p. 213 ≤ p → ∀ listed h. ¬∃k. h = p·k + 1.
   - Sketch:
     - Write p = S^213(z). Case k = 0 gives h = 1, refuted because h ≠ 1.
     - Case k = S(k′) gives h = S^214(p·k′ + z), by mul_succ and add_succ applied 213 times. Peeling h ≤ 212 successors then contradicts Q2.
   - Every step instantiates the axioms at numerals or variables, with no induction.
   - Not measured; estimated at about 45 × 1,000 steps.
   - Together with items 1 and 2, RLT could then prove admissibility for every modulus p ≥ 2 without a primality predicate. That route differs from the paper's counting argument for p ≥ 47, so a record would say "same statement, different proof".
4. **The paper's counting argument for p ≥ 47** (45 elements cannot cover p classes, for arbitrary p) needs a pigeonhole argument over an arbitrary finite set. That is outside RLT without induction.

## 10. Decisions for approval

1. **Scope:** the 14 per-prime theorems (layout B), with or without the positive residue profile (§5).
2. **Repository weight:** commit all element files (i), or a generator plus manifest (ii).
3. **PDF pinning:** P1 (pdftotext member) or P2 (reviewed transcription).
4. **Library naming:** area `admissible`, with ids as in §4.
5. **Lean comparison:** tier 1 plus the vendored lean-bgp212 tier 2 with the residue bridge.
6. **Later items from §9:** whether any belong in RLT-006 or a later milestone.

Nothing is implemented until approval. No release or deployment.

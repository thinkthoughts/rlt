# RLT-007c — Intermediate primes 47 … 211: measured findings

**Status:** study only, 2026-11-12. Nothing here is committed as a proof.
- The certificates were written by an untrusted scratch generator outside the repository.
- Every one was checked by the current `rlt` binary at `9ae34c5`, with the existing resource limits: 1,000,000 term nodes per file, 64 cited-file checks, citation chain 8.
- The kernel, the inference rules and the axioms are unchanged.

## 1. The problem

Obligation (d) of Lemma 12.1 asks, for each of the 33 primes p from 47 to 211, that H₄₅ omits a residue class r_p. The classes are 1, 3, 4 or 6, the same choice as lean-bgp212.

Each element certificate states, for one listed h:

> ¬(∃k. h = (p × k) + r_p)

RLT-006's scheme, applied to these 33 primes, gives 1,485 certificates. **62 of them exceed the 1,000,000-node file limit,** reaching up to 1,404,424 nodes (`O_199_212`). All 62 are for p from 163 to 199 with large h:

| p | 163 | 167 | 173 | 179 | 181 | 191 | 193 | 197 | 199 |
|---|---|---|---|---|---|---|---|---|---|
| certificates over the limit | 6 | 10 | 11 | 8 | 8 | 5 | 5 | 5 | 4 |

**Why the old scheme is too large.** At every case split k = S(k′), the old scheme expands p × S(k′) into S^p(p × k′) with p add_succ steps. It then peels successors off terms that still contain p × k′. Every formula in those chains repeats the numeral p (p + 1 nodes) and the growing S-chain, so one layer costs roughly p² + p·h nodes.

## 2. The transformation tried: a direct scheme with a per-layer lemma

The idea comes from the uniform tail certificate (p ≥ 213). Split k with Q3 and keep the long chains away from p × k′.

1. Peel the r_p successors of (p × k) + r_p, leaving n = p × k with n = h − r_p.
2. Case-split k with Q3, at most ⌊n/p⌋ + 1 times:
   - **k = 0:** p × 0 = 0 by mul_zero, so n = 0. Since h ≢ r_p, n > 0, and Q2 refutes it.
   - **k = S(k′):** mul_succ gives n = (p × k′) + p. A lemma is proved once per layer over a fresh parameter w:

     ∀w. (n = w + p) → (n − p = w)   when n ≥ p,
     ∀w. ¬(n = w + p)        when n < p.

     The lemma peels min(n, p) successors with add_succ and Q1 on the one-node variable w. It is instantiated once, by forall-elim, at p × k′. When n ≥ p, the next layer starts from n − p = p × k′.
3. exists-elim on k, then imp-intro, gives the certificate.

It uses only Q1, Q2, Q3, add_zero, add_succ, mul_zero and mul_succ, with the same rules as RLT-006: hyp, refl, symm, trans, cong, subst, inst, imp-intro/elim, forall-intro/elim, exists-elim and or-cases. There is no induction and no new definition. The goal of each certificate is exactly the RLT-006 form.

## 3. Results (measured)

| | RLT-006 scheme (33 primes) | Direct scheme (33 primes) |
|---|---|---|
| Certificates accepted | 1,423 of 1,485 (**62 over the node limit**) | **1,485 of 1,485** |
| Largest file | 1,404,424 nodes | **324,133 nodes** (`D_199_212`; 32% of the limit) |
| Mean file | 453,219 nodes | 127,851 nodes |
| Total nodes | 673 million | 190 million |
| Proof steps | 1.09 million (accepted files only) | 729,571 (max 967 per file; median 495) |
| Disk | 423 MB | **51 MB** |
| Check time, all files | 814 s (1 process) | 217 s wall on 2 processes, while another job was running |

**The 62 formerly oversize certificates:**
- 62 of 62 accepted;
- 53,251 steps in total (at most 904 per file);
- 3.7 MB;
- 20 s in one process.

**Controls.**
- A certificate edited so that its goal no longer matches the case it eliminates (r changed in the goal) is rejected at the exists-elim step.
- The generator refuses to write a certificate for an occupied class (h ≡ r mod p). That case makes the final layer's equation 0 = p × k′, which is consistent.
- A node counter written for this study reproduces the checker's count: it finds exactly the same 62 files over the limit.

**Unresolved certificates: none.** Under the existing limits, every element certificate for the 33 intermediate primes fits, with at least 3× headroom.

## 4. The uniform approach below 213

The tail's uniform statement, ∀p. 212 < p → ¬(∃k. h = (p × k) + 1), covers p ≥ 213 because one class (r = 1) works for every such p. It has no direct analogue below 213, for two reasons:
- the omitted class depends on p (1, 3, 4 or 6);
- a uniform statement over 47 ≤ p ≤ 211 would still need the case split on p, which is the primality enumeration of RLT-007b.

The useful transfer is the tail's *technique*, not its statement:
- case-split k with Q3;
- keep the chains on a variable;
- instantiate once.

That is the direct scheme above.

## 5. Consequences for the proof architecture

- **Per-prime theorems.** Each of the 33 primes gets a summary citing its 45 direct certificates: 45 of 64 cited-file checks, chain 1 of 8, the same shape as RLT-006.
- **Cost of adding them to the complete run.** About 51 MB generated, and about 430 CPU-seconds per complete pass (RLT-006's family takes about 80 s). Under RLT-007a's scheduling, with one direct pass plus one pass through the per-prime summaries, this adds roughly 4–8 minutes wall time on a 4-processor runner.
- **RLT-006's 630 certificates.** The direct scheme would also shrink them. They are kept byte-for-byte; replacing them would be a separate, reviewed change.
- **Size of a complete single proof.** About 2,200 component files, about 300 million nodes in all, 190 million of them for this family. That rules out one kernel-checked file, and one citing root under the 64-check budget. See RLT-007d.

## 6. Recommendation

Adopt the direct scheme for the 33 intermediate primes in RLT-007b's implementation. It needs no change to the kernel, the rules, the axioms or the limits. The generator becomes a reviewed script in the repository, like `scripts/gen-rlt006.py`, with a manifest and tamper tests.

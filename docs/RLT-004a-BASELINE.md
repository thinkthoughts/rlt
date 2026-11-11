# RLT-004a — Baseline measurements of the unary representation

**Status.** RLT-004a deliverable, measured 2026-11-11 on main at cc01968 (release build). Measurement only: no axiom, definition, rule, limit or record was added or changed for these measurements (decision V6 as modified at approval). Cases that exceed a current limit are recorded with RLT's own message.

## What is measured

Each size is reported separately:

| Quantity | Meaning |
|---|---|
| written characters | length of the statement as a person writes it |
| kernel term size | nodes of the numerals in the kernel: the numeral n is S applied n times to 0, n + 1 nodes |
| steps | proof steps reported by `rlt check` |
| proof nodes | the file's node budget, computed as `syntax/src/proof.rs` computes it (an integer n counts n + 1); the limit is 1,000,000 |
| bytes | proof file size |
| time | wall-clock time of one `rlt check` run, including process start (single run; indicative only) |

Proofs come only from existing mechanisms: `rlt gen` for equations (PEANO-ARITH), and the M3c non-divisibility and M3d enumeration schemes (ROBINSON-Q), whose generator functions `scripts/measure-baseline.py` loads verbatim from `scripts/gen-m3c-tests.py` and `scripts/gen-m3d-tests.py`. The script's node count agrees exactly with the count RLT prints when it refuses a file (e.g. 3,553,369 for 1000 = 3 + 997).

Machine: 2 vCPU Intel Xeon @ 2.10 GHz, Linux 6.18, rustc 1.97.0.

## Written notation and kernel term size of numerals

| n | written characters | kernel nodes (S^n(0)) |
| --- | --- | --- |
| 7 | 1 | 8 |
| 30 | 2 | 31 |
| 100 | 3 | 101 |
| 1000 | 4 | 1001 |
| 10000 | 5 | 10001 |

## Equation certificates n = p + q (`rlt gen`, PEANO-ARITH)

| statement | written chars | numeral nodes | steps | proof nodes | bytes | time | result |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 10 = 3 + 7 | 10 | 23 | 23 | 754 | 1,329 | 0.00 s | exit 0: ACCEPT — unconditional theorem relative to registered axiom set PEANO-ARITH |
| 30 = 7 + 23 | 11 | 63 | 71 | 4,290 | 3,726 | 0.01 s | exit 0: ACCEPT — unconditional theorem relative to registered axiom set PEANO-ARITH |
| 100 = 3 + 97 | 12 | 203 | 293 | 40,219 | 15,210 | 0.05 s | exit 0: ACCEPT — unconditional theorem relative to registered axiom set PEANO-ARITH |
| 300 = 7 + 293 | 13 | 603 | 881 | 330,585 | 47,163 | 0.68 s | exit 0: ACCEPT — unconditional theorem relative to registered axiom set PEANO-ARITH |
| 400 = 3 + 397 | 13 | 803 | 1193 | 581,269 | 64,486 | 0.52 s | exit 0: ACCEPT — unconditional theorem relative to registered axiom set PEANO-ARITH |
| 500 = 13 + 487 | 14 | 1003 | 1463 | 900,574 | 82,168 | 0.77 s | exit 0: ACCEPT — unconditional theorem relative to registered axiom set PEANO-ARITH |
| 1000 = 3 + 997 | 14 | 2003 | — | 3,553,369 | 165,891 | 0.01 s | exit 2: MALFORMED — proof file would build 3553369 term nodes, above the limit 1000000 |
| 10000 = 59 + 9941 | 17 | 20003 | — | — | — | — | `rlt gen` exit 1: rlt gen: NAT-DEC: numeral 10000 exceeds the RLT-001 unary limit 1000 |

## Non-divisibility certificates ¬(d ∣ n) (M3c scheme, ROBINSON-Q)

| statement | layers | steps | proof nodes | bytes | time | result |
| --- | --- | --- | --- | --- | --- | --- |
| ¬(2 ∣ 7) | 4 | 128 | 2,911 | 7,932 | 0.02 s | exit 0: ACCEPT — unconditional theorem relative to registered axiom set ROBINSON-Q |
| ¬(2 ∣ 29) | 15 | 469 | 12,910 | 29,583 | 0.04 s | exit 0: ACCEPT — unconditional theorem relative to registered axiom set ROBINSON-Q |
| ¬(3 ∣ 97) | 33 | 1190 | 57,121 | 81,148 | 0.12 s | exit 0: ACCEPT — unconditional theorem relative to registered axiom set ROBINSON-Q |
| ¬(2 ∣ 97) | 49 | 1523 | 63,706 | 98,569 | 0.11 s | exit 0: ACCEPT — unconditional theorem relative to registered axiom set ROBINSON-Q |
| ¬(2 ∣ 197) | 99 | 3073 | 193,006 | 200,658 | 0.40 s | exit 0: ACCEPT — unconditional theorem relative to registered axiom set ROBINSON-Q |
| ¬(2 ∣ 299) | 150 | 4654 | 391,855 | 310,583 | 0.72 s | exit 0: ACCEPT — unconditional theorem relative to registered axiom set ROBINSON-Q |
| ¬(2 ∣ 397) | 199 | 6173 | 646,606 | 416,276 | 1.31 s | exit 0: ACCEPT — unconditional theorem relative to registered axiom set ROBINSON-Q |
| ¬(31 ∣ 997) | 33 | — | 2,844,019 | 845,999 | 0.09 s | exit 2: MALFORMED — proof file would build 2844019 term nodes, above the limit 1000000 |
| ¬(7 ∣ 997) | 143 | — | 2,672,782 | 669,725 | 0.07 s | exit 2: MALFORMED — proof file would build 2672782 term nodes, above the limit 1000000 |
| ¬(2 ∣ 997) | 499 | — | 3,567,406 | 1,063,376 | 0.07 s | exit 2: MALFORMED — proof file would build 3567406 term nodes, above the limit 1000000 |

## Enumeration ∀x. x < B → x = 0 ∨ … ∨ x = B − 1 (M3d scheme, ROBINSON-Q)

| bound B | steps | proof nodes | bytes | time | result |
| --- | --- | --- | --- | --- | --- |
| 10 | 241 | 10,235 | 20,795 | 0.02 s | exit 0: ACCEPT — unconditional theorem relative to registered axiom set ROBINSON-Q |
| 20 | 556 | 60,725 | 78,468 | 0.06 s | exit 0: ACCEPT — unconditional theorem relative to registered axiom set ROBINSON-Q |
| 30 | 971 | 215,390 | 202,043 | 0.20 s | exit 0: ACCEPT — unconditional theorem relative to registered axiom set ROBINSON-Q |
| 40 | 1486 | 567,730 | 419,518 | 0.54 s | exit 0: ACCEPT — unconditional theorem relative to registered axiom set ROBINSON-Q |
| 47 | 1906 | 996,914 | 642,615 | 1.15 s | exit 0: ACCEPT — unconditional theorem relative to registered axiom set ROBINSON-Q |
| 48 | — | 1,073,912 | 679,922 | 0.07 s | exit 2: MALFORMED — proof file would build 1073912 term nodes, above the limit 1000000 |
| 60 | — | 2,389,435 | 1,248,168 | 0.16 s | exit 2: MALFORMED — proof file would build 2389435 term nodes, above the limit 1000000 |

## Findings

1. **Written size grows with the number of digits; kernel term size grows with the value.** 1000 is 4 characters and 1,001 kernel nodes.
2. **Certificate size is roughly quadratic in the numbers involved.** Each step restates terms containing unary numerals, so proof nodes grow about as (steps) × (numeral size). Doubling n in ¬(2 ∣ n) from 197 to 397 multiplies steps by 2.0 and nodes by 3.4.
3. **The 1,000,000-node file limit is reached at:**
   - enumeration below 48 (below 47 is accepted, 996,914 nodes);
   - equations n = p + q between 500 (accepted, 900,574 nodes) and 1000 (refused);
   - ¬(2 ∣ n) between n = 397 (accepted) and n = 997 (refused); ¬(d ∣ 997) is refused for d = 2, 7 and 31.
4. **`rlt gen` accepts numerals up to 1000** (the RLT-001 unary limit); 10000 is refused by the generator itself.
5. **Checking time stays near or below one second** for every accepted case; refused cases stop at the size check before any checking.

## Consequences recorded as limitations

- A closed primality certificate for a three-digit prime near 1000 (every ¬(d ∣ p) for d ≤ √p) exceeds the current limit for the small divisors, before any question of quantification.
- A *quantified* primality or residue statement for n above 47 cannot use the M3d enumeration scheme within the limit.
- Statements about every n (division with remainder, residue classes for all n, primes above 5) need induction, which Robinson's Q lacks.

Representation changes (binary numerals as definitions), induction axioms and kernel numeral primitives are **deferred to a separate review**, as decided at approval. These measurements are the baseline that review starts from.

## Reproduce

```sh
cargo build --release
python3 scripts/measure-baseline.py target/release/rlt /tmp/rlt-baseline > baseline.md
```

Sizes and verdicts are deterministic (two runs gave identical tables apart from times). Generated proof files are written to the output directory, outside the repository.

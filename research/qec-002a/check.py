#!/usr/bin/env python3
"""RLT-QEC-002a soundness checker, written separately from census.py and
importing nothing from it.

It accepts a list of masks L (one decimal integer per line) only if:
  1. every line is a canonical decimal integer 0 <= u < 2^90;
  2. the list is strictly increasing (hence duplicate-free);
  3. every u has weight <= W (default 7);
  4. every u is a cycle: d1 u = 0, with d1(u, v) = B*u + A*v evaluated by the
     convolution definition (f*g)(h) = sum_g f(h - g) g(g) over Z15 x Z3
     (GF2-02, GF2-03), after decoding u through the registered encoding;
  5. the set is closed under all 45 translations T_t (g, j) -> (g + t, j).

This establishes soundness of L (every element lies in Z_W, no repeats) and
orbit closure. It says nothing about completeness, which rests on the
reviewed argument of docs/RLT-QEC-002a.md §4 (level R).

Usage: check.py FILE [--wmax W]     Exit 0 ACCEPT, 1 REJECT (reason on stderr).
"""
import re
import sys

L_, M_ = 15, 3
N_G = L_ * M_
N_Q = 2 * N_G
# Polynomials as coefficient functions on the group (GF2-01).
A_EXP = {(9, 0), (0, 1), (0, 2)}     # x^9 + y + y^2
B_EXP = {(0, 0), (10, 0), (11, 0)}   # 1 + x^10 + x^11
GROUP = [(x, y) for x in range(L_) for y in range(M_)]


def coeff(poly, g):
    return 1 if (g[0] % L_, g[1] % M_) in poly else 0


def decode(u):
    """mask -> (u0, u1): two functions G -> {0,1}, by the registered encoding
    qubit index k = 45 j + 3 ((-x) mod 15) + ((-y) mod 3)."""
    halves = ({g: 0 for g in GROUP}, {g: 0 for g in GROUP})
    for j in (0, 1):
        for (x, y) in GROUP:
            k = N_G * j + M_ * ((-x) % L_) + ((-y) % M_)
            halves[j][(x, y)] = (u >> k) & 1
    return halves


def encode(halves):
    u = 0
    for j in (0, 1):
        for (x, y), bit in halves[j].items():
            if bit:
                u |= 1 << (N_G * j + M_ * ((-x) % L_) + ((-y) % M_))
    return u


def conv(poly, f):
    """(poly * f)(h) = sum over g of poly(h - g) f(g), mod 2."""
    return {h: sum(coeff(poly, (h[0] - g[0], h[1] - g[1])) * f[g] for g in GROUP) % 2
            for h in GROUP}


def d1(halves):
    left, right = conv(B_EXP, halves[0]), conv(A_EXP, halves[1])
    return {h: (left[h] + right[h]) % 2 for h in GROUP}


def translate(halves, t):
    return tuple({((x + t[0]) % L_, (y + t[1]) % M_): f[(x, y)] for (x, y) in GROUP}
                 for f in halves)


def check(lines, wmax):
    masks = []
    for i, line in enumerate(lines, 1):
        if not re.fullmatch(r"0|[1-9][0-9]*", line):
            return f"line {i}: not a canonical decimal integer"
        u = int(line)
        if u >= 1 << N_Q:
            return f"line {i}: mask >= 2^{N_Q}"
        masks.append(u)
    for i in range(1, len(masks)):
        if masks[i - 1] >= masks[i]:
            return f"line {i + 1}: list is not strictly increasing"
    present = set(masks)
    for i, u in enumerate(masks, 1):
        halves = decode(u)
        if encode(halves) != u:
            return f"line {i}: encoding round trip failed"
        w = sum(halves[0].values()) + sum(halves[1].values())
        if w > wmax:
            return f"line {i}: weight {w} > {wmax}"
        if any(d1(halves).values()):
            return f"line {i}: d1 u != 0"
        for t in GROUP:
            if encode(translate(halves, t)) not in present:
                return f"line {i}: translate by {t} missing from the list"
    return None


def main(argv):
    args = list(argv)
    wmax = 7
    if "--wmax" in args:
        i = args.index("--wmax")
        wmax = int(args[i + 1])
        del args[i:i + 2]
    if len(args) != 1:
        print(__doc__, file=sys.stderr)
        return 2
    with open(args[0]) as f:
        text = f.read()
    if text and not text.endswith("\n"):
        print("REJECT: missing final newline", file=sys.stderr)
        return 1
    reason = check(text.splitlines(), wmax)
    if reason:
        print(f"REJECT: {reason}", file=sys.stderr)
        return 1
    print(f"ACCEPT: {len(text.splitlines())} masks; each a cycle of weight <= {wmax}, "
          f"strictly increasing, closed under the {N_G} translations")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

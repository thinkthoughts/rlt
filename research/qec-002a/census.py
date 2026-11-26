#!/usr/bin/env python3
"""RLT-QEC-002a enumerator: every cycle of weight <= W of a bivariate-bicycle
base complex, from the registered definitions alone.

Inputs are the registered specification only (docs/RLT-QEC-001.md §2, §2.1,
§3.1): the group Z_l x Z_m, the supports of A and B, the GF2-03 boundary map
d1(u, v) = B*u + A*v, and the registered coordinate encoding

    face(x, y)     = m * ((-x) mod l) + ((-y) mod m)
    qubit(x, y, j) = (l*m) * j + face(x, y)
    mask(u)        = sum of 2^k over the qubit indices k in supp(u).

This program reads no source certificate, census or candidate list.

Two routes compute the same set (docs/RLT-QEC-002a.md §3):
  route 1  canonical meet-in-the-middle over all vectors of weight <= W;
  route 2  translation representatives (support contains qubit (0, 0, j)),
           meet-in-the-middle on the remainder, then closure under all l*m
           translations and deduplication by mask.
Every vector either route produces is rebuilt as a mask and its weight and
syndrome are rechecked before it is kept.

Usage: census.py [--route 1|2|both] [--out FILE]
       (defaults: the Bravyi360 base complex, W = 7, both routes, stdout)
Output: one decimal mask per line, ascending. Exit 0 on success, 1 if the
routes disagree or a recheck fails.
"""
import argparse
import hashlib
import sys
from itertools import combinations

if sys.version_info < (3, 8):
    sys.exit("census.py requires Python 3.8 or later")

# Registered Bravyi360 base complex (docs/RLT-QEC-001.md §3.1).
BASE = {
    "l": 15,
    "m": 3,
    "A": [(9, 0), (0, 1), (0, 2)],    # x^9 + y + y^2
    "B": [(0, 0), (10, 0), (11, 0)],  # 1 + x^10 + x^11
}
W_DEFAULT = 7
STATS = []   # search-space accounting, reported on stderr


class Complex:
    """d1 of bbChainComplex A B over Z_l x Z_m in registered coordinates."""

    def __init__(self, l, m, A, B):
        self.l, self.m, self.A, self.B = l, m, list(A), list(B)
        self.ng = l * m
        self.n = 2 * self.ng
        # column[k] = syndrome mask (over face indices) of the unit vector e_k
        self.column = [0] * self.n
        for x in range(l):
            for y in range(m):
                for j, supp in ((0, self.B), (1, self.A)):  # d1(u,v) = B*u + A*v
                    s = 0
                    for (a, b) in supp:
                        s ^= 1 << self.face((x + a) % l, (y + b) % m)
                    self.column[self.qubit(x, y, j)] = s

    def face(self, x, y):
        return self.m * ((-x) % self.l) + ((-y) % self.m)

    def qubit(self, x, y, j):
        return self.ng * j + self.face(x, y)

    def coords(self, k):
        j, f = divmod(k, self.ng)
        a, b = divmod(f, self.m)
        return ((-a) % self.l, (-b) % self.m, j)

    def syndrome(self, mask):
        s, k = 0, 0
        while mask:
            if mask & 1:
                s ^= self.column[k]
            mask >>= 1
            k += 1
        return s

    def translate(self, mask, t):
        tx, ty = t
        out, k = 0, 0
        while mask:
            if mask & 1:
                x, y, j = self.coords(k)
                out |= 1 << self.qubit((x + tx) % self.l, (y + ty) % self.m, j)
            mask >>= 1
            k += 1
        return out


def _half_table(cx, indices, maxsize):
    """syndrome -> list of (mask, size, min index) over subsets of `indices`
    of size <= maxsize. Subsets are listed with their least index so that the
    canonical split condition max(S) < min(T) can be tested."""
    table = {}
    for size in range(maxsize + 1):
        for T in combinations(indices, size):
            s, mk = 0, 0
            for k in T:
                s ^= cx.column[k]
                mk |= 1 << k
            table.setdefault(s, []).append((mk, size, T[0] if T else None))
    return table


def _mitm(cx, indices, wmax, target):
    """All masks v supported on `indices` with |v| <= wmax and syndrome(v) =
    target, each exactly once, via the canonical split: S holds the
    ceil(|v|/2) least indices of supp(v) and T the rest, so |T| is |S| or
    |S| - 1, max(S) < min(T), and syndrome(S) ^ target = syndrome(T)."""
    hi = wmax // 2            # |T| <= floor(wmax / 2)
    lo = wmax - hi            # |S| <= ceil(wmax / 2)
    table = _half_table(cx, indices, hi)
    STATS.append({"indices": len(indices), "wmax": wmax,
                  "T_subsets": sum(len(v) for v in table.values()), "S_subsets": 0})
    found = []
    for size in range(lo + 1):
        for S in combinations(indices, size):
            STATS[-1]["S_subsets"] += 1
            s, mk = 0, 0
            for k in S:
                s ^= cx.column[k]
                mk |= 1 << k
            maxS = S[-1] if S else -1
            for (tm, tsize, tmin) in table.get(s ^ target, ()):
                if tsize not in (size, size - 1):
                    continue
                if tsize and tmin <= maxS:
                    continue
                found.append(mk | tm)
    return found


def _recheck(cx, masks, wmax):
    """Rebuild-and-recheck step of Revision 3 §7.1: weight and syndrome of
    every produced mask; returns the deduplicated sorted list."""
    out = set()
    for u in masks:
        if bin(u).count("1") > wmax or cx.syndrome(u) != 0:
            raise SystemExit(f"recheck failed for mask {u}")
        out.add(u)
    return sorted(out)


def route1(cx, wmax):
    raw = _mitm(cx, list(range(cx.n)), wmax, 0)
    if len(raw) != len(set(raw)):
        raise SystemExit("route 1 produced a vector twice; the canonical split is broken")
    return _recheck(cx, raw, wmax)


def route2(cx, wmax):
    reps = []
    for j in (0, 1):
        q0 = cx.qubit(0, 0, j)
        rest = [k for k in range(cx.n) if k != q0]
        for v in _mitm(cx, rest, wmax - 1, cx.column[q0]):
            reps.append(v | (1 << q0))
    closed = {0}
    for u in reps:
        for tx in range(cx.l):
            for ty in range(cx.m):
                closed.add(cx.translate(u, (tx, ty)))
    return _recheck(cx, closed, wmax)


def main(argv=None):
    ap = argparse.ArgumentParser()
    ap.add_argument("--route", choices=["1", "2", "both"], default="both")
    ap.add_argument("--wmax", type=int, default=W_DEFAULT)
    ap.add_argument("--out")
    a = ap.parse_args(argv)
    cx = Complex(BASE["l"], BASE["m"], BASE["A"], BASE["B"])
    r1 = route1(cx, a.wmax) if a.route in ("1", "both") else None
    r2 = route2(cx, a.wmax) if a.route in ("2", "both") else None
    if r1 is not None and r2 is not None and r1 != r2:
        print(f"routes disagree: route 1 {len(r1)}, route 2 {len(r2)}", file=sys.stderr)
        return 1
    result = r1 if r1 is not None else r2
    text = "".join(f"{u}\n" for u in result)
    if a.out:
        with open(a.out, "w") as f:
            f.write(text)
    else:
        sys.stdout.write(text)
    dist = {}
    for u in result:
        w = bin(u).count("1")
        dist[w] = dist.get(w, 0) + 1
    print(f"route(s) {a.route}: {len(result)} cycles of weight <= {a.wmax}; "
          f"weights {dict(sorted(dist.items()))}; "
          f"sha256 {hashlib.sha256(text.encode()).hexdigest()}", file=sys.stderr)
    for s in STATS:
        print(f"  search: {s['indices']} indices, weight <= {s['wmax']}: "
              f"{s['S_subsets']} S-halves x table of {s['T_subsets']} T-halves", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())

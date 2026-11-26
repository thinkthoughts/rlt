#!/usr/bin/env python3
"""RLT-QEC-002b builder: boundary witnesses, rank certificates and a
nonvacuity witness for the Bravyi360 base complex (docs/RLT-QEC-002b.md).

Inputs: the registered definitions (docs/RLT-QEC-001.md §2, §2.1, §3.1) and
the accepted RLT-QEC-002a census research/qec-002a/Z7.txt, read and checked
against its SHA-256. This program reads no source data and shares no code
with RLT-QEC-002a.

  d2 f = (A*f, B*f)            C2 (45 faces) -> C1 (90 qubits)
  d1 (u0, u1) = B*u0 + A*u1    C1 -> C0 (45 positions)
  face(x, y) = 3((-x) mod 15) + ((-y) mod 3),  qubit(x, y, j) = 45 j + face(x, y)

Outputs (in the output directory, default: this directory):
  W.txt          46 lines "u c": for every census mask u, a coefficient mask c
                 with d2 c = u, found by Gaussian elimination
  ranks.json     rank certificates for d2 and d1 (an invertible minor with its
                 inverse: rank >= r; a kernel basis in unit form: rank <= r),
                 k = 90 - rank d1 - rank d2, and a nonvacuity witness (u, h):
                 d1 u = 0, h^T d2 = 0, h^T u = 1
  structure.txt  for every nonzero census mask, the face g with d2 delta_g = u

Usage: witness.py [--outdir DIR]       Requires Python 3.8 or later.
"""
import argparse
import hashlib
import json
import os
import sys

if sys.version_info < (3, 8):
    sys.exit("witness.py requires Python 3.8 or later")

HERE = os.path.dirname(os.path.abspath(__file__))
CENSUS = os.path.join(HERE, "..", "qec-002a", "Z7.txt")
CENSUS_SHA256 = "e2f8e4b6435f5b34c7044885c2d3fb9be5a89ccfb5dbaed0bf5f7d1522dfff2e"

BASE = {"l": 15, "m": 3,
        "A": [(9, 0), (0, 1), (0, 2)],      # x^9 + y + y^2
        "B": [(0, 0), (10, 0), (11, 0)]}    # 1 + x^10 + x^11


class BB:
    """Boundary maps of bbChainComplex A B over Z_l x Z_m as column masks."""

    def __init__(self, l, m, A, B):
        self.l, self.m, self.ng = l, m, l * m
        self.n1 = 2 * self.ng
        self.group = [(x, y) for x in range(l) for y in range(m)]
        # d2: column face(g) = mask over qubits of d2 delta_g
        self.d2 = [0] * self.ng
        for g in self.group:
            col, pts = 0, []
            for j, supp in ((0, A), (1, B)):
                for (a, b) in supp:
                    q = self.qubit((g[0] + a) % l, (g[1] + b) % m, j)
                    pts.append(q)
                    col ^= 1 << q
            self.d2[self.face(*g)] = col
        # d1: column qubit(g, j) = mask over C0 positions of d1 e_(g, j)
        self.d1 = [0] * self.n1
        for g in self.group:
            for j, supp in ((0, B), (1, A)):
                col = 0
                for (a, b) in supp:
                    col ^= 1 << self.face((g[0] + a) % l, (g[1] + b) % m)
                self.d1[self.qubit(g[0], g[1], j)] = col

    def face(self, x, y):
        return self.m * ((-x) % self.l) + ((-y) % self.m)

    def qubit(self, x, y, j):
        return self.ng * j + self.face(x, y)

    def face_coords(self, i):
        a, b = divmod(i, self.m)
        return ((-a) % self.l, (-b) % self.m)


def apply(cols, c):
    """M c for a matrix given by column masks and a coefficient mask c."""
    out, k = 0, 0
    while c:
        if c & 1:
            out ^= cols[k]
        c >>= 1
        k += 1
    return out


class Elim:
    """Fully reduced column elimination of a GF(2) matrix (columns = masks).

    Basis vectors b_t = M tau_t with distinct pivot rows R_t, each b_t zero on
    every other pivot row; pivot columns C_t; kernel vectors for the remaining
    columns."""

    def __init__(self, cols):
        self.cols = list(cols)
        self.basis = []            # [pivot_row, vec, combo over original columns]
        self.pivot_cols = []
        self.kernel = {}           # non-pivot column -> kernel vector mask
        for j, col in enumerate(self.cols):
            vec, combo = self._reduce(col, 1 << j)
            if vec == 0:
                self.kernel[j] = combo
                continue
            p = (vec & -vec).bit_length() - 1
            for e in self.basis:
                if (e[1] >> p) & 1:
                    e[1] ^= vec
                    e[2] ^= combo
            self.basis.append([p, vec, combo])
            self.pivot_cols.append(j)

    def _reduce(self, vec, combo):
        for p, bv, bc in self.basis:
            if (vec >> p) & 1:
                vec ^= bv
                combo ^= bc
        return vec, combo

    @property
    def rank(self):
        return len(self.basis)

    def solve(self, u):
        """Some c with M c = u, or None."""
        vec, combo = self._reduce(u, 0)
        return combo if vec == 0 else None

    def certificate(self):
        """Lower bound: rows R, columns C, inverse N (as columns over C-index
        bits) with M[R, C] N = I. Upper bound: free columns D and kernel
        vectors K with K_t restricted to D equal to the unit vector at D_t."""
        R = [e[0] for e in self.basis]
        C = list(self.pivot_cols)
        cpos = {c: s for s, c in enumerate(C)}
        N = []
        for p, vec, combo in self.basis:
            tau, j = 0, 0
            while combo:
                if combo & 1:
                    tau |= 1 << cpos[j]   # combo of a basis vector lies on pivot columns
                combo >>= 1
                j += 1
            N.append(tau)
        D = sorted(self.kernel)
        K = [self.kernel[d] for d in D]
        return {"rank": self.rank, "rows": R, "cols": C, "inverse": [str(t) for t in N],
                "free": D, "kernel": [str(v) for v in K]}


def transpose(cols, nrows):
    rows = [0] * nrows
    for j, col in enumerate(cols):
        for i in range(nrows):
            if (col >> i) & 1:
                rows[i] |= 1 << j
    return rows


def read_census():
    with open(CENSUS, "rb") as f:
        data = f.read()
    if hashlib.sha256(data).hexdigest() != CENSUS_SHA256:
        raise SystemExit("research/qec-002a/Z7.txt differs from the accepted census")
    return [int(s) for s in data.decode().split()]


def build(bb, census):
    e2 = Elim(bb.d2)
    e1 = Elim(bb.d1)
    witnesses = []
    for u in census:
        c = e2.solve(u)
        if c is None or apply(bb.d2, c) != u:
            raise SystemExit(f"census mask {u} has no boundary witness")
        witnesses.append((u, c))
    # nonvacuity: first d1-kernel vector (in certificate order) outside im d2,
    # and the first left-kernel vector of d2 pairing to 1 with it
    cert1 = e1.certificate()
    nontrivial = next((int(v) for v in cert1["kernel"] if e2.solve(int(v)) is None), None)
    nonvac = None
    if nontrivial is not None:
        left = Elim(transpose(bb.d2, bb.n1)).certificate()["kernel"]   # h with h^T d2 = 0
        h = next(int(v) for v in left if bin(int(v) & nontrivial).count("1") % 2 == 1)
        nonvac = {"u": str(nontrivial), "h": str(h)}
    k = bb.n1 - e1.rank - e2.rank
    structure = []
    for u, c in witnesses:
        if u:
            i = bb.d2.index(u) if u in bb.d2 else None
            relation = "equal" if c == 1 << i else ("kernel" if apply(bb.d2, c ^ (1 << i)) == 0 else "other")
            structure.append((u, i, bb.face_coords(i), relation))
    ranks = {"complex": "Bravyi360 base, bbChainComplex A B over Z15 x Z3",
             "d2": e2.certificate(), "d1": cert1, "k": k,
             "homology_classes": str(2 ** k), "nonvacuity": nonvac}
    return witnesses, ranks, structure


def main(argv=None):
    ap = argparse.ArgumentParser()
    ap.add_argument("--outdir", default=HERE)
    a = ap.parse_args(argv)
    bb = BB(**BASE)
    if len(set(bb.d2)) != 45 or any(bin(c).count("1") != 6 for c in bb.d2):
        raise SystemExit("d2 columns are not 45 distinct masks of weight 6")
    witnesses, ranks, structure = build(bb, read_census())
    with open(os.path.join(a.outdir, "W.txt"), "w") as f:
        f.write("".join(f"{u} {c}\n" for u, c in witnesses))
    with open(os.path.join(a.outdir, "ranks.json"), "w") as f:
        json.dump(ranks, f, indent=1, sort_keys=True)
        f.write("\n")
    with open(os.path.join(a.outdir, "structure.txt"), "w") as f:
        f.write("".join(f"{u} {i} {g[0]} {g[1]} {rel}\n" for u, i, g, rel in structure))
    nv = ranks["nonvacuity"]
    print(f"witnesses {len(witnesses)}; rank d2 {ranks['d2']['rank']}, rank d1 {ranks['d1']['rank']}, "
          f"k {ranks['k']}, homology classes {ranks['homology_classes']}; "
          f"nonvacuity witness {'u of weight ' + str(bin(int(nv['u'])).count('1')) if nv else 'none'}; "
          f"structure {sorted(set(r for *_, r in structure))}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())

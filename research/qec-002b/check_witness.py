#!/usr/bin/env python3
"""RLT-QEC-002b checker, written separately from witness.py and from the
RLT-QEC-002a programs, importing only the standard library.

Both boundary maps are evaluated by the convolution definition
(a*f)(h) = sum_g a(h - g) f(g) over Z15 x Z3 (GF2-02, GF2-03):
  d2 f = (A*f, B*f),  d1 (u0, u1) = B*u0 + A*u1,
with masks decoded through the registered encoding (docs/RLT-QEC-001.md §2.1):
  qubit (x, y, j) -> bit 45 j + 3((-x) mod 15) + ((-y) mod 3); face (x, y) -> bit 3((-x) mod 15) + ((-y) mod 3).

check_witness.py W.txt Z7.txt RANKS.json STRUCTURE.txt
  ACCEPT (exit 0) only if every condition of docs/RLT-QEC-002b.md §5 holds:
   1 Z7.txt is the accepted census (SHA-256 pin)
   2-4 W.txt: canonical lines "u c", ranges, u column = the census, ascending
   5 every u is a cycle;  6 d2 c = u for every line;  7 the zero vector's witness
   8 C-2: d1(d2 delta_g) = 0 for all 45 faces
   9-10 rank certificates for d2 and d1 (rank >= r by an inverted minor,
        rank <= r by a unit-form kernel basis), so each rank is exact
   11 k = 90 - rank d1 - rank d2 as stated, and 2^k homology classes
   12 nonvacuity witness (u, h, weight): d1 u = 0, h^T d2 = 0, h^T u = 1,
      wt(u) equal to the stated weight, and k >= 1; this certifies a
      nontrivial X-sector class and the upper bound d_X <= weight
   13 structure: every nonzero census mask equals d2 delta_g for its listed face
  REJECT (exit 1) with the first failing condition; exit 2 on usage errors.
Requires Python 3.8 or later.
"""
import hashlib
import json
import re
import sys

if sys.version_info < (3, 8):
    sys.exit("check_witness.py requires Python 3.8 or later")

CENSUS_SHA256 = "e2f8e4b6435f5b34c7044885c2d3fb9be5a89ccfb5dbaed0bf5f7d1522dfff2e"
L_, M_ = 15, 3
NG, NQ = 45, 90
GROUP = [(x, y) for x in range(L_) for y in range(M_)]
A_SUPP = {(9, 0), (0, 1), (0, 2)}
B_SUPP = {(0, 0), (10, 0), (11, 0)}
CANON = re.compile(r"0|[1-9][0-9]*")


class Reject(Exception):
    pass


def coeff(supp, g):
    return 1 if (g[0] % L_, g[1] % M_) in supp else 0


def face_bit(g):
    return M_ * ((-g[0]) % L_) + ((-g[1]) % M_)


def conv(supp, f):
    return {h: sum(coeff(supp, (h[0] - g[0], h[1] - g[1])) * f[g] for g in GROUP) % 2
            for h in GROUP}


def decode_c1(u):
    return tuple({g: (u >> (NG * j + face_bit(g))) & 1 for g in GROUP} for j in (0, 1))


def decode_c2(c):
    return {g: (c >> face_bit(g)) & 1 for g in GROUP}


def encode_c1(halves):
    out = 0
    for j in (0, 1):
        for g, bit in halves[j].items():
            if bit:
                out |= 1 << (NG * j + face_bit(g))
    return out


def encode_c0(f):
    out = 0
    for g, bit in f.items():
        if bit:
            out |= 1 << face_bit(g)
    return out


def d2(c):
    f = decode_c2(c)
    return encode_c1((conv(A_SUPP, f), conv(B_SUPP, f)))


def d1(u):
    u0, u1 = decode_c1(u)
    left, right = conv(B_SUPP, u0), conv(A_SUPP, u1)
    return encode_c0({h: (left[h] + right[h]) % 2 for h in GROUP})


def parity(x):
    return bin(x).count("1") & 1


def canon_int(s, what, bound):
    if not isinstance(s, str) or not CANON.fullmatch(s):
        raise Reject(f"{what}: not a canonical decimal string")
    v = int(s)
    if v >= bound:
        raise Reject(f"{what}: out of range")
    return v


def check_rank(name, cert, column, nrows, ncols):
    """column(j) = mask (over nrows bits) of the j-th column, computed here."""
    keys = {"rank", "rows", "cols", "inverse", "free", "kernel"}
    if not isinstance(cert, dict) or set(cert) != keys:
        raise Reject(f"{name}: certificate keys must be {sorted(keys)}")
    r = cert["rank"]
    if type(r) is not int or not 0 <= r <= min(nrows, ncols):
        raise Reject(f"{name}: rank out of range")
    R, C, D = cert["rows"], cert["cols"], cert["free"]
    for what, idx, bound, size in (("rows", R, nrows, r), ("cols", C, ncols, r), ("free", D, ncols, ncols - r)):
        if (not isinstance(idx, list) or len(idx) != size or len(set(idx)) != size
                or any(type(i) is not int or not 0 <= i < bound for i in idx)):
            raise Reject(f"{name}: {what} must be {size} distinct indices below {bound}")
    if len(cert["inverse"]) != r or len(cert["kernel"]) != ncols - r:
        raise Reject(f"{name}: inverse must have {r} entries and kernel {ncols - r}")
    N = [canon_int(t, f"{name} inverse", 1 << r) for t in cert["inverse"]]
    K = [canon_int(v, f"{name} kernel", 1 << ncols) for v in cert["kernel"]]
    cols = [column(j) for j in range(ncols)]
    # lower bound: M[R, C] N = I
    for t in range(r):
        prod = 0
        for s in range(r):
            if (N[t] >> s) & 1:
                prod ^= cols[C[s]]
        restricted = sum(((prod >> R[i]) & 1) << i for i in range(r))
        if restricted != 1 << t:
            raise Reject(f"{name}: M[R, C] N differs from the identity in column {t}")
    # upper bound: kernel vectors in unit form on D, each annihilated by M
    dmask = sum(1 << d for d in D)
    for t, v in enumerate(K):
        if v & dmask != 1 << D[t]:
            raise Reject(f"{name}: kernel vector {t} is not the unit vector at free index {D[t]} on the free set")
        img = 0
        for j in range(ncols):
            if (v >> j) & 1:
                img ^= cols[j]
        if img:
            raise Reject(f"{name}: kernel vector {t} is outside the kernel")
    return r


def check(w_path, z7_path, ranks_path, structure_path):
    with open(z7_path, "rb") as f:
        z7_bytes = f.read()
    if hashlib.sha256(z7_bytes).hexdigest() != CENSUS_SHA256:                      # 1
        raise Reject("Z7.txt is not the accepted RLT-QEC-002a census")
    census = [int(s) for s in z7_bytes.decode().split()]
    with open(w_path) as f:
        text = f.read()
    if not text.endswith("\n"):
        raise Reject("W.txt: missing final newline")
    pairs = []
    for i, line in enumerate(text.split("\n")[:-1], 1):                           # 2, 3
        parts = line.split(" ")
        if len(parts) != 2:
            raise Reject(f"W.txt line {i}: expected 'u c'")
        pairs.append((canon_int(parts[0], f"W.txt line {i} u", 1 << NQ),
                      canon_int(parts[1], f"W.txt line {i} c", 1 << NG)))
    us = [u for u, _ in pairs]
    if any(a >= b for a, b in zip(us, us[1:])):                                     # 4
        raise Reject("W.txt: u column is not strictly increasing")
    if set(us) != set(census) or len(us) != len(census):
        raise Reject("W.txt: u column differs from the census")
    for u, c in pairs:
        if d1(u):                                                                   # 5
            raise Reject(f"W.txt: u = {u} is not a cycle")
        if d2(c) != u:                                                              # 6, 7
            raise Reject(f"W.txt: d2 c != u for u = {u}")
    columns2 = [d2(1 << i) for i in range(NG)]
    if any(d1(col) for col in columns2):                                            # 8
        raise Reject("C-2: d1 d2 != 0")
    with open(ranks_path) as f:
        ranks = json.load(f)
    keys = {"complex", "d1", "d2", "k", "homology_classes", "nonvacuity"}
    if not isinstance(ranks, dict) or set(ranks) != keys:
        raise Reject(f"ranks.json: keys must be {sorted(keys)}")
    r2 = check_rank("d2", ranks["d2"], lambda j: columns2[j], NQ, NG)               # 9
    r1 = check_rank("d1", ranks["d1"], lambda j: d1(1 << j), NG, NQ)                # 10
    k = NQ - r1 - r2
    if ranks["k"] != k or ranks["homology_classes"] != str(2 ** k):                 # 11
        raise Reject(f"ranks.json: k must be {k} and homology_classes {2 ** k}")
    nv = ranks["nonvacuity"]                                                        # 12
    if not isinstance(nv, dict) or set(nv) != {"u", "h", "weight"}:
        raise Reject("nonvacuity: expected an object with keys u, h, weight")
    nu = canon_int(nv["u"], "nonvacuity u", 1 << NQ)
    nh = canon_int(nv["h"], "nonvacuity h", 1 << NQ)
    if d1(nu):
        raise Reject("nonvacuity: d1 u != 0")
    if any(parity(nh & col) for col in columns2):
        raise Reject("nonvacuity: h^T d2 != 0")
    if parity(nh & nu) != 1:
        raise Reject("nonvacuity: h^T u != 1")
    w = nv["weight"]
    if type(w) is not int or w != bin(nu).count("1"):
        raise Reject("nonvacuity: stated weight differs from wt(u)")
    if k < 1:
        raise Reject("nonvacuity witness present but k < 1")
    with open(structure_path) as f:                                                 # 13
        stext = f.read()
    if not stext.endswith("\n"):
        raise Reject("structure.txt: missing final newline")
    rows = [line.split(" ") for line in stext.split("\n")[:-1]]
    nonzero = [u for u in census if u]
    if len(rows) != len(nonzero):
        raise Reject("structure.txt: one line per nonzero census mask required")
    for row, u in zip(rows, nonzero):
        if len(row) != 5 or row[4] not in ("equal", "kernel"):
            raise Reject("structure.txt: expected 'u face x y equal|kernel'")
        su = canon_int(row[0], "structure u", 1 << NQ)
        i = canon_int(row[1], "structure face", NG)
        g = (canon_int(row[2], "structure x", L_), canon_int(row[3], "structure y", M_))
        if su != u or face_bit(g) != i or columns2[i] != u:
            raise Reject(f"structure.txt: u = {u} is not d2 delta_g for the listed face")
        c = dict(pairs)[u]
        if (row[4] == "equal") != (c == 1 << i):
            raise Reject(f"structure.txt: relation for u = {u} misreported")
    return {"witnesses": len(pairs), "rank_d2": r2, "rank_d1": r1, "k": k,
            "nonvacuity_weight": w}


def main(argv):
    if len(argv) != 4:
        print(__doc__, file=sys.stderr)
        return 2
    try:
        res = check(*argv)
    except Reject as e:
        print(f"REJECT: {e}", file=sys.stderr)
        return 1
    except (OSError, ValueError, UnicodeDecodeError) as e:
        print(f"REJECT: unreadable input ({e})", file=sys.stderr)
        return 1
    print(f"ACCEPT: {res['witnesses']} witnesses with d2 c = u for every census mask; "
          f"rank d2 = {res['rank_d2']}, rank d1 = {res['rank_d1']} (certified both ways); "
          f"k = {res['k']}, {2 ** res['k']} homology classes; nonvacuity witness verified "
          f"(cycle of weight {res['nonvacuity_weight']} outside im d2, so d_X <= {res['nonvacuity_weight']}); "
          f"d1 d2 = 0; structure verified")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

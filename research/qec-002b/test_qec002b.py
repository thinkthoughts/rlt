#!/usr/bin/env python3
"""Tests for RLT-QEC-002b. Run: python3 -I research/qec-002b/test_qec002b.py
Requires Python 3.8 or later. Set QEC_SRC to a bb-distance-lean checkout
containing the pinned commit to also run the source tests (skipped otherwise)."""
import hashlib
import importlib.util
import json
import os
import random
import re
import shutil
import subprocess
import sys
import tempfile
import unittest

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
Z7 = os.path.join(HERE, "..", "qec-002a", "Z7.txt")
OUTPUTS = ("W.txt", "ranks.json", "structure.txt")


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


wit = load("witness", os.path.join(HERE, "witness.py"))
chk = load("check_witness", os.path.join(HERE, "check_witness.py"))
BASE = wit.BB(**wit.BASE)


def run(script, *args, env=None):
    return subprocess.run([sys.executable, "-I", os.path.join(HERE, script), *args],
                          capture_output=True, text=True, env=env)


def popcount(x):
    return bin(x).count("1")


class Maps(unittest.TestCase):
    def test_d2_columns_equal_convolution(self):
        for i in range(45):
            self.assertEqual(BASE.d2[i], chk.d2(1 << i), f"face {i}")

    def test_d1_columns_equal_convolution(self):
        for q in range(90):
            self.assertEqual(BASE.d1[q], chk.d1(1 << q), f"qubit {q}")

    def test_d1_equals_accepted_002a_columns(self):
        census = load("census_002a", os.path.join(HERE, "..", "qec-002a", "census.py"))
        cx = census.Complex(**census.BASE)
        self.assertEqual(BASE.d1, cx.column)

    def test_d2_columns_registered_points(self):
        for i in range(45):
            x, y = BASE.face_coords(i)
            pts = [BASE.qubit((x + 9) % 15, y, 0), BASE.qubit(x, (y + 1) % 3, 0),
                   BASE.qubit(x, (y + 2) % 3, 0), BASE.qubit(x, y, 1),
                   BASE.qubit((x + 10) % 15, y, 1), BASE.qubit((x + 11) % 15, y, 1)]
            self.assertEqual(BASE.d2[i], sum(1 << p for p in pts))
        self.assertEqual(len(set(BASE.d2)), 45)

    def test_d1_d2_zero(self):
        for i in range(45):
            self.assertEqual(wit.apply(BASE.d1, BASE.d2[i]), 0)
            self.assertEqual(chk.d1(chk.d2(1 << i)), 0)


class Elimination(unittest.TestCase):
    def test_random_matrices_against_brute_force(self):
        rng = random.Random(25112026)
        for _ in range(60):
            nrows, ncols = rng.randint(1, 9), rng.randint(1, 9)
            cols = [rng.getrandbits(nrows) for _ in range(ncols)]
            e = wit.Elim(cols)
            image = {wit.apply(cols, c) for c in range(1 << ncols)}
            self.assertEqual(1 << e.rank, len(image))
            for u in range(1 << nrows):
                c = e.solve(u)
                self.assertEqual(c is not None, u in image)
                if c is not None:
                    self.assertEqual(wit.apply(cols, c), u)
            self.assertEqual(chk.check_rank("t", e.certificate(), lambda j: cols[j], nrows, ncols), e.rank)


class ToyComplexes(unittest.TestCase):
    """Ranks, k, witness solvability and nonvacuity against brute force."""
    CASES = [
        (3, 2, [(1, 0), (0, 1), (0, 0)], [(0, 0), (2, 0), (0, 1)]),
        (2, 2, [(0, 0), (1, 1)], [(1, 0), (0, 1)]),
        (4, 2, [(1, 0), (0, 1), (2, 1)], [(0, 0), (1, 1), (3, 0)]),
        (3, 2, [(0, 0), (1, 0)], [(0, 0), (1, 0)]),
        (4, 1, [(0, 0), (1, 0)], [(0, 0), (3, 0)]),
        (2, 3, [(0, 0), (1, 1), (0, 2)], [(1, 0), (0, 1), (1, 2)]),
    ]

    def test_against_brute_force(self):
        ks = set()
        for (l, m, A, B) in self.CASES:
            with self.subTest(l=l, m=m, A=A, B=B):
                bb = wit.BB(l, m, A, B)
                B1 = {wit.apply(bb.d2, c) for c in range(1 << bb.ng)}
                Z1 = [u for u in range(1 << bb.n1) if wit.apply(bb.d1, u) == 0]
                e2, e1 = wit.Elim(bb.d2), wit.Elim(bb.d1)
                self.assertEqual(1 << e2.rank, len(B1))
                self.assertEqual(1 << (bb.n1 - e1.rank), len(Z1))
                k = bb.n1 - e1.rank - e2.rank
                self.assertEqual(1 << k, len(Z1) // len(B1))
                ks.add(k > 0)
                for u in Z1:
                    c = e2.solve(u)
                    self.assertEqual(c is not None, u in B1)
                    if c is not None:
                        self.assertEqual(wit.apply(bb.d2, c), u)
                self.assertEqual(chk.check_rank("d2", e2.certificate(), lambda j: bb.d2[j], bb.n1, bb.ng), e2.rank)
                self.assertEqual(chk.check_rank("d1", e1.certificate(), lambda j: bb.d1[j], bb.ng, bb.n1), e1.rank)
        self.assertEqual(ks, {True, False}, "cases cover both vacuous and nonvacuous homology")


class Outputs(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.mkdtemp()
        p = run("witness.py", "--outdir", self.tmp)
        self.assertEqual(p.returncode, 0, p.stderr)

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def test_committed_outputs_reproduced(self):
        for name in OUTPUTS:
            with open(os.path.join(self.tmp, name), "rb") as a, open(os.path.join(HERE, name), "rb") as b:
                self.assertEqual(a.read(), b.read(), name)

    def test_values(self):
        with open(os.path.join(HERE, "ranks.json")) as f:
            r = json.load(f)
        self.assertEqual((r["d2"]["rank"], r["d1"]["rank"], r["k"], r["homology_classes"]), (41, 41, 8, "256"))
        u = int(r["nonvacuity"]["u"])
        self.assertGreaterEqual(popcount(u), 8, "a nontrivial cycle lighter than 8 would contradict F8")
        self.assertEqual(r["nonvacuity"]["weight"], popcount(u))
        self.assertEqual(r["nonvacuity"]["weight"], 10)
        with open(os.path.join(HERE, "W.txt")) as f:
            lines = f.read().split("\n")[:-1]
        self.assertEqual(len(lines), 46)
        self.assertEqual(lines[0], "0 0")

    def test_002a_unchanged(self):
        with open(Z7, "rb") as f:
            self.assertEqual(hashlib.sha256(f.read()).hexdigest(), wit.CENSUS_SHA256)
        p = subprocess.run(["sha256sum", "--quiet", "-c", "MANIFEST.sha256"],
                           cwd=os.path.join(HERE, "..", "qec-002a"), capture_output=True, text=True)
        self.assertEqual(p.returncode, 0, p.stdout + p.stderr)


class Checker(unittest.TestCase):
    """Every rejection condition of docs/RLT-QEC-002b.md §5."""

    def setUp(self):
        self.tmp = tempfile.mkdtemp()
        for name in OUTPUTS:
            shutil.copy(os.path.join(HERE, name), self.tmp)
        shutil.copy(Z7, os.path.join(self.tmp, "Z7.txt"))

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def path(self, name):
        return os.path.join(self.tmp, name)

    def verdict(self):
        p = run("check_witness.py", self.path("W.txt"), self.path("Z7.txt"),
                self.path("ranks.json"), self.path("structure.txt"))
        return p.returncode, p.stdout + p.stderr

    def edit(self, name, fn):
        with open(self.path(name)) as f:
            text = f.read()
        with open(self.path(name), "w") as f:
            f.write(fn(text))

    def edit_json(self, fn):
        with open(self.path("ranks.json")) as f:
            r = json.load(f)
        fn(r)
        with open(self.path("ranks.json"), "w") as f:
            json.dump(r, f)

    def rejects(self, needle=None):
        code, out = self.verdict()
        self.assertEqual(code, 1, out)
        if needle:
            self.assertIn(needle, out)

    def test_accepts(self):
        code, out = self.verdict()
        self.assertEqual(code, 0, out)
        self.assertIn("ACCEPT", out)

    def wlines(self):
        with open(self.path("W.txt")) as f:
            return f.read().split("\n")[:-1]

    def put_w(self, lines):
        with open(self.path("W.txt"), "w") as f:
            f.write("".join(l + "\n" for l in lines))

    def test_1_census_hash(self):
        self.edit("Z7.txt", lambda t: t + "0\n")
        self.rejects("accepted RLT-QEC-002a census")

    def test_2_format(self):
        L = self.wlines()
        for bad in (["0 00"] + L[1:], ["0  0"] + L[1:], ["0,0"] + L[1:], ["-0 0"] + L[1:]):
            self.put_w(bad)
            self.rejects()
        self.edit("W.txt", lambda t: t.rstrip("\n"))
        self.rejects("final newline")

    def test_3_ranges(self):
        L = self.wlines()
        u = L[1].split()[0]
        self.put_w([L[0], f"{u} {1 << 45}"] + L[2:])
        self.rejects("out of range")

    def test_4_coverage_and_order(self):
        L = self.wlines()
        for bad, needle in ((L[:-1], "differs from the census"), (L + [L[-1]], "strictly increasing"),
                            ([L[1], L[0]] + L[2:], "strictly increasing")):
            self.put_w(bad)
            self.rejects(needle)

    def test_5_non_cycle_with_test_census(self):
        # A census altered to contain a non-cycle is caught at condition 1;
        # condition 5 is exercised directly on the checker's function.
        self.assertNotEqual(chk.d1(1), 0)

    def test_6_wrong_witness(self):
        L = self.wlines()
        u, c = map(int, L[5].split())
        for badc in (c ^ 1, c ^ (1 << 7), 0):
            self.put_w(L[:5] + [f"{u} {badc}"] + L[6:])
            self.rejects("d2 c != u")

    def test_6_kernel_shifted_witness_accepted(self):
        """Review resolution 1: any c with d2 c = u is accepted."""
        with open(self.path("ranks.json")) as f:
            kvec = int(json.load(f)["d2"]["kernel"][0])
        L = self.wlines()
        u, c = map(int, L[5].split())
        self.put_w(L[:5] + [f"{u} {c ^ kvec}"] + L[6:])
        self.edit("structure.txt", lambda t: "\n".join(
            (" ".join(r.split()[:4] + ["equal" if c ^ kvec == 1 << int(r.split()[1]) else "kernel"])
             if r.split()[0] == str(u) else r) for r in t.split("\n")[:-1]) + "\n")
        code, out = self.verdict()
        self.assertEqual(code, 0, out)

    def test_7_zero_vector(self):
        L = self.wlines()
        self.put_w(["0 1"] + L[1:])
        self.rejects("d2 c != u for u = 0")

    def test_9_10_rank_certificates(self):
        cases = [
            (lambda r: r["d2"].__setitem__("rank", 42), None),
            (lambda r: r["d1"].__setitem__("rank", 40), None),
            (lambda r: r["d2"]["inverse"].__setitem__(0, str(int(r["d2"]["inverse"][0]) ^ 2)), "identity"),
            (lambda r: r["d1"]["kernel"].__setitem__(0, str(int(r["d1"]["kernel"][0]) ^ (1 << r["d1"]["cols"][0]))), "outside the kernel"),
            (lambda r: r["d2"]["kernel"].__setitem__(1, r["d2"]["kernel"][0]), "unit vector"),
            (lambda r: r["d2"]["rows"].__setitem__(0, r["d2"]["rows"][1]), "distinct"),
            (lambda r: r["d2"].pop("free"), "keys"),
        ]
        for fn, needle in cases:
            with self.subTest(needle=needle):
                self.setUp()
                self.edit_json(fn)
                self.rejects(needle)

    def test_11_k(self):
        self.edit_json(lambda r: r.__setitem__("k", 7))
        self.rejects("k must be")
        self.setUp()
        self.edit_json(lambda r: r.__setitem__("homology_classes", "128"))
        self.rejects("k must be")

    def test_12_nonvacuity(self):
        with open(self.path("ranks.json")) as f:
            r = json.load(f)
        u, h = int(r["nonvacuity"]["u"]), int(r["nonvacuity"]["h"])
        w = r["nonvacuity"]["weight"]
        face = BASE.d2[0]
        cases = [
            ({"u": str(u ^ 1), "h": str(h), "weight": w}, "d1 u != 0"),
            ({"u": str(u), "h": str(h ^ (1 << next(q for q in range(90) if not (u >> q) & 1))), "weight": w},
             "h^T d2 != 0"),   # h^T u stays 1; only the annihilation of im d2 fails
            ({"u": str(face), "h": str(h), "weight": 6}, "h^T u != 1"),
            ({"u": str(u), "h": str(h)}, "keys u, h, weight"),
            ({"u": str(u), "h": str(h), "weight": w - 2}, "stated weight"),   # claims a lighter cycle
            ({"u": str(u), "h": str(h), "weight": w + 1}, "stated weight"),
            ({"u": str(u), "h": str(h), "weight": str(w)}, "stated weight"),
            ({"u": str(u), "h": str(h), "weight": True}, "stated weight"),
        ]
        for nv, needle in cases:
            with self.subTest(nv=nv):
                self.setUp()
                self.edit_json(lambda r: r.__setitem__("nonvacuity", nv))
                self.rejects(needle)

    def test_12_boundary_shifted_nonvacuity_accepted(self):
        """u + (a boundary) is in the same nontrivial class: still accepted."""
        with open(self.path("ranks.json")) as f:
            r = json.load(f)
        u = int(r["nonvacuity"]["u"]) ^ BASE.d2[3]
        self.edit_json(lambda r: r["nonvacuity"].update({"u": str(u), "weight": popcount(u)}))
        code, out = self.verdict()
        self.assertEqual(code, 0, out)

    def test_13_structure(self):
        def swap_face(t):
            rows = t.split("\n")[:-1]
            a = rows[0].split()
            a[1:4] = rows[1].split()[1:4]
            return "\n".join([" ".join(a)] + rows[1:]) + "\n"
        self.edit("structure.txt", swap_face)
        self.rejects("listed face")
        self.setUp()
        self.edit("structure.txt", lambda t: t.replace(" equal\n", " kernel\n", 1))
        self.rejects("misreported")
        self.setUp()
        self.edit("structure.txt", lambda t: "\n".join(t.split("\n")[1:]))
        self.rejects("one line per nonzero")


class Independence(unittest.TestCase):
    SOURCE = r"allBaseCycles10|BaseCycleCensus|baseBoundaryWindow|baseFloorPairs|pinned_read|bb-distance|\.lean"

    def code(self, name):
        with open(os.path.join(HERE, name)) as f:
            text = re.sub(r'"""(.*?)"""', "", f.read(), flags=re.S)
        return re.sub(r"#[^\n]*", "", text)

    def test_no_source_reads(self):
        for name in ("witness.py", "check_witness.py"):
            self.assertIsNone(re.search(self.SOURCE, self.code(name)), name)

    def test_imports(self):
        imports = lambda n: sorted(set(re.findall(r"^\s*(?:import|from)\s+(\w+)", self.code(n), re.M)))
        self.assertEqual(imports("check_witness.py"), ["hashlib", "json", "re", "sys"])
        self.assertEqual(imports("witness.py"), ["argparse", "hashlib", "json", "os", "sys"])
        for dynamic in ("importlib", "__import__", "exec(", "eval(", "runpy"):
            self.assertNotIn(dynamic, self.code("check_witness.py"))


@unittest.skipUnless(os.environ.get("QEC_SRC"), "QEC_SRC unset: source tests skipped")
class Source(unittest.TestCase):
    def test_correspondence(self):
        p = run("compare_002b.py", os.environ["QEC_SRC"], os.path.join(HERE, "W.txt"), Z7)
        self.assertEqual(p.returncode, 0, p.stdout + p.stderr)
        self.assertIn("A  indexed columns: EQUAL", p.stdout)
        self.assertIn("-> VERIFIED", p.stdout)
        self.assertIn("C  diagnostic", p.stdout)

    def test_coefficient_equality_not_required(self):
        with open(os.path.join(HERE, "ranks.json")) as f:
            kvec = int(json.load(f)["d2"]["kernel"][0])
        tmp = tempfile.mkdtemp()
        try:
            with open(os.path.join(HERE, "W.txt")) as f, open(os.path.join(tmp, "W.txt"), "w") as g:
                g.write("".join(f"{u} {int(c) ^ (kvec if u != '0' else 0)}\n"
                                for u, c in (l.split() for l in f)))
            p = run("compare_002b.py", os.environ["QEC_SRC"], os.path.join(tmp, "W.txt"), Z7)
            self.assertEqual(p.returncode, 0, p.stdout + p.stderr)
        finally:
            shutil.rmtree(tmp)

    def test_altered_census_detected(self):
        tmp = tempfile.mkdtemp()
        try:
            with open(Z7) as f, open(os.path.join(tmp, "Z7.txt"), "w") as g:
                g.write("".join(f.readlines()[:-1]))
            p = run("compare_002b.py", os.environ["QEC_SRC"], os.path.join(HERE, "W.txt"),
                    os.path.join(tmp, "Z7.txt"))
            self.assertEqual(p.returncode, 1, p.stdout)
            self.assertIn("census set DIFFERENT", p.stdout)
        finally:
            shutil.rmtree(tmp)


if __name__ == "__main__":
    unittest.main(verbosity=2)

#!/usr/bin/env python3
"""Tests for RLT-QEC-002a. Run: python3 -I research/qec-002a/test_qec002a.py
Set QEC_SRC to a bb-distance-lean checkout containing the pinned commit to
also run the source-comparison tests (skipped otherwise)."""
import hashlib
import importlib.util
import os
import random
import re
import subprocess
import sys
import tempfile
import unittest
from itertools import combinations

sys.dont_write_bytecode = True   # keep the research directory free of build output
HERE = os.path.dirname(os.path.abspath(__file__))


def load(name):
    spec = importlib.util.spec_from_file_location(name, os.path.join(HERE, name + ".py"))
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


census, check, compare = load("census"), load("check"), load("compare")
BASE = census.Complex(**census.BASE)
COMMITTED = os.path.join(HERE, "Z7.txt")


def run(script, *args):
    return subprocess.run([sys.executable, "-I", os.path.join(HERE, script), *args],
                          capture_output=True, text=True)


def brute_force(cx, wmax):
    out = []
    for w in range(wmax + 1):
        for S in combinations(range(cx.n), w):
            s, mk = 0, 0
            for k in S:
                s ^= cx.column[k]
                mk |= 1 << k
            if s == 0:
                out.append(mk)
    return sorted(out)


class Encoding(unittest.TestCase):
    def test_registered_values(self):
        # face(x, y) = 3((-x) mod 15) + ((-y) mod 3); qubit = 45 j + face
        self.assertEqual(BASE.face(0, 0), 0)
        self.assertEqual(BASE.face(1, 0), 42)
        self.assertEqual(BASE.face(14, 0), 3)
        self.assertEqual(BASE.face(0, 1), 2)
        self.assertEqual(BASE.face(0, 2), 1)
        self.assertEqual(BASE.qubit(0, 0, 1), 45)
        self.assertEqual(BASE.qubit(1, 1, 1), 45 + 42 + 2)

    def test_bijection_and_inverse(self):
        seen = {BASE.qubit(x, y, j) for x in range(15) for y in range(3) for j in (0, 1)}
        self.assertEqual(seen, set(range(90)))
        for k in range(90):
            self.assertEqual(BASE.qubit(*BASE.coords(k)), k)

    def test_two_implementations_agree(self):
        for k in range(90):
            self.assertEqual(check.encode(check.decode(1 << k)), 1 << k)
            x, y, j = BASE.coords(k)
            self.assertEqual(check.decode(1 << k)[j][(x, y)], 1)


class BoundaryMap(unittest.TestCase):
    def test_columns_equal_convolution_definition(self):
        # census.py builds columns from supports; check.py evaluates the
        # convolution sum. Every unit vector must give the same syndrome.
        for k in range(90):
            s = check.d1(check.decode(1 << k))
            mask = 0
            for (x, y), bit in s.items():
                if bit:
                    mask |= 1 << BASE.face(x, y)
            self.assertEqual(mask, BASE.column[k], f"qubit {k}")

    def test_column_weight_three(self):
        self.assertTrue(all(bin(c).count("1") == 3 for c in BASE.column))

    def test_translation_equivariance(self):
        rng = random.Random(20261125)
        for _ in range(40):
            u = rng.getrandbits(90)
            t = (rng.randrange(15), rng.randrange(3))
            halves = check.decode(u)
            lhs = check.d1(check.translate(halves, t))
            d = check.d1(halves)
            rhs = {((x + t[0]) % 15, (y + t[1]) % 3): d[(x, y)] for (x, y) in check.GROUP}
            self.assertEqual(lhs, rhs)
            self.assertEqual(BASE.translate(u, t), check.encode(check.translate(halves, t)))


class ToyCompleteness(unittest.TestCase):
    """Both routes against brute force on small bicycle complexes, including
    odd weights, non-free translation orbits and odd/even weight bounds."""
    CASES = [
        (3, 3, [(1, 0), (0, 1), (0, 0)], [(0, 0), (2, 0), (0, 2)], 7),
        (4, 2, [(1, 0), (0, 1), (2, 1)], [(0, 0), (1, 1), (3, 0)], 7),
        (3, 2, [(0, 0), (1, 0)], [(0, 0), (1, 0)], 6),
        (5, 1, [(1, 0), (2, 0)], [(0, 0), (3, 0)], 5),
        (4, 3, [(1, 0), (0, 1), (2, 2)], [(0, 0), (1, 2), (3, 1)], 7),
        (2, 2, [(0, 0), (1, 1)], [(1, 0), (0, 1)], 8),
    ]

    def test_routes_equal_brute_force(self):
        nontrivial = 0
        for (l, m, A, B, wmax) in self.CASES:
            cx = census.Complex(l, m, A, B)
            expected = brute_force(cx, wmax)
            for w in range(wmax + 1):
                with self.subTest(l=l, m=m, wmax=w):
                    bf = [u for u in expected if bin(u).count("1") <= w]
                    self.assertEqual(census.route1(cx, w), bf)
                    self.assertEqual(census.route2(cx, w), bf)
            nontrivial += len(expected) > 1
        self.assertGreaterEqual(nontrivial, 4, "toy cases should contain cycles")

    def test_toy_cases_exercise_hard_features(self):
        odd, nonfree = False, False
        for (l, m, A, B, wmax) in self.CASES:
            cx = census.Complex(l, m, A, B)
            zs = brute_force(cx, wmax)
            odd |= any(bin(u).count("1") % 2 for u in zs)
            for u in zs:
                orbit = {cx.translate(u, (a, b)) for a in range(l) for b in range(m)}
                nonfree |= u != 0 and len(orbit) < l * m
        self.assertTrue(odd, "some toy cycle has odd weight")
        self.assertTrue(nonfree, "some toy cycle has a non-free translation orbit")


class Census(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.r1 = census.route1(BASE, 7)
        cls.r2 = census.route2(BASE, 7)

    def test_routes_agree(self):
        self.assertEqual(self.r1, self.r2)

    def test_result_shape(self):
        self.assertEqual(len(self.r1), 46)
        self.assertEqual(sorted(bin(u).count("1") for u in self.r1), [0] + [6] * 45)

    def test_committed_list_is_reproduced(self):
        with open(COMMITTED) as f:
            self.assertEqual(f.read(), "".join(f"{u}\n" for u in self.r1))

    def test_cli_output_and_hash(self):
        p = run("census.py")
        self.assertEqual(p.returncode, 0, p.stderr)
        with open(COMMITTED) as f:
            self.assertEqual(p.stdout, f.read())
        self.assertIn(hashlib.sha256(p.stdout.encode()).hexdigest(), p.stderr)


class Checker(unittest.TestCase):
    def setUp(self):
        with open(COMMITTED) as f:
            self.masks = [int(s) for s in f.read().split()]

    def verdict(self, lines, *extra, raw=None):
        with tempfile.NamedTemporaryFile("w", suffix=".txt", delete=False) as f:
            f.write(raw if raw is not None else "".join(f"{x}\n" for x in lines))
        try:
            return run("check.py", f.name, *extra).returncode
        finally:
            os.unlink(f.name)

    def test_accepts_committed_list(self):
        self.assertEqual(self.verdict(self.masks), 0)

    def face_mask(self, g):
        """Face generator d2 delta_g = (A*delta_g, B*delta_g), built here."""
        halves = ({h: 0 for h in check.GROUP}, {h: 0 for h in check.GROUP})
        for j, poly in ((0, check.A_EXP), (1, check.B_EXP)):
            for (a, b) in poly:
                halves[j][((g[0] + a) % 15, (g[1] + b) % 3)] = 1
        return check.encode(halves)

    def test_weight_six_entries_are_the_face_generators(self):
        faces = {self.face_mask(g) for g in check.GROUP}
        self.assertEqual(len(faces), 45)
        self.assertEqual({u for u in self.masks if u}, faces)

    def test_rejections(self):
        m = self.masks
        heavy = self.face_mask((0, 0)) ^ self.face_mask((2, 0))   # disjoint faces: a weight-12 cycle
        self.assertEqual(bin(heavy).count("1"), 12)
        orbit = sorted({self.face_mask(g) ^ self.face_mask(((g[0] + 2) % 15, g[1]))
                        for g in check.GROUP})
        cases = {
            "drop one (closure)": m[:5] + m[6:],
            "duplicate": m[:3] + [m[2]] + m[3:],
            "unsorted": [m[1], m[0]] + m[2:],
            "non-cycle": sorted(m + [1]),
            "too large": m + [1 << 90],
        }
        for name, lines in cases.items():
            with self.subTest(name):
                self.assertEqual(self.verdict(lines), 1)
        with self.subTest("weight above bound"):
            self.assertEqual(self.verdict(sorted(set(m) | set(orbit))), 1)
            self.assertEqual(self.verdict(sorted(set(m) | set(orbit)), "--wmax", "12"), 0)
        with self.subTest("leading zero"):
            raw = "0\n" + f"0{m[1]}\n" + "".join(f"{x}\n" for x in m[2:])
            self.assertEqual(self.verdict(None, raw=raw), 1)
        with self.subTest("missing final newline"):
            self.assertEqual(self.verdict(None, raw="\n".join(map(str, m))), 1)


class Independence(unittest.TestCase):
    """census.py and check.py read no source data and share no code."""
    SOURCE_NAMES = r"allBaseCycles10|BaseCycleCensus|baseBoundaryWindow|cycleSyndrome|pinned_read|bb-distance|\.lean"

    def test_no_source_references(self):
        for name in ("census.py", "check.py"):
            with open(os.path.join(HERE, name)) as f:
                code = re.sub(r'"""(.*?)"""', "", f.read(), flags=re.S)
            code = re.sub(r"#[^\n]*", "", code)
            self.assertIsNone(re.search(self.SOURCE_NAMES, code), name)

    def test_checker_imports_nothing_from_census(self):
        with open(os.path.join(HERE, "check.py")) as f:
            imports = re.findall(r"^\s*(?:import|from)\s+(\w+)", f.read(), re.M)
        self.assertEqual(sorted(set(imports)), ["re", "sys"])


@unittest.skipUnless(os.environ.get("QEC_SRC"), "QEC_SRC unset: source comparison skipped")
class SourceComparison(unittest.TestCase):
    def setUp(self):
        self.src = os.environ["QEC_SRC"]

    def compare(self, masks):
        with tempfile.NamedTemporaryFile("w", suffix=".txt", delete=False) as f:
            f.write("".join(f"{u}\n" for u in masks))
        try:
            return run("compare.py", self.src, f.name)
        finally:
            os.unlink(f.name)

    def test_equal(self):
        p = run("compare.py", self.src, COMMITTED)
        self.assertEqual(p.returncode, 0, p.stdout + p.stderr)
        self.assertIn("EQUAL", p.stdout)

    def test_perturbations_detected(self):
        with open(COMMITTED) as f:
            m = [int(s) for s in f.read().split()]
        self.assertEqual(self.compare(m[1:]).returncode, 1)
        self.assertEqual(self.compare(m + [m[-1] ^ 1]).returncode, 1)

    def test_unreflected_encoding_detected_despite_equal_weights(self):
        """Revision 3 §7.1: weight distributions cannot detect a reflection
        error; mask-set equality does."""
        with open(COMMITTED) as f:
            m = [int(s) for s in f.read().split()]
        def unreflect(u):
            out = 0
            for k in range(90):
                if (u >> k) & 1:
                    x, y, j = BASE.coords(k)
                    out |= 1 << (45 * j + 3 * x + y)
            return out
        alt = sorted(unreflect(u) for u in m)
        self.assertEqual(sorted(bin(u).count("1") for u in alt),
                         sorted(bin(u).count("1") for u in m))
        p = self.compare(alt)
        self.assertEqual(p.returncode, 1, p.stdout)
        self.assertIn("DIFFERENT", p.stdout)


if __name__ == "__main__":
    unittest.main(verbosity=2)

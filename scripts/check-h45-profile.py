#!/usr/bin/env python3
"""RLT-007a (H1): independent check of the H45 residue tables.

Checks certificates/h45/residues.tsv and certificates/h45/profile.tsv against
the reviewed transcription (sources/transcriptions/AXIOM-BGP212-2026-09-03.yaml)
by plain integer arithmetic, independently of scripts/gen-rlt006.py:

  residues.tsv  exactly 14 x 45 rows, in transcription order: for each listed
                prime p (with its tabled class r_p) every element h of H45,
                once, with h = p*q + s, 0 <= s < p, and omits_r_p = yes, which
                must hold (s != r_p);
  profile.tsv   exactly one row per listed prime, in order, with r_p equal to
                the table; occupied classes = { h mod p : h in H45 } (sorted,
                no repeats); occupied_count = their number; omitted classes =
                the sorted complement in 0..p-1; r_p among the omitted.

The tables are computed evidence, not RLT theorems; this script checks that
they say exactly what the transcription implies.

Usage: check-h45-profile.py [ROOT]            check the committed tables
       check-h45-profile.py --self-test [ROOT] apply each altered-table case to
                                               a scratch copy; each must fail
Exit 0 when every check holds (or every altered case is refused).
"""
import os
import re
import sys
import tempfile

import yaml


def load_transcription(root):
    t = yaml.safe_load(open(os.path.join(root, "sources/transcriptions/AXIOM-BGP212-2026-09-03.yaml"), encoding="utf-8"))
    items = {it["id"]: it for it in t["items"]}
    return items["h45-tuple"]["numbers"], items["omitted-primes"]["numbers"], items["omitted-classes"]["numbers"]


def rows(path):
    """Data rows of a table. Strict layout (RLT-007a audit note 10): LF line
    ends only, comment lines only before the first data row, no blank lines,
    and a final newline. A layout problem raises ValueError."""
    out = []
    text = open(path, encoding="utf-8", newline="").read()
    if "\r" in text:
        raise ValueError(f"{os.path.basename(path)}: carriage return in the file")
    if text and not text.endswith("\n"):
        raise ValueError(f"{os.path.basename(path)}: no final newline")
    for n, line in enumerate(text.split("\n")[:-1], 1):
        if line.startswith("#"):
            if out:
                raise ValueError(f"{os.path.basename(path)} line {n}: a comment after the first data row")
            continue
        if not line:
            raise ValueError(f"{os.path.basename(path)} line {n}: blank line")
        out.append(line.split("\t"))
    return out


_NUM = re.compile(r"(0|[1-9][0-9]*)")


def num(x):
    """A natural number written in plain ASCII decimal, no sign, no leading
    zero, no underscore, no other digit script (audit note 10); else ValueError."""
    if not _NUM.fullmatch(x):
        raise ValueError(f"not a plain decimal numeral: {x!r}")
    return int(x)


def ints(field):
    return [num(x) for x in field.split(",")] if field else []


def check(root, residues, profile):
    """Return a list of problems (empty when every check holds)."""
    H, P, R = load_transcription(root)
    problems = []
    # The shape is pinned here as well as read from the transcription.
    if (len(H), len(P), len(R)) != (45, 14, 14):
        return [f"the transcription gives {len(H)} elements and {len(P)} primes / {len(R)} classes; 45, 14 and 14 are pinned"]
    try:
        res = rows(residues)
    except ValueError as e:
        return [f"residues.tsv layout: {e}"]
    want_keys = [(p, h) for p in P for h in H]
    if len(res) != len(want_keys):
        problems.append(f"residues.tsv has {len(res)} rows; expected {len(want_keys)} (14 primes x 45 elements)")
    for i, row in enumerate(res):
        if len(row) != 6:
            problems.append(f"residues.tsv row {i + 1}: {len(row)} fields")
            continue
        try:
            p, r, h, q, s = (num(x) for x in row[:5])
        except ValueError:
            problems.append(f"residues.tsv row {i + 1}: non-integer field")
            continue
        o = row[5]
        if i < len(want_keys) and (p, h) != want_keys[i]:
            problems.append(f"residues.tsv row {i + 1}: (p, h) = ({p}, {h}); expected {want_keys[i]}")
        if p in P and r != R[P.index(p)]:
            problems.append(f"residues.tsv row {i + 1}: r_p = {r}; the table gives {R[P.index(p)]} for p = {p}")
        if h != p * q + s or not 0 <= s < p:
            problems.append(f"residues.tsv row {i + 1}: {h} != {p}*{q} + {s} with 0 <= {s} < {p}")
        if o != "yes" or s == r:
            problems.append(f"residues.tsv row {i + 1}: omits_r_p = {o!r}, s = {s}, r_p = {r}")
    try:
        prof = rows(profile)
    except ValueError as e:
        return problems + [f"profile.tsv layout: {e}"]
    if len(prof) != len(P):
        problems.append(f"profile.tsv has {len(prof)} rows; expected {len(P)}")
    for i, row in enumerate(prof):
        if len(row) != 5:
            problems.append(f"profile.tsv row {i + 1}: {len(row)} fields")
            continue
        try:
            p, r, n = num(row[0]), num(row[1]), num(row[2])
            occ, om = ints(row[3]), ints(row[4])
        except ValueError:
            problems.append(f"profile.tsv row {i + 1}: non-integer field")
            continue
        if i < len(P) and (p, r) != (P[i], R[i]):
            problems.append(f"profile.tsv row {i + 1}: (p, r_p) = ({p}, {r}); the table gives ({P[i]}, {R[i]})")
        true_occ = sorted({h % p for h in H})
        true_om = [c for c in range(p) if c not in set(true_occ)]
        if occ != true_occ:
            problems.append(f"profile.tsv p = {p}: occupied classes differ from H45 mod {p}")
        if n != len(true_occ) or n != len(occ):
            problems.append(f"profile.tsv p = {p}: occupied_count {n}; H45 occupies {len(true_occ)}")
        if om != true_om:
            problems.append(f"profile.tsv p = {p}: omitted classes are not the complement of the occupied classes")
        if r not in om:
            problems.append(f"profile.tsv p = {p}: r_p = {r} is not listed as omitted")
    return problems


# Altered-table cases for --self-test: (name, file, edit on the list of lines).
def _edit_field(lines, row_pred, col, value):
    out, done = [], False
    for l in lines:
        f = l.rstrip("\n").split("\t")
        if not done and not l.startswith("#") and row_pred(f):
            f[col] = value
            l = "\t".join(f) + "\n"
            done = True
        out.append(l)
    assert done
    return out


def _drop(lines, row_pred):
    out, done = [], False
    for l in lines:
        if not done and not l.startswith("#") and row_pred(l.rstrip("\n").split("\t")):
            done = True
            continue
        out.append(l)
    assert done
    return out


def _dup(lines, row_pred):
    out = []
    for l in lines:
        out.append(l)
        if not l.startswith("#") and row_pred(l.rstrip("\n").split("\t")):
            out.append(l)
    return out


CASES = [
    ("residues: a quotient changed", "residues.tsv", lambda L: _edit_field(L, lambda f: f[:3] == ["3", "1", "12"], 3, "5")),
    ("residues: a remainder changed", "residues.tsv", lambda L: _edit_field(L, lambda f: f[:3] == ["3", "1", "12"], 4, "2")),
    ("residues: an omits flag changed", "residues.tsv", lambda L: _edit_field(L, lambda f: f[:3] == ["3", "1", "12"], 5, "no")),
    ("residues: a row dropped", "residues.tsv", lambda L: _drop(L, lambda f: f[:3] == ["43", "3", "212"])),
    ("residues: a row duplicated", "residues.tsv", lambda L: _dup(L, lambda f: f[:3] == ["43", "3", "210"])),
    ("residues: a row dropped and another duplicated", "residues.tsv",
     lambda L: _dup(_drop(L, lambda f: f[:3] == ["43", "3", "212"]), lambda f: f[:3] == ["43", "3", "210"])),
    ("residues: h changed with q kept consistent (12 -> 14, q 6 -> 7)", "residues.tsv",
     lambda L: _edit_field(_edit_field(L, lambda f: f[:3] == ["2", "1", "12"], 2, "14"), lambda f: f[:3] == ["2", "1", "14"] and f[3] == "6", 3, "7")),
    ("residues: r_p differs from the table", "residues.tsv", lambda L: [l.replace("29\t4\t", "29\t13\t") for l in L]),
    ("profile: an occupied count changed", "profile.tsv", lambda L: _edit_field(L, lambda f: f[0] == "13", 2, "11")),
    ("profile: an occupied class removed", "profile.tsv", lambda L: _edit_field(L, lambda f: f[0] == "3", 3, "0")),
    ("profile: an occupied class also listed as omitted", "profile.tsv", lambda L: _edit_field(L, lambda f: f[0] == "3", 4, "0,1")),
    ("profile: an omitted class dropped", "profile.tsv", lambda L: _edit_field(L, lambda f: f[0] == "29", 4, "4")),
    ("profile: an omitted class changed", "profile.tsv", lambda L: _edit_field(L, lambda f: f[0] == "29", 4, "4,14")),
    ("profile: r_p changed", "profile.tsv", lambda L: _edit_field(L, lambda f: f[0] == "43", 1, "9")),
    ("profile: a row dropped", "profile.tsv", lambda L: _drop(L, lambda f: f[0] == "41")),
    # Audit note 10: the same values written another way.
    ("residues: a remainder written +0_6", "residues.tsv", lambda L: _edit_field(L, lambda f: f[:3] == ["7", "6", "12"], 4, "+0_5")),
    ("residues: a remainder with a leading zero", "residues.tsv", lambda L: _edit_field(L, lambda f: f[:3] == ["7", "6", "12"], 4, "05")),
    ("residues: an Arabic-Indic digit", "residues.tsv", lambda L: _edit_field(L, lambda f: f[:3] == ["2", "1", "0"], 3, "\u0660")),
    ("profile: CRLF line ends", "profile.tsv", lambda L: [l.replace("\n", "\r\n") for l in L]),
    ("profile: a comment line between data rows", "profile.tsv", lambda L: L[:5] + ["# inserted\n"] + L[5:]),
    ("residues: a blank line", "residues.tsv", lambda L: L[:10] + ["\n"] + L[10:]),
]


def main():
    args = sys.argv[1:]
    self_test = "--self-test" in args
    args = [a for a in args if a != "--self-test"]
    root = args[0] if args else os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    res = os.path.join(root, "certificates/h45/residues.tsv")
    prof = os.path.join(root, "certificates/h45/profile.tsv")
    base = check(root, res, prof)
    if not self_test:
        for p in base:
            print("PROBLEM  " + p)
        print(f"certificates/h45: {'all checks hold' if not base else str(len(base)) + ' problem(s)'} "
              "(computed evidence checked against the transcription; not kernel-checked)")
        if not base:
            # The same summary line verify.sh printed before RLT-007a.
            pr, nres = rows(prof), len(rows(res))
            print(f"summary {nres} rows with h = p*q + s and s != r_p; "
                  + ", ".join(f"nu_{r[0]}={r[2]}" for r in pr))
        sys.exit(0 if not base else 1)
    if base:
        print("self-test needs the committed tables to pass first")
        sys.exit(1)
    bad = 0
    with tempfile.TemporaryDirectory() as tmp:
        for name, which, edit in CASES:
            r2, p2 = os.path.join(tmp, "residues.tsv"), os.path.join(tmp, "profile.tsv")
            for src, dst in ((res, r2), (prof, p2)):
                open(dst, "w", encoding="utf-8").write(open(src, encoding="utf-8").read())
            target = r2 if which == "residues.tsv" else p2
            lines = open(target, encoding="utf-8").readlines()
            open(target, "w", encoding="utf-8").writelines(edit(lines))
            found = check(root, r2, p2)
            print(f"{'refused ' if found else 'ACCEPTED'} {name}")
            bad += 0 if found else 1
    print(f"self-test: {len(CASES) - bad} of {len(CASES)} altered tables refused")
    sys.exit(0 if bad == 0 else 1)


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""RLT-007d independent composition checker (Python; shares no matching code with Rust).

Usage: compose-check.py RLT_BINARY BUNDLE [--quiet] [--root DIR]

What it uses from Rust: only `rlt check FILE`, i.e. kernel acceptance and the
kernel's printed formulas (the trusted base). Everything else is implemented
here, separately from cli/src/compose.rs:

  * the bundle reader and every structural rule (same refusal classes);
  * a parser for the kernel's printed formulas and a de Bruijn canonical form;
  * the approved statements, rebuilt from the source transcription
    (sources/transcriptions/AXIOM-BGP212-2026-09-03.yaml) and the approved
    interval table: every node whose id is approved must state exactly that
    formula, independently of the specification probes the Rust checker uses;
  * the approved axiom set, convention and definitions pins (constants here).

Exit codes match `rlt compose`: 0 composed, 1 refused, 2 malformed or
missing, 3 stale. The last line is `verdict COMPOSED` or
`verdict REFUSED (class) — reason`.
"""
import concurrent.futures
import hashlib
import os
import re
import subprocess
import tempfile
import sys

import yaml

# ---------------------------------------------------------------------------
# Approved constants (stated here, not read from the Rust sources).
APPROVED_AXIOMS = {"ROBINSON-Q": "4ad55192222576e3901abc4ee7ea5a6760b64ff212e8c55796ab9847e423b00c"}
APPROVED_CONVENTION = "NAT-DEF"
APPROVED_DEFS = ("spec/definitions.rlt", "122e37fc627b136023a1a5ffac0344162ed77ae95312248a9f6a76c2b639ecda")
APPROVED_AXIOM_FILE = "spec/robinson-q.rlt"
# NAT-DEF's interpretation and explanation digests: SHA-256 of the canonical
# text (below) of conventions/nat-def.yaml without, and of, its explanation.
APPROVED_CONVENTION_DIGESTS = ("935f9635e563cab1d60cd4b5d7ffbed121933eb59e790edd9fb7c21d088bd208",
                               "a6a720b2d61f91e92bfaaf86f2947ec7bcab5a267d83a68c1766898c213b0524")
INTERVALS = [(0, 40), (41, 78), (79, 112), (113, 156), (157, 190), (191, 212)]
TRANSCRIPTION = "sources/transcriptions/AXIOM-BGP212-2026-09-03.yaml"
CODES = {"malformed": 2, "missing": 2, "stale": 3}


class Refuse(Exception):
    def __init__(self, cls, msg):
        super().__init__(msg)
        self.cls, self.msg = cls, msg


def sha_file(path):
    with open(path, "rb") as f:
        return hashlib.sha256(f.read()).hexdigest()


# ---------------------------------------------------------------------------
# Formulas: AST tuples. Terms: ("v", name) ("n", k) ("S", t) ("+", a, b) ("*", a, b).
# Formulas: ("=", a, b) ("not", f) ("and", f, g) ("or", f, g) ("imp", f, g)
#           ("all", x, f) ("ex", x, f).
def num(k):
    return ("n", k)


def S(t):
    return ("n", t[1] + 1) if t[0] == "n" else ("S", t)


TOK = re.compile(r"\s*(⊥|∀|∃|¬|∧|∨|→|=|\+|×|\(|\)|\.|S\(|[0-9]+|[A-Za-z_][A-Za-z_0-9'′]*)")


def tokens(s):
    out, i = [], 0
    s = s.rstrip()
    while i < len(s):
        m = TOK.match(s, i)
        if not m or m.end() == i:
            raise ValueError(f"cannot read {s[i:i + 20]!r}")
        out.append(m.group(1))
        i = m.end()
    return out


class Parser:
    """The kernel's printed form: connectives parenthesized, binders reach to
    the end of their group, numerals for S^n(0)."""

    def __init__(self, s):
        self.t, self.i = tokens(s), 0

    def peek(self):
        return self.t[self.i] if self.i < len(self.t) else None

    def eat(self, x=None):
        tk = self.peek()
        if tk is None or (x is not None and tk != x):
            raise ValueError(f"expected {x!r}, got {tk!r}")
        self.i += 1
        return tk

    def formula(self):
        left = self.unary()
        op = self.peek()
        if op in ("∧", "∨", "→"):
            self.eat()
            right = self.unary()
            if self.peek() in ("∧", "∨", "→"):
                raise ValueError("unparenthesized chain of connectives")
            return ({"∧": "and", "∨": "or", "→": "imp"}[op], left, right)
        return left

    def unary(self):
        tk = self.peek()
        if tk in ("∀", "∃"):
            self.eat()
            x = self.eat()
            self.eat(".")
            return ("all" if tk == "∀" else "ex", x, self.formula())
        if tk == "¬":
            self.eat()
            return ("not", self.unary())
        if tk == "⊥":
            self.eat()
            return ("false",)
        if tk == "(":
            save = self.i
            try:
                self.eat("(")
                f = self.formula()
                self.eat(")")
                if self.peek() in ("=", "+", "×"):
                    raise ValueError("a term")
                return f
            except ValueError:
                self.i = save
        a = self.term()
        self.eat("=")
        return ("=", a, self.term())

    def term(self):
        a = self.factor()
        op = self.peek()
        if op in ("+", "×"):
            self.eat()
            b = self.factor()
            if self.peek() in ("+", "×"):
                raise ValueError("unparenthesized chain of operations")
            return ("+" if op == "+" else "*", a, b)
        return a

    def factor(self):
        tk = self.eat()
        if tk == "S(":
            t = self.term()
            self.eat(")")
            return S(t)
        if tk == "(":
            t = self.term()
            self.eat(")")
            return t
        if tk.isdigit():
            return num(int(tk))
        if re.fullmatch(r"[A-Za-z_][A-Za-z_0-9'′]*", tk) and tk != "S":
            return ("v", tk)
        raise ValueError(f"unexpected {tk!r}")


def parse(s):
    p = Parser(s)
    f = p.formula()
    if p.peek() is not None:
        raise ValueError(f"trailing {p.peek()!r}")
    return f


def canon(f, env=()):
    """De Bruijn form: a bound variable becomes the distance to its binder."""
    k = f[0]
    if k in ("all", "ex"):
        return (k, canon(f[2], (f[1],) + env))
    if k in ("and", "or", "imp"):
        return (k, canon(f[1], env), canon(f[2], env))
    if k == "not":
        return (k, canon(f[1], env))
    if k == "false":
        return f
    if k == "=":
        return (k, cterm(f[1], env), cterm(f[2], env))
    raise ValueError(k)


def cterm(t, env):
    k = t[0]
    if k == "v":
        return ("b", env.index(t[1])) if t[1] in env else ("free", t[1])
    if k == "n":
        return t
    if k == "S":
        u = cterm(t[1], env)
        return ("n", u[1] + 1) if u[0] == "n" else ("S", u)
    return (k, cterm(t[1], env), cterm(t[2], env))


# ---------------------------------------------------------------------------
# The approved statements, rebuilt from the transcription.
def approved():
    T = yaml.safe_load(open(TRANSCRIPTION, encoding="utf-8"))
    it = {i["id"]: i for i in T["items"]}
    H = list(it["h45-tuple"]["numbers"])
    cls = dict(zip(it["omitted-primes"]["numbers"], it["omitted-classes"]["numbers"]))
    primes = [p for p in range(2, 213) if all(p % d for d in range(2, p))]
    for p in primes:
        if p > 43:
            cls[p] = next(c for c in range(p) if all(h % p != c for h in H))
    assert len(H) == 45 and sorted(cls) == primes
    v = lambda x: ("v", x)

    def lt(a, b):  # NAT-DEF: a < b  :≡  ∃z. S(z) + a = b
        return ("ex", "z", ("=", ("+", ("S", v("z")), a), b))

    def dvd(d, n):  # NAT-DEF: d ∣ n  :≡  ∃k. n = d × k
        return ("ex", "k", ("=", n, ("*", d, v("k"))))

    def lconj(fs):
        f = fs[0]
        for g in fs[1:]:
            f = ("and", f, g)
        return f

    def rdisj(fs):
        f = fs[-1]
        for g in reversed(fs[:-1]):
            f = ("or", g, f)
        return f

    def om(p, r):
        return lconj([("not", ("ex", "k", ("=", num(h), ("+", ("*", p, v("k")), r)))) for h in H])

    def adm(p):
        return ("ex", "r", ("and", lt(v("r"), p), om(p, v("r"))))

    def pb(p):
        return ("and", lt(num(1), p),
                ("all", "d", ("imp", ("and", lt(num(1), v("d")), lt(v("d"), p)), ("not", dvd(v("d"), p)))))

    ranges = [[q for q in primes if a <= q <= b] for a, b in INTERVALS]
    rng = lambda qs, p: rdisj([("=", p, num(q)) for q in qs])
    P = v("p")
    A = {}
    for q, r in cls.items():
        A[f"om.{q}_{r}"] = om(num(q), num(r))
    A["tail"] = ("all", "p", ("imp", lt(num(212), P), om(P, num(1))))
    A["classify"] = ("all", "p", ("imp", pb(P), rdisj([rng(qs, P) for qs in ranges] + [lt(num(212), P)])))
    groups = [range(0, 11), range(11, 22), range(22, 33), range(33, 44)]
    for k, g in enumerate(groups):
        A[f"distinct.g{k}"] = lconj([lconj([("not", ("=", num(H[i]), num(H[j]))) for j in range(i + 1, 45)]) for i in g])
    A["bounds"] = lconj([("and", ("ex", "z", ("=", ("+", v("z"), num(0)), num(h))),
                          ("ex", "z", ("=", ("+", v("z"), num(h)), num(212)))) for h in H])
    for i, qs in enumerate(ranges, 1):
        A[f"adm.range_{i}"] = ("all", "p", ("imp", rng(qs, P), adm(P)))
    A["adm.tail"] = ("all", "p", ("imp", lt(num(212), P), adm(P)))
    A["adm.all"] = ("all", "p", ("imp", pb(P), adm(P)))
    return {k: canon(f) for k, f in A.items()}


# ---------------------------------------------------------------------------
# Statement fingerprints, recomputed here (format rlt-statement-fingerprint/1).
def yaml_canon(y):
    """Canonical text of a YAML value: typed, length-prefixed, maps sorted."""
    if isinstance(y, bool):
        return "b1" if y else "b0"
    if isinstance(y, int):
        return f"i{y};"
    if isinstance(y, float):
        raise Refuse("incompatible", "conventions: a real number in a convention record")
    if isinstance(y, str):
        return f"s{len(y.encode('utf-8'))}:{y}"
    if y is None:
        return "n"
    if isinstance(y, list):
        return "[" + "".join(yaml_canon(x) for x in y) + "]"
    if isinstance(y, dict):
        items = sorted((yaml_canon(k), yaml_canon(v)) for k, v in y.items())
        return "{" + "".join(f"{k}={v}" for k, v in items) + "}"
    raise Refuse("incompatible", "conventions: an unreadable value")


def convention_digests(path="conventions/nat-def.yaml"):
    y = yaml.safe_load(open(path, encoding="utf-8"))
    interp = {k: v for k, v in y.items() if k != "explanation"}
    h = lambda t: hashlib.sha256(t.encode("utf-8")).hexdigest()
    return h(yaml_canon(interp)), h(yaml_canon(y["explanation"]))


def kterm(t):
    """A canonical term in the fingerprint's prefix form: symbol/arity."""
    k = t[0]
    if k == "b":
        return f"#{t[1]}"
    if k == "n":
        out = "(0/0)"
        for _ in range(t[1]):
            out = f"(S/1 {out})"
        return out
    if k == "S":
        return f"(S/1 {kterm(t[1])})"
    if k in ("+", "*"):
        return f"({k}/2 {kterm(t[1])} {kterm(t[2])})"
    raise ValueError(f"a free variable `{t[1]}` in a statement")


def kform(f):
    k = f[0]
    if k == "=":
        return f"(= {kterm(f[1])} {kterm(f[2])})"
    if k == "false":
        return "false"
    if k == "not":  # ¬A is A → ⊥ in the kernel
        return f"(imp {kform(f[1])} false)"
    if k in ("and", "or", "imp"):
        return f"({k} {kform(f[1])} {kform(f[2])})"
    if k in ("all", "ex"):
        return f"({'forall' if k == 'all' else 'exists'} {kform(f[1])})"
    raise ValueError(k)


def fingerprint(c):
    """SHA-256 of the statement's parts: convention, axiom set, definitions
    used, the checked goal and the assumptions, each from this run."""
    conv = convention_digests()[0]
    t = "rlt-statement-fingerprint/1\n"
    t += f"convention {c.convention} {conv}\n"
    t += f"axiom-set {c.axiom[0]} {APPROVED_AXIOM_FILE} {sha_file(APPROVED_AXIOM_FILE)}\n"
    t += (f"definitions {APPROVED_DEFS[0]} {sha_file(APPROVED_DEFS[0])} used {','.join(c.defs_used)}\n"
          if c.defs_used else "definitions none\n")
    t += f"goal {kform(canon(parse(c.proved)))}\n"
    asm = sorted({kform(canon(parse(g))) for g in c.gamma.values()})
    t += "assumptions none\n" if not asm else "".join(f"assumption {a}\n" for a in asm)
    return hashlib.sha256(t.encode("utf-8")).hexdigest()


# ---------------------------------------------------------------------------
# Kernel runs: `rlt check FILE`, read from its printed report.
class Check:
    def __init__(self, rlt, path):
        r = subprocess.run([rlt, "check", path], capture_output=True, text=True)
        self.code, self.out = r.returncode, r.stdout
        top = [l for l in self.out.splitlines() if l and not l.startswith(" ")]
        field = lambda name: [l[16:] for l in top if l.startswith(name.ljust(16))]
        self.verdict = (field("verdict") or [""])[0]
        self.proved = (field("proved") or [None])[0]
        self.cites = field("cites")
        self.declared = [l.split(" : ", 1)[0] for l in field("declared")]
        conv = (field("convention") or [""])[0]
        self.convention = conv.split(" ")[0]
        ax = (field("axiom set") or [""])[0].split(" ")[0]
        sha = [l[16:] for l in self.out.splitlines() if l.startswith("  sha256        ")]
        self.axiom = (ax, sha[0] if sha else "")
        d = (field("definitions") or [""])[0]
        m = re.search(r"\((\S+) sha256 ([0-9a-f]{64})\)", d)
        self.defs = (m.group(1), m.group(2)) if m else None
        self.defs_used = d.split(" — ", 1)[0].split(", ") if m else []
        g = (field("assumptions") or ["Γ = ∅"])[0]
        self.gamma = {}
        if g.strip() != "Γ = ∅":
            body = g.strip()
            assert body.startswith("Γ = { ") and body.endswith(" }"), body[:40]
            body = body[6:-2]
            parts = re.split(r", ([A-Za-z_][A-Za-z_0-9]*) : ", ", " + body)
            assert parts[0] == "", body[:40]
            for n, f in zip(parts[1::2], parts[2::2]):
                self.gamma[n] = f

    def accepted(self):
        return self.code == 0 and self.verdict.startswith("ACCEPT")

    def parts_problem(self):
        if self.axiom != ("ROBINSON-Q", APPROVED_AXIOMS["ROBINSON-Q"]):
            return f"axiom set {self.axiom[0]} is not the approved ROBINSON-Q"
        if self.convention != APPROVED_CONVENTION:
            return f"convention {self.convention} is not the approved NAT-DEF"
        if self.defs is not None and self.defs != APPROVED_DEFS:
            return f"definitions {self.defs[0]} are not the approved file"
        return None


# ---------------------------------------------------------------------------
def strict_yaml(text):
    """Plain YAML only: no anchors, aliases, explicit tags or repeated keys."""
    try:
        for ev in yaml.parse(text, Loader=yaml.SafeLoader):
            if isinstance(ev, yaml.AliasEvent) or getattr(ev, "anchor", None):
                raise Refuse("malformed", "bundle: anchors and aliases are not allowed")
            if isinstance(ev, yaml.ScalarEvent) and ev.tag is not None and ev.implicit == (False, False):
                raise Refuse("malformed", "bundle: explicit tags are not allowed")
            if isinstance(ev, (yaml.MappingStartEvent, yaml.SequenceStartEvent)) and ev.tag is not None and not ev.implicit:
                raise Refuse("malformed", "bundle: explicit tags are not allowed")

        def walk(n):
            if isinstance(n, yaml.MappingNode):
                keys = [k.value for k, _ in n.value]
                if len(keys) != len(set(keys)):
                    raise Refuse("malformed", "bundle: a key is repeated")
                for _, v in n.value:
                    walk(v)
            elif isinstance(n, yaml.SequenceNode):
                for v in n.value:
                    walk(v)
        walk(yaml.compose(text, Loader=yaml.SafeLoader))
        return yaml.safe_load(text)
    except yaml.YAMLError as e:
        raise Refuse("malformed", f"bundle: not YAML ({str(e).splitlines()[0]})")


PATH_OK = re.compile(r"[A-Za-z0-9_./-]+")


def path_ok(p):
    return isinstance(p, str) and not p.startswith("/") and PATH_OK.fullmatch(p) and all(c not in ("", ".", "..") for c in p.split("/"))


def regular_file(p, nid):
    """A pinned file: inside the repository, no symbolic link on the way."""
    parts = p.split("/")
    for i in range(1, len(parts)):
        if os.path.islink("/".join(parts[:i])):
            raise Refuse("malformed", f"node `{nid}`: {p}: a symbolic link on the path")
    if not os.path.isfile(p) or os.path.islink(p):
        raise Refuse("missing", f"node `{nid}`: {p}: file not found")


def register(path, n):
    if not os.path.isfile(path):
        raise Refuse("missing", f"{path}: file not found")
    out = {}
    for l in open(path, encoding="utf-8"):
        l = l.rstrip("\n")
        if not l or l.startswith("#"):
            continue
        f = l.split("\t")
        if len(f) != n or f[0] in out:
            raise Refuse("malformed", f"{path}: malformed line")
        out[f[0]] = f[1:]
    return out


def load(path):
    try:
        text = open(path, encoding="utf-8").read()
        if text.startswith("\ufeff"):
            raise Refuse("malformed", "bundle: a byte-order mark is not allowed")
        y = strict_yaml(text)
    except FileNotFoundError:
        raise Refuse("missing", f"{path}: file not found")
    top = ["format", "id", "axiom_set", "convention", "definitions", "nodes", "result"]
    if not isinstance(y, dict) or set(y) != set(top):
        extra = sorted(set(y or {}) - set(top)) if isinstance(y, dict) else []
        raise Refuse("malformed", f"bundle: unknown key `{extra[0]}`" if extra else "bundle: keys must be exactly " + ", ".join(top))
    for k in ("id", "axiom_set", "convention"):
        if not isinstance(y[k], str):
            raise Refuse("malformed", f"bundle: `{k}` must be a string")
    if y["format"] != "rlt-compose/1":
        raise Refuse("malformed", "bundle: format must be rlt-compose/1")

    def pin(x, ctx):
        if not isinstance(x, dict) or set(x) != {"path", "sha256"} or not all(isinstance(x[k], str) for k in x):
            raise Refuse("malformed", f"{ctx}: a pin has exactly `path` and `sha256` strings")
        if not path_ok(x["path"]):
            raise Refuse("malformed", f"{ctx}: not a repository-relative path")
        if not re.fullmatch(r"[0-9a-f]{64}", x["sha256"]):
            raise Refuse("malformed", f"{ctx}: sha256 must be 64 lower-case hex digits")
        return x

    nodes = []
    if not isinstance(y["nodes"], list):
        raise Refuse("malformed", "bundle: `nodes` must be a list")
    for i, n in enumerate(y["nodes"], 1):
        ctx = f"bundle: node {i}"
        if not isinstance(n, dict) or not {"id", "layer", "spec"} <= set(n) or set(n) - {"id", "layer", "library", "fingerprint", "assembly", "discharge", "spec"}:
            raise Refuse("malformed", f"{ctx}: wrong keys")
        if not isinstance(n["id"], str):
            raise Refuse("malformed", f"{ctx}: `id` must be a string")
        if not isinstance(n["layer"], int) or isinstance(n["layer"], bool) or not 0 <= n["layer"] < 64:
            raise Refuse("malformed", f"{ctx}: `layer` must be an integer 0 … 63")
        pin(n["spec"], ctx + " spec")
        if n["layer"] == 0:
            if "assembly" in n or "discharge" in n:
                raise Refuse("malformed", f"{ctx} (`{n['id']}`): a layer-0 node names a library theorem, not an assembly")
            if not isinstance(n.get("library"), str) or not isinstance(n.get("fingerprint"), str):
                raise Refuse("malformed", f"{ctx}: a layer-0 node has `library` and `fingerprint`")
            n["discharge"] = []
        else:
            if "library" in n or "fingerprint" in n:
                raise Refuse("malformed", f"{ctx}: a composed node names an assembly, not a library theorem")
            pin(n.get("assembly"), ctx + " assembly")
            if not isinstance(n.get("discharge"), list):
                raise Refuse("malformed", f"{ctx}: `discharge` must be a list")
            for d in n["discharge"]:
                if not isinstance(d, dict) or set(d) != {"assumption", "by"} or not all(isinstance(d[k], str) for k in d):
                    raise Refuse("malformed", f"{ctx}: a discharge row has `assumption` and `by`")
        nodes.append(n)
    if not isinstance(y["result"], list) or not all(isinstance(r, str) for r in y["result"]):
        raise Refuse("malformed", "bundle: `result` lists node ids")
    pin(y["definitions"], "bundle definitions")
    return y, nodes


def compose(rlt, bundle, report):
    y, nodes = load(bundle)
    if y["axiom_set"] not in APPROVED_AXIOMS:
        raise Refuse("incompatible", f"axiom set `{y['axiom_set']}` is not the approved one")
    if y["convention"] != APPROVED_CONVENTION:
        raise Refuse("incompatible", f"convention `{y['convention']}` is not the approved one")
    if (y["definitions"]["path"], y["definitions"]["sha256"]) != APPROVED_DEFS:
        raise Refuse("incompatible", "definitions are not the approved file")
    if sha_file(APPROVED_DEFS[0]) != APPROVED_DEFS[1]:
        raise Refuse("incompatible", "spec/definitions.rlt differs from the approved definitions")
    if sha_file(APPROVED_AXIOM_FILE) != APPROVED_AXIOMS["ROBINSON-Q"]:
        raise Refuse("incompatible", f"{APPROVED_AXIOM_FILE} differs from the approved axioms of ROBINSON-Q")
    if convention_digests() != APPROVED_CONVENTION_DIGESTS:
        raise Refuse("incompatible", "conventions/nat-def.yaml differs from the approved NAT-DEF")
    # The whole context as the checker loads it (every convention, the axiom
    # set, the definitions): a one-step probe written here must be accepted.
    with tempfile.TemporaryDirectory() as d:
        probe = os.path.join(d, "context.rlt")
        with open(probe, "w", encoding="utf-8") as f:
            f.write('(rlt-proof\n  (id RLT007D.CONTEXT)\n  (convention NAT-DEF)\n  (goal "0 < 1")\n  (assume s "0 < 1")\n'
                    '  (mode conditional)\n  (axioms ROBINSON-Q)\n  (steps\n    (G (lt 0 1) (hyp s))))\n')
        c = Check(rlt, probe)
        if not c.accepted():
            raise Refuse("incompatible", f"the checker refuses the context: {c.verdict[:120]}")

    ids = {}
    for n in nodes:
        if n["id"] in ids:
            raise Refuse("duplicated", f"node id `{n['id']}` appears twice")
        ids[n["id"]] = n
    seen_lib, seen_asm = {}, {}
    for n in nodes:
        if n["layer"] == 0:
            if n["library"] in seen_lib:
                raise Refuse("duplicated", f"library theorem `{n['library']}` named twice")
            seen_lib[n["library"]] = n["id"]
        else:
            if n["assembly"]["path"] in seen_asm:
                raise Refuse("duplicated", f"assembly {n['assembly']['path']} named twice")
            seen_asm[n["assembly"]["path"]] = n["id"]
    used = set()
    for n in nodes:
        rows, top = set(), 0
        for d in n["discharge"]:
            if d["assumption"] in rows:
                raise Refuse("duplicated", f"node `{n['id']}`: two rows for `{d['assumption']}`")
            rows.add(d["assumption"])
            if d["by"] not in ids:
                raise Refuse("missing", f"node `{n['id']}`: `{d['by']}` is not a node")
            if ids[d["by"]]["layer"] >= n["layer"]:
                raise Refuse("graph", f"node `{n['id']}`: `{d['by']}` is not of a lower layer")
            top = max(top, ids[d["by"]]["layer"])
            used.add(d["by"])
        if n["layer"] > 0 and not n["discharge"]:
            raise Refuse("graph", f"node `{n['id']}` discharges nothing")
        if n["layer"] > 0 and n["layer"] != top + 1:
            raise Refuse("graph", f"node `{n['id']}`: stated layer {n['layer']}, computed {top + 1}")
    if not y["result"]:
        raise Refuse("malformed", "bundle: `result` is empty")
    seen = set()
    for r in y["result"]:
        if r not in ids:
            raise Refuse("missing", f"result names `{r}`, not a node")
        if r in seen:
            raise Refuse("duplicated", f"result names `{r}` twice")
        seen.add(r)
        used.add(r)
    for n in nodes:
        if n["id"] not in used:
            raise Refuse("unused", f"node `{n['id']}` discharges no assumption and is not part of the result")

    entries = {}
    for n in nodes:
        if n["layer"] == 0:
            area = n["library"].split(".")[0]
            p = f"lib/{area}/{n['library']}.yaml"
            if not re.fullmatch(r"[a-z0-9_]+(\.[a-z0-9_]+)+", n["library"]) or not os.path.isfile(p):
                raise Refuse("missing", f"node `{n['id']}`: `{n['library']}` is not a library id")
            e = yaml.safe_load(open(p, encoding="utf-8"))
            idents = register("lib/identities.tsv", 3)
            fps = [v[0] for v in idents.values()]
            if len(fps) != len(set(fps)):
                raise Refuse("malformed", "lib/identities.tsv: one fingerprint under two ids")
            ident = idents.get(n["library"])
            if ident is None or ident[0] != e["fingerprint"] or (not e.get("history") and ident[1] != e["proof"]["sha256"]):
                raise Refuse("malformed", f"node `{n['id']}`: lib/ entry `{n['library']}` disagrees with lib/identities.tsv")
            if not e.get("citable") or e.get("assumptions"):
                raise Refuse("conditional", f"node `{n['id']}`: `{n['library']}` is conditional")
            if e["fingerprint"] != n["fingerprint"]:
                raise Refuse("stale", f"node `{n['id']}`: fingerprint differs from the library entry")
            entries[n["id"]] = e
    for n in nodes:
        for pn in ([n["assembly"]] if n["layer"] else []) + [n["spec"]]:
            regular_file(pn["path"], n["id"])
            if sha_file(pn["path"]) != pn["sha256"]:
                raise Refuse("stale", f"node `{n['id']}`: {pn['path']} changed")

    # Kernel runs, all in parallel; results are read in a fixed order.
    files = {n["spec"]["path"] for n in nodes}
    files |= {n["assembly"]["path"] for n in nodes if n["layer"]}
    files |= {entries[i]["proof"]["path"] for i in entries}
    with concurrent.futures.ThreadPoolExecutor(max_workers=os.cpu_count() or 2) as ex:
        runs = dict(zip(sorted(files), ex.map(lambda f: Check(rlt, f), sorted(files))))

    APP = approved()
    spec = {}
    specs = register("compose/approved-specifications.tsv", 3)
    for n in nodes:
        if n["id"] not in APP:
            raise Refuse("specification", f"node `{n['id']}` is not an approved node of the H45 composition")
        if specs.get(n["id"]) != [n["spec"]["path"], n["spec"]["sha256"]]:
            raise Refuse("specification", f"node `{n['id']}`: not the registered specification probe")
    for n in nodes:
        c = runs[n["spec"]["path"]]
        if not c.accepted():
            raise Refuse("specification", f"node `{n['id']}`: specification not accepted")
        if c.cites or len(c.declared) != 1 or len(c.gamma) != 1 or canon(parse(next(iter(c.gamma.values())))) != canon(parse(c.proved)):
            raise Refuse("specification", f"node `{n['id']}`: not a specification probe")
        if c.parts_problem():
            raise Refuse("incompatible", f"node `{n['id']}`: specification: {c.parts_problem()}")
        spec[n["id"]] = canon(parse(c.proved))
        if spec[n["id"]] != APP[n["id"]]:
            raise Refuse("specification", f"node `{n['id']}`: the specification is not the statement rebuilt from the transcription")

    # Assemblies first, then components; discharges once all are known.
    stmt, owner, pending = {}, {}, []
    for n in sorted(nodes, key=lambda n: (n["layer"] == 0, n["layer"])):
        if n["layer"] == 0:
            e = entries[n["id"]]
            if sha_file(e["proof"]["path"]) != e["proof"]["sha256"]:
                raise Refuse("stale", f"node `{n['id']}`: the library proof changed")
            c = runs[e["proof"]["path"]]
            if not c.accepted():
                cls = ("stale" if re.search(r"citation `[^`]*`: \S+ has SHA-256", c.verdict)
                       else "missing" if re.search(r"citation `[^`]*`: cited path `[^`]*`: entity not found", c.verdict) else "kernel")
                raise Refuse(cls, f"node `{n['id']}`: not accepted ({c.verdict[:80]})")
            fp = fingerprint(c)
            if fp != n["fingerprint"]:
                raise Refuse("stale", f"node `{n['id']}`: the statement checked in this run has fingerprint {fp}, not the pinned {n['fingerprint']}")
            if c.gamma:
                raise Refuse("conditional", f"node `{n['id']}`: rests on assumptions")
            if c.parts_problem():
                raise Refuse("incompatible", f"node `{n['id']}`: {c.parts_problem()}")
        else:
            c = runs[n["assembly"]["path"]]
            if not c.accepted():
                raise Refuse("kernel", f"node `{n['id']}`: assembly not accepted")
            if c.cites:
                raise Refuse("graph", f"node `{n['id']}`: the assembly cites files")
            if c.parts_problem():
                raise Refuse("incompatible", f"node `{n['id']}`: {c.parts_problem()}")
            for d in c.declared:
                if d not in c.gamma:
                    raise Refuse("unused", f"node `{n['id']}`: assumption `{d}` declared but not used")
            rows = {d["assumption"]: d["by"] for d in n["discharge"]}
            for g in c.gamma:
                if g not in rows:
                    raise Refuse("missing", f"node `{n['id']}`: assumption `{g}` has no discharge row")
            for a, by in rows.items():
                if a not in c.gamma:
                    raise Refuse("unused", f"node `{n['id']}`: the row for `{a}` names no assumption")
                pending.append((n["layer"], n["id"], a, canon(parse(c.gamma[a])), by))
        s = canon(parse(c.proved))
        if s != spec[n["id"]]:
            raise Refuse("specification", f"node `{n['id']}`: statement differs from its specification")
        if s in owner:
            raise Refuse("duplicated", f"nodes `{owner[s]}` and `{n['id']}` state the same formula")
        owner[s] = n["id"]
        stmt[n["id"]] = s
    for (_, node, a, g, by) in sorted(pending, key=lambda p: p[0]):
        if g != stmt[by]:
            raise Refuse("mismatch", f"node `{node}`: assumption `{a}` is not the statement of `{by}`")
    named = register("compose/approved-results.tsv", 3)
    if y["id"] not in named:
        raise Refuse("specification", f"named result `{y['id']}` is not registered")
    if named[y["id"]][0] != ",".join(y["result"]):
        raise Refuse("specification", f"named result `{y['id']}`: the result differs from the registered one")
    edges = [(node, a, by) for (_, node, a, _, by) in sorted(pending, key=lambda p: p[0])]
    report.append(f"graph           {len(nodes)} nodes in layers 0–{max(n['layer'] for n in nodes)}, {len(edges)} discharged assumptions")
    for (n, a, by) in edges:
        report.append(f"  edge          {n}: {a} ← {by}")
    for r in y["result"]:
        lab = "kernel accepted" if ids[r]["layer"] == 0 else "discharged by composition"
        app = "approved statement (rebuilt from the transcription)" if stmt[r] == APP[r] else "NOT the approved statement"
        report.append(f"  result        {r}: {lab}; {app}")
    report.append("remaining       assumptions: none")
    report.append("summary         a composition of kernel-checked theorems, not one kernel theorem")


def main():
    args = sys.argv[1:]
    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    if "--root" in args:
        i = args.index("--root")
        root = os.path.abspath(args[i + 1])
        del args[i:i + 2]
    quiet = "--quiet" in args
    args = [a for a in args if a != "--quiet"]
    if len(args) != 2:
        sys.exit("usage: compose-check.py RLT_BINARY BUNDLE [--quiet] [--root DIR]")
    rlt, bundle = os.path.abspath(args[0]), args[1]
    os.chdir(root)
    report = [f"compose-check   {bundle}"]
    try:
        try:
            compose(rlt, bundle, report)
        except ValueError as e:  # a printed formula this reader cannot read
            raise Refuse("kernel", f"unreadable kernel output: {e}")
        verdict, code = "verdict COMPOSED", 0
    except Refuse as r:
        verdict, code = f"verdict REFUSED ({r.cls}) — {r.msg}", CODES.get(r.cls, 1)
    if not quiet:
        print("\n".join(report))
    print(verdict)
    print(f"exit {code}")
    sys.exit(code)


if __name__ == "__main__":
    main()

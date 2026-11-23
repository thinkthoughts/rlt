#!/usr/bin/env python3
"""RLT-DEF-001a: an independent cross-check of the definition registry.

usage: python3 -I scripts/check-definition-identities.py RLT_BINARY [--mutate FIELD]

Recomputes, without any RLT code, everything the registry asserts:

- the registry file's SHA-256 against PINNED_DEFINITIONS_REGISTRY_SHA, and
  its entries against PINNED_DEFINITION_FILES (both read from
  cli/src/defreg.rs), and the nat-def entry against PINNED_DEFINITIONS
  (cli/src/lib.rs);
- each registered file's SHA-256;
- each function's content identity, from its own parse of the definitions
  file (canonical text rlt-function-definition/1) and expansion identity
  (rlt-function-expansion/1);
- one content identity per name across the registry (I7);

then compares all of it with `rlt definitions --format json`. Exit 0 when
everything agrees, 1 on any disagreement, 2 on a usage or read error.

--mutate FIELD (body, formal, written, params, deps) changes that field of
every canonical text before hashing; the run must then fail (RLT-DEF-001a
DR-22: a cross-check that cannot disagree checks nothing).
"""
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def sha(b):
    return hashlib.sha256(b).hexdigest()


def tokens(text):
    out, i = [], 0
    while i < len(text):
        c = text[i]
        if c == ";":
            while i < len(text) and text[i] != "\n":
                i += 1
        elif c.isspace():
            i += 1
        elif c in "()":
            out.append(c)
            i += 1
        elif c == '"':
            j, s = i + 1, ""
            while text[j] != '"':
                if text[j] == "\\":
                    s += text[j + 1]
                    j += 2
                else:
                    s += text[j]
                    j += 1
            out.append(("str", s))
            i = j + 1
        else:
            j = i
            while j < len(text) and not text[j].isspace() and text[j] not in '();"':
                j += 1
            out.append(text[i:j])
            i = j
    return out


def parse(text):
    toks, pos = tokens(text), 0

    def one():
        nonlocal pos
        t = toks[pos]
        pos += 1
        if t == "(":
            lst = []
            while toks[pos] != ")":
                lst.append(one())
            pos += 1
            return lst
        return t

    forms = []
    while pos < len(toks):
        forms.append(one())
    return forms


def sexp_text(e):
    if isinstance(e, list):
        return "(" + " ".join(sexp_text(x) for x in e) + ")"
    if isinstance(e, tuple):
        return json.dumps(e[1], ensure_ascii=False)
    return e


def numeral(n):
    t = "0"
    for _ in range(n):
        t = f"S({t})"
    return t


def kernel(e, params, earlier, deps):
    """Kernel form of a formal body, earlier functions unfolded."""
    if isinstance(e, str):
        if e in params:
            return ("var", e)
        if e.isdigit():
            return ("num", int(e))
        raise ValueError(f"unexpected symbol {e}")
    head, args = e[0], e[1:]
    if head in ("+", "*") and len(args) == 2:
        return ("app", head, [kernel(a, params, earlier, deps) for a in args])
    if head == "S" and len(args) == 1:
        return ("app", "S", [kernel(args[0], params, earlier, deps)])
    if head in earlier:
        f = earlier[head]
        deps.add(head)
        deps.update(f["deps"])
        vals = [kernel(a, params, earlier, deps) for a in args]
        return subst(f["body"], dict(zip(f["params"], vals)))
    raise ValueError(f"unexpected head {head}")


def subst(t, sigma):
    if t[0] == "var":
        return sigma.get(t[1], t)
    if t[0] == "app":
        return ("app", t[1], [subst(a, sigma) for a in t[2]])
    return t


def kform(t):
    if t[0] == "var":
        return t[1]
    if t[0] == "num":
        return numeral(t[1])
    return f"{t[1]}(" + ", ".join(kform(a) for a in t[2]) + ")"


def definitions_file(text, mutate):
    [top] = parse(text)
    assert top[0] == "rlt-definitions", "not an (rlt-definitions …) file"
    head = {f[0]: f[1] for f in top[1:] if f[0] in ("id", "convention", "domain")}
    order, earlier, out = [], {}, []
    for f in top[1:]:
        if f[0] != "define-function":
            continue
        name, params, body = f[1], f[2], f[3]
        fields = {x[0]: x[1] for x in f[4:]}
        written = fields["written"][1]
        prov = (
            f"source {fields['source']}" if "source" in fields
            else f"origin {fields['origin']}" if "origin" in fields else "none"
        )
        deps = set()
        b = kernel(body, params, earlier, deps)
        dep_list = [n for n in order if n in deps]
        canon = {
            "params": " ".join(params),
            "body": kform(b),
            "formal": sexp_text(body),
            "written": written,
        }
        if mutate in canon:
            canon[mutate] += " "
        text_c = (
            f"rlt-function-definition/1\nname {name}\nparams {canon['params']}\nbody {canon['body']}\n"
            f"formal {canon['formal']}\nwritten {canon['written']}\nconvention {head['convention']}\n"
            f"domain {head['domain']}\nprovenance {prov}\n"
        )
        dep_text = "deps" + "".join(f" {n}={earlier[n]['content']}" for n in dep_list)
        if mutate == "deps":
            dep_text += " x"
        content = sha((text_c + dep_text + "\n").encode())
        renamed = subst(b, {p: ("var", f"p{i}") for i, p in enumerate(params)})
        expansion = sha(f"rlt-function-expansion/1\narity {len(params)}\nbody {kform(renamed)}\n".encode())
        earlier[name] = {"params": params, "body": b, "deps": set(dep_list), "content": content}
        order.append(name)
        out.append({"name": name, "written": written, "content_sha256": content, "expansion_sha256": expansion})
    return head, out


def rust_const(src, name):
    m = re.search(rf"pub const {name}: &str =\s*\"([0-9a-f]{{64}})\";", src)
    return m.group(1) if m else None


def main(argv):
    if len(argv) < 2:
        print(__doc__)
        return 2
    rlt = argv[1]
    mutate = argv[3] if len(argv) == 4 and argv[2] == "--mutate" else None
    problems = []
    defreg = (ROOT / "cli/src/defreg.rs").read_text(encoding="utf-8")
    lib = (ROOT / "cli/src/lib.rs").read_text(encoding="utf-8")
    pin_reg = rust_const(defreg, "PINNED_DEFINITIONS_REGISTRY_SHA")
    block = re.search(r"PINNED_DEFINITION_FILES: &\[\(&str, &str, &str\)\] = &\[(.*?)\];", defreg, re.S).group(1)
    pin_files = re.findall(r'\(\s*"([^"]+)",\s*"([^"]+)",\s*"([0-9a-f]{64})",?\s*\)', block)
    m = re.search(r'PINNED_DEFINITIONS: \(&str, &str\) = \(\s*"([^"]+)",\s*"([0-9a-f]{64})",?\s*\);', lib)
    pin_natdef = (m.group(1), m.group(2))
    reg_path = ROOT / "spec/definitions-registry.yaml"
    reg_bytes = reg_path.read_bytes()
    if sha(reg_bytes) != pin_reg:
        problems.append(f"registry sha256 {sha(reg_bytes)} differs from the pin {pin_reg}")
    import yaml  # system PyYAML; reading only

    reg = yaml.safe_load(reg_bytes.decode("utf-8"))
    entries = reg["files"]
    listed = [(e["id"], e["path"], e["sha256"]) for e in entries]
    if listed != pin_files:
        problems.append(f"registry entries {listed} differ from PINNED_DEFINITION_FILES {pin_files}")
    if reg.get("format") != "rlt-definitions-registry/1" or reg.get("canonical_text") != "rlt-function-definition/1":
        problems.append("registry format or canonical_text is not version 1")
    computed, names = {}, {}
    for e in entries:
        data = (ROOT / e["path"]).read_bytes()
        if sha(data) != e["sha256"]:
            problems.append(f"{e['path']}: sha256 {sha(data)}, registry {e['sha256']}")
        if e["kind"] == "nat-def":
            if (e["path"], e["sha256"]) != pin_natdef:
                problems.append(f"nat-def entry {e['path']} {e['sha256']} differs from PINNED_DEFINITIONS {pin_natdef}")
            found = re.findall(r"^\(define (\S+) ", data.decode("utf-8"), re.M)
            if found != e["names"]:
                problems.append(f"nat-def names {e['names']} differ from the file's {found}")
            continue
        head, defs = definitions_file(data.decode("utf-8"), mutate)
        if (head["id"], head["convention"], head["domain"]) != (e["id"], e["convention"], e["domain"]):
            problems.append(f"{e['path']}: header {head} differs from the entry")
        listed_defs = [(d["name"], d["content_sha256"]) for d in e["defines"]]
        mine = [(d["name"], d["content_sha256"]) for d in defs]
        if listed_defs != mine:
            problems.append(f"{e['id']}: registry defines {listed_defs}, recomputed {mine}")
        for d in defs:
            if names.setdefault(d["name"], d["content_sha256"]) != d["content_sha256"]:
                problems.append(f"I7: `{d['name']}` has two content identities")
        computed[e["id"]] = defs
    out = subprocess.run([rlt, "--root", str(ROOT), "definitions", "--format", "json"], capture_output=True, text=True)
    if out.returncode != 0:
        problems.append(f"rlt definitions exited {out.returncode}")
    else:
        j = json.loads(out.stdout)
        for f in j["files"]:
            if f["kind"] != "functions":
                continue
            theirs = [(d["name"], d["written"], d["content_sha256"], d["expansion_sha256"]) for d in f["defines"]]
            mine = [(d["name"], d["written"], d["content_sha256"], d["expansion_sha256"]) for d in computed.get(f["id"], [])]
            if theirs != mine:
                problems.append(f"{f['id']}: rlt reports {theirs}, recomputed {mine}")
        if j.get("sha256") != sha(reg_bytes):
            problems.append("rlt definitions reports another registry sha256")
    n = sum(len(v) for v in computed.values())
    if problems:
        for p in problems:
            print(f"  DISAGREE: {p}")
        print(f"definition identities: {len(problems)} disagreement(s)")
        return 1
    print(f"definition identities: registry pin, {len(entries)} entries, {n} definitions (content and expansion identities) recomputed independently and equal to rlt's")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

#!/usr/bin/env python3
"""RLT-PROVER-002: bind lean/ProverDefinitions.lean to what RLT checked.

What this script checks (textual, mechanical):

1. For every goal tests/definitions/goals/FN_*.goal, `rlt check` of its
   committed certificate prints the folded statement (`folded`) and the
   unfolded statement the kernel checked (`checked goal`). Both are rewritten
   into Lean syntax, and the Lean file must contain exactly
   `theorem FN_unfolds : (folded) ↔ (unfolded)` and
   `theorem FN_holds : folded`.
2. Each Lean `def` of a registered function is the registered written form
   of definitions/double.rlt or definitions/quadruple.rlt, rewritten the same
   way.

Matching is per declaration, never by substring (D5). Comments (`--`, and
nested `/- … -/`, including doc comments) are removed first; the remaining
source must fit a small explicit grammar or the file is refused:

  namespace RLTProverDefinitions
    def NAME BINDERS : Nat := BODY        (each registered function, once)
    theorem NAME : STATEMENT := PROOF     (each FN_unfolds / FN_holds once,
                                           plus quadruple_body, G1_holds,
                                           G2_holds once each)
  end RLTProverDefinitions
  #print axioms RLTProverDefinitions.NAME (each theorem above, once)

Any other command, keyword, attribute, modifier, string or character
literal, duplicate or unknown name refuses the file. Each declaration's
complete statement (a def's whole text) must equal the expected one.

It is this text comparison, not Lean, that pins the Lean statements to RLT's
particular unfolding. Lean (run separately) then checks the hand-written
theorems: `FN_unfolds` by Iff.rfl, i.e. up to definitional equality, which
shows the folded and unfolded statements are equivalent in Lean's ℕ for
these seven goals; Iff.rfl would equally accept some other equivalent
unfolding (for example `(double 2 = 4) ↔ (3 + 1 = 4)`), so on its own it
does not validate RLT's unfolding algorithm. Nothing here is a general proof
that RLT's elaborator agrees with Lean's semantics.

usage: scripts/check-definitions-lean.py RLT_BINARY [--lean PATH]
"""
import glob
import collections
import os
import re
import subprocess
import sys


def lean_of(s: str, typed: bool) -> str:
    # Function applications f(t) → f (t), with simple atoms unparenthesised.
    prev = None
    while prev != s:
        prev = s
        s = re.sub(r"\b([a-z]{2,})\(([^()]*)\)",
                   lambda m: f"{m.group(1)} " + (m.group(2) if re.fullmatch(r"[a-z0-9]+", m.group(2)) else f"({m.group(2)})"),
                   s)
    s = s.replace("×", "*").replace("¬(", "¬ (")
    # ∀x. BODY → ∀ x, BODY  (∀ x : Nat, on the unfolded side)
    s = re.sub(r"∀([a-z])\. ", (lambda m: f"∀ {m.group(1)} : Nat, ") if typed else (lambda m: f"∀ {m.group(1)}, "), s)
    return s


class Refused(Exception):
    pass


def strip_comments(src: str) -> str:
    """Remove `--` line comments and nested `/- … -/` block comments.
    Newlines inside comments are kept. An unterminated block refuses."""
    out, i, n, depth = [], 0, len(src), 0
    while i < n:
        two = src[i:i + 2]
        if depth == 0 and two == "--":
            j = src.find("\n", i)
            i = n if j < 0 else j
        elif two == "/-":
            depth += 1
            i += 2
        elif depth > 0 and two == "-/":
            depth -= 1
            i += 2
        elif depth > 0:
            if src[i] == "\n":
                out.append("\n")
            i += 1
        else:
            out.append(src[i])
            i += 1
    if depth:
        raise Refused("unterminated block comment")
    return "".join(out)


# Words that begin a Lean command. The checked grammar uses only
# namespace, end, def, theorem and #print; any other refuses the file.
COMMANDS = ("namespace end def theorem lemma abbrev example instance axiom "
            "opaque structure inductive class open section variable universe "
            "set_option attribute notation infix infixl infixr prefix postfix "
            "macro macro_rules syntax elab elab_rules local scoped private "
            "protected noncomputable partial unsafe mutual import deriving "
            "export omit include initialize builtin_initialize").split()
CMD_RE = re.compile(r"(?<![\w.'])(?:" + "|".join(COMMANDS) + r")(?![\w'])|#[a-z_]+|@\[")


def items(code: str):
    """Split comment-free code into commands at every command word."""
    starts = [m.start() for m in CMD_RE.finditer(code)]
    if code[:starts[0] if starts else len(code)].strip():
        raise Refused("text before the first command")
    for a, b in zip(starts, starts[1:] + [len(code)]):
        yield re.sub(r"\s+", " ", code[a:b]).strip()


def declarations(lean: str, theorems: set, defs: set):
    """Return {name: flattened declaration} under the checked grammar."""
    code = strip_comments(lean)
    if '"' in code:
        raise Refused("a string literal outside comments")
    if re.search(r"'(?:\\.|[^\\\s'])'", code):
        raise Refused("a character literal outside comments")
    seq = list(items(code))
    ns = "namespace RLTProverDefinitions"
    if not seq or seq[0] != ns:
        raise Refused(f"the file must begin with `{ns}`")
    try:
        end = seq.index("end RLTProverDefinitions")
    except ValueError:
        raise Refused("missing `end RLTProverDefinitions`")
    found = collections.defaultdict(list)
    for it in seq[1:end]:
        m = re.fullmatch(r"(def|theorem) ([A-Za-z_][A-Za-z0-9_]*) .*", it)
        if not m:
            raise Refused(f"unsupported command `{it[:60]}`")
        kind, name = m.groups()
        if (kind == "def") != (name in defs) or name not in theorems | defs:
            raise Refused(f"unexpected {kind} `{name}`")
        found[name].append(it)
    printed = collections.Counter()
    for it in seq[end + 1:]:
        m = re.fullmatch(r"#print axioms RLTProverDefinitions\.([A-Za-z0-9_]+)", it)
        if not m:
            raise Refused(f"unsupported command after the namespace `{it[:60]}`")
        printed[m.group(1)] += 1
    for name in sorted(theorems | defs):
        if len(found[name]) != 1:
            raise Refused(f"`{name}` is declared {len(found[name])} times; exactly once is required")
    for name in sorted(theorems):
        if printed[name] != 1:
            raise Refused(f"`#print axioms RLTProverDefinitions.{name}` occurs {printed[name]} times; exactly once is required")
    if set(printed) - theorems:
        raise Refused(f"#print axioms for unknown names {sorted(set(printed) - theorems)}")
    return {k: v[0] for k, v in found.items()}


def theorem_statement(decl: str) -> str:
    """`theorem NAME : STATEMENT := PROOF` → STATEMENT (the first ` := `)."""
    m = re.fullmatch(r"theorem [A-Za-z0-9_]+ : (.*?) := (.+)", decl)
    if not m:
        raise Refused(f"`{decl[:60]}` is not `theorem NAME : STATEMENT := PROOF`")
    return m.group(1)


# Lean theorems in the file that carry no RLT statement to compare.
OTHER_THEOREMS = {"quadruple_body", "G1_holds", "G2_holds"}


def main() -> int:
    args = sys.argv[1:]
    lean_path = "lean/ProverDefinitions.lean"
    if "--lean" in args:
        i = args.index("--lean")
        lean_path = args[i + 1]
        del args[i:i + 2]
    rlt = args[0]
    lean = open(lean_path, encoding="utf-8").read()
    bad, n = 0, 0
    goals = sorted(glob.glob("tests/definitions/goals/F*_*.goal"))
    tags = [os.path.basename(g)[:-5].split("_")[0] for g in goals]
    want_defs = {}
    # 2. Lean's defs are the registered written forms.
    for reg in ("definitions/double.rlt", "definitions/quadruple.rlt"):
        text = open(reg, encoding="utf-8").read()
        for m in re.finditer(r'\(written "([a-z]{2,})\(([a-z](?:, [a-z])*)\) := ([^"]*)"\)', text):
            fname, params, rhs = m.group(1), m.group(2).split(", "), m.group(3)
            binders = " ".join(f"({p} : Nat)" for p in params)
            want = f"def {fname} {binders} : Nat := {lean_of(rhs, False)}"
            if want_defs.setdefault(fname, (want, reg))[0] != want:
                print(f"  UNEXPECTED: {reg} registers `{fname}` differently from {want_defs[fname][1]}")
                bad += 1
    theorems = {f"{t}_{k}" for t in tags for k in ("unfolds", "holds")} | OTHER_THEOREMS
    try:
        decls = declarations(lean, theorems, set(want_defs))
    except Refused as e:
        print(f"  UNEXPECTED: {lean_path} refused: {e}")
        print(f"{lean_path}: 0 of {len(goals)} goals checked (the file is outside the checked declaration grammar)")
        return 1
    for fname, (want, reg) in sorted(want_defs.items()):
        if decls[fname] != want:
            print(f"  UNEXPECTED: {lean_path} declares `{decls[fname]}`, registered in {reg} as `{want}`")
            bad += 1
    for g, tag in zip(goals, tags):
        name = os.path.basename(g)[:-5]
        cert = f"tests/definitions/certificates/{name}.rlt"
        out = subprocess.run([rlt, "check", cert], capture_output=True, text=True).stdout
        folded = re.search(r"^  folded        (.*)$", out, re.M)
        checked = re.search(r"^checked goal    (.*)$", out, re.M)
        if not folded or not checked or "verdict         ACCEPT" not in out:
            print(f"  UNEXPECTED: {cert} did not replay with a folded and checked goal")
            bad += 1
            continue
        f, u = folded.group(1), checked.group(1)
        # RLT prints S(x) as x + 1 already; strip RLT's outer parentheses of
        # ∀x. (A → B) only where Lean would need none (none occur here).
        want = {f"{tag}_unfolds": f"({lean_of(f, False)}) ↔ ({lean_of(u, True)})",
                f"{tag}_holds": lean_of(f, False)}
        n += 1
        for thm, stmt in want.items():
            try:
                got = theorem_statement(decls[thm])
            except Refused as e:
                print(f"  UNEXPECTED: {lean_path}: {e}")
                bad += 1
                continue
            if got != stmt:
                print(f"  UNEXPECTED: {lean_path}: theorem {thm} states `{got}`; RLT requires `{stmt}`")
                bad += 1
    expected = len(goals)
    if n != expected or n == 0:
        bad += 1
    print(f"{lean_path}: {n - min(n, bad)} of {expected} goals declared exactly once with RLT's folded and unfolded statements, Lean defs equal to the registered written forms (declaration-level textual correspondence, comments excluded; Lean's Iff.rfl checks equivalence up to definitional equality, not RLT's unfolding algorithm)")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())

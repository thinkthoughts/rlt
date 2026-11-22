#!/usr/bin/env python3
"""RLT-PROVER-002: bind lean/ProverDefinitions.lean to what RLT checked.

The expectations come from RLT and the registered definitions only, never
from the Lean file:

- for every goal tests/definitions/goals/FN_*.goal, `rlt check` of its
  committed certificate prints the folded statement (`folded`) and the
  unfolded statement the kernel checked (`checked goal`); the expected Lean
  theorems are `FN_unfolds : (folded) ↔ (unfolded)` and `FN_holds : folded`;
- `quadruple_body : ∀ x : Nat, quadruple x = BODY`, with BODY the unfolded
  quadruple RLT prints; `G1_holds`, `G2_holds`: the statements `rlt prove`
  prints for tests/definitions/notfound/G*.goal;
- each Lean `def` is the registered written form of definitions/double.rlt
  and definitions/quadruple.rlt.

Two layers check the Lean file against them.

1. Lean declaration correspondence (authoritative; audit D6). The script
   writes an expected Lean module from the expectations alone (`def`s, and
   `def T.expected : Prop := STATEMENT` for each theorem T, with
   `theorem FN_unfolds.by_rfl : FN_unfolds.expected := Iff.rfl`), and has
   Lean compile it and the companion file separately, each to an .olean in a
   fresh directory. scripts/lean/DeclCheck.lean, a program that never imports
   the companion, then loads both environments and compares the elaborated
   declarations as Lean terms (`Expr`, alpha-equivalence): every def's type
   and value, every theorem's type, the companion's imports (Init only), the
   absence of any other constant, and each theorem's axioms (propext,
   Classical.choice, Quot.sound only, collected from the kernel terms). Lean's
   kernel checks each unfolding by Iff.rfl, definitional equality, in the
   clean expected module. Before the comparison, the pinned toolchain's
   `leanchecker` replays every constant of the compiled companion through
   Lean's kernel (H1), so a declaration stored without kernel checking
   (`set_option debug.skipKernelTC true`) is refused. Comments, strings, escaped identifiers `«…»`,
   macros, notation, instances and text inside proofs cannot supply this
   evidence: only the declared constants and their elaborated types count.
2. Source layout (an additional defence, not the semantics). After removing
   Lean comments, the file must fit a small explicit grammar:

     namespace RLTProverDefinitions
       def NAME BINDERS : Nat := BODY        (each registered function, once)
       theorem NAME : STATEMENT := PROOF     (each expected theorem, once)
     end RLTProverDefinitions
     #print axioms RLTProverDefinitions.NAME (each theorem above, once)

   Any other command, keyword, attribute, modifier, string, character
   literal, escaped identifier `«…»`, duplicate or unknown name refuses the
   file, and each declaration's statement text must equal the expected one.

Both layers must pass. Without Lean the correspondence is UNVERIFIED (exit
3); `--layer text` runs only layer 2 and says so. Neither layer is a general
proof that RLT's elaborator agrees with Lean's semantics: the comparison
covers these stated declarations only.

usage: scripts/check-definitions-lean.py RLT_BINARY [--lean PATH]
           [--lean-bin LEAN] [--layer both|lean|text] [--verbose]
exit: 0 pass, 1 refused, 2 usage or internal error, 3 Lean unavailable
"""
import collections
import glob
import os
import re
import shutil
import subprocess
import sys
import tempfile


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


# ── Expectations: from RLT's output and the registered definitions only ──

def expectations(rlt: str):
    """(defs, theorems, unfolds, problems): defs {name: (lean def text, reg)},
    theorems {name: Lean statement}, unfolds [FN_unfolds names]."""
    problems = []
    defs = {}
    for reg in ("definitions/double.rlt", "definitions/quadruple.rlt"):
        text = open(reg, encoding="utf-8").read()
        for m in re.finditer(r'\(written "([a-z]{2,})\(([a-z](?:, [a-z])*)\) := ([^"]*)"\)', text):
            fname, params, rhs = m.group(1), m.group(2).split(", "), m.group(3)
            binders = " ".join(f"({p} : Nat)" for p in params)
            want = f"def {fname} {binders} : Nat := {lean_of(rhs, False)}"
            if defs.setdefault(fname, (want, reg))[0] != want:
                problems.append(f"{reg} registers `{fname}` differently from {defs[fname][1]}")
    theorems, unfolds = {}, []
    goals = sorted(glob.glob("tests/definitions/goals/F*_*.goal"))
    quad_body = None
    for g in goals:
        name = os.path.basename(g)[:-5]
        tag = name.split("_")[0]
        cert = f"tests/definitions/certificates/{name}.rlt"
        out = subprocess.run([rlt, "check", cert], capture_output=True, text=True).stdout
        folded = re.search(r"^  folded        (.*)$", out, re.M)
        checked = re.search(r"^checked goal    (.*)$", out, re.M)
        if not (folded and checked):
            problems.append(f"{cert} printed no folded/checked statement")
            continue
        f, u = folded.group(1), checked.group(1)
        theorems[f"{tag}_unfolds"] = f"({lean_of(f, False)}) ↔ ({lean_of(u, True)})"
        theorems[f"{tag}_holds"] = lean_of(f, False)
        unfolds.append(f"{tag}_unfolds")
        q = re.search(r"definition    quadruple\(x\) := [^,]*, unfolded quadruple\(x\) := (.*?)  \[", out)
        if q:
            body = q.group(1)
            if quad_body not in (None, body):
                problems.append("RLT printed two different unfoldings of quadruple")
            quad_body = body
    if len(goals) != 7:
        problems.append(f"{len(goals)} goals (want 7)")
    if quad_body is None:
        problems.append("no certificate printed the unfolded quadruple")
    else:
        theorems["quadruple_body"] = lean_of(f"∀x. quadruple(x) = {quad_body}", True)
    for g in sorted(glob.glob("tests/definitions/notfound/G*_*.goal")):
        tag = os.path.basename(g).split("_")[0]
        out = subprocess.run([rlt, "prove", g, "--depth", "1"], capture_output=True, text=True).stdout
        st = re.search(r"^statement       (.*)$", out, re.M)
        if not st or "NOT_FOUND_WITHIN_BOUNDS" not in out:
            problems.append(f"{g}: rlt prove printed no statement or did not report NOT_FOUND_WITHIN_BOUNDS")
            continue
        theorems[f"{tag}_holds"] = lean_of(st.group(1), False)
    for t in ("G1_holds", "G2_holds"):
        if t not in theorems:
            problems.append(f"no expectation for {t}")
    return defs, theorems, unfolds, problems


# ── Layer 2: source layout (additional defence) ──

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
            "export omit include initialize builtin_initialize run_cmd "
            "run_elab run_meta declare_syntax_cat").split()
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
    # D6: an escaped identifier «…» is one token to Lean but ordinary text to
    # this lexer (the audit hid `/-` and a whole theorem inside them), so the
    # characters are refused anywhere, comments included.
    if "«" in lean or "»" in lean:
        raise Refused("an escaped identifier `«…»` (refused anywhere in the file)")
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


def layout_layer(lean_path, lean, defs, theorems):
    """Return the number of problems found by the source-layout check."""
    try:
        decls = declarations(lean, set(theorems), set(defs))
    except Refused as e:
        print(f"  UNEXPECTED (layout): {lean_path} refused: {e}")
        return 1
    bad = 0
    for fname, (want, reg) in sorted(defs.items()):
        if decls[fname] != want:
            print(f"  UNEXPECTED (layout): {lean_path} declares `{decls[fname]}`, registered in {reg} as `{want}`")
            bad += 1
    for thm, stmt in sorted(theorems.items()):
        try:
            got = theorem_statement(decls[thm])
        except Refused as e:
            print(f"  UNEXPECTED (layout): {lean_path}: {e}")
            bad += 1
            continue
        if got != stmt:
            print(f"  UNEXPECTED (layout): {lean_path}: theorem {thm} states `{got}`; RLT requires `{stmt}`")
            bad += 1
    return bad


# ── Layer 1: Lean declaration correspondence (authoritative) ──

AXIOM_LINE = re.compile(r"'RLTProverDefinitions\.[A-Za-z0-9_]+' (does not depend on any axioms|depends on axioms: \[[A-Za-z0-9_., ]*\])")


def resolve_lean(explicit):
    """The pinned toolchain's own lean binary and prefix, or None."""
    cand = explicit or os.environ.get("RLT_LEAN") or shutil.which("lean")
    if not cand:
        home = os.path.expanduser("~/.elan/bin/lean")
        cand = home if os.access(home, os.X_OK) else None
    if not cand:
        return None
    try:
        # Inside lean/, so elan selects lean/lean-toolchain.
        pre = subprocess.run([cand, "--print-prefix"], capture_output=True, text=True, cwd="lean", timeout=600)
    except (OSError, subprocess.TimeoutExpired):
        return None
    prefix = pre.stdout.strip().splitlines()[-1] if pre.returncode == 0 and pre.stdout.strip() else ""
    real = os.path.join(prefix, "bin", "lean")
    if not prefix or not os.access(real, os.X_OK):
        return None
    return real, prefix


def expected_module(defs, theorems, unfolds) -> str:
    lines = ["-- Generated by scripts/check-definitions-lean.py from RLT's printed",
             "-- statements and the registered written definitions; never imports",
             "-- the companion file.",
             "namespace RLTProverDefinitions"]
    for _, (want, _) in defs.items():
        lines.append(want)
    for t, stmt in sorted(theorems.items()):
        lines.append(f"def {t}.expected : Prop := {stmt}")
    for t in unfolds:
        lines.append(f"theorem {t}.by_rfl : {t}.expected := Iff.rfl")
    lines.append("end RLTProverDefinitions")
    return "\n".join(lines) + "\n"


def lean_layer(lean_path, lean_text, defs, theorems, unfolds, real, prefix, verbose=False):
    """Return the number of problems; with verbose, print the checker's evidence."""
    env = {k: v for k, v in os.environ.items() if k not in ("LEAN_PATH", "LEAN_SRC_PATH")}
    env["PATH"] = os.path.join(prefix, "bin") + os.pathsep + env.get("PATH", "")
    pinned = open("lean/lean-toolchain", encoding="utf-8").read().strip().rsplit(":v", 1)[-1]
    ver = subprocess.run([real, "--version"], capture_output=True, text=True, env=env).stdout
    if f"version {pinned}," not in ver:
        print(f"  UNEXPECTED (Lean): toolchain `{ver.strip()}` is not the pinned {pinned}")
        return 1
    checker = os.path.join(os.path.dirname(os.path.abspath(__file__)), "lean", "DeclCheck.lean")
    with tempfile.TemporaryDirectory(prefix="rlt-d6-") as tmp:
        u, e, c = (os.path.join(tmp, d) for d in ("u", "e", "c"))
        for d in (u, e, c):
            os.mkdir(d)
        with open(os.path.join(u, "ProverDefinitions.lean"), "w", encoding="utf-8") as fh:
            fh.write(lean_text)
        with open(os.path.join(e, "RLTExpected.lean"), "w", encoding="utf-8") as fh:
            fh.write(expected_module(defs, theorems, unfolds))
        shutil.copy(checker, os.path.join(c, "DeclCheck.lean"))
        spec = os.path.join(tmp, "spec.txt")
        with open(spec, "w", encoding="utf-8") as fh:
            for d in defs:
                fh.write(f"def RLTProverDefinitions.{d}\n")
            for t in sorted(theorems):
                fh.write(f"theorem RLTProverDefinitions.{t}\n")
            for t in unfolds:
                fh.write(f"unfolds RLTProverDefinitions.{t}\n")
        # The expected module: compiled on its own; Lean's kernel checks each
        # FN_unfolds.by_rfl (definitional equality) here.
        r = subprocess.run([real, "-o", "RLTExpected.olean", "RLTExpected.lean"],
                           capture_output=True, text=True, cwd=e, env=env)
        if r.returncode != 0 or (r.stdout + r.stderr).strip():
            print(f"  UNEXPECTED (Lean): the expected module did not compile cleanly:\n{r.stdout}{r.stderr}")
            return 1
        # The companion: compiled by Lean; any message other than its
        # `#print axioms` reports refuses.
        r = subprocess.run([real, "-o", "ProverDefinitions.olean", "ProverDefinitions.lean"],
                           capture_output=True, text=True, cwd=u, env=env)
        noise = [ln for ln in (r.stdout + r.stderr).splitlines() if ln.strip() and not AXIOM_LINE.fullmatch(ln.strip())]
        if r.returncode != 0 or noise:
            print(f"  UNEXPECTED (Lean): {lean_path} does not elaborate cleanly (exit {r.returncode}):")
            for ln in noise[:8]:
                print(f"    {ln}")
            return 1
        # H1: replay every constant of the compiled companion through Lean's
        # kernel with the pinned toolchain's leanchecker, against the
        # environment the file imports. Compilation alone is not enough:
        # `set_option debug.skipKernelTC true` (or a metaprogram adding
        # declarations under it) lets the elaborator store declarations the
        # kernel never checked. The comparison below reads this same .olean.
        checker_bin = os.path.join(prefix, "bin", "leanchecker")
        if not os.access(checker_bin, os.X_OK):
            print(f"  UNEXPECTED (Lean): {checker_bin} is missing; the kernel replay cannot run")
            return 1
        renv = dict(env, LEAN_PATH=u)
        try:
            r = subprocess.run([checker_bin, "ProverDefinitions"], capture_output=True, text=True,
                               cwd=u, env=renv, timeout=600)
        except subprocess.TimeoutExpired:
            print("  UNEXPECTED (Lean): the kernel replay (leanchecker) timed out")
            return 1
        if r.returncode != 0 or (r.stdout + r.stderr).strip():
            print(f"  UNEXPECTED (Lean): kernel replay (leanchecker) refused the compiled companion (exit {r.returncode}):")
            for ln in (r.stdout + r.stderr).strip().splitlines()[:6]:
                print(f"    {ln}")
            return 1
        if verbose:
            print("  Lean: kernel replay: leanchecker re-checked every constant of the compiled companion with Lean's kernel")
        r = subprocess.run([real, "--run", "DeclCheck.lean", prefix, spec, u, "ProverDefinitions", e, "RLTExpected"],
                           capture_output=True, text=True, cwd=c, env=env)
        out = r.stdout.splitlines()
        for ln in out:
            if verbose and not ln.startswith(("  REFUSED", "RESULT")):
                print(f"  Lean:{ln[1:]}")
            if ln.startswith("  REFUSED"):
                print(f"  UNEXPECTED (Lean): {ln.strip()[len('REFUSED: '):]}")
        result = [ln for ln in out if ln.startswith("RESULT ")]
        if r.returncode != 0 or result != [f"RESULT PASS {len(defs) + len(theorems)}"]:
            if not any(ln.startswith("  REFUSED") for ln in out):
                print(f"  UNEXPECTED (Lean): the declaration checker failed (exit {r.returncode}): {(r.stdout + r.stderr)[-600:]}")
            return 1
    return 0


def main() -> int:
    args = sys.argv[1:]
    opts = {"--lean": "lean/ProverDefinitions.lean", "--lean-bin": None, "--layer": "both"}
    verbose = "--verbose" in args
    if verbose:
        args.remove("--verbose")
    for k in list(opts):
        if k in args:
            i = args.index(k)
            opts[k] = args[i + 1]
            del args[i:i + 2]
    if len(args) != 1 or opts["--layer"] not in ("both", "lean", "text"):
        print(__doc__)
        return 2
    rlt, lean_path, layer = args[0], opts["--lean"], opts["--layer"]
    lean_text = open(lean_path, encoding="utf-8").read()
    defs, theorems, unfolds, problems = expectations(rlt)
    for p in problems:
        print(f"  UNEXPECTED (expectations): {p}")
    if problems:
        return 1
    n_goals = len(unfolds)
    summary = []
    bad = 0
    if layer in ("both", "lean"):
        found = resolve_lean(opts["--lean-bin"])
        if found is None:
            print(f"{lean_path}: Lean declaration correspondence UNVERIFIED (no Lean toolchain found)")
            return 3
        lb = lean_layer(lean_path, lean_text, defs, theorems, unfolds, *found, verbose=verbose)
        bad += lb
        summary.append(f"Lean declaration correspondence {'PASS' if not lb else 'FAIL'} "
                       f"({len(theorems)} theorems' and {len(defs)} defs' elaborated declarations "
                       f"against the expected module built from RLT's printed statements and the "
                       f"registered definitions; {n_goals} FN_unfolds by Iff.rfl in that module; standard axioms only; "
                       f"every constant of the compiled companion replayed by leanchecker)")
    if layer in ("both", "text"):
        tb = layout_layer(lean_path, lean_text, defs, theorems)
        bad += tb
        summary.append(f"source layout {'PASS' if not tb else 'FAIL'} (additional defence, comments excluded)")
    verdict = "PASS" if not bad else "FAIL"
    scope = "" if layer == "both" else f" [layer {layer} only; not the full verdict]"
    print(f"{lean_path}: {verdict} for {n_goals} goals: " + "; ".join(summary) + scope)
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""Generate the RLT-004b `rlt explain` fixtures (untrusted, deterministic).

usage: python3 scripts/gen-explain-tests.py      (from the repository root)

Each fixture is one of the four records in statements/ with one change,
renamed so its id equals its file name. tests/explain/expected.tsv pins
every fixture's exit status and a line its text report must contain.
"""
import os
import re
import shutil

BASE = {
    "E": open("statements/MOD30-CLASSIFICATION.yaml", encoding="utf-8").read(),
    "D": open("statements/DVD-2-8.yaml", encoding="utf-8").read(),
    "M": open("statements/MATH29-PAIRING.yaml", encoding="utf-8").read(),
    "C": open("statements/CONDITIONAL-2X3.yaml", encoding="utf-8").read(),
    "N": open("statements/MATCHABLE-NOT-8.yaml", encoding="utf-8").read(),
}
OTHER_SHA = "f" * 64


def sub(text, old, new, count=1):
    assert old in text, old
    return text.replace(old, new, count)


# (name, base, edits, exit, expected line fragment)
CASES = [
    # Invalid records (exit 2).
    ("XR01-UNKNOWN-KEY", "D", [("subject:", "colour: red\nsubject:")], 2, "field `colour` is outside this record type"),
    ("XR02-DUPLICATE-KEY", "D", [("subject:", "subject: \"first\"\nsubject:")], 2, "duplicated key"),
    ("XR03-ID-MISMATCH", "D", [("id: XR03-ID-MISMATCH", "id: DVD-2-8")], 2, "must equal the file name"),
    ("XR04-PARENT-PATH", "D", [("proof: tests/", "proof: ../rlt/tests/")], 2, "must be a repository-relative .rlt path"),
    ("XR06-MISSING-PROOF", "D", [("CA01_two_divides_eight.rlt", "CA99_absent.rlt")], 2, "cited path `tests/m3c/accept/CA99_absent.rlt`"),
    ("XR07-UNKNOWN-CONVENTION", "D", [("convention: NAT-DEF", "convention: NAT-HEX")], 2, "convention `NAT-HEX` is not registered"),
    ("XR08-UNKNOWN-AXIOM-SET", "D", [("axiom_set: ROBINSON-Q", "axiom_set: PEANO-IND")], 2, "axiom set `PEANO-IND`"),
    ("XR09-UNKNOWN-SOURCE", "D", [("registry: LEAN-CORE-NAT-DVD", "registry: LEAN-CORE-NAT-MOD")], 2, "`LEAN-CORE-NAT-MOD` is not in sources/registry.yaml"),
    ("XR10-EXCERPT-ALTERED", "D", [("b = a * c", "b = c * a")], 2, "is not a whole unit of any registered excerpt"),
    ("XR11-COMMIT-DIFFERS", "D", [("commit: 6a10ac8c", "commit: 7a10ac8c")], 2, "differs from the registry's"),
    ("XR12-PARTIAL-NO-DIFFERS", "E", [(re.compile(r"  differs:\n    - [^\n]*\n"), "")], 2, "status `partial` requires `differs`"),
    ("XR13-MATCHES-UNATTRIBUTED", "E", [("status: partial", "status: matches"), (re.compile(r"  differs:\n    - [^\n]*\n"), ""), ("  assessed_by: \"RLT-003 M3e review and report\"\n", "")], 2, "requires `assessed_by`"),
    ("XR14-UNASSESSED-WITH-BASIS", "D", [("status: unassessed", "status: unassessed\n  basis: {path: docs/RLT-003-M3e.md, sha256: " + OTHER_SHA + "}")], 2, "an unassessed statement carries no assessor"),
    ("XR15-PHYSICAL-MISSING", "D", [(re.compile(r"physical:\n(  [^\n]*\n)+"), "")], 2, "missing mapping `physical`"),
    ("XR16-UNKNOWN-CORRESPONDENCE", "M", [("[math29.context]", "[math29.weather]")], 2, "`math29.weather` is not a correspondence record of reading point math29"),
    ("XR17-LEAN-THEOREM-ABSENT", "E", [("RLTM3eReadings.classification", "RLTM3eReadings.classification_all_n")], 2, "is not declared in lean/M3eReadings.lean"),
    ("XR18-VERDICT-KEY", "D", [("subject:", "verdict: ACCEPT\nsubject:")], 2, "field `verdict` is outside this record type"),
    ("XR19-CONTROL-CHARACTER", "D", [("subject: \"Divisibility: 2 divides 8\"", "subject: \"Divisibility\\u0007: 2 divides 8\"")], 2, "control or invisible character U+0007"),
    ("XR20-BAD-SHA", "D", [("sha256: c237e7de", "sha256: C237E7DE")], 2, "must be 64 lower-case hexadecimal digits"),
    ("XR21-FORGED-STATUS", "E", [("status: partial", "status: verified-by-kernel")], 2, "status `verified-by-kernel` is outside the vocabulary"),
    ("XR22-BASIS-MISSING", "E", [("path: docs/RLT-003-M3e.md", "path: docs/RLT-003-M3z.md")], 2, "semantic basis `docs/RLT-003-M3z.md`"),
    ("XR23-UNKNOWN-READING-POINT", "E", [("reading_point: math29", "reading_point: math31")], 2, "reading point `math31`"),
    ("XR24-UNKNOWN-FORMAT", "D", [("format: rlt-statement/1", "format: rlt-statement/2")], 2, "format `rlt-statement/2` is unknown"),
    ("XR25-RELEVANT-WITHOUT-IDS", "D", [("applicability: not-applicable\n  reason: \"a closed arithmetic statement\"", "applicability: relevant\n  reading_point: math29")], 2, "`relevant` requires"),
    ("XR26-ACCEPTED-AS-STATUS", "D", [("status: unassessed", "status: accepted")], 2, "status `accepted` is outside the vocabulary"),
    # Added after the RLT-004b audit.
    ("XR27-BLANK-ASSESSOR", "E", [("assessed_by: \"RLT-003 M3e review and report\"", "assessed_by: \"  \"")], 2, "`assessed_by` must name the assessor"),
    ("XR28-IMPOSSIBLE-DATE", "E", [("date: 2026-11-09", "date: 2026-02-30")], 2, "`date` must be written YYYY-MM-DD"),
    ("XR29-LEAN-OUTSIDE-LEAN-DIR", "E", [("file: lean/M3eReadings.lean, theorem: RLTM3eReadings.classification", "file: target/fake.lean, theorem: Foo.bar")], 2, "must be a top-level file in lean/"),
    ("XR30-EXCERPT-MARKUP", "C", [("only when its assumption context is empty.", "only when its assumption context is &lt;b&gt;empty.&lt;/b&gt;")], 2, "must be written as its normalised unit"),
    ("XR31-ASSUMPTION-NAME", "C", [("{name: h,", "{name: \"g : 1 = 1, h\",")], 2, "must be an identifier"),
    ("XR32-ASSUMPTION-TWICE", "C", [("    - {name: h, formula: \"2 × 3 = 7\"}", "    - {name: h, formula: \"2 × 3 = 7\"}\n    - {name: h, formula: \"2 × 3 = 7\"}")], 2, "is listed twice"),
    # Stale records (exit 3).
    ("XS01-PROOF-SHA", "D", [("sha256: c237e7de9c8a5ef620506fbe5e18469b7b624c652beef5dd63369cdf2765196d", "sha256: " + OTHER_SHA)], 3, "  stale         proof sha256: record " + OTHER_SHA),
    ("XS02-DEFINITIONS-SHA", "D", [("definitions_sha256: 122e37fc627b136023a1a5ffac0344162ed77ae95312248a9f6a76c2b639ecda", "definitions_sha256: " + OTHER_SHA)], 3, "  stale         definitions sha256: record " + OTHER_SHA),
    ("XS03-AXIOM-SET-SHA", "D", [("axiom_set_sha256: 4ad55192", "axiom_set_sha256: 5ad55192")], 3, "  stale         axiom set sha256: record 5ad55192"),
    ("XS04-CHECKED-GOAL", "D", [("checked_goal: \"∃k. 8 = 2 × k\"", "checked_goal: \"∃k. 8 = k × 2\"")], 3, "  stale         checked goal: record ∃k. 8 = k × 2 · current ∃k. 8 = 2 × k"),
    ("XS05-ASSUMPTIONS", "C", [("formula: \"2 × 3 = 7\"", "formula: \"2 × 3 = 6\"")], 3, "  stale         assumptions: record Γ = { h : 2 × 3 = 6 } · current Γ = { h : 2 × 3 = 7 }"),
    ("XS06-CONVENTION", "M", [("convention: NAT-DEC", "convention: NAT-LOGIC")], 3, "  stale         convention: record NAT-LOGIC · current NAT-DEC"),
    ("XS07-AXIOM-SET", "M", [("axiom_set: PEANO-ADD", "axiom_set: PEANO-ARITH")], 3, "  stale         axiom set: record PEANO-ARITH · current PEANO-ADD"),
    ("XS08-DEFINITIONS-PIN-UNUSED", "M", [("axiom_set: PEANO-ADD", "definitions_sha256: 122e37fc627b136023a1a5ffac0344162ed77ae95312248a9f6a76c2b639ecda\n  axiom_set: PEANO-ADD")], 3, "  stale         definitions sha256: record 122e37fc627b136023a1a5ffac0344162ed77ae95312248a9f6a76c2b639ecda · current (none)"),
    # Proof failures, record otherwise faithful.
    ("XF01-REJECTED-PROOF", "D", [("tests/m3c/accept/CA01_two_divides_eight.rlt", "tests/m3c/reject/CR10_two_not_divides_eight.rlt"), ("sha256: c237e7de9c8a5ef620506fbe5e18469b7b624c652beef5dd63369cdf2765196d", "sha256: @CR10"), ("checked_goal: \"∃k. 8 = 2 × k\"", "checked_goal: \"¬(∃k. 8 = 2 × k)\"")], 1, "1 KERNEL ACCEPTANCE: REJECT"),
    ("XF02-MALFORMED-PROOF", "D", [("tests/m3c/accept/CA01_two_divides_eight.rlt", "tests/m3c/reject/CR17a_cited_hash_differs.rlt"), ("sha256: c237e7de9c8a5ef620506fbe5e18469b7b624c652beef5dd63369cdf2765196d", "sha256: @CR17a")], 2, "1 KERNEL ACCEPTANCE: MALFORMED"),
    # Assessment states: visible, exit unaffected (0).
    ("XA01-ASSESSED-GOAL-DIFFERS", "E", [(re.compile(r'  assessed_goal: "∀r\. '), '  assessed_goal: "∀s. ')], 0, "3 SEMANTIC CORRESPONDENCE: partial · assessment stale"),
    ("XA02-BASIS-SHA-DIFFERS", "E", [("sha256: 67d01af5", "sha256: 77d01af5")], 0, "3 SEMANTIC CORRESPONDENCE: partial · assessment stale"),
    ("XA03-UNPINNED-SOURCE", "C", [], 0, "2 SOURCE FIDELITY: registry verified · unpinned"),
    # RLT-005: the archived arXiv source and the matchable record.
    ("XN01-ARCHIVE-PIN-MODIFIED", "N", [("commit: 21674e5f", "commit: 31674e5f")], 2, "differs from the registry's 21674e5fb69c9a58b187bd51a4efc41330e6eee4d764c3f77882a29fa3aaef16"),
    ("XN02-ARCHIVE-PIN-OMITTED", "N", [("  commit: 21674e5fb69c9a58b187bd51a4efc41330e6eee4d764c3f77882a29fa3aaef16\n", "")], 2, "is pinned at commit 21674e5fb69c9a58b187bd51a4efc41330e6eee4d764c3f77882a29fa3aaef16; the record must state that commit"),
    ("XN03-EXCERPT-ALTERED", "N", [("However, 8 is not matchable", "However, 8 is matchable")], 2, "is not a whole unit of any registered excerpt"),
    ("XN04-PROOF-SHA", "N", [("sha256: e1704268aa4e906e20f3b94d9597409126e797338054adf3bb8cfe003916a9a3", "sha256: " + OTHER_SHA)], 3, "  stale         proof sha256: record " + OTHER_SHA),
    ("XN05-CHECKED-GOAL-NO-DISTINCTNESS", "N", [("(((((¬(a = b) ∧ ¬(a = c)) ∧ ¬(a = d)) ∧ ¬(b = c)) ∧ ¬(b = d)) ∧ ¬(c = d))", "a = a")], 3, "  stale         checked goal: record ((∀d."),
    ("XN06-LEAN-THEOREM-ABSENT", "N", [("RLT005Readings.not_matchable_eight", "RLT005Readings.not_matchable_twelve")], 2, "is not declared in lean/RLT005Readings.lean"),
    ("XN07-REJECTED-PROOF", "N", [("proof: tests/rlt005/accept/MN8_eight_is_not_matchable.rlt", "proof: tests/rlt005/reject/RR07_cites_rejected_proof.rlt"), ("sha256: e1704268aa4e906e20f3b94d9597409126e797338054adf3bb8cfe003916a9a3", "sha256: @RR07")], 2, "1 KERNEL ACCEPTANCE: MALFORMED"),
    ("XN08-BASIS-CHANGED", "N", [(re.compile(r"basis: \{path: docs/RLT-005-CORRESPONDENCE.md, sha256: [0-9a-f]{8}"), "basis: {path: docs/RLT-005-CORRESPONDENCE.md, sha256: 00000000")], 0, "3 SEMANTIC CORRESPONDENCE: partial · assessment stale"),
]

FILES = {
    "@CR10": "tests/m3c/reject/CR10_two_not_divides_eight.rlt",
    "@CR17a": "tests/m3c/reject/CR17a_cited_hash_differs.rlt",
    "@RR07": "tests/rlt005/reject/RR07_cites_rejected_proof.rlt",
}


def sha(path):
    import hashlib
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


out = "tests/explain"
shutil.rmtree(out, ignore_errors=True)
os.makedirs(out)
rows = []
for name, base, edits, code, line in CASES:
    text = BASE[base]
    text = re.sub(r"^id: .*$", "id: " + name, text, count=1, flags=re.M)
    for old, new in edits:
        if isinstance(old, re.Pattern):
            assert old.search(text), old.pattern
            text = old.sub(new, text, count=1)
        else:
            text = sub(text, old, new)
    for tag, path in FILES.items():
        text = text.replace("sha256: " + tag, "sha256: " + sha(path))
    with open(os.path.join(out, name + ".yaml"), "w", encoding="utf-8") as f:
        f.write(text)
    rows.append(f"{name}.yaml\t{code}\t{line}")
with open(os.path.join(out, "expected.tsv"), "w", encoding="utf-8") as f:
    f.write("# file\texit\ta line (or line fragment) the text report contains\n")
    f.write("\n".join(rows) + "\n")
print(f"{len(CASES)} fixtures written to {out}/")

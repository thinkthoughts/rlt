//! Golden tests: every proof file, written-layer case, reading point and the
//! record validation are run, and each report must match `tests/expected/`
//! byte for byte. Files in tests/accept must exit 0; files in tests/reject
//! must exit 1. Regenerate with `UPDATE_GOLDEN=1 cargo test`.

use rlt_cli::{
    check_file, lean_export, points_report, rp_report, validate, written_report, Report, ACCEPT,
    MALFORMED, REJECT,
};
use std::path::{Path, PathBuf};

/// The repository root, resolved at run time (cargo sets CARGO_MANIFEST_DIR
/// for each test run), so a reused target directory never tests another tree.
fn root() -> PathBuf {
    let manifest = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_string());
    Path::new(&manifest).parent().unwrap().to_path_buf()
}

fn golden(name: &str, r: &Report) {
    let path = root().join("tests/expected").join(format!("{name}.out"));
    let body = format!("{}exit {}\n", r.text, r.code);
    if std::env::var("UPDATE_GOLDEN").as_deref() == Ok("1") {
        std::fs::write(&path, &body).unwrap();
        return;
    }
    let want = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!(
            "missing golden file {}; run with UPDATE_GOLDEN=1",
            path.display()
        )
    });
    assert_eq!(
        body,
        want,
        "report for {name} differs from {}",
        path.display()
    );
}

fn rlt_files(dir: &str) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(root().join(dir))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "rlt"))
        .collect();
    v.sort();
    v
}

#[test]
fn accepted_derivations() {
    let files = rlt_files("tests/accept");
    assert!(files.len() >= 6);
    for f in files {
        let r = check_file(&root(), &f);
        assert_eq!(
            r.code,
            ACCEPT,
            "{} should be accepted:\n{}",
            f.display(),
            r.text
        );
        golden(f.file_stem().unwrap().to_str().unwrap(), &r);
    }
}

#[test]
fn rejected_derivations() {
    let files = rlt_files("tests/reject");
    assert!(files.len() >= 4);
    for f in files {
        let r = check_file(&root(), &f);
        assert_eq!(
            r.code,
            REJECT,
            "{} should be rejected:\n{}",
            f.display(),
            r.text
        );
        golden(f.file_stem().unwrap().to_str().unwrap(), &r);
    }
}

#[test]
fn written_layer() {
    for name in ["T4_written", "PI1_written", "spacing_written"] {
        let args =
            std::fs::read_to_string(root().join(format!("tests/written/{name}.args"))).unwrap();
        let a: Vec<&str> = args.lines().collect();
        let r = written_report(&root(), a[0], a[1], a[2], a[3]);
        assert_eq!(r.code, ACCEPT);
        golden(name, &r);
    }
}

#[test]
fn reading_points_and_records() {
    for id in ["math0", "math5", "math29"] {
        let r = rp_report(&root(), id);
        assert_eq!(r.code, ACCEPT, "{id}:\n{}", r.text);
        golden(&format!("rp_{id}"), &r);
    }
    let v = validate(&root());
    assert_eq!(v.code, ACCEPT, "{}", v.text);
    golden("validate", &v);
}

/// Copy the repository's data and tests into a scratch root, apply edits
/// (path, from, to), and return the scratch root. Each `from` must occur.
fn scratch_with(name: &str, edits: &[(&str, &str, &str)]) -> PathBuf {
    let scratch = std::env::temp_dir().join(format!("rlt-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    fn copy_dir(src: &Path, dst: &Path) {
        std::fs::create_dir_all(dst).unwrap();
        for e in std::fs::read_dir(src).unwrap() {
            let p = e.unwrap().path();
            let to = dst.join(p.file_name().unwrap());
            if p.is_dir() {
                copy_dir(&p, &to);
            } else {
                std::fs::copy(&p, &to).unwrap();
            }
        }
    }
    for dir in [
        "sources",
        "conventions",
        "correspondence",
        "readingpoints",
        "points",
        "spec",
        "tests/accept",
    ] {
        copy_dir(&root().join(dir), &scratch.join(dir));
    }
    for (path, from, to) in edits {
        let f = scratch.join(path);
        let text = std::fs::read_to_string(&f).unwrap();
        assert!(
            text.contains(from),
            "fixture edit target missing in {path}: {from}"
        );
        std::fs::write(&f, text.replacen(from, to, 1)).unwrap();
    }
    scratch
}

fn validate_with(name: &str, edits: &[(&str, &str, &str)]) -> Report {
    let scratch = scratch_with(name, edits);
    let r = validate(&scratch);
    let _ = std::fs::remove_dir_all(&scratch);
    r
}

fn assert_invalid(r: &Report, needle: &str) {
    assert_eq!(r.code, REJECT, "expected INVALID:\n{}", r.text);
    assert!(
        r.text.contains(needle),
        "expected `{needle}` in:\n{}",
        r.text
    );
}

const RP0: &str = "readingpoints/math0.yaml";
const RP5: &str = "readingpoints/math5.yaml";
const RP29: &str = "readingpoints/math29.yaml";

#[test]
fn validator_keeps_imagined_separate_from_evidence() {
    // `imagined` on a behavior record (the act of imagining) is refused.
    let r = validate_with("behavior-imagined", &[(RP0, "kind: behavior\n    description: \"a reader imagines a referent for a written expression (agent: a reader; place, time and context unspecified)\"\n    status: unestablished", "kind: behavior\n    description: \"a reader imagines a referent for a written expression (agent: a reader; place, time and context unspecified)\"\n    status: imagined")]);
    assert_invalid(&r, "applies to referent records only");
    // An imagined referent promoted to `supported` with zero evidence is refused.
    let r = validate_with(
        "supported-no-evidence",
        &[(RP0, "status: imagined", "status: supported")],
    );
    assert_invalid(&r, "requires evidence");
    // A kernel theorem offered as physical evidence is refused.
    let r = validate_with("theorem-as-evidence", &[(RP0, "status: imagined\n    evidence: []", "status: supported\n    evidence:\n      - type: kernel_theorem\n        ref: tests/accept/T4_math0_pairing.rlt")]);
    assert_invalid(&r, "outside allowed_support");
    // A lift marked verified in RLT-001 is refused (PI-4).
    let r = validate_with(
        "lift-verified",
        &[(RP0, "status: proposed", "status: verified")],
    );
    assert_invalid(&r, "verified");
}

/// Audit V1–V7, X1-style and Y1 cases: each must now be INVALID.
#[test]
fn validator_enforces_verbatim_whole_units_and_completeness() {
    // V1: an example truncated to a fragment.
    let r = validate_with(
        "v1-truncated",
        &[(
            RP5,
            "written: \"1 + 1 + 1 + 1 + 1 = 5\"",
            "written: \"1 = 5\"",
        )],
    );
    assert_invalid(&r, "equals zero whole units");
    // V2: an empty meaning.
    let r = validate_with("v2-empty", &[(RP0, "text: \"Imagine\"", "text: \"\"")]);
    assert_invalid(&r, "equals zero whole units");
    // V3: an invented meaning cited to a missing source.
    let r = validate_with("v3-missing-source", &[(RP0, "  - text: \"Imagine\"\n    source: REPAIREARTH\n", "  - text: \"Imagine\"\n    source: REPAIREARTH\n  - text: \"invented\"\n    source: SDG5-SPECIFICATION\n")]);
    assert_invalid(&r, "cannot document a meaning");
    // V4: meanings swapped between reading points.
    let r = validate_with("v4-swapped", &[
        (RP0, "text: \"Math0 comprehension (now) specifies +1 reading level: local imagination.\"\n    source: NEWS-MATH00", "text: \"Math29 comprehension (now) specifies +1 reading level: everything else.\"\n    source: NEWS-MATH29"),
        (RP29, "text: \"Math29 comprehension (now) specifies +1 reading level: everything else.\"\n    source: NEWS-MATH29", "text: \"Math0 comprehension (now) specifies +1 reading level: local imagination.\"\n    source: NEWS-MATH00"),
    ]);
    assert_invalid(&r, "excerpts tagged for math0");
    // V5: a meaning cut short, changing what it says.
    let r = validate_with(
        "v5-cut",
        &[(
            RP29,
            "\"Zero differs from five, and five is less than 29.\"",
            "\"Zero differs from five\"",
        )],
    );
    assert_invalid(&r, "equals zero whole units");
    // V6: a documented meaning deleted.
    let r = validate_with(
        "v6-deleted",
        &[(
            RP0,
            "  - text: \"count from zero\"\n    source: RLT-INDEX\n",
            "",
        )],
    );
    assert_invalid(&r, "documented unit \"count from zero\"");
    // V7: a PiπΠ stated form truncated.
    let r = validate_with(
        "v7-pi",
        &[(
            "conventions/pi.yaml",
            "\"PiπΠ · conditional arrows retain their premises\"",
            "\"conditional arrows\"",
        )],
    );
    assert_invalid(&r, "PiπΠ PI-3");
    // Y1: a mistyped field fails loudly instead of hiding records.
    let r = validate_with(
        "y1-types",
        &[(
            RP0,
            "correspondence:\n",
            "correspondence: \"see elsewhere\"\nignored:\n",
        )],
    );
    assert_invalid(&r, "must be a list");
    // A convention with zero sources must be marked RLT-authored.
    let r = validate_with(
        "origin",
        &[(
            "conventions/real-pi.yaml",
            "origin: rlt-authored",
            "origin: elsewhere",
        )],
    );
    assert_invalid(&r, "rlt-authored");
}

/// Audit R1/R2: a relationship record must state the proof's checked goal.
#[test]
fn relationships_are_bound_to_their_proofs() {
    let r = validate_with(
        "r1-written",
        &[(RP5, "written: \"5 + 1 = 6\"", "written: \"5 + 1 = 7\"")],
    );
    assert_invalid(&r, "record states `5 + 1 = 7`, proof checks `5 + 1 = 6`");
    let scratch = scratch_with(
        "r1-rp",
        &[(RP5, "written: \"5 + 1 = 6\"", "written: \"5 + 1 = 7\"")],
    );
    let rp = rp_report(&scratch, "math5");
    let _ = std::fs::remove_dir_all(&scratch);
    assert_eq!(rp.code, REJECT);
    assert!(rp.text.contains("UNBOUND"), "{}", rp.text);
    let r = validate_with(
        "r-wrong-rp",
        &[(
            RP5,
            "proof: tests/accept/T1_math5_pairing.rlt",
            "proof: tests/accept/T4_math0_pairing.rlt",
        )],
    );
    assert_invalid(&r, "proof names reading point");
    let r = validate_with(
        "r-path",
        &[(
            RP5,
            "proof: tests/accept/T1_math5_pairing.rlt",
            "proof: ../../etc/passwd",
        )],
    );
    assert_invalid(&r, "must be a tests/");
}

/// Audit A01–A04: axioms come only from registered, hash-locked sets.
#[test]
fn axiom_sets_are_registered_and_hash_locked() {
    let t1 = "tests/accept/T1_math5_pairing.rlt";
    // Tampering with the registered axiom file is detected.
    let s = scratch_with(
        "tamper",
        &[("spec/peano.rlt", "(= (+ x 0) x)", "(= (+ x 0) (S x))")],
    );
    let r = check_file(&s, &s.join(t1));
    assert_eq!(r.code, MALFORMED, "{}", r.text);
    assert!(r.text.contains("changed without review"), "{}", r.text);
    // An unregistered set id is refused.
    let s2 = scratch_with(
        "unregistered-set",
        &[(t1, "(axioms PEANO-ADD)", "(axioms MY-AXIOMS)")],
    );
    let r = check_file(&s2, &s2.join(t1));
    assert_eq!(r.code, MALFORMED);
    assert!(r.text.contains("not pinned"), "{}", r.text);
    // A set whose source is unregistered or unavailable is refused.
    let s3 = scratch_with(
        "set-source",
        &[(
            "spec/axioms.yaml",
            "    source: PEANO",
            "    source: SDG5-SPECIFICATION",
        )],
    );
    let r = check_file(&s3, &s3.join(t1));
    assert_eq!(r.code, MALFORMED);
    assert!(
        r.text
            .contains("not a registered reference or available source"),
        "{}",
        r.text
    );
    // The verdict names the registered set.
    let r = check_file(&root(), &root().join(t1));
    assert!(r
        .text
        .contains("ACCEPT — unconditional theorem relative to registered axiom set PEANO-ADD"));
    for d in [s, s2, s3] {
        let _ = std::fs::remove_dir_all(d);
    }
}

/// Every audit counterexample keeps its required exit code and verdict.
#[test]
fn audit_counterexamples_stay_refused() {
    let tsv = std::fs::read_to_string(root().join("tests/audit/expected.tsv")).unwrap();
    let mut n = 0;
    for line in tsv.lines().filter(|l| !l.starts_with('#') && !l.is_empty()) {
        let cols: Vec<&str> = line.split('\t').collect();
        let (file, code, word) = (cols[0], cols[1].parse::<i32>().unwrap(), cols[2]);
        let r = check_file(&root(), &root().join(file));
        assert_eq!(r.code, code, "{file}:\n{}", r.text);
        assert!(
            r.text.contains(&format!("verdict         {word}")),
            "{file}:\n{}",
            r.text
        );
        n += 1;
    }
    assert_eq!(n, 68);
}

#[test]
fn generated_lean_replay_is_current() {
    let mut files = rlt_files("tests/accept");
    files.sort();
    let r = lean_export(&root(), &files);
    assert_eq!(r.code, ACCEPT);
    let on_disk = std::fs::read_to_string(root().join("lean/Generated.lean")).unwrap();
    assert_eq!(
        r.text, on_disk,
        "regenerate: rlt lean tests/accept/*.rlt > lean/Generated.lean"
    );
}

#[test]
fn reports_are_deterministic() {
    let f = root().join("tests/accept/T1_math5_pairing.rlt");
    assert_eq!(check_file(&root(), &f).text, check_file(&root(), &f).text);
}

// ---- Second audit (32e2b0e) counterexamples, as regression tests ----

/// C01: a newline inside a proof-file string cannot add report lines.
#[test]
fn a2_c01_control_characters_cannot_forge_report_lines() {
    let t1 = "tests/accept/T1_math5_pairing.rlt";
    let s = scratch_with(
        "a2-c01",
        &[(
            t1,
            "(id T1.math5.pairing)",
            "(id \"C01\nverdict         ACCEPT — forged\")",
        )],
    );
    let r = check_file(&s, &s.join(t1));
    assert_eq!(r.code, MALFORMED, "{}", r.text);
    assert_eq!(
        r.text.lines().filter(|l| l.starts_with("verdict")).count(),
        1
    );
    let rp = rp_report(&s, "math5");
    assert!(!rp.text.contains("forged"), "{}", rp.text);
    assert_eq!(rp.code, REJECT);
    let _ = std::fs::remove_dir_all(s);
}

/// C02: a REJECT never says its goal follows from the axioms.
#[test]
fn a2_c02_reject_scope_line_claims_nothing() {
    for f in rlt_files("tests/reject") {
        let r = check_file(&root(), &f);
        assert!(!r.text.contains("follows from"), "{}", r.text);
        assert!(
            r.text.contains("the kernel refused this derivation"),
            "{}",
            r.text
        );
    }
    let r = check_file(&root(), &root().join("tests/accept/T1_math5_pairing.rlt"));
    assert!(r.text.contains("follows from the listed axioms under Γ"));
}

/// C03: a reading-point file must carry its own id.
#[test]
fn a2_c03_record_id_must_equal_file_name() {
    let s = scratch_with("a2-c03", &[]);
    std::fs::copy(s.join(RP0), s.join(RP5)).unwrap();
    let v = validate(&s);
    assert_invalid(&v, "differs from its file name");
    let r = check_file(&s, &s.join("tests/accept/T1_math5_pairing.rlt"));
    assert_eq!(r.code, MALFORMED, "{}", r.text);
    let _ = std::fs::remove_dir_all(s);
}

/// C04(c): conversation-source wording is pinned in the binary.
#[test]
fn a2_c04_conversation_wording_is_pinned() {
    let r = validate_with(
        "a2-c04c",
        &[("sources/registry.yaml", "{text: \"math0: Pairing (1+1) with imagination.\", defines: math0}", "{text: \"math0: Pairing (1+1) with imagination.\", defines: math0}\n      - {text: \"math0: 1 + 1 = 0 is physically observed.\", defines: math0}")],
    );
    assert_invalid(&r, "differs from the registry pinned in this binary");
    let r = validate_with("a2-c04d", &[(RP0, "  - text: \"Imagine\"\n    source: REPAIREARTH\n", "  - text: \"Imagine\"\n    source: REPAIREARTH\n  - text: \"Math0 means 1 + 1 = 0.\"\n    source: PEANO\n")]);
    assert_invalid(&r, "cannot document a meaning");
}

/// C05: a relationship record states the mode and Γ it expects.
#[test]
fn a2_c05_relationship_mode_and_assumptions_are_bound() {
    let r = validate_with(
        "a2-c05-mode",
        &[(
            RP5,
            "proof: tests/accept/T1_math5_pairing.rlt\n    mode: unconditional",
            "proof: tests/accept/T1_math5_pairing.rlt\n    mode: conditional",
        )],
    );
    assert_invalid(&r, "record expects mode conditional");
    let r = validate_with(
        "a2-c05-gamma",
        &[(
            RP0,
            "assumptions: [\"1 + 1 + 1 + 1 + 1 = 0\"]",
            "assumptions: []",
        )],
    );
    assert_invalid(&r, "record expects Γ");
    // A proof that merely assumes its own goal cannot satisfy an unconditional record.
    let t1 = "tests/accept/T1_math5_pairing.rlt";
    let r = validate_with(
        "a2-c05-self",
        &[(
            t1,
            "(mode unconditional)",
            "(mode conditional) (assume h \"5 + 1 = 6\")",
        )],
    );
    assert_invalid(&r, "record expects mode unconditional");
}

/// C06: Lean names are identifiers; vacuous replays are disclosed.
#[test]
fn a2_c06_lean_names_and_vacuity() {
    let t5 = "tests/accept/T5a_math0_fiction_conditional.rlt";
    let s = scratch_with("a2-c06", &[(t5, "(assume h ", "(assume a/- ")]);
    let r = check_file(&s, &s.join(t5));
    assert_eq!(r.code, MALFORMED, "{}", r.text);
    assert!(r.text.contains("identifier"), "{}", r.text);
    let _ = std::fs::remove_dir_all(s);
    let lean = std::fs::read_to_string(root().join("lean/Generated.lean")).unwrap();
    assert_eq!(
        lean.matches("-- VACUOUS REPLAY: hypothesis h is false in ℕ")
            .count(),
        2
    );
}

/// C07: the axiom registry cannot be redirected.
#[test]
fn a2_c07_axiom_registry_variants_are_refused() {
    let t1 = "tests/accept/T1_math5_pairing.rlt";
    let entry = "  - id: PEANO-ADD\n    file: spec/peano.rlt\n";
    type Case<'a> = (&'a str, Vec<(&'a str, &'a str, &'a str)>, &'a str);
    let cases: Vec<Case> = vec![
        ("dot", vec![("spec/axioms.yaml", "file: spec/peano.rlt", "file: spec/./peano.rlt")], "differs from the pinned"),
        ("dup", vec![("spec/axioms.yaml", entry, "  - id: PEANO-ADD\n    file: spec/peano.rlt\n    sha256: 1a3f83444a06fdfbd02c9af2a5881b47137ba102a675a4e012c84be477b10a0d\n    source: PEANO\n    axioms: [add_zero, add_succ]\n  - id: PEANO-ADD\n    file: spec/peano.rlt\n")], "registered 2 times"),
        ("list", vec![("spec/axioms.yaml", "axioms: [add_zero, add_succ]", "axioms: [add_zero, add_succ, 7]")], "must list names"),
        ("sha", vec![("spec/axioms.yaml", "sha256: 1a3f", "sha256: 0a3f")], "differs from the pinned"),
        ("unpinned", vec![(t1, "(axioms PEANO-ADD)", "(axioms MY-SET)")], "not pinned"),
    ];
    for (name, edits, needle) in cases {
        let s = scratch_with(&format!("a2-c07-{name}"), &edits);
        let r = check_file(&s, &s.join(t1));
        assert_eq!(r.code, MALFORMED, "{name}:\n{}", r.text);
        assert!(
            r.text.contains(needle),
            "{name}: expected `{needle}`:\n{}",
            r.text
        );
        let _ = std::fs::remove_dir_all(s);
    }
    // A symbolic link in place of the axiom file is refused, even to identical bytes.
    let s = scratch_with("a2-c07-link", &[]);
    let target = s.join("peano-copy.rlt");
    std::fs::rename(s.join("spec/peano.rlt"), &target).unwrap();
    std::os::unix::fs::symlink(&target, s.join("spec/peano.rlt")).unwrap();
    let r = check_file(&s, &s.join(t1));
    assert_eq!(r.code, MALFORMED, "{}", r.text);
    assert!(r.text.contains("symbolic link"), "{}", r.text);
    let _ = std::fs::remove_dir_all(s);
}

/// C09: evidence refs resolve; kinds come from a fixed vocabulary.
#[test]
fn a2_c09_evidence_and_kinds() {
    let r = validate_with("a2-c09-ref", &[(RP0, "status: imagined\n    evidence: []", "status: supported\n    evidence:\n      - type: physical_measurement\n        ref: \"none\"")]);
    assert_invalid(&r, "must be an available registered source");
    let r = validate_with("a2-c09-kind", &[(RP0, "kind: behavior", "kind: referent")]);
    assert_eq!(
        r.code, ACCEPT,
        "a behavior relabelled referent is a reviewable edit:\n{}",
        r.text
    );
    let r = validate_with("a2-c09-kind2", &[(RP0, "kind: behavior", "kind: hunch")]);
    assert_invalid(&r, "kind `hunch`");
}

/// C10: proof symlinks, duplicate relationship ids, leading zeros.
#[test]
fn a2_c10_binding_edges() {
    let s = scratch_with("a2-c10-link", &[]);
    let t1 = s.join("tests/accept/T1_math5_pairing.rlt");
    let moved = s.join("t1.rlt");
    std::fs::rename(&t1, &moved).unwrap();
    std::os::unix::fs::symlink(&moved, &t1).unwrap();
    assert_invalid(&validate(&s), "must be a tests/");
    let _ = std::fs::remove_dir_all(s);
    let dup = "  - id: math5.count\n";
    let r = validate_with("a2-c10-dup", &[(RP5, dup, "  - id: math5.pairing\n")]);
    assert_invalid(&r, "appears twice");
    let t1 = "tests/accept/T1_math5_pairing.rlt";
    let s = scratch_with(
        "a2-c10-zero",
        &[(t1, "(goal \"5 + 1 = 6\")", "(goal \"05 + 01 = 06\")")],
    );
    let r = check_file(&s, &s.join(t1));
    assert_eq!(r.code, MALFORMED, "{}", r.text);
    assert!(r.text.contains("leading zero"), "{}", r.text);
    let _ = std::fs::remove_dir_all(s);
}

// ---- Final review (88fbdf0) counterexamples, as regression tests ----

const REG: &str = "sources/registry.yaml";

/// Q4: registry and record changed together are caught by the registry pin.
#[test]
fn r3_q4_registry_content_is_pinned() {
    let count5 = "Count five identifiable objects. Specify what one object means and observe any physical context.";
    type Edits<'a> = Vec<(&'a str, String, String)>;
    let cases: Vec<(&str, Edits)> = vec![
        ("untag", vec![
            (REG, format!("      - {{text: \"{count5}\", defines: math5}}"), format!("      - \"{count5}\"")),
            (RP5, format!("  - text: \"{count5}\"\n    source: REPAIREARTH\n"), String::new()),
        ]),
        ("delete-both", vec![
            (REG, "      - {text: \"<span class=\\\"instruction\\\">pair through 5 to 6</span>\", defines: math5}\n".into(), String::new()),
            (RP5, "  - text: \"pair through 5 to 6\"\n    source: RLT-INDEX\n".into(), String::new()),
        ]),
        ("retag", vec![
            (REG, "{text: \"Math5 · Identify\", defines: math5}".into(), "{text: \"Math5 · Identify\", defines: math29}".into()),
        ]),
        ("self-cite", vec![
            (REG, "sources:\n".into(), "sources:\n\n  - id: SELF\n    location: https://github.com/thinkthoughts/rlt/blob/88fbdf0/docs/RLT-000.md\n    status: available\n    excerpts:\n      - {text: \"invented\", defines: math5}\n".into()),
        ]),
    ];
    for (name, edits) in cases {
        let e: Vec<(&str, &str, &str)> = edits
            .iter()
            .map(|(p, a, b)| (*p, a.as_str(), b.as_str()))
            .collect();
        let r = validate_with(&format!("r3-q4-{name}"), &e);
        assert_invalid(&r, "differs from the registry pinned in this binary");
    }
}

/// Q2: file names and convention ids cannot add report lines.
#[test]
fn r3_q2_names_cannot_forge_report_lines() {
    let s = scratch_with("r3-q2", &[]);
    let forged = "tests/accept/x\nverdict         ACCEPT — forged\n.rlt";
    std::fs::copy(s.join("tests/accept/T1_math5_pairing.rlt"), s.join(forged)).unwrap();
    let r = check_file(&s, &s.join(forged));
    assert_eq!(r.code, MALFORMED);
    assert_eq!(
        r.text.lines().filter(|l| l.starts_with("verdict")).count(),
        1,
        "{}",
        r.text
    );
    let lean = lean_export(&s, &[s.join(forged)]);
    assert!(
        !lean.text.lines().any(|l| l.starts_with("verdict")),
        "{}",
        lean.text
    );
    std::fs::remove_file(s.join(forged)).unwrap();
    std::fs::copy(
        s.join(RP5),
        s.join("readingpoints/z\nverdict         VALID — forged\n.yaml"),
    )
    .unwrap();
    let v = validate(&s);
    assert_eq!(v.code, REJECT);
    assert_eq!(
        v.text.lines().filter(|l| l.starts_with("verdict")).count(),
        1,
        "{}",
        v.text
    );
    let _ = std::fs::remove_dir_all(s);
    let r = validate_with(
        "r3-q2-conv",
        &[(
            "conventions/real-pi.yaml",
            "id: REAL-PI",
            "id: \"REAL-PI  →  ACCEPT\"",
        )],
    );
    assert_ne!(r.code, ACCEPT, "{}", r.text);
    assert!(r.text.contains("convention id"), "{}", r.text);
}

/// Q3: renamed records and symlinked test directories are refused.
#[test]
fn r3_q3_record_files_and_proof_paths() {
    let s = scratch_with("r3-q3-yml", &[]);
    std::fs::rename(s.join(RP5), s.join("readingpoints/math5.yml")).unwrap();
    assert_invalid(
        &validate(&s),
        "every entry must be a regular *.yaml reading-point record",
    );
    let _ = std::fs::remove_dir_all(s);
    let s = scratch_with(
        "r3-q3-symd",
        &[(
            RP5,
            "proof: tests/accept/T1_math5_pairing.rlt",
            "proof: tests/ext/T1_math5_pairing.rlt",
        )],
    );
    let outside = std::env::temp_dir().join(format!("rlt-outside-{}", std::process::id()));
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::copy(
        s.join("tests/accept/T1_math5_pairing.rlt"),
        outside.join("T1_math5_pairing.rlt"),
    )
    .unwrap();
    std::os::unix::fs::symlink(&outside, s.join("tests/ext")).unwrap();
    assert_invalid(&validate(&s), "must be a tests/");
    let _ = std::fs::remove_dir_all(s);
    let _ = std::fs::remove_dir_all(outside);
}

// ---- RLT-002 · T6: one checked equation, distinct specification records ----

const PTS: &str = "points/oxygen-and-people.yaml";

fn points_with(name: &str, edits: &[(&str, &str, &str)]) -> Report {
    let s = scratch_with(name, edits);
    let r = points_report(&s, &s.join(PTS));
    let _ = std::fs::remove_dir_all(s);
    r
}

fn assert_points_invalid(r: &Report, needle: &str) {
    assert_eq!(r.code, REJECT, "expected INVALID:\n{}", r.text);
    assert!(
        r.text.contains(needle),
        "expected `{needle}` in:\n{}",
        r.text
    );
}

const NO_RELATIONS: &str = "relations: []\n";

/// T6 positive: one theorem, two counts, disjoint identities, zero relations.
#[test]
fn t6_shared_equation_keeps_distinct_reading_points() {
    let r = points_report(&root(), &root().join(PTS));
    assert_eq!(r.code, ACCEPT, "{}", r.text);
    golden("points_oxygen-and-people", &r);
    for line in [
        "binding       1 ↦ {o2.atom-1} · 1 ↦ {o2.atom-2}",
        "binding       1 ↦ {person.a} · 1 ↦ {person.b}",
        "identities    o2.atom-1, o2.atom-2 — 2 distinct reading point(s); class oxygen-atom",
        "identities    person.a, person.b — 2 distinct reading point(s); class human-being",
        "sharing it relates none of their reading points",
        "structural, physical and identity relations among these reading points stay unestablished",
        "a specification record: the kernel checked the equation",
    ] {
        assert!(r.text.contains(line), "missing `{line}`:\n{}", r.text);
    }
    assert_eq!(
        r.text
            .matches(
                "ACCEPT — unconditional theorem relative to registered axiom set PEANO-ADD, Γ = ∅"
            )
            .count(),
        2
    );
}

/// T6 positive: a relation recorded as unestablished stays unestablished and is reported.
#[test]
fn t6_unestablished_relation_is_reported_as_unestablished() {
    let rel = "relations:\n  - id: rel.same\n    type: same-structure\n    between: [o2.atom-1, person.a]\n    status: unestablished\n    evidence: []\n";
    let r = points_with("t6-rel-unest", &[(PTS, NO_RELATIONS, rel)]);
    assert_eq!(r.code, ACCEPT, "{}", r.text);
    assert!(r.text.contains("relation        rel.same [same-structure] o2.atom-1, person.a : unestablished (evidence: 0)"), "{}", r.text);
}

/// T6 negatives: arithmetic equality never establishes structural or physical equivalence.
#[test]
fn t6_equal_counts_never_establish_equivalence() {
    let rel = |status: &str, evidence: &str| {
        format!("relations:\n  - id: rel.same\n    type: same-structure\n    between: [o2.atom-1, person.a]\n    status: {status}\n    evidence: {evidence}\n")
    };
    // The theorem offered as evidence for structural sameness.
    let e = rel(
        "supported",
        "\n      - type: kernel_theorem\n        ref: tests/accept/T6_one_plus_one.rlt",
    );
    assert_points_invalid(
        &points_with("t6-n1", &[(PTS, NO_RELATIONS, &e)]),
        "counts and theorems are never evidence for a relation",
    );
    // Sameness asserted with zero evidence.
    let e = rel("supported", "[]");
    assert_points_invalid(
        &points_with("t6-n2", &[(PTS, NO_RELATIONS, &e)]),
        "requires evidence",
    );
    // A count cited as the evidence reference.
    let e = rel(
        "supported",
        "\n      - type: physical_measurement\n        ref: count.oxygen",
    );
    assert_points_invalid(
        &points_with("t6-n3", &[(PTS, NO_RELATIONS, &e)]),
        "must be an available registered source",
    );
    // Sameness of species between oxygen and a person, from equal counts.
    let e = "relations:\n  - id: rel.species\n    type: same-species\n    between: [o2.atom-2, person.b]\n    status: supported\n    evidence: []\n";
    assert_points_invalid(
        &points_with("t6-n4", &[(PTS, NO_RELATIONS, e)]),
        "requires evidence",
    );
}

/// T6/T11 negatives: the binding of numerals to reading points stays consistent.
#[test]
fn t6_bindings_preserve_identities() {
    let people = "groups: [[person.a], [person.b]]";
    assert_points_invalid(
        &points_with(
            "t6-dup",
            &[(PTS, people, "groups: [[person.a], [person.a]]")],
        ),
        "named in two groups",
    );
    assert_points_invalid(
        &points_with(
            "t6-size",
            &[(PTS, people, "groups: [[person.a, person.b], []]")],
        ),
        "summand 1 is 1 but its group names 2",
    );
    assert_points_invalid(
        &points_with(
            "t6-unknown",
            &[(PTS, people, "groups: [[person.a], [person.c]]")],
        ),
        "unknown point `person.c`",
    );
    assert_points_invalid(
        &points_with("t6-arity", &[(PTS, people, "groups: [[person.a]]")]),
        "2 summand(s) but 1 group(s)",
    );
}

/// T12-style negatives: the count is bound to its proof by statement, convention, mode and Γ.
#[test]
fn t6_counts_are_bound_to_their_proof() {
    let w = "written: \"1 + 1 = 2\"\n    convention: NAT-DEC\n    proof: tests/accept/T6_one_plus_one.rlt\n    mode: unconditional\n    assumptions: []\n    groups: [[person.a], [person.b]]";
    let edit = |from: &str, to: &str| w.replacen(from, to, 1);
    assert_points_invalid(
        &points_with(
            "t6-goal",
            &[(PTS, w, &edit("\"1 + 1 = 2\"", "\"1 + 1 = 3\""))],
        ),
        "record states `1 + 1 = 3`, proof checks `1 + 1 = 2`",
    );
    assert_points_invalid(
        &points_with(
            "t6-mode",
            &[(PTS, w, &edit("mode: unconditional", "mode: conditional"))],
        ),
        "record expects mode conditional",
    );
    assert_points_invalid(
        &points_with(
            "t6-gamma",
            &[(PTS, w, &edit("assumptions: []", "assumptions: [\"1 = 1\"]"))],
        ),
        "record expects Γ",
    );
    assert_points_invalid(
        &points_with(
            "t6-conv",
            &[(PTS, w, &edit("convention: NAT-DEC", "convention: REAL-PI"))],
        ),
        "proof uses convention NAT-DEC, record says REAL-PI",
    );
}

/// Scope: a count of particular reading points keeps scope particular.
#[test]
fn t6_scope_is_retained() {
    let people = "groups: [[person.a], [person.b]]";
    assert_points_invalid(
        &points_with(
            "t6-scope",
            &[(
                PTS,
                people,
                "groups: [[person.a], [person.b]]\n    scope: general",
            )],
        ),
        "keeps scope particular",
    );
    let pa = "description: \"person A\"\n    scope: particular";
    assert_points_invalid(
        &points_with(
            "t6-general-point",
            &[(PTS, pa, "description: \"person A\"\n    scope: general")],
        ),
        "a count binds particular, identified reading points",
    );
}

/// Reading points: references by id only; statuses and kinds from the vocabulary.
#[test]
fn t6_records_reference_without_duplicating() {
    assert_points_invalid(
        &points_with(
            "t6-ref",
            &[(PTS, "references: [math5]", "references: [math7]")],
        ),
        "references unknown reading point `math7`",
    );
    assert_points_invalid(
        &points_with(
            "t6-copy",
            &[(
                PTS,
                "references: [math5]",
                "references: [math5]\nmeanings: [\"local anatomy\"]",
            )],
        ),
        "field `meanings` is outside this record type",
    );
    let pa = "description: \"person A\"\n    scope: particular\n    physical: {status: unestablished, evidence: []}";
    assert_points_invalid(&points_with("t6-sup", &[(PTS, pa, "description: \"person A\"\n    scope: particular\n    physical: {status: supported, evidence: []}")]), "requires evidence");
    assert_points_invalid(&points_with("t6-imag", &[(PTS, pa, "description: \"person A\"\n    scope: particular\n    physical: {status: imagined, evidence: []}")]), "applies to imagined-possibility points only");
    assert_points_invalid(
        &points_with(
            "t6-basis",
            &[(PTS, "and two human beings.\"", "and two humans.\"")],
        ),
        "equals zero whole units",
    );
    assert_points_invalid(
        &points_with(
            "t6-kind",
            &[(
                PTS,
                "kind: physical-object\n    class: human-being\n    description: \"person A\"",
                "kind: hunch\n    class: human-being\n    description: \"person A\"",
            )],
        ),
        "kind `hunch`",
    );
}

// ---- RLT-002 audit remediation: one regression test per audit counterexample ----

const STATUS: &str = "correspondence/status.yaml";
const PA: &str = "description: \"person A\"\n    scope: particular\n    physical: {status: unestablished, evidence: []}";

fn pa_physical(physical: &str) -> String {
    format!("description: \"person A\"\n    scope: particular\n    physical: {physical}")
}

fn relation(rtype: &str, between: &str, status: &str, evidence: &str) -> String {
    format!("relations:\n  - id: rel.x\n    type: {rtype}\n    between: {between}\n    status: {status}\n    evidence: {evidence}\n")
}

/// A complete, linked evidence item citing RLT's own index page.
fn linked(claim: &str, referents: &str) -> String {
    format!("\n      - type: physical_measurement\n        ref: RLT-INDEX\n        locator: \"page body\"\n        claim: {claim}\n        referents: {referents}\n        criteria: \"a measurement identifying both referents as the same species\"")
}

/// D1: an edited status vocabulary is refused by every command that reads it,
/// and the status `established` is outside what a record may hold.
#[test]
fn r4_d1_status_vocabulary_is_pinned() {
    let edits = [
        (
            STATUS,
            "correspondence_status: [",
            "correspondence_status: [established, ",
        ),
        (
            STATUS,
            "status_without_evidence: [",
            "status_without_evidence: [established, ",
        ),
        (PTS, PA, &pa_physical("{status: established, evidence: []}")),
    ];
    let r = points_with("r4-d1-points", &edits);
    assert_points_invalid(
        &r,
        "differs from the status vocabulary pinned in this binary",
    );
    assert!(!r.text.contains("physical established"), "{}", r.text);
    let v = validate_with("r4-d1-validate", &edits);
    assert_invalid(
        &v,
        "differs from the status vocabulary pinned in this binary",
    );
    let s = scratch_with("r4-d1-rp", &edits);
    let rp = rp_report(&s, "math5");
    let _ = std::fs::remove_dir_all(&s);
    assert_eq!(rp.code, MALFORMED, "{}", rp.text);
    // With the pinned vocabulary, `established` is outside it.
    let r = points_with(
        "r4-d1-word",
        &[(PTS, PA, &pa_physical("{status: established, evidence: []}"))],
    );
    assert_points_invalid(&r, "status `established` is outside the vocabulary");
}

/// D1/D2: `supported` and `rejected` are establishment judgments; a record
/// using one is refused even with complete, linked evidence.
#[test]
fn r4_d1_judgment_statuses_are_refused() {
    for st in ["supported", "rejected"] {
        let ev = format!("{{status: {st}, evidence: [{{type: physical_measurement, ref: RLT-INDEX, locator: body, claim: physical-correspondence, referents: [person.a], criteria: \"c\"}}]}}");
        let r = points_with(&format!("r4-judge-{st}"), &[(PTS, PA, &pa_physical(&ev))]);
        assert_points_invalid(&r, "records an establishment judgment");
        let rel = relation(
            "same-species",
            "[o2.atom-1, person.a]",
            st,
            &linked("same-species", "[o2.atom-1, person.a]"),
        );
        let r = points_with(&format!("r4-judge-rel-{st}"), &[(PTS, NO_RELATIONS, &rel)]);
        assert_points_invalid(&r, "records an establishment judgment");
        assert!(
            r.text.contains(&format!(
                ": unestablished (record states `{st}`, refused; evidence: 1)"
            )),
            "{}",
            r.text
        );
        assert!(!r.text.contains(&format!(": {st} (")), "{}", r.text);
    }
}

/// D2: submitted evidence is shown in full and leaves the relation unestablished.
#[test]
fn r4_d2_submitted_evidence_stays_unestablished() {
    let rel = relation(
        "same-species",
        "[o2.atom-1, person.a]",
        "evidence-submitted",
        &linked("same-species", "[o2.atom-1, person.a]"),
    );
    let r = points_with("r4-d2-submitted", &[(PTS, NO_RELATIONS, &rel)]);
    assert_eq!(r.code, ACCEPT, "{}", r.text);
    for line in [
        "relation        rel.x [same-species] o2.atom-1, person.a : evidence-submitted (1 item(s), not evaluated by RLT; the relation stays unestablished)",
        "evidence      physical_measurement · RLT-INDEX [https://github.com/thinkthoughts/rlt/blob/838417682c8c3c8dfe05b3094e6fc17cdf53fe5c/index.html] at \"page body\" · for same-species(o2.atom-1, person.a)",
        "submitted, not evaluated by RLT",
    ] {
        assert!(r.text.contains(line), "missing `{line}`:\n{}", r.text);
    }
    for word in [
        ": supported",
        ": established",
        "physical supported",
        "physical established",
    ] {
        assert!(!r.text.contains(word), "`{word}` in:\n{}", r.text);
    }
    // A point with submitted evidence reads the same way.
    let ev = "{status: evidence-submitted, evidence: [{type: physical_measurement, ref: RLT-INDEX, locator: body, claim: physical-correspondence, referents: [person.a], criteria: \"c\"}]}";
    let r = points_with("r4-d2-point", &[(PTS, PA, &pa_physical(ev))]);
    assert_eq!(r.code, ACCEPT, "{}", r.text);
    assert!(r.text.contains("physical evidence-submitted (1 item(s), not evaluated by RLT; correspondence unestablished)"), "{}", r.text);
}

/// D2: evidence must be linked to the exact claim and referents, and give a
/// locator and criteria; extra fields are refused.
#[test]
fn r4_d2_evidence_is_linked_to_its_claim() {
    let between = "[o2.atom-1, person.a]";
    let cases: Vec<(String, &str)> = vec![
        (linked("same-structure", between), "this record's claim is `same-species`"),
        (linked("same-species", "[o2.atom-1, person.b]"), "this record's referents are [o2.atom-1, person.a]"),
        (linked("same-species", "[person.a, o2.atom-1]"), "this record's referents are [o2.atom-1, person.a]"),
        ("\n      - type: physical_measurement\n        ref: RLT-INDEX\n        claim: same-species\n        referents: [o2.atom-1, person.a]\n        criteria: \"c\"".to_string(), "must give a `locator`"),
        ("\n      - type: physical_measurement\n        ref: RLT-INDEX\n        locator: body\n        claim: same-species\n        referents: [o2.atom-1, person.a]".to_string(), "must state its validation `criteria`"),
        ("\n      - type: physical_measurement\n        ref: RLT-INDEX\n        locator: body\n        referents: [o2.atom-1, person.a]\n        criteria: \"c\"".to_string(), "must name its claim"),
        (format!("{}\n        verified: true", linked("same-species", between)), "field `verified` is outside this record type"),
        (format!("{}\n        note: \"established by count.oxygen\"", linked("same-species", between)), "field `note` is outside this record type"),
    ];
    for (i, (ev, needle)) in cases.iter().enumerate() {
        let rel = relation("same-species", between, "evidence-submitted", ev);
        let r = points_with(&format!("r4-d2-link-{i}"), &[(PTS, NO_RELATIONS, &rel)]);
        assert_points_invalid(&r, needle);
    }
    // Evidence present with status unestablished must say evidence-submitted.
    let rel = relation(
        "same-species",
        between,
        "unestablished",
        &linked("same-species", between),
    );
    let r = points_with("r4-d2-unest", &[(PTS, NO_RELATIONS, &rel)]);
    assert_points_invalid(&r, "has status `evidence-submitted`");
    // evidence-submitted with zero items.
    let rel = relation("same-species", between, "evidence-submitted", "[]");
    let r = points_with("r4-d2-empty", &[(PTS, NO_RELATIONS, &rel)]);
    assert_points_invalid(&r, "requires evidence");
}

/// D3: `rlt points`, `rlt rp` and `rlt check` refuse a registry that differs
/// from the pin, including edits to fields outside the excerpts.
#[test]
fn r4_d3_registry_pin_is_checked_by_every_command() {
    let reworded = (
        REG,
        "Preserve distinct reading points for two oxygen atoms and two human beings.",
        "These two oxygen atoms and two human beings are established, physically distinct objects.",
    );
    let basis = (
        PTS,
        "text: \"Preserve distinct reading points for two oxygen atoms and two human beings.\"",
        "text: \"These two oxygen atoms and two human beings are established, physically distinct objects.\"",
    );
    let r = points_with("r4-d3-points", &[reworded, basis]);
    assert_points_invalid(&r, "differs from the registry pinned in this binary");
    let retitled = (
        REG,
        "title: rlt/index.html (current site page)",
        "title: laboratory measurement",
    );
    for (name, e) in [("r4-d3-excerpt", reworded), ("r4-d3-title", retitled)] {
        let s = scratch_with(name, &[e]);
        let rp = rp_report(&s, "math5");
        let ck = check_file(&s, &s.join("tests/accept/T6_one_plus_one.rlt"));
        let pt = points_report(&s, &s.join(PTS));
        let _ = std::fs::remove_dir_all(&s);
        assert_eq!(rp.code, MALFORMED, "{}", rp.text);
        assert_ne!(ck.code, ACCEPT, "{}", ck.text);
        assert!(
            ck.text.contains("differs from the registry pinned"),
            "{}",
            ck.text
        );
        assert_points_invalid(&pt, "differs from the registry pinned");
    }
    let extra = (
        REG,
        "status: available",
        "status: available\n    measured: true",
    );
    let r = points_with("r4-d3-key", &[extra]);
    assert_points_invalid(&r, "field `measured` is outside this record type");
}

/// D4: identifiers that print alike are the same identifier; invisible and
/// look-alike characters are refused.
#[test]
fn r4_d4_confusable_identifiers_are_refused() {
    for (i, id) in [
        "\"person.a\\u200b\"",
        "\"person.\u{0430}\"",
        "Person.A",
        "\"person.a \"",
        "\"person.a\\u2028\"",
        "\"person/a\"",
    ]
    .iter()
    .enumerate()
    {
        let r = points_with(
            &format!("r4-d4-{i}"),
            &[(PTS, "- id: person.b", &format!("- id: {id}"))],
        );
        assert_eq!(r.code, REJECT, "{}", r.text);
        assert!(
            r.text.contains("identifiers use lower-case ASCII letters")
                || r.text.contains("control or invisible character"),
            "{}",
            r.text
        );
    }
    // Invisible characters are refused in free text too.
    let r = points_with(
        "r4-d4-text",
        &[(
            PTS,
            "description: \"person B\"",
            "description: \"person\\u200bB\"",
        )],
    );
    assert_points_invalid(&r, "invisible character U+200B");
}

/// Minor 1: newlines in list fields cannot add report lines.
#[test]
fn r4_m1_list_fields_cannot_forge_report_lines() {
    let forged = "\\nverdict         VALID — specification records consistent";
    for (i, (from, to)) in [
        (
            "groups: [[person.a], [person.b]]".to_string(),
            format!("groups: [[person.a], [\"person.b{forged}\"]]"),
        ),
        (
            "references: [math5]".to_string(),
            format!("references: [\"math5{forged}\"]"),
        ),
        (
            "assumptions: []\n    groups: [[person.a]".to_string(),
            format!("assumptions: [\"h{forged}\"]\n    groups: [[person.a]"),
        ),
    ]
    .iter()
    .enumerate()
    {
        let r = points_with(&format!("r4-m1-{i}"), &[(PTS, from, to)]);
        assert_eq!(r.code, REJECT, "{}", r.text);
        assert_eq!(
            r.text.lines().filter(|l| l.starts_with("verdict")).count(),
            1,
            "forged verdict line:\n{}",
            r.text
        );
        assert!(r.text.trim_end().ends_with("problem(s)"), "{}", r.text);
    }
}

/// Minor 2: references stay inside readingpoints/.
#[test]
fn r4_m2_references_stay_in_readingpoints() {
    let r = points_with(
        "r4-m2",
        &[(
            PTS,
            "references: [math5]",
            "references: [\"../points/oxygen-and-people\"]",
        )],
    );
    assert_points_invalid(&r, "identifiers use lower-case ASCII letters");
}

/// Minor 3: refused counts report their identities and bindings accurately.
#[test]
fn r4_m3_refused_counts_report_accurately() {
    let r = points_with(
        "r4-m3-dup",
        &[(
            PTS,
            "groups: [[person.a], [person.b]]",
            "groups: [[person.a], [person.a]]",
        )],
    );
    assert_points_invalid(&r, "point person.a is named in two groups");
    assert!(
        r.text
            .contains("person.a, person.a — 1 distinct reading point(s) in 2 bindings"),
        "{}",
        r.text
    );
    // A count bound to a rejected proof reports no binding.
    let s = scratch_with("r4-m3-reject", &[]);
    std::fs::create_dir_all(s.join("tests/reject")).unwrap();
    std::fs::copy(
        root().join("tests/reject/T2_math5_broken_trans.rlt"),
        s.join("tests/reject/T2_math5_broken_trans.rlt"),
    )
    .unwrap();
    let pts = std::fs::read_to_string(s.join(PTS)).unwrap().replacen(
        "proof: tests/accept/T6_one_plus_one.rlt\n    mode: unconditional\n    assumptions: []\n    groups: [[person.a], [person.b]]",
        "proof: tests/reject/T2_math5_broken_trans.rlt\n    mode: unconditional\n    assumptions: []\n    groups: [[person.a], [person.b]]",
        1,
    );
    std::fs::write(s.join(PTS), pts).unwrap();
    let r = points_report(&s, &s.join(PTS));
    let _ = std::fs::remove_dir_all(&s);
    assert_points_invalid(&r, "its proof is not accepted by the kernel");
    assert!(
        r.text
            .contains("binding       not reported: the kernel did not accept this proof"),
        "{}",
        r.text
    );
    assert_eq!(r.text.matches("binding       1 ↦").count(), 1, "{}", r.text);
}

/// Minor 4: symbolic links, FIFOs and files outside points/ are refused
/// without being opened.
#[test]
fn r4_m4_links_fifos_and_stray_paths_are_refused() {
    use std::os::unix::fs::symlink;
    let s = scratch_with("r4-m4", &[]);
    // A symlinked points file, pointing outside the repository.
    let outside = std::env::temp_dir().join(format!("rlt-r4-m4-out-{}.yaml", std::process::id()));
    std::fs::copy(s.join(PTS), &outside).unwrap();
    symlink(&outside, s.join("points/linked.yaml")).unwrap();
    let r = points_report(&s, &s.join("points/linked.yaml"));
    assert_points_invalid(&r, "is a symbolic link");
    // A points file outside points/.
    let r = points_report(&s, &outside);
    assert_points_invalid(&r, "inside this repository's points/ directory");
    // A FIFO as the points file and as a bound proof: refused, never opened.
    let mk = |p: &Path| {
        assert!(std::process::Command::new("mkfifo")
            .arg(p)
            .status()
            .unwrap()
            .success())
    };
    mk(&s.join("points/fifo.yaml"));
    let r = points_report(&s, &s.join("points/fifo.yaml"));
    assert_points_invalid(&r, "is not a regular file");
    mk(&s.join("tests/accept/T6_fifo.rlt"));
    let pts = std::fs::read_to_string(s.join(PTS)).unwrap().replacen(
        "proof: tests/accept/T6_one_plus_one.rlt",
        "proof: tests/accept/T6_fifo.rlt",
        1,
    );
    std::fs::write(s.join(PTS), pts).unwrap();
    let r = points_report(&s, &s.join(PTS));
    assert_points_invalid(&r, "its proof is not accepted by the kernel");
    let ck = check_file(&s, &s.join("tests/accept/T6_fifo.rlt"));
    assert_eq!(ck.code, MALFORMED, "{}", ck.text);
    assert!(ck.text.contains("not a regular file"), "{}", ck.text);
    let _ = std::fs::remove_dir_all(&s);
    let _ = std::fs::remove_file(&outside);
}

/// Minor 5: basis required; self-relations, contradictory statuses refused;
/// the convention's reading is shown, so an edit to it is visible.
#[test]
fn r4_m5_stricter_records() {
    let r = points_with("r4-m5-basis", &[(PTS, "basis:\n  text: \"Preserve distinct reading points for two oxygen atoms and two human beings.\"\n  source: RLT-002-REQUEST\n", "")]);
    assert_points_invalid(&r, "missing `basis`");
    let r = points_with(
        "r4-m5-self",
        &[(
            PTS,
            NO_RELATIONS,
            &relation("bonded-to", "[person.a, person.a]", "unestablished", "[]"),
        )],
    );
    assert_points_invalid(&r, "a relation names two or more distinct points");
    let r = points_with(
        "r4-m5-one",
        &[(
            PTS,
            NO_RELATIONS,
            &relation("bonded-to", "[person.a]", "unestablished", "[]"),
        )],
    );
    assert_points_invalid(&r, "a relation names two or more distinct points");
    let r = points_with(
        "r4-m5-inapp",
        &[(
            PTS,
            PA,
            &pa_physical("{status: inapplicable, evidence: []}"),
        )],
    );
    assert_points_invalid(&r, "`inapplicable` applies to other kinds");
    let r = points_with(
        "r4-m5-conv",
        &[(
            "conventions/nat-dec.yaml",
            "+ denotes addition on the natural numbers",
            "+ denotes multiplication of physical objects",
        )],
    );
    assert!(r.text.contains("convention    NAT-DEC (interpretation input) — \"A run of decimal digits n denotes the numeral S^n(0). + denotes multiplication of physical objects"), "{}", r.text);
}

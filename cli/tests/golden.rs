//! Golden tests: every proof file, written-layer case, reading point and the
//! record validation are run, and each report must match `tests/expected/`
//! byte for byte. Files in tests/accept must exit 0; files in tests/reject
//! must exit 1. Regenerate with `UPDATE_GOLDEN=1 cargo test`.

use rlt_cli::{
    check_file, lean_export, rp_report, validate, written_report, Report, ACCEPT, MALFORMED, REJECT,
};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn golden(name: &str, r: &Report) {
    let path = root().join("tests/expected").join(format!("{name}.out"));
    let body = format!("{}exit {}\n", r.text, r.code);
    if std::env::var("UPDATE_GOLDEN").is_ok() {
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
    assert!(r.text.contains("unregistered"), "{}", r.text);
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
        r.text.contains("not an available registered source"),
        "{}",
        r.text
    );
    // The verdict names the registered set.
    let r = check_file(&root(), &root().join(t1));
    assert!(r
        .text
        .contains("ACCEPT — unconditional theorem from registered axiom set PEANO-ADD"));
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

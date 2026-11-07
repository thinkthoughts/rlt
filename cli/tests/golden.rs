//! Golden tests: every proof file, written-layer case, reading point and the
//! record validation are run, and each report must match `tests/expected/`
//! byte for byte. Files in tests/accept must exit 0; files in tests/reject
//! must exit 1. Regenerate with `UPDATE_GOLDEN=1 cargo test`.

use rlt_cli::{check_file, rp_report, validate, written_report, Report, ACCEPT, REJECT};
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

/// Copy the data directories into a scratch root, apply one edit to math0.yaml,
/// and return the validation report.
fn validate_with(name: &str, from: &str, to: &str) -> Report {
    let scratch = std::env::temp_dir().join(format!("rlt-validate-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    for dir in [
        "sources",
        "conventions",
        "correspondence",
        "readingpoints",
        "spec",
        "tests/accept",
    ] {
        let (src, dst) = (root().join(dir), scratch.join(dir));
        std::fs::create_dir_all(&dst).unwrap();
        for e in std::fs::read_dir(&src).unwrap() {
            let p = e.unwrap().path();
            if p.is_file() {
                std::fs::copy(&p, dst.join(p.file_name().unwrap())).unwrap();
            }
        }
    }
    let rp = scratch.join("readingpoints/math0.yaml");
    let text = std::fs::read_to_string(&rp).unwrap();
    assert!(text.contains(from), "fixture edit target missing: {from}");
    std::fs::write(&rp, text.replacen(from, to, 1)).unwrap();
    let r = validate(&scratch);
    let _ = std::fs::remove_dir_all(&scratch);
    r
}

#[test]
fn validator_keeps_imagined_separate_from_evidence() {
    // `imagined` on a behavior record (the act of imagining) is refused.
    let r = validate_with(
        "behavior-imagined",
        "kind: behavior\n    description: \"a reader imagines a referent for a written expression (agent: a reader; place, time and context unspecified)\"\n    status: unestablished",
        "kind: behavior\n    description: \"a reader imagines a referent for a written expression (agent: a reader; place, time and context unspecified)\"\n    status: imagined",
    );
    assert_eq!(r.code, REJECT, "{}", r.text);
    assert!(
        r.text.contains("applies to referent records only"),
        "{}",
        r.text
    );

    // An imagined referent promoted to `supported` with zero evidence is refused.
    let r = validate_with(
        "supported-no-evidence",
        "status: imagined",
        "status: supported",
    );
    assert_eq!(r.code, REJECT, "{}", r.text);
    assert!(r.text.contains("requires evidence"), "{}", r.text);

    // A kernel theorem offered as physical evidence is refused.
    let r = validate_with(
        "theorem-as-evidence",
        "status: imagined\n    evidence: []",
        "status: supported\n    evidence:\n      - type: kernel_theorem\n        ref: tests/accept/T4_math0_pairing.rlt",
    );
    assert_eq!(r.code, REJECT, "{}", r.text);
    assert!(r.text.contains("outside allowed_support"), "{}", r.text);

    // A meaning reworded away from its source is refused (verbatim preservation).
    let r = validate_with(
        "reworded-meaning",
        "text: \"local imagination\"",
        "text: \"imagination nearby\"",
    );
    assert_eq!(r.code, REJECT, "{}", r.text);
    assert!(
        r.text.contains("appears in zero registered excerpts"),
        "{}",
        r.text
    );

    // A lift marked verified in RLT-001 is refused (PI-4).
    let r = validate_with("lift-verified", "status: proposed", "status: verified");
    assert_eq!(r.code, REJECT, "{}", r.text);
}

#[test]
fn reports_are_deterministic() {
    let f = root().join("tests/accept/T1_math5_pairing.rlt");
    assert_eq!(check_file(&root(), &f).text, check_file(&root(), &f).text);
}

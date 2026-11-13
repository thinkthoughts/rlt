//! RLT-007b stage 3: for every p with 212 < p, no element h of H45 satisfies
//! h = (p × k) + 1 for any k. The theorem, its statement rebuilt
//! independently, the rejected cases, the generator's checks and the
//! statement record.

use std::path::{Path, PathBuf};

use rlt_cli::check_file;
use rlt_cli::deps::deps;
use rlt_cli::explain::{explain, render_text};

const ACCEPT: i32 = 0;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn generator(arg: &str) -> std::process::Output {
    std::process::Command::new("python3")
        .args([
            "-I",
            "scripts/gen-rlt007s3.py",
            env!("CARGO_BIN_EXE_rlt"),
            arg,
        ])
        .current_dir(root())
        .output()
        .expect("python3 runs the generator")
}

const VERDICT: &str =
    "verdict         ACCEPT — unconditional theorem relative to registered axiom set ROBINSON-Q";

/// H45 as printed in the paper (Axiom Math, September 3, 2026 draft, §12),
/// written out here independently of the generator and the transcription.
const H45: [u32; 45] = [
    0, 2, 12, 14, 24, 26, 30, 36, 44, 50, 54, 56, 60, 66, 72, 74, 80, 84, 92, 96, 102, 110, 114,
    116, 122, 126, 134, 140, 144, 150, 156, 162, 164, 170, 176, 180, 182, 186, 192, 194, 200, 204,
    206, 210, 212,
];

fn goal(rel: &str) -> String {
    let t = std::fs::read_to_string(root().join(rel)).unwrap();
    let i = t.find("(goal \"").unwrap() + 7;
    t[i..i + t[i..].find('"').unwrap()].to_string()
}

#[test]
fn the_statement_is_exactly_the_approved_one_and_is_accepted() {
    let om = |h: u32| format!("¬(∃k. {h} = (p × k) + 1)");
    let mut c = om(H45[0]);
    for &h in &H45[1..] {
        c = format!("({c} ∧ {})", om(h));
    }
    let rel = "tests/rlt007s3/accept/TAIL.rlt";
    assert_eq!(goal(rel), format!("∀p. (212 < p → {c})"));
    let r = check_file(&root(), &root().join(rel));
    assert!(r.text.contains(VERDICT), "{}", r.text);
    assert!(r.text.contains("assumptions     Γ = ∅"), "{}", r.text);
    let d = deps(&root(), rel);
    assert_eq!((d.exit, d.checks, d.chain), (ACCEPT, 0, 0), "no citations");
    let files = std::fs::read_dir(root().join("tests/rlt007s3/accept"))
        .unwrap()
        .count();
    assert_eq!(files, 1);
}

#[test]
fn every_rejected_case_is_refused_for_its_reason() {
    let tsv = std::fs::read_to_string(root().join("tests/rlt007s3/expected.tsv")).unwrap();
    let mut n = 0;
    for l in tsv.lines().filter(|l| !l.starts_with('#') && !l.is_empty()) {
        let f: Vec<&str> = l.splitn(3, '\t').collect();
        let r = check_file(&root(), &root().join(f[0]));
        assert_eq!(r.code.to_string(), f[1], "{}:\n{}", f[0], r.text);
        let verdict = r.text.lines().find(|l| l.starts_with("verdict")).unwrap();
        assert!(
            verdict.contains(f[2]),
            "{}: `{}` not in {verdict}",
            f[0],
            f[2]
        );
        n += 1;
    }
    assert_eq!(n, 9);
    let files = std::fs::read_dir(root().join("tests/rlt007s3/reject"))
        .unwrap()
        .count();
    assert_eq!(files, n, "every reject file is listed");
}

#[test]
fn the_generator_reproduces_every_output_and_refuses_altered_tuples() {
    let c = generator("--check");
    assert!(c.status.success(), "{}", String::from_utf8_lossy(&c.stdout));
    let t = generator("--self-test");
    let out = String::from_utf8_lossy(&t.stdout);
    assert!(t.status.success(), "{out}");
    assert!(out.contains("6 of 6 altered tuples refused"), "{out}");
    assert!(!out.contains("ACCEPTED"), "{out}");
}

#[test]
fn the_statement_record_is_valid_current_and_accepted() {
    let rel = "statements/TAIL-OMITS-H45.yaml";
    let x = explain(&root(), rel);
    let t = render_text(&x);
    assert_eq!(x.exit, ACCEPT, "{t}");
    assert!(
        t.contains("SEMANTIC CORRESPONDENCE: partial · assessment current"),
        "{t}"
    );
    let n = std::fs::read_dir(root().join("statements"))
        .unwrap()
        .filter(|e| {
            e.as_ref()
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with("TAIL-OMITS-")
        })
        .count();
    assert_eq!(n, 1);
}

/// `rlt deps` agrees with `rlt check` on every committed proof of this stage
/// (the sweep in cli/tests/deps.rs skips this directory).
#[test]
fn deps_agrees_with_check_on_the_committed_proofs() {
    let mut n = 0;
    for d in ["tests/rlt007s3/accept", "tests/rlt007s3/reject"] {
        let mut fs: Vec<_> = std::fs::read_dir(root().join(d))
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        fs.sort();
        for f in fs {
            let rel = f
                .strip_prefix(root())
                .unwrap()
                .to_string_lossy()
                .to_string();
            let dd = deps(&root(), &rel);
            let rep = check_file(&root(), &f);
            let node = dd.root.as_ref().expect("a root node");
            assert_eq!(node.kernel.check_report, rep.text, "{rel}");
            assert_eq!(node.kernel.exit, rep.code, "{rel}");
            n += 1;
        }
    }
    assert_eq!(n, 10, "1 accepted and 9 rejected files, exactly");
}

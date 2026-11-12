//! RLT-007b stage 1: obligation (a) of Lemma 12.1 (H45). The 44 generated
//! per-element distinctness theorems (pinned by tests/rlt007s1/manifest.sha256),
//! the four group summaries citing them, the bounds theorem, the rejected
//! cases, the generator's transcription checks, and refusal of a missing or
//! modified per-element theorem.

use std::path::{Path, PathBuf};
use std::sync::Once;

use rlt_cli::deps::deps;
use rlt_cli::explain::{explain, render_text};
use rlt_cli::{check_file, sha256_hex};

const ACCEPT: i32 = 0;
const MALFORMED: i32 = 2;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn manifest() -> Vec<(String, String)> {
    std::fs::read_to_string(root().join("tests/rlt007s1/manifest.sha256"))
        .unwrap()
        .lines()
        .map(|l| {
            let (s, p) = l.split_once("  ").unwrap();
            (s.to_string(), p.to_string())
        })
        .collect()
}

fn generator(arg: &str) -> std::process::Output {
    std::process::Command::new("python3")
        .args([
            "-I",
            "scripts/gen-rlt007s1.py",
            env!("CARGO_BIN_EXE_rlt"),
            arg,
        ])
        .current_dir(root())
        .output()
        .expect("python3 runs the generator")
}

static GEN: Once = Once::new();

fn materialized() {
    GEN.call_once(|| {
        if manifest().iter().any(|(_, p)| !root().join(p).is_file()) {
            assert!(generator("--materialize-only").status.success());
        }
        for (s, p) in manifest() {
            assert_eq!(
                sha256_hex(&std::fs::read(root().join(&p)).unwrap()),
                s,
                "{p} differs from the manifest"
            );
        }
    });
}

const VERDICT: &str =
    "verdict         ACCEPT — unconditional theorem relative to registered axiom set ROBINSON-Q";

#[test]
fn the_manifest_pins_44_accepted_per_element_theorems() {
    materialized();
    let m = manifest();
    assert_eq!(m.len(), 44);
    for (_, p) in &m {
        let r = check_file(&root(), &root().join(p));
        assert!(r.text.contains(VERDICT), "{p}:\n{}", r.text);
    }
}

#[test]
fn four_groups_and_the_bounds_theorem_are_accepted() {
    materialized();
    for k in 0..4 {
        let rel = format!("tests/rlt007s1/accept/DG_{k}.rlt");
        let d = deps(&root(), &rel);
        assert_eq!(d.exit, ACCEPT, "{rel}");
        assert_eq!(d.checks, 11, "{rel}: 11 of 64 cited-file checks");
        assert_eq!(d.chain, 1, "{rel}");
    }
    let r = check_file(&root(), &root().join("tests/rlt007s1/accept/BOUNDS.rlt"));
    assert!(r.text.contains(VERDICT), "{}", r.text);
    assert!(r.text.contains("assumptions     Γ = ∅"), "{}", r.text);
}

#[test]
fn every_rejected_case_is_refused_for_its_reason() {
    materialized();
    let tsv = std::fs::read_to_string(root().join("tests/rlt007s1/expected.tsv")).unwrap();
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
    assert_eq!(n, 7);
    let files = std::fs::read_dir(root().join("tests/rlt007s1/reject"))
        .unwrap()
        .count();
    assert_eq!(files, n, "every reject file is listed");
}

#[test]
fn the_generator_reproduces_every_output_and_refuses_altered_transcriptions() {
    materialized();
    let c = generator("--check");
    assert!(c.status.success(), "{}", String::from_utf8_lossy(&c.stdout));
    let t = generator("--self-test");
    let out = String::from_utf8_lossy(&t.stdout);
    assert!(t.status.success(), "{out}");
    assert!(
        out.contains("11 of 11 altered transcriptions refused"),
        "{out}"
    );
    assert!(!out.contains("ACCEPTED"), "{out}");
}

/// A scratch copy holding the stage-1 files only (plus what checking needs).
fn scratch(name: &str) -> PathBuf {
    let r = root().join("target/rlt007s1-scratch").join(name);
    let _ = std::fs::remove_dir_all(&r);
    for d in ["spec", "conventions", "sources", "tests/rlt007s1"] {
        copy_dir(&root().join(d), &r.join(d));
    }
    r
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let p = e.unwrap().path();
        let t = to.join(p.file_name().unwrap());
        if p.is_dir() {
            copy_dir(&p, &t);
        } else {
            std::fs::copy(&p, &t).unwrap();
        }
    }
}

#[test]
fn a_missing_or_modified_per_element_theorem_is_never_accepted() {
    materialized();
    let r = scratch("missing");
    let g = r.join("tests/rlt007s1/accept/DG_1.rlt");
    assert!(check_file(&r, &g).text.contains(VERDICT), "control");
    std::fs::remove_file(r.join("tests/rlt007s1/gen/D_060.rlt")).unwrap();
    let c = check_file(&r, &g);
    assert_eq!(c.code, MALFORMED, "{}", c.text);
    assert!(c
        .text
        .contains("cited path `tests/rlt007s1/gen/D_060.rlt`: entity not found"));

    let r = scratch("modified");
    let g = r.join("tests/rlt007s1/accept/DG_3.rlt");
    assert!(check_file(&r, &g).text.contains(VERDICT), "control");
    let p = r.join("tests/rlt007s1/gen/D_200.rlt");
    let s = std::fs::read_to_string(&p).unwrap();
    std::fs::write(&p, s.replacen("; RLT-007b", "; rlt-007b", 1)).unwrap();
    let c = check_file(&r, &g);
    assert_eq!(c.code, MALFORMED, "{}", c.text);
    assert!(c.text.contains("tests/rlt007s1/gen/D_200.rlt has SHA-256"));
}

#[test]
fn the_five_statement_records_are_valid_current_and_accepted() {
    materialized();
    let mut n = 0;
    for e in std::fs::read_dir(root().join("statements")).unwrap() {
        let f = e.unwrap().file_name().to_string_lossy().to_string();
        if !(f.starts_with("H45-DISTINCT-") || f.starts_with("H45-BOUNDS")) {
            continue;
        }
        let rel = format!("statements/{f}");
        let x = explain(&root(), &rel);
        let t = render_text(&x);
        assert_eq!(x.exit, ACCEPT, "{rel}:\n{t}");
        assert!(t.contains("partial"), "{rel}:\n{t}");
        n += 1;
    }
    assert_eq!(n, 5);
}

/// The small committed proofs agree between `rlt deps` and `rlt check` (the
/// sweep in cli/tests/deps.rs skips this stage's directory).
#[test]
fn deps_agrees_with_check_on_the_committed_proofs() {
    materialized();
    let mut n = 0;
    for d in ["tests/rlt007s1/accept", "tests/rlt007s1/reject"] {
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
    assert_eq!(n, 12, "5 accepted and 7 rejected files, exactly");
}

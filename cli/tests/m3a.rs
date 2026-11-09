//! RLT-003 M3a: the logical checker. Accepted controls A1–A15, rejected
//! cases R1–R37 (each with its required exit code and reason, from
//! tests/m3a/expected.tsv), the cases that need a scratch repository (R26,
//! R28), and the reporting obligation O9.

use rlt_cli::{check_file, check_file_outcome, check_file_outcome_with, ACCEPT, MALFORMED, REJECT};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    let manifest = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_string());
    Path::new(&manifest).parent().unwrap().to_path_buf()
}

fn rlt_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            rlt_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rlt") {
            out.push(p);
        }
    }
}

fn verdict(text: &str) -> &str {
    text.lines()
        .rev()
        .find(|l| l.starts_with("verdict"))
        .unwrap_or("")
}

#[test]
fn accepted_controls_are_unconditional_theorems() {
    let mut files = Vec::new();
    rlt_files(&root().join("tests/m3a/accept"), &mut files);
    files.sort();
    assert_eq!(files.len(), 16, "A1–A15 with A12 in two files");
    for f in &files {
        let r = check_file(&root(), f);
        assert_eq!(r.code, ACCEPT, "{}", r.text);
        assert!(
            r.text.contains("ACCEPT — unconditional theorem"),
            "{}",
            r.text
        );
        assert!(r.text.contains("assumptions     Γ = ∅"), "{}", r.text);
    }
}

#[test]
fn rejected_cases_fail_for_their_stated_reason() {
    let tsv = std::fs::read_to_string(root().join("tests/m3a/expected.tsv")).unwrap();
    let mut n = 0;
    for line in tsv.lines().filter(|l| !l.starts_with('#') && !l.is_empty()) {
        let cols: Vec<&str> = line.split('\t').collect();
        let (file, code, reason) = (cols[0], cols[1].parse::<i32>().unwrap(), cols[2]);
        let r = check_file(&root(), &root().join(file));
        assert_eq!(r.code, code, "{file}:\n{}", r.text);
        assert!(
            verdict(&r.text).contains(reason),
            "{file}: want `{reason}` in:\n{}",
            r.text
        );
        assert!(
            !r.text.contains("ACCEPT"),
            "{file} printed ACCEPT:\n{}",
            r.text
        );
        n += 1;
    }
    assert_eq!(
        n, 38,
        "R1–R37 with R12, R22 and R23 in two files each, R26 and R28 below"
    );
}

#[test]
fn r08_r33_capture_is_impossible_and_named_in_the_reason() {
    let r = check_file(
        &root(),
        &root().join("tests/m3a/reject/R08_forall_elim_capture_claim.rlt"),
    );
    assert!(r.text.contains("yields `∃y. b = y`"), "{}", r.text);
    let r = check_file(
        &root(),
        &root().join("tests/m3a/reject/R33_subst_capture_claim.rlt"),
    );
    assert!(r.text.contains("yields `∀v. v + c = v + 0`"), "{}", r.text);
}

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

#[test]
fn r26_an_axiom_file_edited_to_hold_a_parameter_breaks_its_pin() {
    let s = std::env::temp_dir().join(format!("rlt-m3a-r26-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&s);
    for d in ["sources", "conventions", "spec", "tests/m3a"] {
        copy_dir(&root().join(d), &s.join(d));
    }
    let f = s.join("spec/peano.rlt");
    let t = std::fs::read_to_string(&f).unwrap();
    std::fs::write(
        &f,
        t.replacen("(axiom add_zero (x)", "(axiom add_zero (x a)", 1),
    )
    .unwrap();
    let r = check_file(&s, &s.join("tests/m3a/accept/A01_forall_add_zero.rlt"));
    let _ = std::fs::remove_dir_all(&s);
    assert_eq!(r.code, MALFORMED, "{}", r.text);
    assert!(verdict(&r.text).contains("PEANO-ADD"), "{}", r.text);
}

#[test]
fn r28_a_formula_over_the_node_limit_is_refused() {
    let s = std::env::temp_dir().join(format!("rlt-m3a-r28-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&s);
    for d in ["sources", "conventions", "spec"] {
        copy_dir(&root().join(d), &s.join(d));
    }
    std::fs::create_dir_all(s.join("tests")).unwrap();
    let big = "(and (= 1000 1000) (= 1000 1000)) ".repeat(600);
    let text = format!(
        "(rlt-proof (id R28) (convention NAT-LOGIC) (goal \"0 = 0\") (mode unconditional) (axioms PEANO-ADD) (steps {}))",
        (0..2).map(|i| format!("(s{i} (and {big} (= 0 0)) (refl 0))")).collect::<Vec<_>>().join(" ")
    );
    std::fs::write(s.join("tests/R28.rlt"), text).unwrap();
    let r = check_file(&s, &s.join("tests/R28.rlt"));
    let _ = std::fs::remove_dir_all(&s);
    assert_eq!(r.code, MALFORMED, "{}", r.text);
    assert!(verdict(&r.text).contains("term nodes"), "{}", r.text);
}

#[test]
fn o9_result_kind_never_decides_a_verdict() {
    // Swap the reporting label for a constant and compare every verdict.
    fn constant(_: &rlt_kernel::Prop) -> &'static str {
        "SWAPPED"
    }
    let mut files = Vec::new();
    rlt_files(&root().join("tests"), &mut files);
    files.sort();
    assert!(files.len() > 250);
    for f in &files {
        let a = check_file_outcome(&root(), f).report;
        let b = check_file_outcome_with(&root(), f, constant).report;
        assert_eq!(a.code, b.code, "{}", f.display());
        assert_eq!(verdict(&a.text), verdict(&b.text), "{}", f.display());
    }
}

#[test]
fn earlier_proofs_keep_their_verdicts_under_nat_dec() {
    // Every earlier proof file names NAT-DEC, which reads equations only,
    // exactly as before M3a; none of them reads a logical symbol.
    let mut files = Vec::new();
    for d in ["tests/accept", "tests/reject", "tests/m1", "tests/m2"] {
        rlt_files(&root().join(d), &mut files);
    }
    let m1_text = std::fs::read_to_string(root().join("tests/m1/expected.tsv")).unwrap();
    let m1_expected: Vec<(String, i32)> = m1_text
        .lines()
        .filter(|l| l.starts_with("tests/"))
        .map(|l| {
            let mut c = l.split('\t');
            (
                c.next().unwrap().to_string(),
                c.next().unwrap().parse().unwrap(),
            )
        })
        .collect();
    assert_eq!(m1_expected.len(), 18);
    for f in &files {
        let text = std::fs::read_to_string(f).unwrap();
        assert!(!text.contains("NAT-LOGIC"), "{}", f.display());
        let r = check_file(&root(), f);
        // Each earlier file's expected exit code, independent of this run
        // (audit at 6ba9f1e, weakness 5): accept/ and m2/ accept, reject/
        // rejects, and m1/ follows its own expected.tsv.
        let rel = f.strip_prefix(root()).unwrap().display().to_string();
        let want = if rel.starts_with("tests/reject") {
            REJECT
        } else if rel.starts_with("tests/m1") {
            m1_expected
                .iter()
                .find(|(file, _)| *file == rel)
                .map(|(_, c)| *c)
                .unwrap_or(ACCEPT)
        } else {
            ACCEPT
        };
        assert_eq!(r.code, want, "{}", f.display());
    }
}

#[test]
fn o8_open_refuses_an_instantiation_over_the_node_limit() {
    // forall-elim of a body with many occurrences of its variable, opened
    // with a large term (fixture from the audit at 6ba9f1e): refused before
    // the result is built (O8).
    let text =
        std::fs::read_to_string(root().join("cli/tests/fixtures/O8_open_limit.rlt")).unwrap();
    let s = std::env::temp_dir().join(format!("rlt-m3a-o8-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&s);
    for d in ["sources", "conventions", "spec"] {
        copy_dir(&root().join(d), &s.join(d));
    }
    std::fs::create_dir_all(s.join("tests")).unwrap();
    std::fs::write(s.join("tests/O8.rlt"), text).unwrap();
    let r = check_file(&s, &s.join("tests/O8.rlt"));
    let _ = std::fs::remove_dir_all(&s);
    assert_eq!(r.code, REJECT, "{}", r.text);
    assert!(
        verdict(&r.text).contains("above the limit 1000000"),
        "{}",
        r.text
    );
}

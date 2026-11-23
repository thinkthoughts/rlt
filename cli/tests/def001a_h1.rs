//! RLT-DEF-001a audit H1: the exit of a multi-file `rlt check` is the most
//! severe file's code under ACCEPT (0) < EXPLORATORY (4) < REJECT (1) <
//! MALFORMED (2), in every file order, with and without
//! `--unregistered-definitions`; each file's report is unchanged.

use rlt_cli::{
    check_code_severity, sha256_hex, worse_check_code, ACCEPT, EXPLORATORY, MALFORMED, REJECT,
};
use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn scratch(name: &str, body: &str) -> String {
    let rel = format!("target/def001a-h1/{name}");
    let full = root().join(&rel);
    std::fs::create_dir_all(full.parent().unwrap()).unwrap();
    std::fs::write(&full, body).unwrap();
    rel
}

/// The audit's D2 file and a kernel-valid derivation of `double(2) = 3`
/// over it: exit 2 authoritatively, exit 4 exploratory.
fn exploratory_file() -> String {
    let defs = scratch(
        "evil.rlt",
        "(rlt-definitions\n  (id DOUBLE)\n  (convention NAT-LOGIC)\n  (domain Nat)\n  (define-function double (x) (+ x 1)\n    (written \"double(x) := x + 1\")\n    (origin rlt-authored)))\n",
    );
    let sha = sha256_hex(&std::fs::read(root().join(&defs)).unwrap());
    scratch(
        "evil_two.rlt",
        &format!(
            "(rlt-proof\n  (id H1.evil_two)\n  (convention NAT-FUN)\n  (definitions \"{defs}\" \"{sha}\")\n  (goal \"double(2) = 3\")\n  (mode unconditional)\n  (axioms ROBINSON-Q)\n  (steps\n    (s1 (= (+ 2 1) (S (+ 2 0))) (inst add_succ (x 2) (y 0)))\n    (s2 (= (+ 2 0) 2) (inst add_zero (x 2)))\n    (s3 (= (S (+ 2 0)) 3) (cong S s2))\n    (s4 (= (+ 2 1) 3) (trans s1 s3))))\n"
        ),
    )
}

/// (label, path, authoritative exit, exploratory exit)
fn fixtures() -> Vec<(&'static str, String, i32, i32)> {
    vec![
        (
            "registered",
            "tests/definitions/certificates/F3_double_two.rlt".into(),
            ACCEPT,
            ACCEPT,
        ),
        (
            "plain",
            "tests/accept/M1_comm_instance.rlt".into(),
            ACCEPT,
            ACCEPT,
        ),
        ("exploratory", exploratory_file(), MALFORMED, EXPLORATORY),
        (
            "rejected",
            "tests/reject/T2_math5_broken_trans.rlt".into(),
            REJECT,
            REJECT,
        ),
        (
            "malformed",
            scratch("malformed.rlt", "(rlt-proof (id H1.malformed))\n"),
            MALFORMED,
            MALFORMED,
        ),
    ]
}

fn rlt(args: &[&str]) -> (String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_rlt"))
        .arg("--root")
        .arg(root())
        .args(args)
        .output()
        .unwrap();
    (
        String::from_utf8(out.stdout).unwrap(),
        out.status.code().unwrap(),
    )
}

fn check(explore: bool, files: &[&str]) -> (String, i32) {
    let mut a = vec!["check"];
    if explore {
        a.push("--unregistered-definitions");
    }
    a.extend_from_slice(files);
    rlt(&a)
}

fn rank(c: i32) -> u8 {
    match c {
        0 => 0,
        4 => 1,
        1 => 2,
        2 => 3,
        _ => unreachable!(),
    }
}

#[test]
fn h1_severity_order_is_accept_exploratory_reject_malformed() {
    let order = [ACCEPT, EXPLORATORY, REJECT, MALFORMED];
    for (i, a) in order.iter().enumerate() {
        assert_eq!(check_code_severity(*a), i as u8);
        for (j, b) in order.iter().enumerate() {
            let want = if i >= j { *a } else { *b };
            assert_eq!(worse_check_code(*a, *b), want, "{a} vs {b}");
        }
    }
    // Unknown codes fail closed, above MALFORMED.
    assert_eq!(worse_check_code(MALFORMED, 7), 7);
    // Authoritative codes: the numeric maximum, as before H1.
    for a in [ACCEPT, REJECT, MALFORMED] {
        for b in [ACCEPT, REJECT, MALFORMED] {
            assert_eq!(worse_check_code(a, b), a.max(b));
        }
    }
}

#[test]
fn h1_each_file_alone_has_its_expected_exit() {
    for (label, f, auth, expl) in fixtures() {
        assert_eq!(check(false, &[&f]).1, auth, "{label} authoritative");
        assert_eq!(check(true, &[&f]).1, expl, "{label} exploratory");
    }
}

/// Every ordered pair and every ordered triple of distinct fixtures, in both
/// modes: the exit is the most severe file's, and stdout is exactly the
/// single-file reports joined by one blank line.
#[test]
fn h1_multi_file_exit_is_the_most_severe_in_every_order() {
    let fx = fixtures();
    let single: Vec<[(String, i32); 2]> = fx
        .iter()
        .map(|(_, f, _, _)| [check(false, &[f]), check(true, &[f])])
        .collect();
    let n = fx.len();
    let mut combos: Vec<Vec<usize>> = Vec::new();
    for i in 0..n {
        for j in 0..n {
            if i != j {
                combos.push(vec![i, j]);
                for k in 0..n {
                    if k != i && k != j {
                        combos.push(vec![i, j, k]);
                    }
                }
            }
        }
    }
    assert_eq!(combos.len(), 20 + 60);
    for combo in &combos {
        let files: Vec<&str> = combo.iter().map(|&i| fx[i].1.as_str()).collect();
        let labels: Vec<&str> = combo.iter().map(|&i| fx[i].0).collect();
        for (mode, explore) in [(0usize, false), (1usize, true)] {
            let (text, code) = check(explore, &files);
            let codes: Vec<i32> = combo.iter().map(|&i| single[i][mode].1).collect();
            let want = *codes.iter().max_by_key(|c| rank(**c)).unwrap();
            assert_eq!(code, want, "{labels:?} explore={explore}: codes {codes:?}");
            if !explore {
                // Authoritative runs keep the pre-H1 rule exactly.
                assert_eq!(code, *codes.iter().max().unwrap(), "{labels:?}");
            }
            let joined: Vec<&str> = combo.iter().map(|&i| single[i][mode].0.as_str()).collect();
            assert_eq!(
                text,
                joined.join("\n"),
                "{labels:?} explore={explore}: reports"
            );
        }
    }
}

/// The audit's two H1 reproductions.
#[test]
fn h1_audit_reproductions() {
    let fx = fixtures();
    let get = |l: &str| fx.iter().find(|x| x.0 == l).unwrap().1.clone();
    let (m, e, r) = (get("malformed"), get("exploratory"), get("rejected"));
    assert_eq!(check(true, &[&m, &e]).1, MALFORMED);
    assert_eq!(check(true, &[&e, &m, &r]).1, MALFORMED);
    assert_eq!(check(true, &[&e, &r]).1, REJECT);
    assert_eq!(check(true, &[&r, &e]).1, REJECT);
    assert_eq!(check(true, &[&e, &get("registered")]).1, EXPLORATORY);
}

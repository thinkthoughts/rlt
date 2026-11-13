//! RLT-007b stage 4: H45 omits a class modulo each prime 47 … 211. The 1,485
//! generated certificates (pinned by tests/rlt007s4/manifest.sha256), the 33
//! per-prime theorems citing them, their statements rebuilt independently,
//! the rejected cases, the generator's checks, the records, and refusal of a
//! missing or modified certificate and of an altered source pin.

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

fn generator(arg: &str) -> std::process::Output {
    std::process::Command::new("python3")
        .args([
            "-I",
            "scripts/gen-rlt007s4.py",
            env!("CARGO_BIN_EXE_rlt"),
            arg,
        ])
        .current_dir(root())
        .output()
        .expect("python3 runs the generator")
}

fn manifest() -> Vec<(String, String)> {
    std::fs::read_to_string(root().join("tests/rlt007s4/manifest.sha256"))
        .unwrap()
        .lines()
        .map(|l| {
            let (s, p) = l.split_once("  ").unwrap();
            (s.to_string(), p.to_string())
        })
        .collect()
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

/// H45 as printed in the paper (§12), written out here independently.
const H45: [u32; 45] = [
    0, 2, 12, 14, 24, 26, 30, 36, 44, 50, 54, 56, 60, 66, 72, 74, 80, 84, 92, 96, 102, 110, 114,
    116, 122, 126, 134, 140, 144, 150, 156, 162, 164, 170, 176, 180, 182, 186, 192, 194, 200, 204,
    206, 210, 212,
];

fn is_prime(n: u32) -> bool {
    n > 1 && (2..n).all(|d| !n.is_multiple_of(d))
}

/// The primes 47 … 211 and the least class H45 omits modulo each.
fn table() -> Vec<(u32, u32)> {
    (47..213)
        .filter(|&p| is_prime(p))
        .map(|p| {
            let r = (0..p).find(|&c| H45.iter().all(|&h| h % p != c)).unwrap();
            (p, r)
        })
        .collect()
}

fn goal(rel: &str) -> String {
    let t = std::fs::read_to_string(root().join(rel)).unwrap();
    let i = t.find("(goal \"").unwrap() + 7;
    t[i..i + t[i..].find('"').unwrap()].to_string()
}

#[test]
fn the_manifest_pins_1485_certificates_with_the_approved_statements() {
    materialized();
    let m = manifest();
    assert_eq!(m.len(), 1485);
    let mut n = 0;
    for (p, r) in table() {
        for h in H45 {
            let rel = format!("tests/rlt007s4/gen/D_{p:03}_{h:03}.rlt");
            assert_eq!(goal(&rel), format!("¬(∃k. {h} = ({p} × k) + {r})"), "{rel}");
            n += 1;
        }
    }
    assert_eq!(n, 1485);
}

#[test]
fn a_sample_of_certificates_is_accepted() {
    materialized();
    // The full family is checked by scripts/verify.sh; here the largest
    // (199, 212), the first and last of each class, and the h < r cases.
    for (p, h) in [
        (199, 212),
        (47, 0),
        (211, 212),
        (53, 2),
        (71, 2),
        (113, 0),
        (59, 212),
    ] {
        let rel = format!("tests/rlt007s4/gen/D_{p:03}_{h:03}.rlt");
        let r = check_file(&root(), &root().join(&rel));
        assert!(r.text.contains(VERDICT), "{rel}:\n{}", r.text);
    }
}

#[test]
fn the_33_summaries_state_exactly_the_approved_omissions() {
    let t = table();
    assert_eq!(t.len(), 33);
    for (p, r) in t {
        let om = |h: u32| format!("¬(∃k. {h} = ({p} × k) + {r})");
        let mut c = om(H45[0]);
        for &h in &H45[1..] {
            c = format!("({c} ∧ {})", om(h));
        }
        assert_eq!(
            goal(&format!("tests/rlt007s4/accept/OM_{p:03}.rlt")),
            c,
            "OM_{p}"
        );
    }
    let files = std::fs::read_dir(root().join("tests/rlt007s4/accept"))
        .unwrap()
        .count();
    assert_eq!(files, 33);
}

#[test]
fn summaries_are_accepted_within_the_citation_budget() {
    materialized();
    for p in [47, 199, 211] {
        let rel = format!("tests/rlt007s4/accept/OM_{p:03}.rlt");
        let d = deps(&root(), &rel);
        assert_eq!((d.exit, d.checks, d.chain), (ACCEPT, 45, 1), "{rel}");
    }
}

#[test]
fn every_rejected_case_is_refused_for_its_reason() {
    materialized();
    let tsv = std::fs::read_to_string(root().join("tests/rlt007s4/expected.tsv")).unwrap();
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
    let files = std::fs::read_dir(root().join("tests/rlt007s4/reject"))
        .unwrap()
        .count();
    assert_eq!(files, n);
}

#[test]
fn the_generator_reproduces_every_output_and_refuses_altered_inputs() {
    materialized();
    let c = generator("--check");
    assert!(c.status.success(), "{}", String::from_utf8_lossy(&c.stdout));
    let t = generator("--self-test");
    let out = String::from_utf8_lossy(&t.stdout);
    assert!(t.status.success(), "{out}");
    assert!(out.contains("7 of 7 altered inputs refused"), "{out}");
    assert!(!out.contains("ACCEPTED"), "{out}");
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
fn a_missing_or_modified_certificate_is_never_accepted() {
    materialized();
    let r = root().join("target/rlt007s4-scratch/cert");
    let _ = std::fs::remove_dir_all(&r);
    for d in ["spec", "conventions", "sources"] {
        copy_dir(&root().join(d), &r.join(d));
    }
    std::fs::create_dir_all(r.join("tests/rlt007s4/accept")).unwrap();
    std::fs::copy(
        root().join("tests/rlt007s4/accept/OM_211.rlt"),
        r.join("tests/rlt007s4/accept/OM_211.rlt"),
    )
    .unwrap();
    std::fs::create_dir_all(r.join("tests/rlt007s4/gen")).unwrap();
    for h in H45 {
        let f = format!("tests/rlt007s4/gen/D_211_{h:03}.rlt");
        std::fs::copy(root().join(&f), r.join(&f)).unwrap();
    }
    let s = r.join("tests/rlt007s4/accept/OM_211.rlt");
    assert!(check_file(&r, &s).text.contains(VERDICT), "control");
    let p = r.join("tests/rlt007s4/gen/D_211_000.rlt");
    let t = std::fs::read_to_string(&p).unwrap();
    std::fs::write(&p, t.replacen("; RLT-007b", "; rlt-007b", 1)).unwrap();
    let c = check_file(&r, &s);
    assert_eq!(c.code, MALFORMED, "{}", c.text);
    assert!(c
        .text
        .contains("tests/rlt007s4/gen/D_211_000.rlt has SHA-256"));
    std::fs::remove_file(r.join("tests/rlt007s4/gen/D_211_212.rlt")).unwrap();
    std::fs::write(&p, t).unwrap();
    let c = check_file(&r, &s);
    assert_eq!(c.code, MALFORMED, "{}", c.text);
    assert!(c
        .text
        .contains("cited path `tests/rlt007s4/gen/D_211_212.rlt`: entity not found"));
}

#[test]
fn records_are_current_and_an_altered_source_pin_is_refused() {
    materialized();
    // scripts/verify.sh explains all 33; here a sample (each explain re-checks
    // the 45 cited certificates).
    let mut n = 0;
    for p in [47, 199, 211] {
        let rel = format!("statements/OMITS-H45-{p:03}.yaml");
        let x = explain(&root(), &rel);
        let t = render_text(&x);
        assert_eq!(x.exit, ACCEPT, "{rel}:\n{t}");
        assert!(
            t.contains("SEMANTIC CORRESPONDENCE: partial · assessment current"),
            "{rel}:\n{t}"
        );
        n += 1;
    }
    assert_eq!(n, 3);
    let all = std::fs::read_dir(root().join("statements"))
        .unwrap()
        .filter(|e| {
            e.as_ref()
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with("OMITS-H45-")
        })
        .count();
    assert_eq!(all, 33);
    let dir = root().join("target/rlt007s4-pin");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let rec = std::fs::read_to_string(root().join("statements/OMITS-H45-211.yaml")).unwrap();
    std::fs::write(
        dir.join("OMITS-H45-211.yaml"),
        rec.replacen("  commit: 2b307ae2", "  commit: 3b307ae2", 1),
    )
    .unwrap();
    let x = explain(&root(), "target/rlt007s4-pin/OMITS-H45-211.yaml");
    let t = render_text(&x);
    assert_eq!(x.exit, MALFORMED, "{t}");
    assert!(t.contains("differs from the registry's"), "{t}");
}

/// `rlt deps` agrees with `rlt check` on the committed proofs of this stage
/// (the sweep in cli/tests/deps.rs skips this directory).
#[test]
fn deps_agrees_with_check_on_the_committed_rejects() {
    materialized();
    let mut fs: Vec<_> = std::fs::read_dir(root().join("tests/rlt007s4/reject"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    fs.sort();
    for f in &fs {
        let rel = f
            .strip_prefix(root())
            .unwrap()
            .to_string_lossy()
            .to_string();
        let dd = deps(&root(), &rel);
        let rep = check_file(&root(), f);
        let node = dd.root.as_ref().expect("a root node");
        assert_eq!(node.kernel.check_report, rep.text, "{rel}");
        assert_eq!(node.kernel.exit, rep.code, "{rel}");
    }
    assert_eq!(fs.len(), 7);
}

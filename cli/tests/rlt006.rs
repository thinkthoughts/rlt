//! RLT-006: the H45 residue certificates (Axiom Math, September 3, 2026 draft,
//! Lemma 12.1).
//!
//! The 630 element certificates are generated, not committed. Tests that need
//! them materialize them with scripts/gen-rlt006.py when absent and then check
//! every file against tests/rlt006/manifest.sha256 before use.
//!
//! - the manifest pins 630 certificates, and every one is accepted;
//! - the 14 per-prime theorems are accepted with 45 of 64 cited-file checks;
//! - rejected cases give their pinned exit codes and reasons;
//! - a missing or modified certificate makes the summary, `rlt deps` and the
//!   library verifier fail;
//! - `rlt validate` refuses a malformed transcribed source;
//! - the statement record reports the four outcomes.

use rlt_cli::deps::{deps, verify_lib};
use rlt_cli::explain::{explain, render_text};
use rlt_cli::{check_file, sha256_hex, validate, ACCEPT, MALFORMED};
use std::path::{Path, PathBuf};
use std::sync::Once;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn manifest() -> Vec<(String, String)> {
    std::fs::read_to_string(root().join("tests/rlt006/manifest.sha256"))
        .unwrap()
        .lines()
        .map(|l| {
            let (s, p) = l.split_once("  ").unwrap();
            (s.to_string(), p.to_string())
        })
        .collect()
}

static GEN: Once = Once::new();

/// Materialize the certificates if any is absent, then require every file to
/// match the manifest.
fn materialized() {
    GEN.call_once(|| {
        if manifest().iter().any(|(_, p)| !root().join(p).is_file()) {
            let st = std::process::Command::new("python3")
                .args([
                    "-I",
                    "scripts/gen-rlt006.py",
                    env!("CARGO_BIN_EXE_rlt"),
                    "--materialize-only",
                ])
                .current_dir(root())
                .status()
                .expect("python3 runs the generator");
            assert!(
                st.success(),
                "scripts/gen-rlt006.py --materialize-only failed"
            );
        }
        for (s, p) in manifest() {
            let bytes = std::fs::read(root().join(&p)).unwrap();
            assert_eq!(sha256_hex(&bytes), s, "{p} differs from the manifest");
        }
    });
}

#[test]
fn the_manifest_pins_630_accepted_certificates() {
    materialized();
    let m = manifest();
    assert_eq!(m.len(), 630);
    for (_, p) in &m {
        let r = check_file(&root(), &root().join(p));
        assert!(
            r.text.contains(
                "verdict         ACCEPT — unconditional theorem relative to registered axiom set ROBINSON-Q"
            ),
            "{p}:\n{}",
            r.text
        );
    }
}

#[test]
fn fourteen_per_prime_theorems_within_the_citation_budget() {
    materialized();
    let mut n = 0;
    for e in std::fs::read_dir(root().join("tests/rlt006/accept")).unwrap() {
        let p = e.unwrap().path();
        let rel = p
            .strip_prefix(root())
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let d = deps(&root(), &rel);
        assert_eq!(d.exit, ACCEPT, "{rel}");
        assert_eq!(d.distinct, 46, "{rel}: itself and 45 certificates");
        assert_eq!(d.checks, 45, "{rel}: 45 of 64 cited-file checks");
        assert_eq!(d.chain, 1, "{rel}");
        let t = rlt_cli::deps::render_text(&d);
        assert!(
            t.contains("budget          cited-file checks 45 of 64 · citing-file chain 1 of 8"),
            "{t}"
        );
        n += 1;
    }
    assert_eq!(n, 14);
}

#[test]
fn every_rejected_case_is_refused_for_its_reason() {
    materialized();
    let tsv = std::fs::read_to_string(root().join("tests/rlt006/expected.tsv")).unwrap();
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
    assert_eq!(n, 6);
    let files = std::fs::read_dir(root().join("tests/rlt006/reject"))
        .unwrap()
        .count();
    assert_eq!(files, n, "every reject file is listed");
}

fn copy_dir(from: &Path, to: &Path, skip: &dyn Fn(&Path) -> bool) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        if skip(&e.path()) {
            continue;
        }
        let t = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &t, skip);
        } else {
            std::fs::copy(e.path(), &t).unwrap();
        }
    }
}

/// A scratch repository with everything but the generated certificates,
/// then only the 45 certificates for p = 43.
fn scratch(name: &str) -> PathBuf {
    let r = root().join("target/rlt006-scratch").join(name);
    let _ = std::fs::remove_dir_all(&r);
    let gen = root().join("tests/rlt006/gen");
    for d in [
        "spec",
        "conventions",
        "readingpoints",
        "sources",
        "correspondence",
        "residues",
        "points",
        "lifts",
        "tests",
        "lib",
        "statements",
        "docs",
        "lean",
    ] {
        if root().join(d).exists() {
            copy_dir(&root().join(d), &r.join(d), &|p: &Path| p == gen);
        }
    }
    std::fs::create_dir_all(r.join("tests/rlt006/gen")).unwrap();
    for e in std::fs::read_dir(&gen).unwrap() {
        let p = e.unwrap().path();
        if p.file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("O_43_")
        {
            std::fs::copy(&p, r.join("tests/rlt006/gen").join(p.file_name().unwrap())).unwrap();
        }
    }
    r
}

/// A scratch repository with all 630 certificates (copied once per case).
fn full_scratch(name: &str) -> PathBuf {
    let r = scratch(name);
    for e in std::fs::read_dir(root().join("tests/rlt006/gen")).unwrap() {
        let p = e.unwrap().path();
        std::fs::copy(&p, r.join("tests/rlt006/gen").join(p.file_name().unwrap())).unwrap();
    }
    r
}

/// The library verifier's failing entries, by id.
fn failing(r: &Path) -> Vec<String> {
    let v = verify_lib(r);
    assert_ne!(v.exit, ACCEPT);
    v.entries
        .iter()
        .filter(|e| e.1 != ACCEPT)
        .map(|e| e.0.clone())
        .collect()
}

#[test]
fn a_missing_or_modified_certificate_is_never_accepted() {
    materialized();
    // Missing: exactly the one affected entry fails.
    let r = full_scratch("missing");
    assert_eq!(
        verify_lib(&r).exit,
        ACCEPT,
        "the full scratch tree verifies"
    );
    std::fs::remove_file(r.join("tests/rlt006/gen/O_43_212.rlt")).unwrap();
    let c = check_file(&r, &r.join("tests/rlt006/accept/OM_43.rlt"));
    assert_eq!(c.code, MALFORMED, "{}", c.text);
    assert!(c
        .text
        .contains("cited path `tests/rlt006/gen/O_43_212.rlt`: entity not found"));
    assert_ne!(deps(&r, "admissible.h45.omits.43_3").exit, ACCEPT);
    assert_eq!(failing(&r), ["admissible.h45.omits.43_3"]);
    // Modified: one byte changed; exactly the one affected entry fails.
    let r = full_scratch("modified");
    let p = r.join("tests/rlt006/gen/O_07_102.rlt");
    let s = std::fs::read_to_string(&p).unwrap();
    std::fs::write(&p, s.replacen("; RLT-006", "; rlt-006", 1)).unwrap();
    let c = check_file(&r, &r.join("tests/rlt006/accept/OM_07.rlt"));
    assert_eq!(c.code, MALFORMED, "{}", c.text);
    assert!(c.text.contains("tests/rlt006/gen/O_07_102.rlt has SHA-256"));
    assert_ne!(deps(&r, "admissible.h45.omits.7_6").exit, ACCEPT);
    assert_eq!(failing(&r), ["admissible.h45.omits.7_6"]);
}

#[test]
fn an_unlisted_item_names_exactly_one_path() {
    // Two citations merged into one `uses` item, and an unlisted path with
    // characters outside the allowed set, are refused when the index loads.
    let r = scratch("unlisted");
    let p = r.join("lib/admissible/admissible.h45.omits.43_3.yaml");
    let s = std::fs::read_to_string(&p).unwrap();
    let merged = s.replacen(
        "  - (unlisted tests/rlt006/gen/O_43_000.rlt)\n  - (unlisted tests/rlt006/gen/O_43_002.rlt)\n",
        "  - (unlisted tests/rlt006/gen/O_43_000.rlt), (unlisted tests/rlt006/gen/O_43_002.rlt)\n",
        1,
    );
    assert_ne!(merged, s);
    std::fs::write(&p, &merged).unwrap();
    let e = rlt_cli::deps::load_library(&r).err().expect("refused");
    assert!(e.contains("must name one repository path"), "{e}");
    let odd = s.replacen(
        "(unlisted tests/rlt006/gen/O_43_000.rlt)",
        "(unlisted tests/rlt006/gen/O 43.rlt)",
        1,
    );
    std::fs::write(&p, &odd).unwrap();
    assert!(rlt_cli::deps::load_library(&r).is_err());
}

#[test]
fn validate_refuses_a_malformed_transcribed_source() {
    let cases = [
        ("commit: 2b307ae2", "commit: 2B307AE2"),
        (
            "url: sources/transcriptions/AXIOM-BGP212-2026-09-03.yaml",
            "url: sources/transcriptions/ABSENT.yaml",
        ),
        (
            "url: sources/transcriptions/AXIOM-BGP212-2026-09-03.yaml",
            "url: sources/transcriptions/../registry.yaml",
        ),
        (
            "location: https://primegaps.axiommath.ai/bgp212.pdf",
            "location: http://primegaps.axiommath.ai/bgp212.pdf",
        ),
    ];
    for (i, (from, to)) in cases.iter().enumerate() {
        let r = root()
            .join("target/rlt006-scratch/validate")
            .join(i.to_string());
        let _ = std::fs::remove_dir_all(&r);
        let gen = root().join("tests/rlt006/gen");
        for d in [
            "spec",
            "conventions",
            "readingpoints",
            "sources",
            "correspondence",
            "residues",
            "points",
            "lifts",
            "tests",
        ] {
            if root().join(d).exists() {
                copy_dir(&root().join(d), &r.join(d), &|p: &Path| p == gen);
            }
        }
        let p = r.join("sources/registry.yaml");
        let s = std::fs::read_to_string(&p).unwrap();
        assert!(s.contains(from), "{from}");
        std::fs::write(&p, s.replacen(from, to, 1)).unwrap();
        let v = validate(&r);
        assert_ne!(v.code, ACCEPT, "case {i}");
        assert!(
            v.text
                .contains("AXIOM-BGP212-2026-09-03: status transcribed requires"),
            "case {i}:\n{}",
            v.text
        );
    }
}

#[test]
fn explain_reports_the_four_outcomes() {
    materialized();
    let x = explain(&root(), "statements/H45-OMITS-43.yaml");
    let t = render_text(&x);
    assert_eq!(x.exit, ACCEPT, "{t}");
    for line in [
        "1 KERNEL ACCEPTANCE: ACCEPT",
        "2 SOURCE FIDELITY: registry verified · pinned",
        "  source        AXIOM-BGP212-2026-09-03 (transcribed) @ 2b307ae26046dffd2153fe7466e5a9a302976dc3ba5142151a8a8104a37b3d80",
        "3 SEMANTIC CORRESPONDENCE: partial · assessment current",
        "4 PHYSICAL CORRESPONDENCE: not applicable",
        "  library       admissible.h45.omits.43_3",
    ] {
        assert!(t.contains(line), "missing `{line}` in\n{t}");
    }
}

#[test]
fn record_fixtures_give_their_exit_codes() {
    materialized();
    let tsv = std::fs::read_to_string(root().join("tests/explain-rlt006/expected.tsv")).unwrap();
    let mut n = 0;
    for l in tsv.lines().filter(|l| !l.starts_with('#') && !l.is_empty()) {
        let f: Vec<&str> = l.splitn(3, '\t').collect();
        let x = explain(&root(), &format!("tests/explain-rlt006/{}", f[0]));
        let t = render_text(&x);
        assert_eq!(x.exit.to_string(), f[1], "{}:\n{t}", f[0]);
        assert!(t.contains(f[2]), "{}: `{}` not in\n{t}", f[0], f[2]);
        n += 1;
    }
    assert_eq!(n, 6);
}

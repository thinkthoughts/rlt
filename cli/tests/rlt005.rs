//! RLT-005: published mathematics — 8 is not matchable (McNew and Pomerance,
//! arXiv:2604.05304v2, §1).
//!
//! - Every obligation accepts as an unconditional theorem under Robinson's Q,
//!   and every rejected case is refused with its pinned exit code and reason.
//! - `rlt deps matchable.not.8`: the complete graph, from the one check.
//! - `rlt explain statements/MATCHABLE-NOT-8.yaml`: current, pinned archived
//!   source, attributed `partial` assessment, physical not applicable.
//! - `rlt validate` refuses a malformed archived source (scratch registry).

use rlt_cli::deps::{deps, load_library};
use rlt_cli::explain::{explain, render_text};
use rlt_cli::{check_file, validate, ACCEPT};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn every_obligation_is_an_unconditional_theorem() {
    let mut n = 0;
    for e in std::fs::read_dir(root().join("tests/rlt005/accept")).unwrap() {
        let p = e.unwrap().path();
        let r = check_file(&root(), &p);
        assert_eq!(r.code, ACCEPT, "{}:\n{}", p.display(), r.text);
        assert!(
            r.text.contains(
                "verdict         ACCEPT — unconditional theorem relative to registered axiom set ROBINSON-Q"
            ),
            "{}",
            p.display()
        );
        n += 1;
    }
    assert_eq!(n, 19);
}

#[test]
fn every_rejected_case_is_refused_for_its_reason() {
    let tsv = std::fs::read_to_string(root().join("tests/rlt005/expected.tsv")).unwrap();
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
    assert_eq!(n, 10);
}

#[test]
fn the_dependency_graph_of_matchable_not_8() {
    let d = deps(&root(), "matchable.not.8");
    assert_eq!(d.exit, ACCEPT);
    assert_eq!(d.distinct, 19, "19 distinct theorems");
    assert_eq!(d.checks, 29, "29 cited-file checks");
    assert_eq!(d.chain, 4, "citing-file chain 4");
    let lib = load_library(&root()).unwrap();
    let e = lib.get("matchable.not.8").unwrap();
    assert!(e.citable);
    assert_eq!(
        e.uses,
        [
            "matchable.divisors.8",
            "matchable.partners.4",
            "matchable.not.8.contradiction"
        ]
    );
    // The central impossibility argument is checked, not asserted: MC cites
    // the three odd-partner lemmas and the pigeonhole lemma.
    let mc = lib.get("matchable.not.8.contradiction").unwrap();
    for u in [
        "matchable.odd_partner.2",
        "matchable.odd_partner.4",
        "matchable.odd_partner.8",
        "matchable.pigeonhole.1_3",
    ] {
        assert!(mc.uses.iter().any(|x| x == u), "MC uses {u}");
    }
}

#[test]
fn explain_reports_the_four_outcomes() {
    let x = explain(&root(), "statements/MATCHABLE-NOT-8.yaml");
    let t = render_text(&x);
    assert_eq!(x.exit, ACCEPT, "{t}");
    for line in [
        "1 KERNEL ACCEPTANCE: ACCEPT",
        "2 SOURCE FIDELITY: registry verified · pinned",
        "  source        MCNEW-POMERANCE-MATCHABLE-V2 (archived) @ 21674e5fb69c9a58b187bd51a4efc41330e6eee4d764c3f77882a29fa3aaef16",
        "3 SEMANTIC CORRESPONDENCE: partial · assessment current",
        "4 PHYSICAL CORRESPONDENCE: not applicable",
        "  library       matchable.not.8",
    ] {
        assert!(t.contains(line), "missing `{line}` in\n{t}");
    }
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        // RLT-006 generated certificates are never copied into scratch trees.
        if e.path().ends_with("tests/rlt006/gen") || e.path().ends_with("tests/rlt007s1/gen") {
            continue;
        }
        let t = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &t);
        } else {
            std::fs::copy(e.path(), &t).unwrap();
        }
    }
}

#[test]
fn validate_refuses_a_malformed_archived_source() {
    let cases = [
        ("commit: 21674e5f", "commit: 21674E5F"),
        (
            "url: https://export.arxiv.org/e-print/2604.05304v2#match051426.tex",
            "url: https://export.arxiv.org/e-print/2604.05304v2",
        ),
        (
            "url: https://export.arxiv.org/e-print/2604.05304v2#match051426.tex",
            "url: http://export.arxiv.org/e-print/2604.05304v2#match051426.tex",
        ),
        (
            "url: https://export.arxiv.org/e-print/2604.05304v2#match051426.tex",
            "url: https://export.arxiv.org/e-print/2604.05304v2#../match051426.tex",
        ),
        (
            "url: https://export.arxiv.org/e-print/2604.05304v2#match051426.tex",
            "url: https://export.arxiv.org/e-print/2604.05304v2#/match051426.tex",
        ),
    ];
    for (i, (from, to)) in cases.iter().enumerate() {
        let r = root().join("target/rlt005-scratch").join(i.to_string());
        let _ = std::fs::remove_dir_all(&r);
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
                copy_dir(&root().join(d), &r.join(d));
            }
        }
        let p = r.join("sources/registry.yaml");
        let s = std::fs::read_to_string(&p).unwrap();
        assert!(s.contains(from));
        std::fs::write(&p, s.replacen(from, to, 1)).unwrap();
        let v = validate(&r);
        assert_ne!(v.code, ACCEPT, "case {i}");
        assert!(
            v.text
                .contains("MCNEW-POMERANCE-MATCHABLE-V2: status archived requires"),
            "case {i}:\n{}",
            v.text
        );
        // The pinned registry digest refuses the change as well.
        assert!(v
            .text
            .contains("differs from the registry pinned in this binary"));
    }
}

//! RLT-003 M3b: Robinson's Q, divisibility and ordering.
//!
//! The accepted demonstrations (tests/m3b/accept) and rejected cases
//! (tests/m3b/expected.tsv), plus the cases that need a scratch repository,
//! the kernel API or a report check: B-R1, B-R2, B-R3, B-R9, B-R11, B-R12,
//! B-R17, B-R18, B-R23, B-R26, B-R27, B-R30 and B-R32.

use rlt_cli::{
    check_file, definition_source_problem, load_axiom_set, load_definitions, pinned_registry,
    residues::residues_report, shared_axiom_problem, validate, ACCEPT, MALFORMED, REJECT,
};
use rlt_kernel::{rules, Axiom, Prop, Term};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    let manifest = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_string());
    Path::new(&manifest).parent().unwrap().to_path_buf()
}

fn files(dir: &str) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(root().join(dir))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "rlt"))
        .collect();
    v.sort();
    v
}

fn verdict(text: &str) -> &str {
    text.lines()
        .find(|l| l.starts_with("verdict"))
        .unwrap_or("")
}

#[test]
fn accepted_demonstrations_are_theorems_under_robinson_q() {
    let fs = files("tests/m3b/accept");
    assert_eq!(fs.len(), 15, "B-A1 to B-A15");
    for f in &fs {
        let r = check_file(&root(), f);
        assert_eq!(r.code, ACCEPT, "{}", r.text);
        let conditional = f.to_string_lossy().contains("BA11");
        let want = if conditional {
            "ACCEPT — conditional theorem relative to registered axiom set ROBINSON-Q"
        } else {
            "ACCEPT — unconditional theorem relative to registered axiom set ROBINSON-Q"
        };
        assert!(verdict(&r.text).contains(want), "{}", r.text);
    }
}

#[test]
fn rejected_cases_fail_for_their_stated_reason() {
    let tsv = std::fs::read_to_string(root().join("tests/m3b/expected.tsv")).unwrap();
    let mut n = 0;
    for line in tsv.lines().filter(|l| l.starts_with("tests/")) {
        let mut c = line.split('\t');
        let (f, code, reason) = (c.next().unwrap(), c.next().unwrap(), c.next().unwrap());
        let r = check_file(&root(), &root().join(f));
        assert_eq!(r.code.to_string(), code, "{f}:\n{}", r.text);
        assert!(
            verdict(&r.text).contains(reason),
            "{f}: want `{reason}`:\n{}",
            r.text
        );
        n += 1;
    }
    assert_eq!(n, files("tests/m3b/reject").len());
    assert_eq!(n, 23);
}

// ── Scratch-repository cases ───────────────────────────────────────────────

fn copy_dir(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for e in std::fs::read_dir(src).unwrap() {
        let p = e.unwrap().path();
        // RLT-006 generated certificates are never copied into scratch trees.
        if p.ends_with("tests/rlt006/gen") {
            continue;
        }
        let to = dst.join(p.file_name().unwrap());
        if p.is_dir() {
            copy_dir(&p, &to);
        } else {
            std::fs::copy(&p, &to).unwrap();
        }
    }
}

fn scratch(name: &str, edits: &[(&str, &str, &str)]) -> PathBuf {
    let s = std::env::temp_dir().join(format!("rlt-m3b-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&s);
    for d in [
        "sources",
        "conventions",
        "correspondence",
        "readingpoints",
        "points",
        "spec",
        "residues",
        "tests",
    ] {
        copy_dir(&root().join(d), &s.join(d));
    }
    for (path, from, to) in edits {
        let f = s.join(path);
        let text = std::fs::read_to_string(&f).unwrap();
        assert!(
            text.contains(from),
            "fixture edit target missing in {path}: {from}"
        );
        std::fs::write(&f, text.replacen(from, to, 1)).unwrap();
    }
    s
}

fn check_in(s: &Path, f: &str) -> rlt_cli::Report {
    check_file(s, &s.join(f))
}

const BA01: &str = "tests/m3b/accept/BA01_q2_instance.rlt";
const BA06: &str = "tests/m3b/accept/BA06_two_divides_eight.rlt";

#[test]
fn b_r1_an_edited_axiom_breaks_its_pin() {
    let s = scratch(
        "r1",
        &[("spec/robinson-q.rlt", "(not (= 0 (S x)))", "(not (= 0 x))")],
    );
    let r = check_in(&s, BA01);
    let _ = std::fs::remove_dir_all(&s);
    assert_eq!(r.code, MALFORMED, "{}", r.text);
    assert!(
        verdict(&r.text).contains("the axiom file changed without review"),
        "{}",
        r.text
    );
}

#[test]
fn b_r2_a_registered_hash_edited_to_match_differs_from_the_binary() {
    let s = scratch(
        "r2",
        &[
            ("spec/robinson-q.rlt", "(not (= 0 (S x)))", "(not (= 0 x))"),
            (
                "spec/axioms.yaml",
                "4ad55192222576e3901abc4ee7ea5a6760b64ff212e8c55796ab9847e423b00c",
                "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            ),
        ],
    );
    let r = check_in(&s, BA01);
    let _ = std::fs::remove_dir_all(&s);
    assert_eq!(r.code, MALFORMED, "{}", r.text);
    assert!(
        verdict(&r.text).contains("differs from the pinned"),
        "{}",
        r.text
    );
}

#[test]
fn b_r3_shared_axioms_must_equal_peano_arith() {
    let q = load_axiom_set(&root(), "ROBINSON-Q").unwrap();
    let pa = load_axiom_set(&root(), "PEANO-ARITH").unwrap();
    let names = ["add_zero", "add_succ", "mul_zero", "mul_succ"];
    assert!(
        shared_axiom_problem("ROBINSON-Q", &q.axioms, "PEANO-ARITH", &pa.axioms, &names).is_ok()
    );
    // A substituted add_zero (x + 0 = 0), as a hand-built binary would carry it.
    let x = Term::Var("x".into());
    let bad = Axiom::new(
        "add_zero",
        vec!["x".into()],
        rlt_kernel::Formula::new(Term::plus(x, Term::zero()), Term::zero()),
        "OLP-ROBINSON-Q",
    )
    .unwrap();
    let mut altered: Vec<Axiom> = q
        .axioms
        .iter()
        .filter(|a| a.name() != "add_zero")
        .cloned()
        .collect();
    altered.push(bad);
    let e = shared_axiom_problem("ROBINSON-Q", &altered, "PEANO-ARITH", &pa.axioms, &names)
        .unwrap_err();
    assert!(
        e.contains("add_zero differs from PEANO-ARITH's add_zero"),
        "{e}"
    );
}

#[test]
fn b_r9_edited_definitions_break_their_pin() {
    let s = scratch(
        "r9",
        &[(
            "spec/definitions.rlt",
            "(exists k (= n (* d k)))",
            "(exists k (= n (* k d)))",
        )],
    );
    let r = check_in(&s, BA06);
    let _ = std::fs::remove_dir_all(&s);
    assert_eq!(r.code, MALFORMED, "{}", r.text);
    assert!(
        verdict(&r.text).contains("the definitions changed without review"),
        "{}",
        r.text
    );
}

#[test]
fn b_r11_b_r12_b_r23_recursive_cyclic_and_open_definitions_are_refused() {
    use rlt_syntax::defs::parse_definitions;
    let rec = r#"(define f (x) (f x) (notation prefix "f") (origin rlt-authored))"#;
    assert!(parse_definitions(rec, &[]).is_err());
    let cyc = r#"(define a (x) (b x) (notation prefix "a") (origin rlt-authored))
(define b (x) (a x) (notation prefix "b") (origin rlt-authored))"#;
    assert!(parse_definitions(cyc, &[]).is_err());
    let open = r#"(define g (x) (= x y) (notation prefix "g") (origin rlt-authored))"#;
    assert!(parse_definitions(open, &[]).is_err());
}

#[test]
fn b_r17_source_substitution_is_refused() {
    use rlt_syntax::defs::parse_definitions;
    let reg = pinned_registry(&root()).unwrap();
    let ok = load_definitions(&root()).unwrap();
    assert!(definition_source_problem(&ok, &reg).is_ok());
    let bad = parse_definitions(
        r#"(define lt (x y) (exists z (= (+ (S z) x) y)) (notation infix "<") (source OLP-INVENTED))"#,
        &[],
    )
    .unwrap();
    let e = definition_source_problem(&bad, &reg).unwrap_err();
    assert!(e.contains("OLP-INVENTED"), "{e}");
    // The registry's Q8 excerpt edited: the registry pin refuses it.
    let s = scratch(
        "r17",
        &[(
            "sources/registry.yaml",
            "\\eq[(z'' + x)][y]",
            "\\eq[(x + z'')][y]",
        )],
    );
    let r = check_in(&s, BA06);
    let v = validate(&s);
    let _ = std::fs::remove_dir_all(&s);
    assert_ne!(r.code, ACCEPT, "{}", r.text);
    assert!(
        v.text
            .contains("differs from the registry pinned in this binary"),
        "{}",
        v.text
    );
}

#[test]
fn b_r18_a_residue_record_cannot_claim_a_theorem() {
    for key in ["backed_by: M3.free235_7", "theorem: free235(7)"] {
        let s = scratch(
            "r18",
            &[(
                "residues/residue-30.yaml",
                "references: [math29]",
                &format!("references: [math29]\n{key}"),
            )],
        );
        let r = residues_report(&s, &s.join("residues/residue-30.yaml"));
        let _ = std::fs::remove_dir_all(&s);
        assert_eq!(r.code, MALFORMED, "{key}: {}", r.text);
    }
}

#[test]
fn b_r26_a_bound_index_is_never_an_axiom_binding() {
    let q = load_axiom_set(&root(), "ROBINSON-Q").unwrap();
    let q3 = q
        .axioms
        .iter()
        .find(|a| a.name() == "q3_zero_or_succ")
        .unwrap();
    let e = rules::inst(q3, &[("x".into(), Term::Bound(0))]).unwrap_err();
    assert!(e.contains("bound index"), "{e}");
    // A parameter binding substitutes under Q3's own binder without capture.
    let th = rules::inst(q3, &[("x".into(), Term::Param("y".into()))]).unwrap();
    assert_eq!(th.conclusion().to_string(), "y = 0 ∨ (∃y′. y = S(y′))");
    assert!(th.conclusion().check_wf(0).is_ok());
    // Nested quantifiers: an axiom variable under two binders.
    let two = Axiom::formula(
        "nested",
        vec!["x".into()],
        Prop::Forall(
            "u".into(),
            Box::new(Prop::Exists(
                "v".into(),
                Box::new(Prop::eq(
                    Term::plus(Term::Var("x".into()), Term::Bound(1)),
                    Term::Bound(0),
                )),
            )),
        ),
        "t",
    )
    .unwrap();
    let th = rules::inst(
        &two,
        &[(
            "x".into(),
            Term::plus(Term::Param("a".into()), Term::numeral(1)),
        )],
    )
    .unwrap();
    assert_eq!(th.conclusion().to_string(), "∀u. ∃v. (a + 1) + u = v");
    assert!(th.conclusion().check_wf(0).is_ok());
}

#[test]
fn b_r27_an_axiom_variable_shadowed_by_its_own_binder_is_refused() {
    let src = "(axiom bad (x) (exists x (= x 0)) (source OLP-ROBINSON-Q))";
    let e = rlt_syntax::proof::parse_axiom_file(src).unwrap_err();
    assert!(e.contains("shadowed name `x`"), "{e}");
    let src = "(axiom bad (x) (exists y (= x z)) (source OLP-ROBINSON-Q))";
    assert!(rlt_syntax::proof::parse_axiom_file(src).is_err());
    let src = "(axiom bad (x) (= (S x) a) (source OLP-ROBINSON-Q))";
    assert!(rlt_syntax::proof::parse_axiom_file(src).is_err());
}

#[test]
fn b_r30_reports_separate_the_registered_set_from_the_axioms_used() {
    let r = check_file(&root(), &root().join(BA01));
    assert!(
        r.text
            .contains("axiom set       ROBINSON-Q (spec/robinson-q.rlt, source OLP-ROBINSON-Q)"),
        "{}",
        r.text
    );
    assert_eq!(
        r.text.matches("\n  axiom         ").count(),
        7,
        "{}",
        r.text
    );
    assert!(
        r.text.contains("\naxioms used     q2_zero_ne_succ\n"),
        "{}",
        r.text
    );
    let r = check_file(&root(), &root().join(BA06));
    let used = r
        .text
        .lines()
        .find(|l| l.starts_with("axioms used"))
        .unwrap();
    assert!(
        !used.contains("q1") && !used.contains("q2") && !used.contains("q3"),
        "{used}"
    );
    assert!(
        used.contains("mul_succ") && used.contains("mul_zero"),
        "{used}"
    );
    assert!(
        r.text
            .contains("definitions     dvd — abbreviations expanded before the kernel"),
        "{}",
        r.text
    );
    assert!(r.text.contains("  folded        2 ∣ 8\n"), "{}", r.text);
    assert!(
        r.text.contains("checked goal    ∃k. 8 = 2 × k\n"),
        "{}",
        r.text
    );
}

#[test]
fn b_r32_a_quoted_convention_reads_the_definitions_too() {
    // Audit W1: `(convention "NAT-DEF")` names the same convention as
    // `(convention NAT-DEF)`, so the definition names are reserved in both.
    let body = |conv: &str| {
        format!(
            "(rlt-proof\n  (id ATK.A17)\n  (convention {conv})\n  (goal \"∀x. x < x + 1 → x < x + 1\")\n  (mode unconditional)\n  (axioms ROBINSON-Q)\n  (params dvd)\n  (steps\n    (a (exists z (= (+ (S z) dvd) (+ dvd 1))) (hyp h))\n    (b (imp (exists z (= (+ (S z) dvd) (+ dvd 1))) (exists z (= (+ (S z) dvd) (+ dvd 1)))) (imp-intro h a))\n    (c (forall lt (imp (exists z (= (+ (S z) lt) (+ lt 1))) (exists z (= (+ (S z) lt) (+ lt 1))))) (forall-intro dvd b))))\n"
        )
    };
    let s = scratch("b-r32", &[]);
    for (name, conv) in [("sym", "NAT-DEF"), ("str", "\"NAT-DEF\"")] {
        let f = format!("tests/m3b/b_r32_{name}.rlt");
        std::fs::write(s.join(&f), body(conv)).unwrap();
        let r = check_in(&s, &f);
        assert_eq!(r.code, MALFORMED, "{name}: {}", r.text);
        assert!(
            r.text
                .contains("parameter `dvd` collides with a definition name"),
            "{name}: {}",
            r.text
        );
    }
}

#[test]
fn earlier_axiom_sets_and_conventions_are_unchanged() {
    let pa = std::fs::read(root().join("spec/peano-arith.rlt")).unwrap();
    let pe = std::fs::read(root().join("spec/peano.rlt")).unwrap();
    use sha2::{Digest, Sha256};
    let h = |b: &[u8]| format!("{:x}", Sha256::digest(b));
    assert_eq!(
        h(&pa),
        "00ceffa9a60b53286f350edbf2f1a5f48a14169a9b09a82cbedc6f9956ff01e4"
    );
    assert_eq!(
        h(&pe),
        "1a3f83444a06fdfbd02c9af2a5881b47137ba102a675a4e012c84be477b10a0d"
    );
    // NAT-DEF reads every NAT-LOGIC formula to the same kernel term.
    let convs = rlt_cli::pinned_conventions(&root()).unwrap();
    for g in [
        "∀x. x + 0 = x",
        "2 × 3 = 6 ∧ 6 × 5 = 30",
        "(∃x. x = 1) → ∃y. y = 1",
        "⊥ → 0 = 1",
    ] {
        let w = rlt_syntax::written::Written::new(g);
        let mut nat_def = convs["NAT-DEF"].convention.clone();
        nat_def.defs = Some(std::rc::Rc::new(load_definitions(&root()).unwrap()));
        let a = rlt_syntax::proof::elaborate_goal(&w, &convs["NAT-LOGIC"].convention).unwrap();
        let b = rlt_syntax::proof::elaborate_goal(&w, &nat_def).unwrap();
        assert_eq!(a, b, "{g}");
        assert_eq!(a.kernel_form(), b.kernel_form(), "{g}");
    }
    let _ = REJECT;
}

//! RLT-PROVER-002: registered function definitions. Stage 1 and 2 goals,
//! definition-aware discovery, and the ten Phase 5 adversarial cases. Every
//! proof-to-definition correspondence check must fail closed.

use rlt_cli::prove::{prove_source, render, Verdict};
use rlt_cli::{check_file_outcome, sha256_hex, ACCEPT};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    let manifest = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_string());
    Path::new(&manifest).parent().unwrap().to_path_buf()
}

fn read(p: &Path) -> String {
    std::fs::read_to_string(p).unwrap()
}

/// A scratch file under target/ (repository-relative path returned).
fn scratch(name: &str, body: &str) -> String {
    let rel = format!("target/definitions-tests/{name}");
    let full = root().join(&rel);
    std::fs::create_dir_all(full.parent().unwrap()).unwrap();
    std::fs::write(&full, body).unwrap();
    rel
}

fn sha(rel: &str) -> String {
    sha256_hex(&std::fs::read(root().join(rel)).unwrap())
}

const DOUBLE_FN: &str =
    r#"(define-function double (x) (+ x x) (written "double(x) := x + x") (origin rlt-authored))"#;

fn registry(id: &str, defs: &str) -> String {
    format!("(rlt-definitions\n  (id {id})\n  (convention NAT-LOGIC)\n  (domain Nat)\n  {defs})\n")
}

fn goal(id: &str, defs_rel: &str, statement: &str) -> String {
    format!(
        "(rlt-goal\n  (id {id})\n  (convention NAT-FUN)\n  (definitions \"{defs_rel}\" \"{}\")\n  (goal \"{statement}\")\n  (mode unconditional)\n  (axioms ROBINSON-Q))\n",
        sha(defs_rel)
    )
}

fn prove(src: &str, tag: &str) -> (rlt_cli::prove::ProveOutcome, String) {
    let out = format!("target/definitions-tests/{tag}.rlt");
    let r = prove_source(&root(), src, &[], Some(Path::new(&out)));
    (r, out)
}

fn text(r: &rlt_cli::prove::ProveOutcome) -> String {
    let mut t = String::new();
    render(&mut t, r);
    t
}

fn check(rel: &str) -> rlt_cli::CheckOutcome {
    check_file_outcome(&root(), &root().join(rel))
}

// ── 1. A valid definition and theorem (stages 1 and 2) ────────────────────

#[test]
fn case1_stage_goals_are_proved_replayed_and_reproduced() {
    let expect = [
        (
            "F1_double_unfolds",
            "∀x. double(x) = x + x",
            "∀x. x + x = x + x",
            "none",
        ),
        ("F2_double_zero", "double(0) = 0", "0 + 0 = 0", "add_zero"),
        (
            "F3_double_two",
            "double(2) = 4",
            "2 + 2 = 4",
            "add_succ, add_zero",
        ),
        (
            "F4_double_succ",
            "∀x. x + (x + 1) = double(x) + 1",
            "∀x. x + (x + 1) = (x + x) + 1",
            "add_succ, add_zero",
        ),
        (
            "F5_double_succ_nonzero",
            "∀x. ¬(double(x + 1) = 0)",
            "∀x. ¬((x + 1) + (x + 1) = 0)",
            "add_succ, add_zero, q2_zero_ne_succ",
        ),
        (
            "F6_quadruple_unfolds",
            "∀x. quadruple(x) = double(x) + double(x)",
            "∀x. (x + x) + (x + x) = (x + x) + (x + x)",
            "none",
        ),
        (
            "F7_quadruple_one",
            "quadruple(1) = 4",
            "(1 + 1) + (1 + 1) = 4",
            "add_succ, add_zero",
        ),
    ];
    for (name, folded, unfolded, axioms) in expect {
        let g = root().join(format!("tests/definitions/goals/{name}.goal"));
        let src = read(&g);
        assert!(!src.contains("(steps"), "goal files supply no proof");
        let (r, out) = prove(&src, name);
        let Verdict::Proved { theorem, .. } = &r.verdict else {
            panic!("{name}: {}", text(&r));
        };
        assert_eq!(r.goal_shown.as_deref(), Some(unfolded), "{name}");
        assert_eq!(
            r.folded_shown.as_ref().map(|f| f.0.as_str()),
            Some(folded),
            "{name}"
        );
        let ax: Vec<&str> = theorem.axioms_used().keys().map(|s| s.as_str()).collect();
        assert_eq!(
            if ax.is_empty() {
                "none".to_string()
            } else {
                ax.join(", ")
            },
            axioms,
            "{name}"
        );
        // The committed certificate is reproduced byte for byte and replays.
        let committed = format!("tests/definitions/certificates/{name}.rlt");
        assert_eq!(
            read(&root().join(&out)),
            read(&root().join(&committed)),
            "{name}"
        );
        let c = check(&committed);
        assert_eq!(c.report.code, ACCEPT, "{name}:\n{}", c.report.text);
        assert!(
            c.report
                .text
                .contains(&format!("  folded        {folded}\n")),
            "{}",
            c.report.text
        );
        assert!(c
            .report
            .text
            .contains("definitions add no axiom, rule or symbol"));
    }
    // Stage 2's double(0) = 0 is one kernel step: Q4 at x := 0.
    let f2 = read(&root().join("tests/definitions/certificates/F2_double_zero.rlt"));
    assert!(
        f2.contains("(s1 (= (+ 0 0) 0) (inst add_zero (x 0)))))"),
        "{f2}"
    );
}

// ── 2. Unfolding under quantifiers without capture ────────────────────────

#[test]
fn case2_unfolding_under_quantifiers_never_captures() {
    // The parameter is named y; uses sit under binders named x and y.
    let defs = scratch(
        "case2.rlt",
        &registry(
            "CASE-TWO",
            r#"(define-function inc (y) (+ y 1) (written "inc(y) := y + 1") (origin rlt-authored))"#,
        ),
    );
    let cases = [
        (
            "∀x. ∀y. inc(y) + x = inc(x) + y",
            "∀x. ∀y. (y + 1) + x = (x + 1) + y",
        ),
        (
            "∀y. ∀x. inc(x) = inc(y) → x = y",
            "∀y. ∀x. (x + 1 = y + 1 → x = y)",
        ),
        (
            "∀x. ∃y. inc(y) = inc(inc(x))",
            "∀x. ∃y. y + 1 = (x + 1) + 1",
        ),
    ];
    for (i, (st, unfolded)) in cases.iter().enumerate() {
        let (r, _) = prove(&goal(&format!("c2.{i}"), &defs, st), &format!("c2_{i}"));
        assert_eq!(
            r.goal_shown.as_deref(),
            Some(*unfolded),
            "{st}: {}",
            text(&r)
        );
    }
    // The last is provable only with the witness x + 1 for y, so the binder
    // structure is what the kernel checked.
    let (r, _) = prove(&goal("c2.w", &defs, cases[2].0), "c2_w");
    assert!(matches!(r.verdict, Verdict::Proved { .. }), "{}", text(&r));
}

// ── 3, 4, 6, 8, 9 (definitions files): refused before any proof ───────────

fn refused_registry(tag: &str, body: &str, needle: &str) {
    let defs = scratch(&format!("{tag}.rlt"), body);
    let (r, _) = prove(&goal(tag, &defs, "∀x. double(x) = x + x"), tag);
    let t = text(&r);
    assert!(matches!(r.verdict, Verdict::Error(_)), "{tag}: {t}");
    assert!(t.contains(needle), "{tag}: expected `{needle}` in:\n{t}");
}

#[test]
fn case3_recursive_and_cyclic_definitions_are_refused() {
    refused_registry(
        "c3_rec",
        &registry(
            "C-REC",
            r#"(define-function double (x) (+ (double x) 0) (written "double(x) := double(x) + 0") (origin rlt-authored))"#,
        ),
        "recursive and cyclic definitions are refused",
    );
    refused_registry(
        "c3_cyc",
        &registry(
            "C-CYC",
            r#"(define-function double (x) (half x) (written "double(x) := half(x)") (origin rlt-authored))
  (define-function half (x) (double x) (written "half(x) := double(x)") (origin rlt-authored))"#,
        ),
        "undefined symbol `half`",
    );
}

#[test]
fn case4_an_unbound_variable_is_refused() {
    refused_registry(
        "c4",
        &registry(
            "C-UNBOUND",
            r#"(define-function double (x) (+ x y) (written "double(x) := x + y") (origin rlt-authored))"#,
        ),
        "unbound variable or undefined constant `y`",
    );
    // Arity and undefined symbols too.
    refused_registry(
        "c4_arity",
        &registry(
            "C-ARITY",
            r#"(define-function double (x) (+ x) (written "double(x) := x") (origin rlt-authored))"#,
        ),
        "has arity 2",
    );
    // And a written form that disagrees with the formal body.
    refused_registry(
        "c4_written",
        &registry(
            "C-WRITTEN",
            r#"(define-function double (x) (+ x x) (written "double(x) := 2 × x") (origin rlt-authored))"#,
        ),
        "but the formal body is",
    );
}

#[test]
fn case6_conflicting_definitions_of_one_name_are_refused() {
    refused_registry(
        "c6",
        &registry("C-TWICE", &format!("{DOUBLE_FN}\n  (define-function double (x) (* 2 x) (written \"double(x) := 2 × x\") (origin rlt-authored))")),
        "defined twice",
    );
    // A name the pinned M3b definitions or the axioms already take.
    refused_registry(
        "c6_lt",
        &registry("C-LT", &format!("{DOUBLE_FN}\n  (define-function lt (x) (+ x 1) (written \"lt(x) := x + 1\") (origin rlt-authored))")),
        "collides with",
    );
}

#[test]
fn case8_an_attempted_axiom_is_refused() {
    refused_registry(
        "c8_file",
        &registry(
            "C-AXIOM",
            &format!(
                "{DOUBLE_FN}\n  (axiom double_zero (x) (= (+ x x) 0) (source OLP-ROBINSON-Q))"
            ),
        ),
        "adds no axiom",
    );
    refused_registry(
        "c8_inner",
        &registry(
            "C-AXIOM2",
            r#"(define-function double (x) (+ x x) (written "double(x) := x + x") (origin rlt-authored) (axiom d (x) (= (+ x x) 0)))"#,
        ),
        "asserts nothing",
    );
    // A certificate step citing a "definitional axiom" is refused by the kernel.
    let f1 = read(&root().join("tests/definitions/certificates/F1_double_unfolds.rlt"));
    let forged = f1.replace(
        "(steps\n",
        "(steps\n    (s0 (= (+ 0 0) 0) (inst double_def (x 0)))\n",
    );
    let p = scratch("c8_cert.rlt", &forged);
    assert_ne!(check(&p).report.code, ACCEPT);
}

#[test]
fn case9_reading_point_relationship_and_context_labels_are_refused() {
    for label in [
        "(readingpoint math29)",
        "(relationship math29.pairing)",
        "(context residues mod30)",
    ] {
        refused_registry(
            &format!("c9_{}", &label[1..5]),
            &registry("C-LABEL", &format!("{DOUBLE_FN}\n  {label}")),
            "carries no reading-point, relationship or context label",
        );
    }
    // An unregistered relationship asserted on a valid definition proof.
    let f1 = read(&root().join("tests/definitions/certificates/F1_double_unfolds.rlt"));
    for label in [
        "(relationship double.unregistered)",
        "(readingpoint double)",
    ] {
        let p = scratch(
            "c9_cert.rlt",
            &f1.replace(
                "  (mode unconditional)",
                &format!("  {label}\n  (mode unconditional)"),
            ),
        );
        let c = check(&p);
        assert_ne!(c.report.code, ACCEPT, "{label}");
        assert!(c.report.text.contains("no registered"), "{}", c.report.text);
    }
}

// ── 5. A definition changed after the proof was generated ────────────────

#[test]
fn case5_a_changed_definition_invalidates_the_proof() {
    let defs = scratch("c5_defs.rlt", &registry("C-CHANGE", DOUBLE_FN));
    let (r, cert) = prove(&goal("c5", &defs, "double(0) = 0"), "c5");
    assert!(matches!(r.verdict, Verdict::Proved { .. }), "{}", text(&r));
    assert_eq!(check(&cert).report.code, ACCEPT);
    // Same name, new meaning: double(x) := x + 1. The stated hash no longer
    // matches, so nothing is checked against the new definitions.
    scratch(
        "c5_defs.rlt",
        &registry(
            "C-CHANGE",
            r#"(define-function double (x) (+ x 1) (written "double(x) := x + 1") (origin rlt-authored))"#,
        ),
    );
    let c = check(&cert);
    assert_ne!(c.report.code, ACCEPT);
    assert!(
        c.report
            .text
            .contains("changed after this statement was written"),
        "{}",
        c.report.text
    );
    // A whitespace-only edit changes the bytes, and is refused too.
    scratch(
        "c5_defs.rlt",
        &format!("{} ", registry("C-CHANGE", DOUBLE_FN)),
    );
    assert_ne!(check(&cert).report.code, ACCEPT);
}

// ── 7. A valid certificate attached to the wrong definition ──────────────

#[test]
fn case7_a_certificate_bound_to_another_definition_is_refused() {
    let f1_rel = "tests/definitions/certificates/F1_double_unfolds.rlt";
    let f1 = read(&root().join(f1_rel));
    assert_eq!(check(f1_rel).report.code, ACCEPT);
    let real = sha("definitions/double.rlt");
    // Another registry where double means something else, with its true hash.
    let other = scratch(
        "c7_other.rlt",
        &registry(
            "OTHER",
            r#"(define-function double (x) (* 2 x) (written "double(x) := 2 × x") (origin rlt-authored))"#,
        ),
    );
    let swapped = f1.replace(
        &format!("\"definitions/double.rlt\" \"{real}\""),
        &format!("\"{other}\" \"{}\"", sha(&other)),
    );
    assert_ne!(swapped, f1);
    let p = scratch("c7_swapped.rlt", &swapped);
    let c = check(&p);
    assert_ne!(c.report.code, ACCEPT, "{}", c.report.text);
    // The right path with a wrong hash.
    let p = scratch("c7_hash.rlt", &f1.replace(&real, &"0".repeat(64)));
    assert_ne!(check(&p).report.code, ACCEPT);
    // The definitions header removed (NAT-FUN without definitions), or kept
    // under another convention.
    let line = format!("  (definitions \"definitions/double.rlt\" \"{real}\")\n");
    let p = scratch("c7_none.rlt", &f1.replace(&line, ""));
    assert_ne!(check(&p).report.code, ACCEPT);
    let p = scratch(
        "c7_logic.rlt",
        &f1.replace("(convention NAT-FUN)", "(convention NAT-LOGIC)"),
    );
    let c = check(&p);
    assert_ne!(c.report.code, ACCEPT);
    assert!(
        c.report.text.contains("reads no registered functions"),
        "{}",
        c.report.text
    );
    // A symlinked or out-of-repository definitions path.
    let p = scratch(
        "c7_path.rlt",
        &f1.replace("\"definitions/double.rlt\"", "\"../double.rlt\""),
    );
    assert_ne!(check(&p).report.code, ACCEPT);
}

// ── 10. A theorem beyond the search bounds ───────────────────────────────

#[test]
fn case10_goals_outside_the_bounds_are_not_found_without_a_truth_claim() {
    for name in ["G1_double_two_times", "G2_double_times_two"] {
        let src = read(&root().join(format!("tests/definitions/notfound/{name}.goal")));
        let (r, _) = prove(&src, name);
        let t = text(&r);
        assert!(matches!(r.verdict, Verdict::NotFound), "{name}: {t}");
        assert!(
            t.contains("this says nothing about whether the goal is true, false or independent")
        );
        for w in ["FALSE", "DISPROVED", "REFUTED"] {
            assert!(!t.contains(w), "{name}");
        }
    }
    // With too small a budget, a provable goal is also only "not found".
    let src = read(&root().join("tests/definitions/goals/F5_double_succ_nonzero.goal"));
    let r = prove_source(
        &root(),
        &src,
        &[("budget".into(), 1)],
        Some(Path::new("target/definitions-tests/c10.rlt")),
    );
    assert!(matches!(r.verdict, Verdict::NotFound), "{}", text(&r));
}

// ── Discovery ────────────────────────────────────────────────────────────

fn copy_record(tag: &str) -> String {
    let src = root().join("research/definition-discovery");
    let rel = format!("target/definitions-tests/{tag}");
    let dst = root().join(&rel);
    let _ = std::fs::remove_dir_all(&dst);
    std::fs::create_dir_all(&dst).unwrap();
    for e in std::fs::read_dir(&src).unwrap() {
        let e = e.unwrap();
        let body = read(&e.path());
        // Paths in the record name the directory itself.
        let name = e.file_name().to_string_lossy().to_string();
        let body = if name.ends_with(".rlt") {
            body
        } else {
            body.replace("research/definition-discovery", &rel)
        };
        std::fs::write(dst.join(&name), body).unwrap();
    }
    rel
}

#[test]
fn discovery_record_verifies_and_every_tamper_is_refused() {
    let (t, code) = rlt_cli::discover_defs::verify(&root(), "research/definition-discovery");
    assert_eq!(code, 0, "{t}");
    assert!(t.contains("126 proved (4 unfolding, 122 inference)"), "{t}");
    let m = read(&root().join("research/definition-discovery/manifest.tsv"));
    for row in ["∀x. double(x) = x + x", "double(0) = 0\t"] {
        assert!(m.contains(row), "{row}");
    }
    let nf = read(&root().join("research/definition-discovery/not-found.tsv"));
    assert!(nf.contains("∀x. double(x) = 2 × x\tNOT_FOUND_WITHIN_BOUNDS"));

    // A copy verifies as it stands.
    let ok = copy_record("disc_ok");
    assert_eq!(rlt_cli::discover_defs::verify(&root(), &ok).1, 0);

    type Tamper = fn(&str) -> String;
    let tampers: [(&str, &str, Tamper); 6] = [
        // An inference theorem reclassified as mere unfolding.
        ("manifest.tsv", "kind", |m| {
            m.replacen("\tinference\t", "\tunfolding\t", 1)
        }),
        // A definition identity that is not the registered one.
        ("manifest.tsv", "definitions", |m| {
            m.replacen("double@", "double@0", 1)
        }),
        // A different elaborated statement.
        ("manifest.tsv", "elaborated", |m| {
            m.replacen("0 + 0 = 0", "0 = 0", 1)
        }),
        // A fabricated row for a candidate listed as not found.
        ("manifest.tsv", "fabricated", |m| {
            format!("{m}DF0033\t∀x. double(x) = 2 × x\tx\tx\tROBINSON-Q\tunconditional\t∅\tx\tx\t1\tnone\tunfolding\t0\n")
        }),
        // The search metadata is outside the bound.
        ("manifest.tsv", "depth", |m| m.replacen("\t0\n", "\t9\n", 1)),
        // The summary names a different definitions hash.
        ("summary.txt", "summary", |s| {
            s.replacen("sha256 8", "sha256 9", 1)
        }),
    ];
    for (file, what, f) in tampers {
        let dir = copy_record(&format!("disc_{what}"));
        let p = root().join(&dir).join(file);
        let body = read(&p);
        let changed = f(&body);
        assert_ne!(changed, body, "{what}: the tamper changed nothing");
        std::fs::write(&p, changed).unwrap();
        let (t, code) = rlt_cli::discover_defs::verify(&root(), &dir);
        assert_ne!(code, 0, "{what}: tampered record verified:\n{t}");
    }

    // A certificate with an added context label, rehashed into the manifest.
    let dir = copy_record("disc_label");
    let mpath = root().join(&dir).join("manifest.tsv");
    let m = read(&mpath);
    let row = m.lines().find(|l| l.starts_with("DF")).unwrap().to_string();
    let id = row.split('\t').next().unwrap();
    let cpath = root().join(&dir).join(format!("{id}.rlt"));
    let old = std::fs::read(&cpath).unwrap();
    let new = read(&cpath).replace(
        "  (mode unconditional)",
        "  (context residues mod30)\n  (mode unconditional)",
    );
    std::fs::write(&cpath, &new).unwrap();
    std::fs::write(
        &mpath,
        m.replace(&sha256_hex(&old), &sha256_hex(new.as_bytes())),
    )
    .unwrap();
    let (t, code) = rlt_cli::discover_defs::verify(&root(), &dir);
    assert_ne!(code, 0, "{t}");

    // The definitions file changed after discovery: the whole record is refused.
    let dir = copy_record("disc_defs");
    let defs = scratch("disc_defs.rlt", &registry("DOUBLE", DOUBLE_FN));
    for n in ["manifest.tsv", "summary.txt"] {
        let p = root().join(&dir).join(n);
        let b = read(&p).replace("definitions/double.rlt", &defs);
        std::fs::write(&p, b).unwrap();
    }
    scratch(
        "disc_defs.rlt",
        &registry(
            "DOUBLE",
            r#"(define-function double (x) (+ x 1) (written "double(x) := x + 1") (origin rlt-authored))"#,
        ),
    );
    let (t, code) = rlt_cli::discover_defs::verify(&root(), &dir);
    assert_ne!(code, 0, "{t}");
    assert!(t.contains("changed after"), "{t}");
}

#[test]
fn discovery_reproduces_the_committed_record() {
    let dir = "target/definitions-tests/disc_rerun";
    let _ = std::fs::remove_dir_all(root().join(dir));
    let (_, code) = rlt_cli::discover_defs::discover(&root(), "definitions/double.rlt", dir, None);
    assert_eq!(code, 0);
    let committed = root().join("research/definition-discovery");
    for e in std::fs::read_dir(&committed).unwrap() {
        let e = e.unwrap();
        let name = e.file_name().to_string_lossy().to_string();
        let want = read(&e.path()).replace("research/definition-discovery", dir);
        assert_eq!(read(&root().join(dir).join(&name)), want, "{name}");
    }
    // Writing into the committed record is refused.
    let tracked = rlt_cli::prove::tracked_by_git(
        &root(),
        &root().join("research/definition-discovery/manifest.tsv"),
    );
    if tracked != Ok(true) {
        return;
    }
    let (t, code) = rlt_cli::discover_defs::discover(
        &root(),
        "definitions/double.rlt",
        "research/definition-discovery",
        Some(1),
    );
    assert_eq!(code, 2, "{t}");
}

#[test]
fn earlier_conventions_read_exactly_as_before() {
    // A NAT-LOGIC goal with a function-like word is refused, as before.
    let src = "(rlt-goal (id n.l) (convention NAT-LOGIC) (goal \"∀x. double(x) = x + x\") (mode unconditional) (axioms ROBINSON-Q))";
    let r = prove_source(
        &root(),
        src,
        &[],
        Some(Path::new("target/definitions-tests/nl.rlt")),
    );
    assert!(matches!(r.verdict, Verdict::Error(_)), "{}", text(&r));
    // The pinned M3b definitions file is untouched.
    assert_eq!(
        sha("spec/definitions.rlt"),
        "122e37fc627b136023a1a5ffac0344162ed77ae95312248a9f6a76c2b639ecda"
    );
}

// ── Audit D1: the unfolding budget is cumulative and checked before allocation ──

/// The audit's reproduction: `aa(x) := x + x` … `ad(x) := ac(ac(x))`, and a
/// certificate whose goal sums 400 uses of `ad(ad(x))` (~52M nodes once
/// unfolded). Before the cumulative budget this goal string of ~10 KB was
/// killed by the OOM killer; now `rlt check` refuses it with an error.
#[test]
fn audit_d1_many_uses_are_refused_with_an_error_not_oom() {
    let chain = r#"(define-function aa (x) (+ x x) (written "aa(x) := x + x") (origin rlt-authored))
  (define-function ab (x) (aa (aa x)) (written "ab(x) := aa(aa(x))") (origin rlt-authored))
  (define-function ac (x) (ab (ab x)) (written "ac(x) := ab(ab(x))") (origin rlt-authored))
  (define-function ad (x) (ac (ac x)) (written "ad(x) := ac(ac(x))") (origin rlt-authored))"#;
    let defs = scratch("d1_chain.rlt", &registry("D1-CHAIN", chain));
    for (k, accepted_size) in [(1usize, true), (400, false)] {
        let goal = format!("∀x. {} = 0", vec!["ad(ad(x))"; k].join(" + "));
        let cert = scratch(
            &format!("d1_{k}.rlt"),
            &format!(
                "(rlt-proof (id D1) (convention NAT-FUN) (definitions \"{defs}\" \"{}\") (goal \"{goal}\") (mode unconditional) (axioms ROBINSON-Q) (steps (s1 (= 0 0) (refl 0))))\n",
                sha(&defs)
            ),
        );
        let started = std::time::Instant::now();
        let c = check(&cert);
        assert_ne!(c.report.code, ACCEPT);
        let t = &c.report.text;
        if accepted_size {
            // Unfolds; the trivial proof is then refused by the kernel for
            // proving 0 = 0 instead of the goal.
            assert!(t.contains("checked goal"), "{t}");
            assert!(!t.contains("would exceed"), "{t}");
        } else {
            assert!(t.contains("would exceed 1000000 nodes"), "{t}");
            assert!(
                started.elapsed() < std::time::Duration::from_secs(10),
                "refusal took {:?}",
                started.elapsed()
            );
        }
    }
}

// ── Audit D4: what the Lean comparison script, as opposed to Lean, checks ──

/// The script, not Lean's `Iff.rfl`, pins the Lean statements to RLT's
/// printed unfolding: `(double 2 = 4) ↔ (3 + 1 = 4)` holds by `Iff.rfl`
/// in Lean, yet is not what RLT unfolded, and the script refuses it. A Lean
/// `def` that differs from the registered written form is refused too.
#[test]
fn audit_d4_lean_comparison_script_checks_text_not_just_definitional_equality() {
    let rlt = env!("CARGO_BIN_EXE_rlt");
    let lean = read(&root().join("lean/ProverDefinitions.lean"));
    let run = |tag: &str, body: &str| {
        let rel = scratch(&format!("d4_{tag}.lean"), body);
        let out = std::process::Command::new("python3")
            .args([
                "-I",
                "scripts/check-definitions-lean.py",
                rlt,
                "--lean",
                &rel,
                "--layer",
                "text",
            ])
            .current_dir(root())
            .output()
            .expect("python3");
        (
            out.status.success(),
            String::from_utf8_lossy(&out.stdout).to_string(),
        )
    };
    let (ok, t) = run("same", &lean);
    assert!(ok, "{t}");
    assert!(
        t.contains("PASS for 7 goals") && t.contains("source layout PASS"),
        "{t}"
    );

    let f3 = "theorem F3_unfolds : (double 2 = 4) ↔ (2 + 2 = 4) := Iff.rfl";
    assert!(lean.contains(f3));
    let swapped = lean.replace(
        f3,
        "theorem F3_unfolds : (double 2 = 4) ↔ (3 + 1 = 4) := Iff.rfl",
    );
    let (ok, t) = run("defeq_but_not_rlt", &swapped);
    assert!(
        !ok,
        "a definitionally equal but different unfolding passed:\n{t}"
    );
    assert!(
        t.contains("theorem F3_unfolds states `(double 2 = 4) ↔ (3 + 1 = 4)`; RLT requires `(double 2 = 4) ↔ (2 + 2 = 4)`"),
        "{t}"
    );

    let d = "def double (x : Nat) : Nat := x + x";
    assert!(lean.contains(d));
    let (ok, t) = run(
        "other_def",
        &lean.replace(d, "def double (x : Nat) : Nat := 2 * x"),
    );
    assert!(!ok, "{t}");
    assert!(
        t.contains("as `def double (x : Nat) : Nat := x + x`"),
        "{t}"
    );
}

// ── Audit D5: the comparison is per declaration, comments excluded ──

/// The F3 attack from the final audit, and its variants. Each mutant
/// compiles in Lean (the attack itself is compiled by scripts/verify.sh
/// when Lean is present) or is outside the script's grammar; every one must
/// be refused, and the original file accepted.
#[test]
fn audit_d5_lean_comparison_refuses_hidden_or_misplaced_declarations() {
    let rlt = env!("CARGO_BIN_EXE_rlt");
    let lean = read(&root().join("lean/ProverDefinitions.lean"));
    let run = |tag: &str, body: &str| {
        let rel = scratch(&format!("d5_{tag}.lean"), body);
        let out = std::process::Command::new("python3")
            .args([
                "-I",
                "scripts/check-definitions-lean.py",
                rlt,
                "--lean",
                &rel,
                "--layer",
                "text",
            ])
            .current_dir(root())
            .output()
            .expect("python3");
        (
            out.status.success(),
            String::from_utf8_lossy(&out.stdout).to_string(),
        )
    };
    let f3 = "theorem F3_unfolds : (double 2 = 4) ↔ (2 + 2 = 4) := Iff.rfl";
    let wrong = "theorem F3_unfolds : (double 2 = 4) ↔ (3 + 1 = 4) := Iff.rfl";
    let correct_text = "(double 2 = 4) ↔ (2 + 2 = 4)";
    assert_eq!(lean.matches(f3).count(), 1);
    let end_ns = "end RLTProverDefinitions";
    assert_eq!(lean.matches(end_ns).count(), 1);
    let before_end = |extra: &str| lean.replace(end_ns, &format!("{extra}\n\n{end_ns}"));

    let (ok, t) = run("original", &lean);
    assert!(ok, "the original must pass:\n{t}");
    assert!(
        t.contains("PASS for 7 goals") && t.contains("source layout PASS"),
        "{t}"
    );

    let cases: Vec<(&str, String, &str)> = vec![
        (
            "block_comment_attack",
            lean.replace(f3, &format!("/- {f3} -/\n{wrong}")),
            "theorem F3_unfolds states `(double 2 = 4) ↔ (3 + 1 = 4)`",
        ),
        (
            "line_comment",
            lean.replace(f3, &format!("-- {f3}\n{wrong}")),
            "theorem F3_unfolds states",
        ),
        (
            "nested_block_comment",
            lean.replace(f3, &format!("/- outer /- {f3} -/ still a comment -/\n{wrong}")),
            "theorem F3_unfolds states",
        ),
        (
            "doc_comment",
            lean.replace(f3, &format!("/-- {f3} -/\n{wrong}")),
            "theorem F3_unfolds states",
        ),
        (
            "duplicate_declaration",
            lean.replace(f3, &format!("{f3}\n{wrong}")),
            "`F3_unfolds` is declared 2 times",
        ),
        (
            "duplicate_in_other_namespace",
            lean.replace(f3, wrong)
                .replace(end_ns, &format!("{end_ns}\nnamespace Decoy\n{f3}\nend Decoy")),
            "unsupported command after the namespace",
        ),
        (
            "mismatched_name",
            lean.replace(
                f3,
                &format!("theorem F3_unfold : {correct_text} := Iff.rfl\n{wrong}"),
            ),
            "unexpected theorem `F3_unfold`",
        ),
        (
            "text_in_string",
            before_end(&format!("def decoy : String := \"{f3}\"")).replace(f3, wrong),
            "a string literal outside comments",
        ),
        (
            "text_in_unrelated_declaration",
            before_end(&format!("theorem decoy : {correct_text} := Iff.rfl")).replace(f3, wrong),
            "unexpected theorem `decoy`",
        ),
        (
            "text_in_another_proof",
            lean.replace(f3, wrong).replace(
                "theorem F3_holds : double 2 = 4 := rfl",
                &format!("theorem F3_holds : double 2 = 4 := by\n  have _h : {correct_text} := Iff.rfl\n  rfl"),
            ),
            "theorem F3_unfolds states",
        ),
        (
            "instance_redefining_addition",
            before_end("instance : HAdd Nat Nat Nat := ⟨Nat.mul⟩"),
            "unsupported command `instance",
        ),
        (
            "unterminated_comment",
            lean.replace(f3, &format!("/- {f3}")),
            "unterminated block comment",
        ),
        (
            "different_def",
            lean.replace(
                "def double (x : Nat) : Nat := x + x",
                "def double (x : Nat) : Nat := 2 * x",
            ),
            "registered in definitions/double.rlt as `def double (x : Nat) : Nat := x + x`",
        ),
        (
            "def_hidden_in_comment",
            lean.replace(
                "def double (x : Nat) : Nat := x + x",
                "-- def double (x : Nat) : Nat := x + x\ndef double (x : Nat) : Nat := 2 * x",
            ),
            "registered in definitions/double.rlt",
        ),
    ];
    for (tag, body, want) in cases {
        assert_ne!(body, lean, "{tag}: the mutation did not apply");
        let (ok, t) = run(tag, &body);
        assert!(!ok, "{tag} passed the comparison script:\n{t}");
        assert!(t.contains(want), "{tag}: expected `{want}` in:\n{t}");
    }
}

// ── Audit D6: the verdict rests on Lean's elaborated declarations ──

/// The Lean binary `scripts/check-definitions-lean.py` would use, run from
/// lean/ so elan selects lean/lean-toolchain; None when Lean is absent.
fn lean_bin() -> Option<String> {
    let home = std::env::var("HOME").unwrap_or_default();
    let elan = format!("{home}/.elan/bin/lean");
    let cands = [
        std::env::var("RLT_LEAN").ok(),
        Some("lean".into()),
        Some(elan),
    ];
    cands.into_iter().flatten().find(|c| {
        std::process::Command::new(c)
            .arg("--version")
            .current_dir(root().join("lean"))
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    })
}

/// The release-audit bypass of `0987be5` and its relatives. Escaped
/// identifiers `«/-»` … `«-/»` made the source-text checker skip a compiled
/// wrong `F3_unfolds` and count a correct one that Lean read as an
/// identifier. The authoritative layer now compares the declarations Lean
/// elaborated with an expected module built from RLT's printed statements
/// and the registered definitions, so every mutant below is refused by the
/// Lean layer on its own (`--layer lean`) and by the full checker, while the
/// original passes. Lean must accept the mutants marked `compiles`, so the
/// refusal comes from the correspondence, not from Lean rejecting the file.
/// Without Lean this test fails unless RLT_ALLOW_NO_LEAN=1 (as verify.sh).
#[test]
fn audit_d6_lean_layer_compares_elaborated_declarations() {
    let rlt = env!("CARGO_BIN_EXE_rlt");
    let Some(lean_bin) = lean_bin() else {
        assert_eq!(
            std::env::var("RLT_ALLOW_NO_LEAN").as_deref(),
            Ok("1"),
            "Lean is unavailable: install elan, or set RLT_ALLOW_NO_LEAN=1 to skip this test explicitly"
        );
        eprintln!("SKIPPED audit_d6 (lean unavailable; RLT_ALLOW_NO_LEAN=1)");
        return;
    };
    let lean = read(&root().join("lean/ProverDefinitions.lean"));
    let check = |rel: &str, layer: &str| {
        let out = std::process::Command::new("python3")
            .args([
                "-I",
                "scripts/check-definitions-lean.py",
                rlt,
                "--lean",
                rel,
                "--layer",
                layer,
            ])
            .current_dir(root())
            .output()
            .expect("python3");
        (
            out.status.code(),
            String::from_utf8_lossy(&out.stdout).to_string(),
        )
    };
    let compiles = |rel: &str| {
        let out = std::process::Command::new(&lean_bin)
            .arg(root().join(rel))
            .current_dir(root().join("lean"))
            .output()
            .expect("lean");
        let msg = String::from_utf8_lossy(&out.stdout).to_lowercase()
            + &String::from_utf8_lossy(&out.stderr).to_lowercase();
        out.status.success() && !msg.contains("warning") && !msg.contains("error")
    };

    let f3 = "theorem F3_unfolds : (double 2 = 4) ↔ (2 + 2 = 4) := Iff.rfl";
    let wrong = "theorem F3_unfolds : (double 2 = 4) ↔ (3 + 1 = 4) := Iff.rfl";
    let f2h = "theorem F2_holds : double 0 = 0 := rfl";
    let f3h = "theorem F3_holds : double 2 = 4 := rfl";
    let d = "def double (x : Nat) : Nat := x + x";
    let ns = "namespace RLTProverDefinitions\n";
    let end_ns = "end RLTProverDefinitions";
    for anchor in [f3, f2h, f3h, d, ns, end_ns] {
        assert_eq!(lean.matches(anchor).count(), 1, "{anchor}");
    }

    let rel = scratch("d6_original.lean", &lean);
    assert!(compiles(&rel));
    for layer in ["lean", "both"] {
        let (code, t) = check(&rel, layer);
        assert_eq!(code, Some(0), "the original must pass ({layer}):\n{t}");
        assert!(
            t.contains("PASS for 7 goals") && t.contains("Lean declaration correspondence PASS"),
            "{t}"
        );
    }

    let f3_type = "theorem `RLTProverDefinitions.F3_unfolds` has elaborated type";
    // (tag, mutant, Lean must compile it, Lean-layer reason, text-layer reason)
    let cases: Vec<(&str, String, bool, &str, &str)> = vec![
        (
            "escaped_identifier_attack",
            lean.replace(
                f2h,
                "theorem F2_holds : double 0 = 0 := by have «/-» : True := trivial; rfl",
            )
            .replace(
                f3,
                &format!("{wrong}\nexample : True := by have «-/» : True := trivial; trivial"),
            )
            .replace(
                f3h,
                &format!(
                    "theorem F3_holds : double 2 = 4 := by have «{f3}» : True := trivial; rfl"
                ),
            ),
            true,
            f3_type,
            "an escaped identifier",
        ),
        (
            "provable_wrong_f3",
            lean.replace(f3, wrong),
            true,
            f3_type,
            "theorem F3_unfolds states",
        ),
        (
            "correct_f3_in_comment",
            lean.replace(f3, &format!("/- {f3} -/\n{wrong}")),
            true,
            f3_type,
            "theorem F3_unfolds states",
        ),
        (
            "missing_f3",
            lean.replace(f3, "")
                .replace("#print axioms RLTProverDefinitions.F3_unfolds\n", ""),
            true,
            "theorem `RLTProverDefinitions.F3_unfolds` is missing from the companion",
            "`F3_unfolds` is declared 0 times",
        ),
        (
            "duplicate_f3",
            lean.replace(f3, &format!("{f3}\n{f3}")),
            false,
            "does not elaborate cleanly",
            "`F3_unfolds` is declared 2 times",
        ),
        (
            "duplicate_in_other_namespace",
            lean.replace(f3, wrong).replace(
                end_ns,
                &format!(
                    "{end_ns}\nnamespace Decoy\n{}\nend Decoy",
                    f3.replace("double", "RLTProverDefinitions.double")
                ),
            ),
            true,
            "unexpected theorem `Decoy.F3_unfolds`",
            "unsupported command after the namespace",
        ),
        (
            "modified_double_still_compiles",
            lean.replace(d, "def double (x : Nat) : Nat := x + x + 0"),
            true,
            "def `RLTProverDefinitions.double` has value",
            "registered in definitions/double.rlt",
        ),
        (
            "macro_rewrites_the_statement",
            lean.replace(ns, &format!("{ns}macro_rules | `(2 + 2) => `(3 + 1)\n")),
            true,
            f3_type,
            "unsupported command `macro_rules",
        ),
        (
            "instance_redefining_addition",
            lean.replace(
                ns,
                &format!("{ns}instance (priority := high) myAdd : HAdd Nat Nat Nat := ⟨Nat.add⟩\n"),
            ),
            true,
            "unexpected def `RLTProverDefinitions.myAdd`",
            "unsupported command `instance",
        ),
        (
            "extra_axiom",
            lean.replace(ns, &format!("{ns}axiom cheat : 2 + 2 = 4\n")),
            true,
            "unexpected axiom `RLTProverDefinitions.cheat`",
            "unsupported command `axiom",
        ),
        (
            "hand_written_internal_name",
            lean.replace(
                end_ns,
                &format!(
                    "theorem F3_holds._proof_9 : (double 2 = 4) ↔ (3 + 1 = 4) := Iff.rfl\n{end_ns}"
                ),
            ),
            true,
            "unexpected theorem `RLTProverDefinitions.F3_holds._proof_9`",
            "unsupported command",
        ),
        (
            "import_beyond_init",
            format!("import Lean\n{lean}"),
            true,
            "the companion imports Lean",
            "the file must begin with `namespace RLTProverDefinitions`",
        ),
        (
            "sorry_proof",
            lean.replace(f3h, "theorem F3_holds : double 2 = 4 := sorry"),
            false,
            "does not elaborate cleanly",
            "",
        ),
    ];
    for (tag, body, must_compile, lean_why, text_why) in cases {
        assert_ne!(body, lean, "{tag}: the mutation did not apply");
        let rel = scratch(&format!("d6_{tag}.lean"), &body);
        if must_compile {
            assert!(compiles(&rel), "{tag}: Lean must accept the mutant");
        }
        let (code, t) = check(&rel, "lean");
        assert_eq!(code, Some(1), "{tag} passed the Lean layer:\n{t}");
        assert!(
            t.contains(lean_why),
            "{tag}: expected `{lean_why}` from the Lean layer in:\n{t}"
        );
        let (code, t) = check(&rel, "both");
        assert_eq!(code, Some(1), "{tag} passed the full checker:\n{t}");
        if !text_why.is_empty() {
            let (code, t) = check(&rel, "text");
            assert_eq!(code, Some(1), "{tag} passed the layout layer:\n{t}");
            assert!(
                t.contains(text_why),
                "{tag}: expected `{text_why}` from the layout layer in:\n{t}"
            );
        }
    }
}

// ── Audit H1: kernel replay of the compiled companion ──

/// `set_option debug.skipKernelTC true` lets Lean store declarations its
/// kernel never checked. Two mutants, each compiled cleanly by Lean:
///
/// - `skip_valid`: the option on a valid theorem (core Lean only). The
///   complete verification refuses it (the source-layout layer rejects
///   `set_option`); the Lean layer replays it with `leanchecker`, finds the
///   proof valid, and its declaration still matches, so it passes there.
/// - `skip_invalid`: a metaprogram adds `F3_holds : double 2 = 4` with the
///   proof `True.intro` under the option. Its type matches the expected
///   proposition, so only the kernel can tell; the Lean layer refuses it at
///   the kernel replay (before the declaration comparison, whose Init-only
///   import rule would otherwise be the sole defence), and so does the
///   complete verification.
///
/// Without Lean this test fails unless RLT_ALLOW_NO_LEAN=1 (as verify.sh).
#[test]
fn audit_h1_kernel_replay_refuses_unchecked_declarations() {
    let rlt = env!("CARGO_BIN_EXE_rlt");
    let Some(lean_bin) = lean_bin() else {
        assert_eq!(
            std::env::var("RLT_ALLOW_NO_LEAN").as_deref(),
            Ok("1"),
            "Lean is unavailable: install elan, or set RLT_ALLOW_NO_LEAN=1 to skip this test explicitly"
        );
        eprintln!("SKIPPED audit_h1 (lean unavailable; RLT_ALLOW_NO_LEAN=1)");
        return;
    };
    let lean = read(&root().join("lean/ProverDefinitions.lean"));
    let check = |rel: &str, layer: &str| {
        let out = std::process::Command::new("python3")
            .args([
                "-I",
                "scripts/check-definitions-lean.py",
                rlt,
                "--lean",
                rel,
                "--layer",
                layer,
            ])
            .current_dir(root())
            .output()
            .expect("python3");
        (
            out.status.code(),
            String::from_utf8_lossy(&out.stdout).to_string(),
        )
    };
    let compiles = |rel: &str| {
        let out = std::process::Command::new(&lean_bin)
            .arg(root().join(rel))
            .current_dir(root().join("lean"))
            .output()
            .expect("lean");
        let msg = String::from_utf8_lossy(&out.stdout).to_lowercase()
            + &String::from_utf8_lossy(&out.stderr).to_lowercase();
        out.status.success() && !msg.contains("warning") && !msg.contains("error")
    };
    let f3h = "theorem F3_holds : double 2 = 4 := rfl";
    assert_eq!(lean.matches(f3h).count(), 1);

    let rel = scratch(
        "h1_skip_valid.lean",
        &lean.replace(
            f3h,
            &format!("set_option debug.skipKernelTC true in\n{f3h}"),
        ),
    );
    assert!(compiles(&rel), "skip_valid: Lean must accept the mutant");
    let (code, t) = check(&rel, "both");
    assert_eq!(
        code,
        Some(1),
        "skip_valid passed the complete verification:\n{t}"
    );
    assert!(
        t.contains("unsupported command `set_option debug.skipKernelTC"),
        "{t}"
    );
    let (code, t) = check(&rel, "lean");
    assert_eq!(
        code,
        Some(0),
        "skip_valid: a kernel-valid replayed companion matches:\n{t}"
    );

    let inject = r#"open Lean Elab Command in
set_option debug.skipKernelTC true in
run_cmd liftTermElabM do
  let ty ← Term.elabType (← `(double 2 = 4))
  Term.synthesizeSyntheticMVarsNoPostponing
  let ty ← instantiateMVars ty
  addDecl (.thmDecl { name := `RLTProverDefinitions.F3_holds, levelParams := [], type := ty, value := mkConst ``True.intro })"#;
    let rel = scratch(
        "h1_skip_invalid.lean",
        &format!("import Lean\n{}", lean.replace(f3h, inject)),
    );
    assert!(compiles(&rel), "skip_invalid: Lean must accept the mutant");
    for layer in ["lean", "both"] {
        let (code, t) = check(&rel, layer);
        assert_eq!(code, Some(1), "skip_invalid passed ({layer}):\n{t}");
        assert!(
            t.contains("kernel replay (leanchecker) refused the compiled companion")
                && t.contains("declaration type mismatch, 'RLTProverDefinitions.F3_holds'"),
            "skip_invalid ({layer}): expected the kernel replay refusal in:\n{t}"
        );
    }
}

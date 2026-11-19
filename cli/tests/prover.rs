//! RLT prover milestone: bounded proof search whose every result is replayed
//! by the existing kernel. Phase 3 goals (proved from statements alone),
//! not-found goals, theorem discovery, and the Phase 5 adversarial checks.

use rlt_cli::prove::{certify, parse_goal_file, prove_source, render, Verdict};
use rlt_cli::{check_file, check_file_outcome, ACCEPT, REJECT};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    let manifest = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_string());
    Path::new(&manifest).parent().unwrap().to_path_buf()
}

fn goals(dir: &str) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(root().join(dir))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "goal"))
        .collect();
    v.sort();
    v
}

fn scratch(name: &str) -> String {
    format!("target/prover-tests/{name}")
}

fn read(p: &Path) -> String {
    std::fs::read_to_string(p).unwrap()
}

/// Run a goal file's search, writing its certificate to a scratch path.
fn run(goal: &Path, tag: &str) -> (rlt_cli::prove::ProveOutcome, String) {
    let out = scratch(&format!("{tag}.rlt"));
    let r = prove_source(&root(), &read(goal), &[], Some(Path::new(&out)));
    (r, out)
}

// ── Phase 3: statements in, kernel-checked theorems out ────────────────────

#[test]
fn phase3_goals_are_proved_and_independently_replayed() {
    let gs = goals("tests/prover/goals");
    assert_eq!(gs.len(), 10);
    for g in &gs {
        let tag = g.file_stem().unwrap().to_string_lossy().to_string();
        let src = read(g);
        assert!(!src.contains("(steps"), "goal files supply no proof");
        let (r, out) = run(g, &tag);
        let Verdict::Proved { theorem, .. } = &r.verdict else {
            let mut t = String::new();
            render(&mut t, &r);
            panic!("{tag}: {t}");
        };
        // A separate `rlt check` of the certificate on disk accepts it.
        let c = check_file_outcome(&root(), &root().join(&out));
        assert_eq!(c.report.code, ACCEPT, "{tag}:\n{}", c.report.text);
        // The replayed goal is exactly the elaborated goal of the goal file.
        assert_eq!(
            c.prop.as_ref().unwrap().to_string(),
            r.goal_shown.clone().unwrap(),
            "{tag}"
        );
        // Γ only ever holds declared assumptions; unconditional goals have Γ = ∅.
        let declared = parse_goal_file(&src).unwrap();
        let names: Vec<String> = declared
            .fields
            .iter()
            .filter_map(|f| match f.head() {
                Some(("assume", [rlt_syntax::sexpr::Sexp::Sym(n), _])) => Some(n.clone()),
                _ => None,
            })
            .collect();
        for h in theorem.assumptions().keys() {
            assert!(names.contains(h), "{tag}: undeclared assumption {h}");
        }
        if src.contains("(mode unconditional)") {
            assert!(theorem.is_unconditional(), "{tag}");
        }
    }
}

#[test]
fn the_four_requested_goals_have_the_requested_shape() {
    let cases = [
        ("P1_symmetry", "∀a. ∀b. (a = b → b = a)", &[][..]),
        (
            "P2_transitivity",
            "∀a. ∀b. ∀c. ((a = b ∧ b = c) → a = c)",
            &[][..],
        ),
        (
            "P3_congruence",
            "∀a. ∀b. ∀c. (a + b = 0 → (c = a → c + b = 0))",
            &[][..],
        ),
        (
            "P4_two_times_three",
            "(2 × 3) + 1 = 7",
            &["add_succ", "add_zero", "mul_succ", "mul_zero"][..],
        ),
    ];
    for (name, shown, axioms) in cases {
        let (r, _) = run(
            &root().join(format!("tests/prover/goals/{name}.goal")),
            &format!("shape-{name}"),
        );
        assert_eq!(r.goal_shown.as_deref(), Some(shown));
        let Verdict::Proved { theorem, .. } = &r.verdict else {
            panic!("{name}")
        };
        let used: Vec<&str> = theorem.axioms_used().keys().map(|s| s.as_str()).collect();
        assert_eq!(
            used, axioms,
            "{name}: equality goals use no arithmetic axiom"
        );
    }
}

#[test]
fn committed_certificates_are_reproduced_byte_for_byte_and_accepted() {
    for g in goals("tests/prover/goals") {
        let stem = g.file_stem().unwrap().to_string_lossy().to_string();
        let committed = root().join(format!("tests/prover/certificates/{stem}.rlt"));
        let rep = check_file(&root(), &committed);
        assert_eq!(rep.code, ACCEPT, "{stem}:\n{}", rep.text);
        let (r, out) = run(&g, &format!("repro-{stem}"));
        assert_eq!(r.code(), 0);
        assert_eq!(
            read(&root().join(out)),
            read(&committed),
            "{stem}: the search is deterministic; regenerate with `rlt prove … --out`"
        );
    }
}

#[test]
fn conditional_goals_report_exactly_the_declared_assumptions_used() {
    let (r, _) = run(&root().join("tests/prover/goals/P5_conditional.goal"), "p5");
    let Verdict::Proved { theorem, .. } = &r.verdict else {
        panic!()
    };
    let g: Vec<&str> = theorem.assumptions().keys().map(|s| s.as_str()).collect();
    assert_eq!(g, ["h1", "h2"]);
    assert!(theorem.axioms_used().is_empty());
}

// ── Search exhaustion is never a disproof ──────────────────────────────────

#[test]
fn not_found_goals_are_reported_without_a_truth_claim() {
    let gs = goals("tests/prover/notfound");
    assert_eq!(gs.len(), 3);
    for g in &gs {
        let tag = g.file_stem().unwrap().to_string_lossy().to_string();
        let (r, out) = run(g, &tag);
        assert!(matches!(r.verdict, Verdict::NotFound), "{tag}");
        assert_eq!(r.code(), 1);
        assert_eq!(r.label(), "NOT_FOUND_WITHIN_BOUNDS");
        assert!(
            !root().join(out).exists(),
            "{tag}: no certificate is written"
        );
        let mut t = String::new();
        render(&mut t, &r);
        assert!(t.contains("says nothing about whether the goal is true, false or independent"));
        for word in ["FALSE", "DISPROVED", "REFUTED", "PROVED —"] {
            assert!(!t.contains(word), "{tag}: {t}");
        }
    }
}

#[test]
fn a_false_equation_is_not_found_and_its_negation_is_proved() {
    // The only route to "false" is a kernel-checked proof of the negation.
    let (n, _) = run(
        &root().join("tests/prover/notfound/N2_false_equation.goal"),
        "n2",
    );
    assert!(matches!(n.verdict, Verdict::NotFound));
    let (p, _) = run(
        &root().join("tests/prover/goals/P6_not_two_plus_two_five.goal"),
        "p6",
    );
    assert!(matches!(p.verdict, Verdict::Proved { .. }));
}

#[test]
fn search_bounds_are_enforced_and_reported() {
    let src = read(&root().join("tests/prover/goals/P8_zero_or_successor.goal"));
    // Too little depth: the case split and the witness cannot both happen.
    let r = prove_source(&root(), &src, &[("depth".into(), 1)], None);
    assert!(matches!(r.verdict, Verdict::NotFound));
    // A tiny call budget is exhausted and reported as such.
    let r = prove_source(&root(), &src, &[("budget".into(), 3)], None);
    assert!(matches!(r.verdict, Verdict::NotFound));
    assert!(r.stats.budget_exhausted);
    let mut t = String::new();
    render(&mut t, &r);
    assert!(t.contains("call budget exhausted"), "{t}");
    // Bounds above their ceilings are refused, not clamped.
    let r = prove_source(&root(), &src, &[("depth".into(), 99)], None);
    assert!(matches!(&r.verdict, Verdict::Error(e) if e.contains("ceiling")));
}

// ── Phase 5: adversarial verification ──────────────────────────────────────

fn p4() -> (String, rlt_cli::prove::GoalFile, String) {
    let src = read(&root().join("tests/prover/goals/P4_two_times_three.goal"));
    let g = parse_goal_file(&src).unwrap();
    let cert = read(&root().join("tests/prover/certificates/P4_two_times_three.rlt"));
    (src, g, cert)
}

fn certify_text(g: &rlt_cli::prove::GoalFile, name: &str, text: &str) -> Result<(), String> {
    let p = root().join(scratch(name));
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(&p, text).unwrap();
    certify(&root(), g, &p, text).map(|_| ())
}

#[test]
fn an_invalid_inference_cannot_be_certified() {
    let (_, g, cert) = p4();
    assert!(certify_text(&g, "ok.rlt", &cert).is_ok());
    // Change one step's claim: 2 × 0 = 0 becomes 2 × 0 = 1.
    let bad = cert.replacen("(= (* 2 0) 0)", "(= (* 2 0) 1)", 1);
    assert_ne!(bad, cert);
    let e = certify_text(&g, "bad-claim.rlt", &bad).unwrap_err();
    assert!(e.contains("did not accept"), "{e}");
    // Use an axiom at the wrong instance.
    let bad = cert.replacen("(inst mul_zero (x 2))", "(inst mul_zero (x 3))", 1);
    assert_ne!(bad, cert);
    assert!(certify_text(&g, "bad-inst.rlt", &bad).is_err());
    // Drop the last step: the proof no longer ends at the goal.
    let mut lines: Vec<&str> = cert.lines().collect();
    let last = lines.len() - 1;
    lines.remove(last);
    let truncated = lines.join("\n") + "))\n";
    assert!(certify_text(&g, "truncated.rlt", &truncated).is_err());
}

#[test]
fn assumptions_cannot_be_introduced_silently() {
    let (_, g, cert) = p4();
    // A certificate that declares an extra assumption and uses it.
    let sneaky = cert.replacen(
        "(mode unconditional)",
        "(mode conditional)\n  (assume h \"2 × 3 + 1 = 7\")",
        1,
    );
    let steps_at = sneaky.find("  (steps").unwrap();
    let sneaky = format!(
        "{}  (steps\n    (s1 (= (+ (* 2 3) 1) 7) (hyp h))))\n",
        &sneaky[..steps_at]
    );
    let e = certify_text(&g, "sneaky.rlt", &sneaky).unwrap_err();
    assert!(e.contains("header differs"), "{e}");
    // The kernel alone also refuses it under the goal's own header: a hyp
    // step names an undeclared assumption.
    let steps_at = cert.find("  (steps").unwrap();
    let undeclared = format!(
        "{}  (steps\n    (s1 (= (+ (* 2 3) 1) 7) (hyp h))))\n",
        &cert[..steps_at]
    );
    let e = certify_text(&g, "undeclared.rlt", &undeclared).unwrap_err();
    assert!(e.contains("did not accept"), "{e}");
    // An unconditional goal is searched with Γ = ∅: the declared assumption
    // that would prove it is unavailable, so nothing is found.
    let (r, _) = run(
        &root().join("tests/prover/notfound/N3_unconditional_needs_assumption.goal"),
        "n3",
    );
    assert!(matches!(r.verdict, Verdict::NotFound));
}

#[test]
fn unregistered_axioms_are_refused() {
    let (src, g, cert) = p4();
    // A goal naming an unpinned axiom set is an error before any search.
    let bad = src.replace("(axioms ROBINSON-Q)", "(axioms MY-AXIOMS)");
    let r = prove_source(&root(), &bad, &[], None);
    assert!(
        matches!(&r.verdict, Verdict::Error(e) if e.contains("not pinned")),
        "{}",
        r.label()
    );
    // A certificate step citing an axiom outside the registered set.
    let forged = cert.replacen("(inst mul_zero (x 2))", "(inst mul_two (x 2))", 1);
    let p = root().join(scratch("forged-axiom.rlt"));
    std::fs::write(&p, &forged).unwrap();
    let rep = check_file(&root(), &p);
    assert_eq!(rep.code, REJECT, "{}", rep.text);
    assert!(rep.text.contains("unregistered"), "{}", rep.text);
    assert!(certify_text(&g, "forged-axiom.rlt", &forged).is_err());
}

#[test]
fn kernel_replay_cannot_be_bypassed() {
    let (_, g, cert) = p4();
    // A certificate for a different goal, even a true one, is refused.
    let other = cert.replacen("(goal \"2 × 3 + 1 = 7\")", "(goal \"7 = 7\")", 1);
    assert!(certify_text(&g, "other-goal.rlt", &other).is_err());
    // An empty proof is refused by the kernel.
    let steps_at = cert.find("  (steps").unwrap();
    let empty = format!("{})\n", &cert[..steps_at]);
    assert!(certify_text(&g, "empty.rlt", &empty).is_err());
    // A certificate citing another file is refused (the prover never cites).
    let cites = cert.replacen(
        "(axioms ROBINSON-Q)",
        "(axioms ROBINSON-Q)\n  (cites (c \"tests/accept/M1_thirty.rlt\" \"0000000000000000000000000000000000000000000000000000000000000000\"))",
        1,
    );
    assert!(certify_text(&g, "cites.rlt", &cites).is_err());
    // PROVED is only ever built from an accepted replay: the outcome carries
    // the kernel's theorem and the replay report of `rlt check`.
    let (r, _) = run(
        &root().join("tests/prover/goals/P4_two_times_three.goal"),
        "p4-gate",
    );
    let Verdict::Proved { replay_report, .. } = &r.verdict else {
        panic!()
    };
    assert!(
        replay_report.contains("verdict         ACCEPT"),
        "{replay_report}"
    );
}

#[test]
fn goal_files_must_not_carry_proofs_or_unknown_fields() {
    let (src, _, _) = p4();
    for extra in [
        "(steps (s1 (= 7 7) (refl 7)))",
        "(params a)",
        "(trust-me yes)",
    ] {
        let bad = src.replacen(
            "(axioms ROBINSON-Q)",
            &format!("(axioms ROBINSON-Q) {extra}"),
            1,
        );
        let r = prove_source(&root(), &bad, &[], None);
        assert!(matches!(r.verdict, Verdict::Error(_)), "{extra}");
    }
}

#[test]
fn arithmetic_claims_stay_within_robinson_q() {
    // ∀x. 0 + x = x and ∀x. x × 1 = x hold in ℕ but need induction; the
    // prover finds neither, while it does find ∀x. x × 1 = 0 + x, which Q proves.
    let mk = |goal: &str| {
        format!(
            "(rlt-goal (id Q.scope) (convention NAT-LOGIC) (goal \"{goal}\") (mode unconditional) (axioms ROBINSON-Q) (bounds (depth 2) (budget 3000)))"
        )
    };
    for g in ["∀x. 0 + x = x", "∀x. x × 1 = x", "∀x. ∀y. x + y = y + x"] {
        let r = prove_source(&root(), &mk(g), &[], None);
        assert!(matches!(r.verdict, Verdict::NotFound), "{g}");
    }
    let r = prove_source(
        &root(),
        &mk("∀x. x × 1 = 0 + x"),
        &[],
        Some(Path::new(&scratch("q1.rlt"))),
    );
    assert!(matches!(r.verdict, Verdict::Proved { .. }));
}

// ── Phase 4: theorem discovery ─────────────────────────────────────────────

#[test]
fn committed_discoveries_verify() {
    let (t, code) = rlt_cli::discover::verify(&root(), "research/prover-discovery");
    assert_eq!(code, 0, "{t}");
    assert!(
        t.contains("verified        9 accepted, 0 refused or stale"),
        "{t}"
    );
}

#[test]
fn discovery_stores_only_kernel_verified_theorems_and_detects_tampering() {
    let dir = scratch("discovery");
    let _ = std::fs::remove_dir_all(root().join(&dir));
    let (summary, code) = rlt_cli::discover::discover(&root(), &dir, false, None);
    assert_eq!(code, 0, "{summary}");
    assert!(summary.contains("proved          9"), "{summary}");
    let committed = read(&root().join("research/prover-discovery/manifest.tsv"));
    let fresh = read(&root().join(&dir).join("manifest.tsv"));
    // Same theorems, same certificate hashes (paths differ by directory).
    let rows = |s: &str| -> Vec<(String, String)> {
        s.lines()
            .filter(|l| !l.starts_with('#'))
            .map(|l| {
                let c: Vec<&str> = l.split('\t').collect();
                (c[1].to_string(), c[6].to_string())
            })
            .collect()
    };
    assert_eq!(rows(&committed), rows(&fresh));
    // Every stored file is a certificate the kernel accepts; nothing else is stored.
    let stored: Vec<PathBuf> = std::fs::read_dir(root().join(&dir))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "rlt"))
        .collect();
    assert_eq!(stored.len(), 9);
    for p in &stored {
        assert_eq!(check_file(&root(), p).code, ACCEPT);
    }
    // Tampering with a stored certificate is detected by --verify.
    let victim = root().join(&dir).join("D023.rlt");
    let text = read(&victim).replace("(inst add_zero (x p1))", "(inst add_zero (x p1)) ");
    std::fs::write(&victim, text).unwrap();
    let (t, code) = rlt_cli::discover::verify(&root(), &dir);
    assert_eq!(code, 1);
    assert!(t.contains("STALE"), "{t}");
}

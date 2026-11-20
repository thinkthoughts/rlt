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

// ── RLT-PROVER-001 audit D5: every manifest column is bound to the replay ──

/// A copy of the committed discovery directory under target/, with the
/// manifest's certificate paths pointing into the copy.
fn discovery_copy(name: &str) -> String {
    let dir = scratch(&format!("d5-{name}"));
    let full = root().join(&dir);
    let _ = std::fs::remove_dir_all(&full);
    std::fs::create_dir_all(&full).unwrap();
    let src = root().join("research/prover-discovery");
    for e in std::fs::read_dir(&src).unwrap() {
        let p = e.unwrap().path();
        std::fs::copy(&p, full.join(p.file_name().unwrap())).unwrap();
    }
    let m =
        read(&full.join("manifest.tsv")).replace("research/prover-discovery/", &format!("{dir}/"));
    std::fs::write(full.join("manifest.tsv"), m).unwrap();
    let sm = read(&full.join("summary.txt")).replace("research/prover-discovery", &dir);
    std::fs::write(full.join("summary.txt"), sm).unwrap();
    dir
}

/// Rewrite the manifest row with the given id through `f` (columns in, columns out).
fn edit_row(dir: &str, id: &str, f: impl Fn(&mut Vec<String>)) {
    let p = root().join(dir).join("manifest.tsv");
    let out: Vec<String> = read(&p)
        .lines()
        .map(|l| {
            let mut c: Vec<String> = l.split('\t').map(str::to_string).collect();
            if c[0] == id {
                f(&mut c);
            }
            c.join("\t")
        })
        .collect();
    std::fs::write(&p, out.join("\n") + "\n").unwrap();
}

fn refused(dir: &str, id: &str, reason: &str) {
    let (t, code) = rlt_cli::discover::verify(&root(), dir);
    assert_eq!(code, 1, "{t}");
    let line = t
        .lines()
        .find(|l| l.starts_with(&format!("{id}  REFUSED")))
        .unwrap_or_else(|| panic!("{id} not refused:\n{t}"));
    assert!(line.contains(reason), "expected `{reason}` in: {line}");
    assert!(
        t.contains("verified        8 accepted, ") || reason.contains("duplicate"),
        "{t}"
    );
}

#[test]
fn d5_the_unmodified_copy_verifies() {
    let dir = discovery_copy("baseline");
    let (t, code) = rlt_cli::discover::verify(&root(), &dir);
    assert_eq!(code, 0, "{t}");
    assert!(
        t.contains("verified        9 accepted, 0 refused or stale"),
        "{t}"
    );
}

#[test]
fn d5_a_conditional_proof_cannot_be_presented_as_unconditional() {
    // The audit's attack: a certificate that assumes its own statement,
    // replayed and accepted by the kernel, listed as unconditional with Γ = ∅.
    let dir = discovery_copy("conditional");
    let goal = "(rlt-goal (id D243) (convention NAT-LOGIC) (goal \"∀x. x × 1 = x\") (mode conditional) (assume h \"∀x. x × 1 = x\") (axioms ROBINSON-Q))";
    let cert = format!("{dir}/D243.rlt");
    let r = prove_source(&root(), goal, &[], Some(Path::new(&cert)));
    let Verdict::Proved { sha256, .. } = r.verdict else {
        panic!("the conditional certificate should replay");
    };
    assert_eq!(check_file(&root(), &root().join(&cert)).code, ACCEPT);
    edit_row(&dir, "D243", |c| {
        c[1] = "∀x. x × 1 = x".into();
        c[6] = sha256.clone();
    });
    refused(
        &dir,
        "D243",
        "mode: manifest says `unconditional`, certificate has `conditional`",
    );
    refused(
        &dir,
        "D243",
        "assumptions: manifest says `∅`, certificate has `h`",
    );
    refused(&dir, "D243", "the replayed theorem depends on assumptions");
}

#[test]
fn d5_a_changed_axiom_set_label_is_refused() {
    let dir = discovery_copy("axiom-set");
    edit_row(&dir, "D023", |c| c[2] = "PEANO-ADD".into());
    refused(
        &dir,
        "D023",
        "axiom set: manifest says `PEANO-ADD`, certificate has `ROBINSON-Q`",
    );
}

#[test]
fn d5_a_fabricated_step_count_is_refused() {
    let dir = discovery_copy("steps");
    edit_row(&dir, "D023", |c| c[7] = "999".into());
    refused(
        &dir,
        "D023",
        "steps: manifest says `999`, certificate has `2`",
    );
}

#[test]
fn d5_incorrect_axiom_dependencies_are_refused() {
    let dir = discovery_copy("axioms-used");
    edit_row(&dir, "D214", |c| c[9] = "none".into());
    refused(
        &dir,
        "D214",
        "axioms used: manifest says `none`, certificate has `add_zero,mul_zero`",
    );
    let dir = discovery_copy("axioms-used-extra");
    edit_row(&dir, "D243", |c| c[9] = "mul_succ,mul_zero,add_zero".into());
    refused(&dir, "D243", "axioms used");
}

#[test]
fn d5_a_duplicate_manifest_row_is_refused() {
    let dir = discovery_copy("duplicate");
    let p = root().join(&dir).join("manifest.tsv");
    let m = read(&p);
    let row = m
        .lines()
        .find(|l| l.starts_with("D023\t"))
        .unwrap()
        .to_string();
    std::fs::write(&p, format!("{m}{row}\n")).unwrap();
    let (t, code) = rlt_cli::discover::verify(&root(), &dir);
    assert_eq!(code, 1, "{t}");
    assert!(
        t.contains("D023  REFUSED ∀x. x + 0 = x — duplicate id, statement, certificate"),
        "{t}"
    );
    assert!(t.contains("verified        9 accepted, "), "{t}");
}

#[test]
fn d5_certificate_paths_cannot_escape_the_discovery_directory() {
    // A certificate elsewhere in the repository that the kernel accepts.
    let other = "tests/prover/certificates/P10_plus_two.rlt";
    let sha = rlt_cli::sha256_hex(&std::fs::read(root().join(other)).unwrap());
    for path in [
        other.to_string(),
        format!("{}/../../../{other}", scratch("d5-escape")),
    ] {
        let dir = discovery_copy("escape");
        edit_row(&dir, "D023", |c| {
            c[5] = path.clone();
            c[6] = sha.clone();
        });
        refused(
            &dir,
            "D023",
            "must be `target/prover-tests/d5-escape/D023.rlt`",
        );
    }
    // A certificate in the directory under another row's name.
    let dir = discovery_copy("misnamed");
    edit_row(&dir, "D023", |c| c[5] = format!("{dir}/D206.rlt"));
    let (t, code) = rlt_cli::discover::verify(&root(), &dir);
    assert_eq!(code, 1, "{t}");
    assert!(t.contains("D023  REFUSED"), "{t}");
}

#[test]
fn d5_inconsistent_metadata_is_refused_although_the_kernel_accepts() {
    // Each certificate still passes plain `rlt check`; only the row is wrong.
    for (id, col, val, reason) in [
        ("D023", 0usize, "D999", "is not a discovery id"),
        ("D023", 0usize, "D024", "certificate path"),
        ("D023", 1, "∀x. x + 0 = 0 + x", "statement: manifest says"),
        (
            "D023",
            3,
            "conditional",
            "mode: manifest says `conditional`",
        ),
        ("D023", 4, "h", "assumptions: manifest says `h`"),
        ("D023", 8, "7", "depth `7`"),
        ("D023", 8, "zero", "depth `zero`"),
    ] {
        let dir =
            discovery_copy(&format!("meta-{col}-{val}").replace(['∀', ' ', '.', '+', '='], ""));
        edit_row(&dir, id, |c| c[col] = val.into());
        let (t, code) = rlt_cli::discover::verify(&root(), &dir);
        assert_eq!(code, 1, "{t}");
        assert!(
            t.lines()
                .any(|l| l.contains("REFUSED") && l.contains(reason)),
            "expected `{reason}`:\n{t}"
        );
        assert_eq!(
            check_file(&root(), &root().join(&dir).join(format!("{id}.rlt"))).code,
            ACCEPT
        );
    }
    // A row with a column missing or added.
    let dir = discovery_copy("columns");
    edit_row(&dir, "D023", |c| {
        c.pop();
    });
    refused(&dir, "D023", "expected 10 tab-separated columns, found 9");
    // The column header itself.
    let dir = discovery_copy("header");
    let p = root().join(&dir).join("manifest.tsv");
    let m: String = read(&p)
        .lines()
        .filter(|l| !l.starts_with("# id"))
        .map(|l| format!("{l}\n"))
        .collect();
    std::fs::write(&p, m).unwrap();
    let (t, code) = rlt_cli::discover::verify(&root(), &dir);
    assert_eq!(code, 1, "{t}");
    assert!(t.contains("column header"), "{t}");
}

// ── Audit K9: the bytes hashed are the bytes replayed ──────────────────────

#[test]
fn k9_the_reported_hash_is_of_the_replayed_certificate_bytes() {
    for g in goals("tests/prover/goals") {
        let tag = format!("k9-{}", g.file_stem().unwrap().to_string_lossy());
        let (r, out) = run(&g, &tag);
        let Verdict::Proved { sha256, .. } = &r.verdict else {
            panic!("{tag}");
        };
        let bytes = std::fs::read(root().join(&out)).unwrap();
        assert_eq!(*sha256, rlt_cli::sha256_hex(&bytes), "{tag}");
        assert!(std::str::from_utf8(&bytes).is_ok(), "{tag}");
        assert_eq!(
            rlt_cli::check_bytes_outcome(&root(), &root().join(&out), bytes)
                .report
                .code,
            ACCEPT,
            "{tag}"
        );
    }
}

// ── Audit K8: outputs never overwrite files the tool did not write ─────────

#[test]
fn k8_prove_refuses_to_overwrite_a_file_it_did_not_write() {
    let dir = scratch("k8");
    let full = root().join(&dir);
    let _ = std::fs::remove_dir_all(&full);
    std::fs::create_dir_all(&full).unwrap();
    // A stand-in for a tracked source file (the audit used spec/robinson-q.rlt).
    let victim = full.join("axioms.rlt");
    let original =
        "; Registered axioms (stand-in)\n(axiom add_zero (x) (= (+ x 0) x) (source S))\n";
    std::fs::write(&victim, original).unwrap();
    let g = root().join("tests/prover/goals/P1_symmetry.goal");
    let r = prove_source(
        &root(),
        &read(&g),
        &[],
        Some(Path::new(&format!("{dir}/axioms.rlt"))),
    );
    let Verdict::Error(e) = &r.verdict else {
        panic!("expected ERROR");
    };
    assert!(e.contains("was not written by this command"), "{e}");
    assert_eq!(read(&victim), original, "the file is untouched");
    // A certificate the prover wrote earlier may be replaced (reruns stay possible).
    let out = format!("{dir}/P1.rlt");
    for _ in 0..2 {
        let r = prove_source(&root(), &read(&g), &[], Some(Path::new(&out)));
        assert!(matches!(r.verdict, Verdict::Proved { .. }));
    }
    // A symlink is never followed.
    #[cfg(unix)]
    {
        let link = full.join("link.rlt");
        std::os::unix::fs::symlink(&victim, &link).unwrap();
        let r = prove_source(
            &root(),
            &read(&g),
            &[],
            Some(Path::new(&format!("{dir}/link.rlt"))),
        );
        assert!(matches!(r.verdict, Verdict::Error(_)));
        assert_eq!(read(&victim), original);
    }
    // The committed pinned axiom file is refused outright, before any write.
    let pinned = root().join("spec/robinson-q.rlt");
    let before = std::fs::read(&pinned).unwrap();
    let r = prove_source(
        &root(),
        &read(&g),
        &[],
        Some(Path::new("spec/robinson-q.rlt")),
    );
    assert!(matches!(r.verdict, Verdict::Error(_)));
    assert_eq!(std::fs::read(&pinned).unwrap(), before);
}

#[test]
fn k8_discover_refuses_a_directory_holding_other_files_of_those_names() {
    let dir = scratch("k8-discover");
    let full = root().join(&dir);
    let _ = std::fs::remove_dir_all(&full);
    std::fs::create_dir_all(&full).unwrap();
    std::fs::write(full.join("summary.txt"), "my own notes\n").unwrap();
    let (t, code) = rlt_cli::discover::discover(&root(), &dir, false, Some(5));
    assert_eq!(code, 2, "{t}");
    assert!(t.contains("was not written by this command"), "{t}");
    assert_eq!(read(&full.join("summary.txt")), "my own notes\n");
    assert!(!full.join("D001.rlt").exists() && !full.join("manifest.tsv").exists());
}

// ── Audit D6: an id names one statement in one grammar ─────────────────────

#[test]
fn d6_ids_name_one_candidate_of_one_grammar() {
    use rlt_cli::discover::{candidates, discovery_id, statement_for_id};
    let one = candidates(false);
    let two = candidates(true);
    assert_eq!((one.len(), two.len()), (399, 3983));
    // The committed one-level ids are unchanged.
    assert_eq!(
        statement_for_id("D243").as_deref(),
        Some("∀x. x × 1 = 0 + x")
    );
    assert_eq!(statement_for_id("D023").as_deref(), Some("∀x. x + 0 = x"));
    // Every id of both grammars is distinct and names its own statement.
    let mut ids = std::collections::BTreeSet::new();
    for (two_level, list) in [(false, &one), (true, &two)] {
        for (i, st) in list.iter().enumerate() {
            let id = discovery_id(two_level, i);
            assert!(ids.insert(id.clone()), "{id} reused");
            assert_eq!(statement_for_id(&id).as_ref(), Some(st), "{id}");
        }
    }
    // The same statement has different ids in the two grammars, never the same id.
    let t = two.iter().position(|s| s == "∀x. x × 1 = 0 + x").unwrap();
    assert_eq!(discovery_id(true, t), "T2547");
    for bad in [
        "D000", "D400", "D0243", "D24", "T243", "T0000", "T3984", "X001", "D", "Dx01",
    ] {
        assert_eq!(statement_for_id(bad), None, "{bad}");
    }
}

#[test]
fn d6_two_level_discovery_writes_t_ids_that_verify() {
    let dir = scratch("d6-two-level");
    let _ = std::fs::remove_dir_all(root().join(&dir));
    let (summary, code) = rlt_cli::discover::discover(&root(), &dir, true, Some(300));
    assert_eq!(code, 0, "{summary}");
    let m = read(&root().join(&dir).join("manifest.tsv"));
    let rows: Vec<&str> = m.lines().filter(|l| !l.starts_with('#')).collect();
    assert!(!rows.is_empty());
    assert!(rows.iter().all(|r| r.starts_with('T')), "{m}");
    let (t, code) = rlt_cli::discover::verify(&root(), &dir);
    assert_eq!(code, 0, "{t}");
}

#[test]
fn d6_a_row_relabelled_with_another_valid_id_is_refused() {
    // Move D023's certificate to D024's name, with a matching header: the
    // kernel accepts it, the path rule holds, but D024 names another statement.
    let dir = discovery_copy("relabel");
    let full = root().join(&dir);
    let text = read(&full.join("D023.rlt")).replace("(id D023)", "(id D024)");
    std::fs::write(full.join("D024.rlt"), &text).unwrap();
    let sha = rlt_cli::sha256_hex(text.as_bytes());
    edit_row(&dir, "D023", |c| {
        c[0] = "D024".into();
        c[5] = format!("{dir}/D024.rlt");
        c[6] = sha.clone();
    });
    assert_eq!(check_file(&root(), &full.join("D024.rlt")).code, ACCEPT);
    refused(
        &dir,
        "D024",
        "the id D024 names the candidate `∀x. x + 0 = 0`, not `∀x. x + 0 = x`",
    );
}

// ── Audit D7: a rerun leaves no stale certificates ─────────────────────────

#[test]
fn d7_rerunning_discovery_removes_its_own_stale_certificates_only() {
    let dir = scratch("d7");
    let full = root().join(&dir);
    let _ = std::fs::remove_dir_all(&full);
    let (s, code) = rlt_cli::discover::discover(&root(), &dir, false, None);
    assert_eq!(code, 0, "{s}");
    // Files the run did not write survive the rerun.
    std::fs::write(full.join("notes.rlt"), "; mine\n").unwrap();
    std::fs::write(
        full.join("D300.rlt"),
        "; also mine, though named like an id\n",
    )
    .unwrap();
    let (s, code) = rlt_cli::discover::discover(&root(), &dir, false, Some(30));
    assert_eq!(code, 0, "{s}");
    assert!(s.contains("proved          1"), "{s}");
    let certs: Vec<String> = std::fs::read_dir(&full)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .filter(|n| n.ends_with(".rlt"))
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    assert_eq!(certs, ["D023.rlt", "D300.rlt", "notes.rlt"]);
    std::fs::remove_file(full.join("D300.rlt")).unwrap();
    let (t, code) = rlt_cli::discover::verify(&root(), &dir);
    assert_eq!(code, 0, "{t}");
    // A stale prover certificate that no row lists is reported by --verify.
    std::fs::copy(
        root().join("research/prover-discovery/D206.rlt"),
        full.join("D206.rlt"),
    )
    .unwrap();
    let (t, code) = rlt_cli::discover::verify(&root(), &dir);
    assert_eq!(code, 1, "{t}");
    assert!(t.contains("D206.rlt  UNLISTED"), "{t}");
}

// ── Audit F1: discovery certificates carry exactly the generated header ────

/// Replace `from` by `to` in DIR/ID.rlt and re-hash its manifest row, so only
/// the header check can object.
fn rewrite_cert(dir: &str, id: &str, from: &str, to: &str) {
    let p = root().join(dir).join(format!("{id}.rlt"));
    let t = read(&p);
    assert!(t.contains(from), "{from}");
    let t = t.replacen(from, to, 1);
    std::fs::write(&p, &t).unwrap();
    let sha = rlt_cli::sha256_hex(t.as_bytes());
    edit_row(dir, id, |c| c[6] = sha.clone());
}

#[test]
fn f1_semantic_assertions_in_a_discovery_certificate_are_refused() {
    // The final audit's three attacks: each certificate still passes plain
    // `rlt check` (which even prints the assertion), and is refused by --verify.
    for (tag, field) in [
        ("relationship", "(relationship \"math29.pairing\")"),
        ("readingpoint", "(readingpoint \"math29\")"),
        ("context", "(context strawberry PROVEN)"),
    ] {
        let dir = discovery_copy(&format!("f1-{tag}"));
        rewrite_cert(
            &dir,
            "D243",
            "(convention NAT-LOGIC)",
            &format!("(convention NAT-LOGIC)\n  {field}"),
        );
        let cert = root().join(&dir).join("D243.rlt");
        assert_eq!(check_file(&root(), &cert).code, ACCEPT, "{tag}");
        refused(
            &dir,
            "D243",
            &format!("fields not generated by discovery: {tag}"),
        );
    }
}

#[test]
fn f1_the_header_must_be_the_generated_one_in_content_and_order() {
    // A repeated (params …), a reordered header, and a changed id/goal pair
    // that the kernel accepts are all refused.
    let dir = discovery_copy("f1-params");
    rewrite_cert(&dir, "D243", "(params p1)", "(params p1)\n  (params p1)");
    let (t, code) = rlt_cli::discover::verify(&root(), &dir);
    assert_eq!(code, 1, "{t}");
    assert!(t.contains("D243  REFUSED"), "{t}");
    let dir = discovery_copy("f1-order");
    rewrite_cert(
        &dir,
        "D243",
        "(mode unconditional)\n  (axioms ROBINSON-Q)",
        "(axioms ROBINSON-Q)\n  (mode unconditional)",
    );
    assert_eq!(
        check_file(&root(), &root().join(&dir).join("D243.rlt")).code,
        ACCEPT
    );
    refused(
        &dir,
        "D243",
        "header: the fields differ from the generated goal",
    );
    // The unmodified certificates pass the header check.
    for e in std::fs::read_dir(root().join("research/prover-discovery")).unwrap() {
        let p = e.unwrap().path();
        if p.extension().is_some_and(|x| x == "rlt") {
            let id = p.file_stem().unwrap().to_string_lossy().to_string();
            rlt_cli::discover::exact_header(&id, &read(&p)).unwrap();
        }
    }
}

// ── Audit F2: the whole record — every candidate once, files in agreement ──

fn record_refused(dir: &str, reason: &str) -> String {
    let (t, code) = rlt_cli::discover::verify(&root(), dir);
    assert_eq!(code, 1, "{t}");
    assert!(
        t.lines()
            .any(|l| l.starts_with("record  REFUSED") && l.contains(reason)),
        "expected record refusal `{reason}`:\n{t}"
    );
    t
}

fn edit_file(dir: &str, name: &str, f: impl Fn(String) -> String) {
    let p = root().join(dir).join(name);
    let t = f(read(&p));
    std::fs::write(&p, t).unwrap();
}

#[test]
fn f2_the_committed_record_is_complete() {
    let (t, code) = rlt_cli::discover::verify(&root(), "research/prover-discovery");
    assert_eq!(code, 0, "{t}");
    assert!(t.contains("record          complete — one-level grammar, 399 of 399 candidates: 9 proved, 390 not found, each once"), "{t}");
}

#[test]
fn f2_a_summary_claiming_other_counts_or_another_grammar_is_refused() {
    // The audit's attack: proved 124, two-level.
    let dir = discovery_copy("f2-summary");
    edit_file(&dir, "summary.txt", |s| {
        s.replace("proved          9", "proved          124")
            .replace(
                "one-level (one operation in L; at most one in R)",
                "two-level (one operation in L, up to two in R)",
            )
    });
    let t = rlt_cli::discover::verify(&root(), &dir).0;
    assert!(t.contains("REFUSED"), "{t}");
    assert_ne!(rlt_cli::discover::verify(&root(), &dir).1, 0);
    // Counts alone.
    for (from, to) in [
        ("proved          9", "proved          124"),
        ("not found       390", "not found       389"),
        ("errors          0", "errors          1"),
        ("(399 candidates)", "(398 candidates)"),
        ("depth ≤ 1", "depth ≤ 12"),
    ] {
        let dir = discovery_copy("f2-counts");
        edit_file(&dir, "summary.txt", |s| s.replace(from, to));
        let (t, code) = rlt_cli::discover::verify(&root(), &dir);
        assert_eq!(code, 1, "{from}: {t}");
    }
    let dir = discovery_copy("f2-grammar");
    edit_file(&dir, "summary.txt", |s| {
        s.replace("one-level", "three-level")
    });
    record_refused(&dir, "no recognised grammar line");
}

#[test]
fn f2_a_silently_dropped_theorem_is_refused() {
    // The audit's attack: D206's row and certificate removed.
    let dir = discovery_copy("f2-drop");
    edit_file(&dir, "manifest.tsv", |m| {
        m.lines()
            .filter(|l| !l.starts_with("D206\t"))
            .map(|l| format!("{l}\n"))
            .collect()
    });
    std::fs::remove_file(root().join(&dir).join("D206.rlt")).unwrap();
    record_refused(&dir, "expected `D206");
}

#[test]
fn f2_a_proved_theorem_listed_as_not_found_is_refused() {
    // The audit's attack: D023 appended to not-found.tsv.
    let dir = discovery_copy("f2-both");
    edit_file(&dir, "not-found.tsv", |s| {
        s + "D023\t∀x. x + 0 = x\tNOT_FOUND_WITHIN_BOUNDS\n"
    });
    record_refused(
        &dir,
        "not-found.tsv: must list exactly the 390 unproved candidates",
    );
}

#[test]
fn f2_a_row_of_another_grammar_is_refused() {
    // The audit's attack: a T-id row in a one-level record.
    let dir = discovery_copy("f2-grammar-row");
    let full = root().join(&dir);
    let text = read(&full.join("D023.rlt")).replace("(id D023)", "(id T0279)");
    std::fs::write(full.join("T0279.rlt"), &text).unwrap();
    std::fs::remove_file(full.join("D023.rlt")).unwrap();
    let sha = rlt_cli::sha256_hex(text.as_bytes());
    edit_row(&dir, "D023", |c| {
        c[0] = "T0279".into();
        c[5] = format!("{dir}/T0279.rlt");
        c[6] = sha.clone();
    });
    record_refused(
        &dir,
        "T0279 is not one of the 399 candidates of the one-level grammar",
    );
}

#[test]
fn f2_omitted_added_or_reordered_candidates_are_refused() {
    let dir = discovery_copy("f2-omit");
    edit_file(&dir, "not-found.tsv", |s| {
        s.lines()
            .filter(|l| !l.starts_with("D001\t"))
            .map(|l| format!("{l}\n"))
            .collect()
    });
    record_refused(&dir, "expected `D001");
    let dir = discovery_copy("f2-add");
    edit_file(&dir, "not-found.tsv", |s| {
        s + "D400\t∀x. x = x\tNOT_FOUND_WITHIN_BOUNDS\n"
    });
    record_refused(&dir, "not-found.tsv");
    let dir = discovery_copy("f2-changed");
    edit_file(&dir, "not-found.tsv", |s| {
        s.replacen("NOT_FOUND_WITHIN_BOUNDS\n", "PROVED\n", 1)
    });
    record_refused(&dir, "not-found.tsv");
    let dir = discovery_copy("f2-order");
    edit_file(&dir, "manifest.tsv", |m| {
        let mut lines: Vec<&str> = m.lines().collect();
        let k = lines.len();
        lines.swap(k - 1, k - 2);
        lines.iter().map(|l| format!("{l}\n")).collect()
    });
    record_refused(&dir, "out of generation order");
    let dir = discovery_copy("f2-missing");
    std::fs::remove_file(root().join(&dir).join("not-found.tsv")).unwrap();
    record_refused(&dir, "not-found.tsv");
}

#[test]
fn f2_a_limited_run_is_verified_as_the_prefix_it_covers() {
    let dir = scratch("f2-limit");
    let _ = std::fs::remove_dir_all(root().join(&dir));
    let (s, code) = rlt_cli::discover::discover(&root(), &dir, false, Some(30));
    assert_eq!(code, 0, "{s}");
    let (t, code) = rlt_cli::discover::verify(&root(), &dir);
    assert_eq!(code, 0, "{t}");
    assert!(
        t.contains("one-level grammar, 30 of 399 candidates: 1 proved, 29 not found"),
        "{t}"
    );
}

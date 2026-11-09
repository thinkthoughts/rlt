//! RLT-003 M3c: divisibility and non-divisibility proofs, and theorem citation.
//!
//! The accepted proofs (tests/m3c/accept) and rejected cases
//! (tests/m3c/expected.tsv), plus the checks that need a scratch repository,
//! the kernel API or a report inspection: CT1 (exact derivation obligations),
//! CT2 (folded/unfolded identity of free235(7)), CT3 (citation reporting),
//! CR14 (edited definitions), CR23c (symlinked citation paths), citation depth
//! and work bounds, path spellings, convention mismatch,
//! and kernel-level refusal of forged or altered cited theorems.

use rlt_cli::{check_file, check_file_outcome, ACCEPT, MALFORMED, MAX_CITATION_DEPTH};
use rlt_kernel::{check, rules, Arg, Axiom, Mode, ProofObject, Prop, Step, Term, Verdict};
use sha2::{Digest, Sha256};
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

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

const CA02: &str = "tests/m3c/accept/CA02_two_not_divides_seven.rlt";
const CA03: &str = "tests/m3c/accept/CA03_three_not_divides_seven.rlt";
const CA04: &str = "tests/m3c/accept/CA04_five_not_divides_seven.rlt";
const CA05: &str = "tests/m3c/accept/CA05_free235_seven.rlt";

#[test]
fn accepted_proofs_are_unconditional_theorems_under_robinson_q() {
    let fs = files("tests/m3c/accept");
    assert_eq!(fs.len(), 9, "CA01 to CA09");
    for f in &fs {
        let r = check_file(&root(), f);
        assert_eq!(r.code, ACCEPT, "{}", r.text);
        assert!(
            verdict(&r.text).contains(
                "ACCEPT — unconditional theorem relative to registered axiom set ROBINSON-Q"
            ),
            "{}",
            r.text
        );
    }
}

#[test]
fn rejected_cases_fail_for_their_stated_reason() {
    let tsv = std::fs::read_to_string(root().join("tests/m3c/expected.tsv")).unwrap();
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
    assert_eq!(n, files("tests/m3c/reject").len());
    assert_eq!(n, 31);
}

// ── CT1: exact derivation obligations ──────────────────────────────────────

/// Number of `inst NAME` steps in a proof file.
fn inst_count(text: &str, axiom: &str) -> usize {
    text.matches(&format!("(inst {axiom} ")).count()
}

#[test]
fn ct1_axiom_instances_match_the_scheme() {
    // (file, d, layers L): q3 = L; q2 = L zero branches + 1 final; q1 = 7;
    // M_d per successor branch: mul_succ = L, add_succ = L·d, add_zero = L;
    // mul_zero = L (one per zero branch).
    let cases = [
        (CA02, 2, 4),
        (CA03, 3, 3),
        (CA04, 5, 2),
        ("tests/m3c/accept/CA06_two_witnesswise.rlt", 2, 4),
        ("tests/m3c/accept/CA07_three_witnesswise.rlt", 3, 3),
        ("tests/m3c/accept/CA08_five_witnesswise.rlt", 5, 2),
    ];
    for (f, d, l) in cases {
        let t = std::fs::read_to_string(root().join(f)).unwrap();
        assert_eq!(inst_count(&t, "q3_zero_or_succ"), l, "{f}: Q3 splits");
        assert_eq!(inst_count(&t, "q2_zero_ne_succ"), l + 1, "{f}: Q2 closures");
        assert_eq!(
            inst_count(&t, "q1_succ_inj"),
            7,
            "{f}: Q1 peels total n = 7"
        );
        assert_eq!(inst_count(&t, "mul_succ"), l, "{f}: M_d lemmas");
        assert_eq!(inst_count(&t, "add_succ"), l * d, "{f}: add_succ in M_d");
        assert_eq!(inst_count(&t, "add_zero"), l, "{f}: add_zero in M_d");
        assert_eq!(inst_count(&t, "mul_zero"), l, "{f}: zero branches");
        // Eigenvariables p0 … p_L, each declared once.
        let params: String = (0..=l)
            .map(|i| format!("p{i}"))
            .collect::<Vec<_>>()
            .join(" ");
        assert!(t.contains(&format!("(params {params})")), "{f}: params");
        let r = check_file(&root(), &root().join(f));
        assert_eq!(r.code, ACCEPT, "{}", r.text);
        assert!(
            r.text.contains("axioms used     add_succ, add_zero, mul_succ, mul_zero, q1_succ_inj, q2_zero_ne_succ, q3_zero_or_succ"),
            "{}",
            r.text
        );
    }
}

// ── CT2: folded and unfolded identity ──────────────────────────────────────

#[test]
fn ct2_free235_is_the_conjunction_of_the_three_cited_goals() {
    let g = |f: &str| check_file_outcome(&root(), &root().join(f)).prop.unwrap();
    let want = Prop::And(
        Box::new(Prop::And(Box::new(g(CA02)), Box::new(g(CA03)))),
        Box::new(g(CA04)),
    );
    let o = check_file_outcome(&root(), &root().join(CA05));
    assert_eq!(o.report.code, ACCEPT, "{}", o.report.text);
    assert_eq!(o.prop.unwrap(), want);
    assert_eq!(o.theorem.unwrap().conclusion(), &want);
}

// ── CT3: dependency and axiom reporting ────────────────────────────────────

#[test]
fn ct3_cites_lines_carry_hash_goal_gamma_set_and_axioms() {
    let o = check_file_outcome(&root(), &root().join(CA05));
    let text = &o.report.text;
    assert_eq!(o.report.code, ACCEPT, "{text}");
    let mut union = std::collections::BTreeSet::new();
    for (label, f) in [("T2", CA02), ("T3", CA03), ("T4", CA04)] {
        let bytes = std::fs::read(root().join(f)).unwrap();
        let cited = check_file_outcome(&root(), &root().join(f));
        let th = cited.theorem.unwrap();
        let used: Vec<String> = th.axioms_used().keys().cloned().collect();
        union.extend(used.iter().cloned());
        let block = format!(
            "cites           {label} = {f} ({})\n  sha256        {}\n  proved        {}\n  assumptions   Γ = ∅\n  axiom set     ROBINSON-Q\n  axioms used   {}\n",
            cited.proof.unwrap().id,
            sha(&bytes),
            th.conclusion(),
            used.join(", ")
        );
        assert!(text.contains(&block), "missing:\n{block}\nin:\n{text}");
    }
    let union: Vec<String> = union.into_iter().collect();
    assert!(
        text.contains(&format!("\naxioms used     {}\n", union.join(", "))),
        "{text}"
    );
    let th = o.theorem.unwrap();
    assert!(th.is_unconditional());
    assert_eq!(th.axioms_used().keys().cloned().collect::<Vec<_>>(), union);
}

// ── Scratch repositories: CR14, CR23c, depth ───────────────────────────────

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

fn scratch(name: &str) -> PathBuf {
    let s = std::env::temp_dir().join(format!("rlt-m3c-{name}-{}", std::process::id()));
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
    s
}

fn citing(id: &str, label: &str, path: &str, hash: &str, claim: &str, goal: &str) -> String {
    format!(
        "(rlt-proof\n  (id {id})\n  (convention NAT-DEF)\n  (goal \"{goal}\")\n  (mode unconditional)\n  (axioms ROBINSON-Q)\n  (cites ({label} \"{path}\" \"{hash}\"))\n  (steps\n    (t {claim} (cite {label}))))\n"
    )
}

#[test]
fn cr14_edited_definitions_break_their_pin_for_m3c_files() {
    let s = scratch("cr14");
    let f = s.join("spec/definitions.rlt");
    let t = std::fs::read_to_string(&f).unwrap();
    std::fs::write(
        &f,
        t.replacen("(exists k (= n (* d k)))", "(exists k (= (* d k) n))", 1),
    )
    .unwrap();
    let r = check_file(&s, &s.join(CA02));
    let r5 = check_file(&s, &s.join(CA05));
    let _ = std::fs::remove_dir_all(&s);
    for r in [r, r5] {
        assert_eq!(r.code, MALFORMED, "{}", r.text);
        assert!(
            verdict(&r.text).contains("the definitions changed without review"),
            "{}",
            r.text
        );
    }
}

#[test]
fn cr23c_symlinked_citation_paths_are_refused() {
    let s = scratch("cr23c");
    let h = sha(&std::fs::read(s.join(CA02)).unwrap());
    // A symlinked file, and a symlinked directory on the way to a regular file.
    std::os::unix::fs::symlink(s.join(CA02), s.join("tests/m3c/accept/link.rlt")).unwrap();
    std::os::unix::fs::symlink(s.join("tests/m3c/accept"), s.join("tests/m3c/linkdir")).unwrap();
    let mut out = Vec::new();
    for (n, p) in [
        ("a", "tests/m3c/accept/link.rlt"),
        ("b", "tests/m3c/linkdir/CA02_two_not_divides_seven.rlt"),
    ] {
        let f = format!("tests/m3c/cr23c_{n}.rlt");
        std::fs::write(
            s.join(&f),
            citing("M3c.CR23c", "T", p, &h, "(not (dvd 2 7))", "¬(2 ∣ 7)"),
        )
        .unwrap();
        out.push(check_file(&s, &s.join(&f)));
    }
    let _ = std::fs::remove_dir_all(&s);
    for r in out {
        assert_eq!(r.code, MALFORMED, "{}", r.text);
        assert!(
            verdict(&r.text).contains("a symlink was found"),
            "{}",
            r.text
        );
    }
}

#[test]
fn citation_chains_are_bounded() {
    // depth_i cites depth_{i-1}; depth_0 cites the uncited leaf CA02. A chain
    // of exactly MAX_CITATION_DEPTH citing files is accepted; one more is
    // refused.
    let s = scratch("depth");
    let mut prev = (CA02.to_string(), sha(&std::fs::read(s.join(CA02)).unwrap()));
    let mut chain = Vec::new();
    for i in 0..=MAX_CITATION_DEPTH {
        let f = format!("tests/m3c/depth_{i}.rlt");
        let text = citing(
            &format!("M3c.D{i}"),
            "T",
            &prev.0,
            &prev.1,
            "(not (dvd 2 7))",
            "¬(2 ∣ 7)",
        );
        std::fs::write(s.join(&f), &text).unwrap();
        prev = (f.clone(), sha(text.as_bytes()));
        chain.push(f);
    }
    // chain[MAX - 1]: exactly MAX citing files; chain[MAX]: MAX + 1.
    let at_limit = check_file(&s, &s.join(&chain[MAX_CITATION_DEPTH - 1]));
    let over = check_file(&s, &s.join(&chain[MAX_CITATION_DEPTH]));
    assert_eq!(at_limit.code, ACCEPT, "{}", at_limit.text);
    assert_eq!(over.code, MALFORMED, "{}", over.text);
    assert!(
        verdict(&over.text).contains("nest deeper than"),
        "{}",
        over.text
    );
    let _ = std::fs::remove_dir_all(&s);
}

#[test]
fn citation_work_is_bounded() {
    // Two citations per file, five levels: 2 + 4 + … + 32 = 62 checks, within
    // MAX_CITED_CHECKS = 64; six levels need 126 and are refused.
    let s = scratch("budget");
    let mut prev = (CA02.to_string(), sha(&std::fs::read(s.join(CA02)).unwrap()));
    let mut files = Vec::new();
    for i in 0..6 {
        let f = format!("tests/m3c/fan_{i}.rlt");
        let text = format!(
            "(rlt-proof\n  (id M3c.F{i})\n  (convention NAT-DEF)\n  (goal \"¬(2 ∣ 7)\")\n  (mode unconditional)\n  (axioms ROBINSON-Q)\n  (cites (A \"{p}\" \"{h}\") (B \"{p}\" \"{h}\"))\n  (steps\n    (t (not (dvd 2 7)) (cite A))))\n",
            p = prev.0,
            h = prev.1
        );
        std::fs::write(s.join(&f), &text).unwrap();
        prev = (f.clone(), sha(text.as_bytes()));
        files.push(f);
    }
    let ok = check_file(&s, &s.join(&files[4]));
    let over = check_file(&s, &s.join(&files[5]));
    let _ = std::fs::remove_dir_all(&s);
    assert_eq!(ok.code, ACCEPT, "{}", ok.text);
    assert_eq!(over.code, MALFORMED, "{}", over.text);
    assert!(
        verdict(&over.text).contains("more than 64 files"),
        "{}",
        over.text
    );
}

#[test]
fn citation_paths_with_dot_or_empty_components_are_refused() {
    let h = sha(&std::fs::read(root().join(CA02)).unwrap());
    let s = scratch("dots");
    let mut out = Vec::new();
    for (n, p) in [
        ("a", "tests/./m3c/accept/CA02_two_not_divides_seven.rlt"),
        ("b", "tests//m3c/accept/CA02_two_not_divides_seven.rlt"),
        ("c", "./tests/m3c/accept/CA02_two_not_divides_seven.rlt"),
    ] {
        let f = format!("tests/m3c/dots_{n}.rlt");
        std::fs::write(
            s.join(&f),
            citing("M3c.Dots", "T", p, &h, "(not (dvd 2 7))", "¬(2 ∣ 7)"),
        )
        .unwrap();
        out.push(check_file(&s, &s.join(&f)));
    }
    let _ = std::fs::remove_dir_all(&s);
    for r in out {
        assert_eq!(r.code, MALFORMED, "{}", r.text);
        assert!(verdict(&r.text).contains("plain components"), "{}", r.text);
    }
}

#[test]
fn a_cited_theorem_under_another_convention_is_refused() {
    // Same axiom set (ROBINSON-Q), different convention (NAT-LOGIC): refused.
    let s = scratch("conv");
    let logic = "(rlt-proof\n  (id M3c.Logic)\n  (convention NAT-LOGIC)\n  (goal \"¬(0 = 7)\")\n  (mode unconditional)\n  (axioms ROBINSON-Q)\n  (steps\n    (s1 (not (= 0 7)) (inst q2_zero_ne_succ (x 6)))))\n";
    std::fs::write(s.join("tests/m3c/logic.rlt"), logic).unwrap();
    let direct = check_file(&s, &s.join("tests/m3c/logic.rlt"));
    let f = "tests/m3c/conv.rlt";
    std::fs::write(
        s.join(f),
        citing(
            "M3c.Conv",
            "T",
            "tests/m3c/logic.rlt",
            &sha(logic.as_bytes()),
            "(not (= 0 7))",
            "¬(0 = 7)",
        ),
    )
    .unwrap();
    let r = check_file(&s, &s.join(f));
    let _ = std::fs::remove_dir_all(&s);
    assert_eq!(direct.code, ACCEPT, "{}", direct.text);
    assert_eq!(r.code, MALFORMED, "{}", r.text);
    assert!(
        verdict(&r.text).contains("convention NAT-LOGIC differs from this file's NAT-DEF"),
        "{}",
        r.text
    );
}

#[test]
fn a_cited_file_is_checked_on_the_bytes_it_was_hashed_from() {
    // Swapping the cited file for another accepted proof with the same goal
    // (CA09 also proves ¬(2 ∣ 7)) breaks the recorded hash.
    let s = scratch("swap");
    let ca09 =
        std::fs::read(s.join("tests/m3c/accept/CA09_witnesswise_gives_not_dvd.rlt")).unwrap();
    std::fs::write(s.join(CA02), &ca09).unwrap();
    let r = check_file(&s, &s.join(CA05));
    let _ = std::fs::remove_dir_all(&s);
    assert_eq!(r.code, MALFORMED, "{}", r.text);
    assert!(
        verdict(&r.text).contains("the citation records"),
        "{}",
        r.text
    );
}

// ── The kernel re-validates every cited theorem ────────────────────────────

fn robinson() -> Vec<Axiom> {
    rlt_cli::load_axiom_set(&root(), "ROBINSON-Q")
        .unwrap()
        .axioms
}

fn cite_proof(claim: Prop, th: rules::Theorem) -> ProofObject {
    ProofObject {
        goal: claim.clone(),
        mode: Mode::Unconditional,
        assumptions: vec![],
        params: vec![],
        steps: vec![Step {
            label: "t".into(),
            claim,
            rule: "cite".into(),
            args: vec![Arg::Name("T".into())],
        }],
        cited: vec![("T".into(), th)],
    }
}

fn reason(v: Verdict) -> String {
    match v {
        Verdict::Rejected { reason, .. } => reason,
        Verdict::Accepted(_) => panic!("accepted"),
    }
}

#[test]
fn kernel_refuses_a_cited_theorem_on_a_foreign_axiom() {
    // A theorem built from an axiom named like q2 but with different content.
    let fake = Axiom::formula(
        "q2_zero_ne_succ",
        vec!["x".into()],
        Prop::Eq(rlt_kernel::Formula::new(
            Term::Var("x".into()),
            Term::Var("x".into()),
        )),
        "OLP-ROBINSON-Q",
    )
    .unwrap();
    let th = rules::inst(&fake, &[("x".into(), Term::numeral(0))]).unwrap();
    let r = reason(check(&cite_proof(th.conclusion().clone(), th), &robinson()));
    assert!(r.contains("differs from or is absent"), "{r}");
}

#[test]
fn kernel_refuses_a_conditional_or_parametric_cited_theorem() {
    let phi = Prop::Eq(rlt_kernel::Formula::new(Term::numeral(0), Term::numeral(1)));
    let cond = rules::hyp("h", &phi).unwrap();
    let r = reason(check(&cite_proof(phi, cond), &robinson()));
    assert!(r.contains("is conditional"), "{r}");
    let p = Term::Param("a".into());
    let open = rules::refl(&p).unwrap();
    let mut po = cite_proof(open.conclusion().clone(), open);
    po.goal = Prop::Eq(rlt_kernel::Formula::new(Term::numeral(0), Term::numeral(0)));
    let r = reason(check(&po, &robinson()));
    assert!(r.contains("occurs in its conclusion"), "{r}");
}

#[test]
fn kernel_cite_is_bookkeeping_not_a_rule() {
    // The 20 inference rules are unchanged; `cite` returns the cited theorem
    // unchanged, and an undeclared citation label is refused.
    assert_eq!(rlt_kernel::RULES.len(), 20);
    assert!(!rlt_kernel::RULES.contains(&"cite"));
    let th = rules::refl(&Term::numeral(3)).unwrap();
    let mut po = cite_proof(th.conclusion().clone(), th.clone());
    match check(&po, &robinson()) {
        Verdict::Accepted(t) => assert_eq!(t, th),
        Verdict::Rejected { reason, .. } => panic!("{reason}"),
    }
    po.cited.clear();
    let r = reason(check(&po, &robinson()));
    assert!(r.contains("is not a declared citation"), "{r}");
}

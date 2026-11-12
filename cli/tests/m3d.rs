//! RLT-003 M3d: finite enumeration below 30.
//!
//! The accepted proofs (tests/m3d/accept) and rejected cases
//! (tests/m3d/expected.tsv), plus CT1 (exact rule and axiom-instance counts),
//! CT2 (the checked goal compared with an independently built formula, and
//! its 30 distinct disjuncts in written order), CT3 (citation reporting in
//! DA05), CR-lt (an edited `lt` definition breaks the pin) and size bounds.

use rlt_cli::{check_file, check_file_outcome, ACCEPT, MALFORMED};
use rlt_kernel::{Formula, Prop, Term};
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

fn line<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    text.lines()
        .find(|l| l.starts_with(key))
        .map(|l| l[key.len()..].trim())
}

const DA01: &str = "tests/m3d/accept/DA01_below_thirty.rlt";
const DA05: &str = "tests/m3d/accept/DA05_seven_from_the_enumeration.rlt";

#[test]
fn accepted_proofs_are_unconditional_theorems_under_robinson_q() {
    let fs = files("tests/m3d/accept");
    assert_eq!(fs.len(), 5, "DA01 to DA05");
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
        assert_eq!(line(&r.text, "assumptions"), Some("Γ = ∅"));
    }
}

#[test]
fn rejected_cases_fail_for_their_stated_reason() {
    let tsv = std::fs::read_to_string(root().join("tests/m3d/expected.tsv")).unwrap();
    let mut n = 0;
    for l in tsv.lines().filter(|l| l.starts_with("tests/")) {
        let mut c = l.split('\t');
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
    assert_eq!(n, files("tests/m3d/reject").len());
    assert_eq!(n, 16);
}

// ── CT1: exact rule and axiom-instance counts ──────────────────────────────

#[test]
fn ct1_da01_has_exactly_the_scheme_s_steps() {
    let t = std::fs::read_to_string(root().join(DA01)).unwrap();
    let count = |needle: &str| t.matches(needle).count();
    for (needle, want) in [
        ("(inst q3_zero_or_succ ", 31),
        ("(inst q1_succ_inj ", 30),
        ("(inst add_succ ", 31),
        ("(inst add_zero ", 1),
        ("(inst q2_zero_ne_succ ", 2),
        ("(inst mul_", 0),
        ("(subst ", 60),
        ("(or-right ", 29),
        ("(or-left ", 435),
        ("(exists-elim ", 32),
        ("(or-cases ", 31),
        ("(false-elim ", 1),
        ("(imp-intro ", 1),
        ("(forall-intro ", 1),
    ] {
        assert_eq!(count(needle), want, "DA01: {needle}");
    }
    // The two closing Q2 instances are at the arbitrary terms w and S(w) + a31.
    assert!(t.contains("(inst q2_zero_ne_succ (x w))"));
    assert!(t.contains("(inst q2_zero_ne_succ (x (+ (S w) a31)))"));
    // Q3 splits exactly a, a1, …, a30 (never a31).
    assert!(t.contains("(inst q3_zero_or_succ (x a))"));
    for i in 1..=30 {
        assert!(
            t.contains(&format!("(inst q3_zero_or_succ (x a{i}))")),
            "a{i}"
        );
    }
    assert!(!t.contains("(inst q3_zero_or_succ (x a31))"));
    let r = check_file(&root(), &root().join(DA01));
    assert_eq!(line(&r.text, "steps"), Some("971"));
    assert_eq!(
        line(&r.text, "axioms used"),
        Some("add_succ, add_zero, q1_succ_inj, q2_zero_ne_succ, q3_zero_or_succ")
    );
    for name in line(&r.text, "rules used").unwrap().split(", ") {
        assert!(rlt_kernel::RULES.contains(&name), "{name}");
    }
}

// ── CT2: the checked goal, built independently ─────────────────────────────

fn eq(a: Term, b: Term) -> Prop {
    Prop::Eq(Formula::new(a, b))
}

/// ∀x. (∃z. S(z) + x = n) → E, with E the left-nested disjunction of x = v.
fn enumeration_goal(n: u64, values: &[u64]) -> Prop {
    // Under ∀x: x is Bound(0); under ∀x ∃z: z is Bound(0), x is Bound(1).
    let lt = Prop::Exists(
        "z".into(),
        Box::new(eq(
            Term::plus(Term::succ(Term::Bound(0)), Term::Bound(1)),
            Term::numeral(n),
        )),
    );
    let mut e = eq(Term::Bound(0), Term::numeral(values[0]));
    for v in &values[1..] {
        e = Prop::Or(Box::new(e), Box::new(eq(Term::Bound(0), Term::numeral(*v))));
    }
    Prop::Forall("x".into(), Box::new(Prop::Imp(Box::new(lt), Box::new(e))))
}

#[test]
fn ct2_the_checked_goal_is_the_enumeration_below_thirty() {
    let o = check_file_outcome(&root(), &root().join(DA01));
    assert_eq!(o.report.code, ACCEPT, "{}", o.report.text);
    let want = enumeration_goal(30, &(0..30).collect::<Vec<_>>());
    let goal = o.prop.unwrap();
    assert_eq!(goal, want);
    assert_eq!(o.theorem.unwrap().conclusion(), &want);

    // Exactly 30 distinct equalities x = 0 … x = 29, in written order, as a
    // left-nested disjunction: every left child is a disjunction or E₀,
    // every right child an equality.
    let Prop::Forall(_, body) = goal else {
        panic!("not ∀")
    };
    let Prop::Imp(_, mut e) = *body else {
        panic!("not →")
    };
    let mut rights: Vec<Prop> = Vec::new();
    loop {
        match *e {
            Prop::Or(l, r) => {
                assert!(matches!(*r, Prop::Eq(_)), "right child is no equality: {r}");
                rights.push(*r);
                e = l;
            }
            Prop::Eq(_) => {
                rights.push(*e);
                break;
            }
            other => panic!("unexpected node in the disjunction: {other}"),
        }
    }
    rights.reverse();
    assert_eq!(rights.len(), 30);
    for (k, d) in rights.iter().enumerate() {
        assert_eq!(
            d,
            &eq(Term::Bound(0), Term::numeral(k as u64)),
            "disjunct {k}"
        );
    }
    let mut seen = std::collections::BTreeSet::new();
    for d in &rights {
        assert!(seen.insert(d.to_string()), "repeated disjunct {d}");
    }
    // The written goal lists the same 30 values in the same order.
    let r = &o.report.text;
    let written: String = (0..30)
        .map(|k| format!("x = {k}"))
        .collect::<Vec<_>>()
        .join(" ∨ ");
    assert!(
        r.contains(&format!("written goal    \"∀x. x < 30 → {written}\"")),
        "{r}"
    );
}

#[test]
fn ct2_the_companions_state_the_intended_bounds() {
    for (f, n, vals) in [
        ("tests/m3d/accept/DA02_below_three.rlt", 3, vec![0, 1, 2]),
        ("tests/m3d/accept/DA03_below_one.rlt", 1, vec![0]),
    ] {
        let o = check_file_outcome(&root(), &root().join(f));
        assert_eq!(o.prop.unwrap(), enumeration_goal(n, &vals), "{f}");
    }
    // DA04: ∀x. ¬(x < 0), that is ∀x. (∃z. S(z) + x = 0) → ⊥.
    let o = check_file_outcome(
        &root(),
        &root().join("tests/m3d/accept/DA04_nothing_below_zero.rlt"),
    );
    let lt0 = Prop::Exists(
        "z".into(),
        Box::new(eq(
            Term::plus(Term::succ(Term::Bound(0)), Term::Bound(1)),
            Term::numeral(0),
        )),
    );
    assert_eq!(
        o.prop.unwrap(),
        Prop::Forall(
            "x".into(),
            Box::new(Prop::Imp(Box::new(lt0), Box::new(Prop::False)))
        )
    );
}

// ── CT3: reuse by citation ─────────────────────────────────────────────────

#[test]
fn ct3_da05_cites_the_exact_enumeration_theorem() {
    let bytes = std::fs::read(root().join(DA01)).unwrap();
    let hash: String = Sha256::digest(&bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let cited = check_file_outcome(&root(), &root().join(DA01));
    let th = cited.theorem.unwrap();
    let r = check_file(&root(), &root().join(DA05));
    assert_eq!(r.code, ACCEPT, "{}", r.text);
    let block = format!(
        "cites           D1 = {DA01} (M3d.DA01)\n  sha256        {hash}\n  proved        {}\n  assumptions   Γ = ∅\n  axiom set     ROBINSON-Q\n  axioms used   add_succ, add_zero, q1_succ_inj, q2_zero_ne_succ, q3_zero_or_succ\n",
        th.conclusion()
    );
    assert!(r.text.contains(&block), "{}", r.text);
    assert_eq!(line(&r.text, "rules used"), Some("forall-elim"));
    assert!(line(&r.text, "theorems cited").unwrap().starts_with("D1 —"));
}

// ── CR-lt and size ─────────────────────────────────────────────────────────

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

#[test]
fn cr_lt_an_edited_order_definition_breaks_its_pin() {
    let s = std::env::temp_dir().join(format!("rlt-m3d-lt-{}", std::process::id()));
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
    let f = s.join("spec/definitions.rlt");
    let t = std::fs::read_to_string(&f).unwrap();
    std::fs::write(
        &f,
        t.replacen(
            "(exists z (= (+ (S z) x) y))",
            "(exists z (= (+ x (S z)) y))",
            1,
        ),
    )
    .unwrap();
    let r = check_file(&s, &s.join(DA01));
    let _ = std::fs::remove_dir_all(&s);
    assert_eq!(r.code, MALFORMED, "{}", r.text);
    assert!(
        verdict(&r.text).contains("the definitions changed without review"),
        "{}",
        r.text
    );
}

/// The parser's node budget for a proof file (as `parse_proof_file_with`
/// computes it): integers count n + 1, lists 1 plus their items, atoms 1.
fn budget(e: &rlt_syntax::sexpr::Sexp) -> u64 {
    use rlt_syntax::sexpr::Sexp;
    match e {
        Sexp::Int(n) => n + 1,
        Sexp::List(v) => v.iter().fold(1, |a, x| a + budget(x)),
        _ => 1,
    }
}

#[test]
fn size_da01_stays_inside_the_documented_bounds() {
    let src = std::fs::read_to_string(root().join(DA01)).unwrap();
    let nodes: u64 = rlt_syntax::sexpr::parse_all(&src)
        .unwrap()
        .iter()
        .map(budget)
        .sum();
    assert!(nodes < 300_000, "DA01 node budget {nodes}");
    assert!(src.len() < 250_000, "DA01 is {} bytes", src.len());
    let t = std::time::Instant::now();
    let r = check_file(&root(), &root().join(DA01));
    assert_eq!(r.code, ACCEPT);
    assert!(t.elapsed().as_secs() < 20, "DA01 took {:?}", t.elapsed());
    eprintln!(
        "DA01: {} bytes, node budget {nodes}, checked in {:?}",
        src.len(),
        t.elapsed()
    );
}

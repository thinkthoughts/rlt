//! RLT-003 M3e: the complete modulo-30 classification.
//!
//! Accepted proofs (tests/m3e/accept) and rejected cases
//! (tests/m3e/expected.tsv), plus CT1 (obligations), CT2 (meaning), CT3
//! (source: E-T1's list equals RESIDUE-30's pinned representatives), CT4
//! (citation graph and budget), CT5 (rules used are kernel rules) and ER15
//! (a residue record still refuses a `backed by` key).

use rlt_cli::residues::residues_report;
use rlt_cli::{check_file, check_file_outcome, ACCEPT, MALFORMED};
use rlt_kernel::{Formula, Prop, Term};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
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

fn verdict(t: &str) -> &str {
    t.lines().find(|l| l.starts_with("verdict")).unwrap_or("")
}

fn line<'a>(t: &'a str, key: &str) -> Option<&'a str> {
    t.lines()
        .find(|l| l.starts_with(key))
        .map(|l| l[key.len()..].trim())
}

const R8: [u64; 8] = [1, 7, 11, 13, 17, 19, 23, 29];
const T1: &str = "tests/m3e/accept/E_T1.rlt";

#[test]
fn accepted_proofs_are_unconditional_theorems_under_robinson_q() {
    let fs = files("tests/m3e/accept");
    assert_eq!(fs.len(), 54, "22 PD, 21 ND, 7 F, E-FWD, E-BWD, E-T1, EA-07");
    for f in &fs {
        let r = check_file(&root(), f);
        assert_eq!(r.code, ACCEPT, "{}", r.text);
        assert!(verdict(&r.text).contains("ROBINSON-Q"), "{}", r.text);
        assert_eq!(
            line(&r.text, "assumptions"),
            Some("Γ = ∅"),
            "{}",
            f.display()
        );
    }
}

#[test]
fn rejected_cases_fail_for_their_stated_reason() {
    let tsv = std::fs::read_to_string(root().join("tests/m3e/expected.tsv")).unwrap();
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
    assert_eq!(n, files("tests/m3e/reject").len());
    assert_eq!(n, 16);
}

fn text(f: &str) -> String {
    std::fs::read_to_string(root().join(f)).unwrap()
}

#[test]
fn ct1_obligations() {
    let fwd = text("tests/m3e/accept/E_FWD.rlt");
    assert_eq!(fwd.matches("(or-cases ").count(), 29, "E-FWD peels E29");
    assert_eq!(
        fwd.matches("(subst ").count(),
        22,
        "one per non-representative"
    );
    assert_eq!(fwd.matches("(cite P").count(), 22);
    let bwd = text("tests/m3e/accept/E_BWD.rlt");
    assert_eq!(bwd.matches("(or-cases ").count(), 7, "E-BWD peels R8");
    assert_eq!(bwd.matches("(subst ").count(), 8, "one per representative");
    assert_eq!(bwd.matches("(cite N").count(), 24);
    // ND leaves: L = ⌊r/d⌋ + 1 Q3 splits and L + 1 Q2 closures (M3c's scheme).
    for r in R8.iter().filter(|r| **r != 7) {
        for d in [2u64, 3, 5] {
            let t = text(&format!("tests/m3e/accept/ND_{d}_{r:02}.rlt"));
            let l = (r / d + 1) as usize;
            assert_eq!(t.matches("(inst q3_zero_or_succ ").count(), l, "ND {d} {r}");
            assert_eq!(
                t.matches("(inst q2_zero_ne_succ ").count(),
                l + 1,
                "ND {d} {r}"
            );
            assert_eq!(
                t.matches("(inst q1_succ_inj ").count(),
                *r as usize,
                "ND {d} {r}"
            );
        }
    }
}

fn num(n: u64) -> Term {
    Term::numeral(n)
}

fn eqp(a: Term, b: Term) -> Prop {
    Prop::Eq(Formula::new(a, b))
}

/// Walk a left-nested disjunction of `v = k` equalities; return the k's.
fn disjuncts(p: &Prop, v: &Term) -> Vec<Term> {
    match p {
        Prop::Or(l, r) => {
            let mut out = disjuncts(l, v);
            match r.as_ref() {
                Prop::Eq(f) if &f.lhs == v => out.push(f.rhs.clone()),
                other => panic!("right child is not an equality on the variable: {other}"),
            }
            out
        }
        Prop::Eq(f) if &f.lhs == v => vec![f.rhs.clone()],
        other => panic!("not a left-nested disjunction of equalities: {other}"),
    }
}

#[test]
fn ct2_checked_goal_is_the_classification() {
    let o = check_file_outcome(&root(), &root().join(T1));
    let goal = o.prop.unwrap();
    assert_eq!(o.theorem.unwrap().conclusion(), &goal);
    // ∀r. (∃z. S(z) + r = 30) → ((F(r) → R8(r)) ∧ (R8(r) → F(r)))
    let Prop::Forall(_, body) = &goal else {
        panic!("{goal}")
    };
    let Prop::Imp(lt, conj) = body.as_ref() else {
        panic!("{goal}")
    };
    let r = Term::Bound(0);
    let want_lt = Prop::Exists(
        "z".into(),
        Box::new(eqp(
            Term::App(
                "+".into(),
                vec![Term::App("S".into(), vec![Term::Bound(0)]), Term::Bound(1)],
            ),
            num(30),
        )),
    );
    assert_eq!(lt.as_ref(), &want_lt, "antecedent is r < 30, unfolded");
    let Prop::And(fwd, bwd) = conj.as_ref() else {
        panic!("{goal}")
    };
    let (Prop::Imp(f1, r1), Prop::Imp(r2, f2)) = (fwd.as_ref(), bwd.as_ref()) else {
        panic!("{goal}")
    };
    assert_eq!(f1, f2, "both directions name the same free235(r)");
    assert_eq!(r1, r2, "both directions name the same R8(r)");
    let ks = disjuncts(r1, &r);
    assert_eq!(
        ks,
        R8.iter().map(|k| num(*k)).collect::<Vec<_>>(),
        "eight, in order"
    );
    // free235(r) = (¬(2∣r) ∧ ¬(3∣r)) ∧ ¬(5∣r), with d ∣ r = ∃k. r = d × k.
    let nd = |d: u64| {
        Prop::Imp(
            Box::new(Prop::Exists(
                "k".into(),
                Box::new(eqp(
                    Term::Bound(1),
                    Term::App("*".into(), vec![num(d), Term::Bound(0)]),
                )),
            )),
            Box::new(Prop::False),
        )
    };
    let free = Prop::And(
        Box::new(Prop::And(Box::new(nd(2)), Box::new(nd(3)))),
        Box::new(nd(5)),
    );
    assert_eq!(f1.as_ref(), &free);
}

#[test]
fn ct3_the_list_equals_residue_30s_pinned_representatives() {
    let r = residues_report(&root(), &root().join("residues/residue-30.yaml"));
    let rep = line(&r.text, "  representatives").unwrap();
    assert!(
        rep.starts_with("[1, 7, 11, 13, 17, 19, 23, 29] quoted from [NEWS-MATH29]"),
        "{rep}"
    );
    let goal = text(T1);
    let written = goal.lines().find(|l| l.contains("(goal")).unwrap();
    assert!(
        written.contains("r = 1 ∨ r = 7 ∨ r = 11 ∨ r = 13 ∨ r = 17 ∨ r = 19 ∨ r = 23 ∨ r = 29"),
        "{written}"
    );
}

#[test]
fn ct4_citation_graph_and_budget() {
    let cites = |f: &str| {
        let t = text(f);
        t.split("(cites").nth(1).map_or(0, |c| {
            c.split("(steps")
                .next()
                .unwrap()
                .matches("\"tests/")
                .count()
        })
    };
    assert_eq!(cites("tests/m3e/accept/E_FWD.rlt"), 23, "DA01 and 22 PD");
    assert_eq!(
        cites("tests/m3e/accept/E_BWD.rlt"),
        24,
        "24 ND (3 from M3c)"
    );
    assert_eq!(cites(T1), 2, "E-FWD and E-BWD");
    // 1 + 23 + 1 + 24 = 49 cited-file checks: inside the budget of 64; two
    // copies of E-T1 exceed it (ER13).
    let r = check_file(&root(), &root().join(T1));
    assert_eq!(r.code, ACCEPT);
    assert_eq!(
        line(&r.text, "theorems cited").unwrap().split(" —").next(),
        Some("BWD, FWD")
    );
}

#[test]
fn ct5_rules_used_are_kernel_rules() {
    for f in files("tests/m3e/accept") {
        let r = check_file(&root(), &f);
        for n in line(&r.text, "rules used")
            .unwrap()
            .split(", ")
            .filter(|x| *x != "none")
        {
            assert!(rlt_kernel::RULES.contains(&n), "{}: {n}", f.display());
        }
    }
}

#[test]
fn er15_residue_30_still_refuses_a_backed_by_key() {
    let s = std::env::temp_dir().join(format!("rlt-m3e-er15-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&s);
    std::fs::create_dir_all(&s).unwrap();
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
        let st = std::process::Command::new("cp")
            .arg("-r")
            .arg(root().join(d))
            .arg(&s)
            .status()
            .unwrap();
        assert!(st.success());
    }
    let f = s.join("residues/residue-30.yaml");
    let t = std::fs::read_to_string(&f).unwrap();
    std::fs::write(
        &f,
        t.replacen(
            "references: [math29]",
            "references: [math29]\nbacked_by: tests/m3e/accept/E_T1.rlt",
            1,
        ),
    )
    .unwrap();
    let r = residues_report(&s, &f);
    let _ = std::fs::remove_dir_all(&s);
    assert_eq!(r.code, MALFORMED, "{}", r.text);
}

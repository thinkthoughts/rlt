//! RLT-007b stage 2a: the primes below 213 under the bounded definition
//! Prime_B(p) :≡ 1 < p ∧ ∀d. ((1 < d ∧ d < p) → ¬(d ∣ p)). The six non-prime
//! certificates, the six exhaustiveness theorems, the six order splits, the
//! classification citing them, the rejected cases, the generator's checks and
//! the seven statement records.

use std::path::{Path, PathBuf};

use rlt_cli::check_file;
use rlt_cli::deps::deps;
use rlt_cli::explain::{explain, render_text};

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
            "scripts/gen-rlt007s2a.py",
            env!("CARGO_BIN_EXE_rlt"),
            arg,
        ])
        .current_dir(root())
        .output()
        .expect("python3 runs the generator")
}

const VERDICT: &str =
    "verdict         ACCEPT — unconditional theorem relative to registered axiom set ROBINSON-Q";

const PB: &str = "(1 < p ∧ ∀d. ((1 < d ∧ d < p) → ¬(d ∣ p)))";

fn goal(rel: &str) -> String {
    let t = std::fs::read_to_string(root().join(rel)).unwrap();
    let i = t.find("(goal \"").unwrap() + 7;
    t[i..i + t[i..].find('"').unwrap()].to_string()
}

#[test]
fn nineteen_theorems_are_accepted_and_state_prime_b_explicitly() {
    let mut n = 0;
    for k in ["N", "E", "O"] {
        for i in 1..=6 {
            let rel = format!("tests/rlt007s2a/accept/{k}_{i}.rlt");
            let r = check_file(&root(), &root().join(&rel));
            assert!(r.text.contains(VERDICT), "{rel}:\n{}", r.text);
            assert!(r.text.contains("assumptions     Γ = ∅"), "{rel}");
            if k == "E" {
                assert!(goal(&rel).contains(PB), "{rel}: Prime_B is written out");
            }
            n += 1;
        }
    }
    let c = "tests/rlt007s2a/accept/C.rlt";
    assert!(check_file(&root(), &root().join(c)).text.contains(VERDICT));
    assert!(goal(c).starts_with(&format!("∀p. ({PB} → ")), "{}", goal(c));
    assert!(goal(c).ends_with("∨ 212 < p)))))))"), "{}", goal(c));
    n += 1;
    let files = std::fs::read_dir(root().join("tests/rlt007s2a/accept"))
        .unwrap()
        .count();
    assert_eq!((n, files), (19, 19));
}

// Independent reconstruction of every statement from the approved intervals,
// with the primes computed here by trial division (not taken from the
// generator).
const INTERVALS: [(u32, u32); 6] = [
    (0, 40),
    (41, 78),
    (79, 112),
    (113, 156),
    (157, 190),
    (191, 212),
];

fn is_prime(n: u32) -> bool {
    n > 1 && (2..n).all(|d| !n.is_multiple_of(d))
}

fn pb(t: &str) -> String {
    format!("(1 < {t} ∧ ∀d. ((1 < d ∧ d < {t}) → ¬(d ∣ {t})))")
}

fn r(qs: &[u32]) -> String {
    let mut f = format!("p = {}", qs[qs.len() - 1]);
    for q in qs[..qs.len() - 1].iter().rev() {
        f = format!("(p = {q} ∨ {f})");
    }
    f
}

fn bconj(xs: &[String]) -> String {
    if xs.len() == 1 {
        return xs[0].clone();
    }
    let h = xs.len().div_ceil(2);
    format!("({} ∧ {})", bconj(&xs[..h]), bconj(&xs[h..]))
}

#[test]
fn every_statement_is_exactly_the_approved_one() {
    let primes: Vec<Vec<u32>> = INTERVALS
        .iter()
        .map(|&(a, b)| (a..=b).filter(|&n| is_prime(n)).collect())
        .collect();
    assert_eq!(primes.iter().map(Vec::len).sum::<usize>(), 47);
    assert_eq!(primes[0].last(), Some(&37));
    assert_eq!(primes[5], [191, 193, 197, 199, 211]);
    for (i, &(a, b)) in INTERVALS.iter().enumerate() {
        let k = i + 1;
        let non: Vec<String> = (a..=b)
            .filter(|&m| !is_prime(m))
            .map(|m| format!("¬{}", pb(&m.to_string())))
            .collect();
        assert_eq!(
            goal(&format!("tests/rlt007s2a/accept/N_{k}.rlt")),
            bconj(&non),
            "N_{k}"
        );
        let (e, o) = if a == 0 {
            (
                format!("∀p. ((p < {} ∧ {}) → {})", b + 1, pb("p"), r(&primes[i])),
                format!("∀p. (p < {} ∨ {b} < p)", b + 1),
            )
        } else {
            (
                format!(
                    "∀p. ((({} < p ∧ p < {}) ∧ {}) → {})",
                    a - 1,
                    b + 1,
                    pb("p"),
                    r(&primes[i])
                ),
                format!("∀p. ({} < p → (p < {} ∨ {b} < p))", a - 1, b + 1),
            )
        };
        assert_eq!(
            goal(&format!("tests/rlt007s2a/accept/E_{k}.rlt")),
            e,
            "E_{k}"
        );
        assert_eq!(
            goal(&format!("tests/rlt007s2a/accept/O_{k}.rlt")),
            o,
            "O_{k}"
        );
    }
    let mut inner = "212 < p".to_string();
    for qs in primes.iter().rev() {
        inner = format!("({} ∨ {inner})", r(qs));
    }
    let c = format!("∀p. ({} → {inner})", pb("p"));
    assert_eq!(goal("tests/rlt007s2a/accept/C.rlt"), c, "C");
}

#[test]
fn zero_and_one_are_refuted_in_the_first_certificate() {
    let g = goal("tests/rlt007s2a/accept/N_1.rlt");
    for m in [0, 1] {
        assert!(
            g.contains(&format!(
                "¬(1 < {m} ∧ ∀d. ((1 < d ∧ d < {m}) → ¬(d ∣ {m})))"
            )),
            "¬Prime_B({m})"
        );
    }
}

#[test]
fn citation_costs() {
    for i in 1..=6 {
        let rel = format!("tests/rlt007s2a/accept/E_{i}.rlt");
        let d = deps(&root(), &rel);
        assert_eq!((d.exit, d.checks, d.chain), (ACCEPT, 1, 1), "{rel}");
    }
    let d = deps(&root(), "tests/rlt007s2a/accept/C.rlt");
    assert_eq!(
        (d.exit, d.checks, d.chain),
        (ACCEPT, 18, 2),
        "C: 18 of 64, chain 2"
    );
}

#[test]
fn every_rejected_case_is_refused_for_its_reason() {
    let tsv = std::fs::read_to_string(root().join("tests/rlt007s2a/expected.tsv")).unwrap();
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
    assert_eq!(n, 9);
    let files = std::fs::read_dir(root().join("tests/rlt007s2a/reject"))
        .unwrap()
        .count();
    assert_eq!(files, n, "every reject file is listed");
}

#[test]
fn the_generator_reproduces_every_output_and_refuses_altered_ranges() {
    let c = generator("--check");
    assert!(c.status.success(), "{}", String::from_utf8_lossy(&c.stdout));
    let t = generator("--self-test");
    let out = String::from_utf8_lossy(&t.stdout);
    assert!(t.status.success(), "{out}");
    assert!(out.contains("8 of 8 altered range lists refused"), "{out}");
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
fn a_missing_or_modified_cited_theorem_is_never_accepted() {
    let r = root().join("target/rlt007s2a-scratch/cites");
    let _ = std::fs::remove_dir_all(&r);
    for d in ["spec", "conventions", "sources", "tests/rlt007s2a/accept"] {
        copy_dir(&root().join(d), &r.join(d));
    }
    let e = r.join("tests/rlt007s2a/accept/E_2.rlt");
    assert!(check_file(&r, &e).text.contains(VERDICT), "control");
    let n = r.join("tests/rlt007s2a/accept/N_2.rlt");
    let s = std::fs::read_to_string(&n).unwrap();
    std::fs::write(&n, s.replacen("; ", ";  ", 1)).unwrap();
    let c = check_file(&r, &e);
    assert_eq!(c.code, MALFORMED, "{}", c.text);
    assert!(
        c.text
            .contains("tests/rlt007s2a/accept/N_2.rlt has SHA-256"),
        "{}",
        c.text
    );
    std::fs::remove_file(r.join("tests/rlt007s2a/accept/O_4.rlt")).unwrap();
    let c = check_file(&r, &r.join("tests/rlt007s2a/accept/C.rlt"));
    assert_eq!(c.code, MALFORMED, "{}", c.text);
    assert!(c.text.contains("O_4.rlt`: entity not found"), "{}", c.text);
}

#[test]
fn the_seven_statement_records_are_valid_current_and_accepted() {
    let mut n = 0;
    for e in std::fs::read_dir(root().join("statements")).unwrap() {
        let f = e.unwrap().file_name().to_string_lossy().to_string();
        if !f.starts_with("PRIMES-") {
            continue;
        }
        let rel = format!("statements/{f}");
        let x = explain(&root(), &rel);
        let t = render_text(&x);
        assert_eq!(x.exit, ACCEPT, "{rel}:\n{t}");
        assert!(
            t.contains("SEMANTIC CORRESPONDENCE: partial · assessment current"),
            "{rel}:\n{t}"
        );
        n += 1;
    }
    assert_eq!(n, 7);
}

/// `rlt deps` agrees with `rlt check` on every committed proof of this stage
/// (the sweep in cli/tests/deps.rs skips this directory).
#[test]
fn deps_agrees_with_check_on_the_committed_proofs() {
    let mut n = 0;
    for d in ["tests/rlt007s2a/accept", "tests/rlt007s2a/reject"] {
        let mut fs: Vec<_> = std::fs::read_dir(root().join(d))
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        fs.sort();
        for f in fs {
            let rel = f
                .strip_prefix(root())
                .unwrap()
                .to_string_lossy()
                .to_string();
            let dd = deps(&root(), &rel);
            let rep = check_file(&root(), &f);
            let node = dd.root.as_ref().expect("a root node");
            assert_eq!(node.kernel.check_report, rep.text, "{rel}");
            assert_eq!(node.kernel.exit, rep.code, "{rel}");
            n += 1;
        }
    }
    assert_eq!(n, 28, "19 accepted and 9 rejected files, exactly");
}

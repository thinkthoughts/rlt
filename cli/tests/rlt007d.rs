//! RLT-007d: the H45 composition. The registered specifications are compared
//! with statements rebuilt here, independently of the generator and of
//! scripts/compose-check.py; the full bundle and the controls compose; every
//! negative case is refused for its stated class, exit and reason; the Python
//! checker agrees on every control and negative case; and the report never
//! describes the result as one kernel theorem.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Once;

use rlt_cli::compose::{compose, render_text, APPROVED_REGISTER};
use rlt_cli::deps::canonical;
use rlt_cli::{check_preread_outcome, sha256_hex};

const ACCEPT: i32 = 0;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn run(args: &[&str]) -> std::process::Output {
    std::process::Command::new(args[0])
        .args(&args[1..])
        .current_dir(root())
        .output()
        .expect("runs")
}

static GEN: Once = Once::new();

/// The layer-0 components cite generated certificates (RLT-006, RLT-007b
/// stages 1 and 4); materialize them as their own tests do.
fn materialized() {
    GEN.call_once(|| {
        for g in ["gen-rlt006.py", "gen-rlt007s1.py", "gen-rlt007s4.py"] {
            let o = run(&[
                "python3",
                "-I",
                &format!("scripts/{g}"),
                env!("CARGO_BIN_EXE_rlt"),
                "--materialize-only",
            ]);
            assert!(
                o.status.success(),
                "{g}: {}",
                String::from_utf8_lossy(&o.stderr)
            );
        }
    });
}

// ---- The approved statements, rebuilt here from the paper's §12. ----

/// H45 as printed in the paper (§12), written out here independently.
const H45: [u32; 45] = [
    0, 2, 12, 14, 24, 26, 30, 36, 44, 50, 54, 56, 60, 66, 72, 74, 80, 84, 92, 96, 102, 110, 114,
    116, 122, 126, 134, 140, 144, 150, 156, 162, 164, 170, 176, 180, 182, 186, 192, 194, 200, 204,
    206, 210, 212,
];
/// The paper's omitted classes for the primes 2 … 43 (§12, proof of Lemma 12.1).
const SMALL: [(u32, u32); 14] = [
    (2, 1),
    (3, 1),
    (5, 3),
    (7, 6),
    (11, 9),
    (13, 3),
    (17, 1),
    (19, 13),
    (23, 17),
    (29, 4),
    (31, 11),
    (37, 4),
    (41, 4),
    (43, 3),
];
/// The approved intervals (docs/RLT-007d-FINAL.md).
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

fn classes() -> BTreeMap<u32, u32> {
    let mut m: BTreeMap<u32, u32> = SMALL.iter().copied().collect();
    for p in (47..213).filter(|&p| is_prime(p)) {
        let r = (0..p).find(|&c| H45.iter().all(|h| h % p != c)).unwrap();
        m.insert(p, r);
    }
    m
}

/// A statement in both encodings: the kernel's s-expression and the written
/// form. The kernel checks that they agree (the probe's one step).
#[derive(Clone)]
struct F {
    s: String,
    w: String,
}

fn and(a: &F, b: &F) -> F {
    F {
        s: format!("(and {} {})", a.s, b.s),
        w: format!("({} ∧ {})", a.w, b.w),
    }
}

fn fold_and(fs: &[F]) -> F {
    fs[1..].iter().fold(fs[0].clone(), |acc, f| and(&acc, f))
}

fn fold_or_right(fs: &[F]) -> F {
    let last = fs.len() - 1;
    fs[..last].iter().rev().fold(fs[last].clone(), |acc, f| F {
        s: format!("(or {} {})", f.s, acc.s),
        w: format!("({} ∨ {})", f.w, acc.w),
    })
}

fn om(p: &str, r: &str) -> F {
    let fs: Vec<F> = H45
        .iter()
        .map(|h| F {
            s: format!("(not (exists k (= {h} (+ (* {p} k) {r}))))"),
            w: format!("¬(∃k. {h} = ({p} × k) + {r})"),
        })
        .collect();
    let f = fold_and(&fs);
    // The outermost conjunction is written without parentheses in the
    // generator's files; both spellings elaborate alike.
    f
}

fn lt(a: &str, b: &str) -> F {
    F {
        s: format!("(lt {a} {b})"),
        w: format!("{a} < {b}"),
    }
}

fn adm(p: &str) -> F {
    let o = om(p, "r");
    let l = lt("r", p);
    F {
        s: format!("(exists r (and {} {}))", l.s, o.s),
        w: format!("∃r. ({} ∧ {})", l.w, o.w),
    }
}

fn pb(p: &str) -> F {
    F {
        s: format!("(and (lt 1 {p}) (forall d (imp (and (lt 1 d) (lt d {p})) (not (dvd d {p})))))"),
        w: format!("(1 < {p} ∧ ∀d. ((1 < d ∧ d < {p}) → ¬(d ∣ {p})))"),
    }
}

fn range(i: usize, p: &str) -> F {
    let (a, b) = INTERVALS[i];
    let fs: Vec<F> = (a..=b)
        .filter(|&q| is_prime(q))
        .map(|q| F {
            s: format!("(= {p} {q})"),
            w: format!("{p} = {q}"),
        })
        .collect();
    fold_or_right(&fs)
}

fn forall_imp(h: &F, c: &F) -> F {
    F {
        s: format!("(forall p (imp {} {}))", h.s, c.s),
        w: format!("∀p. ({} → {})", h.w, c.w),
    }
}

fn approved() -> BTreeMap<String, F> {
    let mut a = BTreeMap::new();
    for (p, r) in classes() {
        a.insert(format!("om.{p}_{r}"), om(&p.to_string(), &r.to_string()));
    }
    a.insert("tail".into(), forall_imp(&lt("212", "p"), &om("p", "1")));
    let mut ds: Vec<F> = (0..6).map(|i| range(i, "p")).collect();
    ds.push(lt("212", "p"));
    a.insert("classify".into(), forall_imp(&pb("p"), &fold_or_right(&ds)));
    for (k, (lo, hi)) in [(0, 11), (11, 22), (22, 33), (33, 44)].iter().enumerate() {
        let groups: Vec<F> = (*lo..*hi)
            .map(|i| {
                let fs: Vec<F> = ((i + 1)..45)
                    .map(|j| F {
                        s: format!("(not (= {} {}))", H45[i], H45[j]),
                        w: format!("¬({} = {})", H45[i], H45[j]),
                    })
                    .collect();
                fold_and(&fs)
            })
            .collect();
        a.insert(format!("distinct.g{k}"), fold_and(&groups));
    }
    let bs: Vec<F> = H45
        .iter()
        .map(|h| F {
            s: format!("(and (exists z (= (+ z 0) {h})) (exists z (= (+ z {h}) 212)))"),
            w: format!("((∃z. z + 0 = {h}) ∧ (∃z. z + {h} = 212))"),
        })
        .collect();
    a.insert("bounds".into(), fold_and(&bs));
    for i in 0..6 {
        a.insert(
            format!("adm.range_{}", i + 1),
            forall_imp(&range(i, "p"), &adm("p")),
        );
    }
    a.insert("adm.tail".into(), forall_imp(&lt("212", "p"), &adm("p")));
    a.insert("adm.all".into(), forall_imp(&pb("p"), &adm("p")));
    a
}

/// The canonical statement the kernel reads from a probe built here.
fn rebuilt(id: &str, f: &F) -> String {
    let src = format!(
        "(rlt-proof\n  (id RLT007D.TEST)\n  (convention NAT-DEF)\n  (goal \"{w}\")\n  (assume s \"{w}\")\n  (mode conditional)\n  (axioms ROBINSON-Q)\n  (steps\n    (G {s} (hyp s))))\n",
        w = f.w,
        s = f.s
    );
    let out = check_preread_outcome(&root(), &root().join("target/rlt007d-test.rlt"), src);
    assert_eq!(
        out.report.code, ACCEPT,
        "{id}: the rebuilt statement is not accepted:\n{}",
        out.report.text
    );
    canonical(out.prop.as_ref().unwrap())
}

fn register() -> Vec<(String, String, String)> {
    std::fs::read_to_string(root().join(APPROVED_REGISTER))
        .unwrap()
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            (f[0].to_string(), f[1].to_string(), f[2].to_string())
        })
        .collect()
}

#[test]
fn the_registered_specifications_are_the_statements_rebuilt_here() {
    let a = approved();
    let reg = register();
    assert_eq!(a.len(), 62);
    assert_eq!(reg.len(), 62, "one registered probe per node");
    for (id, path, sha) in &reg {
        let f = a
            .get(id)
            .unwrap_or_else(|| panic!("{id} is not an approved node"));
        let bytes = std::fs::read(root().join(path)).unwrap();
        assert_eq!(&sha256_hex(&bytes), sha, "{path}");
        let out = check_preread_outcome(
            &root(),
            &root().join(path),
            String::from_utf8(bytes).unwrap(),
        );
        assert_eq!(out.report.code, ACCEPT, "{path}: {}", out.report.text);
        assert_eq!(
            canonical(out.prop.as_ref().unwrap()),
            rebuilt(id, f),
            "{id}: {path}"
        );
    }
    // The bundle carries exactly the registered probes.
    let b = std::fs::read_to_string(root().join("compose/h45.yaml")).unwrap();
    for (id, path, sha) in &reg {
        assert!(
            b.contains(&format!("  - id: {id}\n")),
            "{id} is a node of the bundle"
        );
        assert!(
            b.contains(&format!("      path: {path}\n      sha256: {sha}\n")),
            "{id}"
        );
    }
}

#[test]
fn a_true_but_different_statement_is_not_an_approved_one() {
    // Rebuilt here: the tail from 211 (true, stronger), the tail without the
    // bound r < p (true, weaker), range 6 without 211 (true, weaker): each
    // differs canonically from the approved statement it imitates.
    let a = approved();
    let stronger = forall_imp(&lt("211", "p"), &adm("p"));
    let o = om("p", "r");
    let weaker = forall_imp(
        &lt("212", "p"),
        &F {
            s: format!("(exists r {})", o.s),
            w: format!("∃r. {}", o.w),
        },
    );
    let fs: Vec<F> = [191, 193, 197, 199]
        .iter()
        .map(|q| F {
            s: format!("(= p {q})"),
            w: format!("p = {q}"),
        })
        .collect();
    let short = forall_imp(&fold_or_right(&fs), &adm("p"));
    let tail = rebuilt("adm.tail", &a["adm.tail"]);
    assert_ne!(rebuilt("stronger", &stronger), tail);
    assert_ne!(rebuilt("weaker", &weaker), tail);
    assert_ne!(
        rebuilt("short", &short),
        rebuilt("adm.range_6", &a["adm.range_6"])
    );
}

#[test]
fn the_full_bundle_composes_into_the_six_approved_statements() {
    materialized();
    let c = compose(&root(), "compose/h45.yaml");
    let text = render_text(&c);
    assert_eq!(c.exit, ACCEPT, "{text}");
    assert_eq!(c.checked.len(), 62);
    let edges: usize = c.checked.iter().map(|n| n.edges.len()).sum();
    assert_eq!(edges, 56);
    let a = approved();
    let result: Vec<&str> = c.result.iter().map(String::as_str).collect();
    assert_eq!(
        result,
        [
            "distinct.g0",
            "distinct.g1",
            "distinct.g2",
            "distinct.g3",
            "bounds",
            "adm.all"
        ]
    );
    for n in &c.checked {
        assert_eq!(n.statement, rebuilt(&n.id, &a[&n.id]), "{}", n.id);
    }
    assert!(text.contains("remaining       assumptions: none"), "{text}");
    assert!(
        text.contains("a composition of kernel-checked theorems, not one kernel theorem"),
        "{text}"
    );
    assert!(
        text.lines()
            .any(|l| l.starts_with("  6. adm.all")
                && l.ends_with("discharged by composition (layer 2)")),
        "{text}"
    );
}

#[test]
fn the_controls_compose_including_a_renamed_bound_variable() {
    materialized();
    for b in [
        "tests/rlt007d/accept/tail.yaml",
        "tests/rlt007d/accept/tail_renamed.yaml",
        "tests/rlt007d/accept/range_6.yaml",
    ] {
        let c = compose(&root(), b);
        let text = render_text(&c);
        assert_eq!(c.exit, ACCEPT, "{b}:\n{text}");
        // A composed node is never reported as an unconditional theorem.
        for l in text.lines().filter(|l| l.contains("adm.")) {
            assert!(!l.contains("unconditional"), "{b}: {l}");
        }
    }
    // The renamed file's assumption is written over q, the original's over p.
    let r =
        std::fs::read_to_string(root().join("tests/rlt007d/accept/ADM_TAIL_renamed.rlt")).unwrap();
    assert!(r.contains("(assume tl \"∀q. (212 < q →"));
}

fn expected() -> Vec<(String, i32, String, String)> {
    std::fs::read_to_string(root().join("tests/rlt007d/expected.tsv"))
        .unwrap()
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .map(|l| {
            let c: Vec<&str> = l.split('\t').collect();
            (
                c[0].to_string(),
                c[1].parse().unwrap(),
                c[2].to_string(),
                c[3].to_string(),
            )
        })
        .collect()
}

#[test]
fn every_negative_case_is_refused_for_its_class_and_reason() {
    materialized();
    let e = expected();
    assert_eq!(e.len(), 42);
    let mut classes = std::collections::BTreeSet::new();
    for (b, code, class, reason) in &e {
        let c = compose(&root(), b);
        let text = render_text(&c);
        assert_eq!(c.exit, *code, "{b}:\n{text}");
        let (k, m) = c.refusal.clone().unwrap_or_default();
        assert_eq!(&k, class, "{b}: {m}");
        assert!(m.contains(reason.as_str()), "{b}: want `{reason}` in `{m}`");
        assert!(
            text.contains(&format!("verdict         REFUSED ({class})")),
            "{b}"
        );
        classes.insert(k);
    }
    // Every refusal class is exercised.
    for k in rlt_cli::compose::CLASSES {
        assert!(classes.contains(*k), "no negative case for `{k}`");
    }
}

#[test]
fn the_python_checker_agrees_on_every_control_and_negative_case() {
    // rlt compose's results on the same cases are required by the two tests
    // above (each control composes; each negative gives its listed exit and
    // class), so agreement with the listed values is agreement with Rust.
    materialized();
    let mut cases: Vec<(String, i32, String)> = expected()
        .into_iter()
        .map(|(b, c, k, _)| (b, c, k))
        .collect();
    for b in ["tail", "tail_renamed", "range_6"] {
        cases.push((format!("tests/rlt007d/accept/{b}.yaml"), 0, String::new()));
    }
    for (b, code, class) in cases {
        let o = run(&[
            "python3",
            "-I",
            "scripts/compose-check.py",
            env!("CARGO_BIN_EXE_rlt"),
            &b,
            "--quiet",
        ]);
        let out = String::from_utf8_lossy(&o.stdout).to_string();
        assert_eq!(o.status.code(), Some(code), "{b}: python\n{out}");
        if code == 0 {
            assert!(out.contains("verdict COMPOSED"), "{b}: {out}");
        } else {
            assert!(
                out.contains(&format!("verdict REFUSED ({class})")),
                "{b}: {out}"
            );
        }
    }
}

#[test]
fn the_generator_reproduces_every_output() {
    let o = run(&[
        "python3",
        "-I",
        "scripts/gen-rlt007d.py",
        env!("CARGO_BIN_EXE_rlt"),
        "--check",
    ]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stdout));
}

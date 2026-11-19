//! `rlt discover` — minimal theorem discovery (UNTRUSTED generation).
//!
//! Candidate conjectures are generated from a small fixed grammar over the
//! registered signature: one universally quantified variable `x`, the
//! numerals 0, 1 and 2, and one application of `+` or `×` on each side:
//!
//!   ∀x. L = R,   L ∈ T₁ mentioning x,   R ∈ A ∪ T₁,   A = {x, 0, 1, 2},
//!   T₁ = { a ∘ b | a, b ∈ A, ∘ ∈ {+, ×} }
//!
//! The `two-level` grammar adds, for each such L,
//!
//!   ∀x. L = R,   R ∈ T₂ = { (t) ∘ a | t ∈ T₁, a ∈ A }.
//!
//! (symmetric duplicates dropped). Each candidate goes through `rlt prove`
//! with the stated bounds. A candidate is stored as a theorem only when its
//! certificate was replayed and accepted by the RLT kernel; every other
//! candidate is listed as NOT_FOUND_WITHIN_BOUNDS, which is a statement about
//! the search and never about the candidate's truth.

use crate::prove::{prove_source, Bounds, Verdict};
use crate::{check_file_outcome, sha256_hex, ACCEPT};
use std::fmt::Write as _;
use std::path::Path;

/// The discovery grammar, in a fixed order.
pub fn candidates(two_level: bool) -> Vec<String> {
    let atoms = ["x", "0", "1", "2"];
    let mut t1 = Vec::new();
    for op in ["+", "×"] {
        for a in atoms {
            for b in atoms {
                t1.push(format!("{a} {op} {b}"));
            }
        }
    }
    let has_x = |s: &str| s.split(' ').any(|w| w == "x");
    let mut t2 = Vec::new();
    for op in ["+", "×"] {
        for t in &t1 {
            for a in atoms {
                t2.push(format!("({t}) {op} {a}"));
            }
        }
    }
    let mut out = Vec::new();
    for (i, l) in t1.iter().enumerate() {
        if !has_x(l) {
            continue;
        }
        for r in atoms {
            out.push(format!("∀x. {l} = {r}"));
        }
        for (j, r) in t1.iter().enumerate() {
            // One orientation per unordered pair; right sides without x always.
            if j == i || (has_x(r) && j > i) {
                continue;
            }
            out.push(format!("∀x. {l} = {r}"));
        }
        for r in t2.iter().filter(|_| two_level) {
            out.push(format!("∀x. {l} = {r}"));
        }
    }
    out
}

pub const AXIOM_SET: &str = "ROBINSON-Q";

/// The bounds every candidate is searched with.
pub fn discovery_bounds() -> Bounds {
    Bounds {
        depth: 1,
        rounds: 32,
        nodes: 2000,
        witnesses: 4,
        budget: 200,
    }
}

fn goal_source(id: &str, statement: &str, b: &Bounds) -> String {
    format!(
        "(rlt-goal\n  (id {id})\n  (convention NAT-LOGIC)\n  (goal \"{statement}\")\n  (mode unconditional)\n  (axioms {AXIOM_SET})\n  (bounds (depth {}) (rounds {}) (nodes {}) (witnesses {}) (budget {})))\n",
        b.depth, b.rounds, b.nodes, b.witnesses, b.budget
    )
}

/// Run discovery, writing certificates and a manifest under `out_dir`
/// (repository-relative). Returns the summary text and an exit code.
pub fn discover(
    root: &Path,
    out_dir: &str,
    two_level: bool,
    limit: Option<usize>,
) -> (String, i32) {
    let dir = root.join(out_dir);
    if let Err(e) = std::fs::create_dir_all(&dir) {
        return (
            format!("rlt discover: cannot create {out_dir}: {}\n", e.kind()),
            2,
        );
    }
    let b = discovery_bounds();
    let mut cands = candidates(two_level);
    if let Some(n) = limit {
        cands.truncate(n);
    }
    let mut manifest = String::from(
        "# RLT theorem discovery manifest. Every row is a kernel-replayed certificate.\n# id\tstatement\taxiom_set\tmode\tassumptions\tcertificate\tsha256\tsteps\tdepth\taxioms_used\n",
    );
    let mut not_found = String::from(
        "# Candidates without a proof within the discovery bounds. NOT_FOUND_WITHIN_BOUNDS\n# says nothing about truth: some are true in ℕ but need induction, some are false.\n",
    );
    let (mut proved, mut nf, mut err) = (0usize, 0usize, 0usize);
    for (i, st) in cands.iter().enumerate() {
        let id = format!("D{:03}", i + 1);
        let src = goal_source(&id, st, &b);
        let cert = format!("{out_dir}/{id}.rlt");
        let r = prove_source(root, &src, &[], Some(Path::new(&cert)));
        match &r.verdict {
            Verdict::Proved {
                sha256,
                theorem,
                steps,
                ..
            } => {
                proved += 1;
                let ax: Vec<&str> = theorem.axioms_used().keys().map(|s| s.as_str()).collect();
                let _ =
                    writeln!(
                    manifest,
                    "{id}\t{st}\t{AXIOM_SET}\tunconditional\t∅\t{cert}\t{sha256}\t{steps}\t{}\t{}",
                    r.depth_found.unwrap_or(0),
                    if ax.is_empty() { "none".to_string() } else { ax.join(",") }
                );
            }
            Verdict::NotFound => {
                nf += 1;
                let _ = writeln!(not_found, "{id}\t{st}\tNOT_FOUND_WITHIN_BOUNDS");
            }
            Verdict::Error(e) => {
                err += 1;
                let _ = writeln!(not_found, "{id}\t{st}\tERROR {}", e.replace('\n', " "));
            }
        }
    }
    let summary = format!(
        "RLT discover    {out_dir}\ngrammar         {} — ∀x. L = R over x, 0, 1, 2 ({} candidates)\naxiom set       {AXIOM_SET}\nbounds          {}\nproved          {proved} (certificates replayed and accepted by the kernel; listed in manifest.tsv)\nnot found       {nf} (listed in not-found.tsv; no claim about their truth)\nerrors          {err}\n",
        if two_level { "two-level (one operation in L, up to two in R)" } else { "one-level (one operation in L; at most one in R)" },
        cands.len(),
        b.describe()
    );
    for (name, body) in [
        ("manifest.tsv", &manifest),
        ("not-found.tsv", &not_found),
        ("summary.txt", &summary),
    ] {
        if let Err(e) = std::fs::write(dir.join(name), body) {
            return (
                format!("rlt discover: cannot write {name}: {}\n", e.kind()),
                2,
            );
        }
    }
    (summary, if err == 0 { 0 } else { 2 })
}

/// `rlt discover --verify DIR`: every manifest row is re-hashed and its
/// certificate re-checked from scratch by `rlt check`'s function.
pub fn verify(root: &Path, out_dir: &str) -> (String, i32) {
    let mut t = format!("RLT discover --verify {out_dir}\n");
    let text = match std::fs::read_to_string(root.join(out_dir).join("manifest.tsv")) {
        Ok(s) => s,
        Err(e) => {
            return (
                format!("{t}verdict  ERROR — manifest.tsv: {}\n", e.kind()),
                2,
            )
        }
    };
    let (mut ok, mut bad) = (0, 0);
    for line in text
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
    {
        let cols: Vec<&str> = line.split('\t').collect();
        let [id, statement, _, _, _, cert, sha, ..] = cols.as_slice() else {
            bad += 1;
            let _ = writeln!(t, "malformed row: {line}");
            continue;
        };
        let path = root.join(cert);
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) => {
                bad += 1;
                let _ = writeln!(t, "{id}  MISSING {cert}: {}", e.kind());
                continue;
            }
        };
        if sha256_hex(&bytes) != *sha {
            bad += 1;
            let _ = writeln!(t, "{id}  STALE  {cert}: SHA-256 differs from the manifest");
            continue;
        }
        let out = check_file_outcome(root, &path);
        let goal_matches = out
            .proof
            .as_ref()
            .is_some_and(|p| p.goal.text == *statement);
        if out.report.code == ACCEPT && goal_matches {
            ok += 1;
            let _ = writeln!(t, "{id}  ACCEPT {statement}");
        } else {
            bad += 1;
            let _ = writeln!(t, "{id}  REFUSED {statement}");
        }
    }
    let _ = writeln!(t, "verified        {ok} accepted, {bad} refused or stale");
    (t, if bad == 0 { 0 } else { 1 })
}

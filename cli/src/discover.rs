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
use crate::{sha256_hex, ACCEPT};
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

/// The discovery directory's own files and the first bytes each begins with.
pub const OUTPUT_MARKERS: [(&str, &str); 3] = [
    ("manifest.tsv", "# RLT theorem discovery manifest."),
    (
        "not-found.tsv",
        "# Candidates without a proof within the discovery bounds.",
    ),
    ("summary.txt", "RLT discover "),
];

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
    // Audit K8: only files an earlier discovery run wrote are replaced;
    // checked before any certificate is written.
    for (name, marker) in OUTPUT_MARKERS {
        if let Err(e) = crate::prove::may_overwrite(&dir.join(name), marker) {
            return (format!("rlt discover: {e}\n"), 2);
        }
    }
    let b = discovery_bounds();
    let mut cands = candidates(two_level);
    if let Some(n) = limit {
        cands.truncate(n);
    }
    let mut manifest = format!(
        "# RLT theorem discovery manifest. Every row is a kernel-replayed certificate.\n# {MANIFEST_COLUMNS}\n"
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
                let _ = writeln!(
                    manifest,
                    "{id}\t{st}\t{AXIOM_SET}\tunconditional\t∅\t{cert}\t{sha256}\t{steps}\t{}\t{}",
                    r.depth_found.unwrap_or(0),
                    axioms_column(theorem)
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

/// The manifest's column header, exactly as `discover` writes it.
pub const MANIFEST_COLUMNS: &str =
    "id\tstatement\taxiom_set\tmode\tassumptions\tcertificate\tsha256\tsteps\tdepth\taxioms_used";

/// `axioms_used` as the manifest spells it: the kernel's names, joined by
/// commas in the kernel's (sorted) order, or `none`.
pub fn axioms_column(th: &rlt_kernel::Theorem) -> String {
    let ax: Vec<&str> = th.axioms_used().keys().map(|s| s.as_str()).collect();
    if ax.is_empty() {
        "none".to_string()
    } else {
        ax.join(",")
    }
}

/// Check one manifest row against the certificate it names, replayed by the
/// kernel. Every column is bound to the replayed proof; any disagreement is
/// returned as the reason for refusal. (The ACCEPT of the kernel alone is
/// not enough: a valid certificate can still be mislabelled.)
fn check_row(root: &Path, out_dir: &str, cols: &[&str]) -> Result<(), String> {
    let [id, statement, axiom_set, mode, assumptions, cert, sha, steps, depth, axioms_used] = cols
    else {
        return Err(format!(
            "expected 10 tab-separated columns, found {}",
            cols.len()
        ));
    };
    // The id names the file: the certificate must be exactly DIR/ID.rlt.
    if !is_discovery_id(id) {
        return Err(format!("id `{id}` is not a discovery id"));
    }
    let expected = format!("{out_dir}/{id}.rlt");
    if *cert != expected {
        return Err(format!(
            "certificate path `{cert}` must be `{expected}` (inside the discovery directory, named by its id)"
        ));
    }
    // Plain repository-relative components, no symlink anywhere, a regular file.
    let path = crate::citation_path(root, cert)?;
    let bytes = std::fs::read(&path).map_err(|e| format!("MISSING {cert}: {}", e.kind()))?;
    if sha256_hex(&bytes) != *sha {
        return Err(format!("STALE {cert}: SHA-256 differs from the manifest"));
    }
    // Replay exactly the bytes that were hashed.
    let out = crate::check_bytes_outcome(root, &path, bytes);
    if out.report.code != ACCEPT {
        return Err("the kernel did not accept the certificate".into());
    }
    let (Some(pf), Some(th)) = (out.proof.as_ref(), out.theorem.as_ref()) else {
        return Err("internal: replay accepted without a proof and theorem".into());
    };
    let mut wrong = Vec::new();
    let mut want = |what: &str, claimed: &str, actual: &str| {
        if claimed != actual {
            wrong.push(format!(
                "{what}: manifest says `{claimed}`, certificate has `{actual}`"
            ));
        }
    };
    want("id", id, &pf.id);
    want("statement", statement, &pf.goal.text);
    want("axiom set", axiom_set, &pf.axiom_set);
    want(
        "mode",
        mode,
        match pf.mode {
            rlt_kernel::Mode::Unconditional => "unconditional",
            rlt_kernel::Mode::Conditional => "conditional",
        },
    );
    let declared: Vec<&str> = pf.assumptions.iter().map(|(n, _)| n.as_str()).collect();
    want(
        "assumptions",
        assumptions,
        &if declared.is_empty() {
            "∅".to_string()
        } else {
            declared.join(",")
        },
    );
    want("steps", steps, &pf.steps.len().to_string());
    want("axioms used", axioms_used, &axioms_column(th));
    // The current discovery workflow: unconditional ROBINSON-Q theorems with
    // Γ = ∅, convention NAT-LOGIC, no citations.
    if pf.axiom_set != AXIOM_SET {
        wrong.push(format!("axiom set must be {AXIOM_SET}"));
    }
    if pf.mode != rlt_kernel::Mode::Unconditional || !pf.assumptions.is_empty() {
        wrong
            .push("discovery stores only unconditional proofs with no declared assumptions".into());
    }
    if !th.is_unconditional() || !th.assumptions().is_empty() {
        wrong.push("the replayed theorem depends on assumptions".into());
    }
    if pf.convention != "NAT-LOGIC" {
        wrong.push(format!(
            "convention must be NAT-LOGIC, not {}",
            pf.convention
        ));
    }
    if !pf.cites.is_empty() {
        wrong.push("discovery certificates cite nothing".into());
    }
    // The search depth is not recorded in the certificate; it must at least
    // be a number within the discovery bound.
    match depth.parse::<usize>() {
        Ok(d) if d <= discovery_bounds().depth && d.to_string() == *depth => {}
        _ => wrong.push(format!(
            "depth `{depth}` is not a number within the discovery bound {}",
            discovery_bounds().depth
        )),
    }
    if wrong.is_empty() {
        Ok(())
    } else {
        Err(wrong.join("; "))
    }
}

/// `D` followed by decimal digits.
pub fn is_discovery_id(id: &str) -> bool {
    id.strip_prefix('D')
        .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

/// `rlt discover --verify DIR`: every manifest row is re-hashed, its
/// certificate replayed by `rlt check`'s function from the hashed bytes, and
/// every column checked against the replayed proof. Duplicate rows, ids,
/// statements and certificate paths are refused.
pub fn verify(root: &Path, out_dir: &str) -> (String, i32) {
    let mut t = format!("RLT discover --verify {out_dir}\n");
    let out_dir = out_dir.trim_end_matches('/');
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
    if !text
        .lines()
        .any(|l| l.strip_prefix("# ") == Some(MANIFEST_COLUMNS))
    {
        bad += 1;
        let _ = writeln!(
            t,
            "manifest     REFUSED — the column header `# {MANIFEST_COLUMNS}` is missing"
        );
    }
    let mut seen_ids = std::collections::BTreeSet::new();
    let mut seen_statements = std::collections::BTreeSet::new();
    let mut seen_certs = std::collections::BTreeSet::new();
    for line in text
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
    {
        let cols: Vec<&str> = line.split('\t').collect();
        let id = cols.first().copied().unwrap_or("");
        let statement = cols.get(1).copied().unwrap_or("");
        let mut dup = Vec::new();
        if !seen_ids.insert(id.to_string()) {
            dup.push("id");
        }
        if !seen_statements.insert(statement.to_string()) {
            dup.push("statement");
        }
        if let Some(c) = cols.get(5) {
            if !seen_certs.insert(c.to_string()) {
                dup.push("certificate");
            }
        }
        let res = if dup.is_empty() {
            check_row(root, out_dir, &cols)
        } else {
            Err(format!("duplicate {} in the manifest", dup.join(", ")))
        };
        match res {
            Ok(()) => {
                ok += 1;
                let _ = writeln!(t, "{id}  ACCEPT {statement}");
            }
            Err(e) => {
                bad += 1;
                let _ = writeln!(t, "{id}  REFUSED {statement} — {e}");
            }
        }
    }
    let _ = writeln!(t, "verified        {ok} accepted, {bad} refused or stale");
    (t, if bad == 0 { 0 } else { 1 })
}

//! `rlt discover --definitions FILE DIR` — definition-aware theorem discovery
//! (RLT-PROVER-002; UNTRUSTED generation).
//!
//! Candidates come from a fixed grammar over the unary functions registered
//! in FILE (in file order), the variable x and the numerals 0, 1, 2:
//!
//!   L = R,   A = {x, 0, 1, 2},   U = { f(a) | f registered, a ∈ A },
//!   L ∈ U ∪ { u ∘ a | u ∈ U, a ∈ A, ∘ ∈ {+, ×} },
//!   R ∈ A ∪ { a ∘ b | a, b ∈ A } ∪ U   (one orientation of U = U),
//!
//! closed with ∀x when x occurs. Every candidate uses a registered function.
//! Each goes through `rlt prove` under convention NAT-FUN, bound to FILE's
//! exact SHA-256, with the discovery bounds; it is stored only when its
//! certificate was replayed and accepted by the kernel. Every other
//! candidate is NOT_FOUND_WITHIN_BOUNDS, a statement about the search only.
//!
//! Each theorem is classified from its elaborated statement and checked
//! against the kernel's report:
//! - `unfolding`: the two sides are identical once the definitions are
//!   unfolded; the kernel proof uses no axiom;
//! - `inference`: the sides differ after unfolding; the kernel proof uses at
//!   least one registered axiom.

use crate::defreg::{Authority, Registration};
use crate::discover::{axioms_column, discovery_bounds, only_generated_comments, AXIOM_SET};
use crate::prove::{prove_source_as, Verdict};
use crate::{functions_column, load_function_definitions_as, sha256_hex, ACCEPT};
use rlt_kernel::Prop;
use rlt_syntax::fdefs::FunctionDefs;
use rlt_syntax::proof::DefinitionsRef;
use std::fmt::Write as _;
use std::path::Path;

pub const MANIFEST_MARKER: &str = "# RLT definition-aware discovery manifest.";
pub const MANIFEST_COLUMNS: &str = "id\tstatement\telaborated\tdefinitions\taxiom_set\tmode\tassumptions\tcertificate\tsha256\tsteps\taxioms_used\tkind\tdepth";
const GRAMMAR: &str = "definition-aware";
pub const NOT_FOUND_HEADER: &str = "# Definition-aware candidates without a proof within the discovery bounds.\n# NOT_FOUND_WITHIN_BOUNDS says nothing about truth: some are true in ℕ but need induction, some are false.\n";
const OUTPUT_MARKERS: [(&str, &str); 3] = [
    ("manifest.tsv", MANIFEST_MARKER),
    ("not-found.tsv", "# Definition-aware candidates"),
    ("summary.txt", "RLT discover "),
];

/// The unary functions the grammar uses, in file order.
fn unary(d: &FunctionDefs) -> Vec<String> {
    d.list
        .iter()
        .filter(|f| f.params.len() == 1)
        .map(|f| f.name.clone())
        .collect()
}

/// The candidate statements, in a fixed order.
pub fn candidates(d: &FunctionDefs) -> Vec<String> {
    let atoms = ["x", "0", "1", "2"];
    let mut uses = Vec::new();
    for f in unary(d) {
        for a in atoms {
            uses.push(format!("{f}({a})"));
        }
    }
    let mut lefts: Vec<String> = uses.clone();
    for op in ["+", "×"] {
        for u in &uses {
            for a in atoms {
                lefts.push(format!("{u} {op} {a}"));
            }
        }
    }
    let mut rights: Vec<String> = atoms.iter().map(|a| a.to_string()).collect();
    for op in ["+", "×"] {
        for a in atoms {
            for b in atoms {
                rights.push(format!("{a} {op} {b}"));
            }
        }
    }
    let mut out = Vec::new();
    for (i, l) in lefts.iter().enumerate() {
        let mut rs: Vec<&String> = rights.iter().collect();
        // A use on the right: one orientation per pair of uses.
        for (j, u) in uses.iter().enumerate() {
            if i < uses.len() && j <= i {
                continue;
            }
            rs.push(u);
        }
        for r in rs {
            let st = format!("{l} = {r}");
            let has_x = st
                .split(|c: char| !c.is_ascii_alphanumeric())
                .any(|w| w == "x");
            out.push(if has_x { format!("∀x. {st}") } else { st });
        }
    }
    out
}

pub fn discovery_id(i: usize) -> String {
    format!("DF{:04}", i + 1)
}

fn index_of(id: &str) -> Option<usize> {
    let n = id.strip_prefix("DF")?;
    if n.len() != 4 || !n.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let i: usize = n.parse().ok()?;
    i.checked_sub(1)
}

fn goal_source(id: &str, st: &str, file: &str, sha: &str) -> String {
    let b = discovery_bounds();
    format!(
        "(rlt-goal\n  (id {id})\n  (convention NAT-FUN)\n  (definitions \"{file}\" \"{sha}\")\n  (goal \"{st}\")\n  (mode unconditional)\n  (axioms {AXIOM_SET})\n  (bounds (depth {}) (rounds {}) (nodes {}) (witnesses {}) (budget {})))\n",
        b.depth, b.rounds, b.nodes, b.witnesses, b.budget
    )
}

fn manifest_head(d: &FunctionDefs) -> String {
    format!(
        "{MANIFEST_MARKER} Every row is a kernel-replayed certificate.\n# definitions {} sha256 {} (registry {})\n# bound to the replay: every column except depth; search metadata, not checked by the kernel: depth\n# {MANIFEST_COLUMNS}\n",
        d.file, d.file_sha256, d.id
    )
}

fn summary_text(
    out_dir: &str,
    d: &FunctionDefs,
    n: usize,
    unfolding: usize,
    inference: usize,
    not_found: usize,
    errors: usize,
) -> String {
    format!(
        "RLT discover    {out_dir}\ngrammar         {GRAMMAR} (functions {} of {}) — L = R over x, 0, 1, 2 ({n} candidates)\ndefinitions     {} sha256 {}\naxiom set       {AXIOM_SET}\nbounds          {}\nproved          {} (certificates replayed and accepted by the kernel; listed in manifest.tsv): {unfolding} by unfolding alone, {inference} by inference from the axioms\nnot found       {not_found} (listed in not-found.tsv; no claim about their truth)\nerrors          {errors}\n",
        unary(d).join(", "),
        d.id,
        d.file,
        d.file_sha256,
        discovery_bounds().describe(),
        unfolding + inference
    )
}

/// Elaborate a candidate under NAT-FUN with these definitions: the folded
/// and unfolded formulas and the functions used (with dependencies).
fn elaborate(
    root: &Path,
    d: &FunctionDefs,
    statement: &str,
) -> Result<(Prop, Prop, Vec<String>), String> {
    let convs = crate::pinned_conventions(root)?;
    let mut c = convs
        .get(rlt_syntax::fdefs::FUNCTIONS_CONVENTION)
        .ok_or("convention NAT-FUN is unregistered")?
        .convention
        .clone();
    let fresh = std::rc::Rc::new(FunctionDefs {
        used: Default::default(),
        ..d.clone()
    });
    c.fdefs = Some(fresh.clone());
    let w = rlt_syntax::written::Written::new(statement);
    let folded = rlt_syntax::elab::elaborate_prop_folded(&w, &c)?;
    let unfolded = rlt_syntax::elab::elaborate_prop(&w, &c)?;
    Ok((folded, unfolded, fresh.used_in_order()))
}

/// `unfolding` when the equation's sides are identical after unfolding,
/// `inference` otherwise (the grammar's statements are equations under ∀).
fn kind_of(p: &Prop) -> &'static str {
    let mut q = p;
    while let Prop::Forall(_, b) = q {
        q = b;
    }
    match q {
        Prop::Eq(f) if f.lhs == f.rhs => "unfolding",
        _ => "inference",
    }
}

fn load(
    root: &Path,
    file: &str,
    sha: &str,
    authority: Authority,
) -> Result<(FunctionDefs, Registration), String> {
    load_function_definitions_as(
        root,
        &DefinitionsRef {
            path: file.to_string(),
            sha256: sha.to_string(),
        },
        authority,
    )
}

/// Run definition-aware discovery over the functions registered in `file`.
pub fn discover(root: &Path, file: &str, out_dir: &str, limit: Option<usize>) -> (String, i32) {
    discover_as(root, file, out_dir, limit, Authority::RegisteredOnly)
}

/// As `discover`, with the caller's authority (RLT-DEF-001a). Over an
/// unregistered file (`--unregistered-definitions`) the run is exploratory:
/// its records say so and `rlt discover --verify` refuses them, since that
/// check re-decides registration from the definitions bytes.
pub fn discover_as(
    root: &Path,
    file: &str,
    out_dir: &str,
    limit: Option<usize>,
    authority: Authority,
) -> (String, i32) {
    let fail = |m: String| (format!("rlt discover: {m}\n"), 2);
    let path = match crate::citation_path(root, file) {
        Ok(p) => p,
        Err(e) => return fail(format!("definitions: {e}")),
    };
    let sha = match std::fs::read(&path) {
        Ok(b) => sha256_hex(&b),
        Err(e) => return fail(format!("definitions: {file}: {}", e.kind())),
    };
    let (d, registration) = match load(root, file, &sha, authority) {
        Ok(x) => x,
        Err(e) => return fail(e),
    };
    let exploratory = match &registration {
        Registration::Unregistered(why) => Some(why.clone()),
        Registration::Registered(_) => None,
    };
    if unary(&d).is_empty() {
        return fail(format!(
            "{file} registers no unary function to discover with"
        ));
    }
    let dir = root.join(out_dir);
    if let Err(e) = std::fs::create_dir_all(&dir) {
        return fail(format!("cannot create {out_dir}: {}", e.kind()));
    }
    // As for `rlt discover`: never write where committed files are, and
    // replace only files an earlier run of this command wrote.
    match crate::prove::tracked_by_git(root, &dir) {
        Ok(false) => {}
        Ok(true) => {
            return fail(format!("{out_dir} holds files tracked by the repository; refusing to write there (discover into an untracked directory and compare; `rlt discover --verify {out_dir}` checks a committed record)"))
        }
        Err(e) => return fail(e),
    }
    for (name, marker) in OUTPUT_MARKERS {
        if let Err(e) = crate::prove::may_overwrite(&dir.join(name), marker) {
            return fail(e);
        }
    }
    match stale_certificates(&dir) {
        Ok(old) => {
            for p in old {
                if let Err(e) = std::fs::remove_file(&p) {
                    return fail(format!("cannot remove {}: {}", p.display(), e.kind()));
                }
            }
        }
        Err(e) => return fail(e),
    }
    let mut cands = candidates(&d);
    if let Some(n) = limit {
        cands.truncate(n);
    }
    let mut manifest = manifest_head(&d);
    if exploratory.is_some() {
        manifest.insert_str(
            MANIFEST_MARKER.len() + " Every row is a kernel-replayed certificate.\n".len(),
            "# EXPLORATORY: unregistered definitions; not an authoritative record (rlt discover --verify refuses it).\n",
        );
    }
    let mut not_found = String::from(NOT_FOUND_HEADER);
    let (mut unf, mut inf, mut nf, mut err) = (0usize, 0usize, 0usize, 0usize);
    for (i, st) in cands.iter().enumerate() {
        let id = discovery_id(i);
        let src = goal_source(&id, st, &d.file, &d.file_sha256);
        let cert = format!("{out_dir}/{id}.rlt");
        let r = prove_source_as(root, &src, &[], Some(Path::new(&cert)), authority);
        match &r.verdict {
            Verdict::Proved {
                sha256,
                theorem,
                steps,
                ..
            } => {
                let (_, unfolded, used) = match elaborate(root, &d, st) {
                    Ok(x) => x,
                    Err(e) => {
                        err += 1;
                        let _ = writeln!(not_found, "{id}\t{st}\tERROR {e}");
                        continue;
                    }
                };
                let kind = kind_of(&unfolded);
                if kind == "unfolding" {
                    unf += 1
                } else {
                    inf += 1
                }
                let _ = writeln!(
                    manifest,
                    "{id}\t{st}\t{unfolded}\t{}\t{AXIOM_SET}\tunconditional\t∅\t{cert}\t{sha256}\t{steps}\t{}\t{kind}\t{}",
                    functions_column(&d, &used),
                    axioms_column(theorem),
                    r.depth_found.unwrap_or(0)
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
    let mut summary = summary_text(out_dir, &d, cands.len(), unf, inf, nf, err);
    if let Some(why) = &exploratory {
        let _ = writeln!(summary, "authority       EXPLORATORY — {why}; this record carries no authority and `rlt discover --verify` refuses it");
    }
    for (name, body) in [
        ("manifest.tsv", &manifest),
        ("not-found.tsv", &not_found),
        ("summary.txt", &summary),
    ] {
        if let Err(e) = std::fs::write(dir.join(name), body) {
            return fail(format!("cannot write {name}: {}", e.kind()));
        }
    }
    let code = match (err, &exploratory) {
        (0, None) => 0,
        (0, Some(_)) => crate::EXPLORATORY,
        _ => 2,
    };
    (summary, code)
}

/// Certificates an earlier run left: regular files `DF####.rlt` beginning
/// with the `rlt prove` header.
fn stale_certificates(dir: &Path) -> Result<Vec<std::path::PathBuf>, String> {
    let mut out = Vec::new();
    let rd = std::fs::read_dir(dir)
        .map_err(|e| format!("cannot list {}: {}", dir.display(), e.kind()))?;
    for e in rd {
        let e = e.map_err(|e| format!("cannot list {}: {}", dir.display(), e.kind()))?;
        let name = e.file_name().to_string_lossy().to_string();
        let Some(stem) = name.strip_suffix(".rlt") else {
            continue;
        };
        if index_of(stem).is_none() || !e.file_type().is_ok_and(|t| t.is_file()) {
            continue;
        }
        if std::fs::read(e.path())
            .is_ok_and(|b| b.starts_with(crate::prove::CERT_HEADER.as_bytes()))
        {
            out.push(e.path());
        }
    }
    out.sort();
    Ok(out)
}

/// Whether DIR holds a definition-aware record (its manifest's marker).
pub fn is_record(root: &Path, out_dir: &str) -> bool {
    std::fs::read_to_string(root.join(out_dir).join("manifest.tsv"))
        .is_ok_and(|m| m.starts_with(MANIFEST_MARKER))
}

/// The definitions file and hash a record states in its summary.
fn summary_definitions(summary: &str) -> Option<(String, String)> {
    let line = summary.lines().nth(2)?.strip_prefix("definitions     ")?;
    let (file, sha) = line.split_once(" sha256 ")?;
    Some((file.to_string(), sha.to_string()))
}

/// Check one manifest row against its replayed certificate.
fn check_row(
    root: &Path,
    out_dir: &str,
    d: &FunctionDefs,
    all: &[String],
    cols: &[&str],
) -> Result<(), String> {
    let [id, statement, elaborated, defs_col, axiom_set, mode, assumptions, cert, sha, steps, axioms_used, kind, depth] =
        cols
    else {
        return Err(format!(
            "expected 13 tab-separated columns, found {}",
            cols.len()
        ));
    };
    let Some(i) = index_of(id).filter(|&i| i < all.len()) else {
        return Err(format!("id `{id}` is not one of this grammar's candidates"));
    };
    if all[i] != *statement {
        return Err(format!(
            "the id {id} names the candidate `{}`, not `{statement}`",
            all[i]
        ));
    }
    let expected = format!("{out_dir}/{id}.rlt");
    if *cert != expected {
        return Err(format!("certificate path `{cert}` must be `{expected}`"));
    }
    let path = crate::citation_path(root, cert)?;
    let bytes = std::fs::read(&path).map_err(|e| format!("MISSING {cert}: {}", e.kind()))?;
    if sha256_hex(&bytes) != *sha {
        return Err(format!("STALE {cert}: SHA-256 differs from the manifest"));
    }
    let text =
        String::from_utf8(bytes.clone()).map_err(|_| "the certificate is not UTF-8".to_string())?;
    let out = crate::check_bytes_outcome(root, &path, bytes);
    if out.report.code != ACCEPT {
        return Err("the kernel did not accept the certificate".into());
    }
    let (Some(pf), Some(th), Some(goal)) =
        (out.proof.as_ref(), out.theorem.as_ref(), out.prop.as_ref())
    else {
        return Err("internal: replay accepted without a proof, theorem and goal".into());
    };
    let mut wrong = Vec::new();
    // The header is exactly the generated goal for this id: the definitions
    // file and hash included, nothing further asserted.
    let gen = crate::prove::parse_goal_file(&goal_source(id, statement, &d.file, &d.file_sha256))?;
    if let Err(e) = exact_header(&gen.fields, &text) {
        wrong.push(e);
    }
    if let Err(e) = only_generated_comments(&text) {
        wrong.push(e);
    }
    let (_, unfolded, used) = elaborate(root, d, statement)?;
    let mut want = |what: &str, claimed: &str, actual: &str| {
        if claimed != actual {
            wrong.push(format!(
                "{what}: manifest says `{claimed}`, the replay gives `{actual}`"
            ));
        }
    };
    want("statement", statement, &pf.goal.text);
    want("elaborated", elaborated, &goal.to_string());
    want("elaborated (recomputed)", elaborated, &unfolded.to_string());
    want("definitions", defs_col, &functions_column(d, &used));
    want("axiom set", axiom_set, &pf.axiom_set);
    want(
        "mode",
        mode,
        match pf.mode {
            rlt_kernel::Mode::Unconditional => "unconditional",
            rlt_kernel::Mode::Conditional => "conditional",
        },
    );
    want(
        "assumptions",
        assumptions,
        if pf.assumptions.is_empty() {
            "∅"
        } else {
            "declared"
        },
    );
    want("steps", steps, &pf.steps.len().to_string());
    let ax = axioms_column(th);
    want("axioms used", axioms_used, &ax);
    let k = kind_of(goal);
    want("kind", kind, k);
    // The classification agrees with the kernel: unfolding alone uses no
    // axiom, inference uses at least one.
    if (k == "unfolding") != (ax == "none") {
        wrong.push(format!(
            "kind {k} disagrees with the kernel's axioms used ({ax})"
        ));
    }
    if !th.is_unconditional() || !th.assumptions().is_empty() {
        wrong.push("the replayed theorem depends on assumptions".into());
    }
    if pf.convention != rlt_syntax::fdefs::FUNCTIONS_CONVENTION || !pf.cites.is_empty() {
        wrong.push("discovery certificates read NAT-FUN and cite nothing".into());
    }
    match depth.parse::<usize>() {
        Ok(x) if x <= discovery_bounds().depth && x.to_string() == *depth => {}
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

fn exact_header(expected: &[rlt_syntax::sexpr::Sexp], cert: &str) -> Result<(), String> {
    let top = rlt_syntax::sexpr::parse_all(cert).map_err(|e| format!("certificate: {e}"))?;
    let [form] = top.as_slice() else {
        return Err("certificate: expected exactly one (rlt-proof …) form".into());
    };
    let Some(("rlt-proof", fields)) = form.head() else {
        return Err("certificate: expected (rlt-proof …)".into());
    };
    let (mut params, mut steps) = (0, 0);
    let mut header = Vec::new();
    for f in fields {
        match f.head() {
            Some(("params", _)) => params += 1,
            Some(("steps", _)) => steps += 1,
            _ => header.push(f),
        }
    }
    if steps != 1 || params > 1 {
        return Err("header: one (steps …) and at most one (params …)".into());
    }
    let exp: Vec<&rlt_syntax::sexpr::Sexp> = expected.iter().collect();
    if header != exp {
        let extra: Vec<String> = header
            .iter()
            .filter(|f| !exp.contains(f))
            .map(|f| f.head().map_or("?".to_string(), |(k, _)| k.to_string()))
            .collect();
        return Err(if extra.is_empty() {
            "header: the fields differ from the generated goal (definitions, order, repetition or values)".to_string()
        } else {
            format!(
                "header: fields not generated by discovery: {} (no reading point, relationship or context can be asserted by discovery)",
                extra.join(", ")
            )
        });
    }
    Ok(())
}

/// `rlt discover --verify DIR` for a definition-aware record.
pub fn verify(root: &Path, out_dir: &str) -> (String, i32) {
    let out_dir = out_dir.trim_end_matches('/');
    let mut t = format!("RLT discover --verify {out_dir}\n");
    let read = |n: &str| std::fs::read_to_string(root.join(out_dir).join(n));
    let (manifest, summary, nf_text) = match (
        read("manifest.tsv"),
        read("summary.txt"),
        read("not-found.tsv"),
    ) {
        (Ok(a), Ok(b), Ok(c)) => (a, b, c),
        _ => {
            return (
                format!(
                "{t}verdict  ERROR — manifest.tsv, summary.txt and not-found.tsv are required\n"
            ),
                2,
            )
        }
    };
    let Some((file, sha)) = summary_definitions(&summary) else {
        return (format!("{t}record  REFUSED — summary.txt names no definitions file\nverified        0 accepted, 1 refused or stale\n"), 1);
    };
    // The definitions the record was discovered with, at that exact hash.
    // RLT-DEF-001a: and registered, decided here from the definitions bytes
    // (never from the record's own words).
    let d = match load(root, &file, &sha, Authority::RegisteredOnly) {
        Ok((d, _)) => d,
        Err(e) => {
            return (
                format!(
                    "{t}record  REFUSED — {e}\nverified        0 accepted, 1 refused or stale\n"
                ),
                1,
            )
        }
    };
    let all = candidates(&d);
    let (mut ok, mut bad) = (0usize, 0usize);
    let head = manifest_head(&d);
    let mut canonical = head.clone();
    let mut proved = std::collections::BTreeSet::new();
    let (mut unf, mut inf) = (0usize, 0usize);
    let mut last: Option<usize> = None;
    let mut problems = Vec::new();
    if !manifest.starts_with(&head) {
        problems.push("manifest.tsv: the header lines differ from the generated ones (definitions file, hash, registry or columns)".to_string());
    }
    for line in manifest
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
    {
        canonical.push_str(line);
        canonical.push('\n');
        let cols: Vec<&str> = line.split('\t').collect();
        let id = cols.first().copied().unwrap_or("");
        let st = cols.get(1).copied().unwrap_or("");
        let res = match index_of(id) {
            Some(i) if proved.contains(&i) => Err("duplicate id in the manifest".to_string()),
            Some(i) if last.is_some_and(|l| l >= i) => Err("out of generation order".to_string()),
            _ => check_row(root, out_dir, &d, &all, &cols),
        };
        if let Some(i) = index_of(id) {
            last = Some(i);
            proved.insert(i);
        }
        match res {
            Ok(()) => {
                ok += 1;
                if cols.get(11) == Some(&"unfolding") {
                    unf += 1
                } else {
                    inf += 1
                }
                let _ = writeln!(t, "{id}  ACCEPT {st}");
            }
            Err(e) => {
                bad += 1;
                let _ = writeln!(t, "{id}  REFUSED {st} — {e}");
            }
        }
    }
    if manifest != canonical {
        problems.push("manifest.tsv: must be exactly the generated header lines and one `\\n`-terminated row per proved candidate".into());
    }
    let n = summary
        .lines()
        .nth(1)
        .and_then(|g| g.rsplit_once('('))
        .and_then(|(_, r)| r.strip_suffix(" candidates)"))
        .and_then(|k| k.parse::<usize>().ok())
        .filter(|&k| k > 0 && k <= all.len());
    match n {
        None => problems.push(format!(
            "summary.txt: the candidate count must be 1 … {}",
            all.len()
        )),
        Some(n) => {
            if proved.iter().any(|&i| i >= n) {
                problems.push(format!(
                    "manifest.tsv: a row lies outside the first {n} candidates"
                ));
            }
            let mut exp_nf = String::from(NOT_FOUND_HEADER);
            for (i, st) in all.iter().enumerate().take(n) {
                if !proved.contains(&i) {
                    let _ = writeln!(exp_nf, "{}\t{st}\tNOT_FOUND_WITHIN_BOUNDS", discovery_id(i));
                }
            }
            if nf_text != exp_nf {
                problems.push(format!(
                    "not-found.tsv: must list exactly the {} unproved candidates once each, in order",
                    n - proved.len().min(n)
                ));
            }
            let exp_sum = summary_text(out_dir, &d, n, unf, inf, n - proved.len().min(n), 0);
            if summary != exp_sum {
                problems.push("summary.txt: differs from the summary of this outcome".into());
            }
            if problems.is_empty() && bad == 0 {
                let _ = writeln!(
                    t,
                    "record          complete — {GRAMMAR} grammar over {} ({} sha256 {}), {n} of {} candidates: {} proved ({unf} unfolding, {inf} inference), {} not found, each once",
                    unary(&d).join(", "),
                    d.file,
                    d.file_sha256,
                    all.len(),
                    proved.len(),
                    n - proved.len()
                );
            }
        }
    }
    let listed: std::collections::BTreeSet<String> = proved
        .iter()
        .map(|&i| format!("{}.rlt", discovery_id(i)))
        .collect();
    if let Ok(found) = stale_certificates(&root.join(out_dir)) {
        for p in found {
            let name = p
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            if !listed.contains(&name) {
                problems.push(format!(
                    "{name}: UNLISTED — a discovery certificate no manifest row lists"
                ));
            }
        }
    }
    for p in &problems {
        bad += 1;
        let _ = writeln!(t, "record  REFUSED — {p}");
    }
    let _ = writeln!(t, "verified        {ok} accepted, {bad} refused or stale");
    (t, if bad == 0 { 0 } else { 1 })
}

//! `rlt explain RECORD` (RLT-004b): one statement record, four independently
//! reported outcomes.
//!
//! This module sits outside the trusted kernel. The kernel outcome comes from
//! the same function `rlt check` uses (`check_inner`, through
//! `check_bytes_outcome`), run on exactly the bytes whose SHA-256 is
//! reported. A defect here could misstate a verdict to a reader or a
//! consumer of the JSON; it cannot change what the kernel accepts, and the
//! full `rlt check` report is carried in the JSON so any reader can compare.
//!
//! Every value printed, in text or JSON, comes from one `Explanation` value.

use crate::{citation_path, rp_path, sha256_hex, ACCEPT, MALFORMED, REJECT};
use rlt_syntax::records::{
    load_readingpoint, load_registry, load_statement, read_regular, units, StatementRecord,
};
use std::fmt::Write as _;
use std::path::{Component, Path, PathBuf};

/// Exit status for a record that is valid, but whose pins differ from the
/// current files or kernel result.
pub const STALE: i32 = 3;

// ---------------------------------------------------------------------------
// JSON: a small, dependency-free writer. Objects keep insertion order, so the
// output is deterministic.

#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Int(i64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    fn s(v: impl Into<String>) -> Json {
        Json::Str(v.into())
    }
    fn opt(v: &Option<String>) -> Json {
        v.as_ref().map(|s| Json::s(s.clone())).unwrap_or(Json::Null)
    }
    fn strs(v: &[String]) -> Json {
        Json::Arr(v.iter().map(|s| Json::s(s.clone())).collect())
    }
}

/// A JSON string literal (RFC 8259): `"` and `\` escaped, every character
/// below U+0020 escaped, U+2028 and U+2029 escaped so the output is also
/// valid JavaScript, everything else written as UTF-8.
pub fn json_string(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0C}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 || c == '\u{7F}' || c == '\u{2028}' || c == '\u{2029}' => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

pub fn json_write(v: &Json, indent: usize, out: &mut String) {
    let pad = |n: usize| "  ".repeat(n);
    match v {
        Json::Null => out.push_str("null"),
        Json::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Json::Int(i) => {
            let _ = write!(out, "{i}");
        }
        Json::Str(s) => json_string(s, out),
        Json::Arr(a) if a.is_empty() => out.push_str("[]"),
        Json::Obj(o) if o.is_empty() => out.push_str("{}"),
        Json::Arr(a) => {
            out.push_str("[\n");
            for (i, x) in a.iter().enumerate() {
                out.push_str(&pad(indent + 1));
                json_write(x, indent + 1, out);
                out.push_str(if i + 1 < a.len() { ",\n" } else { "\n" });
            }
            out.push_str(&pad(indent));
            out.push(']');
        }
        Json::Obj(o) => {
            out.push_str("{\n");
            for (i, (k, x)) in o.iter().enumerate() {
                out.push_str(&pad(indent + 1));
                json_string(k, out);
                out.push_str(": ");
                json_write(x, indent + 1, out);
                out.push_str(if i + 1 < o.len() { ",\n" } else { "\n" });
            }
            out.push_str(&pad(indent));
            out.push('}');
        }
    }
}

pub(crate) fn obj(fields: Vec<(&str, Json)>) -> Json {
    Json::Obj(
        fields
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    )
}

// ---------------------------------------------------------------------------
// The one internal report structure.

#[derive(Clone, Debug, Default)]
pub struct Cited {
    pub label: String,
    pub path: String,
    pub proof_id: String,
    pub sha256: String,
    pub proved: String,
}

/// The kernel outcome, read from the `rlt check` report of the checked bytes.
#[derive(Clone, Debug, Default)]
pub struct KernelBlock {
    /// `ACCEPT`, `REJECT` or `MALFORMED`.
    pub verdict: String,
    /// The report's verdict line, after the label.
    pub verdict_line: String,
    pub exit: i32,
    pub proof: String,
    pub sha256: String,
    pub proof_id: Option<String>,
    pub written_goal: Option<String>,
    pub convention: Option<String>,
    pub definitions_used: Vec<String>,
    pub definitions_sha256: Option<String>,
    pub checked_goal: Option<String>,
    pub axiom_set: Option<String>,
    pub axiom_set_sha256: Option<String>,
    pub proved: Option<String>,
    pub assumptions: Option<String>,
    /// Γ as `(name, formula)` pairs from the kernel's theorem, when accepted.
    pub assumption_pairs: Option<Vec<(String, String)>>,
    pub axioms_used: Option<String>,
    pub rules_used: Option<String>,
    pub theorems_cited: Option<String>,
    pub steps: Option<String>,
    pub cites: Vec<Cited>,
    /// The complete `rlt check` report text for these bytes.
    pub check_report: String,
}

#[derive(Clone, Debug)]
pub struct StaleItem {
    pub what: String,
    pub record: String,
    pub current: String,
}

#[derive(Clone, Debug)]
pub struct SourceBlock {
    pub registry: String,
    pub registry_status: String,
    pub commit: Option<String>,
    pub location: String,
    /// Each excerpt and whether it equals a whole unit of a registered excerpt.
    pub excerpts: Vec<String>,
    /// `verified`: every excerpt is a whole unit of the source's registered excerpts.
    pub registry_fidelity: String,
    /// What checks the excerpts against the source itself.
    pub upstream: String,
}

#[derive(Clone, Debug)]
pub struct LeanCheck {
    pub file: String,
    pub theorem: String,
    pub present: bool,
}

#[derive(Clone, Debug)]
pub struct SemanticBlock {
    pub status: String,
    pub assessed_by: Option<String>,
    pub date: Option<String>,
    pub basis_path: Option<String>,
    pub basis_sha256: Option<String>,
    pub basis_current: Option<bool>,
    pub assessed_goal: Option<String>,
    pub assessed_goal_current: Option<bool>,
    /// `current`, `stale` or `none` (unassessed).
    pub assessment: String,
    pub stale_reasons: Vec<String>,
    pub matches: Vec<String>,
    pub differs: Vec<String>,
    pub lean: Vec<LeanCheck>,
}

#[derive(Clone, Debug)]
pub struct PhysicalEntry {
    pub id: String,
    pub kind: String,
    pub description: String,
    pub status: String,
    pub evidence: usize,
}

#[derive(Clone, Debug)]
pub struct PhysicalBlock {
    pub applicability: String,
    pub reason: Option<String>,
    pub reading_point: Option<String>,
    pub entries: Vec<PhysicalEntry>,
}

#[derive(Clone, Debug)]
pub struct Explanation {
    pub path: String,
    pub id: Option<String>,
    pub subject: Option<String>,
    pub identifiers: Vec<(String, String)>,
    /// `invalid`, `stale` or `current`.
    pub state: String,
    pub problems: Vec<String>,
    pub stale: Vec<StaleItem>,
    pub kernel: Option<KernelBlock>,
    pub source: Option<SourceBlock>,
    pub semantic: Option<SemanticBlock>,
    pub physical: Option<PhysicalBlock>,
    pub open_questions: Vec<String>,
    /// RLT-004c: the library entry whose proof pin is this record's, if any,
    /// and the state of the library index (`valid` or the reason it is not).
    pub library_id: Option<String>,
    pub library_index: String,
    pub exit: i32,
}

// ---------------------------------------------------------------------------
// Building the explanation.

/// A repository-relative path of plain components with no symlink at any
/// level, ending in a regular file with the given extension.
fn repo_file(root: &Path, p: &str, ext: &str, what: &str) -> Result<PathBuf, String> {
    let shown = p.escape_debug();
    if p.is_empty()
        || p.chars().any(|c| c.is_control() || c == '\\')
        || !p.split('/').all(|c| !c.is_empty() && c != "." && c != "..")
        || Path::new(p)
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
        || !p.ends_with(ext)
    {
        return Err(format!(
            "{what} `{shown}` must be a repository-relative {ext} path of plain components"
        ));
    }
    let mut cur = root.to_path_buf();
    for c in Path::new(p).components() {
        cur.push(c);
        let meta = std::fs::symlink_metadata(&cur)
            .map_err(|e| format!("{what} `{shown}`: {}", e.kind()))?;
        if meta.file_type().is_symlink() {
            return Err(format!("{what} `{shown}` passes through a symbolic link"));
        }
    }
    if !std::fs::symlink_metadata(&cur)
        .map(|m| m.is_file())
        .unwrap_or(false)
    {
        return Err(format!("{what} `{shown}` is not a regular file"));
    }
    Ok(cur)
}

/// The value of a top-level report line `label` (labels fill 16 columns).
fn line<'a>(report: &'a str, label: &str) -> Option<&'a str> {
    report.lines().find_map(|l| {
        (l.starts_with(label)
            && l.as_bytes().get(label.len()) == Some(&b' ')
            && l[..16.min(l.len())].trim_end() == label)
            .then(|| l.get(16..).unwrap_or(""))
    })
}

fn kernel_block(root: &Path, proof_path: &Path, rel: &str, bytes: &[u8]) -> KernelBlock {
    // One read: the bytes hashed here are the bytes checked, whether or not
    // they are UTF-8 (a non-UTF-8 file gets `rlt check`'s own report).
    let out = crate::check_bytes_outcome(root, proof_path, bytes.to_vec());
    kernel_fields(&out, rel, &sha256_hex(bytes))
}

/// The kernel block of one checked file: `rel` and `sha` name the bytes the
/// outcome checked. Used by `rlt explain` and, per node, by `rlt deps`.
pub fn kernel_fields(out: &crate::CheckOutcome, rel: &str, sha: &str) -> KernelBlock {
    let mut k = KernelBlock {
        proof: rel.to_string(),
        sha256: sha.to_string(),
        ..Default::default()
    };
    let r = &out.report.text;
    k.check_report = r.clone();
    if out.report.code == ACCEPT {
        k.assumption_pairs = out.theorem.as_ref().map(|th| {
            th.assumptions()
                .iter()
                .map(|(n, f)| (n.clone(), f.to_string()))
                .collect()
        });
    }
    k.exit = out.report.code;
    k.verdict = match out.report.code {
        ACCEPT => "ACCEPT",
        REJECT => "REJECT",
        _ => "MALFORMED",
    }
    .into();
    k.verdict_line = line(r, "verdict").unwrap_or("").to_string();
    k.proof_id = line(r, "proof id").map(String::from);
    k.written_goal = line(r, "written goal").map(String::from);
    k.convention = line(r, "convention")
        .and_then(|v| v.split_whitespace().next())
        .map(String::from);
    if let Some(d) = line(r, "definitions") {
        if let Some((used, rest)) = d.split_once(" — ") {
            k.definitions_used = used.split(", ").map(String::from).collect();
            k.definitions_sha256 = rest
                .rsplit(' ')
                .next()
                .map(|s| s.trim_end_matches(')').to_string());
        }
    }
    k.checked_goal = line(r, "checked goal").map(String::from);
    k.axiom_set = line(r, "axiom set")
        .and_then(|v| v.split_whitespace().next())
        .map(String::from);
    // The axiom set's sha256 is the indented line right after `axiom set`.
    let lines: Vec<&str> = r.lines().collect();
    for (i, l) in lines.iter().enumerate() {
        if l.starts_with("axiom set ") {
            k.axiom_set_sha256 = lines
                .get(i + 1)
                .and_then(|n| n.strip_prefix("  sha256"))
                .map(|v| v.trim().to_string());
        }
    }
    // Citations: label, path and hash from the parsed proof file; the cited
    // proof id and formula from the report line whose exact prefix is known.
    if let Some(pf) = &out.proof {
        for c in &pf.cites {
            let prefix = format!("cites           {} = {} (", c.label, c.path.escape_debug());
            let Some(i) = lines.iter().position(|l| l.starts_with(&prefix)) else {
                continue;
            };
            let mut cited = Cited {
                label: c.label.clone(),
                path: c.path.clone(),
                proof_id: lines[i][prefix.len()..]
                    .strip_suffix(')')
                    .unwrap_or("")
                    .to_string(),
                sha256: c.sha256.clone(),
                ..Default::default()
            };
            for n in lines.iter().skip(i + 1).take_while(|n| n.starts_with("  ")) {
                if let Some(v) = n.strip_prefix("  proved") {
                    cited.proved = v.trim().into();
                }
            }
            k.cites.push(cited);
        }
    }
    k.proved = line(r, "proved").map(String::from);
    k.assumptions = line(r, "assumptions").map(String::from);
    k.axioms_used = line(r, "axioms used").map(String::from);
    k.rules_used = line(r, "rules used").map(String::from);
    k.theorems_cited = line(r, "theorems cited").map(String::from);
    k.steps = line(r, "steps").map(String::from);
    k
}

/// Γ as `rlt check` prints it, from a record's `(name, formula)` pins.
fn gamma_text(pins: &[(String, String)]) -> String {
    if pins.is_empty() {
        "Γ = ∅".into()
    } else {
        let parts: Vec<String> = pins.iter().map(|(n, f)| format!("{n} : {f}")).collect();
        format!("Γ = {{ {} }}", parts.join(", "))
    }
}

fn invalid(path: String, problem: String) -> Explanation {
    Explanation {
        path,
        id: None,
        subject: None,
        identifiers: vec![],
        state: "invalid".into(),
        problems: vec![problem],
        stale: vec![],
        kernel: None,
        source: None,
        semantic: None,
        physical: None,
        open_questions: vec![],
        library_id: None,
        library_index: String::new(),
        exit: MALFORMED,
    }
}

/// Build the explanation of one statement record.
pub fn explain(root: &Path, record: &str) -> Explanation {
    let mut x = explain_at(root, record);
    // Loader messages name files by full path; show them repository-relative
    // so the report is the same on every machine.
    let r = root.display().to_string();
    for p in &mut x.problems {
        *p = relative_problem(&r, p);
    }
    x
}

/// A loader message with the repository root removed where it starts a path:
/// at the start of the message or after a character that cannot be part of
/// a path name. `./` is removed only at the start.
pub fn relative_problem(root: &str, p: &str) -> String {
    let r = root.trim_end_matches('/');
    if r == "." || r.is_empty() {
        return p.strip_prefix("./").unwrap_or(p).to_string();
    }
    let pat = format!("{r}/");
    let mut out = String::new();
    let mut rest = p;
    while let Some(i) = rest.find(&pat) {
        let before = if i == 0 {
            out.chars().last()
        } else {
            rest[..i].chars().last()
        };
        let starts_path = before.is_none_or(|c| !(c.is_alphanumeric() || "._-/~".contains(c)));
        out.push_str(&rest[..i]);
        if starts_path {
            rest = rest[i + r.len()..].trim_start_matches('/');
        } else {
            out.push_str(&pat);
            rest = &rest[i + pat.len()..];
        }
    }
    out.push_str(rest);
    out
}

fn explain_at(root: &Path, record: &str) -> Explanation {
    let shown = record.escape_debug().to_string();
    let path = match repo_file(root, record, ".yaml", "statement record") {
        Ok(p) => p,
        Err(e) => return invalid(shown, e),
    };
    let rec = match load_statement(&path) {
        Ok(r) => r,
        Err(e) => return invalid(shown, e),
    };
    match resolve(root, &rec) {
        Ok(x) => {
            let mut e = finish(shown, rec, x);
            // The library link is reported, never relied on: it changes no
            // status and no exit code.
            match crate::deps::load_library(root) {
                Ok(l) => {
                    e.library_index = "valid".into();
                    if let Some(k) = &e.kernel {
                        e.library_id = l.by_pin(&k.proof, &k.sha256).map(|x| x.id.clone());
                    }
                }
                Err(m) => e.library_index = format!("invalid: {m}"),
            }
            e
        }
        Err(e) => {
            let mut x = invalid(shown, e);
            x.id = Some(rec.id);
            x.subject = Some(rec.subject);
            x
        }
    }
}

struct Resolved {
    kernel: KernelBlock,
    source: SourceBlock,
    semantic_basis: Option<String>,
    lean: Vec<LeanCheck>,
    physical: PhysicalBlock,
}

/// Resolve every reference; any failure makes the record invalid.
fn resolve(root: &Path, rec: &StatementRecord) -> Result<Resolved, String> {
    for (rp, _) in &rec.identifiers {
        load_readingpoint(&rp_path(root, rp))
            .map_err(|e| format!("identifiers: reading point `{rp}`: {e}"))?;
    }
    // Source: registry fidelity only; the upstream check is check_excerpts.py.
    let registry = load_registry(&root.join("sources/registry.yaml"))?;
    let src = registry.get(&rec.source.registry).ok_or(format!(
        "source: `{}` is not in sources/registry.yaml",
        rec.source.registry.escape_debug()
    ))?;
    // `available`: a file at a git commit; `archived` (RLT-005): a member of
    // an immutable archive whose SHA-256 is the registry's `commit`. Either
    // way the record must state the same pin.
    let pinned = src.status == "available" || src.status == "archived";
    match (&rec.source.commit, pinned) {
        (Some(c), true) if *c == src.commit => {}
        (Some(c), true) => {
            return Err(format!(
                "source: commit {c} differs from the registry's {} for {}",
                src.commit, src.id
            ))
        }
        (None, true) => {
            return Err(format!(
                "source: {} is pinned at commit {}; the record must state that commit",
                src.id, src.commit
            ))
        }
        (Some(_), false) => {
            return Err(format!(
                "source: {} has registry status `{}` and no commit; the record states one",
                src.id, src.status
            ))
        }
        (None, false) => {}
    }
    let registered: Vec<String> = src.excerpts.iter().flat_map(|e| e.units()).collect();
    for x in &rec.source.excerpts {
        let unit = units(x, true).into_iter().next().unwrap_or_default();
        if unit != *x {
            return Err(format!(
                "source: excerpt \"{}\" must be written as its normalised unit \"{}\" (no markup, entities or repeated spaces)",
                x.escape_debug(),
                unit.escape_debug()
            ));
        }
        if !registered.contains(&unit) {
            return Err(format!(
                "source: excerpt \"{}\" is not a whole unit of any registered excerpt of {}",
                x.escape_debug(),
                src.id
            ));
        }
    }
    let source = SourceBlock {
        registry: src.id.clone(),
        registry_status: src.status.clone(),
        commit: if pinned {
            Some(src.commit.clone())
        } else {
            None
        },
        location: src.location.clone(),
        excerpts: rec.source.excerpts.clone(),
        registry_fidelity: "verified".into(),
        upstream: if pinned {
            format!(
                "not run by rlt explain; scripts/check_excerpts.py compares registered excerpts with the source at {} (run by scripts/verify.sh)",
                src.commit
            )
        } else {
            format!(
                "unpinned: registry status `{}` names no file to check against",
                src.status
            )
        },
    };

    // The record's convention and axiom set must be ones this binary knows.
    if !crate::PINNED_CONVENTIONS
        .iter()
        .any(|(c, _, _)| *c == rec.kernel.convention)
    {
        return Err(format!(
            "kernel: convention `{}` is not registered in this rlt binary",
            rec.kernel.convention.escape_debug()
        ));
    }
    crate::load_axiom_set(root, &rec.kernel.axiom_set).map_err(|e| format!("kernel: {e}"))?;

    // Kernel: read the proof once, hash those bytes and check those bytes.
    let proof_path = citation_path(root, &rec.kernel.proof).map_err(|e| format!("kernel: {e}"))?;
    let bytes = std::fs::read(&proof_path)
        .map_err(|e| format!("kernel: proof `{}`: {}", rec.kernel.proof, e.kind()))?;
    let kernel = kernel_block(root, &proof_path, &rec.kernel.proof, &bytes);

    // Semantic basis and Lean references must exist.
    let semantic_basis = match &rec.semantic.basis {
        Some((p, _)) => {
            let bp = repo_file(root, p, "", "semantic basis")?;
            let b =
                std::fs::read(&bp).map_err(|e| format!("semantic basis `{p}`: {}", e.kind()))?;
            Some(sha256_hex(&b))
        }
        None => None,
    };
    let mut lean = Vec::new();
    for l in &rec.semantic.lean {
        // Only the top-level lean/*.lean files, which scripts/verify.sh
        // compiles and whose referenced theorems it gates with #print axioms.
        if !(l.file.starts_with("lean/") && l.file[5..].split('/').count() == 1) {
            return Err(format!(
                "semantic: Lean file `{}` must be a top-level file in lean/ (the files scripts/verify.sh compiles)",
                l.file.escape_debug()
            ));
        }
        let lp = repo_file(root, &l.file, ".lean", "Lean file")?;
        let text = read_regular(&lp)?;
        let (ns, name) = l.theorem.rsplit_once('.').unwrap_or(("", &l.theorem));
        let has_name = text.lines().any(|t| {
            let mut w = t.split_whitespace();
            matches!(w.next(), Some("theorem") | Some("lemma")) && w.next() == Some(name)
        });
        let has_ns = ns.is_empty() || text.lines().any(|t| t.trim() == format!("namespace {ns}"));
        if !(has_name && has_ns) || name.is_empty() {
            return Err(format!(
                "semantic: Lean theorem `{}` is not declared in {}",
                l.theorem.escape_debug(),
                l.file.escape_debug()
            ));
        }
        lean.push(LeanCheck {
            file: l.file.clone(),
            theorem: l.theorem.clone(),
            present: true,
        });
    }

    // Physical: only when the record says it is relevant.
    let mut physical = PhysicalBlock {
        applicability: rec.physical.applicability.clone(),
        reason: rec.physical.reason.clone(),
        reading_point: rec.physical.reading_point.clone(),
        entries: vec![],
    };
    if let Some(rpid) = &rec.physical.reading_point {
        let rp = load_readingpoint(&rp_path(root, rpid))
            .map_err(|e| format!("physical: reading point `{}`: {e}", rpid.escape_debug()))?;
        for cid in &rec.physical.correspondence {
            let c = rp
                .correspondence
                .iter()
                .find(|c| &c.id == cid)
                .ok_or(format!(
                    "physical: `{}` is not a correspondence record of reading point {rpid}",
                    cid.escape_debug()
                ))?;
            physical.entries.push(PhysicalEntry {
                id: c.id.clone(),
                kind: c.kind.clone(),
                description: c.description.clone(),
                status: c.status.clone(),
                evidence: c.evidence.len(),
            });
        }
    }
    Ok(Resolved {
        kernel,
        source,
        semantic_basis,
        lean,
        physical,
    })
}

fn finish(path: String, rec: StatementRecord, x: Resolved) -> Explanation {
    let k = &x.kernel;
    let p = &rec.kernel;
    let mut stale = Vec::new();
    let mut cmp = |what: &str, record: Option<&str>, current: Option<&str>| {
        if record != current {
            stale.push(StaleItem {
                what: what.into(),
                record: record.unwrap_or("(none)").into(),
                current: current.unwrap_or("(none)").into(),
            });
        }
    };
    cmp("proof sha256", Some(&p.sha256), Some(&k.sha256));
    // Pins the kernel report could not supply (a malformed proof stops early)
    // are compared only when the report states them.
    if k.convention.is_some() {
        cmp("convention", Some(&p.convention), k.convention.as_deref());
    }
    if k.checked_goal.is_some() {
        cmp(
            "checked goal",
            Some(&p.checked_goal),
            k.checked_goal.as_deref(),
        );
        cmp(
            "definitions sha256",
            p.definitions_sha256.as_deref(),
            k.definitions_sha256.as_deref(),
        );
    }
    if k.axiom_set.is_some() {
        cmp("axiom set", Some(&p.axiom_set), k.axiom_set.as_deref());
        cmp(
            "axiom set sha256",
            Some(&p.axiom_set_sha256),
            k.axiom_set_sha256.as_deref(),
        );
    }
    // Γ is compared pair by pair with the kernel's theorem, so a formula
    // containing `, ` cannot make two different contexts print alike.
    if let Some(pairs) = &k.assumption_pairs {
        let mut pinned = p.assumptions.clone();
        pinned.sort();
        if *pairs != pinned {
            stale.push(StaleItem {
                what: "assumptions".into(),
                record: gamma_text(&p.assumptions),
                current: k.assumptions.clone().unwrap_or_else(|| gamma_text(pairs)),
            });
        }
    }

    // Semantic: an attributed assessment, pinned to the goal it assessed.
    let s = &rec.semantic;
    let basis_current = match (&s.basis, &x.semantic_basis) {
        (Some((_, pin)), Some(now)) => Some(pin == now),
        _ => None,
    };
    let goal_current = s
        .assessed_goal
        .as_ref()
        .map(|g| Some(g) == k.checked_goal.as_ref());
    let mut reasons = Vec::new();
    if basis_current == Some(false) {
        reasons.push(format!(
            "basis {} has sha256 {}, the assessment records {}",
            s.basis.as_ref().map(|b| b.0.as_str()).unwrap_or(""),
            x.semantic_basis.clone().unwrap_or_default(),
            s.basis.as_ref().map(|b| b.1.as_str()).unwrap_or("")
        ));
    }
    if goal_current == Some(false) {
        reasons.push("the assessed goal differs from the checked goal".into());
    }
    let semantic = SemanticBlock {
        status: s.status.clone(),
        assessed_by: s.assessed_by.clone(),
        date: s.date.clone(),
        basis_path: s.basis.as_ref().map(|b| b.0.clone()),
        basis_sha256: s.basis.as_ref().map(|b| b.1.clone()),
        basis_current,
        assessed_goal: s.assessed_goal.clone(),
        assessed_goal_current: goal_current,
        assessment: if s.status == "unassessed" {
            "none".into()
        } else if reasons.is_empty() {
            "current".into()
        } else {
            "stale".into()
        },
        stale_reasons: reasons,
        matches: s.matches.clone(),
        differs: s.differs.clone(),
        lean: x.lean,
    };

    let state = if stale.is_empty() { "current" } else { "stale" };
    // Exit: the record's state and the checker's verdict; never the
    // semantic or physical statuses.
    let exit = match k.exit {
        MALFORMED => MALFORMED,
        REJECT => REJECT,
        _ if !stale.is_empty() => STALE,
        _ => ACCEPT,
    };
    Explanation {
        path,
        id: Some(rec.id),
        subject: Some(rec.subject),
        identifiers: rec.identifiers,
        state: state.into(),
        problems: vec![],
        stale,
        kernel: Some(x.kernel),
        source: Some(x.source),
        semantic: Some(semantic),
        physical: Some(x.physical),
        open_questions: rec.open_questions,
        library_id: None,
        library_index: String::new(),
        exit,
    }
}

// ---------------------------------------------------------------------------
// Rendering. Both forms read only the `Explanation`.

const L: usize = 16;

/// One report line. Every value passes through `one`, so no value from a
/// record, proof file or report can add, split or reorder lines.
pub(crate) fn row(t: &mut String, label: &str, value: &str) {
    let _ = writeln!(t, "{label:<L$}{}", one(value), L = L);
}

/// A string from a record or report, shown on one line: control characters,
/// line and paragraph separators, bidirectional controls and other invisible
/// characters are escaped, so no value can add or reorder report lines.
pub(crate) fn one(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_control() || rlt_syntax::records::invisible(c) {
                c.escape_default().to_string()
            } else {
                c.to_string()
            }
        })
        .collect()
}

pub fn render_text(x: &Explanation) -> String {
    let mut t = String::new();
    row(&mut t, "RLT explain", &one(&x.path));
    if let Some(id) = &x.id {
        row(&mut t, "statement", id);
    }
    if let Some(s) = &x.subject {
        row(&mut t, "subject", &one(s));
    }
    for (rp, role) in &x.identifiers {
        row(
            &mut t,
            "  identifier",
            &format!("{rp} ({}; readingpoints/{rp}.yaml)", one(role)),
        );
    }
    if x.state == "invalid" {
        row(&mut t, "record", "invalid");
        for p in &x.problems {
            row(&mut t, "  problem", &one(p));
        }
        row(
            &mut t,
            "verdict",
            "record invalid — nothing further is reported",
        );
        row(&mut t, "exit", &exit_meaning(x.exit, &x.state));
        return t;
    }
    row(&mut t, "record", &format!("valid · {}", x.state));
    for s in &x.stale {
        row(
            &mut t,
            "  stale",
            &format!(
                "{}: record {} · current {}",
                s.what,
                one(&s.record),
                one(&s.current)
            ),
        );
    }

    if let Some(k) = &x.kernel {
        let _ = writeln!(t, "\n1 KERNEL ACCEPTANCE: {}", k.verdict);
        row(
            &mut t,
            "  proof",
            &format!(
                "{} ({})",
                one(&k.proof),
                k.proof_id.as_deref().unwrap_or("—")
            ),
        );
        row(&mut t, "  sha256", &k.sha256);
        row(
            &mut t,
            "  library",
            &match (&x.library_id, x.library_index.as_str()) {
                (Some(id), _) => format!("{id} (rlt deps {id})"),
                (None, "valid") => "no lib/ entry has this proof pin".to_string(),
                (None, m) => format!("lib/ index {m}"),
            },
        );
        for (label, v) in [
            ("  written goal", &k.written_goal),
            ("  convention", &k.convention),
            ("  checked goal", &k.checked_goal),
        ] {
            if let Some(v) = v {
                row(&mut t, label, &one(v));
            }
        }
        if let Some(d) = &k.definitions_sha256 {
            row(
                &mut t,
                "  definitions",
                &format!(
                    "{} (spec/definitions.rlt sha256 {d})",
                    k.definitions_used.join(", ")
                ),
            );
        }
        if let (Some(a), Some(h)) = (&k.axiom_set, &k.axiom_set_sha256) {
            row(&mut t, "  axiom set", &format!("{a} sha256 {h}"));
        }
        for (label, v) in [
            ("  proved", &k.proved),
            ("  assumptions", &k.assumptions),
            ("  axioms used", &k.axioms_used),
            ("  rules used", &k.rules_used),
            ("  steps", &k.steps),
        ] {
            if let Some(v) = v {
                row(&mut t, label, &one(v));
            }
        }
        if k.assumptions.is_none() {
            row(
                &mut t,
                "  assumptions",
                "(reported by the checker only when it accepts)",
            );
        }
        for c in &k.cites {
            row(
                &mut t,
                "  cites",
                &format!(
                    "{} = {} ({}) sha256 {}",
                    c.label,
                    one(&c.path),
                    c.proof_id,
                    c.sha256
                ),
            );
        }
        if let Some(v) = &k.theorems_cited {
            row(&mut t, "  cited", &one(v));
        }
        row(
            &mut t,
            "  checker",
            &format!("{} (rlt check on these bytes)", one(&k.verdict_line)),
        );
        row(
            &mut t,
            "  scope",
            match k.verdict.as_str() {
                "ACCEPT" => "the checked formula follows from the axiom set; nothing further",
                "REJECT" => "this derivation fails at the step shown; the formula's truth stays undetermined",
                _ => "the file could not be checked as a proof",
            },
        );
    }

    if let Some(s) = &x.source {
        let status = match &s.commit {
            Some(_) => "registry verified · pinned",
            None => "registry verified · unpinned",
        };
        let _ = writeln!(t, "\n2 SOURCE FIDELITY: {status}");
        row(
            &mut t,
            "  source",
            &format!(
                "{} ({}){}",
                s.registry,
                s.registry_status,
                s.commit
                    .as_ref()
                    .map(|c| format!(" @ {c}"))
                    .unwrap_or_default()
            ),
        );
        if !s.location.is_empty() {
            row(&mut t, "  location", &one(&s.location));
        }
        for e in &s.excerpts {
            row(
                &mut t,
                "  excerpt",
                &format!("\"{}\" = a registered unit", one(e)),
            );
        }
        row(
            &mut t,
            "  registry",
            "every excerpt equals a whole unit of the source's registered excerpts",
        );
        row(&mut t, "  upstream", &one(&s.upstream));
        row(
            &mut t,
            "  scope",
            "quotation fidelity only; says nothing about translation",
        );
    }

    if let Some(s) = &x.semantic {
        let status = match s.assessment.as_str() {
            "none" => s.status.clone(),
            a => format!("{} · assessment {a}", s.status),
        };
        let _ = writeln!(t, "\n3 SEMANTIC CORRESPONDENCE: {status}");
        if let Some(a) = &s.assessed_by {
            row(
                &mut t,
                "  assessed by",
                &format!("{} · {}", one(a), s.date.as_deref().unwrap_or("")),
            );
        }
        if let (Some(p), Some(h)) = (&s.basis_path, &s.basis_sha256) {
            let cur = if s.basis_current == Some(true) {
                "matches the file"
            } else {
                "differs from the file"
            };
            row(&mut t, "  basis", &format!("{} sha256 {h} ({cur})", one(p)));
        }
        if let Some(c) = s.assessed_goal_current {
            row(
                &mut t,
                "  assessed goal",
                if c {
                    "equals the checked goal"
                } else {
                    "differs from the checked goal"
                },
            );
        }
        for r in &s.stale_reasons {
            row(&mut t, "  stale", &one(r));
        }
        for m in &s.matches {
            row(&mut t, "  matches", &one(m));
        }
        for d in &s.differs {
            row(&mut t, "  differs", &one(d));
        }
        for l in &s.lean {
            row(
                &mut t,
                "  lean",
                &format!(
                    "{} · {} (named in the file; for records in statements/, scripts/verify.sh checks it with Lean #print axioms)",
                    one(&l.file),
                    one(&l.theorem)
                ),
            );
        }
        row(
            &mut t,
            "  scope",
            "an attributed assessment; kernel acceptance never sets it",
        );
    }

    if let Some(p) = &x.physical {
        let status = if p.applicability == "relevant" {
            "relevant"
        } else {
            "not applicable"
        };
        let _ = writeln!(t, "\n4 PHYSICAL CORRESPONDENCE: {status}");
        if let Some(r) = &p.reason {
            row(&mut t, "  reason", &one(r));
        }
        for e in &p.entries {
            row(
                &mut t,
                "  record",
                &format!(
                    "{} [{}] {} (evidence: {}) — \"{}\" (readingpoints/{}.yaml)",
                    e.id,
                    e.kind,
                    e.status,
                    e.evidence,
                    one(&e.description),
                    p.reading_point.as_deref().unwrap_or("")
                ),
            );
        }
    }

    if !x.open_questions.is_empty() {
        let _ = writeln!(t);
        row(
            &mut t,
            "open questions",
            &x.open_questions.len().to_string(),
        );
        for (i, q) in x.open_questions.iter().enumerate() {
            row(&mut t, &format!("  {}", i + 1), &one(q));
        }
    }
    let _ = writeln!(t);
    row(&mut t, "verdict", &summary(x));
    row(&mut t, "exit", &exit_meaning(x.exit, &x.state));
    t
}

/// What an exit status asserts, and what it leaves open.
pub fn exit_meaning(code: i32, state: &str) -> String {
    let pins = if state == "stale" {
        "; the stale pins listed above also differ from the current files or checker result"
    } else {
        ""
    };
    let base: String = match code {
        ACCEPT => "0 — record valid, its proof and specification pins current, and the RLT checker accepts the proof; semantic correspondence may be incomplete and physical correspondence unestablished".into(),
        REJECT => "1 — record valid; the RLT checker rejects the proof".into(),
        STALE => "3 — record valid and the RLT checker accepts the proof, but a proof or specification pin differs from the current files or checker result".into(),
        _ => "2 — the record is invalid, or the RLT checker reports the proof malformed".into(),
    };
    if code == REJECT || (code == MALFORMED && state != "invalid") {
        format!("{base}{pins}")
    } else {
        base
    }
}

fn summary(x: &Explanation) -> String {
    let k = x
        .kernel
        .as_ref()
        .map(|k| k.verdict.as_str())
        .unwrap_or("not run");
    let sem = x
        .semantic
        .as_ref()
        .map(|s| match s.assessment.as_str() {
            "none" => s.status.clone(),
            a => format!("{} ({a})", s.status),
        })
        .unwrap_or_default();
    let src = x
        .source
        .as_ref()
        .map(|s| {
            if s.commit.is_some() {
                "registry verified, pinned"
            } else {
                "registry verified, unpinned"
            }
        })
        .unwrap_or("");
    let phys = x
        .physical
        .as_ref()
        .map(|p| {
            if p.applicability == "relevant" {
                let st: Vec<String> = p
                    .entries
                    .iter()
                    .map(|e| format!("{} {}", e.id, e.status))
                    .collect();
                st.join(", ")
            } else {
                "not applicable".into()
            }
        })
        .unwrap_or_default();
    format!(
        "record valid · {} · proof {k} · source {src} · semantic {sem} · physical {phys} · exit {}",
        x.state, x.exit
    )
}

pub fn to_json(x: &Explanation) -> Json {
    let kernel = x.kernel.as_ref().map(|k| {
        obj(vec![
            ("verdict", Json::s(&k.verdict)),
            ("verdict_line", Json::s(&k.verdict_line)),
            ("checker_exit", Json::Int(k.exit as i64)),
            ("proof", Json::s(&k.proof)),
            ("sha256", Json::s(&k.sha256)),
            ("library_id", Json::opt(&x.library_id)),
            ("library_index", Json::s(&x.library_index)),
            ("proof_id", Json::opt(&k.proof_id)),
            ("written_goal", Json::opt(&k.written_goal)),
            ("convention", Json::opt(&k.convention)),
            (
                "definitions",
                match &k.definitions_sha256 {
                    Some(h) => obj(vec![
                        ("file", Json::s("spec/definitions.rlt")),
                        ("sha256", Json::s(h)),
                        ("used", Json::strs(&k.definitions_used)),
                    ]),
                    None => Json::Null,
                },
            ),
            ("checked_goal", Json::opt(&k.checked_goal)),
            ("axiom_set", Json::opt(&k.axiom_set)),
            ("axiom_set_sha256", Json::opt(&k.axiom_set_sha256)),
            ("proved", Json::opt(&k.proved)),
            ("assumptions", Json::opt(&k.assumptions)),
            ("axioms_used", Json::opt(&k.axioms_used)),
            ("rules_used", Json::opt(&k.rules_used)),
            ("theorems_cited", Json::opt(&k.theorems_cited)),
            ("steps", Json::opt(&k.steps)),
            (
                "cites",
                Json::Arr(
                    k.cites
                        .iter()
                        .map(|c| {
                            obj(vec![
                                ("label", Json::s(&c.label)),
                                ("path", Json::s(&c.path)),
                                ("proof_id", Json::s(&c.proof_id)),
                                ("sha256", Json::s(&c.sha256)),
                                ("proved", Json::s(&c.proved)),
                            ])
                        })
                        .collect(),
                ),
            ),
            ("check_report", Json::s(&k.check_report)),
        ])
    });
    let source = x.source.as_ref().map(|s| {
        obj(vec![
            ("registry", Json::s(&s.registry)),
            ("registry_status", Json::s(&s.registry_status)),
            ("commit", Json::opt(&s.commit)),
            ("location", Json::s(&s.location)),
            ("excerpts", Json::strs(&s.excerpts)),
            ("registry_fidelity", Json::s(&s.registry_fidelity)),
            ("pinned", Json::Bool(s.commit.is_some())),
            ("upstream", Json::s(&s.upstream)),
        ])
    });
    let semantic = x.semantic.as_ref().map(|s| {
        obj(vec![
            ("status", Json::s(&s.status)),
            ("assessment", Json::s(&s.assessment)),
            ("stale_reasons", Json::strs(&s.stale_reasons)),
            ("assessed_by", Json::opt(&s.assessed_by)),
            ("date", Json::opt(&s.date)),
            (
                "basis",
                match (&s.basis_path, &s.basis_sha256) {
                    (Some(p), Some(h)) => obj(vec![
                        ("path", Json::s(p)),
                        ("sha256", Json::s(h)),
                        (
                            "current",
                            s.basis_current.map(Json::Bool).unwrap_or(Json::Null),
                        ),
                    ]),
                    _ => Json::Null,
                },
            ),
            ("assessed_goal", Json::opt(&s.assessed_goal)),
            (
                "assessed_goal_current",
                s.assessed_goal_current
                    .map(Json::Bool)
                    .unwrap_or(Json::Null),
            ),
            ("matches", Json::strs(&s.matches)),
            ("differs", Json::strs(&s.differs)),
            (
                "lean",
                Json::Arr(
                    s.lean
                        .iter()
                        .map(|l| {
                            obj(vec![
                                ("file", Json::s(&l.file)),
                                ("theorem", Json::s(&l.theorem)),
                                ("named_in_file", Json::Bool(l.present)),
                                ("checked_by", Json::s("scripts/verify.sh (Lean #print axioms gate) for records in statements/; not rlt explain")),
                            ])
                        })
                        .collect(),
                ),
            ),
        ])
    });
    let physical = x.physical.as_ref().map(|p| {
        obj(vec![
            ("applicability", Json::s(&p.applicability)),
            ("reason", Json::opt(&p.reason)),
            ("reading_point", Json::opt(&p.reading_point)),
            (
                "records",
                Json::Arr(
                    p.entries
                        .iter()
                        .map(|e| {
                            obj(vec![
                                ("id", Json::s(&e.id)),
                                ("kind", Json::s(&e.kind)),
                                ("description", Json::s(&e.description)),
                                ("status", Json::s(&e.status)),
                                ("evidence", Json::Int(e.evidence as i64)),
                            ])
                        })
                        .collect(),
                ),
            ),
        ])
    });
    obj(vec![
        ("format", Json::s("rlt-explain/1")),
        (
            "record",
            obj(vec![
                ("path", Json::s(&x.path)),
                ("id", Json::opt(&x.id)),
                ("subject", Json::opt(&x.subject)),
                (
                    "identifiers",
                    Json::Arr(
                        x.identifiers
                            .iter()
                            .map(|(rp, role)| {
                                obj(vec![
                                    ("reading_point", Json::s(rp)),
                                    ("role", Json::s(role)),
                                ])
                            })
                            .collect(),
                    ),
                ),
                ("state", Json::s(&x.state)),
                ("problems", Json::strs(&x.problems)),
                (
                    "stale",
                    Json::Arr(
                        x.stale
                            .iter()
                            .map(|s| {
                                obj(vec![
                                    ("what", Json::s(&s.what)),
                                    ("record", Json::s(&s.record)),
                                    ("current", Json::s(&s.current)),
                                ])
                            })
                            .collect(),
                    ),
                ),
            ]),
        ),
        ("exit", Json::Int(x.exit as i64)),
        ("exit_meaning", Json::s(exit_meaning(x.exit, &x.state))),
        ("kernel", kernel.unwrap_or(Json::Null)),
        ("source", source.unwrap_or(Json::Null)),
        ("semantic", semantic.unwrap_or(Json::Null)),
        ("physical", physical.unwrap_or(Json::Null)),
        ("open_questions", Json::strs(&x.open_questions)),
    ])
}

pub fn render_json(x: &Explanation) -> String {
    let mut s = String::new();
    json_write(&to_json(x), 0, &mut s);
    s.push('\n');
    s
}

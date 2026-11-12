//! `rlt deps` (RLT-004c): a theorem's complete dependency graph, and the
//! theorem library in `lib/`.
//!
//! Outside the trusted kernel. Every edge comes from the citation trace that
//! `check_inner` records during the one check `rlt check` performs on the
//! root's bytes; every library field is compared with that check. A defect
//! here could misstate a graph or a verdict to a reader or a JSON consumer;
//! it cannot change what the kernel accepts.

use crate::explain::{
    json_write, kernel_fields, obj, one, row, Json, KernelBlock, StaleItem, STALE,
};
use crate::{
    check_bytes_outcome, citation_path, sha256_hex, CheckOutcome, ACCEPT, MALFORMED,
    MAX_CITATION_DEPTH, MAX_CITED_CHECKS, PINNED_AXIOM_SETS, PINNED_CONVENTIONS,
    PINNED_DEFINITIONS, REJECT,
};
use rlt_syntax::records::{load_lib_entry, load_statement, LibEntry, ProofPin};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

// ---------------------------------------------------------------------------
// Statement fingerprint (C1, refined at approval).

/// A canonical form of a formula: bound variables as de Bruijn indices
/// (`#0` is the nearest binder) and binder names dropped, so alpha-variants
/// share one form; every symbol is written prefix with explicit arity.
pub fn canonical(p: &rlt_kernel::Prop) -> String {
    use rlt_kernel::Prop::*;
    fn term(t: &rlt_kernel::Term, o: &mut String) {
        use rlt_kernel::Term::*;
        match t {
            Var(v) => {
                let _ = write!(o, "var:{v}");
            }
            Param(v) => {
                let _ = write!(o, "param:{v}");
            }
            Bound(i) => {
                let _ = write!(o, "#{i}");
            }
            App(f, args) => {
                let _ = write!(o, "({f}/{}", args.len());
                for a in args {
                    o.push(' ');
                    term(a, o);
                }
                o.push(')');
            }
        }
    }
    fn go(p: &rlt_kernel::Prop, o: &mut String) {
        match p {
            Eq(f) => {
                o.push_str("(= ");
                term(&f.lhs, o);
                o.push(' ');
                term(&f.rhs, o);
                o.push(')');
            }
            False => o.push_str("false"),
            And(a, b) | Or(a, b) | Imp(a, b) => {
                o.push_str(match p {
                    And(..) => "(and ",
                    Or(..) => "(or ",
                    _ => "(imp ",
                });
                go(a, o);
                o.push(' ');
                go(b, o);
                o.push(')');
            }
            Forall(_, b) | Exists(_, b) => {
                o.push_str(if matches!(p, Forall(..)) {
                    "(forall "
                } else {
                    "(exists "
                });
                go(b, o);
                o.push(')');
            }
        }
    }
    let mut o = String::new();
    go(p, &mut o);
    o
}

/// The parts that fix what a theorem states and how it is read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatementParts {
    pub convention: String,
    pub convention_digest: String,
    pub axiom_set: String,
    pub axiom_file: String,
    pub axiom_sha256: String,
    /// `(file, sha256, names used)` when the statement uses definitions.
    pub definitions: Option<(String, String, Vec<String>)>,
    /// Canonical form of the checked formula.
    pub goal: String,
    /// Canonical forms of the assumptions, sorted: their names are labels
    /// inside the proof and leave the statement unchanged.
    pub assumptions: Vec<String>,
}

/// The canonical text the fingerprint hashes. Every line is one part; a
/// change to any part changes the text.
pub fn fingerprint_text(p: &StatementParts) -> String {
    let mut t = String::from("rlt-statement-fingerprint/1\n");
    let _ = writeln!(t, "convention {} {}", p.convention, p.convention_digest);
    let _ = writeln!(
        t,
        "axiom-set {} {} {}",
        p.axiom_set, p.axiom_file, p.axiom_sha256
    );
    match &p.definitions {
        Some((f, h, used)) => {
            let _ = writeln!(t, "definitions {f} {h} used {}", used.join(","));
        }
        None => t.push_str("definitions none\n"),
    }
    let _ = writeln!(t, "goal {}", p.goal);
    if p.assumptions.is_empty() {
        t.push_str("assumptions none\n");
    }
    for f in &p.assumptions {
        let _ = writeln!(t, "assumption {f}");
    }
    t
}

/// The statement parts of an accepted check, from the binary's pins and the
/// kernel's theorem; `None` unless the checker accepted.
pub fn statement_parts(out: &CheckOutcome, k: &KernelBlock) -> Option<StatementParts> {
    if out.report.code != ACCEPT {
        return None;
    }
    let pf = out.proof.as_ref()?;
    let th = out.theorem.as_ref()?;
    let goal = canonical(out.prop.as_ref()?);
    let (_, conv_digest, _) = PINNED_CONVENTIONS
        .iter()
        .find(|(c, _, _)| *c == pf.convention)?;
    let (_, ax_file, ax_sha) = PINNED_AXIOM_SETS
        .iter()
        .find(|(a, _, _)| *a == pf.axiom_set)?;
    // A set: repeated assumptions state nothing more.
    let mut assumptions: Vec<String> = th.assumptions().values().map(canonical).collect();
    assumptions.sort();
    assumptions.dedup();
    Some(StatementParts {
        convention: pf.convention.clone(),
        convention_digest: conv_digest.to_string(),
        axiom_set: pf.axiom_set.clone(),
        axiom_file: ax_file.to_string(),
        axiom_sha256: ax_sha.to_string(),
        definitions: if k.definitions_used.is_empty() {
            None
        } else {
            Some((
                PINNED_DEFINITIONS.0.to_string(),
                PINNED_DEFINITIONS.1.to_string(),
                k.definitions_used.clone(),
            ))
        },
        goal,
        assumptions,
    })
}

// ---------------------------------------------------------------------------
// The library index.

pub struct Library {
    /// `(repository-relative path, entry)`, sorted by id.
    pub entries: Vec<(String, LibEntry)>,
    /// id → (fingerprint, first proof sha256), from lib/identities.tsv.
    pub identities: BTreeMap<String, (String, String)>,
}

impl Library {
    pub fn get(&self, id: &str) -> Option<&LibEntry> {
        self.entries.iter().map(|(_, e)| e).find(|e| e.id == id)
    }
    /// The entry whose proof or alternative is exactly this pin.
    pub fn by_pin(&self, path: &str, sha: &str) -> Option<&LibEntry> {
        let hit = |p: &ProofPin| p.path == path && p.sha256 == sha;
        self.entries
            .iter()
            .map(|(_, e)| e)
            .find(|e| hit(&e.proof) || e.alternatives.iter().any(hit))
    }
}

/// Load and cross-check `lib/`: schema, unique ids and fingerprints, the
/// identity registry, history and `uses`. Any failure makes the index invalid.
pub fn load_library(root: &Path) -> Result<Library, String> {
    let dir = root.join("lib");
    let mut entries = Vec::new();
    let mut areas: Vec<_> = std::fs::read_dir(&dir)
        .map_err(|e| format!("lib/: {}", e.kind()))?
        .collect::<Result<_, _>>()
        .map_err(|e| format!("lib/: {}", e.kind()))?;
    areas.sort_by_key(|e| e.file_name());
    for a in areas {
        let name = a.file_name().to_string_lossy().to_string();
        let ft = a
            .file_type()
            .map_err(|e| format!("lib/{name}: {}", e.kind()))?;
        if ft.is_symlink() {
            return Err(format!("lib/{name} is a symbolic link"));
        }
        if ft.is_file() {
            if name == "README.md" || name == "identities.tsv" {
                continue;
            }
            return Err(format!(
                "lib/{name}: only area folders, README.md and identities.tsv belong in lib/"
            ));
        }
        let mut files: Vec<_> = std::fs::read_dir(a.path())
            .map_err(|e| format!("lib/{name}: {}", e.kind()))?
            .collect::<Result<_, _>>()
            .map_err(|e| format!("lib/{name}: {}", e.kind()))?;
        files.sort_by_key(|e| e.file_name());
        for f in files {
            let fname = f.file_name().to_string_lossy().to_string();
            let rel = format!("lib/{name}/{fname}");
            let ft = f.file_type().map_err(|e| format!("{rel}: {}", e.kind()))?;
            if !ft.is_file() || !fname.ends_with(".yaml") {
                return Err(format!(
                    "{rel}: an area folder holds only <id>.yaml entries"
                ));
            }
            let e = load_lib_entry(&f.path()).map_err(|e| format!("{rel}: {e}"))?;
            entries.push((rel, e));
        }
    }
    entries.sort_by(|a, b| a.1.id.cmp(&b.1.id));

    // The identity registry: id, fingerprint, first proof sha256.
    let reg = rlt_syntax::records::read_regular(&dir.join("identities.tsv"))
        .map_err(|e| format!("lib/identities.tsv: {e}"))?;
    let mut identities = BTreeMap::new();
    let mut fps = BTreeSet::new();
    for (i, l) in reg.lines().enumerate() {
        if l.starts_with('#') || l.is_empty() {
            continue;
        }
        let f: Vec<&str> = l.split('\t').collect();
        let hex = |s: &str| {
            s.len() == 64
                && s.bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        };
        if f.len() != 3 || !hex(f[1]) || !hex(f[2]) {
            return Err(format!(
                "lib/identities.tsv line {}: expected id, fingerprint, first proof sha256",
                i + 1
            ));
        }
        rlt_syntax::records::lib_id(f[0], "lib/identities.tsv")?;
        if identities
            .insert(f[0].to_string(), (f[1].to_string(), f[2].to_string()))
            .is_some()
        {
            return Err(format!(
                "lib/identities.tsv: `{}` is registered twice",
                f[0]
            ));
        }
        if !fps.insert(f[1].to_string()) {
            return Err(format!(
                "lib/identities.tsv: fingerprint {} is registered under two ids",
                f[1]
            ));
        }
    }

    let mut seen_fp: BTreeMap<&str, &str> = BTreeMap::new();
    let mut seen_pin: BTreeMap<(&str, &str), &str> = BTreeMap::new();
    for w in entries.windows(2) {
        if w[0].1.id == w[1].1.id {
            return Err(format!("library id `{}` has two entries", w[0].1.id));
        }
    }
    for (rel, e) in &entries {
        let Some((fp, first)) = identities.get(&e.id) else {
            return Err(format!(
                "{rel}: `{}` is not registered in lib/identities.tsv",
                e.id
            ));
        };
        if *fp != e.fingerprint {
            return Err(format!(
                "{rel}: `{}` claims fingerprint {}, but its registered identity is {fp}: a library id is bound to one statement",
                e.id, e.fingerprint
            ));
        }
        if let Some(other) = seen_fp.insert(&e.fingerprint, &e.id) {
            return Err(format!(
                "{rel}: `{}` and `{other}` name the same statement; one statement has one id",
                e.id
            ));
        }
        for p in std::iter::once(&e.proof).chain(&e.alternatives) {
            if let Some(other) = seen_pin.insert((&p.path, &p.sha256), &e.id) {
                return Err(format!(
                    "{rel}: proof {} is listed under `{}` and `{other}`",
                    p.path, e.id
                ));
            }
        }
        // History: every earlier pin proves the same statement, and the
        // first registered proof stays on record once replaced.
        for h in &e.history {
            if h.fingerprint != *fp {
                return Err(format!(
                    "{rel}: history pin {} has fingerprint {}; a replacement must prove the identical statement",
                    h.sha256, h.fingerprint
                ));
            }
            if h.sha256 == e.proof.sha256 {
                return Err(format!(
                    "{rel}: the current proof {} also appears in history",
                    h.sha256
                ));
            }
        }
        if e.proof.sha256 != *first && !e.history.iter().any(|h| h.sha256 == *first) {
            return Err(format!(
                "{rel}: the proof pin differs from the first registered proof {first}, and history omits it: a replacement must be recorded and reviewed"
            ));
        }
        for u in &e.uses {
            // RLT-006: a cited proof outside the library, such as a generated
            // certificate, is named `(unlisted PATH)`. The entry still pins it:
            // verification compares these names with the citations of the
            // checked proof, and the check re-reads and re-checks every cited
            // file, so a missing or altered file is never accepted.
            if let Some(path) = u
                .strip_prefix("(unlisted ")
                .and_then(|x| x.strip_suffix(')'))
            {
                let ok_path = path.ends_with(".rlt")
                    && !path.starts_with('/')
                    && path
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"_./-".contains(&b))
                    && path
                        .split('/')
                        .all(|c| !c.is_empty() && c != "." && c != "..");
                if !ok_path {
                    return Err(format!(
                        "{rel}: `uses` names `{u}`, which is not a repository-relative proof path"
                    ));
                }
                continue;
            }
            match entries.iter().map(|(_, x)| x).find(|x| x.id == *u) {
                None => return Err(format!("{rel}: `uses` names unknown id `{u}`")),
                Some(x) if !x.citable => {
                    return Err(format!(
                        "{rel}: `uses` names `{u}`, which is conditional and not citable"
                    ))
                }
                _ => {}
            }
        }
    }
    for id in identities.keys() {
        if !entries.iter().any(|(_, e)| e.id == *id) {
            return Err(format!("lib/identities.tsv: `{id}` is registered but has no entry; identities are never retired"));
        }
    }
    Ok(Library {
        entries,
        identities,
    })
}

// ---------------------------------------------------------------------------
// The dependency graph.

#[derive(Clone, Debug)]
pub struct Node {
    /// The citation label in the citing file; `None` for the root.
    pub label: Option<String>,
    pub path: String,
    pub sha256: String,
    pub lib_id: Option<String>,
    pub kernel: KernelBlock,
    pub fingerprint: Option<String>,
    pub fingerprint_text: Option<String>,
    /// Whether this node's file was checked (a citation may be refused
    /// before its file is read).
    pub checked: bool,
    /// Why the citation to this node was refused, when it was.
    pub problem: Option<String>,
    pub children: Vec<Node>,
}

fn build(
    out: &CheckOutcome,
    label: Option<String>,
    path: &str,
    sha: &str,
    lib: Option<&Library>,
) -> Node {
    let k = kernel_fields(out, path, sha);
    let parts = statement_parts(out, &k);
    let text = parts.as_ref().map(fingerprint_text);
    let children = out
        .citations
        .iter()
        .map(|c| match &c.outcome {
            Some(o) => {
                let mut n = build(o, Some(c.label.clone()), &c.path, &c.sha256, lib);
                n.problem = c.problem.clone();
                n
            }
            None => Node {
                label: Some(c.label.clone()),
                path: c.path.clone(),
                sha256: c.sha256.clone(),
                lib_id: lib
                    .and_then(|l| l.by_pin(&c.path, &c.sha256))
                    .map(|e| e.id.clone()),
                kernel: KernelBlock::default(),
                fingerprint: None,
                fingerprint_text: None,
                checked: false,
                problem: c.problem.clone(),
                children: vec![],
            },
        })
        .collect();
    Node {
        label,
        path: path.to_string(),
        sha256: sha.to_string(),
        lib_id: lib.and_then(|l| l.by_pin(path, sha)).map(|e| e.id.clone()),
        fingerprint: text.as_ref().map(|t| sha256_hex(t.as_bytes())),
        fingerprint_text: text,
        kernel: k,
        checked: true,
        problem: None,
        children,
    }
}

fn walk<'a>(n: &'a Node, f: &mut dyn FnMut(&'a Node, usize)) {
    fn go<'a>(n: &'a Node, d: usize, f: &mut dyn FnMut(&'a Node, usize)) {
        f(n, d);
        for c in &n.children {
            go(c, d + 1, f);
        }
    }
    go(n, 0, f)
}

/// Longest chain of citing files from the root (the depth limit counts these).
fn citing_chain(n: &Node) -> usize {
    if n.children.is_empty() {
        // A file refused for nesting too deep was itself a citing file.
        usize::from(n.kernel.verdict_line.contains("citations nest deeper than"))
    } else {
        1 + n.children.iter().map(citing_chain).max().unwrap_or(0)
    }
}

// ---------------------------------------------------------------------------
// One report structure for `rlt deps TARGET`.

#[derive(Clone, Debug)]
pub struct Deps {
    pub target: String,
    pub library: String,
    pub entry: Option<String>,
    pub problems: Vec<String>,
    pub stale: Vec<StaleItem>,
    pub root: Option<Node>,
    pub distinct: usize,
    pub checks: usize,
    /// Checks attempted, counting one refused for the budget.
    pub attempted: usize,
    pub chain: usize,
    pub axioms: Vec<String>,
    pub definitions: Vec<String>,
    pub warnings: Vec<String>,
    /// `(record, state line from rlt explain, its exit)`.
    pub statement_records: Vec<(String, String, i32)>,
    /// The worst checker exit among alternative proofs.
    pub alt_exit: i32,
    /// Notes that change no status (history pins whose bytes are gone).
    pub notes: Vec<String>,
    pub state: String,
    pub exit: i32,
}

fn check_pin(root: &Path, p: &str) -> Result<(String, CheckOutcome), String> {
    let path = citation_path(root, p)?;
    let bytes = std::fs::read(&path).map_err(|e| format!("{}: {}", p.escape_debug(), e.kind()))?;
    let sha = sha256_hex(&bytes);
    Ok((sha, check_bytes_outcome(root, &path, bytes)))
}

/// Compare one library entry with its checked proof (and alternatives,
/// statement records); returns stale items and statement-record states.
fn compare_entry(
    root: &Path,
    lib: &Library,
    e: &LibEntry,
    node: &Node,
    stale: &mut Vec<StaleItem>,
    records: &mut Vec<(String, String, i32)>,
) -> i32 {
    let mut alt_exit = ACCEPT;
    let mut cmp = |what: &str, record: &str, current: &str| {
        if record != current {
            stale.push(StaleItem {
                what: what.into(),
                record: record.into(),
                current: current.into(),
            });
        }
    };
    let k = &node.kernel;
    cmp("proof sha256", &e.proof.sha256, &node.sha256);
    if k.verdict == "ACCEPT" {
        cmp(
            "statement fingerprint",
            &e.fingerprint,
            node.fingerprint.as_deref().unwrap_or("(none)"),
        );
        cmp("proof id", &e.proof_id, k.proof_id.as_deref().unwrap_or(""));
        cmp("axiom set", &e.axiom_set, &crate_axiom_set(node));
        cmp(
            "convention",
            &e.convention,
            k.convention.as_deref().unwrap_or(""),
        );
        cmp(
            "checked goal",
            &e.checked_goal,
            k.checked_goal.as_deref().unwrap_or(""),
        );
        let mut pinned = e.assumptions.clone();
        pinned.sort();
        let current = k.assumption_pairs.clone().unwrap_or_default();
        let show = |v: &[(String, String)]| {
            v.iter()
                .map(|(n, f)| format!("{n} : {f}"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        cmp("assumptions", &show(&pinned), &show(&current));
        let uses: Vec<String> = node
            .children
            .iter()
            .map(|c| {
                c.lib_id
                    .clone()
                    .unwrap_or_else(|| format!("(unlisted {})", c.path))
            })
            .collect();
        // Element by element: one `uses` item stands for exactly one citation.
        if e.uses != uses {
            cmp("uses", &e.uses.join(", "), &uses.join(", "));
            cmp(
                "uses (item count)",
                &e.uses.len().to_string(),
                &uses.len().to_string(),
            );
        }
    }
    for alt in &e.alternatives {
        match check_pin(root, &alt.path) {
            Ok((sha, out)) => {
                if out.report.code != ACCEPT {
                    alt_exit = worse(alt_exit, out.report.code);
                }
                cmp(
                    &format!("alternative {} sha256", alt.path),
                    &alt.sha256,
                    &sha,
                );
                let an = build(&out, None, &alt.path, &sha, Some(lib));
                cmp(
                    &format!("alternative {} fingerprint", alt.path),
                    &e.fingerprint,
                    an.fingerprint.as_deref().unwrap_or("(not accepted)"),
                );
            }
            Err(m) => {
                alt_exit = worse(alt_exit, MALFORMED);
                cmp(&format!("alternative {}", alt.path), "readable", &m)
            }
        }
    }
    // History pins: where the recorded bytes still exist, they must prove
    // the identical statement.
    for h in &e.history {
        let here = check_pin(root, &h.path)
            .ok()
            .filter(|(sha, _)| *sha == h.sha256);
        if here.is_none() {
            records.push((
                format!("history {} {}", h.path, h.sha256),
                "bytes no longer present; the recorded pin is reviewed metadata, not re-checked"
                    .into(),
                ACCEPT,
            ));
        }
        if let Some((sha, out)) = here {
            {
                let hn = build(&out, None, &h.path, &sha, None);
                cmp(
                    &format!("history {} fingerprint", h.path),
                    &h.fingerprint,
                    hn.fingerprint.as_deref().unwrap_or("(not accepted)"),
                );
            }
        }
    }
    for r in &e.statement_records {
        match load_statement(&root.join(r)) {
            Ok(s) => {
                let pin_ok = (s.kernel.proof == e.proof.path && s.kernel.sha256 == e.proof.sha256)
                    || e.alternatives
                        .iter()
                        .any(|a| a.path == s.kernel.proof && a.sha256 == s.kernel.sha256);
                if !pin_ok {
                    stale.push(StaleItem {
                        what: format!("statement record {r} proof pin"),
                        record: format!("{} {}", e.proof.path, e.proof.sha256),
                        current: format!("{} {}", s.kernel.proof, s.kernel.sha256),
                    });
                }
                let x = crate::explain::explain(root, r);
                if x.exit != ACCEPT {
                    stale.push(StaleItem {
                        what: format!("statement record {r}"),
                        record: "rlt explain exit 0".into(),
                        current: format!("rlt explain exit {} ({})", x.exit, x.state),
                    });
                }
                records.push((r.clone(), format!("record {}", x.state), x.exit));
            }
            Err(m) => stale.push(StaleItem {
                what: format!("statement record {r}"),
                record: "valid".into(),
                current: m,
            }),
        }
    }
    alt_exit
}

fn crate_axiom_set(n: &Node) -> String {
    n.kernel.axiom_set.clone().unwrap_or_default()
}

/// `rlt deps TARGET`: TARGET is a library id or a repository-relative proof path.
pub fn deps(root: &Path, target: &str) -> Deps {
    let mut d = Deps {
        target: target.to_string(),
        library: String::new(),
        entry: None,
        problems: vec![],
        stale: vec![],
        root: None,
        distinct: 0,
        checks: 0,
        attempted: 0,
        chain: 0,
        axioms: vec![],
        definitions: vec![],
        warnings: vec![],
        statement_records: vec![],
        alt_exit: ACCEPT,
        notes: vec![],
        state: "invalid".into(),
        exit: MALFORMED,
    };
    let lib = load_library(root);
    d.library = match &lib {
        Ok(l) => format!("lib/ index valid ({} entries)", l.entries.len()),
        Err(e) => format!("lib/ index invalid: {e}"),
    };
    let is_id = !target.contains('/') && !target.ends_with(".rlt");
    let (path, entry) = if is_id {
        let l = match &lib {
            Ok(l) => l,
            Err(e) => {
                d.problems.push(format!("lib/ index invalid: {e}"));
                return d;
            }
        };
        match l.get(target) {
            Some(e) => (e.proof.path.clone(), Some(e.clone())),
            None => {
                d.problems
                    .push(format!("`{}` is not a library id", target.escape_debug()));
                return d;
            }
        }
    } else {
        (target.to_string(), None)
    };
    let (sha, out) = match check_pin(root, &path) {
        Ok(x) => x,
        Err(m) => {
            d.problems.push(m);
            return d;
        }
    };
    let node = build(&out, None, &path, &sha, lib.as_ref().ok());

    let mut pins = BTreeSet::new();
    let mut axioms = BTreeSet::new();
    let mut defs = BTreeSet::new();
    let mut refused_budget = 0;
    walk(&node, &mut |n, depth| {
        pins.insert((n.path.clone(), n.sha256.clone()));
        if depth > 0 && n.checked {
            d.checks += 1;
        }
        if n.problem
            .as_deref()
            .is_some_and(|p| p.contains("would check more than"))
        {
            refused_budget += 1;
        }
        // The unions cover accepted edges only: a refused citation adds
        // nothing to what the root rests on.
        if n.problem.is_none() {
            if let Some(a) = &n.kernel.axioms_used {
                axioms.extend(a.split(", ").filter(|s| *s != "none").map(String::from));
            }
            defs.extend(n.kernel.definitions_used.iter().cloned());
        }
    });
    d.attempted = d.checks + refused_budget;
    d.distinct = pins.len();
    d.chain = citing_chain(&node);
    d.axioms = axioms.into_iter().collect();
    d.definitions = defs.into_iter().collect();
    if d.attempted * 4 > MAX_CITED_CHECKS * 3 {
        d.warnings.push(format!(
            "cited-file checks {} of {MAX_CITED_CHECKS} are above 75% of the limit",
            d.attempted
        ));
    }
    if d.chain * 4 > MAX_CITATION_DEPTH * 3 {
        d.warnings.push(format!(
            "citing-file chain {} of {MAX_CITATION_DEPTH} is above 75% of the limit",
            d.chain
        ));
    }
    if let (Some(e), Ok(l)) = (&entry, &lib) {
        d.entry = Some(e.id.clone());
        d.alt_exit = compare_entry(root, l, e, &node, &mut d.stale, &mut d.statement_records);
        // History notes travel in the same list; they are notes, not records.
        let (notes, recs): (Vec<_>, Vec<_>) = std::mem::take(&mut d.statement_records)
            .into_iter()
            .partition(|(r, _, _)| r.starts_with("history "));
        d.statement_records = recs;
        d.notes = notes
            .into_iter()
            .map(|(r, m, _)| format!("{r}: {m}"))
            .collect();
    }
    d.state = if lib.is_err() {
        "library invalid".into()
    } else if d.stale.is_empty() {
        "current".into()
    } else {
        "stale".into()
    };
    d.exit = match node.kernel.exit {
        MALFORMED => MALFORMED,
        _ if lib.is_err() => MALFORMED,
        REJECT => REJECT,
        _ if d.alt_exit != ACCEPT => d.alt_exit,
        _ if !d.stale.is_empty() => STALE,
        _ => ACCEPT,
    };
    d.root = Some(node);
    d
}

// ---------------------------------------------------------------------------
// `rlt deps --verify-lib`.

#[derive(Clone, Debug)]
pub struct LibReport {
    pub problems: Vec<String>,
    /// `(id, exit, state, stale items)`.
    pub entries: Vec<(String, i32, String, Vec<StaleItem>)>,
    pub exit: i32,
}

fn worse(a: i32, b: i32) -> i32 {
    let rank = |c: i32| match c {
        MALFORMED => 3,
        REJECT => 2,
        STALE => 1,
        _ => 0,
    };
    if rank(b) > rank(a) {
        b
    } else {
        a
    }
}

pub fn verify_lib(root: &Path) -> LibReport {
    let lib = match load_library(root) {
        Ok(l) => l,
        Err(e) => {
            return LibReport {
                problems: vec![e],
                entries: vec![],
                exit: MALFORMED,
            };
        }
    };
    let mut r = LibReport {
        problems: vec![],
        entries: vec![],
        exit: ACCEPT,
    };
    for (_, e) in &lib.entries {
        let d = deps(root, &e.id);
        r.exit = worse(r.exit, d.exit);
        let state = if !d.problems.is_empty() {
            format!("invalid: {}", d.problems.join("; "))
        } else {
            let v = d
                .root
                .as_ref()
                .map(|n| n.kernel.verdict.clone())
                .unwrap_or_default();
            format!("{v} · {}", d.state)
        };
        r.entries.push((e.id.clone(), d.exit, state, d.stale));
    }
    r
}

// ---------------------------------------------------------------------------
// Rendering: text and JSON from the same structures.

fn exit_meaning(code: i32) -> &'static str {
    match code {
        ACCEPT => "0 — every node checked and accepted; the library index valid and the entry, if any, current",
        REJECT => "1 — the checker rejects a proof in the graph or an alternative proof of the entry",
        STALE => "3 — the graph checks, but the library entry differs from the checked files",
        _ => "2 — invalid input, an invalid library index, or a citation or limit failure as `rlt check` reports it",
    }
}

fn node_lines(t: &mut String, n: &Node, prefix: &str, last: bool, top: bool) {
    let (branch, cont) = if top {
        ("", "")
    } else if last {
        ("└─ ", "   ")
    } else {
        ("├─ ", "│  ")
    };
    let k = &n.kernel;
    let head = format!(
        "{}{}{}  ({})  {}",
        n.label
            .as_ref()
            .map(|l| format!("{l}  "))
            .unwrap_or_default(),
        n.lib_id.clone().unwrap_or_else(|| "(unlisted)".into()),
        "",
        k.proof_id.as_deref().unwrap_or("—"),
        n.path
    );
    let _ = writeln!(t, "{prefix}{branch}{}", one(&head));
    let inner = format!(
        "{prefix}{cont}{}",
        if n.children.is_empty() {
            "   "
        } else {
            "│  "
        }
    );
    if let Some(p) = &n.problem {
        let what = if n.checked {
            "citation refused"
        } else {
            "citation failed"
        };
        let _ = writeln!(t, "{inner}{}", one(&format!("{what}: {p}")));
        if !n.checked {
            return;
        }
    }
    let _ = writeln!(
        t,
        "{inner}{}",
        one(&format!(
            "{} · sha256 {} · {} · cites {}",
            k.verdict,
            n.sha256,
            k.assumptions.as_deref().unwrap_or("Γ not reported"),
            n.children.len()
        ))
    );
    if let Some(g) = &k.proved {
        let _ = writeln!(t, "{inner}{}", one(&format!("proved  {g}")));
    } else if k.verdict != "ACCEPT" {
        let _ = writeln!(t, "{inner}{}", one(&format!("checker {}", k.verdict_line)));
    }
    let _ = writeln!(
        t,
        "{inner}{}",
        one(&format!(
            "axioms  {}{}",
            k.axioms_used.as_deref().unwrap_or("—"),
            if k.definitions_used.is_empty() {
                String::new()
            } else {
                format!(" · definitions {}", k.definitions_used.join(", "))
            }
        ))
    );
    if let Some(f) = &n.fingerprint {
        let _ = writeln!(t, "{inner}{}", one(&format!("fingerprint {f}")));
    }
    let count = n.children.len();
    for (i, c) in n.children.iter().enumerate() {
        node_lines(t, c, &format!("{prefix}{cont}"), i + 1 == count, false);
    }
}

pub fn render_text(d: &Deps) -> String {
    let mut t = String::new();
    row(&mut t, "RLT deps", &d.target);
    row(&mut t, "library", &d.library);
    if !d.problems.is_empty() {
        for p in &d.problems {
            row(&mut t, "  problem", p);
        }
        row(&mut t, "verdict", "invalid — nothing further is reported");
        row(&mut t, "exit", exit_meaning(d.exit));
        return t;
    }
    let Some(n) = &d.root else { return t };
    let k = &n.kernel;
    row(
        &mut t,
        "proof",
        &format!("{} ({})", n.path, k.proof_id.as_deref().unwrap_or("—")),
    );
    row(&mut t, "sha256", &n.sha256);
    for (label, v) in [
        ("written goal", &k.written_goal),
        ("convention", &k.convention),
        ("checked goal", &k.checked_goal),
        ("axiom set", &k.axiom_set),
        ("assumptions", &k.assumptions),
    ] {
        if let Some(v) = v {
            row(&mut t, label, v);
        }
    }
    if let Some(f) = &n.fingerprint {
        row(&mut t, "fingerprint", f);
        for l in n.fingerprint_text.as_deref().unwrap_or("").lines() {
            row(&mut t, "  of", l);
        }
    }
    row(&mut t, "checker", &k.verdict_line);
    if let Some(id) = &d.entry {
        row(&mut t, "library entry", &format!("{id} · {}", d.state));
        for s in &d.stale {
            row(
                &mut t,
                "  stale",
                &format!("{}: entry {} · current {}", s.what, s.record, s.current),
            );
        }
    } else if let Some(id) = &n.lib_id {
        row(&mut t, "library entry", id);
    }
    let _ = writeln!(
        t,
        "\ndependency graph (from the citation trace of this one check)"
    );
    node_lines(&mut t, n, "", true, true);
    let _ = writeln!(t);
    row(
        &mut t,
        "theorems",
        &format!("{} distinct · {} cited-file checks", d.distinct, d.checks),
    );
    row(
        &mut t,
        "axioms",
        &format!("{} (union over the graph)", d.axioms.join(", ")),
    );
    row(
        &mut t,
        "definitions",
        &if d.definitions.is_empty() {
            "none".to_string()
        } else {
            format!(
                "{} ({} sha256 {})",
                d.definitions.join(", "),
                PINNED_DEFINITIONS.0,
                PINNED_DEFINITIONS.1
            )
        },
    );
    row(
        &mut t,
        "assumptions",
        "accepted citations carry Γ = ∅, so none cross a citation; the root's Γ is shown above",
    );
    let over = d.attempted > MAX_CITED_CHECKS || d.chain > MAX_CITATION_DEPTH;
    row(
        &mut t,
        "budget",
        &format!(
            "cited-file checks {} of {MAX_CITED_CHECKS} · citing-file chain {} of {MAX_CITATION_DEPTH}{}",
            d.attempted,
            d.chain,
            if over { " — limit exceeded; the checker refused the file" } else { "" }
        ),
    );
    for w in &d.warnings {
        row(&mut t, "  warning", w);
    }
    for n in &d.notes {
        row(&mut t, "note", n);
    }
    for (r, s, x) in &d.statement_records {
        row(
            &mut t,
            "statement",
            &format!("{r} (rlt explain: {s}, exit {x})"),
        );
    }
    row(&mut t, "exit", exit_meaning(d.exit));
    t
}

fn node_json(n: &Node) -> Json {
    let k = &n.kernel;
    let s = |v: &Option<String>| {
        v.as_ref()
            .map(|x| Json::Str(x.clone()))
            .unwrap_or(Json::Null)
    };
    obj(vec![
        ("label", s(&n.label)),
        ("library_id", s(&n.lib_id)),
        ("path", Json::Str(n.path.clone())),
        ("sha256", Json::Str(n.sha256.clone())),
        ("proof_id", s(&k.proof_id)),
        ("verdict", Json::Str(k.verdict.clone())),
        ("verdict_line", Json::Str(k.verdict_line.clone())),
        ("written_goal", s(&k.written_goal)),
        ("checked_goal", s(&k.checked_goal)),
        ("proved", s(&k.proved)),
        ("convention", s(&k.convention)),
        ("axiom_set", s(&k.axiom_set)),
        ("axiom_set_sha256", s(&k.axiom_set_sha256)),
        ("assumptions", s(&k.assumptions)),
        (
            "assumption_pairs",
            match &k.assumption_pairs {
                Some(v) => Json::Arr(
                    v.iter()
                        .map(|(n, f)| {
                            obj(vec![
                                ("name", Json::Str(n.clone())),
                                ("formula", Json::Str(f.clone())),
                            ])
                        })
                        .collect(),
                ),
                None => Json::Null,
            },
        ),
        ("axioms_used", s(&k.axioms_used)),
        ("rules_used", s(&k.rules_used)),
        (
            "definitions_used",
            Json::Arr(
                k.definitions_used
                    .iter()
                    .map(|x| Json::Str(x.clone()))
                    .collect(),
            ),
        ),
        ("fingerprint", s(&n.fingerprint)),
        ("fingerprint_text", s(&n.fingerprint_text)),
        ("checked", Json::Bool(n.checked)),
        ("problem", s(&n.problem)),
        (
            "children",
            Json::Arr(n.children.iter().map(node_json).collect()),
        ),
    ])
}

pub fn to_json(d: &Deps) -> Json {
    let strs = |v: &[String]| Json::Arr(v.iter().map(|x| Json::Str(x.clone())).collect());
    obj(vec![
        ("format", Json::Str("rlt-deps/1".into())),
        ("target", Json::Str(d.target.clone())),
        ("library", Json::Str(d.library.clone())),
        (
            "entry",
            d.entry.clone().map(Json::Str).unwrap_or(Json::Null),
        ),
        ("state", Json::Str(d.state.clone())),
        ("problems", strs(&d.problems)),
        (
            "stale",
            Json::Arr(
                d.stale
                    .iter()
                    .map(|s| {
                        obj(vec![
                            ("what", Json::Str(s.what.clone())),
                            ("entry", Json::Str(s.record.clone())),
                            ("current", Json::Str(s.current.clone())),
                        ])
                    })
                    .collect(),
            ),
        ),
        ("root", d.root.as_ref().map(node_json).unwrap_or(Json::Null)),
        (
            "summary",
            obj(vec![
                ("distinct_theorems", Json::Int(d.distinct as i64)),
                ("cited_file_checks", Json::Int(d.checks as i64)),
                ("cited_file_checks_attempted", Json::Int(d.attempted as i64)),
                ("cited_file_check_limit", Json::Int(MAX_CITED_CHECKS as i64)),
                ("citing_file_chain", Json::Int(d.chain as i64)),
                (
                    "citing_file_chain_limit",
                    Json::Int(MAX_CITATION_DEPTH as i64),
                ),
                ("axioms", strs(&d.axioms)),
                ("definitions", strs(&d.definitions)),
                ("warnings", strs(&d.warnings)),
                ("notes", strs(&d.notes)),
            ]),
        ),
        (
            "statement_records",
            Json::Arr(
                d.statement_records
                    .iter()
                    .map(|(r, s, x)| {
                        obj(vec![
                            ("record", Json::Str(r.clone())),
                            ("explain", Json::Str(s.clone())),
                            ("explain_exit", Json::Int(*x as i64)),
                        ])
                    })
                    .collect(),
            ),
        ),
        ("exit", Json::Int(d.exit as i64)),
        ("exit_meaning", Json::Str(exit_meaning(d.exit).into())),
    ])
}

pub fn render_json(d: &Deps) -> String {
    let mut s = String::new();
    json_write(&to_json(d), 0, &mut s);
    s.push('\n');
    s
}

pub fn render_lib_text(r: &LibReport) -> String {
    let mut t = String::new();
    row(&mut t, "RLT deps", "--verify-lib");
    for p in &r.problems {
        row(&mut t, "  problem", p);
    }
    for (id, x, state, stale) in &r.entries {
        row(&mut t, "entry", &format!("{id} · exit {x} · {state}"));
        for s in stale {
            row(
                &mut t,
                "  stale",
                &format!("{}: entry {} · current {}", s.what, s.record, s.current),
            );
        }
    }
    let ok = r.entries.iter().filter(|e| e.1 == ACCEPT).count();
    row(
        &mut t,
        "entries",
        &format!("{ok} of {} current and accepted", r.entries.len()),
    );
    row(&mut t, "exit", exit_meaning(r.exit));
    t
}

pub fn render_lib_json(r: &LibReport) -> String {
    let v = obj(vec![
        ("format", Json::Str("rlt-deps-lib/1".into())),
        (
            "problems",
            Json::Arr(r.problems.iter().map(|x| Json::Str(x.clone())).collect()),
        ),
        (
            "entries",
            Json::Arr(
                r.entries
                    .iter()
                    .map(|(id, x, s, st)| {
                        obj(vec![
                            ("id", Json::Str(id.clone())),
                            ("exit", Json::Int(*x as i64)),
                            ("state", Json::Str(s.clone())),
                            (
                                "stale",
                                Json::Arr(
                                    st.iter()
                                        .map(|s| {
                                            obj(vec![
                                                ("what", Json::Str(s.what.clone())),
                                                ("entry", Json::Str(s.record.clone())),
                                                ("current", Json::Str(s.current.clone())),
                                            ])
                                        })
                                        .collect(),
                                ),
                            ),
                        ])
                    })
                    .collect(),
            ),
        ),
        ("exit", Json::Int(r.exit as i64)),
    ]);
    let mut s = String::new();
    json_write(&v, 0, &mut s);
    s.push('\n');
    s
}

//! Reports for `rlt check`, `rlt written`, `rlt rp` and `rlt validate`.
//! Every report is a deterministic string so tests can compare it with a
//! golden file.

pub mod deps;
pub mod explain;
pub mod gen;
pub mod residues;
use rlt_kernel::{check, Axiom, Formula, Mode, Theorem, Verdict};
use rlt_syntax::elab::{elaborate_formula, elaborate_term, Convention};
use rlt_syntax::proof::{parse_axiom_file, ProofFile};
use rlt_syntax::records::{
    load_conventions, load_readingpoint, load_registry, load_status, load_yaml, read_regular,
    ConventionRecord, EvidenceItem, ReadingPoint, Relationship, Source, StatusVocabulary,
};
use rlt_syntax::written::Written;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// Exit codes.
pub const ACCEPT: i32 = 0;
pub const REJECT: i32 = 1;
pub const MALFORMED: i32 = 2;

pub struct Report {
    pub text: String,
    pub code: i32,
}

pub(crate) fn malformed(text: &mut String, msg: String) -> Report {
    let _ = writeln!(text, "verdict         MALFORMED — {msg}");
    Report {
        text: std::mem::take(text),
        code: MALFORMED,
    }
}

/// A path for display: relative to the root, with any control character
/// escaped, so a file name can never add lines to a report.
fn shown_path(root: &Path, p: &Path) -> String {
    let s = p.strip_prefix(root).unwrap_or(p).display().to_string();
    s.chars()
        .map(|c| {
            if c.is_control() {
                c.escape_default().to_string()
            } else {
                c.to_string()
            }
        })
        .collect()
}

pub(crate) fn rp_path(root: &Path, id: &str) -> PathBuf {
    root.join("readingpoints").join(format!("{id}.yaml"))
}

fn tokens(w: &Written) -> String {
    w.tokens.join(" · ")
}

fn gamma(th: &Theorem) -> String {
    if th.assumptions().is_empty() {
        "Γ = ∅".to_string()
    } else {
        let parts: Vec<String> = th
            .assumptions()
            .iter()
            .map(|(n, f)| format!("{n} : {f}"))
            .collect();
        format!("Γ = {{ {} }}", parts.join(", "))
    }
}

/// A registered axiom set, loaded only through `load_axiom_set`.
pub struct AxiomSet {
    pub id: String,
    pub file: String,
    pub sha256: String,
    pub source: String,
    pub axioms: Vec<Axiom>,
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Axiom sets approved for RLT-001, pinned in the binary: (id, file, SHA-256).
/// `spec/axioms.yaml` must agree with this table exactly. Changing an axiom
/// therefore requires changing this source file, which is reviewed like the
/// kernel. The pin binds content; the axioms remain assumptions.
pub const PINNED_AXIOM_SETS: &[(&str, &str, &str)] = &[
    (
        "PEANO-ADD",
        "spec/peano.rlt",
        "1a3f83444a06fdfbd02c9af2a5881b47137ba102a675a4e012c84be477b10a0d",
    ),
    // RLT-003 M1: Robinson's Q, axioms Q4–Q7 (source OLP-ROBINSON-Q).
    (
        "PEANO-ARITH",
        "spec/peano-arith.rlt",
        "00ceffa9a60b53286f350edbf2f1a5f48a14169a9b09a82cbedc6f9956ff01e4",
    ),
    // RLT-003 M3b: Robinson's Q, axioms Q1–Q7, self-contained; Q4–Q7 are
    // checked identical to PEANO-ARITH (source OLP-ROBINSON-Q).
    (
        "ROBINSON-Q",
        "spec/robinson-q.rlt",
        "4ad55192222576e3901abc4ee7ea5a6760b64ff212e8c55796ab9847e423b00c",
    ),
];

/// RLT-003 M3b: axiom sets whose equations must equal another set's axioms
/// of the same name, in variables and formula: (set, reference set, names).
pub const SHARED_AXIOMS: &[(&str, &str, &[&str])] = &[(
    "ROBINSON-Q",
    "PEANO-ARITH",
    &["add_zero", "add_succ", "mul_zero", "mul_succ"],
)];

/// RLT-003 M3b: each named axiom of `id` must equal the reference set's
/// axiom of the same name, in variables and body.
pub fn shared_axiom_problem(
    id: &str,
    axioms: &[rlt_kernel::Axiom],
    reference: &str,
    reference_axioms: &[rlt_kernel::Axiom],
    names: &[&str],
) -> Result<(), String> {
    for n in names {
        let mine = axioms.iter().find(|a| a.name() == *n);
        let theirs = reference_axioms.iter().find(|a| a.name() == *n);
        match (mine, theirs) {
            (Some(a), Some(b)) if a.vars() == b.vars() && a.body() == b.body() => {}
            _ => {
                return Err(format!(
                    "axiom set {id}: {n} differs from {reference}'s {n}; shared axioms keep one meaning"
                ))
            }
        }
    }
    Ok(())
}

/// RLT-003 M3b: every definition cites a registered reference or available
/// source, or declares origin rlt-authored.
pub fn definition_source_problem(
    defs: &rlt_syntax::defs::Definitions,
    reg: &BTreeMap<String, rlt_syntax::records::Source>,
) -> Result<(), String> {
    for def in &defs.list {
        if let Some(src) = &def.source {
            match reg.get(src) {
                Some(s) if AXIOM_SOURCE_STATUS.contains(&s.status.as_str()) => {}
                _ => {
                    return Err(format!(
                        "definition `{}` cites `{src}`, which is not a registered reference or available source",
                        def.name
                    ))
                }
            }
        }
    }
    Ok(())
}

/// A GitHub file location at a commit: `https://github.com/OWNER/REPO/blob/HEX/PATH`
/// (RLT-003 M3b: any owner, so the Open Logic Project and Lean core can be
/// checked by excerpt like the thinkthoughts repositories).
fn github_file_at_commit(loc: &str) -> bool {
    let Some(rest) = loc.strip_prefix("https://github.com/") else {
        return false;
    };
    let parts: Vec<&str> = rest.splitn(5, '/').collect();
    matches!(parts.as_slice(), [owner, repo, "blob", commit, path]
        if !owner.is_empty() && !repo.is_empty() && !path.is_empty()
            && commit.len() >= 7 && commit.chars().all(|c| c.is_ascii_hexdigit()))
}

/// RLT-003 M3b: the pinned definitions file (file, SHA-256).
pub const PINNED_DEFINITIONS: (&str, &str) = (
    "spec/definitions.rlt",
    "122e37fc627b136023a1a5ffac0344162ed77ae95312248a9f6a76c2b639ecda",
);

/// Load the pinned definitions. `spec/axioms.yaml` must name the pinned file
/// and hash under `definitions`; the file must be regular and hash to the pin;
/// every definition must cite a registered source or declare origin
/// rlt-authored; no name may collide with a rule or a registered axiom.
pub fn load_definitions(root: &Path) -> Result<rlt_syntax::defs::Definitions, String> {
    let (pin_file, pin_sha) = PINNED_DEFINITIONS;
    let y = load_yaml(&root.join("spec/axioms.yaml"))?;
    let d = &y["definitions"];
    let (file, sha) = match (d["file"].as_str(), d["sha256"].as_str()) {
        (Some(f), Some(h)) => (f.to_string(), h.to_string()),
        _ => return Err("spec/axioms.yaml: `definitions` must give `file` and `sha256`".into()),
    };
    if file != pin_file || sha != pin_sha {
        return Err(format!(
            "definitions: registered {file} {sha} differ from the pinned {pin_file} {pin_sha}"
        ));
    }
    for p in [root.join("spec"), root.join(&file)] {
        let meta = std::fs::symlink_metadata(&p)
            .map_err(|e| format!("definitions: {file}: {}", e.kind()))?;
        if meta.file_type().is_symlink() {
            return Err(format!("definitions: {file} must be a regular file"));
        }
    }
    let bytes = std::fs::read(root.join(&file))
        .map_err(|e| format!("definitions: {file}: {}", e.kind()))?;
    let actual = sha256_hex(&bytes);
    if actual != pin_sha {
        return Err(format!(
            "definitions: {file} has SHA-256 {actual}, pinned {pin_sha}; the definitions changed without review"
        ));
    }
    let text = String::from_utf8(bytes).map_err(|_| format!("definitions: {file} is not UTF-8"))?;
    let mut forbidden: Vec<String> = rlt_kernel::RULES.iter().map(|r| r.to_string()).collect();
    for (id, _, _) in PINNED_AXIOM_SETS {
        if let Ok(set) = load_axiom_set(root, id) {
            forbidden.extend(set.axioms.iter().map(|a| a.name().to_string()));
        }
    }
    let f: Vec<&str> = forbidden.iter().map(String::as_str).collect();
    let defs = rlt_syntax::defs::parse_definitions(&text, &f)?;
    let reg = pinned_registry(root)?;
    definition_source_problem(&defs, &reg)?;
    Ok(defs)
}

/// Source statuses an axiom set may cite.
const AXIOM_SOURCE_STATUS: [&str; 2] = ["available", "reference"];

/// Load a registered axiom set by id. The id must be pinned in
/// `PINNED_AXIOM_SETS`; `spec/axioms.yaml` must hold exactly one entry for it,
/// naming the pinned file and hash; the file must be a regular file (no
/// symlink) whose bytes hash to the pinned value; its axiom names must equal
/// the entry's list; every axiom must cite the set's registered source.
pub fn load_axiom_set(root: &Path, id: &str) -> Result<AxiomSet, String> {
    let Some(&(_, pin_file, pin_sha)) = PINNED_AXIOM_SETS.iter().find(|(p, _, _)| *p == id) else {
        return Err(format!("axiom set `{id}` is not pinned in this rlt binary"));
    };
    let y = load_yaml(&root.join("spec/axioms.yaml"))?;
    let sets = y["sets"]
        .as_vec()
        .ok_or("spec/axioms.yaml: `sets` must be a list")?;
    let entries: Vec<_> = sets
        .iter()
        .filter(|e| e["id"].as_str() == Some(id))
        .collect();
    let entry = match entries.as_slice() {
        [one] => *one,
        [] => {
            return Err(format!(
                "axiom set `{id}` is unregistered (see spec/axioms.yaml)"
            ))
        }
        _ => {
            return Err(format!(
                "axiom set `{id}` is registered {} times",
                entries.len()
            ))
        }
    };
    let field = |k: &str| {
        entry[k]
            .as_str()
            .map(str::to_string)
            .ok_or(format!("axiom set {id}: `{k}` must be a string"))
    };
    let (file, sha, source) = (field("file")?, field("sha256")?, field("source")?);
    if file != pin_file {
        return Err(format!(
            "axiom set {id}: file `{file}` differs from the pinned `{pin_file}`"
        ));
    }
    if sha != pin_sha {
        return Err(format!(
            "axiom set {id}: registered SHA-256 {sha} differs from the pinned {pin_sha}"
        ));
    }
    for p in [root.join("spec"), root.join(&file)] {
        let meta = std::fs::symlink_metadata(&p)
            .map_err(|e| format!("axiom set {id}: {}: {}", file, e.kind()))?;
        if meta.file_type().is_symlink() {
            return Err(format!(
                "axiom set {id}: {} is a symbolic link; axiom files must be regular files",
                p.strip_prefix(root).unwrap_or(&p).display()
            ));
        }
    }
    let bytes = std::fs::read(root.join(&file))
        .map_err(|e| format!("axiom set {id}: {file}: {}", e.kind()))?;
    let actual = sha256_hex(&bytes);
    if actual != pin_sha {
        return Err(format!(
            "axiom set {id}: {file} has SHA-256 {actual}, pinned {pin_sha}; the axiom file changed without review"
        ));
    }
    let text =
        String::from_utf8(bytes).map_err(|_| format!("axiom set {id}: {file} is not UTF-8"))?;
    let axioms = parse_axiom_file(&text)?;
    let listed: Vec<String> = entry["axioms"]
        .as_vec()
        .ok_or(format!("axiom set {id}: `axioms` must be a list"))?
        .iter()
        .map(|x| {
            x.as_str()
                .map(str::to_string)
                .ok_or(format!("axiom set {id}: `axioms` must list names"))
        })
        .collect::<Result<_, _>>()?;
    let found: Vec<String> = axioms.iter().map(|a| a.name().to_string()).collect();
    if listed != found {
        return Err(format!(
            "axiom set {id}: registered names {listed:?}, file holds {found:?}"
        ));
    }
    let reg = pinned_registry(root)?;
    match reg.get(&source) {
        Some(s) if AXIOM_SOURCE_STATUS.contains(&s.status.as_str()) => {}
        _ => {
            return Err(format!(
            "axiom set {id}: source `{source}` is not a registered reference or available source"
        ))
        }
    }
    for a in &axioms {
        if a.source() != source {
            return Err(format!(
                "axiom set {id}: axiom {} cites `{}`, the set's source is `{source}`",
                a.name(),
                a.source()
            ));
        }
    }
    // RLT-003 M3b: shared axioms must be identical to the reference set's.
    for (set, reference, names) in SHARED_AXIOMS {
        if *set != id {
            continue;
        }
        let other = load_axiom_set(root, reference)
            .map_err(|e| format!("axiom set {id}: reference set {reference}: {e}"))?;
        shared_axiom_problem(id, &axioms, reference, &other.axioms, names)?;
    }
    Ok(AxiomSet {
        id: id.to_string(),
        file,
        sha256: sha,
        source,
        axioms,
    })
}

/// Everything `check_file` learned, for callers that bind proofs to records.
pub struct CheckOutcome {
    pub report: Report,
    pub proof: Option<ProofFile>,
    /// The checked goal when it is an equation (records bind equations).
    pub goal: Option<Formula>,
    /// The checked goal as a formula (RLT-003 M3a).
    pub prop: Option<rlt_kernel::Prop>,
    pub theorem: Option<Theorem>,
    /// RLT-004c: the citations this check resolved, in the file's order, each
    /// with the outcome of checking the cited bytes. Recorded during the one
    /// check `rlt check` performs; reports and verdicts are unaffected.
    pub citations: Vec<CitationTrace>,
}

/// One citation as `check_inner` resolved it (RLT-004c, `rlt deps`).
pub struct CitationTrace {
    pub label: String,
    /// The path exactly as written in the citing file.
    pub path: String,
    /// The SHA-256 the citing file records.
    pub sha256: String,
    /// The outcome of checking the cited bytes; `None` when the citation
    /// failed before its file was checked (path, cycle, hash, budget).
    pub outcome: Option<Box<CheckOutcome>>,
    /// Why the citation was refused, when it was: before its file was
    /// checked (`outcome` is `None`) or after (accepted file, but refused
    /// as a citation, e.g. conditional or another axiom set).
    pub problem: Option<String>,
}

/// `rlt check FILE`: replay one proof file and report every layer separately.
pub fn check_file(root: &Path, file: &Path) -> Report {
    check_file_outcome(root, file).report
}

pub fn check_file_outcome(root: &Path, file: &Path) -> CheckOutcome {
    check_file_outcome_with(root, file, result_kind)
}

/// As `check_file_outcome`, with the result-kind label supplied by the caller.
/// The label is computed only after the kernel accepts; a test swaps it to
/// confirm that no verdict depends on it (RLT-003 M3a obligation O9).
pub fn check_file_outcome_with(
    root: &Path,
    file: &Path,
    classify: fn(&rlt_kernel::Prop) -> &'static str,
) -> CheckOutcome {
    let budget = std::cell::Cell::new(MAX_CITED_CHECKS);
    check_inner(root, file, classify, &[], None, &budget)
}

/// As `check_file_outcome`, on bytes the caller already read (RLT-004b):
/// `rlt explain` hashes the proof file once and checks exactly those bytes,
/// through the same function `rlt check` uses.
pub fn check_preread_outcome(root: &Path, file: &Path, src: String) -> CheckOutcome {
    let budget = std::cell::Cell::new(MAX_CITED_CHECKS);
    check_inner(root, file, result_kind, &[], Some(src), &budget)
}

/// As `check_preread_outcome`, from raw bytes (RLT-004b audit): bytes that
/// are not UTF-8 get exactly the report `rlt check` gives for them, built from
/// these bytes without reading the file again.
pub fn check_bytes_outcome(root: &Path, file: &Path, bytes: Vec<u8>) -> CheckOutcome {
    match String::from_utf8(bytes) {
        Ok(src) => check_preread_outcome(root, file, src),
        Err(_) => {
            let mut t = String::new();
            let _ = writeln!(t, "RLT check       {}", shown_path(root, file));
            let msg = if file.to_string_lossy().chars().any(|c| c.is_control()) {
                "the proof file name contains a control character".to_string()
            } else {
                format!(
                    "cannot read the proof file: {}",
                    std::io::Error::from(std::io::ErrorKind::InvalidData).kind()
                )
            };
            CheckOutcome {
                report: malformed(&mut t, msg),
                proof: None,
                goal: None,
                prop: None,
                theorem: None,
                citations: Vec::new(),
            }
        }
    }
}

/// Most cited-file checks one top-level check may run, counted over the
/// whole citation tree (RLT-003 M3c audit note 4): each citation re-checks
/// its file from scratch, so this bounds the work as well as the depth.
pub const MAX_CITED_CHECKS: usize = 64;

/// Longest chain of citations followed from one proof file (RLT-003 M3c).
pub const MAX_CITATION_DEPTH: usize = 8;

/// A citation path: repository-relative, `.rlt`, made only of plain
/// components (no `..`, `.`, root or prefix), with no symlink at any level
/// and a regular file at the end (RLT-003 M3c).
pub fn citation_path(root: &Path, p: &str) -> Result<PathBuf, String> {
    use std::path::Component;
    if p.is_empty() || p.chars().any(|c| c.is_control() || c == '\\') {
        return Err("the cited path must be a plain repository-relative path".into());
    }
    let rel = Path::new(p);
    // The raw spelling is checked too (M3c audit note 1): `Path::components`
    // would silently drop interior `.` and empty components.
    let plain = p.split('/').all(|c| !c.is_empty() && c != "." && c != "..");
    if !p.ends_with(".rlt")
        || !plain
        || rel.components().any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(format!(
            "cited path `{p}` must be a repository-relative .rlt path of plain components: no `..`, `.`, empty component or leading `/`"
        ));
    }
    let mut cur = root.to_path_buf();
    for c in rel.components() {
        cur.push(c);
        let meta = std::fs::symlink_metadata(&cur)
            .map_err(|e| format!("cited path `{p}`: {}", e.kind()))?;
        if meta.file_type().is_symlink() {
            return Err(format!(
                "cited path `{p}` must be a regular file in the repository (a symlink was found)"
            ));
        }
    }
    if !std::fs::symlink_metadata(&cur)
        .map(|m| m.is_file())
        .unwrap_or(false)
    {
        return Err(format!(
            "cited path `{p}` must be a regular file in the repository"
        ));
    }
    Ok(cur)
}

/// Check one proof file. `chain` holds the canonical paths of the files
/// citing it (for cycle refusal); `preread` holds the exact bytes a citing
/// file hashed, so the theorem cited is the result of checking those bytes.
fn check_inner(
    root: &Path,
    file: &Path,
    classify: fn(&rlt_kernel::Prop) -> &'static str,
    chain: &[PathBuf],
    preread: Option<String>,
    budget: &std::cell::Cell<usize>,
) -> CheckOutcome {
    let mut t = String::new();
    let mut out = CheckOutcome {
        report: Report {
            text: String::new(),
            code: MALFORMED,
        },
        proof: None,
        goal: None,
        prop: None,
        theorem: None,
        citations: Vec::new(),
    };
    macro_rules! bail {
        ($msg:expr) => {{
            out.report = malformed(&mut t, $msg);
            return out;
        }};
    }
    let shown = shown_path(root, file);
    let _ = writeln!(t, "RLT check       {shown}");
    if file.to_string_lossy().chars().any(|c| c.is_control()) {
        bail!("the proof file name contains a control character".to_string());
    }
    let src = match preread {
        Some(s) => s,
        None => match read_regular(file) {
            Ok(s) => s,
            Err(e) => bail!(format!(
                "cannot read the proof file: {}",
                e.rsplit(": ").next().unwrap_or("unreadable")
            )),
        },
    };
    // RLT-003 M3b: the pinned definitions, loaded for NAT-DEF files. M3c (M3b
    // audit note 1): the decision rests on the parsed `convention` field,
    // never on the raw text.
    let defs = rlt_syntax::proof::reads_definitions(&src)
        .then(|| load_definitions(root).map(std::rc::Rc::new));
    let pf = match rlt_syntax::proof::parse_proof_file_with(&src, defs.clone()) {
        Ok(p) => p,
        Err(e) => bail!(e),
    };
    let _ = writeln!(t, "proof id        {}", pf.id);

    // Reading-point specification: documented meanings, verbatim.
    let rp: Option<ReadingPoint> = match &pf.readingpoint {
        Some(id) => match load_readingpoint(&rp_path(root, id)) {
            Ok(r) => Some(r),
            Err(e) => bail!(e),
        },
        None => None,
    };
    if let Some(rp) = &rp {
        let _ = writeln!(t, "reading point   {}", rp.id);
        for m in &rp.meanings {
            let _ = writeln!(t, "  meaning       \"{}\"  [{}]", m.text, m.source);
        }
    }
    if let Some(r) = &pf.relationship {
        let _ = writeln!(t, "relationship    {r}");
    }
    if let Some((name, kind)) = &pf.context {
        let _ = writeln!(t, "context         {name} ({kind})");
    }

    // Layer 1: written expression.
    let _ = writeln!(t, "written goal    \"{}\"", pf.goal.text);
    let _ = writeln!(t, "  tokens        {}", tokens(&pf.goal));
    for (n, w) in &pf.assumptions {
        let _ = writeln!(t, "written assume  {n} : \"{}\"", w.text);
    }

    // Layer 2: convention (an input to interpretation).
    let convs = match pinned_conventions(root) {
        Ok(c) => c,
        Err(e) => bail!(e),
    };
    let Some(conv) = convs.get(&pf.convention) else {
        bail!(format!("convention `{}` is unregistered", pf.convention));
    };
    let _ = writeln!(
        t,
        "convention      {} (authority: {})",
        conv.convention.id, conv.authority
    );
    let mut convention = conv.convention.clone();
    if !convention.definitions.is_empty() {
        match &defs {
            Some(Ok(d)) => {
                for n in &convention.definitions {
                    let known = d.list.iter().any(|x| x.shown() == *n);
                    if !known {
                        bail!(format!(
                            "convention {}: notation {n:?} has no registered definition",
                            convention.id
                        ));
                    }
                }
                convention.defs = Some(d.clone());
            }
            Some(Err(e)) => bail!(e.clone()),
            None => bail!("definitions: unavailable".to_string()),
        }
    }
    let mut po = match pf.to_proof_object(&convention) {
        Ok(p) => p,
        Err(e) => bail!(e),
    };

    // Layer 3: the exact statement the kernel checks.
    let _ = writeln!(t, "checked goal    {}", po.goal);
    let _ = writeln!(t, "  kernel form   {}", po.goal.kernel_form());
    // RLT-003 M3b: the definitions expanded anywhere in this file, and the
    // folded goal as written; the kernel checks only the unfolded formula.
    if let Some(Ok(d)) = &defs {
        let used: Vec<String> = d.used.borrow().iter().cloned().collect();
        if !used.is_empty() {
            let _ = writeln!(t, "  folded        {}", pf.goal.text.escape_debug());
            let _ = writeln!(
                t,
                "definitions     {} — abbreviations expanded before the kernel ({} sha256 {})",
                used.join(", "),
                PINNED_DEFINITIONS.0,
                PINNED_DEFINITIONS.1
            );
        }
    }
    for (n, f) in &po.assumptions {
        let _ = writeln!(t, "declared        {n} : {f}");
        let _ = writeln!(t, "  kernel form   {}", f.kernel_form());
    }
    let _ = writeln!(
        t,
        "mode            {}",
        match po.mode {
            Mode::Unconditional => "unconditional (requires Γ = ∅)",
            Mode::Conditional => "conditional (Γ ⊆ declared assumptions)",
        }
    );
    let set = match load_axiom_set(root, &pf.axiom_set) {
        Ok(s) => s,
        Err(e) => bail!(e),
    };
    let _ = writeln!(
        t,
        "axiom set       {} ({}, source {})",
        set.id, set.file, set.source
    );
    let _ = writeln!(t, "  sha256        {}", set.sha256);
    for a in &set.axioms {
        let _ = writeln!(
            t,
            "  axiom         {} : ∀ {}. {}",
            a.name(),
            a.vars().join(" "),
            // Equational axioms print exactly as before M3b.
            a.equation()
                .map(|e| e.to_string())
                .unwrap_or_else(|| a.body().to_string())
        );
    }
    let _ = writeln!(t, "  status        assumed: the SHA-256 binds these axioms' exact content to the approved file; their mathematical truth and any physical correspondence remain assumptions");
    // RLT-003 M3c: citations. Each cited file is read once, its bytes hashed
    // and compared with the citation, then checked from scratch on exactly
    // those bytes; the kernel theorem that check produces is the one cited.
    if !pf.cites.is_empty() {
        let me = match file.canonicalize() {
            Ok(p) => p,
            Err(e) => bail!(format!("cannot resolve the proof file: {}", e.kind())),
        };
        let mut here: Vec<PathBuf> = chain.to_vec();
        here.push(me);
        if here.len() > MAX_CITATION_DEPTH {
            bail!(format!(
                "citations nest deeper than {MAX_CITATION_DEPTH} files"
            ));
        }
        for c in &pf.cites {
            let l = &c.label;
            // A citation that fails before its file is checked is recorded
            // with its problem, then reported exactly as before.
            macro_rules! cite_bail {
                ($msg:expr) => {{
                    let msg: String = $msg;
                    out.citations.push(CitationTrace {
                        label: l.clone(),
                        path: c.path.clone(),
                        sha256: c.sha256.clone(),
                        outcome: None,
                        problem: Some(msg.clone()),
                    });
                    bail!(msg)
                }};
            }
            let path = match citation_path(root, &c.path) {
                Ok(p) => p,
                Err(e) => cite_bail!(format!("citation `{l}`: {e}")),
            };
            let canon = match path.canonicalize() {
                Ok(p) => p,
                Err(e) => cite_bail!(format!("citation `{l}`: {}", e.kind())),
            };
            if here.contains(&canon) {
                cite_bail!(format!(
                    "citation `{l}`: citation cycle through {}",
                    c.path.escape_debug()
                ));
            }
            let bytes = match std::fs::read(&path) {
                Ok(b) => b,
                Err(e) => cite_bail!(format!("citation `{l}`: {}", e.kind())),
            };
            let actual = sha256_hex(&bytes);
            if actual != c.sha256 {
                cite_bail!(format!(
                    "citation `{l}`: {} has SHA-256 {actual}; the citation records {}",
                    c.path.escape_debug(),
                    c.sha256
                ));
            }
            let Ok(text) = String::from_utf8(bytes) else {
                cite_bail!(format!("citation `{l}`: the cited file is not UTF-8"));
            };
            if budget.get() == 0 {
                cite_bail!(format!(
                    "citation `{l}`: the citations would check more than {MAX_CITED_CHECKS} files"
                ));
            }
            budget.set(budget.get() - 1);
            let sub = check_inner(root, &path, classify, &here, Some(text), budget);
            let sub_verdict = sub
                .report
                .text
                .lines()
                .rev()
                .find(|x| x.starts_with("verdict"))
                .unwrap_or("verdict")
                .trim_start_matches("verdict")
                .trim()
                .to_string();
            let (th, spf) = (sub.theorem.clone(), sub.proof.clone());
            out.citations.push(CitationTrace {
                label: l.clone(),
                path: c.path.clone(),
                sha256: c.sha256.clone(),
                outcome: Some(Box::new(sub)),
                problem: None,
            });
            // A refusal after the sub-check is recorded on its trace entry,
            // then reported exactly as before.
            macro_rules! traced_bail {
                ($msg:expr) => {{
                    let msg: String = $msg;
                    if let Some(tr) = out.citations.last_mut() {
                        tr.problem = Some(msg.clone());
                    }
                    bail!(msg)
                }};
            }
            let (Some(th), Some(spf)) = (th, spf) else {
                traced_bail!(format!(
                    "citation `{l}`: {} is not an accepted theorem ({sub_verdict})",
                    c.path.escape_debug()
                ));
            };
            if out
                .citations
                .last()
                .map(|t| t.outcome.as_ref().map(|o| o.report.code))
                != Some(Some(ACCEPT))
            {
                traced_bail!(format!(
                    "citation `{l}`: {} is not an accepted theorem ({sub_verdict})",
                    c.path.escape_debug()
                ));
            }
            if spf.axiom_set != pf.axiom_set {
                traced_bail!(format!(
                    "citation `{l}`: the cited theorem's axiom set {} differs from this file's {}",
                    spf.axiom_set, pf.axiom_set
                ));
            }
            if spf.convention != pf.convention {
                traced_bail!(format!(
                    "citation `{l}`: the cited theorem's convention {} differs from this file's {}",
                    spf.convention, pf.convention
                ));
            }
            if !th.is_unconditional() {
                traced_bail!(format!(
                    "citation `{l}`: the cited theorem is conditional (Γ ≠ ∅)"
                ));
            }
            let used: Vec<&str> = th.axioms_used().keys().map(|s| s.as_str()).collect();
            let _ = writeln!(
                t,
                "cites           {l} = {} ({})",
                c.path.escape_debug(),
                spf.id
            );
            let _ = writeln!(t, "  sha256        {}", c.sha256);
            let _ = writeln!(t, "  proved        {}", th.conclusion());
            let _ = writeln!(t, "  assumptions   {}", gamma(&th));
            let _ = writeln!(t, "  axiom set     {}", spf.axiom_set);
            let _ = writeln!(
                t,
                "  axioms used   {}",
                if used.is_empty() {
                    "none".to_string()
                } else {
                    used.join(", ")
                }
            );
            po.cited.push((l.clone(), th));
        }
    }
    let _ = writeln!(t, "steps           {}", po.steps.len());
    out.goal = po.goal.as_eq().cloned();
    out.prop = Some(po.goal.clone());

    let (verdict_line, code) = match check(&po, &set.axioms) {
        Verdict::Accepted(th) => {
            // Every axiom the theorem rests on must be, by content, an axiom of the registered set.
            if let Some((name, _)) = th
                .axioms_used()
                .iter()
                .find(|(_, a)| !set.axioms.contains(a))
            {
                bail!(format!(
                    "theorem rests on axiom `{name}` outside registered set {}",
                    set.id
                ));
            }
            // The goal and the conclusion are equal up to binder names; the
            // goal's names are the ones the reader wrote.
            let _ = writeln!(t, "proved          {}", po.goal);
            let _ = writeln!(t, "assumptions     {}", gamma(&th));
            let used: Vec<&str> = th.axioms_used().keys().map(|s| s.as_str()).collect();
            let _ = writeln!(
                t,
                "axioms used     {}",
                if used.is_empty() {
                    "none".to_string()
                } else {
                    used.join(", ")
                }
            );
            // Rules of the steps the result rests on: traced back from the last
            // step through step references, so unused steps are not counted.
            let mut needed: Vec<&str> = po
                .steps
                .last()
                .map(|s| s.label.as_str())
                .into_iter()
                .collect();
            let mut seen: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
            let mut rules: Vec<&str> = Vec::new();
            let mut cited_used: Vec<&str> = Vec::new();
            while let Some(label) = needed.pop() {
                if !seen.insert(label) {
                    continue;
                }
                if let Some(st) = po.steps.iter().find(|s| s.label == label) {
                    // RLT-003 M3c: a citation reuses a checked theorem; it is a
                    // dependency, reported apart from the inference rules.
                    if st.rule == "cite" {
                        if let [rlt_kernel::Arg::Name(c)] = st.args.as_slice() {
                            cited_used.push(c.as_str());
                        }
                        continue;
                    }
                    rules.push(st.rule.as_str());
                    for a in &st.args {
                        if let rlt_kernel::Arg::Step(r) = a {
                            needed.push(r.as_str());
                        }
                    }
                }
            }
            rules.sort();
            rules.dedup();
            let _ = writeln!(
                t,
                "rules used      {}",
                if rules.is_empty() {
                    "none".to_string()
                } else {
                    rules.join(", ")
                }
            );
            if !pf.cites.is_empty() {
                cited_used.sort();
                cited_used.dedup();
                let _ = writeln!(
                    t,
                    "theorems cited  {} — checked theorem dependencies (see `cites`), not inference rules",
                    if cited_used.is_empty() { "none".to_string() } else { cited_used.join(", ") }
                );
            }
            // result kind describes scope for readers; it is computed only
            // after acceptance and never affects the verdict (M3a O9).
            let _ = writeln!(t, "result kind     {}", classify(th.conclusion()));
            let kind = if th.is_unconditional() {
                "unconditional theorem"
            } else {
                "conditional theorem"
            };
            out.theorem = Some(th);
            (
                format!(
                    "ACCEPT — {kind} relative to registered axiom set {}",
                    set.id
                ),
                ACCEPT,
            )
        }
        Verdict::Rejected { step, reason } => {
            let at = step.map(|s| format!(" at step {s}")).unwrap_or_default();
            (format!("REJECT{at} — {reason}"), REJECT)
        }
    };
    let _ = writeln!(t, "verdict         {verdict_line}");
    let scope = if code == ACCEPT && po.goal.as_eq().is_none() {
        "the checked formula follows from the listed axioms under Γ by the registered rules; the axioms remain assumptions, and the reading point's meanings and physical claims keep their own separate status"
    } else if code == ACCEPT {
        "the checked equation follows from the listed axioms under Γ; the axioms remain assumptions, and the reading point's meanings and physical claims keep their own separate status"
    } else {
        "the kernel refused this derivation; the goal's truth stays undetermined by this proof, and the reading point's meanings and physical claims keep their own separate status"
    };
    let _ = writeln!(t, "  scope         {scope}");

    // Layer 4: physical correspondence, read from the record and reported
    // beside the verdict. The verdict never changes these statuses.
    if let Some(rp) = &rp {
        for c in &rp.correspondence {
            let _ = writeln!(
                t,
                "physical        {} [{}]: {} (evidence: {})",
                c.id,
                c.kind,
                c.status,
                c.evidence.len()
            );
        }
        let _ = writeln!(t, "  note          physical statuses come from evidence records; the kernel verdict leaves them unchanged");
    }
    out.proof = Some(pf);
    out.report = Report { text: t, code };
    out
}

/// Bind a reading point's relationship to its proof file: the proof must name
/// this reading point and relationship, use the relationship's convention,
/// and its checked goal must be the elaboration of the relationship's
/// written statement under that convention.
pub fn bind_relationship(
    root: &Path,
    rp_id: &str,
    r: &Relationship,
) -> (CheckOutcome, Result<(), String>) {
    bind_proof(root, Some(rp_id), r)
}

/// Bind a stated equation (written form, convention, mode, Γ) to a proof
/// file. With `rp_id`, the proof must also name that reading point and the
/// relationship id (RLT-001 relationships). Without it (RLT-002 counts), the
/// binding is by statement alone, so one checked equation can serve several
/// specification records.
pub fn bind_proof(
    root: &Path,
    rp_id: Option<&str>,
    r: &Relationship,
) -> (CheckOutcome, Result<(), String>) {
    let path = root.join(&r.proof);
    let contained = match (
        root.join(&r.proof).canonicalize(),
        root.join("tests").canonicalize(),
    ) {
        (Ok(p), Ok(tests)) => p.starts_with(&tests),
        _ => false,
    };
    if r.proof.contains("..")
        || !r.proof.starts_with("tests/")
        || !r.proof.ends_with(".rlt")
        || !contained
        || r.proof.chars().any(|c| c.is_control())
    {
        let out = CheckOutcome {
            report: Report {
                text: String::new(),
                code: MALFORMED,
            },
            proof: None,
            goal: None,
            prop: None,
            theorem: None,
            citations: Vec::new(),
        };
        return (
            out,
            Err(format!(
                "proof path `{}` must be a tests/…/*.rlt file",
                r.proof
            )),
        );
    }
    // Every component below the root (tests/, its subdirectories, the file) is
    // a real directory or file: a symbolic link anywhere on the path could
    // point outside the repository while still canonicalizing under tests/.
    let linked = {
        let mut at = root.to_path_buf();
        Path::new(&r.proof).components().any(|c| {
            at.push(c);
            std::fs::symlink_metadata(&at).is_ok_and(|m| m.file_type().is_symlink())
        })
    };
    if linked {
        let out = CheckOutcome {
            report: Report {
                text: String::new(),
                code: MALFORMED,
            },
            proof: None,
            goal: None,
            prop: None,
            theorem: None,
            citations: Vec::new(),
        };
        return (
            out,
            Err(format!(
                "proof `{}` is a symbolic link or lies under one; proof paths are regular directories and files",
                r.proof
            )),
        );
    }
    let out = check_file_outcome(root, &path);
    let res = (|| {
        let pf = out
            .proof
            .as_ref()
            .ok_or_else(|| format!("proof not checked: {}", verdict_text(&out.report)))?;
        if let Some(rp_id) = rp_id {
            if pf.readingpoint.as_deref() != Some(rp_id) {
                return Err(format!(
                    "proof names reading point {:?}, record is `{rp_id}`",
                    pf.readingpoint
                ));
            }
            if pf.relationship.as_deref() != Some(r.id.as_str()) {
                return Err(format!(
                    "proof names relationship {:?}, record is `{}`",
                    pf.relationship, r.id
                ));
            }
        }
        if pf.convention != r.convention {
            return Err(format!(
                "proof uses convention {}, record says {}",
                pf.convention, r.convention
            ));
        }
        let convs = pinned_conventions(root)?;
        let conv = convs
            .get(&r.convention)
            .ok_or(format!("convention `{}` is unregistered", r.convention))?;
        let stated = elaborate_formula(&Written::new(&r.written), &conv.convention)?;
        let goal = out.goal.as_ref().ok_or("proof goal did not elaborate")?;
        if &stated != goal {
            return Err(format!("record states `{stated}`, proof checks `{goal}`"));
        }
        let pf_mode = match pf.mode {
            Mode::Unconditional => "unconditional",
            Mode::Conditional => "conditional",
        };
        if r.mode != pf_mode {
            return Err(format!(
                "record expects mode {}, proof requests {pf_mode}",
                r.mode
            ));
        }
        if let Some(th) = &out.theorem {
            let mut expected: Vec<Formula> = r
                .assumptions
                .iter()
                .map(|w| elaborate_formula(&Written::new(w), &conv.convention))
                .collect::<Result<_, _>>()?;
            let mut actual: Vec<Formula> = th
                .assumptions()
                .values()
                .map(|p| {
                    p.as_eq().cloned().ok_or_else(|| {
                        format!("the theorem rests on `{p}`, which is not an equation")
                    })
                })
                .collect::<Result<_, _>>()?;
            expected.sort();
            actual.sort();
            if expected != actual {
                let show = |v: &[Formula]| {
                    v.iter()
                        .map(|f| f.to_string())
                        .collect::<Vec<_>>()
                        .join("; ")
                };
                return Err(format!(
                    "record expects Γ = {{ {} }}, the theorem rests on Γ = {{ {} }}",
                    show(&expected),
                    show(&actual)
                ));
            }
        }
        Ok(())
    })();
    (out, res)
}

/// `rlt written A B` with a convention for each side.
pub fn written_report(root: &Path, a: &str, ca: &str, b: &str, cb: &str) -> Report {
    let mut t = String::new();
    let convs = match pinned_conventions(root) {
        Ok(c) => c,
        Err(e) => return malformed(&mut t, e),
    };
    let (wa, wb) = (Written::new(a), Written::new(b));
    let _ = writeln!(
        t,
        "written A       \"{}\"  tokens: {}",
        wa.text,
        tokens(&wa)
    );
    let _ = writeln!(
        t,
        "written B       \"{}\"  tokens: {}",
        wb.text,
        tokens(&wb)
    );
    let _ = writeln!(
        t,
        "written ≡w      {}",
        if wa.identical(&wb) {
            "identical"
        } else {
            "distinct"
        }
    );
    let get = |id: &str| -> Result<&Convention, String> {
        convs
            .get(id)
            .map(|c| &c.convention)
            .ok_or(format!("convention `{id}` is unregistered"))
    };
    let (conv_a, conv_b) = match (get(ca), get(cb)) {
        (Ok(x), Ok(y)) => (x, y),
        (Err(e), _) | (_, Err(e)) => return malformed(&mut t, e),
    };
    let ea = elaborate_term(&wa, conv_a);
    let eb = elaborate_term(&wb, conv_b);
    match (&ea, &eb) {
        (Ok(x), Ok(y)) => {
            let _ = writeln!(
                t,
                "elaborated A    {} under {}  kernel form: {}",
                x,
                ca,
                x.kernel_form()
            );
            let _ = writeln!(
                t,
                "elaborated B    {} under {}  kernel form: {}",
                y,
                cb,
                y.kernel_form()
            );
            let _ = writeln!(
                t,
                "terms           {}",
                if x == y {
                    "structurally identical"
                } else {
                    "structurally distinct"
                }
            );
            let _ = writeln!(t, "equality        decided only by a kernel proof (`rlt check`); this report proves nothing");
            Report {
                text: t,
                code: ACCEPT,
            }
        }
        (Err(e), _) | (_, Err(e)) => malformed(&mut t, e.clone()),
    }
}

/// `rlt rp FILE`: a reading point's meanings, its relationships checked live,
/// its correspondence records and lifts — each in its own section.
pub fn rp_report(root: &Path, id: &str) -> Report {
    let mut t = String::new();
    if let Err(e) = rlt_syntax::proof::record_id(id.to_string()) {
        return malformed(&mut t, e);
    }
    let rp = match load_readingpoint(&rp_path(root, id)) {
        Ok(r) => r,
        Err(e) => return malformed(&mut t, e),
    };
    let (reg, vocab) = match (pinned_registry(root), pinned_status(root)) {
        (Ok(r), Ok(v)) => (r, v),
        (Err(e), _) | (_, Err(e)) => return malformed(&mut t, e),
    };
    let _ = writeln!(t, "reading point   {}", rp.id);
    let _ = writeln!(t, "meanings (verbatim; `rlt validate` checks each against its source and checks completeness)");
    for m in &rp.meanings {
        let _ = writeln!(t, "  \"{}\"  [{}]", m.text, m.source);
    }
    let _ = writeln!(t, "examples (written)");
    for m in &rp.examples {
        let _ = writeln!(t, "  \"{}\"  [{}]", m.text, m.source);
    }
    let _ = writeln!(t, "relationships (attached results, checked now)");
    let mut code = ACCEPT;
    for r in &rp.relationships {
        let (out, binding) = bind_relationship(root, &rp.id, r);
        let rep = &out.report;
        let verdict = match (&out.theorem, rep.code) {
            (Some(th), ACCEPT) => format!(
                "ACCEPT — {} theorem relative to registered axiom set {}",
                if th.is_unconditional() {
                    "unconditional"
                } else {
                    "conditional"
                },
                out.proof
                    .as_ref()
                    .map(|p| p.axiom_set.as_str())
                    .unwrap_or("?")
            ),
            (_, REJECT) => "REJECT".to_string(),
            _ => "MALFORMED".to_string(),
        };
        let gamma = out
            .theorem
            .as_ref()
            .map(|th| format!(", {}", gamma(th)))
            .unwrap_or_default();
        let verdict = match &binding {
            Ok(()) => verdict,
            Err(e) => {
                code = REJECT;
                format!("UNBOUND — {e}")
            }
        };
        if rep.code != ACCEPT {
            code = REJECT;
        }
        let _ = writeln!(
            t,
            "  {}  \"{}\" under {}  →  {}{}  ({})",
            r.id, r.written, r.convention, verdict, gamma, r.proof
        );
    }
    let _ = writeln!(
        t,
        "physical correspondence (evidence-graded, independent of verdicts)"
    );
    for c in &rp.correspondence {
        let mut ps = Vec::new();
        let lines = check_status_and_evidence(
            &c.id,
            &c.kind,
            std::slice::from_ref(&c.id),
            &c.status,
            &c.evidence,
            &vocab,
            &reg,
            false,
            &mut ps,
        );
        let _ = writeln!(
            t,
            "  {} [{}] {} · {} — \"{}\"",
            c.id,
            c.kind,
            if ps.is_empty() {
                "record valid"
            } else {
                "record INVALID (see rlt validate)"
            },
            if ps.is_empty() {
                format!(
                    "physical {}",
                    status_shown(&c.status, c.evidence.len(), "correspondence")
                )
            } else {
                "physical not reported (record invalid)".to_string()
            },
            c.description
        );
        for l in lines {
            let _ = writeln!(t, "  {l}");
        }
        if !ps.is_empty() {
            code = REJECT;
        }
    }
    let _ = writeln!(t, "PiπΠ lifts");
    if rp.lifts.is_empty() {
        let _ = writeln!(t, "  (none recorded)");
    }
    for l in &rp.lifts {
        let _ = writeln!(t, "  {} → {} via \"{}\": {}", l.id, l.to, l.via, l.status);
    }
    Report { text: t, code }
}

/// `rlt chem WRITTEN CONVENTION`: how a species-table convention reads one
/// written formula. Exit 0 when the formula has a registered entry, 1 when it
/// is uninterpreted (neither true nor false), 2 when the input or a pin is
/// malformed. No kernel theorem and no physical claim come from this report.
pub fn chem_report(root: &Path, written: &str, conv_id: &str) -> Report {
    let mut t = String::new();
    if let Err(e) = rlt_syntax::records::clean(written, "written", "rlt chem", false) {
        return malformed(&mut t, e);
    }
    let (convs, reg) = match (pinned_conventions(root), pinned_registry(root)) {
        (Ok(c), Ok(r)) => (c, r),
        (Err(e), _) | (_, Err(e)) => return malformed(&mut t, e),
    };
    let Some(conv) = convs.get(conv_id) else {
        return malformed(&mut t, format!("convention `{conv_id}` is unregistered"));
    };
    let w = Written::new(written);
    let _ = writeln!(t, "RLT chem");
    let _ = writeln!(t, "written         \"{}\"  tokens: {}", w.text, tokens(&w));
    let _ = writeln!(
        t,
        "convention      {} (interpretation input; executable: elaborator {}, pinned)",
        conv.convention.id, conv.convention.elaborator
    );
    if conv.convention.elaborator != "species-table" {
        return malformed(
            &mut t,
            format!(
                "convention {} has elaborator `{}`; rlt chem reads species-table conventions",
                conv.convention.id, conv.convention.elaborator
            ),
        );
    }
    match conv.species_entry(written) {
        Some(e) => {
            let src = reg
                .get(&e.source)
                .map(|s| s.status.as_str())
                .unwrap_or("unregistered");
            let _ = writeln!(
                t,
                "entry           {} ↦ {} · {} ({} atoms) [{} {}]",
                e.written,
                e.species,
                e.composition_text(),
                e.total(),
                e.source,
                e.locator
            );
            let names: Vec<String> = e
                .composition
                .iter()
                .map(|(sym, _)| format!("{sym} ↦ {}", conv.element_name(sym).unwrap_or("?")))
                .collect();
            let _ = writeln!(
                t,
                "elements        {} (element identifiers are labels of this convention)",
                names.join(" · ")
            );
            let _ = writeln!(t, "source          {} ({src})", e.source);
            let _ = writeln!(
                t,
                "structure       not stated by this convention (composition only)"
            );
            let _ = writeln!(
                t,
                "physical        not assessed (notation only; no specimen is identified)"
            );
            let _ = writeln!(t, "verdict         INTERPRETED — a registered reading of notation; no kernel theorem and no physical claim");
            Report {
                text: t,
                code: ACCEPT,
            }
        }
        None => {
            let _ = writeln!(t, "entry           none (lookup is by the whole written string; no composition is computed from its letters)");
            let _ = writeln!(
                t,
                "verdict         UNINTERPRETED — {} has no entry for \"{}\"; no composition is assigned, and uninterpreted is neither true nor false",
                conv.convention.id, w.text
            );
            Report {
                text: t,
                code: REJECT,
            }
        }
    }
}

/// SHA-256 of the source registry's documentation content (see
/// `registry_digest`), pinned in the binary. Every documented meaning a
/// reading point may carry comes from a tagged registry excerpt, so fixing the
/// registry's content here means a meaning can be deleted, truncated,
/// untagged, retagged to another reading point, or newly cited only through a
/// reviewed code change. Update it with the digest `rlt validate` prints.
pub const PINNED_REGISTRY_SHA: &str =
    "f85b76fb40c7243e9e4a760449c321ede6f94857ee674877181d0bfbc6c92347";

/// Canonical digest of the registry: for every source (in id order) its id,
/// status and location, then each excerpt's text, `defines` and `split`, one
/// field per line.
pub fn registry_digest(reg: &BTreeMap<String, Source>) -> String {
    let mut buf = String::new();
    for s in reg.values() {
        buf.push_str(&format!(
            "source\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            s.id, s.status, s.location, s.title, s.url, s.commit, s.note
        ));
        for x in &s.excerpts {
            buf.push_str(&format!(
                "excerpt\t{}\t{}\t{}\n",
                x.text,
                x.defines.as_deref().unwrap_or(""),
                if x.whole { "whole" } else { "units" }
            ));
        }
    }
    sha256_hex(buf.as_bytes())
}

/// SHA-256 of the status vocabulary `correspondence/status.yaml` (see
/// `status_digest`), pinned in the binary beside the registry. Every status
/// and evidence rule reads this vocabulary, so it changes only through a
/// reviewed code change. Update it with the digest `rlt validate` prints.
pub const PINNED_STATUS_SHA: &str =
    "80d1d09b316bfee8554e2b2d40f48f7a3fd0c9bfd08faa3885000dc48519fc53";

/// Canonical digest of the status vocabulary: each list, one per line, in a
/// fixed order.
pub fn status_digest(v: &StatusVocabulary) -> String {
    let mut buf = String::new();
    for (name, xs) in [
        ("correspondence_status", &v.correspondence),
        ("status_without_evidence", &v.without_evidence),
        ("referent_only_status", &v.referent_only),
        ("allowed_support", &v.allowed_support),
        ("lift_status", &v.lift),
    ] {
        buf.push_str(name);
        for x in xs {
            buf.push('\t');
            buf.push_str(x);
        }
        buf.push('\n');
    }
    sha256_hex(buf.as_bytes())
}

/// The registry, refused unless it matches `PINNED_REGISTRY_SHA`. Every
/// command that reads the registry reads it through here.
pub fn pinned_registry(root: &Path) -> Result<BTreeMap<String, Source>, String> {
    let reg = load_registry(&root.join("sources/registry.yaml"))?;
    let digest = registry_digest(&reg);
    if digest != PINNED_REGISTRY_SHA {
        return Err(format!(
            "sources/registry.yaml differs from the registry pinned in this binary (digest {digest}, pinned {PINNED_REGISTRY_SHA}); a registry change needs a reviewed update of PINNED_REGISTRY_SHA"
        ));
    }
    Ok(reg)
}

/// The status vocabulary, refused unless it matches `PINNED_STATUS_SHA`.
pub fn pinned_status(root: &Path) -> Result<StatusVocabulary, String> {
    let v = load_status(&root.join("correspondence/status.yaml"))?;
    let digest = status_digest(&v);
    if digest != PINNED_STATUS_SHA {
        return Err(format!(
            "correspondence/status.yaml differs from the status vocabulary pinned in this binary (digest {digest}, pinned {PINNED_STATUS_SHA}); a vocabulary change needs a reviewed update of PINNED_STATUS_SHA"
        ));
    }
    Ok(v)
}

/// Registered conventions, pinned in the binary: (id, interpretation digest,
/// explanation digest). The interpretation digest covers `id`, `authority`,
/// `sources`, `origin` and the `executable` block, which decide what a written
/// expression elaborates to; the explanation digest covers the prose. Either
/// changes only through a reviewed code change. Update with the digests
/// `rlt validate` prints.
pub const PINNED_CONVENTIONS: &[(&str, &str, &str)] = &[
    (
        "CHEM-MOLFORMULA",
        "181a8beca3b0d64445fd4dc8cef8689cba346900b4bb32d22eb299b770fdeb33",
        "de8a1b4a391146d83383710b558ebf3c7d2ea149dd86bae1d0dda421039b78f7",
    ),
    (
        "NAT-DEC",
        "c3d2e84b3ef3037b8bc165f643afd968ea5d5f30aa043deee47af61e39d636bf",
        "fbe76eaec888aca56e70cc728fc0abd8047dffc329b852742600a9483e421762",
    ),
    // RLT-003 M3a: written first-order formulas; NAT-DEC stays as it was.
    // RLT-003 M3b: NAT-LOGIC plus the pinned definitions <, ∣, free235.
    (
        "NAT-DEF",
        "935f9635e563cab1d60cd4b5d7ffbed121933eb59e790edd9fb7c21d088bd208",
        "a6a720b2d61f91e92bfaaf86f2947ec7bcab5a267d83a68c1766898c213b0524",
    ),
    (
        "NAT-LOGIC",
        "4b0c13c7004df97a2a6224a3547dc010df0bc997f06b06b116aa0e8459b4c49b",
        "d327261c129e3a89e2684cc7b83073059b0114386f912ba6f13a8fad8196b4b7",
    ),
    (
        "PIPIPI",
        "126600cb46f737e353942999d73ffea97c1bca8b4e969df2e30dbdecf82cbad6",
        "5dd80717c5cf8ca8212d985266d5baf3efc3722095f5222545b64fec5a2e1069",
    ),
    (
        "PRIME-COUNT",
        "5f582cd0ccfd7315ca6f4d08a4b5e2f73b6bf7ce78864f2cdd68ad94d687a423",
        "0f9bac9aecf2fc0f9d610ab9d75a672781ec6ff8aa768ca3b9e8500a93555a8b",
    ),
    // RLT-003 M2: closed quotient–remainder witnesses modulo 30.
    (
        "RESIDUE-30",
        "1f59729992864d504e8d4e0e929f8a4da17b7bb7409d188e0a1e0277b8b53552",
        "d8127ccea388172a085ca54a7d0dca440e978db4fe99eaaa6a257bec355420ca",
    ),
    (
        "REAL-PI",
        "c431b6dfe296a04977854aa2faff730fd626d8cc7347bcee0ee55e8eaad15886",
        "4bf8ed727569da451ba0bdf2d7906ed13a94b5440599187757b045f0a239f1bd",
    ),
];

/// (interpretation digest, explanation digest) of a convention record.
pub fn convention_digests(c: &ConventionRecord) -> (String, String) {
    (
        sha256_hex(c.interpretation_canon.as_bytes()),
        sha256_hex(c.explanation_canon.as_bytes()),
    )
}

/// Every difference between the loaded conventions and the pins.
pub fn convention_pin_problems(convs: &BTreeMap<String, ConventionRecord>) -> Vec<String> {
    let mut problems = Vec::new();
    for (id, c) in convs {
        let (i, e) = convention_digests(c);
        match PINNED_CONVENTIONS.iter().find(|(p, _, _)| p == id) {
            None => problems.push(format!(
                "convention {id} is outside PINNED_CONVENTIONS (interpretation digest {i}, explanation digest {e}); registering a convention needs a reviewed code change"
            )),
            Some((_, pi, pe)) => {
                if &i != pi {
                    problems.push(format!(
                        "convention {id}: executable fields differ from the convention pinned in this binary (interpretation digest {i}, pinned {pi}); a change to what it elaborates needs a reviewed update of PINNED_CONVENTIONS"
                    ));
                }
                if &e != pe {
                    problems.push(format!(
                        "convention {id}: explanation differs from the convention pinned in this binary (explanation digest {e}, pinned {pe}); a prose change needs a reviewed update of PINNED_CONVENTIONS"
                    ));
                }
            }
        }
    }
    for (id, _, _) in PINNED_CONVENTIONS {
        if !convs.contains_key(*id) {
            problems.push(format!(
                "pinned convention {id} is missing from conventions/"
            ));
        }
    }
    problems
}

/// The conventions, refused unless every one matches its pin. Every command
/// other than `rlt validate` reads conventions through here.
pub fn pinned_conventions(root: &Path) -> Result<BTreeMap<String, ConventionRecord>, String> {
    let convs = load_conventions(&root.join("conventions"))?;
    match convention_pin_problems(&convs).into_iter().next() {
        Some(p) => Err(p),
        None => Ok(convs),
    }
}

/// Statuses a physical or relational record may hold, fixed in the binary.
/// RLT checks that submitted evidence is complete and linked to its claim; it
/// leaves open whether the evidence establishes the claim. So a record states
/// at most `evidence-submitted`, and no configuration or evidence label can
/// make RLT report a claim as established.
pub const RECORDABLE_STATUS: [&str; 4] = [
    "unestablished",
    "evidence-submitted",
    "inapplicable",
    "imagined",
];

/// Statuses that record an establishment judgment (RML `supported`,
/// `rejected`). They stay in the vocabulary as source terms; a record using
/// one is refused, since RLT reports evidence without judging it.
pub const JUDGMENT_STATUS: [&str; 2] = ["supported", "rejected"];

/// Check a record's physical or relational status and its evidence items.
/// `claim` and `referents` are what each evidence item must name. Returns the
/// report lines for the evidence and appends problems.
#[allow(clippy::too_many_arguments)]
fn check_status_and_evidence(
    who: &str,
    claim: &str,
    referents: &[String],
    st: &str,
    evidence: &[EvidenceItem],
    vocab: &StatusVocabulary,
    reg: &BTreeMap<String, Source>,
    relation: bool,
    problems: &mut Vec<String>,
) -> Vec<String> {
    if JUDGMENT_STATUS.contains(&st) {
        problems.push(format!(
            "{who} status `{st}` records an establishment judgment; RLT reports submitted evidence without judging it, so record `evidence-submitted` with linked evidence, or `unestablished`"
        ));
    } else if !vocab.correspondence.iter().any(|v| v == st) || !RECORDABLE_STATUS.contains(&st) {
        problems.push(format!("{who} status `{st}` is outside the vocabulary"));
    }
    if evidence.is_empty() && !vocab.without_evidence.iter().any(|v| v == st) {
        problems.push(format!("{who} status `{st}` requires evidence"));
    }
    if !evidence.is_empty() && st != "evidence-submitted" && !JUDGMENT_STATUS.contains(&st) {
        problems.push(format!(
            "{who} status `{st}` with {} evidence item(s): a record with submitted evidence has status `evidence-submitted`",
            evidence.len()
        ));
    }
    let mut lines = Vec::new();
    for e in evidence {
        if !vocab.allowed_support.contains(&e.etype) {
            problems.push(format!(
                "{who} evidence type `{}` is outside allowed_support{}",
                e.etype,
                if relation {
                    " (counts and theorems are never evidence for a relation)"
                } else {
                    ""
                }
            ));
        }
        let src = reg.get(&e.reference).filter(|s| s.status == "available");
        if src.is_none() {
            problems.push(format!(
                "{who} evidence ref `{}` must be an available registered source",
                e.reference
            ));
        }
        match e.claim.as_deref() {
            Some(c) if c == claim => {}
            Some(c) => problems.push(format!(
                "{who} evidence names claim `{c}`; this record's claim is `{claim}`"
            )),
            None => problems.push(format!(
                "{who} evidence must name its claim (`claim: {claim}`)"
            )),
        }
        if e.referents != referents {
            problems.push(format!(
                "{who} evidence names referents [{}]; this record's referents are [{}]",
                e.referents.join(", "),
                referents.join(", ")
            ));
        }
        if e.locator.as_deref().is_none_or(|l| l.trim().is_empty()) {
            problems.push(format!(
                "{who} evidence must give a `locator`: where in `{}` the evidence sits",
                e.reference
            ));
        }
        if e.criteria.as_deref().is_none_or(|c| c.trim().is_empty()) {
            problems.push(format!(
                "{who} evidence must state its validation `criteria`"
            ));
        }
        lines.push(format!(
            "  evidence      {} · {} [{}] at \"{}\" · for {claim}({}) · criteria \"{}\" · submitted, not evaluated by RLT",
            e.etype,
            e.reference,
            src.map(|s| s.location.as_str()).filter(|l| !l.is_empty()).unwrap_or("no registered location"),
            e.locator.as_deref().unwrap_or("—"),
            e.referents.join(", "),
            e.criteria.as_deref().unwrap_or("—"),
        ));
    }
    lines
}

/// How a status reads in a report: submitted evidence leaves the claim
/// unestablished until a reviewer evaluates it.
fn status_shown(st: &str, n: usize, what: &str) -> String {
    if st == "evidence-submitted" {
        format!("evidence-submitted ({n} item(s), not evaluated by RLT; {what} unestablished)")
    } else if RECORDABLE_STATUS.contains(&st) {
        format!("{st} (evidence: {n})")
    } else {
        // A refused status is shown as what RLT reports: unestablished.
        format!(
            "unestablished (record states `{}`, refused; evidence: {n})",
            st.escape_debug()
        )
    }
}

/// Correspondence record kinds.
const CORRESPONDENCE_KINDS: [&str; 3] = ["behavior", "representation", "referent"];

/// Source statuses that may document a reading-point meaning. `conversation`
/// sources are statements made in the RLT review itself and are quoted verbatim.
pub(crate) const CITABLE: [&str; 2] = ["available", "conversation"];

fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// `rlt validate`: consistency of the data records (no proofs are run).
pub fn validate(root: &Path) -> Report {
    let mut t = String::new();
    let mut problems: Vec<String> = Vec::new();
    let reg = match load_registry(&root.join("sources/registry.yaml")) {
        Ok(r) => r,
        Err(e) => return malformed(&mut t, e),
    };
    let status = match load_status(&root.join("correspondence/status.yaml")) {
        Ok(s) => s,
        Err(e) => return malformed(&mut t, e),
    };
    let sdigest = status_digest(&status);
    if sdigest != PINNED_STATUS_SHA {
        problems.push(format!(
            "correspondence/status.yaml differs from the status vocabulary pinned in this binary (digest {sdigest}, pinned {PINNED_STATUS_SHA}); a vocabulary change needs a reviewed update of PINNED_STATUS_SHA"
        ));
    }
    let convs: BTreeMap<String, ConventionRecord> =
        match load_conventions(&root.join("conventions")) {
            Ok(c) => c,
            Err(e) => return malformed(&mut t, e),
        };
    problems.extend(convention_pin_problems(&convs));
    // Every pinned axiom set must load: file, registry entry, SHA-256, source.
    let mut loaded = Vec::new();
    for (id, _, _) in PINNED_AXIOM_SETS {
        match load_axiom_set(root, id) {
            Ok(set) => loaded.push(format!("{id} ({} axioms)", set.axioms.len())),
            Err(e) => problems.push(e),
        }
    }
    let _ = writeln!(t, "axiom sets      {}", loaded.join(", "));
    let _ = writeln!(t, "sources         {} registered", reg.len());
    let digest = registry_digest(&reg);
    if digest != PINNED_REGISTRY_SHA {
        problems.push(format!(
            "sources/registry.yaml differs from the registry pinned in this binary (digest {digest}, pinned {PINNED_REGISTRY_SHA}); a registry change needs a reviewed update of PINNED_REGISTRY_SHA"
        ));
    }
    for src in reg.values() {
        if src.status == "available"
            && !src.excerpts.is_empty()
            && !github_file_at_commit(&src.location)
        {
            problems.push(format!(
                "{}: status available with excerpts requires a GitHub file location",
                src.id
            ));
        }
    }
    for src in reg.values().filter(|s| s.status == "archived") {
        let hex = src.commit.len() == 64
            && src
                .commit
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
        // Same rule as scripts/check_excerpts.py `member_ok`: a relative
        // path of non-empty components, none of them `.` or `..`.
        let member = src.url.split_once('#').is_some_and(|(a, m)| {
            a.starts_with("https://")
                && !m.is_empty()
                && !m.starts_with('/')
                && m.split('/').all(|c| !c.is_empty() && c != "." && c != "..")
        });
        if !hex || !member || src.excerpts.is_empty() {
            problems.push(format!(
                "{}: status archived requires `commit` = the archive's SHA-256 (64 lowercase hex), `url` = https archive URL + '#' + member path (relative, no `.` or `..` components), and excerpts",
                src.id
            ));
        }
    }
    for (st, n) in reg
        .values()
        .fold(BTreeMap::new(), |mut m: BTreeMap<&str, usize>, s| {
            *m.entry(s.status.as_str()).or_default() += 1;
            m
        })
    {
        let _ = writeln!(t, "  {st:<14}{n}");
    }
    let _ = writeln!(
        t,
        "conventions     {}",
        convs.keys().cloned().collect::<Vec<_>>().join(", ")
    );
    for c in convs.values() {
        if c.authority != "interpretation" {
            problems.push(format!(
                "convention {}: authority must be `interpretation`",
                c.convention.id
            ));
        }
        for s in &c.sources {
            if !reg.contains_key(s) {
                problems.push(format!(
                    "convention {}: source `{s}` is unregistered",
                    c.convention.id
                ));
            }
        }
        for e in &c.species {
            if !reg
                .get(&e.source)
                .is_some_and(|s| ["reference", "available"].contains(&s.status.as_str()))
            {
                problems.push(format!(
                    "convention {}: species entry {:?} cites `{}`, which must be a registered reference or available source",
                    c.convention.id, e.written, e.source
                ));
            }
            if !c.sources.contains(&e.source) {
                problems.push(format!(
                    "convention {}: species entry {:?} cites `{}`, which is missing from the convention's sources",
                    c.convention.id, e.written, e.source
                ));
            }
        }
        // RLT-003 M2: a residue-table convention's factorization theorem and quotation.
        problems.extend(residues::residue_table_problems(root, c, &reg));
        if c.sources.is_empty() && c.origin.as_deref() != Some("rlt-authored") {
            problems.push(format!(
                "convention {}: cites zero sources, so it must be marked `origin: rlt-authored`",
                c.convention.id
            ));
        }
    }
    // PiπΠ conventions: every stated form must appear verbatim in its source's excerpts.
    match rlt_syntax::records::load_yaml(&root.join("conventions/pi.yaml")) {
        Ok(y) => {
            let y = &y["explanation"];
            let Some(items) = y["conventions"].as_vec().cloned() else {
                problems.push("conventions/pi.yaml: `conventions` must be a list".into());
                return finish(t, problems);
            };
            let mut n = 0;
            for c in &items {
                let id = c["id"].as_str().unwrap_or("?");
                let src = c["source"].as_str().unwrap_or("");
                let Some(reg_src) = reg.get(src) else {
                    problems.push(format!("PiπΠ {id}: source `{src}` is unregistered"));
                    continue;
                };
                let stated = c["stated"].as_vec().cloned().unwrap_or_default();
                if stated.is_empty() {
                    problems.push(format!("PiπΠ {id}: `stated` must list the source's words"));
                }
                for st in stated {
                    let st = collapse_ws(st.as_str().unwrap_or(""));
                    if !reg_src.excerpts.iter().any(|x| x.units().contains(&st)) {
                        problems.push(format!(
                            "PiπΠ {id}: \"{st}\" equals zero whole units of `{src}`'s excerpts"
                        ));
                    }
                }
                n += 1;
            }
            let unresolved = y["unresolved"].as_vec().map(|v| v.len()).unwrap_or(0);
            let _ = writeln!(t, "PiπΠ            {n} source conventions traced, {unresolved} unresolved items recorded without definition");
        }
        Err(e) => problems.push(e),
    }
    let mut rps: Vec<_> = std::fs::read_dir(root.join("readingpoints"))
        .map(|d| d.filter_map(|e| e.ok().map(|e| e.path())).collect())
        .unwrap_or_default();
    rps.sort();
    for p in &rps {
        let name = p
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let plain = !name.chars().any(|c| c.is_control()) && name.ends_with(".yaml");
        if !plain || std::fs::symlink_metadata(p).is_ok_and(|m| !m.file_type().is_file()) {
            problems.push(format!(
                "readingpoints/ holds {name:?}: every entry must be a regular *.yaml reading-point record"
            ));
        }
    }
    let rps: Vec<PathBuf> = rps
        .into_iter()
        .filter(|p| {
            p.extension().is_some_and(|x| x == "yaml")
                && !p.to_string_lossy().chars().any(|c| c.is_control())
        })
        .collect();
    for p in rps.iter() {
        let rp = match load_readingpoint(p) {
            Ok(r) => r,
            Err(e) => {
                problems.push(e);
                continue;
            }
        };
        let _ = writeln!(
            t,
            "reading point   {}: {} meanings, {} relationships, {} correspondence records, {} lifts",
            rp.id,
            rp.meanings.len(),
            rp.relationships.len(),
            rp.correspondence.len(),
            rp.lifts.len()
        );
        // Verbatim rule: each meaning and example equals a whole unit of an
        // excerpt, from its cited source, tagged `defines: <this reading point>`.
        for m in rp.meanings.iter().chain(rp.examples.iter()) {
            let Some(src) = reg.get(&m.source) else {
                problems.push(format!("{}: source `{}` is unregistered", rp.id, m.source));
                continue;
            };
            if !CITABLE.contains(&src.status.as_str()) {
                problems.push(format!(
                    "{}: \"{}\" cites `{}`, whose status `{}` cannot document a meaning",
                    rp.id, m.text, m.source, src.status
                ));
                continue;
            }
            let ok = src
                .excerpts
                .iter()
                .filter(|x| x.defines.as_deref() == Some(rp.id.as_str()))
                .any(|x| x.units().contains(&m.text));
            if !ok {
                problems.push(format!(
                    "{}: \"{}\" equals zero whole units of `{}` excerpts tagged for {}",
                    rp.id, m.text, m.source, rp.id
                ));
            }
        }
        // Completeness: every unit of every excerpt tagged for this reading
        // point appears as a meaning or example citing that source.
        for src in reg.values() {
            for x in src
                .excerpts
                .iter()
                .filter(|x| x.defines.as_deref() == Some(rp.id.as_str()))
            {
                for u in x.units() {
                    let present = rp
                        .meanings
                        .iter()
                        .chain(rp.examples.iter())
                        .any(|m| m.source == src.id && m.text == u);
                    if !present {
                        problems.push(format!(
                            "{}: documented unit \"{u}\" from `{}` is missing from the record",
                            rp.id, src.id
                        ));
                    }
                }
            }
        }
        let mut rel_ids: Vec<&str> = Vec::new();
        for r in &rp.relationships {
            if rel_ids.contains(&r.id.as_str()) {
                problems.push(format!(
                    "{}: relationship id `{}` appears twice",
                    rp.id, r.id
                ));
            }
            rel_ids.push(&r.id);
            if !convs.contains_key(&r.convention) {
                problems.push(format!(
                    "{}: relationship {} uses unregistered convention `{}`",
                    rp.id, r.id, r.convention
                ));
            }
            match bind_relationship(root, &rp.id, r) {
                (out, Ok(())) if out.report.code == ACCEPT => {}
                (_, Ok(())) => problems.push(format!(
                    "{}: relationship {} proof `{}` is not accepted by the kernel",
                    rp.id, r.id, r.proof
                )),
                (_, Err(e)) => problems.push(format!("{}: relationship {}: {e}", rp.id, r.id)),
            }
        }
        for c in &rp.correspondence {
            check_status_and_evidence(
                &format!("{}: {}", rp.id, c.id),
                &c.kind,
                std::slice::from_ref(&c.id),
                &c.status,
                &c.evidence,
                &status,
                &reg,
                false,
                &mut problems,
            );
            if status.referent_only.contains(&c.status) && c.kind != "referent" {
                problems.push(format!(
                    "{}: {} status `{}` applies to referent records only",
                    rp.id, c.id, c.status
                ));
            }
            if !CORRESPONDENCE_KINDS.contains(&c.kind.as_str()) {
                problems.push(format!(
                    "{}: {} kind `{}` is outside {{behavior, representation, referent}}",
                    rp.id, c.id, c.kind
                ));
            }
            if let Some(s) = &c.source {
                if !reg.contains_key(s) {
                    problems.push(format!("{}: {} source `{s}` is unregistered", rp.id, c.id));
                }
            }
        }
        for l in &rp.lifts {
            if !status.lift.contains(&l.status) {
                problems.push(format!(
                    "{}: lift {} status `{}` is outside the vocabulary",
                    rp.id, l.id, l.status
                ));
            }
            if l.status == "verified" {
                problems.push(format!(
                    "{}: lift {} marked verified; RLT-001 verifies zero lifts",
                    rp.id, l.id
                ));
            }
        }
    }
    // RLT-002 specification records.
    let mut pts: Vec<PathBuf> = std::fs::read_dir(root.join("points"))
        .map(|d| d.filter_map(|e| e.ok().map(|e| e.path())).collect())
        .unwrap_or_default();
    pts.sort();
    for p in &pts {
        let name = p
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let plain = !name.chars().any(|c| c.is_control()) && name.ends_with(".yaml");
        if !plain || std::fs::symlink_metadata(p).is_ok_and(|m| !m.file_type().is_file()) {
            problems.push(format!(
                "points/ holds {name:?}: every entry must be a regular *.yaml specification record"
            ));
            continue;
        }
        let (_, ps, _) = points_check(root, p);
        let _ = writeln!(t, "points          {}: {} problem(s)", name, ps.len());
        problems.extend(ps);
    }
    // RLT-003 M2 witness records.
    let mut res: Vec<PathBuf> = std::fs::read_dir(root.join("residues"))
        .map(|d| d.filter_map(|e| e.ok().map(|e| e.path())).collect())
        .unwrap_or_default();
    res.sort();
    for p in &res {
        let name = p
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let plain = !name.chars().any(|c| c.is_control()) && name.ends_with(".yaml");
        if !plain || std::fs::symlink_metadata(p).is_ok_and(|m| !m.file_type().is_file()) {
            problems.push(format!(
                "residues/ holds {name:?}: every entry must be a regular *.yaml witness record"
            ));
            continue;
        }
        let (_, ps, _) = residues::residues_check(root, p);
        let _ = writeln!(t, "residues        {}: {} problem(s)", name, ps.len());
        problems.extend(ps);
    }
    let rp_ids: Vec<String> = rps
        .iter()
        .filter_map(|p| p.file_stem().map(|s| s.to_string_lossy().to_string()))
        .collect();
    for src in reg.values() {
        for x in &src.excerpts {
            if let Some(d) = &x.defines {
                if !rp_ids.contains(d) {
                    problems.push(format!(
                        "{}: excerpt tagged for unknown reading point `{d}`",
                        src.id
                    ));
                }
            }
        }
    }
    finish(t, problems)
}

fn finish(mut t: String, problems: Vec<String>) -> Report {
    if problems.is_empty() {
        let _ = writeln!(t, "verdict         VALID — records consistent");
        Report {
            text: t,
            code: ACCEPT,
        }
    } else {
        for p in &problems {
            let _ = writeln!(t, "problem         {p}");
        }
        let _ = writeln!(t, "verdict         INVALID — {} problem(s)", problems.len());
        Report {
            text: t,
            code: REJECT,
        }
    }
}

/// result kind (M3a O9): a reporting label computed from the accepted
/// conclusion; acceptance never depends on it.
pub fn result_kind(p: &rlt_kernel::Prop) -> &'static str {
    if p.is_closed_equation() {
        "closed (one equation between numeral expressions; it establishes this statement only, not a general theorem)"
    } else if p.as_eq().is_some() {
        "general (contains variables)"
    } else {
        "general (a formula with connectives or quantifiers; it establishes this statement only)"
    }
}

fn lean_term(t: &rlt_kernel::Term) -> String {
    lean_term_in(t, &[])
}

fn lean_term_in(t: &rlt_kernel::Term, names: &[String]) -> String {
    use rlt_kernel::Term;
    if let Some(n) = t.as_numeral() {
        return n.to_string();
    }
    let atom = |x: &Term| {
        let s = lean_term_in(x, names);
        if x.as_numeral().is_some() || matches!(x, Term::Bound(_) | Term::Param(_)) {
            s
        } else {
            format!("({s})")
        }
    };
    match t {
        Term::Bound(i) => match names.len().checked_sub(i + 1).and_then(|k| names.get(k)) {
            Some(n) => n.clone(),
            None => format!("sorry /- dangling index {i} -/"),
        },
        Term::App(f, a) if f == "S" && a.len() == 1 => format!("Nat.succ {}", atom(&a[0])),
        Term::App(f, a) if f == "+" && a.len() == 2 => format!("{} + {}", atom(&a[0]), atom(&a[1])),
        Term::App(f, a) if f == "*" && a.len() == 2 => format!("{} * {}", atom(&a[0]), atom(&a[1])),
        other => format!("sorry /- unsupported term {} -/", other.kernel_form()),
    }
}

/// Value of a ground term in ℕ (for disclosure in the Lean replay only).
fn nat_value(t: &rlt_kernel::Term) -> Option<u128> {
    use rlt_kernel::Term;
    match t {
        Term::App(f, a) if f == "0" && a.is_empty() => Some(0),
        Term::App(f, a) if f == "S" && a.len() == 1 => nat_value(&a[0])?.checked_add(1),
        Term::App(f, a) if f == "+" && a.len() == 2 => {
            nat_value(&a[0])?.checked_add(nat_value(&a[1])?)
        }
        Term::App(f, a) if f == "*" && a.len() == 2 => {
            nat_value(&a[0])?.checked_mul(nat_value(&a[1])?)
        }
        _ => None,
    }
}

fn lean_formula(f: &Formula) -> String {
    format!("{} = {}", lean_term(&f.lhs), lean_term(&f.rhs))
}

/// A formula as a Lean proposition over ℕ (RLT-003 M3a). Binder names are
/// made distinct, as in RLT's own display.
fn lean_prop(p: &rlt_kernel::Prop, names: &mut Vec<String>) -> String {
    use rlt_kernel::Prop;
    let paren = |s: String| format!("({s})");
    match p {
        Prop::Eq(f) if names.is_empty() => lean_formula(f),
        Prop::Eq(f) => format!(
            "{} = {}",
            lean_term_in(&f.lhs, names),
            lean_term_in(&f.rhs, names)
        ),
        Prop::False => "False".into(),
        Prop::And(a, b) => format!(
            "{} ∧ {}",
            paren(lean_prop(a, names)),
            paren(lean_prop(b, names))
        ),
        Prop::Or(a, b) => format!(
            "{} ∨ {}",
            paren(lean_prop(a, names)),
            paren(lean_prop(b, names))
        ),
        Prop::Imp(a, b) => format!(
            "{} → {}",
            paren(lean_prop(a, names)),
            paren(lean_prop(b, names))
        ),
        Prop::Forall(h, b) | Prop::Exists(h, b) => {
            let q = if matches!(p, Prop::Forall(..)) {
                "∀"
            } else {
                "∃"
            };
            let mut h = h.clone();
            while names.contains(&h) {
                h.push('\'');
            }
            names.push(h.clone());
            let body = lean_prop(b, names);
            names.pop();
            format!("{q} {h} : Nat, {body}")
        }
    }
}

/// `rlt lean FILE…`: a Lean 4 replay generated from the statements the kernel
/// accepted. Each accepted proof becomes a Lean theorem with exactly the
/// kernel's assumption context Γ as hypotheses; Lean proves it independently
/// (`decide` for closed statements, `omega` from the hypotheses otherwise).
/// Lean sees only the checked statement, never RLT's derivation.
pub fn lean_export(root: &Path, files: &[PathBuf]) -> Report {
    let mut t = String::new();
    let _ = writeln!(t, "/-\nGenerated by `rlt lean` from the accepted RLT proof files. Do not edit:\nregenerate with `rlt lean tests/accept/*.rlt > lean/Generated.lean`.\nEach theorem states exactly the kernel's checked goal under the kernel's\nassumption context Γ. Lean checks the statement independently; it never sees\nRLT's derivation. A conditional theorem whose hypothesis is false in ℕ is\nmarked VACUOUS: Lean then proves it trivially. Run: cd lean && lean Generated.lean\n-/\n\nnamespace RLTGenerated\n");
    let mut code = ACCEPT;
    for f in files {
        let out = check_file_outcome(root, f);
        let (Some(pf), Some(th)) = (&out.proof, &out.theorem) else {
            code = REJECT;
            let _ = writeln!(
                t,
                "-- skipped {}: {}\n",
                shown_path(root, f),
                verdict_text(&out.report)
            );
            continue;
        };
        let name: String = pf
            .id
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect();
        let hyps: String = th
            .assumptions()
            .iter()
            .map(|(n, phi)| format!(" ({n} : {})", lean_prop(phi, &mut Vec::new())))
            .collect();
        // M3a formulas: a fixed list of core tactics, tried in order. For a
        // top-level ∃ proved by exists-intro over a closed witness, Lean is
        // also offered that witness and checks it itself.
        // RLT-003 M3b: any exists-intro step whose claim is the conclusion
        // (not only the last step) offers its closed witness.
        let witness = match th.conclusion() {
            rlt_kernel::Prop::Exists(..) => pf
                .steps
                .iter()
                .filter(|st| st.rule == "exists-intro" && &st.claim == th.conclusion())
                .find_map(|st| match st.args.first() {
                    Some(rlt_kernel::Arg::Term(t)) if t.as_numeral().is_some() => {
                        Some(format!("exact ⟨{}, by decide⟩ | ", lean_term(t)))
                    }
                    _ => None,
                })
                .unwrap_or_default(),
            _ => String::new(),
        };
        // Assumptions that are not equations: their truth in ℕ is not
        // evaluated here (audit at 6ba9f1e, weakness 3), so the replay says so
        // and Lean is offered the full tactic list.
        let unclassified: Vec<&str> = th
            .assumptions()
            .iter()
            .filter(|(_, phi)| phi.as_eq().is_none())
            .map(|(n, _)| n.as_str())
            .collect();
        let tactic = if th.conclusion().as_eq().is_none() || !unclassified.is_empty() {
            format!("first | {witness}decide | omega | (intros; omega) | (simp; done) | (intros; simp_all; done) | grind | (intro x; cases x <;> simp)")
        } else if th.is_unconditional() {
            "decide".to_string()
        } else {
            "omega".to_string()
        };
        let false_hyps: Vec<&str> = th
            .assumptions()
            .iter()
            .filter(|(_, phi)| {
                phi.as_eq().is_some_and(|phi| {
                    matches!((nat_value(&phi.lhs), nat_value(&phi.rhs)), (Some(a), Some(b)) if a != b)
                })
            })
            .map(|(n, _)| n.as_str())
            .collect();
        if !false_hyps.is_empty() {
            let _ = writeln!(
                t,
                "-- VACUOUS REPLAY: hypothesis {} is false in ℕ, so `omega` proves any statement from it.\n-- This theorem confirms only that the statement is stated with RLT's Γ; it adds no evidence.",
                false_hyps.join(", ")
            );
        }
        if !unclassified.is_empty() {
            let _ = writeln!(
                t,
                "-- UNCLASSIFIED HYPOTHESIS: {} is not an equation; whether it is false in ℕ is not checked here,\n-- so this replay may be vacuous. If Lean's tactics do not close it, compilation fails visibly.",
                unclassified.join(", ")
            );
        }
        let _ = writeln!(
            t,
            "-- {}  (axiom set: {})\ntheorem {name}{hyps} : {} := by {tactic}\n",
            shown_path(root, f),
            pf.axiom_set,
            lean_prop(
                &out.prop.clone().unwrap_or_else(|| th.conclusion().clone()),
                &mut Vec::new()
            )
        );
    }
    let _ = writeln!(t, "end RLTGenerated");
    Report { text: t, code }
}

// ---- RLT-002: specification records (points/*.yaml) ----

/// Kinds of reading point a points file may declare.
pub const POINT_KINDS: [&str; 10] = [
    "written-expression",
    "mathematical-object",
    "physical-object",
    "physical-relationship",
    "observed-event",
    "human-action",
    "imagined-possibility",
    "depicted-subject",
    "description",
    "generalization",
];

/// Relation types a points file may declare. A relation carries its own
/// status and evidence; counts and theorems are never evidence for one.
pub const RELATION_TYPES: [&str; 7] = [
    "same-structure",
    "same-species",
    "bonded-to",
    "parent-of",
    "sibling-of",
    "part-of",
    "history",
];

/// The numeral summands of a sum `a + b + …` (left-nested), if every leaf is a numeral.
pub(crate) fn numeral_summands(t: &rlt_kernel::Term) -> Option<Vec<u64>> {
    use rlt_kernel::Term;
    if let Some(n) = t.as_numeral() {
        return Some(vec![n]);
    }
    match t {
        Term::App(f, a) if f == "+" && a.len() == 2 => {
            let mut v = numeral_summands(&a[0])?;
            v.extend(numeral_summands(&a[1])?);
            Some(v)
        }
        _ => None,
    }
}

/// Validate one points file and build its report. Returns (report, problems).
/// The verdict text of a check report, after `verdict`.
pub(crate) fn verdict_text(r: &Report) -> String {
    r.text
        .lines()
        .rev()
        .find_map(|l| l.strip_prefix("verdict"))
        .map(|v| v.trim().to_string())
        .unwrap_or_else(|| "no verdict".to_string())
}

/// One record's verdict line: valid, or invalid with its problem count.
pub(crate) fn record_verdict(
    before: usize,
    problems: &[String],
    tally: &mut (usize, usize),
) -> (bool, String) {
    let n = problems.len() - before;
    tally.0 += 1;
    if n == 0 {
        (true, "record valid".to_string())
    } else {
        tally.1 += 1;
        (
            false,
            format!("record INVALID ({n} problem(s), listed below)"),
        )
    }
}

/// Validate one points file and build its report. Returns the report, the
/// problems, and (records, invalid records).
pub fn points_check(root: &Path, file: &Path) -> (String, Vec<String>, (usize, usize)) {
    let mut tally = (0usize, 0usize);
    let (t, problems) = points_check_inner(root, file, &mut tally);
    (t, problems, tally)
}

fn points_check_inner(
    root: &Path,
    file: &Path,
    tally: &mut (usize, usize),
) -> (String, Vec<String>) {
    use rlt_syntax::records::load_points;
    let mut t = String::new();
    let mut problems: Vec<String> = Vec::new();
    let shown = shown_path(root, file);
    let _ = writeln!(t, "RLT points      {shown}");
    // A points file is a regular *.yaml file directly inside the repository's
    // own points/ directory (itself a plain directory).
    let in_points = std::fs::symlink_metadata(root.join("points")).is_ok_and(|m| m.is_dir())
        && match (
            file.parent().and_then(|p| p.canonicalize().ok()),
            root.join("points").canonicalize(),
        ) {
            (Some(a), Ok(b)) => a == b,
            _ => false,
        }
        && file.extension().is_some_and(|x| x == "yaml");
    if !in_points {
        problems.push(format!(
            "{shown}: a points file is a *.yaml file inside this repository's points/ directory"
        ));
        return (t, problems);
    }
    let pf = match load_points(file) {
        Ok(p) => p,
        Err(e) => {
            problems.push(e);
            return (t, problems);
        }
    };
    // Pinned inputs: the same registry and status vocabulary `rlt validate` checks.
    let (reg, status) = match (pinned_registry(root), pinned_status(root)) {
        (Ok(r), Ok(s)) => (r, s),
        (Err(e), _) | (_, Err(e)) => {
            problems.push(e);
            return (t, problems);
        }
    };
    let convs = match pinned_conventions(root) {
        Ok(c) => c,
        Err(e) => {
            problems.push(e);
            return (t, problems);
        }
    };
    let _ = writeln!(t, "context         {} — \"{}\"", pf.id, pf.context);
    {
        let b = &pf.basis;
        let _ = writeln!(t, "  basis         \"{}\"  [{}]", b.text, b.source);
        match reg.get(&b.source) {
            Some(src) if CITABLE.contains(&src.status.as_str()) => {
                if !src.excerpts.iter().any(|x| x.units().contains(&b.text)) {
                    problems.push(format!(
                        "{}: basis \"{}\" equals zero whole units of `{}`",
                        pf.id, b.text, b.source
                    ));
                }
            }
            _ => problems.push(format!(
                "{}: basis source `{}` must be an available or conversation source",
                pf.id, b.source
            )),
        }
    }
    for r in &pf.references {
        // Ids are plain (no `/` or `..`), so the path stays in readingpoints/.
        let rp_file = rp_path(root, r);
        let rp_dir_plain =
            std::fs::symlink_metadata(root.join("readingpoints")).is_ok_and(|m| m.is_dir());
        if rp_dir_plain && load_readingpoint(&rp_file).is_ok() {
            let _ = writeln!(
                t,
                "references      {r} (its documented meanings stay in readingpoints/{r}.yaml)"
            );
        } else {
            problems.push(format!("{}: references unknown reading point `{r}`", pf.id));
        }
    }

    // Record ids form one namespace per file. A shared id is a problem of every
    // record that carries it, so each of them reports INVALID.
    let mut all_ids: Vec<(&str, &str)> = Vec::new();
    for (kind, id) in pf
        .points
        .iter()
        .map(|p| ("point", p.id.as_str()))
        .chain(
            pf.compositions
                .iter()
                .map(|c| ("composition", c.id.as_str())),
        )
        .chain(pf.counts.iter().map(|c| ("count", c.claim.id.as_str())))
        .chain(pf.relations.iter().map(|r| ("relation", r.id.as_str())))
    {
        all_ids.push((kind, id));
    }
    let shared_id = |id: &str, problems: &mut Vec<String>| {
        let kinds: Vec<&str> = all_ids
            .iter()
            .filter(|(_, i)| *i == id)
            .map(|(k, _)| *k)
            .collect();
        if kinds.len() > 1 {
            problems.push(format!(
                "{}: record id `{id}` is shared by {} records ({}); record ids are unique within a file",
                pf.id,
                kinds.len(),
                kinds.join(", ")
            ));
        }
    };
    let (_, ctx_line) = record_verdict(0, &problems, tally);
    let _ = writeln!(
        t,
        "  {ctx_line} (context: basis, references and record ids)"
    );

    // Reading points.
    let mut ids: Vec<&str> = Vec::new();
    let _ = writeln!(t, "reading points  {}", pf.points.len());
    for p in &pf.points {
        let p0 = problems.len();
        shared_id(&p.id, &mut problems);
        ids.push(&p.id);
        if !POINT_KINDS.contains(&p.kind.as_str()) {
            problems.push(format!(
                "{}: point {} kind `{}` is outside the point kinds",
                pf.id, p.id, p.kind
            ));
        }
        if !["particular", "general"].contains(&p.scope.as_str()) {
            problems.push(format!(
                "{}: point {} scope `{}`: use particular or general",
                pf.id, p.id, p.scope
            ));
        }
        // Registered chemistry: an atom names an element, a molecule a species,
        // each under a species-table convention. Interpretation only.
        let mut chem_desc = String::new();
        if p.element.is_some() && p.species.is_some() {
            problems.push(format!(
                "{}: point {} names both an element and a species; an atom record names an element, a molecule record a species",
                pf.id, p.id
            ));
        }
        if let Some((sym, cid)) = &p.element {
            match convs.get(cid) {
                Some(c) if c.convention.elaborator == "species-table" => match c.element_name(sym) {
                    Some(name) => chem_desc = format!(" · element {sym} ({name}) under {cid}"),
                    None => problems.push(format!(
                        "{}: point {}: element {sym} is outside the {cid} element table",
                        pf.id, p.id
                    )),
                },
                _ => problems.push(format!(
                    "{}: point {}: element convention `{cid}` must be a registered species-table convention",
                    pf.id, p.id
                )),
            }
        }
        if let Some((w, cid)) = &p.species {
            match convs.get(cid) {
                Some(c) if c.convention.elaborator == "species-table" => match c.species_entry(w) {
                    Some(e) => {
                        chem_desc = format!(
                            " · species {} ↦ {} under {cid} (registered interpretation; specimen identity unestablished)",
                            e.written, e.species
                        )
                    }
                    None => problems.push(format!(
                        "{}: point {}: species \"{w}\" has no entry under {cid}; it stays uninterpreted, so this record names no species",
                        pf.id, p.id
                    )),
                },
                _ => problems.push(format!(
                    "{}: point {}: species convention `{cid}` must be a registered species-table convention",
                    pf.id, p.id
                )),
            }
        }
        let ev_lines = check_status_and_evidence(
            &format!("{}: point {} physical", pf.id, p.id),
            "physical-correspondence",
            std::slice::from_ref(&p.id),
            &p.physical_status,
            &p.evidence,
            &status,
            &reg,
            false,
            &mut problems,
        );
        if p.physical_status == "inapplicable"
            && ["physical-object", "physical-relationship", "observed-event"]
                .contains(&p.kind.as_str())
        {
            problems.push(format!(
                "{}: point {} of kind {} has physical correspondence to assess; `inapplicable` applies to other kinds",
                pf.id, p.id, p.kind
            ));
        }
        if p.physical_status == "imagined" && p.kind != "imagined-possibility" {
            problems.push(format!(
                "{}: point {} status `imagined` applies to imagined-possibility points only",
                pf.id, p.id
            ));
        }
        let (valid, rec) = record_verdict(p0, &problems, tally);
        let _ = writeln!(
            t,
            "  {:<14}{} · class {} · scope {}{chem_desc} — \"{}\"",
            p.id,
            p.kind,
            p.class.as_deref().unwrap_or("—"),
            p.scope,
            p.description
        );
        let _ = writeln!(
            t,
            "                {rec} · physical {}",
            if valid {
                status_shown(&p.physical_status, p.evidence.len(), "correspondence")
            } else {
                "not reported (record invalid)".to_string()
            }
        );
        for l in ev_lines {
            let _ = writeln!(t, "{l}");
        }
    }
    let point = |id: &str| pf.points.iter().find(|p| p.id == id);
    let species_entry = |id: &str| {
        point(id)
            .and_then(|p| p.species.as_ref())
            .and_then(|(w, cid)| {
                convs
                    .get(cid)
                    .and_then(|c| c.species_entry(w))
                    .map(|e| (cid.clone(), e))
            })
    };
    let element_of = |id: &str| {
        point(id)
            .and_then(|p| p.element.as_ref())
            .and_then(|(sym, cid)| {
                convs
                    .get(cid)
                    .filter(|c| c.element_name(sym).is_some())
                    .map(|_| (sym.clone(), cid.clone()))
            })
    };

    // Compositions: which atom records this file counts as each molecule's.
    let mut composed: Vec<(&str, &str)> = Vec::new();
    let mut molecules: Vec<&str> = Vec::new();
    for comp in &pf.compositions {
        let k0 = problems.len();
        shared_id(&comp.id, &mut problems);
        if molecules.contains(&comp.molecule.as_str()) {
            problems.push(format!(
                "{}: composition {}: molecule {} already has a composition; a molecule has at most one",
                pf.id, comp.id, comp.molecule
            ));
        }
        molecules.push(&comp.molecule);
        if point(&comp.molecule).is_some_and(|p| p.element.is_some()) {
            problems.push(format!(
                "{}: composition {}: molecule {} also names an element; a molecule record names a species only",
                pf.id, comp.id, comp.molecule
            ));
        }
        let _ = writeln!(
            t,
            "composition     {} · {} = {}",
            comp.id,
            comp.molecule,
            comp.atoms.join(", ")
        );
        let mut seen: Vec<&str> = Vec::new();
        for a in &comp.atoms {
            if seen.contains(&a.as_str()) {
                problems.push(format!(
                    "{}: composition {}: atom {a} is listed twice",
                    pf.id, comp.id
                ));
                continue;
            }
            seen.push(a);
            if let Some((other, _)) = composed.iter().find(|(_, x)| *x == a.as_str()) {
                problems.push(format!(
                    "{}: composition {}: atom {a} is already in composition {other}",
                    pf.id, comp.id
                ));
            }
            composed.push((&comp.id, a));
            if point(a).is_some_and(|p| p.species.is_some()) {
                problems.push(format!(
                    "{}: composition {}: atom {a} names a species; an atom record names an element only",
                    pf.id, comp.id
                ));
            }
        }
        match species_entry(&comp.molecule) {
            None => problems.push(format!(
                "{}: composition {}: molecule {} must be a point with a registered species",
                pf.id, comp.id, comp.molecule
            )),
            Some((cid, e)) => {
                let mut got: BTreeMap<String, u64> = BTreeMap::new();
                for a in &seen {
                    match element_of(a) {
                        Some((sym, acid)) if acid == cid => *got.entry(sym).or_default() += 1,
                        _ => problems.push(format!(
                            "{}: composition {}: atom {a} must be a point with an element under {cid}",
                            pf.id, comp.id
                        )),
                    }
                }
                let want: BTreeMap<String, u64> = e.composition.iter().cloned().collect();
                let mut order: Vec<String> = e.composition.iter().map(|(s, _)| s.clone()).collect();
                for k in got.keys() {
                    if !order.contains(k) {
                        order.push(k.clone());
                    }
                }
                let text = order
                    .iter()
                    .map(|k| format!("{k}×{}", got.get(k).copied().unwrap_or(0)))
                    .collect::<Vec<_>>()
                    .join(" ");
                if got == want && problems.len() == k0 {
                    let _ = writeln!(
                        t,
                        "  check         {text} matches {cid} entry {} ({})",
                        e.written,
                        e.composition_text()
                    );
                } else if got == want {
                    let _ = writeln!(t, "  check         not reported (record invalid)");
                } else {
                    problems.push(format!(
                        "{}: composition {}: composition {text} differs from {cid} entry {} ({})",
                        pf.id,
                        comp.id,
                        e.written,
                        e.composition_text()
                    ));
                }
            }
        }
        let _ = writeln!(t, "  meaning       consistency with the registered interpretation; establishes no observation, bond or specimen identity");
        let (_, rec) = record_verdict(k0, &problems, tally);
        let _ = writeln!(t, "  {rec}");
    }

    // Counts: a checked equation associated with groups of reading points.
    let mut used_proofs: Vec<(String, String)> = Vec::new();
    for c in &pf.counts {
        let c0 = problems.len();
        shared_id(&c.claim.id, &mut problems);
        let _ = writeln!(t, "count           {}", c.claim.id);
        let (out, binding) = bind_proof(root, None, &c.claim);
        let verdict = match (&out.theorem, out.report.code) {
            (Some(th), ACCEPT) => format!(
                "ACCEPT — {} theorem relative to registered axiom set {}, {}",
                if th.is_unconditional() {
                    "unconditional"
                } else {
                    "conditional"
                },
                out.proof
                    .as_ref()
                    .map(|p| p.axiom_set.as_str())
                    .unwrap_or("?"),
                gamma(th)
            ),
            (_, REJECT) => "REJECT".to_string(),
            _ => "MALFORMED".to_string(),
        };
        let _ = writeln!(
            t,
            "  equation      \"{}\" under {} → {}  ({})",
            c.claim.written, c.claim.convention, verdict, c.claim.proof
        );
        if let Err(e) = &binding {
            problems.push(format!("{}: count {}: {e}", pf.id, c.claim.id));
        }
        if let Some(cv) = convs.get(&c.claim.convention) {
            let _ = writeln!(
                t,
                "  convention    {} (interpretation input; executable: elaborator {}, pinned) — explanation: \"{}\"",
                cv.convention.id,
                cv.convention.elaborator,
                cv.reading.join(" ")
            );
        }
        if out.report.code != ACCEPT {
            problems.push(format!(
                "{}: count {}: its proof is not accepted by the kernel",
                pf.id, c.claim.id
            ));
            let _ = writeln!(
                t,
                "  binding       not reported: the kernel did not accept this proof"
            );
        }
        // The binding of numerals to reading points: checked for consistency only.
        if let (Some(goal), ACCEPT) = (&out.goal, out.report.code) {
            match numeral_summands(&goal.lhs) {
                None => problems.push(format!(
                    "{}: count {}: the left side must be a sum of numerals",
                    pf.id, c.claim.id
                )),
                Some(nums) => {
                    if nums.len() != c.groups.len() {
                        problems.push(format!(
                            "{}: count {}: {} summand(s) but {} group(s)",
                            pf.id,
                            c.claim.id,
                            nums.len(),
                            c.groups.len()
                        ));
                    }
                    for (i, (n, g)) in nums.iter().zip(&c.groups).enumerate() {
                        if *n != g.len() as u64 {
                            problems.push(format!(
                                "{}: count {}: summand {} is {n} but its group names {} reading point(s)",
                                pf.id,
                                c.claim.id,
                                i + 1,
                                g.len()
                            ));
                        }
                    }
                    let parts: Vec<String> = nums
                        .iter()
                        .zip(&c.groups)
                        .map(|(n, g)| format!("{n} ↦ {{{}}}", g.join(", ")))
                        .collect();
                    let _ = writeln!(t, "  binding       {}", parts.join(" · "));
                }
            }
            let mut members: Vec<&str> = Vec::new();
            for m in c.groups.iter().flatten() {
                match point(m) {
                    None => problems.push(format!("{}: count {}: unknown point `{m}`", pf.id, c.claim.id)),
                    Some(p) if p.scope != "particular" => problems.push(format!(
                        "{}: count {}: point {m} has scope {}; a count binds particular, identified reading points",
                        pf.id, c.claim.id, p.scope
                    )),
                    _ => {}
                }
                if members.contains(&m.as_str()) {
                    problems.push(format!(
                        "{}: count {}: point {m} is named in two groups",
                        pf.id, c.claim.id
                    ));
                }
                members.push(m);
            }
            if goal.rhs.as_numeral() != Some(members.len() as u64) {
                problems.push(format!(
                    "{}: count {}: the right side must be the numeral {} (the distinct reading points bound)",
                    pf.id,
                    c.claim.id,
                    members.len()
                ));
            }
            let mut classes: Vec<&str> = members
                .iter()
                .filter_map(|m| point(m).and_then(|p| p.class.as_deref()))
                .collect();
            classes.sort();
            classes.dedup();
            let mut distinct = members.clone();
            distinct.sort();
            distinct.dedup();
            let _ = writeln!(
                t,
                "  identities    {} — {} distinct reading point(s){}; class {}",
                members.join(", "),
                distinct.len(),
                if distinct.len() == members.len() {
                    String::new()
                } else {
                    format!(" in {} bindings", members.len())
                },
                if classes.is_empty() {
                    "—".to_string()
                } else {
                    classes.join(", ")
                }
            );
            let scope = c.scope.as_deref().unwrap_or("particular");
            if scope != "particular" {
                problems.push(format!(
                    "{}: count {}: states scope `{scope}`; a count of particular reading points keeps scope particular, and widening needs its own source or evidence",
                    pf.id, c.claim.id
                ));
            }
            let _ = writeln!(t, "  scope         particular (the reading points listed)");
            let _ = writeln!(t, "  association   a specification record: the kernel checked the equation; this file states what each numeral counts");
        }
        let (_, rec) = record_verdict(c0, &problems, tally);
        let _ = writeln!(t, "  {rec} (the kernel verdict above stands on its own)");
        used_proofs.push((c.claim.id.clone(), c.claim.proof.clone()));
    }
    let mut proofs: Vec<&str> = used_proofs.iter().map(|(_, p)| p.as_str()).collect();
    proofs.sort();
    proofs.dedup();
    for p in proofs {
        let sharing: Vec<&str> = used_proofs
            .iter()
            .filter(|(_, q)| q == p)
            .map(|(c, _)| c.as_str())
            .collect();
        if sharing.len() > 1 {
            let _ = writeln!(
                t,
                "shared theorem  {} rest on the same checked equation ({p}); sharing it relates none of their reading points",
                sharing.join(" and ")
            );
        }
    }

    // Relations: each with its own status and evidence.
    if pf.relations.is_empty() {
        let _ = writeln!(t, "relations       none recorded; structural, physical and identity relations among these reading points stay unestablished");
    }
    for r in &pf.relations {
        let r0 = problems.len();
        shared_id(&r.id, &mut problems);
        if !RELATION_TYPES.contains(&r.rtype.as_str()) {
            problems.push(format!(
                "{}: relation {} type `{}` is outside the relation types",
                pf.id, r.id, r.rtype
            ));
        }
        for b in &r.between {
            if point(b).is_none() {
                problems.push(format!("{}: relation {}: unknown point `{b}`", pf.id, r.id));
            }
        }
        let mut uniq = r.between.clone();
        uniq.sort();
        uniq.dedup();
        if r.between.len() < 2 || uniq.len() != r.between.len() {
            problems.push(format!(
                "{}: relation {} relates {} reading point(s) with {} distinct; a relation names two or more distinct points",
                pf.id,
                r.id,
                r.between.len(),
                uniq.len()
            ));
        }
        // Same-species and registered species: once any referent names a
        // species, every referent must carry a registered species and all must
        // be the same; otherwise the claim conflicts with the registered
        // interpretation (a conflict of records, not a physical finding).
        if r.rtype == "same-species"
            && r.between
                .iter()
                .any(|b| point(b).is_some_and(|p| p.species.is_some()))
        {
            let mut shown: Vec<String> = Vec::new();
            let mut ids: Vec<String> = Vec::new();
            let mut convs_used: Vec<String> = Vec::new();
            for b in &r.between {
                match species_entry(b) {
                    Some((cid, e)) => {
                        shown.push(format!("{b}: {}", e.species));
                        ids.push(e.species.clone());
                        convs_used.push(cid);
                    }
                    None => shown.push(format!("{b}: no registered species")),
                }
            }
            let mut distinct = ids.clone();
            distinct.sort();
            distinct.dedup();
            convs_used.sort();
            convs_used.dedup();
            if ids.len() != r.between.len() || distinct.len() > 1 {
                problems.push(format!(
                    "{}: relation {}: registered-specification conflict: same-species contradicts the species identifiers {} assigns ({}); this concerns the records and the registered interpretation, not an experimental conclusion about physical samples",
                    pf.id,
                    r.id,
                    if convs_used.is_empty() {
                        "the registered conventions".to_string()
                    } else {
                        convs_used.join(", ")
                    },
                    shown.join("; ")
                ));
            }
        }
        if r.status == "imagined" {
            problems.push(format!(
                "{}: relation {} status `imagined` is outside the relation statuses",
                pf.id, r.id
            ));
        }
        let ev_lines = check_status_and_evidence(
            &format!("{}: relation {}", pf.id, r.id),
            &r.rtype,
            &r.between,
            &r.status,
            &r.evidence,
            &status,
            &reg,
            true,
            &mut problems,
        );
        let (valid, rec) = record_verdict(r0, &problems, tally);
        let _ = writeln!(
            t,
            "relation        {} [{}] {}",
            r.id,
            r.rtype,
            r.between.join(", ")
        );
        let _ = writeln!(
            t,
            "                {rec} · status {}",
            if valid {
                status_shown(&r.status, r.evidence.len(), "the relation stays")
            } else {
                "not reported (record invalid)".to_string()
            }
        );
        for l in ev_lines {
            let _ = writeln!(t, "{l}");
        }
    }
    (t, problems)
}

/// `rlt points FILE`: the report for one points file, with its validation.
pub fn points_report(root: &Path, file: &Path) -> Report {
    let (mut t, problems, (records, invalid)) = points_check(root, file);
    // Zero records read: the file or a pinned input could not be loaded, which
    // says nothing about any record.
    if records == 0 && !problems.is_empty() {
        let msg = problems.join("; ");
        return malformed(&mut t, msg);
    }
    for p in &problems {
        let _ = writeln!(t, "problem         {p}");
    }
    if problems.is_empty() {
        let _ = writeln!(
            t,
            "verdict         VALID — specification records consistent ({records} record(s) valid; physical correspondence as reported per record)"
        );
        Report {
            text: t,
            code: ACCEPT,
        }
    } else {
        let _ = writeln!(
            t,
            "verdict         INVALID — {invalid} of {records} record(s) invalid; {} problem(s)",
            problems.len()
        );
        Report {
            text: t,
            code: REJECT,
        }
    }
}

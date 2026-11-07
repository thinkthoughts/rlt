//! Reports for `rlt check`, `rlt written`, `rlt rp` and `rlt validate`.
//! Every report is a deterministic string so tests can compare it with a
//! golden file.

use rlt_kernel::{check, Axiom, Formula, Mode, Theorem, Verdict};
use rlt_syntax::elab::{elaborate_formula, elaborate_term, Convention};
use rlt_syntax::proof::{parse_axiom_file, parse_proof_file, ProofFile};
use rlt_syntax::records::{
    load_conventions, load_readingpoint, load_registry, load_status, load_yaml, ConventionRecord,
    ReadingPoint, Relationship,
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

fn malformed(text: &mut String, msg: String) -> Report {
    let _ = writeln!(text, "verdict         MALFORMED — {msg}");
    Report {
        text: std::mem::take(text),
        code: MALFORMED,
    }
}

fn rp_path(root: &Path, id: &str) -> PathBuf {
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
pub const PINNED_AXIOM_SETS: &[(&str, &str, &str)] = &[(
    "PEANO-ADD",
    "spec/peano.rlt",
    "1a3f83444a06fdfbd02c9af2a5881b47137ba102a675a4e012c84be477b10a0d",
)];

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
    let reg = load_registry(&root.join("sources/registry.yaml"))?;
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
    pub goal: Option<Formula>,
    pub theorem: Option<Theorem>,
}

/// `rlt check FILE`: replay one proof file and report every layer separately.
pub fn check_file(root: &Path, file: &Path) -> Report {
    check_file_outcome(root, file).report
}

pub fn check_file_outcome(root: &Path, file: &Path) -> CheckOutcome {
    let mut t = String::new();
    let mut out = CheckOutcome {
        report: Report {
            text: String::new(),
            code: MALFORMED,
        },
        proof: None,
        goal: None,
        theorem: None,
    };
    macro_rules! bail {
        ($msg:expr) => {{
            out.report = malformed(&mut t, $msg);
            return out;
        }};
    }
    let shown = file
        .strip_prefix(root)
        .unwrap_or(file)
        .display()
        .to_string();
    let _ = writeln!(t, "RLT check       {shown}");
    let src = match std::fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => bail!(format!("cannot read the proof file: {}", e.kind())),
    };
    let pf = match parse_proof_file(&src) {
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
    let convs = match load_conventions(&root.join("conventions")) {
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
    let po = match pf.to_proof_object(&conv.convention) {
        Ok(p) => p,
        Err(e) => bail!(e),
    };

    // Layer 3: the exact statement the kernel checks.
    let _ = writeln!(t, "checked goal    {}", po.goal);
    let _ = writeln!(t, "  kernel form   {}", po.goal.kernel_form());
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
            a.equation()
        );
    }
    let _ = writeln!(t, "  status        assumed: the SHA-256 binds these axioms' exact content to the approved file; their mathematical truth and any physical correspondence remain assumptions");
    let _ = writeln!(t, "steps           {}", po.steps.len());
    out.goal = Some(po.goal.clone());

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
            let _ = writeln!(t, "proved          {}", th.conclusion());
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
    let scope = if code == ACCEPT {
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
    let path = root.join(&r.proof);
    if r.proof.contains("..") || !r.proof.starts_with("tests/") || !r.proof.ends_with(".rlt") {
        let out = CheckOutcome {
            report: Report {
                text: String::new(),
                code: MALFORMED,
            },
            proof: None,
            goal: None,
            theorem: None,
        };
        return (
            out,
            Err(format!(
                "proof path `{}` must be a tests/…/*.rlt file",
                r.proof
            )),
        );
    }
    if std::fs::symlink_metadata(&path).is_ok_and(|m| m.file_type().is_symlink()) {
        let out = CheckOutcome {
            report: Report {
                text: String::new(),
                code: MALFORMED,
            },
            proof: None,
            goal: None,
            theorem: None,
        };
        return (out, Err(format!("proof `{}` is a symbolic link", r.proof)));
    }
    let out = check_file_outcome(root, &path);
    let res = (|| {
        let pf = out.proof.as_ref().ok_or("proof file is malformed")?;
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
        if pf.convention != r.convention {
            return Err(format!(
                "proof uses convention {}, record says {}",
                pf.convention, r.convention
            ));
        }
        let convs = load_conventions(&root.join("conventions"))?;
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
            let mut actual: Vec<Formula> = th.assumptions().values().cloned().collect();
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
    let convs = match load_conventions(&root.join("conventions")) {
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
        let _ = writeln!(
            t,
            "  {} [{}] {} — \"{}\" (evidence: {})",
            c.id,
            c.kind,
            c.status,
            c.description,
            c.evidence.len()
        );
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

/// SHA-256 of the conversation sources' excerpts (see `conversation_digest`),
/// pinned in the binary. Conversation sources have no file to check against,
/// so their exact wording is fixed here; changing it requires a reviewed code
/// change.
pub const PINNED_CONVERSATION_SHA: &str =
    "11483d5a7635e13066fb972ba602f1a5e732dc95c956be233616d7f5dca53ff2";

/// Canonical digest of every `conversation` source: id, then each excerpt's
/// text, `defines` and `split`, one field per line.
pub fn conversation_digest(reg: &BTreeMap<String, rlt_syntax::records::Source>) -> String {
    let mut buf = String::new();
    for s in reg.values().filter(|s| s.status == "conversation") {
        buf.push_str(&format!("source\t{}\n", s.id));
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

/// Correspondence record kinds.
const CORRESPONDENCE_KINDS: [&str; 3] = ["behavior", "representation", "referent"];

/// Source statuses that may document a reading-point meaning. `conversation`
/// sources are statements made in the RLT review itself and are quoted verbatim.
const CITABLE: [&str; 2] = ["available", "conversation"];

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
    let convs: BTreeMap<String, ConventionRecord> =
        match load_conventions(&root.join("conventions")) {
            Ok(c) => c,
            Err(e) => return malformed(&mut t, e),
        };
    let _ = writeln!(t, "sources         {} registered", reg.len());
    let conv = conversation_digest(&reg);
    if conv != PINNED_CONVERSATION_SHA {
        problems.push(format!(
            "conversation sources differ from the wording pinned in this binary (digest {conv}, pinned {PINNED_CONVERSATION_SHA})"
        ));
    }
    for src in reg.values() {
        if src.status == "available"
            && !src.excerpts.is_empty()
            && !src
                .location
                .starts_with("https://github.com/thinkthoughts/")
        {
            problems.push(format!(
                "{}: status available with excerpts requires a GitHub file location",
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
    for p in rps
        .iter()
        .filter(|p| p.extension().is_some_and(|x| x == "yaml"))
    {
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
            if !status.correspondence.contains(&c.status) {
                problems.push(format!(
                    "{}: {} status `{}` is outside the vocabulary",
                    rp.id, c.id, c.status
                ));
            }
            if status.referent_only.contains(&c.status) && c.kind != "referent" {
                problems.push(format!(
                    "{}: {} status `{}` applies to referent records only",
                    rp.id, c.id, c.status
                ));
            }
            if c.evidence.is_empty() && !status.without_evidence.contains(&c.status) {
                problems.push(format!(
                    "{}: {} status `{}` requires evidence",
                    rp.id, c.id, c.status
                ));
            }
            if !CORRESPONDENCE_KINDS.contains(&c.kind.as_str()) {
                problems.push(format!(
                    "{}: {} kind `{}` is outside {{behavior, representation, referent}}",
                    rp.id, c.id, c.kind
                ));
            }
            for (_, r) in &c.evidence {
                match reg.get(r) {
                    Some(src) if src.status == "available" => {}
                    _ => problems.push(format!(
                        "{}: {} evidence ref `{r}` must be an available registered source",
                        rp.id, c.id
                    )),
                }
            }
            for (ty, _) in &c.evidence {
                if !status.allowed_support.contains(ty) {
                    problems.push(format!(
                        "{}: {} evidence type `{ty}` is outside allowed_support",
                        rp.id, c.id
                    ));
                }
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

fn lean_term(t: &rlt_kernel::Term) -> String {
    use rlt_kernel::Term;
    if let Some(n) = t.as_numeral() {
        return n.to_string();
    }
    match t {
        Term::App(f, a) if f == "S" && a.len() == 1 => format!("Nat.succ {}", lean_atom(&a[0])),
        Term::App(f, a) if f == "+" && a.len() == 2 => {
            format!("{} + {}", lean_atom(&a[0]), lean_atom(&a[1]))
        }
        other => format!("sorry /- unsupported term {} -/", other.kernel_form()),
    }
}

fn lean_atom(t: &rlt_kernel::Term) -> String {
    let s = lean_term(t);
    if t.as_numeral().is_some() {
        s
    } else {
        format!("({s})")
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
        _ => None,
    }
}

fn lean_formula(f: &Formula) -> String {
    format!("{} = {}", lean_term(&f.lhs), lean_term(&f.rhs))
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
                "-- skipped {}: not accepted by the RLT kernel\n",
                f.strip_prefix(root).unwrap_or(f).display()
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
            .map(|(n, phi)| format!(" ({n} : {})", lean_formula(phi)))
            .collect();
        let tactic = if th.is_unconditional() {
            "decide"
        } else {
            "omega"
        };
        let false_hyps: Vec<&str> = th
            .assumptions()
            .iter()
            .filter(|(_, phi)| {
                matches!((nat_value(&phi.lhs), nat_value(&phi.rhs)), (Some(a), Some(b)) if a != b)
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
        let _ = writeln!(
            t,
            "-- {}  (axiom set: {})\ntheorem {name}{hyps} : {} := by {tactic}\n",
            f.strip_prefix(root).unwrap_or(f).display(),
            pf.axiom_set,
            lean_formula(th.conclusion())
        );
    }
    let _ = writeln!(t, "end RLTGenerated");
    Report { text: t, code }
}

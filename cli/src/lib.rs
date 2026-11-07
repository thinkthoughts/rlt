//! Reports for `rlt check`, `rlt written`, `rlt rp` and `rlt validate`.
//! Every report is a deterministic string so tests can compare it with a
//! golden file.

use rlt_kernel::{check, Mode, Theorem, Verdict};
use rlt_syntax::elab::{elaborate_term, Convention};
use rlt_syntax::proof::{parse_axiom_file, parse_proof_file};
use rlt_syntax::records::{
    load_conventions, load_readingpoint, load_registry, load_status, ConventionRecord, ReadingPoint,
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

/// `rlt check FILE`: replay one proof file and report every layer separately.
pub fn check_file(root: &Path, file: &Path) -> Report {
    let mut t = String::new();
    let shown = file
        .strip_prefix(root)
        .unwrap_or(file)
        .display()
        .to_string();
    let _ = writeln!(t, "RLT check       {shown}");
    let src = match std::fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => return malformed(&mut t, format!("{shown}: {e}")),
    };
    let pf = match parse_proof_file(&src) {
        Ok(p) => p,
        Err(e) => return malformed(&mut t, e),
    };
    let _ = writeln!(t, "proof id        {}", pf.id);

    // Reading-point specification: documented meanings, verbatim.
    let rp: Option<ReadingPoint> = match &pf.readingpoint {
        Some(id) => match load_readingpoint(&rp_path(root, id)) {
            Ok(r) => Some(r),
            Err(e) => return malformed(&mut t, e),
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
        Err(e) => return malformed(&mut t, e),
    };
    let Some(conv) = convs.get(&pf.convention) else {
        return malformed(
            &mut t,
            format!("convention `{}` is unregistered", pf.convention),
        );
    };
    let _ = writeln!(
        t,
        "convention      {} (authority: {})",
        conv.convention.id, conv.authority
    );
    let po = match pf.to_proof_object(&conv.convention) {
        Ok(p) => p,
        Err(e) => return malformed(&mut t, e),
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
    let axioms = match std::fs::read_to_string(root.join(&pf.axioms_file))
        .map_err(|e| format!("{}: {e}", pf.axioms_file))
        .and_then(|s| parse_axiom_file(&s))
    {
        Ok(a) => a,
        Err(e) => return malformed(&mut t, e),
    };
    let _ = writeln!(t, "axiom file      {}", pf.axioms_file);
    for a in &axioms {
        let _ = writeln!(
            t,
            "  axiom         {} : ∀ {}. {}  [{}]",
            a.name(),
            a.vars().join(" "),
            a.equation(),
            a.source()
        );
    }
    let _ = writeln!(t, "steps           {}", po.steps.len());

    let (verdict_line, code) = match check(&po, &axioms) {
        Verdict::Accepted(th) => {
            let _ = writeln!(t, "proved          {}", th.conclusion());
            let _ = writeln!(t, "assumptions     {}", gamma(&th));
            let used: Vec<&String> = th.axioms_used().iter().collect();
            let _ = writeln!(
                t,
                "axioms used     {}",
                if used.is_empty() {
                    "none".into()
                } else {
                    used.iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            );
            let kind = if th.is_unconditional() {
                "unconditional theorem"
            } else {
                "conditional theorem"
            };
            (format!("ACCEPT — {kind}"), ACCEPT)
        }
        Verdict::Rejected { step, reason } => {
            let at = step.map(|s| format!(" at step {s}")).unwrap_or_default();
            (format!("REJECT{at} — {reason}"), REJECT)
        }
    };
    let _ = writeln!(t, "verdict         {verdict_line}");

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
    Report { text: t, code }
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
    let rp = match load_readingpoint(&rp_path(root, id)) {
        Ok(r) => r,
        Err(e) => return malformed(&mut t, e),
    };
    let _ = writeln!(t, "reading point   {}", rp.id);
    let _ = writeln!(t, "meanings (verbatim, all retained)");
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
        let rep = check_file(root, &root.join(&r.proof));
        let verdict = rep
            .text
            .lines()
            .find(|l| l.starts_with("verdict"))
            .map(|l| l.trim_start_matches("verdict").trim().to_string())
            .unwrap_or_default();
        let gamma = rep
            .text
            .lines()
            .find(|l| l.starts_with("assumptions"))
            .map(|l| format!(", {}", l.trim_start_matches("assumptions").trim()))
            .unwrap_or_default();
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
    }
    // PiπΠ conventions: every stated form must appear verbatim in its source's excerpts.
    match rlt_syntax::records::load_yaml(&root.join("conventions/pi.yaml")) {
        Ok(y) => {
            let items = y["conventions"].as_vec().cloned().unwrap_or_default();
            let mut n = 0;
            for c in &items {
                let id = c["id"].as_str().unwrap_or("?");
                let src = c["source"].as_str().unwrap_or("");
                let Some(reg_src) = reg.get(src) else {
                    problems.push(format!("PiπΠ {id}: source `{src}` is unregistered"));
                    continue;
                };
                for st in c["stated"].as_vec().cloned().unwrap_or_default() {
                    let st = st.as_str().unwrap_or("");
                    if !reg_src.excerpts.iter().any(|x| x.contains(st)) {
                        problems.push(format!(
                            "PiπΠ {id}: \"{st}\" appears in zero excerpts of `{src}`"
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
        for m in rp.meanings.iter().chain(rp.examples.iter()) {
            match reg.get(&m.source) {
                None => problems.push(format!("{}: source `{}` is unregistered", rp.id, m.source)),
                Some(src)
                    if !src.excerpts.iter().any(|x| x.contains(&m.text))
                        && src.status == "available" =>
                {
                    problems.push(format!(
                        "{}: \"{}\" appears in zero registered excerpts of `{}`",
                        rp.id, m.text, m.source
                    ))
                }
                _ => {}
            }
        }
        for r in &rp.relationships {
            if !convs.contains_key(&r.convention) {
                problems.push(format!(
                    "{}: relationship {} uses unregistered convention `{}`",
                    rp.id, r.id, r.convention
                ));
            }
            if !root.join(&r.proof).exists() {
                problems.push(format!(
                    "{}: relationship {} proof file `{}` is missing",
                    rp.id, r.id, r.proof
                ));
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

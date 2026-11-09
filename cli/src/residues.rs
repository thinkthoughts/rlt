//! `rlt residues FILE` — closed quotient–remainder witnesses (RLT-003 M2).
//!
//! Four kinds of line, kept apart:
//! - `theorem`: a kernel-checked closed equation (an RLT theorem);
//! - `reading`: a registered interpretation under a residue-table convention
//!   (never a theorem, never printed with ACCEPT);
//! - `consistency`: a check of the complete classified table against the
//!   convention's registered representatives;
//! - `not established`: general claims awaiting later proof capabilities.
//!
//! Everything here is untrusted checker code: it binds records to proofs with
//! the T6 binding rule and compares closed equations' shapes.

use crate::{
    bind_proof, malformed, numeral_summands, pinned_conventions, pinned_registry, record_verdict,
    rp_path, verdict_text, Report, ACCEPT, CITABLE, REJECT,
};
use rlt_kernel::{Formula, Term};
use rlt_syntax::records::{
    load_readingpoint, load_residues, ConventionRecord, Meaning, Relationship, ResidueTable,
    Source, Witness,
};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

thread_local! {
    static ACCEPTED: RefCell<(usize, BTreeSet<String>)> = const { RefCell::new((0, BTreeSet::new())) };
}
use std::fmt::Write as _;
use std::path::Path;

/// The axiom set every M2 witness, bound, example and tally is checked under.
pub const M2_AXIOM_SET: &str = "PEANO-ARITH";

/// Statements M2 leaves open, printed on every report.
pub const NOT_ESTABLISHED: &str = "uniqueness of remainders · non-divisibility · gcd(r, 30) = 1 · completeness of the representatives · congruence · periodicity · statements about primes > 5";

fn num(t: &Term) -> Option<u64> {
    t.as_numeral()
}

fn app<'a>(t: &'a Term, f: &str) -> Option<&'a [Term]> {
    match t {
        Term::App(g, a) if g == f && a.len() == 2 => Some(a),
        _ => None,
    }
}

/// `n = d × q + s` → (n, d, q, s).
fn division_shape(g: &Formula) -> Option<(u64, u64, u64, u64)> {
    let n = num(&g.lhs)?;
    let sum = app(&g.rhs, "+")?;
    let prod = app(&sum[0], "*")?;
    Some((n, num(&prod[0])?, num(&prod[1])?, num(&sum[1])?))
}

/// `s + (k + 1) = d` → (s, k, d).
fn bound_shape(g: &Formula) -> Option<(u64, u64, u64)> {
    let outer = app(&g.lhs, "+")?;
    let inner = app(&outer[1], "+")?;
    if num(&inner[1])? != 1 {
        return None;
    }
    Some((num(&outer[0])?, num(&inner[0])?, num(&g.rhs)?))
}

/// Bind one closed, unconditional NAT-DEC equation to its proof. Returns the
/// elaborated goal when the binding holds; prints the kernel's verdict.
fn bind(
    root: &Path,
    who: &str,
    written: &str,
    proof: &str,
    t: &mut String,
    problems: &mut Vec<String>,
) -> Option<Formula> {
    let rel = Relationship {
        id: who.to_string(),
        written: written.to_string(),
        convention: "NAT-DEC".to_string(),
        proof: proof.to_string(),
        mode: "unconditional".to_string(),
        assumptions: Vec::new(),
    };
    let (out, res) = bind_proof(root, None, &rel);
    // Audit F1: record text is printed only quoted (escaped), and the
    // `theorem` label only for a goal the kernel accepted unconditionally
    // under PEANO-ARITH and that the record binds. Anything else is `refused`.
    let axioms_ok = out
        .proof
        .as_ref()
        .is_some_and(|p| p.axiom_set == M2_AXIOM_SET);
    let bound = out.report.code == ACCEPT && res.is_ok() && axioms_ok;
    let shown = match (&out.goal, bound) {
        (Some(g), true) => g.to_string(),
        _ => format!("{:?}", written),
    };
    let label = if bound {
        "theorem      "
    } else {
        "refused      "
    };
    let _ = writeln!(
        t,
        "  {label} {shown}   [{}] → {}",
        proof.escape_debug(),
        verdict_text(&out.report)
    );
    match (out.report.code, res) {
        (_, Err(e)) => {
            problems.push(format!("{who}: {e}"));
            None
        }
        (ACCEPT, Ok(())) if !axioms_ok => {
            problems.push(format!(
                "{who}: {} is checked under axiom set `{}`; M2 witnesses use {M2_AXIOM_SET}",
                proof.escape_debug(),
                out.proof
                    .as_ref()
                    .map(|p| p.axiom_set.as_str())
                    .unwrap_or("?")
            ));
            None
        }
        (ACCEPT, Ok(())) => {
            // Audit F5: count proof files by canonical path, not by spelling.
            let key = std::fs::canonicalize(root.join(proof))
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| proof.to_string());
            ACCEPTED.with(|a| {
                let mut a = a.borrow_mut();
                a.0 += 1;
                a.1.insert(key);
            });
            out.goal
        }
        (_, Ok(())) => {
            problems.push(format!(
                "{who}: the kernel did not accept {}",
                proof.escape_debug()
            ));
            None
        }
    }
}

/// Check one witness: n = d × q + s and s + (k + 1) = d. Returns s.
fn check_witness(
    root: &Path,
    who: &str,
    n: u64,
    d: u64,
    w: &Witness,
    t: &mut String,
    problems: &mut Vec<String>,
) -> Option<u64> {
    let before = problems.len();
    let g = bind(root, who, &w.written, &w.proof, t, problems);
    let s = match g.as_ref().map(division_shape) {
        Some(Some((gn, gd, _q, s))) if gn == n && gd == d => Some(s),
        Some(_) => {
            problems.push(format!(
                "{who}: `{}` must have the form {n} = {d} × q + s",
                w.written
            ));
            None
        }
        None => None,
    };
    let b = bind(
        root,
        &format!("{who} bound"),
        &w.bound,
        &w.bound_proof,
        t,
        problems,
    );
    match (s, b.as_ref().map(bound_shape)) {
        (Some(s), Some(Some((bs, _k, bd)))) if bs == s && bd == d => {}
        (Some(s), Some(_)) => problems.push(format!(
            "{who}: remainder {s} has no bound below {d}: `{}` must have the form {s} + (k + 1) = {d}",
            w.bound
        )),
        _ => {}
    }
    if problems.len() == before {
        s
    } else {
        None
    }
}

/// Problems with a residue-table convention itself: its factorization theorem
/// and its quotation of the representatives. Shared with `rlt validate`.
pub fn residue_table_problems(
    root: &Path,
    conv: &ConventionRecord,
    reg: &BTreeMap<String, Source>,
) -> Vec<String> {
    let mut p = Vec::new();
    let id = &conv.convention.id;
    let Some(rt) = &conv.residue else {
        return p;
    };
    let product: u64 = rt.primes.iter().product();
    if product != rt.modulus || rt.primes.iter().any(|&q| q < 2) {
        p.push(format!(
            "convention {id}: primes {:?} must multiply to the modulus {}",
            rt.primes, rt.modulus
        ));
    }
    let mut sorted = rt.primes.clone();
    sorted.sort();
    sorted.dedup();
    if sorted.len() != rt.primes.len() {
        p.push(format!("convention {id}: primes are listed once each"));
    }
    // The factorization theorem: modulus = product of exactly these primes.
    let mut scratch = String::new();
    let mut fp = Vec::new();
    let w = format!(
        "{} = {}",
        rt.modulus,
        rt.primes
            .iter()
            .map(|q| q.to_string())
            .collect::<Vec<_>>()
            .join(" × ")
    );
    let goal = bind(
        root,
        &format!("convention {id} factorization"),
        &w,
        &rt.factorization,
        &mut scratch,
        &mut fp,
    );
    if goal.is_none() {
        p.extend(fp);
    }
    // The representatives: an exact quotation, whole unit of the cited source.
    let q = &rt.quote;
    match reg.get(&q.source) {
        Some(src) if CITABLE.contains(&src.status.as_str()) => {
            if !src.excerpts.iter().any(|x| x.units().contains(&q.text)) {
                p.push(format!(
                    "convention {id}: representatives_quote \"{}\" is not a whole unit of `{}`",
                    q.text, q.source
                ));
            }
        }
        _ => p.push(format!(
            "convention {id}: representatives_quote cites `{}`, which must be an available or conversation source",
            q.source
        )),
    }
    if !conv.sources.contains(&q.source) {
        p.push(format!(
            "convention {id}: representatives_quote source `{}` is missing from the convention's sources",
            q.source
        ));
    }
    let quoted: Option<Vec<u64>> = q
        .text
        .strip_prefix('{')
        .and_then(|r| r.split_once('}'))
        .map(|(inner, _)| {
            inner
                .split(',')
                .map(|x| x.trim().parse::<u64>().ok())
                .collect()
        })
        .unwrap_or(None);
    if quoted.as_deref() != Some(rt.representatives.as_slice()) {
        p.push(format!(
            "convention {id}: representatives {:?} differ from the quoted set \"{}\"",
            rt.representatives, q.text
        ));
    }
    p
}

/// The registered classification of one residue from its bounded remainders.
pub fn classify(rt: &ResidueTable, remainders: &[(u64, u64)]) -> (bool, String) {
    let zero: Vec<String> = remainders
        .iter()
        .filter(|(_, s)| *s == 0)
        .map(|(p, _)| p.to_string())
        .collect();
    let shown: Vec<String> = remainders.iter().map(|(_, s)| s.to_string()).collect();
    if zero.is_empty() {
        (
            true,
            format!(
                "remainders {} (all nonzero): coprime representative of {}",
                shown.join(", "),
                rt.modulus
            ),
        )
    } else {
        (
            false,
            format!(
                "remainders {}: zero remainder for {}, so not a coprime representative of {}",
                shown.join(", "),
                zero.join(", "),
                rt.modulus
            ),
        )
    }
}

pub const READING: &str = "registered reading (not a theorem)";

pub fn residues_check(root: &Path, file: &Path) -> (String, Vec<String>, (usize, usize)) {
    let mut tally = (0usize, 0usize);
    ACCEPTED.with(|a| *a.borrow_mut() = (0, BTreeSet::new()));
    let mut t = String::new();
    let mut problems: Vec<String> = Vec::new();
    let shown = file
        .strip_prefix(root)
        .unwrap_or(file)
        .display()
        .to_string()
        .escape_debug()
        .to_string();
    let _ = writeln!(t, "RLT residues    {shown}");
    let in_dir = std::fs::symlink_metadata(root.join("residues")).is_ok_and(|m| m.is_dir())
        && match (
            file.parent().and_then(|p| p.canonicalize().ok()),
            root.join("residues").canonicalize(),
        ) {
            (Some(a), Ok(b)) => a == b,
            _ => false,
        }
        && file.extension().is_some_and(|x| x == "yaml");
    if !in_dir {
        problems.push(format!(
            "{shown}: a residues file is a *.yaml file inside this repository's residues/ directory"
        ));
        return (t, problems, tally);
    }
    let rf = match load_residues(file) {
        Ok(r) => r,
        Err(e) => {
            problems.push(e);
            return (t, problems, tally);
        }
    };
    let (reg, convs) = match (pinned_registry(root), pinned_conventions(root)) {
        (Ok(r), Ok(c)) => (r, c),
        (Err(e), _) | (_, Err(e)) => {
            problems.push(e);
            return (t, problems, tally);
        }
    };

    // Context: basis, quotations, references.
    let b = &rf.basis;
    let _ = writeln!(t, "context         {}", rf.id);
    let _ = writeln!(
        t,
        "  basis         {:?}  [{}]",
        b.text,
        b.source.escape_debug()
    );
    let cite = |m: &Meaning, what: &str, problems: &mut Vec<String>| match reg.get(&m.source) {
        Some(src) if CITABLE.contains(&src.status.as_str()) => {
            match src.excerpts.iter().find(|x| x.units().contains(&m.text)) {
                Some(x) => x.defines.clone(),
                None => {
                    problems.push(format!(
                        "{}: {what} \"{}\" is not a whole unit of `{}`",
                        rf.id, m.text, m.source
                    ));
                    None
                }
            }
        }
        _ => {
            problems.push(format!(
                "{}: {what} source `{}` must be an available or conversation source",
                rf.id, m.source
            ));
            None
        }
    };
    cite(b, "basis", &mut problems);
    for q in &rf.quotes {
        let defines = cite(q, "quotation", &mut problems);
        let _ = writeln!(
            t,
            "  quotation     {:?}  [{}]{} (quoted; the definition stays with its source)",
            q.text,
            q.source.escape_debug(),
            defines
                .map(|d| format!(" defines {d}; readingpoints/{d}.yaml is unchanged"))
                .unwrap_or_default()
        );
    }
    for r in &rf.references {
        let ok = std::fs::symlink_metadata(root.join("readingpoints")).is_ok_and(|m| m.is_dir())
            && load_readingpoint(&rp_path(root, r)).is_ok();
        if ok {
            let _ = writeln!(
                t,
                "  references    {r} (its documented meanings stay in readingpoints/{r}.yaml)"
            );
        } else {
            problems.push(format!("{}: references unknown reading point `{r}`", rf.id));
        }
    }
    let (_, line) = record_verdict(0, &problems, &mut tally);
    let _ = writeln!(t, "  {line} (context: basis, quotations, references)");

    // Convention.
    let c0 = problems.len();
    let conv = convs.get(&rf.convention);
    let rt = match conv.and_then(|c| c.residue.as_ref()) {
        Some(rt) => rt,
        None => {
            problems.push(format!(
                "{}: convention `{}` must be a registered residue-table convention",
                rf.id, rf.convention
            ));
            return (t, problems, tally);
        }
    };
    let conv = conv.unwrap();
    problems.extend(residue_table_problems(root, conv, &reg));
    let _ = writeln!(
        t,
        "convention      {} (interpretation input; pinned) — modulus {} = {} [theorem {}] · rule {}",
        conv.convention.id,
        rt.modulus,
        rt.primes.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(" × "),
        rt.factorization,
        rt.rule
    );
    let _ = writeln!(
        t,
        "  representatives {:?} quoted from [{}] {:?}",
        rt.representatives,
        rt.quote.source.escape_debug(),
        rt.quote.text
    );
    let (_, line) = record_verdict(c0, &problems, &mut tally);
    let _ = writeln!(t, "  {line} (convention)");

    // Residues: coverage, witnesses, readings.
    let mut seen: Vec<u64> = Vec::new();
    let mut coprime: Vec<u64> = Vec::new();
    let mut all_classified = true;
    for e in &rf.residues {
        let r0 = problems.len();
        let _ = writeln!(t, "residue {}", e.r);
        if e.r >= rt.modulus {
            problems.push(format!(
                "{}: residue {} is outside 0 to {}",
                rf.id,
                e.r,
                rt.modulus - 1
            ));
        }
        if seen.contains(&e.r) {
            problems.push(format!("{}: residue {} is listed twice", rf.id, e.r));
        }
        seen.push(e.r);
        let mut remainders: Vec<(u64, u64)> = Vec::new();
        for &p in &rt.primes {
            let ws: Vec<&Witness> = e.witnesses.iter().filter(|w| w.divisor == p).collect();
            match ws.as_slice() {
                [w] => {
                    let who = format!("{}: residue {} witness for {p}", rf.id, e.r);
                    if let Some(s) = check_witness(root, &who, e.r, p, w, &mut t, &mut problems) {
                        remainders.push((p, s));
                    }
                }
                [] => problems.push(format!(
                    "{}: residue {}: missing witness for {p}",
                    rf.id, e.r
                )),
                _ => problems.push(format!(
                    "{}: residue {}: {} witnesses for {p}; one per prime",
                    rf.id,
                    e.r,
                    ws.len()
                )),
            }
        }
        for w in &e.witnesses {
            if !rt.primes.contains(&w.divisor) {
                problems.push(format!(
                    "{}: residue {}: witness for {} is outside the primes {:?}",
                    rf.id, e.r, w.divisor, rt.primes
                ));
            }
        }
        let (valid, line) = record_verdict(r0, &problems, &mut tally);
        if valid {
            let (is_coprime, text) = classify(rt, &remainders);
            if is_coprime {
                coprime.push(e.r);
            }
            let _ = writeln!(t, "  reading       {text} — {READING}");
        } else {
            all_classified = false;
            let _ = writeln!(t, "  reading       not reported (record invalid)");
        }
        let _ = writeln!(t, "  {line}");
    }
    let k0 = problems.len();
    let expected: Vec<u64> = (0..rt.modulus).collect();
    let mut sorted = seen.clone();
    sorted.sort();
    sorted.dedup();
    if sorted != expected {
        problems.push(format!(
            "{}: residues must cover 0 to {} exactly once each",
            rf.id,
            rt.modulus - 1
        ));
    }
    coprime.sort();
    let _ = writeln!(t, "classification");
    if all_classified && sorted == expected {
        if coprime == rt.representatives {
            let _ = writeln!(
                t,
                "  consistency   the registered classification of the {} entries gives {:?} = the registered representatives ({} of {}) — consistency check, not a theorem",
                rt.modulus,
                coprime,
                coprime.len(),
                rt.modulus
            );
        } else {
            problems.push(format!(
                "{}: the classification gives {:?}; the registered representatives are {:?}",
                rf.id, coprime, rt.representatives
            ));
        }
    } else {
        let _ = writeln!(
            t,
            "  consistency   not reported (the residue table is incomplete or holds invalid entries)"
        );
    }
    let (_, line) = record_verdict(k0, &problems, &mut tally);
    let _ = writeln!(t, "  {line}");

    // Class witnesses: n = modulus × q + r.
    let mut class_seen: Vec<u64> = Vec::new();
    for c in &rf.classes {
        let k0 = problems.len();
        let _ = writeln!(t, "class witness {}", c.n);
        if class_seen.contains(&c.n) {
            problems.push(format!("{}: class witness {} is listed twice", rf.id, c.n));
        }
        class_seen.push(c.n);
        let who = format!("{}: class witness {}", rf.id, c.n);
        let g = check_witness(
            root,
            &who,
            c.n,
            rt.modulus,
            &c.witness,
            &mut t,
            &mut problems,
        );
        let (valid, line) = record_verdict(k0, &problems, &mut tally);
        match g {
            Some(r) if valid => {
                let _ = writeln!(
                    t,
                    "  reading       {} lies in the residue class of {r} modulo {} — {READING}",
                    c.n, rt.modulus
                );
            }
            _ => {
                let _ = writeln!(t, "  reading       not reported (record invalid)");
            }
        }
        let _ = writeln!(t, "  {line}");
    }

    // Example and tally.
    if let Some(ex) = &rf.example {
        let k0 = problems.len();
        let _ = writeln!(t, "example");
        let g = bind(
            root,
            &format!("{}: example", rf.id),
            &ex.written,
            &ex.proof,
            &mut t,
            &mut problems,
        );
        // The example states the modulus as a closed product, as math29's
        // (1 + 1) × 3 × 5 = 30 does.
        if let Some(g) = g {
            if g.rhs.as_numeral() != Some(rt.modulus) || app(&g.lhs, "*").is_none() {
                problems.push(format!(
                    "{}: the example must be a closed product equal to the modulus {}",
                    rf.id, rt.modulus
                ));
            }
        }
        let (_, line) = record_verdict(k0, &problems, &mut tally);
        let _ = writeln!(t, "  {line}");
    }
    if let Some((c, binds)) = &rf.tally {
        let k0 = problems.len();
        let _ = writeln!(t, "tally");
        let g = bind(
            root,
            &format!("{}: tally", rf.id),
            &c.written,
            &c.proof,
            &mut t,
            &mut problems,
        );
        if let Some(g) = g {
            let ones = numeral_summands(&g.lhs);
            let mut b = binds.clone();
            b.sort();
            let dup = b.windows(2).any(|w| w[0] == w[1]);
            let shape = ones.as_ref().is_some_and(|v| v.iter().all(|&x| x == 1))
                && ones.as_ref().map(|v| v.len() as u64) == g.rhs.as_numeral()
                && ones.as_ref().map(|v| v.len()) == Some(binds.len());
            if !shape || dup || b != rt.representatives {
                problems.push(format!(
                    "{}: the tally must be 1 + … + 1 = n with one summand per registered representative, binding each once",
                    rf.id
                ));
            } else {
                let _ = writeln!(
                    t,
                    "  binding       one summand per representative {binds:?}: counts the registered representatives, not all coprime residues"
                );
            }
        }
        let (_, line) = record_verdict(k0, &problems, &mut tally);
        let _ = writeln!(t, "  {line}");
    }
    let _ = writeln!(t, "not established {NOT_ESTABLISHED}");
    let (bindings, files) = ACCEPTED.with(|a| {
        let a = a.borrow();
        (a.0, a.1.len())
    });
    let _ = writeln!(
        t,
        "theorems        {bindings} kernel-checked bindings over {files} distinct proof files (closed equations, PEANO-ARITH, Γ = ∅)"
    );
    (t, problems, tally)
}

pub fn residues_report(root: &Path, file: &Path) -> Report {
    let (mut t, problems, (records, invalid)) = residues_check(root, file);
    if records == 0 && !problems.is_empty() {
        return malformed(&mut t, problems.join("; "));
    }
    for p in &problems {
        let _ = writeln!(t, "problem         {p}");
    }
    if problems.is_empty() {
        let _ = writeln!(
            t,
            "verdict         VALID — witness records consistent ({records} record(s) valid; readings are registered interpretations, not theorems)"
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

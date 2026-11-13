//! RLT-007d: `rlt compose BUNDLE` — layered composition of kernel-checked
//! theorems (approved architecture: docs/RLT-007d-FINAL.md).
//!
//! The composition checker is new trusted code, limited to: reading one
//! bundle record, checking its dependency graph, re-checking every proof it
//! names in this run through the existing check functions, and comparing
//! canonical kernel formulas (`deps::canonical`, bound variables as de Bruijn
//! indices). It builds no formula, applies no inference rule and has no access
//! to the kernel's theorem constructor. Its result is a set of statements, each
//! either kernel accepted or discharged by composition; it is never reported as
//! one unconditional kernel theorem.
//!
//! Soundness (FINAL §10): a layer-0 node is an accepted unconditional kernel
//! theorem; a layer-n node is an accepted conditional theorem Γ ⊢ φ whose
//! every γ ∈ Γ is, canonically and under the same axioms, convention and
//! definitions, the statement of a node of lower layer that passed in this
//! run. By kernel soundness and induction on layers, every node's statement
//! holds in every model of the axiom set.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

use yaml_rust2::Yaml;

use crate::deps::{canonical, fingerprint_text, load_library, statement_parts};
use crate::explain::{kernel_fields, STALE};
use crate::{
    check_preread_outcome, sha256_hex, ACCEPT, MALFORMED, PINNED_AXIOM_SETS, PINNED_CONVENTIONS,
    PINNED_DEFINITIONS, REJECT,
};

pub const BUNDLE_FORMAT: &str = "rlt-compose/1";

/// The register of approved specification probes (RLT-007d), one line per
/// node id: `id<TAB>path<TAB>sha256`. Fixed: a bundle cannot choose it.
pub const APPROVED_REGISTER: &str = "compose/approved-specifications.tsv";

/// The register of named results: `id<TAB>node,node,…<TAB>note`. A bundle
/// composes only under a registered id, with exactly its registered result.
pub const RESULTS_REGISTER: &str = "compose/approved-results.tsv";

/// Refusal classes, shared with the independent checker
/// (scripts/compose-check.py) so the two can be compared case by case.
pub const CLASSES: &[&str] = &[
    "malformed",
    "missing",
    "stale",
    "conditional",
    "duplicated",
    "unused",
    "graph",
    "incompatible",
    "kernel",
    "mismatch",
    "specification",
];

#[derive(Clone, Debug)]
pub struct Pin {
    pub path: String,
    pub sha256: String,
}

#[derive(Clone, Debug)]
pub struct Discharge {
    pub assumption: String,
    pub by: String,
}

#[derive(Clone, Debug)]
pub struct BNode {
    pub id: String,
    pub layer: usize,
    pub library: Option<String>,
    pub fingerprint: Option<String>,
    pub assembly: Option<Pin>,
    pub discharge: Vec<Discharge>,
    pub spec: Pin,
}

#[derive(Clone, Debug)]
pub struct Bundle {
    pub id: String,
    pub axiom_set: String,
    pub convention: String,
    pub definitions: Pin,
    pub nodes: Vec<BNode>,
    pub result: Vec<String>,
}

/// One node as checked in this run.
#[derive(Clone, Debug)]
pub struct Checked {
    pub id: String,
    pub layer: usize,
    pub source: String,
    pub statement: String,
    pub printed: String,
    pub label: String,
    pub edges: Vec<(String, String, usize)>,
}

#[derive(Clone, Debug)]
pub struct Composition {
    pub bundle: String,
    pub id: String,
    pub checked: Vec<Checked>,
    /// The result's node ids, in the bundle's order.
    pub result: Vec<String>,
    /// `(class, reason)` of the first refusal, if any.
    pub refusal: Option<(String, String)>,
    /// The registered note of the named result (what lies outside this check).
    pub note: String,
    pub exit: i32,
}

fn s(y: &Yaml, key: &str, ctx: &str) -> Result<String, String> {
    match &y[key] {
        Yaml::String(v) => Ok(v.clone()),
        Yaml::BadValue => Err(format!("{ctx}: `{key}` is missing")),
        _ => Err(format!("{ctx}: `{key}` must be a string")),
    }
}

fn keys(y: &Yaml, allowed: &[&str], required: &[&str], ctx: &str) -> Result<(), String> {
    let Yaml::Hash(h) = y else {
        return Err(format!("{ctx}: expected a mapping"));
    };
    for k in h.keys() {
        let Yaml::String(k) = k else {
            return Err(format!("{ctx}: keys are strings"));
        };
        if !allowed.contains(&k.as_str()) {
            return Err(format!("{ctx}: unknown key `{k}`"));
        }
    }
    for r in required {
        if y[*r].is_badvalue() {
            return Err(format!("{ctx}: `{r}` is missing"));
        }
    }
    Ok(())
}

fn pin(y: &Yaml, ctx: &str) -> Result<Pin, String> {
    keys(y, &["path", "sha256"], &["path", "sha256"], ctx)?;
    let p = Pin {
        path: s(y, "path", ctx)?,
        sha256: s(y, "sha256", ctx)?,
    };
    let ok_path = !p.path.starts_with('/')
        && p.path
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_./-".contains(&b))
        && p.path
            .split('/')
            .all(|c| !c.is_empty() && c != "." && c != "..");
    if !ok_path {
        return Err(format!(
            "{ctx}: `{}` is not a repository-relative path",
            p.path
        ));
    }
    if p.sha256.len() != 64
        || !p
            .sha256
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err(format!("{ctx}: sha256 must be 64 lower-case hex digits"));
    }
    Ok(p)
}

/// Read a bundle record. Any structural problem is `malformed`.
pub fn load_bundle(path: &Path) -> Result<Bundle, String> {
    let y = rlt_syntax::records::load_yaml(path)?;
    let ctx = "bundle";
    keys(
        &y,
        &[
            "format",
            "id",
            "axiom_set",
            "convention",
            "definitions",
            "nodes",
            "result",
        ],
        &[
            "format",
            "id",
            "axiom_set",
            "convention",
            "definitions",
            "nodes",
            "result",
        ],
        ctx,
    )?;
    if s(&y, "format", ctx)? != BUNDLE_FORMAT {
        return Err(format!("{ctx}: format must be {BUNDLE_FORMAT}"));
    }
    let Yaml::Array(ns) = &y["nodes"] else {
        return Err(format!("{ctx}: `nodes` must be a list"));
    };
    let mut nodes = Vec::new();
    for (i, n) in ns.iter().enumerate() {
        let c = format!("{ctx}: node {}", i + 1);
        keys(
            n,
            &[
                "id",
                "layer",
                "library",
                "fingerprint",
                "assembly",
                "discharge",
                "spec",
            ],
            &["id", "layer", "spec"],
            &c,
        )?;
        let id = s(n, "id", &c)?;
        let layer = match &n["layer"] {
            Yaml::Integer(v) if *v >= 0 && *v < 64 => *v as usize,
            _ => return Err(format!("{c}: `layer` must be an integer 0 … 63")),
        };
        let mut b = BNode {
            id: id.clone(),
            layer,
            library: None,
            fingerprint: None,
            assembly: None,
            discharge: vec![],
            spec: pin(&n["spec"], &format!("{c} spec"))?,
        };
        if layer == 0 {
            if !n["assembly"].is_badvalue() || !n["discharge"].is_badvalue() {
                return Err(format!(
                    "{c} (`{id}`): a layer-0 node names a library theorem, not an assembly"
                ));
            }
            b.library = Some(s(n, "library", &c)?);
            b.fingerprint = Some(s(n, "fingerprint", &c)?);
        } else {
            if !n["library"].is_badvalue() || !n["fingerprint"].is_badvalue() {
                return Err(format!(
                    "{c} (`{id}`): a composed node names an assembly, not a library theorem"
                ));
            }
            b.assembly = Some(pin(&n["assembly"], &format!("{c} assembly"))?);
            let Yaml::Array(ds) = &n["discharge"] else {
                return Err(format!("{c} (`{id}`): `discharge` must be a list"));
            };
            for (j, d) in ds.iter().enumerate() {
                let dc = format!("{c} discharge {}", j + 1);
                keys(d, &["assumption", "by"], &["assumption", "by"], &dc)?;
                b.discharge.push(Discharge {
                    assumption: s(d, "assumption", &dc)?,
                    by: s(d, "by", &dc)?,
                });
            }
        }
        nodes.push(b);
    }
    let Yaml::Array(rs) = &y["result"] else {
        return Err(format!("{ctx}: `result` must be a list"));
    };
    let mut result = Vec::new();
    for r in rs {
        match r {
            Yaml::String(v) => result.push(v.clone()),
            _ => return Err(format!("{ctx}: `result` lists node ids")),
        }
    }
    Ok(Bundle {
        id: s(&y, "id", ctx)?,
        axiom_set: s(&y, "axiom_set", ctx)?,
        convention: s(&y, "convention", ctx)?,
        definitions: pin(&y["definitions"], "bundle definitions")?,
        nodes,
        result,
    })
}

fn file_sha(root: &Path, p: &Pin) -> Result<(String, Vec<u8>), (String, String)> {
    let full = root.join(&p.path);
    // Inside the repository: no component of the path may be a symbolic link.
    let mut at = root.to_path_buf();
    let comps: Vec<&str> = p.path.split('/').collect();
    for c in &comps[..comps.len().saturating_sub(1)] {
        at.push(c);
        if std::fs::symlink_metadata(&at).is_ok_and(|m| m.file_type().is_symlink()) {
            return Err((
                "malformed".into(),
                format!(
                    "{}: `{c}` is a symbolic link; pinned files lie inside the repository",
                    p.path
                ),
            ));
        }
    }
    let md = std::fs::symlink_metadata(&full)
        .map_err(|_| ("missing".to_string(), format!("{}: file not found", p.path)))?;
    if !md.file_type().is_file() {
        return Err(("missing".into(), format!("{}: not a regular file", p.path)));
    }
    let bytes = std::fs::read(&full)
        .map_err(|e| ("missing".to_string(), format!("{}: {}", p.path, e.kind())))?;
    Ok((sha256_hex(&bytes), bytes))
}

/// The parts every node must share with the bundle, as `(axiom set,
/// axiom sha, convention digest, definitions file, definitions sha)`; the
/// definitions are compared only when the statement uses one.
struct Pins {
    axiom_set: String,
    axiom_sha: String,
    convention: String,
    convention_digest: String,
    defs_file: String,
    defs_sha: String,
}

fn parts_ok(p: &crate::deps::StatementParts, w: &Pins) -> Result<(), String> {
    if p.axiom_set != w.axiom_set || p.axiom_sha256 != w.axiom_sha {
        return Err(format!(
            "axiom set {} ({}) differs from the bundle's {} ({})",
            p.axiom_set,
            &p.axiom_sha256[..12.min(p.axiom_sha256.len())],
            w.axiom_set,
            &w.axiom_sha[..12]
        ));
    }
    if p.convention != w.convention || p.convention_digest != w.convention_digest {
        return Err(format!(
            "convention {} differs from the bundle's {}",
            p.convention, w.convention
        ));
    }
    if let Some((f, h, _)) = &p.definitions {
        if *f != w.defs_file || *h != w.defs_sha {
            return Err(format!("definitions {f} ({h}) differ from the bundle's"));
        }
    }
    Ok(())
}

/// `rlt compose BUNDLE`.
pub fn compose(root: &Path, bundle_path: &str) -> Composition {
    let mut c = Composition {
        bundle: bundle_path.to_string(),
        id: String::new(),
        checked: vec![],
        result: vec![],
        refusal: None,
        note: String::new(),
        exit: MALFORMED,
    };
    macro_rules! refuse {
        ($class:expr, $msg:expr) => {{
            let class: &str = $class;
            c.exit = match class {
                "malformed" | "missing" => MALFORMED,
                "stale" => STALE,
                _ => REJECT,
            };
            c.refusal = Some((class.to_string(), $msg));
            return c;
        }};
    }
    let b = match load_bundle(&root.join(bundle_path)) {
        Ok(b) => b,
        Err(e) => refuse!("malformed", e),
    };
    c.id = b.id.clone();
    c.result = b.result.clone();

    // ---- Phase 1: structure, pins and the graph (no kernel work). ----
    let Some((_, ax_file, ax_sha)) = PINNED_AXIOM_SETS.iter().find(|(a, _, _)| *a == b.axiom_set)
    else {
        refuse!(
            "incompatible",
            format!("axiom set `{}` is not registered", b.axiom_set)
        );
    };
    let _ = ax_file;
    let Some((_, conv_digest, _)) = PINNED_CONVENTIONS
        .iter()
        .find(|(x, _, _)| *x == b.convention)
    else {
        refuse!(
            "incompatible",
            format!("convention `{}` is not registered", b.convention)
        );
    };
    if b.definitions.path != PINNED_DEFINITIONS.0 || b.definitions.sha256 != PINNED_DEFINITIONS.1 {
        refuse!(
            "incompatible",
            format!(
                "definitions {} ({}) are not the pinned {} ({})",
                b.definitions.path,
                b.definitions.sha256,
                PINNED_DEFINITIONS.0,
                PINNED_DEFINITIONS.1
            )
        );
    }
    match file_sha(root, &b.definitions) {
        Ok((h, _)) if h == b.definitions.sha256 => {}
        Ok((h, _)) => refuse!(
            "stale",
            format!(
                "{} has SHA-256 {h}, not the pinned {}",
                b.definitions.path, b.definitions.sha256
            )
        ),
        Err((k, m)) => refuse!(&k, m),
    }
    let want = Pins {
        axiom_set: b.axiom_set.clone(),
        axiom_sha: ax_sha.to_string(),
        convention: b.convention.clone(),
        convention_digest: conv_digest.to_string(),
        defs_file: PINNED_DEFINITIONS.0.to_string(),
        defs_sha: PINNED_DEFINITIONS.1.to_string(),
    };

    let mut by_id: BTreeMap<&str, &BNode> = BTreeMap::new();
    for n in &b.nodes {
        if by_id.insert(&n.id, n).is_some() {
            refuse!("duplicated", format!("node id `{}` appears twice", n.id));
        }
    }
    let mut libs = BTreeMap::new();
    let mut files = BTreeMap::new();
    for n in &b.nodes {
        if let Some(l) = &n.library {
            if let Some(o) = libs.insert(l.clone(), n.id.clone()) {
                refuse!(
                    "duplicated",
                    format!(
                        "library theorem `{l}` is named by nodes `{o}` and `{}`",
                        n.id
                    )
                );
            }
        }
        if let Some(a) = &n.assembly {
            if let Some(o) = files.insert(a.path.clone(), n.id.clone()) {
                refuse!(
                    "duplicated",
                    format!("assembly {} is named by nodes `{o}` and `{}`", a.path, n.id)
                );
            }
        }
    }
    // Discharge tables: one row per assumption name; each row names an
    // existing node of a lower layer; the stated layer is 1 + the highest.
    let mut referenced: BTreeSet<&str> = BTreeSet::new();
    for n in &b.nodes {
        let mut names = BTreeSet::new();
        let mut top = 0;
        for d in &n.discharge {
            if !names.insert(d.assumption.as_str()) {
                refuse!(
                    "duplicated",
                    format!(
                        "node `{}`: assumption `{}` has two discharge rows",
                        n.id, d.assumption
                    )
                );
            }
            let Some(u) = by_id.get(d.by.as_str()) else {
                refuse!("missing", format!("node `{}`: assumption `{}` is discharged by `{}`, which is not a node of this bundle", n.id, d.assumption, d.by));
            };
            if u.layer >= n.layer {
                refuse!("graph", format!("node `{}` (layer {}): assumption `{}` is discharged by `{}` of layer {}, not a lower layer", n.id, n.layer, d.assumption, u.id, u.layer));
            }
            top = top.max(u.layer);
            referenced.insert(u.id.as_str());
        }
        if n.layer > 0 {
            if n.discharge.is_empty() {
                refuse!(
                    "graph",
                    format!(
                        "node `{}` (layer {}) discharges nothing; a composed node has assumptions",
                        n.id, n.layer
                    )
                );
            }
            if n.layer != top + 1 {
                refuse!(
                    "graph",
                    format!(
                        "node `{}` states layer {}, but its highest discharger is of layer {top}",
                        n.id, n.layer
                    )
                );
            }
        }
    }
    let mut seen_r = BTreeSet::new();
    if b.result.is_empty() {
        refuse!("malformed", "bundle: `result` is empty".into());
    }
    for r in &b.result {
        if !by_id.contains_key(r.as_str()) {
            refuse!(
                "missing",
                format!("result names `{r}`, which is not a node of this bundle")
            );
        }
        if !seen_r.insert(r.as_str()) {
            refuse!("duplicated", format!("result names `{r}` twice"));
        }
        referenced.insert(r);
    }
    for n in &b.nodes {
        if !referenced.contains(n.id.as_str()) {
            refuse!(
                "unused",
                format!(
                    "node `{}` discharges no assumption and is not part of the result",
                    n.id
                )
            );
        }
    }
    // Library entries: present, citable, unconditional, same fingerprint.
    let lib = match load_library(root) {
        Ok(l) => l,
        Err(e) => refuse!("malformed", format!("lib/ index invalid: {e}")),
    };
    for n in &b.nodes {
        if let Some(l) = &n.library {
            let Some(e) = lib.get(l) else {
                refuse!(
                    "missing",
                    format!("node `{}`: `{l}` is not a library id", n.id)
                );
            };
            if !e.citable || !e.assumptions.is_empty() {
                refuse!("conditional", format!("node `{}`: `{l}` is a conditional theorem ({} assumptions); layer 0 takes unconditional theorems only", n.id, e.assumptions.len()));
            }
            if Some(&e.fingerprint) != n.fingerprint.as_ref() {
                refuse!("stale", format!("node `{}`: the bundle pins fingerprint {}, but `{l}` is registered with {}", n.id, n.fingerprint.as_deref().unwrap_or(""), e.fingerprint));
            }
        }
    }
    // File pins.
    for n in &b.nodes {
        for p in n.assembly.iter().chain(std::iter::once(&n.spec)) {
            match file_sha(root, p) {
                Ok((h, _)) if h == p.sha256 => {}
                Ok((h, _)) => refuse!(
                    "stale",
                    format!(
                        "node `{}`: {} has SHA-256 {h}, not the pinned {}",
                        n.id, p.path, p.sha256
                    )
                ),
                Err((k, m)) => refuse!(&k, format!("node `{}`: {m}", n.id)),
            }
        }
    }

    // ---- The approved specifications: each probe re-checked, before any
    // node, so a statement that differs from its specification is refused
    // as soon as that node is checked. ----
    // The register of approved specifications is a fixed, reviewed file: a
    // bundle cannot name its own. Every node must carry the registered
    // probe for its id, so a weaker or different statement paired with a
    // matching probe of its own is still refused.
    let register = match read_register(root, APPROVED_REGISTER, 3) {
        Ok(r) => r,
        Err((k, m)) => refuse!(&k, m),
    };
    for n in &b.nodes {
        match register.get(&n.id) {
            None => refuse!("specification", format!("node `{}` has no approved specification in {APPROVED_REGISTER}", n.id)),
            Some(f) if f[0] != n.spec.path || f[1] != n.spec.sha256 => refuse!(
                "specification",
                format!("node `{}`: specification {} is not the approved specification {} of {APPROVED_REGISTER}", n.id, n.spec.path, f[0])
            ),
            Some(_) => {}
        }
    }
    let mut spec_goal: BTreeMap<String, String> = BTreeMap::new();
    for n in &b.nodes {
        let (ssha, sbytes) = match file_sha(root, &n.spec) {
            Ok(x) => x,
            Err((k, m)) => refuse!(&k, m),
        };
        if ssha != n.spec.sha256 {
            refuse!(
                "stale",
                format!(
                    "node `{}`: {} has SHA-256 {ssha}, not the pinned {}",
                    n.id, n.spec.path, n.spec.sha256
                )
            );
        }
        let ssrc = match String::from_utf8(sbytes) {
            Ok(s) => s,
            Err(_) => refuse!("malformed", format!("{}: not UTF-8", n.spec.path)),
        };
        let sp = match probe_parts(root, &n.spec.path, ssrc, &ssha) {
            Ok(sp) => sp,
            Err(m) => refuse!("specification", format!("node `{}`: {m}", n.id)),
        };
        if let Err(e) = parts_ok(&sp, &want) {
            refuse!(
                "incompatible",
                format!("node `{}`: specification {}: {e}", n.id, n.spec.path)
            );
        }
        spec_goal.insert(n.id.clone(), sp.goal.clone());
    }

    // ---- Phase 2: every proof re-checked in this run. The assemblies come
    // first (each is quick, and a statement that is not the approved one is
    // refused before the long component re-checks), then the components;
    // the discharges are compared once every statement is known. ----
    let mut order: Vec<&BNode> = b.nodes.iter().collect();
    order.sort_by_key(|n| (n.layer == 0, n.layer));
    let mut pending: Vec<(usize, String, String, String, String)> = vec![]; // layer, node, assumption, canonical γ, discharger
    let mut stmt: BTreeMap<String, String> = BTreeMap::new(); // id → canonical statement
    let mut owner: BTreeMap<String, String> = BTreeMap::new(); // canonical → id
    for n in order {
        let (canon, printed, source, label, edges);
        if let Some(l) = &n.library {
            // Re-check the registered proof (and, through its pinned
            // citations, every file it cites) in this run.
            let e = lib.get(l).expect("phase 1");
            let pp = Pin {
                path: e.proof.path.clone(),
                sha256: e.proof.sha256.clone(),
            };
            let (sha, bytes) = match file_sha(root, &pp) {
                Ok(x) => x,
                Err((k, m)) => refuse!(&k, format!("node `{}`: `{l}`: {m}", n.id)),
            };
            if sha != pp.sha256 {
                refuse!(
                    "stale",
                    format!(
                        "node `{}`: `{l}`: {} has SHA-256 {sha}, not the registered {}",
                        n.id, pp.path, pp.sha256
                    )
                );
            }
            let src = match String::from_utf8(bytes) {
                Ok(s) => s,
                Err(_) => refuse!("malformed", format!("{}: not UTF-8", pp.path)),
            };
            let out = check_preread_outcome(root, &root.join(&pp.path), src);
            let k = kernel_fields(&out, &pp.path, &sha);
            if out.report.code != ACCEPT {
                let class = if out.report.code == STALE || citation_problem(&out, " has SHA-256 ") {
                    "stale"
                } else if citation_problem(&out, ": entity not found") {
                    "missing"
                } else {
                    "kernel"
                };
                refuse!(
                    class,
                    format!(
                        "node `{}`: `{l}` ({}) is not accepted ({})",
                        n.id, pp.path, k.verdict_line
                    )
                );
            }
            let Some(parts) = statement_parts(&out, &k) else {
                refuse!("kernel", format!("node `{}`: `{l}` has no statement", n.id));
            };
            let text = fingerprint_text(&parts);
            let fp = sha256_hex(text.as_bytes());
            if Some(&fp) != n.fingerprint.as_ref() {
                refuse!(
                    "stale",
                    format!(
                        "node `{}`: `{l}` now has fingerprint {fp}, not the pinned one",
                        n.id
                    )
                );
            }
            if !parts.assumptions.is_empty() {
                refuse!(
                    "conditional",
                    format!("node `{}`: `{l}` rests on assumptions", n.id)
                );
            }
            if let Err(e) = parts_ok(&parts, &want) {
                refuse!("incompatible", format!("node `{}`: `{l}`: {e}", n.id));
            }
            let goal = parts.goal.clone();
            canon = goal;
            printed = k.checked_goal.clone().unwrap_or_default();
            source = format!("{l} ({})", pp.path);
            label = "kernel accepted (unconditional theorem)".to_string();
            edges = vec![];
        } else {
            let a = n.assembly.as_ref().expect("phase 1");
            let (sha, bytes) = match file_sha(root, a) {
                Ok(x) => x,
                Err((k, m)) => refuse!(&k, m),
            };
            if sha != a.sha256 {
                refuse!(
                    "stale",
                    format!(
                        "node `{}`: {} has SHA-256 {sha}, not the pinned {}",
                        n.id, a.path, a.sha256
                    )
                );
            }
            let src = match String::from_utf8(bytes) {
                Ok(s) => s,
                Err(_) => refuse!("malformed", format!("{}: not UTF-8", a.path)),
            };
            let out = check_preread_outcome(root, &root.join(&a.path), src);
            if out.report.code != ACCEPT {
                let k = kernel_fields(&out, &a.path, &sha);
                refuse!(
                    "kernel",
                    format!(
                        "node `{}`: assembly {} is not accepted ({})",
                        n.id, a.path, k.verdict_line
                    )
                );
            }
            if !out.citations.is_empty() {
                refuse!("graph", format!("node `{}`: assembly {} cites other files; an assembly takes its premises as assumptions only", n.id, a.path));
            }
            let k = kernel_fields(&out, &a.path, &sha);
            let Some(parts) = statement_parts(&out, &k) else {
                refuse!(
                    "kernel",
                    format!("node `{}`: assembly {} has no statement", n.id, a.path)
                );
            };
            if let Err(e) = parts_ok(&parts, &want) {
                refuse!(
                    "incompatible",
                    format!("node `{}`: assembly {}: {e}", n.id, a.path)
                );
            }
            let th = out.theorem.as_ref().expect("accepted");
            let gamma = th.assumptions();
            let declared: BTreeSet<String> = out
                .proof
                .as_ref()
                .map(|p| p.assumptions.iter().map(|(x, _)| x.clone()).collect())
                .unwrap_or_default();
            for d in &declared {
                if !gamma.contains_key(d) {
                    refuse!("unused", format!("node `{}`: assumption `{d}` is declared in {} but not used by the proof", n.id, a.path));
                }
            }
            let rows: BTreeSet<&str> = n.discharge.iter().map(|d| d.assumption.as_str()).collect();
            for g in gamma.keys() {
                if !rows.contains(g.as_str()) {
                    refuse!(
                        "missing",
                        format!(
                            "node `{}`: assumption `{g}` of {} has no discharge row",
                            n.id, a.path
                        )
                    );
                }
            }
            let mut es = vec![];
            for d in &n.discharge {
                let Some(gp) = gamma.get(&d.assumption) else {
                    refuse!(
                        "unused",
                        format!(
                            "node `{}`: the discharge row for `{}` names no assumption of {}",
                            n.id, d.assumption, a.path
                        )
                    );
                };
                let u = by_id[d.by.as_str()];
                pending.push((
                    n.layer,
                    n.id.clone(),
                    d.assumption.clone(),
                    canonical(gp),
                    u.id.clone(),
                ));
                es.push((d.assumption.clone(), u.id.clone(), u.layer));
            }
            canon = parts.goal.clone();
            printed = k.checked_goal.clone().unwrap_or_default();
            source = a.path.clone();
            label = format!(
                "kernel accepted (conditional theorem, {} assumptions); discharged by composition (layer {}): {} of {} assumptions discharged",
                gamma.len(),
                n.layer,
                es.len(),
                gamma.len()
            );
            edges = es;
        }
        let sgoal = &spec_goal[&n.id];
        if *sgoal != canon {
            refuse!(
                "specification",
                format!(
                    "node `{}`: its statement is not, canonically, the approved specification {}",
                    n.id, n.spec.path
                )
            );
        }
        if let Some(o) = owner.insert(canon.clone(), n.id.clone()) {
            refuse!(
                "duplicated",
                format!("nodes `{o}` and `{}` state the same formula", n.id)
            );
        }
        stmt.insert(n.id.clone(), canon.clone());
        c.checked.push(Checked {
            id: n.id.clone(),
            layer: n.layer,
            source,
            statement: canon,
            printed,
            label,
            edges,
        });
    }
    c.checked.sort_by_key(|n| n.layer);
    // Every discharge, in layer order: the assumption is, canonically, the
    // statement of its discharger (each passed above, in this run).
    pending.sort_by_key(|p| p.0);
    for (_, node, a, g, u) in &pending {
        if *g != stmt[u] {
            refuse!(
                "mismatch",
                format!(
                    "node `{node}`: assumption `{a}` is not, canonically, the statement of `{u}`"
                )
            );
        }
    }
    // The named result: its id and its exact, ordered list of statements
    // are registered (a fixed, reviewed file the bundle cannot choose).
    let named = match read_register(root, RESULTS_REGISTER, 3) {
        Ok(r) => r,
        Err((k, m)) => refuse!(&k, m),
    };
    match named.get(&b.id) {
        None => refuse!(
            "specification",
            format!(
                "named result `{}` is not registered in {RESULTS_REGISTER}",
                b.id
            )
        ),
        Some(f) if f[0] != b.result.join(",") => refuse!(
            "specification",
            format!(
                "named result `{}`: the result lists {}, but {RESULTS_REGISTER} registers {}",
                b.id,
                b.result.join(", "),
                f[0].replace(',', ", ")
            )
        ),
        Some(f) => c.note = f[1].clone(),
    }
    c.exit = ACCEPT;
    c
}

/// A specification probe's statement: the file must be accepted, cite
/// nothing, and declare exactly one assumption, which is its goal.
pub fn probe_parts(
    root: &Path,
    path: &str,
    src: String,
    sha: &str,
) -> Result<crate::deps::StatementParts, String> {
    let so = check_preread_outcome(root, &root.join(path), src);
    let sk = kernel_fields(&so, path, sha);
    let Some(sp) = (so.report.code == ACCEPT)
        .then(|| statement_parts(&so, &sk))
        .flatten()
    else {
        return Err(format!(
            "specification {path} is not accepted ({})",
            sk.verdict_line
        ));
    };
    let sth = so.theorem.as_ref().expect("accepted");
    if !so.citations.is_empty()
        || so.proof.as_ref().map(|p| p.assumptions.len()) != Some(1)
        || sth.assumptions().len() != 1
        || sth.assumptions().values().any(|x| canonical(x) != sp.goal)
    {
        return Err(format!("{path} is not a specification probe (exactly one declared assumption, equal to its goal; no citations)"));
    }
    Ok(sp)
}

/// A fixed register: tab-separated lines `id<TAB>field…`, `#` comments;
/// ids unique; `n` fields per line including the id.
fn read_register(
    root: &Path,
    path: &str,
    n: usize,
) -> Result<BTreeMap<String, Vec<String>>, (String, String)> {
    let text = std::fs::read_to_string(root.join(path))
        .map_err(|_| ("missing".to_string(), format!("{path}: file not found")))?;
    let mut m = BTreeMap::new();
    for (i, l) in text.lines().enumerate() {
        if l.is_empty() || l.starts_with('#') {
            continue;
        }
        let f: Vec<&str> = l.split('\t').collect();
        if f.len() != n
            || m.insert(
                f[0].to_string(),
                f[1..].iter().map(|x| x.to_string()).collect::<Vec<_>>(),
            )
            .is_some()
        {
            return Err((
                "malformed".to_string(),
                format!(
                    "{path}: line {}: expected {n} tab-separated fields, ids unique",
                    i + 1
                ),
            ));
        }
    }
    Ok(m)
}

/// Whether a check failed at a citation (at any depth) whose problem
/// contains `what`: a changed file (` has SHA-256 `) or an absent one.
fn citation_problem(out: &crate::CheckOutcome, what: &str) -> bool {
    out.citations.iter().any(|t| {
        t.problem.as_deref().is_some_and(|p| p.contains(what))
            || t.outcome
                .as_deref()
                .is_some_and(|o| citation_problem(o, what))
    })
}

pub fn render_text(c: &Composition) -> String {
    let mut t = String::new();
    let _ = writeln!(t, "RLT compose     {}", c.bundle);
    if !c.id.is_empty() {
        let _ = writeln!(t, "named result    {}", c.id);
    }
    let layers = c.checked.iter().map(|n| n.layer).max().unwrap_or(0);
    let edges: usize = c.checked.iter().map(|n| n.edges.len()).sum();
    for l in 0..=layers {
        let ns: Vec<_> = c.checked.iter().filter(|n| n.layer == l).collect();
        if ns.is_empty() {
            continue;
        }
        let _ = writeln!(t, "layer {l}         {} nodes", ns.len());
        for n in ns {
            let _ = writeln!(t, "  {:<15} {}", n.id, n.source);
            let _ = writeln!(t, "    {:<13} {}", "status", n.label);
            for (a, by, bl) in &n.edges {
                let _ = writeln!(t, "    {:<13} {a} ← {by} (layer {bl})", "discharged");
            }
            let _ = writeln!(
                t,
                "    {:<13} matched (canonical form, bound variables as indices)",
                "specification"
            );
        }
    }
    match &c.refusal {
        None => {
            let _ = writeln!(t, "graph           {} nodes in layers 0–{layers}, {edges} discharged assumptions, acyclic; every proof re-checked in this run", c.checked.len());
            let _ = writeln!(t, "result          {} statements", c.result_nodes().len());
            for (i, n) in c.result_nodes().iter().enumerate() {
                let label = if n.layer == 0 {
                    "kernel accepted".to_string()
                } else {
                    format!("discharged by composition (layer {})", n.layer)
                };
                let _ = writeln!(t, "  {}. {:<12} {label}", i + 1, n.id);
                let _ = writeln!(t, "     {}", n.printed);
            }
            let k = c.result_nodes().iter().filter(|n| n.layer == 0).count();
            let d = c.result_nodes().len() - k;
            let _ = writeln!(t, "remaining       assumptions: none (every assumption of every composed node is discharged)");
            let _ = writeln!(t, "summary         {k} statements kernel accepted, {d} discharged by composition ({layers} composition layers above the components, {edges} edges); a composition of kernel-checked theorems, not one kernel theorem");
            if !c.note.is_empty() {
                let _ = writeln!(t, "outside         {}", c.note);
            }
            let _ = writeln!(t, "verdict         COMPOSED — every assumption discharged by a node of a lower layer that passed in this run");
        }
        Some((class, why)) => {
            let _ = writeln!(t, "verdict         REFUSED ({class}) — {why}");
        }
    }
    let _ = writeln!(t, "exit            {}", c.exit);
    t
}

impl Composition {
    /// The result's statements, in the bundle's order.
    pub fn result_nodes(&self) -> Vec<&Checked> {
        self.result
            .iter()
            .filter_map(|i| self.checked.iter().find(|n| &n.id == i))
            .collect()
    }
}

/// `rlt compose` entry point.
pub fn compose_report(root: &Path, bundle: &str) -> (String, i32) {
    let c = compose(root, bundle);
    (render_text(&c), c.exit)
}

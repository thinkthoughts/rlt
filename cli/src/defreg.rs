//! RLT-DEF-001a: the canonical definition registry (closes D2).
//!
//! `spec/definitions-registry.yaml` lists every definitions file a NAT-FUN
//! proof or goal file may name, pinned here by id, path and SHA-256, and the
//! registry file itself is pinned by SHA-256. A definitions file is
//! *registered* exactly when its path and the SHA-256 of its bytes equal one
//! `functions` entry; only registered definitions give authoritative
//! results. Three identities are kept apart:
//!
//! - file identity: SHA-256 of a definitions file's bytes;
//! - content identity: SHA-256 of one definition's canonical text
//!   (`rlt-function-definition/1`, computed by `crate::read_function_definitions`);
//! - expansion identity: SHA-256 of `rlt-function-expansion/1`, the arity
//!   and the fully unfolded body with parameters renamed p0, p1, … in order
//!   (reported only; it never lets two definitions share a name).
//!
//! Rules (all fail closed):
//! - I1 the registry is a regular file hashing to the pin, with strict keys
//!   (no unknown or duplicate key, no alias, anchor or tag) and the
//!   version-1 `format` and `canonical_text`;
//! - I2 its entries equal `PINNED_DEFINITION_FILES` in id, path, SHA-256 and order;
//! - I3 ids unique, paths unique and plain;
//! - I4 a NAT-FUN header's path and SHA-256 equal one `functions` entry;
//! - I5 the file's own id, convention and domain equal the entry's;
//! - I6 the content identities computed from the file equal the entry's
//!   `defines`, name by name, in file order, and nothing else is defined;
//! - I7 one name, one content identity across the registry; names disjoint
//!   from rules, axiom names and the NAT-DEF names;
//! - I8 the `nat-def` entry equals `PINNED_DEFINITIONS` and lists exactly
//!   the names `spec/definitions.rlt` defines.

use crate::explain::{json_write, Json};
use crate::{sha256_hex, PINNED_DEFINITIONS};
use rlt_kernel::Term;
use rlt_syntax::fdefs::FunctionDefs;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;
use yaml_rust2::Yaml;

/// Where the registry lives.
pub const DEFINITIONS_REGISTRY: &str = "spec/definitions-registry.yaml";

/// RLT-DEF-001a: the registered definitions files (id, path, SHA-256), in
/// registry order. Changing a definitions file, or registering a new one,
/// is a reviewed change to this constant and to the registry pin below.
pub const PINNED_DEFINITION_FILES: &[(&str, &str, &str)] = &[
    (
        "DOUBLE",
        "definitions/double.rlt",
        "8dabb459817f7b225419456080a4e81eb675daf9a830dba529192ff50cfbc2f7",
    ),
    (
        "QUADRUPLE",
        "definitions/quadruple.rlt",
        "7f463b05541ed9643ab98b276bb37de31c2f39054f55502656bd1c20fb703932",
    ),
    (
        "NAT-DEF",
        "spec/definitions.rlt",
        "122e37fc627b136023a1a5ffac0344162ed77ae95312248a9f6a76c2b639ecda",
    ),
];

/// SHA-256 of `spec/definitions-registry.yaml`.
pub const PINNED_DEFINITIONS_REGISTRY_SHA: &str =
    "fea82f7d0ebf52a02866c9b0777257fa7a2100862af40e3223f1abfedb4fbc40";

pub const REGISTRY_FORMAT: &str = "rlt-definitions-registry/1";
pub const CANONICAL_TEXT: &str = "rlt-function-definition/1";
pub const EXPANSION_TEXT: &str = "rlt-function-expansion/1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EntryKind {
    Functions,
    NatDef,
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub id: String,
    pub path: String,
    pub sha256: String,
    pub kind: EntryKind,
    /// `functions` entries only.
    pub convention: String,
    pub domain: String,
    /// `functions`: (name, content SHA-256) in file order.
    pub defines: Vec<(String, String)>,
    /// `nat-def`: the names, in file order.
    pub names: Vec<String>,
}

/// The loaded and checked registry.
pub struct Registry {
    pub sha256: String,
    pub entries: Vec<Entry>,
    /// Each `functions` entry's definitions, loaded from the registered bytes.
    pub loaded: BTreeMap<String, FunctionDefs>,
    /// name → content identity, over all `functions` entries (I7).
    pub canonical: BTreeMap<String, String>,
}

/// How a caller treats definitions that are not registered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Authority {
    /// The default for every path that can give a result authority.
    RegisteredOnly,
    /// `--unregistered-definitions`: unregistered files are checked, and the
    /// result is marked exploratory (exit 4) with no authority.
    ExploratoryAllowed,
}

/// Whether the definitions a check used are registered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Registration {
    Registered(String),
    /// Exploratory only: why the file is not registered.
    Unregistered(String),
}

impl Registration {
    pub fn is_registered(&self) -> bool {
        matches!(self, Registration::Registered(_))
    }
}

fn err(rule: &str, msg: String) -> String {
    format!("definitions registry ({rule}): {msg}")
}

/// Strict scan of the registry's YAML events: aliases, anchors, tags,
/// non-scalar keys and duplicate keys are refused (I1).
fn strict_yaml(src: &str) -> Result<(), String> {
    use yaml_rust2::parser::{Event, MarkedEventReceiver, Parser};
    use yaml_rust2::scanner::Marker;
    enum Frame {
        Map {
            keys: BTreeSet<String>,
            want_key: bool,
        },
        Seq,
    }
    struct Scan {
        err: Option<String>,
        stack: Vec<Frame>,
    }
    impl Scan {
        /// A value or key begins in the current container.
        fn item(&mut self, scalar_key: Option<&str>, line: usize, complex: bool) {
            if let Some(Frame::Map { keys, want_key }) = self.stack.last_mut() {
                if *want_key {
                    match scalar_key {
                        Some(k) if !complex => {
                            if !keys.insert(k.to_string()) {
                                self.err = Some(format!("line {line}: duplicate key `{k}`"));
                            }
                        }
                        _ => {
                            self.err = Some(format!("line {line}: keys must be plain scalars"));
                        }
                    }
                }
                *want_key = !*want_key;
            }
        }
    }
    impl MarkedEventReceiver for Scan {
        fn on_event(&mut self, ev: Event, m: Marker) {
            if self.err.is_some() {
                return;
            }
            let line = m.line();
            match ev {
                Event::Alias(_) => self.err = Some(format!("line {line}: aliases are refused")),
                Event::Scalar(_, _, a, _)
                | Event::SequenceStart(a, _)
                | Event::MappingStart(a, _)
                    if a != 0 =>
                {
                    self.err = Some(format!("line {line}: anchors are refused"))
                }
                Event::Scalar(_, _, _, Some(_))
                | Event::SequenceStart(_, Some(_))
                | Event::MappingStart(_, Some(_)) => {
                    self.err = Some(format!("line {line}: tags are refused"))
                }
                Event::Scalar(v, ..) => self.item(Some(&v), line, false),
                Event::MappingStart(..) => {
                    self.item(None, line, true);
                    self.stack.push(Frame::Map {
                        keys: BTreeSet::new(),
                        want_key: true,
                    });
                }
                Event::SequenceStart(..) => {
                    self.item(None, line, true);
                    self.stack.push(Frame::Seq);
                }
                Event::MappingEnd | Event::SequenceEnd => {
                    self.stack.pop();
                }
                _ => {}
            }
        }
    }
    let mut scan = Scan {
        err: None,
        stack: Vec::new(),
    };
    Parser::new_from_str(src)
        .load(&mut scan, true)
        .map_err(|e| format!("{e}"))?;
    match scan.err {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

fn exact_keys(y: &Yaml, allowed: &[&str], ctx: &str) -> Result<(), String> {
    let Yaml::Hash(h) = y else {
        return Err(format!("{ctx}: expected a mapping"));
    };
    for k in h.keys() {
        match k {
            Yaml::String(k) if allowed.contains(&k.as_str()) => {}
            Yaml::String(k) => return Err(format!("{ctx}: unknown key `{k}`")),
            _ => return Err(format!("{ctx}: keys are strings")),
        }
    }
    for a in allowed {
        if y[*a].is_badvalue() {
            return Err(format!("{ctx}: `{a}` is missing"));
        }
    }
    Ok(())
}

fn string(y: &Yaml, key: &str, ctx: &str) -> Result<String, String> {
    match &y[key] {
        Yaml::String(s) => {
            if s.chars()
                .any(|c| c.is_control() || rlt_syntax::records::invisible(c))
            {
                return Err(format!(
                    "{ctx}: `{key}` holds a control or invisible character"
                ));
            }
            Ok(s.clone())
        }
        _ => Err(format!("{ctx}: `{key}` must be a string")),
    }
}

fn is_hex64(s: &str) -> bool {
    s.len() == 64
        && s.chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
}

fn parse_entries(y: &Yaml) -> Result<Vec<Entry>, String> {
    exact_keys(
        y,
        &["format", "canonical_text", "files"],
        DEFINITIONS_REGISTRY,
    )?;
    let format = string(y, "format", DEFINITIONS_REGISTRY)?;
    if format != REGISTRY_FORMAT {
        return Err(format!(
            "`format` is `{format}`; this binary reads {REGISTRY_FORMAT}"
        ));
    }
    let canon = string(y, "canonical_text", DEFINITIONS_REGISTRY)?;
    if canon != CANONICAL_TEXT {
        return Err(format!(
            "`canonical_text` is `{canon}`; this binary computes content identities as {CANONICAL_TEXT}"
        ));
    }
    let Some(files) = y["files"].as_vec() else {
        return Err("`files` must be a list".into());
    };
    let mut out = Vec::new();
    for (i, f) in files.iter().enumerate() {
        let ctx = format!("files[{i}]");
        let kind = string(f, "kind", &ctx)?;
        let mut e = Entry {
            id: String::new(),
            path: String::new(),
            sha256: String::new(),
            kind: EntryKind::Functions,
            convention: String::new(),
            domain: String::new(),
            defines: Vec::new(),
            names: Vec::new(),
        };
        match kind.as_str() {
            "functions" => {
                exact_keys(
                    f,
                    &[
                        "id",
                        "path",
                        "sha256",
                        "kind",
                        "convention",
                        "domain",
                        "defines",
                    ],
                    &ctx,
                )?;
                e.convention = string(f, "convention", &ctx)?;
                e.domain = string(f, "domain", &ctx)?;
                let Some(ds) = f["defines"].as_vec() else {
                    return Err(format!("{ctx}: `defines` must be a list"));
                };
                for (j, d) in ds.iter().enumerate() {
                    let c = format!("{ctx}.defines[{j}]");
                    exact_keys(d, &["name", "content_sha256"], &c)?;
                    let (n, h) = (string(d, "name", &c)?, string(d, "content_sha256", &c)?);
                    if !is_hex64(&h) {
                        return Err(format!(
                            "{c}: `content_sha256` must be 64 lower-case hex digits"
                        ));
                    }
                    e.defines.push((n, h));
                }
            }
            "nat-def" => {
                exact_keys(f, &["id", "path", "sha256", "kind", "names"], &ctx)?;
                e.kind = EntryKind::NatDef;
                let Some(ns) = f["names"].as_vec() else {
                    return Err(format!("{ctx}: `names` must be a list"));
                };
                for n in ns {
                    match n {
                        Yaml::String(s) => e.names.push(s.clone()),
                        _ => return Err(format!("{ctx}: `names` must list strings")),
                    }
                }
            }
            other => return Err(format!("{ctx}: unknown kind `{other}`")),
        }
        e.id = string(f, "id", &ctx)?;
        e.path = string(f, "path", &ctx)?;
        e.sha256 = string(f, "sha256", &ctx)?;
        if !is_hex64(&e.sha256) {
            return Err(format!("{ctx}: `sha256` must be 64 lower-case hex digits"));
        }
        out.push(e);
    }
    Ok(out)
}

/// The pins a registry is checked against: this binary's constants, or
/// (tests only) others, so each rule after I1 can be exercised on its own.
#[derive(Clone, Copy)]
pub struct Pins<'a> {
    pub registry_sha256: &'a str,
    pub files: &'a [(&'a str, &'a str, &'a str)],
}

/// This binary's pins.
pub const PINS: Pins<'static> = Pins {
    registry_sha256: PINNED_DEFINITIONS_REGISTRY_SHA,
    files: PINNED_DEFINITION_FILES,
};

/// Load and check the whole registry against this binary's pins: I1, I2,
/// I3, and I5–I8 for every registered file (each loaded from bytes that
/// must hash to its entry).
pub fn load_definitions_registry(root: &Path) -> Result<Registry, String> {
    load_definitions_registry_pinned(root, PINS)
}

/// As `load_definitions_registry`, against the given pins. Every
/// authoritative path uses `PINS`; other pins only test the rules.
pub fn load_definitions_registry_pinned(root: &Path, pins: Pins) -> Result<Registry, String> {
    // I1: a regular file (no symlink on the way), hashing to the pin.
    for p in [root.join("spec"), root.join(DEFINITIONS_REGISTRY)] {
        let meta = std::fs::symlink_metadata(&p)
            .map_err(|e| err("I1", format!("{DEFINITIONS_REGISTRY}: {}", e.kind())))?;
        if meta.file_type().is_symlink() {
            return Err(err(
                "I1",
                format!(
                    "{} is a symbolic link; the registry must be a regular file",
                    p.strip_prefix(root).unwrap_or(&p).display()
                ),
            ));
        }
    }
    if !std::fs::symlink_metadata(root.join(DEFINITIONS_REGISTRY)).is_ok_and(|m| m.is_file()) {
        return Err(err(
            "I1",
            format!("{DEFINITIONS_REGISTRY} must be a regular file"),
        ));
    }
    let bytes = std::fs::read(root.join(DEFINITIONS_REGISTRY))
        .map_err(|e| err("I1", format!("{DEFINITIONS_REGISTRY}: {}", e.kind())))?;
    let sha = sha256_hex(&bytes);
    if sha != pins.registry_sha256 {
        return Err(err(
            "I1",
            format!(
                "{DEFINITIONS_REGISTRY} has SHA-256 {sha}, pinned {}; the registry changed without a reviewed update of this binary",
                pins.registry_sha256
            ),
        ));
    }
    let src = String::from_utf8(bytes)
        .map_err(|_| err("I1", format!("{DEFINITIONS_REGISTRY} is not UTF-8")))?;
    strict_yaml(&src).map_err(|e| err("I1", format!("{DEFINITIONS_REGISTRY}: {e}")))?;
    let mut docs = yaml_rust2::YamlLoader::load_from_str(&src)
        .map_err(|e| err("I1", format!("{DEFINITIONS_REGISTRY}: {e}")))?;
    if docs.len() != 1 {
        return Err(err(
            "I1",
            format!("{DEFINITIONS_REGISTRY}: expected one YAML document"),
        ));
    }
    let entries = parse_entries(&docs.remove(0)).map_err(|e| err("I1", e))?;
    // I2: the entries are exactly the pinned list.
    let listed: Vec<(&str, &str, &str)> = entries
        .iter()
        .map(|e| (e.id.as_str(), e.path.as_str(), e.sha256.as_str()))
        .collect();
    if listed.as_slice() != pins.files {
        return Err(err(
            "I2",
            format!(
                "the registry lists {listed:?}; this binary pins {:?}",
                pins.files
            ),
        ));
    }
    // I3: unique ids and paths, each a plain repository-relative path.
    let mut ids = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for e in &entries {
        if !ids.insert(e.id.clone()) {
            return Err(err("I3", format!("id `{}` is listed twice", e.id)));
        }
        if !paths.insert(e.path.clone()) {
            return Err(err("I3", format!("path `{}` is listed twice", e.path)));
        }
        crate::citation_path(root, &e.path).map_err(|x| {
            err(
                "I3",
                format!("{}: {}", e.id, x.replace("cited path", "registered path")),
            )
        })?;
    }
    // I8: the read-only NAT-DEF entry equals the existing pin and names.
    let natdef: Vec<&Entry> = entries
        .iter()
        .filter(|e| e.kind == EntryKind::NatDef)
        .collect();
    let nat_names: Vec<String> = match natdef.as_slice() {
        [e] => {
            if (e.path.as_str(), e.sha256.as_str()) != PINNED_DEFINITIONS {
                return Err(err(
                    "I8",
                    format!(
                        "the nat-def entry names {} {}; PINNED_DEFINITIONS is {} {}",
                        e.path, e.sha256, PINNED_DEFINITIONS.0, PINNED_DEFINITIONS.1
                    ),
                ));
            }
            let defs = crate::load_definitions(root).map_err(|x| err("I8", x))?;
            let found: Vec<String> = defs.list.iter().map(|d| d.name.clone()).collect();
            if found != e.names {
                return Err(err(
                    "I8",
                    format!(
                        "the nat-def entry lists {:?}; {} defines {found:?}",
                        e.names, e.path
                    ),
                ));
            }
            found
        }
        _ => {
            return Err(err(
                "I8",
                format!("expected one nat-def entry, found {}", natdef.len()),
            ))
        }
    };
    // I5–I7 for every functions entry, from its registered bytes.
    let mut loaded = BTreeMap::new();
    let mut canonical: BTreeMap<String, (String, String)> = BTreeMap::new();
    for e in entries.iter().filter(|e| e.kind == EntryKind::Functions) {
        let d = crate::read_function_definitions(
            root,
            &rlt_syntax::proof::DefinitionsRef {
                path: e.path.clone(),
                sha256: e.sha256.clone(),
            },
        )
        .map_err(|x| err("I4", format!("registered file {}: {x}", e.id)))?;
        entry_matches(e, &d)?;
        for (n, h) in &e.defines {
            if nat_names.contains(n) {
                return Err(err("I7", format!("{}: `{n}` is a NAT-DEF name", e.id)));
            }
            match canonical.get(n) {
                Some((h0, id0)) if h0 != h => {
                    return Err(err(
                        "I7",
                        format!(
                            "`{n}` has content sha256 {h0} in {id0} and {h} in {}; one name has one content identity",
                            e.id
                        ),
                    ))
                }
                Some(_) => {}
                None => {
                    canonical.insert(n.clone(), (h.clone(), e.id.clone()));
                }
            }
        }
        loaded.insert(e.id.clone(), d);
    }
    Ok(Registry {
        sha256: sha,
        entries,
        loaded,
        canonical: canonical.into_iter().map(|(n, (h, _))| (n, h)).collect(),
    })
}

/// I5 and I6: the file's own header and computed identities equal the entry.
fn entry_matches(e: &Entry, d: &FunctionDefs) -> Result<(), String> {
    if d.id != e.id || d.convention != e.convention || d.domain != e.domain {
        return Err(err(
            "I5",
            format!(
                "{} declares id {}, convention {}, domain {}; the registry lists {} ({}, {})",
                e.path, d.id, d.convention, d.domain, e.id, e.convention, e.domain
            ),
        ));
    }
    let computed: Vec<(String, String)> = d
        .list
        .iter()
        .map(|x| (x.name.clone(), x.content_sha256.clone()))
        .collect();
    if computed != e.defines {
        return Err(err(
            "I6",
            format!(
                "{} defines {}; the registry lists {}",
                e.path,
                show_defines(&computed),
                show_defines(&e.defines)
            ),
        ));
    }
    Ok(())
}

fn show_defines(v: &[(String, String)]) -> String {
    if v.is_empty() {
        return "nothing".into();
    }
    v.iter()
        .map(|(n, h)| format!("{n}@{h}"))
        .collect::<Vec<_>>()
        .join(", ")
}

impl Registry {
    /// I4 (and I5/I6 again, on the caller's own loaded definitions): whether
    /// these definitions, read from bytes whose SHA-256 the caller checked,
    /// are a registered file.
    pub fn registration_of(&self, d: &FunctionDefs) -> Result<String, String> {
        let entry = self
            .entries
            .iter()
            .find(|e| e.kind == EntryKind::Functions && e.path == d.file);
        let Some(e) = entry else {
            // Recognition by content identity, reported for readers: an
            // unregistered copy of a registered definition is still refused.
            let same: Vec<String> = d
                .list
                .iter()
                .filter(|f| self.canonical.get(&f.name) == Some(&f.content_sha256))
                .map(|f| format!("{}@{}", f.name, f.content_sha256))
                .collect();
            return Err(format!(
                "definitions: {} is not a registered definitions file ({DEFINITIONS_REGISTRY}); authoritative checks read registered definitions only{}",
                d.file,
                if same.is_empty() {
                    String::new()
                } else {
                    format!(
                        " (its {} equal registered content identities; name the registered file to use them)",
                        same.join(", ")
                    )
                }
            ));
        };
        if e.sha256 != d.file_sha256 {
            return Err(format!(
                "definitions: {} is registered at sha256 {}; the file read has sha256 {}, so it is not the registered file",
                d.file, e.sha256, d.file_sha256
            ));
        }
        entry_matches(e, d)?;
        Ok(e.id.clone())
    }
}

/// The expansion identity of one definition: arity and the fully unfolded
/// body with parameters renamed p0, p1, … in order.
pub fn expansion_sha256(f: &rlt_syntax::fdefs::FunctionDef) -> String {
    let sigma: BTreeMap<String, Term> = f
        .params
        .iter()
        .enumerate()
        .map(|(i, p)| (p.clone(), Term::Var(format!("p{i}"))))
        .collect();
    let body = f.body.subst_vars(&sigma);
    sha256_hex(
        format!(
            "{EXPANSION_TEXT}\narity {}\nbody {}\n",
            f.params.len(),
            body.kernel_form()
        )
        .as_bytes(),
    )
}

/// `rlt definitions [ID] [--format text|json]`: the registry, read-only.
pub fn definitions_report(root: &Path, only: Option<&str>, json: bool) -> (String, i32) {
    let reg = match load_definitions_registry(root) {
        Ok(r) => r,
        Err(e) => {
            return if json {
                let mut s = String::new();
                json_write(
                    &Json::Obj(vec![
                        ("format".into(), Json::Str("rlt-definitions/1".into())),
                        ("registry".into(), Json::Str(DEFINITIONS_REGISTRY.into())),
                        ("consistent".into(), Json::Bool(false)),
                        ("problem".into(), Json::Str(e)),
                    ]),
                    0,
                    &mut s,
                );
                s.push('\n');
                (s, crate::MALFORMED)
            } else {
                (
                    format!(
                        "RLT definitions {DEFINITIONS_REGISTRY}\nverdict         INCONSISTENT — {e}\n"
                    ),
                    crate::MALFORMED,
                )
            };
        }
    };
    if let Some(id) = only {
        if !reg.entries.iter().any(|e| e.id == id) {
            let msg = format!("`{id}` is not a registered definitions id");
            return if json {
                let mut s = String::new();
                json_write(
                    &Json::Obj(vec![
                        ("format".into(), Json::Str("rlt-definitions/1".into())),
                        ("registry".into(), Json::Str(DEFINITIONS_REGISTRY.into())),
                        ("consistent".into(), Json::Bool(true)),
                        ("problem".into(), Json::Str(msg)),
                    ]),
                    0,
                    &mut s,
                );
                s.push('\n');
                (s, crate::REJECT)
            } else {
                (
                    format!(
                        "RLT definitions {DEFINITIONS_REGISTRY}\nverdict         UNKNOWN — {msg}\n"
                    ),
                    crate::REJECT,
                )
            };
        }
    }
    // name → (content, [ids]); expansion → names.
    let mut names: BTreeMap<String, (String, Vec<String>)> = BTreeMap::new();
    let mut expansions: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for e in &reg.entries {
        if let Some(d) = reg.loaded.get(&e.id) {
            for f in &d.list {
                let n = names
                    .entry(f.name.clone())
                    .or_insert((f.content_sha256.clone(), Vec::new()));
                n.1.push(e.id.clone());
                expansions
                    .entry(expansion_sha256(f))
                    .or_default()
                    .insert(f.name.clone());
            }
        }
    }
    let shown: Vec<&Entry> = reg
        .entries
        .iter()
        .filter(|e| only.is_none_or(|id| e.id == id))
        .collect();
    let mut t = String::new();
    if json {
        let files: Vec<Json> = shown
            .iter()
            .map(|e| {
                let mut o = vec![
                    ("id".into(), Json::Str(e.id.clone())),
                    ("path".into(), Json::Str(e.path.clone())),
                    ("sha256".into(), Json::Str(e.sha256.clone())),
                ];
                match e.kind {
                    EntryKind::Functions => {
                        o.push(("kind".into(), Json::Str("functions".into())));
                        o.push(("convention".into(), Json::Str(e.convention.clone())));
                        o.push(("domain".into(), Json::Str(e.domain.clone())));
                        let d = &reg.loaded[&e.id];
                        o.push((
                            "defines".into(),
                            Json::Arr(
                                d.list
                                    .iter()
                                    .map(|f| {
                                        Json::Obj(vec![
                                            ("name".into(), Json::Str(f.name.clone())),
                                            ("written".into(), Json::Str(f.written.clone())),
                                            (
                                                "content_sha256".into(),
                                                Json::Str(f.content_sha256.clone()),
                                            ),
                                            (
                                                "expansion_sha256".into(),
                                                Json::Str(expansion_sha256(f)),
                                            ),
                                        ])
                                    })
                                    .collect(),
                            ),
                        ));
                    }
                    EntryKind::NatDef => {
                        o.push(("kind".into(), Json::Str("nat-def".into())));
                        o.push((
                            "names".into(),
                            Json::Arr(e.names.iter().map(|n| Json::Str(n.clone())).collect()),
                        ));
                        o.push(("pinned_as".into(), Json::Str("PINNED_DEFINITIONS".into())));
                    }
                }
                Json::Obj(o)
            })
            .collect();
        let v = Json::Obj(vec![
            ("format".into(), Json::Str("rlt-definitions/1".into())),
            ("registry".into(), Json::Str(DEFINITIONS_REGISTRY.into())),
            ("sha256".into(), Json::Str(reg.sha256.clone())),
            ("registry_format".into(), Json::Str(REGISTRY_FORMAT.into())),
            ("canonical_text".into(), Json::Str(CANONICAL_TEXT.into())),
            ("consistent".into(), Json::Bool(true)),
            ("files".into(), Json::Arr(files)),
            (
                "names".into(),
                Json::Arr(
                    names
                        .iter()
                        .map(|(n, (h, ids))| {
                            Json::Obj(vec![
                                ("name".into(), Json::Str(n.clone())),
                                ("content_sha256".into(), Json::Str(h.clone())),
                                (
                                    "files".into(),
                                    Json::Arr(ids.iter().map(|i| Json::Str(i.clone())).collect()),
                                ),
                            ])
                        })
                        .collect(),
                ),
            ),
            (
                "shared_expansions".into(),
                Json::Arr(
                    expansions
                        .iter()
                        .filter(|(_, ns)| ns.len() > 1)
                        .map(|(h, ns)| {
                            Json::Obj(vec![
                                ("expansion_sha256".into(), Json::Str(h.clone())),
                                (
                                    "names".into(),
                                    Json::Arr(ns.iter().map(|n| Json::Str(n.clone())).collect()),
                                ),
                            ])
                        })
                        .collect(),
                ),
            ),
        ]);
        json_write(&v, 0, &mut t);
        t.push('\n');
        return (t, crate::ACCEPT);
    }
    let _ = writeln!(t, "RLT definitions {DEFINITIONS_REGISTRY}");
    let _ = writeln!(
        t,
        "registry        sha256 {} (pinned in this binary); format {REGISTRY_FORMAT}; content identities {CANONICAL_TEXT}",
        reg.sha256
    );
    for e in &shown {
        match e.kind {
            EntryKind::Functions => {
                let _ = writeln!(
                    t,
                    "file            {} {} sha256 {} (functions, convention {}, domain {})",
                    e.id, e.path, e.sha256, e.convention, e.domain
                );
                for f in &reg.loaded[&e.id].list {
                    let _ = writeln!(t, "  definition    {}", f.written);
                    let _ = writeln!(t, "    content     sha256 {}", f.content_sha256);
                    let _ = writeln!(t, "    expansion   sha256 {}", expansion_sha256(f));
                }
            }
            EntryKind::NatDef => {
                let _ = writeln!(
                    t,
                    "file            {} {} sha256 {} (nat-def, read-only; equal to PINNED_DEFINITIONS)",
                    e.id, e.path, e.sha256
                );
                let _ = writeln!(t, "  names         {}", e.names.join(", "));
            }
        }
    }
    let _ = writeln!(
        t,
        "names           one content identity per name across the registry"
    );
    for (n, (h, ids)) in &names {
        if only.is_none_or(|id| ids.iter().any(|i| i == id)) {
            let _ = writeln!(t, "  {n:<13} {h}  ({})", ids.join(", "));
        }
    }
    let shared: Vec<String> = expansions
        .values()
        .filter(|ns| ns.len() > 1)
        .map(|ns| ns.iter().cloned().collect::<Vec<_>>().join(" = "))
        .collect();
    let _ = writeln!(
        t,
        "expansions      {}",
        if shared.is_empty() {
            "each registered name has its own expansion identity".to_string()
        } else {
            format!(
                "distinct names with one expansion identity: {}",
                shared.join("; ")
            )
        }
    );
    let _ = writeln!(
        t,
        "verdict         CONSISTENT — rules I1–I8 hold; authoritative NAT-FUN checks read only these files at these hashes"
    );
    (t, crate::ACCEPT)
}

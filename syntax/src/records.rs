//! YAML records: source registry, conventions, status vocabulary, reading points.
//! These are data. Loading one never produces a theorem.

use crate::elab::Convention;
use std::collections::BTreeMap;
use std::path::Path;
use yaml_rust2::{Yaml, YamlLoader};

/// Read a record or proof file. Only a regular file is read: a symbolic link,
/// directory, FIFO or device is refused before it is opened, so a record can
/// neither point outside the repository nor block the checker.
pub fn read_regular(path: &Path) -> Result<String, String> {
    let meta =
        std::fs::symlink_metadata(path).map_err(|e| format!("{}: {}", path.display(), e.kind()))?;
    if meta.file_type().is_symlink() {
        return Err(format!(
            "{}: is a symbolic link; records and proofs are regular files",
            path.display()
        ));
    }
    if !meta.file_type().is_file() {
        return Err(format!(
            "{}: is not a regular file; records and proofs are regular files",
            path.display()
        ));
    }
    std::fs::read_to_string(path).map_err(|e| format!("{}: {}", path.display(), e.kind()))
}

pub fn load_yaml(path: &Path) -> Result<Yaml, String> {
    let src = read_regular(path)?;
    let mut docs =
        YamlLoader::load_from_str(&src).map_err(|e| format!("{}: {e}", path.display()))?;
    if docs.len() != 1 {
        return Err(format!("{}: expected one YAML document", path.display()));
    }
    Ok(docs.remove(0))
}

/// An optional string field: absent is fine, any other type is an error.
/// Record strings print on one report line: control characters are refused
/// (line breaks inside source excerpts are allowed; they collapse in units).
/// Invisible and layout-changing characters: Unicode format characters (Cf),
/// line and paragraph separators, and bidirectional controls. Refused in every
/// record string, so two strings that print alike are the same string and no
/// field can break or reorder a report line.
pub fn invisible(c: char) -> bool {
    matches!(c as u32,
        0x00AD | 0x0600..=0x0605 | 0x061C | 0x06DD | 0x070F | 0x0890..=0x0891 | 0x08E2
        | 0x180E | 0x200B..=0x200F | 0x2028..=0x202E | 0x2060..=0x2064 | 0x2066..=0x206F
        | 0xFEFF | 0xFFF9..=0xFFFB | 0x110BD | 0x110CD | 0x13430..=0x1343F
        | 0x1BCA0..=0x1BCA3 | 0x1D173..=0x1D17A | 0xE0001 | 0xE0020..=0xE007F)
}

pub fn clean(v: &str, key: &str, ctx: &str, allow_newline: bool) -> Result<String, String> {
    match v
        .chars()
        .find(|c| (c.is_control() && !(allow_newline && *c == '\n')) || invisible(*c))
    {
        Some(c) => Err(format!(
            "{ctx}: field `{key}` holds control or invisible character U+{:04X}",
            c as u32
        )),
        None => Ok(v.to_string()),
    }
}

/// Record identifiers (reading points, counts, relations, points files and
/// references): lower-case ASCII letters, digits, `.`, `_` and `-`, starting
/// and ending with a letter or digit. Two identifiers that print alike are
/// then the same identifier.
pub fn plain_id(id: &str, what: &str, ctx: &str) -> Result<String, String> {
    let ok_char = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || ".-_".contains(c);
    let edge = |c: Option<char>| c.is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
    if id.chars().all(ok_char) && edge(id.chars().next()) && edge(id.chars().last()) {
        Ok(id.to_string())
    } else {
        Err(format!(
            "{ctx}: {what} {:?}: identifiers use lower-case ASCII letters, digits, `.`, `_` and `-`",
            id.escape_debug().to_string()
        ))
    }
}

fn opt(y: &Yaml, key: &str, ctx: &str) -> Result<Option<String>, String> {
    match &y[key] {
        Yaml::BadValue | Yaml::Null => Ok(None),
        Yaml::String(v) => Ok(Some(clean(v, key, ctx, false)?)),
        _ => Err(format!("{ctx}: field `{key}` must be a string")),
    }
}

fn req(y: &Yaml, key: &str, ctx: &str) -> Result<String, String> {
    match &y[key] {
        Yaml::String(v) => clean(v, key, ctx, false),
        Yaml::BadValue | Yaml::Null => Err(format!("{ctx}: missing string field `{key}`")),
        _ => Err(format!("{ctx}: field `{key}` must be a string")),
    }
}

/// A list field: absent means empty; present with any other type is an error,
/// so a mistyped field cannot make records vanish silently.
fn list<'a>(y: &'a Yaml, key: &str, ctx: &str) -> Result<&'a [Yaml], String> {
    match &y[key] {
        Yaml::BadValue | Yaml::Null => Ok(&[]),
        Yaml::Array(v) => Ok(v.as_slice()),
        _ => Err(format!("{ctx}: field `{key}` must be a list")),
    }
}

fn strings(y: &Yaml, key: &str, ctx: &str) -> Result<Vec<String>, String> {
    list(y, key, ctx)?
        .iter()
        .map(|x| match x.as_str() {
            Some(v) => clean(v, key, ctx, false),
            None => Err(format!("{ctx}: `{key}` must list strings")),
        })
        .collect()
}

fn ids(y: &Yaml, key: &str, ctx: &str) -> Result<Vec<String>, String> {
    strings(y, key, ctx)?
        .iter()
        .map(|v| plain_id(v, key, ctx))
        .collect()
}

/// Inline tags are transparent inside a unit; other tags delimit units.
const INLINE_TAGS: [&str; 8] = ["a", "strong", "em", "b", "i", "code", "sup", "sub"];

fn collapse(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// HTML tag names recognized as markup. Any other `<` is ordinary text, so
/// prose such as `0 < 5` stays intact.
const HTML_TAGS: [&str; 22] = [
    "a", "b", "i", "em", "strong", "code", "sup", "sub", "span", "p", "div", "br", "button", "nav",
    "section", "li", "ul", "ol", "h1", "h2", "h3", "label",
];

/// Decode the HTML entities that occur in RLT's sources.
fn decode_entities(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
        .replace("&middot;", "·")
        .replace("&pi;", "π")
        .replace("&Pi;", "Π")
        .replace("&amp;", "&")
}

/// If `rest` starts with a recognized HTML tag, return (tag name, byte length).
fn html_tag(rest: &str) -> Option<(String, usize)> {
    let body = rest.strip_prefix('<')?;
    let body = body.strip_prefix('/').unwrap_or(body);
    let name: String = body
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric())
        .collect();
    let after = body[name.len()..].chars().next()?;
    if !HTML_TAGS.contains(&name.to_ascii_lowercase().as_str()) || !matches!(after, '>' | ' ' | '/')
    {
        return None;
    }
    // The tag ends at the first '>' outside a quoted attribute value.
    let mut quote: Option<char> = None;
    for (k, c) in rest.char_indices() {
        match (quote, c) {
            (None, '"' | '\'') => quote = Some(c),
            (Some(q), c) if c == q => quote = None,
            (None, '>') => return Some((name.to_ascii_lowercase(), k + 1)),
            _ => {}
        }
    }
    None
}

/// The verbatim units of an excerpt. Recognized inline tags (a, strong, em,
/// b, i, code, sup, sub) are removed with their text kept; other recognized
/// tags delimit units; any other `<` is text. With `whole`, all tags are
/// transparent and the excerpt is one unit. Whitespace runs collapse to one
/// space, then HTML entities are decoded. A meaning must equal a unit exactly.
pub fn units(text: &str, whole: bool) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut i = 0;
    while i < text.len() {
        let rest = &text[i..];
        if let Some((name, len)) = html_tag(rest) {
            if !whole && !INLINE_TAGS.contains(&name.as_str()) {
                let u = decode_entities(&collapse(&cur));
                if !u.is_empty() {
                    out.push(u);
                }
                cur.clear();
            }
            i += len;
        } else {
            let ch = rest.chars().next().unwrap();
            cur.push(ch);
            i += ch.len_utf8();
        }
    }
    let u = decode_entities(&collapse(&cur));
    if !u.is_empty() {
        out.push(u);
    }
    out
}

#[derive(Clone, Debug)]
pub struct Excerpt {
    pub text: String,
    /// The reading point this excerpt documents, if any.
    pub defines: Option<String>,
    pub whole: bool,
}

impl Excerpt {
    pub fn units(&self) -> Vec<String> {
        units(&self.text, self.whole)
    }
}

#[derive(Clone, Debug)]
pub struct Source {
    pub id: String,
    pub status: String,
    pub location: String,
    pub title: String,
    pub url: String,
    pub commit: String,
    pub note: String,
    pub excerpts: Vec<Excerpt>,
}

pub fn load_registry(path: &Path) -> Result<BTreeMap<String, Source>, String> {
    let y = load_yaml(path)?;
    only_keys(&y, &["sources"], "registry")?;
    let mut out = BTreeMap::new();
    for e in list(&y, "sources", "registry")? {
        let id = req(e, "id", "source")?;
        only_keys(
            e,
            &[
                "id", "title", "status", "location", "url", "commit", "note", "excerpts",
            ],
            &format!("source {id}"),
        )?;
        let mut excerpts = Vec::new();
        for x in list(e, "excerpts", &id)? {
            excerpts.push(match x {
                Yaml::String(t) => Excerpt {
                    text: clean(t, "excerpt", &id, true)?,
                    defines: None,
                    whole: false,
                },
                Yaml::Hash(_) => {
                    only_keys(x, &["text", "defines", "split"], &id)?;
                    Excerpt {
                        text: match &x["text"] {
                            // Registry excerpts may span lines of the source file.
                            Yaml::String(v) => clean(v, "text", &id, true)?,
                            _ => return Err(format!("{id}: excerpt `text` must be a string")),
                        },
                        defines: opt(x, "defines", &id)?,
                        whole: match opt(x, "split", &id)?.as_deref() {
                            None | Some("units") => false,
                            Some("whole") => true,
                            Some(o) => {
                                return Err(format!("{id}: split `{o}` must be units or whole"))
                            }
                        },
                    }
                }
                _ => {
                    return Err(format!(
                        "{id}: an excerpt is a string or {{text, defines?, split?}}"
                    ))
                }
            });
        }
        let src = Source {
            status: req(e, "status", &id)?,
            location: opt(e, "location", &id)?.unwrap_or_default(),
            title: opt(e, "title", &id)?.unwrap_or_default(),
            url: opt(e, "url", &id)?.unwrap_or_default(),
            commit: opt(e, "commit", &id)?.unwrap_or_default(),
            note: opt(e, "note", &id)?.unwrap_or_default(),
            excerpts,
            id: id.clone(),
        };
        if out.insert(id.clone(), src).is_some() {
            return Err(format!("source `{id}` registered twice"));
        }
    }
    Ok(out)
}

/// One registered species entry of a `species-table` convention: an exact
/// written formula, the species identifier it is read as, and its element
/// composition, each with the source and locator that support the reading.
/// An entry is an interpretation input; it proves nothing.
#[derive(Clone, Debug)]
pub struct SpeciesEntry {
    pub written: String,
    pub species: String,
    /// (element symbol, count) in the order the entry lists them.
    pub composition: Vec<(String, u64)>,
    pub source: String,
    pub locator: String,
}

impl SpeciesEntry {
    pub fn total(&self) -> u64 {
        self.composition.iter().map(|(_, n)| n).sum()
    }
    /// `C×1 O×2`, in the entry's order.
    pub fn composition_text(&self) -> String {
        self.composition
            .iter()
            .map(|(e, n)| format!("{e}×{n}"))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[derive(Clone, Debug)]
pub struct ConventionRecord {
    pub convention: Convention,
    pub reading: Vec<String>,
    pub sources: Vec<String>,
    pub authority: String,
    pub origin: Option<String>,
    /// Element symbol → element identifier (species-table conventions).
    pub elements: Vec<(String, String)>,
    pub species: Vec<SpeciesEntry>,
    /// Canonical text of the fields that decide interpretation: `id`,
    /// `authority`, `sources`, `origin` and the `executable` block.
    pub interpretation_canon: String,
    /// Canonical text of the `explanation` block.
    pub explanation_canon: String,
}

impl ConventionRecord {
    /// The registered entry for an exact written formula, if any. Lookup is by
    /// the whole string; no composition is ever computed from its letters.
    pub fn species_entry(&self, written: &str) -> Option<&SpeciesEntry> {
        self.species.iter().find(|e| e.written == written)
    }
    pub fn element_name(&self, symbol: &str) -> Option<&str> {
        self.elements
            .iter()
            .find(|(s, _)| s == symbol)
            .map(|(_, n)| n.as_str())
    }
}

/// A canonical, unambiguous text for a parsed YAML value: mappings sorted by
/// key, every string length-prefixed. Two files with the same parsed content
/// give the same text; comments and layout do not count.
pub fn canonical(y: &Yaml) -> Result<String, String> {
    fn go(y: &Yaml, out: &mut String) -> Result<(), String> {
        match y {
            Yaml::String(s) => out.push_str(&format!("s{}:{s}", s.len())),
            Yaml::Integer(i) => out.push_str(&format!("i{i};")),
            Yaml::Real(r) => out.push_str(&format!("r{}:{r}", r.len())),
            Yaml::Boolean(b) => out.push_str(if *b { "b1" } else { "b0" }),
            Yaml::Null => out.push('n'),
            Yaml::Array(v) => {
                out.push('[');
                for x in v {
                    go(x, out)?;
                }
                out.push(']');
            }
            Yaml::Hash(h) => {
                let mut items = Vec::new();
                for (k, v) in h {
                    let mut ks = String::new();
                    go(k, &mut ks)?;
                    let mut vs = String::new();
                    go(v, &mut vs)?;
                    items.push((ks, vs));
                }
                items.sort();
                out.push('{');
                for (k, v) in items {
                    out.push_str(&k);
                    out.push('=');
                    out.push_str(&v);
                }
                out.push('}');
            }
            _ => return Err("a value YAML could not resolve".to_string()),
        }
        Ok(())
    }
    let mut out = String::new();
    go(y, &mut out)?;
    Ok(out)
}

pub const ELABORATORS: [&str; 4] = ["nat-dec", "symbol", "none", "species-table"];

fn string_map(y: &Yaml, key: &str, ctx: &str) -> Result<Vec<(String, String)>, String> {
    match &y[key] {
        Yaml::BadValue | Yaml::Null => Ok(Vec::new()),
        Yaml::Hash(h) => h
            .iter()
            .map(|(k, v)| match (k.as_str(), v.as_str()) {
                (Some(k), Some(v)) => Ok((clean(k, key, ctx, false)?, clean(v, key, ctx, false)?)),
                _ => Err(format!("{ctx}: `{key}` maps strings to strings")),
            })
            .collect(),
        _ => Err(format!("{ctx}: field `{key}` must be a mapping")),
    }
}

pub fn load_convention(path: &Path) -> Result<ConventionRecord, String> {
    let y = load_yaml(path)?;
    let id = req(&y, "id", &path.display().to_string())?;
    if id.is_empty()
        || !id
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(format!(
            "convention id {id:?}: use upper-case letters, digits and `-`"
        ));
    }
    let ctx = format!("convention {id}");
    only_keys(
        &y,
        &[
            "id",
            "authority",
            "sources",
            "origin",
            "executable",
            "explanation",
        ],
        &ctx,
    )?;
    let exe = &y["executable"];
    only_keys(exe, &["elaborator", "symbols", "elements", "species"], &ctx)?;
    let expl = &y["explanation"];
    only_keys(
        expl,
        &["title", "reading", "limits", "conventions", "unresolved"],
        &ctx,
    )?;
    let elaborator = req(exe, "elaborator", &ctx)?;
    if !ELABORATORS.contains(&elaborator.as_str()) {
        return Err(format!(
            "{ctx}: elaborator `{elaborator}` is outside {}",
            ELABORATORS.join(", ")
        ));
    }
    let symbols: BTreeMap<String, String> = string_map(exe, "symbols", &ctx)?.into_iter().collect();
    if !symbols.is_empty() && elaborator != "symbol" {
        return Err(format!("{ctx}: `symbols` belong to the symbol elaborator"));
    }
    let elements = string_map(exe, "elements", &ctx)?;
    let mut species = Vec::new();
    for e in list(exe, "species", &ctx)? {
        only_keys(
            e,
            &["written", "species", "composition", "source", "locator"],
            &ctx,
        )?;
        let written = req(e, "written", &ctx)?;
        let sc = format!("{ctx}: species entry {written:?}");
        let mut composition = Vec::new();
        match &e["composition"] {
            Yaml::Hash(h) if !h.is_empty() => {
                for (k, v) in h {
                    let (Some(sym), Some(n)) = (k.as_str(), v.as_i64()) else {
                        return Err(format!("{sc}: composition maps element symbols to counts"));
                    };
                    if !(1..=1000).contains(&n) {
                        return Err(format!("{sc}: count {n} for {sym} is outside 1..1000"));
                    }
                    if !elements.iter().any(|(s, _)| s == sym) {
                        return Err(format!(
                            "{sc}: element {sym} is outside this convention's element table"
                        ));
                    }
                    composition.push((sym.to_string(), n as u64));
                }
            }
            _ => return Err(format!("{sc}: `composition` must be a non-empty mapping")),
        }
        species.push(SpeciesEntry {
            species: plain_id(&req(e, "species", &sc)?, "species id", &sc)?,
            source: req(e, "source", &sc)?,
            locator: req(e, "locator", &sc)?,
            written,
            composition,
        });
    }
    if (!elements.is_empty() || !species.is_empty()) != (elaborator == "species-table") {
        return Err(format!(
            "{ctx}: `elements` and `species` belong to the species-table elaborator, which needs both"
        ));
    }
    for (sym, name) in &elements {
        let mut cs = sym.chars();
        let ok = cs.next().is_some_and(|c| c.is_ascii_uppercase())
            && cs.all(|c| c.is_ascii_lowercase())
            && sym.len() <= 3;
        if !ok {
            return Err(format!(
                "{ctx}: element symbol {sym:?}: one capital and up to two small ASCII letters"
            ));
        }
        plain_id(name, "element id", &ctx)?;
    }
    for (i, a) in species.iter().enumerate() {
        for b in &species[..i] {
            if a.written == b.written || a.species == b.species {
                return Err(format!(
                    "{ctx}: species entries {:?} and {:?} share a written form or species id",
                    a.written, b.written
                ));
            }
        }
    }
    // Digests: interpretation fields and explanation, as canonical text.
    let mut interp = Yaml::Hash(Default::default());
    if let (Yaml::Hash(src), Yaml::Hash(dst)) = (&y, &mut interp) {
        for (k, v) in src {
            if k.as_str() != Some("explanation") {
                dst.insert(k.clone(), v.clone());
            }
        }
    }
    let interpretation_canon = canonical(&interp).map_err(|e| format!("{ctx}: {e}"))?;
    let explanation_canon = canonical(expl).map_err(|e| format!("{ctx}: {e}"))?;
    Ok(ConventionRecord {
        convention: Convention {
            id: id.clone(),
            elaborator,
            symbols,
        },
        reading: strings(expl, "reading", &ctx)?,
        sources: strings(&y, "sources", &ctx)?,
        authority: req(&y, "authority", &ctx)?,
        origin: opt(&y, "origin", &ctx)?,
        elements,
        species,
        interpretation_canon,
        explanation_canon,
    })
}

/// All conventions in a directory, by id.
pub fn load_conventions(dir: &Path) -> Result<BTreeMap<String, ConventionRecord>, String> {
    let mut paths: Vec<_> = std::fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "yaml"))
        .collect();
    paths.sort();
    let mut out = BTreeMap::new();
    for p in paths {
        let c = load_convention(&p)?;
        if out.insert(c.convention.id.clone(), c).is_some() {
            return Err(format!("{}: duplicate convention id", p.display()));
        }
    }
    Ok(out)
}

#[derive(Clone, Debug)]
pub struct StatusVocabulary {
    pub correspondence: Vec<String>,
    pub lift: Vec<String>,
    pub allowed_support: Vec<String>,
    pub without_evidence: Vec<String>,
    pub referent_only: Vec<String>,
}

pub fn load_status(path: &Path) -> Result<StatusVocabulary, String> {
    let y = load_yaml(path)?;
    let ctx = "status.yaml";
    only_keys(
        &y,
        &[
            "correspondence_status",
            "lift_status",
            "allowed_support",
            "status_without_evidence",
            "referent_only_status",
        ],
        ctx,
    )?;
    Ok(StatusVocabulary {
        correspondence: strings(&y, "correspondence_status", ctx)?,
        lift: strings(&y, "lift_status", ctx)?,
        allowed_support: strings(&y, "allowed_support", ctx)?,
        without_evidence: strings(&y, "status_without_evidence", ctx)?,
        referent_only: strings(&y, "referent_only_status", ctx)?,
    })
}

#[derive(Clone, Debug)]
pub struct Meaning {
    pub text: String,
    pub source: String,
}

#[derive(Clone, Debug)]
pub struct Relationship {
    pub id: String,
    pub written: String,
    pub convention: String,
    pub proof: String,
    /// `unconditional` or `conditional`: what the record claims about Γ.
    pub mode: String,
    /// Written forms of the assumptions Γ the result is expected to rest on.
    pub assumptions: Vec<String>,
}

/// One submitted evidence item. It names the exact claim and referents it is
/// offered for, where in the cited source the evidence sits, and the criteria
/// a reviewer applies. RLT checks that these are present and consistent with
/// the record; whether the evidence meets the criteria is a review judgment
/// RLT leaves open, so the claim stays unestablished.
#[derive(Clone, Debug, Default)]
pub struct EvidenceItem {
    pub etype: String,
    pub reference: String,
    pub locator: Option<String>,
    pub claim: Option<String>,
    pub referents: Vec<String>,
    pub criteria: Option<String>,
}

pub fn evidence(e: &Yaml, ctx: &str) -> Result<Vec<EvidenceItem>, String> {
    list(e, "evidence", ctx)?
        .iter()
        .map(|v| {
            only_keys(
                v,
                &["type", "ref", "locator", "claim", "referents", "criteria"],
                ctx,
            )?;
            Ok(EvidenceItem {
                etype: req(v, "type", ctx)?,
                reference: req(v, "ref", ctx)?,
                locator: opt(v, "locator", ctx)?,
                claim: opt(v, "claim", ctx)?,
                referents: ids(v, "referents", ctx)?,
                criteria: opt(v, "criteria", ctx)?,
            })
        })
        .collect()
}

#[derive(Clone, Debug)]
pub struct Correspondence {
    pub id: String,
    pub kind: String,
    pub description: String,
    pub status: String,
    pub evidence: Vec<EvidenceItem>,
    pub source: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Lift {
    pub id: String,
    pub to: String,
    pub via: String,
    pub status: String,
}

#[derive(Clone, Debug)]
pub struct ReadingPoint {
    pub id: String,
    pub meanings: Vec<Meaning>,
    pub examples: Vec<Meaning>,
    pub relationships: Vec<Relationship>,
    pub correspondence: Vec<Correspondence>,
    pub lifts: Vec<Lift>,
}

pub fn load_readingpoint(path: &Path) -> Result<ReadingPoint, String> {
    let y = load_yaml(path)?;
    let id = req(&y, "id", &path.display().to_string())?;
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    if id != stem {
        return Err(format!(
            "readingpoints/{stem:?}: record id {id:?} differs from its file name"
        ));
    }
    let meaning = |e: &Yaml, key: &str| -> Result<Meaning, String> {
        Ok(Meaning {
            text: req(e, key, &id)?,
            source: req(e, "source", &id)?,
        })
    };
    Ok(ReadingPoint {
        meanings: list(&y, "meanings", &id)?
            .iter()
            .map(|e| meaning(e, "text"))
            .collect::<Result<_, _>>()?,
        examples: list(&y, "examples", &id)?
            .iter()
            .map(|e| meaning(e, "written"))
            .collect::<Result<_, _>>()?,
        relationships: list(&y, "relationships", &id)?
            .iter()
            .map(|e| {
                Ok(Relationship {
                    id: req(e, "id", &id)?,
                    written: req(e, "written", &id)?,
                    convention: req(e, "convention", &id)?,
                    proof: req(e, "proof", &id)?,
                    mode: req(e, "mode", &id)?,
                    assumptions: strings(e, "assumptions", &id)?,
                })
            })
            .collect::<Result<_, String>>()?,
        correspondence: list(&y, "correspondence", &id)?
            .iter()
            .map(|e| {
                Ok(Correspondence {
                    id: req(e, "id", &id)?,
                    kind: req(e, "kind", &id)?,
                    description: req(e, "description", &id)?,
                    status: req(e, "status", &id)?,
                    evidence: evidence(e, &id)?,
                    source: opt(e, "source", &id)?,
                })
            })
            .collect::<Result<_, String>>()?,
        lifts: list(&y, "lifts", &id)?
            .iter()
            .map(|e| {
                Ok(Lift {
                    id: req(e, "id", &id)?,
                    to: req(e, "to", &id)?,
                    via: req(e, "via", &id)?,
                    status: req(e, "status", &id)?,
                })
            })
            .collect::<Result<_, String>>()?,
        id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_tags_delimit_units_inline_tags_do_not() {
        let x = r#"<span class="name">Investigate</span><span class="desc">Zero differs from five, <strong>and</strong> five is less than 29.</span>"#;
        assert_eq!(
            units(x, false),
            vec![
                "Investigate",
                "Zero differs from five, and five is less than 29."
            ]
        );
        let md = r#"For instance (<a href="https://c5.now/">nonfiction</a>): A 🫁 lungs emoji represents human lungs."#;
        assert_eq!(
            units(md, false),
            vec!["For instance (nonfiction): A 🫁 lungs emoji represents human lungs."]
        );
        let closing =
            r#"<span class="red">#Practice</span> the <span class="yellow">#obvious</span>"#;
        assert_eq!(units(closing, true), vec!["#Practice the #obvious"]);
        assert_eq!(units("a\n  b", false), vec!["a b"]);
        // Prose comparisons stay intact; entities decode after splitting.
        assert_eq!(
            units("If 0 < 5 then 5 > 3 holds.", false),
            vec!["If 0 < 5 then 5 > 3 holds."]
        );
        assert_eq!(
            units(r#"<a href="x>y">link</a> text"#, false),
            vec!["link text"]
        );
        assert_eq!(
            units("If 0 &lt; 5 then 5 &gt; 3 holds.", false),
            vec!["If 0 < 5 then 5 > 3 holds."]
        );
        assert_eq!(
            units(
                r#"<span class="equation">(0 ≠ 5) ∧ (5 &lt; 29)</span>"#,
                false
            ),
            vec!["(0 ≠ 5) ∧ (5 < 29)"]
        );
    }

    #[test]
    fn mistyped_list_fields_are_errors() {
        let y = YamlLoader::load_from_str("correspondence: \"see elsewhere\"")
            .unwrap()
            .remove(0);
        assert!(list(&y, "correspondence", "t").is_err());
        assert!(list(&y, "absent", "t").unwrap().is_empty());
    }
}

// ---- RLT-002: specification records (points/*.yaml) ----

/// One specified thing in a context.
#[derive(Clone, Debug)]
pub struct Point {
    pub id: String,
    pub kind: String,
    pub class: Option<String>,
    pub description: String,
    /// `particular` (this identified reading point) or `general` (a stated class).
    pub scope: String,
    pub physical_status: String,
    pub evidence: Vec<EvidenceItem>,
    /// (element symbol, convention): an atom record's registered element.
    pub element: Option<(String, String)>,
    /// (written formula, convention): a molecule record's registered species.
    pub species: Option<(String, String)>,
}

/// Which atom records this file counts as one molecule record's composition.
/// A specification record: RLT checks it against the registered species
/// entry; it states no observation, bond or specimen identity.
#[derive(Clone, Debug)]
pub struct Composition {
    pub id: String,
    pub molecule: String,
    pub atoms: Vec<String>,
}

/// An equation checked by the kernel, associated by its author with groups of
/// reading points: one group per numeral summand of the left side, in order.
/// The association is a specification record; the kernel checks only the equation.
#[derive(Clone, Debug)]
pub struct Count {
    pub claim: Relationship,
    pub groups: Vec<Vec<String>>,
    /// The scope the record states for its conclusion, if any.
    pub scope: Option<String>,
}

/// A typed relation among reading points, with its own status and evidence.
#[derive(Clone, Debug)]
pub struct RelationClaim {
    pub id: String,
    pub rtype: String,
    pub between: Vec<String>,
    pub status: String,
    pub evidence: Vec<EvidenceItem>,
}

#[derive(Clone, Debug)]
pub struct PointsFile {
    pub id: String,
    pub context: String,
    /// A verbatim excerpt of a registered source that states this context.
    pub basis: Meaning,
    /// Named reading points (readingpoints/*.yaml) referenced by id only.
    pub references: Vec<String>,
    pub points: Vec<Point>,
    pub counts: Vec<Count>,
    pub compositions: Vec<Composition>,
    pub relations: Vec<RelationClaim>,
}

fn only_keys(y: &Yaml, allowed: &[&str], ctx: &str) -> Result<(), String> {
    if let Some(h) = y.as_hash() {
        for k in h.keys() {
            let k = k.as_str().unwrap_or("?");
            if !allowed.contains(&k) {
                return Err(format!(
                    "{ctx}: field `{k}` is outside this record type (allowed: {})",
                    allowed.join(", ")
                ));
            }
        }
        Ok(())
    } else {
        Err(format!("{ctx}: expected a mapping"))
    }
}

pub fn load_points(path: &Path) -> Result<PointsFile, String> {
    let y = load_yaml(path)?;
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let ctx = format!("points/{stem}.yaml");
    only_keys(
        &y,
        &[
            "id",
            "context",
            "basis",
            "references",
            "points",
            "compositions",
            "counts",
            "relations",
        ],
        &ctx,
    )?;
    let id = plain_id(&req(&y, "id", &ctx)?, "record id", &ctx)?;
    if id != stem {
        return Err(format!(
            "{ctx}: record id {id:?} differs from its file name"
        ));
    }
    // Every context states its basis: a verbatim unit of a registered source.
    let basis = match &y["basis"] {
        Yaml::BadValue | Yaml::Null => {
            return Err(format!(
            "{ctx}: missing `basis` (a verbatim unit of a registered source stating this context)"
        ))
        }
        b => {
            only_keys(b, &["text", "source"], &ctx)?;
            Meaning {
                text: req(b, "text", &ctx)?,
                source: req(b, "source", &ctx)?,
            }
        }
    };
    let mut points = Vec::new();
    for p in list(&y, "points", &ctx)? {
        only_keys(
            p,
            &[
                "id",
                "kind",
                "class",
                "description",
                "scope",
                "physical",
                "element",
                "species",
            ],
            &ctx,
        )?;
        let pid = plain_id(&req(p, "id", &ctx)?, "point id", &ctx)?;
        let pc = format!("{ctx}: point {pid}");
        let phys = &p["physical"];
        only_keys(phys, &["status", "evidence"], &pc)?;
        points.push(Point {
            kind: req(p, "kind", &pc)?,
            class: opt(p, "class", &pc)?,
            description: req(p, "description", &pc)?,
            scope: req(p, "scope", &pc)?,
            physical_status: req(phys, "status", &pc)?,
            evidence: evidence(phys, &pc)?,
            element: match &p["element"] {
                Yaml::BadValue | Yaml::Null => None,
                e => {
                    only_keys(e, &["symbol", "convention"], &pc)?;
                    Some((req(e, "symbol", &pc)?, req(e, "convention", &pc)?))
                }
            },
            species: match &p["species"] {
                Yaml::BadValue | Yaml::Null => None,
                e => {
                    only_keys(e, &["written", "convention"], &pc)?;
                    Some((req(e, "written", &pc)?, req(e, "convention", &pc)?))
                }
            },
            id: pid,
        });
    }
    let mut counts = Vec::new();
    for c in list(&y, "counts", &ctx)? {
        only_keys(
            c,
            &[
                "id",
                "written",
                "convention",
                "proof",
                "mode",
                "assumptions",
                "groups",
                "scope",
            ],
            &ctx,
        )?;
        let cid = plain_id(&req(c, "id", &ctx)?, "count id", &ctx)?;
        let cc = format!("{ctx}: count {cid}");
        let groups = list(c, "groups", &cc)?
            .iter()
            .map(|g| match g {
                Yaml::Array(v) => v
                    .iter()
                    .map(|m| match m.as_str() {
                        Some(v) => plain_id(v, "group member", &cc),
                        None => Err(format!("{cc}: group members are point ids")),
                    })
                    .collect::<Result<Vec<_>, _>>(),
                _ => Err(format!("{cc}: each group is a list of point ids")),
            })
            .collect::<Result<_, _>>()?;
        counts.push(Count {
            claim: Relationship {
                written: req(c, "written", &cc)?,
                convention: req(c, "convention", &cc)?,
                proof: req(c, "proof", &cc)?,
                mode: req(c, "mode", &cc)?,
                assumptions: strings(c, "assumptions", &cc)?,
                id: cid,
            },
            groups,
            scope: opt(c, "scope", &cc)?,
        });
    }
    let mut compositions = Vec::new();
    for c in list(&y, "compositions", &ctx)? {
        only_keys(c, &["id", "molecule", "atoms"], &ctx)?;
        let cid = plain_id(&req(c, "id", &ctx)?, "composition id", &ctx)?;
        let cc = format!("{ctx}: composition {cid}");
        compositions.push(Composition {
            molecule: plain_id(&req(c, "molecule", &cc)?, "molecule", &cc)?,
            atoms: ids(c, "atoms", &cc)?,
            id: cid,
        });
    }
    let mut relations = Vec::new();
    for r in list(&y, "relations", &ctx)? {
        only_keys(r, &["id", "type", "between", "status", "evidence"], &ctx)?;
        let rid = plain_id(&req(r, "id", &ctx)?, "relation id", &ctx)?;
        let rc = format!("{ctx}: relation {rid}");
        relations.push(RelationClaim {
            rtype: req(r, "type", &rc)?,
            between: ids(r, "between", &rc)?,
            status: req(r, "status", &rc)?,
            evidence: evidence(r, &rc)?,
            id: rid,
        });
    }
    Ok(PointsFile {
        context: req(&y, "context", &ctx)?,
        references: ids(&y, "references", &ctx)?,
        basis,
        points,
        counts,
        compositions,
        relations,
        id,
    })
}

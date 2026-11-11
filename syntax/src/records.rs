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

/// A plain YAML scalar that resolves to an integer must be written in decimal:
/// `0`, or a nonzero digit followed by digits, optionally preceded by `-`.
/// Hexadecimal (`0x07`), octal (`0o7`), a leading `+` and leading zeros
/// (`07`) are refused, so every record numeral reads as written.
pub fn decimal_integer_spelling(v: &str) -> bool {
    let d = v.strip_prefix('-').unwrap_or(v);
    !d.is_empty()
        && d.bytes().all(|b| b.is_ascii_digit())
        && (d == "0" || !d.starts_with('0'))
        && v != "-0"
}

/// Input-shape checks run before a record is built: every record is a plain
/// tree. Anchors and aliases are refused (an alias can expand a short file
/// exponentially), explicit tags are refused (a tag can retype a scalar), and
/// integer scalars must use decimal spelling.
fn check_yaml_events(src: &str, ctx: &str) -> Result<(), String> {
    use yaml_rust2::parser::{Event, MarkedEventReceiver, Parser};
    use yaml_rust2::scanner::{Marker, TScalarStyle};
    struct Scan {
        err: Option<String>,
    }
    impl MarkedEventReceiver for Scan {
        fn on_event(&mut self, ev: Event, m: Marker) {
            if self.err.is_some() {
                return;
            }
            let at = format!("line {}", m.line());
            self.err = match ev {
                Event::Alias(_) => Some(format!(
                    "{at}: YAML aliases are refused; records are written out in full"
                )),
                Event::Scalar(_, _, a, _) | Event::SequenceStart(a, _) | Event::MappingStart(a, _)
                    if a != 0 =>
                {
                    Some(format!("{at}: YAML anchors are refused; records are written out in full"))
                }
                Event::Scalar(_, _, _, Some(_))
                | Event::SequenceStart(_, Some(_))
                | Event::MappingStart(_, Some(_)) => {
                    Some(format!("{at}: YAML tags are refused; a value's type follows from how it is written"))
                }
                Event::Scalar(v, TScalarStyle::Plain, _, None)
                    if matches!(Yaml::from_str(&v), Yaml::Integer(_))
                        && !decimal_integer_spelling(&v) =>
                {
                    Some(format!(
                        "{at}: numeral `{v}` is not decimal; write numerals as 0 or digits without a leading zero, sign `+`, or 0x/0o prefix"
                    ))
                }
                _ => None,
            };
        }
    }
    let mut scan = Scan { err: None };
    Parser::new_from_str(src)
        .load(&mut scan, true)
        .map_err(|e| format!("{ctx}: {e}"))?;
    match scan.err {
        Some(e) => Err(format!("{ctx}: {e}")),
        None => Ok(()),
    }
}

pub fn load_yaml(path: &Path) -> Result<Yaml, String> {
    let src = read_regular(path)?;
    check_yaml_events(&src, &path.display().to_string())?;
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
        0x00AD | 0x034F | 0x115F..=0x1160 | 0x17B4..=0x17B5 | 0x180B..=0x180F | 0x2800
        | 0x3164 | 0xFE00..=0xFE0F | 0xFFA0 | 0xE0100..=0xE01EF | 0x0600..=0x0605 | 0x061C | 0x06DD | 0x070F | 0x0890..=0x0891 | 0x08E2
        | 0x200B..=0x200F | 0x2028..=0x202E | 0x2060..=0x2064 | 0x2066..=0x206F
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

/// The executable part of a `residue-table` convention (RLT-003 M2): a
/// modulus with its factorization theorem, its primes, the registered
/// representative set (with the exact source quotation it comes from), and
/// the one classification rule RLT applies. An interpretation input; it
/// proves nothing.
#[derive(Clone, Debug)]
pub struct ResidueTable {
    pub modulus: u64,
    pub factorization: String,
    pub primes: Vec<u64>,
    pub representatives: Vec<u64>,
    pub quote: Meaning,
    pub rule: String,
}

/// The only classification rule M2 registers.
pub const RESIDUE_RULE: &str = "nonzero-remainders";

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
    pub residue: Option<ResidueTable>,
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

pub const ELABORATORS: [&str; 5] = [
    "nat-dec",
    "symbol",
    "none",
    "species-table",
    "residue-table",
];

fn naturals(y: &Yaml, key: &str, ctx: &str) -> Result<Vec<u64>, String> {
    list(y, key, ctx)?
        .iter()
        .map(|v| match v.as_i64() {
            Some(n) if (0..=1000).contains(&n) => Ok(n as u64),
            _ => Err(format!("{ctx}: `{key}` lists numerals 0..1000")),
        })
        .collect()
}

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
    only_keys(
        exe,
        &[
            "elaborator",
            "symbols",
            "operators",
            "connectives",
            "definitions",
            "elements",
            "species",
            "modulus",
            "factorization",
            "primes",
            "representatives",
            "representatives_quote",
            "rule",
        ],
        &ctx,
    )?;
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
    let listed_ops = strings(exe, "operators", &ctx)?;
    if !listed_ops.is_empty() && elaborator != "nat-dec" {
        return Err(format!(
            "{ctx}: `operators` belong to the nat-dec elaborator"
        ));
    }
    for o in &listed_ops {
        if !["+", "×"].contains(&o.as_str()) {
            return Err(format!("{ctx}: operator {o:?} is outside +, ×"));
        }
    }
    let connectives = strings(exe, "connectives", &ctx)?;
    if !connectives.is_empty() && elaborator != "nat-dec" {
        return Err(format!(
            "{ctx}: `connectives` belong to the nat-dec elaborator"
        ));
    }
    for c in &connectives {
        if !crate::elab::CONNECTIVES.contains(&c.as_str()) {
            return Err(format!(
                "{ctx}: connective {c:?} is outside {}",
                crate::elab::CONNECTIVES.join(", ")
            ));
        }
    }
    // RLT-003 M3b: definition notations, read only with connectives.
    let definitions = strings(exe, "definitions", &ctx)?;
    if !definitions.is_empty() && (elaborator != "nat-dec" || connectives.is_empty()) {
        return Err(format!(
            "{ctx}: `definitions` belong to a nat-dec elaborator that reads connectives"
        ));
    }
    let operators = if listed_ops.is_empty() && elaborator == "nat-dec" {
        vec!["+".to_string()]
    } else {
        listed_ops
    };
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
        composition.sort();
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
    // Residue tables (M2): every field required, and only for residue-table.
    let residue_keys = [
        "modulus",
        "factorization",
        "primes",
        "representatives",
        "representatives_quote",
        "rule",
    ];
    let has_residue_key = residue_keys
        .iter()
        .any(|k| !matches!(exe[*k], Yaml::BadValue));
    let residue = if elaborator == "residue-table" {
        let modulus = match exe["modulus"].as_i64() {
            Some(n) if (2..=1000).contains(&n) => n as u64,
            _ => return Err(format!("{ctx}: `modulus` must be a numeral 2..1000")),
        };
        let q = &exe["representatives_quote"];
        only_keys(q, &["text", "source"], &ctx)?;
        let rule = req(exe, "rule", &ctx)?;
        if rule != RESIDUE_RULE {
            return Err(format!(
                "{ctx}: rule `{rule}` is outside the registered rule {RESIDUE_RULE}"
            ));
        }
        Some(ResidueTable {
            modulus,
            factorization: req(exe, "factorization", &ctx)?,
            primes: naturals(exe, "primes", &ctx)?,
            representatives: naturals(exe, "representatives", &ctx)?,
            quote: Meaning {
                text: req(q, "text", &ctx)?,
                source: req(q, "source", &ctx)?,
            },
            rule,
        })
    } else if has_residue_key {
        return Err(format!(
            "{ctx}: residue fields belong to the residue-table elaborator"
        ));
    } else {
        None
    };
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
            operators,
            connectives,
            definitions,
            defs: None,
        },
        reading: strings(expl, "reading", &ctx)?,
        sources: strings(&y, "sources", &ctx)?,
        authority: req(&y, "authority", &ctx)?,
        origin: opt(&y, "origin", &ctx)?,
        elements,
        species,
        residue,
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

// ---------------------------------------------------------------------------
// RLT-004b: statement records (`statements/<ID>.yaml`). A statement record
// pins a proof, its specification and its checked statement, and carries
// attributed correspondence assessments. It is data: loading one never
// produces or changes a verdict.

/// The format tag every statement record carries.
pub const STATEMENT_FORMAT: &str = "rlt-statement/1";

/// Semantic-correspondence statuses a record may state.
pub const SEMANTIC_STATUSES: [&str; 4] = ["unassessed", "matches", "partial", "differs"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatementSource {
    pub registry: String,
    pub commit: Option<String>,
    pub excerpts: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelPins {
    pub proof: String,
    pub sha256: String,
    pub convention: String,
    pub definitions_sha256: Option<String>,
    pub axiom_set: String,
    pub axiom_set_sha256: String,
    pub checked_goal: String,
    /// `(name, formula)` pairs, in the order the report prints Γ.
    pub assumptions: Vec<(String, String)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeanRef {
    pub file: String,
    pub theorem: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticAssessment {
    pub status: String,
    pub assessed_by: Option<String>,
    pub date: Option<String>,
    /// `(path, sha256)` of the document the assessment rests on.
    pub basis: Option<(String, String)>,
    pub assessed_goal: Option<String>,
    pub matches: Vec<String>,
    pub differs: Vec<String>,
    pub lean: Vec<LeanRef>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhysicalScope {
    /// `not-applicable` or `relevant`.
    pub applicability: String,
    pub reason: Option<String>,
    pub reading_point: Option<String>,
    pub correspondence: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatementRecord {
    pub id: String,
    pub subject: String,
    /// `(reading point, role)` pairs.
    pub identifiers: Vec<(String, String)>,
    pub source: StatementSource,
    pub kernel: KernelPins,
    pub semantic: SemanticAssessment,
    pub physical: PhysicalScope,
    pub open_questions: Vec<String>,
}

/// Statement identifiers: upper-case ASCII letters, digits, `.`, `_` and `-`,
/// starting and ending with a letter or digit.
pub fn statement_id(id: &str, ctx: &str) -> Result<String, String> {
    let ok = |c: char| c.is_ascii_uppercase() || c.is_ascii_digit() || ".-_".contains(c);
    let edge = |c: Option<char>| c.is_some_and(|c| c.is_ascii_uppercase() || c.is_ascii_digit());
    if id.chars().all(ok) && edge(id.chars().next()) && edge(id.chars().last()) {
        Ok(id.to_string())
    } else {
        Err(format!(
            "{ctx}: statement id `{}` must use upper-case letters, digits, `.`, `_` or `-`",
            id.escape_debug()
        ))
    }
}

fn sha_field(y: &Yaml, key: &str, ctx: &str) -> Result<String, String> {
    let v = req(y, key, ctx)?;
    if v.len() == 64
        && v.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        Ok(v)
    } else {
        Err(format!(
            "{ctx}: `{key}` must be 64 lower-case hexadecimal digits"
        ))
    }
}

fn str_list(y: &Yaml, key: &str, ctx: &str) -> Result<Vec<String>, String> {
    list(y, key, ctx)?
        .iter()
        .map(|e| match e {
            Yaml::String(v) => clean(v, key, ctx, false),
            _ => Err(format!("{ctx}: every entry of `{key}` must be a string")),
        })
        .collect()
}

fn mapping<'a>(y: &'a Yaml, key: &str, ctx: &str) -> Result<&'a Yaml, String> {
    match &y[key] {
        v @ Yaml::Hash(_) => Ok(v),
        Yaml::BadValue | Yaml::Null => Err(format!("{ctx}: missing mapping `{key}`")),
        _ => Err(format!("{ctx}: field `{key}` must be a mapping")),
    }
}

/// A calendar date written YYYY-MM-DD (Gregorian, year 1000 or later).
fn is_date(d: &str) -> bool {
    let b = d.as_bytes();
    if !(b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b.iter()
            .enumerate()
            .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit()))
    {
        return false;
    }
    let (y, m, day): (u32, u32, u32) = (
        d[..4].parse().unwrap_or(0),
        d[5..7].parse().unwrap_or(0),
        d[8..].parse().unwrap_or(0),
    );
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let days = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    };
    y >= 1000 && day >= 1 && day <= days
}

/// Read and schema-check one statement record. References to other files
/// (proof, registry, reading points, Lean files) are resolved by the caller.
pub fn load_statement(path: &Path) -> Result<StatementRecord, String> {
    let y = load_yaml(path)?;
    let ctx = "statement record";
    only_keys(
        &y,
        &[
            "format",
            "id",
            "subject",
            "identifiers",
            "source",
            "kernel",
            "semantic",
            "physical",
            "open_questions",
        ],
        ctx,
    )?;
    let format = req(&y, "format", ctx)?;
    if format != STATEMENT_FORMAT {
        return Err(format!(
            "{ctx}: format `{format}` is unknown (expected {STATEMENT_FORMAT})"
        ));
    }
    let id = statement_id(&req(&y, "id", ctx)?, ctx)?;
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    if id != stem || path.extension().and_then(|e| e.to_str()) != Some("yaml") {
        return Err(format!(
            "{ctx}: id `{id}` must equal the file name `{}` (as ID.yaml)",
            stem.escape_debug()
        ));
    }
    let subject = req(&y, "subject", ctx)?;
    let mut identifiers = Vec::new();
    for e in list(&y, "identifiers", ctx)? {
        only_keys(e, &["reading_point", "role"], "identifiers")?;
        identifiers.push((
            plain_id(
                &req(e, "reading_point", "identifiers")?,
                "reading point",
                "identifiers",
            )?,
            req(e, "role", "identifiers")?,
        ));
    }

    let s = mapping(&y, "source", ctx)?;
    only_keys(s, &["registry", "commit", "excerpts"], "source")?;
    let source = StatementSource {
        registry: req(s, "registry", "source")?,
        commit: opt(s, "commit", "source")?,
        excerpts: str_list(s, "excerpts", "source")?,
    };
    if source.excerpts.is_empty() {
        return Err("source: at least one excerpt is required".into());
    }

    let k = mapping(&y, "kernel", ctx)?;
    only_keys(
        k,
        &[
            "proof",
            "sha256",
            "convention",
            "definitions_sha256",
            "axiom_set",
            "axiom_set_sha256",
            "checked_goal",
            "assumptions",
        ],
        "kernel",
    )?;
    let mut assumptions = Vec::new();
    match &k["assumptions"] {
        Yaml::Array(v) => {
            for e in v {
                only_keys(e, &["name", "formula"], "kernel.assumptions")?;
                let name = req(e, "name", "kernel.assumptions")?;
                let ident = name.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
                    && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
                if !ident {
                    return Err(format!(
                        "kernel.assumptions: name `{}` must be an identifier (letters, digits, `_`)",
                        name.escape_debug()
                    ));
                }
                if assumptions.iter().any(|(n, _)| *n == name) {
                    return Err(format!("kernel.assumptions: `{name}` is listed twice"));
                }
                assumptions.push((name, req(e, "formula", "kernel.assumptions")?));
            }
        }
        Yaml::BadValue => return Err("kernel: `assumptions` is required ([] for Γ = ∅)".into()),
        _ => return Err("kernel: `assumptions` must be a list".into()),
    }
    let kernel = KernelPins {
        proof: req(k, "proof", "kernel")?,
        sha256: sha_field(k, "sha256", "kernel")?,
        convention: req(k, "convention", "kernel")?,
        definitions_sha256: match &k["definitions_sha256"] {
            Yaml::BadValue => None,
            _ => Some(sha_field(k, "definitions_sha256", "kernel")?),
        },
        axiom_set: req(k, "axiom_set", "kernel")?,
        axiom_set_sha256: sha_field(k, "axiom_set_sha256", "kernel")?,
        checked_goal: req(k, "checked_goal", "kernel")?,
        assumptions,
    };

    let m = mapping(&y, "semantic", ctx)?;
    only_keys(
        m,
        &[
            "status",
            "assessed_by",
            "date",
            "basis",
            "assessed_goal",
            "matches",
            "differs",
            "lean",
        ],
        "semantic",
    )?;
    let status = req(m, "status", "semantic")?;
    if !SEMANTIC_STATUSES.contains(&status.as_str()) {
        return Err(format!(
            "semantic: status `{status}` is outside the vocabulary ({})",
            SEMANTIC_STATUSES.join(", ")
        ));
    }
    let basis = match &m["basis"] {
        Yaml::BadValue => None,
        b @ Yaml::Hash(_) => {
            only_keys(b, &["path", "sha256"], "semantic.basis")?;
            Some((
                req(b, "path", "semantic.basis")?,
                sha_field(b, "sha256", "semantic.basis")?,
            ))
        }
        _ => return Err("semantic: `basis` must be a mapping {path, sha256}".into()),
    };
    let mut lean = Vec::new();
    for e in list(m, "lean", "semantic")? {
        only_keys(e, &["file", "theorem"], "semantic.lean")?;
        lean.push(LeanRef {
            file: req(e, "file", "semantic.lean")?,
            theorem: req(e, "theorem", "semantic.lean")?,
        });
    }
    let semantic = SemanticAssessment {
        assessed_by: opt(m, "assessed_by", "semantic")?,
        date: opt(m, "date", "semantic")?,
        basis,
        assessed_goal: opt(m, "assessed_goal", "semantic")?,
        matches: str_list(m, "matches", "semantic")?,
        differs: str_list(m, "differs", "semantic")?,
        lean,
        status,
    };
    let sm = &semantic;
    if sm.status == "unassessed" {
        if sm.assessed_by.is_some()
            || sm.date.is_some()
            || sm.basis.is_some()
            || sm.assessed_goal.is_some()
            || !sm.matches.is_empty()
            || !sm.differs.is_empty()
            || !sm.lean.is_empty()
        {
            return Err(
                "semantic: an unassessed statement carries no assessor, date, basis, assessed goal, findings or Lean references"
                    .into(),
            );
        }
    } else {
        for (field, present) in [
            ("assessed_by", sm.assessed_by.is_some()),
            ("date", sm.date.is_some()),
            ("basis", sm.basis.is_some()),
            ("assessed_goal", sm.assessed_goal.is_some()),
        ] {
            if !present {
                return Err(format!(
                    "semantic: status `{}` requires `{field}`: an assessment is always attributed",
                    sm.status
                ));
            }
        }
        if sm
            .assessed_by
            .as_deref()
            .is_some_and(|a| a.trim().is_empty())
        {
            return Err("semantic: `assessed_by` must name the assessor".into());
        }
        if !sm.date.as_deref().is_some_and(is_date) {
            return Err("semantic: `date` must be written YYYY-MM-DD".into());
        }
        let need_matches = matches!(sm.status.as_str(), "matches" | "partial");
        let need_differs = matches!(sm.status.as_str(), "partial" | "differs");
        if need_matches && sm.matches.is_empty() {
            return Err(format!(
                "semantic: status `{}` requires `matches`",
                sm.status
            ));
        }
        if need_differs && sm.differs.is_empty() {
            return Err(format!(
                "semantic: status `{}` requires `differs`",
                sm.status
            ));
        }
        if sm.status == "matches" && !sm.differs.is_empty() {
            return Err("semantic: status `matches` lists no `differs`; use `partial`".into());
        }
        if sm.status == "differs" && !sm.matches.is_empty() {
            return Err("semantic: status `differs` lists no `matches`; use `partial`".into());
        }
    }

    let p = mapping(&y, "physical", ctx)?;
    only_keys(
        p,
        &["applicability", "reason", "reading_point", "correspondence"],
        "physical",
    )?;
    let physical = PhysicalScope {
        applicability: req(p, "applicability", "physical")?,
        reason: opt(p, "reason", "physical")?,
        reading_point: opt(p, "reading_point", "physical")?,
        correspondence: str_list(p, "correspondence", "physical")?,
    };
    match physical.applicability.as_str() {
        "not-applicable" => {
            if physical.reason.is_none()
                || physical.reading_point.is_some()
                || !physical.correspondence.is_empty()
            {
                return Err(
                    "physical: `not-applicable` requires a `reason` and names no correspondence"
                        .into(),
                );
            }
        }
        "relevant" => {
            if physical.reading_point.is_none() || physical.correspondence.is_empty() {
                return Err(
                    "physical: `relevant` requires a `reading_point` and at least one `correspondence` id"
                        .into(),
                );
            }
        }
        o => {
            return Err(format!(
                "physical: applicability `{o}` must be not-applicable or relevant"
            ))
        }
    }
    Ok(StatementRecord {
        open_questions: str_list(&y, "open_questions", ctx)?,
        id,
        subject,
        identifiers,
        source,
        kernel,
        semantic,
        physical,
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
            class: match opt(p, "class", &pc)? {
                Some(c) => Some(plain_id(&c, "class", &pc)?),
                None => None,
            },
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

// ---- RLT-003 M2: residue witness records (residues/*.yaml) ----

/// One closed quotient–remainder witness: `written` (n = d × q + s) proved by
/// `proof`, and `bound` (s + (k + 1) = d) proved by `bound_proof`.
#[derive(Clone, Debug)]
pub struct Witness {
    pub divisor: u64,
    pub written: String,
    pub proof: String,
    pub bound: String,
    pub bound_proof: String,
}

#[derive(Clone, Debug)]
pub struct ResidueEntry {
    pub r: u64,
    pub witnesses: Vec<Witness>,
}

#[derive(Clone, Debug)]
pub struct ClassEntry {
    pub n: u64,
    pub witness: Witness,
}

/// A single bound equation (`written` proved by `proof`).
#[derive(Clone, Debug)]
pub struct Checked {
    pub written: String,
    pub proof: String,
}

#[derive(Clone, Debug)]
pub struct ResiduesFile {
    pub id: String,
    pub convention: String,
    pub basis: Meaning,
    pub references: Vec<String>,
    /// Exact quotations, each a whole unit of a registered excerpt, reported
    /// as quotations; they never replace their source's definitions.
    pub quotes: Vec<Meaning>,
    pub residues: Vec<ResidueEntry>,
    pub classes: Vec<ClassEntry>,
    pub example: Option<Checked>,
    pub tally: Option<(Checked, Vec<u64>)>,
}

fn numeral_field(y: &Yaml, key: &str, ctx: &str) -> Result<u64, String> {
    match y[key].as_i64() {
        Some(n) if (0..=1000).contains(&n) => Ok(n as u64),
        _ => Err(format!("{ctx}: `{key}` must be a numeral 0..1000")),
    }
}

fn meaning(y: &Yaml, ctx: &str) -> Result<Meaning, String> {
    only_keys(y, &["text", "source"], ctx)?;
    Ok(Meaning {
        text: req(y, "text", ctx)?,
        source: req(y, "source", ctx)?,
    })
}

fn witness(y: &Yaml, divisor_key: &str, ctx: &str) -> Result<Witness, String> {
    let keys: Vec<&str> = vec![divisor_key, "written", "proof", "bound", "bound_proof"];
    only_keys(y, &keys, ctx)?;
    Ok(Witness {
        divisor: if divisor_key.is_empty() {
            0
        } else {
            numeral_field(y, divisor_key, ctx)?
        },
        written: req(y, "written", ctx)?,
        proof: req(y, "proof", ctx)?,
        bound: req(y, "bound", ctx)?,
        bound_proof: req(y, "bound_proof", ctx)?,
    })
}

pub fn load_residues(path: &Path) -> Result<ResiduesFile, String> {
    let y = load_yaml(path)?;
    let ctx = path.display().to_string();
    only_keys(
        &y,
        &[
            "id",
            "convention",
            "basis",
            "references",
            "quotes",
            "residues",
            "classes",
            "example",
            "tally",
        ],
        &ctx,
    )?;
    let id = plain_id(&req(&y, "id", &ctx)?, "record id", &ctx)?;
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    if id != stem {
        return Err(format!(
            "{ctx}: id `{id}` differs from the file name `{stem}`"
        ));
    }
    let basis = match &y["basis"] {
        Yaml::BadValue | Yaml::Null => {
            return Err(format!(
                "{ctx}: missing `basis` (a verbatim unit of a registered source)"
            ))
        }
        b => meaning(b, &ctx)?,
    };
    let mut residues = Vec::new();
    for e in list(&y, "residues", &ctx)? {
        only_keys(e, &["r", "witnesses"], &ctx)?;
        let r = numeral_field(e, "r", &ctx)?;
        let rc = format!("{ctx}: residue {r}");
        let witnesses = list(e, "witnesses", &rc)?
            .iter()
            .map(|w| witness(w, "p", &rc))
            .collect::<Result<_, _>>()?;
        residues.push(ResidueEntry { r, witnesses });
    }
    let mut classes = Vec::new();
    for e in list(&y, "classes", &ctx)? {
        let n = numeral_field(e, "n", &ctx)?;
        let cc = format!("{ctx}: class witness {n}");
        let mut w = witness(e, "n", &cc)?;
        w.divisor = 0; // the divisor of a class witness is the convention's modulus
        classes.push(ClassEntry { n, witness: w });
    }
    let checked = |y: &Yaml, extra: &[&str]| -> Result<Checked, String> {
        let mut keys = vec!["written", "proof"];
        keys.extend_from_slice(extra);
        only_keys(y, &keys, &ctx)?;
        Ok(Checked {
            written: req(y, "written", &ctx)?,
            proof: req(y, "proof", &ctx)?,
        })
    };
    let example = match &y["example"] {
        Yaml::BadValue | Yaml::Null => None,
        e => Some(checked(e, &[])?),
    };
    let tally = match &y["tally"] {
        Yaml::BadValue | Yaml::Null => None,
        e => {
            let c = checked(e, &["binds"])?;
            let binds = list(e, "binds", &ctx)?
                .iter()
                .map(|v| match v.as_i64() {
                    Some(n) if (0..=1000).contains(&n) => Ok(n as u64),
                    _ => Err(format!("{ctx}: tally `binds` lists residues")),
                })
                .collect::<Result<_, String>>()?;
            Some((c, binds))
        }
    };
    Ok(ResiduesFile {
        convention: req(&y, "convention", &ctx)?,
        basis,
        references: ids(&y, "references", &ctx)?,
        quotes: list(&y, "quotes", &ctx)?
            .iter()
            .map(|q| meaning(q, &ctx))
            .collect::<Result<_, _>>()?,
        residues,
        classes,
        example,
        tally,
        id,
    })
}

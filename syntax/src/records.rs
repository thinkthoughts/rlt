//! YAML records: source registry, conventions, status vocabulary, reading points.
//! These are data. Loading one never produces a theorem.

use crate::elab::Convention;
use std::collections::BTreeMap;
use std::path::Path;
use yaml_rust2::{Yaml, YamlLoader};

pub fn load_yaml(path: &Path) -> Result<Yaml, String> {
    let src = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
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
fn clean(v: &str, key: &str, ctx: &str, allow_newline: bool) -> Result<String, String> {
    match v
        .chars()
        .find(|c| c.is_control() && !(allow_newline && *c == '\n'))
    {
        Some(c) => Err(format!(
            "{ctx}: field `{key}` holds control character U+{:04X}",
            c as u32
        )),
        None => Ok(v.to_string()),
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
        .map(|x| {
            x.as_str()
                .map(str::to_string)
                .ok_or(format!("{ctx}: `{key}` must list strings"))
        })
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
    pub excerpts: Vec<Excerpt>,
}

pub fn load_registry(path: &Path) -> Result<BTreeMap<String, Source>, String> {
    let y = load_yaml(path)?;
    let mut out = BTreeMap::new();
    for e in list(&y, "sources", "registry")? {
        let id = req(e, "id", "source")?;
        let mut excerpts = Vec::new();
        for x in list(e, "excerpts", &id)? {
            excerpts.push(match x {
                Yaml::String(t) => Excerpt {
                    text: clean(t, "excerpt", &id, true)?,
                    defines: None,
                    whole: false,
                },
                Yaml::Hash(_) => Excerpt {
                    text: match &x["text"] {
                        // Registry excerpts may span lines of the source file.
                        Yaml::String(v) => clean(v, "text", &id, true)?,
                        _ => return Err(format!("{id}: excerpt `text` must be a string")),
                    },
                    defines: opt(x, "defines", &id)?,
                    whole: match opt(x, "split", &id)?.as_deref() {
                        None | Some("units") => false,
                        Some("whole") => true,
                        Some(o) => return Err(format!("{id}: split `{o}` must be units or whole")),
                    },
                },
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
            excerpts,
            id: id.clone(),
        };
        if out.insert(id.clone(), src).is_some() {
            return Err(format!("source `{id}` registered twice"));
        }
    }
    Ok(out)
}

#[derive(Clone, Debug)]
pub struct ConventionRecord {
    pub convention: Convention,
    pub reading: Vec<String>,
    pub sources: Vec<String>,
    pub authority: String,
    pub origin: Option<String>,
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
    let mut symbols = BTreeMap::new();
    if let Some(h) = y["symbols"].as_hash() {
        for (k, v) in h {
            if let (Some(k), Some(v)) = (k.as_str(), v.as_str()) {
                symbols.insert(k.to_string(), v.to_string());
            }
        }
    }
    Ok(ConventionRecord {
        convention: Convention {
            id: id.clone(),
            elaborator: req(&y, "elaborator", &id)?,
            symbols,
        },
        reading: strings(&y, "reading", &id)?,
        sources: strings(&y, "sources", &id)?,
        authority: req(&y, "authority", &id)?,
        origin: opt(&y, "origin", &id)?,
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

#[derive(Clone, Debug)]
pub struct Correspondence {
    pub id: String,
    pub kind: String,
    pub description: String,
    pub status: String,
    pub evidence: Vec<(String, String)>,
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
                    evidence: list(e, "evidence", &id)?
                        .iter()
                        .map(|v| Ok((req(v, "type", &id)?, req(v, "ref", &id)?)))
                        .collect::<Result<_, String>>()?,
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
    pub evidence: Vec<(String, String)>,
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
    pub evidence: Vec<(String, String)>,
}

#[derive(Clone, Debug)]
pub struct PointsFile {
    pub id: String,
    pub context: String,
    /// A verbatim excerpt of a registered source that states this context.
    pub basis: Option<Meaning>,
    /// Named reading points (readingpoints/*.yaml) referenced by id only.
    pub references: Vec<String>,
    pub points: Vec<Point>,
    pub counts: Vec<Count>,
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

fn evidence(e: &Yaml, ctx: &str) -> Result<Vec<(String, String)>, String> {
    list(e, "evidence", ctx)?
        .iter()
        .map(|v| Ok((req(v, "type", ctx)?, req(v, "ref", ctx)?)))
        .collect()
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
            "counts",
            "relations",
        ],
        &ctx,
    )?;
    let id = req(&y, "id", &ctx)?;
    if id != stem {
        return Err(format!(
            "{ctx}: record id {id:?} differs from its file name"
        ));
    }
    let basis = match &y["basis"] {
        Yaml::BadValue | Yaml::Null => None,
        b => {
            only_keys(b, &["text", "source"], &ctx)?;
            Some(Meaning {
                text: req(b, "text", &ctx)?,
                source: req(b, "source", &ctx)?,
            })
        }
    };
    let mut points = Vec::new();
    for p in list(&y, "points", &ctx)? {
        only_keys(
            p,
            &["id", "kind", "class", "description", "scope", "physical"],
            &ctx,
        )?;
        let pid = req(p, "id", &ctx)?;
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
        let cid = req(c, "id", &ctx)?;
        let cc = format!("{ctx}: count {cid}");
        let groups = list(c, "groups", &cc)?
            .iter()
            .map(|g| match g {
                Yaml::Array(v) => v
                    .iter()
                    .map(|m| {
                        m.as_str()
                            .map(str::to_string)
                            .ok_or(format!("{cc}: group members are point ids"))
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
    let mut relations = Vec::new();
    for r in list(&y, "relations", &ctx)? {
        only_keys(r, &["id", "type", "between", "status", "evidence"], &ctx)?;
        let rid = req(r, "id", &ctx)?;
        let rc = format!("{ctx}: relation {rid}");
        relations.push(RelationClaim {
            rtype: req(r, "type", &rc)?,
            between: strings(r, "between", &rc)?,
            status: req(r, "status", &rc)?,
            evidence: evidence(r, &rc)?,
            id: rid,
        });
    }
    Ok(PointsFile {
        context: req(&y, "context", &ctx)?,
        references: strings(&y, "references", &ctx)?,
        basis,
        points,
        counts,
        relations,
        id,
    })
}

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
fn opt(y: &Yaml, key: &str, ctx: &str) -> Result<Option<String>, String> {
    match &y[key] {
        Yaml::BadValue | Yaml::Null => Ok(None),
        Yaml::String(v) => Ok(Some(v.clone())),
        _ => Err(format!("{ctx}: field `{key}` must be a string")),
    }
}

fn req(y: &Yaml, key: &str, ctx: &str) -> Result<String, String> {
    match &y[key] {
        Yaml::String(v) => Ok(v.clone()),
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
const INLINE_TAGS: [&str; 7] = ["a", "strong", "em", "b", "i", "code", "sup"];

fn collapse(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The verbatim units of an excerpt. Inline tags (a, strong, em, …) are
/// removed with their text kept; every other tag delimits units. With
/// `whole`, all tags are transparent and the excerpt is one unit. Whitespace
/// runs collapse to one space. A meaning must equal a unit exactly.
pub fn units(text: &str, whole: bool) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut rest = text;
    while let Some(i) = rest.find('<') {
        let Some(j) = rest[i..].find('>') else { break };
        cur.push_str(&rest[..i]);
        let tag = &rest[i + 1..i + j];
        let name: String = tag
            .trim_start_matches('/')
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric())
            .collect::<String>()
            .to_ascii_lowercase();
        if !whole && !INLINE_TAGS.contains(&name.as_str()) {
            let u = collapse(&cur);
            if !u.is_empty() {
                out.push(u);
            }
            cur.clear();
        }
        rest = &rest[i + j + 1..];
    }
    cur.push_str(rest);
    let u = collapse(&cur);
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
                    text: t.clone(),
                    defines: None,
                    whole: false,
                },
                Yaml::Hash(_) => Excerpt {
                    text: req(x, "text", &id)?,
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

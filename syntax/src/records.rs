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

fn s(y: &Yaml, key: &str) -> Option<String> {
    y[key].as_str().map(str::to_string)
}

fn req(y: &Yaml, key: &str, ctx: &str) -> Result<String, String> {
    s(y, key).ok_or_else(|| format!("{ctx}: missing string field `{key}`"))
}

fn list<'a>(y: &'a Yaml, key: &str) -> &'a [Yaml] {
    y[key].as_vec().map(|v| v.as_slice()).unwrap_or(&[])
}

#[derive(Clone, Debug)]
pub struct Source {
    pub id: String,
    pub status: String,
    pub location: String,
    pub excerpts: Vec<String>,
}

pub fn load_registry(path: &Path) -> Result<BTreeMap<String, Source>, String> {
    let y = load_yaml(path)?;
    let mut out = BTreeMap::new();
    for e in list(&y, "sources") {
        let id = req(e, "id", "source")?;
        let src = Source {
            status: req(e, "status", &id)?,
            location: s(e, "location").unwrap_or_default(),
            excerpts: list(e, "excerpts")
                .iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect(),
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
        reading: list(&y, "reading")
            .iter()
            .filter_map(|x| x.as_str().map(str::to_string))
            .collect(),
        sources: list(&y, "sources")
            .iter()
            .filter_map(|x| x.as_str().map(str::to_string))
            .collect(),
        authority: req(&y, "authority", &id)?,
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
    let strs = |k: &str| {
        list(&y, k)
            .iter()
            .filter_map(|x| x.as_str().map(str::to_string))
            .collect()
    };
    Ok(StatusVocabulary {
        correspondence: strs("correspondence_status"),
        lift: strs("lift_status"),
        allowed_support: strs("allowed_support"),
        without_evidence: strs("status_without_evidence"),
        referent_only: strs("referent_only_status"),
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
        meanings: list(&y, "meanings")
            .iter()
            .map(|e| meaning(e, "text"))
            .collect::<Result<_, _>>()?,
        examples: list(&y, "examples")
            .iter()
            .map(|e| meaning(e, "written"))
            .collect::<Result<_, _>>()?,
        relationships: list(&y, "relationships")
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
        correspondence: list(&y, "correspondence")
            .iter()
            .map(|e| {
                Ok(Correspondence {
                    id: req(e, "id", &id)?,
                    kind: req(e, "kind", &id)?,
                    description: req(e, "description", &id)?,
                    status: req(e, "status", &id)?,
                    evidence: list(e, "evidence")
                        .iter()
                        .map(|v| Ok((req(v, "type", &id)?, req(v, "ref", &id)?)))
                        .collect::<Result<_, String>>()?,
                    source: s(e, "source"),
                })
            })
            .collect::<Result<_, String>>()?,
        lifts: list(&y, "lifts")
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

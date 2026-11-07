//! Proof files (`*.rlt`) and the axiom file, read into kernel proof objects.
//!
//! This reader is untrusted: it only builds the proof object that the kernel
//! replays. Its one interpretive act — elaborating the written goal and
//! written assumptions under the named convention — is reported verbatim.

use crate::elab::{elaborate_formula, Convention};
use crate::sexpr::{parse_all, Sexp};
use crate::written::Written;
use rlt_kernel::{Arg, Axiom, Formula, Mode, ProofObject, Step, Term};

#[derive(Clone, Debug)]
pub struct ProofFile {
    pub id: String,
    pub readingpoint: Option<String>,
    pub relationship: Option<String>,
    pub convention: String,
    pub context: Option<(String, String)>,
    pub goal: Written,
    pub mode: Mode,
    pub assumptions: Vec<(String, Written)>,
    /// Id of a registered axiom set (spec/axioms.yaml); never a path.
    pub axiom_set: String,
    pub steps: Vec<Step>,
}

/// Largest numeral accepted in proof syntax (kernel numerals are unary).
pub const MAX_NUMERAL: u64 = 1_000;

/// A term in proof syntax: integers are numerals; `(S t)`, `(+ a b)`;
/// other symbols become constants, which the kernel's sort check judges.
pub fn term(e: &Sexp, vars: &[String]) -> Result<Term, String> {
    match e {
        Sexp::Int(n) if *n > MAX_NUMERAL => Err(format!(
            "numeral {n} exceeds the RLT-001 unary limit {MAX_NUMERAL}"
        )),
        Sexp::Int(n) => Ok(Term::numeral(*n)),
        Sexp::Sym(s) if vars.contains(s) => Ok(Term::Var(s.clone())),
        Sexp::Sym(s) => Ok(Term::App(s.clone(), vec![])),
        Sexp::List(items) => match items.split_first() {
            Some((Sexp::Sym(f), args)) => Ok(Term::App(
                f.clone(),
                args.iter()
                    .map(|a| term(a, vars))
                    .collect::<Result<_, _>>()?,
            )),
            _ => Err(format!("term: expected (symbol args…), found {e:?}")),
        },
        Sexp::Str(s) => Err(format!(
            "term: found string \"{s}\"; write terms as S-expressions"
        )),
    }
}

pub fn formula(e: &Sexp, vars: &[String]) -> Result<Formula, String> {
    match e.head() {
        Some(("=", [a, b])) => Ok(Formula::new(term(a, vars)?, term(b, vars)?)),
        _ => Err(format!("formula: expected (= lhs rhs), found {e:?}")),
    }
}

fn string_of(rest: &[Sexp], key: &str) -> Result<String, String> {
    match rest {
        [Sexp::Str(s)] => Ok(s.clone()),
        [Sexp::Sym(s)] => Ok(s.clone()),
        _ => Err(format!("({key} …) takes one value")),
    }
}

fn step(e: &Sexp) -> Result<Step, String> {
    let items = e
        .list()
        .ok_or("step: expected (label claim (rule args…))")?;
    let [label, claim, rule] = items else {
        return Err(format!(
            "step: expected (label claim (rule args…)), found {e:?}"
        ));
    };
    let label = label
        .sym()
        .ok_or("step label must be a symbol")?
        .to_string();
    let claim = formula(claim, &[]).map_err(|m| format!("step {label}: {m}"))?;
    let (rule_name, args) = rule
        .head()
        .ok_or(format!("step {label}: expected (rule args…)"))?;
    let mut out = Vec::new();
    for (i, a) in args.iter().enumerate() {
        let arg = match (rule_name, i, a) {
            ("refl", _, a) => Arg::Term(term(a, &[])?),
            ("inst" | "cong" | "hyp", 0, Sexp::Sym(s)) => Arg::Name(s.clone()),
            ("inst", _, Sexp::List(b)) => match b.as_slice() {
                [Sexp::Sym(v), t] => Arg::Bind(v.clone(), term(t, &[])?),
                _ => return Err(format!("step {label}: inst binding must be (var term)")),
            },
            (_, _, Sexp::Sym(s)) => Arg::Step(s.clone()),
            (_, _, other) => Arg::Term(term(other, &[])?),
        };
        out.push(arg);
    }
    Ok(Step {
        label,
        claim,
        rule: rule_name.to_string(),
        args: out,
    })
}

/// Axiom-set ids: upper-case letters, digits and `-`.
fn is_set_id(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-')
}

/// Record ids (reading points, relationships): lower-case letters, digits, `.`, `_`, `-`;
/// no path separators and no `..`.
pub fn record_id(s: String) -> Result<String, String> {
    let ok = !s.is_empty()
        && !s.starts_with('.')
        && !s.contains("..")
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || ".-_".contains(c));
    if ok {
        Ok(s)
    } else {
        Err(format!(
            "record id `{s}`: use lower-case letters, digits, `.`, `_`, `-`"
        ))
    }
}

/// Upper bound on the term nodes a proof file may build (numerals are unary).
pub const MAX_FILE_NODES: u64 = 1_000_000;

fn node_budget(e: &Sexp) -> u64 {
    match e {
        Sexp::Int(n) => n.saturating_add(1),
        Sexp::List(v) => v
            .iter()
            .fold(1u64, |acc, x| acc.saturating_add(node_budget(x))),
        _ => 1,
    }
}

pub fn parse_proof_file(src: &str) -> Result<ProofFile, String> {
    let top = parse_all(src)?;
    let budget = top
        .iter()
        .fold(0u64, |a, e| a.saturating_add(node_budget(e)));
    if budget > MAX_FILE_NODES {
        return Err(format!(
            "proof file would build {budget} term nodes, above the limit {MAX_FILE_NODES}"
        ));
    }
    let [root] = top.as_slice() else {
        return Err("a proof file holds exactly one (rlt-proof …) form".into());
    };
    let Some(("rlt-proof", fields)) = root.head() else {
        return Err("expected (rlt-proof …)".into());
    };
    let mut pf = ProofFile {
        id: String::new(),
        readingpoint: None,
        relationship: None,
        convention: String::new(),
        context: None,
        goal: Written::new(""),
        mode: Mode::Unconditional,
        assumptions: Vec::new(),
        axiom_set: String::new(),
        steps: Vec::new(),
    };
    let mut mode_seen = false;
    for f in fields {
        let (key, rest) = f
            .head()
            .ok_or(format!("field: expected (key value…), found {f:?}"))?;
        match key {
            "id" => pf.id = string_of(rest, key)?,
            "readingpoint" => pf.readingpoint = Some(record_id(string_of(rest, key)?)?),
            "relationship" => pf.relationship = Some(record_id(string_of(rest, key)?)?),
            "convention" => pf.convention = string_of(rest, key)?,
            "axioms" => match rest {
                [Sexp::Sym(id)] if is_set_id(id) => pf.axiom_set = id.clone(),
                _ => return Err("(axioms SET-ID): name a registered axiom set, e.g. (axioms PEANO-ADD); paths are refused".into()),
            },
            "goal" => pf.goal = Written::new(&string_of(rest, key)?),
            "context" => match rest {
                [Sexp::Sym(name), Sexp::Sym(kind)] => {
                    pf.context = Some((name.clone(), kind.clone()))
                }
                _ => return Err("(context NAME KIND)".into()),
            },
            "mode" => {
                mode_seen = true;
                pf.mode = match string_of(rest, key)?.as_str() {
                    "unconditional" => Mode::Unconditional,
                    "conditional" => Mode::Conditional,
                    m => return Err(format!("mode `{m}`: use unconditional or conditional")),
                }
            }
            "assume" => match rest {
                [Sexp::Sym(name), Sexp::Str(w)] => {
                    pf.assumptions.push((name.clone(), Written::new(w)))
                }
                _ => return Err("(assume NAME \"written formula\")".into()),
            },
            "steps" => pf.steps = rest.iter().map(step).collect::<Result<_, _>>()?,
            other => return Err(format!("unknown field `{other}`")),
        }
    }
    for (k, empty) in [
        ("id", pf.id.is_empty()),
        ("convention", pf.convention.is_empty()),
        ("axioms", pf.axiom_set.is_empty()),
        ("goal", pf.goal.text.is_empty()),
    ] {
        if empty {
            return Err(format!("missing field ({k} …)"));
        }
    }
    if !mode_seen {
        return Err("missing field (mode unconditional|conditional)".into());
    }
    Ok(pf)
}

impl ProofFile {
    /// Elaborate the written goal and assumptions, giving the kernel's proof object.
    pub fn to_proof_object(&self, conv: &Convention) -> Result<ProofObject, String> {
        let goal = elaborate_formula(&self.goal, conv).map_err(|e| format!("goal: {e}"))?;
        let assumptions = self
            .assumptions
            .iter()
            .map(|(n, w)| {
                elaborate_formula(w, conv)
                    .map(|f| (n.clone(), f))
                    .map_err(|e| format!("assume {n}: {e}"))
            })
            .collect::<Result<_, _>>()?;
        Ok(ProofObject {
            goal,
            mode: self.mode,
            assumptions,
            steps: self.steps.clone(),
        })
    }
}

/// `(axiom NAME (vars…) (= lhs rhs) (source ID))`
pub fn parse_axiom_file(src: &str) -> Result<Vec<Axiom>, String> {
    parse_all(src)?
        .iter()
        .map(|e| {
            let Some(("axiom", [Sexp::Sym(name), Sexp::List(vs), eq, src])) = e.head() else {
                return Err(format!(
                    "expected (axiom NAME (vars…) (= l r) (source ID)), found {e:?}"
                ));
            };
            let vars: Vec<String> = vs
                .iter()
                .map(|v| {
                    v.sym()
                        .map(str::to_string)
                        .ok_or("axiom variables are symbols".to_string())
                })
                .collect::<Result<_, _>>()?;
            let source = match src.head() {
                Some(("source", [Sexp::Sym(s)])) => s.clone(),
                _ => return Err(format!("axiom {name}: expected (source ID)")),
            };
            Axiom::new(name, vars.clone(), formula(eq, &vars)?, &source)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const OK: &str = r#"(rlt-proof (id t) (readingpoint math5) (relationship math5.pairing)
        (convention NAT-DEC) (goal "1 = 1") (mode unconditional) (axioms PEANO-ADD)
        (steps (s1 (= 1 1) (refl 1))))"#;

    #[test]
    fn a_well_formed_file_parses() {
        let pf = parse_proof_file(OK).unwrap();
        assert_eq!(pf.axiom_set, "PEANO-ADD");
        assert_eq!(pf.steps.len(), 1);
    }

    #[test]
    fn axiom_paths_are_refused() {
        let bad = OK.replace("(axioms PEANO-ADD)", r#"(axioms "spec/peano.rlt")"#);
        assert!(parse_proof_file(&bad)
            .unwrap_err()
            .contains("paths are refused"));
        let bad = OK.replace("(axioms PEANO-ADD)", "(axioms ../x)");
        assert!(parse_proof_file(&bad).is_err());
    }

    #[test]
    fn record_ids_reject_traversal() {
        let bad = OK.replace(
            "(readingpoint math5)",
            r#"(readingpoint "../../etc/hostname")"#,
        );
        assert!(parse_proof_file(&bad).unwrap_err().contains("record id"));
        assert!(record_id("math5.pairing".into()).is_ok());
        assert!(record_id("a/b".into()).is_err());
    }

    #[test]
    fn oversized_files_are_refused_before_building_terms() {
        let wide = OK.replace(
            "(refl 1)",
            &format!("(refl {})", "(S ".repeat(10) + "1000" + &")".repeat(10)),
        );
        assert!(parse_proof_file(&wide).is_ok());
        let huge = OK.replace(
            "(steps",
            &format!("(steps {}", "(x (= 999 999) (refl 999)) ".repeat(1200)),
        );
        assert!(parse_proof_file(&huge).unwrap_err().contains("term nodes"));
    }
}

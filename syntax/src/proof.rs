//! Proof files (`*.rlt`) and the axiom file, read into kernel proof objects.
//!
//! This reader is untrusted: it only builds the proof object that the kernel
//! replays. Its one interpretive act — elaborating the written goal and
//! written assumptions under the named convention — is reported verbatim.

use crate::elab::{elaborate_formula, elaborate_prop, Convention};
use crate::sexpr::{parse_all, Sexp};
use crate::written::Written;
use rlt_kernel::{Arg, Axiom, Formula, Mode, ProofObject, Prop, Step, Term};

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
    /// Declared parameters (eigenvariables), `(params a b)`; RLT-003 M3a.
    pub params: Vec<String>,
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

/// Names in scope while reading a step: declared parameters, declared
/// assumption names, and the binders enclosing the current position
/// (innermost last). Every name resolves once, into exactly one class.
#[derive(Clone, Debug, Default)]
pub struct Scope {
    pub params: Vec<String>,
    pub assumptions: Vec<String>,
    pub binders: Vec<String>,
    /// Axiom variables, when reading an axiom body (RLT-003 M3b).
    pub vars: Vec<String>,
    /// Pinned definitions, when the file's convention reads them (NAT-DEF).
    pub defs: Option<std::rc::Rc<crate::defs::Definitions>>,
}

impl Scope {
    fn bind(&self, x: &str) -> Result<Scope, String> {
        if !is_ident(x) || x.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
            return Err(format!(
                "binder `{}`: use a lower-case identifier",
                x.escape_debug()
            ));
        }
        if self.binders.iter().any(|b| b == x)
            || self.params.iter().any(|p| p == x)
            || self.assumptions.iter().any(|a| a == x)
        {
            return Err(format!(
                "shadowed name `{x}`: a binder may not reuse a binder, parameter or assumption name"
            ));
        }
        // RLT-003 M3b: nor an axiom variable, nor a definition name.
        if self.vars.iter().any(|v| v == x)
            || self.defs.as_ref().is_some_and(|d| d.get(x).is_some())
        {
            return Err(format!(
                "shadowed name `{x}`: a binder may not reuse an axiom variable or a definition name"
            ));
        }
        let mut s = self.clone();
        s.binders.push(x.to_string());
        Ok(s)
    }
}

/// A proof term with parameters and bound variables (RLT-003 M3a). With no
/// parameters and no binders in scope it reads exactly as `term`.
pub fn scoped_term(e: &Sexp, sc: &Scope) -> Result<Term, String> {
    match e {
        Sexp::Sym(s) => {
            if let Some(k) = sc.binders.iter().rposition(|b| b == s) {
                Ok(Term::Bound(sc.binders.len() - 1 - k))
            } else if sc.params.contains(s) {
                Ok(Term::Param(s.clone()))
            } else if sc.vars.contains(s) {
                Ok(Term::Var(s.clone()))
            } else {
                term(e, &[])
            }
        }
        Sexp::List(items) => match items.split_first() {
            Some((Sexp::Sym(f), args)) => Ok(Term::App(
                f.clone(),
                args.iter()
                    .map(|a| scoped_term(a, sc))
                    .collect::<Result<_, _>>()?,
            )),
            _ => Err(format!("term: expected (symbol args…), found {e:?}")),
        },
        other => term(other, &[]),
    }
}

/// A formula in proof syntax: `(= s t)`, `false`, `(and φ ψ)`, `(or φ ψ)`,
/// `(imp φ ψ)`, `(not φ)` (read as `(imp φ false)`), `(forall x φ)`,
/// `(exists x φ)`.
pub fn prop(e: &Sexp, sc: &Scope) -> Result<Prop, String> {
    let bin = |a: &Sexp, b: &Sexp| -> Result<(Box<Prop>, Box<Prop>), String> {
        Ok((Box::new(prop(a, sc)?), Box::new(prop(b, sc)?)))
    };
    match (e, e.head()) {
        (Sexp::Sym(s), _) if s == "false" => Ok(Prop::False),
        (_, Some(("=", [a, b]))) => Ok(Prop::eq(scoped_term(a, sc)?, scoped_term(b, sc)?)),
        (_, Some(("and", [a, b]))) => bin(a, b).map(|(x, y)| Prop::And(x, y)),
        (_, Some(("or", [a, b]))) => bin(a, b).map(|(x, y)| Prop::Or(x, y)),
        (_, Some(("imp", [a, b]))) => bin(a, b).map(|(x, y)| Prop::Imp(x, y)),
        (_, Some(("not", [a]))) => Ok(Prop::Imp(Box::new(prop(a, sc)?), Box::new(Prop::False))),
        (_, Some((q @ ("forall" | "exists"), [Sexp::Sym(x), body]))) => {
            let inner = sc.bind(x)?;
            let b = Box::new(prop(body, &inner)?);
            Ok(if q == "forall" {
                Prop::Forall(x.clone(), b)
            } else {
                Prop::Exists(x.clone(), b)
            })
        }
        // RLT-003 M3b: a use of a pinned definition, expanded here (E1, E2).
        (_, Some((name, args))) if sc.defs.as_ref().is_some_and(|d| d.get(name).is_some()) => {
            let d = sc.defs.as_ref().expect("checked");
            let def = d.get(name).expect("checked");
            d.mark(name);
            let ts = args
                .iter()
                .map(|a| scoped_term(a, sc))
                .collect::<Result<Vec<_>, _>>()?;
            def.expand(&ts)
        }
        _ => Err(format!("formula: expected (= lhs rhs), found {e:?}")),
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

fn step(e: &Sexp, sc: &Scope) -> Result<Step, String> {
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
    let claim = prop(claim, sc).map_err(|m| format!("step {label}: {m}"))?;
    let (rule_name, args) = rule
        .head()
        .ok_or(format!("step {label}: expected (rule args…)"))?;
    let name = |a: &Sexp, what: &str| -> Result<Arg, String> {
        match a {
            Sexp::Sym(s) => Ok(Arg::Name(s.clone())),
            other => Err(format!("step {label}: expected {what}, found {other:?}")),
        }
    };
    let param = |a: &Sexp| -> Result<Arg, String> {
        match a {
            Sexp::Sym(s) if sc.params.contains(s) => Ok(Arg::Name(s.clone())),
            Sexp::Sym(s) => Err(format!("step {label}: parameter `{s}` is undeclared")),
            other => Err(format!(
                "step {label}: expected a parameter name, found {other:?}"
            )),
        }
    };
    let label_arg = |a: &Sexp| -> Result<Arg, String> {
        match a {
            Sexp::Sym(s) => Ok(Arg::Step(s.clone())),
            other => Err(format!(
                "step {label}: expected a step label, found {other:?}"
            )),
        }
    };
    let mut out = Vec::new();
    for (i, a) in args.iter().enumerate() {
        let arg = match (rule_name, i, a) {
            ("refl", _, a) => Arg::Term(scoped_term(a, sc)?),
            ("inst" | "cong" | "hyp", 0, Sexp::Sym(s)) => Arg::Name(s.clone()),
            ("inst", _, Sexp::List(b)) => match b.as_slice() {
                [Sexp::Sym(v), t] => Arg::Bind(v.clone(), scoped_term(t, sc)?),
                _ => return Err(format!("step {label}: inst binding must be (var term)")),
            },
            ("subst", 1, m) => match m.head() {
                Some(("at", [Sexp::Sym(x), body])) => {
                    let inner = sc
                        .bind(x)
                        .map_err(|e| format!("step {label}: motive: {e}"))?;
                    let b = prop(body, &inner).map_err(|e| format!("step {label}: motive: {e}"))?;
                    Arg::Motive(x.clone(), b)
                }
                _ => return Err(format!("step {label}: subst motive must be (at x formula)")),
            },
            // Discharged names are assumption names: identifiers, like
            // declared ones (audit at 6ba9f1e, weakness 1).
            ("or-cases", 1 | 3, Sexp::Sym(s)) | ("imp-intro", 0, Sexp::Sym(s)) | ("exists-elim", 2, Sexp::Sym(s))
                if !is_ident(s) =>
            {
                return Err(format!(
                    "step {label}: assumption name {s:?} is not an identifier (ASCII letters, digits, _)"
                ))
            }
            ("or-cases", 1 | 3, a) => name(a, "an assumption name")?,
            ("imp-intro", 0, a) => name(a, "an assumption name")?,
            ("exists-elim", 2, a) => name(a, "an assumption name")?,
            ("forall-intro", 0, a) | ("exists-elim", 1, a) => param(a)?,
            ("forall-elim", 1, a) | ("exists-intro", 0, a) => Arg::Term(scoped_term(a, sc)?),
            (
                "subst" | "and-intro" | "and-left" | "and-right" | "or-left" | "or-right"
                | "or-cases" | "imp-intro" | "imp-elim" | "false-elim" | "forall-intro"
                | "forall-elim" | "exists-intro" | "exists-elim",
                _,
                a,
            ) => label_arg(a)?,
            (_, _, Sexp::Sym(s)) => Arg::Step(s.clone()),
            (_, _, other) => Arg::Term(scoped_term(other, sc)?),
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

/// Assumption names: identifiers, so they print unambiguously in reports
/// and in the generated Lean replay.
fn is_ident(s: &str) -> bool {
    let mut cs = s.chars();
    matches!(cs.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && cs.all(|c| c.is_ascii_alphanumeric() || c == '_')
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
    parse_proof_file_with(src, None)
}

/// The convention whose proof files read definitions (RLT-003 M3b).
pub const DEFINITIONS_CONVENTION: &str = "NAT-DEF";

/// Parse a proof file; `defs` supplies the pinned definitions, read only when
/// the file's convention is NAT-DEF. Every other file parses exactly as before.
pub fn parse_proof_file_with(
    src: &str,
    defs: Option<Result<std::rc::Rc<crate::defs::Definitions>, String>>,
) -> Result<ProofFile, String> {
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
        params: Vec::new(),
        axiom_set: String::new(),
        steps: Vec::new(),
    };
    // Parameters and assumption names are read first: steps resolve names against them.
    let mut scope = Scope::default();
    let reads_definitions = fields.iter().any(
        |f| matches!(f.head(), Some(("convention", [Sexp::Sym(c)])) if c == DEFINITIONS_CONVENTION),
    );
    if reads_definitions {
        match defs {
            Some(Ok(d)) => scope.defs = Some(d),
            Some(Err(e)) => return Err(format!("definitions: {e}")),
            None => return Err("definitions: unavailable to this reader".into()),
        }
    }
    for f in fields {
        match f.head() {
            Some(("params", names)) => {
                for n in names {
                    match n {
                        Sexp::Sym(s)
                            if is_ident(s)
                                && s.chars().next().is_some_and(|c| c.is_ascii_lowercase()) =>
                        {
                            if scope.defs.as_ref().is_some_and(|d| d.get(s).is_some()) {
                                return Err(format!("parameter `{s}` collides with a definition name"));
                            }
                            if scope.params.contains(s) {
                                return Err(format!("parameter `{s}` declared twice"));
                            }
                            scope.params.push(s.clone())
                        }
                        other => {
                            return Err(format!(
                                "(params …): parameter names are lower-case identifiers, found {other:?}"
                            ))
                        }
                    }
                }
            }
            Some(("assume", [Sexp::Sym(n), ..])) => scope.assumptions.push(n.clone()),
            _ => {}
        }
    }
    let mut mode_seen = false;
    let mut seen_keys: Vec<String> = Vec::new();
    for f in fields {
        let (key, rest) = f
            .head()
            .ok_or(format!("field: expected (key value…), found {f:?}"))?;
        if key != "assume" {
            if seen_keys.iter().any(|k| k == key) {
                return Err(format!("field ({key} …) appears twice"));
            }
            seen_keys.push(key.to_string());
        }
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
                [Sexp::Sym(name), Sexp::Str(w)] if is_ident(name) => {
                    pf.assumptions.push((name.clone(), Written::new(w)))
                }
                [Sexp::Sym(name), _] => {
                    return Err(format!(
                        "assumption name `{name}`: use an identifier [A-Za-z_][A-Za-z0-9_]*"
                    ))
                }
                _ => return Err("(assume NAME \"written formula\")".into()),
            },
            "params" => pf.params = scope.params.clone(),
            "steps" => {
                pf.steps = rest
                    .iter()
                    .map(|e| step(e, &scope))
                    .collect::<Result<_, _>>()?
            }
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
        let goal = elaborate_goal(&self.goal, conv).map_err(|e| format!("goal: {e}"))?;
        let assumptions = self
            .assumptions
            .iter()
            .map(|(n, w)| {
                elaborate_goal(w, conv)
                    .map(|f| (n.clone(), f))
                    .map_err(|e| format!("assume {n}: {e}"))
            })
            .collect::<Result<_, _>>()?;
        Ok(ProofObject {
            goal,
            mode: self.mode,
            assumptions,
            params: self.params.clone(),
            steps: self.steps.clone(),
        })
    }
}

/// A written goal: an equation reads exactly as before (RLT-001 to M2,
/// including every error message); a written formula with logical symbols
/// reads through `elaborate_prop` when the convention lists them.
pub fn elaborate_goal(w: &Written, conv: &Convention) -> Result<Prop, String> {
    match elaborate_formula(w, conv) {
        Ok(f) => Ok(Prop::Eq(f)),
        Err(e) => {
            let logical = w.tokens.iter().any(|t| {
                crate::elab::CONNECTIVES.contains(&t.as_str())
                    || t.chars().all(|c| c.is_ascii_lowercase())
                    || conv.definitions.contains(t)
            });
            if logical && !conv.connectives.is_empty() {
                elaborate_prop(w, conv)
            } else {
                Err(e)
            }
        }
    }
}

/// `(axiom NAME (vars…) BODY (source ID))`: BODY is `(= lhs rhs)`, or from
/// RLT-003 M3b any formula in proof syntax over the declared variables.
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
            // An equation reads exactly as before M3b; any other body is a
            // formula axiom (RLT-003 M3b), its own binders read as indices and
            // its free names as the declared axiom variables.
            match eq.head() {
                Some(("=", _)) => Axiom::new(name, vars.clone(), formula(eq, &vars)?, &source),
                _ => {
                    let sc = Scope {
                        vars: vars.clone(),
                        ..Scope::default()
                    };
                    let body = prop(eq, &sc).map_err(|e| format!("axiom {name}: {e}"))?;
                    Axiom::formula(name, vars.clone(), body, &source)
                }
            }
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
    fn duplicate_fields_and_odd_assumption_names_are_refused() {
        let dup = OK.replace(
            "(axioms PEANO-ADD)",
            "(axioms PEANO-ADD) (axioms PEANO-ADD)",
        );
        assert!(parse_proof_file(&dup)
            .unwrap_err()
            .contains("appears twice"));
        let odd = OK.replace(
            "(mode unconditional)",
            r#"(mode unconditional) (assume -/ "1 = 1")"#,
        );
        assert!(parse_proof_file(&odd).unwrap_err().contains("identifier"));
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

//! Definitions (RLT-003 M3b): pinned abbreviations expanded before the kernel.
//!
//! File syntax (`spec/definitions.rlt`):
//! ```text
//! (define NAME (PARAMS…) BODY (notation infix "<") (source ID))
//! (define NAME (PARAMS…) BODY (notation prefix "free235") (origin rlt-authored))
//! ```
//! BODY is a formula in proof syntax; it may use the parameters and every
//! definition earlier in the file, never itself or a later one (acyclic).
//!
//! Representation: the body is stored with its parameters as the outermost
//! bound indices, outside the body's own binders. Expanding a use
//! instantiates those indices with the argument terms, each shifted past the
//! body's binders, so an argument that mentions a use-site binder keeps
//! pointing at that binder (capture-free, obligation E2). Definitions add no
//! axiom, rule or symbol: an expanded formula is an ordinary kernel formula.

use crate::proof::{prop, Scope};
use crate::sexpr::{parse_all, Sexp};
use rlt_kernel::prop::{MAX_PROP_DEPTH, MAX_PROP_NODES};
use rlt_kernel::{Formula, Prop, Term};
use std::rc::Rc;

/// How a definition is written in NAT-DEF notation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Notation {
    /// `a SYM b`, at the precedence of `=`.
    Infix(String),
    /// `NAME(a, …)`.
    Prefix(String),
}

#[derive(Clone, Debug)]
pub struct Definition {
    pub name: String,
    pub params: Vec<String>,
    /// The body with parameter i at index (binders above it) + (n − 1 − i).
    pub body: Prop,
    pub notation: Notation,
    /// A registered source id, or `None` with `origin` set.
    pub source: Option<String>,
    pub origin: Option<String>,
    /// Definitions this body uses, directly or through others.
    pub deps: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct Definitions {
    pub list: Vec<Definition>,
    /// Names expanded so far (with their dependencies), for the report's
    /// `definitions` line. Recording only; never consulted by a check.
    pub used: std::cell::RefCell<std::collections::BTreeSet<String>>,
}

impl Definitions {
    pub fn get(&self, name: &str) -> Option<&Definition> {
        self.list.iter().find(|d| d.name == name)
    }
    pub fn by_notation(&self, n: &Notation) -> Option<&Definition> {
        self.list.iter().find(|d| &d.notation == n)
    }
    pub fn names(&self) -> Vec<&str> {
        self.list.iter().map(|d| d.name.as_str()).collect()
    }
    /// Record that `name` was expanded, with its dependencies.
    pub fn mark(&self, name: &str) {
        if let Some(d) = self.get(name) {
            let mut u = self.used.borrow_mut();
            u.insert(d.name.clone());
            u.extend(d.deps.iter().cloned());
        }
    }
}

fn is_ident(s: &str) -> bool {
    let mut cs = s.chars();
    matches!(cs.next(), Some(c) if c.is_ascii_lowercase())
        && cs.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Names a definition may not take: the kernel's rule names and the proof
/// syntax's formula heads.
const RESERVED: [&str; 9] = [
    "and", "or", "imp", "not", "forall", "exists", "false", "at", "params",
];

/// Parse a definitions file. `forbidden` lists further names a definition
/// may not take (rule and axiom names).
pub fn parse_definitions(src: &str, forbidden: &[&str]) -> Result<Definitions, String> {
    let mut defs = Definitions::default();
    for e in parse_all(src)? {
        let Some(("define", rest)) = e.head() else {
            return Err(format!("definitions: expected (define …), found {e:?}"));
        };
        let [Sexp::Sym(name), Sexp::List(ps), body, extra @ ..] = rest else {
            return Err(format!(
                "definitions: expected (define NAME (params…) BODY (notation …) (source ID)), found {e:?}"
            ));
        };
        let ctx = format!("definition `{name}`");
        if !is_ident(name) {
            return Err(format!("{ctx}: the name must be a lower-case identifier"));
        }
        if RESERVED.contains(&name.as_str()) || forbidden.contains(&name.as_str()) {
            return Err(format!(
                "{ctx}: the name collides with a rule, axiom or syntax name"
            ));
        }
        if defs.get(name).is_some() {
            return Err(format!("{ctx}: defined twice"));
        }
        let mut params = Vec::new();
        for p in ps {
            match p {
                Sexp::Sym(s) if is_ident(s) && !params.contains(s) => params.push(s.clone()),
                other => {
                    return Err(format!(
                        "{ctx}: parameters are distinct lower-case identifiers, found {other:?}"
                    ))
                }
            }
        }
        if params.is_empty() {
            return Err(format!("{ctx}: at least one parameter"));
        }
        let (mut notation, mut source, mut origin) = (None, None, None);
        for x in extra {
            match x.head() {
                Some(("notation", [Sexp::Sym(k), Sexp::Str(sym)])) if notation.is_none() => {
                    notation = Some(match k.as_str() {
                        "infix" if params.len() == 2 => Notation::Infix(sym.clone()),
                        "prefix" => Notation::Prefix(sym.clone()),
                        _ => {
                            return Err(format!(
                                "{ctx}: notation is (notation infix \"SYM\") for two parameters or (notation prefix \"NAME\")"
                            ))
                        }
                    })
                }
                Some(("source", [Sexp::Sym(s)])) if source.is_none() && origin.is_none() => {
                    source = Some(s.clone())
                }
                Some(("origin", [Sexp::Sym(o)]))
                    if o == "rlt-authored" && source.is_none() && origin.is_none() =>
                {
                    origin = Some(o.clone())
                }
                _ => return Err(format!("{ctx}: unexpected or repeated field {x:?}")),
            }
        }
        let notation = notation.ok_or(format!("{ctx}: (notation …) is required"))?;
        if source.is_none() && origin.is_none() {
            return Err(format!(
                "{ctx}: cite (source ID) or declare (origin rlt-authored)"
            ));
        }
        if defs.by_notation(&notation).is_some() {
            return Err(format!("{ctx}: notation {notation:?} is used twice"));
        }
        // The parameters are the outermost binders of the body; only earlier
        // definitions are in scope, so recursion and cycles are refused here.
        let earlier = Rc::new(Definitions {
            list: defs.list.clone(),
            used: Default::default(),
        });
        let sc = Scope {
            binders: params.clone(),
            defs: Some(earlier.clone()),
            ..Scope::default()
        };
        let body = prop(body, &sc).map_err(|m| format!("{ctx}: {m}"))?;
        let deps: Vec<String> = earlier.used.borrow().iter().cloned().collect();
        body.check_wf(params.len())
            .map_err(|m| format!("{ctx}: {m}"))?;
        defs.list.push(Definition {
            name: name.clone(),
            params,
            body,
            notation,
            source,
            origin,
            deps,
        });
    }
    Ok(defs)
}

fn shift(t: &Term, by: usize) -> Term {
    match t {
        Term::Bound(j) => Term::Bound(j + by),
        Term::App(f, a) => Term::App(f.clone(), a.iter().map(|x| shift(x, by)).collect()),
        other => other.clone(),
    }
}

fn inst_term(t: &Term, k: usize, args: &[Term]) -> Term {
    let n = args.len();
    match t {
        Term::Bound(b) if *b < k => Term::Bound(*b),
        Term::Bound(b) if *b - k < n => shift(&args[n - 1 - (*b - k)], k),
        Term::Bound(b) => Term::Bound(*b - n),
        Term::App(f, a) => Term::App(f.clone(), a.iter().map(|x| inst_term(x, k, args)).collect()),
        other => other.clone(),
    }
}

fn inst_prop(p: &Prop, k: usize, args: &[Term]) -> Prop {
    let b = |x: &Prop, k| Box::new(inst_prop(x, k, args));
    match p {
        Prop::Eq(f) => Prop::Eq(Formula::new(
            inst_term(&f.lhs, k, args),
            inst_term(&f.rhs, k, args),
        )),
        Prop::False => Prop::False,
        Prop::And(x, y) => Prop::And(b(x, k), b(y, k)),
        Prop::Or(x, y) => Prop::Or(b(x, k), b(y, k)),
        Prop::Imp(x, y) => Prop::Imp(b(x, k), b(y, k)),
        Prop::Forall(h, x) => Prop::Forall(h.clone(), b(x, k + 1)),
        Prop::Exists(h, x) => Prop::Exists(h.clone(), b(x, k + 1)),
    }
}

fn depth(p: &Prop) -> usize {
    match p {
        Prop::Eq(_) | Prop::False => 0,
        Prop::And(a, b) | Prop::Or(a, b) | Prop::Imp(a, b) => 1 + depth(a).max(depth(b)),
        Prop::Forall(_, b) | Prop::Exists(_, b) => 1 + depth(b),
    }
}

impl Definition {
    /// Expand a use with these argument terms (use-site indices allowed).
    pub fn expand(&self, args: &[Term]) -> Result<Prop, String> {
        if args.len() != self.params.len() {
            return Err(format!(
                "`{}` takes {} argument(s), given {}",
                self.name,
                self.params.len(),
                args.len()
            ));
        }
        // Size bound before building (obligation O8 of M3a, E1 of M3b).
        let occurrences = self.body.size();
        let widest = args.iter().map(|a| a.size()).max().unwrap_or(1);
        if occurrences.saturating_mul(widest) > MAX_PROP_NODES {
            return Err(format!(
                "expanding `{}` would exceed {MAX_PROP_NODES} nodes",
                self.name
            ));
        }
        let out = inst_prop(&self.body, 0, args);
        if depth(&out) > MAX_PROP_DEPTH {
            return Err(format!(
                "expanding `{}` nests deeper than {MAX_PROP_DEPTH}",
                self.name
            ));
        }
        Ok(out)
    }
    /// The folded name to show in reports: the definition's notation.
    pub fn shown(&self) -> String {
        match &self.notation {
            Notation::Infix(s) | Notation::Prefix(s) => s.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFS: &str = r#"
(define lt (x y) (exists z (= (+ (S z) x) y)) (notation infix "<") (source OLP-ROBINSON-Q))
(define dvd (d n) (exists k (= n (* d k))) (notation infix "∣") (source LEAN-CORE-NAT-DVD))
(define free235 (r) (and (and (not (dvd 2 r)) (not (dvd 3 r))) (not (dvd 5 r))) (notation prefix "free235") (origin rlt-authored))
"#;

    #[test]
    fn definitions_parse_and_expand() {
        let d = parse_definitions(DEFS, &[]).unwrap();
        let e = d
            .get("dvd")
            .unwrap()
            .expand(&[Term::numeral(2), Term::numeral(8)])
            .unwrap();
        assert_eq!(e.to_string(), "∃k. 8 = 2 × k");
        let e = d
            .get("lt")
            .unwrap()
            .expand(&[Term::numeral(7), Term::numeral(30)])
            .unwrap();
        assert_eq!(e.to_string(), "∃z. S(z) + 7 = 30");
    }

    #[test]
    fn a_use_site_binder_is_never_captured() {
        // ∀z. z < 30: the argument is the use site's Bound(0); inside lt's own
        // ∃z it must become Bound(1).
        let d = parse_definitions(DEFS, &[]).unwrap();
        let e = d
            .get("lt")
            .unwrap()
            .expand(&[Term::Bound(0), Term::numeral(30)])
            .unwrap();
        let all = Prop::Forall("z".into(), Box::new(e));
        assert!(all.check_wf(0).is_ok());
        assert_eq!(all.to_string(), "∀z. ∃z′. S(z′) + z = 30");
    }

    #[test]
    fn nested_definitions_expand_through_their_parameters() {
        let d = parse_definitions(DEFS, &[]).unwrap();
        let e = d
            .get("free235")
            .unwrap()
            .expand(&[Term::numeral(7)])
            .unwrap();
        assert_eq!(
            e.to_string(),
            "(¬(∃k. 7 = 2 × k) ∧ ¬(∃k. 7 = 3 × k)) ∧ ¬(∃k. 7 = 5 × k)"
        );
    }

    #[test]
    fn recursion_cycles_names_and_arity_are_refused() {
        let rec = r#"(define f (x) (f x) (notation prefix "f") (origin rlt-authored))"#;
        assert!(parse_definitions(rec, &[]).is_err());
        let cyc = r#"(define a (x) (b x) (notation prefix "a") (origin rlt-authored))
(define b (x) (a x) (notation prefix "b") (origin rlt-authored))"#;
        assert!(parse_definitions(cyc, &[]).is_err());
        let rule = r#"(define refl (x) (= x x) (notation prefix "r") (origin rlt-authored))"#;
        assert!(parse_definitions(rule, &["refl"]).is_err());
        let d = parse_definitions(DEFS, &[]).unwrap();
        assert!(d.get("dvd").unwrap().expand(&[Term::numeral(2)]).is_err());
        let free = r#"(define g (x) (= x y) (notation prefix "g") (origin rlt-authored))"#;
        assert!(parse_definitions(free, &[]).is_err());
    }
}

//! Registered function definitions (RLT-PROVER-002): term abbreviations
//! unfolded before the kernel.
//!
//! A definitions file registers functions on the natural numbers, each by an
//! exact defining expression over the kernel signature {0, S, +, ×}:
//!
//! ```text
//! (rlt-definitions
//!   (id ARITH-DOUBLE)
//!   (convention NAT-LOGIC)
//!   (domain Nat)
//!   (define-function double (x) (+ x x)
//!     (written "double(x) := x + x")
//!     (origin rlt-authored)))
//! ```
//!
//! Semantics. `double(t)` abbreviates the body with each parameter replaced
//! by the argument term. A statement written with registered functions means
//! exactly its unfolding; the kernel checks only the unfolded formula. A
//! definition adds no axiom, no rule and no kernel symbol.
//!
//! Checks made when a file is read (all fail closed):
//! - the file holds only `id`, `convention`, `domain` and `define-function`
//!   forms; an `(axiom …)` or a reading-point, relationship or context label
//!   is refused;
//! - names are new: not a syntax word, rule, axiom, pinned M3b definition or
//!   earlier definition; parameters are distinct one-letter variables;
//! - a body uses only its parameters, numerals, `S`, `+`, `*` and functions
//!   registered above it, each at its arity; recursion and cycles are
//!   therefore undefined symbols;
//! - the written defining expression elaborates (under NAT-LOGIC, with the
//!   earlier functions) to exactly the formal body.
//!
//! Bodies are stored fully unfolded (over the signature only), so one
//! substitution unfolds a use. Bodies contain no binders, so substituting an
//! argument that mentions a use-site bound variable cannot capture it.

use crate::sexpr::{parse_all, Sexp};
use crate::written::Written;
use rlt_kernel::prop::{MAX_PROP_DEPTH, MAX_PROP_NODES};
use rlt_kernel::{Formula, Prop, Term};
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

/// The convention whose files read registered function definitions.
pub const FUNCTIONS_CONVENTION: &str = "NAT-FUN";
/// The convention a definitions file's bodies and written forms are read in.
pub const BASE_CONVENTION: &str = "NAT-LOGIC";
/// The only domain in this milestone.
pub const DOMAIN: &str = "Nat";
/// Largest body, in term nodes, a definition may unfold to.
pub const MAX_BODY_NODES: u64 = 100_000;

#[derive(Clone, Debug)]
pub struct FunctionDef {
    pub name: String,
    /// One-letter parameters, in order.
    pub params: Vec<String>,
    /// The body over `Var(param)` and the signature {0, S, +, *} only
    /// (earlier functions already unfolded).
    pub body: Term,
    /// The written defining expression, verbatim (`double(x) := x + x`).
    pub written: String,
    /// The formal body as written in the file (before unfolding earlier
    /// functions), in proof syntax.
    pub formal: String,
    pub source: Option<String>,
    pub origin: Option<String>,
    /// Functions this body uses, directly or through others, in file order.
    pub deps: Vec<String>,
    /// Content identity (SHA-256 of `canonical()` plus the dependencies'
    /// identities); filled in by the loader, which owns the hash function.
    pub content_sha256: String,
}

#[derive(Clone, Debug, Default)]
pub struct FunctionDefs {
    pub id: String,
    pub convention: String,
    pub domain: String,
    pub list: Vec<FunctionDef>,
    /// Repository-relative path and SHA-256 of the file's bytes; filled in
    /// by the loader.
    pub file: String,
    pub file_sha256: String,
    /// Names unfolded so far (with dependencies). Recording only; never
    /// consulted by a check.
    pub used: RefCell<BTreeSet<String>>,
}

/// Names a function may not take: proof-syntax heads and rule words.
const RESERVED: [&str; 12] = [
    "and", "or", "imp", "not", "forall", "exists", "false", "at", "params", "define", "steps",
    "goal",
];

/// A function name: two or more lower-case ASCII letters (one token when
/// written, and never a one-letter variable).
pub fn is_function_name(s: &str) -> bool {
    s.len() >= 2 && s.chars().all(|c| c.is_ascii_lowercase())
}

fn is_param(s: &str) -> bool {
    s.len() == 1 && s.chars().all(|c| c.is_ascii_lowercase())
}

fn is_registry_id(s: &str) -> bool {
    !s.is_empty()
        && s.chars().next().is_some_and(|c| c.is_ascii_uppercase())
        && s.chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-')
}

impl FunctionDefs {
    pub fn get(&self, name: &str) -> Option<&FunctionDef> {
        self.list.iter().find(|d| d.name == name)
    }
    pub fn mark(&self, name: &str) {
        if let Some(d) = self.get(name) {
            let mut u = self.used.borrow_mut();
            u.insert(d.name.clone());
            u.extend(d.deps.iter().cloned());
        }
    }
    /// Names used so far, in file order.
    pub fn used_in_order(&self) -> Vec<String> {
        let u = self.used.borrow();
        self.list
            .iter()
            .filter(|d| u.contains(&d.name))
            .map(|d| d.name.clone())
            .collect()
    }

    /// Unfold every use of a registered function in a term, innermost first.
    /// Symbols that are not registered functions are left as they are (the
    /// kernel's sort check judges them).
    pub fn unfold_term(&self, t: &Term) -> Result<Term, String> {
        match t {
            Term::App(f, args) => {
                let a = args
                    .iter()
                    .map(|x| self.unfold_term(x))
                    .collect::<Result<Vec<_>, _>>()?;
                match self.get(f) {
                    Some(d) => {
                        self.mark(f);
                        d.instantiate(&a)
                    }
                    None => Ok(Term::App(f.clone(), a)),
                }
            }
            other => Ok(other.clone()),
        }
    }

    /// Unfold every use in a formula; connectives and binders are kept.
    pub fn unfold_prop(&self, p: &Prop) -> Result<Prop, String> {
        let out = self.unfold_in(p)?;
        if prop_nodes(&out) > MAX_PROP_NODES {
            return Err(format!(
                "NAT-FUN: the unfolded formula exceeds {MAX_PROP_NODES} nodes"
            ));
        }
        if prop_depth(&out) > MAX_PROP_DEPTH {
            return Err(format!(
                "NAT-FUN: the unfolded formula nests deeper than {MAX_PROP_DEPTH}"
            ));
        }
        Ok(out)
    }

    fn unfold_in(&self, p: &Prop) -> Result<Prop, String> {
        let b = |x: &Prop| -> Result<Box<Prop>, String> { Ok(Box::new(self.unfold_in(x)?)) };
        Ok(match p {
            Prop::Eq(f) => Prop::Eq(Formula::new(
                self.unfold_term(&f.lhs)?,
                self.unfold_term(&f.rhs)?,
            )),
            Prop::False => Prop::False,
            Prop::And(x, y) => Prop::And(b(x)?, b(y)?),
            Prop::Or(x, y) => Prop::Or(b(x)?, b(y)?),
            Prop::Imp(x, y) => Prop::Imp(b(x)?, b(y)?),
            Prop::Forall(h, x) => Prop::Forall(h.clone(), b(x)?),
            Prop::Exists(h, x) => Prop::Exists(h.clone(), b(x)?),
        })
    }
}

fn prop_nodes(p: &Prop) -> u64 {
    match p {
        Prop::Eq(f) => f.lhs.size().saturating_add(f.rhs.size()),
        Prop::False => 1,
        Prop::And(a, b) | Prop::Or(a, b) | Prop::Imp(a, b) => 1u64
            .saturating_add(prop_nodes(a))
            .saturating_add(prop_nodes(b)),
        Prop::Forall(_, b) | Prop::Exists(_, b) => 1u64.saturating_add(prop_nodes(b)),
    }
}

fn prop_depth(p: &Prop) -> usize {
    match p {
        Prop::Eq(_) | Prop::False => 0,
        Prop::And(a, b) | Prop::Or(a, b) | Prop::Imp(a, b) => 1 + prop_depth(a).max(prop_depth(b)),
        Prop::Forall(_, b) | Prop::Exists(_, b) => 1 + prop_depth(b),
    }
}

/// Replace `Var(p)` by the argument for parameter p. The body has no
/// binders, so an argument's bound indices keep their meaning unshifted.
fn subst_params(t: &Term, params: &[String], args: &[Term]) -> Term {
    match t {
        Term::Var(v) => match params.iter().position(|p| p == v) {
            Some(i) => args[i].clone(),
            None => t.clone(),
        },
        Term::App(f, a) => Term::App(
            f.clone(),
            a.iter().map(|x| subst_params(x, params, args)).collect(),
        ),
        other => other.clone(),
    }
}

fn occurrences(t: &Term) -> u64 {
    match t {
        Term::Var(_) => 1,
        Term::App(_, a) => a.iter().fold(0u64, |s, x| s.saturating_add(occurrences(x))),
        _ => 0,
    }
}

impl FunctionDef {
    /// The body with the parameters replaced by these argument terms.
    pub fn instantiate(&self, args: &[Term]) -> Result<Term, String> {
        if args.len() != self.params.len() {
            return Err(format!(
                "NAT-FUN: `{}` takes {} argument(s), given {}",
                self.name,
                self.params.len(),
                args.len()
            ));
        }
        let widest = args.iter().map(|a| a.size()).max().unwrap_or(1);
        let bound = self
            .body
            .size()
            .saturating_add(occurrences(&self.body).saturating_mul(widest));
        if bound > MAX_PROP_NODES {
            return Err(format!(
                "NAT-FUN: unfolding `{}` would exceed {MAX_PROP_NODES} nodes",
                self.name
            ));
        }
        Ok(subst_params(&self.body, &self.params, args))
    }

    /// The left side as written: `double(x)`.
    pub fn head(&self) -> String {
        format!("{}({})", self.name, self.params.join(", "))
    }

    /// The canonical text the content identity is computed from: every field
    /// that decides or documents the meaning. The loader appends the
    /// dependencies' identities.
    pub fn canonical(&self, convention: &str, domain: &str) -> String {
        let provenance = match (&self.source, &self.origin) {
            (Some(s), _) => format!("source {s}"),
            (None, Some(o)) => format!("origin {o}"),
            (None, None) => "none".to_string(),
        };
        format!(
            "rlt-function-definition/1\nname {}\nparams {}\nbody {}\nformal {}\nwritten {}\nconvention {convention}\ndomain {domain}\nprovenance {provenance}\n",
            self.name,
            self.params.join(" "),
            self.body.kernel_form(),
            self.formal,
            self.written,
        )
    }
}

fn sexp_text(e: &Sexp) -> String {
    match e {
        Sexp::Sym(s) => s.clone(),
        Sexp::Int(n) => n.to_string(),
        Sexp::Str(s) => format!("{s:?}"),
        Sexp::List(v) => format!(
            "({})",
            v.iter().map(sexp_text).collect::<Vec<_>>().join(" ")
        ),
    }
}

/// Read a formal body: parameters, numerals, `(S t)`, `(+ a b)`, `(* a b)`
/// and uses of earlier functions (unfolded here).
fn body_term(
    e: &Sexp,
    ctx: &str,
    me: &str,
    params: &[String],
    earlier: &FunctionDefs,
    deps: &mut BTreeSet<String>,
) -> Result<Term, String> {
    match e {
        Sexp::Int(n) if *n > crate::proof::MAX_NUMERAL => Err(format!(
            "{ctx}: numeral {n} exceeds the unary limit {}",
            crate::proof::MAX_NUMERAL
        )),
        Sexp::Int(n) => Ok(Term::numeral(*n)),
        Sexp::Sym(s) if params.contains(s) => Ok(Term::Var(s.clone())),
        Sexp::Sym(s) => Err(format!(
            "{ctx}: unbound variable or undefined constant `{s}`: a body may use only its parameters, numerals, S, +, * and functions registered above it"
        )),
        Sexp::List(items) => {
            let Some((Sexp::Sym(f), args)) = items.split_first() else {
                return Err(format!("{ctx}: expected (symbol args…), found {}", sexp_text(e)));
            };
            let ts = args
                .iter()
                .map(|a| body_term(a, ctx, me, params, earlier, deps))
                .collect::<Result<Vec<_>, _>>()?;
            match (f.as_str(), ts.len()) {
                ("S", 1) | ("+", 2) | ("*", 2) => Ok(Term::App(f.clone(), ts)),
                ("S" | "+" | "*", n) => Err(format!(
                    "{ctx}: `{f}` has arity {}, applied to {n} argument(s)",
                    if f == "S" { 1 } else { 2 }
                )),
                _ if f == me => Err(format!(
                    "{ctx}: `{f}` is used in its own body; recursive and cyclic definitions are refused (a body may use only functions registered above it)"
                )),
                _ => match earlier.get(f) {
                    Some(d) => {
                        deps.insert(d.name.clone());
                        deps.extend(d.deps.iter().cloned());
                        d.instantiate(&ts).map_err(|m| format!("{ctx}: {m}"))
                    }
                    None => Err(format!(
                        "{ctx}: undefined symbol `{f}`: a body may use only S, +, * and functions registered above it (recursive and cyclic definitions are refused)"
                    )),
                },
            }
        }
        Sexp::Str(s) => Err(format!("{ctx}: found string {s:?} where a term was expected")),
    }
}

/// Check that the written defining expression `NAME(p, …) := RHS` names
/// this function and parameters, and that RHS elaborates to the body.
fn check_written(def: &FunctionDef, ctx: &str, earlier: &Rc<FunctionDefs>) -> Result<(), String> {
    let w = Written::new(&def.written);
    let toks = &w.tokens;
    let mut head = vec![def.name.clone(), "(".to_string()];
    for (i, p) in def.params.iter().enumerate() {
        if i > 0 {
            head.push(",".into());
        }
        head.push(p.clone());
    }
    head.extend([")".to_string(), ":".to_string(), "=".to_string()]);
    if toks.len() <= head.len() || toks[..head.len()] != head[..] {
        return Err(format!(
            "{ctx}: the written definition must read `{} := …`, found {:?}",
            def.head(),
            def.written
        ));
    }
    let folded =
        crate::elab::elaborate_definition_rhs(&toks[head.len()..], &def.params, earlier.clone())
            .map_err(|m| format!("{ctx}: written definition: {m}"))?;
    let unfolded = earlier
        .unfold_term(&folded)
        .map_err(|m| format!("{ctx}: written definition: {m}"))?;
    // The elaborator reads the parameters as bound indices (innermost last).
    let n = def.params.len();
    fn to_vars(t: &Term, params: &[String], n: usize) -> Term {
        match t {
            Term::Bound(i) if *i < n => Term::Var(params[n - 1 - i].clone()),
            Term::App(f, a) => {
                Term::App(f.clone(), a.iter().map(|x| to_vars(x, params, n)).collect())
            }
            other => other.clone(),
        }
    }
    let as_vars = to_vars(&unfolded, &def.params, n);
    if as_vars != def.body {
        return Err(format!(
            "{ctx}: the written definition {:?} reads as {}, but the formal body is {}",
            def.written,
            as_vars.kernel_form(),
            def.body.kernel_form()
        ));
    }
    Ok(())
}

/// Parse a definitions file. `forbidden` lists further names a function may
/// not take (rule names, axiom names, the pinned M3b definitions).
pub fn parse_function_definitions(src: &str, forbidden: &[&str]) -> Result<FunctionDefs, String> {
    let top = parse_all(src)?;
    let [root] = top.as_slice() else {
        return Err(
            "definitions: a definitions file holds exactly one (rlt-definitions …) form".into(),
        );
    };
    let Some(("rlt-definitions", fields)) = root.head() else {
        return Err("definitions: expected (rlt-definitions …)".into());
    };
    let mut defs = FunctionDefs::default();
    let (mut id, mut conv, mut dom) = (None, None, None);
    for f in fields {
        let Some((key, rest)) = f.head() else {
            return Err(format!(
                "definitions: expected (key …), found {}",
                sexp_text(f)
            ));
        };
        let one = |rest: &[Sexp]| -> Result<String, String> {
            match rest {
                [Sexp::Sym(s)] => Ok(s.clone()),
                _ => Err(format!("definitions: ({key} …) takes one symbol")),
            }
        };
        match key {
            "id" if id.is_none() => {
                let s = one(rest)?;
                if !is_registry_id(&s) {
                    return Err(format!(
                        "definitions: registry id `{s}`: upper-case letters, digits and `-`"
                    ));
                }
                id = Some(s)
            }
            "convention" if conv.is_none() => {
                let s = one(rest)?;
                if s != BASE_CONVENTION {
                    return Err(format!(
                        "definitions: convention `{s}`: bodies and written forms are read in {BASE_CONVENTION}, the only base convention registered for definitions"
                    ));
                }
                conv = Some(s)
            }
            "domain" if dom.is_none() => {
                let s = one(rest)?;
                if s != DOMAIN {
                    return Err(format!(
                        "definitions: domain `{s}`: the only registered domain is {DOMAIN} (the natural numbers of the kernel signature)"
                    ));
                }
                dom = Some(s)
            }
            "id" | "convention" | "domain" => {
                return Err(format!("definitions: ({key} …) appears twice"))
            }
            "define-function" => {
                if id.is_none() || conv.is_none() || dom.is_none() {
                    return Err("definitions: (id …), (convention …) and (domain …) come before the first (define-function …)".into());
                }
                let d = parse_one(rest, &defs, forbidden)?;
                defs.list.push(d);
            }
            "axiom" => {
                return Err("definitions: (axiom …) is refused: a definitions file registers abbreviations only and adds no axiom (axioms are registered, pinned sets in spec/axioms.yaml)".into())
            }
            "readingpoint" | "relationship" | "context" => {
                return Err(format!(
                    "definitions: ({key} …) is refused: a mathematical definition carries no reading-point, relationship or context label"
                ))
            }
            other => return Err(format!("definitions: unknown field `{other}`")),
        }
    }
    defs.id = id.ok_or("definitions: missing (id …)")?;
    defs.convention = conv.ok_or("definitions: missing (convention …)")?;
    defs.domain = dom.ok_or("definitions: missing (domain …)")?;
    if defs.list.is_empty() {
        return Err("definitions: register at least one (define-function …)".into());
    }
    Ok(defs)
}

fn parse_one(
    rest: &[Sexp],
    defs: &FunctionDefs,
    forbidden: &[&str],
) -> Result<FunctionDef, String> {
    let [Sexp::Sym(name), Sexp::List(ps), body, extra @ ..] = rest else {
        return Err("definitions: expected (define-function NAME (PARAMS…) BODY (written \"…\") (source ID) | (origin rlt-authored))".into());
    };
    let ctx = format!("definition `{name}`");
    if !is_function_name(name) {
        return Err(format!(
            "{ctx}: a function name is two or more lower-case ASCII letters"
        ));
    }
    if RESERVED.contains(&name.as_str()) || forbidden.contains(&name.as_str()) {
        return Err(format!(
            "{ctx}: the name collides with a syntax word, rule, axiom or registered definition"
        ));
    }
    if defs.get(name).is_some() {
        return Err(format!(
            "{ctx}: defined twice; a name has one registered definition (conflicting definitions are refused)"
        ));
    }
    let mut params = Vec::new();
    for p in ps {
        match p {
            Sexp::Sym(s) if is_param(s) && !params.contains(s) => params.push(s.clone()),
            other => {
                return Err(format!(
                    "{ctx}: parameters are distinct one-letter lower-case variables, found {}",
                    sexp_text(other)
                ))
            }
        }
    }
    if params.is_empty() {
        return Err(format!("{ctx}: at least one parameter"));
    }
    let (mut written, mut source, mut origin) = (None, None, None);
    for x in extra {
        match x.head() {
            Some(("written", [Sexp::Str(w)])) if written.is_none() => written = Some(w.clone()),
            Some(("source", [Sexp::Sym(s)])) if source.is_none() && origin.is_none() => {
                source = Some(s.clone())
            }
            Some(("origin", [Sexp::Sym(o)]))
                if o == "rlt-authored" && source.is_none() && origin.is_none() =>
            {
                origin = Some(o.clone())
            }
            Some(("axiom" | "assume" | "readingpoint" | "relationship" | "context", _)) => {
                return Err(format!(
                    "{ctx}: {} is refused: a definition is an abbreviation and asserts nothing",
                    sexp_text(x)
                ))
            }
            _ => {
                return Err(format!(
                    "{ctx}: unexpected or repeated field {}",
                    sexp_text(x)
                ))
            }
        }
    }
    let written = written.ok_or(format!(
        "{ctx}: (written \"NAME(params) := …\") is required"
    ))?;
    if source.is_none() && origin.is_none() {
        return Err(format!(
            "{ctx}: cite (source ID) or declare (origin rlt-authored)"
        ));
    }
    let mut deps = BTreeSet::new();
    let t = body_term(body, &ctx, name, &params, defs, &mut deps)?;
    if t.size() > MAX_BODY_NODES {
        return Err(format!(
            "{ctx}: the unfolded body exceeds {MAX_BODY_NODES} nodes"
        ));
    }
    let def = FunctionDef {
        name: name.clone(),
        params,
        body: t,
        written,
        formal: sexp_text(body),
        source,
        origin,
        deps: defs
            .list
            .iter()
            .filter(|d| deps.contains(&d.name))
            .map(|d| d.name.clone())
            .collect(),
        content_sha256: String::new(),
    };
    let earlier = Rc::new(FunctionDefs {
        id: defs.id.clone(),
        convention: defs.convention.clone(),
        domain: defs.domain.clone(),
        list: defs.list.clone(),
        ..FunctionDefs::default()
    });
    check_written(&def, &ctx, &earlier)?;
    Ok(def)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE: &str = r#"
(rlt-definitions
  (id ARITH-DOUBLE)
  (convention NAT-LOGIC)
  (domain Nat)
  (define-function double (x) (+ x x)
    (written "double(x) := x + x")
    (origin rlt-authored))
  (define-function quadruple (x) (double (double x))
    (written "quadruple(x) := double(double(x))")
    (origin rlt-authored)))
"#;

    fn file(defs: &str) -> String {
        format!("(rlt-definitions (id T) (convention NAT-LOGIC) (domain Nat) {defs})")
    }

    #[test]
    fn functions_parse_unfold_and_record_dependencies() {
        let d = parse_function_definitions(FILE, &[]).unwrap();
        assert_eq!(d.get("double").unwrap().body.to_string(), "x + x");
        let q = d.get("quadruple").unwrap();
        assert_eq!(q.body.to_string(), "(x + x) + (x + x)");
        assert_eq!(q.deps, vec!["double".to_string()]);
        let t = Term::App("double".into(), vec![Term::numeral(2)]);
        assert_eq!(d.unfold_term(&t).unwrap().to_string(), "2 + 2");
    }

    #[test]
    fn unfolding_under_binders_keeps_bound_indices() {
        // ∀x. ∀y. double(y) = double(x): y is Bound(0), x is Bound(1).
        let d = parse_function_definitions(FILE, &[]).unwrap();
        let p = Prop::Forall(
            "x".into(),
            Box::new(Prop::Forall(
                "y".into(),
                Box::new(Prop::eq(
                    Term::App("double".into(), vec![Term::Bound(0)]),
                    Term::App("double".into(), vec![Term::Bound(1)]),
                )),
            )),
        );
        let u = d.unfold_prop(&p).unwrap();
        assert!(u.check_wf(0).is_ok());
        assert_eq!(u.to_string(), "∀x. ∀y. y + y = x + x");
    }

    #[test]
    fn malformed_definitions_are_refused() {
        let cases = [
            // recursion
            r#"(define-function f (x) (f x) (written "f(x) := f(x)") (origin rlt-authored))"#,
            r#"(define-function loop (x) (loop x) (written "loop(x) := loop(x)") (origin rlt-authored))"#,
            // cycle: a uses b, defined later
            r#"(define-function aa (x) (bb x) (written "aa(x) := bb(x)") (origin rlt-authored))
               (define-function bb (x) (aa x) (written "bb(x) := aa(x)") (origin rlt-authored))"#,
            // unbound variable
            r#"(define-function shift (x) (+ x y) (written "shift(x) := x + y") (origin rlt-authored))"#,
            // arity
            r#"(define-function two (x) (+ x) (written "two(x) := x") (origin rlt-authored))"#,
            // undefined symbol
            r#"(define-function half (x) (div x 2) (written "half(x) := x") (origin rlt-authored))"#,
            // written form disagrees with the body
            r#"(define-function double (x) (+ x x) (written "double(x) := 2 × x") (origin rlt-authored))"#,
            // missing provenance
            r#"(define-function double (x) (+ x x) (written "double(x) := x + x"))"#,
            // axiom injection inside a definition
            r#"(define-function double (x) (+ x x) (written "double(x) := x + x") (origin rlt-authored) (axiom d (x) (= (double x) 0)))"#,
        ];
        for c in cases {
            assert!(parse_function_definitions(&file(c), &[]).is_err(), "{c}");
        }
        let twice = r#"(define-function double (x) (+ x x) (written "double(x) := x + x") (origin rlt-authored))
                       (define-function double (x) (* 2 x) (written "double(x) := 2 × x") (origin rlt-authored))"#;
        let e = parse_function_definitions(&file(twice), &[]).unwrap_err();
        assert!(e.contains("defined twice"), "{e}");
        let ax = "(axiom double_def (x) (= (+ x x) (* 2 x)) (source X))";
        let e = parse_function_definitions(&file(ax), &[]).unwrap_err();
        assert!(e.contains("adds no axiom"), "{e}");
        let ok =
            r#"(define-function lt (x) (+ x x) (written "lt(x) := x + x") (origin rlt-authored))"#;
        assert!(parse_function_definitions(&file(ok), &["lt"]).is_err());
    }
}

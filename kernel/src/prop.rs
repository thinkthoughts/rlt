//! Formulas (RLT-003 M3a): equations, ⊥, ∧, ∨, →, ∀ and ∃, locally nameless.
//!
//! A bound variable is a de Bruijn index (`Term::Bound`): index 0 refers to
//! the nearest enclosing binder. Binder names (`hint`) are for printing only
//! and take no part in equality, so alpha-equivalent formulas are equal and
//! the kernel has no renaming step. A formula is *locally closed* when every
//! index refers to a binder inside it; every formula a theorem holds is
//! locally closed (obligation O1).
//!
//! Two operations change variables, and only these two:
//! - `open(body, t)`: the body of a binder with its index replaced by the
//!   locally closed term `t` (forall-elim, exists-intro, subst, exists-elim).
//! - `close(φ, a)`: parameter `a` turned into the index of a new binder
//!   (forall-intro).
//!
//! Terms contain no binders and `t` contains no indices, so `open` never
//! captures.

use crate::term::{arity, Formula, Term};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Clone, Debug)]
pub enum Prop {
    /// An equation: today's `Formula`, byte for byte.
    Eq(Formula),
    False,
    And(Box<Prop>, Box<Prop>),
    Or(Box<Prop>, Box<Prop>),
    Imp(Box<Prop>, Box<Prop>),
    /// `∀hint. body`; the body refers to this binder by `Bound(0)`.
    Forall(String, Box<Prop>),
    Exists(String, Box<Prop>),
}

/// Equality is alpha-equivalence: binder hints are ignored.
impl PartialEq for Prop {
    fn eq(&self, other: &Prop) -> bool {
        use Prop::*;
        match (self, other) {
            (Eq(a), Eq(b)) => a == b,
            (False, False) => true,
            (And(a, b), And(c, d)) | (Or(a, b), Or(c, d)) | (Imp(a, b), Imp(c, d)) => {
                a == c && b == d
            }
            (Forall(_, a), Forall(_, b)) | (Exists(_, a), Exists(_, b)) => a == b,
            _ => false,
        }
    }
}
impl Eq for Prop {}

/// Upper bound on binder and connective nesting in one formula.
pub const MAX_PROP_DEPTH: usize = 256;

/// Upper bound on the nodes of one formula the kernel builds (the proof-file
/// node limit, RLT-003 M3a obligation O8). `open` checks it before building.
pub const MAX_PROP_NODES: u64 = 1_000_000;

fn check_term(t: &Term, depth: usize) -> Result<(), String> {
    match t {
        Term::Var(v) => Err(format!(
            "variable `{v}` outside an axiom: proof terms are ground in RLT-001"
        )),
        Term::Param(_) => Ok(()),
        Term::Bound(i) if *i < depth => Ok(()),
        Term::Bound(i) => Err(format!("bound index #{i} outside its binder")),
        Term::App(f, args) => match arity(f) {
            None => Err(format!(
                "symbol `{f}` is outside the signature {{0, S, +, *}}"
            )),
            Some(n) if n != args.len() => Err(format!(
                "symbol `{f}` has arity {n}, applied to {} argument(s)",
                args.len()
            )),
            Some(_) => args.iter().try_for_each(|a| check_term(a, depth)),
        },
    }
}

impl Prop {
    pub fn eq(lhs: Term, rhs: Term) -> Prop {
        Prop::Eq(Formula::new(lhs, rhs))
    }

    /// The equation, if this formula is one.
    pub fn as_eq(&self) -> Option<&Formula> {
        match self {
            Prop::Eq(f) => Some(f),
            _ => None,
        }
    }

    /// Well-formedness under `depth` enclosing binders: every term well-sorted,
    /// no axiom variable, every index below `depth` (locally closed at 0).
    pub fn check_wf(&self, depth: usize) -> Result<(), String> {
        self.wf(depth, 0)
    }

    fn wf(&self, depth: usize, nest: usize) -> Result<(), String> {
        if nest > MAX_PROP_DEPTH {
            return Err(format!(
                "formula nested deeper than {MAX_PROP_DEPTH} connectives and binders"
            ));
        }
        match self {
            Prop::Eq(f) => {
                check_term(&f.lhs, depth)?;
                check_term(&f.rhs, depth)
            }
            Prop::False => Ok(()),
            Prop::And(a, b) | Prop::Or(a, b) | Prop::Imp(a, b) => {
                a.wf(depth, nest + 1)?;
                b.wf(depth, nest + 1)
            }
            Prop::Forall(_, b) | Prop::Exists(_, b) => b.wf(depth + 1, nest + 1),
        }
    }

    /// Well-formedness of an axiom body (RLT-003 M3b): every term well-sorted,
    /// axiom variables allowed, no parameter, every index bound by a binder of
    /// the body itself, nesting within the limit.
    pub fn check_axiom_body(&self) -> Result<(), String> {
        fn term(t: &Term, depth: usize) -> Result<(), String> {
            match t {
                Term::Var(_) => Ok(()),
                Term::Param(a) => Err(format!("parameter `{a}` in an axiom")),
                Term::Bound(i) if *i < depth => Ok(()),
                Term::Bound(i) => Err(format!("bound index #{i} outside its binder")),
                Term::App(f, args) => match arity(f) {
                    None => Err(format!(
                        "symbol `{f}` is outside the signature {{0, S, +, *}}"
                    )),
                    Some(n) if n != args.len() => Err(format!(
                        "symbol `{f}` has arity {n}, applied to {} argument(s)",
                        args.len()
                    )),
                    Some(_) => args.iter().try_for_each(|a| term(a, depth)),
                },
            }
        }
        fn go(p: &Prop, depth: usize, nest: usize) -> Result<(), String> {
            if nest > MAX_PROP_DEPTH {
                return Err(format!(
                    "formula nested deeper than {MAX_PROP_DEPTH} connectives and binders"
                ));
            }
            match p {
                Prop::Eq(f) => {
                    term(&f.lhs, depth)?;
                    term(&f.rhs, depth)
                }
                Prop::False => Ok(()),
                Prop::And(a, b) | Prop::Or(a, b) | Prop::Imp(a, b) => {
                    go(a, depth, nest + 1)?;
                    go(b, depth, nest + 1)
                }
                Prop::Forall(_, b) | Prop::Exists(_, b) => go(b, depth + 1, nest + 1),
            }
        }
        go(self, 0, 0)
    }

    /// Axiom variable names occurring in this formula, in first-occurrence order.
    pub fn vars(&self, out: &mut Vec<String>) {
        match self {
            Prop::Eq(f) => {
                f.lhs.vars(out);
                f.rhs.vars(out);
            }
            Prop::False => {}
            Prop::And(a, b) | Prop::Or(a, b) | Prop::Imp(a, b) => {
                a.vars(out);
                b.vars(out);
            }
            Prop::Forall(_, b) | Prop::Exists(_, b) => b.vars(out),
        }
    }

    /// Replace axiom variables throughout, under binders too (`inst`). The
    /// replacement terms are locally closed, so they need no index shift.
    pub fn subst_vars(&self, sigma: &BTreeMap<String, Term>) -> Prop {
        self.map_terms(0, &|t, _| t.subst_vars(sigma))
    }

    /// Parameter names occurring in this formula.
    pub fn params(&self, out: &mut BTreeSet<String>) {
        match self {
            Prop::Eq(f) => {
                f.lhs.params(out);
                f.rhs.params(out);
            }
            Prop::False => {}
            Prop::And(a, b) | Prop::Or(a, b) | Prop::Imp(a, b) => {
                a.params(out);
                b.params(out);
            }
            Prop::Forall(_, b) | Prop::Exists(_, b) => b.params(out),
        }
    }

    pub fn has_param(&self, a: &str) -> bool {
        let mut s = BTreeSet::new();
        self.params(&mut s);
        s.contains(a)
    }

    /// Whether this formula has no quantifier and no parameter.
    pub fn is_closed_equation(&self) -> bool {
        matches!(self, Prop::Eq(f) if !f.lhs.has_param_or_bound() && !f.rhs.has_param_or_bound())
    }

    /// Node count: connectives, binders and every term node.
    pub fn size(&self) -> u64 {
        match self {
            Prop::Eq(f) => 1u64
                .saturating_add(f.lhs.size())
                .saturating_add(f.rhs.size()),
            Prop::False => 1,
            Prop::And(a, b) | Prop::Or(a, b) | Prop::Imp(a, b) => {
                1u64.saturating_add(a.size()).saturating_add(b.size())
            }
            Prop::Forall(_, b) | Prop::Exists(_, b) => 1u64.saturating_add(b.size()),
        }
    }

    fn occurrences(&self, k: usize) -> u64 {
        match self {
            Prop::Eq(f) => f
                .lhs
                .bound_occurrences(k)
                .saturating_add(f.rhs.bound_occurrences(k)),
            Prop::False => 0,
            Prop::And(a, b) | Prop::Or(a, b) | Prop::Imp(a, b) => {
                a.occurrences(k).saturating_add(b.occurrences(k))
            }
            Prop::Forall(_, b) | Prop::Exists(_, b) => b.occurrences(k + 1),
        }
    }

    /// Node count of open(self, t), computed without building it.
    pub fn open_size(&self, t: &Term) -> u64 {
        self.size().saturating_add(
            self.occurrences(0)
                .saturating_mul(t.size().saturating_sub(1)),
        )
    }

    /// open, refused before building when the result would exceed
    /// `MAX_PROP_NODES` (O8).
    pub fn open_checked(&self, t: &Term) -> Result<Prop, String> {
        let n = self.open_size(t);
        if n > MAX_PROP_NODES {
            return Err(format!(
                "the instantiated formula would have {n} nodes, above the limit {MAX_PROP_NODES}"
            ));
        }
        Ok(self.open(t))
    }

    /// Whether this formula has a quantifier anywhere.
    pub fn has_quantifier(&self) -> bool {
        match self {
            Prop::Eq(_) | Prop::False => false,
            Prop::And(a, b) | Prop::Or(a, b) | Prop::Imp(a, b) => {
                a.has_quantifier() || b.has_quantifier()
            }
            Prop::Forall(..) | Prop::Exists(..) => true,
        }
    }

    fn map_terms(&self, k: usize, g: &dyn Fn(&Term, usize) -> Term) -> Prop {
        match self {
            Prop::Eq(f) => Prop::Eq(Formula::new(g(&f.lhs, k), g(&f.rhs, k))),
            Prop::False => Prop::False,
            Prop::And(a, b) => Prop::And(Box::new(a.map_terms(k, g)), Box::new(b.map_terms(k, g))),
            Prop::Or(a, b) => Prop::Or(Box::new(a.map_terms(k, g)), Box::new(b.map_terms(k, g))),
            Prop::Imp(a, b) => Prop::Imp(Box::new(a.map_terms(k, g)), Box::new(b.map_terms(k, g))),
            Prop::Forall(h, b) => Prop::Forall(h.clone(), Box::new(b.map_terms(k + 1, g))),
            Prop::Exists(h, b) => Prop::Exists(h.clone(), Box::new(b.map_terms(k + 1, g))),
        }
    }

    /// open(body, t): the binder's index replaced by the locally closed term t.
    pub fn open(&self, t: &Term) -> Prop {
        self.map_terms(0, &|x, k| x.open_at(t, k))
    }

    /// close(φ, a): parameter a turned into the index of a new outer binder.
    pub fn close(&self, a: &str) -> Prop {
        self.map_terms(0, &|x, k| x.close_at(a, k))
    }

    fn render(&self, names: &mut Vec<String>, kernel: bool, top: bool) -> String {
        let mut avoid = BTreeSet::new();
        self.params(&mut avoid);
        self.render_avoiding(names, &avoid, kernel, top)
    }

    fn render_avoiding(
        &self,
        names: &mut Vec<String>,
        avoid: &BTreeSet<String>,
        kernel: bool,
        top: bool,
    ) -> String {
        let term = |t: &Term, names: &[String]| {
            if kernel {
                t.kernel_form_in(names)
            } else {
                t.display_in(names)
            }
        };
        let sub = |p: &Prop, names: &mut Vec<String>| {
            let s = p.render_avoiding(names, avoid, kernel, false);
            match p {
                Prop::Eq(_) | Prop::False => s,
                Prop::Imp(_, b) if **b == Prop::False => s,
                _ => format!("({s})"),
            }
        };
        let _ = top;
        match self {
            Prop::Eq(f) => format!("{} = {}", term(&f.lhs, names), term(&f.rhs, names)),
            Prop::False => "⊥".to_string(),
            Prop::Imp(a, b) if **b == Prop::False => match **a {
                Prop::Eq(_) => format!("¬({})", a.render_avoiding(names, avoid, kernel, false)),
                _ => format!("¬{}", sub(a, names)),
            },
            Prop::And(a, b) => format!("{} ∧ {}", sub(a, names), sub(b, names)),
            Prop::Or(a, b) => format!("{} ∨ {}", sub(a, names), sub(b, names)),
            Prop::Imp(a, b) => format!("{} → {}", sub(a, names), sub(b, names)),
            Prop::Forall(h, b) | Prop::Exists(h, b) => {
                let q = if matches!(self, Prop::Forall(..)) {
                    "∀"
                } else {
                    "∃"
                };
                // Binder hints are printing names only; a hint already in
                // scope, or equal to a parameter of the formula, is primed so
                // the printed formula stays unambiguous.
                let mut h = h.clone();
                while names.contains(&h) || avoid.contains(&h) {
                    h.push('′');
                }
                names.push(h.clone());
                let inner = b.render_avoiding(names, avoid, kernel, true);
                names.pop();
                // A quantifier extends as far right as possible; a binary
                // body is bracketed so the scope is visible (audit W4).
                let body = match &**b {
                    Prop::And(..) | Prop::Or(..) => format!("({inner})"),
                    Prop::Imp(_, r) if **r != Prop::False => format!("({inner})"),
                    _ => inner,
                };
                format!("{q}{h}. {body}")
            }
        }
    }

    pub fn kernel_form(&self) -> String {
        match self {
            Prop::Eq(f) => f.kernel_form(),
            other => other.render(&mut Vec::new(), true, true),
        }
    }
}

impl fmt::Display for Prop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Prop::Eq(e) => write!(f, "{e}"),
            other => write!(f, "{}", other.render(&mut Vec::new(), false, true)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(a: &str) -> Term {
        Term::Param(a.into())
    }
    fn n(k: u64) -> Term {
        Term::numeral(k)
    }

    #[test]
    fn open_close_round_trip_and_alpha_equality() {
        // φ = ∀y. a + y = y + a
        let phi = Prop::Forall(
            "y".into(),
            Box::new(Prop::eq(
                Term::plus(p("a"), Term::Bound(0)),
                Term::plus(Term::Bound(0), p("a")),
            )),
        );
        let closed = phi.close("a");
        assert!(closed.check_wf(1).is_ok());
        assert!(
            closed.check_wf(0).is_err(),
            "close creates an index for an outer binder"
        );
        assert_eq!(closed.open(&p("a")), phi);
        // close leaves φ unchanged when a is absent
        assert_eq!(phi.close("b"), phi);
        // binder hints are ignored by equality
        let renamed = Prop::Forall(
            "z".into(),
            Box::new(Prop::eq(
                Term::plus(p("a"), Term::Bound(0)),
                Term::plus(Term::Bound(0), p("a")),
            )),
        );
        assert_eq!(phi, renamed);
    }

    #[test]
    fn open_cannot_capture() {
        // body of ∀x. ∃y. x = y, opened with the parameter named y
        let body = Prop::Exists(
            "y".into(),
            Box::new(Prop::eq(Term::Bound(1), Term::Bound(0))),
        );
        let opened = body.open(&p("y"));
        // The printed hint is primed away from the parameter's name, so the
        // display cannot suggest a capture that the terms do not contain.
        assert_eq!(opened.to_string(), "∃y′. y = y′");
        let capture = Prop::Exists(
            "y".into(),
            Box::new(Prop::eq(Term::Bound(0), Term::Bound(0))),
        );
        assert_ne!(
            opened, capture,
            "the parameter y stays distinct from the bound y"
        );
        assert!(opened.has_param("y"));
    }

    #[test]
    fn well_formedness_refuses_dangling_indices_and_axiom_variables() {
        assert!(Prop::eq(Term::Bound(0), n(0)).check_wf(0).is_err());
        assert!(Prop::eq(Term::Var("x".into()), n(0)).check_wf(0).is_err());
        let mut deep = Prop::eq(n(0), n(0));
        for _ in 0..300 {
            deep = Prop::Forall("x".into(), Box::new(deep));
        }
        assert!(deep.check_wf(0).unwrap_err().contains("nested deeper"));
    }

    #[test]
    fn display_matches_formula_display_for_equations() {
        let f = Formula::new(Term::plus(n(1), n(1)), n(2));
        assert_eq!(Prop::Eq(f.clone()).to_string(), f.to_string());
        assert_eq!(Prop::Eq(f.clone()).kernel_form(), f.kernel_form());
        let neg = Prop::Imp(Box::new(Prop::Eq(f)), Box::new(Prop::False));
        assert_eq!(neg.to_string(), "¬(1 + 1 = 2)");
    }
}

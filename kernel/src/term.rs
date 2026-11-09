//! Terms, formulas and the signature (component C: sort checking).
//!
//! RLT-001 has one sort, `Nat`, and three function symbols:
//! `0 : Nat`, `S : Nat -> Nat`, `+ : Nat × Nat -> Nat`.
//! RLT-003 M1 adds one: `* : Nat × Nat -> Nat` (multiplication, printed `×`).
//! Numerals are abbreviations: the numeral n is the term S^n(0).
//! Equality of terms is structural equality of these trees.

use std::fmt;

/// A first-order term. `Var` occurs only inside registered axioms.
/// RLT-003 M3a adds two forms, kept apart from `Var` by construction:
/// `Param` (an eigenvariable, free in a derivation, never in a goal, a
/// declared assumption or an axiom) and `Bound` (a de Bruijn index, only
/// under a binder of the formula that contains it; see `prop`).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Term {
    Var(String),
    App(String, Vec<Term>),
    Param(String),
    Bound(usize),
}

/// An equation `lhs = rhs`. RLT-001 formulas are equations only.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Formula {
    pub lhs: Term,
    pub rhs: Term,
}

/// Arity of each function symbol of the signature (every argument has sort Nat).
pub fn arity(symbol: &str) -> Option<usize> {
    match symbol {
        "0" => Some(0),
        "S" => Some(1),
        "+" => Some(2),
        "*" => Some(2),
        _ => None,
    }
}

impl Term {
    pub fn zero() -> Term {
        Term::App("0".into(), vec![])
    }

    pub fn succ(t: Term) -> Term {
        Term::App("S".into(), vec![t])
    }

    pub fn plus(a: Term, b: Term) -> Term {
        Term::App("+".into(), vec![a, b])
    }

    /// The numeral n, expanded to S^n(0).
    pub fn numeral(n: u64) -> Term {
        let mut t = Term::zero();
        for _ in 0..n {
            t = Term::succ(t);
        }
        t
    }

    /// If this term is S^n(0), return n.
    pub fn as_numeral(&self) -> Option<u64> {
        let mut n = 0u64;
        let mut t = self;
        loop {
            match t {
                Term::App(f, args) if f == "0" && args.is_empty() => return Some(n),
                Term::App(f, args) if f == "S" && args.len() == 1 => {
                    n += 1;
                    t = &args[0];
                }
                _ => return None,
            }
        }
    }

    /// Sort check. `allow_vars` is true only for axiom bodies.
    /// Every symbol must be in the signature and applied to exactly its arity.
    pub fn check_sort(&self, allow_vars: bool) -> Result<(), String> {
        match self {
            Term::Var(v) => {
                if allow_vars {
                    Ok(())
                } else {
                    Err(format!(
                        "variable `{v}` outside an axiom: proof terms are ground in RLT-001"
                    ))
                }
            }
            // A parameter is a term of sort Nat; where it may occur is
            // decided by the checker (goals, assumptions) and by
            // `Axiom::new` (never in an axiom).
            Term::Param(_) => Ok(()),
            // A standalone term is locally closed: an index is legal only
            // inside a formula, checked by `Prop::check_wf`.
            Term::Bound(i) => Err(format!("bound index #{i} outside its binder")),
            Term::App(f, args) => match arity(f) {
                None => Err(format!(
                    "symbol `{f}` is outside the signature {{0, S, +, *}}"
                )),
                Some(n) if n != args.len() => Err(format!(
                    "symbol `{f}` has arity {n}, applied to {} argument(s)",
                    args.len()
                )),
                Some(_) => args.iter().try_for_each(|a| a.check_sort(allow_vars)),
            },
        }
    }

    pub fn vars(&self, out: &mut Vec<String>) {
        match self {
            Term::Var(v) => {
                if !out.contains(v) {
                    out.push(v.clone())
                }
            }
            Term::App(_, args) => args.iter().for_each(|a| a.vars(out)),
            Term::Param(_) | Term::Bound(_) => {}
        }
    }

    /// Parameter names occurring in this term.
    pub fn params(&self, out: &mut std::collections::BTreeSet<String>) {
        match self {
            Term::Param(a) => {
                out.insert(a.clone());
            }
            Term::App(_, args) => args.iter().for_each(|x| x.params(out)),
            Term::Var(_) | Term::Bound(_) => {}
        }
    }

    /// Whether any `Var` (axiom variable) occurs in this term.
    pub fn has_var(&self) -> bool {
        match self {
            Term::Var(_) => true,
            Term::App(_, args) => args.iter().any(|a| a.has_var()),
            Term::Param(_) | Term::Bound(_) => false,
        }
    }

    /// Whether any `Param` or `Bound` occurs in this term.
    pub fn has_param_or_bound(&self) -> bool {
        match self {
            Term::Param(_) | Term::Bound(_) => true,
            Term::App(_, args) => args.iter().any(|a| a.has_param_or_bound()),
            Term::Var(_) => false,
        }
    }

    /// open at binder depth k: `Bound(k)` becomes `t`; deeper indices shift down.
    /// `t` is locally closed and holds no binders, so nothing is captured.
    pub(crate) fn open_at(&self, t: &Term, k: usize) -> Term {
        match self {
            Term::Bound(i) if *i == k => t.clone(),
            Term::Bound(i) if *i > k => Term::Bound(i - 1),
            Term::App(f, args) => {
                Term::App(f.clone(), args.iter().map(|a| a.open_at(t, k)).collect())
            }
            other => other.clone(),
        }
    }

    /// close at binder depth k: `Param(a)` becomes `Bound(k)`.
    pub(crate) fn close_at(&self, a: &str, k: usize) -> Term {
        match self {
            Term::Param(p) if p == a => Term::Bound(k),
            Term::App(f, args) => {
                Term::App(f.clone(), args.iter().map(|x| x.close_at(a, k)).collect())
            }
            other => other.clone(),
        }
    }

    /// Display with binder names in scope (innermost last); `Bound(i)` prints
    /// the name of the binder it refers to.
    pub fn display_in(&self, names: &[String]) -> String {
        struct D<'a>(&'a Term, &'a [String]);
        impl fmt::Display for D<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt_term(self.0, true, self.1, f)
            }
        }
        D(self, names).to_string()
    }

    pub fn kernel_form_in(&self, names: &[String]) -> String {
        match self {
            Term::Bound(i) => bound_name(*i, names),
            Term::App(f, args) if !args.is_empty() => {
                let inner: Vec<String> = args.iter().map(|a| a.kernel_form_in(names)).collect();
                format!("{f}({})", inner.join(", "))
            }
            other => other.kernel_form(),
        }
    }

    /// Kernel form: every numeral fully expanded, e.g. `S(S(0))`.
    pub fn kernel_form(&self) -> String {
        match self {
            Term::Var(v) => v.clone(),
            Term::Param(a) => a.clone(),
            Term::Bound(i) => format!("#{i}"),
            Term::App(f, args) if args.is_empty() => f.clone(),
            Term::App(f, args) => {
                let inner: Vec<String> = args.iter().map(|a| a.kernel_form()).collect();
                format!("{f}({})", inner.join(", "))
            }
        }
    }
}

fn bound_name(i: usize, names: &[String]) -> String {
    match names.len().checked_sub(i + 1).and_then(|k| names.get(k)) {
        Some(n) => n.clone(),
        None => format!("#{i}"),
    }
}

fn fmt_term(t: &Term, top: bool, names: &[String], f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if let Some(n) = t.as_numeral() {
        return write!(f, "{n}");
    }
    match t {
        Term::Var(v) => write!(f, "{v}"),
        Term::Param(a) => write!(f, "{a}"),
        Term::Bound(i) => write!(f, "{}", bound_name(*i, names)),
        Term::App(s, args) if (s == "+" || s == "*") && args.len() == 2 => {
            if !top {
                write!(f, "(")?;
            }
            fmt_term(&args[0], false, names, f)?;
            write!(f, " {} ", if s == "+" { "+" } else { "×" })?;
            fmt_term(&args[1], false, names, f)?;
            if !top {
                write!(f, ")")?;
            }
            Ok(())
        }
        Term::App(s, args) if args.is_empty() => write!(f, "{s}"),
        Term::App(s, args) => {
            write!(f, "{s}(")?;
            for (i, a) in args.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                fmt_term(a, true, names, f)?;
            }
            write!(f, ")")
        }
    }
}

/// Display form: numerals S^n(0) shown as n; `+` infix, fully parenthesised below the top level.
impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_term(self, true, &[], f)
    }
}

impl Formula {
    pub fn new(lhs: Term, rhs: Term) -> Formula {
        Formula { lhs, rhs }
    }

    pub fn check_sort(&self, allow_vars: bool) -> Result<(), String> {
        self.lhs.check_sort(allow_vars)?;
        self.rhs.check_sort(allow_vars)
    }

    pub fn kernel_form(&self) -> String {
        format!("{} = {}", self.lhs.kernel_form(), self.rhs.kernel_form())
    }
}

impl fmt::Display for Formula {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {}", self.lhs, self.rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiplication_is_binary_and_prints_infix() {
        let two_three = Term::App("*".into(), vec![Term::numeral(2), Term::numeral(3)]);
        assert!(two_three.check_sort(false).is_ok());
        assert_eq!(two_three.to_string(), "2 × 3");
        let nested = Term::App("*".into(), vec![two_three.clone(), Term::numeral(5)]);
        assert_eq!(
            Formula::new(Term::numeral(30), nested).to_string(),
            "30 = (2 × 3) × 5"
        );
        assert!(Term::App("*".into(), vec![Term::numeral(2)])
            .check_sort(false)
            .is_err());
        assert!(Term::App(
            "*".into(),
            vec![Term::numeral(2), Term::numeral(3), Term::numeral(5)]
        )
        .check_sort(false)
        .is_err());
    }

    #[test]
    fn numerals_expand_structurally() {
        assert_eq!(Term::numeral(6), Term::succ(Term::numeral(5)));
        assert_eq!(Term::numeral(1), Term::succ(Term::zero()));
        assert_eq!(Term::numeral(30).as_numeral(), Some(30));
        assert_ne!(Term::numeral(6), Term::numeral(5));
    }

    #[test]
    fn sort_check_accepts_signature_terms() {
        let t = Term::plus(Term::numeral(5), Term::numeral(1));
        assert!(t.check_sort(false).is_ok());
    }

    #[test]
    fn sort_check_rejects_wrong_arity_unknown_symbol_and_free_variable() {
        let bad_arity = Term::App("S".into(), vec![]);
        assert!(bad_arity.check_sort(false).is_err());
        let unknown = Term::App("REAL-PI.pi".into(), vec![]);
        assert!(unknown.check_sort(false).is_err());
        let var = Term::Var("x".into());
        assert!(var.check_sort(false).is_err());
        assert!(var.check_sort(true).is_ok());
    }

    #[test]
    fn display_and_kernel_forms() {
        let t = Term::plus(Term::numeral(5), Term::succ(Term::zero()));
        assert_eq!(t.to_string(), "5 + 1");
        assert_eq!(Term::numeral(2).kernel_form(), "S(S(0))");
    }
}

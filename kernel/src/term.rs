//! Terms, formulas and the signature (component C: sort checking).
//!
//! RLT-001 has one sort, `Nat`, and three function symbols:
//! `0 : Nat`, `S : Nat -> Nat`, `+ : Nat × Nat -> Nat`.
//! RLT-003 M1 adds one: `* : Nat × Nat -> Nat` (multiplication, printed `×`).
//! Numerals are abbreviations: the numeral n is the term S^n(0).
//! Equality of terms is structural equality of these trees.

use std::fmt;

/// A first-order term. `Var` occurs only inside registered axioms.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Term {
    Var(String),
    App(String, Vec<Term>),
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
        }
    }

    /// Kernel form: every numeral fully expanded, e.g. `S(S(0))`.
    pub fn kernel_form(&self) -> String {
        match self {
            Term::Var(v) => v.clone(),
            Term::App(f, args) if args.is_empty() => f.clone(),
            Term::App(f, args) => {
                let inner: Vec<String> = args.iter().map(|a| a.kernel_form()).collect();
                format!("{f}({})", inner.join(", "))
            }
        }
    }
}

/// Display form: numerals S^n(0) shown as n; `+` infix, fully parenthesised below the top level.
impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn go(t: &Term, top: bool, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if let Some(n) = t.as_numeral() {
                return write!(f, "{n}");
            }
            match t {
                Term::Var(v) => write!(f, "{v}"),
                Term::App(s, args) if (s == "+" || s == "*") && args.len() == 2 => {
                    if !top {
                        write!(f, "(")?;
                    }
                    go(&args[0], false, f)?;
                    write!(f, " {} ", if s == "+" { "+" } else { "×" })?;
                    go(&args[1], false, f)?;
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
                        go(a, true, f)?;
                    }
                    write!(f, ")")
                }
            }
        }
        go(self, true, f)
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

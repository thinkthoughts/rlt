//! Theorems and the six inference rules (component D).
//!
//! A `Theorem` is a sequent Γ ⊢ lhs = rhs. Its fields are private to this
//! module: outside code obtains a Theorem only by calling one of the rules
//! below, so every Theorem value is the conclusion of a checked derivation.
//!
//! Dependency preservation: every rule's Γ is the union of its premises' Γ,
//! and `hyp` is the only rule that introduces an assumption. A Theorem is
//! unconditional exactly when Γ is empty.

use crate::term::{arity, Formula, Term};
use std::collections::{BTreeMap, BTreeSet};

/// A registered axiom: a universally quantified equation with a source reference.
/// Axioms are trusted inputs; the checker prints every axiom a result depends on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Axiom {
    name: String,
    vars: Vec<String>,
    eq: Formula,
    source: String,
}

impl Axiom {
    pub fn new(name: &str, vars: Vec<String>, eq: Formula, source: &str) -> Result<Axiom, String> {
        eq.check_sort(true)
            .map_err(|e| format!("axiom `{name}`: {e}"))?;
        let mut used = Vec::new();
        eq.lhs.vars(&mut used);
        eq.rhs.vars(&mut used);
        for v in &used {
            if !vars.contains(v) {
                return Err(format!("axiom `{name}`: variable `{v}` is undeclared"));
            }
        }
        let mut seen = BTreeSet::new();
        for v in &vars {
            if !seen.insert(v.clone()) {
                return Err(format!("axiom `{name}`: variable `{v}` declared twice"));
            }
        }
        Ok(Axiom {
            name: name.to_string(),
            vars,
            eq,
            source: source.to_string(),
        })
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn vars(&self) -> &[String] {
        &self.vars
    }
    pub fn equation(&self) -> &Formula {
        &self.eq
    }
    pub fn source(&self) -> &str {
        &self.source
    }
}

/// Γ ⊢ concl, plus the full content (name, equation, source) of every axiom
/// used. Constructed only by the rules in this module. Recording axiom content,
/// not just names, lets a consumer compare the axioms a theorem rests on with
/// the axioms it trusts: an outside `Axiom` that merely reuses a registered
/// name stays distinguishable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Theorem {
    hyps: BTreeMap<String, Formula>,
    concl: Formula,
    axioms: BTreeMap<String, Axiom>,
}

impl Theorem {
    pub fn conclusion(&self) -> &Formula {
        &self.concl
    }
    /// The assumption context Γ, by assumption name.
    pub fn assumptions(&self) -> &BTreeMap<String, Formula> {
        &self.hyps
    }
    /// Every axiom this theorem rests on, by name, with its full content.
    pub fn axioms_used(&self) -> &BTreeMap<String, Axiom> {
        &self.axioms
    }
    pub fn is_unconditional(&self) -> bool {
        self.hyps.is_empty()
    }
}

type Context = (BTreeMap<String, Formula>, BTreeMap<String, Axiom>);

fn union(ths: &[&Theorem]) -> Result<Context, String> {
    let mut hyps: BTreeMap<String, Formula> = BTreeMap::new();
    let mut axioms: BTreeMap<String, Axiom> = BTreeMap::new();
    for th in ths {
        for (name, f) in &th.hyps {
            if let Some(prev) = hyps.get(name) {
                if prev != f {
                    return Err(format!(
                        "assumption name `{name}` refers to two different formulas"
                    ));
                }
            }
            hyps.insert(name.clone(), f.clone());
        }
        for (name, ax) in &th.axioms {
            if let Some(prev) = axioms.get(name) {
                if prev != ax {
                    return Err(format!(
                        "axiom name `{name}` refers to two different axioms"
                    ));
                }
            }
            axioms.insert(name.clone(), ax.clone());
        }
    }
    Ok((hyps, axioms))
}

/// refl: ⊢ t = t
pub fn refl(t: &Term) -> Result<Theorem, String> {
    t.check_sort(false)?;
    Ok(Theorem {
        hyps: BTreeMap::new(),
        concl: Formula::new(t.clone(), t.clone()),
        axioms: BTreeMap::new(),
    })
}

/// symm: from Γ ⊢ a = b infer Γ ⊢ b = a
pub fn symm(th: &Theorem) -> Result<Theorem, String> {
    let (hyps, axioms) = union(&[th])?;
    Ok(Theorem {
        hyps,
        concl: Formula::new(th.concl.rhs.clone(), th.concl.lhs.clone()),
        axioms,
    })
}

/// trans: from Γ₁ ⊢ a = b and Γ₂ ⊢ b = c infer Γ₁ ∪ Γ₂ ⊢ a = c
/// The middle terms must be structurally identical.
pub fn trans(th1: &Theorem, th2: &Theorem) -> Result<Theorem, String> {
    if th1.concl.rhs != th2.concl.lhs {
        return Err(format!(
            "trans requires rhs(first) ≡ lhs(second); here `{}` ≢ `{}`",
            th1.concl.rhs, th2.concl.lhs
        ));
    }
    let (hyps, axioms) = union(&[th1, th2])?;
    Ok(Theorem {
        hyps,
        concl: Formula::new(th1.concl.lhs.clone(), th2.concl.rhs.clone()),
        axioms,
    })
}

/// cong: from Γᵢ ⊢ aᵢ = bᵢ (one per argument) infer ⋃Γᵢ ⊢ f(a…) = f(b…)
pub fn cong(symbol: &str, ths: &[&Theorem]) -> Result<Theorem, String> {
    match arity(symbol) {
        None => return Err(format!("cong: symbol `{symbol}` is outside the signature")),
        Some(0) => {
            return Err(format!(
                "cong: symbol `{symbol}` takes no arguments; use refl"
            ))
        }
        Some(n) if n != ths.len() => {
            return Err(format!(
                "cong {symbol}: needs {n} premise(s), given {}",
                ths.len()
            ))
        }
        Some(_) => {}
    }
    let (hyps, axioms) = union(ths)?;
    let lhs = Term::App(
        symbol.into(),
        ths.iter().map(|t| t.concl.lhs.clone()).collect(),
    );
    let rhs = Term::App(
        symbol.into(),
        ths.iter().map(|t| t.concl.rhs.clone()).collect(),
    );
    Ok(Theorem {
        hyps,
        concl: Formula::new(lhs, rhs),
        axioms,
    })
}

fn substitute(t: &Term, sigma: &BTreeMap<String, Term>) -> Term {
    match t {
        Term::Var(v) => sigma[v].clone(),
        Term::App(f, args) => Term::App(
            f.clone(),
            args.iter().map(|a| substitute(a, sigma)).collect(),
        ),
    }
}

/// inst: ⊢ σ(lhs) = σ(rhs) for a registered axiom, where σ binds every
/// axiom variable exactly once to a ground, well-sorted term.
pub fn inst(ax: &Axiom, bindings: &[(String, Term)]) -> Result<Theorem, String> {
    let mut sigma = BTreeMap::new();
    for (v, t) in bindings {
        if !ax.vars.contains(v) {
            return Err(format!(
                "inst {}: `{v}` is not a variable of this axiom",
                ax.name
            ));
        }
        t.check_sort(false)
            .map_err(|e| format!("inst {}: [{v} := …]: {e}", ax.name))?;
        if sigma.insert(v.clone(), t.clone()).is_some() {
            return Err(format!("inst {}: `{v}` bound twice", ax.name));
        }
    }
    for v in &ax.vars {
        if !sigma.contains_key(v) {
            return Err(format!("inst {}: variable `{v}` is unbound", ax.name));
        }
    }
    let mut axioms = BTreeMap::new();
    axioms.insert(ax.name.clone(), ax.clone());
    Ok(Theorem {
        hyps: BTreeMap::new(),
        concl: Formula::new(
            substitute(&ax.eq.lhs, &sigma),
            substitute(&ax.eq.rhs, &sigma),
        ),
        axioms,
    })
}

/// hyp: {name : φ} ⊢ φ. The only rule that introduces an assumption.
/// The checker allows it only for assumptions the goal declares.
pub fn hyp(name: &str, phi: &Formula) -> Result<Theorem, String> {
    phi.check_sort(false)?;
    let mut hyps = BTreeMap::new();
    hyps.insert(name.to_string(), phi.clone());
    Ok(Theorem {
        hyps,
        concl: phi.clone(),
        axioms: BTreeMap::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn add_zero() -> Axiom {
        let x = Term::Var("x".into());
        Axiom::new(
            "add_zero",
            vec!["x".into()],
            Formula::new(Term::plus(x.clone(), Term::zero()), x),
            "test",
        )
        .unwrap()
    }
    fn add_succ() -> Axiom {
        let (x, y) = (Term::Var("x".into()), Term::Var("y".into()));
        Axiom::new(
            "add_succ",
            vec!["x".into(), "y".into()],
            Formula::new(
                Term::plus(x.clone(), Term::succ(y.clone())),
                Term::succ(Term::plus(x, y)),
            ),
            "test",
        )
        .unwrap()
    }
    fn n(k: u64) -> Term {
        Term::numeral(k)
    }

    #[test]
    fn refl_accepts_ground_and_rejects_ill_sorted() {
        assert!(refl(&n(3)).unwrap().is_unconditional());
        assert!(refl(&Term::App("S".into(), vec![])).is_err());
    }

    #[test]
    fn symm_swaps_and_keeps_assumptions() {
        let h = hyp("h", &Formula::new(n(1), n(3))).unwrap();
        let s = symm(&h).unwrap();
        assert_eq!(s.conclusion(), &Formula::new(n(3), n(1)));
        assert!(s.assumptions().contains_key("h"));
    }

    #[test]
    fn trans_accepts_matching_middle_and_rejects_mismatch() {
        let s1 = inst(&add_succ(), &[("x".into(), n(5)), ("y".into(), n(0))]).unwrap();
        let s2 = inst(&add_zero(), &[("x".into(), n(5))]).unwrap();
        let s3 = cong("S", &[&s2]).unwrap();
        let s4 = trans(&s1, &s3).unwrap();
        assert_eq!(s4.conclusion(), &Formula::new(Term::plus(n(5), n(1)), n(6)));
        assert!(trans(&s1, &s2).is_err());
    }

    #[test]
    fn cong_checks_arity() {
        let r = refl(&n(1)).unwrap();
        assert!(cong("S", &[&r]).is_ok());
        assert!(cong("+", &[&r]).is_err());
        assert!(cong("0", &[]).is_err());
        assert!(cong("REAL-PI.pi", &[&r]).is_err());
    }

    #[test]
    fn inst_rejects_unbound_extra_and_ill_sorted_bindings() {
        assert!(inst(&add_succ(), &[("x".into(), n(5))]).is_err());
        assert!(inst(&add_zero(), &[("x".into(), n(5)), ("z".into(), n(1))]).is_err());
        assert!(inst(&add_zero(), &[("x".into(), Term::Var("q".into()))]).is_err());
        assert!(inst(&add_zero(), &[("x".into(), n(2)), ("x".into(), n(3))]).is_err());
    }

    #[test]
    fn hyp_introduces_one_assumption_and_composition_preserves_it() {
        let h = hyp("h", &Formula::new(Term::plus(n(1), n(1)), n(3))).unwrap();
        assert!(!h.is_unconditional());
        let c = cong("S", &[&h]).unwrap();
        let r = refl(&Term::succ(n(3))).unwrap();
        let t = trans(&c, &r).unwrap();
        assert_eq!(t.assumptions().keys().collect::<Vec<_>>(), vec!["h"]);
        assert!(!t.is_unconditional());
    }

    #[test]
    fn axioms_used_accumulate() {
        let s1 = inst(&add_succ(), &[("x".into(), n(1)), ("y".into(), n(0))]).unwrap();
        let s2 = inst(&add_zero(), &[("x".into(), n(1))]).unwrap();
        let t = trans(&s1, &cong("S", &[&s2]).unwrap()).unwrap();
        assert_eq!(
            t.axioms_used().keys().cloned().collect::<Vec<_>>(),
            vec!["add_succ", "add_zero"]
        );
    }

    #[test]
    fn a_forged_axiom_reusing_a_registered_name_stays_distinguishable() {
        // An outside caller can build an Axiom named `add_zero` with a false
        // equation; the theorem records its content, so it differs from the
        // registered add_zero and the two cannot be combined.
        let forged = Axiom::new(
            "add_zero",
            vec![],
            Formula::new(Term::numeral(1), Term::numeral(2)),
            "test",
        )
        .unwrap();
        let bad = inst(&forged, &[]).unwrap();
        assert_ne!(bad.axioms_used()["add_zero"], add_zero());
        let good = inst(&add_zero(), &[("x".into(), n(1))]).unwrap();
        assert!(trans(&good, &bad).is_err() || trans(&bad, &good).is_err());
        let r = refl(&n(2)).unwrap();
        assert!(trans(&trans(&bad, &r).unwrap(), &r).is_ok());
        let both = cong("+", &[&good, &bad]);
        assert!(
            both.is_err(),
            "same axiom name with two contents must be refused"
        );
    }

    #[test]
    fn axiom_rejects_undeclared_variable() {
        let bad = Axiom::new(
            "bad",
            vec![],
            Formula::new(Term::Var("x".into()), n(0)),
            "test",
        );
        assert!(bad.is_err());
    }
}

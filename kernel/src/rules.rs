//! Theorems and the inference rules (component D).
//!
//! A `Theorem` is a sequent Γ ⊢ φ. Its fields are private to this module:
//! outside code obtains a Theorem only by calling one of the rules below, so
//! every Theorem value is the conclusion of a checked derivation.
//!
//! The six RLT-001 rules (refl, symm, trans, cong, inst, hyp) keep their
//! judgments on every input they accepted before. RLT-003 M3a widens them
//! (parameters in terms; `hyp` on any well-formed formula) and adds 14 rules
//! for ⊥, ∧, ∨, →, ∀, ∃ and substitution.
//!
//! Dependency preservation: every rule's Γ is the union of its premises' Γ,
//! minus exactly the assumptions it discharges, each from the one premise
//! named for it; `hyp` is the only rule that introduces an assumption. A
//! Theorem is unconditional exactly when Γ is empty.

use crate::prop::Prop;
use crate::term::{arity, Formula, Term};
use std::collections::{BTreeMap, BTreeSet};

/// A registered axiom: a formula whose free axiom variables are read as
/// universally quantified, with a source reference. Axioms are trusted inputs;
/// the checker prints every axiom a result depends on. RLT-001 to M3a axioms
/// are equations; RLT-003 M3b admits any formula body (Q1–Q3 of Robinson's Q).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Axiom {
    name: String,
    vars: Vec<String>,
    body: Prop,
    source: String,
}

impl Axiom {
    /// An equational axiom, exactly as before M3b.
    pub fn new(name: &str, vars: Vec<String>, eq: Formula, source: &str) -> Result<Axiom, String> {
        eq.check_sort(true)
            .map_err(|e| format!("axiom `{name}`: {e}"))?;
        if eq.lhs.has_param_or_bound() || eq.rhs.has_param_or_bound() {
            return Err(format!(
                "axiom `{name}`: parameters and bound indices are outside axioms"
            ));
        }
        Axiom::build(name, vars, Prop::Eq(eq), source)
    }

    /// A formula axiom (RLT-003 M3b): the body may use connectives and
    /// quantifiers; its own binders are bound indices, its free variables are
    /// the declared axiom variables, and it holds no parameter.
    pub fn formula(
        name: &str,
        vars: Vec<String>,
        body: Prop,
        source: &str,
    ) -> Result<Axiom, String> {
        body.check_axiom_body()
            .map_err(|e| format!("axiom `{name}`: {e}"))?;
        Axiom::build(name, vars, body, source)
    }

    fn build(name: &str, vars: Vec<String>, body: Prop, source: &str) -> Result<Axiom, String> {
        let mut used = Vec::new();
        body.vars(&mut used);
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
            body,
            source: source.to_string(),
        })
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn vars(&self) -> &[String] {
        &self.vars
    }
    /// The axiom's equation, when it is an equational axiom.
    pub fn equation(&self) -> Option<&Formula> {
        self.body.as_eq()
    }
    /// The axiom's body, with its axiom variables free.
    pub fn body(&self) -> &Prop {
        &self.body
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
    hyps: BTreeMap<String, Prop>,
    concl: Prop,
    axioms: BTreeMap<String, Axiom>,
}

impl Theorem {
    pub fn conclusion(&self) -> &Prop {
        &self.concl
    }
    /// The conclusion, if it is an equation.
    pub fn equation(&self) -> Option<&Formula> {
        self.concl.as_eq()
    }
    /// The assumption context Γ, by assumption name.
    pub fn assumptions(&self) -> &BTreeMap<String, Prop> {
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

type Hyps = BTreeMap<String, Prop>;
type Axioms = BTreeMap<String, Axiom>;

fn merge(parts: &[(&Hyps, &Axioms)]) -> Result<(Hyps, Axioms), String> {
    let mut hyps: Hyps = BTreeMap::new();
    let mut axioms: Axioms = BTreeMap::new();
    for (hs, axs) in parts {
        for (name, f) in hs.iter() {
            if let Some(prev) = hyps.get(name) {
                if prev != f {
                    return Err(format!(
                        "assumption name `{name}` refers to two different formulas"
                    ));
                }
            }
            hyps.insert(name.clone(), f.clone());
        }
        for (name, ax) in axs.iter() {
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

fn union(ths: &[&Theorem]) -> Result<(Hyps, Axioms), String> {
    let parts: Vec<(&Hyps, &Axioms)> = ths.iter().map(|t| (&t.hyps, &t.axioms)).collect();
    merge(&parts)
}

/// The equation a premise concludes; the five equational rules accept only these.
fn eq_of<'a>(rule: &str, th: &'a Theorem) -> Result<&'a Formula, String> {
    th.concl
        .as_eq()
        .ok_or_else(|| format!("{rule}: premise is not an equation: `{}`", th.concl))
}

/// Remove assumption `h` from one premise's Γ. If present, its formula must
/// equal `expected` exactly (up to alpha-equivalence).
fn discharge(rule: &str, hyps: &Hyps, h: &str, expected: &Prop) -> Result<Hyps, String> {
    let mut out = hyps.clone();
    if let Some(f) = out.remove(h) {
        if &f != expected {
            return Err(format!(
                "{rule}: assumption `{h}` does not match: it states `{f}`, the rule discharges `{expected}`"
            ));
        }
    }
    Ok(out)
}

fn params_of(hyps: &Hyps) -> BTreeSet<String> {
    let mut s = BTreeSet::new();
    for f in hyps.values() {
        f.params(&mut s);
    }
    s
}

fn closed_term(rule: &str, t: &Term) -> Result<(), String> {
    t.check_sort(false).map_err(|e| format!("{rule}: {e}"))
}

/// refl: ⊢ t = t
pub fn refl(t: &Term) -> Result<Theorem, String> {
    t.check_sort(false)?;
    Ok(Theorem {
        hyps: BTreeMap::new(),
        concl: Prop::eq(t.clone(), t.clone()),
        axioms: BTreeMap::new(),
    })
}

/// symm: from Γ ⊢ a = b infer Γ ⊢ b = a
pub fn symm(th: &Theorem) -> Result<Theorem, String> {
    let e = eq_of("symm", th)?;
    let (hyps, axioms) = union(&[th])?;
    Ok(Theorem {
        hyps,
        concl: Prop::eq(e.rhs.clone(), e.lhs.clone()),
        axioms,
    })
}

/// trans: from Γ₁ ⊢ a = b and Γ₂ ⊢ b = c infer Γ₁ ∪ Γ₂ ⊢ a = c
/// The middle terms must be structurally identical.
pub fn trans(th1: &Theorem, th2: &Theorem) -> Result<Theorem, String> {
    let (e1, e2) = (eq_of("trans", th1)?, eq_of("trans", th2)?);
    if e1.rhs != e2.lhs {
        return Err(format!(
            "trans requires rhs(first) ≡ lhs(second); here `{}` ≢ `{}`",
            e1.rhs, e2.lhs
        ));
    }
    let (hyps, axioms) = union(&[th1, th2])?;
    Ok(Theorem {
        hyps,
        concl: Prop::eq(e1.lhs.clone(), e2.rhs.clone()),
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
    let eqs = ths
        .iter()
        .map(|t| eq_of("cong", t))
        .collect::<Result<Vec<_>, _>>()?;
    let (hyps, axioms) = union(ths)?;
    let lhs = Term::App(symbol.into(), eqs.iter().map(|e| e.lhs.clone()).collect());
    let rhs = Term::App(symbol.into(), eqs.iter().map(|e| e.rhs.clone()).collect());
    Ok(Theorem {
        hyps,
        concl: Prop::eq(lhs, rhs),
        axioms,
    })
}

/// inst: ⊢ σ(lhs) = σ(rhs) for a registered axiom, where σ binds every
/// axiom variable exactly once to a well-sorted, locally closed term
/// (parameters allowed since M3a; axiom variables never).
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
        // RLT-003 M3b: one substitution for every axiom form. On an equation it
        // computes exactly the pre-M3b result; under the body's own binders the
        // bound terms need no shift, being locally closed.
        concl: ax.body.subst_vars(&sigma),
        axioms,
    })
}

/// hyp: {name : φ} ⊢ φ. The only rule that introduces an assumption.
/// The checker decides which names it may introduce (declared by the goal,
/// or discharged later in the same proof).
pub fn hyp(name: &str, phi: &Prop) -> Result<Theorem, String> {
    phi.check_wf(0)?;
    let mut hyps = BTreeMap::new();
    hyps.insert(name.to_string(), phi.clone());
    Ok(Theorem {
        hyps,
        concl: phi.clone(),
        axioms: BTreeMap::new(),
    })
}

// ── RLT-003 M3a: the 14 logical rules ──────────────────────────────────────

/// subst: from Γ₁ ⊢ s = t and Γ₂ ⊢ ψ[s/x] infer Γ₁ ∪ Γ₂ ⊢ ψ[t/x].
/// `motive` is ψ with x as its distinguished variable: the body of a binder,
/// referring to x by the outermost index. Both sides use `open`.
pub fn subst(eq: &Theorem, motive: &Prop, th: &Theorem) -> Result<Theorem, String> {
    let e = eq_of("subst", eq)?;
    motive
        .check_wf(1)
        .map_err(|m| format!("subst: motive: {m}"))?;
    let left = motive
        .open_checked(&e.lhs)
        .map_err(|m| format!("subst: {m}"))?;
    if th.concl != left {
        return Err(format!(
            "subst: motive does not match premise: the motive at `{}` gives `{left}`, the premise proves `{}`",
            e.lhs, th.concl
        ));
    }
    let (hyps, axioms) = union(&[eq, th])?;
    Ok(Theorem {
        hyps,
        concl: motive
            .open_checked(&e.rhs)
            .map_err(|m| format!("subst: {m}"))?,
        axioms,
    })
}

pub fn and_intro(a: &Theorem, b: &Theorem) -> Result<Theorem, String> {
    let (hyps, axioms) = union(&[a, b])?;
    Ok(Theorem {
        hyps,
        concl: Prop::And(Box::new(a.concl.clone()), Box::new(b.concl.clone())),
        axioms,
    })
}

fn and_parts<'a>(rule: &str, th: &'a Theorem) -> Result<(&'a Prop, &'a Prop), String> {
    match &th.concl {
        Prop::And(a, b) => Ok((a, b)),
        other => Err(format!("{rule}: premise is not a conjunction: `{other}`")),
    }
}

pub fn and_left(th: &Theorem) -> Result<Theorem, String> {
    let (a, _) = and_parts("and-left", th)?;
    Ok(Theorem {
        hyps: th.hyps.clone(),
        concl: a.clone(),
        axioms: th.axioms.clone(),
    })
}

pub fn and_right(th: &Theorem) -> Result<Theorem, String> {
    let (_, b) = and_parts("and-right", th)?;
    Ok(Theorem {
        hyps: th.hyps.clone(),
        concl: b.clone(),
        axioms: th.axioms.clone(),
    })
}

/// or-left: from Γ ⊢ φ infer Γ ⊢ φ ∨ ψ (ψ stated by the step).
pub fn or_left(th: &Theorem, right: &Prop) -> Result<Theorem, String> {
    right.check_wf(0).map_err(|m| format!("or-left: {m}"))?;
    Ok(Theorem {
        hyps: th.hyps.clone(),
        concl: Prop::Or(Box::new(th.concl.clone()), Box::new(right.clone())),
        axioms: th.axioms.clone(),
    })
}

/// or-right: from Γ ⊢ ψ infer Γ ⊢ φ ∨ ψ (φ stated by the step).
pub fn or_right(left: &Prop, th: &Theorem) -> Result<Theorem, String> {
    left.check_wf(0).map_err(|m| format!("or-right: {m}"))?;
    Ok(Theorem {
        hyps: th.hyps.clone(),
        concl: Prop::Or(Box::new(left.clone()), Box::new(th.concl.clone())),
        axioms: th.axioms.clone(),
    })
}

/// or-cases: from Γ ⊢ φ ∨ ψ, Γ₁ ⊢ χ (using h1 : φ) and Γ₂ ⊢ χ (using h2 : ψ)
/// infer Γ ∪ (Γ₁ ∖ {h1}) ∪ (Γ₂ ∖ {h2}) ⊢ χ.
pub fn or_cases(
    th: &Theorem,
    h1: &str,
    c1: &Theorem,
    h2: &str,
    c2: &Theorem,
) -> Result<Theorem, String> {
    let (l, r) = match &th.concl {
        Prop::Or(l, r) => (l, r),
        other => return Err(format!("or-cases: premise is not a disjunction: `{other}`")),
    };
    if c1.concl != c2.concl {
        return Err(format!(
            "or-cases: cases conclude different formulas: `{}` and `{}`",
            c1.concl, c2.concl
        ));
    }
    let g1 = discharge("or-cases", &c1.hyps, h1, l)?;
    let g2 = discharge("or-cases", &c2.hyps, h2, r)?;
    let (hyps, axioms) = merge(&[(&th.hyps, &th.axioms), (&g1, &c1.axioms), (&g2, &c2.axioms)])?;
    Ok(Theorem {
        hyps,
        concl: c1.concl.clone(),
        axioms,
    })
}

/// imp-intro: from Γ ⊢ ψ (using h : φ) infer Γ ∖ {h} ⊢ φ → ψ (φ stated by the step).
pub fn imp_intro(h: &str, antecedent: &Prop, th: &Theorem) -> Result<Theorem, String> {
    antecedent
        .check_wf(0)
        .map_err(|m| format!("imp-intro: {m}"))?;
    let hyps = discharge("imp-intro", &th.hyps, h, antecedent)?;
    Ok(Theorem {
        hyps,
        concl: Prop::Imp(Box::new(antecedent.clone()), Box::new(th.concl.clone())),
        axioms: th.axioms.clone(),
    })
}

/// imp-elim: from Γ₁ ⊢ φ → ψ and Γ₂ ⊢ φ infer Γ₁ ∪ Γ₂ ⊢ ψ.
pub fn imp_elim(imp: &Theorem, arg: &Theorem) -> Result<Theorem, String> {
    let (a, b) = match &imp.concl {
        Prop::Imp(a, b) => (a, b),
        other => {
            return Err(format!(
                "imp-elim: premise is not an implication: `{other}`"
            ))
        }
    };
    if **a != arg.concl {
        return Err(format!(
            "imp-elim: antecedent does not match: `{a}` against `{}`",
            arg.concl
        ));
    }
    let (hyps, axioms) = union(&[imp, arg])?;
    Ok(Theorem {
        hyps,
        concl: (**b).clone(),
        axioms,
    })
}

/// false-elim: from Γ ⊢ ⊥ infer Γ ⊢ φ (φ stated by the step).
pub fn false_elim(th: &Theorem, phi: &Prop) -> Result<Theorem, String> {
    if th.concl != Prop::False {
        return Err(format!("false-elim: premise is not ⊥: `{}`", th.concl));
    }
    phi.check_wf(0).map_err(|m| format!("false-elim: {m}"))?;
    Ok(Theorem {
        hyps: th.hyps.clone(),
        concl: phi.clone(),
        axioms: th.axioms.clone(),
    })
}

/// forall-intro: from Γ ⊢ φ infer Γ ⊢ ∀x. close(φ, a), provided the
/// parameter a occurs in no formula of Γ.
pub fn forall_intro(a: &str, hint: &str, th: &Theorem) -> Result<Theorem, String> {
    if params_of(&th.hyps).contains(a) {
        return Err(format!("forall-intro: parameter `{a}` occurs in Γ"));
    }
    Ok(Theorem {
        hyps: th.hyps.clone(),
        concl: Prop::Forall(hint.to_string(), Box::new(th.concl.close(a))),
        axioms: th.axioms.clone(),
    })
}

/// forall-elim: from Γ ⊢ ∀x. φ infer Γ ⊢ open(φ, t).
pub fn forall_elim(th: &Theorem, t: &Term) -> Result<Theorem, String> {
    closed_term("forall-elim", t)?;
    match &th.concl {
        Prop::Forall(_, body) => Ok(Theorem {
            hyps: th.hyps.clone(),
            concl: body
                .open_checked(t)
                .map_err(|m| format!("forall-elim: {m}"))?,
            axioms: th.axioms.clone(),
        }),
        other => Err(format!(
            "forall-elim: premise is not a ∀ formula: `{other}`"
        )),
    }
}

/// exists-intro: from Γ ⊢ open(φ, t) infer Γ ⊢ ∃x. φ (φ stated by the step).
pub fn exists_intro(hint: &str, body: &Prop, t: &Term, th: &Theorem) -> Result<Theorem, String> {
    closed_term("exists-intro", t)?;
    body.check_wf(1).map_err(|m| format!("exists-intro: {m}"))?;
    let inst = body
        .open_checked(t)
        .map_err(|m| format!("exists-intro: {m}"))?;
    if th.concl != inst {
        return Err(format!(
            "exists-intro: witness does not match premise: `{t}` gives `{inst}`, the premise proves `{}`",
            th.concl
        ));
    }
    Ok(Theorem {
        hyps: th.hyps.clone(),
        concl: Prop::Exists(hint.to_string(), Box::new(body.clone())),
        axioms: th.axioms.clone(),
    })
}

/// exists-elim: from Γ₁ ⊢ ∃x. φ and Γ₂ ⊢ χ (using h : open(φ, a)) infer
/// Γ₁ ∪ (Γ₂ ∖ {h}) ⊢ χ, provided a occurs in neither φ nor χ nor any formula
/// of Γ₂ ∖ {h}. Γ₁ passes through whole (it may mention a; see the M3a
/// specification's soundness argument for Q-f).
pub fn exists_elim(ex: &Theorem, a: &str, h: &str, th: &Theorem) -> Result<Theorem, String> {
    let body = match &ex.concl {
        Prop::Exists(_, body) => body,
        other => {
            return Err(format!(
                "exists-elim: premise is not an ∃ formula: `{other}`"
            ))
        }
    };
    if body.has_param(a) {
        return Err(format!(
            "exists-elim: parameter `{a}` occurs in the eliminated formula `{}`",
            ex.concl
        ));
    }
    if th.concl.has_param(a) {
        return Err(format!(
            "exists-elim: parameter `{a}` occurs in the conclusion `{}`",
            th.concl
        ));
    }
    let witness = body
        .open_checked(&Term::Param(a.to_string()))
        .map_err(|m| format!("exists-elim: {m}"))?;
    let rest = discharge("exists-elim", &th.hyps, h, &witness)?;
    if params_of(&rest).contains(a) {
        return Err(format!(
            "exists-elim: parameter `{a}` occurs in Γ of the branch"
        ));
    }
    let (hyps, axioms) = merge(&[(&ex.hyps, &ex.axioms), (&rest, &th.axioms)])?;
    Ok(Theorem {
        hyps,
        concl: th.concl.clone(),
        axioms,
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
        let h = hyp("h", &Prop::eq(n(1), n(3))).unwrap();
        let s = symm(&h).unwrap();
        assert_eq!(s.conclusion(), &Prop::eq(n(3), n(1)));
        assert!(s.assumptions().contains_key("h"));
    }

    #[test]
    fn trans_accepts_matching_middle_and_rejects_mismatch() {
        let s1 = inst(&add_succ(), &[("x".into(), n(5)), ("y".into(), n(0))]).unwrap();
        let s2 = inst(&add_zero(), &[("x".into(), n(5))]).unwrap();
        let s3 = cong("S", &[&s2]).unwrap();
        let s4 = trans(&s1, &s3).unwrap();
        assert_eq!(s4.conclusion(), &Prop::eq(Term::plus(n(5), n(1)), n(6)));
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
        let h = hyp("h", &Prop::eq(Term::plus(n(1), n(1)), n(3))).unwrap();
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

    // ── RLT-003 M3a: obligations O1, O3, O7 at the kernel API ──────────────

    fn p(a: &str) -> Term {
        Term::Param(a.into())
    }

    #[test]
    fn o1_rules_refuse_dangling_indices_and_axiom_variables() {
        assert!(hyp("h", &Prop::eq(Term::Bound(0), n(0))).is_err());
        assert!(hyp("h", &Prop::eq(Term::Var("x".into()), n(0))).is_err());
        assert!(refl(&Term::Bound(0)).is_err());
        let all = forall_intro("a", "x", &refl(&p("a")).unwrap()).unwrap();
        assert!(forall_elim(&all, &Term::Bound(3)).is_err());
        assert!(forall_elim(&all, &Term::Var("x".into())).is_err());
        let one = refl(&n(1)).unwrap();
        assert!(or_left(&one, &Prop::eq(Term::Bound(0), n(0))).is_err());
        assert!(exists_intro("k", &Prop::eq(Term::Bound(1), n(1)), &n(1), &one).is_err());
        assert!(Axiom::new("bad", vec![], Formula::new(p("a"), n(0)), "t").is_err());
    }

    /// A tiny deterministic generator of locally closed formulas over
    /// parameters a, b and numerals, with up to `depth` connectives/binders.
    fn gen(seed: &mut u64, depth: usize, binders: usize) -> Prop {
        let mut next = |m: u64| {
            *seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (*seed >> 33) % m
        };
        let term = |k: u64, binders: usize| match k % 4 {
            0 => p("a"),
            1 => p("b"),
            2 if binders > 0 => Term::Bound((k as usize / 4) % binders),
            _ => Term::numeral(k % 3),
        };
        if depth == 0 {
            let (x, y) = (next(64), next(64));
            return Prop::eq(Term::plus(term(x, binders), n(1)), term(y, binders));
        }
        match next(6) {
            0 => Prop::And(
                Box::new(gen(seed, depth - 1, binders)),
                Box::new(gen(seed, depth - 1, binders)),
            ),
            1 => Prop::Or(
                Box::new(gen(seed, depth - 1, binders)),
                Box::new(gen(seed, depth - 1, binders)),
            ),
            2 => Prop::Imp(
                Box::new(gen(seed, depth - 1, binders)),
                Box::new(Prop::False),
            ),
            3 => Prop::Forall("x".into(), Box::new(gen(seed, depth - 1, binders + 1))),
            4 => Prop::Exists("y".into(), Box::new(gen(seed, depth - 1, binders + 1))),
            _ => gen(seed, 0, binders),
        }
    }

    #[test]
    fn o3_open_and_close_laws_over_generated_formulas() {
        let mut seed = 7u64;
        for i in 0..2000 {
            let phi = gen(&mut seed, 1 + i % 5, 0);
            assert!(phi.check_wf(0).is_ok(), "{phi}");
            // open(close(φ, a), a) = φ
            let closed = phi.close("a");
            assert!(closed.check_wf(1).is_ok());
            assert!(!closed.has_param("a"));
            assert_eq!(closed.open(&p("a")), phi, "{phi}");
            // close leaves φ unchanged when the parameter is absent
            assert_eq!(phi.close("c"), phi);
            // opening a locally closed body with a locally closed term is locally closed
            let opened = closed.open(&Term::plus(p("b"), n(2)));
            assert!(opened.check_wf(0).is_ok(), "{opened}");
            // equality ignores binder hints
            let renamed = Prop::Forall("q".into(), Box::new(closed.clone()));
            let same = Prop::Forall("r".into(), Box::new(closed));
            assert_eq!(renamed, same);
        }
    }

    #[test]
    fn o7_axioms_used_accumulate_through_every_new_rule() {
        let az = inst(&add_zero(), &[("x".into(), p("a"))]).unwrap();
        let ax = |t: &Theorem| t.axioms_used().keys().cloned().collect::<Vec<_>>();
        let all = forall_intro("a", "x", &az).unwrap();
        assert_eq!(ax(&all), vec!["add_zero"]);
        assert_eq!(ax(&forall_elim(&all, &n(3)).unwrap()), vec!["add_zero"]);
        let s1 = inst(&add_succ(), &[("x".into(), n(1)), ("y".into(), n(0))]).unwrap();
        let both = and_intro(&az, &s1).unwrap();
        assert_eq!(ax(&both), vec!["add_succ", "add_zero"]);
        assert_eq!(ax(&and_left(&both).unwrap()), vec!["add_succ", "add_zero"]);
        let ex = exists_intro(
            "k",
            &Prop::eq(Term::plus(p("a"), n(0)), Term::Bound(0)),
            &p("a"),
            &az,
        )
        .unwrap();
        assert_eq!(ax(&ex), vec!["add_zero"]);
        let imp = imp_intro("h", &Prop::eq(n(0), n(0)), &s1).unwrap();
        assert_eq!(ax(&imp), vec!["add_succ"]);
        let back = imp_elim(&imp, &refl(&n(0)).unwrap()).unwrap();
        assert_eq!(ax(&back), vec!["add_succ"]);
    }

    #[test]
    fn exists_elim_q_f_keeps_the_premise_gamma_and_discharges_only_h() {
        // Γ₁ mentions the eigenparameter a; the branch does not use it.
        let h0 = Prop::And(
            Box::new(Prop::eq(p("a"), p("a"))),
            Box::new(Prop::Exists(
                "x".into(),
                Box::new(Prop::eq(Term::Bound(0), n(1))),
            )),
        );
        let s1 = hyp("h0", &h0).unwrap();
        let s2 = and_right(&s1).unwrap();
        let w = hyp("w", &Prop::eq(p("a"), n(1))).unwrap();
        let s4 = exists_intro("y", &Prop::eq(Term::Bound(0), n(1)), &p("a"), &w).unwrap();
        let out = exists_elim(&s2, "a", "w", &s4).unwrap();
        assert_eq!(
            out.assumptions().keys().collect::<Vec<_>>(),
            vec!["h0"],
            "Γ₁ passes through; only w is discharged"
        );
        // The same branch also using h0 (which mentions a) is refused.
        let s5 = and_intro(&s4, &s2).unwrap();
        assert!(exists_elim(&s2, "a", "w", &s5)
            .unwrap_err()
            .contains("occurs in Γ of the branch"));
        // A name present in both premises survives the discharge from the branch.
        let w_outside = and_intro(&s2, &hyp("w", &Prop::eq(p("a"), n(1))).unwrap()).unwrap();
        let ex = and_left(&w_outside).unwrap();
        let out = exists_elim(&ex, "a", "w", &s4).unwrap();
        assert!(
            out.assumptions().contains_key("w"),
            "w from the existential premise is kept"
        );
    }

    #[test]
    fn subst_uses_the_motive_both_ways_and_cannot_capture() {
        let eq = hyp("h", &Prop::eq(n(0), p("c"))).unwrap();
        let body = Prop::Forall(
            "v".into(),
            Box::new(Prop::eq(
                Term::plus(Term::Bound(0), n(0)),
                Term::plus(Term::Bound(0), n(0)),
            )),
        );
        let prem = forall_intro("b", "v", &refl(&Term::plus(p("b"), n(0))).unwrap()).unwrap();
        assert_eq!(prem.conclusion(), &body);
        // motive: ∀v. v + u = v + 0, u the distinguished variable (index 1 inside v)
        let motive = Prop::Forall(
            "v".into(),
            Box::new(Prop::eq(
                Term::plus(Term::Bound(0), Term::Bound(1)),
                Term::plus(Term::Bound(0), n(0)),
            )),
        );
        let out = subst(&eq, &motive, &prem).unwrap();
        assert_eq!(out.conclusion().to_string(), "∀v. v + c = v + 0");
        // vacuous motive and identical sides restate the premise
        let same = subst(
            &refl(&n(2)).unwrap(),
            &Prop::eq(n(2), n(2)),
            &refl(&n(2)).unwrap(),
        )
        .unwrap();
        assert_eq!(same.conclusion(), &Prop::eq(n(2), n(2)));
        assert!(
            subst(&eq, &Prop::eq(Term::Bound(1), n(0)), &prem).is_err(),
            "motive must be locally closed under one binder"
        );
    }
}

//! The proof checker: replays a proof object step by step through the rules.
//!
//! A proof object is untrusted input (from a parser, a person or a language
//! model). Each step names a rule, its premises and its arguments, and claims
//! a conclusion. The checker recomputes the conclusion with the rule and
//! requires the claim to be structurally identical to it. A rule name outside
//! the registered rules is rejected, whatever the claim says.
//!
//! RLT-003 M3a: claims, goals and assumptions are formulas (`Prop`). Local
//! assumptions are names that some discharging step (imp-intro, or-cases,
//! exists-elim) of the same proof names; each is bound to one formula per
//! proof, never shares a name with a declared assumption, and must be
//! discharged before the end. Every check on the final Γ is as before.

use crate::prop::Prop;
use crate::rules::{self, Axiom, Theorem};
use crate::term::Term;
use std::collections::{BTreeMap, BTreeSet};

/// What the goal asks to be established.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Accept only with an empty assumption context Γ.
    Unconditional,
    /// Accept with Γ ⊆ the declared assumptions; Γ is reported.
    Conditional,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Arg {
    Step(String),
    Term(Term),
    Bind(String, Term),
    Name(String),
    /// A substitution motive `(at x ψ)`: binder hint and body (x is `Bound(0)`).
    Motive(String, Prop),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    pub label: String,
    pub claim: Prop,
    pub rule: String,
    pub args: Vec<Arg>,
}

#[derive(Clone, Debug)]
pub struct ProofObject {
    pub goal: Prop,
    pub mode: Mode,
    pub assumptions: Vec<(String, Prop)>,
    /// Declared parameters (eigenvariables), `(params …)`; RLT-003 M3a.
    pub params: Vec<String>,
    pub steps: Vec<Step>,
}

#[derive(Clone, Debug)]
pub enum Verdict {
    Accepted(Theorem),
    Rejected {
        step: Option<String>,
        reason: String,
    },
}

/// The registered rule names. Anything else is rejected.
pub const RULES: [&str; 20] = [
    "refl",
    "symm",
    "trans",
    "cong",
    "inst",
    "hyp",
    "subst",
    "and-intro",
    "and-left",
    "and-right",
    "or-left",
    "or-right",
    "or-cases",
    "imp-intro",
    "imp-elim",
    "false-elim",
    "forall-intro",
    "forall-elim",
    "exists-intro",
    "exists-elim",
];

/// For each discharging rule, the argument positions that name assumptions.
fn discharged_names(step: &Step) -> Vec<&str> {
    let pos: &[usize] = match step.rule.as_str() {
        "imp-intro" => &[0],
        "or-cases" => &[1, 3],
        "exists-elim" => &[2],
        _ => &[],
    };
    pos.iter()
        .filter_map(|&i| match step.args.get(i) {
            Some(Arg::Name(n)) => Some(n.as_str()),
            _ => None,
        })
        .collect()
}

fn reject(step: Option<&str>, reason: String) -> Verdict {
    Verdict::Rejected {
        step: step.map(|s| s.to_string()),
        reason,
    }
}

fn premise<'a>(done: &'a BTreeMap<String, Theorem>, arg: &Arg) -> Result<&'a Theorem, String> {
    match arg {
        Arg::Step(l) => done
            .get(l)
            .ok_or_else(|| format!("premise `{l}` is not an earlier step")),
        other => Err(format!("expected an earlier step label, found {other:?}")),
    }
}

/// The (name, formula) pairs an accepted discharging step discharges.
fn discharged_formulas(step: &Step, done: &BTreeMap<String, Theorem>) -> Vec<(String, Prop)> {
    let concl = |a: Option<&Arg>| match a {
        Some(Arg::Step(l)) => done.get(l).map(|t| t.conclusion().clone()),
        _ => None,
    };
    let name = |a: Option<&Arg>| match a {
        Some(Arg::Name(n)) => Some(n.clone()),
        _ => None,
    };
    let a = &step.args;
    match step.rule.as_str() {
        "imp-intro" => match (&step.claim, name(a.first())) {
            (Prop::Imp(phi, _), Some(h)) => vec![(h, (**phi).clone())],
            _ => vec![],
        },
        "or-cases" => match (concl(a.first()), name(a.get(1)), name(a.get(3))) {
            (Some(Prop::Or(l, r)), Some(h1), Some(h2)) => {
                vec![(h1, (*l).clone()), (h2, (*r).clone())]
            }
            _ => vec![],
        },
        "exists-elim" => match (concl(a.first()), name(a.get(1)), name(a.get(2))) {
            (Some(Prop::Exists(_, body)), Some(p), Some(h)) => {
                vec![(h, body.open(&Term::Param(p)))]
            }
            _ => vec![],
        },
        _ => vec![],
    }
}

struct Ctx<'a> {
    done: &'a BTreeMap<String, Theorem>,
    axioms: &'a BTreeMap<String, Axiom>,
    declared: &'a BTreeMap<String, Prop>,
    locals: &'a BTreeSet<String>,
    params: &'a BTreeSet<String>,
}

fn name_arg<'a>(arg: Option<&'a Arg>, what: &str) -> Result<&'a str, String> {
    match arg {
        Some(Arg::Name(n)) => Ok(n),
        other => Err(format!("expected {what}, found {other:?}")),
    }
}

fn param_arg<'a>(ctx: &Ctx, arg: Option<&'a Arg>) -> Result<&'a str, String> {
    let a = name_arg(arg, "a parameter name")?;
    if !ctx.params.contains(a) {
        return Err(format!("parameter `{a}` is undeclared"));
    }
    Ok(a)
}

fn local_arg<'a>(ctx: &Ctx, arg: Option<&'a Arg>) -> Result<&'a str, String> {
    let h = name_arg(arg, "an assumption name")?;
    if ctx.declared.contains_key(h) {
        return Err(format!(
            "local name `{h}` clashes with a declared assumption; declared assumptions cannot be discharged"
        ));
    }
    Ok(h)
}

fn term_arg(arg: Option<&Arg>) -> Result<&Term, String> {
    match arg {
        Some(Arg::Term(t)) => Ok(t),
        other => Err(format!("expected a term, found {other:?}")),
    }
}

fn arity_is(rule: &str, a: &[Arg], n: usize) -> Result<(), String> {
    if a.len() == n {
        Ok(())
    } else {
        Err(format!("{rule} takes {n} argument(s), given {}", a.len()))
    }
}

fn apply(step: &Step, ctx: &Ctx) -> Result<Theorem, String> {
    let a = &step.args;
    let done = ctx.done;
    let claim = &step.claim;
    let prem = |i: usize| -> Result<&Theorem, String> {
        match a.get(i) {
            Some(p) => premise(done, p),
            None => Err(format!("{}: missing premise {}", step.rule, i + 1)),
        }
    };
    match step.rule.as_str() {
        "refl" => match a.as_slice() {
            [Arg::Term(t)] => rules::refl(t),
            _ => Err("refl takes one term".into()),
        },
        "symm" => match a.as_slice() {
            [p] => rules::symm(premise(done, p)?),
            _ => Err("symm takes one premise".into()),
        },
        "trans" => match a.as_slice() {
            [p, q] => rules::trans(premise(done, p)?, premise(done, q)?),
            _ => Err("trans takes two premises".into()),
        },
        "cong" => match a.split_first() {
            Some((Arg::Name(f), rest)) => {
                let ths = rest
                    .iter()
                    .map(|p| premise(done, p))
                    .collect::<Result<Vec<_>, _>>()?;
                rules::cong(f, &ths)
            }
            _ => Err("cong takes a symbol and its premises".into()),
        },
        "inst" => match a.split_first() {
            Some((Arg::Name(ax), rest)) => {
                let axiom = ctx
                    .axioms
                    .get(ax)
                    .ok_or_else(|| format!("axiom `{ax}` is unregistered"))?;
                let binds = rest
                    .iter()
                    .map(|b| match b {
                        Arg::Bind(v, t) => Ok((v.clone(), t.clone())),
                        other => Err(format!("inst expects bindings, found {other:?}")),
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                rules::inst(axiom, &binds)
            }
            _ => Err("inst takes an axiom name and bindings".into()),
        },
        "hyp" => match a.as_slice() {
            [Arg::Name(h)] => {
                if let Some(phi) = ctx.declared.get(h) {
                    rules::hyp(h, phi)
                } else if ctx.locals.contains(h) {
                    // A local assumption: its formula is the step's claim.
                    rules::hyp(h, claim)
                } else {
                    Err(format!("assumption `{h}` is undeclared in the goal"))
                }
            }
            _ => Err("hyp takes one assumption name".into()),
        },
        "subst" => {
            arity_is("subst", a, 3)?;
            let (_, motive) = match a.get(1) {
                Some(Arg::Motive(h, m)) => (h, m),
                other => {
                    return Err(format!(
                        "subst: expected a motive (at x ψ), found {other:?}"
                    ))
                }
            };
            rules::subst(prem(0)?, motive, prem(2)?)
        }
        "and-intro" => {
            arity_is("and-intro", a, 2)?;
            rules::and_intro(prem(0)?, prem(1)?)
        }
        "and-left" => {
            arity_is("and-left", a, 1)?;
            rules::and_left(prem(0)?)
        }
        "and-right" => {
            arity_is("and-right", a, 1)?;
            rules::and_right(prem(0)?)
        }
        "or-left" => {
            arity_is("or-left", a, 1)?;
            match claim {
                Prop::Or(_, r) => rules::or_left(prem(0)?, r),
                _ => Err("or-left: the stated formula is not a disjunction".into()),
            }
        }
        "or-right" => {
            arity_is("or-right", a, 1)?;
            match claim {
                Prop::Or(l, _) => rules::or_right(l, prem(0)?),
                _ => Err("or-right: the stated formula is not a disjunction".into()),
            }
        }
        "or-cases" => {
            arity_is("or-cases", a, 5)?;
            let h1 = local_arg(ctx, a.get(1))?;
            let h2 = local_arg(ctx, a.get(3))?;
            rules::or_cases(prem(0)?, h1, prem(2)?, h2, prem(4)?)
        }
        "imp-intro" => {
            arity_is("imp-intro", a, 2)?;
            let h = local_arg(ctx, a.first())?;
            match claim {
                Prop::Imp(ante, _) => rules::imp_intro(h, ante, prem(1)?),
                _ => Err("imp-intro: the stated formula is not an implication".into()),
            }
        }
        "imp-elim" => {
            arity_is("imp-elim", a, 2)?;
            rules::imp_elim(prem(0)?, prem(1)?)
        }
        "false-elim" => {
            arity_is("false-elim", a, 1)?;
            rules::false_elim(prem(0)?, claim)
        }
        "forall-intro" => {
            arity_is("forall-intro", a, 2)?;
            let p = param_arg(ctx, a.first())?;
            let hint = match claim {
                Prop::Forall(h, _) => h.as_str(),
                _ => return Err("forall-intro: the stated formula is not a ∀ formula".into()),
            };
            rules::forall_intro(p, hint, prem(1)?)
        }
        "forall-elim" => {
            arity_is("forall-elim", a, 2)?;
            rules::forall_elim(prem(0)?, term_arg(a.get(1))?)
        }
        "exists-intro" => {
            arity_is("exists-intro", a, 2)?;
            match claim {
                Prop::Exists(h, body) => {
                    rules::exists_intro(h, body, term_arg(a.first())?, prem(1)?)
                }
                _ => Err("exists-intro: the stated formula is not an ∃ formula".into()),
            }
        }
        "exists-elim" => {
            arity_is("exists-elim", a, 4)?;
            let p = param_arg(ctx, a.get(1))?;
            let h = local_arg(ctx, a.get(2))?;
            rules::exists_elim(prem(0)?, p, h, prem(3)?)
        }
        other => Err(format!(
            "rule `{other}` is unregistered (registered: {})",
            RULES.join(", ")
        )),
    }
}

/// Check a proof object against the registered axioms.
pub fn check(proof: &ProofObject, axioms: &[Axiom]) -> Verdict {
    let mut registered: BTreeMap<String, Axiom> = BTreeMap::new();
    for a in axioms {
        if RULES.contains(&a.name()) {
            return reject(
                None,
                format!("axiom name `{}` collides with a rule name", a.name()),
            );
        }
        if registered.insert(a.name().to_string(), a.clone()).is_some() {
            return reject(None, format!("axiom `{}` registered twice", a.name()));
        }
    }
    let axioms = registered;

    if let Err(e) = proof.goal.check_wf(0) {
        return reject(None, format!("goal: {e}"));
    }
    let mut goal_params = BTreeSet::new();
    proof.goal.params(&mut goal_params);
    if let Some(p) = goal_params.iter().next() {
        return reject(None, format!("goal: parameter `{p}` occurs in the goal"));
    }
    let mut declared = BTreeMap::new();
    for (name, f) in &proof.assumptions {
        if let Err(e) = f.check_wf(0) {
            return reject(None, format!("assumption `{name}`: {e}"));
        }
        let mut ps = BTreeSet::new();
        f.params(&mut ps);
        if let Some(p) = ps.iter().next() {
            return reject(
                None,
                format!("assumption `{name}`: parameter `{p}` occurs in a declared assumption"),
            );
        }
        if declared.insert(name.clone(), f.clone()).is_some() {
            return reject(None, format!("assumption `{name}` declared twice"));
        }
        if axioms.contains_key(name) || RULES.contains(&name.as_str()) {
            return reject(
                None,
                format!("assumption name `{name}` collides with an axiom or rule name"),
            );
        }
    }
    let mut params = BTreeSet::new();
    for p in &proof.params {
        if !params.insert(p.clone()) {
            return reject(None, format!("parameter `{p}` declared twice"));
        }
        if axioms.contains_key(p) || RULES.contains(&p.as_str()) || declared.contains_key(p) {
            return reject(
                None,
                format!("parameter name `{p}` collides with an axiom, rule or assumption name"),
            );
        }
    }
    // Local assumptions: the names some discharging step of this proof names.
    let locals: BTreeSet<String> = proof
        .steps
        .iter()
        .flat_map(discharged_names)
        .filter(|n| !declared.contains_key(*n))
        .map(str::to_string)
        .collect();
    if let Some(n) = locals
        .iter()
        .find(|n| params.contains(*n) || axioms.contains_key(*n) || RULES.contains(&n.as_str()))
    {
        return reject(
            None,
            format!("local name `{n}` collides with a parameter, axiom or rule name"),
        );
    }
    let mut local_formula: BTreeMap<String, Prop> = BTreeMap::new();
    let mut done: BTreeMap<String, Theorem> = BTreeMap::new();
    let mut last: Option<Theorem> = None;
    for step in &proof.steps {
        let label = Some(step.label.as_str());
        if done.contains_key(&step.label) {
            return reject(label, format!("step label `{}` reused", step.label));
        }
        // A step's claim needs no separate check: it is compared with a
        // well-formed conclusion, and the rules that read a formula from it
        // (hyp for local assumptions, or-left, or-right, imp-intro,
        // false-elim, exists-intro) check it themselves.
        if step.rule == "hyp" {
            if let [Arg::Name(h)] = step.args.as_slice() {
                if locals.contains(h) {
                    match local_formula.get(h) {
                        Some(f) if f != &step.claim => {
                            return reject(
                                label,
                                format!(
                                    "local name `{h}` is bound to two formulas: `{f}` and `{}`",
                                    step.claim
                                ),
                            )
                        }
                        Some(_) => {}
                        None => {
                            local_formula.insert(h.clone(), step.claim.clone());
                        }
                    }
                }
            }
        }
        let ctx = Ctx {
            done: &done,
            axioms: &axioms,
            declared: &declared,
            locals: &locals,
            params: &params,
        };
        let th = match apply(step, &ctx) {
            Ok(th) => th,
            Err(e) => return reject(label, e),
        };
        if th.conclusion() != &step.claim {
            return reject(
                label,
                format!(
                    "step claims `{}`, rule `{}` yields `{}`",
                    step.claim,
                    step.rule,
                    th.conclusion()
                ),
            );
        }
        // One formula per local name also at discharges (audit at 6ba9f1e,
        // weakness 2): the formula a discharging step requires for its name
        // must agree with every other use of that name, even when the
        // discharge is vacuous.
        for (h, f) in discharged_formulas(step, &done) {
            if !locals.contains(&h) {
                continue;
            }
            match local_formula.get(&h) {
                Some(g) if g != &f => {
                    return reject(
                        label,
                        format!("local name `{h}` is bound to two formulas: `{g}` and `{f}`"),
                    )
                }
                Some(_) => {}
                None => {
                    local_formula.insert(h, f);
                }
            }
        }
        done.insert(step.label.clone(), th.clone());
        last = Some(th);
    }

    let Some(th) = last else {
        return reject(None, "the proof has zero steps".into());
    };
    if th.conclusion() != &proof.goal {
        return reject(
            None,
            format!(
                "last step proves `{}`, the goal is `{}`",
                th.conclusion(),
                proof.goal
            ),
        );
    }
    if let Some(h) = th.assumptions().keys().find(|h| !declared.contains_key(*h)) {
        return reject(
            None,
            format!(
                "local assumption `{h}` remains in Γ: it is undeclared and was never discharged"
            ),
        );
    }
    if proof.mode == Mode::Unconditional && !th.is_unconditional() {
        let names: Vec<&String> = th.assumptions().keys().collect();
        return reject(
            None,
            format!(
                "unconditional goal requested, but the result depends on assumptions {names:?}"
            ),
        );
    }
    Verdict::Accepted(th)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::term::Formula;

    fn n(k: u64) -> Term {
        Term::numeral(k)
    }
    fn axioms() -> Vec<Axiom> {
        let (x, y) = (Term::Var("x".into()), Term::Var("y".into()));
        vec![
            Axiom::new(
                "add_zero",
                vec!["x".into()],
                Formula::new(Term::plus(x.clone(), Term::zero()), x.clone()),
                "test",
            )
            .unwrap(),
            Axiom::new(
                "add_succ",
                vec!["x".into(), "y".into()],
                Formula::new(
                    Term::plus(x.clone(), Term::succ(y.clone())),
                    Term::succ(Term::plus(x, y)),
                ),
                "test",
            )
            .unwrap(),
        ]
    }
    fn step(label: &str, claim: Formula, rule: &str, args: Vec<Arg>) -> Step {
        Step {
            label: label.into(),
            claim: Prop::Eq(claim),
            rule: rule.into(),
            args,
        }
    }
    fn t1_steps() -> Vec<Step> {
        vec![
            step(
                "s1",
                Formula::new(Term::plus(n(5), n(1)), Term::succ(Term::plus(n(5), n(0)))),
                "inst",
                vec![
                    Arg::Name("add_succ".into()),
                    Arg::Bind("x".into(), n(5)),
                    Arg::Bind("y".into(), n(0)),
                ],
            ),
            step(
                "s2",
                Formula::new(Term::plus(n(5), n(0)), n(5)),
                "inst",
                vec![Arg::Name("add_zero".into()), Arg::Bind("x".into(), n(5))],
            ),
            step(
                "s3",
                Formula::new(Term::succ(Term::plus(n(5), n(0))), n(6)),
                "cong",
                vec![Arg::Name("S".into()), Arg::Step("s2".into())],
            ),
            step(
                "s4",
                Formula::new(Term::plus(n(5), n(1)), n(6)),
                "trans",
                vec![Arg::Step("s1".into()), Arg::Step("s3".into())],
            ),
        ]
    }
    fn po(
        goal: Formula,
        mode: Mode,
        assumptions: Vec<(String, Formula)>,
        steps: Vec<Step>,
    ) -> ProofObject {
        ProofObject {
            goal: Prop::Eq(goal),
            mode,
            assumptions: assumptions
                .into_iter()
                .map(|(n, f)| (n, Prop::Eq(f)))
                .collect(),
            params: Vec::new(),
            steps,
        }
    }
    fn goal() -> Formula {
        Formula::new(Term::plus(n(5), n(1)), n(6))
    }
    fn rejected(v: Verdict) -> String {
        match v {
            Verdict::Rejected { reason, .. } => reason,
            Verdict::Accepted(_) => panic!("expected rejection"),
        }
    }

    #[test]
    fn t1_accepts_unconditionally() {
        match check(
            &po(goal(), Mode::Unconditional, vec![], t1_steps()),
            &axioms(),
        ) {
            Verdict::Accepted(th) => assert!(th.is_unconditional()),
            v => panic!("{v:?}"),
        }
    }

    #[test]
    fn a_claim_differing_from_the_rule_result_is_rejected() {
        let mut s = t1_steps();
        s[1].claim = Prop::eq(Term::plus(n(5), n(0)), n(4));
        assert!(rejected(check(
            &po(goal(), Mode::Unconditional, vec![], s),
            &axioms()
        ))
        .contains("yields"));
    }

    #[test]
    fn a_proof_of_another_goal_is_rejected() {
        let other = Formula::new(Term::plus(n(5), n(1)), n(7));
        assert!(rejected(check(
            &po(other, Mode::Unconditional, vec![], t1_steps()),
            &axioms()
        ))
        .contains("goal"));
    }

    #[test]
    fn unregistered_rules_labels_axioms_and_assumptions_are_rejected() {
        let calc = vec![step("s1", goal(), "calc", vec![])];
        assert!(rejected(check(
            &po(goal(), Mode::Unconditional, vec![], calc),
            &axioms()
        ))
        .contains("unregistered"));

        let mut reuse = t1_steps();
        reuse[1].label = "s1".into();
        assert!(rejected(check(
            &po(goal(), Mode::Unconditional, vec![], reuse),
            &axioms()
        ))
        .contains("reused"));

        let mut later = t1_steps();
        later[3].args = vec![Arg::Step("s1".into()), Arg::Step("s9".into())];
        assert!(rejected(check(
            &po(goal(), Mode::Unconditional, vec![], later),
            &axioms()
        ))
        .contains("earlier step"));

        let undeclared = vec![step("s1", goal(), "hyp", vec![Arg::Name("h".into())])];
        assert!(rejected(check(
            &po(goal(), Mode::Conditional, vec![], undeclared),
            &axioms()
        ))
        .contains("undeclared"));

        let mut twice = axioms();
        twice.push(twice[0].clone());
        assert!(rejected(check(
            &po(goal(), Mode::Unconditional, vec![], t1_steps()),
            &twice
        ))
        .contains("twice"));
    }

    #[test]
    fn unconditional_requires_empty_context_and_conditional_reports_it() {
        let h = vec![("h".to_string(), goal())];
        let by_hyp = vec![step("s1", goal(), "hyp", vec![Arg::Name("h".into())])];
        assert!(rejected(check(
            &po(goal(), Mode::Unconditional, h.clone(), by_hyp.clone()),
            &axioms()
        ))
        .contains("depends on assumptions"));
        match check(&po(goal(), Mode::Conditional, h.clone(), by_hyp), &axioms()) {
            Verdict::Accepted(th) => assert!(!th.is_unconditional()),
            v => panic!("{v:?}"),
        }
        // Declared but unused: the result is unconditional, and reported so.
        match check(&po(goal(), Mode::Conditional, h, t1_steps()), &axioms()) {
            Verdict::Accepted(th) => assert!(th.is_unconditional()),
            v => panic!("{v:?}"),
        }
    }

    #[test]
    fn an_empty_proof_is_rejected() {
        assert!(rejected(check(
            &po(goal(), Mode::Unconditional, vec![], vec![]),
            &axioms()
        ))
        .contains("zero steps"));
    }
}

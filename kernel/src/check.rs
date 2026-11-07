//! The proof checker: replays a proof object step by step through the rules.
//!
//! A proof object is untrusted input (from a parser, a person or a language
//! model). Each step names a rule, its premises and its arguments, and claims
//! a conclusion. The checker recomputes the conclusion with the rule and
//! requires the claim to be structurally identical to it. A rule name outside
//! the registered six is rejected, whatever the claim says.

use crate::rules::{self, Axiom, Theorem};
use crate::term::{Formula, Term};
use std::collections::BTreeMap;

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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    pub label: String,
    pub claim: Formula,
    pub rule: String,
    pub args: Vec<Arg>,
}

#[derive(Clone, Debug)]
pub struct ProofObject {
    pub goal: Formula,
    pub mode: Mode,
    pub assumptions: Vec<(String, Formula)>,
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
pub const RULES: [&str; 6] = ["refl", "symm", "trans", "cong", "inst", "hyp"];

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

fn apply(
    step: &Step,
    done: &BTreeMap<String, Theorem>,
    axioms: &BTreeMap<String, Axiom>,
    declared: &BTreeMap<String, Formula>,
) -> Result<Theorem, String> {
    let a = &step.args;
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
                let axiom = axioms
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
                let phi = declared
                    .get(h)
                    .ok_or_else(|| format!("assumption `{h}` is undeclared in the goal"))?;
                rules::hyp(h, phi)
            }
            _ => Err("hyp takes one assumption name".into()),
        },
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
        if registered.insert(a.name().to_string(), a.clone()).is_some() {
            return reject(None, format!("axiom `{}` registered twice", a.name()));
        }
    }
    let axioms = registered;

    if let Err(e) = proof.goal.check_sort(false) {
        return reject(None, format!("goal: {e}"));
    }
    let mut declared = BTreeMap::new();
    for (name, f) in &proof.assumptions {
        if let Err(e) = f.check_sort(false) {
            return reject(None, format!("assumption `{name}`: {e}"));
        }
        if declared.insert(name.clone(), f.clone()).is_some() {
            return reject(None, format!("assumption `{name}` declared twice"));
        }
    }
    let mut done: BTreeMap<String, Theorem> = BTreeMap::new();
    let mut last: Option<Theorem> = None;
    for step in &proof.steps {
        let label = Some(step.label.as_str());
        if done.contains_key(&step.label) {
            return reject(label, format!("step label `{}` reused", step.label));
        }
        let th = match apply(step, &done, &axioms, &declared) {
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
            claim,
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
            goal,
            mode,
            assumptions,
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
        s[1].claim = Formula::new(Term::plus(n(5), n(0)), n(4));
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

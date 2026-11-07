//! Proof files (`*.rlt`) and the axiom file, read into kernel proof objects.
//!
//! This reader is untrusted: it only builds the proof object that the kernel
//! replays. Its one interpretive act — elaborating the written goal and
//! written assumptions under the named convention — is reported verbatim.

use crate::elab::{elaborate_formula, Convention};
use crate::sexpr::{parse_all, Sexp};
use crate::written::Written;
use rlt_kernel::{Arg, Axiom, Formula, Mode, ProofObject, Step, Term};

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
    pub axioms_file: String,
    pub steps: Vec<Step>,
}

/// Largest numeral accepted in proof syntax (kernel numerals are unary).
pub const MAX_NUMERAL: u64 = 10_000;

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

fn step(e: &Sexp) -> Result<Step, String> {
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
    let claim = formula(claim, &[]).map_err(|m| format!("step {label}: {m}"))?;
    let (rule_name, args) = rule
        .head()
        .ok_or(format!("step {label}: expected (rule args…)"))?;
    let mut out = Vec::new();
    for (i, a) in args.iter().enumerate() {
        let arg = match (rule_name, i, a) {
            ("refl", _, a) => Arg::Term(term(a, &[])?),
            ("inst" | "cong" | "hyp", 0, Sexp::Sym(s)) => Arg::Name(s.clone()),
            ("inst", _, Sexp::List(b)) => match b.as_slice() {
                [Sexp::Sym(v), t] => Arg::Bind(v.clone(), term(t, &[])?),
                _ => return Err(format!("step {label}: inst binding must be (var term)")),
            },
            (_, _, Sexp::Sym(s)) => Arg::Step(s.clone()),
            (_, _, other) => Arg::Term(term(other, &[])?),
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

pub fn parse_proof_file(src: &str) -> Result<ProofFile, String> {
    let top = parse_all(src)?;
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
        axioms_file: String::new(),
        steps: Vec::new(),
    };
    let mut mode_seen = false;
    for f in fields {
        let (key, rest) = f
            .head()
            .ok_or(format!("field: expected (key value…), found {f:?}"))?;
        match key {
            "id" => pf.id = string_of(rest, key)?,
            "readingpoint" => pf.readingpoint = Some(string_of(rest, key)?),
            "relationship" => pf.relationship = Some(string_of(rest, key)?),
            "convention" => pf.convention = string_of(rest, key)?,
            "axioms" => pf.axioms_file = string_of(rest, key)?,
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
                [Sexp::Sym(name), Sexp::Str(w)] => {
                    pf.assumptions.push((name.clone(), Written::new(w)))
                }
                _ => return Err("(assume NAME \"written formula\")".into()),
            },
            "steps" => pf.steps = rest.iter().map(step).collect::<Result<_, _>>()?,
            other => return Err(format!("unknown field `{other}`")),
        }
    }
    for (k, empty) in [
        ("id", pf.id.is_empty()),
        ("convention", pf.convention.is_empty()),
        ("axioms", pf.axioms_file.is_empty()),
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
        let goal = elaborate_formula(&self.goal, conv).map_err(|e| format!("goal: {e}"))?;
        let assumptions = self
            .assumptions
            .iter()
            .map(|(n, w)| {
                elaborate_formula(w, conv)
                    .map(|f| (n.clone(), f))
                    .map_err(|e| format!("assume {n}: {e}"))
            })
            .collect::<Result<_, _>>()?;
        Ok(ProofObject {
            goal,
            mode: self.mode,
            assumptions,
            steps: self.steps.clone(),
        })
    }
}

/// `(axiom NAME (vars…) (= lhs rhs) (source ID))`
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
            Axiom::new(name, vars.clone(), formula(eq, &vars)?, &source)
        })
        .collect()
}

//! `rlt gen` — an UNTRUSTED proof generator (RLT-003 M1).
//!
//! Given a closed NAT-DEC equation such as `30 = 2 × 3 × 5`, it writes a proof
//! file whose steps use only the six kernel rules and the axioms of
//! PEANO-ARITH. Nothing here is trusted: every generated step is checked by
//! the kernel exactly as a hand-written proof is, and a wrong step is
//! rejected. The generator evaluates both sides in ℕ to decide which steps to
//! write; when the sides differ it writes no proof.

use crate::pinned_conventions;
use rlt_kernel::{Formula, Term};
use rlt_syntax::elab::elaborate_formula;
use rlt_syntax::written::Written;
use std::fmt::Write as _;
use std::path::Path;

/// Largest numeral the generator writes (the proof syntax limit).
const MAX: u64 = rlt_syntax::proof::MAX_NUMERAL;

/// Proof-syntax rendering of a term: numerals as integers.
fn sx(t: &Term) -> String {
    if let Some(n) = t.as_numeral() {
        return n.to_string();
    }
    match t {
        Term::App(f, args) if args.is_empty() => f.clone(),
        Term::App(f, args) => {
            let inner: Vec<String> = args.iter().map(sx).collect();
            format!("({f} {})", inner.join(" "))
        }
        Term::Var(v) => v.clone(),
    }
}

fn num(n: u64) -> Term {
    Term::numeral(n)
}

fn op(f: &str, a: Term, b: Term) -> Term {
    Term::App(f.into(), vec![a, b])
}

struct Gen {
    out: String,
    n: usize,
}

impl Gen {
    /// Write one step proving `lhs = rhs` by `rule`; return its label.
    fn step(&mut self, lhs: &Term, rhs: &Term, rule: String) -> String {
        self.n += 1;
        let label = format!("s{}", self.n);
        let _ = writeln!(self.out, "    ({label} (= {} {}) {rule})", sx(lhs), sx(rhs));
        label
    }

    /// x + y = (x+y), by add_succ y times and add_zero once.
    fn add(&mut self, x: u64, y: u64) -> Result<String, String> {
        if x + y > MAX {
            return Err(format!("{x} + {y} exceeds the numeral limit {MAX}"));
        }
        let lhs = op("+", num(x), num(y));
        if y == 0 {
            return Ok(self.step(&lhs, &num(x), format!("(inst add_zero (x {x}))")));
        }
        // x + S(y-1) = S(x + (y-1))
        let inner = op("+", num(x), num(y - 1));
        let s1 = self.step(
            &lhs,
            &Term::succ(inner.clone()),
            format!("(inst add_succ (x {x}) (y {}))", y - 1),
        );
        let r = self.add(x, y - 1)?;
        let c = self.step(&Term::succ(inner), &num(x + y), format!("(cong S {r})"));
        Ok(self.step(&lhs, &num(x + y), format!("(trans {s1} {c})")))
    }

    /// x × y = (x·y), by mul_succ y times, mul_zero once, and additions.
    fn mul(&mut self, x: u64, y: u64) -> Result<String, String> {
        if x.saturating_mul(y) > MAX {
            return Err(format!("{x} × {y} exceeds the numeral limit {MAX}"));
        }
        let lhs = op("*", num(x), num(y));
        if y == 0 {
            return Ok(self.step(&lhs, &num(0), format!("(inst mul_zero (x {x}))")));
        }
        let prev = op("*", num(x), num(y - 1));
        let p = x * (y - 1);
        // x × S(y-1) = (x × (y-1)) + x
        let s1 = self.step(
            &lhs,
            &op("+", prev.clone(), num(x)),
            format!("(inst mul_succ (x {x}) (y {}))", y - 1),
        );
        let r = self.mul(x, y - 1)?;
        let rx = self.step(&num(x), &num(x), format!("(refl {x})"));
        let c = self.step(
            &op("+", prev, num(x)),
            &op("+", num(p), num(x)),
            format!("(cong + {r} {rx})"),
        );
        let a = self.add(p, x)?;
        let t1 = self.step(
            &op("+", op("*", num(x), num(y - 1)), num(x)),
            &num(p + x),
            format!("(trans {c} {a})"),
        );
        Ok(self.step(&lhs, &num(x * y), format!("(trans {s1} {t1})")))
    }

    /// Prove `t = n` for the value n of t. Returns (label, n); the label is
    /// None when t is already the numeral n.
    fn eval(&mut self, t: &Term) -> Result<(Option<String>, u64), String> {
        if let Some(n) = t.as_numeral() {
            return Ok((None, n));
        }
        let Term::App(f, args) = t else {
            return Err("variables are outside closed equations".into());
        };
        let (a, b) = match args.as_slice() {
            [a, b] if f == "+" || f == "*" => (a, b),
            _ => return Err(format!("symbol `{f}` is outside closed NAT-DEC arithmetic")),
        };
        let (pa, na) = self.eval(a)?;
        let (pb, nb) = self.eval(b)?;
        let reduced = op(f, num(na), num(nb));
        // t = f(na, nb), by congruence when either side was reduced.
        let c = if pa.is_some() || pb.is_some() {
            let la = match pa {
                Some(l) => l,
                None => self.step(a, a, format!("(refl {})", sx(a))),
            };
            let lb = match pb {
                Some(l) => l,
                None => self.step(b, b, format!("(refl {})", sx(b))),
            };
            Some(self.step(t, &reduced, format!("(cong {f} {la} {lb})")))
        } else {
            None
        };
        let (k, v) = if f == "+" {
            (self.add(na, nb)?, na + nb)
        } else {
            (self.mul(na, nb)?, na * nb)
        };
        Ok(match c {
            Some(c) => (Some(self.step(t, &num(v), format!("(trans {c} {k})"))), v),
            None => (Some(k), v),
        })
    }
}

/// Generate a proof file for a closed NAT-DEC equation under PEANO-ARITH.
pub fn generate(root: &Path, written: &str, id: &str) -> Result<String, String> {
    if !id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || ".-_".contains(c))
        || id.is_empty()
    {
        return Err(format!(
            "proof id {id:?}: use ASCII letters, digits, `.`, `-`, `_`"
        ));
    }
    rlt_syntax::records::clean(written, "goal", "rlt gen", false)?;
    if written.contains('"') || written.contains('\\') {
        return Err("the written goal holds a quote or backslash".into());
    }
    let convs = pinned_conventions(root)?;
    let conv = convs
        .get("NAT-DEC")
        .ok_or("convention NAT-DEC is unregistered")?;
    let goal: Formula = elaborate_formula(&Written::new(written), &conv.convention)?;
    let mut g = Gen {
        out: String::new(),
        n: 0,
    };
    let (pl, nl) = g.eval(&goal.lhs)?;
    let (pr, nr) = g.eval(&goal.rhs)?;
    if nl != nr {
        return Err(format!(
            "`{goal}` is false in ℕ (left side {nl}, right side {nr}); no proof is generated"
        ));
    }
    match (pl, pr) {
        (None, None) => {
            g.step(&goal.lhs, &goal.rhs, format!("(refl {})", sx(&goal.lhs)));
        }
        (Some(_), None) => {}
        (None, Some(b)) => {
            g.step(&goal.lhs, &goal.rhs, format!("(symm {b})"));
        }
        (Some(a), Some(b)) => {
            let sb = g.step(&num(nr), &goal.rhs, format!("(symm {b})"));
            g.step(&goal.lhs, &goal.rhs, format!("(trans {a} {sb})"));
        }
    }
    let mut f = String::new();
    let _ = writeln!(
        f,
        "; Generated by `rlt gen` (untrusted). The kernel checks every step;"
    );
    let _ = writeln!(
        f,
        "; a wrong generated step is rejected like any hand-written one."
    );
    let _ = writeln!(f, "(rlt-proof");
    let _ = writeln!(f, "  (id {id})");
    let _ = writeln!(f, "  (convention NAT-DEC)");
    let _ = writeln!(f, "  (goal \"{written}\")");
    let _ = writeln!(f, "  (mode unconditional)");
    let _ = writeln!(f, "  (axioms PEANO-ARITH)");
    let _ = writeln!(f, "  (steps");
    f.push_str(&g.out);
    // close (steps and (rlt-proof
    while f.ends_with('\n') {
        f.pop();
    }
    f.push_str("))\n");
    Ok(f)
}

//! Layer 2 — conventions and elaboration.
//!
//! A convention names one reading of written notation. Elaboration maps
//! (written expression, convention) to a kernel term or formula. It is an
//! untrusted step: its output is printed in every report and then checked by
//! the kernel, which decides mathematical equality only through a proof.
//!
//! Elaborators in RLT-001:
//! - `nat-dec`: decimal numerals, `+` (left-associative), `=`, parentheses.
//! - `symbol`: a token-to-constant table taken from the convention file.
//! - `none`: the convention records interpretive content (e.g. PiπΠ) and
//!   elaborates nothing.

use crate::written::Written;
use rlt_kernel::{Formula, Prop, Term};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Convention {
    pub id: String,
    pub elaborator: String,
    pub symbols: BTreeMap<String, String>,
    /// NAT-DEC operators this convention reads: `+`, and `×` once listed.
    pub operators: Vec<String>,
    /// NAT-DEC logical symbols this convention reads (RLT-003 M3a), a subset
    /// of `CONNECTIVES`; empty means equations only, as before.
    pub connectives: Vec<String>,
    /// Definition notations this convention reads (RLT-003 M3b, NAT-DEF):
    /// `<`, `∣`, `free235`. Empty for every earlier convention.
    pub definitions: Vec<String>,
    /// The pinned definitions, attached by the caller when `definitions` is
    /// non-empty; never read from the convention file.
    pub defs: Option<std::rc::Rc<crate::defs::Definitions>>,
}

/// The logical symbols NAT-DEC can be configured to read (RLT-003 M3a).
pub const CONNECTIVES: [&str; 7] = ["⊥", "¬", "∧", "∨", "→", "∀", "∃"];

pub fn elaborate_term(w: &Written, conv: &Convention) -> Result<Term, String> {
    match conv.elaborator.as_str() {
        "nat-dec" => {
            let mut p = Parser {
                toks: &w.tokens,
                pos: 0,
                depth: 0,
                numeral_total: 0,
                times: conv.operators.iter().any(|o| o == "×"),
                operators: 0,
                vars: None,
                notations: Vec::new(),
                defs: None,
            };
            let t = p.expr()?;
            p.end()?;
            Ok(t)
        }
        "symbol" => match w.tokens.as_slice() {
            [tok] => conv
                .symbols
                .get(tok)
                .map(|c| Term::App(c.clone(), vec![]))
                .ok_or_else(|| format!("convention {}: token `{tok}` is unassigned", conv.id)),
            _ => Err(format!(
                "convention {}: elaborates single symbols only",
                conv.id
            )),
        },
        "residue-table" => Err(format!(
            "convention {} classifies residue witness records (rlt residues); it elaborates zero kernel terms",
            conv.id
        )),
        "species-table" => Err(format!(
            "convention {} reads chemical formulas as registered species (rlt chem); it elaborates zero kernel terms",
            conv.id
        )),
        other => Err(format!(
            "convention {} has elaborator `{other}`: it elaborates zero expressions",
            conv.id
        )),
    }
}

pub fn elaborate_formula(w: &Written, conv: &Convention) -> Result<Formula, String> {
    if conv.elaborator != "nat-dec" {
        return Err(format!(
            "convention {} elaborates terms only; equations use NAT-DEC",
            conv.id
        ));
    }
    let mut p = Parser {
        toks: &w.tokens,
        pos: 0,
        depth: 0,
        numeral_total: 0,
        times: conv.operators.iter().any(|o| o == "×"),
        operators: 0,
        vars: None,
        notations: Vec::new(),
        defs: None,
    };
    let lhs = p.expr()?;
    p.expect("=")?;
    let rhs = p.expr()?;
    p.end()?;
    Ok(Formula::new(lhs, rhs))
}

/// Maximum parenthesis depth and total numeral size accepted by NAT-DEC.
pub const MAX_PAREN_DEPTH: usize = 256;
pub const MAX_NUMERAL_TOTAL: u64 = 200_000;
/// Maximum number of `+` and `×` operators in one NAT-DEC expression; a
/// longer left-grouped chain would nest deeper than the checker's stack.
pub const MAX_OPERATORS: usize = 4_096;

struct Parser<'a> {
    toks: &'a [String],
    pos: usize,
    depth: usize,
    numeral_total: u64,
    /// Whether `×` is read (multiplication, binding tighter than `+`).
    times: bool,
    operators: usize,
    /// Bound variables in scope (innermost last) when reading a formula with
    /// quantifiers; `None` when reading an equation, exactly as before M3a.
    vars: Option<Vec<String>>,
    /// RLT-003 M3b: definition notations read (NAT-DEF) and the pinned
    /// definitions they expand to; empty and `None` for every other convention.
    notations: Vec<String>,
    defs: Option<std::rc::Rc<crate::defs::Definitions>>,
}

impl Parser<'_> {
    fn peek(&self) -> Option<&str> {
        self.toks.get(self.pos).map(|s| s.as_str())
    }
    fn expect(&mut self, t: &str) -> Result<(), String> {
        if self.peek() == Some(t) {
            self.pos += 1;
            Ok(())
        } else {
            Err(format!(
                "NAT-DEC: expected `{t}` at token {}, found {:?}",
                self.pos + 1,
                self.peek()
            ))
        }
    }
    fn end(&self) -> Result<(), String> {
        match self.peek() {
            None => Ok(()),
            Some(t) => Err(format!(
                "NAT-DEC: unexpected token `{t}` at position {}",
                self.pos + 1
            )),
        }
    }
    fn operator(&mut self) -> Result<(), String> {
        self.operators += 1;
        if self.operators > MAX_OPERATORS {
            return Err(format!(
                "NAT-DEC: more than {MAX_OPERATORS} operators in one expression"
            ));
        }
        self.pos += 1;
        Ok(())
    }
    fn expr(&mut self) -> Result<Term, String> {
        let mut t = self.product()?;
        while self.peek() == Some("+") {
            self.operator()?;
            let r = self.product()?;
            t = Term::plus(t, r);
        }
        Ok(t)
    }
    /// `a × b × c` reads `(a × b) × c`; `×` binds tighter than `+`.
    fn product(&mut self) -> Result<Term, String> {
        let mut t = self.atom()?;
        while self.peek() == Some("×") {
            if !self.times {
                return Err(
                    "NAT-DEC: `×` is outside the operators this convention reads".to_string(),
                );
            }
            self.operator()?;
            let r = self.atom()?;
            t = Term::App("*".into(), vec![t, r]);
        }
        Ok(t)
    }
    fn atom(&mut self) -> Result<Term, String> {
        match self.peek() {
            Some("(") => {
                if self.depth >= MAX_PAREN_DEPTH {
                    return Err(format!(
                        "NAT-DEC: parentheses nested deeper than {MAX_PAREN_DEPTH}"
                    ));
                }
                self.pos += 1;
                self.depth += 1;
                let t = self.expr()?;
                self.depth -= 1;
                self.expect(")")?;
                Ok(t)
            }
            Some(tok) if tok.len() > 1 && tok.starts_with('0') => Err(format!(
                "NAT-DEC: numeral `{tok}` has a leading zero; write it without one"
            )),
            Some(tok) if tok.chars().all(|c| c.is_ascii_digit()) => {
                let n: u64 = tok
                    .parse()
                    .map_err(|e| format!("NAT-DEC numeral `{tok}`: {e}"))?;
                if n > crate::proof::MAX_NUMERAL {
                    return Err(format!(
                        "NAT-DEC: numeral {n} exceeds the RLT-001 unary limit {}",
                        crate::proof::MAX_NUMERAL
                    ));
                }
                self.numeral_total += n;
                if self.numeral_total > MAX_NUMERAL_TOTAL {
                    return Err(format!(
                        "NAT-DEC: numerals total more than {MAX_NUMERAL_TOTAL} in one expression"
                    ));
                }
                self.pos += 1;
                Ok(Term::numeral(n))
            }
            Some(tok) if self.vars.is_some() && is_var(tok) => {
                let vars = self.vars.as_ref().expect("checked");
                match vars.iter().rposition(|v| v == tok) {
                    Some(k) => {
                        let i = vars.len() - 1 - k;
                        self.pos += 1;
                        Ok(Term::Bound(i))
                    }
                    None => Err(format!(
                        "NAT-DEC: free variable `{tok}` in the written formula; bind it with ∀ or ∃"
                    )),
                }
            }
            other => Err(format!(
                "NAT-DEC: expected a numeral or `(`, found {other:?}"
            )),
        }
    }

    fn uses(&self, sym: &str, conns: &[String]) -> Result<(), String> {
        if conns.iter().any(|c| c == sym) {
            Ok(())
        } else {
            Err(format!(
                "NAT-DEC: `{sym}` is outside the logical symbols this convention reads"
            ))
        }
    }

    fn enter(&mut self) -> Result<(), String> {
        if self.depth >= MAX_PAREN_DEPTH {
            return Err(format!(
                "NAT-DEC: formula nested deeper than {MAX_PAREN_DEPTH}"
            ));
        }
        self.depth += 1;
        Ok(())
    }

    /// imp := or ('→' imp)?   (→ groups to the right)
    fn imp(&mut self, c: &[String]) -> Result<Prop, String> {
        self.enter()?;
        let l = self.or(c)?;
        let r = if self.peek() == Some("→") {
            self.uses("→", c)?;
            self.operator()?;
            let r = self.imp(c)?;
            Prop::Imp(Box::new(l), Box::new(r))
        } else {
            l
        };
        self.depth -= 1;
        Ok(r)
    }
    fn or(&mut self, c: &[String]) -> Result<Prop, String> {
        let mut l = self.and(c)?;
        while self.peek() == Some("∨") {
            self.uses("∨", c)?;
            self.operator()?;
            let r = self.and(c)?;
            l = Prop::Or(Box::new(l), Box::new(r));
        }
        Ok(l)
    }
    fn and(&mut self, c: &[String]) -> Result<Prop, String> {
        let mut l = self.unary(c)?;
        while self.peek() == Some("∧") {
            self.uses("∧", c)?;
            self.operator()?;
            let r = self.unary(c)?;
            l = Prop::And(Box::new(l), Box::new(r));
        }
        Ok(l)
    }
    fn unary(&mut self, c: &[String]) -> Result<Prop, String> {
        match self.peek() {
            Some("¬") => {
                self.uses("¬", c)?;
                self.operator()?;
                self.enter()?;
                let p = self.unary(c)?;
                self.depth -= 1;
                Ok(Prop::Imp(Box::new(p), Box::new(Prop::False)))
            }
            Some(q @ ("∀" | "∃")) => {
                let q = q.to_string();
                self.uses(&q, c)?;
                self.operator()?;
                let x = match self.peek() {
                    Some(t) if is_var(t) => t.to_string(),
                    other => {
                        return Err(format!(
                        "NAT-DEC: `{q}` needs a variable (one lower-case letter), found {other:?}"
                    ))
                    }
                };
                let vars = self.vars.as_mut().expect("formula mode");
                if vars.contains(&x) {
                    return Err(format!(
                        "NAT-DEC: shadowed name `{x}`: rename the inner variable"
                    ));
                }
                vars.push(x.clone());
                self.pos += 1;
                self.expect(".")?;
                let body = self.imp(c)?;
                self.vars.as_mut().expect("formula mode").pop();
                Ok(if q == "∀" {
                    Prop::Forall(x, Box::new(body))
                } else {
                    Prop::Exists(x, Box::new(body))
                })
            }
            _ => self.primary(c),
        }
    }
    fn primary(&mut self, c: &[String]) -> Result<Prop, String> {
        if self.peek() == Some("⊥") {
            self.uses("⊥", c)?;
            self.pos += 1;
            return Ok(Prop::False);
        }
        if self.peek() == Some("(") {
            // A parenthesised formula, or an equation whose left side starts
            // with a parenthesised term: try the formula, then the equation.
            // Backtracking restores every piece of parser state, the bound
            // names included (audit at 6ba9f1e, hygiene).
            let save = (self.pos, self.depth, self.numeral_total, self.operators);
            let saved_vars = self.vars.clone();
            self.pos += 1;
            if let Ok(p) = self.imp(c) {
                if self.peek() == Some(")") {
                    self.pos += 1;
                    return Ok(p);
                }
            }
            (self.pos, self.depth, self.numeral_total, self.operators) = save;
            self.vars = saved_vars;
        }
        if let Some(p) = self.prefix_use()? {
            return Ok(p);
        }
        let lhs = self.expr()?;
        // RLT-003 M3b: an infix definition (`<`, `∣`) at the level of `=`.
        if let Some(sym) = self.peek().map(str::to_string) {
            if sym != "=" && self.notations.contains(&sym) {
                self.operator()?;
                let rhs = self.expr()?;
                return self.expand(&crate::defs::Notation::Infix(sym), &[lhs, rhs]);
            }
        }
        self.expect("=")?;
        let rhs = self.expr()?;
        Ok(Prop::eq(lhs, rhs))
    }

    /// A prefix definition `NAME(t, …)`; NAME is written as letters with an
    /// optional digit suffix (`free235` reads as tokens `free`, `235`).
    fn prefix_use(&mut self) -> Result<Option<Prop>, String> {
        let (name, len) = match (self.toks.get(self.pos), self.toks.get(self.pos + 1)) {
            (Some(a), Some(b))
                if a.chars().all(|c| c.is_ascii_alphabetic())
                    && b.chars().all(|c| c.is_ascii_digit()) =>
            {
                (format!("{a}{b}"), 2)
            }
            (Some(a), _) if a.len() > 1 && a.chars().all(|c| c.is_ascii_alphabetic()) => {
                (a.clone(), 1)
            }
            _ => return Ok(None),
        };
        if !self.notations.contains(&name)
            || self.toks.get(self.pos + len).map(String::as_str) != Some("(")
        {
            return Ok(None);
        }
        self.pos += len + 1;
        let mut args = vec![self.expr()?];
        while self.peek() == Some(",") {
            self.pos += 1;
            args.push(self.expr()?);
        }
        self.expect(")")?;
        self.expand(&crate::defs::Notation::Prefix(name), &args)
            .map(Some)
    }

    fn expand(&self, n: &crate::defs::Notation, args: &[Term]) -> Result<Prop, String> {
        let defs = self
            .defs
            .as_ref()
            .ok_or("NAT-DEF: definitions are unavailable")?;
        let def = defs
            .by_notation(n)
            .ok_or(format!("NAT-DEF: notation {n:?} is unregistered"))?;
        defs.mark(&def.name);
        def.expand(args).map_err(|e| format!("NAT-DEF: {e}"))
    }
}

fn is_var(tok: &str) -> bool {
    tok.len() == 1 && tok.chars().all(|c| c.is_ascii_lowercase())
}

/// Elaborate a written formula with logical symbols (RLT-003 M3a). Equations
/// inside read exactly as `elaborate_formula` reads them; variables are single
/// lower-case letters bound by ∀ or ∃ in the same string.
pub fn elaborate_prop(w: &Written, conv: &Convention) -> Result<Prop, String> {
    if conv.elaborator != "nat-dec" {
        return Err(format!(
            "convention {} elaborates terms only; formulas use NAT-DEC",
            conv.id
        ));
    }
    let mut p = Parser {
        toks: &w.tokens,
        pos: 0,
        depth: 0,
        numeral_total: 0,
        times: conv.operators.iter().any(|o| o == "×"),
        operators: 0,
        vars: Some(Vec::new()),
        notations: conv.definitions.clone(),
        defs: if conv.definitions.is_empty() {
            None
        } else {
            conv.defs.clone()
        },
    };
    if !conv.definitions.is_empty() && p.defs.is_none() {
        return Err(format!(
            "convention {} reads definitions, and the pinned definitions are unavailable",
            conv.id
        ));
    }
    let f = p.imp(&conv.connectives)?;
    p.end()?;
    Ok(f)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn nat_times() -> Convention {
        let mut c = nat();
        c.operators.push("×".into());
        c
    }

    #[test]
    fn times_binds_tighter_and_groups_left() {
        let f = elaborate_formula(&Written::new("30 = 2 × 3 × 5"), &nat_times()).unwrap();
        assert_eq!(f.to_string(), "30 = (2 × 3) × 5");
        let f = elaborate_formula(&Written::new("2 × 3 + 1 = 7"), &nat_times()).unwrap();
        assert_eq!(f.to_string(), "(2 × 3) + 1 = 7");
        let f = elaborate_formula(&Written::new("2 × (3 + 1) = 8"), &nat_times()).unwrap();
        assert_eq!(f.to_string(), "2 × (3 + 1) = 8");
    }

    #[test]
    fn operator_chains_are_bounded() {
        let long = format!("{} = 0", vec!["0"; MAX_OPERATORS + 2].join(" × "));
        let e = elaborate_formula(&Written::new(&long), &nat_times()).unwrap_err();
        assert!(e.contains("operators in one expression"), "{e}");
        let ok = format!("{} = 0", vec!["0"; 100].join(" + "));
        assert!(elaborate_formula(&Written::new(&ok), &nat_times()).is_ok());
    }

    #[test]
    fn times_is_read_only_when_the_convention_lists_it() {
        let e = elaborate_formula(&Written::new("2 × 3 = 6"), &nat()).unwrap_err();
        assert!(e.contains("outside the operators"), "{e}");
        // `*` is proof syntax, never NAT-DEC notation.
        assert!(elaborate_formula(&Written::new("2 * 3 = 6"), &nat_times()).is_err());
    }

    fn nat() -> Convention {
        Convention {
            definitions: vec![],
            defs: None,
            id: "NAT-DEC".into(),
            elaborator: "nat-dec".into(),
            symbols: BTreeMap::new(),
            operators: vec!["+".into()],
            connectives: Vec::new(),
        }
    }
    #[test]
    fn nat_dec_reads_sums_left_associatively() {
        let f = elaborate_formula(&Written::new("1 + 1 + 1 = 3"), &nat()).unwrap();
        let one = Term::numeral(1);
        assert_eq!(f.lhs, Term::plus(Term::plus(one.clone(), one.clone()), one));
    }
    #[test]
    fn nat_dec_rejects_symbols_outside_its_reading() {
        assert!(elaborate_formula(&Written::new("2 × 3 = 6"), &nat()).is_err());
        assert!(elaborate_term(&Written::new("π"), &nat()).is_err());
    }
}

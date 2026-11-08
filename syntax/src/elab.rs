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
use rlt_kernel::{Formula, Term};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Convention {
    pub id: String,
    pub elaborator: String,
    pub symbols: BTreeMap<String, String>,
    /// NAT-DEC operators this convention reads: `+`, and `×` once listed.
    pub operators: Vec<String>,
}

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
            other => Err(format!(
                "NAT-DEC: expected a numeral or `(`, found {other:?}"
            )),
        }
    }
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
            id: "NAT-DEC".into(),
            elaborator: "nat-dec".into(),
            symbols: BTreeMap::new(),
            operators: vec!["+".into()],
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

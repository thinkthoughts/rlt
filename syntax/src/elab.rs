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
}

pub fn elaborate_term(w: &Written, conv: &Convention) -> Result<Term, String> {
    match conv.elaborator.as_str() {
        "nat-dec" => {
            let mut p = Parser {
                toks: &w.tokens,
                pos: 0,
                depth: 0,
                numeral_total: 0,
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

struct Parser<'a> {
    toks: &'a [String],
    pos: usize,
    depth: usize,
    numeral_total: u64,
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
    fn expr(&mut self) -> Result<Term, String> {
        let mut t = self.atom()?;
        while self.peek() == Some("+") {
            self.pos += 1;
            let r = self.atom()?;
            t = Term::plus(t, r);
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
    fn nat() -> Convention {
        Convention {
            id: "NAT-DEC".into(),
            elaborator: "nat-dec".into(),
            symbols: BTreeMap::new(),
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

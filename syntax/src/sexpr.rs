//! A small S-expression reader. Atoms are symbols, strings ("…") and
//! non-negative integers. `;` starts a comment that runs to end of line.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Sexp {
    Sym(String),
    Str(String),
    Int(u64),
    List(Vec<Sexp>),
}

impl Sexp {
    pub fn sym(&self) -> Option<&str> {
        match self {
            Sexp::Sym(s) => Some(s),
            _ => None,
        }
    }
    pub fn list(&self) -> Option<&[Sexp]> {
        match self {
            Sexp::List(v) => Some(v),
            _ => None,
        }
    }
    /// For a list whose head is a symbol: (head rest…).
    pub fn head(&self) -> Option<(&str, &[Sexp])> {
        let l = self.list()?;
        let (h, rest) = l.split_first()?;
        Some((h.sym()?, rest))
    }
}

/// Maximum list nesting depth accepted by the reader.
pub const MAX_DEPTH: usize = 256;

pub fn parse_all(src: &str) -> Result<Vec<Sexp>, String> {
    let chars: Vec<char> = src.chars().collect();
    let mut pos = 0;
    let mut out = Vec::new();
    loop {
        skip_ws(&chars, &mut pos);
        if pos >= chars.len() {
            return Ok(out);
        }
        out.push(parse_one(&chars, &mut pos, 0)?);
    }
}

fn skip_ws(c: &[char], pos: &mut usize) {
    while *pos < c.len() {
        if c[*pos].is_whitespace() {
            *pos += 1;
        } else if c[*pos] == ';' {
            while *pos < c.len() && c[*pos] != '\n' {
                *pos += 1;
            }
        } else {
            break;
        }
    }
}

fn line_of(c: &[char], pos: usize) -> usize {
    c[..pos.min(c.len())]
        .iter()
        .filter(|&&ch| ch == '\n')
        .count()
        + 1
}

fn parse_one(c: &[char], pos: &mut usize, depth: usize) -> Result<Sexp, String> {
    skip_ws(c, pos);
    if *pos >= c.len() {
        return Err("unexpected end of input".into());
    }
    match c[*pos] {
        '(' => {
            if depth >= MAX_DEPTH {
                return Err(format!(
                    "nesting deeper than {MAX_DEPTH} on line {}",
                    line_of(c, *pos)
                ));
            }
            let start = *pos;
            *pos += 1;
            let mut items = Vec::new();
            loop {
                skip_ws(c, pos);
                if *pos >= c.len() {
                    return Err(format!("unclosed '(' opened on line {}", line_of(c, start)));
                }
                if c[*pos] == ')' {
                    *pos += 1;
                    return Ok(Sexp::List(items));
                }
                items.push(parse_one(c, pos, depth + 1)?);
            }
        }
        ')' => Err(format!("unexpected ')' on line {}", line_of(c, *pos))),
        '"' => {
            let start = *pos;
            *pos += 1;
            let mut s = String::new();
            while *pos < c.len() && c[*pos] != '"' {
                if c[*pos] == '\\' && *pos + 1 < c.len() {
                    *pos += 1;
                }
                s.push(c[*pos]);
                *pos += 1;
            }
            if *pos >= c.len() {
                return Err(format!(
                    "unclosed string opened on line {}",
                    line_of(c, start)
                ));
            }
            *pos += 1;
            Ok(Sexp::Str(s))
        }
        _ => {
            let mut s = String::new();
            while *pos < c.len() && !c[*pos].is_whitespace() && !"()\";".contains(c[*pos]) {
                s.push(c[*pos]);
                *pos += 1;
            }
            if !s.is_empty() && s.chars().all(|ch| ch.is_ascii_digit()) {
                s.parse::<u64>()
                    .map(Sexp::Int)
                    .map_err(|e| format!("integer `{s}`: {e}"))
            } else {
                Ok(Sexp::Sym(s))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reads_nested_lists_strings_ints_and_comments() {
        let v = parse_all("(a \"x y\" 5 (b)) ; note\n(c)").unwrap();
        assert_eq!(v.len(), 2);
        assert_eq!(
            v[0],
            Sexp::List(vec![
                Sexp::Sym("a".into()),
                Sexp::Str("x y".into()),
                Sexp::Int(5),
                Sexp::List(vec![Sexp::Sym("b".into())])
            ])
        );
        assert!(parse_all("(a").is_err());
    }
    #[test]
    fn deep_nesting_is_refused_without_overflow() {
        let deep = "(".repeat(20_000) + &")".repeat(20_000);
        assert!(parse_all(&deep).unwrap_err().contains("nesting deeper"));
    }
}

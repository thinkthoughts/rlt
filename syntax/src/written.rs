//! Layer 1 — written expressions.
//!
//! A written expression is the exact source text plus its token sequence.
//! Tokens are maximal runs of ASCII digits, maximal runs of letters, and
//! single other non-space characters; whitespace separates tokens.
//! Written identity (≡w) is equality of token sequences. It is decided
//! here, independently of any convention or proof.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Written {
    pub text: String,
    pub tokens: Vec<String>,
}

impl Written {
    pub fn new(text: &str) -> Written {
        Written {
            text: text.to_string(),
            tokens: tokenize(text),
        }
    }

    /// Written identity ≡w: same token sequence.
    pub fn identical(&self, other: &Written) -> bool {
        self.tokens == other.tokens
    }
}

pub fn tokenize(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
        } else if c.is_ascii_digit() {
            let s = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            out.push(chars[s..i].iter().collect());
        } else if c.is_ascii_alphabetic() {
            let s = i;
            while i < chars.len() && chars[i].is_ascii_alphabetic() {
                i += 1;
            }
            out.push(chars[s..i].iter().collect());
        } else {
            out.push(c.to_string());
            i += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn one_plus_one_and_two_are_written_distinct() {
        assert!(!Written::new("1+1").identical(&Written::new("2")));
    }
    #[test]
    fn spacing_is_outside_the_token_sequence() {
        assert!(Written::new("1+1").identical(&Written::new("1 + 1")));
    }
    #[test]
    fn pi_is_written_identical_to_pi() {
        assert!(Written::new("π").identical(&Written::new("π")));
        assert_eq!(tokenize("29 + 1 = 30"), vec!["29", "+", "1", "=", "30"]);
    }
}

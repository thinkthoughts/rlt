//! rlt-kernel — the trusted computing base of RLT (Reading Level Test).
//!
//! Soundness rests on this crate alone (plus the axiom file it is given and
//! the Rust toolchain): term structure and numeral expansion (`term`), sort
//! checking (`term::check_sort`), the six rules and premise bookkeeping
//! (`rules`), and step-by-step replay with the final goal comparison (`check`).
//!
//! Parsers, convention files, elaborators, reading-point records and
//! physical-correspondence records live outside this crate. They supply
//! inputs; they carry zero proof authority.

pub mod check;
pub mod rules;
pub mod term;

pub use check::{check, Arg, Mode, ProofObject, Step, Verdict, RULES};
pub use rules::{Axiom, Theorem};
pub use term::{Formula, Term};

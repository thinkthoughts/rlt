//! rlt-syntax — the untrusted layer of RLT.
//!
//! Layer 1 (written expressions), layer 2 (conventions and elaboration),
//! proof-file and axiom-file reading, and YAML records for sources,
//! conventions, reading points and physical correspondence. Everything here
//! supplies inputs to the kernel or describes context; the kernel alone
//! decides acceptance.

pub mod elab;
pub mod proof;
pub mod records;
pub mod sexpr;
pub mod written;

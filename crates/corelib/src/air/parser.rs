//! Commitment-aware AIR parser for the mini-DSL shared by the CLI, SDK, and
//! bindings.
//!
//! The parser accepts either TOML (the canonical on-disk format) or YAML (for
//! author ergonomics) and always produces an [`AirIr`] ready for commitment
//! validation.  A typical DSL fragment looks like this:
//!
//! ```toml
//! [meta]
//! name = "toy_balance"
//! field = "Prime254"
//! hash = "poseidon2"
//! degree_hint = 8
//!
//! [columns]
//! trace_cols = 8
//! const_cols = 2
//! periodic_cols = 1
//!
//! [constraints]
//! transition_count = 4
//! boundary_count = 2
//!
//! [[public_inputs]]
//! name = "root"
//! type = "bytes"
//!
//! commitments = [
//!     { kind = "poseidon_commit", public = ["root"] }
//! ]
//! ```
//!
//! The example above binds a Poseidon commitment gadget to the `root` public
//! input.  Commitment bindings are validated for well-formedness but do **not**
//! influence degree accounting—the [`AirIr::degree_hint`] remains whatever the
//! AIR author specified under `meta.degree_hint`.
//!
//! # Examples
//!
//! ```
//! use zkprov_corelib::air::parser::parse_air_str;
//! use zkprov_corelib::air::types::{CommitmentKind, PublicTy};
//!
//! let src = r#"
//! [meta]
//! name = "toy_balance"
//! field = "Prime254"
//! hash = "poseidon2"
//! degree_hint = 8
//!
//! [columns]
//! trace_cols = 8
//! const_cols = 2
//! periodic_cols = 1
//!
//! [constraints]
//! transition_count = 4
//! boundary_count = 2
//!
//! [[public_inputs]]
//! name = "root"
//! type = "bytes"
//!
//! commitments = [
//!     { kind = "poseidon_commit", public = ["root"] }
//! ]
//! "#;
//!
//! let ir = parse_air_str(src).unwrap();
//! assert_eq!(ir.meta.name, "toy_balance");
//! assert_eq!(ir.columns.trace_cols, 8);
//! assert_eq!(ir.degree_hint, Some(8));
//! assert_eq!(ir.public_inputs[0].ty, PublicTy::Bytes);
//! assert!(matches!(ir.commitments[0].kind, CommitmentKind::PoseidonCommit));
//! assert_eq!(ir.commitments[0].public_inputs, ["root".to_string()]);
//! ```

use std::path::Path;

use anyhow::{Context, Result};

use super::types::AirIr;
use super::validate::validate_bindings;
use super::AirProgram;

/// Parse an AIR definition from disk and return the validated [`AirIr`].
///
/// This helper accepts either `.air` (TOML) or `.yaml` sources.  Regardless of
/// the input format the same commitment-aware validation is applied.
pub fn parse_air_file(path: &Path) -> Result<AirIr> {
    let program = AirProgram::load_from_file(path)?;
    let ir = AirIr::from(program);
    validate_bindings(&ir)?;
    Ok(ir)
}

/// Parse an in-memory AIR definition encoded as TOML and return the
/// commitment-validated [`AirIr`].
///
/// # Errors
///
/// Returns an error if the input cannot be decoded, violates structural
/// constraints, or declares invalid commitment bindings.
pub fn parse_air_str(src: &str) -> Result<AirIr> {
    let program: AirProgram = toml::from_str(src).context("parsing AIR source")?;
    program.validate()?;
    let ir = AirIr::from(program);
    validate_bindings(&ir)?;
    Ok(ir)
}

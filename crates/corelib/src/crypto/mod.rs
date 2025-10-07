//! Crypto primitives surface area.
//! Phase-0 provides generic hash traits, BLAKE3 and Keccak-256 implementations,
//! placeholder sponge-style hashes, and hash-to-field for the 254-bit prime we
//! use in stubs.

pub mod blake3;
pub mod field;
pub mod hash;
pub mod keccak;
pub mod merkle;
pub mod poseidon2;
pub mod registry;
pub mod rescue;

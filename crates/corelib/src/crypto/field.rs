//! Hash-to-field for a 254-bit prime (BN254-like placeholder).
//! We implement a simple wide-reduce from 256-bit (or 512-bit) digests.

use num_bigint::BigUint;
use num_traits::One;

/// Prime modulus (placeholder Prime254: 2^254 - 127 * 2^120 + 1).
/// This is NOT BN254; it's a "Prime254" placeholder used across the scaffold.
/// Replace with the exact field modulus when wiring real backends.
pub fn prime254_modulus() -> BigUint {
    // p = 2^254 - 127 * 2^120 + 1
    let two = BigUint::from(2u32);
    (two.pow(254) - (BigUint::from(127u32) * two.pow(120))) + BigUint::one()
}

/// Reduce arbitrary bytes to field element in [0, p).
pub fn reduce_to_prime254(bytes: &[u8]) -> BigUint {
    let p = prime254_modulus();
    let x = BigUint::from_bytes_be(bytes);
    x % p
}

/// Convenience: hash-to-field from a 32-byte digest (big-endian)
pub fn h2f_32_be(digest32: [u8; 32]) -> BigUint {
    reduce_to_prime254(&digest32)
}

/// Convenience: hash-to-field from two concatenated 32-byte digests (64 bytes)
pub fn h2f_64_be(digest_a: [u8; 32], digest_b: [u8; 32]) -> BigUint {
    let mut v = [0u8; 64];
    v[..32].copy_from_slice(&digest_a);
    v[32..].copy_from_slice(&digest_b);
    reduce_to_prime254(&v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modulus_reasonable() {
        let p = prime254_modulus();
        assert!(p.bits() >= 250 && p.bits() <= 254);
    }

    #[test]
    fn reduce_basic() {
        let p = prime254_modulus();
        let zero = reduce_to_prime254(&[0u8; 1]);
        assert_eq!(zero, BigUint::from(0u8));

        let ones = reduce_to_prime254(&[0xffu8; 64]);
        assert!(ones < p);
        assert!(ones.bits() <= 254);
    }
}

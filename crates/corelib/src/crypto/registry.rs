//! String-id -> Hash32 mapping and convenience helpers.

use crate::crypto::blake3::Blake3;
use crate::crypto::hash::hash_labeled;
use crate::crypto::keccak::Keccak256;
use crate::crypto::poseidon2::Poseidon2;
use crate::crypto::rescue::Rescue;

fn normalize(id: &str) -> String {
    id.trim().to_ascii_lowercase()
}

/// Return H(label || data) for the given hash id.
///
/// Supported ids: "blake3", "keccak256", "poseidon2", "rescue".
pub fn hash32_by_id(id: &str, label: &str, data: &[u8]) -> Option<[u8; 32]> {
    match normalize(id).as_str() {
        "blake3" => Some(hash_labeled::<Blake3>(label, data)),
        "keccak256" => Some(hash_labeled::<Keccak256>(label, data)),
        "poseidon2" => Some(hash_labeled::<Poseidon2>(label, data)),
        "rescue" => Some(hash_labeled::<Rescue>(label, data)),
        _ => None,
    }
}

/// Convenience helper deriving a u64 from the first 8 bytes (little-endian).
pub fn hash64_by_id(id: &str, label: &str, data: &[u8]) -> Option<u64> {
    hash32_by_id(id, label, data).map(|digest| {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&digest[0..8]);
        u64::from_le_bytes(bytes)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_supports_known_hashes() {
        for id in ["blake3", "keccak256", "poseidon2", "rescue"] {
            assert!(hash32_by_id(id, "LBL", b"data").is_some());
            assert!(hash64_by_id(id, "LBL", b"data").is_some());
        }
    }

    #[test]
    fn registry_unknown_hash_returns_none() {
        assert!(hash32_by_id("unknown", "LBL", b"data").is_none());
        assert!(hash64_by_id("unknown", "LBL", b"data").is_none());
    }

    #[test]
    fn registry_hashes_are_distinct() {
        let blake = hash32_by_id("blake3", "LBL", b"data").unwrap();
        let keccak = hash32_by_id("keccak256", "LBL", b"data").unwrap();
        let poseidon = hash32_by_id("poseidon2", "LBL", b"data").unwrap();
        let rescue = hash32_by_id("rescue", "LBL", b"data").unwrap();
        assert_ne!(blake, keccak);
        assert_ne!(blake, poseidon);
        assert_ne!(blake, rescue);
        assert_ne!(poseidon, rescue);
    }
}

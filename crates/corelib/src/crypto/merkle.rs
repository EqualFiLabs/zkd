//! Simple Merkle trees of arity 2 and 4 over any [`Hash32`] implementation.
//! Domain separation:
//! - leaf:  H("LEAF"  || data)
//! - node2: H("NODE2" || left || right)
//! - node4: H("NODE4" || c0 || c1 || c2 || c3)

use crate::crypto::hash::{hash_labeled, Hash32};

/// Hash a leaf with the `"LEAF"` domain separator.
pub fn leaf_hash<H: Hash32>(data: &[u8]) -> [u8; 32] {
    hash_labeled::<H>("LEAF", data)
}

/// Hash a binary node with the `"NODE2"` domain separator.
pub fn node2_hash<H: Hash32>(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut h = H::new();
    h.update(b"NODE2");
    h.update(left);
    h.update(right);
    h.finalize()
}

/// Hash a quaternary node with the `"NODE4"` domain separator.
pub fn node4_hash<H: Hash32>(
    c0: &[u8; 32],
    c1: &[u8; 32],
    c2: &[u8; 32],
    c3: &[u8; 32],
) -> [u8; 32] {
    let mut h = H::new();
    h.update(b"NODE4");
    h.update(c0);
    h.update(c1);
    h.update(c2);
    h.update(c3);
    h.finalize()
}

/// Compute a Merkle root for an arity-2 tree.
pub fn root_arity2<H: Hash32>(leaves: &[Vec<u8>]) -> [u8; 32] {
    assert!(!leaves.is_empty(), "no leaves");
    let mut level: Vec<[u8; 32]> = leaves.iter().map(|d| leaf_hash::<H>(d)).collect();
    while level.len() > 1 {
        let mut next = Vec::with_capacity(level.len().div_ceil(2));
        for chunk in level.chunks(2) {
            let node = if chunk.len() == 2 {
                node2_hash::<H>(&chunk[0], &chunk[1])
            } else {
                // duplicate last for odd count
                node2_hash::<H>(&chunk[0], &chunk[0])
            };
            next.push(node);
        }
        level = next;
    }
    level[0]
}

/// Compute a Merkle root for an arity-4 tree.
pub fn root_arity4<H: Hash32>(leaves: &[Vec<u8>]) -> [u8; 32] {
    assert!(!leaves.is_empty(), "no leaves");
    let mut level: Vec<[u8; 32]> = leaves.iter().map(|d| leaf_hash::<H>(d)).collect();
    while level.len() > 1 {
        let mut next = Vec::with_capacity(level.len().div_ceil(4));
        for chunk in level.chunks(4) {
            let node = match chunk.len() {
                4 => node4_hash::<H>(&chunk[0], &chunk[1], &chunk[2], &chunk[3]),
                3 => node4_hash::<H>(&chunk[0], &chunk[1], &chunk[2], &chunk[2]),
                2 => node4_hash::<H>(&chunk[0], &chunk[1], &chunk[1], &chunk[1]),
                1 => node4_hash::<H>(&chunk[0], &chunk[0], &chunk[0], &chunk[0]),
                _ => unreachable!(),
            };
            next.push(node);
        }
        level = next;
    }
    level[0]
}

/// Very simple inclusion proof for arity-2: a list of `(is_right, sibling)` pairs.
pub struct Proof2 {
    pub path: Vec<(bool, [u8; 32])>,
}

/// Generate an inclusion proof for a leaf in an arity-2 Merkle tree.
pub fn prove_arity2<H: Hash32>(leaves: &[Vec<u8>], index: usize) -> Proof2 {
    assert!(index < leaves.len());
    let mut idx = index;
    let mut level: Vec<[u8; 32]> = leaves.iter().map(|d| leaf_hash::<H>(d)).collect();
    let mut path = Vec::new();
    while level.len() > 1 {
        let is_right = idx % 2 == 1;
        let sibling_idx = if is_right { idx - 1 } else { idx + 1 };
        let sibling = if sibling_idx < level.len() {
            level[sibling_idx]
        } else {
            level[idx]
        };
        path.push((is_right, sibling));

        let mut next = Vec::with_capacity(level.len().div_ceil(2));
        for chunk in level.chunks(2) {
            let node = if chunk.len() == 2 {
                node2_hash::<H>(&chunk[0], &chunk[1])
            } else {
                node2_hash::<H>(&chunk[0], &chunk[0])
            };
            next.push(node);
        }
        idx /= 2;
        level = next;
    }

    Proof2 { path }
}

/// Verify an inclusion proof for an arity-2 Merkle tree.
pub fn verify_arity2<H: Hash32>(
    leaf: &[u8],
    _index: usize,
    proof: &Proof2,
    root: &[u8; 32],
) -> bool {
    let mut acc = leaf_hash::<H>(leaf);
    for (is_right, sibling) in &proof.path {
        acc = if *is_right {
            node2_hash::<H>(sibling, &acc)
        } else {
            node2_hash::<H>(&acc, sibling)
        };
    }
    &acc == root
}

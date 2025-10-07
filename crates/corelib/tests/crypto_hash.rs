use zkprov_corelib::crypto::blake3::Blake3;
use zkprov_corelib::crypto::hash::{hash_labeled, hash_one_shot};
use zkprov_corelib::crypto::keccak::Keccak256;

#[test]
fn blake3_one_shot_and_labeled() {
    let d0 = hash_one_shot::<Blake3>(b"");
    let d1 = hash_one_shot::<Blake3>(b"abc");
    assert_ne!(d0, d1);
    let dl = hash_labeled::<Blake3>("LBL", b"abc");
    assert_ne!(d1, dl);
}

#[test]
fn keccak256_empty_matches_vector() {
    // c5d246...a470
    let got = hash_one_shot::<Keccak256>(b"");
    let exp =
        hex::decode("c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470").unwrap();
    assert_eq!(got, exp.as_slice());
}

// Tiny hex decoder (test-only)
mod hex {
    pub fn decode(s: &str) -> Result<Vec<u8>, String> {
        if s.len() % 2 != 0 {
            return Err("len".into());
        }
        let mut out = Vec::with_capacity(s.len() / 2);
        let bytes = s.as_bytes();
        for i in (0..bytes.len()).step_by(2) {
            let hi = val(bytes[i])?;
            let lo = val(bytes[i + 1])?;
            out.push((hi << 4) | lo);
        }
        Ok(out)
    }

    fn val(b: u8) -> Result<u8, String> {
        match b {
            b'0'..=b'9' => Ok(b - b'0'),
            b'a'..=b'f' => Ok(b - b'a' + 10),
            b'A'..=b'F' => Ok(b - b'A' + 10),
            _ => Err("hex".into()),
        }
    }
}

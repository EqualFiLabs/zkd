use zkprov_corelib::evm::abi::{
    decode_body, decode_meta, decode_public_io, encode_body, encode_meta, encode_public_io,
};
use zkprov_corelib::evm::digest::keccak256_bytes;
use zkprov_corelib::proof::ProofHeader;

#[test]
fn keccak_empty_matches_vector() {
    let digest = keccak256_bytes(b"");
    let expected =
        hex::decode("c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470").unwrap();
    assert_eq!(digest.as_slice(), expected.as_slice());
}

#[test]
fn abi_round_trip_meta_and_body() {
    let header = ProofHeader {
        backend_id_hash: 42,
        profile_id_hash: 7,
        pubio_hash: 1337,
        body_len: 5,
    };
    let body = b"hello";
    let json = "{\"foo\":42}";

    let encoded_meta = encode_meta(&header);
    let decoded_header = decode_meta(&encoded_meta).expect("meta decode");
    assert_eq!(decoded_header, header);

    let encoded_body = encode_body(body);
    let decoded_body = decode_body(&encoded_body).expect("body decode");
    assert_eq!(decoded_body.as_slice(), body);

    let encoded_io = encode_public_io(json);
    let decoded_io = decode_public_io(&encoded_io).expect("public io decode");
    assert_eq!(decoded_io, json);
}

mod hex {
    pub fn decode(s: &str) -> Result<Vec<u8>, String> {
        if !s.len().is_multiple_of(2) {
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

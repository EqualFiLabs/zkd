use std::ffi::CString;

use serde::Serialize;
use serde_json::{json, Map, Value};

use crate::error::{ErrorCode, ZKP_OK};

const RESERVED_FIELDS: &[&str] = &["ok", "code", "msg"];

pub struct Envelope {
    map: Map<String, Value>,
}

impl Envelope {
    fn new(map: Map<String, Value>) -> Self {
        Self { map }
    }

    fn from_value(value: Value) -> Self {
        match value {
            Value::Object(map) => Self::new(map),
            _ => panic!("JSON envelope must be an object"),
        }
    }

    pub fn into_string(self) -> String {
        serde_json::to_string(&Value::Object(self.map)).expect("failed to serialize JSON envelope")
    }

    pub fn into_cstring(self) -> CString {
        CString::new(self.into_string()).expect("JSON envelopes must not contain NUL bytes")
    }
}

pub fn ok() -> Envelope {
    Envelope::from_value(json!({
        "ok": true,
        "code": ZKP_OK,
        "msg": "OK",
    }))
}

pub fn err(msg_code: ErrorCode, msg: impl Into<String>) -> Envelope {
    Envelope::from_value(json!({
        "ok": false,
        "code": msg_code.code(),
        "msg": msg.into(),
    }))
}

pub fn with_field<T>(mut envelope: Envelope, key: impl Into<String>, value: T) -> Envelope
where
    T: Serialize,
{
    let key = key.into();
    if RESERVED_FIELDS.contains(&key.as_str()) {
        panic!("field '{key}' is reserved by the FFI envelope");
    }

    let value = serde_json::to_value(value).expect("failed to serialize field value");
    envelope.map.insert(key, value);
    envelope
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn ok_envelope_roundtrips() {
        let json = ok().into_string();
        let value: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["ok"], Value::Bool(true));
        assert_eq!(value["code"], Value::from(0));
        assert_eq!(value["msg"], Value::from("OK"));
    }

    #[test]
    fn err_envelope_roundtrips() {
        let json = err(ErrorCode::ProofCorrupt, "invalid proof").into_string();
        let value: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["ok"], Value::Bool(false));
        assert_eq!(value["code"], Value::from(4));
        assert_eq!(value["msg"], Value::from("invalid proof"));
    }

    #[test]
    fn with_field_adds_value() {
        let json = with_field(ok(), "digest", "0xdeadbeef").into_string();
        let value: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["digest"], Value::from("0xdeadbeef"));
    }
}

//! Canonical serialization helpers.
//!
//! Provides CBOR serialization for deterministic hashing and
//! compact storage of simulation state.

use serde::{de::DeserializeOwned, Serialize};
use std::fmt;

/// Serialize a value to canonical CBOR bytes.
pub fn to_cbor<T: Serialize>(value: &T) -> Result<Vec<u8>, SerialError> {
    let mut buf = Vec::new();
    ciborium::into_writer(value, &mut buf).map_err(|e| SerialError(format!("CBOR serialize: {}", e)))?;
    Ok(buf)
}

/// Deserialize a value from CBOR bytes.
pub fn from_cbor<T: DeserializeOwned>(data: &[u8]) -> Result<T, SerialError> {
    ciborium::from_reader(data).map_err(|e| SerialError(format!("CBOR deserialize: {}", e)))
}

/// Serialize a value to JSON bytes (for human-readable output).
pub fn to_json<T: Serialize>(value: &T) -> Result<Vec<u8>, SerialError> {
    serde_json::to_vec_pretty(value).map_err(|e| SerialError(format!("JSON serialize: {}", e)))
}

/// Errors from serialization operations.
#[derive(Debug)]
pub struct SerialError(pub String);

impl fmt::Display for SerialError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "serialization error: {}", self.0)
    }
}

impl std::error::Error for SerialError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cbor_roundtrip() {
        #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        struct TestData {
            name: String,
            value: i64,
            tags: Vec<String>,
        }

        let data = TestData {
            name: "test".to_string(),
            value: 42,
            tags: vec!["a".to_string(), "b".to_string()],
        };

        let bytes = to_cbor(&data).unwrap();
        let restored: TestData = from_cbor(&bytes).unwrap();
        assert_eq!(data, restored);
    }

    #[test]
    fn cbor_is_deterministic() {
        #[derive(serde::Serialize)]
        struct D {
            a: u32,
            b: String,
        }
        let d = D {
            a: 1,
            b: "x".into(),
        };
        let bytes1 = to_cbor(&d).unwrap();
        let bytes2 = to_cbor(&d).unwrap();
        assert_eq!(bytes1, bytes2);
    }
}

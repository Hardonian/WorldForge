//! Canonical serialization helpers.
//!
//! Provides CBOR serialization for deterministic hashing and
//! compact storage of simulation state.

use serde::{de::DeserializeOwned, Serialize};

/// Serialize a value to canonical CBOR bytes.
pub fn to_cbor<T: Serialize>(value: &T) -> Result<Vec<u8>, CborError> {
    let mut buf = Vec::new();
    ciborium::into_writer(value, &mut buf).map_err(|e| CborError::Serialize(e.to_string()))?;
    Ok(buf)
}

/// Deserialize a value from CBOR bytes.
pub fn from_cbor<T: DeserializeOwned>(data: &[u8]) -> Result<T, CborError> {
    ciborium::from_reader(data).map_err(|e| CborError::Deserialize(e.to_string()))
}

/// Serialize a value to JSON bytes (for human-readable output).
pub fn to_json<T: Serialize>(value: &T) -> Result<Vec<u8>, CborError> {
    serde_json::to_vec_pretty(value).map_err(|e| CborError::Serialize(e.to_string()))
}

/// Errors from serialization operations.
#[derive(Debug, thiserror::Error)]
pub enum CborError {
    #[error("serialization failed: {0}")]
    Serialize(String),
    #[error("deserialization failed: {0}")]
    Deserialize(String),
}

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

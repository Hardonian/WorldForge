//! BLAKE3-based fingerprinting and hashing utilities.
//!
//! All simulation-critical hashing uses BLAKE3 for speed and security.
//! Fingerprints are used throughout for determinism verification,
//! replay integrity, and package reproducibility.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A BLAKE3 hash digest used as a fingerprint for simulation state,
/// events, packages, and other artifacts.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Fingerprint([u8; 32]);

impl Fingerprint {
    /// The zero fingerprint (used as initial chain value).
    pub const ZERO: Fingerprint = Fingerprint([0u8; 32]);

    /// Create a fingerprint from raw bytes.
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Get the raw bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Hash arbitrary bytes into a fingerprint.
    pub fn hash(data: &[u8]) -> Self {
        Self(*blake3::hash(data).as_bytes())
    }

    /// Hash serializable data by first serializing to canonical CBOR.
    pub fn hash_cbor<T: Serialize>(value: &T) -> Self {
        let mut buf = Vec::new();
        ciborium::into_writer(value, &mut buf).expect("CBOR serialization failed");
        Self::hash(&buf)
    }

    /// Chain two fingerprints together (for hash chains).
    /// Result = BLAKE3(self || other)
    pub fn chain(&self, other: &Fingerprint) -> Fingerprint {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.0);
        hasher.update(&other.0);
        Self(*hasher.finalize().as_bytes())
    }

    /// Chain with raw data.
    pub fn chain_data(&self, data: &[u8]) -> Fingerprint {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.0);
        hasher.update(data);
        Self(*hasher.finalize().as_bytes())
    }

    /// Hex string representation.
    pub fn to_hex(&self) -> String {
        hex_encode(&self.0)
    }

    /// Short hex string (first 16 chars) for display.
    pub fn to_short_hex(&self) -> String {
        self.to_hex()[..16].to_string()
    }
}

impl fmt::Debug for Fingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fingerprint({})", self.to_short_hex())
    }
}

impl fmt::Display for Fingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Incremental hasher for building fingerprints from multiple data sources.
pub struct FingerprintBuilder {
    hasher: blake3::Hasher,
}

impl FingerprintBuilder {
    pub fn new() -> Self {
        Self {
            hasher: blake3::Hasher::new(),
        }
    }

    /// Feed raw bytes.
    pub fn update(&mut self, data: &[u8]) -> &mut Self {
        self.hasher.update(data);
        self
    }

    /// Feed a serializable value (CBOR-encoded).
    pub fn update_cbor<T: Serialize>(&mut self, value: &T) -> &mut Self {
        let mut buf = Vec::new();
        ciborium::into_writer(value, &mut buf).expect("CBOR serialization failed");
        self.hasher.update(&buf);
        self
    }

    /// Feed a fingerprint.
    pub fn update_fingerprint(&mut self, fp: &Fingerprint) -> &mut Self {
        self.hasher.update(fp.as_bytes());
        self
    }

    /// Finalize and produce the fingerprint.
    pub fn finalize(&self) -> Fingerprint {
        Fingerprint(*self.hasher.finalize().as_bytes())
    }
}

impl Default for FingerprintBuilder {
    fn default() -> Self {
        Self::new()
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_data_same_fingerprint() {
        let a = Fingerprint::hash(b"hello world");
        let b = Fingerprint::hash(b"hello world");
        assert_eq!(a, b);
    }

    #[test]
    fn different_data_different_fingerprint() {
        let a = Fingerprint::hash(b"hello");
        let b = Fingerprint::hash(b"world");
        assert_ne!(a, b);
    }

    #[test]
    fn chain_is_deterministic() {
        let a = Fingerprint::hash(b"a");
        let b = Fingerprint::hash(b"b");
        let c1 = a.chain(&b);
        let c2 = a.chain(&b);
        assert_eq!(c1, c2);
    }

    #[test]
    fn chain_order_matters() {
        let a = Fingerprint::hash(b"a");
        let b = Fingerprint::hash(b"b");
        assert_ne!(a.chain(&b), b.chain(&a));
    }

    #[test]
    fn builder_matches_direct() {
        let data = b"test data for builder";
        let direct = Fingerprint::hash(data);
        let built = FingerprintBuilder::new().update(data).finalize();
        assert_eq!(direct, built);
    }

    #[test]
    fn hex_roundtrip() {
        let fp = Fingerprint::hash(b"roundtrip test");
        let hex = fp.to_hex();
        assert_eq!(hex.len(), 64);
    }

    #[test]
    fn cbor_hash_deterministic() {
        #[derive(Serialize)]
        struct TestData {
            x: i32,
            y: String,
        }
        let data = TestData {
            x: 42,
            y: "hello".to_string(),
        };
        let a = Fingerprint::hash_cbor(&data);
        let b = Fingerprint::hash_cbor(&data);
        assert_eq!(a, b);
    }
}

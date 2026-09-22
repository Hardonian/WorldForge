//! # worldforge-core
//!
//! Core types, strongly-typed identifiers, deterministic primitives, and shared
//! traits for the World Forge simulation platform.
//!
//! This crate provides the foundational building blocks used by all other
//! worldforge crates:
//!
//! - **Typed IDs** — `WorldId`, `EntityId`, `SystemId`, `ResourceId`, etc.
//! - **Time** — `Tick` type with fixed-timestep semantics
//! - **Fixed-point** — `Fixed64` (Q32.32) for deterministic economy math
//! - **Deterministic RNG** — `DeterministicRng` wrapping ChaCha8
//! - **Hashing** — BLAKE3-based fingerprinting via `Fingerprint`
//! - **Errors** — Structured error codes (WF1001, WF2004, etc.)
//! - **Serialization** — Canonical CBOR helpers
//! - **Feature status** — Graceful degradation primitives

pub mod error;
pub mod fixed;
pub mod hash;
pub mod id;
pub mod rng;
pub mod serial;
pub mod time;
pub mod version;

// Re-export key types at crate root
pub use error::{ErrorCode, WorldForgeError};
pub use fixed::Fixed64;
pub use hash::Fingerprint;
pub use id::*;
pub use rng::DeterministicRng;
pub use time::Tick;
pub use version::{EngineVersion, FormatVersion};

/// Feature availability status for graceful degradation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum FeatureStatus<T> {
    /// Feature is available and returned a value.
    Available(T),
    /// Feature is not yet implemented.
    Unavailable { reason: &'static str },
    /// Feature is implemented but operating in a reduced capacity.
    Degraded { reason: String, fallback: Option<T> },
}

impl<T> FeatureStatus<T> {
    /// Returns the value if available, or the fallback if degraded.
    pub fn value(self) -> Option<T> {
        match self {
            FeatureStatus::Available(v) => Some(v),
            FeatureStatus::Degraded { fallback, .. } => fallback,
            FeatureStatus::Unavailable { .. } => None,
        }
    }

    /// Returns true if the feature is fully available.
    pub fn is_available(&self) -> bool {
        matches!(self, FeatureStatus::Available(_))
    }
}

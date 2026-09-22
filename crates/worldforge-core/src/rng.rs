//! Deterministic random number generation.
//!
//! Uses ChaCha8 for fast, cryptographically-derived deterministic RNG.
//! Every simulation run with the same seed produces identical random sequences.

use rand::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::Fixed64;

/// A deterministic RNG that produces identical sequences given the same seed.
///
/// Wraps ChaCha8Rng for speed while maintaining full determinism.
/// Named streams allow different subsystems to have independent but
/// reproducible random sequences.
#[derive(Debug, Clone)]
pub struct DeterministicRng {
    rng: ChaCha8Rng,
    seed: u64,
    stream_name: String,
}

impl DeterministicRng {
    /// Create a new RNG from a seed. The stream name differentiates
    /// independent random sequences (e.g., "economy", "agents", "events").
    pub fn new(seed: u64, stream_name: &str) -> Self {
        let combined_seed = Self::derive_seed(seed, stream_name);
        Self {
            rng: ChaCha8Rng::seed_from_u64(combined_seed),
            seed,
            stream_name: stream_name.to_string(),
        }
    }

    /// Derive a child RNG for a subsystem, maintaining determinism.
    pub fn child(&self, child_name: &str) -> Self {
        let combined = format!("{}/{}", self.stream_name, child_name);
        Self::new(self.seed, &combined)
    }

    /// Get the original seed.
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Get the stream name.
    pub fn stream_name(&self) -> &str {
        &self.stream_name
    }

    /// Get a mutable reference to the underlying RNG.
    /// Use with `rand::Rng` trait methods.
    pub fn inner_mut(&mut self) -> &mut ChaCha8Rng {
        &mut self.rng
    }

    /// Generate a random u64.
    pub fn next_u64(&mut self) -> u64 {
        self.rng.gen()
    }

    /// Generate a random u32 in [0, max).
    pub fn next_u32_below(&mut self, max: u32) -> u32 {
        self.rng.gen_range(0..max)
    }

    /// Generate a deterministic Fixed64 in [0, 1).
    pub fn next_fixed(&mut self) -> Fixed64 {
        let v: u64 = self.rng.gen();
        // Map u64 to [0, 1) by using upper 32 bits as fractional part
        Fixed64::from_raw((v >> 32) as i64)
    }

    /// Generate a boolean with the given probability (as Fixed64 in [0, 1]).
    pub fn chance(&mut self, probability: Fixed64) -> bool {
        self.next_fixed() < probability
    }

    fn derive_seed(base_seed: u64, stream_name: &str) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        base_seed.hash(&mut hasher);
        stream_name.hash(&mut hasher);
        hasher.finish()
    }
}

/// Metadata about an RNG stream, for inclusion in replay artifacts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RngStreamInfo {
    pub seed: u64,
    pub stream_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = DeterministicRng::new(42, "test");
        let mut b = DeterministicRng::new(42, "test");
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_different_sequences() {
        let mut a = DeterministicRng::new(42, "test");
        let mut b = DeterministicRng::new(43, "test");
        let mut any_different = false;
        for _ in 0..10 {
            if a.next_u64() != b.next_u64() {
                any_different = true;
                break;
            }
        }
        assert!(any_different);
    }

    #[test]
    fn different_streams_different_sequences() {
        let mut a = DeterministicRng::new(42, "economy");
        let mut b = DeterministicRng::new(42, "agents");
        let mut any_different = false;
        for _ in 0..10 {
            if a.next_u64() != b.next_u64() {
                any_different = true;
                break;
            }
        }
        assert!(any_different);
    }

    #[test]
    fn child_rng_is_deterministic() {
        let parent = DeterministicRng::new(42, "root");
        let mut c1 = parent.child("economy");
        let mut c2 = DeterministicRng::new(42, "root").child("economy");
        for _ in 0..50 {
            assert_eq!(c1.next_u64(), c2.next_u64());
        }
    }

    #[test]
    fn next_fixed_in_range() {
        let mut rng = DeterministicRng::new(42, "test");
        for _ in 0..1000 {
            let v = rng.next_fixed();
            assert!(v.is_non_negative());
            assert!(v < Fixed64::ONE);
        }
    }
}

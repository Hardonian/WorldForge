//! # worldforge-proof
//!
//! Deterministic proof layer for World Forge simulation runs.
//! Provides run fingerprints, event hash chains, artifact hashing,
//! and verification reports.

use serde::{Deserialize, Serialize};
use worldforge_core::hash::{Fingerprint, FingerprintBuilder};
// EngineVersion used indirectly via RunProof fields

/// A cryptographic proof of a simulation run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunProof {
    pub run_id: String,
    pub engine_version: String,
    pub world_hash: Fingerprint,
    pub scenario_hash: Fingerprint,
    pub seed: u64,
    pub mod_hashes: Vec<Fingerprint>,
    pub initial_state_hash: Fingerprint,
    pub final_state_hash: Fingerprint,
    pub event_chain_root: Fingerprint,
    pub total_ticks: u64,
}

impl RunProof {
    /// Compute the overall proof fingerprint.
    pub fn fingerprint(&self) -> Fingerprint {
        let mut builder = FingerprintBuilder::new();
        builder.update(self.run_id.as_bytes());
        builder.update(self.engine_version.as_bytes());
        builder.update_fingerprint(&self.world_hash);
        builder.update_fingerprint(&self.scenario_hash);
        builder.update(&self.seed.to_le_bytes());
        for mod_hash in &self.mod_hashes {
            builder.update_fingerprint(mod_hash);
        }
        builder.update_fingerprint(&self.initial_state_hash);
        builder.update_fingerprint(&self.final_state_hash);
        builder.update_fingerprint(&self.event_chain_root);
        builder.update(&self.total_ticks.to_le_bytes());
        builder.finalize()
    }
}

/// Builds an event hash chain incrementally during simulation.
pub struct EventChain {
    current: Fingerprint,
    count: u64,
}

impl EventChain {
    pub fn new() -> Self {
        Self {
            current: Fingerprint::ZERO,
            count: 0,
        }
    }

    /// Add an event to the chain. The hash incorporates the previous chain state.
    pub fn add_event(&mut self, event_data: &[u8]) {
        self.current = self.current.chain_data(event_data);
        self.count += 1;
    }

    /// Add a serializable event to the chain.
    pub fn add_event_cbor<T: Serialize>(&mut self, event: &T) {
        let data = worldforge_core::serial::to_cbor(event).unwrap_or_default();
        self.add_event(&data);
    }

    /// Get the current chain root.
    pub fn root(&self) -> Fingerprint {
        self.current
    }

    /// Get the number of events in the chain.
    pub fn count(&self) -> u64 {
        self.count
    }
}

impl Default for EventChain {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of verifying a run proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationReport {
    pub run_id: String,
    pub checks: Vec<VerificationCheck>,
}

impl VerificationReport {
    pub fn new(run_id: &str) -> Self {
        Self {
            run_id: run_id.to_string(),
            checks: Vec::new(),
        }
    }

    pub fn add_check(&mut self, name: &str, passed: bool, detail: &str) {
        self.checks.push(VerificationCheck {
            name: name.to_string(),
            passed,
            detail: detail.to_string(),
        });
    }

    pub fn all_passed(&self) -> bool {
        self.checks.iter().all(|c| c.passed)
    }

    pub fn failed_checks(&self) -> Vec<&VerificationCheck> {
        self.checks.iter().filter(|c| !c.passed).collect()
    }
}

/// A single verification check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCheck {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

/// Future adapter trait for external proof backends.
pub trait ProofBackend: Send + Sync {
    fn store_proof(&self, proof: &RunProof) -> Result<(), String>;
    fn verify_proof(&self, proof: &RunProof) -> Result<VerificationReport, String>;
}

/// Default local proof backend (stores proofs as files).
pub struct LocalProofBackend;

impl ProofBackend for LocalProofBackend {
    fn store_proof(&self, _proof: &RunProof) -> Result<(), String> {
        // Local storage is handled by the replay system
        Ok(())
    }

    fn verify_proof(&self, proof: &RunProof) -> Result<VerificationReport, String> {
        let mut report = VerificationReport::new(&proof.run_id);
        // Basic structural checks
        report.add_check(
            "proof_structure",
            !proof.run_id.is_empty(),
            "proof has valid run ID",
        );
        report.add_check(
            "engine_version",
            !proof.engine_version.is_empty(),
            &format!("engine version: {}", proof.engine_version),
        );
        report.add_check(
            "state_hashes_differ",
            proof.initial_state_hash != proof.final_state_hash || proof.total_ticks == 0,
            "initial and final state hashes are distinct (simulation progressed)",
        );
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_chain_is_deterministic() {
        let mut chain1 = EventChain::new();
        let mut chain2 = EventChain::new();

        for i in 0u64..100 {
            let data = i.to_le_bytes();
            chain1.add_event(&data);
            chain2.add_event(&data);
        }

        assert_eq!(chain1.root(), chain2.root());
        assert_eq!(chain1.count(), 100);
    }

    #[test]
    fn event_chain_is_order_sensitive() {
        let mut chain1 = EventChain::new();
        chain1.add_event(b"event_a");
        chain1.add_event(b"event_b");

        let mut chain2 = EventChain::new();
        chain2.add_event(b"event_b");
        chain2.add_event(b"event_a");

        assert_ne!(chain1.root(), chain2.root());
    }

    #[test]
    fn run_proof_fingerprint_is_stable() {
        let proof = RunProof {
            run_id: "test-run".to_string(),
            engine_version: "0.1.0".to_string(),
            world_hash: Fingerprint::hash(b"world"),
            scenario_hash: Fingerprint::hash(b"scenario"),
            seed: 42,
            mod_hashes: vec![],
            initial_state_hash: Fingerprint::hash(b"initial"),
            final_state_hash: Fingerprint::hash(b"final"),
            event_chain_root: Fingerprint::hash(b"events"),
            total_ticks: 1000,
        };

        let fp1 = proof.fingerprint();
        let fp2 = proof.fingerprint();
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn verification_report() {
        let mut report = VerificationReport::new("test");
        report.add_check("check1", true, "ok");
        report.add_check("check2", false, "mismatch");
        assert!(!report.all_passed());
        assert_eq!(report.failed_checks().len(), 1);
    }
}

//! # worldforge-replay
//!
//! Replay recording and verification for World Forge simulations.
//!
//! A replay artifact captures everything needed to verify a simulation run:
//! format version, engine version, world/scenario/mod fingerprints,
//! seed, initial and final state fingerprints, and all tick events.

use serde::{Deserialize, Serialize};
use worldforge_core::hash::Fingerprint;
use worldforge_core::version::EngineVersion;
use worldforge_proof::{EventChain, RunProof};
use worldforge_world::SimulationEvent;

/// A complete replay artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayArtifact {
    pub format_version: String,
    pub engine_version: String,
    pub world_fingerprint: Fingerprint,
    pub scenario_fingerprint: Fingerprint,
    pub seed: u64,
    pub mod_fingerprints: Vec<Fingerprint>,
    pub initial_state_fingerprint: Fingerprint,
    pub final_state_fingerprint: Fingerprint,
    pub event_chain_root: Fingerprint,
    pub total_ticks: u64,
    pub events: Vec<SimulationEvent>,
    pub run_id: String,
    pub timestamp: String,
}

impl ReplayArtifact {
    /// Serialize to CBOR bytes.
    pub fn to_cbor(&self) -> Vec<u8> {
        worldforge_core::serial::to_cbor(self).expect("replay serialization failed")
    }

    /// Deserialize from CBOR bytes.
    pub fn from_cbor(data: &[u8]) -> Result<Self, worldforge_core::error::WorldForgeError> {
        worldforge_core::serial::from_cbor(data).map_err(|e| {
            worldforge_core::error::WorldForgeError::new(
                worldforge_core::error::ErrorCode::ReplayFormatInvalid,
                format!("failed to deserialize replay: {}", e),
            )
        })
    }

    /// Save to a file.
    pub fn save(
        &self,
        path: &std::path::Path,
    ) -> Result<(), worldforge_core::error::WorldForgeError> {
        let data = self.to_cbor();
        std::fs::write(path, &data).map_err(|e| {
            worldforge_core::error::WorldForgeError::new(
                worldforge_core::error::ErrorCode::ReplayFormatInvalid,
                format!("failed to write replay: {}", e),
            )
        })
    }

    /// Load from a file.
    pub fn load(path: &std::path::Path) -> Result<Self, worldforge_core::error::WorldForgeError> {
        let data = std::fs::read(path).map_err(|e| {
            worldforge_core::error::WorldForgeError::new(
                worldforge_core::error::ErrorCode::ReplayFormatInvalid,
                format!("failed to read replay: {}", e),
            )
        })?;
        Self::from_cbor(&data)
    }

    /// Extract a RunProof from this replay.
    pub fn to_proof(&self) -> RunProof {
        RunProof {
            run_id: self.run_id.clone(),
            engine_version: self.engine_version.clone(),
            world_hash: self.world_fingerprint,
            scenario_hash: self.scenario_fingerprint,
            seed: self.seed,
            mod_hashes: self.mod_fingerprints.clone(),
            initial_state_hash: self.initial_state_fingerprint,
            final_state_hash: self.final_state_fingerprint,
            event_chain_root: self.event_chain_root,
            total_ticks: self.total_ticks,
        }
    }

    /// Verify internal consistency of the replay.
    pub fn verify_internal(&self) -> worldforge_proof::VerificationReport {
        let mut report = worldforge_proof::VerificationReport::new(&self.run_id);

        // Verify event chain
        let mut chain = EventChain::new();
        for event in &self.events {
            chain.add_event_cbor(event);
        }
        let chain_matches = chain.root() == self.event_chain_root;
        report.add_check(
            "event_chain",
            chain_matches,
            if chain_matches {
                "event chain root matches".to_string()
            } else {
                format!(
                    "event chain mismatch: expected {}, got {}",
                    self.event_chain_root.to_short_hex(),
                    chain.root().to_short_hex()
                )
            }
            .as_str(),
        );

        // Check engine version
        let current = EngineVersion::current();
        let version_compatible = self.engine_version == current.version.to_string();
        report.add_check(
            "engine_version",
            version_compatible,
            &format!(
                "replay engine: {}, current: {}",
                self.engine_version, current
            ),
        );

        // Check format version
        let expected_format = worldforge_core::version::formats::replay_format();
        let format_ok = self.format_version == expected_format.version.to_string();
        report.add_check(
            "format_version",
            format_ok,
            &format!("replay format: {}", self.format_version),
        );

        report
    }
}

/// Builder for creating replay artifacts during simulation.
pub struct ReplayWriter {
    events: Vec<SimulationEvent>,
    event_chain: EventChain,
    world_fingerprint: Fingerprint,
    scenario_fingerprint: Fingerprint,
    seed: u64,
    mod_fingerprints: Vec<Fingerprint>,
    initial_state_fingerprint: Fingerprint,
    run_id: String,
}

impl ReplayWriter {
    pub fn new(
        run_id: String,
        world_fingerprint: Fingerprint,
        scenario_fingerprint: Fingerprint,
        seed: u64,
        initial_state_fingerprint: Fingerprint,
    ) -> Self {
        Self {
            events: Vec::new(),
            event_chain: EventChain::new(),
            world_fingerprint,
            scenario_fingerprint,
            seed,
            mod_fingerprints: Vec::new(),
            initial_state_fingerprint,
            run_id,
        }
    }

    /// Record an event.
    pub fn record_event(&mut self, event: SimulationEvent) {
        self.event_chain.add_event_cbor(&event);
        self.events.push(event);
    }

    /// Record multiple events.
    pub fn record_events(&mut self, events: &[SimulationEvent]) {
        for event in events {
            self.record_event(event.clone());
        }
    }

    /// Finalize the replay with the final state fingerprint.
    pub fn finalize(
        self,
        final_state_fingerprint: Fingerprint,
        total_ticks: u64,
    ) -> ReplayArtifact {
        let engine = EngineVersion::current();
        let format = worldforge_core::version::formats::replay_format();

        ReplayArtifact {
            format_version: format.version.to_string(),
            engine_version: engine.version.to_string(),
            world_fingerprint: self.world_fingerprint,
            scenario_fingerprint: self.scenario_fingerprint,
            seed: self.seed,
            mod_fingerprints: self.mod_fingerprints,
            initial_state_fingerprint: self.initial_state_fingerprint,
            final_state_fingerprint: final_state_fingerprint,
            event_chain_root: self.event_chain.root(),
            total_ticks,
            events: self.events,
            run_id: self.run_id,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worldforge_core::Tick;
    use worldforge_world::EventType;

    #[test]
    fn replay_roundtrip() {
        let mut writer = ReplayWriter::new(
            "test-run".to_string(),
            Fingerprint::hash(b"world"),
            Fingerprint::hash(b"scenario"),
            42,
            Fingerprint::hash(b"initial"),
        );

        writer.record_event(SimulationEvent::new(
            Tick::new(1),
            EventType::ProductionCompleted {
                entity: "mine".to_string(),
                resource: "ore".to_string(),
                amount: worldforge_core::Fixed64::from_int(10),
            },
        ));

        let replay = writer.finalize(Fingerprint::hash(b"final"), 100);
        let data = replay.to_cbor();
        let restored = ReplayArtifact::from_cbor(&data).unwrap();

        assert_eq!(restored.seed, 42);
        assert_eq!(restored.events.len(), 1);
        assert_eq!(restored.event_chain_root, replay.event_chain_root);
    }

    #[test]
    fn replay_verification_passes() {
        let mut writer = ReplayWriter::new(
            "test".to_string(),
            Fingerprint::hash(b"w"),
            Fingerprint::hash(b"s"),
            42,
            Fingerprint::hash(b"i"),
        );
        writer.record_event(SimulationEvent::new(
            Tick::new(1),
            EventType::SimulationDegraded {
                reason: "test".to_string(),
            },
        ));
        let replay = writer.finalize(Fingerprint::hash(b"f"), 1);

        let report = replay.verify_internal();
        assert!(
            report
                .checks
                .iter()
                .find(|c| c.name == "event_chain")
                .unwrap()
                .passed
        );
    }

    #[test]
    fn tampered_replay_fails_verification() {
        let mut writer = ReplayWriter::new(
            "test".to_string(),
            Fingerprint::hash(b"w"),
            Fingerprint::hash(b"s"),
            42,
            Fingerprint::hash(b"i"),
        );
        writer.record_event(SimulationEvent::new(
            Tick::new(1),
            EventType::SimulationDegraded {
                reason: "original".to_string(),
            },
        ));
        let mut replay = writer.finalize(Fingerprint::hash(b"f"), 1);

        // Tamper with the events
        replay.events[0] = SimulationEvent::new(
            Tick::new(1),
            EventType::SimulationDegraded {
                reason: "tampered".to_string(),
            },
        );

        let report = replay.verify_internal();
        let chain_check = report
            .checks
            .iter()
            .find(|c| c.name == "event_chain")
            .unwrap();
        assert!(!chain_check.passed); // Tampered replay should fail
    }
}

//! # worldforge-mod-api
//!
//! Capability-based API definitions for World Forge mods.
//! Mods never receive unrestricted host access. All capabilities
//! are explicitly granted via a deny-by-default policy.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use worldforge_core::FeatureStatus;

/// A capability that can be granted to a mod.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Capability {
    // World data access
    #[serde(rename = "world.entity.read")]
    EntityRead,
    #[serde(rename = "world.entity.write")]
    EntityWrite,
    #[serde(rename = "world.resource.read")]
    ResourceRead,
    #[serde(rename = "world.resource.write")]
    ResourceWrite,
    #[serde(rename = "world.event.emit")]
    EventEmit,
    #[serde(rename = "world.economy.read")]
    EconomyRead,
    #[serde(rename = "world.economy.transact")]
    EconomyTransact,
    #[serde(rename = "world.agent.read")]
    AgentRead,
    #[serde(rename = "world.objective.read")]
    ObjectiveRead,
    #[serde(rename = "world.telemetry.write")]
    TelemetryWrite,
}

/// Capabilities that are always denied for mods.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeniedCapability {
    Filesystem,
    Network,
    Shell,
    Environment,
    Process,
    Secrets,
}

impl DeniedCapability {
    pub fn all() -> Vec<DeniedCapability> {
        vec![
            DeniedCapability::Filesystem,
            DeniedCapability::Network,
            DeniedCapability::Shell,
            DeniedCapability::Environment,
            DeniedCapability::Process,
            DeniedCapability::Secrets,
        ]
    }
}

/// A mod's manifest declaring its identity and requested capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub capabilities: Vec<Capability>,
    /// Future: cryptographic signature for signed mod manifests.
    #[serde(skip)]
    pub signature: Option<Vec<u8>>,
}

/// A capability policy that evaluates access requests.
#[derive(Debug, Clone)]
pub struct CapabilityPolicy {
    granted: BTreeSet<Capability>,
}

impl CapabilityPolicy {
    /// Create a new policy with no capabilities granted (deny-by-default).
    pub fn deny_all() -> Self {
        Self {
            granted: BTreeSet::new(),
        }
    }

    /// Create a policy granting specific capabilities.
    pub fn with_capabilities(caps: Vec<Capability>) -> Self {
        Self {
            granted: caps.into_iter().collect(),
        }
    }

    /// Grant a capability.
    pub fn grant(&mut self, cap: Capability) {
        self.granted.insert(cap);
    }

    /// Check if a capability is granted.
    pub fn check(&self, cap: &Capability) -> bool {
        self.granted.contains(cap)
    }

    /// Require a capability, returning an error if denied.
    pub fn require(&self, cap: &Capability) -> Result<(), worldforge_core::error::WorldForgeError> {
        if self.check(cap) {
            Ok(())
        } else {
            Err(worldforge_core::error::WorldForgeError::new(
                worldforge_core::error::ErrorCode::ModCapabilityDenied,
                format!("capability {:?} is not granted", cap),
            ))
        }
    }

    /// Check a denied (system) capability — always returns denied.
    pub fn check_denied(&self, cap: &DeniedCapability) -> FeatureStatus<()> {
        FeatureStatus::Unavailable {
            reason: match cap {
                DeniedCapability::Filesystem => "filesystem access denied for mods",
                DeniedCapability::Network => "network access denied for mods",
                DeniedCapability::Shell => "shell access denied for mods",
                DeniedCapability::Environment => "environment variable access denied for mods",
                DeniedCapability::Process => "process spawning denied for mods",
                DeniedCapability::Secrets => "secrets access denied for mods",
            },
        }
    }
}

/// Mod lifecycle callbacks.
pub trait ModCallbacks {
    fn init(&mut self) -> Result<(), String>;
    fn on_tick(&mut self, tick: u64) -> Result<(), String>;
    fn on_event(&mut self, event_data: &[u8]) -> Result<(), String>;
    fn shutdown(&mut self) -> Result<(), String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deny_all_denies_everything() {
        let policy = CapabilityPolicy::deny_all();
        assert!(!policy.check(&Capability::EntityRead));
        assert!(!policy.check(&Capability::EntityWrite));
        assert!(!policy.check(&Capability::ResourceRead));
    }

    #[test]
    fn selective_grant() {
        let policy =
            CapabilityPolicy::with_capabilities(vec![Capability::EntityRead, Capability::ResourceRead]);
        assert!(policy.check(&Capability::EntityRead));
        assert!(policy.check(&Capability::ResourceRead));
        assert!(!policy.check(&Capability::EntityWrite));
    }

    #[test]
    fn system_capabilities_always_denied() {
        let policy = CapabilityPolicy::deny_all();
        for denied in DeniedCapability::all() {
            assert!(!policy.check_denied(&denied).is_available());
        }
    }

    #[test]
    fn require_denied_produces_error() {
        let policy = CapabilityPolicy::deny_all();
        let result = policy.require(&Capability::EntityWrite);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, worldforge_core::error::ErrorCode::ModCapabilityDenied);
    }
}

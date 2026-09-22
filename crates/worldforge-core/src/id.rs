//! Strongly-typed identifiers for World Forge entities.
//!
//! Every domain concept gets its own ID type to prevent accidental misuse
//! (e.g., passing a `WorldId` where an `EntityId` is expected).

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Macro to generate a strongly-typed ID wrapper around UUID.
macro_rules! define_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(Uuid);

        impl $name {
            /// Create a new random ID.
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            /// Create an ID from raw UUID bytes.
            pub fn from_bytes(bytes: [u8; 16]) -> Self {
                Self(Uuid::from_bytes(bytes))
            }

            /// Create a deterministic ID from a seed and index.
            /// Useful for reproducible test scenarios.
            pub fn deterministic(seed: u64, index: u64) -> Self {
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                stringify!($name).hash(&mut hasher);
                seed.hash(&mut hasher);
                index.hash(&mut hasher);
                let h = hasher.finish();

                let mut bytes = [0u8; 16];
                bytes[..8].copy_from_slice(&h.to_le_bytes());
                bytes[8..16].copy_from_slice(&index.to_le_bytes());
                // Set UUID version 4 variant bits for validity
                bytes[6] = (bytes[6] & 0x0f) | 0x40;
                bytes[8] = (bytes[8] & 0x3f) | 0x80;
                Self(Uuid::from_bytes(bytes))
            }

            /// Get the underlying UUID.
            pub fn as_uuid(&self) -> &Uuid {
                &self.0
            }

            /// Get the raw bytes.
            pub fn as_bytes(&self) -> &[u8; 16] {
                self.0.as_bytes()
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), &self.0.to_string()[..8])
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

define_id!(
    /// Unique identifier for a world definition.
    WorldId
);

define_id!(
    /// Unique identifier for an entity within a simulation.
    EntityId
);

define_id!(
    /// Unique identifier for a registered system.
    SystemId
);

define_id!(
    /// Unique identifier for a resource type.
    ResourceId
);

define_id!(
    /// Unique identifier for a scenario.
    ScenarioId
);

define_id!(
    /// Unique identifier for a simulation run.
    RunId
);

define_id!(
    /// Unique identifier for a mod.
    ModId
);

define_id!(
    /// Unique identifier for a region.
    RegionId
);

define_id!(
    /// Unique identifier for an organization.
    OrganizationId
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_ids_are_stable() {
        let a = EntityId::deterministic(42, 0);
        let b = EntityId::deterministic(42, 0);
        assert_eq!(a, b);
    }

    #[test]
    fn deterministic_ids_differ_by_index() {
        let a = EntityId::deterministic(42, 0);
        let b = EntityId::deterministic(42, 1);
        assert_ne!(a, b);
    }

    #[test]
    fn different_id_types_are_not_interchangeable() {
        // This test is really a compile-time check. If EntityId and WorldId
        // were the same type, the type system wouldn't prevent misuse.
        let _e: EntityId = EntityId::deterministic(1, 0);
        let _w: WorldId = WorldId::deterministic(1, 0);
        // They have different types even with same seed/index
    }

    #[test]
    fn id_debug_format_is_short() {
        let id = EntityId::deterministic(42, 0);
        let debug = format!("{:?}", id);
        assert!(debug.starts_with("EntityId("));
        assert!(debug.len() < 30); // Short form, not full UUID
    }

    #[test]
    fn id_serialization_roundtrip() {
        let id = WorldId::deterministic(99, 5);
        let json = serde_json::to_string(&id).unwrap();
        let restored: WorldId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, restored);
    }
}

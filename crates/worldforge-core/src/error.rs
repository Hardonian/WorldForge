//! Structured error types for World Forge.
//!
//! Every error has a stable error code (e.g., WF1001) for documentation,
//! search, and programmatic handling. Errors are grouped by subsystem:
//!
//! - WF1xxx: World/manifest errors
//! - WF2xxx: Mod errors
//! - WF3xxx: Replay errors
//! - WF4xxx: Economy/resource errors
//! - WF5xxx: Runtime errors
//! - WF6xxx: Package errors
//! - WF9xxx: Internal errors

use std::fmt;

/// Stable error codes for World Forge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ErrorCode {
    // World / Manifest (1xxx)
    WorldManifestInvalid,
    WorldManifestMissing,
    WorldNotFound,
    WorldSchemaViolation,
    ScenarioInvalid,
    ScenarioMissing,

    // Mod (2xxx)
    ModLoadFailed,
    ModInitFailed,
    ModCapabilityDenied,
    ModExecutionFailed,
    ModFuelExhausted,
    ModTrap,

    // Replay (3xxx)
    ReplayFormatInvalid,
    ReplayHashMismatch,
    ReplayVersionIncompatible,
    ReplayEventChainCorrupted,
    ReplayFingerprintMismatch,

    // Economy / Resources (4xxx)
    ResourceInvariantFailed,
    InsufficientResource,
    TransferFailed,
    ProductionFailed,
    NegativeInventory,

    // Runtime (5xxx)
    RuntimeInitFailed,
    RuntimeTickFailed,
    RuntimeStateMismatch,
    SystemRegistrationFailed,
    SimulationDegraded,

    // Package (6xxx)
    PackageBuildFailed,
    PackageInvalid,
    PackageFingerprintMismatch,
    PackageDependencyMissing,

    // Internal (9xxx)
    InternalError,
    NotImplemented,
}

impl ErrorCode {
    /// Get the numeric code for this error.
    pub fn code(&self) -> u16 {
        match self {
            // World
            ErrorCode::WorldManifestInvalid => 1001,
            ErrorCode::WorldManifestMissing => 1002,
            ErrorCode::WorldNotFound => 1003,
            ErrorCode::WorldSchemaViolation => 1004,
            ErrorCode::ScenarioInvalid => 1005,
            ErrorCode::ScenarioMissing => 1006,
            // Mod
            ErrorCode::ModLoadFailed => 2001,
            ErrorCode::ModInitFailed => 2002,
            ErrorCode::ModCapabilityDenied => 2004,
            ErrorCode::ModExecutionFailed => 2005,
            ErrorCode::ModFuelExhausted => 2006,
            ErrorCode::ModTrap => 2007,
            // Replay
            ErrorCode::ReplayFormatInvalid => 3001,
            ErrorCode::ReplayHashMismatch => 3002,
            ErrorCode::ReplayVersionIncompatible => 3003,
            ErrorCode::ReplayEventChainCorrupted => 3004,
            ErrorCode::ReplayFingerprintMismatch => 3005,
            // Economy
            ErrorCode::ResourceInvariantFailed => 4001,
            ErrorCode::InsufficientResource => 4002,
            ErrorCode::TransferFailed => 4003,
            ErrorCode::ProductionFailed => 4004,
            ErrorCode::NegativeInventory => 4005,
            // Runtime
            ErrorCode::RuntimeInitFailed => 5001,
            ErrorCode::RuntimeTickFailed => 5002,
            ErrorCode::RuntimeStateMismatch => 5003,
            ErrorCode::SystemRegistrationFailed => 5004,
            ErrorCode::SimulationDegraded => 5005,
            // Package
            ErrorCode::PackageBuildFailed => 6001,
            ErrorCode::PackageInvalid => 6002,
            ErrorCode::PackageFingerprintMismatch => 6003,
            ErrorCode::PackageDependencyMissing => 6004,
            // Internal
            ErrorCode::InternalError => 9001,
            ErrorCode::NotImplemented => 9002,
        }
    }

    /// Get the string identifier for this error.
    pub fn name(&self) -> &'static str {
        match self {
            ErrorCode::WorldManifestInvalid => "WORLD_MANIFEST_INVALID",
            ErrorCode::WorldManifestMissing => "WORLD_MANIFEST_MISSING",
            ErrorCode::WorldNotFound => "WORLD_NOT_FOUND",
            ErrorCode::WorldSchemaViolation => "WORLD_SCHEMA_VIOLATION",
            ErrorCode::ScenarioInvalid => "SCENARIO_INVALID",
            ErrorCode::ScenarioMissing => "SCENARIO_MISSING",
            ErrorCode::ModLoadFailed => "MOD_LOAD_FAILED",
            ErrorCode::ModInitFailed => "MOD_INIT_FAILED",
            ErrorCode::ModCapabilityDenied => "MOD_CAPABILITY_DENIED",
            ErrorCode::ModExecutionFailed => "MOD_EXECUTION_FAILED",
            ErrorCode::ModFuelExhausted => "MOD_FUEL_EXHAUSTED",
            ErrorCode::ModTrap => "MOD_TRAP",
            ErrorCode::ReplayFormatInvalid => "REPLAY_FORMAT_INVALID",
            ErrorCode::ReplayHashMismatch => "REPLAY_HASH_MISMATCH",
            ErrorCode::ReplayVersionIncompatible => "REPLAY_VERSION_INCOMPATIBLE",
            ErrorCode::ReplayEventChainCorrupted => "REPLAY_EVENT_CHAIN_CORRUPTED",
            ErrorCode::ReplayFingerprintMismatch => "REPLAY_FINGERPRINT_MISMATCH",
            ErrorCode::ResourceInvariantFailed => "RESOURCE_INVARIANT_FAILED",
            ErrorCode::InsufficientResource => "INSUFFICIENT_RESOURCE",
            ErrorCode::TransferFailed => "TRANSFER_FAILED",
            ErrorCode::ProductionFailed => "PRODUCTION_FAILED",
            ErrorCode::NegativeInventory => "NEGATIVE_INVENTORY",
            ErrorCode::RuntimeInitFailed => "RUNTIME_INIT_FAILED",
            ErrorCode::RuntimeTickFailed => "RUNTIME_TICK_FAILED",
            ErrorCode::RuntimeStateMismatch => "RUNTIME_STATE_MISMATCH",
            ErrorCode::SystemRegistrationFailed => "SYSTEM_REGISTRATION_FAILED",
            ErrorCode::SimulationDegraded => "SIMULATION_DEGRADED",
            ErrorCode::PackageBuildFailed => "PACKAGE_BUILD_FAILED",
            ErrorCode::PackageInvalid => "PACKAGE_INVALID",
            ErrorCode::PackageFingerprintMismatch => "PACKAGE_FINGERPRINT_MISMATCH",
            ErrorCode::PackageDependencyMissing => "PACKAGE_DEPENDENCY_MISSING",
            ErrorCode::InternalError => "INTERNAL_ERROR",
            ErrorCode::NotImplemented => "NOT_IMPLEMENTED",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WF{:04}", self.code())
    }
}

/// The primary error type for World Forge operations.
#[derive(Debug, thiserror::Error)]
#[error("[{code}] {code_name}: {message}")]
pub struct WorldForgeError {
    pub code: ErrorCode,
    pub code_name: &'static str,
    pub message: String,
    #[source]
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl WorldForgeError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code_name: code.name(),
            code,
            message: message.into(),
            source: None,
        }
    }

    pub fn with_source(
        code: ErrorCode,
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            code_name: code.name(),
            code,
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
}

/// Convenience type alias for World Forge results.
pub type Result<T> = std::result::Result<T, WorldForgeError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_code_formatting() {
        assert_eq!(format!("{}", ErrorCode::WorldManifestInvalid), "WF1001");
        assert_eq!(format!("{}", ErrorCode::ModCapabilityDenied), "WF2004");
        assert_eq!(format!("{}", ErrorCode::ReplayHashMismatch), "WF3002");
        assert_eq!(format!("{}", ErrorCode::ResourceInvariantFailed), "WF4001");
    }

    #[test]
    fn error_display() {
        let err = WorldForgeError::new(
            ErrorCode::WorldManifestInvalid,
            "missing required field 'name'",
        );
        let msg = format!("{}", err);
        assert!(msg.contains("WF1001"));
        assert!(msg.contains("WORLD_MANIFEST_INVALID"));
        assert!(msg.contains("missing required field 'name'"));
    }
}

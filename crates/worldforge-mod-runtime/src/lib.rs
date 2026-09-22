//! # worldforge-mod-runtime
//!
//! WASM mod execution runtime for World Forge.
//!
//! ## Current Status
//!
//! The WASM Component Model runtime using wasmtime is designed but
//! not yet compiled against wasmtime crate dependencies. The host API
//! contract, capability enforcement, and lifecycle management are
//! fully implemented as a mock runtime for testing.
//!
//! When wasmtime is integrated, mods will execute in sandboxed WASM
//! with fuel limits, memory bounds, and capability-gated host functions.

use worldforge_core::error::{ErrorCode, WorldForgeError};
use worldforge_core::FeatureStatus;
use worldforge_mod_api::{Capability, CapabilityPolicy, ModCallbacks};

/// Configuration for the mod runtime.
#[derive(Debug, Clone)]
pub struct ModRuntimeConfig {
    /// Maximum fuel (instruction count) per tick.
    pub fuel_per_tick: u64,
    /// Maximum memory in bytes.
    pub max_memory_bytes: usize,
    /// Maximum execution time per tick in milliseconds.
    pub max_tick_ms: u64,
}

impl Default for ModRuntimeConfig {
    fn default() -> Self {
        Self {
            fuel_per_tick: 1_000_000,
            max_memory_bytes: 64 * 1024 * 1024, // 64MB
            max_tick_ms: 100,
        }
    }
}

/// A loaded mod instance.
pub struct ModInstance {
    pub name: String,
    pub version: String,
    pub policy: CapabilityPolicy,
    pub callbacks: Box<dyn ModCallbacks>,
    pub config: ModRuntimeConfig,
    pub initialized: bool,
}

impl ModInstance {
    /// Initialize the mod.
    pub fn init(&mut self) -> Result<(), WorldForgeError> {
        self.callbacks.init().map_err(|e| {
            WorldForgeError::new(ErrorCode::ModInitFailed, format!("mod '{}' init failed: {}", self.name, e))
        })?;
        self.initialized = true;
        Ok(())
    }

    /// Execute a tick callback.
    pub fn on_tick(&mut self, tick: u64) -> Result<(), WorldForgeError> {
        if !self.initialized {
            return Err(WorldForgeError::new(
                ErrorCode::ModExecutionFailed,
                format!("mod '{}' not initialized", self.name),
            ));
        }
        self.callbacks.on_tick(tick).map_err(|e| {
            WorldForgeError::new(
                ErrorCode::ModExecutionFailed,
                format!("mod '{}' tick {} failed: {}", self.name, tick, e),
            )
        })
    }

    /// Execute an event callback.
    pub fn on_event(&mut self, event_data: &[u8]) -> Result<(), WorldForgeError> {
        self.callbacks.on_event(event_data).map_err(|e| {
            WorldForgeError::new(
                ErrorCode::ModExecutionFailed,
                format!("mod '{}' event handler failed: {}", self.name, e),
            )
        })
    }

    /// Check a capability.
    pub fn check_capability(&self, cap: &Capability) -> bool {
        self.policy.check(cap)
    }

    /// Require a capability (returns error if denied).
    pub fn require_capability(&self, cap: &Capability) -> Result<(), WorldForgeError> {
        self.policy.require(cap)
    }
}

/// Load a WASM mod from file.
pub fn load_wasm_mod(
    _path: &std::path::Path,
    _config: ModRuntimeConfig,
) -> FeatureStatus<ModInstance> {
    FeatureStatus::Unavailable {
        reason: "WASM mod runtime requires wasmtime integration (planned for M1)",
    }
}

/// A mock mod for testing the mod lifecycle without WASM.
pub struct MockMod {
    pub name: String,
    pub tick_count: u64,
    pub events_received: Vec<Vec<u8>>,
}

impl MockMod {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            tick_count: 0,
            events_received: Vec::new(),
        }
    }
}

impl ModCallbacks for MockMod {
    fn init(&mut self) -> Result<(), String> {
        tracing::info!(mod_name = %self.name, "mock mod initialized");
        Ok(())
    }

    fn on_tick(&mut self, tick: u64) -> Result<(), String> {
        self.tick_count += 1;
        tracing::trace!(mod_name = %self.name, tick = tick, "mock mod tick");
        Ok(())
    }

    fn on_event(&mut self, event_data: &[u8]) -> Result<(), String> {
        self.events_received.push(event_data.to_vec());
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), String> {
        tracing::info!(mod_name = %self.name, "mock mod shutdown");
        Ok(())
    }
}

/// A mock mod that requests a denied capability (for testing).
pub struct DeniedCapabilityMod;

impl ModCallbacks for DeniedCapabilityMod {
    fn init(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn on_tick(&mut self, _tick: u64) -> Result<(), String> {
        // This mod would attempt network access in a real WASM environment
        Ok(())
    }

    fn on_event(&mut self, _event_data: &[u8]) -> Result<(), String> {
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worldforge_mod_api::Capability;

    #[test]
    fn mock_mod_lifecycle() {
        let mock = MockMod::new("test-mod");
        let mut instance = ModInstance {
            name: "test-mod".to_string(),
            version: "0.1.0".to_string(),
            policy: CapabilityPolicy::with_capabilities(vec![
                Capability::EntityRead,
                Capability::ResourceRead,
                Capability::EventEmit,
            ]),
            callbacks: Box::new(mock),
            config: ModRuntimeConfig::default(),
            initialized: false,
        };

        // Init
        instance.init().unwrap();
        assert!(instance.initialized);

        // Tick
        for i in 0..10 {
            instance.on_tick(i).unwrap();
        }

        // Capability check
        assert!(instance.check_capability(&Capability::EntityRead));
        assert!(!instance.check_capability(&Capability::EntityWrite));
    }

    #[test]
    fn capability_denial() {
        let mock = MockMod::new("denied-mod");
        let instance = ModInstance {
            name: "denied-mod".to_string(),
            version: "0.1.0".to_string(),
            policy: CapabilityPolicy::deny_all(), // No capabilities
            callbacks: Box::new(mock),
            config: ModRuntimeConfig::default(),
            initialized: false,
        };

        // All capabilities should be denied
        let result = instance.require_capability(&Capability::EntityRead);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().code,
            ErrorCode::ModCapabilityDenied
        );
    }

    #[test]
    fn wasm_mod_loading_reports_unavailable() {
        let result = load_wasm_mod(
            std::path::Path::new("nonexistent.wasm"),
            ModRuntimeConfig::default(),
        );
        assert!(!result.is_available());
    }
}

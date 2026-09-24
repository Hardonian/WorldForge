//! # worldforge-mod-runtime
//!
//! WASM mod execution runtime for World Forge.
//!
//! Wasmtime executes core WebAssembly modules without WASI, so filesystem,
//! network, environment, process, and shell APIs are absent. The deliberately
//! small v0 host ABI is capability-gated and execution is bounded by fuel and
//! linear-memory limits. The versioned Component Model contract lives in `/wit`.

use wasmtime::{
    Caller, Config, Engine, Linker, Module, Store, StoreLimits, StoreLimitsBuilder, TypedFunc,
};
use worldforge_core::error::{ErrorCode, WorldForgeError};
use worldforge_mod_api::{Capability, CapabilityPolicy, ModCallbacks};

/// Configuration for the mod runtime.
#[derive(Debug, Clone)]
pub struct ModRuntimeConfig {
    /// Maximum fuel (instruction count) per tick.
    pub fuel_per_tick: u64,
    /// Maximum memory in bytes.
    pub max_memory_bytes: usize,
}

impl Default for ModRuntimeConfig {
    fn default() -> Self {
        Self {
            fuel_per_tick: 1_000_000,
            max_memory_bytes: 64 * 1024 * 1024, // 64MB
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
            WorldForgeError::new(
                ErrorCode::ModInitFailed,
                format!("mod '{}' init failed: {}", self.name, e),
            )
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

struct WasmHostState {
    policy: CapabilityPolicy,
    resource_amount: i64,
    emitted_events: Vec<i64>,
    limits: StoreLimits,
    capability_denied: bool,
}

/// A loaded, sandboxed WebAssembly mod using the minimal v0 core ABI.
pub struct WasmModInstance {
    store: Store<WasmHostState>,
    init: TypedFunc<(), ()>,
    on_tick: TypedFunc<i64, ()>,
    on_event: TypedFunc<i64, ()>,
    config: ModRuntimeConfig,
}

impl WasmModInstance {
    /// Load a module from bytes. No WASI interfaces are linked.
    pub fn from_bytes(
        bytes: &[u8],
        policy: CapabilityPolicy,
        config: ModRuntimeConfig,
        resource_amount: i64,
    ) -> Result<Self, WorldForgeError> {
        let mut engine_config = Config::new();
        engine_config.consume_fuel(true);
        let engine = Engine::new(&engine_config)
            .map_err(|e| WorldForgeError::new(ErrorCode::ModLoadFailed, e.to_string()))?;
        let module = Module::new(&engine, bytes)
            .map_err(|e| WorldForgeError::new(ErrorCode::ModLoadFailed, e.to_string()))?;
        let mut linker = Linker::new(&engine);
        linker
            .func_wrap(
                "worldforge",
                "read_resource",
                |mut caller: Caller<'_, WasmHostState>| -> wasmtime::Result<i64> {
                    if let Err(error) = caller.data().policy.require(&Capability::ResourceRead) {
                        caller.data_mut().capability_denied = true;
                        return Err(wasmtime::Error::msg(error.to_string()));
                    }
                    Ok(caller.data().resource_amount)
                },
            )
            .map_err(|e| WorldForgeError::new(ErrorCode::ModLoadFailed, e.to_string()))?;
        linker
            .func_wrap(
                "worldforge",
                "emit_event",
                |mut caller: Caller<'_, WasmHostState>, event: i64| -> wasmtime::Result<()> {
                    if let Err(error) = caller.data().policy.require(&Capability::EventEmit) {
                        caller.data_mut().capability_denied = true;
                        return Err(wasmtime::Error::msg(error.to_string()));
                    }
                    caller.data_mut().emitted_events.push(event);
                    Ok(())
                },
            )
            .map_err(|e| WorldForgeError::new(ErrorCode::ModLoadFailed, e.to_string()))?;

        let limits = StoreLimitsBuilder::new()
            .memory_size(config.max_memory_bytes)
            .instances(1)
            .memories(1)
            .build();
        let mut store = Store::new(
            &engine,
            WasmHostState {
                policy,
                resource_amount,
                emitted_events: Vec::new(),
                limits,
                capability_denied: false,
            },
        );
        store.limiter(|state| &mut state.limits);
        store
            .set_fuel(config.fuel_per_tick)
            .map_err(|e| WorldForgeError::new(ErrorCode::ModLoadFailed, e.to_string()))?;
        let instance = linker.instantiate(&mut store, &module).map_err(|e| {
            WorldForgeError::new(
                ErrorCode::ModCapabilityDenied,
                format!("mod imports denied host API: {e}"),
            )
        })?;
        let init = instance
            .get_typed_func::<(), ()>(&mut store, "init")
            .map_err(|e| WorldForgeError::new(ErrorCode::ModLoadFailed, e.to_string()))?;
        let on_tick = instance
            .get_typed_func::<i64, ()>(&mut store, "on_tick")
            .map_err(|e| WorldForgeError::new(ErrorCode::ModLoadFailed, e.to_string()))?;
        let on_event = instance
            .get_typed_func::<i64, ()>(&mut store, "on_event")
            .map_err(|e| WorldForgeError::new(ErrorCode::ModLoadFailed, e.to_string()))?;
        Ok(Self {
            store,
            init,
            on_tick,
            on_event,
            config,
        })
    }

    fn prepare_call(&mut self) -> Result<(), WorldForgeError> {
        self.store.data_mut().capability_denied = false;
        self.store
            .set_fuel(self.config.fuel_per_tick)
            .map_err(|e| WorldForgeError::new(ErrorCode::ModExecutionFailed, e.to_string()))
    }

    pub fn init(&mut self) -> Result<(), WorldForgeError> {
        self.prepare_call()?;
        let result = self.init.call(&mut self.store, ());
        self.map_call_result(result)
    }

    pub fn on_tick(&mut self, tick: u64) -> Result<(), WorldForgeError> {
        self.prepare_call()?;
        let result = self.on_tick.call(&mut self.store, tick as i64);
        self.map_call_result(result)
    }

    pub fn on_event(&mut self, event: i64) -> Result<(), WorldForgeError> {
        self.prepare_call()?;
        let result = self.on_event.call(&mut self.store, event);
        self.map_call_result(result)
    }

    pub fn emitted_events(&self) -> &[i64] {
        &self.store.data().emitted_events
    }

    /// Drain numeric events emitted since the previous host synchronization.
    pub fn take_emitted_events(&mut self) -> Vec<i64> {
        std::mem::take(&mut self.store.data_mut().emitted_events)
    }

    /// Refresh the deterministic aggregate value exposed by the v0
    /// `read_resource` host function before invoking a callback.
    pub fn set_resource_amount(&mut self, amount: i64) {
        self.store.data_mut().resource_amount = amount;
    }

    fn map_call_result(&self, result: wasmtime::Result<()>) -> Result<(), WorldForgeError> {
        result.map_err(|error| {
            if self.store.data().capability_denied {
                WorldForgeError::new(ErrorCode::ModCapabilityDenied, error.to_string())
            } else {
                map_wasm_trap(error)
            }
        })
    }
}

fn map_wasm_trap(error: wasmtime::Error) -> WorldForgeError {
    let message = error.to_string();
    let code = if message.contains("fuel") {
        ErrorCode::ModFuelExhausted
    } else if message.contains("WF2004") || message.contains("not granted") {
        ErrorCode::ModCapabilityDenied
    } else {
        ErrorCode::ModTrap
    };
    WorldForgeError::new(code, message)
}

/// Load a sandboxed WASM mod from a file using a deny-by-default policy.
pub fn load_wasm_mod(
    path: &std::path::Path,
    policy: CapabilityPolicy,
    config: ModRuntimeConfig,
) -> Result<WasmModInstance, WorldForgeError> {
    let bytes = std::fs::read(path)
        .map_err(|e| WorldForgeError::new(ErrorCode::ModLoadFailed, e.to_string()))?;
    WasmModInstance::from_bytes(&bytes, policy, config, 0)
}

/// A mock mod for testing the mod lifecycle without WASM.
#[cfg(test)]
pub struct MockMod {
    pub name: String,
    pub tick_count: u64,
    pub events_received: Vec<Vec<u8>>,
}

#[cfg(test)]
impl MockMod {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            tick_count: 0,
            events_received: Vec::new(),
        }
    }
}

#[cfg(test)]
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
#[cfg(test)]
pub struct DeniedCapabilityMod;

#[cfg(test)]
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
        assert_eq!(result.unwrap_err().code, ErrorCode::ModCapabilityDenied);
    }

    #[test]
    fn wasm_mod_loading_reports_missing_file() {
        let result = load_wasm_mod(
            std::path::Path::new("nonexistent.wasm"),
            CapabilityPolicy::deny_all(),
            ModRuntimeConfig::default(),
        );
        assert!(result.is_err());
    }

    const ALLOWED_MOD: &str = r#"
        (module
          (import "worldforge" "read_resource" (func $read (result i64)))
          (import "worldforge" "emit_event" (func $emit (param i64)))
          (func (export "init"))
          (func (export "on_tick") (param i64)
            call $read
            call $emit)
          (func (export "on_event") (param i64)))
    "#;

    #[test]
    fn real_wasm_reads_resource_and_emits_event() {
        let policy = CapabilityPolicy::with_capabilities(vec![
            Capability::ResourceRead,
            Capability::EventEmit,
        ]);
        let mut instance = WasmModInstance::from_bytes(
            ALLOWED_MOD.as_bytes(),
            policy,
            ModRuntimeConfig::default(),
            73,
        )
        .unwrap();
        instance.init().unwrap();
        instance.on_tick(1).unwrap();
        assert_eq!(instance.emitted_events(), &[73]);
    }

    #[test]
    fn denied_capability_traps_closed() {
        let mut instance = WasmModInstance::from_bytes(
            ALLOWED_MOD.as_bytes(),
            CapabilityPolicy::with_capabilities(vec![Capability::EventEmit]),
            ModRuntimeConfig::default(),
            73,
        )
        .unwrap();
        instance.init().unwrap();
        let error = instance.on_tick(1).unwrap_err();
        assert_eq!(error.code, ErrorCode::ModCapabilityDenied);
        assert!(instance.emitted_events().is_empty());
    }

    #[test]
    fn network_import_is_not_linked() {
        let network_mod = br#"(module
          (import "wasi:sockets/network" "connect" (func $connect))
          (func (export "init"))
          (func (export "on_tick") (param i64))
          (func (export "on_event") (param i64)))"#;
        let error = WasmModInstance::from_bytes(
            network_mod,
            CapabilityPolicy::deny_all(),
            ModRuntimeConfig::default(),
            0,
        )
        .err()
        .expect("network import must be denied");
        assert_eq!(error.code, ErrorCode::ModCapabilityDenied);
    }
}

//! Core world model types.

use serde::{Deserialize, Serialize};
use worldforge_core::{Fixed64, RegionId, ResourceId};
use worldforge_ecs::Component;

/// A named resource type in the simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDef {
    pub id: ResourceId,
    pub name: String,
    /// Unit of measurement (e.g., "tons", "units", "kWh").
    pub unit: String,
    /// Whether this resource obeys conservation (total cannot change except via production/consumption).
    pub conserved: bool,
}

/// A region in the world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    pub id: RegionId,
    pub name: String,
}

impl Component for Region {
    fn to_fingerprint_bytes(&self) -> Vec<u8> {
        worldforge_core::serial::to_cbor(self).unwrap_or_default()
    }
}

/// A production rule: transforms inputs into outputs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionRule {
    pub name: String,
    /// Resources consumed per tick.
    pub inputs: Vec<(String, Fixed64)>,
    /// Resources produced per tick.
    pub outputs: Vec<(String, Fixed64)>,
    /// Capacity multiplier (1.0 = full capacity).
    pub capacity: Fixed64,
    /// Normalized energy-cost metadata. Physical energy or power consumption
    /// must also be declared explicitly in `inputs` so resource accounting
    /// remains unambiguous across worlds.
    pub energy_cost: Fixed64,
}

impl Component for ProductionRule {
    fn to_fingerprint_bytes(&self) -> Vec<u8> {
        worldforge_core::serial::to_cbor(self).unwrap_or_default()
    }
}

/// Inventory: resource quantities held by an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    /// Resource name → quantity (sorted map for determinism).
    pub resources: std::collections::BTreeMap<String, Fixed64>,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            resources: std::collections::BTreeMap::new(),
        }
    }

    pub fn get(&self, resource: &str) -> Fixed64 {
        self.resources
            .get(resource)
            .copied()
            .unwrap_or(Fixed64::ZERO)
    }

    pub fn set(&mut self, resource: &str, amount: Fixed64) {
        self.resources.insert(resource.to_string(), amount);
    }

    pub fn add(&mut self, resource: &str, amount: Fixed64) {
        let current = self.get(resource);
        self.resources
            .insert(resource.to_string(), current + amount);
    }

    /// Try to subtract. Returns Err if would go negative.
    pub fn try_subtract(&mut self, resource: &str, amount: Fixed64) -> Result<(), String> {
        let current = self.get(resource);
        match current.checked_sub_non_negative(amount) {
            Some(new_val) => {
                self.resources.insert(resource.to_string(), new_val);
                Ok(())
            }
            None => Err(format!(
                "insufficient {}: have {}, need {}",
                resource, current, amount
            )),
        }
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Inventory {
    fn to_fingerprint_bytes(&self) -> Vec<u8> {
        worldforge_core::serial::to_cbor(self).unwrap_or_default()
    }
}

/// An entity's name and type in the world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityInfo {
    pub name: String,
    pub entity_type: String,
    pub region: String,
}

impl Component for EntityInfo {
    fn to_fingerprint_bytes(&self) -> Vec<u8> {
        worldforge_core::serial::to_cbor(self).unwrap_or_default()
    }
}

/// Price information for a resource at a location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceSignal {
    pub prices: std::collections::BTreeMap<String, Fixed64>,
}

impl PriceSignal {
    pub fn new() -> Self {
        Self {
            prices: std::collections::BTreeMap::new(),
        }
    }

    pub fn get(&self, resource: &str) -> Fixed64 {
        self.prices.get(resource).copied().unwrap_or(Fixed64::ONE)
    }

    pub fn set(&mut self, resource: &str, price: Fixed64) {
        self.prices.insert(resource.to_string(), price);
    }
}

impl Default for PriceSignal {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for PriceSignal {
    fn to_fingerprint_bytes(&self) -> Vec<u8> {
        worldforge_core::serial::to_cbor(self).unwrap_or_default()
    }
}

/// Supply/demand transfer link between entities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplyLink {
    pub resource: String,
    pub from_entity: String,
    pub to_entity: String,
    pub max_per_tick: Fixed64,
}

impl Component for SupplyLink {
    fn to_fingerprint_bytes(&self) -> Vec<u8> {
        worldforge_core::serial::to_cbor(self).unwrap_or_default()
    }
}

/// Simulation configuration loaded from the world manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub name: String,
    pub description: String,
    pub version: String,
    pub duration_ticks: u64,
    pub base_price: Fixed64,
    pub price_sensitivity: Fixed64,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            name: "unnamed".to_string(),
            description: String::new(),
            version: "0.1.0".to_string(),
            duration_ticks: 1000,
            base_price: Fixed64::from_int(100),
            price_sensitivity: Fixed64::from_ratio(1, 10),
        }
    }
}

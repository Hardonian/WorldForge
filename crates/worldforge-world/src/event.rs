//! Typed simulation events.

use serde::{Deserialize, Serialize};
use worldforge_core::{Fixed64, Tick};

/// Types of events that occur during simulation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventType {
    ProductionCompleted {
        entity: String,
        resource: String,
        amount: Fixed64,
    },
    ResourceTransferred {
        from: String,
        to: String,
        resource: String,
        amount: Fixed64,
    },
    InventoryShortage {
        entity: String,
        resource: String,
        needed: Fixed64,
        available: Fixed64,
    },
    PriceChanged {
        resource: String,
        old_price: Fixed64,
        new_price: Fixed64,
    },
    CapacityChanged {
        entity: String,
        old_capacity: Fixed64,
        new_capacity: Fixed64,
    },
    ObjectiveUpdated {
        objective: String,
        status: String,
    },
    SimulationDegraded {
        reason: String,
    },
}

/// A simulation event with its tick and type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationEvent {
    pub tick: Tick,
    pub event_type: EventType,
}

impl SimulationEvent {
    pub fn new(tick: Tick, event_type: EventType) -> Self {
        Self { tick, event_type }
    }

    /// Get a human-readable summary of this event.
    pub fn summary(&self) -> String {
        match &self.event_type {
            EventType::ProductionCompleted {
                entity,
                resource,
                amount,
            } => {
                format!("{}: produced {} {}", entity, amount, resource)
            }
            EventType::ResourceTransferred {
                from,
                to,
                resource,
                amount,
            } => {
                format!("{} → {}: {} {}", from, to, amount, resource)
            }
            EventType::InventoryShortage {
                entity,
                resource,
                needed,
                available,
            } => {
                format!(
                    "SHORTAGE at {}: {} needs {}, has {}",
                    entity, resource, needed, available
                )
            }
            EventType::PriceChanged {
                resource,
                old_price,
                new_price,
            } => {
                format!("{} price: {} → {}", resource, old_price, new_price)
            }
            EventType::CapacityChanged {
                entity,
                old_capacity,
                new_capacity,
            } => {
                format!("{} capacity: {} → {}", entity, old_capacity, new_capacity)
            }
            EventType::ObjectiveUpdated { objective, status } => {
                format!("objective '{}': {}", objective, status)
            }
            EventType::SimulationDegraded { reason } => {
                format!("DEGRADED: {}", reason)
            }
        }
    }
}

/// A pre-scheduled event that fires at a specific tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledEvent {
    pub tick: u64,
    pub event_type: ScheduledEventType,
}

/// Types of events that can be scheduled in a scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ScheduledEventType {
    #[serde(rename = "capacity_change")]
    CapacityChange { target: String, value: f64 },
}

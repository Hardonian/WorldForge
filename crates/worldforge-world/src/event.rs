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
    PlayerCapacityChanged {
        entity: String,
        old_capacity: Fixed64,
        new_capacity: Fixed64,
    },
    BuildingConstructed {
        building: String,
        district: String,
        count: u32,
    },
    TechnologyUnlocked {
        technology: String,
        branch: String,
    },
    PopulationChanged {
        amount: Fixed64,
        population: Fixed64,
        housing: u64,
    },
    CivicDilemmaOpened {
        dilemma: String,
        deadline_tick: Option<u64>,
    },
    PlayerCivicDecision {
        dilemma: String,
        option: String,
    },
    CivicDecisionResolved {
        dilemma: String,
        option: String,
    },
    ObjectiveUpdated {
        objective: String,
        status: String,
    },
    SimulationDegraded {
        reason: String,
    },
    GeopoliticalStanceChanged {
        entity: String,
        from: String,
        to: String,
    },
    TributeCollected {
        entity: String,
        resources: std::collections::BTreeMap<String, f64>,
    },
    WarlordIncursion {
        entity: String,
        damage: f64,
        repelled: bool,
    },
    RefugeeWaveArrived {
        origin: String,
        count: f64,
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
            EventType::PlayerCapacityChanged {
                entity,
                old_capacity,
                new_capacity,
            } => format!(
                "{} capacity: {} → {} (player decision)",
                entity, old_capacity, new_capacity
            ),
            EventType::BuildingConstructed {
                building,
                district,
                count,
            } => format!("built {building} in {district} (district count: {count})"),
            EventType::TechnologyUnlocked { technology, branch } => {
                format!("researched {technology} in the {branch} branch")
            }
            EventType::PopulationChanged {
                amount,
                population,
                housing,
            } => format!("population grew by {amount} to {population} ({housing} housing)"),
            EventType::CivicDilemmaOpened {
                dilemma,
                deadline_tick,
            } => deadline_tick.map_or_else(
                || format!("civic dilemma opened: {dilemma}"),
                |deadline| format!("civic dilemma opened: {dilemma} (deadline t{deadline})"),
            ),
            EventType::PlayerCivicDecision { dilemma, option } => {
                format!("civic decision: {dilemma} → {option}")
            }
            EventType::CivicDecisionResolved { dilemma, option } => {
                format!("civic deadline resolved: {dilemma} → {option}")
            }
            EventType::ObjectiveUpdated { objective, status } => {
                format!("objective '{}': {}", objective, status)
            }
            EventType::SimulationDegraded { reason } => {
                format!("DEGRADED: {}", reason)
            }
            EventType::GeopoliticalStanceChanged { entity, from, to } => {
                format!("geopolitical stance of '{entity}' changed: {from} → {to}")
            }
            EventType::TributeCollected { entity, resources } => {
                let details = resources
                    .iter()
                    .map(|(k, v)| format!("{v} {k}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("collected tribute from '{entity}': {details}")
            }
            EventType::WarlordIncursion {
                entity,
                damage,
                repelled,
            } => {
                if *repelled {
                    format!("warlord incursion by '{entity}' repelled by defense forces")
                } else {
                    format!("warlord incursion by '{entity}' breached perimeter ({damage} damage)")
                }
            }
            EventType::RefugeeWaveArrived { origin, count } => {
                format!("{count} refugees arrived from '{origin}' seeking sanctuary")
            }
        }
    }
}

/// A pre-scheduled event that fires at a specific tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledEvent {
    pub tick: u64,
    #[serde(flatten)]
    pub event_type: ScheduledEventType,
}

/// Types of events that can be scheduled in a scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ScheduledEventType {
    #[serde(rename = "capacity_change")]
    CapacityChange { target: String, value: f64 },
}

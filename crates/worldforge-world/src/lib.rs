//! # worldforge-world
//!
//! The canonical world model for World Forge simulations.
//!
//! Defines the data structures for worlds, regions, entities, resources,
//! production rules, objectives, events, and scenarios. Also handles
//! manifest parsing and scenario validation.

pub mod config;
pub mod event;
pub mod manifest;
pub mod model;
pub mod objective;
pub mod scenario;

pub use config::{EntitiesConfig, EntityConfig, LinkConfig, ProductionConfig};
pub use event::{EventType, ScheduledEvent, ScheduledEventType, SimulationEvent};
pub use manifest::WorldManifest;
pub use model::*;
pub use objective::{Objective, ObjectiveStatus, ObjectiveType};
pub use scenario::Scenario;

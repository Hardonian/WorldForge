//! # worldforge-world
//!
//! The canonical world model for World Forge simulations.
//!
//! Defines the data structures for worlds, regions, entities, resources,
//! production rules, objectives, events, and scenarios. Also handles
//! manifest parsing and scenario validation.

pub mod event;
pub mod manifest;
pub mod model;
pub mod objective;
pub mod scenario;

pub use event::{SimulationEvent, ScheduledEvent, EventType};
pub use manifest::WorldManifest;
pub use model::*;
pub use objective::{Objective, ObjectiveStatus, ObjectiveType};
pub use scenario::Scenario;

//! # worldforge-ecs
//!
//! A purpose-built, deterministic Entity-Component-System for World Forge.
//!
//! Unlike general-purpose ECS frameworks that optimize for parallelism,
//! this ECS prioritizes **determinism**: given the same initial state and
//! the same sequence of operations, the world state will be identical.
//!
//! Key design choices:
//! - Single-threaded execution with explicit system ordering
//! - Components stored in sorted BTreeMaps for deterministic iteration
//! - Fingerprinting support for state verification
//!
//! This is NOT a general-purpose game ECS. It is specifically designed
//! for headless, deterministic simulation.

pub mod component;
pub mod schedule;
pub mod world;

pub use component::{AnyComponent, Component, ComponentStorage};
pub use schedule::{SimulationSchedule, SystemFn, SystemId};
pub use world::SimulationWorld;

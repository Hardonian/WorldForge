//! The simulation world — the central ECS data store.
//!
//! All simulation state lives here. Components are stored per-type
//! in deterministic-order maps.

use std::any::TypeId;
use std::collections::BTreeMap;

use worldforge_core::hash::FingerprintBuilder;
use worldforge_core::{EntityId, Fingerprint, Tick};

use crate::component::{AnyComponent, Component, ComponentStorage};

/// The central simulation world containing all entities and components.
#[derive(Clone)]
pub struct SimulationWorld {
    /// Current simulation tick.
    current_tick: Tick,
    /// Component storages keyed by type name (sorted for determinism).
    storages: BTreeMap<String, ComponentStorage>,
    /// TypeId → type name mapping.
    type_names: BTreeMap<TypeId, String>,
    /// All known entity IDs (sorted for deterministic iteration).
    entities: std::collections::BTreeSet<[u8; 16]>,
    /// Arbitrary per-tick metadata (for systems to communicate).
    tick_data: BTreeMap<String, Vec<u8>>,
}

impl SimulationWorld {
    pub fn new() -> Self {
        Self {
            current_tick: Tick::ZERO,
            storages: BTreeMap::new(),
            type_names: BTreeMap::new(),
            entities: std::collections::BTreeSet::new(),
            tick_data: BTreeMap::new(),
        }
    }

    /// Get the current tick.
    pub fn current_tick(&self) -> Tick {
        self.current_tick
    }

    /// Advance to the next tick.
    pub fn advance_tick(&mut self) {
        self.current_tick = self.current_tick.next();
    }

    /// Set the current tick (used during initialization).
    pub fn set_tick(&mut self, tick: Tick) {
        self.current_tick = tick;
    }

    /// Spawn a new entity with no components.
    pub fn spawn(&mut self, entity: EntityId) {
        self.entities.insert(*entity.as_bytes());
    }

    /// Check if an entity exists.
    pub fn has_entity(&self, entity: &EntityId) -> bool {
        self.entities.contains(entity.as_bytes())
    }

    /// Get all entity IDs.
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Insert a component for an entity.
    pub fn insert_component<T: Component + 'static>(&mut self, entity: EntityId, component: T) {
        let type_name = std::any::type_name::<T>();
        self.entities.insert(*entity.as_bytes());

        let type_id = TypeId::of::<T>();
        self.type_names
            .entry(type_id)
            .or_insert_with(|| type_name.to_string());

        let storage = self
            .storages
            .entry(type_name.to_string())
            .or_insert_with(|| ComponentStorage::new(type_name));
        storage.insert(entity, AnyComponent::new(component));
    }

    /// Get a component reference for an entity.
    pub fn get_component<T: Component + 'static>(&self, entity: &EntityId) -> Option<&T> {
        let type_name = std::any::type_name::<T>();
        self.storages
            .get(type_name)
            .and_then(|s| s.get(entity))
            .and_then(|c| c.downcast_ref::<T>())
    }

    /// Get a mutable component reference for an entity.
    pub fn get_component_mut<T: Component + 'static>(
        &mut self,
        entity: &EntityId,
    ) -> Option<&mut T> {
        let type_name = std::any::type_name::<T>();
        self.storages
            .get_mut(type_name)
            .and_then(|s| s.get_mut(entity))
            .and_then(|c| c.downcast_mut::<T>())
    }

    /// Get the storage for a specific component type.
    pub fn storage<T: Component + 'static>(&self) -> Option<&ComponentStorage> {
        let type_name = std::any::type_name::<T>();
        self.storages.get(type_name)
    }

    /// Get mutable storage for a specific component type.
    pub fn storage_mut<T: Component + 'static>(&mut self) -> Option<&mut ComponentStorage> {
        let type_name = std::any::type_name::<T>();
        self.storages.get_mut(type_name)
    }

    /// Get storage by type name string (for dynamic lookups from mods).
    pub fn storage_by_name(&self, type_name: &str) -> Option<&ComponentStorage> {
        self.storages.get(type_name)
    }

    /// Store arbitrary tick data (cleared each tick by convention).
    pub fn set_tick_data(&mut self, key: &str, data: Vec<u8>) {
        self.tick_data.insert(key.to_string(), data);
    }

    /// Get tick data.
    pub fn get_tick_data(&self, key: &str) -> Option<&[u8]> {
        self.tick_data.get(key).map(|v| v.as_slice())
    }

    /// Clear tick data (called between ticks).
    pub fn clear_tick_data(&mut self) {
        self.tick_data.clear();
    }

    /// Compute a fingerprint of the entire world state.
    /// This is deterministic: same state → same fingerprint.
    pub fn fingerprint(&self) -> Fingerprint {
        let mut builder = FingerprintBuilder::new();

        // Include tick
        builder.update(&self.current_tick.value().to_le_bytes());

        // Include all storages in sorted order (BTreeMap guarantees this)
        for (type_name, storage) in &self.storages {
            builder.update(type_name.as_bytes());
            builder.update_fingerprint(&storage.fingerprint());
        }

        builder.finalize()
    }
}

impl Default for SimulationWorld {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for SimulationWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimulationWorld")
            .field("tick", &self.current_tick)
            .field("entities", &self.entities.len())
            .field("component_types", &self.storages.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct Health(i32);

    impl Component for Health {
        fn to_fingerprint_bytes(&self) -> Vec<u8> {
            worldforge_core::serial::to_cbor(&self.0).unwrap_or_default()
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct Velocity {
        dx: i32,
        dy: i32,
    }

    impl Component for Velocity {
        fn to_fingerprint_bytes(&self) -> Vec<u8> {
            worldforge_core::serial::to_cbor(self).unwrap_or_default()
        }
    }

    #[test]
    fn world_insert_and_get() {
        let mut world = SimulationWorld::new();
        let entity = EntityId::deterministic(1, 0);
        world.insert_component(entity, Health(100));
        let health = world.get_component::<Health>(&entity).unwrap();
        assert_eq!(health.0, 100);
    }

    #[test]
    fn world_fingerprint_deterministic() {
        let build_world = || {
            let mut world = SimulationWorld::new();
            for i in 0..10 {
                let e = EntityId::deterministic(42, i);
                world.insert_component(e, Health(100 - i as i32));
                world.insert_component(
                    e,
                    Velocity {
                        dx: i as i32,
                        dy: 0,
                    },
                );
            }
            world.set_tick(Tick::new(50));
            world
        };

        let w1 = build_world();
        let w2 = build_world();
        assert_eq!(w1.fingerprint(), w2.fingerprint());
    }

    #[test]
    fn world_fingerprint_changes_with_state() {
        let mut world = SimulationWorld::new();
        let entity = EntityId::deterministic(1, 0);
        world.insert_component(entity, Health(100));
        let fp1 = world.fingerprint();

        world.get_component_mut::<Health>(&entity).unwrap().0 = 50;
        let fp2 = world.fingerprint();

        assert_ne!(fp1, fp2);
    }

    #[test]
    fn world_tick_advancement() {
        let mut world = SimulationWorld::new();
        assert_eq!(world.current_tick(), Tick::ZERO);
        world.advance_tick();
        assert_eq!(world.current_tick(), Tick::new(1));
    }
}

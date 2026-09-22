//! Component storage with deterministic iteration order.

use serde::{Deserialize, Serialize};
use std::any::{Any, TypeId};
use std::collections::BTreeMap;
use std::fmt;

use worldforge_core::EntityId;

/// Trait for components that can be stored in the ECS.
pub trait Component: Any + Send + Sync + fmt::Debug + ComponentClone {
    /// Serialize this component to CBOR bytes for fingerprinting.
    fn to_fingerprint_bytes(&self) -> Vec<u8>;
}

/// Helper trait for cloning boxed components.
pub trait ComponentClone {
    fn clone_box(&self) -> Box<dyn Component>;
}

impl<T: Component + Clone + 'static> ComponentClone for T {
    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }
}

/// Type-erased component.
pub struct AnyComponent {
    inner: Box<dyn Component>,
    type_id: TypeId,
    type_name: &'static str,
}

impl AnyComponent {
    pub fn new<T: Component + 'static>(value: T) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: std::any::type_name::<T>(),
            inner: Box::new(value),
        }
    }

    pub fn downcast_ref<T: Component + 'static>(&self) -> Option<&T> {
        if self.type_id == TypeId::of::<T>() {
            // SAFETY: We verified the TypeId matches
            Some(unsafe { &*(self.inner.as_ref() as *const dyn Component as *const T) })
        } else {
            None
        }
    }

    pub fn downcast_mut<T: Component + 'static>(&mut self) -> Option<&mut T> {
        if self.type_id == TypeId::of::<T>() {
            // SAFETY: We verified the TypeId matches
            Some(unsafe { &mut *(self.inner.as_mut() as *mut dyn Component as *mut T) })
        } else {
            None
        }
    }

    pub fn type_name(&self) -> &'static str {
        self.type_name
    }

    pub fn fingerprint_bytes(&self) -> Vec<u8> {
        self.inner.to_fingerprint_bytes()
    }
}

impl Clone for AnyComponent {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone_box(),
            type_id: self.type_id,
            type_name: self.type_name,
        }
    }
}

impl fmt::Debug for AnyComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AnyComponent({})", self.type_name)
    }
}

/// Storage for a single component type, keyed by EntityId.
/// Uses BTreeMap for deterministic iteration order.
#[derive(Debug, Clone)]
pub struct ComponentStorage {
    /// Component type name for diagnostics.
    type_name: &'static str,
    /// Components stored by entity, in deterministic order.
    entries: BTreeMap<[u8; 16], AnyComponent>,
}

impl ComponentStorage {
    pub fn new(type_name: &'static str) -> Self {
        Self {
            type_name,
            entries: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, entity: EntityId, component: AnyComponent) {
        self.entries.insert(*entity.as_bytes(), component);
    }

    pub fn get(&self, entity: &EntityId) -> Option<&AnyComponent> {
        self.entries.get(entity.as_bytes())
    }

    pub fn get_mut(&mut self, entity: &EntityId) -> Option<&mut AnyComponent> {
        self.entries.get_mut(entity.as_bytes())
    }

    pub fn remove(&mut self, entity: &EntityId) -> Option<AnyComponent> {
        self.entries.remove(entity.as_bytes())
    }

    pub fn contains(&self, entity: &EntityId) -> bool {
        self.entries.contains_key(entity.as_bytes())
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn type_name(&self) -> &'static str {
        self.type_name
    }

    /// Iterate over all components in deterministic order.
    pub fn iter(&self) -> impl Iterator<Item = (&[u8; 16], &AnyComponent)> {
        self.entries.iter()
    }

    /// Iterate mutably over all components in deterministic order.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&[u8; 16], &mut AnyComponent)> {
        self.entries.iter_mut()
    }

    /// Compute a fingerprint of all components in this storage.
    pub fn fingerprint(&self) -> worldforge_core::Fingerprint {
        let mut builder = worldforge_core::hash::FingerprintBuilder::new();
        builder.update(self.type_name.as_bytes());
        for (key, component) in &self.entries {
            builder.update(key);
            builder.update(&component.fingerprint_bytes());
        }
        builder.finalize()
    }
}

/// Implement Component for common serializable types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameComponent(pub String);

impl Component for NameComponent {
    fn to_fingerprint_bytes(&self) -> Vec<u8> {
        worldforge_core::serial::to_cbor(&self.0).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct Position {
        x: i32,
        y: i32,
    }

    impl Component for Position {
        fn to_fingerprint_bytes(&self) -> Vec<u8> {
            worldforge_core::serial::to_cbor(self).unwrap_or_default()
        }
    }

    #[test]
    fn storage_insert_and_get() {
        let mut storage = ComponentStorage::new("Position");
        let entity = EntityId::deterministic(1, 0);
        storage.insert(entity, AnyComponent::new(Position { x: 10, y: 20 }));
        let comp = storage.get(&entity).unwrap();
        let pos = comp.downcast_ref::<Position>().unwrap();
        assert_eq!(pos.x, 10);
        assert_eq!(pos.y, 20);
    }

    #[test]
    fn storage_deterministic_iteration() {
        let mut storage = ComponentStorage::new("Position");
        // Insert in arbitrary order
        for i in (0..10).rev() {
            let entity = EntityId::deterministic(42, i);
            storage.insert(entity, AnyComponent::new(Position { x: i as i32, y: 0 }));
        }

        // Iteration should be in sorted key order
        let keys: Vec<_> = storage.iter().map(|(k, _)| *k).collect();
        let mut sorted = keys.clone();
        sorted.sort();
        assert_eq!(keys, sorted);
    }

    #[test]
    fn storage_fingerprint_deterministic() {
        let mut s1 = ComponentStorage::new("Position");
        let mut s2 = ComponentStorage::new("Position");

        for i in 0..5 {
            let entity = EntityId::deterministic(42, i);
            s1.insert(entity, AnyComponent::new(Position { x: i as i32, y: i as i32 }));
            s2.insert(entity, AnyComponent::new(Position { x: i as i32, y: i as i32 }));
        }

        assert_eq!(s1.fingerprint(), s2.fingerprint());
    }
}

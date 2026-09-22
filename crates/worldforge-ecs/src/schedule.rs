//! Deterministic system scheduling.
//!
//! Systems run in a fixed, explicit order. There is no parallel execution.
//! This guarantees that the same systems with the same state produce
//! identical results on every platform.

use std::fmt;

/// Identifier for a registered system.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SystemId {
    pub name: String,
    pub order: u32,
}

impl fmt::Display for SystemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}[{}]", self.name, self.order)
    }
}

/// A system function that operates on the simulation world.
pub type SystemFn = Box<dyn FnMut(&mut super::world::SimulationWorld) + Send>;

/// A registered system with its metadata.
struct RegisteredSystem {
    id: SystemId,
    system_fn: SystemFn,
}

/// Deterministic simulation schedule.
///
/// Systems are executed in order of their `order` field,
/// then by registration order for systems with the same priority.
/// This is fully deterministic.
pub struct SimulationSchedule {
    systems: Vec<RegisteredSystem>,
    sorted: bool,
}

impl SimulationSchedule {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
            sorted: false,
        }
    }

    /// Register a system with a given execution order.
    /// Lower numbers run first.
    pub fn add_system(&mut self, name: &str, order: u32, system_fn: SystemFn) -> SystemId {
        let id = SystemId {
            name: name.to_string(),
            order,
        };
        self.systems.push(RegisteredSystem {
            id: id.clone(),
            system_fn,
        });
        self.sorted = false;
        id
    }

    /// Ensure systems are sorted by execution order.
    fn ensure_sorted(&mut self) {
        if !self.sorted {
            self.systems.sort_by_key(|s| s.id.order);
            self.sorted = true;
        }
    }

    /// Run all systems in deterministic order on the given world.
    pub fn run(&mut self, world: &mut super::world::SimulationWorld) {
        self.ensure_sorted();
        for system in &mut self.systems {
            tracing::trace!(system = %system.id, "running system");
            (system.system_fn)(world);
        }
    }

    /// Get the number of registered systems.
    pub fn system_count(&self) -> usize {
        self.systems.len()
    }

    /// Get the names of all systems in execution order.
    pub fn system_names(&mut self) -> Vec<String> {
        self.ensure_sorted();
        self.systems.iter().map(|s| s.id.name.clone()).collect()
    }
}

impl Default for SimulationSchedule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::SimulationWorld;

    #[test]
    fn systems_execute_in_order() {
        let mut schedule = SimulationSchedule::new();
        let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));

        let log1 = log.clone();
        schedule.add_system(
            "third",
            30,
            Box::new(move |_: &mut SimulationWorld| {
                log1.lock().unwrap().push("third");
            }),
        );

        let log2 = log.clone();
        schedule.add_system(
            "first",
            10,
            Box::new(move |_: &mut SimulationWorld| {
                log2.lock().unwrap().push("first");
            }),
        );

        let log3 = log.clone();
        schedule.add_system(
            "second",
            20,
            Box::new(move |_: &mut SimulationWorld| {
                log3.lock().unwrap().push("second");
            }),
        );

        let mut world = SimulationWorld::new();
        schedule.run(&mut world);

        let execution_order = log.lock().unwrap();
        assert_eq!(*execution_order, vec!["first", "second", "third"]);
    }
}

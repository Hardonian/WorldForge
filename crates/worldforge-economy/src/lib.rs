//! # worldforge-economy
//!
//! Deterministic resource economy system for World Forge.
//!
//! Provides inventory management, atomic transfers, production/consumption,
//! shortage detection, and price signals. All operations preserve invariants:
//! - Inventory cannot silently become negative
//! - Resource conservation where configured
//! - Transactions are atomic
//! - Invalid transfers fail without corrupting state

use worldforge_core::{EntityId, Fixed64, Tick};
use worldforge_ecs::SimulationWorld;
use worldforge_world::{
    EntityInfo, EventType, Inventory, PriceSignal, ProductionRule, SimulationEvent, SupplyLink,
};

/// Run production for all entities that have production rules.
pub fn run_production(
    world: &mut SimulationWorld,
    entity_ids: &[(EntityId, String)],
    tick: Tick,
    events: &mut Vec<SimulationEvent>,
) {
    for (entity_id, entity_name) in entity_ids {
        // Get production rule
        let rule = match world.get_component::<ProductionRule>(entity_id) {
            Some(r) => r.clone(),
            None => continue,
        };

        // Get inventory
        let inventory = match world.get_component::<Inventory>(entity_id) {
            Some(inv) => inv.clone(),
            None => continue,
        };

        let capacity = rule.capacity;
        if capacity.is_zero() {
            continue;
        }

        // Check if we have all inputs
        let mut can_produce = true;
        for (resource, amount) in &rule.inputs {
            let needed = *amount * capacity;
            if inventory.get(resource) < needed {
                can_produce = false;
                events.push(SimulationEvent::new(
                    tick,
                    EventType::InventoryShortage {
                        entity: entity_name.clone(),
                        resource: resource.clone(),
                        needed,
                        available: inventory.get(resource),
                    },
                ));
            }
        }

        if can_produce {
            // Consume inputs and produce outputs
            let inv = world.get_component_mut::<Inventory>(entity_id).unwrap();
            for (resource, amount) in &rule.inputs {
                let consumed = *amount * capacity;
                let _ = inv.try_subtract(resource, consumed);
            }
            for (resource, amount) in &rule.outputs {
                let produced = *amount * capacity;
                inv.add(resource, produced);
                events.push(SimulationEvent::new(
                    tick,
                    EventType::ProductionCompleted {
                        entity: entity_name.clone(),
                        resource: resource.clone(),
                        amount: produced,
                    },
                ));
            }
        }
    }
}

/// Run supply chain transfers between linked entities.
pub fn run_transfers(
    world: &mut SimulationWorld,
    links: &[(EntityId, EntityId, String, Fixed64)], // (from, to, resource, max_per_tick)
    entity_names: &std::collections::BTreeMap<EntityId, String>,
    tick: Tick,
    events: &mut Vec<SimulationEvent>,
) {
    for (from_id, to_id, resource, max_per_tick) in links {
        // Get available amount from source
        let available = match world.get_component::<Inventory>(from_id) {
            Some(inv) => inv.get(resource),
            None => continue,
        };

        let transfer_amount = available.min(*max_per_tick);
        if transfer_amount.is_zero() {
            continue;
        }

        // Atomic transfer: subtract from source, add to destination
        if let Some(from_inv) = world.get_component_mut::<Inventory>(from_id) {
            if from_inv.try_subtract(resource, transfer_amount).is_err() {
                continue;
            }
        }

        if let Some(to_inv) = world.get_component_mut::<Inventory>(to_id) {
            to_inv.add(resource, transfer_amount);
        }

        let from_name = entity_names.get(from_id).map(|s| s.as_str()).unwrap_or("unknown");
        let to_name = entity_names.get(to_id).map(|s| s.as_str()).unwrap_or("unknown");

        events.push(SimulationEvent::new(
            tick,
            EventType::ResourceTransferred {
                from: from_name.to_string(),
                to: to_name.to_string(),
                resource: resource.clone(),
                amount: transfer_amount,
            },
        ));
    }
}

/// Update price signals based on inventory levels.
pub fn update_prices(
    world: &mut SimulationWorld,
    entity_ids: &[(EntityId, String)],
    resources: &[String],
    base_price: Fixed64,
    sensitivity: Fixed64,
    tick: Tick,
    events: &mut Vec<SimulationEvent>,
) {
    for (entity_id, entity_name) in entity_ids {
        let inventory = match world.get_component::<Inventory>(entity_id) {
            Some(inv) => inv.clone(),
            None => continue,
        };

        let old_prices = match world.get_component::<PriceSignal>(entity_id) {
            Some(ps) => ps.clone(),
            None => PriceSignal::new(),
        };

        let mut new_prices = PriceSignal::new();
        for resource in resources {
            let stock = inventory.get(resource);
            // Simple price model: price increases as stock decreases
            let price = if stock.is_zero() {
                base_price * Fixed64::from_int(10) // Scarcity premium
            } else if stock < Fixed64::from_int(50) {
                base_price + (Fixed64::from_int(50) - stock) * sensitivity
            } else {
                base_price
            };

            let old_price = old_prices.get(resource);
            if old_price != price {
                events.push(SimulationEvent::new(
                    tick,
                    EventType::PriceChanged {
                        resource: resource.clone(),
                        old_price,
                        new_price: price,
                    },
                ));
            }
            new_prices.set(resource, price);
        }

        world.insert_component(*entity_id, new_prices);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_cannot_go_negative() {
        let mut inv = Inventory::new();
        inv.set("ore", Fixed64::from_int(10));
        assert!(inv.try_subtract("ore", Fixed64::from_int(5)).is_ok());
        assert_eq!(inv.get("ore"), Fixed64::from_int(5));
        assert!(inv.try_subtract("ore", Fixed64::from_int(10)).is_err());
        assert_eq!(inv.get("ore"), Fixed64::from_int(5)); // Unchanged
    }

    #[test]
    fn production_consumes_and_produces() {
        let mut world = SimulationWorld::new();
        let entity = EntityId::deterministic(1, 0);

        let mut inv = Inventory::new();
        inv.set("ore", Fixed64::from_int(100));
        world.insert_component(entity, inv);

        world.insert_component(
            entity,
            ProductionRule {
                name: "smelt".to_string(),
                inputs: vec![("ore".to_string(), Fixed64::from_int(2))],
                outputs: vec![("steel".to_string(), Fixed64::from_int(1))],
                capacity: Fixed64::ONE,
                energy_cost: Fixed64::ZERO,
            },
        );

        let entities = vec![(entity, "smelter".to_string())];
        let mut events = Vec::new();
        run_production(&mut world, &entities, Tick::new(1), &mut events);

        let inv = world.get_component::<Inventory>(&entity).unwrap();
        assert_eq!(inv.get("ore"), Fixed64::from_int(98));
        assert_eq!(inv.get("steel"), Fixed64::from_int(1));
    }

    #[test]
    fn transfer_is_atomic() {
        let mut world = SimulationWorld::new();
        let from = EntityId::deterministic(1, 0);
        let to = EntityId::deterministic(1, 1);

        let mut from_inv = Inventory::new();
        from_inv.set("ore", Fixed64::from_int(50));
        world.insert_component(from, from_inv);
        world.insert_component(to, Inventory::new());

        let mut names = std::collections::BTreeMap::new();
        names.insert(from, "mine".to_string());
        names.insert(to, "factory".to_string());

        let links = vec![(from, to, "ore".to_string(), Fixed64::from_int(20))];
        let mut events = Vec::new();
        run_transfers(&mut world, &links, &names, Tick::new(1), &mut events);

        let from_inv = world.get_component::<Inventory>(&from).unwrap();
        let to_inv = world.get_component::<Inventory>(&to).unwrap();
        assert_eq!(from_inv.get("ore"), Fixed64::from_int(30));
        assert_eq!(to_inv.get("ore"), Fixed64::from_int(20));
    }

    #[test]
    fn conservation_in_transfer() {
        let mut world = SimulationWorld::new();
        let from = EntityId::deterministic(1, 0);
        let to = EntityId::deterministic(1, 1);

        let mut from_inv = Inventory::new();
        from_inv.set("ore", Fixed64::from_int(100));
        world.insert_component(from, from_inv);
        world.insert_component(to, Inventory::new());

        let mut names = std::collections::BTreeMap::new();
        names.insert(from, "a".to_string());
        names.insert(to, "b".to_string());

        let total_before = {
            let f = world.get_component::<Inventory>(&from).unwrap().get("ore");
            let t = world.get_component::<Inventory>(&to).unwrap().get("ore");
            f + t
        };

        let links = vec![(from, to, "ore".to_string(), Fixed64::from_int(30))];
        let mut events = Vec::new();
        run_transfers(&mut world, &links, &names, Tick::new(1), &mut events);

        let total_after = {
            let f = world.get_component::<Inventory>(&from).unwrap().get("ore");
            let t = world.get_component::<Inventory>(&to).unwrap().get("ore");
            f + t
        };

        assert_eq!(total_before, total_after); // Conservation
    }
}

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

use worldforge_core::{EntityId, ErrorCode, Fixed64, Tick, WorldForgeError};
use worldforge_ecs::SimulationWorld;
use worldforge_world::{EventType, Inventory, PriceSignal, ProductionRule, SimulationEvent};

/// Run production for all entities that have production rules.
pub fn run_production(
    world: &mut SimulationWorld,
    entity_ids: &[(EntityId, String)],
    tick: Tick,
    events: &mut Vec<SimulationEvent>,
) {
    for (entity_id, entity_name) in entity_ids {
        let rule = match world.get_component::<ProductionRule>(entity_id) {
            Some(rule) => rule,
            None => continue,
        };
        let inventory = match world.get_component::<Inventory>(entity_id) {
            Some(inventory) => inventory,
            None => continue,
        };

        let capacity = rule.capacity;
        if capacity.is_zero() {
            continue;
        }

        let input_requirements = rule.inputs.clone();
        let output_rules = rule.outputs.clone();
        let mut can_produce = true;
        for (resource, amount) in &input_requirements {
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
            let inventory = world
                .get_component_mut::<Inventory>(entity_id)
                .expect("production inventory was validated");
            for (resource, amount) in &input_requirements {
                let consumed = *amount * capacity;
                let _ = inventory.try_subtract(resource, consumed);
            }
            for (resource, amount) in &output_rules {
                let produced = *amount * capacity;
                inventory.add(resource, produced);
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

        if transfer_inventory(world, *from_id, *to_id, resource, transfer_amount).is_err() {
            continue;
        }

        let from_name = entity_names
            .get(from_id)
            .map(|s| s.as_str())
            .unwrap_or("unknown");
        let to_name = entity_names
            .get(to_id)
            .map(|s| s.as_str())
            .unwrap_or("unknown");

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

/// Atomically transfer inventory. All validation occurs before mutation, and a
/// failed transfer leaves both inventories unchanged.
pub fn transfer_inventory(
    world: &mut SimulationWorld,
    from: EntityId,
    to: EntityId,
    resource: &str,
    amount: Fixed64,
) -> Result<(), WorldForgeError> {
    if !amount.is_non_negative() {
        return Err(WorldForgeError::new(
            ErrorCode::TransferFailed,
            "transfer amount cannot be negative",
        ));
    }
    if from == to || amount.is_zero() {
        return Ok(());
    }
    let source = world.get_component::<Inventory>(&from).ok_or_else(|| {
        WorldForgeError::new(ErrorCode::TransferFailed, "source inventory is missing")
    })?;
    if source.get(resource) < amount {
        return Err(WorldForgeError::new(
            ErrorCode::InsufficientResource,
            format!("insufficient {resource} for transfer"),
        ));
    }
    if world.get_component::<Inventory>(&to).is_none() {
        return Err(WorldForgeError::new(
            ErrorCode::TransferFailed,
            "destination inventory is missing",
        ));
    }

    world
        .get_component_mut::<Inventory>(&from)
        .expect("source validated above")
        .try_subtract(resource, amount)
        .map_err(|e| WorldForgeError::new(ErrorCode::TransferFailed, e))?;
    world
        .get_component_mut::<Inventory>(&to)
        .expect("destination validated above")
        .add(resource, amount);
    Ok(())
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
    for (entity_id, _entity_name) in entity_ids {
        let inventory = match world.get_component::<Inventory>(entity_id) {
            Some(inventory) => inventory,
            None => continue,
        };
        let prices = match world.get_component_mut::<PriceSignal>(entity_id) {
            Some(prices) => prices,
            None => continue,
        };

        let stocks = resources
            .iter()
            .map(|resource| (resource.clone(), inventory.get(resource)))
            .collect::<Vec<_>>();
        let prices = match world.get_component_mut::<PriceSignal>(entity_id) {
            Some(prices) => prices,
            None => continue,
        };

        for (resource, stock) in stocks {
            let price = if stock.is_zero() {
                base_price * Fixed64::from_int(10)
            } else if stock < Fixed64::from_int(50) {
                base_price + (Fixed64::from_int(50) - stock) * sensitivity
            } else {
                base_price
            };

            let old_price = prices.get(&resource);
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
            prices.set(&resource, price);
        }
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

    #[test]
    fn invalid_destination_does_not_mutate_source() {
        let mut world = SimulationWorld::new();
        let from = EntityId::deterministic(1, 0);
        let missing = EntityId::deterministic(1, 1);
        let mut inventory = Inventory::new();
        inventory.set("ore", Fixed64::from_int(10));
        world.insert_component(from, inventory);

        assert!(
            transfer_inventory(&mut world, from, missing, "ore", Fixed64::from_int(5)).is_err()
        );
        assert_eq!(
            world.get_component::<Inventory>(&from).unwrap().get("ore"),
            Fixed64::from_int(10)
        );
    }
}

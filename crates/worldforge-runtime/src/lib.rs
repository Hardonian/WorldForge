//! # worldforge-runtime
//!
//! The simulation execution runtime for World Forge.
//!
//! Owns the full lifecycle of a simulation run: loading, validation,
//! initialization, tick execution, event dispatch, replay recording,
//! proof generation, and result reporting.

use std::collections::BTreeMap;
use std::path::Path;

use worldforge_core::error::{ErrorCode, WorldForgeError};
use worldforge_core::hash::Fingerprint;
use worldforge_core::{EntityId, Fixed64, Tick};
use worldforge_economy::{run_production, run_transfers, update_prices};
use worldforge_ecs::SimulationWorld;
use worldforge_proof::RunProof;
use worldforge_replay::{ReplayArtifact, ReplayWriter};
use worldforge_world::*;

/// State of a simulation run.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum RunState {
    Created,
    Validated,
    Ready,
    Running,
    Completed,
    Failed { reason: String },
    Degraded { reason: String },
}

/// Result of a completed simulation run.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RunResult {
    pub state: RunState,
    pub seed: u64,
    pub total_ticks: u64,
    pub initial_state_fingerprint: Fingerprint,
    pub final_state_fingerprint: Fingerprint,
    pub world_fingerprint: Fingerprint,
    pub scenario_fingerprint: Fingerprint,
    pub proof: RunProof,
    pub objective_results: Vec<ObjectiveResult>,
    pub event_count: usize,
    pub shortage_count: usize,
    pub snapshots: Vec<ResourceSnapshot>,
    pub entities: Vec<RunEntity>,
    pub links: Vec<RunLink>,
}

/// Aggregate resource levels after a simulation tick.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ResourceSnapshot {
    pub tick: u64,
    pub levels: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RunEntity {
    pub name: String,
    pub entity_type: String,
    pub region: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RunLink {
    pub from: String,
    pub to: String,
    pub resource: String,
    pub max_per_tick: f64,
}

/// Result of an objective evaluation.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ObjectiveResult {
    pub name: String,
    pub status: ObjectiveStatus,
}

/// The simulation runtime — executes worlds.
pub struct SimulationRuntime {
    world_path: std::path::PathBuf,
    world: SimulationWorld,
    scenario: Scenario,
    state: RunState,
    entity_ids: Vec<(EntityId, String)>,
    entity_id_map: BTreeMap<String, EntityId>,
    entity_name_map: BTreeMap<EntityId, String>,
    supply_links: Vec<(EntityId, EntityId, String, Fixed64)>,
    tracked_resources: Vec<String>,
    events: Vec<SimulationEvent>,
    replay_writer: Option<ReplayWriter>,
    replay: Option<ReplayArtifact>,
    world_fingerprint: Fingerprint,
    scenario_fingerprint: Fingerprint,
    run_entities: Vec<RunEntity>,
    run_links: Vec<RunLink>,
}

impl SimulationRuntime {
    /// Load and initialize a simulation from a world directory.
    pub fn load(
        world_path: &Path,
        seed: u64,
        duration_ticks: Option<u64>,
    ) -> Result<Self, WorldForgeError> {
        // Load manifest
        let manifest_path = world_path.join("world.toml");
        let manifest = WorldManifest::from_file(&manifest_path)?;
        manifest.validate()?;
        if !manifest.extends.is_empty() {
            return Err(WorldForgeError::new(
                ErrorCode::PackageDependencyMissing,
                "world inheritance requires a resolved local package; remote dependency resolution is not available",
            ));
        }
        if !manifest.mods.is_empty() {
            return Err(WorldForgeError::new(
                ErrorCode::ModLoadFailed,
                "manifest-driven mod resolution is not available; load local modules through worldforge-mod-runtime",
            ));
        }

        // Load scenario
        let scenario_path = world_path.join("scenario.toml");
        let mut scenario = Scenario::from_file(&scenario_path)?;
        let entities_path = world_path.join("entities.toml");
        let entities_config = EntitiesConfig::from_file(&entities_path)?;
        entities_config.validate()?;
        scenario.validate_against(&manifest, &entities_config)?;
        scenario.seed = seed;
        if let Some(ticks) = duration_ticks {
            if ticks == 0 {
                return Err(WorldForgeError::new(
                    ErrorCode::ScenarioInvalid,
                    "duration_ticks must be > 0",
                ));
            }
            scenario.duration_ticks = ticks;
        }

        // Compute fingerprints
        let world_fingerprint = worldforge_package::fingerprint_world(world_path)?;

        let scenario_content = std::fs::read_to_string(&scenario_path)
            .map_err(|e| WorldForgeError::new(ErrorCode::ScenarioMissing, e.to_string()))?;
        let scenario_fingerprint = Fingerprint::hash(scenario_content.as_bytes());

        // Initialize ECS world
        let mut ecs_world = SimulationWorld::new();
        let mut entity_ids = Vec::new();
        let mut entity_id_map = BTreeMap::new();
        let mut entity_name_map = BTreeMap::new();
        let mut supply_links = Vec::new();
        let mut tracked_resources = Vec::new();
        let mut run_entities = Vec::new();
        let mut run_links = Vec::new();

        // Register entities from config
        for (idx, entity_cfg) in entities_config.entities.iter().enumerate() {
            let entity_id = EntityId::deterministic(seed, idx as u64);
            ecs_world.spawn(entity_id);

            // Entity info
            ecs_world.insert_component(
                entity_id,
                EntityInfo {
                    name: entity_cfg.name.clone(),
                    entity_type: entity_cfg.entity_type.clone(),
                    region: entity_cfg.region.clone(),
                },
            );

            // Inventory
            let mut inventory = Inventory::new();
            for (resource, amount) in &entity_cfg.initial_inventory {
                inventory.set(resource, Fixed64::from_f64_lossy(*amount));
                if !tracked_resources.contains(resource) {
                    tracked_resources.push(resource.clone());
                }
            }
            ecs_world.insert_component(entity_id, inventory);

            // Production rule
            if let Some(ref prod) = entity_cfg.production {
                let inputs: Vec<(String, Fixed64)> = prod
                    .inputs
                    .iter()
                    .map(|(r, a)| (r.clone(), Fixed64::from_f64_lossy(*a)))
                    .collect();
                let outputs: Vec<(String, Fixed64)> = prod
                    .outputs
                    .iter()
                    .map(|(r, a)| (r.clone(), Fixed64::from_f64_lossy(*a)))
                    .collect();
                ecs_world.insert_component(
                    entity_id,
                    ProductionRule {
                        name: entity_cfg.name.clone(),
                        inputs,
                        outputs,
                        capacity: Fixed64::ONE,
                        energy_cost: Fixed64::from_f64_lossy(prod.energy_cost),
                    },
                );
                for resource in prod.inputs.keys().chain(prod.outputs.keys()) {
                    if !tracked_resources.contains(resource) {
                        tracked_resources.push(resource.clone());
                    }
                }
            }

            // Price signal
            ecs_world.insert_component(entity_id, PriceSignal::new());

            entity_ids.push((entity_id, entity_cfg.name.clone()));
            entity_id_map.insert(entity_cfg.name.clone(), entity_id);
            entity_name_map.insert(entity_id, entity_cfg.name.clone());
            run_entities.push(RunEntity {
                name: entity_cfg.name.clone(),
                entity_type: entity_cfg.entity_type.clone(),
                region: entity_cfg.region.clone(),
            });
        }

        // Register supply links
        for link_cfg in &entities_config.links {
            let from_id = entity_id_map[&link_cfg.from];
            let to_id = entity_id_map[&link_cfg.to];
            supply_links.push((
                from_id,
                to_id,
                link_cfg.resource.clone(),
                Fixed64::from_f64_lossy(link_cfg.max_per_tick),
            ));
            if !tracked_resources.contains(&link_cfg.resource) {
                tracked_resources.push(link_cfg.resource.clone());
            }
            run_links.push(RunLink {
                from: link_cfg.from.clone(),
                to: link_cfg.to.clone(),
                resource: link_cfg.resource.clone(),
                max_per_tick: link_cfg.max_per_tick,
            });
        }

        tracked_resources.sort();

        let initial_fp = ecs_world.fingerprint();
        let run_id = format!(
            "run-{}-{}-{}",
            world_fingerprint.to_short_hex(),
            scenario_fingerprint.to_short_hex(),
            seed
        );

        let replay_writer = ReplayWriter::new(
            run_id,
            world_fingerprint,
            scenario_fingerprint,
            seed,
            initial_fp,
        );

        Ok(Self {
            world_path: world_path.to_path_buf(),
            world: ecs_world,
            scenario,
            state: RunState::Validated,
            entity_ids,
            entity_id_map,
            entity_name_map,
            supply_links,
            tracked_resources,
            events: Vec::new(),
            replay_writer: Some(replay_writer),
            replay: None,
            world_fingerprint,
            scenario_fingerprint,
            run_entities,
            run_links,
        })
    }

    /// Run the simulation to completion.
    pub fn run(&mut self) -> Result<RunResult, WorldForgeError> {
        if self.state == RunState::Running || self.state == RunState::Completed {
            return Err(WorldForgeError::new(
                ErrorCode::RuntimeStateMismatch,
                "a simulation runtime can only be run once",
            ));
        }
        self.state = RunState::Running;
        let initial_fp = self.world.fingerprint();
        let duration = self.scenario.duration_ticks;

        let mut shortage_count = 0usize;
        let mut production_totals: BTreeMap<String, Fixed64> = BTreeMap::new();
        let mut objectives = self.scenario.objectives.clone();
        let mut snapshots = vec![self.resource_snapshot(0)];
        self.evaluate_continuous_objectives(&mut objectives, &production_totals, 0, false);

        for tick_num in 0..duration {
            let tick = Tick::new(tick_num);
            self.world.set_tick(tick);

            // Check for scheduled events
            self.apply_scheduled_events(tick_num)?;

            // Run economy systems
            let mut tick_events = Vec::new();

            run_production(&mut self.world, &self.entity_ids, tick, &mut tick_events);
            run_transfers(
                &mut self.world,
                &self.supply_links,
                &self.entity_name_map,
                tick,
                &mut tick_events,
            );
            update_prices(
                &mut self.world,
                &self.entity_ids,
                &self.tracked_resources,
                Fixed64::from_int(100),
                Fixed64::from_ratio(1, 10),
                tick,
                &mut tick_events,
            );

            // Count shortages
            for event in &tick_events {
                match &event.event_type {
                    EventType::InventoryShortage { resource, .. } => {
                        shortage_count += 1;
                        for objective in &mut objectives {
                            if matches!(
                                &objective.objective_type,
                                ObjectiveType::AvoidShortage { resource: expected }
                                    if expected == resource
                            ) {
                                objective.status = ObjectiveStatus::Failed;
                                objective.ever_failed = true;
                            }
                        }
                    }
                    EventType::ProductionCompleted {
                        resource, amount, ..
                    } => {
                        let total = production_totals
                            .entry(resource.clone())
                            .or_insert(Fixed64::ZERO);
                        *total += *amount;
                    }
                    _ => {}
                }
            }

            self.evaluate_continuous_objectives(
                &mut objectives,
                &production_totals,
                tick_num + 1,
                false,
            );

            // Record to replay
            if let Some(ref mut writer) = self.replay_writer {
                writer.record_events(&tick_events);
            }

            self.events.extend(tick_events);
            self.world.advance_tick();
            snapshots.push(self.resource_snapshot(tick_num + 1));
        }

        let final_fp = self.world.fingerprint();

        // Evaluate objectives
        self.evaluate_continuous_objectives(&mut objectives, &production_totals, duration, true);
        let mut objective_results = Vec::new();
        for obj in &objectives {
            objective_results.push(ObjectiveResult {
                name: obj.name(),
                status: obj.status.clone(),
            });
        }

        // Build proof
        let mut replay = self
            .replay_writer
            .take()
            .ok_or_else(|| {
                WorldForgeError::new(ErrorCode::RuntimeStateMismatch, "replay writer unavailable")
            })?
            .finalize(final_fp, duration);
        replay.world_path = self.world_path.to_string_lossy().into_owned();
        replay.seal();
        let proof = replay.to_proof();
        self.replay = Some(replay);

        self.state = RunState::Completed;

        Ok(RunResult {
            state: RunState::Completed,
            seed: self.scenario.seed,
            total_ticks: duration,
            initial_state_fingerprint: initial_fp,
            final_state_fingerprint: final_fp,
            world_fingerprint: self.world_fingerprint,
            scenario_fingerprint: self.scenario_fingerprint,
            proof,
            objective_results,
            event_count: self.events.len(),
            shortage_count,
            snapshots,
            entities: self.run_entities.clone(),
            links: self.run_links.clone(),
        })
    }

    /// Get the replay artifact (must call after run()).
    pub fn take_replay(&mut self) -> Option<ReplayArtifact> {
        self.replay.take()
    }

    /// Borrow the finalized replay after `run` completes.
    pub fn replay(&self) -> Option<&ReplayArtifact> {
        self.replay.as_ref()
    }

    fn apply_scheduled_events(&mut self, tick: u64) -> Result<(), WorldForgeError> {
        for scheduled in &self.scenario.events {
            if scheduled.tick == tick {
                match &scheduled.event_type {
                    ScheduledEventType::CapacityChange { target, value } => {
                        if let Some(&entity_id) = self.entity_id_map.get(target) {
                            if let Some(rule) =
                                self.world.get_component_mut::<ProductionRule>(&entity_id)
                            {
                                let old = rule.capacity;
                                rule.capacity = Fixed64::from_f64_lossy(*value);
                                let new = rule.capacity;
                                let event = SimulationEvent::new(
                                    Tick::new(tick),
                                    EventType::CapacityChanged {
                                        entity: target.clone(),
                                        old_capacity: old,
                                        new_capacity: new,
                                    },
                                );
                                if let Some(ref mut writer) = self.replay_writer {
                                    writer.record_event(event.clone());
                                }
                                self.events.push(event);
                            } else {
                                return Err(WorldForgeError::new(
                                    ErrorCode::ScenarioInvalid,
                                    format!(
                                        "capacity_change target '{target}' has no production rule"
                                    ),
                                ));
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn total_inventory(&self, resource: &str) -> Fixed64 {
        self.entity_ids
            .iter()
            .fold(Fixed64::ZERO, |total, (entity, _)| {
                total
                    + self
                        .world
                        .get_component::<Inventory>(entity)
                        .map(|inventory| inventory.get(resource))
                        .unwrap_or(Fixed64::ZERO)
            })
    }

    fn resource_snapshot(&self, tick: u64) -> ResourceSnapshot {
        let levels = self
            .tracked_resources
            .iter()
            .map(|resource| {
                (
                    resource.clone(),
                    self.total_inventory(resource).to_f64_lossy(),
                )
            })
            .collect();
        ResourceSnapshot { tick, levels }
    }

    fn evaluate_continuous_objectives(
        &self,
        objectives: &mut [Objective],
        production_totals: &BTreeMap<String, Fixed64>,
        elapsed_ticks: u64,
        final_evaluation: bool,
    ) {
        for objective in objectives {
            match &objective.objective_type {
                ObjectiveType::MaintainInventory { resource, minimum } => {
                    if self.total_inventory(resource) < Fixed64::from_f64_lossy(*minimum) {
                        objective.status = ObjectiveStatus::Failed;
                        objective.ever_failed = true;
                    } else if !objective.ever_failed {
                        objective.status = ObjectiveStatus::Passed;
                    }
                }
                ObjectiveType::AvoidShortage { .. } => {
                    if !objective.ever_failed {
                        objective.status = ObjectiveStatus::Passed;
                    }
                }
                ObjectiveType::ReachProductionTarget { resource, target } => {
                    let produced = production_totals
                        .get(resource)
                        .copied()
                        .unwrap_or(Fixed64::ZERO);
                    if produced >= Fixed64::from_f64_lossy(*target) {
                        objective.status = ObjectiveStatus::Passed;
                    } else if final_evaluation {
                        objective.status = ObjectiveStatus::Failed;
                    }
                }
                ObjectiveType::ReachInventoryTarget { resource, target } => {
                    if objective.status != ObjectiveStatus::Passed {
                        if self.total_inventory(resource) >= Fixed64::from_f64_lossy(*target) {
                            objective.status = ObjectiveStatus::Passed;
                        } else if final_evaluation {
                            objective.status = ObjectiveStatus::Failed;
                        }
                    }
                }
                ObjectiveType::SurviveUntilTick { tick } => {
                    if elapsed_ticks >= *tick {
                        objective.status = ObjectiveStatus::Passed;
                    } else if final_evaluation {
                        objective.status = ObjectiveStatus::Failed;
                    }
                }
            }
        }
    }

    /// Get current state.
    pub fn state(&self) -> &RunState {
        &self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example(name: &str) -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("examples")
            .join(name)
    }

    #[test]
    fn shortened_run_ignores_events_after_requested_duration() {
        let mut runtime = SimulationRuntime::load(&example("supply-chain"), 42, Some(10)).unwrap();
        let result = runtime.run().unwrap();
        assert_eq!(result.total_ticks, 10);
        assert_eq!(result.snapshots.len(), 11);
    }

    #[test]
    fn finalized_replay_is_available_exactly_once() {
        let mut runtime = SimulationRuntime::load(&example("minimal-world"), 7, Some(2)).unwrap();
        let result = runtime.run().unwrap();
        let replay = runtime.take_replay().expect("run must produce replay");
        assert_eq!(
            replay.final_state_fingerprint,
            result.final_state_fingerprint
        );
        assert!(replay.verify_internal().all_passed());
        assert!(runtime.take_replay().is_none());
        assert_eq!(
            runtime.run().unwrap_err().code,
            ErrorCode::RuntimeStateMismatch
        );
    }

    #[test]
    fn objectives_track_the_whole_run_and_cumulative_production() {
        let directory = std::env::temp_dir().join(format!(
            "worldforge-runtime-objectives-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            directory.join("world.toml"),
            "name = \"objective-test\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        std::fs::write(
            directory.join("scenario.toml"),
            r#"world = "objective-test"
duration_ticks = 3

[[objectives]]
type = "maintain_inventory"
resource = "goods"
minimum = 6.0

[[objectives]]
type = "reach_production_target"
resource = "goods"
target = 3.0

[[objectives]]
type = "avoid_shortage"
resource = "input"
"#,
        )
        .unwrap();
        std::fs::write(
            directory.join("entities.toml"),
            r#"[[entities]]
name = "producer"
entity_type = "producer"
region = "test"
[entities.initial_inventory]
input = 2.0
goods = 5.0
[entities.production.inputs]
input = 1.0
[entities.production.outputs]
goods = 2.0
"#,
        )
        .unwrap();

        let mut runtime = SimulationRuntime::load(&directory, 1, None).unwrap();
        let result = runtime.run().unwrap();
        assert_eq!(result.objective_results[0].status, ObjectiveStatus::Failed);
        assert_eq!(result.objective_results[1].status, ObjectiveStatus::Passed);
        assert_eq!(result.objective_results[2].status, ObjectiveStatus::Failed);

        std::fs::remove_dir_all(directory).unwrap();
    }
}

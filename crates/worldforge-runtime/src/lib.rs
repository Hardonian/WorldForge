//! # worldforge-runtime
//!
//! The simulation execution runtime for World Forge.
//!
//! Owns the full lifecycle of a simulation run: loading, validation,
//! initialization, tick execution, event dispatch, replay recording,
//! proof generation, and result reporting.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use worldforge_core::error::{ErrorCode, WorldForgeError};
use worldforge_core::hash::Fingerprint;
use worldforge_core::{EntityId, Fixed64, Tick};
use worldforge_economy::{run_production, run_transfers, update_prices};
use worldforge_ecs::SimulationWorld;
use worldforge_proof::RunProof;
use worldforge_replay::{ReplayArtifact, ReplayWriter};
use worldforge_world::*;

const MAX_RECENT_EVENTS: usize = 256;

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

/// Current entity state exposed to interactive simulation clients.
#[derive(Debug, Clone, serde::Serialize)]
pub struct EntityState {
    pub name: String,
    pub inventory: BTreeMap<String, f64>,
    pub capacity: Option<f64>,
}

/// Incremental state returned while a simulation is being played.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RunProgress {
    pub state: RunState,
    pub seed: u64,
    pub current_tick: u64,
    pub total_ticks: u64,
    pub state_fingerprint: Fingerprint,
    pub objective_results: Vec<ObjectiveResult>,
    pub event_count: usize,
    pub shortage_count: usize,
    pub snapshot: ResourceSnapshot,
    pub entity_states: Vec<EntityState>,
    pub recent_events: Vec<SimulationEvent>,
    pub entities: Vec<RunEntity>,
    pub links: Vec<RunLink>,
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
    scheduled_events: BTreeMap<u64, Vec<ScheduledEvent>>,
    tracked_resources: Vec<String>,
    resource_totals: BTreeMap<String, Fixed64>,
    replay_writer: Option<ReplayWriter>,
    replay: Option<ReplayArtifact>,
    world_fingerprint: Fingerprint,
    scenario_fingerprint: Fingerprint,
    run_entities: Vec<RunEntity>,
    run_links: Vec<RunLink>,
    initial_state_fingerprint: Fingerprint,
    state_fingerprint: Fingerprint,
    shortage_count: usize,
    event_count: usize,
    production_totals: BTreeMap<String, Fixed64>,
    objectives: Vec<Objective>,
    snapshots: Vec<ResourceSnapshot>,
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
        let mut tracked_resource_set = BTreeSet::new();
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
                if tracked_resource_set.insert(resource.clone()) {
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
            if tracked_resource_set.insert(link_cfg.resource.clone()) {
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

        let mut resource_totals = BTreeMap::new();
        for resource in &tracked_resources {
            resource_totals.insert(resource.clone(), Fixed64::ZERO);
        }
        for (_, entity_id) in &entity_ids {
            if let Some(inventory) = ecs_world.get_component::<Inventory>(entity_id) {
                for (resource, amount) in &inventory.resources {
                    *resource_totals.entry(resource.clone()).or_insert(Fixed64::ZERO) += *amount;
                }
            }
        }

        let mut scheduled_events = BTreeMap::new();
        for scheduled in &scenario.events {
            scheduled_events
                .entry(scheduled.tick)
                .or_insert_with(Vec::new)
                .push(scheduled.clone());
        }

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
        let objectives = scenario.objectives.clone();

        Ok(Self {
            world_path: world_path.to_path_buf(),
            world: ecs_world,
            scenario,
            state: RunState::Validated,
            entity_ids,
            entity_id_map,
            entity_name_map,
            supply_links,
            scheduled_events,
            tracked_resources,
            resource_totals,
            replay_writer: Some(replay_writer),
            replay: None,
            world_fingerprint,
            scenario_fingerprint,
            run_entities,
            run_links,
            initial_state_fingerprint: initial_fp,
            state_fingerprint: initial_fp,
            shortage_count: 0,
            event_count: 0,
            production_totals: BTreeMap::new(),
            objectives,
            snapshots: Vec::new(),
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
        let duration = self.scenario.duration_ticks;
        self.step(duration)?;
        self.completed_result()
    }

    /// Advance an interactive run by up to `tick_count` ticks.
    pub fn step(&mut self, tick_count: u64) -> Result<RunProgress, WorldForgeError> {
        if tick_count == 0 {
            return Err(WorldForgeError::new(
                ErrorCode::ScenarioInvalid,
                "tick_count must be greater than zero",
            ));
        }
        if self.state == RunState::Completed {
            return Ok(self.current_progress());
        }
        if matches!(self.state, RunState::Failed { .. }) {
            return Err(WorldForgeError::new(
                ErrorCode::RuntimeStateMismatch,
                "a failed simulation cannot be advanced",
            ));
        }
        if self.state != RunState::Running {
            self.state = RunState::Running;
            self.snapshots.push(self.resource_snapshot(0));
            self.evaluate_objectives(0, false);
        }

        let duration = self.scenario.duration_ticks;
        let remaining = duration.saturating_sub(self.world.current_tick().value());
        let steps = tick_count.min(remaining);
        let mut recent_events = Vec::new();

        for _ in 0..steps {
            let tick_num = self.world.current_tick().value();
            let tick = Tick::new(tick_num);
            self.world.set_tick(tick);

            let mut tick_events = self.apply_scheduled_events(tick_num)?;

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
            self.refresh_resource_totals();

            for event in &tick_events {
                match &event.event_type {
                    EventType::InventoryShortage { resource, .. } => {
                        self.shortage_count += 1;
                        for objective in &mut self.objectives {
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
                        let total = self
                            .production_totals
                            .entry(resource.clone())
                            .or_insert(Fixed64::ZERO);
                        *total += *amount;
                    }
                    _ => {}
                }
            }

            self.evaluate_objectives(tick_num + 1, false);

            recent_events = tick_events
                .iter()
                .rev()
                .take(MAX_RECENT_EVENTS)
                .rev()
                .cloned()
                .collect::<Vec<_>>();
            let event_count = tick_events.len();
            if let Some(ref mut writer) = self.replay_writer {
                writer.record_events(tick_events);
            }
            self.event_count += event_count;
            self.world.advance_tick();
            self.state_fingerprint = self.world.fingerprint();
            self.snapshots.push(self.resource_snapshot(tick_num + 1));
        }

        if self.world.current_tick().value() >= duration {
            self.finalize()?;
        }

        Ok(self.progress_with_events(recent_events))
    }

    /// Change a producer's capacity during an interactive run.
    pub fn set_capacity(
        &mut self,
        entity: &str,
        capacity: f64,
    ) -> Result<RunProgress, WorldForgeError> {
        if !capacity.is_finite() || !(0.0..=2.0).contains(&capacity) {
            return Err(WorldForgeError::new(
                ErrorCode::ScenarioInvalid,
                "capacity must be a finite number between 0 and 2",
            ));
        }
        if self.state == RunState::Completed || matches!(self.state, RunState::Failed { .. }) {
            return Err(WorldForgeError::new(
                ErrorCode::RuntimeStateMismatch,
                "a completed simulation cannot be changed",
            ));
        }
        let entity_id = self.entity_id_map.get(entity).copied().ok_or_else(|| {
            WorldForgeError::new(
                ErrorCode::ScenarioInvalid,
                format!("entity '{entity}' does not exist"),
            )
        })?;
        let rule = self
            .world
            .get_component_mut::<ProductionRule>(&entity_id)
            .ok_or_else(|| {
                WorldForgeError::new(
                    ErrorCode::ScenarioInvalid,
                    format!("entity '{entity}' has no production capacity"),
                )
            })?;
        let old_capacity = rule.capacity;
        let new_capacity = Fixed64::from_f64_lossy(capacity);
        rule.capacity = new_capacity;
        let event = SimulationEvent::new(
            self.world.current_tick(),
            EventType::CapacityChanged {
                entity: entity.to_string(),
                old_capacity,
                new_capacity,
            },
        );
        if let Some(writer) = &mut self.replay_writer {
            writer.record_event(event.clone());
        }
        self.event_count += 1;
        self.state_fingerprint = self.world.fingerprint();
        Ok(self.progress_with_events(vec![event]))
    }

    fn finalize(&mut self) -> Result<(), WorldForgeError> {
        let duration = self.scenario.duration_ticks;
        self.evaluate_objectives(duration, true);
        let final_fp = self.state_fingerprint;

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
        self.replay = Some(replay);

        self.state = RunState::Completed;

        Ok(())
    }

    fn completed_result(&self) -> Result<RunResult, WorldForgeError> {
        if self.state != RunState::Completed {
            return Err(WorldForgeError::new(
                ErrorCode::RuntimeStateMismatch,
                "simulation has not completed",
            ));
        }
        let proof = self
            .replay
            .as_ref()
            .ok_or_else(|| {
                WorldForgeError::new(ErrorCode::RuntimeStateMismatch, "replay unavailable")
            })?
            .to_proof();

        Ok(RunResult {
            state: RunState::Completed,
            seed: self.scenario.seed,
            total_ticks: self.scenario.duration_ticks,
            initial_state_fingerprint: self.initial_state_fingerprint,
            final_state_fingerprint: self.state_fingerprint,
            world_fingerprint: self.world_fingerprint,
            scenario_fingerprint: self.scenario_fingerprint,
            proof,
            objective_results: self.objective_results(),
            event_count: self.event_count,
            shortage_count: self.shortage_count,
            snapshots: self.snapshots.clone(),
            entities: self.run_entities.clone(),
            links: self.run_links.clone(),
        })
    }

    /// Inspect the current state without advancing the simulation.
    pub fn current_progress(&self) -> RunProgress {
        self.progress_with_events(Vec::new())
    }

    fn progress_with_events(&self, recent_events: Vec<SimulationEvent>) -> RunProgress {
        RunProgress {
            state: self.state.clone(),
            seed: self.scenario.seed,
            current_tick: self.world.current_tick().value(),
            total_ticks: self.scenario.duration_ticks,
            state_fingerprint: self.state_fingerprint,
            objective_results: self.objective_results(),
            event_count: self.event_count,
            shortage_count: self.shortage_count,
            snapshot: self.resource_snapshot(self.world.current_tick().value()),
            entity_states: self.entity_states(),
            recent_events,
            entities: self.run_entities.clone(),
            links: self.run_links.clone(),
        }
    }

    fn objective_results(&self) -> Vec<ObjectiveResult> {
        self.objectives
            .iter()
            .map(|objective| ObjectiveResult {
                name: objective.name(),
                status: objective.status.clone(),
            })
            .collect()
    }

    fn entity_states(&self) -> Vec<EntityState> {
        self.entity_ids
            .iter()
            .map(|(entity_id, name)| {
                let inventory = self
                    .world
                    .get_component::<Inventory>(entity_id)
                    .map(|value| {
                        value
                            .resources
                            .iter()
                            .map(|(resource, amount)| (resource.clone(), amount.to_f64_lossy()))
                            .collect()
                    })
                    .unwrap_or_default();
                let capacity = self
                    .world
                    .get_component::<ProductionRule>(entity_id)
                    .map(|rule| rule.capacity.to_f64_lossy());
                EntityState {
                    name: name.clone(),
                    inventory,
                    capacity,
                }
            })
            .collect()
    }

    /// Get the replay artifact (must call after run()).
    pub fn take_replay(&mut self) -> Option<ReplayArtifact> {
        self.replay.take()
    }

    /// Borrow the finalized replay after `run` completes.
    pub fn replay(&self) -> Option<&ReplayArtifact> {
        self.replay.as_ref()
    }

    fn apply_scheduled_events(&mut self, tick: u64) -> Result<Vec<SimulationEvent>, WorldForgeError> {
        let mut events = Vec::new();
        if let Some(scheduled_events) = self.scheduled_events.get(&tick) {
            for scheduled in scheduled_events {
                if let ScheduledEventType::CapacityChange { target, value } = &scheduled.event_type
                {
                    if let Some(&entity_id) = self.entity_id_map.get(target) {
                        if let Some(rule) =
                            self.world.get_component_mut::<ProductionRule>(&entity_id)
                        {
                            let old = rule.capacity;
                            rule.capacity = Fixed64::from_f64_lossy(*value);
                            events.push(SimulationEvent::new(
                                Tick::new(tick),
                                EventType::CapacityChanged {
                                    entity: target.clone(),
                                    old_capacity: old,
                                    new_capacity: rule.capacity,
                                },
                            ));
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
        if !events.is_empty() {
            self.state_fingerprint = self.world.fingerprint();
        }
        Ok(events)
    }

    fn total_inventory(&self, resource: &str) -> Fixed64 {
        self.resource_totals
            .get(resource)
            .copied()
            .unwrap_or(Fixed64::ZERO)
    }

    fn refresh_resource_totals(&mut self) {
        self.resource_totals.clear();
        for resource in &self.tracked_resources {
            self.resource_totals.insert(resource.clone(), Fixed64::ZERO);
        }
        for (_, entity_id) in &self.entity_ids {
            if let Some(inventory) = self.world.get_component::<Inventory>(entity_id) {
                for (resource, amount) in &inventory.resources {
                    *self
                        .resource_totals
                        .entry(resource.clone())
                        .or_insert(Fixed64::ZERO) += *amount;
                }
            }
        }
    }

    fn resource_snapshot(&self, tick: u64) -> ResourceSnapshot {
        let levels = self
            .resource_totals
            .iter()
            .map(|(resource, amount)| (resource.clone(), amount.to_f64_lossy()))
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

    fn evaluate_objectives(&mut self, elapsed_ticks: u64, final_evaluation: bool) {
        self.evaluate_continuous_objectives(
            &mut self.objectives,
            &self.production_totals,
            elapsed_ticks,
            final_evaluation,
        );
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

    #[test]
    fn incremental_steps_match_a_continuous_run() {
        let path = example("supply-chain");
        let mut continuous = SimulationRuntime::load(&path, 42, Some(25)).unwrap();
        let expected = continuous.run().unwrap();

        let mut incremental = SimulationRuntime::load(&path, 42, Some(25)).unwrap();
        let first = incremental.step(7).unwrap();
        assert_eq!(first.current_tick, 7);
        assert_eq!(first.state, RunState::Running);
        let second = incremental.step(9).unwrap();
        assert_eq!(second.current_tick, 16);
        let final_progress = incremental.step(100).unwrap();
        assert_eq!(final_progress.current_tick, 25);
        assert_eq!(final_progress.state, RunState::Completed);
        let actual = incremental.completed_result().unwrap();

        assert_eq!(
            actual.final_state_fingerprint,
            expected.final_state_fingerprint
        );
        assert_eq!(
            actual.proof.event_chain_root,
            expected.proof.event_chain_root
        );
        assert_eq!(actual.event_count, expected.event_count);
        assert_eq!(actual.shortage_count, expected.shortage_count);
    }

    #[test]
    fn interactive_capacity_changes_are_recorded_and_deterministic() {
        let path = example("supply-chain");
        let run_intervention = || {
            let mut runtime = SimulationRuntime::load(&path, 11, Some(12)).unwrap();
            runtime.step(3).unwrap();
            let progress = runtime.set_capacity("factory", 0.25).unwrap();
            assert_eq!(progress.recent_events.len(), 1);
            assert_eq!(progress.entity_states[2].capacity, Some(0.25));
            runtime.step(20).unwrap();
            runtime.completed_result().unwrap()
        };

        let first = run_intervention();
        let second = run_intervention();
        assert_eq!(
            first.final_state_fingerprint,
            second.final_state_fingerprint
        );
        assert_eq!(first.proof.event_chain_root, second.proof.event_chain_root);
    }
}

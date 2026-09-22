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
use worldforge_core::hash::{Fingerprint, FingerprintBuilder};
use worldforge_core::rng::DeterministicRng;
use worldforge_core::version::EngineVersion;
use worldforge_core::{EntityId, Fixed64, Tick};
use worldforge_ecs::SimulationWorld;
use worldforge_economy::{run_production, run_transfers, update_prices};
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
}

/// Result of an objective evaluation.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ObjectiveResult {
    pub name: String,
    pub status: ObjectiveStatus,
}

/// The simulation runtime — executes worlds.
pub struct SimulationRuntime {
    world: SimulationWorld,
    scenario: Scenario,
    rng: DeterministicRng,
    state: RunState,
    entity_ids: Vec<(EntityId, String)>,
    entity_id_map: BTreeMap<String, EntityId>,
    entity_name_map: BTreeMap<EntityId, String>,
    supply_links: Vec<(EntityId, EntityId, String, Fixed64)>,
    tracked_resources: Vec<String>,
    events: Vec<SimulationEvent>,
    replay_writer: Option<ReplayWriter>,
    world_fingerprint: Fingerprint,
    scenario_fingerprint: Fingerprint,
}

impl SimulationRuntime {
    /// Load and initialize a simulation from a world directory.
    pub fn load(world_path: &Path, seed: u64, duration_ticks: Option<u64>) -> Result<Self, WorldForgeError> {
        // Load manifest
        let manifest_path = world_path.join("world.toml");
        let manifest = WorldManifest::from_file(&manifest_path)?;
        manifest.validate()?;

        // Load scenario
        let scenario_path = world_path.join("scenario.toml");
        let mut scenario = Scenario::from_file(&scenario_path)?;
        scenario.seed = seed;
        if let Some(ticks) = duration_ticks {
            scenario.duration_ticks = ticks;
        }
        scenario.validate()?;

        // Compute fingerprints
        let manifest_content = std::fs::read_to_string(&manifest_path)
            .map_err(|e| WorldForgeError::new(ErrorCode::WorldManifestMissing, e.to_string()))?;
        let world_fingerprint = Fingerprint::hash(manifest_content.as_bytes());

        let scenario_content = std::fs::read_to_string(&scenario_path)
            .map_err(|e| WorldForgeError::new(ErrorCode::ScenarioMissing, e.to_string()))?;
        let scenario_fingerprint = Fingerprint::hash(scenario_content.as_bytes());

        // Load entities definition
        let entities_path = world_path.join("entities.toml");
        let entities_config = Self::load_entities(&entities_path)?;

        // Initialize ECS world
        let mut ecs_world = SimulationWorld::new();
        let mut entity_ids = Vec::new();
        let mut entity_id_map = BTreeMap::new();
        let mut entity_name_map = BTreeMap::new();
        let mut supply_links = Vec::new();
        let mut tracked_resources = Vec::new();

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
            }

            // Price signal
            ecs_world.insert_component(entity_id, PriceSignal::new());

            entity_ids.push((entity_id, entity_cfg.name.clone()));
            entity_id_map.insert(entity_cfg.name.clone(), entity_id);
            entity_name_map.insert(entity_id, entity_cfg.name.clone());
        }

        // Register supply links
        for link_cfg in &entities_config.links {
            if let (Some(&from_id), Some(&to_id)) =
                (entity_id_map.get(&link_cfg.from), entity_id_map.get(&link_cfg.to))
            {
                supply_links.push((
                    from_id,
                    to_id,
                    link_cfg.resource.clone(),
                    Fixed64::from_f64_lossy(link_cfg.max_per_tick),
                ));
            }
        }

        tracked_resources.sort();

        let rng = DeterministicRng::new(seed, "simulation");

        let initial_fp = ecs_world.fingerprint();
        let run_id = format!("run-{}-{}", seed, chrono::Utc::now().timestamp());

        let replay_writer = ReplayWriter::new(
            run_id,
            world_fingerprint,
            scenario_fingerprint,
            seed,
            initial_fp,
        );

        Ok(Self {
            world: ecs_world,
            scenario,
            rng,
            state: RunState::Validated,
            entity_ids,
            entity_id_map,
            entity_name_map,
            supply_links,
            tracked_resources,
            events: Vec::new(),
            replay_writer: Some(replay_writer),
            world_fingerprint,
            scenario_fingerprint,
        })
    }

    /// Run the simulation to completion.
    pub fn run(&mut self) -> Result<RunResult, WorldForgeError> {
        self.state = RunState::Running;
        let initial_fp = self.world.fingerprint();
        let duration = self.scenario.duration_ticks;

        let mut shortage_count = 0usize;

        for tick_num in 0..duration {
            let tick = Tick::new(tick_num);
            self.world.set_tick(tick);

            // Check for scheduled events
            self.apply_scheduled_events(tick_num);

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

            // Count shortages
            for event in &tick_events {
                if matches!(event.event_type, EventType::InventoryShortage { .. }) {
                    shortage_count += 1;
                }
            }

            // Record to replay
            if let Some(ref mut writer) = self.replay_writer {
                writer.record_events(&tick_events);
            }

            self.events.extend(tick_events);
            self.world.advance_tick();
        }

        let final_fp = self.world.fingerprint();

        // Evaluate objectives
        let mut objective_results = Vec::new();
        let mut objectives = self.scenario.objectives.clone();
        for obj in &mut objectives {
            // Get total inventory across all entities for objective evaluation
            let entity_ids = self.entity_ids.clone();
            let world = &self.world;
            obj.evaluate(
                &|resource: &str| -> Fixed64 {
                    let mut total = Fixed64::ZERO;
                    for (eid, _) in &entity_ids {
                        if let Some(inv) = world.get_component::<Inventory>(eid) {
                            total += inv.get(resource);
                        }
                    }
                    total
                },
                duration,
            );
            objective_results.push(ObjectiveResult {
                name: obj.name(),
                status: obj.status.clone(),
            });
        }

        // Build proof
        let replay = self.replay_writer.take().unwrap().finalize(final_fp, duration);
        let proof = replay.to_proof();

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
        })
    }

    /// Get the replay artifact (must call after run()).
    pub fn take_replay(&mut self) -> Option<ReplayArtifact> {
        // Replay is consumed during run() - we need to build it there
        None
    }

    /// Build the replay from the run result.
    pub fn build_replay(&self, result: &RunResult) -> ReplayArtifact {
        let engine = EngineVersion::current();
        let format = worldforge_core::version::formats::replay_format();

        ReplayArtifact {
            format_version: format.version.to_string(),
            engine_version: engine.version.to_string(),
            world_fingerprint: result.world_fingerprint,
            scenario_fingerprint: result.scenario_fingerprint,
            seed: result.seed,
            mod_fingerprints: vec![],
            initial_state_fingerprint: result.initial_state_fingerprint,
            final_state_fingerprint: result.final_state_fingerprint,
            event_chain_root: result.proof.event_chain_root,
            total_ticks: result.total_ticks,
            events: self.events.clone(),
            run_id: result.proof.run_id.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    fn apply_scheduled_events(&mut self, tick: u64) {
        for scheduled in &self.scenario.events {
            if scheduled.tick == tick {
                match &scheduled.event_type {
                    ScheduledEventType::CapacityChange { target, value } => {
                        if let Some(&entity_id) = self.entity_id_map.get(target) {
                            if let Some(rule) = self.world.get_component_mut::<ProductionRule>(&entity_id) {
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
                            }
                        }
                    }
                }
            }
        }
    }

    fn load_entities(path: &Path) -> Result<EntitiesConfig, WorldForgeError> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            WorldForgeError::new(
                ErrorCode::WorldNotFound,
                format!("cannot read entities config: {}", e),
            )
        })?;
        toml::from_str(&content).map_err(|e| {
            WorldForgeError::new(
                ErrorCode::WorldSchemaViolation,
                format!("invalid entities config: {}", e),
            )
        })
    }

    /// Get current state.
    pub fn state(&self) -> &RunState {
        &self.state
    }
}

/// Configuration file format for entity definitions.
#[derive(Debug, serde::Deserialize)]
struct EntitiesConfig {
    #[serde(default)]
    entities: Vec<EntityConfig>,
    #[serde(default)]
    links: Vec<LinkConfig>,
}

#[derive(Debug, serde::Deserialize)]
struct EntityConfig {
    name: String,
    entity_type: String,
    region: String,
    #[serde(default)]
    initial_inventory: BTreeMap<String, f64>,
    production: Option<ProductionConfig>,
}

#[derive(Debug, serde::Deserialize)]
struct ProductionConfig {
    inputs: BTreeMap<String, f64>,
    outputs: BTreeMap<String, f64>,
    #[serde(default)]
    energy_cost: f64,
}

#[derive(Debug, serde::Deserialize)]
struct LinkConfig {
    from: String,
    to: String,
    resource: String,
    max_per_tick: f64,
}

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
/// Charts cannot display millions of distinct samples, and retaining one map
/// per tick makes long simulations memory-bound. Keep enough points for a
/// high-resolution plot while always preserving the initial and final state.
const MAX_RETAINED_SNAPSHOTS: u64 = 2_048;

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
    pub event_type_counts: RunEventCounts,
    pub resource_metrics: Vec<ResourceMetric>,
    pub snapshots: Vec<ResourceSnapshot>,
    pub entities: Vec<RunEntity>,
    pub links: Vec<RunLink>,
    pub city: Option<CityProgress>,
}

/// Aggregate event telemetry retained independently of replay capture.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct RunEventCounts {
    pub production: usize,
    pub transfer: usize,
    pub shortage: usize,
    pub price: usize,
    pub system: usize,
    pub construction: usize,
    pub research: usize,
}

impl RunEventCounts {
    fn record(&mut self, event: &SimulationEvent) {
        match &event.event_type {
            EventType::ProductionCompleted { .. } => self.production += 1,
            EventType::ResourceTransferred { .. } => self.transfer += 1,
            EventType::InventoryShortage { .. } => self.shortage += 1,
            EventType::PriceChanged { .. } => self.price += 1,
            EventType::CapacityChanged { .. }
            | EventType::PlayerCapacityChanged { .. }
            | EventType::ObjectiveUpdated { .. }
            | EventType::PopulationChanged { .. }
            | EventType::SimulationDegraded { .. } => self.system += 1,
            EventType::BuildingConstructed { .. } => self.construction += 1,
            EventType::TechnologyUnlocked { .. } => self.research += 1,
        }
    }
}

/// Aggregate resource levels after a simulation tick.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ResourceSnapshot {
    pub tick: u64,
    pub levels: BTreeMap<String, f64>,
}

/// Exact whole-run telemetry for a tracked resource. Unlike chart snapshots,
/// these extrema are updated every tick and are therefore not affected by
/// visualization downsampling.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceMetric {
    pub resource: String,
    pub initial: f64,
    pub final_level: f64,
    pub minimum: f64,
    pub minimum_tick: u64,
    pub maximum: f64,
    pub maximum_tick: u64,
    pub net_change: f64,
}

#[derive(Debug, Clone)]
struct ResourceExtrema {
    initial: Fixed64,
    minimum: Fixed64,
    minimum_tick: u64,
    maximum: Fixed64,
    maximum_tick: u64,
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
    pub resource_metrics: Vec<ResourceMetric>,
    pub snapshot: ResourceSnapshot,
    pub entity_states: Vec<EntityState>,
    pub recent_events: Vec<SimulationEvent>,
    pub entities: Vec<RunEntity>,
    pub links: Vec<RunLink>,
    pub city: Option<CityProgress>,
}

/// Player-facing city layer derived from proof-chained simulation state.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CityProgress {
    pub treasury: String,
    pub population: f64,
    pub housing: u64,
    pub jobs: u64,
    pub wellbeing: f64,
    pub employment_rate: f64,
    pub districts: Vec<CityDistrictProgress>,
    pub buildings: Vec<CityBuildingProgress>,
    pub technologies: Vec<CityTechnologyProgress>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CityDistrictProgress {
    pub id: String,
    pub name: String,
    pub description: String,
    pub slots: u32,
    pub used_slots: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CityBuildingProgress {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub tags: Vec<String>,
    pub allowed_districts: Vec<String>,
    pub footprint: u32,
    pub count: u32,
    pub max_count: u32,
    pub cost: BTreeMap<String, f64>,
    pub upkeep: BTreeMap<String, f64>,
    pub outputs: BTreeMap<String, f64>,
    pub housing: u32,
    pub jobs: u32,
    pub wellbeing: f64,
    pub requires_technologies: Vec<String>,
    pub unlocked: bool,
    pub affordable: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CityTechnologyProgress {
    pub id: String,
    pub name: String,
    pub description: String,
    pub branch: String,
    pub cost: BTreeMap<String, f64>,
    pub prerequisites: Vec<String>,
    pub excludes: Vec<String>,
    pub researched: bool,
    pub available: bool,
    pub affordable: bool,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct CityRuntimeState {
    /// `district/building` -> constructed count. IDs cannot contain `/`.
    placements: BTreeMap<String, u32>,
    unlocked_technologies: BTreeSet<String>,
}

impl worldforge_ecs::Component for CityRuntimeState {
    fn to_fingerprint_bytes(&self) -> Vec<u8> {
        worldforge_core::serial::to_cbor(self).unwrap_or_default()
    }
}

/// Result of an objective evaluation.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ObjectiveResult {
    pub name: String,
    pub status: ObjectiveStatus,
    pub kind: String,
    pub resource: Option<String>,
    pub current: f64,
    pub target: f64,
    /// Normalized completion in the inclusive range 0.0..=1.0.
    pub progress: f64,
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
    resource_extrema: BTreeMap<String, ResourceExtrema>,
    replay_writer: Option<ReplayWriter>,
    replay: Option<ReplayArtifact>,
    proof: Option<RunProof>,
    retained_events: Vec<SimulationEvent>,
    bounded_event_limit: Option<usize>,
    world_fingerprint: Fingerprint,
    scenario_fingerprint: Fingerprint,
    run_entities: Vec<RunEntity>,
    run_links: Vec<RunLink>,
    city_config: Option<CityConfig>,
    city_treasury_id: Option<EntityId>,
    initial_state_fingerprint: Fingerprint,
    state_fingerprint: Fingerprint,
    shortage_count: usize,
    shortage_counts_by_resource: BTreeMap<String, usize>,
    event_count: usize,
    event_type_counts: RunEventCounts,
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
        Self::load_internal(world_path, seed, duration_ticks, None)
    }

    /// Load a proof-only runtime that retains only the newest event window.
    /// Every event still contributes to aggregate counters and the hash chain.
    pub fn load_bounded(
        world_path: &Path,
        seed: u64,
        duration_ticks: Option<u64>,
        event_limit: usize,
    ) -> Result<Self, WorldForgeError> {
        Self::load_internal(world_path, seed, duration_ticks, Some(event_limit))
    }

    fn load_internal(
        world_path: &Path,
        seed: u64,
        duration_ticks: Option<u64>,
        bounded_event_limit: Option<usize>,
    ) -> Result<Self, WorldForgeError> {
        // Resolve the complete local inheritance graph before initializing ECS.
        let resolved = worldforge_package::resolve_world(world_path)?;
        if !resolved.effective_mods.is_empty() {
            return Err(WorldForgeError::new(
                ErrorCode::ModLoadFailed,
                "manifest-driven mod resolution is not available; load local modules through worldforge-mod-runtime",
            ));
        }

        let resolved_world_path = resolved.path.clone();
        let scenario_path = resolved_world_path.join("scenario.toml");
        let mut scenario = resolved.scenario;
        let entities_config = resolved.entities;
        let city_config = resolved.city;
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
        let world_fingerprint = resolved.fingerprint;

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
                    if tracked_resource_set.insert(resource.clone()) {
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

        let city_treasury_id = city_config
            .as_ref()
            .and_then(|city| entity_id_map.get(&city.treasury).copied());
        if let (Some(city), Some(treasury_id)) = (&city_config, city_treasury_id) {
            ecs_world.insert_component(treasury_id, CityRuntimeState::default());
            let mut track = |resource: &String| {
                if tracked_resource_set.insert(resource.clone()) {
                    tracked_resources.push(resource.clone());
                }
            };
            track(&city.population_resource);
            for resource in city.population_needs.keys() {
                track(resource);
            }
            for building in &city.buildings {
                for resource in building
                    .cost
                    .keys()
                    .chain(building.upkeep.keys())
                    .chain(building.outputs.keys())
                {
                    track(resource);
                }
            }
            for technology in &city.technologies {
                for resource in technology.cost.keys() {
                    track(resource);
                }
            }
        }

        tracked_resources.sort();

        let mut resource_totals = BTreeMap::new();
        for resource in &tracked_resources {
            resource_totals.insert(resource.clone(), Fixed64::ZERO);
        }
        for (entity_id, _) in &entity_ids {
            if let Some(inventory) = ecs_world.get_component::<Inventory>(entity_id) {
                for (resource, amount) in &inventory.resources {
                    *resource_totals
                        .entry(resource.clone())
                        .or_insert(Fixed64::ZERO) += *amount;
                }
            }
        }

        let resource_extrema = resource_totals
            .iter()
            .map(|(resource, amount)| {
                (
                    resource.clone(),
                    ResourceExtrema {
                        initial: *amount,
                        minimum: *amount,
                        minimum_tick: 0,
                        maximum: *amount,
                        maximum_tick: 0,
                    },
                )
            })
            .collect();

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
        let replay_writer = match bounded_event_limit {
            Some(limit) => replay_writer.with_event_limit(limit),
            None => replay_writer,
        };
        let objectives = scenario.objectives.clone();

        Ok(Self {
            world_path: resolved_world_path,
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
            resource_extrema,
            replay_writer: Some(replay_writer),
            replay: None,
            proof: None,
            retained_events: Vec::new(),
            bounded_event_limit,
            world_fingerprint,
            scenario_fingerprint,
            run_entities,
            run_links,
            city_config,
            city_treasury_id,
            initial_state_fingerprint: initial_fp,
            state_fingerprint: initial_fp,
            shortage_count: 0,
            shortage_counts_by_resource: BTreeMap::new(),
            event_count: 0,
            event_type_counts: RunEventCounts::default(),
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
            tick_events.extend(self.run_city_economy(tick)?);
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
            self.refresh_resource_totals(tick_num + 1);

            for event in &tick_events {
                self.event_type_counts.record(event);
                match &event.event_type {
                    EventType::InventoryShortage { resource, .. } => {
                        self.shortage_count += 1;
                        *self
                            .shortage_counts_by_resource
                            .entry(resource.clone())
                            .or_default() += 1;
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

            if !tick_events.is_empty() {
                if tick_events.len() >= MAX_RECENT_EVENTS {
                    recent_events = tick_events[tick_events.len() - MAX_RECENT_EVENTS..].to_vec();
                } else {
                    let overflow = recent_events
                        .len()
                        .saturating_add(tick_events.len())
                        .saturating_sub(MAX_RECENT_EVENTS);
                    if overflow > 0 {
                        recent_events.drain(..overflow);
                    }
                    recent_events.extend(tick_events.iter().cloned());
                }
            }
            let event_count = tick_events.len();
            if let Some(ref mut writer) = self.replay_writer {
                writer.record_events(tick_events);
            }
            self.event_count += event_count;
            self.world.advance_tick();
            let completed_tick = tick_num + 1;
            if self.should_retain_snapshot(completed_tick) {
                self.snapshots.push(self.resource_snapshot(completed_tick));
            }
        }

        // Fingerprinting serializes every component. Do it once per requested
        // batch, not once per internal tick; callers observe the same state.
        self.state_fingerprint = self.world.fingerprint();
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
            EventType::PlayerCapacityChanged {
                entity: entity.to_string(),
                old_capacity,
                new_capacity,
            },
        );
        if let Some(writer) = &mut self.replay_writer {
            writer.record_event(event.clone());
        }
        self.event_count += 1;
        self.event_type_counts.record(&event);
        self.state_fingerprint = self.world.fingerprint();
        Ok(self.progress_with_events(vec![event]))
    }

    /// Construct one configured building in a district. Costs, unlocks, slot
    /// limits, and per-template caps are checked atomically.
    pub fn construct_building(
        &mut self,
        building_id: &str,
        district_id: &str,
    ) -> Result<RunProgress, WorldForgeError> {
        self.ensure_player_action_allowed()?;
        let city = self.city_config.as_ref().ok_or_else(|| {
            WorldForgeError::new(
                ErrorCode::ScenarioInvalid,
                "this world has no city-building rules",
            )
        })?;
        let building = city
            .buildings
            .iter()
            .find(|building| building.id == building_id)
            .ok_or_else(|| {
                WorldForgeError::new(
                    ErrorCode::ScenarioInvalid,
                    format!("building '{building_id}' does not exist"),
                )
            })?
            .clone();
        let district = city
            .districts
            .iter()
            .find(|district| district.id == district_id)
            .ok_or_else(|| {
                WorldForgeError::new(
                    ErrorCode::ScenarioInvalid,
                    format!("district '{district_id}' does not exist"),
                )
            })?;
        if !building
            .allowed_districts
            .iter()
            .any(|allowed| allowed == district_id)
        {
            return Err(action_error(format!(
                "'{}' cannot be constructed in '{}'",
                building.name, district.name
            )));
        }
        let treasury_id = self.city_treasury_id.ok_or_else(|| {
            WorldForgeError::new(ErrorCode::RuntimeInitFailed, "city treasury is unavailable")
        })?;
        let state = self
            .world
            .get_component::<CityRuntimeState>(&treasury_id)
            .cloned()
            .ok_or_else(|| {
                WorldForgeError::new(ErrorCode::RuntimeInitFailed, "city state is unavailable")
            })?;
        if building
            .requires_technologies
            .iter()
            .any(|technology| !state.unlocked_technologies.contains(technology))
        {
            return Err(action_error(format!(
                "building '{}' is locked by the research tree",
                building.id
            )));
        }
        let total_count = city_building_count(&state, &building.id);
        if total_count >= building.max_count {
            return Err(action_error(format!(
                "building '{}' reached its limit of {}",
                building.id, building.max_count
            )));
        }
        let used_slots = city_district_slots(city, &state, district_id);
        if used_slots.saturating_add(building.footprint) > district.slots {
            return Err(action_error(format!(
                "district '{}' does not have {} free slots",
                district.id, building.footprint
            )));
        }
        spend_resources(&mut self.world, treasury_id, &building.cost)?;
        let key = placement_key(district_id, building_id);
        let district_count = {
            let state = self
                .world
                .get_component_mut::<CityRuntimeState>(&treasury_id)
                .expect("validated city state");
            let count = state.placements.entry(key).or_default();
            *count += 1;
            *count
        };
        let event = SimulationEvent::new(
            self.world.current_tick(),
            EventType::BuildingConstructed {
                building: building.id,
                district: district_id.to_string(),
                count: district_count,
            },
        );
        self.record_player_event(event.clone());
        self.refresh_resource_totals(self.world.current_tick().value());
        self.state_fingerprint = self.world.fingerprint();
        Ok(self.progress_with_events(vec![event]))
    }

    /// Unlock a technology in the world's branching research graph.
    pub fn research_technology(
        &mut self,
        technology_id: &str,
    ) -> Result<RunProgress, WorldForgeError> {
        self.ensure_player_action_allowed()?;
        let city = self.city_config.as_ref().ok_or_else(|| {
            WorldForgeError::new(
                ErrorCode::ScenarioInvalid,
                "this world has no research tree",
            )
        })?;
        let technology = city
            .technologies
            .iter()
            .find(|technology| technology.id == technology_id)
            .ok_or_else(|| {
                WorldForgeError::new(
                    ErrorCode::ScenarioInvalid,
                    format!("technology '{technology_id}' does not exist"),
                )
            })?
            .clone();
        let treasury_id = self.city_treasury_id.ok_or_else(|| {
            WorldForgeError::new(ErrorCode::RuntimeInitFailed, "city treasury is unavailable")
        })?;
        let state = self
            .world
            .get_component::<CityRuntimeState>(&treasury_id)
            .ok_or_else(|| {
                WorldForgeError::new(ErrorCode::RuntimeInitFailed, "city state is unavailable")
            })?;
        if state.unlocked_technologies.contains(technology_id) {
            return Err(action_error(format!(
                "technology '{technology_id}' is already researched"
            )));
        }
        if technology
            .prerequisites
            .iter()
            .any(|required| !state.unlocked_technologies.contains(required))
        {
            return Err(action_error(format!(
                "technology '{technology_id}' has unmet prerequisites"
            )));
        }
        if technology
            .excludes
            .iter()
            .any(|excluded| state.unlocked_technologies.contains(excluded))
        {
            return Err(action_error(format!(
                "technology '{technology_id}' is excluded by an earlier choice"
            )));
        }
        spend_resources(&mut self.world, treasury_id, &technology.cost)?;
        self.world
            .get_component_mut::<CityRuntimeState>(&treasury_id)
            .expect("validated city state")
            .unlocked_technologies
            .insert(technology.id.clone());
        let event = SimulationEvent::new(
            self.world.current_tick(),
            EventType::TechnologyUnlocked {
                technology: technology.id,
                branch: technology.branch,
            },
        );
        self.record_player_event(event.clone());
        self.refresh_resource_totals(self.world.current_tick().value());
        self.state_fingerprint = self.world.fingerprint();
        Ok(self.progress_with_events(vec![event]))
    }

    fn ensure_player_action_allowed(&self) -> Result<(), WorldForgeError> {
        if self.state == RunState::Completed || matches!(self.state, RunState::Failed { .. }) {
            return Err(WorldForgeError::new(
                ErrorCode::RuntimeStateMismatch,
                "a completed simulation cannot be changed",
            ));
        }
        Ok(())
    }

    fn record_player_event(&mut self, event: SimulationEvent) {
        if let Some(writer) = &mut self.replay_writer {
            writer.record_event(event.clone());
        }
        self.event_count += 1;
        self.event_type_counts.record(&event);
    }

    fn finalize(&mut self) -> Result<(), WorldForgeError> {
        let duration = self.scenario.duration_ticks;
        self.evaluate_objectives(duration, true);
        let final_fp = self.state_fingerprint;

        // Build proof
        let writer = self.replay_writer.take().ok_or_else(|| {
            WorldForgeError::new(ErrorCode::RuntimeStateMismatch, "replay writer unavailable")
        })?;
        if self.bounded_event_limit.is_some() {
            let (proof, retained_events) = writer.finalize_proof(final_fp, duration);
            self.proof = Some(proof);
            self.retained_events = retained_events;
        } else {
            let mut replay = writer.finalize(final_fp, duration);
            replay.world_path = self.world_path.to_string_lossy().into_owned();
            replay.seal();
            self.proof = Some(replay.to_proof());
            self.replay = Some(replay);
        }

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
            .proof
            .as_ref()
            .ok_or_else(|| {
                WorldForgeError::new(ErrorCode::RuntimeStateMismatch, "run proof unavailable")
            })?
            .clone();

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
            event_type_counts: self.event_type_counts.clone(),
            resource_metrics: self.resource_metrics(),
            snapshots: self.snapshots.clone(),
            entities: self.run_entities.clone(),
            links: self.run_links.clone(),
            city: self.city_progress(),
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
            resource_metrics: self.resource_metrics(),
            snapshot: self.resource_snapshot(self.world.current_tick().value()),
            entity_states: self.entity_states(),
            recent_events,
            entities: self.run_entities.clone(),
            links: self.run_links.clone(),
            city: self.city_progress(),
        }
    }

    fn objective_results(&self) -> Vec<ObjectiveResult> {
        self.objectives
            .iter()
            .map(|objective| {
                let (kind, resource, current, target) = match &objective.objective_type {
                    ObjectiveType::MaintainInventory { resource, minimum } => (
                        "maintain_inventory",
                        Some(resource.clone()),
                        self.resource_extrema
                            .get(resource)
                            .map_or(0.0, |metric| metric.minimum.to_f64_lossy()),
                        *minimum,
                    ),
                    ObjectiveType::AvoidShortage { resource } => (
                        "avoid_shortage",
                        Some(resource.clone()),
                        self.shortage_counts_by_resource
                            .get(resource)
                            .copied()
                            .unwrap_or_default() as f64,
                        0.0,
                    ),
                    ObjectiveType::ReachProductionTarget { resource, target } => (
                        "reach_production_target",
                        Some(resource.clone()),
                        self.production_totals
                            .get(resource)
                            .copied()
                            .unwrap_or(Fixed64::ZERO)
                            .to_f64_lossy(),
                        *target,
                    ),
                    ObjectiveType::ReachInventoryTarget { resource, target } => (
                        "reach_inventory_target",
                        Some(resource.clone()),
                        self.resource_extrema
                            .get(resource)
                            .map_or(0.0, |metric| metric.maximum.to_f64_lossy()),
                        *target,
                    ),
                    ObjectiveType::SurviveUntilTick { tick } => (
                        "survive_until_tick",
                        None,
                        self.world.current_tick().value() as f64,
                        *tick as f64,
                    ),
                };
                let progress = if kind == "avoid_shortage" {
                    if current == 0.0 {
                        1.0
                    } else {
                        0.0
                    }
                } else if target <= 0.0 {
                    1.0
                } else {
                    (current / target).clamp(0.0, 1.0)
                };
                ObjectiveResult {
                    name: objective.name(),
                    status: objective.status.clone(),
                    kind: kind.to_string(),
                    resource,
                    current,
                    target,
                    progress,
                }
            })
            .collect()
    }

    fn resource_metrics(&self) -> Vec<ResourceMetric> {
        self.tracked_resources
            .iter()
            .map(|resource| {
                let final_level = self
                    .resource_totals
                    .get(resource)
                    .copied()
                    .unwrap_or(Fixed64::ZERO);
                let extrema = self.resource_extrema.get(resource);
                let initial = extrema.map_or(Fixed64::ZERO, |value| value.initial);
                ResourceMetric {
                    resource: resource.clone(),
                    initial: initial.to_f64_lossy(),
                    final_level: final_level.to_f64_lossy(),
                    minimum: extrema.map_or(0.0, |value| value.minimum.to_f64_lossy()),
                    minimum_tick: extrema.map_or(0, |value| value.minimum_tick),
                    maximum: extrema.map_or(0.0, |value| value.maximum.to_f64_lossy()),
                    maximum_tick: extrema.map_or(0, |value| value.maximum_tick),
                    net_change: (final_level - initial).to_f64_lossy(),
                }
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

    fn city_progress(&self) -> Option<CityProgress> {
        let city = self.city_config.as_ref()?;
        let treasury_id = self.city_treasury_id?;
        let state = self.world.get_component::<CityRuntimeState>(&treasury_id)?;
        let inventory = self.world.get_component::<Inventory>(&treasury_id)?;
        let (housing, jobs, wellbeing) = city_stats(city, state);
        let population = inventory.get(&city.population_resource).to_f64_lossy();
        let employment_rate = if population <= 0.0 {
            1.0
        } else {
            (jobs as f64 / population).clamp(0.0, 1.0)
        };
        let districts = city
            .districts
            .iter()
            .map(|district| CityDistrictProgress {
                id: district.id.clone(),
                name: district.name.clone(),
                description: district.description.clone(),
                slots: district.slots,
                used_slots: city_district_slots(city, state, &district.id),
            })
            .collect();
        let buildings = city
            .buildings
            .iter()
            .map(|building| {
                let unlocked = building
                    .requires_technologies
                    .iter()
                    .all(|technology| state.unlocked_technologies.contains(technology));
                CityBuildingProgress {
                    id: building.id.clone(),
                    name: building.name.clone(),
                    description: building.description.clone(),
                    category: building.category.clone(),
                    tags: building.tags.clone(),
                    allowed_districts: building.allowed_districts.clone(),
                    footprint: building.footprint,
                    count: city_building_count(state, &building.id),
                    max_count: building.max_count,
                    cost: building.cost.clone(),
                    upkeep: building.upkeep.clone(),
                    outputs: building.outputs.clone(),
                    housing: building.housing,
                    jobs: building.jobs,
                    wellbeing: building.wellbeing,
                    requires_technologies: building.requires_technologies.clone(),
                    unlocked,
                    affordable: unlocked && resources_available(inventory, &building.cost),
                }
            })
            .collect();
        let technologies = city
            .technologies
            .iter()
            .map(|technology| {
                let researched = state.unlocked_technologies.contains(&technology.id);
                let available = !researched
                    && technology
                        .prerequisites
                        .iter()
                        .all(|required| state.unlocked_technologies.contains(required))
                    && technology
                        .excludes
                        .iter()
                        .all(|excluded| !state.unlocked_technologies.contains(excluded));
                CityTechnologyProgress {
                    id: technology.id.clone(),
                    name: technology.name.clone(),
                    description: technology.description.clone(),
                    branch: technology.branch.clone(),
                    cost: technology.cost.clone(),
                    prerequisites: technology.prerequisites.clone(),
                    excludes: technology.excludes.clone(),
                    researched,
                    available,
                    affordable: available && resources_available(inventory, &technology.cost),
                }
            })
            .collect();
        Some(CityProgress {
            treasury: city.treasury.clone(),
            population,
            housing,
            jobs,
            wellbeing,
            employment_rate,
            districts,
            buildings,
            technologies,
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

    fn run_city_economy(&mut self, tick: Tick) -> Result<Vec<SimulationEvent>, WorldForgeError> {
        let (Some(city), Some(treasury_id)) = (self.city_config.clone(), self.city_treasury_id)
        else {
            return Ok(Vec::new());
        };
        let state = self
            .world
            .get_component::<CityRuntimeState>(&treasury_id)
            .cloned()
            .ok_or_else(|| {
                WorldForgeError::new(ErrorCode::RuntimeInitFailed, "city state is unavailable")
            })?;
        let district_tags = city_district_tags(&city, &state);
        let mut events = Vec::new();

        for (key, count) in &state.placements {
            let Some((district_id, building_id)) = key.split_once('/') else {
                continue;
            };
            let Some(building) = city
                .buildings
                .iter()
                .find(|building| building.id == building_id)
            else {
                continue;
            };
            let quantity = Fixed64::from_f64_lossy(f64::from(*count));
            let inventory = self
                .world
                .get_component::<Inventory>(&treasury_id)
                .ok_or_else(|| {
                    WorldForgeError::new(
                        ErrorCode::RuntimeInitFailed,
                        "city treasury inventory is unavailable",
                    )
                })?;
            let shortage = building.upkeep.iter().find_map(|(resource, amount)| {
                let needed = Fixed64::from_f64_lossy(*amount) * quantity;
                let available = inventory.get(resource);
                (available < needed).then(|| (resource.clone(), needed, available))
            });
            if let Some((resource, needed, available)) = shortage {
                events.push(SimulationEvent::new(
                    tick,
                    EventType::InventoryShortage {
                        entity: format!("{district_id}/{building_id}"),
                        resource,
                        needed,
                        available,
                    },
                ));
                continue;
            }

            let inventory = self
                .world
                .get_component_mut::<Inventory>(&treasury_id)
                .expect("validated city treasury inventory");
            for (resource, amount) in &building.upkeep {
                inventory
                    .try_subtract(resource, Fixed64::from_f64_lossy(*amount) * quantity)
                    .expect("upkeep was checked atomically");
            }
            for (resource, amount) in &building.outputs {
                let multiplier = city_output_multiplier(
                    &city,
                    &state,
                    &district_tags,
                    district_id,
                    building,
                    resource,
                );
                let produced = Fixed64::from_f64_lossy(*amount)
                    * quantity
                    * Fixed64::from_f64_lossy(multiplier);
                inventory.add(resource, produced);
                events.push(SimulationEvent::new(
                    tick,
                    EventType::ProductionCompleted {
                        entity: format!("{district_id}/{building_id}"),
                        resource: resource.clone(),
                        amount: produced,
                    },
                ));
            }
        }

        if city.population_growth_per_tick > 0.0 {
            let stats = city_stats(&city, &state);
            let inventory = self
                .world
                .get_component::<Inventory>(&treasury_id)
                .expect("validated city treasury inventory");
            let current = inventory.get(&city.population_resource);
            let capacity = Fixed64::from_f64_lossy(stats.0 as f64);
            let growth = Fixed64::from_f64_lossy(city.population_growth_per_tick)
                .min((capacity - current).max(Fixed64::ZERO));
            if growth > Fixed64::ZERO {
                let can_grow = city.population_needs.iter().all(|(resource, per_person)| {
                    inventory.get(resource) >= Fixed64::from_f64_lossy(*per_person) * growth
                });
                if can_grow {
                    let inventory = self
                        .world
                        .get_component_mut::<Inventory>(&treasury_id)
                        .expect("validated city treasury inventory");
                    for (resource, per_person) in &city.population_needs {
                        inventory
                            .try_subtract(resource, Fixed64::from_f64_lossy(*per_person) * growth)
                            .expect("population needs were checked atomically");
                    }
                    inventory.add(&city.population_resource, growth);
                    events.push(SimulationEvent::new(
                        tick,
                        EventType::PopulationChanged {
                            amount: growth,
                            population: current + growth,
                            housing: stats.0,
                        },
                    ));
                }
            }
        }
        Ok(events)
    }

    /// Events retained by the configured capture policy after completion.
    pub fn retained_events(&self) -> &[SimulationEvent] {
        self.replay
            .as_ref()
            .map(|replay| replay.events.as_slice())
            .unwrap_or(&self.retained_events)
    }

    fn apply_scheduled_events(
        &mut self,
        tick: u64,
    ) -> Result<Vec<SimulationEvent>, WorldForgeError> {
        let mut events = Vec::new();
        if let Some(scheduled_events) = self.scheduled_events.get(&tick) {
            for scheduled in scheduled_events {
                let ScheduledEventType::CapacityChange { target, value } = &scheduled.event_type;
                if let Some(&entity_id) = self.entity_id_map.get(target) {
                    if let Some(rule) = self.world.get_component_mut::<ProductionRule>(&entity_id) {
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
                            format!("capacity_change target '{target}' has no production rule"),
                        ));
                    }
                }
            }
        }
        Ok(events)
    }

    fn refresh_resource_totals(&mut self, tick: u64) {
        self.resource_totals.clear();
        for resource in &self.tracked_resources {
            self.resource_totals.insert(resource.clone(), Fixed64::ZERO);
        }
        for (entity_id, _) in &self.entity_ids {
            if let Some(inventory) = self.world.get_component::<Inventory>(entity_id) {
                for (resource, amount) in &inventory.resources {
                    *self
                        .resource_totals
                        .entry(resource.clone())
                        .or_insert(Fixed64::ZERO) += *amount;
                }
            }
        }
        for (resource, amount) in &self.resource_totals {
            let extrema =
                self.resource_extrema
                    .entry(resource.clone())
                    .or_insert(ResourceExtrema {
                        initial: Fixed64::ZERO,
                        minimum: *amount,
                        minimum_tick: tick,
                        maximum: *amount,
                        maximum_tick: tick,
                    });
            if *amount < extrema.minimum {
                extrema.minimum = *amount;
                extrema.minimum_tick = tick;
            }
            if *amount > extrema.maximum {
                extrema.maximum = *amount;
                extrema.maximum_tick = tick;
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

    fn should_retain_snapshot(&self, tick: u64) -> bool {
        let duration = self.scenario.duration_ticks;
        if tick == duration {
            return true;
        }
        let intervals = MAX_RETAINED_SNAPSHOTS.saturating_sub(1).max(1);
        let stride = duration.saturating_add(intervals - 1) / intervals;
        tick.checked_rem(stride.max(1)) == Some(0)
    }

    fn evaluate_continuous_objectives(
        resource_totals: &BTreeMap<String, Fixed64>,
        objectives: &mut [Objective],
        production_totals: &BTreeMap<String, Fixed64>,
        elapsed_ticks: u64,
        final_evaluation: bool,
    ) {
        for objective in objectives {
            match &objective.objective_type {
                ObjectiveType::MaintainInventory { resource, minimum } => {
                    let inventory = resource_totals
                        .get(resource)
                        .copied()
                        .unwrap_or(Fixed64::ZERO);
                    if inventory < Fixed64::from_f64_lossy(*minimum) {
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
                        let inventory = resource_totals
                            .get(resource)
                            .copied()
                            .unwrap_or(Fixed64::ZERO);
                        if inventory >= Fixed64::from_f64_lossy(*target) {
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
        let resource_totals = &self.resource_totals;
        let production_totals = &self.production_totals;
        let objectives = &mut self.objectives;
        Self::evaluate_continuous_objectives(
            resource_totals,
            objectives,
            production_totals,
            elapsed_ticks,
            final_evaluation,
        );
    }

    /// Get current state.
    pub fn state(&self) -> &RunState {
        &self.state
    }
}

fn placement_key(district: &str, building: &str) -> String {
    format!("{district}/{building}")
}

fn city_building_count(state: &CityRuntimeState, building_id: &str) -> u32 {
    state
        .placements
        .iter()
        .filter_map(|(key, count)| key.split_once('/').map(|(_, building)| (building, count)))
        .filter(|(building, _)| *building == building_id)
        .fold(0u32, |total, (_, count)| total.saturating_add(*count))
}

fn city_district_slots(city: &CityConfig, state: &CityRuntimeState, district_id: &str) -> u32 {
    state
        .placements
        .iter()
        .filter_map(|(key, count)| {
            let (district, building_id) = key.split_once('/')?;
            (district == district_id).then_some((building_id, count))
        })
        .filter_map(|(building_id, count)| {
            city.buildings
                .iter()
                .find(|building| building.id == building_id)
                .map(|building| building.footprint.saturating_mul(*count))
        })
        .fold(0u32, u32::saturating_add)
}

fn city_district_tags(
    city: &CityConfig,
    state: &CityRuntimeState,
) -> BTreeMap<String, BTreeMap<String, u32>> {
    let mut result = BTreeMap::<String, BTreeMap<String, u32>>::new();
    for (key, count) in &state.placements {
        let Some((district, building_id)) = key.split_once('/') else {
            continue;
        };
        let Some(building) = city
            .buildings
            .iter()
            .find(|building| building.id == building_id)
        else {
            continue;
        };
        for tag in &building.tags {
            let tag_count = result
                .entry(district.to_string())
                .or_default()
                .entry(tag.clone())
                .or_default();
            *tag_count = tag_count.saturating_add(*count);
        }
    }
    result
}

fn city_output_multiplier(
    city: &CityConfig,
    state: &CityRuntimeState,
    district_tags: &BTreeMap<String, BTreeMap<String, u32>>,
    district_id: &str,
    building: &BuildingDefinition,
    resource: &str,
) -> f64 {
    let mut multiplier = 1.0;
    for technology in &city.technologies {
        if !state.unlocked_technologies.contains(&technology.id) {
            continue;
        }
        if let Some(value) = technology.effects.building_multipliers.get(&building.id) {
            multiplier *= *value;
        }
        if let Some(value) = technology.effects.resource_multipliers.get(resource) {
            multiplier *= *value;
        }
    }
    if let Some(tags) = district_tags.get(district_id) {
        for synergy in &building.synergies {
            if tags.get(&synergy.with_tag).copied().unwrap_or_default() > 0 {
                multiplier *= synergy.output_multiplier;
            }
        }
    }
    multiplier.clamp(0.1, 100.0)
}

fn city_stats(city: &CityConfig, state: &CityRuntimeState) -> (u64, u64, f64) {
    let mut housing = 0u64;
    let mut jobs = 0u64;
    let mut wellbeing = 0.0;
    for (key, count) in &state.placements {
        let Some((_, building_id)) = key.split_once('/') else {
            continue;
        };
        let Some(building) = city
            .buildings
            .iter()
            .find(|building| building.id == building_id)
        else {
            continue;
        };
        housing = housing.saturating_add(u64::from(building.housing) * u64::from(*count));
        jobs = jobs.saturating_add(u64::from(building.jobs) * u64::from(*count));
        wellbeing += building.wellbeing * f64::from(*count);
    }
    let mut housing_multiplier = 1.0;
    let mut jobs_multiplier = 1.0;
    for technology in &city.technologies {
        if !state.unlocked_technologies.contains(&technology.id) {
            continue;
        }
        if technology.effects.housing_multiplier > 0.0 {
            housing_multiplier *= technology.effects.housing_multiplier;
        }
        if technology.effects.jobs_multiplier > 0.0 {
            jobs_multiplier *= technology.effects.jobs_multiplier;
        }
        wellbeing += technology.effects.wellbeing_bonus;
    }
    (
        (housing as f64 * housing_multiplier).round().max(0.0) as u64,
        (jobs as f64 * jobs_multiplier).round().max(0.0) as u64,
        wellbeing,
    )
}

fn resources_available(inventory: &Inventory, costs: &BTreeMap<String, f64>) -> bool {
    costs
        .iter()
        .all(|(resource, amount)| inventory.get(resource) >= Fixed64::from_f64_lossy(*amount))
}

fn spend_resources(
    world: &mut SimulationWorld,
    treasury_id: EntityId,
    costs: &BTreeMap<String, f64>,
) -> Result<(), WorldForgeError> {
    let inventory = world
        .get_component::<Inventory>(&treasury_id)
        .ok_or_else(|| action_error("city treasury inventory is unavailable"))?;
    if let Some((resource, needed)) = costs
        .iter()
        .find(|(resource, amount)| inventory.get(resource) < Fixed64::from_f64_lossy(**amount))
    {
        return Err(action_error(format!(
            "insufficient {resource}: need {needed}, have {}",
            inventory.get(resource)
        )));
    }
    let inventory = world
        .get_component_mut::<Inventory>(&treasury_id)
        .expect("validated city treasury inventory");
    for (resource, amount) in costs {
        inventory
            .try_subtract(resource, Fixed64::from_f64_lossy(*amount))
            .expect("construction cost was checked atomically");
    }
    Ok(())
}

fn action_error(message: impl Into<String>) -> WorldForgeError {
    WorldForgeError::new(ErrorCode::ScenarioInvalid, message)
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
    fn inherited_world_runs_with_effective_content_and_fingerprint() {
        let path = example("supply-chain-recovery");
        let resolved = worldforge_package::resolve_world(&path).unwrap();
        assert_eq!(resolved.dependencies.len(), 1);
        assert_eq!(resolved.entities.entities.len(), 7);

        let mut runtime = SimulationRuntime::load(&path, 4242, Some(25)).unwrap();
        let result = runtime.run().unwrap();
        assert_eq!(result.entities.len(), 7);
        assert_eq!(result.world_fingerprint, resolved.fingerprint);
        assert!(result.entities.iter().any(|entity| entity.name == "mine"));
        assert!(result
            .entities
            .iter()
            .any(|entity| entity.name == "emergency-smelter"));
    }

    #[test]
    fn long_runs_bound_chart_history_and_preserve_endpoints() {
        let ticks = 5_000;
        let mut runtime =
            SimulationRuntime::load(&example("minimal-world"), 42, Some(ticks)).unwrap();
        let result = runtime.run().unwrap();
        assert!(result.snapshots.len() <= MAX_RETAINED_SNAPSHOTS as usize);
        assert_eq!(
            result.snapshots.first().map(|snapshot| snapshot.tick),
            Some(0)
        );
        assert_eq!(
            result.snapshots.last().map(|snapshot| snapshot.tick),
            Some(ticks)
        );
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
    fn bounded_capture_preserves_proof_and_aggregate_counts() {
        let path = example("stress-test");
        let mut full = SimulationRuntime::load(&path, 42, Some(50)).unwrap();
        let full_result = full.run().unwrap();

        let mut bounded = SimulationRuntime::load_bounded(&path, 42, Some(50), 32).unwrap();
        let bounded_result = bounded.run().unwrap();
        assert_eq!(bounded.retained_events().len(), 32);
        assert!(bounded.replay().is_none());
        assert_eq!(
            bounded_result.proof.event_chain_root,
            full_result.proof.event_chain_root
        );
        assert_eq!(
            bounded_result.final_state_fingerprint,
            full_result.final_state_fingerprint
        );
        let counts = &bounded_result.event_type_counts;
        assert_eq!(
            counts.production + counts.transfer + counts.shortage + counts.price + counts.system,
            bounded_result.event_count
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
        assert_eq!(result.objective_results[0].kind, "maintain_inventory");
        assert_eq!(result.objective_results[0].current, 5.0);
        assert_eq!(result.objective_results[0].target, 6.0);
        assert!((result.objective_results[0].progress - (5.0 / 6.0)).abs() < 0.000_001);
        assert_eq!(result.objective_results[1].current, 4.0);
        assert_eq!(result.objective_results[1].progress, 1.0);
        assert_eq!(result.objective_results[2].current, 1.0);
        assert_eq!(result.objective_results[2].progress, 0.0);

        let goods = result
            .resource_metrics
            .iter()
            .find(|metric| metric.resource == "goods")
            .unwrap();
        assert_eq!(goods.initial, 5.0);
        assert_eq!(goods.final_level, 9.0);
        assert_eq!(goods.minimum, 5.0);
        assert_eq!(goods.minimum_tick, 0);
        assert_eq!(goods.maximum, 9.0);
        assert_eq!(goods.maximum_tick, 2);
        assert_eq!(goods.net_change, 4.0);

        let input = result
            .resource_metrics
            .iter()
            .find(|metric| metric.resource == "input")
            .unwrap();
        assert_eq!(input.minimum, 0.0);
        assert_eq!(input.minimum_tick, 2);

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

    #[test]
    fn city_building_and_research_form_a_deterministic_growth_loop() {
        let play = || {
            let mut runtime =
                SimulationRuntime::load(&example("micro-city"), 99, Some(20)).unwrap();
            let initial = runtime.current_progress().city.unwrap();
            assert_eq!(initial.population, 40.0);
            assert_eq!(initial.housing, 0);

            runtime.research_technology("solar-weave").unwrap();
            runtime
                .construct_building("solar-canopy", "sun-belt")
                .unwrap();
            runtime
                .construct_building("courtyard-homes", "civic-core")
                .unwrap();
            let progress = runtime.step(20).unwrap();
            let city = progress.city.as_ref().unwrap();
            assert_eq!(city.housing, 90);
            assert!(city.population > 40.0);
            assert!(city
                .technologies
                .iter()
                .any(|technology| technology.id == "solar-weave" && technology.researched));
            assert!(runtime
                .retained_events()
                .iter()
                .any(|event| matches!(event.event_type, EventType::BuildingConstructed { .. })));
            assert!(runtime
                .retained_events()
                .iter()
                .any(|event| matches!(event.event_type, EventType::TechnologyUnlocked { .. })));
            (
                progress.state_fingerprint,
                runtime.replay().unwrap().event_chain_root,
            )
        };

        assert_eq!(play(), play());
    }
}

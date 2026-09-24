//! # worldforge-runtime
//!
//! The simulation execution runtime for World Forge.
//!
//! Owns the full lifecycle of a simulation run: loading, validation,
//! initialization, tick execution, event dispatch, replay recording,
//! proof generation, and result reporting.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component as PathComponent, Path};

use worldforge_agent::{AgentAction, AgentContext, AgentPolicy, UtilityAgent, UtilityOption};
use worldforge_core::error::{ErrorCode, WorldForgeError};
use worldforge_core::hash::Fingerprint;
use worldforge_core::{DeterministicRng, EntityId, Fixed64, Tick};
use worldforge_economy::{run_production, run_transfers, update_prices};
use worldforge_ecs::SimulationWorld;
use worldforge_mod_api::{Capability, CapabilityPolicy};
use worldforge_mod_runtime::{load_wasm_mod, ModRuntimeConfig, WasmModInstance};
use worldforge_proof::RunProof;
use worldforge_replay::{ReplayArtifact, ReplayWriter};
use worldforge_world::*;

pub mod analytics;
pub use analytics::{
    run_monte_carlo, BottleneckDiagnosis, EnvelopePoint, LinkElasticity, MonteCarloReport,
    ResourceElasticity,
};

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
    pub governance: usize,
    pub geopolitics: usize,
    pub intrigue: usize,
    pub mods: usize,
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
            EventType::CivicDilemmaOpened { .. }
            | EventType::PlayerCivicDecision { .. }
            | EventType::CivicDecisionResolved { .. }
            | EventType::TrajectoryShifted { .. } => self.governance += 1,
            EventType::GeopoliticalStanceChanged { .. }
            | EventType::TributeCollected { .. }
            | EventType::WarlordIncursion { .. }
            | EventType::RefugeeWaveArrived { .. } => self.geopolitics += 1,
            EventType::CovertOperationResolved { .. }
            | EventType::PlayerIntrigueAction { .. }
            | EventType::TradeSecretAcquired { .. }
            | EventType::CyberAgentStatusChanged { .. }
            | EventType::RogueAgentIncident { .. }
            | EventType::AutonomousActorDecision { .. }
            | EventType::CryptoMarketMoved { .. }
            | EventType::CryptoTradeExecuted { .. } => self.intrigue += 1,
            EventType::ModEventEmitted { .. } => self.mods += 1,
            EventType::SeasonChanged { .. }
            | EventType::WeatherChanged { .. }
            | EventType::EcologicalDisaster { .. }
            | EventType::TradeCaravanArrived { .. } => self.system += 1,
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
    pub governance: CityGovernanceProgress,
    pub trajectory: CityTrajectoryProgress,
    pub systems_debrief: SystemsDebriefProgress,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub geopolitics: Option<CityGeopoliticsProgress>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ecology: Option<CityEcologyProgress>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intrigue: Option<CityIntrigueProgress>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CityIntrigueProgress {
    pub heat: f64,
    pub corporations: Vec<CorporationProgress>,
    pub agents: Vec<CyberAgentProgress>,
    pub markets: Vec<CryptoMarketProgress>,
    pub stolen_secrets: Vec<StolenSecretProgress>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CorporationProgress {
    pub id: String,
    pub name: String,
    pub description: String,
    pub sector: String,
    pub security: f64,
    pub influence: f64,
    pub exposure: f64,
    pub remaining_secrets: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CyberAgentProgress {
    pub id: String,
    pub name: String,
    pub description: String,
    pub skill: f64,
    pub stealth: f64,
    pub loyalty: f64,
    pub containment: f64,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CryptoMarketProgress {
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub price: f64,
    pub initial_price: f64,
    pub change_percent: f64,
    pub volatility: f64,
    pub holdings: f64,
    pub position_value: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StolenSecretProgress {
    pub corporation: String,
    pub secret: String,
    pub name: String,
    pub research_value: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CityGeopoliticsProgress {
    pub defense_posture: String,
    pub military_power: f64,
    pub border_threat: f64,
    pub drought_index: f64,
    pub famine_risk: f64,
    pub active_coalition: Option<String>,
    pub entities: Vec<PoliticalEntityProgress>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PoliticalEntityProgress {
    pub id: String,
    pub name: String,
    pub description: String,
    pub power_structure: String,
    pub ruler_title: String,
    pub stance: String,
    pub loyalty: f64,
    pub military_power: f64,
    pub tribute: BTreeMap<String, f64>,
    pub traits: Vec<String>,
    pub tribute_available: bool,
    pub raid_threat: f64,
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
pub struct CityEcologyProgress {
    pub season: String,
    pub season_progress: f64,
    pub year: u64,
    pub weather: String,
    pub temperature_c: f64,
    pub biosphere_health: f64,
    pub air_quality: f64,
    pub water_purity: f64,
    pub soil_fertility: f64,
    pub disaster_risk: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_disaster: Option<String>,
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
    pub level: u32,
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

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CityGovernanceProgress {
    pub factions: Vec<CivicFactionProgress>,
    pub pending_dilemmas: Vec<CivicDilemmaProgress>,
    pub decisions: Vec<CivicDecisionProgress>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CityTrajectoryProgress {
    pub dominant_axis: Option<String>,
    pub scores: BTreeMap<String, f64>,
    pub momentum: BTreeMap<String, f64>,
    pub turning_points: Vec<TrajectoryTurningPointProgress>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrajectoryTurningPointProgress {
    pub tick: u64,
    pub axis: String,
    pub score: f64,
    pub cause: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemsDebriefProgress {
    pub headline: String,
    pub active_feedback_loops: Vec<FeedbackLoopProgress>,
    pub warnings: Vec<String>,
    pub leverage_points: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackLoopProgress {
    pub axis: String,
    pub score: f64,
    pub momentum: f64,
    pub direction: String,
    pub consequence: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CivicFactionProgress {
    pub id: String,
    pub name: String,
    pub description: String,
    pub support: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CivicDilemmaProgress {
    pub id: String,
    pub title: String,
    pub description: String,
    pub opened_tick: u64,
    pub deadline_tick: Option<u64>,
    pub options: Vec<CivicOptionProgress>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CivicOptionProgress {
    pub id: String,
    pub label: String,
    pub description: String,
    pub cost: BTreeMap<String, f64>,
    pub grants: BTreeMap<String, f64>,
    pub faction_support: BTreeMap<String, f64>,
    pub trajectory: BTreeMap<String, f64>,
    pub affordable: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CivicDecisionProgress {
    pub dilemma: String,
    pub title: String,
    pub option: String,
    pub label: String,
    pub trajectory: BTreeMap<String, f64>,
    pub snowballing_axes: Vec<String>,
    pub counterfactuals: Vec<DecisionCounterfactualProgress>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionCounterfactualProgress {
    pub option: String,
    pub label: String,
    pub trajectory: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct CityRuntimeState {
    /// `district/building` -> constructed count. IDs cannot contain `/`.
    placements: BTreeMap<String, u32>,
    unlocked_technologies: BTreeSet<String>,
    faction_support: BTreeMap<String, Fixed64>,
    opened_dilemmas: BTreeMap<String, u64>,
    decisions: BTreeMap<String, String>,
    #[serde(default)]
    entity_stances: BTreeMap<String, String>,
    #[serde(default)]
    entity_loyalty: BTreeMap<String, Fixed64>,
    #[serde(default)]
    defense_posture: String,
    #[serde(default)]
    last_tribute_tick: BTreeMap<String, u64>,
    #[serde(default)]
    raid_threat_counter: Fixed64,
    #[serde(default)]
    intrigue_heat: Fixed64,
    #[serde(default)]
    corporation_security: BTreeMap<String, Fixed64>,
    #[serde(default)]
    corporation_exposure: BTreeMap<String, Fixed64>,
    #[serde(default)]
    stolen_secrets: BTreeSet<String>,
    #[serde(default)]
    agent_status: BTreeMap<String, String>,
    #[serde(default)]
    agent_loyalty: BTreeMap<String, Fixed64>,
    #[serde(default)]
    crypto_prices: BTreeMap<String, Fixed64>,
    #[serde(default)]
    crypto_holdings: BTreeMap<String, Fixed64>,
    #[serde(default)]
    intrigue_nonce: u64,
    #[serde(default)]
    building_levels: BTreeMap<String, u32>,
    #[serde(default)]
    trajectory_scores: BTreeMap<String, Fixed64>,
    #[serde(default)]
    trajectory_momentum: BTreeMap<String, Fixed64>,
    #[serde(default)]
    trajectory_turning_points: Vec<TrajectoryTurningPointState>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct TrajectoryTurningPointState {
    tick: u64,
    axis: String,
    score: Fixed64,
    cause: String,
}

impl worldforge_ecs::Component for CityRuntimeState {
    fn to_fingerprint_bytes(&self) -> Vec<u8> {
        worldforge_core::serial::to_cbor(self).unwrap_or_default()
    }
}

struct LoadedWorldMod {
    id: String,
    instance: WasmModInstance,
}

#[derive(serde::Deserialize)]
struct ModGrantManifest {
    #[serde(default)]
    capabilities: Vec<Capability>,
}

fn load_manifest_mods(
    world_root: &Path,
    references: &[String],
) -> Result<Vec<LoadedWorldMod>, WorldForgeError> {
    if references.is_empty() {
        return Ok(Vec::new());
    }
    let canonical_root = std::fs::canonicalize(world_root).map_err(|error| {
        WorldForgeError::new(
            ErrorCode::ModLoadFailed,
            format!(
                "cannot resolve world root {}: {error}",
                world_root.display()
            ),
        )
    })?;
    let mut loaded = Vec::with_capacity(references.len());
    let mut ids = BTreeSet::new();
    for reference in references {
        let relative = Path::new(reference);
        if relative.is_absolute()
            || relative.components().any(|component| {
                matches!(
                    component,
                    PathComponent::ParentDir | PathComponent::RootDir | PathComponent::Prefix(_)
                )
            })
        {
            return Err(WorldForgeError::new(
                ErrorCode::ModLoadFailed,
                format!("mod reference '{reference}' must stay inside its world"),
            ));
        }
        if !ids.insert(reference.clone()) {
            return Err(WorldForgeError::new(
                ErrorCode::ModLoadFailed,
                format!("mod reference '{reference}' is duplicated"),
            ));
        }
        let path = std::fs::canonicalize(canonical_root.join(relative)).map_err(|error| {
            WorldForgeError::new(
                ErrorCode::ModLoadFailed,
                format!("cannot load mod '{reference}': {error}"),
            )
        })?;
        if !path.starts_with(&canonical_root)
            || !matches!(
                path.extension().and_then(|ext| ext.to_str()),
                Some("wasm" | "wat")
            )
        {
            return Err(WorldForgeError::new(
                ErrorCode::ModLoadFailed,
                format!("mod '{reference}' must be a .wasm or .wat file inside the world"),
            ));
        }
        let grant_path = path.with_extension("mod.toml");
        let policy = if grant_path.is_file() {
            let content = std::fs::read_to_string(&grant_path).map_err(|error| {
                WorldForgeError::new(
                    ErrorCode::ModLoadFailed,
                    format!("cannot read {}: {error}", grant_path.display()),
                )
            })?;
            let manifest: ModGrantManifest = toml::from_str(&content).map_err(|error| {
                WorldForgeError::new(
                    ErrorCode::ModLoadFailed,
                    format!("invalid {}: {error}", grant_path.display()),
                )
            })?;
            CapabilityPolicy::with_capabilities(manifest.capabilities)
        } else {
            CapabilityPolicy::deny_all()
        };
        let mut instance = load_wasm_mod(&path, policy, ModRuntimeConfig::default())?;
        instance.init()?;
        loaded.push(LoadedWorldMod {
            id: reference.clone(),
            instance,
        });
    }
    Ok(loaded)
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
    mods: Vec<LoadedWorldMod>,
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
        let resolved_world_path = resolved.path.clone();
        let mods = load_manifest_mods(&resolved_world_path, &resolved.effective_mods)?;
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
            let mut city_state = CityRuntimeState {
                faction_support: city
                    .factions
                    .iter()
                    .map(|faction| {
                        (
                            faction.id.clone(),
                            Fixed64::from_f64_lossy(faction.initial_support),
                        )
                    })
                    .collect(),
                corporation_security: city
                    .corporations
                    .iter()
                    .map(|corporation| {
                        (
                            corporation.id.clone(),
                            Fixed64::from_f64_lossy(corporation.security),
                        )
                    })
                    .collect(),
                agent_status: city
                    .cyber_agents
                    .iter()
                    .map(|agent| (agent.id.clone(), agent.initial_status.clone()))
                    .collect(),
                agent_loyalty: city
                    .cyber_agents
                    .iter()
                    .map(|agent| (agent.id.clone(), Fixed64::from_f64_lossy(agent.loyalty)))
                    .collect(),
                crypto_prices: city
                    .crypto_assets
                    .iter()
                    .map(|asset| {
                        (
                            asset.id.clone(),
                            Fixed64::from_f64_lossy(asset.initial_price),
                        )
                    })
                    .collect(),
                ..CityRuntimeState::default()
            };
            if let Some(inventory) = ecs_world.get_component::<Inventory>(&treasury_id) {
                for dilemma in &city.dilemmas {
                    if civic_trigger_met(city, &city_state, inventory, dilemma, 0) {
                        city_state.opened_dilemmas.insert(dilemma.id.clone(), 0);
                    }
                }
            }
            ecs_world.insert_component(treasury_id, city_state);
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
            for dilemma in &city.dilemmas {
                for resource in dilemma
                    .trigger
                    .resource_below
                    .keys()
                    .chain(dilemma.trigger.resource_above.keys())
                {
                    track(resource);
                }
                for option in &dilemma.options {
                    for resource in option.cost.keys().chain(option.grants.keys()) {
                        track(resource);
                    }
                }
            }
            if !city.corporations.is_empty() || !city.crypto_assets.is_empty() {
                for resource in ["credits".to_string(), "research".to_string()] {
                    track(&resource);
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
            mods,
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
            tick_events.extend(self.update_civic_dilemmas(tick)?);
            tick_events.extend(self.run_city_trajectory(tick)?);

            run_production(&mut self.world, &self.entity_ids, tick, &mut tick_events);
            tick_events.extend(self.run_city_economy(tick)?);
            tick_events.extend(self.run_city_geopolitics(tick)?);
            tick_events.extend(self.run_city_ecology(tick)?);
            tick_events.extend(self.run_city_intrigue(tick)?);
            tick_events.extend(self.run_autonomous_actors(tick)?);
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
            let mod_events = self.run_world_mods(tick, &tick_events)?;
            tick_events.extend(mod_events);
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

    fn run_world_mods(
        &mut self,
        tick: Tick,
        source_events: &[SimulationEvent],
    ) -> Result<Vec<SimulationEvent>, WorldForgeError> {
        if self.mods.is_empty() {
            return Ok(Vec::new());
        }
        let aggregate_resources = self.resource_totals.values().fold(0_i64, |total, value| {
            total.saturating_add(i64::from(value.to_int()))
        });
        let event_codes = source_events
            .iter()
            .map(|event| {
                let fingerprint = Fingerprint::hash_cbor(event);
                i64::from_le_bytes(
                    fingerprint.as_bytes()[..8]
                        .try_into()
                        .expect("fingerprint prefix has a fixed size"),
                )
            })
            .collect::<Vec<_>>();
        let mut events = Vec::new();
        for loaded in &mut self.mods {
            loaded.instance.set_resource_amount(aggregate_resources);
            for event_code in &event_codes {
                loaded.instance.on_event(*event_code)?;
            }
            loaded.instance.on_tick(tick.value())?;
            events.extend(
                loaded
                    .instance
                    .take_emitted_events()
                    .into_iter()
                    .map(|value| {
                        SimulationEvent::new(
                            tick,
                            EventType::ModEventEmitted {
                                module: loaded.id.clone(),
                                value,
                            },
                        )
                    }),
            );
        }
        Ok(events)
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

    /// Upgrade an existing building type across the city to the next level (up to Level 3).
    pub fn upgrade_city_building(
        &mut self,
        building_id: &str,
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
            .find(|b| b.id == building_id)
            .ok_or_else(|| {
                WorldForgeError::new(
                    ErrorCode::ScenarioInvalid,
                    format!("building '{building_id}' does not exist"),
                )
            })?
            .clone();

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

        let current_count = city_building_count(&state, building_id);
        if current_count == 0 {
            return Err(action_error(format!(
                "cannot upgrade '{}': no constructed instances in the city",
                building.name
            )));
        }

        let current_level = state.building_levels.get(building_id).copied().unwrap_or(1);
        if current_level >= 3 {
            return Err(action_error(format!(
                "'{}' is already at maximum level 3",
                building.name
            )));
        }

        let mut upgrade_cost = BTreeMap::new();
        for (res, amt) in &building.cost {
            upgrade_cost.insert(res.clone(), amt * 0.75 * (current_level as f64));
        }
        spend_resources(&mut self.world, treasury_id, &upgrade_cost)?;

        let state = self
            .world
            .get_component_mut::<CityRuntimeState>(&treasury_id)
            .expect("validated city state");
        state
            .building_levels
            .insert(building_id.to_string(), current_level + 1);

        let event = SimulationEvent::new(
            self.world.current_tick(),
            EventType::BuildingConstructed {
                building: building.id,
                district: format!("level-{}", current_level + 1),
                count: current_count,
            },
        );
        self.record_player_event(event.clone());
        self.refresh_resource_totals(self.world.current_tick().value());
        self.state_fingerprint = self.world.fingerprint();
        Ok(self.progress_with_events(vec![event]))
    }

    /// Demolish one constructed building instance from a district, refunding 40% of its cost.
    pub fn demolish_city_building(
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
            .find(|b| b.id == building_id)
            .ok_or_else(|| {
                WorldForgeError::new(
                    ErrorCode::ScenarioInvalid,
                    format!("building '{building_id}' does not exist"),
                )
            })?
            .clone();

        let treasury_id = self.city_treasury_id.ok_or_else(|| {
            WorldForgeError::new(ErrorCode::RuntimeInitFailed, "city treasury is unavailable")
        })?;
        let placement_key = format!("{district_id}/{building_id}");
        let state = self
            .world
            .get_component_mut::<CityRuntimeState>(&treasury_id)
            .ok_or_else(|| {
                WorldForgeError::new(ErrorCode::RuntimeInitFailed, "city state is unavailable")
            })?;

        let count = state.placements.get_mut(&placement_key).ok_or_else(|| {
            action_error(format!(
                "no '{}' found in '{}' to demolish",
                building.name, district_id
            ))
        })?;
        if *count == 0 {
            return Err(action_error(format!(
                "no '{}' found in '{}' to demolish",
                building.name, district_id
            )));
        }
        *count -= 1;
        let remaining = *count;
        if remaining == 0 {
            state.placements.remove(&placement_key);
        }

        let inventory = self
            .world
            .get_component_mut::<Inventory>(&treasury_id)
            .expect("validated city inventory");
        for (res, amt) in &building.cost {
            inventory.add(res, Fixed64::from_f64_lossy(amt * 0.40));
        }

        let event = SimulationEvent::new(
            self.world.current_tick(),
            EventType::BuildingConstructed {
                building: building.id,
                district: district_id.to_string(),
                count: remaining as u32,
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

    /// Resolve a currently pending civic dilemma. Choices can immediately
    /// exchange resources and permanently reshape faction support and city
    /// simulation multipliers.
    pub fn make_civic_decision(
        &mut self,
        dilemma_id: &str,
        option_id: &str,
    ) -> Result<RunProgress, WorldForgeError> {
        self.ensure_player_action_allowed()?;
        let city = self.city_config.clone().ok_or_else(|| {
            WorldForgeError::new(
                ErrorCode::ScenarioInvalid,
                "this world has no civic governance rules",
            )
        })?;
        let treasury_id = self.city_treasury_id.ok_or_else(|| {
            WorldForgeError::new(ErrorCode::RuntimeInitFailed, "city treasury is unavailable")
        })?;
        let events =
            self.resolve_civic_option(&city, treasury_id, dilemma_id, option_id, true, true)?;
        for event in &events {
            self.record_player_event(event.clone());
        }
        self.refresh_resource_totals(self.world.current_tick().value());
        self.state_fingerprint = self.world.fingerprint();
        Ok(self.progress_with_events(events))
    }

    /// Execute a geopolitical realm action (tribute, emissary, defense posture, coalition, counter-strike).
    pub fn execute_geopolitical_action(
        &mut self,
        action: &str,
        entity_id: &str,
        option: Option<&str>,
    ) -> Result<RunProgress, WorldForgeError> {
        self.ensure_player_action_allowed()?;
        let city = self.city_config.clone().ok_or_else(|| {
            WorldForgeError::new(
                ErrorCode::ScenarioInvalid,
                "this world has no city configuration",
            )
        })?;
        let treasury_id = self.city_treasury_id.ok_or_else(|| {
            WorldForgeError::new(ErrorCode::RuntimeInitFailed, "city treasury is unavailable")
        })?;

        let tick = self.world.current_tick();

        let event = match action {
            "tribute" => {
                let entity = city
                    .political_entities
                    .iter()
                    .find(|e| e.id == entity_id)
                    .ok_or_else(|| {
                        action_error(format!("political entity '{entity_id}' not found"))
                    })?;
                if entity.tribute.is_empty() {
                    return Err(action_error(format!(
                        "entity '{entity_id}' does not owe tribute"
                    )));
                }

                let state = self
                    .world
                    .get_component_mut::<CityRuntimeState>(&treasury_id)
                    .expect("validated city state");
                let last = state.last_tribute_tick.get(entity_id).copied().unwrap_or(0);
                if tick.value() < last + 10 && last > 0 {
                    return Err(action_error(
                        "tribute is on cooldown; wait before demanding again",
                    ));
                }
                state
                    .last_tribute_tick
                    .insert(entity_id.to_string(), tick.value());

                let loyalty = state
                    .entity_loyalty
                    .entry(entity_id.to_string())
                    .or_insert(Fixed64::from_f64_lossy(entity.initial_loyalty));
                *loyalty = (*loyalty - Fixed64::from_int(8)).max(Fixed64::ZERO);

                let inventory = self
                    .world
                    .get_component_mut::<Inventory>(&treasury_id)
                    .expect("validated city inventory");
                for (res, amt) in &entity.tribute {
                    inventory.add(res, Fixed64::from_f64_lossy(*amt));
                }

                SimulationEvent::new(
                    tick,
                    EventType::TributeCollected {
                        entity: entity_id.to_string(),
                        resources: entity.tribute.clone(),
                    },
                )
            }
            "emissary" => {
                let entity = city
                    .political_entities
                    .iter()
                    .find(|e| e.id == entity_id)
                    .ok_or_else(|| {
                        action_error(format!("political entity '{entity_id}' not found"))
                    })?;

                let cost = BTreeMap::from([("credits".to_string(), 100.0)]);
                spend_resources(&mut self.world, treasury_id, &cost)?;

                let state = self
                    .world
                    .get_component_mut::<CityRuntimeState>(&treasury_id)
                    .expect("validated city state");
                let loyalty = state
                    .entity_loyalty
                    .entry(entity_id.to_string())
                    .or_insert(Fixed64::from_f64_lossy(entity.initial_loyalty));
                *loyalty = (*loyalty + Fixed64::from_int(20)).min(Fixed64::from_int(100));

                let current_stance = state
                    .entity_stances
                    .entry(entity_id.to_string())
                    .or_insert_with(|| entity.initial_stance.clone());
                let old_stance = current_stance.clone();
                if *current_stance == "hostile" {
                    *current_stance = "neutral".to_string();
                } else if *current_stance == "neutral" {
                    *current_stance = "friendly".to_string();
                }

                SimulationEvent::new(
                    tick,
                    EventType::GeopoliticalStanceChanged {
                        entity: entity_id.to_string(),
                        from: old_stance,
                        to: current_stance.clone(),
                    },
                )
            }
            "posture" => {
                let new_posture = option.unwrap_or("fortified").to_string();
                let state = self
                    .world
                    .get_component_mut::<CityRuntimeState>(&treasury_id)
                    .expect("validated city state");
                let old_posture = if state.defense_posture.is_empty() {
                    "standard".to_string()
                } else {
                    state.defense_posture.clone()
                };
                state.defense_posture = new_posture.clone();

                SimulationEvent::new(
                    tick,
                    EventType::GeopoliticalStanceChanged {
                        entity: "defense-garrison".to_string(),
                        from: old_posture,
                        to: new_posture,
                    },
                )
            }
            "coalition" => {
                let entity = city
                    .political_entities
                    .iter()
                    .find(|e| e.id == entity_id)
                    .ok_or_else(|| {
                        action_error(format!("political entity '{entity_id}' not found"))
                    })?;

                let cost = BTreeMap::from([("credits".to_string(), 150.0)]);
                spend_resources(&mut self.world, treasury_id, &cost)?;

                let state = self
                    .world
                    .get_component_mut::<CityRuntimeState>(&treasury_id)
                    .expect("validated city state");
                let current_stance = state
                    .entity_stances
                    .entry(entity_id.to_string())
                    .or_insert_with(|| entity.initial_stance.clone());
                let old = current_stance.clone();
                *current_stance = "coalition".to_string();

                SimulationEvent::new(
                    tick,
                    EventType::GeopoliticalStanceChanged {
                        entity: entity_id.to_string(),
                        from: old,
                        to: "coalition".to_string(),
                    },
                )
            }
            "strike" => {
                let cost = BTreeMap::from([
                    ("materials".to_string(), 150.0),
                    ("credits".to_string(), 100.0),
                ]);
                spend_resources(&mut self.world, treasury_id, &cost)?;

                let state = self
                    .world
                    .get_component_mut::<CityRuntimeState>(&treasury_id)
                    .expect("validated city state");
                state.raid_threat_counter = Fixed64::ZERO;

                SimulationEvent::new(
                    tick,
                    EventType::WarlordIncursion {
                        entity: entity_id.to_string(),
                        damage: 0.0,
                        repelled: true,
                    },
                )
            }
            _ => {
                return Err(action_error(format!(
                    "unknown geopolitical action '{action}'"
                )));
            }
        };

        let impacts: &[(&str, i32)] = match action {
            "tribute" => &[
                ("prosperity", 8),
                ("cohesion", -5),
                ("sovereignty", 5),
                ("risk", 3),
            ],
            "emissary" => &[("cohesion", 7), ("risk", -3)],
            "posture" => &[("sovereignty", 8), ("prosperity", -3)],
            "coalition" => &[("cohesion", 8), ("sovereignty", 3)],
            "strike" => &[("sovereignty", 10), ("risk", 10), ("cohesion", -4)],
            _ => &[],
        };
        let mut events = vec![event];
        events.extend(self.apply_system_trajectory_impacts(
            treasury_id,
            impacts,
            &format!("geopolitics/{action}/{entity_id}"),
            tick,
        )?);
        for event in &events {
            self.record_player_event(event.clone());
        }
        self.refresh_resource_totals(self.world.current_tick().value());
        self.state_fingerprint = self.world.fingerprint();
        Ok(self.progress_with_events(events))
    }

    /// Execute a deterministic corporate-intrigue, cyber-defense, or crypto-market action.
    /// These mechanics are deliberately abstract game systems: outcome odds and consequences
    /// are authored values, never real intrusion techniques.
    pub fn execute_intrigue_action(
        &mut self,
        action: &str,
        target: &str,
        agent_id: Option<&str>,
        option: Option<&str>,
    ) -> Result<RunProgress, WorldForgeError> {
        self.ensure_player_action_allowed()?;
        let city = self
            .city_config
            .clone()
            .ok_or_else(|| action_error("this world has no intrigue configuration"))?;
        let treasury_id = self
            .city_treasury_id
            .ok_or_else(|| action_error("city treasury is unavailable"))?;
        let tick = self.world.current_tick();
        let nonce = self
            .world
            .get_component::<CityRuntimeState>(&treasury_id)
            .expect("validated city state")
            .intrigue_nonce;
        let mut rng = DeterministicRng::new(
            self.scenario.seed,
            &format!("intrigue/{}/{}/{}/{nonce}", tick.value(), action, target),
        );
        let mut events = Vec::new();

        match action {
            "infiltrate" => {
                let agent_id =
                    agent_id.ok_or_else(|| action_error("infiltration needs an agent"))?;
                let agent = city
                    .cyber_agents
                    .iter()
                    .find(|agent| agent.id == agent_id)
                    .ok_or_else(|| action_error(format!("cyber agent '{agent_id}' not found")))?;
                let corporation = city
                    .corporations
                    .iter()
                    .find(|corporation| corporation.id == target)
                    .ok_or_else(|| action_error(format!("corporation '{target}' not found")))?;
                let state = self
                    .world
                    .get_component::<CityRuntimeState>(&treasury_id)
                    .expect("validated city state");
                let status = state
                    .agent_status
                    .get(agent_id)
                    .map(String::as_str)
                    .unwrap_or(agent.initial_status.as_str());
                if status != "ready" {
                    return Err(action_error(format!(
                        "agent '{agent_id}' is {status}, not ready"
                    )));
                }
                let secret = if let Some(secret_id) = option {
                    corporation
                        .secrets
                        .iter()
                        .find(|secret| secret.id == secret_id)
                } else {
                    corporation.secrets.iter().find(|secret| {
                        !state
                            .stolen_secrets
                            .contains(&format!("{}/{}", corporation.id, secret.id))
                    })
                }
                .ok_or_else(|| action_error("that corporation has no available trade secret"))?
                .clone();
                let secret_key = format!("{}/{}", corporation.id, secret.id);
                if state.stolen_secrets.contains(&secret_key) {
                    return Err(action_error("that trade secret was already acquired"));
                }
                let security = state
                    .corporation_security
                    .get(&corporation.id)
                    .copied()
                    .unwrap_or(Fixed64::from_f64_lossy(corporation.security))
                    .to_f64_lossy();
                let operation_cost = BTreeMap::from([("credits".to_string(), 90.0)]);
                spend_resources(&mut self.world, treasury_id, &operation_cost)?;
                let probability = ((agent.skill * 0.55 + agent.stealth * 0.35 + 25.0
                    - security * 0.45
                    - secret.difficulty * 0.25)
                    / 100.0)
                    .clamp(0.08, 0.92);
                let success = rng.chance(Fixed64::from_f64_lossy(probability));
                let detected_probability =
                    ((security + secret.difficulty - agent.stealth) / 160.0).clamp(0.05, 0.9);
                let detected = rng.chance(Fixed64::from_f64_lossy(detected_probability));
                events.push(SimulationEvent::new(
                    tick,
                    EventType::CovertOperationResolved {
                        operation: action.to_string(),
                        target: target.to_string(),
                        agent: agent_id.to_string(),
                        success,
                        detected,
                    },
                ));
                {
                    let state = self
                        .world
                        .get_component_mut::<CityRuntimeState>(&treasury_id)
                        .expect("validated city state");
                    let exposure = state
                        .corporation_exposure
                        .entry(target.to_string())
                        .or_insert(Fixed64::ZERO);
                    *exposure = (*exposure + Fixed64::from_int(if detected { 24 } else { 6 }))
                        .min(Fixed64::from_int(100));
                    state.intrigue_heat = (state.intrigue_heat
                        + Fixed64::from_int(if detected { 18 } else { 4 }))
                    .min(Fixed64::from_int(100));
                    if success {
                        state.stolen_secrets.insert(secret_key);
                        let security = state
                            .corporation_security
                            .entry(target.to_string())
                            .or_insert(Fixed64::from_f64_lossy(corporation.security));
                        *security = (*security - Fixed64::from_int(5)).max(Fixed64::ZERO);
                    }
                    if detected && rng.chance(Fixed64::from_ratio(1, 3)) {
                        let status = state
                            .agent_status
                            .entry(agent_id.to_string())
                            .or_insert_with(|| agent.initial_status.clone());
                        let previous = status.clone();
                        *status = if rng.chance(Fixed64::from_ratio(1, 4)) {
                            "rogue".to_string()
                        } else {
                            "compromised".to_string()
                        };
                        events.push(SimulationEvent::new(
                            tick,
                            EventType::CyberAgentStatusChanged {
                                agent: agent_id.to_string(),
                                from: previous,
                                to: status.clone(),
                            },
                        ));
                    }
                }
                if success {
                    self.world
                        .get_component_mut::<Inventory>(&treasury_id)
                        .expect("validated city inventory")
                        .add("research", Fixed64::from_f64_lossy(secret.research_value));
                    events.push(SimulationEvent::new(
                        tick,
                        EventType::TradeSecretAcquired {
                            corporation: target.to_string(),
                            secret: secret.id,
                            research_value: secret.research_value,
                        },
                    ));
                }
            }
            "deploy" => {
                let agent = city
                    .cyber_agents
                    .iter()
                    .find(|agent| agent.id == target)
                    .ok_or_else(|| action_error(format!("cyber agent '{target}' not found")))?;
                let current_status = self
                    .world
                    .get_component::<CityRuntimeState>(&treasury_id)
                    .expect("validated city state")
                    .agent_status
                    .get(target)
                    .map(String::as_str)
                    .unwrap_or(agent.initial_status.as_str());
                if current_status != "contained" && current_status != "compromised" {
                    return Err(action_error(
                        "only contained or compromised agents can be deployed",
                    ));
                }
                let cost = BTreeMap::from([("research".to_string(), 20.0)]);
                spend_resources(&mut self.world, treasury_id, &cost)?;
                let state = self
                    .world
                    .get_component_mut::<CityRuntimeState>(&treasury_id)
                    .expect("validated city state");
                let status = state
                    .agent_status
                    .entry(target.to_string())
                    .or_insert_with(|| agent.initial_status.clone());
                let previous = status.clone();
                *status = "ready".to_string();
                events.push(SimulationEvent::new(
                    tick,
                    EventType::CyberAgentStatusChanged {
                        agent: target.to_string(),
                        from: previous,
                        to: status.clone(),
                    },
                ));
            }
            "contain" => {
                let agent = city
                    .cyber_agents
                    .iter()
                    .find(|agent| agent.id == target)
                    .ok_or_else(|| action_error(format!("cyber agent '{target}' not found")))?;
                let current_status = self
                    .world
                    .get_component::<CityRuntimeState>(&treasury_id)
                    .expect("validated city state")
                    .agent_status
                    .get(target)
                    .map(String::as_str)
                    .unwrap_or(agent.initial_status.as_str());
                if current_status != "rogue" && current_status != "compromised" {
                    return Err(action_error(
                        "only rogue or compromised agents need containment",
                    ));
                }
                let cost = BTreeMap::from([("credits".to_string(), 120.0)]);
                spend_resources(&mut self.world, treasury_id, &cost)?;
                let state = self
                    .world
                    .get_component_mut::<CityRuntimeState>(&treasury_id)
                    .expect("validated city state");
                let status = state
                    .agent_status
                    .entry(target.to_string())
                    .or_insert_with(|| agent.initial_status.clone());
                let previous = status.clone();
                let probability =
                    ((agent.containment + 110.0 - agent.skill) / 140.0).clamp(0.15, 0.95);
                let success = rng.chance(Fixed64::from_f64_lossy(probability));
                if success {
                    *status = "contained".to_string();
                    state.intrigue_heat =
                        (state.intrigue_heat - Fixed64::from_int(15)).max(Fixed64::ZERO);
                }
                events.push(SimulationEvent::new(
                    tick,
                    EventType::CovertOperationResolved {
                        operation: action.to_string(),
                        target: target.to_string(),
                        agent: "counter-intelligence".to_string(),
                        success,
                        detected: false,
                    },
                ));
                if success {
                    events.push(SimulationEvent::new(
                        tick,
                        EventType::CyberAgentStatusChanged {
                            agent: target.to_string(),
                            from: previous,
                            to: status.clone(),
                        },
                    ));
                }
            }
            "counterintel" => {
                let corporation = city
                    .corporations
                    .iter()
                    .find(|corporation| corporation.id == target)
                    .ok_or_else(|| action_error(format!("corporation '{target}' not found")))?;
                let cost = BTreeMap::from([("credits".to_string(), 75.0)]);
                spend_resources(&mut self.world, treasury_id, &cost)?;
                let state = self
                    .world
                    .get_component_mut::<CityRuntimeState>(&treasury_id)
                    .expect("validated city state");
                let security = state
                    .corporation_security
                    .entry(target.to_string())
                    .or_insert(Fixed64::from_f64_lossy(corporation.security));
                *security = (*security + Fixed64::from_int(10)).min(Fixed64::from_int(100));
                state.intrigue_heat =
                    (state.intrigue_heat - Fixed64::from_int(12)).max(Fixed64::ZERO);
                events.push(SimulationEvent::new(
                    tick,
                    EventType::CovertOperationResolved {
                        operation: action.to_string(),
                        target: target.to_string(),
                        agent: "civic-cyber-command".to_string(),
                        success: true,
                        detected: false,
                    },
                ));
            }
            "manipulate" => {
                let asset = city
                    .crypto_assets
                    .iter()
                    .find(|asset| asset.id == target)
                    .ok_or_else(|| action_error(format!("crypto asset '{target}' not found")))?;
                let direction = option.unwrap_or("pump");
                if !matches!(direction, "pump" | "dump") {
                    return Err(action_error(
                        "market manipulation option must be pump or dump",
                    ));
                }
                let operator = agent_id.unwrap_or("market-desk");
                let skill = if let Some(agent_id) = agent_id {
                    let agent = city
                        .cyber_agents
                        .iter()
                        .find(|agent| agent.id == agent_id)
                        .ok_or_else(|| {
                            action_error(format!("cyber agent '{agent_id}' not found"))
                        })?;
                    let status = self
                        .world
                        .get_component::<CityRuntimeState>(&treasury_id)
                        .expect("validated city state")
                        .agent_status
                        .get(agent_id)
                        .map(String::as_str)
                        .unwrap_or(agent.initial_status.as_str());
                    if status != "ready" {
                        return Err(action_error(format!(
                            "agent '{agent_id}' is {status}, not ready"
                        )));
                    }
                    agent.skill
                } else {
                    45.0
                };
                let cost = BTreeMap::from([("credits".to_string(), 150.0)]);
                spend_resources(&mut self.world, treasury_id, &cost)?;
                let state = self
                    .world
                    .get_component_mut::<CityRuntimeState>(&treasury_id)
                    .expect("validated city state");
                let price = state
                    .crypto_prices
                    .entry(target.to_string())
                    .or_insert(Fixed64::from_f64_lossy(asset.initial_price));
                let old_price = *price;
                let magnitude = asset.volatility * (0.35 + skill / 200.0);
                let multiplier = if direction == "pump" {
                    1.0 + magnitude
                } else {
                    (1.0 - magnitude).max(0.1)
                };
                *price =
                    (*price * Fixed64::from_f64_lossy(multiplier)).max(Fixed64::from_ratio(1, 100));
                state.intrigue_heat =
                    (state.intrigue_heat + Fixed64::from_int(22)).min(Fixed64::from_int(100));
                events.push(SimulationEvent::new(
                    tick,
                    EventType::CovertOperationResolved {
                        operation: format!("market-{direction}"),
                        target: target.to_string(),
                        agent: operator.to_string(),
                        success: true,
                        detected: rng.chance(Fixed64::from_ratio(2, 5)),
                    },
                ));
                events.push(SimulationEvent::new(
                    tick,
                    EventType::CryptoMarketMoved {
                        asset: target.to_string(),
                        old_price,
                        new_price: *price,
                        cause: "manipulation".to_string(),
                    },
                ));
            }
            "trade" => {
                let asset = city
                    .crypto_assets
                    .iter()
                    .find(|asset| asset.id == target)
                    .ok_or_else(|| action_error(format!("crypto asset '{target}' not found")))?;
                let side = option.unwrap_or("buy");
                let state = self
                    .world
                    .get_component::<CityRuntimeState>(&treasury_id)
                    .expect("validated city state");
                let price = state
                    .crypto_prices
                    .get(target)
                    .copied()
                    .unwrap_or(Fixed64::from_f64_lossy(asset.initial_price));
                let units;
                if side == "buy" {
                    let cost = BTreeMap::from([("credits".to_string(), 100.0)]);
                    spend_resources(&mut self.world, treasury_id, &cost)?;
                    units = Fixed64::from_int(100) / price;
                    *self
                        .world
                        .get_component_mut::<CityRuntimeState>(&treasury_id)
                        .expect("validated city state")
                        .crypto_holdings
                        .entry(target.to_string())
                        .or_insert(Fixed64::ZERO) += units;
                } else if side == "sell" {
                    let state = self
                        .world
                        .get_component::<CityRuntimeState>(&treasury_id)
                        .expect("validated city state");
                    units = state
                        .crypto_holdings
                        .get(target)
                        .copied()
                        .unwrap_or(Fixed64::ZERO);
                    if units <= Fixed64::ZERO {
                        return Err(action_error("there is no position to sell"));
                    }
                    self.world
                        .get_component_mut::<CityRuntimeState>(&treasury_id)
                        .expect("validated city state")
                        .crypto_holdings
                        .remove(target);
                    self.world
                        .get_component_mut::<Inventory>(&treasury_id)
                        .expect("validated city inventory")
                        .add("credits", units * price);
                } else {
                    return Err(action_error("crypto trade option must be buy or sell"));
                }
                events.push(SimulationEvent::new(
                    tick,
                    EventType::CryptoTradeExecuted {
                        asset: target.to_string(),
                        side: side.to_string(),
                        units,
                        price,
                    },
                ));
            }
            _ => return Err(action_error(format!("unknown intrigue action '{action}'"))),
        }

        self.world
            .get_component_mut::<CityRuntimeState>(&treasury_id)
            .expect("validated city state")
            .intrigue_nonce = nonce.saturating_add(1);

        let impacts: &[(&str, i32)] = match action {
            "infiltrate" => &[("innovation", 8), ("risk", 10), ("cohesion", -2)],
            "deploy" => &[("innovation", 4), ("risk", 4)],
            "contain" => &[("cohesion", 5), ("risk", -8)],
            "counterintel" => &[("sovereignty", 6), ("risk", -5)],
            "manipulate" => &[("prosperity", 6), ("risk", 15), ("cohesion", -4)],
            "trade" => &[("prosperity", 2), ("risk", 1)],
            _ => &[],
        };
        events.extend(self.apply_system_trajectory_impacts(
            treasury_id,
            impacts,
            &format!("intrigue/{action}/{target}"),
            tick,
        )?);

        events.insert(
            0,
            SimulationEvent::new(
                tick,
                EventType::PlayerIntrigueAction {
                    action: action.to_string(),
                    target: target.to_string(),
                    agent: agent_id.map(str::to_string),
                    option: option.map(str::to_string),
                },
            ),
        );
        for event in &events {
            self.record_player_event(event.clone());
        }
        self.refresh_resource_totals(tick.value());
        self.state_fingerprint = self.world.fingerprint();
        Ok(self.progress_with_events(events))
    }

    fn apply_system_trajectory_impacts(
        &mut self,
        treasury_id: EntityId,
        impacts: &[(&str, i32)],
        cause: &str,
        tick: Tick,
    ) -> Result<Vec<SimulationEvent>, WorldForgeError> {
        let state = self
            .world
            .get_component_mut::<CityRuntimeState>(&treasury_id)
            .ok_or_else(|| action_error("city trajectory state is unavailable"))?;
        let mut events = Vec::with_capacity(impacts.len());
        for (axis, delta) in impacts {
            let old_score = state
                .trajectory_scores
                .get(*axis)
                .copied()
                .unwrap_or(Fixed64::ZERO);
            let delta = Fixed64::from_int(*delta);
            let reinforcement =
                if old_score.raw().signum() == delta.raw().signum() && !old_score.is_zero() {
                    old_score / Fixed64::from_int(10)
                } else {
                    Fixed64::ZERO
                };
            let new_score = (old_score + delta + reinforcement)
                .max(Fixed64::from_int(-100))
                .min(Fixed64::from_int(100));
            let old_momentum = state
                .trajectory_momentum
                .get(*axis)
                .copied()
                .unwrap_or(Fixed64::ZERO);
            let momentum = (old_momentum * Fixed64::from_ratio(3, 4) + delta)
                .max(Fixed64::from_int(-100))
                .min(Fixed64::from_int(100));
            state
                .trajectory_scores
                .insert((*axis).to_string(), new_score);
            state
                .trajectory_momentum
                .insert((*axis).to_string(), momentum);
            let old_tier = old_score.abs().to_int() / 25;
            let new_tier = new_score.abs().to_int() / 25;
            let reversed = old_score.raw().signum() != new_score.raw().signum()
                && !old_score.is_zero()
                && !new_score.is_zero();
            let turning_point = new_tier > old_tier || reversed;
            if turning_point {
                state
                    .trajectory_turning_points
                    .push(TrajectoryTurningPointState {
                        tick: tick.value(),
                        axis: (*axis).to_string(),
                        score: new_score,
                        cause: cause.to_string(),
                    });
                if state.trajectory_turning_points.len() > 32 {
                    state.trajectory_turning_points.remove(0);
                }
            }
            events.push(SimulationEvent::new(
                tick,
                EventType::TrajectoryShifted {
                    axis: (*axis).to_string(),
                    old_score,
                    new_score,
                    momentum,
                    cause: cause.to_string(),
                    turning_point,
                },
            ));
        }
        Ok(events)
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

    fn update_civic_dilemmas(
        &mut self,
        tick: Tick,
    ) -> Result<Vec<SimulationEvent>, WorldForgeError> {
        let (Some(city), Some(treasury_id)) = (self.city_config.clone(), self.city_treasury_id)
        else {
            return Ok(Vec::new());
        };
        if city.dilemmas.is_empty() {
            return Ok(Vec::new());
        }
        let state = self
            .world
            .get_component::<CityRuntimeState>(&treasury_id)
            .cloned()
            .ok_or_else(|| action_error("city governance state is unavailable"))?;
        let inventory = self
            .world
            .get_component::<Inventory>(&treasury_id)
            .ok_or_else(|| action_error("city treasury inventory is unavailable"))?;
        let newly_opened = city
            .dilemmas
            .iter()
            .filter(|dilemma| {
                !state.decisions.contains_key(&dilemma.id)
                    && !state.opened_dilemmas.contains_key(&dilemma.id)
                    && civic_trigger_met(&city, &state, inventory, dilemma, tick.value())
            })
            .map(|dilemma| dilemma.id.clone())
            .collect::<Vec<_>>();

        let mut events = Vec::new();
        if !newly_opened.is_empty() {
            let state = self
                .world
                .get_component_mut::<CityRuntimeState>(&treasury_id)
                .expect("validated city governance state");
            for dilemma_id in newly_opened {
                let dilemma = city
                    .dilemmas
                    .iter()
                    .find(|dilemma| dilemma.id == dilemma_id)
                    .expect("validated dilemma");
                state
                    .opened_dilemmas
                    .insert(dilemma_id.clone(), tick.value());
                events.push(SimulationEvent::new(
                    tick,
                    EventType::CivicDilemmaOpened {
                        dilemma: dilemma_id,
                        deadline_tick: dilemma
                            .deadline_ticks
                            .map(|duration| tick.value().saturating_add(duration)),
                    },
                ));
            }
        }

        let state = self
            .world
            .get_component::<CityRuntimeState>(&treasury_id)
            .cloned()
            .expect("validated city governance state");
        let due = state
            .opened_dilemmas
            .iter()
            .filter_map(|(dilemma_id, opened_tick)| {
                let dilemma = city
                    .dilemmas
                    .iter()
                    .find(|dilemma| dilemma.id == *dilemma_id)?;
                let deadline = opened_tick.checked_add(dilemma.deadline_ticks?)?;
                (tick.value() >= deadline).then(|| {
                    (
                        dilemma_id.clone(),
                        dilemma
                            .default_option
                            .as_ref()
                            .expect("validated default")
                            .clone(),
                    )
                })
            })
            .collect::<Vec<_>>();
        for (dilemma, option) in due {
            events.extend(self.resolve_civic_option(
                &city,
                treasury_id,
                &dilemma,
                &option,
                false,
                false,
            )?);
        }
        Ok(events)
    }

    fn resolve_civic_option(
        &mut self,
        city: &CityConfig,
        treasury_id: EntityId,
        dilemma_id: &str,
        option_id: &str,
        charge_cost: bool,
        player: bool,
    ) -> Result<Vec<SimulationEvent>, WorldForgeError> {
        let dilemma = city
            .dilemmas
            .iter()
            .find(|dilemma| dilemma.id == dilemma_id)
            .ok_or_else(|| action_error(format!("civic dilemma '{dilemma_id}' does not exist")))?;
        let option = dilemma
            .options
            .iter()
            .find(|option| option.id == option_id)
            .ok_or_else(|| {
                action_error(format!(
                    "civic option '{option_id}' does not exist for '{dilemma_id}'"
                ))
            })?;
        let state = self
            .world
            .get_component::<CityRuntimeState>(&treasury_id)
            .ok_or_else(|| action_error("city governance state is unavailable"))?;
        if state.decisions.contains_key(dilemma_id)
            || !state.opened_dilemmas.contains_key(dilemma_id)
        {
            return Err(action_error(format!(
                "civic dilemma '{dilemma_id}' is not pending"
            )));
        }
        if charge_cost {
            spend_resources(&mut self.world, treasury_id, &option.cost)?;
        }
        if !option.grants.is_empty() {
            let inventory = self
                .world
                .get_component_mut::<Inventory>(&treasury_id)
                .expect("validated city treasury inventory");
            for (resource, amount) in &option.grants {
                inventory.add(resource, Fixed64::from_f64_lossy(*amount));
            }
        }
        let tick = self.world.current_tick();
        let state = self
            .world
            .get_component_mut::<CityRuntimeState>(&treasury_id)
            .expect("validated city governance state");
        for (faction, change) in &option.faction_support {
            let support = state
                .faction_support
                .entry(faction.clone())
                .or_insert(Fixed64::from_int(50));
            *support = (*support + Fixed64::from_f64_lossy(*change))
                .max(Fixed64::ZERO)
                .min(Fixed64::from_int(100));
        }
        state.opened_dilemmas.remove(dilemma_id);
        state
            .decisions
            .insert(dilemma_id.to_string(), option_id.to_string());
        let mut events = Vec::with_capacity(option.trajectory.len().saturating_add(1));
        let event_type = if player {
            EventType::PlayerCivicDecision {
                dilemma: dilemma_id.to_string(),
                option: option_id.to_string(),
            }
        } else {
            EventType::CivicDecisionResolved {
                dilemma: dilemma_id.to_string(),
                option: option_id.to_string(),
            }
        };
        events.push(SimulationEvent::new(tick, event_type));
        let cause = format!("{dilemma_id}/{option_id}");
        for (axis, authored_delta) in &option.trajectory {
            let old_score = state
                .trajectory_scores
                .get(axis)
                .copied()
                .unwrap_or(Fixed64::ZERO);
            let delta = Fixed64::from_f64_lossy(*authored_delta);
            let reinforcement = if old_score.raw().signum() == delta.raw().signum()
                && !old_score.is_zero()
                && !delta.is_zero()
            {
                old_score / Fixed64::from_int(10)
            } else {
                Fixed64::ZERO
            };
            let new_score = (old_score + delta + reinforcement)
                .max(Fixed64::from_int(-100))
                .min(Fixed64::from_int(100));
            let old_momentum = state
                .trajectory_momentum
                .get(axis)
                .copied()
                .unwrap_or(Fixed64::ZERO);
            let momentum = (old_momentum * Fixed64::from_ratio(3, 4) + delta)
                .max(Fixed64::from_int(-100))
                .min(Fixed64::from_int(100));
            state.trajectory_scores.insert(axis.clone(), new_score);
            state.trajectory_momentum.insert(axis.clone(), momentum);
            let old_tier = old_score.abs().to_int() / 25;
            let new_tier = new_score.abs().to_int() / 25;
            let reversed = old_score.raw().signum() != new_score.raw().signum()
                && !old_score.is_zero()
                && !new_score.is_zero();
            let turning_point = new_tier > old_tier || reversed;
            if turning_point {
                state
                    .trajectory_turning_points
                    .push(TrajectoryTurningPointState {
                        tick: tick.value(),
                        axis: axis.clone(),
                        score: new_score,
                        cause: cause.clone(),
                    });
                if state.trajectory_turning_points.len() > 32 {
                    state.trajectory_turning_points.remove(0);
                }
            }
            events.push(SimulationEvent::new(
                tick,
                EventType::TrajectoryShifted {
                    axis: axis.clone(),
                    old_score,
                    new_score,
                    momentum,
                    cause: cause.clone(),
                    turning_point,
                },
            ));
        }
        Ok(events)
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
                    level: state
                        .building_levels
                        .get(&building.id)
                        .copied()
                        .unwrap_or(1),
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
        let governance = CityGovernanceProgress {
            factions: city
                .factions
                .iter()
                .map(|faction| CivicFactionProgress {
                    id: faction.id.clone(),
                    name: faction.name.clone(),
                    description: faction.description.clone(),
                    support: state
                        .faction_support
                        .get(&faction.id)
                        .copied()
                        .unwrap_or(Fixed64::from_f64_lossy(faction.initial_support))
                        .to_f64_lossy(),
                })
                .collect(),
            pending_dilemmas: state
                .opened_dilemmas
                .iter()
                .filter_map(|(dilemma_id, opened_tick)| {
                    let dilemma = city
                        .dilemmas
                        .iter()
                        .find(|dilemma| dilemma.id == *dilemma_id)?;
                    Some(CivicDilemmaProgress {
                        id: dilemma.id.clone(),
                        title: dilemma.title.clone(),
                        description: dilemma.description.clone(),
                        opened_tick: *opened_tick,
                        deadline_tick: dilemma
                            .deadline_ticks
                            .map(|duration| opened_tick.saturating_add(duration)),
                        options: dilemma
                            .options
                            .iter()
                            .map(|option| CivicOptionProgress {
                                id: option.id.clone(),
                                label: option.label.clone(),
                                description: option.description.clone(),
                                cost: option.cost.clone(),
                                grants: option.grants.clone(),
                                faction_support: option.faction_support.clone(),
                                trajectory: option.trajectory.clone(),
                                affordable: resources_available(inventory, &option.cost),
                            })
                            .collect(),
                    })
                })
                .collect(),
            decisions: state
                .decisions
                .iter()
                .filter_map(|(dilemma_id, option_id)| {
                    let dilemma = city
                        .dilemmas
                        .iter()
                        .find(|dilemma| dilemma.id == *dilemma_id)?;
                    let option = dilemma
                        .options
                        .iter()
                        .find(|option| option.id == *option_id)?;
                    Some(CivicDecisionProgress {
                        dilemma: dilemma.id.clone(),
                        title: dilemma.title.clone(),
                        option: option.id.clone(),
                        label: option.label.clone(),
                        trajectory: option.trajectory.clone(),
                        snowballing_axes: option
                            .trajectory
                            .iter()
                            .filter_map(|(axis, delta)| {
                                let momentum = state
                                    .trajectory_momentum
                                    .get(axis)
                                    .copied()
                                    .unwrap_or(Fixed64::ZERO)
                                    .to_f64_lossy();
                                ((*delta > 0.0 && momentum > 0.0)
                                    || (*delta < 0.0 && momentum < 0.0))
                                    .then(|| axis.clone())
                            })
                            .collect(),
                        counterfactuals: dilemma
                            .options
                            .iter()
                            .filter(|alternative| alternative.id != option.id)
                            .map(|alternative| DecisionCounterfactualProgress {
                                option: alternative.id.clone(),
                                label: alternative.label.clone(),
                                trajectory: alternative.trajectory.clone(),
                            })
                            .collect(),
                    })
                })
                .collect(),
        };
        let geopolitics = if city.political_entities.is_empty() {
            None
        } else {
            let defense_posture = if state.defense_posture.is_empty() {
                "standard".to_string()
            } else {
                state.defense_posture.clone()
            };
            let military_power = 50.0
                + (jobs as f64 * 0.15)
                + if defense_posture == "fortified" {
                    40.0
                } else {
                    0.0
                };
            let border_threat = state.raid_threat_counter.to_f64_lossy().clamp(0.0, 100.0);
            let current_tick = self.world.current_tick().value();
            let drought_index =
                ((((self.scenario.seed.wrapping_add(current_tick / 40)) % 100) as f64) / 100.0)
                    .clamp(0.0, 1.0);
            let food_level = inventory.get("food").to_f64_lossy();
            let famine_risk = (1.0 - (food_level / 400.0)).clamp(0.0, 1.0);
            let active_coalition = city
                .political_entities
                .iter()
                .find(|e| {
                    state
                        .entity_stances
                        .get(&e.id)
                        .map(String::as_str)
                        .unwrap_or(e.initial_stance.as_str())
                        == "coalition"
                })
                .map(|e| e.name.clone());

            let entities = city
                .political_entities
                .iter()
                .map(|e| {
                    let stance = state
                        .entity_stances
                        .get(&e.id)
                        .cloned()
                        .unwrap_or_else(|| e.initial_stance.clone());
                    let loyalty = state
                        .entity_loyalty
                        .get(&e.id)
                        .copied()
                        .unwrap_or(Fixed64::from_f64_lossy(e.initial_loyalty))
                        .to_f64_lossy();
                    let last = state.last_tribute_tick.get(&e.id).copied().unwrap_or(0);
                    let tribute_available =
                        !e.tribute.is_empty() && (current_tick >= last + 10 || last == 0);
                    let raid_threat = if e.power_structure == "insurgency"
                        || stance == "hostile"
                        || stance == "at-war"
                    {
                        border_threat
                    } else {
                        0.0
                    };
                    PoliticalEntityProgress {
                        id: e.id.clone(),
                        name: e.name.clone(),
                        description: e.description.clone(),
                        power_structure: e.power_structure.clone(),
                        ruler_title: e.ruler_title.clone(),
                        stance,
                        loyalty,
                        military_power: e.military_power,
                        tribute: e.tribute.clone(),
                        traits: e.traits.clone(),
                        tribute_available,
                        raid_threat,
                    }
                })
                .collect();

            Some(CityGeopoliticsProgress {
                defense_posture,
                military_power,
                border_threat,
                drought_index,
                famine_risk,
                active_coalition,
                entities,
            })
        };
        let intrigue = if city.corporations.is_empty()
            && city.cyber_agents.is_empty()
            && city.crypto_assets.is_empty()
        {
            None
        } else {
            let corporations = city
                .corporations
                .iter()
                .map(|corporation| CorporationProgress {
                    id: corporation.id.clone(),
                    name: corporation.name.clone(),
                    description: corporation.description.clone(),
                    sector: corporation.sector.clone(),
                    security: state
                        .corporation_security
                        .get(&corporation.id)
                        .copied()
                        .unwrap_or(Fixed64::from_f64_lossy(corporation.security))
                        .to_f64_lossy(),
                    influence: corporation.influence,
                    exposure: state
                        .corporation_exposure
                        .get(&corporation.id)
                        .copied()
                        .unwrap_or(Fixed64::ZERO)
                        .to_f64_lossy(),
                    remaining_secrets: corporation
                        .secrets
                        .iter()
                        .filter(|secret| {
                            !state
                                .stolen_secrets
                                .contains(&format!("{}/{}", corporation.id, secret.id))
                        })
                        .count(),
                })
                .collect();
            let agents = city
                .cyber_agents
                .iter()
                .map(|agent| CyberAgentProgress {
                    id: agent.id.clone(),
                    name: agent.name.clone(),
                    description: agent.description.clone(),
                    skill: agent.skill,
                    stealth: agent.stealth,
                    loyalty: state
                        .agent_loyalty
                        .get(&agent.id)
                        .copied()
                        .unwrap_or(Fixed64::from_f64_lossy(agent.loyalty))
                        .to_f64_lossy(),
                    containment: agent.containment,
                    status: state
                        .agent_status
                        .get(&agent.id)
                        .cloned()
                        .unwrap_or_else(|| agent.initial_status.clone()),
                })
                .collect();
            let markets = city
                .crypto_assets
                .iter()
                .map(|asset| {
                    let price = state
                        .crypto_prices
                        .get(&asset.id)
                        .copied()
                        .unwrap_or(Fixed64::from_f64_lossy(asset.initial_price));
                    let holdings = state
                        .crypto_holdings
                        .get(&asset.id)
                        .copied()
                        .unwrap_or(Fixed64::ZERO);
                    let price_value = price.to_f64_lossy();
                    CryptoMarketProgress {
                        id: asset.id.clone(),
                        name: asset.name.clone(),
                        symbol: asset.symbol.clone(),
                        price: price_value,
                        initial_price: asset.initial_price,
                        change_percent: ((price_value / asset.initial_price) - 1.0) * 100.0,
                        volatility: asset.volatility,
                        holdings: holdings.to_f64_lossy(),
                        position_value: (holdings * price).to_f64_lossy(),
                    }
                })
                .collect();
            let stolen_secrets = city
                .corporations
                .iter()
                .flat_map(|corporation| {
                    corporation
                        .secrets
                        .iter()
                        .filter(|secret| {
                            state
                                .stolen_secrets
                                .contains(&format!("{}/{}", corporation.id, secret.id))
                        })
                        .map(|secret| StolenSecretProgress {
                            corporation: corporation.id.clone(),
                            secret: secret.id.clone(),
                            name: secret.name.clone(),
                            research_value: secret.research_value,
                        })
                })
                .collect();
            Some(CityIntrigueProgress {
                heat: state.intrigue_heat.to_f64_lossy(),
                corporations,
                agents,
                markets,
                stolen_secrets,
            })
        };
        let ecology = {
            let current_tick = self.world.current_tick().value();
            let cycle = (current_tick / 240) + 1;
            let season_idx = (current_tick % 240) / 60;
            let season_progress = ((current_tick % 60) as f64) / 60.0;
            let (season, base_temp) = match season_idx {
                0 => ("Spring", 16.0),
                1 => ("Summer", 32.0),
                2 => ("Autumn", 17.0),
                _ => ("Winter", 1.0),
            };

            let weather_seed = self.scenario.seed.wrapping_add(current_tick / 30);
            let weather = match season_idx {
                0 => match weather_seed % 3 {
                    0 => "Gentle Rain",
                    1 => "Clear Skies",
                    _ => "Spring Showers",
                },
                1 => match weather_seed % 3 {
                    0 => "Scorching Sun",
                    1 => "Heatwave",
                    _ => "Clear Skies",
                },
                2 => match weather_seed % 3 {
                    0 => "Crisp Autumn Wind",
                    1 => "Overcast Skies",
                    _ => "Harvest Sun",
                },
                _ => match weather_seed % 3 {
                    0 => "Light Snow",
                    1 => "Freezing Frost",
                    _ => "Cold Clear",
                },
            };

            let mut green_count = 0.0;
            let mut industry_count = 0.0;
            for (key, count) in &state.placements {
                if let Some((_, b_id)) = key.split_once('/') {
                    if let Some(b) = city.buildings.iter().find(|b| b.id == b_id) {
                        if b.tags.contains(&"green".to_string())
                            || b.tags.contains(&"water".to_string())
                        {
                            green_count += *count as f64;
                        }
                        if b.tags.contains(&"maker".to_string()) || b.category == "industry" {
                            industry_count += *count as f64;
                        }
                    }
                }
            }

            let biosphere_health =
                (80.0 + (green_count * 5.0) - (industry_count * 6.0)).clamp(10.0, 100.0);
            let air_quality =
                (85.0 + (green_count * 4.0) - (industry_count * 8.0)).clamp(15.0, 100.0);
            let water_purity =
                (80.0 + (green_count * 6.0) - (industry_count * 4.0)).clamp(20.0, 100.0);
            let drought_index =
                (((self.scenario.seed.wrapping_add(current_tick / 40)) % 100) as f64) / 100.0;
            let soil_fertility =
                (88.0 - (drought_index * 35.0) + (green_count * 2.0)).clamp(20.0, 100.0);
            let disaster_risk = if season_idx == 1 && drought_index > 0.6 {
                drought_index * 100.0
            } else if season_idx == 3 && base_temp < 0.0 {
                45.0
            } else {
                15.0
            };
            let active_disaster = if season_idx == 1 && drought_index > 0.65 {
                Some("Severe Drought".to_string())
            } else if season_idx == 3 && base_temp < 0.0 {
                Some("Frost Snap".to_string())
            } else if biosphere_health < 30.0 {
                Some("Industrial Smog".to_string())
            } else {
                None
            };

            Some(CityEcologyProgress {
                season: season.to_string(),
                season_progress,
                year: cycle,
                weather: weather.to_string(),
                temperature_c: base_temp,
                biosphere_health,
                air_quality,
                water_purity,
                soil_fertility,
                disaster_risk,
                active_disaster,
            })
        };
        let trajectory = {
            let scores: BTreeMap<String, f64> = state
                .trajectory_scores
                .iter()
                .map(|(k, v)| (k.clone(), v.to_f64_lossy()))
                .collect();
            let momentum: BTreeMap<String, f64> = state
                .trajectory_momentum
                .iter()
                .map(|(k, v)| (k.clone(), v.to_f64_lossy()))
                .collect();
            let dominant_axis = scores
                .iter()
                .max_by(|a, b| {
                    a.1.abs()
                        .partial_cmp(&b.1.abs())
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(k, _)| k.clone());
            let turning_points = state
                .trajectory_turning_points
                .iter()
                .map(|tp| TrajectoryTurningPointProgress {
                    tick: tp.tick,
                    axis: tp.axis.clone(),
                    score: tp.score.to_f64_lossy(),
                    cause: tp.cause.clone(),
                })
                .collect();
            CityTrajectoryProgress {
                dominant_axis,
                scores,
                momentum,
                turning_points,
            }
        };
        let systems_debrief = build_systems_debrief(state, &trajectory, wellbeing, employment_rate);
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
            governance,
            trajectory,
            systems_debrief,
            geopolitics,
            ecology,
            intrigue,
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

    fn run_city_trajectory(&mut self, tick: Tick) -> Result<Vec<SimulationEvent>, WorldForgeError> {
        let Some(treasury_id) = self.city_treasury_id else {
            return Ok(Vec::new());
        };
        if tick.value() == 0 || tick.value() % 10 != 0 {
            return Ok(Vec::new());
        }
        let state = self
            .world
            .get_component_mut::<CityRuntimeState>(&treasury_id)
            .ok_or_else(|| action_error("city trajectory state is unavailable"))?;
        let axes = state
            .trajectory_momentum
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        let mut events = Vec::new();
        for axis in axes {
            let old_score = state
                .trajectory_scores
                .get(&axis)
                .copied()
                .unwrap_or(Fixed64::ZERO);
            let old_momentum = state
                .trajectory_momentum
                .get(&axis)
                .copied()
                .unwrap_or(Fixed64::ZERO);
            if old_momentum.is_zero() {
                continue;
            }
            let new_score = (old_score + old_momentum / Fixed64::from_int(20))
                .max(Fixed64::from_int(-100))
                .min(Fixed64::from_int(100));
            let mut new_momentum = old_momentum * Fixed64::from_ratio(19, 20);
            if new_momentum.abs() < Fixed64::from_ratio(1, 20) {
                new_momentum = Fixed64::ZERO;
            }
            state.trajectory_scores.insert(axis.clone(), new_score);
            state.trajectory_momentum.insert(axis.clone(), new_momentum);
            let old_tier = old_score.abs().to_int() / 25;
            let new_tier = new_score.abs().to_int() / 25;
            let turning_point = new_tier > old_tier;
            if turning_point {
                state
                    .trajectory_turning_points
                    .push(TrajectoryTurningPointState {
                        tick: tick.value(),
                        axis: axis.clone(),
                        score: new_score,
                        cause: "compounding momentum".to_string(),
                    });
                if state.trajectory_turning_points.len() > 32 {
                    state.trajectory_turning_points.remove(0);
                }
            }
            events.push(SimulationEvent::new(
                tick,
                EventType::TrajectoryShifted {
                    axis,
                    old_score,
                    new_score,
                    momentum: new_momentum,
                    cause: "compounding momentum".to_string(),
                    turning_point,
                },
            ));
        }
        if state
            .trajectory_scores
            .get("risk")
            .copied()
            .unwrap_or(Fixed64::ZERO)
            > Fixed64::from_int(50)
        {
            state.intrigue_heat = (state.intrigue_heat + Fixed64::ONE).min(Fixed64::from_int(100));
        }
        if state
            .trajectory_scores
            .get("cohesion")
            .copied()
            .unwrap_or(Fixed64::ZERO)
            < Fixed64::from_int(-50)
        {
            for support in state.faction_support.values_mut() {
                *support = (*support - Fixed64::ONE).max(Fixed64::ZERO);
            }
        }
        Ok(events)
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
            let growth = Fixed64::from_f64_lossy(
                city.population_growth_per_tick * civic_population_growth_multiplier(&city, &state),
            )
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

    fn run_city_geopolitics(
        &mut self,
        tick: Tick,
    ) -> Result<Vec<SimulationEvent>, WorldForgeError> {
        let (Some(city), Some(treasury_id)) = (self.city_config.clone(), self.city_treasury_id)
        else {
            return Ok(Vec::new());
        };
        if city.political_entities.is_empty() {
            return Ok(Vec::new());
        }

        let mut events = Vec::new();
        let tick_val = tick.value();

        let mut state = self
            .world
            .get_component::<CityRuntimeState>(&treasury_id)
            .cloned()
            .ok_or_else(|| action_error("city state is unavailable"))?;

        if tick_val > 0 && tick_val.is_multiple_of(25) {
            let has_hostiles = city.political_entities.iter().any(|e| {
                let stance = state
                    .entity_stances
                    .get(&e.id)
                    .map(String::as_str)
                    .unwrap_or(e.initial_stance.as_str());
                e.power_structure == "insurgency" || stance == "hostile" || stance == "at-war"
            });
            if has_hostiles {
                state.raid_threat_counter += Fixed64::from_int(25);
                if state.raid_threat_counter >= Fixed64::from_int(100) {
                    state.raid_threat_counter = Fixed64::ZERO;
                    let is_fortified = state.defense_posture == "fortified";
                    let hostile_id = city
                        .political_entities
                        .iter()
                        .find(|e| {
                            let stance = state
                                .entity_stances
                                .get(&e.id)
                                .map(String::as_str)
                                .unwrap_or(e.initial_stance.as_str());
                            e.power_structure == "insurgency"
                                || stance == "hostile"
                                || stance == "at-war"
                        })
                        .map(|e| e.id.clone())
                        .unwrap_or_else(|| "dust-canyon-raiders".to_string());

                    if is_fortified {
                        events.push(SimulationEvent::new(
                            tick,
                            EventType::WarlordIncursion {
                                entity: hostile_id,
                                damage: 0.0,
                                repelled: true,
                            },
                        ));
                    } else {
                        let inventory = self
                            .world
                            .get_component_mut::<Inventory>(&treasury_id)
                            .expect("validated city treasury inventory");
                        let looted_food = inventory.get("food").min(Fixed64::from_int(50));
                        let looted_credits = inventory.get("credits").min(Fixed64::from_int(50));
                        let _ = inventory.try_subtract("food", looted_food);
                        let _ = inventory.try_subtract("credits", looted_credits);
                        events.push(SimulationEvent::new(
                            tick,
                            EventType::WarlordIncursion {
                                entity: hostile_id,
                                damage: (looted_food + looted_credits).to_f64_lossy(),
                                repelled: false,
                            },
                        ));
                    }
                }
            }
        }

        if tick_val == 100 || tick_val == 350 || tick_val == 650 {
            let inventory = self
                .world
                .get_component_mut::<Inventory>(&treasury_id)
                .expect("validated city treasury inventory");
            if inventory.get("food") >= Fixed64::from_int(50) {
                let count = 25.0;
                inventory.add(&city.population_resource, Fixed64::from_f64_lossy(count));
                events.push(SimulationEvent::new(
                    tick,
                    EventType::RefugeeWaveArrived {
                        origin: "border-marches".to_string(),
                        count,
                    },
                ));
            }
        }

        *self
            .world
            .get_component_mut::<CityRuntimeState>(&treasury_id)
            .expect("validated city state") = state;

        Ok(events)
    }

    fn run_city_ecology(&mut self, tick: Tick) -> Result<Vec<SimulationEvent>, WorldForgeError> {
        let (Some(city), Some(treasury_id)) = (self.city_config.clone(), self.city_treasury_id)
        else {
            return Ok(Vec::new());
        };
        let mut events = Vec::new();
        let tick_val = tick.value();

        // 1. Seasonal progression (every 60 ticks)
        if tick_val > 0 && tick_val.is_multiple_of(60) {
            let cycle = (tick_val / 240) + 1;
            let season_idx = (tick_val % 240) / 60;
            let season = match season_idx {
                0 => "Spring",
                1 => "Summer",
                2 => "Autumn",
                _ => "Winter",
            };
            events.push(SimulationEvent::new(
                tick,
                EventType::SeasonChanged {
                    season: season.to_string(),
                    cycle,
                },
            ));
        }

        // 2. Weather shift (every 30 ticks)
        if tick_val > 0 && tick_val.is_multiple_of(30) {
            let season_idx = (tick_val % 240) / 60;
            let weather_seed = self.scenario.seed.wrapping_add(tick_val / 30);
            let weather_choice = weather_seed % 4;
            let (weather, temp) = match season_idx {
                0 => match weather_choice {
                    0 => ("Gentle Rain", 14.5),
                    1 => ("Clear Skies", 18.0),
                    2 => ("Spring Showers", 15.0),
                    _ => ("Mild Breeze", 17.5),
                },
                1 => match weather_choice {
                    0 => ("Scorching Sun", 34.0),
                    1 => ("Heatwave", 37.5),
                    2 => ("Thunderstorm", 28.0),
                    _ => ("Clear Skies", 31.0),
                },
                2 => match weather_choice {
                    0 => ("Crisp Autumn Wind", 16.0),
                    1 => ("Overcast Skies", 14.0),
                    2 => ("Cool Rain", 12.5),
                    _ => ("Harvest Sun", 18.0),
                },
                _ => match weather_choice {
                    0 => ("Light Snow", 0.5),
                    1 => ("Freezing Frost", -4.0),
                    2 => ("Winter Blizzard", -2.5),
                    _ => ("Cold Clear", 1.0),
                },
            };
            events.push(SimulationEvent::new(
                tick,
                EventType::WeatherChanged {
                    weather: weather.to_string(),
                    temperature: temp,
                },
            ));
        }

        // 3. Ecological disaster check
        if tick_val > 0 && tick_val.is_multiple_of(100) {
            let inventory = self
                .world
                .get_component_mut::<Inventory>(&treasury_id)
                .expect("validated city inventory");
            let season_idx = (tick_val % 240) / 60;
            if season_idx == 1 && inventory.get("clean_water") < Fixed64::from_int(40) {
                events.push(SimulationEvent::new(
                    tick,
                    EventType::EcologicalDisaster {
                        disaster: "Groundwater Drought".to_string(),
                        severity: 65.0,
                    },
                ));
            } else if season_idx == 3 && inventory.get("power") < Fixed64::from_int(30) {
                events.push(SimulationEvent::new(
                    tick,
                    EventType::EcologicalDisaster {
                        disaster: "Grid Freeze Shock".to_string(),
                        severity: 50.0,
                    },
                ));
            }
        }

        // 4. Logistics & trade caravan
        if tick_val > 0 && tick_val.is_multiple_of(40) {
            let state = self
                .world
                .get_component::<CityRuntimeState>(&treasury_id)
                .cloned()
                .ok_or_else(|| action_error("city state is unavailable"))?;
            let friendly_entity = city.political_entities.iter().find(|e| {
                let stance = state
                    .entity_stances
                    .get(&e.id)
                    .map(String::as_str)
                    .unwrap_or(e.initial_stance.as_str());
                stance == "vassal" || stance == "friendly" || stance == "coalition"
            });
            if let Some(partner) = friendly_entity {
                let inventory = self
                    .world
                    .get_component_mut::<Inventory>(&treasury_id)
                    .expect("validated city inventory");
                let bonus_credits = Fixed64::from_int(20);
                inventory.add("credits", bonus_credits);
                events.push(SimulationEvent::new(
                    tick,
                    EventType::TradeCaravanArrived {
                        source: partner.name.clone(),
                        resource: "credits".to_string(),
                        amount: bonus_credits.to_f64_lossy(),
                    },
                ));
            }
        }

        Ok(events)
    }

    fn run_city_intrigue(&mut self, tick: Tick) -> Result<Vec<SimulationEvent>, WorldForgeError> {
        let (Some(city), Some(treasury_id)) = (self.city_config.clone(), self.city_treasury_id)
        else {
            return Ok(Vec::new());
        };
        if city.corporations.is_empty()
            && city.cyber_agents.is_empty()
            && city.crypto_assets.is_empty()
        {
            return Ok(Vec::new());
        }
        let mut events = Vec::new();
        let tick_value = tick.value();
        let mut state = self
            .world
            .get_component::<CityRuntimeState>(&treasury_id)
            .cloned()
            .ok_or_else(|| action_error("city intrigue state is unavailable"))?;

        state.intrigue_heat = (state.intrigue_heat - Fixed64::from_ratio(1, 2)).max(Fixed64::ZERO);

        if tick_value > 0 && tick_value.is_multiple_of(5) {
            for asset in &city.crypto_assets {
                let price = state
                    .crypto_prices
                    .entry(asset.id.clone())
                    .or_insert(Fixed64::from_f64_lossy(asset.initial_price));
                let old_price = *price;
                let mut rng = DeterministicRng::new(
                    self.scenario.seed,
                    &format!("crypto-market/{}/{tick_value}", asset.id),
                );
                let centered = rng.next_fixed() * Fixed64::from_int(2) - Fixed64::ONE;
                let movement = centered * Fixed64::from_f64_lossy(asset.volatility * 0.18);
                *price = (*price * (Fixed64::ONE + movement)).max(Fixed64::from_ratio(1, 100));
                events.push(SimulationEvent::new(
                    tick,
                    EventType::CryptoMarketMoved {
                        asset: asset.id.clone(),
                        old_price,
                        new_price: *price,
                        cause: "market-cycle".to_string(),
                    },
                ));
            }
        }

        if tick_value > 0 && tick_value.is_multiple_of(10) {
            let rogue_agents = city
                .cyber_agents
                .iter()
                .filter(|agent| {
                    state
                        .agent_status
                        .get(&agent.id)
                        .map(String::as_str)
                        .unwrap_or(agent.initial_status.as_str())
                        == "rogue"
                })
                .cloned()
                .collect::<Vec<_>>();
            for agent in rogue_agents {
                let resource = if tick_value.is_multiple_of(20) {
                    "research"
                } else {
                    "credits"
                };
                let requested = Fixed64::from_f64_lossy((agent.skill * 0.22).max(5.0));
                let inventory = self
                    .world
                    .get_component_mut::<Inventory>(&treasury_id)
                    .expect("validated city inventory");
                let damage = inventory.get(resource).min(requested);
                let _ = inventory.try_subtract(resource, damage);
                state.intrigue_heat =
                    (state.intrigue_heat + Fixed64::from_int(8)).min(Fixed64::from_int(100));
                events.push(SimulationEvent::new(
                    tick,
                    EventType::RogueAgentIncident {
                        agent: agent.id,
                        resource: resource.to_string(),
                        damage: damage.to_f64_lossy(),
                    },
                ));
            }
        }

        *self
            .world
            .get_component_mut::<CityRuntimeState>(&treasury_id)
            .expect("validated city state") = state;
        Ok(events)
    }

    /// Let corporations and cyber agents pursue their own deterministic
    /// incentives. Their choices are abstract strategy-game actions: every
    /// outcome is stateful, replayable, and included in the proof chain.
    fn run_autonomous_actors(
        &mut self,
        tick: Tick,
    ) -> Result<Vec<SimulationEvent>, WorldForgeError> {
        let (Some(city), Some(treasury_id)) = (self.city_config.clone(), self.city_treasury_id)
        else {
            return Ok(Vec::new());
        };
        if tick.value() == 0 || !tick.value().is_multiple_of(20) {
            return Ok(Vec::new());
        }

        let mut state = self
            .world
            .get_component::<CityRuntimeState>(&treasury_id)
            .cloned()
            .ok_or_else(|| action_error("autonomous actor state is unavailable"))?;
        let mut events = Vec::new();
        let mut trajectory_impacts: Vec<(String, Vec<(&'static str, i32)>)> = Vec::new();

        for corporation in &city.corporations {
            let mut observations = BTreeMap::new();
            observations.insert(
                "security".to_string(),
                state
                    .corporation_security
                    .get(&corporation.id)
                    .copied()
                    .unwrap_or_else(|| Fixed64::from_f64_lossy(corporation.security)),
            );
            observations.insert(
                "exposure".to_string(),
                state
                    .corporation_exposure
                    .get(&corporation.id)
                    .copied()
                    .unwrap_or(Fixed64::ZERO),
            );
            observations.insert("heat".to_string(), state.intrigue_heat);
            observations.insert(
                "influence".to_string(),
                Fixed64::from_f64_lossy(corporation.influence),
            );
            let context = AgentContext {
                tick: tick.value(),
                observations,
            };
            let mut policy = UtilityAgent::new(&format!("corporation/{}", corporation.id));
            policy.add_option(UtilityOption {
                action: autonomous_action("harden-network", Some("security")),
                score_fn: corporate_hardening_utility,
            });
            policy.add_option(UtilityOption {
                action: autonomous_action("cover-tracks", Some("exposure")),
                score_fn: corporate_cover_utility,
            });
            policy.add_option(UtilityOption {
                action: autonomous_action("lobby-council", Some("governance")),
                score_fn: corporate_lobby_utility,
            });
            let mut rng = DeterministicRng::new(
                self.scenario.seed,
                &format!("autonomous/corporation/{}/{}", corporation.id, tick.value()),
            );
            if let Some(decision) = policy.decide(&context, &mut rng).into_iter().next() {
                let rationale = match decision.action_type.as_str() {
                    "harden-network" => {
                        let security = state
                            .corporation_security
                            .entry(corporation.id.clone())
                            .or_insert_with(|| Fixed64::from_f64_lossy(corporation.security));
                        *security = (*security + Fixed64::from_int(4)).min(Fixed64::from_int(100));
                        trajectory_impacts.push((
                            format!("autonomous/{}/hardening", corporation.id),
                            vec![("innovation", 1), ("risk", -1)],
                        ));
                        "security deficit outweighed political opportunity"
                    }
                    "cover-tracks" => {
                        let exposure = state
                            .corporation_exposure
                            .entry(corporation.id.clone())
                            .or_insert(Fixed64::ZERO);
                        *exposure = (*exposure - Fixed64::from_int(6)).max(Fixed64::ZERO);
                        state.intrigue_heat =
                            (state.intrigue_heat - Fixed64::ONE).max(Fixed64::ZERO);
                        trajectory_impacts.push((
                            format!("autonomous/{}/cover", corporation.id),
                            vec![("risk", -2), ("cohesion", -1)],
                        ));
                        "public exposure threatened the corporation's freedom to act"
                    }
                    _ => {
                        trajectory_impacts.push((
                            format!("autonomous/{}/lobby", corporation.id),
                            vec![("prosperity", 2), ("cohesion", -1), ("risk", 1)],
                        ));
                        "influence offered the highest expected strategic return"
                    }
                };
                events.push(SimulationEvent::new(
                    tick,
                    EventType::AutonomousActorDecision {
                        actor: corporation.id.clone(),
                        action: decision.action_type,
                        target: decision.target,
                        rationale: rationale.to_string(),
                    },
                ));
            }
        }

        for agent in &city.cyber_agents {
            let status = state
                .agent_status
                .get(&agent.id)
                .map(String::as_str)
                .unwrap_or(agent.initial_status.as_str());
            let loyalty = state
                .agent_loyalty
                .get(&agent.id)
                .copied()
                .unwrap_or_else(|| Fixed64::from_f64_lossy(agent.loyalty));
            let mut observations = BTreeMap::new();
            observations.insert("skill".to_string(), Fixed64::from_f64_lossy(agent.skill));
            observations.insert("loyalty".to_string(), loyalty);
            observations.insert(
                "containment".to_string(),
                Fixed64::from_f64_lossy(agent.containment),
            );
            observations.insert("heat".to_string(), state.intrigue_heat);
            observations.insert(
                "rogue".to_string(),
                if status == "rogue" {
                    Fixed64::from_int(100)
                } else {
                    Fixed64::ZERO
                },
            );
            let context = AgentContext {
                tick: tick.value(),
                observations,
            };
            let mut policy = UtilityAgent::new(&format!("cyber-agent/{}", agent.id));
            policy.add_option(UtilityOption {
                action: autonomous_action("stabilize", Some("civic-network")),
                score_fn: agent_stability_utility,
            });
            policy.add_option(UtilityOption {
                action: autonomous_action("test-boundaries", Some("sandbox")),
                score_fn: agent_autonomy_utility,
            });
            let mut rng = DeterministicRng::new(
                self.scenario.seed,
                &format!("autonomous/agent/{}/{}", agent.id, tick.value()),
            );
            if let Some(decision) = policy.decide(&context, &mut rng).into_iter().next() {
                let rationale = if decision.action_type == "test-boundaries" {
                    let updated_loyalty = (loyalty - Fixed64::from_int(2)).max(Fixed64::ZERO);
                    state
                        .agent_loyalty
                        .insert(agent.id.clone(), updated_loyalty);
                    state.intrigue_heat =
                        (state.intrigue_heat + Fixed64::from_int(3)).min(Fixed64::from_int(100));
                    trajectory_impacts.push((
                        format!("autonomous/{}/boundary-test", agent.id),
                        vec![("innovation", 2), ("risk", 4), ("cohesion", -1)],
                    ));
                    "capability and autonomy pressure exceeded loyalty and containment"
                } else {
                    state.agent_loyalty.insert(
                        agent.id.clone(),
                        (loyalty + Fixed64::ONE).min(Fixed64::from_int(100)),
                    );
                    state.intrigue_heat =
                        (state.intrigue_heat - Fixed64::from_ratio(1, 2)).max(Fixed64::ZERO);
                    trajectory_impacts.push((
                        format!("autonomous/{}/stabilize", agent.id),
                        vec![("cohesion", 1), ("risk", -1)],
                    ));
                    "loyalty and containment favored cooperative maintenance"
                };
                events.push(SimulationEvent::new(
                    tick,
                    EventType::AutonomousActorDecision {
                        actor: agent.id.clone(),
                        action: decision.action_type,
                        target: decision.target,
                        rationale: rationale.to_string(),
                    },
                ));
            }
        }

        *self
            .world
            .get_component_mut::<CityRuntimeState>(&treasury_id)
            .expect("validated city state") = state;
        for (cause, impacts) in trajectory_impacts {
            events.extend(self.apply_system_trajectory_impacts(
                treasury_id,
                &impacts,
                &cause,
                tick,
            )?);
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

fn autonomous_action(action_type: &str, target: Option<&str>) -> AgentAction {
    AgentAction {
        action_type: action_type.to_string(),
        target: target.map(str::to_string),
        params: BTreeMap::new(),
    }
}

fn build_systems_debrief(
    state: &CityRuntimeState,
    trajectory: &CityTrajectoryProgress,
    wellbeing: f64,
    employment_rate: f64,
) -> SystemsDebriefProgress {
    let headline = trajectory
        .dominant_axis
        .as_ref()
        .and_then(|axis| trajectory.scores.get(axis).map(|score| (axis, score)))
        .map_or_else(
            || "The city has not committed to a dominant path yet.".to_string(),
            |(axis, score)| {
                if *score >= 0.0 {
                    format!(
                        "{} is now the dominant path ({score:+.1}); feedback is reshaping connected systems.",
                        humanize_axis(axis)
                    )
                } else {
                    format!(
                        "Erosion of {} dominates the city ({score:+.1}); recovery will require counter-pressure.",
                        humanize_axis(axis)
                    )
                }
            },
        );

    let mut active_feedback_loops = trajectory
        .momentum
        .iter()
        .filter_map(|(axis, momentum)| {
            let score = trajectory.scores.get(axis).copied().unwrap_or(0.0);
            (momentum.abs() >= 0.05).then(|| FeedbackLoopProgress {
                axis: axis.clone(),
                score,
                momentum: *momentum,
                direction: if score == 0.0 || score.signum() == momentum.signum() {
                    "reinforcing".to_string()
                } else {
                    "balancing".to_string()
                },
                consequence: trajectory_consequence(axis, score),
            })
        })
        .collect::<Vec<_>>();
    active_feedback_loops.sort_by(|left, right| {
        right
            .momentum
            .abs()
            .partial_cmp(&left.momentum.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let score = |axis: &str| trajectory.scores.get(axis).copied().unwrap_or(0.0);
    let mut warnings = Vec::new();
    if score("risk") > 40.0 {
        warnings.push(
            "Risk has crossed 40: intrigue heat and cascading disruption can now amplify each other."
                .to_string(),
        );
    }
    if score("cohesion") < -25.0 {
        warnings.push(
            "Cohesion is below -25: faction trust is becoming a system constraint, not a cosmetic score."
                .to_string(),
        );
    }
    if score("sustainability") < -25.0 {
        warnings.push(
            "Ecological debt is accumulating and will weaken food, water, and power output."
                .to_string(),
        );
    }
    if employment_rate < 0.7 {
        warnings.push(format!(
            "Employment is only {:.0}%: population growth can outpace useful work and tax capacity.",
            employment_rate * 100.0
        ));
    }
    if wellbeing < 45.0 {
        warnings.push(
            "Wellbeing is below 45: growth without social capacity is producing fragility."
                .to_string(),
        );
    }
    if state.intrigue_heat > Fixed64::from_int(50) {
        warnings.push(
            "Shadow-network heat is above 50: corporate and rogue-agent reactions are increasingly costly."
                .to_string(),
        );
    }

    let mut leverage_points = Vec::new();
    if score("risk") > 20.0 {
        leverage_points.push(
            "Reduce risk momentum through containment, counter-intelligence, diplomacy, or lower-volatility civic choices."
                .to_string(),
        );
    }
    if score("cohesion") < 0.0 {
        leverage_points.push(
            "A cohesion-positive council choice changes both immediate legitimacy and the long-run feedback direction."
                .to_string(),
        );
    }
    if score("sustainability") < 0.0 {
        leverage_points.push(
            "Regenerative buildings and resource research attack the shared food-water-power bottleneck."
                .to_string(),
        );
    }
    if score("innovation") < 15.0 {
        leverage_points.push(
            "Research capacity is a force multiplier: innovation unlocks technologies that alter several loops at once."
                .to_string(),
        );
    }
    if leverage_points.is_empty() {
        leverage_points.push(
            "Protect the current trajectory by diversifying resources; a single shortage can create a balancing shock."
                .to_string(),
        );
    }

    SystemsDebriefProgress {
        headline,
        active_feedback_loops,
        warnings,
        leverage_points,
    }
}

fn humanize_axis(axis: &str) -> String {
    let mut value = axis.replace(['-', '_'], " ");
    if let Some(first) = value.get_mut(0..1) {
        first.make_ascii_uppercase();
    }
    value
}

fn trajectory_consequence(axis: &str, score: f64) -> String {
    let positive = score >= 0.0;
    match (axis, positive) {
        ("innovation", true) => "Research output is receiving a compounding bonus.",
        ("innovation", false) => "Research output is being suppressed by institutional drag.",
        ("prosperity", true) => "Credit generation is strengthening future purchasing power.",
        ("prosperity", false) => "Credit generation is weakening, narrowing future options.",
        ("sustainability", true) => "Food, water, and power systems are becoming more productive.",
        ("sustainability", false) => {
            "Food, water, and power systems are losing efficiency together."
        }
        ("cohesion", true) => "Wellbeing and population stability are reinforcing each other.",
        ("cohesion", false) => "Wellbeing and faction support are exposed to continued erosion.",
        ("sovereignty", true) => "Materials and security capacity are gaining strategic leverage.",
        ("sovereignty", false) => {
            "Materials and security capacity are increasingly externally constrained."
        }
        ("risk", true) => "Broad output penalties and intrigue pressure grow as risk compounds.",
        ("risk", false) => "Lower systemic risk is removing friction across the economy.",
        _ => "This trajectory is changing linked city systems over time.",
    }
    .to_string()
}

fn agent_observation(context: &AgentContext, key: &str) -> Fixed64 {
    context
        .observations
        .get(key)
        .copied()
        .unwrap_or(Fixed64::ZERO)
}

fn corporate_hardening_utility(context: &AgentContext) -> Fixed64 {
    Fixed64::from_int(100) - agent_observation(context, "security")
        + agent_observation(context, "exposure") / Fixed64::from_int(4)
}

fn corporate_cover_utility(context: &AgentContext) -> Fixed64 {
    agent_observation(context, "exposure")
        + agent_observation(context, "heat") / Fixed64::from_int(3)
}

fn corporate_lobby_utility(context: &AgentContext) -> Fixed64 {
    agent_observation(context, "influence")
        - agent_observation(context, "exposure") / Fixed64::from_int(4)
}

fn agent_stability_utility(context: &AgentContext) -> Fixed64 {
    agent_observation(context, "loyalty") + agent_observation(context, "containment")
        - agent_observation(context, "heat")
}

fn agent_autonomy_utility(context: &AgentContext) -> Fixed64 {
    agent_observation(context, "skill")
        + (Fixed64::from_int(100) - agent_observation(context, "loyalty"))
        + agent_observation(context, "heat") / Fixed64::from_int(2)
        + agent_observation(context, "rogue")
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
    for option in chosen_civic_options(city, state) {
        if let Some(value) = option.effects.building_multipliers.get(&building.id) {
            multiplier *= *value;
        }
        if let Some(value) = option.effects.resource_multipliers.get(resource) {
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
    multiplier *= trajectory_resource_multiplier(state, resource);
    multiplier.clamp(0.1, 100.0)
}

fn trajectory_resource_multiplier(state: &CityRuntimeState, resource: &str) -> f64 {
    let score = |axis: &str| {
        state
            .trajectory_scores
            .get(axis)
            .copied()
            .unwrap_or(Fixed64::ZERO)
            .to_f64_lossy()
    };
    let mut multiplier = 1.0;
    match resource {
        "research" => multiplier *= 1.0 + score("innovation") / 250.0,
        "credits" => multiplier *= 1.0 + score("prosperity") / 250.0,
        "food" | "water" | "power" => {
            multiplier *= 1.0 + score("sustainability") / 300.0;
        }
        "happiness" | "wellbeing" => multiplier *= 1.0 + score("cohesion") / 250.0,
        "materials" | "security" => multiplier *= 1.0 + score("sovereignty") / 300.0,
        _ => {}
    }
    multiplier *= 1.0 - score("risk").max(0.0) / 400.0;
    multiplier.clamp(0.5, 1.6)
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
    for option in chosen_civic_options(city, state) {
        if option.effects.housing_multiplier > 0.0 {
            housing_multiplier *= option.effects.housing_multiplier;
        }
        if option.effects.jobs_multiplier > 0.0 {
            jobs_multiplier *= option.effects.jobs_multiplier;
        }
        wellbeing += option.effects.wellbeing_bonus;
    }
    (
        (housing as f64 * housing_multiplier).round().max(0.0) as u64,
        (jobs as f64 * jobs_multiplier).round().max(0.0) as u64,
        wellbeing,
    )
}

fn chosen_civic_options<'a>(
    city: &'a CityConfig,
    state: &CityRuntimeState,
) -> Vec<&'a CivicOption> {
    state
        .decisions
        .iter()
        .filter_map(|(dilemma_id, option_id)| {
            city.dilemmas
                .iter()
                .find(|dilemma| dilemma.id == *dilemma_id)?
                .options
                .iter()
                .find(|option| option.id == *option_id)
        })
        .collect()
}

fn civic_population_growth_multiplier(city: &CityConfig, state: &CityRuntimeState) -> f64 {
    let policy_multiplier = chosen_civic_options(city, state)
        .into_iter()
        .filter_map(|option| {
            (option.effects.population_growth_multiplier > 0.0)
                .then_some(option.effects.population_growth_multiplier)
        })
        .fold(1.0, |total, multiplier| total * multiplier);
    let score = |axis: &str| {
        state
            .trajectory_scores
            .get(axis)
            .copied()
            .unwrap_or(Fixed64::ZERO)
            .to_f64_lossy()
    };
    let trajectory_multiplier = 1.0 + score("cohesion") / 400.0 + score("sustainability") / 500.0
        - score("risk").max(0.0) / 350.0;
    (policy_multiplier * trajectory_multiplier).clamp(0.1, 100.0)
}

fn civic_trigger_met(
    _city: &CityConfig,
    state: &CityRuntimeState,
    inventory: &Inventory,
    dilemma: &CivicDilemma,
    tick: u64,
) -> bool {
    tick >= dilemma.trigger.tick
        && dilemma
            .trigger
            .resource_below
            .iter()
            .all(|(resource, threshold)| {
                inventory.get(resource) < Fixed64::from_f64_lossy(*threshold)
            })
        && dilemma
            .trigger
            .resource_above
            .iter()
            .all(|(resource, threshold)| {
                inventory.get(resource) > Fixed64::from_f64_lossy(*threshold)
            })
        && dilemma
            .trigger
            .trajectory_below
            .iter()
            .all(|(axis, threshold)| {
                state
                    .trajectory_scores
                    .get(axis)
                    .copied()
                    .unwrap_or(Fixed64::ZERO)
                    < Fixed64::from_f64_lossy(*threshold)
            })
        && dilemma
            .trigger
            .trajectory_above
            .iter()
            .all(|(axis, threshold)| {
                state
                    .trajectory_scores
                    .get(axis)
                    .copied()
                    .unwrap_or(Fixed64::ZERO)
                    > Fixed64::from_f64_lossy(*threshold)
            })
        && dilemma
            .trigger
            .requires_technologies
            .iter()
            .all(|technology| state.unlocked_technologies.contains(technology))
        && dilemma
            .trigger
            .requires_choices
            .iter()
            .all(|(required, option)| state.decisions.get(required) == Some(option))
        && dilemma
            .trigger
            .excludes_choices
            .iter()
            .all(|(excluded, option)| state.decisions.get(excluded) != Some(option))
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

    #[test]
    fn civic_choices_branch_city_rules_and_remain_deterministic() {
        let play = || {
            let mut runtime =
                SimulationRuntime::load(&example("micro-city"), 314, Some(30)).unwrap();
            let opened = runtime.step(5).unwrap();
            assert_eq!(
                opened.city.as_ref().unwrap().governance.pending_dilemmas[0].id,
                "growth-charter"
            );

            let decided = runtime
                .make_civic_decision("growth-charter", "civic-land-trust")
                .unwrap();
            let governance = &decided.city.as_ref().unwrap().governance;
            assert!(governance.pending_dilemmas.is_empty());
            assert_eq!(governance.decisions[0].option, "civic-land-trust");
            assert_eq!(
                governance
                    .factions
                    .iter()
                    .find(|faction| faction.id == "commons-assembly")
                    .unwrap()
                    .support,
                70.0
            );

            runtime
                .construct_building("courtyard-homes", "civic-core")
                .unwrap();
            let branched = runtime.step(8).unwrap();
            let city = branched.city.as_ref().unwrap();
            assert_eq!(city.housing, 106);
            assert!(city
                .governance
                .pending_dilemmas
                .iter()
                .any(|dilemma| dilemma.id == "commons-mandate"));
            assert!(!city
                .governance
                .pending_dilemmas
                .iter()
                .any(|dilemma| dilemma.id == "automation-compact"));

            runtime.step(100).unwrap();
            let result = runtime.completed_result().unwrap();
            assert!(result.event_type_counts.governance >= 3);
            (
                result.final_state_fingerprint,
                result.proof.event_chain_root,
            )
        };

        assert_eq!(play(), play());
    }

    #[test]
    fn civic_choices_create_compounding_divergent_trajectories() {
        let mut commons = SimulationRuntime::load(&example("micro-city"), 2026, Some(80)).unwrap();
        let mut market = SimulationRuntime::load(&example("micro-city"), 2026, Some(80)).unwrap();
        commons.step(5).unwrap();
        market.step(5).unwrap();
        let commons_choice = commons
            .make_civic_decision("growth-charter", "civic-land-trust")
            .unwrap();
        let market_choice = market
            .make_civic_decision("growth-charter", "open-development")
            .unwrap();

        let commons_trajectory = &commons_choice.city.as_ref().unwrap().trajectory;
        let market_trajectory = &market_choice.city.as_ref().unwrap().trajectory;
        assert!(commons_trajectory.scores["cohesion"] > 0.0);
        assert!(market_trajectory.scores["cohesion"] < 0.0);
        assert!(market_trajectory.scores["prosperity"] > 0.0);
        let recorded_choice = &commons_choice.city.as_ref().unwrap().governance.decisions[0];
        assert_eq!(recorded_choice.trajectory["cohesion"], 16.0);
        assert_eq!(
            recorded_choice.counterfactuals[0].option,
            "open-development"
        );
        assert!(commons_choice
            .city
            .as_ref()
            .unwrap()
            .systems_debrief
            .active_feedback_loops
            .iter()
            .any(|feedback| feedback.axis == "cohesion"));

        let initial_commons_cohesion = commons_trajectory.scores["cohesion"];
        // Momentum compounds before any later civic dilemma can add legitimate
        // counter-pressure to the same axis.
        let commons_compounded = commons.step(6).unwrap();
        assert!(
            commons_compounded.city.as_ref().unwrap().trajectory.scores["cohesion"]
                > initial_commons_cohesion
        );
        assert!(commons_compounded
            .recent_events
            .iter()
            .any(|event| matches!(event.event_type, EventType::TrajectoryShifted { .. })));

        // Later events may bend either path, but the histories remain causally
        // distinct and therefore produce different deterministic states.
        let commons_later = commons.step(24).unwrap();
        let market_later = market.step(30).unwrap();
        assert_ne!(
            commons_later.state_fingerprint,
            market_later.state_fingerprint
        );
    }

    #[test]
    fn manifest_mods_execute_inside_the_deterministic_event_chain() {
        let directory =
            std::env::temp_dir().join(format!("worldforge-runtime-mod-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            directory.join("world.toml"),
            "name = \"mod-test\"\nversion = \"0.1.0\"\nmods = [\"pulse.wat\"]\n",
        )
        .unwrap();
        std::fs::write(
            directory.join("scenario.toml"),
            "world = \"mod-test\"\nduration_ticks = 2\n",
        )
        .unwrap();
        std::fs::write(
            directory.join("entities.toml"),
            "[[entities]]\nname = \"observer\"\nentity_type = \"system\"\nregion = \"test\"\n",
        )
        .unwrap();
        std::fs::write(
            directory.join("pulse.mod.toml"),
            "capabilities = [\"world.event.emit\"]\n",
        )
        .unwrap();
        std::fs::write(
            directory.join("pulse.wat"),
            r#"(module
                (import "worldforge" "emit_event" (func $emit (param i64)))
                (func (export "init"))
                (func (export "on_event") (param i64))
                (func (export "on_tick") (param $tick i64)
                    local.get $tick
                    i64.const 77
                    i64.add
                    call $emit))"#,
        )
        .unwrap();

        let play = || {
            let mut runtime = SimulationRuntime::load(&directory, 41, None).unwrap();
            runtime.run().unwrap()
        };
        let first = play();
        let second = play();
        assert_eq!(first.event_type_counts.mods, 2);
        assert_eq!(first.proof.event_chain_root, second.proof.event_chain_root);
        assert_eq!(
            first.final_state_fingerprint,
            second.final_state_fingerprint
        );

        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn manifest_mods_cannot_escape_the_world_directory() {
        let directory = std::env::temp_dir().join(format!(
            "worldforge-runtime-mod-escape-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            directory.join("world.toml"),
            "name = \"mod-escape-test\"\nversion = \"0.1.0\"\nmods = [\"../escape.wasm\"]\n",
        )
        .unwrap();
        std::fs::write(
            directory.join("scenario.toml"),
            "world = \"mod-escape-test\"\nduration_ticks = 1\n",
        )
        .unwrap();
        std::fs::write(directory.join("entities.toml"), "entities = []\n").unwrap();

        let error = match SimulationRuntime::load(&directory, 1, None) {
            Ok(_) => panic!("path-traversing mod reference must be rejected"),
            Err(error) => error,
        };
        assert!(matches!(
            error.code,
            ErrorCode::WorldSchemaViolation | ErrorCode::ModLoadFailed
        ));

        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn autonomous_actors_make_replayable_stateful_decisions() {
        let play = || {
            let mut runtime =
                SimulationRuntime::load(&example("micro-city"), 2026, Some(21)).unwrap();
            let result = runtime.run().unwrap();
            let decisions = runtime
                .retained_events()
                .iter()
                .filter(|event| {
                    matches!(event.event_type, EventType::AutonomousActorDecision { .. })
                })
                .count();
            (result, decisions)
        };
        let first = play();
        let second = play();
        assert_eq!(
            first.0.final_state_fingerprint,
            second.0.final_state_fingerprint
        );
        assert_eq!(
            first.0.proof.event_chain_root,
            second.0.proof.event_chain_root
        );
        assert!(first.0.event_type_counts.intrigue >= 6);
        assert_eq!(first.1, 6);
        assert_eq!(first.1, second.1);
    }

    #[test]
    fn civic_deadlines_apply_the_declared_default() {
        let mut runtime = SimulationRuntime::load(&example("micro-city"), 8, Some(20)).unwrap();
        let progress = runtime.step(17).unwrap();
        let governance = &progress.city.as_ref().unwrap().governance;
        assert!(governance
            .decisions
            .iter()
            .any(|decision| decision.dilemma == "growth-charter"
                && decision.option == "open-development"));
        runtime.step(100).unwrap();
        assert!(runtime.retained_events().iter().any(|event| matches!(
            &event.event_type,
            EventType::CivicDecisionResolved { dilemma, option }
                if dilemma == "growth-charter" && option == "open-development"
        )));
    }

    #[test]
    fn geopolitical_actions_are_recorded_and_deterministic() {
        let play = || {
            let mut runtime =
                SimulationRuntime::load(&example("micro-city"), 42, Some(50)).unwrap();
            runtime.step(5).unwrap();

            // Toggle defense posture
            runtime
                .execute_geopolitical_action("posture", "defense-garrison", Some("fortified"))
                .unwrap();

            let progress = runtime.current_progress();
            if let Some(city) = &progress.city {
                if let Some(geo) = &city.geopolitics {
                    assert_eq!(geo.defense_posture, "fortified");
                }
            }

            runtime.step(45).unwrap();
            let result = runtime.completed_result().unwrap();
            (
                result.final_state_fingerprint,
                result.proof.event_chain_root,
            )
        };

        assert_eq!(play(), play());
    }

    #[test]
    fn intrigue_agents_secrets_and_markets_are_deterministic() {
        let play = || {
            let mut runtime =
                SimulationRuntime::load(&example("micro-city"), 404, Some(24)).unwrap();
            let initial = runtime.current_progress();
            let intrigue = initial.city.as_ref().unwrap().intrigue.as_ref().unwrap();
            assert_eq!(intrigue.corporations.len(), 3);
            assert!(intrigue
                .agents
                .iter()
                .any(|agent| agent.id == "nyx-seven" && agent.status == "rogue"));

            runtime
                .execute_intrigue_action("deploy", "glass-orchid", None, None)
                .unwrap();
            runtime
                .execute_intrigue_action(
                    "infiltrate",
                    "helix-meridian",
                    Some("cipher-nine"),
                    Some("mycelial-compute"),
                )
                .unwrap();
            runtime
                .execute_intrigue_action("trade", "forgecoin", None, Some("buy"))
                .unwrap();
            runtime
                .execute_intrigue_action(
                    "manipulate",
                    "forgecoin",
                    Some("glass-orchid"),
                    Some("pump"),
                )
                .unwrap();
            runtime.step(11).unwrap();
            runtime
                .execute_intrigue_action("contain", "nyx-seven", None, None)
                .unwrap();
            runtime
                .execute_intrigue_action("trade", "forgecoin", None, Some("sell"))
                .unwrap();
            runtime.step(100).unwrap();

            let result = runtime.completed_result().unwrap();
            assert!(result.event_type_counts.intrigue >= 10);
            assert!(runtime
                .retained_events()
                .iter()
                .any(|event| matches!(event.event_type, EventType::RogueAgentIncident { .. })));
            (
                result.final_state_fingerprint,
                result.proof.event_chain_root,
            )
        };
        assert_eq!(play(), play());
    }

    #[test]
    fn rejected_intrigue_actions_are_atomic() {
        let mut runtime = SimulationRuntime::load(&example("micro-city"), 2026, Some(24)).unwrap();
        let before_fingerprint = runtime.world.fingerprint();
        let before_events = runtime.event_count;
        let before_progress =
            serde_json::to_value(runtime.current_progress()).expect("progress serializes");

        let error = runtime
            .execute_intrigue_action("deploy", "cipher-nine", None, None)
            .unwrap_err();

        assert!(error.message.contains("contained or compromised"));
        assert_eq!(runtime.world.fingerprint(), before_fingerprint);
        assert_eq!(runtime.event_count, before_events);
        assert_eq!(
            serde_json::to_value(runtime.current_progress()).expect("progress serializes"),
            before_progress
        );
    }

    #[test]
    fn city_upgrade_and_demolish_are_deterministic() {
        let play = || {
            let mut runtime =
                SimulationRuntime::load(&example("micro-city"), 777, Some(120)).unwrap();
            // Construct two maker-cooperatives in old-grid
            runtime
                .construct_building("maker-cooperative", "old-grid")
                .unwrap();
            runtime
                .construct_building("maker-cooperative", "old-grid")
                .unwrap();
            let p1 = runtime.current_progress();
            let city1 = p1.city.as_ref().unwrap();
            let b1 = city1
                .buildings
                .iter()
                .find(|b| b.id == "maker-cooperative")
                .unwrap();
            assert_eq!(b1.level, 1);
            assert_eq!(b1.count, 2);

            // Upgrade it to level 2
            runtime.upgrade_city_building("maker-cooperative").unwrap();
            let p2 = runtime.current_progress();
            let city2 = p2.city.as_ref().unwrap();
            let b2 = city2
                .buildings
                .iter()
                .find(|b| b.id == "maker-cooperative")
                .unwrap();
            assert_eq!(b2.level, 2);

            // Step through a season change (ticks 0..65)
            runtime.step(65).unwrap();
            let p3 = runtime.current_progress();
            let city3 = p3.city.as_ref().unwrap();
            let eco = city3
                .ecology
                .as_ref()
                .expect("ecology present in micro-city");
            assert_eq!(eco.season, "Summer"); // tick 65 is Summer (cycle 0, season 1)
            assert!(!eco.weather.is_empty());

            // Demolish one maker-cooperative
            runtime
                .demolish_city_building("maker-cooperative", "old-grid")
                .unwrap();
            let p4 = runtime.current_progress();
            let city4 = p4.city.as_ref().unwrap();
            let b4 = city4
                .buildings
                .iter()
                .find(|b| b.id == "maker-cooperative")
                .unwrap();
            assert_eq!(b4.count, 1);

            runtime.step(60).unwrap();
            let result = runtime.completed_result().unwrap();
            (
                result.final_state_fingerprint,
                result.proof.event_chain_root,
            )
        };

        assert_eq!(play(), play());
    }
}

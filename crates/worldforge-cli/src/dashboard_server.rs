//! Small, dependency-free localhost dashboard server backed by the real engine.

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{sync_channel, TrySendError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use worldforge_core::{EngineVersion, ErrorCode, WorldForgeError};
use worldforge_runtime::{RunProgress, SimulationRuntime};
use worldforge_world::{
    CityConfig, EntitiesConfig, EntityConfig, EventType, LinkConfig, Objective, ObjectiveStatus,
    ObjectiveType, ProductionConfig, Scenario, ScheduledEvent, ScheduledEventType, SimulationEvent,
    WorldManifest,
};

const INDEX: &str = include_str!("../../../dashboard/index.html");
const SCRIPT: &str = include_str!("../../../dashboard/dashboard.js");
const STYLE: &str = include_str!("../../../dashboard/dashboard.css");
const WORLD_ATLAS: &[u8] = include_bytes!("../../../dashboard/assets/world-atlas.png");
const TACTICAL_HOLO_BG: &[u8] = include_bytes!("../../../dashboard/assets/tactical-holo-bg.jpg");
const CLEAN_TACTICAL_BLUEPRINT: &[u8] =
    include_bytes!("../../../dashboard/assets/clean-tactical-blueprint.jpg");
const MAX_REQUEST_BYTES: usize = 64 * 1024;
const MAX_TICKS: u64 = 1_000_000;
const MAX_BENCHMARK_REPS: u32 = 20;
const MAX_PLAY_SESSIONS: usize = 32;
const MAX_STEP_TICKS: u64 = 5_000;
const MAX_DASHBOARD_EVENTS: usize = 5_000;
const SESSION_TTL: Duration = Duration::from_secs(4 * 60 * 60);
const MAX_SERVER_WORKERS: usize = 16;
const REQUEST_QUEUE_CAPACITY: usize = 128;
const MAX_SAVE_SLOTS: usize = 100;
const MAX_SAVE_BYTES: u64 = 2 * 1024 * 1024;
const MAX_SAVE_INTERVENTIONS: usize = 50_000;
const MAX_WORLD_PACKAGES: usize = 200;

type SharedPlaySession = Arc<Mutex<PlaySession>>;
type SessionRegistry = BTreeMap<String, SharedPlaySession>;

struct ServerState {
    worlds_dir: PathBuf,
    saves_dir: PathBuf,
    sessions: Mutex<SessionRegistry>,
    save_io: Mutex<()>,
    world_io: Mutex<()>,
}

struct PlaySession {
    world: String,
    seed: u64,
    total_ticks: u64,
    interventions: Vec<InterventionRecord>,
    runtime: SimulationRuntime,
    last_access: Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InterventionRecord {
    tick: u64,
    #[serde(default = "default_capacity_action")]
    action: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    entity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    capacity: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    building: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    district: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    technology: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    dilemma: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    option: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    geo_action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    intrigue_action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    target: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    agent: Option<String>,
}

#[derive(Deserialize)]
struct GeopoliticsRequest {
    action: String,
    entity: String,
    #[serde(default)]
    option: Option<String>,
}

#[derive(Deserialize)]
struct IntrigueRequest {
    action: String,
    target: String,
    #[serde(default)]
    agent: Option<String>,
    #[serde(default)]
    option: Option<String>,
}

fn default_capacity_action() -> String {
    "capacity".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveGame {
    format_version: u32,
    id: String,
    name: String,
    world: String,
    world_fingerprint: String,
    seed: u64,
    total_ticks: u64,
    current_tick: u64,
    interventions: Vec<InterventionRecord>,
    created_at: u64,
    updated_at: u64,
}

#[derive(Deserialize)]
struct RunRequest {
    world: String,
    seed: u64,
    ticks: u64,
}

#[derive(Deserialize)]
struct BenchmarkRequest {
    world: String,
    ticks: u64,
    #[serde(default = "default_benchmark_reps")]
    reps: u32,
}

#[derive(Deserialize)]
struct AnalyzeRequest {
    world: String,
    ticks: u64,
    #[serde(default = "default_analyze_seed")]
    seed: u64,
    #[serde(default = "default_analyze_runs")]
    runs: usize,
}

fn default_analyze_seed() -> u64 {
    42
}

fn default_analyze_runs() -> usize {
    20
}

#[derive(Deserialize)]
struct StepRequest {
    #[serde(default = "default_step_ticks")]
    ticks: u64,
}

#[derive(Deserialize)]
struct InterventionRequest {
    entity: String,
    capacity: f64,
}

#[derive(Deserialize)]
struct ConstructRequest {
    building: String,
    district: String,
}

#[derive(Deserialize)]
struct ResearchRequest {
    technology: String,
}

#[derive(Deserialize)]
struct DecisionRequest {
    dilemma: String,
    option: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveRequest {
    session_id: String,
    name: String,
    #[serde(default)]
    save_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorldBuildRequest {
    id: String,
    title: String,
    #[serde(default)]
    description: String,
    template: String,
    difficulty: String,
    seed: u64,
    ticks: u64,
}

fn default_step_ticks() -> u64 {
    1
}

fn default_benchmark_reps() -> u32 {
    5
}

pub fn serve(bind: &str, worlds_dir: &Path, saves_dir: &Path) -> Result<(), WorldForgeError> {
    if !worlds_dir.is_dir() {
        return Err(WorldForgeError::new(
            ErrorCode::WorldNotFound,
            format!("world catalog does not exist: {}", worlds_dir.display()),
        ));
    }
    let worlds_dir = std::fs::canonicalize(worlds_dir).map_err(io_error)?;
    std::fs::create_dir_all(saves_dir).map_err(io_error)?;
    let saves_dir = std::fs::canonicalize(saves_dir).map_err(io_error)?;
    recover_save_directory(&saves_dir)?;
    let listener = TcpListener::bind(bind).map_err(|error| {
        WorldForgeError::new(
            ErrorCode::RuntimeInitFailed,
            format!("cannot bind dashboard to {bind}: {error}"),
        )
    })?;
    println!("World Forge dashboard: http://{bind}");
    println!("World catalog: {}", worlds_dir.display());
    println!("Save slots: {}", saves_dir.display());
    println!("Press Ctrl+C to stop.");

    let state = Arc::new(ServerState {
        worlds_dir,
        saves_dir,
        sessions: Mutex::new(BTreeMap::new()),
        save_io: Mutex::new(()),
        world_io: Mutex::new(()),
    });
    let worker_count = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(4)
        .clamp(2, MAX_SERVER_WORKERS);
    let (sender, receiver) = sync_channel::<TcpStream>(REQUEST_QUEUE_CAPACITY);
    let receiver = Arc::new(Mutex::new(receiver));
    for _ in 0..worker_count {
        let state = Arc::clone(&state);
        let receiver = Arc::clone(&receiver);
        std::thread::spawn(move || loop {
            let stream = match receiver.lock() {
                Ok(receiver) => receiver.recv(),
                Err(_) => return,
            };
            let Ok(mut stream) = stream else { return };
            if let Err(error) = handle_connection(&mut stream, &state) {
                let _ = respond_error(&mut stream, status_for_error(&error), &error);
            }
        });
    }
    for connection in listener.incoming() {
        match connection {
            Ok(stream) => match sender.try_send(stream) {
                Ok(()) => {}
                Err(TrySendError::Full(mut stream)) => {
                    let _ = respond_json(
                        &mut stream,
                        "503 Service Unavailable",
                        &json!({
                            "error": "server is at capacity; retry shortly",
                            "code": "SERVER_BUSY"
                        }),
                    );
                }
                Err(TrySendError::Disconnected(_)) => {
                    return Err(WorldForgeError::new(
                        ErrorCode::InternalError,
                        "dashboard worker pool stopped unexpectedly",
                    ));
                }
            },
            Err(error) => eprintln!("dashboard connection failed: {error}"),
        }
    }
    Ok(())
}

fn handle_connection(stream: &mut TcpStream, state: &ServerState) -> Result<(), WorldForgeError> {
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(io_error)?;
    let request = read_request(stream)?;
    let header_end = request
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| invalid_request("request headers are incomplete"))?;
    let header = String::from_utf8_lossy(&request[..header_end]);
    let request_line = header.lines().next().unwrap_or_default();
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let raw_route = parts.next().unwrap_or_default();
    let route = raw_route.split('?').next().unwrap_or(raw_route);
    let body = &request[header_end + 4..];

    match (method, route) {
        ("GET", "/") | ("GET", "/index.html") => {
            respond(stream, "200 OK", "text/html; charset=utf-8", INDEX.as_bytes(), false)
        }
        ("GET", "/dashboard.js") => respond(
            stream,
            "200 OK",
            "text/javascript; charset=utf-8",
            SCRIPT.as_bytes(),
            false,
        ),
        ("GET", "/dashboard.css") => respond(
            stream,
            "200 OK",
            "text/css; charset=utf-8",
            STYLE.as_bytes(),
            false,
        ),
        ("GET", "/assets/world-atlas.png") => respond(
            stream,
            "200 OK",
            "image/png",
            WORLD_ATLAS,
            true,
        ),
        ("GET", "/assets/tactical-holo-bg.jpg") => respond(
            stream,
            "200 OK",
            "image/jpeg",
            TACTICAL_HOLO_BG,
            true,
        ),
        ("GET", "/assets/clean-tactical-blueprint.jpg") => respond(
            stream,
            "200 OK",
            "image/jpeg",
            CLEAN_TACTICAL_BLUEPRINT,
            true,
        ),
        ("GET", "/api/health") => {
            let world_count = catalog(&state.worlds_dir)?.as_array().map_or(0, Vec::len);
            let active_sessions = lock_sessions(state)?.len();
            respond_json(
                stream,
                "200 OK",
                &json!({
                    "status": "healthy",
                    "engineVersion": EngineVersion::current().to_string(),
                    "worlds": world_count,
                    "activeSessions": active_sessions,
                }),
            )
        }
        ("GET", "/api/worlds") => {
            let worlds = catalog(&state.worlds_dir)?;
            respond_json(stream, "200 OK", &json!({ "worlds": worlds }))
        }
        ("POST", "/api/worlds") => {
            require_json_content_type(&header)?;
            let request: WorldBuildRequest = parse_json(body)?;
            let world = create_world(state, request)?;
            respond_json(stream, "201 Created", &json!({ "world": world }))
        }
        ("POST", "/api/run") => {
            require_json_content_type(&header)?;
            let run: RunRequest = parse_json(body)?;
            validate_ticks(run.ticks)?;
            let world_path = resolve_world(&state.worlds_dir, &run.world)?;
            let document = super::commands::simulation_export_for_dashboard(
                &world_path,
                run.ticks,
                run.seed,
                MAX_DASHBOARD_EVENTS,
            )?;
            respond_json(stream, "200 OK", &document)
        }
        ("POST", "/api/benchmark") => {
            require_json_content_type(&header)?;
            let request: BenchmarkRequest = parse_json(body)?;
            validate_ticks(request.ticks)?;
            if request.reps == 0 || request.reps > MAX_BENCHMARK_REPS {
                return Err(invalid_request(format!(
                    "reps must be between 1 and {MAX_BENCHMARK_REPS}"
                )));
            }
            let world_path = resolve_world(&state.worlds_dir, &request.world)?;
            let report = benchmark_report(&world_path, &request)?;
            respond_json(stream, "200 OK", &report)
        }
        ("POST", "/api/analyze") => {
            require_json_content_type(&header)?;
            let request: AnalyzeRequest = parse_json(body)?;
            validate_ticks(request.ticks)?;
            let runs = request.runs.clamp(1, 100);
            let world_path = resolve_world(&state.worlds_dir, &request.world)?;
            let report = worldforge_runtime::run_monte_carlo(&world_path, &request.world, request.ticks, request.seed, runs)?;
            respond_json(stream, "200 OK", &serde_json::to_value(&report).map_err(|e| WorldForgeError::new(worldforge_core::ErrorCode::InternalError, e.to_string()))?)
        }
        ("POST", "/api/play/sessions") => {
            require_json_content_type(&header)?;
            let request: RunRequest = parse_json(body)?;
            validate_ticks(request.ticks)?;
            let response = create_play_session(state, request)?;
            respond_json(stream, "201 Created", &response)
        }
        ("GET", route) if play_session_id(route).is_some() => {
            let id = play_session_id(route).unwrap_or_default();
            let response = inspect_play_session(state, id)?;
            respond_json(stream, "200 OK", &response)
        }
        ("POST", route) if route.ends_with("/step") && play_action_id(route, "step").is_some() => {
            require_json_content_type(&header)?;
            let request: StepRequest = parse_json(body)?;
            if request.ticks == 0 || request.ticks > MAX_STEP_TICKS {
                return Err(invalid_request(format!(
                    "step ticks must be between 1 and {MAX_STEP_TICKS}"
                )));
            }
            let id = play_action_id(route, "step").unwrap_or_default();
            let response = step_play_session(state, id, request.ticks)?;
            respond_json(stream, "200 OK", &response)
        }
        ("POST", route)
            if route.ends_with("/intervene") && play_action_id(route, "intervene").is_some() =>
        {
            require_json_content_type(&header)?;
            let request: InterventionRequest = parse_json(body)?;
            let id = play_action_id(route, "intervene").unwrap_or_default();
            let response = intervene_play_session(state, id, request)?;
            respond_json(stream, "200 OK", &response)
        }
        ("POST", route)
            if route.ends_with("/construct") && play_action_id(route, "construct").is_some() =>
        {
            require_json_content_type(&header)?;
            let request: ConstructRequest = parse_json(body)?;
            let id = play_action_id(route, "construct").unwrap_or_default();
            let response = construct_in_play_session(state, id, request)?;
            respond_json(stream, "200 OK", &response)
        }
        ("POST", route)
            if route.ends_with("/research") && play_action_id(route, "research").is_some() =>
        {
            require_json_content_type(&header)?;
            let request: ResearchRequest = parse_json(body)?;
            let id = play_action_id(route, "research").unwrap_or_default();
            let response = research_in_play_session(state, id, request)?;
            respond_json(stream, "200 OK", &response)
        }
        ("POST", route)
            if route.ends_with("/decide") && play_action_id(route, "decide").is_some() =>
        {
            require_json_content_type(&header)?;
            let request: DecisionRequest = parse_json(body)?;
            let id = play_action_id(route, "decide").unwrap_or_default();
            let response = decide_in_play_session(state, id, request)?;
            respond_json(stream, "200 OK", &response)
        }
        ("POST", route)
            if route.ends_with("/geopolitics")
                && play_action_id(route, "geopolitics").is_some() =>
        {
            require_json_content_type(&header)?;
            let request: GeopoliticsRequest = parse_json(body)?;
            let id = play_action_id(route, "geopolitics").unwrap_or_default();
            let response = geopolitics_in_play_session(state, id, request)?;
            respond_json(stream, "200 OK", &response)
        }
        ("POST", route)
            if route.ends_with("/intrigue") && play_action_id(route, "intrigue").is_some() =>
        {
            require_json_content_type(&header)?;
            let request: IntrigueRequest = parse_json(body)?;
            let id = play_action_id(route, "intrigue").unwrap_or_default();
            let response = intrigue_in_play_session(state, id, request)?;
            respond_json(stream, "200 OK", &response)
        }
        ("GET", route)
            if route.ends_with("/replay") && play_action_id(route, "replay").is_some() =>
        {
            let id = play_action_id(route, "replay").unwrap_or_default();
            let (filename, replay) = play_replay(state, id)?;
            respond_download(stream, &filename, "application/cbor", &replay)
        }
        ("DELETE", route) if play_session_id(route).is_some() => {
            let id = play_session_id(route).unwrap_or_default();
            delete_play_session(state, id)?;
            respond(stream, "204 No Content", "text/plain", &[], false)
        }
        ("GET", "/api/saves") => {
            let saves = list_saves(state)?;
            respond_json(stream, "200 OK", &json!({ "saves": saves }))
        }
        ("POST", "/api/saves") => {
            require_json_content_type(&header)?;
            let request: SaveRequest = parse_json(body)?;
            let save = save_play_session(state, request)?;
            respond_json(stream, "201 Created", &json!({ "save": save }))
        }
        ("POST", route) if save_action_id(route, "load").is_some() => {
            let id = save_action_id(route, "load").unwrap_or_default();
            let response = load_save(state, id)?;
            respond_json(stream, "201 Created", &response)
        }
        ("DELETE", route) if save_id(route).is_some() => {
            let id = save_id(route).unwrap_or_default();
            delete_save(state, id)?;
            respond(stream, "204 No Content", "text/plain", &[], false)
        }
        ("OPTIONS", route) if route.starts_with("/api/") => {
            respond(stream, "204 No Content", "text/plain", &[], false)
        }
        (_, route) if route.starts_with("/api/") => respond_json(
            stream,
            "404 Not Found",
            &json!({ "error": "API route not found", "code": "NOT_FOUND" }),
        ),
        _ => respond(
            stream,
            "404 Not Found",
            "text/html; charset=utf-8",
            br#"<!doctype html><title>Not found</title><h1>404</h1><p>That page does not exist.</p>"#,
            false,
        ),
    }
}

fn play_session_id(route: &str) -> Option<&str> {
    let id = route.strip_prefix("/api/play/sessions/")?;
    (!id.is_empty() && !id.contains('/')).then_some(id)
}

fn play_action_id<'a>(route: &'a str, action: &str) -> Option<&'a str> {
    let prefix = "/api/play/sessions/";
    let suffix = format!("/{action}");
    let middle = route.strip_prefix(prefix)?.strip_suffix(&suffix)?;
    (!middle.is_empty() && !middle.contains('/')).then_some(middle)
}

fn save_id(route: &str) -> Option<&str> {
    let id = route.strip_prefix("/api/saves/")?;
    valid_save_id(id).then_some(id)
}

fn save_action_id<'a>(route: &'a str, action: &str) -> Option<&'a str> {
    let suffix = format!("/{action}");
    let id = route.strip_prefix("/api/saves/")?.strip_suffix(&suffix)?;
    valid_save_id(id).then_some(id)
}

fn valid_save_id(id: &str) -> bool {
    id.len() == 32 && id.chars().all(|character| character.is_ascii_hexdigit())
}

fn lock_sessions(
    state: &ServerState,
) -> Result<std::sync::MutexGuard<'_, SessionRegistry>, WorldForgeError> {
    state.sessions.lock().map_err(|_| {
        WorldForgeError::new(
            ErrorCode::InternalError,
            "play session registry is unavailable",
        )
    })
}

fn create_play_session(state: &ServerState, request: RunRequest) -> Result<Value, WorldForgeError> {
    let world_path = resolve_world(&state.worlds_dir, &request.world)?;
    let runtime = SimulationRuntime::load(&world_path, request.seed, Some(request.ticks))?;
    let progress = runtime.current_progress();
    let session = PlaySession {
        world: request.world.clone(),
        seed: request.seed,
        total_ticks: request.ticks,
        interventions: Vec::new(),
        runtime,
        last_access: Instant::now(),
    };
    let id = register_session(state, session)?;
    Ok(play_document(&id, &request.world, &progress, None, true))
}

fn register_session(state: &ServerState, session: PlaySession) -> Result<String, WorldForgeError> {
    let id = uuid::Uuid::new_v4().simple().to_string();
    let mut sessions = lock_sessions(state)?;
    sessions.retain(|_, session| {
        session
            .lock()
            .is_ok_and(|session| session.last_access.elapsed() < SESSION_TTL)
    });
    if sessions.len() >= MAX_PLAY_SESSIONS {
        return Err(WorldForgeError::new(
            ErrorCode::RuntimeInitFailed,
            format!("at most {MAX_PLAY_SESSIONS} play sessions may be active"),
        ));
    }
    sessions.insert(id.clone(), Arc::new(Mutex::new(session)));
    Ok(id)
}

fn inspect_play_session(state: &ServerState, id: &str) -> Result<Value, WorldForgeError> {
    with_play_session(state, id, |session| {
        let progress = session.runtime.current_progress();
        Ok(play_document(
            id,
            &session.world,
            &progress,
            session.runtime.replay(),
            true,
        ))
    })
}

fn step_play_session(state: &ServerState, id: &str, ticks: u64) -> Result<Value, WorldForgeError> {
    with_play_session(state, id, |session| {
        let progress = session.runtime.step(ticks)?;
        Ok(play_document(
            id,
            &session.world,
            &progress,
            session.runtime.replay(),
            false,
        ))
    })
}

fn intervene_play_session(
    state: &ServerState,
    id: &str,
    request: InterventionRequest,
) -> Result<Value, WorldForgeError> {
    with_play_session(state, id, |session| {
        let tick = session.runtime.current_progress().current_tick;
        let progress = session
            .runtime
            .set_capacity(&request.entity, request.capacity)?;
        session.interventions.push(InterventionRecord {
            tick,
            action: "capacity".to_string(),
            entity: Some(request.entity),
            capacity: Some(request.capacity),
            building: None,
            district: None,
            technology: None,
            dilemma: None,
            option: None,
            geo_action: None,
            intrigue_action: None,
            target: None,
            agent: None,
        });
        Ok(play_document(
            id,
            &session.world,
            &progress,
            session.runtime.replay(),
            false,
        ))
    })
}

fn construct_in_play_session(
    state: &ServerState,
    id: &str,
    request: ConstructRequest,
) -> Result<Value, WorldForgeError> {
    with_play_session(state, id, |session| {
        let tick = session.runtime.current_progress().current_tick;
        let progress = session
            .runtime
            .construct_building(&request.building, &request.district)?;
        session.interventions.push(InterventionRecord {
            tick,
            action: "construct".to_string(),
            entity: None,
            capacity: None,
            building: Some(request.building),
            district: Some(request.district),
            technology: None,
            dilemma: None,
            option: None,
            geo_action: None,
            intrigue_action: None,
            target: None,
            agent: None,
        });
        Ok(play_document(
            id,
            &session.world,
            &progress,
            session.runtime.replay(),
            false,
        ))
    })
}

fn research_in_play_session(
    state: &ServerState,
    id: &str,
    request: ResearchRequest,
) -> Result<Value, WorldForgeError> {
    with_play_session(state, id, |session| {
        let tick = session.runtime.current_progress().current_tick;
        let progress = session.runtime.research_technology(&request.technology)?;
        session.interventions.push(InterventionRecord {
            tick,
            action: "research".to_string(),
            entity: None,
            capacity: None,
            building: None,
            district: None,
            technology: Some(request.technology),
            dilemma: None,
            option: None,
            geo_action: None,
            intrigue_action: None,
            target: None,
            agent: None,
        });
        Ok(play_document(
            id,
            &session.world,
            &progress,
            session.runtime.replay(),
            false,
        ))
    })
}

fn decide_in_play_session(
    state: &ServerState,
    id: &str,
    request: DecisionRequest,
) -> Result<Value, WorldForgeError> {
    with_play_session(state, id, |session| {
        let tick = session.runtime.current_progress().current_tick;
        let progress = session
            .runtime
            .make_civic_decision(&request.dilemma, &request.option)?;
        session.interventions.push(InterventionRecord {
            tick,
            action: "decision".to_string(),
            entity: None,
            capacity: None,
            building: None,
            district: None,
            technology: None,
            dilemma: Some(request.dilemma),
            option: Some(request.option),
            geo_action: None,
            intrigue_action: None,
            target: None,
            agent: None,
        });
        Ok(play_document(
            id,
            &session.world,
            &progress,
            session.runtime.replay(),
            false,
        ))
    })
}

fn geopolitics_in_play_session(
    state: &ServerState,
    id: &str,
    request: GeopoliticsRequest,
) -> Result<Value, WorldForgeError> {
    with_play_session(state, id, |session| {
        let tick = session.runtime.current_progress().current_tick;
        let progress = session.runtime.execute_geopolitical_action(
            &request.action,
            &request.entity,
            request.option.as_deref(),
        )?;
        session.interventions.push(InterventionRecord {
            tick,
            action: "geopolitics".to_string(),
            entity: Some(request.entity),
            capacity: None,
            building: None,
            district: None,
            technology: None,
            dilemma: None,
            option: request.option,
            geo_action: Some(request.action),
            intrigue_action: None,
            target: None,
            agent: None,
        });
        Ok(play_document(
            id,
            &session.world,
            &progress,
            session.runtime.replay(),
            false,
        ))
    })
}

fn intrigue_in_play_session(
    state: &ServerState,
    id: &str,
    request: IntrigueRequest,
) -> Result<Value, WorldForgeError> {
    with_play_session(state, id, |session| {
        let tick = session.runtime.current_progress().current_tick;
        let progress = session.runtime.execute_intrigue_action(
            &request.action,
            &request.target,
            request.agent.as_deref(),
            request.option.as_deref(),
        )?;
        session.interventions.push(InterventionRecord {
            tick,
            action: "intrigue".to_string(),
            entity: None,
            capacity: None,
            building: None,
            district: None,
            technology: None,
            dilemma: None,
            option: request.option,
            geo_action: None,
            intrigue_action: Some(request.action),
            target: Some(request.target),
            agent: request.agent,
        });
        Ok(play_document(
            id,
            &session.world,
            &progress,
            session.runtime.replay(),
            false,
        ))
    })
}

fn with_play_session<T>(
    state: &ServerState,
    id: &str,
    operation: impl FnOnce(&mut PlaySession) -> Result<T, WorldForgeError>,
) -> Result<T, WorldForgeError> {
    let session = lock_sessions(state)?.get(id).cloned().ok_or_else(|| {
        WorldForgeError::new(ErrorCode::WorldNotFound, "play session does not exist")
    })?;
    let mut session = session.lock().map_err(|_| {
        WorldForgeError::new(ErrorCode::InternalError, "play session is unavailable")
    })?;
    if session.last_access.elapsed() >= SESSION_TTL {
        return Err(WorldForgeError::new(
            ErrorCode::WorldNotFound,
            "play session has expired",
        ));
    }
    session.last_access = Instant::now();
    operation(&mut session)
}

fn delete_play_session(state: &ServerState, id: &str) -> Result<(), WorldForgeError> {
    if lock_sessions(state)?.remove(id).is_none() {
        return Err(WorldForgeError::new(
            ErrorCode::WorldNotFound,
            "play session does not exist",
        ));
    }
    Ok(())
}

fn play_replay(state: &ServerState, id: &str) -> Result<(String, Vec<u8>), WorldForgeError> {
    with_play_session(state, id, |session| {
        let replay = session.runtime.replay().ok_or_else(|| {
            WorldForgeError::new(
                ErrorCode::RuntimeStateMismatch,
                "replay is available only after the session completes",
            )
        })?;
        Ok((
            format!("{}-{}.replay", session.world, &id[..8]),
            replay.to_cbor(),
        ))
    })
}

fn save_play_session(
    state: &ServerState,
    request: SaveRequest,
) -> Result<SaveGame, WorldForgeError> {
    let name = request.name.trim();
    validate_save_name(name)?;
    if request
        .save_id
        .as_deref()
        .is_some_and(|id| !valid_save_id(id))
    {
        return Err(invalid_request("save id is invalid"));
    }
    let now = unix_timestamp()?;
    let requested_overwrite = request.save_id.is_some();
    let id = request
        .save_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().simple().to_string());
    let _save_guard = lock_save_io(state)?;
    let existing = if save_path(state, &id).is_file() {
        Some(read_save_unlocked(state, &id)?)
    } else {
        None
    };
    if existing.is_none() && requested_overwrite {
        return Err(WorldForgeError::new(
            ErrorCode::WorldNotFound,
            "save slot does not exist",
        ));
    }
    if existing.is_none() && save_slot_count(&state.saves_dir)? >= MAX_SAVE_SLOTS {
        return Err(WorldForgeError::new(
            ErrorCode::RuntimeInitFailed,
            format!("at most {MAX_SAVE_SLOTS} save slots may be stored"),
        ));
    }

    let save = with_play_session(state, &request.session_id, |session| {
        let progress = session.runtime.current_progress();
        let fingerprint =
            worldforge_package::fingerprint_resolved_world(&state.worlds_dir.join(&session.world))?;
        Ok(SaveGame {
            format_version: 1,
            id: id.clone(),
            name: name.to_string(),
            world: session.world.clone(),
            world_fingerprint: fingerprint.to_string(),
            seed: session.seed,
            total_ticks: session.total_ticks,
            current_tick: progress.current_tick,
            interventions: session.interventions.clone(),
            created_at: existing.as_ref().map_or(now, |save| save.created_at),
            updated_at: now,
        })
    })?;
    let encoded = serde_json::to_vec_pretty(&save)
        .map_err(|error| WorldForgeError::new(ErrorCode::InternalError, error.to_string()))?;
    atomic_replace(&save_path(state, &id), &encoded)?;
    Ok(save)
}

fn list_saves(state: &ServerState) -> Result<Vec<SaveGame>, WorldForgeError> {
    let _save_guard = lock_save_io(state)?;
    let mut saves = std::fs::read_dir(&state.saves_dir)
        .map_err(io_error)?
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "json")
        })
        .filter_map(|entry| {
            let id = entry.path().file_stem()?.to_str()?.to_string();
            read_save_unlocked(state, &id).ok()
        })
        .collect::<Vec<_>>();
    saves.sort_by_key(|save| std::cmp::Reverse(save.updated_at));
    Ok(saves)
}

fn load_save(state: &ServerState, id: &str) -> Result<Value, WorldForgeError> {
    let save = read_save(state, id)?;
    if save.format_version != 1 {
        return Err(WorldForgeError::new(
            ErrorCode::ReplayVersionIncompatible,
            format!("save format {} is not supported", save.format_version),
        ));
    }
    validate_ticks(save.total_ticks)?;
    if save.current_tick > save.total_ticks {
        return Err(WorldForgeError::new(
            ErrorCode::ReplayFormatInvalid,
            "save tick exceeds its configured duration",
        ));
    }
    let world_path = resolve_world(&state.worlds_dir, &save.world)?;
    let current_fingerprint = worldforge_package::fingerprint_resolved_world(&world_path)?;
    if current_fingerprint.to_string() != save.world_fingerprint {
        return Err(WorldForgeError::new(
            ErrorCode::ReplayFingerprintMismatch,
            "world content changed after this save was created",
        ));
    }
    let mut runtime = SimulationRuntime::load(&world_path, save.seed, Some(save.total_ticks))?;
    for intervention in &save.interventions {
        let current_tick = runtime.current_progress().current_tick;
        if intervention.tick < current_tick || intervention.tick > save.current_tick {
            return Err(WorldForgeError::new(
                ErrorCode::ReplayFormatInvalid,
                "save interventions are not in deterministic tick order",
            ));
        }
        if intervention.tick > current_tick {
            runtime.step(intervention.tick - current_tick)?;
        }
        match intervention.action.as_str() {
            "capacity" => runtime.set_capacity(
                intervention.entity.as_deref().ok_or_else(|| {
                    WorldForgeError::new(
                        ErrorCode::ReplayFormatInvalid,
                        "capacity intervention is missing its entity",
                    )
                })?,
                intervention.capacity.ok_or_else(|| {
                    WorldForgeError::new(
                        ErrorCode::ReplayFormatInvalid,
                        "capacity intervention is missing its value",
                    )
                })?,
            )?,
            "construct" => runtime.construct_building(
                intervention.building.as_deref().ok_or_else(|| {
                    WorldForgeError::new(
                        ErrorCode::ReplayFormatInvalid,
                        "construction intervention is missing its building",
                    )
                })?,
                intervention.district.as_deref().ok_or_else(|| {
                    WorldForgeError::new(
                        ErrorCode::ReplayFormatInvalid,
                        "construction intervention is missing its district",
                    )
                })?,
            )?,
            "research" => runtime.research_technology(
                intervention.technology.as_deref().ok_or_else(|| {
                    WorldForgeError::new(
                        ErrorCode::ReplayFormatInvalid,
                        "research intervention is missing its technology",
                    )
                })?,
            )?,
            "decision" => runtime.make_civic_decision(
                intervention.dilemma.as_deref().ok_or_else(|| {
                    WorldForgeError::new(
                        ErrorCode::ReplayFormatInvalid,
                        "civic decision is missing its dilemma",
                    )
                })?,
                intervention.option.as_deref().ok_or_else(|| {
                    WorldForgeError::new(
                        ErrorCode::ReplayFormatInvalid,
                        "civic decision is missing its option",
                    )
                })?,
            )?,
            "geopolitics" => runtime.execute_geopolitical_action(
                intervention.geo_action.as_deref().unwrap_or("posture"),
                intervention.entity.as_deref().unwrap_or("defense-garrison"),
                intervention.option.as_deref(),
            )?,
            "intrigue" => runtime.execute_intrigue_action(
                intervention.intrigue_action.as_deref().ok_or_else(|| {
                    WorldForgeError::new(
                        ErrorCode::ReplayFormatInvalid,
                        "intrigue intervention is missing its action",
                    )
                })?,
                intervention.target.as_deref().ok_or_else(|| {
                    WorldForgeError::new(
                        ErrorCode::ReplayFormatInvalid,
                        "intrigue intervention is missing its target",
                    )
                })?,
                intervention.agent.as_deref(),
                intervention.option.as_deref(),
            )?,
            _ => {
                return Err(WorldForgeError::new(
                    ErrorCode::ReplayFormatInvalid,
                    "save contains an unknown intervention action",
                ));
            }
        };
    }
    let current_tick = runtime.current_progress().current_tick;
    if save.current_tick > current_tick {
        runtime.step(save.current_tick - current_tick)?;
    }
    let progress = runtime.current_progress();
    let world = save.world.clone();
    let session = PlaySession {
        world: world.clone(),
        seed: save.seed,
        total_ticks: save.total_ticks,
        interventions: save.interventions,
        runtime,
        last_access: Instant::now(),
    };
    let session_id = register_session(state, session)?;
    let session = lock_sessions(state)?
        .get(&session_id)
        .cloned()
        .ok_or_else(|| WorldForgeError::new(ErrorCode::InternalError, "session unavailable"))?;
    let session = session
        .lock()
        .map_err(|_| WorldForgeError::new(ErrorCode::InternalError, "session unavailable"))?;
    Ok(play_document(
        &session_id,
        &world,
        &progress,
        session.runtime.replay(),
        true,
    ))
}

fn delete_save(state: &ServerState, id: &str) -> Result<(), WorldForgeError> {
    if !valid_save_id(id) {
        return Err(invalid_request("save id is invalid"));
    }
    let _save_guard = lock_save_io(state)?;
    let path = save_path(state, id);
    if !path.is_file() {
        return Err(WorldForgeError::new(
            ErrorCode::WorldNotFound,
            "save slot does not exist",
        ));
    }
    std::fs::remove_file(path).map_err(io_error)
}

fn read_save(state: &ServerState, id: &str) -> Result<SaveGame, WorldForgeError> {
    let _save_guard = lock_save_io(state)?;
    read_save_unlocked(state, id)
}

fn read_save_unlocked(state: &ServerState, id: &str) -> Result<SaveGame, WorldForgeError> {
    if !valid_save_id(id) {
        return Err(invalid_request("save id is invalid"));
    }
    let path = save_path(state, id);
    let metadata = std::fs::metadata(&path).map_err(|error| {
        WorldForgeError::new(
            ErrorCode::WorldNotFound,
            format!("cannot read save slot: {error}"),
        )
    })?;
    if metadata.len() > MAX_SAVE_BYTES {
        return Err(WorldForgeError::new(
            ErrorCode::ReplayFormatInvalid,
            format!("save slot exceeds the {MAX_SAVE_BYTES}-byte limit"),
        ));
    }
    let bytes = std::fs::read(path).map_err(io_error)?;
    let save = serde_json::from_slice(&bytes).map_err(|error| {
        WorldForgeError::new(
            ErrorCode::ReplayFormatInvalid,
            format!("save slot is invalid: {error}"),
        )
    })?;
    validate_save_game(&save, id)?;
    Ok(save)
}

fn save_path(state: &ServerState, id: &str) -> PathBuf {
    state.saves_dir.join(format!("{id}.json"))
}

fn lock_save_io(state: &ServerState) -> Result<std::sync::MutexGuard<'_, ()>, WorldForgeError> {
    state
        .save_io
        .lock()
        .map_err(|_| WorldForgeError::new(ErrorCode::InternalError, "save storage is unavailable"))
}

fn validate_save_name(name: &str) -> Result<(), WorldForgeError> {
    if name.is_empty() || name.len() > 64 || name.chars().any(char::is_control) {
        return Err(invalid_request(
            "save name must contain 1 to 64 printable characters",
        ));
    }
    Ok(())
}

fn validate_save_game(save: &SaveGame, expected_id: &str) -> Result<(), WorldForgeError> {
    if save.format_version != 1 {
        return Err(WorldForgeError::new(
            ErrorCode::ReplayVersionIncompatible,
            format!("save format {} is not supported", save.format_version),
        ));
    }
    if save.id != expected_id || !valid_save_id(&save.id) {
        return Err(WorldForgeError::new(
            ErrorCode::ReplayFormatInvalid,
            "save id does not match its file name",
        ));
    }
    validate_save_name(save.name.trim())?;
    if save.world.is_empty()
        || !save
            .world
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
    {
        return Err(WorldForgeError::new(
            ErrorCode::ReplayFormatInvalid,
            "save contains an invalid world id",
        ));
    }
    if save.world_fingerprint.len() != 64
        || !save
            .world_fingerprint
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err(WorldForgeError::new(
            ErrorCode::ReplayFormatInvalid,
            "save contains an invalid world fingerprint",
        ));
    }
    validate_ticks(save.total_ticks)?;
    if save.current_tick > save.total_ticks || save.created_at > save.updated_at {
        return Err(WorldForgeError::new(
            ErrorCode::ReplayFormatInvalid,
            "save contains an invalid tick or timestamp range",
        ));
    }
    if save.interventions.len() > MAX_SAVE_INTERVENTIONS {
        return Err(WorldForgeError::new(
            ErrorCode::ReplayFormatInvalid,
            format!("save exceeds {MAX_SAVE_INTERVENTIONS} interventions"),
        ));
    }
    let mut previous_tick = 0;
    for (index, intervention) in save.interventions.iter().enumerate() {
        let valid_action = match intervention.action.as_str() {
            "capacity" => {
                intervention.entity.as_deref().is_some_and(valid_action_id)
                    && intervention.capacity.is_some_and(|capacity| {
                        capacity.is_finite() && (0.0..=2.0).contains(&capacity)
                    })
            }
            "construct" => {
                intervention
                    .building
                    .as_deref()
                    .is_some_and(valid_action_id)
                    && intervention
                        .district
                        .as_deref()
                        .is_some_and(valid_action_id)
            }
            "research" => intervention
                .technology
                .as_deref()
                .is_some_and(valid_action_id),
            "decision" => {
                intervention.dilemma.as_deref().is_some_and(valid_action_id)
                    && intervention.option.as_deref().is_some_and(valid_action_id)
            }
            "geopolitics" => intervention.entity.as_deref().is_some_and(valid_action_id),
            "intrigue" => {
                intervention
                    .intrigue_action
                    .as_deref()
                    .is_some_and(valid_action_id)
                    && intervention.target.as_deref().is_some_and(valid_action_id)
                    && intervention.agent.as_deref().is_none_or(valid_action_id)
                    && intervention.option.as_deref().is_none_or(valid_action_id)
            }
            _ => false,
        };
        if intervention.tick > save.current_tick
            || (index > 0 && intervention.tick < previous_tick)
            || !valid_action
        {
            return Err(WorldForgeError::new(
                ErrorCode::ReplayFormatInvalid,
                "save contains an invalid intervention",
            ));
        }
        previous_tick = intervention.tick;
    }
    Ok(())
}

fn valid_action_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && !value.chars().any(char::is_control)
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '-' || character == '_'
        })
}

fn save_slot_count(saves_dir: &Path) -> Result<usize, WorldForgeError> {
    Ok(std::fs::read_dir(saves_dir)
        .map_err(io_error)?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "json"))
        .count())
}

fn atomic_replace(destination: &Path, bytes: &[u8]) -> Result<(), WorldForgeError> {
    if bytes.len() as u64 > MAX_SAVE_BYTES {
        return Err(WorldForgeError::new(
            ErrorCode::RuntimeInitFailed,
            format!("save exceeds the {MAX_SAVE_BYTES}-byte limit"),
        ));
    }
    let parent = destination
        .parent()
        .ok_or_else(|| WorldForgeError::new(ErrorCode::InternalError, "save path has no parent"))?;
    let stem = destination
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| WorldForgeError::new(ErrorCode::InternalError, "save path is invalid"))?;
    let temporary = parent.join(format!(
        ".worldforge-save-{stem}-{}.tmp",
        uuid::Uuid::new_v4().simple()
    ));
    let backup = destination.with_extension("json.bak");

    let write_result = (|| -> Result<(), WorldForgeError> {
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)
            .map_err(io_error)?;
        file.write_all(bytes).map_err(io_error)?;
        file.sync_all().map_err(io_error)?;
        drop(file);

        if destination.exists() {
            if backup.exists() {
                std::fs::remove_file(&backup).map_err(io_error)?;
            }
            std::fs::rename(destination, &backup).map_err(io_error)?;
            if let Err(error) = std::fs::rename(&temporary, destination) {
                let _ = std::fs::rename(&backup, destination);
                return Err(io_error(error));
            }
            // The destination is already durable. A leftover backup is safe
            // and will be cleaned during the next startup recovery pass.
            let _ = std::fs::remove_file(&backup);
        } else {
            std::fs::rename(&temporary, destination).map_err(io_error)?;
        }
        if let Ok(directory) = std::fs::File::open(parent) {
            let _ = directory.sync_all();
        }
        Ok(())
    })();
    if write_result.is_err() && temporary.exists() {
        let _ = std::fs::remove_file(&temporary);
    }
    write_result
}

fn recover_save_directory(saves_dir: &Path) -> Result<(), WorldForgeError> {
    for entry in std::fs::read_dir(saves_dir).map_err(io_error)? {
        let entry = entry.map_err(io_error)?;
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with(".worldforge-save-") && path.extension().is_some_and(|ext| ext == "tmp")
        {
            std::fs::remove_file(path).map_err(io_error)?;
            continue;
        }
        if path.extension().is_some_and(|extension| extension == "bak") {
            let destination = path.with_extension("");
            let is_save_backup = destination
                .extension()
                .is_some_and(|extension| extension == "json")
                && destination
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .is_some_and(valid_save_id);
            if !is_save_backup {
                continue;
            }
            if destination.exists() {
                std::fs::remove_file(path).map_err(io_error)?;
            } else {
                std::fs::rename(path, destination).map_err(io_error)?;
            }
        }
    }
    Ok(())
}

fn unix_timestamp() -> Result<u64, WorldForgeError> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|error| WorldForgeError::new(ErrorCode::InternalError, error.to_string()))
}

fn play_document(
    id: &str,
    world: &str,
    progress: &RunProgress,
    replay: Option<&worldforge_replay::ReplayArtifact>,
    include_topology: bool,
) -> Value {
    let events = progress
        .recent_events
        .iter()
        .map(event_document)
        .collect::<Vec<_>>();
    let proof = replay.map(|replay| {
        json!({
            "world": replay.world_fingerprint.to_string(),
            "scenario": replay.scenario_fingerprint.to_string(),
            "initial": replay.initial_state_fingerprint.to_string(),
            "final": replay.final_state_fingerprint.to_string(),
            "eventChain": replay.event_chain_root.to_string(),
            "runId": replay.run_id,
        })
    });
    json!({
        "sessionId": id,
        "world": world,
        "state": progress.state,
        "seed": progress.seed,
        "currentTick": progress.current_tick,
        "totalTicks": progress.total_ticks,
        "stateFingerprint": progress.state_fingerprint.to_string(),
        "eventCount": progress.event_count,
        "shortageCount": progress.shortage_count,
        "resourceMetrics": progress.resource_metrics,
        "snapshot": progress.snapshot,
        "entityStates": progress.entity_states,
        "city": progress.city,
        "objectives": progress.objective_results.iter().map(|objective| json!({
            "name": objective.name,
            "status": format!("{:?}", objective.status),
            "kind": objective.kind,
            "resource": objective.resource,
            "current": objective.current,
            "target": objective.target,
            "progress": objective.progress,
        })).collect::<Vec<_>>(),
        "recentEvents": events,
        "entities": include_topology.then_some(&progress.entities),
        "links": include_topology.then_some(&progress.links),
        "proof": proof,
        "completed": replay.is_some(),
    })
}

fn event_document(event: &SimulationEvent) -> Value {
    let (event_type, entity, resource, amount) = match &event.event_type {
        EventType::ProductionCompleted {
            entity,
            resource,
            amount,
        } => (
            "production",
            entity.clone(),
            resource.clone(),
            amount.to_f64_lossy(),
        ),
        EventType::ResourceTransferred {
            from,
            to,
            resource,
            amount,
        } => (
            "transfer",
            format!("{from} → {to}"),
            resource.clone(),
            amount.to_f64_lossy(),
        ),
        EventType::InventoryShortage {
            entity,
            resource,
            needed,
            ..
        } => (
            "shortage",
            entity.clone(),
            resource.clone(),
            needed.to_f64_lossy(),
        ),
        EventType::PriceChanged {
            resource,
            new_price,
            ..
        } => (
            "price",
            "market".to_string(),
            resource.clone(),
            new_price.to_f64_lossy(),
        ),
        EventType::CapacityChanged {
            entity,
            new_capacity,
            ..
        } => (
            "system",
            entity.clone(),
            "capacity".to_string(),
            new_capacity.to_f64_lossy(),
        ),
        EventType::PlayerCapacityChanged {
            entity,
            new_capacity,
            ..
        } => (
            "system",
            entity.clone(),
            "capacity".to_string(),
            new_capacity.to_f64_lossy(),
        ),
        EventType::BuildingConstructed {
            building,
            district,
            count,
        } => (
            "construction",
            district.clone(),
            building.clone(),
            f64::from(*count),
        ),
        EventType::TechnologyUnlocked { technology, branch } => {
            ("research", branch.clone(), technology.clone(), 0.0)
        }
        EventType::PopulationChanged { population, .. } => (
            "system",
            "city".to_string(),
            "population".to_string(),
            population.to_f64_lossy(),
        ),
        EventType::CivicDilemmaOpened {
            dilemma,
            deadline_tick,
        } => (
            "governance",
            "council".to_string(),
            dilemma.clone(),
            deadline_tick.map_or(0.0, |tick| tick as f64),
        ),
        EventType::PlayerCivicDecision { dilemma, option }
        | EventType::CivicDecisionResolved { dilemma, option } => {
            ("governance", dilemma.clone(), option.clone(), 1.0)
        }
        EventType::ObjectiveUpdated { objective, .. } => {
            ("system", "objective".to_string(), objective.clone(), 0.0)
        }
        EventType::SimulationDegraded { reason } => {
            ("system", "runtime".to_string(), reason.clone(), 0.0)
        }
        EventType::GeopoliticalStanceChanged { entity, from, to } => {
            ("geopolitics", entity.clone(), format!("{from}→{to}"), 0.0)
        }
        EventType::TributeCollected { entity, resources } => {
            let total = resources.values().sum::<f64>();
            ("geopolitics", entity.clone(), "tribute".to_string(), total)
        }
        EventType::WarlordIncursion {
            entity,
            damage,
            repelled,
        } => {
            let status = if *repelled {
                "repelled".to_string()
            } else {
                "breached".to_string()
            };
            ("geopolitics", entity.clone(), status, *damage)
        }
        EventType::RefugeeWaveArrived { origin, count } => (
            "geopolitics",
            origin.clone(),
            "refugees".to_string(),
            *count,
        ),
        EventType::CovertOperationResolved {
            operation,
            target,
            success,
            ..
        } => (
            "intrigue",
            target.clone(),
            operation.clone(),
            if *success { 1.0 } else { 0.0 },
        ),
        EventType::PlayerIntrigueAction { action, target, .. } => {
            ("intrigue", target.clone(), action.clone(), 1.0)
        }
        EventType::TradeSecretAcquired {
            corporation,
            secret,
            research_value,
        } => (
            "intrigue",
            corporation.clone(),
            secret.clone(),
            *research_value,
        ),
        EventType::CyberAgentStatusChanged { agent, to, .. } => {
            ("intrigue", agent.clone(), to.clone(), 0.0)
        }
        EventType::RogueAgentIncident {
            agent,
            resource,
            damage,
        } => ("intrigue", agent.clone(), resource.clone(), *damage),
        EventType::CryptoMarketMoved {
            asset, new_price, ..
        } => (
            "intrigue",
            asset.clone(),
            "market".to_string(),
            new_price.to_f64_lossy(),
        ),
        EventType::CryptoTradeExecuted {
            asset, side, units, ..
        } => (
            "intrigue",
            asset.clone(),
            side.clone(),
            units.to_f64_lossy(),
        ),
    };
    json!({
        "tick": event.tick.value(),
        "type": event_type,
        "entity": entity,
        "resource": resource,
        "amount": amount,
        "summary": event.summary(),
    })
}

fn create_world(state: &ServerState, request: WorldBuildRequest) -> Result<Value, WorldForgeError> {
    let id = request.id.trim();
    let title = request.title.trim();
    let description = request.description.trim();
    validate_world_build_request(id, title, description, &request)?;
    let _world_guard = state.world_io.lock().map_err(|_| {
        WorldForgeError::new(ErrorCode::InternalError, "world storage is unavailable")
    })?;
    let destination = state.worlds_dir.join(id);
    if destination.exists() {
        return Err(WorldForgeError::new(
            ErrorCode::RuntimeStateMismatch,
            format!("world '{id}' already exists"),
        ));
    }
    let world_count = catalog(&state.worlds_dir)?.as_array().map_or(0, Vec::len);
    if world_count >= MAX_WORLD_PACKAGES {
        return Err(WorldForgeError::new(
            ErrorCode::RuntimeInitFailed,
            format!("at most {MAX_WORLD_PACKAGES} worlds may be stored"),
        ));
    }

    let (manifest, scenario, entities, city) = build_world_package(
        title,
        description,
        &request.template,
        &request.difficulty,
        request.seed,
        request.ticks,
    )?;
    let temporary = state.worlds_dir.join(format!(
        ".worldforge-new-{id}-{}",
        uuid::Uuid::new_v4().simple()
    ));
    std::fs::create_dir(&temporary).map_err(io_error)?;
    let result = (|| -> Result<Value, WorldForgeError> {
        write_toml_file(&temporary.join("world.toml"), &manifest)?;
        write_toml_file(&temporary.join("scenario.toml"), &scenario)?;
        write_toml_file(&temporary.join("entities.toml"), &entities)?;
        if let Some(city) = &city {
            write_toml_file(&temporary.join("city.toml"), city)?;
        }
        worldforge_package::validate_world(&temporary)?;
        std::fs::rename(&temporary, &destination).map_err(|error| {
            if destination.exists() {
                WorldForgeError::new(
                    ErrorCode::RuntimeStateMismatch,
                    format!("world '{id}' was created by another request"),
                )
            } else {
                io_error(error)
            }
        })?;
        if let Ok(directory) = std::fs::File::open(&state.worlds_dir) {
            let _ = directory.sync_all();
        }
        catalog_entry(&destination)?.ok_or_else(|| {
            WorldForgeError::new(
                ErrorCode::InternalError,
                "created world is missing required files",
            )
        })
    })();
    if result.is_err() && temporary.exists() {
        let _ = std::fs::remove_dir_all(&temporary);
    }
    result
}

fn validate_world_build_request(
    id: &str,
    title: &str,
    description: &str,
    request: &WorldBuildRequest,
) -> Result<(), WorldForgeError> {
    if id.len() < 3
        || id.len() > 48
        || id.starts_with('-')
        || id.ends_with('-')
        || id.contains("--")
        || !id.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
    {
        return Err(invalid_request(
            "world id must be 3 to 48 lowercase letters, numbers, or single hyphens",
        ));
    }
    if title.len() < 2 || title.len() > 80 || title.chars().any(char::is_control) {
        return Err(invalid_request(
            "world title must contain 2 to 80 printable characters",
        ));
    }
    if description.len() > 240 || description.chars().any(char::is_control) {
        return Err(invalid_request(
            "world description must contain at most 240 printable characters",
        ));
    }
    if !matches!(
        request.template.as_str(),
        "industrial" | "city" | "ecosystem"
    ) {
        return Err(invalid_request(
            "template must be industrial, city, or ecosystem",
        ));
    }
    if !matches!(request.difficulty.as_str(), "easy" | "standard" | "hard") {
        return Err(invalid_request(
            "difficulty must be easy, standard, or hard",
        ));
    }
    if !(10..=MAX_TICKS).contains(&request.ticks) {
        return Err(invalid_request(format!(
            "ticks must be between 10 and {MAX_TICKS}"
        )));
    }
    Ok(())
}

fn build_world_package(
    title: &str,
    description: &str,
    template: &str,
    difficulty: &str,
    seed: u64,
    ticks: u64,
) -> Result<(WorldManifest, Scenario, EntitiesConfig, Option<CityConfig>), WorldForgeError> {
    let (names, resources, region) = match template {
        "industrial" => (
            ["quarry", "refinery", "assembly", "depot", "market"],
            ["ore", "alloy", "goods", "energy"],
            "forge-district",
        ),
        "city" => (
            [
                "reservoir",
                "waterworks",
                "urban-farm",
                "distribution-center",
                "residential-district",
            ],
            ["water", "clean-water", "food", "power"],
            "metro-core",
        ),
        "ecosystem" => (
            [
                "sunlit-meadow",
                "plant-colony",
                "herbivore-herd",
                "habitat",
                "predator-pack",
            ],
            ["sunlight", "biomass", "prey", "water"],
            "living-basin",
        ),
        _ => return Err(invalid_request("unknown world template")),
    };
    let (stock, demand, disruption, objective_minimum) = match difficulty {
        "easy" => (1.5, 2.0, 0.8, 20.0),
        "standard" => (1.0, 3.0, 0.55, 30.0),
        "hard" => (0.7, 4.0, 0.3, 40.0),
        _ => return Err(invalid_request("unknown difficulty")),
    };
    let [raw, processed, final_resource, energy] = resources;
    let energy_supply = ticks as f64 * 2.0 + 100.0;
    let mut entities = vec![
        EntityConfig {
            name: names[0].to_string(),
            entity_type: "source".to_string(),
            region: region.to_string(),
            initial_inventory: resource_amounts([(raw, 120.0 * stock), (energy, energy_supply)]),
            production: Some(ProductionConfig {
                inputs: resource_amounts([(energy, 1.0)]),
                outputs: resource_amounts([(raw, 10.0)]),
                energy_cost: 1.0,
            }),
        },
        EntityConfig {
            name: names[1].to_string(),
            entity_type: "processor".to_string(),
            region: region.to_string(),
            initial_inventory: resource_amounts([
                (raw, 80.0 * stock),
                (processed, 30.0 * stock),
                (energy, energy_supply),
            ]),
            production: Some(ProductionConfig {
                inputs: resource_amounts([(raw, 5.0), (energy, 1.0)]),
                outputs: resource_amounts([(processed, 4.0)]),
                energy_cost: 1.0,
            }),
        },
        EntityConfig {
            name: names[2].to_string(),
            entity_type: "producer".to_string(),
            region: region.to_string(),
            initial_inventory: resource_amounts([
                (processed, 70.0 * stock),
                (final_resource, 20.0 * stock),
                (energy, energy_supply),
            ]),
            production: Some(ProductionConfig {
                inputs: resource_amounts([(processed, 3.0), (energy, 1.0)]),
                outputs: resource_amounts([(final_resource, 5.0)]),
                energy_cost: 1.0,
            }),
        },
        EntityConfig {
            name: names[3].to_string(),
            entity_type: "storage".to_string(),
            region: region.to_string(),
            initial_inventory: resource_amounts([(final_resource, 90.0 * stock)]),
            production: None,
        },
        EntityConfig {
            name: names[4].to_string(),
            entity_type: "consumer".to_string(),
            region: region.to_string(),
            initial_inventory: resource_amounts([(final_resource, 60.0 * stock)]),
            production: Some(ProductionConfig {
                inputs: resource_amounts([(final_resource, demand)]),
                outputs: BTreeMap::new(),
                energy_cost: 0.0,
            }),
        },
    ];
    if template == "city" {
        entities[4].initial_inventory.extend(resource_amounts([
            ("credits", 5_000.0),
            ("materials", 700.0),
            ("research", 140.0),
            ("population", 30.0),
        ]));
    }
    let links = vec![
        LinkConfig {
            from: names[0].to_string(),
            to: names[1].to_string(),
            resource: raw.to_string(),
            max_per_tick: 8.0,
        },
        LinkConfig {
            from: names[1].to_string(),
            to: names[2].to_string(),
            resource: processed.to_string(),
            max_per_tick: 5.0,
        },
        LinkConfig {
            from: names[2].to_string(),
            to: names[3].to_string(),
            resource: final_resource.to_string(),
            max_per_tick: 6.0,
        },
        LinkConfig {
            from: names[3].to_string(),
            to: names[4].to_string(),
            resource: final_resource.to_string(),
            max_per_tick: 5.0,
        },
    ];
    let manifest = WorldManifest {
        name: title.to_string(),
        description: description.to_string(),
        version: "0.1.0".to_string(),
        extends: Vec::new(),
        mods: Vec::new(),
    };
    let scenario = Scenario {
        world: title.to_string(),
        seed,
        duration_ticks: ticks,
        events: vec![ScheduledEvent {
            tick: (ticks / 3).max(1).min(ticks - 1),
            event_type: ScheduledEventType::CapacityChange {
                target: names[2].to_string(),
                value: disruption,
            },
        }],
        objectives: vec![
            Objective {
                objective_type: ObjectiveType::MaintainInventory {
                    resource: final_resource.to_string(),
                    minimum: objective_minimum,
                },
                status: ObjectiveStatus::Pending,
                ever_failed: false,
            },
            Objective {
                objective_type: ObjectiveType::AvoidShortage {
                    resource: final_resource.to_string(),
                },
                status: ObjectiveStatus::Pending,
                ever_failed: false,
            },
        ],
    };
    let config = EntitiesConfig { entities, links };
    let city = (template == "city")
        .then(generated_city_config)
        .transpose()?;
    manifest.validate()?;
    config.validate()?;
    if let Some(city) = &city {
        city.validate(&config)?;
    }
    scenario.validate_against(&manifest, &config)?;
    Ok((manifest, scenario, config, city))
}

fn generated_city_config() -> Result<CityConfig, WorldForgeError> {
    CityConfig::from_toml(
        r#"
treasury = "residential-district"
population_growth_per_tick = 1.0
[population_needs]
food = 0.1
clean-water = 0.1
power = 0.1

[[districts]]
id = "metro-core"
name = "Metro Core"
slots = 10
[[districts]]
id = "green-ring"
name = "Green Ring"
slots = 12

[[buildings]]
id = "mixed-housing"
name = "Mixed Housing"
category = "residential"
allowed_districts = ["metro-core", "green-ring"]
tags = ["housing", "community"]
footprint = 2
housing = 100
jobs = 10
wellbeing = 3.0
[buildings.cost]
credits = 600.0
materials = 80.0
[buildings.upkeep]
food = 1.0
clean-water = 1.0
power = 1.0
[buildings.outputs]
credits = 4.0

[[buildings]]
id = "research-commons"
name = "Research Commons"
category = "knowledge"
allowed_districts = ["metro-core"]
tags = ["science", "community"]
footprint = 2
jobs = 30
wellbeing = 2.0
[buildings.cost]
credits = 800.0
materials = 100.0
[buildings.upkeep]
power = 1.5
[buildings.outputs]
research = 3.0

[[buildings]]
id = "solar-garden"
name = "Solar Garden"
category = "energy"
allowed_districts = ["green-ring"]
tags = ["energy", "green"]
requires_technologies = ["distributed-energy"]
[buildings.cost]
credits = 500.0
materials = 60.0
[buildings.outputs]
power = 8.0

[[buildings]]
id = "food-loop"
name = "Food Loop"
category = "food"
allowed_districts = ["green-ring", "metro-core"]
tags = ["food", "green"]
requires_technologies = ["circular-systems"]
[buildings.cost]
credits = 650.0
materials = 90.0
[buildings.upkeep]
clean-water = 0.8
power = 1.0
[buildings.outputs]
food = 6.0
[[buildings.synergies]]
with_tag = "energy"
output_multiplier = 1.35

[[technologies]]
id = "civic-learning"
name = "Civic Learning"
branch = "knowledge"
[technologies.cost]
research = 40.0
[technologies.effects.resource_multipliers]
research = 1.2

[[technologies]]
id = "distributed-energy"
name = "Distributed Energy"
branch = "infrastructure"
prerequisites = ["civic-learning"]
[technologies.cost]
research = 65.0
[technologies.effects.resource_multipliers]
power = 1.3

[[technologies]]
id = "circular-systems"
name = "Circular Systems"
branch = "ecology"
prerequisites = ["civic-learning"]
[technologies.cost]
research = 65.0
[technologies.effects.resource_multipliers]
food = 1.25

[[factions]]
id = "residents-council"
name = "Residents Council"
description = "Households organizing for security, services, and accountable growth."
initial_support = 56.0

[[factions]]
id = "innovation-league"
name = "Innovation League"
description = "Builders and researchers pressing for ambitious urban experiments."
initial_support = 52.0

[[dilemmas]]
id = "founding-charter"
title = "Choose the City's Founding Charter"
description = "Set the values that will steer the first generation of development."
deadline_ticks = 12
default_option = "innovation-district"
[dilemmas.trigger]
tick = 5

[[dilemmas.options]]
id = "neighborhood-commons"
label = "Neighborhood commons"
description = "Prioritize shared land, housing capacity, and public life."
[dilemmas.options.cost]
credits = 150.0
[dilemmas.options.faction_support]
residents-council = 12.0
innovation-league = -4.0
[dilemmas.options.effects]
housing_multiplier = 1.15
wellbeing_bonus = 5.0

[[dilemmas.options]]
id = "innovation-district"
label = "Innovation district"
description = "Give new ventures room to move and compound productive capacity."
[dilemmas.options.faction_support]
innovation-league = 12.0
residents-council = -5.0
[dilemmas.options.effects]
jobs_multiplier = 1.18
[dilemmas.options.effects.resource_multipliers]
research = 1.15

[[corporations]]
id = "nova-consortium"
name = "Nova Consortium"
sector = "urban-technology"
security = 55.0
influence = 68.0
[[corporations.secrets]]
id = "adaptive-infrastructure"
name = "Adaptive Infrastructure Model"
difficulty = 50.0
research_value = 60.0

[[cyber_agents]]
id = "civic-specter"
name = "Civic Specter"
description = "A sandboxed municipal intelligence agent for abstract covert operations."
skill = 75.0
stealth = 72.0
loyalty = 85.0
containment = 80.0
initial_status = "ready"

[[crypto_assets]]
id = "metro-token"
name = "Metro Token"
symbol = "MTR"
initial_price = 8.0
volatility = 0.18
liquidity = 4000.0
"#,
    )
}

fn resource_amounts<const N: usize>(pairs: [(&str, f64); N]) -> BTreeMap<String, f64> {
    pairs
        .into_iter()
        .map(|(resource, amount)| (resource.to_string(), amount))
        .collect()
}

fn write_toml_file<T: Serialize>(path: &Path, value: &T) -> Result<(), WorldForgeError> {
    let encoded = toml::to_string_pretty(value)
        .map_err(|error| WorldForgeError::new(ErrorCode::InternalError, error.to_string()))?;
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(io_error)?;
    file.write_all(encoded.as_bytes()).map_err(io_error)?;
    file.sync_all().map_err(io_error)
}

fn catalog(worlds_dir: &Path) -> Result<Value, WorldForgeError> {
    let catalog_root = std::fs::canonicalize(worlds_dir).map_err(io_error)?;
    let mut directories = std::fs::read_dir(worlds_dir)
        .map_err(io_error)?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .collect::<Vec<_>>();
    directories.sort_by_key(|entry| entry.file_name());

    let mut worlds = Vec::new();
    for entry in directories {
        let Ok(path) = std::fs::canonicalize(entry.path()) else {
            continue;
        };
        if path.parent() != Some(catalog_root.as_path()) {
            continue;
        }
        if let Some(world) = catalog_entry(&path)? {
            worlds.push(world);
        }
    }
    Ok(Value::Array(worlds))
}

fn catalog_entry(path: &Path) -> Result<Option<Value>, WorldForgeError> {
    if !path.join("world.toml").is_file()
        || !path.join("scenario.toml").is_file()
        || !path.join("entities.toml").is_file()
    {
        return Ok(None);
    }
    let resolved = worldforge_package::resolve_world(path)?;
    let manifest = resolved.manifest;
    let scenario = resolved.scenario;
    let entities = resolved.entities;
    let city = resolved.city;
    let slug = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| invalid_request("world directory name is not valid UTF-8"))?;
    let mut resources = std::collections::BTreeSet::new();
    for entity in &entities.entities {
        resources.extend(entity.initial_inventory.keys().cloned());
        if let Some(production) = &entity.production {
            resources.extend(production.inputs.keys().cloned());
            resources.extend(production.outputs.keys().cloned());
        }
    }
    for link in &entities.links {
        resources.insert(link.resource.clone());
    }

    Ok(Some(json!({
        "id": slug,
        "title": manifest.name,
        "description": manifest.description,
        "version": manifest.version,
        "defaultSeed": scenario.seed,
        "defaultTicks": scenario.duration_ticks,
        "entityCount": entities.entities.len(),
        "resourceCount": resources.len(),
        "cityBuildingCount": city.as_ref().map_or(0, |city| city.buildings.len()),
        "technologyCount": city.as_ref().map_or(0, |city| city.technologies.len()),
        "entities": entities.entities.iter().map(|entity| &entity.name).collect::<Vec<_>>(),
        "resources": resources,
        "links": entities.links,
        "dependencies": resolved.dependencies.iter().map(|dependency| json!({
            "reference": dependency.reference,
            "name": dependency.name,
            "version": dependency.version,
            "fingerprint": dependency.fingerprint.to_string(),
        })).collect::<Vec<_>>(),
        "effectiveFingerprint": resolved.fingerprint.to_string(),
    })))
}

fn benchmark_report(path: &Path, request: &BenchmarkRequest) -> Result<Value, WorldForgeError> {
    let mut samples_ms = Vec::with_capacity(request.reps as usize);
    let mut event_count = 0usize;
    for _ in 0..request.reps {
        let started = Instant::now();
        let mut runtime = SimulationRuntime::load_bounded(path, 42, Some(request.ticks), 0)?;
        let result = runtime.run()?;
        samples_ms.push(started.elapsed().as_secs_f64() * 1000.0);
        event_count = result.event_count;
    }
    let total_ms = samples_ms.iter().sum::<f64>();
    let average_ms = total_ms / f64::from(request.reps);
    let min_ms = samples_ms.iter().copied().fold(f64::INFINITY, f64::min);
    let max_ms = samples_ms.iter().copied().fold(0.0, f64::max);
    let ticks_per_second = if average_ms > 0.0 {
        request.ticks as f64 / (average_ms / 1000.0)
    } else {
        0.0
    };
    let events_per_second = if average_ms > 0.0 {
        event_count as f64 / (average_ms / 1000.0)
    } else {
        0.0
    };

    Ok(json!({
        "world": request.world,
        "ticks": request.ticks,
        "reps": request.reps,
        "samplesMs": samples_ms,
        "averageMs": average_ms,
        "minMs": min_ms,
        "maxMs": max_ms,
        "ticksPerSecond": ticks_per_second,
        "eventsPerSecond": events_per_second,
        "eventsPerRun": event_count,
    }))
}

fn resolve_world(worlds_dir: &Path, world: &str) -> Result<PathBuf, WorldForgeError> {
    if world.is_empty()
        || !world
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
    {
        return Err(invalid_request(
            "world must contain only letters, numbers, and hyphens",
        ));
    }
    let world_path = worlds_dir.join(world);
    if !world_path.join("world.toml").is_file() {
        return Err(WorldForgeError::new(
            ErrorCode::WorldNotFound,
            format!("world '{world}' does not exist"),
        ));
    }
    let catalog_root = std::fs::canonicalize(worlds_dir).map_err(io_error)?;
    let canonical = std::fs::canonicalize(&world_path).map_err(io_error)?;
    if canonical.parent() != Some(catalog_root.as_path()) {
        return Err(invalid_request(
            "world must resolve inside the configured catalog",
        ));
    }
    Ok(canonical)
}

fn validate_ticks(ticks: u64) -> Result<(), WorldForgeError> {
    if ticks == 0 || ticks > MAX_TICKS {
        return Err(invalid_request(format!(
            "ticks must be between 1 and {MAX_TICKS}"
        )));
    }
    Ok(())
}

fn require_json_content_type(header: &str) -> Result<(), WorldForgeError> {
    let is_json = header.lines().any(|line| {
        line.split_once(':').is_some_and(|(name, value)| {
            name.eq_ignore_ascii_case("content-type")
                && value
                    .trim()
                    .to_ascii_lowercase()
                    .starts_with("application/json")
        })
    });
    if is_json {
        Ok(())
    } else {
        Err(invalid_request("content-type must be application/json"))
    }
}

fn parse_json<T: for<'de> Deserialize<'de>>(body: &[u8]) -> Result<T, WorldForgeError> {
    serde_json::from_slice(body)
        .map_err(|error| invalid_request(format!("invalid JSON request: {error}")))
}

fn read_request(stream: &mut TcpStream) -> Result<Vec<u8>, WorldForgeError> {
    let mut request = Vec::new();
    let mut buffer = [0u8; 8192];
    let mut expected_length = None;

    loop {
        let count = stream.read(&mut buffer).map_err(io_error)?;
        if count == 0 {
            break;
        }
        request.extend_from_slice(&buffer[..count]);
        if request.len() > MAX_REQUEST_BYTES {
            return Err(invalid_request(format!(
                "HTTP request exceeds {MAX_REQUEST_BYTES} bytes"
            )));
        }
        if let Some(header_end) = request.windows(4).position(|window| window == b"\r\n\r\n") {
            let content_length = expected_length.get_or_insert_with(|| {
                let header = String::from_utf8_lossy(&request[..header_end]);
                header
                    .lines()
                    .find_map(|line| {
                        let (name, value) = line.split_once(':')?;
                        name.eq_ignore_ascii_case("content-length")
                            .then(|| value.trim().parse::<usize>().ok())
                            .flatten()
                    })
                    .unwrap_or(0)
            });
            if *content_length > MAX_REQUEST_BYTES {
                return Err(invalid_request("request body is too large"));
            }
            if request.len() >= header_end + 4 + *content_length {
                request.truncate(header_end + 4 + *content_length);
                break;
            }
        }
    }
    Ok(request)
}

fn respond_json(
    stream: &mut TcpStream,
    status: &str,
    value: &Value,
) -> Result<(), WorldForgeError> {
    let body = serde_json::to_vec(value)
        .map_err(|error| WorldForgeError::new(ErrorCode::InternalError, error.to_string()))?;
    respond(
        stream,
        status,
        "application/json; charset=utf-8",
        &body,
        false,
    )
}

fn respond_download(
    stream: &mut TcpStream,
    filename: &str,
    content_type: &str,
    body: &[u8],
) -> Result<(), WorldForgeError> {
    write!(
        stream,
        concat!(
            "HTTP/1.1 200 OK\r\n",
            "Content-Type: {content_type}\r\n",
            "Content-Disposition: attachment; filename=\"{filename}\"\r\n",
            "Content-Length: {length}\r\n",
            "Cache-Control: no-store\r\n",
            "X-Content-Type-Options: nosniff\r\n",
            "Connection: close\r\n\r\n"
        ),
        content_type = content_type,
        filename = filename,
        length = body.len(),
    )
    .map_err(io_error)?;
    stream.write_all(body).map_err(io_error)
}

fn respond_error(
    stream: &mut TcpStream,
    status: &str,
    error: &WorldForgeError,
) -> Result<(), WorldForgeError> {
    respond_json(
        stream,
        status,
        &json!({
            "error": error.message,
            "code": error.code_name,
            "errorId": error.code.to_string(),
        }),
    )
}

fn respond(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &[u8],
    immutable: bool,
) -> Result<(), WorldForgeError> {
    let cache_control = if immutable {
        "public, max-age=31536000, immutable"
    } else {
        "no-store"
    };
    write!(
        stream,
        concat!(
            "HTTP/1.1 {status}\r\n",
            "Content-Type: {content_type}\r\n",
            "Content-Length: {length}\r\n",
            "Cache-Control: {cache_control}\r\n",
            "Content-Security-Policy: default-src 'self'; img-src 'self' data:; style-src 'self'; script-src 'self'; connect-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'\r\n",
            "Referrer-Policy: no-referrer\r\n",
            "X-Content-Type-Options: nosniff\r\n",
            "X-Frame-Options: DENY\r\n",
            "Permissions-Policy: camera=(), microphone=(), geolocation=()\r\n",
            "Connection: close\r\n\r\n"
        ),
        status = status,
        content_type = content_type,
        length = body.len(),
        cache_control = cache_control,
    )
    .map_err(io_error)?;
    stream.write_all(body).map_err(io_error)
}

fn status_for_error(error: &WorldForgeError) -> &'static str {
    match error.code {
        ErrorCode::WorldNotFound | ErrorCode::WorldManifestMissing | ErrorCode::ScenarioMissing => {
            "404 Not Found"
        }
        ErrorCode::WorldManifestInvalid
        | ErrorCode::WorldSchemaViolation
        | ErrorCode::ScenarioInvalid
        | ErrorCode::ReplayFormatInvalid
        | ErrorCode::ReplayVersionIncompatible => "400 Bad Request",
        ErrorCode::RuntimeStateMismatch | ErrorCode::ReplayFingerprintMismatch => "409 Conflict",
        _ => "500 Internal Server Error",
    }
}

fn invalid_request(message: impl Into<String>) -> WorldForgeError {
    WorldForgeError::new(ErrorCode::ScenarioInvalid, message)
}

fn io_error(error: std::io::Error) -> WorldForgeError {
    WorldForgeError::new(ErrorCode::RuntimeInitFailed, error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn examples_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples")
    }

    fn test_state() -> (ServerState, PathBuf) {
        let saves_dir = std::env::temp_dir().join(format!(
            "worldforge-dashboard-saves-{}",
            uuid::Uuid::new_v4().simple()
        ));
        std::fs::create_dir_all(&saves_dir).unwrap();
        (
            ServerState {
                worlds_dir: examples_dir(),
                saves_dir: saves_dir.clone(),
                sessions: Mutex::new(BTreeMap::new()),
                save_io: Mutex::new(()),
                world_io: Mutex::new(()),
            },
            saves_dir,
        )
    }

    #[test]
    fn catalog_is_derived_from_world_files() {
        let worlds = catalog(&examples_dir()).unwrap();
        let worlds = worlds.as_array().unwrap();
        assert!(worlds.len() >= 5);
        let supply_chain = worlds
            .iter()
            .find(|world| world["id"] == "supply-chain")
            .unwrap();
        assert_eq!(supply_chain["entityCount"], 5);
        assert_eq!(supply_chain["defaultTicks"], 1000);
        assert!(supply_chain["resources"].as_array().unwrap().len() >= 4);

        let recovery = worlds
            .iter()
            .find(|world| world["id"] == "supply-chain-recovery")
            .unwrap();
        assert_eq!(recovery["entityCount"], 7);
        assert_eq!(recovery["dependencies"].as_array().unwrap().len(), 1);
        assert!(recovery["effectiveFingerprint"].is_string());
    }

    #[test]
    fn world_resolution_rejects_traversal_and_missing_worlds() {
        assert!(resolve_world(&examples_dir(), "../supply-chain").is_err());
        assert!(resolve_world(&examples_dir(), "does-not-exist").is_err());
        assert!(resolve_world(&examples_dir(), "supply-chain").is_ok());
    }

    #[test]
    fn benchmark_limits_are_enforced() {
        assert!(validate_ticks(0).is_err());
        assert!(validate_ticks(MAX_TICKS + 1).is_err());
        assert!(validate_ticks(1000).is_ok());
    }

    #[test]
    fn dashboard_run_bounds_event_payload_without_losing_aggregates() {
        let document = super::super::commands::simulation_export_for_dashboard(
            &examples_dir().join("stress-test"),
            100,
            42,
            50,
        )
        .unwrap();
        assert_eq!(document["events"].as_array().unwrap().len(), 50);
        assert_eq!(document["eventsTruncated"], true);
        assert!(document["totalEvents"].as_u64().unwrap() > 50);
        let counted = document["eventTypeCounts"]
            .as_object()
            .unwrap()
            .values()
            .map(|value| value.as_u64().unwrap())
            .sum::<u64>();
        assert_eq!(counted, document["totalEvents"].as_u64().unwrap());
        let metrics = document["resourceMetrics"].as_array().unwrap();
        assert!(!metrics.is_empty());
        assert!(metrics.iter().all(|metric| metric["minimumTick"].is_u64()));
        let objective_document = super::super::commands::simulation_export_for_dashboard(
            &examples_dir().join("supply-chain"),
            10,
            42,
            50,
        )
        .unwrap();
        let objective = objective_document["objectives"]
            .as_array()
            .unwrap()
            .first()
            .unwrap();
        assert!(objective["kind"].is_string());
        assert!(objective["progress"].is_number());
    }

    #[test]
    fn play_routes_only_accept_exact_session_shapes() {
        assert_eq!(play_session_id("/api/play/sessions/abc"), Some("abc"));
        assert_eq!(play_session_id("/api/play/sessions/abc/step"), None);
        assert_eq!(
            play_action_id("/api/play/sessions/abc/step", "step"),
            Some("abc")
        );
        assert_eq!(play_action_id("/api/play/sessions//step", "step"), None);
    }

    #[test]
    fn play_sessions_step_intervene_and_delete() {
        let (state, saves_dir) = test_state();
        let created = create_play_session(
            &state,
            RunRequest {
                world: "supply-chain".to_string(),
                seed: 42,
                ticks: 8,
            },
        )
        .unwrap();
        let id = created["sessionId"].as_str().unwrap();
        assert_eq!(created["currentTick"], 0);
        assert!(!created["resourceMetrics"].as_array().unwrap().is_empty());
        assert!(created["objectives"][0]["progress"].is_number());

        let stepped = step_play_session(&state, id, 3).unwrap();
        assert_eq!(stepped["currentTick"], 3);
        assert_eq!(stepped["state"], "Running");
        assert!(!stepped["recentEvents"].as_array().unwrap().is_empty());

        let changed = intervene_play_session(
            &state,
            id,
            InterventionRequest {
                entity: "factory".to_string(),
                capacity: 0.5,
            },
        )
        .unwrap();
        assert_eq!(changed["recentEvents"][0]["type"], "system");

        let completed = step_play_session(&state, id, 20).unwrap();
        assert_eq!(completed["currentTick"], 8);
        assert_eq!(completed["completed"], true);
        assert!(completed["proof"]["eventChain"].is_string());

        delete_play_session(&state, id).unwrap();
        assert!(inspect_play_session(&state, id).is_err());
        std::fs::remove_dir_all(saves_dir).unwrap();
    }

    #[test]
    fn city_actions_survive_save_and_resume_with_identical_proof() {
        let (state, saves_dir) = test_state();
        let created = create_play_session(
            &state,
            RunRequest {
                world: "micro-city".to_string(),
                seed: 99,
                ticks: 12,
            },
        )
        .unwrap();
        let original_id = created["sessionId"].as_str().unwrap();
        let researched = research_in_play_session(
            &state,
            original_id,
            ResearchRequest {
                technology: "solar-weave".to_string(),
            },
        )
        .unwrap();
        assert_eq!(researched["recentEvents"][0]["type"], "research");
        let built = construct_in_play_session(
            &state,
            original_id,
            ConstructRequest {
                building: "solar-canopy".to_string(),
                district: "sun-belt".to_string(),
            },
        )
        .unwrap();
        assert_eq!(built["recentEvents"][0]["type"], "construction");
        construct_in_play_session(
            &state,
            original_id,
            ConstructRequest {
                building: "courtyard-homes".to_string(),
                district: "civic-core".to_string(),
            },
        )
        .unwrap();
        step_play_session(&state, original_id, 5).unwrap();
        let decided = decide_in_play_session(
            &state,
            original_id,
            DecisionRequest {
                dilemma: "growth-charter".to_string(),
                option: "civic-land-trust".to_string(),
            },
        )
        .unwrap();
        assert_eq!(decided["recentEvents"][0]["type"], "governance");
        let traded = intrigue_in_play_session(
            &state,
            original_id,
            IntrigueRequest {
                action: "trade".to_string(),
                target: "forgecoin".to_string(),
                agent: None,
                option: Some("buy".to_string()),
            },
        )
        .unwrap();
        assert!(traded["city"]["intrigue"]["markets"]
            .as_array()
            .unwrap()
            .iter()
            .any(
                |market| market["id"] == "forgecoin" && market["holdings"].as_f64().unwrap() > 0.0
            ));
        let save = save_play_session(
            &state,
            SaveRequest {
                session_id: original_id.to_string(),
                name: "The solar turn".to_string(),
                save_id: None,
            },
        )
        .unwrap();
        let original_final = step_play_session(&state, original_id, 20).unwrap();
        let resumed = load_save(&state, &save.id).unwrap();
        assert_eq!(resumed["city"]["housing"], 106);
        assert_eq!(
            resumed["city"]["governance"]["decisions"][0]["option"],
            "civic-land-trust"
        );
        assert!(resumed["city"]["intrigue"]["markets"]
            .as_array()
            .unwrap()
            .iter()
            .any(
                |market| market["id"] == "forgecoin" && market["holdings"].as_f64().unwrap() > 0.0
            ));
        let resumed_id = resumed["sessionId"].as_str().unwrap();
        let resumed_final = step_play_session(&state, resumed_id, 20).unwrap();
        assert_eq!(
            resumed_final["proof"]["eventChain"],
            original_final["proof"]["eventChain"]
        );
        assert_eq!(
            resumed_final["proof"]["final"],
            original_final["proof"]["final"]
        );
        std::fs::remove_dir_all(saves_dir).unwrap();
    }

    #[test]
    fn save_slots_resume_with_the_same_final_proof() {
        let (state, saves_dir) = test_state();
        let created = create_play_session(
            &state,
            RunRequest {
                world: "supply-chain".to_string(),
                seed: 19,
                ticks: 15,
            },
        )
        .unwrap();
        let original_id = created["sessionId"].as_str().unwrap();
        step_play_session(&state, original_id, 4).unwrap();
        intervene_play_session(
            &state,
            original_id,
            InterventionRequest {
                entity: "factory".to_string(),
                capacity: 0.65,
            },
        )
        .unwrap();
        step_play_session(&state, original_id, 3).unwrap();

        let save = save_play_session(
            &state,
            SaveRequest {
                session_id: original_id.to_string(),
                name: "Before the disruption".to_string(),
                save_id: None,
            },
        )
        .unwrap();
        assert_eq!(list_saves(&state).unwrap().len(), 1);

        let overwritten = save_play_session(
            &state,
            SaveRequest {
                session_id: original_id.to_string(),
                name: "Before the disruption · updated".to_string(),
                save_id: Some(save.id.clone()),
            },
        )
        .unwrap();
        assert_eq!(overwritten.id, save.id);
        assert_eq!(overwritten.created_at, save.created_at);
        assert_eq!(read_save(&state, &save.id).unwrap().name, overwritten.name);
        assert!(std::fs::read_dir(&saves_dir).unwrap().all(|entry| {
            let name = entry.unwrap().file_name().to_string_lossy().into_owned();
            !name.ends_with(".tmp") && !name.ends_with(".bak")
        }));

        let original_final = step_play_session(&state, original_id, 100).unwrap();
        let resumed = load_save(&state, &save.id).unwrap();
        assert_eq!(resumed["currentTick"], 7);
        let resumed_id = resumed["sessionId"].as_str().unwrap();
        let resumed_final = step_play_session(&state, resumed_id, 100).unwrap();
        assert_eq!(
            resumed_final["proof"]["eventChain"],
            original_final["proof"]["eventChain"]
        );
        assert_eq!(
            resumed_final["proof"]["final"],
            original_final["proof"]["final"]
        );

        delete_save(&state, &save.id).unwrap();
        assert!(list_saves(&state).unwrap().is_empty());
        std::fs::remove_dir_all(saves_dir).unwrap();
    }

    #[test]
    fn save_recovery_restores_backup_and_removes_partial_temporary_files() {
        let saves_dir = std::env::temp_dir().join(format!(
            "worldforge-dashboard-recovery-{}",
            uuid::Uuid::new_v4().simple()
        ));
        std::fs::create_dir_all(&saves_dir).unwrap();
        let id = uuid::Uuid::new_v4().simple().to_string();
        let backup = saves_dir.join(format!("{id}.json.bak"));
        let temporary = saves_dir.join(format!(".worldforge-save-{id}-partial.tmp"));
        std::fs::write(&backup, b"recoverable").unwrap();
        std::fs::write(&temporary, b"partial").unwrap();

        recover_save_directory(&saves_dir).unwrap();

        assert_eq!(
            std::fs::read(saves_dir.join(format!("{id}.json"))).unwrap(),
            b"recoverable"
        );
        assert!(!backup.exists());
        assert!(!temporary.exists());
        std::fs::remove_dir_all(saves_dir).unwrap();
    }

    #[test]
    fn world_builder_creates_valid_playable_templates_atomically() {
        let root = std::env::temp_dir().join(format!(
            "worldforge-dashboard-builder-{}",
            uuid::Uuid::new_v4().simple()
        ));
        let worlds_dir = root.join("worlds");
        let saves_dir = root.join("saves");
        std::fs::create_dir_all(&worlds_dir).unwrap();
        std::fs::create_dir_all(&saves_dir).unwrap();
        let state = ServerState {
            worlds_dir: worlds_dir.clone(),
            saves_dir,
            sessions: Mutex::new(BTreeMap::new()),
            save_io: Mutex::new(()),
            world_io: Mutex::new(()),
        };

        for (index, template) in ["industrial", "city", "ecosystem"].into_iter().enumerate() {
            let id = format!("generated-{template}");
            let created = create_world(
                &state,
                WorldBuildRequest {
                    id: id.clone(),
                    title: format!("Generated {template}"),
                    description: "Generated by the validated builder".to_string(),
                    template: template.to_string(),
                    difficulty: ["easy", "standard", "hard"][index].to_string(),
                    seed: 100 + index as u64,
                    ticks: 60,
                },
            )
            .unwrap();
            assert_eq!(created["id"], id);
            worldforge_package::validate_world(&worlds_dir.join(&id)).unwrap();
            let mut runtime = SimulationRuntime::load(&worlds_dir.join(&id), 42, Some(20)).unwrap();
            assert_eq!(runtime.run().unwrap().total_ticks, 20);
        }
        assert_eq!(catalog(&worlds_dir).unwrap().as_array().unwrap().len(), 3);
        assert!(create_world(
            &state,
            WorldBuildRequest {
                id: "generated-city".to_string(),
                title: "Duplicate".to_string(),
                description: String::new(),
                template: "city".to_string(),
                difficulty: "standard".to_string(),
                seed: 42,
                ticks: 100,
            }
        )
        .is_err());
        assert!(create_world(
            &state,
            WorldBuildRequest {
                id: "../escape".to_string(),
                title: "Invalid".to_string(),
                description: String::new(),
                template: "industrial".to_string(),
                difficulty: "standard".to_string(),
                seed: 42,
                ticks: 100,
            }
        )
        .is_err());
        assert!(std::fs::read_dir(&worlds_dir).unwrap().all(|entry| !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".worldforge-new-")));
        std::fs::remove_dir_all(root).unwrap();
    }
}

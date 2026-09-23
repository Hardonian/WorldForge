//! Small, dependency-free localhost dashboard server backed by the real engine.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::Deserialize;
use serde_json::{json, Value};
use worldforge_core::{EngineVersion, ErrorCode, WorldForgeError};
use worldforge_world::{EntitiesConfig, Scenario, WorldManifest};

const INDEX: &str = include_str!("../../../dashboard/index.html");
const SCRIPT: &str = include_str!("../../../dashboard/dashboard.js");
const STYLE: &str = include_str!("../../../dashboard/dashboard.css");
const WORLD_ATLAS: &[u8] = include_bytes!("../../../dashboard/assets/world-atlas.png");
const MAX_REQUEST_BYTES: usize = 64 * 1024;
const MAX_TICKS: u64 = 1_000_000;
const MAX_BENCHMARK_REPS: u32 = 20;

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

fn default_benchmark_reps() -> u32 {
    5
}

pub fn serve(bind: &str, worlds_dir: &Path) -> Result<(), WorldForgeError> {
    if !worlds_dir.is_dir() {
        return Err(WorldForgeError::new(
            ErrorCode::WorldNotFound,
            format!("world catalog does not exist: {}", worlds_dir.display()),
        ));
    }
    let worlds_dir = std::fs::canonicalize(worlds_dir).map_err(io_error)?;
    let listener = TcpListener::bind(bind).map_err(|error| {
        WorldForgeError::new(
            ErrorCode::RuntimeInitFailed,
            format!("cannot bind dashboard to {bind}: {error}"),
        )
    })?;
    println!("World Forge dashboard: http://{bind}");
    println!("World catalog: {}", worlds_dir.display());
    println!("Press Ctrl+C to stop.");

    let worlds_dir = Arc::new(worlds_dir);
    for connection in listener.incoming() {
        match connection {
            Ok(mut stream) => {
                let worlds_dir = Arc::clone(&worlds_dir);
                std::thread::spawn(move || {
                    if let Err(error) = handle_connection(&mut stream, &worlds_dir) {
                        let _ = respond_error(&mut stream, status_for_error(&error), &error);
                    }
                });
            }
            Err(error) => eprintln!("dashboard connection failed: {error}"),
        }
    }
    Ok(())
}

fn handle_connection(stream: &mut TcpStream, worlds_dir: &Path) -> Result<(), WorldForgeError> {
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
        ("GET", "/api/health") => {
            let world_count = catalog(worlds_dir)?.as_array().map_or(0, Vec::len);
            respond_json(
                stream,
                "200 OK",
                &json!({
                    "status": "healthy",
                    "engineVersion": EngineVersion::current().to_string(),
                    "worlds": world_count,
                }),
            )
        }
        ("GET", "/api/worlds") => {
            let worlds = catalog(worlds_dir)?;
            respond_json(stream, "200 OK", &json!({ "worlds": worlds }))
        }
        ("POST", "/api/run") => {
            require_json_content_type(&header)?;
            let run: RunRequest = parse_json(body)?;
            validate_ticks(run.ticks)?;
            let world_path = resolve_world(worlds_dir, &run.world)?;
            let document = super::commands::simulation_export(&world_path, run.ticks, run.seed)?;
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
            let world_path = resolve_world(worlds_dir, &request.world)?;
            let report = benchmark_report(&world_path, &request)?;
            respond_json(stream, "200 OK", &report)
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

fn catalog(worlds_dir: &Path) -> Result<Value, WorldForgeError> {
    let mut directories = std::fs::read_dir(worlds_dir)
        .map_err(io_error)?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .collect::<Vec<_>>();
    directories.sort_by_key(|entry| entry.file_name());

    let worlds = directories
        .into_iter()
        .filter_map(|entry| catalog_entry(&entry.path()).transpose())
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Value::Array(worlds))
}

fn catalog_entry(path: &Path) -> Result<Option<Value>, WorldForgeError> {
    if !path.join("world.toml").is_file()
        || !path.join("scenario.toml").is_file()
        || !path.join("entities.toml").is_file()
    {
        return Ok(None);
    }
    let manifest = WorldManifest::from_file(&path.join("world.toml"))?;
    let scenario = Scenario::from_file(&path.join("scenario.toml"))?;
    let entities = EntitiesConfig::from_file(&path.join("entities.toml"))?;
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
        "entities": entities.entities.iter().map(|entity| &entity.name).collect::<Vec<_>>(),
        "resources": resources,
        "links": entities.links,
    })))
}

fn benchmark_report(path: &Path, request: &BenchmarkRequest) -> Result<Value, WorldForgeError> {
    let mut samples_ms = Vec::with_capacity(request.reps as usize);
    let mut event_count = 0usize;
    for _ in 0..request.reps {
        let started = Instant::now();
        let result = super::commands::simulation_export(path, request.ticks, 42)?;
        samples_ms.push(started.elapsed().as_secs_f64() * 1000.0);
        event_count = result["totalEvents"].as_u64().unwrap_or_default() as usize;
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
    Ok(world_path)
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
                && value.trim().to_ascii_lowercase().starts_with("application/json")
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
        | ErrorCode::ScenarioInvalid => "400 Bad Request",
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
}

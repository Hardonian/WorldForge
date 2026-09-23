//! Minimal localhost dashboard server with a real simulation API.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::time::Duration;

use serde::Deserialize;
use worldforge_core::{ErrorCode, WorldForgeError};

const INDEX: &str = include_str!("../../../dashboard/index.html");
const SCRIPT: &str = include_str!("../../../dashboard/dashboard.js");
const STYLE: &str = include_str!("../../../dashboard/dashboard.css");

#[derive(Deserialize)]
struct RunRequest {
    world: String,
    seed: u64,
    ticks: u64,
}

pub fn serve(bind: &str) -> Result<(), WorldForgeError> {
    let listener = TcpListener::bind(bind).map_err(|error| {
        WorldForgeError::new(
            ErrorCode::RuntimeInitFailed,
            format!("cannot bind dashboard to {bind}: {error}"),
        )
    })?;
    println!("World Forge dashboard: http://{bind}");
    println!("Press Ctrl+C to stop.");

    for connection in listener.incoming() {
        match connection {
            Ok(mut stream) => {
                if let Err(error) = handle_connection(&mut stream) {
                    let payload = serde_json::json!({ "error": error.to_string() }).to_string();
                    let _ = respond(
                        &mut stream,
                        "500 Internal Server Error",
                        "application/json; charset=utf-8",
                        payload.as_bytes(),
                    );
                }
            }
            Err(error) => eprintln!("dashboard connection failed: {error}"),
        }
    }
    Ok(())
}

fn handle_connection(stream: &mut TcpStream) -> Result<(), WorldForgeError> {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(io_error)?;
    let request = read_request(stream)?;
    let header_end = request
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| {
            WorldForgeError::new(ErrorCode::RuntimeInitFailed, "invalid HTTP request")
        })?;
    let header = String::from_utf8_lossy(&request[..header_end]);
    let request_line = header.lines().next().unwrap_or_default();
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let route = parts.next().unwrap_or_default();

    match (method, route) {
        ("GET", "/") | ("GET", "/index.html") => respond(
            stream,
            "200 OK",
            "text/html; charset=utf-8",
            INDEX.as_bytes(),
        ),
        ("GET", "/dashboard.js") => respond(
            stream,
            "200 OK",
            "text/javascript; charset=utf-8",
            SCRIPT.as_bytes(),
        ),
        ("GET", "/dashboard.css") => respond(
            stream,
            "200 OK",
            "text/css; charset=utf-8",
            STYLE.as_bytes(),
        ),
        ("GET", "/api/health") => respond(
            stream,
            "200 OK",
            "application/json; charset=utf-8",
            br#"{"status":"healthy"}"#,
        ),
        ("POST", "/api/run") => {
            let body = &request[header_end + 4..];
            let run: RunRequest = serde_json::from_slice(body).map_err(|error| {
                WorldForgeError::new(
                    ErrorCode::ScenarioInvalid,
                    format!("invalid run request: {error}"),
                )
            })?;
            if run.ticks == 0 || run.ticks > 1_000_000 {
                return Err(WorldForgeError::new(
                    ErrorCode::ScenarioInvalid,
                    "ticks must be between 1 and 1,000,000",
                ));
            }
            if run.world.is_empty()
                || !run
                    .world
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || character == '-')
            {
                return Err(WorldForgeError::new(
                    ErrorCode::WorldNotFound,
                    "world must be an example world name",
                ));
            }
            let world_path = Path::new("examples").join(&run.world);
            if !world_path.join("world.toml").is_file() {
                return Err(WorldForgeError::new(
                    ErrorCode::WorldNotFound,
                    format!("example world '{}' does not exist", run.world),
                ));
            }
            let document = super::commands::simulation_export(&world_path, run.ticks, run.seed)?;
            let payload = serde_json::to_vec(&document).map_err(|error| {
                WorldForgeError::new(ErrorCode::InternalError, error.to_string())
            })?;
            respond(
                stream,
                "200 OK",
                "application/json; charset=utf-8",
                &payload,
            )
        }
        _ => respond(
            stream,
            "404 Not Found",
            "application/json; charset=utf-8",
            br#"{"error":"not found"}"#,
        ),
    }
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
        if request.len() > 64 * 1024 {
            return Err(WorldForgeError::new(
                ErrorCode::RuntimeInitFailed,
                "HTTP request exceeds 64 KiB",
            ));
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
            if request.len() >= header_end + 4 + *content_length {
                request.truncate(header_end + 4 + *content_length);
                break;
            }
        }
    }
    Ok(request)
}

fn respond(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &[u8],
) -> Result<(), WorldForgeError> {
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .map_err(io_error)?;
    stream.write_all(body).map_err(io_error)
}

fn io_error(error: std::io::Error) -> WorldForgeError {
    WorldForgeError::new(ErrorCode::RuntimeInitFailed, error.to_string())
}

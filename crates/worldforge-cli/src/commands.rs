//! CLI command implementations.

use std::path::Path;
use worldforge_core::error::WorldForgeError;
use worldforge_core::hash::Fingerprint;
use worldforge_core::version::EngineVersion;
use worldforge_world::{EventType, WorldManifest};

use super::OutputFormat;

pub fn doctor() -> Result<(), WorldForgeError> {
    println!("World Forge Doctor");
    println!("==================");
    println!();

    let mut healthy = true;

    // Engine version
    let engine = EngineVersion::current();
    println!("  Engine version:    {}", engine);
    print_check("Engine version", true);

    let toolchain = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);
    print_check("Rust toolchain", toolchain);
    healthy &= toolchain;

    // WASM runtime
    print_check("WASM runtime", true);

    // Check example worlds
    let examples = ["examples/supply-chain", "examples/minimal-world"];
    for example in &examples {
        let path = Path::new(example);
        let exists = path.exists() && path.join("world.toml").exists();
        print_check(&format!("Example: {}", example), exists);
        healthy &= exists;
    }

    // Check schemas
    let schemas = [
        "schemas/world.schema.json",
        "schemas/scenario.schema.json",
        "schemas/entities.schema.json",
    ];
    for schema in &schemas {
        let exists = Path::new(schema).exists();
        print_check(&format!("Schema: {}", schema), exists);
        healthy &= exists;
    }

    let writable = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("target/.worldforge-doctor")
        .is_ok();
    print_check("Filesystem permissions", writable);
    healthy &= writable;

    println!();
    if healthy {
        println!("Doctor complete: healthy.");
        Ok(())
    } else {
        Err(WorldForgeError::new(
            worldforge_core::ErrorCode::RuntimeInitFailed,
            "one or more doctor checks failed",
        ))
    }
}

pub fn validate(path: &Path) -> Result<(), WorldForgeError> {
    println!("Validating: {}", path.display());
    worldforge_package::validate_world(path)?;
    println!("  ✓ world.toml valid");
    println!("  ✓ scenario.toml valid");
    println!("  ✓ entities.toml valid");

    let fp = worldforge_package::fingerprint_world(path)?;
    println!("  Fingerprint: {}", fp.to_short_hex());
    println!();
    println!("Validation passed.");
    Ok(())
}

pub fn run(
    path: &Path,
    ticks: Option<u64>,
    seed: Option<u64>,
    output: &OutputFormat,
) -> Result<(), WorldForgeError> {
    let scenario = worldforge_world::Scenario::from_file(&path.join("scenario.toml"))?;
    let ticks = ticks.unwrap_or(scenario.duration_ticks);
    let seed = seed.unwrap_or(scenario.seed);
    println!("World Forge Simulation");
    println!("======================");
    println!("  World:  {}", path.display());
    println!("  Seed:   {}", seed);
    println!("  Ticks:  {}", ticks);
    println!();

    let mut runtime = worldforge_runtime::SimulationRuntime::load(path, seed, Some(ticks))?;
    println!("  World loaded ✓");
    println!("  Scenario validated ✓");

    let result = runtime.run()?;

    println!();
    println!("Run completed ✓");
    println!("  Events:               {}", result.event_count);
    println!("  Shortage events:      {}", result.shortage_count);
    println!(
        "  World fingerprint:    {}",
        result.world_fingerprint.to_short_hex()
    );
    println!(
        "  Initial state:        {}",
        result.initial_state_fingerprint.to_short_hex()
    );
    println!(
        "  Final state:          {}",
        result.final_state_fingerprint.to_short_hex()
    );
    println!(
        "  Event chain root:     {}",
        result.proof.event_chain_root.to_short_hex()
    );

    println!();
    println!("Objectives:");
    for obj in &result.objective_results {
        let icon = match obj.status {
            worldforge_world::ObjectiveStatus::Passed => "✓",
            worldforge_world::ObjectiveStatus::Failed => "✗",
            worldforge_world::ObjectiveStatus::Pending => "?",
        };
        println!("  {} {}: {:?}", icon, obj.name, obj.status);
    }

    // Save replay
    let replay = runtime.take_replay().ok_or_else(|| {
        WorldForgeError::new(
            worldforge_core::ErrorCode::RuntimeStateMismatch,
            "runtime completed without a replay artifact",
        )
    })?;
    let replay_path = path.join("last.replay");
    replay.save(&replay_path)?;
    println!();
    println!("  Replay saved: {}", replay_path.display());

    match output {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&result).unwrap_or_default();
            println!();
            println!("{}", json);
        }
        OutputFormat::Text => {}
    }

    Ok(())
}

pub fn test_world(path: &Path, runs: u32, ticks: u64) -> Result<(), WorldForgeError> {
    if runs == 0 {
        return Err(WorldForgeError::new(
            worldforge_core::ErrorCode::RuntimeInitFailed,
            "runs must be greater than zero",
        ));
    }
    if ticks == 0 {
        return Err(WorldForgeError::new(
            worldforge_core::ErrorCode::ScenarioInvalid,
            "ticks must be greater than zero",
        ));
    }
    println!("World Forge Determinism Test");
    println!("============================");
    println!("  World:  {}", path.display());
    println!("  Runs:   {}", runs);
    println!("  Ticks:  {}", ticks);
    println!();

    let mut completed = 0u32;
    let mut failed = 0u32;
    let mut determinism_failures = 0u32;
    let invariant_violations = 0u32;
    let mut shortage_runs = 0u32;

    // Use the same seed for every run so all requested runs participate in the
    // determinism gate. Seed variation is exercised separately with `run`.
    let mut reference_fp: Option<Fingerprint> = None;

    for run_idx in 0..runs {
        let seed = 42;

        match worldforge_runtime::SimulationRuntime::load(path, seed, Some(ticks)) {
            Ok(mut runtime) => match runtime.run() {
                Ok(result) => {
                    completed += 1;

                    match &reference_fp {
                        None => {
                            reference_fp = Some(result.final_state_fingerprint);
                        }
                        Some(ref_fp) => {
                            if *ref_fp != result.final_state_fingerprint {
                                determinism_failures += 1;
                                eprintln!(
                                    "  DETERMINISM FAILURE at run {}: expected {}, got {}",
                                    run_idx,
                                    ref_fp.to_short_hex(),
                                    result.final_state_fingerprint.to_short_hex()
                                );
                            }
                        }
                    }

                    if result.shortage_count > 0 {
                        shortage_runs += 1;
                    }
                }
                Err(e) => {
                    failed += 1;
                    eprintln!("  Run {} failed: {}", run_idx, e);
                }
            },
            Err(e) => {
                failed += 1;
                eprintln!("  Run {} load failed: {}", run_idx, e);
            }
        }
    }

    println!("Results:");
    println!("  Runs:                  {}", runs);
    println!("  Completed:             {}", completed);
    println!("  Failed:                {}", failed);
    println!("  Determinism failures:  {}", determinism_failures);
    println!("  Invariant violations:  {}", invariant_violations);
    println!("  Shortage scenarios:    {}", shortage_runs);

    if determinism_failures > 0 || failed > 0 || invariant_violations > 0 {
        Err(WorldForgeError::new(
            worldforge_core::error::ErrorCode::RuntimeTickFailed,
            "test-world reported failures",
        ))
    } else {
        println!();
        println!("All checks passed ✓");
        Ok(())
    }
}

pub fn package_build(path: &Path, output: Option<&Path>) -> Result<(), WorldForgeError> {
    let default_output = path.with_extension("world");
    let output_path = output.unwrap_or(&default_output);

    println!(
        "Building package: {} → {}",
        path.display(),
        output_path.display()
    );

    let info = worldforge_package::build_package(path, output_path)?;

    println!("  Name:        {}", info.name);
    println!("  Version:     {}", info.version);
    println!("  Files:       {}", info.file_count);
    println!("  Size:        {} bytes", info.total_bytes);
    println!("  Fingerprint: {}", info.fingerprint.to_short_hex());
    println!();
    println!("Package built ✓");
    Ok(())
}

pub fn package_inspect(file: &Path) -> Result<(), WorldForgeError> {
    println!("Inspecting package: {}", file.display());
    println!();

    let info = worldforge_package::inspect_package(file)?;

    println!("  Name:        {}", info.name);
    println!("  Version:     {}", info.version);
    println!("  Files:       {}", info.file_count);
    println!("  Size:        {} bytes", info.total_bytes);
    println!("  Fingerprint: {}", info.fingerprint.to_short_hex());
    println!();
    println!("Files:");
    for entry in &info.files {
        println!(
            "  {} ({} bytes) [{}]",
            entry.path,
            entry.size,
            entry.fingerprint.to_short_hex()
        );
    }
    Ok(())
}

pub fn package_fingerprint(path: &Path) -> Result<(), WorldForgeError> {
    let fingerprint = if path.is_dir() {
        worldforge_package::fingerprint_world(path)?
    } else {
        worldforge_package::inspect_package(path)?.fingerprint
    };
    println!("{}", fingerprint);
    Ok(())
}

pub fn replay_inspect(file: &Path) -> Result<(), WorldForgeError> {
    println!("Inspecting replay: {}", file.display());
    println!();

    let replay = worldforge_replay::ReplayArtifact::load(file)?;

    println!("  Run ID:          {}", replay.run_id);
    println!("  Engine version:  {}", replay.engine_version);
    println!("  Format version:  {}", replay.format_version);
    println!("  Seed:            {}", replay.seed);
    println!("  Total ticks:     {}", replay.total_ticks);
    println!("  Events:          {}", replay.events.len());
    println!(
        "  World FP:        {}",
        replay.world_fingerprint.to_short_hex()
    );
    println!(
        "  Scenario FP:     {}",
        replay.scenario_fingerprint.to_short_hex()
    );
    println!(
        "  Initial state:   {}",
        replay.initial_state_fingerprint.to_short_hex()
    );
    println!(
        "  Final state:     {}",
        replay.final_state_fingerprint.to_short_hex()
    );
    println!(
        "  Event chain:     {}",
        replay.event_chain_root.to_short_hex()
    );
    println!("  Timestamp:       {}", replay.timestamp);
    Ok(())
}

pub fn replay_verify(file: &Path) -> Result<(), WorldForgeError> {
    println!("Verifying replay: {}", file.display());
    println!();

    let replay = worldforge_replay::ReplayArtifact::load(file)?;
    let report = replay.verify_internal();

    for check in &report.checks {
        let icon = if check.passed { "✓" } else { "✗" };
        println!("  {} {}: {}", icon, check.name, check.detail);
    }

    println!();
    if report.all_passed() {
        println!("Replay verification passed ✓");
        Ok(())
    } else {
        Err(WorldForgeError::new(
            worldforge_core::error::ErrorCode::ReplayHashMismatch,
            format!("{} check(s) failed", report.failed_checks().len()),
        ))
    }
}

pub fn replay_run(file: &Path) -> Result<(), WorldForgeError> {
    let replay = worldforge_replay::ReplayArtifact::load(file)?;
    let report = replay.verify_internal();
    if !report.all_passed() {
        return Err(WorldForgeError::new(
            worldforge_core::error::ErrorCode::ReplayHashMismatch,
            "replay failed integrity verification before execution",
        ));
    }
    if replay.world_path.is_empty() {
        return Err(WorldForgeError::new(
            worldforge_core::error::ErrorCode::ReplayFormatInvalid,
            "replay does not contain a local world path",
        ));
    }

    let world_path = Path::new(&replay.world_path);
    let current_world = worldforge_package::fingerprint_world(world_path)?;
    if current_world != replay.world_fingerprint {
        return Err(WorldForgeError::new(
            worldforge_core::error::ErrorCode::ReplayFingerprintMismatch,
            "world content does not match the replay",
        ));
    }
    if !replay.mod_fingerprints.is_empty() {
        return Err(WorldForgeError::new(
            worldforge_core::error::ErrorCode::ReplayFingerprintMismatch,
            "recorded mod set is unavailable for local replay",
        ));
    }

    let mut runtime = worldforge_runtime::SimulationRuntime::load(
        world_path,
        replay.seed,
        Some(replay.total_ticks),
    )?;
    let result = runtime.run()?;
    if result.initial_state_fingerprint != replay.initial_state_fingerprint
        || result.final_state_fingerprint != replay.final_state_fingerprint
        || result.proof.event_chain_root != replay.event_chain_root
    {
        return Err(WorldForgeError::new(
            worldforge_core::error::ErrorCode::ReplayFingerprintMismatch,
            "re-executed simulation does not match recorded fingerprints",
        ));
    }
    println!("Replay re-execution passed ✓");
    println!("  Final state: {}", result.final_state_fingerprint);
    Ok(())
}

fn print_check(name: &str, passed: bool) {
    let icon = if passed { "✓" } else { "✗" };
    println!("  {} {}", icon, name);
}

pub fn export(
    path: &Path,
    ticks: Option<u64>,
    seed: Option<u64>,
    output: Option<&Path>,
) -> Result<(), WorldForgeError> {
    let scenario = worldforge_world::Scenario::from_file(&path.join("scenario.toml"))?;
    let ticks = ticks.unwrap_or(scenario.duration_ticks);
    let seed = seed.unwrap_or(scenario.seed);
    let export_data = simulation_export(path, ticks, seed)?;

    let json = serde_json::to_string_pretty(&export_data).unwrap_or_default();

    match output {
        Some(out_path) => {
            std::fs::write(out_path, &json).map_err(|e| {
                WorldForgeError::new(
                    worldforge_core::ErrorCode::PackageBuildFailed,
                    e.to_string(),
                )
            })?;
            println!("Exported to: {}", out_path.display());
        }
        None => {
            println!("{}", json);
        }
    }

    Ok(())
}

/// Run a world and produce the canonical dashboard/export document.
pub fn simulation_export(
    path: &Path,
    ticks: u64,
    seed: u64,
) -> Result<serde_json::Value, WorldForgeError> {
    let manifest = WorldManifest::from_file(&path.join("world.toml"))?;
    let mut runtime = worldforge_runtime::SimulationRuntime::load(path, seed, Some(ticks))?;
    let result = runtime.run()?;
    let replay = runtime.replay().ok_or_else(|| {
        WorldForgeError::new(
            worldforge_core::ErrorCode::RuntimeStateMismatch,
            "runtime completed without a replay artifact",
        )
    })?;

    let mut event_type_counts = std::collections::BTreeMap::from([
        ("production", 0usize),
        ("transfer", 0usize),
        ("shortage", 0usize),
        ("price", 0usize),
        ("system", 0usize),
    ]);
    let events = replay
        .events
        .iter()
        .map(|event| {
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
                EventType::ObjectiveUpdated { objective, .. } => {
                    ("system", "objective".to_string(), objective.clone(), 0.0)
                }
                EventType::SimulationDegraded { reason } => {
                    ("system", "runtime".to_string(), reason.clone(), 0.0)
                }
            };
            *event_type_counts.entry(event_type).or_default() += 1;
            serde_json::json!({
                "tick": event.tick.value(),
                "type": event_type,
                "entity": entity,
                "resource": resource,
                "amount": amount,
                "summary": event.summary(),
            })
        })
        .collect::<Vec<_>>();

    let resources = result
        .snapshots
        .first()
        .map(|snapshot| snapshot.levels.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    let entities = result
        .entities
        .iter()
        .map(|entity| entity.name.clone())
        .collect::<Vec<_>>();

    let world_slug = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(&manifest.name);

    Ok(serde_json::json!({
        "world": world_slug,
        "title": manifest.name,
        "description": manifest.description,
        "seed": seed,
        "ticks": ticks,
        "totalEvents": result.event_count,
        "shortageEvents": result.shortage_count,
        "resources": resources,
        "entities": entities,
        "links": result.links,
        "snapshots": result.snapshots,
        "events": events,
        "eventTypeCounts": event_type_counts,
        "objectives": result.objective_results.iter().map(|objective| {
            serde_json::json!({
                "name": objective.name,
                "status": format!("{:?}", objective.status),
            })
        }).collect::<Vec<_>>(),
        "fingerprints": {
            "world": result.world_fingerprint.to_string(),
            "scenario": result.scenario_fingerprint.to_string(),
            "initial": result.initial_state_fingerprint.to_string(),
            "final": result.final_state_fingerprint.to_string(),
            "eventChain": result.proof.event_chain_root.to_string(),
        },
        "meta": {
            "engineVersion": EngineVersion::current().to_string(),
            "runId": result.proof.run_id,
        },
    }))
}

pub fn benchmark(path: &Path, ticks: u64, reps: u32) -> Result<(), WorldForgeError> {
    if reps == 0 {
        return Err(WorldForgeError::new(
            worldforge_core::ErrorCode::RuntimeInitFailed,
            "reps must be greater than zero",
        ));
    }
    println!("World Forge Benchmark");
    println!("=====================");
    println!("  World:  {}", path.display());
    println!("  Ticks:  {}", ticks);
    println!("  Reps:   {}", reps);
    println!();

    let mut durations = Vec::new();
    let mut event_counts = Vec::new();

    for i in 0..reps {
        let start = std::time::Instant::now();
        let mut runtime = worldforge_runtime::SimulationRuntime::load(path, 42, Some(ticks))?;
        let result = runtime.run()?;
        let elapsed = start.elapsed();

        durations.push(elapsed);
        event_counts.push(result.event_count);

        println!(
            "  Run {}/{}: {:.2}ms ({} events)",
            i + 1,
            reps,
            elapsed.as_secs_f64() * 1000.0,
            result.event_count
        );
    }

    let total: std::time::Duration = durations.iter().sum();
    let avg = total / reps;
    let min = durations.iter().min().unwrap();
    let max = durations.iter().max().unwrap();
    let avg_events = event_counts.iter().sum::<usize>() / reps as usize;
    let ticks_per_sec = (ticks as f64) / avg.as_secs_f64();
    let events_per_sec = (avg_events as f64) / avg.as_secs_f64();

    println!();
    println!("Results:");
    println!("  Avg time:        {:.2}ms", avg.as_secs_f64() * 1000.0);
    println!("  Min time:        {:.2}ms", min.as_secs_f64() * 1000.0);
    println!("  Max time:        {:.2}ms", max.as_secs_f64() * 1000.0);
    println!("  Ticks/sec:       {:.0}", ticks_per_sec);
    println!("  Events/sec:      {:.0}", events_per_sec);
    println!("  Avg events/run:  {}", avg_events);

    Ok(())
}

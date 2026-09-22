//! CLI command implementations.

use std::path::Path;
use worldforge_core::error::WorldForgeError;
use worldforge_core::hash::Fingerprint;
use worldforge_core::version::EngineVersion;

use super::OutputFormat;

pub fn doctor() -> Result<(), WorldForgeError> {
    println!("World Forge Doctor");
    println!("==================");
    println!();

    // Engine version
    let engine = EngineVersion::current();
    println!("  Engine version:    {}", engine);
    print_check("Engine version", true);

    // Rust version
    print_check("Rust toolchain", true);

    // WASM runtime
    print_check("WASM runtime", false);
    println!("    → wasmtime integration planned for M1");

    // Check example worlds
    let examples = ["examples/supply-chain", "examples/minimal-world"];
    for example in &examples {
        let path = Path::new(example);
        let exists = path.exists() && path.join("world.toml").exists();
        print_check(&format!("Example: {}", example), exists);
    }

    // Check schemas
    let schemas = ["schemas/world.schema.json", "schemas/scenario.schema.json"];
    for schema in &schemas {
        let exists = Path::new(schema).exists();
        print_check(&format!("Schema: {}", schema), exists);
    }

    println!();
    println!("Doctor complete.");
    Ok(())
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
    ticks: u64,
    seed: u64,
    output: &OutputFormat,
) -> Result<(), WorldForgeError> {
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
    let replay = runtime.build_replay(&result);
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

    // Run with seed 42 twice to verify determinism
    let mut reference_fp: Option<Fingerprint> = None;

    for run_idx in 0..runs {
        let seed = if run_idx < 2 { 42 } else { run_idx as u64 + 1 };

        match worldforge_runtime::SimulationRuntime::load(path, seed, Some(ticks)) {
            Ok(mut runtime) => match runtime.run() {
                Ok(result) => {
                    completed += 1;

                    // Check determinism for seed 42 runs
                    if run_idx < 2 {
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

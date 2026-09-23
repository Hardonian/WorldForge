# World Forge

**A deterministic, moddable simulation runtime where entire worlds are packages.**

World Forge is a simulation operating system for building games, simulations, worlds, scenarios, challenges, and developer-created content. Every simulation run is fully reproducible: same seed + same world = same result, on every platform, every time.

## Status

**Beta release candidate.** The deterministic runtime, packaging toolchain, replay verification, mod sandbox, CLI, and local Simulation Studio are implemented and covered by the workspace verification suite. The beta is local-first: the dashboard binds to localhost by default and runs simulations on the same machine.

| Feature | Status | Notes |
|---------|--------|-------|
| Fixed-point math (Q32.32) | ✅ Implemented | `Fixed64` for all simulation-critical values |
| Deterministic RNG | ✅ Implemented | ChaCha8-based with named streams |
| BLAKE3 fingerprinting | ✅ Implemented | State hashing, event chains, package fingerprints |
| Purpose-built ECS | ✅ Implemented | BTreeMap-backed, deterministic iteration |
| World manifest + scenarios | ✅ Implemented | TOML-based with schema validation |
| Resource economy | ✅ Implemented | Production, transfer, conservation, prices |
| Objective system | ✅ Implemented | Evaluate pass/fail conditions per scenario |
| Replay artifacts | ✅ Implemented | CBOR-serialized with tamper detection |
| Proof chain | ✅ Implemented | Event hash chains with verification |
| Package system (.world) | ✅ Implemented | Deterministic tar with content fingerprinting |
| Agent framework | ✅ Implemented | Rule-based and utility-based policies |
| Capability-based mod API | ✅ Implemented | Deny-by-default capability system |
| WASM mod runtime | ✅ Implemented | Wasmtime core-Wasm sandbox, fuel/memory limits, capability-gated host ABI |
| Simulation Studio | ✅ Implemented | World Builder, durable saves, Play Mode, live decisions, charts, proofs, comparisons, benchmarks, export |
| CLI | ✅ Implemented | doctor, validate, run, export, benchmark, dashboard, package, replay |
| CI pipeline | ✅ Implemented | Cross-platform tests + determinism verification |

## Quick Start

```bash
# Clone and build
git clone https://github.com/Hardonian/worldforge.git
cd worldforge
cargo build

# Run the supply-chain scenario
cargo run -p worldforge-cli -- run examples/supply-chain --seed 42 --ticks 1000

# Verify determinism across 10 runs
cargo run -p worldforge-cli -- test-world examples/supply-chain --runs 10 --ticks 1000

# Verify a replay artifact
cargo run -p worldforge-cli -- replay verify examples/supply-chain/last.replay

# Check runtime health
cargo run -p worldforge-cli -- doctor

# Open the engine-backed Simulation Studio at http://127.0.0.1:8787
cargo run -p worldforge-cli -- dashboard
```

## Architecture

World Forge is a Rust workspace of 12 crates:

```
worldforge-core        Foundation: typed IDs, Fixed64, Tick, RNG, BLAKE3, errors
worldforge-ecs         Deterministic ECS with BTreeMap storage
worldforge-world       World manifest, scenarios, entities, events, objectives
worldforge-economy     Production, transfers, pricing, conservation invariants
worldforge-proof       Event hash chains, run proofs, verification reports
worldforge-replay      CBOR replay artifacts with tamper detection
worldforge-runtime     Simulation executor (load → validate → tick → proof)
worldforge-agent       Rule-based and utility-based agent policies
worldforge-mod-api     Capability-based mod API (deny-by-default)
worldforge-mod-runtime Sandboxed Wasmtime lifecycle with capability-gated host ABI
worldforge-package     .world package format with reproducible fingerprints
worldforge-cli         Command-line interface
```

### Design Principles

1. **Determinism is non-negotiable.** `Fixed64` replaces all floating-point in simulation. `ChaCha8Rng` with explicit seeding. `BTreeMap` for all iteration. No threading in the simulation loop.

2. **Worlds are packages.** A world is a directory containing `world.toml`, `scenario.toml`, `entities.toml`, and optionally mods. Package into `.world` archives with reproducible content fingerprints.

3. **No fake features.** Validation is typed and referential: malformed values, duplicate entities, dangling links, mismatched scenarios, and invalid event targets fail before execution.

4. **Replay verifies the run.** Every simulation produces a CBOR replay artifact containing the event hash chain. Tampering with any event invalidates the chain root.

5. **Mods cannot escape the sandbox.** The capability-based API denies filesystem, network, shell, environment, process, and secrets access. Always.

6. **Observability is bounded by default.** Dashboard runs retain screen-useful history and a recent event window while hashing and counting the complete run. Full replay capture remains opt-in where a lossless artifact is required. See [performance and scalability](docs/performance.md).

## Vertical Slice: Supply Chain

The `examples/supply-chain` directory demonstrates a complete simulation:

- **5 entities**: mine → steel-mill → factory → warehouse → city-market
- **Resource flow**: ore → steel → goods, with energy costs
- **Disruption event**: Factory capacity drops to 50% at tick 250
- **Objectives**: Maintain goods inventory ≥ 50, avoid stockouts
- **Outcome**: The disruption causes cascading shortages; objectives are evaluated continuously and production targets use cumulative output

```bash
cargo run -p worldforge-cli -- run examples/supply-chain --seed 42 --ticks 1000
```

## Verification

```bash
# Run all checks: compile, test, validate, determinism, replay
./scripts/verify.sh       # Linux/macOS
.\scripts\verify.ps1      # Windows
```

The CI pipeline runs these checks on every push:
- Workspace compilation
- 80+ unit and integration tests across all crates
- Cross-platform tests (Linux, Windows, macOS)
- Determinism verification (20 runs with same seed)
- Format and lint checks
- Replay integrity verification

## World Format

```toml
# world.toml
name = "my-world"
description = "A custom simulation"
version = "0.1.0"

# scenario.toml
world = "my-world"
seed = 42
duration_ticks = 500

[[events]]
tick = 100
type = "capacity_change"
target = "factory"
value = 0.5

[[objectives]]
type = "maintain_inventory"
resource = "goods"
minimum = 50.0
```

## Modding

Mods declare capabilities in their manifest and receive only those capabilities at runtime:

```rust
use worldforge_mod_api::{Capability, CapabilityPolicy};

let policy = CapabilityPolicy::with_capabilities(vec![
    Capability::EntityRead,
    Capability::ResourceRead,
    Capability::EventEmit,
]);

// EntityWrite is not granted — mod cannot modify entities
assert!(!policy.check(&Capability::EntityWrite));
```

WIT interfaces for the Component Model are defined in `wit/worldforge/`. The executable v0 runtime loads core WebAssembly modules without WASI, enforces fuel and memory limits, and exposes only capability-checked `read_resource` and `emit_event` host calls. See `examples/mods/read-emit.wat` and `docs/architecture/mod-security.md`.

## Simulation Studio and exports

`worldforge dashboard` serves the bundled UI and a localhost-only simulation API. Every chart, event, objective, benchmark, and fingerprint comes from the Rust runtime. The studio includes:

- A world catalog derived from the packaged examples instead of hardcoded UI data
- A validated World Builder that atomically publishes industrial, city, and ecosystem packages and opens them directly in Play Mode
- A fully engine-backed Play Mode with play, pause, single-step, speed control, restart, live topology, resources, objectives, and event feed
- Crash-safe local save slots with deterministic resume reconstruction and world-fingerprint compatibility checks
- Deterministic production-capacity decisions recorded inside the replay proof chain
- Responsive resource, topology, and event visualizations
- Filterable event audit trails and objective status
- Same-seed determinism checks and cross-seed run comparison
- Server-side performance benchmarks
- Canonical JSON export, copyable proof fingerprints, and local recent-run history
- Bounded high-DPI rendering, bounded dashboard payloads, and proof-only capture for long analytical runs

Use a different catalog or local bind address when needed:

```bash
cargo run -p worldforge-cli -- dashboard --bind 127.0.0.1:9000 --worlds-dir ./examples --saves-dir ./.worldforge/saves
```

For other tools, export the same canonical document directly:

```bash
cargo run -p worldforge-cli -- export examples/ecosystem --ticks 1000 --seed 42 --output ecosystem.json
```

## License

MIT OR Apache-2.0

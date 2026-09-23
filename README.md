# World Forge

**A deterministic, moddable simulation runtime where entire worlds are packages.**

World Forge is a simulation operating system for building games, simulations, worlds, scenarios, challenges, and developer-created content. Every simulation run is fully reproducible: same seed + same world = same result, on every platform, every time.

## Status

**0.2.0 development milestone.** The deterministic runtime, local world inheritance, packaging toolchain, replay verification, mod sandbox, CLI, and local Simulation Studio are implemented and covered by the workspace verification suite. The beta is local-first: the dashboard binds to localhost by default and runs simulations on the same machine.

| Feature | Status | Notes |
|---------|--------|-------|
| Fixed-point math (Q32.32) | ✅ Implemented | `Fixed64` for all simulation-critical values |
| Deterministic RNG | ✅ Implemented | ChaCha8-based with named streams |
| BLAKE3 fingerprinting | ✅ Implemented | State hashing, event chains, package fingerprints |
| Purpose-built ECS | ✅ Implemented | BTreeMap-backed, deterministic iteration |
| World manifest + scenarios | ✅ Implemented | TOML-based with schema validation |
| Resource economy | ✅ Implemented | Production, transfer, conservation, prices |
| City construction | ✅ Implemented | District slots, costs, upkeep, yields, housing, jobs, wellbeing, population growth, co-location synergies |
| Research trees | ✅ Implemented | Data-driven branches, prerequisites, exclusions, unlocks, and systemic multipliers |
| Civic governance | ✅ Implemented | Factions, timed dilemmas, branching mandates, support shifts, defaults, and persistent systemic consequences |
| Objective system | ✅ Implemented | Evaluate pass/fail conditions per scenario |
| Replay artifacts | ✅ Implemented | CBOR-serialized with tamper detection |
| Proof chain | ✅ Implemented | Event hash chains with verification |
| Package system (.world) | ✅ Implemented | Deterministic tar with content fingerprinting |
| Local world inheritance | ✅ Implemented | Recursive sibling resolution, deterministic overlays, dependency locks, cycle and traversal defense |
| Agent framework | ✅ Implemented | Rule-based and utility-based policies |
| Capability-based mod API | ✅ Implemented | Deny-by-default capability system |
| WASM mod runtime | ✅ Implemented | Wasmtime core-Wasm sandbox, fuel/memory limits, capability-gated host ABI |
| Simulation Studio | ✅ Implemented | World Builder, durable saves, Play Mode, live decisions, charts, proofs, comparisons, benchmarks, export |
| Operational analytics | ✅ Implemented | Exact resource extrema, objective progress, stress/growth/recovery signals, resilience scoring |
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

World Forge is a Rust workspace of 13 crates (12 product crates plus the benchmark harness):

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
worldforge-bench       Criterion simulation and stress benchmarks
```

### Design Principles

1. **Determinism is non-negotiable.** `Fixed64` replaces all floating-point in simulation. `ChaCha8Rng` with explicit seeding. `BTreeMap` for all iteration. No threading in the simulation loop.

2. **Worlds are packages.** A world is a directory containing `world.toml`, `scenario.toml`, `entities.toml`, and optionally `city.toml` and mods. Package into `.world` archives with reproducible content fingerprints.

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

## Playable city and research layer

`examples/micro-city` is now a player-directed city rather than a passive scenario. Its four districts have finite land budgets; eight building types consume construction resources and ongoing upkeep; tagged neighbors create local production synergies; housing supports population growth; and jobs and wellbeing respond to the built form.

Nine technologies span knowledge, habitat, ecology, energy, and synthesis. Paths cross-link, unlock new buildings, transform resource yields, and include an exclusive choice between autonomous and human-scale power systems. Four civic factions contest a branching campaign of timed dilemmas: development policy opens different successor crises, shifts support, and permanently rewrites growth, housing, employment, wellbeing, and production. Construction, research, governance, and capacity choices are deterministic events preserved by saves, replay re-execution, state fingerprints, and the proof chain. See [city systems, research, and governance](docs/city-systems.md).

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
- A playable district construction layer and branching research constellation with atomic actions, affordability/slot feedback, and deterministic save/replay restoration
- Responsive resource, topology, and event visualizations
- Exact whole-run resource extrema, measurable objective progress, resilience scoring, and operational insights
- Filterable event audit trails and objective status
- Same-seed determinism checks and cross-seed run comparison
- Server-side performance benchmarks
- Canonical JSON export, copyable proof fingerprints, and local recent-run history
- Bounded high-DPI rendering, bounded dashboard payloads, and proof-only capture for long analytical runs

The chart history is intentionally downsampled for long runs, while the resource ledger's initial, final, minimum, maximum, and extrema ticks are computed on every simulation tick. See [operational analytics](docs/analytics.md) for the metric definitions and scoring model.

Use a different catalog or local bind address when needed:

```bash
cargo run -p worldforge-cli -- dashboard --bind 127.0.0.1:9000 --worlds-dir ./examples --saves-dir ./.worldforge/saves
```

For other tools, export the same canonical document directly:

```bash
cargo run -p worldforge-cli -- export examples/ecosystem --ticks 1000 --seed 42 --output ecosystem.json
```

## World inheritance

Derived worlds can extend one or more sibling packages without copying their entities and links:

```toml
# examples/supply-chain-recovery/world.toml
name = "supply-chain-recovery"
version = "0.2.0"
extends = ["supply-chain"]
```

Parents apply in declaration order and the derived `entities.toml` fragment applies last. Runtime, replay, save, export, and package fingerprints commit to the complete resolved graph. Packaged derived worlds contain a generated `worldforge.lock` with exact dependency fingerprints. See [local world inheritance](docs/inheritance.md).

## License

MIT OR Apache-2.0

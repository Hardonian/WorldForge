# Changelog

## 0.1.0-beta

### Simulation Studio

- Added an operational-insights layer with resilience scoring, stress/growth/recovery signals, exact resource ledgers, and measurable objective progress in completed and live runs.
- Added the eight-entity Coastal Resilience scenario with coupled energy, water, food, medicine, care, and wellbeing systems plus staged disruption and recovery events.
- Added a Tactical CG Game Viewport with continuous 60 FPS animations, camera pan/zoom/fit controls, animated flow conduits, traveling resource packet particles, archetype visual silhouettes, floating delta text, production shockwaves, shortage warnings, holographic HUD cards, and dual CG/Schematic view modes.
- Added 4 high-contrast sci-fi color themes (Cyber Tactical, Solaris Gold, Bio Synthetic, Cryo Vector) with real-time theme engine re-skinning and durable localStorage persistence.
- Added procedural vector SVG world heraldic flags and crests for all scenario worlds, rendered in the sidebar catalog and hero eyebrow banner.
- Added an Entity Avatar Command Module with continuous 60 FPS animated archetype silhouettes, dynamic status pips (active, overdrive, warning), and sector telemetry.
- Added a Tactical Radar Minimap with real-time entity blips, conduits, radar sweep, camera frustum viewport projection, and click-and-drag pan navigation.
- Added a System Vitality Gauge widget with dynamic progress arc calculation and status labels based on objective progress and shortage frequency.
- Added Quick Capacity Presets (0%, 50%, 100%, 150%, 200%) in the Decision Console with electric overdrive spark FX.
- Added persistent Play Mode sessions with pause, step, variable speed, restart, live entity topology, inventory meters, mission status, and world activity feed.
- Added proof-chained player interventions for production capacity, with deterministic incremental runtime execution.
- Added an engine-derived world catalog and health metadata API.
- Added complete run comparison, server-side benchmark, JSON export, recent-run, event filtering, and proof-copy workflows.
- Added responsive navigation, keyboard operation, accessible live states, reduced-motion support, polished charts, animation, and original ambient atlas artwork.
- Added structured API errors, request bounds, traversal protection, concurrent request handling, immutable asset caching, and browser security headers.
- Added dashboard catalog and path-validation regression tests plus a configurable `--worlds-dir` option.
- Added a responsive World Builder with industrial, city, and ecosystem templates, difficulty profiles, typed validation, atomic publication, and immediate Play Mode handoff.
- Added crash-safe local save slots with synchronized writes, backup recovery, strict size/count/content limits, world-fingerprint checks, and deterministic resume parity.

### Release readiness

- Added per-tick resource extrema telemetry that remains exact even when long-run chart history is downsampled, with API contract and regression coverage.
- Documented beta scope, verification commands, security posture, and known limitations in `docs/beta-release.md`.
- Added bounded proof capture with full event-chain parity, 2,048-point snapshot retention, 5,000-event dashboard windows, and accurate full-run aggregates.
- Replaced unbounded request threads with a CPU-sized worker pool and bounded overload queue.
- Added high-density visualization budgets, linear topology lookup, dense-network effect reduction, and explicit performance documentation.
- Fixed Criterion world discovery and added 40-entity stress benchmarks plus bounded-capture regression coverage.
- Added full create → catalog → play → save → resume → complete → replay lifecycle verification.

All notable changes to World Forge will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-09-22

### Added
- 12-crate workspace architecture
- `worldforge-core`: Fixed64 (Q32.32), deterministic RNG (ChaCha8), BLAKE3 fingerprinting, strongly-typed IDs, structured error codes, Tick type, CBOR serialization, SemVer management
- `worldforge-ecs`: Purpose-built deterministic ECS with BTreeMap storage and explicit system scheduling
- `worldforge-world`: TOML-based world manifests, scenario definitions, entity models, event types, objective system
- `worldforge-economy`: Resource production, atomic transfers, conservation invariants, price signals
- `worldforge-proof`: Event hash chains, run proofs, verification reports
- `worldforge-replay`: CBOR replay artifacts with tamper detection and internal verification
- `worldforge-runtime`: Full simulation lifecycle (load → validate → tick → proof)
- `worldforge-agent`: Rule-based and utility-based agent policies with deterministic RNG
- `worldforge-mod-api`: Capability-based mod API with deny-by-default policy
- `worldforge-mod-runtime`: Wasmtime sandbox with fuel/memory limits and capability-gated host calls
- `worldforge-package`: Deterministic .world package format with reproducible fingerprints
- `worldforge-cli`: Commands: doctor, validate, run, test-world, export, benchmark, dashboard, package, replay
- Supply-chain vertical slice example with 5 entities, disruption events, and objectives
- WIT interface definitions for WASM Component Model mods
- JSON schemas for world.toml and scenario.toml
- CI pipeline with cross-platform testing and determinism verification
- Typed, referential validation for entities, links, scenarios, and scheduled events
- Engine-backed dashboard API with real snapshots, events, objectives, and proofs
- Continuous objective evaluation and cumulative production targets
- 80+ unit and integration tests covering all crates

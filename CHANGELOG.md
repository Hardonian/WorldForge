# Changelog

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
- `worldforge-mod-runtime`: Mock mod runtime with lifecycle management (WASM planned for M1)
- `worldforge-package`: Deterministic .world package format with reproducible fingerprints
- `worldforge-cli`: Commands: doctor, validate, run, test-world, package (build/inspect), replay (inspect/verify)
- Supply-chain vertical slice example with 5 entities, disruption events, and objectives
- WIT interface definitions for WASM Component Model mods
- JSON schemas for world.toml and scenario.toml
- CI pipeline with cross-platform testing and determinism verification
- 73+ unit tests covering all crates

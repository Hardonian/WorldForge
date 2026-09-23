# World Forge 0.1.0 Beta Release Guide

World Forge 0.1.0 is a local-first beta of the deterministic simulation runtime and Simulation Studio. The beta is intended for world authors, mod developers, and evaluators running trusted world packages on their own machine.

## Included product surface

- Deterministic fixed-point simulation runtime and ECS
- Typed world, scenario, entity, objective, and event configuration
- Replay artifacts, event hash chains, and run proofs
- Reproducible `.world` packaging and inspection
- Capability-gated WebAssembly mod runtime
- CLI validation, simulation, determinism testing, export, benchmarking, replay, packaging, and diagnostics
- Browser-based Simulation Studio with World Builder, durable saves, interactive Play Mode, live decisions, charts, audit logs, comparisons, benchmarks, proof inspection, and JSON export

## World Builder

Open **World Builder** to create an industrial, city, or ecosystem package. The builder writes to the configured `--worlds-dir` only after the complete temporary package passes manifest, scenario, entity, link, event, and objective validation. Publication is an atomic directory rename, so partially generated worlds never appear in the catalog.

Generated worlds include five connected entities, four resources, a scheduled disruption, and two objectives. Difficulty changes reserves, demand, disruption severity, and objective thresholds. A successful build becomes the selected world and opens directly in Play Mode.

## Play Mode

Open **Play world** from the Simulation Studio navigation. Play Mode creates an isolated engine session and exposes:

- Play, pause, single-tick step, variable playback speed, and deterministic restart
- Live entity topology, aggregate world resources, objectives, and chronological activity
- Producer-capacity decisions from 0% through 200%
- A final BLAKE3 state fingerprint and event-chain proof containing every player decision

Incremental execution is parity-tested against continuous execution: with the same world, seed, duration, and decisions it produces the same final state and event-chain root.

Use **Save** in Play Mode to create a durable local slot under `--saves-dir`. Saves are reconstructed from the seed and ordered proof-chained interventions; they are not opaque memory snapshots. Writes use synchronized temporary files and backup replacement, and startup recovery restores the previous valid slot after an interrupted overwrite. A save will refuse to load if its world package fingerprint has changed.

## Release gate

Run the platform-specific verification script from the repository root:

```powershell
.\scripts\verify.ps1
```

The gate must pass formatting, Clippy, the complete workspace test suite, world validation, deterministic re-runs, and replay verification. The dashboard-specific smoke gate is:

```powershell
cargo test -p worldforge-cli
cargo run -p worldforge-cli -- dashboard
```

Then confirm `http://127.0.0.1:8787/api/health` reports `healthy`, run the flagship Supply Chain world, compare seed 42 with seed 42, and execute a five-repetition benchmark.

## Security posture

- The dashboard binds to `127.0.0.1:8787` by default. Do not expose it to an untrusted network.
- World identifiers are slug-validated and resolved only under the configured catalog directory.
- API bodies are bounded to 64 KiB; simulations are bounded to 1,000,000 ticks; benchmarks are bounded to 20 repetitions.
- Play stepping is bounded to 5,000 ticks per request, sessions expire after four idle hours, and at most 32 sessions may be active.
- Save storage is synchronized, limited to 100 slots and 2 MiB per slot, and rejects malformed interventions or incompatible world fingerprints.
- World creation accepts slug-safe package IDs and publishes only validated packages below the 200-world catalog limit.
- Static responses include a restrictive Content Security Policy, frame denial, MIME sniffing protection, a no-referrer policy, and disabled browser device permissions.
- The WebAssembly runtime has no WASI access and applies capability, fuel, and memory limits.

Report security issues using the private process in [SECURITY.md](../SECURITY.md).

## Known beta constraints

- Active in-memory sessions do not survive a dashboard restart; named save slots do and can reconstruct the exact tick and decision history.
- Recent-run summaries are browser-local; canonical run exports must be downloaded explicitly.
- The server intentionally implements only the small HTTP surface needed by the bundled studio. It is not a general-purpose public web service.
- The dashboard catalog expects unpacked world directories containing `world.toml`, `scenario.toml`, and `entities.toml`.

## Release artifact checklist

1. Run the full verification gate on Windows, Linux, and macOS CI.
2. Confirm the changelog and crate versions match the release tag.
3. Package each example world and inspect its fingerprint.
4. Smoke-test the dashboard at desktop and mobile widths.
5. Create the Git tag only after CI and determinism checks pass.

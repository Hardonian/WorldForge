# World Forge

World Forge is a deterministic, headless simulation runtime where worlds are
portable packages. The current vertical slice models a three-region supply
chain and provides reproducible state fingerprints, replay proofs, deterministic
archives, deterministic agents, and a capability-gated Wasmtime mod boundary.

## Implemented now

- Fixed-tick ECS execution with fixed-point economy math and seeded IDs/RNG.
- Inventory, atomic transfer, production/consumption, shortages, and prices.
- Typed events and objectives included in a BLAKE3 event chain.
- Replay inspect, verify, and local deterministic re-execution.
- Reproducible `.world` tar packages with normalized metadata.
- Versioned WIT contracts plus a minimal core-Wasm v0 host ABI. Mods have no
  WASI access; resource/event calls require explicit capabilities and execution
  has fuel and memory limits.
- `doctor`, `validate`, `run`, `test-world`, `package`, and `replay` CLI flows.

## Quick start

```bash
cargo run -p worldforge-cli -- doctor
cargo run -p worldforge-cli -- run examples/supply-chain --ticks 1000 --seed 42
cargo run -p worldforge-cli -- test-world examples/supply-chain --runs 100
./scripts/smoke.sh
```

Use `cargo run -p worldforge-cli -- --help` for all commands. Formats have
independent versions in `worldforge-core`; the engine is currently pre-1.0.

## Planned later

Remote registries, package signing, a marketplace, multiplayer/distributed
simulation, cloud hosting, graphical authoring, and LLM agents are not
implemented. The WASM host currently uses a compact core-Wasm ABI while the WIT
Component Model binding layer remains the compatibility contract to adopt next.

See [security policy](SECURITY.md), [architecture](docs/architecture/README.md),
and [error codes](docs/error-codes.md).

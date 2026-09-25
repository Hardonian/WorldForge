# World Forge 0.2.0 Stable Release Guide

World Forge 0.2.0 is the official stable release of the deterministic, moddable simulation runtime and Simulation Studio. The platform enables world authors, simulation researchers, and game developers to construct, simulate, and verify complex socio-ecological and industrial systems with 100% cross-platform mathematical determinism.

## Included Product Surface

### 1. Simulation Runtime Core
- **Fixed-Point Arithmetic**: `Fixed64` (Q32.32) replaces all floating-point math in simulation-critical paths to guarantee bit-exact outcomes across hardware architectures and compilers.
- **Deterministic Pseudo-Randomness**: ChaCha8 generator with isolated, named streams for independent subsystem progression.
- **Cryptographic Event Chains**: BLAKE3 hashing of every tick mutation, creating verifiable event hash chains and immutable run proofs.
- **Deterministic ECS**: Entity Component System backed by ordered `BTreeMap` storage, ensuring deterministic iteration orders.
- **Local World Inheritance**: Recursive sibling package resolution (`extends`), fragment overlays, dependency lockfiles (`worldforge.lock`), and strict protection against cyclic or directory-traversal vulnerabilities.
- **Sandboxed WebAssembly Mods**: Wasmtime core-Wasm engine with capability-based security gating (denies network, filesystem, and subprocess execution).

### 2. Living Systems & City Simulation
- **13-Building Isometric Architectural Suite**: Full catalog of residential, civic, agricultural, industrial, and high-tech structures (`courtyard-homes`, `civic-lab`, `maker-cooperative`, `solar-canopy`, `vertical-farm`, `water-garden`, `arcology-spine`, `story-forum`, `fusion-reactor`, `nanotech-foundry`, `grand-amphitheater`, `hyperloop-exchange`, `quantum-observatory`).
- **Tiered Visual Progression**: Distinct Level 1, Level 2, and Level 3 architectural graphics with glowing holographic tactical pill badges (`LVL 2 ★`, `LVL 3 ★`).
- **Environmental Climate & 4-Season Cycle**: Deterministic 60-tick seasons (Spring, Summer, Autumn, Winter) with agricultural yield bonuses, solar power shifts, heating upkeep, and drought/frost risks.
- **Biosphere Health Index**: Real-time modeling of atmospheric air quality, water table purity, and soil fertility based on heavy industrial output versus green infrastructure.
- **Atmospheric Weather VFX**: Real-time canvas rain streaks, surface puddles, expanding splash rings, storm lightning flashes, winter snow, and heatwave shimmer.
- **Autonomous Logistics & Air Life**: Quad-rotor hovering sky drones with downward scanning lasers, propeller blur, blinking LEDs, and cargo crates; ground haulers transporting resource packets with floating badges.

### 3. Strategy, Geopolitics & Governance
- **Grand Realm War Room**: Tactical radar grid backdrop, threat alert pulses, and multi-polar diplomacy across 5 political entities (*Barony of Oakhaven*, *Riverside Protectorate*, *Iron Mandate Hegemony*, *Free Mercantile League*, *Dust Canyon Raiders*).
- **Sensory Immersion**: Procedurally synthesized Web Audio SFX (military war horns, battle impact clash, tribute coin chimes, and tactical sirens).
- **Hall of Triumphs**: 10 milestone trophies with golden holographic sheen animations and cyber-blueprint locked states.
- **Civic Factions & Branching Mandates**: Timed dilemmas, faction support dynamics, and persistent city policy transformations.

### 4. Specialized Scenario Visualizers
- **Coastal Resilience Offshore Seascape**: Dynamic ocean wave swells, surf foam, storm surge floodgates, and rotating wind turbines.
- **Wilderness Predator-Prey Visualizer**: Trophic web simulation tracking savanna vegetation, grazing deer herds, and hunting wolf packs.

---

## Release Verification Gate

Run the platform verification script from the repository root:

```powershell
.\scripts\verify.ps1
```

The release gate enforces:
1. Workspace compilation across all 13 crates (`cargo check --workspace`)
2. Comprehensive unit and integration testing (`cargo test --workspace`)
3. Formatting compliance (`cargo fmt --all -- --check`)
4. Zero Clippy warnings (`cargo clippy --workspace --all-targets -- -D warnings`)
5. Engine health verification (`worldforge doctor`)
6. Validation of all 8 example worlds (`examples/`)
7. Determinism verification over 1,000+ ticks (`worldforge test-world`)
8. CBOR replay serialization, verification, and bit-exact re-execution
9. Canonical JSON snapshot export

# City systems and research

World Forge worlds may include an optional `city.toml`. This turns the deterministic resource network into a player-directed city: construction and research consume real inventory, buildings participate in the tick economy, and every decision is included in saves, replays, event-chain proofs, and state fingerprints.

## Gameplay loop

1. Build inside finite district slot budgets.
2. Pay construction costs from the configured treasury entity.
3. Sustain each building's per-tick upkeep or lose its output for that tick.
4. Combine tagged buildings in one district to activate configured output synergies.
5. Generate research and choose paths through the technology graph.
6. Use technology effects to alter building- and resource-specific yields, housing, jobs, and wellbeing.
7. Grow population toward constructed housing capacity while paying per-resident needs.

Research supports prerequisites, cross-branch synthesis, and mutually exclusive choices. The engine validates references and rejects prerequisite cycles before a world can run.

## Runtime contract

`city.toml` identifies one existing entity as its treasury. Player actions are atomic: the engine verifies unlocks, exclusions, affordability, district compatibility, free slots, and count limits before changing state. Fixed-point accounting is used after configuration values enter the simulation.

The live API exposes the effective city state in `city` and accepts:

- `POST /api/play/sessions/{id}/construct` with `{ "building": "...", "district": "..." }`
- `POST /api/play/sessions/{id}/research` with `{ "technology": "..." }`

The existing intervention endpoint remains available for production-capacity decisions. All three action types are restored in deterministic tick order when loading a save.

## Content inheritance

A derived world inherits its last resolved base world's `city.toml`. A local `city.toml` replaces that city layer as a complete ruleset, while the normal entity graph still uses named overlays. This keeps research graphs internally coherent and makes an intentional ruleset fork obvious in package review.

See `examples/micro-city/city.toml` for a full four-district, nine-technology world with adjacency-like tag synergies and an exclusive energy-policy fork. `schemas/city.schema.json` documents the serialized shape; engine validation remains authoritative for graph and cross-file constraints.

# Local world inheritance

World Forge 0.2 can compose a world from sibling packages in the same local catalog. The feature is deliberately local and closed: dependency references are directory IDs, not paths or network coordinates.

## Declaring bases

```toml
name = "supply-chain-recovery"
version = "0.2.0"
extends = ["supply-chain"]
```

If the derived world is `examples/supply-chain-recovery`, the reference resolves only to `examples/supply-chain`. IDs accept lowercase ASCII letters, digits, hyphens, and underscores. Absolute paths, separators, traversal, remote-style references, symlink escapes, duplicate bases, cycles, and graphs deeper than 16 levels fail before execution.

## Merge rules

Base worlds apply from left to right; the derived world's `entities.toml` applies last.

- An entity with a new name is appended.
- An entity with an inherited name replaces that complete entity definition while retaining its deterministic position.
- A link with a new `(from, to, resource)` identity is appended.
- A link with an inherited identity replaces that complete link definition.
- Fragment links may reference entities supplied by a base world.
- The derived world owns its scenario. Parent events and objectives are not implicitly copied.

The fully merged entity and link graph is validated after composition, so dangling references and invalid numeric values still fail before runtime initialization.

## Fingerprints and locks

The effective world fingerprint hashes the derived package plus a deterministic lock document containing each direct dependency's name, version, reference, and effective fingerprint. Because dependency fingerprints are recursive, a change anywhere in the base graph changes the derived fingerprint.

`package build` adds this document as `worldforge.lock` inside a derived `.world` archive. An unpacked package may retain the generated lock; resolution verifies it byte-for-byte against the current dependency graph. A stale or unnecessary lock fails closed. The package fingerprint therefore matches the effective runtime fingerprint.

Replay re-execution and durable save loading compare effective fingerprints. Modifying a base after a run safely invalidates artifacts created from the previous graph rather than silently reconstructing a different world.

## Current boundary

Remote dependency registries are intentionally unsupported. Inherited mod declarations are detected across the graph, but manifest-driven mod loading remains a separate milestone and fails closed at runtime.

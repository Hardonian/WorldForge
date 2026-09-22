# Architecture

`worldforge-core` owns IDs, fixed point, RNG, hashes, versions, and errors.
`worldforge-ecs` owns deterministic storage/scheduling. `worldforge-world` owns
manifests, scenarios, events, objectives, and canonical components.
`worldforge-economy` implements resource mechanics. `worldforge-runtime`
coordinates ticks and produces results. `worldforge-proof` and
`worldforge-replay` provide hash chains and artifacts. `worldforge-package`
builds reproducible archives. `worldforge-mod-api` and
`worldforge-mod-runtime` define and enforce the mod boundary.
`worldforge-agent` supplies deterministic policies, and `worldforge-cli` is the
headless operator surface.

Format versions (world, replay, package, WIT) evolve separately from the engine
version. World inheritance is a manifest contract using `extends`; only local
content is executed today and no remote resolver is present.

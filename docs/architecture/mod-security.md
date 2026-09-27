# Mod security

Mods receive only `worldforge` host functions linked by the runtime. The v0 ABI
currently exposes resource reads and numeric event emission. Each call checks an
explicit `CapabilityPolicy`; all capabilities default to denied. No WASI context
is linked, so filesystem, sockets, environment, clocks, processes, shell, and
secrets are unavailable. Unknown imports fail instantiation.

Every lifecycle call receives a deterministic fuel allowance. Store limits cap
linear memory and instance/memory counts. A trap, fuel exhaustion, missing
import, or capability denial returns a structured `WF2xxx` error without
terminating the simulation host. Manifest module paths and capability-grant
sidecars are canonicalized beneath their declaring world, including inherited
worlds, before loading. The exact module and grant bytes are fingerprinted into
run proofs and replay artifacts.

The runtime deliberately has no external imports that can block independently
of Wasm execution, so deterministic fuel is the execution-time boundary.
Publisher identity/signature infrastructure and remote mod registries are
outside the local-first 0.2 trust model; the sandbox should not be described as
a production multi-tenant security boundary.

The versioned WIT files describe the intended Component Model surface. The
current executable example uses the smaller core-Wasm ABI documented in
`examples/mods/read-emit.wat`.

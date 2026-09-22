# Mod security

Mods receive only `worldforge` host functions linked by the runtime. The v0 ABI
currently exposes resource reads and numeric event emission. Each call checks an
explicit `CapabilityPolicy`; all capabilities default to denied. No WASI context
is linked, so filesystem, sockets, environment, clocks, processes, shell, and
secrets are unavailable. Unknown imports fail instantiation.

Every lifecycle call receives a deterministic fuel allowance. Store limits cap
linear memory and instance/memory counts. A trap, fuel exhaustion, missing
import, or capability denial returns a structured `WF2xxx` error without
terminating the simulation host. Wall-clock interruption and signed mod
provenance are not yet implemented, so the sandbox should not be described as a
production multi-tenant security boundary.

The versioned WIT files describe the intended Component Model surface. The
current executable example uses the smaller core-Wasm ABI documented in
`examples/mods/read-emit.wat`.

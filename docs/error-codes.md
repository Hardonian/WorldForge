# Error codes

- `WF1001` world manifest invalid; `WF1004` world schema violation.
- `WF2004` mod capability denied; `WF2006` fuel exhausted; `WF2007` WASM trap.
- `WF3002` replay hash mismatch; `WF3003` incompatible version; `WF3005` replay fingerprint mismatch.
- `WF4001` resource invariant failed; `WF4002` insufficient resource; `WF4003` transfer failed.
- `WF5002` runtime tick failed; `WF5005` simulation degraded.
- `WF6001` package build failed; `WF6002` package invalid; `WF6004` dependency missing, unsafe, cyclic, or deeper than the supported local inheritance limit.

The Rust enum in `worldforge-core/src/error.rs` is authoritative.

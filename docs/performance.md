# Performance and scalability

World Forge separates deterministic simulation work from presentation capture. A run can use either:

- **Full replay capture** for playable sessions, replay export, and forensic CLI exports. Every event is retained.
- **Bounded proof capture** for dashboards and benchmarks. Every event is counted and included in the cryptographic hash chain, while only the newest event window is held in memory.

Both modes produce the same final-state fingerprint and event-chain root. Regression tests enforce this parity.

## Production budgets

| Surface | Budget | Behavior at the limit |
|---|---:|---|
| Runtime chart history | 2,048 snapshots | Deterministically samples the run and always keeps ticks 0 and final |
| Dashboard run events | 5,000 newest events | Full aggregate counts and proof remain exact |
| Interactive step events | 256 newest events | Replay capture remains complete |
| Browser network nodes | 256 nodes | Visualization is bounded; simulation state is not |
| Browser network links | 768 links | Dense-mode effects are disabled to protect frame time |
| Server workers | 2–16, based on CPU availability | Fixed pool prevents unbounded thread creation |
| Pending HTTP work | 128 requests | Excess requests receive `503 SERVER_BUSY` |
| Active play sessions | 32 | Additional sessions are rejected until capacity is available |

Canvas backing stores are device-pixel-ratio aware and capped at 2×. This keeps charts crisp on high-density displays without allowing extreme display scaling to multiply GPU memory uncontrollably. Static network lookup is linear rather than quadratic.

## Reference stress result

On the development machine, the 40-entity `examples/stress-test` world at 5,000 ticks produced 289,410 events:

- Core proof-only benchmark: about **471 ms**, **10,600 ticks/sec**, and **615,000 events/sec** (three-run release average).
- Dashboard HTTP response: about **585 ms** and **1.16 MB**, returning 5,000 events and 1,668 chart snapshots while preserving totals and proof data for the complete run.

These values are a local reference, not a cross-machine guarantee. Use release builds for meaningful measurements:

```powershell
cargo build --release -p worldforge-cli
.\target\release\worldforge.exe benchmark examples\stress-test --ticks 5000 --reps 5
cargo bench -p worldforge-bench --bench simulation
```

## Cost model

Simulation work is deterministic and single-threaded per world. Independent HTTP requests and sessions run across a bounded worker pool. This avoids synchronization inside a world while scaling separate runs across available CPU cores.

Full replay capture intentionally scales with event count because it is the lossless audit artifact. Use `SimulationRuntime::load_bounded` for previews, dashboards, AI evaluation batches, and benchmarks that need exact proofs but not a downloadable event-by-event replay.


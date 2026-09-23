# Operational analytics

World Forge analytics are derived from deterministic engine state; the browser does not reconstruct simulation results. Completed-run exports expose `resourceMetrics` and detailed `objectives`, and incremental Play Mode responses expose the same fields for the current tick.

## Resource metrics

Each tracked resource reports:

- `initial` and `finalLevel`
- `minimum` and the exact `minimumTick`
- `maximum` and the exact `maximumTick`
- `netChange` (`finalLevel - initial`)

The runtime updates extrema after every tick. These values remain exact when chart history is downsampled to the bounded visualization budget.

## Objective progress

Every objective reports a stable `kind`, optional `resource`, measured `current` value, `target`, and normalized `progress` from 0 through 1.

- `maintain_inventory` uses the lowest observed whole-world inventory.
- `avoid_shortage` uses the number of matching shortage events and is complete only at zero.
- `reach_production_target` uses cumulative production.
- `reach_inventory_target` uses the highest observed inventory because reaching the target is durable once achieved.
- `survive_until_tick` uses elapsed ticks.

The authoritative pass, pending, or fail state remains the objective `status`; progress is explanatory telemetry and does not replace scenario evaluation.

## Studio interpretation

The Studio's resilience score weights objective completion at 60% and shortage pressure at 40%. Shortage pressure is the shortage-event count divided by elapsed ticks, capped at 100%. The score is an operator-friendly summary, not part of the deterministic proof or a scenario win condition.

The insight cards derive:

- **Most stressed:** largest decline from initial to minimum inventory.
- **Strongest gain:** largest positive net resource change.
- **Best recovery:** largest rise from minimum to final inventory.
- **World activity:** total events per simulated tick.

Canonical exports include the underlying measurements so external tools can apply a different interpretation without rerunning the simulation.

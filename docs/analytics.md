# Operational Analytics & Monte Carlo Risk Modeling

World Forge analytics are derived strictly from deterministic engine state using bit-exact fixed-point (`Fixed64`) arithmetic; neither the runtime nor the Studio dashboard relies on floating-point nondeterminism. Analytics operate across two complementary modalities:
1. **Single-Run Deterministic Audit:** Exact per-tick extrema, objective progress metrics, resource ledgers, and tamper-evident event chains.
2. **Multi-Seed Monte Carlo Risk Analysis:** Automated batch sweeps executing parametric seed sequences to compute statistical confidence envelopes, systemic bottleneck rankings, link/resource elasticity metrics, and cross-resource Pearson correlation heatmaps.

---

## 1. Single-Run Operational Metrics

### Resource Extrema & Ledgers
Each tracked resource records continuous telemetry across all simulation ticks:
- `initial` and `finalLevel`: Starting inventory vs final tick inventory.
- `minimum` and `minimumTick`: Exact global minimum inventory and the tick it occurred.
- `maximum` and `maximumTick`: Exact global maximum inventory and the tick it occurred.
- `netChange`: Net production/consumption differential (`finalLevel - initial`).

The runtime updates extrema after every tick. These values remain exact even when time-series history is downsampled to bounded visualization budgets.

### Objective Progress Telemetry
Every scenario objective reports a stable `kind`, optional `resource`, measured `current` value, `target`, and normalized `progress` from 0 through 1:
- `maintain_inventory`: Tracks the lowest observed whole-world inventory against the minimum floor.
- `avoid_shortage`: Counts matching shortage events; complete only when zero shortages occur.
- `reach_production_target`: Evaluates cumulative units produced across all producers.
- `reach_inventory_target`: Evaluates the highest observed inventory (durable once achieved).
- `survive_until_tick`: Evaluates elapsed simulation ticks.

Authoritative scenario completion remains the objective `status` (`Passed`, `Failed`, `Pending`); progress is explanatory telemetry.

### Studio Resilience Score
The Studio resilience score provides an operator-friendly health index from 0 to 100:
$$\text{Resilience} = \text{round}(0.60 \times \text{AvgObjectiveProgress} - 0.40 \times \text{ShortagePressure}) \times 100$$
where shortage pressure is the count of shortage events divided by elapsed ticks (capped at 100%).

---

## 2. Multi-Seed Monte Carlo Risk Modeling

The `worldforge analyze` command and Studio Risk Lab execute multi-seed Monte Carlo sweeps to evaluate system resilience under stochastic variability.

### CLI Usage
```bash
# Run a 20-run Monte Carlo sweep across 500 ticks
worldforge analyze examples/supply-chain --runs 20 --ticks 500 --seed 42

# Export deterministic JSON risk report for CI/CD pipelines
worldforge analyze examples/supply-chain --runs 10 --ticks 1000 --format json
```

### Deterministic Fixed-Point Statistics (`crates/worldforge-core/src/stats.rs`)
All statistical metrics are computed using 64-bit fixed-point (`Fixed64`) with integer arithmetic to preserve bit-exact reproducibility across OS architectures:
- **Integer Square Root (`checked_sqrt`):** Computes $\sqrt{x}$ on Q32.32 values by scaling the underlying integer representation by 32 bits and taking `u128::isqrt`, guaranteeing zero float rounding errors.
- **Percentiles ($p_{10}, p_{25}, p_{50}, p_{75}, p_{90}$):** Computed via linear rank-index interpolation over sorted integer samples.
- **Sample Variance & Standard Deviation:** Computed using two-pass mean-deviation summation in `Fixed64`.
- **Pearson Correlation Matrix:** Cross-resource Pearson correlation coefficient $r_{xy} = \frac{\text{cov}(X, Y)}{\sigma_X \sigma_Y}$ bounded between $-1.00$ and $+1.00$.

---

## 3. Confidence Envelopes & Fan Charts

For every tracked resource, Monte Carlo sweeps generate temporal confidence bands downsampled across regular tick intervals:
- **$p_{10} - p_{90}$ (Outer Envelope):** 80% confidence interval representing extreme variability bounds.
- **$p_{25} - p_{75}$ (Inner Envelope):** 50% interquartile range representing standard operational flow.
- **Median ($p_{50}$):** The central expected inventory trajectory.
- **Min / Max:** Empirical bounding extrema across all seeds.

In the Simulation Studio **Risk Lab**, this is visualized as an interactive Canvas 2D Fan Chart with live resource switching and automatic scale fitting.

---

## 4. Systemic Bottleneck Telemetry

Bottleneck diagnosis isolates friction points throttling downstream entities. Each entity is scored via:

$$\text{Severity} = (\text{StarvationRatio} \times 100 \times 0.6) + (\text{ShortageRatio} \times 100 \times 0.2) + (\text{CongestionRatio} \times 100 \times 0.2)$$

### Diagnostic Categorization
- **Critical Failure ($\text{Severity} \ge 25.0$):** Severe cascade; starvation throttles downstream networks continuously.
- **Moderate Friction ($8.0 \le \text{Severity} < 25.0$):** Intermittent input deficits dampening throughput.
- **Optimal Flow ($\text{Severity} < 8.0$):** Inputs and transport maintained at nominal capacity.

---

## 5. Elasticity & Buffer Runway Telemetry

### Resource Buffer Elasticity
- **Burn Rate ($B$):** Average consumption per tick across all consumers.
- **Production Rate ($P$):** Average generation per tick across all producers.
- **Replenishment Ratio ($R$):** Ratio $P / B$. Ratios $< 1.0$ indicate structural deficit.
- **Buffer Runway Ticks:** Expected ticks before complete inventory exhaustion:
  $$\text{Runway} = \frac{\text{CurrentInventory}}{\max(0.01, B - P)}$$
- **Volatility:** Normalized standard deviation of inventory over time ($\sigma / \mu$).

### Link Elasticity & Congestion
- **Utilization:** Total transferred units divided by nominal max link capacity.
- **Saturated Ticks:** Number of ticks where transfer volume was capped at link bandwidth.
- **Congestion Ratio:** Saturated ticks divided by total active ticks. Ratios $> 30\%$ indicate systemic distribution choke points.

---

## 6. Simulation Studio "Risk Lab" & REST API

The Simulation Studio provides an interactive Risk Lab (`#nav-analytics`):
- **Endpoint:** `POST /api/analyze`
  ```json
  {
    "world": "supply-chain",
    "runs": 15,
    "ticks": 500,
    "seed": 42
  }
  ```
- **Response:** Complete `MonteCarloReport` containing resilience distribution, objective pass probabilities, confidence bands per resource, bottleneck rankings, elasticity tables, and cross-resource correlation matrices.
- **Visuals:** High-DPI HTML5 Canvas fan charts, color-coded bottleneck severity meters, and correlation heatmap tables.

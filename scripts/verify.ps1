# verify.ps1 — Run the full World Forge verification suite (Windows)
# Usage: .\scripts\verify.ps1

$ErrorActionPreference = 'Stop'

Write-Host "=== World Forge Verification Suite ===" -ForegroundColor Cyan
Write-Host ""

# 1. Check workspace
Write-Host "▸ Checking workspace..." -ForegroundColor Yellow
cargo check --workspace
if ($LASTEXITCODE -ne 0) { throw "Workspace check failed" }
Write-Host "  ✓ Workspace compiles" -ForegroundColor Green
Write-Host ""

# 2. Run all tests
Write-Host "▸ Running tests..." -ForegroundColor Yellow
cargo test --workspace
if ($LASTEXITCODE -ne 0) { throw "Tests failed" }
Write-Host "  ✓ All tests pass" -ForegroundColor Green
Write-Host ""

Write-Host "▸ Checking formatting and lints..." -ForegroundColor Yellow
cargo fmt --all -- --check
if ($LASTEXITCODE -ne 0) { throw "Format check failed" }
cargo clippy --workspace --all-targets -- -D warnings
if ($LASTEXITCODE -ne 0) { throw "Clippy failed" }
Write-Host "  ✓ Formatting and lints pass" -ForegroundColor Green
Write-Host ""

# 3. Doctor
Write-Host "▸ Running doctor..." -ForegroundColor Yellow
cargo run -p worldforge-cli --quiet -- doctor
if ($LASTEXITCODE -ne 0) { throw "Doctor check failed" }
Write-Host ""

# 4. Validate example worlds
Write-Host "▸ Validating example worlds..." -ForegroundColor Yellow
Get-ChildItem examples -Directory | Where-Object { Test-Path (Join-Path $_.FullName 'world.toml') } | ForEach-Object {
    cargo run -p worldforge-cli --quiet -- validate $_.FullName
    if ($LASTEXITCODE -ne 0) { throw "Validation failed: $($_.Name)" }
}
Write-Host "  ✓ All example worlds valid" -ForegroundColor Green
Write-Host ""

# 5. Determinism test
Write-Host "▸ Running determinism test (10 runs)..." -ForegroundColor Yellow
cargo run -p worldforge-cli --quiet -- test-world examples/supply-chain --runs 10 --ticks 1000
if ($LASTEXITCODE -ne 0) { throw "Determinism test failed" }
Write-Host ""

# 6. Run and verify replay
Write-Host "▸ Running simulation..." -ForegroundColor Yellow
cargo run -p worldforge-cli --quiet -- run examples/supply-chain --seed 42 --ticks 500
if ($LASTEXITCODE -ne 0) { throw "Simulation run failed" }
Write-Host ""

Write-Host "▸ Verifying replay..." -ForegroundColor Yellow
cargo run -p worldforge-cli --quiet -- replay verify examples/supply-chain/last.replay
if ($LASTEXITCODE -ne 0) { throw "Replay verification failed" }
cargo run -p worldforge-cli --quiet -- replay run examples/supply-chain/last.replay
if ($LASTEXITCODE -ne 0) { throw "Replay re-execution failed" }
Write-Host ""

Write-Host "▸ Verifying canonical JSON export..." -ForegroundColor Yellow
cargo run -p worldforge-cli --quiet -- export examples/supply-chain --seed 42 --ticks 25 --output target/supply-chain-export.json
if ($LASTEXITCODE -ne 0) { throw "Export failed" }
Write-Host ""

Write-Host "=== All verification checks passed ✓ ===" -ForegroundColor Green

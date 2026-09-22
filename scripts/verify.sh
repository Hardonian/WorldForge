#!/usr/bin/env bash
# verify.sh — Run the full World Forge verification suite
# Usage: ./scripts/verify.sh
set -euo pipefail

echo "=== World Forge Verification Suite ==="
echo ""

# 1. Check workspace
echo "▸ Checking workspace..."
cargo check --workspace
echo "  ✓ Workspace compiles"
echo ""

# 2. Run all tests
echo "▸ Running tests..."
cargo test --workspace
echo "  ✓ All tests pass"
echo ""

# 3. Format check
echo "▸ Checking formatting..."
cargo fmt --all -- --check
echo "  ✓ Code is formatted"
echo ""

# 4. Clippy
echo "▸ Running clippy..."
cargo clippy --workspace -- -D warnings
echo "  ✓ No clippy warnings"
echo ""

# 5. Doctor
echo "▸ Running doctor..."
cargo run -p worldforge-cli --quiet -- doctor
echo ""

# 6. Validate example worlds
echo "▸ Validating example worlds..."
cargo run -p worldforge-cli --quiet -- validate examples/supply-chain
cargo run -p worldforge-cli --quiet -- validate examples/minimal-world
echo "  ✓ All example worlds valid"
echo ""

# 7. Determinism test
echo "▸ Running determinism test (10 runs)..."
cargo run -p worldforge-cli --quiet -- test-world examples/supply-chain --runs 10 --ticks 1000
echo ""

# 8. Run simulation and verify replay
echo "▸ Running simulation..."
cargo run -p worldforge-cli --quiet -- run examples/supply-chain --seed 42 --ticks 500
echo ""

echo "▸ Verifying replay..."
cargo run -p worldforge-cli --quiet -- replay verify examples/supply-chain/last.replay
echo ""

echo "=== All verification checks passed ✓ ==="

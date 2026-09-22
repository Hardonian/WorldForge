#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo build --workspace --release
cargo run -q -p worldforge-cli -- test-world examples/supply-chain --runs 100 --ticks 1000
./scripts/smoke.sh

first=$(cargo run -q -p worldforge-cli -- package fingerprint examples/supply-chain)
cargo run -q -p worldforge-cli -- package build examples/supply-chain --output target/repro-a.world >/dev/null
cargo run -q -p worldforge-cli -- package build examples/supply-chain --output target/repro-b.world >/dev/null
a=$(cargo run -q -p worldforge-cli -- package fingerprint target/repro-a.world)
b=$(cargo run -q -p worldforge-cli -- package fingerprint target/repro-b.world)
test "$first" = "$a"
test "$a" = "$b"

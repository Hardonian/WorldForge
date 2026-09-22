#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

CLI=(cargo run -q -p worldforge-cli --)
WORLD=examples/supply-chain
PACKAGE=target/supply-chain-smoke.world

"${CLI[@]}" validate "$WORLD"
"${CLI[@]}" run "$WORLD" --ticks 1000 --seed 42
"${CLI[@]}" replay verify "$WORLD/last.replay"
"${CLI[@]}" replay run "$WORLD/last.replay"
"${CLI[@]}" package build "$WORLD" --output "$PACKAGE"
"${CLI[@]}" package inspect "$PACKAGE"
"${CLI[@]}" package fingerprint "$PACKAGE"

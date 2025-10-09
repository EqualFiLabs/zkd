#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)"
OUT_DIR="$ROOT_DIR/tests/golden_vectors/benchmarks"

mkdir -p "$OUT_DIR"

timestamp=$(date -u +"%Y%m%dT%H%M%SZ")
log_file="$OUT_DIR/bench_${timestamp}.json"

echo "[vector] benchmarking proof generation across backends"
cargo run -p zkprov-cli -- vector bench --root "$ROOT_DIR/tests/golden_vectors" --out "$log_file" "$@"

echo "[vector] wrote benchmark log to $log_file"

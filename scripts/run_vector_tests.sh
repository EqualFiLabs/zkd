#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)"

echo "[vector] regenerating golden vectors"
cargo run -p zkprov-cli -- vector regenerate --root "$ROOT_DIR/tests/golden_vectors" "$@"

echo "[vector] validating parity across backends"
cargo run -p zkprov-cli -- vector check --root "$ROOT_DIR/tests/golden_vectors" "$@"

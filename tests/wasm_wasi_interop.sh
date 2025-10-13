#!/usr/bin/env bash
set -euo pipefail

TARGET=wasm32-wasi
if ! rustc --print target-list | grep -q "^${TARGET}$"; then
  TARGET=wasm32-wasip1
fi

rustup target add "${TARGET}" >/dev/null
cargo build -p zkprov-ffi-c --target "${TARGET}" --release
mkdir -p bindings/wasm
artifact="target/${TARGET}/release/zkprov.wasm"
if [[ ! -f "${artifact}" ]]; then
  echo "error: expected wasm artifact at ${artifact}" >&2
  exit 1
fi
cp "${artifact}" bindings/wasm/zkprov_wasi.wasm
node bindings/wasm/smoke_node.mjs

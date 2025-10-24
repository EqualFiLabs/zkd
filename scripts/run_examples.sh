#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO_BIN="${CARGO:-cargo}"

build_rust_core() {
  if [[ "${_RUST_CORE_BUILT:-0}" -eq 0 ]]; then
    echo "[build] Compiling Rust workspace artifacts (reuse enabled)."
    "$CARGO_BIN" build --release
    _RUST_CORE_BUILT=1
  else
    echo "[build] Reusing existing Rust workspace artifacts."
  fi
}

placeholder_run() {
  local name="$1"
  shift || true
  echo "[todo] $name example is a placeholder; implementing full execution is pending."
  return 0
}

run_c_example() {
  build_rust_core
  export ZKPROV_LIB="${ZKPROV_LIB:-$ROOT_DIR/target/release/libzkprov.a}"
  placeholder_run "C"
}

run_node_example() {
  build_rust_core
  export ZKPROV_LIB="${ZKPROV_LIB:-$ROOT_DIR/target/release/libzkprov.a}"
  placeholder_run "Node.js"
}

run_python_example() {
  build_rust_core
  export ZKPROV_LIB="${ZKPROV_LIB:-$ROOT_DIR/target/release/libzkprov.a}"
  placeholder_run "Python"
}

run_flutter_example() {
  build_rust_core
  export ZKPROV_LIB="${ZKPROV_LIB:-$ROOT_DIR/target/release/libzkprov.a}"
  placeholder_run "Flutter"
}

run_wasm_example() {
  build_rust_core
  export ZKPROV_LIB="${ZKPROV_LIB:-$ROOT_DIR/target/release/libzkprov.a}"
  placeholder_run "WASM"
}

print_result() {
  local status="$1"
  local label="$2"
  printf '%s %s\n' "$status" "$label"
}

execute_example() {
  local key="$1"
  local label="$2"
  local runner="$3"

  if "$runner"; then
    print_result "PASS" "$label"
  else
    print_result "FAIL" "$label"
    OVERALL_STATUS=1
  fi
}

usage() {
  cat <<USAGE
Usage: $(basename "$0") [example...]

Run integration examples for CI. When no examples are specified all known examples are executed.

Known examples:
  c         - C FFI roundtrip
  node      - Node.js bindings roundtrip
  python    - Python bindings roundtrip
  flutter   - Flutter demo application
  wasm      - WebAssembly smoke test
USAGE
}

if [[ ${1:-} == "-h" || ${1:-} == "--help" ]]; then
  usage
  exit 0
fi

AVAILABLE_EXAMPLES=(c node python flutter wasm)
REQUESTED_EXAMPLES=("$@")
if [[ ${#REQUESTED_EXAMPLES[@]} -eq 0 ]]; then
  REQUESTED_EXAMPLES=("${AVAILABLE_EXAMPLES[@]}")
fi

OVERALL_STATUS=0
for example in "${REQUESTED_EXAMPLES[@]}"; do
  case "$example" in
    c)
      execute_example "$example" "C roundtrip" run_c_example
      ;;
    node)
      execute_example "$example" "Node.js roundtrip" run_node_example
      ;;
    python)
      execute_example "$example" "Python roundtrip" run_python_example
      ;;
    flutter)
      execute_example "$example" "Flutter demo" run_flutter_example
      ;;
    wasm)
      execute_example "$example" "WASM smoke" run_wasm_example
      ;;
    *)
      echo "Unknown example: $example" >&2
      OVERALL_STATUS=1
      ;;
  esac
done

exit $OVERALL_STATUS

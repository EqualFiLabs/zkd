#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO_BIN="${CARGO:-cargo}"
TARGET_ROOT="${CARGO_TARGET_DIR:-${ROOT_DIR}/target}"
OS_NAME="$(uname -s)"

case "$OS_NAME" in
  Linux*)    SHARED_NAME="libzkprov.so"; STATIC_NAME="libzkprov.a"; LIB_ENV="LD_LIBRARY_PATH"; EXE_SUFFIX="";;
  Darwin*)   SHARED_NAME="libzkprov.dylib"; STATIC_NAME="libzkprov.a"; LIB_ENV="DYLD_LIBRARY_PATH"; EXE_SUFFIX="";;
  MINGW*|MSYS*|CYGWIN*|Windows_NT)
             SHARED_NAME="zkprov.dll"; STATIC_NAME="zkprov.lib"; LIB_ENV="PATH"; EXE_SUFFIX=".exe";;
  *) echo "Unsupported platform: $OS_NAME" >&2; exit 1;;
esac

info() {
  printf '[examples] %s\n' "$*"
}

have_cmd() {
  command -v "$1" >/dev/null 2>&1
}

resolve_artifact() {
  local name="$1"
  local path
  path="$(find "$TARGET_ROOT" -type f -path "*/release/$name" -print -quit 2>/dev/null || true)"
  if [[ -z "$path" ]]; then
    echo "Artifact not found: $name (searched under $TARGET_ROOT)" >&2
    return 1
  fi
  printf '%s\n' "$path"
}

build_rust_core() {
  if [[ "${_RUST_CORE_BUILT:-0}" -eq 0 ]]; then
    info "Building zkprov-ffi-c (release)."
    "$CARGO_BIN" build -p zkprov-ffi-c --release
    _RUST_CORE_BUILT=1
  else
    info "Reusing cached release artifacts."
  fi
}

build_node_binding() {
  local static_lib
  static_lib="$(resolve_artifact "$STATIC_NAME")" || return 1
  export ZKPROV_STATIC="$static_lib"
  (
    cd "$ROOT_DIR/bindings/node"
    if [[ ! -d node_modules ]]; then
      info "Installing Node dependencies (bindings/node)."
      npm ci
    fi
    info "Building Node addon with node-gyp."
    npx node-gyp rebuild
  )
}

setup_python_env() {
  if [[ "${_PY_ENV_READY:-0}" -eq 1 ]]; then
    return 0
  fi
  local py_bin
  if have_cmd python3; then
    py_bin=python3
  elif have_cmd python; then
    py_bin=python
  else
    echo "Python interpreter not found." >&2
    return 1
  fi

  PY_VENV_DIR="${PY_VENV_DIR:-$TARGET_ROOT/examples-python-venv}"
  local use_user_site=0

  if [[ ! -d "$PY_VENV_DIR" ]]; then
    info "Creating Python virtual environment."
    if ! "$py_bin" -m venv "$PY_VENV_DIR"; then
      info "Python venv module unavailable; falling back to user-site installation."
      use_user_site=1
    fi
  fi

  if [[ "$use_user_site" -eq 0 && -d "$PY_VENV_DIR" ]]; then
    local activate_path
    if [[ "$OS_NAME" == "Windows_NT" || "$OS_NAME" == MINGW* || "$OS_NAME" == MSYS* || "$OS_NAME" == CYGWIN* ]]; then
      activate_path="$PY_VENV_DIR/Scripts/activate"
    else
      activate_path="$PY_VENV_DIR/bin/activate"
    fi

    if [[ -f "$activate_path" ]]; then
      # shellcheck disable=SC1090
      source "$activate_path"
      python -m pip install --upgrade pip >/dev/null
      python -m pip install -e "$ROOT_DIR/bindings/python" >/dev/null
      PYTHON_RUNNER="python"
      PYTHON_USER_SITE=""
    else
      info "Virtual environment activation script missing; using user-site installation instead."
      use_user_site=1
    fi
  fi

  if [[ "$use_user_site" -eq 1 ]]; then
    if ! "$py_bin" -m pip install --user --break-system-packages -e "$ROOT_DIR/bindings/python" >/dev/null 2>&1; then
      "$py_bin" -m pip install --user -e "$ROOT_DIR/bindings/python" >/dev/null
    fi
    PYTHON_RUNNER="$py_bin"
    PYTHON_USER_SITE="$("$py_bin" -m site --user-site)"
  fi

  _PY_ENV_READY=1
}

run_c_example() {
  build_rust_core
  local shared_lib
  shared_lib="$(resolve_artifact "$SHARED_NAME")" || return 1
  local release_dir
  release_dir="$(dirname "$shared_lib")"
  local build_dir="$TARGET_ROOT/examples"
  mkdir -p "$build_dir"
  local exe_path="$build_dir/roundtrip_c${EXE_SUFFIX}"

  if have_cmd clang; then
    info "Compiling C harness with clang."
    local -a clang_args=(
      "-I$ROOT_DIR/include"
      "$ROOT_DIR/examples/c/roundtrip.c"
      "-L$release_dir"
      -lzkprov
    )
    local rpath_flag=""
    local -a extra_ldflags
    extra_ldflags=()
    case "$OS_NAME" in
      Linux*)
        rpath_flag="-Wl,-rpath,$release_dir"
        ;;
      Darwin*)
        rpath_flag="-Wl,-rpath,$release_dir"
        ;;
      MINGW*|MSYS*|CYGWIN*|Windows_NT)
        extra_ldflags=(-lws2_32 -luserenv -lntdll)
        ;;
      *)
        rpath_flag=""
        ;;
    esac
    if [[ -n "$rpath_flag" ]]; then
      clang_args+=("$rpath_flag")
    fi
    if [[ ${#extra_ldflags[@]} -gt 0 ]]; then
      clang_args+=("${extra_ldflags[@]}")
    fi
    clang_args+=(-o "$exe_path")
    clang "${clang_args[@]}"
  elif have_cmd cl; then
    info "Compiling C harness with cl."
    local src_win include_win out_win lib_dir_win
    src_win="$(cygpath -w "$ROOT_DIR/examples/c/roundtrip.c")"
    include_win="$(cygpath -w "$ROOT_DIR/include")"
    out_win="$(cygpath -w "$exe_path")"
    lib_dir_win="$(cygpath -w "$release_dir")"
    cmd.exe /C "cl /nologo /I\"$include_win\" \"$src_win\" /Fe:\"$out_win\" /link /LIBPATH:\"$lib_dir_win\" zkprov.lib ws2_32.lib userenv.lib ntdll.lib" >/dev/null
  else
    echo "Neither clang nor cl compiler is available for C example." >&2
    return 1
  fi

  info "Running C harness."
  case "$LIB_ENV" in
    PATH)
      env PATH="$release_dir:$PATH" "$exe_path"
      ;;
    *)
      env "$LIB_ENV=$release_dir" "$exe_path"
      ;;
  esac
}

run_node_example() {
  build_rust_core
  build_node_binding
  info "Installing Node example dependencies."
  (
    cd "$ROOT_DIR/examples/node"
    npm install
    info "Executing Node roundtrip example."
    node roundtrip.mjs
  )
}

run_python_example() {
  build_rust_core
  local shared_lib
  shared_lib="$(resolve_artifact "$SHARED_NAME")" || return 1
  setup_python_env
  info "Executing Python roundtrip example."
  if [[ -n "${PYTHON_USER_SITE:-}" ]]; then
    PYTHONPATH="${PYTHON_USER_SITE}${PYTHONPATH:+:$PYTHONPATH}" \
      ZKPROV_LIB="$shared_lib" "$PYTHON_RUNNER" "$ROOT_DIR/examples/python/roundtrip.py"
  else
    ZKPROV_LIB="$shared_lib" "$PYTHON_RUNNER" "$ROOT_DIR/examples/python/roundtrip.py"
  fi
}

run_flutter_example() {
  build_rust_core
  if ! have_cmd flutter; then
    echo "Flutter SDK is required for the Flutter example." >&2
    return 1
  fi
  local shared_lib
  shared_lib="$(resolve_artifact "$SHARED_NAME")" || return 1
  local example_dir="$ROOT_DIR/examples/flutter_app"
  info "Resolving Flutter dependencies."
  (
    cd "$example_dir"
    flutter --suppress-analytics pub get
  )
  info "Building Flutter APK (release)."
  (
    cd "$example_dir"
    ZKPROV_LIBRARY_PATH="$shared_lib" flutter --suppress-analytics build apk
  )
}

run_wasm_example() {
  build_rust_core
  info "Building wasm32-wasip1 artifact."
  "$CARGO_BIN" build -p zkprov-ffi-c --release --target wasm32-wasip1

  local wasm_path
  wasm_path="$(find "$TARGET_ROOT" -type f -path "*/wasm32-wasip1/release/zkprov.wasm" -print -quit 2>/dev/null || true)"
  if [[ -z "$wasm_path" ]]; then
    echo "WASM artifact not found under $TARGET_ROOT." >&2
    return 1
  fi

  info "Copying wasm artifact to bindings/wasm/zkprov_wasi.wasm."
  cp "$wasm_path" "$ROOT_DIR/bindings/wasm/zkprov_wasi.wasm"

  info "Executing WASM smoke test via Node."
  node "$ROOT_DIR/examples/wasm/smoke_node.mjs"
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

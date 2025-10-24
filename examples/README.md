# Examples

This directory contains language- and platform-specific integrations that exercise the proving
and verification APIs. Each example builds the relevant binding or artifact and performs a
round-trip proof verification using the toy AIR profile.

## Index

- **C roundtrip** (`examples/c/roundtrip.c`)
  - Build: `scripts/run_examples.sh` compiles the Rust FFI crate in release mode and then builds the standalone C harness, wiring up the shared library search path.
  - CI invocation: `scripts/run_examples.sh c`.
- **Node.js roundtrip** (`examples/node/roundtrip.mjs`)
  - Build: the helper script rebuilds the N-API addon with the freshly compiled static library and installs the example app dependencies before executing the roundtrip.
  - CI invocation: `scripts/run_examples.sh node`.
- **Python roundtrip** (`examples/python/roundtrip.py`)
  - Build: `scripts/run_examples.sh` installs the Python bindings (virtual environment when available, user site fall-back otherwise) and runs the CLI harness against the native library.
  - CI invocation: `scripts/run_examples.sh python`.
- **Flutter demo app** (`examples/flutter_app/`)
  - Build: the script triggers `flutter pub get` and `flutter build apk`, allowing CI to validate the mobile integration using prebuilt JNI stubs or a `ZKPROV_LIBRARY_PATH` override.
  - CI invocation: `scripts/run_examples.sh flutter`.
- **WASM smoke test** (`examples/wasm/smoke_node.mjs`)
  - Build: the script compiles the WASI target, copies the resulting module into `bindings/wasm`, and executes the Node smoke test to ensure the bindings produce a verified proof.
  - CI invocation: `scripts/run_examples.sh wasm`.

The helper script aggregates the results and prints concise PASS/FAIL lines for each example so that
CI can gate on the final status.

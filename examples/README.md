# Examples

This directory contains language- and platform-specific integrations that exercise the proving
and verification APIs. Each example has a placeholder implementation that will be fleshed out in
future releases.

## Index

- **C roundtrip** (`examples/c/roundtrip.c`)
  - Build: orchestrated by `scripts/run_examples.sh` (compiles the Rust core and then builds the C harness).
  - CI invocation: `scripts/run_examples.sh c` (called automatically by the top-level script).
- **Node.js roundtrip** (`examples/node/roundtrip.mjs`)
  - Build: handled by `scripts/run_examples.sh`, reusing the Rust build artifacts and installing Node dependencies when required.
  - CI invocation: `scripts/run_examples.sh node`.
- **Python roundtrip** (`examples/python/roundtrip.py`)
  - Build: the top-level script ensures the native library is present before running Python bindings.
  - CI invocation: `scripts/run_examples.sh python`.
- **Flutter demo app** (`examples/flutter_app/`)
  - Build: driven by `scripts/run_examples.sh`, which ensures Flutter and the native artifacts are available before the app is launched for integration tests.
  - CI invocation: `scripts/run_examples.sh flutter`.
- **WASM smoke test** (`examples/wasm/smoke_node.mjs`)
  - Build: executed by `scripts/run_examples.sh`, which builds the WebAssembly package and wires it into Node.js.
  - CI invocation: `scripts/run_examples.sh wasm`.

The helper script aggregates the results and prints concise PASS/FAIL lines for each example so that
CI can gate on the final status.

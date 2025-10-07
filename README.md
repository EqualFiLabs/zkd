# ZKProv — General-Purpose STARK Prover (Scaffold)

This is the workspace scaffold. It includes:
- `corelib`: shared library (registry, profiles)
- `backends/native`: stub backend
- `ffi-c`: C ABI (init/free)
- `cli` (`zkd`): command-line entry point

## Quickstart
```bash
cargo build
cargo test
cargo run -p zkd
cargo run -p zkd -- backend-ls
cargo run -p zkd -- profile-ls
```

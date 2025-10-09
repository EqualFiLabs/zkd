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

## Determinism Policy

**Determinism Policy**
ZKD guarantees proof-level determinism: same source, same inputs, same backend, same digest.
Prebuilt binaries are convenience builds only.
To verify determinism, rebuild from source and compare proof digests against the published golden vectors in `/tests/golden_vectors/`.

## Building shared libraries

The `zkprov-ffi-c` crate produces the shared library used by host applications.
The build commands below assume a release build; drop `--release` for debug
artifacts.

### Linux

```bash
cargo build -p zkprov-ffi-c --release
ls target/release/libzkprov.so
```

### macOS

```bash
cargo build -p zkprov-ffi-c --release
ls target/release/libzkprov.dylib
```

On Apple Silicon the default host triple already targets `aarch64-apple-darwin`.
For Intel Macs pass `--target x86_64-apple-darwin` to build for Rosetta.

### Android (NDK)

The workspace ships with a minimal cross-compilation stanza in
`.cargo/config.toml` that expects the Android NDK toolchain binaries to be on
`$PATH`. The commands below demonstrate building for `aarch64-linux-android`
with API level 21; adjust `darwin-x86_64` to `linux-x86_64` when running from a
Linux host.

```bash
export NDK_HOME="$HOME/Library/Android/sdk/ndk/26.1.10909125"
export TOOLCHAIN="$NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin"
export PATH="$TOOLCHAIN:$PATH"
export TARGET=aarch64-linux-android

cargo build -p zkprov-ffi-c --release --target $TARGET
ls target/$TARGET/release/libzkprov.so
```

When invoking the FFI from Android, provide absolute or asset-extracted paths;
the library deliberately avoids `std::fs::canonicalize` to keep asset lookups
under the caller's control.

## EVM interoperability harness

The EVM bridge expects the digest contract to compute

```
D = keccak256(
      abi.encode(
        backendIdHash:uint64,
        profileIdHash:uint64,
        pubioHash:uint64,
        bodyLen:uint64,
        body:bytes
      )
    )
```

Rust fixtures are encoded with [Alloy](https://github.com/alloy-rs/alloy)'s ABI support—ethers-rs
is intentionally **not** used in this flow. The parity harness bakes a native proof, emits
`meta.json`, `body.bin`, `digest.hex`, `meta.abi`, and `body.abi`, and then runs the Solidity stub
tests.

Reproduce the full flow locally:

```bash
cargo test --test evm_interop
(cd examples/evm_verifier && forge test -vv)
```

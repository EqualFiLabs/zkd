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

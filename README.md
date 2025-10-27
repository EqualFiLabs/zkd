# ZKD - General-Purpose STARK Prover

ZKD is a deterministic, backend-agnostic STARK proving engine that ships with a CLI, a stable C ABI, and maintained language bindings. It targets developers who need portable proofs across devices and stacks, with clear validation, golden vectors, and an emphasis on honest determinism at the proof layer rather than brittle binary reproducibility. The workspace contains the core library, adapters, crypto bundles, docs, tests, examples, and multi-language FFI bindings.

> Mantra: same input, same output, any backend.

---

## Table of Contents

* [Why ZKD](#why-zkd)
* [Features](#features)
* [Architecture](#architecture)
* [Backends and Capabilities](#backends-and-capabilities)
* [Determinism Policy](#determinism-policy)
* [Install](#install)
* [Quickstart](#quickstart)
* [CLI Reference](#cli-reference)
* [C ABI and Language Bindings](#c-abi-and-language-bindings)
* [WASM and Mobile](#wasm-and-mobile)
* [Pre-baked Profiles](#pre-baked-profiles)
* [EVM Interop](#evm-interop)
* [Recursion (Aggregations)](#recursion-aggregations)
* [Testing, Golden Vectors, and CI](#testing-golden-vectors-and-ci)
* [Roadmap](#roadmap)
* [Security and Threat Model](#security-and-threat-model)
* [Contributing](#contributing)
* [License](#license)

---

## Why ZKD

* **Deterministic by contract**: identical inputs with the same backend and profile produce identical proof digests. Determinism is validated at the proof layer with manifest hashing and golden vectors.  
* **Backend-agnostic**: a portable AIR-IR and adapter layer allow multiple engines to coexist without changing your AIR. Swap backends while preserving public output digests. 
* **Developer-friendly**: clean CLI, stable C ABI, and maintained bindings for Node, Python, Flutter/Dart, and WASI (Go/.NET/Java/Kotlin/Swift follow the bindings cookbook until official releases return). 
* **Privacy gadgets**: Pedersen commitments, range checks, and Merkle tools shipped as reusable bundles. 
* **Ready profiles**: drop-in AIRs for auth, allowlists, balance checks, age gates, and more. 

---

## Features

* **Portable AIR-IR** with YAML front matter and canonical encoding. 
* **Backend adapters** for a reference native path and real engines like Winterfell and Plonky families. 
* **Profile system** for predictable performance envelopes on desktop and mobile. 
* **C ABI** with six core functions exposed through safe bindings across languages. 
* **Commitments and crypto**: Poseidon2, Rescue, Merkle (arity 2 and 4), Keccak, Pedersen. 
* **Golden vectors and validation reports** wired into CI to catch drift.  

---

## Architecture

ZKD is a layered stack:

```
CLI / SDK / FFI
Coordinator (capability checks, orchestration)
AIR-IR (backend-neutral algebra)
Bundle engine (gadgets like Merkle, Range, Pedersen)
Backend adapters (native, Winterfell, Plonky2/3)
Crypto core (field math, FRI, Merkle, transcripts)
```

Responsibilities are strictly separated, which keeps proofs portable and the system extensible.  

---

## Backends and Capabilities

Adapters advertise a capability matrix, including fields, hashes, FRI arities, recursion support, and commitment gadgets. Example entries include `native`, `winterfell@0.6`, and the Plonky family. 

---

## Determinism Policy

ZKD guarantees proof-level determinism. Verify determinism by rebuilding from source and comparing proof digests against the golden vectors in `tests/golden_vectors`. Prebuilt binaries are convenience only. 

---

## Install

### Rust workspace

```bash
git clone https://github.com/EqualFiLabs/zkd
cd zkd
cargo build
cargo test
```

### Build shared libraries

Linux:

```bash
cargo build -p zkprov-ffi-c --release
ls target/release/libzkprov.so
```

macOS:

```bash
cargo build -p zkprov-ffi-c --release
ls target/release/libzkprov.dylib
```

Android (NDK), with toolchain on PATH:

```bash
export NDK_HOME="$HOME/Library/Android/sdk/ndk/26.1.10909125"
export TOOLCHAIN="$NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin"
export PATH="$TOOLCHAIN:$PATH"
export TARGET=aarch64-linux-android

cargo build -p zkprov-ffi-c --release --target $TARGET
ls target/$TARGET/release/libzkprov.so
```

---

## Quickstart

CLI:

```bash
cargo run -p zkd -- backend-ls
cargo run -p zkd -- profile-ls
```

Minimal prove and verify for a toy program (example paths):

```bash
zkd prove  -p tests/fixtures/toy.air -i tests/fixtures/toy.json -b native --profile balanced -o /tmp/toy.proof
zkd verify -p tests/fixtures/toy.air -i tests/fixtures/toy.json -b native --profile balanced -P /tmp/toy.proof
```

Expected: proof verifies with stats printed. 

---

## CLI Reference

Core commands:

* `zkd compile` converts YAML into canonical AIR.
* `zkd prove` runs the selected backend under a given profile.
* `zkd verify` replays the transcript deterministically.
* `zkd profile ls` lists available profiles.
* `zkd backend ls` shows registered adapters and capabilities.
* `zkd vector validate` enforces golden vector parity. 

Exit codes and common flags are documented in `docs/INTERFACES.md`. 

---

## C ABI and Language Bindings

Exported symbols:

* `zkp_init`, `zkp_prove`, `zkp_verify`, `zkp_list_backends`, `zkp_list_profiles`, `zkp_free`
* plus helpers like `zkp_version`, `zkp_set_callback`, `zkp_cancel` for richer integrations.
  Error returns are UTF-8 JSON blobs that callers must free via `zkp_free`. The context is thread safe and supports concurrent prove and verify.   

Official bindings ship for Node, Python, Flutter/Dart, and WASI. Go, .NET, Java/Kotlin, and Swift rely on the DIY bindings cookbook until official packages return. Each wrapper maps the C ABI into idiomatic APIs and types. 

---

## WASM and Mobile

* **WASI target** exports the same C ABI for serverless and browsers. A small JS loader exposes `prove` and `verify` that mirror the Node API. 
* **Flutter plugin** wraps the shared library for Android and iOS and includes finalizers that call `zkp_free`. Cookbook guidance covers minimal Swift overlays until the Ecosystem phase adds an official package. 

---

## Pre-baked Profiles

ZKD ships a catalog of ready profiles:

* `zk-auth-pedersen-secret` for passwordless auth
* `zk-allowlist-merkle` for set membership
* `zk-attr-range` for bounded attributes
* `zk-balance-geq` for balance threshold
* `zk-age-over` for simple mobile age gates
* and more, each with a defined public IO schema and gadgets.  

Profiles share a canonical digest rule that binds `profile_id`, `version`, public inputs, and proof bytes:

```
D = H(profile_id || profile_version || canon_pub_inputs || proof_bytes)
```

This ensures deterministic cross-backend behavior. 

---

## EVM Interop

ZKD emits a Solidity-ready digest `D` that a simple verifier stub can check on chain. The digest uses Solidity ABI packing of:

```
(uint64 backendIdHash, uint64 profileIdHash, uint64 pubioHash, uint64 bodyLen, bytes body)
```

and then Keccak256. There is a Foundry stub and parity harness in the repo. 
To reproduce locally:

```bash
cargo test --test evm_interop
(cd examples/evm_verifier && forge test -vv)
```

---

## Recursion (Aggregations)

ZKD defines a backend-neutral Recursion IR so outer proofs can verify multiple inner proofs and reduce them to a single digest. Modes include stark-in-stark with Plonky adapters and an optional SNARK wrapper later. Mobile profiles cap rows and the number of inner proofs to avoid OOM.

---

## Testing, Golden Vectors, and CI

The test plan covers unit, integration, cross-backend parity, negative tests, fuzzing, and performance budgets. CI enforces vector parity and emits validation reports. Targets include native and Winterfell in Phase 0, with Plonky adapters added later.  

> Determinism tests assert equal seeds, headers, and digest D across runs and hosts. Failures raise specific drift errors. 

---

## Roadmap

* Phase 0: foundation, commitments, EVM interop, C ABI, bindings, docs.
* Phase 1: portability with Plonky2 and mobile profiles.
* Phase 2: acceleration, Plonky3, recursion execution, optional SNARK wrapper.
* Phase 3: service mode, Docker, SDKs, metrics, cache, auth.
* Phase 4: registry, docs site, public CI matrix, security hardening, v1.0 packaging.  

For a detailed task map and acceptance gates, see `docs/TASKLIST.md`. 

---

## Security and Threat Model

* Proof integrity depends on deterministic transcripts and validated manifests.
* Commitment gadgets bind public outputs to constraints and enforce curve checks and blinding rules.
* Golden Vectors and manifest hashes provide a reproducibility oracle to spot supply chain tampering.  

Read `docs/threat-model.md` for complete coverage. 

---

## Contributing

Issues and PRs are welcome. Please include:

* a concise problem statement
* reproducible steps or fixtures
* CI-ready tests and updated docs where relevant

See the test plan and interfaces docs for expected outputs and error contracts.  

---

## License

MIT © EqualFi Labs.